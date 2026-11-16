//! Source-derived acceptance vectors for the production GFR replay entry point.
#![allow(dead_code)] // The shared independent fixture has helpers for M1 checks too.

#[path = "../../../viewer-gfr/independent-harness/src/m2_fixture.rs"]
mod fixture;
#[path = "../../../viewer-gfr/independent-harness/src/m2_world.rs"]
mod world;

use fixture::{Fixture, key};
use poc_types::domain::*;
use poc_viewer_gfr::*;
use std::collections::BTreeSet;
use world::{World, end, ep, expiry};

fn sec(value: u64) -> u64 {
    value * 1_000_000
}

fn reader<'a>(fixture: &'a Fixture, name: &[u8]) -> World<'a> {
    let mut world = World::new(fixture, name);
    world.params = GfrParameters {
        announcement_us: sec(10),
        grace_us: sec(3),
        expiry_us: sec(8),
    };
    world
}

fn force(block: u64, tx: u64, log: u64, seq: u64, target: u64) -> ControlRef {
    ControlRef::static_force(L1Position { block, tx, log }, seq, target)
}

fn position(time: u64, control: &ControlRef) -> ReplayPosition {
    ReplayPosition {
        key_us: sec(time),
        kind: ReplayKind::Force,
        index: ReplayIndex::L1(control.position),
    }
}

fn publish(
    world: &mut World<'_>,
    q: u64,
    time: u64,
    subject: &Descriptor,
    bodies: &[PhysicalBlock],
) -> ReplayPosition {
    let messages = world.result_messages(subject, bodies);
    world.deliver(&World::block(q, sec(time), messages));
    ep(sec(time), q)
}

fn watermark(world: &mut World<'_>, q: u64, time: u64) -> ReplayPosition {
    world.deliver(&World::block(q, sec(time), vec![]));
    ep(sec(time), q)
}

fn prefix(world: &mut World<'_>, last: u64, time: u64) -> (ReplayState, Vec<PhysicalBlock>) {
    let fixture = world.f;
    let genesis = fixture.genesis();
    let descriptor = fixture.descriptor(0, &genesis.anchor, &genesis.anchor);
    let blocks = fixture.physical_chain(genesis, &(1..=last).collect::<Vec<_>>(), Some(3));
    let start = world.init();
    let through = publish(world, 0, time, &descriptor, &blocks);
    let result = world.advance(&start, &through);
    assert_eq!(result.status, ReplayStatus::Ready);
    assert_eq!(result.state.public_state().entries.len(), last as usize);
    for (index, entry) in result.state.public_state().entries.iter().enumerate() {
        let block = &blocks[index];
        assert_eq!(
            entry.block,
            fixture.expected_entry(
                block.metadata.height,
                Some(block.digest().unwrap()),
                Some(3)
            )
        );
    }
    (result.state, blocks)
}

fn next(state: &ReplayState) -> &Descriptor {
    let Tail::Next(descriptor) = &state.public_state().tail else {
        panic!("expected NEXT, got {:?}", state.public_state().tail)
    };
    descriptor
}

fn record_positions(result: &ReplayResult) -> Vec<ReplayPosition> {
    assert_eq!(result.status, ReplayStatus::Ready);
    result
        .added_records
        .iter()
        .map(|row| row.position.clone())
        .collect()
}

