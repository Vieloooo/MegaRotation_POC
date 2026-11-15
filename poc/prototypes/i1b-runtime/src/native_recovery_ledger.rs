use crate::{Error, Id, Result, hash_parts};
use commonware_consensus::simplex::recovery::RecoveryReceipt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Read, Seek, SeekFrom, Write},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

const MAGIC: &[u8; 8] = b"I1RCHEAD";
const LIMIT: usize = 16 * 1024 * 1024;
const MARKER: &[u8] = b",\"data\":";
pub(crate) const AUX_PENDING_HEAD: &str = "native-aux.head.pending";
pub(crate) const ROOT_PENDING_HEAD: &str = "root-step.head.pending";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Header {
    pub schema: u32,
    pub sequence: u64,
    pub previous_record_hash: String,
    pub grant_intent_ref: Option<String>,
    pub kind: String,
}

#[derive(Clone, Debug)]
pub(crate) struct LedgerRecord {
    pub header: Header,
    pub data: Vec<u8>,
    pub receipt: RecoveryReceipt,
}

#[derive(Clone, Copy)]
pub(crate) struct Identity {
    pub run: Id,
    pub config: Id,
    pub worker: u32,
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
    fn receipt(self) -> RecoveryReceipt {
        RecoveryReceipt {
            sequence: self.sequence,
            hash: self.hash,
        }
    }
    fn encode(self, id: Identity) -> [u8; 88] {
        let mut out = [0; 88];
        out[..8].copy_from_slice(MAGIC);
        out[8..16].copy_from_slice(&self.offset.to_be_bytes());
        out[16..24].copy_from_slice(&self.sequence.to_be_bytes());
        out[24..56].copy_from_slice(&self.hash);
        let sum = hash_parts(
            b"I1b/recovery-head/v1",
            &[&id.run, &id.config, &id.worker.to_be_bytes(), &out[..56]],
        );
        out[56..].copy_from_slice(&sum);
        out
    }
    fn decode(bytes: &[u8], id: Identity) -> Result<Self> {
        require(bytes.len() == 88, "invalid native recovery head length")?;
        let head = Self {
            offset: u64::from_be_bytes(bytes[8..16].try_into().unwrap()),
            sequence: u64::from_be_bytes(bytes[16..24].try_into().unwrap()),
            hash: bytes[24..56].try_into().unwrap(),
        };
        require(
            head.encode(id) == bytes,
            "native recovery head binding or checksum mismatch",
        )?;
        Ok(head)
    }
}
pub(crate) struct RecoveryLedger {
    file: File,
    dir: File,
    root: PathBuf,
    identity: Identity,
    head: Head,
    records: Vec<LedgerRecord>,
}
impl RecoveryLedger {
    pub fn create(root: &Path, id: Identity) -> Result<Self> {
        let mut ledger = Self {
            file: io(OpenOptions::new()
                .create_new(true)
                .read(true)
                .write(true)
                .open(root.join("native-aux")))?,
            dir: io(File::open(root))?,
            root: root.to_owned(),
            identity: id,
            head: Head::EMPTY,
            records: vec![],
        };
        io(ledger.file.sync_all())?;
        ledger.install_head(Head::EMPTY, false)?;
        Ok(ledger)
    }
    /// Open an exclusive mutable working copy. Keep the original crash image
    /// separately. Recovery may preserve an owned scratch and promote a tail.
    pub fn open(
        root: &Path,
        id: Identity,
        minimum: RecoveryReceipt,
        authorized_tail: Option<&dyn Fn(&LedgerRecord) -> bool>,
    ) -> Result<Self> {
        let head_path = root.join("native-aux.head");
        require(
            io(fs::symlink_metadata(&head_path))?.file_type().is_file(),
            "invalid native recovery head path",
        )?;
        require(
            io(fs::metadata(&head_path))?.len() == 88,
            "invalid native recovery head length",
        )?;
        let committed = Head::decode(&io(fs::read(head_path))?, id)?;
        require(
            minimum.sequence <= committed.sequence,
            "native recovery head below trusted prefix",
        )?;
        let path = root.join("native-aux");
        require(
            io(fs::symlink_metadata(&path))?.file_type().is_file(),
            "invalid native recovery ledger path",
        )?;
        let file = io(OpenOptions::new().read(true).write(true).open(path))?;
        let length = io(file.metadata())?.len();
        require(
            committed.offset <= length,
            "missing committed native recovery bytes",
        )?;
        let mut ledger = Self {
            file,
            dir: io(File::open(root))?,
            root: root.to_owned(),
            identity: id,
            head: Head::EMPTY,
            records: vec![],
        };
        let mut anchored = minimum == Head::EMPTY.receipt();
        while ledger.head.offset < committed.offset {
            let item = ledger
                .read_frame(committed.offset - ledger.head.offset)?
                .ok_or(Error(
                    "torn frame intersects committed native recovery bytes",
                ))?;
            ledger.adopt(item);
            anchored |= ledger.head.receipt() == minimum;
        }
        require(
            ledger.head == committed && anchored,
            "native recovery chain conflicts with trusted snapshot",
        )?;
        if length > committed.offset {
            match ledger.read_frame(length - committed.offset)? {
                Some((head, item)) => {
                    require(
                        head.offset == length && authorized_tail.is_some_and(|allow| allow(&item)),
                        "unauthorized native recovery descendant",
                    )?;
                    preserve_pending_head(
                        root,
                        &ledger.dir,
                        AUX_PENDING_HEAD,
                        Some((head.sequence, &head.encode(id))),
                    )?;
                    io(ledger.file.sync_all())?;
                    ledger.install_head(head, true)?;
                    ledger.records.push(item);
                }
                None => {
                    preserve_pending_head(root, &ledger.dir, AUX_PENDING_HEAD, None)?;
                    io(ledger.file.set_len(committed.offset))?;
                    io(ledger.file.sync_all())?;
                }
            }
        } else {
            preserve_pending_head(root, &ledger.dir, AUX_PENDING_HEAD, None)?;
        }
        io(ledger.file.seek(SeekFrom::Start(ledger.head.offset)))?;
        Ok(ledger)
    }
    pub fn receipt(&self) -> RecoveryReceipt {
        self.head.receipt()
    }
    pub fn records(&self) -> &[LedgerRecord] {
        &self.records
    }
    pub fn append(
        &mut self,
        kind: &str,
        grant_ref: Option<Id>,
        data: &impl Serialize,
    ) -> Result<RecoveryReceipt> {
        let data = serde_json::to_vec(data).map_err(|_| Error("encode recovery data"))?;
        self.append_canonical(kind, grant_ref, data)
    }
    pub fn append_canonical(
        &mut self,
        kind: &str,
        grant_ref: Option<Id>,
        data: Vec<u8>,
    ) -> Result<RecoveryReceipt> {
        require(
            io(self.file.metadata())?.len() == self.head.offset,
            "unexpected existing native recovery tail",
        )?;
        let header = Header {
            schema: 1,
            sequence: self
                .head
                .sequence
                .checked_add(1)
                .ok_or(Error("auxiliary sequence overflow"))?,
            previous_record_hash: hex(&self.head.hash),
            grant_intent_ref: grant_ref.map(|id| hex(&id)),
            kind: kind.to_owned(),
        };
        let payload = encode(&header, &data)?;
        let hash = hash_parts(b"I1b/recovery-record/v1", &[&payload]);
        let length = payload.len() as u64;
        io(self.file.write_all(&length.to_be_bytes()))?;
        io(self.file.write_all(&(!length).to_be_bytes()))?;
        io(self.file.write_all(&payload))?;
        io(self.file.write_all(&hash))?;
        io(self.file.sync_all())?;
        let offset = self
            .head
            .offset
            .checked_add(length + 48)
            .ok_or(Error("auxiliary offset overflow"))?;
        let head = Head {
            offset,
            sequence: header.sequence,
            hash,
        };
        self.install_head(head, true)?;
        let receipt = head.receipt();
        self.records.push(LedgerRecord {
            header,
            data,
            receipt,
        });
        Ok(receipt)
    }
    fn install_head(&mut self, head: Head, replace: bool) -> Result<()> {
        let dest = self.root.join("native-aux.head");
        let path = if replace {
            self.root.join(AUX_PENDING_HEAD)
        } else {
            dest.clone()
        };
        let mut f = io(OpenOptions::new().write(true).create_new(true).open(&path))?;
        io(f.write_all(&head.encode(self.identity)))?;
        io(f.sync_all())?;
        if replace {
            io(fs::rename(path, dest))?;
        }
        io(self.dir.sync_all())?;
        self.head = head;
        Ok(())
    }
    fn read_frame(&mut self, available: u64) -> Result<Option<(Head, LedgerRecord)>> {
        if available < 16 {
            return Ok(None);
        }
        let mut header = [0; 16];
        io(self.file.read_exact(&mut header))?;
        let size = u64::from_be_bytes(header[..8].try_into().unwrap());
        let inv = u64::from_be_bytes(header[8..].try_into().unwrap());
        require(
            size == !inv && size > 0 && size <= LIMIT as u64,
            "invalid native recovery record length",
        )?;
        let frame = size + 48;
        if frame > available {
            return Ok(None);
        }
        let mut payload = vec![0; size as usize];
        let mut stored = [0; 32];
        io(self.file.read_exact(&mut payload))?;
        io(self.file.read_exact(&mut stored))?;
        require(
            hash_parts(b"I1b/recovery-record/v1", &[&payload]) == stored,
            "native recovery record checksum mismatch",
        )?;
        let (header, data) = parse(&payload)?;
        require(
            header.schema == 1
                && Some(header.sequence) == self.head.sequence.checked_add(1)
                && header.previous_record_hash == hex(&self.head.hash),
            "native recovery chain sequence mismatch",
        )?;
        let head = Head {
            offset: self
                .head
                .offset
                .checked_add(frame)
                .ok_or(Error("native recovery ledger offset overflow"))?,
            sequence: header.sequence,
            hash: stored,
        };
        let item = LedgerRecord {
            header,
            data,
            receipt: head.receipt(),
        };
        Ok(Some((head, item)))
    }
    fn adopt(&mut self, (head, item): (Head, LedgerRecord)) {
        self.head = head;
        self.records.push(item);
    }
}

