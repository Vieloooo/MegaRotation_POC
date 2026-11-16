//! EC16-NATIVE-1 observations from the pinned local Espresso API.
#![forbid(unsafe_code)]

mod bundle;
mod reader;

pub use bundle::{BUNDLE_SCHEMA, CaptureBundle, Publication, ReplaySession, replay_bundle};

pub use reader::{NativePrefix, OfficialCaptureReader, ReceiveOutcome};

use std::{fmt, sync::Arc};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub use sim_espresso::{EspressoPosition, LogMessage, PublicTimeUs};

pub const OFFICIAL_RELEASE: &str = "20260918";
pub const OFFICIAL_COMMIT: &str = "fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171";
pub const IMAGE_INDEX_DIGEST: &str =
    "sha256:af73f55630690caa35aa0bc015d60dc03333791f966fdcc07b0798e33d4f768e";
pub const IMAGE_AMD64_DIGEST: &str =
    "sha256:cfb845e49cb5369f77cd8bbda0b4fe4aad340793cfd130b7f2bb631e59d2db66";
pub const PROTOCOL_VERSION: &str = "0.3";
pub const API_VERSION: &str = "v1";
pub const PUBLICATION_NAMESPACE: u64 = 160_016;
pub const FIXTURE_CODEC: u32 = 0xEC16_0001;
pub const MAX_CAPTURE_FILE_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_CAPTURE_BUNDLE_BYTES: usize = 256 * 1024 * 1024;
const SOURCE_SCHEMA: &str = "EC16-SOURCE-1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceIdentity {
    pub schema: String,
    pub official_commit: String,
    pub image_amd64_digest: String,
    pub protocol_version: String,
    pub api_version: String,
    pub namespace: u64,
    pub run_id: String,
    pub genesis_block_hash: String,
}

impl SourceIdentity {
    pub fn pinned(run_id: impl Into<String>, genesis_block_hash: impl Into<String>) -> Self {
        Self {
            schema: SOURCE_SCHEMA.into(),
            official_commit: OFFICIAL_COMMIT.into(),
            image_amd64_digest: IMAGE_AMD64_DIGEST.into(),
            protocol_version: PROTOCOL_VERSION.into(),
            api_version: API_VERSION.into(),
            namespace: PUBLICATION_NAMESPACE,
            run_id: run_id.into(),
            genesis_block_hash: genesis_block_hash.into(),
        }
    }

    pub fn source_id(&self) -> Result<String, Error> {
        if self.schema != SOURCE_SCHEMA
            || self.official_commit != OFFICIAL_COMMIT
            || self.image_amd64_digest != IMAGE_AMD64_DIGEST
            || self.protocol_version != PROTOCOL_VERSION
            || self.api_version != API_VERSION
            || self.namespace != PUBLICATION_NAMESPACE
        {
            return Err(Error::InvalidSource(
                "pinned source profile mismatch".into(),
            ));
        }
        if !valid_id(&self.run_id) || !tagged(&self.genesis_block_hash, "BLOCK~") {
            return Err(Error::InvalidSource(
                "invalid run ID or genesis hash".into(),
            ));
        }
        let preimage = format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
            self.schema,
            self.official_commit,
            self.image_amd64_digest,
            self.protocol_version,
            self.api_version,
            self.namespace,
            self.run_id,
            self.genesis_block_hash,
        );
        Ok(format!("EC16:{}", sha256_hex(preimage.as_bytes())))
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    use fmt::Write;
    let mut hex = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(hex, "{byte:02x}").expect("writing to String cannot fail");
    }
    hex
}

pub(crate) fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

pub(crate) fn tagged(value: &str, tag: &str) -> bool {
    value.strip_prefix(tag).is_some_and(|suffix| {
        !suffix.is_empty()
            && suffix
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    })
}

pub(crate) fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureKind {
    Official,
    Fixture,
}

/// A recording binding identifies a run. It does not authenticate a recording.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceBinding {
    identity: SourceIdentity,
    source_id: String,
    capture_manifest_sha256: String,
    capture_kind: CaptureKind,
}

