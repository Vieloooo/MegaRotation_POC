use crate::{
    Error, Id, Result, hash_parts,
    native_config::Resolved,
    native_wire::{Ack, Grant, WorkerFrame, worker_frame},
    native_wire_support as wire,
};
use prost::Message;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Serialize)]
struct Record<'a> {
    schema: u32,
    sequence: u64,
    previous_record_hash: Id,
    grant: &'a Grant,
    ack: &'a Ack,
}

pub(crate) struct StepJournal {
    file: File,
    dir: File,
    root: PathBuf,
    run: Id,
    config: Id,
    worker: u32,
    sequence: u64,
    offset: u64,
    head: Id,
    acks: BTreeMap<u64, (Grant, Ack)>,
}
fn io<T>(r: std::io::Result<T>) -> Result<T> {
    r.map_err(|_| Error("native root step filesystem error"))
}
impl StepJournal {
    pub fn create(root: &Path, config: &Resolved, worker: u32) -> Result<Self> {
        let file = io(OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(root.join("root-step")))?;
        io(file.sync_all())?;
        let dir = io(File::open(root))?;
        let journal = Self {
            file,
            dir,
            root: root.to_owned(),
            run: config.run_id,
            config: config.config_hash,
            worker,
            sequence: 0,
            offset: 0,
            head: [0; 32],
            acks: BTreeMap::new(),
        };
        journal.sync_head(false)?;
        Ok(journal)
    }
    fn sync_head(&self, rename: bool) -> Result<()> {
        let mut bytes = Vec::with_capacity(88);
        bytes.extend_from_slice(b"I1SHEAD1");
        bytes.extend_from_slice(&self.offset.to_be_bytes());
        bytes.extend_from_slice(&self.sequence.to_be_bytes());
        bytes.extend_from_slice(&self.head);
        let checksum = hash_parts(
            b"I1b/root-step-head/v1",
            &[&self.run, &self.config, &self.worker.to_be_bytes(), &bytes],
        );
        bytes.extend_from_slice(&checksum);
        let head = self.root.join("root-step.head");
        let file = if rename {
            self.root.join("root-step.head.pending")
        } else {
            head.clone()
        };
        let mut f = io(OpenOptions::new().create_new(true).write(true).open(&file))?;
        io(f.write_all(&bytes))?;
        io(f.sync_all())?;
        if rename {
            io(fs::rename(file, head))?;
        }
        io(self.dir.sync_all())
    }
    pub fn cached(&self, grant: &Grant) -> Result<Option<Ack>> {
        if let Some((known, ack)) = self.acks.get(&grant.logical_step_id) {
            if known != grant {
                return Err(Error("altered duplicate native grant"));
            }
            return Ok(Some(ack.clone()));
        }
        if self
            .acks
            .last_key_value()
            .is_some_and(|(id, _)| grant.logical_step_id <= *id)
        {
            return Err(Error("obsolete native grant"));
        }
        Ok(None)
    }
    pub fn commit(&mut self, grant: Grant, mut ack: Ack) -> Result<Ack> {
        let sequence = self
            .sequence
            .checked_add(1)
            .ok_or(Error("root step sequence exhausted"))?;
        ack.durable_step_sequence = sequence;
        ack.logical_ack_hash = wire::ack_hash(&ack).to_vec();
        let mut bounded = ack.clone();
        bounded.durable_step_hash = vec![0; 32];
        if (WorkerFrame {
            frame: Some(worker_frame::Frame::Ack(bounded)),
        })
        .encoded_len()
            > wire::MAX_FRAME
        {
            return Err(Error("complete native Ack exceeds protobuf bound"));
        }
        let record = Record {
            schema: 1,
            sequence,
            previous_record_hash: self.head,
            grant: &grant,
            ack: &ack,
        };
        let bytes =
            serde_json::to_vec(&record).map_err(|_| Error("root step record encode failure"))?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(Error("root step record exceeds bound"));
        }
        let len = bytes.len() as u64;
        let hash = hash_parts(b"I1b/root-step-record/v1", &[&bytes]);
        io(self.file.write_all(&len.to_be_bytes()))?;
        io(self.file.write_all(&(!len).to_be_bytes()))?;
        io(self.file.write_all(&bytes))?;
        io(self.file.write_all(&hash))?;
        io(self.file.sync_all())?;
        self.sequence = sequence;
        self.offset = self
            .offset
            .checked_add(len + 48)
            .ok_or(Error("root step offset exhausted"))?;
        self.head = hash;
        self.sync_head(true)?;
        ack.durable_step_hash = hash.to_vec();
        self.acks
            .insert(grant.logical_step_id, (grant, ack.clone()));
        Ok(ack)
    }
}
