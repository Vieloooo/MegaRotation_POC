use crate::{
    Id, hash_parts,
    native_application::Block,
    native_config::{self as settings, Launch, NativeScheme, Resolved},
    native_network::{self, PacketFactory, RawVote},
    native_wire::{
        self as p, body_packet, effect,
        envelope::Provenance,
        hub_frame, input,
        native_run_server::{NativeRun, NativeRunServer},
        worker_frame, *,
    },
    native_wire_support as wire,
};
use commonware_codec::Encode as _;
use commonware_consensus::{
    Epochable, Viewable,
    simplex::types::{Attributable, Certificate, Finalization, Finalize, Notarize, Proposal, Vote},
};
use commonware_cryptography::{bls12381::primitives::group::Private, sha256::Digest};
use commonware_parallel::Sequential;
use prost::Message;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs::{self, File, OpenOptions},
    io::{BufWriter, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{net::TcpListener, sync::mpsc};
use tokio_stream::wrappers::{ReceiverStream, TcpListenerStream};
use tonic::{Request, Response, Status, Streaming, transport::Server};

type Failure = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Failure>;
const WATCHDOG: Duration = Duration::from_secs(30);
fn error(message: impl Into<String>) -> Failure {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.into()).into()
}
fn ensure(ok: bool, message: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(error(message)) }
}

type Output = mpsc::Sender<std::result::Result<HubFrame, Status>>;
enum Incoming {
    Ready(u32, Output),
    Ack(u32, Ack),
    Lost(u32, String),
}
#[derive(Clone)]
struct Service {
    expected: Arc<Resolved>,
    registration: Arc<Mutex<[u8; 2]>>,
    broker: mpsc::Sender<Incoming>,
}
#[tonic::async_trait]
impl NativeRun for Service {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> std::result::Result<Response<RegisterReply>, Status> {
        let registration = request.into_inner();
        let node = registration
            .binding
            .as_ref()
            .map(|b| b.worker_id)
            .ok_or_else(|| Status::invalid_argument("binding"))?;
        if node > 1 || registration != self.expected.registration(node) {
            return Err(Status::invalid_argument("identity"));
        }
        let mut state = self.registration.lock().unwrap();
        if state[node as usize] != 0 {
            return Err(Status::already_exists("duplicate register"));
        }
        state[node as usize] = 1;
        Ok(Response::new(RegisterReply {
            binding: registration.binding,
            clock_domain: self.expected.profile(node).clock_domain().to_vec(),
        }))
    }
    type RunStream = ReceiverStream<std::result::Result<HubFrame, Status>>;
    async fn run(
        &self,
        request: Request<Streaming<WorkerFrame>>,
    ) -> std::result::Result<Response<Self::RunStream>, Status> {
        let mut stream = request.into_inner();
        let Some(WorkerFrame {
            frame: Some(worker_frame::Frame::Hello(hello)),
        }) = stream.message().await?
        else {
            return Err(Status::invalid_argument("expected Hello"));
        };
        let node = hello
            .binding
            .as_ref()
            .map(|b| b.worker_id)
            .ok_or_else(|| Status::invalid_argument("binding"))?;
        if node > 1 || self.expected.registration(node) != hello {
            return Err(Status::invalid_argument("invalid stream identity"));
        }
        {
            let mut state = self.registration.lock().unwrap();
            if state[node as usize] != 1 {
                return Err(Status::failed_precondition(
                    "unregistered or duplicate stream",
                ));
            }
            state[node as usize] = 2;
        }
        let (send, receive) = mpsc::channel(4);
        let broker = self.broker.clone();
        broker
            .send(Incoming::Ready(node, send))
            .await
            .map_err(|_| Status::unavailable("hub stopped"))?;
        tokio::spawn(async move {
            let reason = loop {
                match stream.message().await {
                    Ok(Some(WorkerFrame {
                        frame: Some(worker_frame::Frame::Ack(ack)),
                    })) => {
                        if broker.send(Incoming::Ack(node, ack)).await.is_err() {
                            return;
                        }
                    }
                    Ok(None) => break "EOF".to_owned(),
                    Ok(Some(_)) => break "unexpected worker frame".to_owned(),
                    Err(error) => break error.to_string(),
                }
            };
            let _ = broker.send(Incoming::Lost(node, reason)).await;
        });
        Ok(Response::new(ReceiverStream::new(receive)))
    }
}

struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
fn spawn_worker(request: &Launch, endpoint: &str) -> Result<ChildGuard> {
    let launch = request.work_dir.join("launch-private.json");
    let mut file = OpenOptions::new();
    file.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        file.mode(0o600);
    }
    serde_json::to_writer(file.open(&launch)?, request)?;
    let executable = std::env::current_exe()?
        .with_file_name(format!("native-worker{}", std::env::consts::EXE_SUFFIX));
    ensure(
        executable.is_file(),
        "build native-worker next to native-hub",
    )?;
    Ok(ChildGuard(
        Command::new(executable)
            .arg(endpoint)
            .arg(&launch)
            .stdin(Stdio::null())
            .stdout(File::create(request.work_dir.join("stdout"))?)
            .stderr(File::create(request.work_dir.join("stderr"))?)
            .spawn()?,
    ))
}

#[derive(Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    tick_us: u64,
    causal_wave: u64,
    tie_rank: Id,
    event_id: u64,
}
enum Event {
    Grant(u32, p::Input),
    ClockService(u32, u64),
    Route(Route),
    Timer(u32, Timer),
    Lease(u32, u64),
}
type Route = (u32, u32, i32);
#[derive(Clone, Default)]
struct Worker {
    tx: Option<Output>,
    started: bool,
    closed: bool,
    accepted_lease: Option<u64>,
    clock_sequence: u64,
    last_request: u64,
    step_sequence: u64,
    height: u64,
    delivered_tip: Option<Id>,
    witness: Option<Receipt>,
    proofs: BTreeSet<Id>,
    first_proof: Option<u64>,
    first_delivery: Option<u64>,
    close_reason: Option<String>,
}
struct Hub {
    config: Arc<Resolved>,
    scripts: Vec<Script>,
    events: BTreeMap<Key, Event>,
    next_event: u64,
    next_step: u64,
    workers: [Worker; 2],
    bus: mpsc::Receiver<Incoming>,
    routes: BTreeMap<Route, VecDeque<(u64, Envelope)>>,
    timers: BTreeMap<(u32, u64), Timer>,
    requested_close: BTreeSet<u32>,
    attempts: BTreeMap<Vec<u8>, u32>,
    accepted_acks: BTreeMap<(u32, u64), Ack>,
    overflows: u64,
    trace: BufWriter<File>,
    frames: BufWriter<File>,
}