impl SourceBinding {
    pub fn new(
        identity: SourceIdentity,
        capture_manifest_sha256: impl Into<String>,
        capture_kind: CaptureKind,
    ) -> Result<Self, Error> {
        let capture_manifest_sha256 = capture_manifest_sha256.into();
        if !valid_digest(&capture_manifest_sha256) {
            return Err(Error::InvalidRecording(
                "invalid capture manifest SHA-256".into(),
            ));
        }
        let source_id = identity.source_id()?;
        Ok(Self {
            identity,
            source_id,
            capture_manifest_sha256,
            capture_kind,
        })
    }

    pub fn identity(&self) -> &SourceIdentity {
        &self.identity
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn capture_manifest_sha256(&self) -> &str {
        &self.capture_manifest_sha256
    }

    pub fn capture_kind(&self) -> CaptureKind {
        self.capture_kind
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureRecordRef {
    pub reader_id: String,
    pub record_id: u64,
    pub path: String,
}

/// One request outcome delivered to a reader. HTTP/transport time stays in the
/// recording metadata. A transport failure has no HTTP status.
#[derive(Clone, Copy, Debug)]
pub struct CaptureRecord<'a> {
    pub source_id: &'a str,
    pub reader_id: &'a str,
    pub id: u64,
    pub method: &'a str,
    pub path: &'a str,
    pub status: Option<u16>,
    pub response: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FinalityBasis {
    TrustedLocalOfficialApi {
        binding: Arc<SourceBinding>,
        leaf_record_ref: CaptureRecordRef,
        block_record_ref: CaptureRecordRef,
    },
    Fixture {
        binding: Arc<SourceBinding>,
        leaf_record_ref: CaptureRecordRef,
        block_record_ref: CaptureRecordRef,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub enum Unknown {
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct UnknownTiming {
    pub proposed_at: Unknown,
    pub finalized_at: Unknown,
    pub live_at_proposal: Unknown,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct NativePosition {
    pub q: u64,
    pub source_transaction_index: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedMessage {
    pub position: EspressoPosition,
    pub source_position: NativePosition,
    pub message: LogMessage,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeTransaction {
    pub position: NativePosition,
    pub namespace: u64,
    pub payload: Vec<u8>,
    pub transaction_hash: String,
    pub block_hash: String,
    pub pos_in_namespace: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeBlock {
    pub q: u64,
    pub theta: PublicTimeUs,
    pub timestamp_seconds: u64,
    pub timestamp_millis: Option<u64>,
    pub block_hash: String,
    pub ordered_transactions: Vec<NativeTransaction>,
    pub complete_count: u32,
    pub finality_basis: FinalityBasis,
    pub timing: UnknownTiming,
    pub transaction_refs: Vec<CaptureRecordRef>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissingData {
    pub block: bool,
    pub leaf: bool,
    pub transaction_indices: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    Wait { next_q: u64, missing: MissingData },
    InvalidRecording(String),
    InvalidSource(String),
    UnsupportedRepresentation(String),
    Capacity(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wait { next_q, missing } => write!(f, "WAIT at q={next_q}: {missing:?}"),
            Self::InvalidRecording(reason) => write!(f, "invalid recording: {reason}"),
            Self::InvalidSource(reason) => write!(f, "invalid source: {reason}"),
            Self::UnsupportedRepresentation(reason) => {
                write!(f, "unsupported representation: {reason}")
            }
            Self::Capacity(bound) => write!(f, "capacity exceeded: {bound}"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnsupportedTimingEvidence {
    pub missing: [&'static str; 3],
}

const UNSUPPORTED_TIMING: UnsupportedTimingEvidence = UnsupportedTimingEvidence {
    missing: ["proposed_at", "finalized_at", "live_at_proposal"],
};

pub fn export_pe_log() -> Result<Vec<u8>, UnsupportedTimingEvidence> {
    Err(UNSUPPORTED_TIMING)
}

pub fn export_pe_reader() -> Result<sim_espresso::ReaderState, UnsupportedTimingEvidence> {
    Err(UNSUPPORTED_TIMING)
}
