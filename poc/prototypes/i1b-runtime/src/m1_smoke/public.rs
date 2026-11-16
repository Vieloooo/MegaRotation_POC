use super::{M1Result, check, checked, profile::*, worker::durable_file};
use poc_core::{fixture::CheckedSetup, metadata::RegionalMetadata};
use poc_publication_bridge::{
    AttemptCustody, BridgeError, BridgeInputs, BridgeResult, BridgeStateCodec, ContextVerifier,
    CustodyBinding, CustodyOperation, DurableCommitPort, NativePublicationVerifier, RecordDelivery,
    ScheduledMessage, StopLedger, StopLedgerBinding, SubmissionVerifier,
};
use poc_publisher::{ConfirmOutcome, Publisher, StoreBinding, Submission};
use poc_types::domain::*;
use poc_viewer_gfr::{
    self as gfr, FixtureL1Reader, GlobalSource, InitInput, ReplayReads, ReplayServices,
    ReplayState, ReplayStatus, Tail, VerificationProfile, VerifierBinding,
};
use serde_json::{Value, json};
use sim_espresso::{FinalizedLogReader, ReaderState, SimEspresso, TickUs, TimestampPolicy};
use std::{
    fs,
    path::{Path, PathBuf},
};

const READER: &[u8] = b"M1/gfr-independent-reader";
const OWNER: &[u8] = b"M1/durable-public-service";
const PUBLISHER: &[u8] = b"M1/healthy/publisher";

pub(crate) struct Public {
    root: PathBuf,
    pub inputs: PublicInputs,
    pub finality: NativeFinality,
    setup: CheckedSetup,
    l1: FixtureL1Reader,
    reader: ReaderState,
    espresso: SimEspresso,
    pub replay: ReplayState,
    verification: VerificationProfile,
    store_binding: StoreBinding,
    custody: AttemptCustody,
    stops: StopLedger,
    pub next_q: u64,
    next_commit: u64,
}

pub(crate) struct Submitted {
    pub q: u64,
    pub result: BridgeResult,
    pub submission: Submission<ResultKey>,
    pub dispatch_ref: Vec<u8>,
}

struct DurableReceipt(PathBuf, Vec<u8>);
impl DurableCommitPort for DurableReceipt {
    fn verify_commit(&self, candidate_hash: Hash32, receipt: &[u8]) -> Result<(), BridgeError> {
        let bytes = fs::read(&self.0).map_err(|_| BridgeError::SourceMismatch)?;
        if receipt != candidate_hash.as_bytes()
            || bytes != self.1
            || bytes.get(..32) != Some(candidate_hash.as_bytes())
        {
            return Err(BridgeError::SourceMismatch);
        }
        Ok(())
    }
}

struct ExactSubmission<'a>(&'a Submission<ResultKey>);
impl SubmissionVerifier for ExactSubmission<'_> {
    fn verify_submission(
        &self,
        source_id: &[u8],
        submission: &Submission<ResultKey>,
    ) -> Result<(), BridgeError> {
        if source_id != SOURCE || submission != self.0 {
            return Err(BridgeError::SourceMismatch);
        }
        Ok(())
    }
}

struct Context<'a> {
    replay: &'a ReplayState,
    finality: &'a NativeFinality,
}
impl ContextVerifier for Context<'_> {
    fn verify_context(
        &self,
        config: &PublicConfig,
        result: &RegionalResult,
    ) -> InputStatus<ExecutionAnchor> {
        let descriptor = &result.descriptor;
        let public = self.replay.public_state();
        let session = &descriptor.plan.session;
        let known = public.selected.get(&(session.beta, session.r));
        let reference = if session.r == 0 {
            &config.genesis
        } else {
            match public.selected.get(&(session.beta, session.r - 1)) {
                Some(prior) => &prior.anchor,
                None => return InputStatus::Reject(InputError::InvalidSource),
            }
        };
        if known != Some(descriptor)
            || !matches!(
                self.finality
                    .0
                    .validate_plan(&descriptor.plan, &Reference(reference)),
                poc_authorization::PlanOutcome::Ready(_)
            )
        {
            return InputStatus::Reject(InputError::InvalidSource);
        }
        let Some((anchor, source)) = public_anchor(self.replay, descriptor) else {
            return InputStatus::Reject(InputError::InvalidSource);
        };
        if result.anchor_evidence.source != source
            || result.anchor_evidence.source_objects.as_slice()
                != [anchor
                    .encode_checked()
                    .expect("committed GFR anchor encoding")]
        {
            return InputStatus::Reject(InputError::InvalidSource);
        }
        InputStatus::Ready(anchor)
    }
}

