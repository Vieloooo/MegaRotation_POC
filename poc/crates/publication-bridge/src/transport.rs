use std::collections::BTreeMap;

use poc_publisher::{AttemptId, Submission};
use poc_types::domain::{Canonical, Hash32, ResultKey};
use sim_espresso::{
    LogMessage, MAX_BLOCK_BYTES, MAX_SOURCE_BYTES, decode_finalized, encode_finalized,
    validate_message_list,
};
use sim_transport::{ChannelId, Delivery, EndpointBinding, Id};

use crate::{
    BridgeError, SubmissionVerifier,
    bounded::{Reader, digest, identity},
};

const HEADER: &[u8] = b"H3TF\0\x01";
const HEADER_SIZE: usize = 83;
const CHUNK: usize = 61_440;
const MAX_CHUNKS: usize = 1024;
const MAX_COMPLETE: usize = 4096;
const MAX_PARTIAL: usize = 4;
const SUBMISSION: u8 = 0;
const FINALIZED: u8 = 1;

pub fn attempt_payload_id(
    source_id: &[u8],
    messages: &[LogMessage],
) -> Result<Hash32, BridgeError> {
    validate_message_list(source_id, messages).map_err(|_| BridgeError::Capacity)?;
    if messages.is_empty() {
        return Err(BridgeError::InvalidResult);
    }
    let count = u32::try_from(messages.len())
        .map_err(|_| BridgeError::Capacity)?
        .to_be_bytes();
    let fields: Vec<_> = messages
        .iter()
        .enumerate()
        .map(|(i, message)| ((i as u32).to_be_bytes(), message.codec.to_be_bytes()))
        .collect();
    let mut parts: Vec<&[u8]> = vec![source_id, &count];
    for ((ordinal, codec), message) in fields.iter().zip(messages) {
        parts.extend([
            ordinal.as_slice(),
            codec.as_slice(),
            message.bytes.as_slice(),
        ]);
    }
    Ok(digest("H3/AttemptPayload/v1", &parts))
}

pub fn submission_binding_id(
    run: Id,
    config: Id,
    source_id: &[u8],
    id: &AttemptId,
    key: ResultKey,
    payload_id: Hash32,
    ordinal: u32,
    codec: u32,
) -> Result<Hash32, BridgeError> {
    identity(source_id, MAX_SOURCE_BYTES)?;
    identity(&id.namespace, MAX_SOURCE_BYTES)?;
    if id.task_number == 0 || id.attempt_number == 0 || codec == 0 {
        return Err(BridgeError::InvalidResult);
    }
    Ok(digest(
        "H3/SubmissionBinding/v1",
        &[
            &run,
            &config,
            source_id,
            &id.namespace,
            &id.task_number.to_be_bytes(),
            &id.attempt_number.to_be_bytes(),
            &key.encode(),
            payload_id.as_bytes(),
            &ordinal.to_be_bytes(),
            &codec.to_be_bytes(),
        ],
    ))
}

pub fn finalized_block_binding_id(
    run: Id,
    config: Id,
    source_id: &[u8],
    q: u64,
) -> Result<Hash32, BridgeError> {
    identity(source_id, MAX_SOURCE_BYTES)?;
    Ok(digest(
        "H3/FinalizedBlockBinding/v1",
        &[&run, &config, source_id, &q.to_be_bytes()],
    ))
}

fn object_digest(kind: u8, binding: Hash32, object: &[u8]) -> Hash32 {
    digest(
        "H3/TransportObject/v1",
        &[&[kind], binding.as_bytes(), object],
    )
}

fn limits(total: usize, payload_limit: usize) -> Result<(usize, usize), BridgeError> {
    if total == 0 {
        return Err(BridgeError::InvalidResult);
    }
    let chunk = payload_limit
        .checked_sub(HEADER_SIZE)
        .filter(|n| *n != 0)
        .ok_or(BridgeError::Capacity)?
        .min(CHUNK);
    if total > MAX_BLOCK_BYTES {
        return Err(BridgeError::Capacity);
    }
    let pieces = total.div_ceil(chunk);
    if pieces > MAX_CHUNKS {
        return Err(BridgeError::Capacity);
    }
    Ok((chunk, pieces))
}

