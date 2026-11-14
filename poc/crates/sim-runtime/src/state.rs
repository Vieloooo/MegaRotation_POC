use super::*;
use poc_types::{effect::Kind as E, input::Kind as I};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct State {
    run_id: Vec<u8>,
    config_hash: Vec<u8>,
    node_id: u32,
    fixtures: BTreeMap<String, FixtureState>,
    timers: BTreeMap<u64, TimerState>,
    first_received: BTreeMap<String, FirstReceive>,
    sample_request_counter: u64,
    outstanding_requests: BTreeSet<u64>,
    accepted_sequence: u64,
    prior_lease_end_tick_us: Option<u64>,
    sample: Option<ClockSample>,
    last_tick_us: Option<u64>,
    runtime_incarnation: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct FixtureState {
    descriptor: Fixture,
    created_at_tick_us: u64,
    anchor_ready_tick_us: Option<u64>,
    entered_at_tick_us: Option<u64>,
    view: u64,
    terminal: Option<Terminal>,
    paused: bool,
    signed_intents: BTreeMap<u64, SignedIntent>,
    pending_work: BTreeMap<u64, PendingWork>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Terminal {
    reason: String,
    tick_us: u64,
    lease_end_tick_us: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PendingWork {
    work: Work,
    due_tick_us: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct TimerState {
    timer: Timer,
    active: bool,
    observed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct FirstReceive {
    envelope: Envelope,
    first_tick_us: u64,
    first_local_time_us: Option<i64>,
}

impl State {
    pub fn new(identity: &Identity) -> Self {
        Self {
            run_id: identity.run_id.clone(),
            config_hash: identity.config_hash.clone(),
            node_id: identity.node_id,
            fixtures: BTreeMap::new(),
            timers: BTreeMap::new(),
            first_received: BTreeMap::new(),
            sample_request_counter: 0,
            outstanding_requests: BTreeSet::new(),
            accepted_sequence: 0,
            prior_lease_end_tick_us: None,
            sample: None,
            last_tick_us: None,
            runtime_incarnation: 1,
        }
    }

    pub fn validate_identity(&self, identity: &Identity) -> Result<()> {
        require(
            self.run_id == identity.run_id
                && self.config_hash == identity.config_hash
                && self.node_id == identity.node_id,
            "journal state identity mismatch",
        )
    }

    pub(super) fn incarnation(&self) -> u64 {
        self.runtime_incarnation
    }

    pub fn hash(&self) -> Result<Vec<u8>> {
        Ok(hash_parts("I1/state/v1", &[&serde_json::to_vec(self)?]))
    }

    pub(super) fn apply(
        &mut self,
        identity: &Identity,
        grant: &Grant,
        journal_seq: u64,
    ) -> Result<Ack> {
        let tick = grant.tick_us;
        require(
            self.last_tick_us.is_none_or(|previous| tick >= previous),
            "virtual time regressed",
        )?;
        self.last_tick_us = Some(tick);
        let input = grant
            .input
            .as_ref()
            .and_then(|input| input.kind.as_ref())
            .ok_or_else(|| invalid("missing input"))?;
        let mut effects = Vec::new();
        let status =
            match self.apply_input(identity, grant.incarnation, tick, input, &mut effects)? {
                None => AckStatus::Quiescent,
                Some(rejected) => {
                    effects.push(rejected);
                    AckStatus::Rejected
                }
            };
        let mut ack = Ack {
            schema_version: SCHEMA_VERSION,
            run_id: identity.run_id.clone(),
            node_id: identity.node_id,
            incarnation: grant.incarnation,
            logical_step_id: grant.logical_step_id,
            event_id: grant.event_id,
            grant_hash: grant.grant_hash.clone(),
            status: status as i32,
            effects,
            durable_journal_seq: journal_seq,
            state_hash: self.hash()?,
            logical_ack_hash: Vec::new(),
        };
        ack.logical_ack_hash = ack_hash(&ack);
        Ok(ack)
    }

    fn apply_input(
        &mut self,
        identity: &Identity,
        incarnation: u64,
        tick: u64,
        input: &I,
        effects: &mut Vec<Effect>,
    ) -> Result<Option<Effect>> {
        match input {
            I::RequestSample(_) => self.request_sample(effects)?,
            I::Sample(sample) => {
                if let Some(reason) = self.sample_error(identity, incarnation, tick, sample)? {
                    return Ok(Some(effect(E::SampleRejected(notice(
                        &[],
                        reason,
                        sample.request_id,
                    )))));
                }
                self.expire_clock(tick, effects);
                self.outstanding_requests.remove(&sample.request_id);
                self.accepted_sequence = sample.sequence;
                self.prior_lease_end_tick_us = Some(sample.lease_end_tick_us);
                self.sample = Some(sample.clone());
                effects.push(effect(E::SampleAccepted(sample.clone())));
                let local = identity.local_time(tick)?;
                self.stop_expired_fixtures(tick, local, effects);
            }
            I::Create(descriptor) => {
                let cid = &descriptor.cid;
                if cid.len() != 32 {
                    return Ok(reject(cid, "FixtureCid"));
                }
                let key = hex(cid);
                if let Some(fixture) = self.fixtures.get(&key) {
                    if fixture.descriptor != *descriptor {
                        return Ok(reject(cid, "FixtureDescriptorConflict"));
                    }
                } else {
                    self.fixtures.insert(
                        key,
                        FixtureState {
                            descriptor: descriptor.clone(),
                            created_at_tick_us: tick,
                            anchor_ready_tick_us: None,
                            entered_at_tick_us: None,
                            view: 0,
                            terminal: None,
                            paused: false,
                            signed_intents: BTreeMap::new(),
                            pending_work: BTreeMap::new(),
                        },
                    );
                    effects.push(effect(E::Created(Cid { cid: cid.clone() })));
                }
            }
            I::AnchorReady(cid) => {
                let Some(fixture) = self.fixtures.get_mut(&hex(&cid.cid)) else {
                    return Ok(reject(&cid.cid, "UnknownFixture"));
                };
                if fixture.terminal.is_some() {
                    return Ok(reject(&cid.cid, "Terminated"));
                }
                if fixture.anchor_ready_tick_us.is_none() {
                    fixture.anchor_ready_tick_us = Some(tick);
                    effects.push(effect(E::AnchorReady(cid.clone())));
                }
            }
            I::Enter(cid) => {
                if let Some(reason) = self.guard(identity, tick, &cid.cid, &["Manager"], effects)? {
                    return Ok(reject(&cid.cid, reason));
                }
                let fixture = self
                    .fixtures
                    .get_mut(&hex(&cid.cid))
                    .expect("guard found fixture");
                if fixture.anchor_ready_tick_us.is_none() {
                    return Ok(reject(&cid.cid, "AnchorNotReady"));
                }
                if fixture.entered_at_tick_us.is_none() {
                    fixture.entered_at_tick_us = Some(tick);
                    effects.push(effect(E::Entered(cid.clone())));
                }
            }
            I::ReadClock(cid) => {
                if let Some(reason) =
                    self.guard(identity, tick, &cid.cid, &["Manager", "BFT"], effects)?
                {
                    return Ok(reject(&cid.cid, reason));
                }
            }
            I::CheckClock(_) => self.expire_clock(tick, effects),
            I::BeginWork(work) => {
                if let Some(reason) = self.active_guard(identity, tick, &work.cid, effects)? {
                    return Ok(reject(&work.cid, reason));
                }
                let fixture = self
                    .fixtures
                    .get_mut(&hex(&work.cid))
                    .expect("guard found fixture");
                if work.subject.len() != 32 || work.duration_us == 0 {
                    return Ok(reject(&work.cid, "InvalidWork"));
                }
                if fixture.signed_intents.contains_key(&work.slot)
                    || fixture.pending_work.contains_key(&work.slot)
                {
                    return Ok(reject(&work.cid, "WorkCoordinateOccupied"));
                }
                let due_tick_us = tick
                    .checked_add(work.duration_us)
                    .ok_or_else(|| invalid("work deadline overflow"))?;
                fixture.pending_work.insert(
                    work.slot,
                    PendingWork {
                        work: work.clone(),
                        due_tick_us,
                    },
                );
                effects.push(effect(E::WorkStarted(work.clone())));
            }
            I::WorkCompleted(work) => {
                let key = hex(&work.cid);
                let pending = self
                    .fixtures
                    .get(&key)
                    .and_then(|fixture| fixture.pending_work.get(&work.slot));
                if !pending
                    .is_some_and(|pending| pending.work == *work && pending.due_tick_us <= tick)
                {
                    return Ok(reject(&work.cid, "UnknownWorkCompletion"));
                }
                self.fixtures
                    .get_mut(&key)
                    .expect("pending fixture")
                    .pending_work
                    .remove(&work.slot);
                if let Some(reason) = self.active_guard(identity, tick, &work.cid, effects)? {
                    return Ok(reject(&work.cid, reason));
                }
                let fixture = self.fixtures.get_mut(&key).expect("guard found fixture");
                let mut intent = SignedIntent {
                    cid: work.cid.clone(),
                    slot: work.slot,
                    subject: work.subject.clone(),
                    intent_hash: Vec::new(),
                };
                intent.intent_hash = intent_hash(identity, &intent);
                fixture.view = fixture.view.max(work.slot);
                fixture.signed_intents.insert(work.slot, intent.clone());
                effects.push(effect(E::SignedIntent(intent)));
            }
            I::ArmTimer(timer) => {
                if let Some(reason) = self.timer_guard(identity, tick, timer, effects)? {
                    return Ok(reject(&timer.cid, reason));
                }
                if let Some(previous) = self.timers.get(&timer.timer_id) {
                    if previous.active
                        || previous.timer.generation.checked_add(1) != Some(timer.generation)
                    {
                        return Ok(reject(&timer.cid, "InvalidTimerGeneration"));
                    }
                } else if timer.generation != 1 {
                    return Ok(reject(&timer.cid, "InvalidTimerGeneration"));
                }
                self.timers.insert(
                    timer.timer_id,
                    TimerState {
                        timer: timer.clone(),
                        active: true,
                        observed: false,
                    },
                );
                effects.push(effect(E::ArmTimer(timer.clone())));
            }
            I::CancelTimer(timer) => {
                let Some(previous) = self.timers.get_mut(&timer.timer_id) else {
                    return Ok(reject(&timer.cid, "StaleTimerGeneration"));
                };
                if !previous.active || previous.timer != *timer {
                    return Ok(reject(&timer.cid, "StaleTimerGeneration"));
                }
                previous.active = false;
                effects.push(effect(E::CancelTimer(timer.clone())));
            }
            I::TimerFired(timer) => {
                if !self
                    .timers
                    .get(&timer.timer_id)
                    .is_some_and(|previous| previous.active && previous.timer == *timer)
                {
                    return Ok(reject(&timer.cid, "StaleTimerGeneration"));
                }
                if let Some(reason) = self.timer_guard(identity, tick, timer, effects)? {
                    return Ok(reject(&timer.cid, reason));
                }
                let local = self
                    .project_sample(tick)?
                    .ok_or_else(|| invalid("timer guard has no sample"))?;
                if local < timer.local_deadline_us {
                    return Ok(reject(&timer.cid, "EarlyTimer"));
                }
                let previous = self.timers.get_mut(&timer.timer_id).expect("timer checked");
                previous.active = false;
                previous.observed = true;
                effects.push(effect(E::TimerObserved(timer.clone())));
            }
            I::Close(cid) => {
                if !self.fixtures.contains_key(&hex(&cid.cid)) {
                    return Ok(reject(&cid.cid, "UnknownFixture"));
                }
                self.terminate(&cid.cid, tick, "Close", effects);
            }
            I::Pause(cid) | I::Resume(cid) => {
                if let Some(reason) = self.guard(identity, tick, &cid.cid, &["Manager"], effects)? {
                    return Ok(reject(&cid.cid, reason));
                }
                let paused = matches!(input, I::Pause(_));
                let fixture = self
                    .fixtures
                    .get_mut(&hex(&cid.cid))
                    .expect("guard found fixture");
                if fixture.entered_at_tick_us.is_none() {
                    return Ok(reject(&cid.cid, "NotEntered"));
                }
                if fixture.paused != paused {
                    fixture.paused = paused;
                    effects.push(effect(if paused {
                        E::Paused(cid.clone())
                    } else {
                        E::Resumed(cid.clone())
                    }));
                }
            }
            I::Send(envelope) | I::Deliver(envelope) => {
                if let Some(reason) = envelope_error(envelope, &identity.run_id) {
                    return Ok(reject(&envelope.cid, reason));
                }
                if !envelope.cid.is_empty() && !self.fixtures.contains_key(&hex(&envelope.cid)) {
                    return Ok(reject(&envelope.cid, "UnknownFixture"));
                }
                if matches!(input, I::Send(_)) {
                    if envelope.src != identity.node_id
                        || envelope.sent_at_tick_us != tick
                        || envelope.attempt != 0
                    {
                        return Ok(reject(&envelope.cid, "EnvelopeSender"));
                    }
                    effects.push(effect(E::Send(envelope.clone())));
                } else {
                    if envelope.dst != identity.node_id || envelope.sent_at_tick_us > tick {
                        return Ok(reject(&envelope.cid, "EnvelopeReceiver"));
                    }
                    self.expire_clock(tick, effects);
                    let key = hex(&envelope.message_id);
                    if self.first_received.contains_key(&key) {
                        return Ok(reject(&envelope.cid, "DuplicateEnvelope"));
                    }
                    let clock = self.clock(identity, tick)?;
                    self.first_received.insert(
                        key,
                        FirstReceive {
                            envelope: envelope.clone(),
                            first_tick_us: tick,
                            first_local_time_us: clock.map(|(local, _)| local),
                        },
                    );
                    let Some((local, end)) = clock else {
                        return Ok(reject(&envelope.cid, "ClockUnsafe"));
                    };
                    effects.push(clock_read(&envelope.cid, "Receiver", tick, local, end));
                    effects.push(effect(E::MessageReceived(envelope.clone())));
                }
            }
            I::Recover(_) => {
                self.runtime_incarnation = incarnation;
                self.expire_clock(tick, effects);
                self.sample = None;
                self.outstanding_requests.clear();
                self.request_sample(effects)?;
            }
        }
        Ok(None)
    }

    fn request_sample(&mut self, effects: &mut Vec<Effect>) -> Result<()> {
        self.sample_request_counter = self
            .sample_request_counter
            .checked_add(1)
            .ok_or_else(|| invalid("request counter overflow"))?;
        self.outstanding_requests
            .insert(self.sample_request_counter);
        effects.push(effect(E::RequestSample(ClockRequest {
            request_id: self.sample_request_counter,
        })));
        Ok(())
    }

    fn sample_error(
        &self,
        identity: &Identity,
        incarnation: u64,
        tick: u64,
        sample: &ClockSample,
    ) -> Result<Option<&'static str>> {
        let reason = if sample.run_id != identity.run_id
            || sample.node_id != identity.node_id
            || sample.incarnation != incarnation
        {
            "SampleIdentity"
        } else if sample.clock_domain != identity.clock_domain()
            || sample.rate_profile_ref != rate_profile()
        {
            "SampleDomainOrRate"
        } else if !self.outstanding_requests.contains(&sample.request_id) {
            "SampleRequest"
        } else if sample.sequence <= self.accepted_sequence {
            "SampleSequence"
        } else if sample.as_of_tick_us > tick || tick >= sample.lease_end_tick_us {
            "SampleTime"
        } else if identity.local_time(sample.as_of_tick_us).ok()
            != Some(sample.node_time_at_as_of_us)
        {
            "SampleOscillator"
        } else if sample.uncertainty_us > 1000
            || sample.uncertainty_us < identity.epsilon_us.unsigned_abs()
        {
            "SampleUncertainty"
        } else {
            let projected = project(sample, tick).ok();
            if projected.is_none()
                || projected != identity.local_time(tick).ok()
                || self
                    .project_sample(tick)?
                    .is_some_and(|old| Some(old) != projected)
            {
                return Ok(Some("SampleContinuity"));
            }
            return Ok(None);
        };
        Ok(Some(reason))
    }

    fn project_sample(&self, tick: u64) -> Result<Option<i64>> {
        self.sample
            .as_ref()
            .map(|sample| project(sample, tick))
            .transpose()
    }

    fn clock(&self, identity: &Identity, tick: u64) -> Result<Option<(i64, u64)>> {
        let Some(sample) = self
            .sample
            .as_ref()
            .filter(|sample| tick < sample.lease_end_tick_us)
        else {
            return Ok(None);
        };
        let local = project(sample, tick)?;
        require(
            local == identity.local_time(tick)? && local.abs_diff(i64::try_from(tick)?) <= 1000,
            "profile violation at clock read",
        )?;
        Ok(Some((local, sample.lease_end_tick_us)))
    }

    fn guard(
        &mut self,
        identity: &Identity,
        tick: u64,
        cid: &[u8],
        callers: &[&str],
        effects: &mut Vec<Effect>,
    ) -> Result<Option<&'static str>> {
        self.expire_clock(tick, effects);
        let Some((local, lease_end)) = self.clock(identity, tick)? else {
            return Ok(Some("ClockUnsafe"));
        };
        let Some(fixture) = self.fixtures.get(&hex(cid)) else {
            return Ok(Some("UnknownFixture"));
        };
        if fixture.terminal.is_some() {
            return Ok(Some("Terminated"));
        }
        if local >= fixture.descriptor.stop_local_us {
            self.terminate(cid, tick, "Stop", effects);
            return Ok(Some("LocalStop"));
        }
        for caller in callers {
            effects.push(clock_read(cid, caller, tick, local, lease_end));
        }
        Ok(None)
    }

    fn active_guard(
        &mut self,
        identity: &Identity,
        tick: u64,
        cid: &[u8],
        effects: &mut Vec<Effect>,
    ) -> Result<Option<&'static str>> {
        if let Some(reason) = self.guard(identity, tick, cid, &["BFT"], effects)? {
            return Ok(Some(reason));
        }
        let fixture = &self.fixtures[&hex(cid)];
        if fixture.entered_at_tick_us.is_none() {
            return Ok(Some("NotEntered"));
        }
        Ok(fixture.paused.then_some("Paused"))
    }

    fn timer_guard(
        &mut self,
        identity: &Identity,
        tick: u64,
        timer: &Timer,
        effects: &mut Vec<Effect>,
    ) -> Result<Option<&'static str>> {
        if timer.cid.is_empty() {
            self.expire_clock(tick, effects);
            return Ok(self
                .clock(identity, tick)?
                .is_none()
                .then_some("ClockUnsafe"));
        }
        self.guard(identity, tick, &timer.cid, &["BFT"], effects)
    }

    fn stop_expired_fixtures(&mut self, tick: u64, local: i64, effects: &mut Vec<Effect>) {
        let cids: Vec<_> = self
            .fixtures
            .values()
            .filter(|f| {
                f.entered_at_tick_us.is_some()
                    && f.terminal.is_none()
                    && local >= f.descriptor.stop_local_us
            })
            .map(|f| f.descriptor.cid.clone())
            .collect();
        for cid in cids {
            self.terminate(&cid, tick, "Stop", effects);
        }
    }

    fn expire_clock(&mut self, tick: u64, effects: &mut Vec<Effect>) {
        if self.prior_lease_end_tick_us.is_some_and(|end| tick >= end) {
            let cids: Vec<_> = self
                .fixtures
                .values()
                .filter(|fixture| {
                    fixture.entered_at_tick_us.is_some() && fixture.terminal.is_none()
                })
                .map(|fixture| fixture.descriptor.cid.clone())
                .collect();
            for cid in cids {
                self.terminate(&cid, tick, "ClockUnsafe", effects);
            }
        }
    }

    fn terminate(&mut self, cid: &[u8], tick: u64, reason: &str, effects: &mut Vec<Effect>) {
        let fixture = self.fixtures.get_mut(&hex(cid)).expect("known fixture");
        let first = fixture.terminal.is_none();
        let terminal = fixture.terminal.get_or_insert_with(|| Terminal {
            reason: reason.to_owned(),
            tick_us: tick,
            lease_end_tick_us: self.prior_lease_end_tick_us,
        });
        effects.push(effect(E::Terminated(notice(cid, &terminal.reason, 0))));
        if first {
            for timer in self
                .timers
                .values_mut()
                .filter(|timer| timer.active && timer.timer.cid == cid)
            {
                timer.active = false;
                effects.push(effect(E::CancelTimer(timer.timer.clone())));
            }
        }
    }
}

fn project(sample: &ClockSample, tick: u64) -> Result<i64> {
    let elapsed = tick
        .checked_sub(sample.as_of_tick_us)
        .ok_or_else(|| invalid("projection precedes sample"))?;
    sample
        .node_time_at_as_of_us
        .checked_add(i64::try_from(elapsed)?)
        .ok_or_else(|| invalid("clock projection overflow"))
}

fn clock_read(cid: &[u8], caller: &str, tick: u64, local: i64, end: u64) -> Effect {
    effect(E::ClockRead(ClockRead {
        cid: cid.to_vec(),
        caller: caller.to_owned(),
        tick_us: tick,
        node_time_us: local,
        lease_end_tick_us: end,
    }))
}

fn notice(cid: &[u8], reason: &str, request_id: u64) -> Notice {
    Notice {
        cid: cid.to_vec(),
        reason: reason.to_owned(),
        request_id,
    }
}

fn effect(kind: E) -> Effect {
    Effect { kind: Some(kind) }
}

fn reject(cid: &[u8], reason: &str) -> Option<Effect> {
    Some(effect(E::Rejected(notice(cid, reason, 0))))
}
