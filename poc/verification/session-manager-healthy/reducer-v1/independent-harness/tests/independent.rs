//! Expected values below are manual consequences of the source-first matrix.
//! This is explicitly a trusted-input, non-cryptographic component harness.

use h2_reducer_independent::fixtures::*;
use poc_session_manager::*;
use poc_types::domain::*;
use poc_viewer_gfr::ReplayRecord;

fn binding(worker: u8) -> ManagerBinding {
    ManagerBinding {
        manager_id: h(0xa0 + worker),
        worker: pid(worker),
        public_config: config().reference().unwrap(),
        gfr_source: h(0xb0),
        preparation_version: 1,
    }
}

fn fresh(worker: u8) -> Manager {
    let cfg = config();
    Manager::new(
        binding(worker),
        cfg.clone(),
        VerifiedGenesis {
            descriptor: genesis(&cfg),
            source_ref: h(0x60),
        },
    )
    .unwrap()
}

fn install(m: &mut Manager, input: ManagerInput, now: u64) -> CommitStatus {
    match m.plan(input, now).unwrap() {
        PlanOutcome::Planned(d) => {
            let receipt = CommitRef::for_delta(&d, h(0xd0));
            m.accept_commit(d, receipt).unwrap()
        }
        PlanOutcome::Unchanged(status) => status,
        PlanOutcome::Wait(reason) => panic!("unexpected WAIT: {reason:?}"),
    }
}

fn delta(m: &Manager, input: ManagerInput, now: u64) -> ManagerDelta {
    match m.plan(input, now).unwrap() {
        PlanOutcome::Planned(d) => d,
        other => panic!("expected a proposal, got {other:?}"),
    }
}

fn snap(m: &Manager) -> String {
    let v = m.view();
    format!(
        "{:?}",
        (
            v.revision(),
            v.local_time_us(),
            v.cursor(),
            v.next_source_seq(),
            v.programs(),
            v.actions(),
            v.results(),
            v.guards(),
            v.faulted()
        )
    )
}

fn mark_clock(m: &mut Manager, generation: u64, now: u64) {
    install(
        m,
        ManagerInput::ClockReady {
            generation,
            basis_ref: h(0xc0 + generation as u8),
        },
        now,
    );
}

fn register(m: &mut Manager, predecessor: &Descriptor, d: &Descriptor, now: u64) {
    install(
        m,
        ManagerInput::RegisterCandidate(VerifiedCandidate {
            descriptor: d.clone(),
            predecessor: predecessor.cid,
            provenance_ref: h(0x61),
        }),
        now,
    );
}

fn source(seq: u64, record: ReplayRecord) -> ManagerInput {
    ManagerInput::ConsumeRecord(VerifiedReplayInput {
        source: h(0xb0),
        seq,
        record,
        validation_ref: h(0x62),
    })
}

fn recv(m: &mut Manager, id: ActionId, outcome: ReceiptOutcome, now: u64) {
    install(
        m,
        ManagerInput::Receipt(ActionReceipt {
            id,
            receipt_ref: h(0x63),
            outcome,
        }),
        now,
    );
}

fn target(kind: &ActionKind) -> Cid {
    match kind {
        ActionKind::Create { descriptor, .. } => descriptor.cid,
        ActionKind::Prepare { cid, .. }
        | ActionKind::Enter { cid, .. }
        | ActionKind::Pause { cid, .. }
        | ActionKind::SelectBack { cid, .. }
        | ActionKind::Close { cid, .. } => *cid,
        ActionKind::Publish { key, .. } => key.cid,
    }
}

fn name(kind: &ActionKind) -> &'static str {
    match kind {
        ActionKind::Create { .. } => "create",
        ActionKind::Prepare { .. } => "prepare",
        ActionKind::Enter { .. } => "enter",
        ActionKind::Pause { .. } => "pause",
        ActionKind::SelectBack { .. } => "back",
        ActionKind::Close { .. } => "close",
        ActionKind::Publish { .. } => "publish",
    }
}

fn actions(m: &Manager, cid: Cid, kind: &str) -> Vec<ActionIntent> {
    m.view()
        .actions()
        .values()
        .filter(|e| target(&e.intent.kind) == cid && name(&e.intent.kind) == kind)
        .map(|e| e.intent.clone())
        .collect()
}

fn one(m: &Manager, cid: Cid, kind: &str) -> ActionIntent {
    let values = actions(m, cid, kind);
    assert_eq!(values.len(), 1, "expected exactly one {kind} for {cid:?}");
    values[0].clone()
}

fn dispatches(m: &Manager, cid: Cid, kind: &str, now: u64) -> bool {
    m.dispatchable(now)
        .iter()
        .any(|a| target(&a.kind) == cid && name(&a.kind) == kind)
}

fn created(m: &mut Manager, cid: Cid, handle: u8, now: u64) {
    let id = one(m, cid, "create").id;
    recv(
        m,
        id,
        ReceiptOutcome::Created {
            instance_ref: h(handle),
        },
        now,
    );
}

fn prepared_input(m: &Manager, cid: Cid) -> PreparedRef {
    let a = one(m, cid, "prepare");
    match a.kind {
        ActionKind::Prepare {
            instance_ref,
            anchor,
            provenance_ref,
            validation_version,
            ..
        } => PreparedRef {
            prepare_action: a.id,
            worker: m.view().binding().worker,
            cid,
            instance_ref,
            anchor,
            config_ref: config().reference().unwrap(),
            validation_version,
            execution_ref: h(0x70),
            provenance_ref,
        },
        _ => unreachable!(),
    }
}

