use std::{collections::BTreeMap, sync::Arc};

use poc_types::domain::{
    Cid, CloseReason, Descriptor, Hash32, ReplayPosition, ResultKey, SessionId,
};

use crate::{
    AcceptedRecord, ActionEntry, ActionId, CommitFailure, CommitStatus, ManagerBinding, Program,
    ResultTask, SessionState, TerminalCause,
};

pub(crate) type SessionKey = (u64, u64);

pub(crate) fn session_key(session: &SessionId) -> SessionKey {
    (session.beta, session.r)
}

/// Guard projection for the trusted commit owner to install before releasing any action.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GuardState {
    pub(crate) terminals: BTreeMap<Cid, TerminalCause>,
    pub(crate) closed_sessions: BTreeMap<SessionKey, CloseReason>,
    pub(crate) closed_branches: BTreeMap<u64, CloseReason>,
    pub(crate) physical_tips: BTreeMap<Cid, u64>,
    pub(crate) scheduled: BTreeMap<SessionKey, Cid>,
    pub(crate) clock_enabled: bool,
    pub(crate) clock_generation: Option<u64>,
    pub(crate) ready_basis: Option<Hash32>,
    pub(crate) unsafe_basis: Option<Hash32>,
    pub(crate) unsafe_session_fence: Option<SessionKey>,
}

impl GuardState {
    pub fn terminal(&self, cid: Cid) -> Option<TerminalCause> {
        self.terminals.get(&cid).copied()
    }

    pub fn terminals(&self) -> &BTreeMap<Cid, TerminalCause> {
        &self.terminals
    }

    pub fn closed_session(&self, session: &SessionId) -> Option<CloseReason> {
        self.closed_sessions.get(&session_key(session)).copied()
    }

    pub fn closed_sessions(&self) -> &BTreeMap<(u64, u64), CloseReason> {
        &self.closed_sessions
    }

    pub fn closed_branch(&self, beta: u64) -> Option<CloseReason> {
        self.closed_branches.get(&beta).copied()
    }

    pub fn closed_branches(&self) -> &BTreeMap<u64, CloseReason> {
        &self.closed_branches
    }

    pub fn physical_tip(&self, cid: Cid) -> Option<u64> {
        self.physical_tips.get(&cid).copied()
    }

    pub fn physical_tips(&self) -> &BTreeMap<Cid, u64> {
        &self.physical_tips
    }

    pub fn scheduled(&self, session: &SessionId) -> Option<Cid> {
        self.scheduled.get(&session_key(session)).copied()
    }

    pub fn schedules(&self) -> &BTreeMap<(u64, u64), Cid> {
        &self.scheduled
    }

    pub fn clock_enabled(&self) -> bool {
        self.clock_enabled
    }

    pub fn clock_generation(&self) -> Option<u64> {
        self.clock_generation
    }

    pub fn ready_basis(&self) -> Option<Hash32> {
        self.ready_basis
    }

    pub fn unsafe_basis(&self) -> Option<Hash32> {
        self.unsafe_basis
    }

    pub fn unsafe_session_fence(&self) -> Option<(u64, u64)> {
        self.unsafe_session_fence
    }

    pub(crate) fn clock_allows(&self, session: &SessionId) -> bool {
        self.clock_enabled
            && self
                .unsafe_session_fence
                .is_none_or(|fence| session_key(session) > fence)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct State {
    pub revision: u64,
    pub local_time_us: u64,
    pub programs: BTreeMap<Cid, Program>,
    pub sessions: BTreeMap<SessionKey, SessionState>,
    pub guards: GuardState,
    pub next_source_seq: u64,
    pub records: Vec<Arc<AcceptedRecord>>,
    pub resumes: BTreeMap<poc_types::domain::L1Position, Arc<poc_viewer_gfr::ResumeBasis>>,
    pub actions: BTreeMap<ActionId, ActionEntry>,
    pub results: BTreeMap<ResultKey, ResultTask>,
}

impl State {
    pub fn cursor(&self) -> Option<&ReplayPosition> {
        self.records
            .last()
            .map(|accepted| &accepted.record.position)
    }

    pub fn selected(&self, key: SessionKey) -> Option<&Descriptor> {
        self.sessions
            .get(&key)?
            .selected
            .as_ref()
            .map(|selection| &selection.descriptor)
    }

    pub fn record(&self, seq: u64) -> Option<&AcceptedRecord> {
        self.records
            .get(usize::try_from(seq).ok()?)
            .map(Arc::as_ref)
    }
}

#[derive(Clone, Copy)]
pub struct ManagerView<'a> {
    pub(crate) binding: &'a ManagerBinding,
    pub(crate) state: &'a State,
    pub(crate) fault: Option<&'a CommitFailure>,
}

impl<'a> ManagerView<'a> {
    pub fn binding(&self) -> &'a ManagerBinding {
        self.binding
    }

    pub fn revision(&self) -> u64 {
        self.state.revision
    }

    pub fn local_time_us(&self) -> u64 {
        self.state.local_time_us
    }

    pub fn program(&self, cid: Cid) -> Option<&'a Program> {
        self.state.programs.get(&cid)
    }

    pub fn programs(&self) -> &'a BTreeMap<Cid, Program> {
        &self.state.programs
    }

    pub fn selected(&self, session: &SessionId) -> Option<&'a Descriptor> {
        self.state.selected(session_key(session))
    }

    pub fn scheduled(&self, session: &SessionId) -> Option<Cid> {
        self.state.guards.scheduled(session)
    }

    pub fn session(&self, session: &SessionId) -> Option<&'a SessionState> {
        self.state.sessions.get(&session_key(session))
    }

    pub fn cursor(&self) -> Option<&'a ReplayPosition> {
        self.state.cursor()
    }

    pub fn next_source_seq(&self) -> u64 {
        self.state.next_source_seq
    }

    pub fn records(&self) -> &'a [Arc<AcceptedRecord>] {
        &self.state.records
    }

    pub fn record(&self, seq: u64) -> Option<&'a AcceptedRecord> {
        self.state.record(seq)
    }

    pub fn actions(&self) -> &'a BTreeMap<ActionId, ActionEntry> {
        &self.state.actions
    }

    pub fn action(&self, id: ActionId) -> Option<&'a ActionEntry> {
        self.state.actions.get(&id)
    }

    pub fn results(&self) -> &'a BTreeMap<ResultKey, ResultTask> {
        &self.state.results
    }

    pub fn guards(&self) -> &'a GuardState {
        &self.state.guards
    }

    pub fn faulted(&self) -> Option<&'a CommitFailure> {
        self.fault
    }

    /// Status across all accepted actions, including previously held actions.
    pub fn status(&self) -> CommitStatus {
        if self.fault.is_some() {
            CommitStatus::Faulted
        } else {
            let pending = self
                .state
                .actions
                .values()
                .filter(|action| action.status == crate::ActionStatus::Pending)
                .count();
            crate::commit_status(pending)
        }
    }
}
