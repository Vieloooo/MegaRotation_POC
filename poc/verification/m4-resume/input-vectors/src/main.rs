//! Input vectors against the real Anvil 09 reader and closed 11/12 capabilities.
use batch_services::{
    accepted_view::{AcceptedRecord, InboxGateway, ReadError},
    crypto::InboxSigners,
};
use l1_adapter::{Action, Command, Config, Query};
use poc_core::fixture::ValidatedDepFixture;
use poc_derivation::{
    Checkpoint, Derivation, DerivationCause, DerivationConfig, GatewaySource, InputPoint, Outcome,
    Reject, ResumeCut, WaitReason,
};
use poc_types::domain::{
    Account, Anchor, BlockRef, Credit, DepFixture, ExecutionAnchor, ExecutionState, Hash32,
    L1Position, Metadata, PublicConfig,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};

const GENESIS_US: u64 = 1_000_000_000;
const WINDOW: u64 = 4;
type TestResult<T> = Result<T, Box<dyn std::error::Error>>;

fn profile() -> (DerivationConfig, ExecutionAnchor, DepFixture) {
    let genesis = ExecutionAnchor {
        anchor: Anchor {
            block: BlockRef {
                height: 0,
                origin: 0,
                block_id: Hash32::new([0x55; 32]),
            },
            timestamp_us: GENESIS_US,
        },
        state: ExecutionState {
            metadata: Metadata {
                height: 0,
                timestamp_us: GENESIS_US,
                origin: 0,
            },
            accounts: [(
                1,
                Account {
                    nonce: 0,
                    balance: 10,
                },
            )]
            .into(),
        },
        native_hash: Hash32::new([0x11; 32]),
    };
    let mut deposits = DepFixture {
        first_height: 1,
        rows: vec![vec![]; 64],
    };
    deposits.rows[0] = vec![Credit { to: 2, amount: 2 }];
    deposits.rows[1] = vec![Credit { to: 1, amount: 3 }];
    let public = PublicConfig {
        chain_id: 31337,
        rule_version: 1,
        verifier_version: 1,
        registry_hash: Hash32::new([0x33; 32]),
        dep_fixture_hash: deposits.digest().unwrap(),
        l1_fixture_hash: poc_core::fixture::reg_a_l1_fixture().digest().unwrap(),
        genesis: genesis.anchor.clone(),
        genesis_state_root: genesis.state.root().unwrap(),
        genesis_native_hash: genesis.native_hash,
        l2_slot_us: 1_000_000,
        l1_genesis_timestamp_us: GENESIS_US,
        l1_block_us: 2_000_000,
        t_dep_us: 0,
        t_drift_us: 15_000_000,
        metadata_window_us: 30_000_000,
        byte_limit: 250,
    };
    (
        DerivationConfig::new(public, WINDOW).unwrap(),
        genesis,
        deposits,
    )
}

fn hex(bytes: &[u8]) -> String {
    format!("0x{}", poc_types::hex(bytes))
}

fn decode<const N: usize>(text: &str) -> TestResult<[u8; N]> {
    let text = text
        .strip_prefix("0x")
        .ok_or("expected hexadecimal prefix")?;
    if text.len() != N * 2 || !text.is_ascii() {
        return Err("incorrect hexadecimal width".into());
    }
    let mut bytes = [0; N];
    for (index, output) in bytes.iter_mut().enumerate() {
        *output = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16)?;
    }
    Ok(bytes)
}

fn read_json(path: &Path) -> TestResult<Value> {
    Ok(serde_json::from_reader(fs::File::open(path)?)?)
}

fn ready<T>(result: Outcome<T>) -> TestResult<T> {
    match result {
        Outcome::Ready(value) => Ok(value),
        Outcome::Wait(value) => Err(format!("unexpected WAIT: {value:?}").into()),
        Outcome::Reject(value) => Err(format!("unexpected REJECT: {value:?}").into()),
    }
}

fn p(block: u64, tx: u64, log: u64) -> L1Position {
    L1Position { block, tx, log }
}

fn wait_payload(
    result: Outcome<()>,
    expected: L1Position,
    commitment: Hash32,
) -> TestResult<Value> {
    match result {
        Outcome::Wait(wait)
            if wait.position == InputPoint::Log(expected)
                && wait.reason == (WaitReason::Feature11AcceptedPayloadRequired { commitment }) =>
        {
            Ok(json!({"kind": "wait_accepted_da", "position": expected, "commitment": commitment}))
        }
        Outcome::Wait(other) => Err(format!("different D WAIT: {other:?}").into()),
        Outcome::Ready(()) => Err("D advanced past unresolved DA".into()),
        Outcome::Reject(other) => Err(format!("unexpected D REJECT: {other:?}").into()),
    }
}

