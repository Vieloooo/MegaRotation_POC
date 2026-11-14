mod support;

use commonware_codec::{Copying, DecodeExt as _, Encode as _, varint::UInt};
use commonware_consensus::{
    simplex::{
        scheme::bls12381_multisig::Scheme,
        types::{Artifact, Certificate, Finalize, Notarization, Notarize, Proposal, Vote},
    },
    types::{Epoch, Round, View},
};
use commonware_cryptography::{
    Crc32, Signer as _,
    bls12381::primitives::{ops, variant::MinSig},
    certificate::Verifier as _,
    ed25519::{PrivateKey, PublicKey},
    sha256::Digest,
};
use commonware_parallel::Sequential;
use commonware_runtime::{
    Blob, Storage, Supervisor as _, WriteOptions,
    buffer::paged::{self, CacheRef},
};
use commonware_storage::journal::segmented::variable::{Config, Journal};
use commonware_utils::{NZUsize, iter::NonEmpty, ordered::BiMap};
use poc_native_runtime::{
    ControlledDriver, Event, Grant, Input, LedgerEntry, LedgerTail, NativeWire, RecoveryPaths,
    SimContext, WitnessConfig, WitnessFault, WitnessHead, WitnessLedger, WitnessMember,
    WitnessPolicy, recover_witness_generation,
};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use std::{fs, future::Future, time::Duration};
use support::*;

type S = Scheme<PublicKey, MinSig>;
type A = Artifact<S, Digest>;
const PARTITION: &str = "i1b-voter-0";
const SECTION: [u8; 8] = 1u64.to_be_bytes();

struct Fixture {
    policy: WitnessPolicy,
    target: WitnessPolicy,
    verifier: S,
    records: Vec<A>,
    vote: NativeWire,
    certificate: NativeWire,
}

fn fixture() -> Fixture {
    fixture_with_context(CID, profile().config_hash)
}

fn fixture_with_context(cid: [u8; 32], config_hash: [u8; 32]) -> Fixture {
    let p = profile();
    let cfg = WitnessConfig {
        run_id: p.run_id,
        config_hash,
        cid,
        worker: p.node,
        member: 0,
        epoch: 1,
        generation: 1,
        partition: PARTITION.to_owned(),
        sections: vec![1, 2],
    };
    let namespace = cfg.voting_namespace();
    let mut rng = ChaCha20Rng::from_seed([42; 32]);
    let mut keys = vec![];
    let mut members = vec![];
    let mut secrets = vec![];
    for member in 0..5u32 {
        let key = PrivateKey::decode(Copying(&[16u8 + member as u8; 32]))
            .unwrap()
            .public_key();
        let (secret, bls_key) = ops::keypair::<_, MinSig>(&mut rng);
        let pop = ops::sign_proof_of_possession::<MinSig>(&secret, b"I1b/storage/pop/v1");
        keys.push((key.clone(), bls_key.clone()));
        members.push(WitnessMember {
            member,
            key,
            bls_key,
            pop,
        });
        secrets.push(secret);
    }
    let registry = BiMap::try_from(keys).unwrap();
    let verifier = S::verifier(&namespace, registry.clone());
    let signers: Vec<_> = secrets
        .into_iter()
        .map(|secret| S::signer(&namespace, registry.clone(), secret).unwrap())
        .collect();
    let round = Round::new(Epoch::new(1), View::new(1));
    let proposal = Proposal::new(round, View::new(0), Digest([0x44; 32]));
    let notarizes: Vec<_> = signers
        .iter()
        .map(|signer| Notarize::sign(signer, proposal.clone()).unwrap())
        .collect();
    let cert = Notarization::from_notarizes(
        &verifier,
        NonEmpty::new(&notarizes[0], notarizes[1..4].iter()),
        &Sequential,
    )
    .unwrap();
    let finalize = Finalize::sign(&signers[0], proposal).unwrap();
    let vote = NativeWire {
        channel: 1,
        bytes: Vote::Finalize(finalize.clone()).encode().to_vec(),
    };
    let certificate = NativeWire {
        channel: 2,
        bytes: Certificate::Notarization(cert.clone()).encode().to_vec(),
    };
    let records = vec![
        A::Notarize(notarizes[0].clone()),
        A::Notarization(cert),
        A::Certification(round, true),
        A::Finalize(finalize),
    ];
    let policy = WitnessPolicy::new(cfg.clone(), &members).unwrap();
    let target = WitnessPolicy::new(
        WitnessConfig {
            generation: 2,
            ..cfg
        },
        &members,
    )
    .unwrap();
    Fixture {
        policy,
        target,
        verifier,
        records,
        vote,
        certificate,
    }
}

