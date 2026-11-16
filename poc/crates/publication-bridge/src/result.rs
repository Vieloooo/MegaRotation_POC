use std::sync::Arc;

use poc_publisher::{CodecError, StateCodec};
use poc_types::domain::{
    AnchorEvidence, AnchorEvidenceSource, L1Position, Canonical, CanonicalObject, ConfigRef, Descriptor,
    InputMissing, PhysicalBlock, RegionalResult, ResultKey,
};
use poc_viewer_gfr::{
    MAX_DESCRIPTOR_BYTES, MAX_PROOF_BYTES, PUBLICATION_CODEC, PublicObject, encode_publication,
};
use sim_espresso::{LogMessage, validate_message_list};

use crate::BridgeError;

const STORE_HEADER: &[u8] = b"H3RS\0\x01";
const MAX_STORE_BYTES: usize = 128 * 1024;
const MAX_CHAIN: usize = 10;
const MAX_TRANSFERS: usize = 4;
const BODY_PREFIX: usize = 94;
const MAX_BODY_BYTES: usize = BODY_PREFIX + MAX_TRANSFERS * (4 + 19);
const MAX_ANCHOR_OBJECTS: usize = 16;
const MAX_ANCHOR_OBJECT_BYTES: usize = 4096;
const MAX_ANCHOR_BYTES: usize = 65_536;

/// Private, immutable H3-M1-SMALL-1 input, including partial Offer material.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BridgeResult(Arc<StoredResult>);

#[derive(Debug, PartialEq, Eq)]
struct StoredResult {
    result: RegionalResult,
    key: ResultKey,
    bytes: Vec<u8>,
}

impl BridgeResult {
    pub fn new(result: RegionalResult) -> Result<Self, BridgeError> {
        bound(result.final_proof.len(), MAX_PROOF_BYTES)?;
        bound(result.chain.len(), MAX_CHAIN)?;
        bound(
            result.anchor_evidence.source_objects.len(),
            MAX_ANCHOR_OBJECTS,
        )?;
        check_anchor_lengths(result.anchor_evidence.source_objects.iter().map(Vec::len))?;
        for block in std::iter::once(&result.block).chain(&result.chain) {
            bound(block.transactions.len(), MAX_TRANSFERS)?;
            block.validate()?;
        }
        if let Some((tail, earlier)) = result.chain.split_last()
            && (tail != &result.block || earlier.contains(&result.block))
        {
            return Err(BridgeError::InvalidResult);
        }
        let (tag, reference) = match &result.anchor_evidence.source {
            AnchorEvidenceSource::Genesis(reference) => (0, reference.encode()),
            AnchorEvidenceSource::Regional(reference) => (1, reference.encode()),
            AnchorEvidenceSource::Resume(reference) => (2, reference.encode()),
            _ => return Err(BridgeError::InvalidResult),
        };
        let descriptor = result.descriptor.encode_checked()?;
        bound(descriptor.len(), MAX_DESCRIPTOR_BYTES)?;
        let target = result.block.encode_checked()?;
        let chain = result
            .chain
            .iter()
            .map(PhysicalBlock::encode_checked)
            .collect::<Result<Vec<_>, _>>()?;
        let key = ResultKey {
            cid: result.descriptor.cid,
            block_hash: result.block.digest()?,
        };

        // Three u32/scalar fields outside the frames: chain count, source tag,
        // source-object count. Frame lengths and the total are checked together.
        let mut frame_lengths = [
            descriptor.len(),
            target.len(),
            result.final_proof.len(),
            reference.len(),
        ]
        .into_iter()
        .chain(chain.iter().map(Vec::len))
        .chain(result.anchor_evidence.source_objects.iter().map(Vec::len));
        let capacity = frame_lengths
            .try_fold(STORE_HEADER.len() + 4 + 1 + 4, |total, len| {
                total.checked_add(4)?.checked_add(len)
            })
            .ok_or(BridgeError::Capacity)?;
        bound(capacity, MAX_STORE_BYTES)?;
        let mut bytes = Vec::with_capacity(capacity);
        bytes.extend_from_slice(STORE_HEADER);
        put_frame(&mut bytes, &descriptor)?;
        put_frame(&mut bytes, &target)?;
        put_frame(&mut bytes, &result.final_proof)?;
        put_count(&mut bytes, chain.len())?;
        for block in chain {
            put_frame(&mut bytes, &block)?;
        }
        bytes.push(tag);
        put_frame(&mut bytes, &reference)?;
        put_count(&mut bytes, result.anchor_evidence.source_objects.len())?;
        for object in &result.anchor_evidence.source_objects {
            put_frame(&mut bytes, object)?;
        }
        Ok(Self(Arc::new(StoredResult { result, key, bytes })))
    }

