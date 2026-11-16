use std::{cmp::Ordering, sync::Arc};

use poc_types::domain::{
    Boundary, BoundaryCause, Cid, CloseReason, Descriptor, Hash32, LifecycleBody, ReplayIndex, ReplayKind,
    ReplayPosition,
};
use poc_viewer_gfr::{
    GlobalSource, InputRef, ReplayEffect, ReplayRecord, ResumeBasis, Tail, compare_positions, encode_position,
    lifecycle_id, lifecycle_projection,
};

use crate::{
    AcceptedRecord, BRANCHES, CapacityKind, EFFECTS, EVENTS, LIFECYCLE_BYTES, ManagerError,
    Planner, RECORDS, SESSIONS, Selection, SessionKey, TerminalCause, Update, WaitReason, VerifiedReplayInput,
    add, bound, room, session_key, validate_descriptor,
};

impl Planner<'_> {
    pub(crate) fn consume_record(
        &mut self,
        input: &VerifiedReplayInput,
        material: Option<&ResumeBasis>,
    ) -> Result<Update, ManagerError> {
        let record = &input.record;
        if let (ReplayKind::Resume,ReplayIndex::L1(position)) = (record.position.kind,&record.position.index) {
            let Some(basis)=material else {
                return Ok(Update::Wait(WaitReason::ResumeMaterial{position:*position}));
            };
            validate_resume(record,basis,self.config)?;
        } else if material.is_some() {
            return Err(ManagerError::InvalidRecord);
        }
        let ids = self.validate_record(record)?;
        room(self.state.records.len(), CapacityKind::Records, RECORDS)?;

        let mut assignments = Vec::new();
        let mut closes = Vec::new();
        for event in &record.events {
            match &event.body {
                LifecycleBody::SessionAssigned { descriptor } => {
                    self.assign(
                        descriptor,
                        &record.position,
                        input.validation_ref,
                        &mut closes,
                    )?;
                    assignments.push(descriptor);
                }
                LifecycleBody::SessionClosed { session, reason } => {
                    let key = session_key(session);
                    self.reserve_session(key)?;
                    self.state
                        .sessions
                        .entry(key)
                        .or_default()
                        .closed
                        .get_or_insert(*reason);
                    self.state
                        .guards
                        .closed_sessions
                        .entry(key)
                        .or_insert(*reason);
                    self.close_scope(|other| other == key, *reason, &mut closes);
                }
                LifecycleBody::BranchClosed { branch, reason } => {
                    if !self.state.guards.closed_branches.contains_key(branch) {
                        room(
                            self.state.guards.closed_branches.len(),
                            CapacityKind::Branches,
                            BRANCHES,
                        )?;
                        self.state.guards.closed_branches.insert(*branch, *reason);
                    }
                    self.close_scope(|other| other.0 == *branch, *reason, &mut closes);
                }
                LifecycleBody::Resumed { .. } => {
                    let basis=material.expect("validated RESUME material");
                    self.state.resumes.insert(basis.position,Arc::new(basis.clone()));
                }
            }
        }
        // The lists have independent order. Process effects using the selected descriptor
        // history, which now includes assignments from a multi-handoff record.
        for effect in &record.effects {
            if let ReplayEffect::Append {
                first,
                last,
                source: GlobalSource::Regional(key),
            } = effect
            {
                self.append_tip(key.cid, *first, *last, &mut closes)?;
            }
        }
        for descriptor in assignments {
            let cid = descriptor.cid;
            let session = &descriptor.plan.session;
            let predecessor = if matches!(descriptor.plan.cause,BoundaryCause::Force|BoundaryCause::Resume) {
                None
            } else {
                Some(self.selected_predecessor(descriptor)?.cid)
            };
            if predecessor
                .and_then(|cid| self.state.guards.physical_tip(cid))
                .is_some_and(|tip| descriptor.anchor.block.height < tip)
            {
                return Err(ManagerError::InvalidRecord);
            }
            if !self.is_member(descriptor)
                || self.state.guards.closed_session(session).is_some()
                || self.state.guards.closed_branch(session.beta).is_some()
            {
                continue;
            }
            if let Some(program) = self.state.programs.get(&cid) {
                self.obligations.extend(program.create_action);
            } else {
                self.register_program(
                    descriptor.clone(),
                    predecessor,
                    input.validation_ref,
                    Selection::Selected,
                )?;
                if self.state.local_time_us >= self.state.programs[&cid].stop_us {
                    self.mark_terminal(cid, TerminalCause::LocalTime);
                    closes.push(cid);
                } else {
                    self.issue_create(cid)?;
                }
            }
        }
        for cid in closes {
            self.issue_close(cid)?;
        }
        self.state.next_source_seq = add(input.seq, 1)?;
        self.state.records.push(Arc::new(AcceptedRecord {
            seq: input.seq,
            source: input.source,
            record: record.clone(),
            validation_ref: input.validation_ref,
            lifecycle_ids: ids,
            revision: self.state.revision,
        }));
        Ok(Update::Changed)
    }

    fn validate_record(&self, record: &ReplayRecord) -> Result<Vec<Hash32>, ManagerError> {
        let position = &record.position;
        encode_position(position).map_err(|_| ManagerError::InvalidRecord)?;
        let matching_source = match (&position.index, &record.input_ref) {
            (ReplayIndex::Espresso(q), InputRef::Espresso(other)) => q == other,
            (ReplayIndex::Height(height), InputRef::Expire(other)) => height == other,
            (ReplayIndex::Session(session), InputRef::SessionEnd(other)) => session == other,
            (ReplayIndex::L1(at), InputRef::L1(control)) => {
                position.kind == control.kind && *at == control.position && (control.force_branch().is_some() || control.resume_control().is_some())
            }
            _ => false,
        };
        if !matching_source {
            return Err(ManagerError::InvalidRecord);
        }
        if let Some(pc) = self.state.cursor()
            && compare_positions(pc, position).map_err(|_| ManagerError::InvalidRecord)?
                != Ordering::Less
        {
            return Err(ManagerError::InvalidRecord);
        }
        bound(record.events.len(), CapacityKind::Events, EVENTS)?;
        bound(record.effects.len(), CapacityKind::Effects, EFFECTS)?;
        if let InputRef::L1(control) = &record.input_ref
            && position.kind == ReplayKind::Force {
            validate_force(record, control.force_branch().ok_or(ManagerError::InvalidRecord)?)?;
        }
        for effect in &record.effects {
            match effect {
                ReplayEffect::Reset { .. } if position.kind == ReplayKind::Resume => {}
                ReplayEffect::Reset { .. } => return Err(ManagerError::InvalidRecord),
                ReplayEffect::Duty { before, after } => {
                    self.validate_tail(before)?;
                    self.validate_tail(after)?;
                }
                ReplayEffect::Append { .. } if position.kind != ReplayKind::Espresso => {
                    return Err(ManagerError::InvalidRecord);
                }
                ReplayEffect::Append { .. } => {}
            }
        }
        let mut ids = Vec::with_capacity(record.events.len());
        let mut lifecycle_bytes = 0usize;
        for event in &record.events {
            if event.position != *position {
                return Err(ManagerError::InvalidRecord);
            }
            match &event.body {
                LifecycleBody::SessionAssigned { descriptor } => {
                    validate_descriptor(self.binding, self.config, descriptor)?;
                    if position.kind == ReplayKind::Expire
                        || (descriptor.plan.cause == BoundaryCause::Force)
                            != (position.kind == ReplayKind::Force)
                        || (descriptor.plan.cause == BoundaryCause::Resume) != (position.kind == ReplayKind::Resume)
                    {
                        return Err(ManagerError::InvalidRecord);
                    }
                }
                LifecycleBody::SessionClosed { reason, .. } => {
                    if !matches!(reason, CloseReason::Complete | CloseReason::Expired) {
                        return Err(ManagerError::Unsupported);
                    }
                }
                LifecycleBody::BranchClosed { reason, .. } => {
                    if !matches!((position.kind, *reason),
                        (ReplayKind::Expire, CloseReason::Expired) | (ReplayKind::Force, CloseReason::Force) | (ReplayKind::Resume, CloseReason::Resume))
                    {
                        return Err(ManagerError::Unsupported);
                    }
                }
                LifecycleBody::Resumed { .. } if position.kind == ReplayKind::Resume => {}
                LifecycleBody::Resumed { .. } => return Err(ManagerError::InvalidRecord),
            }
            let encoded = lifecycle_projection(event).map_err(|_| ManagerError::InvalidRecord)?;
            lifecycle_bytes = lifecycle_bytes
                .checked_add(encoded.len())
                .ok_or(ManagerError::Overflow)?;
            bound(
                lifecycle_bytes,
                CapacityKind::LifecycleBytes,
                LIFECYCLE_BYTES,
            )?;
            ids.push(lifecycle_id(event).map_err(|_| ManagerError::InvalidRecord)?);
        }
        Ok(ids)
    }

    fn validate_tail(&self, tail: &Tail) -> Result<(), ManagerError> {
        match tail {
            Tail::Next(descriptor) => validate_descriptor(self.binding, self.config, descriptor),
            Tail::Pending(boundary) | Tail::Halt { boundary, .. } => validate_boundary(boundary),
        }
    }

    fn reserve_session(&self, key: SessionKey) -> Result<(), ManagerError> {
        if !self.state.sessions.contains_key(&key) {
            room(self.state.sessions.len(), CapacityKind::Sessions, SESSIONS)?;
        }
        Ok(())
    }

    pub(crate) fn select_public(
        &mut self,
        descriptor: Descriptor,
        position: Option<ReplayPosition>,
        source_ref: Hash32,
    ) -> Result<(), ManagerError> {
        let key = session_key(&descriptor.plan.session);
        self.reserve_session(key)?;
        let session = self.state.sessions.entry(key).or_default();
        if let Some(selected) = &session.selected {
            return Err(if selected.descriptor == descriptor {
                ManagerError::InvalidRecord
            } else {
                ManagerError::DescriptorConflict
            });
        }
        session.selected = Some(crate::PublicSelection {
            descriptor,
            source_ref,
            position,
        });
        Ok(())
    }

    fn assign(
        &mut self,
        descriptor: &Descriptor,
        position: &ReplayPosition,
        source: Hash32,
        closes: &mut Vec<Cid>,
    ) -> Result<(), ManagerError> {
        let session = &descriptor.plan.session;
        if descriptor.plan.cause == BoundaryCause::Handoff {
            self.selected_predecessor(descriptor)?;
        } else if !matches!(descriptor.plan.cause,BoundaryCause::Force|BoundaryCause::Resume) {
            return Err(ManagerError::InvalidRecord);
        }
        if self.state.guards.closed_session(session).is_some()
            || self.state.guards.closed_branch(session.beta).is_some()
        {
            return Err(ManagerError::InvalidRecord);
        }
        if let Some(program) = self.state.programs.get(&descriptor.cid)
            && program.descriptor != *descriptor
        {
            return Err(ManagerError::DescriptorConflict);
        }
        self.select_public(descriptor.clone(), Some(position.clone()), source)?;
        let affected = self
            .state
            .programs
            .iter()
            .filter_map(|(&cid, program)| {
                (program.descriptor.plan.session == *session).then_some(cid)
            })
            .collect::<Vec<_>>();
        for cid in affected {
            if cid == descriptor.cid {
                self.program_mut(cid).selection = Selection::Selected;
            } else {
                self.exclude(cid, closes);
            }
        }
        Ok(())
    }

    fn close_scope(
        &mut self,
        affected: impl Fn(SessionKey) -> bool,
        reason: CloseReason,
        closes: &mut Vec<Cid>,
    ) {
        let cids = self
            .state
            .programs
            .iter()
            .filter_map(|(&cid, program)| {
                affected(session_key(&program.descriptor.plan.session)).then_some(cid)
            })
            .collect::<Vec<_>>();
        for cid in cids {
            if self.mark_terminal(cid, TerminalCause::Public(reason)) {
                closes.push(cid);
            }
        }
    }

    fn append_tip(
        &mut self,
        predecessor: Cid,
        first: u64,
        last: u64,
        closes: &mut Vec<Cid>,
    ) -> Result<(), ManagerError> {
        let descriptor = self
            .state
            .sessions
            .values()
            .filter_map(|session| session.selected.as_ref())
            .find(|selected| selected.descriptor.cid == predecessor)
            .map(|selected| &selected.descriptor)
            .ok_or(ManagerError::InvalidRecord)?;
        if first > last
            || first <= descriptor.anchor.block.height
            || last < descriptor.plan.window.h_min
            || last > descriptor.plan.window.h_max
            || self
                .state
                .guards
                .physical_tip(predecessor)
                .is_some_and(|tip| last <= tip)
        {
            return Err(ManagerError::InvalidRecord);
        }
        self.state.guards.physical_tips.insert(predecessor, last);
        let excluded = self
            .state
            .programs
            .iter()
            .filter_map(|(&cid, program)| {
                (program.predecessor == Some(predecessor)
                    && program.descriptor.anchor.block.height < last)
                    .then_some(cid)
            })
            .collect::<Vec<_>>();
        for cid in excluded {
            if self.state.programs[&cid].selection == Selection::Selected {
                return Err(ManagerError::InvalidRecord);
            }
            self.exclude(cid, closes);
        }
        Ok(())
    }

    fn exclude(&mut self, cid: Cid, closes: &mut Vec<Cid>) {
        self.program_mut(cid).selection = Selection::Excluded;
        if self.mark_terminal(cid, TerminalCause::Public(CloseReason::Excluded)) {
            closes.push(cid);
        }
    }
}

