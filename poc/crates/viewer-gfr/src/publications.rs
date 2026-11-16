use std::{cell::Cell, collections::BTreeMap, sync::Arc};

use poc_core::{
    merge::{MergedBlock, merge},
    metadata::{MetadataProvider, RegionalMetadata, l2_timestamp_us},
};
use poc_types::domain::{DomainError, MAX_FIXTURE_ROWS, Metadata, validate_transactions};
use sim_espresso::{EspressoPosition, FinalizedBlock, VerifiedPrefix};

use crate::{
    Anchor, ConfigRef, Descriptor, ExecutionAnchor, GfrError, Hash32, InputError, InputStatus,
    Missing, PUBLICATION_CODEC, PhysicalBlock, PublicConfig, PublicObject, ReplayReads,
    ReplayServices, ResultKey, VerificationProfile, VerifierBinding, decode_publication,
    frontier::{expiry_key, input_missing, session_end_key, u64_index},
    reader::validate_source_id,
    replay::{StepError, StepResult, verify_profile},
    state::PublicReplayState,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PublicationIndex {
    pub bodies: BTreeMap<Hash32, BodyEvidence>,
    pub results: BTreeMap<ResultKey, ResultEvidence>,
    pub ignored: BTreeMap<EspressoPosition, PublicationDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyEvidence {
    pub payload: Arc<[u8]>,
    pub block: Arc<PhysicalBlock>,
    pub locations: Vec<EspressoPosition>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResultEvidence {
    /// Exact canonical payload as key, retaining every public position for duplicate bytes.
    pub proofs: BTreeMap<Arc<[u8]>, ProofEvidence>,
    /// Computed while this exact descriptor holds NEXT. Raw evidence for other duties remains unchecked.
    pub q_avail: Option<u64>,
    pub physical_chain: Vec<Hash32>,
    pub merge_verified: bool,
    pub exclusion: Option<ResultExclusion>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofEvidence {
    pub descriptor: Arc<Descriptor>,
    pub proof: Arc<[u8]>,
    pub locations: Vec<EspressoPosition>,
    pub verdict: ProofVerdict,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProofVerdict {
    Unchecked,
    DescriptorMismatch,
    Invalid,
    Valid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResultExclusion {
    InvalidAncestry,
    InvalidExecution(InputError),
    PublicCutoff,
    Expired,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PublicationDiagnostic {
    UnknownCodec(u32),
    Malformed(GfrError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicCompleteness {
    pub key: ResultKey,
    pub first_complete_q: u64,
    pub publication_position: EspressoPosition,
    /// Physical bodies ordered from the anchor's child through the target.
    pub physical_chain: Vec<PhysicalBlock>,
    pub valid_proof_locations: Vec<EspressoPosition>,
    pub body_locations: Vec<(Hash32, Vec<EspressoPosition>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompletenessInvalid {
    Ancestry,
    FinalProof,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompletenessOutcome {
    Complete(PublicCompleteness),
    Incomplete,
    Wait(Missing),
    Invalid(CompletenessInvalid),
    Reject(GfrError),
}

/// Evaluate exact public evidence independent of selection and acceptance time.
pub fn public_completeness(
    prefix: &VerifiedPrefix<'_>,
    through_q: u64,
    config: &PublicConfig,
    expected: &VerificationProfile,
    descriptor: &Descriptor,
    target: Hash32,
    finality: Option<&VerifierBinding<'_>>,
) -> CompletenessOutcome {
    let check = || -> StepResult<CompletenessOutcome> {
        if through_q > prefix.tip().0 {
            return Err(GfrError::HistoricalBound.into());
        }
        validate_source_id(prefix.source_id())?;
        let config_ref = config.reference()?;
        verify_profile(config, expected, finality)?;
        descriptor.validate()?;
        if descriptor.plan.config_ref != config_ref {
            return Err(GfrError::ConfigMismatch.into());
        }
        let key = ResultKey {
            cid: descriptor.cid,
            block_hash: target,
        };
        let mut index = PublicationIndex::default();
        let cut = prefix
            .blocks()
            .get(..=u64_index(through_q)?)
            .ok_or(GfrError::InvalidSource)?;
        for block in cut {
            index.index(block)?;
        }
        let Some(evidence) = index.results.get_mut(&key) else {
            return Ok(CompletenessOutcome::Incomplete);
        };
        let complete = match evaluate_completeness(
            &index.bodies,
            evidence,
            key,
            descriptor,
            config,
            finality,
        )? {
            EvaluationOutcome::Incomplete => return Ok(CompletenessOutcome::Incomplete),
            EvaluationOutcome::Invalid(reason) => return Ok(CompletenessOutcome::Invalid(reason)),
            EvaluationOutcome::Complete(complete) => complete,
        };
        let mut valid_proof_locations: Vec<_> = valid_proof_positions(evidence).collect();
        valid_proof_locations.sort_unstable();
        valid_proof_locations.dedup();
        let body_locations = complete
            .digests
            .into_iter()
            .map(|digest| {
                let mut locations = index.bodies[&digest].locations.clone();
                locations.sort_unstable();
                locations.dedup();
                (digest, locations)
            })
            .collect();
        Ok(CompletenessOutcome::Complete(PublicCompleteness {
            key,
            first_complete_q: complete.first_complete_q,
            publication_position: complete.publication_position,
            physical_chain: complete.blocks,
            valid_proof_locations,
            body_locations,
        }))
    };
    match check() {
        Ok(outcome) => outcome,
        Err(StepError::Wait(missing)) => CompletenessOutcome::Wait(missing),
        Err(StepError::Reject(error)) => CompletenessOutcome::Reject(error),
    }
}

pub(crate) struct Candidate {
    pub(crate) key: ResultKey,
    pub(crate) blocks: Vec<MergedBlock>,
}

impl PublicationIndex {
    pub(crate) fn index(&mut self, block: &FinalizedBlock) -> Result<(), GfrError> {
        for (index, msg) in block.proposal.messages.iter().enumerate() {
            let location = EspressoPosition {
                q: block.proposal.q,
                message_index: u32::try_from(index).map_err(|_| DomainError::Overflow)?,
            };
            if msg.codec != PUBLICATION_CODEC {
                self.ignored
                    .insert(location, PublicationDiagnostic::UnknownCodec(msg.codec));
                continue;
            }
            let object = match decode_publication(&msg.bytes) {
                Ok(object) => object,
                Err(error) => {
                    self.ignored
                        .insert(location, PublicationDiagnostic::Malformed(error));
                    continue;
                }
            };
            match object {
                PublicObject::Body(block) => {
                    let entry =
                        self.bodies
                            .entry(block.digest()?)
                            .or_insert_with(|| BodyEvidence {
                                payload: Arc::from(msg.bytes.as_slice()),
                                block: Arc::new(block),
                                locations: Vec::new(),
                            });
                    if entry.payload.as_ref() != msg.bytes {
                        return Err(GfrError::InvalidSource);
                    }
                    entry.locations.push(location);
                }
                PublicObject::FinalProof {
                    descriptor,
                    target,
                    proof,
                } => {
                    let key = ResultKey {
                        cid: descriptor.cid,
                        block_hash: target,
                    };
                    let result = self.results.entry(key).or_default();
                    let entry = result
                        .proofs
                        .entry(Arc::from(msg.bytes.as_slice()))
                        .or_insert_with(|| ProofEvidence {
                            descriptor: Arc::new(descriptor),
                            proof: Arc::from(proof),
                            locations: Vec::new(),
                            verdict: ProofVerdict::Unchecked,
                        });
                    entry.locations.push(location);
                }
            }
        }
        Ok(())
    }
}

pub(crate) struct Evaluation<'a> {
    pub(crate) espresso: &'a [FinalizedBlock],
    pub(crate) reads: &'a ReplayReads<'a>,
    pub(crate) services: &'a ReplayServices<'a>,
    pub(crate) used_origin: &'a Cell<Option<u64>>,
}

impl Evaluation<'_> {
    pub(crate) fn candidates(
        &self,
        state: &mut PublicReplayState,
        descriptor: &Descriptor,
        anchor: &ExecutionAnchor,
        head: u64,
    ) -> StepResult<Vec<Candidate>> {
        let metadata = CheckedMetadata {
            regional: RegionalMetadata::new(&state.config, self.reads.l1)?,
            supplied: self.services.metadata,
            used_origin: self.used_origin,
        };
        let end = session_end_key(&state.config, state.parameters, &descriptor.plan.window)?;
        let first = ResultKey {
            cid: descriptor.cid,
            block_hash: Hash32::new([0; 32]),
        };
        let last = ResultKey {
            cid: descriptor.cid,
            block_hash: Hash32::new([255; 32]),
        };
        let bodies = &state.publications.bodies;
        let mut verified_path = Vec::new();
        let mut candidates = Vec::new();
        for (&key, evidence) in state.publications.results.range_mut(first..=last) {
            if evidence.exclusion.is_some() {
                continue;
            }
            let CompletePath {
                blocks,
                digests,
                first_complete_q: q_avail,
                ..
            } = match evaluate_completeness(
                bodies,
                evidence,
                key,
                descriptor,
                &state.config,
                self.services.finality.as_ref(),
            )? {
                EvaluationOutcome::Incomplete
                | EvaluationOutcome::Invalid(CompletenessInvalid::FinalProof) => continue,
                EvaluationOutcome::Invalid(CompletenessInvalid::Ancestry) => {
                    evidence.exclusion = Some(ResultExclusion::InvalidAncestry);
                    continue;
                }
                EvaluationOutcome::Complete(complete) => complete,
            };
            let target = blocks.last().ok_or(GfrError::InvalidSource)?;
            evidence.q_avail = Some(q_avail);
            evidence.physical_chain.clone_from(&digests);
            let q_block = self
                .espresso
                .get(u64_index(q_avail)?)
                .ok_or(GfrError::InvalidSource)?;
            if q_block.proposal.theta_us.0 >= end {
                evidence.exclusion = Some(ResultExclusion::PublicCutoff);
                continue;
            }
            if evidence.merge_verified && target.metadata.height <= head {
                check_compatible(&mut verified_path, &digests, key)?;
                continue;
            }
            let output = match merge(
                anchor,
                &blocks,
                &descriptor.plan.window,
                &metadata,
                self.services.dep,
                &state.config,
            ) {
                InputStatus::Ready(output) => output,
                InputStatus::Wait(missing) => return Err(StepError::Wait(input_missing(missing))),
                InputStatus::Reject(
                    error @ InputError::Malformed(DomainError::Malformed { .. }),
                ) => {
                    evidence.exclusion = Some(ResultExclusion::InvalidExecution(error));
                    continue;
                }
                InputStatus::Reject(error) => return Err(StepError::Reject(error.into())),
            };
            evidence.merge_verified = true;
            check_compatible(&mut verified_path, &digests, key)?;
            candidates.push(Candidate {
                key,
                blocks: output.blocks,
            });
        }
        Ok(candidates)
    }
}

struct CompletePath {
    blocks: Vec<PhysicalBlock>,
    digests: Vec<Hash32>,
    first_complete_q: u64,
    publication_position: EspressoPosition,
}

enum EvaluationOutcome {
    Incomplete,
    Invalid(CompletenessInvalid),
    Complete(CompletePath),
}

// Both callers index checked public messages themselves. Only replay carries
// verdicts forward, inside its sealed state under a fixed verification profile.
fn evaluate_completeness(
    bodies: &BTreeMap<Hash32, BodyEvidence>,
    evidence: &mut ResultEvidence,
    key: ResultKey,
    descriptor: &Descriptor,
    config: &PublicConfig,
    finality: Option<&VerifierBinding<'_>>,
) -> StepResult<EvaluationOutcome> {
    for proof in evidence.proofs.values_mut() {
        if *proof.descriptor != *descriptor {
            proof.verdict = ProofVerdict::DescriptorMismatch;
        }
    }
    if !evidence
        .proofs
        .values()
        .any(|proof| matches!(proof.verdict, ProofVerdict::Unchecked | ProofVerdict::Valid))
    {
        // Invalid verdicts can only have been determined after a complete path.
        return Ok(
            if evidence
                .proofs
                .values()
                .any(|p| p.verdict == ProofVerdict::Invalid)
            {
                EvaluationOutcome::Invalid(CompletenessInvalid::FinalProof)
            } else {
                EvaluationOutcome::Incomplete
            },
        );
    }
    let (blocks, digests, first_body_q) =
        match ancestry(bodies, key.block_hash, descriptor, config)? {
            BodyPath::Missing => return Ok(EvaluationOutcome::Incomplete),
            BodyPath::Invalid => {
                return Ok(EvaluationOutcome::Invalid(CompletenessInvalid::Ancestry));
            }
            BodyPath::Complete {
                blocks,
                digests,
                first_q,
            } => (blocks, digests, first_q),
        };
    let target = blocks.last().ok_or(GfrError::InvalidSource)?;
    verify_proofs(evidence, descriptor, target, finality)?;
    let Some(publication_position) = valid_proof_positions(evidence).min() else {
        return Ok(EvaluationOutcome::Invalid(CompletenessInvalid::FinalProof));
    };
    Ok(EvaluationOutcome::Complete(CompletePath {
        blocks,
        digests,
        first_complete_q: first_body_q.max(publication_position.q),
        publication_position,
    }))
}

fn valid_proof_positions(evidence: &ResultEvidence) -> impl Iterator<Item = EspressoPosition> + '_ {
    evidence
        .proofs
        .values()
        .filter(|proof| proof.verdict == ProofVerdict::Valid)
        .flat_map(|proof| proof.locations.iter().copied())
}

fn verify_proofs(
    evidence: &mut ResultEvidence,
    descriptor: &Descriptor,
    target: &PhysicalBlock,
    finality: Option<&VerifierBinding<'_>>,
) -> StepResult<()> {
    let unchecked = evidence
        .proofs
        .values_mut()
        .filter(|proof| proof.verdict == ProofVerdict::Unchecked);
    let mut unchecked = unchecked.peekable();
    if unchecked.peek().is_none() {
        return Ok(());
    }
    let binding = finality.ok_or(StepError::Wait(Missing::FinalVerifierUnavailable))?;
    match binding.basis.require(descriptor, target) {
        InputStatus::Ready(()) => {}
        InputStatus::Wait(missing) => return Err(StepError::Wait(input_missing(missing))),
        InputStatus::Reject(error) => return Err(StepError::Reject(error.into())),
    }
    for proof in unchecked {
        proof.verdict = if binding.verifier.check(descriptor, target, &proof.proof) {
            ProofVerdict::Valid
        } else {
            ProofVerdict::Invalid
        };
    }
    Ok(())
}

fn check_compatible(
    longest: &mut Vec<Hash32>,
    path: &[Hash32],
    key: ResultKey,
) -> Result<(), GfrError> {
    if !path.starts_with(longest) && !longest.starts_with(path) {
        return Err(GfrError::InconsistentFinality(key));
    }
    if path.len() > longest.len() {
        *longest = path.to_vec();
    }
    Ok(())
}

enum BodyPath {
    Missing,
    Invalid,
    Complete {
        blocks: Vec<PhysicalBlock>,
        digests: Vec<Hash32>,
        first_q: u64,
    },
}

fn ancestry(
    bodies: &BTreeMap<Hash32, BodyEvidence>,
    target: Hash32,
    descriptor: &Descriptor,
    config: &crate::PublicConfig,
) -> Result<BodyPath, GfrError> {
    let anchor = &descriptor.anchor;
    let anchor_id = anchor.block.block_id;
    if target == anchor_id {
        return Ok(BodyPath::Invalid);
    }
    let window = &descriptor.plan.window;
    let mut digest = target;
    let mut later_height = None;
    let mut blocks = Vec::new();
    let mut digests = Vec::new();
    let mut first_q = 0;
    while digest != anchor_id {
        let Some(body) = bodies.get(&digest) else {
            return Ok(BodyPath::Missing);
        };
        let block = &body.block;
        let height = block.metadata.height;
        if !(window.h_min..=window.h_max).contains(&height)
            || later_height.is_some_and(|later| height >= later)
            || (height == window.h_min && block.parent != anchor_id)
            || block.metadata.timestamp_us != l2_timestamp_us(config, height)?
            || validate_transactions(&block.transactions, config.byte_limit as usize).is_err()
        {
            return Ok(BodyPath::Invalid);
        }
        first_q = first_q.max(body.locations[0].q);
        later_height = Some(height);
        digests.push(digest);
        blocks.push(block.as_ref().clone());
        digest = block.parent;
    }
    digests.reverse();
    blocks.reverse();
    Ok(BodyPath::Complete {
        blocks,
        digests,
        first_q,
    })
}

struct CheckedMetadata<'a> {
    regional: RegionalMetadata<'a>,
    supplied: &'a dyn MetadataProvider,
    used_origin: &'a Cell<Option<u64>>,
}

impl MetadataProvider for CheckedMetadata<'_> {
    fn config_ref(&self) -> ConfigRef {
        self.regional.config_ref()
    }

    fn at(&self, anchor: &Anchor, height: u64) -> InputStatus<Metadata> {
        let expected = match self.regional.at(anchor, height) {
            InputStatus::Ready(metadata) => metadata,
            status => return status,
        };
        self.used_origin.set(Some(
            self.used_origin
                .get()
                .unwrap_or(0)
                .max(anchor.block.origin)
                .max(expected.origin),
        ));
        match self.supplied.at(anchor, height) {
            InputStatus::Ready(actual) if expected != actual => {
                InputStatus::Reject(InputError::InvalidSource)
            }
            status => status,
        }
    }
}

/// Check the whole newly appended interval against its current public acceptance key.
pub fn interval_unexpired(
    config: &crate::PublicConfig,
    parameters: crate::GfrParameters,
    first: u64,
    last: u64,
    acceptance_key: u64,
) -> Result<bool, GfrError> {
    let length = last
        .checked_sub(first)
        .ok_or(DomainError::Malformed {
            reason: "reversed expiry interval",
        })?
        .checked_add(1)
        .ok_or(DomainError::Overflow)?;
    let length = u64_index(length)?;
    if length > MAX_FIXTURE_ROWS {
        return Err(DomainError::LimitExceeded {
            limit: MAX_FIXTURE_ROWS,
            actual: length,
        }
        .into());
    }
    for height in first..=last {
        if acceptance_key >= expiry_key(config, parameters, height)? {
            return Ok(false);
        }
    }
    Ok(true)
}
