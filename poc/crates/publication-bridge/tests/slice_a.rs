use std::{cell::Cell, collections::BTreeMap};

use poc_core::{
    fixture::{CheckedSetup, FixtureReader, check_setup, reg_a_l1_fixture},
    metadata::{MetadataProvider, RegionalMetadata},
};
use poc_publication_bridge::{
    BridgeError, BridgeInputs, BridgeResult, BridgeStateCodec, ContextVerifier,
    NativePublicationVerifier, publication_messages,
};
use poc_publisher::{
    ConfirmOutcome, OfferVerdict, Phase, PublicationVerifier, Publisher, StoreBinding,
};
use poc_types::domain::*;
use poc_viewer_gfr::{
    CompletenessOutcome, GfrError, GfrParameters, VerificationBasis, VerificationProfile,
    VerifierBinding, public_completeness,
};
use sim_espresso::{
    EspressoPosition, FinalizedLogReader, PublicTimeUs, ReaderState, SimEspresso, TickUs,
    TimestampPolicy, encode_finalized,
};

const SOURCE: &[u8] = b"h3-a/synthetic";
const READER: &[u8] = b"h3-a/reader";
const FIRST_PROOF: &[u8] = b"synthetic proof A";
const SECOND_PROOF: &[u8] = b"synthetic proof B";
const PARAMETERS: GfrParameters = GfrParameters {
    announcement_us: 1,
    grace_us: 9,
    expiry_us: 30,
};

