//! Independent source/contract-derived checks for f9364a2.
//! All keys, proofs, receipts and execution references below are synthetic SHAPE
//! fixtures. No test claims signature validity, issuer authority, fsync or engine execution.
use poc_healthy_session_api::*;
use poc_types::domain::{
    Account, AnchorEvidenceSource, BlockRef, Boundary, BoundaryCause, CommitteeEntry,
    ExecutionState, LifecycleBody, LifecycleEvent, Metadata, PlanInput, PlanRecord, ReplayIndex,
    ReplayKind, RuleSpec, Window,
};
use poc_viewer_gfr::{ControlRef, InputRef, ReplayEffect, Tail};
use std::{cell::Cell, collections::BTreeMap};

#[allow(dead_code)]
#[path = "../../../../../crates/healthy-session-api/examples/h2_consumer.rs"]
mod h2;
#[allow(dead_code)]
#[path = "../../../../../crates/healthy-session-api/examples/h3_consumer.rs"]
mod h3;

fn hash(byte: u8) -> Hash32 {
    Hash32::new([byte; 32])
}
fn id(byte: u8) -> LocalId {
    LocalId::from_hash(hash(byte)).unwrap()
}
fn scope() -> WorkerScope {
    WorkerScope {
        run: hash(0x11),
        config: config().reference().unwrap(),
        worker: Pid::new(7).unwrap(),
        incarnation: 1,
    }
}
fn evidence() -> EvidenceRef {
    EvidenceRef {
        source: hash(0x44),
        object: hash(0x55),
    }
}
fn root() -> RootRef {
    RootRef {
        sequence: 1,
        hash: hash(0x66),
    }
}
fn origin() -> ControlledOrigin {
    ControlledOrigin {
        grant: GrantRef {
            scope: scope(),
            sequence: 1,
            hash: hash(0x77),
            tick_us: 10_000_000,
        },
        root: root(),
        effect_index: 0,
    }
}
fn anchor() -> Anchor {
    Anchor {
        block: BlockRef {
            height: 0,
            origin: 0,
            block_id: hash(0xa0),
        },
        timestamp_us: 10_000_000,
    }
}
fn execution() -> ExecutionAnchor {
    ExecutionAnchor {
        anchor: anchor(),
        state: ExecutionState {
            metadata: anchor().metadata(),
            accounts: BTreeMap::from([(
                1,
                Account {
                    nonce: 0,
                    balance: 10,
                },
            )]),
        },
        native_hash: hash(0xa1),
    }
}
fn config() -> PublicConfig {
    PublicConfig {
        chain_id: 1,
        rule_version: 1,
        verifier_version: 1,
        registry_hash: hash(1),
        dep_fixture_hash: hash(2),
        l1_fixture_hash: hash(3),
        genesis: anchor(),
        genesis_state_root: hash(4),
        genesis_native_hash: hash(0xa1),
        l2_slot_us: 1_000_000,
        l1_genesis_timestamp_us: 0,
        l1_block_us: 2_000_000,
        t_dep_us: 5_000_000,
        t_drift_us: 15_000_000,
        metadata_window_us: 200_000,
        byte_limit: 65_536,
    }
}
fn descriptor() -> Descriptor {
    let committee = (5..10)
        .map(|n| CommitteeEntry {
            pid: Pid::new(n).unwrap(),
            identity_key: [n; 32],
            vote_key: [n; 96],
            weight: 1,
        })
        .collect();
    Descriptor {
        plan: PlanRecord {
            session: SessionId { beta: 0, r: 1 },
            cause: BoundaryCause::Handoff,
            source_region: Some(RegionId::new(0).unwrap()),
            region: RegionId::new(1).unwrap(),
            committee,
            total_weight: 5,
            fault_bound: 1,
            eligible_proposers: (5..10).map(|n| Pid::new(n).unwrap()).collect(),
            leader_rule: RuleSpec::static_schedule(),
            window: Window {
                h_min: 1,
                h_max: 10,
            },
            reference_block: anchor().block,
            config_ref: config().reference().unwrap(),
            authorization_ref: hash(0xa2),
        },
        anchor: anchor(),
        cid: Cid::new(hash(0xa3)),
    }
}
fn program() -> ProgramHandle {
    ProgramHandle {
        scope: scope(),
        cid: descriptor().cid,
        instance_id: id(0xb1),
        generation: 1,
    }
}
fn job() -> PreparationJob {
    PreparationJob {
        program: program(),
        id: id(0xb2),
        generation: 1,
    }
}
fn token() -> ReadyToken {
    ReadyToken {
        id: id(0xb3),
        job: job(),
        anchor: anchor(),
        state_root: execution().state.root().unwrap(),
        native_hash: execution().native_hash,
        validation_profile: hash(0xb4),
        execution_basis: evidence(),
        origin: origin(),
    }
}
fn ready() -> ReadyState {
    ReadyState {
        token: token(),
        execution_anchor: execution(),
        ordered_input_refs: vec![evidence()],
    }
}
fn material() -> StateMaterial {
    StateMaterial {
        checkpoint: None,
        physical_prefix: vec![],
        virtual_prefix: vec![],
        input_refs: vec![],
    }
}
fn request() -> PreparationRequest {
    PreparationRequest {
        generation: 1,
        anchor: anchor(),
        anchor_evidence: AnchorEvidence {
            source: AnchorEvidenceSource::Genesis(scope().config),
            source_objects: vec![],
        },
        material: material(),
        provenance: vec![],
        validation_profile: hash(0xb4),
    }
}
fn profile() -> VerificationProfile {
    VerificationProfile::Native {
        protocol_version: 1,
        verifier_version: 1,
        source_revision: "a".repeat(40),
    }
}
fn chain(n: u64) -> Vec<PhysicalBlock> {
    let mut blocks = Vec::new();
    let mut parent = anchor().block.block_id;
    for height in 1..=n {
        let b = PhysicalBlock {
            parent,
            metadata: Metadata {
                height,
                timestamp_us: 10_000_000 + height * 1_000_000,
                origin: 0,
            },
            pre_state_root: hash(0xc1),
            transactions: vec![],
        };
        parent = b.digest().unwrap();
        blocks.push(b);
    }
    blocks
}
fn delivery() -> RegionalDelivery {
    let blocks = chain(2);
    RegionalDelivery {
        id: id(0xc2),
        result: RegionalResult {
            descriptor: descriptor(),
            block: blocks[1].clone(),
            final_proof: vec![0x01],
            chain: blocks,
            anchor_evidence: request().anchor_evidence,
        },
        validation_basis: vec![evidence()],
        native_profile: profile(),
        origin: origin(),
    }
}
fn position() -> ReplayPosition {
    ReplayPosition {
        key_us: 24_000_000,
        kind: ReplayKind::Espresso,
        index: ReplayIndex::Espresso(9),
    }
}
fn batch() -> ReplayBatch {
    let pos = position();
    let events = vec![
        LifecycleEvent {
            position: pos.clone(),
            body: LifecycleBody::SessionClosed {
                session: SessionId { beta: 0, r: 0 },
                reason: CloseReason::Complete,
            },
        },
        LifecycleEvent {
            position: pos.clone(),
            body: LifecycleBody::SessionAssigned {
                descriptor: descriptor(),
            },
        },
    ];
    ReplayBatch {
        id: derive_local_id(&scope(), IdKind::ReplayBatch, 1, evidence().source).unwrap(),
        source: evidence().source,
        sequence: 1,
        previous: None,
        source_record: evidence(),
        record: ReplayRecord {
            position: pos,
            input_ref: InputRef::Espresso(9),
            effects: vec![],
            events: events.clone(),
        },
        lifecycle_ids: events.iter().map(|e| lifecycle_id(e).unwrap()).collect(),
        origin: origin(),
    }
}
fn action(i: usize, body: ActionBody) -> ActionIntent {
    ActionIntent {
        id: derive_local_id(&scope(), IdKind::Action, i as u64 + 1, *id(0xd0).as_hash()).unwrap(),
        body,
    }
}
fn delta(commands: Vec<ProgramCommand>) -> ManagerDelta {
    ManagerDelta {
        id: id(0xd0),
        scope: scope(),
        base_revision: 0,
        next_revision: 1,
        cause: ManagerCause::Local {
            operation: id(0xd1),
            input: evidence(),
        },
        opaque: OpaqueManagerDelta {
            codec: 1,
            bytes: vec![1],
        },
        guards: vec![],
        actions: commands
            .into_iter()
            .enumerate()
            .map(|(i, c)| action(i, ActionBody::Program(c)))
            .collect(),
    }
}
fn pause() -> ProgramCommand {
    ProgramCommand::Pause {
        program: program(),
        decision: evidence(),
    }
}
fn committed(n: usize) -> RootCommitted {
    RootCommitted {
        scope: scope(),
        delta_id: id(0xd0),
        manager_revision: 1,
        root: root(),
        replay_cursor: None,
        actions_pending: (0..n)
            .map(|i| action(i, ActionBody::Program(pause())).id)
            .collect(),
    }
}
fn receipt(c: &RootCommitted, i: usize) -> AppliedReceipt {
    AppliedReceipt {
        delta_id: c.delta_id,
        action_id: c.actions_pending[i],
        committed_root: c.root,
        origin: origin(),
        reply: ActionReply::Program(ProgramReply::Paused(program())),
    }
}
fn applied(n: usize) -> ActionsApplied {
    let c = committed(n);
    ActionsApplied {
        receipts: (0..n).map(|i| receipt(&c, i)).collect(),
        committed: c,
        completion_origin: origin(),
    }
}
fn fail(disposition: FailureDisposition, commit: Option<RootCommitted>) -> FailClosed {
    FailClosed {
        scope: scope(),
        delta_id: id(0xd0),
        reason: RejectReason::PersistenceFailure,
        disposition,
        actions_pending: commit
            .as_ref()
            .map(|c| c.actions_pending.clone())
            .unwrap_or_default(),
        committed: commit,
        durable_guards: vec![evidence()],
    }
}
fn assert_bad(r: Result<(), ApiError>) {
    assert!(r.is_err(), "shape mutation unexpectedly accepted");
}
fn hex(h: &Hash32) -> String {
    h.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
fn from_hex(s: &str) -> Hash32 {
    Hash32::new(std::array::from_fn(|i| {
        u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap()
    }))
}
fn kind(n: u64) -> IdKind {
    match n {
        1 => IdKind::Operation,
        2 => IdKind::Program,
        3 => IdKind::Preparation,
        4 => IdKind::Ready,
        5 => IdKind::Action,
        6 => IdKind::ManagerDelta,
        7 => IdKind::ReplayBatch,
        8 => IdKind::Delivery,
        _ => panic!("oracle kind"),
    }
}

#[test]
fn independent_literal_ids_and_derived_obligation_ids() {
    let g: serde_json::Value = serde_json::from_str(include_str!("../../goldens.json")).unwrap();
    for c in g["local_id_valid"].as_array().unwrap() {
        let s = WorkerScope {
            run: from_hex(c["run"].as_str().unwrap()),
            config: ConfigRef::new(from_hex(c["config"].as_str().unwrap())),
            worker: Pid::new(c["pid"].as_u64().unwrap() as u8).unwrap(),
            incarnation: c["incarnation"].as_u64().unwrap(),
        };
        let got = derive_local_id(
            &s,
            kind(c["kind"].as_u64().unwrap()),
            c["ordinal"].as_u64().unwrap(),
            from_hex(c["subject"].as_str().unwrap()),
        )
        .unwrap();
        assert_eq!(hex(got.as_hash()), c["expected_hash_hex"], "{}", c["id"]);
        assert_eq!(
            got,
            derive_local_id(
                &s,
                kind(c["kind"].as_u64().unwrap()),
                c["ordinal"].as_u64().unwrap(),
                from_hex(c["subject"].as_str().unwrap())
            )
            .unwrap()
        );
    }
    let s = WorkerScope {
        run: hash(0x11),
        config: ConfigRef::new(hash(0x22)),
        worker: Pid::new(7).unwrap(),
        incarnation: 1,
    };
    for c in g["manager_delta_action_ids"].as_array().unwrap() {
        assert_eq!(
            hex(derive_local_id(
                &s,
                IdKind::Action,
                c["ordinal"].as_u64().unwrap(),
                from_hex(c["subject_delta_id"].as_str().unwrap())
            )
            .unwrap()
            .as_hash()),
            c["expected_hash_hex"]
        );
    }
    for c in g["replay_batch_ids"].as_array().unwrap() {
        assert_eq!(
            hex(derive_local_id(
                &s,
                IdKind::ReplayBatch,
                c["sequence"].as_u64().unwrap(),
                from_hex(c["subject_source"].as_str().unwrap())
            )
            .unwrap()
            .as_hash()),
            c["expected_hash_hex"]
        );
    }
    let original = derive_local_id(&s, IdKind::Action, 9, hash(0x33)).unwrap();
    for m in g["mutations"].as_array().unwrap() {
        assert_ne!(
            hex(original.as_hash()),
            m["expected_hash_hex"],
            "{}",
            m["id"]
        );
    }
}

#[test]
fn public_type_identity_and_no_local_kind_authentication_claim() {
    fn identity(d: poc_types::domain::Descriptor) -> Descriptor {
        d
    }
    fn record(r: poc_viewer_gfr::ReplayRecord) -> ReplayRecord {
        r
    }
    assert_eq!(identity(descriptor()), descriptor());
    assert_eq!(record(batch().record.clone()), batch().record);
    assert!(LocalId::from_hash(hash(1)).is_ok()); // Arbitrary nonzero local ID is not issuer authority.
    let mut s = scope();
    s.config = ConfigRef::new(hash(0));
    assert!(s.validate_shape().is_ok());
}

#[test]
fn basic_bounds_and_error_order_are_explicit() {
    assert_eq!(
        LocalId::from_hash(hash(0)),
        Err(ApiError::InvalidField("local_id"))
    );
    let mut s = scope();
    s.run = hash(0);
    s.incarnation = 2;
    assert_eq!(s.validate_shape(), Err(ApiError::InvalidField("run")));
    for inc in [0, 2, u64::MAX] {
        let mut s = scope();
        s.incarnation = inc;
        assert_eq!(
            s.validate_shape(),
            Err(ApiError::InvalidField("incarnation"))
        );
    }
    assert_eq!(
        derive_local_id(&scope(), IdKind::Action, 0, hash(0)),
        Err(ApiError::InvalidField("ordinal"))
    );
    for pid in [0, 14] {
        let mut s = scope();
        s.worker = Pid::new(pid).unwrap();
        s.validate_shape().unwrap();
    }
    let mut g = origin().grant;
    g.sequence = 0;
    assert_eq!(g.validate_shape(), Err(ApiError::InvalidField("sequence")));
    let mut r = root();
    r.hash = hash(0);
    assert_eq!(r.validate_shape(), Err(ApiError::InvalidField("hash")));
    let mut e = evidence();
    e.source = hash(0);
    assert_eq!(e.validate_shape(), Err(ApiError::InvalidField("source")));
}

#[test]
fn handles_jobs_and_ready_require_exact_binding_but_not_proof_truth() {
    ready().validate_shape().unwrap();
    let mut p = program();
    p.generation = 2;
    assert_bad(p.validate_shape());
    let mut j = job();
    j.generation = 0;
    assert_bad(j.validate_shape());
    let mut r = ready();
    r.token.origin.grant.scope.worker = Pid::new(8).unwrap();
    assert_bad(r.validate_shape());
    let mut r = ready();
    r.token.anchor.block.block_id = hash(0xfe);
    assert_eq!(r.validate_shape(), Err(ApiError::BindingMismatch("anchor")));
    let mut r = ready();
    r.token.anchor.timestamp_us += 1;
    assert_bad(r.validate_shape());
    let mut r = ready();
    r.token.state_root = hash(0xfe);
    assert_eq!(
        r.validate_shape(),
        Err(ApiError::BindingMismatch("state_root"))
    );
    let mut r = ready();
    r.token.native_hash = hash(0xfe);
    assert_eq!(
        r.validate_shape(),
        Err(ApiError::BindingMismatch("native_hash"))
    );
    let mut r = ready();
    r.ordered_input_refs.clear();
    assert_bad(r.validate_shape());
    let mut r = ready();
    r.execution_anchor
        .state
        .accounts
        .get_mut(&1)
        .unwrap()
        .balance += 1;
    assert_bad(r.validate_shape());
    // Restoring self consistency is allowed even though no execution/issuer validation occurred.
    r.token.state_root = r.execution_anchor.state.root().unwrap();
    r.validate_shape().unwrap();
}

#[test]
fn preparation_missing_material_and_bounded_sources() {
    request().validate_shape().unwrap(); // No checkpoint/body/provenance: express WAIT, not Ready.
    let mut q = request();
    q.generation = 0;
    assert_bad(q.validate_shape());
    let mut q = request();
    q.material.physical_prefix = vec![chain(1)[0].clone(); 128];
    q.validate_shape().unwrap();
    q.material.physical_prefix.push(chain(1)[0].clone());
    assert!(matches!(
        q.validate_shape(),
        Err(ApiError::LimitExceeded {
            field: "physical_prefix",
            actual: 129,
            max: 128
        })
    ));
    let mut q = request();
    q.provenance = vec![evidence(); 128];
    q.validate_shape().unwrap();
    q.provenance.push(evidence());
    assert_bad(q.validate_shape());
    let mut q = request();
    q.anchor_evidence.source_objects = vec![vec![0; 65_536]; 128];
    q.validate_shape().unwrap();
    q.anchor_evidence.source_objects[0].push(0);
    assert_bad(q.validate_shape());
    let mut q = request();
    q.material.virtual_prefix.push(VirtualBlock {
        metadata: Metadata {
            height: 1,
            timestamp_us: 11_000_000,
            origin: 0,
        },
        dep: vec![],
        pre_state_root: hash(1),
        transactions: vec![],
    });
    assert_bad(q.validate_shape());
    let mut r = ready();
    r.ordered_input_refs = vec![evidence(); 128];
    r.validate_shape().unwrap();
    r.ordered_input_refs.push(evidence());
    assert_bad(r.validate_shape());
}

#[test]
fn commands_bind_worker_cid_instance_and_ready_job() {
    let commands = vec![
        ProgramCommand::Create {
            descriptor: descriptor(),
            plan_basis: vec![evidence()],
        },
        ProgramCommand::Prepare {
            program: program(),
            request: request(),
        },
        ProgramCommand::Supply {
            job: job(),
            material: material(),
            provenance: vec![],
        },
        ProgramCommand::CancelPrepare {
            job: job(),
            cause: CancelCause::Explicit,
        },
        ProgramCommand::Enter {
            program: program(),
            ready: token(),
        },
        pause(),
        ProgramCommand::SelectBack {
            program: program(),
            decision: evidence(),
        },
        ProgramCommand::Close {
            scope: CloseScope::Session(descriptor().plan.session),
            cause: CloseCause::ClockUnsafe {
                evidence: evidence(),
            },
            basis: vec![evidence()],
        },
    ];
    delta(commands).validate_shape().unwrap();
    let mut p = program();
    p.scope.worker = Pid::new(8).unwrap();
    assert_bad(
        delta(vec![ProgramCommand::Pause {
            program: p,
            decision: evidence(),
        }])
        .validate_shape(),
    );
    let mut r = token();
    r.job.program.cid = Cid::new(hash(0xfe));
    assert_eq!(
        delta(vec![ProgramCommand::Enter {
            program: program(),
            ready: r
        }])
        .validate_shape(),
        Err(ApiError::BindingMismatch("program"))
    );
    let mut r = token();
    r.job.program.instance_id = id(0xfe);
    assert_bad(
        delta(vec![ProgramCommand::Enter {
            program: program(),
            ready: r,
        }])
        .validate_shape(),
    );
    let mut d = descriptor();
    d.plan.config_ref = ConfigRef::new(hash(0xfe));
    assert_bad(
        delta(vec![ProgramCommand::Create {
            descriptor: d,
            plan_basis: vec![],
        }])
        .validate_shape(),
    );
}

#[test]
fn duplicate_actions_precede_derived_id_and_revision_checks_remain_first() {
    let mut d = delta(vec![pause(), pause()]);
    d.actions[1].id = d.actions[0].id;
    assert_eq!(
        d.validate_shape(),
        Err(ApiError::DuplicateId(d.actions[0].id))
    );
    d.next_revision = 2;
    assert_eq!(
        d.validate_shape(),
        Err(ApiError::BindingMismatch("next_revision"))
    );
    let mut d = delta(vec![pause()]);
    d.base_revision = u64::MAX;
    d.next_revision = 0;
    assert_eq!(d.validate_shape(), Err(ApiError::Overflow));
    let mut d = delta(vec![pause()]);
    d.actions[0].id = id(0xfa);
    assert_eq!(d.validate_shape(), Err(ApiError::BindingMismatch("id")));
}

#[test]
fn manager_payload_action_and_guard_limits() {
    let mut d = delta(vec![pause(); 128]);
    d.guards = vec![
        GuardIntent::BlockNewEnter {
            clock_evidence: evidence()
        };
        128
    ];
    d.opaque.bytes = vec![7; 65_536];
    d.validate_shape().unwrap();
    d.actions.push(action(128, ActionBody::Program(pause())));
    assert_bad(d.validate_shape());
    let mut d = delta(vec![]);
    d.guards = vec![
        GuardIntent::BlockNewEnter {
            clock_evidence: evidence()
        };
        129
    ];
    assert_bad(d.validate_shape());
    let mut d = delta(vec![]);
    d.opaque.bytes = vec![1; 65_537];
    assert_bad(d.validate_shape());
    let mut d = delta(vec![]);
    d.opaque.bytes.clear();
    assert_bad(d.validate_shape());
    let mut d = delta(vec![]);
    d.opaque.codec = 0;
    assert_bad(d.validate_shape());
}

#[test]
fn cross_worker_results_are_allowed_but_cross_run_config_are_not() {
    let mut x = delivery();
    x.origin.grant.scope.worker = Pid::new(8).unwrap();
    let mut d = delta(vec![]);
    d.actions
        .push(action(0, ActionBody::OfferRegional(x.clone())));
    d.validate_shape().unwrap();
    d.actions[0].body = ActionBody::ForwardRegional {
        delivery: x.clone(),
        to: RegionId::new(2).unwrap(),
    };
    d.validate_shape().unwrap();
    x.origin.grant.scope.run = hash(0xfe);
    d.actions[0].body = ActionBody::OfferRegional(x);
    assert_bad(d.validate_shape());
}

#[test]
fn replay_full_same_position_batch_and_empty_events() {
    let b = batch();
    b.validate_shape().unwrap();
    assert_ne!(b.lifecycle_ids[0], b.lifecycle_ids[1]);
    let mut empty = b.clone();
    empty.record.events.clear();
    empty.lifecycle_ids.clear();
    empty.validate_shape().unwrap();
    let mut next = empty.clone();
    next.sequence = 2;
    next.previous = Some(empty.id);
    next.id = derive_local_id(&scope(), IdKind::ReplayBatch, 2, next.source).unwrap();
    next.validate_shape().unwrap();
    next.previous = None;
    assert_bad(next.validate_shape());
    let mut d = delta(vec![]);
    d.cause = ManagerCause::Replay(b.clone());
    d.validate_shape().unwrap();
    if let ManagerCause::Replay(x) = &mut d.cause {
        x.origin.grant.scope.worker = Pid::new(8).unwrap();
    }
    assert_bad(d.validate_shape());
}

#[test]
fn replay_event_binding_mutations_and_cardinality() {
    let b = batch();
    let mut x = b.clone();
    x.lifecycle_ids.swap(0, 1);
    assert_bad(x.validate_shape());
    let mut x = b.clone();
    x.lifecycle_ids.pop();
    assert_bad(x.validate_shape());
    let mut x = b.clone();
    x.record.events[1].position.key_us += 1;
    assert_bad(x.validate_shape());
    let mut x = b.clone();
    x.record.events[1] = x.record.events[0].clone();
    x.lifecycle_ids[1] = x.lifecycle_ids[0];
    assert_eq!(
        x.validate_shape(),
        Err(ApiError::InvalidField("lifecycle_ids"))
    );
    let mut x = b.clone();
    x.source_record.source = hash(0xfa);
    assert_bad(x.validate_shape());
    let mut x = b.clone();
    x.sequence = 0;
    assert_bad(x.validate_shape());
    let mut x = b.clone();
    x.record.effects = vec![
        ReplayEffect::Reset {
            resume_position: poc_types::domain::L1Position {
                block: 0,
                tx: 0,
                log: 0
            },
            target_height: 0
        };
        129
    ];
    assert_bad(x.validate_shape());
}

#[test]
fn replay_opaque_input_and_effects_are_preserved_not_deep_validated() {
    let mut b = batch();
    b.record.events.clear();
    b.lifecycle_ids.clear();
    let mut invalid = descriptor();
    invalid.plan.window.h_max = 999;
    assert!(invalid.validate().is_err());
    b.record.effects = vec![ReplayEffect::Duty {
        before: Tail::Next(invalid.clone()),
        after: Tail::Pending(Boundary {
            input: PlanInput {
                cause: BoundaryCause::Force,
                session: SessionId { beta: 99, r: 99 },
                source_region: None,
                window: Window {
                    h_min: 20,
                    h_max: 1,
                },
                reference_block: anchor().block,
            },
            anchor: None,
        }),
    }];
    b.record.input_ref = InputRef::L1(ControlRef {
        position: poc_types::domain::L1Position {
            block: 99,
            tx: 99,
            log: 99,
        },
        kind: ReplayKind::Resume,
        control_seq: 99,
        source_bytes: vec![0xaa; 65_537],
    });
    let original = b.record.clone();
    b.validate_shape().unwrap();
    assert_eq!(b.record, original);
    // The API deliberately neither legitimizes this record nor enforces its total nested bytes.
    let wrapped = h3::replay_batch(
        original.clone(),
        b.source_record,
        b.sequence,
        b.previous,
        b.origin,
    )
    .unwrap();
    assert_eq!(wrapped.record, original);
}

#[test]
fn committed_is_not_applied_and_execution_binds_original_pending() {
    let c = committed(2);
    c.validate_shape().unwrap();
    let x = ActionExecution {
        committed: c.clone(),
        action_id: c.actions_pending[0],
        grant: origin().grant,
    };
    x.validate_shape().unwrap();
    let mut x = x.clone();
    x.action_id = id(0xfb);
    assert_bad(x.validate_shape());
    let mut x = ActionExecution {
        committed: c,
        action_id: committed(2).actions_pending[0],
        grant: origin().grant,
    };
    x.grant.scope.worker = Pid::new(8).unwrap();
    assert_bad(x.validate_shape());
    let mut c = committed(2);
    c.actions_pending[1] = c.actions_pending[0];
    assert_bad(c.validate_shape());
    let mut c = committed(129);
    assert_bad(c.validate_shape());
    c.actions_pending.truncate(128);
    c.validate_shape().unwrap();
    let mut a = applied(2);
    a.receipts.clear();
    assert_bad(a.validate_shape());
    applied(0).validate_shape().unwrap();
}

#[test]
fn applied_receipts_must_exactly_cover_order_root_delta_and_scope() {
    let a = applied(3);
    a.validate_shape().unwrap();
    let mut x = a.clone();
    x.receipts.swap(0, 1);
    assert_bad(x.validate_shape());
    let mut x = a.clone();
    x.receipts[1] = x.receipts[0].clone();
    assert_bad(x.validate_shape());
    let mut x = a.clone();
    x.receipts[0].delta_id = id(0xfb);
    assert_bad(x.validate_shape());
    let mut x = a.clone();
    x.receipts[0].committed_root.hash = hash(0xfb);
    assert_bad(x.validate_shape());
    let mut x = a.clone();
    x.receipts[0].origin.grant.scope.worker = Pid::new(8).unwrap();
    assert_bad(x.validate_shape());
    let mut x = a.clone();
    x.completion_origin.grant.scope.worker = Pid::new(8).unwrap();
    assert_bad(x.validate_shape());
    let mut x = a.clone();
    x.receipts[0].reply = ActionReply::SuppressedByGuard { guard: evidence() };
    x.validate_shape().unwrap();
    assert_eq!(a.committed.actions_pending.len(), 3); // History is retained, not cleared to fake completion.
}

#[test]
fn fail_closed_any_commit_must_bind_and_pending_remains_bounded_unique() {
    for disposition in [
        FailureDisposition::GuardsRetained,
        FailureDisposition::CommittedPending,
        FailureDisposition::RuntimeStopped,
    ] {
        let f = fail(disposition, Some(committed(2)));
        f.validate_shape().unwrap();
        let mut x = f.clone();
        x.committed.as_mut().unwrap().scope.worker = Pid::new(8).unwrap();
        assert_eq!(x.validate_shape(), Err(ApiError::BindingMismatch("scope")));
        let mut x = f.clone();
        x.committed.as_mut().unwrap().delta_id = id(0xfb);
        assert_eq!(
            x.validate_shape(),
            Err(ApiError::BindingMismatch("delta_id"))
        );
    }
    let mut f = fail(FailureDisposition::RuntimeStopped, None);
    f.actions_pending = committed(129).actions_pending;
    assert_bad(f.validate_shape());
    f.actions_pending = vec![id(1), id(1)];
    assert_eq!(f.validate_shape(), Err(ApiError::DuplicateId(id(1))));
    let mut f = fail(FailureDisposition::GuardsRetained, None);
    f.durable_guards.clear();
    assert_bad(f.validate_shape());
    assert_bad(fail(FailureDisposition::CommittedPending, None).validate_shape());
}

#[test]
fn fail_closed_subsets_preserve_obligation_order() {
    let mut f = fail(FailureDisposition::CommittedPending, Some(committed(3)));
    f.actions_pending = vec![
        committed(3).actions_pending[0],
        committed(3).actions_pending[2],
    ];
    f.validate_shape().unwrap();
    f.actions_pending.reverse();
    assert_bad(f.validate_shape());
    f.actions_pending = vec![id(0xfa)];
    assert_bad(f.validate_shape());
    f.actions_pending.clear();
    f.validate_shape().unwrap(); // Metadata only: not a claim actions really completed.
}

fn observation(phase: ProgramPhase) -> ProgramObservation {
    let entered = matches!(phase, ProgramPhase::Running | ProgramPhase::Paused);
    ProgramObservation {
        program: program(),
        terminal: if phase == ProgramPhase::Terminated {
            Some(CloseCause::ClockUnsafe {
                evidence: evidence(),
            })
        } else {
            None
        },
        phase,
        original_entry_us: entered.then_some(10_000_000),
        view: entered.then_some(7),
        history: Some(evidence()),
        timers: Some(evidence()),
        pending_work: Some(evidence()),
        origin: origin(),
    }
}
#[test]
fn phases_pause_and_terminal_are_separate_metadata() {
    for phase in [
        ProgramPhase::Assigned,
        ProgramPhase::Preparing,
        ProgramPhase::Ready,
        ProgramPhase::Running,
        ProgramPhase::Paused,
        ProgramPhase::Terminated,
    ] {
        observation(phase).validate_shape().unwrap();
    }
    let mut x = observation(ProgramPhase::Paused);
    x.original_entry_us = None;
    assert_bad(x.validate_shape());
    let mut x = observation(ProgramPhase::Running);
    x.view = Some(0);
    assert_bad(x.validate_shape());
    let mut x = observation(ProgramPhase::Assigned);
    x.original_entry_us = Some(1);
    assert_bad(x.validate_shape());
    let mut x = observation(ProgramPhase::Terminated);
    x.terminal = None;
    assert_bad(x.validate_shape());
    let mut x = observation(ProgramPhase::Ready);
    x.terminal = Some(CloseCause::Program(CloseReason::Time));
    assert_bad(x.validate_shape());
    let mut x = observation(ProgramPhase::Terminated);
    x.view = Some(3);
    assert_bad(x.validate_shape());
    x.original_entry_us = Some(1);
    x.validate_shape().unwrap();
}

fn proof() -> ProofObserved {
    ProofObserved {
        key: ResultKey {
            cid: descriptor().cid,
            block_hash: chain(1)[0].digest().unwrap(),
        },
        descriptor: descriptor(),
        native_finalization: vec![1],
        check: ProofCheck::CertificateSubject { basis: vec![] },
        native_profile: profile(),
        origin: origin(),
    }
}
#[test]
fn proof_subject_target_and_delivery_have_distinct_shapes() {
    let p = proof();
    p.validate_shape().unwrap(); // Fake bytes pass shape, never called a BLS proof.
    let mut target = p.clone();
    target.check = ProofCheck::TargetBound {
        block: chain(1)[0].clone(),
        basis: vec![evidence()],
    };
    target.validate_shape().unwrap();
    target.key.block_hash = hash(0xfc);
    assert_bad(target.validate_shape());
    let mut x = p.clone();
    x.key.cid = Cid::new(hash(0xfc));
    assert_bad(x.validate_shape());
    let mut x = p.clone();
    x.native_finalization.clear();
    assert_bad(x.validate_shape());
    let mut x = p.clone();
    x.native_finalization = vec![1; 32_768];
    x.validate_shape().unwrap();
    x.native_finalization.push(1);
    assert_bad(x.validate_shape());
    let mut x = p.clone();
    x.native_profile = VerificationProfile::NonCryptographicFixture {
        id: "explicit-test".into(),
    };
    assert_eq!(x.validate_shape(), Err(ApiError::NonNativeProfile));
}

#[test]
fn regional_complete_chain_shape_does_not_execute_or_verify() {
    delivery().validate_shape().unwrap();
    let mut d = delivery();
    d.result.chain = chain(10);
    d.result.block = d.result.chain[9].clone();
    d.validate_shape().unwrap();
    d.result.chain = chain(11);
    d.result.block = d.result.chain[10].clone();
    assert_bad(d.validate_shape());
    let mut d = delivery();
    d.result.chain.clear();
    assert_bad(d.validate_shape());
    let mut d = delivery();
    d.result.chain[0].parent = hash(0xfe);
    assert_bad(d.validate_shape());
    let mut d = delivery();
    d.result.block.parent = hash(0xfe);
    assert_bad(d.validate_shape());
    let mut d = delivery();
    d.validation_basis.clear();
    assert_bad(d.validate_shape());
    let mut d = delivery();
    d.result.chain[1].metadata.height = 3;
    d.result.block = d.result.chain[1].clone();
    d.validate_shape().unwrap(); // physical gap allowed; no automatic execution/padding assertion
    let mut d = delivery();
    d.result.final_proof.clear();
    assert_bad(d.validate_shape());
}

struct CryptoSpy {
    checks: Cell<usize>,
    bases: Cell<usize>,
}
impl FinalVerifier for CryptoSpy {
    fn check(&self, _: &Descriptor, _: &PhysicalBlock, _: &[u8]) -> bool {
        self.checks.set(self.checks.get() + 1);
        false
    }
}
impl VerificationBasis for CryptoSpy {
    fn require(&self, _: &Descriptor, _: &PhysicalBlock) -> InputStatus<()> {
        self.bases.set(self.bases.get() + 1);
        InputStatus::Wait(InputMissing::State(hash(1)))
    }
}
#[test]
fn native_binding_shape_never_invokes_crypto_or_basis() {
    let spy = CryptoSpy {
        checks: Cell::new(0),
        bases: Cell::new(0),
    };
    let mut b = VerifierBinding {
        verifier: &spy,
        basis: &spy,
        profile: profile(),
    };
    validate_native_binding(&b, &config()).unwrap();
    assert_eq!((spy.checks.get(), spy.bases.get()), (0, 0));
    b.profile = VerificationProfile::Native {
        protocol_version: 2,
        verifier_version: 1,
        source_revision: "a".repeat(40),
    };
    assert!(matches!(
        validate_native_binding(&b, &config()),
        Err(ApiError::Gfr(_))
    ));
    b.profile = VerificationProfile::Native {
        protocol_version: 1,
        verifier_version: 1,
        source_revision: "A".repeat(40),
    };
    assert_bad(validate_native_binding(&b, &config()));
    b.profile = VerificationProfile::NonCryptographicFixture {
        id: "explicit-test".into(),
    };
    assert_eq!(
        validate_native_binding(&b, &config()),
        Err(ApiError::NonNativeProfile)
    );
    assert_eq!((spy.checks.get(), spy.bases.get()), (0, 0));
}

struct TestProgramPort {
    response: PortResult<ReadyState>,
    executions: usize,
}
impl ProgramPort for TestProgramPort {
    fn execute(&mut self, _: &ActionExecution, _: &ProgramCommand) -> ActionProgress {
        self.executions += 1;
        ActionProgress::Pending {
            action_id: id(1),
            missing: vec![WaitReason::NativeVerifierBasis],
        }
    }
    fn observe(&self, _: &ProgramHandle) -> PortResult<ProgramObservation> {
        PortResult::Wait(vec![WaitReason::NativeVerifierBasis])
    }
    fn preparation(&self, _: &PreparationJob) -> PortResult<ReadyState> {
        self.response.clone()
    }
}
#[test]
fn h2_consumer_keeps_wait_reject_and_exact_ready_job_separate() {
    let mut port = TestProgramPort {
        response: PortResult::Wait(vec![WaitReason::Work(job())]),
        executions: 0,
    };
    assert!(matches!(
        h2::enter_when_ready(&port, &job()),
        PortResult::Wait(_)
    ));
    port.response = PortResult::Reject(RejectReason::InvalidState);
    assert!(matches!(
        h2::enter_when_ready(&port, &job()),
        PortResult::Reject(RejectReason::InvalidState)
    ));
    port.response = PortResult::Ready(ready());
    assert!(matches!(
        h2::enter_when_ready(&port, &job()),
        PortResult::Ready(ProgramCommand::Enter { .. })
    ));
    assert_eq!(port.executions, 0);
    let mut r = ready();
    r.token.job.generation = 2;
    port.response = PortResult::Ready(r);
    assert!(matches!(
        h2::enter_when_ready(&port, &job()),
        PortResult::Reject(RejectReason::Shape(ApiError::BindingMismatch("job")))
    ));
}

struct TestBindingPort {
    spy: CryptoSpy,
    mode: u8,
}
impl NativeVerifierPort for TestBindingPort {
    fn binding(&self) -> PortResult<VerifierBinding<'_>> {
        match self.mode {
            0 => PortResult::Ready(VerifierBinding {
                verifier: &self.spy,
                basis: &self.spy,
                profile: profile(),
            }),
            1 => PortResult::Wait(vec![WaitReason::NativeVerifierBasis]),
            _ => PortResult::Reject(RejectReason::InvalidSource),
        }
    }
}
#[test]
fn h3_consumer_binding_is_not_proof_or_execution_readiness() {
    let mut p = TestBindingPort {
        spy: CryptoSpy {
            checks: Cell::new(0),
            bases: Cell::new(0),
        },
        mode: 0,
    };
    assert!(matches!(
        h3::native_for_delivery(&p, &config(), &delivery()),
        PortResult::Ready(_)
    ));
    assert_eq!((p.spy.checks.get(), p.spy.bases.get()), (0, 0));
    p.mode = 1;
    assert!(matches!(
        h3::native_for_delivery(&p, &config(), &delivery()),
        PortResult::Wait(_)
    ));
    p.mode = 2;
    assert!(matches!(
        h3::native_for_delivery(&p, &config(), &delivery()),
        PortResult::Reject(RejectReason::InvalidSource)
    ));
}

