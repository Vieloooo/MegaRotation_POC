use super::*;
use poc_types::{input::Kind as I, worker_frame::Frame};
use prost::Message;
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

const MAX_RECORD_BYTES: usize = 16 * 1024 * 1024;
const HEADER_BYTES: u64 = 16;
const FRAME_OVERHEAD: u64 = HEADER_BYTES + 32;
const HEAD_MAGIC: &[u8; 8] = b"I1HEAD01";
const HEAD_BYTES: usize = 88;

struct CommittedHead {
    offset: u64,
    seq: u64,
    record_hash: [u8; 32],
}

impl CommittedHead {
    const EMPTY: Self = Self {
        offset: 0,
        seq: 0,
        record_hash: [0; 32],
    };

    fn encode(&self, identity: &Identity) -> [u8; HEAD_BYTES] {
        let mut bytes = [0; HEAD_BYTES];
        bytes[..8].copy_from_slice(HEAD_MAGIC);
        bytes[8..16].copy_from_slice(&self.offset.to_be_bytes());
        bytes[16..24].copy_from_slice(&self.seq.to_be_bytes());
        bytes[24..56].copy_from_slice(&self.record_hash);
        let checksum = hash_parts(
            "I1/journal-head/v1",
            &[
                &identity.run_id,
                &identity.config_hash,
                &identity.node_id.to_be_bytes(),
                &bytes[..56],
            ],
        );
        bytes[56..].copy_from_slice(&checksum);
        bytes
    }

