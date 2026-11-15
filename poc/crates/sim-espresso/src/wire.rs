use crate::{
    FinalizedBlock, LogMessage, MAX_BLOCK_BYTES, MAX_MESSAGE_BYTES, MAX_MESSAGES, MAX_SOURCE_BYTES,
    ProposedBlock, PublicTimeUs, TickUs,
};

pub(crate) const HEADER_BYTES: usize = 47;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WireError {
    BadMagic,
    UnsupportedVersion,
    InvalidSourceId,
    InvalidBoolean,
    InvalidTime,
    InvalidCodec,
    TooManyMessages,
    MessageTooLarge,
    BlockTooLarge,
    Truncated,
    TrailingBytes,
}

pub(crate) fn validate_source(source_id: &[u8]) -> Result<(), WireError> {
    if (1..=MAX_SOURCE_BYTES).contains(&source_id.len()) {
        Ok(())
    } else {
        Err(WireError::InvalidSourceId)
    }
}

pub(crate) fn message_len(message: &LogMessage) -> Result<usize, WireError> {
    if message.codec == 0 {
        return Err(WireError::InvalidCodec);
    }
    if message.bytes.len() > MAX_MESSAGE_BYTES {
        return Err(WireError::MessageTooLarge);
    }
    Ok(8 + message.bytes.len())
}

pub(crate) fn block_len(source_id: &[u8], messages: &[LogMessage]) -> Result<usize, WireError> {
    validate_source(source_id)?;
    if messages.len() > MAX_MESSAGES {
        return Err(WireError::TooManyMessages);
    }
    let mut len = HEADER_BYTES + source_id.len();
    for message in messages {
        len += message_len(message)?;
        if len > MAX_BLOCK_BYTES {
            return Err(WireError::BlockTooLarge);
        }
    }
    Ok(len)
}

pub fn encode_finalized(source_id: &[u8], block: &FinalizedBlock) -> Result<Vec<u8>, WireError> {
    let proposal = &block.proposal;
    let len = block_len(source_id, &proposal.messages)?;
    if block.finalized_at_us < proposal.proposed_at_us {
        return Err(WireError::InvalidTime);
    }
    let mut bytes = Vec::with_capacity(len);
    bytes.extend_from_slice(b"PELG");
    bytes.extend_from_slice(&1u16.to_le_bytes());
    put_bytes(&mut bytes, source_id);
    bytes.extend_from_slice(&proposal.q.to_le_bytes());
    bytes.extend_from_slice(&proposal.theta_us.0.to_le_bytes());
    bytes.extend_from_slice(&proposal.proposed_at_us.0.to_le_bytes());
    bytes.push(u8::from(proposal.live_at_proposal));
    bytes.extend_from_slice(&block.finalized_at_us.0.to_le_bytes());
    bytes.extend_from_slice(&(proposal.messages.len() as u32).to_le_bytes());
    for message in &proposal.messages {
        put_message(&mut bytes, message);
    }
    Ok(bytes)
}

/// Structural decoding. The caller checks provenance before reader delivery.
pub fn decode_finalized(bytes: &[u8]) -> Result<(Vec<u8>, FinalizedBlock), WireError> {
    if bytes.len() > MAX_BLOCK_BYTES {
        return Err(WireError::BlockTooLarge);
    }
    let mut decoder = Decoder(bytes);
    if decoder.take(4)? != b"PELG" {
        return Err(WireError::BadMagic);
    }
    if decoder.u16()? != 1 {
        return Err(WireError::UnsupportedVersion);
    }
    let source_id = decoder.bounded_bytes(MAX_SOURCE_BYTES, WireError::InvalidSourceId)?;
    validate_source(source_id)?;
    let q = decoder.u64()?;
    let theta_us = PublicTimeUs(decoder.u64()?);
    let proposed_at_us = TickUs(decoder.u64()?);
    let live_at_proposal = decoder.boolean()?;
    let finalized_at_us = TickUs(decoder.u64()?);
    if finalized_at_us < proposed_at_us {
        return Err(WireError::InvalidTime);
    }
    let count = decoder.u32()? as usize;
    if count > MAX_MESSAGES {
        return Err(WireError::TooManyMessages);
    }
    let mut messages = Vec::new();
    for _ in 0..count {
        messages.push(decoder.message()?);
    }
    decoder.finish()?;
    Ok((
        source_id.to_vec(),
        FinalizedBlock {
            proposal: ProposedBlock {
                q,
                theta_us,
                proposed_at_us,
                live_at_proposal,
                messages,
            },
            finalized_at_us,
        },
    ))
}

pub(crate) fn put_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
}

pub(crate) fn put_message(out: &mut Vec<u8>, message: &LogMessage) {
    out.extend_from_slice(&message.codec.to_le_bytes());
    put_bytes(out, &message.bytes);
}

pub(crate) struct Decoder<'a>(pub &'a [u8]);

impl<'a> Decoder<'a> {
    pub fn take(&mut self, len: usize) -> Result<&'a [u8], WireError> {
        let (head, tail) = self.0.split_at_checked(len).ok_or(WireError::Truncated)?;
        self.0 = tail;
        Ok(head)
    }

    pub fn boolean(&mut self) -> Result<bool, WireError> {
        match self.take(1)?[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(WireError::InvalidBoolean),
        }
    }

    pub fn u16(&mut self) -> Result<u16, WireError> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().expect("exact length"),
        ))
    }

    pub fn u32(&mut self) -> Result<u32, WireError> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("exact length"),
        ))
    }

    pub fn u64(&mut self) -> Result<u64, WireError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("exact length"),
        ))
    }

    pub fn bounded_bytes(&mut self, max: usize, error: WireError) -> Result<&'a [u8], WireError> {
        let len = self.u32()? as usize;
        if len > max {
            return Err(error);
        }
        self.take(len)
    }

    pub fn message(&mut self) -> Result<LogMessage, WireError> {
        let codec = self.u32()?;
        if codec == 0 {
            return Err(WireError::InvalidCodec);
        }
        let bytes = self.bounded_bytes(MAX_MESSAGE_BYTES, WireError::MessageTooLarge)?;
        Ok(LogMessage {
            codec,
            bytes: bytes.to_vec(),
        })
    }

    pub fn finish(self) -> Result<(), WireError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(WireError::TrailingBytes)
        }
    }
}