pub fn fragment(
    kind: u8,
    binding_id: Hash32,
    object: &[u8],
    payload_limit: usize,
) -> Result<Vec<Vec<u8>>, BridgeError> {
    if !matches!(kind, SUBMISSION | FINALIZED) {
        return Err(BridgeError::InvalidResult);
    }
    let (chunk, count) = limits(object.len(), payload_limit)?;
    let hash = object_digest(kind, binding_id, object);
    Ok(object
        .chunks(chunk)
        .enumerate()
        .map(|(i, bytes)| {
            let mut out = Vec::with_capacity(HEADER_SIZE + bytes.len());
            out.extend_from_slice(HEADER);
            out.push(kind);
            out.extend_from_slice(binding_id.as_bytes());
            out.extend_from_slice(hash.as_bytes());
            out.extend_from_slice(&(object.len() as u32).to_be_bytes());
            out.extend_from_slice(&(i as u16).to_be_bytes());
            out.extend_from_slice(&(count as u16).to_be_bytes());
            out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
            out.extend_from_slice(bytes);
            out
        })
        .collect())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReassemblyRoute {
    pub run: Id,
    pub config: Id,
    pub source: EndpointBinding,
    pub destination: EndpointBinding,
    pub channel: ChannelId,
    pub context: Option<Id>,
    pub codec: Id,
    pub payload_limit: usize,
}

/// Checks the actual owner-provided route/custody fact for this delivery.
pub trait DeliveryVerifier {
    fn verify_delivery(&self, delivery: &Delivery) -> Result<(), BridgeError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reassembled {
    pub kind: u8,
    pub binding_id: Hash32,
    pub bytes: Vec<u8>,
}

pub struct Reassembler<'a> {
    routes: Vec<ReassemblyRoute>,
    delivery: &'a dyn DeliveryVerifier,
    registered: BTreeMap<ObjectKey, Expected>,
    partial: BTreeMap<ObjectKey, Partial>,
    complete: BTreeMap<ObjectKey, Complete>,
}

type ObjectKey = (usize, u8, Hash32);

#[derive(PartialEq, Eq)]
enum Expected {
    Submission { digest: Hash32, length: usize },
    Finalized { source_id: Vec<u8>, q: u64 },
}

struct Partial {
    object_digest: Hash32,
    total: usize,
    chunks: Vec<Option<Vec<u8>>>,
    received: usize,
}

struct Complete {
    object_digest: Hash32,
    total: usize,
    chunk_ids: Vec<Hash32>,
}

struct Frame<'a> {
    kind: u8,
    binding: Hash32,
    object_digest: Hash32,
    total: usize,
    index: usize,
    count: usize,
    chunk: &'a [u8],
}

impl<'a> Reassembler<'a> {
    pub fn new(
        routes: Vec<ReassemblyRoute>,
        delivery: &'a dyn DeliveryVerifier,
    ) -> Result<Self, BridgeError> {
        if routes.is_empty() {
            return Err(BridgeError::InvalidConfig);
        }
        for (index, route) in routes.iter().enumerate() {
            limits(1, route.payload_limit)?;
            if route.source.endpoint == route.destination.endpoint
                || route.source.incarnation == 0
                || route.destination.incarnation == 0
                || route.codec == [0; 32]
                || route.context == Some([0; 32])
                || routes[..index]
                    .iter()
                    .any(|old| old.identity() == route.identity())
            {
                return Err(BridgeError::InvalidConfig);
            }
        }
        Ok(Self {
            routes,
            delivery,
            registered: BTreeMap::new(),
            partial: BTreeMap::new(),
            complete: BTreeMap::new(),
        })
    }

    pub fn register_submission(
        &mut self,
        route: &ReassemblyRoute,
        source_id: &[u8],
        submission: &Submission<ResultKey>,
        ordinal: u32,
        owner: &dyn SubmissionVerifier,
    ) -> Result<Hash32, BridgeError> {
        let route_index = self.route(route)?;
        crate::custody::validate_attempt_limits(source_id, submission)?;
        let message = submission
            .exact_payload
            .get(ordinal as usize)
            .ok_or(BridgeError::InvalidResult)?;
        limits(message.bytes.len(), route.payload_limit)?;
        let payload = attempt_payload_id(source_id, &submission.exact_payload)?;
        let id = submission_binding_id(
            route.run,
            route.config,
            source_id,
            &submission.stable_id,
            submission.key,
            payload,
            ordinal,
            message.codec,
        )?;
        owner.verify_submission(source_id, submission)?;
        self.register(
            (route_index, SUBMISSION, id),
            Expected::Submission {
                digest: object_digest(SUBMISSION, id, &message.bytes),
                length: message.bytes.len(),
            },
        )?;
        Ok(id)
    }

