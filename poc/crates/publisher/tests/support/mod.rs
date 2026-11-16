//! Non-cryptographic component fixture for the real Publisher adapter traits.
#![allow(dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use poc_publisher::*;
use sim_espresso::*;

pub const SOURCE: &[u8] = b"S";
pub const READER: &[u8] = b"publisher-reader";
pub const TEST_CODEC: u32 = 7;
const PROOF: u8 = b'P';
const BODY: u8 = b'B';
const ANCESTOR: u8 = b'A';

pub type Key = [u8; 2];
pub const KEY: Key = [9, 2];
pub const OTHER: Key = [9, 3];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureResult {
    pub key: Key,
    pub claimed_cid: u8,
    pub proof: u8,
    pub body: u8,
    pub anchor: u8,
    pub parent: u8,
}

impl FixtureResult {
    pub fn valid(key: Key) -> Self {
        Self {
            key,
            claimed_cid: key[0],
            proof: 1,
            body: key[1],
            anchor: 4,
            parent: 1,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ReceiptFault {
    None,
    Key,
    Source,
    Index,
    PublicationAfterCompleteness,
    Future,
}

pub struct FixtureInputs {
    pub config: bool,
    pub anchor: bool,
    pub ancestor: bool,
    pub receipt_fault: ReceiptFault,
}

impl Default for FixtureInputs {
    fn default() -> Self {
        Self {
            config: true,
            anchor: true,
            ancestor: true,
            receipt_fault: ReceiptFault::None,
        }
    }
}

pub struct NonCryptographicFixture;

impl StateCodec<Key, FixtureResult> for NonCryptographicFixture {
    fn encode_key(&self, key: &Key) -> Vec<u8> {
        key.to_vec()
    }
    fn decode_key(&self, bytes: &[u8]) -> Result<Key, CodecError> {
        bytes.try_into().map_err(|_| CodecError::InvalidKey)
    }
    fn encode_result(&self, value: &FixtureResult) -> Vec<u8> {
        vec![
            value.key[0],
            value.key[1],
            value.claimed_cid,
            value.proof,
            value.body,
            value.anchor,
            value.parent,
        ]
    }
    fn decode_result(&self, bytes: &[u8]) -> Result<FixtureResult, CodecError> {
        let &[cid, hash, claimed_cid, proof, body, anchor, parent] = bytes else {
            return Err(CodecError::InvalidResult);
        };
        Ok(FixtureResult {
            key: [cid, hash],
            claimed_cid,
            proof,
            body,
            anchor,
            parent,
        })
    }
}

fn rejection(code: u32, reference: &[u8]) -> RejectReason {
    RejectReason {
        code,
        reference: reference.to_vec(),
    }
}

pub fn missing(code: u32, reference: &[u8]) -> MissingInput {
    MissingInput {
        code,
        reference: reference.to_vec(),
    }
}

pub fn proof(key: Key, variant: u8) -> LogMessage {
    message(&[PROOF, key[0], key[1], variant])
}

pub fn body(key: Key) -> LogMessage {
    message(&[BODY, key[0], key[1]])
}

pub fn ancestor(key: Key) -> LogMessage {
    message(&[ANCESTOR, key[0], 1, 4])
}

pub fn message(bytes: &[u8]) -> LogMessage {
    LogMessage {
        codec: TEST_CODEC,
        bytes: bytes.to_vec(),
    }
}

impl PublicationVerifier for NonCryptographicFixture {
    type Key = Key;
    type Result = FixtureResult;
    type PublicInputs = FixtureInputs;

    fn key(&self, result: &FixtureResult) -> Result<Key, RejectReason> {
        Ok(result.key)
    }

    fn verify_offer(
        &self,
        result: FixtureResult,
        inputs: &FixtureInputs,
    ) -> OfferVerdict<Key, FixtureResult> {
        let key = result.key;
        if result.claimed_cid != key[0] {
            return OfferVerdict::Reject(rejection(1, b"cid"));
        }
        if result.body != key[1] {
            return OfferVerdict::Reject(rejection(2, b"body"));
        }
        if result.proof != 1 && result.proof != 2 {
            return OfferVerdict::Reject(rejection(3, b"proof"));
        }
        if result.parent != 1 || result.anchor != 4 {
            return OfferVerdict::Reject(rejection(4, b"ancestry"));
        }
        let mut missing_inputs = Vec::new();
        for (supplied, code, label) in [
            (inputs.config, 1, b"config".as_slice()),
            (inputs.anchor, 2, b"anchor".as_slice()),
            (inputs.ancestor, 3, b"ancestor".as_slice()),
        ] {
            if !supplied {
                missing_inputs.push(missing(code, label));
            }
        }
        if !missing_inputs.is_empty() {
            return OfferVerdict::Wait {
                key,
                partial_result: result,
                missing: missing_inputs,
                verified_deadline: inputs.config.then_some(PublicTimeUs(105)),
            };
        }
        OfferVerdict::Verified {
            key,
            publication_messages: vec![proof(key, result.proof), body(key), ancestor(key)],
            complete_result: result,
            validation_basis: b"non-crypto:matched symbolic proof/body/ancestry/anchor".to_vec(),
            deadline: PublicTimeUs(105),
        }
    }

    fn verify_recorded(
        &self,
        task: &PublicationTask<Key, FixtureResult>,
        prefix: &VerifiedPrefix<'_>,
        inputs: &FixtureInputs,
    ) -> RecordVerdict<Key> {
        let key = task.key;
        let mut first_proof = None;
        let mut body_q = None;
        let mut ancestor_q = None;
        for block in prefix.blocks() {
            let q = block.proposal.q;
            for (index, message) in block.proposal.messages.iter().enumerate() {
                if *message == proof(key, 1) || *message == proof(key, 2) {
                    first_proof.get_or_insert(EspressoPosition {
                        q,
                        message_index: index as u32,
                    });
                } else if *message == body(key) {
                    body_q.get_or_insert(q);
                } else if *message == ancestor(key) {
                    ancestor_q.get_or_insert(q);
                }
            }
        }
        let Some(position) = first_proof else {
            return RecordVerdict::NotYetPublic;
        };
        let (Some(body_q), Some(ancestor_q)) = (body_q, ancestor_q) else {
            let mut absent = Vec::new();
            if body_q.is_none() {
                absent.push(missing(4, b"public body"));
            }
            if ancestor_q.is_none() {
                absent.push(missing(5, b"public ancestor"));
            }
            return RecordVerdict::Wait { missing: absent };
        };
        let mut receipt = RecordedAt {
            publication_position: position,
            complete_through_q: position.q.max(body_q).max(ancestor_q),
            source_id: prefix.source_id().to_vec(),
            verification_basis: b"non-crypto:located first proof and both bodies".to_vec(),
        };
        let mut observed = key;
        match inputs.receipt_fault {
            ReceiptFault::None => (),
            ReceiptFault::Key => observed[0] ^= 1,
            ReceiptFault::Source => receipt.source_id = b"foreign".to_vec(),
            ReceiptFault::Index => receipt.publication_position.message_index = 1024,
            ReceiptFault::PublicationAfterCompleteness => {
                receipt.publication_position.q = prefix.tip().0 + 1;
            }
            ReceiptFault::Future => receipt.complete_through_q = prefix.tip().0 + 1,
        }
        RecordVerdict::Recorded {
            key: observed,
            receipt,
        }
    }
}

pub type FixturePublisher = Publisher<NonCryptographicFixture, NonCryptographicFixture>;

pub fn binding() -> StoreBinding {
    StoreBinding {
        namespace: b"fixture-publisher".to_vec(),
        configuration: b"fixture-config-v1".to_vec(),
        source_id: SOURCE.to_vec(),
        reader_id: READER.to_vec(),
    }
}

pub fn create(path: &Path) -> FixturePublisher {
    Publisher::create(
        path,
        binding(),
        NonCryptographicFixture,
        NonCryptographicFixture,
    )
    .unwrap()
}

pub fn reopen(path: &Path) -> FixturePublisher {
    Publisher::open(
        path,
        binding(),
        NonCryptographicFixture,
        NonCryptographicFixture,
    )
    .unwrap()
}

pub fn log() -> SimEspresso {
    SimEspresso::new(SOURCE.to_vec(), TimestampPolicy::Fixed { theta_us: 104 }).unwrap()
}

pub fn reader(id: &[u8]) -> ReaderState {
    ReaderState::new(SOURCE.to_vec(), id.to_vec()).unwrap()
}

pub fn produce(
    log: &mut SimEspresso,
    proposal: u64,
    theta_us: u64,
    messages: Vec<LogMessage>,
) -> FinalizedBlock {
    log.set_policy(TimestampPolicy::Fixed { theta_us });
    let q = log.propose(TickUs(proposal), true, messages).unwrap().q;
    log.finalize(q, TickUs(proposal + 1)).unwrap().clone()
}

pub struct EvidenceDirectory {
    pub path: PathBuf,
    preserve: bool,
}

impl EvidenceDirectory {
    pub fn new(name: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let configured = std::env::var_os("PE_TEST_EVIDENCE_DIR");
        let preserve = configured.is_some();
        let parent = configured.map_or_else(std::env::temp_dir, PathBuf::from);
        let prefix = format!("pe-{name}-{}", std::process::id());
        loop {
            let number = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!("{prefix}-{number}"));
            match fs::create_dir(&path) {
                Ok(()) => return Self { path, preserve },
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (),
                Err(error) => panic!("cannot create {}: {error}", path.display()),
            }
        }
    }
}

impl Drop for EvidenceDirectory {
    fn drop(&mut self) {
        if !self.preserve {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
