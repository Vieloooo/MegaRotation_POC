//! Explicit bounded fixture source. Real feature 07 replay will implement the same capability.

use crate::{
    ServiceConfig,
    accepted_view::{CurrentCursor, ReadError, ResumeRequirement},
    crypto::CursorWire,
    frame::{BlockRecord, commitment},
    storage::{json_bytes, reject, wait},
};
use poc_core::{
    execution::apply_with_byte_limit,
    fixture::{DepProvider, ValidatedDepFixture},
    metadata::l2_timestamp_us,
};
use poc_types::domain::{
    DepFixture, ExecutionState, Hash32, InputStatus, L1Position, MAX_FIXTURE_ROWS, NativeBlock,
    PublicConfig, ReplayPosition, VirtualBlock,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureSnapshot {
    pub source_fixture: String,
    pub gpos: Vec<u8>,
    pub position: ReplayPosition,
    pub config: PublicConfig,
    pub dep: DepFixture,
    pub parent: ExecutionState,
    pub native_parent: Hash32,
    pub blocks: Vec<VirtualBlock>,
    pub control_coverage: L1Position,
    pub processed_resumes: Vec<ResumeRequirement>,
}

pub struct SourceQuery<'a> {
    pub gpos: &'a [u8],
    pub first: u64,
    pub last: u64,
    pub cursor_written_at: L1Position,
    pub required_resume: Option<ResumeRequirement>,
}

pub trait BatchSource {
    fn checked_snapshot(&self, query: &SourceQuery<'_>) -> Result<&FixtureSnapshot, ReadError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FixtureMissing {
    Anchor,
    State,
    Metadata,
    Deposits,
    Height(u64),
}

#[derive(Default)]
pub struct FixtureSourceView {
    snapshots: BTreeMap<Vec<u8>, FixtureSnapshot>,
    missing: BTreeMap<Vec<u8>, FixtureMissing>,
}

impl FixtureSourceView {
    /// An explicit data-delivery wait, distinct from a supplied contradictory snapshot.
    pub fn defer(&mut self, gpos: Vec<u8>, material: FixtureMissing) -> Result<(), ReadError> {
        if !(1..=1024).contains(&gpos.len()) || self.snapshots.contains_key(&gpos) {
            return Err(reject("invalid fixture deferral"));
        }
        self.missing.insert(gpos, material);
        Ok(())
    }

