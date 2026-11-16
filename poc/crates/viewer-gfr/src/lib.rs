//! Pure Viewer GFR over delivered Espresso and L1 inputs.

mod api;
mod codec;
mod frontier;
mod publications;
mod reader;
mod replay;
mod state;

pub use api::*;
pub use codec::*;
pub use poc_types::domain::{
    Anchor, CloseReason, ConfigRef, Descriptor, ExecutionAnchor, FinalVerifier, Hash32, InputError,
    InputMissing, InputStatus, L1Header, L1Position, LifecycleBody, LifecycleEvent, PhysicalBlock,
    PlanInput, PlanRecord, PublicConfig, ReplayIndex, ReplayKind, ReplayPosition, ResultKey,
    SessionId, VirtualBlock, Window,
};
pub use publications::{
    BodyEvidence, ProofEvidence, ProofVerdict, PublicationDiagnostic, PublicationIndex,
    ResultEvidence, ResultExclusion, interval_unexpired,
};
pub use reader::FixtureL1Reader;
pub use replay::{advance_global, init};
pub use state::{
    GlobalCheckpoint, InitInput, InitResult, PublicReplayState, ReplayResult, ReplayState,
    SupportingCuts, chain, responsibility,
};
