//! Source-derived vectors for the historical Plan at the production GFR boundary.
#![allow(dead_code)] // The shared independent fixtures also serve M1 and M2.

#[path = "../../../viewer-gfr/independent-harness/src/m2_fixture.rs"]
mod fixture;
#[path = "../../../viewer-gfr/independent-harness/src/m2_world.rs"]
mod world;

use fixture::Fixture;
use poc_core::metadata::RegionalMetadata;
use poc_types::{domain::*, hash_parts};
use poc_viewer_gfr::*;
use world::{World, ep, expiry, profile};

const SECOND: u64 = 1_000_000;
const POLICY_FORCE: &[u8] = b"PoC/PolicyForce/v1";
const DEFAULT_ORDER: [u8; 3] = [0, 1, 2];

#[derive(Clone, Copy)]
struct Policy {
    mask: u8,
    order: [u8; 3],
    signed_floor: u64,
}

fn policy(mask: u8, order: [u8; 3]) -> Policy {
    Policy {
        mask,
        order,
        signed_floor: 99,
    }
}

impl Policy {
    fn bits(self) -> [u8; 4] {
        [self.mask, self.order[0], self.order[1], self.order[2]]
    }

    fn control(self, position: (u64, u64, u64), seq: u64, target: u64) -> ControlRef {
        let (block, tx, log) = position;
        let mut source_bytes = POLICY_FORCE.to_vec();
        source_bytes.extend_from_slice(&target.to_be_bytes());
        source_bytes.extend_from_slice(&self.bits());
        source_bytes.extend_from_slice(&self.signed_floor.to_be_bytes());
        ControlRef {
            position: L1Position { block, tx, log },
            kind: ReplayKind::Force,
            control_seq: seq,
            source_bytes,
        }
    }
}

fn reader<'a>(f: &'a Fixture, id: &[u8]) -> World<'a> {
    let mut world = World::new(f, id);
    world.params = GfrParameters {
        announcement_us: 10 * SECOND,
        grace_us: 3 * SECOND,
        expiry_us: 30 * SECOND,
    };
    world
}

fn inputs<'a>(world: &'a World<'_>) -> ReplayReads<'a> {
    ReplayReads {
        espresso: &world.e,
        l1: &world.l1,
    }
}

fn services<'a>(world: &'a World<'_>, metadata: &'a RegionalMetadata<'a>) -> ReplayServices<'a> {
    ReplayServices {
        config: world.f.config(),
        // Exercise the production provider rather than the M2 test-only PlanGate.
        plan: &world.f.plan,
        metadata,
        dep: world.f.setup.dep(),
        finality: Some(VerifierBinding {
            verifier: &world.proofs,
            basis: &world.basis,
            profile: profile(),
        }),
    }
}

fn initialize(world: &World<'_>, input: &InitInput) -> Result<InitResult, GfrError> {
    let metadata = RegionalMetadata::new(world.f.config(), &world.l1).unwrap();
    init(input, &inputs(world), &services(world, &metadata))
}

fn genesis(world: &World<'_>) -> ReplayState {
    let result = initialize(world, &world.genesis_input()).unwrap();
    assert_eq!(result.status, ReplayStatus::Ready);
    result.state.unwrap()
}

fn advance(world: &World<'_>, before: &ReplayState, through: &ReplayPosition) -> ReplayResult {
    let metadata = RegionalMetadata::new(world.f.config(), &world.l1).unwrap();
    advance_global(
        before,
        &inputs(world),
        &services(world, &metadata),
        Some(through),
    )
}

fn at_force(control: &ControlRef) -> ReplayPosition {
    ReplayPosition {
        key_us: (control.position.block * 2 + 10) * SECOND,
        kind: ReplayKind::Force,
        index: ReplayIndex::L1(control.position),
    }
}

fn heartbeat(world: &mut World<'_>, q: u64, timestamp_us: u64) -> ReplayPosition {
    world.deliver(&World::block(q, timestamp_us, vec![]));
    ep(timestamp_us, q)
}

fn selected(state: &ReplayState) -> &Descriptor {
    let Tail::Next(descriptor) = &state.public_state().tail else {
        panic!(
            "expected a selected descriptor, got {:?}",
            state.public_state().tail
        );
    };
    descriptor
}