fn ready(m: &mut Manager, cid: Cid, handle: u8, now: u64) {
    created(m, cid, handle, now);
    let p = prepared_input(m, cid);
    install(m, ManagerInput::PreparationCompleted(p), now);
    assert_eq!(m.view().program(cid).unwrap().phase, Phase::Ready);
}

fn enter(m: &mut Manager, d: &Descriptor, now: u64) -> ActionId {
    install(m, ManagerInput::Schedule(d.plan.session.clone()), now);
    assert!(!m.view().program(d.cid).unwrap().entered);
    assert_eq!(m.view().program(d.cid).unwrap().phase, Phase::Entering);
    let id = one(m, d.cid, "enter").id;
    let instance_ref = m.view().program(d.cid).unwrap().instance_ref.unwrap();
    assert!(dispatches(m, d.cid, "enter", now));
    recv(
        m,
        id.clone(),
        ReceiptOutcome::Entered {
            instance_ref,
            entry_time_us: now,
        },
        now,
    );
    assert!(m.view().program(d.cid).unwrap().entered);
    assert_eq!(m.view().program(d.cid).unwrap().phase, Phase::Running);
    id
}

fn pair() -> (Manager, Descriptor, Descriptor, Descriptor) {
    let cfg = config();
    let predecessor = genesis(&cfg);
    let a = successor(&cfg, &predecessor, anchor(9, 0x19));
    let b = successor(&cfg, &predecessor, anchor(10, 0x1a));
    let mut m = fresh(5);
    mark_clock(&mut m, 1, 1);
    register(&mut m, &predecessor, &a, 2);
    register(&mut m, &predecessor, &b, 3);
    (m, predecessor, a, b)
}

#[test]
fn independent_unique_descriptor_create_and_preparation_binding() {
    let (mut m, predecessor, a, _) = pair();
    let id = one(&m, a.cid, "create").id;
    register(&mut m, &predecessor, &a, 3);
    assert_eq!(one(&m, a.cid, "create").id, id);
    let before = snap(&m);
    let mut bad = a.clone();
    bad.anchor.timestamp_us += 1;
    assert!(
        m.plan(
            ManagerInput::RegisterCandidate(VerifiedCandidate {
                descriptor: bad,
                predecessor: predecessor.cid,
                provenance_ref: h(0x61),
            }),
            3
        )
        .is_err()
    );
    assert_eq!(snap(&m), before);
    created(&mut m, a.cid, 0x81, 4);
    assert_eq!(m.view().program(a.cid).unwrap().phase, Phase::Preparing);
    assert!(!dispatches(&m, a.cid, "enter", 4));
    let p = prepared_input(&m, a.cid);
    for field in 0..7 {
        let mut bad = p.clone();
        match field {
            0 => bad.worker = pid(6),
            1 => bad.cid = Cid::new(h(0xfe)),
            2 => bad.anchor.block.block_id = h(0xfe),
            3 => bad.instance_ref = h(0xfe),
            4 => bad.validation_version += 1,
            5 => bad.config_ref = ConfigRef::new(h(0xfe)),
            _ => bad.provenance_ref = h(0xfe),
        }
        let s = snap(&m);
        assert!(m.plan(ManagerInput::PreparationCompleted(bad), 4).is_err());
        assert_eq!(snap(&m), s);
    }
    install(&mut m, ManagerInput::PreparationCompleted(p.clone()), 5);
    let rev = m.view().revision();
    install(&mut m, ManagerInput::PreparationCompleted(p), 5);
    assert_eq!(m.view().revision(), rev);
    assert_eq!(m.view().program(a.cid).unwrap().phase, Phase::Ready);
    println!("PASS R01-R05 trusted-input binding/uniqueness");
}

#[test]
fn independent_higher_preparing_then_ready_and_original_select_back() {
    let (mut m, predecessor, a, b) = pair();
    ready(&mut m, a.cid, 0x81, 4);
    let original_enter = enter(&mut m, &a, 5);
    install(&mut m, ManagerInput::Schedule(a.plan.session.clone()), 6);
    assert_eq!(m.view().scheduled(&a.plan.session), Some(a.cid));
    ready(&mut m, b.cid, 0x82, 7);
    install(&mut m, ManagerInput::Schedule(a.plan.session.clone()), 8);
    assert!(dispatches(&m, a.cid, "pause", 8));
    assert!(!dispatches(&m, b.cid, "enter", 8));
    let park_id = one(&m, a.cid, "pause").id;
    recv(
        &mut m,
        park_id,
        ReceiptOutcome::Paused {
            instance_ref: h(0x81),
        },
        9,
    );
    assert!(dispatches(&m, b.cid, "enter", 9));
    let enter_b_id = one(&m, b.cid, "enter").id;
    recv(
        &mut m,
        enter_b_id,
        ReceiptOutcome::Entered {
            instance_ref: h(0x82),
            entry_time_us: 10,
        },
        10,
    );
    let rec = session_end(
        24_000_000,
        predecessor.plan.session.clone(),
        vec![closed(&predecessor.plan.session), assigned(&a)],
    );
    install(&mut m, source(0, rec), 11);
    assert_eq!(
        m.view().program(b.cid).unwrap().selection,
        Selection::Excluded
    );
    assert_eq!(m.view().program(b.cid).unwrap().phase, Phase::Terminated);
    install(&mut m, ManagerInput::Schedule(a.plan.session.clone()), 12);
    assert!(!dispatches(&m, a.cid, "back", 12));
    let close_b_id = one(&m, b.cid, "close").id;
    recv(
        &mut m,
        close_b_id,
        ReceiptOutcome::Closed {
            instance_ref: Some(h(0x82)),
        },
        13,
    );
    assert!(dispatches(&m, a.cid, "back", 13));
    let back_a_id = one(&m, a.cid, "back").id;
    recv(
        &mut m,
        back_a_id,
        ReceiptOutcome::SelectedBack {
            instance_ref: h(0x81),
        },
        14,
    );
    let view = m.view();
    let p = view.program(a.cid).unwrap();
    assert_eq!(p.instance_ref, Some(h(0x81)));
    assert_eq!(p.entry_time_us, Some(5));
    assert_eq!(one(&m, a.cid, "enter").id, original_enter);
    assert_eq!(p.phase, Phase::Running);
    println!("PASS R06 R08 R10 original-reference candidate switch");
}

