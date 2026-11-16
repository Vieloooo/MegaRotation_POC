//! GFR-POS-1, GFR-LIFE-1 and GFR-PUBLIC-1.

use std::cmp::Ordering;

use poc_types::domain::{
    CanonicalObject, DomainError, MAX_TRANSACTION_BYTES, MAX_TRANSACTIONS, validate_dep,
    validate_transactions,
};
use sim_espresso::MAX_MESSAGE_BYTES;

use crate::{
    CloseReason, Descriptor, GfrError, Hash32, L1Position, LifecycleBody, LifecycleEvent,
    PhysicalBlock, ReplayIndex, ReplayKind, ReplayPosition, SessionId, VirtualBlock,
};

const VERSION: u16 = 1;
const PUBLICATION_MAGIC: &[u8; 4] = b"GFRP";
pub const PUBLICATION_CODEC: u32 = 0x47465201;
pub const MAX_PROOF_BYTES: usize = 32_768;
pub const MAX_DESCRIPTOR_BYTES: usize = 16_384;
pub const MAX_RESUMED_BLOCKS: usize = 4_096;
pub const MAX_LIFECYCLE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PublicObject {
    Body(PhysicalBlock),
    FinalProof {
        descriptor: Descriptor,
        target: Hash32,
        proof: Vec<u8>,
    },
}

pub fn compare_positions(a: &ReplayPosition, b: &ReplayPosition) -> Result<Ordering, GfrError> {
    a.validate()?;
    b.validate()?;
    if let (ReplayIndex::L1(pa), ReplayIndex::L1(pb)) = (&a.index, &b.index)
        && pa == pb
        && a.kind != b.kind
    {
        return Err(GfrError::ConflictingControl(*pa));
    }
    Ok(position_key(a).cmp(&position_key(b)))
}

fn position_key(value: &ReplayPosition) -> (u64, u8, u64, u64, u64) {
    let (class, x, y, z) = match &value.index {
        ReplayIndex::L1(position) => (0, position.block, position.tx, position.log),
        ReplayIndex::Height(height) => (1, *height, 0, 0),
        ReplayIndex::Session(session) => (2, session.beta, session.r, 0),
        ReplayIndex::Espresso(q) => (3, *q, 0, 0),
    };
    (value.key_us, class, x, y, z)
}

pub fn encode_position(value: &ReplayPosition) -> Result<Vec<u8>, GfrError> {
    value.validate()?;
    let tag = match value.kind {
        ReplayKind::Force => 0,
        ReplayKind::Resume => 1,
        ReplayKind::Expire => 2,
        ReplayKind::SessionEnd => 3,
        ReplayKind::Espresso => 4,
    };
    let mut bytes = Vec::with_capacity(35);
    bytes.extend_from_slice(&VERSION.to_be_bytes());
    bytes.extend_from_slice(&value.key_us.to_be_bytes());
    bytes.push(tag);
    match &value.index {
        ReplayIndex::L1(position) => {
            bytes.extend_from_slice(&position.block.to_be_bytes());
            bytes.extend_from_slice(&position.tx.to_be_bytes());
            bytes.extend_from_slice(&position.log.to_be_bytes());
        }
        ReplayIndex::Height(h) | ReplayIndex::Espresso(h) => {
            bytes.extend_from_slice(&h.to_be_bytes());
        }
        ReplayIndex::Session(session) => {
            bytes.extend_from_slice(&session.beta.to_be_bytes());
            bytes.extend_from_slice(&session.r.to_be_bytes());
        }
    }
    Ok(bytes)
}

pub fn decode_position(bytes: &[u8]) -> Result<ReplayPosition, GfrError> {
    let mut reader = Bytes(bytes);
    reader.version()?;
    let key_us = reader.u64()?;
    let [tag] = reader.fixed()?;
    let (kind, index) = match tag {
        0 | 1 => {
            let kind = if tag == 0 {
                ReplayKind::Force
            } else {
                ReplayKind::Resume
            };
            let pos = L1Position {
                block: reader.u64()?,
                tx: reader.u64()?,
                log: reader.u64()?,
            };
            (kind, ReplayIndex::L1(pos))
        }
        2 => (ReplayKind::Expire, ReplayIndex::Height(reader.u64()?)),
        3 => {
            let session = SessionId {
                beta: reader.u64()?,
                r: reader.u64()?,
            };
            (ReplayKind::SessionEnd, ReplayIndex::Session(session))
        }
        4 => (ReplayKind::Espresso, ReplayIndex::Espresso(reader.u64()?)),
        _ => return Err(malformed("replay kind tag")),
    };
    reader.finish()?;
    Ok(ReplayPosition {
        key_us,
        kind,
        index,
    })
}

