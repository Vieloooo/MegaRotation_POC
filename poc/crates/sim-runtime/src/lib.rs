use poc_types::*;
use serde::{Deserialize, Serialize};
use std::{error::Error, io};

pub mod journal;
mod state;

pub use state::State;
pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub fn invalid(message: impl Into<String>) -> Box<dyn Error + Send + Sync> {
    io::Error::new(io::ErrorKind::InvalidData, message.into()).into()
}

pub fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(invalid(message))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Identity {
    pub run_id: Vec<u8>,
    pub config_hash: Vec<u8>,
    pub node_id: u32,
    pub epsilon_us: i64,
}

impl Identity {
    pub fn validate(&self) -> Result<()> {
        require(
            self.run_id.len() == 32 && self.config_hash.len() == 32,
            "invalid launch hashes",
        )?;
        require(
            self.node_id < 2 && self.epsilon_us.unsigned_abs() <= 1000,
            "invalid oscillator",
        )
    }

    pub fn clock_domain(&self) -> Vec<u8> {
        hash_parts(
            "I1/clock-domain/v1",
            &[&self.run_id, &self.config_hash, &self.node_id.to_be_bytes()],
        )
    }

    pub fn local_time(&self, tick_us: u64) -> Result<i64> {
        i64::try_from(tick_us)?
            .checked_add(self.epsilon_us)
            .ok_or_else(|| invalid("oscillator overflow"))
    }

    pub fn validate_grant(&self, incarnation: u64, grant: &Grant) -> Result<()> {
        require(
            grant.schema_version == SCHEMA_VERSION,
            "unknown grant schema",
        )?;
        require(
            grant.run_id == self.run_id
                && grant.node_id == self.node_id
                && grant.incarnation == incarnation,
            "wrong grant identity",
        )?;
        require(
            grant.logical_step_id > 0 && grant.event_id > 0 && grant.tie_rank.len() == 32,
            "invalid grant coordinates",
        )?;
        require(
            grant.grant_hash == grant_hash(grant),
            "grant digest mismatch",
        )?;
        require(
            grant
                .input
                .as_ref()
                .is_some_and(|input| input.kind.is_some()),
            "absent grant input",
        )
    }

    pub fn validate_ack(&self, incarnation: u64, ack: &Ack) -> Result<()> {
        require(ack.schema_version == SCHEMA_VERSION, "unknown Ack schema")?;
        require(
            ack.run_id == self.run_id
                && ack.node_id == self.node_id
                && ack.incarnation == incarnation,
            "wrong Ack identity",
        )?;
        require(
            ack.logical_step_id > 0 && ack.event_id > 0 && ack.durable_journal_seq > 0,
            "invalid Ack coordinates",
        )?;
        require(
            ack.grant_hash.len() == 32
                && ack.state_hash.len() == 32
                && ack.logical_ack_hash == ack_hash(ack),
            "invalid Ack digest",
        )?;
        AckStatus::try_from(ack.status)?;
        require(
            ack.effects.iter().all(|effect| effect.kind.is_some()),
            "absent Ack effect",
        )
    }
}

pub fn rate_profile() -> Vec<u8> {
    hash_parts("I1/rate/v1", &[b"fixed-rate-one-v1"])
}

pub fn intent_hash(identity: &Identity, intent: &SignedIntent) -> Vec<u8> {
    hash_parts(
        "I1/intent/v1",
        &[
            &identity.run_id,
            &identity.config_hash,
            &intent.cid,
            &identity.node_id.to_be_bytes(),
            &intent.slot.to_be_bytes(),
            &intent.subject,
        ],
    )
}

pub fn payload_hash(payload: &[u8]) -> Vec<u8> {
    hash_parts("I1/payload/v1", &[payload])
}

pub fn message_id(envelope: &Envelope) -> Vec<u8> {
    hash_parts(
        "I1/message/v1",
        &[
            &envelope.run_id,
            &envelope.src.to_be_bytes(),
            &envelope.dst.to_be_bytes(),
            &envelope.channel.to_be_bytes(),
            &envelope.cid,
            &envelope.sent_at_tick_us.to_be_bytes(),
            &envelope.payload_hash,
        ],
    )
}

pub fn envelope_error(envelope: &Envelope, run_id: &[u8]) -> Option<&'static str> {
    if envelope.run_id != run_id
        || envelope.src > 1
        || envelope.dst > 1
        || envelope.src == envelope.dst
    {
        return Some("EnvelopeIdentity");
    }
    if !envelope.cid.is_empty() && envelope.cid.len() != 32 {
        return Some("EnvelopeCid");
    }
    if envelope.payload_bytes.len() > MAX_PAYLOAD_BYTES {
        return Some("EnvelopeSize");
    }
    if envelope.payload_hash != payload_hash(&envelope.payload_bytes)
        || envelope.message_id != message_id(envelope)
    {
        return Some("EnvelopeDigest");
    }
    None
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Checkpoint {
    BeforeAppend,
    PartialTail,
    AfterRecordSyncBeforeHead,
    AfterSyncBeforeAck,
    AfterHubCommit,
}

impl Checkpoint {
    pub fn exit_code(self) -> i32 {
        match self {
            Self::BeforeAppend => 71,
            Self::PartialTail => 72,
            Self::AfterRecordSyncBeforeHead => 75,
            Self::AfterSyncBeforeAck => 73,
            Self::AfterHubCommit => 74,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Retry {
    pub logical_step_id: u64,
    pub grant_hash: Vec<u8>,
    pub original_incarnation: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Launch {
    pub identity: Identity,
    pub incarnation: u64,
    pub journal: std::path::PathBuf,
    pub wall_pause_ms: u64,
    pub faults: Vec<(u64, Checkpoint)>,
    pub retry: Option<Retry>,
}
