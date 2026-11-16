//! Trusted-input fixture. Uses the M0 public object types and the existing config and key fixture.
use poc_session_manager::*;
use poc_types::domain::*;
use poc_viewer_gfr::{GlobalSource, InputRef, ReplayEffect, ReplayRecord, Tail};

#[path = "../../../verification/mock-plan-execution/implementation-v1/component_fixture.rs"]
mod component_fixture;

const SEC: u64 = 1_000_000;

fn hash(byte: u8) -> Hash32 {
    Hash32::new([byte; 32])
}

struct Fixture {
    config: PublicConfig,
    genesis: Descriptor,
    worker: Pid,
    source: Hash32,
}

impl Fixture {
    fn new(worker: u8) -> Self {
        let config = component_fixture::config();
        let genesis = descriptor(&config, None, config.genesis.clone());
        Self {
            config,
            genesis,
            worker: Pid::new(worker).unwrap(),
            source: hash(24),
        }
    }

    fn manager(&self) -> Manager {
        Manager::new(
            ManagerBinding {
                manager_id: hash(231),
                worker: self.worker,
                public_config: self.config.reference().unwrap(),
                gfr_source: self.source,
                preparation_version: 1,
            },
            self.config.clone(),
            VerifiedGenesis {
                descriptor: self.genesis.clone(),
                source_ref: hash(230),
            },
        )
        .unwrap()
    }

    fn successor(&self, previous: &Descriptor, height: u64) -> Descriptor {
        let anchor = Anchor {
            block: BlockRef {
                height,
                origin: previous.anchor.block.origin,
                block_id: hash(u8::try_from(height).unwrap()),
            },
            timestamp_us: self.config.genesis.timestamp_us + height * SEC,
        };
        descriptor(&self.config, Some(previous), anchor)
    }

    fn record(&self, seq: u64, replay: ReplayRecord) -> ManagerInput {
        ManagerInput::ConsumeRecord(VerifiedReplayInput {
            source: self.source,
            seq,
            record: replay,
            validation_ref: hash(25),
        })
    }

    fn candidate(&self, descriptor: &Descriptor) -> ManagerInput {
        ManagerInput::RegisterCandidate(VerifiedCandidate {
            descriptor: descriptor.clone(),
            predecessor: self.genesis.cid,
            provenance_ref: hash(26),
        })
    }
}

fn descriptor(config: &PublicConfig, previous: Option<&Descriptor>, anchor: Anchor) -> Descriptor {
    let (r, end, cause, source_region, reference_block) = match previous {
        Some(prior) => (
            prior.plan.session.r + 1,
            prior.plan.window.h_max + 10,
            BoundaryCause::Handoff,
            Some(prior.plan.region),
            prior.anchor.block.clone(),
        ),
        None => (
            0,
            config.genesis.block.height + 10,
            BoundaryCause::Genesis,
            None,
            config.genesis.block.clone(),
        ),
    };
    let region = RegionId::new((r % 3) as u8).unwrap();
    let registry: serde_json::Value = serde_json::from_str(include_str!(
        "../../../verification/mock-plan-execution/implementation-v1/public-registry.json"
    ))
    .unwrap();
    let committee: Vec<_> = registry["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| {
            let pid = Pid::new(u8::try_from(entry["pid"].as_u64().unwrap()).unwrap()).unwrap();
            (pid.region() == region).then(|| CommitteeEntry {
                pid,
                identity_key: component_fixture::unhex(entry["identity_key"].as_str().unwrap())
                    .try_into()
                    .unwrap(),
                vote_key: component_fixture::unhex(entry["vote_key"].as_str().unwrap())
                    .try_into()
                    .unwrap(),
                weight: 1,
            })
        })
        .collect();
    let config_ref = config.reference().unwrap();
    let mut plan = PlanRecord {
        session: SessionId { beta: 0, r },
        cause,
        source_region,
        region,
        total_weight: 5,
        fault_bound: 1,
        eligible_proposers: committee.iter().map(|entry| entry.pid).collect(),
        committee,
        leader_rule: RuleSpec::static_schedule(),
        window: Window {
            h_min: end - 9,
            h_max: end,
        },
        reference_block,
        config_ref,
        authorization_ref: hash(0),
    };
    plan.authorization_ref =
        authorization_ref(config_ref, config.registry_hash, &plan.input()).unwrap();
    let program = ProgramConfig::from_plan(config, &plan).unwrap();
    let cid = regional_cid(&plan.session, anchor.block.block_id, &program).unwrap();
    Descriptor { plan, anchor, cid }
}

fn espresso(q: u64, effects: Vec<ReplayEffect>, bodies: Vec<LifecycleBody>) -> ReplayRecord {
    let position = ReplayPosition {
        key_us: q,
        kind: ReplayKind::Espresso,
        index: ReplayIndex::Espresso(q),
    };
    replay(position, InputRef::Espresso(q), effects, bodies)
}

