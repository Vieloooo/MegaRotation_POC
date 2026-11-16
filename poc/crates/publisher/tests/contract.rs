mod support;

use poc_publisher::*;
use sim_espresso::*;
use support::*;

#[test]
fn pe11_pe12_offer_validation_retry_and_same_key_deduplication() {
    let evidence = EvidenceDirectory::new("offer");
    let dir = evidence.path.join("publisher");
    let mut publisher = create(&dir);
    let result = FixtureResult::valid(KEY);
    let inputs = FixtureInputs {
        config: false,
        anchor: false,
        ancestor: false,
        ..Default::default()
    };
    let outcome = publisher.offer(result.clone(), &inputs).unwrap();
    assert_eq!(outcome.phase, Phase::Verifying);
    assert_eq!(
        outcome.missing,
        vec![
            missing(1, b"config"),
            missing(2, b"anchor"),
            missing(3, b"ancestor")
        ]
    );
    drop(publisher);
    let mut publisher = reopen(&dir);
    assert_eq!(publisher.task(&KEY).unwrap().missing, outcome.missing);
    assert!(matches!(
        publisher.prepare_attempt(&KEY, TickUs(100)),
        Err(PublisherError::SubmissionUnavailable(Phase::Verifying))
    ));

    let mut bad_cid = result.clone();
    bad_cid.claimed_cid = 1;
    let mut bad_body = result.clone();
    bad_body.body = 3;
    let mut bad_proof = result.clone();
    bad_proof.proof = 0;
    let mut bad_ancestry = result.clone();
    bad_ancestry.parent = 0;
    for (bad, code) in [
        (bad_cid, 1),
        (bad_body, 2),
        (bad_proof, 3),
        (bad_ancestry, 4),
    ] {
        let generation = publisher.generation();
        assert!(matches!(
            publisher.offer(bad, &FixtureInputs::default()),
            Err(PublisherError::Rejected(RejectReason {code:observed, ..})) if code == observed
        ));
        assert_eq!(publisher.generation(), generation);
        assert_eq!(publisher.task(&KEY).unwrap().missing, outcome.missing);
    }

    assert_eq!(
        publisher
            .offer(result.clone(), &FixtureInputs::default())
            .unwrap()
            .phase,
        Phase::Queued
    );
    let generation = publisher.generation();
    publisher
        .offer(result.clone(), &FixtureInputs::default())
        .unwrap();
    let mut alternative = result.clone();
    alternative.proof = 2;
    publisher
        .offer(alternative, &FixtureInputs::default())
        .unwrap();
    assert_eq!(publisher.generation(), generation);
    assert_eq!(publisher.task(&KEY).unwrap().result, result);
    publisher
        .offer(FixtureResult::valid(OTHER), &FixtureInputs::default())
        .unwrap();
    assert_eq!(publisher.tasks().count(), 2);
    assert_ne!(
        publisher.task(&KEY).unwrap().task_number,
        publisher.task(&OTHER).unwrap().task_number
    );
}