fn single_effect(result: &ReplayResult, position: &ReplayPosition) {
    assert_eq!(result.status, ReplayStatus::Ready);
    assert_eq!(result.added_records.len(), 1);
    assert_eq!(&result.added_records[0].position, position);
    assert_eq!(result.state.public_state().pc.as_ref(), Some(position));
}

fn expect_wait_for_history(
    result: &ReplayResult,
    before: &ReplayState,
    position: &ReplayPosition,
    height: u64,
) {
    let ReplayStatus::Wait {
        next_position,
        missing,
    } = &result.status
    else {
        panic!("expected historical WAIT, got {:?}", result.status);
    };
    assert_eq!(next_position.as_ref(), Some(position));
    assert!(
        missing.iter().any(|item| matches!(
            item,
            Missing::L1Header { height: missing_height }
            | Missing::Input(InputMissing::L1Header(missing_height))
                if *missing_height == height
        )),
        "expected historical header {height} in {missing:?}"
    );
    assert_eq!(result.state, *before);
    assert_eq!(result.state.checkpoint(), before.checkpoint());
    assert!(result.events.is_empty() && result.added_records.is_empty());
    println!("historical WAIT at {position:?}: {missing:?}");
}

fn put_u64(buffer: &mut Vec<u8>, value: u64) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

fn legacy_input_digest(config: &PublicConfig, input: &PlanInput) -> Vec<u8> {
    let mut preimage = 1u16.to_be_bytes().to_vec();
    preimage.extend_from_slice(config.reference().unwrap().as_bytes());
    preimage.extend_from_slice(config.registry_hash.as_bytes());
    preimage.push(match input.cause {
        BoundaryCause::Genesis => 0,
        BoundaryCause::Handoff => 1,
        BoundaryCause::Force => 2,
        BoundaryCause::Resume => 3,
    });
    put_u64(&mut preimage, input.session.beta);
    put_u64(&mut preimage, input.session.r);
    if let Some(region) = input.source_region {
        preimage.extend_from_slice(&[1, region.get()]);
    } else {
        preimage.push(0);
    }
    put_u64(&mut preimage, input.window.h_min);
    put_u64(&mut preimage, input.window.h_max);
    put_u64(&mut preimage, input.reference_block.height);
    put_u64(&mut preimage, input.reference_block.origin);
    preimage.extend_from_slice(input.reference_block.block_id.as_bytes());
    hash_parts("PoC/Authorization/v1", &[&preimage])
}

struct HistoricalBasis<'a> {
    policy: Policy,
    control: &'a ControlRef,
    effective_floor: u64,
    header: &'a L1Header,
}

fn historical_digest(
    config: &PublicConfig,
    input: &PlanInput,
    basis: &HistoricalBasis<'_>,
    region: u8,
) -> Hash32 {
    let legacy = legacy_input_digest(config, input);
    let L1Position { block, tx, log } = basis.control.position;
    let digest = hash_parts(
        "PoC/HistoricalPlan/v1",
        &[
            &legacy,
            &block.to_be_bytes(),
            &u32::try_from(tx).unwrap().to_be_bytes(),
            &u32::try_from(log).unwrap().to_be_bytes(),
            &basis.control.control_seq.to_be_bytes(),
            &basis.policy.bits(),
            &basis.policy.signed_floor.to_be_bytes(),
            &basis.effective_floor.to_be_bytes(),
            &basis.header.height.to_be_bytes(),
            basis.header.hash.as_bytes(),
            &[region],
        ],
    );
    Hash32::new(digest.try_into().unwrap())
}

struct ExpectedDuty<'a> {
    cause: BoundaryCause,
    session: (u64, u64),
    source: Option<u8>,
    region: u8,
    window: (u64, u64),
    reference: &'a BlockRef,
    anchor: &'a Anchor,
}

