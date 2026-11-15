use crate::{
    Error, IoIdentity, IoOperation, IoResult, IoTarget, Result, WitnessHead, WitnessPolicy,
    hash_parts,
    storage::WitnessIo,
    witness::{MAX_BLOB_BYTES, WitnessRuntime, require},
    witness_ledger::{LedgerEntry, RecoveryLink, WitnessLedger, bounded_read, io, side, snapshot},
};
use commonware_cryptography::Crc32;
use commonware_runtime::{
    Blob, BlobLayout, BlobVersion, IoBufs, Runner as _, Storage, WriteOptions,
    tokio::{Config, Runner},
};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

const HEADER: usize = 4096;

pub struct RecoveryPaths<'a> {
    pub source_storage: &'a Path,
    pub source_ledger: &'a Path,
    pub fresh_target: &'a Path,
}

#[derive(Debug)]
pub struct RecoveryReport {
    pub new_head: WitnessHead,
    pub source: RecoveryLink,
    pub records: Vec<crate::LedgerRecord>,
}

/// Offline verifier-only reconstruction. The old native directory is never opened
/// by a Commonware Runner. A new Runner produces the target generation's barriers.
pub fn recover_witness_generation(
    paths: RecoveryPaths<'_>,
    source: WitnessPolicy,
    target: WitnessPolicy,
    expected: WitnessHead,
) -> Result<RecoveryReport> {
    let old = &source.seal().config;
    let mut required = source.seal().clone();
    required.config.generation = old
        .generation
        .checked_add(1)
        .ok_or(Error("witness generation overflow"))?;
    require(
        *target.seal() == required,
        "recovery changes witness policy or generation",
    )?;
    let old_root = io(paths.source_storage.canonicalize())?;
    let source_file = io(paths.source_ledger.canonicalize())?;
    let parent = paths
        .fresh_target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let target_parent = io(parent.canonicalize())?;
    let target_path = target_parent.join(
        paths
            .fresh_target
            .file_name()
            .ok_or(Error("invalid recovery target"))?,
    );
    require(
        !target_path.starts_with(&old_root)
            && !source_file.starts_with(&target_path)
            && fs::symlink_metadata(&target_path)
                .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound),
        "recovery target must be separate and absent",
    )?;

    // Capture bounded bytes without invoking native open, replay or sync on the source.
    let source_partition = old_root.join(&old.partition);
    require(
        io(fs::symlink_metadata(&source_partition))?
            .file_type()
            .is_dir(),
        "missing source witness partition",
    )?;
    let mut copied = Vec::new();
    for entry in io(fs::read_dir(&source_partition))? {
        let entry = io(entry)?;
        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or(Error("malformed source section name"))?;
        let section =
            u64::from_str_radix(name, 16).map_err(|_| Error("malformed source section name"))?;
        require(
            name == format!("{section:016x}") && old.sections.contains(&section),
            "source section outside scope",
        )?;
        let raw = bounded_read(&entry.path(), (HEADER + MAX_BLOB_BYTES) as u64)?;
        copied.push((section, raw));
    }
    copied.sort_by_key(|(section, _)| *section);

    io(fs::create_dir(&target_path))?;
    let audit = target_path.join("source-snapshot");
    io(fs::create_dir(&audit))?;
    let copied_ledger = audit.join("witness");
    copy_regular(paths.source_ledger, &copied_ledger)?;
    let head_bytes = bounded_read(&side(paths.source_ledger, ".head"), 88)?;
    save(&audit.join("witness.head"), &head_bytes)?;
    let mut hashes = Vec::new();
    for (section, raw) in &copied {
        save(&audit.join(format!("{section:016x}")), raw)?;
        hashes.push((*section, hash_parts(b"I1b/source-raw/v1", &[raw])));
    }
    io(io(File::open(&audit))?.sync_all())?;
    io(io(File::open(&target_path))?.sync_all())?;
    io(io(File::open(&target_parent))?.sync_all())?;

    let old_ledger = snapshot(&copied_ledger, &source, expected)?;
    let mut frames = Vec::new();
    for (section, raw) in &copied {
        let data = native_data(raw)?;
        frames.extend(source.decode_section(*section, data)?);
    }
    for protected in old_ledger.index.witnesses.values() {
        require(
            frames.iter().any(|frame| {
                frame.section == protected.frame.section
                    && frame.logical_offset == protected.frame.logical_offset
                    && frame.frame_bytes == protected.frame.frame_bytes
                    && frame.wire == protected.frame.wire
            }),
            "source lost committed artifact bytes",
        )?;
    }
    let mut recovered_payloads = old_ledger.index.eligible;
    let images = copied
        .into_iter()
        .map(|(section, raw)| (section, raw[HEADER..].to_vec()))
        .collect::<Vec<_>>();
    let link = RecoveryLink {
        old_generation: old.generation,
        old_head_hash: old_ledger.head_hash,
        source_files: hashes,
    };
    let storage = target_path.join("native");
    let ledger = target_path.join("witness");
    copy_regular(&copied_ledger, &ledger)?;
    save(&side(&ledger, ".head"), &head_bytes)?;
    io(io(File::open(&target_path))?.sync_all())?;
    let partition = old.partition.clone();
    let native_cfg = Config::default()
        .with_storage_directory(storage)
        .with_storage_blob_layouts(BlobLayout::V1..=BlobLayout::V1);
    Runner::new(native_cfg).start(|native| async move {
        let mut ledger = WitnessLedger::open_anchored(
            &ledger,
            source,
            crate::LedgerTail::RejectComplete,
            expected,
        )?;
        ledger.begin_recovery(target.clone(), link.clone())?;
        let mut runtime = WitnessRuntime::new(target, ledger);
        for (ordinal, (section, bytes)) in images.into_iter().enumerate() {
            let cfg = &runtime.policy.seal().config;
            let id = (ordinal as u64) * 3 + 1;
            // Offline coordinates are not grants, clock reads or protocol work.
            let identity = IoIdentity {
                io_id: id,
                grant: 0,
                tick_us: 0,
                run_id: cfg.run_id,
                node: cfg.worker,
                incarnation: cfg.generation,
                cid: cfg.cid,
                task: 0,
            };
            let (native_blob, size) = native
                .open(&partition, &section.to_be_bytes())
                .await
                .map_err(|_| Error("cannot open fresh native recovery blob"))?;
            require(size == 0, "recovery target blob already populated")?;
            let native_blob = std::sync::Arc::new(native_blob);
            let target = IoTarget {
                partition: partition.clone(),
                name: Some(section.to_be_bytes().to_vec()),
                open_id: Some(id),
            };
            let open = WitnessIo {
                target: target.clone(),
                operation: IoOperation::Open {
                    min_version: 0,
                    max_version: 0,
                },
                submitted: None,
                native: Some(native_blob.clone()),
            };
            let (updated, result) = runtime
                .observe(
                    identity.clone(),
                    open,
                    IoResult::Open {
                        len: 0,
                        version: BlobVersion::new(0).get(),
                    },
                    None,
                )
                .await;
            runtime = updated;
            result?;
            let submitted: IoBufs = bytes.clone().into();
            native_blob
                .write_at(0, submitted.clone(), WriteOptions::default())
                .await
                .map_err(|_| Error("cannot copy native recovery image"))?;
            let write = WitnessIo {
                target: target.clone(),
                operation: IoOperation::Write {
                    offset: 0,
                    len: bytes.len(),
                    digest: <sha2::Sha256 as sha2::Digest>::digest(&bytes).into(),
                    sync: false,
                    dont_cache: false,
                },
                submitted: Some(submitted),
                native: None,
            };
            let write_id = IoIdentity {
                io_id: id + 1,
                ..identity.clone()
            };
            let (updated, result) = runtime.observe(write_id, write, IoResult::Ok, None).await;
            runtime = updated;
            result?;
            native_blob
                .sync()
                .await
                .map_err(|_| Error("cannot sync copied native recovery image"))?;
            let sync = WitnessIo {
                target,
                operation: IoOperation::Sync { detached: false },
                submitted: None,
                native: Some(native_blob),
            };
            let sync_id = IoIdentity {
                io_id: id + 2,
                ..identity
            };
            let (updated, result) = runtime.observe(sync_id, sync, IoResult::Ok, None).await;
            runtime = updated;
            result?;
        }
        for payload in runtime.ledger.index.eligible.values() {
            if let Some(old) = recovered_payloads.remove(&payload.id) {
                require(old.wire == payload.wire, "recovered wire bytes changed")?;
            }
        }
        require(
            recovered_payloads.is_empty(),
            "recovery missing old eligible payload",
        )?;
        runtime
            .ledger
            .append(vec![LedgerEntry::Recovery(link.clone())], None)?;
        Ok(RecoveryReport {
            new_head: runtime.ledger.head(),
            source: link,
            records: runtime.ledger.records().to_vec(),
        })
    })
}

