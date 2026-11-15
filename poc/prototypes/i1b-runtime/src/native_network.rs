use crate::{
    Error, Id, NativeWire, Result,
    clock::system_time,
    driver::{ArchiveAuthority, Authority},
    hash_parts,
    native_config::{CAPACITY, NativeScheme, Resolved},
    native_wire::{BodyProvenance, Channel, Envelope, ResolverProvenance, envelope::Provenance},
    native_wire_support as wire,
};
use bytes::{Buf as _, Bytes};
use commonware_actor::{Feedback, Unreliable};
use commonware_codec::{Copying, Decode, Read, ReadExt};
use commonware_consensus::{
    Epochable,
    simplex::types::{Certificate, Vote},
};
use commonware_cryptography::{certificate::Verifier as _, ed25519::PublicKey, sha256::Digest};
use commonware_p2p::{
    BlockedSubscription, Blocker, CheckedSender, LimitedSender, Message, Receiver, Recipients,
};
use commonware_parallel::Sequential;
use commonware_runtime::{IoBuf, IoBufs};
use commonware_utils::{channel::ring, ordered::Set};
use governor::{Quota, RateLimiter};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    num::{NonZeroU32, NonZeroUsize},
    sync::{Arc, Mutex},
    time::SystemTime,
};
use tokio::sync::mpsc;

pub(crate) type RawVote = Vote<NativeScheme, Digest>;
pub(crate) type RawCertificate = Certificate<NativeScheme, Digest>;

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub(crate) enum NetworkAudit {
    Submit {
        channel: i32,
        destination: u32,
        attempt: u32,
        message_id: Vec<u8>,
    },
    RateLimited {
        channel: i32,
        destination: u32,
    },
    Receive {
        channel: i32,
        source: u32,
        attempt: u32,
        message_id: Vec<u8>,
    },
    Rejected {
        channel: i32,
        reason: &'static str,
    },
    Block {
        peer: u32,
    },
}

#[derive(Clone)]
pub(crate) struct PacketFactory {
    pub config: Arc<Resolved>,
    pub member: u32,
    counters: Arc<Mutex<BTreeMap<Id, u32>>>,
}

impl PacketFactory {
    pub fn new(config: Arc<Resolved>, member: u32) -> Self {
        Self {
            config,
            member,
            counters: Arc::default(),
        }
    }
    pub fn packet(
        &self,
        dst: u32,
        channel: Channel,
        bytes: Vec<u8>,
        priority: bool,
        provenance: Option<Provenance>,
    ) -> Result<Envelope> {
        if dst >= 5 || dst == self.member || bytes.len() > wire::MAX_PAYLOAD {
            return Err(Error("invalid native packet"));
        }
        let mut envelope = Envelope {
            run_id: self.config.run_id.to_vec(),
            cid: self.config.cid.to_vec(),
            epoch: self.config.base.epoch,
            src_member_id: self.member,
            dst_member_id: dst,
            channel: channel as i32,
            payload_hash: wire::wire_hash(channel as u32, &bytes).to_vec(),
            payload_bytes: bytes,
            priority,
            provenance,
            ..Default::default()
        };
        let id = wire::message_hash(&self.config.config_hash, &envelope);
        let mut counters = self.counters.lock().unwrap();
        let next = counters.entry(id).or_default();
        *next = next
            .checked_add(1)
            .ok_or(Error("native send counter exhausted"))?;
        envelope.attempt = *next;
        envelope.message_id = id.to_vec();
        Ok(envelope)
    }
}

