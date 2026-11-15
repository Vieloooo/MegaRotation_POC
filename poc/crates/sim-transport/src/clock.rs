use crate::{
    Endpoint, Error, Id, LocalTime, Tick, framed_hash,
    identity::rebind,
    snapshot::{SCHEMA, decode_snapshot, encode_snapshot, schema, strictly_ordered},
};
use serde::{Deserialize, Serialize};

const MAX_OUTSTANDING: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClockIdentity {
    pub run: Id,
    pub config: Id,
    pub endpoint: Endpoint,
    pub incarnation: u64,
    pub domain: Id,
    pub epsilon_us: i64,
}

impl ClockIdentity {
    fn check(&self) -> Result<(), Error> {
        if self.run == [0; 32]
            || self.config == [0; 32]
            || self.domain == [0; 32]
            || self.incarnation == 0
            || self.epsilon_us.unsigned_abs() > 1000
        {
            return Err(Error::InvalidConfig);
        }
        Ok(())
    }

    fn project(&self, tick: Tick) -> Result<LocalTime, Error> {
        let local = i128::from(tick) + i128::from(self.epsilon_us);
        LocalTime::try_from(local).map_err(|_| Error::ArithmeticOverflow)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClockSample {
    pub run: Id,
    pub config: Id,
    pub endpoint: Endpoint,
    pub incarnation: u64,
    pub domain: Id,
    pub rate_profile: Id,
    pub request_id: u64,
    pub sequence: u64,
    pub as_of_tick_us: Tick,
    pub local_at_as_of_us: LocalTime,
    pub uncertainty_us: u64,
    pub lease_end_tick_us: Tick,
}

fn check_sample(
    identity: &ClockIdentity,
    delivery: Tick,
    sample: &ClockSample,
) -> Result<(), Error> {
    if sample.run != identity.run || sample.config != identity.config {
        return Err(Error::ConfigMismatch);
    }
    if sample.endpoint != identity.endpoint || sample.incarnation != identity.incarnation {
        return Err(Error::BindingMismatch);
    }
    if sample.domain != identity.domain
        || sample.rate_profile != framed_hash("SIMT/rate/v1", &[b"fixed-rate-one-v1"])
        || sample.as_of_tick_us > delivery
        || delivery >= sample.lease_end_tick_us
        || !(identity.epsilon_us.unsigned_abs()..=1000).contains(&sample.uncertainty_us)
        || identity.project(sample.as_of_tick_us)? != sample.local_at_as_of_us
    {
        return Err(Error::InvalidSample);
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ClockView {
    reference_tick_us: Tick,
    local_time_us: LocalTime,
    lease_end_tick_us: Tick,
    sample_sequence: u64,
    epsilon_us: i64,
}

impl ClockView {
    pub fn reference_tick_us(&self) -> Tick {
        self.reference_tick_us
    }

    pub fn local_time_us(&self) -> LocalTime {
        self.local_time_us
    }

    pub fn lease_end_tick_us(&self) -> Tick {
        self.lease_end_tick_us
    }

    pub fn sample_sequence(&self) -> u64 {
        self.sample_sequence
    }

    pub fn local_deadline_tick_us(&self, absolute_local_us: LocalTime) -> Result<Tick, Error> {
        let tick = i128::from(absolute_local_us) - i128::from(self.epsilon_us);
        Tick::try_from(tick.max(0)).map_err(|_| Error::ArithmeticOverflow)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClockSnapshot {
    schema: u32,
    identity: ClockIdentity,
    request_counter: u64,
    outstanding: Vec<u64>,
    accepted_sequence: u64,
    prior_lease_end_tick_us: u64,
    sample: Option<ClockSample>,
    sample_delivery_tick_us: Option<u64>,
}

#[derive(Debug)]
pub struct ClockTracker {
    state: ClockSnapshot,
}

impl ClockTracker {
    pub fn new(identity: ClockIdentity) -> Result<Self, Error> {
        identity.check()?;
        let state = ClockSnapshot {
            schema: SCHEMA,
            identity,
            request_counter: 0,
            outstanding: Vec::new(),
            accepted_sequence: 0,
            prior_lease_end_tick_us: 0,
            sample: None,
            sample_delivery_tick_us: None,
        };
        Ok(Self { state })
    }

    pub fn request(&mut self) -> Result<u64, Error> {
        if self.state.outstanding.len() >= MAX_OUTSTANDING {
            return Err(Error::ResourceLimit);
        }
        let request = self
            .state
            .request_counter
            .checked_add(1)
            .ok_or(Error::ArithmeticOverflow)?;
        self.state.outstanding.push(request);
        self.state.request_counter = request;
        Ok(request)
    }

    pub fn install(&mut self, delivery_tick_us: Tick, sample: ClockSample) -> Result<(), Error> {
        check_sample(&self.state.identity, delivery_tick_us, &sample)?;
        if sample.sequence <= self.state.accepted_sequence {
            return Err(Error::StaleSample);
        }
        if self
            .state
            .sample_delivery_tick_us
            .is_some_and(|tick| delivery_tick_us < tick)
        {
            return Err(Error::TimeRegression);
        }
        let index = self
            .state
            .outstanding
            .binary_search(&sample.request_id)
            .map_err(|_| Error::InvalidSample)?;
        self.state.outstanding.remove(index);
        self.state.accepted_sequence = sample.sequence;
        self.state.prior_lease_end_tick_us = sample.lease_end_tick_us;
        self.state.sample = Some(sample);
        self.state.sample_delivery_tick_us = Some(delivery_tick_us);
        Ok(())
    }

    pub fn view(&self, tick_us: Tick) -> Result<ClockView, Error> {
        let sample = self.state.sample.as_ref().ok_or(Error::Uncalibrated)?;
        let delivery = self
            .state
            .sample_delivery_tick_us
            .ok_or(Error::CorruptSnapshot)?;
        if tick_us < delivery {
            return Err(Error::TimeRegression);
        }
        if tick_us >= sample.lease_end_tick_us {
            return Err(Error::LeaseExpired);
        }
        Ok(ClockView {
            reference_tick_us: tick_us,
            local_time_us: self.state.identity.project(tick_us)?,
            lease_end_tick_us: sample.lease_end_tick_us,
            sample_sequence: sample.sequence,
            epsilon_us: self.state.identity.epsilon_us,
        })
    }

    pub fn rebind(&mut self, expected_incarnation: u64, new_incarnation: u64) -> Result<(), Error> {
        self.state.identity.incarnation = rebind(
            self.state.identity.incarnation,
            expected_incarnation,
            new_incarnation,
        )?;
        self.state.sample = None;
        self.state.outstanding.clear();
        Ok(())
    }

    pub fn snapshot_bytes(&self) -> Result<Vec<u8>, Error> {
        encode_snapshot(&self.state)
    }

    pub fn from_snapshot(identity: ClockIdentity, bytes: &[u8]) -> Result<Self, Error> {
        identity.check()?;
        let state: ClockSnapshot = decode_snapshot(bytes)?;
        schema(state.schema)?;
        if state.identity != identity {
            return Err(Error::ConfigMismatch);
        }
        if state.outstanding.len() > MAX_OUTSTANDING
            || !strictly_ordered(&state.outstanding, |id| *id)
            || state
                .outstanding
                .iter()
                .any(|&id| id == 0 || id > state.request_counter)
            || (state.accepted_sequence == 0) != state.sample_delivery_tick_us.is_none()
            || (state.accepted_sequence == 0) != (state.prior_lease_end_tick_us == 0)
            || (state.accepted_sequence != 0 && state.request_counter == 0)
            || state
                .sample_delivery_tick_us
                .is_some_and(|tick| tick >= state.prior_lease_end_tick_us)
        {
            return Err(Error::CorruptSnapshot);
        }
        if let Some(sample) = &state.sample {
            let delivery = state
                .sample_delivery_tick_us
                .ok_or(Error::CorruptSnapshot)?;
            check_sample(&identity, delivery, sample).map_err(|_| Error::CorruptSnapshot)?;
            if sample.sequence != state.accepted_sequence
                || sample.sequence == 0
                || sample.request_id == 0
                || sample.request_id > state.request_counter
                || state.outstanding.binary_search(&sample.request_id).is_ok()
                || sample.lease_end_tick_us != state.prior_lease_end_tick_us
            {
                return Err(Error::CorruptSnapshot);
            }
        }
        Ok(Self { state })
    }
}
