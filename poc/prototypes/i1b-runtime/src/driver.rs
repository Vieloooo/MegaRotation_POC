use crate::{
    ClockSample, Error, Id, Profile, Result, RngCheckpoint, RoleRng, SimContext, Timer,
    clock::NodeClock,
    storage::{
        CompletedIo, DiskQueue, IoDisposition, IoIdentity, IoJob, IoOperation, IoResult, IoTarget,
        StorageAccess,
    },
};
use crate::{
    witness::{NativeWire, Submission, WitnessPolicy, WitnessRuntime},
    witness_ledger::{LedgerEntry, LedgerRecord, WitnessFault, WitnessHead, WitnessLedger},
};
use commonware_runtime::{
    Handle, Supervisor as _,
    signal::Stopper,
    tokio::{Config, Context, ControlledJob, Runner},
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    future::{Future, poll_fn},
    panic::{AssertUnwindSafe, catch_unwind},
    pin::pin,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context as TaskContext, Poll, Wake, Waker},
    thread::{self, ThreadId},
};

const MAX_POLLS: u64 = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grant {
    pub id: u64,
    pub tick_us: u64,
}

#[derive(Clone, Debug)]
pub enum Input {
    None,
    RequestSample,
    DeliverSample(ClockSample),
    Timer(Timer),
    Close(Id),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Placement {
    Shared,
    Blocking,
    Dedicated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    Grant {
        grant: u64,
        tick_us: u64,
    },
    SampleRequested {
        request: u64,
    },
    SampleAccepted {
        request: u64,
        sequence: u64,
    },
    SampleRejected {
        request: u64,
        reason: &'static str,
    },
    TimerArmed {
        timer: Timer,
    },
    TimerTransferred {
        timer: Timer,
        from: u64,
        to: u64,
    },
    TimerFired {
        timer: Timer,
    },
    TimerSuppressed {
        timer: Timer,
    },
    TimerCancelled {
        timer: Timer,
    },
    Retired {
        cid: Id,
        reason: &'static str,
    },
    TaskRegistered {
        task: u64,
        parent: u64,
        cid: Id,
        role: Id,
        placement: Placement,
    },
    Construct {
        task: u64,
    },
    Poll {
        task: u64,
        grant: u64,
        sequence: u64,
    },
    Cancel {
        task: u64,
    },
    Dispose {
        task: u64,
        grant: u64,
        sequence: u64,
        cleanup: bool,
    },
    ClockRead {
        caller: String,
        snapshot: ClockAudit,
    },
    RngDraw {
        task: u64,
        role: Id,
        words: u64,
    },
    Failure {
        reason: &'static str,
    },
    IoIssued {
        identity: IoIdentity,
        target: IoTarget,
        operation: IoOperation,
    },
    IoSyncResponseLost {
        io_id: u64,
    },
    IoReady {
        io_id: u64,
        result: IoResult,
    },
    IoReleased {
        io_id: u64,
        delivered: bool,
    },
    IoDiscarded {
        io_id: u64,
    },
    WitnessCommitted {
        io_id: u64,
        sequence: u64,
        head_hash: Id,
    },
    WitnessSubmitted {
        submission: Submission,
        wire: NativeWire,
    },
    CleanupBoundExceeded,
    CleanupComplete,
    Ack {
        grant: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub grant: Grant,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ClockAudit {
    pub cid: Id,
    pub grant_id: u64,
    pub task_id: u64,
    pub tick_us: u64,
    pub local_time_us: Option<i64>,
    pub sample_sequence: u64,
    pub lease_end_tick_us: u64,
    pub local_stop_us: Option<i64>,
}

#[derive(Clone)]
pub(crate) struct Authority {
    pub shared: Weak<Shared>,
    pub cid: Id,
    pub role: String,
}

impl Authority {
    pub fn clock_read(&self, caller: &str) -> ClockAudit {
        self.shared
            .upgrade()
            .expect("native task owner")
            .audit_clock(self.cid, &format!("{}/{}", self.role, caller))
    }

    pub fn check(&self) -> Result<ClockAudit> {
        let shared = self
            .shared
            .upgrade()
            .ok_or(Error("native task owner closed"))?;
        shared.try_ordinary(Some(self.cid))?;
        Ok(shared.audit_clock(self.cid, &format!("{}/authority", self.role)))
    }

    pub fn snapshot(&self) -> ClockAudit {
        self.shared
            .upgrade()
            .map_or_else(ClockAudit::default, |s| s.snapshot_clock(self.cid))
    }

    pub fn stage_route(&self, wire: NativeWire, dest: u32) -> Result<()> {
        let shared = self
            .shared
            .upgrade()
            .ok_or(Error("native task owner closed"))?;
        shared.try_ordinary(Some(self.cid))?;
        shared.inner.lock().unwrap().actor_routes.push((wire, dest));
        Ok(())
    }
}

pub(crate) struct ArchiveAuthority {
    shared: Weak<Shared>,
    cid: Id,
}

impl ArchiveAuthority {
    pub fn check(&self) -> Result<()> {
        let state = self
            .shared
            .upgrade()
            .ok_or(Error("archive driver closed"))?;
        let permit = state.try_ordinary(None)?;
        if permit.task != 0
            || permit.cid.is_some()
            || state
                .audit_clock(self.cid, "archive/clock")
                .local_time_us
                .is_none()
        {
            return Err(Error("archive send requires a calibrated root grant"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Permit {
    grant: u64,
    task: u64,
    sequence: u64,
    cid: Option<Id>,
    mode: PermitMode,
    thread: ThreadId,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PermitMode {
    Ordinary,
    IoRelease,
    Cleanup,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TaskPhase {
    Enrolled,
    Started,
    Completed,
}

struct Job {
    native: ControlledJob,
    cid: Id,
    phase: Arc<Mutex<TaskPhase>>,
    sequence: u64,
}

pub(crate) struct Session {
    stop_local_us: i64,
    retired: bool,
}

struct TimerWait {
    timer: Timer,
    owner: u64,
    waker: Arc<Waker>,
    fired: bool,
}

pub(crate) struct Shared {
    pub(crate) inner: Mutex<Inner>,
    pub(crate) failure: Arc<AtomicBool>,
}

pub(crate) struct Inner {
    clock: NodeClock,
    current: Option<Grant>,
    queued_io: BTreeMap<u64, IoJob>,
    next_io: u64,
    native_clock_audit: bool,
    actor_routes: Vec<(NativeWire, u32)>,
    witness_partition: Option<String>,
    permit: Option<Permit>,
    sessions: BTreeMap<Id, Session>,
    enrolled: BTreeMap<u64, Job>,
    ready: BTreeSet<u64>,
    live: BTreeSet<u64>,
    next_task: u64,
    timers: BTreeMap<u64, TimerWait>,
    next_timer: u64,
    events: Vec<Event>,
    rngs: BTreeMap<(Id, Id), RoleRng>,
}

struct Ready {
    shared: Weak<Shared>,
    task: u64,
}

impl Wake for Ready {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        if let Some(shared) = self.shared.upgrade() {
            let mut inner = shared.inner.lock().unwrap();
            if !inner.live.contains(&self.task) {
                return;
            }
            if inner
                .permit
                .is_some_and(|permit| permit.thread == thread::current().id())
            {
                inner.ready.insert(self.task);
            } else {
                shared.failure.store(true, Ordering::SeqCst);
                inner.events.push(Event::Failure {
                    reason: "task wake outside controlled permit",
                });
            }
        }
    }
}

pub struct ControlledDriver {
    native: Arc<Context>,
    native_storage_root: std::path::PathBuf,
    native_creation_layout: commonware_runtime::BlobLayout,
    disk: DiskQueue,
    witness: Option<WitnessRuntime>,
    evidence: Option<EvidenceIo>,
    witness_fault: Option<(u64, WitnessFault)>,
    lost_sync_response: Option<u64>,
    witness_hold: Option<(u64, WitnessHold)>,
    staged_submissions: BTreeMap<Id, (Submission, NativeWire)>,
    io_wall_pause: std::time::Duration,
    fail_io_release: Option<u64>,
    io_hold: Option<(u64, tokio::sync::oneshot::Receiver<()>)>,
    shared: Arc<Shared>,
    jobs: BTreeMap<u64, Job>,
    sessions: BTreeMap<Id, Handle<()>>,
    last_grant: u64,
    poll_limit: u64,
    cleanup_limit: usize,
    stopped: bool,
}

impl ControlledDriver {
    /// Run a synchronous controller inside a native Runner. The root owns the
    /// registry across controller errors and panics and drains it before return.
    pub fn run<T>(
        config: Config,
        profile: Profile,
        controller: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        Self::run_async(config, profile, async |driver| controller(driver))
    }

    /// The controller can await transport here. The returned future borrows the
    /// driver, so disconnect or panic cannot transfer registry ownership away.
    pub fn run_async<T>(
        config: Config,
        profile: Profile,
        controller: impl AsyncFnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        use commonware_runtime::Runner as _;
        use futures::FutureExt as _;
        let native_storage_root = config.storage_directory().clone();
        let native_creation_layout = *config.storage_blob_layouts().end();
        Runner::new(config.with_catch_panics(true)).start(move |native| async move {
            let mut driver =
                Self::new(native, profile, native_storage_root, native_creation_layout)?;
            let result = AssertUnwindSafe(async { controller(&mut driver).await })
                .catch_unwind()
                .await;
            let result = match result {
                Ok(result) => result,
                Err(_) => Err(Error("controller panic")),
            };
            if let Err(error) = &result {
                driver.fail(error.0);
            }
            let cleanup = driver.shutdown_async().await;
            result.and_then(|value| cleanup.map(|()| value))
        })
    }

    fn new(
        native: Context,
        profile: Profile,
        native_storage_root: std::path::PathBuf,
        native_creation_layout: commonware_runtime::BlobLayout,
    ) -> Result<Self> {
        let shared = Arc::new(Shared {
            failure: Arc::new(AtomicBool::new(false)),
            inner: Mutex::new(Inner {
                clock: NodeClock::new(profile)?,
                current: None,
                queued_io: BTreeMap::new(),
                next_io: 0,
                native_clock_audit: false,
                actor_routes: vec![],
                witness_partition: None,
                permit: None,
                sessions: BTreeMap::new(),
                enrolled: BTreeMap::new(),
                ready: BTreeSet::new(),
                live: BTreeSet::new(),
                next_task: 0,
                timers: BTreeMap::new(),
                next_timer: 0,
                events: Vec::new(),
                rngs: BTreeMap::new(),
            }),
        });
        Ok(Self {
            native: Arc::new(native),
            native_storage_root,
            native_creation_layout,
            disk: DiskQueue::default(),
            witness: None,
            evidence: None,
            witness_fault: None,
            lost_sync_response: None,
            witness_hold: None,
            staged_submissions: BTreeMap::new(),
            io_wall_pause: std::time::Duration::ZERO,
            fail_io_release: None,
            io_hold: None,
            shared,
            jobs: BTreeMap::new(),
            sessions: BTreeMap::new(),
            last_grant: 0,
            poll_limit: MAX_POLLS,
            cleanup_limit: MAX_POLLS as usize,
            stopped: false,
        })
    }

    pub fn with_limits(&mut self, polls: u64, disposals: usize) {
        assert_eq!(self.last_grant, 0, "limits are fixed before first grant");
        self.poll_limit = polls;
        self.cleanup_limit = disposals;
    }

    /// Inject a diagnostic wall pause before each physical operation.
    pub fn set_io_wall_pause(&mut self, pause: std::time::Duration) -> Result<()> {
        self.configure_io(true)?;
        self.io_wall_pause = pause;
        Ok(())
    }

    /// Inject a failure after this physical completion is retained and before release.
    pub fn fail_before_io_release(&mut self, io_id: u64) -> Result<()> {
        self.configure_io(io_id > 0 && self.fail_io_release.is_none())?;
        self.fail_io_release = Some(io_id);
        Ok(())
    }

    /// Hold this operation after installing the active slot and before dispatch.
    pub fn hold_io_before_dispatch(
        &mut self,
        io_id: u64,
    ) -> Result<tokio::sync::oneshot::Sender<()>> {
        self.configure_io(io_id > 0 && self.io_hold.is_none())?;
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.io_hold = Some((io_id, rx));
        Ok(tx)
    }

    fn configure_io(&mut self, valid: bool) -> Result<()> {
        if self.last_grant != 0 || !valid {
            return Err(Error("invalid or late diagnostic I/O configuration"));
        }
        Ok(())
    }

    /// Active operation held by the driver, including while the controller is paused.
    pub fn active_io_id(&self) -> Option<u64> {
        self.disk.active.as_ref().map(|job| job.identity.io_id)
    }

    pub fn configure_witness(
        &mut self,
        policy: WitnessPolicy,
        path: &std::path::Path,
    ) -> Result<()> {
        self.configure_io(self.witness.is_none())?;
        let config = &policy.seal().config;
        let profile = self.shared.inner.lock().unwrap().clock.profile.clone();
        if config.run_id != profile.run_id
            || config.config_hash != profile.config_hash
            || config.worker != profile.node
        {
            return Err(Error("witness policy does not match worker profile"));
        }
        crate::witness::require(
            self.native_creation_layout == commonware_runtime::BlobLayout::V1,
            "selected witness storage must create V1 blobs",
        )?;
        crate::witness_recovery::validate_existing_layout(&self.native_storage_root, &policy)?;
        let ledger = WitnessLedger::create(path, policy.clone())?;
        self.shared.inner.lock().unwrap().witness_partition =
            Some(policy.seal().config.partition.clone());
        self.witness = Some(WitnessRuntime::new(policy, ledger));
        Ok(())
    }

    pub fn witness_records(&self) -> Option<&[LedgerRecord]> {
        self.witness
            .as_ref()
            .map(|witness| witness.ledger.records())
    }

    pub fn witness_head(&self) -> Option<WitnessHead> {
        self.witness.as_ref().map(|witness| witness.ledger.head())
    }

    /// Diagnostic: replace a successful selected full sync response with Closed.
    /// Physical work still completes. Selected evidence is marked uncertain.
    pub fn inject_lost_sync_response(&mut self, io_id: u64) -> Result<()> {
        self.configure_io(io_id > 0 && self.lost_sync_response.is_none())?;
        self.lost_sync_response = Some(io_id);
        Ok(())
    }

    pub fn inject_witness_fault(&mut self, io_id: u64, fault: WitnessFault) -> Result<()> {
        self.configure_io(io_id > 0 && self.witness_fault.is_none())?;
        self.witness_fault = Some((io_id, fault));
        Ok(())
    }

    pub fn hold_witness_before_persist(
        &mut self,
        io_id: u64,
    ) -> Result<(
        tokio::sync::oneshot::Receiver<()>,
        tokio::sync::oneshot::Sender<()>,
    )> {
        self.configure_io(io_id > 0 && self.witness_hold.is_none())?;
        let (entered, observe) = tokio::sync::oneshot::channel();
        let (release, wait) = tokio::sync::oneshot::channel();
        self.witness_hold = Some((io_id, WitnessHold { entered, wait }));
        Ok((observe, release))
    }

    pub fn active_evidence_id(&self) -> Option<u64> {
        self.evidence.as_ref().map(|evidence| evidence.io_id)
    }

    pub fn stage_witnessed_send(&mut self, wire: NativeWire, destinations: &[u32]) -> Result<()> {
        self.shared.try_ordinary(None)?;
        let witness = self
            .witness
            .as_ref()
            .ok_or(Error("witness policy not configured"))?;
        let eligible = witness
            .ledger
            .index
            .payload(&wire)
            .ok_or(Error("native payload lacks durable witness"))?;
        crate::witness::require(
            !destinations.is_empty()
                && destinations
                    .iter()
                    .all(|dest| *dest < 5 && *dest != witness.policy.seal.config.member),
            "invalid witnessed destinations",
        )?;
        for dest in destinations {
            let submission = Submission::new(eligible.id, *dest);
            self.staged_submissions
                .insert(submission.id, (submission, wire.clone()));
        }
        Ok(())
    }

    pub fn events(&self) -> Vec<Event> {
        self.shared.inner.lock().unwrap().events.clone()
    }

    pub fn fail_transport(&mut self) -> Result<()> {
        self.fail("transport disconnected");
        Err(Error("transport disconnected"))
    }

    pub(crate) fn enable_native_clock_audit(&mut self) {
        self.shared.inner.lock().unwrap().native_clock_audit = true;
    }

    pub(crate) fn archive_authority(&self, cid: Id) -> ArchiveAuthority {
        ArchiveAuthority {
            shared: Arc::downgrade(&self.shared),
            cid,
        }
    }

    pub(crate) fn native_manager_time(&self, cid: Id) -> Result<i64> {
        self.shared.try_ordinary(None)?;
        self.shared
            .audit_clock(cid, "manager/clock")
            .local_time_us
            .ok_or(Error("manager clock unsafe"))
    }

    pub(crate) fn native_clock_ready(&self) -> bool {
        self.shared.inner.lock().unwrap().clock.local().is_some()
    }

    pub(crate) fn native_witness_for(
        &self,
        wire: &NativeWire,
        destination: u32,
    ) -> Result<crate::native_wire::LocalWitness> {
        let ledger = &self
            .witness
            .as_ref()
            .ok_or(Error("missing witness policy"))?
            .ledger;
        let payload = ledger
            .index
            .payload(wire)
            .ok_or(Error("missing exact native wire witness"))?;
        let route = Submission::new(payload.id, destination);
        if !ledger.index.submissions.contains_key(&route.id) {
            return Err(Error("uncommitted native route"));
        }
        let head = ledger.head();
        Ok(crate::native_wire::LocalWitness {
            witness_ids: vec![payload.witness_id.to_vec()],
            authorization_id: route.id.to_vec(),
            receipt: Some(crate::native_wire::Receipt {
                sequence: head.sequence,
                hash: head.hash.to_vec(),
            }),
        })
    }

    pub fn manager_time_us(&self) -> Result<i64> {
        self.shared.try_ordinary(None)?;
        self.shared
            .inner
            .lock()
            .unwrap()
            .clock
            .local()
            .ok_or(Error("ClockUnsafe"))
    }

    pub fn grant(
        &mut self,
        grant: Grant,
        input: Input,
        action: impl FnOnce(&mut Self) -> Result<()>,
    ) -> Result<Step> {
        self.check_grant_entry()?;
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let start = self.begin_grant(grant, input, action)?;
            let mut polls = 0;
            self.poll_ready(&mut polls)?;
            if self.has_io() {
                return Err(Error("synchronous grant issued physical I/O"));
            }
            if !self.staged_submissions.is_empty() {
                return Err(Error("synchronous grant staged native publication"));
            }
            self.finish_grant(grant, start)
        }));
        let result = outcome.unwrap_or(Err(Error("driver panic")));
        if let Err(error) = &result {
            self.fail(error.0);
        }
        result
    }

    /// Execute one fixed-tick grant, retaining all physical I/O custody in the driver.
    pub async fn grant_async(
        &mut self,
        grant: Grant,
        input: Input,
        action: impl FnOnce(&mut Self) -> Result<()>,
    ) -> Result<Step> {
        use futures::FutureExt as _;
        self.check_grant_entry()?;
        let mut custody = GrantCustody {
            shared: self.shared.clone(),
            grant,
            armed: true,
        };
        let result = AssertUnwindSafe(async {
            let start = self.begin_grant(grant, input, action)?;
            let mut polls = 0;
            loop {
                self.poll_ready(&mut polls)?;
                if !self.has_io() {
                    break;
                }
                self.await_io(PermitMode::IoRelease).await?;
                if self.shared.failed() {
                    return Err(Error("native task failure"));
                }
                self.await_evidence(PermitMode::IoRelease).await?;
                self.release_io(PermitMode::IoRelease);
            }
            let routes = std::mem::take(&mut self.shared.inner.lock().unwrap().actor_routes);
            for (wire, destination) in routes {
                let permit = self.enter_permit(0, 0, None, PermitMode::Ordinary);
                self.stage_witnessed_send(wire, &[destination])?;
                drop(permit);
            }
            self.commit_submissions().await?;
            self.finish_grant(grant, start)
        })
        .catch_unwind()
        .await
        .unwrap_or(Err(Error("driver panic")));
        custody.armed = false;
        if let Err(error) = &result {
            self.fail(error.0);
        }
        result
    }

    fn check_grant_entry(&mut self) -> Result<()> {
        if self.has_active_permit() {
            self.fail("nested grant");
            return Err(Error("nested grant"));
        }
        if self.stopped {
            return Err(Error("driver stopped"));
        }
        if self.shared.failed() {
            self.fail("native task failure");
            return Err(Error("native task failure"));
        }
        Ok(())
    }

    fn begin_grant(
        &mut self,
        grant: Grant,
        input: Input,
        action: impl FnOnce(&mut Self) -> Result<()>,
    ) -> Result<usize> {
        let (start, wake) = {
            let mut inner = self.shared.inner.lock().unwrap();
            if grant.id <= self.last_grant || grant.tick_us < inner.clock.tick_us {
                return Err(Error("invalid grant coordinates"));
            }
            self.last_grant = grant.id;
            inner.clock.tick_us = grant.tick_us;
            inner.current = Some(grant);
            let start = inner.events.len();
            inner.events.push(Event::Grant {
                grant: grant.id,
                tick_us: grant.tick_us,
            });
            inner.check_sessions();
            (start, inner.apply(input)?)
        };
        let permit = self.enter_permit(0, 0, None, PermitMode::Ordinary);
        if let Some(waker) = wake {
            waker.wake_by_ref();
        }
        drop(permit);
        self.dispose_retired();
        if self.shared.failed() {
            return Err(Error("native task failure"));
        }
        let permit = self.enter_permit(0, 0, None, PermitMode::Ordinary);
        let result = action(self);
        drop(permit);
        result?;
        Ok(start)
    }

    fn poll_ready(&mut self, polls: &mut u64) -> Result<()> {
        loop {
            if self.shared.failed() {
                return Err(Error("native task failure"));
            }
            self.import_jobs();
            let next = self.shared.inner.lock().unwrap().ready.pop_first();
            let Some(id) = next else { return Ok(()) };
            if *polls >= self.poll_limit {
                return Err(Error("logical poll limit exceeded"));
            }
            *polls += 1;
            self.poll_job(id)?;
            self.dispose_retired();
        }
    }

    fn finish_grant(&mut self, grant: Grant, start: usize) -> Result<Step> {
        let mut inner = self.shared.inner.lock().unwrap();
        if self.shared.failed() {
            return Err(Error("native task failure"));
        }
        inner.current = None;
        for (_, (submission, wire)) in std::mem::take(&mut self.staged_submissions) {
            inner
                .events
                .push(Event::WitnessSubmitted { submission, wire });
        }
        inner.events.push(Event::Ack { grant: grant.id });
        Ok(Step {
            grant,
            events: inner.events[start..].to_vec(),
        })
    }

    fn has_io(&self) -> bool {
        self.disk.active.is_some()
            || self.disk.completed.is_some()
            || self.evidence.is_some()
            || !self.shared.inner.lock().unwrap().queued_io.is_empty()
    }

    fn poll_io(&mut self, cx: &mut TaskContext<'_>, disposal_mode: PermitMode) -> Poll<Result<()>> {
        if self.disk.completed.is_some() {
            return Poll::Ready(Ok(()));
        }
        if self.disk.active.is_none() {
            let next = self.shared.inner.lock().unwrap().queued_io.pop_first();
            let Some((_, mut job)) = next else {
                return Poll::Ready(Err(Error("missing physical I/O")));
            };
            if self
                .io_hold
                .as_ref()
                .is_some_and(|(id, _)| *id == job.identity.io_id)
            {
                let (_, hold) = self.io_hold.take().expect("configured I/O hold");
                let future = job.future;
                job.future = Box::pin(async move {
                    hold.await
                        .expect("physical I/O hold released by the controller");
                    future.await
                });
            }
            if !self.io_wall_pause.is_zero() {
                let pause = self.io_wall_pause;
                let future = job.future;
                job.future = Box::pin(async move {
                    tokio::time::sleep(pause).await;
                    future.await
                });
            }
            self.disk.active = Some(job);
            if let Some(active) = self.disk.active.as_ref() {
                if let (Some(witness), Some(io)) = (&self.witness, &active.witness) {
                    if let Err(error) = witness.preflight(&active.identity, io) {
                        let active = self.disk.active.take().expect("active selected operation");
                        let id = active.identity.clone();
                        let permit =
                            self.enter_permit(id.task, id.io_id, Some(id.cid), PermitMode::Cleanup);
                        if catch_unwind(AssertUnwindSafe(|| drop(active))).is_err() {
                            self.shared.record_failure("selected I/O disposal panic");
                        }
                        self.shared.event(Event::IoDiscarded { io_id: id.io_id });
                        drop(permit);
                        return Poll::Ready(Err(error));
                    }
                }
            }
        }
        let identity = self
            .disk
            .active
            .as_ref()
            .expect("driver owns active operation")
            .identity
            .clone();
        let permit = self.enter_permit(
            identity.task,
            identity.io_id,
            Some(identity.cid),
            disposal_mode,
        );
        let job = self
            .disk
            .active
            .as_mut()
            .expect("driver owns active operation");
        let outcome = catch_unwind(AssertUnwindSafe(|| job.future.as_mut().poll(cx)));
        drop(permit);
        match outcome {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready((result, release, opened))) => {
                let mut witness = self
                    .disk
                    .active
                    .as_mut()
                    .expect("completed active operation")
                    .witness
                    .take();
                if let (Some(io), Some(native)) = (&mut witness, opened) {
                    io.native = Some(native);
                }
                let lost_sync_response = self.lost_sync_response == Some(identity.io_id);
                let result = if lost_sync_response {
                    self.lost_sync_response = None;
                    if result == IoResult::Ok
                        && witness
                            .as_ref()
                            .is_some_and(|io| matches!(io.operation, IoOperation::Sync { .. }))
                    {
                        self.shared.event(Event::IoSyncResponseLost {
                            io_id: identity.io_id,
                        });
                        IoResult::Error { kind: "closed" }
                    } else {
                        self.shared.record_failure(
                            "lost-sync diagnostic did not target a successful selected sync",
                        );
                        result
                    }
                } else {
                    result
                };
                self.disk.completed = Some(CompletedIo {
                    identity: identity.clone(),
                    result,
                    release,
                    witness,
                    lost_sync_response,
                });
                let completed = self.disk.completed.as_ref().expect("stored completion");
                self.shared.event(Event::IoReady {
                    io_id: identity.io_id,
                    result: completed.result.clone(),
                });
                let active = self.disk.active.take().expect("active physical operation");
                let permit = self.enter_permit(
                    identity.task,
                    identity.io_id,
                    Some(identity.cid),
                    disposal_mode,
                );
                let disposed = catch_unwind(AssertUnwindSafe(|| drop(active)));
                drop(permit);
                if self.fail_io_release == Some(identity.io_id) {
                    self.fail_io_release = None;
                    self.shared
                        .record_failure("scripted I/O failure before release");
                }
                if disposed.is_err() {
                    self.shared
                        .record_failure("physical I/O future disposal panic");
                }
                Poll::Ready(disposed.map_err(|_| Error("physical I/O future disposal panic")))
            }
            Err(_) => {
                self.shared.record_failure("physical I/O poll panic");
                let active = self.disk.active.take().expect("active physical operation");
                let id = active.identity.clone();
                let permit =
                    self.enter_permit(id.task, id.io_id, Some(id.cid), PermitMode::Cleanup);
                if catch_unwind(AssertUnwindSafe(|| drop(active))).is_err() {
                    self.shared
                        .record_failure("physical I/O future disposal panic");
                }
                self.shared.event(Event::IoDiscarded { io_id: id.io_id });
                drop(permit);
                Poll::Ready(Err(Error("physical I/O poll panic")))
            }
        }
    }

    async fn await_io(&mut self, disposal_mode: PermitMode) -> Result<()> {
        poll_fn(|cx| self.poll_io(cx, disposal_mode)).await
    }

    async fn await_evidence(&mut self, mode: PermitMode) -> Result<()> {
        if self.evidence.is_none() {
            let Some(completed) = self.disk.completed.as_mut() else {
                return Ok(());
            };
            let Some(io) = completed.witness.take() else {
                return Ok(());
            };
            let witness = self
                .witness
                .take()
                .ok_or(Error("selected I/O lacks witness runtime"))?;
            let identity = completed.identity.clone();
            let io_id = identity.io_id;
            let result = completed.result.clone();
            let fault = self
                .witness_fault
                .filter(|(id, _)| *id == io_id)
                .map(|(_, fault)| fault);
            let hold = self
                .witness_hold
                .as_ref()
                .is_some_and(|(id, _)| *id == io_id)
                .then(|| self.witness_hold.take().expect("witness hold").1);
            let before = witness.ledger.records().len();
            self.evidence = Some(EvidenceIo {
                io_id,
                before,
                future: Box::pin(async move {
                    if let Some(hold) = hold {
                        let _ = hold.entered.send(());
                        if hold.wait.await.is_err() {
                            return (witness, Err(Error("witness hold abandoned")));
                        }
                    }
                    witness.observe(identity, io, result, fault).await
                }),
            });
        }
        self.complete_evidence(mode).await
    }

    async fn complete_evidence(&mut self, mode: PermitMode) -> Result<()> {
        poll_fn(|cx| {
            let Some(evidence) = self.evidence.as_ref() else {
                return Poll::Ready(Ok(()));
            };
            let io_id = evidence.io_id;
            let before = evidence.before;
            let permit = self.enter_permit(0, io_id, None, mode);
            let evidence = self.evidence.as_mut().expect("owned witness future");
            let result = catch_unwind(AssertUnwindSafe(|| evidence.future.as_mut().poll(cx)));
            drop(permit);
            match result {
                Ok(Poll::Pending) => Poll::Pending,
                Ok(Poll::Ready((witness, result))) => {
                    let sequence = witness.ledger.records().len();
                    let head_hash = witness.ledger.head_hash();
                    self.witness = Some(witness);
                    let disposal = self.drop_evidence(mode);
                    if sequence > before {
                        self.shared.event(Event::WitnessCommitted {
                            io_id,
                            sequence: sequence as u64,
                            head_hash,
                        });
                    }
                    Poll::Ready(result.and(disposal))
                }
                Err(_) => {
                    let _ = self.drop_evidence(PermitMode::Cleanup);
                    Poll::Ready(Err(Error("witness operation panic")))
                }
            }
        })
        .await
    }

    fn drop_evidence(&mut self, mode: PermitMode) -> Result<()> {
        let permit = self.enter_permit(0, 0, None, mode);
        let evidence = self.evidence.take();
        let disposed = catch_unwind(AssertUnwindSafe(|| drop(evidence)))
            .map_err(|_| Error("witness disposal panic"));
        drop(permit);
        disposed
    }

    async fn commit_submissions(&mut self) -> Result<()> {
        if self.staged_submissions.is_empty() {
            return Ok(());
        }
        let mut witness = self
            .witness
            .take()
            .ok_or(Error("witness policy not configured"))?;
        let entries: Vec<_> = self
            .staged_submissions
            .values()
            .filter(|(submission, _)| {
                !witness
                    .ledger
                    .index
                    .submissions
                    .contains_key(&submission.id)
            })
            .map(|(submission, _)| LedgerEntry::Submission(submission.clone()))
            .collect();
        if !entries.is_empty() {
            let before = witness.ledger.records().len();
            self.evidence = Some(EvidenceIo {
                io_id: 0,
                before,
                future: Box::pin(async move {
                    let result = witness.ledger.append(entries, None);
                    (witness, result)
                }),
            });
            self.complete_evidence(PermitMode::IoRelease).await?;
        } else {
            self.witness = Some(witness);
        }
        Ok(())
    }

    fn release_io(&mut self, mode: PermitMode) {
        if mode != PermitMode::Cleanup && self.shared.failed() {
            return;
        }
        let Some(completed) = self.disk.completed.take() else {
            return;
        };
        let CompletedIo {
            identity: id,
            release,
            witness,
            lost_sync_response,
            ..
        } = completed;
        let permit = self.enter_permit(id.task, id.io_id, Some(id.cid), mode);
        // Sending only marks readiness. Record the release before polling
        // any consumer. Dispose an abandoned result under this same permit.
        let result = catch_unwind(AssertUnwindSafe(|| {
            drop(witness);
            let disposition = match (mode, lost_sync_response) {
                (PermitMode::IoRelease, true) => IoDisposition::LostSyncResponse,
                (PermitMode::IoRelease, false) => IoDisposition::Deliver,
                _ => IoDisposition::Discard,
            };
            release(disposition)
        }));
        match result {
            Ok(delivered) if mode == PermitMode::IoRelease => {
                self.shared.event(Event::IoReleased {
                    io_id: id.io_id,
                    delivered,
                })
            }
            Ok(_) => self.shared.event(Event::IoDiscarded { io_id: id.io_id }),
            Err(_) => self
                .shared
                .record_failure("physical I/O result disposal panic"),
        }
        drop(permit);
    }

    pub fn spawn_session<F, Fut>(&mut self, cid: Id, stop_local_us: i64, f: F) -> Result<()>
    where
        F: FnOnce(SimContext) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.shared.try_ordinary(None)?;
        let stopper = {
            let mut inner = self.shared.inner.lock().unwrap();
            let now = inner.clock.local().ok_or(Error("ClockUnsafe"))?;
            if now >= stop_local_us || inner.sessions.contains_key(&cid) {
                return Err(Error("session stopped or already exists"));
            }
            let stopper = Arc::new(Mutex::new(Stopper::new()));
            inner.sessions.insert(
                cid,
                Session {
                    stop_local_us,
                    retired: false,
                },
            );
            stopper
        };
        use commonware_runtime::Spawner as _;
        let context = SimContext::new(
            self.native.child("session"),
            Arc::downgrade(&self.shared),
            cid,
            RoleRng::root_role(),
            stopper,
            0,
            StorageAccess::new(self.native.clone(), Arc::downgrade(&self.shared), cid),
        );
        self.sessions.insert(cid, context.spawn(f));
        Ok(())
    }

    pub fn rng_checkpoint(&self, cid: Id, role: Id) -> Result<RngCheckpoint> {
        self.shared.try_ordinary(None)?;
        let mut inner = self.shared.inner.lock().unwrap();
        Ok(inner.role_rng(cid, role).checkpoint())
    }

    pub fn restore_rng(&self, cid: Id, role: Id, checkpoint: RngCheckpoint) -> Result<()> {
        self.shared.try_ordinary(None)?;
        let mut inner = self.shared.inner.lock().unwrap();
        if inner.rngs.contains_key(&(cid, role)) {
            return Err(Error("RNG role initialized"));
        }
        let profile = &inner.clock.profile;
        if checkpoint.seed
            != RoleRng::derive(profile.run_seed, profile.node, cid, role)
                .checkpoint()
                .seed
        {
            return Err(Error("RNG checkpoint role mismatch"));
        }
        inner
            .rngs
            .insert((cid, role), RoleRng::restore(checkpoint)?);
        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<()> {
        if self.has_active_permit() {
            self.fail("shutdown inside controlled permit");
            return Err(Error("shutdown inside controlled permit"));
        }
        if self.has_io() {
            self.fail("synchronous shutdown with physical I/O");
            return Err(Error("synchronous shutdown with physical I/O"));
        }
        self.shutdown_jobs()
    }

    /// Drain accepted physical I/O and dispose results and jobs before returning.
    pub async fn shutdown_async(&mut self) -> Result<()> {
        if self.has_active_permit() {
            self.fail("shutdown inside controlled permit");
            return Err(Error("shutdown inside controlled permit"));
        }
        if self.has_io() {
            self.shared
                .record_failure("cleanup with outstanding physical I/O");
            self.stopped = true;
            while self.has_io() {
                if self.evidence.is_some() {
                    if let Err(error) = self.complete_evidence(PermitMode::Cleanup).await {
                        self.shared.record_failure(error.0);
                    }
                } else if self.disk.completed.is_none() {
                    if let Err(error) = self.await_io(PermitMode::Cleanup).await {
                        self.shared.record_failure(error.0);
                    }
                }
                // An already-retained evidence operation can complete during
                // cleanup, while an unstarted evidence hook grants no output.
                self.release_io(PermitMode::Cleanup);
            }
            self.drain(None);
        }
        self.shutdown_jobs()
    }

    fn shutdown_jobs(&mut self) -> Result<()> {
        if self.stopped {
            return if self.shared.failed() {
                Err(Error("driver failed"))
            } else {
                Ok(())
            };
        }
        self.stopped = true;
        if self.drain(None) && !self.shared.failed() {
            Ok(())
        } else {
            self.shared.record_failure("shutdown cleanup failed");
            Err(Error("shutdown cleanup failed"))
        }
    }

    fn fail(&mut self, reason: &'static str) {
        if self.stopped {
            return;
        }
        self.shared.record_failure(reason);
        self.staged_submissions.clear();
        self.shared.inner.lock().unwrap().actor_routes.clear();
        // A synchronous owner must release its permit. Root can await disk
        // cleanup after a failed or abandoned async grant.
        if !self.has_active_permit() {
            self.shared.inner.lock().unwrap().current = None;
            if !self.has_io() {
                self.stopped = true;
                self.drain(None);
            }
        }
    }

    fn has_active_permit(&self) -> bool {
        self.shared.inner.lock().unwrap().permit.is_some()
    }

    fn import_jobs(&mut self) {
        self.jobs
            .append(&mut self.shared.inner.lock().unwrap().enrolled);
    }

    fn poll_job(&mut self, id: u64) -> Result<()> {
        let mut job = self.jobs.remove(&id).ok_or(Error("ready task missing"))?;
        let sequence = match job.sequence.checked_add(1) {
            Some(sequence) => sequence,
            None => {
                self.jobs.insert(id, job);
                return Err(Error("poll sequence overflow"));
            }
        };
        job.sequence = sequence;
        let scope = self.enter_permit(id, sequence, Some(job.cid), PermitMode::Ordinary);
        self.shared.event(Event::Poll {
            task: id,
            grant: scope.permit.grant,
            sequence,
        });
        let waker = Waker::from(Arc::new(Ready {
            shared: Arc::downgrade(&self.shared),
            task: id,
        }));
        let result = catch_unwind(AssertUnwindSafe(|| {
            let mut future = pin!(tokio::task::unconstrained(job.native.as_mut()));
            future.as_mut().poll(&mut TaskContext::from_waker(&waker))
        }));
        match result {
            Ok(Poll::Pending) => {
                self.jobs.insert(id, job);
            }
            Ok(Poll::Ready(())) => self.dispose(id, job, &scope),
            Err(_) => {
                self.shared.record_failure("native poll unwind");
                self.jobs.insert(id, job);
            }
        }
        Ok(())
    }

    fn enter_permit(
        &self,
        task: u64,
        sequence: u64,
        cid: Option<Id>,
        mode: PermitMode,
    ) -> PermitScope {
        let mut inner = self.shared.inner.lock().unwrap();
        if inner.permit.is_some() {
            drop(inner);
            panic!("a controlled permit is already active");
        }
        let permit = Permit {
            grant: inner.current.map_or(self.last_grant, |grant| grant.id),
            task,
            sequence,
            cid,
            mode,
            thread: thread::current().id(),
        };
        inner.permit = Some(permit);
        PermitScope {
            shared: self.shared.clone(),
            permit,
        }
    }

    fn dispose(&self, id: u64, job: Job, scope: &PermitScope) {
        {
            let mut inner = self.shared.inner.lock().unwrap();
            inner.live.remove(&id);
            inner.ready.remove(&id);
            if (scope.permit.mode == PermitMode::Cleanup)
                || *job.phase.lock().unwrap() != TaskPhase::Completed
            {
                inner.events.push(Event::Cancel { task: id });
            }
            inner.events.push(Event::Dispose {
                task: id,
                grant: scope.permit.grant,
                sequence: scope.permit.sequence,
                cleanup: (scope.permit.mode == PermitMode::Cleanup),
            });
        }
        if catch_unwind(AssertUnwindSafe(|| drop(job))).is_err() {
            self.shared.record_failure("native disposal unwind");
        }
    }

    fn dispose_retired(&mut self) {
        let retired: Vec<Id> = self
            .shared
            .inner
            .lock()
            .unwrap()
            .sessions
            .iter()
            .filter_map(|(cid, session)| session.retired.then_some(*cid))
            .collect();
        for cid in retired {
            if self.sessions.contains_key(&cid) && !self.drain(Some(cid)) {
                self.shared.record_failure("session cleanup failed");
            }
        }
    }

    fn drain(&mut self, cid: Option<Id>) -> bool {
        self.import_jobs();
        let mut clean = true;
        let mut count = 0;
        loop {
            let next = self
                .jobs
                .iter()
                .find(|(_, job)| cid.is_none_or(|cid| job.cid == cid))
                .map(|(id, _)| *id);
            let Some(id) = next else { break };
            count += 1;
            if count > self.cleanup_limit && clean {
                clean = false;
                self.shared.event(Event::CleanupBoundExceeded);
                self.shared
                    .record_failure("cleanup disposal limit exceeded");
            }
            let job = self.jobs.remove(&id).expect("owned job");
            let sequence = job.sequence.checked_add(1).unwrap_or_else(|| {
                clean = false;
                u64::MAX
            });
            let scope = self.enter_permit(id, sequence, Some(job.cid), PermitMode::Cleanup);
            self.dispose(id, job, &scope);
            drop(scope);
            self.import_jobs();
        }
        let session_ids: Vec<_> = self
            .sessions
            .keys()
            .filter(|id| cid.is_none_or(|cid| cid == **id))
            .copied()
            .collect();
        for id in session_ids {
            let scope = self.enter_permit(0, 0, Some(id), PermitMode::Cleanup);
            if let Some(handle) = self.sessions.remove(&id) {
                if catch_unwind(AssertUnwindSafe(|| drop(handle))).is_err() {
                    self.shared.record_failure("session handle disposal unwind");
                    clean = false;
                }
            }
            drop(scope);
        }
        // Failure cleanup still has a real drained endpoint. An exceeded bound
        // must not advertise successful cleanup even if disposal later empties it.
        if clean {
            self.shared.event(Event::CleanupComplete);
        }
        clean
    }
}

struct WitnessHold {
    entered: tokio::sync::oneshot::Sender<()>,
    wait: tokio::sync::oneshot::Receiver<()>,
}

struct EvidenceIo {
    io_id: u64,
    before: usize,
    future: std::pin::Pin<Box<dyn Future<Output = (WitnessRuntime, Result<()>)> + Send>>,
}

struct GrantCustody {
    shared: Arc<Shared>,
    grant: Grant,
    armed: bool,
}
impl Drop for GrantCustody {
    fn drop(&mut self) {
        if self.armed {
            let mut inner = self.shared.inner.lock().unwrap();
            if inner.current == Some(self.grant) {
                inner.current = None;
                self.shared.failure.store(true, Ordering::SeqCst);
                inner.events.push(Event::Failure {
                    reason: "async grant abandoned",
                });
            }
        }
    }
}

struct PermitScope {
    shared: Arc<Shared>,
    permit: Permit,
}
impl Drop for PermitScope {
    fn drop(&mut self) {
        self.shared.inner.lock().unwrap().permit = None;
    }
}

impl Shared {
    pub fn witness_partition(&self, partition: &str) -> bool {
        self.inner.lock().unwrap().witness_partition.as_deref() == Some(partition)
    }

    pub fn issue_io(
        &self,
        scope: Option<Id>,
        mut target: IoTarget,
        operation: IoOperation,
    ) -> IoIdentity {
        let issuer = self.try_ordinary(scope).and_then(|permit| {
            permit
                .cid
                .map(|cid| (permit, cid))
                .ok_or(Error("storage requires a session permit"))
        });
        let (permit, cid) = issuer.unwrap_or_else(|error| {
            self.record_failure(error.0);
            panic!("{error}");
        });
        let mut inner = self.inner.lock().unwrap();
        let Some(io_id) = inner.next_io.checked_add(1) else {
            drop(inner);
            self.record_failure("I/O counter exhausted");
            panic!("I/O counter exhausted");
        };
        inner.next_io = io_id;
        if matches!(operation, IoOperation::Open { .. }) {
            target.open_id = Some(io_id);
        }
        let profile = &inner.clock.profile;
        let identity = IoIdentity {
            io_id,
            grant: permit.grant,
            tick_us: inner.clock.tick_us,
            run_id: profile.run_id,
            node: profile.node,
            incarnation: profile.incarnation,
            cid,
            task: permit.task,
        };
        inner.events.push(Event::IoIssued {
            identity: identity.clone(),
            target,
            operation,
        });
        identity
    }

    pub fn enqueue_io(&self, job: IoJob) {
        self.inner
            .lock()
            .unwrap()
            .queued_io
            .insert(job.identity.io_id, job);
    }

    pub fn failed(&self) -> bool {
        self.failure.load(Ordering::SeqCst)
    }

    pub fn event(&self, event: Event) {
        self.inner.lock().unwrap().events.push(event);
    }

    fn record_failure(&self, reason: &'static str) {
        self.failure.store(true, Ordering::SeqCst);
        self.event(Event::Failure { reason });
    }

    fn snapshot_clock(&self, cid: Id) -> ClockAudit {
        self.inner.lock().unwrap().clock_snapshot(cid)
    }

    fn audit_clock(&self, cid: Id, caller: &str) -> ClockAudit {
        let mut inner = self.inner.lock().unwrap();
        let snapshot = inner.clock_snapshot(cid);
        if inner.native_clock_audit {
            inner.events.push(Event::ClockRead {
                caller: caller.to_owned(),
                snapshot: snapshot.clone(),
            });
        }
        snapshot
    }

    fn try_ordinary(&self, cid: Option<Id>) -> Result<Permit> {
        let inner = self.inner.lock().unwrap();
        if self.failed() {
            return Err(Error("permanent infrastructure failure"));
        }
        let permit = inner.permit.ok_or(Error("no controlled permit"))?;
        if permit.mode != PermitMode::Ordinary || permit.thread != thread::current().id() {
            return Err(Error("ordinary operation outside controlled poll"));
        }
        if let Some(cid) = cid {
            let session = inner.sessions.get(&cid).ok_or(Error("unknown session"))?;
            if permit.cid.is_some_and(|owner| owner != cid) {
                return Err(Error("session permit identity mismatch"));
            }
            if session.retired
                || inner
                    .clock
                    .local()
                    .is_none_or(|local| local >= session.stop_local_us)
            {
                return Err(Error("session clock unsafe or terminated"));
            }
        }
        Ok(permit)
    }

    pub fn ordinary(&self, cid: Id) {
        if let Err(error) = self.try_ordinary(Some(cid)) {
            self.failure.store(true, Ordering::SeqCst);
            panic!("{error}");
        }
    }

    pub fn can_publish(&self, cid: Id) -> bool {
        self.try_ordinary(Some(cid)).is_ok()
    }

    pub fn reserve_task(&self, cid: Id) -> u64 {
        self.ordinary(cid);
        let id = {
            let mut inner = self.inner.lock().unwrap();
            inner
                .next_task
                .checked_add(1)
                .inspect(|next| inner.next_task = *next)
        };
        id.expect("task id overflow")
    }

    pub fn enroll(
        &self,
        id: u64,
        parent: u64,
        cid: Id,
        role: Id,
        placement: Placement,
        phase: Arc<Mutex<TaskPhase>>,
        native: ControlledJob,
    ) {
        // Called synchronously by native admission. The IDs have already been
        // reserved, and no user code runs or disposes a job while this lock is held.
        let mut inner = self.inner.lock().unwrap();
        inner.enrolled.insert(
            id,
            Job {
                native,
                cid,
                phase,
                sequence: 0,
            },
        );
        inner.live.insert(id);
        inner.ready.insert(id);
        inner.events.push(Event::TaskRegistered {
            task: id,
            parent,
            cid,
            role,
            placement,
        });
    }

    pub fn draw<T>(
        &self,
        cid: Id,
        role: Id,
        words: u64,
        draw: impl FnOnce(&mut RoleRng) -> Result<T>,
    ) -> T {
        let permit = self.try_ordinary(Some(cid)).unwrap_or_else(|error| {
            self.failure.store(true, Ordering::SeqCst);
            panic!("{error}");
        });
        let result = {
            let mut inner = self.inner.lock().unwrap();
            let result = draw(inner.role_rng(cid, role));
            if result.is_ok() {
                inner.events.push(Event::RngDraw {
                    task: permit.task,
                    role,
                    words,
                });
            }
            result
        };
        result.unwrap_or_else(|error| {
            self.record_failure(error.0);
            panic!("{error}");
        })
    }

    pub fn arm(&self, cid: Id, deadline_us: i64, waker: &Waker) -> u64 {
        let permit = self
            .try_ordinary(Some(cid))
            .expect("timer requires poll permit");
        let waker = Arc::new(waker.clone());
        let mut inner = self.inner.lock().unwrap();
        let next = inner.next_timer.checked_add(1);
        let Some(id) = next else {
            drop(inner);
            panic!("timer id overflow");
        };
        inner.next_timer = id;
        let timer = Timer {
            id,
            cid,
            deadline_us,
        };
        inner.timers.insert(
            id,
            TimerWait {
                timer: timer.clone(),
                owner: permit.task,
                waker,
                fired: false,
            },
        );
        inner.events.push(Event::TimerArmed { timer });
        id
    }

    pub fn timer_polled(&self, cid: Id, id: u64, waker: &Waker) -> bool {
        let permit = self
            .try_ordinary(Some(cid))
            .expect("timer poll requires permit");
        let new_waker = Arc::new(waker.clone());
        let mut inner = self.inner.lock().unwrap();
        let Some(wait) = inner.timers.get_mut(&id) else {
            drop(inner);
            panic!("registered timer missing");
        };
        let transfer = if wait.owner != permit.task {
            let from = std::mem::replace(&mut wait.owner, permit.task);
            Some(Event::TimerTransferred {
                timer: wait.timer.clone(),
                from,
                to: permit.task,
            })
        } else {
            None
        };
        if wait.fired {
            let detached = inner.timers.remove(&id);
            inner.events.extend(transfer);
            drop(inner);
            drop(detached);
            true
        } else {
            let old_waker = std::mem::replace(&mut wait.waker, new_waker);
            inner.events.extend(transfer);
            drop(inner);
            drop(old_waker);
            false
        }
    }

    pub fn cancel_timer(&self, id: u64) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(wait) = inner.timers.remove(&id) {
            let valid = inner.permit.is_some_and(|p| {
                p.thread == thread::current().id() && p.cid == Some(wait.timer.cid)
            });
            if !valid {
                self.failure.store(true, Ordering::SeqCst);
                inner.events.push(Event::Failure {
                    reason: "timer disposed outside permit",
                });
            }
            inner
                .events
                .push(Event::TimerCancelled { timer: wait.timer });
            drop(inner);
            drop(wait.waker);
        }
    }
}

impl Inner {
    fn clock_snapshot(&self, cid: Id) -> ClockAudit {
        let (sample_sequence, lease_end_tick_us) = self.clock.calibration().unwrap_or_default();
        ClockAudit {
            cid,
            grant_id: self.current.map_or(0, |g| g.id),
            task_id: self.permit.map_or(0, |p| p.task),
            tick_us: self.clock.tick_us,
            local_time_us: self.clock.local(),
            sample_sequence,
            lease_end_tick_us,
            local_stop_us: self.sessions.get(&cid).map(|s| s.stop_local_us),
        }
    }
}

impl Inner {
    fn role_rng(&mut self, cid: Id, role: Id) -> &mut RoleRng {
        self.rngs.entry((cid, role)).or_insert_with(|| {
            RoleRng::derive(
                self.clock.profile.run_seed,
                self.clock.profile.node,
                cid,
                role,
            )
        })
    }

    fn retire(&mut self, cid: Id, reason: &'static str) {
        if let Some(session) = self
            .sessions
            .get_mut(&cid)
            .filter(|session| !session.retired)
        {
            session.retired = true;
            self.events.push(Event::Retired { cid, reason });
        }
    }

    fn check_sessions(&mut self) {
        let local = self.clock.local();
        let expired: Vec<_> = self
            .sessions
            .iter()
            .filter_map(|(cid, session)| {
                if session.retired {
                    return None;
                }
                match local {
                    None => Some((*cid, "ClockUnsafe")),
                    Some(local) if local >= session.stop_local_us => Some((*cid, "LocalStop")),
                    _ => None,
                }
            })
            .collect();
        for (cid, reason) in expired {
            self.retire(cid, reason);
        }
    }

    fn apply(&mut self, input: Input) -> Result<Option<Arc<Waker>>> {
        match input {
            Input::None => {}
            Input::RequestSample => {
                let request = self.clock.request()?;
                self.events.push(Event::SampleRequested { request });
            }
            Input::DeliverSample(sample) => {
                let request = sample.request_id;
                let sequence = sample.sequence;
                match self.clock.sample(sample) {
                    Ok(()) => self
                        .events
                        .push(Event::SampleAccepted { request, sequence }),
                    Err(error) => self.events.push(Event::SampleRejected {
                        request,
                        reason: error.0,
                    }),
                }
            }
            Input::Timer(timer) => {
                let due = self
                    .clock
                    .local()
                    .is_some_and(|local| local >= timer.deadline_us);
                let live = self
                    .sessions
                    .get(&timer.cid)
                    .is_some_and(|session| !session.retired);
                let wait = self
                    .timers
                    .get_mut(&timer.id)
                    .filter(|wait| wait.timer == timer && !wait.fired && due && live);
                if let Some(wait) = wait {
                    wait.fired = true;
                    self.events.push(Event::TimerFired { timer });
                    return Ok(Some(wait.waker.clone()));
                }
                self.events.push(Event::TimerSuppressed { timer });
            }
            Input::Close(cid) => self.retire(cid, "Close"),
        }
        Ok(None)
    }
}