fn expected_descriptor(
    fixture: &Fixture,
    duty: &ExpectedDuty<'_>,
    basis: &HistoricalBasis<'_>,
) -> Descriptor {
    let config = fixture.config();
    let region = RegionId::new(duty.region).unwrap();
    let input = PlanInput {
        cause: duty.cause,
        session: SessionId {
            beta: duty.session.0,
            r: duty.session.1,
        },
        source_region: duty.source.map(|id| RegionId::new(id).unwrap()),
        window: Window {
            h_min: duty.window.0,
            h_max: duty.window.1,
        },
        reference_block: duty.reference.clone(),
    };
    let committee = fixture.plan.registry().committee(region);
    let authorization_ref = historical_digest(config, &input, basis, duty.region);
    let plan = PlanRecord {
        session: input.session.clone(),
        cause: input.cause,
        source_region: input.source_region,
        region,
        eligible_proposers: committee.iter().map(|entry| entry.pid).collect(),
        committee,
        total_weight: 5,
        fault_bound: 1,
        leader_rule: RuleSpec {
            rule_id: 2,
            version: 1,
            params: authorization_ref.as_bytes().to_vec(),
        },
        window: input.window,
        reference_block: input.reference_block,
        config_ref: config.reference().unwrap(),
        authorization_ref,
    };
    let program = ProgramConfig::from_plan(config, &plan).unwrap();
    let mut cid_input = 1u16.to_be_bytes().to_vec();
    put_u64(&mut cid_input, duty.session.0);
    put_u64(&mut cid_input, duty.session.1);
    cid_input.extend_from_slice(duty.anchor.block.block_id.as_bytes());
    let program_bytes = program.encode_checked().unwrap();
    cid_input.extend_from_slice(&program_bytes[2..]); // Omit the standalone object version.
    let cid = Cid::new(Hash32::new(
        hash_parts("PoC/Regional/v1", &[&cid_input])
            .try_into()
            .unwrap(),
    ));
    Descriptor {
        plan,
        anchor: duty.anchor.clone(),
        cid,
    }
}

fn check_assignment(
    fixture: &Fixture,
    result: &ReplayResult,
    duty: &ExpectedDuty<'_>,
    basis: &HistoricalBasis<'_>,
) {
    let actual = selected(&result.state);
    assert_eq!(actual, &expected_descriptor(fixture, duty, basis));
    assert_eq!(result.state.public_state().l1_floor, basis.effective_floor);
    assert_eq!(
        actual,
        &Descriptor::decode(&actual.encode_checked().unwrap()).unwrap()
    );
    assert!(result.events.iter().any(|event| matches!(
        &event.body,
        LifecycleBody::SessionAssigned { descriptor } if descriptor == actual
    )));
    let cut = result.state.checkpoint().cuts;
    assert_eq!(
        cut.l1_headers.get(basis.header.height as usize),
        Some(basis.header)
    );
    println!(
        "selected {:?} R{} window {:?} historical L1 header {} {:?} auth {:?} cid {:?}",
        actual.plan.session,
        duty.region,
        duty.window,
        basis.header.height,
        basis.header.hash,
        actual.plan.authorization_ref,
        actual.cid,
    );
}

fn first_duty(fixture: &Fixture, region: u8) -> ExpectedDuty<'_> {
    ExpectedDuty {
        cause: BoundaryCause::Force,
        session: (1, 0),
        source: Some(0),
        region,
        window: (8, 17),
        reference: &fixture.genesis().anchor.block,
        anchor: &fixture.genesis().anchor,
    }
}

