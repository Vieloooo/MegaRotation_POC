use poc_publication_bridge::{BridgeResult, RecordDelivery};
use poc_session_manager::{
    ActionId, ActionIntent, ActionKind, ActionReceipt, CommitRef, Manager, ManagerBinding,
    ManagerInput, PlanOutcome, PreparedRef, ReceiptOutcome, VerifiedGenesis, VerifiedReplayInput,
};
use poc_types::domain::*;
use poc_viewer_gfr::{GlobalSource, InputRef, ReplayEffect, lifecycle_projection};
use prost::Message;
use serde_json::{Value, json};
use sim_espresso::{FinalizedBlock, TickUs};
use super::m5;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Write,
    path::PathBuf,
    process::Stdio,
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::{Child, ChildStdin, ChildStdout, Command},
};

use super::{
    M1Result, check, checked, governance::{self as gov, Scenario}, l1, problem,
    profile::*,
    public::{Public, Submitted, initial_descriptor},
    worker::{IDLE_RETIRE, Launch, durable_file},
};
use crate::{
    Id, Profile, hash_parts,
    native_config::Resolved,
    native_network,
    native_wire::{self as p, effect, input},
    native_wire_support as wire,
};

const PUBLIC_LINK_US: u64 = 5_000;
const FINALIZE_US: u64 = 1_000;
const WATCHDOG: Duration = Duration::from_secs(20);
const MAX_EVENTS: u64 = 100_000;
const ROUTE_BOUND: usize = 64;

#[derive(Default)]
struct Regional {
    proofs: BTreeMap<Id, p::ProofObserved>,
    bodies: BTreeMap<u64, PhysicalBlock>,
}

struct Worker {
    pid: Pid,
    os_pid: u32,
    child: Child,
    tx: Option<ChildStdin>,
    rx: ChildStdout,
    manager: Manager,
    config: Option<Arc<Resolved>>,
    pending_prepare: Option<(ActionIntent, Install)>,
    local_ready: bool,
    last_manager_time_us: u64,
    sample_seq: u64,
    request_seq: u64,
    steps: u64,
    terminal: bool,
    offline: bool,
    last_work: Option<p::Work>,
    regional: Regional,
    local_stop_ack: Option<p::Ack>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Key {
    tick: u64,
    wave: u64,
    tie: Id,
    id: u64,
}

type Route = (u32, u32, i32);
enum Event {
    Actions(u32),
    ApplyRecords(u32, Arc<[(u64, RecordDelivery)]>),
    Grant(u32, p::input::Kind),
    Clock(u32, u64),
    Timer(u32, p::Timer),
    Deliver(Route, p::Envelope),
    Publish(u32, ActionId, BridgeResult),
    Finalize(Submitted),
    CutoffRetire(u32),
    L1(l1::Event),
    CloseProbe(u32, p::Work),
    OfflineRegion,
    Heartbeat,
    FinalizeHeartbeat(u64),
    GovernancePoll,
    GovernanceClock(u64),
    GovernanceEvaluate,
    GovernanceL1(usize, Arc<Value>),
    GovernanceEspresso(usize, Arc<FinalizedBlock>),
    GovernanceSign(usize, gov::Request),
    GovernanceSubmit(gov::Request),
    GovernanceResolve,
    ContrastCutoff,
    M4Batch(u64),
    M4Resume,
    M5Stall,
    M5Recover,
    M5CoverageProbe,
    M5Release,

}

struct Hub {
    root: PathBuf,
    trace: File,
    frames: File,
    public: Public,
    workers: Vec<Worker>,
    source: Hash32,
    now: Key,
    next_id: u64,
    next_step: u64,
    trace_seq: u64,
    events: BTreeMap<Key, Event>,
    timers: BTreeMap<(u32, u64), p::Timer>,
    routes: BTreeMap<Route, (u64, usize)>,
    attempts: BTreeMap<Vec<u8>, u32>,
    scheduled_work: BTreeSet<(u32, Vec<u8>)>,
    history: Vec<AnchorStep>,
    publication_pending: bool,
    force: bool,
    wall_pause_ms: u64,
    l1: Option<l1::Port>,
    reader_b: l1::Reader,
    scenario: Option<Scenario>,
    governors: Vec<gov::Signer>,
    advertised_l1: Option<u64>,
    governance_intent: Option<gov::Request>,
    governance_signatures: BTreeMap<usize, Value>,
    pending_control: bool,
    submitted_control: Option<String>,
    governance_applied: bool,
    contrast_complete: bool,
    resume: bool,
    derived_head: Option<ExecutionAnchor>,
    conflict_seen: bool,
    included_cursor: u64,
    m5: Option<m5::Mode>,
    m5_held: Option<(u32, ActionId, BridgeResult)>,
    m5_eligibility: Option<(u64, gov::Request)>,
    m5_solution: Option<m5::Solution>,

}

pub(crate) async fn run(root: PathBuf, force: bool, wall_pause_ms: u64, scenario: Option<Scenario>, resume:bool, m5: Option<m5::Mode>) -> M1Result<()> {
    fs::create_dir(&root)?;
    let root = root.canonicalize()?;
    let public = Public::new_mode(&root, force, scenario.is_some(),resume,m5.is_some())?;
    let governors = if m5.is_some() {
        (0..3).map(|id|gov::Signer::resumable(id, &public.replay)).collect::<M1Result<_>>()?
    } else if scenario.is_some() {
        (0..3).map(|id| gov::Signer::new(id, &public.replay)).collect::<M1Result<_>>()?
    } else {
        Vec::new()
    };
    let source = Hash32::new(hash_parts(b"PoC/M1/GFR/source/v1", &[SOURCE, public.public_l1().source_id()]));
    let l1 = if force {
        Some(l1::Port::start(&root, Hash32::new(*public.inputs.config.reference()?.as_bytes()))?)
    } else {
        None
    };
    let mut hub = Hub {
        trace: File::options()
            .write(true)
            .create_new(true)
            .open(root.join("trace.jsonl"))?,
        frames: File::options()
            .write(true)
            .create_new(true)
            .open(root.join("frames.bin"))?,
        root,
        public,
        source,
        workers: Vec::new(),
        now: Key {
            tick: 0,
            wave: 0,
            tie: [0; 32],
            id: 0,
        },
        next_id: 0,
        next_step: 0,
        trace_seq: 0,
        events: BTreeMap::new(),
        timers: BTreeMap::new(),
        routes: BTreeMap::new(),
        attempts: BTreeMap::new(),
        scheduled_work: BTreeSet::new(),
        history: Vec::new(),
        publication_pending: false,
        force, wall_pause_ms, l1,
        reader_b: if resume {l1::Reader::resumable()} else if scenario.is_some() { l1::Reader::regional() } else { l1::Reader::default() },
        scenario,
        governors,
        advertised_l1: None,
        governance_intent: None,
        governance_signatures: BTreeMap::new(),
        pending_control: false,
        submitted_control: None,
        governance_applied: false,
        contrast_complete: false,
        resume, derived_head:None, conflict_seen:false, included_cursor:0,
        m5, m5_held:None, m5_eligibility:None, m5_solution:None,

    };
    let execution = hub.run_inner().await;
    let exits = hub.reap(execution.is_ok()).await;
    let l1 = hub.l1.take().map_or(Ok(()), l1::Port::finish);
    let result = execution.and(exits).and(l1).and_then(|()| hub.complete());
    if let Err(error) = &result {
        hub.log("RunFailed", None, json!({"reason": error.to_string()}))?;
    }
    hub.trace.sync_all()?;
    hub.frames.sync_all()?;
    result
}

impl Hub {
    fn log(&mut self, kind: &str, node: Option<u32>, detail: Value) -> M1Result<()> {
        self.trace_seq += 1;
        serde_json::to_writer(
            &mut self.trace,
            &json!({
                "seq": self.trace_seq, "event_id": self.now.id, "tick_us": self.now.tick,
                "kind": kind, "pid": node, "detail": detail,
            }),
        )?;
        self.trace.write_all(b"\n")?;
        Ok(())
    }

    fn schedule(&mut self, tick: u64, event: Event) -> M1Result<()> {
        check(
            tick >= self.now.tick && self.next_id < MAX_EVENTS,
            "M1 scheduler bound or time regression",
        )?;
        self.next_id += 1;
        let key = Key {
            tick,
            wave: if tick == self.now.tick {
                self.now.wave + 1
            } else {
                0
            },
            tie: wire::tie_hash(SEED, self.next_id),
            id: self.next_id,
        };
        self.events.insert(key, event);
        Ok(())
    }

    async fn spawn(&mut self) -> M1Result<()> {
        let binary = std::env::current_exe()?;
        let descriptor = initial_descriptor(&self.public.replay)?.clone();
        let public_config = self.public.inputs.config.clone();
        let genesis_bytes = descriptor.encode_checked()?;
        durable_file(&self.root.join("genesis-descriptor.bin"), &genesis_bytes)?;
        let mut profile = json!({
                "schema": "M1-HEALTHY-SMOKE-1", "seed": SEED, "start_us": START_US,
                "cutoff_us": CUT_US, "announcement_us": parameters().announcement_us,
                "grace_us": GRACE_US, "expiry_us": EXPIRY_US, "clock_rpc_one_way_us": CLOCK_TRANSIT_US,
                "clock_refresh_us": REFRESH_US, "clock_lease_us": LEASE_US,
                "epsilon_us_by_member": [-1000,-500,0,500,1000],
                "link_delay_us": LINK_US, "work_us": WORK_US, "local_enter_spacing_us": STARTUP_SKEW_US,
                "publisher_link_us": PUBLIC_LINK_US, "espresso_finalize_delay_us": FINALIZE_US,
            });
        if self.force {
            profile["schema"] = json!("M2-HEALTHY-FORCE-1");
            profile["force"] = json!({
                "submit_us": l1::SUBMIT_US, "act_us": l1::ACT_US, "force_init_us": 3 * SECOND,
                "l1_slot_us": l1::L1_SLOT_US, "l1_finality_us": l1::FINALITY_US,
                "worker_l1_projection": "reg-a-height-timestamp-v1",
                "worker_l1_visibility_lag_us": l1::WORKER_VISIBILITY_LAG_US,
                "l1_source": std::str::from_utf8(l1::SOURCE)?,
                "target_beta": 0, "stop_height": self.stop_height(),
                "t_replay_us": 100_000, "t_state_us": 500_000, "t_produce_us": 500_000,
                "t_pack_us": 50_000, "t_pub_us": 200_000, "t_e_us": SECOND, "delta_ts_us": SECOND,
            });
        }
        if let Some(scenario) = self.scenario {
            profile["schema"] = json!("M3-AUTO-FORCE-1");
            profile["expiry_us"] = json!(30 * SECOND);
            profile["cutoff_us"] = json!(60 * SECOND);
            profile["auto_force"] = json!({
                "scenario":scenario.name(), "outage_us":gov::OUTAGE, "heartbeat_us":gov::HEARTBEAT,
                "poll_us":gov::POLL, "stall_us":2 * SECOND, "committee":3, "threshold":gov::THRESHOLD,
                "reader_delays_us":[0,10000,20000], "sign_us":gov::SIGN_US,
                "broadcast_us":gov::BROADCAST_US,
            });
            profile.as_object_mut().expect("profile object").remove("force");
        }
        if self.resume {
            profile["schema"]=json!("M4-D-RESUME-1");
            profile["expiry_us"]=json!(30*SECOND);
            profile["cutoff_us"]=json!(60*SECOND);
            profile["resume"]=json!({"sequence_window_blocks":4,"submit_us":17*SECOND,
                "target_height":20,"origin":9,"threshold":2,"heartbeat_us":gov::HEARTBEAT});
            profile.as_object_mut().expect("profile").remove("force");
        }
        if let Some(mode)=self.m5 {
            profile["schema"]=json!("M5-AUTO-RESUME-1");
            profile["expiry_us"]=json!(m5::EXPIRY_US);
            profile["clock_rpc_one_way_us"]=json!(mode.transit());
            profile["resume"]=json!({"sequence_window_blocks":4,"heartbeat_us":gov::HEARTBEAT});
            profile["auto_resume"]=json!({"scenario":mode.name(),"stall_us":mode.stall(),
                "recover_us":mode.recovery(),"clock_rpc_one_way_us":mode.transit(),
                "poll_us":gov::POLL,"sign_us":gov::SIGN_US,"broadcast_us":gov::BROADCAST_US,
                "governance_readers":m5::SIGNER_READERS,"reader_delays_us":[0,10_000,20_000],
                "threshold":2,"t_prep_us":m5::PREP_US,"t_ready_us":m5::READY_US,
                "epsilon_us":m5::EPSILON_US});
        }
        durable_file(&self.root.join("profile.json"), &serde_json::to_vec_pretty(&profile)?)?;
        for number in 0..15u8 {
            let pid = Pid::new(number)?;
            let node = u32::from(number);
            let dir = self.root.join(format!("worker-{node}"));
            fs::create_dir(&dir)?;
            fs::create_dir(dir.join("manager"))?;
            let manager = Manager::new(
                ManagerBinding {
                    manager_id: Hash32::new(hash_parts(
                        b"PoC/M1/manager/v1",
                        &[public_config.reference()?.as_bytes(), &[number]],
                    )),
                    worker: pid,
                    public_config: public_config.reference()?,
                    gfr_source: self.source,
                    preparation_version: 1,
                },
                public_config.clone(),
                VerifiedGenesis {
                    descriptor: descriptor.clone(),
                    source_ref: Hash32::new(hash_parts(
                        b"PoC/M1/manager-genesis/v1",
                        &[&genesis_bytes],
                    )),
                },
            )?;
            let mut child = Command::new(&binary);
            child
                .arg(if self.scenario.is_some() || self.m5.is_some() { "--auto-force-worker" } else { "--worker" })
                .arg(node.to_string())
                .arg(&dir)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(File::create(dir.join("stderr.log"))?)
                .kill_on_drop(true);
            let mut child = child.spawn()?;
            let os_pid = child.id().ok_or_else(|| problem("missing spawned PID"))?;
            let tx = child
                .stdin
                .take()
                .ok_or_else(|| problem("missing worker input pipe"))?;
            let rx = child
                .stdout
                .take()
                .ok_or_else(|| problem("missing worker output pipe"))?;
            let proc_stat = fs::read_to_string(format!("/proc/{os_pid}/stat"))?;
            self.workers.push(Worker {
                pid,
                os_pid,
                child,
                tx: Some(tx),
                rx,
                manager,
                config: None,
                pending_prepare: None,
                local_ready: false,
                last_manager_time_us: 0,
                sample_seq: 0,
                request_seq: 0,
                steps: 0,
                terminal: false,
                offline: false,
                last_work: None,
                regional: Regional::default(),
                local_stop_ack:None,
            });
            self.log(
                "WorkerSpawn",
                Some(node),
                json!({
                    "os_pid": os_pid, "region": pid.region(), "member": pid.member(),
                    "identity_key": self.public.finality.0.registry().entry(pid).identity_key,
                    "signer_index": self.public.finality.0.registry().signer_index(pid),
                    "proc_stat": proc_stat.trim(), "wall_time_ns": wall_time_ns(),
                }),
            )?;
            if node < 5 {
                self.schedule(
                    START_US + u64::from(node) * STARTUP_SKEW_US,
                    Event::Actions(node),
                )?;
            }
        }
        Ok(())
    }