#[test]
fn pe13_prepare_dispatch_ack_and_record_have_distinct_durable_boundaries() {
    let evidence = EvidenceDirectory::new("receipt");
    let dir = evidence.path.join("publisher");
    let inputs = FixtureInputs::default();
    let mut publisher = create(&dir);
    drop(publisher);
    publisher = reopen(&dir);
    assert!(publisher.task(&KEY).is_none());
    publisher.offer(FixtureResult::valid(KEY), &inputs).unwrap();
    drop(publisher);
    publisher = reopen(&dir);
    assert_eq!(publisher.task(&KEY).unwrap().phase(), Phase::Queued);
    let first = publisher.prepare_attempt(&KEY, TickUs(100)).unwrap();
    drop(publisher);
    let mut publisher = reopen(&dir);
    assert_eq!(publisher.task(&KEY).unwrap().phase(), Phase::Queued);
    assert_eq!(publisher.prepare_attempt(&KEY, TickUs(101)).unwrap(), first);
    assert!(matches!(
        publisher.mark_service_acknowledged(
            &first.stable_id,
            b"d1",
            &first.exact_payload,
            b"ack".to_vec()
        ),
        Err(PublisherError::AcknowledgementMismatch)
    ));

    publisher
        .mark_dispatched(&first.stable_id, b"d1".to_vec())
        .unwrap();
    drop(publisher);
    let mut publisher = reopen(&dir);
    assert_eq!(publisher.task(&KEY).unwrap().phase(), Phase::Submitted);
    assert!(matches!(
        publisher.mark_dispatched(&first.stable_id, b"altered".to_vec()),
        Err(PublisherError::DispatchMismatch)
    ));
    let mut wrong_payload = first.exact_payload.to_vec();
    wrong_payload.pop();
    assert!(matches!(
        publisher.mark_service_acknowledged(
            &first.stable_id,
            b"d1",
            &wrong_payload,
            b"ack".to_vec()
        ),
        Err(PublisherError::AcknowledgementMismatch)
    ));
    let mut wrong_id = first.stable_id.clone();
    wrong_id.attempt_number = 99;
    assert!(matches!(
        publisher.mark_service_acknowledged(
            &wrong_id,
            b"d1",
            &first.exact_payload,
            b"ack".to_vec()
        ),
        Err(PublisherError::UnknownAttempt)
    ));
    publisher
        .mark_service_acknowledged(
            &first.stable_id,
            b"d1",
            &first.exact_payload,
            b"ack".to_vec(),
        )
        .unwrap();
    drop(publisher);
    let mut publisher = reopen(&dir);
    assert_eq!(publisher.task(&KEY).unwrap().phase(), Phase::Submitted);

    let mut log = log();
    let q0 = produce(&mut log, 200, 104, vec![proof(KEY, 1)]);
    let q1 = produce(&mut log, 202, 104, vec![body(KEY)]);
    let q2 = produce(&mut log, 204, 104, vec![ancestor(KEY)]);
    drop(publisher);
    publisher = reopen(&dir);
    assert!(publisher.task(&KEY).unwrap().recorded_at.is_none());
    let mut local = reader(READER);
    let mut foreign = reader(b"other-reader");
    foreign
        .receive_block(SOURCE, q0.clone(), TickUs(300))
        .unwrap();
    assert!(matches!(
        publisher.confirm_recorded(&KEY, &foreign.require_prefix_through(0).unwrap(), &inputs),
        Err(PublisherError::PrefixIdentityMismatch)
    ));
    local.receive_block(SOURCE, q0, TickUs(300)).unwrap();
    local.receive_block(SOURCE, q1, TickUs(301)).unwrap();
    assert_eq!(
        publisher
            .confirm_recorded(&KEY, &local.require_prefix_through(1).unwrap(), &inputs)
            .unwrap(),
        ConfirmOutcome::Wait {
            missing: vec![missing(5, b"public ancestor")]
        }
    );
    assert!(publisher.task(&KEY).unwrap().recorded_at.is_none());
    local.receive_block(SOURCE, q2, TickUs(302)).unwrap();
    let prefix = local.require_prefix_through(2).unwrap();
    for receipt_fault in [
        ReceiptFault::Key,
        ReceiptFault::Source,
        ReceiptFault::Index,
        ReceiptFault::PublicationAfterCompleteness,
        ReceiptFault::Future,
    ] {
        let generation = publisher.generation();
        assert!(matches!(
            publisher.confirm_recorded(
                &KEY,
                &prefix,
                &FixtureInputs {
                    receipt_fault,
                    ..Default::default()
                }
            ),
            Err(PublisherError::InvalidReceipt)
        ));
        assert_eq!(publisher.generation(), generation);
    }
    let receipt = match publisher.confirm_recorded(&KEY, &prefix, &inputs).unwrap() {
        ConfirmOutcome::Recorded { key: KEY, receipt } => receipt,
        other => panic!("unexpected confirmation {other:?}"),
    };
    assert_eq!(
        receipt.publication_position,
        EspressoPosition {
            q: 0,
            message_index: 0
        }
    );
    assert_eq!(receipt.complete_through_q, 2);
    drop(publisher);
    let mut publisher = reopen(&dir);
    let generation = publisher.generation();
    let task = publisher.task(&KEY).unwrap();
    assert_eq!(task.phase(), Phase::Recorded);
    assert_eq!(task.recorded_at, Some(receipt));
    assert_eq!(
        publisher.confirm_recorded(&KEY, &prefix, &inputs).unwrap(),
        ConfirmOutcome::AlreadyRecorded
    );
    publisher
        .mark_service_acknowledged(
            &first.stable_id,
            b"d1",
            &first.exact_payload,
            b"ack".to_vec(),
        )
        .unwrap();
    assert_eq!(publisher.generation(), generation);
}