pub(crate) fn validate_envelope(config: &Resolved, envelope: &Envelope) -> Result<Channel> {
    let channel =
        Channel::try_from(envelope.channel).map_err(|_| Error("unknown native channel"))?;
    if channel == Channel::Unspecified
        || envelope.run_id != config.run_id
        || envelope.cid != config.cid
        || envelope.epoch != config.base.epoch
        || envelope.src_member_id >= 5
        || envelope.dst_member_id >= 5
        || envelope.src_member_id == envelope.dst_member_id
        || envelope.attempt == 0
        || envelope.payload_bytes.len() > wire::MAX_PAYLOAD
        || envelope.payload_hash != wire::wire_hash(channel as u32, &envelope.payload_bytes)
        || envelope.message_id != wire::message_hash(&config.config_hash, envelope)
    {
        return Err(Error("invalid native envelope context or digest"));
    }
    let provenance = envelope
        .provenance
        .as_ref()
        .ok_or(Error("native envelope has no provenance"))?;
    let hash32 = |bytes: &[u8]| bytes.len() == 32;
    let valid = match provenance {
        Provenance::Scripted(p) => {
            envelope.src_member_id >= 2 && hash32(&p.transcript_id) && hash32(&p.entry_id)
        }
        Provenance::LocalWitness(p) => {
            envelope.src_member_id < 2
                && matches!(channel, Channel::Vote | Channel::Certificate)
                && !p.witness_ids.is_empty()
                && p.witness_ids.iter().all(|v| hash32(v))
                && hash32(&p.authorization_id)
                && p.receipt
                    .as_ref()
                    .is_some_and(|r| r.sequence > 0 && hash32(&r.hash))
        }
        Provenance::Resolver(p) => {
            envelope.src_member_id < 2
                && channel == Channel::Resolver
                && hash32(&p.source_record_id)
        }
        Provenance::Body(p) => {
            envelope.src_member_id < 2 && channel == Channel::Body && hash32(&p.content_id)
        }
    };
    if !valid {
        return Err(Error("invalid native envelope provenance"));
    }
    Ok(channel)
}

pub(crate) fn decode_vote(bytes: &[u8]) -> Result<RawVote> {
    RawVote::decode_cfg(Copying(bytes), &()).map_err(|_| Error("malformed native vote"))
}
pub(crate) fn decode_certificate(
    scheme: &NativeScheme,
    epoch: u64,
    bytes: &[u8],
) -> Result<RawCertificate> {
    let certificate =
        RawCertificate::decode_cfg(Copying(bytes), &scheme.certificate_codec_config())
            .map_err(|_| Error("malformed native certificate"))?;
    let mut rng = ChaCha20Rng::from_seed(hash_parts(b"I1b/public-certificate-check/v1", &[bytes]));
    if certificate.epoch().get() != epoch || !certificate.verify(&mut rng, scheme, &Sequential) {
        return Err(Error("invalid native proof"));
    }
    Ok(certificate)
}

#[derive(Clone)]
struct QuotaClock(Authority);
impl governor::clock::Clock for QuotaClock {
    type Instant = SystemTime;
    fn now(&self) -> SystemTime {
        system_time(
            self.0
                .clock_read("governor")
                .local_time_us
                .expect("controlled quota clock"),
        )
    }
}

type Limiter = commonware_runtime::RateLimiter<QuotaClock>;
#[derive(Default)]
struct BlockState {
    peers: BTreeSet<PublicKey>,
    subscriptions: Vec<ring::Sender<Set<PublicKey>>>,
}

#[derive(Clone)]
pub(crate) struct Network {
    pub factory: PacketFactory,
    members: [PublicKey; 5],
    verify: NativeScheme,
    scope: Authority,
    blocked: Arc<Mutex<BlockState>>,
    quotas: Arc<Mutex<BTreeMap<(i32, u32), Limiter>>>,
    out: Arc<Mutex<Vec<Envelope>>>,
    pub audit: Arc<Mutex<Vec<NetworkAudit>>>,
}