    async fn reap(&mut self, healthy: bool) -> M1Result<()> {
        let mut successful = true;
        for node in 0..self.workers.len() {
            let early_status = self.workers[node].child.try_wait()?;
            if healthy && (self.scenario.is_some() || self.m5.is_some()) && early_status.is_none()
                && !self.workers[node].offline && self.workers[node].config.is_none()
            {
                self.send(node as u32, IDLE_RETIRE).await?;
                self.log("IdleRetire", Some(node as u32), json!({"assigned":false}))?;
            }
            let worker = &mut self.workers[node];
            worker.tx.take();
            if !healthy && early_status.is_none() {
                let _ = worker.child.start_kill();
            }
            let status = tokio::time::timeout(WATCHDOG, worker.child.wait()).await??;
            let expected = healthy && if worker.offline { !status.success() } else { status.success() && early_status.is_none() };
            successful &= expected;
            let os_pid = worker.os_pid;
            self.log("WorkerExit", Some(node as u32), json!({
                "os_pid": os_pid, "exit": status.code(), "expected": expected,
                "wall_time_ns": wall_time_ns(),
            }))?;
        }
        check(successful || !healthy, "unplanned M1 worker exit")
    }
}

fn wall_time_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("valid diagnostic wall time")
        .as_nanos() as u64
}

fn action_id(id: ActionId) -> Value {
    json!({"manager_id": id.manager_id, "worker": id.worker,
        "revision": id.origin_revision, "ordinal": id.ordinal})
}

impl Hub {
    fn commit_manager(&mut self, node: u32, input: ManagerInput) -> M1Result<()> {
        let worker = &mut self.workers[node as usize];
        let now = worker.last_manager_time_us;
        let outcome = checked(worker.manager.plan(input, now))?;
        let PlanOutcome::Planned(delta) = outcome else {
            return Err(problem(format!(
                "M1 Manager unchanged or waited: {outcome:?}"
            )));
        };
        let cause = &delta.cause().input;
        let (kind, detail) = match cause {
            ManagerInput::ConsumeRecord(input) | ManagerInput::ConsumeResume {input,..} => (
                "Replay",
                json!({
                    "source_seq": input.seq, "source": input.source, "record_id": input.validation_ref,
                    "position": input.record.position,
                }),
            ),
            ManagerInput::PreparationCompleted(prepared) => (
                "Prepared",
                json!({
                    "action": action_id(prepared.prepare_action), "cid": prepared.cid,
                    "anchor": prepared.anchor, "execution_ref": prepared.execution_ref,
                    "instance_ref": prepared.instance_ref, "provenance_ref": prepared.provenance_ref,
                }),
            ),
            ManagerInput::Receipt(receipt) => (
                "Receipt",
                json!({
                    "action": action_id(receipt.id), "receipt_ref": receipt.receipt_ref,
                    "outcome": format!("{:?}", receipt.outcome),
                }),
            ),
            ManagerInput::ClockReady {
                generation,
                basis_ref,
            } => (
                "ClockReady",
                json!({"generation": generation, "basis_ref": basis_ref}),
            ),
            ManagerInput::ObserveTime => ("ObserveTime", json!({"local_time_us":now})),
            ManagerInput::Schedule(session) => ("Schedule", json!({"session": session})),
            ManagerInput::RetainResult {
                key,
                verified_result_ref,
            } => (
                "RetainResult",
                json!({"cid":key.cid, "physical_id":key.block_hash, "result_ref":verified_result_ref}),
            ),
            _ => return Err(problem("unexpected Manager input for healthy smoke")),
        };
        let intents = delta
            .intents()
            .iter()
            .map(|intent| {
                json!({
                    "id": action_id(intent.id), "kind": format!("{:?}", intent.kind),
                })
            })
            .collect::<Vec<_>>();
        let revision = delta.target_revision();
        let bytes = serde_json::to_vec(&json!({
            "revision": revision, "local_time_us": now, "decision": kind, "input": detail,
            "intents": intents, "event_id": self.now.id,
        }))?;
        let durable_ref = Hash32::new(hash_parts(b"PoC/M1/manager-commit/v1", &[&bytes]));
        let path = self
            .root
            .join(format!("worker-{node}/manager/{revision:04}.json"));
        durable_file(&path, &bytes)?;
        let receipt = CommitRef::for_delta(&delta, durable_ref);
        checked(worker.manager.accept_commit(delta, receipt))?;
        self.log(
            "ManagerCommitted",
            Some(node),
            json!({
                "revision": revision, "local_time_us": now, "decision": kind,
                "input": detail, "intents": intents, "commit_ref": durable_ref,
            }),
        )
    }

    fn manager_receipt(
        &mut self,
        node: u32,
        id: ActionId,
        hash: Hash32,
        outcome: ReceiptOutcome,
    ) -> M1Result<()> {
        self.commit_manager(
            node,
            ManagerInput::Receipt(ActionReceipt {
                id,
                receipt_ref: hash,
                outcome,
            }),
        )
    }

    async fn manager_actions(&mut self, node: u32) -> M1Result<()> {
        let worker = &self.workers[node as usize];
        if worker.offline || ((self.scenario.is_some() || self.resume) && node < 5 && worker.config.is_none()
            && self.now.tick < START_US + u64::from(node) * STARTUP_SKEW_US)
        {
            return Ok(());
        }
        loop {
            let worker = &self.workers[node as usize];
            let intent = worker
                .manager
                .dispatchable(worker.last_manager_time_us)
                .into_iter()
                .find(|intent| match intent.kind {
                    ActionKind::Prepare { .. } => worker.pending_prepare.is_none(),
                    ActionKind::Publish { .. } => false,
                    _ => true,
                })
                .cloned();
            let Some(intent) = intent else {
                return Ok(());
            };
            let id = intent.id;
            match intent.kind.clone() {
                ActionKind::Create {
                    descriptor,
                    provenance_ref,
                } => {
                    check(
                        self.workers[node as usize].config.is_none(),
                        "duplicate M1 instance installation",
                    )?;
                    let bytes = descriptor.encode_checked()?;
                    let instance = Hash32::new(hash_parts(
                        b"PoC/M1/instance/v1",
                        &[&node.to_be_bytes(), &bytes, provenance_ref.as_bytes()],
                    ));
                    let dir = self.root.join(format!("worker-{node}"));
                    durable_file(&dir.join("descriptor.bin"), &bytes)?;
                    let cfg = Arc::new(Resolved::normal(&self.public.finality.0, &descriptor)?);
                    self.workers[node as usize].config = Some(cfg);
                    self.log("Create", Some(node), json!({"action": action_id(id), "cid": descriptor.cid,
                        "descriptor": descriptor, "instance_ref": instance, "public_record_ref": provenance_ref}))?;
                    self.manager_receipt(
                        node,
                        id,
                        instance,
                        ReceiptOutcome::Created {
                            instance_ref: instance,
                        },
                    )?;
                }
                ActionKind::Prepare { cid, .. } => {
                    let descriptor = self.workers[node as usize]
                        .manager
                        .view()
                        .program(cid)
                        .ok_or_else(|| problem("unknown Prepare cid"))?
                        .descriptor
                        .clone();
                    let install =
                        self.public
                            .install(descriptor, self.history.clone(), self.now.tick)?;
                    let frame = serde_json::to_vec(&Launch {
                        install: install.clone(),
                        inputs: self.public.inputs.clone(),
                    })?;
                    // Fsync the exact controlled install input before delivering it.
                    durable_file(
                        &self.root.join(format!("worker-{node}/install-grant.json")),
                        &frame,
                    )?;
                    self.log(
                        "Prepare",
                        Some(node),
                        json!({
                            "action": action_id(id), "cid": cid, "anchor": install.execution.anchor,
                            "state_root": install.execution.state.root()?,
                        }),
                    )?;
                    self.workers[node as usize].pending_prepare = Some((intent, install));
                    self.send(node, &frame).await?;
                    self.schedule(
                        self.now.tick,
                        Event::Grant(node, input::Kind::RequestSample(p::Empty {})),
                    )?;
                }
                ActionKind::Enter {
                    cid, instance_ref, ..
                } => {
                    let start = self.cfg(node)?.start();
                    let ack = self.grant(node, input::Kind::Start(start)).await?;
                    require_lifecycle(&ack, cid, "started")?;
                    let entry_time_us = self.workers[node as usize].last_manager_time_us;
                    self.log(
                        "Enter",
                        Some(node),
                        json!({
                            "cid": cid, "action": action_id(id), "local_time_us": entry_time_us,
                            "ack": ack.durable_step_hash, "logical_step_id": ack.logical_step_id,
                        }),
                    )?;
                    self.manager_receipt(
                        node,
                        id,
                        root_hash(&ack)?,
                        ReceiptOutcome::Entered {
                            instance_ref,
                            entry_time_us,
                        },
                    )?;
                }
                ActionKind::Close {
                    cid,
                    instance_ref,
                    cause,
                } => {
                    let native_stop=self.workers[node as usize].local_stop_ack.clone();
                    let ack=if let Some(ack)=native_stop {ack} else {
                        self.grant(node,input::Kind::Close(p::Cid {cid:cid.as_bytes().to_vec()})).await?
                    };
                    require_lifecycle(&ack, cid, "terminated")?;
                    self.workers[node as usize].terminal = true;
                    self.log(
                        "Close",
                        Some(node),
                        json!({
                            "cid": cid, "action": action_id(id), "cause": format!("{cause:?}"),
                            "ack": ack.durable_step_hash, "logical_step_id": ack.logical_step_id,
                            "retained_bodies": self.workers[node as usize].regional.bodies.len(),
                            "retained_proofs": self.workers[node as usize].regional.proofs.len(),
                        }),
                    )?;
                    self.manager_receipt(
                        node,
                        id,
                        root_hash(&ack)?,
                        ReceiptOutcome::Closed { instance_ref },
                    )?;
                    if self.force && cause == poc_session_manager::TerminalCause::Public(CloseReason::Force) {
                        let work = self.workers[node as usize].last_work.clone()
                            .ok_or_else(|| problem("old Force worker lacks prior native work"))?;
                        self.schedule(self.now.tick, Event::CloseProbe(node, work))?;
                    }
                }
                _ => return Err(problem("unexpected non-healthy Manager action")),
            }
        }
    }