fn frame(value: &A) -> Vec<u8> {
    let bytes = value.encode();
    let mut framed = UInt(bytes.len() as u32).encode().to_vec();
    framed.extend_from_slice(&bytes);
    framed
}

fn pages(logical: &[u8]) -> Vec<u8> {
    logical
        .chunks(244)
        .flat_map(|chunk| {
            let mut page = vec![0; 256];
            page[..chunk.len()].copy_from_slice(chunk);
            page[244..246].copy_from_slice(&(chunk.len() as u16).to_be_bytes());
            page[246..250].copy_from_slice(&Crc32::checksum(chunk).to_be_bytes());
            page
        })
        .collect()
}

fn grant(id: u64) -> Grant {
    Grant {
        id,
        tick_us: START + id,
    }
}

#[test]
fn actual_native_journal_seals_real_voter_artifacts_and_submissions() {
    let f = fixture();
    let dir = TestDir::new();
    let p = profile();
    let path = dir.config().storage_directory().join("witness");
    ControlledDriver::run_async(dir.config(), p.clone(), async |driver| {
        driver.configure_witness(f.policy.clone(), &path)?;
        calibrate(driver, &p, START + 500);
        let mut wrong = f.vote.clone();
        wrong.bytes[0] = 5; // native Artifact::Finalize tag, not Vote::Finalize
        let events = driver.grant_async(grant(3), Input::None, |driver| {
            assert!(driver.stage_witnessed_send(f.vote.clone(), &[2]).is_err());
            let codec_config = f.verifier.certificate_codec_config();
            driver.spawn_session(CID, STOP, move |context| async move {
                type J = Journal<SimContext, A>;
                let mut journal = J::init(context, Config {
                    partition: PARTITION.to_owned(), compression: None, codec_config,
                    page_cache: CacheRef::new(paged::page_size(256), NZUsize!(4)), write_buffer: NZUsize!(488)
                }).await.unwrap();
                for value in f.records {
                    journal = journal.append(1, &value).await.unwrap().0.sync(1).await.unwrap();
                }
            })
        }).await?;
        for (index, event) in events.events.iter().enumerate() {
            if let Event::WitnessCommitted { io_id, .. } = event {
                let ready = events.events[..index].iter().any(|e| matches!(e, Event::IoReady { io_id: id, .. } if id == io_id));
                let delivered = events.events[index + 1..].iter().any(|e| matches!(e, Event::IoReleased { io_id: id, delivered: true } if id == io_id));
                assert!(ready && delivered);
            }
        }
        let before = driver.witness_records().unwrap().len();
        let step = driver.grant_async(grant(4), Input::None, |driver| {
            assert!(driver.stage_witnessed_send(wrong, &[2]).is_err());
            driver.stage_witnessed_send(f.vote.clone(), &[2, 2])?;
            driver.stage_witnessed_send(f.certificate.clone(), &[2])?;
            Ok(())
        }).await?;
        assert_eq!(driver.witness_records().unwrap().len(), before + 1);
        let first: Vec<_> = step.events.into_iter().filter(|event| matches!(event, Event::WitnessSubmitted { .. })).collect();
        assert_eq!(first.len(), 2);
        let duplicate = driver.grant_async(grant(5), Input::None, |driver| {
            driver.stage_witnessed_send(f.vote.clone(), &[2])?;
            driver.stage_witnessed_send(f.certificate.clone(), &[2])
        }).await?;
        assert_eq!(duplicate.events.into_iter().filter(|event| matches!(event, Event::WitnessSubmitted { .. })).collect::<Vec<_>>(), first);
        assert_eq!(driver.witness_records().unwrap().len(), before + 1);
        Ok(())
    }).unwrap();
    WitnessLedger::open(&path, f.policy, LedgerTail::RejectComplete).unwrap();
}