impl Network {
    pub fn new(
        factory: PacketFactory,
        scope: Authority,
        out: Arc<Mutex<Vec<Envelope>>>,
        audit: Arc<Mutex<Vec<NetworkAudit>>>,
    ) -> Result<Self> {
        let members = factory.config.public_keys()?;
        let verify = factory.config.verifier()?;
        Ok(Self {
            factory,
            members,
            verify,
            scope,
            blocked: Arc::default(),
            quotas: Arc::default(),
            out,
            audit,
        })
    }
    pub(crate) fn empty_network_frontier(&self) -> Result<()> {
        if !self.factory.counters.lock().unwrap().is_empty()
            || !self.blocked.lock().unwrap().peers.is_empty()
            || !self.quotas.lock().unwrap().is_empty()
        {
            return Err(Error(
                "bounded pending Verify cut cannot omit network history",
            ));
        }
        Ok(())
    }

    pub fn members(&self) -> &[PublicKey; 5] {
        &self.members
    }
    pub fn sender(&self, channel: Channel) -> NativeTx {
        NativeTx {
            network: self.clone(),
            channel,
        }
    }
    pub fn blocked_member(&self, member: u32) -> bool {
        self.members
            .get(member as usize)
            .is_some_and(|key| self.blocked.lock().unwrap().peers.contains(key))
    }
    fn peer_index(&self, key: &PublicKey) -> Option<u32> {
        self.members
            .iter()
            .position(|known| known == key)
            .map(|i| i as u32)
    }
    fn recipients(&self, list: Recipients<PublicKey>) -> Option<BTreeSet<u32>> {
        match list {
            Recipients::All => Some((0..5).filter(|id| *id != self.factory.member).collect()),
            Recipients::One(key) => Some(BTreeSet::from([self.peer_index(&key)?])),
            Recipients::Some(list) => list.iter().map(|key| self.peer_index(key)).collect(),
        }
    }
    pub fn send_body(
        &self,
        dst: u32,
        bytes: Vec<u8>,
        content: Id,
        request_id: u64,
    ) -> Result<bool> {
        self.scope.check()?;
        self.stage_body(dst, bytes, content, request_id)
    }
    pub fn send_archived_body(
        &self,
        authority: &ArchiveAuthority,
        dst: u32,
        bytes: Vec<u8>,
        content: Id,
        request_id: u64,
    ) -> Result<bool> {
        if authority.check().is_err() {
            return Ok(false);
        }
        self.stage_body(dst, bytes, content, request_id)
    }
    fn stage_body(&self, dst: u32, bytes: Vec<u8>, content: Id, request_id: u64) -> Result<bool> {
        if self.blocked_member(dst) || self.check_quota(Channel::Body, dst).is_err() {
            return Ok(false);
        }
        let packet = self.factory.packet(
            dst,
            Channel::Body,
            bytes,
            false,
            Some(Provenance::Body(BodyProvenance {
                content_id: content.to_vec(),
                request_id,
            })),
        )?;
        self.stage(packet);
        Ok(true)
    }
    fn check_quota(&self, channel: Channel, dst: u32) -> std::result::Result<(), SystemTime> {
        let Some(per_second) = NonZeroU32::new(self.factory.config.base.quota_per_second) else {
            return Ok(());
        };
        let mut quotas = self.quotas.lock().unwrap();
        let limiter = quotas.entry((channel as i32, dst)).or_insert_with(|| {
            RateLimiter::direct_with_clock(
                Quota::per_second(per_second),
                QuotaClock(self.scope.clone()),
            )
        });
        limiter.check().map(|_| ()).map_err(|limited| {
            self.audit.lock().unwrap().push(NetworkAudit::RateLimited {
                channel: channel as i32,
                destination: dst,
            });
            limited.earliest_possible()
        })
    }
    fn stage(&self, packet: Envelope) {
        self.audit.lock().unwrap().push(NetworkAudit::Submit {
            channel: packet.channel,
            destination: packet.dst_member_id,
            attempt: packet.attempt,
            message_id: packet.message_id.clone(),
        });
        self.out.lock().unwrap().push(packet);
    }
    fn resolve_provenance(&self, data: &[u8]) -> Result<ResolverProvenance> {
        let mut input = Copying(data);
        let id = u64::read(&mut input).map_err(|_| Error("malformed resolver id"))?;
        let operation = u8::read(&mut input).map_err(|_| Error("malformed resolver operation"))?;
        match operation {
            0 => {
                let _ = u64::read(&mut input).map_err(|_| Error("malformed resolver key"))?;
            }
            1 => {
                let body = Bytes::read_cfg(&mut input, &(..=wire::MAX_PAYLOAD).into())
                    .map_err(|_| Error("malformed resolver response"))?;
                decode_certificate(&self.verify, self.factory.config.base.epoch, &body)?;
            }
            2 => {}
            _ => return Err(Error("unknown resolver operation")),
        }
        if input.has_remaining() {
            return Err(Error("extra resolver packet bytes"));
        }
        if operation == 0 && id == u64::MAX && self.factory.config.base.recovery.is_some() {
            self.scope.fail("native resolver request id exhausted");
            return Err(Error("native resolver request id exhausted"));
        }
        Ok(ResolverProvenance {
            request_id: id,
            source_record_id: hash_parts(b"I1b/resolver-source/v1", &[data]).to_vec(),
        })
    }
}

