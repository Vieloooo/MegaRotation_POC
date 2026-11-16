use crate::{DerivationConfig, DerivationIdentity, L1_ADAPTER_REVISION, L1Identity, Reject};
use l1_adapter::{
    Action, BatchWire, BlockWire, Command, Config, CursorWire, Driver, EventRecord, EventWire,
    Fault, LogWire, Outcome as DriverOutcome, PolicyWire, Query, Reply, rpc::Rpc,
};
use poc_core::fixture::{FixtureFinalizer, FixtureReader, ValidatedFixture};
use poc_types::domain::{FinalizedL1Reader, Hash32, InputStatus, L1BlockRef, L1Fixture, L1Header};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DriverSourceError {
    Binding(Reject),
    Adapter(Fault),
}

/// Owns the checked 09 driver and collects only successful named reader block queries.
pub struct DriverSource<R> {
    driver: Driver<R>,
    reader: String,
    identity: DerivationIdentity,
    blocks: BTreeMap<u64, ObservedBlock>,
}

impl<R: Rpc> DriverSource<R> {
    pub fn new(
        derivation: &DerivationConfig,
        driver: Config,
        reader: String,
        expected_l1_genesis: Hash32,
        rpc: R,
    ) -> Result<Self, DriverSourceError> {
        let invalid = || DriverSourceError::Binding(Reject::Configuration("L1 driver binding"));
        if !driver.readers.contains(&reader)
            || driver.genesis_height != derivation.public().genesis.block.height
            || driver.genesis_origin != derivation.public().genesis.block.origin
        {
            return Err(invalid());
        }
        let l1 = L1Identity::Driver {
            chain_id: driver.chain_id,
            config_ref: hash32(&driver.config_ref).ok_or_else(invalid)?.into(),
            inbox: hex_array(&driver.inbox).ok_or_else(invalid)?,
            authority: hex_array(&driver.authority).ok_or_else(invalid)?,
            submitter: hex_array(&driver.submitter).ok_or_else(invalid)?,
            l1_genesis_hash: expected_l1_genesis,
            l1_genesis_timestamp_s: driver.genesis_timestamp_seconds,
            slot_seconds: driver.slot_seconds,
            announcement_seconds: driver.announcement_seconds,
            finality_delay_us: driver.finality_delay_us,
            adapter_revision: L1_ADAPTER_REVISION.into(),
        };
        let identity =
            DerivationIdentity::new(derivation, l1).map_err(DriverSourceError::Binding)?;
        Ok(Self {
            driver: Driver::new(driver, rpc).map_err(DriverSourceError::Adapter)?,
            reader,
            identity,
            blocks: BTreeMap::new(),
        })
    }

    /// Run an explicitly ordered driver command. Other readers' replies remain diagnostics.
    pub fn apply(&mut self, command: Command) -> Result<Reply, Reject> {
        let named_block = match &command.action {
            Action::Read {
                reader,
                query: Query::Block { height },
            } if reader == &self.reader => Some(*height),
            _ => None,
        };
        let reply = self.driver.apply(command);
        if let (Some(height), DriverOutcome::Ready { result }) = (named_block, &reply.outcome) {
            if reply.finalized_height < height
                || reply
                    .visible_heights
                    .get(&self.reader)
                    .is_none_or(|n| *n < height)
                || result["control_coverage_through"].as_u64() != Some(height)
            {
                return Err(Reject::Source("driver misreported reader coverage"));
            }
            let block = read_block(&result["block"])?;
            if block.header.height != height {
                return Err(Reject::Source("driver returned a different block"));
            }
            if height == 0
                && !matches!(&self.identity.l1, L1Identity::Driver { l1_genesis_hash, .. }
                    if *l1_genesis_hash == block.header.hash)
            {
                return Err(Reject::Source("L1 genesis hash mismatch"));
            }
            match self.blocks.entry(height) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(block);
                }
                std::collections::btree_map::Entry::Occupied(entry)
                    if entry.get().stamp == block.stamp => {}
                _ => return Err(Reject::Source("conflicting verified block read")),
            }
        }
        Ok(reply)
    }

    pub fn inputs(&self) -> VerifiedInputs<'_> {
        VerifiedInputs {
            identity: &self.identity,
            view: InputView::Driver(&self.blocks),
        }
    }
}

