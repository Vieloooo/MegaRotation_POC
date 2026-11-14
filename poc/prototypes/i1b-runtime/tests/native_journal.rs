mod support;

use commonware_codec::{DecodeExt as _, Encode as _};
use commonware_consensus::{
    simplex::{
        scheme::bls12381_multisig::Scheme,
        types::{Artifact, Finalize, Notarize, Proposal, Vote},
    },
    types::{Epoch, Round, View},
};
use commonware_cryptography::{
    Signer as _,
    bls12381::primitives::{ops, variant::MinSig},
    certificate::{Scheme as _, Verifier as _},
    ed25519::{PrivateKey, PublicKey},
    sha256::Digest,
};
use commonware_runtime::{
    ReadOptions,
    buffer::paged::{self, CacheRef},
};
use commonware_storage::journal::segmented::variable::{Config, Journal};
use commonware_utils::{NZUsize, ordered::BiMap};
use poc_native_runtime::{ControlledDriver, Event, Grant, Input, IoOperation, SimContext};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
use support::*;

type NativeScheme = Scheme<PublicKey, MinSig>;
type NativeArtifact = Artifact<NativeScheme, Digest>;
type NativeJournal = Journal<SimContext, NativeArtifact>;

const POP: &[u8] = b"I1b/storage/pop/v1";
const VOTE: &[u8] = b"I1b/storage/vote/v1";

struct Fixture {
    verifier: NativeScheme,
    signer: NativeScheme,
    signer_index: commonware_utils::Participant,
}

fn fixture() -> Fixture {
    let mut rng = ChaCha20Rng::from_seed([42; 32]);
    let mut mapping = vec![];
    let mut secrets = vec![];
    for i in 0..5 {
        let identity = PrivateKey::decode(commonware_codec::Copying(&[16u8 + i; 32]))
            .unwrap()
            .public_key();
        let (secret, public) = ops::keypair::<_, MinSig>(&mut rng);
        let pop = ops::sign_proof_of_possession::<MinSig>(&secret, POP);
        ops::verify_proof_of_possession::<MinSig>(&public, POP, &pop).unwrap();
        mapping.push((identity.clone(), public));
        secrets.push((identity, secret));
    }
    let mapping = BiMap::try_from(mapping).unwrap();
    let (identity, secret) = secrets.remove(0);
    let signer = NativeScheme::signer(VOTE, mapping.clone(), secret).unwrap();
    let index = signer.me().unwrap().signer().unwrap();
    assert_eq!(signer.participants().get(index.into()), Some(&identity));
    Fixture {
        verifier: NativeScheme::verifier(VOTE, mapping),
        signer,
        signer_index: index,
    }
}

fn cfg(verifier: &NativeScheme) -> Config<<NativeArtifact as commonware_codec::Read>::Cfg> {
    Config {
        partition: "i1b-voter-0".to_owned(),
        compression: None,
        codec_config: verifier.certificate_codec_config(),
        page_cache: CacheRef::new(paged::page_size(256), NZUsize!(4)),
        write_buffer: NZUsize!(488),
    }
}

fn proposal(view: u64, parent: u64, digest: u8) -> Proposal<Digest> {
    Proposal::new(
        Round::new(Epoch::new(1), View::new(view)),
        View::new(parent),
        Digest([digest; 32]),
    )
}

fn artifacts() -> (NativeScheme, Vec<(u64, NativeArtifact, Vec<u8>)>) {
    let Fixture {
        verifier,
        signer,
        signer_index,
    } = fixture();
    let first = proposal(1, 0, 0x44);
    let notarize = Notarize::sign(&signer, first.clone()).unwrap();
    let finalize = Finalize::sign(&signer, first).unwrap();
    let second = Notarize::sign(&signer, proposal(2, 1, 0x55)).unwrap();
    assert_eq!(notarize.attestation.signer, signer_index);
    let wire_1 = Vote::Notarize(notarize.clone()).encode().to_vec();
    let wire_2 = Vote::Finalize(finalize.clone()).encode().to_vec();
    let wire_3 = Vote::Notarize(second.clone()).encode().to_vec();
    let cert = NativeArtifact::Certification(Round::new(Epoch::new(1), View::new(1)), true);
    let mut rows = vec![
        (1, NativeArtifact::Notarize(notarize), wire_1),
        (1, NativeArtifact::Finalize(finalize), wire_2),
    ];
    // Legal storage stress records. Span a physical page while retaining a
    // partial tail so later syncs rewrite that tail.
    rows.extend((0..24).map(|_| (1, cert.clone(), vec![])));
    rows.push((2, NativeArtifact::Notarize(second), wire_3));
    (verifier, rows)
}

