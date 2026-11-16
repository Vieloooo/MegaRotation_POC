mod abi;
mod history;
pub mod rpc;

pub use abi::{BatchWire, CursorWire, PolicyWire};
pub use history::{BlockWire, CursorWrite, EventRecord, EventWire, LogWire};

use history::{bytes_hex, quantity};
use rpc::{Rpc, RpcError};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: u64,
    pub config_ref: String,
    pub authority: String,
    pub inbox: String,
    pub submitter: String,
    pub genesis_height: u64,
    pub genesis_origin: u64,
    pub genesis_timestamp_seconds: u64,
    pub slot_seconds: u64,
    pub announcement_seconds: u64,
    pub finality_delay_us: u64,
    pub readers: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub regional_policy: bool,
}

fn is_false(value: &bool) -> bool {
    !value
}

impl Config {
    fn validate(&mut self) -> Result<(), Fault> {
        if self.slot_seconds == 0 || self.announcement_seconds == 0 {
            return Err(Fault::Reject(
                "zero slot interval or announcement delay".into(),
            ));
        }
        for (value, size) in [
            (&mut self.authority, 20),
            (&mut self.inbox, 20),
            (&mut self.submitter, 20),
            (&mut self.config_ref, 32),
        ] {
            let bytes = abi::unhex(value)
                .filter(|bytes| bytes.len() == size && bytes.iter().any(|b| *b != 0))
                .ok_or_else(|| Fault::Reject("invalid configured address or configRef".into()))?;
            *value = abi::hex(&bytes);
        }
        if self.authority == self.inbox
            || self.readers.is_empty()
            || self.readers.iter().any(String::is_empty)
            || self.readers.iter().collect::<BTreeSet<_>>().len() != self.readers.len()
        {
            return Err(Fault::Reject(
                "invalid configured contract or reader identities".into(),
            ));
        }
        self.slot_tick(1)?;
        self.timestamp(1)?;
        Ok(())
    }

    fn slot_seconds(&self, slot: u64) -> Result<u64, Fault> {
        slot.checked_mul(self.slot_seconds)
            .ok_or_else(|| Fault::Reject("slot seconds overflow".into()))
    }

    fn timestamp(&self, slot: u64) -> Result<u64, Fault> {
        self.genesis_timestamp_seconds
            .checked_add(self.slot_seconds(slot)?)
            .filter(|seconds| seconds.checked_mul(1_000_000).is_some())
            .ok_or_else(|| Fault::Reject("header timestamp overflow".into()))
    }

    fn slot_tick(&self, slot: u64) -> Result<u64, Fault> {
        self.slot_seconds(slot)?
            .checked_mul(1_000_000)
            .ok_or_else(|| Fault::Reject("slot tick overflow".into()))
    }

