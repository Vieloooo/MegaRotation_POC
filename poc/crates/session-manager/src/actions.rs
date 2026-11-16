use std::collections::BTreeMap;

use poc_types::domain::{Cid, SessionId};

use crate::{
    ACTIONS, ActionEntry, ActionId, ActionIntent, ActionKind, ActionReceipt, ActionStatus,
    CapacityKind, DEPENDENCIES, ManagerError, Phase, Planner, PreparedRef, Program, ReceiptOutcome,
    Selection, State, Update, WaitReason, bound, room, session_key,
};

pub(crate) fn cancel_production(program: &Program, actions: &mut BTreeMap<ActionId, ActionEntry>) {
    for id in [
        program.create_action,
        program.prepare_action,
        program.enter_action,
        program.select_back_action,
    ]
    .into_iter()
    .flatten()
    {
        let action = actions.get_mut(&id).expect("known action");
        if action.status == ActionStatus::Pending {
            action.status = ActionStatus::Cancelled;
        }
    }
}

fn barriers<'a>(state: &'a State, session: &SessionId) -> impl Iterator<Item = ActionId> + 'a {
    let session = session_key(session);
    state
        .programs
        .values()
        .filter(move |program| session_key(&program.descriptor.plan.session) == session)
        .flat_map(|program| {
            [program.pause_action, program.close_action]
                .into_iter()
                .flatten()
        })
        .filter(|id| state.actions[id].status == ActionStatus::Pending)
}

fn dependencies_applied(state: &State, action: &ActionEntry) -> bool {
    action
        .intent
        .dependencies
        .iter()
        .all(|id| state.actions[id].status == ActionStatus::Applied)
}

pub(crate) fn dispatchable(state: &State, action: &ActionEntry, now: u64) -> bool {
    if action.status != ActionStatus::Pending || !dependencies_applied(state, action) {
        return false;
    }
    match &action.intent.kind {
        ActionKind::Pause { .. } | ActionKind::Close { .. } | ActionKind::Publish { .. } => true,
        ActionKind::Create { descriptor, .. } => live(state, &state.programs[&descriptor.cid], now),
        ActionKind::Prepare { cid, .. } => live(state, &state.programs[cid], now),
        ActionKind::Enter { cid, .. } | ActionKind::SelectBack { cid, .. } => {
            let program = &state.programs[cid];
            let session = &program.descriptor.plan.session;
            live(state, program, now)
                && state.guards.clock_allows(session)
                && state.guards.scheduled(session) == Some(*cid)
                && barriers(state, session).next().is_none()
        }
    }
}

fn live(state: &State, program: &Program, now: u64) -> bool {
    let session = &program.descriptor.plan.session;
    program.terminal_cause.is_none()
        && program.selection != Selection::Excluded
        && state.guards.closed_session(session).is_none()
        && state.guards.closed_branch(session.beta).is_none()
        && now < program.stop_us
}

