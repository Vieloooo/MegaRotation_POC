//! Replay the exact RPC and command transcript produced by the standalone 09 reader.
use l1_adapter::{
    Action, Command, Config, Query,
    rpc::{Rpc, RpcError},
};
use poc_core::fixture::ValidatedDepFixture;
use poc_derivation::{
    Checkpoint, Derivation, DerivationConfig, DriverSource, Outcome, Step, WaitReason,
};
use poc_types::domain::{
    Account, Anchor, BlockRef, CanonicalObject, Credit, DepFixture, ExecutionAnchor,
    ExecutionState, Hash32, L1Position, Metadata, PublicConfig,
};
use serde_json::{Value, json};
use std::{cell::RefCell, collections::VecDeque, fs, path::Path, rc::Rc};

const GENESIS_US: u64 = 1_000_000_000;

fn profile() -> (DerivationConfig, ExecutionAnchor, DepFixture) {
    let state = ExecutionState {
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
    };
    let genesis = ExecutionAnchor {
        anchor: Anchor {
            block: BlockRef {
                height: 0,
                origin: 0,
                block_id: Hash32::new([0x55; 32]),
            },
            timestamp_us: GENESIS_US,
        },
        state,
        native_hash: Hash32::new([0x11; 32]),
    };
    let mut deposits = DepFixture {
        first_height: 1,
        rows: vec![vec![]; 128],
    };
    deposits.rows[0] = vec![Credit { to: 2, amount: 2 }];
    deposits.rows[1] = vec![Credit { to: 1, amount: 3 }];
    deposits.rows[7] = vec![Credit { to: 0, amount: 0 }];
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
        DerivationConfig::new(public, 40).unwrap(),
        genesis,
        deposits,
    )
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::from("0x");
    for byte in bytes {
        use std::fmt::Write;
        write!(out, "{byte:02x}").unwrap();
    }
    out
}

fn decode_hash(text: &str) -> Result<Hash32, String> {
    let digits = text.strip_prefix("0x").ok_or("missing hex prefix")?;
    if digits.len() != 64 || !digits.is_ascii() {
        return Err("invalid hash".into());
    }
    let mut out = [0; 32];
    for (index, value) in out.iter_mut().enumerate() {
        *value =
            u8::from_str_radix(&digits[2 * index..2 * index + 2], 16).map_err(|e| e.to_string())?;
    }
    Ok(Hash32::new(out))
}

#[derive(Default)]
struct Transcript {
    rows: VecDeque<Value>,
    next_id: u64,
    mismatch: Option<String>,
}

#[derive(Clone, Default)]
struct ReplayRpc(Rc<RefCell<Transcript>>);

impl ReplayRpc {
    fn feed(&self, rows: &Value) -> Result<(), String> {
        self.drained()?;
        self.0.borrow_mut().rows = rows.as_array().ok_or("missing RPC rows")?.clone().into();
        Ok(())
    }
    fn drained(&self) -> Result<(), String> {
        let state = self.0.borrow();
        if let Some(error) = &state.mismatch {
            return Err(error.clone());
        }
        if !state.rows.is_empty() {
            return Err("unused RPC rows after driver command".into());
        }
        Ok(())
    }
}

impl Rpc for ReplayRpc {
    fn call(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        let mut state = self.0.borrow_mut();
        let id = state.next_id;
        state.next_id += 1;
        let expected = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        let row = state.rows.pop_front().unwrap_or(Value::Null);
        let request = &row["request"];
        let response = &row["response"];
        if request == &expected && response["jsonrpc"] == "2.0" && response["id"] == id {
            match (response.get("result"), response.get("error")) {
                (Some(value), None) if row["result"] == json!({"Ok": value}) => {
                    return Ok(value.clone());
                }
                (None, Some(error))
                    if error["code"].is_i64()
                        && row["result"] == json!({"Err": {"remote": error}}) =>
                {
                    return Err(RpcError::Remote(error.clone()));
                }
                _ => {}
            }
        }
        let message = format!("RPC transcript constraint failed at id {id}, method {method}");
        state.mismatch = Some(message.clone());
        Err(RpcError::InvalidResponse(message))
    }
}

