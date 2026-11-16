use std::collections::BTreeSet;
use std::fmt;

use poc_types::domain::{MAX_TRANSACTION_BYTES, validate_dep, validate_transactions};
use poc_viewer_gfr::MAX_PROOF_BYTES;

use crate::{
    API_VERSION, ActionBody, ActionExecution, ActionReply, ActionsApplied, AnchorEvidence,
    ApiError, AppliedReceipt, Canonical, CanonicalObject, CloseCause, CloseScope, ControlledOrigin,
    DomainError, EvidenceRef, FailClosed, FailureDisposition, GfrError, GrantRef, GuardIntent,
    Hash32, IdKind, InputMissing, LocalId, MAX_ACTIONS, MAX_GUARDS, MAX_OPAQUE_BYTES,
    MAX_PREP_BLOCKS, MAX_REFS, MAX_REPLAY_EFFECTS, MAX_REPLAY_EVENTS, ManagerCause, ManagerDelta,
    PreparationJob, PreparationRequest, ProgramCommand, ProgramHandle, ProgramObservation,
    ProgramPhase, ProgramReply, ProofCheck, ProofObserved, PublicConfig, ReadyState, ReadyToken,
    RegionalDelivery, ReplayBatch, ReplayCursor, RootCommitted, RootRef, StateMaterial,
    VerificationProfile, VerifierBinding, WaitReason, WorkerScope, lifecycle_id,
};

impl From<DomainError> for ApiError {
    fn from(error: DomainError) -> Self {
        Self::Domain(error)
    }
}

impl From<GfrError> for ApiError {
    fn from(error: GfrError) -> Self {
        Self::Gfr(error)
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ApiError {}

fn require(valid: bool, field: &'static str) -> Result<(), ApiError> {
    if valid {
        Ok(())
    } else {
        Err(ApiError::InvalidField(field))
    }
}

fn binding(matches: bool, field: &'static str) -> Result<(), ApiError> {
    if matches {
        Ok(())
    } else {
        Err(ApiError::BindingMismatch(field))
    }
}

fn nonzero(hash: &Hash32, field: &'static str) -> Result<(), ApiError> {
    require(*hash != Hash32::new([0; 32]), field)
}

fn limit(actual: usize, max: usize, field: &'static str) -> Result<(), ApiError> {
    if actual <= max {
        Ok(())
    } else {
        Err(ApiError::LimitExceeded { field, actual, max })
    }
}

fn unique_ids(ids: impl IntoIterator<Item = LocalId>) -> Result<(), ApiError> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(ApiError::DuplicateId(id));
        }
    }
    Ok(())
}

fn check_refs(refs: &[EvidenceRef], field: &'static str) -> Result<(), ApiError> {
    limit(refs.len(), MAX_REFS, field)?;
    refs.iter().try_for_each(EvidenceRef::validate_shape)
}

fn check_anchor_evidence(evidence: &AnchorEvidence) -> Result<(), ApiError> {
    limit(evidence.source_objects.len(), MAX_REFS, "source_objects")?;
    for object in &evidence.source_objects {
        limit(object.len(), MAX_OPAQUE_BYTES, "source_objects")?;
    }
    Ok(evidence.validate_structure()?)
}

impl LocalId {
    pub fn from_hash(hash: Hash32) -> Result<Self, ApiError> {
        nonzero(&hash, "local_id")?;
        Ok(Self(hash))
    }

    pub fn as_hash(&self) -> &Hash32 {
        &self.0
    }
}

/// Derive an identity for a local operation using the shared length-framed hash helper.
pub fn derive_local_id(
    scope: &WorkerScope,
    kind: IdKind,
    ordinal: u64,
    subject: Hash32,
) -> Result<LocalId, ApiError> {
    scope.validate_shape()?;
    require(ordinal > 0, "ordinal")?;
    let hash = poc_types::hash_parts(
        "H1/API/ControlId/v1",
        &[
            &API_VERSION.to_be_bytes(),
            &[kind as u8],
            scope.run.as_bytes(),
            scope.config.as_bytes(),
            &scope.worker.encode(),
            &scope.incarnation.to_be_bytes(),
            &ordinal.to_be_bytes(),
            subject.as_bytes(),
        ],
    );
    LocalId::from_hash(Hash32::try_from(hash.as_slice())?)
}