/// Recover the explicit absent-partition case without manufacturing native
/// storage evidence. The source is read-only, even with an uncommitted tail.
pub(crate) fn recover_empty_witness_generation(
    paths: RecoveryPaths<'_>,
    source: WitnessPolicy,
    target: WitnessPolicy,
    empty: &crate::native_recovery::AuthorizedEmptyBoundary,
) -> Result<RecoveryReport> {
    let old = source.seal().config.clone();
    let mut required = source.seal().clone();
    required.config.generation = old
        .generation
        .checked_add(1)
        .ok_or(Error("witness generation overflow"))?;
    require(
        old.generation == empty.generation() && *target.seal() == required,
        "empty recovery changes witness policy or generation",
    )?;
    require(
        io(fs::symlink_metadata(paths.source_storage))?
            .file_type()
            .is_dir(),
        "missing original native storage root",
    )?;
    require(
        io(fs::symlink_metadata(paths.source_ledger))?
            .file_type()
            .is_file(),
        "missing original witness ledger",
    )?;
    let old_root = io(paths.source_storage.canonicalize())?;
    let source_file = io(paths.source_ledger.canonicalize())?;
    require(
        fs::symlink_metadata(old_root.join(&old.partition))
            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound),
        "source voter partition is not absent",
    )?;

    let old_snapshot = snapshot(&source_file, &source, empty.witness())?;
    let observed_head = WitnessHead {
        sequence: u64::try_from(old_snapshot.records.len())
            .map_err(|_| Error("empty witness history bound"))?,
        hash: old_snapshot.head_hash,
    };
    require(
        old_snapshot
            .records
            .iter()
            .flat_map(|record| &record.entries)
            .all(|entry| match entry {
                LedgerEntry::Policy(_) => true,
                LedgerEntry::GenerationStarted(link) | LedgerEntry::Recovery(link) => {
                    link.source_files.is_empty()
                }
                _ => false,
            }),
        "empty recovery would drop protected witness history",
    )?;

    let parent = paths
        .fresh_target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let target_parent = io(parent.canonicalize())?;
    let target_path = target_parent.join(
        paths
            .fresh_target
            .file_name()
            .ok_or(Error("invalid empty recovery target"))?,
    );
    require(
        !target_path.starts_with(&old_root)
            && !source_file.starts_with(&target_path)
            && fs::symlink_metadata(&target_path)
                .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound),
        "empty recovery target must be separate and absent",
    )?;

    io(fs::create_dir(&target_path))?;
    let audit = target_path.join("source-snapshot");
    io(fs::create_dir(&audit))?;
    let copied_ledger = audit.join("witness");
    copy_regular(&source_file, &copied_ledger)?;
    let head_bytes = bounded_read(&side(&source_file, ".head"), 88)?;
    save(&audit.join("witness.head"), &head_bytes)?;
    #[derive(serde::Serialize)]
    struct Absence<'a> {
        partition: &'a str,
        source_files: [String; 0],
        absent: bool,
        cut_hash: String,
    }
    let absence = serde_json::to_vec(&Absence {
        partition: &old.partition,
        source_files: [],
        absent: true,
        cut_hash: crate::native_recovery_ledger::hex(&empty.cut()),
    })
    .map_err(|_| Error("encode empty native source audit"))?;
    save(&audit.join("absence.json"), &absence)?;
    io(io(File::open(&audit))?.sync_all())?;

    let ledger_path = target_path.join("witness");
    copy_regular(&copied_ledger, &ledger_path)?;
    save(&side(&ledger_path, ".head"), &head_bytes)?;
    io(fs::create_dir(target_path.join("native")))?;
    io(io(File::open(&target_path))?.sync_all())?;
    io(io(File::open(&target_parent))?.sync_all())?;

    let mut ledger = WitnessLedger::open_anchored(
        &ledger_path,
        source,
        crate::LedgerTail::RejectComplete,
        observed_head,
    )?;
    require(
        ledger.head() == observed_head,
        "witness source changed after empty history validation",
    )?;
    let link = RecoveryLink {
        old_generation: old.generation,
        old_head_hash: observed_head.hash,
        source_files: vec![],
    };
    ledger.begin_recovery(target, link.clone())?;
    ledger.append(vec![LedgerEntry::Recovery(link.clone())], None)?;
    Ok(RecoveryReport {
        new_head: ledger.head(),
        source: link,
        records: ledger.records().to_vec(),
    })
}