impl Planner<'_> {
    pub(crate) fn issue(
        &mut self,
        kind: ActionKind,
        mut dependencies: Vec<ActionId>,
    ) -> Result<ActionId, ManagerError> {
        dependencies.sort_unstable();
        dependencies.dedup();
        bound(dependencies.len(), CapacityKind::Dependencies, DEPENDENCIES)?;
        room(self.state.actions.len(), CapacityKind::Actions, ACTIONS)?;
        let id = ActionId {
            manager_id: self.binding.manager_id,
            worker: self.binding.worker,
            origin_revision: self.state.revision,
            ordinal: u16::try_from(self.intents.len()).map_err(|_| ManagerError::Overflow)?,
        };
        let intent = ActionIntent {
            id,
            kind,
            dependencies,
        };
        self.state.actions.insert(
            id,
            ActionEntry {
                intent: intent.clone(),
                status: ActionStatus::Pending,
                receipt: None,
                preparation: None,
                last_retry: None,
                issued_at_us: self.state.local_time_us,
                followup: None,
            },
        );
        self.intents.push(intent);
        self.obligations.insert(id);
        Ok(id)
    }

    pub(crate) fn issue_create(&mut self, cid: Cid) -> Result<(), ManagerError> {
        let program = &self.state.programs[&cid];
        let id = self.issue(
            ActionKind::Create {
                descriptor: program.descriptor.clone(),
                provenance_ref: program.provenance_ref,
            },
            Vec::new(),
        )?;
        self.program_mut(cid).create_action = Some(id);
        Ok(())
    }

    pub(crate) fn issue_close(&mut self, cid: Cid) -> Result<ActionId, ManagerError> {
        let program = &self.state.programs[&cid];
        let first_close = program.close_action;
        let dependencies = first_close.into_iter().collect();
        let id = self.issue(
            ActionKind::Close {
                cid,
                instance_ref: program.instance_ref,
                cause: program.terminal_cause.expect("terminal guard installed"),
            },
            dependencies,
        )?;
        if let Some(first_close) = first_close {
            self.state
                .actions
                .get_mut(&first_close)
                .expect("known Close")
                .followup = Some(id);
        }
        self.program_mut(cid).close_action = Some(id);
        Ok(id)
    }

    pub(crate) fn complete_preparation(
        &mut self,
        prepared: &PreparedRef,
    ) -> Result<Update, ManagerError> {
        let action = self
            .state
            .actions
            .get_mut(&prepared.prepare_action)
            .ok_or(ManagerError::UnknownAction)?;
        let ActionKind::Prepare {
            cid,
            instance_ref,
            anchor,
            provenance_ref,
            validation_version,
        } = &action.intent.kind
        else {
            return Err(ManagerError::PreparationBinding);
        };
        if prepared.worker != self.binding.worker
            || prepared.cid != *cid
            || prepared.instance_ref != *instance_ref
            || prepared.anchor != *anchor
            || prepared.config_ref != self.binding.public_config
            || prepared.validation_version != *validation_version
            || prepared.provenance_ref != *provenance_ref
        {
            return Err(ManagerError::PreparationBinding);
        }
        if let Some(previous) = &action.preparation {
            return if previous == prepared {
                Ok(Update::Unchanged)
            } else {
                Err(ManagerError::PreparationBinding)
            };
        }
        let cid = *cid;
        action.preparation = Some(prepared.clone());
        if action.status == ActionStatus::Pending {
            action.status = ActionStatus::Applied;
        }
        self.obligations.insert(prepared.prepare_action);
        let program = self.program_mut(cid);
        program.prepared = Some(prepared.clone());
        if program.terminal_cause.is_none() {
            program.phase = Phase::Ready;
        }
        Ok(Update::Changed)
    }

    pub(crate) fn schedule(&mut self, session: &SessionId) -> Result<Update, ManagerError> {
        let key = session_key(session);
        if !self.state.guards.clock_allows(session) {
            return Ok(Update::Wait(WaitReason::ClockDisabled));
        }
        let wanted = self
            .state
            .programs
            .values()
            .filter(|program| {
                program.descriptor.plan.session == *session
                    && live(&self.state, program, self.state.local_time_us)
            })
            .filter(|program| program.prepared.is_some())
            .max_by_key(|program| program.descriptor.anchor.block.height)
            .map(|program| program.descriptor.cid);
        let Some(cid) = wanted else {
            return Ok(Update::Wait(match self.state.selected(key) {
                Some(descriptor)
                    if self
                        .state
                        .programs
                        .get(&descriptor.cid)
                        .is_some_and(|p| live(&self.state, p, self.state.local_time_us)) =>
                {
                    WaitReason::Preparation {
                        cid: descriptor.cid,
                    }
                }
                _ => WaitReason::NoRunnableCandidate,
            }));
        };
        if self.state.guards.scheduled(session) == Some(cid) {
            return Ok(Update::Unchanged);
        }
        if let Some(old) = self.state.guards.scheduled.insert(key, cid) {
            self.park(old)?;
        }
        self.activate(cid)?;
        Ok(Update::Changed)
    }

    fn park(&mut self, cid: Cid) -> Result<(), ManagerError> {
        let program = &self.state.programs[&cid];
        let previous_pause = program.pause_action;
        let entered = program.entered;
        if let Some(id) = program.select_back_action {
            let action = self.state.actions.get_mut(&id).expect("known select-back");
            if action.status == ActionStatus::Pending {
                action.status = ActionStatus::Cancelled;
            }
        }
        if let Some(id) = previous_pause
            && self.state.actions[&id].status == ActionStatus::Pending
        {
            self.obligations.insert(id);
        } else {
            let instance_ref = self.state.programs[&cid]
                .instance_ref
                .expect("scheduled instance");
            let id = self.issue(ActionKind::Pause { cid, instance_ref }, Vec::new())?;
            self.program_mut(cid).pause_action = Some(id);
        }
        if entered {
            self.program_mut(cid).phase = Phase::Pausing;
        }
        Ok(())
    }

    fn activate(&mut self, cid: Cid) -> Result<Option<ActionId>, ManagerError> {
        let program = &self.state.programs[&cid];
        let mut dependencies: Vec<_> =
            barriers(&self.state, &program.descriptor.plan.session).collect();
        dependencies.extend(program.create_action);
        dependencies.extend(program.prepare_action);
        let instance_ref = program.instance_ref.expect("prepared instance");
        if !program.entered
            && let Some(id) = program.enter_action
        {
            // The original Enter may have been released. Its ID and intent stay fixed while
            // dispatchable() uses the current session's parking obligations as a second gate.
            self.obligations.insert(id);
            self.obligations.extend(dependencies);
            self.program_mut(cid).phase = Phase::Entering;
            return Ok(Some(id));
        }
        let entered = program.entered;
        let command = if entered {
            ActionKind::SelectBack { cid, instance_ref }
        } else {
            ActionKind::Enter {
                cid,
                instance_ref,
                prepared_ref: program.prepared.clone().expect("prepared program"),
            }
        };
        let id = self.issue(command, dependencies)?;
        let program = self.program_mut(cid);
        if entered {
            program.select_back_action = Some(id);
            program.phase = Phase::Resuming;
        } else {
            program.enter_action = Some(id);
            program.phase = Phase::Entering;
        }
        Ok(Some(id))
    }

    pub(crate) fn apply_receipt(
        &mut self,
        receipt: &ActionReceipt,
    ) -> Result<Update, ManagerError> {
        let action = self
            .state
            .actions
            .get(&receipt.id)
            .ok_or(ManagerError::UnknownAction)?;
        if matches!(receipt.outcome, ReceiptOutcome::Retryable { .. }) {
            if action.status != ActionStatus::Pending || action.last_retry.as_ref() == Some(receipt)
            {
                return Ok(Update::Unchanged);
            }
            self.state
                .actions
                .get_mut(&receipt.id)
                .expect("known action")
                .last_retry = Some(receipt.clone());
            self.obligations.insert(receipt.id);
            return Ok(Update::Changed);
        }
        if let Some(original) = &action.receipt {
            return if original == receipt {
                Ok(Update::Unchanged)
            } else {
                Err(ManagerError::ReceiptConflict)
            };
        }
        if !receipt_matches(&self.state, action, receipt.outcome)
            || !dependencies_applied(&self.state, action)
        {
            return Err(ManagerError::ReceiptConflict);
        }
        let cid = action.intent.kind.program();
        let was_cancelled = action.status == ActionStatus::Cancelled;
        let action = self
            .state
            .actions
            .get_mut(&receipt.id)
            .expect("known action");
        action.receipt = Some(receipt.clone());
        if !was_cancelled {
            action.status = ActionStatus::Applied;
        }
        self.obligations.insert(receipt.id);
        let Some(cid) = cid else {
            return Ok(Update::Changed);
        };
        let followup = match receipt.outcome {
            ReceiptOutcome::Created { instance_ref } => {
                let program = self.program_mut(cid);
                program.instance_ref = Some(instance_ref);
                if program.terminal_cause.is_some() {
                    Some(self.issue_close(cid)?)
                } else {
                    let kind = ActionKind::Prepare {
                        cid,
                        instance_ref,
                        anchor: program.descriptor.anchor.clone(),
                        provenance_ref: program.provenance_ref,
                        validation_version: self.binding.preparation_version,
                    };
                    let id = self.issue(kind, vec![receipt.id])?;
                    let program = self.program_mut(cid);
                    program.phase = Phase::Preparing;
                    program.prepare_action = Some(id);
                    Some(id)
                }
            }
            ReceiptOutcome::Entered { entry_time_us, .. } => self.entered(cid, entry_time_us)?,
            ReceiptOutcome::Paused { .. } => {
                let scheduled = self.is_scheduled(cid);
                let program = self.program_mut(cid);
                if program.terminal_cause.is_none()
                    && !scheduled
                    && program.pause_action == Some(receipt.id)
                {
                    program.phase = if program.entered {
                        Phase::Paused
                    } else {
                        Phase::Ready
                    };
                }
                None
            }
            ReceiptOutcome::SelectedBack { .. } => {
                // A superseded SelectBack might have happened before our new parking guard.
                // Record its receipt above while preserving the current schedule and phase.
                if !was_cancelled && self.may_run(cid) {
                    self.program_mut(cid).phase = Phase::Running;
                }
                None
            }
            ReceiptOutcome::Closed { .. } => None,
            ReceiptOutcome::Published { .. } | ReceiptOutcome::Retryable { .. } => {
                unreachable!("receipt kind matched")
            }
        };
        if let Some(id) = followup {
            self.state
                .actions
                .get_mut(&receipt.id)
                .expect("known action")
                .followup = Some(id);
        }
        Ok(Update::Changed)
    }

    fn entered(&mut self, cid: Cid, entry_time_us: u64) -> Result<Option<ActionId>, ManagerError> {
        let program = self.program_mut(cid);
        program.entered = true;
        program.entry_time_us = Some(entry_time_us);
        let pause = program.pause_action;
        if program.terminal_cause.is_some() {
            return Ok(None);
        }
        if self.may_run(cid) {
            if pause.is_none() {
                self.program_mut(cid).phase = Phase::Running;
                return Ok(None);
            }
            // The receipt may refer to the original release, before the latest parking guard.
            // Confirm activation through SelectBack on the already entered instance.
            return self.activate(cid);
        }
        if let Some(id) = pause {
            let pausing = self.state.actions[&id].status == ActionStatus::Pending;
            self.program_mut(cid).phase = if pausing {
                Phase::Pausing
            } else {
                Phase::Paused
            };
        }
        Ok(None)
    }

    fn is_scheduled(&self, cid: Cid) -> bool {
        let program = &self.state.programs[&cid];
        self.state
            .guards
            .scheduled(&program.descriptor.plan.session)
            == Some(cid)
    }

    fn may_run(&self, cid: Cid) -> bool {
        let program = &self.state.programs[&cid];
        self.is_scheduled(cid)
            && self
                .state
                .guards
                .clock_allows(&program.descriptor.plan.session)
            && live(&self.state, program, self.state.local_time_us)
    }

    pub(crate) fn program_mut(&mut self, cid: Cid) -> &mut Program {
        self.state
            .programs
            .get_mut(&cid)
            .expect("registered program")
    }
}

fn receipt_matches(state: &State, action: &ActionEntry, observation: ReceiptOutcome) -> bool {
    match (&action.intent.kind, observation) {
        (ActionKind::Create { .. }, ReceiptOutcome::Created { .. }) => true,
        (
            ActionKind::Enter {
                cid, instance_ref, ..
            },
            ReceiptOutcome::Entered {
                instance_ref: actual,
                entry_time_us,
            },
        ) => {
            *instance_ref == actual
                && entry_time_us >= action.issued_at_us
                && entry_time_us <= state.local_time_us
                && entry_time_us < state.programs[cid].stop_us
        }
        (
            ActionKind::Pause { instance_ref, .. },
            ReceiptOutcome::Paused {
                instance_ref: actual,
            },
        )
        | (
            ActionKind::SelectBack { instance_ref, .. },
            ReceiptOutcome::SelectedBack {
                instance_ref: actual,
            },
        ) => *instance_ref == actual,
        (
            ActionKind::Close { instance_ref, .. },
            ReceiptOutcome::Closed {
                instance_ref: actual,
            },
        ) => *instance_ref == actual,
        (ActionKind::Publish { key, .. }, ReceiptOutcome::Published { key: actual }) => {
            *key == actual
        }
        _ => false,
    }
}
