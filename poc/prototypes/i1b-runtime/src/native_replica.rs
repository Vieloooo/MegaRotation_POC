use crate::{
    ControlledDriver, Error, Event, Id, Result,
    driver::ArchiveAuthority,
    native_application::{App, AppAudit, Archive, Block},
    native_config::{Launch, NativeScheme, PARTITION, RecoveryLaunch, Resolved},
    native_elector::MemberElector,
    native_network::{self, Ingress, NativeRx, Network, NetworkAudit, PacketFactory},
    native_recovery::{
        self as recovery_driver, ActorStore, EmptyCutContext, GrantIntent, LedgerWriter,
        StartupFence,
    },
    native_recovery_ledger::{Identity as AuxIdentity, RecoveryLedger},
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
        recovery::{RecoveryOptions, RecoveryStartup, RecoveryStore},
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
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Clone)]
struct EngineRecovery {
    store: Arc<dyn RecoveryStore>,
    startup: Arc<dyn RecoveryStartup>,
    session_not_before: Option<std::time::SystemTime>,
}

struct RecoveryGrant<'a> {
    store: &'a Arc<crate::native_recovery::ActorStore>,
    input_ref: Id,
    start: Option<EngineRecovery>,
    expected_recover: Option<&'a RecoverEngine>,
    rngs: &'a [(Id, crate::RngCheckpoint)],
    release: Option<(
        &'a StartupFence,
        &'a LedgerWriter,
        commonware_consensus::simplex::recovery::RecoveryReceipt,
    )>,
}

