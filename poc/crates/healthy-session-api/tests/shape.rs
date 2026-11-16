use poc_healthy_session_api::*;
use poc_types::domain::{
    Account, AnchorEvidenceSource, Attributes, BlockRef, BoundaryCause, CommitteeEntry,
    ExecutionState, LifecycleBody, LifecycleEvent, Metadata, PlanRecord, ReplayIndex, ReplayKind,
    RuleSpec, Window,
};
use poc_viewer_gfr::{GlobalSource, InputRef, MAX_PROOF_BYTES, ReplayEffect};

fn hash(byte: u8) -> Hash32 {
    Hash32::new([byte; 32])
}

fn id(byte: u8) -> LocalId {
    LocalId::from_hash(hash(byte)).unwrap()
}

fn scope() -> WorkerScope {
    WorkerScope {
        run: hash(1),
        config: ConfigRef::new(hash(2)),
        worker: Pid::new(0).unwrap(),
        incarnation: 1,
    }
}

fn origin() -> ControlledOrigin {
    ControlledOrigin {
        grant: GrantRef {
            scope: scope(),
            sequence: 1,
            hash: hash(3),
            tick_us: 0,
        },
        root: RootRef {
            sequence: 1,
            hash: hash(4),
        },
        effect_index: 0,
    }
}

fn evidence() -> EvidenceRef {
    EvidenceRef {
        source: hash(5),
        object: hash(6),
    }
}

fn descriptor() -> Descriptor {
    let anchor = Anchor {
        block: BlockRef {
            height: 0,
            origin: 0,
            block_id: hash(7),
        },
        timestamp_us: 0,
    };
    let committee = (0..5)
        .map(|pid| CommitteeEntry {
            pid: Pid::new(pid).unwrap(),
            identity_key: [0; 32],
            vote_key: [0; 96],
            weight: 1,
        })
        .collect::<Vec<_>>();
    let plan = PlanRecord {
        session: SessionId { beta: 0, r: 0 },
        cause: BoundaryCause::Genesis,
        source_region: None,
        region: scope().worker.region(),
        eligible_proposers: committee.iter().map(|entry| entry.pid).collect(),
        committee,
        fault_bound: 1,
        total_weight: 5,
        leader_rule: RuleSpec::static_schedule(),
        window: Window {
            h_min: 1,
            h_max: 10,
        },
        reference_block: anchor.block.clone(),
        config_ref: scope().config,
        authorization_ref: hash(8),
    };
    Descriptor {
        plan,
        anchor,
        cid: Cid::new(hash(9)),
    }
}

fn program() -> ProgramHandle {
    ProgramHandle {
        scope: scope(),
        cid: descriptor().cid,
        instance_id: id(10),
        generation: 1,
    }
}

fn anchor_evidence() -> AnchorEvidence {
    AnchorEvidence {
        source: AnchorEvidenceSource::Genesis(scope().config),
        source_objects: vec![],
    }
}

fn missing_material() -> StateMaterial {
    StateMaterial {
        checkpoint: None,
        physical_prefix: vec![],
        virtual_prefix: vec![],
        input_refs: vec![],
    }
}

fn preparation() -> PreparationRequest {
    PreparationRequest {
        generation: 2,
        anchor: descriptor().anchor,
        anchor_evidence: anchor_evidence(),
        material: missing_material(),
        provenance: vec![],
        validation_profile: hash(11),
    }
}

fn ready() -> ReadyState {
    let anchor = descriptor().anchor;
    let execution_anchor = ExecutionAnchor {
        state: ExecutionState {
            metadata: anchor.metadata(),
            accounts: Default::default(),
        },
        anchor: anchor.clone(),
        native_hash: hash(12),
    };
    ReadyState {
        token: ReadyToken {
            id: id(13),
            job: PreparationJob {
                program: program(),
                id: id(14),
                generation: 2,
            },
            anchor,
            state_root: execution_anchor.state.root().unwrap(),
            native_hash: execution_anchor.native_hash,
            validation_profile: hash(11),
            execution_basis: evidence(),
            origin: origin(),
        },
        execution_anchor,
        ordered_input_refs: vec![evidence()],
    }
}