    pub fn deliver(&mut self, snapshot: FixtureSnapshot) -> Result<(), ReadError> {
        if !(1..=1024).contains(&snapshot.gpos.len()) || snapshot.blocks.len() > MAX_FIXTURE_ROWS {
            return Err(reject("fixture gpos or block bound"));
        }
        if let Some(old) = self.snapshots.get(&snapshot.gpos) {
            if old != &snapshot {
                return Err(reject("immutable source snapshot conflicts"));
            }
            return Ok(());
        }
        self.missing.remove(&snapshot.gpos);
        self.snapshots.insert(snapshot.gpos.clone(), snapshot);
        Ok(())
    }
}

impl BatchSource for FixtureSourceView {
    fn checked_snapshot(&self, query: &SourceQuery<'_>) -> Result<&FixtureSnapshot, ReadError> {
        if let Some(missing) = self.missing.get(query.gpos) {
            return Err(wait(&format!(
                "fixture source material pending: {missing:?}"
            )));
        }
        self.snapshots
            .get(query.gpos)
            .ok_or_else(|| wait("exact gpos fixture snapshot unavailable"))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppliedBlock {
    pub height: u64,
    pub kept: Vec<Vec<u8>>,
    pub post_state: ExecutionState,
    pub post_state_root: Hash32,
    pub native_hash: Hash32,
}

pub(crate) struct Replayed {
    pub(crate) fingerprint: Hash32,
    pub(crate) records: Vec<BlockRecord>,
    pub(crate) virtual_blocks: Vec<VirtualBlock>,
    pub(crate) audit: Vec<AppliedBlock>,
    pub(crate) native_parent: Hash32,
}

pub(crate) fn replay(
    source: &dyn BatchSource,
    cursor: &CurrentCursor,
    gpos: &[u8],
    first: u64,
    last: u64,
    config: &ServiceConfig,
) -> Result<Replayed, ReadError> {
    replay_range(
        source,
        &SourceQuery {
            gpos,
            first,
            last,
            cursor_written_at: cursor.written_at(),
            required_resume: cursor.latest_resume(),
        },
        cursor.cursor(),
        config,
    )
}

pub(crate) fn replay_range(
    source: &dyn BatchSource,
    query: &SourceQuery<'_>,
    prev: &CursorWire,
    config: &ServiceConfig,
) -> Result<Replayed, ReadError> {
    let SourceQuery {
        gpos, first, last, ..
    } = *query;
    if !(1..=1024).contains(&gpos.len())
        || !matches!(last.checked_sub(first), Some(0 | 1))
        || prev.height.checked_add(1) != Some(first)
    {
        return Err(reject("invalid source request range or gpos"));
    }
    let source = source.checked_snapshot(query)?;
    if source.gpos != gpos
        || source.source_fixture != config.source_fixture
        || source
            .config
            .reference()
            .map_err(|e| reject(&e.to_string()))?
            .as_hash()
            != &config.inbox.config_ref
        || source.config.chain_id != config.inbox.chain_id
    {
        return Err(reject("source fixture identity mismatch"));
    }
    source
        .position
        .validate()
        .map_err(|e| reject(&e.to_string()))?;
    if query.required_resume.is_some_and(|control| {
        source.control_coverage < control.resume || !source.processed_resumes.contains(&control)
    }) {
        return Err(wait(
            "source has not covered and processed required RESUME history",
        ));
    }
    let parent_height = source.parent.metadata.height;
    if parent_height >= first || parent_height < source.config.genesis.block.height {
        return Err(reject("source parent lies outside requested prefix"));
    }
    if source.parent.metadata.height == source.config.genesis.block.height
        && (source.parent.metadata.origin != source.config.genesis.block.origin
            || source.parent.metadata.timestamp_us != source.config.genesis.timestamp_us
            || source.native_parent != source.config.genesis_native_hash
            || source.parent.root().map_err(|e| reject(&e.to_string()))?
                != source.config.genesis_state_root)
    {
        return Err(reject("parent disagrees with configured genesis"));
    }
    let dep = ValidatedDepFixture::load(&source.config, source.dep.clone())
        .map_err(|e| reject(&format!("source DEP fixture: {e:?}")))?;
    let count = (last - first + 1) as usize;
    let through = usize::try_from(last - parent_height)
        .map_err(|_| wait("source lacks requested virtual heights"))?;
    if source.blocks.len() < through {
        return Err(wait("source lacks requested virtual heights"));
    }
    let start = through - count;
    let mut state = source.parent.clone();
    let mut parent = source.native_parent;
    let mut records = Vec::with_capacity(count);
    let mut audit = Vec::with_capacity(through);
    let mut selected_parent = parent;
    for (offset, block) in source.blocks[..through].iter().enumerate() {
        let metadata = &block.metadata;
        if metadata.height == first {
            selected_parent = parent;
        }
        if metadata.height != parent_height + offset as u64 + 1
            || (metadata.height == first && state.metadata.origin != prev.origin)
            || l2_timestamp_us(&source.config, metadata.height) != Ok(metadata.timestamp_us)
            || metadata
                .origin
                .checked_sub(state.metadata.origin)
                .is_none_or(|delta| delta > 1)
            || state.root().map_err(|e| reject(&e.to_string()))? != block.pre_state_root
        {
            return Err(reject("source block metadata or pre-state mismatch"));
        }
        let expected_dep = match dep.at(metadata) {
            InputStatus::Ready(dep) => dep,
            InputStatus::Wait(_)
            | InputStatus::Reject(poc_types::domain::InputError::FixtureExhausted { .. }) => {
                return Err(wait("required DEP data unavailable"));
            }
            InputStatus::Reject(e) => return Err(reject(&format!("source DEP rejected: {e:?}"))),
        };
        if expected_dep != block.dep {
            return Err(reject("source block contains contradictory DEP bytes"));
        }
        let output = apply_with_byte_limit(
            &state,
            metadata,
            &expected_dep,
            &block.transactions,
            source.config.byte_limit as usize,
        )
        .map_err(|e| reject(&format!("03 Apply rejected source: {e:?}")))?;
        state = output.state;
        let native = NativeBlock {
            parent,
            metadata: metadata.clone(),
            dep: expected_dep,
            kept: output.kept,
            post_state_root: state.root().map_err(|e| reject(&e.to_string()))?,
        };
        parent = native.digest().map_err(|e| reject(&e.to_string()))?;
        if metadata.height >= first {
            records.push(BlockRecord {
                block_number: metadata.height,
                epoch_num: metadata.origin,
                transactions: native.kept.clone(),
            });
        }
        audit.push(AppliedBlock {
            height: metadata.height,
            kept: native.kept,
            post_state: state.clone(),
            post_state_root: native.post_state_root,
            native_hash: parent,
        });
    }
    // Local journal fingerprint only. No public ReplayPosition wire codec.
    Ok(Replayed {
        fingerprint: commitment(&json_bytes(source)?),
        records,
        virtual_blocks: source.blocks[start..through].to_vec(),
        audit,
        native_parent: selected_parent,
    })
}