    fn read(path: &Path, identity: &Identity) -> Result<Self> {
        let mut file = File::open(path)?;
        require(
            file.metadata()?.len() == HEAD_BYTES as u64,
            "invalid journal head size",
        )?;
        let mut bytes = [0; HEAD_BYTES];
        file.read_exact(&mut bytes)?;
        let head = Self {
            offset: u64::from_be_bytes(bytes[8..16].try_into()?),
            seq: u64::from_be_bytes(bytes[16..24].try_into()?),
            record_hash: bytes[24..56].try_into()?,
        };
        require(
            head.encode(identity) == bytes,
            "journal head checksum or identity mismatch",
        )?;
        Ok(head)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StepCommit {
    pub journal_seq: u64,
    pub previous_record_hash: Vec<u8>,
    pub resulting_state: State,
    pub logical_step_id: u64,
    pub grant_hash: Vec<u8>,
    pub ack: Ack,
}

pub struct Journal {
    file: File,
    head_path: PathBuf,
    pending_head_path: PathBuf,
    directory: File,
    identity: Identity,
    state: State,
    acks: BTreeMap<u64, Ack>,
    previous_record_hash: Vec<u8>,
    seq: u64,
}

pub enum StepResult {
    Ack(Ack),
    Crash(Checkpoint),
}

impl Journal {
    pub fn open(path: &Path, identity: Identity, allow_create: bool) -> Result<Self> {
        identity.validate()?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let directory = File::open(parent)?;
        let head_path = adjacent_path(path, ".head");
        let pending_head_path = adjacent_path(path, ".head.pending");
        let mut options = OpenOptions::new();
        options.read(true).write(true);
        let (file, head) = if allow_create {
            let file = options.create_new(true).open(path)?;
            file.sync_all()?;
            let head = CommittedHead::EMPTY;
            let mut head_file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&head_path)?;
            head_file.write_all(&head.encode(&identity))?;
            head_file.sync_all()?;
            directory.sync_all()?;
            (file, head)
        } else {
            (
                options.open(path)?,
                CommittedHead::read(&head_path, &identity)?,
            )
        };
        let mut journal = Self {
            file,
            head_path,
            pending_head_path,
            directory,
            state: State::new(&identity),
            identity,
            acks: BTreeMap::new(),
            previous_record_hash: vec![0; 32],
            seq: 0,
        };
        journal.recover_records(&head)?;
        Ok(journal)
    }

    pub fn sequence(&self) -> u64 {
        self.seq
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    fn read_frame_size(&mut self) -> Result<usize> {
        let mut header = [0; HEADER_BYTES as usize];
        self.file.read_exact(&mut header)?;
        let size = u64::from_be_bytes(header[..8].try_into()?);
        let complement = u64::from_be_bytes(header[8..].try_into()?);
        require(size == !complement, "journal length complement mismatch")?;
        let size = usize::try_from(size)?;
        require(
            size > 0 && size <= MAX_RECORD_BYTES,
            "invalid journal frame length",
        )?;
        Ok(size)
    }

    fn recover_records(&mut self, head: &CommittedHead) -> Result<()> {
        let length = self.file.metadata()?.len();
        require(length >= head.offset, "journal shorter than committed head")?;
        let mut offset = 0;
        while offset < head.offset {
            let remaining = head.offset - offset;
            require(
                remaining >= HEADER_BYTES,
                "incomplete header below journal head",
            )?;
            let size = self.read_frame_size()?;
            let frame_size = u64::try_from(size)? + FRAME_OVERHEAD;
            require(
                remaining >= frame_size,
                "incomplete frame below journal head",
            )?;
            let (record, checksum) = self.read_record(size)?;
            self.install(record, checksum);
            offset += frame_size;
        }
        require(
            self.seq == head.seq && self.previous_record_hash == head.record_hash,
            "journal head does not match verified chain",
        )?;
        let tail = length - head.offset;
        if tail >= HEADER_BYTES {
            let size = self.read_frame_size()?;
            let frame_size = u64::try_from(size)? + FRAME_OVERHEAD;
            if tail >= frame_size {
                require(tail == frame_size, "multiple frames above committed head")?;
                let (record, checksum) = self.read_record(size)?;
                self.file.sync_all()?;
                offset += frame_size;
                self.sync_head(offset, record.journal_seq, &checksum)?;
                self.install(record, checksum);
            }
        }
        if offset < length {
            self.file.set_len(offset)?;
            self.file.sync_all()?;
        }
        self.file.seek(SeekFrom::Start(offset))?;
        Ok(())
    }

    fn read_record(&mut self, size: usize) -> Result<(StepCommit, Vec<u8>)> {
        let mut payload = vec![0; size];
        self.file.read_exact(&mut payload)?;
        let mut checksum = [0; 32];
        self.file.read_exact(&mut checksum)?;
        require(
            record_hash(&payload) == checksum,
            "journal checksum mismatch",
        )?;
        let record: StepCommit = serde_json::from_slice(&payload)?;
        require(
            serde_json::to_vec(&record)? == payload,
            "noncanonical journal record bytes",
        )?;
        self.validate_record(&record)?;
        Ok((record, checksum.to_vec()))
    }

    fn sync_head(&self, offset: u64, seq: u64, record_hash: &[u8]) -> Result<()> {
        let head = CommittedHead {
            offset,
            seq,
            record_hash: record_hash.try_into()?,
        };
        let mut temporary = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.pending_head_path)?;
        temporary.write_all(&head.encode(&self.identity))?;
        temporary.sync_all()?;
        fs::rename(&self.pending_head_path, &self.head_path)?;
        self.directory.sync_all()?;
        Ok(())
    }

    fn validate_record(&self, record: &StepCommit) -> Result<()> {
        let ack = &record.ack;
        require(ack.incarnation > 0, "invalid journal incarnation")?;
        self.identity.validate_ack(ack.incarnation, ack)?;
        record.resulting_state.validate_identity(&self.identity)?;
        require(
            self.seq.checked_add(1) == Some(record.journal_seq)
                && ack.durable_journal_seq == record.journal_seq,
            "journal sequence mismatch",
        )?;
        require(
            self.previous_record_hash == record.previous_record_hash,
            "journal chain mismatch",
        )?;
        require(
            ack.grant_hash == record.grant_hash && ack.logical_step_id == record.logical_step_id,
            "journal Ack binding mismatch",
        )?;
        require(
            self.acks
                .last_key_value()
                .is_none_or(|(step, _)| record.logical_step_id > *step),
            "journal logical step regressed",
        )?;
        require(
            ack.state_hash == record.resulting_state.hash()?,
            "journal state hash mismatch",
        )
    }

    fn install(&mut self, record: StepCommit, checksum: Vec<u8>) {
        self.seq = record.journal_seq;
        self.previous_record_hash = checksum;
        self.state = record.resulting_state;
        self.acks.insert(record.logical_step_id, record.ack);
    }

    pub fn process(
        &mut self,
        grant: &Grant,
        incarnation: u64,
        retry: Option<&Retry>,
        fault: Option<Checkpoint>,
    ) -> Result<StepResult> {
        self.identity.validate_grant(incarnation, grant)?;
        if let Some(retry) = retry {
            require(
                grant.logical_step_id == retry.logical_step_id
                    && grant.grant_hash == retry.grant_hash
                    && retry.original_incarnation.checked_add(1) == Some(incarnation),
                "unauthorized retry",
            )?;
        }
        if let Some(previous) = self.acks.get(&grant.logical_step_id) {
            require(
                previous.grant_hash == grant.grant_hash && previous.event_id == grant.event_id,
                "altered duplicate grant",
            )?;
            require(
                incarnation == previous.incarnation
                    || retry
                        .is_some_and(|retry| retry.original_incarnation == previous.incarnation),
                "cached Ack requires recovery authorization",
            )?;
            let mut ack = previous.clone();
            ack.incarnation = incarnation;
            return Ok(StepResult::Ack(ack));
        }
        require(
            self.acks
                .last_key_value()
                .is_none_or(|(step, _)| grant.logical_step_id > *step),
            "obsolete grant",
        )?;
        require(retry.is_none(), "recovery retry requires a durable Ack")?;
        let recovering = matches!(
            grant.input.as_ref().and_then(|input| input.kind.as_ref()),
            Some(I::Recover(_))
        );
        if incarnation != self.state.incarnation() {
            require(
                recovering && self.state.incarnation() < incarnation,
                "recovery grant required",
            )?;
        } else {
            require(!recovering, "unexpected recovery grant")?;
        }
        let mut state = self.state.clone();
        let seq = self
            .seq
            .checked_add(1)
            .ok_or_else(|| invalid("journal sequence overflow"))?;
        let ack = state.apply(&self.identity, grant, seq)?;
        let frame = WorkerFrame {
            frame: Some(Frame::Ack(ack.clone())),
        };
        require(
            frame.encoded_len() <= MAX_FRAME_BYTES,
            "complete Ack exceeds gRPC message bound",
        )?;
        if fault == Some(Checkpoint::BeforeAppend) {
            return Ok(StepResult::Crash(Checkpoint::BeforeAppend));
        }
        let record = StepCommit {
            journal_seq: seq,
            previous_record_hash: self.previous_record_hash.clone(),
            resulting_state: state,
            logical_step_id: grant.logical_step_id,
            grant_hash: grant.grant_hash.clone(),
            ack: ack.clone(),
        };
        let payload = serde_json::to_vec(&record)?;
        require(
            payload.len() <= MAX_RECORD_BYTES,
            "journal record exceeds bound",
        )?;
        let checksum = record_hash(&payload);
        let size = u64::try_from(payload.len())?;
        self.file.write_all(&size.to_be_bytes())?;
        self.file.write_all(&(!size).to_be_bytes())?;
        if fault == Some(Checkpoint::PartialTail) {
            self.file.write_all(&payload[..payload.len() / 2])?;
            self.file.sync_all()?;
            return Ok(StepResult::Crash(Checkpoint::PartialTail));
        }
        self.file.write_all(&payload)?;
        self.file.write_all(&checksum)?;
        self.file.sync_all()?;
        if fault == Some(Checkpoint::AfterRecordSyncBeforeHead) {
            return Ok(StepResult::Crash(Checkpoint::AfterRecordSyncBeforeHead));
        }
        let offset = self.file.stream_position()?;
        self.sync_head(offset, seq, &checksum)?;
        self.install(record, checksum);
        if fault == Some(Checkpoint::AfterSyncBeforeAck) {
            return Ok(StepResult::Crash(Checkpoint::AfterSyncBeforeAck));
        }
        Ok(StepResult::Ack(ack))
    }
}

fn record_hash(payload: &[u8]) -> Vec<u8> {
    hash_parts("I1/journal/v1", &[payload])
}

fn adjacent_path(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}