fn expected_force(
    result: &ReplayResult,
    before: &ReplayState,
    at: &ReplayPosition,
    anchor: &Anchor,
    target: (u64, u64, u8, u8),
    window: (u64, u64),
    floor: u64,
) {
    let (beta, r, region, source) = target;
    let state = result.state.public_state();
    let descriptor = next(&result.state);
    assert_eq!(record_positions(result), vec![at.clone()]);
    assert_eq!(state.branch, beta);
    assert_eq!(state.l1_floor, floor);
    assert!(
        state
            .closed_branches
            .contains(&before.public_state().branch)
    );
    assert_eq!(state.pc.as_ref(), Some(at));
    assert_eq!(state.entries, before.public_state().entries);
    assert_eq!(descriptor.plan.session, SessionId { beta, r });
    assert_eq!(descriptor.plan.region.get(), region);
    assert_eq!(
        descriptor.plan.source_region,
        Some(RegionId::new(source).unwrap())
    );
    assert_eq!(descriptor.plan.cause, BoundaryCause::Force);
    assert_eq!(
        descriptor.plan.window,
        Window {
            h_min: window.0,
            h_max: window.1
        }
    );
    assert_eq!(&descriptor.anchor, anchor);
    assert_eq!(descriptor.plan.reference_block, anchor.block);
    let program = ProgramConfig::from_plan(&state.config, &descriptor.plan).unwrap();
    assert_eq!(
        descriptor.cid,
        regional_cid(&SessionId { beta, r }, anchor.block.block_id, &program).unwrap()
    );
    assert_eq!(
        result.added_records[0].effects,
        [ReplayEffect::Duty {
            before: before.public_state().tail.clone(),
            after: Tail::Next(descriptor.clone()),
        }]
    );
    assert_eq!(
        result.events,
        [
            LifecycleEvent {
                position: at.clone(),
                body: LifecycleBody::BranchClosed {
                    branch: before.public_state().branch,
                    reason: CloseReason::Force,
                },
            },
            LifecycleEvent {
                position: at.clone(),
                body: LifecycleBody::SessionAssigned {
                    descriptor: descriptor.clone()
                },
            }
        ]
    );
    assert_eq!(result.events, result.added_records[0].events);
    assert!(matches!(
        &result.added_records[0].input_ref,
        InputRef::L1(control)
            if ReplayIndex::L1(control.position) == at.index
                && control.force_branch() == Some(before.public_state().branch)
    ));
    println!(
        "Force {:?}: beta={beta} r={r} R{region} source=R{source} anchor={} window={window:?} floor={floor} cid={:?}",
        at, anchor.block.height, descriptor.cid
    );
}

fn expect_noop(result: &ReplayResult, before: &ReplayState, at: &ReplayPosition) {
    assert_eq!(record_positions(result), vec![at.clone()]);
    let old = before.public_state();
    let new = result.state.public_state();
    assert_eq!(new.branch, old.branch);
    assert_eq!(new.l1_floor, old.l1_floor);
    assert_eq!(new.tail, old.tail);
    assert_eq!(new.entries, old.entries);
    assert_eq!(new.closed_branches, old.closed_branches);
    assert_eq!(new.closed_sessions, old.closed_sessions);
    assert_eq!(new.selected, old.selected);
    assert_eq!(new.pc.as_ref(), Some(at));
    assert!(result.added_records[0].effects.is_empty());
    assert!(result.added_records[0].events.is_empty());
    assert!(result.events.is_empty());
    println!("Control at {at:?}: no-op");
}

fn expect_wait(
    result: &ReplayResult,
    before: &ReplayState,
    known_position: Option<&ReplayPosition>,
    reason: Missing,
) {
    let ReplayStatus::Wait {
        next_position,
        missing,
    } = &result.status
    else {
        panic!("expected WAIT, got {:?}", result.status)
    };
    if let Some(at) = known_position {
        assert_eq!(next_position.as_ref(), Some(at));
    }
    assert!(
        missing.contains(&reason),
        "expected {reason:?} in {missing:?}"
    );
    assert_eq!(&result.state, before);
    assert!(result.added_records.is_empty());
    assert!(result.events.is_empty());
    println!("WAIT {next_position:?}: {missing:?}");
}

#[test]
fn hf04_next_without_physical_contribution() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"unused-next");
    let initial = world.init();
    let control = force(4, 0, 0, 1, 0);
    let at = position(18, &control);
    world.l1.controls.insert(4, vec![control]);
    watermark(&mut world, 0, 18);
    let result = world.advance(&initial, &at);
    expected_force(
        &result,
        &initial,
        &at,
        &fixture.genesis().anchor,
        (1, 0, 1, 0),
        (12, 21),
        4,
    );
}