fn replay() -> ReplayBatch {
    let descriptor = descriptor();
    let session = descriptor.plan.session.clone();
    let position = ReplayPosition {
        key_us: 20,
        kind: ReplayKind::SessionEnd,
        index: ReplayIndex::Session(session.clone()),
    };
    let mut successor = descriptor.clone();
    successor.plan.session.r += 1;
    successor.plan.cause = BoundaryCause::Handoff;
    successor.plan.source_region = Some(descriptor.plan.region);
    successor.plan.window = Window {
        h_min: 11,
        h_max: 20,
    };
    successor.cid = Cid::new(hash(19));
    let events = [
        LifecycleBody::SessionClosed {
            session: session.clone(),
            reason: CloseReason::Time,
        },
        LifecycleBody::SessionAssigned {
            descriptor: successor,
        },
    ]
    .into_iter()
    .map(|body| LifecycleEvent {
        position: position.clone(),
        body,
    })
    .collect::<Vec<_>>();
    let lifecycle_ids = events
        .iter()
        .map(lifecycle_id)
        .collect::<Result<_, _>>()
        .unwrap();
    let source_record = evidence();
    ReplayBatch {
        id: derive_local_id(&scope(), IdKind::ReplayBatch, 1, source_record.source).unwrap(),
        source: source_record.source,
        sequence: 1,
        previous: None,
        source_record,
        record: ReplayRecord {
            position,
            input_ref: InputRef::SessionEnd(session),
            effects: vec![ReplayEffect::Append {
                first: 1,
                last: 1,
                source: GlobalSource::Regional(ResultKey {
                    cid: descriptor.cid,
                    block_hash: hash(15),
                }),
            }],
            events,
        },
        lifecycle_ids,
        origin: origin(),
    }
}

fn action(delta: &ManagerDelta, ordinal: u64, command: ProgramCommand) -> ActionIntent {
    ActionIntent {
        id: derive_local_id(&delta.scope, IdKind::Action, ordinal, *delta.id.as_hash()).unwrap(),
        body: ActionBody::Program(command),
    }
}

fn delta() -> ManagerDelta {
    let batch = replay();
    let LifecycleBody::SessionAssigned {
        descriptor: successor,
    } = &batch.record.events[1].body
    else {
        unreachable!()
    };
    let successor_program = ProgramHandle {
        cid: successor.cid,
        ..program()
    };
    let mut delta = ManagerDelta {
        id: id(16),
        scope: scope(),
        base_revision: 0,
        next_revision: 1,
        cause: ManagerCause::Replay(batch),
        opaque: OpaqueManagerDelta {
            codec: 1,
            bytes: vec![7],
        },
        guards: vec![GuardIntent::Terminate {
            scope: CloseScope::Session(descriptor().plan.session),
            cause: CloseCause::Program(CloseReason::Time),
            basis: vec![evidence()],
        }],
        actions: vec![],
    };
    delta.actions.push(action(
        &delta,
        1,
        ProgramCommand::Prepare {
            program: successor_program,
            request: preparation(),
        },
    ));
    delta
}

fn limit(field: &'static str, max: usize) -> Result<(), ApiError> {
    Err(ApiError::LimitExceeded {
        field,
        actual: max + 1,
        max,
    })
}