struct Replica {
    config: Arc<Resolved>,
    node: u32,
    incarnation: u64,
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
            incarnation: 1,
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
        self.start_engine(driver, None)
    }
    fn start_engine(
        &mut self,
        driver: &mut ControlledDriver,
        recovery: Option<EngineRecovery>,
    ) -> Result<()> {
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
            let recovery_store = recovery.as_ref().map(|hooks| hooks.store.clone());
            let application = App::new(context.child("application"), archive, network.clone())
                .with_recovery(recovery_store);
            let duration = Duration::from_micros;
            let engine_config = EngineConfig {
                scheme,
                elector: MemberElector(std::array::from_fn(|member| {
                    config.members[member].signer_index
                })),
                blocker: network.clone(),
                automaton: application.clone(),
                relay: application.clone(),
                reporter: application.clone(),
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
            };
            let engine_context = context.child("engine");
            let engine = match recovery {
                Some(recovery) => Engine::new_with_recovery(
                    engine_context,
                    engine_config,
                    RecoveryOptions {
                        store: recovery.store,
                        startup: recovery.startup,
                        reconnector: Arc::new(application),
                        session_not_before: recovery.session_not_before,
                    },
                ),
                None => Engine::new(engine_context, engine_config),
            };
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
    fn deliver(
        &mut self,
        authority: &ArchiveAuthority,
        envelope: Envelope,
        recovery: Option<(Arc<crate::native_recovery::ActorStore>, Id)>,
    ) -> Result<bool> {
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
            && !native_network::submit_input(
                channel,
                &envelope,
                &self.ingress,
                &self.keys,
                recovery,
            )
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
        self.step_controlled(driver, grant, None).await
    }
    async fn step_controlled(
        &mut self,
        driver: &mut ControlledDriver,
        grant: &proto::Grant,
        recovery: Option<RecoveryGrant<'_>>,
    ) -> Result<Ack> {
        let mut binding = self.config.binding(self.node);
        binding.incarnation = self.incarnation;
        if grant.binding.as_ref() != Some(&binding)
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
            input::Kind::Recover(value)
                if recovery.as_ref().and_then(|r| r.expected_recover) == Some(&value) =>
            {
                (crate::Input::None, Some(input::Kind::Recover(value)))
            }
            input::Kind::Recover(_) | input::Kind::RecoveryWork(_) | input::Kind::ArmFault(_) => {
                return Err(Error("Fresh rejects native recovery input"));
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
                            if let Some(options) = recovery.as_ref().and_then(|r| r.start.clone()) {
                                self.start_engine(driver, Some(options))?;
                            } else if recovery.is_none() {
                                self.start(driver)?;
                            } else {
                                return Err(Error(
                                    "recovery-mode Start lacks verified initial material",
                                ));
                            }
                            started = true;
                        }
                        Some(input::Kind::Deliver(envelope)) => {
                            rejected = !self.deliver(
                                &authority,
                                envelope,
                                recovery.as_ref().map(|r| (r.store.clone(), r.input_ref)),
                            )?;
                        }
                        Some(input::Kind::Recover(_)) => {
                            if !driver.native_clock_ready()
                                || driver.native_manager_time(cid)? >= self.config.base.stop
                            {
                                return Err(Error(
                                    "native recovery start lacks safe new calibration",
                                ));
                            }
                            let options =
                                recovery
                                    .as_ref()
                                    .and_then(|r| r.start.clone())
                                    .ok_or(Error(
                                        "authorized RecoverEngine lacks validated startup options",
                                    ))?;
                            for (role, rng) in recovery.as_ref().expect("authorized recovery").rngs
                            {
                                driver.restore_rng(cid, *role, rng.clone())?;
                            }
                            self.start_engine(driver, Some(options))?;
                        }
                        Some(input::Kind::WorkCompleted(work)) => {
                            self.archive.lock().unwrap().complete(&work)?
                        }
                        Some(input::Kind::Close(_)) => self.archive.lock().unwrap().close(),
                        Some(input::Kind::DriveReady(_)) => {
                            if let Some((fence, writer, receipt)) =
                                recovery.as_ref().and_then(|r| r.release)
                            {
                                fence.release(writer, receipt)?;
                            } else {
                                self.stage_stress(grant.tick_us)?;
                            }
                        }
                        _ => {}
                    }
                    if recovery.as_ref().is_none_or(|r| r.release.is_none()) {
                        self.requests(&authority, grant.tick_us)?;
                    }
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
            binding: Some(binding),
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

struct ResumeMaterial {
    cut: recovery_driver::ResumeCut,
    expected: RecoverEngine,
    recovery_tick: u64,
    witness_path: PathBuf,
    witness_policy: crate::WitnessPolicy,
    witness_head: crate::WitnessHead,
}
struct PreparedWorker {
    storage: PathBuf,
    journal: StepJournal,
    writer: Option<LedgerWriter>,
    registration: RegisterRequest,
    resume: Option<ResumeMaterial>,
}
fn prepare_worker(
    config: &Resolved,
    node: u32,
    root: &Path,
    recovery: Option<RecoveryLaunch>,
) -> Result<PreparedWorker> {
    use std::fs;
    let io = |_| Error("native recovery launch filesystem error");
    let target = config
        .base
        .recovery
        .as_ref()
        .is_some_and(|scope| scope.target_worker == node);
    if target != recovery.is_some() {
        return Err(Error("native recovery launch does not match scope"));
    }
    let aux_id = AuxIdentity {
        run: config.run_id,
        config: config.config_hash,
        worker: node,
    };
    let mut registration = config.registration(node);
    let mut storage = root.join("native");
    let (journal, writer, resume) = match recovery {
        None => (StepJournal::create(root, config, node)?, None, None),
        Some(RecoveryLaunch::Initial) => {
            fs::create_dir(&storage).map_err(io)?;
            File::open(root)
                .and_then(|parent| parent.sync_all())
                .map_err(io)?;
            (
                StepJournal::create(root, config, node)?,
                Some(LedgerWriter::new(RecoveryLedger::create(root, aux_id)?)),
                None,
            )
        }
        Some(RecoveryLaunch::Resume {
            source,
            fresh_target,
            floor,
            old_incarnation,
            storage_generation,
            recovery_tick_us,
        }) => {
            let next = old_incarnation
                .checked_add(1)
                .ok_or(Error("native incarnation exhausted"))?;
            if old_incarnation != 1
                || storage_generation != 1
                || node != 1
                || source.parent() != root.parent()
                || fresh_target.parent() != Some(root)
                || source == root
                || !fs::symlink_metadata(&source)
                    .map_err(io)?
                    .file_type()
                    .is_dir()
            {
                return Err(Error("untrusted empty recovery storage coordinates"));
            }
            for name in [
                "native-aux",
                "native-aux.head",
                "root-step",
                "root-step.head",
            ] {
                let from = source.join(name);
                if !fs::symlink_metadata(&from)
                    .map_err(io)?
                    .file_type()
                    .is_file()
                {
                    return Err(Error("source recovery ledger must be regular"));
                }
                let mut input = File::open(from).map_err(io)?;
                let mut output = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join(name))
                    .map_err(io)?;
                std::io::copy(&mut input, &mut output).map_err(io)?;
                output.sync_all().map_err(io)?;
            }
            File::open(root)
                .and_then(|file| file.sync_all())
                .map_err(io)?;
            let root_floor = recovery_driver::read_wire_receipt(floor.root_receipt.as_ref())?;
            let aux_floor = recovery_driver::read_wire_receipt(floor.auxiliary_receipt.as_ref())?;
            let witness_floor = recovery_driver::read_wire_receipt(floor.witness_receipt.as_ref())?;
            let journal = StepJournal::open_recovery(root, config, node, root_floor, None)?;
            let ledger = RecoveryLedger::open(root, aux_id, aux_floor, None)?;
            let cut = recovery_driver::validate_resume_empty_cut(
                &ledger, config, node, &floor, &journal,
            )?;
            let authorized = recovery_driver::AuthorizedEmptyBoundary::from_validated_cut(
                &ledger,
                aux_floor,
                cut.bundle.cut,
                crate::WitnessHead {
                    sequence: witness_floor.sequence,
                    hash: witness_floor.hash,
                },
                storage_generation,
            )?;
            let from = config.witness(node)?;
            let mut policy = from.clone();
            policy.seal.config.generation = next;
            let report = crate::witness_recovery::recover_empty_witness_generation(
                crate::RecoveryPaths {
                    source_storage: &source.join("native"),
                    source_ledger: &source.join("witness"),
                    fresh_target: &fresh_target,
                },
                from,
                policy.clone(),
                &authorized,
            )?;
            registration =
                config.resumed_registration(node, old_incarnation, next, floor.clone())?;
            storage = fresh_target.join("native");
            let expected = RecoverEngine {
                floor: Some(floor),
                engine: Some(config.start()),
                old_incarnation,
                old_generation: storage_generation,
                new_generation: next,
                outstanding_grant_hash: vec![],
                recovery_profile_hash: config.config_hash.to_vec(),
            };
            (
                journal,
                Some(LedgerWriter::new(ledger)),
                Some(ResumeMaterial {
                    cut,
                    expected,
                    recovery_tick: recovery_tick_us,
                    witness_path: fresh_target.join("witness"),
                    witness_policy: policy,
                    witness_head: report.new_head,
                }),
            )
        }
    };
    Ok(PreparedWorker {
        storage,
        journal,
        writer,
        registration,
        resume,
    })
}

