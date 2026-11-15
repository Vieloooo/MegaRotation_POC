//! Validated P0 registry and static session authorization.

mod registry;
mod schedule;

pub use registry::{POP_NAMESPACE, ParticipantMap, RegistryError, ValidatedRegistry};
pub use schedule::{ConfigError, PlanError, PlanOutcome, StaticScheduleProvider, schedule};
