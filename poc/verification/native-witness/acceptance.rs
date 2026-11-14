//! Independent NATIVE-WITNESS W1–W12 acceptance, derived before candidate inspection.
//! Offline fixtures do not exercise a live consensus engine or transport.
use commonware_codec::{Copying, DecodeExt as _, Encode as _};
use commonware_consensus::{
    simplex::{
        scheme::bls12381_multisig::Scheme,
        types::{Artifact, Certificate, Finalize, Notarization, Notarize, Proposal, Vote},
    },
    types::{Epoch, Round, View},
};
use commonware_cryptography::{
    Signer as _,
    bls12381::primitives::{ops, variant::MinSig},
    certificate::Verifier as _,
    ed25519::{PrivateKey, PublicKey},
    sha256::Digest,
};
use commonware_parallel::Sequential;
use commonware_runtime::{
    Blob, ReadOptions, Storage, Supervisor as _, WriteOptions,
    buffer::paged::{self, CacheRef},
    tokio::Config,
};
use commonware_storage::journal::segmented::variable::{Config as JournalConfig, Journal};
use commonware_utils::{NZUsize, iter::NonEmpty, ordered::BiMap};
use poc_native_runtime::*;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use serde_json::json;
use sha2::{Digest as _, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

const RUN: [u8; 32] = [0x11; 32];
const CONFIG: [u8; 32] = [0x22; 32];
const CID: [u8; 32] = [0x33; 32];
const PART: &str = "i1b-voter-0";
const NAME: [u8; 8] = 1u64.to_be_bytes();
const T: u64 = 1_000_000;
type S = Scheme<PublicKey, MinSig>;
type A = Artifact<S, Digest>;
type Id = [u8; 32];
static ROOT: OnceLock<PathBuf> = OnceLock::new();
static NEXT: AtomicU64 = AtomicU64::new(0);
fn place(label: &str) -> PathBuf {
    let root = ROOT.get_or_init(|| {
        if let Some(root) = std::env::var_os("NATIVE_WITNESS_VERIFY_ARTIFACTS") {
            let root = PathBuf::from(root);
            fs::create_dir_all(&root).unwrap();
            return root;
        }
        for suffix in 0..u64::MAX {
            let p = std::env::temp_dir().join(format!(
                "native-witness-verifier-{}-{suffix}",
                std::process::id()
            ));
            match fs::create_dir(&p) {
                Ok(()) => return p,
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => panic!("reserve {}: {e}", p.display()),
            }
        }
        panic!("temporary suffix exhausted")
    });
    let p = root.join(format!("{label}-{}", NEXT.fetch_add(1, Ordering::SeqCst)));
    fs::create_dir(&p).unwrap();
    fs::create_dir(p.join("store")).unwrap();
    p
}
fn h(domain: &str, parts: &[&[u8]]) -> Id {
    let mut s = Sha256::new();
    for p in std::iter::once(domain.as_bytes()).chain(parts.iter().copied()) {
        s.update((p.len() as u64).to_be_bytes());
        s.update(p);
    }
    s.finalize().into()
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn save(p: &Path, v: &impl serde::Serialize) {
    fs::write(p, serde_json::to_vec_pretty(v).unwrap()).unwrap();
}
fn crc(b: &[u8]) -> u32 {
    let mut x = !0u32;
    for b in b {
        x ^= u32::from(*b);
        for _ in 0..8 {
            x = (x >> 1) ^ if x & 1 != 0 { 0x82f63b78 } else { 0 };
        }
    }
    !x
}
fn frame(a: &A) -> Vec<u8> {
    let body = a.encode();
    let mut n = body.len() as u32;
    let mut out = Vec::new();
    while n >= 128 {
        out.push((n as u8 & 127) | 128);
        n >>= 7;
    }
    out.push(n as u8);
    out.extend_from_slice(&body);
    out
}
fn pages(b: &[u8]) -> Vec<u8> {
    b.chunks(244)
        .flat_map(|c| {
            let mut p = vec![0; 256];
            p[..c.len()].copy_from_slice(c);
            slot(&mut p, 0, c.len());
            p
        })
        .collect()
}
fn slot(p: &mut [u8], i: usize, n: usize) {
    let sum = crc(&p[..n]);
    p[244 + i * 6..246 + i * 6].copy_from_slice(&(n as u16).to_be_bytes());
    p[246 + i * 6..250 + i * 6].copy_from_slice(&sum.to_be_bytes());
}
fn cert_bit(bit: bool) -> A {
    A::Certification(Round::new(Epoch::new(1), View::new(1)), bit)
}
fn cfg(generation: u64) -> WitnessConfig {
    WitnessConfig {
        run_id: RUN,
        config_hash: CONFIG,
        cid: CID,
        worker: 0,
        member: 0,
        epoch: 1,
        generation,
        partition: PART.into(),
        sections: vec![1, 2],
    }
}
fn profile(seed: u64) -> Profile {
    Profile {
        run_id: RUN,
        config_hash: CONFIG,
        run_seed: seed,
        node: 0,
        incarnation: 1,
        epsilon_us: 1000,
    }
}
struct Fixture {
    policy: WitnessPolicy,
    target: WitnessPolicy,
    members: Vec<WitnessMember>,
    signers: Vec<S>,
    verifier: S,
    artifacts: Vec<A>,
    wire: NativeWire,
    cert: NativeWire,
}
fn fixture() -> Fixture {
    let ns = h(
        "I1b/voter-namespace/v1",
        &[&RUN, &CONFIG, &CID, &1u64.to_be_bytes()],
    );
    let mut rng = ChaCha20Rng::from_seed([42; 32]);
    let mut members = Vec::new();
    let mut keys = Vec::new();
    let mut secrets = Vec::new();
    for i in 0..5u32 {
        let key = PrivateKey::decode(Copying(&[16 + i as u8; 32]))
            .unwrap()
            .public_key();
        let (secret, bls_key) = ops::keypair::<_, MinSig>(&mut rng);
        let pop = ops::sign_proof_of_possession::<MinSig>(&secret, b"I1b/storage/pop/v1");
        keys.push((key.clone(), bls_key.clone()));
        members.push(WitnessMember {
            member: i,
            key,
            bls_key,
            pop,
        });
        secrets.push(secret);
    }
    let registry = BiMap::try_from(keys).unwrap();
    let verifier = S::verifier(&ns, registry.clone());
    let signers: Vec<_> = secrets
        .into_iter()
        .map(|s| S::signer(&ns, registry.clone(), s).unwrap())
        .collect();
    let round = Round::new(Epoch::new(1), View::new(1));
    let proposal = Proposal::new(round, View::new(0), Digest([0x44; 32]));
    let votes: Vec<_> = signers
        .iter()
        .map(|s| Notarize::sign(s, proposal.clone()).unwrap())
        .collect();
    // A legitimate quorum certificate deliberately excludes the local member.
    let certificate = Notarization::from_notarizes(
        &verifier,
        NonEmpty::new(&votes[1], votes[2..5].iter()),
        &Sequential,
    )
    .unwrap();
    let finalize = Finalize::sign(&signers[0], proposal).unwrap();
    let wire = NativeWire {
        channel: 1,
        bytes: Vote::Finalize(finalize.clone()).encode().to_vec(),
    };
    let cert = NativeWire {
        channel: 2,
        bytes: Certificate::Notarization(certificate.clone())
            .encode()
            .to_vec(),
    };
    Fixture {
        policy: WitnessPolicy::new(cfg(1), &members).unwrap(),
        target: WitnessPolicy::new(cfg(2), &members).unwrap(),
        members,
        signers,
        verifier,
        artifacts: vec![
            A::Notarize(votes[0].clone()),
            A::Notarization(certificate),
            cert_bit(true),
            A::Finalize(finalize),
        ],
        wire,
        cert,
    }
}
fn calibrate(d: &mut ControlledDriver) {
    d.grant(g(1), Input::RequestSample, |_| Ok(())).unwrap();
    d.grant(
        g(2),
        Input::DeliverSample(ClockSample {
            run_id: RUN,
            node: 0,
            incarnation: 1,
            clock_domain: h("I1/clock-domain/v1", &[&RUN, &CONFIG, &0u32.to_be_bytes()]),
            rate_profile: h("I1/rate/v1", &[b"fixed-rate-one-v1"]),
            request_id: 1,
            sequence: 1,
            as_of_tick_us: T,
            node_time_at_as_of_us: T as i64 + 1000,
            uncertainty_us: 1000,
            lease_end_tick_us: T + 100_000,
        }),
        |_| Ok(()),
    )
    .unwrap();
}
fn g(id: u64) -> Grant {
    Grant { id, tick_us: T }
}
struct Run {
    ok: bool,
    events: Vec<Event>,
    records: Vec<LedgerRecord>,
    head: WitnessHead,
}
fn run(
    path: &Path,
    policy: WitnessPolicy,
    seed: u64,
    pause: u64,
    configure: impl FnOnce(&mut ControlledDriver),
    f: impl AsyncFnOnce(&mut ControlledDriver) -> Result<(), Error>,
) -> Run {
    let captured = Arc::new(Mutex::new(None));
    let output = captured.clone();
    let result = ControlledDriver::run_async(
        Config::default()
            .with_worker_threads(1)
            .with_storage_directory(path.join("store")),
        profile(seed),
        async |d| {
            d.set_io_wall_pause(Duration::from_millis(pause))?;
            d.configure_witness(policy, &path.join("witness"))?;
            configure(d);
            calibrate(d);
            let result = f(d).await;
            let cleanup = d.shutdown_async().await;
            *output.lock().unwrap() = Some((
                d.events(),
                d.witness_records().unwrap_or(&[]).to_vec(),
                d.witness_head().unwrap(),
            ));
            result.and(cleanup)
        },
    );
    let (events, records, head) = captured
        .lock()
        .unwrap()
        .take()
        .expect("controller assertion escaped before capture");
    let mut failed = false;
    for e in &events {
        match e {
            Event::Failure { .. } => failed = true,
            Event::Ack { .. } => assert!(!failed, "Ack after failure"),
            _ => {}
        }
    }
    save(&path.join("events.json"), &events);
    save(&path.join("records.json"), &records);
    save(
        &path.join("outcome.json"),
        &json!({"ok":result.is_ok(),"error":result.as_ref().err().map(ToString::to_string),"head":head}),
    );
    Run {
        ok: result.is_ok(),
        events,
        records,
        head,
    }
}
fn count(records: &[LedgerRecord], name: &str) -> usize {
    records
        .iter()
        .flat_map(|r| &r.entries)
        .filter(|e| {
            matches!(
                (name, e),
                ("barrier", LedgerEntry::Barrier(_))
                    | ("witness", LedgerEntry::Witness(_))
                    | ("eligible", LedgerEntry::Eligible(_))
                    | ("submission", LedgerEntry::Submission(_))
            )
        })
        .count()
}
fn make_run(path: &Path, fx: &Fixture, submit: bool) -> Run {
    let image = pages(&fx.artifacts.iter().flat_map(frame).collect::<Vec<_>>());
    let wire = fx.wire.clone();
    run(
        path,
        fx.policy.clone(),
        7,
        0,
        |_| {},
        async move |d| {
            d.grant_async(g(3), Input::None, |d| {
                d.spawn_session(CID, 2_000_000, move |c| async move {
                    let (b, n) = c.open(PART, &NAME).await.unwrap();
                    assert_eq!(n, 0);
                    b.write_at(0, image, WriteOptions::default()).await.unwrap();
                    b.sync().await.unwrap();
                })
            })
            .await?;
            if submit {
                d.grant_async(g(4), Input::None, |d| d.stage_witnessed_send(wire, &[2]))
                    .await?;
            }
            Ok(())
        },
    )
}
fn copy_ledger(source: &Path, dest: &Path) {
    fs::copy(source, dest).unwrap();
    fs::copy(source.with_extension("head"), dest.with_extension("head")).unwrap();
}
fn write_ledger(path: &Path, records: &mut [LedgerRecord]) -> WitnessHead {
    let mut bytes = Vec::new();
    let mut previous = [0; 32];
    for (i, r) in records.iter_mut().enumerate() {
        r.sequence = i as u64 + 1;
        r.previous_record_hash = previous;
        let body = serde_json::to_vec(r).unwrap();
        let n = body.len() as u64;
        previous = h("I1b/witness-record/v1", &[&body]);
        bytes.extend_from_slice(&n.to_be_bytes());
        bytes.extend_from_slice(&(!n).to_be_bytes());
        bytes.extend_from_slice(&body);
        bytes.extend_from_slice(&previous);
    }
    let mut head = b"I1WHEAD1".to_vec();
    head.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    head.extend_from_slice(&(records.len() as u64).to_be_bytes());
    head.extend_from_slice(&previous);
    head.extend_from_slice(&h(
        "I1b/witness-head/v1",
        &[&RUN, &CONFIG, &0u32.to_be_bytes(), &head],
    ));
    fs::write(path, bytes).unwrap();
    fs::write(path.with_extension("head"), head).unwrap();
    WitnessHead {
        sequence: records.len() as u64,
        hash: previous,
    }
}
fn barrier(blob_id: Id, io_id: u64, kind: u8, offset: u64, bytes: Vec<u8>) -> Barrier {
    let id = h(
        "I1b/native-barrier/v1",
        &[
            &blob_id,
            &io_id.to_be_bytes(),
            &[kind],
            &offset.to_be_bytes(),
            &(bytes.len() as u64).to_be_bytes(),
            &h("I1b/barrier-bytes/v1", &[&bytes]),
        ],
    );
    Barrier {
        blob_id,
        io_id,
        kind,
        offset,
        bytes,
        id,
    }
}
fn witness(blob_id: Id, frame: DecodedArtifact, mut refs: Vec<Id>) -> ArtifactWitness {
    refs.sort();
    refs.dedup();
    let artifact_hash = h("I1b/native-artifact/v1", &[&frame.artifact_bytes]);
    let joined: Vec<_> = refs.iter().flatten().copied().collect();
    let id = h(
        "I1b/artifact-witness/v1",
        &[
            &RUN,
            &CONFIG,
            &CID,
            &1u64.to_be_bytes(),
            &blob_id,
            &frame.section.to_be_bytes(),
            &frame.logical_offset.to_be_bytes(),
            &artifact_hash,
            &joined,
        ],
    );
    ArtifactWitness {
        id,
        blob_id,
        artifact_hash,
        barrier_ids: refs,
        frame,
    }
}

#[test]
fn w4_independent_crc_page_header_and_hash_literals() {
    assert_eq!(crc(b"123456789"), 0xe3069283);
    assert_eq!(crc(&[0; 32]), 0x8a9136aa);
    let f = frame(&cert_bit(true));
    assert_eq!(f, [4, 2, 1, 1, 1]);
    assert_eq!(crc(&f), 0x5ed8620e);
    let p = pages(&f);
    assert_eq!(
        hex(&Sha256::digest(&p)),
        "332439dad59808f66577c97ed328c2dd15796199cc4c015dc89fa4d0dbc6e1f3"
    );
    let mut header = vec![0; 4096];
    header[..8].copy_from_slice(b"CWIK\0\x01\0\0");
    let sum = crc(&header[..8]);
    header[8..12].copy_from_slice(&sum.to_be_bytes());
    assert_eq!(
        hex(&Sha256::digest(&header)),
        "bfdac390422af0a631e15b75f522c13119bb0c021cbecb892853f44bb4498494"
    );
    let mut raw = header;
    raw.extend_from_slice(&p);
    assert_eq!(native_data(&raw).unwrap(), p);
    assert_eq!(
        hex(&h("I1b/source-raw/v1", &[&raw])),
        "7ce88277c94661901e7c404470cf6dfd9f2d6187ea131ed067f6d10686cb4181"
    );
    let fx = fixture();
    assert_eq!(
        hex(&fx.policy.seal().namespace_hash),
        "20ba426b3ebbcaa269c1a058dced7633e6f223925fb02595e03c53bc0bee883f"
    );
    let blob = h(
        "I1b/blob/v1",
        &[
            &RUN,
            &CONFIG,
            &0u32.to_be_bytes(),
            &1u64.to_be_bytes(),
            PART.as_bytes(),
            &NAME,
            &7u64.to_be_bytes(),
        ],
    );
    assert_eq!(
        hex(&blob),
        "e6a26765a5767c5b0f4d7ea6a84a32c0185e260ba51f77975a8c6a1a793cfd0a"
    );
    let b1 = barrier(blob, 9, 1, 0, p.clone());
    let b2 = barrier(blob, 10, 2, 0, p.clone());
    assert_eq!(
        hex(&b1.id),
        "1faa8eaa0664154f7f960198207fac89efde925aac8960023078239fb56966ef"
    );
    assert_eq!(
        hex(&b2.id),
        "4a7209e05286866bae2fe9de3b1e9e71f072f03c6d48cdcbd6cc99c060a2a96a"
    );
    let decoded = fx.policy.decode_section(1, &p).unwrap().remove(0);
    let w = witness(blob, decoded, vec![b1.id, b2.id]);
    assert_eq!(
        hex(&w.id),
        "1c01943f109cb6dd886a52d03718076e49306df8740bc9a5811f2f04032e66c8"
    );
    let parts: Vec<Vec<u8>> = fx
        .members
        .iter()
        .flat_map(|m| {
            [
                m.member.to_be_bytes().to_vec(),
                m.key.encode().to_vec(),
                m.bls_key.encode().to_vec(),
                m.pop.encode().to_vec(),
            ]
        })
        .collect();
    assert_eq!(
        h(
            "I1b/voter-registry/v1",
            &parts.iter().map(Vec::as_slice).collect::<Vec<_>>()
        ),
        fx.policy.seal().registry_hash
    );
    let path = place("goldens");
    save(
        &path.join("vectors.json"),
        &json!({"registry":hex(&fx.policy.seal().registry_hash),"namespace":hex(&fx.policy.seal().namespace_hash),"blob":hex(&blob),"range_barrier":hex(&b1.id),"full_barrier":hex(&b2.id),"witness":hex(&w.id),"raw":raw}),
    );
}

#[test]
fn w6_w9_recovery_preserves_committed_authorization_history() {
    let path = place("recovery-auth");
    let fx = fixture();
    let run = make_run(&path, &fx, true);
    assert!(run.ok);
    assert_eq!(count(&run.records, "submission"), 1);
    let source = fs::read(path.join("witness")).unwrap();
    let raw = fs::read(path.join("store").join(PART).join("0000000000000001")).unwrap();
    let target = path.join("generation2");
    let report = recover_witness_generation(
        RecoveryPaths {
            source_storage: &path.join("store"),
            source_ledger: &path.join("witness"),
            fresh_target: &target,
        },
        fx.policy.clone(),
        fx.target.clone(),
        run.head,
    )
    .unwrap();
    assert_eq!(fs::read(path.join("witness")).unwrap(), source);
    assert_eq!(
        fs::read(path.join("store").join(PART).join("0000000000000001")).unwrap(),
        raw
    );
    save(&path.join("recovered-records.json"), &report.records);
    let mut ledger = WitnessLedger::open_anchored(
        &target.join("witness"),
        fx.target.clone(),
        LedgerTail::RejectComplete,
        report.new_head,
    )
    .unwrap();
    let before = ledger.head();
    let authorized = ledger.submit_recovered(&fx.wire, &[2]).unwrap();
    save(
        &path.join("duplicate-recovery.json"),
        &json!({"before":before,"after":ledger.head(),"authorization":authorized,"source_submissions":count(&run.records,"submission"),"target_submissions_before":count(&report.records,"submission")}),
    );
    assert_eq!(
        ledger.head(),
        before,
        "recovery duplicated a previously committed route authorization"
    );
}

#[test]
fn w2_w10_rehashed_stale_durable_version_is_rejected() {
    let path = place("stale-version");
    let fx = fixture();
    let a = pages(&frame(&cert_bit(true)));
    let b = pages(&frame(&cert_bit(false)));
    let aa = a.clone();
    let run = run(
        &path,
        fx.policy.clone(),
        7,
        0,
        |_| {},
        async move |d| {
            d.grant_async(g(3), Input::None, |d| {
                d.spawn_session(CID, 2_000_000, move |c| async move {
                    let (blob, _) = c.open(PART, &NAME).await.unwrap();
                    blob.write_at(0, aa.clone(), WriteOptions::default())
                        .await
                        .unwrap();
                    blob.write_at(0, aa[..5].to_vec(), WriteOptions::SYNC)
                        .await
                        .unwrap();
                    blob.write_at(0, b.clone(), WriteOptions::default())
                        .await
                        .unwrap();
                    blob.write_at(0, b[..5].to_vec(), WriteOptions::SYNC)
                        .await
                        .unwrap();
                })
            })
            .await?;
            Ok(())
        },
    );
    assert!(run.ok);
    assert_eq!(count(&run.records, "witness"), 0);
    assert_eq!(count(&run.records, "barrier"), 2);
    let mut records = run.records.clone();
    let bs: Vec<_> = records
        .iter()
        .flat_map(|r| &r.entries)
        .filter_map(|e| {
            if let LedgerEntry::Barrier(b) = e {
                Some(b.clone())
            } else {
                None
            }
        })
        .collect();
    let footer = barrier(bs[0].blob_id, 99, 1, 244, a[244..].to_vec());
    let w = witness(
        bs[0].blob_id,
        fx.policy.decode_section(1, &a).unwrap().remove(0),
        vec![bs[0].id, footer.id],
    );
    records.push(LedgerRecord {
        schema: 1,
        sequence: 0,
        previous_record_hash: [0; 32],
        identity: records[0].identity.clone(),
        entries: vec![LedgerEntry::Barrier(footer), LedgerEntry::Witness(w)],
    });
    let mutant = path.join("mutant");
    let head = write_ledger(&mutant, &mut records);
    save(&path.join("mutant-records.json"), &records);
    let outcome = WitnessLedger::open_anchored(
        &mutant,
        fx.policy.clone(),
        LedgerTail::RejectComplete,
        run.head,
    );
    save(
        &path.join("mutant-outcome.json"),
        &json!({"minimum_trusted_receipt":run.head,"mutant_head":head,"accepted":outcome.is_ok(),"error":outcome.as_ref().err().map(ToString::to_string)}),
    );
    assert!(
        outcome.is_err(),
        "ledger accepted stale version after a committed conflicting SYNC"
    );
}

#[test]
fn w4_decoder_slot_selection_bounds_and_canonical_frames() {
    let fx = fixture();
    let f = frame(&cert_bit(true));
    let mut p = pages(&[f.clone(), f.clone()].concat());
    slot(&mut p, 0, 5);
    slot(&mut p, 1, 10);
    assert_eq!(page_slot(&p).unwrap(), (1, 10));
    assert_eq!(fx.policy.decode_section(1, &p).unwrap().len(), 2);
    p[9] ^= 1;
    assert_eq!(page_slot(&p).unwrap(), (0, 5));
    assert_eq!(fx.policy.decode_section(1, &p).unwrap().len(), 1);
    slot(&mut p, 1, 5);
    assert_eq!(page_slot(&p).unwrap(), (0, 5));
    let invalid = vec![
        ("zero-slot", vec![0; 256]),
        ("partial-physical", p[..255].to_vec()),
        ("bad-crc", {
            let mut v = p.clone();
            v[0] ^= 1;
            v
        }),
        ("zero-frame", pages(&[0])),
        ("noncanonical-varint", pages(&[0x84, 0, 2, 1, 1, 1])),
        ("oversized-frame", pages(&[0x81, 0x20])),
        ("incomplete-frame", pages(&[4, 2, 1])),
        ("trailing-artifact-byte", pages(&[5, 2, 1, 1, 1, 0])),
        ("nonfinal-partial-page", [pages(&f), pages(&f)].concat()),
        ("blob-bound", vec![0; 65_792]),
        ("record-bound", pages(&f.repeat(1025))),
    ];
    for (label, bytes) in invalid {
        assert!(fx.policy.decode_section(1, &bytes).is_err(), "{label}");
    }
    let mut bad_len = pages(&f);
    bad_len[244..246].copy_from_slice(&245u16.to_be_bytes());
    assert!(page_slot(&bad_len).is_err());
    let many = pages(&f.repeat(100));
    let frames = fx.policy.decode_section(1, &many).unwrap();
    assert_eq!(frames.len(), 100);
    assert_eq!(frames[48].logical_offset, 240);
    assert_eq!(frames[48].pages.len(), 2);
    assert_eq!(frames[48].pages[0].physical_offset, 0);
    assert_eq!(frames[48].pages[1].physical_offset, 256);
}

#[test]
fn w5_context_pop_foreign_votes_and_typed_wire_mapping() {
    let fx = fixture();
    let image = pages(&fx.artifacts.iter().flat_map(frame).collect::<Vec<_>>());
    let decoded = fx.policy.decode_section(1, &image).unwrap();
    assert_eq!(decoded.len(), 4);
    assert_eq!(decoded[1].wire.as_ref(), Some(&fx.cert));
    assert_eq!(decoded[2].wire, None);
    assert_eq!(decoded[3].wire.as_ref(), Some(&fx.wire));
    assert_ne!(decoded[3].artifact_bytes[0], fx.wire.bytes[0]);
    let proposal = Proposal::new(
        Round::new(Epoch::new(1), View::new(1)),
        View::new(0),
        Digest([0x44; 32]),
    );
    let foreign = A::Notarize(Notarize::sign(&fx.signers[1], proposal).unwrap());
    assert!(
        fx.policy
            .decode_section(1, &pages(&frame(&foreign)))
            .is_err()
    );
    assert!(fx.policy.decode_section(2, &image).is_err());
    for field in 0..3 {
        let mut cfg = cfg(1);
        match field {
            0 => cfg.cid[0] ^= 1,
            1 => cfg.config_hash[0] ^= 1,
            _ => cfg.run_id[0] ^= 1,
        };
        let changed = WitnessPolicy::new(cfg, &fx.members).unwrap();
        assert!(changed.decode_section(1, &image).is_err());
    }
    let wrong_epoch = pages(&frame(&A::Certification(
        Round::new(Epoch::new(2), View::new(1)),
        true,
    )));
    assert!(fx.policy.decode_section(1, &wrong_epoch).is_err());
    let mut members = fixture().members;
    members[0].pop = members[1].pop.clone();
    assert!(WitnessPolicy::new(cfg(1), &members).is_err());
    let mut members = fixture().members;
    members[0].bls_key = members[1].bls_key.clone();
    members[0].pop = members[1].pop.clone();
    assert!(WitnessPolicy::new(cfg(1), &members).is_err());
    let mut members = fixture().members;
    members[0].key = members[1].key.clone();
    assert!(WitnessPolicy::new(cfg(1), &members).is_err());
    let mut members = fixture().members;
    members.reverse();
    assert_eq!(
        WitnessPolicy::new(cfg(1), &members).unwrap().seal(),
        fx.policy.seal()
    );
    let mut members = fixture().members;
    members[4].member = 3;
    assert!(WitnessPolicy::new(cfg(1), &members).is_err());
}

#[test]
fn w1_w2_exact_crc_prefix_footer_extent_and_empty_sync() {
    for scenario in ["extent", "prefix", "unrelated"] {
        let path = place(scenario);
        let fx = fixture();
        let f = frame(&cert_bit(true));
        let image = pages(&[f.clone(), f].concat());
        let expected = image.clone();
        let (release, wait) = tokio::sync::oneshot::channel();
        let r = run(
            &path,
            fx.policy.clone(),
            7,
            0,
            |_| {},
            async move |d| {
                d.grant_async(g(3), Input::None, |d| {
                    d.spawn_session(CID, 2_000_000, move |c| async move {
                        let (b, _) = c.open(PART, &NAME).await.unwrap();
                        b.write_at(0, image.clone(), WriteOptions::default())
                            .await
                            .unwrap();
                        let read = b.read_at(0, 256, ReadOptions::DONT_CACHE).await.unwrap();
                        assert_eq!(read.coalesce().as_ref(), expected);
                        match scenario {
                            "extent" => {
                                b.write_at(0, image[..250].to_vec(), WriteOptions::SYNC)
                                    .await
                                    .unwrap();
                                b.write_at(250, Vec::<u8>::new(), WriteOptions::SYNC)
                                    .await
                                    .unwrap();
                            }
                            "prefix" => {
                                b.write_at(5, image[5..].to_vec(), WriteOptions::SYNC)
                                    .await
                                    .unwrap();
                            }
                            _ => {
                                let (other, _) = c.open("unrelated", b"x").await.unwrap();
                                other.write_at(0, image, WriteOptions::SYNC).await.unwrap();
                                other.sync().await.unwrap();
                            }
                        }
                        wait.await.unwrap();
                        b.sync().await.unwrap();
                    })
                })
                .await?;
                assert_eq!(count(d.witness_records().unwrap(), "witness"), 0);
                d.grant_async(g(4), Input::None, |_| {
                    release.send(()).unwrap();
                    Ok(())
                })
                .await?;
                Ok(())
            },
        );
        assert!(r.ok);
        assert_eq!(count(&r.records, "witness"), 2);
        assert_eq!(count(&r.records, "eligible"), 0);
    }
}

#[test]
fn w3_w4_protected_reopen_mutations_and_live_incomplete_tail() {
    for operation in ["rewrite", "shrink", "remove", "fallback"] {
        let path = place(operation);
        let fx = fixture();
        let image = pages(&frame(&cert_bit(true)));
        let original = image.clone();
        let r = run(
            &path,
            fx.policy.clone(),
            7,
            0,
            |_| {},
            async move |d| {
                d.grant_async(g(3), Input::None, |d| {
                    d.spawn_session(CID, 2_000_000, move |c| async move {
                        let (b, _) = c.open(PART, &NAME).await.unwrap();
                        b.write_at(0, image, WriteOptions::SYNC).await.unwrap();
                        drop(b);
                        let (b, n) = c.open(PART, &NAME).await.unwrap();
                        assert_eq!(n, 256);
                        match operation {
                            "rewrite" => {
                                b.write_at(
                                    0,
                                    pages(&frame(&cert_bit(false))),
                                    WriteOptions::default(),
                                )
                                .await
                                .unwrap();
                            }
                            "shrink" => {
                                b.resize(255).await.unwrap();
                            }
                            "remove" => {
                                drop(b);
                                c.remove(PART, Some(&NAME)).await.unwrap();
                            }
                            _ => {
                                let mut corrupt = pages(&frame(&cert_bit(true)));
                                slot(&mut corrupt, 0, 4);
                                b.write_at(0, corrupt, WriteOptions::default())
                                    .await
                                    .unwrap();
                            }
                        }
                    })
                })
                .await?;
                Ok(())
            },
        );
        assert!(!r.ok, "protected {operation} must fail");
        assert_eq!(count(&r.records, "witness"), 1);
        assert!(
            !r.events
                .iter()
                .any(|e| matches!(e, Event::Ack { grant: 3 }))
        );
        let raw = fs::read(path.join("store").join(PART).join("0000000000000001")).unwrap();
        assert_eq!(native_data(&raw).unwrap(), original);
    }
    let path = place("incomplete-tail");
    let fx = fixture();
    let mut body = frame(&cert_bit(true));
    body.extend_from_slice(&[100, 0]);
    let image = pages(&body);
    assert!(fx.policy.decode_section(1, &image).is_err());
    let r = run(
        &path,
        fx.policy.clone(),
        7,
        0,
        |_| {},
        async move |d| {
            d.grant_async(g(3), Input::None, |d| {
                d.spawn_session(CID, 2_000_000, move |c| async move {
                    let (b, _) = c.open(PART, &NAME).await.unwrap();
                    b.write_at(0, image, WriteOptions::default()).await.unwrap();
                    b.sync().await.unwrap();
                })
            })
            .await?;
            Ok(())
        },
    );
    assert!(r.ok);
    assert_eq!(count(&r.records, "witness"), 1);
    assert_eq!(count(&r.records, "eligible"), 0);
    let source = fs::read(path.join("store").join(PART).join("0000000000000001")).unwrap();
    let result = recover_witness_generation(
        RecoveryPaths {
            source_storage: &path.join("store"),
            source_ledger: &path.join("witness"),
            fresh_target: &path.join("rejected-recovery"),
        },
        fx.policy,
        fx.target,
        r.head,
    );
    assert!(result.is_err());
    assert_eq!(
        source,
        fs::read(path.join("store").join(PART).join("0000000000000001")).unwrap()
    );
}

#[test]
fn w6_explicit_authorization_is_atomic_deduplicated_and_not_a_transport_attempt() {
    let path = place("authorization");
    let fx = fixture();
    let image = pages(&fx.artifacts.iter().flat_map(frame).collect::<Vec<_>>());
    let wire = fx.wire.clone();
    let cert = fx.cert.clone();
    let r = run(
        &path,
        fx.policy.clone(),
        7,
        0,
        |_| {},
        async move |d| {
            d.grant_async(g(3), Input::None, |d| {
                assert!(d.stage_witnessed_send(wire.clone(), &[1]).is_err());
                d.spawn_session(CID, 2_000_000, move |c| async move {
                    let (b, _) = c.open(PART, &NAME).await.unwrap();
                    b.write_at(0, image, WriteOptions::SYNC).await.unwrap();
                })
            })
            .await?;
            assert_eq!(count(d.witness_records().unwrap(), "submission"), 0);
            d.grant_async(g(4), Input::None, |d| {
                for destinations in [&[][..], &[0][..], &[2, 0][..], &[2, 5][..]] {
                    assert!(d.stage_witnessed_send(wire.clone(), destinations).is_err());
                }
                let mut wrong = wire.clone();
                wrong.channel = 2;
                assert!(d.stage_witnessed_send(wrong, &[2]).is_err());
                d.stage_witnessed_send(wire.clone(), &[3, 1, 3])?;
                d.stage_witnessed_send(cert.clone(), &[4])?;
                assert_eq!(count(d.witness_records().unwrap(), "submission"), 0);
                Ok(())
            })
            .await?;
            let head = d.witness_head().unwrap();
            assert_eq!(count(d.witness_records().unwrap(), "submission"), 3);
            d.grant_async(g(5), Input::None, |d| d.stage_witnessed_send(wire, &[1, 3]))
                .await?;
            assert_eq!(d.witness_head().unwrap(), head);
            Ok(())
        },
    );
    assert!(r.ok);
    assert_eq!(count(&r.records, "eligible"), 3);
    assert_eq!(count(&r.records, "submission"), 3);
    let committed = r
        .events
        .iter()
        .rposition(|e| matches!(e, Event::WitnessCommitted { io_id: 0, .. }))
        .unwrap();
    let submitted = r
        .events
        .iter()
        .position(|e| matches!(e, Event::WitnessSubmitted { .. }))
        .unwrap();
    assert!(committed < submitted);
    assert!(
        r.events
            .iter()
            .all(|e| !matches!(e, Event::WitnessSubmitted { .. }) || submitted > committed)
    );
    let mut forensic =
        WitnessLedger::open(&path.join("witness"), fx.policy, LedgerTail::RejectComplete).unwrap();
    assert!(forensic.submit_recovered(&fx.wire, &[2]).is_err());
}

#[test]
fn w7_durability_cut_points_and_explicit_single_batch_promotion() {
    for fault in [
        WitnessFault::BeforeLedger,
        WitnessFault::AfterRecordSync,
        WitnessFault::AfterHeadSync,
    ] {
        let path = place("cut");
        let fx = fixture();
        let image = pages(&frame(&cert_bit(true)));
        let reached = Arc::new(AtomicBool::new(false));
        let marker = reached.clone();
        let r = run(
            &path,
            fx.policy.clone(),
            7,
            0,
            |d| d.inject_witness_fault(3, fault).unwrap(),
            async move |d| {
                d.grant_async(g(3), Input::None, |d| {
                    d.spawn_session(CID, 2_000_000, move |c| async move {
                        let (b, _) = c.open(PART, &NAME).await.unwrap();
                        b.write_at(0, image, WriteOptions::default()).await.unwrap();
                        b.sync().await.unwrap();
                        marker.store(true, Ordering::SeqCst);
                    })
                })
                .await?;
                Ok(())
            },
        );
        assert!(!r.ok);
        assert!(!reached.load(Ordering::SeqCst));
        assert!(!r.events.iter().any(|e| matches!(
            e,
            Event::IoReleased { io_id: 3, .. }
                | Event::Ack { grant: 3 }
                | Event::WitnessSubmitted { .. }
        )));
        let copied = path.join("promote");
        copy_ledger(&path.join("witness"), &copied);
        let rejected = WitnessLedger::open_anchored(
            &copied,
            fx.policy.clone(),
            LedgerTail::RejectComplete,
            r.head,
        );
        if fault == WitnessFault::AfterRecordSync {
            assert!(rejected.is_err());
            let promoted = WitnessLedger::open_anchored(
                &copied,
                fx.policy.clone(),
                LedgerTail::PromoteVerified,
                r.head,
            )
            .unwrap();
            assert_eq!(promoted.head().sequence, r.head.sequence + 1);
            assert_eq!(count(promoted.records(), "witness"), 1);
        } else {
            let checked = rejected.unwrap();
            assert_eq!(checked.head(), r.head);
            assert_eq!(
                count(checked.records(), "witness"),
                usize::from(fault == WitnessFault::AfterHeadSync)
            );
        }
    }
}

#[test]
fn w8_cancelled_grant_retains_evidence_and_never_releases_consumer() {
    let path = place("cancel-evidence");
    let fx = fixture();
    let image = pages(&frame(&cert_bit(true)));
    let reached = Arc::new(AtomicBool::new(false));
    let marker = reached.clone();
    let hold = Arc::new(Mutex::new(None));
    let configured = hold.clone();
    let r = run(
        &path,
        fx.policy.clone(),
        7,
        0,
        |d| {
            *configured.lock().unwrap() = Some(d.hold_witness_before_persist(3).unwrap());
        },
        async move |d| {
            let (mut entered, release) = hold.lock().unwrap().take().unwrap();
            {
                let mut grant = Box::pin(d.grant_async(g(3), Input::None, |d| {
                    d.spawn_session(CID, 2_000_000, move |c| async move {
                        let (b, _) = c.open(PART, &NAME).await.unwrap();
                        b.write_at(0, image, WriteOptions::default()).await.unwrap();
                        b.sync().await.unwrap();
                        marker.store(true, Ordering::SeqCst);
                    })
                }));
                tokio::select! {r=&mut entered=>r.unwrap(),v=&mut grant=>panic!("grant escaped held evidence: {v:?}")};
                drop(grant);
            }
            assert_eq!(d.active_evidence_id(), Some(3));
            assert!(d.grant(g(4), Input::None, |_| Ok(())).is_err());
            assert!(d.grant(g(5), Input::None, |_| Ok(())).is_err());
            assert_eq!(d.active_evidence_id(), Some(3));
            release.send(()).unwrap();
            Ok(())
        },
    );
    assert!(!r.ok);
    assert!(!reached.load(Ordering::SeqCst));
    assert_eq!(count(&r.records, "witness"), 1);
    assert!(
        r.events
            .iter()
            .any(|e| matches!(e, Event::IoDiscarded { io_id: 3 }))
    );
    assert!(!r.events.iter().any(|e| matches!(
        e,
        Event::IoReleased { io_id: 3, .. }
            | Event::Ack { grant: 3 }
            | Event::WitnessSubmitted { .. }
    )));
}

fn journal_config(s: &S) -> JournalConfig<<A as commonware_codec::Read>::Cfg> {
    JournalConfig {
        partition: PART.into(),
        compression: None,
        codec_config: s.certificate_codec_config(),
        page_cache: CacheRef::new(paged::page_size(256), NZUsize!(4)),
        write_buffer: NZUsize!(488),
    }
}
fn actual_journal(seed: u64, pause: u64) -> serde_json::Value {
    let path = place(&format!("journal-{seed}-{pause}"));
    let fx = fixture();
    let verifier = fx.verifier.clone();
    let mut artifacts = vec![fx.artifacts[0].clone()];
    artifacts.extend(std::iter::repeat_n(fx.artifacts[3].clone(), 6));
    artifacts.extend([fx.artifacts[1].clone(), fx.artifacts[2].clone()]);
    let expected: Vec<_> = artifacts.iter().map(|a| a.encode().to_vec()).collect();
    let wire = fx.wire.clone();
    let cert = fx.cert.clone();
    let r = run(
        &path,
        fx.policy.clone(),
        seed,
        pause,
        |_| {},
        async move |d| {
            d.grant_async(g(3), Input::None, |d| {
                d.spawn_session(CID, 2_000_000, move |c| async move {
                    let mut journal: Journal<_, A> =
                        Journal::init(c.child("journal"), journal_config(&verifier))
                            .await
                            .unwrap();
                    let mut offset = 0;
                    for (index, a) in artifacts.iter().enumerate() {
                        let (next, at, length) = journal.append(1, a).await.unwrap();
                        assert_eq!(at, offset);
                        assert_eq!(length as usize, a.encode().len());
                        offset += frame(a).len() as u64;
                        journal = next;
                        if index == 0 || index == 6 || index == 8 {
                            journal = journal.sync(1).await.unwrap();
                        }
                    }
                })
            })
            .await?;
            d.grant_async(g(4), Input::None, |d| {
                d.stage_witnessed_send(wire.clone(), &[4, 1])?;
                d.stage_witnessed_send(cert, &[2])
            })
            .await?;
            let head = d.witness_head();
            d.grant_async(g(5), Input::None, |d| d.stage_witnessed_send(wire, &[1, 4]))
                .await?;
            assert_eq!(head, d.witness_head());
            Ok(())
        },
    );
    assert!(r.ok);
    assert_eq!(count(&r.records, "witness"), expected.len());
    assert_eq!(count(&r.records, "eligible"), 3);
    assert_eq!(count(&r.records, "submission"), 3);
    let raw = fs::read(path.join("store").join(PART).join("0000000000000001")).unwrap();
    let decoded = fx
        .policy
        .decode_section(1, native_data(&raw).unwrap())
        .unwrap();
    assert_eq!(
        decoded
            .iter()
            .map(|d| d.artifact_bytes.clone())
            .collect::<Vec<_>>(),
        expected
    );
    assert!(decoded.iter().any(|f| f.pages.len() > 1));
    let target = path.join("generation2");
    let report = recover_witness_generation(
        RecoveryPaths {
            source_storage: &path.join("store"),
            source_ledger: &path.join("witness"),
            fresh_target: &target,
        },
        fx.policy.clone(),
        fx.target.clone(),
        r.head,
    )
    .unwrap();
    assert_eq!(
        fs::read(path.join("store").join(PART).join("0000000000000001")).unwrap(),
        raw
    );
    assert_eq!(
        report.source.source_files,
        vec![(1, h("I1b/source-raw/v1", &[&raw]))]
    );
    let target_raw = fs::read(target.join("native").join(PART).join("0000000000000001")).unwrap();
    assert_eq!(target_raw, raw);
    assert_eq!(count(&report.records, "witness"), expected.len());
    let old_barriers: Vec<_> = r
        .records
        .iter()
        .flat_map(|r| &r.entries)
        .filter_map(|e| {
            if let LedgerEntry::Barrier(b) = e {
                Some(b.id)
            } else {
                None
            }
        })
        .collect();
    let new_barriers: Vec<_> = report
        .records
        .iter()
        .flat_map(|r| &r.entries)
        .filter_map(|e| {
            if let LedgerEntry::Barrier(b) = e {
                Some(b)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(new_barriers.len(), 1);
    assert_eq!(new_barriers[0].kind, 2);
    assert!(!old_barriers.contains(&new_barriers[0].id));
    let wire_set = |records: &[LedgerRecord]| {
        let mut values: Vec<_> = records
            .iter()
            .flat_map(|r| &r.entries)
            .filter_map(|e| {
                if let LedgerEntry::Eligible(p) = e {
                    Some((p.id, p.wire.clone()))
                } else {
                    None
                }
            })
            .collect();
        values.sort_by_key(|v| v.0);
        values
    };
    assert_eq!(wire_set(&r.records), wire_set(&report.records));
    let output = json!({"seed":seed,"events":r.events,"records":r.records,"head":r.head,"raw":raw,"recovery_records":report.records,"recovery_head":report.new_head,"recovery_source":report.source});
    save(&path.join("logical-evidence.json"), &output);
    output
}

#[test]
fn w9_w11_actual_journal_cross_page_partial_rewrite_and_three_seed_wall_pairs() {
    for seed in [7, 42, 99] {
        assert_eq!(
            actual_journal(seed, 0),
            actual_journal(seed, 2),
            "logical evidence changed under wall pause for seed {seed}"
        );
    }
}

#[test]
fn w10_committed_floor_head_tail_and_raw_header_corruptions() {
    let path = place("corruptions");
    let fx = fixture();
    let r = make_run(&path, &fx, true);
    assert!(r.ok);
    let mut prefix = r.records[..1].to_vec();
    let prefix_file = path.join("prefix");
    let floor = write_ledger(&prefix_file, &mut prefix);
    let descendants = WitnessLedger::open_anchored(
        &path.join("witness"),
        fx.policy.clone(),
        LedgerTail::RejectComplete,
        floor,
    )
    .unwrap();
    assert_eq!(descendants.head(), r.head);
    drop(descendants);
    assert!(
        WitnessLedger::open_anchored(
            &prefix_file,
            fx.policy.clone(),
            LedgerTail::RejectComplete,
            r.head
        )
        .is_err()
    );
    let wrong_floor = WitnessHead {
        hash: [0; 32],
        ..floor
    };
    assert!(
        WitnessLedger::open_anchored(
            &path.join("witness"),
            fx.policy.clone(),
            LedgerTail::RejectComplete,
            wrong_floor
        )
        .is_err()
    );
    for corruption in [
        "header-length",
        "complement",
        "record-hash",
        "head-hash",
        "truncate",
        "missing-head",
        "incomplete-tail",
        "complete-corrupt-tail",
    ] {
        let copied = path.join(corruption);
        copy_ledger(&path.join("witness"), &copied);
        let mut bytes = fs::read(&copied).unwrap();
        match corruption {
            "header-length" => bytes[7] ^= 1,
            "complement" => bytes[8] ^= 1,
            "record-hash" => {
                let len = bytes.len();
                bytes[len - 1] ^= 1;
            }
            "truncate" => {
                bytes.pop();
            }
            "incomplete-tail" => bytes.extend_from_slice(&[1, 2, 3]),
            "complete-corrupt-tail" => {
                bytes.extend_from_slice(&1u64.to_be_bytes());
                bytes.extend_from_slice(&(!1u64).to_be_bytes());
                bytes.push(0);
                bytes.extend_from_slice(&[0; 32]);
            }
            "head-hash" => {
                let hp = copied.with_extension("head");
                let mut hd = fs::read(&hp).unwrap();
                hd[87] ^= 1;
                fs::write(hp, hd).unwrap();
            }
            "missing-head" => {
                fs::rename(
                    copied.with_extension("head"),
                    path.join("retained-missing-head"),
                )
                .unwrap();
            }
            _ => unreachable!(),
        }
        fs::write(&copied, &bytes).unwrap();
        let result = WitnessLedger::open_anchored(
            &copied,
            fx.policy.clone(),
            LedgerTail::PromoteVerified,
            r.head,
        );
        if corruption == "incomplete-tail" {
            assert_eq!(result.unwrap().head(), r.head);
            assert_eq!(fs::read(&copied).unwrap().len(), bytes.len() - 3);
        } else {
            assert!(result.is_err(), "{corruption}");
            assert_eq!(
                fs::read(&copied).unwrap(),
                bytes,
                "corrupt committed evidence was mutated"
            );
        }
    }
    let raw = fs::read(path.join("store").join(PART).join("0000000000000001")).unwrap();
    for offset in [0, 4, 6, 8, 12, 4095] {
        let mut corrupted = raw.clone();
        corrupted[offset] ^= 1;
        assert!(native_data(&corrupted).is_err(), "header byte {offset}");
    }
    let mut wrong_layout = raw.clone();
    wrong_layout[5] = 2;
    let sum = crc(&wrong_layout[..8]);
    wrong_layout[8..12].copy_from_slice(&sum.to_be_bytes());
    assert!(native_data(&wrong_layout).is_err());
}

#[test]
fn w10_semantic_coverage_mutants_rehash_all_affected_ids() {
    let path = place("coverage-mutants");
    let fx = fixture();
    let image = pages(&[frame(&cert_bit(true)), frame(&cert_bit(false))].concat());
    let raw = image.clone();
    let r = run(
        &path,
        fx.policy.clone(),
        7,
        0,
        |_| {},
        async move |d| {
            d.grant_async(g(3), Input::None, |d| {
                d.spawn_session(CID, 2_000_000, move |c| async move {
                    let (b, _) = c.open(PART, &NAME).await.unwrap();
                    b.write_at(0, image, WriteOptions::SYNC).await.unwrap();
                })
            })
            .await?;
            Ok(())
        },
    );
    assert!(r.ok);
    let blob = r
        .records
        .iter()
        .flat_map(|r| &r.entries)
        .find_map(|e| {
            if let LedgerEntry::Blob(b) = e {
                Some(b.clone())
            } else {
                None
            }
        })
        .unwrap();
    let mut prefix = r.records[..1].to_vec();
    let floor = write_ledger(&path.join("floor"), &mut prefix);
    let decoded = fx.policy.decode_section(1, &raw).unwrap();
    for kind in [
        "missing-prefix",
        "missing-footer",
        "missing-extent",
        "wrong-selected-slot",
        "wrong-logical-offset",
    ] {
        let ranges: Vec<(usize, usize)> = match kind {
            "missing-prefix" => vec![(5, 256)],
            "missing-footer" => vec![(0, 244), (250, 256)],
            "missing-extent" => vec![(0, 250)],
            _ => vec![(0, 256)],
        };
        let bs: Vec<_> = ranges
            .iter()
            .enumerate()
            .map(|(i, (start, end))| {
                barrier(
                    blob.id,
                    i as u64 + 20,
                    1,
                    *start as u64,
                    raw[*start..*end].to_vec(),
                )
            })
            .collect();
        let mut frame = decoded[1].clone();
        match kind {
            "wrong-selected-slot" => frame.pages[0].selected_slot = 1,
            "wrong-logical-offset" => frame.logical_offset += 1,
            _ => {}
        }
        let w = witness(blob.id, frame, bs.iter().map(|b| b.id).collect());
        let mut entries = vec![LedgerEntry::Blob(blob.clone())];
        entries.extend(bs.into_iter().map(LedgerEntry::Barrier));
        entries.push(LedgerEntry::Witness(w));
        let mut records = prefix.clone();
        records.push(LedgerRecord {
            schema: 1,
            sequence: 0,
            previous_record_hash: [0; 32],
            identity: prefix[0].identity.clone(),
            entries,
        });
        let target = path.join(kind);
        let head = write_ledger(&target, &mut records);
        let result = WitnessLedger::open_anchored(
            &target,
            fx.policy.clone(),
            LedgerTail::RejectComplete,
            floor,
        );
        save(
            &path.join(format!("{kind}.json")),
            &json!({"head":head,"accepted":result.is_ok(),"error":result.as_ref().err().map(ToString::to_string),"records":records}),
        );
        assert!(result.is_err(), "semantic mutant {kind} accepted");
    }
}

#[test]
fn w9_w10_recovery_rejects_existing_alias_and_conflicting_source_without_repair() {
    let path = place("recovery-paths");
    let fx = fixture();
    let r = make_run(&path, &fx, false);
    assert!(r.ok);
    let store = path.join("store");
    let ledger = path.join("witness");
    let raw_file = store.join(PART).join("0000000000000001");
    let original = fs::read(&raw_file).unwrap();
    let empty = path.join("existing-empty");
    fs::create_dir(&empty).unwrap();
    let link = path.join("link");
    std::os::unix::fs::symlink(&store, &link).unwrap();
    for target in [&empty, &link, &store, &store.join("inside")] {
        let result = recover_witness_generation(
            RecoveryPaths {
                source_storage: &store,
                source_ledger: &ledger,
                fresh_target: target,
            },
            fx.policy.clone(),
            fx.target.clone(),
            r.head,
        );
        assert!(result.is_err());
        assert_eq!(fs::read(&raw_file).unwrap(), original);
    }
    // The raw corruption is applied only to this test's private copy. Its new CRC
    // is valid, so rejection must compare protected canonical history.
    let mut corrupted = original.clone();
    let data = &mut corrupted[4096..];
    let frames = fx.policy.decode_section(1, data).unwrap();
    let offset = frames[2].logical_offset as usize;
    let physical = (offset / 244) * 256 + offset % 244;
    data[physical + 4] ^= 1;
    let page_start = physical / 256 * 256;
    let page = &mut data[page_start..page_start + 256];
    let len = usize::from(u16::from_be_bytes(page[244..246].try_into().unwrap()));
    slot(page, 0, len);
    fs::write(&raw_file, &corrupted).unwrap();
    let result = recover_witness_generation(
        RecoveryPaths {
            source_storage: &store,
            source_ledger: &ledger,
            fresh_target: &path.join("corrupt-target"),
        },
        fx.policy,
        fx.target,
        r.head,
    );
    assert!(result.is_err());
    assert_eq!(fs::read(&raw_file).unwrap(), corrupted);
    assert_eq!(
        fs::read(path.join("corrupt-target/source-snapshot/0000000000000001")).unwrap(),
        corrupted
    );
}

#[test]
fn w2_completed_start_sync_is_evidence_despite_dropped_or_aborted_observer() {
    for abort in [false, true] {
        let path = place("start-sync");
        let fx = fixture();
        let image = pages(&frame(&cert_bit(true)));
        let r = run(
            &path,
            fx.policy,
            7,
            0,
            |_| {},
            async move |d| {
                d.grant_async(g(3), Input::None, |d| {
                    d.spawn_session(CID, 2_000_000, move |c| async move {
                        let (b, _) = c.open(PART, &NAME).await.unwrap();
                        b.write_at(0, image, WriteOptions::default()).await.unwrap();
                        let handle = b.start_sync().await;
                        if abort {
                            handle.abort();
                        }
                        drop(handle);
                    })
                })
                .await?;
                Ok(())
            },
        );
        assert!(r.ok);
        assert_eq!(count(&r.records, "witness"), 1);
        let barrier = r
            .records
            .iter()
            .flat_map(|r| &r.entries)
            .find_map(|e| {
                if let LedgerEntry::Barrier(b) = e {
                    Some(b)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(barrier.kind, 2);
        assert!(r.events.iter().any(|e| matches!(
            e,
            Event::IoReleased {
                io_id: 3,
                delivered: false
            }
        )));
    }
}

#[test]
fn w8_root_survives_controller_panic_with_retained_evidence() {
    let path = place("controller-panic");
    let fx = fixture();
    let image = pages(&frame(&cert_bit(true)));
    let reached = Arc::new(AtomicBool::new(false));
    let marker = reached.clone();
    let policy = fx.policy.clone();
    let result: Result<(), Error> = ControlledDriver::run_async(
        Config::default()
            .with_worker_threads(1)
            .with_storage_directory(path.join("store")),
        profile(7),
        async |d| {
            d.configure_witness(policy, &path.join("witness"))?;
            let (mut entered, release) = d.hold_witness_before_persist(3)?;
            calibrate(d);
            let mut grant = Box::pin(d.grant_async(g(3), Input::None, |d| {
                d.spawn_session(CID, 2_000_000, move |c| async move {
                    let (b, _) = c.open(PART, &NAME).await.unwrap();
                    b.write_at(0, image, WriteOptions::default()).await.unwrap();
                    b.sync().await.unwrap();
                    marker.store(true, Ordering::SeqCst);
                })
            }));
            tokio::select! {r=&mut entered=>r.unwrap(),v=&mut grant=>panic!("unexpected held grant completion: {v:?}")};
            drop(grant);
            save(&path.join("before-panic-events.json"), &d.events());
            release.send(()).unwrap();
            panic!("independent controller fault")
        },
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "controller panic");
    assert!(!reached.load(Ordering::SeqCst));
    let ledger =
        WitnessLedger::open(&path.join("witness"), fx.policy, LedgerTail::RejectComplete).unwrap();
    assert_eq!(count(ledger.records(), "witness"), 1);
    assert_eq!(count(ledger.records(), "submission"), 0);
    save(
        &path.join("after-root-drain-records.json"),
        &ledger.records(),
    );
}

#[test]
fn w4_w10_rehashed_artifact_substring_is_not_a_journal_frame() {
    let path = place("nonboundary-frame");
    let fx = fixture();
    let embedded = frame(&cert_bit(true));
    let mut digest = [0x55; 32];
    digest[..embedded.len()].copy_from_slice(&embedded);
    let proposal = Proposal::new(
        Round::new(Epoch::new(1), View::new(1)),
        View::new(0),
        Digest(digest),
    );
    let artifact = A::Finalize(Finalize::sign(&fx.signers[0], proposal).unwrap());
    let physical = pages(&frame(&artifact));
    let original = fx.policy.decode_section(1, &physical).unwrap();
    assert_eq!(original.len(), 1);
    let offset = physical
        .windows(embedded.len())
        .position(|part| part == embedded)
        .unwrap();
    assert!(offset > 0);
    let r = run(
        &path,
        fx.policy.clone(),
        7,
        0,
        |_| {},
        async move |d| {
            d.grant_async(g(3), Input::None, |d| {
                d.spawn_session(CID, 2_000_000, move |c| async move {
                    let (blob, _) = c.open(PART, &NAME).await.unwrap();
                    blob.write_at(0, physical, WriteOptions::SYNC)
                        .await
                        .unwrap();
                })
            })
            .await?;
            Ok(())
        },
    );
    assert!(r.ok);
    let b = r
        .records
        .iter()
        .flat_map(|r| &r.entries)
        .find_map(|entry| {
            if let LedgerEntry::Barrier(b) = entry {
                Some(b.clone())
            } else {
                None
            }
        })
        .unwrap();
    let forged = witness(
        b.blob_id,
        DecodedArtifact {
            section: 1,
            logical_offset: offset as u64,
            frame_bytes: embedded,
            artifact_bytes: cert_bit(true).encode().to_vec(),
            pages: original[0].pages.clone(),
            wire: None,
        },
        vec![b.id],
    );
    let mut records = r.records.clone();
    records.push(LedgerRecord {
        schema: 1,
        sequence: 0,
        previous_record_hash: [0; 32],
        identity: records[0].identity.clone(),
        entries: vec![LedgerEntry::Witness(forged)],
    });
    let mutant = path.join("mutant");
    let head = write_ledger(&mutant, &mut records);
    let outcome =
        WitnessLedger::open_anchored(&mutant, fx.policy, LedgerTail::RejectComplete, r.head);
    save(
        &path.join("mutant-outcome.json"),
        &json!({"minimum_trusted_receipt":r.head,"mutant_head":head,"embedded_offset":offset,"accepted":outcome.is_ok(),"error":outcome.as_ref().err().map(ToString::to_string),"records":records}),
    );
    assert!(
        outcome.is_err(),
        "ledger accepted bytes inside a signed payload as a separate journal frame"
    );
}
