//! Independent replay fixtures. The reader holds synthetic verified cuts and
//! the shared fixture uses exact-subject structural Final witnesses.
#![allow(dead_code)] // Reused fixtures also provide M1–M3-only controls.

#[path = "../../../viewer-gfr/independent-harness/src/m2_fixture.rs"]
mod fixture;
#[path = "../../../viewer-gfr/independent-harness/src/m2_world.rs"]
mod world;

use fixture::Fixture;
use poc_core::{
    fixture::{DepProvider, reg_a_l1_fixture},
    merge::MergedBlock,
    metadata::RegionalMetadata,
};
use poc_types::{domain::*, hash_parts};
use poc_viewer_gfr::*;
use std::{cell::Cell, collections::BTreeMap};
use world::{ControlFixture, World, ep, expiry, profile};

const SEC: u64 = 1_000_000;
const POLICY: [u8; 4] = [7, 2, 0, 1];
const SIGNED_ORIGIN: u64 = 0; // Evidence is signed independently from actual D origin.
const SIGNED_FLOOR: u64 = 99; // The actual floor comes from the cut anchor.

fn hash(n: u8) -> Hash32 {
    Hash32::new([n; 32])
}

struct CutReader<'a> {
    l1: &'a ControlFixture,
    cuts: &'a BTreeMap<L1Position, ResumeBasis>,
}

impl FinalizedL1Reader for CutReader<'_> {
    fn header(&self, height: u64) -> InputStatus<L1Header> {
        self.l1.header(height)
    }

    fn require_control_coverage(&self, height: u64) -> InputStatus<()> {
        self.l1.require_control_coverage(height)
    }

    fn finalized_tip(&self) -> Option<L1BlockRef> {
        self.l1.finalized_tip()
    }
}

impl L1ControlReader for CutReader<'_> {
    fn source_id(&self) -> &[u8] {
        self.l1.source_id()
    }

    fn controls_at(&self, height: u64) -> InputStatus<Vec<ControlRef>> {
        self.l1.controls_at(height)
    }

    fn resume_cut(&self, position: L1Position) -> InputStatus<ResumeBasis> {
        match self.cuts.get(&position) {
            Some(cut) => InputStatus::Ready(cut.clone()),
            None => InputStatus::Wait(InputMissing::ControlCoverage(position.block)),
        }
    }
}

struct PlanInputs<'a> {
    fixture: &'a Fixture,
    wait: &'a Cell<bool>,
    calls: &'a Cell<usize>,
}

impl PlanProvider for PlanInputs<'_> {
    fn resolve(&self, input: &PlanInput, refs: &dyn PublicReferenceView) -> PlanOutcome {
        self.fixture.plan.resolve(input, refs)
    }

    fn resolve_historical(
        &self,
        input: &PlanInput,
        refs: &dyn PublicReferenceView,
        basis: &HistoricalPlan,
    ) -> PlanOutcome {
        self.calls.set(self.calls.get() + 1);
        if self.wait.get() {
            return PlanOutcome::Wait(InputMissing::Reference(input.reference_block.clone()));
        }
        self.fixture.plan.resolve_historical(input, refs, basis)
    }

    fn descriptor(&self, plan: &PlanRecord, anchor: &Anchor) -> Result<Descriptor, PlanError> {
        self.fixture.plan.descriptor(plan, anchor)
    }
}

struct DepInputs<'a> {
    source: &'a dyn DepProvider,
    missing_height: Option<u64>,
}

impl DepProvider for DepInputs<'_> {
    fn config_ref(&self) -> ConfigRef {
        self.source.config_ref()
    }

    fn at(&self, metadata: &Metadata) -> InputStatus<Vec<Vec<u8>>> {
        if Some(metadata.height) == self.missing_height {
            return InputStatus::Wait(InputMissing::Dep(metadata.height));
        }
        self.source.at(metadata)
    }
}

struct Replay<'a> {
    world: World<'a>,
    cuts: BTreeMap<L1Position, ResumeBasis>,
    plan_wait: Cell<bool>,
    plan_reads: Cell<usize>,
    dep_wait: Cell<Option<u64>>,
}

impl<'a> Replay<'a> {
    fn new(f: &'a Fixture, name: &[u8]) -> Self {
        let mut world = World::new(f, name);
        world.params = GfrParameters {
            announcement_us: 10 * SEC,
            grace_us: 3 * SEC,
            expiry_us: 8 * SEC,
        };
        Self {
            world,
            cuts: BTreeMap::new(),
            plan_wait: Cell::new(false),
            plan_reads: Cell::new(0),
            dep_wait: Cell::new(None),
        }
    }