#[test]
fn independent_unconfirmed_enter_requires_parking_before_other_candidate() {
    let (mut m, _, a, b) = pair();
    ready(&mut m, a.cid, 0x81, 4);
    install(&mut m, ManagerInput::Schedule(a.plan.session.clone()), 5);
    let pending_enter = one(&m, a.cid, "enter").id;
    assert!(!m.view().program(a.cid).unwrap().entered);
    ready(&mut m, b.cid, 0x82, 6);
    install(&mut m, ManagerInput::Schedule(a.plan.session.clone()), 7);
    assert!(!dispatches(&m, a.cid, "enter", 7));
    assert!(!dispatches(&m, b.cid, "enter", 7));
    let park = one(&m, a.cid, "pause");
    recv(
        &mut m,
        park.id,
        ReceiptOutcome::Paused {
            instance_ref: h(0x81),
        },
        8,
    );
    assert!(!m.view().program(a.cid).unwrap().entered);
    assert_ne!(m.view().program(a.cid).unwrap().phase, Phase::Paused);
    assert!(dispatches(&m, b.cid, "enter", 8));
    recv(
        &mut m,
        pending_enter.clone(),
        ReceiptOutcome::Entered {
            instance_ref: h(0x81),
            entry_time_us: 5,
        },
        9,
    );
    assert!(m.view().program(a.cid).unwrap().entered);
    assert_ne!(m.view().program(a.cid).unwrap().phase, Phase::Running);
    assert_eq!(m.view().scheduled(&a.plan.session), Some(b.cid));
    assert_eq!(one(&m, a.cid, "enter").id, pending_enter);
    println!("PASS CR2 R23 R26 unconfirmed entry parked before switch");
}

#[test]
fn independent_record_atomicity_cursor_identity_and_empty_effects() {
    let (mut m, predecessor, a, b) = pair();
    ready(&mut m, b.cid, 0x82, 4);
    enter(&mut m, &b, 5);
    let rec = session_end(
        24_000_000,
        predecessor.plan.session.clone(),
        vec![closed(&predecessor.plan.session), assigned(&a)],
    );
    let before = snap(&m);
    let mut bad = rec.clone();
    bad.events[1].position.key_us += 1;
    assert!(m.plan(source(0, bad), 6).is_err());
    assert_eq!(snap(&m), before);
    assert!(matches!(
        m.plan(source(1, rec.clone()), 6).unwrap(),
        PlanOutcome::Wait(_)
    ));
    assert_eq!(snap(&m), before);
    install(&mut m, source(0, rec.clone()), 6);
    assert_eq!(m.view().next_source_seq(), 1);
    assert_eq!(m.view().cursor(), Some(&rec.position));
    assert_eq!(
        m.view().program(a.cid).unwrap().selection,
        Selection::Selected
    );
    assert!(!m.view().program(a.cid).unwrap().entered);
    assert_eq!(m.view().program(b.cid).unwrap().phase, Phase::Terminated);
    let original = snap(&m);
    install(&mut m, source(0, rec.clone()), 6);
    assert_eq!(snap(&m), original);
    let mut changed = rec.clone();
    changed.effects.push(append(10, 10, predecessor.cid));
    assert!(m.plan(source(0, changed), 6).is_err());
    assert_eq!(snap(&m), original);
    let empty = record(25_000_000, 9, vec![], vec![]);
    install(&mut m, source(1, empty.clone()), 7);
    assert_eq!(m.view().next_source_seq(), 2);
    assert_eq!(m.view().cursor(), Some(&empty.position));
    let after = snap(&m);
    install(&mut m, source(0, rec), 7);
    assert_eq!(
        snap(&m),
        after,
        "older exact replay must not rewind or allocate"
    );
    println!("PASS R14-R17 full-record identity/atomicity/cursor");
}

#[test]
fn independent_same_pc_multiple_assignments_do_not_start_intermediate_session() {
    let cfg = config();
    let pred = genesis(&cfg);
    let a = successor(&cfg, &pred, anchor(10, 0x1a));
    let c = successor(&cfg, &a, anchor(20, 0x24));
    let mut m = fresh(5);
    let rec = record(
        35_000_000,
        20,
        vec![
            closed(&pred.plan.session),
            assigned(&a),
            closed(&a.plan.session),
            assigned(&c),
        ],
        vec![],
    );
    install(&mut m, source(0, rec.clone()), 1);
    assert_eq!(m.view().records()[0].lifecycle_ids.len(), 4);
    assert!(actions(&m, a.cid, "create").is_empty());
    assert!(actions(&m, a.cid, "enter").is_empty());
    assert_eq!(
        m.view().guards().closed_session(&a.plan.session),
        Some(CloseReason::Complete)
    );
    assert_eq!(m.view().selected(&c.plan.session), Some(&c));
    assert_eq!(m.view().cursor(), Some(&rec.position));
    println!("PASS R14 multiple same-pc lifecycle bodies preserve final closed state");
}