/// Explicitly labeled finite 03 fixture source with separate header and receipt delivery.
pub struct FixtureSource {
    finalizer: FixtureFinalizer,
    reader: FixtureReader,
    identity: DerivationIdentity,
}

impl FixtureSource {
    pub fn new(config: &DerivationConfig, fixture: L1Fixture) -> Result<Self, Reject> {
        let commitment = fixture.digest()?;
        let source = ValidatedFixture::load(config.public(), fixture)?;
        Ok(Self {
            finalizer: source.finalizer(),
            reader: source.reader(),
            identity: DerivationIdentity::new(config, L1Identity::Fixture { commitment })?,
        })
    }

    /// Deliver a finalized mock header, optionally with its committed empty receipts.
    pub fn deliver(
        &mut self,
        header: &L1Header,
        finalized_at_us: u64,
        delivered_at_us: u64,
        complete: bool,
    ) -> Result<(), Reject> {
        let finalized = self.finalizer.finalize(header, finalized_at_us)?;
        self.reader.deliver_header(&finalized, delivered_at_us)?;
        if complete {
            self.reader.deliver_empty_control(
                &finalized,
                poc_types::domain::empty_control_commitment(),
                delivered_at_us,
            )?;
        }
        Ok(())
    }

    pub fn inputs(&self) -> VerifiedInputs<'_> {
        VerifiedInputs {
            identity: &self.identity,
            view: InputView::Fixture(&self.reader),
        }
    }
}

/// Borrowed source capability. This type has no public constructor from JSON or a block.
pub struct VerifiedInputs<'a> {
    identity: &'a DerivationIdentity,
    view: InputView<'a>,
}

#[derive(Clone, Copy)]
enum InputView<'a> {
    Driver(&'a BTreeMap<u64, ObservedBlock>),
    Fixture(&'a FixtureReader),
    #[cfg(test)]
    Cases {
        headers: &'a BTreeMap<u64, L1Header>,
        blocks: &'a BTreeMap<u64, ObservedBlock>,
    },
}

impl VerifiedInputs<'_> {
    pub fn identity(&self) -> &DerivationIdentity {
        self.identity
    }

    pub fn header(&self, height: u64) -> InputStatus<L1Header> {
        use poc_types::domain::InputMissing::L1Header as MissingHeader;
        match self.view {
            InputView::Driver(blocks) => blocks
                .get(&height)
                .map_or(InputStatus::Wait(MissingHeader(height)), |block| {
                    InputStatus::Ready(block.header.clone())
                }),
            InputView::Fixture(reader) => reader.header(height),
            #[cfg(test)]
            InputView::Cases { headers, .. } => headers
                .get(&height)
                .cloned()
                .map_or(InputStatus::Wait(MissingHeader(height)), InputStatus::Ready),
        }
    }

    pub(crate) fn block(&self, height: u64) -> InputStatus<std::borrow::Cow<'_, ObservedBlock>> {
        use poc_types::domain::InputMissing::ControlCoverage;
        use std::borrow::Cow;
        match self.view {
            InputView::Driver(blocks) => blocks
                .get(&height)
                .map_or(InputStatus::Wait(ControlCoverage(height)), |block| {
                    InputStatus::Ready(Cow::Borrowed(block))
                }),
            InputView::Fixture(reader) => match reader.require_control_coverage(height) {
                InputStatus::Ready(()) => match reader.header(height) {
                    InputStatus::Ready(header) => {
                        InputStatus::Ready(Cow::Owned(ObservedBlock::empty(header)))
                    }
                    InputStatus::Wait(missing) => InputStatus::Wait(missing),
                    InputStatus::Reject(error) => InputStatus::Reject(error),
                },
                InputStatus::Wait(missing) => InputStatus::Wait(missing),
                InputStatus::Reject(error) => InputStatus::Reject(error),
            },
            #[cfg(test)]
            InputView::Cases { blocks, .. } => blocks
                .get(&height)
                .map_or(InputStatus::Wait(ControlCoverage(height)), |block| {
                    InputStatus::Ready(Cow::Borrowed(block))
                }),
        }
    }
}

