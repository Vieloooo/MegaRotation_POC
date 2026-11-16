use super::*;
use poc_publication_bridge::*;
use poc_viewer_gfr::{
    FixtureL1Reader, InitInput, PlanError, PlanOutcome, PlanProvider, ReplayReads, ReplayServices,
    ReplayState, ReplayStatus, advance_global, init,
};
use sim_transport::Id;

mod transport;

const L1_SOURCE: &[u8] = b"h3-b/l1";
const B_PARAMETERS: GfrParameters = GfrParameters {
    expiry_us: 60_000_000,
    ..PARAMETERS
};
const RUN: Id = [1; 32];
const CONFIG: Id = [2; 32];
const RECEIPT: &[u8] = b"synthetic committed fixture";

struct SyntheticReceipt(Hash32);

impl DurableCommitPort for SyntheticReceipt {
    fn verify_commit(&self, candidate_hash: Hash32, receipt: &[u8]) -> Result<(), BridgeError> {
        if candidate_hash == self.0 && receipt == RECEIPT {
            Ok(())
        } else {
            Err(BridgeError::SourceMismatch)
        }
    }
}

struct OnePlan(Descriptor);

impl PlanProvider for OnePlan {
    fn resolve(&self, input: &PlanInput, _: &dyn PublicReferenceView) -> PlanOutcome {
        if *input == self.0.plan.input() {
            PlanOutcome::Ready(self.0.plan.clone())
        } else {
            PlanOutcome::None
        }
    }
    fn descriptor(&self, _: &PlanRecord, _: &Anchor) -> Result<Descriptor, PlanError> {
        Ok(self.0.clone())
    }
}

fn session_end(
    setup: &CheckedSetup,
    descriptor: &Descriptor,
    raw_parameters: GfrParameters,
) -> ReplayState {
    let mut l1 = FixtureL1Reader::new(L1_SOURCE.to_vec(), setup.l1().reader()).unwrap();
    let mut finalize = setup.l1().finalizer();
    for row in reg_a_l1_fixture().rows {
        let finalized = finalize.finalize(&row.header, row.finalized_at_us).unwrap();
        l1.deliver_header(&finalized, row.finalized_at_us).unwrap();
        l1.deliver_empty_control(&finalized, row.finalized_at_us)
            .unwrap();
    }
    let reader = watermark_reader(21_000_009);
    let reads = ReplayReads {
        espresso: &reader,
        l1: &l1,
    };
    let metadata = RegionalMetadata::new(setup.config(), &l1).unwrap();
    let plan = OnePlan(descriptor.clone());
    let services = ReplayServices {
        config: setup.config(),
        plan: &plan,
        metadata: &metadata,
        dep: setup.dep(),
        finality: None,
    };
    let initial = init(
        &InitInput::Genesis {
            config: setup.config().clone(),
            genesis: setup.genesis().clone(),
            parameters: raw_parameters,
            expected_verification: synthetic_profile(),
        },
        &reads,
        &services,
    )
    .unwrap()
    .state
    .unwrap();
    let position = ReplayPosition {
        key_us: 21_000_009,
        kind: ReplayKind::SessionEnd,
        index: ReplayIndex::Session(descriptor.plan.session.clone()),
    };
    let result = advance_global(&initial, &reads, &services, Some(&position));
    assert_eq!(result.status, ReplayStatus::Ready);
    assert_eq!(result.state.public_state().pc, Some(position));
    result.state
}

fn watermark_reader(theta: u64) -> ReaderState {
    let mut log =
        SimEspresso::new(SOURCE.to_vec(), TimestampPolicy::Fixed { theta_us: theta }).unwrap();
    log.propose(TickUs(0), false, vec![]).unwrap();
    let block = log.finalize(0, TickUs(0)).unwrap().clone();
    let mut reader = ReaderState::new(SOURCE.to_vec(), READER.to_vec()).unwrap();
    reader.receive_block(SOURCE, block, TickUs(0)).unwrap();
    reader
}

fn binding(setup: &CheckedSetup) -> StopLedgerBinding {
    StopLedgerBinding {
        config: setup.config().clone(),
        expected: synthetic_profile(),
        parameters: B_PARAMETERS,
        espresso_source: SOURCE.to_vec(),
        l1_source: L1_SOURCE.to_vec(),
        publisher_reader: READER.to_vec(),
        owner_reference: b"synthetic owner".to_vec(),
    }
}

