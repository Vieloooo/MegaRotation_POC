use std::{
    cell::Cell,
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use poc_types::domain::{
    BlockRef, Boundary, BoundaryCause, DomainError, MAX_FIXTURE_ROWS, ProgramConfig,
    authorization_ref, regional_cid,
};
use sim_espresso::MAX_BLOCKS;

use crate::{
    Anchor, ChainEntry, CloseReason, ExecutionAnchor, GfrError, GfrParameters, GlobalCheckpoint,
    GlobalSource, InitInput, InitResult, InputRef, LifecycleBody, LifecycleEvent, Missing,
    PlanError, PlanInput, PlanOutcome, PublicConfig, PublicReplayState, ReplayEffect,
    ReplayPosition, ReplayReads, ReplayRecord, ReplayResult, ReplayServices, ReplayState,
    ReplayStatus, SessionId, SupportingCuts, Tail, VerificationProfile, VerifierBinding, Window,
    compare_positions,
    frontier::{OrderedEvent, Sources, add, input_missing, u64_index},
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
    };
    state.public.tail = match resolve(&state, boundary, services) {
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
    state: &ReplayState,
    boundary: Boundary,
    services: &ReplayServices<'_>,
) -> StepResult<Tail> {
    let plan = match services.plan.resolve(&boundary.input, state) {
        PlanOutcome::Ready(plan) => plan,
        PlanOutcome::Wait(missing) => return Err(StepError::Wait(input_missing(missing))),
        PlanOutcome::None => return Ok(Tail::Pending(boundary)),
        PlanOutcome::Reject(error) => return Err(error.into()),
    };
    let config = &state.public.config;
    let config_ref = config.reference()?;
    if plan.input() != boundary.input
        || plan.config_ref != config_ref
        || plan.authorization_ref
            != authorization_ref(config_ref, config.registry_hash, &boundary.input)?
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
        if let InputRef::L1(control) = &event.input_ref {
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
        InputRef::SessionEnd(_) => regular(&mut staged, &mut record, services)?,
        InputRef::L1(_) => return Err(GfrError::InvalidSource.into()),
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
                regular(state, record, evaluation.services)?;
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
    let successor = resolve(state, boundary, services)?;
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