fn tail_wait(result: Outcome<()>) -> TestResult<()> {
    match result {
        Outcome::Wait(wait)
            if matches!(
                wait.reason,
                WaitReason::CompleteBlock(_) | WaitReason::CompleteWindow { .. }
            ) =>
        {
            Ok(())
        }
        Outcome::Wait(other) => Err(format!("different finite-prefix WAIT: {other:?}").into()),
        Outcome::Ready(()) => Err("finite input unexpectedly complete".into()),
        Outcome::Reject(other) => Err(format!("unexpected D REJECT: {other:?}").into()),
    }
}

fn archive_identity(root: &Path) -> TestResult<()> {
    let fixture = read_json(&root.join("fixture-snapshot.json"))?;
    let config = DerivationConfig::new(serde_json::from_value(fixture["config"].clone())?, WINDOW)
        .map_err(|err| format!("{err:?}"))?;
    let driver: Config = serde_json::from_value(read_json(&root.join("driver-config.json"))?)?;
    let bundle = read_json(&root.join("raw-rpc-bundle.json"))?;
    let genesis_hash = bundle["blocks"][0]["hash"]
        .as_str()
        .ok_or("missing archive genesis hash")?;
    assert_eq!(
        hex(config.config_ref().unwrap().as_bytes()),
        driver.config_ref
    );
    let candidate = GatewaySource::new(
        &config,
        &driver,
        "observer".into(),
        Hash32::new(decode(genesis_hash)?),
    );
    assert!(matches!(
        candidate,
        Err(Reject::Configuration(
            "L1 identity and public configuration disagree"
        ))
    ));
    println!(
        "{}",
        json!({
            "status": "PASS",
            "classification": "STRICT_D_SOURCE_IDENTITY_REJECT",
            "signed_config_ref": driver.config_ref,
            "fixture_l1_genesis_timestamp_us": config.public().l1_genesis_timestamp_us,
            "driver_l1_genesis_timestamp_s": driver.genesis_timestamp_seconds
        })
    );
    Ok(())
}

struct Session {
    gateway: InboxGateway,
    sources: BTreeMap<String, GatewaySource>,
    accepted: BTreeMap<(String, L1Position), AcceptedRecord>,
    artifact: PathBuf,
}

impl Session {
    fn command(&mut self, command: Command) -> TestResult<Value> {
        let reply = match &command.action {
            Action::Read {
                reader,
                query: Query::Block { height },
            } => {
                let (reply, block) = self
                    .gateway
                    .read_finalized_block(command.id, reader, *height);
                match block {
                    Ok(block) => {
                        if let Some(source) = self.sources.get_mut(reader) {
                            source
                                .receive_block(&block)
                                .map_err(|err| format!("{err:?}"))?;
                        }
                        for record in block.accepted() {
                            self.accepted
                                .insert((reader.clone(), record.position()), record.clone());
                        }
                    }
                    Err(ReadError::Wait(_)) => (),
                    Err(other) => return Err(other.into()),
                }
                reply
            }
            _ => self.gateway.apply(command),
        };
        Ok(json!({"reply": reply, "rpc": self.gateway.drain_rpc_evidence()}))
    }

    fn get(&self, reader: &str, position: L1Position) -> TestResult<AcceptedRecord> {
        self.accepted
            .get(&(reader.to_owned(), position))
            .cloned()
            .ok_or_else(|| format!("missing {reader} capability at {position:?}").into())
    }

    fn payload(&self, record: &AcceptedRecord) -> TestResult<Vec<u8>> {
        Ok(fs::read(self.artifact.join("da").join(format!(
            "{}.bin",
            hex(record.commitment().as_bytes())
        )))?)
    }

    fn attach(&mut self, reader: &str, position: L1Position) -> TestResult<()> {
        let accepted = self.get(reader, position)?;
        let bytes = self.payload(&accepted)?;
        self.sources
            .get_mut(reader)
            .ok_or("unknown reader")?
            .receive_payload(accepted.load_payload(Some(&bytes))?)
            .map_err(|err| format!("{err:?}").into())
    }

    fn init(&self, reader: &str) -> TestResult<Derivation> {
        let (config, genesis, dep) = profile();
        let dep =
            ValidatedDepFixture::load(config.public(), dep).map_err(|err| format!("{err:?}"))?;
        ready(Derivation::init(
            config,
            genesis,
            dep,
            &self.sources[reader].inputs(),
        ))
    }