    pub fn register_finalized(
        &mut self,
        route: &ReassemblyRoute,
        source_id: &[u8],
        q: u64,
    ) -> Result<Hash32, BridgeError> {
        let id = finalized_block_binding_id(route.run, route.config, source_id, q)?;
        self.register(
            (self.route(route)?, FINALIZED, id),
            Expected::Finalized {
                source_id: source_id.to_vec(),
                q,
            },
        )?;
        Ok(id)
    }

    fn route(&self, route: &ReassemblyRoute) -> Result<usize, BridgeError> {
        self.routes
            .iter()
            .position(|expected| expected == route)
            .ok_or(BridgeError::SourceMismatch)
    }

    fn register(&mut self, key: ObjectKey, item: Expected) -> Result<(), BridgeError> {
        if let Some(old) = self.registered.get(&key) {
            return if *old == item {
                Ok(())
            } else {
                Err(BridgeError::InvalidResult)
            };
        }
        if self.registered.len() >= MAX_COMPLETE + MAX_PARTIAL {
            return Err(BridgeError::Capacity);
        }
        self.registered.insert(key, item);
        Ok(())
    }

    pub fn receive(&mut self, delivery: &Delivery) -> Result<Option<Reassembled>, BridgeError> {
        let message = &delivery.envelope.message;
        let route = self
            .routes
            .iter()
            .position(|route| route.matches(delivery))
            .ok_or(BridgeError::SourceMismatch)?;
        let frame = Frame::decode(&message.payload, self.routes[route].payload_limit)?;
        let key = (route, frame.kind, frame.binding);
        let expected = self
            .registered
            .get(&key)
            .ok_or(BridgeError::SourceMismatch)?;
        if let Expected::Submission { digest, length } = expected
            && (*digest != frame.object_digest || *length != frame.total)
        {
            return Err(BridgeError::InvalidResult);
        }
        message
            .validate_hashes()
            .map_err(|_| BridgeError::InvalidResult)?;
        self.delivery.verify_delivery(delivery)?;
        if let Some(done) = self.complete.get(&key) {
            if done.object_digest != frame.object_digest
                || done.total != frame.total
                || done.chunk_ids.get(frame.index) != Some(&chunk_id(frame.chunk))
            {
                return Err(BridgeError::InvalidResult);
            }
            return Ok(None);
        }
        let prior = self.partial.get(&key);
        if let Some(partial) = prior {
            if partial.object_digest != frame.object_digest
                || partial.total != frame.total
                || partial.chunks.len() != frame.count
            {
                return Err(BridgeError::InvalidResult);
            }
            if let Some(bytes) = &partial.chunks[frame.index] {
                return if bytes == frame.chunk {
                    Ok(None)
                } else {
                    Err(BridgeError::InvalidResult)
                };
            }
        }
        let already = prior.map_or(0, |item| item.received);
        if already + 1 == frame.count {
            if self.complete.len() >= MAX_COMPLETE {
                return Err(BridgeError::Capacity);
            }
            let mut object = Vec::with_capacity(frame.total);
            let mut chunk_ids = Vec::with_capacity(frame.count);
            for index in 0..frame.count {
                let bytes = if index == frame.index {
                    frame.chunk
                } else {
                    prior
                        .and_then(|item| item.chunks[index].as_deref())
                        .ok_or(BridgeError::InvalidResult)?
                };
                object.extend_from_slice(bytes);
                chunk_ids.push(chunk_id(bytes));
            }
            if object.len() != frame.total
                || object_digest(frame.kind, frame.binding, &object) != frame.object_digest
            {
                return Err(BridgeError::InvalidResult);
            }
            if let Expected::Finalized { source_id, q } = expected {
                let (source, block) =
                    decode_finalized(&object).map_err(|_| BridgeError::InvalidEncoding)?;
                if source != *source_id
                    || block.proposal.q != *q
                    || encode_finalized(&source, &block)
                        .map_err(|_| BridgeError::InvalidEncoding)?
                        != object
                {
                    return Err(BridgeError::SourceMismatch);
                }
            }
            self.partial.remove(&key);
            self.complete.insert(
                key,
                Complete {
                    object_digest: frame.object_digest,
                    total: frame.total,
                    chunk_ids,
                },
            );
            return Ok(Some(Reassembled {
                kind: frame.kind,
                binding_id: frame.binding,
                bytes: object,
            }));
        }
        if prior.is_none() {
            let destination = self.routes[route].destination.endpoint;
            let count = self
                .partial
                .keys()
                .filter(|(index, _, _)| self.routes[*index].destination.endpoint == destination)
                .count();
            // Each declared object is at most 1 MiB, so these count bounds also
            // enforce the per-destination 2 MiB and global 4 MiB limits.
            if self.partial.len() >= MAX_PARTIAL || count >= 2 {
                return Err(BridgeError::Capacity);
            }
        }
        let partial = self.partial.entry(key).or_insert_with(|| Partial {
            object_digest: frame.object_digest,
            total: frame.total,
            chunks: vec![None; frame.count],
            received: 0,
        });
        partial.chunks[frame.index] = Some(frame.chunk.to_vec());
        partial.received += 1;
        Ok(None)
    }
}

