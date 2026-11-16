use crate::{
    DerivationCause, DerivationConfig, DerivationIdentity, DerivedItem, InputPoint, Outcome,
    Reject, ResumeCut, ResumeEvidence, VerifiedInputs, Wait, WaitReason,
    source::{InputStamp, ObservedBlock, digest, hash32, hex_array, position},
};
use l1_adapter::{CursorWire, EventRecord, EventWire, LogWire};
use poc_core::{
    execution::apply_with_byte_limit,
    fixture::{DepProvider, ValidatedDepFixture},
    metadata::l2_timestamp_us,
};
use poc_types::domain::{
    Anchor, BlockRef, ExecutionAnchor, Hash32, InputStatus, L1BlockRef, L1Header, L1Position,
    Metadata, NativeBlock, PhysicalBlock, VirtualBlock, derived_anchor_id,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A single completed input action, or a single fallback output at a completed block end.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Step {
    BlockStarted {
        block: L1BlockRef,
    },
    Event {
        position: L1Position,
    },
    CursorSet {
        position: L1Position,
        resume_position: L1Position,
        fill_count: usize,
    },
    BlockComplete {
        block: L1BlockRef,
    },
    Fallback {
        height: u64,
        origin: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Advance {
    pub new_items: Vec<DerivedItem>,
    pub new_cuts: Vec<ResumeCut>,
    pub status: Outcome<()>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Cursor {
    Start {
        block: u64,
    },
    Open {
        block: InputStamp,
        next_event: usize,
        next_log: Option<L1Position>,
        accepted: Option<Hash32>,
    },
    End {
        block: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CutIndex {
    resume_position: L1Position,
    evidence_digest: Hash32,
}

/// Local state claim. Restoring it always requires verified inputs and deterministic replay.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    identity: DerivationIdentity,
    genesis: ExecutionAnchor,
    genesis_origin: L1Header,
    action_count: u64,
    cursor: Cursor,
    last_log: Option<L1Position>,
    covered: Vec<InputStamp>,
    derived: Vec<DerivedItem>,
    current_epoch: u64,
    epoch_has_output: bool,
    cuts: Vec<CutIndex>,
}

/// Independent D state. Only verified L1 and the configured 03 deposits can advance it.
pub struct Derivation {
    config: DerivationConfig,
    dep: ValidatedDepFixture,
    checkpoint: Checkpoint,
    cuts: BTreeMap<L1Position, ResumeCut>,
    #[cfg(test)]
    pub(crate) pending_deposit: Option<u64>,
}

impl Derivation {
    pub fn init(
        config: DerivationConfig,
        genesis: ExecutionAnchor,
        dep: ValidatedDepFixture,
        inputs: &VerifiedInputs<'_>,
    ) -> Outcome<Self> {
        outcome(Self::initialize(config, genesis, dep, inputs))
    }

    fn initialize(
        config: DerivationConfig,
        genesis: ExecutionAnchor,
        dep: ValidatedDepFixture,
        inputs: &VerifiedInputs<'_>,
    ) -> Result<Self, Halt> {
        let identity = DerivationIdentity::new(&config, inputs.identity().l1.clone())?;
        if &identity != inputs.identity() || dep.config_ref() != identity.config_ref {
            return Err(
                Reject::Configuration("unbound derivation configuration or deposits").into(),
            );
        }
        genesis.validate()?;
        let public = config.public();
        if genesis.anchor != public.genesis
            || genesis.state.root()? != public.genesis_state_root
            || genesis.native_hash != public.genesis_native_hash
        {
            return Err(Reject::Configuration("unbound D genesis").into());
        }
        let current_epoch = genesis.anchor.block.origin;
        let zero = require_header(inputs, 0, InputPoint::BlockStart(0))?;
        validate_time(public, &zero, 0)?;
        if let crate::L1Identity::Driver {
            l1_genesis_hash, ..
        } = &identity.l1
            && zero.hash != *l1_genesis_hash
        {
            return Err(Reject::Source("L1 origin history is unbound").into());
        }
        let origin = require_header(inputs, current_epoch, InputPoint::BlockStart(current_epoch))?;
        validate_time(public, &origin, current_epoch)?;
        if genesis
            .anchor
            .timestamp_us
            .checked_sub(origin.timestamp_us()?)
            .is_none_or(|drift| drift > public.t_drift_us)
        {
            return Err(Reject::Configuration("invalid genesis origin timestamp").into());
        }
        config.window_end(current_epoch)?;
        Ok(Self {
            config,
            dep,
            checkpoint: Checkpoint {
                identity,
                genesis,
                genesis_origin: origin,
                action_count: 0,
                cursor: Cursor::Start { block: 0 },
                last_log: None,
                covered: Vec::new(),
                derived: Vec::new(),
                current_epoch,
                epoch_has_output: false,
                cuts: Vec::new(),
            },
            cuts: BTreeMap::new(),
            #[cfg(test)]
            pending_deposit: None,
        })
    }

    pub fn checkpoint(&self) -> Checkpoint {
        self.checkpoint.clone()
    }

    pub fn restore(
        expected: &Checkpoint,
        config: DerivationConfig,
        genesis: ExecutionAnchor,
        dep: ValidatedDepFixture,
        inputs: &VerifiedInputs<'_>,
    ) -> Outcome<Self> {
        outcome(Self::replay(expected, config, genesis, dep, inputs))
    }

    fn replay(
        expected: &Checkpoint,
        config: DerivationConfig,
        genesis: ExecutionAnchor,
        dep: ValidatedDepFixture,
        inputs: &VerifiedInputs<'_>,
    ) -> Result<Self, Halt> {
        if expected.identity != *inputs.identity() || expected.genesis != genesis {
            return Err(Reject::CheckpointMismatch.into());
        }
        let mut state = Self::initialize(config, genesis, dep, inputs)?;
        if state.checkpoint.genesis_origin != expected.genesis_origin {
            return Err(Reject::CheckpointMismatch.into());
        }
        let mut actions = 0usize;
        for stamp in &expected.covered {
            let block = require_block(
                inputs,
                stamp.reference.height,
                InputPoint::BlockStart(stamp.reference.height),
            )?;
            if block.stamp != *stamp {
                return Err(Reject::CheckpointMismatch.into());
            }
            actions = actions
                .checked_add(block.events.len())
                .and_then(|n| n.checked_add(2))
                .ok_or(Reject::CheckpointMismatch)?;
        }
        if let Cursor::Open {
            block, next_event, ..
        } = &expected.cursor
        {
            let observed = require_block(
                inputs,
                block.reference.height,
                InputPoint::BlockStart(block.reference.height),
            )?;
            if observed.stamp != *block || *next_event > observed.events.len() {
                return Err(Reject::CheckpointMismatch.into());
            }
            actions = actions
                .checked_add(*next_event)
                .and_then(|n| n.checked_add(1))
                .ok_or(Reject::CheckpointMismatch)?;
        }
        actions = actions
            .checked_add(
                expected
                    .derived
                    .iter()
                    .filter(|item| matches!(item.cause, DerivationCause::Fallback { .. }))
                    .count(),
            )
            .ok_or(Reject::CheckpointMismatch)?;
        if u64::try_from(actions).ok() != Some(expected.action_count) {
            return Err(Reject::CheckpointMismatch.into());
        }
        while state.checkpoint.action_count < expected.action_count {
            // A valid checkpoint can contain only this many completed/opened inputs.
            // Stop replay when it exceeds the claimed prefix, even with future blocks visible.
            if state.checkpoint.covered.len() > expected.covered.len()
                || state.checkpoint.derived.len() > expected.derived.len()
                || state.checkpoint.cuts.len() > expected.cuts.len()
            {
                return Err(Reject::CheckpointMismatch.into());
            }
            state.run_step(inputs)?;
        }
        if state.checkpoint != *expected {
            return Err(Reject::CheckpointMismatch.into());
        }
        Ok(state)
    }

    pub fn genesis(&self) -> &ExecutionAnchor {
        &self.checkpoint.genesis
    }
    pub fn derived_prefix(&self) -> &[DerivedItem] {
        &self.checkpoint.derived
    }
    pub fn current_epoch(&self) -> u64 {
        self.checkpoint.current_epoch
    }
    pub fn epoch_has_output(&self) -> bool {
        self.checkpoint.epoch_has_output
    }
    pub fn complete_frontier(&self) -> Option<&L1BlockRef> {
        self.checkpoint.covered.last().map(|block| &block.reference)
    }
    pub fn last_log(&self) -> Option<L1Position> {
        self.checkpoint.last_log
    }
    pub fn head(&self) -> ExecutionAnchor {
        self.derived_prefix()
            .last()
            .map_or_else(|| self.genesis().clone(), anchor)
    }
    pub fn native_hash_at(&self, height: u64) -> Option<Hash32> {
        let first = self.genesis().anchor.block.height;
        match height.checked_sub(first)? {
            0 => Some(self.genesis().native_hash),
            delta => self
                .derived_prefix()
                .get(usize::try_from(delta - 1).ok()?)
                .map(|item| item.native_hash),
        }
    }

    pub fn derive_at_resume(&self, resume: L1Position) -> Outcome<ResumeCut> {
        if let Some(cut) = self.cuts.get(&resume) {
            return Outcome::Ready(cut.clone());
        }
        if self
            .complete_frontier()
            .is_some_and(|block| resume.block <= block.height)
            || self.last_log().is_some_and(|last| resume <= last)
        {
            return Outcome::Reject(Reject::NonResume(resume));
        }
        if let Cursor::Open {
            next_log: Some(log),
            accepted: Some(commitment),
            ..
        } = &self.checkpoint.cursor
        {
            return Outcome::Wait(Wait {
                position: InputPoint::Log(*log),
                reason: WaitReason::Feature11AcceptedPayloadRequired {
                    commitment: *commitment,
                },
            });
        }
        Outcome::Wait(Wait {
            position: self.next_position(),
            reason: WaitReason::ResumeCoverage(resume),
        })
    }

    pub fn next_position(&self) -> InputPoint {
        match &self.checkpoint.cursor {
            Cursor::Start { block } => InputPoint::BlockStart(*block),
            Cursor::Open {
                next_log: Some(log),
                ..
            } => InputPoint::Log(*log),
            Cursor::Open { block, .. } => InputPoint::BlockEnd(block.reference.height),
            Cursor::End { block } => match self.config.window_end(self.current_epoch()) {
                Ok(end) if *block < end => block
                    .checked_add(1)
                    .map_or(InputPoint::BlockEnd(*block), InputPoint::BlockStart),
                Ok(_) => self.head_metadata().height.checked_add(1).map_or(
                    InputPoint::BlockEnd(*block),
                    |height| InputPoint::Fallback {
                        frontier: *block,
                        height,
                    },
                ),
                Err(_) => InputPoint::BlockEnd(*block),
            },
        }
    }

    pub fn step(&mut self, inputs: &VerifiedInputs<'_>) -> Outcome<Step> {
        outcome(
            self.check_source(inputs)
                .and_then(|()| self.run_step(inputs)),
        )
    }

    pub fn advance(&mut self, inputs: &VerifiedInputs<'_>) -> Advance {
        let old_items = self.derived_prefix().len();
        let old_cuts = self.checkpoint.cuts.len();
        let status = (|| {
            self.check_source(inputs)?;
            loop {
                self.run_step(inputs)?;
            }
        })();
        Advance {
            new_items: self.derived_prefix()[old_items..].to_vec(),
            new_cuts: self.checkpoint.cuts[old_cuts..]
                .iter()
                .filter_map(|cut| self.cuts.get(&cut.resume_position).cloned())
                .collect(),
            status: outcome(status),
        }
    }

    fn check_source(&self, inputs: &VerifiedInputs<'_>) -> Result<(), Halt> {
        if inputs.identity() != &self.checkpoint.identity {
            return Err(Reject::Source("different derivation source identity").into());
        }
        // The source is append-only. A different reader must agree on every previously
        // used block before it can resume this state.
        for stamp in &self.checkpoint.covered {
            let height = stamp.reference.height;
            let observed = require_block(inputs, height, InputPoint::BlockStart(height))?;
            if observed.stamp != *stamp {
                return Err(Reject::Source("conflicting prior receipt coverage").into());
            }
        }
        let origin = &self.checkpoint.genesis_origin;
        if require_header(inputs, origin.height, InputPoint::BlockStart(origin.height))? != *origin
        {
            return Err(Reject::Source("conflicting genesis origin").into());
        }
        Ok(())
    }

    fn run_step(&mut self, inputs: &VerifiedInputs<'_>) -> Result<Step, Halt> {
        let count = self
            .checkpoint
            .action_count
            .checked_add(1)
            .ok_or(Reject::Arithmetic("action count"))?;
        let step = match self.checkpoint.cursor.clone() {
            Cursor::Start { block } => self.open(inputs, block)?,
            Cursor::Open {
                block, next_event, ..
            } => {
                let observed = require_block(inputs, block.reference.height, self.next_position())?;
                if observed.stamp != block {
                    return Err(Reject::Source("conflicting open block").into());
                }
                if let Some(event) = observed.events.get(next_event) {
                    let next = cursor_for(
                        &observed,
                        next_event
                            .checked_add(1)
                            .ok_or(Reject::Arithmetic("event index"))?,
                    )?;
                    let step = self.event(inputs, &observed, event)?;
                    self.checkpoint.last_log = Some(position(&event.position));
                    self.checkpoint.cursor = next;
                    step
                } else {
                    self.checkpoint.covered.push(block.clone());
                    self.checkpoint.cursor = Cursor::End {
                        block: block.reference.height,
                    };
                    Step::BlockComplete {
                        block: block.reference,
                    }
                }
            }
            Cursor::End { block } => {
                let boundary = self.config.window_end(self.current_epoch())?;
                if block >= boundary {
                    self.fallback(inputs, block, boundary)?
                } else {
                    self.open(
                        inputs,
                        block
                            .checked_add(1)
                            .ok_or(Reject::Arithmetic("next L1 block"))?,
                    )?
                }
            }
        };
        self.checkpoint.action_count = count;
        Ok(step)
    }

    fn open(&mut self, inputs: &VerifiedInputs<'_>, height: u64) -> Result<Step, Halt> {
        let observed = match inputs.block(height) {
            InputStatus::Ready(block) => block,
            InputStatus::Reject(error) => return Err(Reject::Input(error).into()),
            InputStatus::Wait(_) => {
                let reason = match inputs.header(height) {
                    InputStatus::Ready(_) => WaitReason::CompleteBlock(height),
                    InputStatus::Wait(_) => WaitReason::CompleteWindow {
                        through: self.config.window_end(self.current_epoch())?,
                    },
                    InputStatus::Reject(error) => return Err(Reject::Input(error).into()),
                };
                return Err(wait(InputPoint::BlockStart(height), reason));
            }
        };
        validate_time(self.config.public(), &observed.header, height)?;
        if let Some(previous) = self.complete_frontier()
            && (previous.height.checked_add(1) != Some(height)
                || previous.hash != observed.header.parent_hash)
        {
            return Err(Reject::Source("nonextending completed L1 history").into());
        }
        self.checkpoint.cursor = cursor_for(&observed, 0)?;
        Ok(Step::BlockStarted {
            block: observed.stamp.reference.clone(),
        })
    }

    fn fallback(
        &mut self,
        inputs: &VerifiedInputs<'_>,
        block: u64,
        window_end: u64,
    ) -> Result<Step, Halt> {
        let prior = self.head_metadata();
        let (height, time) = self.next_height_time(prior)?;
        let point = InputPoint::Fallback {
            frontier: block,
            height,
        };
        let next_origin = prior
            .origin
            .checked_add(1)
            .ok_or(Reject::Arithmetic("fallback next origin"))?;
        let next = require_header(inputs, next_origin, point)?;
        validate_time(self.config.public(), &next, next_origin)?;
        let origin = if !self.epoch_has_output() || time < next.timestamp_us()? {
            prior.origin
        } else {
            next_origin
        };
        let item = self.empty_item(
            inputs,
            self.derived_prefix().last(),
            Metadata {
                height,
                timestamp_us: time,
                origin,
            },
            DerivationCause::Fallback {
                window_end,
                frontier: self
                    .complete_frontier()
                    .ok_or(Reject::Source("fallback without complete frontier"))?
                    .clone(),
            },
            point,
        )?;
        self.commit_items(std::iter::once(item));
        Ok(Step::Fallback { height, origin })
    }

    fn event(
        &mut self,
        inputs: &VerifiedInputs<'_>,
        block: &ObservedBlock,
        event: &EventRecord,
    ) -> Result<Step, Halt> {
        let pos = position(&event.position);
        match &event.event {
            EventWire::CursorAdvanced { batch, .. } => Err(wait(
                InputPoint::Log(pos),
                WaitReason::Feature11AcceptedPayloadRequired {
                    commitment: hash32(&batch.commitment)
                        .ok_or(Reject::Source("invalid accepted commitment"))?,
                },
            )),
            EventWire::CursorSet { next, resume, .. } => {
                self.set(inputs, block, &event.position, next, resume)
            }
            EventWire::Resume { set, .. } => {
                if self
                    .cuts
                    .get(&pos)
                    .is_none_or(|cut| cut.cursor_set_position != position(set))
                {
                    return Err(Reject::Source("RESUME lacks completed matching CursorSet").into());
                }
                Ok(Step::Event { position: pos })
            }
            EventWire::CursorInitialized { .. } | EventWire::Force { .. } => {
                Ok(Step::Event { position: pos })
            }
        }
    }

    fn set(
        &mut self,
        inputs: &VerifiedInputs<'_>,
        block: &ObservedBlock,
        marker: &LogWire,
        target: &CursorWire,
        resume: &LogWire,
    ) -> Result<Step, Halt> {
        let set_position = position(marker);
        let resume_position = position(resume);
        let point = InputPoint::Log(set_position);
        if set_position.block != resume_position.block
            || set_position.tx != resume_position.tx
            || set_position.log >= resume_position.log
            || marker.tx_hash != resume.tx_hash
            || marker.rpc_log_index >= resume.rpc_log_index
        {
            return Err(Reject::Source("CursorSet and RESUME position mismatch").into());
        }
        let paired = block
            .events
            .iter()
            .find(|event| position(&event.position) == resume_position)
            .ok_or_else(|| wait(point, WaitReason::ResumePair(resume_position)))?;
        let EventWire::Resume {
            request_id,
            seq,
            height: target_height,
            origin,
            set,
            ..
        } = &paired.event
        else {
            return Err(Reject::Source("CursorSet points to non-RESUME event").into());
        };
        let seq_bytes = hex_array::<32>(&target.source)
            .ok_or(Reject::Source("invalid RESUME cursor sequence"))?;
        if &paired.position != resume
            || set != marker
            || *target_height != target.height
            || *origin != target.origin
            || target.source_kind != 2
            || seq_bytes[..24] != [0; 24]
            || seq_bytes[24..] != seq.to_be_bytes()
            || self.cuts.contains_key(&resume_position)
        {
            return Err(Reject::Source("contradictory CursorSet/RESUME pair").into());
        }
        let evidence = ResumeEvidence {
            derivation: self.checkpoint.identity.clone(),
            including_block: block.stamp.reference.clone(),
            transaction_hash: hash32(&marker.tx_hash)
                .ok_or(Reject::Source("invalid RESUME transaction hash"))?,
            request_id: hash32(request_id).ok_or(Reject::Source("invalid RESUME request id"))?,
            sequence: *seq,
            cursor_set_rpc_log_index: marker.rpc_log_index,
            resume_rpc_log_index: resume.rpc_log_index,
        };
        let mut staged: Vec<DerivedItem> = Vec::new();
        let mut metadata = self.head_metadata().clone();
        while metadata.height < *target_height {
            let (height, time) = self.next_height_time(&metadata)?;
            let mut origin = metadata.origin;
            if let Some(next_origin) = origin.checked_add(1).filter(|n| *n <= set_position.block) {
                let next = require_header(inputs, next_origin, point)?;
                validate_time(self.config.public(), &next, next_origin)?;
                if next.timestamp_us()? <= time {
                    origin = next_origin;
                }
            }
            metadata = Metadata {
                height,
                timestamp_us: time,
                origin,
            };
            let parent = staged.last().or_else(|| self.derived_prefix().last());
            staged.push(self.empty_item(
                inputs,
                parent,
                metadata.clone(),
                DerivationCause::CursorSet {
                    position: set_position,
                },
                point,
            )?);
        }
        let fill_count = staged.len();
        let genesis = self.genesis().clone();
        let derived_prefix: Vec<_> = self
            .derived_prefix()
            .iter()
            .chain(&staged)
            .cloned()
            .collect();
        let delta = target_height
            .checked_sub(genesis.anchor.block.height)
            .ok_or(Reject::Source("RESUME target before genesis"))?;
        let target_anchor = if delta == 0 {
            genesis.clone()
        } else {
            let index =
                usize::try_from(delta - 1).map_err(|_| Reject::Arithmetic("target index"))?;
            anchor(
                derived_prefix
                    .get(index)
                    .ok_or(Reject::Source("RESUME target missing from D"))?,
            )
        };
        let cut = ResumeCut {
            resume_position,
            cursor_set_position: set_position,
            target_height: *target_height,
            genesis,
            derived_prefix,
            target_anchor,
            evidence,
        };
        let encoded = serde_json::to_vec(&cut)
            .map_err(|_| Reject::Source("failed to encode checkpoint cut evidence"))?;
        let checked_cut = CutIndex {
            resume_position,
            evidence_digest: digest("PoC/Derivation/ResumeCutCheckpoint/v1", &encoded),
        };
        self.commit_items(staged);
        self.checkpoint.cuts.push(checked_cut);
        self.cuts.insert(resume_position, cut);
        Ok(Step::CursorSet {
            position: set_position,
            resume_position,
            fill_count,
        })
    }

    fn empty_item(
        &self,
        inputs: &VerifiedInputs<'_>,
        parent: Option<&DerivedItem>,
        metadata: Metadata,
        cause: DerivationCause,
        at: InputPoint,
    ) -> Result<DerivedItem, Halt> {
        let header = require_header(inputs, metadata.origin, at)?;
        validate_time(self.config.public(), &header, metadata.origin)?;
        if metadata.timestamp_us < header.timestamp_us()? {
            return Err(Reject::Source("L2 timestamp precedes selected origin").into());
        }
        #[cfg(test)]
        if self.pending_deposit == Some(metadata.height) {
            return Err(wait(
                at,
                WaitReason::Deposit(poc_types::domain::InputMissing::Dep(metadata.height)),
            ));
        }
        let dep = match self.dep.at(&metadata) {
            InputStatus::Ready(dep) => dep,
            InputStatus::Wait(missing) => return Err(wait(at, WaitReason::Deposit(missing))),
            InputStatus::Reject(error) => return Err(Reject::Input(error).into()),
        };
        let (parent_state, native_parent, physical_parent) = match parent {
            Some(item) => (&item.state, item.native_hash, item.physical_block.digest()?),
            None => (
                &self.genesis().state,
                self.genesis().native_hash,
                self.genesis().anchor.block.block_id,
            ),
        };
        let pre_state_root = parent_state.root()?;
        let applied = apply_with_byte_limit(
            parent_state,
            &metadata,
            &dep,
            &[],
            self.config.public().byte_limit as usize,
        )?;
        let native_block = NativeBlock {
            parent: native_parent,
            metadata: metadata.clone(),
            dep: dep.clone(),
            kept: applied.kept.clone(),
            post_state_root: applied.state.root()?,
        };
        let native_hash = native_block.digest()?;
        Ok(DerivedItem {
            virtual_block: VirtualBlock {
                metadata: metadata.clone(),
                dep,
                pre_state_root,
                transactions: applied.kept.clone(),
            },
            physical_block: PhysicalBlock {
                parent: physical_parent,
                metadata,
                pre_state_root,
                transactions: applied.kept,
            },
            native_block,
            native_hash,
            state: applied.state,
            cause,
        })
    }

    fn commit_items(&mut self, items: impl IntoIterator<Item = DerivedItem>) {
        for item in items {
            self.checkpoint.current_epoch = item.state.metadata.origin;
            self.checkpoint.epoch_has_output = true;
            self.checkpoint.derived.push(item);
        }
    }

    fn head_metadata(&self) -> &Metadata {
        self.derived_prefix()
            .last()
            .map_or(&self.genesis().state.metadata, |item| &item.state.metadata)
    }

    fn next_height_time(&self, parent: &Metadata) -> Result<(u64, u64), Halt> {
        let height = parent
            .height
            .checked_add(1)
            .ok_or(Reject::Arithmetic("next L2 height"))?;
        let time = l2_timestamp_us(self.config.public(), height)?;
        Ok((height, time))
    }
}

fn anchor(item: &DerivedItem) -> ExecutionAnchor {
    let metadata = &item.state.metadata;
    ExecutionAnchor {
        anchor: Anchor {
            block: BlockRef {
                height: metadata.height,
                origin: metadata.origin,
                block_id: derived_anchor_id(metadata.height, item.native_hash),
            },
            timestamp_us: metadata.timestamp_us,
        },
        state: item.state.clone(),
        native_hash: item.native_hash,
    }
}

fn cursor_for(block: &ObservedBlock, next_event: usize) -> Result<Cursor, Reject> {
    let next = block.events.get(next_event);
    let accepted = match next.map(|event| &event.event) {
        Some(EventWire::CursorAdvanced { batch, .. }) => {
            Some(hash32(&batch.commitment).ok_or(Reject::Source("invalid accepted commitment"))?)
        }
        _ => None,
    };
    Ok(Cursor::Open {
        block: block.stamp.clone(),
        next_event,
        next_log: next.map(|event| position(&event.position)),
        accepted,
    })
}

fn require_header(
    inputs: &VerifiedInputs<'_>,
    height: u64,
    point: InputPoint,
) -> Result<L1Header, Halt> {
    match inputs.header(height) {
        InputStatus::Ready(header) => Ok(header),
        InputStatus::Wait(_) => Err(wait(point, WaitReason::L1Header(height))),
        InputStatus::Reject(error) => Err(Reject::Input(error).into()),
    }
}

fn require_block<'source>(
    inputs: &'source VerifiedInputs<'_>,
    height: u64,
    point: InputPoint,
) -> Result<std::borrow::Cow<'source, ObservedBlock>, Halt> {
    match inputs.block(height) {
        InputStatus::Ready(block) => Ok(block),
        InputStatus::Wait(_) => Err(wait(point, WaitReason::CompleteBlock(height))),
        InputStatus::Reject(error) => Err(Reject::Input(error).into()),
    }
}

fn validate_time(
    config: &poc_types::domain::PublicConfig,
    header: &L1Header,
    height: u64,
) -> Result<(), Halt> {
    let timestamp = config
        .l1_block_us
        .checked_mul(height)
        .and_then(|value| value.checked_add(config.l1_genesis_timestamp_us))
        .ok_or(Reject::Arithmetic("L1 slot timestamp"))?;
    if header.height != height || header.timestamp_us()? != timestamp {
        return Err(Reject::Source("L1 header violates bound slot schedule").into());
    }
    Ok(())
}

#[derive(Debug)]
enum Halt {
    Wait(Wait),
    Reject(Reject),
}
impl From<Reject> for Halt {
    fn from(error: Reject) -> Self {
        Self::Reject(error)
    }
}
impl From<poc_types::domain::DomainError> for Halt {
    fn from(error: poc_types::domain::DomainError) -> Self {
        Self::Reject(error.into())
    }
}
impl From<poc_core::execution::ExecutionError> for Halt {
    fn from(error: poc_core::execution::ExecutionError) -> Self {
        Self::Reject(error.into())
    }
}
fn wait(position: InputPoint, reason: WaitReason) -> Halt {
    Halt::Wait(Wait { position, reason })
}
fn outcome<T>(result: Result<T, Halt>) -> Outcome<T> {
    match result {
        Ok(value) => Outcome::Ready(value),
        Err(Halt::Wait(wait)) => Outcome::Wait(wait),
        Err(Halt::Reject(error)) => Outcome::Reject(error),
    }
}
