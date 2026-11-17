//! M4's sole L1 driver and independent Inbox/DA derivation actors.
use super::{M1Result, check, checked, problem, worker::durable_file};
use batch_services::{
    ServiceConfig,
    accepted_view::{AcceptedRecord, FinalizedBlock, InboxGateway, ReadError},
    batcher::Batcher,
    certifier::Certifier,
    crypto::{InboxSigners, SigningKey, decode_fixed, hex},
    da::DaStore,
    source::{FixtureSnapshot, FixtureSourceView},
    transaction::LegacyTerms,
};
use l1_adapter::{Action, Command, Config, Outcome as DriverOutcome, Query};
use poc_core::{fixture::ValidatedDepFixture, merge::MergedBlock};
use poc_derivation::{Derivation, DerivationConfig, GatewaySource, Outcome, ResumeCut};
use poc_types::domain::*;
use poc_viewer_gfr::ResumeBasis;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};

const READER: &str = "A";
pub(super) const SOURCE_FIXTURE: &str = "m4-committed-g/v1";

pub(super) fn project_cut(bytes: &[u8]) -> M1Result<ResumeBasis> {
    let cut: ResumeCut = serde_json::from_slice(bytes)?;
    check(
        serde_json::to_vec(&cut)? == bytes,
        "noncanonical RESUME cut",
    )?;
    let mut blocks = Vec::new();
    for item in &cut.derived_prefix {
        if item.state.metadata.height > cut.target_height {
            break;
        }
        check(
            item.state.metadata == item.virtual_block.metadata
                && item.native_block.digest()? == item.native_hash
                && item.native_block.post_state_root == item.state.root()?,
            "malformed RESUME item",
        )?;
        blocks.push(MergedBlock {
            virtual_block: item.virtual_block.clone(),
            physical_digest: None,
            state: item.state.clone(),
            native_block: item.native_block.clone(),
            native_hash: item.native_hash,
        });
    }
    check(
        cut.resume_position.block == cut.cursor_set_position.block
            && cut.resume_position.tx == cut.cursor_set_position.tx
            && cut.resume_position.log > cut.cursor_set_position.log
            && cut.target_height == cut.target_anchor.anchor.block.height,
        "malformed RESUME cut binding",
    )?;
    Ok(ResumeBasis {
        position: cut.resume_position,
        set_position: cut.cursor_set_position,
        config_ref: cut.evidence.derivation.config_ref,
        including_block: cut.evidence.including_block,
        transaction_hash: cut.evidence.transaction_hash,
        digest: Hash32::try_from(
            poc_types::hash_parts("PoC/Derivation/ResumeCutCheckpoint/v1", &[bytes]).as_slice(),
        )?,
        genesis: cut.genesis,
        target: cut.target_anchor,
        blocks,
    })
}

struct Feed {
    source: GatewaySource,
    d: Option<Derivation>,
    pending: BTreeMap<L1Position, AcceptedRecord>,
    received: Option<u64>,
    inbox_height: Option<u64>,
}

struct Port {
    gateway: InboxGateway,
    driver: Config,
    config: DerivationConfig,
    genesis: ExecutionAnchor,
    dep: ValidatedDepFixture,
    feeds: BTreeMap<String, Feed>,
    l1_genesis: Option<Hash32>,
    l1_a: Vec<FinalizedBlock>,
    deliveries: BTreeMap<(String, u64), Value>,
    jobs: Vec<Hash32>,
    batcher: Batcher,
    certifier: Certifier,
    da: DaStore,
    source: FixtureSourceView,
    root: PathBuf,
    auxiliary: File,
    derivation: File,
    actor_id: u64,
    observe_resume: bool,
}

fn key(role: &str) -> M1Result<SigningKey> {
    let digest = Sha256::digest(format!("ANVIL-L1-COMPONENT-v1/{role}"));
    checked(SigningKey::new(digest.into()))
}