#[test]
fn hf04_force_l1_floor_does_not_regress() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"l1-floor-max");
    let initial = world.init();
    assert_eq!(initial.public_state().l1_floor, 3);
    let control = force(2, 0, 0, 1, 0);
    let at = position(14, &control);
    world.l1.controls.insert(2, vec![control]);
    watermark(&mut world, 0, 14);
    let result = world.advance(&initial, &at);
    expected_force(
        &result,
        &initial,
        &at,
        &fixture.genesis().anchor,
        (1, 0, 1, 0),
        (8, 17),
        3,
    );
}

#[test]
fn hf04_next_with_physical_contribution() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"contributed-next");
    let (before, blocks) = prefix(&mut world, 1, 12);
    let expected_anchor = fixture.execution_anchor(&blocks[0], Some(3));
    let control = force(4, 0, 0, 1, 0);
    let at = position(18, &control);
    world.l1.controls.insert(4, vec![control]);
    watermark(&mut world, 1, 18);
    let result = world.advance(&before, &at);
    expected_force(
        &result,
        &before,
        &at,
        &expected_anchor.anchor,
        (1, 1, 2, 0),
        (12, 21),
        4,
    );
}

#[test]
fn hf04_pending_keeps_saved_r_and_source() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"regular-pending");
    // Publish h1 early enough to keep the next-missing timer behind the prefix.
    let (first, one) = prefix(&mut world, 1, 12);
    let old = fixture.descriptor(0, &fixture.genesis().anchor, &fixture.genesis().anchor);
    let blocks = fixture.physical_chain(fixture.genesis(), &(1..=9).collect::<Vec<_>>(), Some(3));
    assert_eq!(one[0], blocks[0]);
    let first_nine = publish(&mut world, 1, 19, &old, &blocks);
    let partial = world.advance(&first, &first_nine);
    assert_eq!(partial.state.public_state().entries.len(), 9);
    world.plan.none.set(Some(1));
    watermark(&mut world, 2, 24);
    let pending = world.advance(&partial.state, &end(sec(24), 0));
    let Tail::Pending(boundary) = &pending.state.public_state().tail else {
        panic!("no transition to PENDING at SessionEnd")
    };
    assert_eq!(boundary.input.session, SessionId { beta: 0, r: 1 });
    assert_eq!(
        boundary.input.source_region,
        Some(RegionId::new(0).unwrap())
    );
    assert_eq!(pending.events.len(), 1);

    let anchor = fixture.execution_anchor(&blocks[8], Some(3));
    assert_eq!(boundary.anchor.as_ref(), Some(&anchor.anchor));
    world.plan.none.set(None);
    let control = force(8, 0, 0, 1, 0);
    let at = position(26, &control);
    world.l1.controls.insert(8, vec![control]);
    watermark(&mut world, 3, 26);
    let result = world.advance(&pending.state, &at);
    assert_eq!(record_positions(&result), vec![ep(sec(24), 2), at.clone()]);
    let before_force = world.advance(&pending.state, &ep(sec(24), 2)).state;
    let result = world.advance(&before_force, &at);
    expected_force(
        &result,
        &before_force,
        &at,
        &anchor.anchor,
        (1, 1, 2, 0),
        (20, 29),
        8,
    );
}

fn through_nine(world: &mut World<'_>) -> (ReplayState, Descriptor, Vec<PhysicalBlock>) {
    let fixture = world.f;
    let (first, _) = prefix(world, 1, 12);
    let genesis = fixture.genesis();
    let old = fixture.descriptor(0, &genesis.anchor, &genesis.anchor);
    // Two public rows keep the entire accepted interval ahead of its expiry.
    let blocks = fixture.physical_chain(genesis, &(1..=10).collect::<Vec<_>>(), Some(3));
    let p = publish(world, 1, 19, &old, &blocks[..9]);
    let result = world.advance(&first, &p);
    assert_eq!(result.status, ReplayStatus::Ready);
    assert_eq!(result.state.public_state().entries.len(), 9);
    (result.state, old, blocks)
}