#[test]
fn independent_append_tip_excludes_lower_and_blocks_late_candidate() {
    let cfg = config();
    let pred = genesis(&cfg);
    let a7 = successor(&cfg, &pred, anchor(7, 0x17));
    let a8 = successor(&cfg, &pred, anchor(8, 0x18));
    let mut m = fresh(5);
    register(&mut m, &pred, &a7, 1);
    register(&mut m, &pred, &a8, 2);
    assert_ne!(
        m.view().program(a7.cid).unwrap().selection,
        Selection::Excluded,
        "regional higher candidate alone cannot publicly kill lower candidate"
    );
    install(
        &mut m,
        source(
            0,
            record(19_000_000, 7, vec![], vec![append(1, 8, pred.cid)]),
        ),
        3,
    );
    assert_eq!(m.view().program(a7.cid).unwrap().phase, Phase::Terminated);
    assert_eq!(
        m.view().program(a7.cid).unwrap().selection,
        Selection::Excluded
    );
    assert_eq!(m.view().guards().physical_tip(pred.cid), Some(8));
    assert_ne!(m.view().program(a8.cid).unwrap().phase, Phase::Terminated);
    let a6 = successor(&cfg, &pred, anchor(6, 0x16));
    let input = ManagerInput::RegisterCandidate(VerifiedCandidate {
        descriptor: a6.clone(),
        predecessor: pred.cid,
        provenance_ref: h(0x61),
    });
    match m.plan(input, 4) {
        Err(_) | Ok(PlanOutcome::Wait(_)) | Ok(PlanOutcome::Unchanged(_)) => (),
        Ok(PlanOutcome::Planned(d)) => {
            let r = CommitRef::for_delta(&d, h(0xd0));
            m.accept_commit(d, r).unwrap();
        }
    }
    assert!(
        m.view()
            .program(a6.cid)
            .is_none_or(|p| p.phase == Phase::Terminated)
    );
    assert!(actions(&m, a6.cid, "create").is_empty());
    println!("PASS R30 CR1 predecessor physical-tip barrier and late registration");
}

#[test]
fn independent_closed_scope_late_ready_and_selected_terminal() {
    let (mut m, predecessor, a, b) = pair();
    created(&mut m, a.cid, 0x81, 4);
    let completion = prepared_input(&m, a.cid);
    install(&mut m, source(0, expiry(41_000_000, 1, 0)), 5);
    assert_eq!(
        m.view().guards().closed_branch(0),
        Some(CloseReason::Expired)
    );
    assert_eq!(m.view().program(a.cid).unwrap().phase, Phase::Terminated);
    assert_eq!(m.view().program(b.cid).unwrap().phase, Phase::Terminated);
    install(&mut m, ManagerInput::PreparationCompleted(completion), 6);
    assert_eq!(m.view().program(a.cid).unwrap().phase, Phase::Terminated);
    let later = successor(&config(), &predecessor, anchor(8, 0x18));
    let input = ManagerInput::RegisterCandidate(VerifiedCandidate {
        descriptor: later.clone(),
        predecessor: predecessor.cid,
        provenance_ref: h(0x61),
    });
    if let Ok(PlanOutcome::Planned(d)) = m.plan(input, 7) {
        assert!(
            !d.intents()
                .iter()
                .any(|intent| target(&intent.kind) == later.cid
                    && matches!(
                        intent.kind,
                        ActionKind::Create { .. } | ActionKind::Enter { .. }
                    ))
        );
        assert!(
            d.view()
                .program(later.cid)
                .is_none_or(|p| p.phase == Phase::Terminated)
        );
    }
    assert!(!dispatches(&m, later.cid, "create", 7));
    println!("PASS R12-R13 late material cannot reopen closed branch");
}

#[test]
fn independent_commit_cause_time_identity_and_stale_replay() {
    let mut m = fresh(0);
    let before = snap(&m);
    let input = ManagerInput::ClockReady {
        generation: 1,
        basis_ref: h(0xc1),
    };
    let first = delta(&m, input.clone(), 1);
    let repeated = delta(&m, input.clone(), 1);
    let stale = delta(&m, ManagerInput::ObserveTime, 21_000_000);
    assert_eq!(first.intents(), repeated.intents());
    assert_eq!(snap(&m), before, "plan is immutable and inert");
    let other_time = delta(&m, input, 2);
    let wrong = CommitRef::for_delta(&first, h(0xd0));
    assert!(m.accept_commit(other_time, wrong).is_err());
    assert_eq!(snap(&m), before);
    let receipt = CommitRef::for_delta(&first, h(0xd0));
    m.accept_commit(first, receipt).unwrap();
    let accepted = snap(&m);
    let duplicate = CommitRef::for_delta(&repeated, h(0xd0));
    m.accept_commit(repeated, duplicate).unwrap();
    assert_eq!(snap(&m), accepted);
    let current = snap(&m);
    let r = CommitRef::for_delta(&stale, h(0xd0));
    assert!(m.accept_commit(stale, r).is_err());
    assert_eq!(snap(&m), current);
    println!("PASS R18 R20-R22 CR3 exact cause/time and at-most-once installation");
}

