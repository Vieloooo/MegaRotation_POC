//! Typed coordination between the healthy session runtime, Manager, and publication bridge.
//!
//! Shape checks establish structural consistency. Issuers and consumers establish
//! provenance, execution, durability, and finality through their own ports.

mod shape;

pub use shape::{derive_local_id, validate_native_binding};

pub use poc_types::domain::{
    Anchor, AnchorEvidence, Canonical, CanonicalObject, Cid, CloseReason, ConfigRef, Descriptor,
    DomainError, ExecutionAnchor, FinalVerifier, Hash32, InputMissing, InputStatus, PhysicalBlock,
    Pid, PublicConfig, RegionId, RegionalResult, ReplayPosition, ResultKey, SessionId,
    VirtualBlock,
};
pub use poc_viewer_gfr::{
    GfrError, ReplayRecord, VerificationBasis, VerificationProfile, VerifierBinding,
    encode_position, lifecycle_id, lifecycle_projection,
};

pub const API_VERSION: u16 = 1;
pub const MAX_OPAQUE_BYTES: usize = 65_536;
pub const MAX_ACTIONS: usize = 128;
pub const MAX_GUARDS: usize = 128;
pub const MAX_REFS: usize = 128;
pub const MAX_PREP_BLOCKS: usize = 128;
pub const MAX_REPLAY_EVENTS: usize = 128;
pub const MAX_REPLAY_EFFECTS: usize = 128;