    fn reader(&self) -> CutReader<'_> {
        CutReader {
            l1: &self.world.l1,
            cuts: &self.cuts,
        }
    }

    fn plan_inputs(&self) -> PlanInputs<'_> {
        PlanInputs {
            fixture: self.world.f,
            wait: &self.plan_wait,
            calls: &self.plan_reads,
        }
    }

    fn with_io<T>(&self, call: impl FnOnce(ReplayReads<'_>, ReplayServices<'_>) -> T) -> T {
        let reader = self.reader();
        let plan = self.plan_inputs();
        let f = self.world.f;
        let metadata = RegionalMetadata::new(f.config(), &reader).unwrap();
        let dep = DepInputs {
            source: f.setup.dep(),
            missing_height: self.dep_wait.get(),
        };
        let verified = VerifierBinding {
            verifier: &self.world.proofs,
            basis: &self.world.basis,
            profile: profile(),
        };
        call(
            ReplayReads {
                espresso: &self.world.e,
                l1: &reader,
            },
            ReplayServices {
                config: f.config(),
                plan: &plan,
                metadata: &metadata,
                dep: &dep,
                finality: Some(verified),
            },
        )
    }

    fn init(&self, input: &InitInput) -> Result<InitResult, GfrError> {
        self.with_io(|reads, services| init(input, &reads, &services))
    }

    fn advance(&self, state: &ReplayState, through: &ReplayPosition) -> ReplayResult {
        self.with_io(|reads, services| advance_global(state, &reads, &services, Some(through)))
    }

    fn add_control(&mut self, control: &Resume) {
        self.world
            .l1
            .controls
            .entry(control.position.block)
            .or_default()
            .push(control.control_ref());
    }

    fn add_cut(&mut self, control: &Resume, derived: &[MergedBlock]) {
        let basis = control.cut(self.world.f, derived);
        assert!(self.cuts.insert(control.position, basis).is_none());
    }

    fn heartbeat(&mut self, index: u64, seconds: u64) -> ReplayPosition {
        self.world
            .deliver(&World::block(index, seconds * SEC, vec![]));
        ep(seconds * SEC, index)
    }

    fn old_g(&mut self, tip: u64) -> ReplayState {
        let f = self.world.f;
        let genesis = f.genesis();
        let desc = f.descriptor(0, &genesis.anchor, &genesis.anchor);
        let bodies = f.physical_chain(genesis, &(1..=tip).collect::<Vec<_>>(), None);
        let contents = self.world.result_messages(&desc, &bodies);
        let time = (10 + tip) * SEC;
        self.world.deliver(&World::block(0, time, contents));
        let state = self
            .init(&self.world.genesis_input())
            .unwrap()
            .state
            .unwrap();
        let result = self.advance(&state, &ep(time, 0));
        assert_eq!(result.status, ReplayStatus::Ready);
        assert_eq!(result.state.public_state().entries.len(), tip as usize);
        for (item, body) in result.state.public_state().entries.iter().zip(&bodies) {
            assert_eq!(
                item.block,
                f.expected_entry(body.metadata.height, Some(body.digest().unwrap()), None)
            );
        }
        result.state
    }
}

#[derive(Clone, Copy)]
struct Resume {
    position: L1Position,
    seq: u64,
    target: u64,
}

impl Resume {
    fn new(block: u64, tx: u64, seq: u64, target: u64) -> Self {
        Self {
            position: L1Position { block, tx, log: 1 },
            seq,
            target,
        }
    }

    fn set_position(&self) -> L1Position {
        L1Position {
            log: 0,
            ..self.position
        }
    }

    fn transaction_hash(&self) -> Hash32 {
        hash((100 + self.seq).try_into().unwrap())
    }

    fn control_ref(&self) -> ControlRef {
        // Encode the contract's authenticated projection independently.
        let mut source_bytes = b"PoC/PolicyResume/v1".to_vec();
        source_bytes.extend_from_slice(&self.target.to_be_bytes());
        source_bytes.extend_from_slice(&SIGNED_ORIGIN.to_be_bytes());
        source_bytes.extend_from_slice(&POLICY);
        source_bytes.extend_from_slice(&SIGNED_FLOOR.to_be_bytes());
        source_bytes.extend_from_slice(&self.set_position().log.to_be_bytes());
        source_bytes.extend_from_slice(self.transaction_hash().as_bytes());
        ControlRef {
            position: self.position,
            kind: ReplayKind::Resume,
            control_seq: self.seq,
            source_bytes,
        }
    }

    fn replay_position(&self) -> ReplayPosition {
        ReplayPosition {
            key_us: (self.position.block * 2 + 10) * SEC,
            kind: ReplayKind::Resume,
            index: ReplayIndex::L1(self.position),
        }
    }

    fn cut(&self, f: &Fixture, historical: &[MergedBlock]) -> ResumeBasis {
        let target = usize::try_from(self.target).unwrap();
        let digest = hash_parts(
            "fixture/independent-positioned-cut",
            &[
                &self.position.block.to_be_bytes(),
                &self.position.tx.to_be_bytes(),
                &self.position.log.to_be_bytes(),
                &self.target.to_be_bytes(),
                &serde_json::to_vec(&historical.iter().map(|b| b.native_hash).collect::<Vec<_>>())
                    .unwrap(),
            ],
        );
        let include = f.tokens[self.position.block as usize].header();
        ResumeBasis {
            position: self.position,
            set_position: self.set_position(),
            config_ref: f.config().reference().unwrap(),
            including_block: L1BlockRef {
                height: include.height,
                hash: include.hash,
            },
            transaction_hash: self.transaction_hash(),
            digest: Hash32::new(digest.try_into().unwrap()),
            genesis: f.genesis().clone(),
            target: derived_anchor(&historical[target - 1]),
            blocks: historical[..target].to_vec(),
        }
    }
}

fn derived_anchor(item: &MergedBlock) -> ExecutionAnchor {
    let metadata = &item.state.metadata;
    let height = metadata.height;
    let mut preimage = 1u16.to_be_bytes().to_vec();
    preimage.extend_from_slice(&height.to_be_bytes());
    preimage.extend_from_slice(item.native_hash.as_bytes());
    let digest = hash_parts("PoC/DerivedAnchor/v1", &[&preimage]);
    ExecutionAnchor {
        anchor: Anchor {
            block: BlockRef {
                height,
                origin: metadata.origin,
                block_id: Hash32::new(digest.try_into().unwrap()),
            },
            timestamp_us: metadata.timestamp_us,
        },
        state: item.state.clone(),
        native_hash: item.native_hash,
    }
}

fn derived_blocks(f: &Fixture, origins: &[u64]) -> Vec<MergedBlock> {
    // The accepted fixture's independent oracle computes deposits/accounts.
    // Override only origin/attributes and recompute native linkage explicitly.
    let mut parent = f.genesis().native_hash;
    let mut input = f.genesis().state.clone();
    let mut blocks = vec![];
    for (index, origin) in origins.iter().enumerate() {
        let height = u64::try_from(index).unwrap() + 1;
        let mut state = Fixture::expected_state(height, None);
        state.metadata.origin = *origin;
        let mut dep = Fixture::dep(height);
        dep[0] = Attributes {
            metadata: state.metadata.clone(),
        }
        .encode_raw();
        let native_block = NativeBlock {
            parent,
            metadata: state.metadata.clone(),
            dep: dep.clone(),
            kept: vec![],
            post_state_root: state.root().unwrap(),
        };
        let native_hash = native_block.digest().unwrap();
        blocks.push(MergedBlock {
            virtual_block: VirtualBlock {
                metadata: state.metadata.clone(),
                dep,
                pre_state_root: input.root().unwrap(),
                transactions: vec![],
            },
            physical_digest: None,
            state: state.clone(),
            native_block,
            native_hash,
        });
        parent = native_hash;
        input = state;
    }
    blocks
}