#[test]
fn pe14_public_stop_and_cutoff_are_scoped_monotonic_and_preserve_receipts() {
    let evidence = EvidenceDirectory::new("stop");
    let dir = evidence.path.join("publisher");
    let inputs = FixtureInputs::default();
    let mut publisher = create(&dir);
    for key in [KEY, OTHER] {
        publisher.offer(FixtureResult::valid(key), &inputs).unwrap();
    }
    let first = publisher.prepare_attempt(&KEY, TickUs(10)).unwrap();
    publisher
        .mark_dispatched(&first.stable_id, b"d1".to_vec())
        .unwrap();
    // An unrelated assignment has no projection for either task. A Program TIME
    // event and a Manager switch have no stop input at this component interface.
    assert_eq!(
        publisher
            .apply_public_stop(StopDecision {
                affected_keys: vec![],
                reason: LifecycleReason::OtherCidAssigned,
                public_basis: b"unrelated session".to_vec(),
            })
            .unwrap(),
        0
    );
    let retry = publisher.prepare_attempt(&KEY, TickUs(11)).unwrap();
    assert_eq!(retry.stable_id.attempt_number, 2);
    assert_eq!(retry.exact_payload, first.exact_payload);

    let mut log = log();
    let q0 = produce(&mut log, 100, 104, first.exact_payload.to_vec());
    let q1 = produce(&mut log, 102, 105, vec![message(b"watermark material")]);
    let mut reader = reader(READER);
    reader.receive_block(SOURCE, q0, TickUs(110)).unwrap();
    let p0 = reader.require_prefix_through(0).unwrap();
    assert_eq!(
        publisher.apply_watermark(&KEY, &p0).unwrap(),
        WatermarkOutcome::BelowDeadline
    );
    reader
        .receive_header(SOURCE, BlockHeader::from(&q1), TickUs(111))
        .unwrap();
    assert!(reader.require_watermark(PublicTimeUs(105)).is_err());
    assert_eq!(publisher.prepare_attempt(&KEY, TickUs(12)).unwrap(), retry);
    reader.receive_block(SOURCE, q1, TickUs(113)).unwrap();
    let prefix = reader.require_watermark(PublicTimeUs(105)).unwrap();
    assert_eq!(
        publisher.apply_watermark(&KEY, &prefix).unwrap(),
        WatermarkOutcome::Applied
    );
    assert_eq!(publisher.task(&KEY).unwrap().phase(), Phase::Stopped);
    assert_eq!(publisher.task(&OTHER).unwrap().phase(), Phase::Queued);
    drop(publisher);
    let mut publisher = reopen(&dir);
    assert!(matches!(
        publisher.prepare_attempt(&KEY, TickUs(120)),
        Err(PublisherError::SubmissionUnavailable(Phase::Stopped))
    ));
    assert!(matches!(
        publisher.confirm_recorded(&KEY, &prefix, &inputs),
        Ok(ConfirmOutcome::Recorded { .. })
    ));
    assert_eq!(publisher.task(&KEY).unwrap().phase(), Phase::Recorded);
    assert!(matches!(
        publisher.task(&KEY).unwrap().stop_reason,
        Some(PublicStop::Deadline {
            q: 1,
            theta: PublicTimeUs(105),
            deadline: PublicTimeUs(105)
        })
    ));
    publisher
        .mark_service_acknowledged(
            &first.stable_id,
            b"d1",
            &first.exact_payload,
            b"late service ack".to_vec(),
        )
        .unwrap();
    assert_eq!(publisher.task(&KEY).unwrap().phase(), Phase::Recorded);
    assert!(matches!(
        publisher.prepare_attempt(&KEY, TickUs(121)),
        Err(PublisherError::SubmissionUnavailable(Phase::Recorded))
    ));

    publisher
        .apply_public_stop(StopDecision {
            affected_keys: vec![OTHER],
            reason: LifecycleReason::SessionClosed,
            public_basis: b"close position".to_vec(),
        })
        .unwrap();
    drop(publisher);
    let publisher = reopen(&dir);
    assert_eq!(publisher.task(&KEY).unwrap().phase(), Phase::Recorded);
    assert_eq!(publisher.task(&OTHER).unwrap().phase(), Phase::Stopped);
}

#[test]
fn pe14_an_unverified_stopped_task_cannot_record_until_offer_is_verified() {
    let evidence = EvidenceDirectory::new("unverified-stop");
    let dir = evidence.path.join("publisher");
    let mut publisher = create(&dir);
    let pending = FixtureInputs {
        anchor: false,
        ..Default::default()
    };
    let verified = FixtureInputs::default();
    publisher
        .offer(FixtureResult::valid(KEY), &pending)
        .unwrap();
    let mut log = log();
    let q0 = produce(
        &mut log,
        90,
        104,
        vec![proof(KEY, 1), body(KEY), ancestor(KEY)],
    );
    let mut reader = reader(READER);
    reader.receive_block(SOURCE, q0, TickUs(100)).unwrap();
    let prefix = reader.require_prefix_through(0).unwrap();
    let expected_wait = ConfirmOutcome::Wait {
        missing: vec![missing(2, b"anchor")],
    };
    assert_eq!(
        publisher
            .confirm_recorded(&KEY, &prefix, &verified)
            .unwrap(),
        expected_wait
    );
    publisher
        .apply_public_stop(StopDecision {
            affected_keys: vec![KEY],
            reason: LifecycleReason::BranchClosed,
            public_basis: b"verified branch close".to_vec(),
        })
        .unwrap();
    assert_eq!(
        publisher
            .confirm_recorded(&KEY, &prefix, &verified)
            .unwrap(),
        expected_wait
    );
    publisher
        .offer(FixtureResult::valid(KEY), &verified)
        .unwrap();
    assert_eq!(publisher.task(&KEY).unwrap().phase(), Phase::Stopped);
    assert!(matches!(
        publisher.confirm_recorded(&KEY, &prefix, &verified),
        Ok(ConfirmOutcome::Recorded { .. })
    ));
    assert!(publisher.task(&KEY).unwrap().stop_reason.is_some());
    assert!(matches!(
        publisher.prepare_attempt(&KEY, TickUs(200)),
        Err(PublisherError::SubmissionUnavailable(Phase::Recorded))
    ));
}
