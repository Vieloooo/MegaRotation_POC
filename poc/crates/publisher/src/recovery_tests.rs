#[path = "../tests/support/mod.rs"]
mod support;

use std::{fs, io::ErrorKind};

use super::*;
use sim_espresso::FinalizedLogReader;
use support::*;

#[test]
fn pe15_pre_rename_failures_keep_original_main_and_pending_intent() {
    let evidence = EvidenceDirectory::new("pre-rename");
    for stage in [
        CommitStage::OpenTemp,
        CommitStage::WriteTemp,
        CommitStage::SyncTemp,
        CommitStage::Rename,
    ] {
        let dir = evidence.path.join(format!("{stage:?}"));
        let mut publisher = create(&dir);
        publisher
            .offer(FixtureResult::valid(KEY), &FixtureInputs::default())
            .unwrap();
        let main = dir.join("state.snapshot");
        let before = fs::read(&main).unwrap();
        publisher.store.fail_once = Some(stage);
        assert!(matches!(publisher.prepare_attempt(&KEY, TickUs(100)),
            Err(PublisherError::Store(StoreError::Io {stage: failed, ..})) if stage == failed));
        assert_eq!(publisher.generation(), 1);
        assert!(publisher.task(&KEY).unwrap().attempts.is_empty());
        assert_eq!(fs::read(&main).unwrap(), before);
        if let Ok(bytes) = fs::read(dir.join("state.tmp")) {
            fs::write(dir.join("uncommitted-evidence.snapshot"), bytes).unwrap();
        }
        drop(publisher);
        let mut publisher = reopen(&dir);
        assert_eq!(publisher.generation(), 1);
        assert!(publisher.task(&KEY).unwrap().attempts.is_empty());
        let submission = publisher.prepare_attempt(&KEY, TickUs(100)).unwrap();
        assert_eq!(submission.stable_id.attempt_number, 1);
        drop(publisher);
        assert_eq!(
            reopen(&dir).prepare_attempt(&KEY, TickUs(200)).unwrap(),
            submission
        );
    }
}

#[test]
fn pe15_rename_then_directory_sync_error_poison_prevents_prepared_replay() {
    let evidence = EvidenceDirectory::new("indeterminate");
    let dir = evidence.path.join("publisher");
    let mut publisher = create(&dir);
    publisher
        .offer(FixtureResult::valid(KEY), &FixtureInputs::default())
        .unwrap();
    let pending = publisher.prepare_attempt(&KEY, TickUs(100)).unwrap();
    let before = fs::read(dir.join("state.snapshot")).unwrap();
    fs::write(evidence.path.join("before.snapshot"), &before).unwrap();
    publisher.store.fail_once = Some(CommitStage::SyncDirectory);
    assert!(matches!(
        publisher.apply_public_stop(StopDecision {
            affected_keys: vec![KEY],
            reason: LifecycleReason::BranchClosed,
            public_basis: b"verified close".to_vec(),
        }),
        Err(PublisherError::Store(StoreError::CommitUnknown(_)))
    ));
    assert_eq!(publisher.generation(), 2);
    assert_eq!(publisher.task(&KEY).unwrap().phase(), Phase::Queued);
    assert_ne!(fs::read(dir.join("state.snapshot")).unwrap(), before);
    assert!(matches!(
        publisher.prepare_attempt(&KEY, TickUs(101)),
        Err(PublisherError::Store(StoreError::Poisoned))
    ));
    assert!(matches!(
        publisher.mark_dispatched(&pending.stable_id, b"grant-40".to_vec()),
        Err(PublisherError::Store(StoreError::Poisoned))
    ));
    drop(publisher);
    let mut publisher = reopen(&dir);
    assert_eq!(publisher.generation(), 3);
    assert_eq!(publisher.task(&KEY).unwrap().phase(), Phase::Stopped);
    assert_eq!(
        publisher.task(&KEY).unwrap().attempts[0].stable_id,
        pending.stable_id
    );
    assert!(matches!(
        publisher.prepare_attempt(&KEY, TickUs(104)),
        Err(PublisherError::SubmissionUnavailable(Phase::Stopped))
    ));
}

