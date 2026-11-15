//! Opt-in recovery records. These types describe state transitions and live
//! callback ownership without serializing native tasks or channel internals.
use crate::{Handoff, simplex::types::Context, types::Round};
use commonware_cryptography::{Digest, PublicKey};
use commonware_utils::channel::oneshot;
use futures::future::BoxFuture;
use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub type InputRef = [u8; 32];
pub type CutRef = [u8; 32];
pub type RecordToken = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActorId {
    Voter,
    Batcher,
    Resolver,
    Application,
    Archive,
    Driver,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RecoveryReceipt {
    pub sequence: u64,
    pub hash: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryError(pub &'static str);
impl std::fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for RecoveryError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignatureState {
    NotApplicable,
    Pending,
    Verified,
    Rejected,
}
/// Opaque input carried from the raw receive to native proposal selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProposalOrigin {
    pub input_ref: InputRef,
    pub signature_state: SignatureState,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationKind {
    Prepare,
    Propose,
    Verify,
    Certify,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationPhase {
    Claimed,
    Offered,
    Consumed,
    Cancelled,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelperPhase {
    WaitInput,
    WorkScheduled,
    WorkReady,
    BodyPersisting,
    ReadyToOffer,
    Retired,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProposalPhase {
    None,
    Preparing,
    Proposing,
    Waiting,
    Closed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionPhase {
    Pending,
    Nullify,
    Finalize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Digest(Vec<u8>),
    Bool(bool),
    HandoffWait,
    ResponseClosed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProposalFact {
    pub fact_sequence: u64,
    pub native_proposal_bytes: Vec<u8>,
    pub selected_input_ref: InputRef,
    pub selection_sequence: u64,
    pub first_receive_input_ref: InputRef,
    pub original_receive_tick_us: u64,
    pub original_receive_local_us: Option<i64>,
    pub transport_binding_verified: bool,
    pub decoded: bool,
    pub elected_leader_checked: bool,
    pub signature_state: SignatureState,
    pub verifying_parent_digest: Option<Vec<u8>>,
    pub application_result_ref: Option<RecordToken>,
    pub observed_slot_status: &'static str,
    pub conflicting_input_refs: Vec<InputRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationRecord {
    pub operation_id: [u8; 32],
    pub original_work_binding: Vec<u8>,
    pub kind: OperationKind,
    pub canonical_native_context_bytes: Option<Vec<u8>>,
    pub epoch: u64,
    pub view: u64,
    pub parent_view: Option<u64>,
    pub parent_digest: Option<Vec<u8>>,
    pub requested_payload: Option<Vec<u8>>,
    pub phase: OperationPhase,
    pub helper_phase: HelperPhase,
    pub claimed_sequence: u64,
    pub original_start_tick_us: Option<u64>,
    pub original_due_tick_us: Option<u64>,
    pub missing_dependencies: Vec<Vec<u8>>,
    pub offered_outcome: Option<Outcome>,
    pub outcome_evidence_refs: Vec<RecordToken>,
    pub offered_sequence: Option<u64>,
    pub consumed_sequence: Option<u64>,
    pub cancelled_sequence: Option<u64>,
    pub cancellation_reason: Option<&'static str>,
    pub reconnect_generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActorPhase {
    pub pending_proposal_operation: Option<RecordToken>,
    pub pending_proposal_context: Option<Vec<u8>>,
    pub proposal_phase: ProposalPhase,
    pub pending_verify_operation: Option<RecordToken>,
    pub verifying_proposal_bytes: Option<Vec<u8>>,
    pub verifying_parent_digest: Option<Vec<u8>>,
    pub outstanding_certify_operations: Vec<RecordToken>,
    pub committed_relay_refs: Vec<RecordToken>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoundTiming {
    pub view: u64,
    pub entered_at_local_us: Option<i64>,
    pub proposed_at_local_us: Option<i64>,
    pub comparison_decision_phase: DecisionPhase,
    pub leader_deadline_local_us: Option<i64>,
    pub certification_deadline_local_us: Option<i64>,
    pub latch_time_local_us: Option<i64>,
    pub latch_reason: Option<&'static str>,
    pub latch_cause_ref: Option<InputRef>,
    pub retry_deadline_local_us: Option<i64>,
    pub retry_arm_local_us: Option<i64>,
    pub stall_deadline_local_us: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimingSnapshot {
    pub current_view: u64,
    pub last_finalized_view: u64,
    pub stall_anchor: u64,
    pub native_state_projection_hash: [u8; 32],
    pub rounds: Vec<RoundTiming>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalPosition {
    pub section: u64,
    pub offset: u64,
    pub frame_bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallbackClaim {
    pub kind: OperationKind,
    pub epoch: u64,
    pub view: u64,
    pub context: Option<Vec<u8>>,
    pub parent_view: Option<u64>,
    pub parent_digest: Option<Vec<u8>>,
    pub payload: Option<Vec<u8>>,
}

/// Exact result of the real application dispatch following a native claim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallbackBinding {
    pub claim: CallbackClaim,
    pub operation_id: [u8; 32],
    pub original_work_binding: Vec<u8>,
    pub missing_dependencies: Vec<Vec<u8>>,
    pub original_start_tick_us: Option<u64>,
    pub original_due_tick_us: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryRecord {
    ProposalSelected {
        native_proposal_bytes: Vec<u8>,
        origin: ProposalOrigin,
    },
    IngressSignature {
        input_ref: InputRef,
        signature_state: SignatureState,
    },
    Claim(CallbackClaim),
    WorkScheduled {
        claim: RecordToken,
        start_tick_us: u64,
        due_tick_us: u64,
    },
    WorkReady {
        claim: RecordToken,
        block_bytes: Option<Vec<u8>>,
    },
    BodyPersisting {
        claim: RecordToken,
    },
    BodyDurable {
        claim: RecordToken,
        digest: [u8; 32],
        block_bytes: Vec<u8>,
    },
    Offer {
        claim: RecordToken,
        outcome: Outcome,
        evidence: Vec<RecordToken>,
    },
    Consume {
        claim: RecordToken,
        outcome: Outcome,
        applied: bool,
    },
    Cancel {
        claim: RecordToken,
        reason: &'static str,
    },
    ActorPhase(ActorPhase),
    Timing(TimingSnapshot),
    Preparation {
        positions: Vec<JournalPosition>,
        prospective_artifact: Option<Vec<u8>>,
    },
    ResolverStarted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryLoad {
    pub actor: ActorId,
    pub epoch: u64,
    pub native_boundary: Vec<JournalPosition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryBundle {
    pub cut: CutRef,
    pub timing: Option<TimingSnapshot>,
    pub proposals: Vec<ProposalFact>,
    pub operations: Vec<OperationRecord>,
    pub actor_phase: Option<ActorPhase>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryCut {
    pub cut_hash: CutRef,
    pub canonical_bytes: Vec<u8>,
}

pub struct RecoveryOptions<D: Digest, P: PublicKey> {
    pub store: Arc<dyn RecoveryStore>,
    pub reconnector: Arc<dyn RecoveryReconnector<D, P>>,
    pub startup: Arc<dyn RecoveryStartup>,
    pub session_not_before: Option<SystemTime>,
}

pub trait RecoveryStore: Send + Sync + 'static {
    fn stage(&self, record: RecoveryRecord) -> Result<RecordToken, RecoveryError>;
    fn bind_callback(&self, binding: CallbackBinding) -> Result<RecordToken, RecoveryError>;
    fn flush<'a>(
        &'a self,
        through: RecordToken,
    ) -> BoxFuture<'a, Result<RecoveryReceipt, RecoveryError>>;
    fn load<'a>(
        &'a self,
        request: RecoveryLoad,
    ) -> BoxFuture<'a, Result<RecoveryBundle, RecoveryError>>;
    fn checkpoint<'a>(
        &'a self,
        cut: RecoveryCut,
    ) -> BoxFuture<'a, Result<RecoveryReceipt, RecoveryError>>;
    fn claim_ingress(
        &self,
        actor: ActorId,
        channel: u32,
        source_key: Vec<u8>,
        exact_bytes: Vec<u8>,
    ) -> Result<InputRef, RecoveryError>;
}

#[derive(Clone)]
pub struct ReconnectRequest<D: Digest, P: PublicKey> {
    pub context: Option<Context<D, P>>,
    pub round: Round,
    pub operation: OperationRecord,
    pub reconnect_generation: u64,
}

pub enum Reconnected<D: Digest> {
    Prepare(oneshot::Receiver<Handoff<D>>),
    Propose(oneshot::Receiver<D>),
    Verify(oneshot::Receiver<bool>),
    Certify(oneshot::Receiver<bool>),
    Waiting,
    Closed,
    Cancelled,
}

pub trait RecoveryReconnector<D: Digest, P: PublicKey>: Send + Sync + 'static {
    fn reconnect<'a>(
        &'a self,
        request: ReconnectRequest<D, P>,
    ) -> BoxFuture<'a, Result<Reconnected<D>, RecoveryError>>;
}

pub trait RecoveryStartup: Send + Sync + 'static {
    fn ready(&self, actor: ActorId, cut: CutRef, state_hash: [u8; 32])
    -> Result<(), RecoveryError>;
    fn released<'a>(&'a self) -> BoxFuture<'a, Result<(), RecoveryError>>;
}

#[derive(Clone)]
pub(crate) struct ActorRecovery {
    pub store: Arc<dyn RecoveryStore>,
    pub startup: Arc<dyn RecoveryStartup>,
}
impl ActorRecovery {
    pub(crate) async fn wait(&self, actor: ActorId, epoch: u64, projection: &[u8]) {
        let restored = self
            .store
            .load(RecoveryLoad {
                actor,
                epoch,
                native_boundary: Vec::new(),
            })
            .await
            .expect("load actor recovery cut");
        use commonware_cryptography::{Hasher as _, Sha256};
        let domain = b"I1b/recovery-state/v1";
        let hash = Sha256::hash(&[
            &(domain.len() as u64).to_be_bytes(),
            domain,
            &(projection.len() as u64).to_be_bytes(),
            projection,
        ])
        .0;
        self.startup
            .ready(actor, restored.cut, hash)
            .expect("record recovery readiness");
        self.startup
            .released()
            .await
            .expect("native recovery startup");
    }
}

pub(crate) fn micros(time: SystemTime) -> Result<i64, RecoveryError> {
    let (positive, duration) = match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => (true, duration),
        Err(error) => (false, error.duration()),
    };
    if duration.subsec_nanos() % 1000 != 0 {
        return Err(RecoveryError(
            "native recovery time is not an exact microsecond",
        ));
    }
    let value = i128::try_from(duration.as_micros())
        .map_err(|_| RecoveryError("native timestamp overflow"))?;
    i64::try_from(if positive { value } else { -value })
        .map_err(|_| RecoveryError("native timestamp overflow"))
}

pub(crate) fn system_time(value: i64) -> Result<SystemTime, RecoveryError> {
    let delta = Duration::from_micros(value.unsigned_abs());
    if value >= 0 {
        UNIX_EPOCH.checked_add(delta)
    } else {
        UNIX_EPOCH.checked_sub(delta)
    }
    .ok_or(RecoveryError("native timestamp overflow"))
}

/// Carry the received canonical bytes before the typed codec consumes them.
/// The controller ties the next raw channel receive to its write-ahead input.
pub(crate) struct IngressCapture<R> {
    receiver: R,
    store: Option<Arc<dyn RecoveryStore>>,
    channel: u32,
    stamp: Arc<std::sync::Mutex<Option<InputRef>>>,
}
impl<R> IngressCapture<R> {
    pub(crate) fn new(
        receiver: R,
        store: Option<Arc<dyn RecoveryStore>>,
        channel: u32,
    ) -> (Self, Arc<std::sync::Mutex<Option<InputRef>>>) {
        let stamp = Arc::new(std::sync::Mutex::new(None));
        (
            Self {
                receiver,
                store,
                channel,
                stamp: stamp.clone(),
            },
            stamp,
        )
    }
}
impl<R: std::fmt::Debug> std::fmt::Debug for IngressCapture<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IngressCapture")
            .field("receiver", &self.receiver)
            .field("channel", &self.channel)
            .finish()
    }
}
impl<R: commonware_p2p::Receiver> commonware_p2p::Receiver for IngressCapture<R> {
    type PublicKey = R::PublicKey;
    type Error = R::Error;
    async fn recv(&mut self) -> Result<commonware_p2p::Message<Self::PublicKey>, Self::Error> {
        use commonware_codec::Encode as _;
        let (key, raw) = self.receiver.recv().await?;
        if let Some(store) = &self.store {
            let id = store
                .claim_ingress(
                    ActorId::Batcher,
                    self.channel,
                    key.encode().to_vec(),
                    raw.as_ref().to_vec(),
                )
                .expect("claim exact granted native ingress");
            assert!(
                self.stamp.lock().unwrap().replace(id).is_none(),
                "unconsumed native ingress reference"
            );
        }
        Ok((key, raw))
    }
    fn capacity(&self) -> std::num::NonZeroUsize {
        self.receiver.capacity()
    }
}