pub(crate) fn validate_existing_layout(root: &Path, policy: &WitnessPolicy) -> Result<()> {
    let partition = root.join(&policy.seal().config.partition);
    match fs::symlink_metadata(&partition) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        entry => {
            require(
                io(entry)?.file_type().is_dir(),
                "selected journal partition is not a directory",
            )?;
            for entry in io(fs::read_dir(partition))? {
                let entry = io(entry)?;
                let name = entry.file_name();
                require(
                    policy
                        .seal()
                        .config
                        .sections
                        .iter()
                        .any(|section| name == format!("{section:016x}").as_str()),
                    "unknown selected journal file",
                )?;
                let raw = bounded_read(&entry.path(), (HEADER + MAX_BLOB_BYTES) as u64)?;
                native_data(&raw)?;
            }
            Ok(())
        }
    }
}

pub(crate) fn v1_raw_hash(data: &[u8]) -> crate::Id {
    let mut raw = vec![0; HEADER];
    raw[..8].copy_from_slice(b"CWIK\0\x01\0\0");
    let crc = Crc32::checksum(&raw[..8]);
    raw[8..12].copy_from_slice(&crc.to_be_bytes());
    raw.extend_from_slice(data);
    hash_parts(b"I1b/source-raw/v1", &[&raw])
}

pub fn native_data(raw: &[u8]) -> Result<&[u8]> {
    require(
        (HEADER..=HEADER + MAX_BLOB_BYTES).contains(&raw.len())
            && &raw[..8] == b"CWIK\0\x01\0\0"
            && raw[8..12] == Crc32::checksum(&raw[..8]).to_be_bytes()
            && raw[12..HEADER].iter().all(|byte| *byte == 0),
        "invalid native V1 witness container",
    )?;
    Ok(&raw[HEADER..])
}

fn copy_regular(source: &Path, target: &Path) -> Result<()> {
    require(
        io(fs::symlink_metadata(source))?.file_type().is_file(),
        "expected regular source ledger",
    )?;
    let mut input = io(File::open(source))?;
    let mut output = io(OpenOptions::new().create_new(true).write(true).open(target))?;
    io(std::io::copy(&mut input, &mut output))?;
    io(output.sync_all())
}

fn save(path: &PathBuf, bytes: &[u8]) -> Result<()> {
    let mut file = io(OpenOptions::new().create_new(true).write(true).open(path))?;
    io(file.write_all(bytes))?;
    io(file.sync_all())
}
