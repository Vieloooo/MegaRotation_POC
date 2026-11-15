use crate::{
    Id, hash_parts,
    native_wire::{Ack, Envelope, Grant},
};
use prost::Message;
pub const VERSION: u32 = 1;
pub const MAX_FRAME: usize = 1_048_576;
pub const MAX_PAYLOAD: usize = 65_536;
pub fn grant_hash(value: &Grant) -> Id {
    let mut value = value.clone();
    value.grant_hash.clear();
    hash_parts(b"I1b/fresh-grant/v1", &[&value.encode_to_vec()])
}
pub fn ack_hash(value: &Ack) -> Id {
    let mut value = value.clone();
    value.logical_ack_hash.clear();
    value.durable_step_hash.clear();
    hash_parts(b"I1b/fresh-ack/v1", &[&value.encode_to_vec()])
}
pub fn wire_hash(channel: u32, bytes: &[u8]) -> Id {
    hash_parts(b"I1b/native-wire/v1", &[&channel.to_be_bytes(), bytes])
}
pub fn message_hash(config: &[u8; 32], value: &Envelope) -> Id {
    hash_parts(
        b"I1b/native-message/v1",
        &[
            &value.run_id,
            config,
            &value.cid,
            &value.epoch.to_be_bytes(),
            &value.src_member_id.to_be_bytes(),
            &value.dst_member_id.to_be_bytes(),
            &(value.channel as u32).to_be_bytes(),
            &value.payload_hash,
        ],
    )
}

pub fn tie_hash(seed: u64, event_id: u64) -> Id {
    hash_parts(
        b"I1b/fresh-tie/v1",
        &[&seed.to_be_bytes(), &event_id.to_be_bytes()],
    )
}