impl Hub {
    fn record(&mut self, kind: &str, key: Option<&Key>, detail: Value) -> Result<()> {
        serde_json::to_writer(
            &mut self.trace,
            &json!({"kind":kind,"event":key,"detail":detail}),
        )?;
        writeln!(self.trace)?;
        self.trace.flush()?;
        Ok(())
    }
    fn raw(&mut self, direction: u8, node: u32, frame: &impl Message) -> Result<()> {
        let bytes = frame.encode_to_vec();
        self.frames.write_all(&[direction, node as u8])?;
        self.frames.write_all(&(bytes.len() as u64).to_be_bytes())?;
        self.frames.write_all(&bytes)?;
        self.frames.flush()?;
        Ok(())
    }
    fn schedule(&mut self, parent: Option<&Key>, tick: u64, event: Event) -> Result<()> {
        let key = event_key(self.config.base.seed, &mut self.next_event, parent, tick)?;
        self.events.insert(key, event);
        Ok(())
    }
    async fn await_registration(&mut self) -> Result<()> {
        let deadline = tokio::time::Instant::now() + WATCHDOG;
        while self.workers.iter().any(|worker| worker.tx.is_none()) {
            match tokio::time::timeout_at(deadline, self.bus.recv()).await? {
                Some(Incoming::Ready(node, tx)) if self.workers[node as usize].tx.is_none() => {
                    self.workers[node as usize].tx = Some(tx)
                }
                Some(Incoming::Lost(node, reason)) => {
                    return Err(error(format!(
                        "worker {node} disconnected before registration: {reason}"
                    )));
                }
                _ => return Err(error("unexpected frame while awaiting native Hello")),
            }
        }
        Ok(())
    }
    fn route(&mut self, key: &Key, envelope: Envelope) -> Result<()> {
        let (due, log) = stage_packet(
            &self.config,
            key,
            &mut self.routes,
            &mut self.attempts,
            envelope,
        )?;
        self.overflows += u64::from(log.0 == "QueueOverflow");
        self.record(log.0, Some(key), log.1)?;
        if let Some((tick, route)) = due {
            self.schedule(Some(key), tick, Event::Route(route))?;
        }
        Ok(())
    }
    async fn run(&mut self) -> Result<()> {
        self.await_registration().await?;
        for worker in 0..2 {
            self.record("Registered", None, json!(self.config.registration(worker)))?;
            let tick = self.config.base.start + u64::from(worker) * self.config.base.startup_skew;
            self.schedule(
                None,
                tick,
                Event::Grant(worker, make(input::Kind::RequestSample(Empty {}))),
            )?;
        }
        if let Some(tick) = self.config.base.stress_tick {
            self.schedule(
                None,
                tick,
                Event::Grant(0, make(input::Kind::DriveReady(Empty {}))),
            )?;
        }
        while let Some((key, event)) = self.events.pop_first() {
            if key.tick_us > self.config.base.cutoff {
                break;
            }
            while let Ok(frame) = self.bus.try_recv() {
                match frame {
                    Incoming::Lost(node, reason) => {
                        return Err(error(format!(
                            "unplanned native worker {node} exit: {reason}"
                        )));
                    }
                    Incoming::Ack(node, ack) => self.duplicate(node, &ack)?,
                    _ => return Err(error("out-of-grant native worker traffic")),
                }
            }
            let tick = key.tick_us;
            match event {
                Event::Grant(node, input) => self.grant(&key, node, input).await?,
                Event::ClockService(node, request) => {
                    let index = node as usize;
                    self.workers[index].clock_sequence += 1;
                    let sample = ClockSample {
                        run_id: self.config.run_id.to_vec(),
                        worker_id: node,
                        incarnation: 1,
                        clock_domain: self.config.profile(node).clock_domain().to_vec(),
                        rate_profile: crate::Profile::rate_profile().to_vec(),
                        request_id: request,
                        sequence: self.workers[index].clock_sequence,
                        as_of_tick_us: tick,
                        node_time_at_as_of_us: i64::try_from(tick)?
                            + self.config.base.epsilon[index],
                        uncertainty_us: self.config.base.epsilon[index].unsigned_abs(),
                        lease_end_tick_us: tick + self.config.base.lease,
                    };
                    self.record("ClockServiceSample", Some(&key), json!(&sample))?;
                    if self
                        .config
                        .base
                        .lose_clock_after
                        .is_some_and(|limit| tick >= limit)
                    {
                        self.record("ClockResponseLost", Some(&key), json!(sample))?;
                    } else {
                        self.schedule(
                            Some(&key),
                            tick + self.config.base.clock_transit,
                            Event::Grant(node, make(input::Kind::Sample(sample))),
                        )?;
                    }
                }
                Event::Lease(node, end) => {
                    if self.workers[node as usize].accepted_lease == Some(end)
                        && !self.workers[node as usize].closed
                    {
                        self.grant(&key, node, make(input::Kind::DriveReady(Empty {})))
                            .await?;
                    }
                }
                Event::Timer(node, timer) => {
                    if self.timers.get(&(node, timer.id)) == Some(&timer)
                        && !self.workers[node as usize].closed
                    {
                        self.timers.remove(&(node, timer.id));
                        self.grant(&key, node, make(input::Kind::TimerFired(timer)))
                            .await?;
                    } else {
                        self.record(
                            "TimerSuppressed",
                            Some(&key),
                            json!({"node":node,"timer":timer}),
                        )?;
                    }
                }
                Event::Route(route) => {
                    let queue = self
                        .routes
                        .get_mut(&route)
                        .ok_or_else(|| error("unknown native route"))?;
                    let (_, packet) = queue
                        .pop_front()
                        .ok_or_else(|| error("native route missing envelope"))?;
                    if let Some((due, _)) = queue.front() {
                        let due = (*due).max(tick);
                        self.schedule(Some(&key), due, Event::Route(route))?;
                    }
                    self.record("NetworkDelivered", Some(&key), json!(&packet))?;
                    let dest = packet.dst_member_id;
                    if dest < 2 {
                        self.grant(&key, dest, make(input::Kind::Deliver(packet)))
                            .await?;
                    } else if let Some(script) = self.scripts.get_mut((dest - 2) as usize) {
                        let fault = script.fault(&packet)?;
                        let responses = script.receive(packet)?;
                        if let Some(fault) = fault {
                            self.record("InjectedNativeDelivery", Some(&key), json!(&fault))?;
                            if fault.cid == self.config.cid {
                                self.route(&key, fault)?;
                            } else {
                                self.schedule(
                                    Some(&key),
                                    tick + self.config.base.link_delay,
                                    Event::Grant(
                                        fault.dst_member_id,
                                        make(input::Kind::Deliver(fault)),
                                    ),
                                )?;
                            }
                        }
                        for response in responses {
                            self.record("ScriptedSend", Some(&key), json!(&response))?;
                            self.route(&key, response)?;
                        }
                    } else {
                        self.record("ScriptedPeerOffline", Some(&key), json!(packet))?;
                    }
                }
            }
            if self.config.base.scenario == "healthy"
                && self
                    .workers
                    .iter()
                    .all(|worker| worker.height == self.config.base.height_limit)
            {
                for node in 0..2 {
                    if self.requested_close.insert(node) {
                        self.schedule(
                            Some(&key),
                            tick,
                            Event::Grant(
                                node,
                                make(input::Kind::Close(Cid {
                                    cid: self.config.cid.to_vec(),
                                })),
                            ),
                        )?;
                    }
                }
                if self.workers.iter().all(|worker| worker.closed) {
                    break;
                }
            }
        }
        if self.config.base.scenario == "healthy" {
            ensure(
                self.workers
                    .iter()
                    .all(|worker| worker.height == self.config.base.height_limit),
                "healthy run did not deliver at both real workers",
            )?;
        }
        if self.config.base.scenario == "two-real-alone" {
            ensure(
                self.workers.iter().all(|worker| worker.height == 0),
                "two real engines unexpectedly finalized",
            )?;
        }
        let scenario = self.config.base.scenario.as_str();
        match scenario {
            "proof-before-body" => {
                let worker = &self.workers[1];
                let hold = self
                    .config
                    .base
                    .body_hold_until
                    .expect("proof-before-body hold");
                ensure(
                    worker.first_proof.is_some_and(|tick| tick < hold)
                        && worker.first_delivery.is_some_and(|tick| tick >= hold),
                    "proof-before-body control did not execute",
                )?;
            }
            "close-propose" | "close-verify" | "close-certify" => {
                ensure(
                    self.requested_close.iter().any(|node| {
                        self.workers[*node as usize].close_reason.as_deref() == Some("Close")
                    }),
                    "requested WorkStarted close control did not execute",
                )?;
            }
            "lease-loss" => {
                ensure(
                    self.workers
                        .iter()
                        .all(|worker| worker.close_reason.as_deref() == Some("ClockUnsafe")),
                    "lost lease did not retire both native workers",
                )?;
            }
            "wrong-context" | "native-proof-negatives" => {
                let expected = if scenario == "wrong-context" {
                    "wrong_envelope_binding"
                } else {
                    "invalid_native_certificate"
                };
                ensure(
                    self.accepted_acks.values().any(|ack| {
                        ack.status == AckStatus::Rejected as i32
                            && serde_json::from_slice::<Value>(&ack.application_events_json)
                                .ok()
                                .and_then(|value| value["work"].as_array().cloned())
                                .is_some_and(|work| {
                                    work.iter().any(|entry| {
                                        entry["event"] == "rejected_input"
                                            && entry["reason"] == expected
                                    })
                                })
                    }),
                    "native negative input was not explicitly rejected",
                )?;
            }
            "router-overflow" => {
                ensure(self.overflows > 0, "native router stress did not overflow")?
            }
            "quota" => {
                ensure(
                    self.accepted_acks.values().any(|ack| {
                        serde_json::from_slice::<Value>(&ack.application_events_json)
                            .ok()
                            .and_then(|value| value["network"].as_array().cloned())
                            .is_some_and(|rows| {
                                rows.iter().any(|entry| {
                                    entry["event"] == "rate_limited"
                                        && entry["channel"] == Channel::Body as i32
                                })
                            })
                    }),
                    "native quota stress did not exercise rate limit",
                )?;
            }
            _ => {}
        }
        Ok(())
    }

