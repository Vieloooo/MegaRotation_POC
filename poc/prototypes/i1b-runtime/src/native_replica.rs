use crate::{
    ControlledDriver, Error, Event, Id, Result,
    driver::ArchiveAuthority,
    native_application::{App, AppAudit, Archive, Block},
    native_config::{Launch, NativeScheme, PARTITION, Resolved},
    native_elector::MemberElector,
    native_network::{self, Ingress, NativeRx, Network, NetworkAudit, PacketFactory},
    native_scheme::GuardedScheme,
    native_step_journal::StepJournal,
    native_wire::{
        self as proto, body_packet, effect, envelope::Provenance, hub_frame, input,
        native_run_client::NativeRunClient, worker_frame, *,
    },
    native_wire_support as wire,
};
use commonware_codec::{Copying, DecodeExt as _};
use commonware_consensus::{
    simplex::{
        Engine,
        config::{Config as EngineConfig, Floor, ForwardPolicy, SkipPolicy},
    },
    types::{Epoch, ViewDelta},
};
use commonware_cryptography::{
    bls12381::primitives::group::Private, ed25519::PublicKey, sha256::Digest,
};
use commonware_parallel::Sequential;
use commonware_runtime::{
    Supervisor,
    buffer::paged::{self, CacheRef},
};
use commonware_utils::{NZDuration, NZUsize};
use prost::Message;
use serde::Serialize;
use std::{
    fs::File,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

struct Replica {
    config: Arc<Resolved>,
    node: u32,
    native: NativeScheme,
    keys: [PublicKey; 5],
    receive: Option<[NativeRx; 3]>,
    ingress: [Ingress; 3],
    network: Arc<Mutex<Option<Network>>>,
    archive: Arc<Mutex<Archive>>,
    factory: PacketFactory,
    out: Arc<Mutex<Vec<Envelope>>>,
    audit: Arc<Mutex<Vec<NetworkAudit>>>,
    sign: Arc<Mutex<Vec<SignAudit>>>,
}
#[derive(Serialize)]
struct ApplicationEvents {
    work: Vec<AppAudit>,
    network: Vec<NetworkAudit>,
    signing: Vec<SignAudit>,
}

impl Replica {
    fn new(config: Arc<Resolved>, node: u32, native: NativeScheme) -> Result<Self> {
        let keys = config.public_keys()?;
        let (ingress, receive) = native_network::channels();
        Ok(Self {
            node,
            native,
            keys,
            receive: Some(receive),
            ingress,
            network: Arc::default(),
            archive: Arc::new(Mutex::new(Archive::new(config.clone()))),
            factory: PacketFactory::new(config.clone(), node),
            config,
            out: Arc::default(),
            audit: Arc::default(),
            sign: Arc::default(),
        })
    }
    fn start(&mut self, driver: &mut ControlledDriver) -> Result<()> {
        let config = self.config.clone();
        let native = self.native.clone();
        let sign = self.sign.clone();
        let node = self.node;
        let factory = self.factory.clone();
        let output = self.out.clone();
        let audit = self.audit.clone();
        let archive = self.archive.clone();
        let network_slot = self.network.clone();
        let [vote, certificate, resolver] = self
            .receive
            .take()
            .ok_or(Error("native engine already started"))?;
        archive.lock().unwrap().active = true;
        driver.spawn_session(config.cid, config.base.stop, move |context| async move {
            let network = Network::new(factory, context.authority("native-network"), output, audit)
                .expect("native network fixture");
            *network_slot.lock().unwrap() = Some(network.clone());
            let scheme = GuardedScheme::new(
                native,
                context.authority("native-signer"),
                config.cid,
                config.base.epoch,
                config.members[node as usize].signer_index,
                sign,
            );
            let application = App::new(context.child("application"), archive, network.clone());
            let duration = Duration::from_micros;
            let engine = Engine::new(
                context.child("engine"),
                EngineConfig {
                    scheme,
                    elector: MemberElector(std::array::from_fn(|member| {
                        config.members[member].signer_index
                    })),
                    blocker: network.clone(),
                    automaton: application.clone(),
                    relay: application.clone(),
                    reporter: application,
                    strategy: Sequential,
                    partition: PARTITION.into(),
                    mailbox_size: NZUsize!(64),
                    epoch: Epoch::new(config.base.epoch),
                    floor: Floor::Genesis(Digest(config.genesis)),
                    leader_timeout: duration(config.base.leader_timeout),
                    certification_timeout: duration(config.base.certification_timeout),
                    timeout_retry: duration(config.base.timeout_retry),
                    fetch_timeout: NZDuration!(duration(config.base.fetch_timeout)),
                    fetch_retry_delay: duration(config.base.fetch_retry),
                    view_retention: ViewDelta::new(config.base.view_retention),
                    skip: SkipPolicy::Disabled,
                    replay_buffer: NZUsize!(256),
                    write_buffer: NZUsize!(488),
                    page_cache: CacheRef::new(paged::page_size(256), NZUsize!(4)),
                    forward: ForwardPolicy::Disabled,
                    track_historical_votes: false,
                },
            );
            engine
                .start(
                    (network.sender(Channel::Vote), vote),
                    (network.sender(Channel::Certificate), certificate),
                    (network.sender(Channel::Resolver), resolver),
                )
                .await
                .expect("native engine completion");
        })
    }
    fn body_envelope(
        &self,
        authority: &ArchiveAuthority,
        dst: u32,
        digest: Id,
        id: u64,
        body: Vec<u8>,
    ) -> Result<()> {
        let network = self.network.lock().unwrap();
        if let Some(network) = network.as_ref() {
            let _ = network.send_archived_body(authority, dst, body, digest, id)?;
        }
        Ok(())
    }
    fn requests(&self, authority: &ArchiveAuthority, tick: u64) -> Result<()> {
        if self.network.lock().unwrap().is_none() || authority.check().is_err() {
            return Ok(());
        }
        let requests = self.archive.lock().unwrap().body_requests(tick);
        for (digest, id, request) in requests {
            for dest in 0..5 {
                if dest != self.node {
                    self.body_envelope(authority, dest, digest, id, request.clone())?;
                }
            }
        }
        Ok(())
    }
    fn deliver(&mut self, authority: &ArchiveAuthority, envelope: Envelope) -> Result<bool> {
        let channel = match native_network::validate_envelope(&self.config, &envelope) {
            Ok(channel) if envelope.dst_member_id == self.node => channel,
            _ => {
                self.archive
                    .lock()
                    .unwrap()
                    .events
                    .push(AppAudit::RejectedInput {
                        reason: "wrong_envelope_binding",
                    });
                return Ok(false);
            }
        };
        self.audit.lock().unwrap().push(NetworkAudit::Receive {
            channel: envelope.channel,
            source: envelope.src_member_id,
            attempt: envelope.attempt,
            message_id: envelope.message_id.clone(),
        });
        if channel == Channel::Body {
            return self.deliver_body(authority, envelope);
        }
        if channel == Channel::Certificate {
            let verifier = self.config.verifier()?;
            let cert = match native_network::decode_certificate(
                &verifier,
                self.config.base.epoch,
                &envelope.payload_bytes,
            ) {
                Ok(value) => value,
                Err(_) => {
                    self.archive
                        .lock()
                        .unwrap()
                        .events
                        .push(AppAudit::RejectedInput {
                            reason: "invalid_native_certificate",
                        });
                    return Ok(false);
                }
            };
            self.archive.lock().unwrap().proof(cert);
        }
        if self.archive.lock().unwrap().active
            && self
                .network
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|network| !network.blocked_member(envelope.src_member_id))
            && !native_network::submit_input(channel, &envelope, &self.ingress, &self.keys)
        {
            self.audit.lock().unwrap().push(NetworkAudit::Rejected {
                channel: channel as i32,
                reason: "receiver_capacity_or_closed",
            });
        }
        Ok(true)
    }
    fn deliver_body(&self, authority: &ArchiveAuthority, envelope: Envelope) -> Result<bool> {
        let decoded = match BodyPacket::decode(envelope.payload_bytes.as_slice()) {
            Ok(value) => value.kind,
            _ => None,
        };
        let meta = match envelope.provenance.as_ref() {
            Some(Provenance::Body(value)) => Some(value),
            _ => None,
        };
        let valid = match decoded {
            Some(body_packet::Kind::Push(push)) if push.cid == self.config.cid => {
                let block = Block::read(&self.config, &push.block_bytes);
                match block {
                    Ok(block)
                        if meta
                            .is_none_or(|p| p.content_id == block.digest && p.request_id == 0) =>
                    {
                        self.archive.lock().unwrap().accept_block(&block.bytes)?;
                        true
                    }
                    _ => false,
                }
            }
            Some(body_packet::Kind::Reply(reply))
                if reply.cid == self.config.cid && reply.request_id > 0 =>
            {
                match Block::read(&self.config, &reply.block_bytes) {
                    Ok(block)
                        if meta.is_none_or(|p| {
                            p.content_id == block.digest && p.request_id == reply.request_id
                        }) && self
                            .archive
                            .lock()
                            .unwrap()
                            .matches_body_request(&block.digest, reply.request_id) =>
                    {
                        self.archive.lock().unwrap().accept_block(&block.bytes)?;
                        true
                    }
                    _ => false,
                }
            }
            Some(body_packet::Kind::Request(request))
                if request.cid == self.config.cid && request.request_id > 0 =>
            {
                let digest: Id = match request.wanted_digest.try_into() {
                    Ok(d) => d,
                    Err(_) => return Ok(false),
                };
                if meta
                    .is_some_and(|p| p.content_id != digest || p.request_id != request.request_id)
                {
                    return Ok(false);
                }
                let block = self.archive.lock().unwrap().blocks.get(&digest).cloned();
                if let Some(block) = block {
                    let reply = BodyPacket {
                        kind: Some(body_packet::Kind::Reply(BodyReply {
                            request_id: request.request_id,
                            cid: self.config.cid.to_vec(),
                            block_bytes: block.bytes,
                        })),
                    };
                    self.body_envelope(
                        authority,
                        envelope.src_member_id,
                        digest,
                        request.request_id,
                        reply.encode_to_vec(),
                    )?;
                }
                true
            }
            _ => false,
        };
        if !valid {
            self.archive
                .lock()
                .unwrap()
                .events
                .push(AppAudit::InvalidBody);
        }
        Ok(valid)
    }
    fn stage_stress(&self, tick: u64) -> Result<()> {
        if self.node != 0 || self.config.base.stress_tick != Some(tick) {
            return Ok(());
        }
        let network = self.network.lock().unwrap();
        let network = network
            .as_ref()
            .ok_or(Error("stress before native network readiness"))?;
        for ordinal in 1..=self.config.base.stress_count {
            let id = u64::from(ordinal);
            let digest = crate::hash_parts(
                b"I1b/stress-content/v1",
                &[&self.config.cid, &id.to_be_bytes()],
            );
            let request = BodyPacket {
                kind: Some(body_packet::Kind::Request(BodyRequest {
                    request_id: id,
                    cid: self.config.cid.to_vec(),
                    wanted_digest: digest.to_vec(),
                })),
            };
            let _ = network.send_body(1, request.encode_to_vec(), digest, id)?;
        }
        Ok(())
    }
    async fn step(&mut self, driver: &mut ControlledDriver, grant: &proto::Grant) -> Result<Ack> {
        if grant.binding.as_ref() != Some(&self.config.binding(self.node))
            || grant.grant_hash != wire::grant_hash(grant)
            || grant.event_id == 0
            || grant.tie_rank != wire::tie_hash(self.config.base.seed, grant.event_id)
        {
            return Err(Error("invalid native grant binding"));
        }
        let kind = grant
            .input
            .as_ref()
            .and_then(|input| input.kind.clone())
            .ok_or(Error("empty native grant"))?;
        let cid = self.config.cid;
        let (input, action) = match kind {
            input::Kind::RequestSample(_) => (crate::Input::RequestSample, None),
            input::Kind::Sample(sample) => {
                (crate::Input::DeliverSample(clock_sample(sample)?), None)
            }
            input::Kind::TimerFired(timer) if timer.cid == cid => (
                crate::Input::Timer(crate::Timer {
                    id: timer.id,
                    cid,
                    deadline_us: timer.local_deadline_us,
                }),
                None,
            ),
            input::Kind::Close(close) if close.cid == cid => {
                (crate::Input::Close(cid), Some(input::Kind::Close(close)))
            }
            input::Kind::TimerFired(_) | input::Kind::Close(_) => {
                return Err(Error("native timer or close cid mismatch"));
            }
            other => (crate::Input::None, Some(other)),
        };
        let mut rejected = false;
        let mut started = false;
        self.archive.lock().unwrap().tick = grant.tick_us;
        let step = driver
            .grant_async(
                crate::Grant {
                    id: grant.logical_step_id,
                    tick_us: grant.tick_us,
                },
                input,
                |driver| {
                    let authority = driver.archive_authority(cid);
                    match action {
                        Some(input::Kind::Start(start)) => {
                            if start != self.config.start() || !driver.native_clock_ready() {
                                return Err(Error(
                                    "native start before calibrated readiness or wrong fixture",
                                ));
                            }
                            let now = driver.native_manager_time(cid)?;
                            if now >= self.config.base.stop {
                                return Err(Error("native manager reached local stop"));
                            }
                            self.start(driver)?;
                            started = true;
                        }
                        Some(input::Kind::Deliver(envelope)) => {
                            rejected = !self.deliver(&authority, envelope)?;
                        }
                        Some(input::Kind::WorkCompleted(work)) => {
                            self.archive.lock().unwrap().complete(&work)?
                        }
                        Some(input::Kind::Close(_)) => self.archive.lock().unwrap().close(),
                        Some(input::Kind::DriveReady(_)) => self.stage_stress(grant.tick_us)?,
                        _ => {}
                    }
                    self.requests(&authority, grant.tick_us)?;
                    Ok(())
                },
            )
            .await?;
        let mut effects = Vec::new();
        if started {
            effects.push(Effect {
                kind: Some(effect::Kind::Lifecycle(Lifecycle {
                    cid: cid.to_vec(),
                    event: "started".into(),
                    reason: "calibrated_material_ready".into(),
                })),
            });
        }
        for event in &step.events {
            let kind = match event {
                Event::SampleRequested { request } => {
                    Some(effect::Kind::ClockRequest(ClockRequest {
                        request_id: *request,
                    }))
                }
                Event::TimerArmed { timer } => Some(effect::Kind::TimerArm(timer_message(timer))),
                Event::TimerCancelled { timer } => {
                    Some(effect::Kind::TimerCancel(timer_message(timer)))
                }
                Event::Retired { cid, reason } => {
                    self.archive.lock().unwrap().close();
                    Some(effect::Kind::Lifecycle(Lifecycle {
                        cid: cid.to_vec(),
                        event: "terminated".into(),
                        reason: (*reason).into(),
                    }))
                }
                Event::SampleRejected { .. } => {
                    rejected = true;
                    None
                }
                _ => None,
            };
            if let Some(kind) = kind {
                effects.push(Effect { kind: Some(kind) });
            }
        }
        let mut archive = self.archive.lock().unwrap();
        archive.start_ready();
        effects.append(&mut archive.effects);
        let work = std::mem::take(&mut archive.events);
        drop(archive);
        let signing = std::mem::take(&mut *self.sign.lock().unwrap());
        for sign in &signing {
            effects.push(Effect {
                kind: Some(effect::Kind::Sign(sign.clone())),
            });
        }
        for mut envelope in std::mem::take(&mut *self.out.lock().unwrap()) {
            if envelope.provenance.is_none() {
                let wire = crate::NativeWire {
                    channel: envelope.channel as u32,
                    bytes: envelope.payload_bytes.clone(),
                };
                envelope.provenance = Some(Provenance::LocalWitness(
                    driver.native_witness_for(&wire, envelope.dst_member_id)?,
                ));
            }
            native_network::validate_envelope(&self.config, &envelope)?;
            effects.push(Effect {
                kind: Some(effect::Kind::Send(envelope)),
            });
        }
        let network = std::mem::take(&mut *self.audit.lock().unwrap());
        let witness = driver
            .witness_head()
            .ok_or(Error("missing native witness ledger"))?;
        Ok(Ack {
            binding: Some(self.config.binding(self.node)),
            logical_step_id: grant.logical_step_id,
            event_id: grant.event_id,
            grant_hash: grant.grant_hash.clone(),
            status: (if rejected {
                AckStatus::Rejected
            } else {
                AckStatus::Quiescent
            }) as i32,
            effects,
            runtime_events_json: serde_json::to_vec(&step.events)
                .map_err(|_| Error("native runtime audit encoding"))?,
            application_events_json: serde_json::to_vec(&ApplicationEvents {
                work,
                network,
                signing,
            })
            .map_err(|_| Error("native application audit encoding"))?,
            witness_receipt: Some(Receipt {
                sequence: witness.sequence,
                hash: witness.hash.to_vec(),
            }),
            ..Default::default()
        })
    }
}

