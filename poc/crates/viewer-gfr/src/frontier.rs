use std::{cmp::Ordering, collections::BTreeSet, sync::Arc};

use poc_core::metadata::l2_timestamp_us;
use poc_types::domain::{DomainError, MAX_FIXTURE_ROWS};
use sim_espresso::{FinalizedBlock, MAX_BLOCKS, MissingLog, ReadError};

use crate::{
    ControlRef, GfrError, GfrParameters, Hash32, InputMissing, InputRef, InputStatus, L1Header,
    Missing, PublicConfig, ReplayIndex, ReplayKind, ReplayPosition, ReplayReads, ReplayState,
    ReplayStatus, SupportingCuts, Tail, Window, compare_positions,
};

pub(crate) struct Sources<'a> {
    pub(crate) espresso: &'a [FinalizedBlock],
    l1_headers: Vec<L1Header>,
    l1_controls: Vec<Vec<ControlRef>>,
    first_missing_controls: Option<InputMissing>,
}

pub(crate) struct OrderedEvent {
    pub(crate) position: ReplayPosition,
    pub(crate) input_ref: InputRef,
    espresso_cut: usize,
    l1_cut: usize,
}

impl<'a> Sources<'a> {
    pub(crate) fn read(reads: &ReplayReads<'a>, state: &ReplayState) -> Result<Self, GfrError> {
        if reads.espresso.source_id() != state.public.espresso_source
            || reads.l1.source_id() != state.public.l1_source
        {
            return Err(GfrError::InvalidSource);
        }
        let mut source = Self {
            espresso: read_espresso(reads)?,
            l1_headers: Vec::new(),
            l1_controls: Vec::new(),
            first_missing_controls: None,
        };
        source.read_l1(reads, &state.public.config)?;
        let cuts = &state.cuts;
        if source.espresso.len() < cuts.espresso.len()
            || !cuts
                .espresso
                .iter()
                .zip(source.espresso)
                .all(|(old, new)| old.as_ref() == new)
            || !source.l1_headers.starts_with(&cuts.l1_headers)
            || !source.l1_controls.starts_with(&cuts.l1_controls)
        {
            return Err(GfrError::InvalidSource);
        }
        Ok(source)
    }

    fn read_l1(&mut self, reads: &ReplayReads<'_>, config: &PublicConfig) -> Result<(), GfrError> {
        let Some(tip) = reads.l1.finalized_tip() else {
            return Ok(());
        };
        ensure_index(tip.height, MAX_FIXTURE_ROWS)?;
        let mut parent = Hash32::new([0; 32]);
        let mut seen_sequences = BTreeSet::new();
        for height in 0..=tip.height {
            let header = match reads.l1.header(height) {
                InputStatus::Ready(header) => header,
                InputStatus::Wait(_) => return Err(GfrError::InvalidSource),
                InputStatus::Reject(error) => return Err(error.into()),
            };
            if header.height != height
                || header.parent_hash != parent
                || header.timestamp_us()? != l1_timestamp(config, height)?
            {
                return Err(GfrError::InvalidSource);
            }
            parent = header.hash;
            self.l1_headers.push(header);
            if self.first_missing_controls.is_some() {
                continue;
            }
            let controls = match reads.l1.require_control_coverage(height) {
                InputStatus::Ready(()) => reads.l1.controls_at(height),
                InputStatus::Wait(missing) => InputStatus::Wait(missing),
                InputStatus::Reject(error) => InputStatus::Reject(error),
            };
            match controls {
                InputStatus::Ready(controls) => {
                    validate_controls(height, &controls, &mut seen_sequences)?;
                    self.l1_controls.push(controls);
                }
                InputStatus::Wait(missing) => self.first_missing_controls = Some(missing),
                InputStatus::Reject(error) => return Err(error.into()),
            }
        }
        if parent != tip.hash {
            return Err(GfrError::InvalidSource);
        }
        Ok(())
    }

