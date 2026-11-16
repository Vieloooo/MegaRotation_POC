//! Pure H2 decision reducer. All evidence and commit attestations come from trusted adapters.

mod actions;
mod api;
mod replay;
mod state;

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use poc_core::metadata::l2_timestamp_us;
use poc_types::domain::{
    BoundaryCause, CanonicalObject, Cid, Descriptor, DomainError, Hash32, ProgramConfig,
    PublicConfig, ResultKey, regional_cid,
};
use poc_viewer_gfr::MAX_DESCRIPTOR_BYTES;

pub use api::*;
pub use state::{GuardState, ManagerView};
use state::{SessionKey, State, session_key};

const PROGRAMS: usize = 64;
const SESSIONS: usize = 128;
const BRANCHES: usize = 16;
const RECORDS: usize = 2_048;
const ACTIONS: usize = 4_096;
const RESULTS: usize = 256;
const EVENTS: usize = 64;
const EFFECTS: usize = 128;
const LIFECYCLE_BYTES: usize = 1024 * 1024;
const DEPENDENCIES: usize = 64;

pub struct Manager {
    binding: ManagerBinding,
    config: PublicConfig,
    context: Arc<()>,
    state: Arc<State>,
    commits: BTreeMap<u64, AcceptedCommit>,
    fault: Option<CommitFailure>,
}

#[derive(Clone, Debug)]
pub enum PlanOutcome {
    Unchanged(CommitStatus),
    Wait(WaitReason),
    Planned(ManagerDelta),
}

/// Immutable decision and its candidate root/aux projection. Only a trusted owner may attest to
/// having installed it, including guards, before calling `accept_commit`.
#[derive(Clone, Debug)]
pub struct ManagerDelta {
    binding: ManagerBinding,
    context: Arc<()>,
    base: Arc<State>,
    next: Arc<State>,
    cause: Arc<DecisionCause>,
    intents: Vec<ActionIntent>,
    obligations: BTreeSet<ActionId>,
}

impl ManagerDelta {
    pub fn binding(&self) -> &ManagerBinding {
        &self.binding
    }

    pub fn base_revision(&self) -> u64 {
        self.base.revision
    }

    pub fn target_revision(&self) -> u64 {
        self.next.revision
    }

    pub fn cause(&self) -> &DecisionCause {
        &self.cause
    }

    pub fn intents(&self) -> &[ActionIntent] {
        &self.intents
    }

    pub fn guards(&self) -> &GuardState {
        &self.next.guards
    }

