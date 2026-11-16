use std::fmt;

use poc_types::domain::{
    Anchor, Cid, CloseReason, ConfigRef, Descriptor, Hash32, Pid, ReplayPosition, ResultKey,
    SessionId,
};
use poc_viewer_gfr::ReplayRecord;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagerBinding {
    pub manager_id: Hash32,
    pub worker: Pid,
    pub public_config: ConfigRef,
    pub gfr_source: Hash32,
    pub preparation_version: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedGenesis {
    pub descriptor: Descriptor,
    pub source_ref: Hash32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedCandidate {
    pub descriptor: Descriptor,
    pub predecessor: Cid,
    pub provenance_ref: Hash32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedReplayInput {
    pub source: Hash32,
    pub seq: u64,
    pub record: ReplayRecord,
    pub validation_ref: Hash32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActionId {
    pub manager_id: Hash32,
    pub worker: Pid,
    pub origin_revision: u64,
    pub ordinal: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedRef {
    pub prepare_action: ActionId,
    pub worker: Pid,
    pub cid: Cid,
    pub instance_ref: Hash32,
    pub anchor: Anchor,
    pub config_ref: ConfigRef,
    pub validation_version: u16,
    pub execution_ref: Hash32,
    pub provenance_ref: Hash32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManagerInput {
    RegisterCandidate(VerifiedCandidate),
    ConsumeRecord(VerifiedReplayInput),
    PreparationCompleted(PreparedRef),
    Receipt(ActionReceipt),
    Schedule(SessionId),
    ObserveTime,
    ClockReady {
        generation: u64,
        basis_ref: Hash32,
    },
    ClockUnsafe {
        basis_ref: Hash32,
    },
    RetainResult {
        key: ResultKey,
        verified_result_ref: Hash32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionCause {
    pub input: ManagerInput,
    pub local_time_us: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalCause {
    Public(CloseReason),
    LocalTime,
    LocalClockUnsafe {
        generation: Option<u64>,
        basis_ref: Hash32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionKind {
    Create {
        descriptor: Descriptor,
        provenance_ref: Hash32,
    },
    Prepare {
        cid: Cid,
        instance_ref: Hash32,
        anchor: Anchor,
        provenance_ref: Hash32,
        validation_version: u16,
    },
    Enter {
        cid: Cid,
        instance_ref: Hash32,
        prepared_ref: PreparedRef,
    },
    Pause {
        cid: Cid,
        instance_ref: Hash32,
    },
    SelectBack {
        cid: Cid,
        instance_ref: Hash32,
    },
    Close {
        cid: Cid,
        instance_ref: Option<Hash32>,
        cause: TerminalCause,
    },
    Publish {
        key: ResultKey,
        verified_result_ref: Hash32,
    },
}

impl ActionKind {
    pub(crate) fn program(&self) -> Option<Cid> {
        match self {
            Self::Create { descriptor, .. } => Some(descriptor.cid),
            Self::Prepare { cid, .. }
            | Self::Enter { cid, .. }
            | Self::Pause { cid, .. }
            | Self::SelectBack { cid, .. }
            | Self::Close { cid, .. } => Some(*cid),
            Self::Publish { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionIntent {
    pub id: ActionId,
    pub kind: ActionKind,
    pub dependencies: Vec<ActionId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionReceipt {
    pub id: ActionId,
    pub receipt_ref: Hash32,
    pub outcome: ReceiptOutcome,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReceiptOutcome {
    Created {
        instance_ref: Hash32,
    },
    Entered {
        instance_ref: Hash32,
        entry_time_us: u64,
    },
    Paused {
        instance_ref: Hash32,
    },
    SelectedBack {
        instance_ref: Hash32,
    },
    Closed {
        instance_ref: Option<Hash32>,
    },
    Published {
        key: ResultKey,
    },
    Retryable {
        failure_ref: Hash32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionStatus {
    Pending,
    Applied,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionEntry {
    pub intent: ActionIntent,
    pub status: ActionStatus,
    pub receipt: Option<ActionReceipt>,
    pub preparation: Option<PreparedRef>,
    pub last_retry: Option<ActionReceipt>,
    pub issued_at_us: u64,
    pub(crate) followup: Option<ActionId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    Candidate,
    Selected,
    Excluded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Assigned,
    Preparing,
    Ready,
    Entering,
    Running,
    Pausing,
    Paused,
    Resuming,
    Terminated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub descriptor: Descriptor,
    pub provenance_ref: Hash32,
    pub predecessor: Option<Cid>,
    pub stop_us: u64,
    pub selection: Selection,
    pub phase: Phase,
    pub instance_ref: Option<Hash32>,
    pub prepared: Option<PreparedRef>,
    pub entered: bool,
    pub entry_time_us: Option<u64>,
    pub terminal_cause: Option<TerminalCause>,
    pub create_action: Option<ActionId>,
    pub prepare_action: Option<ActionId>,
    pub enter_action: Option<ActionId>,
    pub pause_action: Option<ActionId>,
    pub select_back_action: Option<ActionId>,
    pub close_action: Option<ActionId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicSelection {
    pub descriptor: Descriptor,
    pub source_ref: Hash32,
    pub position: Option<ReplayPosition>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SessionState {
    pub selected: Option<PublicSelection>,
    pub closed: Option<CloseReason>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptedRecord {
    pub seq: u64,
    pub source: Hash32,
    pub record: ReplayRecord,
    pub validation_ref: Hash32,
    pub lifecycle_ids: Vec<Hash32>,
    pub revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResultTask {
    pub verified_result_ref: Hash32,
    pub publish_action: ActionId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitStatus {
    Committed { actions_pending: usize },
    ActionsApplied,
    Faulted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaitReason {
    SourceGap { expected: u64, received: u64 },
    Preparation { cid: Cid },
    ClockDisabled,
    NoRunnableCandidate,
    PendingAction { action: ActionId },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapacityKind {
    Programs,
    Sessions,
    Branches,
    Records,
    Actions,
    Results,
    Events,
    Effects,
    DescriptorBytes,
    LifecycleBytes,
    Dependencies,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManagerError {
    InvalidBinding,
    Malformed,
    DescriptorConflict,
    CidMismatch,
    WrongCommittee,
    CandidateConflict,
    WrongPredecessor,
    PreparationBinding,
    SourceConflict,
    InvalidRecord,
    ReceiptConflict,
    UnknownAction,
    StaleDelta,
    CommitBinding,
    ClockRegression,
    ClockGeneration,
    Unsupported,
    Capacity { kind: CapacityKind, limit: usize },
    Overflow,
    RuntimeFaulted,
}

impl fmt::Display for ManagerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ManagerError {}