    pub(crate) fn next_event(
        &self,
        state: &ReplayState,
        reads: &ReplayReads<'_>,
        through: Option<&ReplayPosition>,
    ) -> Result<Option<OrderedEvent>, ReplayStatus> {
        let next = self.earliest(state).map_err(ReplayStatus::Reject)?;
        let next = match (next, through) {
            (Some(next), Some(limit))
                if compare_positions(&next.0, limit).map_err(ReplayStatus::Reject)?
                    == Ordering::Greater =>
            {
                None
            }
            (next, _) => next,
        };
        let key = match (&next, through) {
            (Some((position, _)), _) => position.key_us,
            (None, Some(limit)) => limit.key_us,
            (None, None) if self.espresso.len() == MAX_BLOCKS => return Ok(None),
            (None, None) if !self.espresso.is_empty() => {
                let missing = missing_espresso(reads, state.next_q)?;
                return if matches!(missing, Missing::EspressoMessages { .. }) {
                    Err(frontier_wait(vec![missing]))
                } else {
                    Ok(None)
                };
            }
            (None, None) => return Err(frontier_wait(vec![missing_espresso(reads, 0)?])),
        };
        let mut missing = Vec::new();
        let watermarker = self
            .espresso
            .partition_point(|block| block.proposal.theta_us.0 < key);
        let espresso_cut = if watermarker < self.espresso.len() {
            watermarker + 1
        } else {
            let q = u64::try_from(self.espresso.len())
                .map_err(|_| ReplayStatus::Reject(DomainError::Overflow.into()))?;
            missing.push(missing_espresso(reads, q)?);
            missing.push(Missing::EspressoWatermark { key_us: key });
            0
        };

        // An equal-key watermark alone cannot close an Espresso bound beyond that q.
        if let (
            None,
            Some(ReplayPosition {
                index: ReplayIndex::Espresso(q),
                ..
            }),
            Some(tip),
        ) = (&next, through, self.espresso.last())
            && tip.proposal.theta_us.0 == key
            && tip.proposal.q < *q
        {
            let next_q = tip
                .proposal
                .q
                .checked_add(1)
                .ok_or_else(|| ReplayStatus::Reject(DomainError::Overflow.into()))?;
            missing.push(missing_espresso(reads, next_q)?);
        }

        let height = covering_l1_height(&state.public.config, state.public.parameters, key)
            .map_err(ReplayStatus::Reject)?;
        let index = ensure_index(height, MAX_FIXTURE_ROWS).map_err(ReplayStatus::Reject)?;
        if self.l1_controls.len() <= index {
            missing.push(self.missing_l1(reads, height, index)?);
        }
        if !missing.is_empty() {
            missing.dedup();
            return Err(frontier_wait(missing));
        }
        next.map(|(position, input_ref)| {
            if let Some(pc) = &state.public.pc
                && compare_positions(&position, pc).map_err(ReplayStatus::Reject)?
                    != Ordering::Greater
            {
                return Err(ReplayStatus::Reject(GfrError::InvalidSource));
            }
            // A same-key later q uses its own complete block.
            let espresso_cut = match input_ref {
                InputRef::Espresso(q) => {
                    ensure_index(q, MAX_BLOCKS).map_err(ReplayStatus::Reject)? + 1
                }
                _ => espresso_cut,
            };
            Ok(OrderedEvent {
                position,
                input_ref,
                espresso_cut,
                l1_cut: index + 1,
            })
        })
        .transpose()
    }