fn require_ready<T>(outcome: Outcome<T>) -> Result<T, String> {
    match outcome {
        Outcome::Ready(value) => Ok(value),
        Outcome::Wait(wait) => Err(format!("unexpected WAIT {wait:?}")),
        Outcome::Reject(error) => Err(format!("unexpected REJECT {error:?}")),
    }
}

fn read_json(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(|e| e.to_string())
}

struct Run {
    config: DerivationConfig,
    genesis: ExecutionAnchor,
    dep: ValidatedDepFixture,
    rpc: ReplayRpc,
    source: DriverSource<ReplayRpc>,
    commands: Vec<Value>,
}

fn open_run(run: &Path, reader: &str) -> Result<Run, String> {
    let driver: Config = serde_json::from_value(read_json(&run.join("driver-config.json"))?)
        .map_err(|e| e.to_string())?;
    let recorded = read_json(&run.join("driver-trace.json"))?;
    let all_rows = recorded
        .as_array()
        .ok_or("driver transcript is not an array")?;
    let (startup, commands) = all_rows.split_first().ok_or("empty driver transcript")?;
    let boot = &startup["startup_rpc"];
    let genesis_hash = boot
        .as_array()
        .ok_or("missing startup RPCs")?
        .iter()
        .find(|row| {
            row["request"]["method"] == "eth_getBlockByNumber"
                && row["request"]["params"] == json!(["0x0", true])
        })
        .and_then(|row| row["response"]["result"]["hash"].as_str())
        .ok_or("missing actual L1 genesis hash")?;
    let (config, genesis, deposits) = profile();
    let dep = ValidatedDepFixture::load(config.public(), deposits).map_err(|e| format!("{e:?}"))?;
    let rpc = ReplayRpc::default();
    rpc.feed(boot)?;
    let source = DriverSource::new(
        &config,
        driver,
        reader.into(),
        decode_hash(genesis_hash)?,
        rpc.clone(),
    )
    .map_err(|e| format!("{e:?}"))?;
    rpc.drained()?;
    Ok(Run {
        config,
        genesis,
        dep,
        rpc,
        source,
        commands: commands.to_vec(),
    })
}

fn recorded_command(
    row: &Value,
    rpc: &ReplayRpc,
    source: &mut DriverSource<ReplayRpc>,
    command: Command,
) -> Result<l1_adapter::Reply, String> {
    rpc.feed(&row["rpc"])?;
    let reply = source.apply(command).map_err(|e| format!("{e:?}"))?;
    rpc.drained()?;
    if serde_json::to_value(&reply).map_err(|e| e.to_string())? != row["reply"] {
        return Err(format!("09 driver reply mismatch on command {}", reply.id));
    }
    Ok(reply)
}

