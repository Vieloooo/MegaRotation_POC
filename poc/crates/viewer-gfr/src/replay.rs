use std::{
    cell::Cell,
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use poc_types::domain::{
    BlockRef, Boundary, BoundaryCause, DomainError, MAX_FIXTURE_ROWS, ProgramConfig, RuleSpec,
    authorization_ref, regional_cid, derived_anchor_id, ReplayKind, L1Position, InputError,
};
use sim_espresso::MAX_BLOCKS;

use crate::{
    ActivePolicy, Anchor, ChainEntry, CloseReason, HistoricalPlan, L1ControlReader, InputStatus, ControlRef, ExecutionAnchor, GfrError, GfrParameters, GlobalCheckpoint,
    GlobalSource, InitInput, InitResult, InputRef, LifecycleBody, LifecycleEvent, Missing,
    PlanError, PlanInput, PlanOutcome, PublicConfig, PublicReplayState, ReplayEffect,
    ReplayPosition, ReplayReads, ReplayRecord, ReplayResult, ReplayServices, ReplayState,
    ReplayStatus, SessionId, SupportingCuts, Tail, VerificationProfile, VerifierBinding, Window,
    compare_positions,
    frontier::{OrderedEvent, Sources, add, input_missing, u64_index, expiry_key, session_end_key},
    publications::{Candidate, Evaluation, ResultExclusion, interval_unexpired},
    reader::validate_source_id,
};

const MAX_REPLAY_RECORDS: usize = MAX_BLOCKS + MAX_FIXTURE_ROWS + 1;

pub(crate) type StepResult<T> = Result<T, StepError>;

pub(crate) enum StepError {
    Wait(Missing),
    Reject(GfrError),
}

impl From<GfrError> for StepError {
    fn from(error: GfrError) -> Self {
        Self::Reject(error)
    }
}

impl From<DomainError> for StepError {
    fn from(error: DomainError) -> Self {
        Self::Reject(error.into())
    }
}

impl From<PlanError> for StepError {
    fn from(error: PlanError) -> Self {
        Self::Reject(error.into())
    }
}

pub fn init(
    input: &InitInput,
    reads: &ReplayReads<'_>,
    services: &ReplayServices<'_>,
) -> Result<InitResult, GfrError> {
    match input {
        InitInput::Genesis {
            config,
            genesis,
            parameters,
            expected_verification,
        } => genesis_state(
            config,
            genesis,
            *parameters,
            expected_verification,
            reads,
            services,
        ),
        InitInput::Checkpoint(checkpoint) => restore(checkpoint, reads, services),
    }
}

fn genesis_state(
    config: &PublicConfig,
    genesis: &ExecutionAnchor,
    parameters: GfrParameters,
    verification: &VerificationProfile,
    reads: &ReplayReads<'_>,
    services: &ReplayServices<'_>,
) -> Result<InitResult, GfrError> {
    verify_services(config, verification, services)?;
    genesis.validate()?;
    if genesis.anchor != config.genesis
        || genesis.native_hash != config.genesis_native_hash
        || genesis.state.root()? != config.genesis_state_root
    {
        return Err(GfrError::GenesisMismatch);
    }
    if parameters.announcement_us == 0 || parameters.grace_us == 0 || parameters.expiry_us == 0 {
        return Err(GfrError::InvalidParameters);
    }
    validate_source_id(reads.espresso.source_id())?;
    validate_source_id(reads.l1.source_id())?;
    let height = genesis.anchor.block.height;
    let boundary = Boundary {
        input: PlanInput {
            cause: BoundaryCause::Genesis,
            session: SessionId { beta: 0, r: 0 },
            source_region: None,
            window: Window {
                h_min: add(height, 1)?,
                h_max: add(height, 10)?,
            },
            reference_block: genesis.anchor.block.clone(),
        },
        anchor: Some(genesis.anchor.clone()),
    };
    let mut state = ReplayState {
        public: PublicReplayState {
            config: config.clone(),
            genesis: genesis.clone(),
            parameters,
            verification: verification.clone(),
            espresso_source: reads.espresso.source_id().to_vec(),
            l1_source: reads.l1.source_id().to_vec(),
            branch: 0,
            l1_floor: genesis.anchor.block.origin,
            policies: BTreeMap::new(),
            plan_bases: BTreeMap::new(),
            resumes: BTreeMap::new(),
            tail: Tail::Pending(boundary.clone()),
            pc: None,
            selected: BTreeMap::new(),
            closed_sessions: BTreeSet::new(),
            closed_branches: BTreeSet::new(),
            entries: Vec::new(),
            records: Vec::new(),
            publications: Default::default(),
        },
        cuts: SupportingCuts::default(),
        anchors: BTreeMap::from([(genesis.anchor.block.block_id, genesis.clone())]),
        next_q: 0,
        head_anchor: genesis.anchor.block.block_id,
    };
    state.public.tail = match resolve(&mut state, boundary, services, reads.l1, &Cell::new(None)) {
        Ok(tail) => tail,
        Err(StepError::Reject(error)) => return Err(error),
        Err(StepError::Wait(missing)) => return Ok(init_wait(None, vec![missing])),
    };
    if let Tail::Next(descriptor) = &state.public.tail {
        state
            .public
            .selected
            .insert(session_key(&descriptor.plan.session), descriptor.clone());
    }
    Ok(InitResult {
        state: Some(state),
        status: ReplayStatus::Ready,
    })
}

fn init_wait(next_position: Option<ReplayPosition>, missing: Vec<Missing>) -> InitResult {
    InitResult {
        state: None,
        status: ReplayStatus::Wait {
            next_position,
            missing,
        },
    }
}

fn restore(
    checkpoint: &GlobalCheckpoint,
    reads: &ReplayReads<'_>,
    services: &ReplayServices<'_>,
) -> Result<InitResult, GfrError> {
    let expected = &checkpoint.state;
    let binding = &checkpoint.genesis_binding;
    if binding.espresso_source != reads.espresso.source_id()
        || binding.l1_source != reads.l1.source_id()
        || expected.espresso_source != binding.espresso_source
        || expected.l1_source != binding.l1_source
        || expected.config != binding.config
        || expected.genesis != binding.execution
        || expected.parameters != binding.parameters
        || expected.verification != binding.verification
    {
        return Err(GfrError::CheckpointMismatch);
    }
    let initial = genesis_state(
        &binding.config,
        &binding.execution,
        binding.parameters,
        &binding.verification,
        reads,
        services,
    )?;
    let Some(mut state) = initial.state else {
        return Ok(initial);
    };
    if let Some(pc) = &expected.pc {
        let replayed = advance_global(&state, reads, services, Some(pc));
        match replayed.status {
            ReplayStatus::Ready => state = replayed.state,
            ReplayStatus::Wait {
                next_position,
                missing,
            } => return Ok(init_wait(next_position, missing)),
            ReplayStatus::Reject(error) => return Err(error),
            ReplayStatus::UnsupportedControl { .. } => return Err(GfrError::CheckpointMismatch),
        }
    }
    if state.public != *expected || state.cuts != checkpoint.cuts {
        return Err(GfrError::CheckpointMismatch);
    }
    Ok(InitResult {
        state: Some(state),
        status: ReplayStatus::Ready,
    })
}

fn verify_services(
    config: &PublicConfig,
    expected: &VerificationProfile,
    services: &ReplayServices<'_>,
) -> Result<(), GfrError> {
    let config_ref = config.reference()?;
    if config != services.config
        || services.metadata.config_ref() != config_ref
        || services.dep.config_ref() != config_ref
    {
        return Err(GfrError::ConfigMismatch);
    }
    verify_profile(config, expected, services.finality.as_ref())
}

pub(crate) fn verify_profile(
    config: &PublicConfig,
    expected: &VerificationProfile,
    finality: Option<&VerifierBinding<'_>>,
) -> Result<(), GfrError> {
    expected.validate(config)?;
    if let Some(binding) = finality {
        binding.profile.validate(config)?;
        if binding.profile != *expected {
            return Err(GfrError::VerificationProfileMismatch);
        }
    }
    Ok(())
}

fn resolve(
    state: &mut ReplayState,
    boundary: Boundary,
    services: &ReplayServices<'_>,
    l1: &dyn L1ControlReader,
    used_header: &Cell<Option<u64>>,
) -> StepResult<Tail> {
    let basis = if let Some(active) = state.public.policies.get(&boundary.input.session.beta).copied() {
        let height = boundary.input.reference_block.origin.max(active.floor);
        let header = match l1.header(height) {
            InputStatus::Ready(header) if header.height == height => header,
            InputStatus::Ready(_) => return Err(GfrError::InvalidSource.into()),
            InputStatus::Wait(missing) => return Err(StepError::Wait(input_missing(missing))),
            InputStatus::Reject(error) => return Err(GfrError::Input(error).into()),
        };
        used_header.set(Some(used_header.get().map_or(height, |old| old.max(height))));
        Some(HistoricalPlan { active, height, block_hash: header.hash })
    } else {
        None
    };
    let outcome = match basis.as_ref() {
        Some(basis) => services.plan.resolve_historical(&boundary.input, state, basis),
        None => services.plan.resolve(&boundary.input, state),
    };
    let plan = match outcome {
        PlanOutcome::Ready(plan) => plan,
        PlanOutcome::Wait(missing) => return Err(StepError::Wait(input_missing(missing))),
        PlanOutcome::None => return Ok(Tail::Pending(boundary)),
        PlanOutcome::Reject(error) => return Err(error.into()),
    };
    let config = &state.public.config;
    let config_ref = config.reference()?;
    let legacy = authorization_ref(config_ref, config.registry_hash, &boundary.input)?;
    let (expected_ref, expected_rule) = match basis {
        Some(basis) => {
            if basis.active.policy.select(boundary.input.source_region)? != Some(plan.region) {
                return Err(PlanError::PlanMismatch.into());
            }
            let reference = basis.bind(legacy, plan.region)?;
            (reference, RuleSpec::historical_plan(reference))
        }
        None => (legacy, RuleSpec::static_schedule()),
    };
    if plan.input() != boundary.input
        || plan.config_ref != config_ref
        || plan.authorization_ref != expected_ref
        || plan.leader_rule != expected_rule
    {
        return Err(PlanError::PlanMismatch.into());
    }
    let anchor = boundary.anchor.ok_or(GfrError::InvalidSource)?;
    let descriptor = services.plan.descriptor(&plan, &anchor)?;
    let program = ProgramConfig::from_plan(config, &plan)?;
    descriptor.validate()?;
    if descriptor.plan != plan
        || descriptor.anchor != anchor
        || descriptor.cid != regional_cid(&plan.session, anchor.block.block_id, &program)?
    {
        return Err(PlanError::CidMismatch.into());
    }
    if let Some(basis) = basis {
        state.public.plan_bases.insert(descriptor.cid, basis);
    }
    Ok(Tail::Next(descriptor))
}

pub fn advance_global(
    state: &ReplayState,
    reads: &ReplayReads<'_>,
    services: &ReplayServices<'_>,
    through: Option<&ReplayPosition>,
) -> ReplayResult {
    let mut result = ReplayResult {
        state: state.clone(),
        added_records: Vec::new(),
        events: Vec::new(),
        status: ReplayStatus::Ready,
    };
    result.status = advance(&mut result, reads, services, through).unwrap_or_else(|status| status);
    result
}

fn advance(
    result: &mut ReplayResult,
    reads: &ReplayReads<'_>,
    services: &ReplayServices<'_>,
    through: Option<&ReplayPosition>,
) -> Result<ReplayStatus, ReplayStatus> {
    verify_services(
        &result.state.public.config,
        &result.state.public.verification,
        services,
    )
    .map_err(ReplayStatus::Reject)?;
    let sources = Sources::read(reads, &result.state).map_err(ReplayStatus::Reject)?;
    if let Some(bound) = through {
        bound
            .validate()
            .map_err(|error| ReplayStatus::Reject(error.into()))?;
        if let Some(pc) = &result.state.public.pc
            && compare_positions(bound, pc).map_err(ReplayStatus::Reject)? == Ordering::Less
        {
            return Err(ReplayStatus::Reject(GfrError::HistoricalBound));
        }
    }
    while !matches!((result.state.public.pc.as_ref(), through), (Some(pc), Some(bound)) if pc == bound)
    {
        let Some(event) = sources.next_event(&result.state, reads, through)? else {
            break;
        };
        if let InputRef::L1(control) = &event.input_ref
            && control.force_branch().is_none() && control.resume_control().is_none()
        {
            return Ok(ReplayStatus::UnsupportedControl {
                position: event.position,
                kind: control.kind,
            });
        }
        if result.state.public.records.len() >= MAX_REPLAY_RECORDS {
            return Err(ReplayStatus::Reject(
                DomainError::LimitExceeded {
                    limit: MAX_REPLAY_RECORDS,
                    actual: MAX_REPLAY_RECORDS + 1,
                }
                .into(),
            ));
        }
        let (staged, record) = apply_event(&result.state, &sources, &event, reads, services)
            .map_err(|error| match error {
                StepError::Wait(missing) => ReplayStatus::Wait {
                    next_position: Some(event.position.clone()),
                    missing: vec![missing],
                },
                StepError::Reject(error) => ReplayStatus::Reject(error),
            })?;
        result.state = staged;
        result.events.extend_from_slice(&record.events);
        result.added_records.push(record);
    }
    Ok(ReplayStatus::Ready)
}

fn apply_event(
    state: &ReplayState,
    sources: &Sources<'_>,
    event: &OrderedEvent,
    reads: &ReplayReads<'_>,
    services: &ReplayServices<'_>,
) -> StepResult<(ReplayState, ReplayRecord)> {
    let mut staged = state.clone();
    let mut record = ReplayRecord {
        position: event.position.clone(),
        input_ref: event.input_ref.clone(),
        effects: Vec::new(),
        events: Vec::new(),
    };
    let used_origin = Cell::new(None);
    match &event.input_ref {
        InputRef::Espresso(q) => {
            let block = sources
                .espresso
                .get(u64_index(*q)?)
                .ok_or(GfrError::InvalidSource)?;
            let evaluation = Evaluation {
                espresso: sources.espresso,
                reads,
                services,
                used_origin: &used_origin,
            };
            apply_espresso(&mut staged, block, &evaluation, &mut record)?;
            staged.next_q = add(*q, 1)?;
        }
        InputRef::Expire(_) => expire(&mut staged, &mut record)?,
        InputRef::SessionEnd(_) => regular(&mut staged, &mut record, services, reads.l1, &used_origin)?,
        InputRef::L1(control) if control.kind == ReplayKind::Resume => resume(&mut staged, control, &mut record, services, reads.l1, &used_origin)?,
        InputRef::L1(control) => force(&mut staged, control, &mut record, services, reads.l1, &used_origin)?,
    }
    sources.support(&mut staged.cuts, event, used_origin.get())?;
    staged.public.pc = Some(record.position.clone());
    staged.public.records.push(record.clone());
    Ok((staged, record))
}

fn apply_espresso(
    state: &mut ReplayState,
    block: &sim_espresso::FinalizedBlock,
    evaluation: &Evaluation<'_>,
    record: &mut ReplayRecord,
) -> StepResult<()> {
    state.public.publications.index(block)?;
    while let Tail::Next(descriptor) = &state.public.tail {
        let descriptor = descriptor.clone();
        let session = session_key(&descriptor.plan.session);
        if state.public.branch != session.0
            || state.public.closed_branches.contains(&session.0)
            || state.public.closed_sessions.contains(&session)
        {
            return Err(GfrError::InvalidSource.into());
        }
        let head = state.head_height();
        let anchor = state
            .anchors
            .get(&descriptor.anchor.block.block_id)
            .ok_or(GfrError::InvalidSource)?;
        let candidates = evaluation.candidates(&mut state.public, &descriptor, anchor, head)?;
        for candidate in candidates {
            if append(state, candidate, record)?
                && state.head_height() == descriptor.plan.window.h_max
            {
                regular(state, record, evaluation.services, evaluation.reads.l1, evaluation.used_origin)?;
                break;
            }
        }
        if let Tail::Next(next) = &state.public.tail
            && *next == descriptor
        {
            break;
        }
    }
    Ok(())
}

fn append(
    state: &mut ReplayState,
    candidate: Candidate,
    record: &mut ReplayRecord,
) -> StepResult<bool> {
    let head = state.head_height();
    let mut suffix = candidate.blocks.into_iter().peekable();
    while let Some(block) = suffix.next_if(|block| block.virtual_block.metadata.height <= head) {
        let height = block.virtual_block.metadata.height;
        let prior = &state.entry(height).ok_or(GfrError::InvalidSource)?.block;
        if prior != &block {
            return Err(GfrError::InvalidSource.into());
        }
    }
    let Some(first_block) = suffix.peek() else {
        return Ok(false);
    };
    let first = first_block.virtual_block.metadata.height;
    if first != add(head, 1)? {
        return Err(GfrError::InvalidSource.into());
    }
    let last = add(
        head,
        u64::try_from(suffix.len()).map_err(|_| DomainError::Overflow)?,
    )?;
    if !interval_unexpired(
        &state.public.config,
        state.public.parameters,
        first,
        last,
        record.position.key_us,
    )? {
        state
            .public
            .publications
            .results
            .get_mut(&candidate.key)
            .ok_or(GfrError::InvalidSource)?
            .exclusion = Some(ResultExclusion::Expired);
        return Ok(false);
    }
    let limit = MAX_FIXTURE_ROWS;
    let actual = state
        .public
        .entries
        .len()
        .checked_add(suffix.len())
        .ok_or(DomainError::Overflow)?;
    if actual > limit {
        return Err(DomainError::LimitExceeded { limit, actual }.into());
    }
    let source = GlobalSource::Regional(candidate.key);
    for block in suffix {
        if let Some(digest) = block.physical_digest {
            state.head_anchor = digest;
            let metadata = &block.virtual_block.metadata;
            let anchor = Anchor {
                block: BlockRef {
                    height: metadata.height,
                    origin: metadata.origin,
                    block_id: digest,
                },
                timestamp_us: metadata.timestamp_us,
            };
            state.anchors.insert(
                digest,
                ExecutionAnchor {
                    anchor,
                    state: block.state.clone(),
                    native_hash: block.native_hash,
                },
            );
        }
        state.public.entries.push(ChainEntry {
            block,
            source: source.clone(),
            position: record.position.clone(),
        });
    }
    record.effects.push(ReplayEffect::Append {
        first,
        last,
        source,
    });
    Ok(true)
}

fn regular(
    state: &mut ReplayState,
    record: &mut ReplayRecord,
    services: &ReplayServices<'_>,
    l1: &dyn L1ControlReader,
    used_header: &Cell<Option<u64>>,
) -> StepResult<()> {
    let Tail::Next(predecessor) = &state.public.tail else {
        return Err(GfrError::InvalidSource.into());
    };
    let old = predecessor.plan.session.clone();
    let last_height = predecessor.plan.window.h_max;
    let boundary = Boundary {
        input: PlanInput {
            cause: BoundaryCause::Handoff,
            session: SessionId {
                beta: old.beta,
                r: add(old.r, 1)?,
            },
            source_region: Some(predecessor.plan.region),
            window: Window {
                h_min: add(last_height, 1)?,
                h_max: add(last_height, 10)?,
            },
            reference_block: predecessor.anchor.block.clone(),
        },
        anchor: Some(state.last_anchor().clone()),
    };
    let successor = resolve(state, boundary, services, l1, used_header)?;
    if !state.public.closed_sessions.insert(session_key(&old)) {
        return Err(GfrError::InvalidSource.into());
    }
    lifecycle(
        record,
        LifecycleBody::SessionClosed {
            session: old,
            reason: CloseReason::Complete,
        },
    );
    if let Tail::Next(descriptor) = &successor {
        if state
            .public
            .selected
            .insert(session_key(&descriptor.plan.session), descriptor.clone())
            .is_some()
        {
            return Err(GfrError::InvalidSource.into());
        }
        lifecycle(
            record,
            LifecycleBody::SessionAssigned {
                descriptor: descriptor.clone(),
            },
        );
    }
    change_duty(state, record, successor);
    Ok(())
}

fn force(
    state: &mut ReplayState,
    control: &ControlRef,
    record: &mut ReplayRecord,
    services: &ReplayServices<'_>,
    l1: &dyn L1ControlReader,
    used_header: &Cell<Option<u64>>,
) -> StepResult<()> {
    // M2 uses the fixed, public static-schedule-v1 preparation budget.
    const FORCE_INIT_US: u64 = 3_000_000;

    let branch = state.public.branch;
    let target = control.force_branch().ok_or(GfrError::InvalidSource)?;
    if target != branch || state.public.closed_branches.contains(&target) {
        return Ok(());
    }
    let (r, source_region) = match &state.public.tail {
        Tail::Next(current) => {
            let contributed = state.public.entries.last().is_some_and(|entry| {
                matches!(entry.source, GlobalSource::Regional(key) if key.cid == current.cid)
                    && entry.block.physical_digest.is_some()
            });
            (add(current.plan.session.r, u64::from(contributed))?, Some(current.plan.region))
        }
        Tail::Pending(boundary) => (boundary.input.session.r, boundary.input.source_region),
        Tail::Halt { .. } => return Ok(()),
    };
    let anchor = state.last_anchor().clone();
    let key = add(record.position.key_us, FORCE_INIT_US)?;
    let genesis = &state.public.config.genesis;
    let elapsed_slots = key.saturating_sub(genesis.timestamp_us) / state.public.config.l2_slot_us;
    let first = add(anchor.block.height, 1)?.max(add(add(genesis.block.height, elapsed_slots)?, 1)?);
    let beta = add(branch, 1)?;
    let boundary = Boundary {
        input: PlanInput {
            cause: BoundaryCause::Force,
            session: SessionId { beta, r },
            source_region,
            window: Window { h_min: first, h_max: add(first, 9)? },
            reference_block: anchor.block.clone(),
        },
        anchor: Some(anchor),
    };
    state.public.l1_floor = state.public.l1_floor.max(control.position.block);
    if let Some(policy) = control.regional_policy() {
        state.public.policies.insert(beta, ActivePolicy {
            policy,
            force: control.position,
            control_seq: control.control_seq,
            floor: state.public.l1_floor,
        });
    }
    let successor = resolve(state, boundary, services, l1, used_header)?;
    state.public.closed_branches.insert(branch);
    state.public.branch = beta;
    lifecycle(record, LifecycleBody::BranchClosed { branch, reason: CloseReason::Force });
    if let Tail::Next(descriptor) = &successor {
        if state.public.selected.insert(session_key(&descriptor.plan.session), descriptor.clone()).is_some() {
            return Err(GfrError::InvalidSource.into());
        }
        lifecycle(record, LifecycleBody::SessionAssigned { descriptor: descriptor.clone() });
    }
    change_duty(state, record, successor);
    Ok(())
}

fn resume(
    state: &mut ReplayState, control: &ControlRef, record: &mut ReplayRecord,
    services: &ReplayServices<'_>, l1: &dyn L1ControlReader, used_header: &Cell<Option<u64>>,
) -> StepResult<()> {
    let command = control.resume_control().ok_or(GfrError::InvalidSource)?;
    let basis = match l1.resume_cut(control.position) {
        InputStatus::Ready(cut) => cut,
        InputStatus::Wait(_) => return Err(StepError::Wait(Missing::ResumeCut { position: control.position })),
        InputStatus::Reject(error) => return Err(GfrError::Input(error).into()),
    };
    let header = match l1.header(control.position.block) {
        InputStatus::Ready(header) => header,
        InputStatus::Wait(missing) => return Err(StepError::Wait(input_missing(missing))),
        InputStatus::Reject(error) => return Err(GfrError::Input(error).into()),
    };
    if basis.position != control.position || basis.set_position != (L1Position { log: command.set_log, ..control.position })
        || basis.config_ref != state.public.config.reference()? || basis.genesis != state.public.genesis
        || basis.including_block != header.reference() || basis.transaction_hash != command.transaction_hash
        || basis.target.anchor.block.height != command.target_height || basis.blocks.len() > MAX_FIXTURE_ROWS {
        return Err(GfrError::InvalidSource.into());
    }
    let mut native_parent = basis.genesis.native_hash;
    let mut execution = &basis.genesis.state;
    for row in &basis.blocks {
        let data = &row.virtual_block;
        let metadata = &data.metadata;
        if row.physical_digest.is_some() || metadata.height != add(execution.metadata.height,1)?
            || metadata.origin < execution.metadata.origin || metadata.origin > add(execution.metadata.origin,1)?
            || metadata.timestamp_us != poc_core::metadata::l2_timestamp_us(&state.public.config, metadata.height)?
            || row.native_block.parent != native_parent || row.native_block.digest()? != row.native_hash
            || &row.native_block.metadata != metadata || row.native_block.dep != data.dep
            || data.pre_state_root != execution.root()? {
            return Err(GfrError::InvalidSource.into());
        }
        let dep = match services.dep.at(metadata) {
            InputStatus::Ready(dep) => dep,
            InputStatus::Wait(missing) => return Err(StepError::Wait(input_missing(missing))),
            InputStatus::Reject(error) => return Err(GfrError::Input(error).into()),
        };
        let applied = poc_core::execution::apply_with_byte_limit(execution,metadata,&dep,&data.transactions,
            state.public.config.byte_limit as usize).map_err(|_| GfrError::Input(InputError::InvalidSource))?;
        if dep != data.dep || applied.state != row.state || applied.kept != row.native_block.kept
            || row.native_block.post_state_root != row.state.root()? {
            return Err(GfrError::InvalidSource.into());
        }
        execution=&row.state;
        native_parent=row.native_hash;
    }
    let anchor=&basis.target.anchor;
    if execution.metadata != anchor.metadata() || execution != &basis.target.state
        || native_parent != basis.target.native_hash
        || anchor.block.block_id != derived_anchor_id(command.target_height, native_parent) {
        return Err(GfrError::InvalidSource.into());
    }
    let common = state.public.entries.iter().zip(&basis.blocks)
        .take_while(|(old,new)| old.block.native_hash==new.native_hash).count();
    state.public.entries.truncate(common);
    for row in &basis.blocks[common..] {
        state.public.entries.push(ChainEntry {
            block: row.clone(),
            source: GlobalSource::Derived { resume_position: control.position, height: row.state.metadata.height },
            position: record.position.clone(),
        });
    }
    state.head_anchor=anchor.block.block_id;
    state.anchors.insert(state.head_anchor,basis.target.clone());
    let branch=state.public.branch;
    let beta=add(branch,1)?;
    let session=SessionId {beta,r:0};
    let boundary=Boundary {
        input: PlanInput {cause: BoundaryCause::Resume, session: session.clone(), source_region: None,
            window: Window {h_min:add(command.target_height,1)?,h_max:add(command.target_height,10)?},
            reference_block:anchor.block.clone()},
        anchor:Some(anchor.clone()),
    };
    state.public.closed_branches.insert(branch);
    state.public.branch=beta;
    state.public.l1_floor=anchor.block.origin;
    state.public.policies.insert(beta,ActivePolicy {policy:command.policy,force:control.position,
        control_seq:control.control_seq,floor:anchor.block.origin});
    record.effects.push(ReplayEffect::Reset{resume_position:control.position,target_height:command.target_height,cut_digest:basis.digest});
    lifecycle(record,LifecycleBody::BranchClosed{branch,reason:CloseReason::Resume});
    lifecycle(record,LifecycleBody::Resumed{control_seq:control.control_seq,branch:beta,anchor:anchor.clone(),
        derived_prefix:basis.blocks.iter().map(|row|row.virtual_block.clone()).collect()});
    let expiry=expiry_key(&state.public.config,state.public.parameters,boundary.input.window.h_min)?;
    let end=session_end_key(&state.public.config,state.public.parameters,&boundary.input.window)?;
    let tail=if expiry<record.position.key_us || end<record.position.key_us {
        state.public.closed_branches.insert(beta);
        state.public.closed_sessions.insert(session_key(&session));
        lifecycle(record,LifecycleBody::SessionClosed{session,reason:CloseReason::Expired});
        Tail::Halt{boundary,anchor:anchor.clone()}
    } else {
        let tail=resolve(state,boundary,services,l1,used_header)?;
        if let Tail::Next(descriptor)=&tail {
            state.public.selected.insert(session_key(&descriptor.plan.session),descriptor.clone());
            lifecycle(record,LifecycleBody::SessionAssigned{descriptor:descriptor.clone()});
        }
        tail
    };
    state.public.resumes.insert(control.position,basis);
    change_duty(state,record,tail);
    Ok(())
}

fn expire(state: &mut ReplayState, record: &mut ReplayRecord) -> StepResult<()> {
    let boundary = match &state.public.tail {
        Tail::Next(descriptor) => Boundary {
            input: descriptor.plan.input(),
            anchor: Some(descriptor.anchor.clone()),
        },
        Tail::Pending(boundary) => boundary.clone(),
        Tail::Halt { .. } => return Err(GfrError::InvalidSource.into()),
    };
    let branch = state.public.branch;
    if !state.public.closed_branches.insert(branch) {
        return Err(GfrError::InvalidSource.into());
    }
    lifecycle(
        record,
        LifecycleBody::BranchClosed {
            branch,
            reason: CloseReason::Expired,
        },
    );
    let anchor = state.last_anchor().clone();
    change_duty(state, record, Tail::Halt { boundary, anchor });
    Ok(())
}

fn change_duty(state: &mut ReplayState, record: &mut ReplayRecord, after: Tail) {
    let before = std::mem::replace(&mut state.public.tail, after.clone());
    record.effects.push(ReplayEffect::Duty { before, after });
}

fn lifecycle(record: &mut ReplayRecord, body: LifecycleBody) {
    record.events.push(LifecycleEvent {
        position: record.position.clone(),
        body,
    });
}

fn session_key(session: &SessionId) -> (u64, u64) {
    (session.beta, session.r)
}