#[test]
fn historical_header_is_required_after_public_control_coverage() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"historical-wait");
    world.clear_l1();
    world.deliver_l1(2, 2);
    let policy = policy(6, DEFAULT_ORDER);
    let control = policy.control((2, 0, 0), 1, 0);
    let position = at_force(&control);
    world.l1.controls.insert(2, vec![control.clone()]);
    let before = genesis(&world);
    heartbeat(&mut world, 0, 14 * SECOND);
    let waiting = advance(&world, &before, &position);
    expect_wait_for_history(&waiting, &before, &position, 3);

    world.deliver_l1(3, 3);
    let after = advance(&world, &waiting.state, &position);
    single_effect(&after, &position);
    let checkpoint = after.state.checkpoint();
    let header = fixture.tokens[3].header();
    check_assignment(
        &fixture,
        &after,
        &first_duty(&fixture, 1),
        &HistoricalBasis {
            policy,
            control: &control,
            effective_floor: 3,
            header,
        },
    );
    assert_eq!(
        after.state.public_state().entries,
        before.public_state().entries
    );
    assert!(after.state.public_state().closed_branches.contains(&0));
    assert!(matches!(
        after.events[0].body,
        LifecycleBody::BranchClosed {
            branch: 0,
            reason: CloseReason::Force
        }
    ));
    assert!(matches!(&after.added_records[0].input_ref, InputRef::L1(x) if x == &control));

    let restored = initialize(&world, &InitInput::Checkpoint(checkpoint.clone())).unwrap();
    assert_eq!(restored.status, ReplayStatus::Ready);
    assert_eq!(restored.state.as_ref(), Some(&after.state));
    let mut corrupt = checkpoint;
    corrupt.cuts.l1_headers[3].hash = Hash32::new([0xFE; 32]);
    assert!(initialize(&world, &InitInput::Checkpoint(corrupt)).is_err());
}

#[test]
fn signed_mask_and_cycle_select_different_fixed_committees() {
    let fixture = Fixture::new();
    let mut seen = Vec::new();
    for (regional_policy, region) in [
        (policy(6, DEFAULT_ORDER), 1),
        (policy(4, DEFAULT_ORDER), 2),
        (policy(6, [0, 2, 1]), 2),
    ] {
        let mut world = reader(&fixture, b"mask-and-cycle");
        let control = regional_policy.control((2, 0, 0), 1, 0);
        let at = at_force(&control);
        world.l1.controls.insert(2, vec![control.clone()]);
        let before = genesis(&world);
        heartbeat(&mut world, 0, 14 * SECOND);
        let result = advance(&world, &before, &at);
        single_effect(&result, &at);
        check_assignment(
            &fixture,
            &result,
            &first_duty(&fixture, region),
            &HistoricalBasis {
                policy: regional_policy,
                control: &control,
                effective_floor: 3,
                header: fixture.tokens[3].header(),
            },
        );
        let new = selected(&result.state);
        assert!(seen.iter().all(|(old_cid, old_auth)| {
            *old_cid != new.cid && *old_auth != new.plan.authorization_ref
        }));
        seen.push((new.cid, new.plan.authorization_ref));
    }
}

#[test]
fn a_contributed_r0_selects_r1_instead_of_using_the_session_number() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"contributed-policy");
    let start = genesis(&world);
    let old = selected(&start).clone();
    let chain = fixture.physical_chain(fixture.genesis(), &[1], Some(1));
    let h1 = &chain[0];
    let messages = world.result_messages(&old, &chain);
    world.deliver(&World::block(0, 12 * SECOND, messages));
    let contributed = advance(&world, &start, &ep(12 * SECOND, 0));
    assert_eq!(contributed.status, ReplayStatus::Ready);
    assert_eq!(contributed.state.public_state().entries.len(), 1);
    let anchor = fixture.execution_anchor(h1, Some(1));

    let new_policy = policy(6, DEFAULT_ORDER);
    let control = new_policy.control((7, 0, 0), 1, 0);
    let at = at_force(&control);
    world.l1.controls.insert(7, vec![control.clone()]);
    heartbeat(&mut world, 1, 24 * SECOND);
    let result = advance(&world, &contributed.state, &at);
    single_effect(&result, &at);
    check_assignment(
        &fixture,
        &result,
        &ExpectedDuty {
            cause: BoundaryCause::Force,
            session: (1, 1),
            source: Some(0),
            region: 1,
            window: (18, 27),
            reference: &anchor.anchor.block,
            anchor: &anchor.anchor,
        },
        &HistoricalBasis {
            policy: new_policy,
            control: &control,
            effective_floor: 7,
            header: fixture.tokens[7].header(),
        },
    );
    assert_eq!(
        result.state.public_state().entries,
        contributed.state.public_state().entries
    );
}