fn public_anchor(
    state: &ReplayState,
    descriptor: &Descriptor,
) -> Option<(ExecutionAnchor, AnchorEvidenceSource)> {
    let view = state.public_state();
    if descriptor.anchor == view.genesis.anchor {
        return Some((
            view.genesis.clone(),
            AnchorEvidenceSource::Genesis(descriptor.plan.config_ref),
        ));
    }
    let entry = view.entries.iter().find(|entry| {
        entry.block.state.metadata == descriptor.anchor.metadata()
            && entry.block.physical_digest == Some(descriptor.anchor.block.block_id)
    })?;
    let GlobalSource::Regional(key) = entry.source;
    Some((
        ExecutionAnchor {
            anchor: descriptor.anchor.clone(),
            state: entry.block.state.clone(),
            native_hash: entry.block.native_hash,
        },
        AnchorEvidenceSource::Regional(key),
    ))
}

impl Public {
    pub fn new(root: &Path) -> M1Result<Self> {
        let inputs = PublicInputs::fixture()?;
        let setup = inputs.setup()?;
        let finality = NativeFinality(inputs.plans()?);
        let config = &inputs.config;
        let mut l1 = checked(FixtureL1Reader::new(
            L1_SOURCE.to_vec(),
            setup.l1().reader(),
        ))?;
        inputs.deliver_l1(&setup, &mut l1, START_US)?;
        let reader = checked(ReaderState::new(SOURCE.to_vec(), READER.to_vec()))?;
        let espresso = checked(SimEspresso::new(
            SOURCE.to_vec(),
            TimestampPolicy::Offset {
                offset_us: 0,
                quantum_us: 1,
            },
        ))?;
        let verification = VerificationProfile::Native {
            protocol_version: 1,
            verifier_version: 1,
            source_revision: std::env::var("M1_CANDIDATE")
                .unwrap_or_else(|_| "commonware-2026.10.0".into()),
        };
        let metadata = RegionalMetadata::new(config, &l1)?;
        let init = checked(gfr::init(
            &InitInput::Genesis {
                config: config.clone(),
                genesis: inputs.genesis.clone(),
                parameters: parameters(),
                expected_verification: verification.clone(),
            },
            &ReplayReads {
                espresso: &reader,
                l1: &l1,
            },
            &ReplayServices {
                config,
                plan: &finality.0,
                metadata: &metadata,
                dep: setup.dep(),
                finality: Some(VerifierBinding {
                    verifier: &finality,
                    basis: &finality,
                    profile: verification.clone(),
                }),
            },
        ))?;
        let replay = init
            .state
            .ok_or_else(|| super::problem(format!("M1 genesis {:?}", init.status)))?;
        let initial = initial_descriptor(&replay)?;
        let native = crate::native_config::Resolved::normal(&finality.0, initial)?;
        let store_binding = StoreBinding {
            namespace: PUBLISHER.to_vec(),
            configuration: config.reference()?.encode(),
            source_id: SOURCE.to_vec(),
            reader_id: READER.to_vec(),
        };
        let publisher = checked(NativePublicationVerifier::new(
            config.clone(),
            verification.clone(),
            parameters(),
            SOURCE.to_vec(),
        ))?;
        checked(Publisher::create(
            root.join("publisher"),
            store_binding.clone(),
            publisher,
            BridgeStateCodec,
        ))?;
        let custody = checked(AttemptCustody::new(CustodyBinding {
            run: native.run_id,
            config: native.config_hash,
            source_id: SOURCE.to_vec(),
            owner_reference: OWNER.to_vec(),
        }))?;
        let stops = checked(StopLedger::new(StopLedgerBinding {
            config: config.clone(),
            expected: verification.clone(),
            parameters: parameters(),
            espresso_source: SOURCE.to_vec(),
            l1_source: L1_SOURCE.to_vec(),
            publisher_reader: READER.to_vec(),
            owner_reference: OWNER.to_vec(),
        }))?;
        for (name, bytes) in [
            ("config.bin", config.encode_checked()?),
            ("registry.bin", inputs.registry.encode_checked()?),
            ("genesis.bin", inputs.genesis.encode_checked()?),
            ("dep.bin", inputs.dep.encode_checked()?),
            ("l1.bin", inputs.l1.encode_checked()?),
        ] {
            durable_file(&root.join(name), &bytes)?;
        }
        fs::create_dir(root.join("espresso"))?;
        fs::create_dir(root.join("service-commits"))?;
        Ok(Self {
            root: root.to_owned(),
            inputs,
            finality,
            setup,
            l1,
            reader,
            espresso,
            replay,
            verification,
            store_binding,
            custody,
            stops,
            next_q: 0,
            next_commit: 0,
        })
    }

    pub fn install(
        &self,
        descriptor: Descriptor,
        history: Vec<AnchorStep>,
        now: u64,
    ) -> M1Result<Install> {
        let (execution, _) = public_anchor(&self.replay, &descriptor)
            .ok_or_else(|| super::problem("Missing GFR execution anchor"))?;
        Ok(Install {
            descriptor,
            execution,
            history,
            prepared_at_us: now,
        })
    }