#[test]
fn committed_session_close_and_receipt_bound_restore() {
    let (setup, _, result) = fixture();
    let replay = session_end(&setup, &result.descriptor, B_PARAMETERS);
    let record = RecordDelivery::from_replay(&replay, 0).unwrap();
    assert_eq!(record.event_ids().len(), 1);
    let other_parameters = GfrParameters {
        announcement_us: B_PARAMETERS.announcement_us + 1,
        ..B_PARAMETERS
    };
    let reconfigured = session_end(&setup, &result.descriptor, other_parameters);
    let other = RecordDelivery::from_replay(&reconfigured, 0).unwrap();
    assert_eq!(record.id(), other.id());
    assert_ne!(record.parameters(), other.parameters());
    let bridge_result = BridgeResult::new(result.clone()).unwrap();
    let mut ledger = StopLedger::new(binding(&setup)).unwrap();
    let plan = ledger.plan_record(&record).unwrap();
    let owner = SyntheticReceipt(plan.candidate_hash());
    let snapshot = plan.candidate_bytes().to_vec();
    let stale = ledger.plan_record(&record).unwrap();
    assert!(ledger.permit_dispatch(&bridge_result).unwrap());
    assert!(
        ledger
            .confirm(stale, b"fabricated receipt", &owner)
            .is_err()
    );
    ledger.confirm(plan, RECEIPT, &owner).unwrap();
    assert!(!ledger.permit_dispatch(&bridge_result).unwrap());
    assert!(ledger.plan_record(&other).is_err());
    assert_eq!(
        ledger.plan_record(&record).unwrap().candidate_bytes(),
        snapshot
    );
    let restored = StopLedger::restore(binding(&setup), &snapshot, RECEIPT, &owner).unwrap();
    assert_eq!(
        restored.public_stop(&bridge_result).unwrap(),
        ledger.public_stop(&bridge_result).unwrap()
    );
    assert!(
        StopLedger::restore(
            StopLedgerBinding {
                parameters: other_parameters,
                ..binding(&setup)
            },
            &snapshot,
            RECEIPT,
            &owner
        )
        .is_err()
    );
    let mut forged = snapshot;
    *forged.last_mut().unwrap() ^= 1;
    assert!(StopLedger::restore(binding(&setup), &forged, RECEIPT, &owner).is_err());
    let mut forged = result;
    forged.descriptor.plan.session.r += 1;
    let forged = BridgeResult::new(forged).unwrap();
    assert_eq!(forged.key(), bridge_result.key());
    assert!(ledger.public_stop(&forged).is_err());
}