#[test]
fn synthetic_offer_preserves_first_payload_and_records_actual_earliest_proof() {
    let (setup, l1, result) = fixture();
    let context = SyntheticContext(setup.genesis().clone());
    let finality = SyntheticFinality {
        expected: &result,
        calls: Cell::new(0),
    };
    let profile = synthetic_profile();
    let adapter = || {
        NativePublicationVerifier::new(
            setup.config().clone(),
            profile.clone(),
            PARAMETERS,
            SOURCE.to_vec(),
        )
        .unwrap()
    };
    let binding = StoreBinding {
        namespace: b"h3-a/test-publisher".to_vec(),
        configuration: setup.config().reference().unwrap().encode(),
        source_id: SOURCE.to_vec(),
        reader_id: READER.to_vec(),
    };
    let artifact_dir = std::env::var_os("H3_A_TEST_ARTIFACT_DIR");
    let path = artifact_dir
        .clone()
        .map_or_else(std::env::temp_dir, std::path::PathBuf::from)
        .join(format!("h3-a-test-publisher-{}", std::process::id()));
    let mut publisher =
        Publisher::create(&path, binding.clone(), adapter(), BridgeStateCodec).unwrap();

    let unavailable = BridgeInputs {
        finality: None,
        context: &context,
        l1: &l1,
        dep: setup.dep(),
    };
    let full = BridgeResult::new(result.clone()).unwrap();
    let key = full.key();
    let mut partial = result.clone();
    partial.final_proof.clear();
    partial.chain.clear();
    let partial = BridgeResult::new(partial).unwrap();
    let outcome = publisher.offer(partial.clone(), &unavailable).unwrap();
    assert_eq!(outcome.phase, Phase::Verifying);
    assert_eq!(outcome.missing[0].code, 1);
    assert_eq!(finality.calls.get(), 0);
    drop(publisher);
    let mut publisher = Publisher::open(&path, binding, adapter(), BridgeStateCodec).unwrap();
    assert_eq!(publisher.task(&key).unwrap().result, partial);

    let basis_wait = Basis(InputStatus::Wait(InputMissing::State(key.block_hash)));
    let inputs = |basis| BridgeInputs {
        finality: Some(VerifierBinding {
            verifier: &finality,
            basis,
            profile: profile.clone(),
        }),
        ..unavailable
    };
    assert!(
        matches!(adapter().verify_offer(full.clone(), &inputs(&basis_wait)), OfferVerdict::Wait { missing, .. } if missing[0].code == 2)
    );
    assert_eq!(finality.calls.get(), 0);
    let basis_ready = Basis(InputStatus::Ready(()));
    let ready = inputs(&basis_ready);
    assert_eq!(
        publisher.offer(full.clone(), &ready).unwrap().phase,
        Phase::Queued
    );
    let stored = publisher.task(&key).unwrap();
    assert_eq!(stored.verified_deadline, Some(PublicTimeUs(21_000_009)));
    let first_payload = stored.verified_payload.clone().unwrap();

    let mut second = result.clone();
    second.final_proof = SECOND_PROOF.to_vec();
    let second = BridgeResult::new(second).unwrap();
    let public_second = publication_messages(&second, SOURCE).unwrap();
    assert_eq!(
        publisher.offer(second, &ready).unwrap().phase,
        Phase::Queued
    );
    assert_eq!(publisher.task(&key).unwrap().result, full);
    let attempt = publisher.prepare_attempt(&key, TickUs(1)).unwrap();
    assert_eq!(attempt.exact_payload, first_payload);

    // Actual public log publishes the other valid proof at q1, its body at q2,
    // and the first Offer's proof at q3. No GFR selection is supplied.
    let mut log = SimEspresso::new(
        SOURCE.to_vec(),
        TimestampPolicy::Fixed {
            theta_us: 40_000_000,
        },
    )
    .unwrap();
    let mut reader = ReaderState::new(SOURCE.to_vec(), READER.to_vec()).unwrap();
    for (q, messages) in [
        vec![],
        vec![public_second[1].clone()],
        vec![public_second[0].clone()],
        vec![first_payload[1].clone()],
    ]
    .into_iter()
    .enumerate()
    {
        let tick = TickUs(q as u64 + 1);
        log.propose(tick, false, messages).unwrap();
        let block = log.finalize(q as u64, tick).unwrap().clone();
        reader.receive_block(SOURCE, block, tick).unwrap();
    }
    let prefix_through_proof = reader.require_prefix_through(1).unwrap();
    assert_eq!(
        publisher
            .confirm_recorded(&key, &prefix_through_proof, &ready)
            .unwrap(),
        ConfirmOutcome::NotYetPublic
    );
    let prefix = reader.require_prefix_through(3).unwrap();
    let completeness = |cut| {
        public_completeness(
            &prefix,
            cut,
            setup.config(),
            &profile,
            &full.result().descriptor,
            key.block_hash,
            ready.finality.as_ref(),
        )
    };
    assert_eq!(completeness(1), CompletenessOutcome::Incomplete);
    assert_eq!(
        completeness(4),
        CompletenessOutcome::Reject(GfrError::HistoricalBound)
    );
    let CompletenessOutcome::Complete(through_two) = completeness(2) else {
        panic!("complete at q2")
    };
    assert_eq!(through_two.first_complete_q, 2);
    assert_eq!(
        through_two.valid_proof_locations,
        vec![EspressoPosition {
            q: 1,
            message_index: 0
        }]
    );
    let ConfirmOutcome::Recorded { receipt, .. } =
        publisher.confirm_recorded(&key, &prefix, &ready).unwrap()
    else {
        panic!("recorded receipt")
    };
    assert_eq!(
        receipt.publication_position,
        through_two.publication_position
    );
    assert_eq!(receipt.complete_through_q, 3);
    assert_eq!(
        publisher.confirm_recorded(&key, &prefix, &ready).unwrap(),
        ConfirmOutcome::AlreadyRecorded
    );
    if artifact_dir.is_none() {
        drop(publisher);
        std::fs::remove_dir_all(path).unwrap();
    }
}