fn replay(
    position: ReplayPosition,
    input_ref: InputRef,
    effects: Vec<ReplayEffect>,
    bodies: Vec<LifecycleBody>,
) -> ReplayRecord {
    let events = bodies
        .into_iter()
        .map(|body| LifecycleEvent {
            position: position.clone(),
            body,
        })
        .collect();
    ReplayRecord {
        position,
        input_ref,
        effects,
        events,
    }
}

fn closing(descriptor: &Descriptor) -> LifecycleBody {
    LifecycleBody::SessionClosed {
        session: descriptor.plan.session.clone(),
        reason: CloseReason::Complete,
    }
}

fn session_end(previous: &Descriptor, assigned: &Descriptor) -> ReplayRecord {
    let session = previous.plan.session.clone();
    replay(
        ReplayPosition {
            key_us: 24 * SEC,
            kind: ReplayKind::SessionEnd,
            index: ReplayIndex::Session(session.clone()),
        },
        InputRef::SessionEnd(session),
        vec![],
        vec![closing(previous), assigning(assigned)],
    )
}

fn assigning(descriptor: &Descriptor) -> LifecycleBody {
    LifecycleBody::SessionAssigned {
        descriptor: descriptor.clone(),
    }
}

fn append(descriptor: &Descriptor, first: u64, last: u64) -> ReplayEffect {
    ReplayEffect::Append {
        first,
        last,
        source: GlobalSource::Regional(ResultKey {
            cid: descriptor.cid,
            block_hash: hash(last as u8),
        }),
    }
}

fn plan(manager: &Manager, input: ManagerInput, now: u64) -> ManagerDelta {
    match manager.plan(input, now).unwrap() {
        PlanOutcome::Planned(delta) => delta,
        other => panic!("expected delta, got {other:?}"),
    }
}

fn commit(manager: &mut Manager, input: ManagerInput, now: u64) -> Vec<ActionIntent> {
    let delta = plan(manager, input, now);
    let intents = delta.intents().to_vec();
    let attestation = CommitRef::for_delta(&delta, hash(99));
    manager.accept_commit(delta, attestation).unwrap();
    intents
}

fn receipt(id: ActionId, outcome: ReceiptOutcome) -> ManagerInput {
    ManagerInput::Receipt(ActionReceipt {
        id,
        receipt_ref: hash(100),
        outcome,
    })
}

fn prepare(manager: &mut Manager, cid: Cid, instance_ref: Hash32, now: u64) -> PreparedRef {
    let create = manager.view().program(cid).unwrap().create_action.unwrap();
    let prepare = commit(
        manager,
        receipt(create, ReceiptOutcome::Created { instance_ref }),
        now,
    );
    let ActionKind::Prepare {
        anchor,
        provenance_ref,
        validation_version,
        ..
    } = &prepare[0].kind
    else {
        panic!("expected Prepare");
    };
    PreparedRef {
        prepare_action: prepare[0].id,
        worker: manager.view().binding().worker,
        cid,
        instance_ref,
        anchor: anchor.clone(),
        config_ref: manager.view().binding().public_config,
        validation_version: *validation_version,
        execution_ref: hash(73),
        provenance_ref: *provenance_ref,
    }
}

fn ready(manager: &mut Manager, cid: Cid, instance_ref: Hash32, now: u64) {
    let prepared = prepare(manager, cid, instance_ref, now);
    commit(manager, ManagerInput::PreparationCompleted(prepared), now);
}

fn enable(manager: &mut Manager, now: u64) {
    commit(
        manager,
        ManagerInput::ClockReady {
            generation: 1,
            basis_ref: hash(32),
        },
        now,
    );
}

fn error(manager: &Manager, input: ManagerInput, now: u64, expected: ManagerError) {
    assert_eq!(manager.plan(input, now).unwrap_err(), expected);
}

fn is_dispatchable(manager: &Manager, id: ActionId, now: u64) -> bool {
    manager
        .dispatchable(now)
        .iter()
        .any(|intent| intent.id == id)
}