fn timer_message(timer: &crate::Timer) -> proto::Timer {
    proto::Timer {
        id: timer.id,
        cid: timer.cid.to_vec(),
        local_deadline_us: timer.deadline_us,
    }
}
fn id(bytes: Vec<u8>) -> Result<Id> {
    bytes
        .try_into()
        .map_err(|_| Error("native 32-byte id required"))
}
fn clock_sample(value: proto::ClockSample) -> Result<crate::ClockSample> {
    Ok(crate::ClockSample {
        run_id: id(value.run_id)?,
        node: value.worker_id,
        incarnation: value.incarnation,
        clock_domain: id(value.clock_domain)?,
        rate_profile: id(value.rate_profile)?,
        request_id: value.request_id,
        sequence: value.sequence,
        as_of_tick_us: value.as_of_tick_us,
        node_time_at_as_of_us: value.node_time_at_as_of_us,
        uncertainty_us: value.uncertainty_us,
        lease_end_tick_us: value.lease_end_tick_us,
    })
}

pub fn run_worker(
    endpoint: &str,
    launch_file: &Path,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let launch: Launch = serde_json::from_reader(File::open(launch_file)?)?;
    if launch.member > 1 || launch.wall_pause_ms > 2 {
        return Err(Box::new(Error("invalid native worker configuration")));
    }
    launch.resolved.validate()?;
    let secret = Private::decode(Copying(&launch.secret))
        .map_err(|_| Error("invalid native private key"))?;
    let config = Arc::new(launch.resolved);
    let native = config.signer(launch.member, secret)?;
    let mut replica = Replica::new(config.clone(), launch.member, native)?;
    let root = launch.work_dir;
    let mut steps = StepJournal::create(&root, &config, launch.member)?;
    let profile = config.profile(launch.member);
    ControlledDriver::run_async(
        commonware_runtime::tokio::Config::default().with_storage_directory(root.join("native")),
        profile.clone(),
        async |driver| {
            driver.enable_native_clock_audit();
            driver.configure_witness(config.witness(launch.member)?, &root.join("witness"))?;
            let mut client = NativeRunClient::connect(endpoint.to_owned())
                .await
                .map_err(|_| Error("native worker connect failed"))?
                .max_decoding_message_size(wire::MAX_FRAME)
                .max_encoding_message_size(wire::MAX_FRAME);
            let register = config.registration(launch.member);
            let reply = client
                .register(register.clone())
                .await
                .map_err(|_| Error("native register rejected"))?
                .into_inner();
            if reply.binding != register.binding || reply.clock_domain != profile.clock_domain() {
                return Err(Error("wrong native register reply"));
            }
            let (send, receive) = mpsc::channel(4);
            send.send(WorkerFrame {
                frame: Some(worker_frame::Frame::Hello(register)),
            })
            .await
            .map_err(|_| Error("native stream Hello failed"))?;
            let mut stream = client
                .run(ReceiverStream::new(receive))
                .await
                .map_err(|_| Error("native stream open failed"))?
                .into_inner();
            while let Some(frame) = stream
                .message()
                .await
                .map_err(|_| Error("native stream disconnected"))?
            {
                match frame.frame.ok_or(Error("empty native hub frame"))? {
                    hub_frame::Frame::Grant(grant) => {
                        if launch.wall_pause_ms != 0 {
                            tokio::time::sleep(Duration::from_millis(launch.wall_pause_ms)).await;
                        }
                        let ack = if let Some(ack) = steps.cached(&grant)? {
                            ack
                        } else {
                            let ack = replica.step(driver, &grant).await?;
                            steps.commit(grant, ack)?
                        };
                        send.send(WorkerFrame {
                            frame: Some(worker_frame::Frame::Ack(ack)),
                        })
                        .await
                        .map_err(|_| Error("native worker Ack channel closed"))?;
                    }
                    hub_frame::Frame::Shutdown(stop) if stop.reason == "complete" => return Ok(()),
                    _ => return Err(Error("unexpected native worker shutdown")),
                }
            }
            Err(Error("native worker stream closed without shutdown"))
        },
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_config::{START, WORK_US};
    use std::fs;

    #[test]
    fn actual_engine_granted_proposal_yields_witnessed_vote_after_work() {
        // Debug native Engine futures exceed Rust's default test-thread stack.
        std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(actual_engine_granted_proposal)
            .unwrap()
            .join()
            .unwrap();
    }

    fn actual_engine_granted_proposal() {
        let (config, mut keys) = Resolved::generate("healthy", 7).unwrap();
        let native = config.signer(0, keys.remove(0)).unwrap();
        let config = Arc::new(config);
        let mut replica = Replica::new(config.clone(), 0, native).unwrap();
        let profile = config.profile(0);
        let root = (0..u64::MAX)
            .map(|i| std::env::temp_dir().join(format!("native-voter-{}-{i}", std::process::id())))
            .find(|path| fs::create_dir(path).is_ok())
            .unwrap();
        let mut journal = StepJournal::create(&root, &config, 0).unwrap();
        ControlledDriver::run_async(
            commonware_runtime::tokio::Config::default().with_storage_directory(root.join("native")),profile.clone(),
            async |driver| {
                driver.configure_witness(config.witness(0)?,&root.join("witness"))?;
                driver.enable_native_clock_audit();
                let grant=|step,tick,input| {
                    let mut grant=proto::Grant{
                        binding:Some(config.binding(0)),logical_step_id:step,event_id:step,tick_us:tick,
                        causal_wave:0,tie_rank:wire::tie_hash(7,step).to_vec(),
                        input:Some(proto::Input{kind:Some(input)}),grant_hash:vec![]
                    };
                    grant.grant_hash=wire::grant_hash(&grant).to_vec(); grant
                };
                let a=grant(1,START,input::Kind::RequestSample(Empty{}));
                let ack=replica.step(driver,&a).await?;
                journal.commit(a,ack)?;
                let b=grant(2,START+500,input::Kind::Sample(ClockSample{
                    run_id:config.run_id.to_vec(),worker_id:0,incarnation:1,
                    clock_domain:profile.clock_domain().to_vec(),rate_profile:crate::Profile::rate_profile().to_vec(),
                    request_id:1, sequence:1, as_of_tick_us:START+250,
                    node_time_at_as_of_us:START as i64+1250,uncertainty_us:1000,lease_end_tick_us:START+100_250,
                }));
                let ack=replica.step(driver,&b).await?;
                journal.commit(b,ack)?;
                let c=grant(3,START+500,input::Kind::Start(config.start()));
                let ack=replica.step(driver,&c).await?;
                assert!(!ack.effects.iter().any(|e|matches!(e.kind,Some(effect::Kind::Sign(_)))));
                let work=ack.effects.iter().find_map(|e|match &e.kind {
                    Some(effect::Kind::Work(work)) if work.kind==WorkKind::Propose as i32=>Some(work.clone()),_=>None,
                }).expect("native automaton work");
                journal.commit(c,ack)?;
                let d=grant(4,START+500+WORK_US,input::Kind::WorkCompleted(work));
                let ack=replica.step(driver,&d).await?;
                assert!(ack.effects.iter().any(|e|matches!(&e.kind,Some(effect::Kind::Sign(s)) if s.action==SignAction::Created as i32)));
                let votes:Vec<_>=ack.effects.iter().filter_map(|e|match &e.kind{
                    Some(effect::Kind::Send(packet)) if packet.channel==Channel::Vote as i32=>Some(packet),
                    _=>None
                }).collect();
                assert_eq!(votes.len(),4);
                assert!(votes.iter().all(|packet|packet.attempt==1 && matches!(packet.provenance,Some(Provenance::LocalWitness(_)))));
                let sealed=journal.commit(d.clone(),ack)?;
                assert_eq!(journal.cached(&d)?.unwrap(),sealed);
                Ok(())
            }
        ).unwrap();
        drop(journal);
        fs::remove_dir_all(root).unwrap();
    }
}
