//! Explicit component scenario for the private 09 Anvil driver.

#[path = "support/process.rs"]
mod process;

use batch_services::{
    FaultPoint, ServiceConfig,
    accepted_view::{
        CurrentCursor, FinalizedBlock, InboxGateway, ReadError, materialize_accepted_view,
    },
    batcher::{BatchJob, Batcher, JobTerminal},
    certifier::{CertificationRequest, Certifier},
    crypto::{
        Address, BatchWire, InboxDomain, InboxSigners, SigningKey, decode_hex, hex,
        submit_batch_calldata,
    },
    da::DaStore,
    frame::{commitment, decode_closed},
    source::{FixtureMissing, FixtureSnapshot, FixtureSourceView},
    transaction::{LegacyTerms, PreparedTransaction, create_address, sign_legacy},
};
use l1_adapter::{
    Action, Command, Config, Outcome, Query, Reply,
    rpc::{AnvilRpc, Rpc, RpcError},
};
use poc_core::{
    execution::apply,
    fixture::{DepProvider, ValidatedDepFixture},
};
use poc_types::domain::{
    Account, Anchor, BlockRef, Credit, DepFixture, ExecutionState, Hash32, Metadata, PublicConfig,
    ReplayIndex, ReplayKind, ReplayPosition, Transfer, VirtualBlock,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

const CHAIN: u64 = 31337;
const START: u64 = 1_700_000_000;
const FIXTURE: &str = "poc11-real-inbox-fixture-v1";
const GPOS: &[u8] = b"poc11/fixtures/gpos/1";
const BATCHER_READER: &str = "batcher";
const CERTIFIER_READER: &str = "certifier";
const OBSERVER: &str = "observer";

fn sig_key(scalar: u8) -> SigningKey {
    let mut secret = [0; 32];
    secret[31] = scalar;
    SigningKey::new(secret).unwrap()
}

fn terms(nonce: u64) -> LegacyTerms {
    LegacyTerms {
        nonce,
        gas_price: 2_000_000_000,
        gas_limit: 18_000_000,
    }
}

fn save(path: &Path, value: &impl Serialize) {
    let mut body = serde_json::to_vec_pretty(value).unwrap();
    body.push(b'\n');
    fs::write(path, body).unwrap();
}

fn setup() -> (FixtureSnapshot, ServiceConfig, Config) {
    let genesis = ExecutionState {
        metadata: Metadata {
            height: 0,
            origin: 0,
            timestamp_us: 0,
        },
        accounts: BTreeMap::from([(
            0,
            Account {
                nonce: 0,
                balance: 10,
            },
        )]),
    };
    let native = Hash32::new([44; 32]);
    let dep = DepFixture {
        first_height: 1,
        rows: vec![vec![Credit { to: 0, amount: 5 }], vec![], vec![], vec![]],
    };
    let config = PublicConfig {
        chain_id: CHAIN,
        rule_version: 1,
        verifier_version: 1,
        registry_hash: Hash32::new([13; 32]),
        dep_fixture_hash: dep.digest().unwrap(),
        l1_fixture_hash: Hash32::new([14; 32]),
        genesis: Anchor {
            block: BlockRef {
                height: 0,
                origin: 0,
                block_id: Hash32::new([45; 32]),
            },
            timestamp_us: 0,
        },
        genesis_state_root: genesis.root().unwrap(),
        genesis_native_hash: native,
        l2_slot_us: 1_000_000,
        l1_genesis_timestamp_us: 0,
        l1_block_us: 2_000_000,
        t_dep_us: 5_000_000,
        t_drift_us: 15_000_000,
        metadata_window_us: 200_000,
        byte_limit: 65_536,
    };
    let provide = ValidatedDepFixture::load(&config, dep.clone()).unwrap();
    let mut blocks = Vec::new();
    let mut state = genesis.clone();
    let tx = |from, to, nonce, amount| {
        Transfer {
            from,
            to,
            nonce,
            amount,
        }
        .encode_raw()
    };
    for (index, transactions) in [
        vec![
            tx(0, 1, 0, 7),
            tx(0, 1, 0, 1),
            tx(0, 1, 1, 9),
            tx(0, 1, 1, 8),
        ],
        vec![],
        vec![tx(1, 0, 0, 4)],
        vec![tx(1, 0, 1, 2)],
    ]
    .into_iter()
    .enumerate()
    {
        let h = (index + 1) as u64;
        let metadata = Metadata {
            height: h,
            origin: 0,
            timestamp_us: h * 1_000_000,
        };
        let poc_types::domain::InputStatus::Ready(dep) = provide.at(&metadata) else {
            panic!("fixture DEP absent")
        };
        let block = VirtualBlock {
            metadata,
            dep,
            pre_state_root: state.root().unwrap(),
            transactions,
        };
        state = apply(&state, &block.metadata, &block.dep, &block.transactions)
            .unwrap()
            .state;
        blocks.push(block);
    }
    let submitter = sig_key(1).address();
    let authority = create_address(submitter, 0);
    let domain = InboxDomain {
        chain_id: CHAIN,
        inbox: create_address(authority, 1),
        config_ref: *config.reference().unwrap().as_hash(),
    };
    let signers = InboxSigners {
        da: sig_key(2).address(),
        certifier: sig_key(3).address(),
    };
    let service = ServiceConfig {
        inbox: domain,
        signers,
        submitter,
        source_fixture: FIXTURE.into(),
    };
    let driver = Config {
        chain_id: CHAIN,
        config_ref: hex(domain.config_ref.as_bytes()),
        authority: hex(&authority),
        inbox: hex(&domain.inbox),
        submitter: hex(&submitter),
        genesis_height: 0,
        genesis_origin: 0,
        genesis_timestamp_seconds: START,
        slot_seconds: 2,
        announcement_seconds: 6,
        finality_delay_us: 2_000_000,
        readers: vec![
            BATCHER_READER.into(),
            CERTIFIER_READER.into(),
            OBSERVER.into(),
        ],
    };
    let source = FixtureSnapshot {
        source_fixture: FIXTURE.into(),
        gpos: GPOS.into(),
        position: ReplayPosition {
            key_us: 5_000_000,
            kind: ReplayKind::Espresso,
            index: ReplayIndex::Espresso(2),
        },
        config,
        dep,
        parent: genesis,
        native_parent: native,
        blocks,
        control_coverage: poc_types::domain::L1Position {
            block: 1,
            tx: 0,
            log: 0,
        },
        processed_resumes: vec![],
    };
    (source, service, driver)
}

fn word(value: u64) -> [u8; 32] {
    let mut word = [0; 32];
    word[24..].copy_from_slice(&value.to_be_bytes());
    word
}
fn address_word(address: Address) -> [u8; 32] {
    let mut word = [0; 32];
    word[12..].copy_from_slice(&address);
    word
}
fn hash(bytes: &str) -> [u8; 32] {
    commitment(bytes.as_bytes()).into_bytes()
}
fn words_hash(words: &[[u8; 32]]) -> [u8; 32] {
    commitment(words.as_flattened()).into_bytes()
}

fn deployment(artifact: &Path, config: &ServiceConfig) -> PreparedTransaction {
    let artifact: Value = serde_json::from_slice(&fs::read(artifact).unwrap()).unwrap();
    let mut data = decode_hex(
        artifact["bytecode"]["object"].as_str().unwrap(),
        1024 * 1024,
        "authority bytecode",
    )
    .unwrap();
    data.extend(config.inbox.config_ref.as_bytes());
    for address in [
        sig_key(4).address(),
        config.submitter,
        config.signers.certifier,
        config.signers.da,
    ] {
        data.extend(address_word(address));
    }
    for value in [6, 0, 0] {
        data.extend(word(value));
    }
    sign_legacy(&sig_key(1), CHAIN, None, terms(0), &data).unwrap()
}

fn resume(config: &ServiceConfig, nonce: u64) -> PreparedTransaction {
    let authority = create_address(config.submitter, 0);
    let policy = [word(7), word(0), word(1), word(2), word(0)];
    let mut policy_hash = vec![hash(
        "Policy(uint8 enabledMask,uint8 order0,uint8 order1,uint8 order2,uint64 l1Floor)",
    )];
    policy_hash.extend(policy);
    let request = [83; 32];
    let h = 8;
    let body = words_hash(&[
        hash(
            "Resume(address caller,bytes32 requestId,uint64 controlSeq,bytes32 policyHash,uint64 targetHeight,uint64 origin)",
        ),
        address_word(config.submitter),
        request,
        word(0),
        words_hash(&policy_hash),
        word(h),
        word(0),
    ]);
    let domain = words_hash(&[
        hash(
            "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract,bytes32 salt)",
        ),
        hash("SequencerRotationControlAuthority"),
        hash("1"),
        word(CHAIN),
        address_word(authority),
        config.inbox.config_ref.into_bytes(),
    ]);
    let mut preimage = vec![0x19, 0x01];
    preimage.extend(domain);
    preimage.extend(body);
    let signature = sig_key(4).sign_digest(commitment(&preimage)).unwrap();
    let mut data =
        hash("submitResume(bytes32,uint64,(uint8,uint8,uint8,uint8,uint64),uint64,uint64,bytes)")
            [..4]
            .to_vec();
    data.extend(request);
    data.extend(word(0));
    data.extend(policy.as_flattened());
    data.extend(word(h));
    data.extend(word(0));
    data.extend(word(10 * 32));
    data.extend(word(65));
    data.extend(signature);
    data.extend([0; 31]);
    sign_legacy(&sig_key(1), CHAIN, Some(authority), terms(nonce), &data).unwrap()
}

struct Host {
    gateway: InboxGateway,
    next: u64,
    trace: Vec<Value>,
    protocol: Vec<Value>,
}
impl Host {
    fn call(&mut self, action: Action) -> Reply {
        let command = Command {
            id: self.next,
            action,
        };
        self.next += 1;
        let reply = self.gateway.apply(command.clone());
        self.trace.push(json!({"command": command, "reply": reply}));
        reply
    }
    fn ready(&mut self, action: Action) -> Value {
        let reply = self.call(action);
        match reply.outcome {
            Outcome::Ready { result } => result,
            other => panic!("driver command failed: {other:?}"),
        }
    }
    fn submit(&mut self, tx: &PreparedTransaction) {
        let result = self.ready(Action::Submit { raw: hex(tx.raw()) });
        assert_eq!(result["submitted_tx_hash"], hex(tx.hash().as_bytes()));
    }
    fn mine(&mut self, slot: u64) {
        self.ready(Action::Tick {
            reference_us: slot * 2_000_000,
        });
        self.ready(Action::Mine { slot });
    }
    fn finalize(&mut self, slot: u64) {
        self.ready(Action::Tick {
            reference_us: (slot + 1) * 2_000_000,
        });
        self.ready(Action::Finalize {
            through_height: slot,
        });
    }
    fn deliver(&mut self, slot: u64, reader: &str) {
        self.ready(Action::Deliver {
            reader: reader.into(),
            through_height: slot,
            from_height: None,
        });
    }
    fn deliver_all(&mut self, slot: u64) {
        for reader in [BATCHER_READER, CERTIFIER_READER, OBSERVER] {
            self.deliver(slot, reader);
        }
    }
    fn block(&mut self, slot: u64, reader: &str) -> Result<FinalizedBlock, ReadError> {
        let id = self.next;
        self.next += 1;
        let (reply, result) = self.gateway.read_finalized_block(id, reader, slot);
        let command = Command {
            id,
            action: Action::Read {
                reader: reader.into(),
                query: Query::Block { height: slot },
            },
        };
        self.trace.push(json!({"command": command, "reply": reply}));
        result
    }
    fn prefix(&mut self, slot: u64, reader: &str) -> Vec<FinalizedBlock> {
        (0..=slot).map(|h| self.block(h, reader).unwrap()).collect()
    }
    fn cursor(&mut self, slot: u64, reader: &str) -> CurrentCursor {
        let (trace, cursor) = self
            .gateway
            .read_current_cursor(&mut self.next, reader, slot);
        self.trace.extend(
            trace
                .into_iter()
                .map(|(command, reply)| json!({"command": command, "reply": reply})),
        );
        cursor.unwrap()
    }
}

struct Actors {
    batcher: Option<Batcher>,
    certifier: Option<Certifier>,
    da: Option<DaStore>,
    config: ServiceConfig,
    out: PathBuf,
    crashes: Vec<Value>,
    view_batcher: FixtureSourceView,
    view_certifier: FixtureSourceView,
    b: u64,
    c: u64,
    d: u64,
}
fn next(id: &mut u64) -> u64 {
    let output = *id;
    *id += 1;
    output
}

impl Actors {
    fn crash_batcher(
        &mut self,
        point: FaultPoint,
        run: impl FnOnce(&mut Batcher, &FixtureSourceView),
    ) {
        drop(self.batcher.take());
        let root = self.out.join("batcher");
        let mut evidence = process::crash_at(&root, "batcher", point, || {
            let mut actor = Batcher::open_with_hook(
                &root,
                self.config.clone(),
                BATCHER_READER,
                sig_key(1),
                process::inject,
            )
            .unwrap();
            run(&mut actor, &self.view_batcher);
        });
        self.batcher =
            Some(Batcher::open(&root, self.config.clone(), BATCHER_READER, sig_key(1)).unwrap());
        process::recovered(&mut evidence, &root);
        self.crashes.push(evidence);
    }

    fn crash_certifier(
        &mut self,
        point: FaultPoint,
        run: impl FnOnce(&mut Certifier, &FixtureSourceView),
    ) {
        drop(self.certifier.take());
        let root = self.out.join("certifier");
        let mut evidence = process::crash_at(&root, "certifier", point, || {
            let mut actor = Certifier::open_with_hook(
                &root,
                self.config.clone(),
                CERTIFIER_READER,
                sig_key(3),
                process::inject,
            )
            .unwrap();
            run(&mut actor, &self.view_certifier);
        });
        self.certifier = Some(
            Certifier::open(&root, self.config.clone(), CERTIFIER_READER, sig_key(3)).unwrap(),
        );
        process::recovered(&mut evidence, &root);
        self.crashes.push(evidence);
    }

    fn crash_da(&mut self, point: FaultPoint, command: u64, payload: &[u8]) {
        drop(self.da.take());
        let root = self.out.join("da");
        let mut evidence = process::crash_at(&root, "da", point, || {
            DaStore::open_with_hook(&root, self.config.clone(), sig_key(2), process::inject)
                .unwrap()
                .disperse(command, payload)
                .unwrap();
        });
        let d = hex(commitment(payload).as_bytes());
        let bytes = root.join(format!("{d}.bin"));
        let receipt = root.join(format!("{d}.receipt"));
        assert_eq!(bytes.exists(), point != FaultPoint::DaBeforeBytes);
        assert_eq!(receipt.exists(), point == FaultPoint::DaReceiptDurable);
        if bytes.exists() {
            assert_eq!(fs::read(bytes).unwrap(), payload);
        }
        self.da = Some(DaStore::open(&root, self.config.clone(), sig_key(2)).unwrap());
        process::recovered(&mut evidence, &root);
        self.crashes.push(evidence);
    }

    fn lost_submission_response(
        &mut self,
        host: &mut Host,
        job: &BatchJob,
        originally_prepared: &PreparedTransaction,
    ) {
        drop(self.batcher.take());
        let root = self.out.join("batcher");
        let mut evidence = process::crash_after_send(
            &root,
            || {
                let actor =
                    Batcher::open(&root, self.config.clone(), BATCHER_READER, sig_key(1)).unwrap();
                let tx = actor.pending_transaction(job.id()).unwrap().clone();
                (actor, tx)
            },
            |tx| {
                assert_eq!(tx, originally_prepared);
                host.submit(tx);
            },
        );
        let recovered =
            Batcher::open(&root, self.config.clone(), BATCHER_READER, sig_key(1)).unwrap();
        assert_eq!(
            recovered.pending_transaction(job.id()).unwrap(),
            originally_prepared
        );
        assert_eq!(
            recovered.job(job.id()).unwrap().phase(),
            batch_services::batcher::JobPhase::SubmittedUnknown
        );
        self.batcher = Some(recovered);
        process::recovered(&mut evidence, &root);
        self.crashes.push(evidence);
    }

    fn stage(
        &mut self,
        host: &mut Host,
        slot: u64,
        first: u64,
        last: u64,
        nonce: u64,
    ) -> (
        BatchJob,
        CertificationRequest,
        BatchWire,
        PreparedTransaction,
    ) {
        let batcher_cursor = host.cursor(slot, BATCHER_READER);
        let certifier_cursor = host.cursor(slot, CERTIFIER_READER);
        let b_id = next(&mut self.b);
        if first == 1 {
            self.crash_batcher(FaultPoint::BatcherBuiltDurable, |actor, source| {
                actor
                    .build(
                        b_id,
                        &batcher_cursor,
                        batcher_cursor.cursor(),
                        GPOS,
                        first,
                        last,
                        source,
                    )
                    .unwrap();
            });
        }
        let job = self
            .batcher
            .as_mut()
            .unwrap()
            .build(
                b_id,
                &batcher_cursor,
                batcher_cursor.cursor(),
                GPOS,
                first,
                last,
                &self.view_batcher,
            )
            .unwrap();
        assert!(matches!(
            self.batcher.as_mut().unwrap().build(
                b_id,
                &batcher_cursor,
                batcher_cursor.cursor(),
                b"different",
                first,
                last,
                &self.view_batcher
            ),
            Err(ReadError::Reject(_))
        ));
        let d_id = next(&mut self.d);
        if first == 1 {
            for point in [
                FaultPoint::DaBeforeBytes,
                FaultPoint::DaBytesDurable,
                FaultPoint::DaReceiptDurable,
            ] {
                self.crash_da(point, d_id, job.payload());
            }
        }
        let (d, da_receipt) = self
            .da
            .as_mut()
            .unwrap()
            .disperse(d_id, job.payload())
            .unwrap();
        let b_id = next(&mut self.b);
        if first == 1 {
            let receipt_path = self
                .out
                .join("da")
                .join(format!("{}.receipt", hex(d.as_bytes())));
            assert_eq!(fs::read(receipt_path).unwrap(), da_receipt);
            self.crash_batcher(FaultPoint::BatcherDaDurable, |actor, _| {
                actor.record_da(b_id, job.id(), d, &da_receipt).unwrap();
            });
        }
        let job = self
            .batcher
            .as_mut()
            .unwrap()
            .record_da(b_id, job.id(), d, &da_receipt)
            .unwrap();
        let fetched = self
            .da
            .as_mut()
            .unwrap()
            .retrieve(next(&mut self.d), d)
            .unwrap();
        let request = job.certification_request().unwrap();
        let c_id = next(&mut self.c);
        assert!(matches!(
            self.certifier.as_mut().unwrap().certify(
                c_id,
                &request,
                &certifier_cursor,
                Some(&fetched),
                &FixtureSourceView::default()
            ),
            Err(ReadError::Wait(_))
        ));
        assert!(matches!(
            self.certifier.as_mut().unwrap().certify(
                c_id,
                &request,
                &certifier_cursor,
                None,
                &self.view_certifier
            ),
            Err(ReadError::Wait(_))
        ));
        if first == 1 {
            for material in [
                FixtureMissing::Anchor,
                FixtureMissing::State,
                FixtureMissing::Metadata,
                FixtureMissing::Deposits,
                FixtureMissing::Height(1),
            ] {
                let mut waiting = FixtureSourceView::default();
                waiting.defer(GPOS.to_vec(), material).unwrap();
                assert!(
                    matches!(self.certifier.as_mut().unwrap().certify(c_id, &request, &certifier_cursor, Some(&fetched), &waiting),
                    Err(ReadError::Wait(reason)) if reason.contains("material pending"))
                );
            }
            let (mut conflicting, _, _) = setup();
            conflicting.blocks[0].pre_state_root = Hash32::new([99; 32]);
            let mut wrong = FixtureSourceView::default();
            wrong.deliver(conflicting).unwrap();
            assert!(
                matches!(self.certifier.as_mut().unwrap().certify(c_id, &request, &certifier_cursor, Some(&fetched), &wrong),
                Err(ReadError::Reject(reason)) if reason.contains("pre-state"))
            );
            assert!(
                matches!(self.certifier.as_mut().unwrap().certify(c_id, &request, &batcher_cursor, Some(&fetched), &self.view_certifier),
                Err(ReadError::Reject(reason)) if reason.contains("reader"))
            );
            self.certifier.as_mut().unwrap().set_available(false);
            assert!(
                matches!(self.certifier.as_mut().unwrap().certify(c_id, &request, &certifier_cursor, Some(&fetched), &self.view_certifier),
                Err(ReadError::Wait(reason)) if reason.contains("unavailable"))
            );
            self.certifier.as_mut().unwrap().set_available(true);
            self.batcher.as_mut().unwrap().set_available(false);
            assert!(
                matches!(self.batcher.as_mut().unwrap().pending_transaction(job.id()), Err(ReadError::Wait(reason)) if reason.contains("unavailable"))
            );
            self.batcher.as_mut().unwrap().set_available(true);
        }
        if first == 1 {
            for point in [
                FaultPoint::CertifierBeforeSignaturePersist,
                FaultPoint::CertifierSignatureDurable,
            ] {
                self.crash_certifier(point, |actor, source| {
                    actor
                        .certify(c_id, &request, &certifier_cursor, Some(&fetched), source)
                        .unwrap();
                });
            }
        }
        let signed = self
            .certifier
            .as_mut()
            .unwrap()
            .certify(
                c_id,
                &request,
                &certifier_cursor,
                Some(&fetched),
                &self.view_certifier,
            )
            .unwrap();
        assert_eq!(
            signed,
            self.certifier
                .as_mut()
                .unwrap()
                .certify(
                    c_id,
                    &request,
                    &certifier_cursor,
                    Some(&fetched),
                    &self.view_certifier
                )
                .unwrap()
        );
        let b_id = next(&mut self.b);
        if first == 1 {
            self.crash_batcher(FaultPoint::BatcherCertifiedDurable, |actor, _| {
                actor.record_certified(b_id, job.id(), &signed).unwrap();
            });
        }
        let job = self
            .batcher
            .as_mut()
            .unwrap()
            .record_certified(b_id, job.id(), &signed)
            .unwrap();
        let b_id = next(&mut self.b);
        if first == 1 {
            self.crash_batcher(FaultPoint::BatcherRawDurable, |actor, _| {
                actor
                    .prepare_transaction(b_id, job.id(), terms(nonce))
                    .unwrap();
            });
        }
        let tx = self
            .batcher
            .as_mut()
            .unwrap()
            .prepare_transaction(b_id, job.id(), terms(nonce))
            .unwrap();
        assert_eq!(
            self.batcher
                .as_mut()
                .unwrap()
                .pending_transaction(job.id())
                .unwrap(),
            &tx
        );
        host.protocol.push(json!({"kind":"raw_durable", "job_id":job.id(), "prev":request.prev,
            "range":[first,last], "commitment":d, "nonce":nonce, "tx_hash":tx.hash(), "batch":signed}));
        (job, request, signed, tx)
    }
}

fn account_rows(state: &ExecutionState) -> Vec<(u8, u64, u64)> {
    state
        .accounts
        .iter()
        .map(|(&id, value)| (id, value.nonce, value.balance))
        .collect()
}
fn source_artifact(
    source: &FixtureSnapshot,
    second: &BatchJob,
    stale: &BatchJob,
    domain: InboxDomain,
) -> Value {
    let mut blocks = Vec::new();
    let mut observed = BTreeMap::new();
    for report in second
        .execution_audit()
        .iter()
        .chain(stale.execution_audit())
    {
        observed.insert(report.height, report);
    }
    for block in &source.blocks {
        let actual = observed[&block.metadata.height];
        blocks.push(json!({"metadata":block.metadata, "dep":block.dep.iter().map(|data| hex(data)).collect::<Vec<_>>(),
            "raw_transactions":block.transactions.iter().map(|data| hex(data)).collect::<Vec<_>>(),
            "pre_state_root":hex(block.pre_state_root.as_bytes()),
            "observed":{"kept":actual.kept.iter().map(|data| hex(data)).collect::<Vec<_>>(), "post_accounts":account_rows(&actual.post_state),
                "post_state_root":hex(actual.post_state_root.as_bytes()),"native_hash":hex(actual.native_hash.as_bytes())}}));
    }
    json!({"schema":"BS-EXECUTION-EVIDENCE-v1", "source_mode":"fixture", "fixture_id":FIXTURE,
        "config_ref":hex(domain.config_ref.as_bytes()), "byte_limit":source.config.byte_limit,
        "snapshots":[{"gpos":hex(&source.gpos),"initial":{"metadata":source.parent.metadata,"accounts":account_rows(&source.parent),
            "native_parent":hex(source.native_parent.as_bytes())},"blocks":blocks}]})
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    assert_eq!(
        args.len(),
        5,
        "run_component LOOPBACK AUTHORITY_ARTIFACT NEW_OUTPUT WALL_PAUSE_MS"
    );
    let (source, config, driver) = setup();
    let out = PathBuf::from(&args[3]);
    fs::create_dir(&out).unwrap();
    let pause = Duration::from_millis(args[4].parse().unwrap());
    save(&out.join("driver-config.json"), &driver);
    save(&out.join("fixture-snapshot.json"), &source);
    let mut host = Host {
        gateway: InboxGateway::connect(args[1].parse().unwrap(), driver.clone(), config.signers)
            .unwrap(),
        next: 1,
        trace: vec![],
        protocol: vec![],
    };
    let mut actors = Actors {
        batcher: Some(
            Batcher::open(
                &out.join("batcher"),
                config.clone(),
                BATCHER_READER,
                sig_key(1),
            )
            .unwrap(),
        ),
        certifier: Some(
            Certifier::open(
                &out.join("certifier"),
                config.clone(),
                CERTIFIER_READER,
                sig_key(3),
            )
            .unwrap(),
        ),
        da: Some(DaStore::open(&out.join("da"), config.clone(), sig_key(2)).unwrap()),
        config: config.clone(),
        out: out.clone(),
        crashes: vec![],
        view_batcher: FixtureSourceView::default(),
        view_certifier: FixtureSourceView::default(),
        b: 1,
        c: 1,
        d: 1,
    };
    actors.view_batcher.deliver(source.clone()).unwrap();
    actors.view_certifier.deliver(source.clone()).unwrap();
    host.submit(&deployment(Path::new(&args[2]), &config));
    host.mine(1);
    host.finalize(1);
    host.deliver_all(1);

    let (first, first_request, first_signed, first_tx) = actors.stage(&mut host, 1, 1, 2, 1);
    assert_eq!(
        decode_closed(first.payload())
            .unwrap()
            .records
            .iter()
            .map(|record| record.transactions.len())
            .collect::<Vec<_>>(),
        [2, 0]
    );
    actors.lost_submission_response(&mut host, &first, &first_tx);
    let pending = host.prefix(1, BATCHER_READER);
    assert!(
        matches!(actors.batcher.as_mut().unwrap().reconcile(next(&mut actors.b), first.id(), &pending), Err(ReadError::Wait(reason)) if reason.contains("SubmittedUnknown"))
    );
    assert!(matches!(host.block(2, OBSERVER), Err(ReadError::Wait(_))));
    host.mine(2);
    assert!(matches!(host.block(2, OBSERVER), Err(ReadError::Wait(_))));
    host.finalize(2);
    assert!(matches!(host.block(2, OBSERVER), Err(ReadError::Wait(_))));
    std::thread::sleep(pause);
    host.deliver_all(2);
    let accepted = host.block(2, OBSERVER).unwrap();
    let first_record = &accepted.accepted()[0];
    assert_eq!(first_record.identity().transaction_hash, first_tx.hash());
    actors.da.as_mut().unwrap().set_available(false);
    assert!(matches!(
        actors
            .da
            .as_mut()
            .unwrap()
            .retrieve(next(&mut actors.d), first.commitment()),
        Err(ReadError::Wait(_))
    ));
    assert!(matches!(
        first_record.load_payload(None),
        Err(ReadError::Wait(_))
    ));
    assert!(matches!(
        first_record.load_payload(Some(b"corrupt")),
        Err(ReadError::Wait(_))
    ));
    actors.da.as_mut().unwrap().set_available(true);
    let object = out
        .join("da")
        .join(format!("{}.bin", hex(first.commitment().as_bytes())));
    let held = out.join("held-confirmed-da.bin");
    fs::rename(&object, &held).unwrap();
    assert!(matches!(
        actors
            .da
            .as_mut()
            .unwrap()
            .retrieve(next(&mut actors.d), first.commitment()),
        Err(ReadError::Wait(_))
    ));
    fs::write(&object, b"corrupted by scenario").unwrap();
    assert!(
        matches!(actors.da.as_mut().unwrap().retrieve(next(&mut actors.d), first.commitment()), Err(ReadError::Wait(reason)) if reason.contains("integrity"))
    );
    assert!(matches!(
        actors
            .da
            .as_mut()
            .unwrap()
            .disperse(next(&mut actors.d), first.payload()),
        Err(ReadError::Reject(_))
    ));
    assert_eq!(fs::read(&object).unwrap(), b"corrupted by scenario");
    fs::rename(&held, &object).unwrap();
    let loaded = actors
        .da
        .as_mut()
        .unwrap()
        .retrieve(next(&mut actors.d), first.commitment())
        .unwrap();
    assert_eq!(
        first_record
            .load_payload(Some(loaded.payload()))
            .unwrap()
            .frame()
            .records
            .len(),
        2
    );
    let prefix = host.prefix(2, BATCHER_READER);
    let b_id = next(&mut actors.b);
    actors.crash_batcher(FaultPoint::BatcherAcceptedDurable, |actor, _| {
        actor.reconcile(b_id, first.id(), &prefix).unwrap();
    });
    let recovered = actors.batcher.as_mut().unwrap();
    assert!(matches!(
        recovered.job(first.id()).unwrap().terminal(),
        Some(JobTerminal::Accepted { .. })
    ));
    assert!(matches!(
        recovered.reconcile(b_id, first.id(), &prefix).unwrap(),
        JobTerminal::Accepted { .. }
    ));
    let cursor = host.cursor(2, CERTIFIER_READER);
    assert!(
        matches!(actors.certifier.as_mut().unwrap().certify(next(&mut actors.c), &first_request, &cursor, Some(&loaded), &actors.view_certifier), Err(ReadError::Reject(reason)) if reason == "StalePrev")
    );

    let (second, _, _, second_tx) = actors.stage(&mut host, 2, 3, 3, 2);
    assert_eq!(
        decode_closed(second.payload()).unwrap().records[0]
            .transactions
            .len(),
        1
    );
    host.submit(&second_tx);
    host.mine(3);
    host.finalize(3);
    host.deliver_all(3);
    let prefix = host.prefix(3, BATCHER_READER);
    assert!(matches!(
        actors
            .batcher
            .as_mut()
            .unwrap()
            .reconcile(next(&mut actors.b), second.id(), &prefix)
            .unwrap(),
        JobTerminal::Accepted { .. }
    ));
    let cursor = host.cursor(3, OBSERVER);
    let observer_prefix = host.prefix(3, OBSERVER);
    let g = materialize_accepted_view(
        cursor.clone(),
        &observer_prefix,
        &actors.view_batcher,
        &config,
    )
    .unwrap();
    assert_eq!(g.virtual_prefix.len(), 3);
    let missing = materialize_accepted_view(
        cursor,
        &observer_prefix,
        &FixtureSourceView::default(),
        &config,
    )
    .unwrap();
    assert!(missing.virtual_prefix.is_empty() && missing.missing_sources.len() == 1);
    host.protocol.push(json!({"kind":"materialization", "ready_heights":g.virtual_prefix.len(), "missing":missing.missing_sources}));

    // Same known request at a different L1 tx nonce. Inbox emits no new log.
    let retry = sign_legacy(
        &sig_key(1),
        CHAIN,
        Some(config.inbox.inbox),
        terms(3),
        &submit_batch_calldata(&first_signed).unwrap(),
    )
    .unwrap();
    host.submit(&retry);
    host.mine(4);
    host.finalize(4);
    host.deliver_all(4);
    assert!(host.block(4, OBSERVER).unwrap().accepted().is_empty());

    let (stale, stale_request, stale_signed, stale_tx) = actors.stage(&mut host, 4, 4, 4, 5);
    host.submit(&resume(&config, 4));
    host.mine(5);
    host.finalize(5);
    host.deliver_all(5);
    let cursor = host.cursor(5, CERTIFIER_READER);
    let control = cursor.latest_resume().unwrap();
    assert_eq!(control.set, cursor.written_at());
    assert!(control.resume > control.set);
    let recovery_payload =
        batch_services::frame::encode_closed(&[batch_services::frame::BlockRecord {
            block_number: 9,
            epoch_num: 0,
            transactions: vec![],
        }])
        .unwrap();
    let (d, signature) = actors
        .da
        .as_mut()
        .unwrap()
        .disperse(next(&mut actors.d), &recovery_payload)
        .unwrap();
    let no_replay = CertificationRequest {
        request_id: Hash32::new([44; 32]),
        prev: cursor.cursor().clone(),
        payload: recovery_payload,
        commitment: d,
        da_certificate: signature.to_vec(),
        gpos: GPOS.to_vec(),
    };
    assert!(
        matches!(actors.certifier.as_mut().unwrap().certify(next(&mut actors.c), &no_replay, &cursor, None, &actors.view_certifier), Err(ReadError::Wait(reason)) if reason.contains("RESUME"))
    );
    assert!(
        matches!(actors.certifier.as_mut().unwrap().certify(next(&mut actors.c), &stale_request, &cursor, None, &actors.view_certifier), Err(ReadError::Reject(reason)) if reason == "StalePrev")
    );
    let mut wrong_signer = stale_signed.clone();
    wrong_signer.certifier_signature = hex(&sig_key(4)
        .sign_digest(config.inbox.batch_digest(&wrong_signer).unwrap())
        .unwrap());
    let mut wrong_commitment = stale_signed.clone();
    wrong_commitment.commitment = hex(&[55; 32]);
    wrong_commitment.certifier_signature = hex(&sig_key(3)
        .sign_digest(config.inbox.batch_digest(&wrong_commitment).unwrap())
        .unwrap());
    let mut rpc = AnvilRpc::new(args[1].parse().unwrap()).unwrap();
    for (batch, error) in [
        (&stale_signed, "StalePrev()"),
        (&wrong_signer, "InvalidSignature()"),
        (&wrong_commitment, "InvalidDA()"),
    ] {
        let calldata = hex(&submit_batch_calldata(batch).unwrap());
        let res = rpc.call(
            "eth_call",
            json!([{"to":driver.inbox,"from":driver.submitter,"data":calldata},"0x5"]),
        );
        let Err(RpcError::Remote(error_data)) = res else {
            panic!("expected {error} diagnostic revert: {res:?}")
        };
        assert_eq!(error_data["data"], hex(&hash(error)[..4]));
    }
    save(&out.join("revert-diagnostics.json"), &rpc.evidence);
    host.submit(&stale_tx);
    for (batch, nonce) in [(&wrong_signer, 6), (&wrong_commitment, 7)] {
        host.submit(
            &sign_legacy(
                &sig_key(1),
                CHAIN,
                Some(config.inbox.inbox),
                terms(nonce),
                &submit_batch_calldata(batch).unwrap(),
            )
            .unwrap(),
        );
    }
    host.mine(6);
    host.finalize(6);
    host.deliver_all(6);
    let prefix = host.prefix(6, BATCHER_READER);
    assert!(matches!(
        actors
            .batcher
            .as_mut()
            .unwrap()
            .reconcile(next(&mut actors.b), stale.id(), &prefix)
            .unwrap(),
        JobTerminal::Reverted { .. }
    ));
    let cursor = host.cursor(6, OBSERVER);
    let observer_prefix = host.prefix(6, OBSERVER);
    let view =
        materialize_accepted_view(cursor, &observer_prefix, &actors.view_batcher, &config).unwrap();
    assert!(
        view.virtual_prefix.is_empty()
            && view.historical_prefix.len() == 3
            && view.missing_sources.len() == 1
    );
    save(&out.join("accepted-view-after-override.json"), &view);

    let receipts: Vec<_> = observer_prefix
        .iter()
        .flat_map(FinalizedBlock::raw_receipts)
        .collect();
    let blocks: Vec<_> = observer_prefix
        .iter()
        .map(FinalizedBlock::raw_block)
        .collect();
    let records: Vec<_> = observer_prefix
        .iter()
        .flat_map(FinalizedBlock::accepted)
        .collect();
    assert_eq!(records.len(), 2);
    assert_eq!(receipts.iter().filter(|r| r["status"] == "0x0").count(), 3);
    let bundle_config = json!({"chainId": CHAIN,"configRef":driver.config_ref,"authority":driver.authority,
        "inbox":driver.inbox,"submitter":driver.submitter,"genesisHeight":0,"genesisOrigin":0,
        "genesisTimestampSeconds":START,"slotSeconds":2,"announcementSeconds":6,
        "signers":{"governance":hex(&sig_key(4).address()),"certifier":hex(&config.signers.certifier),"da":hex(&config.signers.da)}});
    save(
        &out.join("raw-rpc-bundle.json"),
        &json!({"config":bundle_config,"blocks":blocks,"receipts":receipts,"historicalCursorReads":[]}),
    );
    save(&out.join("accepted-records.json"), &records);
    let mut payload_index = BTreeMap::new();
    for job in [&first, &second] {
        let d = hex(job.commitment().as_bytes());
        let file = format!("da/{d}.bin");
        assert_eq!(fs::read(out.join(&file)).unwrap(), job.payload());
        payload_index.insert(d, file);
    }
    save(
        &out.join("da-index.json"),
        &json!({"payloads":payload_index}),
    );
    save(
        &out.join("execution-source.json"),
        &source_artifact(&source, &second, &stale, config.inbox),
    );
    save(&out.join("driver-trace.json"), &host.trace);
    save(
        &out.join("rpc-transcript.json"),
        &host.gateway.drain_rpc_evidence(),
    );
    save(&out.join("protocol-trace.json"), &host.protocol);
    save(&out.join("crash-trace.json"), &actors.crashes);
    drop(actors.batcher.take());
    drop(actors.certifier.take());
    drop(actors.da.take());
    let mut other = config.clone();
    other.source_fixture.push_str("/wrong");
    assert!(matches!(
        Batcher::open(
            &out.join("batcher"),
            other.clone(),
            BATCHER_READER,
            sig_key(1)
        ),
        Err(ReadError::Reject(_))
    ));
    assert!(matches!(
        Certifier::open(
            &out.join("certifier"),
            other.clone(),
            CERTIFIER_READER,
            sig_key(3)
        ),
        Err(ReadError::Reject(_))
    ));
    assert!(matches!(
        DaStore::open(&out.join("da"), other, sig_key(2)),
        Err(ReadError::Reject(_))
    ));

    save(
        &out.join("result.json"),
        &json!({"status":"PASS", "seed":"none (explicit script)", "source_mode":"fixture",
        "real_inbox_accepted":records.len(),"stale_prev_status":0,"crashes":actors.crashes.len(),"wall_pause_ms":pause.as_millis()}),
    );
}