#[test]
fn requires_exact_ready_state_and_execution_context() {
    let original = ready();
    assert_eq!(original.validate_shape(), Ok(()));

    let mut mutated = original.clone();
    mutated.token.anchor.block.origin += 1;
    assert_eq!(
        mutated.validate_shape(),
        Err(ApiError::BindingMismatch("anchor"))
    );
    let mut mutated = original.clone();
    mutated.execution_anchor.state.accounts.insert(
        0,
        Account {
            nonce: 0,
            balance: 1,
        },
    );
    assert_eq!(
        mutated.validate_shape(),
        Err(ApiError::BindingMismatch("state_root"))
    );
    let mut mutated = original.clone();
    mutated.token.native_hash = hash(17);
    assert_eq!(
        mutated.validate_shape(),
        Err(ApiError::BindingMismatch("native_hash"))
    );
    let mut mutated = original.clone();
    mutated.token.origin.grant.scope.worker = Pid::new(14).unwrap();
    assert_eq!(
        mutated.validate_shape(),
        Err(ApiError::BindingMismatch("scope"))
    );
    let mut mutated = original.clone();
    mutated.ordered_input_refs.clear();
    assert_eq!(
        mutated.validate_shape(),
        Err(ApiError::InvalidField("ordered_input_refs"))
    );

    let mut delta = delta();
    let mut token = original.token;
    token.job.program.cid = Cid::new(hash(17));
    delta.actions[0] = action(
        &delta,
        1,
        ProgramCommand::Enter {
            program: program(),
            ready: token,
        },
    );
    assert_eq!(
        delta.validate_shape(),
        Err(ApiError::BindingMismatch("program"))
    );
}

#[test]
fn validates_preparation_material_shape_without_execution() {
    let mut request = preparation();
    assert_eq!(request.validate_shape(), Ok(()));
    let metadata = descriptor().anchor.metadata();
    let virtual_block = VirtualBlock {
        dep: vec![
            Attributes {
                metadata: metadata.clone(),
            }
            .encode_raw(),
        ],
        metadata,
        pre_state_root: hash(18),
        transactions: vec![],
    };
    request.material.virtual_prefix = vec![virtual_block; MAX_PREP_BLOCKS];
    request.provenance = vec![evidence(); MAX_REFS];
    request.anchor_evidence.source_objects = vec![vec![]; MAX_REFS];
    request.anchor_evidence.source_objects[0] = vec![1; MAX_OPAQUE_BYTES];
    assert_eq!(request.validate_shape(), Ok(()));

    request.anchor_evidence.source_objects[0].push(1);
    assert_eq!(
        request.validate_shape(),
        limit("source_objects", MAX_OPAQUE_BYTES)
    );
    request.anchor_evidence.source_objects[0].pop();
    request.anchor_evidence.source_objects.push(vec![]);
    assert_eq!(request.validate_shape(), limit("source_objects", MAX_REFS));
    request.anchor_evidence.source_objects.pop();
    request.provenance.push(evidence());
    assert_eq!(request.validate_shape(), limit("provenance", MAX_REFS));
    request.provenance.pop();
    request
        .material
        .virtual_prefix
        .push(request.material.virtual_prefix[0].clone());
    assert_eq!(
        request.validate_shape(),
        limit("virtual_prefix", MAX_PREP_BLOCKS)
    );
    request.material.virtual_prefix.pop();
    request.material.virtual_prefix[0].dep.clear();
    assert!(matches!(request.validate_shape(), Err(ApiError::Domain(_))));
}

#[test]
fn program_observation_keeps_entry_and_terminal_facts_distinct() {
    let mut observation = ProgramObservation {
        program: program(),
        phase: ProgramPhase::Paused,
        original_entry_us: Some(-1),
        view: Some(3),
        terminal: None,
        history: Some(evidence()),
        timers: Some(evidence()),
        pending_work: Some(evidence()),
        origin: origin(),
    };
    assert_eq!(observation.validate_shape(), Ok(()));
    observation.phase = ProgramPhase::Ready;
    assert_eq!(
        observation.validate_shape(),
        Err(ApiError::InvalidField("original_entry_us"))
    );
    observation.phase = ProgramPhase::Terminated;
    assert_eq!(
        observation.validate_shape(),
        Err(ApiError::InvalidField("terminal"))
    );
    observation.terminal = Some(CloseCause::ClockUnsafe {
        evidence: evidence(),
    });
    assert_eq!(observation.validate_shape(), Ok(()));
    observation.original_entry_us = None;
    assert_eq!(
        observation.validate_shape(),
        Err(ApiError::InvalidField("original_entry_us"))
    );
    observation.view = None;
    assert_eq!(observation.validate_shape(), Ok(()));
}

