use crate::{
    scenario::{ActionKind, Link, SampleReply, Scenario, cid},
    transport::{Incoming, Transport, deadline},
};
use poc_types::{
    effect::Kind as E, hub_frame::Frame as H, input::Kind as I, worker_frame::Frame as W, *,
};
use prost::Message;
use serde::Serialize;
use serde_json::{Value, json};
use sim_runtime::{
    Checkpoint, Identity, Launch, Result, Retry, State, envelope_error, intent_hash, invalid,
    message_id, payload_hash, rate_profile, require,
};
use std::{
    collections::{BTreeMap, VecDeque},
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct EventKey {
    tick_us: u64,
    causal_wave: u64,
    tie_rank: Vec<u8>,
    event_id: u64,
}

#[derive(Clone)]
enum Event {
    Action(ActionKind),
    Grant(Input),
    ClockTransit(Request),
    ClockService(Request),
    ClockDeliver(ClockSample),
    LeaseCheck { incarnation: u64, end: u64 },
    Timer(Timer),
    PacketDue((u32, u32, u32)),
    PacketRetry(Envelope),
}

#[derive(Clone)]
struct Request {
    request_id: u64,
    incarnation: u64,
    request_us: u64,
    response_us: u64,
    response: SampleReply,
}

#[derive(Default)]
struct Node {
    incarnation: u64,
    phase: Phase,
    seq: u64,
    state_hash: Vec<u8>,
    sample_seq: u64,
    lease_end: Option<u64>,
    cached: BTreeMap<u64, (Grant, Ack)>,
    pending: Vec<Input>,
    faults: BTreeMap<u64, Checkpoint>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    Offline,
    Recovering,
    Calibrating,
    Running,
}

impl Phase {
    fn allows(self, input: &Input) -> bool {
        match self {
            Self::Recovering => matches!(input.kind, Some(I::Recover(_))),
            Self::Calibrating => matches!(
                input.kind,
                Some(I::Sample(_) | I::Deliver(_) | I::RequestSample(_))
            ),
            Self::Running => true,
            Self::Offline => false,
        }
    }
}

impl Node {
    fn online(&self) -> bool {
        self.phase != Phase::Offline
    }
}

#[derive(Clone)]
struct TimerWait {
    timer: Timer,
    scheduled: Option<u64>,
}

#[derive(Clone, Default)]
struct Queue {
    packets: VecDeque<(u64, Envelope)>,
}

struct StagedEffects {
    node: u32,
    seed: u64,
    phase: Phase,
    lease_end: Option<u64>,
    pending: Vec<Input>,
    timers: BTreeMap<(u32, u64), TimerWait>,
    queues: BTreeMap<(u32, u32, u32), Queue>,
    next_event_id: u64,
    events: BTreeMap<EventKey, (u32, Event)>,
    trace: Vec<(&'static str, Value)>,
}

impl StagedEffects {
    fn enqueue(&mut self, parent: &EventKey, tick: u64, node: u32, event: Event) -> Result<u64> {
        let key = event_key(self.seed, &mut self.next_event_id, Some(parent), tick)?;
        let id = key.event_id;
        self.events.insert(key, (node, event));
        Ok(id)
    }

    fn remove_wait(&mut self, timer: &Timer) -> Result<()> {
        let found = self.timers.remove(&(self.node, timer.timer_id));
        require(
            found.is_some_and(|wait| wait.timer == *timer),
            "timer effect without matching active wait",
        )
    }

    fn schedule_timer(&mut self, parent: &EventKey, id: u64, epsilon: i64) -> Result<()> {
        let timer = self.timers[&(self.node, id)].timer.clone();
        let deadline = i128::from(timer.local_deadline_us) - i128::from(epsilon);
        let tick = u64::try_from(deadline.max(i128::from(parent.tick_us)))?;
        let event = self.enqueue(parent, tick, self.node, Event::Timer(timer))?;
        self.timers
            .get_mut(&(self.node, id))
            .expect("timer exists")
            .scheduled = Some(event);
        Ok(())
    }
}

pub struct Hub {
    scenario: Scenario,
    pub run_id: Vec<u8>,
    pub config_hash: Vec<u8>,
    identities: [Identity; 2],
    nodes: [Node; 2],
    events: BTreeMap<EventKey, (u32, Event)>,
    timers: BTreeMap<(u32, u64), TimerWait>,
    queues: BTreeMap<(u32, u32, u32), Queue>,
    next_event_id: u64,
    next_step_id: u64,
    trace: BufWriter<File>,
    frames: BufWriter<File>,
    output: PathBuf,
    transport: Transport,
    wall_pause_ms: u64,
}

impl Hub {
    pub async fn new(scenario: Scenario, output: &Path, wall_pause_ms: u64) -> Result<Self> {
        let config_hash = hash_parts("I1/config/v1", &[&serde_json::to_vec(&scenario)?]);
        let run_id = hash_parts("I1/run/v1", &[&config_hash]);
        let identities = std::array::from_fn(|node| Identity {
            run_id: run_id.clone(),
            config_hash: config_hash.clone(),
            node_id: node as u32,
            epsilon_us: scenario.epsilon_us[node],
        });
        let transport = Transport::new(output, identities.clone()).await?;
        let mut hub = Self {
            scenario,
            run_id,
            config_hash,
            identities,
            nodes: Default::default(),
            events: BTreeMap::new(),
            timers: BTreeMap::new(),
            queues: BTreeMap::new(),
            next_event_id: 0,
            next_step_id: 0,
            trace: BufWriter::new(File::create(output.join("virtual-trace.jsonl"))?),
            frames: BufWriter::new(File::create(output.join("frames.bin"))?),
            output: output.to_owned(),
            transport,
            wall_pause_ms,
        };
        for action in hub.scenario.actions.clone() {
            if let ActionKind::FaultWork {
                ref work,
                checkpoint,
            } = action.kind
            {
                hub.nodes[action.node as usize]
                    .faults
                    .insert(work.slot, checkpoint);
            }
            hub.enqueue(
                None,
                action.tick_us,
                action.node,
                Event::Action(action.kind),
            )?;
        }
        hub.trace_endpoint(hub.scenario.start_tick_us)?;
        Ok(hub)
    }

    pub async fn run(&mut self, reverse_start: bool) -> Result<()> {
        let order = if reverse_start { [1, 0] } else { [0, 1] };
        for node in order {
            self.spawn(node, None)?;
        }
        let mut ready = [None, None];
        let limit = deadline();
        while ready.iter().any(Option::is_none) {
            match self.transport.receive(limit).await? {
                Incoming::Ready(reg, sender) => {
                    let node = reg.node_id;
                    self.transport.attach(node, sender)?;
                    self.validate_registration(&reg, 0, false)?;
                    ready[node as usize] = Some(reg);
                }
                other => self.unexpected(other)?,
            }
        }
        require(
            self.transport.is_live(0)? && self.transport.is_live(1)?,
            "both processes must be live before scheduling",
        )?;
        for reg in ready.into_iter().flatten() {
            self.trace("Registered", None, json!(reg))?;
        }

        let mut last_tick = self.scenario.start_tick_us;
        while let Some((key, (node, event))) = self.events.pop_first() {
            self.poll_frames()?;
            last_tick = key.tick_us;
            self.dispatch(key, node, event).await?;
        }
        self.poll_frames()?;
        self.trace_endpoint(last_tick)?;
        self.trace("RunComplete", None, json!({}))?;
        Ok(())
    }

    fn trace_endpoint(&mut self, tick: u64) -> Result<()> {
        for node in 0..2 {
            let value = self.identities[node].local_time(tick)?;
            self.trace(
                "ClockProfileEndpoint",
                None,
                json!({"node_id": node, "tick_us": tick, "node_time_us": value}),
            )?;
        }
        Ok(())
    }

    pub async fn finish(
        &mut self,
        failure: Option<&(dyn std::error::Error + Send + Sync)>,
    ) -> Result<()> {
        let mut first_error = None;
        if let Some(error) = failure {
            self.transport
                .diagnostic("InfrastructureFailure", json!({"error": error.to_string()}))?;
            self.trace(
                "InfrastructureFailure",
                None,
                json!({"reason": error.to_string()}),
            )?;
        }
        for node in 0..2 {
            if self.transport.process_exists(node as usize) {
                let result = if failure.is_some() {
                    let kill = self.transport.is_live(node as usize)?;
                    self.transport.reap(node, None, kill).await
                } else {
                    self.transport.stop_and_reap(node, "complete", 0).await
                };
                if let Err(error) = result {
                    first_error.get_or_insert(error);
                }
                self.nodes[node as usize].phase = Phase::Offline;
            }
        }
        if failure.is_none() {
            self.poll_frames()?;
        }
        self.frames.flush()?;
        self.trace.flush()?;
        first_error.map_or(Ok(()), Err)
    }

    fn enqueue(
        &mut self,
        parent: Option<&EventKey>,
        tick: u64,
        node: u32,
        event: Event,
    ) -> Result<u64> {
        let key = event_key(self.scenario.seed, &mut self.next_event_id, parent, tick)?;
        let id = key.event_id;
        self.events.insert(key, (node, event));
        Ok(id)
    }

    fn trace(&mut self, kind: &str, key: Option<&EventKey>, detail: Value) -> Result<()> {
        serde_json::to_writer(
            &mut self.trace,
            &json!({"kind": kind, "event": key, "detail": detail}),
        )?;
        writeln!(self.trace)?;
        self.trace.flush()?;
        Ok(())
    }

    fn frame<M: Message>(&mut self, from_worker: bool, node: u32, frame: &M) -> Result<()> {
        let body = frame.encode_to_vec();
        require(
            body.len() <= MAX_FRAME_BYTES,
            "protobuf frame exceeds bound",
        )?;
        self.frames
            .write_all(&[u8::from(from_worker), u8::try_from(node)?])?;
        self.frames
            .write_all(&u64::try_from(body.len())?.to_be_bytes())?;
        self.frames.write_all(&body)?;
        self.frames.flush()?;
        Ok(())
    }

    fn spawn(&mut self, node: u32, retry: Option<Retry>) -> Result<()> {
        let i = node as usize;
        let state = &mut self.nodes[i];
        state.incarnation = add(state.incarnation, 1)?;
        let launch = Launch {
            identity: self.identities[i].clone(),
            incarnation: state.incarnation,
            journal: self.output.join(format!("worker-{i}/journal")),
            wall_pause_ms: self.wall_pause_ms,
            faults: state
                .faults
                .iter()
                .map(|(slot, fault)| (*slot, *fault))
                .collect(),
            retry,
        };
        self.transport.spawn(&launch)?;
        state.phase = if state.incarnation == 1 {
            Phase::Running
        } else {
            Phase::Recovering
        };
        Ok(())
    }

    fn validate_registration(
        &mut self,
        reg: &RegisterRequest,
        expected_seq: u64,
        new_record: bool,
    ) -> Result<()> {
        let node = &self.nodes[reg.node_id as usize];
        require(
            reg.incarnation == node.incarnation && reg.recovered_journal_seq == expected_seq,
            "unexpected recovered journal sequence",
        )?;
        if !new_record {
            let expected_hash = if node.seq == 0 {
                State::new(&self.identities[reg.node_id as usize]).hash()?
            } else {
                node.state_hash.clone()
            };
            require(
                reg.recovered_state_hash == expected_hash,
                "unexpected recovered state hash",
            )?;
        }
        self.frame(
            true,
            reg.node_id,
            &WorkerFrame {
                frame: Some(W::Hello(reg.clone())),
            },
        )
    }

    async fn recover_connection(
        &mut self,
        node: u32,
        retry: Option<Retry>,
        key: &EventKey,
    ) -> Result<RegisterRequest> {
        let committed_seq = self.nodes[node as usize].seq;
        let expected_seq = add(committed_seq, u64::from(retry.is_some()))?;
        self.spawn(node, retry.clone())?;
        let limit = deadline();
        loop {
            match self.transport.receive(limit).await? {
                Incoming::Ready(reg, sender) if reg.node_id == node => {
                    self.validate_registration(&reg, expected_seq, retry.is_some())?;
                    self.transport.attach(node, sender)?;
                    self.trace("Registered", Some(key), json!(reg))?;
                    return Ok(reg);
                }
                other => self.unexpected(other)?,
            }
        }
    }

    fn poll_frames(&mut self) -> Result<()> {
        while let Some(frame) = self.transport.try_receive()? {
            self.unexpected(frame)?;
        }
        Ok(())
    }

    fn unexpected(&mut self, incoming: Incoming) -> Result<()> {
        match incoming {
            Incoming::Ack(node, incarnation, ack) => {
                self.frame(
                    true,
                    node,
                    &WorkerFrame {
                        frame: Some(W::Ack(ack.clone())),
                    },
                )?;
                let state = &self.nodes[node as usize];
                require(
                    incarnation == state.incarnation && state.online(),
                    "obsolete Ack transport",
                )?;
                self.identities[node as usize].validate_ack(incarnation, &ack)?;
                let cached = state
                    .cached
                    .get(&ack.logical_step_id)
                    .ok_or_else(|| invalid("unsolicited Ack"))?;
                require(cached.1 == ack, "altered duplicate Ack")?;
                self.trace("AckDuplicate", None, json!(ack))?;
                Ok(())
            }
            Incoming::Disconnected(node, incarnation, reason) => {
                let state = &self.nodes[node as usize];
                if incarnation < state.incarnation || !state.online() {
                    return Ok(());
                }
                Err(invalid(format!(
                    "unscripted worker {node}/{incarnation} disconnect: {reason}"
                )))
            }
            Incoming::Ready(_, _) => Err(invalid("unsolicited registered stream")),
        }
    }

    async fn dispatch(&mut self, key: EventKey, node: u32, event: Event) -> Result<()> {
        let tick = key.tick_us;
        let i = node as usize;
        match event {
            Event::Action(action) => match action {
                ActionKind::Input(input) => self.execute(&key, node, input).await?,
                ActionKind::Packet {
                    channel,
                    length,
                    value,
                } => {
                    let payload_bytes = vec![value; length];
                    let mut envelope = Envelope {
                        run_id: self.run_id.clone(),
                        src: node,
                        dst: 1 - node,
                        channel,
                        message_id: vec![],
                        cid: cid("first"),
                        sent_at_tick_us: tick,
                        payload_hash: payload_hash(&payload_bytes),
                        payload_bytes,
                        attempt: 0,
                    };
                    envelope.message_id = message_id(&envelope);
                    self.execute(&key, node, input(I::Send(envelope))).await?;
                }
                ActionKind::FaultWork { work, .. } => {
                    self.execute(&key, node, input(I::BeginWork(work))).await?
                }
                ActionKind::Disconnect => {
                    require(self.nodes[i].online(), "worker already offline")?;
                    self.transport
                        .stop_and_reap(node, "scripted-offline", 0)
                        .await?;
                    self.mark_offline(node);
                    self.trace("ScriptedOffline", Some(&key), json!({"node_id": node}))?;
                }
                ActionKind::Restart => {
                    require(!self.nodes[i].online(), "worker already online")?;
                    self.recover_connection(node, None, &key).await?;
                    self.execute(&key, node, input(I::Recover(Empty {})))
                        .await?;
                }
                ActionKind::ReplayIntent { slot } => {
                    let (grant, _) = self.nodes[i].cached.values().find(|(_, ack)| {
                        ack.effects.iter().any(|e| matches!(&e.kind, Some(E::SignedIntent(signed)) if signed.slot == slot))
                    }).cloned().ok_or_else(|| invalid("missing intent for duplicate grant"))?;
                    self.duplicate_grant(&key, node, grant).await?;
                }
            },
            Event::Grant(input) => self.execute(&key, node, input).await?,
            Event::ClockTransit(request) => {
                self.trace("ClockRequestTransit", Some(&key), json!({
                    "node_id": node, "incarnation": request.incarnation, "request_id": request.request_id,
                    "one_way_delay_us": request.request_us,
                }))?;
                self.enqueue(
                    Some(&key),
                    add(tick, request.request_us)?,
                    node,
                    Event::ClockService(request),
                )?;
            }
            Event::ClockService(request) => self.sample(&key, node, request)?,
            Event::ClockDeliver(sample) => {
                self.trace(
                    "ClockResponseDelivery",
                    Some(&key),
                    json!({"sample": sample, "online": self.nodes[i].online()}),
                )?;
                if self.nodes[i].online() {
                    self.execute(&key, node, input(I::Sample(sample))).await?;
                }
            }
            Event::LeaseCheck { incarnation, end } => {
                if self.nodes[i].online()
                    && self.nodes[i].incarnation == incarnation
                    && self.nodes[i].lease_end == Some(end)
                {
                    self.trace(
                        "LeaseBoundary",
                        Some(&key),
                        json!({"node_id": node, "lease_end_tick_us": end}),
                    )?;
                    self.execute(&key, node, input(I::CheckClock(Empty {})))
                        .await?;
                } else {
                    self.trace("LeaseCheckSuppressed", Some(&key), json!({"node_id": node, "incarnation": incarnation, "lease_end_tick_us": end}))?;
                }
            }
            Event::Timer(timer) => {
                let slot = (node, timer.timer_id);
                let armed = self.timers.get(&slot).is_some_and(|wait| {
                    wait.timer == timer && wait.scheduled == Some(key.event_id)
                });
                if !armed
                    || !self.nodes[i].online()
                    || !self.nodes[i].lease_end.is_some_and(|end| tick < end)
                {
                    if armed {
                        self.timers.get_mut(&slot).expect("active timer").scheduled = None;
                    }
                    self.trace(
                        "TimerSuppressed",
                        Some(&key),
                        json!({"node_id": node, "timer": timer, "active": armed}),
                    )?;
                } else {
                    self.trace(
                        "TimerDelivery",
                        Some(&key),
                        json!({"node_id": node, "timer": timer}),
                    )?;
                    self.execute(&key, node, input(I::TimerFired(timer)))
                        .await?;
                }
            }
            Event::PacketDue(link) => {
                let queue = self
                    .queues
                    .get_mut(&link)
                    .ok_or_else(|| invalid("unknown packet queue"))?;
                let (_, envelope) = queue
                    .packets
                    .pop_front()
                    .ok_or_else(|| invalid("empty packet queue"))?;
                if let Some((due, next)) = queue.packets.front() {
                    let (due, dst) = (*due, next.dst);
                    self.enqueue(Some(&key), tick.max(due), dst, Event::PacketDue(link))?;
                }
                self.deliver_packet(&key, envelope).await?;
            }
            Event::PacketRetry(envelope) => self.admit_packet(&key, envelope)?,
        }
        Ok(())
    }

    async fn execute(&mut self, key: &EventKey, node: u32, input: Input) -> Result<()> {
        let i = node as usize;
        if !self.nodes[i].online() {
            self.trace(
                "OfflineInput",
                Some(key),
                json!({"node_id": node, "input": input}),
            )?;
            if matches!(input.kind, Some(I::WorkCompleted(_))) {
                self.nodes[i].pending.push(input);
            }
            return Ok(());
        }
        let phase = self.nodes[i].phase;
        let allowed = phase.allows(&input);
        if !allowed {
            if matches!(input.kind, Some(I::Sample(_))) {
                self.trace(
                    "StaleTransportSample",
                    Some(key),
                    json!({"node_id": node, "input": input}),
                )?;
            } else {
                self.trace(
                    "RecoveryInputDeferred",
                    Some(key),
                    json!({"node_id": node, "input": input}),
                )?;
                self.nodes[i].pending.push(input);
            }
            return Ok(());
        }
        let checkpoint = match input.kind.as_ref() {
            Some(I::WorkCompleted(work)) => self.nodes[i].faults.get(&work.slot).copied(),
            _ => None,
        };
        self.next_step_id = add(self.next_step_id, 1)?;
        let mut grant = Grant {
            schema_version: SCHEMA_VERSION,
            run_id: self.run_id.clone(),
            node_id: node,
            incarnation: self.nodes[i].incarnation,
            logical_step_id: self.next_step_id,
            event_id: key.event_id,
            tick_us: key.tick_us,
            causal_wave: key.causal_wave,
            tie_rank: key.tie_rank.clone(),
            input: Some(input.clone()),
            grant_hash: vec![],
        };
        grant.grant_hash = grant_hash(&grant);
        self.send_grant(key, &grant, "Grant").await?;
        let in_grant_fault = checkpoint.filter(|fault| *fault != Checkpoint::AfterHubCommit);
        let registration = if let Some(fault) = in_grant_fault {
            self.expect_checkpoint_disconnect(&grant, fault).await?;
            self.trace(
                "ScriptedCheckpoint",
                Some(key),
                json!({"checkpoint": fault, "grant": grant}),
            )?;
            self.remove_fault(i, &input);
            let lease = self.nodes[i].lease_end;
            self.mark_offline(node);
            if matches!(
                fault,
                Checkpoint::AfterSyncBeforeAck | Checkpoint::AfterRecordSyncBeforeHead
            ) {
                let retry = Retry {
                    logical_step_id: grant.logical_step_id,
                    grant_hash: grant.grant_hash.clone(),
                    original_incarnation: grant.incarnation,
                };
                let reg = self.recover_connection(node, Some(retry), key).await?;
                grant.incarnation = reg.incarnation;
                self.nodes[i].lease_end = lease;
                self.send_grant(key, &grant, "RecoverPending").await?;
                Some(reg)
            } else {
                self.trace(
                    "StepAborted",
                    Some(key),
                    json!({"node_id": node, "step": grant.logical_step_id}),
                )?;
                self.nodes[i].pending.push(input);
                self.recover_connection(node, None, key).await?;
                self.enqueue(Some(key), key.tick_us, node, Event::Grant(input_recover()))?;
                return Ok(());
            }
        } else {
            None
        };
        let ack = self.await_ack(&grant).await?;
        if let Some(reg) = &registration {
            require(
                reg.recovered_state_hash == ack.state_hash
                    && reg.recovered_journal_seq == ack.durable_journal_seq,
                "replayed Ack disagrees with recovered journal",
            )?;
        }
        self.commit(key, &grant, ack, registration.is_some())?;
        if matches!(input.kind, Some(I::Recover(_))) {
            self.nodes[i].phase = Phase::Calibrating;
        }
        if registration.is_some() {
            self.nodes[i].lease_end = None;
        }
        if checkpoint == Some(Checkpoint::AfterHubCommit) {
            self.duplicate_grant(key, node, grant.clone()).await?;
            self.transport
                .stop_and_reap(
                    node,
                    "scripted-after-hub-commit",
                    Checkpoint::AfterHubCommit.exit_code(),
                )
                .await?;
            self.trace(
                "ScriptedCheckpoint",
                Some(key),
                json!({"checkpoint": checkpoint, "grant": grant}),
            )?;
            self.remove_fault(i, &input);
            self.mark_offline(node);
            self.recover_connection(node, None, key).await?;
        }
        if checkpoint.is_some() {
            self.enqueue(Some(key), key.tick_us, node, Event::Grant(input_recover()))?;
        }
        Ok(())
    }

    fn remove_fault(&mut self, node: usize, input: &Input) {
        if let Some(I::WorkCompleted(work)) = input.kind.as_ref() {
            self.nodes[node].faults.remove(&work.slot);
        }
    }

    fn mark_offline(&mut self, node: u32) {
        let state = &mut self.nodes[node as usize];
        state.phase = Phase::Offline;
        state.lease_end = None;
    }

    async fn expect_checkpoint_disconnect(
        &mut self,
        grant: &Grant,
        fault: Checkpoint,
    ) -> Result<()> {
        let end = deadline();
        loop {
            match self.transport.receive(end).await? {
                Incoming::Disconnected(node, incarnation, reason)
                    if (node, incarnation) == (grant.node_id, grant.incarnation) =>
                {
                    self.transport.diagnostic(
                        "ScriptedDisconnect",
                        json!({"reason": reason, "checkpoint": fault}),
                    )?;
                    self.transport
                        .reap(node, Some(fault.exit_code()), false)
                        .await?;
                    return Ok(());
                }
                other => self.unexpected(other)?,
            }
        }
    }

    async fn send_grant(&mut self, key: &EventKey, grant: &Grant, trace_kind: &str) -> Result<()> {
        let frame = H::Grant(grant.clone());
        self.frame(
            false,
            grant.node_id,
            &HubFrame {
                frame: Some(frame.clone()),
            },
        )?;
        self.trace(trace_kind, Some(key), json!(grant))?;
        if self.wall_pause_ms != 0 && self.wall_pause_ms <= 2 {
            tokio::time::sleep(Duration::from_millis(self.wall_pause_ms)).await;
        }
        self.transport.send(grant.node_id, frame).await
    }

    async fn await_ack(&mut self, grant: &Grant) -> Result<Ack> {
        let end = deadline();
        loop {
            match self.transport.receive(end).await? {
                Incoming::Ack(node, incarnation, ack)
                    if node == grant.node_id && ack.logical_step_id == grant.logical_step_id =>
                {
                    self.frame(
                        true,
                        node,
                        &WorkerFrame {
                            frame: Some(W::Ack(ack.clone())),
                        },
                    )?;
                    require(
                        incarnation == grant.incarnation,
                        "Ack transport does not match grant",
                    )?;
                    self.identities[node as usize].validate_ack(incarnation, &ack)?;
                    require(
                        ack.grant_hash == grant.grant_hash && ack.event_id == grant.event_id,
                        "Ack does not match grant",
                    )?;
                    return Ok(ack);
                }
                other => self.unexpected(other)?,
            }
        }
    }

    async fn duplicate_grant(&mut self, key: &EventKey, node: u32, grant: Grant) -> Result<()> {
        let ack = self.nodes[node as usize].cached[&grant.logical_step_id]
            .1
            .clone();
        self.send_grant(key, &grant, "GrantDuplicate").await?;
        let duplicate = self.await_ack(&grant).await?;
        require(duplicate == ack, "altered duplicate Ack")?;
        self.trace("AckDuplicate", Some(key), json!(duplicate))
    }

    fn commit(&mut self, key: &EventKey, grant: &Grant, ack: Ack, replay: bool) -> Result<()> {
        let i = grant.node_id as usize;
        let node = &self.nodes[i];
        require(
            add(node.seq, 1)? == ack.durable_journal_seq
                && !node.cached.contains_key(&ack.logical_step_id),
            "Ack journal seq or logical step mismatch",
        )?;
        self.validate_effects(grant, &ack)?;
        let mut staged = StagedEffects {
            node: grant.node_id,
            seed: self.scenario.seed,
            phase: node.phase,
            lease_end: node.lease_end,
            pending: node.pending.clone(),
            timers: self.timers.clone(),
            queues: self.queues.clone(),
            next_event_id: self.next_event_id,
            events: BTreeMap::new(),
            trace: Vec::new(),
        };
        for effect in &ack.effects {
            self.stage_effect(key, effect, &mut staged)?;
        }
        self.trace("JournalDurable", Some(key), json!({
            "node_id": ack.node_id, "logical_step_id": ack.logical_step_id, "journal_seq": ack.durable_journal_seq,
            "state_hash": ack.state_hash, "recovered": replay,
        }))?;
        self.trace(
            "AckCommitted",
            Some(key),
            json!({"ack": ack, "recovered": replay}),
        )?;
        for (kind, detail) in staged.trace {
            self.trace(kind, Some(key), detail)?;
        }
        let node = &mut self.nodes[i];
        node.seq = ack.durable_journal_seq;
        node.state_hash = ack.state_hash.clone();
        node.cached
            .insert(ack.logical_step_id, (grant.clone(), ack));
        node.phase = staged.phase;
        node.lease_end = staged.lease_end;
        node.pending = staged.pending;
        self.timers = staged.timers;
        self.queues = staged.queues;
        self.next_event_id = staged.next_event_id;
        self.events.extend(staged.events);
        Ok(())
    }

    fn validate_effects(&self, grant: &Grant, ack: &Ack) -> Result<()> {
        let Some(input) = grant.input.as_ref().and_then(|input| input.kind.as_ref()) else {
            return Err(invalid("missing grant kind"));
        };
        let id = &self.identities[grant.node_id as usize];
        let mut seen = std::collections::BTreeSet::new();
        for effect in &ack.effects {
            let kind = effect
                .kind
                .as_ref()
                .ok_or_else(|| invalid("empty Ack effect"))?;
            require(
                seen.insert(effect.encode_to_vec()),
                "duplicate effect within Ack",
            )?;
            let matches_input = match kind {
                E::RequestSample(request) => request.request_id > 0 && matches!(input, I::RequestSample(_) | I::Recover(_)),
                E::SampleAccepted(sample) => matches!(input, I::Sample(given) if sample == given)
                    && sample.run_id == id.run_id && sample.node_id == id.node_id && sample.incarnation == grant.incarnation
                    && sample.clock_domain == id.clock_domain() && sample.rate_profile_ref == rate_profile()
                    && sample.as_of_tick_us <= grant.tick_us && grant.tick_us < sample.lease_end_tick_us,
                E::SampleRejected(notice) => matches!(input, I::Sample(sample) if notice.request_id == sample.request_id),
                E::ArmTimer(timer) => matches!(input, I::ArmTimer(expected) if timer == expected),
                E::CancelTimer(timer) => matches!(input, I::CancelTimer(expected) if timer == expected)
                    || ack.effects.iter().any(|e| matches!(&e.kind, Some(E::Terminated(notice)) if notice.cid == timer.cid)),
                E::TimerObserved(timer) => matches!(input, I::TimerFired(expected) if timer == expected),
                E::WorkStarted(work) => matches!(input, I::BeginWork(expected) if work == expected) && work.duration_us > 0,
                E::Send(packet) => matches!(input, I::Send(expected) if packet == expected)
                    && packet.src == id.node_id && packet.sent_at_tick_us == grant.tick_us && packet.attempt == 0
                    && envelope_error(packet, &id.run_id).is_none() && self.link(packet.channel).is_ok(),
                E::MessageReceived(packet) => matches!(input, I::Deliver(expected) if packet == expected)
                    && packet.dst == id.node_id && envelope_error(packet, &id.run_id).is_none(),
                E::ClockRead(read) => (read.cid.is_empty() || read.cid.len() == 32) && ["Manager", "BFT", "Receiver"].contains(&read.caller.as_str())
                    && read.tick_us == grant.tick_us && read.node_time_us == id.local_time(grant.tick_us)?
                    && read.lease_end_tick_us > grant.tick_us
                    && self.nodes[grant.node_id as usize].lease_end == Some(read.lease_end_tick_us),
                E::SignedIntent(intent) => matches!(input, I::WorkCompleted(work) if work.cid == intent.cid && work.slot == intent.slot && work.subject == intent.subject)
                    && intent.intent_hash == intent_hash(id, intent)
                    && ack.effects.iter().any(|e| matches!(&e.kind, Some(E::ClockRead(read)) if read.caller == "BFT" && read.cid == intent.cid)),
                E::Entered(cid) => matches!(input, I::Enter(expected) if cid == expected),
                E::Terminated(notice) => notice.cid.len() == 32 && matches!(notice.reason.as_str(), "ClockUnsafe" | "Stop" | "Close"),
                E::Rejected(notice) => !notice.reason.is_empty(),
                E::Created(cid) => matches!(input, I::Create(descriptor) if cid.cid == descriptor.cid),
                E::AnchorReady(cid) => matches!(input, I::AnchorReady(expected) if cid == expected),
                E::Paused(cid) => matches!(input, I::Pause(expected) if cid == expected),
                E::Resumed(cid) => matches!(input, I::Resume(expected) if cid == expected),
            };
            require(matches_input, "Ack effect does not match the granted input")?;
        }
        let rejections = ack
            .effects
            .iter()
            .filter(|e| matches!(e.kind, Some(E::Rejected(_) | E::SampleRejected(_))))
            .count();
        require(
            rejections == usize::from(ack.status == AckStatus::Rejected as i32),
            "Ack status/effects mismatch",
        )?;
        Ok(())
    }

    fn stage_effect(
        &self,
        key: &EventKey,
        effect: &Effect,
        staged: &mut StagedEffects,
    ) -> Result<()> {
        let kind = effect
            .kind
            .as_ref()
            .ok_or_else(|| invalid("empty committed effect"))?;
        let node = staged.node;
        let i = node as usize;
        staged
            .trace
            .push(("Effect", json!({"node_id": node, "effect": effect})));
        match kind {
            E::RequestSample(req) => {
                let override_ = self
                    .scenario
                    .clock
                    .overrides
                    .iter()
                    .find(|o| o.node == node && o.request_id == req.request_id);
                let request = Request {
                    request_id: req.request_id,
                    incarnation: self.nodes[i].incarnation,
                    request_us: override_
                        .map_or(self.scenario.clock.default_request_us, |o| o.request_us),
                    response_us: override_
                        .map_or(self.scenario.clock.default_response_us, |o| o.response_us),
                    response: override_.map_or(SampleReply::Valid, |o| o.response),
                };
                staged.enqueue(key, key.tick_us, node, Event::ClockTransit(request))?;
                if let (Some(refresh), Some(until)) = (
                    self.scenario.clock.refresh_us,
                    self.scenario.clock.refresh_until_tick_us,
                ) {
                    let tick = add(key.tick_us, refresh)?;
                    if tick <= until {
                        staged.enqueue(
                            key,
                            tick,
                            node,
                            Event::Grant(input(I::RequestSample(Empty {}))),
                        )?;
                    }
                }
            }
            E::SampleAccepted(sample) => {
                staged.phase = Phase::Running;
                staged.lease_end = Some(sample.lease_end_tick_us);
                staged.enqueue(
                    key,
                    sample.lease_end_tick_us,
                    node,
                    Event::LeaseCheck {
                        incarnation: sample.incarnation,
                        end: sample.lease_end_tick_us,
                    },
                )?;
                let ids: Vec<_> = staged
                    .timers
                    .iter()
                    .filter(|((owner, _), wait)| node == *owner && wait.scheduled.is_none())
                    .map(|((_, id), _)| *id)
                    .collect();
                for id in ids {
                    staged.schedule_timer(key, id, self.identities[i].epsilon_us)?;
                }
                for mut pending in std::mem::take(&mut staged.pending) {
                    if let Some(I::Send(packet)) = pending.kind.as_mut() {
                        packet.sent_at_tick_us = key.tick_us;
                        packet.message_id = message_id(packet);
                    }
                    staged.enqueue(key, key.tick_us, node, Event::Grant(pending))?;
                }
            }
            E::ArmTimer(timer) => {
                let old = staged.timers.insert(
                    (node, timer.timer_id),
                    TimerWait {
                        timer: timer.clone(),
                        scheduled: None,
                    },
                );
                require(old.is_none(), "timer arm overwrites an active wait")?;
                staged.schedule_timer(key, timer.timer_id, self.identities[i].epsilon_us)?;
            }
            E::CancelTimer(timer) | E::TimerObserved(timer) => {
                staged.remove_wait(timer)?;
            }
            E::WorkStarted(work) => {
                staged.enqueue(
                    key,
                    add(key.tick_us, work.duration_us)?,
                    node,
                    Event::Grant(input(I::WorkCompleted(work.clone()))),
                )?;
            }
            E::Send(packet) => {
                let plan = packet_admission(
                    self.link(packet.channel)?,
                    self.scenario.seed,
                    key,
                    &mut staged.queues,
                    packet.clone(),
                )?;
                if let Some((tick, dst, event)) = plan.event {
                    staged.enqueue(key, tick, dst, event)?;
                }
                staged.trace.push((plan.kind, plan.detail));
            }
            _ => {}
        }
        Ok(())
    }

    fn sample(&mut self, key: &EventKey, node: u32, request: Request) -> Result<()> {
        let i = node as usize;
        let id = &self.identities[i];
        let clock = &mut self.nodes[i];
        clock.sample_seq = add(clock.sample_seq, 1)?;
        let local = id.local_time(key.tick_us)?;
        let mut sample = ClockSample {
            run_id: self.run_id.clone(),
            node_id: node,
            incarnation: request.incarnation,
            request_id: request.request_id,
            clock_domain: id.clock_domain(),
            sequence: clock.sample_seq,
            as_of_tick_us: key.tick_us,
            node_time_at_as_of_us: local,
            uncertainty_us: id.epsilon_us.unsigned_abs(),
            lease_end_tick_us: add(key.tick_us, self.scenario.clock.lease_us)?,
            rate_profile_ref: rate_profile(),
        };
        self.trace(
            "ClockServiceSample",
            Some(key),
            json!({"sample": sample, "one_way_response_delay_us": request.response_us}),
        )?;
        let delivery = add(key.tick_us, request.response_us)?;
        match request.response {
            SampleReply::Lost => self.trace(
                "ClockResponseDropped",
                Some(key),
                json!({"sample": sample, "scheduled_delivery_tick_us": delivery}),
            )?,
            SampleReply::Valid => {
                self.enqueue(Some(key), delivery, node, Event::ClockDeliver(sample))?;
            }
            SampleReply::Replay => {
                sample.sequence = 1;
                self.enqueue(Some(key), delivery, node, Event::ClockDeliver(sample))?;
            }
            SampleReply::InvalidThenValid => {
                let mut bad = sample.clone();
                bad.sequence = 100;
                bad.node_time_at_as_of_us += 1;
                let mut wrong_run = sample.clone();
                wrong_run.run_id = vec![0; 32];
                let mut wrong_domain = sample.clone();
                wrong_domain.clock_domain = vec![0; 32];
                let mut wrong_incarnation = sample.clone();
                wrong_incarnation.incarnation = 0;
                let mut wrong_request = sample.clone();
                wrong_request.request_id = 99;
                let mut future = sample.clone();
                future.as_of_tick_us = add(delivery, 100)?;
                let mut expired = sample.clone();
                expired.lease_end_tick_us = delivery;
                let attempts = [
                    bad,
                    wrong_run,
                    wrong_domain,
                    wrong_incarnation,
                    wrong_request,
                    future,
                    expired,
                    sample.clone(),
                    sample,
                ];
                for (offset, attempt) in attempts.into_iter().enumerate() {
                    self.enqueue(
                        Some(key),
                        add(delivery, u64::try_from(offset)?)?,
                        node,
                        Event::ClockDeliver(attempt),
                    )?;
                }
            }
        }
        Ok(())
    }

    fn link(&self, channel: u32) -> Result<&Link> {
        self.scenario
            .links
            .iter()
            .find(|link| link.channel == channel)
            .ok_or_else(|| invalid("unknown envelope channel"))
    }

    fn admit_packet(&mut self, key: &EventKey, envelope: Envelope) -> Result<()> {
        let link = self.link(envelope.channel)?.clone();
        let plan = packet_admission(&link, self.scenario.seed, key, &mut self.queues, envelope)?;
        if let Some((tick, dst, event)) = plan.event {
            self.enqueue(Some(key), tick, dst, event)?;
        }
        self.trace(plan.kind, Some(key), plan.detail)
    }

    async fn deliver_packet(&mut self, key: &EventKey, mut envelope: Envelope) -> Result<()> {
        let link = self.link(envelope.channel)?.clone();
        let online = self.nodes[envelope.dst as usize].online()
            && self.nodes[envelope.dst as usize].phase != Phase::Recovering
            && !link
                .offline
                .iter()
                .any(|[start, end]| *start <= key.tick_us && key.tick_us < *end);
        if !online {
            self.trace(
                "PacketDroppedOffline",
                Some(key),
                json!({"envelope": envelope}),
            )?;
            if envelope.attempt == 0 {
                if let Some(delay) = link.retry_delay_us {
                    envelope.attempt = 1;
                    self.trace(
                        "PacketRetryScheduled",
                        Some(key),
                        json!({"envelope": envelope, "retry_delay_us": delay}),
                    )?;
                    self.enqueue(
                        Some(key),
                        add(key.tick_us, delay)?,
                        envelope.dst,
                        Event::PacketRetry(envelope),
                    )?;
                }
            }
            return Ok(());
        }
        self.trace("PacketDelivered", Some(key), json!({"envelope": envelope}))?;
        self.execute(key, envelope.dst, input(I::Deliver(envelope.clone())))
            .await?;
        if envelope.attempt == 0 {
            if let Some(delay) = link.duplicate_delay_us {
                envelope.attempt = 1;
                self.trace(
                    "PacketDuplicateScheduled",
                    Some(key),
                    json!({"envelope": envelope, "duplicate_delay_us": delay}),
                )?;
                self.enqueue(
                    Some(key),
                    add(key.tick_us, delay)?,
                    envelope.dst,
                    Event::PacketRetry(envelope),
                )?;
            }
        }
        Ok(())
    }
}

struct Admission {
    kind: &'static str,
    detail: Value,
    event: Option<(u64, u32, Event)>,
}

fn packet_admission(
    link: &Link,
    seed: u64,
    key: &EventKey,
    queues: &mut BTreeMap<(u32, u32, u32), Queue>,
    envelope: Envelope,
) -> Result<Admission> {
    let route = (envelope.src, envelope.dst, envelope.channel);
    let queue = queues.entry(route).or_default();
    if queue.packets.len() == link.capacity {
        return Ok(Admission {
            kind: "QueueOverflow",
            detail: json!({"envelope": envelope, "capacity": link.capacity, "policy": link.overflow}),
            event: None,
        });
    }
    let jitter = if link.jitter_us > 0 {
        let rank = hash_parts(
            "I1/tie/v1",
            &[&seed.to_be_bytes(), &key.event_id.to_be_bytes()],
        );
        u64::from_be_bytes(rank[..8].try_into()?) % add(link.jitter_us, 1)?
    } else {
        0
    };
    let due = add(add(key.tick_us, link.delay_us)?, jitter)?;
    let event = queue
        .packets
        .is_empty()
        .then_some((due, envelope.dst, Event::PacketDue(route)));
    queue.packets.push_back((due, envelope.clone()));
    Ok(Admission {
        kind: "PacketAdmitted",
        detail: json!({"envelope": envelope, "earliest_delivery_tick_us": due, "jitter_us": jitter}),
        event,
    })
}

fn event_key(
    seed: u64,
    next_event_id: &mut u64,
    parent: Option<&EventKey>,
    tick: u64,
) -> Result<EventKey> {
    let wave = if let Some(parent) = parent {
        require(tick >= parent.tick_us, "derived event in the past")?;
        if tick == parent.tick_us {
            add(parent.causal_wave, 1)?
        } else {
            0
        }
    } else {
        0
    };
    *next_event_id = add(*next_event_id, 1)?;
    let id = *next_event_id;
    let tie_rank = hash_parts("I1/tie/v1", &[&seed.to_be_bytes(), &id.to_be_bytes()]);
    Ok(EventKey {
        tick_us: tick,
        causal_wave: wave,
        tie_rank,
        event_id: id,
    })
}

fn add(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| invalid("unsigned counter/time overflow"))
}

fn input(kind: I) -> Input {
    Input { kind: Some(kind) }
}
fn input_recover() -> Input {
    input(I::Recover(Empty {}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_and_calibration_do_not_release_waiting_work() {
        let work = input(I::WorkCompleted(Work::default()));
        let recover = input_recover();
        let sample = input(I::Sample(ClockSample::default()));
        assert!(!Phase::Recovering.allows(&work));
        assert!(!Phase::Recovering.allows(&sample));
        assert!(Phase::Recovering.allows(&recover));
        assert!(!Phase::Calibrating.allows(&work));
        assert!(Phase::Calibrating.allows(&sample));
        assert!(Phase::Running.allows(&work));
    }

    #[test]
    fn failed_effect_plan_leaves_live_timer_and_counter_untouched() {
        let live = BTreeMap::from([(
            (0, 1),
            TimerWait {
                timer: Timer {
                    timer_id: 1,
                    generation: 1,
                    local_deadline_us: 1_050_000,
                    cid: cid("first"),
                },
                scheduled: Some(4),
            },
        )]);
        let mut staged = StagedEffects {
            node: 0,
            seed: 7,
            phase: Phase::Running,
            lease_end: Some(1_100_000),
            pending: vec![],
            timers: live.clone(),
            queues: BTreeMap::new(),
            next_event_id: 4,
            events: BTreeMap::new(),
            trace: vec![],
        };
        let parent = EventKey {
            tick_us: 1_040_000,
            causal_wave: 0,
            tie_rank: vec![0; 32],
            event_id: 3,
        };
        staged
            .enqueue(&parent, parent.tick_us, 0, Event::Grant(input_recover()))
            .unwrap();
        let original = live[&(0, 1)].timer.clone();
        let changed = Timer {
            generation: 2,
            ..original.clone()
        };
        assert!(staged.remove_wait(&changed).is_err());
        assert_eq!(live[&(0, 1)].timer, original);
        assert_eq!(live[&(0, 1)].scheduled, Some(4));
        assert_eq!(staged.next_event_id, 5);
    }
}