#[derive(Clone)]
pub(crate) struct NativeTx {
    network: Network,
    channel: Channel,
}
pub(crate) struct Checked<'a> {
    sender: &'a NativeTx,
    destinations: Vec<u32>,
}
impl LimitedSender for NativeTx {
    type PublicKey = PublicKey;
    type Checked<'a> = Checked<'a>;
    fn check(
        &mut self,
        list: Recipients<PublicKey>,
    ) -> std::result::Result<Self::Checked<'_>, SystemTime> {
        let Ok(access) = self.network.scope.check() else {
            return Err(SystemTime::UNIX_EPOCH);
        };
        let now = system_time(access.local_time_us.expect("checked node clock"));
        let Some(destinations) = self.network.recipients(list) else {
            self.network
                .audit
                .lock()
                .unwrap()
                .push(NetworkAudit::Rejected {
                    channel: self.channel as i32,
                    reason: "unknown_peer",
                });
            return Err(now);
        };
        let mut admitted = vec![];
        let mut earliest = None;
        for dst in destinations {
            if dst == self.network.factory.member || self.network.blocked_member(dst) {
                continue;
            }
            match self.network.check_quota(self.channel, dst) {
                Ok(()) => admitted.push(dst),
                Err(next) => {
                    earliest = Some(earliest.map_or(next, |old: SystemTime| old.min(next)))
                }
            }
        }
        if admitted.is_empty() {
            return Err(earliest.unwrap_or(now));
        }
        Ok(Checked {
            sender: self,
            destinations: admitted,
        })
    }
}
impl CheckedSender for Checked<'_> {
    type PublicKey = PublicKey;
    fn recipients(&self) -> Vec<PublicKey> {
        self.destinations
            .iter()
            .map(|dst| self.sender.network.members[*dst as usize].clone())
            .collect()
    }
    fn send(self, data: impl Into<IoBufs> + Send, priority: bool) -> Unreliable<Feedback> {
        let sender = self.sender;
        let network = &sender.network;
        if network.scope.check().is_err() {
            return Unreliable::rejected();
        }
        let data = data.into().coalesce();
        let bytes = data.as_ref();
        if bytes.len() > wire::MAX_PAYLOAD {
            return Unreliable::rejected();
        }
        let provenance = match sender.channel {
            Channel::Vote | Channel::Certificate => {
                if !priority {
                    return Unreliable::rejected();
                }
                for dst in &self.destinations {
                    network
                        .scope
                        .stage_route(
                            NativeWire {
                                channel: sender.channel as u32,
                                bytes: bytes.to_vec(),
                            },
                            *dst,
                        )
                        .expect("controlled native route staging");
                }
                None
            }
            Channel::Resolver => match network.resolve_provenance(bytes) {
                Ok(value) => Some(Provenance::Resolver(value)),
                Err(_) => {
                    network.audit.lock().unwrap().push(NetworkAudit::Rejected {
                        channel: sender.channel as i32,
                        reason: "invalid_resolver_payload",
                    });
                    return Unreliable::rejected();
                }
            },
            _ => return Unreliable::rejected(),
        };
        for dst in self.destinations {
            let packet = network
                .factory
                .packet(
                    dst,
                    sender.channel,
                    bytes.to_vec(),
                    priority,
                    provenance.clone(),
                )
                .expect("native packet staging");
            network.stage(packet);
        }
        Unreliable::new(Feedback::Ok)
    }
}
impl Blocker for Network {
    type PublicKey = PublicKey;
    fn block(&mut self, peer: PublicKey) -> Feedback {
        if self.scope.check().is_err() {
            return Feedback::Closed;
        }
        let Some(id) = self.peer_index(&peer) else {
            return Feedback::Closed;
        };
        let mut state = self.blocked.lock().unwrap();
        if state.peers.insert(peer) {
            let peers = Set::try_from(state.peers.iter().cloned().collect::<Vec<_>>())
                .expect("unique blocked peers");
            state
                .subscriptions
                .retain(|tx| tx.send_lossy(peers.clone()));
            self.audit
                .lock()
                .unwrap()
                .push(NetworkAudit::Block { peer: id });
        }
        Feedback::Ok
    }
    fn blocked(&mut self) -> BlockedSubscription<PublicKey> {
        let (tx, rx) = ring::channel(NonZeroUsize::new(1).unwrap());
        let mut state = self.blocked.lock().unwrap();
        let peers = Set::try_from(state.peers.iter().cloned().collect::<Vec<_>>())
            .expect("unique blocked peers");
        let _ = tx.send_lossy(peers);
        state.subscriptions.push(tx);
        rx
    }
}

