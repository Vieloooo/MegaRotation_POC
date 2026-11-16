use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::SystemTime,
};

use base64::{Engine, engine::general_purpose::STANDARD};
use espresso_compatibility::{
    CaptureBundle, CaptureKind, CaptureRecord, Error, FIXTURE_CODEC, FinalityBasis,
    OfficialCaptureReader, PublicTimeUs, ReceiveOutcome, SourceBinding, SourceIdentity, sha256_hex,
};
use serde_json::{Value, json};

const VECTORS: &str = include_str!("../../../verification/espresso-compatibility/vectors-v1.json");

fn vectors() -> Value {
    serde_json::from_str(VECTORS).unwrap()
}

fn binding(vectors: &Value) -> SourceBinding {
    SourceBinding::new(
        serde_json::from_value(vectors["source_identity"].clone()).unwrap(),
        "0".repeat(64),
        CaptureKind::Fixture,
    )
    .unwrap()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

struct Driver {
    reader: OfficialCaptureReader,
    id: u64,
}

impl Driver {
    fn new(vectors: &Value) -> Self {
        Self {
            reader: OfficialCaptureReader::new(binding(vectors), "first").unwrap(),
            id: 0,
        }
    }

    fn feed(&mut self, path: &str, body: &Value) -> Result<ReceiveOutcome, Error> {
        let bytes = serde_json::to_vec(body).unwrap();
        self.raw("GET", path, Some(200), &bytes)
    }

    fn raw(
        &mut self,
        method: &str,
        path: &str,
        status: Option<u16>,
        response: &[u8],
    ) -> Result<ReceiveOutcome, Error> {
        self.id += 1;
        let source_id = self.reader.binding().source_id().to_owned();
        self.reader.receive(CaptureRecord {
            source_id: &source_id,
            reader_id: "first",
            id: self.id,
            method,
            path,
            status,
            response,
        })
    }

    fn block(&mut self, block: &Value) {
        let q = block["q"].as_u64().unwrap();
        self.feed(
            &format!("/v1/availability/block/{q}"),
            &block["block_response"],
        )
        .unwrap();
        self.feed(
            &format!("/v1/availability/leaf/{q}"),
            &block["leaf_response"],
        )
        .unwrap();
        for (i, tx) in block["transaction_responses"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            self.feed(&format!("/v1/availability/transaction/{q}/{i}/noproof"), tx)
                .unwrap();
        }
    }
}

fn assert_audit(reader: &OfficialCaptureReader, vectors: &Value, n: usize) {
    let expected = &vectors["expected_audit_prefixes"][n];
    assert_eq!(
        reader.binding().source_id(),
        vectors["expected_source_id"].as_str().unwrap(),
    );
    let bytes = reader.public_audit_bytes(FIXTURE_CODEC).unwrap();
    assert_eq!(hex(&bytes), expected["bytes_hex"].as_str().unwrap());
    assert_eq!(
        reader.public_sha256(FIXTURE_CODEC).unwrap(),
        expected["sha256"]
    );
}

#[test]
fn independent_vectors_keep_gaps_and_duplicate_publications() {
    let vectors = vectors();
    let b = &vectors["blocks"];
    let mut driver = Driver::new(&vectors);
    assert_audit(&driver.reader, &vectors, 0);

    assert!(matches!(
        driver.raw("POST", "/v1/submit/submit", Some(200), br#""TX~ack""#),
        Ok(ReceiveOutcome::Submitted { .. })
    ));
    assert_eq!(driver.reader.tip(), None);
    assert!(driver.reader.find_publication("TX~ack", b"A").is_empty());

    driver.block(&b[2]);
    driver.block(&b[0]);
    assert_audit(&driver.reader, &vectors, 1);
    assert!(matches!(
        driver.reader.read(2),
        Err(Error::Wait { next_q: 1, .. })
    ));
    assert!(matches!(
        driver.reader.require_watermark(PublicTimeUs(7_000_000)),
        Err(Error::Wait { .. })
    ));
    assert!(matches!(
        driver.raw("GET", "/v1/availability/block/1", Some(404), b"unavailable"),
        Ok(ReceiveOutcome::Unavailable)
    ));

    driver
        .feed("/v1/availability/block/1", &b[1]["block_response"])
        .unwrap();
    driver
        .feed("/v1/availability/leaf/1", &b[1]["leaf_response"])
        .unwrap();
    for i in [3, 0, 2] {
        driver
            .feed(
                &format!("/v1/availability/transaction/1/{i}/noproof"),
                &b[1]["transaction_responses"][i],
            )
            .unwrap();
    }
    assert!(matches!(
        driver.reader.read(1), Err(Error::Wait { next_q: 1, missing })
            if missing.transaction_indices == [1]
    ));
    let mut wrong = b[1]["transaction_responses"][1].clone();
    wrong["transaction"]["namespace"] = json!(9);
    assert!(matches!(
        driver.feed("/v1/availability/transaction/1/1/noproof", &wrong),
        Err(Error::InvalidSource(_))
    ));
    assert_audit(&driver.reader, &vectors, 1);
    driver
        .feed(
            "/v1/availability/transaction/1/1/noproof",
            &b[1]["transaction_responses"][1],
        )
        .unwrap();
    assert_audit(&driver.reader, &vectors, 3);

    let block = driver.reader.read(1).unwrap();
    assert!(matches!(
        block.finality_basis,
        FinalityBasis::Fixture { .. }
    ));
    let selected = driver
        .reader
        .project_publication_messages(1, FIXTURE_CODEC)
        .unwrap();
    assert_eq!(
        selected
            .iter()
            .map(|item| (
                item.position.message_index,
                item.source_position.source_transaction_index,
                item.message.bytes.as_slice()
            ))
            .collect::<Vec<_>>(),
        [
            (0, 1, b"A".as_slice()),
            (1, 2, b"A".as_slice()),
            (2, 3, b"".as_slice())
        ],
    );
    assert!(
        driver
            .reader
            .project_publication_messages(2, FIXTURE_CODEC)
            .unwrap()
            .is_empty()
    );

    let duplicate = serde_json::to_vec_pretty(&b[1]["block_response"]).unwrap();
    assert_eq!(
        driver.raw("GET", "/v1/availability/block/1", Some(200), &duplicate),
        Ok(ReceiveOutcome::Duplicate),
    );
    let mut altered = b[1]["block_response"].clone();
    altered["header"]["fields"]["fixture_annotation"] = json!("different public object");
    assert!(matches!(
        driver.feed("/v1/availability/block/1", &altered),
        Err(Error::InvalidSource(_))
    ));
    assert_audit(&driver.reader, &vectors, 3);
    driver.block(&b[3]);
    assert_audit(&driver.reader, &vectors, 4);
    let missing = driver.reader.export_pe_log().unwrap_err().missing;
    assert_eq!(missing, ["proposed_at", "finalized_at", "live_at_proposal"]);
    assert_eq!(
        driver.reader.export_pe_reader().err().unwrap().missing,
        missing
    );
}

#[test]
fn invalid_timestamps_and_malformed_bytes_are_atomic() {
    let vectors = vectors();
    let b = &vectors["blocks"];
    let mut driver = Driver::new(&vectors);
    driver.block(&b[0]);
    driver.block(&b[1]);
    let mut changed = b[2]["block_response"].clone();
    changed["header"]["fields"]["timestamp"] = json!(6);
    assert!(matches!(
        driver.feed("/v1/availability/block/2", &changed),
        Err(Error::InvalidSource(_))
    ));
    changed["header"]["fields"]["timestamp"] = json!(18_446_744_073_710u64);
    assert!(matches!(
        driver.feed("/v1/availability/block/2", &changed),
        Err(Error::InvalidSource(_))
    ));
    changed["header"]["fields"]["timestamp"] = json!(7);
    changed["payload"]["raw_payload"] = json!("bad raw payload");
    assert!(matches!(
        driver.feed("/v1/availability/block/2", &changed),
        Err(Error::InvalidSource(_))
    ));
    assert!(matches!(
        driver.raw(
            "GET",
            "/v1/node/block-height",
            Some(200),
            br#"{"duplicate":1,"duplicate":2}"#
        ),
        Err(Error::InvalidSource(_))
    ));
    assert_audit(&driver.reader, &vectors, 2);
    driver.block(&b[2]);
    assert_audit(&driver.reader, &vectors, 3);
    let mut q3 = b[3].clone();
    q3["block_response"]["header"]["fields"]["timestamp"] = json!(18_446_744_073_709u64);
    q3["leaf_response"]["leaf"]["block_header"] = q3["block_response"]["header"].clone();
    driver.block(&q3);
    assert_eq!(
        driver.reader.tip(),
        Some((3, PublicTimeUs(18_446_744_073_709_000_000)))
    );
}

#[test]
fn native_payload_and_selected_envelope_capacity_boundaries() {
    let fixture = vectors();
    let mut driver = Driver::new(&fixture);
    let mut tx = fixture["blocks"][1]["transaction_responses"][0].clone();
    tx["transaction"]["payload"] = json!(STANDARD.encode(vec![0xdd; 65_537]));
    assert!(matches!(
        driver.feed("/v1/availability/transaction/1/0/noproof", &tx),
        Err(Error::Capacity(_))
    ));
    tx["transaction"]["payload"] = json!(STANDARD.encode(vec![0xdd; 65_536]));
    assert_eq!(
        driver.feed("/v1/availability/transaction/1/0/noproof", &tx),
        Ok(ReceiveOutcome::Stored),
    );
    tx["namespace"] = json!(160_016);
    tx["transaction"]["namespace"] = json!(160_016);
    for i in 1..=16 {
        tx["index"] = json!(i);
        let path = format!("/v1/availability/transaction/1/{i}/noproof");
        if i == 16 {
            tx["transaction"]["payload"] = json!(STANDARD.encode(vec![0xcc; 65_293]));
            assert!(matches!(driver.feed(&path, &tx), Err(Error::Capacity(_))));
            tx["transaction"]["payload"] = json!(STANDARD.encode(vec![0xcc; 65_292]));
        }
        assert_eq!(driver.feed(&path, &tx), Ok(ReceiveOutcome::Stored));
    }
    assert_eq!(driver.reader.tip(), None);
}

struct Temp(PathBuf);

impl Temp {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let time = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_nanos();
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("ec16-test-{}-{time}-{count}", std::process::id(),));
        fs::create_dir(&dir).unwrap();
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn file_ref(dir: &Temp, path: &str, contents: &[u8]) -> Value {
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.path().join(path))
        .unwrap();
    file.write_all(contents).unwrap();
    json!({"path": path, "sha256": sha256_hex(contents)})
}

fn write_manifest(dir: &Temp, manifest: &Value) {
    fs::write(
        dir.path().join("manifest.json"),
        serde_json::to_vec(manifest).unwrap(),
    )
    .unwrap();
}

#[test]
fn bundle_seals_raw_evidence_and_never_shares_reader_progress() {
    let fixture = vectors();
    let dir = Temp::new();
    let b0 = &fixture["blocks"][0];
    let source_id = &fixture["expected_source_id"];
    let block = file_ref(
        &dir,
        "block0",
        &serde_json::to_vec(&b0["block_response"]).unwrap(),
    );
    let leaf = file_ref(
        &dir,
        "leaf0",
        &serde_json::to_vec(&b0["leaf_response"]).unwrap(),
    );
    let mut manifest = json!({
        "schema": "EC16-CAPTURE-1", "capture_kind": "fixture",
        "identity": fixture["source_identity"], "source_id": source_id, "configuration": {},
        "artifacts": [],
        "records": [
            {"id": 0, "reader_id": "first", "source_id": source_id, "method": "GET",
             "path": "/v1/availability/block/0", "status": 200, "request": null,
             "response": block, "observation": {"wall_ns": 1}},
            {"id": 1, "reader_id": "second", "source_id": source_id, "method": "GET",
             "path": "/v1/availability/block/0", "status": 200, "request": null,
             "response": block, "observation": {"wall_ns": 20}},
            {"id": 2, "reader_id": "first", "source_id": source_id, "method": "GET",
             "path": "/v1/availability/leaf/0", "status": 200, "request": null,
             "response": leaf, "observation": {"wall_ns": 30}},
        ]
    });
    write_manifest(&dir, &manifest);
    let bundle = CaptureBundle::open(dir.path()).unwrap();
    assert_audit(&bundle.replay("first").unwrap().reader, &fixture, 1);
    assert_audit(&bundle.replay("second").unwrap().reader, &fixture, 0);

    manifest["records"][2]["observation"]["wall_ns"] = json!(5000);
    write_manifest(&dir, &manifest);
    let bundle = CaptureBundle::open(dir.path()).unwrap();
    assert_audit(&bundle.replay("first").unwrap().reader, &fixture, 1);

    let outside = Temp::new();
    let external = outside.path().join("leaf0");
    fs::copy(dir.path().join("leaf0"), &external).unwrap();
    fs::remove_file(dir.path().join("leaf0")).unwrap();
    assert!(matches!(
        CaptureBundle::open(dir.path()),
        Err(Error::InvalidRecording(_))
    ));
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&external, dir.path().join("leaf0")).unwrap();
        assert!(matches!(
            CaptureBundle::open(dir.path()),
            Err(Error::InvalidRecording(_))
        ));
        fs::remove_file(dir.path().join("leaf0")).unwrap();
    }
    fs::write(dir.path().join("leaf0"), b"altered").unwrap();
    assert!(matches!(
        CaptureBundle::open(dir.path()),
        Err(Error::InvalidRecording(_))
    ));
}