struct ProcessRecovery {
    writer: LedgerWriter,
    store: Arc<ActorStore>,
    fence: Option<Arc<StartupFence>>,
    expected: Option<RecoverEngine>,
    rngs: Vec<(Id, crate::RngCheckpoint)>,
    input_sequence: u64,
    predecessor: Id,
    source_input: Option<commonware_consensus::simplex::recovery::OperationRecord>,
    old_lease_end: Option<u64>,
    restored: bool,
}
impl ProcessRecovery {
    async fn step(
        &mut self,
        replica: &mut Replica,
        driver: &mut ControlledDriver,
        root: &mut StepJournal,
        grant: proto::Grant,
    ) -> Result<Ack> {
        use commonware_consensus::simplex::recovery::ActorId;
        let config = replica.config.clone();
        let cid = config.cid;
        if self.old_lease_end.is_some_and(|end| grant.tick_us >= end) {
            return Err(Error("original calibrated lease expired before startup"));
        }

        let coords = crate::Grant {
            id: grant.logical_step_id,
            tick_us: grant.tick_us,
        };
        let kind = grant
            .input
            .as_ref()
            .and_then(|i| i.kind.as_ref())
            .ok_or(Error("missing recovery input"))?;
        if self.fence.is_some() && self.old_lease_end.is_some() {
            let permitted = match kind {
                input::Kind::RequestSample(_) | input::Kind::Sample(_) => !self.restored,
                input::Kind::Recover(recover) => {
                    !self.restored && self.expected.as_ref() == Some(recover)
                }
                input::Kind::DriveReady(_) => self.restored,
                _ => false,
            };
            if !permitted {
                return Err(Error(
                    "bounded recovery startup requires an exact authorized input",
                ));
            }
        }
        let (start, releasing) = match (&self.fence, kind) {
            (None, input::Kind::Start(_)) => (
                Some(EngineRecovery {
                    store: self.store.clone(),
                    startup: Arc::new(recovery_driver::InitialStartup),
                    session_not_before: None,
                }),
                false,
            ),
            (Some(fence), input::Kind::Recover(recover))
                if self.expected.as_ref() == Some(recover) && !self.restored =>
            {
                (
                    Some(EngineRecovery {
                        store: self.store.clone(),
                        startup: fence.clone(),
                        session_not_before: None,
                    }),
                    false,
                )
            }
            (Some(_), input::Kind::DriveReady(_)) if self.restored => (None, true),
            (Some(_), input::Kind::Start(_)) => {
                return Err(Error("restored process cannot accept Fresh Start"));
            }
            _ => (None, false),
        };
        self.input_sequence = self
            .input_sequence
            .checked_add(1)
            .ok_or(Error("recovery input counter exhausted"))?;
        let (input_ref, intent) = GrantIntent::from_grant(
            &config,
            replica.node,
            replica.incarnation,
            self.input_sequence,
            &grant,
            root.receipt(),
            self.predecessor,
            driver.recovery_local_at(grant.tick_us),
        )?;
        self.store
            .write_ahead(driver, coords, input_ref, intent)
            .await?;
        let mut ready = None;
        let release = if releasing {
            let fence = self.fence.as_ref().unwrap();
            let op = self
                .source_input
                .as_ref()
                .ok_or(Error("missing restored Verify ownership"))?;
            if !replica
                .archive
                .lock()
                .unwrap()
                .live_recovered_work(op.operation_id)
                || !driver.native_clock_ready()
            {
                return Err(Error(
                    "startup lacks live restored pending Verify or calibration",
                ));
            }
            let floor = self.expected.as_ref().unwrap().floor.as_ref().unwrap();
            let cut = id(floor.cut_hash.clone())?;
            let body = serde_json::to_vec(&driver.recovery_clock_checkpoint(cid)?)
                .map_err(|_| Error("clock audit encoding"))?;
            for (actor, hash) in [
                (
                    ActorId::Application,
                    crate::hash_parts(b"I1b/recovery-state/v1", &[&op.original_work_binding]),
                ),
                (
                    ActorId::Archive,
                    crate::hash_parts(
                        b"I1b/recovery-state/v1",
                        &[&op.operation_id, &config.genesis],
                    ),
                ),
                (
                    ActorId::Driver,
                    crate::hash_parts(b"I1b/recovery-state/v1", &[&body]),
                ),
            ] {
                fence.ready(actor, cut, hash).map_err(|e| Error(e.0))?;
            }
            let token = fence.stage_release(&self.writer)?;
            let complete = self.writer.flush_root(driver, coords, cid, token).await?;
            let witness = driver
                .witness_head()
                .ok_or(Error("startup witness missing"))?;
            ready = Some(RecoveryReady {
                cid: cid.to_vec(),
                cut_hash: cut.to_vec(),
                actors: fence.actor_reports(),
                auxiliary_receipt: Some(recovery_driver::wire_receipt(complete)),
                witness_receipt: Some(Receipt {
                    sequence: witness.sequence,
                    hash: witness.hash.to_vec(),
                }),
                terminal: false,
            });
            Some((fence.as_ref(), &self.writer, complete))
        } else {
            None
        };
        let mut ack = replica
            .step_controlled(
                driver,
                &grant,
                Some(RecoveryGrant {
                    store: &self.store,
                    input_ref,
                    start,
                    expected_recover: self.expected.as_ref(),
                    rngs: &self.rngs,
                    release,
                }),
            )
            .await?;
        if matches!(kind, input::Kind::Recover(_)) {
            self.restored = true;
        }
        let (work_generation, active) = {
            let archive = replica.archive.lock().unwrap();
            (archive.empty_archive_frontier()?, archive.active)
        };
        if let Some(network) = replica.network.lock().unwrap().as_ref() {
            network.empty_network_frontier()?;
        }
        let (cut, receipt) = recovery_driver::checkpoint_empty_trajectory(
            &self.writer,
            driver,
            EmptyCutContext {
                config: &config,
                worker: replica.node,
                generation: replica.incarnation,
                sequence: root
                    .receipt()
                    .sequence
                    .checked_add(1)
                    .ok_or(Error("root step sequence exhausted"))?,
                previous: self.predecessor,
                work_generation,
                active,
                terminal: false,
            },
            &grant,
            &ack,
        )
        .await?;
        self.predecessor = cut;
        if let Some(ready) = ready {
            self.old_lease_end = None;
            ack.effects.push(Effect {
                kind: Some(effect::Kind::RecoveryReady(ready)),
            });
        }
        ack.recovery_receipt = Some(RecoveryCutReceipt {
            cut_hash: cut.to_vec(),
            auxiliary: Some(recovery_driver::wire_receipt(receipt)),
        });
        root.commit(grant, ack)
    }
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
    let PreparedWorker {
        storage,
        journal: mut steps,
        writer,
        registration,
        resume,
    } = prepare_worker(&config, launch.member, &root, launch.recovery)?;
    let mut profile = config.profile(launch.member);
    profile.incarnation = registration.binding.as_ref().unwrap().incarnation;
    replica.incarnation = profile.incarnation;
    ControlledDriver::run_async(
        commonware_runtime::tokio::Config::default().with_storage_directory(storage),
        profile.clone(),
        async |driver| {
            use commonware_consensus::simplex::recovery::RecoveryBundle;
            driver.enable_native_clock_audit();
            let mut process_recovery = match (writer, resume) {
                (Some(writer), Some(resume)) => {
                    driver.install_recovered_clock(
                        &resume.cut.clock,
                        resume.recovery_tick,
                        config.base.stop,
                    )?;
                    driver.restore_timer_frontier(resume.cut.last_timer)?;
                    driver.configure_recovered_witness(
                        resume.witness_policy,
                        &resume.witness_path,
                        resume.witness_head,
                    )?;
                    replica
                        .archive
                        .lock()
                        .unwrap()
                        .restore_work_frontier(resume.cut.work_generation)?;
                    let cut = resume.cut.bundle.cut;
                    let source_input = resume.cut.bundle.operations.first().cloned();
                    let store = Arc::new(ActorStore::new(
                        writer.clone(),
                        driver.recovery_authority(config.cid),
                        config.base.epoch,
                        resume.cut.bundle,
                        vec![],
                    ));
                    Some(ProcessRecovery {
                        writer,
                        store,
                        fence: Some(Arc::new(StartupFence::new(cut))),
                        expected: Some(resume.expected),
                        rngs: resume.cut.rngs,
                        input_sequence: resume.cut.input_sequence,
                        predecessor: cut,
                        source_input,
                        old_lease_end: resume.cut.clock.old_lease_end(),
                        restored: false,
                    })
                }
                (Some(writer), None) => {
                    driver
                        .configure_witness(config.witness(launch.member)?, &root.join("witness"))?;
                    let store = Arc::new(ActorStore::new(
                        writer.clone(),
                        driver.recovery_authority(config.cid),
                        config.base.epoch,
                        RecoveryBundle {
                            cut: [0; 32],
                            timing: None,
                            proposals: vec![],
                            operations: vec![],
                            actor_phase: None,
                        },
                        vec![],
                    ));
                    Some(ProcessRecovery {
                        writer,
                        store,
                        fence: None,
                        expected: None,
                        rngs: vec![],
                        input_sequence: 0,
                        predecessor: [0; 32],
                        source_input: None,
                        old_lease_end: None,
                        restored: false,
                    })
                }
                (None, None) => {
                    driver
                        .configure_witness(config.witness(launch.member)?, &root.join("witness"))?;
                    None
                }
                _ => return Err(Error("resumed process has no shared recovery ledger")),
            };
            let mut client = NativeRunClient::connect(endpoint.to_owned())
                .await
                .map_err(|_| Error("native worker connect failed"))?
                .max_decoding_message_size(wire::MAX_FRAME)
                .max_encoding_message_size(wire::MAX_FRAME);
            let register = registration;
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
                        } else if let Some(lane) = process_recovery.as_mut() {
                            lane.step(&mut replica, driver, &mut steps, grant).await?
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
                let first = journal.commit(a,ack)?;
                // Independent golden: Fresh formal fec7ade, healthy seed7, worker0 record1.
                assert_eq!(
                    crate::native_recovery_ledger::hex(&first.durable_step_hash),
                    "17a0801a09ecf0fe217fd0cb4b9469485521512e7ee0eb04df31f899b4933166",
                );
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

#[cfg(test)]
mod recovery_tests {
    use super::*;
    use crate::{
        native_config::{START, WORK_US},
        native_recovery::{
            ActorStore, GrantIntent, LedgerWriter, StartupFence, fixture::Hooks,
            load_pending_verify_fragment,
        },
        native_recovery_ledger::{Identity, RecoveryLedger},
    };
    use commonware_consensus::simplex::{recovery as r, types::Vote};
    use std::fs;

    #[test]
    fn native_pending_verify_reconnects_with_original_certificate_deadline() {
        std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(pending_verify)
            .unwrap()
            .join()
            .unwrap();
    }

    fn pending_verify() {
        let (config, mut keys) = Resolved::generate("healthy", 7).unwrap();
        let receiver = config.signer(1, keys.remove(1)).unwrap();
        let leader = config.signer(0, keys.remove(0)).unwrap();
        let config = Arc::new(config);
        let root = (0..u64::MAX)
            .map(|i| {
                std::env::temp_dir().join(format!("native-reconnect-{}-{i}", std::process::id()))
            })
            .find(|path| fs::create_dir(path).is_ok())
            .unwrap();
        let empty = r::RecoveryBundle {
            cut: [0; 32],
            timing: None,
            proposals: vec![],
            operations: vec![],
            actor_phase: None,
        };
        let tick = START + 500;
        let voted_at = START + 24_000;
        let original_deadline = tick as i64 - 1000 + 400_000;
        let mut replica = Replica::new(config.clone(), 1, receiver.clone()).unwrap();
        let packet = produce_leader_vote(&root, config.clone(), leader);
        let Vote::Notarize(vote) = native_network::decode_vote(&packet.payload_bytes).unwrap()
        else {
            panic!("expected actual witnessed leader Notarize");
        };
        let proposal = vote.proposal;
        let deliver = grant(
            &config,
            1,
            4,
            voted_at,
            input::Kind::Deliver(packet.clone()),
        );
        let directory = root.join("initial-native");
        let identity = Identity {
            run: config.run_id,
            config: config.config_hash,
            worker: 1,
        };
        let writer = LedgerWriter::new(RecoveryLedger::create(&root, identity).unwrap());
        let (initial_step, receipt, input_ref) = ControlledDriver::run_async(
            commonware_runtime::tokio::Config::default().with_storage_directory(directory.clone()),
            config.profile(1),
            async |driver| {
                driver.configure_witness(config.witness(1)?, &root.join("initial-witness"))?;
                sample(&mut replica, driver, &config, 1, tick).await?;
                let gate = Arc::new(Hooks::new(empty.cut));
                let source = Arc::new(ActorStore::new(
                    writer.clone(),
                    driver.recovery_authority(config.cid),
                    config.base.epoch,
                    empty,
                    vec![],
                ));
                let start = grant(&config, 1, 3, tick, input::Kind::Start(config.start()));
                let (start_id, intent) = GrantIntent::from_grant(
                    &config,
                    1,
                    1,
                    1,
                    &start,
                    r::RecoveryReceipt::default(),
                    [0; 32],
                    Some(tick as i64 - 1000),
                )?;
                let start_grant = crate::Grant {
                    id: 3,
                    tick_us: tick,
                };
                source
                    .write_ahead(driver, start_grant, start_id, intent)
                    .await?;
                replica.archive.lock().unwrap().tick = tick;
                driver
                    .grant_async(start_grant, crate::Input::None, |driver| {
                        replica.start_engine(
                            driver,
                            Some(EngineRecovery {
                                store: source.clone(),
                                startup: gate,
                                session_not_before: None,
                            }),
                        )
                    })
                    .await?;
                let (reference, intent) = GrantIntent::from_grant(
                    &config,
                    1,
                    1,
                    2,
                    &deliver,
                    r::RecoveryReceipt::default(),
                    [0; 32],
                    Some(voted_at as i64 - 1000),
                )?;
                let delivered = crate::Grant {
                    id: 4,
                    tick_us: voted_at,
                };
                source
                    .write_ahead(driver, delivered, reference, intent)
                    .await?;
                replica.archive.lock().unwrap().tick = voted_at;
                let captured = driver
                    .grant_async(delivered, crate::Input::None, |_| {
                        if !native_network::submit_input(
                            Channel::Vote,
                            &packet,
                            &replica.ingress,
                            &replica.keys,
                            Some((source.clone(), reference)),
                        ) {
                            return Err(Error("controlled fixture ingress unavailable"));
                        }
                        Ok(())
                    })
                    .await?;
                assert!(
                    !replica
                        .archive
                        .lock()
                        .unwrap()
                        .blocks
                        .contains_key(&proposal.payload.0)
                );
                assert!(replica.sign.lock().unwrap().is_empty());
                let clock = driver.recovery_clock_checkpoint(config.cid)?;
                writer.stage(
                    "ROLE_RNG_CHECKPOINTS",
                    &driver.capture_role_rngs(config.cid)?,
                )?;
                writer.stage("CLOCK_CHECKPOINT", &clock)?;
                let receipt = writer
                    .flush_root(driver, delivered, config.cid, writer.frontier())
                    .await?;
                Ok((captured, receipt, reference))
            },
        )
        .unwrap();
        assert!(initial_step.events.iter().any(|event| matches!(event,
            Event::TimerArmed {timer} if timer.deadline_us == original_deadline)));
        assert!(directory.is_dir());
        assert!(!directory.join(PARTITION).exists());
        let opened = RecoveryLedger::open(&root, identity, receipt, None).unwrap();
        let names = opened
            .records()
            .iter()
            .map(|record| record.header.kind.as_str())
            .collect::<Vec<_>>();
        let selection = names
            .iter()
            .position(|&kind| kind == "PROPOSAL_FACT")
            .unwrap();
        assert_eq!(
            names[selection + 1..selection + 3],
            ["CALLBACK_CLAIM", "OPERATION_RECORD"]
        );
        let bundle = load_pending_verify_fragment(&opened, receipt, [38; 32], &config, 1).unwrap();
        let clock: crate::clock::ClockCheckpoint =
            serde_json::from_slice(&opened.records().last().unwrap().data).unwrap();
        let rngs: Vec<(Id, crate::RngCheckpoint)> = serde_json::from_slice(
            &opened
                .records()
                .iter()
                .find(|r| r.header.kind == "ROLE_RNG_CHECKPOINTS")
                .unwrap()
                .data,
        )
        .unwrap();
        drop(opened);
        assert_eq!(bundle.proposals.len(), 1);
        let fact = &bundle.proposals[0];
        assert_eq!(fact.selected_input_ref, input_ref);
        assert_eq!(fact.first_receive_input_ref, input_ref);
        assert_eq!(fact.original_receive_tick_us, voted_at);
        assert_eq!(fact.original_receive_local_us, Some(voted_at as i64 - 1000));
        assert_eq!(fact.signature_state, r::SignatureState::Pending);
        assert_eq!(bundle.operations.len(), 1);
        let op = &bundle.operations[0];
        assert_eq!(op.kind, r::OperationKind::Verify);
        assert_eq!(op.helper_phase, r::HelperPhase::WaitInput);
        assert_eq!(op.missing_dependencies, vec![proposal.payload.0.to_vec()]);
        let operation_id = op.operation_id;
        let binding = op.original_work_binding.clone();
        let cut = bundle.cut;
        let startup = Arc::new(StartupFence::new(cut));
        let copy = root.join("reopened-aux");
        fs::create_dir(&copy).unwrap();
        for name in ["native-aux", "native-aux.head"] {
            fs::copy(root.join(name), copy.join(name)).unwrap();
        }
        let resumed_writer =
            LedgerWriter::new(RecoveryLedger::open(&copy, identity, receipt, None).unwrap());
        let mut reopened = Replica::new(config.clone(), 1, receiver).unwrap();
        let saved: crate::native_application::WorkBinding =
            serde_json::from_slice(&binding).unwrap();
        reopened
            .archive
            .lock()
            .unwrap()
            .restore_work_frontier(saved.request_generation)
            .unwrap();
        let later = tick + 50_000;
        let mut profile = config.profile(1);
        profile.incarnation = 2;
        ControlledDriver::run_async(
            commonware_runtime::tokio::Config::default()
                .with_storage_directory(root.join("next-native")),
            profile.clone(),
            async |driver| {
                driver.configure_witness(config.witness(1)?, &root.join("next-witness"))?;
                driver.install_recovered_clock(&clock, later, config.base.stop)?;
                let first = driver
                    .grant_async(
                        crate::Grant {
                            id: 5,
                            tick_us: later,
                        },
                        crate::Input::RequestSample,
                        |_| Ok(()),
                    )
                    .await?;
                assert!(
                    first
                        .events
                        .iter()
                        .any(|event| matches!(event, Event::SampleRequested { request: 2 }))
                );
                driver
                    .grant_async(
                        crate::Grant {
                            id: 6,
                            tick_us: later,
                        },
                        crate::Input::DeliverSample(crate::ClockSample {
                            run_id: config.run_id,
                            node: 1,
                            incarnation: 2,
                            clock_domain: profile.clock_domain(),
                            rate_profile: crate::Profile::rate_profile(),
                            request_id: 2,
                            sequence: 2,
                            as_of_tick_us: later,
                            node_time_at_as_of_us: later as i64 - 1000,
                            uncertainty_us: 1000,
                            lease_end_tick_us: later + 100_000,
                        }),
                        |_| Ok(()),
                    )
                    .await?;
                let restored = Arc::new(ActorStore::new(
                    resumed_writer.clone(),
                    driver.recovery_authority(config.cid),
                    config.base.epoch,
                    bundle,
                    vec![],
                ));
                let mut recover = grant(&config, 1, 7, later, input::Kind::Start(config.start()));
                recover.binding.as_mut().unwrap().incarnation = 2;
                recover.grant_hash = wire::grant_hash(&recover).to_vec();
                let (reference, intent) = GrantIntent::from_grant(
                    &config,
                    1,
                    2,
                    3,
                    &recover,
                    r::RecoveryReceipt::default(),
                    [38; 32],
                    Some(later as i64 - 1000),
                )?;
                reopened.archive.lock().unwrap().tick = later;
                let recovered_grant = crate::Grant {
                    id: 7,
                    tick_us: later,
                };
                restored
                    .write_ahead(driver, recovered_grant, reference, intent)
                    .await?;
                let step = driver
                    .grant_async(recovered_grant, crate::Input::None, |driver| {
                        for (role, checkpoint) in rngs {
                            driver.restore_rng(config.cid, role, checkpoint)?;
                        }
                        reopened.start_engine(
                            driver,
                            Some(EngineRecovery {
                                store: restored.clone(),
                                startup: startup.clone(),
                                session_not_before: None,
                            }),
                        )
                    })
                    .await?;
                assert!(
                    !step
                        .events
                        .iter()
                        .any(|event| matches!(event, Event::TimerArmed { .. }))
                );
                assert!(reopened.sign.lock().unwrap().is_empty());
                {
                    let archive = reopened.archive.lock().unwrap();
                    assert!(archive.live_recovered_work(operation_id));
                    let bound = archive
                        .events
                        .iter()
                        .find_map(|event| match event {
                            AppAudit::WorkReconnected {
                                binding,
                                reconnect_generation: 1,
                                original_due_tick_us: None,
                            } => Some(binding),
                            _ => None,
                        })
                        .expect("restored real application/Archive producer");
                    assert_eq!(serde_json::to_vec(bound).unwrap(), binding);
                }
                assert!(startup.stage_release(&resumed_writer).is_err());
                startup
                    .ready(
                        r::ActorId::Application,
                        cut,
                        crate::hash_parts(b"I1b/recovery-state/v1", &[&binding]),
                    )
                    .unwrap();
                startup
                    .ready(
                        r::ActorId::Archive,
                        cut,
                        crate::hash_parts(
                            b"I1b/recovery-state/v1",
                            &[&operation_id, &config.genesis],
                        ),
                    )
                    .unwrap();
                assert!(startup.stage_release(&resumed_writer).is_err());
                assert!(driver.native_clock_ready());
                let report =
                    serde_json::to_vec(&driver.recovery_clock_checkpoint(config.cid)?).unwrap();
                startup
                    .ready(
                        r::ActorId::Driver,
                        cut,
                        crate::hash_parts(b"I1b/recovery-state/v1", &[&report]),
                    )
                    .unwrap();
                let token = startup.stage_release(&resumed_writer)?;
                let release_receipt = resumed_writer
                    .flush_root(driver, recovered_grant, config.cid, token)
                    .await?;
                let released = driver
                    .grant_async(
                        crate::Grant {
                            id: 8,
                            tick_us: later,
                        },
                        crate::Input::None,
                        |_| startup.release(&resumed_writer, release_receipt),
                    )
                    .await?;
                assert!(released.events.iter().any(|event| matches!(event,
                    Event::TimerArmed{timer} if timer.deadline_us==original_deadline)));
                assert!(!released.events.iter().any(|event| matches!(event,
                    Event::TimerArmed{timer} if timer.deadline_us==later as i64-1000+400_000)));
                assert!(reopened.sign.lock().unwrap().is_empty());
                Ok(())
            },
        )
        .unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    fn grant(
        config: &Resolved,
        node: u32,
        step: u64,
        tick: u64,
        kind: input::Kind,
    ) -> proto::Grant {
        let mut grant = proto::Grant {
            binding: Some(config.binding(node)),
            logical_step_id: step,
            event_id: step,
            tick_us: tick,
            causal_wave: 0,
            tie_rank: wire::tie_hash(config.base.seed, step).to_vec(),
            input: Some(proto::Input { kind: Some(kind) }),
            grant_hash: vec![],
        };
        grant.grant_hash = wire::grant_hash(&grant).to_vec();
        grant
    }

    fn produce_leader_vote(root: &Path, config: Arc<Resolved>, leader: NativeScheme) -> Envelope {
        let tick = START + 500;
        let mut voter = Replica::new(config.clone(), 0, leader).unwrap();
        ControlledDriver::run_async(
            commonware_runtime::tokio::Config::default()
                .with_storage_directory(root.join("leader-native")),
            config.profile(0),
            async |driver| {
                driver.configure_witness(config.witness(0)?, &root.join("leader-witness"))?;
                sample(&mut voter, driver, &config, 0, tick).await?;
                let ack = voter
                    .step(
                        driver,
                        &grant(&config, 0, 3, tick, input::Kind::Start(config.start())),
                    )
                    .await?;
                let work = ack
                    .effects
                    .into_iter()
                    .find_map(|effect| match effect.kind {
                        Some(effect::Kind::Work(work)) if work.kind == WorkKind::Propose as i32 => {
                            Some(work)
                        }
                        _ => None,
                    })
                    .ok_or(Error("fixture actual leader work missing"))?;
                let ack = voter
                    .step(
                        driver,
                        &grant(
                            &config,
                            0,
                            4,
                            tick + WORK_US,
                            input::Kind::WorkCompleted(work),
                        ),
                    )
                    .await?;
                ack.effects
                    .into_iter()
                    .find_map(|effect| match effect.kind {
                        Some(effect::Kind::Send(envelope))
                            if envelope.channel == Channel::Vote as i32
                                && envelope.dst_member_id == 1
                                && matches!(
                                    envelope.provenance,
                                    Some(Provenance::LocalWitness(_))
                                ) =>
                        {
                            Some(envelope)
                        }
                        _ => None,
                    })
                    .ok_or(Error("fixture actual witnessed leader vote missing"))
            },
        )
        .unwrap()
    }

    async fn sample(
        replica: &mut Replica,
        driver: &mut ControlledDriver,
        config: &Resolved,
        node: u32,
        tick: u64,
    ) -> Result<()> {
        replica
            .step(
                driver,
                &grant(
                    config,
                    node,
                    1,
                    tick - 500,
                    input::Kind::RequestSample(Empty {}),
                ),
            )
            .await?;
        replica
            .step(
                driver,
                &grant(
                    config,
                    node,
                    2,
                    tick,
                    input::Kind::Sample(ClockSample {
                        run_id: config.run_id.to_vec(),
                        worker_id: node,
                        incarnation: 1,
                        clock_domain: config.profile(node).clock_domain().to_vec(),
                        rate_profile: crate::Profile::rate_profile().to_vec(),
                        request_id: 1,
                        sequence: 1,
                        as_of_tick_us: tick - 250,
                        node_time_at_as_of_us: (tick - 250) as i64
                            + config.base.epsilon[node as usize],
                        uncertainty_us: 1000,
                        lease_end_tick_us: tick + 99_750,
                    }),
                ),
            )
            .await?;
        Ok(())
    }
}
