use crate::{
    Error, Id, Result, SimContext, hash_parts,
    native_config::Resolved,
    native_network::{Network, RawCertificate},
    native_scheme::GuardedScheme,
    native_wire::{
        BodyPacket, BodyPush, BodyRequest, Delivered, Effect, ProofObserved, Work, WorkKind,
        body_packet, effect,
    },
};
use commonware_actor::Feedback;
use commonware_codec::Encode as _;
use commonware_consensus::{
    Automaton, CertifiableAutomaton, Relay, Reporter,
    simplex::{
        Plan, recovery as recovery_hooks,
        types::{Activity, Certificate, Context},
    },
    types::Round,
};
use commonware_cryptography::{ed25519::PublicKey, sha256::Digest};
use commonware_p2p::Recipients;
use commonware_runtime::{Blob, ReadOptions, Spawner, Storage, Supervisor, WriteOptions};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};
use tokio::sync::oneshot;

pub(crate) type AppContext = Context<Digest, PublicKey>;

#[derive(Clone, Debug)]
pub(crate) struct Block {
    pub digest: Id,
    pub parent: Id,
    pub height: u64,
    pub bytes: Vec<u8>,
}
impl Block {
    pub fn new(cid: Id, height: u64, parent: Id) -> Self {
        let mut bytes = Vec::with_capacity(333);
        bytes.push(1);
        bytes.extend_from_slice(&cid);
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&parent);
        bytes.extend_from_slice(&256u32.to_be_bytes());
        bytes.extend((0..256).map(|i| (height as u8).wrapping_add(i as u8)));
        let digest = hash_parts(b"I1b/fixture-block/v1", &[&bytes]);
        Self {
            digest,
            parent,
            height,
            bytes,
        }
    }
    pub fn read(config: &Resolved, bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 333
            || bytes[0] != 1
            || bytes[1..33] != config.cid
            || bytes[73..77] != 256u32.to_be_bytes()
        {
            return Err(Error("invalid fixture block encoding"));
        }
        let height = u64::from_be_bytes(bytes[33..41].try_into().unwrap());
        let parent = bytes[41..73].try_into().unwrap();
        let value = Self::new(config.cid, height, parent);
        if height == 0 || height > config.base.height_limit || value.bytes != bytes {
            return Err(Error("invalid fixture block payload or height"));
        }
        Ok(value)
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub(crate) enum AppAudit {
    WorkPending {
        binding: WorkBinding,
    },
    WorkStarted {
        binding: WorkBinding,
        start_tick_us: u64,
        due_tick_us: u64,
        body_digest: Option<Id>,
        body_parent: Option<Id>,
        native_parent_view: Option<u64>,
    },
    WorkReconnected {
        binding: WorkBinding,
        reconnect_generation: u64,
        original_due_tick_us: Option<u64>,
    },
    WorkCompleted {
        binding: WorkBinding,
        completed_tick_us: u64,
        result_valid: bool,
        body_digest: Option<Id>,
        body_parent: Option<Id>,
        native_parent_view: Option<u64>,
    },
    WorkDiscarded {
        work: Work,
        binding: Option<WorkBinding>,
        cancel_tick_us: u64,
        reason: &'static str,
    },
    Persisted {
        digest: Id,
        height: u64,
    },
    BlockReceived {
        digest: Id,
        height: u64,
    },
    InvalidBody,
    Proof {
        proof: ProofObserved,
    },
    Delivery {
        delivered: Delivered,
    },
    RejectedInput {
        reason: &'static str,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct WorkBinding {
    pub(crate) work: Work,
    pub(crate) member: u32,
    pub(crate) epoch: u64,
    pub(crate) view: u64,
    pub(crate) parent_view: u64,
    pub(crate) parent_digest: Id,
    pub(crate) requested_payload: Id,
    pub(crate) request_generation: u64,
    pub(crate) request_tick_us: u64,
}
impl WorkBinding {
    fn identity(&self, cid: Id) -> Id {
        hash_parts(
            b"I1b/fresh-work/v1",
            &[
                &cid,
                &self.member.to_be_bytes(),
                &(self.work.kind as u32).to_be_bytes(),
                &self.epoch.to_be_bytes(),
                &self.view.to_be_bytes(),
                &self.parent_view.to_be_bytes(),
                &self.parent_digest,
                &self.requested_payload,
                &self.request_generation.to_be_bytes(),
            ],
        )
    }
}
struct Pending {
    binding: WorkBinding,
    target: Target,
    complete: oneshot::Sender<Option<Block>>,
    due: Option<u64>,
    recovery_claim: Option<(
        Arc<dyn recovery_hooks::RecoveryStore>,
        recovery_hooks::RecordToken,
    )>,
}
#[derive(Clone)]
enum Target {
    Build { parent: Id, view: u64 },
    Verify { digest: Id, parent: Id, view: u64 },
    Certify { digest: Id },
}
enum Availability {
    Pending(Id),
    Invalid,
    Ready(Block),
}

pub(crate) struct Archive {
    pub config: Arc<Resolved>,
    pub blocks: BTreeMap<Id, Block>,
    pending: BTreeMap<Id, Pending>,
    durable: BTreeSet<Id>,
    proof: BTreeMap<Id, ProofObserved>,
    native_parents: BTreeMap<(u64, Id), u64>,
    requests: BTreeMap<Id, u64>,
    sent_requests: BTreeMap<Id, u64>,
    delivered: Id,
    pub height: u64,
    pub active: bool,
    pub tick: u64,
    work_generation: u64,
    pub events: Vec<AppAudit>,
    pub effects: Vec<Effect>,
}

impl Archive {
    pub fn new(config: Arc<Resolved>) -> Self {
        Self {
            delivered: config.genesis,
            config,
            blocks: BTreeMap::new(),
            pending: BTreeMap::new(),
            durable: BTreeSet::new(),
            proof: BTreeMap::new(),
            native_parents: BTreeMap::new(),
            requests: BTreeMap::new(),
            sent_requests: BTreeMap::new(),
            height: 0,
            active: false,
            tick: 0,
            work_generation: 0,
            events: vec![],
            effects: vec![],
        }
    }
    pub(crate) fn live_recovered_work(&self, id: Id) -> bool {
        self.pending
            .get(&id)
            .is_some_and(|pending| !pending.complete.is_closed())
    }

    pub(crate) fn empty_archive_frontier(&self) -> Result<u64> {
        if !self.blocks.is_empty()
            || !self.durable.is_empty()
            || !self.proof.is_empty()
            || !self.native_parents.is_empty()
            || !self.requests.is_empty()
            || !self.sent_requests.is_empty()
            || self.delivered != self.config.genesis
            || self.height != 0
            || self.pending.len() > 1
            || self.pending.values().any(|p| {
                p.due.is_some()
                    || p.binding.work.kind != WorkKind::Verify as i32
                    || p.complete.is_closed()
            })
        {
            return Err(Error(
                "bounded pending Verify cut cannot omit live archive history",
            ));
        }
        Ok(self.work_generation)
    }

    pub(crate) fn restore_work_frontier(&mut self, generation: u64) -> Result<()> {
        if self.active || self.work_generation != 0 || !self.pending.is_empty() {
            return Err(Error("native archive work frontier already installed"));
        }
        self.work_generation = generation;
        Ok(())
    }

    fn availability(&self, target: &Target) -> Availability {
        match target {
            Target::Build { parent, .. } => {
                let parent_height = if *parent == self.config.genesis {
                    0
                } else {
                    match self.ancestry(*parent) {
                        Availability::Ready(b) => b.height,
                        other => return other,
                    }
                };
                if parent_height >= self.config.base.height_limit {
                    return Availability::Invalid;
                }
                Availability::Ready(Block::new(self.config.cid, parent_height + 1, *parent))
            }
            Target::Verify { digest, parent, .. } => match self.ancestry(*digest) {
                Availability::Ready(block) if block.parent != *parent => Availability::Invalid,
                other => other,
            },
            Target::Certify { digest } => self.ancestry(*digest),
        }
    }
    fn ancestry(&self, digest: Id) -> Availability {
        let Some(tip) = self.blocks.get(&digest) else {
            return Availability::Pending(digest);
        };
        let mut cursor = tip;
        loop {
            if cursor.height == 1 {
                return if cursor.parent == self.config.genesis {
                    Availability::Ready(tip.clone())
                } else {
                    Availability::Invalid
                };
            }
            let Some(parent) = self.blocks.get(&cursor.parent) else {
                return Availability::Pending(cursor.parent);
            };
            if parent.height + 1 != cursor.height {
                return Availability::Invalid;
            }
            cursor = parent;
        }
    }
    fn queue(
        &mut self,
        member: u32,
        kind: WorkKind,
        round: Round,
        target: Target,
        complete: oneshot::Sender<Option<Block>>,
    ) -> WorkBinding {
        let zero = [0; 32];
        let (parent_view, parent_digest, requested_payload) = match target {
            Target::Build { parent, view } => (view, parent, zero),
            Target::Verify {
                digest,
                parent,
                view,
            } => (view, parent, digest),
            Target::Certify { digest } => (0, zero, digest),
        };
        self.work_generation = self
            .work_generation
            .checked_add(1)
            .expect("native request generation overflow");
        let epoch = round.epoch().get();
        let view = round.view().get();
        let work = Work {
            operation_id: vec![],
            cid: self.config.cid.to_vec(),
            kind: kind as i32,
            duration_us: self.config.base.work_us,
        };
        let mut binding = WorkBinding {
            work,
            member,
            epoch,
            view,
            parent_view,
            parent_digest,
            requested_payload,
            request_generation: self.work_generation,
            request_tick_us: self.tick,
        };
        let operation = binding.identity(self.config.cid);
        binding.work.operation_id = operation.to_vec();
        assert!(
            self.pending
                .insert(
                    operation,
                    Pending {
                        binding: binding.clone(),
                        target,
                        complete,
                        due: None,
                        recovery_claim: None,
                    }
                )
                .is_none(),
            "duplicate live fixture request"
        );
        self.events.push(AppAudit::WorkPending {
            binding: binding.clone(),
        });
        self.start_ready();
        binding
    }
    fn discarded(&mut self, p: Pending, reason: &'static str) {
        self.events.push(AppAudit::WorkDiscarded {
            work: p.binding.work.clone(),
            binding: Some(p.binding),
            cancel_tick_us: self.tick,
            reason,
        });
    }
    pub fn start_ready(&mut self) {
        let ids = self.pending.keys().copied().collect::<Vec<_>>();
        for id in ids {
            let p = &self.pending[&id];
            if p.complete.is_closed() {
                let p = self.pending.remove(&id).unwrap();
                self.discarded(p, "abandoned_consumer");
            } else if p.due.is_none() {
                let (body_digest, body_parent) = match self.availability(&p.target) {
                    Availability::Pending(_) => continue,
                    Availability::Ready(block) => (Some(block.digest), Some(block.parent)),
                    Availability::Invalid => (None, None),
                };
                let p = self.pending.get_mut(&id).unwrap();
                let due = self
                    .tick
                    .checked_add(self.config.base.work_us)
                    .expect("native work deadline overflow");
                p.due = Some(due);
                if let Some((store, claim)) = &p.recovery_claim {
                    store
                        .stage(recovery_hooks::RecoveryRecord::WorkScheduled {
                            claim: *claim,
                            start_tick_us: self.tick,
                            due_tick_us: due,
                        })
                        .expect("record native helper work schedule");
                }
                let native_parent_view = body_digest
                    .and_then(|digest| self.native_parents.get(&(p.binding.view, digest)).copied());
                self.events.push(AppAudit::WorkStarted {
                    binding: p.binding.clone(),
                    start_tick_us: self.tick,
                    due_tick_us: due,
                    body_digest,
                    body_parent,
                    native_parent_view,
                });
                self.effects.push(Effect {
                    kind: Some(effect::Kind::Work(p.binding.work.clone())),
                });
            }
        }
    }
    pub fn complete(&mut self, work: &Work) -> Result<()> {
        let id: Id = work
            .operation_id
            .as_slice()
            .try_into()
            .map_err(|_| Error("invalid work id"))?;
        if work.cid != self.config.cid
            || work.duration_us != self.config.base.work_us
            || !matches!(
                WorkKind::try_from(work.kind),
                Ok(WorkKind::Propose | WorkKind::Verify | WorkKind::Certify)
            )
        {
            return Err(Error("invalid work coordinates"));
        }
        let Some(p) = self.pending.remove(&id) else {
            self.events.push(AppAudit::WorkDiscarded {
                work: work.clone(),
                binding: None,
                cancel_tick_us: self.tick,
                reason: "unknown_or_retired_work",
            });
            return Ok(());
        };
        if p.due != Some(self.tick) || p.binding.work != *work {
            return Err(Error("unregistered native work completion"));
        }
        if !self.active || p.complete.is_closed() {
            self.discarded(p, "inactive_or_abandoned");
            return Ok(());
        }
        let value = match self.availability(&p.target) {
            Availability::Ready(block) => Some(block),
            Availability::Invalid => None,
            Availability::Pending(_) => return Err(Error("ready work lost required input")),
        };
        let native_parent_view = value.as_ref().and_then(|body| {
            self.native_parents
                .get(&(p.binding.view, body.digest))
                .copied()
        });
        self.events.push(AppAudit::WorkCompleted {
            binding: p.binding,
            completed_tick_us: self.tick,
            result_valid: value.is_some(),
            body_digest: value.as_ref().map(|b| b.digest),
            body_parent: value.as_ref().map(|b| b.parent),
            native_parent_view,
        });
        let _ = p.complete.send(value);
        Ok(())
    }
    pub fn close(&mut self) {
        self.active = false;
        for (_, p) in std::mem::take(&mut self.pending) {
            self.discarded(p, "session_retired");
        }
    }
    pub fn accept_block(&mut self, bytes: &[u8]) -> Result<Id> {
        let block = Block::read(&self.config, bytes)?;
        let digest = block.digest;
        if self.blocks.insert(digest, block.clone()).is_none() {
            self.events.push(AppAudit::BlockReceived {
                digest,
                height: block.height,
            });
        }
        if self.active {
            self.start_ready();
        }
        self.deliver_ready();
        Ok(digest)
    }
    pub fn proof(&mut self, certificate: RawCertificate) {
        let proposal = match &certificate {
            RawCertificate::Notarization(value) => &value.proposal,
            RawCertificate::Finalization(value) => &value.proposal,
            RawCertificate::Nullification(_) => return,
        };
        self.native_parents
            .entry((proposal.round.view().get(), proposal.payload.0))
            .or_insert(proposal.parent.get());
        let RawCertificate::Finalization(finalized) = certificate else {
            return;
        };
        let digest = finalized.proposal.payload.0;
        let proof = ProofObserved {
            cid: self.config.cid.to_vec(),
            epoch: finalized.proposal.round.epoch().get(),
            view: finalized.proposal.round.view().get(),
            payload_digest: digest.to_vec(),
            finalization_bytes: finalized.encode().to_vec(),
        };
        if self.proof.contains_key(&digest) {
            return;
        }
        self.proof.insert(digest, proof.clone());
        self.events.push(AppAudit::Proof {
            proof: proof.clone(),
        });
        self.effects.push(Effect {
            kind: Some(effect::Kind::Proof(proof)),
        });
        self.deliver_ready();
    }
    fn deliver_ready(&mut self) {
        // A verified finalization proves its contiguous validated ancestry.
        let proofs: Vec<_> = self.proof.keys().copied().collect();
        for tip in proofs {
            let Availability::Ready(mut current) = self.ancestry(tip) else {
                continue;
            };
            if current.height <= self.height {
                continue;
            }
            let mut chain = vec![];
            while current.height > self.height {
                chain.push(current.clone());
                if current.height == self.height + 1 {
                    break;
                }
                current = self.blocks[&current.parent].clone();
            }
            if current.parent != self.delivered {
                continue;
            }
            for block in chain.into_iter().rev() {
                self.height = block.height;
                self.delivered = block.digest;
                let delivered = Delivered {
                    cid: self.config.cid.to_vec(),
                    height: block.height,
                    digest: block.digest.to_vec(),
                    parent_digest: block.parent.to_vec(),
                    block_bytes: block.bytes,
                };
                self.events.push(AppAudit::Delivery {
                    delivered: delivered.clone(),
                });
                self.effects.push(Effect {
                    kind: Some(effect::Kind::Delivered(delivered)),
                });
            }
        }
    }
    pub fn matches_body_request(&self, digest: &Id, request: u64) -> bool {
        self.requests.get(digest) == Some(&request)
    }

    pub fn body_requests(&mut self, tick: u64) -> Vec<(Id, u64, Vec<u8>)> {
        let mut needed = BTreeSet::new();
        for p in self.pending.values() {
            if let Availability::Pending(d) = self.availability(&p.target) {
                needed.insert(d);
            }
        }
        for digest in self.proof.keys() {
            if let Availability::Pending(d) = self.ancestry(*digest) {
                needed.insert(d);
            }
        }
        needed
            .into_iter()
            .filter(|d| self.sent_requests.get(d).is_none_or(|until| tick >= *until))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|digest| {
                let id = (self.requests.len() as u64)
                    .checked_add(1)
                    .expect("request counter");
                let id = *self.requests.entry(digest).or_insert(id);
                self.sent_requests.insert(
                    digest,
                    tick.checked_add(self.config.base.fetch_retry)
                        .expect("body retry interval overflow"),
                );
                let request = BodyPacket {
                    kind: Some(body_packet::Kind::Request(BodyRequest {
                        request_id: id,
                        cid: self.config.cid.to_vec(),
                        wanted_digest: digest.to_vec(),
                    })),
                };
                (digest, id, request.encode_to_vec())
            })
            .collect()
    }
}

#[derive(Clone)]
pub(crate) struct App {
    context: Arc<SimContext>,
    pub archive: Arc<Mutex<Archive>>,
    pub network: Network,
    recovery: Option<Arc<dyn recovery_hooks::RecoveryStore>>,
}

impl App {
    pub fn new(context: SimContext, archive: Arc<Mutex<Archive>>, network: Network) -> Self {
        Self {
            context: Arc::new(context),
            archive,
            network,
            recovery: None,
        }
    }
    pub fn with_recovery(mut self, store: Option<Arc<dyn recovery_hooks::RecoveryStore>>) -> Self {
        self.recovery = store;
        self
    }
    fn start<T: Send + 'static>(
        &self,
        kind: WorkKind,
        round: Round,
        target: Target,
        native_context: Option<Vec<u8>>,
        send: oneshot::Sender<T>,
        result: impl FnOnce(Option<Block>) -> Option<T> + Send + 'static,
        encode_outcome: impl Fn(&T) -> recovery_hooks::Outcome + Send + 'static,
    ) {
        let (complete, wait) = oneshot::channel();
        let mut archive = self.archive.lock().unwrap();
        let binding = archive.queue(self.network.factory.member, kind, round, target, complete);
        let recovery = if let Some(store) = &self.recovery {
            let operation_id: Id = binding
                .work
                .operation_id
                .as_slice()
                .try_into()
                .expect("native WorkBinding operation ID");
            let pending = archive
                .pending
                .get(&operation_id)
                .expect("application owns callback producer");
            let missing_dependencies = match archive.availability(&pending.target) {
                Availability::Pending(missing) => vec![missing.to_vec()],
                _ => vec![],
            };
            let (op_kind, parent_view, parent_digest, payload) = match &pending.target {
                Target::Build { parent, view } => (
                    recovery_hooks::OperationKind::Propose,
                    Some(*view),
                    Some(parent.to_vec()),
                    None,
                ),
                Target::Verify {
                    digest,
                    parent,
                    view,
                } => (
                    recovery_hooks::OperationKind::Verify,
                    Some(*view),
                    Some(parent.to_vec()),
                    Some(digest.to_vec()),
                ),
                Target::Certify { digest } => (
                    recovery_hooks::OperationKind::Certify,
                    None,
                    None,
                    Some(digest.to_vec()),
                ),
            };
            let callback = recovery_hooks::CallbackBinding {
                claim: recovery_hooks::CallbackClaim {
                    kind: op_kind,
                    epoch: binding.epoch,
                    view: binding.view,
                    context: native_context,
                    parent_view,
                    parent_digest,
                    payload,
                },
                operation_id,
                original_work_binding: serde_json::to_vec(&binding)
                    .expect("encode native application callback identity"),
                missing_dependencies,
                original_start_tick_us: pending.due.map(|_| archive.tick),
                original_due_tick_us: pending.due,
            };
            // The normal application has now allocated its real producer and
            // WorkBinding. Bind that exact dispatch before returning the receiver.
            drop(archive);
            let token = store
                .bind_callback(callback)
                .expect("bind actual native application callback");
            self.archive
                .lock()
                .unwrap()
                .pending
                .get_mut(&operation_id)
                .expect("bound callback retains original producer")
                .recovery_claim = Some((store.clone(), token));
            Some((store.clone(), token))
        } else {
            drop(archive);
            None
        };
        self.spawn_completion(kind, send, wait, recovery, result, encode_outcome);
    }
    fn spawn_completion<T: Send + 'static>(
        &self,
        kind: WorkKind,
        mut send: oneshot::Sender<T>,
        wait: oneshot::Receiver<Option<Block>>,
        recovery: Option<(
            Arc<dyn recovery_hooks::RecoveryStore>,
            recovery_hooks::RecordToken,
        )>,
        result: impl FnOnce(Option<Block>) -> Option<T> + Send + 'static,
        encode_outcome: impl Fn(&T) -> recovery_hooks::Outcome + Send + 'static,
    ) {
        let app = self.clone();
        drop(self.context.child("work").spawn(move |context| async move {
            let value =
                tokio::select! {biased; _=send.closed()=>return,value=wait=>value.ok().flatten()};
            use recovery_hooks::RecoveryRecord as R;
            let mut evidence = Vec::new();
            if let Some((store, claim)) = &recovery {
                evidence.push(
                    store
                        .stage(R::WorkReady {
                            claim: *claim,
                            block_bytes: value.as_ref().map(|block| block.bytes.clone()),
                        })
                        .expect("stage actual helper result"),
                );
            }
            if let Some(value) = &value {
                if let Some((store, claim)) = &recovery {
                    let token = store
                        .stage(R::BodyPersisting { claim: *claim })
                        .expect("stage recovery body persistence");
                    store
                        .flush(token)
                        .await
                        .expect("durably record persistence intent");
                    app.persist(&context, value).await;
                    evidence.push(
                        store
                            .stage(R::BodyDurable {
                                claim: *claim,
                                digest: value.digest,
                                block_bytes: value.bytes.clone(),
                            })
                            .expect("record actual persisted body"),
                    );
                } else if matches!(kind, WorkKind::Propose | WorkKind::Certify) {
                    app.persist(&context, value).await;
                }
            }
            let value = result(value);
            if let Some((store, claim)) = &recovery {
                if send.is_closed() {
                    return;
                }
                let outcome = value
                    .as_ref()
                    .map_or(recovery_hooks::Outcome::ResponseClosed, encode_outcome);
                let token = store
                    .stage(R::Offer {
                        claim: *claim,
                        outcome,
                        evidence,
                    })
                    .expect("stage exact native callback response");
                store
                    .flush(token)
                    .await
                    .expect("sync native callback outcome before resolving sender");
            }
            if let Some(value) = value {
                let _ = send.send(value);
            }
        }));
    }
    async fn persist(&self, context: &SimContext, block: &Block) {
        if self.archive.lock().unwrap().durable.contains(&block.digest) {
            return;
        }
        let (blob, len) = context
            .open("body-fixture", &block.digest)
            .await
            .expect("open fixture body");
        if len == 0 {
            blob.write_at(0, block.bytes.clone(), WriteOptions::SYNC)
                .await
                .expect("sync fixture body");
        } else {
            assert_eq!(
                len,
                block.bytes.len() as u64,
                "immutable fixture body extent"
            );
            let bytes = blob
                .read_at(0, len as usize, ReadOptions::default())
                .await
                .expect("read fixture body")
                .coalesce();
            assert_eq!(bytes.as_ref(), block.bytes, "immutable fixture body bytes");
        }
        let mut archive = self.archive.lock().unwrap();
        archive
            .accept_block(&block.bytes)
            .expect("validated fixture body");
        archive.durable.insert(block.digest);
        archive.events.push(AppAudit::Persisted {
            digest: block.digest,
            height: block.height,
        });
    }
}