struct TestFeed {
    value: PortResult<ReplayBatch>,
}
impl ReplayFeedPort for TestFeed {
    fn next(&self, _: Option<&ReplayCursor>) -> PortResult<ReplayBatch> {
        self.value.clone()
    }
}
#[test]
fn h3_feed_preserves_full_record_and_rejects_shape_conflict() {
    let feed = TestFeed {
        value: PortResult::Ready(batch()),
    };
    match h3::read_batch(&feed, None) {
        PortResult::Ready(b) => assert_eq!(b.record, batch().record),
        _ => panic!("expected typed batch"),
    }
    let mut bad = batch();
    bad.lifecycle_ids.clear();
    assert!(matches!(
        h3::read_batch(
            &TestFeed {
                value: PortResult::Ready(bad)
            },
            None
        ),
        PortResult::Reject(_)
    ));
    assert!(matches!(
        h3::read_batch(
            &TestFeed {
                value: PortResult::Wait(vec![WaitReason::NativeVerifierBasis])
            },
            None
        ),
        PortResult::Wait(_)
    ));
}

struct TestCommit {
    calls: usize,
}
impl ManagerCommitPort for TestCommit {
    fn commit(&mut self, _: &ManagerDelta, _: &GrantRef) -> CommitOutcome {
        self.calls += 1;
        CommitOutcome::Committed(committed(1))
    }
    fn complete(
        &mut self,
        _: &RootCommitted,
        _: &[AppliedReceipt],
        _: &GrantRef,
    ) -> CompletionOutcome {
        self.calls += 1;
        CompletionOutcome::Pending(committed(1))
    }
}
#[test]
fn h2_consumer_known_close_with_missing_successor_is_committed_pending() {
    let d = h2::handoff_delta(
        0,
        id(0xd0),
        batch(),
        (
            CloseScope::Session(SessionId { beta: 0, r: 0 }),
            CloseCause::Program(CloseReason::Complete),
            vec![evidence()],
        ),
        (program(), request()),
        OpaqueManagerDelta {
            codec: 1,
            bytes: vec![0],
        },
    )
    .unwrap();
    assert_eq!(d.guards.len(), 1);
    assert!(matches!(
        d.actions[0].body,
        ActionBody::Program(ProgramCommand::Prepare { .. })
    ));
    let mut port = TestCommit { calls: 0 };
    let c = match h2::submit(&mut port, &d, &origin().grant) {
        CommitOutcome::Committed(c) => c,
        _ => panic!("test receipt"),
    };
    assert_eq!(c.actions_pending.len(), 1);
    assert!(matches!(
        h2::complete(&mut port, &c, &[], &origin().grant).unwrap(),
        CompletionOutcome::Pending(_)
    ));
    assert_eq!(port.calls, 2); // Deliberately only a call-count spy, not a simulated runtime.
}