#[test]
fn verified_watermark_and_one_in_flight_attempt_across_two_finalized_blocks() {
    let (setup, l1, result) = fixture();
    let context = SyntheticContext(setup.genesis().clone());
    let finality = SyntheticFinality {
        expected: &result,
        calls: Cell::new(0),
    };
    let basis = Basis(InputStatus::Ready(()));
    let inputs = BridgeInputs {
        finality: Some(VerifierBinding {
            verifier: &finality,
            basis: &basis,
            profile: synthetic_profile(),
        }),
        context: &context,
        l1: &l1,
        dep: setup.dep(),
    };
    let store_path = std::env::var_os("H3_B_TEST_ARTIFACT_DIR")
        .map_or_else(std::env::temp_dir, std::path::PathBuf::from)
        .join(format!("h3-b-test-publisher-{}", std::process::id()));
    let bridge = BridgeResult::new(result.clone()).unwrap();
    let key = bridge.key();
    let mut publisher = Publisher::create(
        &store_path,
        StoreBinding {
            namespace: b"h3-b/publisher".to_vec(),
            configuration: setup.config().reference().unwrap().encode(),
            source_id: SOURCE.to_vec(),
            reader_id: READER.to_vec(),
        },
        NativePublicationVerifier::new(
            setup.config().clone(),
            synthetic_profile(),
            B_PARAMETERS,
            SOURCE.to_vec(),
        )
        .unwrap(),
        BridgeStateCodec,
    )
    .unwrap();
    publisher.offer(bridge.clone(), &inputs).unwrap();
    let mut ledger = StopLedger::new(binding(&setup)).unwrap();
    assert!(ledger.permit_dispatch(&bridge).unwrap());
    let attempt = publisher.prepare_attempt(&key, TickUs(1)).unwrap();
    let mut custody = AttemptCustody::new(CustodyBinding {
        run: RUN,
        config: CONFIG,
        source_id: SOURCE.to_vec(),
        owner_reference: b"synthetic owner".to_vec(),
    })
    .unwrap();
    let plan = custody
        .plan_admit(attempt.clone(), &KnownSubmission(attempt.clone()))
        .unwrap();
    let owner = SyntheticReceipt(plan.candidate_hash());
    custody.confirm(plan, RECEIPT, &owner).unwrap();
    let early = watermark_reader(21_000_008);
    let on_time = watermark_reader(21_000_009);
    assert!(
        ledger
            .plan_watermark(&publisher, &key, &early.require_prefix_through(0).unwrap())
            .unwrap()
            .is_none()
    );
    let plan = ledger
        .plan_watermark(
            &publisher,
            &key,
            &on_time.require_prefix_through(0).unwrap(),
        )
        .unwrap()
        .unwrap();
    let owner = SyntheticReceipt(plan.candidate_hash());
    assert!(ledger.permit_dispatch(&bridge).unwrap());
    ledger.confirm(plan, RECEIPT, &owner).unwrap();
    assert!(!ledger.permit_dispatch(&bridge).unwrap());
    assert_eq!(
        ledger.deadline_watermark(&bridge).unwrap(),
        Some((0, PublicTimeUs(21_000_009), PublicTimeUs(21_000_009)))
    );
    let other = StopLedger::new(StopLedgerBinding {
        parameters: GfrParameters {
            grace_us: 10,
            ..B_PARAMETERS
        },
        ..binding(&setup)
    })
    .unwrap();
    assert!(
        other
            .plan_watermark(
                &publisher,
                &key,
                &on_time.require_prefix_through(0).unwrap()
            )
            .is_err()
    );

    // The already admitted attempt continues through source finalization, while
    // its retained identity and full payload do not change.
    assert_eq!(
        custody
            .plan_admit(attempt.clone(), &KnownSubmission(attempt.clone()))
            .unwrap()
            .operation(),
        &CustodyOperation::AlreadyCommitted
    );
    let mut log = SimEspresso::new(
        SOURCE.to_vec(),
        TimestampPolicy::Fixed {
            theta_us: 21_000_009,
        },
    )
    .unwrap();
    let mut reader = ReaderState::new(SOURCE.to_vec(), READER.to_vec()).unwrap();
    log.propose(TickUs(0), false, vec![]).unwrap();
    reader
        .receive_block(
            SOURCE,
            log.finalize(0, TickUs(0)).unwrap().clone(),
            TickUs(0),
        )
        .unwrap();
    for (q, ordinal) in [(1, 1), (2, 0)] {
        let id = Hash32::new([q as u8; 32]);
        let selected = [ScheduledMessage {
            stable_id: attempt.stable_id.clone(),
            ordinal,
            repeat: false,
        }];
        let plan = custody.plan_propose(id, q, &selected).unwrap();
        let CustodyOperation::Propose { messages, .. } = plan.operation() else {
            panic!("first proposal plan")
        };
        let tick = TickUs(q);
        log.propose(tick, false, messages.clone()).unwrap();
        let owner = SyntheticReceipt(plan.candidate_hash());
        custody.confirm(plan, RECEIPT, &owner).unwrap();
        assert_eq!(
            custody.plan_propose(id, q, &selected).unwrap().operation(),
            &CustodyOperation::AlreadyCommitted
        );
        let block = log.finalize(q, tick).unwrap().clone();
        let plan = custody.plan_finalize(&block).unwrap();
        let owner = SyntheticReceipt(plan.candidate_hash());
        custody.confirm(plan, RECEIPT, &owner).unwrap();
        reader.receive_block(SOURCE, block, tick).unwrap();
    }
    let before = reader.require_prefix_through(1).unwrap();
    assert_eq!(
        publisher.confirm_recorded(&key, &before, &inputs).unwrap(),
        ConfirmOutcome::NotYetPublic
    );
    let after = reader.require_prefix_through(2).unwrap();
    let ConfirmOutcome::Recorded { receipt, .. } =
        publisher.confirm_recorded(&key, &after, &inputs).unwrap()
    else {
        panic!("complete public record")
    };
    assert_eq!(receipt.publication_position.q, 1);
    assert_eq!(receipt.complete_through_q, 2);
    if std::env::var_os("H3_B_TEST_ARTIFACT_DIR").is_none() {
        drop(publisher);
        std::fs::remove_dir_all(store_path).unwrap();
    }
}

struct KnownSubmission(poc_publisher::Submission<ResultKey>);
impl SubmissionVerifier for KnownSubmission {
    fn verify_submission(
        &self,
        source: &[u8],
        submission: &poc_publisher::Submission<ResultKey>,
    ) -> Result<(), BridgeError> {
        if source == SOURCE && *submission == self.0 {
            Ok(())
        } else {
            Err(BridgeError::SourceMismatch)
        }
    }
}