    pub fn result(&self) -> &RegionalResult {
        &self.0.result
    }

    pub fn key(&self) -> ResultKey {
        self.0.key
    }

    pub fn encode_store(&self) -> Vec<u8> {
        self.0.bytes.clone()
    }

    pub(crate) fn store_bytes(&self) -> &[u8] {
        &self.0.bytes
    }

    pub fn decode_store(bytes: &[u8]) -> Result<Self, BridgeError> {
        bound(bytes.len(), MAX_STORE_BYTES)?;
        let mut input = Decoder(
            bytes
                .strip_prefix(STORE_HEADER)
                .ok_or(BridgeError::InvalidEncoding)?,
        );
        let descriptor = input.frame(MAX_DESCRIPTOR_BYTES)?;
        let target = input.body_frame()?;
        let proof = input.frame(MAX_PROOF_BYTES)?;
        let chain_len = input.count(MAX_CHAIN)?;
        let mut chain = [Default::default(); MAX_CHAIN];
        for body in &mut chain[..chain_len] {
            *body = input.body_frame()?;
        }
        let tag = input.take(1)?[0];
        let source = match tag {
            0 => AnchorEvidenceSource::Genesis(ConfigRef::decode(input.frame(34)?)?),
            1 => AnchorEvidenceSource::Regional(ResultKey::decode(input.frame(66)?)?),
            2 => AnchorEvidenceSource::Resume(L1Position::decode(input.frame(26)?)?),
            _ => return Err(BridgeError::InvalidEncoding),
        };
        let anchor_len = input.count(MAX_ANCHOR_OBJECTS)?;
        let mut objects = [Default::default(); MAX_ANCHOR_OBJECTS];
        for object in &mut objects[..anchor_len] {
            *object = input.frame(MAX_ANCHOR_OBJECT_BYTES)?;
        }
        check_anchor_lengths(objects[..anchor_len].iter().map(|object| object.len()))?;
        if !input.0.is_empty() {
            return Err(BridgeError::InvalidEncoding);
        }

        let result = Self::new(RegionalResult {
            descriptor: Descriptor::decode(descriptor)?,
            block: PhysicalBlock::decode(target)?,
            final_proof: proof.to_vec(),
            chain: chain[..chain_len]
                .iter()
                .map(|body| PhysicalBlock::decode(body))
                .collect::<Result<_, _>>()?,
            anchor_evidence: AnchorEvidence {
                source,
                source_objects: objects[..anchor_len]
                    .iter()
                    .map(|object| object.to_vec())
                    .collect(),
            },
        })?;
        if result.store_bytes() != bytes {
            return Err(BridgeError::InvalidEncoding);
        }
        Ok(result)
    }

    /// Check the declared order and identify a missing first physical ancestor.
    pub(crate) fn missing_ancestry(&self) -> Result<Option<InputMissing>, BridgeError> {
        let result = self.result();
        let anchor = &result.descriptor.anchor;
        let anchor_id = anchor.block.block_id;
        let window = &result.descriptor.plan.window;
        let heights = window.h_min..=window.h_max;
        if !heights.contains(&result.block.metadata.height)
            || self.key().block_hash == anchor_id
            || (result.block.metadata.height == window.h_min && result.block.parent != anchor_id)
            || result.block.parent == self.key().block_hash
        {
            return Err(BridgeError::InvalidResult);
        }
        let Some(first) = result.chain.first() else {
            return Ok(Some(InputMissing::Parent(self.key().block_hash)));
        };
        let mut previous = None;
        for block in &result.chain {
            let hash = block.digest()?;
            if !heights.contains(&block.metadata.height)
                || hash == anchor_id
                || hash == first.parent
                || previous.is_some_and(|(height, hash)| {
                    block.metadata.height <= height || block.parent != hash
                })
            {
                return Err(BridgeError::InvalidResult);
            }
            previous = Some((block.metadata.height, hash));
        }
        if first.parent == anchor_id {
            Ok(None)
        } else if first.metadata.height == window.h_min {
            Err(BridgeError::InvalidResult)
        } else {
            Ok(Some(InputMissing::Parent(first.parent)))
        }
    }
}

