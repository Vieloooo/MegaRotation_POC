//! Versioned public identities and references, separate from the existing I1 wire format.
//!
//! Constructing an opaque reference does not authenticate its referenced object.

use serde::{Deserialize, Serialize};
use std::fmt;

const VERSION: u16 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomainError {
    InvalidPid { value: u8 },
    InvalidRegion { value: u8 },
    InvalidMember { value: u8 },
    InvalidSignerIndex { value: u8 },
    InvalidLength { expected: usize, actual: usize },
    UnsupportedVersion { version: u16 },
    Malformed { reason: &'static str },
    LimitExceeded { limit: usize, actual: usize },
    Overflow,
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for DomainError {}

/// Fixed-width, context-typed encoding. Nested fields do not repeat the version.
pub trait Canonical: Sized {
    fn encode(&self) -> Vec<u8>;
    fn decode(bytes: &[u8]) -> Result<Self, DomainError>;
}

fn body(bytes: &[u8], expected: usize) -> Result<&[u8], DomainError> {
    if bytes.len() != expected {
        return Err(DomainError::InvalidLength {
            expected,
            actual: bytes.len(),
        });
    }
    let version = u16::from_be_bytes([bytes[0], bytes[1]]);
    if version != VERSION {
        return Err(DomainError::UnsupportedVersion { version });
    }
    Ok(&bytes[2..])
}

fn versioned(raw: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(2 + raw.len());
    bytes.extend_from_slice(&VERSION.to_be_bytes());
    bytes.extend_from_slice(raw);
    bytes
}

macro_rules! identity {
    ($name:ident, $max:expr, $error:ident) => {
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(try_from = "u8", into = "u8")]
        pub struct $name(u8);

        impl $name {
            pub fn new(value: u8) -> Result<Self, DomainError> {
                if value > $max {
                    return Err(DomainError::$error { value });
                }
                Ok(Self(value))
            }
            pub const fn get(self) -> u8 {
                self.0
            }
        }

        impl TryFrom<u8> for $name {
            type Error = DomainError;
            fn try_from(value: u8) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for u8 {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl Canonical for $name {
            fn encode(&self) -> Vec<u8> {
                versioned(&[self.0])
            }
            fn decode(bytes: &[u8]) -> Result<Self, DomainError> {
                Self::new(body(bytes, 3)?[0])
            }
        }
    };
}

identity!(RegionId, 2, InvalidRegion);
identity!(MemberId, 4, InvalidMember);
identity!(Pid, 14, InvalidPid);
identity!(SignerIndex, 4, InvalidSignerIndex);

impl Pid {
    pub const fn from_region_member(region: RegionId, member: MemberId) -> Self {
        Self(5 * region.0 + member.0)
    }
    pub const fn region(self) -> RegionId {
        RegionId(self.0 / 5)
    }
    pub const fn member(self) -> MemberId {
        MemberId(self.0 % 5)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Hash32([u8; 32]);

impl Hash32 {
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl TryFrom<&[u8]> for Hash32 {
    type Error = DomainError;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let raw = bytes.try_into().map_err(|_| DomainError::InvalidLength {
            expected: 32,
            actual: bytes.len(),
        })?;
        Ok(Self(raw))
    }
}

impl Canonical for Hash32 {
    fn encode(&self) -> Vec<u8> {
        versioned(&self.0)
    }
    fn decode(bytes: &[u8]) -> Result<Self, DomainError> {
        Self::try_from(body(bytes, 34)?)
    }
}

macro_rules! opaque_hash {
    ($name:ident) => {
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(Hash32);

        impl $name {
            pub const fn new(hash: Hash32) -> Self {
                Self(hash)
            }
            pub const fn as_hash(&self) -> &Hash32 {
                &self.0
            }
            pub const fn as_bytes(&self) -> &[u8; 32] {
                self.0.as_bytes()
            }
        }
        impl From<Hash32> for $name {
            fn from(hash: Hash32) -> Self {
                Self(hash)
            }
        }
        impl Canonical for $name {
            fn encode(&self) -> Vec<u8> {
                self.0.encode()
            }
            fn decode(bytes: &[u8]) -> Result<Self, DomainError> {
                Ok(Self(Hash32::decode(bytes)?))
            }
        }
    };
}

opaque_hash!(Cid);
opaque_hash!(ConfigRef);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct L1Position {
    pub block: u64,
    pub tx: u64,
    pub log: u64,
}

fn read_u64(raw: &[u8]) -> u64 {
    let mut bytes = [0; 8];
    bytes.copy_from_slice(raw);
    u64::from_be_bytes(bytes)
}

impl Canonical for L1Position {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = versioned(&self.block.to_be_bytes());
        bytes.extend_from_slice(&self.tx.to_be_bytes());
        bytes.extend_from_slice(&self.log.to_be_bytes());
        bytes
    }
    fn decode(bytes: &[u8]) -> Result<Self, DomainError> {
        let raw = body(bytes, 26)?;
        Ok(Self {
            block: read_u64(&raw[..8]),
            tx: read_u64(&raw[8..16]),
            log: read_u64(&raw[16..]),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct L1BlockRef {
    pub height: u64,
    pub hash: Hash32,
}

impl Canonical for L1BlockRef {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = versioned(&self.height.to_be_bytes());
        bytes.extend_from_slice(self.hash.as_bytes());
        bytes
    }
    fn decode(bytes: &[u8]) -> Result<Self, DomainError> {
        let raw = body(bytes, 42)?;
        Ok(Self {
            height: read_u64(&raw[..8]),
            hash: Hash32::try_from(&raw[8..])?,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultKey {
    pub cid: Cid,
    pub block_hash: Hash32,
}

impl Canonical for ResultKey {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = versioned(self.cid.as_bytes());
        bytes.extend_from_slice(self.block_hash.as_bytes());
        bytes
    }
    fn decode(bytes: &[u8]) -> Result<Self, DomainError> {
        let raw = body(bytes, 66)?;
        Ok(Self {
            cid: Cid::new(Hash32::try_from(&raw[..32])?),
            block_hash: Hash32::try_from(&raw[32..])?,
        })
    }
}