#[test]
fn hf05_force_precedes_old_session_end_and_same_key_publication() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"force-before-old-cutoff");
    let (before, old, blocks) = through_nine(&mut world);
    let old_key = key(&old, &blocks[9]);
    let control = force(7, 0, 0, 1, 0);
    let at = position(24, &control);
    world.l1.controls.insert(7, vec![control]);
    let same_key = publish(&mut world, 2, 24, &old, &blocks);
    let result = world.advance(&before, &same_key);
    assert_eq!(
        record_positions(&result),
        vec![at.clone(), same_key.clone()]
    );
    assert_eq!(
        result.state.public_state().entries,
        before.public_state().entries
    );
    assert_eq!(result.events.len(), 2);
    assert!(result.events.iter().all(|event| event.position == at));
    assert!(result.added_records[1].effects.is_empty());
    assert!(result.added_records[1].events.is_empty());
    let exact_force = world.advance(&before, &at);
    expected_force(
        &exact_force,
        &before,
        &at,
        &fixture.execution_anchor(&blocks[8], Some(3)).anchor,
        (1, 1, 2, 0),
        (18, 27),
        7,
    );
    // The old result is a valid historical witness in this public input.
    // Admission remains governed by duty and the same-key cutoff.
    assert_ne!(next(&result.state).cid, old_key.cid);
    assert_eq!(result.state.public_state().pc.as_ref(), Some(&same_key));
    println!(
        "Observed positions: {:?}; old h10 {:?} excluded",
        record_positions(&result),
        old_key
    );
}

#[test]
fn hf06_same_key_expire_closes_new_branch() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"force-then-expire");
    let (before, blocks) = prefix(&mut world, 1, 12);
    let control = force(5, 0, 0, 1, 0);
    let at = position(20, &control);
    world.l1.controls.insert(5, vec![control]);
    let same_key = watermark(&mut world, 1, 20);
    let forced = world.advance(&before, &at);
    let anchor = fixture.execution_anchor(&blocks[0], Some(3));
    expected_force(
        &forced,
        &before,
        &at,
        &anchor.anchor,
        (1, 1, 2, 0),
        (14, 23),
        5,
    );

    let result = world.advance(&before, &same_key);
    let timer = expiry(sec(20), 2);
    assert_eq!(record_positions(&result), vec![at, timer.clone(), same_key]);
    let state = result.state.public_state();
    assert_eq!(state.entries, before.public_state().entries);
    assert_eq!(state.branch, 1);
    let Tail::Halt {
        boundary,
        anchor: halted_anchor,
    } = &state.tail
    else {
        panic!("equal-key Expire did not halt")
    };
    assert_eq!(boundary.input.session, SessionId { beta: 1, r: 1 });
    assert_eq!(halted_anchor, &anchor.anchor);
    assert_eq!(state.closed_branches, BTreeSet::from([0, 1]));
    assert_eq!(
        result.added_records[1].events,
        [LifecycleEvent {
            position: timer,
            body: LifecycleBody::BranchClosed {
                branch: 1,
                reason: CloseReason::Expired
            },
        }]
    );
    println!("Observed positions: {:?}", record_positions(&result));
}

