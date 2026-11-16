use std::fmt;

use poc_core::{fixture::DepProvider, merge::MergedBlock, metadata::MetadataProvider};
use poc_types::domain::{Boundary, DomainError, FinalizedL1Reader, PublicReferenceView};
use sim_espresso::{EspressoPosition, FinalizedLogReader, SourceError};

use crate::{
    Anchor, Descriptor, ExecutionAnchor, FinalVerifier, Hash32, InputError, InputMissing,
    InputStatus, L1Position, LifecycleEvent, PhysicalBlock, PlanInput, PlanRecord, PublicConfig,
    ReplayKind, ReplayPosition, ResultKey, SessionId,
};

pub use poc_authorization::{ActivePolicy, HistoricalPlan, PlanError, PlanOutcome, RegionalPolicy, StaticScheduleProvider};

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
    fn resolve_historical(&self, _: &PlanInput, _: &dyn PublicReferenceView, _: &HistoricalPlan) -> PlanOutcome {
        PlanOutcome::Reject(PlanError::PlanMismatch)
    }
    fn descriptor(&self, plan: &PlanRecord, anchor: &Anchor) -> Result<Descriptor, PlanError>;
}

impl PlanProvider for StaticScheduleProvider {
    fn resolve(&self, input: &PlanInput, references: &dyn PublicReferenceView) -> PlanOutcome {
        StaticScheduleProvider::resolve(self, input, references)
    }

    fn resolve_historical(&self, input: &PlanInput, references: &dyn PublicReferenceView, basis: &HistoricalPlan) -> PlanOutcome {
        StaticScheduleProvider::resolve_historical(self, input, references, basis)
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
    /// The exact immutable cut read from the verified derivation associated with this L1 reader.
    fn resume_cut(&self, position: L1Position) -> InputStatus<ResumeBasis> {
        InputStatus::Wait(InputMissing::ControlCoverage(position.block))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResumeBasis {
    pub position: L1Position,
    pub set_position: L1Position,
    pub config_ref: crate::ConfigRef,
    pub including_block: poc_types::domain::L1BlockRef,
    pub transaction_hash: Hash32,
    pub digest: Hash32,
    pub genesis: ExecutionAnchor,
    pub target: ExecutionAnchor,
    /// Contiguous derived rows from the child of genesis through the target inclusive.
    pub blocks: Vec<MergedBlock>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResumeControl {
    pub target_height: u64,
    pub signed_origin: u64,
    pub policy: RegionalPolicy,
    pub set_log: u64,
    pub transaction_hash: Hash32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlRef {
    pub position: L1Position,
    pub kind: ReplayKind,
    pub control_seq: u64,
    pub source_bytes: Vec<u8>,
}

impl ControlRef {
    const POLICY_RESUME: &'static [u8] = b"PoC/PolicyResume/v1";
    const POLICY_FORCE: &'static [u8] = b"PoC/PolicyForce/v1";
    const STATIC_FORCE: &'static [u8] = b"PoC/StaticForce/v1";

    /// Project a Force from an authenticated, finalized L1 log. The source
    /// adapter must validate the Authority and the fixed static policy first.
    pub fn static_force(position: L1Position, control_seq: u64, branch: u64) -> Self {
        let mut source_bytes = Self::STATIC_FORCE.to_vec();
        source_bytes.extend_from_slice(&branch.to_be_bytes());
        Self { position, kind: ReplayKind::Force, control_seq, source_bytes }
    }

    pub fn policy_force(position: L1Position, control_seq: u64, branch: u64, policy: RegionalPolicy) -> Self {
        let mut source_bytes = Self::POLICY_FORCE.to_vec();
        source_bytes.extend_from_slice(&branch.to_be_bytes());
        source_bytes.extend_from_slice(&policy.mask_and_order());
        source_bytes.extend_from_slice(&policy.signed_floor.to_be_bytes());
        Self { position, kind: ReplayKind::Force, control_seq, source_bytes }
    }

    pub fn resume(position: L1Position, control_seq: u64, resume: ResumeControl) -> Self {
        let mut source_bytes = Self::POLICY_RESUME.to_vec();
        source_bytes.extend_from_slice(&resume.target_height.to_be_bytes());
        source_bytes.extend_from_slice(&resume.signed_origin.to_be_bytes());
        source_bytes.extend_from_slice(&resume.policy.mask_and_order());
        source_bytes.extend_from_slice(&resume.policy.signed_floor.to_be_bytes());
        source_bytes.extend_from_slice(&resume.set_log.to_be_bytes());
        source_bytes.extend_from_slice(resume.transaction_hash.as_bytes());
        Self {position, kind: ReplayKind::Resume, control_seq, source_bytes}
    }
    pub fn is_policy_resume(&self) -> bool { self.source_bytes.starts_with(Self::POLICY_RESUME) }
    pub fn resume_control(&self) -> Option<ResumeControl> {
        let bytes = self.source_bytes.strip_prefix(Self::POLICY_RESUME)?;
        if self.kind != ReplayKind::Resume || bytes.len()!=68 {return None;}
        let number=|offset| Some(u64::from_be_bytes(bytes.get(offset..offset+8)?.try_into().ok()?));
        let set_log=number(28)?;
        let signed_origin=number(8)?;
        if set_log >= self.position.log || signed_origin > self.position.block {return None;}
        Some(ResumeControl {
            target_height: number(0)?, signed_origin, policy: RegionalPolicy::from_parts(
                bytes[16..20].try_into().ok()?, number(20)?).ok()?,
            set_log, transaction_hash: Hash32::new(bytes[36..].try_into().ok()?),
        })
    }

    pub fn is_policy_force(&self) -> bool {
        self.source_bytes.starts_with(Self::POLICY_FORCE)
    }

    pub fn regional_policy(&self) -> Option<RegionalPolicy> {
        let bytes = self.source_bytes.strip_prefix(Self::POLICY_FORCE)?;
        if self.kind != ReplayKind::Force || bytes.len() != 20 {
            return None;
        }
        RegionalPolicy::from_parts(
            bytes[8..12].try_into().ok()?,
            u64::from_be_bytes(bytes[12..].try_into().ok()?),
        ).ok()
    }

    pub fn force_branch(&self) -> Option<u64> {
        if self.kind != ReplayKind::Force {
            return None;
        }
        if self.is_policy_force() {
            self.regional_policy()?;
            let bytes = self.source_bytes.strip_prefix(Self::POLICY_FORCE)?;
            return Some(u64::from_be_bytes(bytes[..8].try_into().ok()?));
        }
        let branch = self.source_bytes.strip_prefix(Self::STATIC_FORCE)?;
        Some(u64::from_be_bytes(branch.try_into().ok()?))
    }
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
    Derived { resume_position: L1Position, height: u64 },
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
        cut_digest: Hash32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayRecord {
    pub position: ReplayPosition,
    pub input_ref: InputRef,
    pub effects: Vec<ReplayEffect>,
    pub events: Vec<LifecycleEvent>,
}
