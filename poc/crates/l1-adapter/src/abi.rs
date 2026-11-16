use serde::Serialize;
use sha3::{Digest, Keccak256};

const CURSOR_WORDS: usize = 5;
const BATCH_WORDS: usize = 8;

pub fn hash(bytes: &[u8]) -> [u8; 32] {
    Keccak256::digest(bytes).into()
}

pub fn hex(bytes: &[u8]) -> String {
    let mut value = String::from("0x");
    for byte in bytes {
        use std::fmt::Write;
        write!(value, "{byte:02x}").expect("write to String");
    }
    value
}

pub fn unhex(input: &str) -> Option<Vec<u8>> {
    let bytes = input.strip_prefix("0x")?.as_bytes();
    if bytes.len() % 2 != 0 {
        return None;
    }
    fn nibble(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _ => None,
        }
    }
    bytes
        .chunks_exact(2)
        .map(|pair| Some(nibble(pair[0])? * 16 + nibble(pair[1])?))
        .collect()
}

pub fn selector(signature: &str) -> [u8; 4] {
    hash(signature.as_bytes())[..4].try_into().unwrap()
}

pub fn topic(signature: &str) -> String {
    hex(&hash(signature.as_bytes()))
}

pub(crate) const INITIALIZED: &str = "CursorInitialized((uint64,uint64,bytes32,uint8,bytes))";
pub(crate) const ADVANCED: &str = "CursorAdvanced(address,bytes32,(uint64,uint64,bytes32,uint8,bytes),(uint64,uint64,bytes32,uint8,bytes),bytes32)";
pub(crate) const SET: &str =
    "CursorSet((uint64,uint64,bytes32,uint8,bytes),(uint64,uint64,bytes32,uint8,bytes))";
pub(crate) const FORCE: &str =
    "ForceRecorded(address,bytes32,uint64,uint64,(uint8,uint8,uint8,uint8,uint64),uint64)";
pub(crate) const RESUME: &str =
    "ResumeRecorded(address,bytes32,uint64,(uint8,uint8,uint8,uint8,uint64),uint64,uint64,uint64)";
pub(crate) const SUBMIT_BATCH: &str = "submitBatch(bytes32,((uint64,uint64,bytes32,uint8,bytes),(uint64,uint64,bytes32,uint8,bytes),bytes32,bytes,uint64,uint64,bytes,bytes))";

pub(crate) fn word(data: &[u8], index: usize) -> Option<&[u8; 32]> {
    let start = index.checked_mul(32)?;
    data.get(start..start.checked_add(32)?)?.try_into().ok()
}

pub(crate) fn uint(word: &[u8; 32]) -> Option<u64> {
    if word[..24] != [0; 24] {
        return None;
    }
    Some(u64::from_be_bytes(word[24..].try_into().ok()?))
}

pub(crate) fn uint_at(data: &[u8], index: usize) -> Option<u64> {
    uint(word(data, index)?)
}

fn expect_offset(data: &[u8], word_index: usize, expected: usize) -> Option<()> {
    (usize::try_from(uint_at(data, word_index)?).ok()? == expected).then_some(())
}