#[test]
fn empty_eligible_set_with_complete_history_becomes_pending() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"empty-mask");
    let control = policy(0, DEFAULT_ORDER).control((2, 0, 0), 1, 0);
    let at = at_force(&control);
    world.clear_l1();
    world.deliver_l1(2, 2);
    world.l1.controls.insert(2, vec![control]);
    heartbeat(&mut world, 0, 14 * SECOND);
    let initial = genesis(&world);
    let waiting = advance(&world, &initial, &at);
    expect_wait_for_history(&waiting, &initial, &at, 3);

    world.deliver_l1(3, 3);
    let ready = advance(&world, &waiting.state, &at);
    single_effect(&ready, &at);
    let Tail::Pending(boundary) = &ready.state.public_state().tail else {
        panic!("fully verified policy with an empty eligible mask must become PENDING");
    };
    assert_eq!(boundary.input.session, SessionId { beta: 1, r: 0 });
    assert_eq!(boundary.input.cause, BoundaryCause::Force);
    assert_eq!(
        boundary.input.source_region,
        Some(RegionId::new(0).unwrap())
    );
    assert_eq!(
        boundary.input.reference_block,
        fixture.genesis().anchor.block
    );
    assert_eq!(boundary.anchor.as_ref(), Some(&fixture.genesis().anchor));
    assert_eq!(
        boundary.input.window,
        Window {
            h_min: 8,
            h_max: 17
        }
    );
    assert_eq!(ready.events.len(), 1);
    assert!(matches!(
        ready.events[0].body,
        LifecycleBody::BranchClosed {
            branch: 0,
            reason: CloseReason::Force
        }
    ));
    println!("PENDING at {at:?}: {boundary:?}");
}

fn check_noop(result: &ReplayResult, before: &ReplayState, at: &ReplayPosition) {
    single_effect(result, at);
    assert!(result.events.is_empty());
    assert!(result.added_records[0].effects.is_empty());
    assert!(result.added_records[0].events.is_empty());
    let mut expected = before.public_state().clone();
    expected.pc = Some(at.clone());
    expected.records.push(result.added_records[0].clone());
    assert_eq!(result.state.public_state(), &expected);
    println!("public no-op at {at:?}");
}

#[test]
fn inclusion_is_inactive_before_watermark_and_same_block_controls_recheck_branch() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"ordered-policy-activation");
    let first_policy = policy(6, DEFAULT_ORDER);
    let first = first_policy.control((2, 0, 0), 1, 0);
    let stale_before = policy(0, DEFAULT_ORDER).control((3, 0, 0), 2, 0);
    let restored_policy = policy(7, [1, 0, 2]);
    let restored = restored_policy.control((3, 0, 1), 3, 1);
    let stale_after = policy(4, DEFAULT_ORDER).control((3, 1, 0), 4, 1);
    world.l1.controls.insert(2, vec![first.clone()]);
    world.l1.controls.insert(
        3,
        vec![stale_before.clone(), restored.clone(), stale_after.clone()],
    );
    let initial = genesis(&world);
    let before_public = heartbeat(&mut world, 0, 14 * SECOND - 1);
    let before_force = advance(&world, &initial, &before_public);
    assert_eq!(before_force.status, ReplayStatus::Ready);
    assert_eq!(selected(&before_force.state), selected(&initial));
    let waiting = advance(&world, &before_force.state, &at_force(&first));
    assert!(matches!(waiting.status, ReplayStatus::Wait { .. }));
    assert_eq!(waiting.state, before_force.state);
    assert!(waiting.events.is_empty() && waiting.added_records.is_empty());

    let first_watermark = heartbeat(&mut world, 1, 14 * SECOND);
    let first_activation = advance(&world, &waiting.state, &at_force(&first));
    single_effect(&first_activation, &at_force(&first));
    check_assignment(
        &fixture,
        &first_activation,
        &first_duty(&fixture, 1),
        &HistoricalBasis {
            policy: first_policy,
            control: &first,
            effective_floor: 3,
            header: fixture.tokens[3].header(),
        },
    );
    let old = selected(&first_activation.state).clone();
    let after_row = advance(&world, &first_activation.state, &first_watermark);
    heartbeat(&mut world, 2, 16 * SECOND);
    let invalid = advance(&world, &after_row.state, &at_force(&stale_before));
    check_noop(&invalid, &after_row.state, &at_force(&stale_before));

    let valid = advance(&world, &invalid.state, &at_force(&restored));
    single_effect(&valid, &at_force(&restored));
    check_assignment(
        &fixture,
        &valid,
        &ExpectedDuty {
            cause: BoundaryCause::Force,
            session: (2, 0),
            source: Some(1),
            region: 0,
            window: (10, 19),
            reference: &fixture.genesis().anchor.block,
            anchor: &fixture.genesis().anchor,
        },
        &HistoricalBasis {
            policy: restored_policy,
            control: &restored,
            effective_floor: 3,
            header: fixture.tokens[3].header(),
        },
    );
    let invalid_again = advance(&world, &valid.state, &at_force(&stale_after));
    check_noop(&invalid_again, &valid.state, &at_force(&stale_after));
    assert_eq!(
        invalid_again.state.public_state().selected.get(&(1, 0)),
        Some(&old)
    );
    let restored_state = initialize(
        &world,
        &InitInput::Checkpoint(invalid_again.state.checkpoint()),
    )
    .unwrap();
    assert_eq!(restored_state.state.as_ref(), Some(&invalid_again.state));
}