#[test]
fn keeps_multiple_lifecycle_events_and_complete_records() {
    let mut batch = replay();
    assert_ne!(batch.lifecycle_ids[0], batch.lifecycle_ids[1]);
    assert_eq!(batch.validate_shape(), Ok(()));
    batch.lifecycle_ids.swap(0, 1);
    assert_eq!(
        batch.validate_shape(),
        Err(ApiError::BindingMismatch("lifecycle_ids"))
    );
    batch.lifecycle_ids.swap(0, 1);
    let second = batch.lifecycle_ids.pop().unwrap();
    assert_eq!(
        batch.validate_shape(),
        Err(ApiError::BindingMismatch("lifecycle_ids"))
    );
    batch.lifecycle_ids.push(second);
    batch.record.events[1].position.key_us += 1;
    assert_eq!(
        batch.validate_shape(),
        Err(ApiError::BindingMismatch("position"))
    );
    batch.record.events[1] = batch.record.events[0].clone();
    batch.lifecycle_ids[1] = batch.lifecycle_ids[0];
    assert_eq!(
        batch.validate_shape(),
        Err(ApiError::InvalidField("lifecycle_ids"))
    );

    let original = batch.clone();
    batch.previous = Some(batch.id);
    batch.sequence = 2;
    batch.id = derive_local_id(&scope(), IdKind::ReplayBatch, 2, batch.source).unwrap();
    batch.record.events.clear();
    batch.lifecycle_ids.clear();
    assert_eq!(batch.validate_shape(), Ok(()));
    assert_eq!(batch.record.effects, original.record.effects);
    assert_eq!(batch.record.input_ref, original.record.input_ref);
    let effect = batch.record.effects[0].clone();
    batch
        .record
        .effects
        .resize(MAX_REPLAY_EFFECTS, effect.clone());
    assert_eq!(batch.validate_shape(), Ok(()));
    batch.record.effects.push(effect);
    assert_eq!(batch.validate_shape(), limit("effects", MAX_REPLAY_EFFECTS));
    batch.record.effects.pop();
    batch.previous = None;
    assert_eq!(
        batch.validate_shape(),
        Err(ApiError::BindingMismatch("previous"))
    );
}

#[test]
fn manager_checks_revision_and_action_identity_before_action_contents() {
    let mut delta = delta();
    assert_eq!(delta.validate_shape(), Ok(()));
    delta.base_revision = u64::MAX;
    assert_eq!(delta.validate_shape(), Err(ApiError::Overflow));
    delta.base_revision = 1;
    assert_eq!(
        delta.validate_shape(),
        Err(ApiError::BindingMismatch("next_revision"))
    );
    delta.base_revision = 0;
    delta.actions[0].id = id(17);
    delta.actions.push(delta.actions[0].clone());
    assert_eq!(delta.validate_shape(), Err(ApiError::DuplicateId(id(17))));
    delta.actions.pop();
    assert_eq!(delta.validate_shape(), Err(ApiError::BindingMismatch("id")));

    let mut job = ready().token.job;
    job.generation = 0;
    delta.actions[0] = action(
        &delta,
        1,
        ProgramCommand::CancelPrepare {
            job: job.clone(),
            cause: CancelCause::Closed,
        },
    );
    assert_eq!(
        delta.validate_shape(),
        Err(ApiError::InvalidField("generation"))
    );
    job.generation = 1;
    job.program.scope.worker = Pid::new(1).unwrap();
    delta.actions[0] = action(
        &delta,
        1,
        ProgramCommand::Supply {
            job,
            material: missing_material(),
            provenance: vec![],
        },
    );
    assert_eq!(
        delta.validate_shape(),
        Err(ApiError::BindingMismatch("scope"))
    );
}