    fn cfg(&self, node: u32) -> M1Result<Arc<Resolved>> {
        self.workers[node as usize]
            .config
            .clone()
            .ok_or_else(|| problem("worker not installed"))
    }

    async fn send(&mut self, node: u32, bytes: &[u8]) -> M1Result<()> {
        check(
            !bytes.is_empty() && bytes.len() <= wire::MAX_FRAME,
            "M1 outgoing frame bound",
        )?;
        let tx = self.workers[node as usize]
            .tx
            .as_mut()
            .ok_or_else(|| problem("missing child control pipe"))?;
        tokio::time::timeout(WATCHDOG, async {
            tx.write_all(&(bytes.len() as u32).to_be_bytes()).await?;
            tx.write_all(bytes).await?;
            tx.flush().await
        })
        .await??;
        Ok(())
    }

    fn frame(&mut self, direction: u8, node: u32, bytes: &[u8]) -> M1Result<()> {
        self.frames.write_all(&[direction])?;
        self.frames.write_all(&node.to_be_bytes())?;
        self.frames.write_all(&(bytes.len() as u32).to_be_bytes())?;
        self.frames.write_all(bytes)?;
        self.frames.sync_data()?;
        Ok(())
    }

    async fn grant(&mut self, node: u32, kind: input::Kind) -> M1Result<p::Ack> {
        let cfg = self.cfg(node)?;
        self.next_step += 1;
        let member = node % 5;
        let mut grant = p::Grant {
            binding: Some(cfg.binding(member)),
            logical_step_id: self.next_step,
            event_id: self.now.id,
            tick_us: self.now.tick,
            causal_wave: self.now.wave,
            tie_rank: self.now.tie.to_vec(),
            input: Some(p::Input {
                kind: Some(kind.clone()),
            }),
            grant_hash: vec![],
        };
        grant.grant_hash = wire::grant_hash(&grant).to_vec();
        let bytes = grant.encode_to_vec();
        self.frame(0, node, &bytes)?;
        let mut detail = json!({
            "logical_step_id":grant.logical_step_id, "grant_hash": grant.grant_hash, "input": kind,
        });
        if self.force {
            detail["worker_l1_through"] = json!(self.public.ensure_worker_l1_projection(self.now.tick)?);
        }
        self.log("Grant", Some(node), detail)?;
        self.send(node, &bytes).await?;
        let rx = &mut self.workers[node as usize].rx;
        let bytes = tokio::time::timeout(WATCHDOG, async {
            let len = rx.read_u32().await? as usize;
            if len == 0 || len > wire::MAX_FRAME {
                return Err(std::io::Error::other("invalid M1 Ack wire length"));
            }
            let mut bytes = vec![0; len];
            rx.read_exact(&mut bytes).await?;
            Ok::<_, std::io::Error>(bytes)
        })
        .await
        .map_err(|_| problem(format!("worker {node} Ack timeout")))?
        .map_err(|error| problem(format!("worker {node} failed to return Ack: {error}")))?;
        self.frame(1, node, &bytes)?;
        let ack = p::Ack::decode(bytes.as_slice())?;
        let worker = &mut self.workers[node as usize];
        let head = fs::read(self.root.join(format!("worker-{node}/root-step.head")))?;
        check(
            ack.binding == grant.binding
                && ack.logical_step_id == grant.logical_step_id
                && ack.event_id == grant.event_id
                && ack.grant_hash == grant.grant_hash
                && ack.logical_ack_hash == wire::ack_hash(&ack)
                && ack.durable_step_sequence == worker.steps + 1
                && ack
                    .witness_receipt
                    .as_ref()
                    .is_some_and(|r| r.sequence > 0 && r.hash.len() == 32)
                && ack.recovery_receipt.is_none()
                && matches!(
                    p::AckStatus::try_from(ack.status),
                    Ok(p::AckStatus::Quiescent | p::AckStatus::Rejected)
                )
                && head.len() == 88
                && &head[..8] == b"I1SHEAD1"
                && head[16..24] == ack.durable_step_sequence.to_be_bytes()
                && head[24..56] == ack.durable_step_hash
                && head[56..]
                    == hash_parts(
                        b"I1b/root-step-head/v1",
                        &[
                            &cfg.run_id,
                            &cfg.config_hash,
                            &node.to_be_bytes(),
                            &head[..56],
                        ],
                    ),
            format!("worker {node} invalid Ack or root commitment"),
        )?;
        worker.steps += 1;
        let runtime: Vec<Value> = serde_json::from_slice(&ack.runtime_events_json)?;
        if let Some(now) = runtime.iter().find_map(|row| {
            (row["caller"] == "manager/clock")
                .then(|| row["snapshot"]["local_time_us"].as_u64())
                .flatten()
        }) {
            check(
                now >= worker.last_manager_time_us && now.abs_diff(self.now.tick) <= 1000,
                "worker local Clock outside bound",
            )?;
            worker.last_manager_time_us = now;
        }
        self.log("AckCommitted", Some(node), json!({
            "logical_step_id": ack.logical_step_id, "durable_step_sequence": ack.durable_step_sequence,
            "root_hash": ack.durable_step_hash, "effects": ack.effects.len(),
        }))?;
        for effect in &ack.effects {
            if let Some(effect) = &effect.kind {
                self.effect(node, &cfg, &ack, effect)?;
            }
        }
        if self.m5.is_some() && ack.effects.iter().any(|effect| {
            matches!(&effect.kind,Some(effect::Kind::Lifecycle(e)) if e.event=="terminated" && e.reason=="LocalStop")
        }) {
            let worker=&mut self.workers[node as usize];
            check(worker.local_stop_ack.is_none() && worker.last_manager_time_us >= u64::try_from(cfg.base.stop)?,
                "invalid native LocalStop acknowledgement")?;
            worker.local_stop_ack=Some(ack.clone());
            worker.terminal=true;
            let local=worker.last_manager_time_us;
            self.log("LocalStopped",Some(node),json!({"cid":Cid::new(Hash32::new(cfg.cid)),
                "local_time_us":local,"local_stop_us":cfg.base.stop,"ack":ack.durable_step_hash,
                "logical_step_id":ack.logical_step_id}))?;
            self.commit_manager(node,ManagerInput::ObserveTime)?;
            self.schedule(self.now.tick,Event::Actions(node))?;
        }
        self.maybe_publish(node, &ack)?;
        if let input::Kind::Sample(sample) = kind {
            check(
                runtime.iter().any(|row| {
                    row["kind"] == "sample_accepted" && row["sequence"] == sample.sequence
                }),
                format!("worker {node} rejected healthy Clock sample"),
            )?;
            if !self.workers[node as usize].local_ready {
                self.workers[node as usize].local_ready = true;
                self.commit_manager(
                    node,
                    ManagerInput::ClockReady {
                        generation: 1,
                        basis_ref: root_hash(&ack)?,
                    },
                )?;
                self.schedule(
                    self.now.tick,
                    Event::Grant(node, input::Kind::DriveReady(p::Empty {})),
                )?;
            }
        }
        if matches!(
            grant.input.as_ref().and_then(|input| input.kind.as_ref()),
            Some(input::Kind::DriveReady(_))
        ) && let Some((pending, install)) = self.workers[node as usize].pending_prepare.take()
        {
            self.ready(node, pending, install, &ack)?;
        }
        Ok(ack)
    }

    fn ready(
        &mut self,
        node: u32,
        pending: ActionIntent,
        install: Install,
        ack: &p::Ack,
    ) -> M1Result<()> {
        let descriptor = install.descriptor;
        require_lifecycle(ack, descriptor.cid, "prepared")?;
        let ActionKind::Prepare {
            cid,
            instance_ref,
            anchor,
            provenance_ref,
            validation_version,
        } = pending.kind
        else {
            return Err(problem("wrong pending M1 Prepare action"));
        };
        let execution_bytes = install.execution.encode_checked()?;
        let execution_ref = Hash32::new(hash_parts(
            b"PoC/M1/verified-execution/v1",
            &[&execution_bytes, &ack.durable_step_hash],
        ));
        let prepared = PreparedRef {
            prepare_action: pending.id,
            worker: self.workers[node as usize].pid,
            cid,
            instance_ref,
            anchor,
            config_ref: descriptor.plan.config_ref,
            validation_version,
            execution_ref,
            provenance_ref,
        };
        self.log(
            "Ready",
            Some(node),
            json!({
                "cid": cid, "action": action_id(pending.id), "anchor": prepared.anchor,
                "state_root": install.execution.state.root()?, "execution_ref": execution_ref,
                "ack": ack.durable_step_hash, "logical_step_id": ack.logical_step_id,
            }),
        )?;
        self.commit_manager(node, ManagerInput::PreparationCompleted(prepared))?;
        self.commit_manager(node, ManagerInput::Schedule(descriptor.plan.session))?;
        self.schedule(self.now.tick, Event::Actions(node))
    }

    fn effect(
        &mut self,
        node: u32,
        cfg: &Resolved,
        ack: &p::Ack,
        effect: &effect::Kind,
    ) -> M1Result<()> {
        let member = node % 5;
        match effect {
            effect::Kind::ClockRequest(request) => {
                let delay = self.m5.map_or(CLOCK_TRANSIT_US,m5::Mode::transit);
                let worker = &mut self.workers[node as usize];
                check(
                    worker.request_seq + 1 == request.request_id,
                    "M1 clock request sequence",
                )?;
                worker.request_seq = request.request_id;
                self.log(
                    "ClockRequestTransit",
                    Some(node),
                    json!({
                        "request_id": request.request_id, "delay_us": delay,
                    }),
                )?;
                self.schedule(
                    self.now.tick + delay,
                    Event::Clock(node, request.request_id),
                )?;
                self.schedule(
                    self.now.tick + REFRESH_US,
                    Event::Grant(node, input::Kind::RequestSample(p::Empty {})),
                )?;
            }
            effect::Kind::TimerArm(timer) => {
                check(timer.cid == cfg.cid && timer.id > 0, "M1 unbound timer")?;
                check(
                    self.timers
                        .insert((node, timer.id), timer.clone())
                        .is_none(),
                    "M1 duplicate timer",
                )?;
                let local_to_tick = timer.local_deadline_us - cfg.profile(member).epsilon_us;
                let tick = self.now.tick.max(u64::try_from(local_to_tick.max(0))?);
                self.schedule(tick, Event::Timer(node, timer.clone()))?;
            }
            effect::Kind::TimerCancel(timer) => {
                check(
                    timer.cid == cfg.cid && timer.id > 0,
                    "M1 unbound cancelled timer",
                )?;
                self.timers.remove(&(node, timer.id));
            }
            effect::Kind::Work(work) => {
                check(
                    work.cid == cfg.cid
                        && work.operation_id.len() == 32
                        && work.duration_us == WORK_US
                        && self
                            .scheduled_work
                            .insert((node, work.operation_id.clone())),
                    "invalid or duplicate M1 work continuation",
                )?;
                self.workers[node as usize].last_work = Some(work.clone());
                self.schedule(
                    self.now.tick + work.duration_us,
                    Event::Grant(node, input::Kind::WorkCompleted(work.clone())),
                )?;
            }
            effect::Kind::Send(envelope) => self.route(node, cfg, ack, envelope.clone())?,
            effect::Kind::Sign(sign) => {
                check(
                    !self.workers[node as usize].terminal
                        && sign.cid == cfg.cid
                        && sign.grant_id == ack.logical_step_id
                        && sign.tick_us == self.now.tick
                        && (1..=VIEW_MAX).contains(&sign.view)
                        && sign.epoch == 1
                        && cfg.members[member as usize].signer_index == sign.signer_index
                        && sign.local_time_us.abs_diff(i64::try_from(self.now.tick)?) <= 1000
                        && sign.tick_us < sign.lease_end_tick_us
                        && sign.local_time_us < cfg.base.stop
                        && cfg.base.stop == sign.local_stop_us,
                    "unsafe M1 sign audit",
                )?;
                if self.m5.is_some() && self.governance_applied
                    && sign.kind==p::SignKind::Notarize as i32 && sign.action==p::SignAction::Created as i32
                    && (sign.view-1)%5==u64::from(member)
                {
                    use commonware_codec::{Copying,DecodeExt};
                    use commonware_consensus::simplex::types::Proposal;
                    use commonware_cryptography::sha256::Digest;
                    let proposal=checked(Proposal::<Digest>::decode(Copying(&sign.subject_bytes)))?;
                    self.log("NativeProposal",Some(node),json!({"cid":Cid::new(Hash32::new(cfg.cid)),
                        "physical_id":Hash32::new(proposal.payload.0),"view":sign.view,
                        "logical_step_id":ack.logical_step_id,"ack":ack.durable_step_hash}))?;
                }
            }
            effect::Kind::Proof(proof) => {
                check(
                    proof.cid == cfg.cid
                        && proof.epoch == 1
                        && (1..=VIEW_MAX).contains(&proof.view),
                    "proof in a foreign M1 context",
                )?;
                let digest = id32(&proof.payload_digest)?;
                self.workers[node as usize]
                    .regional
                    .proofs
                    .entry(digest)
                    .or_insert_with(|| proof.clone());
                self.log("ProofObserved", Some(node), json!({
                    "logical_step_id": ack.logical_step_id, "cid": Cid::new(Hash32::new(cfg.cid)),
                    "physical_id": Hash32::new(digest), "view": proof.view,
                }))?;
            }
            effect::Kind::Delivered(delivery) => {
                let block = PhysicalBlock::decode(&delivery.block_bytes)?;
                let digest = block.digest()?;
                check(
                    delivery.cid == cfg.cid
                        && delivery.digest == digest.as_bytes()
                        && delivery.parent_digest == block.parent.as_bytes()
                        && delivery.height == block.metadata.height,
                    "M1 delivered body identity mismatch",
                )?;
                let prior = self.workers[node as usize]
                    .regional
                    .bodies
                    .insert(delivery.height, block);
                check(prior.is_none(), "M1 archive released duplicate delivery")?;
                self.log("RegionalDelivered", Some(node), json!({
                    "logical_step_id": ack.logical_step_id, "cid": Cid::new(Hash32::new(cfg.cid)),
                    "height": delivery.height, "physical_id": digest,
                }))?;
            }
            effect::Kind::Lifecycle(value) => {
                check(
                    value.cid == cfg.cid
                        && ["prepared", "started", "terminated"].contains(&value.event.as_str()),
                    "unexpected M1 lifecycle effect",
                )?;
            }
            _ => return Err(problem("recovery effect outside healthy M1 profile")),
        }
        Ok(())
    }

