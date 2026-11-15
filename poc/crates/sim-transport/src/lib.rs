//! Pure, deterministic transport and clock state for a controlled simulation.

mod clock;
mod config;
mod identity;
mod reliable;
mod route;
mod service;
mod snapshot;

pub use clock::*;
pub use config::*;
pub use identity::*;
pub use reliable::*;
pub use route::*;
pub use service::*;

use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Error {
    InvalidConfig,
    UnsupportedSchema,
    DuplicateId,
    UnknownEndpoint,
    UnknownChannel,
    MissingProfile,
    AmbiguousProfile,
    PermissionDenied,
    ContextMismatch,
    BindingMismatch,
    HashMismatch,
    InvalidOrigin,
    InvalidAttempt,
    SelfRoute,
    PayloadTooLarge,
    ResourceLimit,
    ArithmeticOverflow,
    TimeRegression,
    StepRegression,
    CrossInstance,
    ConfigMismatch,
    StalePlan,
    InvalidReceipt,
    UnknownMessage,
    MessageConflict,
    RetryNotDue,
    NonCanonical,
    SnapshotTooLarge,
    CorruptSnapshot,
    Uncalibrated,
    InvalidSample,
    StaleSample,
    LeaseExpired,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

/// A plan keeps its instance allocation alive so a new instance cannot reuse it.
#[derive(Debug)]
pub(crate) struct Instance(Arc<()>);

impl Instance {
    fn new() -> Self {
        Self(Arc::new(()))
    }

    fn for_plan(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    fn verify(&self, plan: &Self) -> Result<(), Error> {
        if Arc::ptr_eq(&self.0, &plan.0) {
            Ok(())
        } else {
            Err(Error::CrossInstance)
        }
    }
}
