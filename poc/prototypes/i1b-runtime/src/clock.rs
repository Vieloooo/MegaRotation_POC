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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ClockCheckpoint {
    #[serde(with = "checkpoint_id")]
    run_id: Id,
    worker_id: u32,
    old_incarnation: u64,
    #[serde(with = "checkpoint_id")]
    clock_domain: Id,
    #[serde(with = "checkpoint_id")]
    rate_profile: Id,
    last_issued_request_id: u64,
    last_accepted_sample_sequence: u64,
    last_observed_reference_tick_us: u64,
    last_observed_local_us: Option<i64>,
    old_lease_end_tick_us: Option<u64>,
    clock_terminal_state: bool,
    outstanding_request_ids: Vec<ClockRequestBinding>,
}

mod checkpoint_id {
    use crate::{Id, native_recovery_ledger::hex};
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(value: &Id, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex(value))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Id, D::Error> {
        let text = String::deserialize(deserializer)?;
        crate::native_recovery::fixed(&text).map_err(serde::de::Error::custom)
    }
}
impl ClockCheckpoint {
    pub(crate) fn old_lease_end(&self) -> Option<u64> {
        self.old_lease_end_tick_us
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ClockRequestBinding {
    request_id: u64,
    original_incarnation: u64,
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

    pub(crate) fn recovery_checkpoint(&self, terminal: bool) -> ClockCheckpoint {
        ClockCheckpoint {
            run_id: self.profile.run_id,
            worker_id: self.profile.node,
            old_incarnation: self.profile.incarnation,
            clock_domain: self.profile.clock_domain(),
            rate_profile: Profile::rate_profile(),
            last_issued_request_id: self.last_request,
            last_accepted_sample_sequence: self.last_sequence,
            last_observed_reference_tick_us: self.tick_us,
            last_observed_local_us: self.local(),
            old_lease_end_tick_us: self.calibration().map(|(_, expiry)| expiry),
            clock_terminal_state: terminal,
            outstanding_request_ids: self
                .requests
                .iter()
                .map(|id| ClockRequestBinding {
                    request_id: *id,
                    original_incarnation: self.profile.incarnation,
                })
                .collect(),
        }
    }

    /// Restore the anti-replay frontiers only. The older sample and its outstanding
    /// responses never become clock authority in the new process.
    pub(crate) fn restore_recovery(
        &mut self,
        checkpoint: &ClockCheckpoint,
        recovery_tick: u64,
        original_stop_us: i64,
    ) -> Result<()> {
        let profile = &self.profile;
        let expected_old_local = i64::try_from(checkpoint.last_observed_reference_tick_us)
            .ok()
            .and_then(|tick| tick.checked_add(profile.epsilon_us));
        let recovery_local = i64::try_from(recovery_tick)
            .ok()
            .and_then(|tick| tick.checked_add(profile.epsilon_us));
        if self.prior.is_some()
            || self.tick_us != 0
            || !self.requests.is_empty()
            || self.last_request != 0
            || self.last_sequence != 0
            || checkpoint.run_id != profile.run_id
            || checkpoint.worker_id != profile.node
            || checkpoint.old_incarnation.checked_add(1) != Some(profile.incarnation)
            || checkpoint.clock_domain != profile.clock_domain()
            || checkpoint.rate_profile != Profile::rate_profile()
            || checkpoint.clock_terminal_state
            || checkpoint.last_issued_request_id == 0
            || checkpoint.last_issued_request_id == u64::MAX
            || checkpoint.last_accepted_sample_sequence == 0
            || checkpoint.last_accepted_sample_sequence == u64::MAX
            || checkpoint.last_observed_local_us.is_none()
            || checkpoint.last_observed_local_us != expected_old_local
            || recovery_tick < checkpoint.last_observed_reference_tick_us
            || checkpoint
                .old_lease_end_tick_us
                .is_none_or(|end| recovery_tick >= end)
            || recovery_local.is_none_or(|local| local >= original_stop_us)
            || checkpoint
                .outstanding_request_ids
                .windows(2)
                .any(|pair| pair[0].request_id >= pair[1].request_id)
            || checkpoint.outstanding_request_ids.iter().any(|entry| {
                entry.request_id == 0
                    || entry.request_id > checkpoint.last_issued_request_id
                    || entry.original_incarnation != checkpoint.old_incarnation
            })
        {
            return Err(Error("ineligible recovery clock checkpoint"));
        }
        self.last_request = checkpoint.last_issued_request_id;
        self.last_sequence = checkpoint.last_accepted_sample_sequence;
        self.tick_us = recovery_tick;
        Ok(())
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

    pub(crate) fn calibration(&self) -> Option<(u64, u64)> {
        self.prior
            .as_ref()
            .map(|sample| (sample.sequence, sample.lease_end_tick_us))
    }

    pub(crate) fn projected_at(&self, tick: u64) -> Option<i64> {
        let sample = self.prior.as_ref()?;
        if tick < self.tick_us || tick >= sample.lease_end_tick_us {
            return None;
        }
        Self::project(sample, tick)
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

#[cfg(test)]
mod recovery_tests {
    use super::*;

    #[test]
    fn new_incarnation_keeps_frontiers_and_requires_its_own_calibration() {
        let profile = Profile {
            run_id: [1; 32],
            config_hash: [2; 32],
            run_seed: 7,
            node: 1,
            incarnation: 1,
            epsilon_us: -1000,
        };
        let mut old = NodeClock::new(profile.clone()).unwrap();
        old.tick_us = 1_020_000;
        assert_eq!(old.request().unwrap(), 1);
        let old_sample = ClockSample {
            run_id: profile.run_id,
            node: profile.node,
            incarnation: 1,
            clock_domain: profile.clock_domain(),
            rate_profile: Profile::rate_profile(),
            request_id: 1,
            sequence: 7,
            as_of_tick_us: 1_020_000,
            node_time_at_as_of_us: 1_019_000,
            uncertainty_us: 1000,
            lease_end_tick_us: 1_120_000,
        };
        old.sample(old_sample.clone()).unwrap();
        assert_eq!(old.request().unwrap(), 2);
        let checkpoint = old.recovery_checkpoint(false);
        let mut profile2 = profile;
        profile2.incarnation = 2;
        let mut next = NodeClock::new(profile2.clone()).unwrap();
        next.restore_recovery(&checkpoint, 1_050_000, 1_800_000)
            .unwrap();
        assert!(next.local().is_none());
        assert_eq!(next.request().unwrap(), 3);
        assert!(next.sample(old_sample.clone()).is_err());
        let mut fresh = ClockSample {
            incarnation: 2,
            request_id: 2,
            sequence: 8,
            as_of_tick_us: 1_050_000,
            node_time_at_as_of_us: 1_049_000,
            lease_end_tick_us: 1_150_000,
            ..old_sample
        };
        assert!(next.sample(fresh.clone()).is_err()); // old outstanding ID
        fresh.request_id = 3;
        fresh.sequence = 7;
        assert!(next.sample(fresh.clone()).is_err()); // old sample frontier
        fresh.sequence = 8;
        next.sample(fresh).unwrap();
        assert_eq!(next.local(), Some(1_049_000));
        for (tick, stop, terminal) in [
            (1_120_000, 1_800_000, false),
            (1_050_000, 1_049_000, false),
            (1_050_000, 1_800_000, true),
        ] {
            let mut rejected = NodeClock::new(profile2.clone()).unwrap();
            assert!(
                rejected
                    .restore_recovery(&old.recovery_checkpoint(terminal), tick, stop)
                    .is_err()
            );
            assert!(rejected.local().is_none());
        }
    }
}