impl WorkerScope {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        nonzero(&self.run, "run")?;
        require(self.incarnation == 1, "incarnation")
    }
}

impl EvidenceRef {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        nonzero(&self.source, "source")?;
        nonzero(&self.object, "object")
    }
}

impl RootRef {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        require(self.sequence > 0, "sequence")?;
        nonzero(&self.hash, "hash")
    }
}

impl GrantRef {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        self.scope.validate_shape()?;
        require(self.sequence > 0, "sequence")?;
        nonzero(&self.hash, "hash")
    }
}

impl ControlledOrigin {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        self.grant.validate_shape()?;
        self.root.validate_shape()
    }
}

impl ProgramHandle {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        self.scope.validate_shape()?;
        require(self.generation == 1, "generation")
    }

    fn validate_in_scope(&self, scope: &WorkerScope) -> Result<(), ApiError> {
        self.validate_shape()?;
        binding(self.scope == *scope, "scope")
    }
}

impl PreparationJob {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        self.program.validate_shape()?;
        require(self.generation > 0, "generation")
    }

    fn validate_in_scope(&self, scope: &WorkerScope) -> Result<(), ApiError> {
        self.validate_shape()?;
        binding(self.program.scope == *scope, "scope")
    }
}

impl StateMaterial {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        if let Some(checkpoint) = &self.checkpoint {
            checkpoint.validate()?;
        }
        limit(
            self.physical_prefix.len(),
            MAX_PREP_BLOCKS,
            "physical_prefix",
        )?;
        for block in &self.physical_prefix {
            block.validate()?;
        }
        limit(self.virtual_prefix.len(), MAX_PREP_BLOCKS, "virtual_prefix")?;
        for block in &self.virtual_prefix {
            block.metadata.validate()?;
            validate_dep(&block.metadata, &block.dep)?;
            validate_transactions(&block.transactions, MAX_TRANSACTION_BYTES)?;
        }
        check_refs(&self.input_refs, "input_refs")
    }
}

impl PreparationRequest {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        require(self.generation > 0, "generation")?;
        self.anchor.validate()?;
        check_anchor_evidence(&self.anchor_evidence)?;
        self.material.validate_shape()?;
        check_refs(&self.provenance, "provenance")?;
        nonzero(&self.validation_profile, "validation_profile")
    }
}

impl ReadyToken {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        self.job.validate_shape()?;
        self.anchor.validate()?;
        nonzero(&self.validation_profile, "validation_profile")?;
        self.execution_basis.validate_shape()?;
        self.origin.validate_shape()?;
        binding(self.origin.grant.scope == self.job.program.scope, "scope")
    }
}

impl ReadyState {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        self.token.validate_shape()?;
        self.execution_anchor.validate()?;
        binding(self.execution_anchor.anchor == self.token.anchor, "anchor")?;
        binding(
            self.execution_anchor.state.root()? == self.token.state_root,
            "state_root",
        )?;
        binding(
            self.execution_anchor.native_hash == self.token.native_hash,
            "native_hash",
        )?;
        require(!self.ordered_input_refs.is_empty(), "ordered_input_refs")?;
        check_refs(&self.ordered_input_refs, "ordered_input_refs")
    }
}

impl CloseCause {
    fn validate_shape(&self) -> Result<(), ApiError> {
        if let Self::ClockUnsafe { evidence } = self {
            evidence.validate_shape()?;
        }
        Ok(())
    }
}

impl CloseScope {
    fn validate_shape(&self) -> Result<(), ApiError> {
        if let Self::Session(session) = self {
            session.validate()?;
        }
        Ok(())
    }
}

impl ProgramObservation {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        self.program.validate_shape()?;
        match self.phase {
            ProgramPhase::Assigned | ProgramPhase::Preparing | ProgramPhase::Ready => {
                require(self.original_entry_us.is_none(), "original_entry_us")?;
                require(self.view.is_none(), "view")?;
            }
            ProgramPhase::Running | ProgramPhase::Paused => {
                require(self.original_entry_us.is_some(), "original_entry_us")?;
                require(self.view.is_some_and(|view| view > 0), "view")?;
            }
            ProgramPhase::Terminated => {
                if let Some(view) = self.view {
                    require(self.original_entry_us.is_some(), "original_entry_us")?;
                    require(view > 0, "view")?;
                }
            }
        }
        require(
            (self.phase == ProgramPhase::Terminated) == self.terminal.is_some(),
            "terminal",
        )?;
        if let Some(cause) = &self.terminal {
            cause.validate_shape()?;
        }
        for evidence in [self.history, self.timers, self.pending_work]
            .into_iter()
            .flatten()
        {
            evidence.validate_shape()?;
        }
        self.origin.validate_shape()?;
        binding(self.program.scope == self.origin.grant.scope, "scope")
    }
}