#[test]
fn pinned_source_identity_cannot_mix_runs() {
    let fixture = vectors();
    let identity = SourceIdentity::pinned("other-run", "BLOCK~fixture-genesis");
    let binding = SourceBinding::new(identity, "0".repeat(64), CaptureKind::Fixture).unwrap();
    let mut reader = OfficialCaptureReader::new(binding, "first").unwrap();
    assert!(matches!(
        reader.receive(CaptureRecord {
            source_id: fixture["expected_source_id"].as_str().unwrap(),
            reader_id: "first",
            id: 0,
            method: "GET",
            path: "/v1/node/block-height",
            status: Some(200),
            response: b"9",
        }),
        Err(Error::InvalidSource(_))
    ));
    assert_eq!(reader.tip(), None);
}

#[test]
fn retained_native_bytes_and_count_include_pending_foreign_transactions() {
    let fixture = vectors();
    let mut driver = Driver::new(&fixture);
    let mut block = fixture["blocks"][1]["block_response"].clone();
    block["num_transactions"] = json!(1025);
    assert_eq!(
        driver.feed("/v1/availability/block/1", &block),
        Err(Error::Capacity("transactions in one block")),
    );
    block["num_transactions"] = json!(1024);
    assert_eq!(
        driver.feed("/v1/availability/block/1", &block),
        Ok(ReceiveOutcome::Stored),
    );

    let mut tx = fixture["blocks"][1]["transaction_responses"][0].clone();
    tx["transaction"]["payload"] = json!(STANDARD.encode(vec![0xdd; 65_536]));
    for i in 0..1024 {
        tx["index"] = json!(i);
        tx["pos_in_namespace"] = json!(i);
        assert_eq!(
            driver.feed(&format!("/v1/availability/transaction/1/{i}/noproof"), &tx),
            Ok(ReceiveOutcome::Stored),
        );
    }
    assert_eq!(
        driver.feed("/v1/availability/transaction/1/1023/noproof", &tx),
        Ok(ReceiveOutcome::Duplicate),
    );
    tx["block_height"] = json!(2);
    tx["index"] = json!(0);
    tx["pos_in_namespace"] = json!(0);
    tx["transaction"]["payload"] = json!("AA==");
    assert_eq!(
        driver.feed("/v1/availability/transaction/2/0/noproof", &tx),
        Err(Error::Capacity("retained native transaction payload bytes")),
    );
    assert_audit(&driver.reader, &fixture, 0);
}

