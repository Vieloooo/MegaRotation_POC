use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{
    CaptureKind, CaptureRecord, Error, IMAGE_AMD64_DIGEST, IMAGE_INDEX_DIGEST,
    MAX_CAPTURE_BUNDLE_BYTES, MAX_CAPTURE_FILE_BYTES, NativePosition, OFFICIAL_COMMIT,
    OfficialCaptureReader, PUBLICATION_NAMESPACE, ReceiveOutcome, SourceBinding, SourceIdentity,
    reader::{Endpoint, decode_payload, endpoint, strict_json},
    sha256_hex, valid_digest, valid_id,
};

pub const BUNDLE_SCHEMA: &str = "EC16-CAPTURE-1";
const IMAGE_CONFIG_DIGEST: &str =
    "sha256:b376be602febaa35fd38c1f4bc1144fb79017bfed274a4a26ed90bf052bfd06f";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    path: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedRequest {
    id: u64,
    reader_id: String,
    source_id: String,
    method: String,
    path: String,
    status: Option<u16>,
    request: Option<RawFile>,
    response: Option<RawFile>,
    observation: Map<String, Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    capture_kind: CaptureKind,
    identity: SourceIdentity,
    source_id: String,
    configuration: Map<String, Value>,
    artifacts: Vec<RawFile>,
    records: Vec<RecordedRequest>,
}

#[derive(Debug, Serialize)]
pub struct Publication {
    pub submit_record_id: u64,
    pub transaction_hash: String,
    pub payload: Vec<u8>,
    pub positions: Vec<NativePosition>,
}

pub struct ReplaySession {
    pub reader: OfficialCaptureReader,
    pub publications: Vec<Publication>,
    pub unavailable_records: Vec<u64>,
    pub rejected_submissions: Vec<(u64, u16)>,
}

struct VerifiedFile {
    bytes: Vec<u8>,
    sha256: String,
}

/// An offline package with every referenced file checked against its recorded
/// SHA-256. The package has no signatures and authenticates no network peer.
pub struct CaptureBundle {
    binding: SourceBinding,
    manifest: Manifest,
    files: BTreeMap<PathBuf, VerifiedFile>,
    paths: BTreeMap<String, PathBuf>,
}

