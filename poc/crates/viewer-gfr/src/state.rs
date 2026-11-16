use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use poc_types::domain::{L1Header, PublicReferenceView};
use sim_espresso::FinalizedBlock;

use crate::{
    Anchor, ChainEntry, ChainView, ControlRef, Descriptor, ExecutionAnchor, GfrParameters, Hash32,
    InputError, InputStatus, LifecycleEvent, PublicConfig, ReplayPosition, ReplayRecord,
    ReplayStatus, ResponsibilityView, Tail, VerificationProfile, publications::PublicationIndex,
};

/// Deterministic at equal pc. Downloaded future inputs and receipt times are absent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicReplayState {
    pub config: PublicConfig,
    pub genesis: ExecutionAnchor,
    pub parameters: GfrParameters,
    pub verification: VerificationProfile,
    pub espresso_source: Vec<u8>,
    pub l1_source: Vec<u8>,
    pub branch: u64,
    pub tail: Tail,
    pub pc: Option<ReplayPosition>,
    pub selected: BTreeMap<(u64, u64), Descriptor>,
    pub closed_sessions: BTreeSet<(u64, u64)>,
    pub closed_branches: BTreeSet<u64>,
    pub entries: Vec<ChainEntry>,
    pub records: Vec<ReplayRecord>,
    pub publications: PublicationIndex,
}

/// The exact Espresso watermark and L1 material that supported committed events.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SupportingCuts {
    pub espresso: Vec<Arc<FinalizedBlock>>,
    pub l1_headers: Vec<L1Header>,
    pub l1_controls: Vec<Vec<ControlRef>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalCheckpoint {
    pub state: PublicReplayState,
    pub cuts: SupportingCuts,
    pub(crate) genesis_binding: GenesisBinding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GenesisBinding {
    pub(crate) config: PublicConfig,
    pub(crate) execution: ExecutionAnchor,
    pub(crate) parameters: GfrParameters,
    pub(crate) verification: VerificationProfile,
    pub(crate) espresso_source: Vec<u8>,
    pub(crate) l1_source: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InitInput {
    Genesis {
        config: PublicConfig,
        genesis: ExecutionAnchor,
        parameters: GfrParameters,
        expected_verification: VerificationProfile,
    },
    Checkpoint(GlobalCheckpoint),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayState {
    pub(crate) public: PublicReplayState,
    pub(crate) cuts: SupportingCuts,
    pub(crate) anchors: BTreeMap<Hash32, ExecutionAnchor>,
    pub(crate) next_q: u64,
}

impl ReplayState {
    pub fn public_state(&self) -> &PublicReplayState {
        &self.public
    }

    pub fn checkpoint(&self) -> GlobalCheckpoint {
        GlobalCheckpoint {
            state: self.public.clone(),
            cuts: self.cuts.clone(),
            genesis_binding: GenesisBinding {
                config: self.public.config.clone(),
                execution: self.public.genesis.clone(),
                parameters: self.public.parameters,
                verification: self.public.verification.clone(),
                espresso_source: self.public.espresso_source.clone(),
                l1_source: self.public.l1_source.clone(),
            },
        }
    }

    pub(crate) fn head_height(&self) -> u64 {
        self.public
            .entries
            .last()
            .map_or(self.public.genesis.anchor.block.height, |e| {
                e.block.virtual_block.metadata.height
            })
    }

    pub(crate) fn entry(&self, height: u64) -> Option<&ChainEntry> {
        let offset = height
            .checked_sub(self.public.genesis.anchor.block.height)?
            .checked_sub(1)?;
        self.public.entries.get(usize::try_from(offset).ok()?)
    }

    pub(crate) fn last_anchor(&self) -> &Anchor {
        let Some(entry) = self.public.entries.last() else {
            return &self.public.genesis.anchor;
        };
        // G only appends intervals ending in a physical block.
        &self.anchors[&entry.block.physical_digest.expect("physical G head")].anchor
    }
}

impl PublicReferenceView for ReplayState {
    fn reference(&self, reference: &poc_types::domain::BlockRef) -> InputStatus<Anchor> {
        match self.anchors.get(&reference.block_id) {
            Some(material) if material.anchor.block == *reference => {
                InputStatus::Ready(material.anchor.clone())
            }
            _ => InputStatus::Reject(InputError::InvalidSource),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitResult {
    pub state: Option<ReplayState>,
    pub status: ReplayStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayResult {
    pub state: ReplayState,
    pub added_records: Vec<ReplayRecord>,
    pub events: Vec<LifecycleEvent>,
    pub status: ReplayStatus,
}

pub fn chain(state: &ReplayState) -> ChainView<'_> {
    ChainView {
        genesis: &state.public.genesis,
        entries: &state.public.entries,
        pc: state.public.pc.as_ref(),
    }
}

pub fn responsibility(state: &ReplayState) -> ResponsibilityView<'_> {
    ResponsibilityView {
        tail: &state.public.tail,
        branch: state.public.branch,
        pc: state.public.pc.as_ref(),
    }
}