#[test]
fn distinct_height_capacity_counts_pending_slots() {
    let fixture = vectors();
    let mut driver = Driver::new(&fixture);
    let mut leaf = fixture["blocks"][0]["leaf_response"].clone();
    for q in 1..=4097 {
        leaf["leaf"]["block_header"]["fields"]["height"] = json!(q);
        leaf["qc"]["data"]["block_number"] = json!(q);
        let delivery = driver.feed(&format!("/v1/availability/leaf/{q}"), &leaf);
        if q <= 4096 {
            assert_eq!(delivery, Ok(ReceiveOutcome::Stored));
        } else {
            assert_eq!(
                delivery,
                Err(Error::Capacity("distinct block heights including pending"))
            );
        }
    }
    assert!(matches!(
        driver.reader.read(1),
        Err(Error::Wait { next_q: 0, .. })
    ));
}

fn fixture_manifest(fixture: &Value) -> Value {
    json!({
        "schema": "EC16-CAPTURE-1", "capture_kind": "fixture",
        "identity": fixture["source_identity"], "source_id": fixture["expected_source_id"],
        "configuration": {}, "artifacts": [], "records": [],
    })
}

fn sparse_file(dir: &Temp, name: &str, size: usize) {
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.path().join(name))
        .unwrap();
    file.set_len(size.try_into().unwrap()).unwrap();
}