impl Automaton for App {
    type Context = AppContext;
    type Digest = Digest;
    async fn propose(&mut self, context: AppContext) -> oneshot::Receiver<Digest> {
        let (send, receive) = oneshot::channel();
        self.start(
            WorkKind::Propose,
            context.round,
            Target::Build {
                parent: context.parent.1.0,
                view: context.parent.0.get(),
            },
            self.recovery.as_ref().map(|_| context.encode().to_vec()),
            send,
            |block| block.map(|b| Digest(b.digest)),
            |digest| recovery_hooks::Outcome::Digest(digest.0.to_vec()),
        );
        receive
    }
    async fn verify(&mut self, context: AppContext, payload: Digest) -> oneshot::Receiver<bool> {
        let (send, receive) = oneshot::channel();
        self.start(
            WorkKind::Verify,
            context.round,
            Target::Verify {
                digest: payload.0,
                parent: context.parent.1.0,
                view: context.parent.0.get(),
            },
            self.recovery.as_ref().map(|_| context.encode().to_vec()),
            send,
            |block| Some(block.is_some()),
            |valid| recovery_hooks::Outcome::Bool(*valid),
        );
        receive
    }
}

impl CertifiableAutomaton for App {
    async fn certify(&mut self, round: Round, payload: Digest) -> oneshot::Receiver<bool> {
        let (send, receive) = oneshot::channel();
        self.start(
            WorkKind::Certify,
            round,
            Target::Certify { digest: payload.0 },
            None,
            send,
            |block| Some(block.is_some()),
            |valid| recovery_hooks::Outcome::Bool(*valid),
        );
        receive
    }
}