#[test]
fn hf07_l1_tx_and_local_log_order_deduplicates_replay() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"same-l1-block");
    let (before, blocks) = prefix(&mut world, 1, 12);
    let controls = vec![
        force(4, 0, 1, 1, 0),
        force(4, 1, 0, 2, 0),
        force(4, 1, 1, 3, 1),
    ];
    let positions: Vec<_> = controls
        .iter()
        .map(|control| position(18, control))
        .collect();
    world.l1.controls.insert(4, controls);
    let same_key = watermark(&mut world, 1, 18);
    let anchor = fixture.execution_anchor(&blocks[0], Some(3));

    let first = world.advance(&before, &positions[0]);
    expected_force(
        &first,
        &before,
        &positions[0],
        &anchor.anchor,
        (1, 1, 2, 0),
        (12, 21),
        4,
    );
    let second = world.advance(&first.state, &positions[1]);
    expect_noop(&second, &first.state, &positions[1]);
    let third = world.advance(&second.state, &positions[2]);
    expected_force(
        &third,
        &second.state,
        &positions[2],
        &anchor.anchor,
        (2, 1, 0, 2),
        (12, 21),
        4,
    );
    assert_ne!(next(&first.state).cid, next(&third.state).cid);

    let result = world.advance(&before, &same_key);
    assert_eq!(
        record_positions(&result),
        [positions, vec![same_key.clone()]].concat()
    );
    world.deliver_l1(4, 4); // Re-deliver both identical public prefixes.
    watermark(&mut world, 1, 18);
    let again = world.advance(&result.state, &same_key);
    assert_eq!(again.status, ReplayStatus::Ready);
    assert_eq!(again.state, result.state);
    assert!(again.added_records.is_empty());
    assert!(again.events.is_empty());

    let restored = world
        .init_input(&InitInput::Checkpoint(result.state.checkpoint()))
        .unwrap();
    assert_eq!(restored.status, ReplayStatus::Ready);
    assert_eq!(restored.state, Some(result.state.clone()));
    println!(
        "Observed positions: {:?}; identical prefix and checkpoint agree",
        record_positions(&result)
    );
}

#[test]
fn hf10_wait_for_complete_public_inputs_and_force_plan() {
    let fixture = Fixture::new();
    let mut a = reader(&fixture, b"l1-first");
    let mut b = reader(&fixture, b"espresso-first");
    a.clear_l1();
    a.deliver_l1(4, 4);
    b.clear_l1();
    b.deliver_l1(4, 3);
    let start_a = a.init();
    let start_b = b.init();
    let control = force(4, 0, 0, 1, 0);
    let at = position(18, &control);
    a.l1.controls.insert(4, vec![control.clone()]);
    b.l1.controls.insert(4, vec![control]);
    let only_l1 = a.advance(&start_a, &at);
    expect_wait(
        &only_l1,
        &start_a,
        None,
        Missing::EspressoWatermark { key_us: sec(18) },
    );

    watermark(&mut b, 0, 18);
    let only_espresso = b.advance(&start_b, &at);
    expect_wait(
        &only_espresso,
        &start_b,
        None,
        Missing::L1ControlCoverage { through_height: 4 },
    );

    let through_a = watermark(&mut a, 0, 18);
    let through_b = ep(sec(18), 0);
    b.deliver_l1(4, 4);
    a.plan.wait.set(Some(0));
    b.plan.wait.set(Some(0));
    for (world, before) in [(&a, &start_a), (&b, &start_b)] {
        let blocked = world.advance(before, &at);
        expect_wait(
            &blocked,
            before,
            Some(&at),
            Missing::Input(InputMissing::Reference(
                fixture.genesis().anchor.block.clone(),
            )),
        );
        world.plan.wait.set(None);
    }
    let result_a = a.advance(&start_a, &through_a);
    let result_b = b.advance(&start_b, &through_b);
    assert_eq!(record_positions(&result_a), vec![at.clone(), through_a]);
    assert_eq!(record_positions(&result_b), vec![at, through_b]);
    assert_eq!(result_a.events, result_b.events);
    assert_eq!(result_a.state.public_state(), result_b.state.public_state());
    assert_eq!(result_a.state.checkpoint(), result_b.state.checkpoint());
    println!(
        "Independent reader delivery orders agree at {:?}",
        result_a.state.public_state().pc
    );
}

#[test]
fn hf11_stale_control_preserves_responsibility() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"stale-target");
    let before = world.init();
    let control = force(4, 0, 0, 1, 7);
    let at = position(18, &control);
    world.l1.controls.insert(4, vec![control]);
    watermark(&mut world, 0, 18);
    expect_noop(&world.advance(&before, &at), &before, &at);
}