#[test]
fn switches_entered_instances_and_selects_back_original() {
    let fixture = Fixture::new(5);
    let mut manager = fixture.manager();
    assert!(manager.view().program(fixture.genesis.cid).is_none());
    assert_eq!(
        manager.view().selected(&fixture.genesis.plan.session),
        Some(&fixture.genesis)
    );

    let a = fixture.successor(&fixture.genesis, 9);
    let b = fixture.successor(&fixture.genesis, 10);
    let session = &a.plan.session;
    commit(&mut manager, fixture.candidate(&a), 1);
    commit(&mut manager, fixture.candidate(&b), 1);
    let instance_a = hash(40);
    let instance_b = hash(41);
    ready(&mut manager, a.cid, instance_a, 1);
    enable(&mut manager, 1);
    let enter_a = commit(&mut manager, ManagerInput::Schedule(session.clone()), 1)[0].id;
    let program = manager.view().program(a.cid).unwrap();
    assert_eq!(program.phase, Phase::Entering);
    assert_eq!(program.entry_time_us, None);
    assert!(!program.entered);
    assert!(is_dispatchable(&manager, enter_a, 1));

    commit(
        &mut manager,
        receipt(
            enter_a,
            ReceiptOutcome::Entered {
                instance_ref: instance_a,
                entry_time_us: 1,
            },
        ),
        2,
    );
    assert_eq!(manager.view().scheduled(session), Some(a.cid));
    assert!(matches!(
        manager
            .plan(ManagerInput::Schedule(session.clone()), 2)
            .unwrap(),
        PlanOutcome::Unchanged(_)
    ));
    ready(&mut manager, b.cid, instance_b, 2);
    let switch = commit(&mut manager, ManagerInput::Schedule(session.clone()), 2);
    let (pause_a, enter_b) = (switch[0].id, switch[1].id);
    assert!(
        matches!(switch[0].kind, ActionKind::Pause { cid, instance_ref } if cid == a.cid && instance_ref == instance_a)
    );
    assert!(switch[1].dependencies.contains(&pause_a));
    assert!(!is_dispatchable(&manager, enter_b, 2));
    commit(
        &mut manager,
        receipt(
            pause_a,
            ReceiptOutcome::Paused {
                instance_ref: instance_a,
            },
        ),
        2,
    );
    assert!(is_dispatchable(&manager, enter_b, 2));
    commit(
        &mut manager,
        receipt(
            enter_b,
            ReceiptOutcome::Entered {
                instance_ref: instance_b,
                entry_time_us: 2,
            },
        ),
        2,
    );

    let prefix = espresso(1, vec![append(&fixture.genesis, 1, 9)], vec![]);
    commit(&mut manager, fixture.record(0, prefix), 3);
    let record = session_end(&fixture.genesis, &a);
    let close_b = commit(&mut manager, fixture.record(1, record), 3)[0].id;
    assert_eq!(manager.view().selected(session), Some(&a));
    assert_eq!(
        manager.view().program(b.cid).unwrap().selection,
        Selection::Excluded
    );
    let select = commit(&mut manager, ManagerInput::Schedule(session.clone()), 3);
    let [select] = select.as_slice() else {
        panic!("expected only a SelectBack");
    };
    assert_eq!(
        select.kind,
        ActionKind::SelectBack {
            cid: a.cid,
            instance_ref: instance_a
        }
    );
    assert!(!is_dispatchable(&manager, select.id, 3));
    commit(
        &mut manager,
        receipt(
            close_b,
            ReceiptOutcome::Closed {
                instance_ref: Some(instance_b),
            },
        ),
        3,
    );
    assert!(is_dispatchable(&manager, select.id, 3));
    commit(
        &mut manager,
        receipt(
            select.id,
            ReceiptOutcome::SelectedBack {
                instance_ref: instance_a,
            },
        ),
        3,
    );
    let program = manager.view().program(a.cid).unwrap();
    assert_eq!(program.phase, Phase::Running);
    assert_eq!(program.enter_action, Some(enter_a));
    assert_eq!(program.entry_time_us, Some(1));
}