    fn finality_tick(&self, slot: u64) -> Result<u64, Fault> {
        self.slot_tick(slot)?
            .checked_add(self.finality_delay_us)
            .ok_or_else(|| Fault::Reject("finality tick overflow".into()))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Command {
    pub id: u64,
    #[serde(flatten)]
    pub action: Action,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Tick {
        reference_us: u64,
    },
    Submit {
        raw: String,
    },
    Mine {
        slot: u64,
    },
    Finalize {
        through_height: u64,
    },
    Deliver {
        reader: String,
        through_height: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        from_height: Option<u64>,
    },
    Read {
        reader: String,
        query: Query,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Query {
    Block { height: u64 },
    Cursor { height: u64 },
    Event { height: u64, tx: u64, log: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "status", content = "reason", rename_all = "snake_case")]
pub enum Fault {
    Wait(String),
    Reject(String),
    Infrastructure(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Outcome {
    Ready { result: Value },
    Wait { reason: String },
    Reject { reason: String },
    Infrastructure { reason: String },
}

impl From<Result<Value, Fault>> for Outcome {
    fn from(value: Result<Value, Fault>) -> Self {
        match value {
            Ok(result) => Self::Ready { result },
            Err(Fault::Wait(reason)) => Self::Wait { reason },
            Err(Fault::Reject(reason)) => Self::Reject { reason },
            Err(Fault::Infrastructure(reason)) => Self::Infrastructure { reason },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Reply {
    pub id: u64,
    pub reference_us: u64,
    pub included_height: u64,
    pub finalized_height: u64,
    pub visible_heights: BTreeMap<String, u64>,
    #[serde(flatten)]
    pub outcome: Outcome,
}

#[derive(Default)]
struct Reader {
    visible: u64,
    received: BTreeSet<u64>,
}

#[derive(Clone)]
enum Pending {
    Submit { hash: String },
    Mine { slot: u64 },
}

/// One owner of Anvil writes. The owner supplies a single ordered command stream.
pub struct Driver<R> {
    config: Config,
    rpc: R,
    tick: u64,
    blocks: Vec<BlockWire>,
    finalized: u64,
    readers: BTreeMap<String, Reader>,
    commands: BTreeMap<u64, (Action, Reply)>,
    pending: Option<(u64, Pending)>,
    submitted: BTreeSet<String>,
}

impl<R: Rpc> Driver<R> {
    pub fn new(mut config: Config, rpc: R) -> Result<Self, Fault> {
        config.validate()?;
        let readers = config
            .readers
            .iter()
            .map(|reader| (reader.clone(), Reader::default()))
            .collect();
        let mut driver = Self {
            config,
            rpc,
            tick: 0,
            blocks: vec![],
            finalized: 0,
            readers,
            commands: BTreeMap::new(),
            pending: None,
            submitted: BTreeSet::new(),
        };
        if quantity(&driver.call("eth_chainId", json!([]))?) != Some(driver.config.chain_id) {
            return Err(Fault::Reject(
                "configured chain ID disagrees with Anvil".into(),
            ));
        }
        if driver.call("anvil_getAutomine", json!([]))? != false {
            return Err(Fault::Reject("Anvil automining is enabled".into()));
        }
        if driver.head()? != 0 {
            return Err(Fault::Reject(
                "fresh driver requires Anvil at genesis".into(),
            ));
        }
        let genesis = driver.fetch_block(0)?;
        if !genesis.receipts.is_empty() {
            return Err(Fault::Reject("unexpected genesis transactions".into()));
        }
        driver.blocks.push(genesis);
        Ok(driver)
    }

    /// Drain only backend-specific telemetry outside command execution.
    pub fn rpc_evidence(&mut self) -> &mut Vec<rpc::RpcEvidence>
    where
        R: EvidenceSource,
    {
        self.rpc.evidence()
    }

    pub fn apply(&mut self, command: Command) -> Reply {
        if let Some((action, old_reply)) = self.commands.get(&command.id) {
            if action != &command.action {
                return self.reply(
                    command.id,
                    Err(Fault::Reject("command id content conflict".into())),
                );
            }
            if self
                .pending
                .as_ref()
                .is_some_and(|(id, _)| *id == command.id)
            {
                let result = self.reconcile();
                let reply = self.reply(command.id, result);
                self.commands
                    .insert(command.id, (command.action, reply.clone()));
                return reply;
            }
            return old_reply.clone();
        }
        if self
            .commands
            .last_key_value()
            .is_some_and(|(id, _)| command.id <= *id)
        {
            return self.reply(
                command.id,
                Err(Fault::Reject("command id out of order".into())),
            );
        }
        let outcome = self.execute(command.id, &command.action);
        let reply = self.reply(command.id, outcome);
        self.commands
            .insert(command.id, (command.action, reply.clone()));
        reply
    }

    fn reply(&self, id: u64, outcome: Result<Value, Fault>) -> Reply {
        Reply {
            id,
            reference_us: self.tick,
            included_height: self.included(),
            finalized_height: self.finalized,
            visible_heights: self
                .readers
                .iter()
                .map(|(reader, state)| (reader.clone(), state.visible))
                .collect(),
            outcome: outcome.into(),
        }
    }

    fn included(&self) -> u64 {
        self.blocks.last().map_or(0, |block| block.number)
    }

    fn call(&mut self, method: &str, params: Value) -> Result<Value, Fault> {
        self.rpc
            .call(method, params)
            .map_err(|error| Fault::Infrastructure(format!("{method}: {error:?}")))
    }

    fn head(&mut self) -> Result<u64, Fault> {
        quantity(&self.call("eth_blockNumber", json!([]))?)
            .ok_or_else(|| Fault::Infrastructure("invalid Anvil block number".into()))
    }

    fn execute(&mut self, id: u64, action: &Action) -> Result<Value, Fault> {
        if self.pending.is_some() && matches!(action, Action::Submit { .. } | Action::Mine { .. }) {
            return Err(Fault::Wait(
                "resolve pending Anvil mutation using the original command id".into(),
            ));
        }
        match action {
            Action::Tick { reference_us } => {
                if *reference_us < self.tick {
                    return Err(Fault::Reject("reference tick moved backwards".into()));
                }
                self.tick = *reference_us;
                Ok(json!({"tick": self.tick}))
            }
            Action::Submit { raw } => self.submit(id, raw),
            Action::Mine { slot } => self.mine(id, *slot),
            Action::Finalize { through_height } => self.finalize(*through_height),
            Action::Deliver {
                reader,
                through_height,
                from_height,
            } => self.deliver(reader, *through_height, *from_height),
            Action::Read { reader, query } => self.read(reader, query),
        }
    }

    fn submit(&mut self, id: u64, raw: &str) -> Result<Value, Fault> {
        let bytes = abi::unhex(raw)
            .filter(|bytes| !bytes.is_empty())
            .ok_or_else(|| Fault::Reject("invalid signed transaction envelope".into()))?;
        let hash = abi::hex(&abi::hash(&bytes));
        if self.submitted.contains(&hash) {
            return Ok(json!({"submitted_tx_hash": hash}));
        }
        self.pending = Some((id, Pending::Submit { hash: hash.clone() }));
        match self
            .rpc
            .call("eth_sendRawTransaction", json!([abi::hex(&bytes)]))
        {
            Ok(result) if bytes_hex(&result, 32).as_ref() == Some(&hash) => {
                self.record_submit(hash)
            }
            Err(RpcError::Remote(error)) => {
                // An explicit JSON-RPC rejection from local Anvil ends this submission.
                self.pending = None;
                Err(Fault::Reject(format!(
                    "eth_sendRawTransaction rejected: {error}"
                )))
            }
            _ => Err(Fault::Wait(
                "unknown submit result; transaction hash must be reconciled".into(),
            )),
        }
    }

    fn record_submit(&mut self, hash: String) -> Result<Value, Fault> {
        self.submitted.insert(hash.clone());
        self.pending = None;
        Ok(json!({"submitted_tx_hash": hash}))
    }

    fn mine(&mut self, id: u64, slot: u64) -> Result<Value, Fault> {
        if self.included().checked_add(1) != Some(slot) {
            return Err(Fault::Reject(
                "mine requires consecutive slot number".into(),
            ));
        }
        if self.tick < self.config.slot_tick(slot)? {
            return Err(Fault::Reject("mine requested before slot tick".into()));
        }
        let timestamp = self.config.timestamp(slot)?;
        if self.head()? != self.included() {
            return Err(Fault::Reject(
                "Anvil head changed outside the driver".into(),
            ));
        }
        self.call(
            "evm_setNextBlockTimestamp",
            json!([format!("0x{timestamp:x}")]),
        )?;
        self.pending = Some((id, Pending::Mine { slot }));
        if self.rpc.call("evm_mine", json!([])).is_err() {
            return Err(Fault::Wait(
                "unknown mine result; reconcile canonical head".into(),
            ));
        }
        self.reconcile()
    }

    fn reconcile(&mut self) -> Result<Value, Fault> {
        match self
            .pending
            .clone()
            .expect("reconcile requires pending command")
            .1
        {
            Pending::Submit { hash } => {
                let tx = self.call("eth_getTransactionByHash", json!([hash]))?;
                if tx.is_null() {
                    return Err(Fault::Wait(
                        "unknown submit result; tx hash is still unavailable".into(),
                    ));
                }
                if bytes_hex(&tx["hash"], 32).as_ref() != Some(&hash) {
                    return Err(Fault::Reject(
                        "submission reconciliation returned wrong hash".into(),
                    ));
                }
                self.record_submit(hash)
            }
            Pending::Mine { slot } => {
                let head = self.head()?;
                if head < slot {
                    return Err(Fault::Wait("pending mine slot not yet available".into()));
                }
                if head != slot {
                    return Err(Fault::Reject(
                        "head advanced beyond pending mine slot".into(),
                    ));
                }
                let block = self.fetch_block(slot)?;
                let result = json!({"included_block": &block});
                self.blocks.push(block);
                self.pending = None;
                Ok(result)
            }
        }
    }

    fn fetch_block(&mut self, number: u64) -> Result<BlockWire, Fault> {
        let block = self.call(
            "eth_getBlockByNumber",
            json!([format!("0x{number:x}"), true]),
        )?;
        if block.is_null() {
            return Err(Fault::Wait("canonical block is unavailable".into()));
        }
        let transactions = block["transactions"]
            .as_array()
            .ok_or_else(|| Fault::Wait("full canonical transactions are unavailable".into()))?;
        let mut receipts = Vec::with_capacity(transactions.len());
        for tx in transactions {
            let hash = bytes_hex(&tx["hash"], 32)
                .ok_or_else(|| Fault::Wait("full transaction is unavailable".into()))?;
            let receipt = self.call("eth_getTransactionReceipt", json!([hash]))?;
            if receipt.is_null() {
                return Err(Fault::Wait(format!("receipt unavailable for {hash}")));
            }
            receipts.push(receipt);
        }
        history::verify_block(
            &self.config,
            number,
            self.config.timestamp(number)?,
            self.blocks.last(),
            block,
            receipts,
        )
    }

    fn finalize(&mut self, through: u64) -> Result<Value, Fault> {
        if through < self.finalized {
            return Err(Fault::Reject("finalized prefix moved backwards".into()));
        }
        if through > self.included() {
            return Err(Fault::Wait(
                "missing verified included block coverage".into(),
            ));
        }
        if through > self.finalized && self.tick < self.config.finality_tick(through)? {
            return Err(Fault::Reject("finality requested before slot delay".into()));
        }
        self.finalized = through;
        Ok(json!({"finalized_through": through}))
    }

    fn deliver(&mut self, reader: &str, through: u64, from: Option<u64>) -> Result<Value, Fault> {
        let state = self
            .readers
            .get_mut(reader)
            .ok_or_else(|| Fault::Reject("unknown reader".into()))?;
        if through > self.finalized {
            return Err(Fault::Wait(
                "requested delivery exceeds finalized coverage".into(),
            ));
        }
        if let Some(from) = from {
            if from > through {
                return Err(Fault::Reject("invalid delivery range".into()));
            }
            state.received.extend(from.max(state.visible + 1)..=through);
        } else {
            state.received.extend(state.visible + 1..=through);
        }
        while state.received.remove(&(state.visible + 1)) {
            state.visible += 1;
        }
        Ok(json!({"reader": reader, "delivered_through": state.visible}))
    }

    fn read(&mut self, reader: &str, query: &Query) -> Result<Value, Fault> {
        let state = self
            .readers
            .get(reader)
            .ok_or_else(|| Fault::Reject("unknown reader".into()))?;
        let height = match query {
            Query::Block { height } | Query::Cursor { height } | Query::Event { height, .. } => {
                *height
            }
        };
        if height > state.visible {
            return Err(Fault::Wait(
                "reader has not received complete finalized prefix".into(),
            ));
        }
        let block = &self.blocks[height as usize];
        match query {
            Query::Block { .. } => Ok(json!({"block": block, "control_coverage_through": height})),
            Query::Event { tx, log, .. } => {
                let record = block
                    .events
                    .iter()
                    .find(|record| record.position.tx == *tx && record.position.log == *log);
                Ok(json!({"record": record, "block_hash": block.hash}))
            }
            Query::Cursor { .. } => {
                let write = block.cursor_at_end.clone().ok_or_else(|| {
                    Fault::Wait("cursor not initialized at requested block".into())
                })?;
                let block_hash = block.hash.clone();
                let cursor_return = self.call(
                    "eth_call",
                    json!([
                        {"to": self.config.inbox, "data": abi::hex(&abi::selector("currentCursor()"))},
                        {"blockHash": block_hash, "requireCanonical": true}
                    ]),
                )?;
                if cursor_return.is_null() {
                    return Err(Fault::Wait("historical cursor state is unavailable".into()));
                }
                if history::data(&cursor_return)
                    .and_then(|data| abi::single_cursor(&data))
                    .as_ref()
                    != Some(&write.cursor)
                {
                    return Err(Fault::Reject(
                        "historical currentCursor disagrees with receipt replay".into(),
                    ));
                }
                let position = &write.written_at;
                let writing_block = &self.blocks[position.block as usize];
                Ok(json!({
                    "as_of": height,
                    "cursor": write.cursor,
                    "written_at": position,
                    "read_evidence": {
                        "block_hash": block_hash,
                        "writing_block_hash": writing_block.hash,
                        "writing_transaction": writing_block.header_and_transactions["transactions"][position.tx as usize],
                        "writing_receipt": writing_block.receipts[position.tx as usize],
                        "return_data": cursor_return,
                        "chain_id": self.config.chain_id,
                        "config_ref": self.config.config_ref,
                        "authority": self.config.authority,
                        "inbox": self.config.inbox,
                    },
                }))
            }
        }
    }
}

pub trait EvidenceSource {
    fn evidence(&mut self) -> &mut Vec<rpc::RpcEvidence>;
}

impl EvidenceSource for rpc::AnvilRpc {
    fn evidence(&mut self) -> &mut Vec<rpc::RpcEvidence> {
        &mut self.evidence
    }
}
