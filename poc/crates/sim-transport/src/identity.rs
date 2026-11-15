use crate::Error;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub type Id = [u8; 32];
pub type Tick = u64;
pub type LocalTime = i64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Endpoint {
    Replica(u32),
    Service(u32),
    Reader(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum EndpointKind {
    Replica,
    Service,
    Reader,
}

impl Endpoint {
    pub(crate) fn kind(self) -> EndpointKind {
        match self {
            Self::Replica(_) => EndpointKind::Replica,
            Self::Service(_) => EndpointKind::Service,
            Self::Reader(_) => EndpointKind::Reader,
        }
    }

    pub(crate) fn hash_bytes(self) -> [u8; 5] {
        let (tag, id) = match self {
            Self::Replica(id) => (1, id),
            Self::Service(id) => (2, id),
            Self::Reader(id) => (3, id),
        };
        let mut bytes = [tag, 0, 0, 0, 0];
        bytes[1..].copy_from_slice(&id.to_be_bytes());
        bytes
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChannelId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointBinding {
    pub endpoint: Endpoint,
    pub incarnation: u64,
}

pub(crate) fn rebind(current: u64, expected: u64, next: u64) -> Result<u64, Error> {
    if current != expected {
        return Err(Error::BindingMismatch);
    }
    if current.checked_add(1).ok_or(Error::ArithmeticOverflow)? != next {
        return Err(Error::BindingMismatch);
    }
    Ok(next)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepStamp {
    pub logical_step_id: u64,
    pub tick_us: Tick,
    pub event_id: u64,
    pub causal_wave: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Origin {
    pub step_id: u64,
    pub effect_index: u32,
    pub sent_tick_us: Tick,
    pub source_incarnation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    pub run: Id,
    pub config: Id,
    pub source: Endpoint,
    pub destination: Endpoint,
    pub channel: ChannelId,
    pub context: Option<Id>,
    pub codec: Id,
    pub origin: Origin,
    pub payload_hash: Id,
    pub logical_id: Id,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub message: Message,
    pub attempt: u32,
    pub source_incarnation: u64,
    pub destination_incarnation: u64,
}

pub fn framed_hash(domain: &str, parts: &[&[u8]]) -> Id {
    let mut hash = Sha256::new();
    for part in std::iter::once(domain.as_bytes()).chain(parts.iter().copied()) {
        hash.update((part.len() as u64).to_be_bytes());
        hash.update(part);
    }
    hash.finalize().into()
}

pub fn payload_hash(payload: &[u8]) -> Id {
    framed_hash("SIMT/payload/v1", &[payload])
}

pub fn message_id(message: &Message) -> Id {
    message.header().hash_id()
}

impl Message {
    pub fn validate_hashes(&self) -> Result<(), Error> {
        if self.payload_hash != payload_hash(&self.payload) || self.logical_id != message_id(self) {
            return Err(Error::HashMismatch);
        }
        Ok(())
    }

    pub(crate) fn header(&self) -> MessageHeader {
        MessageHeader {
            run: self.run,
            config: self.config,
            source: self.source,
            destination: self.destination,
            channel: self.channel,
            context: self.context,
            codec: self.codec,
            origin: self.origin,
            payload_hash: self.payload_hash,
            logical_id: self.logical_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MessageHeader {
    pub run: Id,
    pub config: Id,
    pub source: Endpoint,
    pub destination: Endpoint,
    pub channel: ChannelId,
    pub context: Option<Id>,
    pub codec: Id,
    pub origin: Origin,
    pub payload_hash: Id,
    pub logical_id: Id,
}

impl MessageHeader {
    pub(crate) fn hash_id(&self) -> Id {
        let mut context = [0; 33];
        let context = if let Some(id) = self.context {
            context[0] = 1;
            context[1..].copy_from_slice(&id);
            &context[..]
        } else {
            &context[..1]
        };
        framed_hash(
            "SIMT/message/v1",
            &[
                &self.run,
                &self.config,
                &self.source.hash_bytes(),
                &self.destination.hash_bytes(),
                &self.channel.0.to_be_bytes(),
                context,
                &self.codec,
                &self.origin.step_id.to_be_bytes(),
                &self.origin.effect_index.to_be_bytes(),
                &self.origin.sent_tick_us.to_be_bytes(),
                &self.origin.source_incarnation.to_be_bytes(),
                &self.payload_hash,
            ],
        )
    }

    pub(crate) fn with_payload(&self, payload: Vec<u8>) -> Message {
        Message {
            run: self.run,
            config: self.config,
            source: self.source,
            destination: self.destination,
            channel: self.channel,
            context: self.context,
            codec: self.codec,
            origin: self.origin,
            payload_hash: self.payload_hash,
            logical_id: self.logical_id,
            payload,
        }
    }
}