    fn route(
        &mut self,
        node: u32,
        cfg: &Resolved,
        ack: &p::Ack,
        envelope: p::Envelope,
    ) -> M1Result<()> {
        native_network::validate_envelope(cfg, &envelope)?;
        check(
            envelope.src_member_id == node % 5,
            "M1 envelope worker source mismatch",
        )?;
        let expected = self
            .attempts
            .entry(envelope.message_id.clone())
            .or_default();
        check(
            *expected + 1 == envelope.attempt,
            "M1 envelope attempt replay",
        )?;
        *expected = envelope.attempt;
        let dest = cfg.public_pid(envelope.dst_member_id);
        let key = (node, dest, envelope.channel);
        let (prior, queued) = self.routes.entry(key).or_default();
        check(*queued < ROUTE_BOUND, "M1 link queue exhausted")?;
        let due = (self.now.tick + LINK_US).max(prior.saturating_add(1));
        *prior = due;
        *queued += 1;
        self.log("NetworkAdmitted", Some(node), json!({
            "dest": dest, "message_id": envelope.message_id, "attempt": envelope.attempt,
            "channel": envelope.channel, "due_tick_us": due, "logical_step_id": ack.logical_step_id,
        }))?;
        self.schedule(due, Event::Deliver(key, envelope))
    }

    fn maybe_publish(&mut self, node: u32, ack: &p::Ack) -> M1Result<()> {
        if self.publication_pending || self.workers[node as usize].terminal {
            return Ok(());
        }
        let selected = match &self.public.replay.public_state().tail {
            poc_viewer_gfr::Tail::Next(descriptor) => descriptor.clone(),
            _ if self.scenario.is_some() || self.resume => return Ok(()),
            _ => return Err(problem("M1 expected an assigned session")),
        };
        let height = (self.public.replay.public_state().entries.len() as u64 + 1)
            .max(selected.plan.window.h_min);
        let worker = &self.workers[node as usize];
        let Some(body) = worker.regional.bodies.get(&height) else {
            return Ok(());
        };
        let digest = body.digest()?;
        let Some(proof) = worker.regional.proofs.get(digest.as_bytes()) else {
            return Ok(());
        };
        let cfg = self.cfg(node)?;
        if selected.cid.as_bytes() != &cfg.cid {
            return Ok(());
        }
        check(
            self.public
                .finality
                .check(&selected, body, &proof.finalization_bytes),
            "real Final failed historical context verification",
        )?;
        let chain = (selected.plan.window.h_min..=height)
            .map(|height| {
                worker
                    .regional
                    .bodies
                    .get(&height)
                    .cloned()
                    .ok_or_else(|| problem("missing complete regional chain"))
            })
            .collect::<M1Result<Vec<_>>>()?;
        let result = self
            .public
            .result(selected, chain, proof.finalization_bytes.clone())?;
        let key = result.key();
        let bytes = result.encode_store();
        let result_ref = Hash32::new(hash_parts(
            b"PoC/M1/retained-result/v1",
            &[&bytes, &ack.durable_step_hash],
        ));
        let dir = self.root.join("results");
        durable_file(&dir.join(format!("{height:02}.bin")), &bytes)?;
        self.commit_manager(
            node,
            ManagerInput::RetainResult {
                key,
                verified_result_ref: result_ref,
            },
        )?;
        let worker = &self.workers[node as usize];
        let action = worker.manager.dispatchable(worker.last_manager_time_us).into_iter()
            .find(|action| matches!(action.kind, ActionKind::Publish { key: current, .. } if current == key))
            .ok_or_else(|| problem("Manager Publish action was not dispatchable"))?;
        let id = action.id;
        self.publication_pending = true;
        self.log("PublishScheduled", Some(node), json!({
            "action": action_id(id), "height": height, "cid": key.cid, "physical_id": key.block_hash,
            "ack": ack.durable_step_hash, "logical_step_id": ack.logical_step_id,
        }))?;
        let due = if self.force && self.scenario.is_none() && !self.resume && selected_branch(&result) == 0 && height == 11 {
            self.log("LatePublicationHeld", Some(node), json!({
                "cid": key.cid, "physical_id": key.block_hash, "height": height, "until_us": l1::ACT_US,
            }))?;
            l1::ACT_US
        } else {
            self.now.tick + PUBLIC_LINK_US
        };
        self.schedule(due, Event::Publish(node, id, result))
    }

    fn accepted_result(&self, block_id: Hash32) -> M1Result<BridgeResult> {
        let height = self.public.replay.public_state().entries.iter()
            .find(|entry| entry.block.physical_digest == Some(block_id))
            .map(|entry| entry.block.state.metadata.height)
            .ok_or_else(|| problem("missing G physical anchor"))?;
        let result = checked(BridgeResult::decode_store(&fs::read(self.root.join("results").join(format!("{height:02}.bin")))?))?;
        check(result.key().block_hash == block_id, "stored result does not match G")?;
        Ok(result)
    }

    fn replay(&mut self, finalized: Option<&BridgeResult>) -> M1Result<()> {
        let old = if self.resume {self.public.replay.public_state().entries.iter()
            .map(|row|row.block.native_hash).collect::<Vec<_>>()} else {vec![]};
        let (mut before, records) = self.public.advance()?;
        if let Some(effect)=records.iter().flat_map(|(_,event)| &event.record().effects)
            .find(|effect|matches!(effect,ReplayEffect::Reset{..})) {
            let ReplayEffect::Reset{resume_position,target_height,cut_digest}=effect else {unreachable!()};
            before=old.iter().zip(&self.public.replay.public_state().entries)
                .take_while(|(hash,row)|**hash==row.block.native_hash).count();
            self.log("GReset",None,json!({"resume_position":resume_position,"cut_digest":cut_digest,
                "old_height":old.len(),"common_height":before,"target_height":target_height,
                "old_hashes":&old[before..]}))?;
        }
        let rows = self.public.replay.public_state().entries[before..].to_vec();
        if !self.force {
            check(rows.len() == 1, "M1 expected exactly one new G block per finalized publication")?;
        }
        for (offset, row) in rows.iter().enumerate() {
            let source=match row.source {
                GlobalSource::Regional(source)=>source,
                GlobalSource::Derived{resume_position,height}=>{
                    self.log("GDerived",None,json!({"height":height,"resume_position":resume_position,
                        "native_hash":row.block.native_hash,"state_root":row.block.state.root()?,
                        "position":row.position}))?;
                    continue;
                }
            };
            let height = row.block.state.metadata.height;
            check(finalized.is_none_or(|result| self.resume || self.scenario.is_some() || source == result.key())
                && height == (before + offset + 1) as u64,
                "G append identity or height does not match its finalized result")?;
            if let Some(identity) = row.block.physical_digest {
                check(source.block_hash == identity && !row.block.native_block.kept.is_empty(),
                    "G physical block lost result identity or the fixture transaction")?;
                let physical = match finalized {
                    Some(result) if result.key() == source => result.clone(),
                    _ => self.accepted_result(identity)?,
                };
                self.log("GAccepted", None, json!({
                    "height": height, "cid": source.cid, "physical_id": identity,
                    "parent": physical.result().block.parent, "native_hash": row.block.native_hash,
                    "state_root": row.block.state.root()?, "position": row.position,
                }))?;
            } else {
                check(self.force && row.block.native_block.kept.is_empty(), "unexpected G padding")?;
                self.log("GPadding", None, json!({
                    "height": height, "cid": source.cid, "physical_id": source.block_hash,
                    "native_hash": row.block.native_hash, "state_root": row.block.state.root()?,
                    "position": row.position,
                }))?;
            }
        }
        for (sequence, delivery) in &records {
            self.record_delivery(*sequence, delivery, finalized)?;
        }
        if (self.scenario.is_some() || self.resume) && records.is_empty() {
            return Ok(());
        }
        let records: Arc<[_]> = records.into();
        for node in 0..15 {
            self.schedule(self.now.tick, Event::ApplyRecords(node, Arc::clone(&records)))?;
        }
        Ok(())
    }