impl ReplayBatch {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        nonzero(&self.source, "source")?;
        require(self.sequence > 0, "sequence")?;
        self.source_record.validate_shape()?;
        binding(self.source_record.source == self.source, "source")?;

        self.record.position.validate()?;
        limit(self.record.effects.len(), MAX_REPLAY_EFFECTS, "effects")?;
        limit(self.record.events.len(), MAX_REPLAY_EVENTS, "events")?;
        limit(self.lifecycle_ids.len(), MAX_REPLAY_EVENTS, "lifecycle_ids")?;
        binding(
            self.lifecycle_ids.len() == self.record.events.len(),
            "lifecycle_ids",
        )?;
        let mut seen = BTreeSet::new();
        for (event, id) in self.record.events.iter().zip(&self.lifecycle_ids) {
            binding(event.position == self.record.position, "position")?;
            binding(lifecycle_id(event)? == *id, "lifecycle_ids")?;
            require(seen.insert(*id), "lifecycle_ids")?;
        }

        self.origin.validate_shape()?;
        let scope = &self.origin.grant.scope;
        let expected_previous = if self.sequence == 1 {
            None
        } else {
            Some(derive_local_id(
                scope,
                IdKind::ReplayBatch,
                self.sequence - 1,
                self.source,
            )?)
        };
        binding(self.previous == expected_previous, "previous")?;
        binding(
            self.id == derive_local_id(scope, IdKind::ReplayBatch, self.sequence, self.source)?,
            "id",
        )
    }
}

impl ProgramCommand {
    fn validate_in_scope(&self, scope: &WorkerScope) -> Result<(), ApiError> {
        match self {
            Self::Create {
                descriptor,
                plan_basis,
            } => {
                descriptor.encode_checked()?;
                binding(descriptor.plan.config_ref == scope.config, "config_ref")?;
                check_refs(plan_basis, "plan_basis")
            }
            Self::Prepare { program, request } => {
                program.validate_in_scope(scope)?;
                request.validate_shape()
            }
            Self::Supply {
                job,
                material,
                provenance,
            } => {
                job.validate_in_scope(scope)?;
                material.validate_shape()?;
                check_refs(provenance, "provenance")
            }
            Self::CancelPrepare { job, .. } => job.validate_in_scope(scope),
            Self::Enter { program, ready } => {
                program.validate_in_scope(scope)?;
                ready.validate_shape()?;
                binding(ready.job.program == *program, "program")
            }
            Self::Pause { program, decision } | Self::SelectBack { program, decision } => {
                program.validate_in_scope(scope)?;
                decision.validate_shape()
            }
            Self::Close {
                scope,
                cause,
                basis,
            } => {
                scope.validate_shape()?;
                cause.validate_shape()?;
                check_refs(basis, "basis")
            }
        }
    }
}

impl GuardIntent {
    fn validate_in_scope(&self, scope: &WorkerScope) -> Result<(), ApiError> {
        match self {
            Self::Terminate {
                scope,
                cause,
                basis,
            } => {
                scope.validate_shape()?;
                cause.validate_shape()?;
                check_refs(basis, "basis")
            }
            Self::Schedule { session, running } => {
                session.validate()?;
                if let Some(program) = running {
                    program.validate_in_scope(scope)?;
                }
                Ok(())
            }
            Self::BlockNewEnter { clock_evidence } => clock_evidence.validate_shape(),
        }
    }
}

impl ManagerDelta {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        self.scope.validate_shape()?;
        let expected_revision = self
            .base_revision
            .checked_add(1)
            .ok_or(ApiError::Overflow)?;
        binding(self.next_revision == expected_revision, "next_revision")?;
        match &self.cause {
            ManagerCause::Replay(batch) => {
                batch.validate_shape()?;
                binding(batch.origin.grant.scope == self.scope, "scope")?;
            }
            ManagerCause::Regional { evidence, .. } => evidence.validate_shape()?,
            ManagerCause::Local { input, .. } => input.validate_shape()?,
        }
        require(self.opaque.codec != 0, "codec")?;
        limit(self.opaque.bytes.len(), MAX_OPAQUE_BYTES, "bytes")?;
        require(!self.opaque.bytes.is_empty(), "bytes")?;

