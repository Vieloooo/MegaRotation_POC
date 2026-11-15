use crate::{
    Error, Id, Result, hash_parts,
    witness::{
        ArtifactWitness, Barrier, EligiblePayload, Image, Images, MAX_BLOB_BYTES, NativeWire,
        PolicySeal, Submission, WitnessPolicy, require, verify_protected,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
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
    pub initial_bytes: Vec<u8>,
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
            initial_bytes: Vec::new(),
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
pub struct IoObservation {
    pub io_id: u64,
    pub change: ObservedChange,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum ObservedChange {
    Write {
        blob_id: Id,
        offset: u64,
        bytes: Vec<u8>,
    },
    Resize {
        blob_id: Id,
        len: u64,
    },
    Remove {
        section: Option<u64>,
    },
    Uncertain {
        section: Option<u64>,
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "entry", content = "value", rename_all = "snake_case")]
pub enum LedgerEntry {
    Policy(PolicySeal),
    Blob(BlobSeal),
    Observation(IoObservation),
    Barrier(Barrier),
    Witness(ArtifactWitness),
    Eligible(EligiblePayload),
    Submission(Submission),
    GenerationStarted(RecoveryLink),
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
    pub images: Images,
    pub uncertain_sections: BTreeSet<u64>,
    last_io: u64,
    recovering: Option<RecoveryLink>,
    recovery: Option<RecoveryLink>,
}

impl Records {
    fn apply(
        &mut self,
        supplied: &WitnessPolicy,
        previous_hash: Id,
        entries: &[LedgerEntry],
    ) -> Result<()> {
        require(!entries.is_empty(), "empty witness batch")?;
        let mut decoded = BTreeMap::new();
        for entry in entries {
            if self.policy.is_none() {
                let LedgerEntry::Policy(seal) = entry else {
                    return Err(Error("missing initial witness policy"));
                };
                let mut expected = supplied.seal().clone();
                expected.config.generation = seal.config.generation;
                require(
                    *seal == expected
                        && seal.config.generation <= supplied.seal().config.generation,
                    "witness policy mismatch",
                )?;
                self.policy = Some(seal.clone());
                continue;
            }
            let policy = self.policy.clone().expect("sealed generation");
            match entry {
                LedgerEntry::Policy(_) => return Err(Error("repeated witness policy")),
                LedgerEntry::Blob(blob) => {
                    self.advance_io(blob.open_id)?;
                    require(
                        policy.config.sections.contains(&blob.section)
                            && blob.id == BlobSeal::new(&policy, blob.open_id, blob.section).id
                            && blob.initial_bytes.len() <= MAX_BLOB_BYTES
                            && (self.recovering.is_none() || blob.initial_bytes.is_empty()),
                        "invalid witness blob identity or initial image",
                    )?;
                    verify_protected(self.witnesses.values(), blob.section, &blob.initial_bytes)
                        .or_else(|error| {
                            if self.recovering.is_some() {
                                Ok(())
                            } else {
                                Err(error)
                            }
                        })?;
                    self.drop_images(Some(blob.section));
                    let mut image = Image::new(blob.initial_bytes.clone());
                    image.uncertain = self.uncertain_sections.contains(&blob.section);
                    self.images.insert(blob.id, image);
                    require(
                        self.blobs.insert(blob.id, blob.clone()).is_none(),
                        "repeated witness blob",
                    )?;
                }
                LedgerEntry::Observation(observation) => {
                    self.advance_io(observation.io_id)?;
                    self.observe_mutation(&policy, &observation.change)?;
                    decoded.clear();
                }
                LedgerEntry::Barrier(barrier) => {
                    self.advance_io(barrier.io_id)?;
                    self.install_barrier(barrier)?;
                    decoded.remove(&barrier.blob_id);
                }
                LedgerEntry::Witness(witness) => {
                    let section = self
                        .blobs
                        .get(&witness.blob_id)
                        .ok_or(Error("unknown witness blob"))?
                        .section;
                    require(
                        section == witness.frame.section
                            && witness.barrier_ids.windows(2).all(|ids| ids[0] < ids[1])
                            && *witness
                                == ArtifactWitness::new(
                                    &policy,
                                    witness.blob_id,
                                    witness.frame.clone(),
                                    witness.barrier_ids.clone(),
                                ),
                        "invalid witness identity or section",
                    )?;
                    let image = self
                        .images
                        .get(&witness.blob_id)
                        .ok_or(Error("witness is not on a live blob"))?;
                    require(
                        !self.uncertain_sections.contains(&section)
                            && image.provenance(&witness.frame).as_ref()
                                == Some(&witness.barrier_ids),
                        "witness disagrees with current durable byte versions",
                    )?;
                    let frames = match decoded.entry(witness.blob_id) {
                        std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
                        std::collections::btree_map::Entry::Vacant(entry) => {
                            entry.insert(supplied.decode_frames(section, &image.bytes, true)?)
                        }
                    };
                    require(
                        frames.iter().any(|frame| *frame == witness.frame),
                        "witness does not identify a current canonical journal frame",
                    )?;
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
                                self.images.contains_key(&witness.blob_id)
                                    && witness.frame.wire.as_ref() == Some(&payload.wire)
                            })
                            && *payload
                                == EligiblePayload::new(
                                    &policy,
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
                        self.recovering.is_none()
                            && self.eligible.contains_key(&submission.payload_id)
                            && submission.destination < 5
                            && submission.destination != policy.config.member
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
                LedgerEntry::GenerationStarted(link) => {
                    require(
                        entries.len() == 1
                            && self.recovering.is_none()
                            && link.old_head_hash == previous_hash,
                        "recovery transition lacks exact predecessor head",
                    )?;
                    self.validate_link(&policy, link)?;
                    let generation = link
                        .old_generation
                        .checked_add(1)
                        .ok_or(Error("witness generation overflow"))?;
                    require(
                        generation <= supplied.seal().config.generation,
                        "recovery exceeds configured generation",
                    )?;
                    self.policy
                        .as_mut()
                        .expect("sealed generation")
                        .config
                        .generation = generation;
                    self.recovering = Some(link.clone());
                    self.recovery = None;
                    self.images.clear();
                    self.uncertain_sections.clear();
                    self.last_io = 0;
                }
                LedgerEntry::Recovery(link) => {
                    require(
                        entries.len() == 1 && self.recovering.as_ref() == Some(link),
                        "recovery completion lacks pending transition",
                    )?;
                    self.complete_recovery(link, supplied)?;
                    self.recovering = None;
                    self.recovery = Some(link.clone());
                }
            }
        }
        Ok(())
    }

    pub(crate) fn in_recovery(&self) -> bool {
        self.recovering.is_some()
    }

    fn advance_io(&mut self, id: u64) -> Result<()> {
        require(
            id > self.last_io,
            "selected witness I/O ID is not increasing",
        )?;
        self.last_io = id;
        Ok(())
    }

    fn drop_images(&mut self, section: Option<u64>) {
        self.images
            .retain(|id, _| section.is_some_and(|section| self.blobs[id].section != section));
    }

    fn observe_mutation(&mut self, policy: &PolicySeal, change: &ObservedChange) -> Result<()> {
        match change {
            ObservedChange::Write {
                blob_id,
                offset,
                bytes,
            } => {
                let image = self
                    .images
                    .get_mut(blob_id)
                    .ok_or(Error("write is not on a live blob"))?;
                require(
                    !image.uncertain && !bytes.is_empty(),
                    "invalid observed write",
                )?;
                image.write(
                    usize::try_from(*offset).map_err(|_| Error("observed offset overflow"))?,
                    bytes,
                )?;
                if self.recovering.is_none() {
                    verify_protected(
                        self.witnesses.values(),
                        self.blobs[blob_id].section,
                        &image.bytes,
                    )?;
                }
            }
            ObservedChange::Resize { blob_id, len } => {
                let image = self
                    .images
                    .get_mut(blob_id)
                    .ok_or(Error("resize is not on a live blob"))?;
                require(
                    !image.uncertain && self.recovering.is_none(),
                    "invalid observed resize",
                )?;
                image.resize(
                    usize::try_from(*len).map_err(|_| Error("observed extent overflow"))?,
                )?;
                verify_protected(
                    self.witnesses.values(),
                    self.blobs[blob_id].section,
                    &image.bytes,
                )?;
            }
            ObservedChange::Remove { section } => {
                require(
                    self.recovering.is_none()
                        && section.is_none_or(|value| policy.config.sections.contains(&value))
                        && !self.witnesses.values().any(|witness| {
                            section.is_none_or(|section| witness.frame.section == section)
                        }),
                    "observed removal erases protected journal history",
                )?;
                self.drop_images(*section);
            }
            ObservedChange::Uncertain { section, reason } => {
                require(
                    self.recovering.is_none()
                        && matches!(reason.as_str(), "write" | "resize" | "sync" | "remove")
                        && section.map_or(reason == "remove", |value| {
                            policy.config.sections.contains(&value)
                        }),
                    "invalid uncertainty observation",
                )?;
                for value in policy
                    .config
                    .sections
                    .iter()
                    .filter(|value| section.is_none_or(|section| section == **value))
                {
                    self.uncertain_sections.insert(*value);
                }
                for (id, image) in &mut self.images {
                    if section.is_none_or(|value| self.blobs[id].section == value) {
                        image.uncertain = true;
                    }
                }
            }
        }
        Ok(())
    }

    fn install_barrier(&mut self, barrier: &Barrier) -> Result<()> {
        let image = self
            .images
            .get_mut(&barrier.blob_id)
            .ok_or(Error("barrier is not on a live blob"))?;
        let section = self.blobs[&barrier.blob_id].section;
        let start =
            usize::try_from(barrier.offset).map_err(|_| Error("barrier offset overflow"))?;
        let end = start
            .checked_add(barrier.bytes.len())
            .filter(|end| *end <= MAX_BLOB_BYTES)
            .ok_or(Error("barrier exceeds witness bounds"))?;
        require(
            !image.uncertain
                && !self.uncertain_sections.contains(&section)
                && !barrier.bytes.is_empty()
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
        match barrier.kind {
            // A range barrier is a single combined successful SYNC write.
            1 if self.recovering.is_none() => image.write(start, &barrier.bytes)?,
            2 if start == 0 && image.bytes == barrier.bytes => {}
            _ => return Err(Error("barrier disagrees with current extent or bytes")),
        }
        verify_protected(self.witnesses.values(), section, &image.bytes)?;
        image.cover(start..end, barrier.id);
        require(
            self.barriers.insert(barrier.id, barrier.clone()).is_none(),
            "repeated witness barrier",
        )
    }

    fn validate_link(&self, policy: &PolicySeal, link: &RecoveryLink) -> Result<()> {
        require(
            link.old_generation == policy.config.generation
                && link
                    .source_files
                    .windows(2)
                    .all(|pair| pair[0].0 < pair[1].0)
                && link
                    .source_files
                    .iter()
                    .all(|(section, _)| policy.config.sections.contains(section)),
            "invalid recovery generation link",
        )
    }

    fn complete_recovery(&self, link: &RecoveryLink, policy: &WitnessPolicy) -> Result<()> {
        require(
            self.images.len() == link.source_files.len(),
            "recovery image set mismatch",
        )?;
        for (section, hash) in &link.source_files {
            let (blob_id, image) = self
                .images
                .iter()
                .find(|(id, _)| self.blobs[*id].section == *section)
                .ok_or(Error("copied section missing after recovery"))?;
            require(
                crate::witness_recovery::v1_raw_hash(&image.bytes) == *hash
                    && (image.bytes.is_empty()
                        || self.barriers.values().any(|barrier| {
                            barrier.blob_id == *blob_id
                                && barrier.kind == 2
                                && barrier.bytes == image.bytes
                        })),
                "recovery lacks matching new full barrier",
            )?;
            for frame in policy.decode_section(*section, &image.bytes)? {
                require(
                    self.witnesses
                        .values()
                        .any(|witness| witness.blob_id == *blob_id && witness.frame == frame),
                    "recovery lacks new artifact witness",
                )?;
            }
        }
        for witness in self
            .witnesses
            .values()
            .filter(|witness| !self.images.contains_key(&witness.blob_id))
        {
            require(
                self.witnesses.values().any(|new| {
                    self.images.contains_key(&new.blob_id)
                        && new.frame.section == witness.frame.section
                        && new.frame.logical_offset == witness.frame.logical_offset
                        && new.frame.frame_bytes == witness.frame.frame_bytes
                        && new.frame.wire == witness.frame.wire
                }),
                "recovery lost protected journal history",
            )?;
        }
        Ok(())
    }

    pub(crate) fn payload(&self, wire: &NativeWire) -> Option<&EligiblePayload> {
        self.eligible.values().find(|payload| payload.wire == *wire)
    }
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

    pub(crate) fn begin_recovery(
        &mut self,
        target: WitnessPolicy,
        link: RecoveryLink,
    ) -> Result<()> {
        require(
            self.anchored && self.index.recovering.is_none(),
            "recovery predecessor is not settled and anchored",
        )?;
        self.policy = target;
        self.append(vec![LedgerEntry::GenerationStarted(link)], None)
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
        index.apply(&self.policy, self.head.hash, &entries)?;
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
        scan.index.apply(policy, scan.end.hash, &record.entries)?;
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
    require(
        scan.index.policy.as_ref() == Some(policy.seal()),
        "ledger does not reach configured generation",
    )?;
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
    fn current_image_provenance_requires_the_crc_prefix_and_extent() {
        let blob = [1; 32];
        let page: Vec<_> = (0..=255).collect();
        let whole_crc_not_extent = Barrier::new(blob, 1, 1, 0, page[..250].to_vec());
        let second_frame_and_extent = Barrier::new(blob, 2, 1, 86, page[86..].to_vec());
        let extent = Barrier::new(blob, 3, 1, 250, page[250..].to_vec());
        let mut image = Image::new(page.clone());
        let frame = DecodedArtifact {
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
        };
        image.cover(86..256, second_frame_and_extent.id);
        assert!(image.provenance(&frame).is_none());
        let mut image = Image::new(image.bytes);
        image.cover(0..250, whole_crc_not_extent.id);
        assert!(image.provenance(&frame).is_none());
        image.cover(250..256, extent.id);
        let expected = BTreeSet::from([whole_crc_not_extent.id, extent.id]);
        assert_eq!(
            image
                .provenance(&frame)
                .unwrap()
                .into_iter()
                .collect::<BTreeSet<_>>(),
            expected
        );
        image.write(0, &[19]).unwrap();
        assert!(image.provenance(&frame).is_none());
        image.write(0, &[0]).unwrap();
        assert!(image.provenance(&frame).is_none());
    }
}
