use crate::{Error, Id, Result, hash_parts};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub run_id: Id,
    pub config_hash: Id,
    pub run_seed: u64,
    pub node: u32,
    pub incarnation: u64,
    pub epsilon_us: i64,
}

impl Profile {
    pub fn clock_domain(&self) -> Id {
        hash_parts(
            b"I1/clock-domain/v1",
            &[&self.run_id, &self.config_hash, &self.node.to_be_bytes()],
        )
    }

    pub fn rate_profile() -> Id {
        hash_parts(b"I1/rate/v1", &[b"fixed-rate-one-v1"])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockSample {
    pub run_id: Id,
    pub node: u32,
    pub incarnation: u64,
    pub clock_domain: Id,
    pub rate_profile: Id,
    pub request_id: u64,
    pub sequence: u64,
    pub as_of_tick_us: u64,
    pub node_time_at_as_of_us: i64,
    pub uncertainty_us: u64,
    pub lease_end_tick_us: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timer {
    pub id: u64,
    pub cid: Id,
    pub deadline_us: i64,
}

pub(crate) struct NodeClock {
    pub profile: Profile,
    pub tick_us: u64,
    prior: Option<ClockSample>,
    requests: BTreeSet<u64>,
    last_request: u64,
    last_sequence: u64,
}

impl NodeClock {
    pub fn new(profile: Profile) -> Result<Self> {
        if profile.incarnation == 0 || profile.epsilon_us.unsigned_abs() > 1000 {
            return Err(Error("invalid clock profile"));
        }
        Ok(Self {
            profile,
            tick_us: 0,
            prior: None,
            requests: BTreeSet::new(),
            last_request: 0,
            last_sequence: 0,
        })
    }

    pub fn request(&mut self) -> Result<u64> {
        self.last_request = self
            .last_request
            .checked_add(1)
            .ok_or(Error("clock request counter overflow"))?;
        self.requests.insert(self.last_request);
        Ok(self.last_request)
    }

    pub fn sample(&mut self, sample: ClockSample) -> Result<()> {
        let profile = &self.profile;
        if sample.run_id != profile.run_id
            || sample.node != profile.node
            || sample.incarnation != profile.incarnation
            || sample.clock_domain != profile.clock_domain()
            || sample.rate_profile != Profile::rate_profile()
        {
            return Err(Error("clock sample identity"));
        }
        if !self.requests.contains(&sample.request_id) || sample.sequence <= self.last_sequence {
            return Err(Error("clock sample replay or unknown request"));
        }
        if sample.as_of_tick_us > self.tick_us || self.tick_us >= sample.lease_end_tick_us {
            return Err(Error("clock sample time"));
        }
        if sample.uncertainty_us > 1000 || sample.uncertainty_us < profile.epsilon_us.unsigned_abs()
        {
            return Err(Error("clock sample uncertainty"));
        }
        let at = i64::try_from(sample.as_of_tick_us)
            .ok()
            .and_then(|tick| tick.checked_add(profile.epsilon_us));
        if at != Some(sample.node_time_at_as_of_us)
            || Self::project(&sample, self.tick_us).is_none()
        {
            return Err(Error("clock sample oscillator"));
        }
        self.requests.remove(&sample.request_id);
        self.last_sequence = sample.sequence;
        self.prior = Some(sample);
        Ok(())
    }

    pub fn local(&self) -> Option<i64> {
        let sample = self.prior.as_ref()?;
        if self.tick_us >= sample.lease_end_tick_us {
            return None;
        }
        Self::project(sample, self.tick_us)
    }

    fn project(sample: &ClockSample, tick: u64) -> Option<i64> {
        let elapsed = i64::try_from(tick.checked_sub(sample.as_of_tick_us)?).ok()?;
        sample.node_time_at_as_of_us.checked_add(elapsed)
    }
}

pub(crate) fn system_time(local: i64) -> SystemTime {
    let duration = Duration::from_micros(local.unsigned_abs());
    if local >= 0 {
        UNIX_EPOCH + duration
    } else {
        UNIX_EPOCH - duration
    }
}

pub(crate) fn local_time(time: SystemTime) -> i64 {
    let micros = match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            i128::try_from(duration.as_nanos().div_ceil(1000)).expect("positive deadline overflow")
        }
        Err(error) => {
            -i128::try_from(error.duration().as_micros()).expect("negative deadline overflow")
        }
    };
    i64::try_from(micros).expect("clock deadline outside i64 microseconds")
}
