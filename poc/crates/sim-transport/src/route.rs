use crate::{
    ChannelId, Endpoint, EndpointBinding, Envelope, Error, Id, Instance, LinkProfile, OfflineMode,
    OrderMode, StepStamp, Tick, TransportConfig,
    config::RouteKey,
    framed_hash,
    identity::rebind,
    snapshot::{
        MAX_SNAPSHOT, SCHEMA, decode_snapshot, encode, encode_snapshot, schema, strictly_ordered,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Arc,
};

const MAX_COMMANDS: usize = 4096;
const MAX_EVENTS: usize = 8192;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum RouteCommand {
    Send(Envelope),
    Drain,
    Rebind {
        endpoint: Endpoint,
        expected_incarnation: u64,
        new_incarnation: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PacketRef {
    pub logical_id: Id,
    pub attempt: u32,
    pub copy_index: u8,
    pub packet_seq: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum DropReason {
    Fault,
    Offline,
    LinkCapacity,
    GlobalCapacity,
    PayloadBudget,
    StaleBinding,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Delivery {
    pub envelope: Envelope,
    pub packet: PacketRef,
    pub raw_due_us: Tick,
    pub eligible_tick_us: Tick,
    pub delivered_tick_us: Tick,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum RouteEventKind {
    Queued {
        packet: PacketRef,
        raw_due_us: Tick,
    },
    Dropped {
        packet: PacketRef,
        reason: DropReason,
        raw_due_us: Option<Tick>,
    },
    Delivered(Delivery),
    Rebound(EndpointBinding),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteEvent {
    pub step: StepStamp,
    pub effect_index: u32,
    pub kind: RouteEventKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct QueuedPacket {
    envelope: Envelope,
    packet: PacketRef,
    raw_due_us: Tick,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Link {
    source: Endpoint,
    destination: Endpoint,
    channel: ChannelId,
    send_ordinal: u64,
    next_release_us: Tick,
    queue: VecDeque<QueuedPacket>,
}

impl Link {
    fn key(&self) -> RouteKey {
        (self.source, self.destination, self.channel)
    }

    fn first(&self, profile: &LinkProfile, seed: u64) -> Option<(usize, &QueuedPacket)> {
        match profile.order {
            OrderMode::Fifo => self.queue.front().map(|packet| (0, packet)),
            OrderMode::Unordered => self.queue.iter().enumerate().min_by_key(|(_, p)| {
                (
                    p.raw_due_us,
                    tie(seed, p.packet.packet_seq),
                    p.packet.packet_seq,
                )
            }),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Attempt {
    logical_id: Id,
    last_attempt: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteSnapshot {
    schema: u32,
    kernel_config_hash: Id,
    revision: u64,
    last_step: Option<StepStamp>,
    next_packet_seq: u64,
    bindings: Vec<EndpointBinding>,
    attempts: Vec<Attempt>,
    links: Vec<Link>,
}

impl RouteSnapshot {
    fn binding(&self, endpoint: Endpoint) -> Result<u64, Error> {
        Ok(self.bindings[self.binding_index(endpoint)?].incarnation)
    }

    fn binding_index(&self, endpoint: Endpoint) -> Result<usize, Error> {
        self.bindings
            .binary_search_by_key(&endpoint, |b| b.endpoint)
            .map_err(|_| Error::UnknownEndpoint)
    }

    fn allocate_packet(&mut self, envelope: &Envelope, copy_index: u8) -> Result<PacketRef, Error> {
        let packet_seq = self.next_packet_seq;
        self.next_packet_seq = packet_seq.checked_add(1).ok_or(Error::ArithmeticOverflow)?;
        Ok(PacketRef {
            logical_id: envelope.message.logical_id,
            attempt: envelope.attempt,
            copy_index,
            packet_seq,
        })
    }

    fn track_attempt(&mut self, envelope: &Envelope, limit: u32) -> Result<(), Error> {
        let logical_id = envelope.message.logical_id;
        let last_attempt = envelope.attempt;
        match self
            .attempts
            .binary_search_by_key(&logical_id, |a| a.logical_id)
        {
            Ok(index) => {
                if last_attempt <= self.attempts[index].last_attempt {
                    return Err(Error::InvalidAttempt);
                }
                self.attempts[index].last_attempt = last_attempt;
            }
            Err(index) => {
                if self.attempts.len() >= limit as usize {
                    return Err(Error::ResourceLimit);
                }
                self.attempts.insert(
                    index,
                    Attempt {
                        logical_id,
                        last_attempt,
                    },
                );
            }
        }
        Ok(())
    }

    fn queued_usage(&self) -> (usize, usize) {
        self.links
            .iter()
            .flat_map(|link| link.queue.iter())
            .fold((0, 0), |(copies, bytes), queued| {
                (copies + 1, bytes + queued.envelope.message.payload.len())
            })
    }
}

#[derive(Debug)]
struct Profile {
    config: TransportConfig,
    expanded: BTreeMap<RouteKey, usize>,
}

impl Profile {
    fn link(&self, key: RouteKey) -> &LinkProfile {
        &self.config.rules[self.expanded[&key]].profile
    }
}

#[derive(Debug)]
pub struct RouteKernel {
    instance: Instance,
    profile: Arc<Profile>,
    state: RouteSnapshot,
}

impl Clone for RouteKernel {
    fn clone(&self) -> Self {
        Self {
            instance: Instance::new(),
            profile: Arc::clone(&self.profile),
            state: self.state.clone(),
        }
    }
}

#[derive(Debug)]
pub struct RoutePlan {
    instance: Instance,
    base_revision: u64,
    candidate: RouteSnapshot,
    events: Vec<RouteEvent>,
}

impl RoutePlan {
    pub fn base_revision(&self) -> u64 {
        self.base_revision
    }

    pub fn candidate_snapshot_bytes(&self) -> Result<Vec<u8>, Error> {
        encode_snapshot(&self.candidate)
    }
}

#[derive(Debug)]
struct Events {
    step: StepStamp,
    bytes: usize,
    values: Vec<RouteEvent>,
}

impl Events {
    fn new(step: StepStamp) -> Self {
        Self {
            step,
            bytes: 2,
            values: Vec::new(),
        }
    }

    fn push(&mut self, kind: RouteEventKind) -> Result<(), Error> {
        if self.values.len() >= MAX_EVENTS {
            return Err(Error::ResourceLimit);
        }
        let effect_index =
            u32::try_from(self.values.len()).map_err(|_| Error::ArithmeticOverflow)?;
        let event = RouteEvent {
            step: self.step,
            effect_index,
            kind,
        };
        let bytes = self.bytes + usize::from(!self.values.is_empty()) + encode(&event)?.len();
        if bytes > MAX_SNAPSHOT {
            return Err(Error::ResourceLimit);
        }
        self.bytes = bytes;
        self.values.push(event);
        Ok(())
    }

    fn drop_packet(
        &mut self,
        packet: PacketRef,
        reason: DropReason,
        raw_due_us: Option<Tick>,
    ) -> Result<(), Error> {
        self.push(RouteEventKind::Dropped {
            packet,
            reason,
            raw_due_us,
        })
    }
}

#[derive(Clone, Copy)]
struct Candidate {
    link_index: usize,
    queue_index: usize,
    eligible_tick: Tick,
    tie_rank: Id,
    packet_seq: u64,
}

fn tie(seed: u64, packet_seq: u64) -> Id {
    framed_hash(
        "SIMT/tie/v1",
        &[&seed.to_be_bytes(), &packet_seq.to_be_bytes()],
    )
}

fn jitter(seed: u64, envelope: &Envelope, bound: u64) -> u64 {
    let message = &envelope.message;
    let digest = framed_hash(
        "SIMT/jitter/v1",
        &[
            &seed.to_be_bytes(),
            &message.source.hash_bytes(),
            &message.destination.hash_bytes(),
            &message.channel.0.to_be_bytes(),
            &message.logical_id,
            &envelope.attempt.to_be_bytes(),
        ],
    );
    let word = u64::from_be_bytes(digest[..8].try_into().expect("SHA-256 has 32 bytes"));
    word % (bound + 1)
}

impl RouteKernel {
    pub fn new(config: TransportConfig) -> Result<Self, Error> {
        let (config, expanded) = config.normalize_and_expand()?;
        let state = RouteSnapshot {
            schema: SCHEMA,
            kernel_config_hash: config.normalized_hash()?,
            revision: 0,
            last_step: None,
            next_packet_seq: 1,
            bindings: config
                .endpoints
                .iter()
                .map(|endpoint| endpoint.binding)
                .collect(),
            attempts: Vec::new(),
            links: expanded
                .keys()
                .map(|&(source, destination, channel)| Link {
                    source,
                    destination,
                    channel,
                    send_ordinal: 0,
                    next_release_us: 0,
                    queue: VecDeque::new(),
                })
                .collect(),
        };
        encode_snapshot(&state)?;
        Ok(Self {
            instance: Instance::new(),
            profile: Arc::new(Profile { config, expanded }),
            state,
        })
    }

    pub fn plan(&self, step: StepStamp, commands: &[RouteCommand]) -> Result<RoutePlan, Error> {
        if commands.len() > MAX_COMMANDS {
            return Err(Error::ResourceLimit);
        }
        if step.event_id == 0 || step.logical_step_id == 0 {
            return Err(Error::StepRegression);
        }
        if let Some(previous) = self.state.last_step {
            if step.tick_us < previous.tick_us {
                return Err(Error::TimeRegression);
            }
            if step.logical_step_id <= previous.logical_step_id {
                return Err(Error::StepRegression);
            }
        }

        let mut candidate = self.state.clone();
        candidate.revision = candidate
            .revision
            .checked_add(1)
            .ok_or(Error::ArithmeticOverflow)?;
        candidate.last_step = Some(step);
        let mut events = Events::new(step);
        for command in commands {
            match command {
                RouteCommand::Send(envelope) => self.send(&mut candidate, envelope, &mut events)?,
                RouteCommand::Drain => self.drain(&mut candidate, &mut events)?,
                RouteCommand::Rebind {
                    endpoint,
                    expected_incarnation,
                    new_incarnation,
                } => {
                    let index = candidate.binding_index(*endpoint)?;
                    let binding = &mut candidate.bindings[index];
                    binding.incarnation =
                        rebind(binding.incarnation, *expected_incarnation, *new_incarnation)?;
                    events.push(RouteEventKind::Rebound(*binding))?;
                }
            }
        }

        if encode(&candidate)?.len() + events.bytes > MAX_SNAPSHOT {
            return Err(Error::ResourceLimit);
        }
        Ok(RoutePlan {
            instance: self.instance.for_plan(),
            base_revision: self.state.revision,
            candidate,
            events: events.values,
        })
    }

    pub fn commit(&mut self, plan: RoutePlan) -> Result<Vec<RouteEvent>, Error> {
        self.instance.verify(&plan.instance)?;
        if plan.candidate.kernel_config_hash != self.config_hash() {
            return Err(Error::ConfigMismatch);
        }
        if plan.base_revision != self.revision() {
            return Err(Error::StalePlan);
        }
        self.state = plan.candidate;
        Ok(plan.events)
    }

    pub fn next_wakeup_us(&self) -> Option<Tick> {
        let committed = self.state.last_step.map_or(0, |s| s.tick_us);
        self.best_candidate(&self.state, committed)
            .map(|candidate| candidate.eligible_tick.max(committed))
    }

    pub fn revision(&self) -> u64 {
        self.state.revision
    }

    pub fn config_hash(&self) -> Id {
        self.state.kernel_config_hash
    }

    pub fn snapshot_bytes(&self) -> Result<Vec<u8>, Error> {
        encode_snapshot(&self.state)
    }

    pub fn from_snapshot(config: TransportConfig, bytes: &[u8]) -> Result<Self, Error> {
        let state: RouteSnapshot = decode_snapshot(bytes)?;
        schema(state.schema)?;
        let mut kernel = Self::new(config)?;
        if state.kernel_config_hash != kernel.config_hash() {
            return Err(Error::ConfigMismatch);
        }
        kernel.validate_snapshot(&state)?;
        kernel.state = state;
        Ok(kernel)
    }

    fn send(
        &self,
        state: &mut RouteSnapshot,
        envelope: &Envelope,
        events: &mut Events,
    ) -> Result<(), Error> {
        let message = &envelope.message;
        let config = &self.profile.config;
        config.check_message(message, false)?;
        if state.binding(message.source)? != envelope.source_incarnation
            || state.binding(message.destination)? != envelope.destination_incarnation
        {
            return Err(Error::BindingMismatch);
        }
        if envelope.attempt == 0 {
            return Err(Error::InvalidAttempt);
        }
        let tick = events.step.tick_us;
        if message.origin.sent_tick_us > tick
            || message.origin.source_incarnation > envelope.source_incarnation
        {
            return Err(Error::InvalidOrigin);
        }
        let key = (message.source, message.destination, message.channel);
        let link_index = state
            .links
            .binary_search_by_key(&key, Link::key)
            .map_err(|_| Error::MissingProfile)?;
        state.track_attempt(envelope, config.limits.tracked_messages)?;
        let link = &mut state.links[link_index];
        link.send_ordinal = link
            .send_ordinal
            .checked_add(1)
            .ok_or(Error::ArithmeticOverflow)?;
        let profile = self.profile.link(key);
        let fault = profile
            .faults
            .binary_search_by_key(&link.send_ordinal, |f| f.send_ordinal)
            .ok()
            .map(|index| &profile.faults[index]);
        let packet = state.allocate_packet(envelope, 0)?;
        if fault.is_some_and(|f| f.drop) {
            return events.drop_packet(packet, DropReason::Fault, None);
        }
        if profile.offline_mode == OfflineMode::Drop && profile.offline_at(tick) {
            return events.drop_packet(packet, DropReason::Offline, None);
        }

        let raw_due = tick
            .checked_add(profile.latency_us)
            .and_then(|v| v.checked_add(jitter(config.seed, envelope, profile.jitter_us)))
            .and_then(|v| v.checked_add(fault.map_or(0, |f| f.extra_delay_us)))
            .ok_or(Error::ArithmeticOverflow)?;
        self.enqueue_copy(state, link_index, envelope, packet, raw_due, events)?;
        if let Some(delay) = fault.and_then(|f| f.duplicate_delay_us) {
            let raw_due = raw_due
                .checked_add(delay)
                .ok_or(Error::ArithmeticOverflow)?;
            let duplicate = state.allocate_packet(envelope, 1)?;
            self.enqueue_copy(state, link_index, envelope, duplicate, raw_due, events)?;
        }
        Ok(())
    }

    fn enqueue_copy(
        &self,
        state: &mut RouteSnapshot,
        link_index: usize,
        envelope: &Envelope,
        packet: PacketRef,
        raw_due_us: Tick,
        events: &mut Events,
    ) -> Result<(), Error> {
        let link = &state.links[link_index];
        let profile = self.profile.link(link.key());
        let (copies, bytes) = state.queued_usage();
        let limits = &self.profile.config.limits;
        let reason = if link.queue.len() >= profile.capacity as usize {
            Some(DropReason::LinkCapacity)
        } else if copies >= limits.queued_copies as usize {
            Some(DropReason::GlobalCapacity)
        } else if (bytes + envelope.message.payload.len()) as u64 > limits.queued_payload_bytes {
            Some(DropReason::PayloadBudget)
        } else {
            None
        };
        if let Some(reason) = reason {
            return events.drop_packet(packet, reason, Some(raw_due_us));
        }
        state.links[link_index].queue.push_back(QueuedPacket {
            envelope: envelope.clone(),
            packet,
            raw_due_us,
        });
        events.push(RouteEventKind::Queued { packet, raw_due_us })
    }

    fn best_candidate(&self, state: &RouteSnapshot, current_tick: Tick) -> Option<Candidate> {
        let seed = self.profile.config.seed;
        state
            .links
            .iter()
            .enumerate()
            .filter_map(|(link_index, link)| {
                let profile = self.profile.link(link.key());
                let (queue_index, queued) = link.first(profile, seed)?;
                let mut eligible_tick = queued.raw_due_us.max(link.next_release_us);
                if profile.offline_mode == OfflineMode::Hold {
                    eligible_tick = profile.online_tick(eligible_tick);
                    // A late Drain cannot deliver inside a later outage.
                    if eligible_tick <= current_tick && profile.offline_at(current_tick) {
                        eligible_tick = profile.online_tick(current_tick);
                    }
                }
                let packet_seq = queued.packet.packet_seq;
                Some(Candidate {
                    link_index,
                    queue_index,
                    eligible_tick,
                    tie_rank: tie(seed, packet_seq),
                    packet_seq,
                })
            })
            .min_by_key(|c| (c.eligible_tick, c.tie_rank, c.packet_seq))
    }

    fn drain(&self, state: &mut RouteSnapshot, events: &mut Events) -> Result<(), Error> {
        let tick = events.step.tick_us;
        while let Some(candidate) = self
            .best_candidate(state, tick)
            .filter(|c| c.eligible_tick <= tick)
        {
            let link = &mut state.links[candidate.link_index];
            let profile = self.profile.link(link.key());
            let QueuedPacket {
                envelope,
                packet,
                raw_due_us,
            } = link
                .queue
                .remove(candidate.queue_index)
                .expect("selected queue entry exists");
            let reason = if profile.offline_mode == OfflineMode::Drop && profile.offline_at(tick) {
                Some(DropReason::Offline)
            } else if state.binding(envelope.message.destination)?
                != envelope.destination_incarnation
            {
                Some(DropReason::StaleBinding)
            } else {
                None
            };
            if let Some(reason) = reason {
                events.drop_packet(packet, reason, Some(raw_due_us))?;
                continue;
            }
            let spacing = profile.spacing(envelope.message.payload.len())?;
            state.links[candidate.link_index].next_release_us =
                tick.checked_add(spacing).ok_or(Error::ArithmeticOverflow)?;
            events.push(RouteEventKind::Delivered(Delivery {
                envelope,
                packet,
                raw_due_us,
                eligible_tick_us: candidate.eligible_tick,
                delivered_tick_us: tick,
            }))?;
        }
        Ok(())
    }

    fn validate_snapshot(&self, state: &RouteSnapshot) -> Result<(), Error> {
        let config = &self.profile.config;
        if state.next_packet_seq == 0
            || state.attempts.len() > config.limits.tracked_messages as usize
            || state.attempts.iter().any(|a| a.last_attempt == 0)
            || !strictly_ordered(&state.attempts, |a| a.logical_id)
            || state.bindings.len() != config.endpoints.len()
            || state.links.len() != self.profile.expanded.len()
            || (state.revision == 0) != state.last_step.is_none()
        {
            return Err(Error::CorruptSnapshot);
        }
        if state
            .bindings
            .iter()
            .zip(&config.endpoints)
            .any(|(binding, origin)| {
                binding.endpoint != origin.binding.endpoint
                    || binding.incarnation < origin.binding.incarnation
            })
        {
            return Err(Error::CorruptSnapshot);
        }
        if state.revision == 0
            && (state.next_packet_seq != 1
                || !state.attempts.is_empty()
                || state.bindings != self.state.bindings)
        {
            return Err(Error::CorruptSnapshot);
        }
        let (copies, bytes) = state.queued_usage();
        if copies > config.limits.queued_copies as usize
            || bytes as u64 > config.limits.queued_payload_bytes
        {
            return Err(Error::CorruptSnapshot);
        }
        let mut packet_sequences = BTreeSet::new();
        let mut copies = BTreeSet::new();
        let mut used_ordinals = 0_u64;
        for (link, &key) in state.links.iter().zip(self.profile.expanded.keys()) {
            let profile = self.profile.link(key);
            used_ordinals = used_ordinals
                .checked_add(link.send_ordinal)
                .ok_or(Error::CorruptSnapshot)?;
            if link.key() != key
                || link.queue.len() > profile.capacity as usize
                || (state.revision == 0 && (link.send_ordinal != 0 || link.next_release_us != 0))
                || (link.send_ordinal == 0 && (!link.queue.is_empty() || link.next_release_us != 0))
                || !link
                    .queue
                    .iter()
                    .map(|q| q.packet.packet_seq)
                    .is_sorted_by(|a, b| a < b)
            {
                return Err(Error::CorruptSnapshot);
            }
            for queued in &link.queue {
                let QueuedPacket {
                    envelope,
                    packet,
                    raw_due_us,
                } = queued;
                let message = &envelope.message;
                config
                    .check_message(message, false)
                    .map_err(|_| Error::CorruptSnapshot)?;
                let step = state.last_step.ok_or(Error::CorruptSnapshot)?;
                let source = config
                    .endpoint(message.source)
                    .map_err(|_| Error::CorruptSnapshot)?;
                let destination = config
                    .endpoint(message.destination)
                    .map_err(|_| Error::CorruptSnapshot)?;
                let minimum_due = message
                    .origin
                    .sent_tick_us
                    .checked_add(profile.latency_us)
                    .and_then(|v| v.checked_add(jitter(config.seed, envelope, profile.jitter_us)));
                let last_attempt = state
                    .attempts
                    .binary_search_by_key(&packet.logical_id, |a| a.logical_id)
                    .ok()
                    .map(|index| state.attempts[index].last_attempt);
                if (message.source, message.destination, message.channel) != key
                    || last_attempt.is_none_or(|a| a < packet.attempt)
                    || packet.logical_id != message.logical_id
                    || packet.attempt == 0
                    || packet.attempt != envelope.attempt
                    || packet.copy_index > 1
                    || packet.packet_seq == 0
                    || packet.packet_seq >= state.next_packet_seq
                    || !packet_sequences.insert(packet.packet_seq)
                    || !copies.insert((packet.logical_id, packet.attempt, packet.copy_index))
                    || message.origin.sent_tick_us > step.tick_us
                    || minimum_due.is_none_or(|minimum| *raw_due_us < minimum)
                    || envelope.source_incarnation < source.binding.incarnation
                    || envelope.destination_incarnation < destination.binding.incarnation
                    || envelope.source_incarnation < message.origin.source_incarnation
                    || envelope.source_incarnation > state.binding(message.source)?
                    || envelope.destination_incarnation > state.binding(message.destination)?
                {
                    return Err(Error::CorruptSnapshot);
                }
            }
        }
        let allocated = state.next_packet_seq - 1;
        if used_ordinals > allocated
            || allocated > used_ordinals.saturating_mul(2)
            || state.attempts.len() as u64 > used_ordinals
            || state
                .last_step
                .is_some_and(|s| s.logical_step_id < state.revision || s.event_id == 0)
        {
            return Err(Error::CorruptSnapshot);
        }
        Ok(())
    }
}