    fn duplicate(&mut self, node: u32, ack: &Ack) -> Result<()> {
        self.check_duplicate(node, ack)?;
        self.raw(
            1,
            node,
            &WorkerFrame {
                frame: Some(worker_frame::Frame::Ack(ack.clone())),
            },
        )?;
        self.record("AckDuplicate", None, json!(ack))
    }
    fn check_duplicate(&self, node: u32, ack: &Ack) -> Result<()> {
        ensure(
            self.accepted_acks.get(&(node, ack.logical_step_id)) == Some(ack),
            "unsolicited or altered native duplicate Ack",
        )
    }
    async fn grant(&mut self, key: &Key, node: u32, input: p::Input) -> Result<()> {
        let idx = node as usize;
        self.next_step = self
            .next_step
            .checked_add(1)
            .ok_or_else(|| error("native step counter exhausted"))?;
        let mut grant = Grant {
            binding: Some(self.config.binding(node)),
            logical_step_id: self.next_step,
            event_id: key.event_id,
            tick_us: key.tick_us,
            causal_wave: key.causal_wave,
            tie_rank: key.tie_rank.to_vec(),
            input: Some(input.clone()),
            grant_hash: vec![],
        };
        grant.grant_hash = wire::grant_hash(&grant).to_vec();
        let frame = HubFrame {
            frame: Some(hub_frame::Frame::Grant(grant.clone())),
        };
        self.raw(0, node, &frame)?;
        self.record("Grant", Some(key), json!(&grant))?;
        self.workers[idx]
            .tx
            .as_ref()
            .ok_or_else(|| error("missing native worker stream"))?
            .send(Ok(frame))
            .await?;
        let deadline = tokio::time::Instant::now() + WATCHDOG;
        let ack = loop {
            match tokio::time::timeout_at(deadline, self.bus.recv()).await? {
                Some(Incoming::Ack(source, ack))
                    if source == node && ack.logical_step_id == grant.logical_step_id =>
                {
                    break ack;
                }
                Some(Incoming::Ack(source, ack)) => self.duplicate(source, &ack)?,
                Some(Incoming::Lost(source, reason)) => {
                    return Err(error(format!(
                        "native worker {source} lost at grant {}: {reason}",
                        grant.logical_step_id
                    )));
                }
                _ => return Err(error("unsolicited or wrong native Ack")),
            }
        };
        self.raw(
            1,
            node,
            &WorkerFrame {
                frame: Some(worker_frame::Frame::Ack(ack.clone())),
            },
        )?;
        self.commit_ack(key, node, &grant, &input, ack)
    }