pub(crate) struct ReceivedIngress {
    message: Message<PublicKey>,
    recovery: Option<(Arc<crate::native_recovery::ActorStore>, u32, u32, Id)>,
}

pub(crate) struct NativeRx(mpsc::Receiver<ReceivedIngress>);
impl std::fmt::Debug for NativeRx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NativeRx")
    }
}
impl Receiver for NativeRx {
    type Error = Error;
    type PublicKey = PublicKey;
    async fn recv(&mut self) -> Result<Message<PublicKey>> {
        use commonware_codec::Encode as _;
        let received = self.0.recv().await.ok_or(Error("native ingress closed"))?;
        if let Some((store, channel, source, reference)) = received.recovery {
            store
                .consumed(
                    channel,
                    source,
                    received.message.0.encode().to_vec(),
                    received.message.1.as_ref().to_vec(),
                    reference,
                )
                .expect("consume exact durable native ingress");
        }
        Ok(received.message)
    }
    fn capacity(&self) -> NonZeroUsize {
        NonZeroUsize::new(CAPACITY).unwrap()
    }
}
pub(crate) type Ingress = mpsc::Sender<ReceivedIngress>;
pub(crate) fn channels() -> ([Ingress; 3], [NativeRx; 3]) {
    let [(a, aa), (b, bb), (c, cc)] = std::array::from_fn(|_| mpsc::channel(CAPACITY));
    ([a, b, c], [NativeRx(aa), NativeRx(bb), NativeRx(cc)])
}
pub(crate) fn submit_input(
    channel: Channel,
    envelope: &Envelope,
    ingress: &[Ingress; 3],
    keys: &[PublicKey; 5],
    recovery: Option<(Arc<crate::native_recovery::ActorStore>, Id)>,
) -> bool {
    let index = (channel as usize)
        .checked_sub(1)
        .expect("nonempty native channel");
    ingress.get(index).is_some_and(|ingress| {
        ingress
            .try_send(ReceivedIngress {
                message: (
                    keys[envelope.src_member_id as usize].clone(),
                    IoBuf::from(envelope.payload_bytes.clone()),
                ),
                recovery: recovery
                    .map(|(store, id)| (store, channel as u32, envelope.src_member_id, id)),
            })
            .is_ok()
    })
}