impl CaptureBundle {
    pub fn open(directory: impl AsRef<Path>) -> Result<Self, Error> {
        let directory = canonicalize(directory.as_ref())?;
        let manifest_path = file_in_directory(&directory, "manifest.json")?;
        let raw = read_bounded(&manifest_path, MAX_CAPTURE_FILE_BYTES, "manifest")?;
        let manifest_sha256 = sha256_hex(&raw);
        let manifest: Manifest = serde_json::from_value(strict_json(&raw).map_err(manifest_error)?)
            .map_err(manifest_error)?;
        if manifest.schema != BUNDLE_SCHEMA {
            return Err(recording("unknown capture bundle schema"));
        }
        let binding = SourceBinding::new(
            manifest.identity.clone(),
            manifest_sha256.clone(),
            manifest.capture_kind,
        )?;
        if manifest.source_id != binding.source_id() {
            return Err(recording(
                "manifest source ID does not match derived source ID",
            ));
        }
        if manifest.capture_kind == CaptureKind::Official {
            check_official_configuration(&manifest.configuration)?;
        }

        let mut files = BTreeMap::new();
        let mut paths = BTreeMap::new();
        let mut total = raw.len();
        let mut last_id = None;
        for record in &manifest.records {
            if last_id.is_some_and(|last| record.id <= last) {
                return Err(recording("manifest record IDs must strictly increase"));
            }
            last_id = Some(record.id);
            if !valid_id(&record.reader_id) || record.source_id != manifest.source_id {
                return Err(recording("manifest record reader ID or source ID invalid"));
            }
            let record_endpoint = endpoint(&record.method, &record.path)
                .map_err(|err| recording(&err.to_string()))?;
            if matches!(record_endpoint, Endpoint::Submit | Endpoint::L1)
                && record.request.is_none()
            {
                return Err(recording("POST request has no recorded body"));
            }
            if record.status.is_some() != record.response.is_some() {
                return Err(recording(
                    "HTTP status and response file must occur together",
                ));
            }
            if manifest.capture_kind == CaptureKind::Official {
                check_official_observation(&manifest.configuration, record, record_endpoint)?;
            }
        }

        let all_refs = manifest.artifacts.iter().chain(
            manifest
                .records
                .iter()
                .flat_map(|record| record.request.iter().chain(record.response.iter())),
        );
        for reference in all_refs {
            if !valid_digest(&reference.sha256) {
                return Err(recording("invalid file SHA-256"));
            }
            let path = file_in_directory(&directory, &reference.path)?;
            if path == manifest_path {
                return Err(recording("manifest cannot reference its own file"));
            }
            paths.insert(reference.path.clone(), path.clone());
            if let Some(file) = files.get(&path) {
                let VerifiedFile { sha256, .. } = file;
                if sha256 != &reference.sha256 {
                    return Err(recording("inconsistent hashes for the same file"));
                }
                continue;
            }

            let remaining = MAX_CAPTURE_BUNDLE_BYTES - total;
            if file_len(&path)? > MAX_CAPTURE_FILE_BYTES {
                return Err(Error::Capacity("raw file"));
            }
            let bytes = read_bounded(
                &path,
                remaining.min(MAX_CAPTURE_FILE_BYTES),
                "recording bundle",
            )?;
            total += bytes.len();
            let sha256 = sha256_hex(&bytes);
            if sha256 != reference.sha256 {
                return Err(recording(&format!(
                    "SHA-256 mismatch for {}",
                    reference.path
                )));
            }
            files.insert(path, VerifiedFile { bytes, sha256 });
        }

        if manifest.capture_kind == CaptureKind::Official {
            let configuration = &manifest.configuration;
            let artifacts = &configuration["artifacts"];
            if artifacts["official_commit"].as_str() != Some(OFFICIAL_COMMIT) {
                return Err(recording(
                    "launch metadata has wrong official source revision",
                ));
            }
            let pins = [
                ("official-index.json", "index", IMAGE_INDEX_DIGEST),
                (
                    "official-manifest-amd64.json",
                    "manifest_amd64",
                    IMAGE_AMD64_DIGEST,
                ),
                ("official-config.json", "image_config", IMAGE_CONFIG_DIGEST),
            ];
            for (path, key, expected) in pins {
                if artifacts[key].as_str() != Some(expected)
                    || !manifest.artifacts.iter().any(|file| {
                        file.path == path
                            && Some(file.sha256.as_str()) == expected.strip_prefix("sha256:")
                    })
                {
                    return Err(recording(
                        "official image metadata differs from the pinned image",
                    ));
                }
            }
        }

        Ok(Self {
            binding,
            manifest,
            files,
            paths,
        })
    }

    pub fn binding(&self) -> &SourceBinding {
        &self.binding
    }

    pub fn reader_ids(&self) -> BTreeSet<&str> {
        self.manifest
            .records
            .iter()
            .map(|record| record.reader_id.as_str())
            .collect()
    }

    pub fn configuration(&self) -> &Map<String, Value> {
        &self.manifest.configuration
    }

    pub fn replay(&self, reader_id: &str) -> Result<ReplaySession, Error> {
        if !self.reader_ids().contains(reader_id) {
            return Err(recording("requested reader is absent from recording"));
        }
        let mut reader = OfficialCaptureReader::new(self.binding.clone(), reader_id)?;
        let mut publications = Vec::new();
        let mut unavailable_records = Vec::new();
        let mut rejected_submissions = Vec::new();
        for record in self
            .manifest
            .records
            .iter()
            .filter(|record| record.reader_id == reader_id)
        {
            let response = record
                .response
                .as_ref()
                .map_or(&[][..], |reference| self.file(reference));
            let submit_payload = if endpoint(&record.method, &record.path)? == Endpoint::Submit {
                let request = record.request.as_ref().expect("validated POST request");
                Some(publication_request(self.file(request))?)
            } else {
                None
            };
            let outcome = reader.receive(CaptureRecord {
                source_id: &record.source_id,
                reader_id: &record.reader_id,
                id: record.id,
                method: &record.method,
                path: &record.path,
                status: record.status,
                response,
            })?;
            match outcome {
                ReceiveOutcome::Submitted { transaction_hash } => {
                    publications.push(Publication {
                        submit_record_id: record.id,
                        transaction_hash,
                        payload: submit_payload.expect("submit payload validated"),
                        positions: Vec::new(),
                    });
                }
                ReceiveOutcome::Unavailable => unavailable_records.push(record.id),
                ReceiveOutcome::SubmitRejected { status } => {
                    rejected_submissions.push((record.id, status));
                }
                _ => {}
            }
        }
        for publication in &mut publications {
            publication.positions =
                reader.find_publication(&publication.transaction_hash, &publication.payload);
        }
        Ok(ReplaySession {
            reader,
            publications,
            unavailable_records,
            rejected_submissions,
        })
    }