    fn commit_ack(
        &mut self,
        key: &Key,
        node: u32,
        grant: &Grant,
        input: &p::Input,
        ack: Ack,
    ) -> Result<()> {
        let worker = &self.workers[node as usize];
        ensure(
            ack.binding == grant.binding
                && ack.logical_step_id == grant.logical_step_id
                && ack.event_id == key.event_id
                && ack.grant_hash == grant.grant_hash
                && ack.logical_ack_hash == wire::ack_hash(&ack)
                && worker.step_sequence.checked_add(1) == Some(ack.durable_step_sequence)
                && ack.durable_step_hash.len() == 32
                && ack
                    .witness_receipt
                    .as_ref()
                    .is_some_and(|r| r.sequence > 0 && r.hash.len() == 32)
                && matches!(
                    AckStatus::try_from(ack.status),
                    Ok(AckStatus::Quiescent | AckStatus::Rejected)
                ),
            "native Ack binding or status",
        )?;
        let _: Value = serde_json::from_slice(&ack.runtime_events_json)?;
        let _: Value = serde_json::from_slice(&ack.application_events_json)?;
        let mut planned = AckPlan {
            worker: worker.clone(),
            timers: self.timers.clone(),
            routes: self.routes.clone(),
            attempts: self.attempts.clone(),
            requested_close: self.requested_close.clone(),
            events: BTreeMap::new(),
            next_event: self.next_event,
            logs: vec![],
        };
        planned.apply(&self.config, key, node, grant, input, &ack)?;
        self.record("AckCommitted", Some(key), json!(&ack))?;
        self.workers[node as usize] = planned.worker;
        self.timers = planned.timers;
        self.routes = planned.routes;
        self.attempts = planned.attempts;
        self.requested_close = planned.requested_close;
        self.events.extend(planned.events);
        self.next_event = planned.next_event;
        self.accepted_acks.insert((node, ack.logical_step_id), ack);
        for (kind, details) in planned.logs {
            self.overflows += u64::from(kind == "QueueOverflow");
            self.record(kind, Some(key), details)?;
        }
        Ok(())
    }
}