#[test]
fn a_regular_successor_uses_the_policy_active_at_its_own_replay_position() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"regular-active-policy");
    let first_policy = policy(6, DEFAULT_ORDER);
    let first = first_policy.control((2, 0, 0), 1, 0);
    // This control's inclusion is visible to readers before SessionEnd, but act=32 s.
    let future = policy(1, [2, 0, 1]).control((11, 0, 0), 2, 1);
    world.l1.controls.insert(2, vec![first.clone()]);
    world.l1.controls.insert(11, vec![future]);
    let initial = genesis(&world);
    let water = heartbeat(&mut world, 0, 14 * SECOND);
    let before_end = advance(&world, &initial, &water).state;
    let predecessor = selected(&before_end).clone();
    heartbeat(&mut world, 1, 31 * SECOND);
    let at = ReplayPosition {
        key_us: 31 * SECOND,
        kind: ReplayKind::SessionEnd,
        index: ReplayIndex::Session(SessionId { beta: 1, r: 0 }),
    };
    let regular = advance(&world, &before_end, &at);
    single_effect(&regular, &at);
    check_assignment(
        &fixture,
        &regular,
        &ExpectedDuty {
            cause: BoundaryCause::Handoff,
            session: (1, 1),
            source: Some(1),
            region: 2,
            window: (18, 27),
            reference: &predecessor.anchor.block,
            anchor: &fixture.genesis().anchor,
        },
        &HistoricalBasis {
            policy: first_policy,
            control: &first,
            effective_floor: 3,
            header: fixture.tokens[3].header(),
        },
    );
    assert_eq!(
        regular.state.public_state().selected.get(&(1, 0)),
        Some(&predecessor)
    );
}

#[test]
fn halt_does_not_activate_an_otherwise_valid_control() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"halted");
    let control = policy(6, DEFAULT_ORDER).control((16, 0, 0), 1, 0);
    world.l1.controls.insert(16, vec![control.clone()]);
    let initial = genesis(&world);
    let before = heartbeat(&mut world, 0, 41 * SECOND);
    let expired = advance(&world, &initial, &before);
    assert_eq!(expired.status, ReplayStatus::Ready);
    assert!(matches!(
        expired.state.public_state().tail,
        Tail::Halt { .. }
    ));
    assert!(
        expired
            .added_records
            .iter()
            .any(|row| row.position == expiry(41 * SECOND, 1))
    );
    heartbeat(&mut world, 1, 42 * SECOND);
    let after = advance(&world, &expired.state, &at_force(&control));
    check_noop(&after, &expired.state, &at_force(&control));
}