impl Port {
    fn new(endpoint: std::net::SocketAddr, config_path: &Path) -> M1Result<Self> {
        let driver: Config = serde_json::from_reader(File::open(config_path)?)?;
        let run = config_path
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| problem("M4 run directory"))?;
        let root = run.join("derivation");
        fs::create_dir(&root)?;
        let public = PublicConfig::decode(&fs::read(run.join("config.bin"))?)?;
        let genesis = ExecutionAnchor::decode(&fs::read(run.join("genesis.bin"))?)?;
        let dep = DepFixture::decode(&fs::read(run.join("dep.bin"))?)?;
        let setup = checked(poc_core::fixture::ValidatedDepFixture::load(&public, dep))?;
        let config = checked(DerivationConfig::new(public, 4))?;
        let da = key("da")?;
        let certifier = key("certifier")?;
        let submitter = key("caller")?;
        let signers = InboxSigners {
            da: da.address(),
            certifier: certifier.address(),
        };
        let gateway = checked(InboxGateway::connect(endpoint, driver.clone(), signers))?;
        let service = ServiceConfig {
            inbox: gateway.domain(),
            signers,
            submitter: submitter.address(),
            source_fixture: SOURCE_FIXTURE.into(),
        };
        let batcher = checked(Batcher::open(
            &root.join("batcher"),
            service.clone(),
            READER,
            submitter,
        ))?;
        let certifier = checked(Certifier::open(
            &root.join("certifier"),
            service.clone(),
            READER,
            certifier,
        ))?;
        let da = checked(DaStore::open(&root.join("da"), service, da))?;
        let create = |name| {
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(root.join(name))
        };
        let auxiliary = create("cursor-reads.jsonl")?;
        let derivation = create("trace.jsonl")?;
        Ok(Self {
            gateway,
            driver,
            config,
            genesis,
            dep: setup,
            feeds: BTreeMap::new(),
            l1_genesis: None,
            l1_a: Vec::new(),
            deliveries: BTreeMap::new(),
            jobs: Vec::new(),
            batcher,
            certifier,
            da,
            source: FixtureSourceView::default(),
            auxiliary,
            derivation,
            root,
            actor_id: 0,
            observe_resume: std::env::var_os("M5_SCENARIO").is_some(),
        })
    }

    fn bump(counter: &mut u64) -> u64 {
        *counter += 1;
        *counter
    }

    fn prepare(&mut self, value: &Value) -> M1Result<Value> {
        let snapshot: FixtureSnapshot = serde_json::from_value(value["snapshot"].clone())?;
        let nonce = value["nonce"]
            .as_u64()
            .ok_or_else(|| problem("M4 submitter nonce"))?;
        let height = snapshot
            .blocks
            .first()
            .ok_or_else(|| problem("M4 empty source"))?
            .metadata
            .height;
        check(
            snapshot.blocks.len() == 1,
            "M4 expects one captured height per frame",
        )?;
        let gpos = snapshot.gpos.clone();
        let cursor_height = self
            .l1_a
            .last()
            .ok_or_else(|| problem("M4 missing finalized cursor"))?
            .height();
        let mut query_id = value["id"]
            .as_u64()
            .ok_or_else(|| problem("M4 batch command id"))?;
        let (reads, cursor) =
            self.gateway
                .read_current_cursor(&mut query_id, READER, cursor_height);
        for (command, reply) in reads {
            write_row(
                &mut self.auxiliary,
                &json!({"command":command,"reply":reply}),
            )?;
        }
        let cursor = checked(cursor)?;
        checked(self.source.deliver(snapshot.clone()))?;
        let job = checked(self.batcher.build(
            Self::bump(&mut self.actor_id),
            &cursor,
            cursor.cursor(),
            &gpos,
            height,
            height,
            &self.source,
        ))?;
        let (commitment, receipt) = checked(
            self.da
                .disperse(Self::bump(&mut self.actor_id), job.payload()),
        )?;
        let job = checked(self.batcher.record_da(
            Self::bump(&mut self.actor_id),
            job.id(),
            commitment,
            &receipt,
        ))?;
        let data = checked(self.da.retrieve(Self::bump(&mut self.actor_id), commitment))?;
        let request = checked(job.certification_request())?;
        let batch = checked(self.certifier.certify(
            Self::bump(&mut self.actor_id),
            &request,
            &cursor,
            Some(&data),
            &self.source,
        ))?;
        checked(
            self.batcher
                .record_certified(Self::bump(&mut self.actor_id), job.id(), &batch),
        )?;
        let prepared = checked(self.batcher.prepare_transaction(
            Self::bump(&mut self.actor_id),
            job.id(),
            LegacyTerms {
                nonce,
                gas_price: 1_000_000_000,
                gas_limit: 1_000_000,
            },
        ))?;
        self.jobs.push(job.id());
        let evidence = json!({"height":height,"job":job.id(),"snapshot":snapshot,"certified":batch,
            "execution":job.execution_audit(),"raw":hex(prepared.raw()),"hash":hex(prepared.hash().as_bytes()),
            "nonce":prepared.nonce(),"commitment":commitment});
        checked(durable_file(
            &self.root.join(format!("batch-h{height}.json")),
            &serde_json::to_vec(&evidence)?,
        ))?;
        Ok(evidence)
    }

    fn receive(&mut self, block: FinalizedBlock, tick: u64) -> M1Result<Value> {
        let key = (block.reader().to_owned(), block.height());
        let view = if let Some(old) = self.deliveries.get(&key) {
            let feed = &self.feeds[&key.0];
            if feed.received != Some(key.1) || feed.pending.is_empty() {
                return Ok(old.clone());
            }
            self.advance(&key.0, key.1, tick)?
        } else {
            self.deliver(block, tick)?
        };
        self.deliveries.insert(key, view.clone());
        Ok(view)
    }

    fn deliver(&mut self, block: FinalizedBlock, tick: u64) -> M1Result<Value> {
        let reader = block.reader().to_owned();
        let height = block.height();
        if height == 0 {
            let hash = Hash32::new(checked(decode_fixed(
                block.raw_block()["hash"]
                    .as_str()
                    .ok_or_else(|| problem("M4 L1 genesis hash missing"))?,
                "M4 genesis hash",
            ))?);
            if let Some(old) = self.l1_genesis {
                check(old == hash, "divergent L1 genesis")?;
            }
            self.l1_genesis = Some(hash);
            self.feeds.insert(
                reader.clone(),
                Feed {
                    source: checked(GatewaySource::new(
                        &self.config,
                        &self.driver,
                        reader.clone(),
                        hash,
                    ))?,
                    d: None,
                    pending: BTreeMap::new(),
                    received: None,
                    inbox_height: None,
                },
            );
        }
        let feed = self
            .feeds
            .get_mut(&reader)
            .ok_or_else(|| problem("M4 L1 genesis missing"))?;
        checked(feed.source.receive_block(&block))?;
        for record in block.accepted() {
            let pos = record.position();
            checked(durable_file(
                &self.root.join(format!(
                    "accepted-{reader}-{}-{}-{}.json",
                    pos.block, pos.tx, pos.log
                )),
                &serde_json::to_vec(record)?,
            ))?;
            feed.pending.insert(pos, record.clone());
        }
        feed.received = Some(height);
        feed.inbox_height = block.verified_block()["cursor_at_end"]["cursor"]["height"].as_u64();
        if reader == READER {
            self.l1_a.push(block.clone());
            for job in &self.jobs {
                match self
                    .batcher
                    .reconcile(Self::bump(&mut self.actor_id), *job, &self.l1_a)
                {
                    Ok(_) | Err(ReadError::Wait(_)) => {}
                    Err(error) => return Err(problem(format!("M4 reconcile {error:?}"))),
                }
            }
        }
        self.advance(&reader, height, tick)
    }

    fn advance(&mut self, reader: &str, height: u64, tick: u64) -> M1Result<Value> {
        let feed = self.feeds.get_mut(reader).expect("registered reader");
        let mut delivered = Vec::new();
        for (pos, record) in &feed.pending {
            let result = self
                .da
                .retrieve(Self::bump(&mut self.actor_id), record.commitment())
                .and_then(|object| record.load_payload(Some(object.payload())));
            match result {
                Ok(payload) => {
                    checked(feed.source.receive_payload(payload))?;
                    delivered.push(*pos);
                }
                Err(ReadError::Wait(reason)) => {
                    write_row(
                        &mut self.derivation,
                        &json!({"reader":reader,"tick_us":tick,
                        "l1_height":height,"da_wait":{"position":pos,"commitment":record.commitment(),"reason":reason}}),
                    )?;
                }
                Err(error) => return Err(problem(format!("M4 accepted DA rejected: {error:?}"))),
            }
        }
        for pos in delivered {
            feed.pending.remove(&pos);
        }
        if feed.d.is_none() {
            match Derivation::init(
                self.config.clone(),
                self.genesis.clone(),
                self.dep.clone(),
                &feed.source.inputs(),
            ) {
                Outcome::Ready(state) => feed.d = Some(state),
                Outcome::Wait(wait) => {
                    write_row(
                        &mut self.derivation,
                        &json!({"reader":reader,"tick_us":tick,"l1_height":height,"wait":format!("{wait:?}")}),
                    )?;
                    return Ok(json!({"head":null,"cuts":[]}));
                }
                Outcome::Reject(error) => return Err(problem(format!("M4 D init {error:?}"))),
            }
        }
        let state = feed.d.as_mut().expect("D init checked");
        let mut cuts = Vec::new();
        for _ in 0..1024 {
            let prior = state.derived_prefix().len();
            match state.step(&feed.source.inputs()) {
                Outcome::Ready(step) => {
                    write_row(
                        &mut self.derivation,
                        &json!({"reader":reader,"tick_us":tick,"l1_height":height,
                        "step":step,"new_items":&state.derived_prefix()[prior..]}),
                    )?;
                    if let poc_derivation::Step::CursorSet {
                        resume_position: pos,
                        ..
                    } = step
                    {
                        let Outcome::Ready(cut) = state.derive_at_resume(pos) else {
                            return Err(problem("D lost completed cut"));
                        };
                        let bytes = serde_json::to_vec(&cut)?;
                        let basis = project_cut(&bytes)?;
                        let name =
                            format!("cut-{reader}-{}-{}-{}.json", pos.block, pos.tx, pos.log);
                        checked(durable_file(&self.root.join(&name), &bytes))?;
                        cuts.push(
                            json!({"position":pos,"digest":basis.digest,"file":name,"bytes":bytes}),
                        );
                    }
                }
                Outcome::Wait(wait) => {
                    write_row(
                        &mut self.derivation,
                        &json!({"reader":reader,"tick_us":tick,"l1_height":height,"wait":format!("{wait:?}")}),
                    )?;
                    let mut view = json!({"head":state.head(),"h1":state.native_hash_at(1),
                        "h2":state.native_hash_at(2),"cuts":cuts,"wait":format!("{wait:?}")});
                    if self.observe_resume && let Some(inbox_height) = feed.inbox_height {
                        let d = state.head();
                        let drained = feed.pending.is_empty()
                            && state.pending_batches_empty()
                            && state.complete_frontier().is_some_and(|tip| tip.height == height)
                            && match wait.reason {
                                poc_derivation::WaitReason::CompleteWindow { through } => through > height,
                                poc_derivation::WaitReason::CompleteBlock(block) => block > height,
                                _ => false,
                            };
                        // Recipients independently authenticate the ordinary h1 payload; no additional
                        // batch is queued in the bounded M5 policy. An unprocessed accepted payload
                        // invalidates this readiness/cursor envelope.
                        let h1_commitment=state.derived_prefix().first().and_then(|row|
                            match row.cause {
                                poc_derivation::DerivationCause::AcceptedBatch{commitment,..} => Some(commitment),
                                _=>None,
                            });
                        let da_probe=h1_commitment.is_some_and(|commitment|
                            self.da.retrieve(Self::bump(&mut self.actor_id),commitment).is_ok());
                        let batches_reconciled=self.jobs.iter().all(|job|self.batcher.job(*job)
                            .is_some_and(|entry|entry.terminal().is_some()));
                        let services_ready=feed.pending.is_empty() && da_probe
                            && batches_reconciled && self.batcher.is_available()
                            && self.certifier.is_available();
                        write_row(&mut self.derivation,&json!({"reader":reader,"tick_us":tick,
                            "l1_height":height,"service_probe":{"commitment":h1_commitment,
                            "da_verified":da_probe,"batches_reconciled":batches_reconciled,
                            "batcher_available":self.batcher.is_available(),
                            "certifier_available":self.certifier.is_available()}}))?;
                        view["observation"] = json!(super::m5::DObservation {
                            l1_height: height,
                            inbox_height,
                            d_height: d.state.metadata.height,
                            d_origin: d.state.metadata.origin,
                            hashes: std::iter::once(state.genesis().native_hash)
                                .chain(state.derived_prefix().iter().map(|item|item.native_hash))
                                .collect(),
                            drained,
                            services_ready,
                            pending_batch_through: (drained && batches_reconciled).then_some(inbox_height),
                        });
                    }
                    return Ok(view);
                }
                Outcome::Reject(reason) => return Err(problem(format!("M4 D step {reason:?}"))),
            }
        }
        Err(problem("M4 D positive work limit"))
    }
}