#[test]
fn bounded_private_store_and_canonical_public_sizes() {
    let (_, _, mut result) = fixture();
    let mut block = result.block.clone();
    block.transactions.resize(4, block.transactions[0].clone());
    result.chain.clear();
    for height in 1..=10 {
        block.metadata.height = height;
        block.metadata.timestamp_us = 10_000_000 + height * 1_000_000;
        result.chain.push(block.clone());
        block.parent = block.digest().unwrap();
    }
    result.block = result.chain[9].clone();
    result.final_proof = vec![11; 32_768];
    result.anchor_evidence.source_objects = vec![vec![22; 4096]; 16];
    let bounded = BridgeResult::new(result.clone()).unwrap();
    let private = bounded.encode_store();
    assert_eq!(&private[..6], b"H3RS\0\x01");
    assert_eq!(BridgeResult::decode_store(&private).unwrap(), bounded);
    assert!(BridgeResult::decode_store(&private[..private.len() - 1]).is_err());
    let mut trailing = private;
    trailing.push(0);
    assert!(BridgeResult::decode_store(&trailing).is_err());
    let source = vec![1; 256];
    let messages = publication_messages(&bounded, &source).unwrap();
    assert_eq!(messages.len(), 11);
    assert!(
        messages[..10]
            .iter()
            .all(|message| message.bytes.len() == 197)
    );
    let mut log = SimEspresso::new(source.clone(), TimestampPolicy::Fixed { theta_us: 0 }).unwrap();
    log.propose(TickUs(0), false, messages).unwrap();
    let public = encode_finalized(&source, log.finalize(0, TickUs(0)).unwrap()).unwrap();
    assert!(public.len() <= 51_560);

    let mut over_proof = result.clone();
    over_proof.final_proof.push(0);
    assert_eq!(BridgeResult::new(over_proof), Err(BridgeError::Capacity));
    let mut over_chain = result.clone();
    over_chain.chain.push(block);
    assert_eq!(BridgeResult::new(over_chain), Err(BridgeError::Capacity));
    let mut over_anchor = result.clone();
    over_anchor.anchor_evidence.source_objects[0].push(0);
    assert_eq!(BridgeResult::new(over_anchor), Err(BridgeError::Capacity));
    result
        .block
        .transactions
        .push(result.block.transactions[0].clone());
    assert_eq!(BridgeResult::new(result), Err(BridgeError::Capacity));
}

fn synthetic_profile() -> VerificationProfile {
    VerificationProfile::Native {
        protocol_version: 1,
        verifier_version: 1,
        source_revision: "SYNTHETIC_NATIVE_BOUNDARY_NOT_CRYPTO".into(),
    }
}

struct SyntheticFinality<'a> {
    expected: &'a RegionalResult,
    calls: Cell<usize>,
}

impl FinalVerifier for SyntheticFinality<'_> {
    fn check(&self, descriptor: &Descriptor, block: &PhysicalBlock, proof: &[u8]) -> bool {
        self.calls.set(self.calls.get() + 1);
        *descriptor == self.expected.descriptor
            && *block == self.expected.block
            && (proof == FIRST_PROOF || proof == SECOND_PROOF)
    }
}

struct Basis(InputStatus<()>);

impl VerificationBasis for Basis {
    fn require(&self, _: &Descriptor, _: &PhysicalBlock) -> InputStatus<()> {
        self.0.clone()
    }
}

struct SyntheticContext(ExecutionAnchor);

impl ContextVerifier for SyntheticContext {
    fn verify_context(
        &self,
        config: &PublicConfig,
        result: &RegionalResult,
    ) -> InputStatus<ExecutionAnchor> {
        if result.descriptor.anchor == self.0.anchor
            && result.anchor_evidence.source
                == AnchorEvidenceSource::Genesis(config.reference().unwrap())
        {
            InputStatus::Ready(self.0.clone())
        } else {
            InputStatus::Reject(InputError::Conflict)
        }
    }
}