fn record_positions(result: &ReplayResult) -> Vec<ReplayPosition> {
    assert_eq!(result.status, ReplayStatus::Ready);
    result
        .added_records
        .iter()
        .map(|r| r.position.clone())
        .collect()
}

fn assert_uncommitted(result: &ReplayResult, prior: &ReplayState) {
    assert!(matches!(result.status, ReplayStatus::Wait { .. }));
    assert_eq!(result.state, *prior);
    assert_eq!(result.state.checkpoint(), prior.checkpoint());
    assert!(result.events.is_empty());
    assert!(result.added_records.is_empty());
}

fn assert_exact_wait(result: &ReplayResult, prior: &ReplayState, p: &ReplayPosition) {
    assert_uncommitted(result, prior);
    let ReplayStatus::Wait {
        next_position,
        missing,
    } = &result.status
    else {
        unreachable!()
    };
    assert_eq!(next_position.as_ref(), Some(p));
    assert!(!missing.is_empty());
    println!("WAIT {p:?}: {missing:?}");
}

fn assigned(
    result: &ReplayResult,
    control: &Resume,
    branch: u64,
    target: &ExecutionAnchor,
) -> Descriptor {
    assert_eq!(result.status, ReplayStatus::Ready);
    let [closed, resumed, selected] = result.events.as_slice() else {
        panic!(
            "expected one ordered RESUME triple, got {:?}",
            result.events
        );
    };
    assert!(
        result
            .events
            .iter()
            .all(|e| e.position == control.replay_position())
    );
    assert_eq!(
        closed.body,
        LifecycleBody::BranchClosed {
            branch: branch - 1,
            reason: CloseReason::Resume
        }
    );
    let expected_prefix = result.state.public_state().resumes[&control.position]
        .blocks
        .iter()
        .map(|b| b.virtual_block.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        resumed.body,
        LifecycleBody::Resumed {
            control_seq: control.seq,
            branch,
            anchor: target.anchor.clone(),
            derived_prefix: expected_prefix,
        }
    );
    let LifecycleBody::SessionAssigned { descriptor } = &selected.body else {
        panic!("missing successor at {:?}", selected.position);
    };
    assert_resume_descriptor(descriptor, branch, target);
    assert_eq!(
        result.state.public_state().l1_floor,
        target.anchor.block.origin
    );
    assert_eq!(result.state.public_state().branch, branch);
    assert!(matches!(&result.state.public_state().tail, Tail::Next(d) if d == descriptor));
    let actual = result.state.public_state();
    assert_eq!(
        actual.plan_bases[&descriptor.cid].height,
        target.anchor.block.origin
    );
    assert_eq!(
        actual.plan_bases[&descriptor.cid].active.force,
        control.position
    );
    let historical = &actual.plan_bases[&descriptor.cid];
    let expected_header = &reg_a_l1_fixture().rows[target.anchor.block.origin as usize].header;
    assert_eq!(historical.block_hash, expected_header.hash);
    let h_position = &control.position;
    let mut plan = 1u16.to_be_bytes().to_vec();
    plan.extend_from_slice(actual.config.reference().unwrap().as_bytes());
    plan.extend_from_slice(actual.config.registry_hash.as_bytes());
    plan.push(3); // RESUME boundary
    plan.extend_from_slice(&branch.to_be_bytes());
    plan.extend_from_slice(&0u64.to_be_bytes()); // r
    plan.push(0); // no source region
    plan.extend_from_slice(&descriptor.plan.window.h_min.to_be_bytes());
    plan.extend_from_slice(&descriptor.plan.window.h_max.to_be_bytes());
    let reference = &target.anchor.block;
    plan.extend_from_slice(&reference.height.to_be_bytes());
    plan.extend_from_slice(&reference.origin.to_be_bytes());
    plan.extend_from_slice(reference.block_id.as_bytes());
    let source_bound = hash_parts("PoC/Authorization/v1", &[&plan]);
    let policy_digest = hash_parts(
        "PoC/HistoricalPlan/v1",
        &[
            &source_bound,
            &h_position.block.to_be_bytes(),
            &u32::try_from(h_position.tx).unwrap().to_be_bytes(),
            &u32::try_from(h_position.log).unwrap().to_be_bytes(),
            &control.seq.to_be_bytes(),
            &POLICY,
            &SIGNED_FLOOR.to_be_bytes(),
            &reference.origin.to_be_bytes(),
            &reference.origin.to_be_bytes(),
            expected_header.hash.as_bytes(),
            &[2],
        ],
    );
    assert_eq!(
        descriptor.plan.authorization_ref.as_bytes(),
        policy_digest.as_slice()
    );
    assert!(
        result.added_records[0]
            .effects
            .iter()
            .any(|effect| matches!(
                effect,
                ReplayEffect::Reset { resume_position, target_height, cut_digest }
                    if *resume_position == control.position && *target_height == reference.height
                        && *cut_digest == actual.resumes[&control.position].digest
            ))
    );
    descriptor.clone()
}

fn assert_resume_descriptor(descriptor: &Descriptor, branch: u64, target: &ExecutionAnchor) {
    let h = target.anchor.block.height;
    assert_eq!(descriptor.plan.session, SessionId { beta: branch, r: 0 });
    assert_eq!(descriptor.plan.cause, BoundaryCause::Resume);
    assert_eq!(descriptor.plan.source_region, None);
    assert_eq!(descriptor.plan.region.get(), 2);
    assert_eq!(
        descriptor.plan.window,
        Window {
            h_min: h + 1,
            h_max: h + 10
        }
    );
    assert_eq!(descriptor.plan.reference_block, target.anchor.block);
    assert_eq!(descriptor.anchor, target.anchor);
}

