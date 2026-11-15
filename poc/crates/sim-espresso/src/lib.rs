//! Deterministic Espresso log and independently delivered reader prefixes.
#![forbid(unsafe_code)]

mod reader;
mod wire;

pub use reader::{
    BlockHeader, DeliveryError, FinalizedLogReader, MissingLog, ReadError, ReaderReceipt,
    ReaderState, SourceError, VerifiedPrefix,
};
pub use wire::{WireError, decode_finalized, encode_finalized};

pub const MAX_SOURCE_BYTES: usize = 256;
pub const MAX_MESSAGE_BYTES: usize = 65_536;
pub const MAX_MESSAGES: usize = 1_024;
pub const MAX_BLOCK_BYTES: usize = 1_048_576;

/// A finite component run retains its history up to these bounds.
pub const MAX_BLOCKS: usize = 4_096;
pub const MAX_LOG_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct TickUs(pub u64);

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct PublicTimeUs(pub u64);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EspressoPosition {
    pub q: u64,
    pub message_index: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogMessage {
    pub codec: u32,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposedBlock {
    pub q: u64,
    pub theta_us: PublicTimeUs,
    pub proposed_at_us: TickUs,
    pub live_at_proposal: bool,
    pub messages: Vec<LogMessage>,
}

impl ProposedBlock {
    pub fn assess_timestamp(&self, delta_ts_us: u64) -> TimestampAssessment {
        if !self.live_at_proposal {
            return TimestampAssessment::NotAssumedNonLive;
        }
        let deviation_us = self.theta_us.0.abs_diff(self.proposed_at_us.0);
        if deviation_us <= delta_ts_us {
            TimestampAssessment::WithinBound
        } else {
            TimestampAssessment::OutsideBound {
                deviation_us,
                bound_us: delta_ts_us,
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinalizedBlock {
    pub proposal: ProposedBlock,
    pub finalized_at_us: TickUs,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimestampPolicy {
    Offset { offset_us: i64, quantum_us: u64 },
    Fixed { theta_us: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimestampAssessment {
    WithinBound,
    OutsideBound { deviation_us: u64, bound_us: u64 },
    NotAssumedNonLive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimestampError {
    InvalidQuantum,
    ArithmeticRange,
    Regressed,
}

impl TimestampPolicy {
    pub fn theta(
        self,
        proposed_at: TickUs,
        previous: Option<PublicTimeUs>,
    ) -> Result<PublicTimeUs, TimestampError> {
        let theta = PublicTimeUs(match self {
            Self::Fixed { theta_us } => theta_us,
            Self::Offset {
                offset_us,
                quantum_us,
            } => {
                if quantum_us == 0 {
                    return Err(TimestampError::InvalidQuantum);
                }
                let base = u64::try_from(i128::from(proposed_at.0) + i128::from(offset_us))
                    .map_err(|_| TimestampError::ArithmeticRange)?;
                (base / quantum_us) * quantum_us
            }
        });
        if previous.is_some_and(|last| theta < last) {
            return Err(TimestampError::Regressed);
        }
        Ok(theta)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LogError {
    InvalidStructure(WireError),
    Timestamp(TimestampError),
    TickRegressed,
    UnknownProposal { q: u64 },
    FinalizeOutOfOrder { next_q: u64 },
    Capacity,
}

/// In-memory producer driven exclusively by explicit proposal and finalize inputs.
pub struct SimEspresso {
    source_id: Vec<u8>,
    policy: TimestampPolicy,
    proposed: std::collections::VecDeque<ProposedBlock>,
    finalized: Vec<FinalizedBlock>,
    last_tick: Option<TickUs>,
    retained_bytes: usize,
}

impl SimEspresso {
    pub fn new(source_id: Vec<u8>, policy: TimestampPolicy) -> Result<Self, LogError> {
        wire::validate_source(&source_id).map_err(LogError::InvalidStructure)?;
        Ok(Self {
            source_id,
            policy,
            proposed: Default::default(),
            finalized: Vec::new(),
            last_tick: None,
            retained_bytes: 0,
        })
    }

    pub fn source_id(&self) -> &[u8] {
        &self.source_id
    }

    pub fn set_policy(&mut self, policy: TimestampPolicy) {
        self.policy = policy;
    }

    pub fn propose(
        &mut self,
        tick: TickUs,
        live_at_proposal: bool,
        messages: Vec<LogMessage>,
    ) -> Result<&ProposedBlock, LogError> {
        self.check_tick(tick)?;
        let q = self.finalized.len() + self.proposed.len();
        let bytes =
            wire::block_len(&self.source_id, &messages).map_err(LogError::InvalidStructure)?;
        if q >= MAX_BLOCKS || bytes > MAX_LOG_BYTES - self.retained_bytes {
            return Err(LogError::Capacity);
        }
        let previous = self
            .proposed
            .back()
            .or_else(|| self.finalized.last().map(|block| &block.proposal));
        let theta_us = self
            .policy
            .theta(tick, previous.map(|block| block.theta_us))
            .map_err(LogError::Timestamp)?;
        self.proposed.push_back(ProposedBlock {
            q: q as u64,
            theta_us,
            proposed_at_us: tick,
            live_at_proposal,
            messages,
        });
        self.retained_bytes += bytes;
        self.last_tick = Some(tick);
        Ok(self.proposed.back().expect("just inserted"))
    }

    pub fn finalize(&mut self, q: u64, tick: TickUs) -> Result<&FinalizedBlock, LogError> {
        let next_q = self.finalized.len() as u64;
        if q < next_q {
            return Ok(&self.finalized[q as usize]);
        }
        if q >= next_q + self.proposed.len() as u64 {
            return Err(LogError::UnknownProposal { q });
        }
        if q != next_q {
            return Err(LogError::FinalizeOutOfOrder { next_q });
        }
        self.check_tick(tick)?;
        let proposal = self.proposed.pop_front().expect("known proposal");
        self.finalized.push(FinalizedBlock {
            proposal,
            finalized_at_us: tick,
        });
        self.last_tick = Some(tick);
        Ok(self.finalized.last().expect("just inserted"))
    }

    /// Producer-side lookup. Consumers query their own delivered reader instead.
    pub fn finalized(&self, q: u64) -> Option<&FinalizedBlock> {
        usize::try_from(q).ok().and_then(|q| self.finalized.get(q))
    }

    fn check_tick(&self, tick: TickUs) -> Result<(), LogError> {
        if self.last_tick.is_some_and(|last| tick < last) {
            Err(LogError::TickRegressed)
        } else {
            Ok(())
        }
    }
}