fn replay(run: &Path, reader: &str, output: &Path, restore: Option<&Path>) -> Result<(), String> {
    let Run {
        config,
        genesis,
        dep,
        rpc,
        mut source,
        commands,
    } = open_run(run, reader)?;
    let mut state = None;
    let mut steps = Vec::new();
    let mut snapshots = Vec::new();
    let mut cuts = Vec::new();
    let mut checkpoints = Vec::new();
    let mut last_wait = None;
    let mut saved_restart = restore.map(read_json).transpose()?;
    let mut restart_done = false;
    let mut first_fallback = false;
    for row in &commands {
        let command: Command =
            serde_json::from_value(row["command"].clone()).map_err(|e| e.to_string())?;
        let read_height = match &command.action {
            Action::Read {
                reader: name,
                query: Query::Block { height },
            } if name == reader => Some(*height),
            _ => None,
        };
        let reply = recorded_command(row, &rpc, &mut source, command)?;
        let Some(height) = read_height else {
            continue;
        };
        let inputs = source.inputs();
        if state.is_none() {
            match Derivation::init(config.clone(), genesis.clone(), dep.clone(), &inputs) {
                Outcome::Ready(initial) => state = Some(initial),
                Outcome::Wait(_) => continue,
                Outcome::Reject(error) => return Err(format!("genesis rejected {error:?}")),
            }
        }
        let d = state.as_mut().unwrap();
        loop {
            match d.step(&inputs) {
                Outcome::Ready(step) => {
                    let checkpoint = match &step {
                        Step::CursorSet {
                            resume_position, ..
                        } => {
                            cuts.push(require_ready(d.derive_at_resume(*resume_position))?);
                            true
                        }
                        Step::Fallback { .. } if !first_fallback => {
                            first_fallback = true;
                            true
                        }
                        _ => false,
                    };
                    steps.push(step);
                    if checkpoint {
                        let actual = d.checkpoint();
                        let encoded = serde_json::to_value(&actual).map_err(|e| e.to_string())?;
                        let stored: Checkpoint =
                            serde_json::from_value(encoded.clone()).map_err(|e| e.to_string())?;
                        let d0 = require_ready(Derivation::restore(
                            &stored,
                            config.clone(),
                            genesis.clone(),
                            dep.clone(),
                            &inputs,
                        ))?;
                        if d0.checkpoint() != actual {
                            return Err("checkpoint roundtrip mismatch".into());
                        }
                        for cut in &cuts {
                            if require_ready(d0.derive_at_resume(cut.resume_position))? != *cut {
                                return Err("historical cut changed on restart".into());
                            }
                        }
                        *d = d0;
                        checkpoints.push(encoded.clone());
                        if saved_restart.as_ref() == Some(&encoded) {
                            let prior: Checkpoint =
                                serde_json::from_value(saved_restart.take().unwrap())
                                    .map_err(|error| error.to_string())?;
                            *d = require_ready(Derivation::restore(
                                &prior,
                                config.clone(),
                                genesis.clone(),
                                dep.clone(),
                                &inputs,
                            ))?;
                            restart_done = true;
                        }
                    }
                }
                Outcome::Wait(wait) => {
                    let detail = match &wait.reason {
                        WaitReason::CompleteWindow { through } => {
                            json!({"kind":"complete_window", "through": through})
                        }
                        WaitReason::CompleteBlock(height) => {
                            json!({"kind":"complete_block", "height": height})
                        }
                        WaitReason::Feature11AcceptedPayloadRequired { commitment } => {
                            json!({"kind":"feature_11_accepted_payload_required", "commitment": commitment})
                        }
                        why => return Err(format!("unexpected replay wait {why:?}")),
                    };
                    snapshots.push(json!({"requested": height, "block_read": reply.outcome,
                        "frontier": d.complete_frontier(), "head": d.head().anchor.block.height,
                        "epoch": d.current_epoch(), "epoch_has_output": d.epoch_has_output(),
                        "waiting": {"position": wait.position, "reason": detail}}));
                    last_wait = Some(wait);
                    break;
                }
                Outcome::Reject(error) => return Err(format!("replay rejected {error:?}")),
            }
        }
    }
    if saved_restart.is_some() {
        return Err("saved checkpoint position never reached".into());
    }
    let d = state.ok_or("no usable reader genesis")?;
    for cut in &cuts {
        if require_ready(d.derive_at_resume(cut.resume_position))? != *cut {
            return Err("historical cut changed as D grew".into());
        }
        let mistaken_global = L1Position {
            log: cut.evidence.resume_rpc_log_index,
            ..cut.resume_position
        };
        if mistaken_global != cut.resume_position
            && !matches!(d.derive_at_resume(mistaken_global), Outcome::Reject(_))
        {
            return Err("block-global log index was accepted as transaction-local".into());
        }
    }
    if let Some(wait) = last_wait.as_ref().filter(|wait| {
        matches!(
            wait.reason,
            WaitReason::Feature11AcceptedPayloadRequired { .. }
        )
    }) {
        let recovered = require_ready(Derivation::restore(
            &d.checkpoint(),
            config,
            genesis,
            dep,
            &source.inputs(),
        ))?;
        let next = recovered.derive_at_resume(L1Position {
            block: 40,
            tx: 1,
            log: 1,
        });
        if next != Outcome::Wait(wait.clone()) {
            return Err("pending accepted obligation disappeared after replay".into());
        }
    }
    let native: Vec<_> = d.derived_prefix().iter().map(|item| {
        Ok(json!({"native_hash": hex(item.native_hash.as_bytes()), "native_bytes": hex(&item.native_block.encode_checked().map_err(|e| format!("{e:?}"))?),
            "state_root":hex(item.native_block.post_state_root.as_bytes()), "metadata":item.native_block.metadata,
            "derived_anchor_id":hex(poc_types::domain::derived_anchor_id(item.state.metadata.height, item.native_hash).as_bytes()) }))
    }).collect::<Result<_, String>>()?;
    write_json(
        output,
        &json!({"head": d.head(), "native": native, "derived": d.derived_prefix(), "cuts":cuts,
        "canonical_checkpoint": d.checkpoint(), "action_trace":steps, "snapshots":snapshots, "checkpoints":checkpoints,
        "independent_process_checkpoint_replayed":restart_done}),
    )
}