fn write_at<F: Future<Output = ()> + Send + 'static>(
    id: u64,
    driver: &mut ControlledDriver,
    cid: [u8; 32],
    execute: impl FnOnce(poc_native_runtime::ControlledBlob) -> F + Send + 'static,
) -> impl Future<Output = Result<poc_native_runtime::Step, poc_native_runtime::Error>> {
    driver.grant_async(grant(id), Input::None, move |driver| {
        driver.spawn_session(cid, STOP, |context| async move {
            let (blob, _) = context.open(PARTITION, &SECTION).await.unwrap();
            execute(blob).await;
        })
    })
}

#[test]
fn exact_range_and_full_sync_coverage_do_not_inherit_unrelated_or_empty_barrier() {
    let f = fixture();
    let dir = TestDir::new();
    let p = profile();
    let path = dir.config().storage_directory().join("witness");
    let image = pages(&frame(&f.records[3]));
    ControlledDriver::run_async(dir.config(), p.clone(), async |driver| {
        driver.configure_witness(f.policy.clone(), &path)?;
        calibrate(driver, &p, START + 500);
        let (tx, rx) = tokio::sync::oneshot::channel();
        let (next, wait) = tokio::sync::oneshot::channel();
        write_at(3, driver, CID, async move |blob| {
            blob.write_at(0, image, WriteOptions::default())
                .await
                .unwrap();
            blob.write_at(255, vec![0], WriteOptions::SYNC)
                .await
                .unwrap();
            blob.write_at(0, Vec::<u8>::new(), WriteOptions::SYNC)
                .await
                .unwrap();
            tx.send(()).unwrap();
            wait.await.unwrap();
            blob.sync().await.unwrap();
        })
        .await?;
        assert!(rx.await.is_ok());
        driver
            .grant_async(grant(4), Input::None, |driver| {
                assert!(driver.stage_witnessed_send(f.vote.clone(), &[2]).is_err());
                next.send(()).unwrap();
                Ok(())
            })
            .await?;
        driver
            .grant_async(grant(5), Input::None, |driver| {
                driver.stage_witnessed_send(f.vote, &[2])
            })
            .await?;
        Ok(())
    })
    .unwrap();
}

#[test]
fn witness_faults_and_canceled_evidence_keep_no_failed_step_ack() {
    for fault in [
        None,
        Some(WitnessFault::BeforeLedger),
        Some(WitnessFault::AfterRecordSync),
        Some(WitnessFault::AfterHeadSync),
    ] {
        let f = fixture();
        let dir = TestDir::new();
        let p = profile();
        let path = dir.config().storage_directory().join("witness");
        let image = pages(&frame(&f.records[3]));
        let mut cleanup = false;
        let result = ControlledDriver::run_async(dir.config(), p.clone(), async |driver| {
            driver.configure_witness(f.policy.clone(), &path)?;
            let hold = if let Some(fault) = fault {
                driver.inject_witness_fault(2, fault)?;
                None
            } else {
                Some(driver.hold_witness_before_persist(2)?)
            };
            calibrate(driver, &p, START + 500);
            let active = write_at(3, driver, CID, async move |blob| {
                blob.write_at(0, image, WriteOptions::SYNC).await.unwrap();
                panic!("consumer resumed after witness failure");
            });
            if let Some((entered, hold)) = hold {
                let mut active = Box::pin(active);
                tokio::select! {
                    biased;
                    result = &mut active => panic!("unexpected grant completion {result:?}"),
                    result = entered => result.unwrap(),
                }
                drop(active);
                assert_eq!(driver.active_evidence_id(), Some(2));
                hold.send(()).unwrap();
            } else {
                assert!(active.await.is_err());
            }
            assert!(driver.shutdown_async().await.is_err());
            let events = driver.events();
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e, Event::IoReady { io_id: 2, .. }))
            );
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e, Event::IoDiscarded { io_id: 2 }))
            );
            assert!(!events.iter().any(|e| matches!(
                e,
                Event::Ack { grant: 3 } | Event::IoReleased { io_id: 2, .. }
            )));
            cleanup = true;
            Ok(())
        });
        assert!(result.is_err() && cleanup);
        match fault {
            Some(WitnessFault::AfterRecordSync) => {
                assert!(
                    WitnessLedger::open(&path, f.policy.clone(), LedgerTail::RejectComplete)
                        .is_err()
                );
                assert_eq!(
                    WitnessLedger::open(&path, f.policy, LedgerTail::PromoteVerified)
                        .unwrap()
                        .records()
                        .len(),
                    3
                );
            }
            Some(WitnessFault::BeforeLedger) => assert_eq!(
                WitnessLedger::open(&path, f.policy, LedgerTail::RejectComplete)
                    .unwrap()
                    .records()
                    .len(),
                2
            ),
            _ => assert_eq!(
                WitnessLedger::open(&path, f.policy, LedgerTail::RejectComplete)
                    .unwrap()
                    .records()
                    .len(),
                3
            ),
        }
    }
}

