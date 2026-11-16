use std::fmt;

use poc_core::{fixture::DepProvider, merge::MergedBlock, metadata::MetadataProvider};
use poc_types::domain::{Boundary, DomainError, FinalizedL1Reader, PublicReferenceView};
use sim_espresso::{EspressoPosition, FinalizedLogReader, SourceError};

use crate::{
    Anchor, Descriptor, ExecutionAnchor, FinalVerifier, Hash32, InputError, InputMissing,
    InputStatus, L1Position, LifecycleEvent, PhysicalBlock, PlanInput, PlanRecord, PublicConfig,
    ReplayKind, ReplayPosition, ResultKey, SessionId,
};

pub use poc_authorization::{PlanError, PlanOutcome, StaticScheduleProvider};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GfrParameters {
    pub announcement_us: u64,
    pub grace_us: u64,
    pub expiry_us: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GfrError {
    Domain(DomainError),
    InvalidParameters,
    GenesisMismatch,
    ConfigMismatch,
    VerificationProfileMismatch,
    UnsupportedVerificationProfile,
    InvalidSource,
    EspressoSource(SourceError),
    Input(InputError),
    Plan(PlanError),
    ConflictingControl(L1Position),
    InconsistentFinality(ResultKey),
    CheckpointMismatch,
    HistoricalBound,
}

impl From<DomainError> for GfrError {
    fn from(error: DomainError) -> Self {
        Self::Domain(error)
    }
}

impl From<InputError> for GfrError {
    fn from(error: InputError) -> Self {
        Self::Input(error)
    }
}

impl From<PlanError> for GfrError {
    fn from(error: PlanError) -> Self {
        Self::Plan(error)
    }
}

impl fmt::Display for GfrError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for GfrError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Missing {
    EspressoHeader {
        q: u64,
    },
    EspressoMessages {
        q: u64,
        indices: Vec<u32>,
    },
    EspressoWatermark {
        key_us: u64,
    },
    L1Header {
        height: u64,
    },
    L1ControlCoverage {
        through_height: u64,
    },
    PublicObject {
        position: EspressoPosition,
        hash: Hash32,
    },
    Input(InputMissing),
    FinalVerifierUnavailable,
    ResumeCut {
        position: L1Position,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayStatus {
    Ready,
    Wait {
        next_position: Option<ReplayPosition>,
        missing: Vec<Missing>,
    },
    UnsupportedControl {
        position: ReplayPosition,
        kind: ReplayKind,
    },
    Reject(GfrError),
}

pub struct ReplayReads<'a> {
    pub espresso: &'a dyn FinalizedLogReader,
    pub l1: &'a dyn L1ControlReader,
}

pub struct ReplayServices<'a> {
    pub config: &'a PublicConfig,
    pub plan: &'a dyn PlanProvider,
    pub metadata: &'a dyn MetadataProvider,
    pub dep: &'a dyn DepProvider,
    pub finality: Option<VerifierBinding<'a>>,
}

pub trait PlanProvider {
    fn resolve(&self, input: &PlanInput, references: &dyn PublicReferenceView) -> PlanOutcome;
    fn descriptor(&self, plan: &PlanRecord, anchor: &Anchor) -> Result<Descriptor, PlanError>;
}

impl PlanProvider for StaticScheduleProvider {
    fn resolve(&self, input: &PlanInput, references: &dyn PublicReferenceView) -> PlanOutcome {
        StaticScheduleProvider::resolve(self, input, references)
    }

    fn descriptor(&self, plan: &PlanRecord, anchor: &Anchor) -> Result<Descriptor, PlanError> {
        StaticScheduleProvider::descriptor(self, plan, anchor)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerificationProfile {
    NonCryptographicFixture {
        id: String,
    },
    Native {
        protocol_version: u16,
        verifier_version: u16,
        source_revision: String,
    },
}

impl VerificationProfile {
    pub fn validate(&self, config: &PublicConfig) -> Result<(), GfrError> {
        match self {
            Self::NonCryptographicFixture { id } if !id.is_empty() => Ok(()),
            Self::Native {
                protocol_version,
                verifier_version,
                source_revision,
            } if *protocol_version == config.rule_version
                && *verifier_version == config.verifier_version
                && !source_revision.is_empty() =>
            {
                Ok(())
            }
            _ => Err(GfrError::UnsupportedVerificationProfile),
        }
    }
}

pub trait VerificationBasis {
    fn require(&self, descriptor: &Descriptor, block: &PhysicalBlock) -> InputStatus<()>;
}

pub struct VerifierBinding<'a> {
    pub verifier: &'a dyn FinalVerifier,
    pub basis: &'a dyn VerificationBasis,
    pub profile: VerificationProfile,
}

pub trait L1ControlReader: FinalizedL1Reader {
    fn source_id(&self) -> &[u8];
    fn controls_at(&self, height: u64) -> InputStatus<Vec<ControlRef>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlRef {
    pub position: L1Position,
    pub kind: ReplayKind,
    pub control_seq: u64,
    pub source_bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tail {
    Next(Descriptor),
    Pending(Boundary),
    Halt { boundary: Boundary, anchor: Anchor },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GlobalSource {
    Regional(ResultKey),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainEntry {
    pub block: MergedBlock,
    pub source: GlobalSource,
    pub position: ReplayPosition,
}

pub struct ChainView<'a> {
    pub genesis: &'a ExecutionAnchor,
    /// Contiguous post-genesis entries in ascending height order.
    pub entries: &'a [ChainEntry],
    pub pc: Option<&'a ReplayPosition>,
}

pub struct ResponsibilityView<'a> {
    pub tail: &'a Tail,
    pub branch: u64,
    pub pc: Option<&'a ReplayPosition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputRef {
    L1(ControlRef),
    Expire(u64),
    SessionEnd(SessionId),
    Espresso(u64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayEffect {
    Append {
        first: u64,
        last: u64,
        source: GlobalSource,
    },
    Duty {
        before: Tail,
        after: Tail,
    },
    Reset {
        resume_position: L1Position,
        target_height: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayRecord {
    pub position: ReplayPosition,
    pub input_ref: InputRef,
    pub effects: Vec<ReplayEffect>,
    pub events: Vec<LifecycleEvent>,
}