    pub fn result(
        &self,
        descriptor: Descriptor,
        chain: Vec<PhysicalBlock>,
        final_proof: Vec<u8>,
    ) -> M1Result<BridgeResult> {
        let block = chain
            .last()
            .ok_or_else(|| super::problem("empty delivered chain"))?
            .clone();
        let (anchor, source) = public_anchor(&self.replay, &descriptor)
            .ok_or_else(|| super::problem("missing Publisher anchor"))?;
        checked(BridgeResult::new(RegionalResult {
            descriptor,
            block,
            chain,
            final_proof,
            anchor_evidence: AnchorEvidence {
                source,
                source_objects: vec![anchor.encode_checked()?],
            },
        }))
    }

    fn persist(
        root: &Path,
        sequence: &mut u64,
        label: &str,
        hash: Hash32,
        bytes: &[u8],
    ) -> M1Result<DurableReceipt> {
        *sequence += 1;
        let path = root
            .join("service-commits")
            .join(format!("{sequence:04}-{label}.bin"));
        let bytes = [hash.as_bytes().as_slice(), bytes].concat();
        durable_file(&path, &bytes)?;
        Ok(DurableReceipt(path, bytes))
    }

    pub fn propose(&mut self, result: BridgeResult, tick: u64) -> M1Result<Submitted> {
        check(
            checked(self.stops.permit_dispatch(&result))?,
            "closed public scope cannot dispatch",
        )?;
        self.inputs.deliver_l1(&self.setup, &mut self.l1, tick)?;
        let mut publisher = checked(Publisher::open(
            self.root.join("publisher"),
            self.store_binding.clone(),
            checked(NativePublicationVerifier::new(
                self.inputs.config.clone(),
                self.verification.clone(),
                parameters(),
                SOURCE.to_vec(),
            ))?,
            BridgeStateCodec,
        ))?;
        let context = Context {
            replay: &self.replay,
            finality: &self.finality,
        };
        let key = result.key();
        let inputs = BridgeInputs {
            finality: Some(VerifierBinding {
                verifier: &self.finality,
                basis: &self.finality,
                profile: self.verification.clone(),
            }),
            context: &context,
            l1: &self.l1,
            dep: self.setup.dep(),
        };
        checked(publisher.offer(result.clone(), &inputs))?;
        let submission = checked(publisher.prepare_attempt(&key, TickUs(tick)))?;
        let plan = checked(
            self.custody
                .plan_admit(submission.clone(), &ExactSubmission(&submission)),
        )?;
        let hash = plan.candidate_hash();
        let receipt = Self::persist(
            &self.root,
            &mut self.next_commit,
            "admit",
            hash,
            plan.candidate_bytes(),
        )?;
        checked(self.custody.confirm(plan, hash.as_bytes(), &receipt))?;
        let dispatch_ref = hash.as_bytes().to_vec();
        checked(publisher.mark_dispatched(&submission.stable_id, dispatch_ref.clone()))?;
        let q = self.next_q;
        self.next_q += 1;
        let intent = Hash32::new(crate::hash_parts(
            b"PoC/M1/public-propose/v1",
            &[&q.to_be_bytes(), &dispatch_ref],
        ));
        let schedule = (0..submission.exact_payload.len())
            .map(|index| ScheduledMessage {
                stable_id: submission.stable_id.clone(),
                ordinal: index as u32,
                repeat: false,
            })
            .collect::<Vec<_>>();
        let plan = checked(self.custody.plan_propose(intent, q, &schedule))?;
        let CustodyOperation::Propose { messages, .. } = plan.operation() else {
            return Err(super::problem("missing fresh custody propose"));
        };
        let messages = messages.clone();
        check(
            messages.as_slice() == submission.exact_payload.as_ref(),
            "proposal changed Publisher bytes",
        )?;
        let hash = plan.candidate_hash();
        let receipt = Self::persist(
            &self.root,
            &mut self.next_commit,
            "propose",
            hash,
            plan.candidate_bytes(),
        )?;
        checked(self.custody.confirm(plan, hash.as_bytes(), &receipt))?;
        let proposal = checked(self.espresso.propose(TickUs(tick), true, messages))?;
        check(proposal.q == q, "Espresso proposal sequence mismatch")?;
        Ok(Submitted {
            q,
            result,
            submission,
            dispatch_ref,
        })
    }