impl Relay for App {
    type Digest = Digest;
    type PublicKey = PublicKey;
    type Plan = Plan<PublicKey>;
    fn broadcast(&mut self, payload: Digest, plan: Self::Plan) -> Feedback {
        let archive = self.archive.lock().unwrap();
        let Some(block) = archive.blocks.get(&payload.0) else {
            return Feedback::Closed;
        };
        let encoded = BodyPacket {
            kind: Some(body_packet::Kind::Push(BodyPush {
                cid: archive.config.cid.to_vec(),
                block_bytes: block.bytes.clone(),
            })),
        }
        .encode_to_vec();
        let recipient: Vec<_> = match plan {
            Plan::Forward { recipients, .. } => match recipients {
                Recipients::All => (0..5).collect(),
                Recipients::One(key) => self
                    .network
                    .members()
                    .iter()
                    .position(|k| *k == key)
                    .map(|i| i as u32)
                    .into_iter()
                    .collect(),
                Recipients::Some(keys) => self
                    .network
                    .members()
                    .iter()
                    .enumerate()
                    .filter(|(_, key)| keys.contains(key))
                    .map(|(i, _)| i as u32)
                    .collect(),
            },
            _ => (0..5).collect(),
        };
        drop(archive);
        for dest in recipient {
            if dest != self.network.factory.member {
                self.network
                    .send_body(dest, encoded.clone(), payload.0, 0)
                    .expect("relay under controlled authority");
            }
        }
        Feedback::Ok
    }
}