    fn earliest(
        &self,
        state: &ReplayState,
    ) -> Result<Option<(ReplayPosition, InputRef)>, GfrError> {
        let mut earliest = None;
        if let Some(block) = self.espresso.get(u64_index(state.next_q)?) {
            earliest = Some((
                espresso_position(block),
                InputRef::Espresso(block.proposal.q),
            ));
        }
        if !matches!(state.public.tail, Tail::Halt { .. }) {
            let h = add(state.head_height(), 1)?;
            insert_earlier(
                &mut earliest,
                (
                    ReplayPosition {
                        key_us: expiry_key(&state.public.config, state.public.parameters, h)?,
                        kind: ReplayKind::Expire,
                        index: ReplayIndex::Height(h),
                    },
                    InputRef::Expire(h),
                ),
            )?;
        }
        if let Tail::Next(descriptor) = &state.public.tail {
            let session = descriptor.plan.session.clone();
            insert_earlier(
                &mut earliest,
                (
                    ReplayPosition {
                        key_us: session_end_key(
                            &state.public.config,
                            state.public.parameters,
                            &descriptor.plan.window,
                        )?,
                        kind: ReplayKind::SessionEnd,
                        index: ReplayIndex::Session(session.clone()),
                    },
                    InputRef::SessionEnd(session),
                ),
            )?;
        }
        // No control gets committed at this stage.
        if let Some(control) = self.l1_controls.iter().find_map(|row| row.first()) {
            let key_us = add(
                l1_timestamp(&state.public.config, control.position.block)?,
                state.public.parameters.announcement_us,
            )?;
            insert_earlier(
                &mut earliest,
                (
                    ReplayPosition {
                        key_us,
                        kind: control.kind,
                        index: ReplayIndex::L1(control.position),
                    },
                    InputRef::L1(control.clone()),
                ),
            )?;
        }
        Ok(earliest)
    }

    fn missing_l1(
        &self,
        reads: &ReplayReads<'_>,
        height: u64,
        index: usize,
    ) -> Result<Missing, ReplayStatus> {
        if self.l1_headers.len() <= index {
            return match reads.l1.header(height) {
                InputStatus::Wait(missing) => Ok(input_missing(missing)),
                InputStatus::Reject(error) => Err(ReplayStatus::Reject(error.into())),
                InputStatus::Ready(_) => Err(ReplayStatus::Reject(GfrError::InvalidSource)),
            };
        }
        self.first_missing_controls
            .clone()
            .map(input_missing)
            .ok_or(ReplayStatus::Reject(GfrError::InvalidSource))
    }

    pub(crate) fn support(
        &self,
        target: &mut SupportingCuts,
        event: &OrderedEvent,
        origin: Option<u64>,
    ) -> Result<(), GfrError> {
        let header_count = match origin {
            Some(origin) => event
                .l1_cut
                .max(ensure_index(origin, MAX_FIXTURE_ROWS)? + 1),
            None => event.l1_cut,
        };
        if header_count > self.l1_headers.len() {
            return Err(GfrError::InvalidSource);
        }
        if target.espresso.len() < event.espresso_cut {
            target.espresso.extend(
                self.espresso[target.espresso.len()..event.espresso_cut]
                    .iter()
                    .cloned()
                    .map(Arc::new),
            );
        }
        if target.l1_headers.len() < header_count {
            target
                .l1_headers
                .extend_from_slice(&self.l1_headers[target.l1_headers.len()..header_count]);
        }
        if target.l1_controls.len() < event.l1_cut {
            target
                .l1_controls
                .extend_from_slice(&self.l1_controls[target.l1_controls.len()..event.l1_cut]);
        }
        Ok(())
    }
}

fn read_espresso<'a>(reads: &ReplayReads<'a>) -> Result<&'a [FinalizedBlock], GfrError> {
    let Some(tip) = reads.espresso.tip() else {
        return Ok(&[]);
    };
    let q = ensure_index(tip.0, MAX_BLOCKS)?;
    let prefix = reads
        .espresso
        .require_prefix_through(tip.0)
        .map_err(|error| match error {
            ReadError::Wait { .. } => GfrError::InvalidSource,
            ReadError::InvalidSource(error) => GfrError::EspressoSource(error),
        })?;
    if prefix.tip() != tip
        || prefix.source_id() != reads.espresso.source_id()
        || prefix.blocks().len() != q + 1
    {
        return Err(GfrError::InvalidSource);
    }
    Ok(prefix.blocks())
}

fn validate_controls(
    height: u64,
    controls: &[ControlRef],
    sequences: &mut BTreeSet<u64>,
) -> Result<(), GfrError> {
    let mut previous = None;
    for control in controls {
        if control.position.block != height
            || !matches!(control.kind, ReplayKind::Force | ReplayKind::Resume)
        {
            return Err(GfrError::InvalidSource);
        }
        if previous.is_some_and(|old| old >= control.position)
            || !sequences.insert(control.control_seq)
        {
            return Err(GfrError::ConflictingControl(control.position));
        }
        previous = Some(control.position);
    }
    Ok(())
}