fn validate_boundary(boundary: &Boundary) -> Result<(), ManagerError> {
    boundary.validate().map_err(|_| ManagerError::InvalidRecord)
}

fn validate_resume(record: &ReplayRecord,basis: &ResumeBasis,config: &poc_types::domain::PublicConfig) -> Result<(),ManagerError> {
    let bad=ManagerError::InvalidRecord;
    let InputRef::L1(control)=&record.input_ref else{return Err(bad)};
    let command=control.resume_control().ok_or(bad.clone())?;
    let [ReplayEffect::Reset {resume_position,target_height,cut_digest}, ReplayEffect::Duty{before,after}]=record.effects.as_slice() else{return Err(bad)};
    let [close,resumed,rest @ ..]=record.events.as_slice() else{return Err(bad)};
    let branch=match before {Tail::Next(d)=>d.plan.session.beta,Tail::Pending(b)|Tail::Halt{boundary:b,..}=>b.input.session.beta};
    let beta=add(branch,1)?;
    let (expected,anchor)=match (after,rest) {
        (Tail::Next(d),[a]) if a.body==(LifecycleBody::SessionAssigned{descriptor:d.clone()})=>
            (d.plan.input(),&d.anchor),
        (Tail::Pending(b),[]) => (b.input.clone(),b.anchor.as_ref().ok_or(bad.clone())?),
        (Tail::Halt{boundary,anchor},[end]) if end.body==(LifecycleBody::SessionClosed{
            session:boundary.input.session.clone(),reason:CloseReason::Expired}) => (boundary.input.clone(),anchor),
        _=>return Err(bad),
    };
    if close.body!=(LifecycleBody::BranchClosed{branch,reason:CloseReason::Resume})
        || resumed.body!=(LifecycleBody::Resumed{control_seq:control.control_seq,branch:beta,
            anchor:basis.target.anchor.clone(),derived_prefix:basis.blocks.iter().map(|b|b.virtual_block.clone()).collect()})
        || expected.cause!=BoundaryCause::Resume || expected.session.beta!=beta || expected.session.r!=0
        || expected.source_region.is_some() || expected.reference_block!=anchor.block
        || expected.window.h_min!=add(command.target_height,1)? || expected.window.h_max!=add(command.target_height,10)?
        || anchor!=&basis.target.anchor || anchor.block.height!=command.target_height
        || target_height!=&command.target_height || resume_position!=&control.position
        || basis.position!=control.position || basis.set_position != (poc_types::domain::L1Position{log:command.set_log,..control.position})
        || basis.transaction_hash!=command.transaction_hash || &basis.digest!=cut_digest
        || config.reference().map_err(|_|bad.clone())?!=basis.config_ref
        || basis.target.validate().is_err()
        || basis.blocks.last().is_none_or(|row| row.state!=basis.target.state || row.native_hash!=basis.target.native_hash)
        || anchor.block.block_id!=poc_types::domain::derived_anchor_id(command.target_height,basis.target.native_hash)
    {
        return Err(bad);
    }
    Ok(())
}

