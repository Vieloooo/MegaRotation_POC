use crate::{
    Config, Fault,
    abi::{
        self, ADVANCED, BatchWire, CursorWire, FORCE, INITIALIZED, PolicyWire, RESUME, SET, hex,
        unhex,
    },
};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;

pub(crate) fn quantity(value: &Value) -> Option<u64> {
    let digits = value.as_str()?.strip_prefix("0x")?;
    if digits.len() > 1 && digits.starts_with('0') {
        return None;
    }
    u64::from_str_radix(digits, 16).ok()
}

pub(crate) fn data(value: &Value) -> Option<Vec<u8>> {
    unhex(value.as_str()?)
}

pub(crate) fn bytes_hex(value: &Value, size: usize) -> Option<String> {
    let bytes = data(value)?;
    (bytes.len() == size).then(|| hex(&bytes))
}

fn check(condition: bool, label: &str) -> Result<(), Fault> {
    if condition {
        Ok(())
    } else {
        Err(Fault::Reject(label.into()))
    }
}

fn checked<T>(value: Option<T>, label: &str) -> Result<T, Fault> {
    value.ok_or_else(|| Fault::Reject(label.into()))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LogWire {
    pub block: u64,
    pub tx: u64,
    pub log: u64,
    pub rpc_log_index: u64,
    pub tx_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum EventWire {
    CursorInitialized {
        cursor: CursorWire,
    },
    CursorAdvanced {
        caller: String,
        batch: Box<BatchWire>,
    },
    CursorSet {
        old: CursorWire,
        next: CursorWire,
        resume: LogWire,
    },
    Force {
        caller: String,
        request_id: String,
        seq: u64,
        policy: PolicyWire,
        branch: u64,
        act: u64,
    },
    Resume {
        caller: String,
        request_id: String,
        seq: u64,
        policy: PolicyWire,
        height: u64,
        origin: u64,
        act: u64,
        set: LogWire,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EventRecord {
    pub position: LogWire,
    #[serde(flatten)]
    pub event: EventWire,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CursorWrite {
    pub cursor: CursorWire,
    pub written_at: LogWire,
}

#[derive(Clone, Debug, Serialize)]
pub struct BlockWire {
    pub number: u64,
    pub hash: String,
    pub timestamp: u64,
    pub header_and_transactions: Value,
    pub receipts: Vec<Value>,
    pub events: Vec<EventRecord>,
    pub cursor_at_end: Option<CursorWrite>,
    pub last_control_seq: Option<u64>,
}

struct ResumeBody {
    caller: String,
    request_id: String,
    seq: u64,
    policy: PolicyWire,
    height: u64,
    origin: u64,
    act: u64,
}

enum Decoded {
    CursorInitialized(CursorWire),
    CursorAdvanced(String, Box<BatchWire>),
    CursorSet(CursorWire, CursorWire),
    Force(EventWire),
    Resume(ResumeBody),
}

struct Decoder {
    initialized: String,
    advanced: String,
    set: String,
    force: String,
    resume: String,
}

impl Decoder {
    fn new() -> Self {
        Self {
            initialized: abi::topic(INITIALIZED),
            advanced: abi::topic(ADVANCED),
            set: abi::topic(SET),
            force: abi::topic(FORCE),
            resume: abi::topic(RESUME),
        }
    }

    fn event(
        &self,
        config: &Config,
        emitter: &str,
        topics: &[String],
        data: &[u8],
        tx: &Value,
        block: u64,
        timestamp: u64,
    ) -> Option<Option<Decoded>> {
        let Some(topic) = topics.first() else {
            return (emitter != config.authority && emitter != config.inbox).then_some(None);
        };
        if emitter == config.inbox {
            if topic == &self.initialized && topics.len() == 1 {
                return Some(Some(Decoded::CursorInitialized(abi::single_cursor(data)?)));
            }
            if topic == &self.set && topics.len() == 1 {
                let (prev, next) = abi::cursor_pair(data, false)?;
                return Some(Some(Decoded::CursorSet(prev, next)));
            }
            if topic == &self.advanced && topics.len() == 3 {
                let caller = topic_address(&topics[1])?;
                let batch = abi::batch_calldata(&self::data(&tx["input"])?)?;
                let (prev, next) = abi::cursor_pair(data, true)?;
                let commitment = hex(abi::word(data, 2)?);
                if batch.prev != prev
                    || batch.next != next
                    || batch.commitment != commitment
                    || batch.request_id != topics[2]
                    || caller != config.submitter
                    || bytes_hex(&tx["from"], 20)? != caller
                    || bytes_hex(&tx["to"], 20)? != config.inbox
                    || batch.next.source_kind != 1
                    || batch.next.source != batch.gpos
                    || prev.height.checked_add(1)? != batch.first_height
                    || next.height != batch.last_height
                    || batch.last_height.checked_sub(batch.first_height)? > 1
                    || next.origin > block
                {
                    return None;
                }
                return Some(Some(Decoded::CursorAdvanced(caller, Box::new(batch))));
            }
            return None;
        }
        if emitter == config.authority {
            if topics.len() != 4 {
                return None;
            }
            let caller = topic_address(&topics[1])?;
            let request_id = topics[2].clone();
            let seq = abi::uint(&unhex(&topics[3])?.try_into().ok()?)?;
            let expected_act = timestamp.checked_add(config.announcement_seconds)?;
            if topic == &self.force && data.len() == 7 * 32 {
                let act = abi::uint_at(data, 6)?;
                if act != expected_act {
                    return None;
                }
                return Some(Some(Decoded::Force(EventWire::Force {
                    caller,
                    request_id,
                    seq,
                    policy: abi::policy_at(data, 1, config.regional_policy)?,
                    branch: abi::uint_at(data, 0)?,
                    act,
                })));
            }
            if topic == &self.resume && data.len() == 8 * 32 {
                let act = abi::uint_at(data, 7)?;
                let origin = abi::uint_at(data, 6)?;
                if act != expected_act || origin > block {
                    return None;
                }
                return Some(Some(Decoded::Resume(ResumeBody {
                    caller,
                    request_id,
                    seq,
                    policy: abi::policy_at(data, 0, config.regional_policy)?,
                    height: abi::uint_at(data, 5)?,
                    origin,
                    act,
                })));
            }
            return None;
        }
        Some(None)
    }
}

fn topic_address(topic: &str) -> Option<String> {
    let word = unhex(topic)?;
    if word.len() != 32 || word[..12] != [0; 12] {
        return None;
    }
    Some(hex(&word[12..]))
}

fn require_tx_link(value: &Value, hash: &str, number: u64, tx_index: u64) -> Result<(), Fault> {
    check(
        bytes_hex(&value["blockHash"], 32).as_deref() == Some(hash)
            && quantity(&value["blockNumber"]) == Some(number)
            && quantity(&value["transactionIndex"]) == Some(tx_index),
        "RPC transaction identity disagrees with canonical block",
    )
}

pub(crate) fn verify_block(
    config: &Config,
    number: u64,
    timestamp: u64,
    previous: Option<&BlockWire>,
    block: Value,
    receipts: Vec<Value>,
) -> Result<BlockWire, Fault> {
    let hash = checked(bytes_hex(&block["hash"], 32), "malformed block hash")?;
    let parent = checked(bytes_hex(&block["parentHash"], 32), "malformed parent hash")?;
    check(
        quantity(&block["number"]) == Some(number)
            && quantity(&block["timestamp"]) == Some(timestamp)
            && previous.is_none_or(|previous| previous.hash == parent),
        "block number, timestamp or parent mismatch",
    )?;
    let transactions = checked(
        block["transactions"].as_array(),
        "missing full block transactions",
    )?;
    if transactions.len() != receipts.len() {
        return Err(Fault::Wait("incomplete receipt coverage".into()));
    }

    let decoder = Decoder::new();
    let mut cursor = previous.and_then(|block| block.cursor_at_end.clone());
    let mut seq = previous.and_then(|block| block.last_control_seq);
    let mut all_events = Vec::new();
    let mut seen_hashes = BTreeSet::new();
    let mut global_log = 0;
    for (index, (tx, receipt)) in transactions.iter().zip(&receipts).enumerate() {
        let index = index as u64;
        let tx_hash = checked(
            bytes_hex(&tx["hash"], 32),
            "malformed full transaction hash",
        )?;
        require_tx_link(tx, &hash, number, index)?;
        require_tx_link(receipt, &hash, number, index)?;
        check(
            seen_hashes.insert(tx_hash.clone())
                && bytes_hex(&receipt["transactionHash"], 32).as_ref() == Some(&tx_hash),
            "receipt does not uniquely bind the ordered transaction",
        )?;
        check(
            data(&tx["input"]).is_some()
                && bytes_hex(&tx["from"], 20).is_some()
                && (tx["to"].is_null() || bytes_hex(&tx["to"], 20).is_some()),
            "missing transaction input or invalid transaction addresses",
        )?;
        if receipt["logs"].is_null() {
            return Err(Fault::Wait("receipt log data is unavailable".into()));
        }
        let logs = checked(receipt["logs"].as_array(), "invalid receipt logs")?;
        if quantity(&receipt["status"]) == Some(0) {
            check(logs.is_empty(), "failed transaction has surviving logs")?;
            continue;
        }
        check(
            quantity(&receipt["status"]) == Some(1),
            "invalid receipt status",
        )?;

        let mut decoded = Vec::new();
        for (local_log, log) in logs.iter().enumerate() {
            require_tx_link(log, &hash, number, index)?;
            check(
                bytes_hex(&log["transactionHash"], 32).as_ref() == Some(&tx_hash)
                    && quantity(&log["logIndex"]) == Some(global_log)
                    && log["removed"] == false,
                "receipt log identity or order mismatch",
            )?;
            let position = LogWire {
                block: number,
                tx: index,
                log: local_log as u64,
                rpc_log_index: global_log,
                tx_hash: tx_hash.clone(),
            };
            global_log += 1;
            let emitter = checked(bytes_hex(&log["address"], 20), "malformed emitter")?;
            let data = checked(data(&log["data"]), "invalid log data")?;
            let topics: Vec<_> = checked(log["topics"].as_array(), "malformed log topics")?
                .iter()
                .map(|value| checked(bytes_hex(value, 32), "malformed log topic"))
                .collect::<Result<_, _>>()?;
            if let Some(event) = checked(
                decoder.event(config, &emitter, &topics, &data, tx, number, timestamp),
                "invalid event from configured contract",
            )? {
                decoded.push((position, event));
            }
        }
        verify_transaction(
            config,
            tx,
            receipt,
            decoded,
            &mut cursor,
            &mut seq,
            &mut all_events,
        )?;
    }
    Ok(BlockWire {
        number,
        hash,
        timestamp,
        header_and_transactions: block,
        receipts,
        events: all_events,
        cursor_at_end: cursor,
        last_control_seq: seq,
    })
}

fn verify_transaction(
    config: &Config,
    tx: &Value,
    receipt: &Value,
    decoded: Vec<(LogWire, Decoded)>,
    cursor: &mut Option<CursorWrite>,
    seq: &mut Option<u64>,
    all_events: &mut Vec<EventRecord>,
) -> Result<(), Fault> {
    // Stage one receipt as a unit. Pair by the RESUME source seq and check the
    // complete receipt before exposing either member.
    let mut pending_set = None;
    for (position, event) in decoded {
        let event = match event {
            Decoded::CursorInitialized(initial) => {
                check(
                    cursor.is_none()
                        && initial.height == config.genesis_height
                        && initial.origin == config.genesis_origin
                        && initial.origin <= position.block
                        && initial.source_kind == 0
                        && tx["to"].is_null()
                        && bytes_hex(&receipt["contractAddress"], 20).as_ref()
                            == Some(&config.authority),
                    "cursor initialization disagrees with deployment",
                )?;
                *cursor = Some(CursorWrite {
                    cursor: initial.clone(),
                    written_at: position.clone(),
                });
                EventWire::CursorInitialized { cursor: initial }
            }
            Decoded::CursorAdvanced(caller, batch) => {
                check(
                    pending_set.is_none()
                        && cursor
                            .as_ref()
                            .is_some_and(|write| write.cursor == batch.prev),
                    "CursorAdvanced prev disagrees with receipt history",
                )?;
                *cursor = Some(CursorWrite {
                    cursor: batch.next.clone(),
                    written_at: position.clone(),
                });
                EventWire::CursorAdvanced { caller, batch }
            }
            Decoded::CursorSet(old, next) => {
                check(
                    pending_set.is_none()
                        && cursor.as_ref().is_some_and(|write| write.cursor == old)
                        && next.height > old.height
                        && next.origin <= position.block,
                    "inconsistent pending CursorSet",
                )?;
                pending_set = Some((position, old, next));
                continue;
            }
            Decoded::Force(EventWire::Force {
                caller,
                request_id,
                seq: accepted,
                policy,
                branch,
                act,
            }) => {
                check(
                    pending_set.is_none() && seq.is_none_or(|previous| previous < accepted),
                    "invalid FORCE sequence or interrupted RESUME pair",
                )?;
                *seq = Some(accepted);
                EventWire::Force {
                    caller,
                    request_id,
                    seq: accepted,
                    policy,
                    branch,
                    act,
                }
            }
            Decoded::Force(_) => unreachable!("decoder always produces a Force"),
            Decoded::Resume(ResumeBody {
                caller,
                request_id,
                seq: accepted,
                policy,
                height,
                origin,
                act,
            }) => {
                let (set, old, next) = checked(
                    pending_set.take(),
                    "RESUME is missing its same-receipt CursorSet",
                )?;
                check(
                    next.resume_seq() == Some(accepted)
                        && next.height == height
                        && next.origin == origin
                        && seq.is_none_or(|previous| previous < accepted),
                    "CursorSet and RESUME disagree",
                )?;
                *seq = Some(accepted);
                *cursor = Some(CursorWrite {
                    cursor: next.clone(),
                    written_at: set.clone(),
                });
                all_events.push(EventRecord {
                    position: set.clone(),
                    event: EventWire::CursorSet {
                        old,
                        next,
                        resume: position.clone(),
                    },
                });
                EventWire::Resume {
                    caller,
                    request_id,
                    seq: accepted,
                    policy,
                    height,
                    origin,
                    act,
                    set,
                }
            }
        };
        all_events.push(EventRecord { position, event });
    }
    check(
        pending_set.is_none(),
        "CursorSet is missing its same-receipt RESUME",
    )
}