fn missing_espresso(reads: &ReplayReads<'_>, q: u64) -> Result<Missing, ReplayStatus> {
    ensure_index(q, MAX_BLOCKS).map_err(ReplayStatus::Reject)?;
    match reads.espresso.read(q) {
        Ok(_) => Err(ReplayStatus::Reject(GfrError::InvalidSource)),
        Err(ReadError::InvalidSource(error)) => {
            Err(ReplayStatus::Reject(GfrError::EspressoSource(error)))
        }
        Err(ReadError::Wait { next_q, missing }) if next_q == q => Ok(match missing {
            MissingLog::Header => Missing::EspressoHeader { q },
            MissingLog::Messages { mut indices } => {
                indices.sort_unstable();
                indices.dedup();
                Missing::EspressoMessages { q, indices }
            }
        }),
        Err(ReadError::Wait { .. }) => Err(ReplayStatus::Reject(GfrError::InvalidSource)),
    }
}

pub(crate) fn input_missing(value: InputMissing) -> Missing {
    match value {
        InputMissing::L1Header(height) => Missing::L1Header { height },
        InputMissing::ControlCoverage(through_height) => {
            Missing::L1ControlCoverage { through_height }
        }
        value => Missing::Input(value),
    }
}

fn frontier_wait(missing: Vec<Missing>) -> ReplayStatus {
    ReplayStatus::Wait {
        next_position: None,
        missing,
    }
}

fn espresso_position(block: &FinalizedBlock) -> ReplayPosition {
    ReplayPosition {
        key_us: block.proposal.theta_us.0,
        kind: ReplayKind::Espresso,
        index: ReplayIndex::Espresso(block.proposal.q),
    }
}

fn insert_earlier(
    earliest: &mut Option<(ReplayPosition, InputRef)>,
    candidate: (ReplayPosition, InputRef),
) -> Result<(), GfrError> {
    if let Some((position, _)) = earliest
        && compare_positions(&candidate.0, position)? != Ordering::Less
    {
        return Ok(());
    }
    *earliest = Some(candidate);
    Ok(())
}

fn l1_timestamp(config: &PublicConfig, height: u64) -> Result<u64, GfrError> {
    add(
        config.l1_genesis_timestamp_us,
        height
            .checked_mul(config.l1_block_us)
            .ok_or(DomainError::Overflow)?,
    )
}

fn covering_l1_height(
    config: &PublicConfig,
    parameters: GfrParameters,
    key: u64,
) -> Result<u64, GfrError> {
    // Clamp at the genesis witness and calculate a checked ceiling.
    let elapsed = key
        .saturating_sub(parameters.announcement_us)
        .saturating_sub(config.l1_genesis_timestamp_us);
    let whole = elapsed / config.l1_block_us;
    add(
        whole,
        u64::from(!elapsed.is_multiple_of(config.l1_block_us)),
    )
}

pub(crate) fn expiry_key(
    config: &PublicConfig,
    parameters: GfrParameters,
    height: u64,
) -> Result<u64, GfrError> {
    add(l2_timestamp_us(config, height)?, parameters.expiry_us)
}

pub(crate) fn session_end_key(
    config: &PublicConfig,
    parameters: GfrParameters,
    window: &Window,
) -> Result<u64, GfrError> {
    add(
        l2_timestamp_us(config, add(window.h_max, 1)?)?,
        parameters.grace_us,
    )
}

pub(crate) fn add(a: u64, b: u64) -> Result<u64, GfrError> {
    Ok(a.checked_add(b).ok_or(DomainError::Overflow)?)
}

pub(crate) fn ensure_index(index: u64, limit: usize) -> Result<usize, GfrError> {
    let index = u64_index(index)?;
    if index >= limit {
        return Err(DomainError::LimitExceeded {
            limit,
            actual: index.checked_add(1).ok_or(DomainError::Overflow)?,
        }
        .into());
    }
    Ok(index)
}

pub(crate) fn u64_index(index: u64) -> Result<usize, GfrError> {
    Ok(usize::try_from(index).map_err(|_| DomainError::Overflow)?)
}