pub struct BridgeStateCodec;

impl StateCodec<ResultKey, BridgeResult> for BridgeStateCodec {
    fn encode_key(&self, key: &ResultKey) -> Vec<u8> {
        key.encode()
    }

    fn decode_key(&self, bytes: &[u8]) -> Result<ResultKey, CodecError> {
        ResultKey::decode(bytes).map_err(|_| CodecError::InvalidKey)
    }

    fn encode_result(&self, result: &BridgeResult) -> Vec<u8> {
        result.encode_store()
    }

    fn decode_result(&self, bytes: &[u8]) -> Result<BridgeResult, CodecError> {
        BridgeResult::decode_store(bytes).map_err(|_| CodecError::InvalidResult)
    }
}

/// Encode a complete structural result with the existing public message codec.
pub fn publication_messages(
    result: &BridgeResult,
    source_id: &[u8],
) -> Result<Vec<LogMessage>, BridgeError> {
    sim_espresso::validate_message_list(source_id, &[]).map_err(|_| BridgeError::SourceMismatch)?;
    if result.missing_ancestry()?.is_some() || result.result().final_proof.is_empty() {
        return Err(BridgeError::InvalidResult);
    }
    let mut messages = Vec::new();
    for block in &result.result().chain {
        messages.push(publication_message(PublicObject::Body(block.clone()))?);
    }
    messages.push(publication_message(PublicObject::FinalProof {
        descriptor: result.result().descriptor.clone(),
        target: result.key().block_hash,
        proof: result.result().final_proof.clone(),
    })?);
    validate_message_list(source_id, &messages).map_err(|_| BridgeError::Capacity)?;
    Ok(messages)
}

fn publication_message(object: PublicObject) -> Result<LogMessage, BridgeError> {
    Ok(LogMessage {
        codec: PUBLICATION_CODEC,
        bytes: encode_publication(&object)?,
    })
}

fn check_anchor_lengths(lengths: impl Iterator<Item = usize>) -> Result<(), BridgeError> {
    let mut total = 0usize;
    for len in lengths {
        bound(len, MAX_ANCHOR_OBJECT_BYTES)?;
        total = total.checked_add(len).ok_or(BridgeError::Capacity)?;
    }
    bound(total, MAX_ANCHOR_BYTES)
}

fn bound(actual: usize, max: usize) -> Result<(), BridgeError> {
    if actual > max {
        Err(BridgeError::Capacity)
    } else {
        Ok(())
    }
}

fn put_count(out: &mut Vec<u8>, count: usize) -> Result<(), BridgeError> {
    let count = u32::try_from(count).map_err(|_| BridgeError::Capacity)?;
    out.extend_from_slice(&count.to_be_bytes());
    Ok(())
}

fn put_frame(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), BridgeError> {
    put_count(out, bytes.len())?;
    out.extend_from_slice(bytes);
    Ok(())
}

struct Decoder<'a>(&'a [u8]);

impl<'a> Decoder<'a> {
    fn take(&mut self, len: usize) -> Result<&'a [u8], BridgeError> {
        let (head, tail) = self
            .0
            .split_at_checked(len)
            .ok_or(BridgeError::InvalidEncoding)?;
        self.0 = tail;
        Ok(head)
    }

    fn count(&mut self, max: usize) -> Result<usize, BridgeError> {
        let bytes = self
            .take(4)?
            .try_into()
            .map_err(|_| BridgeError::InvalidEncoding)?;
        let count =
            usize::try_from(u32::from_be_bytes(bytes)).map_err(|_| BridgeError::Capacity)?;
        bound(count, max)?;
        Ok(count)
    }

    fn frame(&mut self, max: usize) -> Result<&'a [u8], BridgeError> {
        let len = self.count(max)?;
        self.take(len)
    }

    fn body_frame(&mut self) -> Result<&'a [u8], BridgeError> {
        let body = self.frame(MAX_BODY_BYTES)?;
        let prefix = body
            .get(..BODY_PREFIX)
            .ok_or(BridgeError::InvalidEncoding)?;
        // Preflight the nested03 transaction count before its decoder allocates.
        Decoder(&prefix[BODY_PREFIX - 4..]).count(MAX_TRANSFERS)?;
        Ok(body)
    }
}