fn fixture() -> (CheckedSetup, FixtureReader, RegionalResult) {
    let anchor = Anchor {
        block: BlockRef {
            height: 0,
            origin: 3,
            block_id: Hash32::new([0x21; 32]),
        },
        timestamp_us: 10_000_000,
    };
    let genesis = ExecutionAnchor {
        anchor: anchor.clone(),
        state: ExecutionState {
            metadata: anchor.metadata(),
            accounts: BTreeMap::from([(
                1,
                Account {
                    nonce: 0,
                    balance: 100,
                },
            )]),
        },
        native_hash: Hash32::new([0x42; 32]),
    };
    let l1_fixture = reg_a_l1_fixture();
    let dep = DepFixture {
        first_height: 1,
        rows: vec![vec![]; 10],
    };
    let config = PublicConfig {
        chain_id: 7,
        rule_version: 1,
        verifier_version: 1,
        registry_hash: Hash32::new([30; 32]),
        dep_fixture_hash: dep.digest().unwrap(),
        l1_fixture_hash: l1_fixture.digest().unwrap(),
        genesis: anchor.clone(),
        genesis_state_root: genesis.state.root().unwrap(),
        genesis_native_hash: genesis.native_hash,
        l2_slot_us: 1_000_000,
        l1_genesis_timestamp_us: 0,
        l1_block_us: 2_000_000,
        t_dep_us: 5_000_000,
        t_drift_us: 15_000_000,
        metadata_window_us: 200_000,
        byte_limit: 65_536,
    };
    let setup = check_setup(&config, &genesis, dep, l1_fixture.clone()).unwrap();
    let mut reader = setup.l1().reader();
    let mut finalizer = setup.l1().finalizer();
    for row in l1_fixture.rows {
        let finalized = finalizer
            .finalize(&row.header, row.finalized_at_us)
            .unwrap();
        reader
            .deliver_header(&finalized, row.finalized_at_us)
            .unwrap();
    }
    let config_ref = config.reference().unwrap();
    let window = Window {
        h_min: 1,
        h_max: 10,
    };
    let session = SessionId { beta: 0, r: 0 };
    let input = PlanInput {
        cause: BoundaryCause::Genesis,
        session: session.clone(),
        source_region: None,
        window: window.clone(),
        reference_block: anchor.block.clone(),
    };
    let committee: Vec<_> = (0..5)
        .map(|pid| CommitteeEntry {
            pid: Pid::new(pid).unwrap(),
            identity_key: [pid; 32],
            vote_key: [pid; 96],
            weight: 1,
        })
        .collect();
    let plan = PlanRecord {
        session,
        cause: input.cause,
        source_region: None,
        region: RegionId::new(0).unwrap(),
        eligible_proposers: committee.iter().map(|entry| entry.pid).collect(),
        committee,
        total_weight: 5,
        fault_bound: 1,
        leader_rule: RuleSpec::static_schedule(),
        window,
        reference_block: anchor.block.clone(),
        config_ref,
        authorization_ref: authorization_ref(config_ref, config.registry_hash, &input).unwrap(),
    };
    let program = ProgramConfig::from_plan(&config, &plan).unwrap();
    let cid = regional_cid(&plan.session, anchor.block.block_id, &program).unwrap();
    let metadata = RegionalMetadata::new(&config, &reader).unwrap();
    let InputStatus::Ready(metadata) = metadata.at(&anchor, 1) else {
        panic!("valid fixture metadata")
    };
    let block = PhysicalBlock {
        parent: anchor.block.block_id,
        metadata,
        pre_state_root: genesis.state.root().unwrap(),
        transactions: vec![
            Transfer {
                from: 1,
                to: 2,
                nonce: 0,
                amount: 3,
            }
            .encode_raw(),
        ],
    };
    let result = RegionalResult {
        descriptor: Descriptor { plan, anchor, cid },
        block: block.clone(),
        final_proof: FIRST_PROOF.to_vec(),
        chain: vec![block],
        anchor_evidence: AnchorEvidence {
            source: AnchorEvidenceSource::Genesis(config_ref),
            source_objects: vec![b"synthetic context inputs".to_vec()],
        },
    };
    (setup, reader, result)
}

mod b_cases;