    fn record_delivery(
        &mut self,
        seq: u64,
        delivery: &RecordDelivery,
        finalized: Option<&BridgeResult>,
    ) -> M1Result<()> {
        let record = delivery.record();
        let append = record.effects.iter().filter_map(|effect| match effect {
            ReplayEffect::Append { first, last, source: GlobalSource::Regional(key) } =>
                Some(json!({"first": first, "last": last, "cid":key.cid, "physical_id":key.block_hash})),
            _ => None,
        }).collect::<Vec<_>>();
        let mut detail = json!({
            "source_seq": seq, "id": delivery.id(), "position": record.position, "append": append,
            "events": record.events, "event_ids": delivery.event_ids(),
        });
        if let InputRef::L1(control) = &record.input_ref {
            detail["input"] = json!({
                "target_beta": control.force_branch(), "control_seq": control.control_seq,
                "position": control.position,
            });
        }
        if let Some(basis)=delivery.resume_material() {
            detail["resume"]=json!({"position":basis.position,"set_position":basis.set_position,
                "cut_digest":basis.digest,"target":basis.target});
        }
        self.log("GfrReplayRecord", None, detail)?;
        if let Some(basis)=delivery.resume_material() {
            let state=self.public.replay.public_state();
            if self.m5.is_some() {
                let descriptor = match &state.tail {
                    poc_viewer_gfr::Tail::Next(descriptor) => Some(descriptor),
                    _ => None,
                };
                if let Some(descriptor)=descriptor {
                    durable_file(&self.root.join("descriptor-b1-r0.bin"),&descriptor.encode_checked()?)?;
                }
                let activation=json!({"position":record.position,"resume_position":basis.position,
                    "set_position":basis.set_position,"record_id":delivery.id(),"descriptor":descriptor,
                    "cut_digest":basis.digest,"anchor_native_hash":basis.target.native_hash,
                    "target":basis.target,"l1_floor":state.l1_floor,"plan_basis_height":
                    descriptor.map(|d|state.plan_bases[&d.cid].height),
                    "halted":matches!(state.tail,poc_viewer_gfr::Tail::Halt{..})});
                self.pending_control=false;
                self.governance_applied=true;
                self.journal("Applied",activation.clone())?;
                self.log("ResumeActivated",None,activation)?;
                return Ok(());
            }
            let poc_viewer_gfr::Tail::Next(descriptor)=&state.tail else {return Err(problem("M4 missing assignment"));};
            check(descriptor.plan.region.get()==2 && descriptor.anchor==basis.target.anchor
                && descriptor.plan.window.h_min==21 && descriptor.plan.window.h_max==30,"M4 incorrect restoration")?;
            durable_file(&self.root.join("descriptor-b1-r0.bin"),&descriptor.encode_checked()?)?;
            self.log("ResumeActivated",None,json!({"position":record.position,"resume_position":basis.position,
                "set_position":basis.set_position,"record_id":delivery.id(),"descriptor":descriptor,
                "cut_digest":basis.digest,"anchor_native_hash":basis.target.native_hash,
                "plan_basis_height":state.plan_bases[&descriptor.cid].height,"l1_floor":state.l1_floor}))?;
            return Ok(());
        }
        if let [closed, assigned] = record.events.as_slice()
            && let (
                LifecycleBody::SessionClosed {
                    session,
                    reason: CloseReason::Complete,
                },
                LifecycleBody::SessionAssigned { descriptor },
            ) = (&closed.body, &assigned.body)
        {
            let anchored = match finalized {
                Some(result) if result.key().block_hash == descriptor.anchor.block.block_id => result.clone(),
                _ => self.accepted_result(descriptor.anchor.block.block_id)?,
            };
            let prior = anchored.result();
            check(
                &prior.descriptor.plan.session == session
                    && descriptor.anchor.block.block_id == anchored.key().block_hash,
                "M1 replay handoff did not bind accepted anchor",
            )?;
            self.history.push(AnchorStep {
                descriptor: prior.descriptor.clone(),
                chain: prior.chain.clone(),
                final_proof: prior.final_proof.clone(),
            });
            durable_file(
                &self
                    .root
                    .join(format!("descriptor-r{}.bin", descriptor.plan.session.r)),
                &descriptor.encode_checked()?,
            )?;
            let projections = record
                .events
                .iter()
                .map(lifecycle_projection)
                .collect::<std::result::Result<Vec<_>, _>>();
            self.log("Handoff", None, json!({
                "record_id": delivery.id(), "position": record.position, "old_cid": prior.descriptor.cid,
                "new_cid": descriptor.cid, "anchor": descriptor.anchor, "window": descriptor.plan.window,
                "reference_block": descriptor.plan.reference_block, "lifecycle_projection": projections?,
            }))?;
        } else if let [closed, assigned] = record.events.as_slice()
            && let (
                LifecycleBody::BranchClosed { branch, reason: CloseReason::Force },
                LifecycleBody::SessionAssigned { descriptor },
            ) = (&closed.body, &assigned.body)
        {
            let state = self.public.replay.public_state();
            let anchor = state.entries.iter().find(|entry|
                entry.block.physical_digest == Some(descriptor.anchor.block.block_id))
                .ok_or_else(|| problem("Force anchor is absent from G"))?;
            durable_file(&self.root.join(format!("descriptor-b{}-r{}.bin",
                descriptor.plan.session.beta, descriptor.plan.session.r)), &descriptor.encode_checked()?)?;
            let mut activation = json!({
                "position": record.position, "old_branch": branch, "new_branch": descriptor.plan.session.beta,
                "descriptor": descriptor, "anchor_native_hash": anchor.block.native_hash,
                "l1_floor": state.l1_floor, "head_height": descriptor.anchor.block.height,
                "espresso_q": self.public.next_q - 1, "record_id": delivery.id(),
            });
            if let Some(basis) = state.plan_bases.get(&descriptor.cid).copied() {
                activation["policy"] = json!({"enabled_mask":basis.active.policy.enabled_mask,
                    "order":basis.active.policy.order, "signed_floor":basis.active.policy.signed_floor,
                    "position":basis.active.force, "control_seq":basis.active.control_seq, "floor":basis.active.floor});
                activation["plan_basis"] = json!({"height":basis.height, "block_hash":basis.block_hash});
                let prior = self.accepted_result(descriptor.anchor.block.block_id)?;
                self.history.push(AnchorStep {
                    descriptor: prior.result().descriptor.clone(),
                    chain: prior.result().chain.clone(),
                    final_proof: prior.result().final_proof.clone(),
                });
                self.pending_control = false;
                self.governance_applied = true;
                self.journal("Applied", activation.clone())?;
            }
            self.log("ForceActivated", None, activation)?;
        } else if self.m5.is_some() && record.position.kind==ReplayKind::Expire {
            self.log("PublicHalt",None,json!({"position":record.position,"record_id":delivery.id()}))?;
        } else if self.scenario.is_some() {
            if let InputRef::L1(control) = &record.input_ref
                && self.pending_control
                && self.governance_intent.as_ref().is_some_and(|request| {
                    request.control_seq == control.control_seq
                        && Some(request.target_beta) == control.force_branch()
                })
            {
                let phase = if record.events.is_empty() { "Noop" } else { "Applied" };
                self.pending_control = false;
                self.governance_intent = None;
                self.governance_signatures.clear();
                self.journal(phase, json!({"position":record.position,"record_id":delivery.id()}))?;
            }
        } else {
            check(record.events.is_empty(), "unexpected public lifecycle event")?;
        }
        Ok(())
    }