#[test]
fn unconfirmed_enter_is_parked_and_stale_receipts_preserve_guards() {
    let fixture = Fixture::new(5);
    let mut manager = fixture.manager();
    let a = fixture.successor(&fixture.genesis, 8);
    let b = fixture.successor(&fixture.genesis, 9);
    let c = fixture.successor(&fixture.genesis, 10);
    let session = a.plan.session.clone();
    let instance_a = hash(40);
    let instance_b = hash(41);
    enable(&mut manager, 1);
    commit(&mut manager, fixture.candidate(&a), 1);
    ready(&mut manager, a.cid, instance_a, 1);
    let enter_a = commit(&mut manager, ManagerInput::Schedule(session.clone()), 1)[0].id;
    let first_failure = receipt(
        enter_a,
        ReceiptOutcome::Retryable {
            failure_ref: hash(101),
        },
    );
    commit(&mut manager, first_failure.clone(), 1);
    for (descriptor, instance_ref) in [(&b, instance_b), (&c, hash(42))] {
        commit(&mut manager, fixture.candidate(descriptor), 2);
        ready(&mut manager, descriptor.cid, instance_ref, 2);
        commit(&mut manager, ManagerInput::Schedule(session.clone()), 2);
    }
    let pause_a = manager.view().program(a.cid).unwrap().pause_action.unwrap();
    let pause_b = manager.view().program(b.cid).unwrap().pause_action.unwrap();
    let enter_c = manager.view().program(c.cid).unwrap().enter_action.unwrap();
    assert!(!is_dispatchable(&manager, enter_a, 2));
    assert!(is_dispatchable(&manager, pause_a, 2));
    commit(
        &mut manager,
        receipt(
            pause_b,
            ReceiptOutcome::Paused {
                instance_ref: instance_b,
            },
        ),
        2,
    );
    assert!(!is_dispatchable(&manager, enter_c, 2));
    commit(
        &mut manager,
        receipt(
            pause_a,
            ReceiptOutcome::Paused {
                instance_ref: instance_a,
            },
        ),
        3,
    );
    assert!(is_dispatchable(&manager, enter_c, 3));
    assert!(!manager.view().program(a.cid).unwrap().entered);
    assert_eq!(manager.view().program(a.cid).unwrap().phase, Phase::Ready);
    let entered_a = receipt(
        enter_a,
        ReceiptOutcome::Entered {
            instance_ref: instance_a,
            entry_time_us: 1,
        },
    );
    commit(&mut manager, entered_a.clone(), 3);
    assert_eq!(manager.view().program(a.cid).unwrap().phase, Phase::Paused);
    assert!(matches!(
        manager.plan(first_failure, 3).unwrap(),
        PlanOutcome::Unchanged(_)
    ));
    assert!(matches!(
        manager
            .plan(
                receipt(
                    enter_a,
                    ReceiptOutcome::Retryable {
                        failure_ref: hash(11)
                    }
                ),
                3
            )
            .unwrap(),
        PlanOutcome::Unchanged(_)
    ));
    assert_eq!(
        manager.view().action(enter_a).unwrap().status,
        ActionStatus::Applied
    );
    assert!(matches!(
        manager.plan(entered_a, 3).unwrap(),
        PlanOutcome::Unchanged(_)
    ));
    error(
        &manager,
        receipt(
            enter_a,
            ReceiptOutcome::Entered {
                instance_ref: instance_a,
                entry_time_us: 2,
            },
        ),
        3,
        ManagerError::ReceiptConflict,
    );

    let stop = manager.view().program(c.cid).unwrap().stop_us;
    assert!(is_dispatchable(&manager, enter_c, stop - 1));
    assert!(!is_dispatchable(&manager, enter_c, stop));
    commit(&mut manager, ManagerInput::ObserveTime, stop);
    assert_eq!(
        manager.view().program(a.cid).unwrap().phase,
        Phase::Terminated
    );
    assert_eq!(
        manager.view().action(enter_c).unwrap().status,
        ActionStatus::Cancelled
    );
    let stale = receipt(
        enter_c,
        ReceiptOutcome::Retryable {
            failure_ref: hash(11),
        },
    );
    assert!(matches!(
        manager.plan(stale, stop).unwrap(),
        PlanOutcome::Unchanged(_)
    ));
    let late = receipt(
        enter_c,
        ReceiptOutcome::Entered {
            instance_ref: hash(42),
            entry_time_us: stop - 1,
        },
    );
    commit(&mut manager, late, stop);
    assert_eq!(
        manager.view().program(c.cid).unwrap().entry_time_us,
        Some(stop - 1)
    );
    assert_eq!(
        manager.view().program(c.cid).unwrap().phase,
        Phase::Terminated
    );
    assert_eq!(
        manager.view().action(enter_c).unwrap().status,
        ActionStatus::Cancelled
    );
}