    fn restore(&self, reader: &str, checkpoint: &Checkpoint) -> TestResult<Derivation> {
        let (config, genesis, dep) = profile();
        let dep =
            ValidatedDepFixture::load(config.public(), dep).map_err(|err| format!("{err:?}"))?;
        ready(Derivation::restore(
            checkpoint,
            config,
            genesis,
            dep,
            &self.sources[reader].inputs(),
        ))
    }

    fn paired_block(&mut self) -> TestResult<(Value, Derivation, Derivation)> {
        let first = p(3, 0, 1);
        let accepted = p(3, 1, 0);
        let second = p(3, 2, 1);
        let a_record = self.get("A", accepted)?;
        assert!(matches!(
            a_record.load_payload(None),
            Err(ReadError::Wait(_))
        ));
        let mut a = self.init("A")?;
        let blocked = wait_payload(
            a.advance(&self.sources["A"].inputs()).status,
            accepted,
            a_record.commitment(),
        )?;
        let initial_pass = ready(a.derive_at_resume(first))?;
        let unchanged = a.checkpoint();
        assert_eq!(initial_pass.target_height, 1);
        assert_eq!(initial_pass.derived_prefix.len(), 1);
        assert_eq!(a.head().anchor.block.height, 1);
        assert!(matches!(a.derive_at_resume(second), Outcome::Wait(wait)
            if wait.position == InputPoint::Log(accepted)));
        let exact = self.payload(&a_record)?;
        let mut wrong = exact.clone();
        wrong[0] ^= 1;
        assert!(matches!(
            a_record.load_payload(Some(&wrong)),
            Err(ReadError::Wait(_))
        ));
        let after_bad = wait_payload(
            a.advance(&self.sources["A"].inputs()).status,
            accepted,
            a_record.commitment(),
        )?;
        assert_eq!(blocked, after_bad);
        assert_eq!(unchanged, a.checkpoint());

        let exact_a = a_record.load_payload(Some(&exact))?;
        assert!(
            self.sources
                .get_mut("B")
                .unwrap()
                .receive_payload(exact_a.clone())
                .is_err(),
            "A capability cannot impersonate reader B"
        );
        self.sources
            .get_mut("A")
            .unwrap()
            .receive_payload(exact_a)
            .map_err(|err| format!("{err:?}"))?;
        self.attach("B", accepted)?;
        tail_wait(a.advance(&self.sources["A"].inputs()).status)?;
        let second_pass = ready(a.derive_at_resume(second))?;
        assert_eq!(ready(a.derive_at_resume(first))?, initial_pass);
        check_pair(&initial_pass, &second_pass)?;

        let mut b = self.init("B")?;
        tail_wait(b.advance(&self.sources["B"].inputs()).status)?;
        assert_eq!(a.checkpoint(), b.checkpoint());
        assert_eq!(initial_pass, ready(b.derive_at_resume(first))?);
        assert_eq!(second_pass, ready(b.derive_at_resume(second))?);
        Ok((
            json!({
                "status": "PASS", "case": "same_block", "missing": blocked, "after_corrupt": after_bad,
                "cuts": [initial_pass, second_pass], "head": a.head(),
                "reader_convergence": true, "cross_reader_payload_rejected": true,
            }),
            a,
            b,
        ))
    }

    fn verify_tail(
        &self,
        original: &Value,
        mut a: Derivation,
        mut b: Derivation,
    ) -> TestResult<Value> {
        let first: ResumeCut = serde_json::from_value(original["cuts"][0].clone())?;
        let second: ResumeCut = serde_json::from_value(original["cuts"][1].clone())?;
        for (reader, derive) in [("A", &mut a), ("B", &mut b)] {
            tail_wait(derive.advance(&self.sources[reader].inputs()).status)?;
            assert!(derive.head().anchor.block.height > second.target_height);
            assert_eq!(
                first,
                ready(derive.derive_at_resume(first.resume_position))?
            );
            assert_eq!(
                second,
                ready(derive.derive_at_resume(second.resume_position))?
            );
            let restored = self.restore(reader, &derive.checkpoint())?;
            assert_eq!(derive.checkpoint(), restored.checkpoint());
            assert_eq!(
                first,
                ready(restored.derive_at_resume(first.resume_position))?
            );
            assert_eq!(
                second,
                ready(restored.derive_at_resume(second.resume_position))?
            );
        }
        assert_eq!(a.checkpoint(), b.checkpoint());
        Ok(json!({
            "status": "PASS", "derived_head": a.head(), "cut_immutability": true, "checkpoint_restore": true,
        }))
    }