#[test]
fn lifecycle_commands_can_reuse_the_handle_returned_by_create() {
    let mut delta = delta();
    let committed = committed();
    let assigned = program();
    let create = AppliedReceipt {
        delta_id: committed.delta_id,
        action_id: committed.actions_pending[0],
        committed_root: committed.root,
        origin: origin(),
        reply: ActionReply::Program(ProgramReply::Assigned(assigned)),
    };
    create.validate_shape().unwrap();
    let ActionReply::Program(ProgramReply::Assigned(program)) = create.reply else {
        unreachable!()
    };
    let ready = ready().token;

    let commands = [
        ProgramCommand::Create {
            descriptor: descriptor(),
            plan_basis: vec![evidence()],
        },
        ProgramCommand::Prepare {
            program: program.clone(),
            request: preparation(),
        },
        ProgramCommand::Supply {
            job: ready.job.clone(),
            material: missing_material(),
            provenance: vec![],
        },
        ProgramCommand::Enter {
            program: program.clone(),
            ready,
        },
        ProgramCommand::Pause {
            program: program.clone(),
            decision: evidence(),
        },
        ProgramCommand::SelectBack {
            program: program.clone(),
            decision: evidence(),
        },
        ProgramCommand::Close {
            scope: CloseScope::Cid(program.cid),
            cause: CloseCause::Program(CloseReason::Time),
            basis: vec![evidence()],
        },
    ];
    delta.actions = commands
        .into_iter()
        .enumerate()
        .map(|(index, command)| action(&delta, index as u64 + 1, command))
        .collect();
    assert_eq!(delta.validate_shape(), Ok(()));
}

#[test]
fn checks_manager_resource_bounds() {
    let mut delta = delta();
    delta.opaque.bytes.resize(MAX_OPAQUE_BYTES, 0);
    delta.guards.resize(MAX_GUARDS, delta.guards[0].clone());
    let body = delta.actions[0].body.clone();
    delta.actions = (1..=MAX_ACTIONS as u64)
        .map(|ordinal| ActionIntent {
            id: derive_local_id(&delta.scope, IdKind::Action, ordinal, *delta.id.as_hash())
                .unwrap(),
            body: body.clone(),
        })
        .collect();
    assert_eq!(delta.validate_shape(), Ok(()));

    delta.opaque.bytes.push(0);
    assert_eq!(delta.validate_shape(), limit("bytes", MAX_OPAQUE_BYTES));
    delta.opaque.bytes.pop();
    delta.guards.push(delta.guards[0].clone());
    assert_eq!(delta.validate_shape(), limit("guards", MAX_GUARDS));
    delta.guards.pop();
    delta.actions.push(delta.actions[0].clone());
    assert_eq!(delta.validate_shape(), limit("actions", MAX_ACTIONS));
}

fn committed() -> RootCommitted {
    let delta = delta();
    let ManagerCause::Replay(batch) = delta.cause else {
        unreachable!()
    };
    RootCommitted {
        scope: delta.scope,
        delta_id: delta.id,
        manager_revision: delta.next_revision,
        root: origin().root,
        replay_cursor: Some(ReplayCursor {
            source: batch.source,
            sequence: batch.sequence,
            batch_id: batch.id,
            position: batch.record.position,
        }),
        actions_pending: vec![delta.actions[0].id, id(20)],
    }
}

#[test]
fn completion_covers_the_original_commit_exactly() {
    let committed = committed();
    let origin = origin();
    let receipts = committed
        .actions_pending
        .iter()
        .map(|action_id| AppliedReceipt {
            delta_id: committed.delta_id,
            action_id: *action_id,
            committed_root: committed.root,
            origin,
            reply: ActionReply::SuppressedByGuard { guard: evidence() },
        })
        .collect();
    let mut completion = ActionsApplied {
        committed: committed.clone(),
        receipts,
        completion_origin: origin,
    };
    assert_eq!(completion.validate_shape(), Ok(()));

    let second = completion.receipts.pop().unwrap();
    assert_eq!(
        completion.validate_shape(),
        Err(ApiError::BindingMismatch("receipts"))
    );
    completion.receipts.push(completion.receipts[0].clone());
    assert_eq!(
        completion.validate_shape(),
        Err(ApiError::BindingMismatch("action_id"))
    );
    completion.receipts[1] = second;
    completion.receipts[1].committed_root.hash = hash(21);
    assert_eq!(
        completion.validate_shape(),
        Err(ApiError::BindingMismatch("committed_root"))
    );
    completion.receipts[1].committed_root = committed.root;
    completion.receipts[1].origin.grant.scope.worker = Pid::new(14).unwrap();
    assert_eq!(
        completion.validate_shape(),
        Err(ApiError::BindingMismatch("scope"))
    );

    completion.receipts.clear();
    completion.committed.actions_pending.clear();
    assert_eq!(completion.validate_shape(), Ok(()));
    let execution = ActionExecution {
        committed,
        action_id: id(22),
        grant: origin.grant,
    };
    assert_eq!(
        execution.validate_shape(),
        Err(ApiError::BindingMismatch("action_id"))
    );
}