pub fn lifecycle_projection(value: &LifecycleEvent) -> Result<Vec<u8>, GfrError> {
    let mut out = BoundedBytes::new(MAX_LIFECYCLE_BYTES);
    out.push(&VERSION.to_be_bytes())?;
    out.frame(&encode_position(&value.position)?)?;
    match &value.body {
        LifecycleBody::SessionAssigned { descriptor } => {
            out.push(&[0])?;
            out.frame(&descriptor.encode_checked()?)?;
        }
        LifecycleBody::SessionClosed { session, reason } => {
            out.push(&[1])?;
            out.push(&session.beta.to_be_bytes())?;
            out.push(&session.r.to_be_bytes())?;
            out.push(&[reason_tag(*reason)])?;
        }
        LifecycleBody::BranchClosed { branch, reason } => {
            out.push(&[2])?;
            out.push(&branch.to_be_bytes())?;
            out.push(&[reason_tag(*reason)])?;
        }
        LifecycleBody::Resumed {
            control_seq,
            branch,
            anchor,
            derived_prefix,
        } => {
            bound(derived_prefix.len(), MAX_RESUMED_BLOCKS)?;
            out.push(&[3])?;
            out.push(&control_seq.to_be_bytes())?;
            out.push(&branch.to_be_bytes())?;
            out.frame(&anchor.encode_checked()?)?;
            out.count(derived_prefix.len())?;
            for block in derived_prefix {
                out.frame(&virtual_projection(block)?)?;
            }
        }
    }
    Ok(out.bytes)
}

fn virtual_projection(block: &VirtualBlock) -> Result<Vec<u8>, GfrError> {
    validate_dep(&block.metadata, &block.dep)?;
    validate_transactions(&block.transactions, MAX_TRANSACTION_BYTES)?;
    let mut out = BoundedBytes::new(MAX_LIFECYCLE_BYTES);
    out.frame(&block.metadata.encode_checked()?)?;
    out.list(&block.dep)?;
    out.push(block.pre_state_root.as_bytes())?;
    out.list(&block.transactions)?;
    Ok(out.bytes)
}

fn reason_tag(reason: CloseReason) -> u8 {
    match reason {
        CloseReason::Time => 0,
        CloseReason::Complete => 1,
        CloseReason::Force => 2,
        CloseReason::Resume => 3,
        CloseReason::Expired => 4,
        CloseReason::Excluded => 5,
        CloseReason::Incompatible => 6,
    }
}

pub fn lifecycle_id(value: &LifecycleEvent) -> Result<Hash32, GfrError> {
    let bytes = lifecycle_projection(value)?;
    Ok(Hash32::try_from(
        poc_types::hash_parts("PoC/GfrLifecycle/v1", &[&bytes]).as_slice(),
    )?)
}

pub fn encode_publication(value: &PublicObject) -> Result<Vec<u8>, GfrError> {
    let mut out = BoundedBytes::new(MAX_MESSAGE_BYTES);
    match value {
        PublicObject::Body(block) => {
            // 03 v1 has a fixed 94-byte block prefix and a u32 frame per transaction.
            bound(block.transactions.len(), MAX_TRANSACTIONS)?;
            let envelope_len = block
                .transactions
                .iter()
                .try_fold(7 + 4 + 94, |length, tx| total_len(&[length, 4, tx.len()]))?;
            bound(envelope_len, MAX_MESSAGE_BYTES)?;
            let body = block.encode_checked()?;
            bound(total_len(&[7, 4, body.len()])?, MAX_MESSAGE_BYTES)?;
            publication_header(&mut out, 0)?;
            out.frame(&body)?;
        }
        PublicObject::FinalProof {
            descriptor,
            target,
            proof,
        } => {
            bound(proof.len(), MAX_PROOF_BYTES)?;
            let descriptor = descriptor.encode_checked()?;
            bound(descriptor.len(), MAX_DESCRIPTOR_BYTES)?;
            bound(
                total_len(&[7, 4, descriptor.len(), 32, 4, proof.len()])?,
                MAX_MESSAGE_BYTES,
            )?;
            publication_header(&mut out, 1)?;
            out.frame(&descriptor)?;
            out.push(target.as_bytes())?;
            out.frame(proof)?;
        }
    }
    Ok(out.bytes)
}