    pub fn view(&self) -> ManagerView<'_> {
        ManagerView {
            binding: &self.binding,
            state: &self.next,
            fault: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitRef {
    pub binding: ManagerBinding,
    pub base_revision: u64,
    pub target_revision: u64,
    pub cause: Arc<DecisionCause>,
    pub durable_ref: Hash32,
}

impl CommitRef {
    /// Constructor for the trusted commit owner. This constructor performs no I/O.
    pub fn for_delta(delta: &ManagerDelta, durable_ref: Hash32) -> Self {
        Self {
            binding: delta.binding.clone(),
            base_revision: delta.base_revision(),
            target_revision: delta.target_revision(),
            cause: delta.cause.clone(),
            durable_ref,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitFailure {
    pub base_revision: u64,
    pub target_revision: u64,
    pub cause: Arc<DecisionCause>,
    pub guard_ref: Hash32,
    pub attempted_guards: GuardState,
    pub pending_intents: Vec<ActionIntent>,
}

struct AcceptedCommit {
    commit: CommitRef,
    obligations: BTreeSet<ActionId>,
}

impl Manager {
    pub fn new(
        binding: ManagerBinding,
        config: PublicConfig,
        genesis: VerifiedGenesis,
    ) -> Result<Self, ManagerError> {
        if binding.preparation_version != 1
            || config.reference().map_err(domain_error)? != binding.public_config
        {
            return Err(ManagerError::InvalidBinding);
        }
        let descriptor = genesis.descriptor;
        let plan = &descriptor.plan;
        if plan.cause != BoundaryCause::Genesis
            || session_key(&plan.session) != (0, 0)
            || plan.source_region.is_some()
            || plan.reference_block != config.genesis.block
            || descriptor.anchor != config.genesis
            || plan.window.h_min != add(config.genesis.block.height, 1)?
            || plan.window.h_max != add(config.genesis.block.height, 10)?
        {
            return Err(ManagerError::Malformed);
        }
        validate_descriptor(&binding, &config, &descriptor)?;
        let mut planner = Planner::new(&binding, &config, State::default());
        planner.select_public(descriptor.clone(), None, genesis.source_ref)?;
        if planner.is_member(&descriptor) {
            let cid = descriptor.cid;
            planner.register_program(descriptor, None, genesis.source_ref, Selection::Selected)?;
            planner.issue_create(cid)?;
        }
        Ok(Self {
            state: Arc::new(planner.state),
            binding,
            config,
            context: Arc::new(()),
            commits: BTreeMap::new(),
            fault: None,
        })
    }

    pub fn plan(
        &self,
        input: ManagerInput,
        local_time_us: u64,
    ) -> Result<PlanOutcome, ManagerError> {
        if self.fault.is_some() {
            return Err(ManagerError::RuntimeFaulted);
        }
        if local_time_us < self.state.local_time_us {
            return Err(ManagerError::ClockRegression);
        }
        if let ManagerInput::ConsumeRecord(input) = &input {
            if input.source != self.binding.gfr_source {
                return Err(ManagerError::SourceConflict);
            }
            let expected = self.state.next_source_seq;
            if input.seq < expected {
                let original = self
                    .state
                    .record(input.seq)
                    .ok_or(ManagerError::SourceConflict)?;
                if original.record != input.record {
                    return Err(ManagerError::SourceConflict);
                }
                return Ok(PlanOutcome::Unchanged(
                    self.decision_status(original.revision),
                ));
            }
            if input.seq > expected {
                return Ok(PlanOutcome::Wait(WaitReason::SourceGap {
                    expected,
                    received: input.seq,
                }));
            }
        }
        if let ManagerInput::Receipt(receipt) = &input
            && matches!(receipt.outcome, ReceiptOutcome::Retryable { .. })
            && self.commits.values().any(|accepted| {
                matches!(&accepted.commit.cause.input, ManagerInput::Receipt(prior) if prior == receipt)
            })
        {
            return Ok(PlanOutcome::Unchanged(self.view().status()));
        }
        let mut next = (*self.state).clone();
        next.revision = add(next.revision, 1)?;
        next.local_time_us = local_time_us;
        let mut planner = Planner::new(&self.binding, &self.config, next);
        let update = match &input {
            ManagerInput::RegisterCandidate(candidate) => planner.register_candidate(candidate),
            ManagerInput::ConsumeRecord(record) => planner.consume_record(record),
            ManagerInput::PreparationCompleted(prepared) => planner.complete_preparation(prepared),
            ManagerInput::Receipt(receipt) => planner.apply_receipt(receipt),
            ManagerInput::Schedule(session) => planner.schedule(session),
            ManagerInput::ObserveTime => planner.observe_time(),
            ManagerInput::ClockReady {
                generation,
                basis_ref,
            } => planner.clock_ready(*generation, *basis_ref),
            ManagerInput::ClockUnsafe { basis_ref } => planner.clock_unsafe(*basis_ref),
            ManagerInput::RetainResult {
                key,
                verified_result_ref,
            } => planner.retain_result(*key, *verified_result_ref),
        }?;
        match update {
            Update::Unchanged => Ok(PlanOutcome::Unchanged(self.view().status())),
            Update::Wait(reason) => Ok(PlanOutcome::Wait(reason)),
            Update::Changed => Ok(PlanOutcome::Planned(ManagerDelta {
                binding: self.binding.clone(),
                context: self.context.clone(),
                base: self.state.clone(),
                next: Arc::new(planner.state),
                cause: Arc::new(DecisionCause {
                    input,
                    local_time_us,
                }),
                intents: planner.intents,
                obligations: planner.obligations,
            })),
        }
    }

    pub fn accept_commit(
        &mut self,
        delta: ManagerDelta,
        commit: CommitRef,
    ) -> Result<CommitStatus, ManagerError> {
        if self.fault.is_some() {
            return Err(ManagerError::RuntimeFaulted);
        }
        if !self.owns(&delta)
            || commit.binding != self.binding
            || commit.base_revision != delta.base_revision()
            || commit.target_revision != delta.target_revision()
            || commit.cause != delta.cause
        {
            return Err(ManagerError::CommitBinding);
        }
        if let Some(accepted) = self.commits.get(&delta.target_revision()) {
            if accepted.commit == commit {
                return Ok(self.decision_status(delta.target_revision()));
            }
            return Err(ManagerError::StaleDelta);
        }
        self.check_base(&delta)?;
        let revision = delta.target_revision();
        self.state = delta.next;
        self.commits.insert(
            revision,
            AcceptedCommit {
                commit,
                obligations: delta.obligations,
            },
        );
        Ok(self.decision_status(revision))
    }

    pub fn latch_commit_failure(
        &mut self,
        delta: &ManagerDelta,
        guard_ref: Hash32,
    ) -> Result<(), ManagerError> {
        if let Some(failure) = &self.fault {
            if self.owns(delta)
                && failure.base_revision == delta.base_revision()
                && failure.target_revision == delta.target_revision()
                && failure.cause == delta.cause
                && failure.guard_ref == guard_ref
            {
                return Ok(());
            }
            return Err(ManagerError::RuntimeFaulted);
        }
        self.check_base(delta)?;
        let state = Arc::make_mut(&mut self.state);
        let attempted = delta.guards();
        for (&cid, &cause) in &attempted.terminals {
            state.guards.terminals.entry(cid).or_insert(cause);
            if let Some(program) = state.programs.get_mut(&cid)
                && program.terminal_cause.is_none()
            {
                program.terminal_cause = Some(cause);
                program.phase = Phase::Terminated;
                actions::cancel_production(program, &mut state.actions);
            }
        }
        state
            .guards
            .closed_sessions
            .extend(&attempted.closed_sessions);
        state
            .guards
            .closed_branches
            .extend(&attempted.closed_branches);
        state.guards.physical_tips.extend(&attempted.physical_tips);
        for (&key, &reason) in &state.guards.closed_sessions {
            state
                .sessions
                .entry(key)
                .or_default()
                .closed
                .get_or_insert(reason);
        }
        state.guards.clock_enabled = false;
        state.guards.scheduled.clear();
        state.guards.unsafe_session_fence = attempted.unsafe_session_fence;
        state.guards.unsafe_basis = attempted.unsafe_basis;
        self.fault = Some(CommitFailure {
            base_revision: delta.base_revision(),
            target_revision: delta.target_revision(),
            cause: delta.cause.clone(),
            guard_ref,
            attempted_guards: attempted.clone(),
            pending_intents: delta.intents.clone(),
        });
        Ok(())
    }

    pub fn view(&self) -> ManagerView<'_> {
        ManagerView {
            binding: &self.binding,
            state: &self.state,
            fault: self.fault.as_ref(),
        }
    }

    pub fn dispatchable(&self, local_time_us: u64) -> Vec<&ActionIntent> {
        if self.fault.is_some() || local_time_us < self.state.local_time_us {
            return Vec::new();
        }
        self.state
            .actions
            .values()
            .filter(|action| actions::dispatchable(&self.state, action, local_time_us))
            .map(|action| &action.intent)
            .collect()
    }

    fn check_base(&self, delta: &ManagerDelta) -> Result<(), ManagerError> {
        if !self.owns(delta) {
            return Err(ManagerError::CommitBinding);
        }
        if !Arc::ptr_eq(&delta.base, &self.state) {
            return Err(ManagerError::StaleDelta);
        }
        Ok(())
    }

    fn owns(&self, delta: &ManagerDelta) -> bool {
        delta.binding == self.binding && Arc::ptr_eq(&delta.context, &self.context)
    }

    fn decision_status(&self, revision: u64) -> CommitStatus {
        let Some(accepted) = self.commits.get(&revision) else {
            return self.view().status();
        };
        let mut checked = accepted.obligations.clone();
        let mut visit = checked.iter().copied().collect::<Vec<_>>();
        let mut pending = 0;
        while let Some(id) = visit.pop() {
            let action = &self.state.actions[&id];
            if action.status == ActionStatus::Pending {
                pending += 1;
            }
            for next in action
                .intent
                .dependencies
                .iter()
                .copied()
                .chain(action.followup)
            {
                if checked.insert(next) {
                    visit.push(next);
                }
            }
        }
        commit_status(pending)
    }
}

fn commit_status(pending: usize) -> CommitStatus {
    if pending == 0 {
        CommitStatus::ActionsApplied
    } else {
        CommitStatus::Committed {
            actions_pending: pending,
        }
    }
}

enum Update {
    Changed,
    Unchanged,
    Wait(WaitReason),
}

struct Planner<'a> {
    binding: &'a ManagerBinding,
    config: &'a PublicConfig,
    state: State,
    intents: Vec<ActionIntent>,
    obligations: BTreeSet<ActionId>,
}

impl<'a> Planner<'a> {
    fn new(binding: &'a ManagerBinding, config: &'a PublicConfig, state: State) -> Self {
        Self {
            binding,
            config,
            state,
            intents: Vec::new(),
            obligations: BTreeSet::new(),
        }
    }

    fn is_member(&self, descriptor: &Descriptor) -> bool {
        descriptor
            .plan
            .committee
            .iter()
            .any(|member| member.pid == self.binding.worker)
    }

    fn register_program(
        &mut self,
        descriptor: Descriptor,
        predecessor: Option<Cid>,
        provenance_ref: Hash32,
        selection: Selection,
    ) -> Result<(), ManagerError> {
        room(self.state.programs.len(), CapacityKind::Programs, PROGRAMS)?;
        let stop_us = descriptor_stop(self.config, &descriptor)?;
        self.state.programs.insert(
            descriptor.cid,
            Program {
                descriptor,
                provenance_ref,
                predecessor,
                stop_us,
                selection,
                phase: Phase::Assigned,
                instance_ref: None,
                prepared: None,
                entered: false,
                entry_time_us: None,
                terminal_cause: None,
                create_action: None,
                prepare_action: None,
                enter_action: None,
                pause_action: None,
                select_back_action: None,
                close_action: None,
            },
        );
        Ok(())
    }

    fn register_candidate(
        &mut self,
        candidate: &VerifiedCandidate,
    ) -> Result<Update, ManagerError> {
        let descriptor = &candidate.descriptor;
        if let Some(program) = self.state.programs.get(&descriptor.cid) {
            if program.descriptor != *descriptor {
                return Err(ManagerError::DescriptorConflict);
            }
            if program.predecessor != Some(candidate.predecessor) {
                return Err(ManagerError::WrongPredecessor);
            }
            return Ok(Update::Unchanged);
        }
        validate_descriptor(self.binding, self.config, descriptor)?;
        if !self.is_member(descriptor) {
            return Err(ManagerError::WrongCommittee);
        }
        let session = &descriptor.plan.session;
        let predecessor = self.selected_predecessor(descriptor)?;
        if predecessor.cid != candidate.predecessor {
            return Err(ManagerError::WrongPredecessor);
        }
        let guards = &self.state.guards;
        if guards.closed_session(session).is_some()
            || guards.closed_branch(session.beta).is_some()
            || guards
                .physical_tip(candidate.predecessor)
                .is_some_and(|tip| descriptor.anchor.block.height < tip)
            || guards
                .unsafe_session_fence
                .is_some_and(|fence| session_key(session) <= fence)
        {
            return Err(ManagerError::CandidateConflict);
        }
        if self.state.selected(session_key(session)).is_some()
            || self.state.programs.values().any(|other| {
                other.predecessor == Some(candidate.predecessor)
                    && other.descriptor.anchor.block.height == descriptor.anchor.block.height
                    && other.descriptor.anchor != descriptor.anchor
            })
        {
            return Err(ManagerError::CandidateConflict);
        }
        let cid = descriptor.cid;
        self.register_program(
            descriptor.clone(),
            Some(candidate.predecessor),
            candidate.provenance_ref,
            Selection::Candidate,
        )?;
        if self.state.local_time_us < self.state.programs[&cid].stop_us {
            self.issue_create(cid)?;
        } else {
            self.mark_terminal(cid, TerminalCause::LocalTime);
            self.issue_close(cid)?;
        }
        Ok(Update::Changed)
    }

    fn selected_predecessor(&self, descriptor: &Descriptor) -> Result<&Descriptor, ManagerError> {
        let plan = &descriptor.plan;
        let r = plan
            .session
            .r
            .checked_sub(1)
            .ok_or(ManagerError::WrongPredecessor)?;
        let predecessor = self
            .state
            .selected((plan.session.beta, r))
            .ok_or(ManagerError::WrongPredecessor)?;
        let previous = &predecessor.plan;
        let height = descriptor.anchor.block.height;
        if plan.cause != BoundaryCause::Handoff
            || plan.window.h_min != add(previous.window.h_max, 1)?
            || plan.window.h_max != add(previous.window.h_max, 10)?
            || plan.reference_block != predecessor.anchor.block
            || plan.source_region != Some(previous.region)
            || height > previous.window.h_max
            || (height < previous.window.h_min && descriptor.anchor != predecessor.anchor)
        {
            return Err(ManagerError::WrongPredecessor);
        }
        Ok(predecessor)
    }

    fn mark_terminal(&mut self, cid: Cid, cause: TerminalCause) -> bool {
        let program = self
            .state
            .programs
            .get_mut(&cid)
            .expect("registered program");
        if program.terminal_cause.is_some() {
            return false;
        }
        program.phase = Phase::Terminated;
        program.terminal_cause = Some(cause);
        self.state.guards.terminals.insert(cid, cause);
        let key = session_key(&program.descriptor.plan.session);
        if self.state.guards.scheduled.get(&key) == Some(&cid) {
            self.state.guards.scheduled.remove(&key);
        }
        actions::cancel_production(program, &mut self.state.actions);
        true
    }

    fn observe_time(&mut self) -> Result<Update, ManagerError> {
        let elapsed = self
            .state
            .programs
            .iter()
            .filter_map(|(&cid, program)| {
                (program.terminal_cause.is_none() && self.state.local_time_us >= program.stop_us)
                    .then_some(cid)
            })
            .collect::<Vec<_>>();
        for &cid in &elapsed {
            self.mark_terminal(cid, TerminalCause::LocalTime);
            self.issue_close(cid)?;
        }
        Ok(if elapsed.is_empty() {
            Update::Unchanged
        } else {
            Update::Changed
        })
    }

    fn clock_ready(&mut self, generation: u64, basis: Hash32) -> Result<Update, ManagerError> {
        let guards = &mut self.state.guards;
        if guards.clock_generation == Some(generation) && guards.ready_basis == Some(basis) {
            return Ok(Update::Unchanged);
        }
        if guards.clock_enabled
            || guards
                .clock_generation
                .is_some_and(|current| generation <= current)
        {
            return Err(ManagerError::ClockGeneration);
        }
        guards.clock_enabled = true;
        guards.clock_generation = Some(generation);
        guards.ready_basis = Some(basis);
        Ok(Update::Changed)
    }

    fn clock_unsafe(&mut self, basis_ref: Hash32) -> Result<Update, ManagerError> {
        let guards = &mut self.state.guards;
        let cause = TerminalCause::LocalClockUnsafe {
            generation: guards.clock_generation,
            basis_ref,
        };
        let fence = self
            .state
            .sessions
            .iter()
            .filter_map(|(&key, session)| session.selected.as_ref().map(|_| key))
            .chain(
                self.state
                    .programs
                    .values()
                    .map(|program| session_key(&program.descriptor.plan.session)),
            )
            .chain(guards.unsafe_session_fence)
            .max();
        let open = self
            .state
            .programs
            .iter()
            .filter_map(|(&cid, program)| program.terminal_cause.is_none().then_some(cid))
            .collect::<Vec<_>>();
        if !guards.clock_enabled
            && guards.unsafe_basis == Some(basis_ref)
            && guards.unsafe_session_fence == fence
            && open.is_empty()
        {
            return Ok(Update::Unchanged);
        }
        guards.clock_enabled = false;
        guards.unsafe_basis = Some(basis_ref);
        guards.unsafe_session_fence = fence;
        for cid in open {
            self.mark_terminal(cid, cause);
            self.issue_close(cid)?;
        }
        Ok(Update::Changed)
    }

    fn retain_result(
        &mut self,
        key: ResultKey,
        verified_result_ref: Hash32,
    ) -> Result<Update, ManagerError> {
        if !self.state.programs.contains_key(&key.cid) {
            return Err(ManagerError::Malformed);
        }
        if self.state.results.contains_key(&key) {
            return Ok(Update::Unchanged);
        }
        room(self.state.results.len(), CapacityKind::Results, RESULTS)?;
        let publish_action = self.issue(
            ActionKind::Publish {
                key,
                verified_result_ref,
            },
            Vec::new(),
        )?;
        self.state.results.insert(
            key,
            ResultTask {
                verified_result_ref,
                publish_action,
            },
        );
        Ok(Update::Changed)
    }
}

fn validate_descriptor(
    binding: &ManagerBinding,
    config: &PublicConfig,
    descriptor: &Descriptor,
) -> Result<(), ManagerError> {
    if matches!(
        descriptor.plan.cause,
        BoundaryCause::Resume
    ) {
        return Err(ManagerError::Unsupported);
    }
    if descriptor.plan.config_ref != binding.public_config {
        return Err(ManagerError::InvalidBinding);
    }
    descriptor.validate().map_err(domain_error)?;
    let encoded = descriptor.encode_checked().map_err(domain_error)?;
    bound(
        encoded.len(),
        CapacityKind::DescriptorBytes,
        MAX_DESCRIPTOR_BYTES,
    )?;
    let program = ProgramConfig::from_plan(config, &descriptor.plan).map_err(domain_error)?;
    let actual = regional_cid(
        &descriptor.plan.session,
        descriptor.anchor.block.block_id,
        &program,
    )
    .map_err(domain_error)?;
    if actual != descriptor.cid {
        return Err(ManagerError::CidMismatch);
    }
    descriptor_stop(config, descriptor)?;
    Ok(())
}

fn descriptor_stop(config: &PublicConfig, descriptor: &Descriptor) -> Result<u64, ManagerError> {
    l2_timestamp_us(config, add(descriptor.plan.window.h_max, 1)?).map_err(domain_error)
}

fn domain_error(error: DomainError) -> ManagerError {
    match error {
        DomainError::Overflow => ManagerError::Overflow,
        _ => ManagerError::Malformed,
    }
}

fn add(value: u64, increment: u64) -> Result<u64, ManagerError> {
    value.checked_add(increment).ok_or(ManagerError::Overflow)
}

fn bound(value: usize, kind: CapacityKind, limit: usize) -> Result<(), ManagerError> {
    if value > limit {
        return Err(ManagerError::Capacity { kind, limit });
    }
    Ok(())
}

fn room(count: usize, kind: CapacityKind, limit: usize) -> Result<(), ManagerError> {
    bound(
        count.checked_add(1).ok_or(ManagerError::Overflow)?,
        kind,
        limit,
    )
}