/// Opaque local control identity with a nonzero hash.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalId(Hash32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum IdKind {
    Operation = 1,
    Program = 2,
    Preparation = 3,
    Ready = 4,
    Action = 5,
    ManagerDelta = 6,
    ReplayBatch = 7,
    Delivery = 8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkerScope {
    pub run: Hash32,
    pub config: ConfigRef,
    pub worker: Pid,
    pub incarnation: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvidenceRef {
    pub source: Hash32,
    pub object: Hash32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootRef {
    pub sequence: u64,
    pub hash: Hash32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GrantRef {
    pub scope: WorkerScope,
    pub sequence: u64,
    pub hash: Hash32,
    pub tick_us: u64,
}
/// Reference attached after the producing root commits the effect payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlledOrigin {
    pub grant: GrantRef,
    pub root: RootRef,
    pub effect_index: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramHandle {
    pub scope: WorkerScope,
    pub cid: Cid,
    pub instance_id: LocalId,
    pub generation: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparationJob {
    pub program: ProgramHandle,
    pub id: LocalId,
    pub generation: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateMaterial {
    pub checkpoint: Option<ExecutionAnchor>,
    pub physical_prefix: Vec<PhysicalBlock>,
    pub virtual_prefix: Vec<VirtualBlock>,
    pub input_refs: Vec<EvidenceRef>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparationRequest {
    pub generation: u64,
    pub anchor: Anchor,
    pub anchor_evidence: AnchorEvidence,
    pub material: StateMaterial,
    pub provenance: Vec<EvidenceRef>,
    pub validation_profile: Hash32,
}
/// Claim of readiness whose issuance and current validity are checked by the runtime.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadyToken {
    pub id: LocalId,
    pub job: PreparationJob,
    pub anchor: Anchor,
    pub state_root: Hash32,
    pub native_hash: Hash32,
    pub validation_profile: Hash32,
    pub execution_basis: EvidenceRef,
    pub origin: ControlledOrigin,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadyState {
    pub token: ReadyToken,
    pub execution_anchor: ExecutionAnchor,
    pub ordered_input_refs: Vec<EvidenceRef>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProgramPhase {
    Assigned,
    Preparing,
    Ready,
    Running,
    Paused,
    Terminated,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramObservation {
    pub program: ProgramHandle,
    pub phase: ProgramPhase,
    pub original_entry_us: Option<i64>,
    pub view: Option<u64>,
    pub terminal: Option<CloseCause>,
    pub history: Option<EvidenceRef>,
    pub timers: Option<EvidenceRef>,
    pub pending_work: Option<EvidenceRef>,
    pub origin: ControlledOrigin,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CancelCause {
    Superseded,
    Explicit,
    Closed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloseCause {
    Program(CloseReason),
    ClockUnsafe { evidence: EvidenceRef },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloseScope {
    Cid(Cid),
    Session(SessionId),
    Branch(u64),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProgramCommand {
    Create {
        descriptor: Descriptor,
        plan_basis: Vec<EvidenceRef>,
    },
    Prepare {
        program: ProgramHandle,
        request: PreparationRequest,
    },
    Supply {
        job: PreparationJob,
        material: StateMaterial,
        provenance: Vec<EvidenceRef>,
    },
    CancelPrepare {
        job: PreparationJob,
        cause: CancelCause,
    },
    Enter {
        program: ProgramHandle,
        ready: ReadyToken,
    },
    Pause {
        program: ProgramHandle,
        decision: EvidenceRef,
    },
    SelectBack {
        program: ProgramHandle,
        decision: EvidenceRef,
    },
    Close {
        scope: CloseScope,
        cause: CloseCause,
        basis: Vec<EvidenceRef>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayCursor {
    pub source: Hash32,
    pub sequence: u64,
    pub batch_id: LocalId,
    pub position: ReplayPosition,
}
/// Complete replay record with a sequence-based identity for its source stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayBatch {
    pub id: LocalId,
    pub source: Hash32,
    pub sequence: u64,
    pub previous: Option<LocalId>,
    pub source_record: EvidenceRef,
    pub record: ReplayRecord,
    pub lifecycle_ids: Vec<Hash32>,
    pub origin: ControlledOrigin,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManagerCause {
    Replay(ReplayBatch),
    Regional {
        key: ResultKey,
        evidence: EvidenceRef,
    },
    Local {
        operation: LocalId,
        input: EvidenceRef,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpaqueManagerDelta {
    pub codec: u32,
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuardIntent {
    Terminate {
        scope: CloseScope,
        cause: CloseCause,
        basis: Vec<EvidenceRef>,
    },
    Schedule {
        session: SessionId,
        running: Option<ProgramHandle>,
    },
    BlockNewEnter {
        clock_evidence: EvidenceRef,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionBody {
    Program(ProgramCommand),
    OfferRegional(RegionalDelivery),
    ForwardRegional {
        delivery: RegionalDelivery,
        to: RegionId,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionIntent {
    pub id: LocalId,
    pub body: ActionBody,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagerDelta {
    pub id: LocalId,
    pub scope: WorkerScope,
    pub base_revision: u64,
    pub next_revision: u64,
    pub cause: ManagerCause,
    pub opaque: OpaqueManagerDelta,
    pub guards: Vec<GuardIntent>,
    pub actions: Vec<ActionIntent>,
}
/// Acceptance of a Manager revision and its action obligations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootCommitted {
    pub scope: WorkerScope,
    pub delta_id: LocalId,
    pub manager_revision: u64,
    pub root: RootRef,
    pub replay_cursor: Option<ReplayCursor>,
    pub actions_pending: Vec<LocalId>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionExecution {
    pub committed: RootCommitted,
    pub action_id: LocalId,
    pub grant: GrantRef,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProgramReply {
    Assigned(ProgramHandle),
    Preparing {
        job: PreparationJob,
        missing: Vec<WaitReason>,
    },
    Ready(ReadyState),
    Entered {
        program: ProgramHandle,
        original_entry_us: i64,
    },
    Paused(ProgramHandle),
    Running(ProgramHandle),
    Terminated {
        scope: CloseScope,
        cause: CloseCause,
        durable_guard: EvidenceRef,
    },
    PrepareCancelled(PreparationJob),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionReply {
    Program(ProgramReply),
    PublicationAccepted {
        key: ResultKey,
        task: EvidenceRef,
    },
    ForwardAccepted {
        key: ResultKey,
        custody: EvidenceRef,
    },
    SuppressedByGuard {
        guard: EvidenceRef,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppliedReceipt {
    pub delta_id: LocalId,
    pub action_id: LocalId,
    pub committed_root: RootRef,
    pub origin: ControlledOrigin,
    pub reply: ActionReply,
}
/// Completion of all obligations in the original commit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionsApplied {
    pub committed: RootCommitted,
    pub receipts: Vec<AppliedReceipt>,
    pub completion_origin: ControlledOrigin,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FailureDisposition {
    GuardsRetained,
    CommittedPending,
    RuntimeStopped,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailClosed {
    pub scope: WorkerScope,
    pub delta_id: LocalId,
    pub reason: RejectReason,
    pub disposition: FailureDisposition,
    pub committed: Option<RootCommitted>,
    pub durable_guards: Vec<EvidenceRef>,
    pub actions_pending: Vec<LocalId>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommitOutcome {
    Rejected(RejectReason),
    Committed(RootCommitted),
    FailClosed(FailClosed),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionProgress {
    Applied(AppliedReceipt),
    Pending {
        action_id: LocalId,
        missing: Vec<WaitReason>,
    },
    FailClosed(FailClosed),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompletionOutcome {
    Pending(RootCommitted),
    Applied(ActionsApplied),
    FailClosed(FailClosed),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProofCheck {
    CertificateSubject {
        basis: Vec<EvidenceRef>,
    },
    TargetBound {
        block: PhysicalBlock,
        basis: Vec<EvidenceRef>,
    },
}
/// Observation of a native finalization with subject or target-body evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofObserved {
    pub key: ResultKey,
    pub descriptor: Descriptor,
    pub native_finalization: Vec<u8>,
    pub check: ProofCheck,
    pub native_profile: VerificationProfile,
    pub origin: ControlledOrigin,
}
/// Complete regional result for consumer validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionalDelivery {
    pub id: LocalId,
    pub result: RegionalResult,
    pub validation_basis: Vec<EvidenceRef>,
    pub native_profile: VerificationProfile,
    pub origin: ControlledOrigin,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaitReason {
    Input(InputMissing),
    Evidence(EvidenceRef),
    Body(Hash32),
    Finality(ResultKey),
    NativeVerifierBasis,
    NotBefore { local_deadline_us: i64 },
    Work(PreparationJob),
    Action(LocalId),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApiError {
    Domain(DomainError),
    Gfr(GfrError),
    InvalidField(&'static str),
    BindingMismatch(&'static str),
    LimitExceeded {
        field: &'static str,
        actual: usize,
        max: usize,
    },
    DuplicateId(LocalId),
    Overflow,
    NonNativeProfile,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RejectReason {
    Shape(ApiError),
    InvalidSource,
    InvalidProof,
    InvalidBody,
    InvalidAncestry,
    InvalidState,
    UnsupportedProfile,
    Conflict,
    StaleRevision,
    Terminated,
    ClockUnsafe,
    Capacity,
    PersistenceFailure,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PortResult<T> {
    Ready(T),
    Wait(Vec<WaitReason>),
    Reject(RejectReason),
}

pub trait ManagerCommitPort {
    fn commit(&mut self, delta: &ManagerDelta, grant: &GrantRef) -> CommitOutcome;
    fn complete(
        &mut self,
        committed: &RootCommitted,
        receipts: &[AppliedReceipt],
        grant: &GrantRef,
    ) -> CompletionOutcome;
}
pub trait ProgramPort {
    fn execute(&mut self, execution: &ActionExecution, command: &ProgramCommand) -> ActionProgress;
    fn observe(&self, program: &ProgramHandle) -> PortResult<ProgramObservation>;
    fn preparation(&self, job: &PreparationJob) -> PortResult<ReadyState>;
}
pub trait RegionalResultPort {
    fn offer(&mut self, execution: &ActionExecution, delivery: &RegionalDelivery)
    -> ActionProgress;
}
pub trait ReplayFeedPort {
    fn next(&self, after: Option<&ReplayCursor>) -> PortResult<ReplayBatch>;
}
pub trait NativeVerifierPort {
    fn binding(&self) -> PortResult<VerifierBinding<'_>>;
}