        limit(self.guards.len(), MAX_GUARDS, "guards")?;
        for guard in &self.guards {
            guard.validate_in_scope(&self.scope)?;
        }
        limit(self.actions.len(), MAX_ACTIONS, "actions")?;
        unique_ids(self.actions.iter().map(|action| action.id))?;
        for (i, action) in self.actions.iter().enumerate() {
            let expected = derive_local_id(
                &self.scope,
                IdKind::Action,
                i as u64 + 1,
                *self.id.as_hash(),
            )?;
            binding(action.id == expected, "id")?;
            match &action.body {
                ActionBody::Program(command) => command.validate_in_scope(&self.scope)?,
                ActionBody::OfferRegional(delivery)
                | ActionBody::ForwardRegional { delivery, .. } => {
                    delivery.validate_shape()?;
                    let source = &delivery.origin.grant.scope;
                    binding(source.run == self.scope.run, "run")?;
                    binding(source.config == self.scope.config, "config")?;
                }
            }
        }
        Ok(())
    }
}

impl ReplayCursor {
    fn validate_shape(&self) -> Result<(), ApiError> {
        require(self.sequence > 0, "sequence")?;
        Ok(self.position.validate()?)
    }
}

impl RootCommitted {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        self.scope.validate_shape()?;
        require(self.manager_revision > 0, "manager_revision")?;
        self.root.validate_shape()?;
        if let Some(cursor) = &self.replay_cursor {
            cursor.validate_shape()?;
        }
        limit(self.actions_pending.len(), MAX_ACTIONS, "actions_pending")?;
        unique_ids(self.actions_pending.iter().copied())
    }
}

impl ActionExecution {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        self.committed.validate_shape()?;
        binding(
            self.committed.actions_pending.contains(&self.action_id),
            "action_id",
        )?;
        self.grant.validate_shape()?;
        binding(self.grant.scope == self.committed.scope, "scope")
    }
}

impl ProgramReply {
    fn validate_in_scope(&self, scope: &WorkerScope) -> Result<(), ApiError> {
        match self {
            Self::Assigned(program)
            | Self::Entered { program, .. }
            | Self::Paused(program)
            | Self::Running(program) => program.validate_in_scope(scope),
            Self::Preparing { job, missing } => {
                job.validate_in_scope(scope)?;
                for reason in missing {
                    match reason {
                        WaitReason::Input(InputMissing::Reference(reference)) => {
                            reference.validate()?
                        }
                        WaitReason::Evidence(evidence) => evidence.validate_shape()?,
                        WaitReason::Work(job) => job.validate_in_scope(scope)?,
                        _ => {}
                    }
                }
                Ok(())
            }
            Self::Ready(state) => {
                state.validate_shape()?;
                binding(state.token.job.program.scope == *scope, "scope")
            }
            Self::Terminated {
                scope,
                cause,
                durable_guard,
            } => {
                scope.validate_shape()?;
                cause.validate_shape()?;
                durable_guard.validate_shape()
            }
            Self::PrepareCancelled(job) => job.validate_in_scope(scope),
        }
    }
}

impl AppliedReceipt {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        self.committed_root.validate_shape()?;
        self.origin.validate_shape()?;
        match &self.reply {
            ActionReply::Program(reply) => reply.validate_in_scope(&self.origin.grant.scope),
            ActionReply::PublicationAccepted { task, .. } => task.validate_shape(),
            ActionReply::ForwardAccepted { custody, .. } => custody.validate_shape(),
            ActionReply::SuppressedByGuard { guard } => guard.validate_shape(),
        }
    }
}

impl ActionsApplied {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        self.committed.validate_shape()?;
        limit(self.receipts.len(), MAX_ACTIONS, "receipts")?;
        binding(
            self.receipts.len() == self.committed.actions_pending.len(),
            "receipts",
        )?;
        for (receipt, action_id) in self.receipts.iter().zip(&self.committed.actions_pending) {
            receipt.validate_shape()?;
            binding(receipt.delta_id == self.committed.delta_id, "delta_id")?;
            binding(receipt.action_id == *action_id, "action_id")?;
            binding(
                receipt.committed_root == self.committed.root,
                "committed_root",
            )?;
            binding(receipt.origin.grant.scope == self.committed.scope, "scope")?;
        }
        self.completion_origin.validate_shape()?;
        binding(
            self.completion_origin.grant.scope == self.committed.scope,
            "scope",
        )
    }
}

