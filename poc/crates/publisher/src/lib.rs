//! Durable Publisher tasks and explicit controlled-output intents.
#![forbid(unsafe_code)]

mod codec;
mod store;

use std::{collections::BTreeMap, path::Path, sync::Arc};

use sim_espresso::{EspressoPosition, LogMessage, PublicTimeUs, TickUs, VerifiedPrefix};

pub use store::{CommitStage, StoreError};

/// The integration supplies the frozen canonical key and result encodings.
pub trait StateCodec<K, R> {
    fn encode_key(&self, key: &K) -> Vec<u8>;
    fn decode_key(&self, bytes: &[u8]) -> Result<K, CodecError>;
    fn encode_result(&self, result: &R) -> Vec<u8>;
    fn decode_result(&self, bytes: &[u8]) -> Result<R, CodecError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodecError {
    InvalidKey,
    InvalidResult,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissingInput {
    pub code: u32,
    pub reference: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectReason {
    pub code: u32,
    pub reference: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OfferVerdict<K, R> {
    Verified {
        key: K,
        complete_result: R,
        publication_messages: Vec<LogMessage>,
        validation_basis: Vec<u8>,
        deadline: PublicTimeUs,
    },
    Wait {
        key: K,
        partial_result: R,
        missing: Vec<MissingInput>,
        verified_deadline: Option<PublicTimeUs>,
    },
    Reject(RejectReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordVerdict<K> {
    Recorded { key: K, receipt: RecordedAt },
    Wait { missing: Vec<MissingInput> },
    NotYetPublic,
    Reject(RejectReason),
}

/// Production integrations check final evidence, exact anchor provenance, body
/// identity and physical ancestry here. The component owns the durable transitions.
pub trait PublicationVerifier {
    type Key: Clone + Ord;
    type Result: Clone;
    type PublicInputs: ?Sized;

    fn key(&self, result: &Self::Result) -> Result<Self::Key, RejectReason>;
    fn verify_offer(
        &self,
        result: Self::Result,
        inputs: &Self::PublicInputs,
    ) -> OfferVerdict<Self::Key, Self::Result>;
    fn verify_recorded(
        &self,
        task: &PublicationTask<Self::Key, Self::Result>,
        prefix: &VerifiedPrefix<'_>,
        inputs: &Self::PublicInputs,
    ) -> RecordVerdict<Self::Key>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoreBinding {
    /// Stable per-publisher identity in the configured run. Shared by all reopens.
    pub namespace: Vec<u8>,
    pub configuration: Vec<u8>,
    pub source_id: Vec<u8>,
    pub reader_id: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Verifying,
    Queued,
    Submitted,
    Recorded,
    Stopped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedAt {
    pub publication_position: EspressoPosition,
    pub complete_through_q: u64,
    pub source_id: Vec<u8>,
    pub verification_basis: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleReason {
    SessionClosed,
    BranchClosed,
    OtherCidAssigned,
}

/// Trusted GFR adapter projection with the exact affected task keys.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StopDecision<K> {
    pub affected_keys: Vec<K>,
    pub reason: LifecycleReason,
    /// Encoded source identity, replay position and scope, supplied by the adapter.
    pub public_basis: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PublicStop {
    Lifecycle {
        reason: LifecycleReason,
        public_basis: Vec<u8>,
    },
    Deadline {
        q: u64,
        theta: PublicTimeUs,
        deadline: PublicTimeUs,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptId {
    pub namespace: Vec<u8>,
    pub task_number: u64,
    pub attempt_number: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttemptState {
    Prepared,
    Dispatched {
        controlled_delivery_ref: Vec<u8>,
    },
    ServiceAcknowledged {
        controlled_delivery_ref: Vec<u8>,
        service_ref: Vec<u8>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attempt {
    pub stable_id: AttemptId,
    pub number: u64,
    pub exact_payload: Arc<[LogMessage]>,
    pub created_tick: TickUs,
    pub state: AttemptState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Submission<K> {
    pub key: K,
    pub stable_id: AttemptId,
    pub exact_payload: Arc<[LogMessage]>,
    pub created_tick: TickUs,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicationTask<K, R> {
    pub key: K,
    pub result: R,
    pub task_number: u64,
    pub missing: Vec<MissingInput>,
    pub verified_deadline: Option<PublicTimeUs>,
    pub verified_payload: Option<Arc<[LogMessage]>>,
    pub validation_basis: Option<Vec<u8>>,
    pub attempts: Vec<Attempt>,
    pub recorded_at: Option<RecordedAt>,
    pub stop_reason: Option<PublicStop>,
}

impl<K, R> PublicationTask<K, R> {
    pub fn phase(&self) -> Phase {
        if self.recorded_at.is_some() {
            Phase::Recorded
        } else if self.stop_reason.is_some() {
            Phase::Stopped
        } else if self.verified_payload.is_none() {
            Phase::Verifying
        } else if self
            .attempts
            .first()
            .is_some_and(|first| first.state != AttemptState::Prepared)
        {
            Phase::Submitted
        } else {
            Phase::Queued
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OfferOutcome<K> {
    pub key: K,
    pub phase: Phase,
    pub missing: Vec<MissingInput>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfirmOutcome<K> {
    Recorded { key: K, receipt: RecordedAt },
    AlreadyRecorded,
    Wait { missing: Vec<MissingInput> },
    NotYetPublic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WatermarkOutcome {
    Applied,
    AlreadyStopped,
    BelowDeadline,
    DeadlineUnknown,
}

#[derive(Debug)]
pub enum PublisherError {
    Store(StoreError),
    Rejected(RejectReason),
    InvalidVerifierOutput,
    UnknownTask,
    SubmissionUnavailable(Phase),
    UnknownAttempt,
    DispatchMismatch,
    AcknowledgementMismatch,
    InvalidBasis,
    PrefixIdentityMismatch,
    InvalidReceipt,
    TickRegressed,
}

impl From<StoreError> for PublisherError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

impl std::fmt::Display for PublisherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for PublisherError {}

#[derive(Clone)]
pub(crate) struct Snapshot<K, R> {
    generation: u64,
    tasks: BTreeMap<K, PublicationTask<K, R>>,
}

type Task<V> = PublicationTask<<V as PublicationVerifier>::Key, <V as PublicationVerifier>::Result>;

pub struct Publisher<V: PublicationVerifier, C> {
    verifier: V,
    codec: C,
    binding: StoreBinding,
    store: store::FileStore,
    snapshot: Snapshot<V::Key, V::Result>,
}

impl<V, C> Publisher<V, C>
where
    V: PublicationVerifier,
    C: StateCodec<V::Key, V::Result>,
{
    /// Creates an empty store at a new directory. Existing directories are recovery inputs.
    pub fn create(
        dir: impl AsRef<Path>,
        binding: StoreBinding,
        verifier: V,
        codec: C,
    ) -> Result<Self, PublisherError> {
        codec::validate_binding(&binding)?;
        let mut store = store::FileStore::create(dir.as_ref())?;
        let snapshot = Snapshot {
            generation: 0,
            tasks: BTreeMap::new(),
        };
        store.commit(&codec::encode_snapshot(&codec, &binding, &snapshot)?)?;
        Ok(Self {
            verifier,
            codec,
            binding,
            store,
            snapshot,
        })
    }

    /// Opens only the committed main snapshot and checks its key/result bindings.
    pub fn open(
        dir: impl AsRef<Path>,
        binding: StoreBinding,
        verifier: V,
        codec: C,
    ) -> Result<Self, PublisherError> {
        codec::validate_binding(&binding)?;
        let (store, bytes) = store::FileStore::open(dir.as_ref())?;
        let snapshot = codec::decode_snapshot(&codec, &binding, &bytes)?;
        for task in snapshot.tasks.values() {
            if verifier.key(&task.result).ok().as_ref() != Some(&task.key) {
                return Err(StoreError::InvalidState("task result key").into());
            }
        }
        Ok(Self {
            verifier,
            codec,
            binding,
            store,
            snapshot,
        })
    }

    pub fn generation(&self) -> u64 {
        self.snapshot.generation
    }

    pub fn task(&self, key: &V::Key) -> Option<&Task<V>> {
        self.snapshot.tasks.get(key)
    }

    pub fn tasks(&self) -> impl Iterator<Item = &Task<V>> {
        self.snapshot.tasks.values()
    }

    pub fn offer(
        &mut self,
        result: V::Result,
        inputs: &V::PublicInputs,
    ) -> Result<OfferOutcome<V::Key>, PublisherError> {
        self.store.check_healthy()?;
        let key = self
            .verifier
            .key(&result)
            .map_err(PublisherError::Rejected)?;
        let verdict = self.verifier.verify_offer(result, inputs);
        let (result, missing, payload, basis, deadline) = match verdict {
            OfferVerdict::Reject(reason) => return Err(PublisherError::Rejected(reason)),
            OfferVerdict::Wait {
                key: observed,
                partial_result,
                missing,
                verified_deadline,
            } => {
                if observed != key || missing.is_empty() {
                    return Err(PublisherError::InvalidVerifierOutput);
                }
                (partial_result, missing, None, None, verified_deadline)
            }
            OfferVerdict::Verified {
                key: observed,
                complete_result,
                publication_messages,
                validation_basis,
                deadline,
            } => {
                if observed != key || publication_messages.is_empty() || validation_basis.is_empty()
                {
                    return Err(PublisherError::InvalidVerifierOutput);
                }
                codec::validate_payload(&self.binding.source_id, &publication_messages)
                    .map_err(|_| PublisherError::InvalidVerifierOutput)?;
                (
                    complete_result,
                    Vec::new(),
                    Some(Arc::from(publication_messages)),
                    Some(validation_basis),
                    Some(deadline),
                )
            }
        };
        if self.verifier.key(&result).ok().as_ref() != Some(&key) {
            return Err(PublisherError::InvalidVerifierOutput);
        }
        let existing = self.task(&key);
        let prior_deadline = existing.and_then(|task| task.verified_deadline);
        if let (Some(prior), Some(observed)) = (prior_deadline, deadline)
            && prior != observed
        {
            return Err(PublisherError::InvalidVerifierOutput);
        }
        if let Some(task) = existing.filter(|task| task.verified_payload.is_some()) {
            return Ok(offer_outcome(task));
        }
        if let Some(task) = existing.filter(|task| {
            task.missing == missing
                && task.verified_deadline == deadline.or(prior_deadline)
                && self.codec.encode_result(&task.result) == self.codec.encode_result(&result)
        }) {
            return Ok(offer_outcome(task));
        }

        let mut snapshot = self.snapshot.clone();
        let task_number = existing.map_or(snapshot.tasks.len() as u64 + 1, |task| task.task_number);
        let stop_reason = existing.and_then(|task| task.stop_reason.clone());
        let task = PublicationTask {
            key: key.clone(),
            result,
            task_number,
            missing,
            verified_deadline: deadline.or(prior_deadline),
            verified_payload: payload,
            validation_basis: basis,
            attempts: Vec::new(),
            recorded_at: None,
            stop_reason,
        };
        let output = offer_outcome(&task);
        snapshot.tasks.insert(key, task);
        self.commit(snapshot)?;
        Ok(output)
    }

    /// Returns a pending output only after its intent is durably committed.
    pub fn prepare_attempt(
        &mut self,
        key: &V::Key,
        tick: TickUs,
    ) -> Result<Submission<V::Key>, PublisherError> {
        self.store.check_healthy()?;
        let task = self.task(key).ok_or(PublisherError::UnknownTask)?;
        if !matches!(task.phase(), Phase::Queued | Phase::Submitted) {
            return Err(PublisherError::SubmissionUnavailable(task.phase()));
        }
        if let Some(attempt) = task.attempts.last() {
            if attempt.state == AttemptState::Prepared {
                return Ok(submission(key, attempt));
            }
            if tick < attempt.created_tick {
                return Err(PublisherError::TickRegressed);
            }
        }
        let number = task.attempts.len() as u64 + 1;
        let attempt = Attempt {
            stable_id: AttemptId {
                namespace: self.binding.namespace.clone(),
                task_number: task.task_number,
                attempt_number: number,
            },
            number,
            exact_payload: task.verified_payload.clone().expect("queued or submitted"),
            created_tick: tick,
            state: AttemptState::Prepared,
        };
        let output = submission(key, &attempt);
        let mut snapshot = self.snapshot.clone();
        snapshot
            .tasks
            .get_mut(key)
            .expect("known key")
            .attempts
            .push(attempt);
        self.commit(snapshot)?;
        Ok(output)
    }

    /// Confirms a controlled commit of an existing submission intent.
    pub fn mark_dispatched(
        &mut self,
        id: &AttemptId,
        controlled_delivery_ref: Vec<u8>,
    ) -> Result<(), PublisherError> {
        self.store.check_healthy()?;
        validate_basis(&controlled_delivery_ref)?;
        let (key, index, attempt) = self.attempt(id)?;
        match &attempt.state {
            AttemptState::Prepared => self.set_attempt(
                key.clone(),
                index,
                AttemptState::Dispatched {
                    controlled_delivery_ref,
                },
            ),
            AttemptState::Dispatched {
                controlled_delivery_ref: first,
            }
            | AttemptState::ServiceAcknowledged {
                controlled_delivery_ref: first,
                ..
            } => {
                if *first != controlled_delivery_ref {
                    return Err(PublisherError::DispatchMismatch);
                }
                Ok(())
            }
        }
    }

    pub fn mark_service_acknowledged(
        &mut self,
        id: &AttemptId,
        controlled_ref: &[u8],
        payload: &[LogMessage],
        service_ref: Vec<u8>,
    ) -> Result<(), PublisherError> {
        self.store.check_healthy()?;
        validate_basis(&service_ref)?;
        let (key, index, attempt) = self.attempt(id)?;
        if attempt.exact_payload.as_ref() != payload {
            return Err(PublisherError::AcknowledgementMismatch);
        }
        match &attempt.state {
            AttemptState::Dispatched {
                controlled_delivery_ref,
            } if controlled_delivery_ref == controlled_ref => self.set_attempt(
                key.clone(),
                index,
                AttemptState::ServiceAcknowledged {
                    controlled_delivery_ref: controlled_ref.to_vec(),
                    service_ref,
                },
            ),
            AttemptState::ServiceAcknowledged {
                controlled_delivery_ref,
                service_ref: first,
            } if controlled_delivery_ref == controlled_ref && *first == service_ref => Ok(()),
            _ => Err(PublisherError::AcknowledgementMismatch),
        }
    }

    pub fn confirm_recorded(
        &mut self,
        key: &V::Key,
        prefix: &VerifiedPrefix<'_>,
        inputs: &V::PublicInputs,
    ) -> Result<ConfirmOutcome<V::Key>, PublisherError> {
        self.store.check_healthy()?;
        self.check_prefix(prefix)?;
        let task = self.task(key).ok_or(PublisherError::UnknownTask)?;
        if task.recorded_at.is_some() {
            return Ok(ConfirmOutcome::AlreadyRecorded);
        }
        if task.verified_payload.is_none() {
            return Ok(ConfirmOutcome::Wait {
                missing: task.missing.clone(),
            });
        }
        match self.verifier.verify_recorded(task, prefix, inputs) {
            RecordVerdict::Wait { missing } => {
                if missing.is_empty() {
                    return Err(PublisherError::InvalidVerifierOutput);
                }
                Ok(ConfirmOutcome::Wait { missing })
            }
            RecordVerdict::NotYetPublic => Ok(ConfirmOutcome::NotYetPublic),
            RecordVerdict::Reject(reason) => Err(PublisherError::Rejected(reason)),
            RecordVerdict::Recorded {
                key: observed,
                receipt,
            } => {
                let position = receipt.publication_position;
                if &observed != key
                    || receipt.source_id != self.binding.source_id
                    || receipt.verification_basis.is_empty()
                    || position.q > receipt.complete_through_q
                    || receipt.complete_through_q > prefix.tip().0
                    || prefix
                        .read(position.q)
                        .and_then(|block| {
                            block.proposal.messages.get(position.message_index as usize)
                        })
                        .is_none()
                {
                    return Err(PublisherError::InvalidReceipt);
                }
                let mut snapshot = self.snapshot.clone();
                snapshot.tasks.get_mut(key).expect("known key").recorded_at = Some(receipt.clone());
                self.commit(snapshot)?;
                Ok(ConfirmOutcome::Recorded {
                    key: key.clone(),
                    receipt,
                })
            }
        }
    }

    /// Applies one verified lifecycle projection atomically to existing affected tasks.
    pub fn apply_public_stop(
        &mut self,
        decision: StopDecision<V::Key>,
    ) -> Result<usize, PublisherError> {
        self.store.check_healthy()?;
        validate_basis(&decision.public_basis)?;
        let mut snapshot = self.snapshot.clone();
        let mut changed = 0;
        for key in decision.affected_keys {
            if let Some(task) = snapshot.tasks.get_mut(&key)
                && task.stop_reason.is_none()
            {
                task.stop_reason = Some(PublicStop::Lifecycle {
                    reason: decision.reason,
                    public_basis: decision.public_basis.clone(),
                });
                changed += 1;
            }
        }
        if changed != 0 {
            self.commit(snapshot)?;
        }
        Ok(changed)
    }

    /// The task's verified cutoff and this reader's complete prefix authorize the stop.
    pub fn apply_watermark(
        &mut self,
        key: &V::Key,
        prefix: &VerifiedPrefix<'_>,
    ) -> Result<WatermarkOutcome, PublisherError> {
        self.store.check_healthy()?;
        self.check_prefix(prefix)?;
        let task = self.task(key).ok_or(PublisherError::UnknownTask)?;
        if task.stop_reason.is_some() {
            return Ok(WatermarkOutcome::AlreadyStopped);
        }
        let Some(deadline) = task.verified_deadline else {
            return Ok(WatermarkOutcome::DeadlineUnknown);
        };
        let (q, theta) = prefix.tip();
        if theta < deadline {
            return Ok(WatermarkOutcome::BelowDeadline);
        }
        let mut snapshot = self.snapshot.clone();
        snapshot.tasks.get_mut(key).expect("known key").stop_reason =
            Some(PublicStop::Deadline { q, theta, deadline });
        self.commit(snapshot)?;
        Ok(WatermarkOutcome::Applied)
    }

    fn check_prefix(&self, prefix: &VerifiedPrefix<'_>) -> Result<(), PublisherError> {
        if prefix.source_id() != self.binding.source_id
            || prefix.reader_id() != self.binding.reader_id
        {
            return Err(PublisherError::PrefixIdentityMismatch);
        }
        Ok(())
    }

    fn attempt(&self, id: &AttemptId) -> Result<(&V::Key, usize, &Attempt), PublisherError> {
        if id.namespace != self.binding.namespace {
            return Err(PublisherError::UnknownAttempt);
        }
        let task = self
            .snapshot
            .tasks
            .values()
            .find(|entry| entry.task_number == id.task_number)
            .ok_or(PublisherError::UnknownAttempt)?;
        let index = id
            .attempt_number
            .checked_sub(1)
            .and_then(|number| usize::try_from(number).ok())
            .ok_or(PublisherError::UnknownAttempt)?;
        let attempt = task
            .attempts
            .get(index)
            .filter(|attempt| attempt.stable_id == *id)
            .ok_or(PublisherError::UnknownAttempt)?;
        Ok((&task.key, index, attempt))
    }

    fn set_attempt(
        &mut self,
        key: V::Key,
        index: usize,
        state: AttemptState,
    ) -> Result<(), PublisherError> {
        let mut snapshot = self.snapshot.clone();
        snapshot.tasks.get_mut(&key).expect("known key").attempts[index].state = state;
        self.commit(snapshot)
    }

    fn commit(&mut self, mut next: Snapshot<V::Key, V::Result>) -> Result<(), PublisherError> {
        next.generation = self
            .snapshot
            .generation
            .checked_add(1)
            .ok_or(StoreError::Capacity)?;
        let bytes = codec::encode_snapshot(&self.codec, &self.binding, &next)?;
        self.store.commit(&bytes)?;
        self.snapshot = next;
        Ok(())
    }
}

fn offer_outcome<K: Clone, R>(task: &PublicationTask<K, R>) -> OfferOutcome<K> {
    OfferOutcome {
        key: task.key.clone(),
        phase: task.phase(),
        missing: task.missing.clone(),
    }
}

fn submission<K: Clone>(key: &K, attempt: &Attempt) -> Submission<K> {
    Submission {
        key: key.clone(),
        stable_id: attempt.stable_id.clone(),
        exact_payload: Arc::clone(&attempt.exact_payload),
        created_tick: attempt.created_tick,
    }
}

fn validate_basis(bytes: &[u8]) -> Result<(), PublisherError> {
    if bytes.is_empty() || bytes.len() > codec::MAX_REFERENCE_BYTES {
        return Err(PublisherError::InvalidBasis);
    }
    Ok(())
}

#[cfg(test)]
extern crate self as poc_publisher;

#[cfg(test)]
mod recovery_tests;
