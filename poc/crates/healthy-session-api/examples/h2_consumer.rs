//! Compile-only use of the H1 ports by a Manager.

use poc_healthy_session_api::{
    ActionBody, ActionExecution, ActionIntent, ActionProgress, ApiError, AppliedReceipt,
    CloseCause, CloseScope, CommitOutcome, CompletionOutcome, EvidenceRef, GrantRef, GuardIntent,
    IdKind, LocalId, ManagerCause, ManagerCommitPort, ManagerDelta, OpaqueManagerDelta, PortResult,
    PreparationJob, PreparationRequest, ProgramCommand, ProgramHandle, ProgramPort, RejectReason,
    ReplayBatch, RootCommitted, derive_local_id,
};

/// The caller supplies the Manager's encoded selected/preparing state and an existing handle.
/// The preparation request can describe material that is still missing.
pub fn handoff_delta(
    previous_revision: u64,
    id: LocalId,
    batch: ReplayBatch,
    terminal_guard: (CloseScope, CloseCause, Vec<EvidenceRef>),
    successor: (ProgramHandle, PreparationRequest),
    selected_preparing: OpaqueManagerDelta,
) -> Result<ManagerDelta, ApiError> {
    let scope = batch.origin.grant.scope;
    let (closing, cause, basis) = terminal_guard;
    let (program, request) = successor;
    let delta = ManagerDelta {
        id,
        scope,
        base_revision: previous_revision,
        next_revision: previous_revision.checked_add(1).ok_or(ApiError::Overflow)?,
        cause: ManagerCause::Replay(batch),
        opaque: selected_preparing,
        guards: vec![GuardIntent::Terminate {
            scope: closing,
            cause,
            basis,
        }],
        actions: vec![ActionIntent {
            id: derive_local_id(&scope, IdKind::Action, 1, *id.as_hash())?,
            body: ActionBody::Program(ProgramCommand::Prepare { program, request }),
        }],
    };
    delta.validate_shape()?;
    Ok(delta)
}

pub fn submit(
    runtime: &mut dyn ManagerCommitPort,
    delta: &ManagerDelta,
    grant: &GrantRef,
) -> CommitOutcome {
    // Committed may carry pending actions. Keep the original receipt for execution and completion.
    runtime.commit(delta, grant)
}

pub fn run_pending_program_action(
    runtime: &mut dyn ProgramPort,
    committed: &RootCommitted,
    action: &ActionIntent,
    later_grant: GrantRef,
) -> Result<ActionProgress, ApiError> {
    let ActionBody::Program(command) = &action.body else {
        return Err(ApiError::InvalidField("body"));
    };
    let execution = ActionExecution {
        committed: committed.clone(),
        action_id: action.id,
        grant: later_grant,
    };
    execution.validate_shape()?;
    Ok(runtime.execute(&execution, command))
}

/// Applied is a separate receipt covering every action obligation from the original commit.
pub fn complete(
    runtime: &mut dyn ManagerCommitPort,
    committed: &RootCommitted,
    receipts: &[AppliedReceipt],
    later_grant: &GrantRef,
) -> Result<CompletionOutcome, ApiError> {
    let progress = runtime.complete(committed, receipts, later_grant);
    if let CompletionOutcome::Applied(applied) = &progress {
        applied.validate_shape()?;
    }
    Ok(progress)
}

pub fn enter_when_ready(
    runtime: &dyn ProgramPort,
    job: &PreparationJob,
) -> PortResult<ProgramCommand> {
    match runtime.preparation(job) {
        PortResult::Ready(ready) => {
            if let Err(error) = ready.validate_shape() {
                return PortResult::Reject(RejectReason::Shape(error));
            }
            if ready.token.job != *job {
                return PortResult::Reject(RejectReason::Shape(ApiError::BindingMismatch("job")));
            }
            PortResult::Ready(ProgramCommand::Enter {
                program: job.program.clone(),
                ready: ready.token,
            })
        }
        PortResult::Wait(missing) => PortResult::Wait(missing),
        PortResult::Reject(reason) => PortResult::Reject(reason),
    }
}

fn main() {
    println!("Compile-only H2 consumer. Runtime ports are supplied by H1.");
}