#[test]
fn pe15_corrupt_missing_foreign_and_stale_temp_recovery_inputs() {
    let evidence = EvidenceDirectory::new("bad-main");
    let dir = evidence.path.join("publisher");
    let mut publisher = create(&dir);
    publisher
        .offer(FixtureResult::valid(KEY), &FixtureInputs::default())
        .unwrap();
    let open = |binding| {
        Publisher::open(
            &dir,
            binding,
            NonCryptographicFixture,
            NonCryptographicFixture,
        )
    };
    assert!(matches!(
        open(binding()),
        Err(PublisherError::Store(StoreError::WriterActive))
    ));
    drop(publisher);
    let main = dir.join("state.snapshot");
    let valid = fs::read(&main).unwrap();
    fs::write(dir.join("state.tmp"), b"truncated uncommitted file").unwrap();
    assert!(reopen(&dir).task(&KEY).is_some());
    for field in ["namespace", "configuration", "source_id", "reader_id"] {
        let mut foreign = binding();
        match field {
            "namespace" => foreign.namespace.push(1),
            "configuration" => foreign.configuration.push(1),
            "source_id" => foreign.source_id.push(1),
            "reader_id" => foreign.reader_id.push(1),
            _ => unreachable!(),
        }
        assert!(matches!(
            open(foreign),
            Err(PublisherError::Store(StoreError::BindingMismatch))
        ));
    }
    let mut checksum = valid.clone();
    *checksum.last_mut().unwrap() ^= 1;
    let mut version = valid.clone();
    version[4] = 2;
    let mut unknown = valid.clone();
    unknown[0] ^= 1;
    for (name, mutation) in [
        ("truncated", &valid[..valid.len() - 1]),
        ("checksum", checksum.as_slice()),
        ("version", version.as_slice()),
        ("magic", unknown.as_slice()),
    ] {
        fs::write(evidence.path.join(format!("{name}.snapshot")), mutation).unwrap();
        fs::write(&main, mutation).unwrap();
        let opened = open(binding());
        assert!(match (name, opened) {
            ("truncated", Err(PublisherError::Store(StoreError::Malformed)))
            | ("checksum", Err(PublisherError::Store(StoreError::Checksum)))
            | ("version", Err(PublisherError::Store(StoreError::UnsupportedVersion)))
            | ("magic", Err(PublisherError::Store(StoreError::BadMagic))) => true,
            _ => false,
        });
    }
    fs::remove_file(&main).unwrap();
    assert!(
        matches!(open(binding()), Err(PublisherError::Store(StoreError::Io {
        stage:CommitStage::ReadMain, source,
    })) if source.kind() == ErrorKind::NotFound)
    );
    assert!(
        matches!(Publisher::create(&dir, binding(), NonCryptographicFixture, NonCryptographicFixture),
        Err(PublisherError::Store(StoreError::Io {stage:CommitStage::CreateDirectory, source}))
            if source.kind() == ErrorKind::AlreadyExists)
    );
}

#[test]
fn pe15_pre_commit_failures_at_every_task_transition_recover_exactly() {
    let evidence = EvidenceDirectory::new("transitions");
    let dir = evidence.path.join("publisher");
    let main = dir.join("state.snapshot");
    let mut publisher = create(&dir);
    let inputs = FixtureInputs::default();
    let result = FixtureResult::valid(KEY);
    let mut log = log();
    let row = produce(
        &mut log,
        100,
        104,
        vec![proof(KEY, 1), body(KEY), ancestor(KEY)],
    );
    let mut reader = reader(READER);
    reader.receive_block(SOURCE, row, TickUs(110)).unwrap();
    let prefix = reader.require_prefix_through(0).unwrap();

    let mut pending = None;
    for transition in [
        "enqueue",
        "prepare",
        "dispatch",
        "service_ack",
        "record",
        "stop",
    ] {
        let before = fs::read(&main).unwrap();
        fs::write(
            evidence.path.join(format!("before-{transition}.snapshot")),
            &before,
        )
        .unwrap();
        let generation = publisher.generation();
        let mut apply = |publisher: &mut FixturePublisher| {
            match transition {
                "enqueue" => {
                    publisher.offer(result.clone(), &inputs)?;
                }
                "prepare" => {
                    pending = Some(publisher.prepare_attempt(&KEY, TickUs(10))?);
                }
                "dispatch" => {
                    let submission = pending.as_ref().unwrap();
                    publisher
                        .mark_dispatched(&submission.stable_id, b"controlled-dispatch".to_vec())?;
                }
                "service_ack" => {
                    let submission = pending.as_ref().unwrap();
                    publisher.mark_service_acknowledged(
                        &submission.stable_id,
                        b"controlled-dispatch",
                        &submission.exact_payload,
                        b"service-ack".to_vec(),
                    )?;
                }
                "record" => {
                    assert!(matches!(
                        publisher.confirm_recorded(&KEY, &prefix, &inputs)?,
                        ConfirmOutcome::Recorded { .. }
                    ));
                }
                "stop" => {
                    publisher.apply_public_stop(StopDecision {
                        affected_keys: vec![KEY],
                        reason: LifecycleReason::SessionClosed,
                        public_basis: b"source position close".to_vec(),
                    })?;
                }
                _ => unreachable!(),
            }
            Ok::<_, PublisherError>(())
        };
        publisher.store.fail_once = Some(CommitStage::Rename);
        assert!(matches!(
            apply(&mut publisher),
            Err(PublisherError::Store(StoreError::Io {
                stage: CommitStage::Rename,
                ..
            }))
        ));
        assert_eq!(publisher.generation(), generation);
        drop(publisher);
        publisher = reopen(&dir);
        assert_eq!(fs::read(&main).unwrap(), before);
        assert_eq!(publisher.generation(), generation);
        apply(&mut publisher).unwrap();
        fs::copy(
            &main,
            evidence.path.join(format!("after-{transition}.snapshot")),
        )
        .unwrap();
        drop(publisher);
        publisher = reopen(&dir);
        assert_eq!(publisher.generation(), generation + 1);
        let task = publisher.task(&KEY).unwrap();
        match transition {
            "enqueue" => assert_eq!(task.phase(), Phase::Queued),
            "prepare" => assert_eq!(task.attempts[0].state, AttemptState::Prepared),
            "dispatch" => assert!(matches!(
                task.attempts[0].state,
                AttemptState::Dispatched { .. }
            )),
            "service_ack" => assert!(matches!(
                task.attempts[0].state,
                AttemptState::ServiceAcknowledged { .. }
            )),
            "record" => assert_eq!(task.phase(), Phase::Recorded),
            "stop" => {
                assert_eq!(task.phase(), Phase::Recorded);
                assert!(task.stop_reason.is_some());
            }
            _ => unreachable!(),
        }
    }
}