#[test]
fn missing_exact_cut_and_plan_wait_without_partial_reset_then_ab_agree() {
    let f = Fixture::new();
    let mut a = Replay::new(&f, b"m4-a");
    let mut b = Replay::new(&f, b"m4-b");
    let p = Resume::new(5, 0, 1, 1);
    let at = p.replay_position();
    let d = derived_blocks(&f, &[3, 4, 5]);
    a.world.clear_l1();
    // Headers (including h1 origin) arrive before the control receipt coverage.
    a.world.deliver_l1(5, 1);
    a.add_control(&p);
    b.add_control(&p);
    let old_a = a.old_g(1);
    let old_b = b.old_g(1);
    a.heartbeat(1, 20);
    let without_l1 = a.advance(&old_a, &at);
    assert_uncommitted(&without_l1, &old_a);
    a.world.deliver_l1(5, 5);
    let without_cut = a.advance(&old_a, &at);
    assert_exact_wait(&without_cut, &old_a, &at);
    assert!(
        matches!(without_cut.status, ReplayStatus::Wait { missing, .. }
        if missing.contains(&Missing::ResumeCut { position: p.position }))
    );

    // A cut for a different transaction in the same L1 block is not a match.
    let wrong_position = Resume::new(5, 1, 2, 1);
    a.add_cut(&wrong_position, &d);
    assert_exact_wait(&a.advance(&old_a, &at), &old_a, &at);
    a.add_cut(&p, &d);
    a.plan_wait.set(true);
    let plan_wait = a.advance(&old_a, &at);
    assert_exact_wait(&plan_wait, &old_a, &at);
    let ReplayStatus::Wait { missing, .. } = &plan_wait.status else {
        unreachable!()
    };
    assert!(
        missing
            .iter()
            .any(|m| matches!(m, Missing::Input(InputMissing::Reference(_))))
    );
    a.plan_wait.set(false);
    let ready_a = a.advance(&old_a, &at);

    // Independently deliver cut and L1 before the Espresso watermark to B.
    b.add_cut(&p, &d);
    assert_uncommitted(&b.advance(&old_b, &at), &old_b);
    b.heartbeat(1, 20);
    let ready_b = b.advance(&old_b, &at);
    assert_eq!(record_positions(&ready_a), [at.clone()]);
    assert_eq!(record_positions(&ready_b), [at]);
    assert_eq!(ready_a.state, ready_b.state);
    assert_eq!(ready_a.events, ready_b.events);
    let target = derived_anchor(&d[0]);
    assigned(&ready_a, &p, 1, &target);
    assert_eq!(
        ready_a.state.public_state().entries,
        old_a.public_state().entries
    );
    let old_physical = old_a.public_state().entries[0]
        .block
        .physical_digest
        .unwrap();
    assert_ne!(target.anchor.block.block_id, old_physical);
    assert_eq!(target.native_hash, d[0].native_hash);
    assert_eq!(d.len(), 3);
    println!(
        "A/B agree on G.h1 and derived continuation at {:?}; D snapshot has h3",
        p.position
    );
}

#[test]
fn retains_equal_native_prefix_and_only_replaces_divergent_suffix_through_target() {
    let f = Fixture::new();
    let mut replay = Replay::new(&f, b"m4-divergence");
    let p = Resume::new(5, 0, 1, 2);
    replay.add_control(&p);
    let old = replay.old_g(2);
    let d = derived_blocks(&f, &[3, 4, 5]);
    assert_eq!(
        old.public_state().entries[0].block.native_hash,
        d[0].native_hash
    );
    assert_ne!(
        old.public_state().entries[1].block.native_hash,
        d[1].native_hash
    );
    replay.add_cut(&p, &d);
    replay.heartbeat(1, 20);
    replay.dep_wait.set(Some(2));
    let missing_dep = replay.advance(&old, &p.replay_position());
    assert_exact_wait(&missing_dep, &old, &p.replay_position());
    assert!(
        matches!(missing_dep.status, ReplayStatus::Wait { missing, .. }
        if missing.contains(&Missing::Input(InputMissing::Dep(2))))
    );
    replay.dep_wait.set(None);
    let ready = replay.advance(&old, &p.replay_position());
    assert_eq!(record_positions(&ready), [p.replay_position()]);
    let target = derived_anchor(&d[1]);
    assigned(&ready, &p, 1, &target);
    let entries = &ready.state.public_state().entries;
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0], old.public_state().entries[0]);
    assert_eq!(entries[1].block, d[1]);
    assert_eq!(entries[1].position, p.replay_position());
    assert!(
        matches!(entries[1].source, GlobalSource::Derived { resume_position, height }
        if resume_position == p.position && height == 2)
    );
    println!(
        "retained native h1, replaced h2 from {:?}, D.h3 not adopted",
        p.position
    );
}