impl ReassemblyRoute {
    fn identity(
        &self,
    ) -> (
        Id,
        Id,
        EndpointBinding,
        EndpointBinding,
        ChannelId,
        Option<Id>,
        Id,
    ) {
        (
            self.run,
            self.config,
            self.source,
            self.destination,
            self.channel,
            self.context,
            self.codec,
        )
    }

    fn matches(&self, delivery: &Delivery) -> bool {
        let env = &delivery.envelope;
        let message = &env.message;
        (
            message.run,
            message.config,
            EndpointBinding {
                endpoint: message.source,
                incarnation: env.source_incarnation,
            },
            EndpointBinding {
                endpoint: message.destination,
                incarnation: env.destination_incarnation,
            },
            message.channel,
            message.context,
            message.codec,
        ) == self.identity()
            && message.origin.source_incarnation != 0
            && message.origin.source_incarnation <= env.source_incarnation
            && message.origin.step_id != 0
            && message.origin.sent_tick_us <= delivery.delivered_tick_us
            && env.attempt != 0
            && delivery.packet.logical_id == message.logical_id
            && delivery.packet.attempt == env.attempt
            && delivery.raw_due_us <= delivery.eligible_tick_us
            && delivery.eligible_tick_us <= delivery.delivered_tick_us
    }
}

impl<'a> Frame<'a> {
    fn decode(bytes: &'a [u8], payload_limit: usize) -> Result<Self, BridgeError> {
        if bytes.len() > payload_limit {
            return Err(BridgeError::Capacity);
        }
        let mut input = Reader(bytes);
        if input.take(HEADER.len())? != HEADER {
            return Err(BridgeError::InvalidEncoding);
        }
        let kind = input.tag()?;
        if !matches!(kind, SUBMISSION | FINALIZED) {
            return Err(BridgeError::InvalidEncoding);
        }
        let binding = Hash32::new(input.fixed()?);
        let object_digest = Hash32::new(input.fixed()?);
        let total = input.count(MAX_BLOCK_BYTES)?;
        let index = usize::from(u16::from_be_bytes(input.fixed()?));
        let count = usize::from(u16::from_be_bytes(input.fixed()?));
        let (chunk, canonical_count) = limits(total, payload_limit)?;
        if count != canonical_count || index >= count {
            return Err(BridgeError::InvalidEncoding);
        }
        let actual = input.frame(CHUNK)?;
        let expected = chunk.min(total - chunk * index);
        if actual.len() != expected {
            return Err(BridgeError::InvalidEncoding);
        }
        input.finish()?;
        Ok(Self {
            kind,
            binding,
            object_digest,
            total,
            index,
            count,
            chunk: actual,
        })
    }
}

fn chunk_id(chunk: &[u8]) -> Hash32 {
    digest("H3/TransportChunk/v1", &[chunk])
}
