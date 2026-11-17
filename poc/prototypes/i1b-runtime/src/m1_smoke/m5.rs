//! M5-only fault profiles and recovery authorization arithmetic.

use poc_types::domain::{Hash32, PublicConfig};
use serde::{Deserialize, Serialize};

use super::{M1Result, problem, profile::SECOND};

pub(super) const EXPIRY_US: u64 = 12 * SECOND;
pub(super) const PREP_US: u64 = 12 * SECOND;
pub(super) const READY_US: u64 = 500_000;
pub(super) const EPSILON_US: u64 = SECOND;
pub(super) const SKEW_US: u64 = 1_000;
pub(super) const LATE_RELEASE_US: u64 = 43_500_000;
pub(super) const SIGNER_READERS: [&str; 3] = ["S0", "S1", "S2"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Mode {
    Stopped,
    Frozen,
    SlowRpc,
    Late,
    Short,
    Divergence,
    NoTarget,
    InputsWait,
}

impl Mode {
    pub fn parse(name: &str) -> M1Result<Self> {
        Ok(match name {
            "stopped" => Self::Stopped,
            "frozen" => Self::Frozen,
            "slow-rpc" => Self::SlowRpc,
            "late" => Self::Late,
            "short" => Self::Short,
            "divergence" => Self::Divergence,
            "no-target" => Self::NoTarget,
            "inputs-wait" => Self::InputsWait,
            _ => return Err(problem("unknown M5 scenario")),
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Frozen => "frozen",
            Self::SlowRpc => "slow-rpc",
            Self::Late => "late",
            Self::Short => "short",
            Self::Divergence => "divergence",
            Self::NoTarget => "no-target",
            Self::InputsWait => "inputs-wait",
        }
    }

    pub fn stall(self) -> u64 {
        if self == Self::Divergence {
            12_250_000
        } else {
            11_250_000
        }
    }

    pub fn recovery(self) -> u64 {
        match self {
            Self::Short => 13 * SECOND,
            Self::Divergence => 22_500_000,
            _ => 24_500_000,
        }
    }

    pub fn transit(self) -> u64 {
        if self == Self::SlowRpc {
            1_000
        } else {
            super::profile::CLOCK_TRANSIT_US
        }
    }

    pub fn contrast(self) -> Option<u64> {
        match self {
            Self::Short => Some(14_500_000),
            Self::Divergence => Some(24 * SECOND),
            Self::NoTarget | Self::InputsWait => Some(26 * SECOND),
            _ => None,
        }
    }
}

/// Values emitted only by the named recipient of a checked finalized L1 read.
/// The genesis row is included in hashes at index zero.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct DObservation {
    pub l1_height: u64,
    pub inbox_height: u64,
    pub d_height: u64,
    pub d_origin: u64,
    pub hashes: Vec<Hash32>,
    pub drained: bool,
    pub services_ready: bool,
    pub pending_batch_through: Option<u64>,
}

#[derive(Clone, Copy)]
pub(super) struct Heads {
    pub g: u64,
    pub halted: bool,
    pub d: u64,
    pub origin: u64,
    pub inbox: u64,
    pub pending_batch_through: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(super) struct Solution {
    pub t_l_min_us: u64,
    pub t_l_max_us: u64,
    pub b_hi: u64,
    pub e_hi: u64,
    pub h_time: u64,
    pub extra_origin: u64,
    pub g_max: u64,
    pub d_max: u64,
    pub inbox_max: u64,
    pub h_guard: u64,
    pub lower_time_us: u64,
    pub upper_time_us: u64,
    pub lower_height: u64,
    pub upper_height: Option<u64>,
    pub target: Option<u64>,
}

impl Solution {
    pub fn accepts_inclusion(self, timestamp_us: u64) -> bool {
        (self.t_l_min_us..=self.t_l_max_us).contains(&timestamp_us)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(super) struct ResumeTarget {
    pub target: u64,
    pub origin: u64,
}

/// Convert the configured time interval to an inclusive integer interval,
/// then intersect it with the independently bounded production horizon.
pub(super) fn solve(
    config: &PublicConfig,
    announcement_us: u64,
    heads: Heads,
    t_l_min_us: u64,
    t_l_max_us: u64,
) -> Option<Solution> {
    let l1_genesis = config.l1_genesis_timestamp_us;
    let l2_genesis = config.genesis.timestamp_us;
    let h0 = config.genesis.block.height;
    if config.l2_slot_us == 0
        || config.l1_block_us == 0
        || t_l_min_us > t_l_max_us
        || t_l_min_us.checked_sub(l1_genesis)? % config.l1_block_us != 0
        || t_l_max_us.checked_sub(l1_genesis)? % config.l1_block_us != 0
        || PREP_US < announcement_us
        || announcement_us
            .checked_add(READY_US)?
            .checked_add(SKEW_US)?
            > PREP_US.checked_add(config.l2_slot_us)?
    {
        return None;
    }
    let ceil = |t: u64| h0.checked_add(t.saturating_sub(l2_genesis).div_ceil(config.l2_slot_us));
    let floor = |t: u64| h0.checked_add(t.checked_sub(l2_genesis)?.checked_div(config.l2_slot_us)?);
    let b_hi = t_l_max_us
        .checked_sub(l1_genesis)?
        .checked_div(config.l1_block_us)?;
    let e_hi = b_hi.checked_add(1)?.saturating_sub(4);
    let h_time = ceil(l1_genesis.checked_add(e_hi.checked_mul(config.l1_block_us)?)?)?;
    let extra_origin = e_hi.saturating_sub(heads.origin).checked_add(1)?;
    let inbox_max = heads.inbox.max(heads.pending_batch_through);
    let d_max = heads
        .d
        .max(inbox_max)
        .max(h_time)
        .checked_add(extra_origin)?;
    let g_max = if heads.halted {
        heads.g
    } else {
        let horizon = t_l_max_us.checked_add(SKEW_US)?;
        heads.g.max(if horizon < l2_genesis {
            h0
        } else {
            floor(horizon)?
        })
    };
    let h_guard = g_max.max(d_max).max(inbox_max);
    let lower_time_us = t_l_max_us.checked_add(PREP_US)?;
    let upper_time_us = t_l_min_us
        .checked_add(config.t_drift_us)?
        .checked_sub(EPSILON_US)?;
    let lower_height = h_guard.checked_add(1)?.max(ceil(lower_time_us)?);
    let upper_height = floor(upper_time_us);
    let target = upper_height
        .filter(|upper| *upper >= lower_height && lower_time_us <= upper_time_us)
        .map(|_| lower_height);
    Some(Solution {
        t_l_min_us,
        t_l_max_us,
        b_hi,
        e_hi,
        h_time,
        extra_origin,
        g_max,
        d_max,
        inbox_max,
        h_guard,
        lower_time_us,
        upper_time_us,
        lower_height,
        upper_height,
        target,
    })
}

pub(super) fn recovery_reason(
    g: &[Hash32],
    d: &[Hash32],
    halted: bool,
) -> Option<(&'static str, Option<(usize, Hash32, Hash32)>)> {
    if let Some((h, (&g, &d))) = g.iter().zip(d).enumerate().find(|(_, (g, d))| g != d) {
        return Some(("HASH_DIVERGENCE", Some((h, g, d))));
    }
    (halted && d.len() > g.len()).then_some(("HALTED_D_ADVANCED", None))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m1_smoke::profile::PublicInputs;

    fn heads() -> Heads {
        Heads {
            g: 1,
            halted: true,
            d: 6,
            origin: 8,
            inbox: 1,
            pending_batch_through: 1,
        }
    }

    #[test]
    fn verifies_inclusive_time_bounds_and_strict_growing_heads() {
        let config = PublicInputs::fixture().unwrap().config;
        let solve = |h, a, b| super::solve(&config, 10 * SECOND, h, a, b);
        let plan = solve(heads(), 26 * SECOND, 28 * SECOND).unwrap();
        assert_eq!((plan.g_max, plan.inbox_max, plan.d_max), (1, 1, 16));
        assert_eq!(
            (plan.lower_time_us, plan.upper_time_us),
            (40 * SECOND, 40 * SECOND)
        );
        assert_eq!(plan.target, Some(30));
        assert!(plan.accepts_inclusion(26 * SECOND));
        assert!(plan.accepts_inclusion(28 * SECOND));
        assert!(!plan.accepts_inclusion(30 * SECOND));
        assert!(
            solve(
                Heads {
                    d: u64::MAX,
                    ..heads()
                },
                26 * SECOND,
                28 * SECOND
            )
            .is_none()
        );
        assert_eq!(
            solve(Heads { g: 30, ..heads() }, 26 * SECOND, 28 * SECOND)
                .unwrap()
                .target,
            None
        );
        assert_eq!(
            solve(heads(), 26 * SECOND, 30 * SECOND).unwrap().target,
            None
        );
        assert!(solve(heads(), 28 * SECOND, 26 * SECOND).is_none());
        assert!(solve(heads(), u64::MAX - 1, u64::MAX).is_none());
    }

    #[test]
    fn rounding_uses_the_l2_epoch_and_upper_time() {
        let mut config = PublicInputs::fixture().unwrap().config;
        config.genesis.timestamp_us += 250_000;
        let announcement = 10 * SECOND;
        let wide = solve(&config, announcement, heads(), 26 * SECOND, 26 * SECOND).unwrap();
        assert_eq!(
            (wide.lower_height, wide.upper_height, wide.target),
            (28, Some(29), Some(28))
        );
        let singleton = solve(&config, announcement, heads(), 26 * SECOND, 28 * SECOND).unwrap();
        assert_eq!(
            (
                singleton.lower_height,
                singleton.upper_height,
                singleton.target
            ),
            (30, Some(29), None)
        );
    }

    #[test]
    fn different_heads_do_not_imply_conflict() {
        let a = Hash32::new([1; 32]);
        let b = Hash32::new([2; 32]);
        assert_eq!(recovery_reason(&[a, b], &[a], false), None);
        assert_eq!(recovery_reason(&[a], &[a, b], false), None);
        assert_eq!(
            recovery_reason(&[a], &[a, b], true),
            Some(("HALTED_D_ADVANCED", None))
        );
        assert_eq!(
            recovery_reason(&[a, b], &[a, a], false),
            Some(("HASH_DIVERGENCE", Some((1, b, a))))
        );
    }
}
