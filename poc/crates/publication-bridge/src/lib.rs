//! Bounded M1 publication inputs and their historical verification adapter.
#![forbid(unsafe_code)]

mod result;
mod verifier;

use poc_types::domain::{DomainError, InputError};
use poc_viewer_gfr::GfrError;

pub use result::{BridgeResult, BridgeStateCodec, publication_messages};
pub use verifier::{BridgeInputs, ContextVerifier, NativePublicationVerifier};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BridgeError {
    InvalidConfig,
    NonNativeProfile,
    Capacity,
    InvalidEncoding,
    InvalidResult,
    SourceMismatch,
    Gfr(GfrError),
    Input(InputError),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for BridgeError {}

impl From<DomainError> for BridgeError {
    fn from(error: DomainError) -> Self {
        if matches!(error, DomainError::LimitExceeded { .. }) {
            Self::Capacity
        } else {
            Self::Input(InputError::Malformed(error))
        }
    }
}

impl From<GfrError> for BridgeError {
    fn from(error: GfrError) -> Self {
        Self::Gfr(error)
    }
}