#[test]
fn replay_tip_and_scope_block_late_candidates_and_bind_entire_record() {
    let fixture = Fixture::new(5);
    let mut manager = fixture.manager();
    let low = fixture.successor(&fixture.genesis, 7);
    let lower = fixture.successor(&fixture.genesis, 6);
    let high = fixture.successor(&fixture.genesis, 10);
    commit(&mut manager, fixture.candidate(&low), 1);
    let prepared = prepare(&mut manager, low.cid, hash(42), 1);
    let mut wrong = prepared.clone();
    wrong.validation_version += 1;
    error(
        &manager,
        ManagerInput::PreparationCompleted(wrong),
        1,
        ManagerError::PreparationBinding,
    );
    let first = espresso(1, vec![append(&fixture.genesis, 1, 8)], vec![]);
    assert!(matches!(
        manager.plan(fixture.record(1, first.clone()), 1).unwrap(),
        PlanOutcome::Wait(WaitReason::SourceGap {
            expected: 0,
            received: 1
        })
    ));
    commit(&mut manager, fixture.record(0, first.clone()), 1);
    assert_eq!(manager.view().record(0).unwrap().record, first);
    assert_eq!(
        manager.view().guards().physical_tip(fixture.genesis.cid),
        Some(8)
    );
    assert_eq!(
        manager.view().program(low.cid).unwrap().phase,
        Phase::Terminated
    );
    assert_eq!(
        manager.view().program(low.cid).unwrap().selection,
        Selection::Excluded
    );
    error(
        &manager,
        fixture.candidate(&lower),
        1,
        ManagerError::CandidateConflict,
    );
    commit(
        &mut manager,
        ManagerInput::PreparationCompleted(prepared),
        1,
    );
    assert_eq!(
        manager.view().program(low.cid).unwrap().phase,
        Phase::Terminated
    );
    assert!(matches!(
        manager.plan(fixture.record(0, first.clone()), 1).unwrap(),
        PlanOutcome::Unchanged(_)
    ));
    let mut conflict = first;
    conflict.effects.clear();
    error(
        &manager,
        fixture.record(0, conflict),
        1,
        ManagerError::SourceConflict,
    );

    let mut malformed = espresso(2, vec![], vec![closing(&fixture.genesis), assigning(&high)]);
    if let LifecycleBody::SessionAssigned { descriptor } = &mut malformed.events[1].body {
        descriptor.cid = Cid::new(hash(0));
    }
    let revision = manager.view().revision();
    error(
        &manager,
        fixture.record(1, malformed),
        1,
        ManagerError::CidMismatch,
    );
    assert_eq!(manager.view().revision(), revision);
    assert_eq!(
        manager
            .view()
            .guards()
            .closed_session(&fixture.genesis.plan.session),
        None
    );

    let r2 = fixture.successor(&high, 20);
    let multiple = espresso(
        2,
        vec![append(&fixture.genesis, 9, 10), append(&high, 11, 20)],
        vec![
            closing(&fixture.genesis),
            assigning(&high),
            closing(&high),
            assigning(&r2),
        ],
    );
    let ids: Vec<_> = multiple
        .events
        .iter()
        .map(|event| poc_viewer_gfr::lifecycle_id(event).unwrap())
        .collect();
    let intents = commit(&mut manager, fixture.record(1, multiple.clone()), 1);
    assert!(
        intents.is_empty(),
        "intermediate/foreign assignment has no local instance"
    );
    assert_eq!(manager.view().record(1).unwrap().record, multiple);
    assert_eq!(manager.view().record(1).unwrap().lifecycle_ids, ids);
    assert_eq!(manager.view().selected(&r2.plan.session), Some(&r2));
    assert_eq!(
        manager.view().guards().closed_session(&high.plan.session),
        Some(CloseReason::Complete)
    );
    error(
        &manager,
        fixture.candidate(&high),
        1,
        ManagerError::CandidateConflict,
    );
}

#[test]
fn delta_cause_binds_input_time_and_worker_and_partial_failure_latches() {
    let fixture = Fixture::new(0);
    let mut manager = fixture.manager();
    let remote = fixture.successor(&fixture.genesis, 9);
    let record = espresso(
        1,
        vec![],
        vec![closing(&fixture.genesis), assigning(&remote)],
    );
    let input = fixture.record(0, record);
    let pending = plan(&manager, input.clone(), 10);
    let replan = plan(&manager, input.clone(), 10);
    let different_time = plan(&manager, input.clone(), 11);
    let different_input = plan(
        &manager,
        ManagerInput::ClockUnsafe {
            basis_ref: hash(99),
        },
        10,
    );
    assert_eq!(replan.intents(), pending.intents());
    assert_eq!(manager.view().revision(), 0);
    assert_eq!(manager.view().cursor(), None);
    assert_eq!(
        manager.view().program(fixture.genesis.cid).unwrap().phase,
        Phase::Assigned
    );
    for delta in [&different_time, &different_input] {
        assert_eq!(
            manager.accept_commit(pending.clone(), CommitRef::for_delta(delta, hash(1))),
            Err(ManagerError::CommitBinding)
        );
    }
    let durable = CommitRef::for_delta(&pending, hash(1));
    let mut wrong = durable.clone();
    wrong.binding.worker = Pid::new(1).unwrap();
    assert_eq!(
        manager.accept_commit(pending.clone(), wrong),
        Err(ManagerError::CommitBinding)
    );

    let foreign_fixture = Fixture::new(1);
    let foreign = foreign_fixture.manager();
    let foreign_delta = plan(&foreign, input.clone(), 10);
    assert_eq!(
        manager.latch_commit_failure(&foreign_delta, hash(2)),
        Err(ManagerError::CommitBinding)
    );
    assert!(manager.view().faulted().is_none());

    drop(replan);
    assert_eq!(
        manager
            .view()
            .guards()
            .closed_session(&fixture.genesis.plan.session),
        None
    );
    let key = ResultKey {
        cid: fixture.genesis.cid,
        block_hash: hash(130),
    };
    commit(
        &mut manager,
        ManagerInput::RetainResult {
            key,
            verified_result_ref: hash(131),
        },
        10,
    );
    assert_eq!(
        manager.latch_commit_failure(&pending, hash(2)),
        Err(ManagerError::StaleDelta)
    );
    let pending = plan(&manager, input, 10);
    manager.latch_commit_failure(&pending, hash(2)).unwrap();
    manager.latch_commit_failure(&pending, hash(2)).unwrap();
    assert_eq!(manager.view().faulted().unwrap().guard_ref, hash(2));
    assert_eq!(
        manager.view().program(fixture.genesis.cid).unwrap().phase,
        Phase::Terminated
    );
    assert_eq!(
        manager
            .view()
            .guards()
            .closed_session(&fixture.genesis.plan.session),
        Some(CloseReason::Complete)
    );
    assert_eq!(manager.view().next_source_seq(), 0);
    assert_eq!(manager.view().cursor(), None);
    assert!(manager.view().selected(&remote.plan.session).is_none());
    assert!(manager.view().results().contains_key(&key));
    assert_eq!(manager.view().status(), CommitStatus::Faulted);
    assert!(manager.dispatchable(10).is_empty());
    error(
        &manager,
        ManagerInput::ClockReady {
            generation: 9,
            basis_ref: hash(7),
        },
        11,
        ManagerError::RuntimeFaulted,
    );
    assert_eq!(
        manager.accept_commit(pending.clone(), CommitRef::for_delta(&pending, hash(3))),
        Err(ManagerError::RuntimeFaulted)
    );
}

