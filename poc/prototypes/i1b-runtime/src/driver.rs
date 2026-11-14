use crate::{
    ClockSample, Error, Id, Profile, Result, RngCheckpoint, RoleRng, SimContext, Timer,
    clock::NodeClock,
};
use commonware_runtime::{
    Handle, Supervisor as _,
    signal::Stopper,
    tokio::{Config, Context, ControlledJob, Runner},
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
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
    RngDraw {
        task: u64,
        role: Id,
        words: u64,
    },
    Failure {
        reason: &'static str,
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

#[derive(Clone, Copy)]
struct Permit {
    grant: u64,
    task: u64,
    sequence: u64,
    cid: Option<Id>,
    cleanup: bool,
    thread: ThreadId,
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
    native: Context,
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
        Runner::new(config.with_catch_panics(true)).start(move |native| async move {
            let mut driver = Self::new(native, profile)?;
            let result = AssertUnwindSafe(async { controller(&mut driver).await })
                .catch_unwind()
                .await;
            match result {
                Ok(Ok(value)) => {
                    driver.shutdown()?;
                    Ok(value)
                }
                Ok(Err(error)) => {
                    driver.fail(error.0);
                    Err(error)
                }
                Err(_) => {
                    driver.fail("controller panic");
                    Err(Error("controller panic"))
                }
            }
        })
    }

    fn new(native: Context, profile: Profile) -> Result<Self> {
        let shared = Arc::new(Shared {
            failure: Arc::new(AtomicBool::new(false)),
            inner: Mutex::new(Inner {
                clock: NodeClock::new(profile)?,
                current: None,
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
            native,
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

    pub fn events(&self) -> Vec<Event> {
        self.shared.inner.lock().unwrap().events.clone()
    }

    pub fn fail_transport(&mut self) -> Result<()> {
        self.fail("transport disconnected");
        Err(Error("transport disconnected"))
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
        let outcome = catch_unwind(AssertUnwindSafe(|| self.drive(grant, input, action)));
        match outcome {
            Ok(Ok(step)) => Ok(step),
            Ok(Err(error)) => {
                self.fail(error.0);
                Err(error)
            }
            Err(_) => {
                self.fail("driver panic");
                Err(Error("driver panic"))
            }
        }
    }

    fn drive(
        &mut self,
        grant: Grant,
        input: Input,
        action: impl FnOnce(&mut Self) -> Result<()>,
    ) -> Result<Step> {
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
        if let Some(waker) = wake {
            let permit = self.enter_permit(0, 0, None, false);
            waker.wake_by_ref();
            drop(permit);
        }
        self.dispose_retired();
        if self.shared.failed() {
            return Err(Error("native task failure"));
        }
        let permit = self.enter_permit(0, 0, None, false);
        let result = action(self);
        drop(permit);
        result?;
        let mut polls = 0;
        loop {
            if self.shared.failed() {
                return Err(Error("native task failure"));
            }
            self.import_jobs();
            let next = self.shared.inner.lock().unwrap().ready.pop_first();
            let Some(id) = next else { break };
            if polls >= self.poll_limit {
                return Err(Error("logical poll limit exceeded"));
            }
            polls += 1;
            self.poll_job(id)?;
            self.dispose_retired();
        }
        let mut inner = self.shared.inner.lock().unwrap();
        if self.shared.failed() {
            return Err(Error("native task failure"));
        }
        inner.current = None;
        inner.events.push(Event::Ack { grant: grant.id });
        Ok(Step {
            grant,
            events: inner.events[start..].to_vec(),
        })
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
        // The grant owner releases its permit before handling the latched
        // failure. Reentrant control must leave the job registry in its custody.
        if !self.has_active_permit() {
            self.stopped = true;
            self.drain(None);
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
        let scope = self.enter_permit(id, sequence, Some(job.cid), false);
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
        cleanup: bool,
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
            cleanup,
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
            if scope.permit.cleanup || *job.phase.lock().unwrap() != TaskPhase::Completed {
                inner.events.push(Event::Cancel { task: id });
            }
            inner.events.push(Event::Dispose {
                task: id,
                grant: scope.permit.grant,
                sequence: scope.permit.sequence,
                cleanup: scope.permit.cleanup,
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
            let scope = self.enter_permit(id, sequence, Some(job.cid), true);
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
            let scope = self.enter_permit(0, 0, Some(id), true);
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

    fn try_ordinary(&self, cid: Option<Id>) -> Result<Permit> {
        let inner = self.inner.lock().unwrap();
        if self.failed() {
            return Err(Error("permanent infrastructure failure"));
        }
        let permit = inner.permit.ok_or(Error("no controlled permit"))?;
        if permit.cleanup || permit.thread != thread::current().id() {
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

    pub fn local(&self, cid: Id) -> i64 {
        self.ordinary(cid);
        self.inner
            .lock()
            .unwrap()
            .clock
            .local()
            .expect("guarded NodeClock")
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
