//! Controlled task, time and RNG prototype for the Commonware 2026.10.0 local runtime patch.

mod clock;
mod context;
mod driver;
mod native_application;
mod native_config;
mod native_elector;
pub mod native_hub;
mod native_network;
pub mod native_replica;
mod native_scheme;
mod native_step_journal;
mod rng;
mod storage;
mod witness;
mod witness_ledger;
mod witness_recovery;

pub use clock::{ClockSample, Profile, Timer};
pub use context::SimContext;
pub use driver::{ControlledDriver, Event, Grant, Input, Placement, Step};
pub use rng::{RngCheckpoint, RoleRng};
pub use storage::{ControlledBlob, IoIdentity, IoOperation, IoResult, IoTarget};
pub use witness::{
    ArtifactWitness, Barrier, DecodedArtifact, EligiblePayload, NativeWire, PageEvidence,
    PolicySeal, Submission, WitnessConfig, WitnessMember, WitnessPolicy, page_slot,
};

type Id = [u8; 32];
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq)]
pub struct Error(&'static str);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for Error {}

fn hash_parts(domain: &[u8], parts: &[&[u8]]) -> Id {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    for part in std::iter::once(domain).chain(parts.iter().copied()) {
        hash.update((part.len() as u64).to_be_bytes());
        hash.update(part);
    }
    hash.finalize().into()
}

pub use witness_ledger::{
    BlobSeal, IoObservation, LedgerEntry, LedgerIdentity, LedgerRecord, LedgerTail, ObservedChange,
    RecoveryLink, WitnessFault, WitnessHead, WitnessLedger,
};
pub use witness_recovery::{
    RecoveryPaths, RecoveryReport, native_data, recover_witness_generation,
};

/// Versioned native-process transport; the original I1a wire remains unchanged.
pub mod native_wire {
    tonic::include_proto!("rotation.native.v1");
}

pub mod native_wire_support;