#[test]
fn all_same_key_resumes_run_before_dynamic_expire_and_reopen_with_historical_cuts() {
    let f = Fixture::new();
    let mut replay = Replay::new(&f, b"m4-same-key");
    let p1 = Resume::new(5, 0, 1, 1);
    let p2 = Resume::new(5, 1, 2, 3);
    replay.add_control(&p1);
    replay.add_control(&p2);
    let old = replay.old_g(1);
    let d = derived_blocks(&f, &[3, 4, 5]);
    replay.add_cut(&p1, &d);
    replay.add_cut(&p2, &d);
    let e1 = replay.heartbeat(1, 20);
    let e2 = replay.heartbeat(2, 20);

    let first = replay.advance(&old, &p1.replay_position());
    assert_eq!(record_positions(&first), [p1.replay_position()]);
    assigned(&first, &p1, 1, &derived_anchor(&d[0]));
    assert_eq!(
        first.state.public_state().entries,
        old.public_state().entries
    );
    let second = replay.advance(&first.state, &p2.replay_position());
    assert_eq!(record_positions(&second), [p2.replay_position()]);
    assigned(&second, &p2, 2, &derived_anchor(&d[2]));
    assert_eq!(second.state.public_state().entries.len(), 3);
    let combined = replay.advance(&old, &e2);
    assert_eq!(
        record_positions(&combined),
        [p1.replay_position(), p2.replay_position(), e1, e2]
    );
    assert_eq!(
        combined
            .state
            .public_state()
            .closed_branches
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert!(matches!(combined.state.public_state().tail, Tail::Next(_)));
    assert_eq!(replay.plan_reads.get(), 4);
    let checkpoint = combined.state.checkpoint();

    // Later public input neither rekeys nor replaces either historical cut.
    replay.heartbeat(3, 21);
    let reopened = replay
        .init(&InitInput::Checkpoint(checkpoint.clone()))
        .unwrap();
    assert_eq!(reopened.status, ReplayStatus::Ready);
    assert_eq!(reopened.state.unwrap().checkpoint(), checkpoint);
    let original = replay.cuts.remove(&p1.position).unwrap();
    let unready = replay
        .init(&InitInput::Checkpoint(checkpoint.clone()))
        .unwrap();
    assert!(matches!(unready.status, ReplayStatus::Wait { .. }));
    let mut conflicting = original.clone();
    conflicting.digest = hash(230);
    replay.cuts.insert(p1.position, conflicting);
    assert!(matches!(
        replay.init(&InitInput::Checkpoint(checkpoint.clone())),
        Err(GfrError::CheckpointMismatch)
            | Ok(InitResult {
                status: ReplayStatus::Reject(_),
                ..
            })
    ));
    replay.cuts.insert(p1.position, original);
    assert_eq!(
        replay
            .init(&InitInput::Checkpoint(checkpoint.clone()))
            .unwrap()
            .state
            .unwrap()
            .checkpoint(),
        checkpoint
    );
    println!(
        "checkpoint reopened after controls {:?}, {:?}, distinct historical targets 1 and 3",
        p1.position, p2.position
    );
}

#[test]
fn equal_expire_runs_after_only_resume_and_closes_the_new_branch() {
    let f = Fixture::new();
    let mut replay = Replay::new(&f, b"m4-equal-expire");
    let p = Resume::new(5, 0, 1, 1);
    replay.add_control(&p);
    let old = replay.old_g(1);
    let d = derived_blocks(&f, &[3]);
    replay.add_cut(&p, &d);
    let e = replay.heartbeat(1, 20);
    let at_control = replay.advance(&old, &p.replay_position());
    assigned(&at_control, &p, 1, &derived_anchor(&d[0]));
    let both = replay.advance(&old, &e);
    assert_eq!(
        record_positions(&both),
        [p.replay_position(), expiry(20 * SEC, 2), e]
    );
    assert!(matches!(both.state.public_state().tail, Tail::Halt { .. }));
    assert_eq!(
        both.state.public_state().entries,
        at_control.state.public_state().entries
    );
    assert!(both.added_records[1].events.iter().any(|event| matches!(
        event.body,
        LifecycleBody::BranchClosed {
            branch: 1,
            reason: CloseReason::Expired
        }
    )));
}

fn expect_align_then_halt(
    replay: &Replay<'_>,
    before: &ReplayState,
    control: &Resume,
    expected: &MergedBlock,
    expected_before_plan_calls: usize,
) {
    let p = control.replay_position();
    let ready = replay.advance(before, &p);
    assert_eq!(record_positions(&ready), [p.clone()]);
    assert_eq!(
        replay.plan_reads.get(),
        expected_before_plan_calls,
        "strict past timer must skip Plan"
    );
    let state = ready.state.public_state();
    assert_eq!(state.entries.len(), 1);
    assert_eq!(&state.entries[0].block, expected);
    assert!(matches!(&state.tail, Tail::Halt { .. }));
    let [closed, resumed, expired] = ready.events.as_slice() else {
        panic!(
            "expected close, material and expired, got {:?}",
            ready.events
        );
    };
    assert!(ready.events.iter().all(|e| e.position == p));
    assert_eq!(
        closed.body,
        LifecycleBody::BranchClosed {
            branch: 0,
            reason: CloseReason::Resume
        }
    );
    assert!(matches!(
        &resumed.body,
        LifecycleBody::Resumed { branch: 1, control_seq, anchor, .. }
            if *control_seq == control.seq && *anchor == derived_anchor(expected).anchor
    ));
    assert_eq!(
        expired.body,
        LifecycleBody::SessionClosed {
            session: SessionId { beta: 1, r: 0 },
            reason: CloseReason::Expired
        }
    );
}

#[test]
fn past_expire_still_adopts_d_and_resumes_from_old_halt() {
    let f = Fixture::new();
    let mut replay = Replay::new(&f, b"m4-past-expire");
    let p = Resume::new(6, 0, 1, 1);
    replay.add_control(&p);
    let old = replay.old_g(1);
    let expired_e = replay.heartbeat(1, 20);
    let expired = replay.advance(&old, &expired_e);
    assert_eq!(expired.status, ReplayStatus::Ready);
    assert!(matches!(
        expired.state.public_state().tail,
        Tail::Halt { .. }
    ));
    let d = derived_blocks(&f, &[4]);
    assert_ne!(
        expired.state.public_state().entries[0].block.native_hash,
        d[0].native_hash
    );
    replay.add_cut(&p, &d);
    replay.heartbeat(2, 22);
    expect_align_then_halt(&replay, &expired.state, &p, &d[0], 0);
}

#[test]
fn past_session_end_still_adopts_d_before_new_expiry_margin() {
    let f = Fixture::new();
    let mut replay = Replay::new(&f, b"m4-past-end");
    replay.world.params.expiry_us = 30 * SEC;
    let p = Resume::new(8, 0, 1, 1);
    replay.add_control(&p);
    let old = replay.old_g(1);
    // Old regular session ended at 24s. Distinguish that from the replacement's
    // own SessionEnd at 25s and from its next-height Expire at 42s.
    let e1 = replay.heartbeat(1, 24);
    let handed_off = replay.advance(&old, &e1);
    assert_eq!(handed_off.status, ReplayStatus::Ready);
    assert!(matches!(handed_off.state.public_state().tail,
        Tail::Next(ref d) if d.plan.session == (SessionId { beta: 0, r: 1 })));
    let d = derived_blocks(&f, &[4]);
    replay.add_cut(&p, &d);
    replay.heartbeat(2, 26);
    expect_align_then_halt(&replay, &handed_off.state, &p, &d[0], 0);
}

#[test]
fn same_key_force_reads_the_derived_anchor_even_when_g_retains_regional_h1() {
    let f = Fixture::new();
    let mut replay = Replay::new(&f, b"m4-effective-anchor");
    let resume = Resume::new(5, 0, 1, 1);
    replay.add_control(&resume);
    let force_position = L1Position {
        block: 5,
        tx: 1,
        log: 0,
    };
    let force = ControlRef::policy_force(
        force_position,
        2,
        1,
        RegionalPolicy::from_parts([7, 0, 1, 2], SIGNED_FLOOR).unwrap(),
    );
    replay.world.l1.controls.get_mut(&5).unwrap().push(force);
    let old = replay.old_g(1);
    let d = derived_blocks(&f, &[3]);
    replay.add_cut(&resume, &d);
    replay.heartbeat(1, 20);
    let at_force = ReplayPosition {
        key_us: 20 * SEC,
        kind: ReplayKind::Force,
        index: ReplayIndex::L1(force_position),
    };
    let applied = replay.advance(&old, &at_force);
    assert_eq!(
        record_positions(&applied),
        [resume.replay_position(), at_force]
    );
    assert_eq!(
        applied.state.public_state().entries,
        old.public_state().entries
    );
    let Tail::Next(assigned) = &applied.state.public_state().tail else {
        panic!("Force after RESUME does not assign");
    };
    assert_eq!(assigned.plan.cause, BoundaryCause::Force);
    assert_eq!(assigned.plan.session, SessionId { beta: 2, r: 0 });
    assert_eq!(assigned.plan.source_region.unwrap().get(), 2);
    assert_eq!(assigned.plan.region.get(), 0);
    assert_eq!(
        assigned.plan.window,
        Window {
            h_min: 14,
            h_max: 23
        }
    );
    assert_eq!(assigned.anchor, derived_anchor(&d[0]).anchor);
    assert_eq!(applied.state.public_state().l1_floor, 5);
}

#[test]
fn given_cut_with_a_contradictory_binding_rejects_atomically() {
    let f = Fixture::new();
    let mut replay = Replay::new(&f, b"m4-conflicting-cut");
    let p = Resume::new(5, 0, 1, 2);
    replay.add_control(&p);
    let old = replay.old_g(2);
    let d = derived_blocks(&f, &[3, 4]);
    replay.heartbeat(1, 20);
    let real = p.cut(&f, &d);
    for bad in [
        ResumeBasis {
            position: L1Position {
                tx: 1,
                ..p.position
            },
            ..real.clone()
        },
        ResumeBasis {
            set_position: p.position,
            ..real.clone()
        },
        ResumeBasis {
            transaction_hash: hash(0),
            ..real.clone()
        },
        ResumeBasis {
            including_block: L1BlockRef {
                height: 5,
                hash: hash(0),
            },
            ..real.clone()
        },
        ResumeBasis {
            target: derived_anchor(&d[0]),
            ..real.clone()
        },
        ResumeBasis {
            config_ref: hash(0).into(),
            ..real.clone()
        },
    ] {
        replay.cuts.insert(p.position, bad);
        let rejected = replay.advance(&old, &p.replay_position());
        assert!(
            matches!(rejected.status, ReplayStatus::Reject(_)),
            "{:?}",
            rejected.status
        );
        assert_eq!(rejected.state, old);
        assert!(rejected.added_records.is_empty() && rejected.events.is_empty());
    }
    replay.cuts.insert(p.position, real);
    let ready = replay.advance(&old, &p.replay_position());
    assigned(&ready, &p, 1, &derived_anchor(&d[1]));
}

mod manager {
    use super::*;
    use poc_session_manager::{
        ActionId, ActionIntent, ActionKind, ActionReceipt, CommitRef, Manager, ManagerBinding,
        ManagerDelta, ManagerError, ManagerInput, Phase, PlanOutcome as Decision, PreparedRef,
        ReceiptOutcome, TerminalCause, VerifiedCandidate, VerifiedGenesis, VerifiedReplayInput,
        WaitReason,
    };

    const NOW: u64 = 19 * SEC;

    fn start(f: &Fixture, worker: u8, initial: &ReplayState) -> Manager {
        let descriptor = f.descriptor(0, &f.genesis().anchor, &f.genesis().anchor);
        let mut manager = Manager::new(
            ManagerBinding {
                manager_id: hash(220 + worker),
                worker: Pid::new(worker).unwrap(),
                public_config: f.config().reference().unwrap(),
                gfr_source: hash(22),
                preparation_version: 1,
            },
            f.config().clone(),
            VerifiedGenesis {
                descriptor,
                source_ref: hash(23),
            },
        )
        .unwrap();
        commit(
            &mut manager,
            ManagerInput::ClockReady {
                generation: 1,
                basis_ref: hash(32),
            },
        );
        let [old] = initial.public_state().records.as_slice() else {
            panic!("old replay fixture had an unexpected boundary");
        };
        commit(
            &mut manager,
            ManagerInput::ConsumeRecord(replay_input(0, old.clone())),
        );
        manager
    }

    fn replay_input(seq: u64, record: ReplayRecord) -> VerifiedReplayInput {
        VerifiedReplayInput {
            source: hash(22),
            seq,
            record,
            validation_ref: hash(25),
        }
    }

    fn delta(manager: &Manager, input: ManagerInput) -> ManagerDelta {
        match manager.plan(input, NOW).unwrap() {
            Decision::Planned(delta) => delta,
            status => panic!("expected Manager delta, got {status:?}"),
        }
    }

    fn accept(manager: &mut Manager, planned: ManagerDelta) -> Vec<ActionIntent> {
        let intents = planned.intents().to_vec();
        let receipt = CommitRef::for_delta(&planned, hash(27));
        manager.accept_commit(planned, receipt).unwrap();
        intents
    }

    fn commit(manager: &mut Manager, input: ManagerInput) -> Vec<ActionIntent> {
        let staged = delta(manager, input);
        accept(manager, staged)
    }

    fn receipt(id: ActionId, outcome: ReceiptOutcome) -> ManagerInput {
        ManagerInput::Receipt(ActionReceipt {
            id,
            receipt_ref: hash(80),
            outcome,
        })
    }

    fn prepare(
        manager: &mut Manager,
        descriptor: &Descriptor,
        instance_ref: Hash32,
    ) -> PreparedRef {
        let cid = descriptor.cid;
        let create = manager.view().program(cid).unwrap().create_action.unwrap();
        let intents = commit(
            manager,
            receipt(create, ReceiptOutcome::Created { instance_ref }),
        );
        let [
            ActionIntent {
                id,
                kind:
                    ActionKind::Prepare {
                        cid: prepared_cid,
                        anchor,
                        provenance_ref,
                        validation_version,
                        ..
                    },
                ..
            },
        ] = intents.as_slice()
        else {
            panic!("expected one Prepare, got {intents:?}");
        };
        assert_eq!(*prepared_cid, cid);
        assert_eq!(anchor, &descriptor.anchor);
        PreparedRef {
            prepare_action: *id,
            worker: manager.view().binding().worker,
            cid,
            instance_ref,
            anchor: anchor.clone(),
            config_ref: manager.view().binding().public_config,
            validation_version: *validation_version,
            execution_ref: hash(70),
            provenance_ref: *provenance_ref,
        }
    }

    fn register(f: &Fixture, manager: &mut Manager, height: u64) -> Descriptor {
        let before = f.genesis();
        let old = f.descriptor(0, &before.anchor, &before.anchor);
        let body = f.physical_chain(before, &[height], None).remove(0);
        let anchor = f.execution_anchor(&body, None);
        let successor = f.descriptor(1, &anchor.anchor, &before.anchor);
        commit(
            manager,
            ManagerInput::RegisterCandidate(VerifiedCandidate {
                descriptor: successor.clone(),
                predecessor: old.cid,
                provenance_ref: hash(26),
            }),
        );
        successor
    }

    #[test]
    fn waits_for_resume_material_then_commits_all_terminal_guards_before_actions() {
        let f = Fixture::new();
        let mut replay = Replay::new(&f, b"m4-manager");
        let p = Resume::new(5, 0, 1, 2);
        replay.add_control(&p);
        let old_public = replay.old_g(2);
        let d = derived_blocks(&f, &[3, 4]);
        replay.add_cut(&p, &d);
        replay.heartbeat(1, 20);
        let restored = replay.advance(&old_public, &p.replay_position());
        let next = assigned(&restored, &p, 1, &derived_anchor(&d[1]));
        let input = replay_input(1, restored.added_records[0].clone());
        let verified = restored.state.public_state().resumes[&p.position].clone();

        // Keep old private candidate states distinct from the public h2 tip.
        let mut old_region = start(&f, 5, &old_public);
        let paused = register(&f, &mut old_region, 3);
        let instance_a = hash(40);
        let ready_a = prepare(&mut old_region, &paused, instance_a);
        commit(&mut old_region, ManagerInput::PreparationCompleted(ready_a));
        let enter = commit(
            &mut old_region,
            ManagerInput::Schedule(paused.plan.session.clone()),
        )[0]
        .id;
        commit(
            &mut old_region,
            receipt(
                enter,
                ReceiptOutcome::Entered {
                    instance_ref: instance_a,
                    entry_time_us: NOW,
                },
            ),
        );
        let active = register(&f, &mut old_region, 4);
        let instance_b = hash(41);
        let ready_b = prepare(&mut old_region, &active, instance_b);
        commit(&mut old_region, ManagerInput::PreparationCompleted(ready_b));
        let switch = commit(
            &mut old_region,
            ManagerInput::Schedule(active.plan.session.clone()),
        );
        assert_eq!(switch.len(), 2);
        commit(
            &mut old_region,
            receipt(
                switch[0].id,
                ReceiptOutcome::Paused {
                    instance_ref: instance_a,
                },
            ),
        );
        let late_enter = receipt(
            switch[1].id,
            ReceiptOutcome::Entered {
                instance_ref: instance_b,
                entry_time_us: NOW,
            },
        );
        commit(&mut old_region, late_enter.clone());
        let preparing = register(&f, &mut old_region, 5);
        let late_ready = prepare(&mut old_region, &preparing, hash(42));
        let previous = [
            (paused.cid, Phase::Paused),
            (active.cid, Phase::Running),
            (preparing.cid, Phase::Preparing),
        ];
        let before = old_region.view().revision();
        let cursor = old_region.view().cursor().cloned();
        let missing = ManagerInput::ConsumeResume {
            input: input.clone(),
            material: None,
        };
        assert!(matches!(
            old_region.plan(missing, NOW).unwrap(),
            Decision::Wait(WaitReason::ResumeMaterial { position }) if position == p.position
        ));
        assert_eq!(old_region.view().revision(), before);
        assert_eq!(old_region.view().cursor(), cursor.as_ref());

        let mut wrong = verified.clone();
        wrong.digest = hash(0);
        assert_eq!(
            old_region
                .plan(
                    ManagerInput::ConsumeResume {
                        input: input.clone(),
                        material: Some(wrong)
                    },
                    NOW
                )
                .unwrap_err(),
            ManagerError::InvalidRecord
        );
        let mut unordered = input.clone();
        unordered.record.events.swap(0, 1);
        assert_eq!(
            old_region
                .plan(
                    ManagerInput::ConsumeResume {
                        input: unordered,
                        material: Some(verified.clone())
                    },
                    NOW
                )
                .unwrap_err(),
            ManagerError::InvalidRecord
        );
        assert_eq!(old_region.view().revision(), before);
        assert_eq!(old_region.view().cursor(), cursor.as_ref());

        let actual = ManagerInput::ConsumeResume {
            input: input.clone(),
            material: Some(verified.clone()),
        };
        let staged = delta(&old_region, actual.clone());
        let closed = TerminalCause::Public(CloseReason::Resume);
        for (cid, phase) in previous {
            assert_eq!(old_region.view().program(cid).unwrap().phase, phase);
            assert_eq!(staged.guards().terminal(cid), Some(closed));
        }
        let closes = staged
            .intents()
            .iter()
            .filter(|a| {
                matches!(
                    a.kind, ActionKind::Close { cause, .. } if cause == closed
                )
            })
            .map(|a| a.id)
            .collect::<Vec<_>>();
        assert_eq!(closes.len(), 3);
        assert!(
            old_region
                .dispatchable(NOW)
                .iter()
                .all(|a| !closes.contains(&a.id))
        );
        accept(&mut old_region, staged);
        assert_eq!(old_region.view().cursor(), Some(&p.replay_position()));
        for (cid, _) in previous {
            assert_eq!(
                old_region.view().program(cid).unwrap().phase,
                Phase::Terminated
            );
            assert_eq!(old_region.view().guards().terminal(cid), Some(closed));
        }
        assert_eq!(
            old_region
                .dispatchable(NOW)
                .iter()
                .filter(|a| closes.contains(&a.id))
                .count(),
            3
        );
        assert!(matches!(
            old_region.plan(actual.clone(), NOW).unwrap(),
            Decision::Unchanged(_)
        ));
        assert!(matches!(
            old_region.plan(late_enter, NOW).unwrap(),
            Decision::Unchanged(_)
        ));
        match old_region
            .plan(ManagerInput::PreparationCompleted(late_ready), NOW)
            .unwrap()
        {
            Decision::Unchanged(_) => {}
            Decision::Planned(change) => assert!(change.intents().iter().all(|action| !matches!(
                action.kind,
                ActionKind::Enter { .. }
                    | ActionKind::Create { .. }
                    | ActionKind::SelectBack { .. }
            ))),
            unexpected => panic!("late readiness produced {unexpected:?}"),
        }

        // Hold a different replica's first Enter receipt across RESUME. This
        // is pending old work, independent of the acknowledged duplicate above.
        let mut outstanding = start(&f, 6, &old_public);
        let entering = register(&f, &mut outstanding, 3);
        let pending_instance = hash(45);
        let prepared = prepare(&mut outstanding, &entering, pending_instance);
        commit(
            &mut outstanding,
            ManagerInput::PreparationCompleted(prepared),
        );
        let in_flight = commit(
            &mut outstanding,
            ManagerInput::Schedule(entering.plan.session.clone()),
        )[0]
        .id;
        assert_eq!(
            outstanding.view().program(entering.cid).unwrap().phase,
            Phase::Entering
        );
        commit(&mut outstanding, actual.clone());
        let response = receipt(
            in_flight,
            ReceiptOutcome::Entered {
                instance_ref: pending_instance,
                entry_time_us: NOW,
            },
        );
        match outstanding.plan(response, NOW).unwrap() {
            Decision::Unchanged(_) => {}
            Decision::Planned(change) => {
                assert!(change.intents().iter().all(|a| !matches!(
                    a.kind,
                    ActionKind::Enter { .. } | ActionKind::SelectBack { .. }
                )));
                accept(&mut outstanding, change);
            }
            unexpected => panic!("late first Enter produced {unexpected:?}"),
        }
        assert_eq!(
            outstanding.view().program(entering.cid).unwrap().phase,
            Phase::Terminated
        );
        assert_eq!(
            outstanding.view().guards().terminal(entering.cid),
            Some(closed)
        );

        // The same authorized record grants a member of the selected region a
        // new Create only on commit; its Prepare points at the cut's anchor.
        let mut restored_region = start(&f, 10, &old_public);
        assert!(matches!(
            restored_region
                .plan(
                    ManagerInput::ConsumeResume {
                        input,
                        material: None
                    },
                    NOW
                )
                .unwrap(),
            Decision::Wait(WaitReason::ResumeMaterial { .. })
        ));
        let replacement = delta(&restored_region, actual);
        assert!(restored_region.view().program(next.cid).is_none());
        assert!(replacement.intents().iter().any(|a| matches!(
            &a.kind, ActionKind::Create { descriptor, .. } if descriptor == &next
        )));
        accept(&mut restored_region, replacement);
        assert_eq!(
            restored_region.view().selected(&next.plan.session),
            Some(&next)
        );
        assert!(
            restored_region
                .view()
                .program(next.cid)
                .unwrap()
                .predecessor
                .is_none()
        );
        let start = prepare(&mut restored_region, &next, hash(50));
        assert_eq!(start.anchor, verified.target.anchor);
        assert_eq!(start.provenance_ref, hash(25));
        println!(
            "Manager atomic commit retained verified Resume material and closed {} old cids",
            closes.len()
        );
    }
}

#[test]
fn later_regular_handoff_uses_the_original_resumed_floor_and_derived_anchor() {
    let f = Fixture::new();
    let mut replay = Replay::new(&f, b"m4-resumed-regular");
    replay.world.params.expiry_us = 30 * SEC;
    let resume = Resume::new(5, 0, 1, 1);
    replay.add_control(&resume);
    let old = replay.old_g(1);
    let d = derived_blocks(&f, &[3]);
    replay.add_cut(&resume, &d);
    let e1 = replay.heartbeat(1, 20);
    let e2 = replay.heartbeat(2, 25);
    let first = replay.advance(&old, &resume.replay_position());
    let target = derived_anchor(&d[0]);
    assigned(&first, &resume, 1, &target);
    let handed_off = replay.advance(&first.state, &e2);
    let end = ReplayPosition {
        key_us: 25 * SEC,
        kind: ReplayKind::SessionEnd,
        index: ReplayIndex::Session(SessionId { beta: 1, r: 0 }),
    };
    assert_eq!(record_positions(&handed_off), [e1, end, e2]);
    let state = handed_off.state.public_state();
    assert_eq!(state.l1_floor, 3);
    assert_eq!(state.entries, old.public_state().entries);
    let Tail::Next(successor) = &state.tail else {
        panic!("regular successor in resumed branch is absent");
    };
    assert_eq!(successor.plan.cause, BoundaryCause::Handoff);
    assert_eq!(successor.plan.session, SessionId { beta: 1, r: 1 });
    assert_eq!(successor.plan.source_region.unwrap().get(), 2);
    assert_eq!(successor.plan.region.get(), 0);
    assert_eq!(
        successor.plan.window,
        Window {
            h_min: 12,
            h_max: 21
        }
    );
    assert_eq!(successor.anchor, target.anchor);
    assert_eq!(state.plan_bases[&successor.cid].height, 3);
    assert_eq!(
        state.plan_bases[&successor.cid].active.force,
        resume.position
    );
}
