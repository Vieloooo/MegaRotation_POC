use prost::Message;
use sha2::{Digest, Sha256};
pub mod domain;
pub mod wire {
    tonic::include_proto!("rotation.i1");
}
pub use wire::*;
pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_FRAME_BYTES: usize = 1_048_576;
pub const MAX_PAYLOAD_BYTES: usize = 65_536;
pub const LINK_QUEUE_BOUND: usize = 64;
/// Domain and every component are framed with a big-endian u64 byte length.
pub fn hash_parts(domain: &str, parts: &[&[u8]]) -> Vec<u8> {
    let mut h = Sha256::new();
    for bytes in std::iter::once(domain.as_bytes()).chain(parts.iter().copied()) {
        h.update((bytes.len() as u64).to_be_bytes());
        h.update(bytes);
    }
    h.finalize().to_vec()
}
pub fn digest<M: Message>(domain: &str, message: &M) -> Vec<u8> {
    hash_parts(domain, &[&message.encode_to_vec()])
}
pub fn grant_hash(grant: &Grant) -> Vec<u8> {
    let mut g = grant.clone();
    g.grant_hash.clear();
    g.incarnation = 0;
    digest("I1/grant/v1", &g)
}
pub fn ack_hash(ack: &Ack) -> Vec<u8> {
    let mut a = ack.clone();
    a.logical_ack_hash.clear();
    a.incarnation = 0;
    digest("I1/ack/v1", &a)
}
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