#[test]
fn readers_with_different_delivery_order_agree_at_the_same_position() {
    let fixture = Fixture::new();
    let mut a = reader(&fixture, b"A");
    let mut b = reader(&fixture, b"B");
    let control = policy(6, DEFAULT_ORDER).control((2, 0, 0), 1, 0);
    let first = at_force(&control);
    for world in [&mut a, &mut b] {
        world.clear_l1();
        world.l1.controls.insert(2, vec![control.clone()]);
    }
    let initial_a = genesis(&a);
    let initial_b = genesis(&b);
    let end = heartbeat(&mut a, 0, 14 * SECOND);
    let a_wait = advance(&a, &initial_a, &first);
    assert!(matches!(a_wait.status, ReplayStatus::Wait { .. }));
    assert_eq!(a_wait.state, initial_a);
    b.deliver_l1(3, 3);
    let b_wait = advance(&b, &initial_b, &first);
    assert!(matches!(b_wait.status, ReplayStatus::Wait { .. }));
    assert_eq!(b_wait.state, initial_b);

    a.deliver_l1(3, 3);
    heartbeat(&mut b, 0, 14 * SECOND);
    let after_a = advance(&a, &a_wait.state, &end);
    let after_b = advance(&b, &b_wait.state, &end);
    assert_eq!(after_a.status, ReplayStatus::Ready);
    assert_eq!(after_b.status, ReplayStatus::Ready);
    assert_eq!(after_a.state, after_b.state);
    assert_eq!(after_a.events, after_b.events);
    assert_eq!(after_a.added_records, after_b.added_records);
    println!(
        "same historical assignment at {end:?}: {:?}",
        selected(&after_a.state).cid
    );
}

#[test]
fn an_exact_subject_proof_needs_the_selected_historical_descriptor() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"historical-proof-and-body");
    let start = genesis(&world);
    let control = policy(6, DEFAULT_ORDER).control((2, 0, 0), 1, 0);
    world.l1.controls.insert(2, vec![control]);
    let at = heartbeat(&mut world, 0, 14 * SECOND);
    let pre_publication = advance(&world, &start, &at).state;
    let honest = selected(&pre_publication).clone();

    // Construct a structurally valid descriptor and a matching fixture witness
    // with a changed Ref. It does not authenticate the frozen selection.
    let mut forged = honest.clone();
    forged.plan.reference_block.block_id = Hash32::new([0x99; 32]);
    let chain = fixture.physical_chain(fixture.genesis(), &[8], None);
    let physical_id = chain[0].digest().unwrap();
    let bad_messages = world.result_messages(&forged, &chain);
    world.deliver(&World::block(1, 18 * SECOND, bad_messages));
    let stale = advance(&world, &pre_publication, &ep(18 * SECOND, 1));
    assert_eq!(stale.status, ReplayStatus::Ready);
    assert!(stale.state.public_state().entries.is_empty());
    assert_eq!(selected(&stale.state), &honest);

    let correct = world.proof(&honest, &chain[0]);
    world.deliver(&World::block(2, 19 * SECOND, vec![correct]));
    let admitted = advance(&world, &stale.state, &ep(19 * SECOND, 2));
    assert_eq!(admitted.status, ReplayStatus::Ready);
    let entries = &admitted.state.public_state().entries;
    assert_eq!(entries.len(), 8);
    for (index, entry) in entries.iter().enumerate() {
        let h = (index + 1) as u64;
        assert_eq!(
            entry.block,
            fixture.expected_entry(h, (h == 8).then_some(physical_id), None)
        );
        assert_eq!(entry.position, ep(19 * SECOND, 2));
    }
    println!("historical subject recognized at first complete authorized position q=2");
}

#[test]
fn no_source_starts_at_the_first_eligible_cycle_entry() {
    let fixture = Fixture::new();
    let world = reader(&fixture, b"no-source");
    let view = genesis(&world);
    let regional_policy = policy(6, [2, 0, 1]);
    let control = regional_policy.control((2, 0, 0), 1, 0);
    let basis = HistoricalBasis {
        policy: regional_policy,
        control: &control,
        effective_floor: 3,
        header: fixture.tokens[3].header(),
    };
    let expected = expected_descriptor(
        &fixture,
        &ExpectedDuty {
            source: None,
            ..first_duty(&fixture, 2)
        },
        &basis,
    );
    let verified = HistoricalPlan {
        active: ActivePolicy {
            policy: RegionalPolicy {
                enabled_mask: 6,
                order: [2, 0, 1],
                signed_floor: 99,
            },
            force: control.position,
            control_seq: 1,
            floor: 3,
        },
        height: 3,
        block_hash: fixture.tokens[3].header().hash,
    };
    assert_eq!(
        fixture
            .plan
            .resolve_historical(&expected.plan.input(), &view, &verified),
        PlanOutcome::Ready(expected.plan)
    );
}