#[test]
fn unsafe_closes_preparing_and_running_without_dropping_result() {
    let fixture = Fixture::new(5);
    let mut manager = fixture.manager();
    let a = fixture.successor(&fixture.genesis, 8);
    let b = fixture.successor(&fixture.genesis, 9);
    let other = fixture.successor(&fixture.genesis, 10);
    enable(&mut manager, 1);
    commit(&mut manager, fixture.candidate(&a), 1);
    commit(&mut manager, fixture.candidate(&b), 1);
    let instance_a = hash(41);
    ready(&mut manager, a.cid, instance_a, 2);
    let prepare_b = prepare(&mut manager, b.cid, hash(42), 2);
    let enter_a = commit(
        &mut manager,
        ManagerInput::Schedule(a.plan.session.clone()),
        2,
    )[0]
    .id;
    commit(
        &mut manager,
        receipt(
            enter_a,
            ReceiptOutcome::Entered {
                instance_ref: instance_a,
                entry_time_us: 2,
            },
        ),
        2,
    );
    let key = ResultKey {
        cid: a.cid,
        block_hash: hash(50),
    };
    let publish = commit(
        &mut manager,
        ManagerInput::RetainResult {
            key,
            verified_result_ref: hash(51),
        },
        2,
    )[0]
    .id;

    let unsafe_clock = ManagerInput::ClockUnsafe {
        basis_ref: hash(52),
    };
    let closes = commit(&mut manager, unsafe_clock, 2);
    assert_eq!(closes.len(), 2);
    assert_eq!(manager.view().guards().unsafe_session_fence(), Some((0, 1)));
    for cid in [a.cid, b.cid] {
        let program = manager.view().program(cid).unwrap();
        assert_eq!(program.phase, Phase::Terminated);
        assert_eq!(program.selection, Selection::Candidate);
        assert_eq!(
            program.terminal_cause,
            Some(TerminalCause::LocalClockUnsafe {
                generation: Some(1),
                basis_ref: hash(52)
            })
        );
    }
    assert!(!manager.view().guards().clock_enabled());
    assert!(is_dispatchable(&manager, publish, 2));
    commit(
        &mut manager,
        ManagerInput::PreparationCompleted(prepare_b),
        2,
    );
    assert_eq!(
        manager.view().program(b.cid).unwrap().phase,
        Phase::Terminated
    );
    let old_clock = ManagerInput::ClockReady {
        generation: 1,
        basis_ref: hash(32),
    };
    assert!(matches!(
        manager.plan(old_clock, 2).unwrap(),
        PlanOutcome::Unchanged(_)
    ));
    assert!(!manager.view().guards().clock_enabled());
    error(
        &manager,
        ManagerInput::ClockReady {
            generation: 1,
            basis_ref: hash(53),
        },
        2,
        ManagerError::ClockGeneration,
    );
    commit(
        &mut manager,
        ManagerInput::ClockReady {
            generation: 2,
            basis_ref: hash(54),
        },
        3,
    );
    error(
        &manager,
        fixture.candidate(&other),
        3,
        ManagerError::CandidateConflict,
    );
    assert!(matches!(
        manager
            .plan(ManagerInput::Schedule(a.plan.session.clone()), 3)
            .unwrap(),
        PlanOutcome::Wait(WaitReason::ClockDisabled)
    ));
    error(
        &manager,
        ManagerInput::ObserveTime,
        2,
        ManagerError::ClockRegression,
    );
    assert!(matches!(
        manager
            .plan(
                ManagerInput::RetainResult {
                    key,
                    verified_result_ref: hash(55)
                },
                3
            )
            .unwrap(),
        PlanOutcome::Unchanged(_)
    ));
    assert_eq!(manager.view().results()[&key].publish_action, publish);
}

