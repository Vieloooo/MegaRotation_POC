//! Verify a physical chain and expand only the virtual heights covered by its tip.

use crate::{
    execution::{ExecutionError, apply_with_byte_limit},
    fixture::DepProvider,
    metadata::{MetadataProvider, l2_timestamp_us},
};
use poc_types::domain::{
    Anchor, DomainError, ExecutionAnchor, ExecutionState, Hash32, InputError, InputMissing,
    InputStatus, MAX_FIXTURE_ROWS, NativeBlock, PhysicalBlock, PublicConfig, VirtualBlock, Window,
    validate_transactions,
};

const MAX_PHYSICAL_BLOCKS: usize = 10;
const MAX_EXPANDED_HEIGHTS: usize = MAX_FIXTURE_ROWS;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MergedBlock {
    pub virtual_block: VirtualBlock,
    pub physical_digest: Option<Hash32>,
    pub state: ExecutionState,
    pub native_block: NativeBlock,
    pub native_hash: Hash32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MergeOutput {
    pub blocks: Vec<MergedBlock>,
}

pub type MergeOutcome = InputStatus<MergeOutput>;

/// Reconstruct a complete ordered physical chain. A supplied conflicting chain rejects.
/// Use merge_from_tip to fetch the bodies of a selected result by digest.
pub fn merge(
    anchor: &ExecutionAnchor,
    chain: &[PhysicalBlock],
    window: &Window,
    metadata: &dyn MetadataProvider,
    dep: &dyn DepProvider,
    config: &PublicConfig,
) -> MergeOutcome {
    let digests = match validate_chain(anchor, chain, window, metadata, dep, config) {
        Ok(digests) => digests,
        Err(error) => return InputStatus::Reject(error),
    };
    let Some(tip) = chain.last() else {
        return InputStatus::Ready(MergeOutput { blocks: vec![] });
    };

    let mut state = anchor.state.clone();
    let mut pre_state_root = match state.root() {
        Ok(root) => root,
        Err(error) => return InputStatus::Reject(InputError::Malformed(error)),
    };
    let mut native_parent = anchor.native_hash;
    let mut next_physical = chain.iter().zip(digests).peekable();
    let mut blocks = Vec::new();

    // validate_chain checked strict progress and the expansion bound before this loop.
    for height in (anchor.anchor.block.height + 1)..=tip.metadata.height {
        let physical = next_physical.next_if(|(block, _)| block.metadata.height == height);
        let actual = match metadata.at(&anchor.anchor, height) {
            InputStatus::Ready(actual) => actual,
            InputStatus::Wait(missing) => return InputStatus::Wait(missing),
            InputStatus::Reject(error) => return InputStatus::Reject(error),
        };
        if actual.height != height
            || l2_timestamp_us(config, height) != Ok(actual.timestamp_us)
            || !actual
                .origin
                .checked_sub(state.metadata.origin)
                .is_some_and(|n| n <= 1)
        {
            return reject("inconsistent merge metadata");
        }

        let (transactions, physical_digest) = if let Some((block, digest)) = physical {
            if block.metadata != actual {
                return reject("physical metadata mismatch");
            }
            if block.pre_state_root != pre_state_root {
                return reject("physical pre-state root mismatch");
            }
            (block.transactions.as_slice(), Some(digest))
        } else {
            (&[][..], None)
        };
        let dep = match dep.at(&actual) {
            InputStatus::Ready(dep) => dep,
            InputStatus::Wait(missing) => return InputStatus::Wait(missing),
            InputStatus::Reject(error) => return InputStatus::Reject(error),
        };
        let applied = match apply_with_byte_limit(
            &state,
            &actual,
            &dep,
            transactions,
            config.byte_limit as usize,
        ) {
            Ok(output) => output,
            Err(ExecutionError::Malformed(error)) => {
                return InputStatus::Reject(InputError::Malformed(error));
            }
            Err(ExecutionError::CreditOverflow { .. }) => {
                return InputStatus::Reject(InputError::Malformed(DomainError::Overflow));
            }
        };
        let post_state_root = match applied.state.root() {
            Ok(root) => root,
            Err(error) => return InputStatus::Reject(InputError::Malformed(error)),
        };
        let native_block = NativeBlock {
            parent: native_parent,
            metadata: actual.clone(),
            dep: dep.clone(),
            kept: applied.kept,
            post_state_root,
        };
        let native_hash = match native_block.digest() {
            Ok(hash) => hash,
            Err(error) => return InputStatus::Reject(InputError::Malformed(error)),
        };
        state = applied.state;
        blocks.push(MergedBlock {
            virtual_block: VirtualBlock {
                metadata: actual,
                dep,
                pre_state_root,
                transactions: transactions.to_vec(),
            },
            physical_digest,
            state: state.clone(),
            native_block,
            native_hash,
        });
        native_parent = native_hash;
        pre_state_root = post_state_root;
    }

    InputStatus::Ready(MergeOutput { blocks })
}

fn validate_chain(
    anchor: &ExecutionAnchor,
    chain: &[PhysicalBlock],
    window: &Window,
    metadata: &dyn MetadataProvider,
    dep: &dyn DepProvider,
    config: &PublicConfig,
) -> Result<Vec<Hash32>, InputError> {
    validate_context(&anchor.anchor, window, metadata, dep, config)?;
    anchor.validate().map_err(InputError::Malformed)?;
    if anchor.anchor.block.height == config.genesis.block.height
        && (anchor.native_hash != config.genesis_native_hash
            || anchor.state.root().map_err(InputError::Malformed)? != config.genesis_state_root)
    {
        return Err(InputError::InvalidSource);
    }
    if chain.len() > MAX_PHYSICAL_BLOCKS {
        return Err(limit(MAX_PHYSICAL_BLOCKS, chain.len()));
    }
    let mut previous_height = anchor.anchor.block.height;
    let mut expected_parent = anchor.anchor.block.block_id;
    let mut digests = Vec::with_capacity(chain.len());
    for block in chain {
        let height = block.metadata.height;
        if height <= previous_height || !(window.h_min..=window.h_max).contains(&height) {
            return Err(malformed("invalid physical height"));
        }
        if block.parent != expected_parent {
            return Err(InputError::Conflict);
        }
        let span = height - anchor.anchor.block.height;
        if span > MAX_EXPANDED_HEIGHTS as u64 {
            return Err(limit(
                MAX_EXPANDED_HEIGHTS,
                usize::try_from(span).unwrap_or(usize::MAX),
            ));
        }
        if block.metadata.timestamp_us
            != l2_timestamp_us(config, height).map_err(InputError::Malformed)?
        {
            return Err(malformed("physical timestamp mismatch"));
        }
        validate_transactions(&block.transactions, config.byte_limit as usize)
            .map_err(InputError::Malformed)?;
        expected_parent = block.digest().map_err(InputError::Malformed)?;
        previous_height = height;
        digests.push(expected_parent);
    }
    Ok(digests)
}

fn validate_context(
    anchor: &Anchor,
    window: &Window,
    metadata: &dyn MetadataProvider,
    dep: &dyn DepProvider,
    config: &PublicConfig,
) -> Result<(), InputError> {
    let config_ref = config.reference().map_err(InputError::Malformed)?;
    if metadata.config_ref() != config_ref || dep.config_ref() != config_ref {
        return Err(InputError::InvalidSource);
    }
    window.validate().map_err(InputError::Malformed)?;
    if anchor.block.height < config.genesis.block.height || anchor.block.height >= window.h_min {
        return Err(malformed("anchor outside physical window"));
    }
    if anchor.timestamp_us
        != l2_timestamp_us(config, anchor.block.height).map_err(InputError::Malformed)?
    {
        return Err(malformed("invalid anchor timestamp"));
    }
    if anchor.block.height == config.genesis.block.height && *anchor != config.genesis {
        return Err(InputError::InvalidSource);
    }
    Ok(())
}

pub trait PhysicalSource {
    fn physical_block(&self, digest: Hash32) -> InputStatus<PhysicalBlock>;
}

/// Material for a selected physical tip. Anchor and tip identities come from the caller's proof context.
pub struct SelectedTip<'a> {
    pub anchor: &'a Anchor,
    pub execution_anchor: Option<&'a ExecutionAnchor>,
    pub tip: Hash32,
}

