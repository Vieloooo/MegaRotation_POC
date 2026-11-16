//! BS-FRAME-v1, the P0 closed channel format shared with derivation.

use poc_types::domain::{Hash32, MAX_TRANSACTION_BYTES, MAX_TRANSACTIONS, validate_transactions};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

const MAGIC: &[u8; 4] = b"P0B1";
const VERSION: u16 = 1;
const TRANSFER_LEN: usize = 19;
const MAX_DATA: usize = 2 + 2 * (20 + MAX_TRANSACTIONS * (4 + TRANSFER_LEN));
pub const MAX_PAYLOAD: usize = 29 + MAX_DATA;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BlockRecord {
    pub block_number: u64,
    pub epoch_num: u64,
    pub transactions: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ClosedFrame {
    pub channel_id: [u8; 16],
    pub records: Vec<BlockRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CodecError {
    UnsupportedVersion(u16),
    Malformed(&'static str),
    LimitExceeded,
    Overflow,
    CommitmentMismatch,
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for CodecError {}

pub fn commitment(payload: &[u8]) -> Hash32 {
    Hash32::new(keccak(payload))
}

pub(crate) fn keccak(bytes: &[u8]) -> [u8; 32] {
    Keccak256::digest(bytes).into()
}

fn channel_id(data: &[u8]) -> [u8; 16] {
    keccak(data)[..16].try_into().expect("fixed hash width")
}

fn count_records(count: usize) -> Result<(), CodecError> {
    if matches!(count, 1 | 2) {
        Ok(())
    } else {
        Err(CodecError::Malformed("record count"))
    }
}

fn check_record(record: &BlockRecord, previous: Option<u64>) -> Result<(), CodecError> {
    if let Some(previous) = previous {
        let expected = previous.checked_add(1).ok_or(CodecError::Overflow)?;
        if expected != record.block_number {
            return Err(CodecError::Malformed("noncontiguous heights"));
        }
    }
    if record.transactions.len() > MAX_TRANSACTIONS {
        return Err(CodecError::LimitExceeded);
    }
    if record
        .transactions
        .iter()
        .any(|tx| tx.len() != TRANSFER_LEN)
    {
        return Err(CodecError::Malformed("ordinary transaction width"));
    }
    validate_transactions(&record.transactions, MAX_TRANSACTION_BYTES)
        .map_err(|_| CodecError::Malformed("ordinary transaction"))
}

pub fn encode_closed(records: &[BlockRecord]) -> Result<Vec<u8>, CodecError> {
    count_records(records.len())?;
    let mut data = Vec::new();
    data.extend((records.len() as u16).to_be_bytes());
    let mut previous = None;
    for record in records {
        check_record(record, previous)?;
        previous = Some(record.block_number);
        data.extend(record.block_number.to_be_bytes());
        data.extend(record.epoch_num.to_be_bytes());
        data.extend((record.transactions.len() as u32).to_be_bytes());
        for tx in &record.transactions {
            data.extend((TRANSFER_LEN as u32).to_be_bytes());
            data.extend(tx);
        }
    }
    let mut payload = Vec::with_capacity(29 + data.len());
    payload.extend(MAGIC);
    payload.extend(VERSION.to_be_bytes());
    payload.extend(channel_id(&data));
    payload.extend(0u16.to_be_bytes());
    payload.extend((data.len() as u32).to_be_bytes());
    payload.extend(data);
    payload.push(1);
    Ok(payload)
}

/// Decode one exact frame. A format error never returns a partial record list.
pub fn decode_closed(payload: &[u8]) -> Result<ClosedFrame, CodecError> {
    if payload.len() > MAX_PAYLOAD {
        return Err(CodecError::LimitExceeded);
    }
    let mut payload = payload;
    if take::<4>(&mut payload)? != *MAGIC {
        return Err(CodecError::Malformed("magic"));
    }
    let version = u16::from_be_bytes(take(&mut payload)?);
    if version != VERSION {
        return Err(CodecError::UnsupportedVersion(version));
    }
    let id = take::<16>(&mut payload)?;
    if take::<2>(&mut payload)? != [0; 2] {
        return Err(CodecError::Malformed("nonzero frame number"));
    }
    let len = u32::from_be_bytes(take(&mut payload)?) as usize;
    if len > MAX_DATA {
        return Err(CodecError::LimitExceeded);
    }
    let (mut data, trailer) = payload
        .split_at_checked(len)
        .ok_or(CodecError::Malformed("frame data length"))?;
    if trailer != [1] || channel_id(data) != id {
        return Err(CodecError::Malformed("frame trailer or channel id"));
    }
    let count = u16::from_be_bytes(take(&mut data)?) as usize;
    count_records(count)?;
    let mut records = Vec::<BlockRecord>::with_capacity(count);
    for _ in 0..count {
        let block_number = u64::from_be_bytes(take(&mut data)?);
        let epoch_num = u64::from_be_bytes(take(&mut data)?);
        let tx_count = u32::from_be_bytes(take(&mut data)?) as usize;
        if tx_count > MAX_TRANSACTIONS {
            return Err(CodecError::LimitExceeded);
        }
        if data.len() < tx_count * (4 + TRANSFER_LEN) {
            return Err(CodecError::Malformed("truncated transactions"));
        }
        let mut transactions = Vec::with_capacity(tx_count);
        for _ in 0..tx_count {
            if u32::from_be_bytes(take(&mut data)?) as usize != TRANSFER_LEN {
                return Err(CodecError::Malformed("ordinary transaction width"));
            }
            transactions.push(take::<TRANSFER_LEN>(&mut data)?.to_vec());
        }
        let record = BlockRecord {
            block_number,
            epoch_num,
            transactions,
        };
        check_record(&record, records.last().map(|r| r.block_number))?;
        records.push(record);
    }
    if !data.is_empty() {
        return Err(CodecError::Malformed("trailing frame data"));
    }
    Ok(ClosedFrame {
        channel_id: id,
        records,
    })
}

pub fn verify_payload(expected: Hash32, payload: &[u8]) -> Result<ClosedFrame, CodecError> {
    if payload.len() > MAX_PAYLOAD {
        return Err(CodecError::LimitExceeded);
    }
    if commitment(payload) != expected {
        return Err(CodecError::CommitmentMismatch);
    }
    decode_closed(payload)
}

fn take<const N: usize>(data: &mut &[u8]) -> Result<[u8; N], CodecError> {
    let (part, remainder) = data
        .split_at_checked(N)
        .ok_or(CodecError::Malformed("truncated frame"))?;
    *data = remainder;
    Ok(part.try_into().expect("fixed field width"))
}