    async fn run_inner(&mut self) -> M1Result<()> {
        self.spawn().await?;
        if self.force {
            self.schedule_l1()?;
        }
        fs::create_dir(self.root.join("results"))?;
        if let Some(mode)=self.m5 {
            self.journal("Observing",json!({"scenario":mode.name()}))?;
            self.schedule(10*SECOND,Event::Heartbeat)?;
            self.schedule(10*SECOND+gov::POLL,Event::GovernancePoll)?;
            self.schedule(11_500_000,Event::M4Batch(1))?;
            self.schedule(mode.stall(),Event::M5Stall)?;
            self.schedule(mode.recovery(),Event::M5Recover)?;
            if mode.recovery()>24*SECOND {
                self.schedule(24_100_000,Event::M5CoverageProbe)?;
            }
            if let Some(cutoff)=mode.contrast() {
                self.schedule(cutoff,Event::ContrastCutoff)?;
            }
            if mode != m5::Mode::Short {
                for node in 0..5 {
                    let offset= [-1000i64,-500,0,500,1000][node as usize];
                    self.schedule(u64::try_from(21_000_000-offset)?,Event::Grant(node,
                        input::Kind::RequestSample(p::Empty{})))?;
                }
            }
        } else if self.resume {
            self.schedule(10*SECOND, Event::Heartbeat)?;
            self.schedule(11_500_000,Event::M4Batch(1))?;
            self.schedule(15_750_000,Event::M4Batch(2))?;
            self.schedule(17*SECOND,Event::M4Resume)?;
        }
        if let Some(scenario) = self.scenario {
            self.journal("Observing", json!({"scenario":scenario.name()}))?;
            self.schedule(10 * SECOND, Event::Heartbeat)?;
            self.schedule(10 * SECOND + gov::POLL, Event::GovernancePoll)?;
            if scenario != Scenario::Healthy {
                self.schedule(gov::OUTAGE, Event::OfflineRegion)?;
            }
            if matches!(scenario, Scenario::OrderingStopped | Scenario::WatermarkFrozen) {
                self.schedule(16 * SECOND, Event::ContrastCutoff)?;
            }
        }
        let mut cutoff_scheduled = false;
        while let Some((&next, _)) = self.events.first_key_value() {
            let height = self.public.replay.public_state().entries.len();
            if self.reached_cutoff() && next.tick > self.now.tick {
                if cutoff_scheduled {
                    break;
                }
                let active: Vec<_> = self
                    .workers
                    .iter()
                    .filter(|worker| !worker.terminal && !worker.offline && ((self.scenario.is_none() && self.m5.is_none()) || worker.config.is_some()))
                    .map(|worker| u32::from(worker.pid.get()))
                    .collect();
                check(self.scenario.is_some() || self.m5.is_some() || active.len() == 5, "unexpected live committee at M1 cutoff")?;
                if active.is_empty() {
                    break;
                }
                for node in active {
                    self.schedule(self.now.tick, Event::CutoffRetire(node))?;
                }
                cutoff_scheduled = true;
                continue;
            }
            let (key, event) = self.events.pop_first().expect("peeked M1 next event");
            if key.tick > if self.scenario.is_some() || self.resume { 60 * SECOND } else { CUT_US } {
                return Err(problem(format!("M1 virtual cutoff at G height {height}")));
            }
            self.now = key;
            for worker in &mut self.workers {
                if !worker.offline && let Some(status) = worker.child.try_wait()? {
                    return Err(problem(format!(
                        "unplanned PID {} exit at virtual {}: {status}",
                        worker.os_pid, self.now.tick
                    )));
                }
            }
            match event {
                Event::Actions(node) => self.manager_actions(node).await?,
                Event::ApplyRecords(node, records) => {
                    let mut assigned = false;
                    for (sequence, delivery) in records.iter() {
                        assigned |= delivery.record().events.iter().any(|event| {
                            matches!(&event.body,
                                LifecycleBody::SessionAssigned { descriptor }
                                    if descriptor.plan.region == self.workers[node as usize].pid.region())
                        });
                        let input=VerifiedReplayInput {
                            source:self.source,seq:*sequence,record:delivery.record().clone(),validation_ref:delivery.id(),
                        };
                        let input=if input.record.position.kind==ReplayKind::Resume {
                            ManagerInput::ConsumeResume{input,material:delivery.resume_material().cloned()}
                        } else {ManagerInput::ConsumeRecord(input)};
                        self.commit_manager(node,input)?;
                    }
                    let delay = if assigned {
                        1_000 + u64::from(node % 5) * STARTUP_SKEW_US
                    } else {
                        0
                    };
                    self.schedule(self.now.tick + delay, Event::Actions(node))?;
                }
                Event::Grant(node, kind) => {
                    if self.workers[node as usize].offline { continue; }
                    if self.workers[node as usize].terminal
                        && matches!(kind, input::Kind::RequestSample(_) | input::Kind::Sample(_))
                    {
                        continue;
                    }
                    self.grant(node, kind).await?;
                }
                Event::Clock(node, request_id) => {
                    if self.workers[node as usize].offline || self.workers[node as usize].terminal {
                        continue;
                    }
                    let cfg = self.cfg(node)?;
                    let member = node % 5;
                    let profile = cfg.profile(member);
                    let worker = &mut self.workers[node as usize];
                    worker.sample_seq += 1;
                    let sample = p::ClockSample {
                        run_id: cfg.run_id.to_vec(),
                        worker_id: node,
                        incarnation: 1,
                        clock_domain: profile.clock_domain().to_vec(),
                        rate_profile: Profile::rate_profile().to_vec(),
                        request_id,
                        sequence: worker.sample_seq,
                        as_of_tick_us: self.now.tick,
                        node_time_at_as_of_us: i64::try_from(self.now.tick)? + profile.epsilon_us,
                        uncertainty_us: profile.epsilon_us.unsigned_abs(),
                        lease_end_tick_us: self.now.tick + LEASE_US,
                    };
                    self.log("ClockServiceSample", Some(node), json!(sample))?;
                    self.schedule(
                        self.now.tick + self.m5.map_or(CLOCK_TRANSIT_US,m5::Mode::transit),
                        Event::Grant(node, input::Kind::Sample(sample)),
                    )?;
                }
                Event::Timer(node, timer) => {
                    if self.timers.get(&(node, timer.id)) == Some(&timer)
                        && !self.workers[node as usize].terminal
                        && !self.workers[node as usize].offline
                    {
                        self.timers.remove(&(node, timer.id));
                        self.grant(node, input::Kind::TimerFired(timer)).await?;
                    }
                }
                Event::Deliver(key, envelope) => {
                    let route = self
                        .routes
                        .get_mut(&key)
                        .ok_or_else(|| problem("M1 pending route not found"))?;
                    route.1 -= 1;
                    if self.workers[key.1 as usize].offline || self.workers[key.0 as usize].offline {
                        self.log("NetworkDroppedOffline", Some(key.1), json!({"src":key.0,"message_id":envelope.message_id}))?;
                        continue;
                    }
                    self.log("NetworkDelivered", Some(key.1), json!({
                        "src": key.0, "channel": key.2, "message_id": envelope.message_id, "attempt": envelope.attempt,
                    }))?;
                    self.grant(key.1, input::Kind::Deliver(envelope)).await?;
                }
                Event::Publish(node, action, result) => {
                    if self.m5.is_some_and(|mode| self.now.tick < mode.recovery()
                        && self.now.tick + FINALIZE_US >= mode.stall()) {
                        self.log("LatePublicationHeld",Some(node),json!({"height":result.result().block.metadata.height,
                            "cid":result.key().cid,"physical_id":result.key().block_hash,"action":action_id(action)}))?;
                        check(self.m5_held.replace((node,action,result)).is_none(),"duplicate M5 publication hold")?;
                        continue;
                    }
                    if self.m5.is_some() && !matches!(&self.public.replay.public_state().tail,
                        poc_viewer_gfr::Tail::Next(d) if d.cid==result.key().cid)
                    {
                        self.log("LatePublicationExcluded",Some(node),json!({"height":result.result().block.metadata.height,
                            "cid":result.key().cid,"physical_id":result.key().block_hash,"action":action_id(action),
                            "reason":"responsibility_closed_before_dispatch"}))?;
                        self.publication_pending=false;
                        continue;
                    }
                    if (self.scenario.is_some() || self.resume) && !self.public.espresso_idle() {
                        self.schedule(self.now.tick + FINALIZE_US, Event::Publish(node, action, result))?;
                        continue;
                    }
                    if self.force && self.scenario.is_none() && !self.resume && self.now.tick == l1::ACT_US && self.wall_pause_ms != 0 {
                        tokio::time::sleep(Duration::from_millis(self.wall_pause_ms)).await;
                    }
                    let staged = self.public.propose(result, self.now.tick)?;
                    let key = staged.result.key();
                    let hash = Hash32::new(id32(&staged.dispatch_ref)?);
                    self.log("PublisherDispatched", Some(node), json!({
                        "action":action_id(action), "q":staged.q, "height":staged.result.result().block.metadata.height,
                        "cid":key.cid, "physical_id":key.block_hash, "dispatch_ref":hash,
                        "attempt": staged.submission.stable_id.attempt_number, "messages":staged.submission.exact_payload.len(),
                    }))?;
                    self.manager_receipt(node, action, hash, ReceiptOutcome::Published { key })?;
                    self.schedule(self.now.tick + FINALIZE_US, Event::Finalize(staged))?;
                }
                Event::Finalize(staged) => {
                    let result = staged.result.clone();
                    let detail = self.public.finalize(staged, self.now.tick)?;
                    self.log("PublisherRecorded", None, detail)?;
                    self.replay(Some(&result))?;
                    if self.scenario.is_some() || self.m5.is_some() {
                        let block = self.public.finalized_row(self.public.next_q - 1)?;
                        self.advertise_espresso(block)?;
                        if self.governance_applied && selected_branch(&result) > 0 {
                            self.journal("RecoveryObserved", json!({"height":result.result().block.metadata.height,"cid":result.key().cid}))?;
                        }
                    }
                    if self.force && self.scenario.is_none() && !self.resume && selected_branch(&result) == 0
                        && result.result().block.metadata.height == 11
                    {
                        check(self.public.replay.public_state().branch == 1
                            && self.public.replay.public_state().entries.len() == 10,
                            "same-key old Final changed G or won priority over Force")?;
                        self.log("LatePublicationExcluded", None, json!({
                            "q": self.public.next_q - 1, "height": 11,
                            "cid": result.key().cid, "physical_id": result.key().block_hash,
                            "head_height": 10, "branch": 1,
                        }))?;
                    }
                    self.publication_pending = false;
                }
                Event::L1(event) => self.run_l1(event)?,
                Event::M4Batch(height)=>self.m4_batch(height)?,
                Event::M4Resume=>self.m4_resume().await? ,
                Event::M5Stall => {
                    let tip=self.public.espresso_tip().ok_or_else(||problem("M5 no Espresso tip at fault"))?;
                    self.log("OrderingFault",None,json!({"scenario":self.m5.map(m5::Mode::name),
                        "q":tip.0,"theta_us":tip.1.0}))?;
                }
                Event::M5Recover => {
                    self.log("OrderingRecovered",None,json!({"scenario":self.m5.map(m5::Mode::name)}))?;
                    if let Some((node,action,result))=self.m5_held.take() {
                        self.schedule(self.now.tick+100_000,Event::Publish(node,action,result))?;
                    }
                }
                Event::M5CoverageProbe => {
                    let through=ReplayPosition { key_us:24*SECOND, kind:ReplayKind::Expire,
                        index:ReplayIndex::Height(2) };
                    let result=self.public.coverage_probe(&through)?;
                    check(matches!(result.status,poc_viewer_gfr::ReplayStatus::Wait{..})
                        && result.state.public_state().pc==self.public.replay.public_state().pc,
                        "M5 uncovered Expire did not WAIT at original replay position")?;
                    self.log("GfrCoverageWait",None,json!({"through":through,"status":format!("{:?}",result.status),
                        "pc":result.state.public_state().pc}))?;
                }
                Event::M5Release=>self.broadcast_governance(false)?,

                Event::OfflineRegion => self.offline_region().await?,
                Event::Heartbeat => self.heartbeat()?,
                Event::FinalizeHeartbeat(q) => {
                    let row = self.public.finalize_heartbeat(q, self.now.tick)?;
                    self.log("EspressoHeartbeat", None, json!({"q": q, "theta_us": row.proposal.theta_us.0,
                        "message_count": row.proposal.messages.len()}))?;
                    if self.m5.is_some() {
                        self.log("OrderingWatermark",None,json!({"q":q,"theta_us":row.proposal.theta_us.0,
                            "live_at_proposal":row.proposal.live_at_proposal}))?;
                    }
                    self.replay(None)?;
                    self.advertise_espresso(row)?;
                }
                Event::GovernancePoll => {
                    self.schedule(self.now.tick + self.m5.map_or(CLOCK_TRANSIT_US,m5::Mode::transit), Event::GovernanceClock(self.now.tick))?;
                    self.schedule(self.now.tick + gov::POLL, Event::GovernancePoll)?;
                }
                Event::GovernanceClock(sample_us) => {
                    for signer in &mut self.governors {
                        signer.sample_clock(sample_us, self.now.tick);
                    }
                    if self.m5.is_some() {self.m5_poll(false)?;}
                    self.schedule(self.now.tick, Event::GovernanceEvaluate)?;
                }
                Event::GovernanceEvaluate => if self.m5.is_some() {self.m5_poll(true)?;} else {self.poll_governance()?;},
                Event::GovernanceL1(signer, block) => {
                    let header = self.governors[signer].l1.receive(&block, &self.public.inputs)?;
                    self.log("GovernanceL1Available", None, json!({"signer":signer,"height":header.height,"hash":header.hash}))?;
                }
                Event::GovernanceEspresso(signer, row) => {
                    checked(self.governors[signer].espresso.receive_block(SOURCE, (*row).clone(), TickUs(self.now.tick)))?;
                    self.log("GovernanceEspressoAvailable", None,
                        json!({"signer":signer,"q":row.proposal.q,"theta_us":row.proposal.theta_us.0}))?;
                    if self.m5.is_some() {self.m5_poll(false)?;}
                }
                Event::GovernanceSign(signer, request) => self.sign_governance(signer, request)?,
                Event::GovernanceSubmit(request) => self.submit_governance(request).await?,
                Event::GovernanceResolve => self.broadcast_governance(true)?,
                Event::ContrastCutoff => self.contrast_complete = true,
                Event::CloseProbe(node, work) => {
                    check(self.workers[node as usize].terminal, "Force probe before Close")?;
                    let ack = self.grant(node, input::Kind::WorkCompleted(work.clone())).await?;
                    let app: Value = serde_json::from_slice(&ack.application_events_json)?;
                    let discarded = app["work"].as_array().is_some_and(|items| items.iter().any(|item|
                        item["event"] == "work_discarded"
                            && item["reason"] == "unknown_or_retired_work"
                            && item["work"]["operation_id"] == json!(work.operation_id)
                    ));
                    check(discarded && ack.effects.iter().all(|effect| !matches!(
                        effect.kind, Some(effect::Kind::Sign(_) | effect::Kind::Send(_)))),
                        "old Force instance executed post-terminal work or emitted a new output")?;
                    self.log("CloseProbe", Some(node), json!({
                        "cid": Cid::new(Hash32::new(id32(&work.cid)?)),
                        "operation_id": work.operation_id, "discarded": true,
                        "reason": "unknown_or_retired_work", "ack": ack.durable_step_hash,
                        "logical_step_id": ack.logical_step_id,
                    }))?;
                }
                Event::CutoffRetire(node) => {
                    let cfg = self.cfg(node)?;
                    let cid = Cid::new(Hash32::new(cfg.cid));
                    // Retire process-local actors under a committed grant after the observation
                    // cut. The already accepted GFR position and Manager duty do not change.
                    self.workers[node as usize].terminal = true;
                    let ack = self
                        .grant(
                            node,
                            input::Kind::Close(p::Cid {
                                cid: cfg.cid.to_vec(),
                            }),
                        )
                        .await?;
                    require_lifecycle(&ack, cid, "terminated")?;
                    self.log(
                        "CutoffRetire",
                        Some(node),
                        json!({
                            "cid": cid, "ack": ack.durable_step_hash,
                            "logical_step_id": ack.logical_step_id, "height": height,
                        }),
                    )?;
                }
            }
        }
        if self.m5.is_some() {return check(self.reached_cutoff(),"M5 acceptance cutoff not reached");}
        if self.resume {return check(self.reached_cutoff() && self.conflict_seen,"M4 cutoff failed");}
        if self.scenario.is_some() {
            return check(self.reached_cutoff(), "M3 acceptance cutoff not reached");
        }
        check(
            self.public.replay.public_state().entries.len() == self.stop_height()
                && self.history.len() == if self.force { 1 } else { 2 }
                && self
                    .workers
                    .iter()
                    .all(|worker| worker.config.is_some() && worker.terminal),
            "incomplete healthy smoke",
        )
    }

    fn stop_height(&self) -> usize {
        if self.force && !self.resume { 16 } else { 21 }
    }

    fn schedule_l1(&mut self) -> M1Result<()> {
        self.schedule(0, Event::L1(l1::Event::Genesis))?;
        self.schedule(5 * SECOND, Event::L1(l1::Event::Deploy))?;
        if self.scenario.is_none() && !self.resume {
            self.schedule(l1::SUBMIT_US, Event::L1(l1::Event::Submit))?;
            self.schedule(13 * SECOND, Event::L1(l1::Event::Retry))?;
        }
        for height in 1..=if self.scenario.is_some() || self.resume { 30 } else { 13 } {
            let inclusion = height * l1::L1_SLOT_US;
            self.schedule(inclusion, Event::L1(l1::Event::Mine(height)))?;
            self.schedule(inclusion + l1::FINALITY_US, Event::L1(l1::Event::Finalize(height)))?;
        }
        Ok(())
    }

