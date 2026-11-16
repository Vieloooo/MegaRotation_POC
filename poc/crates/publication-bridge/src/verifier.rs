use std::marker::PhantomData;

use poc_core::{
    fixture::DepProvider,
    merge::merge,
    metadata::{RegionalMetadata, l2_timestamp_us},
};
use poc_publisher::{
    MissingInput, OfferVerdict, PublicationTask, PublicationVerifier, RecordVerdict, RecordedAt,
    RejectReason,
};
use poc_types::{
    domain::{
        AnchorEvidenceSource, Canonical, CanonicalObject, ConfigRef, ExecutionAnchor,
        FinalizedL1Reader, InputStatus, ProgramConfig, PublicConfig, RegionalResult, ResultKey,
        authorization_ref, regional_cid, validate_transactions,
    },
    hash_parts,
};
use poc_viewer_gfr::{
    CompletenessInvalid, CompletenessOutcome, GfrError, GfrParameters, Missing,
    VerificationProfile, VerifierBinding, public_completeness,
};
use sim_espresso::{LogMessage, PublicTimeUs, VerifiedPrefix, validate_message_list};

use crate::{BridgeError, BridgeResult, publication_messages};

const MISSING_VERIFIER: u32 = 1;
const MISSING_BASIS: u32 = 2;
const MISSING_CONTEXT: u32 = 3;
const MISSING_EXECUTION: u32 = 4;
const REJECT_CONFIG: u32 = 101;
const REJECT_ENCODING: u32 = 102;
const REJECT_CONTEXT: u32 = 103;
const REJECT_PROOF: u32 = 104;
const REJECT_EXECUTION: u32 = 105;
const REJECT_SOURCE: u32 = 106;

/// Validates historical Plan/configuration and exact execution-anchor provenance.
pub trait ContextVerifier {
    fn verify_context(
        &self,
        config: &PublicConfig,
        result: &RegionalResult,
    ) -> InputStatus<ExecutionAnchor>;
}

pub struct BridgeInputs<'a> {
    pub finality: Option<VerifierBinding<'a>>,
    pub context: &'a dyn ContextVerifier,
    pub l1: &'a dyn FinalizedL1Reader,
    pub dep: &'a dyn DepProvider,
}

pub struct NativePublicationVerifier<'a> {
    config: PublicConfig,
    config_ref: ConfigRef,
    expected: VerificationProfile,
    parameters: GfrParameters,
    source_id: Vec<u8>,
    profile_digest: Vec<u8>,
    inputs: PhantomData<&'a ()>,
}

impl<'a> NativePublicationVerifier<'a> {
    pub fn new(
        config: PublicConfig,
        expected: VerificationProfile,
        parameters: GfrParameters,
        source_id: Vec<u8>,
    ) -> Result<Self, BridgeError> {
        let VerificationProfile::Native {
            source_revision, ..
        } = &expected
        else {
            return Err(BridgeError::NonNativeProfile);
        };
        let config_ref = config.reference().map_err(|_| BridgeError::InvalidConfig)?;
        expected.validate(&config)?;
        if parameters.announcement_us == 0 || parameters.grace_us == 0 || parameters.expiry_us == 0
        {
            return Err(BridgeError::InvalidConfig);
        }
        validate_message_list(&source_id, &[]).map_err(|_| BridgeError::SourceMismatch)?;
        let profile_digest = hash_parts(
            "PoC/H3NativeProfile/v1",
            &[config_ref.as_bytes(), source_revision.as_bytes()],
        );
        Ok(Self {
            config,
            config_ref,
            expected,
            parameters,
            source_id,
            profile_digest,
            inputs: PhantomData,
        })
    }

    fn binding_matches(&self, binding: Option<&VerifierBinding<'_>>) -> Result<(), BridgeError> {
        if let Some(binding) = binding {
            binding.profile.validate(&self.config)?;
            if binding.profile != self.expected {
                return Err(GfrError::VerificationProfileMismatch.into());
            }
        }
        Ok(())
    }