impl Reporter for App {
    type Activity = Activity<GuardedScheme, Digest>;
    fn report(&mut self, activity: Self::Activity) -> Feedback {
        let certificate = match activity {
            Activity::Notarization(value) | Activity::Certification(value) => {
                Certificate::Notarization(value)
            }
            Activity::Finalization(value) => Certificate::Finalization(value),
            _ => return Feedback::Ok,
        };
        {
            let encoded = certificate.encode();
            let config = self.archive.lock().unwrap().config.clone();
            let cert = crate::native_network::decode_certificate(
                &config.verifier().expect("registry"),
                config.base.epoch,
                &encoded,
            )
            .expect("report emitted an invalid finalization");
            self.archive.lock().unwrap().proof(cert);
        }
        Feedback::Ok
    }
}

impl recovery_hooks::RecoveryReconnector<Digest, PublicKey> for App {
    fn reconnect<'a>(
        &'a self,
        request: recovery_hooks::ReconnectRequest<Digest, PublicKey>,
    ) -> futures::future::BoxFuture<
        'a,
        std::result::Result<recovery_hooks::Reconnected<Digest>, recovery_hooks::RecoveryError>,
    > {
        Box::pin(async move {
            use recovery_hooks::{HelperPhase, OperationKind, OperationPhase, RecoveryError};
            let op = request.operation;
            if op.kind != OperationKind::Verify
                || op.phase != OperationPhase::Claimed
                || op.helper_phase != HelperPhase::WaitInput
                || op.original_start_tick_us.is_some()
                || op.original_due_tick_us.is_some()
                || op.reconnect_generation.checked_add(1) != Some(request.reconnect_generation)
            {
                return Err(RecoveryError(
                    "unsupported pending operation for Verify reconnection",
                ));
            }
            let context = request
                .context
                .ok_or(RecoveryError("pending Verify has no native context"))?;
            let original: WorkBinding = serde_json::from_slice(&op.original_work_binding)
                .map_err(|_| RecoveryError("malformed original WorkBinding"))?;
            let encoded = serde_json::to_vec(&original)
                .map_err(|_| RecoveryError("encode original WorkBinding"))?;
            let target = Target::Verify {
                digest: original.requested_payload,
                parent: original.parent_digest,
                view: original.parent_view,
            };
            let (complete, wait) = oneshot::channel();
            let (send, receive) = oneshot::channel();
            let store = self.recovery.clone().ok_or(RecoveryError(
                "reconnect requires the shared operation store",
            ))?;
            let mut archive = self.archive.lock().unwrap();
            let config = &archive.config;
            if encoded != op.original_work_binding
                || original.work.operation_id != op.operation_id
                || original.identity(config.cid) != op.operation_id
                || original.work.cid != config.cid
                || original.work.duration_us != config.base.work_us
                || original.work.kind != WorkKind::Verify as i32
                || original.member != self.network.factory.member
                || original.epoch != context.round.epoch().get()
                || original.view != context.round.view().get()
                || original.parent_view != context.parent.0.get()
                || original.parent_digest != context.parent.1.0
                || op.canonical_native_context_bytes.as_deref() != Some(context.encode().as_ref())
                || op.requested_payload.as_deref() != Some(&original.requested_payload)
                || request.round != context.round
                || !(1..=archive.work_generation).contains(&original.request_generation)
                || original.request_tick_us > archive.tick
                || archive.pending.len() >= 64
                || archive.pending.contains_key(&op.operation_id)
            {
                return Err(RecoveryError(
                    "restored application callback disagrees with native binding",
                ));
            }
            let Availability::Pending(missing) = archive.availability(&target) else {
                return Err(RecoveryError(
                    "pending Verify dependencies incompatible with retained archive",
                ));
            };
            if op.missing_dependencies != [missing.to_vec()] {
                return Err(RecoveryError(
                    "pending Verify changed its original missing dependency",
                ));
            }
            let recovery = (store, op.claimed_sequence);
            archive.pending.insert(
                op.operation_id,
                Pending {
                    binding: original.clone(),
                    target,
                    complete,
                    due: None,
                    recovery_claim: Some(recovery.clone()),
                },
            );
            archive.events.push(AppAudit::WorkReconnected {
                binding: original,
                reconnect_generation: request.reconnect_generation,
                original_due_tick_us: None,
            });
            drop(archive);
            self.spawn_completion(
                WorkKind::Verify,
                send,
                wait,
                Some(recovery),
                |block| Some(block.is_some()),
                |value| recovery_hooks::Outcome::Bool(*value),
            );
            Ok(recovery_hooks::Reconnected::Verify(receive))
        })
    }
}