    fn run_l1(&mut self, event: l1::Event) -> M1Result<()> {
        let port = self.l1.as_mut().ok_or_else(|| problem("no L1 driver"))?;
        let detail = port.step(event.command(self.now.tick))?;
        if let l1::Event::Deliver(height, reader)=event
            && let Some(signer)=m5::SIGNER_READERS.iter().position(|name| *name==reader)
        {
            let h=self.governors[signer].l1.receive(&detail,&self.public.inputs)?;
            check(h.height==height,"wrong M5 signer L1 height")?;
            self.governors[signer].d=if detail["derivation"]["observation"].is_null() {None}
                else {Some(serde_json::from_value(detail["derivation"]["observation"].clone())?)};
            self.log("GovernanceL1Available",None,json!({"signer":signer,"reader":reader,
                "height":height,"hash":h.hash,"d":self.governors[signer].d}))?;
            self.m5_poll(false)?;
            return Ok(());
        }
        match event {
            l1::Event::Genesis => {
                if self.m5.is_some() {
                    self.advertised_l1=Some(0);
                    for (id,name) in m5::SIGNER_READERS.into_iter().enumerate() {
                        self.schedule(self.now.tick+id as u64*10_000,Event::L1(l1::Event::Deliver(0,name)))?;
                    }
                }
                if self.scenario.is_some() {
                    self.advertised_l1 = Some(0);
                    self.advertise_l1(0, detail.clone())?;
                }
                let header = self.public.deliver_anvil(&detail)?;
                self.reader_b.receive(&detail, &self.public.inputs)?;
                for reader in ["A", "B"] {
                    self.log("L1ReaderAvailable", None, json!({
                        "reader": reader, "height": 0, "hash": header.hash,
                    }))?;
                }
            }
            l1::Event::Deploy | l1::Event::Submit | l1::Event::Retry => {
                let transactions = detail.as_array().ok_or_else(|| problem("missing submitted transactions"))?;
                for transaction in transactions {
                    let mut log = transaction.clone();
                    log["head_height"] = json!(self.public.replay.public_state().entries.len());
                    if matches!(event, l1::Event::Submit) {
                        check(self.public.replay.public_state().branch == 0
                            && self.public.replay.public_state().entries.len() == 1,
                            "governance did not submit during healthy R0 production")?;
                        log["reason"] = json!("scheduled_healthy_force");
                    }
                    self.log("L1Submitted", None, log)?;
                }
            }
            l1::Event::Mine(height) => {
                let before_inbox=self.included_cursor;
                if self.resume && let Some(cursor)=detail["cursor_at_end"]["cursor"]["height"].as_u64() {
                    self.included_cursor=cursor;
                }
                self.log("L1Included", None, json!({"height": height, "timestamp_s":detail["timestamp"],"hash":detail["hash"]}))?;
                if let Some(hash) = self.submitted_control.as_deref()
                    && let Some(receipt) = detail["receipts"].as_array().and_then(|rows|
                        rows.iter().find(|row| row["transactionHash"] == hash))
                {
                    if receipt["status"] == "0x1" {
                        self.journal("Included", json!({"height":height,"receipt":receipt}))?;
                        if let Some(plan)=self.m5_solution {
                            let t=self.now.tick;
                            let inside=plan.accepts_inclusion(t);
                            let g=self.public.replay.public_state().entries.len() as u64;
                            let d=self.derived_head.as_ref().map(|d|d.state.metadata.height)
                                .ok_or_else(||problem("M5 requires an observed D at inclusion"))?;
                            if inside {
                                check(g<=plan.g_max && d<=plan.d_max && before_inbox<=plan.inbox_max,
                                    "M5 violated its stated head growth bounds")?;
                            }
                            self.log("ResumeTiming",None,json!({"height":height,
                                "t_l_min_us":plan.t_l_min_us,"t_l_max_us":plan.t_l_max_us,
                                "actual_inclusion_us":t,"act_us":t+10*SECOND,"inside_interval":inside,
                                "transaction_hash":receipt["transactionHash"],"solution":plan,
                                "g_before":g,"d_verified_before":d,"inbox_before":before_inbox}))?;
                        }
                    } else {
                        self.pending_control = false;
                        self.journal("Failed", json!({"height":height,"receipt":receipt}))?;
                    }
                }
            },
            l1::Event::Finalize(height) => {
                if self.scenario.is_some() || self.m5.is_some() {
                    self.advertised_l1 = Some(height);
                }
                self.log("L1Finalized", None, json!({"height": height}))?;
                self.schedule(self.now.tick, Event::L1(l1::Event::Deliver(height, "A")))?;
                self.schedule(self.now.tick + 10_000, Event::L1(l1::Event::Deliver(height, "B")))?;
                if self.m5.is_some() {
                    for (id,name) in m5::SIGNER_READERS.into_iter().enumerate() {
                        self.schedule(self.now.tick+id as u64*10_000,Event::L1(l1::Event::Deliver(height,name)))?;
                    }
                }
            }
            l1::Event::Deliver(height, reader) => {
                let header = if reader == "A" {
                    if self.scenario.is_some() {
                        self.advertise_l1(height, detail.clone())?;
                        if detail["events"].as_array().is_some_and(|events| events.iter().any(|event|event["event"]=="force")) {
                            self.journal("Finalized", json!({"height":height,"control":detail["events"]}))?;
                        }
                    }
                    if self.m5.is_some() && detail["events"].as_array().is_some_and(|events|
                        events.iter().any(|event|event["event"]=="resume"))
                    {
                        self.journal("Finalized",json!({"height":height,"control":detail["events"]}))?;
                    }
                    self.public.deliver_anvil(&detail)?
                } else {
                    self.reader_b.receive(&detail, &self.public.inputs)?
                };
                self.log("L1ReaderAvailable", None, json!({
                    "reader": reader, "height": height, "hash": header.hash,
                }))?;
                if self.resume {
                    self.log("DerivationAvailable",None,json!({"reader":reader,"l1_height":height,
                        "head":detail["derivation"]["head"],"h1":detail["derivation"]["h1"],
                        "h2":detail["derivation"]["h2"],"wait":detail["derivation"]["wait"],
                        "cuts":detail["derivation"]["cuts"].as_array().map(|cuts| cuts.iter().map(|cut|
                            json!({"file":cut["file"],"position":cut["position"],"digest":cut["digest"]})).collect::<Vec<_>>())}))?;
                    if reader=="A" {
                        if !detail["derivation"]["head"].is_null() {
                            self.derived_head=Some(serde_json::from_value(detail["derivation"]["head"].clone())?);
                        }
                        if height==7 && self.m5.is_none() {
                            let current=&self.public.replay.public_state().entries;
                            let h1:Hash32=serde_json::from_value(detail["derivation"]["h1"].clone())?;
                            let h2:Hash32=serde_json::from_value(detail["derivation"]["h2"].clone())?;
                            check(current[0].block.native_hash==h1 && current[1].block.native_hash!=h2,
                                "M4 expected ordinary D h1 and conflicting fallback D h2")?;
                            let g2=current[1].block.native_hash;
                            self.conflict_seen=true;
                            self.log("DerivationConflict",None,json!({"height":2,"g_hash":g2,"d_hash":h2,"common_h1":h1}))?;
                        }
                    }
                    if let Some(cuts)=detail["derivation"]["cuts"].as_array() {
                        use poc_viewer_gfr::L1ControlReader;
                        for cut in cuts {
                            let pos:L1Position=serde_json::from_value(cut["position"].clone())?;
                            let a=self.public.public_l1().resume_cut(pos);
                            let b=self.reader_b.resume_cut(pos);
                            if let (InputStatus::Ready(a),InputStatus::Ready(b))=(a,b) {
                                check(a==b,"M4 independent reader cut mismatch")?;
                            }
                        }
                    }
                }
                if reader == "A" && (self.scenario.is_some() || self.resume) {
                    self.replay(None)?;
                }
            }
        }
        Ok(())
    }

    fn m4_batch(&mut self,height:u64)->M1Result<()> {
        use batch_services::source::FixtureSnapshot;
        let state=self.public.replay.public_state();
        let index=usize::try_from(height-1)?;
        let row=state.entries.get(index).ok_or_else(||problem("M4 target batch not committed in G"))?;
        check(!row.block.native_block.kept.is_empty(),"M4 requires ordinary transactions")?;
        let (parent,native_parent)=if index==0 {
            (state.genesis.state.clone(),state.genesis.native_hash)
        } else {(state.entries[index-1].block.state.clone(),state.entries[index-1].block.native_hash)};
        let l1=self.public.public_l1().finalized_tip().ok_or_else(||problem("M4 no finalized L1"))?;
        let snapshot=FixtureSnapshot {
            source_fixture:super::m4::SOURCE_FIXTURE.into(),
            gpos:checked(poc_viewer_gfr::encode_position(&row.position))?,
            position:row.position.clone(),config:self.public.inputs.config.clone(),dep:self.public.inputs.dep.clone(),
            parent,native_parent,blocks:vec![row.block.virtual_block.clone()],
            control_coverage:L1Position {block:l1.height,tx:u64::MAX,log:u64::MAX},processed_resumes:vec![],
        };
        let value=self.l1.as_mut().expect("M4 port").step(json!({"op":"m4_prepare_batch","tick_us":self.now.tick,
            "snapshot":snapshot}))?;
        self.log("BatchSubmitted",None,value)
    }

    async fn m4_resume(&mut self)->M1Result<()> {
        let state=self.public.replay.public_state();
        let d=self.derived_head.as_ref().ok_or_else(||problem("M4 no verified D head"))?.clone();
        check(self.conflict_seen && state.entries.len()<20 && d.state.metadata.height<20
            && self.included_cursor<20 && self.public.espresso_tip().is_some_and(|(_,theta)|
                theta.0<=self.now.tick && self.now.tick-theta.0<SECOND),
            "M4 scheduled governance prerequisites")?;
        let request=json!({"op":"m4_resume","tick_us":self.now.tick,"target":20,"origin":9,
            "g_head":state.entries.len(),"d_head":d.state.metadata.height,"inbox_height":self.included_cursor,
            "eshot":self.public.espresso_tip().map(|(q,theta)|(q,theta.0))});
        self.log("ResumePreset",None,request.clone())?;
        if self.wall_pause_ms>0 {tokio::time::sleep(Duration::from_millis(self.wall_pause_ms)).await;}
        let result=self.l1.as_mut().expect("M4 port").step(request)?;
        self.log("ResumeSubmitted",None,result)
    }

    fn reached_cutoff(&self) -> bool {
        let state = self.public.replay.public_state();
        if let Some(mode)=self.m5 {
            if mode.contrast().is_some() {
                return self.contrast_complete && (self.submitted_control.is_some()==(mode==m5::Mode::Divergence));
            }
            if mode==m5::Mode::Late {return self.governance_applied && matches!(state.tail,poc_viewer_gfr::Tail::Halt{..});}
            return self.governance_applied && state.entries.last().is_some_and(|entry| {
                matches!(entry.source,GlobalSource::Regional(key) if state.selected.values()
                    .any(|d|d.cid==key.cid && d.plan.session.beta>0))
            });
        }
        if self.resume {
            return state.branch==1 && state.entries.len()==21
                && matches!(state.entries[20].source,GlobalSource::Regional(key) if state.selected.get(&(1,0)).is_some_and(|d|d.cid==key.cid));
        }
        match self.scenario {
            None => state.entries.len() == self.stop_height(),
            Some(Scenario::Healthy) => state.entries.len() >= 11 && self.submitted_control.is_none()
                && self.governance_signatures.is_empty(),
            Some(Scenario::OrderingStopped | Scenario::WatermarkFrozen) =>
                self.contrast_complete && self.submitted_control.is_none() && self.governance_signatures.is_empty(),
            Some(Scenario::Outage | Scenario::ReaderLag) => {
                self.governance_applied && state.entries.last().is_some_and(|entry| {
                    matches!(entry.source,GlobalSource::Regional(key) if state.selected.values().any(|d|d.cid==key.cid && d.plan.session.beta>0))
                })
            }
        }
    }

    async fn offline_region(&mut self) -> M1Result<()> {
        use std::os::unix::process::ExitStatusExt;
        let state = self.public.replay.public_state();
        let selected = initial_descriptor(&self.public.replay)?;
        check(state.entries.len() == 1 && selected.plan.region.get() == 0, "M3 outage must follow h1 on R0")?;
        let cid = selected.cid;
        for node in 0..5 {
            let worker = &mut self.workers[node];
            worker.child.start_kill()?;
            let status = tokio::time::timeout(WATCHDOG, worker.child.wait()).await??;
            check(status.signal() == Some(9), "unexpected worker outage exit")?;
            worker.offline = true;
            worker.tx.take();
            let info = json!({"os_pid":worker.os_pid,"signal":status.signal(),"cid":cid,"region":0});
            self.log("WorkerOffline", Some(node as u32), info)?;
        }
        Ok(())
    }

