use poc_core::execution::ExecutionError;
use poc_types::domain::{
    ConfigRef, DomainError, ExecutionAnchor, ExecutionState, Hash32, InputError, InputMissing,
    L1BlockRef, L1Position, NativeBlock, PhysicalBlock, PublicConfig, VirtualBlock,
};
use serde::{Deserialize, Serialize};

pub const PROFILE_VERSION: u16 = 1;
pub const L1_ADAPTER_REVISION: &str = "87daee8e9e77bbe27c337383d873a5eaf4a694ba";

/// Rule configuration. The verified source supplies the separate L1 identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivationConfig {
    public: PublicConfig,
    sequence_window_blocks: u64,
    profile_version: u16,
}

impl DerivationConfig {
    pub fn new(public: PublicConfig, sequence_window_blocks: u64) -> Result<Self, Reject> {
        let config = Self {
            public,
            sequence_window_blocks,
            profile_version: PROFILE_VERSION,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Reject> {
        self.public.validate()?;
        if self.profile_version != PROFILE_VERSION || self.sequence_window_blocks == 0 {
            return Err(Reject::Configuration(
                "invalid version or sequencing window",
            ));
        }
        Ok(())
    }

    pub fn public(&self) -> &PublicConfig {
        &self.public
    }

    pub fn config_ref(&self) -> Result<ConfigRef, Reject> {
        Ok(self.public.reference()?)
    }

    pub fn sequence_window_blocks(&self) -> u64 {
        self.sequence_window_blocks
    }

    pub fn window_end(&self, origin: u64) -> Result<u64, Reject> {
        origin
            .checked_add(self.sequence_window_blocks)
            .ok_or(Reject::Arithmetic("sequencing window end"))
    }

    /// Inclusion at the final block of the sequencing window is timely.
    pub fn inclusion_is_late(&self, origin: u64, inclusion: u64) -> Result<bool, Reject> {
        Ok(inclusion > self.window_end(origin)?)
    }
}

/// Public identity of the source, shared by readers of the same history.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case", deny_unknown_fields)]
pub enum L1Identity {
    Fixture {
        commitment: Hash32,
    },
    Driver {
        chain_id: u64,
        config_ref: ConfigRef,
        inbox: [u8; 20],
        authority: [u8; 20],
        submitter: [u8; 20],
        l1_genesis_hash: Hash32,
        l1_genesis_timestamp_s: u64,
        slot_seconds: u64,
        announcement_seconds: u64,
        finality_delay_us: u64,
        adapter_revision: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivationIdentity {
    pub config_ref: ConfigRef,
    pub sequence_window_blocks: u64,
    pub profile_version: u16,
    pub l1: L1Identity,
}

impl DerivationIdentity {
    pub fn new(config: &DerivationConfig, l1: L1Identity) -> Result<Self, Reject> {
        config.validate()?;
        let config_ref = config.config_ref()?;
        let public = config.public();
        let is_bound = match &l1 {
            L1Identity::Fixture { commitment } => *commitment == public.l1_fixture_hash,
            L1Identity::Driver {
                chain_id,
                config_ref: driver_ref,
                inbox,
                authority,
                submitter,
                l1_genesis_timestamp_s,
                slot_seconds,
                announcement_seconds,
                adapter_revision,
                ..
            } => {
                *chain_id == public.chain_id
                    && *driver_ref == config_ref
                    && inbox != authority
                    && [inbox, authority, submitter].iter().all(|a| **a != [0; 20])
                    && l1_genesis_timestamp_s.checked_mul(1_000_000)
                        == Some(public.l1_genesis_timestamp_us)
                    && slot_seconds.checked_mul(1_000_000) == Some(public.l1_block_us)
                    && *announcement_seconds > 0
                    && adapter_revision == L1_ADAPTER_REVISION
            }
        };
        if !is_bound {
            return Err(Reject::Configuration(
                "L1 identity and public configuration disagree",
            ));
        }
        Ok(Self {
            config_ref,
            sequence_window_blocks: config.sequence_window_blocks,
            profile_version: config.profile_version,
            l1,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DerivationCause {
    AcceptedBatch {
        position: L1Position,
        commitment: Hash32,
        record_index: u16,
        substituted: bool,
    },
    Fallback {
        window_end: u64,
        frontier: L1BlockRef,
    },
    CursorSet {
        position: L1Position,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedItem {
    pub virtual_block: VirtualBlock,
    pub physical_block: PhysicalBlock,
    pub native_block: NativeBlock,
    pub native_hash: Hash32,
    pub state: ExecutionState,
    pub cause: DerivationCause,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeEvidence {
    pub derivation: DerivationIdentity,
    pub including_block: L1BlockRef,
    pub transaction_hash: Hash32,
    pub request_id: Hash32,
    pub sequence: u64,
    pub cursor_set_rpc_log_index: u64,
    pub resume_rpc_log_index: u64,
}

/// Snapshot immediately after the paired CursorSet. D may already be beyond target.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeCut {
    pub resume_position: L1Position,
    pub cursor_set_position: L1Position,
    pub target_height: u64,
    pub genesis: ExecutionAnchor,
    pub derived_prefix: Vec<DerivedItem>,
    pub target_anchor: ExecutionAnchor,
    pub evidence: ResumeEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "at", rename_all = "snake_case")]
pub enum InputPoint {
    BlockStart(u64),
    Log(L1Position),
    BlockEnd(u64),
    Fallback { frontier: u64, height: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaitReason {
    L1Header(u64),
    CompleteBlock(u64),
    CompleteWindow { through: u64 },
    ResumePair(L1Position),
    ResumeCoverage(L1Position),
    Feature11AcceptedPayloadRequired { commitment: Hash32 },
    Deposit(InputMissing),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wait {
    pub position: InputPoint,
    pub reason: WaitReason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reject {
    Configuration(&'static str),
    Arithmetic(&'static str),
    Source(&'static str),
    NonResume(L1Position),
    CheckpointMismatch,
    Domain(DomainError),
    Input(InputError),
    Execution(ExecutionError),
}

impl From<DomainError> for Reject {
    fn from(error: DomainError) -> Self {
        Self::Domain(error)
    }
}

impl From<InputError> for Reject {
    fn from(error: InputError) -> Self {
        Self::Input(error)
    }
}

impl From<ExecutionError> for Reject {
    fn from(error: ExecutionError) -> Self {
        Self::Execution(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome<T> {
    Ready(T),
    Wait(Wait),
    Reject(Reject),
}