    fn native_case(&mut self, mode: &str) -> TestResult<Value> {
        let position = match mode {
            "before" | "substitute" | "malformed" => p(3, 0, 0),
            "boundary" => p(4, 0, 0),
            "after" | "future" => p(5, 0, 0),
            _ => return Err("unknown focused input case".into()),
        };
        let record_b = self.get("B", position)?;
        if mode == "malformed" {
            let malformed = self.payload(&record_b)?;
            assert!(matches!(
                record_b.load_payload(Some(&malformed)),
                Err(ReadError::Reject(_))
            ));
            let mut state = self.init("B")?;
            let stalled = wait_payload(
                state.advance(&self.sources["B"].inputs()).status,
                position,
                record_b.commitment(),
            )?;
            assert_eq!(state.head().anchor.block.height, 0);
            assert!(state.derived_prefix().is_empty());
            return Ok(json!({
                "status":"PASS", "case": mode, "accepted": position,
                "payload":"REJECT_FRAME", "derivation": stalled, "head": state.head(),
            }));
        }
        for reader in ["A", "B"] {
            self.attach(reader, position)?;
        }
        let mut a = self.init("A")?;
        let mut b = self.init("B")?;
        tail_wait(a.advance(&self.sources["A"].inputs()).status)?;
        tail_wait(b.advance(&self.sources["B"].inputs()).status)?;
        assert_eq!(a.checkpoint(), b.checkpoint());
        let prefix = a.derived_prefix();
        match mode {
            "before" | "boundary" => {
                assert!(matches!(
                    prefix[0].cause,
                    DerivationCause::AcceptedBatch {
                        substituted: false,
                        ..
                    }
                ));
                assert_eq!(prefix[0].native_block.kept, vec![transfer(1, 2, 0, 3)]);
                assert_eq!(prefix[0].virtual_block.metadata.origin, 0);
                assert_eq!(
                    prefix[0].state.accounts[&1],
                    Account {
                        nonce: 1,
                        balance: 7
                    }
                );
            }
            "after" => {
                assert!(matches!(
                    prefix[0].cause,
                    DerivationCause::Fallback {
                        window_end: WINDOW,
                        ..
                    }
                ));
                assert!(prefix.iter().all(|row| row.native_block.kept.is_empty()));
                assert_eq!(
                    prefix[0].state.accounts[&1],
                    Account {
                        nonce: 0,
                        balance: 10
                    }
                );
            }
            "substitute" => {
                assert!(matches!(
                    prefix[0].cause,
                    DerivationCause::AcceptedBatch {
                        substituted: true,
                        record_index: 0,
                        ..
                    }
                ));
                assert!(prefix[0].native_block.kept.is_empty());
                assert_eq!(prefix[0].virtual_block.metadata.origin, 0);
                assert!(matches!(
                    prefix[1].cause,
                    DerivationCause::AcceptedBatch {
                        substituted: false,
                        record_index: 1,
                        ..
                    }
                ));
                assert_eq!(prefix[1].virtual_block.metadata.origin, 1);
                assert_eq!(prefix[1].native_block.kept, vec![transfer(1, 2, 0, 3)]);
                assert_eq!(
                    prefix[1].state.accounts[&1],
                    Account {
                        nonce: 1,
                        balance: 10
                    }
                );
            }
            "future" => {
                assert!(matches!(
                    prefix[2].cause,
                    DerivationCause::Fallback { window_end: 5, .. }
                ));
                assert!(prefix[2].native_block.kept.is_empty());
                assert_eq!(prefix[2].virtual_block.metadata.origin, 1);
                assert!(matches!(
                    prefix[3].cause,
                    DerivationCause::AcceptedBatch {
                        substituted: false,
                        record_index: 1,
                        ..
                    }
                ));
                assert_eq!(prefix[3].virtual_block.metadata.origin, 2);
                assert_eq!(prefix[3].native_block.kept, vec![transfer(1, 2, 0, 3)]);
            }
            _ => unreachable!(),
        }
        Ok(json!({
            "status": "PASS", "case": mode, "accepted": position, "reader_convergence": true,
            "derived": prefix,
        }))
    }
}

fn transfer(from: u8, to: u8, nonce: u64, amount: u64) -> Vec<u8> {
    [
        vec![1, from, to],
        nonce.to_be_bytes().to_vec(),
        amount.to_be_bytes().to_vec(),
    ]
    .concat()
}