fn prepare(dir: &TestDir, f: &Fixture, seed: u64, pause: u64) -> (WitnessHead, Vec<Event>) {
    let mut p = profile();
    p.run_seed = seed;
    let path = dir.config().storage_directory().join("witness");
    let image = pages(&f.records.iter().flat_map(frame).collect::<Vec<_>>());
    ControlledDriver::run_async(dir.config(), p.clone(), async |driver| {
        driver.set_io_wall_pause(Duration::from_millis(pause))?;
        driver.configure_witness(f.policy.clone(), &path)?;
        calibrate(driver, &p, START + 500);
        write_at(3, driver, CID, async move |blob| {
            blob.write_at(0, image, WriteOptions::SYNC).await.unwrap();
        })
        .await?;
        driver
            .grant_async(grant(4), Input::None, |driver| {
                driver.stage_witnessed_send(f.vote.clone(), &[2])
            })
            .await?;
        Ok((driver.witness_head().unwrap(), driver.events()))
    })
    .unwrap()
}

#[test]
fn fresh_generation_resyncs_exact_history_and_preserves_source() {
    let f = fixture();
    let source = TestDir::new();
    let target = TestDir::new();
    let source_path = source.config().storage_directory().clone();
    let ledger = source_path.join("witness");
    let raw = source_path.join(PARTITION).join("0000000000000001");
    let target_path = target.config().storage_directory().join("fresh");
    let (anchor, timeline) = prepare(&source, &f, 7, 0);
    let minimum = timeline
        .iter()
        .find_map(|event| match event {
            Event::WitnessCommitted {
                io_id: 1,
                sequence,
                head_hash,
            } => Some(WitnessHead {
                sequence: *sequence,
                hash: *head_hash,
            }),
            _ => None,
        })
        .unwrap();
    assert!(minimum.sequence < anchor.sequence);
    let opened = WitnessLedger::open_anchored(
        &ledger,
        f.policy.clone(),
        LedgerTail::RejectComplete,
        minimum,
    )
    .unwrap();
    assert_eq!(opened.head(), anchor);
    let before = fs::read(&raw).unwrap();
    let ledger_before = fs::read(&ledger).unwrap();
    let old = WitnessLedger::open_anchored(
        &ledger,
        f.policy.clone(),
        LedgerTail::RejectComplete,
        minimum,
    )
    .unwrap();
    let report = recover_witness_generation(
        RecoveryPaths {
            source_storage: &source_path,
            source_ledger: &ledger,
            fresh_target: &target_path,
        },
        f.policy.clone(),
        f.target.clone(),
        anchor,
    )
    .unwrap();
    assert_eq!(report.source.old_head_hash, anchor.hash);
    assert_eq!(fs::read(&raw).unwrap(), before);
    assert_eq!(fs::read(&ledger).unwrap(), ledger_before);
    assert_eq!(
        fs::read(target_path.join("source-snapshot/0000000000000001")).unwrap(),
        before
    );
    let eligible = |records: &[poc_native_runtime::LedgerRecord]| {
        records
            .iter()
            .flat_map(|record| &record.entries)
            .filter_map(|entry| match entry {
                LedgerEntry::Eligible(payload) => Some((payload.id, payload.wire.clone())),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(eligible(old.records()), eligible(&report.records));
    assert!(
        report
            .records
            .iter()
            .flat_map(|record| &record.entries)
            .any(|entry| matches!(entry, LedgerEntry::Barrier(barrier) if barrier.kind == 2))
    );
    let original = old
        .records()
        .iter()
        .flat_map(|record| &record.entries)
        .find_map(|entry| match entry {
            LedgerEntry::Submission(value) => Some(value.clone()),
            _ => None,
        })
        .unwrap();
    let mut resumed = WitnessLedger::open_anchored(
        &target_path.join("witness"),
        f.target.clone(),
        LedgerTail::RejectComplete,
        report.new_head,
    )
    .unwrap();
    assert_eq!(
        resumed.submit_recovered(&f.vote, &[2, 2]).unwrap(),
        [original.clone()]
    );
    let submitted_head = resumed.head();
    assert_eq!(resumed.submit_recovered(&f.vote, &[2]).unwrap(), [original]);
    assert_eq!(resumed.head(), submitted_head);
    assert!(
        recover_witness_generation(
            RecoveryPaths {
                source_storage: &source_path,
                source_ledger: &ledger,
                fresh_target: &target_path
            },
            f.policy.clone(),
            f.target.clone(),
            anchor
        )
        .is_err()
    );
    let mut corrupt = before;
    corrupt[4096] ^= 1;
    fs::write(&raw, &corrupt).unwrap();
    let failure_target = target.config().storage_directory().join("rejected");
    assert!(
        recover_witness_generation(
            RecoveryPaths {
                source_storage: &source_path,
                source_ledger: &ledger,
                fresh_target: &failure_target
            },
            f.policy,
            f.target,
            anchor
        )
        .is_err()
    );
    assert_eq!(fs::read(&raw).unwrap(), corrupt);
    assert_eq!(
        fs::read(failure_target.join("source-snapshot/0000000000000001")).unwrap(),
        corrupt
    );
}

#[test]
fn complete_native_witness_traces_replay_across_wall_pauses() {
    let evidence = std::env::var_os("I1B_WITNESS_TEST_EVIDENCE").map(std::path::PathBuf::from);
    if let Some(path) = &evidence {
        fs::create_dir(path).unwrap();
    }
    for seed in [7, 42, 99] {
        let f = fixture();
        let a = TestDir::new();
        let b = TestDir::new();
        let baseline = prepare(&a, &f, seed, 0);
        let wall = prepare(&b, &f, seed, 2);
        assert_eq!(baseline, wall);
        if let Some(evidence) = &evidence {
            for (dir, (head, events), pause) in [(&a, baseline, 0), (&b, wall, 2)] {
                let output = evidence.join(format!("seed{seed}-wall{pause}ms"));
                fs::create_dir(&output).unwrap();
                let cfg = dir.config();
                let source = cfg.storage_directory();
                for item in ["witness", "witness.head", "i1b-voter-0/0000000000000001"] {
                    let path = source.join(item);
                    fs::copy(&path, output.join(path.file_name().unwrap())).unwrap();
                }
                let trace = serde_json::json!({"seed": seed, "policy": f.policy.seal(), "head": head, "events": events});
                fs::write(
                    output.join("logical-trace.json"),
                    serde_json::to_vec(&trace).unwrap(),
                )
                .unwrap();
                fs::write(
                    output.join("diagnostics.json"),
                    format!("{{\"wall_pause_ms\":{pause}}}"),
                )
                .unwrap();
            }
        }
    }
}

#[test]
fn typed_failed_mutation_leaves_section_uncertain_even_after_reopen() {
    let f = fixture();
    let dir = TestDir::new();
    let p = profile();
    let path = dir.config().storage_directory().join("witness");
    let image = pages(&frame(&f.records[3]));
    ControlledDriver::run_async(dir.config(), p.clone(), async |driver| {
        driver.configure_witness(f.policy.clone(), &path)?;
        calibrate(driver, &p, START + 500);
        let step = driver
            .grant_async(grant(3), Input::None, |driver| {
                driver.spawn_session(CID, STOP, |context| async move {
                    let (blob, _) = context.open(PARTITION, &SECTION).await.unwrap();
                    blob.write_at(0, image, WriteOptions::default())
                        .await
                        .unwrap();
                    assert!(matches!(
                        blob.write_at(u64::MAX, Vec::<u8>::new(), WriteOptions::SYNC)
                            .await,
                        Err(commonware_runtime::Error::OffsetOverflow)
                    ));
                    blob.sync().await.unwrap();
                    drop(blob);
                    let (reopened, _) = context.open(PARTITION, &SECTION).await.unwrap();
                    reopened.sync().await.unwrap();
                })
            })
            .await?;
        assert!(step.events.iter().any(|event| matches!(
            event,
            Event::IoReady {
                result: poc_native_runtime::IoResult::Error {
                    kind: "offset_overflow"
                },
                ..
            }
        )));
        assert!(
            !driver
                .witness_records()
                .unwrap()
                .iter()
                .flat_map(|r| &r.entries)
                .any(|entry| matches!(entry, LedgerEntry::Barrier(_) | LedgerEntry::Eligible(_)))
        );
        driver
            .grant_async(grant(4), Input::None, |driver| {
                assert!(driver.stage_witnessed_send(f.vote, &[2]).is_err());
                Ok(())
            })
            .await?;
        Ok(())
    })
    .unwrap();
}

#[test]
fn protected_bytes_cannot_be_overwritten_or_pruned_by_selected_operations() {
    for mutation in ["write", "resize", "remove"] {
        let f = fixture();
        let dir = TestDir::new();
        let p = profile();
        let path = dir.config().storage_directory().join("witness");
        let image = pages(&frame(&f.records[3]));
        let baseline = image.clone();
        let mut checked = false;
        let result = ControlledDriver::run_async(dir.config(), p.clone(), async |driver| {
            driver.configure_witness(f.policy, &path)?;
            calibrate(driver, &p, START + 500);
            let (tx, rx) = tokio::sync::oneshot::channel();
            let outcome = driver
                .grant_async(grant(3), Input::None, |driver| {
                    driver.spawn_session(CID, STOP, move |context| async move {
                        let (blob, _) = context.open(PARTITION, &SECTION).await.unwrap();
                        blob.write_at(0, image, WriteOptions::SYNC).await.unwrap();
                        rx.await.unwrap();
                        match mutation {
                            "write" => blob
                                .write_at(1, vec![0xff], WriteOptions::default())
                                .await
                                .unwrap(),
                            "resize" => blob.resize(12).await.unwrap(),
                            _ => context.remove(PARTITION, None).await.unwrap(),
                        }
                    })
                })
                .await?;
            assert!(
                outcome
                    .events
                    .iter()
                    .any(|e| matches!(e, Event::WitnessCommitted { io_id: 2, .. }))
            );
            assert!(
                driver
                    .grant_async(grant(4), Input::None, |_| {
                        tx.send(()).unwrap();
                        Ok(())
                    })
                    .await
                    .is_err()
            );
            let events = driver.events();
            assert!(events.contains(&Event::IoDiscarded { io_id: 3 }));
            assert!(!events.iter().any(|event| matches!(
                event,
                Event::IoReady { io_id: 3, .. } | Event::Ack { grant: 4 }
            )));
            checked = true;
            Ok(())
        });
        assert!(result.is_err() && checked);
        let raw = fs::read(
            dir.config()
                .storage_directory()
                .join(PARTITION)
                .join("0000000000000001"),
        )
        .unwrap();
        assert_eq!(poc_native_runtime::native_data(&raw).unwrap(), baseline);
    }
}

#[test]
fn malformed_frames_wrong_scope_and_native_context_are_rejected() {
    let f = fixture();
    for bytes in [
        vec![0],
        UInt(4097u32).encode().to_vec(),
        vec![0x81, 0, 9],
        vec![2, 0x11],
        frame(&f.records[3])[..24].to_vec(),
    ] {
        assert!(f.policy.decode_section(1, &pages(&bytes)).is_err());
    }
    let good = pages(&frame(&f.records[3]));
    assert!(f.policy.decode_section(2, &good).is_err());
    let mut foreign = fixture_with_context([0x41; 32], [0x22; 32]);
    assert!(foreign.policy.decode_section(1, &good).is_err());
    foreign = fixture_with_context(CID, [0x40; 32]);
    assert!(foreign.policy.decode_section(1, &good).is_err());

    let mut anchor = [0u8; 256];
    let cert = frame(&f.records[2]);
    anchor[..cert.len()].copy_from_slice(&cert);
    anchor[244..246].copy_from_slice(&(cert.len() as u16).to_be_bytes());
    anchor[246..250].copy_from_slice(&Crc32::checksum(&cert).to_be_bytes());
    anchor[250..252].copy_from_slice(&(244u16).to_be_bytes());
    assert!(
        f.policy.decode_section(1, &anchor).unwrap()[0]
            .wire
            .is_none()
    );
}

#[test]
fn head_corruption_truncation_and_anchored_rollback_fail_without_replacement() {
    let f = fixture();
    let dir = TestDir::new();
    let ledger = dir.config().storage_directory().join("witness");
    let path = ledger.with_extension("head");
    let anchor = prepare(&dir, &f, 7, 0).0;
    let forged_floor = WitnessHead {
        sequence: anchor.sequence - 1,
        hash: [0x99; 32],
    };
    assert!(
        WitnessLedger::open_anchored(
            &ledger,
            f.policy.clone(),
            LedgerTail::RejectComplete,
            forged_floor,
        )
        .is_err()
    );
    let bytes = fs::read(&ledger).unwrap();
    let head = fs::read(&path).unwrap();
    fs::write(&ledger, &bytes[..bytes.len() - 1]).unwrap();
    assert!(
        WitnessLedger::open_anchored(
            &ledger,
            f.policy.clone(),
            LedgerTail::RejectComplete,
            anchor
        )
        .is_err()
    );
    assert_eq!(fs::read(&ledger).unwrap(), &bytes[..bytes.len() - 1]);
    fs::write(&ledger, &bytes).unwrap();
    let mut wrong = head.clone();
    wrong[10] ^= 1;
    fs::write(&path, &wrong).unwrap();
    assert!(WitnessLedger::open(&ledger, f.policy.clone(), LedgerTail::PromoteVerified).is_err());
    fs::remove_file(&path).unwrap();
    assert!(WitnessLedger::open(&ledger, f.policy.clone(), LedgerTail::PromoteVerified).is_err());
    assert!(!path.exists());

    let older = TestDir::new();
    let old = older.config().storage_directory().join("witness");
    let stale = WitnessLedger::create(&old, f.policy.clone()).unwrap();
    assert_ne!(stale.head(), anchor);
    fs::copy(&old, &ledger).unwrap();
    fs::copy(old.with_extension("head"), &path).unwrap();
    assert!(
        WitnessLedger::open_anchored(&ledger, f.policy, LedgerTail::RejectComplete, anchor)
            .is_err()
    );
    assert_eq!(fs::read(&ledger).unwrap(), fs::read(&old).unwrap());
}

#[test]
fn range_sync_requires_other_frame_bytes_and_physical_extent() {
    for omit_earlier_frame in [true, false] {
        let f = fixture();
        let dir = TestDir::new();
        let p = profile();
        let path = dir.config().storage_directory().join("witness");
        let a = frame(&f.records[0]);
        let b = frame(&f.records[3]);
        let image = pages(&[a.as_slice(), b.as_slice()].concat());
        assert_eq!(image.len(), 256);
        let cut = a.len();
        ControlledDriver::run_async(dir.config(), p.clone(), async |driver| {
            driver.configure_witness(f.policy.clone(), &path)?;
            calibrate(driver, &p, START + 500);
            let (more, resume) = tokio::sync::oneshot::channel();
            write_at(3, driver, CID, async move |blob| {
                blob.write_at(0, image.clone(), WriteOptions::default())
                    .await
                    .unwrap();
                let (offset, first, second_offset, second) = if omit_earlier_frame {
                    // End beyond the footer and establish page extent, while A stays dirty.
                    (cut, &image[cut..], 0, &image[..cut])
                } else {
                    // All checksum data and slot zero, but no durable full page extent.
                    (0, &image[..250], 250, &image[250..])
                };
                blob.write_at(offset as u64, first.to_vec(), WriteOptions::SYNC)
                    .await
                    .unwrap();
                resume.await.unwrap();
                blob.write_at(second_offset as u64, second.to_vec(), WriteOptions::SYNC)
                    .await
                    .unwrap();
            })
            .await?;
            assert!(
                !driver
                    .witness_records()
                    .unwrap()
                    .iter()
                    .flat_map(|record| &record.entries)
                    .any(|entry| matches!(entry, LedgerEntry::Witness(_)))
            );
            driver
                .grant_async(grant(4), Input::None, |driver| {
                    assert!(driver.stage_witnessed_send(f.vote.clone(), &[2]).is_err());
                    more.send(()).unwrap();
                    Ok(())
                })
                .await?;
            driver
                .grant_async(grant(5), Input::None, |driver| {
                    driver.stage_witnessed_send(f.vote, &[2])
                })
                .await?;
            Ok(())
        })
        .unwrap();
        WitnessLedger::open(&path, f.policy, LedgerTail::RejectComplete).unwrap();
    }
}

#[test]
fn native_journal_flush_can_end_inside_an_unsealed_frame() {
    let f = fixture();
    let dir = TestDir::new();
    let p = profile();
    let path = dir.config().storage_directory().join("witness");
    let value = f.records[0].clone();
    assert_eq!(frame(&value).len(), 86);
    ControlledDriver::run_async(dir.config(), p.clone(), async |driver| {
        driver.configure_witness(f.policy.clone(), &path)?;
        calibrate(driver, &p, START + 500);
        let (flush, held) = tokio::sync::oneshot::channel();
        let codec = f.verifier.certificate_codec_config();
        let step = driver
            .grant_async(grant(3), Input::None, |driver| {
                driver.spawn_session(CID, STOP, move |context| async move {
                    type J = Journal<SimContext, A>;
                    let inspection = context.child("inspection");
                    let mut journal = J::init(
                        context,
                        Config {
                            partition: PARTITION.to_owned(),
                            compression: None,
                            codec_config: codec,
                            page_cache: CacheRef::new(paged::page_size(256), NZUsize!(4)),
                            write_buffer: NZUsize!(488),
                        },
                    )
                    .await
                    .unwrap();
                    journal = journal
                        .append(1, &value)
                        .await
                        .unwrap()
                        .0
                        .sync(1)
                        .await
                        .unwrap();
                    for _ in 0..5 {
                        journal = journal.append(1, &value).await.unwrap().0;
                    }
                    held.await.unwrap();
                    drop(journal);
                    let (blob, len) = inspection.open(PARTITION, &SECTION).await.unwrap();
                    assert_eq!(len, 512);
                    blob.sync().await.unwrap();
                })
            })
            .await?;
        assert!(step.events.iter().any(|event| matches!(
            event,
            Event::IoIssued {
                operation: poc_native_runtime::IoOperation::Write {
                    offset: 0,
                    len: 512,
                    sync: false,
                    ..
                },
                ..
            }
        )));
        // A separate native full barrier sees the two complete pages and their
        // incomplete sixth frame physically present. Only five frames seal.
        driver
            .grant_async(grant(4), Input::None, |_| {
                flush.send(()).unwrap();
                Ok(())
            })
            .await?;
        let raw = fs::read(
            dir.config()
                .storage_directory()
                .join(PARTITION)
                .join("0000000000000001"),
        )
        .unwrap();
        assert_eq!(poc_native_runtime::native_data(&raw).unwrap().len(), 512);
        assert!(
            f.policy
                .decode_section(1, poc_native_runtime::native_data(&raw).unwrap())
                .is_err()
        );
        let records = driver.witness_records().unwrap();
        let offsets: std::collections::BTreeSet<_> = records
            .iter()
            .flat_map(|record| &record.entries)
            .filter_map(|entry| match entry {
                LedgerEntry::Witness(witness) => Some(witness.frame.logical_offset),
                _ => None,
            })
            .collect();
        assert_eq!(offsets, [0, 86, 172, 258, 344].into_iter().collect());
        Ok(())
    })
    .unwrap();
}