    fn verify_subject(&self, result: &RegionalResult) -> Result<(), BridgeError> {
        let descriptor = &result.descriptor;
        let plan = &descriptor.plan;
        let program = ProgramConfig::from_plan(&self.config, plan)?;
        if (plan.leader_rule.rule_id == 1 && plan.authorization_ref
            != authorization_ref(self.config_ref, self.config.registry_hash, &plan.input())?)
            || descriptor.cid
                != regional_cid(&plan.session, descriptor.anchor.block.block_id, &program)?
        {
            return Err(BridgeError::InvalidResult);
        }
        if let AnchorEvidenceSource::Genesis(reference) = result.anchor_evidence.source
            && (reference != self.config_ref || descriptor.anchor != self.config.genesis)
        {
            return Err(BridgeError::InvalidResult);
        }
        Ok(())
    }

    fn audit_reference(&self, key: ResultKey, detail: &[u8]) -> Vec<u8> {
        let mut reference = b"H3EV\0\x01".to_vec();
        reference.extend_from_slice(&self.config_ref.encode());
        reference.extend_from_slice(&key.encode());
        reference.extend_from_slice(&self.profile_digest);
        // Source IDs have already passed the shared 06 limit (1..=256).
        reference.extend_from_slice(&(self.source_id.len() as u16).to_be_bytes());
        reference.extend_from_slice(&self.source_id);
        reference.extend_from_slice(detail);
        reference
    }

    fn missing(&self, key: ResultKey, code: u32, detail: impl std::fmt::Debug) -> MissingInput {
        MissingInput {
            code,
            reference: self.audit_reference(key, format!("{detail:?}").as_bytes()),
        }
    }

    fn reject(&self, key: ResultKey, code: u32, error: impl std::fmt::Debug) -> RejectReason {
        RejectReason {
            code,
            reference: self.audit_reference(key, format!("{error:?}").as_bytes()),
        }
    }