#[test]
fn recording_limits_apply_to_manifest_each_raw_file_and_unique_file_sum() {
    let fixture = vectors();
    let dir = Temp::new();
    let per_file = 16 * 1024 * 1024;
    let total = 256 * 1024 * 1024;
    let mut manifest = fixture_manifest(&fixture);

    let mut padded = serde_json::to_vec(&manifest).unwrap();
    padded.resize(per_file, b' ');
    fs::write(dir.path().join("manifest.json"), &padded).unwrap();
    assert!(CaptureBundle::open(dir.path()).is_ok());
    padded.push(b' ');
    fs::write(dir.path().join("manifest.json"), &padded).unwrap();
    assert!(matches!(
        CaptureBundle::open(dir.path()),
        Err(Error::Capacity("manifest"))
    ));
    drop(padded);

    let zeros = vec![0; per_file];
    let sha256 = sha256_hex(&zeros);
    let files = manifest["artifacts"].as_array_mut().unwrap();
    for i in 0..15 {
        let name = format!("{i:02}.raw");
        sparse_file(&dir, &name, per_file);
        files.push(json!({"path": name, "sha256": sha256}));
    }
    // A second reference to the same path must not contribute to total bytes.
    files.push(files[0].clone());
    files.push(json!({"path": "last.raw", "sha256": "0".repeat(64)}));
    let manifest_bytes = serde_json::to_vec(&manifest).unwrap().len();
    let last_size = total - manifest_bytes - 15 * per_file;
    let last_sha256 = sha256_hex(&zeros[..last_size]);
    manifest["artifacts"][16]["sha256"] = json!(last_sha256);
    sparse_file(&dir, "last.raw", last_size);
    write_manifest(&dir, &manifest);

    sparse_file(&dir, "00.raw", per_file + 1);
    assert!(matches!(
        CaptureBundle::open(dir.path()),
        Err(Error::Capacity("raw file"))
    ));
    sparse_file(&dir, "00.raw", per_file);
    assert!(CaptureBundle::open(dir.path()).is_ok());

    sparse_file(&dir, "last.raw", last_size + 1);
    assert!(matches!(
        CaptureBundle::open(dir.path()),
        Err(Error::Capacity("recording bundle"))
    ));
}