type RouteQueues = BTreeMap<Route, VecDeque<(u64, Envelope)>>;
type TraceEvent = (&'static str, Value);
fn stage_packet(
    config: &Resolved,
    key: &Key,
    routes: &mut RouteQueues,
    attempts: &mut BTreeMap<Vec<u8>, u32>,
    envelope: Envelope,
) -> Result<(Option<(u64, Route)>, TraceEvent)> {
    native_network::validate_envelope(config, &envelope)?;
    let last = attempts.get(&envelope.message_id).copied().unwrap_or(0);
    ensure(
        last.checked_add(1) == Some(envelope.attempt),
        "non-sequential native packet attempt",
    )?;
    attempts.insert(envelope.message_id.clone(), envelope.attempt);
    let route = (
        envelope.src_member_id,
        envelope.dst_member_id,
        envelope.channel,
    );
    let queue = routes.entry(route).or_default();
    if queue.len() >= config.base.queue_capacity {
        return Ok((None, ("QueueOverflow", json!(envelope))));
    }
    let mut due = key
        .tick_us
        .checked_add(config.base.link_delay)
        .ok_or_else(|| error("native link due overflow"))?;
    if envelope.dst_member_id == 1 && envelope.channel == Channel::Body as i32 {
        if let Some(hold) = config.base.body_hold_until {
            due = due.max(hold);
        }
    }
    let empty = queue.is_empty();
    queue.push_back((due, envelope.clone()));
    Ok((
        empty.then_some((due, route)),
        (
            "NetworkAdmitted",
            json!({"envelope":envelope,"due_tick_us":due}),
        ),
    ))
}

fn make(kind: input::Kind) -> p::Input {
    p::Input { kind: Some(kind) }
}

struct Script {
    factory: PacketFactory,
    signer: NativeScheme,
    verifier: NativeScheme,
    blocks: BTreeMap<Id, Block>,
    proposals: BTreeMap<(u64, Id), Proposal<Digest>>,
    notarized: BTreeSet<(u64, Id)>,
    signed: BTreeSet<(bool, u64)>,
    next: u64,
    faulted: bool,
}
impl Script {
    fn new(config: Arc<Resolved>, member: u32, secret: Private) -> Result<Self> {
        let signer = config.signer(member, secret)?;
        let verifier = config.verifier()?;
        Ok(Self {
            factory: PacketFactory::new(config, member),
            signer,
            verifier,
            blocks: BTreeMap::new(),
            proposals: BTreeMap::new(),
            notarized: BTreeSet::new(),
            signed: BTreeSet::new(),
            next: 0,
            faulted: false,
        })
    }
    fn body_valid(&self, id: Id) -> bool {
        let Some(mut block) = self.blocks.get(&id) else {
            return false;
        };
        while block.height > 1 {
            let Some(parent) = self.blocks.get(&block.parent) else {
                return false;
            };
            if parent.height + 1 != block.height {
                return false;
            }
            block = parent;
        }
        block.parent == self.factory.config.genesis
    }
    fn receive(&mut self, packet: Envelope) -> Result<Vec<Envelope>> {
        let channel = Channel::try_from(packet.channel)?;
        match channel {
            Channel::Body => {
                let body = BodyPacket::decode(packet.payload_bytes.as_slice())?;
                let bytes = match body.kind {
                    Some(body_packet::Kind::Push(body)) => Some(body.block_bytes),
                    Some(body_packet::Kind::Reply(body)) => Some(body.block_bytes),
                    _ => None,
                };
                if let Some(bytes) = bytes {
                    if let Ok(block) = Block::read(&self.factory.config, &bytes) {
                        self.blocks.insert(block.digest, block);
                    }
                }
            }
            Channel::Vote => {
                if let Ok(RawVote::Notarize(vote)) =
                    native_network::decode_vote(&packet.payload_bytes)
                {
                    let view = vote.view().get();
                    let member = ((view.saturating_sub(1)) % 5) as u32;
                    if view > 0
                        && member < 2
                        && packet.src_member_id == member
                        && vote.signer().get()
                            == self.factory.config.members[member as usize].signer_index
                        && vote.epoch().get() == self.factory.config.base.epoch
                    {
                        let mut rng = ChaCha20Rng::from_seed([91; 32]);
                        if vote.verify(&mut rng, &self.verifier, &Sequential) {
                            self.proposals
                                .insert((view, vote.proposal.payload.0), vote.proposal);
                        }
                    }
                }
            }
            Channel::Certificate => {
                if let Ok(Certificate::Notarization(cert)) = native_network::decode_certificate(
                    &self.verifier,
                    self.factory.config.base.epoch,
                    &packet.payload_bytes,
                ) {
                    let view = cert.view().get();
                    self.notarized.insert((view, cert.proposal.payload.0));
                }
            }
            _ => {}
        }
        let mut out = vec![];
        for ((view, digest), proposal) in self.proposals.clone() {
            if !self.body_valid(digest) {
                continue;
            }
            let parent = self.blocks[&digest].parent;
            if proposal.parent.get() == 0 {
                if parent != self.factory.config.genesis {
                    continue;
                }
            } else if !self.notarized.contains(&(proposal.parent.get(), parent)) {
                continue;
            }
            if self.signed.insert((false, view)) {
                let vote = Notarize::sign(&self.signer, proposal.clone())
                    .ok_or_else(|| error("scripted notarizer refused"))?;
                self.emit(RawVote::Notarize(vote), &mut out)?;
            }
            if self.notarized.contains(&(view, digest)) && self.signed.insert((true, view)) {
                let vote = Finalize::sign(&self.signer, proposal)
                    .ok_or_else(|| error("scripted finalizer refused"))?;
                self.emit(Vote::Finalize(vote), &mut out)?;
            }
        }
        Ok(out)
    }
    fn fault(&mut self, packet: &Envelope) -> Result<Option<Envelope>> {
        let mode = self.factory.config.base.scenario.as_str();
        if self.faulted
            || self.factory.member != 2
            || packet.channel != Channel::Certificate as i32
            || !matches!(mode, "native-proof-negatives" | "wrong-context")
        {
            return Ok(None);
        }
        let Ok(Certificate::Notarization(value)) = native_network::decode_certificate(
            &self.verifier,
            self.factory.config.base.epoch,
            &packet.payload_bytes,
        ) else {
            return Ok(None);
        };
        self.faulted = true;
        let transcript = hash_parts(
            b"I1b/fresh-script/v1",
            &[
                &self.factory.config.run_id,
                &self.factory.member.to_be_bytes(),
            ],
        );
        let wrong_kind = Certificate::<NativeScheme, Digest>::Finalization(Finalization {
            proposal: value.proposal,
            certificate: value.certificate,
        })
        .encode()
        .to_vec();
        let provenance = Provenance::Scripted(ScriptedProvenance {
            transcript_id: transcript.to_vec(),
            entry_id: hash_parts(
                b"I1b/fresh-script-negative/v1",
                &[&transcript, mode.as_bytes(), &wrong_kind],
            )
            .to_vec(),
        });
        let mut envelope =
            self.factory
                .packet(1, Channel::Certificate, wrong_kind, true, Some(provenance))?;
        if mode == "wrong-context" {
            envelope.cid =
                hash_parts(b"I1b/wrong-context-fixture/v1", &[&self.factory.config.cid]).to_vec();
            envelope.message_id =
                wire::message_hash(&self.factory.config.config_hash, &envelope).to_vec();
        }
        Ok(Some(envelope))
    }
    fn emit(&mut self, vote: RawVote, out: &mut Vec<Envelope>) -> Result<()> {
        self.next += 1;
        let bytes = vote.encode().to_vec();
        let transcript_id = hash_parts(
            b"I1b/fresh-script/v1",
            &[
                &self.factory.config.run_id,
                &self.factory.member.to_be_bytes(),
            ],
        );
        let entry_id = hash_parts(
            b"I1b/fresh-script-entry/v1",
            &[&transcript_id, &self.next.to_be_bytes(), &bytes],
        );
        let provenance = Provenance::Scripted(ScriptedProvenance {
            transcript_id: transcript_id.to_vec(),
            entry_id: entry_id.to_vec(),
        });
        for dest in 0..2 {
            out.push(self.factory.packet(
                dest,
                Channel::Vote,
                bytes.clone(),
                true,
                Some(provenance.clone()),
            )?);
        }
        Ok(())
    }
}

struct Options {
    scenario: String,
    seed: u64,
    output: PathBuf,
    pause: u64,
    reverse: bool,
}
impl Options {
    fn parse(args: &[String]) -> Result<Self> {
        let mut args = args.iter();
        let (mut scenario, mut seed, mut output, mut pause, mut reverse) =
            (None, None, None, None, false);
        while let Some(flag) = args.next() {
            let missing = || error(format!("missing value for {flag}"));
            match flag.as_str() {
                "--scenario" if scenario.is_none() => {
                    scenario = Some(args.next().ok_or_else(missing)?.clone())
                }
                "--seed" if seed.is_none() => {
                    seed = Some(args.next().ok_or_else(missing)?.parse::<u64>()?)
                }
                "--output" if output.is_none() => {
                    output = Some(PathBuf::from(args.next().ok_or_else(missing)?))
                }
                "--wall-pause-ms" if pause.is_none() => {
                    pause = Some(args.next().ok_or_else(missing)?.parse::<u64>()?)
                }
                "--reverse-start" if !reverse => reverse = true,
                _ => return Err(error("unknown or repeated native CLI option")),
            }
        }
        let pause = pause.unwrap_or(0);
        ensure(pause <= 2, "native wall pause at most 2ms")?;
        Ok(Self {
            scenario: scenario.ok_or_else(|| error("missing scenario"))?,
            seed: seed.ok_or_else(|| error("missing seed"))?,
            output: output.ok_or_else(|| error("missing output"))?,
            pause,
            reverse,
        })
    }
}
pub async fn run_hub(args: &[String]) -> Result<()> {
    let options = Options::parse(args)?;
    let (resolved, mut keys) = Resolved::generate(&options.scenario, options.seed)?;
    let resolved = Arc::new(resolved);
    fs::create_dir(&options.output)?;
    let root = options.output.canonicalize()?;
    serde_json::to_writer_pretty(File::create(root.join("scenario.json"))?, &*resolved)?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let endpoint = format!("http://{}", listener.local_addr()?);
    let (broker, bus) = mpsc::channel(32);
    let service = Service {
        expected: resolved.clone(),
        registration: Arc::default(),
        broker,
    };
    let server = tokio::spawn(
        Server::builder()
            .add_service(
                NativeRunServer::new(service)
                    .max_decoding_message_size(wire::MAX_FRAME)
                    .max_encoding_message_size(wire::MAX_FRAME),
            )
            .serve_with_incoming(TcpListenerStream::new(listener)),
    );
    let scripted_secrets = keys.split_off(2);
    let scripts = if resolved.base.scripted {
        scripted_secrets
            .into_iter()
            .enumerate()
            .map(|(i, key)| Script::new(resolved.clone(), i as u32 + 2, key))
            .collect::<Result<_>>()?
    } else {
        vec![]
    };
    let mut children = Vec::new();
    let mut process = vec![];
    let start = Instant::now();
    let order = if options.reverse { [1, 0] } else { [0, 1] };
    for member in order {
        let dir = root.join(format!("worker-{member}"));
        fs::create_dir(&dir)?;
        let request = Launch {
            resolved: (*resolved).clone(),
            member,
            secret: keys[member as usize].encode().to_vec(),
            work_dir: dir,
            wall_pause_ms: options.pause,
        };
        let worker = spawn_worker(&request, &endpoint)?;
        process.push(json!({"worker":member,"pid":worker.0.id(),"start_wall_elapsed_us":start.elapsed().as_micros()}));
        children.push((member, worker));
    }
    drop(keys);
    let mut hub = Hub {
        config: resolved.clone(),
        scripts,
        events: BTreeMap::new(),
        next_event: 0,
        next_step: 0,
        workers: Default::default(),
        bus,
        routes: BTreeMap::new(),
        timers: BTreeMap::new(),
        requested_close: BTreeSet::new(),
        attempts: BTreeMap::new(),
        accepted_acks: BTreeMap::new(),
        overflows: 0,
        trace: BufWriter::new(File::create(root.join("virtual-trace.jsonl"))?),
        frames: BufWriter::new(File::create(root.join("frames.bin"))?),
    };
    let mut result = hub.run().await;
    if result.is_ok() {
        for (node, child) in &mut children {
            if let Some(sender) = hub.workers[*node as usize].tx.as_ref() {
                if sender
                    .send(Ok(HubFrame {
                        frame: Some(hub_frame::Frame::Shutdown(Shutdown {
                            reason: "complete".into(),
                        })),
                    }))
                    .await
                    .is_err()
                {
                    result = Err(error("native shutdown stream closed"));
                    break;
                }
            }
            let end = Instant::now() + WATCHDOG;
            loop {
                if let Some(status) = child.0.try_wait()? {
                    if !status.success() {
                        result = Err(error(format!("native child exit {status}")));
                    }
                    process
                        .push(json!({"worker":node,"pid":child.0.id(),"exit_code":status.code()}));
                    break;
                }
                if Instant::now() >= end {
                    result = Err(error("native child shutdown watchdog"));
                    break;
                }
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        }
    }
    hub.record(if result.is_ok(){"RunComplete"}else{"InfrastructureFailure"},None,json!({"delivered_heights":hub.workers.iter().map(|w|w.height).collect::<Vec<_>>(),"error":result.as_ref().err().map(ToString::to_string)}))?;
    let heights = hub.workers.iter().map(|w| w.height).collect::<Vec<_>>();
    serde_json::to_writer_pretty(
        File::create(root.join("manifest.json"))?,
        &json!({"gate":settings::PROFILE,"scenario":resolved.base.scenario,"seed":resolved.base.seed,
        "delivered_heights":heights,"outcome":if result.is_ok(){"COMPLETED"}else{"INFRASTRUCTURE_FAILURE"},"error":result.as_ref().err().map(ToString::to_string),"commands":args}),
    )?;
    serde_json::to_writer_pretty(
        File::create(root.join("transport-diagnostics.json"))?,
        &json!({"endpoint":endpoint,"processes":process,"wall_pause_ms":options.pause,"reverse_start":options.reverse,"elapsed_us":start.elapsed().as_micros()}),
    )?;
    server.abort();
    result
}

fn event_key(seed: u64, counter: &mut u64, parent: Option<&Key>, tick: u64) -> Result<Key> {
    let wave = match parent {
        Some(parent) if tick == parent.tick_us => parent
            .causal_wave
            .checked_add(1)
            .ok_or_else(|| error("causal wave overflow"))?,
        Some(parent) if tick < parent.tick_us => return Err(error("native derived event in past")),
        _ => 0,
    };
    *counter = counter
        .checked_add(1)
        .ok_or_else(|| error("event counter overflow"))?;
    let id = *counter;
    Ok(Key {
        tick_us: tick,
        causal_wave: wave,
        tie_rank: wire::tie_hash(seed, id),
        event_id: id,
    })
}

struct AckPlan {
    worker: Worker,
    timers: BTreeMap<(u32, u64), Timer>,
    routes: RouteQueues,
    attempts: BTreeMap<Vec<u8>, u32>,
    requested_close: BTreeSet<u32>,
    events: BTreeMap<Key, Event>,
    next_event: u64,
    logs: Vec<TraceEvent>,
}
impl AckPlan {
    fn schedule(&mut self, config: &Resolved, parent: &Key, tick: u64, event: Event) -> Result<()> {
        let key = event_key(config.base.seed, &mut self.next_event, Some(parent), tick)?;
        self.events.insert(key, event);
        Ok(())
    }

    fn apply(
        &mut self,
        config: &Resolved,
        key: &Key,
        node: u32,
        grant: &Grant,
        input: &p::Input,
        ack: &Ack,
    ) -> Result<()> {
        let receipt = ack
            .witness_receipt
            .as_ref()
            .ok_or_else(|| error("native Ack lacks witness head"))?;
        if let Some(prior) = &self.worker.witness {
            ensure(
                receipt.sequence >= prior.sequence
                    && (receipt.sequence > prior.sequence || receipt.hash == prior.hash),
                "native witness head regressed",
            )?;
        }
        if let Some(input::Kind::Sample(sample)) = &input.kind {
            if ack.status == AckStatus::Quiescent as i32 {
                ensure(
                    sample.lease_end_tick_us > key.tick_us,
                    "invalid accepted Sample lease",
                )?;
                self.worker.accepted_lease = Some(sample.lease_end_tick_us);
                self.schedule(
                    config,
                    key,
                    sample.lease_end_tick_us,
                    Event::Lease(node, sample.lease_end_tick_us),
                )?;
                if !self.worker.started && !self.worker.closed {
                    self.worker.started = true;
                    self.schedule(
                        config,
                        key,
                        key.tick_us,
                        Event::Grant(node, make(input::Kind::Start(config.start()))),
                    )?;
                }
            }
        }
        let original_height = self.worker.height;
        let mut works = BTreeSet::new();
        for output in &ack.effects {
            let kind = output
                .kind
                .as_ref()
                .ok_or_else(|| error("native Ack absent effect"))?;
            self.apply_effect(config, key, node, grant, receipt, kind, &mut works)?;
        }
        if self.worker.height > original_height {
            ensure(
                self.worker
                    .delivered_tip
                    .is_some_and(|digest| self.worker.proofs.contains(&digest)),
                "delivery lacks a verified finalization for the delivered tip",
            )?;
        }
        self.worker.witness = Some(receipt.clone());
        self.worker.step_sequence = ack.durable_step_sequence;
        Ok(())
    }
}

impl AckPlan {
    fn apply_effect(
        &mut self,
        config: &Resolved,
        key: &Key,
        node: u32,
        grant: &Grant,
        receipt: &Receipt,
        kind: &effect::Kind,
        works: &mut BTreeSet<Vec<u8>>,
    ) -> Result<()> {
        let idx = node as usize;
        match kind {
            effect::Kind::Send(envelope) => {
                ensure(
                    envelope.src_member_id == node
                        && !matches!(envelope.provenance, Some(Provenance::Scripted(_))),
                    "worker attempted foreign or scripted output",
                )?;
                if let Some(Provenance::LocalWitness(proof)) = &envelope.provenance {
                    let channel = (envelope.channel as u32).to_be_bytes();
                    let eligible = hash_parts(
                        b"I1b/outbox-payload/v1",
                        &[
                            &config.run_id,
                            &config.config_hash,
                            &config.cid,
                            &envelope.epoch.to_be_bytes(),
                            &node.to_be_bytes(),
                            &channel,
                            &envelope.payload_hash,
                        ],
                    );
                    let route = hash_parts(
                        b"I1b/outbox/v1",
                        &[&eligible, &envelope.dst_member_id.to_be_bytes()],
                    );
                    ensure(
                        envelope.priority
                            && proof.authorization_id == route
                            && proof.receipt.as_ref() == Some(receipt),
                        "bad native route authorization",
                    )?;
                }
                let (due, log) = stage_packet(
                    config,
                    key,
                    &mut self.routes,
                    &mut self.attempts,
                    envelope.clone(),
                )?;
                self.logs.push(log);
                if let Some((tick, route)) = due {
                    self.schedule(config, key, tick, Event::Route(route))?;
                }
            }
            effect::Kind::Sign(signed) => {
                let action = SignAction::try_from(signed.action)?;
                ensure(
                    signed.grant_id == grant.logical_step_id
                        && signed.tick_us == key.tick_us
                        && signed.task_id > 0
                        && signed.cid == config.cid
                        && signed.epoch == config.base.epoch
                        && signed.signer_index == config.members[idx].signer_index
                        && matches!(
                            SignKind::try_from(signed.kind),
                            Ok(SignKind::Notarize | SignKind::Nullify | SignKind::Finalize)
                        ),
                    "native signing outside grant",
                )?;
                if matches!(action, SignAction::Created | SignAction::CacheHit) {
                    ensure(
                        !self.worker.closed
                            && (1..=8).contains(&signed.view)
                            && signed.sample_sequence > 0
                            && self.worker.accepted_lease == Some(signed.lease_end_tick_us)
                            && key.tick_us < signed.lease_end_tick_us
                            && signed.local_time_us
                                == i64::try_from(key.tick_us)? + config.base.epsilon[idx]
                            && signed.local_stop_us == config.base.stop
                            && signed.local_time_us < signed.local_stop_us
                            && !signed.attestation_bytes.is_empty(),
                        "unsafe native signing audit",
                    )?;
                } else {
                    ensure(
                        action == SignAction::Denied && signed.attestation_bytes.is_empty(),
                        "invalid denied signing audit",
                    )?;
                }
            }
            effect::Kind::ClockRequest(request) => {
                ensure(
                    self.worker.last_request.checked_add(1) == Some(request.request_id),
                    "native clock request sequence",
                )?;
                self.worker.last_request = request.request_id;
                self.logs.push(("ClockRequestTransit",json!({"node":node,"request_id":request.request_id,"delay_us":config.base.clock_transit})));
                self.schedule(
                    config,
                    key,
                    key.tick_us + config.base.clock_transit,
                    Event::ClockService(node, request.request_id),
                )?;
                if key.tick_us + config.base.refresh <= config.base.cutoff {
                    self.schedule(
                        config,
                        key,
                        key.tick_us + config.base.refresh,
                        Event::Grant(node, make(input::Kind::RequestSample(Empty {}))),
                    )?;
                }
            }
            effect::Kind::TimerArm(timer) => {
                ensure(
                    timer.cid == config.cid
                        && timer.id > 0
                        && self
                            .timers
                            .insert((node, timer.id), timer.clone())
                            .is_none(),
                    "invalid native TimerArm",
                )?;
                let due = timer
                    .local_deadline_us
                    .checked_sub(config.base.epsilon[idx])
                    .ok_or_else(|| error("clock deadline overflow"))?;
                let tick = (due.max(0) as u64).max(key.tick_us);
                self.schedule(config, key, tick, Event::Timer(node, timer.clone()))?;
            }
            effect::Kind::TimerCancel(timer) => {
                ensure(
                    timer.cid == config.cid && timer.id > 0,
                    "invalid native TimerCancel",
                )?;
                if self.timers.get(&(node, timer.id)) == Some(timer) {
                    self.timers.remove(&(node, timer.id));
                }
            }
            effect::Kind::Work(work) => {
                ensure(
                    work.operation_id.len() == 32
                        && works.insert(work.operation_id.clone())
                        && work.cid == config.cid
                        && work.duration_us == config.base.work_us
                        && matches!(
                            WorkKind::try_from(work.kind),
                            Ok(WorkKind::Propose | WorkKind::Verify | WorkKind::Certify)
                        ),
                    "bad native work continuation",
                )?;
                if config.base.close_on_work == Some(work.kind) && self.requested_close.is_empty() {
                    self.requested_close.insert(node);
                    self.schedule(
                        config,
                        key,
                        key.tick_us + 2_500,
                        Event::Grant(
                            node,
                            make(input::Kind::Close(Cid {
                                cid: config.cid.to_vec(),
                            })),
                        ),
                    )?;
                }
                self.schedule(
                    config,
                    key,
                    key.tick_us + work.duration_us,
                    Event::Grant(node, make(input::Kind::WorkCompleted(work.clone()))),
                )?;
            }
            effect::Kind::Proof(proof) => {
                use commonware_codec::{Copying, Decode as _};
                use commonware_cryptography::certificate::Verifier as _;
                let verifier = config.verifier()?;
                let native = Finalization::<NativeScheme, Digest>::decode_cfg(
                    Copying(&proof.finalization_bytes),
                    &verifier.certificate_codec_config(),
                )
                .map_err(|_| error("malformed native proof effect"))?;
                ensure(
                    proof.cid == config.cid
                        && proof.epoch == config.base.epoch
                        && native.epoch().get() == proof.epoch
                        && native.view().get() == proof.view
                        && proof.payload_digest == native.proposal.payload.0
                        && native.verify(
                            &mut ChaCha20Rng::from_seed([95; 32]),
                            &verifier,
                            &Sequential,
                        ),
                    "invalid native proof effect",
                )?;
                self.worker.proofs.insert(native.proposal.payload.0);
                self.worker.first_proof.get_or_insert(key.tick_us);
            }
            effect::Kind::Delivered(delivered) => {
                let block = Block::read(config, &delivered.block_bytes)?;
                ensure(
                    delivered.cid == config.cid
                        && self.worker.height.checked_add(1) == Some(delivered.height)
                        && delivered.height == block.height
                        && delivered.digest == block.digest
                        && delivered.parent_digest == block.parent
                        && block.parent == self.worker.delivered_tip.unwrap_or(config.genesis),
                    "invalid delivered prefix effect",
                )?;
                self.worker.height = delivered.height;
                self.worker.delivered_tip = Some(block.digest);
                self.worker.first_delivery.get_or_insert(key.tick_us);
            }
            effect::Kind::Lifecycle(lifecycle) => {
                ensure(
                    lifecycle.cid == config.cid
                        && matches!(lifecycle.event.as_str(), "started" | "terminated"),
                    "invalid lifecycle effect",
                )?;
                if lifecycle.event == "terminated" {
                    self.worker.closed = true;
                    self.worker.close_reason = Some(lifecycle.reason.clone());
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forged_ack_has_atomic_effects_and_committed_duplicate_identity() {
        let (config, _) = Resolved::generate("healthy", 7).unwrap();
        let config = Arc::new(config);
        let root = (0..u64::MAX)
            .map(|i| {
                std::env::temp_dir().join(format!("native-hub-ack-{}-{i}", std::process::id()))
            })
            .find(|path| fs::create_dir(path).is_ok())
            .unwrap();
        let (_tx, bus) = mpsc::channel(1);
        let mut hub = Hub {
            config: config.clone(),
            scripts: vec![],
            events: BTreeMap::new(),
            next_event: 0,
            next_step: 1,
            workers: Default::default(),
            bus,
            routes: BTreeMap::new(),
            timers: BTreeMap::new(),
            requested_close: BTreeSet::new(),
            attempts: BTreeMap::new(),
            accepted_acks: BTreeMap::new(),
            overflows: 0,
            trace: BufWriter::new(File::create(root.join("trace")).unwrap()),
            frames: BufWriter::new(File::create(root.join("frames")).unwrap()),
        };
        let key = event_key(7, &mut hub.next_event, None, settings::START).unwrap();
        let input = make(input::Kind::DriveReady(Empty {}));
        let mut grant = Grant {
            binding: Some(config.binding(0)),
            logical_step_id: 1,
            event_id: key.event_id,
            tick_us: key.tick_us,
            causal_wave: 0,
            tie_rank: key.tie_rank.to_vec(),
            input: Some(input.clone()),
            grant_hash: vec![],
        };
        grant.grant_hash = wire::grant_hash(&grant).to_vec();
        let packet = PacketFactory::new(config.clone(), 0)
            .packet(
                1,
                Channel::Body,
                vec![1, 2, 3],
                false,
                Some(Provenance::Body(BodyProvenance {
                    content_id: vec![42; 32],
                    request_id: 1,
                })),
            )
            .unwrap();
        assert_eq!(packet.attempt, 1);
        let mut invalid_attempt = packet.clone();
        invalid_attempt.attempt = 0;
        assert!(native_network::validate_envelope(&config, &invalid_attempt).is_err());
        let mut ack = Ack {
            binding: grant.binding.clone(),
            logical_step_id: 1,
            event_id: grant.event_id,
            grant_hash: grant.grant_hash.clone(),
            status: AckStatus::Quiescent as i32,
            runtime_events_json: b"[]".to_vec(),
            application_events_json: b"[]".to_vec(),
            witness_receipt: Some(Receipt {
                sequence: 1,
                hash: vec![41; 32],
            }),
            durable_step_sequence: 1,
            durable_step_hash: vec![10; 32],
            effects: vec![
                Effect {
                    kind: Some(effect::Kind::Send(packet)),
                },
                Effect {
                    kind: Some(effect::Kind::TimerArm(Timer {
                        id: 0,
                        cid: vec![],
                        local_deadline_us: 0,
                    })),
                },
            ],
            ..Default::default()
        };
        ack.logical_ack_hash = wire::ack_hash(&ack).to_vec();
        assert!(
            hub.commit_ack(&key, 0, &grant, &input, ack.clone())
                .is_err()
        );
        assert!(
            hub.routes.is_empty()
                && hub.attempts.is_empty()
                && hub.events.is_empty()
                && hub.accepted_acks.is_empty()
        );
        assert_eq!(hub.next_event, 1);
        assert_eq!(hub.workers[0].step_sequence, 0);
        assert_eq!(fs::metadata(root.join("trace")).unwrap().len(), 0);

        ack.effects.pop();
        ack.logical_ack_hash = wire::ack_hash(&ack).to_vec();
        hub.commit_ack(&key, 0, &grant, &input, ack.clone())
            .unwrap();
        assert_eq!(hub.workers[0].step_sequence, 1);
        assert_eq!(hub.routes.values().next().unwrap().len(), 1);
        assert!(hub.duplicate(0, &ack).is_ok());
        assert_eq!(hub.routes.values().next().unwrap().len(), 1);
        ack.application_events_json = b"[1]".to_vec();
        ack.logical_ack_hash = wire::ack_hash(&ack).to_vec();
        assert!(hub.duplicate(0, &ack).is_err());
        assert_eq!(hub.events.len(), 1);
        drop(hub);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn native_quorum_is_four_distinct_matching_signers() {
        use commonware_consensus::{
            simplex::types::Notarization,
            types::{Epoch, Round, View},
        };
        use commonware_utils::iter::NonEmpty;
        let (config, keys) = Resolved::generate("native-proof-negatives", 7).unwrap();
        let verifier = config.verifier().unwrap();
        let proposal = Proposal::new(
            Round::new(Epoch::new(1), View::new(1)),
            View::new(0),
            Digest([0x48; 32]),
        );
        let votes: Vec<_> = keys
            .into_iter()
            .enumerate()
            .map(|(i, key)| {
                Notarize::sign(&config.signer(i as u32, key).unwrap(), proposal.clone()).unwrap()
            })
            .collect();
        assert!(
            Notarization::from_notarizes(
                &verifier,
                NonEmpty::new(&votes[0], votes[1..3].iter()),
                &Sequential
            )
            .is_err()
        );
        let duplicate = Notarization::from_notarizes(
            &verifier,
            NonEmpty::new(&votes[0], [&votes[0], &votes[1], &votes[2]].into_iter()),
            &Sequential,
        );
        if let Ok(duplicate) = duplicate {
            assert!(!duplicate.verify(
                &mut ChaCha20Rng::from_seed([31; 32]),
                &verifier,
                &Sequential
            ));
        }
        let certificate = Notarization::from_notarizes(
            &verifier,
            NonEmpty::new(&votes[0], votes[1..4].iter()),
            &Sequential,
        )
        .unwrap();
        let good = Certificate::Notarization(certificate.clone()).encode();
        assert!(native_network::decode_certificate(&verifier, 1, &good).is_ok());
        assert!(native_network::decode_certificate(&verifier, 2, &good).is_err());
        let wrong = Certificate::<NativeScheme, Digest>::Finalization(Finalization {
            proposal: certificate.proposal,
            certificate: certificate.certificate,
        })
        .encode();
        assert!(native_network::decode_certificate(&verifier, 1, &wrong).is_err());
    }
}