pub fn merge_from_tip(
    selected: SelectedTip<'_>,
    source: &dyn PhysicalSource,
    window: &Window,
    metadata: &dyn MetadataProvider,
    dep: &dyn DepProvider,
    config: &PublicConfig,
) -> MergeOutcome {
    if let Err(error) = validate_context(selected.anchor, window, metadata, dep, config) {
        return InputStatus::Reject(error);
    }
    let mut digest = selected.tip;
    let anchor_id = selected.anchor.block.block_id;
    let mut h_next = None;
    let mut chain = Vec::new();
    while digest != anchor_id {
        if chain.len() == MAX_PHYSICAL_BLOCKS {
            return InputStatus::Reject(limit(MAX_PHYSICAL_BLOCKS, MAX_PHYSICAL_BLOCKS + 1));
        }
        let block = match source.physical_block(digest) {
            InputStatus::Ready(block) => block,
            InputStatus::Wait(_) => return InputStatus::Wait(InputMissing::Parent(digest)),
            InputStatus::Reject(error) => return InputStatus::Reject(error),
        };
        match block.digest() {
            Ok(actual) if actual == digest => {}
            Ok(_) => return InputStatus::Reject(InputError::Conflict),
            Err(error) => return InputStatus::Reject(InputError::Malformed(error)),
        }
        let height = block.metadata.height;
        if !(window.h_min..=window.h_max).contains(&height)
            || h_next.is_some_and(|next| height >= next)
        {
            return reject("invalid physical ancestry");
        }
        h_next = Some(height);
        digest = block.parent;
        chain.push(block);
    }
    let Some(execution_anchor) = selected.execution_anchor else {
        return InputStatus::Wait(InputMissing::State(anchor_id));
    };
    if execution_anchor.anchor != *selected.anchor {
        return InputStatus::Reject(InputError::Conflict);
    }
    chain.reverse();
    merge(execution_anchor, &chain, window, metadata, dep, config)
}

fn limit(limit: usize, actual: usize) -> InputError {
    InputError::Malformed(DomainError::LimitExceeded { limit, actual })
}

fn malformed(reason: &'static str) -> InputError {
    InputError::Malformed(DomainError::Malformed { reason })
}

fn reject<T>(reason: &'static str) -> InputStatus<T> {
    InputStatus::Reject(malformed(reason))
}