#[test]
fn fail_closed_checks_commit_binding_for_every_disposition_and_keeps_pending_order() {
    let committed = committed();
    let mut failure = FailClosed {
        scope: committed.scope,
        delta_id: committed.delta_id,
        reason: RejectReason::PersistenceFailure,
        disposition: FailureDisposition::GuardsRetained,
        committed: None,
        durable_guards: vec![evidence()],
        actions_pending: vec![committed.actions_pending[1]],
    };
    assert_eq!(failure.validate_shape(), Ok(()));
    failure.disposition = FailureDisposition::CommittedPending;
    assert_eq!(
        failure.validate_shape(),
        Err(ApiError::InvalidField("committed"))
    );
    failure.committed = Some(committed.clone());
    assert_eq!(failure.validate_shape(), Ok(()));
    failure.actions_pending.push(committed.actions_pending[0]);
    assert_eq!(
        failure.validate_shape(),
        Err(ApiError::BindingMismatch("actions_pending"))
    );

    failure.actions_pending.clear();
    for disposition in [
        FailureDisposition::GuardsRetained,
        FailureDisposition::RuntimeStopped,
    ] {
        failure.disposition = disposition;
        failure.scope.worker = Pid::new(14).unwrap();
        assert_eq!(
            failure.validate_shape(),
            Err(ApiError::BindingMismatch("scope"))
        );
        failure.scope = committed.scope;
        failure.delta_id = id(23);
        assert_eq!(
            failure.validate_shape(),
            Err(ApiError::BindingMismatch("delta_id"))
        );
        failure.delta_id = committed.delta_id;
    }
    failure.committed = None;
    failure.actions_pending = vec![id(24); MAX_ACTIONS + 1];
    assert_eq!(
        failure.validate_shape(),
        limit("actions_pending", MAX_ACTIONS)
    );
    failure.actions_pending.truncate(2);
    assert_eq!(failure.validate_shape(), Err(ApiError::DuplicateId(id(24))));
}

fn native_profile() -> VerificationProfile {
    VerificationProfile::Native {
        protocol_version: 1,
        verifier_version: 1,
        source_revision: "0123456789abcdef0123456789abcdef01234567".into(),
    }
}

fn delivery() -> RegionalDelivery {
    let descriptor = descriptor();
    let mut parent = descriptor.anchor.block.block_id;
    let mut chain = vec![];
    for height in 1..=10 {
        let block = PhysicalBlock {
            parent,
            metadata: Metadata {
                height,
                timestamp_us: height,
                origin: 0,
            },
            pre_state_root: hash(25),
            transactions: vec![],
        };
        parent = block.digest().unwrap();
        chain.push(block);
    }

    RegionalDelivery {
        id: id(26),
        result: RegionalResult {
            descriptor,
            block: chain.last().unwrap().clone(),
            final_proof: vec![1],
            chain,
            anchor_evidence: anchor_evidence(),
        },
        validation_basis: vec![evidence()],
        native_profile: native_profile(),
        origin: origin(),
    }
}