fn dynamic_bytes(data: &[u8], offset: usize, max: usize) -> Option<(&[u8], usize)> {
    let data = data.get(offset..)?;
    let size = usize::try_from(uint_at(data, 0)?).ok()?;
    if size > max {
        return None;
    }
    let end = 32 + size;
    let padded_end = end.next_multiple_of(32);
    if data.get(end..padded_end)?.iter().any(|byte| *byte != 0) {
        return None;
    }
    Some((data.get(32..end)?, offset.checked_add(padded_end)?))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CursorWire {
    pub height: u64,
    pub origin: u64,
    pub frames_digest: String,
    pub source_kind: u8,
    pub source: String,
}

impl CursorWire {
    pub(crate) fn resume_seq(&self) -> Option<u64> {
        if self.source_kind != 2 {
            return None;
        }
        uint(&unhex(&self.source)?.try_into().ok()?)
    }
}

fn cursor_at(data: &[u8], offset: usize) -> Option<(CursorWire, usize)> {
    let data = data.get(offset..)?;
    expect_offset(data, 4, CURSOR_WORDS * 32)?;
    let (source, end) = dynamic_bytes(data, CURSOR_WORDS * 32, 1024)?;
    let source_kind = u8::try_from(uint_at(data, 3)?).ok()?;
    let frames = word(data, 2)?;
    let valid = match source_kind {
        0 => source.is_empty(),
        1 => !source.is_empty(),
        2 => source.len() == 32 && uint(source.try_into().ok()?).is_some(),
        _ => false,
    };
    if !valid || *frames != [0u8; 32] {
        return None;
    }
    Some((
        CursorWire {
            height: uint_at(data, 0)?,
            origin: uint_at(data, 1)?,
            frames_digest: hex(frames),
            source_kind,
            source: hex(source),
        },
        offset.checked_add(end)?,
    ))
}

pub(crate) fn single_cursor(data: &[u8]) -> Option<CursorWire> {
    expect_offset(data, 0, 32)?;
    let (cursor, end) = cursor_at(data, 32)?;
    (end == data.len()).then_some(cursor)
}

pub(crate) fn cursor_pair(data: &[u8], advanced: bool) -> Option<(CursorWire, CursorWire)> {
    let head = if advanced { 96 } else { 64 };
    expect_offset(data, 0, head)?;
    let (prev, next_start) = cursor_at(data, head)?;
    expect_offset(data, 1, next_start)?;
    let (next, end) = cursor_at(data, next_start)?;
    (end == data.len()).then_some((prev, next))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BatchWire {
    pub request_id: String,
    pub prev: CursorWire,
    pub next: CursorWire,
    pub commitment: String,
    pub da_certificate: String,
    pub first_height: u64,
    pub last_height: u64,
    pub gpos: String,
    pub certifier_signature: String,
}

pub(crate) fn batch_calldata(calldata: &[u8]) -> Option<BatchWire> {
    if calldata.get(..4)? != selector(SUBMIT_BATCH) {
        return None;
    }
    let args = calldata.get(4..)?;
    let request_id = hex(word(args, 0)?);
    expect_offset(args, 1, 64)?;
    let data = args.get(64..)?;
    expect_offset(data, 0, BATCH_WORDS * 32)?;
    let (prev, next_start) = cursor_at(data, BATCH_WORDS * 32)?;
    expect_offset(data, 1, next_start)?;
    let (next, da_start) = cursor_at(data, next_start)?;
    expect_offset(data, 3, da_start)?;
    let (da, gpos_start) = dynamic_bytes(data, da_start, 65)?;
    expect_offset(data, 6, gpos_start)?;
    let (gpos, cert_start) = dynamic_bytes(data, gpos_start, 1024)?;
    expect_offset(data, 7, cert_start)?;
    let (cert, end) = dynamic_bytes(data, cert_start, 65)?;
    if end != data.len() || da.len() != 65 || cert.len() != 65 || gpos.is_empty() {
        return None;
    }
    Some(BatchWire {
        request_id,
        prev,
        next,
        commitment: hex(word(data, 2)?),
        da_certificate: hex(da),
        first_height: uint_at(data, 4)?,
        last_height: uint_at(data, 5)?,
        gpos: hex(gpos),
        certifier_signature: hex(cert),
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PolicyWire {
    pub enabled_mask: u8,
    pub order: [u8; 3],
    pub l1_floor: u64,
}

pub(crate) fn policy_at(data: &[u8], index: usize, regional: bool) -> Option<PolicyWire> {
    let enabled_mask = u8::try_from(uint_at(data, index)?).ok()?;
    let order = [
        u8::try_from(uint_at(data, index + 1)?).ok()?,
        u8::try_from(uint_at(data, index + 2)?).ok()?,
        u8::try_from(uint_at(data, index + 3)?).ok()?,
    ];
    let mut sorted = order;
    sorted.sort_unstable();
    if (regional && (enabled_mask > 7 || sorted != [0, 1, 2]))
        || (!regional && (enabled_mask != 7 || order != [0, 1, 2]))
    {
        return None;
    }
    Some(PolicyWire {
        enabled_mask,
        order,
        l1_floor: uint_at(data, index + 4)?,
    })
}
