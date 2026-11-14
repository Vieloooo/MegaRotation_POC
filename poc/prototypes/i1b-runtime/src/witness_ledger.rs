use crate::{
    Error, Id, Result, hash_parts,
    witness::{
        ArtifactWitness, Barrier, EligiblePayload, NativeWire, PolicySeal, Submission,
        WitnessPolicy, require,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

const LIMIT: usize = 16 * 1024 * 1024;
const MAGIC: &[u8; 8] = b"I1WHEAD1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerIdentity {
    pub run_id: Id,
    pub config_hash: Id,
    pub worker: u32,
}

impl From<&PolicySeal> for LedgerIdentity {
    fn from(policy: &PolicySeal) -> Self {
        Self {
            run_id: policy.config.run_id,
            config_hash: policy.config.config_hash,
            worker: policy.config.worker,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobSeal {
    pub id: Id,
    pub open_id: u64,
    pub section: u64,
}
impl BlobSeal {
    pub(crate) fn new(policy: &PolicySeal, open_id: u64, section: u64) -> Self {
        let cfg = &policy.config;
        let id = hash_parts(
            b"I1b/blob/v1",
            &[
                &cfg.run_id,
                &cfg.config_hash,
                &cfg.worker.to_be_bytes(),
                &cfg.generation.to_be_bytes(),
                cfg.partition.as_bytes(),
                &section.to_be_bytes(),
                &open_id.to_be_bytes(),
            ],
        );
        Self {
            id,
            open_id,
            section,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryLink {
    pub old_generation: u64,
    pub old_head_hash: Id,
    pub source_files: Vec<(u64, Id)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "entry", content = "value", rename_all = "snake_case")]
pub enum LedgerEntry {
    Policy(PolicySeal),
    Blob(BlobSeal),
    Barrier(Barrier),
    Witness(ArtifactWitness),
    Eligible(EligiblePayload),
    Submission(Submission),
    Recovery(RecoveryLink),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerRecord {
    pub schema: u32,
    pub sequence: u64,
    pub previous_record_hash: Id,
    pub identity: LedgerIdentity,
    pub entries: Vec<LedgerEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WitnessFault {
    BeforeLedger,
    AfterRecordSync,
    AfterHeadSync,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedgerTail {
    RejectComplete,
    PromoteVerified,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessHead {
    pub sequence: u64,
    pub hash: Id,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Head {
    offset: u64,
    sequence: u64,
    hash: Id,
}

impl Head {
    const EMPTY: Self = Self {
        offset: 0,
        sequence: 0,
        hash: [0; 32],
    };
    fn anchor(self) -> WitnessHead {
        WitnessHead {
            sequence: self.sequence,
            hash: self.hash,
        }
    }
    fn bytes(self, identity: &LedgerIdentity) -> [u8; 88] {
        let mut bytes = [0; 88];
        bytes[..8].copy_from_slice(MAGIC);
        bytes[8..16].copy_from_slice(&self.offset.to_be_bytes());
        bytes[16..24].copy_from_slice(&self.sequence.to_be_bytes());
        bytes[24..56].copy_from_slice(&self.hash);
        let digest = hash_parts(
            b"I1b/witness-head/v1",
            &[
                &identity.run_id,
                &identity.config_hash,
                &identity.worker.to_be_bytes(),
                &bytes[..56],
            ],
        );
        bytes[56..].copy_from_slice(&digest);
        bytes
    }
    fn decode(bytes: &[u8], identity: &LedgerIdentity) -> Result<Self> {
        require(bytes.len() == 88, "invalid witness head length")?;
        let head = Self {
            offset: u64::from_be_bytes(bytes[8..16].try_into().unwrap()),
            sequence: u64::from_be_bytes(bytes[16..24].try_into().unwrap()),
            hash: bytes[24..56].try_into().unwrap(),
        };
        require(
            head.bytes(identity) == bytes,
            "invalid witness head identity or checksum",
        )?;
        Ok(head)
    }
}

#[derive(Clone, Default)]
pub(crate) struct Records {
    pub policy: Option<PolicySeal>,
    pub blobs: BTreeMap<Id, BlobSeal>,
    pub barriers: BTreeMap<Id, Barrier>,
    pub witnesses: BTreeMap<Id, ArtifactWitness>,
    pub eligible: BTreeMap<Id, EligiblePayload>,
    pub submissions: BTreeMap<Id, Submission>,
    recovery: Option<RecoveryLink>,
}

impl Records {
    fn apply(&mut self, policy: &WitnessPolicy, entries: &[LedgerEntry]) -> Result<()> {
        require(!entries.is_empty(), "empty witness batch")?;
        for entry in entries {
            if self.policy.is_none() {
                let LedgerEntry::Policy(seal) = entry else {
                    return Err(Error("missing initial witness policy"));
                };
                require(seal == policy.seal(), "witness policy mismatch")?;
                self.policy = Some(seal.clone());
                continue;
            }
            match entry {
                LedgerEntry::Policy(_) => return Err(Error("repeated witness policy")),
                LedgerEntry::Blob(blob) => {
                    require(
                        blob.open_id > 0
                            && policy.seal.config.sections.contains(&blob.section)
                            && *blob == BlobSeal::new(policy.seal(), blob.open_id, blob.section),
                        "invalid witness blob identity",
                    )?;
                    require(
                        self.blobs.insert(blob.id, blob.clone()).is_none(),
                        "repeated witness blob",
                    )?;
                }
                LedgerEntry::Barrier(barrier) => {
                    require(
                        self.blobs.contains_key(&barrier.blob_id)
                            && matches!(barrier.kind, 1 | 2)
                            && !barrier.bytes.is_empty()
                            && barrier.io_id > 0
                            && barrier
                                .offset
                                .checked_add(barrier.bytes.len() as u64)
                                .is_some_and(|end| end <= crate::witness::MAX_BLOB_BYTES as u64)
                            && (barrier.kind == 1 || barrier.offset == 0)
                            && *barrier
                                == Barrier::new(
                                    barrier.blob_id,
                                    barrier.io_id,
                                    barrier.kind,
                                    barrier.offset,
                                    barrier.bytes.clone(),
                                ),
                        "invalid witness barrier",
                    )?;
                    require(
                        self.barriers.insert(barrier.id, barrier.clone()).is_none(),
                        "repeated witness barrier",
                    )?;
                }
                LedgerEntry::Witness(witness) => {
                    require(
                        self.blobs
                            .get(&witness.blob_id)
                            .is_some_and(|blob| blob.section == witness.frame.section),
                        "witness blob/view mismatch",
                    )?;
                    require(
                        witness.barrier_ids.windows(2).all(|ids| ids[0] < ids[1])
                            && *witness
                                == ArtifactWitness::new(
                                    policy.seal(),
                                    witness.blob_id,
                                    witness.frame.clone(),
                                    witness.barrier_ids.clone(),
                                ),
                        "invalid witness identity",
                    )?;
                    policy.verify_evidence(&witness.frame)?;
                    verify_coverage(witness, &self.barriers)?;
                    require(
                        self.witnesses.insert(witness.id, witness.clone()).is_none(),
                        "repeated artifact witness",
                    )?;
                }
                LedgerEntry::Eligible(payload) => {
                    require(
                        self.witnesses
                            .get(&payload.witness_id)
                            .is_some_and(|witness| {
                                witness.frame.wire.as_ref() == Some(&payload.wire)
                            })
                            && *payload
                                == EligiblePayload::new(
                                    policy.seal(),
                                    payload.witness_id,
                                    payload.wire.clone(),
                                ),
                        "invalid eligible payload",
                    )?;
                    require(
                        self.eligible.insert(payload.id, payload.clone()).is_none(),
                        "repeated eligible payload",
                    )?;
                }
                LedgerEntry::Submission(submission) => {
                    require(
                        self.eligible.contains_key(&submission.payload_id)
                            && submission.destination < 5
                            && submission.destination != policy.seal.config.member
                            && *submission
                                == Submission::new(submission.payload_id, submission.destination),
                        "invalid witness submission",
                    )?;
                    require(
                        self.submissions
                            .insert(submission.id, submission.clone())
                            .is_none(),
                        "repeated witness submission",
                    )?;
                }
                LedgerEntry::Recovery(link) => {
                    require(
                        link.old_generation.checked_add(1) == Some(policy.seal.config.generation)
                            && link
                                .source_files
                                .windows(2)
                                .all(|pair| pair[0].0 < pair[1].0)
                            && link
                                .source_files
                                .iter()
                                .all(|(section, _)| policy.seal.config.sections.contains(section)),
                        "invalid recovery generation link",
                    )?;
                    require(
                        self.recovery.replace(link.clone()).is_none(),
                        "repeated recovery generation link",
                    )?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn payload(&self, wire: &NativeWire) -> Option<&EligiblePayload> {
        self.eligible.values().find(|payload| payload.wire == *wire)
    }
}

fn verify_coverage(witness: &ArtifactWitness, known: &BTreeMap<Id, Barrier>) -> Result<()> {
    use crate::witness::{DATA_BYTES, PAGE_BYTES};
    let barriers: Vec<_> = witness
        .barrier_ids
        .iter()
        .map(|id| {
            known
                .get(id)
                .filter(|b| b.blob_id == witness.blob_id)
                .ok_or(Error("missing or foreign witness barrier"))
        })
        .collect::<Result<_>>()?;
    for page in &witness.frame.pages {
        let physical = page.physical_offset as usize;
        require(
            barriers.iter().any(|barrier| {
                barrier
                    .offset
                    .checked_add(barrier.bytes.len() as u64)
                    .is_some_and(|end| end >= (physical + PAGE_BYTES) as u64)
            }),
            "witness lacks durable physical page extent",
        )?;
        let footer = DATA_BYTES + usize::from(page.selected_slot) * 6;
        for offset in (0..usize::from(page.selected_length)).chain(footer..footer + 6) {
            let absolute = (physical + offset) as u64;
            require(
                barriers.iter().any(|b| {
                    absolute
                        .checked_sub(b.offset)
                        .and_then(|local| b.bytes.get(local as usize))
                        .is_some_and(|byte| *byte == page.bytes[offset])
                }),
                "selected CRC byte lacks matching durable coverage",
            )?;
        }
    }
    Ok(())
}

pub struct WitnessLedger {
    path: PathBuf,
    file: File,
    directory: File,
    identity: LedgerIdentity,
    head: Head,
    policy: WitnessPolicy,
    pub(crate) index: Records,
    records: Vec<LedgerRecord>,
    anchored: bool,
}

impl WitnessLedger {
    pub fn create(path: &Path, policy: WitnessPolicy) -> Result<Self> {
        let identity = LedgerIdentity::from(policy.seal());
        let file = io(OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path))?;
        let directory = directory(path)?;
        io(file.sync_all())?;
        let head = Head::EMPTY;
        let mut head_file = io(OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(side(path, ".head")))?;
        io(head_file.write_all(&head.bytes(&identity)))?;
        io(head_file.sync_all())?;
        io(directory.sync_all())?;
        let initial = LedgerEntry::Policy(policy.seal().clone());
        let mut ledger = Self {
            path: path.to_owned(),
            file,
            directory,
            identity,
            head,
            policy,
            index: Records::default(),
            records: vec![],
            anchored: false,
        };
        ledger.append(vec![initial], None)?;
        Ok(ledger)
    }

    pub fn open(path: &Path, policy: WitnessPolicy, tail: LedgerTail) -> Result<Self> {
        Self::open_inner(path, policy, tail, None)
    }

    pub fn open_anchored(
        path: &Path,
        policy: WitnessPolicy,
        tail: LedgerTail,
        expected: WitnessHead,
    ) -> Result<Self> {
        Self::open_inner(path, policy, tail, Some(expected))
    }

    fn open_inner(
        path: &Path,
        policy: WitnessPolicy,
        tail: LedgerTail,
        expected: Option<WitnessHead>,
    ) -> Result<Self> {
        let identity = LedgerIdentity::from(policy.seal());
        require(
            io(fs::symlink_metadata(path))?.file_type().is_file(),
            "expected regular witness ledger",
        )?;
        let mut file = io(OpenOptions::new().read(true).write(true).open(path))?;
        let length = io(file.metadata())?.len();
        let head = read_head(path, &identity)?;
        let scan = read_records(&mut file, length, head, &policy, tail, expected)?;
        let mut ledger = Self {
            path: path.to_owned(),
            file,
            directory: directory(path)?,
            identity,
            head,
            policy,
            index: scan.index,
            records: scan.records,
            anchored: expected.is_some(),
        };
        if scan.end != head {
            io(ledger.file.sync_all())?;
            ledger.install_head(scan.end)?;
        }
        if length > scan.end.offset {
            io(ledger.file.set_len(scan.end.offset))?;
            io(ledger.file.sync_all())?;
        }
        io(ledger.file.seek(SeekFrom::Start(scan.end.offset)))?;
        Ok(ledger)
    }

    /// Authorize an explicit recovery submission on a completed, anchored
    /// target generation. The supplied exact wire bytes are bound to the
    /// committed destination authorizations returned here.
    pub fn submit_recovered(
        &mut self,
        wire: &NativeWire,
        destinations: &[u32],
    ) -> Result<Vec<Submission>> {
        require(
            self.index.recovery.is_some() && self.anchored,
            "recovery submission requires anchored completed recovery",
        )?;
        let payload = self
            .index
            .payload(wire)
            .ok_or(Error("recovery payload lacks a durable witness"))?;
        require(
            !destinations.is_empty()
                && destinations.iter().all(|destination| {
                    *destination < 5 && *destination != self.policy.seal.config.member
                }),
            "invalid recovery destinations",
        )?;
        let destinations: std::collections::BTreeSet<_> = destinations.iter().copied().collect();
        let submissions: Vec<_> = destinations
            .into_iter()
            .map(|destination| Submission::new(payload.id, destination))
            .collect();
        let pending = submissions
            .iter()
            .filter(|submission| !self.index.submissions.contains_key(&submission.id))
            .map(|submission| LedgerEntry::Submission(submission.clone()))
            .collect();
        self.append(pending, None)?;
        Ok(submissions)
    }

    pub fn records(&self) -> &[LedgerRecord] {
        &self.records
    }
    pub fn head_hash(&self) -> Id {
        self.head.hash
    }
    pub fn head(&self) -> WitnessHead {
        self.head.anchor()
    }

    pub(crate) fn append(
        &mut self,
        entries: Vec<LedgerEntry>,
        fault: Option<WitnessFault>,
    ) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }
        require(
            fault != Some(WitnessFault::BeforeLedger),
            "scripted witness failure before ledger",
        )?;
        require(
            io(self.file.metadata())?.len() == self.head.offset,
            "witness ledger already has uncommitted tail",
        )?;
        let mut index = self.index.clone();
        index.apply(&self.policy, &entries)?;
        let record = LedgerRecord {
            schema: 1,
            sequence: self
                .head
                .sequence
                .checked_add(1)
                .ok_or(Error("witness sequence overflow"))?,
            previous_record_hash: self.head.hash,
            identity: self.identity.clone(),
            entries,
        };
        let payload =
            serde_json::to_vec(&record).map_err(|_| Error("witness record serialization"))?;
        require(payload.len() <= LIMIT, "witness record exceeds bound")?;
        let size = payload.len() as u64;
        let hash = hash_parts(b"I1b/witness-record/v1", &[&payload]);
        io(self.file.write_all(&size.to_be_bytes()))?;
        io(self.file.write_all(&(!size).to_be_bytes()))?;
        io(self.file.write_all(&payload))?;
        io(self.file.write_all(&hash))?;
        io(self.file.sync_all())?;
        require(
            fault != Some(WitnessFault::AfterRecordSync),
            "scripted witness failure after record sync",
        )?;
        let end = Head {
            offset: io(self.file.stream_position())?,
            sequence: record.sequence,
            hash,
        };
        self.install_head(end)?;
        self.index = index;
        self.records.push(record);
        require(
            fault != Some(WitnessFault::AfterHeadSync),
            "scripted witness failure after head sync",
        )
    }

    fn install_head(&mut self, head: Head) -> Result<()> {
        let pending = side(&self.path, ".head.pending");
        let mut file = io(OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&pending))?;
        io(file.write_all(&head.bytes(&self.identity)))?;
        io(file.sync_all())?;
        io(fs::rename(pending, side(&self.path, ".head")))?;
        io(self.directory.sync_all())?;
        self.head = head;
        Ok(())
    }
}

pub(crate) struct LedgerSnapshot {
    pub head_hash: Id,
    pub index: Records,
}

pub(crate) fn snapshot(
    path: &Path,
    policy: &WitnessPolicy,
    expected: WitnessHead,
) -> Result<LedgerSnapshot> {
    let identity = LedgerIdentity::from(policy.seal());
    let head_bytes = bounded_read(&side(path, ".head"), 88)?;
    let head = Head::decode(&head_bytes, &identity)?;
    require(
        io(fs::symlink_metadata(path))?.file_type().is_file(),
        "expected regular copied witness ledger",
    )?;
    let mut file = io(File::open(path))?;
    let length = io(file.metadata())?.len();
    let scan = read_records(
        &mut file,
        length,
        head,
        policy,
        LedgerTail::RejectComplete,
        Some(expected),
    )?;
    Ok(LedgerSnapshot {
        head_hash: head.hash,
        index: scan.index,
    })
}

pub(crate) fn bounded_read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    require(
        io(fs::symlink_metadata(path))?.file_type().is_file(),
        "expected a regular evidence file",
    )?;
    let file = io(File::open(path))?;
    require(
        io(file.metadata())?.len() <= limit,
        "witness file exceeds bound",
    )?;
    let mut bytes = Vec::new();
    io(file.take(limit + 1).read_to_end(&mut bytes))?;
    require(bytes.len() as u64 <= limit, "witness file grew above bound")?;
    Ok(bytes)
}

struct Scan {
    end: Head,
    index: Records,
    records: Vec<LedgerRecord>,
}

fn read_records(
    file: &mut impl Read,
    length: u64,
    head: Head,
    policy: &WitnessPolicy,
    tail: LedgerTail,
    expected: Option<WitnessHead>,
) -> Result<Scan> {
    require(
        length >= head.offset && head.sequence > 0,
        "missing committed witness bytes or policy",
    )?;
    require(
        expected.is_none_or(|floor| floor.sequence > 0 && floor.sequence <= head.sequence),
        "trusted witness receipt is above committed head or invalid",
    )?;
    let identity = LedgerIdentity::from(policy.seal());
    let mut scan = Scan {
        end: Head::EMPTY,
        index: Records::default(),
        records: vec![],
    };
    loop {
        if scan.end.offset >= head.offset {
            require(
                scan.end == head,
                "witness chain does not match committed head",
            )?;
        }
        let available = length - scan.end.offset;
        if available < 16 {
            require(
                scan.end.offset >= head.offset,
                "incomplete header below witness head",
            )?;
            break;
        }
        let mut header = [0; 16];
        io(file.read_exact(&mut header))?;
        let size = u64::from_be_bytes(header[..8].try_into().unwrap());
        let complement = u64::from_be_bytes(header[8..].try_into().unwrap());
        require(
            size > 0 && size <= LIMIT as u64 && size == !complement,
            "invalid witness record header",
        )?;
        let frame = size + 48;
        if scan.end.offset < head.offset {
            require(
                frame <= head.offset - scan.end.offset,
                "incomplete frame below witness head",
            )?;
        } else if frame > available {
            break;
        } else {
            require(
                frame == available && tail == LedgerTail::PromoteVerified,
                "unexpected complete witness tail",
            )?;
        }
        let mut payload = vec![0; size as usize];
        let mut hash = [0; 32];
        io(file.read_exact(&mut payload))?;
        io(file.read_exact(&mut hash))?;
        require(
            hash == hash_parts(b"I1b/witness-record/v1", &[&payload]),
            "witness record checksum mismatch",
        )?;
        let record: LedgerRecord =
            serde_json::from_slice(&payload).map_err(|_| Error("witness record decoding"))?;
        require(
            serde_json::to_vec(&record).map_err(|_| Error("witness record encoding"))? == payload
                && record.schema == 1
                && record.identity == identity
                && record.previous_record_hash == scan.end.hash
                && scan.end.sequence.checked_add(1) == Some(record.sequence),
            "witness record chain/identity mismatch",
        )?;
        scan.index.apply(policy, &record.entries)?;
        let was_committed = scan.end.offset < head.offset;
        scan.end = Head {
            offset: scan.end.offset + frame,
            sequence: record.sequence,
            hash,
        };
        if let Some(floor) = expected.filter(|floor| floor.sequence == record.sequence) {
            require(
                was_committed && scan.end.anchor() == floor,
                "witness chain conflicts with trusted prefix",
            )?;
        }
        scan.records.push(record);
        if !was_committed {
            break;
        }
    }
    Ok(scan)
}

fn read_head(path: &Path, identity: &LedgerIdentity) -> Result<Head> {
    Head::decode(&bounded_read(&side(path, ".head"), 88)?, identity)
}

pub(crate) fn side(path: &Path, suffix: &str) -> PathBuf {
    let mut raw = path.as_os_str().to_os_string();
    raw.push(suffix);
    raw.into()
}

fn directory(path: &Path) -> Result<File> {
    io(File::open(
        path.parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new(".")),
    ))
}

pub(crate) fn io<T>(result: std::io::Result<T>) -> Result<T> {
    result.map_err(|_| Error("witness filesystem operation failed"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::witness::{DecodedArtifact, PageEvidence};

    #[test]
    fn reopen_evidence_rejects_missing_crc_prefix_bytes_or_extent() {
        let blob = [1; 32];
        let page: Vec<_> = (0..=255).collect();
        let whole_crc_not_extent = Barrier::new(blob, 1, 1, 0, page[..250].to_vec());
        let second_frame_and_extent = Barrier::new(blob, 2, 1, 86, page[86..].to_vec());
        let extent = Barrier::new(blob, 3, 1, 250, page[250..].to_vec());
        let known: BTreeMap<_, _> = [&whole_crc_not_extent, &second_frame_and_extent, &extent]
            .into_iter()
            .map(|barrier| (barrier.id, barrier.clone()))
            .collect();
        let mut witness = ArtifactWitness {
            id: [0; 32],
            blob_id: blob,
            artifact_hash: [0; 32],
            barrier_ids: vec![second_frame_and_extent.id],
            frame: DecodedArtifact {
                section: 1,
                logical_offset: 86,
                frame_bytes: page[86..172].to_vec(),
                artifact_bytes: page[87..172].to_vec(),
                pages: vec![PageEvidence {
                    physical_offset: 0,
                    selected_slot: 0,
                    selected_length: 172,
                    bytes: page,
                }],
                wire: None,
            },
        };
        assert_eq!(
            verify_coverage(&witness, &known),
            Err(Error("selected CRC byte lacks matching durable coverage"))
        );
        witness.barrier_ids = vec![whole_crc_not_extent.id];
        assert_eq!(
            verify_coverage(&witness, &known),
            Err(Error("witness lacks durable physical page extent"))
        );
        witness.barrier_ids.push(extent.id);
        assert_eq!(verify_coverage(&witness, &known), Ok(()));
    }
}