    fn check_offer(
        &self,
        item: &BridgeResult,
        inputs: &BridgeInputs<'a>,
    ) -> Result<VerifiedOffer, OfferProblem> {
        let key = item.key();
        let result = item.result();
        let descriptor = &result.descriptor;
        let item_digest = hash_parts("PoC/H3StoredResult/v1", &[item.store_bytes()]);
        let reject = |code, error| self.reject(key, code, (&item_digest, error));
        let missing_input = |code, missing| self.missing(key, code, (&item_digest, missing));
        if descriptor.plan.config_ref != self.config_ref
            || inputs.dep.config_ref() != self.config_ref
        {
            return Err(reject(REJECT_CONFIG, BridgeError::InvalidConfig).into());
        }
        self.binding_matches(inputs.finality.as_ref())
            .map_err(|error| reject(REJECT_CONFIG, error))?;
        self.verify_subject(result)
            .map_err(|error| reject(REJECT_CONTEXT, error))?;
        let missing_ancestry = item
            .missing_ancestry()
            .map_err(|error| reject(REJECT_EXECUTION, error))?;
        for block in std::iter::once(&result.block).chain(&result.chain) {
            let timestamp = l2_timestamp_us(&self.config, block.metadata.height)
                .map_err(|error| reject(REJECT_EXECUTION, error.into()))?;
            if block.metadata.timestamp_us != timestamp {
                return Err(reject(REJECT_EXECUTION, BridgeError::InvalidResult).into());
            }
            validate_transactions(&block.transactions, self.config.byte_limit as usize)
                .map_err(|error| reject(REJECT_EXECUTION, error.into()))?;
        }
        let deadline = descriptor
            .plan
            .window
            .h_max
            .checked_add(1)
            .and_then(|height| l2_timestamp_us(&self.config, height).ok())
            .and_then(|timestamp| timestamp.checked_add(self.parameters.grace_us))
            .map(PublicTimeUs)
            .ok_or_else(|| reject(REJECT_CONFIG, BridgeError::InvalidConfig))?;

        let anchor = match inputs.context.verify_context(&self.config, result) {
            InputStatus::Ready(anchor) => anchor,
            InputStatus::Wait(missing) => {
                return Err(OfferProblem::Wait(
                    missing_input(MISSING_CONTEXT, missing),
                    None,
                ));
            }
            InputStatus::Reject(error) => {
                return Err(reject(REJECT_CONTEXT, BridgeError::Input(error)).into());
            }
        };
        if anchor.anchor != descriptor.anchor {
            return Err(reject(REJECT_CONTEXT, BridgeError::InvalidResult).into());
        }
        let anchor_bytes = anchor
            .encode_checked()
            .map_err(|error| reject(REJECT_CONTEXT, error.into()))?;
        if anchor.anchor == self.config.genesis
            && (anchor.native_hash != self.config.genesis_native_hash
                || anchor
                    .state
                    .root()
                    .map_err(|error| reject(REJECT_CONTEXT, error.into()))?
                    != self.config.genesis_state_root)
        {
            return Err(reject(REJECT_CONTEXT, BridgeError::InvalidResult).into());
        }

        let wait = |missing| OfferProblem::Wait(missing, Some(deadline));
        let finality = inputs.finality.as_ref().ok_or_else(|| {
            wait(self.missing(
                key,
                MISSING_VERIFIER,
                (&item_digest, Missing::FinalVerifierUnavailable),
            ))
        })?;
        if result.final_proof.is_empty() {
            return Err(wait(self.missing(
                key,
                MISSING_BASIS,
                (&item_digest, "missing final proof"),
            )));
        }
        match finality.basis.require(descriptor, &result.block) {
            InputStatus::Ready(()) => {}
            InputStatus::Wait(missing) => {
                return Err(wait(missing_input(MISSING_BASIS, missing)));
            }
            InputStatus::Reject(error) => {
                return Err(reject(REJECT_PROOF, BridgeError::Input(error)).into());
            }
        }
        if !finality
            .verifier
            .check(descriptor, &result.block, &result.final_proof)
        {
            return Err(reject(REJECT_PROOF, BridgeError::InvalidResult).into());
        }
        if let Some(missing) = missing_ancestry {
            return Err(wait(missing_input(MISSING_EXECUTION, missing)));
        }

        let metadata = RegionalMetadata::new(&self.config, inputs.l1)
            .map_err(|error| reject(REJECT_CONFIG, error.into()))?;
        let output = match merge(
            &anchor,
            &result.chain,
            &descriptor.plan.window,
            &metadata,
            inputs.dep,
            &self.config,
        ) {
            InputStatus::Ready(output) => output,
            InputStatus::Wait(missing) => {
                return Err(wait(missing_input(MISSING_EXECUTION, missing)));
            }
            InputStatus::Reject(error) => {
                return Err(reject(REJECT_EXECUTION, BridgeError::Input(error)).into());
            }
        };
        let tip = output
            .blocks
            .last()
            .ok_or_else(|| reject(REJECT_EXECUTION, BridgeError::InvalidResult))?;
        if tip.physical_digest != Some(key.block_hash) {
            return Err(reject(REJECT_EXECUTION, BridgeError::InvalidResult).into());
        }
        let messages = publication_messages(item, &self.source_id)
            .map_err(|error| reject(REJECT_ENCODING, error))?;
        let basis_hash = hash_parts(
            "PoC/H3OfferBasis/v1",
            &[&item_digest, &anchor_bytes, tip.native_hash.as_bytes()],
        );
        Ok(VerifiedOffer {
            messages,
            basis: self.audit_reference(key, &basis_hash),
            deadline,
        })
    }
}

impl<'a> PublicationVerifier for NativePublicationVerifier<'a> {
    type Key = ResultKey;
    type Result = BridgeResult;
    type PublicInputs = BridgeInputs<'a>;

    fn key(&self, result: &BridgeResult) -> Result<ResultKey, RejectReason> {
        Ok(result.key())
    }

    fn verify_offer(
        &self,
        result: BridgeResult,
        inputs: &BridgeInputs<'a>,
    ) -> OfferVerdict<ResultKey, BridgeResult> {
        let key = result.key();
        match self.check_offer(&result, inputs) {
            Ok(verified) => OfferVerdict::Verified {
                key,
                complete_result: result,
                publication_messages: verified.messages,
                validation_basis: verified.basis,
                deadline: verified.deadline,
            },
            Err(OfferProblem::Wait(missing, verified_deadline)) => OfferVerdict::Wait {
                key,
                partial_result: result,
                missing: vec![missing],
                verified_deadline,
            },
            Err(OfferProblem::Reject(reason)) => OfferVerdict::Reject(reason),
        }
    }