fn write_row(file: &mut File, row: &Value) -> M1Result<()> {
    serde_json::to_writer(&mut *file, row)?;
    file.write_all(b"\n")?;
    file.sync_data()?;
    Ok(())
}

pub fn run_driver(args: &[String]) -> M1Result<()> {
    let [endpoint, config] = args else {
        return Err(problem("usage: m4-driver ENDPOINT CONFIG"));
    };
    let mut port = Port::new(endpoint.parse()?, Path::new(config))?;
    let mut output = io::stdout().lock();
    let respond = |out: &mut io::StdoutLock<'_>, row: Value| -> M1Result<()> {
        serde_json::to_writer(&mut *out, &row)?;
        out.write_all(b"\n")?;
        out.flush()?;
        Ok(())
    };
    respond(
        &mut output,
        json!({"startup_rpc":port.gateway.drain_rpc_evidence()}),
    )?;
    for line in io::stdin().lock().lines() {
        let value: Value = serde_json::from_str(&line?)?;
        let reply = if value["op"] == "m4_prepare_batch" {
            json!({"status":"ready","result":port.prepare(&value)?})
        } else {
            let command: Command = serde_json::from_value(value)?;
            if let Action::Read {
                reader,
                query: Query::Block { height },
            } = &command.action
            {
                let (reply, block) = port
                    .gateway
                    .read_finalized_block(command.id, reader, *height);
                let tick = reply.reference_us;
                let mut value = serde_json::to_value(&reply)?;
                if matches!(reply.outcome, DriverOutcome::Ready { .. }) {
                    value["result"]["block"]["derivation"] = port.receive(checked(block)?, tick)?;
                }
                value
            } else {
                serde_json::to_value(port.gateway.apply(command))?
            }
        };
        respond(
            &mut output,
            json!({"reply":reply,"rpc":port.gateway.drain_rpc_evidence()}),
        )?;
    }
    Ok(())
}