struct ReadyBasisRejectingVerifier {
    checks: Cell<usize>,
    bases: Cell<usize>,
}
impl FinalVerifier for ReadyBasisRejectingVerifier {
    fn check(&self, _: &Descriptor, _: &PhysicalBlock, _: &[u8]) -> bool {
        self.checks.set(self.checks.get() + 1);
        false
    }
}
impl VerificationBasis for ReadyBasisRejectingVerifier {
    fn require(&self, _: &Descriptor, _: &PhysicalBlock) -> InputStatus<()> {
        self.bases.set(self.bases.get() + 1);
        InputStatus::Ready(())
    }
}
#[test]
fn available_historical_basis_never_means_valid_finality_or_execution_ready() {
    let x = ReadyBasisRejectingVerifier {
        checks: Cell::new(0),
        bases: Cell::new(0),
    };
    let b = VerifierBinding {
        verifier: &x,
        basis: &x,
        profile: profile(),
    };
    validate_native_binding(&b, &config()).unwrap();
    assert_eq!((x.checks.get(), x.bases.get()), (0, 0));
    assert!(matches!(
        b.basis.require(&descriptor(), &chain(1)[0]),
        InputStatus::Ready(())
    ));
    assert!(!b.verifier.check(&descriptor(), &chain(1)[0], &[1]));
    assert_eq!((x.checks.get(), x.bases.get()), (1, 1));
    // There is no conversion from InputStatus<()> to the separate H1 ReadyToken/ReadyState.
}

#[test]
fn replay_event_limit_equality_and_plus_one_use_distinct_events() {
    let mut b = batch();
    b.record.events = (0..128)
        .map(|r| LifecycleEvent {
            position: position(),
            body: LifecycleBody::SessionClosed {
                session: SessionId { beta: 0, r },
                reason: CloseReason::Complete,
            },
        })
        .collect();
    b.lifecycle_ids = b
        .record
        .events
        .iter()
        .map(|e| lifecycle_id(e).unwrap())
        .collect();
    b.validate_shape().unwrap();
    b.record.events.push(LifecycleEvent {
        position: position(),
        body: LifecycleBody::SessionClosed {
            session: SessionId { beta: 0, r: 128 },
            reason: CloseReason::Complete,
        },
    });
    b.lifecycle_ids = b
        .record
        .events
        .iter()
        .map(|e| lifecycle_id(e).unwrap())
        .collect();
    assert!(matches!(
        b.validate_shape(),
        Err(ApiError::LimitExceeded {
            field: "events",
            actual: 129,
            max: 128
        })
    ));
}