struct OwnedScratch {
    file: File,
    bytes: Vec<u8>,
}

fn read_owned_scratch(dir: &File, path: &Path) -> Result<Option<OwnedScratch>> {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(Error("cannot inspect recovery head scratch")),
    };
    require(
        meta.file_type().is_file()
            && meta.nlink() == 1
            && meta.uid() == io(dir.metadata())?.uid()
            && meta.len() <= 88,
        "invalid owned recovery head scratch",
    )?;
    let mut file = io(File::open(path))?;
    let opened = io(file.metadata())?;
    require(
        opened.dev() == meta.dev()
            && opened.ino() == meta.ino()
            && opened.uid() == meta.uid()
            && opened.nlink() == 1
            && opened.len() == meta.len(),
        "changed recovery head scratch",
    )?;
    let mut bytes = Vec::with_capacity(88);
    io((&mut file).take(89).read_to_end(&mut bytes))?;
    require(
        bytes.len() as u64 == meta.len(),
        "changed recovery head scratch size",
    )?;
    Ok(Some(OwnedScratch { file, bytes }))
}

/// Preserve a crash-left scratch from the exclusive working directory. The
/// caller supplies a head only after authorizing its complete synced record.
/// Evidence files are consulted here solely to make preservation retryable.
pub(crate) fn preserve_pending_head(
    root: &Path,
    dir: &File,
    name: &str,
    candidate: Option<(u64, &[u8; 88])>,
) -> Result<()> {
    let pending_path = root.join(name);
    let Some(pending) = read_owned_scratch(dir, &pending_path)? else {
        return Ok(());
    };
    let (sequence, expected) =
        candidate.ok_or(Error("recovery head scratch has no complete descendant"))?;
    require(
        expected.starts_with(&pending.bytes),
        "recovery head scratch differs from authorized descendant",
    )?;
    let evidence_path = root.join(format!(
        "{name}.recovery.{sequence}.{}",
        pending.bytes.len()
    ));
    if let Some(evidence) = read_owned_scratch(dir, &evidence_path)? {
        require(
            evidence.bytes == pending.bytes,
            "conflicting preserved recovery head scratch",
        )?;
        io(evidence.file.sync_all())?;
        // Make the already-matching evidence durable before disposal.
        io(dir.sync_all())?;
        io(fs::remove_file(pending_path))?;
    } else {
        io(pending.file.sync_all())?;
        io(fs::rename(pending_path, evidence_path))?;
    }
    io(dir.sync_all())
}

