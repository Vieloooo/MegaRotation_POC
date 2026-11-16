use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use poc_publisher::{AttemptId, Submission};
use poc_types::domain::{Canonical, Hash32, ResultKey};
use poc_viewer_gfr::{PUBLICATION_CODEC, PublicObject, decode_publication, encode_publication};
use sim_espresso::{
    FinalizedBlock, LogMessage, MAX_SOURCE_BYTES, encode_finalized, validate_message_list,
};
use sim_transport::Id;

use crate::{
    BridgeError, DurableCommitPort, attempt_payload_id,
    bounded::{Writer, digest, identity},
};

const MAX_PENDING: usize = 8;
const MAX_RETAINED: usize = 1024 * 1024;
const MAX_MESSAGES: usize = 11;
const MAX_BODY: usize = 197;
const MAX_FINAL: usize = 49_199;

pub trait SubmissionVerifier {
    fn verify_submission(
        &self,
        source_id: &[u8],
        submission: &Submission<ResultKey>,
    ) -> Result<(), BridgeError>;
}

#[derive(Clone, Debug)]
pub struct CustodyBinding {
    pub run: Id,
    pub config: Id,
    pub source_id: Vec<u8>,
    pub owner_reference: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduledMessage {
    pub stable_id: AttemptId,
    pub ordinal: u32,
    pub repeat: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CustodyOperation {
    AlreadyCommitted,
    Admit {
        stable_id: AttemptId,
        payload_id: Hash32,
    },
    Propose {
        intent_id: Hash32,
        q: u64,
        messages: Vec<LogMessage>,
    },
    Finalize {
        q: u64,
        witness: Hash32,
    },
}

pub struct AttemptCustody {
    binding: CustodyBinding,
    instance: Arc<()>,
    state: State,
}

pub struct CustodyPlan {
    instance: Arc<()>,
    base_revision: u64,
    state: State,
    operation: CustodyOperation,
    bytes: Vec<u8>,
    hash: Hash32,
}

type Key = (Vec<u8>, u64, u64);

#[derive(Clone, Default)]
struct State {
    revision: u64,
    attempts: BTreeMap<Key, Attempt>,
    proposals: BTreeMap<Hash32, Proposal>,
    positions: BTreeMap<u64, Hash32>,
}

#[derive(Clone)]
struct Attempt {
    submission: Submission<ResultKey>,
    payload_id: Hash32,
    ordinals: Vec<Ordinal>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ordinal {
    Pending,
    Proposed(u64, u32),
    Finalized(u64, u32, Hash32),
}

#[derive(Clone)]
struct Proposal {
    q: u64,
    selection: Vec<ScheduledMessage>,
    finalized: Option<Hash32>,
}

impl AttemptCustody {
    pub fn new(binding: CustodyBinding) -> Result<Self, BridgeError> {
        identity(&binding.source_id, MAX_SOURCE_BYTES)?;
        identity(&binding.owner_reference, 4096)?;
        Ok(Self {
            binding,
            instance: Arc::new(()),
            state: State::default(),
        })
    }

    pub fn plan_admit(
        &self,
        submission: Submission<ResultKey>,
        owner: &dyn SubmissionVerifier,
    ) -> Result<CustodyPlan, BridgeError> {
        let source = &self.binding.source_id;
        validate_attempt_limits(source, &submission)?;
        owner.verify_submission(source, &submission)?;
        let key = attempt_key(&submission.stable_id);
        if let Some(prior) = self.state.attempts.get(&key) {
            if prior.submission != submission {
                return Err(BridgeError::InvalidResult);
            }
            return self.plan(self.state.clone(), CustodyOperation::AlreadyCommitted);
        }
        validate_publication(&submission)?;
        if self
            .state
            .attempts
            .values()
            .filter(|row| {
                row.ordinals
                    .iter()
                    .any(|item| !matches!(item, Ordinal::Finalized(..)))
            })
            .count()
            >= MAX_PENDING
        {
            return Err(BridgeError::Capacity);
        }
        let payload_id = attempt_payload_id(source, &submission.exact_payload)?;
        let operation = CustodyOperation::Admit {
            stable_id: submission.stable_id.clone(),
            payload_id,
        };
        let ordinals = vec![Ordinal::Pending; submission.exact_payload.len()];
        let mut state = self.state.clone();
        state.attempts.insert(
            key,
            Attempt {
                submission,
                payload_id,
                ordinals,
            },
        );
        state.advance()?;
        self.plan(state, operation)
    }

    pub fn plan_propose(
        &self,
        intent_id: Hash32,
        q: u64,
        selection: &[ScheduledMessage],
    ) -> Result<CustodyPlan, BridgeError> {
        if selection.len() > sim_espresso::MAX_MESSAGES {
            return Err(BridgeError::Capacity);
        }
        for item in selection {
            identity(&item.stable_id.namespace, MAX_SOURCE_BYTES)?;
        }
        if let Some(prior) = self.state.proposals.get(&intent_id) {
            if prior.q != q || prior.selection != selection {
                return Err(BridgeError::InvalidResult);
            }
            return self.plan(self.state.clone(), CustodyOperation::AlreadyCommitted);
        }
        if self.state.positions.contains_key(&q) {
            return Err(BridgeError::InvalidResult);
        }
        let mut unique = BTreeSet::new();
        let mut state = self.state.clone();
        for (index, item) in selection.iter().enumerate() {
            let key = attempt_key(&item.stable_id);
            if !unique.insert((key.clone(), item.ordinal)) {
                return Err(BridgeError::InvalidResult);
            }
            let ordinal = state
                .attempts
                .get_mut(&key)
                .and_then(|attempt| attempt.ordinals.get_mut(item.ordinal as usize))
                .ok_or(BridgeError::InvalidResult)?;
            match (item.repeat, *ordinal) {
                (false, Ordinal::Pending) => *ordinal = Ordinal::Proposed(q, index as u32),
                (true, Ordinal::Finalized(..)) => {}
                _ => return Err(BridgeError::InvalidResult),
            }
        }
        let messages = state.messages(selection)?;
        validate_message_list(&self.binding.source_id, &messages)
            .map_err(|_| BridgeError::Capacity)?;
        state.proposals.insert(
            intent_id,
            Proposal {
                q,
                selection: selection.to_vec(),
                finalized: None,
            },
        );
        state.positions.insert(q, intent_id);
        state.advance()?;
        self.plan(
            state,
            CustodyOperation::Propose {
                intent_id,
                q,
                messages,
            },
        )
    }

    /// Validate exact finalization bytes against the committed proposal. The
    /// returned plan still requires a matching controlled owner receipt.
    pub fn plan_finalize(&self, block: &FinalizedBlock) -> Result<CustodyPlan, BridgeError> {
        let q = block.proposal.q;
        let intent = self
            .state
            .positions
            .get(&q)
            .ok_or(BridgeError::InvalidResult)?;
        let proposal = &self.state.proposals[intent];
        let messages = self.state.messages(&proposal.selection)?;
        if block.proposal.messages != messages {
            return Err(BridgeError::InvalidResult);
        }
        let bytes = encode_finalized(&self.binding.source_id, block)
            .map_err(|_| BridgeError::InvalidEncoding)?;
        let witness = digest("H3/CustodyFinalized/v1", &[&bytes]);
        if let Some(prior) = proposal.finalized {
            if prior != witness {
                return Err(BridgeError::InvalidResult);
            }
            return self.plan(self.state.clone(), CustodyOperation::AlreadyCommitted);
        }
        let mut state = self.state.clone();
        for (index, item) in proposal.selection.iter().enumerate() {
            if !item.repeat {
                let ordinal = state
                    .attempts
                    .get_mut(&attempt_key(&item.stable_id))
                    .and_then(|attempt| attempt.ordinals.get_mut(item.ordinal as usize))
                    .ok_or(BridgeError::InvalidResult)?;
                if *ordinal != Ordinal::Proposed(q, index as u32) {
                    return Err(BridgeError::InvalidResult);
                }
                *ordinal = Ordinal::Finalized(q, index as u32, witness);
            }
        }
        state
            .proposals
            .get_mut(intent)
            .ok_or(BridgeError::InvalidResult)?
            .finalized = Some(witness);
        state.advance()?;
        self.plan(state, CustodyOperation::Finalize { q, witness })
    }

    pub fn confirm(
        &mut self,
        plan: CustodyPlan,
        receipt: &[u8],
        owner: &dyn DurableCommitPort,
    ) -> Result<(), BridgeError> {
        if !Arc::ptr_eq(&self.instance, &plan.instance) || plan.base_revision != self.state.revision
        {
            return Err(BridgeError::SourceMismatch);
        }
        owner.verify_commit(plan.hash, receipt)?;
        self.state = plan.state;
        Ok(())
    }

    fn plan(&self, state: State, operation: CustodyOperation) -> Result<CustodyPlan, BridgeError> {
        let mut out = Writer::new(MAX_RETAINED);
        out.push(b"H3CS\0\x01")?;
        out.push(&self.binding.run)?;
        out.push(&self.binding.config)?;
        out.frame(&self.binding.source_id)?;
        out.frame(&self.binding.owner_reference)?;
        out.push(&state.revision.to_be_bytes())?;
        out.count(state.attempts.len())?;
        for item in state.attempts.values() {
            let attempt = &item.submission;
            write_id(&mut out, &attempt.stable_id)?;
            out.frame(&attempt.key.encode())?;
            out.push(&attempt.created_tick.0.to_be_bytes())?;
            out.push(item.payload_id.as_bytes())?;
            out.count(attempt.exact_payload.len())?;
            for (message, ordinal) in attempt.exact_payload.iter().zip(&item.ordinals) {
                out.push(&message.codec.to_be_bytes())?;
                out.frame(&message.bytes)?;
                match ordinal {
                    Ordinal::Pending => out.tag(0)?,
                    Ordinal::Proposed(q, index) | Ordinal::Finalized(q, index, _) => {
                        out.tag(if matches!(ordinal, Ordinal::Proposed(..)) {
                            1
                        } else {
                            2
                        })?;
                        out.push(&q.to_be_bytes())?;
                        out.push(&index.to_be_bytes())?;
                        if let Ordinal::Finalized(_, _, witness) = ordinal {
                            out.push(witness.as_bytes())?;
                        }
                    }
                }
            }
        }
        out.count(state.proposals.len())?;
        for (intent_id, proposal) in &state.proposals {
            out.push(intent_id.as_bytes())?;
            out.push(&proposal.q.to_be_bytes())?;
            out.count(proposal.selection.len())?;
            for item in &proposal.selection {
                write_id(&mut out, &item.stable_id)?;
                out.push(&item.ordinal.to_be_bytes())?;
                out.tag(u8::from(item.repeat))?;
            }
            out.tag(u8::from(proposal.finalized.is_some()))?;
            if let Some(witness) = proposal.finalized {
                out.push(witness.as_bytes())?;
            }
        }
        let hash = digest("H3/CustodySnapshot/v1", &[&out.bytes]);
        Ok(CustodyPlan {
            instance: Arc::clone(&self.instance),
            base_revision: self.state.revision,
            state,
            operation,
            bytes: out.bytes,
            hash,
        })
    }
}

impl CustodyPlan {
    pub fn operation(&self) -> &CustodyOperation {
        &self.operation
    }
    pub fn candidate_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn candidate_hash(&self) -> Hash32 {
        self.hash
    }
}

impl State {
    fn advance(&mut self) -> Result<(), BridgeError> {
        self.revision = self.revision.checked_add(1).ok_or(BridgeError::Capacity)?;
        Ok(())
    }

    fn messages(&self, selection: &[ScheduledMessage]) -> Result<Vec<LogMessage>, BridgeError> {
        let rows = selection
            .iter()
            .map(|item| {
                self.attempts
                    .get(&attempt_key(&item.stable_id))
                    .and_then(|attempt| attempt.submission.exact_payload.get(item.ordinal as usize))
                    .ok_or(BridgeError::InvalidResult)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut size = 0usize;
        for message in &rows {
            size = size
                .checked_add(8 + message.bytes.len())
                .ok_or(BridgeError::Capacity)?;
            if size > sim_espresso::MAX_BLOCK_BYTES {
                return Err(BridgeError::Capacity);
            }
        }
        Ok(rows.into_iter().cloned().collect())
    }
}

fn attempt_key(id: &AttemptId) -> Key {
    (id.namespace.clone(), id.task_number, id.attempt_number)
}

fn write_id(out: &mut Writer, id: &AttemptId) -> Result<(), BridgeError> {
    out.frame(&id.namespace)?;
    out.push(&id.task_number.to_be_bytes())?;
    out.push(&id.attempt_number.to_be_bytes())?;
    Ok(())
}

pub(crate) fn validate_attempt_limits(
    source_id: &[u8],
    submission: &Submission<ResultKey>,
) -> Result<(), BridgeError> {
    let id = &submission.stable_id;
    identity(source_id, MAX_SOURCE_BYTES)?;
    identity(&id.namespace, MAX_SOURCE_BYTES)?;
    let messages = &submission.exact_payload;
    if id.task_number == 0 || id.attempt_number == 0 || messages.is_empty() {
        return Err(BridgeError::InvalidResult);
    }
    if messages.len() > MAX_MESSAGES {
        return Err(BridgeError::Capacity);
    }
    for message in messages.iter() {
        if message.codec != PUBLICATION_CODEC || message.bytes.is_empty() {
            return Err(BridgeError::InvalidResult);
        }
    }
    validate_message_list(source_id, messages).map_err(|_| BridgeError::Capacity)
}

fn validate_publication(submission: &Submission<ResultKey>) -> Result<(), BridgeError> {
    let (proof, bodies) = submission
        .exact_payload
        .split_last()
        .ok_or(BridgeError::InvalidResult)?;
    if bodies.is_empty() {
        return Err(BridgeError::InvalidResult);
    }
    if proof.bytes.len() > MAX_FINAL {
        return Err(BridgeError::Capacity);
    }
    let proof_value = decode_publication(&proof.bytes)?;
    let PublicObject::FinalProof {
        descriptor,
        target,
        proof: exact_proof,
    } = &proof_value
    else {
        return Err(BridgeError::InvalidResult);
    };
    if descriptor.cid != submission.key.cid
        || *target != submission.key.block_hash
        || exact_proof.is_empty()
        || encode_publication(&proof_value)? != proof.bytes
    {
        return Err(BridgeError::InvalidResult);
    }
    let mut parent = descriptor.anchor.block.block_id;
    let mut height = descriptor.anchor.block.height;
    for message in bodies {
        if message.bytes.len() > MAX_BODY {
            return Err(BridgeError::Capacity);
        }
        let value = decode_publication(&message.bytes)?;
        let PublicObject::Body(block) = &value else {
            return Err(BridgeError::InvalidResult);
        };
        if block.parent != parent
            || block.metadata.height <= height
            || block.transactions.len() > 4
            || !(descriptor.plan.window.h_min..=descriptor.plan.window.h_max)
                .contains(&block.metadata.height)
            || encode_publication(&value)? != message.bytes
        {
            return Err(BridgeError::InvalidResult);
        }
        parent = block.digest()?;
        height = block.metadata.height;
    }
    if parent != *target {
        return Err(BridgeError::InvalidResult);
    }
    Ok(())
}
