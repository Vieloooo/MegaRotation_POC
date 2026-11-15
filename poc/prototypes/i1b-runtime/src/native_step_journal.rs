use crate::{
    Error, Id, Result, hash_parts,
    native_config::Resolved,
    native_recovery_ledger::{ROOT_PENDING_HEAD, preserve_pending_head},
    native_wire::{Ack, Grant, Receipt, RecoveredAck, WorkerFrame, worker_frame},
    native_wire_support as wire,
};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
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

#[derive(Deserialize)]
struct ParsedRecord {
    schema: u32,
    sequence: u64,
    previous_record_hash: Id,
    grant: Grant,
    ack: Ack,
}
const FRAME_LIMIT: usize = 16 * 1024 * 1024;

pub(crate) struct StepJournal {
    file: File,
    dir: File,
    root: PathBuf,
    run: Id,
    config: Id,
    worker: u32,
    recovery_profile: bool,
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
            recovery_profile: config
                .base
                .recovery
                .as_ref()
                .is_some_and(|scope| scope.target_worker == worker),
            sequence: 0,
            offset: 0,
            head: [0; 32],
            acks: BTreeMap::new(),
        };
        journal.sync_head(false)?;
        Ok(journal)
    }
    /// Open an exclusive mutable working copy with a separately retained
    /// original crash image. Only authorized complete tails may be promoted.
    pub fn open_recovery(
        root: &Path,
        config: &Resolved,
        worker: u32,
        trusted: commonware_consensus::simplex::recovery::RecoveryReceipt,
        authorized: Option<&Grant>,
    ) -> Result<Self> {
        let path = root.join("root-step");
        let head = root.join("root-step.head");
        if !io(fs::symlink_metadata(&path))?.file_type().is_file()
            || !io(fs::symlink_metadata(&head))?.file_type().is_file()
            || io(fs::metadata(&head))?.len() != 88
        {
            return Err(Error("invalid root step files"));
        }
        let bytes = io(fs::read(&head))?;
        let file = io(OpenOptions::new().read(true).write(true).open(path))?;
        let mut journal = Self {
            file,
            dir: io(File::open(root))?,
            root: root.to_owned(),
            run: config.run_id,
            config: config.config_hash,
            worker,
            recovery_profile: config
                .base
                .recovery
                .as_ref()
                .is_some_and(|scope| scope.target_worker == worker),
            sequence: 0,
            offset: 0,
            head: [0; 32],
            acks: BTreeMap::new(),
        };
        let bound = journal.bound_head(&bytes[..56]);
        if &bytes[..8] != b"I1SHEAD1" || bytes[56..] != bound {
            return Err(Error("invalid anchored root step head"));
        }
        let committed = u64::from_be_bytes(bytes[8..16].try_into().unwrap());
        let last = u64::from_be_bytes(bytes[16..24].try_into().unwrap());
        let length = io(journal.file.metadata())?.len();
        if length < committed || last < trusted.sequence {
            return Err(Error("missing committed root step bytes"));
        }
        let mut anchored = trusted == journal.receipt();
        while journal.offset < committed {
            let parsed = journal
                .read_frame(committed - journal.offset, config)?
                .ok_or(Error("torn protected root step frame"))?;
            journal.adopt_recovery_record(parsed)?;
            anchored |= trusted == journal.receipt();
        }
        if !anchored || journal.sequence != last || journal.head != bytes[24..56] {
            return Err(Error("root step history conflicts with trusted prefix"));
        }
        if length > committed {
            match journal.read_frame(length - committed, config)? {
                Some(record)
                    if Some(&record.0.grant) == authorized
                        && journal.offset.checked_add(record.2) == Some(length) =>
                {
                    let sequence = record.0.sequence;
                    let next_head = journal.head_bytes(length, sequence, record.1);
                    preserve_pending_head(
                        root,
                        &journal.dir,
                        ROOT_PENDING_HEAD,
                        Some((sequence, &next_head)),
                    )?;
                    io(journal.file.sync_all())?;
                    journal.adopt_recovery_record(record)?;
                    journal.sync_head(true)?;
                }
                Some(_) => return Err(Error("unauthorized complete root step descendant")),
                None => {
                    preserve_pending_head(root, &journal.dir, ROOT_PENDING_HEAD, None)?;
                    io(journal.file.set_len(committed))?;
                    io(journal.file.sync_all())?;
                }
            }
        } else {
            preserve_pending_head(root, &journal.dir, ROOT_PENDING_HEAD, None)?;
        }
        io(journal.file.seek(SeekFrom::Start(journal.offset)))?;
        Ok(journal)
    }

    pub fn latest(&self) -> Option<(&Grant, &Ack)> {
        self.acks
            .last_key_value()
            .map(|(_, pair)| (&pair.0, &pair.1))
    }

    pub fn receipt(&self) -> commonware_consensus::simplex::recovery::RecoveryReceipt {
        commonware_consensus::simplex::recovery::RecoveryReceipt {
            sequence: self.sequence,
            hash: self.head,
        }
    }

    fn check_ack_frames(&self, ack: &Ack) -> Result<()> {
        let mut bounded = ack.clone();
        bounded.logical_ack_hash = vec![0; 32];
        bounded.durable_step_hash = vec![0; 32];
        if (WorkerFrame {
            frame: Some(worker_frame::Frame::Ack(bounded.clone())),
        })
        .encoded_len()
            > wire::MAX_FRAME
        {
            return Err(Error("complete native Ack exceeds protobuf bound"));
        }
        if self.recovery_profile {
            let recovery = ack
                .recovery_receipt
                .as_ref()
                .ok_or(Error("recovery root step requires cut receipt"))?;
            if recovery.cut_hash.len() != 32
                || ack.grant_hash.len() != 32
                || ack.durable_step_sequence == 0
                || recovery
                    .auxiliary
                    .as_ref()
                    .is_none_or(|receipt| receipt.sequence == 0 || receipt.hash.len() != 32)
            {
                return Err(Error("invalid native recovery cut receipt"));
            }
            if recovered_frame(&bounded, u64::MAX)?.encoded_len() > wire::MAX_FRAME {
                return Err(Error("future RecoveredAck exceeds protobuf bound"));
            }
        } else if ack.recovery_receipt.is_some() {
            return Err(Error("Fresh rejects recovery cut receipt"));
        }
        Ok(())
    }

    fn bound_head(&self, prefix: &[u8]) -> Id {
        hash_parts(
            b"I1b/root-step-head/v1",
            &[&self.run, &self.config, &self.worker.to_be_bytes(), prefix],
        )
    }

    fn read_frame(
        &mut self,
        available: u64,
        config: &Resolved,
    ) -> Result<Option<(ParsedRecord, Id, u64)>> {
        if available < 16 {
            return Ok(None);
        }
        let mut length = [0; 16];
        io(self.file.read_exact(&mut length))?;
        let size = u64::from_be_bytes(length[..8].try_into().unwrap());
        let complement = u64::from_be_bytes(length[8..].try_into().unwrap());
        if size != !complement || size == 0 || size > FRAME_LIMIT as u64 {
            return Err(Error("invalid root step frame size"));
        }
        let frame_len = size + 48;
        if frame_len > available {
            return Ok(None);
        }
        let mut bytes = vec![0; size as usize];
        let mut hash = [0; 32];
        io(self.file.read_exact(&mut bytes))?;
        io(self.file.read_exact(&mut hash))?;
        if hash != hash_parts(b"I1b/root-step-record/v1", &[&bytes]) {
            return Err(Error("invalid root step record checksum"));
        }
        let parsed: ParsedRecord =
            serde_json::from_slice(&bytes).map_err(|_| Error("invalid native root step record"))?;
        let canonical = serde_json::to_vec(&Record {
            schema: parsed.schema,
            sequence: parsed.sequence,
            previous_record_hash: parsed.previous_record_hash,
            grant: &parsed.grant,
            ack: &parsed.ack,
        })
        .map_err(|_| Error("encode validated root step record"))?;
        let mut expected = config.binding(self.worker);
        expected.incarnation = parsed.grant.binding.as_ref().map_or(0, |b| b.incarnation);
        if canonical != bytes
            || parsed.schema != 1
            || Some(parsed.sequence) != self.sequence.checked_add(1)
            || parsed.previous_record_hash != self.head
            || expected.incarnation == 0
            || parsed.grant.binding.as_ref() != Some(&expected)
            || parsed.ack.binding.as_ref() != Some(&expected)
            || parsed.grant.logical_step_id == 0
            || parsed.grant.event_id == 0
            || parsed.grant.tie_rank != wire::tie_hash(config.base.seed, parsed.grant.event_id)
            || parsed.grant.grant_hash != wire::grant_hash(&parsed.grant)
            || parsed.ack.event_id != parsed.grant.event_id
            || parsed.ack.logical_step_id != parsed.grant.logical_step_id
            || parsed.ack.grant_hash != parsed.grant.grant_hash
            || parsed.ack.durable_step_sequence != parsed.sequence
            || !parsed.ack.durable_step_hash.is_empty()
            || parsed.ack.logical_ack_hash != wire::ack_hash(&parsed.ack)
            || self.acks.last_key_value().is_some_and(|(step, (old, _))| {
                *step >= parsed.grant.logical_step_id
                    || old.tick_us > parsed.grant.tick_us
                    || old.binding.as_ref().is_some_and(|b| {
                        b.incarnation > expected.incarnation
                            || expected.incarnation - b.incarnation > 1
                    })
            })
        {
            return Err(Error("root step record history or binding mismatch"));
        }
        self.check_ack_frames(&parsed.ack)?;
        Ok(Some((parsed, hash, frame_len)))
    }

    fn adopt_recovery_record(
        &mut self,
        (record, hash, bytes): (ParsedRecord, Id, u64),
    ) -> Result<()> {
        let mut ack = record.ack;
        self.sequence = record.sequence;
        self.head = hash;
        self.offset = self
            .offset
            .checked_add(bytes)
            .ok_or(Error("root step offset exhausted"))?;
        ack.durable_step_hash = hash.to_vec();
        self.acks
            .insert(record.grant.logical_step_id, (record.grant, ack));
        Ok(())
    }

    fn head_bytes(&self, offset: u64, sequence: u64, hash: Id) -> [u8; 88] {
        let mut bytes = [0; 88];
        bytes[..8].copy_from_slice(b"I1SHEAD1");
        bytes[8..16].copy_from_slice(&offset.to_be_bytes());
        bytes[16..24].copy_from_slice(&sequence.to_be_bytes());
        bytes[24..56].copy_from_slice(&hash);
        let checksum = self.bound_head(&bytes[..56]);
        bytes[56..].copy_from_slice(&checksum);
        bytes
    }

    fn sync_head(&self, rename: bool) -> Result<()> {
        let bytes = self.head_bytes(self.offset, self.sequence, self.head);
        let head = self.root.join("root-step.head");
        let file = if rename {
            self.root.join(ROOT_PENDING_HEAD)
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
        self.check_ack_frames(&ack)?;
        let record = Record {
            schema: 1,
            sequence,
            previous_record_hash: self.head,
            grant: &grant,
            ack: &ack,
        };
        let bytes =
            serde_json::to_vec(&record).map_err(|_| Error("root step record encode failure"))?;
        if bytes.len() > FRAME_LIMIT {
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

fn recovered_frame(original: &Ack, incarnation: u64) -> Result<WorkerFrame> {
    let mut binding = original
        .binding
        .clone()
        .ok_or(Error("missing original Ack binding"))?;
    binding.incarnation = incarnation;
    let receipt = original
        .recovery_receipt
        .as_ref()
        .ok_or(Error("missing original Ack cut"))?;
    Ok(WorkerFrame {
        frame: Some(worker_frame::Frame::RecoveredAck(RecoveredAck {
            current_binding: Some(binding),
            original_ack_bytes: original.encode_to_vec(),
            original_grant_hash: original.grant_hash.clone(),
            root_receipt: Some(Receipt {
                sequence: original.durable_step_sequence,
                hash: original.durable_step_hash.clone(),
            }),
            cut_hash: receipt.cut_hash.clone(),
        })),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_wire::{Empty, Input, input};

    #[test]
    fn anchored_root_replays_exact_ack_from_authorized_synced_tail() {
        let root = (0..u64::MAX)
            .map(|i| {
                std::env::temp_dir().join(format!("native-root-reopen-{}-{i}", std::process::id()))
            })
            .find(|path| fs::create_dir(path).is_ok())
            .unwrap();
        let (config, _) = Resolved::generate("healthy", 7).unwrap();
        let mut journal = StepJournal::create(&root, &config, 0).unwrap();
        let make = |step| {
            let binding = Some(config.binding(0));
            let mut grant = Grant {
                binding: binding.clone(),
                logical_step_id: step,
                event_id: step,
                tick_us: 1_000_000 + step,
                tie_rank: wire::tie_hash(7, step).to_vec(),
                input: Some(Input {
                    kind: Some(input::Kind::RequestSample(Empty {})),
                }),
                ..Default::default()
            };
            grant.grant_hash = wire::grant_hash(&grant).to_vec();
            let ack = Ack {
                binding,
                logical_step_id: step,
                event_id: step,
                grant_hash: grant.grant_hash.clone(),
                status: 1,
                ..Default::default()
            };
            (grant, ack)
        };
        let (g1, a1) = make(1);
        let (g2, a2) = make(2);
        let a1 = journal.commit(g1.clone(), a1).unwrap();
        let anchor = journal.receipt();
        let head_path = root.join("root-step.head");
        let first_head = fs::read(&head_path).unwrap();
        let a2 = journal.commit(g2.clone(), a2).unwrap();
        let completed = journal.receipt();
        let second_head = fs::read(&head_path).unwrap();
        drop(journal);
        fs::write(&head_path, &first_head).unwrap();
        let pending = root.join(ROOT_PENDING_HEAD);
        fs::write(&pending, &second_head[..41]).unwrap();

        assert!(StepJournal::open_recovery(&root, &config, 0, anchor, None).is_err());
        assert_eq!(fs::read(&head_path).unwrap(), first_head);
        assert_eq!(fs::read(&pending).unwrap(), &second_head[..41]);
        let journal = StepJournal::open_recovery(&root, &config, 0, anchor, Some(&g2)).unwrap();
        assert_eq!(journal.receipt(), completed);
        assert_eq!(fs::read(&head_path).unwrap(), second_head);
        assert!(!pending.exists());
        assert_eq!(
            fs::read(root.join(format!("{ROOT_PENDING_HEAD}.recovery.2.41"))).unwrap(),
            &second_head[..41],
        );
        assert_eq!(
            journal.cached(&g1).unwrap().unwrap().encode_to_vec(),
            a1.encode_to_vec()
        );
        assert_eq!(
            journal.cached(&g2).unwrap().unwrap().encode_to_vec(),
            a2.encode_to_vec()
        );
        drop(journal);

        let data_path = root.join("root-step");
        let complete_file = fs::read(&data_path).unwrap();
        fs::write(&data_path, &complete_file[..complete_file.len() - 1]).unwrap();
        assert!(StepJournal::open_recovery(&root, &config, 0, anchor, Some(&g2)).is_err());
        assert_eq!(fs::read(&data_path).unwrap().len(), complete_file.len() - 1);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn recovery_frame_limit_is_checked_before_root_mutation() {
        use crate::native_wire::{RecoveryCutReceipt, worker_frame::Frame};
        let root = (0..u64::MAX)
            .map(|i| {
                std::env::temp_dir()
                    .join(format!("native-recovered-frame-{}-{i}", std::process::id()))
            })
            .find(|path| fs::create_dir(path).is_ok())
            .unwrap();
        let (mut config, _) = Resolved::generate("healthy", 7).unwrap();
        config.base.recovery = Some(crate::native_config::RecoverySchedule {
            target_worker: 0,
            boundary: "TEST".into(),
            downtime_us: 0,
        });
        let mut journal = StepJournal::create(&root, &config, 0).unwrap();
        let mut grant = Grant {
            binding: Some(config.binding(0)),
            logical_step_id: 1,
            event_id: 1,
            tick_us: 1_000_000,
            tie_rank: wire::tie_hash(7, 1).to_vec(),
            input: Some(Input {
                kind: Some(input::Kind::RequestSample(Empty {})),
            }),
            ..Default::default()
        };
        grant.grant_hash = wire::grant_hash(&grant).to_vec();
        let mut ack = Ack {
            binding: grant.binding.clone(),
            logical_step_id: 1,
            event_id: 1,
            grant_hash: grant.grant_hash.clone(),
            status: 1,
            recovery_receipt: Some(RecoveryCutReceipt {
                cut_hash: vec![22; 32],
                auxiliary: Some(Receipt {
                    sequence: 3,
                    hash: vec![33; 32],
                }),
            }),
            ..Default::default()
        };
        let mut sized = ack.clone();
        sized.durable_step_sequence = 1;
        sized.durable_step_hash = vec![0; 32];
        sized.logical_ack_hash = vec![0; 32];
        // Compute the content length against a directly constructed protobuf
        // boundary frame, separate from the journal's preflight sizing path.
        let outer_len = |inner: &Ack, incarnation| {
            let mut binding = grant.binding.clone().unwrap();
            binding.incarnation = incarnation;
            WorkerFrame {
                frame: Some(Frame::RecoveredAck(RecoveredAck {
                    current_binding: Some(binding),
                    original_ack_bytes: inner.encode_to_vec(),
                    original_grant_hash: grant.grant_hash.clone(),
                    root_receipt: Some(Receipt {
                        sequence: 1,
                        hash: vec![77; 32],
                    }),
                    cut_hash: vec![22; 32],
                })),
            }
            .encoded_len()
        };
        assert_eq!(outer_len(&sized, 128), outer_len(&sized, 127) + 1);
        assert_eq!(outer_len(&sized, u64::MAX), outer_len(&sized, 128) + 8);
        sized.application_events_json = vec![b'x'; wire::MAX_FRAME];
        let overhead = outer_len(&sized, u64::MAX) - sized.application_events_json.len();
        let limit = wire::MAX_FRAME - overhead;
        sized.application_events_json.truncate(limit);
        assert_eq!(outer_len(&sized, u64::MAX), wire::MAX_FRAME);
        let before = fs::read(root.join("root-step.head")).unwrap();
        ack.application_events_json = vec![b'x'; limit + 1];
        assert!(journal.commit(grant.clone(), ack.clone()).is_err());
        assert!(journal.cached(&grant).unwrap().is_none());
        assert_eq!(fs::metadata(root.join("root-step")).unwrap().len(), 0);
        assert_eq!(fs::read(root.join("root-step.head")).unwrap(), before);
        ack.application_events_json.truncate(limit);
        let result = journal.commit(grant.clone(), ack).unwrap();
        assert_eq!(outer_len(&result, u64::MAX), wire::MAX_FRAME);
        drop(journal);
        fs::remove_dir_all(root).unwrap();
    }
}