fn replay(seed: u64, pause: Duration, evidence: Option<&Path>) -> Vec<u8> {
    let dir = TestDir::new();
    let mut profile = profile();
    profile.run_seed = seed;
    let (verifier, entries) = artifacts();
    let expected: Vec<_> = entries
        .iter()
        .map(|(section, value, bytes)| (*section, value.encode().to_vec(), bytes.clone()))
        .collect();
    assert!(expected.iter().all(|(_, encoded, _)| encoded.len() <= 4096));
    let loaded = Arc::new(Mutex::new(vec![]));
    let receiver = loaded.clone();
    let written = ControlledDriver::run_async(dir.config(), profile.clone(), async |driver| {
        driver.set_io_wall_pause(pause)?;
        calibrate(driver, &profile, START + 500);
        let step = driver.grant_async(Grant { id: 3, tick_us: START }, Input::None, |driver| {
            let configuration = cfg(&verifier);
            driver.spawn_session(CID, STOP, |context| async move {
                let mut journal = NativeJournal::init(context, configuration).await.unwrap();
                for (section, artifact, _) in entries {
                    let (new, offset, size) = journal.append(section, &artifact).await.unwrap();
                    assert!(offset + u64::from(size) <= 65_536);
                    journal = new.sync(section).await.unwrap();
                }
            })
        }).await?;
        assert!(step.events.iter().any(|e| matches!(e, Event::IoIssued { target, operation: IoOperation::Write { offset: 0, len: 256, sync: true, .. }, .. } if target.name.as_deref() == Some(&1u64.to_be_bytes()))));
        assert!(step.events.iter().any(|e| matches!(e, Event::IoIssued { target, operation: IoOperation::Write { offset: 256, len: 256, .. }, .. } if target.name.as_deref() == Some(&1u64.to_be_bytes()))));
        driver.shutdown()?;
        Ok(driver.events())
    }).unwrap();

    profile.incarnation += 1;
    let reopened = ControlledDriver::run_async(dir.config(), profile.clone(), async |driver| {
        driver.set_io_wall_pause(pause)?;
        calibrate(driver, &profile, START + 500);
        driver
            .grant_async(
                Grant {
                    id: 3,
                    tick_us: START,
                },
                Input::None,
                |driver| {
                    let configuration = cfg(&verifier);
                    driver.spawn_session(CID, STOP, |context| async move {
                        let journal = NativeJournal::init(context, configuration).await.unwrap();
                        let mut read = journal
                            .replay(0, 0, NZUsize!(256), ReadOptions::DONT_CACHE)
                            .await
                            .unwrap();
                        let mut values = vec![];
                        while let Some(result) = read.next().await {
                            let (section, _, _, artifact) = result.unwrap();
                            let encoded = artifact.encode().to_vec();
                            let wire = match artifact {
                                NativeArtifact::Notarize(vote) => {
                                    Vote::Notarize(vote).encode().to_vec()
                                }
                                NativeArtifact::Finalize(vote) => {
                                    Vote::Finalize(vote).encode().to_vec()
                                }
                                _ => vec![],
                            };
                            values.push((section, encoded, wire));
                        }
                        let _ = read.finish().unwrap();
                        *receiver.lock().unwrap() = values;
                    })
                },
            )
            .await?;
        driver.shutdown()?;
        Ok(driver.events())
    })
    .unwrap();
    let replayed = loaded.lock().unwrap();
    assert_eq!(*replayed, expected);
    let evidence_record = serde_json::json!({
        "native_config": {
            "seed": seed, "epoch": 1, "partition": "i1b-voter-0", "section_range": [1, 2],
            "physical_page_size": 256, "logical_page_size": 244, "cache_pages": 4,
            "write_buffer": 488, "replay_buffer": 256, "compression": null
        },
        "written": written, "reopened": reopened, "source_artifacts_and_vote_bytes": expected,
        "replayed_artifacts_and_vote_bytes": *replayed
    });
    let bytes = serde_json::to_vec(&evidence_record).unwrap();
    if let Some(evidence) = evidence {
        let run = evidence.join(format!("seed{seed}-wall{}ms", pause.as_millis()));
        fs::create_dir(&run).unwrap();
        fs::write(run.join("logical-trace.json"), &bytes).unwrap();
        fs::write(
            run.join("diagnostics.json"),
            serde_json::to_vec(&serde_json::json!({"pause_ms": pause.as_millis()})).unwrap(),
        )
        .unwrap();
        for section in [1u64, 2] {
            let name = format!("{section:016x}");
            let bytes = fs::read(
                dir.config()
                    .storage_directory()
                    .join("i1b-voter-0")
                    .join(&name),
            )
            .unwrap();
            assert!(bytes.len() <= 69_632);
            fs::write(run.join(name), bytes).unwrap();
        }
    }
    bytes
}

#[test]
fn real_journal_replays_complete_artifacts_and_wire_bytes_under_wall_pauses() {
    let evidence = std::env::var_os("I1B_STORAGE_TEST_EVIDENCE").map(std::path::PathBuf::from);
    if let Some(path) = &evidence {
        fs::create_dir(path).unwrap();
    }
    for seed in [7, 42, 99] {
        assert_eq!(
            replay(seed, Duration::ZERO, evidence.as_deref()),
            replay(seed, Duration::from_millis(2), evidence.as_deref()),
            "seed {seed}"
        );
    }
}