    pub fn finalize(&mut self, submitted: Submitted, tick: u64) -> M1Result<Value> {
        let block = checked(self.espresso.finalize(submitted.q, TickUs(tick)))?.clone();
        let plan = checked(self.custody.plan_finalize(&block))?;
        let hash = plan.candidate_hash();
        let receipt = Self::persist(
            &self.root,
            &mut self.next_commit,
            "finalize",
            hash,
            plan.candidate_bytes(),
        )?;
        checked(self.custody.confirm(plan, hash.as_bytes(), &receipt))?;
        let bytes = checked(sim_espresso::encode_finalized(SOURCE, &block))?;
        durable_file(
            &self
                .root
                .join("espresso")
                .join(format!("{:04}.bin", submitted.q)),
            &bytes,
        )?;
        checked(
            self.reader
                .receive_block(SOURCE, block.clone(), TickUs(tick)),
        )?;
        self.inputs.deliver_l1(&self.setup, &mut self.l1, tick)?;
        let mut publisher = checked(Publisher::open(
            self.root.join("publisher"),
            self.store_binding.clone(),
            checked(NativePublicationVerifier::new(
                self.inputs.config.clone(),
                self.verification.clone(),
                parameters(),
                SOURCE.to_vec(),
            ))?,
            BridgeStateCodec,
        ))?;
        checked(publisher.mark_service_acknowledged(
            &submitted.submission.stable_id,
            &submitted.dispatch_ref,
            &block.proposal.messages,
            hash.as_bytes().to_vec(),
        ))?;
        let context = Context {
            replay: &self.replay,
            finality: &self.finality,
        };
        let inputs = BridgeInputs {
            finality: Some(VerifierBinding {
                verifier: &self.finality,
                basis: &self.finality,
                profile: self.verification.clone(),
            }),
            context: &context,
            l1: &self.l1,
            dep: self.setup.dep(),
        };
        let key = submitted.result.key();
        let prefix = checked(self.reader.require_prefix_through(submitted.q))?;
        let status = checked(publisher.confirm_recorded(&key, &prefix, &inputs))?;
        let ConfirmOutcome::Recorded { receipt, .. } = status else {
            return Err(super::problem(format!(
                "M1 Publisher not Recorded: {status:?}"
            )));
        };
        Ok(
            json!({"q": submitted.q, "q_avail": receipt.complete_through_q,
            "message_index": receipt.publication_position.message_index,
            "theta_us": block.proposal.theta_us.0, "cid": key.cid, "physical_id": key.block_hash,
            "height": submitted.result.result().block.metadata.height,
            "proposal_tick_us": block.proposal.proposed_at_us.0,
            "service_receipt": hash, "dispatch_ref": submitted.dispatch_ref}),
        )
    }

    pub fn advance(&mut self) -> M1Result<(usize, Vec<(u64, RecordDelivery)>)> {
        let q = self
            .next_q
            .checked_sub(1)
            .ok_or_else(|| super::problem("no finalized Espresso rows"))?;
        let prefix = checked(self.reader.require_prefix_through(q))?;
        let row = prefix
            .read(q)
            .ok_or_else(|| super::problem("missing Espresso replay bound"))?;
        let bound = ReplayPosition {
            key_us: row.proposal.theta_us.0,
            kind: ReplayKind::Espresso,
            index: ReplayIndex::Espresso(q),
        };
        let old_entries = self.replay.public_state().entries.len();
        let old_records = self.replay.public_state().records.len();
        let metadata = RegionalMetadata::new(&self.inputs.config, &self.l1)?;
        let result = gfr::advance_global(
            &self.replay,
            &ReplayReads {
                espresso: &self.reader,
                l1: &self.l1,
            },
            &ReplayServices {
                config: &self.inputs.config,
                plan: &self.finality.0,
                metadata: &metadata,
                dep: self.setup.dep(),
                finality: Some(VerifierBinding {
                    verifier: &self.finality,
                    basis: &self.finality,
                    profile: self.verification.clone(),
                }),
            },
            Some(&bound),
        );
        check(
            result.status == ReplayStatus::Ready,
            format!("M1 replay failed: {:?}", result.status),
        )?;
        self.replay = result.state;
        let mut records = Vec::new();
        for index in old_records..self.replay.public_state().records.len() {
            let record = checked(RecordDelivery::from_replay(&self.replay, index))?;
            let plan = checked(self.stops.plan_record(&record))?;
            let hash = plan.candidate_hash();
            let receipt = Self::persist(
                &self.root,
                &mut self.next_commit,
                "stop-record",
                hash,
                plan.candidate_bytes(),
            )?;
            checked(self.stops.confirm(plan, hash.as_bytes(), &receipt))?;
            records.push((index as u64, record));
        }
        Ok((old_entries, records))
    }
}

pub(crate) fn initial_descriptor(state: &ReplayState) -> M1Result<&Descriptor> {
    match &state.public_state().tail {
        Tail::Next(descriptor) => Ok(descriptor),
        other => Err(super::problem(format!(
            "M1 missing selected descriptor: {other:?}"
        ))),
    }
}