impl FailClosed {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        self.scope.validate_shape()?;
        if let Some(committed) = &self.committed {
            committed.validate_shape()?;
            binding(committed.scope == self.scope, "scope")?;
            binding(committed.delta_id == self.delta_id, "delta_id")?;
        }
        check_refs(&self.durable_guards, "durable_guards")?;
        match self.disposition {
            FailureDisposition::GuardsRetained => {
                require(!self.durable_guards.is_empty(), "durable_guards")?
            }
            FailureDisposition::CommittedPending => require(self.committed.is_some(), "committed")?,
            FailureDisposition::RuntimeStopped => {}
        }
        limit(self.actions_pending.len(), MAX_ACTIONS, "actions_pending")?;
        unique_ids(self.actions_pending.iter().copied())?;
        if let Some(committed) = &self.committed {
            let mut original = committed.actions_pending.iter();
            for id in &self.actions_pending {
                binding(original.any(|pending| pending == id), "actions_pending")?;
            }
        }
        Ok(())
    }
}

fn check_native_profile(profile: &VerificationProfile) -> Result<(), ApiError> {
    let VerificationProfile::Native {
        protocol_version,
        verifier_version,
        source_revision,
    } = profile
    else {
        return Err(ApiError::NonNativeProfile);
    };
    require(*protocol_version > 0, "protocol_version")?;
    require(*verifier_version > 0, "verifier_version")?;
    require(
        source_revision.len() == 40
            && source_revision
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
        "source_revision",
    )
}

/// Check the native profile's shape and version compatibility with the historical config.
/// The verifier and basis are supplied by the consumer.
pub fn validate_native_binding(
    binding: &VerifierBinding<'_>,
    config: &PublicConfig,
) -> Result<(), ApiError> {
    check_native_profile(&binding.profile)?;
    Ok(binding.profile.validate(config)?)
}

fn check_proof(proof: &[u8], field: &'static str) -> Result<(), ApiError> {
    limit(proof.len(), MAX_PROOF_BYTES, field)?;
    require(!proof.is_empty(), field)
}

impl ProofObserved {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        self.descriptor.encode_checked()?;
        binding(self.key.cid == self.descriptor.cid, "cid")?;
        check_proof(&self.native_finalization, "native_finalization")?;
        match &self.check {
            ProofCheck::CertificateSubject { basis } => check_refs(basis, "basis")?,
            ProofCheck::TargetBound { block, basis } => {
                binding(block.digest()? == self.key.block_hash, "block_hash")?;
                check_refs(basis, "basis")?;
            }
        }
        check_native_profile(&self.native_profile)?;
        self.origin.validate_shape()?;
        binding(
            self.origin.grant.scope.config == self.descriptor.plan.config_ref,
            "config_ref",
        )
    }
}

impl RegionalDelivery {
    pub fn validate_shape(&self) -> Result<(), ApiError> {
        let result = &self.result;
        result.descriptor.encode_checked()?;
        result.block.validate()?;
        check_proof(&result.final_proof, "final_proof")?;

        let window = &result.descriptor.plan.window;
        limit(result.chain.len(), 10, "chain")?;
        require(!result.chain.is_empty(), "chain")?;
        binding(result.chain.last() == Some(&result.block), "chain")?;
        let anchor = &result.descriptor.anchor.block;
        let mut parent = anchor.block_id;
        let mut height = anchor.height;
        for block in &result.chain {
            let digest = block.digest()?;
            binding(block.parent == parent, "chain")?;
            let next_height = block.metadata.height;
            require(
                next_height > height && (window.h_min..=window.h_max).contains(&next_height),
                "chain",
            )?;
            parent = digest;
            height = next_height;
        }
        check_anchor_evidence(&result.anchor_evidence)?;

        require(!self.validation_basis.is_empty(), "validation_basis")?;
        check_refs(&self.validation_basis, "validation_basis")?;
        check_native_profile(&self.native_profile)?;
        self.origin.validate_shape()?;
        binding(
            self.origin.grant.scope.config == result.descriptor.plan.config_ref,
            "config_ref",
        )
    }
}