#[test]
fn independent_fault_latch_keeps_guard_and_never_installs_cursor() {
    let (mut m, _, a, _) = pair();
    let rec = expiry(50_000_000, 1, 0);
    let before_plan = snap(&m);
    let d = delta(&m, source(0, rec.clone()), 5);
    let repeated = delta(&m, source(0, rec), 5);
    assert_eq!(d.intents(), repeated.intents());
    assert_eq!(snap(&m), before_plan);
    let close_ids: Vec<_> = d
        .intents()
        .iter()
        .filter(|a| matches!(a.kind, ActionKind::Close { .. }))
        .map(|a| a.id)
        .collect();
    assert_eq!(close_ids.len(), 2);
    assert_ne!(close_ids[0], close_ids[1]);
    assert!(
        close_ids
            .iter()
            .all(|id| id.origin_revision == d.target_revision())
    );
    let mut foreign = fresh(6);
    let foreign_before = snap(&foreign);
    assert!(foreign.latch_commit_failure(&d, h(0xf0)).is_err());
    assert_eq!(snap(&foreign), foreign_before);
    assert_eq!(m.view().next_source_seq(), 0);
    m.latch_commit_failure(&d, h(0xf0)).unwrap();
    assert!(m.view().faulted().is_some());
    assert_eq!(m.view().next_source_seq(), 0);
    assert!(m.view().cursor().is_none());
    assert_eq!(
        m.view().guards().closed_branch(0),
        Some(CloseReason::Expired)
    );
    assert_eq!(m.view().program(a.cid).unwrap().phase, Phase::Terminated);
    assert!(!dispatches(&m, a.cid, "enter", 6));
    assert!(
        m.plan(
            ManagerInput::ClockReady {
                generation: 2,
                basis_ref: h(0xc2)
            },
            6
        )
        .is_err()
    );
    let before = snap(&m);
    m.latch_commit_failure(&d, h(0xf0)).unwrap();
    assert_eq!(snap(&m), before);
    let receipt = CommitRef::for_delta(&d, h(0xd0));
    assert!(m.accept_commit(d, receipt).is_err());
    assert_eq!(snap(&m), before);
    println!("PASS R19 fail-closed guard retained with unaccepted public cursor");
}

#[test]
fn independent_typed_action_ids_match_fixed_committed_coordinates() {
    let mut m = fresh(0);
    let d = genesis(&config());
    let create = one(&m, d.cid, "create");
    assert_eq!(
        create.id,
        ActionId {
            manager_id: h(0xa0),
            worker: pid(0),
            origin_revision: 0,
            ordinal: 0
        }
    );
    created(&mut m, d.cid, 0x81, 1);
    let prepare = one(&m, d.cid, "prepare");
    assert_eq!(
        prepare.id,
        ActionId {
            manager_id: h(0xa0),
            worker: pid(0),
            origin_revision: 1,
            ordinal: 0
        }
    );
    assert_ne!(create.id, prepare.id);
    let foreign = fresh(1);
    let other_create = one(&foreign, d.cid, "create");
    assert_ne!(create.id, other_create.id);
    assert_eq!(other_create.id.worker, pid(1));
    assert_eq!(other_create.id.origin_revision, 0);
    println!("PASS R21-R22 fixed typed ActionId coordinates and worker isolation");
}

#[test]
fn independent_public_source_mutations_and_local_clock_monotonicity() {
    let cfg = config();
    let pred = genesis(&cfg);
    let a = successor(&cfg, &pred, anchor(9, 0x19));
    let mut m = fresh(5);
    mark_clock(&mut m, 1, 100);
    let s = snap(&m);
    assert!(m.plan(ManagerInput::ObserveTime, 99).is_err());
    assert!(
        m.plan(
            ManagerInput::ClockReady {
                generation: 0,
                basis_ref: h(0xc0)
            },
            100
        )
        .is_err()
    );
    assert!(
        m.plan(
            ManagerInput::ClockReady {
                generation: 1,
                basis_ref: h(0xcf)
            },
            100
        )
        .is_err()
    );
    assert_eq!(snap(&m), s);
    let rec = record(
        24_000_000,
        10,
        vec![closed(&pred.plan.session), assigned(&a)],
        vec![],
    );
    for mutation in 0..4 {
        let mut bad = VerifiedReplayInput {
            source: h(0xb0),
            seq: 0,
            record: rec.clone(),
            validation_ref: h(0x62),
        };
        match mutation {
            0 => bad.source = h(0xbf),
            1 => bad.record.input_ref = poc_viewer_gfr::InputRef::Espresso(11),
            2 => bad.record.events[0].position.key_us += 1,
            _ => bad.record.position.index = ReplayIndex::Height(10),
        }
        assert!(m.plan(ManagerInput::ConsumeRecord(bad), 100).is_err());
        assert_eq!(snap(&m), s);
    }
    install(&mut m, source(0, rec.clone()), 100);
    assert_eq!(m.view().selected(&a.plan.session), Some(&a));
    let next = record(24_000_000, 11, vec![], vec![]);
    install(&mut m, source(1, next), 101);
    assert_eq!(m.view().next_source_seq(), 2);
    println!("PASS R16-R17 R27 R28 source provenance binding and monotonic local clock");
}