#[test]
fn hf11_force_on_halt_cannot_wake_branch() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"force-on-halt");
    let initial = world.init();
    // At x=19s the reader already needs all of block 5, which includes the
    // future-key Force. Its own x=20s still lacks an Espresso watermark.
    let control = force(5, 0, 0, 1, 0);
    let at = position(20, &control);
    world.l1.controls.insert(5, vec![control]);
    let p = watermark(&mut world, 0, 19);
    let halted = world.advance(&initial, &p);
    assert_eq!(record_positions(&halted), vec![expiry(sec(19), 1), p]);
    assert!(matches!(
        halted.state.public_state().tail,
        Tail::Halt { .. }
    ));
    watermark(&mut world, 1, 20);
    expect_noop(&world.advance(&halted.state, &at), &halted.state, &at);
}

#[test]
fn hf14_session_end_from_h9_pads_h10_only_on_h11_admission() {
    let fixture = Fixture::new();
    let mut world = reader(&fixture, b"regular-cutoff-padding");
    let (before, old, blocks) = through_nine(&mut world);
    let control = force(7, 0, 0, 1, 7);
    let at = position(24, &control);
    world.l1.controls.insert(7, vec![control]);
    let late_h10 = publish(&mut world, 2, 24, &old, &blocks);
    let transitioned = world.advance(&before, &late_h10);
    let cutoff = end(sec(24), 0);
    // A same-key stale Force is recorded, then old SessionEnd owns the
    // transition. The same-key physical h10 cannot retroactively fill G.
    assert_eq!(
        record_positions(&transitioned),
        vec![at, cutoff.clone(), late_h10]
    );
    let state = transitioned.state.public_state();
    assert_eq!(state.branch, 0);
    assert_eq!(state.entries, before.public_state().entries);
    assert_eq!(transitioned.events.len(), 2);
    assert!(
        transitioned
            .events
            .iter()
            .all(|event| event.position == cutoff)
    );
    assert_eq!(
        transitioned.events[0].body,
        LifecycleBody::SessionClosed {
            session: SessionId { beta: 0, r: 0 },
            reason: CloseReason::Complete,
        },
    );
    let anchor = fixture.execution_anchor(&blocks[8], Some(3));
    let successor = fixture.descriptor(1, &anchor.anchor, &fixture.genesis().anchor);
    assert_eq!(next(&transitioned.state), &successor);
    assert_eq!(
        successor.plan.window,
        Window {
            h_min: 11,
            h_max: 20
        }
    );
    assert_eq!(successor.anchor.block.height, 9);
    assert_eq!(
        successor.plan.reference_block,
        fixture.genesis().anchor.block
    );

    let h11 = fixture.physical_chain(&anchor, &[11], Some(3));
    let p = publish(&mut world, 3, 25, &successor, &h11);
    let admitted = world.advance(&transitioned.state, &p);
    assert_eq!(record_positions(&admitted), vec![p.clone()]);
    let entries = &admitted.state.public_state().entries;
    assert_eq!(entries.len(), 11);
    assert_eq!(entries[..9], state.entries);
    assert_eq!(entries[9].block, fixture.expected_entry(10, None, Some(3)));
    assert_eq!(
        entries[10].block,
        fixture.expected_entry(11, Some(h11[0].digest().unwrap()), Some(3))
    );
    let source = GlobalSource::Regional(key(&successor, &h11[0]));
    for entry in &entries[9..] {
        assert_eq!(entry.source, source);
        assert_eq!(entry.position, p);
    }
    assert_eq!(
        admitted.added_records[0].effects,
        [ReplayEffect::Append {
            first: 10,
            last: 11,
            source
        }]
    );
    println!(
        "SessionEnd selected h9 reference/genesis window [11,20]; at {p:?}, h10 padding and h11 physical admitted together"
    );
}