/// Copy both original scratch images without using either to infer a head.
pub(crate) fn copy_pending_heads(source: &Path, working: &Path) -> Result<()> {
    let source_dir = io(File::open(source))?;
    let scratch = [AUX_PENDING_HEAD, ROOT_PENDING_HEAD]
        .map(|name| read_owned_scratch(&source_dir, &source.join(name)));
    let [aux, root] = scratch;
    for (name, image) in [(AUX_PENDING_HEAD, aux?), (ROOT_PENDING_HEAD, root?)] {
        if let Some(image) = image {
            let mut target = io(OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(working.join(name)))?;
            io(target.write_all(&image.bytes))?;
            io(target.sync_all())?;
        }
    }
    io(io(File::open(working))?.sync_all())
}

fn encode(header: &Header, data: &[u8]) -> Result<Vec<u8>> {
    require(
        !header.kind.is_empty()
            && header
                .kind
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b == b'_'),
        "invalid native recovery record kind",
    )?;
    require(
        header.previous_record_hash.len() == 64
            && canonical_hex(&header.previous_record_hash)
            && header
                .grant_intent_ref
                .as_ref()
                .is_none_or(|id| id.len() == 64 && canonical_hex(id)),
        "invalid native recovery hash encoding",
    )?;
    let _: serde_json::Value =
        serde_json::from_slice(data).map_err(|_| Error("invalid native recovery data"))?;
    let mut bytes =
        serde_json::to_vec(header).map_err(|_| Error("encode native recovery metadata"))?;
    bytes.pop();
    bytes.extend_from_slice(MARKER);
    bytes.extend_from_slice(data);
    bytes.push(b'}');
    require(bytes.len() <= LIMIT, "native recovery record exceeds bound")?;
    Ok(bytes)
}
fn parse(payload: &[u8]) -> Result<(Header, Vec<u8>)> {
    let marker = payload
        .windows(MARKER.len())
        .position(|x| x == MARKER)
        .ok_or(Error("native recovery data absent"))?;
    require(
        payload.last() == Some(&b'}'),
        "invalid native recovery record terminator",
    )?;
    let mut header = payload[..marker].to_vec();
    header.push(b'}');
    let header: Header =
        serde_json::from_slice(&header).map_err(|_| Error("decode native recovery header"))?;
    let data = &payload[marker + MARKER.len()..payload.len() - 1];
    require(
        encode(&header, data)? == payload,
        "noncanonical native recovery record",
    )?;
    Ok((header, data.to_vec()))
}
fn canonical_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub(crate) fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut out, "{byte:02x}").unwrap();
    }
    out
}
fn io<T>(value: std::io::Result<T>) -> Result<T> {
    value.map_err(|_| Error("native recovery filesystem operation failed"))
}
fn require(condition: bool, message: &'static str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(Error(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchored_native_aux_descendant_and_protected_truncation() {
        let stem = std::env::temp_dir();
        let root = (0..u64::MAX)
            .map(|index| stem.join(format!("native-aux-{}-{index}", std::process::id())))
            .find(|path| fs::create_dir(path).is_ok())
            .unwrap();
        let identity = Identity {
            run: [1; 32],
            config: [2; 32],
            worker: 1,
        };
        let mut ledger = RecoveryLedger::create(&root, identity).unwrap();
        let floor = ledger.append("GRANT_INTENT", None, &[7u64, 17]).unwrap();
        let head = ledger
            .append("PREPARATION", Some(floor.hash), &[11u64, 12])
            .unwrap();
        drop(ledger);
        let ledger = RecoveryLedger::open(&root, identity, floor, None).unwrap();
        assert_eq!(ledger.receipt(), head);
        assert_eq!(ledger.records()[1].data, b"[11,12]");
        assert_eq!(ledger.records()[1].receipt, head);
        assert_eq!(ledger.records()[0].header.kind, "GRANT_INTENT");
        drop(ledger);
        let wrong = RecoveryReceipt {
            sequence: floor.sequence,
            hash: [9; 32],
        };
        assert!(RecoveryLedger::open(&root, identity, wrong, None).is_err());
        let path = root.join("native-aux");
        let original = fs::read(&path).unwrap();
        fs::write(&path, &original[..original.len() - 1]).unwrap();
        assert!(RecoveryLedger::open(&root, identity, floor, None).is_err());
        assert_eq!(fs::read(&path).unwrap(), original[..original.len() - 1]);
        fs::remove_dir_all(root).unwrap();
    }
    struct InterruptedHead {
        root: PathBuf,
        identity: Identity,
        floor: RecoveryReceipt,
        next: RecoveryReceipt,
        old_head: Vec<u8>,
        new_head: Vec<u8>,
    }

    impl InterruptedHead {
        fn new() -> Self {
            let root = (0..u64::MAX)
                .map(|index| {
                    std::env::temp_dir()
                        .join(format!("aux-scratch-test-{}-{index}", std::process::id()))
                })
                .find(|path| fs::create_dir(path).is_ok())
                .unwrap();
            let identity = Identity {
                run: [5; 32],
                config: [6; 32],
                worker: 1,
            };
            let mut ledger = RecoveryLedger::create(&root, identity).unwrap();
            let floor = ledger.append("GRANT_INTENT", None, &[1]).unwrap();
            let old_head = fs::read(root.join("native-aux.head")).unwrap();
            let next = ledger
                .append("PREPARATION", Some(floor.hash), &[2])
                .unwrap();
            let new_head = fs::read(root.join("native-aux.head")).unwrap();
            drop(ledger);
            fs::write(root.join("native-aux.head"), &old_head).unwrap();
            Self {
                root,
                identity,
                floor,
                next,
                old_head,
                new_head,
            }
        }

        fn open(&self, authorized: bool) -> Result<RecoveryLedger> {
            RecoveryLedger::open(
                &self.root,
                self.identity,
                self.floor,
                Some(&|record| authorized && record.receipt == self.next),
            )
        }

        fn evidence(&self, length: usize) -> PathBuf {
            self.root
                .join(format!("{AUX_PENDING_HEAD}.recovery.2.{length}"))
        }
    }

    impl Drop for InterruptedHead {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }

    #[test]
    fn preserves_each_valid_scratch_shape_through_retry_and_append() {
        for length in [0, 1, 43, 87, 88] {
            let fixture = InterruptedHead::new();
            let pending = fixture.root.join(AUX_PENDING_HEAD);
            let bytes = &fixture.new_head[..length];
            fs::write(&pending, bytes).unwrap();
            let evidence = fixture.evidence(length);
            let mut opened = fixture.open(true).unwrap();
            assert_eq!(opened.receipt(), fixture.next);
            assert!(!pending.exists());
            assert_eq!(fs::read(&evidence).unwrap(), bytes);
            drop(opened);

            // Emulate another interruption after preserving the first scratch:
            // the old committed head remains and the head reinstall has retried.
            fs::write(fixture.root.join("native-aux.head"), &fixture.old_head).unwrap();
            fs::write(&pending, bytes).unwrap();
            opened = fixture.open(true).unwrap();
            assert!(!pending.exists());
            assert_eq!(fs::read(evidence).unwrap(), bytes);
            assert_eq!(
                opened
                    .append("FOLLOWUP", Some(fixture.next.hash), &[3])
                    .unwrap()
                    .sequence,
                3
            );
        }
    }

    #[test]
    fn rejects_invalid_scratch_before_changing_ledger_or_evidence() {
        use std::os::unix::fs::symlink;
        for case in [
            "wrong_prefix",
            "wrong_binding",
            "oversize",
            "symlink",
            "directory",
            "alias",
            "conflicting_backup",
            "missing_tail",
            "torn_tail",
            "unauthorized_tail",
        ] {
            let fixture = InterruptedHead::new();
            let pending = fixture.root.join(AUX_PENDING_HEAD);
            let data = fixture.root.join("native-aux");
            let committed_offset = u64::from_be_bytes(fixture.old_head[8..16].try_into().unwrap());
            match case {
                "wrong_prefix" => fs::write(&pending, b"I1RX").unwrap(),
                "wrong_binding" => {
                    let head = Head::decode(&fixture.new_head, fixture.identity).unwrap();
                    let foreign = Identity {
                        worker: 2,
                        ..fixture.identity
                    };
                    fs::write(&pending, head.encode(foreign)).unwrap();
                }
                "oversize" => fs::write(&pending, [0; 89]).unwrap(),
                "symlink" => symlink(fixture.root.join("native-aux.head"), &pending).unwrap(),
                "directory" => fs::create_dir(&pending).unwrap(),
                "alias" => fs::hard_link(fixture.root.join("native-aux.head"), &pending).unwrap(),
                "conflicting_backup" => {
                    fs::write(&pending, &fixture.new_head[..4]).unwrap();
                    fs::write(fixture.evidence(4), b"abcd").unwrap();
                }
                "missing_tail" | "torn_tail" => {
                    fs::write(&pending, []).unwrap();
                    let end = if case == "missing_tail" {
                        committed_offset
                    } else {
                        committed_offset + 16
                    };
                    File::options()
                        .write(true)
                        .open(&data)
                        .unwrap()
                        .set_len(end)
                        .unwrap();
                }
                "unauthorized_tail" => fs::write(&pending, &fixture.new_head).unwrap(),
                _ => unreachable!(),
            }
            let before = fs::read(&data).unwrap();
            let pending_meta = fs::symlink_metadata(&pending).unwrap();
            let pending_bytes = pending_meta
                .file_type()
                .is_file()
                .then(|| fs::read(&pending).unwrap());
            assert!(fixture.open(case != "unauthorized_tail").is_err(), "{case}");
            let after = fs::symlink_metadata(&pending).unwrap();
            assert_eq!(
                (after.dev(), after.ino(), after.len()),
                (pending_meta.dev(), pending_meta.ino(), pending_meta.len()),
                "{case}"
            );
            if let Some(bytes) = pending_bytes {
                assert_eq!(fs::read(&pending).unwrap(), bytes, "{case}");
            }
            assert_eq!(fs::read(&data).unwrap(), before, "{case}");
            assert_eq!(
                fs::read(fixture.root.join("native-aux.head")).unwrap(),
                fixture.old_head,
                "{case}"
            );
            if case == "conflicting_backup" {
                assert_eq!(fs::read(fixture.evidence(4)).unwrap(), b"abcd");
            }
        }
    }

    #[test]
    fn copies_pending_images_without_changing_the_original() {
        let fixture = InterruptedHead::new();
        let working = fixture.root.join("working");
        fs::create_dir(&working).unwrap();
        fs::write(fixture.root.join(AUX_PENDING_HEAD), &fixture.new_head[..20]).unwrap();
        fs::write(fixture.root.join(ROOT_PENDING_HEAD), []).unwrap();
        copy_pending_heads(&fixture.root, &working).unwrap();
        for name in [AUX_PENDING_HEAD, ROOT_PENDING_HEAD] {
            assert_eq!(
                fs::read(working.join(name)).unwrap(),
                fs::read(fixture.root.join(name)).unwrap()
            );
            assert_ne!(
                fs::metadata(working.join(name)).unwrap().ino(),
                fs::metadata(fixture.root.join(name)).unwrap().ino()
            );
        }
        assert_eq!(
            fs::read(fixture.root.join("native-aux.head")).unwrap(),
            fixture.old_head
        );

        let rejected = fixture.root.join("rejected");
        fs::create_dir(&rejected).unwrap();
        fs::remove_file(fixture.root.join(ROOT_PENDING_HEAD)).unwrap();
        std::os::unix::fs::symlink(
            fixture.root.join("native-aux.head"),
            fixture.root.join(ROOT_PENDING_HEAD),
        )
        .unwrap();
        assert!(copy_pending_heads(&fixture.root, &rejected).is_err());
        assert!(fs::read_dir(rejected).unwrap().next().is_none());
        assert!(
            fs::symlink_metadata(fixture.root.join(ROOT_PENDING_HEAD))
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }
}