#[cfg(test)]
impl<'a> VerifiedInputs<'a> {
    pub(crate) fn cases(
        identity: &'a DerivationIdentity,
        headers: &'a BTreeMap<u64, L1Header>,
        blocks: &'a BTreeMap<u64, ObservedBlock>,
    ) -> Self {
        Self {
            identity,
            view: InputView::Cases { headers, blocks },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InputStamp {
    pub reference: L1BlockRef,
    pub read_evidence_digest: Hash32,
}

#[derive(Clone, Debug)]
pub(crate) struct ObservedBlock {
    pub header: L1Header,
    pub stamp: InputStamp,
    pub events: Vec<EventRecord>,
}

impl ObservedBlock {
    pub(crate) fn empty(header: L1Header) -> Self {
        Self {
            stamp: InputStamp {
                reference: header.reference(),
                read_evidence_digest: poc_types::domain::empty_control_commitment(),
            },
            header,
            events: vec![],
        }
    }
}

pub(crate) fn digest(domain: &str, bytes: &[u8]) -> Hash32 {
    let output = poc_types::hash_parts(domain, &[bytes]);
    let mut hash = [0; 32];
    hash.copy_from_slice(&output);
    Hash32::new(hash)
}

// 09 exports Serialize on its result types. Convert its in-process JSON reply back to
// those existing types. This function is never a public verified source constructor.
fn read_block(value: &Value) -> Result<ObservedBlock, Reject> {
    const INVALID: Reject = Reject::Source("invalid typed 09 reader reply");
    let block = parse_block(value).ok_or(INVALID)?;
    let hash = hash32(&block.hash).ok_or(INVALID)?;
    let raw = &block.header_and_transactions;
    let header = L1Header {
        height: block.number,
        hash,
        parent_hash: raw["parentHash"].as_str().and_then(hash32).ok_or(INVALID)?,
        timestamp_s: block.timestamp,
    };
    let transactions = raw["transactions"].as_array().ok_or(INVALID)?;
    if transactions.len() != block.receipts.len()
        || block
            .events
            .iter()
            .any(|event| event.position.block != block.number)
        || block
            .events
            .windows(2)
            .any(|pair| position(&pair[0].position) >= position(&pair[1].position))
    {
        return Err(INVALID);
    }
    let bytes = serde_json::to_vec(&block).map_err(|_| INVALID)?;
    Ok(ObservedBlock {
        stamp: InputStamp {
            reference: header.reference(),
            read_evidence_digest: digest("PoC/Derivation/L1ReadEvidence/v1", &bytes),
        },
        header,
        events: block.events,
    })
}

pub(crate) fn position(log: &LogWire) -> poc_types::domain::L1Position {
    poc_types::domain::L1Position {
        block: log.block,
        tx: log.tx,
        log: log.log,
    }
}

pub(crate) fn hex_array<const N: usize>(value: &str) -> Option<[u8; N]> {
    let bytes = value.strip_prefix("0x")?.as_bytes();
    if bytes.len() != N * 2 {
        return None;
    }
    let mut out = [0; N];
    for (slot, pair) in out.iter_mut().zip(bytes.chunks_exact(2)) {
        *slot = ((pair[0] as char).to_digit(16)? * 16 + (pair[1] as char).to_digit(16)?) as u8;
    }
    Some(out)
}

pub(crate) fn hash32(value: &str) -> Option<Hash32> {
    hex_array(value).map(Hash32::new)
}

fn string(value: &Value, key: &str) -> Option<String> {
    Some(value[key].as_str()?.to_owned())
}

fn parse_log(value: &Value) -> Option<LogWire> {
    Some(LogWire {
        block: value["block"].as_u64()?,
        tx: value["tx"].as_u64()?,
        log: value["log"].as_u64()?,
        rpc_log_index: value["rpc_log_index"].as_u64()?,
        tx_hash: string(value, "tx_hash")?,
    })
}

fn parse_cursor(value: &Value) -> Option<CursorWire> {
    Some(CursorWire {
        height: value["height"].as_u64()?,
        origin: value["origin"].as_u64()?,
        frames_digest: string(value, "frames_digest")?,
        source_kind: value["source_kind"].as_u64()?.try_into().ok()?,
        source: string(value, "source")?,
    })
}

fn parse_policy(value: &Value) -> Option<PolicyWire> {
    Some(PolicyWire {
        enabled_mask: value["enabled_mask"].as_u64()?.try_into().ok()?,
        order: serde_json::from_value(value["order"].clone()).ok()?,
        l1_floor: value["l1_floor"].as_u64()?,
    })
}

fn parse_batch(value: &Value) -> Option<BatchWire> {
    Some(BatchWire {
        request_id: string(value, "request_id")?,
        prev: parse_cursor(&value["prev"])?,
        next: parse_cursor(&value["next"])?,
        commitment: string(value, "commitment")?,
        da_certificate: string(value, "da_certificate")?,
        first_height: value["first_height"].as_u64()?,
        last_height: value["last_height"].as_u64()?,
        gpos: string(value, "gpos")?,
        certifier_signature: string(value, "certifier_signature")?,
    })
}

fn parse_event(value: &Value) -> Option<EventRecord> {
    let event = match value["event"].as_str()? {
        "cursor_initialized" => EventWire::CursorInitialized {
            cursor: parse_cursor(&value["cursor"])?,
        },
        "cursor_advanced" => EventWire::CursorAdvanced {
            caller: string(value, "caller")?,
            batch: Box::new(parse_batch(&value["batch"])?),
        },
        "cursor_set" => EventWire::CursorSet {
            old: parse_cursor(&value["old"])?,
            next: parse_cursor(&value["next"])?,
            resume: parse_log(&value["resume"])?,
        },
        "force" => EventWire::Force {
            caller: string(value, "caller")?,
            request_id: string(value, "request_id")?,
            seq: value["seq"].as_u64()?,
            policy: parse_policy(&value["policy"])?,
            branch: value["branch"].as_u64()?,
            act: value["act"].as_u64()?,
        },
        "resume" => EventWire::Resume {
            caller: string(value, "caller")?,
            request_id: string(value, "request_id")?,
            seq: value["seq"].as_u64()?,
            policy: parse_policy(&value["policy"])?,
            height: value["height"].as_u64()?,
            origin: value["origin"].as_u64()?,
            act: value["act"].as_u64()?,
            set: parse_log(&value["set"])?,
        },
        _ => return None,
    };
    Some(EventRecord {
        position: parse_log(&value["position"])?,
        event,
    })
}

fn parse_block(value: &Value) -> Option<BlockWire> {
    let written = &value["cursor_at_end"];
    Some(BlockWire {
        number: value["number"].as_u64()?,
        hash: string(value, "hash")?,
        timestamp: value["timestamp"].as_u64()?,
        header_and_transactions: value["header_and_transactions"].clone(),
        receipts: value["receipts"].as_array()?.clone(),
        events: value["events"]
            .as_array()?
            .iter()
            .map(parse_event)
            .collect::<Option<_>>()?,
        cursor_at_end: if written.is_null() {
            None
        } else {
            Some(l1_adapter::CursorWrite {
                cursor: parse_cursor(&written["cursor"])?,
                written_at: parse_log(&written["written_at"])?,
            })
        },
        last_control_seq: value["last_control_seq"].as_u64(),
    })
}