#[test]
fn worker_facing_plan_verifier_recomputes_the_supplied_historical_basis() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"worker-facing-history");
    let initial = genesis(&world);
    let regional_policy = policy(6, DEFAULT_ORDER);
    let control = regional_policy.control((2, 0, 0), 1, 0);
    let at = at_force(&control);
    world.l1.controls.insert(2, vec![control]);
    heartbeat(&mut world, 0, 14 * SECOND);
    let replay = advance(&world, &initial, &at).state;
    let descriptor = selected(&replay);
    let plan = &descriptor.plan;
    let basis = replay.public_state().plan_bases[&descriptor.cid];
    assert_eq!(
        fixture.plan.validate_historical_plan(plan, &replay, &basis),
        PlanOutcome::Ready(plan.clone())
    );
    assert_eq!(basis.height, 3);
    assert_eq!(basis.block_hash, fixture.tokens[3].header().hash);
    assert_eq!(basis.active, replay.public_state().policies[&1]);
    assert_eq!(HistoricalPlan::decode(&basis.encode()).unwrap(), basis);

    let mut bad_height = basis;
    bad_height.height = 2;
    let mut bad_hash = basis;
    bad_hash.block_hash = Hash32::new([0xFF; 32]);
    let mut bad_policy = basis;
    bad_policy.active.policy.order = [0, 2, 1];
    let mut bad_floor = basis;
    bad_floor.active.floor = 99;
    let mut bad_position = basis;
    bad_position.active.force.tx = u64::from(u32::MAX) + 1;
    for bad_basis in [bad_height, bad_hash, bad_policy, bad_floor, bad_position] {
        assert!(
            matches!(
                fixture
                    .plan
                    .validate_historical_plan(plan, &replay, &bad_basis),
                PlanOutcome::Reject(_)
            ),
            "altered basis was accepted: {bad_basis:?}"
        );
    }

    let mut wrong_committee = plan.clone();
    wrong_committee.committee[0].vote_key[0] ^= 1;
    assert!(matches!(
        fixture
            .plan
            .validate_historical_plan(&wrong_committee, &replay, &basis),
        PlanOutcome::Reject(_)
    ));
    let mut wrong_ref = plan.clone();
    wrong_ref.reference_block.block_id = Hash32::new([0xFE; 32]);
    assert!(matches!(
        fixture
            .plan
            .validate_historical_plan(&wrong_ref, &replay, &basis),
        PlanOutcome::Wait(InputMissing::Reference(_)) | PlanOutcome::Reject(_)
    ));
    let mut incorrect_rule = plan.clone();
    incorrect_rule.leader_rule.params[0] ^= 1;
    assert!(incorrect_rule.validate().is_err());
}

#[test]
fn malformed_mask_and_nonpermutation_fail_historical_plan_resolution() {
    let fixture = Fixture::new();
    let world = reader(&fixture, b"malformed-policy");
    let state = genesis(&world);
    let input = PlanInput {
        cause: BoundaryCause::Force,
        session: SessionId { beta: 1, r: 0 },
        source_region: Some(RegionId::new(0).unwrap()),
        window: Window {
            h_min: 8,
            h_max: 17,
        },
        reference_block: fixture.genesis().anchor.block.clone(),
    };
    for (mask, order) in [(8, [0, 1, 2]), (7, [1, 1, 2]), (7, [0, 1, 3])] {
        let basis = HistoricalPlan {
            active: ActivePolicy {
                policy: RegionalPolicy {
                    enabled_mask: mask,
                    order,
                    signed_floor: 99,
                },
                force: L1Position {
                    block: 2,
                    tx: 0,
                    log: 0,
                },
                control_seq: 1,
                floor: 3,
            },
            height: 3,
            block_hash: fixture.tokens[3].header().hash,
        };
        assert!(
            matches!(
                fixture.plan.resolve_historical(&input, &state, &basis),
                PlanOutcome::Reject(_)
            ),
            "malformed policy was accepted: {:?}, {:?}",
            mask,
            order
        );
    }
}