fn publication_header(out: &mut BoundedBytes, tag: u8) -> Result<(), GfrError> {
    out.push(PUBLICATION_MAGIC)?;
    out.push(&VERSION.to_be_bytes())?;
    out.push(&[tag])
}

pub fn decode_publication(bytes: &[u8]) -> Result<PublicObject, GfrError> {
    bound(bytes.len(), MAX_MESSAGE_BYTES)?;
    let mut reader = Bytes(bytes);
    if &reader.fixed()? != PUBLICATION_MAGIC {
        return Err(malformed("publication magic"));
    }
    reader.version()?;
    let [tag] = reader.fixed()?;
    let object = match tag {
        0 => {
            let body = reader.frame(MAX_MESSAGE_BYTES)?;
            reader.finish()?;
            PublicObject::Body(PhysicalBlock::decode(body)?)
        }
        1 => {
            let descriptor = reader.frame(MAX_DESCRIPTOR_BYTES)?;
            let target = Hash32::new(reader.fixed()?);
            let proof = reader.frame(MAX_PROOF_BYTES)?;
            reader.finish()?;
            PublicObject::FinalProof {
                descriptor: Descriptor::decode(descriptor)?,
                target,
                proof: proof.to_vec(),
            }
        }
        _ => return Err(malformed("publication tag")),
    };
    Ok(object)
}

fn malformed(reason: &'static str) -> GfrError {
    DomainError::Malformed { reason }.into()
}

fn bound(actual: usize, limit: usize) -> Result<(), GfrError> {
    if actual > limit {
        return Err(DomainError::LimitExceeded { limit, actual }.into());
    }
    Ok(())
}

fn total_len(lengths: &[usize]) -> Result<usize, GfrError> {
    lengths.iter().try_fold(0usize, |total, n| {
        total.checked_add(*n).ok_or(DomainError::Overflow.into())
    })
}

struct Bytes<'a>(&'a [u8]);

impl<'a> Bytes<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], GfrError> {
        let (bytes, remaining) = self
            .0
            .split_at_checked(length)
            .ok_or(malformed("truncated frame"))?;
        self.0 = remaining;
        Ok(bytes)
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], GfrError> {
        self.take(N)?
            .try_into()
            .map_err(|_| malformed("fixed-width value"))
    }

    fn version(&mut self) -> Result<(), GfrError> {
        let version = u16::from_be_bytes(self.fixed()?);
        if version != VERSION {
            return Err(DomainError::UnsupportedVersion { version }.into());
        }
        Ok(())
    }

    fn u64(&mut self) -> Result<u64, GfrError> {
        Ok(u64::from_be_bytes(self.fixed()?))
    }

    fn frame(&mut self, max: usize) -> Result<&'a [u8], GfrError> {
        let length = usize::try_from(u32::from_be_bytes(self.fixed()?))
            .map_err(|_| DomainError::Overflow)?;
        bound(length, max)?;
        self.take(length)
    }

    fn finish(self) -> Result<(), GfrError> {
        if !self.0.is_empty() {
            return Err(malformed("trailing bytes"));
        }
        Ok(())
    }
}

struct BoundedBytes {
    bytes: Vec<u8>,
    max: usize,
}

impl BoundedBytes {
    fn new(max: usize) -> Self {
        Self {
            bytes: Vec::new(),
            max,
        }
    }

    fn push(&mut self, bytes: &[u8]) -> Result<(), GfrError> {
        bound(total_len(&[self.bytes.len(), bytes.len()])?, self.max)?;
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }

    fn count(&mut self, count: usize) -> Result<(), GfrError> {
        let encoded = u32::try_from(count).map_err(|_| DomainError::Overflow)?;
        self.push(&encoded.to_be_bytes())
    }

    fn frame(&mut self, bytes: &[u8]) -> Result<(), GfrError> {
        bound(total_len(&[self.bytes.len(), 4, bytes.len()])?, self.max)?;
        self.count(bytes.len())?;
        self.push(bytes)
    }

    fn list(&mut self, values: &[Vec<u8>]) -> Result<(), GfrError> {
        self.count(values.len())?;
        for value in values {
            self.frame(value)?;
        }
        Ok(())
    }
}