#[test]
fn complete_actions_progress_commit_status_and_keep_original_receipts() {
    let fixture = Fixture::new(5);
    let mut manager = fixture.manager();
    let a = fixture.successor(&fixture.genesis, 9);
    let delta = plan(&manager, fixture.candidate(&a), 1);
    let attestation = CommitRef::for_delta(&delta, hash(66));
    assert_eq!(
        manager.accept_commit(delta.clone(), attestation.clone()),
        Ok(CommitStatus::Committed { actions_pending: 1 })
    );
    assert_eq!(
        manager.accept_commit(delta.clone(), attestation.clone()),
        Ok(CommitStatus::Committed { actions_pending: 1 })
    );
    let prepared = prepare(&mut manager, a.cid, hash(41), 1);
    assert_eq!(
        manager.accept_commit(delta.clone(), attestation.clone()),
        Ok(CommitStatus::Committed { actions_pending: 1 })
    );
    commit(
        &mut manager,
        ManagerInput::PreparationCompleted(prepared),
        1,
    );
    assert_eq!(
        manager.accept_commit(delta, attestation),
        Ok(CommitStatus::ActionsApplied)
    );
}

#[test]
fn bounds_reject_full_record_atomically() {
    let fixture = Fixture::new(5);
    let mut manager = fixture.manager();
    let event = closing(&fixture.genesis);
    let max = espresso(1, vec![], vec![event.clone(); 64]);
    let over = espresso(1, vec![], vec![event; 65]);
    error(
        &manager,
        fixture.record(0, over),
        1,
        ManagerError::Capacity {
            kind: CapacityKind::Events,
            limit: 64,
        },
    );
    assert_eq!(manager.view().next_source_seq(), 0);
    assert_eq!(
        manager
            .view()
            .guards()
            .closed_session(&fixture.genesis.plan.session),
        None
    );
    commit(&mut manager, fixture.record(0, max), 1);
    assert_eq!(manager.view().record(0).unwrap().lifecycle_ids.len(), 64);
}

#[test]
fn session_end_without_early_result_preserves_window_and_waits_for_preparation() {
    let fixture = Fixture::new(5);
    let mut manager = fixture.manager();
    let successor = fixture.successor(&fixture.genesis, 9);
    let session = successor.plan.session.clone();
    let now = 24 * SEC;
    enable(&mut manager, 1);
    let prefix = espresso(1, vec![append(&fixture.genesis, 1, 9)], vec![]);
    commit(&mut manager, fixture.record(0, prefix), now);
    let record = session_end(&fixture.genesis, &successor);
    let intents = commit(&mut manager, fixture.record(1, record), now);
    assert!(
        matches!(&intents[0].kind, ActionKind::Create { descriptor, .. } if descriptor == &successor)
    );
    assert!(
        matches!(manager.plan(ManagerInput::Schedule(session.clone()), now).unwrap(), PlanOutcome::Wait(WaitReason::Preparation { cid }) if cid == successor.cid)
    );
    let program = manager.view().program(successor.cid).unwrap();
    assert_eq!(
        program.descriptor.plan.window,
        Window {
            h_min: 11,
            h_max: 20
        }
    );
    assert!(manager.view().results().is_empty());
    ready(&mut manager, successor.cid, hash(81), now);
    let enter = commit(&mut manager, ManagerInput::Schedule(session), now)[0].id;
    commit(
        &mut manager,
        receipt(
            enter,
            ReceiptOutcome::Entered {
                instance_ref: hash(81),
                entry_time_us: now,
            },
        ),
        now,
    );
    let result = ResultKey {
        cid: successor.cid,
        block_hash: hash(14),
    };
    commit(
        &mut manager,
        ManagerInput::RetainResult {
            key: result,
            verified_result_ref: hash(82),
        },
        now,
    );
    assert_eq!(
        manager.view().results().keys().collect::<Vec<_>>(),
        vec![&result]
    );
}

