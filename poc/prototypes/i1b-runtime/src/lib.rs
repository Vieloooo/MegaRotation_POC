//! Controlled task, time and RNG prototype for the Commonware 2026.10.0 local runtime patch.

mod clock;
mod context;
mod driver;
mod rng;
mod storage;

pub use clock::{ClockSample, Profile, Timer};
pub use context::SimContext;
pub use driver::{ControlledDriver, Event, Grant, Input, Placement, Step};
pub use rng::{RngCheckpoint, RoleRng};
pub use storage::{ControlledBlob, IoIdentity, IoOperation, IoResult, IoTarget};

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
