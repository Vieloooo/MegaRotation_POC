//! SYNTHETIC_NATIVE_BOUNDARY_NOT_CRYPTO. Never real H1/M1 acceptance evidence.
#![allow(dead_code)]
mod fixtures;

use fixtures::{Basis, ExactProofs, Fixture};
use poc_authorization::PlanOutcome;
use poc_core::metadata::RegionalMetadata;
use poc_publication_bridge::{
    BridgeInputs, BridgeResult, BridgeStateCodec, ContextVerifier, NativePublicationVerifier,
    publication_messages,
};
use poc_publisher::{ConfirmOutcome, Phase, PublicationVerifier, Publisher, PublisherError, StoreBinding};
use poc_types::domain::*;
use poc_viewer_gfr::{
    CompletenessInvalid, CompletenessOutcome, FixtureL1Reader, GfrParameters, InitInput,
    PublicObject, ReplayReads, ReplayServices, ReplayStatus, VerificationProfile, VerifierBinding,
    PUBLICATION_CODEC, advance_global, encode_publication, init, public_completeness,
};
use sim_espresso::{
    BlockHeader, FinalizedBlock, FinalizedLogReader, LogMessage, ReaderState, SimEspresso,
    TickUs, TimestampPolicy,
};
use std::{cell::Cell, path::PathBuf, sync::atomic::{AtomicU64, Ordering}};

const SOURCE: &[u8] = b"h3-independent-A-espresso";
const READER: &[u8] = b"h3-independent-A-publisher";
const SYNTHETIC_REV: &str = "TEST-ONLY/SYNTHETIC-NATIVE-BOUNDARY-NOT-CRYPTO/v1";

fn unhex(s: &str) -> Vec<u8> {
    assert_eq!(s.len() % 2, 0);
    s.as_bytes().chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap()).collect()
}

fn profile() -> VerificationProfile {
    VerificationProfile::Native { protocol_version: 1, verifier_version: 1,
        source_revision: SYNTHETIC_REV.into() }
}
fn parameters() -> GfrParameters {
    GfrParameters { announcement_us: 6_000_000, grace_us: 3_000_000, expiry_us: 30_000_000 }
}
fn l1(f: &Fixture) -> FixtureL1Reader {
    let mut r = FixtureL1Reader::new(b"h3-independent-A-l1".to_vec(), f.setup.l1().reader()).unwrap();
    for t in &f.tokens {
        r.deliver_header(t, t.finalized_at_us() + 500_000).unwrap();
        r.deliver_empty_control(t, t.finalized_at_us() + 500_000).unwrap();
    }
    r
}
fn binding<'a>(p: &'a ExactProofs, b: &'a Basis) -> VerifierBinding<'a> {
    VerifierBinding { verifier: p, basis: b, profile: profile() }
}
fn verifier<'a>(f: &Fixture) -> NativePublicationVerifier<'a> {
    NativePublicationVerifier::new(f.config().clone(), profile(), parameters(), SOURCE.to_vec()).unwrap()
}

struct RefView(Anchor);
impl PublicReferenceView for RefView {
    fn reference(&self, b: &BlockRef) -> InputStatus<Anchor> {
        if self.0.block == *b { InputStatus::Ready(self.0.clone()) }
        else { InputStatus::Reject(InputError::InvalidSource) }
    }
}
/// Test-only provenance capability; real Plan/descriptor validation precedes its injected gate.
struct Context<'a> { f: &'a Fixture, mode: Cell<u8> }
impl ContextVerifier for Context<'_> {
    fn verify_context(&self, c: &PublicConfig, r: &RegionalResult) -> InputStatus<ExecutionAnchor> {
        if c != self.f.config() || self.f.plan.validate_descriptor(&r.descriptor).is_err()
            || !matches!(self.f.plan.validate_plan(&r.descriptor.plan,
                &RefView(self.f.genesis().anchor.clone())), PlanOutcome::Ready(_))
            || r.descriptor.anchor != self.f.genesis().anchor
            || r.anchor_evidence.source != AnchorEvidenceSource::Genesis(c.reference().unwrap()) {
            return InputStatus::Reject(InputError::InvalidSource);
        }
        match self.mode.get() {
            0 => InputStatus::Ready(self.f.genesis().clone()),
            1 => InputStatus::Wait(InputMissing::State(c.genesis_state_root)),
            2 => InputStatus::Reject(InputError::InvalidSource),
            _ => {
                let mut wrong = self.f.genesis().clone();
                wrong.anchor.block.block_id = Hash32::new([0x55; 32]);
                InputStatus::Ready(wrong)
            }
        }
    }
}