fn check_pair(first: &ResumeCut, second: &ResumeCut) -> TestResult<()> {
    assert_eq!(first.resume_position, p(3, 0, 1));
    assert_eq!(first.cursor_set_position, p(3, 0, 0));
    assert_eq!(second.resume_position, p(3, 2, 1));
    assert_eq!(second.cursor_set_position, p(3, 2, 0));
    assert_eq!(
        (
            first.evidence.cursor_set_rpc_log_index,
            first.evidence.resume_rpc_log_index
        ),
        (0, 1)
    );
    assert_eq!(
        (
            second.evidence.cursor_set_rpc_log_index,
            second.evidence.resume_rpc_log_index
        ),
        (3, 4)
    );
    assert_ne!(
        first.evidence.transaction_hash,
        second.evidence.transaction_hash
    );
    assert_eq!(
        first.evidence.including_block,
        second.evidence.including_block
    );
    assert_eq!(second.target_height, 3);
    assert_eq!(second.derived_prefix.len(), 3);
    assert_eq!(first.derived_prefix[0], second.derived_prefix[0]);
    let mid = &second.derived_prefix[1];
    assert!(matches!(mid.cause, DerivationCause::AcceptedBatch {
        position, record_index: 0, substituted: false, ..
    } if position == p(3, 1, 0)));
    assert_eq!(mid.native_block.kept, vec![transfer(1, 2, 0, 3)]);
    assert_eq!(mid.virtual_block.metadata.origin, 1);
    assert_eq!(mid.native_block.parent, first.target_anchor.native_hash);
    assert_eq!(
        mid.state.accounts[&1],
        Account {
            nonce: 1,
            balance: 10
        }
    );
    assert_eq!(
        mid.state.accounts[&2],
        Account {
            nonce: 0,
            balance: 5
        }
    );
    assert_eq!(
        second.derived_prefix[2].native_block.parent,
        mid.native_hash
    );
    assert_eq!(second.target_anchor.state, second.derived_prefix[2].state);
    Ok(())
}

fn write(stdout: &mut impl Write, value: Value) -> TestResult<()> {
    writeln!(stdout, "{value}")?;
    stdout.flush()?;
    Ok(())
}

fn live(args: &[String]) -> TestResult<()> {
    let [endpoint, config_path, certifier, da, artifact] = args else {
        return Err("live expects endpoint, driver config, certifier, DA signer and output".into());
    };
    let config: Config = serde_json::from_reader(fs::File::open(config_path)?)?;
    let mut gateway = InboxGateway::connect(
        endpoint.parse()?,
        config.clone(),
        InboxSigners {
            certifier: decode(certifier)?,
            da: decode(da)?,
        },
    )?;
    let startup = gateway.drain_rpc_evidence();
    let raw = serde_json::to_value(&startup)?;
    let genesis_hash = raw
        .as_array()
        .ok_or("invalid startup RPC")?
        .iter()
        .find(|row| row["request"]["method"] == "eth_getBlockByNumber")
        .and_then(|row| row["result"]["Ok"]["hash"].as_str())
        .ok_or("genesis hash missing from actual RPC")?;
    let genesis = Hash32::new(decode(genesis_hash)?);
    let (derivation, _, _) = profile();
    let mut sources = BTreeMap::new();
    for reader in ["A", "B"] {
        let source = GatewaySource::new(&derivation, &config, reader.into(), genesis)
            .map_err(|err| format!("{err:?}"))?;
        sources.insert(reader.into(), source);
    }
    let mut session = Session {
        gateway,
        sources,
        accepted: BTreeMap::new(),
        artifact: artifact.into(),
    };
    let mut stdout = io::stdout().lock();
    write(&mut stdout, json!({"startup_rpc": startup}))?;
    let mut pair = None;
    for line in io::stdin().lock().lines() {
        let input: Value = serde_json::from_str(&line?)?;
        let output = match input["vector"].as_str() {
            Some("same_block") => {
                let (output, a, b) = session.paired_block()?;
                pair = Some((output.clone(), a, b));
                output
            }
            Some("same_block_tail") => {
                let (original, a, b) = pair.take().ok_or("missing initial cuts")?;
                session.verify_tail(&original, a, b)?
            }
            Some(mode) => session.native_case(mode)?,
            None => session.command(serde_json::from_value(input)?)?,
        };
        write(&mut stdout, output)?;
    }
    Ok(())
}

fn main() -> TestResult<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [mode] if mode == "profile" => {
            let (config, genesis, deposits) = profile();
            println!("{}", json!({
                "config": config, "genesis": genesis, "deposits": deposits,
                "config_ref": hex(config.config_ref().unwrap().as_bytes()),
            }));
            Ok(())
        }
        [mode, archive] if mode == "archive-identity" => archive_identity(Path::new(archive)),
        [mode, rest @ ..] if mode == "live" => live(rest),
        _ => Err("usage: m4-input-vectors profile | archive-identity RUN | live ENDPOINT CONFIG CERTIFIER DA ARTIFACT".into()),
    }
}