#[test]
fn independent_wrong_binding_predecessor_conflicting_anchor_and_window() {
    let cfg = config();
    let pred = genesis(&cfg);
    let mut bad_binding = binding(5);
    bad_binding.public_config = ConfigRef::new(h(0xff));
    assert!(
        Manager::new(
            bad_binding,
            cfg.clone(),
            VerifiedGenesis {
                descriptor: pred.clone(),
                source_ref: h(0x60),
            }
        )
        .is_err()
    );
    let mut m = fresh(5);
    let a = successor(&cfg, &pred, anchor(9, 0x19));
    register(&mut m, &pred, &a, 1);
    let before = snap(&m);
    let conflicting = successor(&cfg, &pred, anchor(9, 0x29));
    assert!(
        m.plan(
            ManagerInput::RegisterCandidate(VerifiedCandidate {
                descriptor: conflicting,
                predecessor: pred.cid,
                provenance_ref: h(0x61),
            }),
            1
        )
        .is_err()
    );
    assert_eq!(snap(&m), before);
    assert!(
        m.plan(
            ManagerInput::RegisterCandidate(VerifiedCandidate {
                descriptor: a.clone(),
                predecessor: Cid::new(h(0xff)),
                provenance_ref: h(0x61),
            }),
            1
        )
        .is_err()
    );
    let recursive = successor(&cfg, &a, anchor(19, 0x23));
    assert!(
        m.plan(
            ManagerInput::RegisterCandidate(VerifiedCandidate {
                descriptor: recursive,
                predecessor: a.cid,
                provenance_ref: h(0x61),
            }),
            1
        )
        .is_err()
    );
    let mut shifted_plan = a.plan.clone();
    shifted_plan.window = Window {
        h_min: 12,
        h_max: 21,
    };
    let shifted = descriptor(&cfg, shifted_plan, a.anchor.clone());
    assert!(
        m.plan(
            ManagerInput::RegisterCandidate(VerifiedCandidate {
                descriptor: shifted,
                predecessor: pred.cid,
                provenance_ref: h(0x61),
            }),
            1
        )
        .is_err()
    );
    assert_eq!(snap(&m), before);
    println!("PASS R02 R07 R31 R36 binding/lineage/fixed-window checks");
}

#[test]
fn independent_public_selects_terminal_a_still_excludes_b() {
    let (mut m, pred, a, b) = pair();
    install(&mut m, ManagerInput::ObserveTime, 31_000_000);
    assert_eq!(m.view().program(a.cid).unwrap().phase, Phase::Terminated);
    assert_eq!(
        m.view().program(b.cid).unwrap().selection,
        Selection::Candidate
    );
    install(
        &mut m,
        source(
            0,
            session_end(
                24_000_000,
                pred.plan.session.clone(),
                vec![closed(&pred.plan.session), assigned(&a)],
            ),
        ),
        31_000_000,
    );
    assert_eq!(
        m.view().program(a.cid).unwrap().selection,
        Selection::Selected
    );
    assert_eq!(m.view().program(a.cid).unwrap().phase, Phase::Terminated);
    assert_eq!(
        m.view().program(b.cid).unwrap().selection,
        Selection::Excluded
    );
    assert!(!dispatches(&m, a.cid, "enter", 31_000_000));
    println!("PASS R09 selected terminal A does not prevent public exclusion of B");
}

#[test]
fn independent_cancelled_start_late_success_and_old_failure_cannot_reopen() {
    let (mut m, _, a, _) = pair();
    ready(&mut m, a.cid, 0x81, 4);
    install(&mut m, ManagerInput::Schedule(a.plan.session.clone()), 5);
    let pending = one(&m, a.cid, "enter").id;
    install(&mut m, source(0, expiry(41_000_000, 1, 0)), 6);
    assert_eq!(m.view().program(a.cid).unwrap().phase, Phase::Terminated);
    assert!(!dispatches(&m, a.cid, "enter", 6));
    let before = snap(&m);
    recv(
        &mut m,
        pending,
        ReceiptOutcome::Retryable {
            failure_ref: h(0xf1),
        },
        6,
    );
    assert_eq!(snap(&m), before);
    recv(
        &mut m,
        pending,
        ReceiptOutcome::Entered {
            instance_ref: h(0x81),
            entry_time_us: 5,
        },
        7,
    );
    let p = m.view().program(a.cid).unwrap();
    assert!(p.entered);
    assert_eq!(p.entry_time_us, Some(5));
    assert_eq!(p.phase, Phase::Terminated);
    assert_eq!(
        p.terminal_cause,
        Some(TerminalCause::Public(CloseReason::Expired))
    );
    assert!(!dispatches(&m, a.cid, "enter", 7));
    let mut unknown = pending;
    unknown.ordinal = 60_000;
    let current = snap(&m);
    assert!(
        m.plan(
            ManagerInput::Receipt(ActionReceipt {
                id: unknown,
                receipt_ref: h(0x63),
                outcome: ReceiptOutcome::Entered {
                    instance_ref: h(0x81),
                    entry_time_us: 5
                }
            }),
            7
        )
        .is_err()
    );
    assert_eq!(snap(&m), current);
    println!("PASS R24-R26 cancelled start remains closed under late receipt/failure");
}

#[test]
fn independent_fixed_history_is_deterministic_and_clockunsafe_recovery_fence_holds() {
    let cfg = config();
    let pred = genesis(&cfg);
    let a = successor(&cfg, &pred, anchor(9, 0x19));
    let mut left = fresh(5);
    let mut right = fresh(5);
    for m in [&mut left, &mut right] {
        mark_clock(m, 1, 1);
        register(m, &pred, &a, 2);
        ready(m, a.cid, 0x81, 3);
        enter(m, &a, 4);
    }
    assert_eq!(snap(&left), snap(&right));
    install(
        &mut right,
        ManagerInput::ClockReady {
            generation: 1,
            basis_ref: h(0xc1),
        },
        4,
    );
    assert_eq!(snap(&left), snap(&right));
    install(
        &mut left,
        ManagerInput::ClockUnsafe { basis_ref: h(0xf0) },
        4,
    );
    mark_clock(&mut left, 2, 5);
    let other_anchor = successor(&cfg, &pred, anchor(10, 0x1a));
    let request = ManagerInput::RegisterCandidate(VerifiedCandidate {
        descriptor: other_anchor.clone(),
        predecessor: pred.cid,
        provenance_ref: h(0x61),
    });
    if let Ok(PlanOutcome::Planned(d)) = left.plan(request, 6) {
        let receipt = CommitRef::for_delta(&d, h(0xd0));
        left.accept_commit(d, receipt).unwrap();
    }
    assert_eq!(left.view().guards().unsafe_session_fence(), Some((0, 1)));
    assert!(!dispatches(&left, other_anchor.cid, "enter", 6));
    assert!(!dispatches(&left, a.cid, "enter", 6));
    assert_eq!(left.view().program(a.cid).unwrap().phase, Phase::Terminated);
    println!("PASS R28 R32 duplicate-clock replay determinism and no same-session fence bypass");
}

