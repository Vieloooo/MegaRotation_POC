use crate::{Error, Id, Result, hash_parts};
use rand_chacha::ChaCha20Rng;
use rand_core::{SeedableRng, TryCryptoRng, TryRng};
use serde::{Deserialize, Serialize};

const WORD_LIMIT: u128 = 1 << 68;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RngCheckpoint {
    pub seed: Id,
    pub stream: u64,
    pub word_position_hi: u64,
    pub word_position_lo: u64,
}

pub struct RoleRng {
    seed: Id,
    stream: u64,
    rng: ChaCha20Rng,
}

impl RoleRng {
    pub fn derive(run_seed: u64, node: u32, cid: Id, role: Id) -> Self {
        let seed = hash_parts(
            b"I1b/role-seed/v1",
            &[&run_seed.to_be_bytes(), &node.to_be_bytes(), &cid, &role],
        );
        Self::from_seed(seed, 0)
    }

    pub fn root_role() -> Id {
        hash_parts(b"I1b/role-path/v1", &[b"root"])
    }

    pub fn child_role(parent: Id, label: &str) -> Id {
        hash_parts(b"I1b/role-child/v1", &[&parent, label.as_bytes()])
    }

    pub fn attribute_role(parent: Id, key: &str, value: &str) -> Id {
        hash_parts(
            b"I1b/role-attribute/v1",
            &[&parent, key.as_bytes(), value.as_bytes()],
        )
    }

    fn from_seed(seed: Id, stream: u64) -> Self {
        let mut rng = ChaCha20Rng::from_seed(seed);
        rng.set_stream(stream);
        Self { seed, stream, rng }
    }

    pub fn checkpoint(&self) -> RngCheckpoint {
        let position = self.rng.get_word_pos();
        RngCheckpoint {
            seed: self.seed,
            stream: self.stream,
            word_position_hi: (position >> 64) as u64,
            word_position_lo: position as u64,
        }
    }

    pub fn restore(checkpoint: RngCheckpoint) -> Result<Self> {
        if checkpoint.word_position_hi > 15 {
            return Err(Error("RNG cursor outside stream"));
        }
        let position = (u128::from(checkpoint.word_position_hi) << 64)
            | u128::from(checkpoint.word_position_lo);
        let mut role = Self::from_seed(checkpoint.seed, checkpoint.stream);
        role.rng.set_word_pos(position);
        Ok(role)
    }

    fn reserve(&self, words: u128) -> Result<()> {
        // Keep the next position representable in a checkpoint. A position of
        // exactly WORD_LIMIT would alias zero in the native stream cursor.
        if self.rng.get_word_pos() + words >= WORD_LIMIT {
            return Err(Error("RNG stream exhausted"));
        }
        Ok(())
    }
}

impl TryRng for RoleRng {
    type Error = Error;

    fn try_next_u32(&mut self) -> Result<u32> {
        self.reserve(1)?;
        Ok(self.rng.try_next_u32().unwrap())
    }

    fn try_next_u64(&mut self) -> Result<u64> {
        self.reserve(2)?;
        Ok(self.rng.try_next_u64().unwrap())
    }

    fn try_fill_bytes(&mut self, bytes: &mut [u8]) -> Result<()> {
        self.reserve(bytes.len().div_ceil(4) as u128)?;
        self.rng.try_fill_bytes(bytes).unwrap();
        Ok(())
    }
}
impl TryCryptoRng for RoleRng {}