    fn file(&self, reference: &RawFile) -> &[u8] {
        &self.files[&self.paths[&reference.path]].bytes
    }
}

pub fn replay_bundle(directory: impl AsRef<Path>, reader_id: &str) -> Result<ReplaySession, Error> {
    CaptureBundle::open(directory)?.replay(reader_id)
}

fn publication_request(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let body = strict_json(bytes).map_err(|err| recording(&format!("submit request: {err}")))?;
    let payload = body["payload"]
        .as_str()
        .ok_or_else(|| recording("missing submit payload"))?;
    if body["namespace"].as_u64() != Some(PUBLICATION_NAMESPACE) {
        return Err(recording("submit request has wrong namespace"));
    }
    decode_payload(payload)
}

fn check_official_configuration(config: &Map<String, Value>) -> Result<(), Error> {
    let revision = config.get("candidate_sha").and_then(Value::as_str);
    if !revision.is_some_and(|revision| {
        revision.len() == 40
            && revision
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    }) {
        return Err(recording("missing or invalid official candidate SHA"));
    }
    for key in ["execution_method", "network"] {
        if !config
            .get(key)
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty())
        {
            return Err(recording(&format!(
                "missing official configuration field {key}"
            )));
        }
    }
    for key in ["commands", "ports", "l1", "artifacts"] {
        if !config.get(key).is_some_and(Value::is_object) {
            return Err(recording(&format!(
                "missing official configuration field {key}"
            )));
        }
    }
    let ports = &config["ports"];
    let mut unique_ports = BTreeSet::new();
    for key in ["api", "dev", "builder", "prover", "tonic", "anvil"] {
        let port = ports[key]
            .as_u64()
            .and_then(|port| u16::try_from(port).ok());
        if !port.is_some_and(|port| port != 0 && unique_ports.insert(port)) {
            return Err(recording("invalid or repeated official local port"));
        }
    }
    Ok(())
}

fn check_official_observation(
    config: &Map<String, Value>,
    record: &RecordedRequest,
    endpoint: Endpoint,
) -> Result<(), Error> {
    let port = match endpoint {
        Endpoint::DevInfo => "dev",
        Endpoint::L1 => "anvil",
        _ => "api",
    };
    let origin = format!("http://127.0.0.1:{}", config["ports"][port]);
    if record.observation.get("origin").and_then(Value::as_str) != Some(&origin) {
        return Err(recording(
            "official observation is not bound to the recorded local endpoint",
        ));
    }
    Ok(())
}

fn file_in_directory(directory: &Path, recorded_path: &str) -> Result<PathBuf, Error> {
    if recorded_path.contains('\\')
        || recorded_path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(recording("invalid relative file path"));
    }
    let resolved = canonicalize(&directory.join(recorded_path))?;
    let info = fs::metadata(&resolved).map_err(|err| io_error(&resolved, err))?;
    if !resolved.starts_with(directory) || !info.is_file() {
        return Err(recording(
            "recorded file escapes bundle or is not a regular file",
        ));
    }
    Ok(resolved)
}

fn canonicalize(path: &Path) -> Result<PathBuf, Error> {
    fs::canonicalize(path).map_err(|err| io_error(path, err))
}

fn file_len(path: &Path) -> Result<usize, Error> {
    let len = fs::metadata(path).map_err(|err| io_error(path, err))?.len();
    usize::try_from(len).map_err(|_| Error::Capacity("raw file"))
}

fn read_bounded(path: &Path, limit: usize, label: &'static str) -> Result<Vec<u8>, Error> {
    let file = File::open(path).map_err(|err| io_error(path, err))?;
    let metadata = file.metadata().map_err(|err| io_error(path, err))?;
    let n = usize::try_from(metadata.len()).map_err(|_| Error::Capacity(label))?;
    if !metadata.is_file() {
        return Err(recording("recording file is not regular"));
    }
    if n > limit {
        return Err(Error::Capacity(label));
    }
    let mut bytes = Vec::with_capacity(n);
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|err| io_error(path, err))?;
    if bytes.len() > limit {
        return Err(Error::Capacity(label));
    }
    Ok(bytes)
}

fn recording(reason: &str) -> Error {
    Error::InvalidRecording(reason.into())
}

fn io_error(path: &Path, error: std::io::Error) -> Error {
    recording(&format!("{}: {error}", path.display()))
}

fn manifest_error(error: serde_json::Error) -> Error {
    recording(&format!("manifest JSON: {error}"))
}