fn result(f: &Fixture, p: &ExactProofs, r: u64, heights: &[u64], transfer: bool) -> BridgeResult {
    let descriptor = f.descriptor(r, &f.genesis().anchor, &f.genesis().anchor);
    let mut chain = f.physical_chain(f.genesis(), heights, None);
    if transfer {
        chain.last_mut().unwrap().transactions = vec![Transfer { from: 0, to: 1,
            nonce: 0, amount: 5 }.encode_raw()];
    }
    let block = chain.last().unwrap().clone();
    let proof = p.register(&descriptor, &block);
    BridgeResult::new(RegionalResult { descriptor, block, final_proof: proof, chain,
        anchor_evidence: AnchorEvidence {
            source: AnchorEvidenceSource::Genesis(f.config().reference().unwrap()),
            source_objects: vec![],
        } }).unwrap()
}
fn body(b: &PhysicalBlock) -> LogMessage {
    LogMessage { codec: PUBLICATION_CODEC,
        bytes: encode_publication(&PublicObject::Body(b.clone())).unwrap() }
}
fn proof(r: &RegionalResult, bytes: Vec<u8>) -> LogMessage {
    LogMessage { codec: PUBLICATION_CODEC, bytes: encode_publication(&PublicObject::FinalProof {
        descriptor: r.descriptor.clone(), target: r.block.digest().unwrap(), proof: bytes,
    }).unwrap() }
}
fn blocks(source: &[u8], lists: Vec<Vec<LogMessage>>, start: u64) -> Vec<FinalizedBlock> {
    let mut log = SimEspresso::new(source.to_vec(), TimestampPolicy::Offset {
        offset_us: 0, quantum_us: 1 }).unwrap();
    let mut out = Vec::new();
    for (q, list) in lists.into_iter().enumerate() {
        let t = start + q as u64 * 10;
        log.propose(TickUs(t), true, list).unwrap();
        out.push(log.finalize(q as u64, TickUs(t + 1)).unwrap().clone());
    }
    out
}
fn reader(source: &[u8], id: &[u8], bs: &[FinalizedBlock], order: &[usize]) -> ReaderState {
    let mut r = ReaderState::new(source.to_vec(), id.to_vec()).unwrap();
    for &i in order { r.receive_block(source, bs[i].clone(), TickUs(50_000_000 + i as u64)).unwrap(); }
    r
}
fn store_binding() -> StoreBinding {
    StoreBinding { namespace: b"h3-independent-A".to_vec(), configuration: b"native-routing-synthetic-v1".to_vec(),
        source_id: SOURCE.to_vec(), reader_id: READER.to_vec() }
}
struct OwnDir(PathBuf);
impl OwnDir {
    fn new() -> Self {
        static COUNT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::var_os("H3_VERIFY_RUN_DIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
        Self(root.join(format!("h3-independent-a-{}-{}", std::process::id(), COUNT.fetch_add(1, Ordering::Relaxed))))
    }
}
impl Drop for OwnDir { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }

#[test]
fn public_cut_excludes_future_bodies_and_preserves_original_positions() {
    let f = Fixture::new(); let p = ExactProofs::default(); let b = Basis::default();
    let r = result(&f, &p, 0, &[1, 3], false); let rr = r.result();
    let bs = blocks(SOURCE, vec![vec![body(&rr.chain[0])], vec![proof(rr, rr.final_proof.clone())],
        vec![body(&rr.block)]], 14_000_000);
    let read = reader(SOURCE, READER, &bs, &[2, 0, 1]);
    let prefix = read.require_prefix_through(2).unwrap(); let bind = binding(&p, &b);
    assert!(matches!(public_completeness(&prefix, 1, f.config(), &profile(), &rr.descriptor,
        r.key().block_hash, Some(&bind)), CompletenessOutcome::Incomplete));
    let CompletenessOutcome::Complete(c) = public_completeness(&prefix, 2, f.config(), &profile(),
        &rr.descriptor, r.key().block_hash, Some(&bind)) else { panic!("expected complete at q2") };
    assert_eq!(c.first_complete_q, 2); assert_eq!(c.publication_position.q, 1);
    assert_eq!(c.publication_position.message_index, 0); assert_eq!(c.physical_chain, rr.chain);
    assert_eq!(c.body_locations[0].1[0].q, 0); assert_eq!(c.body_locations[1].1[0].q, 2);
    assert!(matches!(public_completeness(&prefix, 3, f.config(), &profile(), &rr.descriptor,
        r.key().block_hash, Some(&bind)), CompletenessOutcome::Reject(_)));
    assert!(matches!(public_completeness(&prefix, 1, f.config(), &profile(), &rr.descriptor,
        r.key().block_hash, Some(&bind)), CompletenessOutcome::Incomplete));
}

#[test]
fn missing_basis_precedes_boolean_checker_and_profile_mismatch_rejects() {
    let f = Fixture::new(); let p = ExactProofs::default(); let b = Basis::default();
    let r = result(&f, &p, 0, &[3], false);
    let bs = blocks(SOURCE, vec![publication_messages(&r, SOURCE).unwrap()], 14_000_000);
    let read = reader(SOURCE, READER, &bs, &[0]); let prefix = read.require_prefix_through(0).unwrap();
    let bind = binding(&p, &b);
    for mode in [1, 2] {
        b.mode.set(mode); let before = p.calls.get();
        let got = public_completeness(&prefix, 0, f.config(), &profile(), &r.result().descriptor,
            r.key().block_hash, Some(&bind));
        assert!(if mode == 1 { matches!(got, CompletenessOutcome::Wait(_)) }
                else { matches!(got, CompletenessOutcome::Reject(_)) });
        assert_eq!(p.calls.get(), before);
    }
    b.mode.set(0);
    assert!(matches!(public_completeness(&prefix, 0, f.config(), &profile(), &r.result().descriptor,
        r.key().block_hash, None), CompletenessOutcome::Wait(_)));
    let wrong = VerifierBinding { profile: VerificationProfile::NonCryptographicFixture { id: "wrong".into() },
        verifier: &p, basis: &b };
    assert!(matches!(public_completeness(&prefix, 0, f.config(), &profile(), &r.result().descriptor,
        r.key().block_hash, Some(&wrong)), CompletenessOutcome::Reject(_)));
    assert!(matches!(public_completeness(&prefix, 0, f.config(), &profile(), &r.result().descriptor,
        r.key().block_hash, Some(&bind)), CompletenessOutcome::Complete(_)));
}

#[test]
fn invalid_proof_then_valid_variants_do_not_poison_later_completeness() {
    let f = Fixture::new(); let p = ExactProofs::default(); let b = Basis::default();
    let r = result(&f, &p, 0, &[3], false); let rr = r.result();
    let second = p.register(&rr.descriptor, &rr.block);
    let bs = blocks(SOURCE, vec![vec![body(&rr.block), proof(rr, vec![0xff])],
        vec![proof(rr, rr.final_proof.clone())], vec![proof(rr, second)]], 14_000_000);
    let read = reader(SOURCE, READER, &bs, &[2, 1, 0]); let prefix = read.require_prefix_through(2).unwrap();
    let bind = binding(&p, &b);
    assert!(matches!(public_completeness(&prefix, 0, f.config(), &profile(), &rr.descriptor,
        r.key().block_hash, Some(&bind)), CompletenessOutcome::Invalid(CompletenessInvalid::FinalProof)));
    let CompletenessOutcome::Complete(c) = public_completeness(&prefix, 2, f.config(), &profile(),
        &rr.descriptor, r.key().block_hash, Some(&bind)) else { panic!("valid later proof lost") };
    assert_eq!(c.first_complete_q, 1); assert_eq!(c.publication_position.q, 1);
    assert_eq!(c.valid_proof_locations.len(), 2);
}

#[test]
fn recorded_waits_for_later_valid_proof_without_poisoning_task() {
    let f = Fixture::new(); let p = ExactProofs::default(); let b = Basis::default(); let l = l1(&f);
    let r = result(&f, &p, 0, &[3], false); let rr = r.result();
    let context = Context { f: &f, mode: Cell::new(0) };
    let inputs = BridgeInputs { finality: Some(binding(&p, &b)), context: &context, l1: &l, dep: f.setup.dep() };
    let dir = OwnDir::new(); let mut publisher = Publisher::create(&dir.0, store_binding(), verifier(&f), BridgeStateCodec).unwrap();
    publisher.offer(r.clone(), &inputs).unwrap();
    let bs = blocks(SOURCE, vec![vec![body(&rr.block), proof(rr, vec![0xff])],
        vec![proof(rr, rr.final_proof.clone())]], 14_000_000);
    let mut read = reader(SOURCE, READER, &bs, &[0]);
    assert!(matches!(publisher.confirm_recorded(&r.key(), &read.require_prefix_through(0).unwrap(), &inputs).unwrap(),
        ConfirmOutcome::NotYetPublic));
    assert_eq!(publisher.task(&r.key()).unwrap().phase(), Phase::Queued);
    read.receive_block(SOURCE, bs[1].clone(), TickUs(50_000_001)).unwrap();
    let ConfirmOutcome::Recorded { receipt, .. } = publisher.confirm_recorded(&r.key(),
        &read.require_prefix_through(1).unwrap(), &inputs).unwrap() else { panic!("late valid proof rejected") };
    assert_eq!(receipt.publication_position.q, 1); assert_eq!(receipt.complete_through_q, 1);
}

#[test]
fn sealed_internal_gfr_cache_can_advance_empty_q_without_optional_binding() {
    let f = Fixture::new(); let p = ExactProofs::default(); let b = Basis::default(); let l = l1(&f);
    let r = result(&f, &p, 0, &[3], false);
    let bs = blocks(SOURCE, vec![publication_messages(&r, SOURCE).unwrap(), vec![]], 14_000_000);
    let read = reader(SOURCE, READER, &bs, &[0, 1]);
    let metadata = RegionalMetadata::new(f.config(), &l).unwrap();
    let reads = ReplayReads { espresso: &read, l1: &l };
    let verified_services = ReplayServices { config: f.config(), plan: &f.plan, metadata: &metadata,
        dep: f.setup.dep(), finality: Some(binding(&p, &b)) };
    let initial = init(&InitInput::Genesis { config: f.config().clone(), genesis: f.genesis().clone(),
        parameters: parameters(), expected_verification: profile() }, &reads, &verified_services).unwrap().state.unwrap();
    let q0 = ReplayPosition { key_us: 14_000_000, kind: ReplayKind::Espresso, index: ReplayIndex::Espresso(0) };
    let first = advance_global(&initial, &reads, &verified_services, Some(&q0));
    assert!(matches!(first.status, ReplayStatus::Ready));
    assert_eq!(first.state.public_state().entries.len(), 3);
    let no_binding = ReplayServices { config: f.config(), plan: &f.plan, metadata: &metadata,
        dep: f.setup.dep(), finality: None };
    let q1 = ReplayPosition { key_us: 14_000_010, kind: ReplayKind::Espresso, index: ReplayIndex::Espresso(1) };
    let second = advance_global(&first.state, &reads, &no_binding, Some(&q1));
    assert!(matches!(second.status, ReplayStatus::Ready), "trusted historical verdict lost: {:?}", second.status);
    assert_eq!(second.state.public_state().pc, Some(q1));
    assert_eq!(second.state.public_state().entries, first.state.public_state().entries);
    assert!(matches!(public_completeness(&read.require_prefix_through(1).unwrap(), 1,
        f.config(), &profile(), &r.result().descriptor, r.key().block_hash, None), CompletenessOutcome::Wait(_)));
}

#[test]
fn source_hole_is_not_public_incompleteness() {
    let f = Fixture::new(); let p = ExactProofs::default(); let r = result(&f, &p, 0, &[3], false);
    let bs = blocks(SOURCE, vec![publication_messages(&r, SOURCE).unwrap()], 14_000_000);
    let mut read = ReaderState::new(SOURCE.to_vec(), READER.to_vec()).unwrap();
    read.receive_header(SOURCE, BlockHeader::from(&bs[0]), TickUs(50_000_000)).unwrap();
    read.receive_message(SOURCE, 0, 0, bs[0].proposal.messages[0].clone(), TickUs(50_000_001)).unwrap();
    assert!(read.require_prefix_through(0).is_err());
    read.receive_message(SOURCE, 0, 1, bs[0].proposal.messages[1].clone(), TickUs(50_000_002)).unwrap();
    assert!(read.require_prefix_through(0).is_ok());
}

#[test]
fn full_offer_partial_storage_and_real_file_receipt_preserve_one_task() {
    let f = Fixture::new(); let p = ExactProofs::default(); let b = Basis::default(); let l = l1(&f);
    let context = Context { f: &f, mode: Cell::new(0) };
    let inputs = BridgeInputs { finality: Some(binding(&p, &b)), context: &context, l1: &l, dep: f.setup.dep() };
    let r = result(&f, &p, 0, &[1, 3], true);
    let mut partial = r.result().clone(); partial.final_proof.clear(); partial.chain.clear();
    let partial = BridgeResult::new(partial).unwrap();
    let restored = BridgeResult::decode_store(&partial.encode_store()).unwrap();
    assert!(restored.result().chain.is_empty()); assert!(publication_messages(&restored, SOURCE).is_err());
    let dir = OwnDir::new();
    let mut publisher = Publisher::create(&dir.0, store_binding(), verifier(&f), BridgeStateCodec).unwrap();
    assert_eq!(publisher.offer(partial, &inputs).unwrap().phase, Phase::Verifying);
    assert_eq!(publisher.offer(r.clone(), &inputs).unwrap().phase, Phase::Queued);
    let frozen = publisher.task(&r.key()).unwrap().verified_payload.clone().unwrap();
    let mut alternative = r.result().clone(); alternative.final_proof = p.register(&alternative.descriptor, &alternative.block);
    publisher.offer(BridgeResult::new(alternative).unwrap(), &inputs).unwrap();
    assert_eq!(publisher.task(&r.key()).unwrap().verified_payload.as_ref().unwrap().as_ref(), frozen.as_ref());
    let submission = publisher.prepare_attempt(&r.key(), TickUs(14_000_000)).unwrap();
    assert_eq!(publisher.task(&r.key()).unwrap().phase(), Phase::Queued);
    drop(publisher);
    let mut publisher = Publisher::open(&dir.0, store_binding(), verifier(&f), BridgeStateCodec).unwrap();
    assert_eq!(publisher.prepare_attempt(&r.key(), TickUs(14_000_001)).unwrap().stable_id, submission.stable_id);
    let rr = r.result();
    let bs = blocks(SOURCE, vec![vec![body(&rr.chain[0])], vec![proof(rr, rr.final_proof.clone())],
        vec![body(&rr.block)]], 14_000_000);
    let mut read = reader(SOURCE, READER, &bs, &[0, 1]);
    assert!(matches!(publisher.confirm_recorded(&r.key(), &read.require_prefix_through(1).unwrap(), &inputs).unwrap(),
        ConfirmOutcome::NotYetPublic));
    read.receive_block(SOURCE, bs[2].clone(), TickUs(50_000_002)).unwrap();
    let ConfirmOutcome::Recorded { receipt, .. } = publisher.confirm_recorded(&r.key(),
        &read.require_prefix_through(2).unwrap(), &inputs).unwrap() else { panic!("receipt missing") };
    assert_eq!(receipt.publication_position.q, 1); assert_eq!(receipt.complete_through_q, 2);
    assert_eq!(publisher.task(&r.key()).unwrap().phase(), Phase::Recorded);
    assert!(matches!(publisher.confirm_recorded(&r.key(), &read.require_prefix_through(2).unwrap(), &inputs).unwrap(),
        ConfirmOutcome::AlreadyRecorded));
}

#[test]
fn wrong_source_or_reader_never_records() {
    let f = Fixture::new(); let p = ExactProofs::default(); let b = Basis::default(); let l = l1(&f);
    let context = Context { f: &f, mode: Cell::new(0) };
    let inputs = BridgeInputs { finality: Some(binding(&p, &b)), context: &context, l1: &l, dep: f.setup.dep() };
    let r = result(&f, &p, 0, &[3], false); let dir = OwnDir::new();
    let mut publisher = Publisher::create(&dir.0, store_binding(), verifier(&f), BridgeStateCodec).unwrap();
    publisher.offer(r.clone(), &inputs).unwrap();
    for (source, id) in [(b"foreign-source".as_slice(), READER), (SOURCE, b"foreign-reader".as_slice())] {
        let bs = blocks(source, vec![publication_messages(&r, source).unwrap()], 14_000_000);
        let read = reader(source, id, &bs, &[0]);
        assert!(matches!(publisher.confirm_recorded(&r.key(), &read.require_prefix_through(0).unwrap(), &inputs),
            Err(PublisherError::PrefixIdentityMismatch)));
        assert_eq!(publisher.task(&r.key()).unwrap().phase(), Phase::Queued);
    }
}

#[test]
fn recorded_before_selection_and_g_unchanged() {
    let f = Fixture::new(); let p = ExactProofs::default(); let b = Basis::default(); let l = l1(&f);
    let r = result(&f, &p, 1, &[11], false); // successor candidate; initial GFR NEXT is r0.
    let bs = blocks(SOURCE, vec![publication_messages(&r, SOURCE).unwrap()], 22_000_000);
    let read = reader(SOURCE, READER, &bs, &[0]);
    let metadata = RegionalMetadata::new(f.config(), &l).unwrap();
    let reads = ReplayReads { espresso: &read, l1: &l };
    let services = ReplayServices { config: f.config(), plan: &f.plan, metadata: &metadata,
        dep: f.setup.dep(), finality: Some(binding(&p, &b)) };
    let initialized = init(&InitInput::Genesis { config: f.config().clone(), genesis: f.genesis().clone(),
        parameters: parameters(), expected_verification: profile() }, &reads, &services).unwrap();
    assert!(matches!(initialized.status, ReplayStatus::Ready));
    let before = initialized.state.unwrap();
    let pos = ReplayPosition { key_us: 22_000_000, kind: ReplayKind::Espresso, index: ReplayIndex::Espresso(0) };
    let after = advance_global(&before, &reads, &services, Some(&pos));
    assert!(matches!(after.status, ReplayStatus::Ready));
    assert!(after.state.public_state().entries.is_empty());
    let context = Context { f: &f, mode: Cell::new(0) };
    let inputs = BridgeInputs { finality: Some(binding(&p, &b)), context: &context, l1: &l, dep: f.setup.dep() };
    let dir = OwnDir::new(); let mut publisher = Publisher::create(&dir.0, store_binding(), verifier(&f), BridgeStateCodec).unwrap();
    assert_eq!(publisher.offer(r.clone(), &inputs).unwrap().phase, Phase::Queued);
    assert!(matches!(publisher.confirm_recorded(&r.key(), &read.require_prefix_through(0).unwrap(), &inputs).unwrap(),
        ConfirmOutcome::Recorded { .. }));
    assert!(after.state.public_state().entries.is_empty());
}

#[test]
fn production_fixture_profile_rejected_and_context_failure_preserved() {
    let f = Fixture::new();
    assert!(NativePublicationVerifier::new(f.config().clone(), VerificationProfile::NonCryptographicFixture {
        id: "explicit-noncrypto".into() }, parameters(), SOURCE.to_vec()).is_err());
    let p = ExactProofs::default(); let b = Basis::default(); let l = l1(&f);
    let context = Context { f: &f, mode: Cell::new(1) }; let r = result(&f, &p, 0, &[3], false);
    let inputs = BridgeInputs { finality: Some(binding(&p, &b)), context: &context, l1: &l, dep: f.setup.dep() };
    let dir = OwnDir::new(); let mut publisher = Publisher::create(&dir.0, store_binding(), verifier(&f), BridgeStateCodec).unwrap();
    assert_eq!(publisher.offer(r.clone(), &inputs).unwrap().phase, Phase::Verifying);
    for mode in [2, 3] { context.mode.set(mode); assert!(publisher.offer(r.clone(), &inputs).is_err());
        assert_eq!(publisher.task(&r.key()).unwrap().phase(), Phase::Verifying); }
    context.mode.set(0); assert_eq!(publisher.offer(r, &inputs).unwrap().phase, Phase::Queued);
}

#[test]
fn offer_waits_for_actual_l1_inputs_then_uses_merge() {
    let f = Fixture::new(); let p = ExactProofs::default(); let b = Basis::default();
    let full_l1 = l1(&f);
    let missing_l1 = FixtureL1Reader::new(b"h3-independent-A-l1".to_vec(), f.setup.l1().reader()).unwrap();
    let context = Context { f: &f, mode: Cell::new(0) }; let r = result(&f, &p, 0, &[3], true);
    let missing = BridgeInputs { finality: Some(binding(&p, &b)), context: &context,
        l1: &missing_l1, dep: f.setup.dep() };
    let complete = BridgeInputs { finality: Some(binding(&p, &b)), context: &context,
        l1: &full_l1, dep: f.setup.dep() };
    let dir = OwnDir::new(); let mut publisher = Publisher::create(&dir.0, store_binding(), verifier(&f), BridgeStateCodec).unwrap();
    assert_eq!(publisher.offer(r.clone(), &missing).unwrap().phase, Phase::Verifying);
    assert!(publisher.prepare_attempt(&r.key(), TickUs(14_000_000)).is_err());
    assert_eq!(publisher.offer(r, &complete).unwrap().phase, Phase::Queued);
}

#[test]
fn valid_synthetic_proof_does_not_bypass_bad_execution_root() {
    let f = Fixture::new(); let p = ExactProofs::default(); let b = Basis::default(); let l = l1(&f);
    let context = Context { f: &f, mode: Cell::new(0) };
    let inputs = BridgeInputs { finality: Some(binding(&p, &b)), context: &context, l1: &l, dep: f.setup.dep() };
    let r = result(&f, &p, 0, &[3], false);
    let mut invalid = r.result().clone(); invalid.block.pre_state_root = Hash32::new([0x77; 32]);
    invalid.chain = vec![invalid.block.clone()];
    invalid.final_proof = p.register(&invalid.descriptor, &invalid.block);
    let invalid = BridgeResult::new(invalid).unwrap();
    let dir = OwnDir::new(); let mut publisher = Publisher::create(&dir.0, store_binding(), verifier(&f), BridgeStateCodec).unwrap();
    assert!(matches!(publisher.offer(invalid, &inputs), Err(PublisherError::Rejected(_))));
}

#[test]
fn bounds_rerender_and_tail_contradiction_are_not_silently_normalized() {
    let f = Fixture::new(); let p = ExactProofs::default(); let r = result(&f, &p, 0, &[3], false);
    let mut n4 = r.result().clone();
    n4.block.transactions = (0..4).map(|n| Transfer { from: 0, to: 1, nonce: n, amount: 1 }.encode_raw()).collect();
    n4.chain = vec![n4.block.clone()]; let good = BridgeResult::new(n4.clone()).unwrap();
    assert_eq!(publication_messages(&good, SOURCE).unwrap()[0].bytes.len(), 197);
    assert_eq!(BridgeResult::decode_store(&good.encode_store()).unwrap().encode_store(), good.encode_store());
    n4.block.transactions.push(Transfer { from: 0, to: 1, nonce: 4, amount: 1 }.encode_raw());
    n4.chain = vec![n4.block.clone()]; assert!(BridgeResult::new(n4).is_err());
    let mut duplicate = r.result().clone(); duplicate.chain.push(duplicate.block.clone());
    if let Ok(d) = BridgeResult::new(duplicate) { assert!(publication_messages(&d, SOURCE).is_err()); }
    let mut bad_tail = r.result().clone(); bad_tail.chain[0].metadata.height = 2;
    if let Ok(d) = BridgeResult::new(bad_tail) { assert!(publication_messages(&d, SOURCE).is_err()); }
    let mut over = r.result().clone(); over.final_proof = vec![1; 32769]; assert!(BridgeResult::new(over).is_err());
}

#[test]
fn independent_private_wire_literals_and_negatives() {
    let all: serde_json::Value = serde_json::from_str(include_str!("../../wire-vectors-v1.json")).unwrap();
    for v in all["vectors"].as_array().unwrap() {
        let id = v["id"].as_str().unwrap();
        if matches!(id, "SW02-partial-store" | "SW03-structural-chain-store") {
            let bytes = unhex(v["expected_hex"].as_str().unwrap());
            let r = BridgeResult::decode_store(&bytes).unwrap(); assert_eq!(r.encode_store(), bytes);
        } else if id.starts_with("SW06") || id.starts_with("SW07") || id.starts_with("SW08")
            || id.starts_with("SW09") || id.starts_with("SW10") {
            let bytes = unhex(v["hex"].as_str().unwrap());
            assert!(BridgeResult::decode_store(&bytes).is_err(), "accepted {id}");
        }
    }
}