    fn heartbeat(&mut self) -> M1Result<()> {
        if let Some(mode)=self.m5 {
            let stalled=(mode.stall()..mode.recovery()).contains(&self.now.tick);
            let crosses_start=self.now.tick<mode.stall() && self.now.tick+FINALIZE_US>=mode.stall();
            if crosses_start || (stalled && mode!=m5::Mode::Frozen) {
                return self.schedule(self.now.tick+gov::HEARTBEAT,Event::Heartbeat);
            }
            if !self.public.espresso_idle() {
                return self.schedule(self.now.tick+FINALIZE_US,Event::Heartbeat);
            }
            let theta=if stalled {
                self.public.espresso_tip().map(|(_,theta)|theta.0)
            } else {None};
            let q=self.public.heartbeat_live(self.now.tick,theta,!stalled)?;
            self.schedule(self.now.tick+FINALIZE_US,Event::FinalizeHeartbeat(q))?;
            return self.schedule(self.now.tick+gov::HEARTBEAT,Event::Heartbeat);
        }
        let scenario = self.scenario.unwrap_or(Scenario::Healthy);
        if scenario == Scenario::OrderingStopped && self.now.tick >= gov::OUTAGE {
            return Ok(());
        }
        if !self.public.espresso_idle() {
            self.schedule(self.now.tick + FINALIZE_US, Event::Heartbeat)?;
            return Ok(());
        }
        let frozen = if scenario == Scenario::WatermarkFrozen && self.now.tick >= gov::OUTAGE {
            self.public.espresso_tip().map(|(_, theta)| theta.0)
        } else {
            None
        };
        let q = self.public.heartbeat(self.now.tick, frozen)?;
        self.schedule(self.now.tick + FINALIZE_US, Event::FinalizeHeartbeat(q))?;
        self.schedule(self.now.tick + gov::HEARTBEAT, Event::Heartbeat)
    }

    fn advertise_l1(&mut self, _height: u64, block: Value) -> M1Result<()> {
        let block = Arc::new(block);
        for id in 0..self.governors.len() {
            self.schedule(self.now.tick + id as u64 * 10_000, Event::GovernanceL1(id, Arc::clone(&block)))?;
        }
        Ok(())
    }

    fn advertise_espresso(&mut self, row: FinalizedBlock) -> M1Result<()> {
        let row = Arc::new(row);
        for id in 0..self.governors.len() {
            let mut delivery = self.now.tick + id as u64 * 10_000;
            if self.m5==Some(m5::Mode::InputsWait) && id>0
                && self.now.tick>=m5::Mode::InputsWait.recovery()
            {
                continue;
            }
            if self.scenario == Some(Scenario::ReaderLag) && id > 0 && self.now.tick >= gov::OUTAGE {
                let release = if id == 1 { 13_650_000 } else { 13_800_000 };
                delivery = delivery.max(release + row.proposal.q * 10 + id as u64);
            }
            self.schedule(delivery, Event::GovernanceEspresso(id, Arc::clone(&row)))?;
        }
        Ok(())
    }

    fn advertised(&self) -> gov::Advertised {
        gov::Advertised { l1: self.advertised_l1, espresso: self.public.espresso_tip() }
    }

    fn m5_poll(&mut self, poll: bool) -> M1Result<()> {
        let advertised=self.advertised();
        let slots=if self.m5==Some(m5::Mode::NoTarget) {2} else {1};
        let mut votes=Vec::new();
        for id in 0..self.governors.len() {
            let (detail, request)=self.governors[id].poll_resume(
                &self.public,advertised,self.now.tick,self.pending_control,slots)?;
            if let Some(request)=request { votes.push((id,request,detail["solution"].clone())); }
            self.log(if poll {"GovernancePoll"} else {"ResumeInputEvaluation"},None,detail)?;
        }
        if self.pending_control || self.governance_applied {return Ok(());}
        let Some((_,request,solution))=votes.iter().find(|(_,request,_)| {
            votes.iter().filter(|(_,other,_)|other==request).count()>=gov::THRESHOLD
        }).cloned() else {
            return Ok(());
        };
        if self.m5_eligibility.as_ref().is_none_or(|(_,previous)|previous!=&request) {
            self.m5_eligibility=Some((self.now.tick,request.clone()));
            self.log("ResumeEligibility",None,json!({"t_eligible_us":self.now.tick,"request":request,
                "signers":votes.iter().filter_map(|(id,vote,_)|(vote==&request).then_some(id))
                    .collect::<Vec<_>>(),"solution":solution}))?;
        }
        if !poll || self.governance_intent.as_ref()==Some(&request) {return Ok(());}
        self.governance_intent=Some(request.clone());
        self.governance_signatures.clear();
        self.m5_solution=Some(serde_json::from_value(solution.clone())?);
        self.journal("Proposed",json!({"request":request,"solution":solution,
            "t_eligible_us":self.m5_eligibility.as_ref().map(|(tick,_)|tick)}))?;
        for (id,vote,_) in votes {
            if vote==request {
                self.schedule(self.now.tick+gov::SIGN_US,Event::GovernanceSign(id,vote))?;
            }
        }
        Ok(())
    }

    fn poll_governance(&mut self) -> M1Result<()> {
        let advertised = self.advertised();
        for id in 0..self.governors.len() {
            let (detail, vote) = self.governors[id].poll(
                &self.public, advertised, self.now.tick, self.pending_control, true,
            )?;
            self.log("GovernancePoll", None, detail)?;
            let Some(vote) = vote else { continue; };
            if self.governance_intent.as_ref() != Some(&vote) {
                self.governance_intent = Some(vote.clone());
                self.governance_signatures.clear();
                self.journal("Proposed", json!(&vote))?;
            }
            if !self.governance_signatures.contains_key(&id) {
                self.schedule(self.now.tick + gov::SIGN_US, Event::GovernanceSign(id, vote))?;
            }
        }
        Ok(())
    }

    fn journal(&mut self, phase: &str, detail: Value) -> M1Result<()> {
        let mut file = File::options().create(true).append(true).open(self.root.join("governance.jsonl"))?;
        serde_json::to_writer(&mut file, &json!({"phase":phase,"tick_us":self.now.tick,"detail":&detail}))?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        File::open(&self.root)?.sync_all()?;
        self.log("GovernanceState", None, json!({"phase":phase,"evidence":detail}))
    }

    fn sign_governance(&mut self, signer: usize, request: gov::Request) -> M1Result<()> {
        if self.pending_control || self.governance_signatures.contains_key(&signer)
            || self.governance_intent.as_ref() != Some(&request) {
            return Ok(());
        }
        let advertised = self.advertised();
        let (_, decision) = if self.m5.is_some() {
            self.governors[signer].poll_resume(&self.public,advertised,self.now.tick,false,1)?
        } else { self.governors[signer].poll(&self.public, advertised,self.now.tick,false,false)? };
        if decision.as_ref() != Some(&request) {
            return Ok(());
        }
        let response = self.l1.as_mut().ok_or_else(|| problem("M3 L1 port missing"))?
            .step(json!({"op":"governance_sign","signer":signer,"request":request,"tick_us":self.now.tick}))?;
        check(response["signer"] == signer as u64 && response["request"] == json!(&request),
            "signer output not bound to request")?;
        self.governance_signatures.insert(signer, response.clone());
        self.log("GovernanceSignature", None, response)?;
        if self.governance_signatures.len() == gov::THRESHOLD {
            self.journal("Signed", json!({"request":request,"signatures":self.governance_signatures}))?;
            self.schedule(self.now.tick + gov::BROADCAST_US, Event::GovernanceSubmit(request))?;
        }
        Ok(())
    }

    async fn submit_governance(&mut self, request: gov::Request) -> M1Result<()> {
        if self.pending_control || self.governance_intent.as_ref() != Some(&request) {
            return Ok(());
        }
        let advertised = self.advertised();
        let mut signers = Vec::new();
        for (&id, signature) in &self.governance_signatures {
            let (_, decision) = if self.m5.is_some() {
                self.governors[id].poll_resume(&self.public,advertised,self.now.tick,false,1)?
            } else { self.governors[id].poll(&self.public,advertised,self.now.tick,false,false)? };
            if decision.as_ref() == Some(&request) {
                signers.push(signature.clone());
            }
        }
        if signers.len() < gov::THRESHOLD {
            self.governance_intent = None;
            self.governance_signatures.clear();
            self.journal("ProposedSuperseded", json!(&request))?;
            return Ok(());
        }
        signers.truncate(gov::THRESHOLD);
        if self.m5.is_some() {
            let eligible=self.m5_eligibility.as_ref().filter(|(_,vote)|vote==&request)
                .map(|(tick,_)|*tick).ok_or_else(||problem("M5 missing eligible quorum"))?;
            let elapsed=self.now.tick.saturating_sub(eligible);
            check(elapsed<=gov::POLL+gov::SIGN_US+gov::BROADCAST_US,"M5 governance submission missed its budget")?;
            self.log("ResumeSubmissionBudget",None,json!({"t_eligible_us":eligible,
                "prepare_us":self.now.tick,"elapsed_us":elapsed,"budget_us":
                gov::POLL+gov::SIGN_US+gov::BROADCAST_US,"expected_inclusion_us":
                self.m5_solution.map(|p|[p.t_l_min_us,p.t_l_max_us])}))?;
        }

        if self.wall_pause_ms != 0 {
            tokio::time::sleep(Duration::from_millis(self.wall_pause_ms)).await;
        }
        let prepared = self.l1.as_mut().ok_or_else(|| problem("M3 L1 port missing"))?
            .step(json!({"op":"governance_prepare","request":request,"signers":signers,"tick_us":self.now.tick}))?;
        let hash = prepared["hash"].as_str().ok_or_else(|| problem("prepared transaction has no hash"))?;
        self.submitted_control = Some(hash.to_owned());
        self.pending_control = true;
        self.journal("Prepared", prepared.clone())?;
        self.log("GovernanceSubmission", None, prepared)?;
        if self.m5==Some(m5::Mode::Late) {
            self.journal("HeldBeforeDriver",json!({"release_us":m5::LATE_RELEASE_US,
                "transaction_hash":self.submitted_control}))?;
            self.schedule(m5::LATE_RELEASE_US,Event::M5Release)
        } else {
            self.broadcast_governance(false)
        }
    }

    fn broadcast_governance(&mut self, reconcile: bool) -> M1Result<()> {
        let op = if reconcile { "governance_reconcile" } else { "governance_broadcast" };
        let result = self.l1.as_mut().ok_or_else(|| problem("M3 L1 port missing"))?
            .step(json!({"op":op,"tick_us":self.now.tick}))?;
        let hash = self.submitted_control.as_ref().ok_or_else(|| problem("no durable control to broadcast"))?;
        check(result["hash"] == *hash, "broadcast changed request transaction")?;
        match result["status"].as_str() {
            Some("ready") => {
                self.journal("Submitted", result)?;
                Ok(())
            }
            Some("wait") => {
                self.journal("SubmittedUnknown", result)?;
                self.schedule(self.now.tick + 10_000, Event::GovernanceResolve)
            }
            _ => {
                self.pending_control = false;
                self.journal("Failed", result)?;
                Err(problem("governance broadcast rejected"))
            }
        }
    }

    fn complete(&mut self) -> M1Result<()> {
        let entries = &self.public.replay.public_state().entries;
        let height = if self.scenario.is_some() || self.m5.is_some() { entries.len() } else { self.stop_height() };
        let tip = entries.get(height.saturating_sub(1)).ok_or_else(|| problem("no G tip at cutoff"))?;
        let result=match tip.source {
            GlobalSource::Regional(result)=>Some(result),
            GlobalSource::Derived{..} if self.m5==Some(m5::Mode::Late)=>None,
            _=>return Err(problem("cutoff requires a native result")),
        };
        let summary = json!({
            "status": "Completed", "height": height, "seed": SEED,
            "cid": result.map(|row|row.cid), "physical_id": result.map(|row|row.block_hash), "native_hash": tip.block.native_hash,
            "state_root": tip.block.state.root()?, "position": tip.position,
            "virtual_cutoff_us": self.now.tick, "worker_count": self.workers.len(),
            "pending_events": self.events.len(),
        });
        self.log("SmokeComplete", None, summary.clone())?;
        durable_file(
            &self.root.join("summary.json"),
            &serde_json::to_vec_pretty(&summary)?,
        )?;
        Ok(())
    }
}

fn selected_branch(result: &BridgeResult) -> u64 {
    result.result().descriptor.plan.session.beta
}

fn id32(bytes: &[u8]) -> M1Result<Id> {
    bytes
        .try_into()
        .map_err(|_| problem("M1 expected a 32-byte identifier"))
}

fn root_hash(ack: &p::Ack) -> M1Result<Hash32> {
    Ok(Hash32::new(id32(&ack.durable_step_hash)?))
}

fn require_lifecycle(ack: &p::Ack, cid: Cid, event: &str) -> M1Result<()> {
    check(ack.effects.iter().filter(|effect| matches!(&effect.kind,
        Some(effect::Kind::Lifecycle(value)) if value.cid == cid.as_bytes() && value.event == event
    )).count() == 1, format!("missing committed {event} lifecycle effect"))
}