#[test]
fn independent_source_record_capacity_boundary() {
    let mut m = fresh(5);
    for seq in 0..2048u64 {
        install(
            &mut m,
            source(seq, record(10_000_000 + seq, seq, vec![], vec![])),
            1,
        );
    }
    assert_eq!(m.view().next_source_seq(), 2048);
    let before = snap(&m);
    assert!(matches!(
        m.plan(source(2048, record(10_002_048, 2048, vec![], vec![])), 1),
        Err(ManagerError::Capacity { .. })
    ));
    assert_eq!(snap(&m), before);
    println!("PASS R33 source-record exact2048/+1 capacity, no silent pruning");
}

#[test]
fn independent_session_scope_capacity_boundary() {
    let mut m = fresh(5);
    install(
        &mut m,
        source(
            0,
            record(
                20_000_000,
                0,
                (1..=64).map(|r| closed(&sid(0, r))).collect(),
                vec![],
            ),
        ),
        1,
    );
    install(
        &mut m,
        source(
            1,
            record(
                20_000_001,
                1,
                (65..=127).map(|r| closed(&sid(0, r))).collect(),
                vec![],
            ),
        ),
        1,
    );
    assert_eq!(m.view().next_source_seq(), 2);
    let before = snap(&m);
    assert!(matches!(
        m.plan(
            source(2, record(20_000_002, 2, vec![closed(&sid(0, 128))], vec![])),
            1
        ),
        Err(ManagerError::Capacity { .. })
    ));
    assert_eq!(snap(&m), before);
    println!("PASS R33 exact128/+1 combined selected/closed session capacity");
}

#[test]
fn independent_result_task_capacity_and_duplicate_identity() {
    let mut m = fresh(0);
    let cid = genesis(&config()).cid;
    for byte in 0..=255u8 {
        install(
            &mut m,
            ManagerInput::RetainResult {
                key: ResultKey {
                    cid,
                    block_hash: h(byte),
                },
                verified_result_ref: h(byte),
            },
            1,
        );
    }
    assert_eq!(m.view().results().len(), 256);
    let before = snap(&m);
    install(
        &mut m,
        ManagerInput::RetainResult {
            key: ResultKey {
                cid,
                block_hash: h(1),
            },
            verified_result_ref: h(1),
        },
        1,
    );
    assert_eq!(snap(&m), before);
    let mut distinct = [0u8; 32];
    distinct[31] = 1;
    assert!(matches!(
        m.plan(
            ManagerInput::RetainResult {
                key: ResultKey {
                    cid,
                    block_hash: Hash32::new(distinct)
                },
                verified_result_ref: h(0xee),
            },
            1
        ),
        Err(ManagerError::Capacity { .. })
    ));
    assert_eq!(snap(&m), before);
    println!("PASS R29 R33 exact256/+1 result tasks and stable duplicate identity");
}

#[test]
fn independent_receipts_are_correlated_monotonic_and_never_retry_after_success() {
    let mut m = fresh(0);
    let d = genesis(&config());
    let create = one(&m, d.cid, "create");
    recv(
        &mut m,
        create.id.clone(),
        ReceiptOutcome::Retryable {
            failure_ref: h(0xf1),
        },
        1,
    );
    assert_eq!(
        m.view().action(create.id.clone()).unwrap().status,
        ActionStatus::Pending
    );
    recv(
        &mut m,
        create.id.clone(),
        ReceiptOutcome::Created {
            instance_ref: h(0x81),
        },
        2,
    );
    assert_eq!(
        m.view().action(create.id.clone()).unwrap().status,
        ActionStatus::Applied
    );
    let s = snap(&m);
    recv(
        &mut m,
        create.id.clone(),
        ReceiptOutcome::Created {
            instance_ref: h(0x81),
        },
        2,
    );
    assert_eq!(snap(&m), s);
    recv(
        &mut m,
        create.id.clone(),
        ReceiptOutcome::Retryable {
            failure_ref: h(0xf2),
        },
        2,
    );
    assert_eq!(snap(&m), s, "stale retry cannot undo completed action");
    let bad = ManagerInput::Receipt(ActionReceipt {
        id: create.id.clone(),
        receipt_ref: h(0x63),
        outcome: ReceiptOutcome::Created {
            instance_ref: h(0x82),
        },
    });
    assert!(m.plan(bad, 2).is_err());
    assert_eq!(snap(&m), s);
    let wrong_kind = ManagerInput::Receipt(ActionReceipt {
        id: create.id.clone(),
        receipt_ref: h(0x63),
        outcome: ReceiptOutcome::Published {
            key: ResultKey {
                cid: d.cid,
                block_hash: h(0x91),
            },
        },
    });
    assert!(m.plan(wrong_kind, 2).is_err());
    assert_eq!(one(&m, d.cid, "create").id, create.id);
    assert_eq!(actions(&m, d.cid, "prepare").len(), 1);
    println!("PASS R23-R25 stable receipt/action ledger");
}

