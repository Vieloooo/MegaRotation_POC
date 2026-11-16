use std::collections::BTreeMap;

use crate::{
    FinalizedBlock, LogMessage, MAX_BLOCK_BYTES, MAX_BLOCKS, MAX_LOG_BYTES, MAX_MESSAGES,
    MAX_SOURCE_BYTES, ProposedBlock, PublicTimeUs, TickUs, WireError,
    wire::{self, Decoder},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockHeader {
    pub q: u64,
    pub theta_us: PublicTimeUs,
    pub proposed_at_us: TickUs,
    pub live_at_proposal: bool,
    pub finalized_at_us: TickUs,
    pub message_count: u32,
}

impl From<&FinalizedBlock> for BlockHeader {
    fn from(block: &FinalizedBlock) -> Self {
        let proposal = &block.proposal;
        Self {
            q: proposal.q,
            theta_us: proposal.theta_us,
            proposed_at_us: proposal.proposed_at_us,
            live_at_proposal: proposal.live_at_proposal,
            finalized_at_us: block.finalized_at_us,
            // An independently manufactured block with too many entries is
            // rejected by receive_block before this conversion is used.
            message_count: u32::try_from(proposal.messages.len()).unwrap_or(u32::MAX),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderReceipt {
    pub q: u64,
    pub reader_received_at_us: TickUs,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceError {
    SourceMismatch,
    ReaderMismatch,
    ConflictingHeader { q: u64 },
    ConflictingMessage { q: u64, index: u32 },
    InvalidIndex { q: u64, index: u32 },
    Regressed { earlier_q: u64, later_q: u64 },
    EventTickRegressed { earlier_q: u64, later_q: u64 },
    ReceivedBeforeFinalization { q: u64 },
    InvalidStructure(WireError),
    InvalidSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeliveryError {
    InvalidSource(SourceError),
    Capacity,
}

impl From<SourceError> for DeliveryError {
    fn from(value: SourceError) -> Self {
        Self::InvalidSource(value)
    }
}

impl From<WireError> for DeliveryError {
    fn from(value: WireError) -> Self {
        SourceError::InvalidStructure(value).into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MissingLog {
    Header,
    Messages { indices: Vec<u32> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadError {
    Wait { next_q: u64, missing: MissingLog },
    InvalidSource(SourceError),
}

/// Reader-neutral interface for both modeled and source-verified delivery.
pub trait FinalizedLogReader {
    fn source_id(&self) -> &[u8];
    fn reader_id(&self) -> &[u8];
    fn tip(&self) -> Option<(u64, PublicTimeUs)>;
    fn read(&self, q: u64) -> Result<&FinalizedBlock, ReadError>;
    fn require_prefix_through(&self, q: u64) -> Result<VerifiedPrefix<'_>, ReadError>;
    fn require_watermark(&self, x: PublicTimeUs) -> Result<VerifiedPrefix<'_>, ReadError>;
}

/// Borrowed complete prefix. Only ReaderState can construct this view.
#[derive(Clone, Copy, Debug)]
pub struct VerifiedPrefix<'a> {
    source_id: &'a [u8],
    reader_id: &'a [u8],
    blocks: &'a [FinalizedBlock],
}

impl<'a> VerifiedPrefix<'a> {
    pub fn source_id(&self) -> &'a [u8] {
        self.source_id
    }

    pub fn reader_id(&self) -> &'a [u8] {
        self.reader_id
    }

    pub fn blocks(&self) -> &'a [FinalizedBlock] {
        self.blocks
    }

    pub fn tip(&self) -> (u64, PublicTimeUs) {
        let last = &self.blocks.last().expect("nonempty prefix").proposal;
        (last.q, last.theta_us)
    }

    pub fn read(&self, q: u64) -> Option<&'a FinalizedBlock> {
        self.blocks.get(usize::try_from(q).ok()?)
    }
}

#[derive(Clone, Debug, Default)]
struct Fragments {
    header: Option<(BlockHeader, TickUs)>,
    messages: BTreeMap<u32, (LogMessage, TickUs)>,
}

impl Fragments {
    fn validate(&self, q: u64, source_id: &[u8]) -> Result<usize, DeliveryError> {
        let count = if let Some((header, tick)) = self.header {
            if header.message_count as usize > MAX_MESSAGES {
                return Err(WireError::TooManyMessages.into());
            }
            if header.finalized_at_us < header.proposed_at_us {
                return Err(WireError::InvalidTime.into());
            }
            if tick < header.finalized_at_us {
                return Err(SourceError::ReceivedBeforeFinalization { q }.into());
            }
            for (&index, (_, received)) in &self.messages {
                if index >= header.message_count {
                    return Err(SourceError::InvalidIndex { q, index }.into());
                }
                if *received < header.finalized_at_us {
                    return Err(SourceError::ReceivedBeforeFinalization { q }.into());
                }
            }
            header.message_count as usize
        } else {
            self.messages.len()
        };
        let mut len = wire::HEADER_BYTES + source_id.len() + 8 * count;
        for (message, _) in self.messages.values() {
            len += wire::message_len(message)? - 8;
            if len > MAX_BLOCK_BYTES {
                return Err(WireError::BlockTooLarge.into());
            }
        }
        Ok(len)
    }

    fn is_complete(&self) -> bool {
        self.header
            .is_some_and(|(header, _)| header.message_count as usize == self.messages.len())
    }
}

#[derive(Clone, Debug)]
struct ReceiptTimes {
    header: TickUs,
    messages: Vec<TickUs>,
}

pub struct ReaderState {
    source_id: Vec<u8>,
    reader_id: Vec<u8>,
    blocks: Vec<FinalizedBlock>,
    receipts: Vec<ReceiptTimes>,
    pending: BTreeMap<u64, Fragments>,
    retained_bytes: usize,
}

impl ReaderState {
    pub fn new(source_id: Vec<u8>, reader_id: Vec<u8>) -> Result<Self, DeliveryError> {
        wire::validate_source(&source_id)?;
        wire::validate_source(&reader_id)?;
        Ok(Self {
            source_id,
            reader_id,
            blocks: Vec::new(),
            receipts: Vec::new(),
            pending: BTreeMap::new(),
            retained_bytes: 0,
        })
    }

    /// Caller must authenticate this finalized header and its exact message count.
    pub fn receive_header(
        &mut self,
        source_id: &[u8],
        header: BlockHeader,
        received_at: TickUs,
    ) -> Result<(), DeliveryError> {
        self.check_source(source_id)?;
        let q = header.q;
        if let Some(block) = self.visible_block(q) {
            if header != BlockHeader::from(block) {
                return Err(SourceError::ConflictingHeader { q }.into());
            }
            return Self::check_received(header, received_at);
        }
        self.update(q, |row| Self::put_header(row, header, received_at))
    }

    /// Caller must authenticate this fragment at its original public position.
    pub fn receive_message(
        &mut self,
        source_id: &[u8],
        q: u64,
        index: u32,
        message: LogMessage,
        received_at: TickUs,
    ) -> Result<(), DeliveryError> {
        self.check_source(source_id)?;
        if let Some(block) = self.visible_block(q) {
            let header = BlockHeader::from(block);
            match block.proposal.messages.get(index as usize) {
                None => return Err(SourceError::InvalidIndex { q, index }.into()),
                Some(first) if *first != message => {
                    return Err(SourceError::ConflictingMessage { q, index }.into());
                }
                Some(_) => return Self::check_received(header, received_at),
            }
        }
        self.update(q, |row| {
            Self::put_message(row, q, index, message, received_at)
        })
    }

    /// Atomic delivery of an authenticated complete block.
    pub fn receive_block(
        &mut self,
        source_id: &[u8],
        block: FinalizedBlock,
        received_at: TickUs,
    ) -> Result<(), DeliveryError> {
        self.check_source(source_id)?;
        wire::block_len(source_id, &block.proposal.messages)?;
        let header = BlockHeader::from(&block);
        let q = header.q;
        if let Some(first) = self.visible_block(q) {
            if BlockHeader::from(first) != header {
                return Err(SourceError::ConflictingHeader { q }.into());
            }
            for (index, (old, new)) in first
                .proposal
                .messages
                .iter()
                .zip(&block.proposal.messages)
                .enumerate()
            {
                if old != new {
                    return Err(SourceError::ConflictingMessage {
                        q,
                        index: index as u32,
                    }
                    .into());
                }
            }
            return Self::check_received(header, received_at);
        }
        self.update(q, |row| {
            Self::put_header(row, header, received_at)?;
            for (index, message) in block.proposal.messages.into_iter().enumerate() {
                Self::put_message(row, q, index as u32, message, received_at)?;
            }
            Ok(())
        })
    }

    /// First virtual tick by which this reader had all fragments of visible q.
    pub fn receipt(&self, q: u64) -> Option<ReaderReceipt> {
        let times = self.receipts.get(usize::try_from(q).ok()?)?;
        let reader_received_at_us = times
            .messages
            .iter()
            .copied()
            .fold(times.header, TickUs::max);
        Some(ReaderReceipt {
            q,
            reader_received_at_us,
        })
    }

    pub fn header_received_at(&self, q: u64) -> Option<TickUs> {
        let q_index = usize::try_from(q).ok()?;
        Some(self.receipts.get(q_index)?.header)
    }

    pub fn message_received_at(&self, q: u64, index: u32) -> Option<TickUs> {
        let q_index = usize::try_from(q).ok()?;
        self.receipts
            .get(q_index)?
            .messages
            .get(index as usize)
            .copied()
    }

    fn visible_block(&self, q: u64) -> Option<&FinalizedBlock> {
        self.blocks.get(usize::try_from(q).ok()?)
    }

    fn check_source(&self, source_id: &[u8]) -> Result<(), DeliveryError> {
        if source_id != self.source_id {
            return Err(SourceError::SourceMismatch.into());
        }
        Ok(())
    }

    fn check_received(header: BlockHeader, received_at: TickUs) -> Result<(), DeliveryError> {
        if received_at < header.finalized_at_us {
            return Err(SourceError::ReceivedBeforeFinalization { q: header.q }.into());
        }
        Ok(())
    }

    fn put_header(
        row: &mut Fragments,
        header: BlockHeader,
        at: TickUs,
    ) -> Result<(), DeliveryError> {
        Self::check_received(header, at)?;
        if let Some((first, _)) = row.header {
            if first != header {
                return Err(SourceError::ConflictingHeader { q: header.q }.into());
            }
        } else {
            row.header = Some((header, at));
        }
        Ok(())
    }

    fn put_message(
        row: &mut Fragments,
        q: u64,
        index: u32,
        message: LogMessage,
        at: TickUs,
    ) -> Result<(), DeliveryError> {
        if index as usize >= MAX_MESSAGES {
            return Err(SourceError::InvalidIndex { q, index }.into());
        }
        match row.messages.entry(index) {
            std::collections::btree_map::Entry::Occupied(first) if first.get().0 != message => {
                return Err(SourceError::ConflictingMessage { q, index }.into());
            }
            std::collections::btree_map::Entry::Occupied(_) => {}
            std::collections::btree_map::Entry::Vacant(slot) => {
                slot.insert((message, at));
            }
        }
        if let Some((header, _)) = row.header {
            Self::check_received(header, at)?;
        }
        Ok(())
    }

    fn update(
        &mut self,
        q: u64,
        change: impl FnOnce(&mut Fragments) -> Result<(), DeliveryError>,
    ) -> Result<(), DeliveryError> {
        if q >= MAX_BLOCKS as u64 {
            return Err(DeliveryError::Capacity);
        }
        let old = self.pending.get(&q);
        let old_len = old.map_or(Ok(0), |row| row.validate(q, &self.source_id))?;
        let mut row = old.cloned().unwrap_or_default();
        change(&mut row)?;
        let new_len = row.validate(q, &self.source_id)?;
        let retained_bytes = self.retained_bytes - old_len + new_len;
        if retained_bytes > MAX_LOG_BYTES {
            return Err(DeliveryError::Capacity);
        }
        if let Some((header, _)) = row.header {
            self.check_neighbors(header)?;
        }

        self.pending.insert(q, row);
        self.retained_bytes = retained_bytes;
        self.promote_prefix();
        Ok(())
    }

    fn check_neighbors(&self, header: BlockHeader) -> Result<(), DeliveryError> {
        use std::ops::Bound::{Excluded, Unbounded};
        let q = header.q;
        let previous = self
            .pending
            .range(..q)
            .rev()
            .find_map(|(_, row)| row.header)
            .map(|(value, _)| value)
            .or_else(|| self.blocks.last().map(BlockHeader::from));
        let next = self
            .pending
            .range((Excluded(q), Unbounded))
            .find_map(|(_, row)| row.header)
            .map(|(value, _)| value);
        for (earlier, later) in previous
            .map(|value| (value, header))
            .into_iter()
            .chain(next.map(|value| (header, value)))
        {
            if later.theta_us < earlier.theta_us {
                return Err(SourceError::Regressed {
                    earlier_q: earlier.q,
                    later_q: later.q,
                }
                .into());
            }
            if later.proposed_at_us < earlier.proposed_at_us
                || later.finalized_at_us < earlier.finalized_at_us
            {
                return Err(SourceError::EventTickRegressed {
                    earlier_q: earlier.q,
                    later_q: later.q,
                }
                .into());
            }
        }
        Ok(())
    }

    fn promote_prefix(&mut self) {
        let mut q = self.blocks.len() as u64;
        while self.pending.get(&q).is_some_and(Fragments::is_complete) {
            let row = self.pending.remove(&q).expect("complete row");
            let (header, header_tick) = row.header.expect("complete header");
            let (messages, message_ticks) = row.messages.into_values().unzip();
            self.blocks.push(FinalizedBlock {
                proposal: ProposedBlock {
                    q,
                    theta_us: header.theta_us,
                    proposed_at_us: header.proposed_at_us,
                    live_at_proposal: header.live_at_proposal,
                    messages,
                },
                finalized_at_us: header.finalized_at_us,
            });
            self.receipts.push(ReceiptTimes {
                header: header_tick,
                messages: message_ticks,
            });
            q += 1;
        }
    }

    fn wait(&self) -> ReadError {
        let next_q = self.blocks.len() as u64;
        let missing = self
            .pending
            .get(&next_q)
            .and_then(|row| row.header.map(|(header, _)| (header, row)))
            .map_or(MissingLog::Header, |(header, row)| MissingLog::Messages {
                indices: (0..header.message_count)
                    .filter(|index| !row.messages.contains_key(index))
                    .collect(),
            });
        ReadError::Wait { next_q, missing }
    }

    fn latest_prefix(&self, count: usize) -> VerifiedPrefix<'_> {
        VerifiedPrefix {
            source_id: &self.source_id,
            reader_id: &self.reader_id,
            blocks: &self.blocks[..count],
        }
    }

    /// Private reader checkpoint. Receipt times and future fragments are reader-local.
    pub fn encode_snapshot(&self) -> Vec<u8> {
        let mut out = b"PERS\x01\0".to_vec();
        wire::put_bytes(&mut out, &self.source_id);
        wire::put_bytes(&mut out, &self.reader_id);
        out.extend_from_slice(&(self.blocks.len() as u32).to_le_bytes());
        out.extend_from_slice(&(self.pending.len() as u32).to_le_bytes());
        for (block, ticks) in self.blocks.iter().zip(&self.receipts) {
            put_row(
                &mut out,
                block.proposal.q,
                Some((BlockHeader::from(block), ticks.header)),
                block
                    .proposal
                    .messages
                    .iter()
                    .zip(ticks.messages.iter().copied())
                    .enumerate()
                    .map(|(index, (msg, tick))| (index as u32, msg, tick)),
            );
        }
        for (&q, row) in &self.pending {
            put_row(
                &mut out,
                q,
                row.header,
                row.messages
                    .iter()
                    .map(|(&index, (msg, tick))| (index, msg, *tick)),
            );
        }
        out
    }

    /// Rebuild validates bounds, ordering, completeness, receipt causality and identity.
    /// The checkpoint is trusted local storage and must be authenticated by its owner.
    pub fn decode_snapshot(
        expected_source: &[u8],
        expected_reader: &[u8],
        bytes: &[u8],
    ) -> Result<Self, DeliveryError> {
        if bytes.len() > 2 * MAX_LOG_BYTES {
            return Err(DeliveryError::Capacity);
        }
        let mut decoder = Decoder(bytes);
        if decoder.take(4)? != b"PERS" {
            return Err(WireError::BadMagic.into());
        }
        if decoder.u16()? != 1 {
            return Err(WireError::UnsupportedVersion.into());
        }
        let source_id = decoder.bounded_bytes(MAX_SOURCE_BYTES, WireError::InvalidSourceId)?;
        if source_id != expected_source {
            return Err(SourceError::SourceMismatch.into());
        }
        let reader_id = decoder.bounded_bytes(MAX_SOURCE_BYTES, WireError::InvalidSourceId)?;
        if reader_id != expected_reader {
            return Err(SourceError::ReaderMismatch.into());
        }
        let mut restored = Self::new(source_id.to_vec(), reader_id.to_vec())?;
        let visible = decoder.u32()? as usize;
        let pending = decoder.u32()? as usize;
        let rows = visible
            .checked_add(pending)
            .ok_or(DeliveryError::Capacity)?;
        if rows > MAX_BLOCKS {
            return Err(DeliveryError::Capacity);
        }
        let mut previous_q = None;
        for row_number in 0..rows {
            let q = decoder.u64()?;
            if previous_q.is_some_and(|previous| q <= previous)
                || (row_number < visible && q != row_number as u64)
            {
                return Err(SourceError::InvalidSnapshot.into());
            }
            previous_q = Some(q);
            let header = if decoder.boolean()? {
                Some((
                    BlockHeader {
                        q,
                        theta_us: PublicTimeUs(decoder.u64()?),
                        proposed_at_us: TickUs(decoder.u64()?),
                        live_at_proposal: decoder.boolean()?,
                        finalized_at_us: TickUs(decoder.u64()?),
                        message_count: decoder.u32()?,
                    },
                    TickUs(decoder.u64()?),
                ))
            } else {
                None
            };
            let count = decoder.u32()? as usize;
            if count > MAX_MESSAGES {
                return Err(WireError::TooManyMessages.into());
            }
            if count == 0 && header.is_none() {
                return Err(SourceError::InvalidSnapshot.into());
            }
            restored.update(q, |row| {
                row.header = header;
                let mut previous_index = None;
                for _ in 0..count {
                    let index = decoder.u32()?;
                    if previous_index.is_some_and(|previous| index <= previous) {
                        return Err(SourceError::InvalidSnapshot.into());
                    }
                    previous_index = Some(index);
                    let tick = TickUs(decoder.u64()?);
                    Self::put_message(row, q, index, decoder.message()?, tick)?;
                }
                Ok(())
            })?;
            if row_number < visible && restored.blocks.len() != row_number + 1 {
                return Err(SourceError::InvalidSnapshot.into());
            }
        }
        if restored.blocks.len() != visible || restored.pending.len() != pending {
            return Err(SourceError::InvalidSnapshot.into());
        }
        decoder.finish()?;
        Ok(restored)
    }
}

fn put_row<'a>(
    out: &mut Vec<u8>,
    q: u64,
    header: Option<(BlockHeader, TickUs)>,
    messages: impl ExactSizeIterator<Item = (u32, &'a LogMessage, TickUs)>,
) {
    out.extend_from_slice(&q.to_le_bytes());
    out.push(u8::from(header.is_some()));
    if let Some((header, tick)) = header {
        out.extend_from_slice(&header.theta_us.0.to_le_bytes());
        out.extend_from_slice(&header.proposed_at_us.0.to_le_bytes());
        out.push(u8::from(header.live_at_proposal));
        out.extend_from_slice(&header.finalized_at_us.0.to_le_bytes());
        out.extend_from_slice(&header.message_count.to_le_bytes());
        out.extend_from_slice(&tick.0.to_le_bytes());
    }
    out.extend_from_slice(&(messages.len() as u32).to_le_bytes());
    for (index, message, tick) in messages {
        out.extend_from_slice(&index.to_le_bytes());
        out.extend_from_slice(&tick.0.to_le_bytes());
        wire::put_message(out, message);
    }
}

impl FinalizedLogReader for ReaderState {
    fn source_id(&self) -> &[u8] {
        &self.source_id
    }
    fn reader_id(&self) -> &[u8] {
        &self.reader_id
    }

    fn tip(&self) -> Option<(u64, PublicTimeUs)> {
        self.blocks
            .last()
            .map(|block| (block.proposal.q, block.proposal.theta_us))
    }

    fn read(&self, q: u64) -> Result<&FinalizedBlock, ReadError> {
        usize::try_from(q)
            .ok()
            .and_then(|index| self.blocks.get(index))
            .ok_or_else(|| self.wait())
    }

    fn require_prefix_through(&self, q: u64) -> Result<VerifiedPrefix<'_>, ReadError> {
        self.read(q)?;
        Ok(self.latest_prefix(q as usize + 1))
    }

    fn require_watermark(&self, x: PublicTimeUs) -> Result<VerifiedPrefix<'_>, ReadError> {
        if self.tip().is_some_and(|(_, theta)| theta >= x) {
            Ok(self.latest_prefix(self.blocks.len()))
        } else {
            Err(self.wait())
        }
    }
}
