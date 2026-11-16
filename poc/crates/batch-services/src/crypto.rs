//! Ethereum EIP-712 signing and verification for the feature 09 Inbox.

pub use l1_adapter::{BatchWire, CursorWire};
use poc_types::domain::Hash32;
use secp256k1::{
    Message, PublicKey, Secp256k1, SecretKey,
    ecdsa::{RecoverableSignature, RecoveryId},
};
use serde::{Deserialize, Serialize};

use crate::frame::keccak;

pub type Address = [u8; 20];
pub type Signature = [u8; 65];

const DOMAIN: &str = "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract,bytes32 salt)";
const INBOX: &str = "SequencerRotationBatchInbox";
const CURSOR: &str =
    "Cursor(uint64 height,uint64 origin,bytes32 framesDigest,uint8 sourceKind,bytes source)";
const DA: &str = "DAReceipt(bytes32 commitment)";
const ATTESTATION: &str = "BatchAttestation(bytes32 prevHash,bytes32 nextHash,bytes32 commitment,bytes32 daCertificateHash,uint64 firstHeight,uint64 lastHeight,bytes32 gposHash)";
const SUBMIT_BATCH: &str = "submitBatch(bytes32,((uint64,uint64,bytes32,uint8,bytes),(uint64,uint64,bytes32,uint8,bytes),bytes32,bytes,uint64,uint64,bytes,bytes))";

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InboxDomain {
    pub chain_id: u64,
    pub inbox: Address,
    pub config_ref: Hash32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InboxSigners {
    pub da: Address,
    pub certifier: Address,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CryptoError {
    InvalidEncoding(&'static str),
    InvalidDomain,
    InvalidCursor,
    InvalidRange,
    InvalidSecretKey,
    InvalidSignature,
    InvalidSigner,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CryptoError {}

/// Local signing capability. Its secret is omitted from logs and serialized evidence.
pub struct SigningKey(SecretKey);

impl SigningKey {
    pub fn new(key: [u8; 32]) -> Result<Self, CryptoError> {
        SecretKey::from_byte_array(key)
            .map(Self)
            .map_err(|_| CryptoError::InvalidSecretKey)
    }

    pub fn address(&self) -> Address {
        address_of(&PublicKey::from_secret_key(
            &Secp256k1::signing_only(),
            &self.0,
        ))
    }

    pub fn sign_digest(&self, digest: Hash32) -> Result<Signature, CryptoError> {
        let signed = Secp256k1::signing_only()
            .sign_ecdsa_recoverable(Message::from_digest(digest.into_bytes()), &self.0);
        let (recovery_id, raw) = signed.serialize_compact();
        let v = i32::from(recovery_id);
        if v > 1 {
            return Err(CryptoError::InvalidSignature);
        }
        let mut signature = [0; 65];
        signature[..64].copy_from_slice(&raw);
        signature[64] = 27 + v as u8;
        Ok(signature)
    }
}

pub fn recover_signer(digest: Hash32, signature: &[u8]) -> Result<Address, CryptoError> {
    let signature: &[u8; 65] = signature
        .try_into()
        .map_err(|_| CryptoError::InvalidSignature)?;
    let v = match signature[64] {
        27 => RecoveryId::Zero,
        28 => RecoveryId::One,
        _ => return Err(CryptoError::InvalidSignature),
    };
    // from_compact bounds both scalars. Ethereum additionally disallows zero and high s.
    if signature[..32] == [0; 32] || signature[32..64] == [0; 32] {
        return Err(CryptoError::InvalidSignature);
    }
    let signature = RecoverableSignature::from_compact(&signature[..64], v)
        .map_err(|_| CryptoError::InvalidSignature)?;
    let standard = signature.to_standard();
    let mut normalized = standard;
    normalized.normalize_s();
    if standard != normalized {
        return Err(CryptoError::InvalidSignature);
    }
    let public = Secp256k1::verification_only()
        .recover_ecdsa(Message::from_digest(digest.into_bytes()), &signature)
        .map_err(|_| CryptoError::InvalidSignature)?;
    let address = address_of(&public);
    if address == [0; 20] {
        return Err(CryptoError::InvalidSigner);
    }
    Ok(address)
}

fn address_of(key: &PublicKey) -> Address {
    keccak(&key.serialize_uncompressed()[1..])[12..]
        .try_into()
        .expect("fixed address width")
}

pub fn check_da(
    commitment: Hash32,
    receipt: &[u8],
    domain: &InboxDomain,
    expected_signer: Address,
) -> Result<(), CryptoError> {
    check_signer(domain.da_digest(commitment)?, receipt, expected_signer)
}

/// Check the complete attestation against its configured certifier.
/// Call `check_da` as well when authenticating an Inbox submission.
pub fn verify_batch(
    batch: &BatchWire,
    domain: &InboxDomain,
    expected_signer: Address,
) -> Result<(), CryptoError> {
    check_signer(
        domain.batch_digest(batch)?,
        &decode_fixed::<65>(&batch.certifier_signature, "certifier signature")?,
        expected_signer,
    )
}

fn check_signer(digest: Hash32, signature: &[u8], expected: Address) -> Result<(), CryptoError> {
    if recover_signer(digest, signature)? != expected {
        return Err(CryptoError::InvalidSigner);
    }
    Ok(())
}

impl InboxDomain {
    pub fn separator(&self) -> Result<Hash32, CryptoError> {
        if self.inbox == [0; 20] || self.config_ref.as_bytes() == &[0; 32] {
            return Err(CryptoError::InvalidDomain);
        }
        let mut address = [0; 32];
        address[12..].copy_from_slice(&self.inbox);
        Ok(words_hash(&[
            keccak(DOMAIN.as_bytes()),
            keccak(INBOX.as_bytes()),
            keccak(b"1"),
            uint_word(self.chain_id),
            address,
            self.config_ref.into_bytes(),
        ]))
    }

    pub fn da_digest(&self, commitment: Hash32) -> Result<Hash32, CryptoError> {
        self.digest(words_hash(&[
            keccak(DA.as_bytes()),
            commitment.into_bytes(),
        ]))
    }

    pub fn batch_digest(&self, batch: &BatchWire) -> Result<Hash32, CryptoError> {
        let commitment = decode_fixed(&batch.commitment, "commitment")?;
        let receipt = decode_fixed::<65>(&batch.da_certificate, "DA certificate")?;
        let gpos = decode_hex(&batch.gpos, 1024, "gpos")?;
        self.digest(words_hash(&[
            keccak(ATTESTATION.as_bytes()),
            cursor_hash(&batch.prev)?.into_bytes(),
            cursor_hash(&batch.next)?.into_bytes(),
            commitment,
            keccak(&receipt),
            uint_word(batch.first_height),
            uint_word(batch.last_height),
            keccak(&gpos),
        ]))
    }

    fn digest(&self, body: Hash32) -> Result<Hash32, CryptoError> {
        let mut input = [0; 66];
        input[..2].copy_from_slice(&[0x19, 0x01]);
        input[2..34].copy_from_slice(self.separator()?.as_bytes());
        input[34..].copy_from_slice(body.as_bytes());
        Ok(Hash32::new(keccak(&input)))
    }
}

pub fn cursor_hash(cursor: &CursorWire) -> Result<Hash32, CryptoError> {
    Ok(words_hash(&[
        keccak(CURSOR.as_bytes()),
        uint_word(cursor.height),
        uint_word(cursor.origin),
        decode_fixed(&cursor.frames_digest, "frames digest")?,
        uint_word(cursor.source_kind.into()),
        keccak(&decode_hex(&cursor.source, 1024, "cursor source")?),
    ]))
}

pub fn validate_submission(batch: &BatchWire) -> Result<(), CryptoError> {
    validate_cursor(&batch.prev)?;
    validate_cursor(&batch.next)?;
    let gpos = decode_hex(&batch.gpos, 1024, "gpos")?;
    if batch.next.source_kind != 1 || decode_hex(&batch.next.source, 1024, "cursor source")? != gpos
    {
        return Err(CryptoError::InvalidCursor);
    }
    if batch.prev.height.checked_add(1) != Some(batch.first_height)
        || batch.last_height != batch.next.height
        || !matches!(
            batch.last_height.checked_sub(batch.first_height),
            Some(0 | 1)
        )
    {
        return Err(CryptoError::InvalidRange);
    }
    Ok(())
}

pub(crate) fn validate_cursor(cursor: &CursorWire) -> Result<(), CryptoError> {
    let source = decode_hex(&cursor.source, 1024, "cursor source")?;
    let source_ok = match cursor.source_kind {
        0 => source.is_empty(),
        1 => !source.is_empty(),
        2 => source.len() == 32 && source[..24] == [0; 24],
        _ => false,
    };
    if !source_ok || decode_fixed::<32>(&cursor.frames_digest, "frames digest")? != [0; 32] {
        return Err(CryptoError::InvalidCursor);
    }
    Ok(())
}

/// Canonical calldata for the existing 09 BatchInbox ABI.
pub fn submit_batch_calldata(batch: &BatchWire) -> Result<Vec<u8>, CryptoError> {
    validate_submission(batch)?;
    let rid = decode_fixed::<32>(&batch.request_id, "request id")?;
    let d = decode_fixed::<32>(&batch.commitment, "commitment")?;
    let mut body = vec![0; 8 * 32];
    body[64..96].copy_from_slice(&d);
    body[128..160].copy_from_slice(&uint_word(batch.first_height));
    body[160..192].copy_from_slice(&uint_word(batch.last_height));
    for (at, tail) in [
        (0, encode_cursor(&batch.prev)?),
        (1, encode_cursor(&batch.next)?),
        (
            3,
            encode_bytes(&decode_fixed::<65>(
                &batch.da_certificate,
                "DA certificate",
            )?),
        ),
        (6, encode_bytes(&decode_hex(&batch.gpos, 1024, "gpos")?)),
        (
            7,
            encode_bytes(&decode_fixed::<65>(
                &batch.certifier_signature,
                "certifier signature",
            )?),
        ),
    ] {
        let offset = uint_word(body.len() as u64);
        body[at * 32..(at + 1) * 32].copy_from_slice(&offset);
        body.extend(tail);
    }
    let mut calldata = Vec::with_capacity(4 + 64 + body.len());
    calldata.extend(&keccak(SUBMIT_BATCH.as_bytes())[..4]);
    calldata.extend(rid);
    calldata.extend(uint_word(64));
    calldata.extend(body);
    Ok(calldata)
}

fn encode_cursor(cursor: &CursorWire) -> Result<Vec<u8>, CryptoError> {
    let mut bytes = Vec::new();
    bytes.extend(uint_word(cursor.height));
    bytes.extend(uint_word(cursor.origin));
    bytes.extend(decode_fixed::<32>(&cursor.frames_digest, "frames digest")?);
    bytes.extend(uint_word(cursor.source_kind.into()));
    bytes.extend(uint_word(160));
    bytes.extend(encode_bytes(&decode_hex(
        &cursor.source,
        1024,
        "cursor source",
    )?));
    Ok(bytes)
}

fn encode_bytes(value: &[u8]) -> Vec<u8> {
    let mut bytes = uint_word(value.len() as u64).to_vec();
    bytes.extend(value);
    bytes.resize(32 + value.len().next_multiple_of(32), 0);
    bytes
}

fn words_hash(words: &[[u8; 32]]) -> Hash32 {
    Hash32::new(keccak(words.as_flattened()))
}

pub(crate) fn uint_word(value: u64) -> [u8; 32] {
    let mut word = [0; 32];
    word[24..].copy_from_slice(&value.to_be_bytes());
    word
}

pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut output = String::with_capacity(2 + 2 * bytes.len());
    output.push_str("0x");
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String");
    }
    output
}

pub fn decode_hex(input: &str, limit: usize, label: &'static str) -> Result<Vec<u8>, CryptoError> {
    let err = CryptoError::InvalidEncoding(label);
    let raw = input
        .strip_prefix("0x")
        .ok_or_else(|| err.clone())?
        .as_bytes();
    if raw.len() % 2 != 0 || raw.len() / 2 > limit {
        return Err(err);
    }
    fn digit(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _ => None,
        }
    }
    raw.chunks_exact(2)
        .map(|pair| {
            digit(pair[0])
                .zip(digit(pair[1]))
                .map(|(a, b)| a * 16 + b)
                .ok_or_else(|| err.clone())
        })
        .collect()
}

pub fn decode_fixed<const N: usize>(
    input: &str,
    label: &'static str,
) -> Result<[u8; N], CryptoError> {
    decode_hex(input, N, label)?
        .try_into()
        .map_err(|_| CryptoError::InvalidEncoding(label))
}