    fn verify_recorded(
        &self,
        task: &PublicationTask<ResultKey, BridgeResult>,
        prefix: &VerifiedPrefix<'_>,
        inputs: &BridgeInputs<'a>,
    ) -> RecordVerdict<ResultKey> {
        let key = task.key;
        if prefix.source_id() != self.source_id {
            return RecordVerdict::Reject(self.reject(
                key,
                REJECT_SOURCE,
                (BridgeError::SourceMismatch, prefix.source_id()),
            ));
        }
        let result = task.result.result();
        if task.result.key() != key {
            return RecordVerdict::Reject(self.reject(
                key,
                REJECT_ENCODING,
                BridgeError::InvalidResult,
            ));
        }
        let through_q = prefix.tip().0;
        match public_completeness(
            prefix,
            through_q,
            &self.config,
            &self.expected,
            &result.descriptor,
            key.block_hash,
            inputs.finality.as_ref(),
        ) {
            CompletenessOutcome::Incomplete
            | CompletenessOutcome::Invalid(CompletenessInvalid::FinalProof) => {
                RecordVerdict::NotYetPublic
            }
            CompletenessOutcome::Wait(missing) => {
                let code = if missing == Missing::FinalVerifierUnavailable {
                    MISSING_VERIFIER
                } else {
                    MISSING_BASIS
                };
                RecordVerdict::Wait {
                    missing: vec![self.missing(
                        key,
                        code,
                        (prefix.reader_id(), through_q, missing),
                    )],
                }
            }
            CompletenessOutcome::Invalid(reason) => RecordVerdict::Reject(self.reject(
                key,
                REJECT_EXECUTION,
                (prefix.reader_id(), through_q, reason),
            )),
            CompletenessOutcome::Reject(error) => {
                let code = match error {
                    GfrError::ConfigMismatch
                    | GfrError::UnsupportedVerificationProfile
                    | GfrError::VerificationProfileMismatch => REJECT_CONFIG,
                    GfrError::InvalidSource
                    | GfrError::EspressoSource(_)
                    | GfrError::HistoricalBound => REJECT_SOURCE,
                    GfrError::Input(_) => REJECT_PROOF,
                    _ => REJECT_EXECUTION,
                };
                RecordVerdict::Reject(self.reject(
                    key,
                    code,
                    (prefix.reader_id(), through_q, BridgeError::Gfr(error)),
                ))
            }
            CompletenessOutcome::Complete(complete) => {
                let position = complete.publication_position;
                let Some(proof_message) = prefix
                    .read(position.q)
                    .and_then(|block| block.proposal.messages.get(position.message_index as usize))
                else {
                    return RecordVerdict::Reject(self.reject(
                        key,
                        REJECT_SOURCE,
                        BridgeError::SourceMismatch,
                    ));
                };
                let basis_hash = hash_parts(
                    "PoC/H3ReceiptBasis/v1",
                    &[
                        prefix.reader_id(),
                        &position.q.to_be_bytes(),
                        &position.message_index.to_be_bytes(),
                        &complete.first_complete_q.to_be_bytes(),
                        &through_q.to_be_bytes(),
                        &proof_message.bytes,
                    ],
                );
                RecordVerdict::Recorded {
                    key,
                    receipt: RecordedAt {
                        publication_position: position,
                        complete_through_q: through_q,
                        source_id: self.source_id.clone(),
                        verification_basis: self.audit_reference(key, &basis_hash),
                    },
                }
            }
        }
    }
}

struct VerifiedOffer {
    messages: Vec<LogMessage>,
    basis: Vec<u8>,
    deadline: PublicTimeUs,
}

enum OfferProblem {
    Wait(MissingInput, Option<PublicTimeUs>),
    Reject(RejectReason),
}

impl From<RejectReason> for OfferProblem {
    fn from(reason: RejectReason) -> Self {
        Self::Reject(reason)
    }
}
