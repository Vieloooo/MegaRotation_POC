use poc_publication_bridge::{BridgeResult, RecordDelivery};
use poc_session_manager::{
    ActionId, ActionIntent, ActionKind, ActionReceipt, CommitRef, Manager, ManagerBinding,
    ManagerInput, PlanOutcome, PreparedRef, ReceiptOutcome, VerifiedGenesis, VerifiedReplayInput,
};
use poc_types::domain::*;
use poc_viewer_gfr::{GlobalSource, ReplayEffect, lifecycle_projection};
use prost::Message;
use serde_json::{Value, json};
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
    M1Result, check, checked, problem,
    profile::*,
    public::{Public, Submitted, initial_descriptor},
    worker::{Launch, durable_file},
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
    regional: Regional,
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
    ApplyRecord(u32, u64, Arc<RecordDelivery>),
    Grant(u32, p::input::Kind),
    Clock(u32, u64),
    Timer(u32, p::Timer),
    Deliver(Route, p::Envelope),
    Publish(u32, ActionId, BridgeResult),
    Finalize(Submitted),
    CutoffRetire(u32),
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
}

pub(crate) async fn run(root: PathBuf) -> M1Result<()> {
    fs::create_dir(&root)?;
    let root = root.canonicalize()?;
    let public = Public::new(&root)?;
    let source = Hash32::new(hash_parts(b"PoC/M1/GFR/source/v1", &[SOURCE, L1_SOURCE]));
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
    };
    let execution = hub.run_inner().await;
    let exits = hub.reap(execution.is_ok()).await;
    let result = execution.and(exits).and_then(|()| hub.complete());
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
        durable_file(
            &self.root.join("profile.json"),
            &serde_json::to_vec_pretty(&json!({
                "schema": "M1-HEALTHY-SMOKE-1", "seed": SEED, "start_us": START_US,
                "cutoff_us": CUT_US, "announcement_us": parameters().announcement_us,
                "grace_us": GRACE_US, "expiry_us": EXPIRY_US, "clock_rpc_one_way_us": CLOCK_TRANSIT_US,
                "clock_refresh_us": REFRESH_US, "clock_lease_us": LEASE_US,
                "epsilon_us_by_member": [-1000,-500,0,500,1000],
                "link_delay_us": LINK_US, "work_us": WORK_US, "local_enter_spacing_us": STARTUP_SKEW_US,
                "publisher_link_us": PUBLIC_LINK_US, "espresso_finalize_delay_us": FINALIZE_US,
            }))?,
        )?;
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
                .arg("--worker")
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
                regional: Regional::default(),
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
            let worker = &mut self.workers[node];
            let early_status = worker.child.try_wait()?;
            worker.tx.take();
            if !healthy && early_status.is_none() {
                let _ = worker.child.start_kill();
            }
            let status = tokio::time::timeout(WATCHDOG, worker.child.wait()).await??;
            successful &= status.success() && early_status.is_none();
            let os_pid = worker.os_pid;
            self.log("WorkerExit", Some(node as u32), json!({
                "os_pid": os_pid, "exit": status.code(), "expected": healthy && early_status.is_none(),
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
            ManagerInput::ConsumeRecord(input) => (
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
                    let ack = self
                        .grant(
                            node,
                            input::Kind::Close(p::Cid {
                                cid: cid.as_bytes().to_vec(),
                            }),
                        )
                        .await?;
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
        self.log(
            "Grant",
            Some(node),
            json!({"logical_step_id":grant.logical_step_id,
            "grant_hash": grant.grant_hash, "input": kind}),
        )?;
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
                        "request_id": request.request_id, "delay_us": CLOCK_TRANSIT_US,
                    }),
                )?;
                self.schedule(
                    self.now.tick + CLOCK_TRANSIT_US,
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
        let height = self.public.replay.public_state().entries.len() as u64 + 1;
        let worker = &self.workers[node as usize];
        let Some(body) = worker.regional.bodies.get(&height) else {
            return Ok(());
        };
        let digest = body.digest()?;
        let Some(proof) = worker.regional.proofs.get(digest.as_bytes()) else {
            return Ok(());
        };
        let selected = initial_descriptor(&self.public.replay)?.clone();
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
        self.schedule(
            self.now.tick + PUBLIC_LINK_US,
            Event::Publish(node, id, result),
        )
    }

    fn replay(&mut self, finalized: &BridgeResult) -> M1Result<()> {
        let (before, records) = self.public.advance()?;
        let rows = self.public.replay.public_state().entries[before..].to_vec();
        check(
            rows.len() == 1,
            "M1 expected exactly one new G block per finalized publication",
        )?;
        let row = &rows[0];
        let GlobalSource::Regional(source) = row.source;
        let identity = row
            .block
            .physical_digest
            .ok_or_else(|| problem("M1 G contained padding"))?;
        check(
            source == finalized.key()
                && source.block_hash == identity
                && row.block.state.metadata.height == before as u64 + 1
                && !row.block.native_block.kept.is_empty(),
            "M1 G append identity or state-changing transaction missing",
        )?;
        self.log("GAccepted", None, json!({
            "height": row.block.state.metadata.height, "cid": source.cid, "physical_id": identity,
            "parent": finalized.result().block.parent, "native_hash": row.block.native_hash,
            "state_root": row.block.state.root()?, "position": row.position,
        }))?;
        for (sequence, delivery) in records {
            self.record_delivery(sequence, Arc::new(delivery), finalized)?;
        }
        Ok(())
    }

    fn record_delivery(
        &mut self,
        seq: u64,
        delivery: Arc<RecordDelivery>,
        finalized: &BridgeResult,
    ) -> M1Result<()> {
        let record = delivery.record();
        let append = record.effects.iter().filter_map(|effect| match effect {
            ReplayEffect::Append { first, last, source: GlobalSource::Regional(key) } =>
                Some(json!({"first": first, "last": last, "cid":key.cid, "physical_id":key.block_hash})),
            _ => None,
        }).collect::<Vec<_>>();
        self.log("GfrReplayRecord", None, json!({
            "source_seq": seq, "id": delivery.id(), "position": record.position, "append": append,
            "events": record.events, "event_ids": delivery.event_ids(),
        }))?;
        if let [closed, assigned] = record.events.as_slice()
            && let (
                LifecycleBody::SessionClosed {
                    session,
                    reason: CloseReason::Complete,
                },
                LifecycleBody::SessionAssigned { descriptor },
            ) = (&closed.body, &assigned.body)
        {
            let prior = finalized.result();
            check(
                &prior.descriptor.plan.session == session
                    && descriptor.anchor.block.block_id == finalized.key().block_hash,
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
        } else {
            check(
                record.events.is_empty(),
                "M1 unexpected public lifecycle event",
            )?;
        }
        for node in 0..15 {
            self.schedule(
                self.now.tick,
                Event::ApplyRecord(node, seq, delivery.clone()),
            )?;
        }
        Ok(())
    }

    async fn run_inner(&mut self) -> M1Result<()> {
        self.spawn().await?;
        fs::create_dir(self.root.join("results"))?;
        let mut cutoff_scheduled = false;
        while let Some((&next, _)) = self.events.first_key_value() {
            let height = self.public.replay.public_state().entries.len();
            if height == 21 && next.tick > self.now.tick {
                if cutoff_scheduled {
                    break;
                }
                let active: Vec<_> = self
                    .workers
                    .iter()
                    .filter(|worker| !worker.terminal)
                    .map(|worker| u32::from(worker.pid.get()))
                    .collect();
                check(active.len() == 5, "unexpected live committee at M1 cutoff")?;
                for node in active {
                    self.schedule(self.now.tick, Event::CutoffRetire(node))?;
                }
                cutoff_scheduled = true;
                continue;
            }
            let (key, event) = self.events.pop_first().expect("peeked M1 next event");
            if key.tick > CUT_US {
                return Err(problem(format!("M1 virtual cutoff at G height {height}")));
            }
            self.now = key;
            for worker in &mut self.workers {
                if let Some(status) = worker.child.try_wait()? {
                    return Err(problem(format!(
                        "unplanned PID {} exit at virtual {}: {status}",
                        worker.os_pid, self.now.tick
                    )));
                }
            }
            match event {
                Event::Actions(node) => self.manager_actions(node).await?,
                Event::ApplyRecord(node, sequence, delivery) => {
                    let assigned =
                        delivery
                            .record()
                            .events
                            .iter()
                            .find_map(|event| match &event.body {
                                LifecycleBody::SessionAssigned { descriptor } => {
                                    Some(descriptor.plan.region)
                                }
                                _ => None,
                            });
                    self.commit_manager(
                        node,
                        ManagerInput::ConsumeRecord(VerifiedReplayInput {
                            source: self.source,
                            seq: sequence,
                            record: delivery.record().clone(),
                            validation_ref: delivery.id(),
                        }),
                    )?;
                    let delay = if assigned == Some(self.workers[node as usize].pid.region()) {
                        1_000 + u64::from(node % 5) * STARTUP_SKEW_US
                    } else {
                        0
                    };
                    self.schedule(self.now.tick + delay, Event::Actions(node))?;
                }
                Event::Grant(node, kind) => {
                    if self.workers[node as usize].terminal
                        && matches!(kind, input::Kind::RequestSample(_) | input::Kind::Sample(_))
                    {
                        continue;
                    }
                    self.grant(node, kind).await?;
                }
                Event::Clock(node, request_id) => {
                    if self.workers[node as usize].terminal {
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
                        self.now.tick + CLOCK_TRANSIT_US,
                        Event::Grant(node, input::Kind::Sample(sample)),
                    )?;
                }
                Event::Timer(node, timer) => {
                    if self.timers.get(&(node, timer.id)) == Some(&timer)
                        && !self.workers[node as usize].terminal
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
                    self.log("NetworkDelivered", Some(key.1), json!({
                        "src": key.0, "channel": key.2, "message_id": envelope.message_id, "attempt": envelope.attempt,
                    }))?;
                    self.grant(key.1, input::Kind::Deliver(envelope)).await?;
                }
                Event::Publish(node, action, result) => {
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
                    self.replay(&result)?;
                    self.publication_pending = false;
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
        check(
            self.public.replay.public_state().entries.len() == 21
                && self.history.len() == 2
                && self
                    .workers
                    .iter()
                    .all(|worker| worker.config.is_some() && worker.terminal),
            "incomplete healthy M1 smoke",
        )
    }

    fn complete(&mut self) -> M1Result<()> {
        let entries = &self.public.replay.public_state().entries;
        let height21 = &entries[20];
        let GlobalSource::Regional(result) = height21.source;
        let summary = json!({
            "status": "Completed", "height": 21, "seed": SEED,
            "cid": result.cid, "physical_id": result.block_hash, "native_hash": height21.block.native_hash,
            "state_root": height21.block.state.root()?, "position": height21.position,
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