#[test]
fn independent_local_stop_and_clock_unsafe_preserve_public_state_and_tasks() {
    let (mut m, _, a, b) = pair();
    ready(&mut m, a.cid, 0x81, 4);
    enter(&mut m, &a, 5);
    let key = ResultKey {
        cid: a.cid,
        block_hash: h(0x91),
    };
    install(
        &mut m,
        ManagerInput::RetainResult {
            key,
            verified_result_ref: h(0x92),
        },
        6,
    );
    let publication = one(&m, a.cid, "publish").id;
    install(&mut m, ManagerInput::ClockUnsafe { basis_ref: h(0xf3) }, 6);
    assert_eq!(m.view().program(a.cid).unwrap().phase, Phase::Terminated);
    assert_eq!(m.view().program(b.cid).unwrap().phase, Phase::Terminated);
    assert!(matches!(
        m.view().program(a.cid).unwrap().terminal_cause,
        Some(TerminalCause::LocalClockUnsafe { .. })
    ));
    assert_eq!(m.view().next_source_seq(), 0);
    assert!(m.view().cursor().is_none());
    assert!(!m.view().guards().clock_enabled());
    assert_eq!(m.view().guards().unsafe_session_fence(), Some((0, 1)));
    assert_eq!(one(&m, a.cid, "publish").id, publication);
    mark_clock(&mut m, 2, 7);
    assert_eq!(m.view().program(a.cid).unwrap().phase, Phase::Terminated);
    assert!(!dispatches(&m, a.cid, "back", 7));
    assert!(!dispatches(&m, b.cid, "enter", 7));

    let mut t = fresh(0);
    mark_clock(&mut t, 1, 1);
    let initial = genesis(&config());
    ready(&mut t, initial.cid, 0x83, 2);
    install(
        &mut t,
        ManagerInput::Schedule(initial.plan.session.clone()),
        20_999_999,
    );
    assert!(dispatches(&t, initial.cid, "enter", 20_999_999));
    assert!(!dispatches(&t, initial.cid, "enter", 21_000_000));
    install(&mut t, ManagerInput::ObserveTime, 21_000_000);
    assert_eq!(
        t.view().program(initial.cid).unwrap().phase,
        Phase::Terminated
    );
    assert_eq!(
        t.view().program(initial.cid).unwrap().terminal_cause,
        Some(TerminalCause::LocalTime)
    );
    assert_eq!(t.view().next_source_seq(), 0);
    assert!(t.view().cursor().is_none());
    println!("PASS R27-R29 local time/unsafe do not forge public closure or erase tasks");
}

#[test]
fn independent_early_h11_reference_and_late_legal_h14_are_separate() {
    for (entry_time, early) in [(21_020_000, true), (24_020_000, false)] {
        let cfg = config();
        let pred = genesis(&cfg);
        let a = successor(&cfg, &pred, anchor(9, 0x19));
        let mut m = fresh(5);
        mark_clock(&mut m, 1, 1);
        register(&mut m, &pred, &a, 2);
        let assignment = session_end(
            24_000_000,
            pred.plan.session.clone(),
            vec![closed(&pred.plan.session), assigned(&a)],
        );
        if !early {
            install(&mut m, source(0, assignment.clone()), 24_000_000);
        }
        ready(&mut m, a.cid, 0x81, entry_time - 1);
        let original = enter(&mut m, &a, entry_time);
        let result_ref = if early { h(0xe1) } else { h(0xe4) };
        install(
            &mut m,
            ManagerInput::RetainResult {
                key: ResultKey {
                    cid: a.cid,
                    block_hash: result_ref,
                },
                verified_result_ref: result_ref,
            },
            entry_time,
        );
        if early {
            install(&mut m, source(0, assignment), 24_100_000);
        }
        assert_eq!(
            m.view().program(a.cid).unwrap().descriptor.plan.window,
            Window {
                h_min: 11,
                h_max: 20
            }
        );
        assert_eq!(
            m.view().program(a.cid).unwrap().entry_time_us,
            Some(entry_time)
        );
        assert_eq!(one(&m, a.cid, "enter").id, original);
        assert_eq!(actions(&m, a.cid, "publish").len(), 1);
        println!(
            "PASS {} fixture only: production/firstSeen/Final/G admission not exercised",
            if early {
                "R34 early-h11 preserved reference"
            } else {
                "R35 late-legal-h14 allowed"
            }
        );
    }
}

#[test]
fn independent_record_limits_and_unsupported_inputs_reject_without_partial_state() {
    let cfg = config();
    let initial = genesis(&cfg);
    let mut m = fresh(5);
    let before = snap(&m);
    let p = espresso_position(20_000_000, 7);
    let rec = record(
        20_000_000,
        7,
        (0..65).map(|_| closed(&initial.plan.session)).collect(),
        vec![],
    );
    assert!(m.plan(source(0, rec), 1).is_err());
    assert_eq!(snap(&m), before);
    let mut bad = record(20_000_000, 7, vec![], vec![]);
    bad.position = ReplayPosition {
        key_us: p.key_us,
        kind: ReplayKind::Resume,
        index: ReplayIndex::L1(L1Position {
            block: 1,
            tx: 0,
            log: 0,
        }),
    };
    assert!(m.plan(source(0, bad), 1).is_err());
    assert_eq!(snap(&m), before);
    let bounded = record(
        20_000_000,
        7,
        (0..64).map(|i| closed(&sid(0, 100 + i))).collect(),
        vec![],
    );
    install(&mut m, source(0, bounded), 1);
    assert_eq!(m.view().next_source_seq(), 1);
    println!("PASS R31 R33 bounded lifecycle list and unsupported recovery");
}