/// Reconcile the same 09 pending mine after delivering its withheld receipt.
fn receipt_gap(run: &Path, output: &Path) -> Result<(), String> {
    const HEIGHT: u64 = 12;
    const READER: &str = "A";
    let Run {
        config,
        genesis,
        dep,
        rpc,
        mut source,
        commands,
    } = open_run(run, READER)?;
    let mut withheld = None;
    for row in &commands {
        let command: Command =
            serde_json::from_value(row["command"].clone()).map_err(|e| e.to_string())?;
        if command.action != (Action::Mine { slot: HEIGHT }) {
            recorded_command(row, &rpc, &mut source, command)?;
            continue;
        }
        let mut rows = row["rpc"].as_array().ok_or("missing mine RPCs")?.clone();
        let last = rows.last().ok_or("empty mine RPCs")?;
        if last["request"]["method"] != "eth_getTransactionReceipt"
            || last["response"]["result"].is_null()
        {
            return Err("expected target block to contain one verified receipt".into());
        }
        let head = rows
            .iter()
            .rposition(|rpc| rpc["request"]["method"] == "eth_blockNumber")
            .ok_or("missing reconciliation head")?;
        let mut reconciliation = rows[head..].to_vec();
        let last = rows.last_mut().unwrap();
        last["response"]["result"] = Value::Null;
        last["result"] = json!({"Ok":null});
        rpc.feed(&json!(rows))?;
        let waited = source
            .apply(command.clone())
            .map_err(|e| format!("{e:?}"))?;
        rpc.drained()?;
        if !matches!(waited.outcome, l1_adapter::Outcome::Wait { .. })
            || waited.included_height != HEIGHT - 1
        {
            return Err("09 inclusion crossed a missing receipt".into());
        }
        let start = rpc.0.borrow().next_id;
        for (index, row) in reconciliation.iter_mut().enumerate() {
            let id = start + index as u64;
            row["request"]["id"] = id.into();
            row["response"]["id"] = id.into();
        }
        withheld = Some((command, waited, reconciliation));
        break;
    }
    let (command, waited, reconciliation) =
        withheld.ok_or("receipt-gap run has no target block")?;
    let mut id = u64::MAX - 100;
    let mut cmd = |src: &mut DriverSource<ReplayRpc>, action| {
        id += 1;
        src.apply(Command { id, action })
            .map_err(|e| format!("{e:?}"))
    };
    let read = |height| Action::Read {
        reader: READER.into(),
        query: Query::Block { height },
    };
    let deliver = |height| Action::Deliver {
        reader: READER.into(),
        through_height: height,
        from_height: None,
    };
    use l1_adapter::Outcome::{Ready as L1Ready, Wait as L1Wait};
    if !matches!(
        cmd(
            &mut source,
            Action::Tick {
                reference_us: (HEIGHT + 1) * 2_000_000
            }
        )?
        .outcome,
        L1Ready { .. }
    ) || !matches!(
        cmd(
            &mut source,
            Action::Finalize {
                through_height: HEIGHT
            }
        )?
        .outcome,
        L1Wait { .. }
    ) || !matches!(
        cmd(
            &mut source,
            Action::Finalize {
                through_height: HEIGHT - 1
            }
        )?
        .outcome,
        L1Ready { .. }
    ) || !matches!(
        cmd(&mut source, deliver(HEIGHT - 1))?.outcome,
        L1Ready { .. }
    ) {
        return Err("09 accepted incomplete finality or declined complete prefix".into());
    }
    for number in 0..HEIGHT {
        if !matches!(cmd(&mut source, read(number))?.outcome, L1Ready { .. }) {
            return Err("unexpected absent block preceding receipt gap".into());
        }
    }
    if !matches!(cmd(&mut source, read(HEIGHT))?.outcome, L1Wait { .. }) {
        return Err("reader crossed receipt gap".into());
    }
    let mut d = require_ready(Derivation::init(config, genesis, dep, &source.inputs()))?;
    let before = d.advance(&source.inputs());
    let initial = d.head();
    if !before.new_items.is_empty()
        || d.complete_frontier().map(|r| r.height) != Some(HEIGHT - 1)
        || !matches!(before.status, Outcome::Wait(_))
    {
        return Err("D advanced past missing receipt".into());
    }
    rpc.feed(&json!(reconciliation))?;
    let reconciled = source.apply(command).map_err(|e| format!("{e:?}"))?;
    rpc.drained()?;
    if !matches!(reconciled.outcome, L1Ready { .. })
        || !matches!(
            cmd(
                &mut source,
                Action::Finalize {
                    through_height: HEIGHT
                }
            )?
            .outcome,
            L1Ready { .. }
        )
        || !matches!(cmd(&mut source, deliver(HEIGHT))?.outcome, L1Ready { .. })
        || !matches!(cmd(&mut source, read(HEIGHT))?.outcome, L1Ready { .. })
    {
        return Err("D input failed to reconcile from the genuine receipt".into());
    }
    let after = d.advance(&source.inputs());
    if after.new_items.len() != 20
        || after.new_cuts.len() != 1
        || d.head().anchor.block.origin != 10
    {
        return Err("D did not apply the resolved marker at its own position".into());
    }
    write_json(
        output,
        &json!({"injected":"missing_receipt_in_local_09_rpc_replay",
        "target_block":HEIGHT, "actual_mine_withheld":waited, "actual_mine_reconciled":reconciled,
        "before":initial, "after":d.head(), "resolved_cut":after.new_cuts[0]}),
    )
}

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("profile") if args.len() == 2 => {
            let (config, genesis, deposits) = profile();
            println!("{}", json!({"config_ref":hex(config.config_ref().unwrap().as_bytes()), "config":config, "genesis":genesis, "deposits":deposits}));
            Ok(())
        }
        Some("replay") if args.len() == 5 || args.len() == 6 => replay(Path::new(&args[2]), &args[3], Path::new(&args[4]), args.get(5).map(Path::new)),
        Some("receipt-gap") if args.len() == 4 => receipt_gap(Path::new(&args[2]), Path::new(&args[3])),
        _ => Err("usage: derivation-replay profile | replay REAL_RUN_DIR READER OUTPUT_JSON [CHECKPOINT_JSON] | receipt-gap REAL_WINDOW_RUN_DIR OUTPUT_JSON".into()),
    }
}