#[test]
fn distinguishes_subject_proof_target_body_and_complete_delivery_shape() {
    let mut delivery = delivery();
    assert_eq!(delivery.validate_shape(), Ok(()));
    let result = &delivery.result;
    let mut proof = ProofObserved {
        key: ResultKey {
            cid: result.descriptor.cid,
            block_hash: result.block.digest().unwrap(),
        },
        descriptor: result.descriptor.clone(),
        native_finalization: vec![1; MAX_PROOF_BYTES],
        check: ProofCheck::CertificateSubject {
            basis: vec![evidence()],
        },
        native_profile: native_profile(),
        origin: origin(),
    };
    assert_eq!(proof.validate_shape(), Ok(()));
    proof.check = ProofCheck::TargetBound {
        block: result.block.clone(),
        basis: vec![evidence()],
    };
    assert_eq!(proof.validate_shape(), Ok(()));
    proof.native_finalization.push(1);
    assert_eq!(
        proof.validate_shape(),
        limit("native_finalization", MAX_PROOF_BYTES)
    );
    proof.native_finalization.pop();
    proof.key.block_hash = hash(27);
    assert_eq!(
        proof.validate_shape(),
        Err(ApiError::BindingMismatch("block_hash"))
    );
    proof.key.cid = Cid::new(hash(27));
    assert_eq!(
        proof.validate_shape(),
        Err(ApiError::BindingMismatch("cid"))
    );

    delivery.result.chain.push(delivery.result.block.clone());
    assert_eq!(delivery.validate_shape(), limit("chain", 10));
    delivery.result.chain.pop();
    delivery.result.chain[3].parent = hash(27);
    assert_eq!(
        delivery.validate_shape(),
        Err(ApiError::BindingMismatch("chain"))
    );
}

#[test]
fn allows_remote_delivery_from_the_same_run_and_config() {
    let mut delta = delta();
    let mut delivery = delivery();
    delivery.origin.grant.scope.worker = Pid::new(14).unwrap();
    delta.actions[0].body = ActionBody::OfferRegional(delivery.clone());
    assert_eq!(delta.validate_shape(), Ok(()));
    delivery.origin.grant.scope.run = hash(28);
    delta.actions[0].body = ActionBody::ForwardRegional {
        delivery,
        to: scope().worker.region(),
    };
    assert_eq!(
        delta.validate_shape(),
        Err(ApiError::BindingMismatch("run"))
    );
}

#[test]
fn native_binding_checks_version_and_shape_without_calling_verifier_or_basis() {
    struct Untouched;
    impl FinalVerifier for Untouched {
        fn check(&self, _: &Descriptor, _: &PhysicalBlock, _: &[u8]) -> bool {
            panic!("shape validation must not invoke finality verification")
        }
    }
    impl VerificationBasis for Untouched {
        fn require(&self, _: &Descriptor, _: &PhysicalBlock) -> InputStatus<()> {
            panic!("shape validation must not require historical inputs")
        }
    }

    let ready = ready();
    let mut config = PublicConfig {
        chain_id: 1,
        rule_version: 1,
        verifier_version: 1,
        registry_hash: hash(28),
        dep_fixture_hash: hash(29),
        l1_fixture_hash: hash(30),
        genesis: ready.token.anchor,
        genesis_state_root: ready.token.state_root,
        genesis_native_hash: ready.token.native_hash,
        l2_slot_us: 1,
        l1_genesis_timestamp_us: 0,
        l1_block_us: 2,
        t_dep_us: 1,
        t_drift_us: 4,
        metadata_window_us: 1,
        byte_limit: 1,
    };
    let mut binding = VerifierBinding {
        verifier: &Untouched,
        basis: &Untouched,
        profile: native_profile(),
    };
    assert_eq!(validate_native_binding(&binding, &config), Ok(()));
    config.verifier_version += 1;
    assert_eq!(
        validate_native_binding(&binding, &config),
        Err(ApiError::Gfr(GfrError::UnsupportedVerificationProfile)),
    );
    let VerificationProfile::Native {
        source_revision, ..
    } = &mut binding.profile
    else {
        unreachable!()
    };
    source_revision.make_ascii_uppercase();
    assert_eq!(
        validate_native_binding(&binding, &config),
        Err(ApiError::InvalidField("source_revision"))
    );
    binding.profile = VerificationProfile::NonCryptographicFixture {
        id: "shape test".into(),
    };
    assert_eq!(
        validate_native_binding(&binding, &config),
        Err(ApiError::NonNativeProfile)
    );
}