fn validate_force(record: &ReplayRecord, target: u64) -> Result<(), ManagerError> {
    if record.effects.is_empty() && record.events.is_empty() {
        return Ok(());
    }
    let [ReplayEffect::Duty { before, after }] = record.effects.as_slice() else {
        return Err(ManagerError::InvalidRecord);
    };
    let prior = match before {
        Tail::Next(descriptor) => descriptor.plan.input(),
        Tail::Pending(boundary) => boundary.input.clone(),
        Tail::Halt { .. } => return Err(ManagerError::InvalidRecord),
    };
    if prior.session.beta != target {
        return Err(ManagerError::InvalidRecord);
    }
    let [closed, remaining @ ..] = record.events.as_slice() else {
        return Err(ManagerError::InvalidRecord);
    };
    if closed.body != (LifecycleBody::BranchClosed { branch: target, reason: CloseReason::Force }) {
        return Err(ManagerError::InvalidRecord);
    }
    let (input, anchor) = match (after, remaining) {
        (Tail::Next(descriptor), [assigned]) if assigned.body == (LifecycleBody::SessionAssigned {
            descriptor: descriptor.clone(),
        }) => (descriptor.plan.input(), &descriptor.anchor),
        (Tail::Pending(boundary), []) => (
            boundary.input.clone(), boundary.anchor.as_ref().ok_or(ManagerError::InvalidRecord)?,
        ),
        _ => return Err(ManagerError::InvalidRecord),
    };
    if input.cause != BoundaryCause::Force || input.session.beta != add(target, 1)?
        || input.reference_block != anchor.block
    {
        return Err(ManagerError::InvalidRecord);
    }
    Ok(())
}