#[test]
fn force_closes_running_preparing_and_paused_cids_before_releasing_close() {
    let fixture = Fixture::new(5);
    let mut manager = fixture.manager();
    let paused = fixture.successor(&fixture.genesis, 8);
    let running = fixture.successor(&fixture.genesis, 9);
    let preparing = fixture.successor(&fixture.genesis, 10);
    let session = paused.plan.session.clone();
    enable(&mut manager, 1);

    commit(&mut manager, fixture.candidate(&paused), 1);
    ready(&mut manager, paused.cid, hash(40), 1);
    let enter = commit(&mut manager, ManagerInput::Schedule(session.clone()), 1)[0].id;
    commit(
        &mut manager,
        receipt(enter, ReceiptOutcome::Entered { instance_ref: hash(40), entry_time_us: 1 }),
        1,
    );
    commit(&mut manager, fixture.candidate(&running), 2);
    ready(&mut manager, running.cid, hash(41), 2);
    let switch = commit(&mut manager, ManagerInput::Schedule(session.clone()), 2);
    commit(
        &mut manager,
        receipt(switch[0].id, ReceiptOutcome::Paused { instance_ref: hash(40) }),
        2,
    );
    let entered_running = receipt(
        switch[1].id,
        ReceiptOutcome::Entered { instance_ref: hash(41), entry_time_us: 2 },
    );
    commit(&mut manager, entered_running.clone(), 2);
    commit(&mut manager, fixture.candidate(&preparing), 2);
    let pending = prepare(&mut manager, preparing.cid, hash(42), 2);
    let old = [&paused, &running, &preparing];
    for (descriptor, phase) in old.into_iter().zip([Phase::Paused, Phase::Running, Phase::Preparing]) {
        assert_eq!(manager.view().program(descriptor.cid).unwrap().phase, phase);
    }

    let result = ResultKey { cid: running.cid, block_hash: hash(80) };
    commit(
        &mut manager,
        ManagerInput::RetainResult { key: result, verified_result_ref: hash(81) },
        2,
    );

    let mut successor = descriptor(
        &fixture.config,
        Some(&fixture.genesis),
        fixture.genesis.anchor.clone(),
    );
    successor.plan.session = SessionId { beta: 1, r: 0 };
    successor.plan.cause = BoundaryCause::Force;
    successor.plan.reference_block = successor.anchor.block.clone();
    successor.plan.window = Window { h_min: 16, h_max: 25 };
    successor.plan.authorization_ref = authorization_ref(
        successor.plan.config_ref, fixture.config.registry_hash, &successor.plan.input(),
    ).unwrap();
    successor.cid = regional_cid(
        &successor.plan.session, successor.anchor.block.block_id,
        &ProgramConfig::from_plan(&fixture.config, &successor.plan).unwrap(),
    ).unwrap();
    let position = L1Position { block: 6, tx: 0, log: 0 };
    let record = replay(
        ReplayPosition { key_us: 22 * SEC, kind: ReplayKind::Force, index: ReplayIndex::L1(position) },
        InputRef::L1(poc_viewer_gfr::ControlRef::static_force(position, 1, 0)),
        vec![ReplayEffect::Duty {
            before: Tail::Next(fixture.genesis.clone()),
            after: Tail::Next(successor.clone()),
        }],
        vec![
            LifecycleBody::BranchClosed { branch: 0, reason: CloseReason::Force },
            assigning(&successor),
        ],
    );
    let input = fixture.record(0, record);
    let delta = plan(&manager, input.clone(), 3);
    let close = TerminalCause::Public(CloseReason::Force);
    for descriptor in old {
        assert_eq!(delta.guards().terminal(descriptor.cid), Some(close));
        assert!(manager.view().program(descriptor.cid).unwrap().terminal_cause.is_none());
    }
    let closes: Vec<_> = delta.intents().iter().filter_map(|intent| match intent.kind {
        ActionKind::Close { cid, cause, .. } if cause == close => Some((cid, intent.id)),
        _ => None,
    }).collect();
    assert_eq!(closes.len(), 3);
    for (_, action) in &closes {
        assert!(!is_dispatchable(&manager, *action, 3));
    }
    let commit_ref = CommitRef::for_delta(&delta, hash(99));
    manager.accept_commit(delta, commit_ref).unwrap();
    for (cid, action) in closes {
        assert!(is_dispatchable(&manager, action, 3));
        let program = manager.view().program(cid).unwrap();
        assert_eq!(program.phase, Phase::Terminated);
        assert_eq!(program.terminal_cause, Some(close));
    }
    assert_eq!(manager.view().selected(&successor.plan.session), Some(&successor));
    assert!(matches!(manager.plan(input, 3).unwrap(), PlanOutcome::Unchanged(_)));
    assert!(matches!(manager.plan(entered_running, 3).unwrap(), PlanOutcome::Unchanged(_)));
    let late = commit(&mut manager, ManagerInput::PreparationCompleted(pending), 3);
    assert!(late.iter().all(|intent| !matches!(intent.kind,
        ActionKind::Enter { .. } | ActionKind::SelectBack { .. }
    )));
    assert!(manager.dispatchable(3).iter().any(|intent| matches!(
        intent.kind, ActionKind::Publish { key, .. } if key == result
    )));
    assert_eq!(manager.view().program(preparing.cid).unwrap().phase, Phase::Terminated);
}
