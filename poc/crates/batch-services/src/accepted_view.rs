//! Accepted batch inputs from the actual feature 09 finalized reader.

use l1_adapter::{
    Action, Command, Config, Driver, Outcome, Query, Reply,
    rpc::{AnvilRpc, RpcEvidence},
};
use poc_types::domain::{Hash32, L1Position};
use serde::Serialize;
use serde_json::Value;
use std::net::SocketAddr;

use crate::{
    crypto::{
        Address, BatchWire, CryptoError, CursorWire, InboxDomain, InboxSigners, check_da,
        decode_fixed, decode_hex, submit_batch_calldata, validate_submission, verify_batch,
    },
    frame::{ClosedFrame, CodecError, verify_payload},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadError {
    Wait(String),
    Reject(String),
    Infrastructure(String),
}
impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ReadError {}
impl From<CryptoError> for ReadError {
    fn from(error: CryptoError) -> Self {
        Self::Reject(error.to_string())
    }
}

/// Real 09 gateway bound to one configured private Anvil instance. The caller
/// supplies the ordered command stream, including explicit finalization and delivery.
pub struct InboxGateway {
    driver: Driver<AnvilRpc>,
    domain: InboxDomain,
    submitter: Address,
    signers: InboxSigners,
}

impl InboxGateway {
    pub fn connect(
        endpoint: SocketAddr,
        config: Config,
        signers: InboxSigners,
    ) -> Result<Self, ReadError> {
        let domain = InboxDomain {
            chain_id: config.chain_id,
            inbox: decode_fixed(&config.inbox, "configured Inbox")?,
            config_ref: Hash32::new(decode_fixed(&config.config_ref, "configured configRef")?),
        };
        domain.separator()?;
        let submitter = decode_fixed(&config.submitter, "configured submitter")?;
        if signers.da == [0; 20] || signers.certifier == [0; 20] || signers.da == signers.certifier
        {
            return Err(ReadError::Reject(
                "invalid deployment signer configuration".into(),
            ));
        }
        let rpc = AnvilRpc::new(endpoint)
            .map_err(|error| ReadError::Infrastructure(error.to_string()))?;
        let driver = Driver::new(config, rpc).map_err(|fault| match fault {
            l1_adapter::Fault::Wait(reason) => ReadError::Wait(reason),
            l1_adapter::Fault::Reject(reason) => ReadError::Reject(reason),
            l1_adapter::Fault::Infrastructure(reason) => ReadError::Infrastructure(reason),
        })?;
        Ok(Self {
            driver,
            domain,
            submitter,
            signers,
        })
    }

    pub fn domain(&self) -> InboxDomain {
        self.domain
    }

    pub fn apply(&mut self, command: Command) -> Reply {
        self.driver.apply(command)
    }

    pub fn drain_rpc_evidence(&mut self) -> Vec<RpcEvidence> {
        std::mem::take(self.driver.rpc_evidence())
    }

    /// Returns the exact 09 reply alongside the accepted inputs for audit/replay.
    /// The host must assign `command_id` in the driver's ordered command stream.
    pub fn read_accepted_block(
        &mut self,
        command_id: u64,
        reader: &str,
        height: u64,
    ) -> (Reply, Result<Vec<AcceptedRecord>, ReadError>) {
        let reply = self.apply(Command {
            id: command_id,
            action: Action::Read {
                reader: reader.into(),
                query: Query::Block { height },
            },
        });
        let records = self.accepted_in_reply(&reply, reader, height);
        (reply, records)
    }

    fn accepted_in_reply(
        &self,
        reply: &Reply,
        reader: &str,
        height: u64,
    ) -> Result<Vec<AcceptedRecord>, ReadError> {
        let result = match &reply.outcome {
            Outcome::Ready { result } => result,
            Outcome::Wait { reason } => return Err(ReadError::Wait(reason.clone())),
            Outcome::Reject { reason } => return Err(ReadError::Reject(reason.clone())),
            Outcome::Infrastructure { reason } => {
                return Err(ReadError::Infrastructure(reason.clone()));
            }
        };
        let visible = reply
            .visible_heights
            .get(reader)
            .copied()
            .ok_or_else(|| reject("reader identity"))?;
        if height > visible || height > reply.finalized_height {
            return Err(ReadError::Wait(
                "reader has not received finalized block".into(),
            ));
        }
        let block = &result["block"];
        let header = &block["header_and_transactions"];
        let block_hash = hash(&block["hash"], "block hash")?;
        let timestamp = number(&block["timestamp"], "inclusion timestamp")?;
        if number(&result["control_coverage_through"], "coverage")? != height
            || number(&block["number"], "block height")? != height
            || quantity(&header["number"], "header height")? != height
            || quantity(&header["timestamp"], "header timestamp")? != timestamp
            || hash(&header["hash"], "header hash")? != block_hash
        {
            return Err(reject("block provenance mismatch"));
        }
        let mut last = None;
        let mut records = Vec::new();
        for event in block["events"]
            .as_array()
            .ok_or_else(|| reject("block events"))?
        {
            if event["event"] != "cursor_advanced" {
                continue;
            }
            let log = &event["position"];
            let position = L1Position {
                block: number(&log["block"], "log block")?,
                tx: number(&log["tx"], "transaction index")?,
                log: number(&log["log"], "local log index")?,
            };
            if position.block != height || last.is_some_and(|old| old >= position) {
                return Err(reject("noncanonical event position"));
            }
            last = Some(position);
            let tx_hash = hash(&log["tx_hash"], "transaction hash")?;
            let rpc_log_index = number(&log["rpc_log_index"], "RPC log index")?;
            let tx_index = usize::try_from(position.tx).map_err(|_| reject("transaction index"))?;
            let log_index = usize::try_from(position.log).map_err(|_| reject("local log index"))?;
            let tx = &header["transactions"][tx_index];
            let receipt = &block["receipts"][tx_index];
            let raw_log = &receipt["logs"][log_index];
            let batch = parse_batch(&event["batch"])?;
            let request_id = Hash32::new(decode_fixed(&batch.request_id, "request id")?);
            if address(&event["caller"], "caller")? != self.submitter
                || address(&tx["from"], "transaction sender")? != self.submitter
                || address(&tx["to"], "transaction target")? != self.domain.inbox
                || address(&raw_log["address"], "log emitter")? != self.domain.inbox
                || hash(&tx["hash"], "transaction hash")? != tx_hash
                || hash(&receipt["transactionHash"], "receipt transaction hash")? != tx_hash
                || hash(&receipt["blockHash"], "receipt block hash")? != block_hash
                || quantity(&receipt["status"], "receipt status")? != 1
                || quantity(&receipt["transactionIndex"], "receipt transaction index")?
                    != position.tx
                || quantity(&raw_log["logIndex"], "log RPC index")? != rpc_log_index
                || hash(&raw_log["topics"][2], "event request id")? != request_id
                || decode_hex(
                    string(&tx["input"], "transaction input")?,
                    8192,
                    "transaction input",
                )? != submit_batch_calldata(&batch)?
                || batch.next.origin > height
            {
                return Err(reject(
                    "CursorAdvanced transaction, receipt, or calldata mismatch",
                ));
            }
            // The 09 driver already paired event prev/next/commitment with this calldata.
            authenticate(&batch, &self.domain, self.signers)?;
            records.push(AcceptedRecord {
                identity: AcceptanceIdentity {
                    chain_id: self.domain.chain_id,
                    inbox: self.domain.inbox,
                    block_hash,
                    transaction_hash: tx_hash,
                    local_log: position.log,
                },
                config_ref: self.domain.config_ref,
                position,
                rpc_log_index,
                request_id,
                submission: batch,
                inclusion_timestamp_seconds: timestamp,
                finalized_through: reply.finalized_height,
                reader_visible_through: visible,
                evidence: AcceptedEvidence {
                    command_id: reply.id,
                    reader: reader.into(),
                    reference_us: reply.reference_us,
                    included_through: reply.included_height,
                    header: header.clone(),
                    transaction: tx.clone(),
                    receipt: receipt.clone(),
                },
                signers: self.signers,
            });
        }
        Ok(records)
    }
}

/// Stable identity across readers and repeated queries of the same canonical log.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct AcceptanceIdentity {
    pub chain_id: u64,
    pub inbox: Address,
    pub block_hash: Hash32,
    pub transaction_hash: Hash32,
    pub local_log: u64,
}

/// Closed construction from `InboxGateway::read_accepted_block`.
/// Serialized evidence is useful for audit but cannot be deserialized as trusted input.
#[derive(Clone, Debug, Serialize)]
pub struct AcceptedRecord {
    identity: AcceptanceIdentity,
    config_ref: Hash32,
    position: L1Position,
    rpc_log_index: u64,
    request_id: Hash32,
    submission: BatchWire,
    inclusion_timestamp_seconds: u64,
    finalized_through: u64,
    reader_visible_through: u64,
    evidence: AcceptedEvidence,
    #[serde(skip)]
    signers: InboxSigners,
}

#[derive(Clone, Debug, Serialize)]
pub struct AcceptedEvidence {
    pub command_id: u64,
    pub reader: String,
    pub reference_us: u64,
    pub included_through: u64,
    pub header: Value,
    pub transaction: Value,
    pub receipt: Value,
}

impl AcceptedRecord {
    pub fn identity(&self) -> AcceptanceIdentity {
        self.identity
    }
    pub fn position(&self) -> L1Position {
        self.position
    }
    pub fn rpc_log_index(&self) -> u64 {
        self.rpc_log_index
    }
    pub fn request_id(&self) -> Hash32 {
        self.request_id
    }
    pub fn submission(&self) -> &BatchWire {
        &self.submission
    }
    pub fn inclusion_timestamp_seconds(&self) -> u64 {
        self.inclusion_timestamp_seconds
    }
    pub fn finalized_through(&self) -> u64 {
        self.finalized_through
    }
    pub fn reader_visible_through(&self) -> u64 {
        self.reader_visible_through
    }
    pub fn evidence(&self) -> &AcceptedEvidence {
        &self.evidence
    }
    pub fn domain(&self) -> InboxDomain {
        InboxDomain {
            chain_id: self.identity.chain_id,
            inbox: self.identity.inbox,
            config_ref: self.config_ref,
        }
    }
    pub fn commitment(&self) -> Hash32 {
        Hash32::new(
            decode_fixed(&self.submission.commitment, "commitment").expect("verified commitment"),
        )
    }

    /// Attach DA bytes at this original input position. Missing or corrupt bytes are
    /// a WAIT at this position, so later inputs cannot use a fallback in their place.
    pub fn load_payload(&self, bytes: Option<&[u8]>) -> Result<AcceptedPayload, ReadError> {
        let Some(bytes) = bytes else {
            return Err(ReadError::Wait(format!(
                "DA unavailable at {:?}",
                self.position
            )));
        };
        let d = self.commitment();
        if crate::frame::commitment(bytes) != d {
            return Err(ReadError::Wait(format!(
                "DA commitment mismatch at {:?}",
                self.position
            )));
        }
        let frame = verify_payload(d, bytes).map_err(|error| match error {
            CodecError::CommitmentMismatch => {
                ReadError::Wait(format!("DA commitment mismatch at {:?}", self.position))
            }
            _ => ReadError::Reject(format!("accepted DA frame at {:?}: {error}", self.position)),
        })?;
        let first = frame
            .records
            .first()
            .expect("decode ensures one or two records");
        let last = frame
            .records
            .last()
            .expect("decode ensures one or two records");
        if first.block_number != self.submission.first_height
            || last.block_number != self.submission.last_height
            || last.epoch_num != self.submission.next.origin
        {
            return Err(reject(
                "accepted frame range or terminal origin contradicts submission",
            ));
        }
        let receipt = authenticate(&self.submission, &self.domain(), self.signers)?;
        Ok(AcceptedPayload {
            accepted: self.clone(),
            da: VerifiedDaObject {
                commitment: d,
                payload: bytes.into(),
                receipt,
            },
            frame,
        })
    }
}

#[derive(Clone, Debug)]
pub struct VerifiedDaObject {
    pub(crate) commitment: Hash32,
    pub(crate) payload: Vec<u8>,
    pub(crate) receipt: [u8; 65],
}

impl VerifiedDaObject {
    pub fn commitment(&self) -> Hash32 {
        self.commitment
    }
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
    pub fn receipt(&self) -> &[u8; 65] {
        &self.receipt
    }
}

#[derive(Clone, Debug)]
pub struct AcceptedPayload {
    accepted: AcceptedRecord,
    da: VerifiedDaObject,
    frame: ClosedFrame,
}

impl AcceptedPayload {
    pub fn accepted(&self) -> &AcceptedRecord {
        &self.accepted
    }
    pub fn da_object(&self) -> &VerifiedDaObject {
        &self.da
    }
    pub fn frame(&self) -> &ClosedFrame {
        &self.frame
    }
}

pub(crate) fn authenticate(
    batch: &BatchWire,
    domain: &InboxDomain,
    signers: InboxSigners,
) -> Result<[u8; 65], ReadError> {
    validate_submission(batch)?;
    let commitment = Hash32::new(decode_fixed(&batch.commitment, "commitment")?);
    let receipt = decode_fixed(&batch.da_certificate, "DA certificate")?;
    check_da(commitment, &receipt, domain, signers.da)?;
    verify_batch(batch, domain, signers.certifier)?;
    Ok(receipt)
}

pub(crate) fn parse_batch(wire: &Value) -> Result<BatchWire, ReadError> {
    Ok(BatchWire {
        request_id: string(&wire["request_id"], "request id")?.into(),
        prev: parse_cursor(&wire["prev"])?,
        next: parse_cursor(&wire["next"])?,
        commitment: string(&wire["commitment"], "commitment")?.into(),
        da_certificate: string(&wire["da_certificate"], "DA certificate")?.into(),
        first_height: number(&wire["first_height"], "first height")?,
        last_height: number(&wire["last_height"], "last height")?,
        gpos: string(&wire["gpos"], "gpos")?.into(),
        certifier_signature: string(&wire["certifier_signature"], "certifier signature")?.into(),
    })
}

pub(crate) fn parse_cursor(wire: &Value) -> Result<CursorWire, ReadError> {
    Ok(CursorWire {
        height: number(&wire["height"], "cursor height")?,
        origin: number(&wire["origin"], "cursor origin")?,
        frames_digest: string(&wire["frames_digest"], "frames digest")?.into(),
        source_kind: u8::try_from(number(&wire["source_kind"], "source kind")?)
            .map_err(|_| reject("source kind"))?,
        source: string(&wire["source"], "cursor source")?.into(),
    })
}

fn reject(label: &str) -> ReadError {
    ReadError::Reject(label.into())
}
fn string<'a>(value: &'a Value, label: &str) -> Result<&'a str, ReadError> {
    value.as_str().ok_or_else(|| reject(label))
}
fn number(value: &Value, label: &str) -> Result<u64, ReadError> {
    value.as_u64().ok_or_else(|| reject(label))
}
fn quantity(value: &Value, label: &str) -> Result<u64, ReadError> {
    let digits = string(value, label)?
        .strip_prefix("0x")
        .ok_or_else(|| reject(label))?;
    if digits.len() > 1 && digits.starts_with('0') {
        return Err(reject(label));
    }
    u64::from_str_radix(digits, 16).map_err(|_| reject(label))
}
fn hash(value: &Value, label: &'static str) -> Result<Hash32, ReadError> {
    Ok(Hash32::new(decode_fixed(string(value, label)?, label)?))
}
fn address(value: &Value, label: &'static str) -> Result<Address, ReadError> {
    Ok(decode_fixed(string(value, label)?, label)?)
}

/// A paired control in the 09 verified history, with both transaction-local positions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, Serialize)]
pub struct ResumeRequirement {
    pub set: L1Position,
    pub resume: L1Position,
    pub control_seq: u64,
}

/// One named reader's latest delivered cursor, including verified recovery history.
#[derive(Clone, Debug, Serialize)]
pub struct CurrentCursor {
    domain: InboxDomain,
    reader: String,
    as_of: u64,
    cursor: CursorWire,
    written_at: L1Position,
    latest_resume: Option<ResumeRequirement>,
    evidence: Value,
}

impl CurrentCursor {
    pub fn cursor(&self) -> &CursorWire {
        &self.cursor
    }
    pub fn domain(&self) -> InboxDomain {
        self.domain
    }
    pub fn reader(&self) -> &str {
        &self.reader
    }
    pub fn as_of(&self) -> u64 {
        self.as_of
    }
    pub fn written_at(&self) -> L1Position {
        self.written_at
    }
    pub fn latest_resume(&self) -> Option<ResumeRequirement> {
        self.latest_resume
    }
    pub fn evidence(&self) -> &Value {
        &self.evidence
    }
}

/// One complete, reader-visible finalized block from the concrete gateway.
#[derive(Clone, Debug, Serialize)]
pub struct FinalizedBlock {
    domain: InboxDomain,
    reader: String,
    height: u64,
    accepted: Vec<AcceptedRecord>,
    block: Value,
}

impl FinalizedBlock {
    pub fn domain(&self) -> InboxDomain {
        self.domain
    }
    pub fn reader(&self) -> &str {
        &self.reader
    }
    pub fn height(&self) -> u64 {
        self.height
    }
    pub fn accepted(&self) -> &[AcceptedRecord] {
        &self.accepted
    }
    pub fn raw_block(&self) -> &Value {
        &self.block["header_and_transactions"]
    }
    pub fn raw_receipts(&self) -> &[Value] {
        self.block["receipts"]
            .as_array()
            .expect("verified 09 receipt list")
    }

    pub fn overrides(&self) -> Result<Vec<ResumeRequirement>, ReadError> {
        parse_overrides(&self.block, self.height)
    }
}

impl InboxGateway {
    pub fn read_finalized_block(
        &mut self,
        command_id: u64,
        reader: &str,
        height: u64,
    ) -> (Reply, Result<FinalizedBlock, ReadError>) {
        let (reply, records) = self.read_accepted_block(command_id, reader, height);
        let result = records.map(|accepted| {
            let Outcome::Ready { result } = &reply.outcome else {
                unreachable!("accepted extraction requires ready reply")
            };
            FinalizedBlock {
                domain: self.domain,
                reader: reader.into(),
                height,
                accepted,
                block: result["block"].clone(),
            }
        });
        (reply, result)
    }

    /// Reads the current delivered cursor and proves latest-RESUME coverage by
    /// scanning the same reader's verified block history at/before `written_at`.
    /// Returned commands/replies preserve the exact queries for the run trace.
    pub fn read_current_cursor(
        &mut self,
        next_id: &mut u64,
        reader: &str,
        height: u64,
    ) -> (Vec<(Command, Reply)>, Result<CurrentCursor, ReadError>) {
        let mut trace = Vec::new();
        let domain = self.domain;
        let mut read = |query| {
            let id = *next_id;
            *next_id = id
                .checked_add(1)
                .ok_or_else(|| reject("driver command id exhausted"))?;
            let command = Command {
                id,
                action: Action::Read {
                    reader: reader.into(),
                    query,
                },
            };
            let reply = self.apply(command.clone());
            let result = match &reply.outcome {
                Outcome::Ready { result } => Ok((
                    result.clone(),
                    reply.visible_heights.get(reader).copied(),
                    reply.finalized_height,
                )),
                Outcome::Wait { reason } => Err(ReadError::Wait(reason.clone())),
                Outcome::Reject { reason } => Err(ReadError::Reject(reason.clone())),
                Outcome::Infrastructure { reason } => {
                    Err(ReadError::Infrastructure(reason.clone()))
                }
            };
            trace.push((command, reply));
            result
        };
        let result = (|| {
            let (result, visible, finalized) = read(Query::Cursor { height })?;
            if visible != Some(height) || height > finalized {
                return Err(ReadError::Wait(
                    "cursor request was not the current reader prefix".into(),
                ));
            }
            let evidence = &result["read_evidence"];
            if number(&result["as_of"], "cursor as_of")? != height
                || number(&evidence["chain_id"], "cursor evidence chain")? != domain.chain_id
                || address(&evidence["inbox"], "cursor evidence Inbox")? != domain.inbox
                || hash(&evidence["config_ref"], "cursor evidence configRef")? != domain.config_ref
            {
                return Err(reject("cursor evidence deployment mismatch"));
            }
            let cursor = parse_cursor(&result["cursor"])?;
            crate::crypto::validate_cursor(&cursor)?;
            let written_at = log_position(&result["written_at"])?;
            if written_at.block > height {
                return Err(reject("cursor written outside reader prefix"));
            }
            let latest_resume = latest_resume_in_history(written_at, |height| {
                let (result, _, _) = read(Query::Block { height })?;
                Ok(result["block"].clone())
            })?;
            Ok(CurrentCursor {
                domain,
                reader: reader.into(),
                as_of: height,
                cursor,
                written_at,
                latest_resume,
                evidence: result,
            })
        })();
        (trace, result)
    }
}

fn log_position(wire: &Value) -> Result<L1Position, ReadError> {
    Ok(L1Position {
        block: number(&wire["block"], "position block")?,
        tx: number(&wire["tx"], "position transaction")?,
        log: number(&wire["log"], "position log")?,
    })
}

fn latest_resume_in_history(
    written_at: L1Position,
    mut read_block: impl FnMut(u64) -> Result<Value, ReadError>,
) -> Result<Option<ResumeRequirement>, ReadError> {
    for height in (0..=written_at.block).rev() {
        let block = read_block(height)?;
        if let Some(control) = parse_overrides(&block, height)?
            .into_iter()
            .rev()
            .find(|control| control.set <= written_at)
        {
            return Ok(Some(control));
        }
    }
    Ok(None)
}

fn parse_overrides(block: &Value, height: u64) -> Result<Vec<ResumeRequirement>, ReadError> {
    if number(&block["number"], "override block height")? != height {
        return Err(reject("override block mismatch"));
    }
    let events = block["events"]
        .as_array()
        .ok_or_else(|| reject("override history events"))?;
    let mut controls = vec![];
    for event in events.iter().filter(|event| event["event"] == "cursor_set") {
        let set = log_position(&event["position"])?;
        let resume = log_position(&event["resume"])?;
        let matching = events
            .iter()
            .find(|other| other["event"] == "resume" && other["position"] == event["resume"])
            .ok_or_else(|| reject("missing paired RESUME in complete block"))?;
        if resume.block != height
            || set.block != height
            || set.tx != resume.tx
            || set.log >= resume.log
            || matching["set"] != event["position"]
        {
            return Err(reject("RESUME and CursorSet position mismatch"));
        }
        controls.push(ResumeRequirement {
            set,
            resume,
            control_seq: number(&matching["seq"], "RESUME control seq")?,
        });
    }
    Ok(controls)
}

/// Materialization of the accepted view from exact gpos fixture sources. Controls
/// retain the historical prefix while the matching ResumeCut is unavailable.
#[derive(Clone, Debug, Serialize)]
pub struct AcceptedView {
    pub cursor_read: CurrentCursor,
    pub genesis_height: u64,
    pub virtual_prefix: Vec<poc_types::domain::VirtualBlock>,
    pub historical_prefix: Vec<poc_types::domain::VirtualBlock>,
    pub missing_sources: Vec<MissingSource>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MissingSource {
    Gpos {
        position: L1Position,
        gpos: String,
        reason: String,
    },
    ResumeCut {
        control: ResumeRequirement,
    },
}

pub fn materialize_accepted_view(
    cursor_read: CurrentCursor,
    blocks: &[FinalizedBlock],
    source: &dyn crate::source::BatchSource,
    configuration: &crate::ServiceConfig,
) -> Result<AcceptedView, ReadError> {
    use crate::{
        frame::{commitment, encode_closed},
        source::{SourceQuery, replay_range},
    };
    if cursor_read.domain() != configuration.inbox {
        return Err(reject("materialization cursor deployment mismatch"));
    }
    if blocks.last().map(FinalizedBlock::height) != Some(cursor_read.as_of()) {
        return Err(ReadError::Wait(
            "materialization requires reader's complete finalized prefix".into(),
        ));
    }
    let mut initialized: Option<(u64, CursorWire, L1Position)> = None;
    let mut materialized = Vec::new();
    let mut materialized_tip: Option<(Hash32, Hash32)> = None;
    let mut historical = Vec::new();
    let mut missing = Vec::new();
    for (index, block) in blocks.iter().enumerate() {
        if block.domain() != configuration.inbox
            || block.reader() != cursor_read.reader()
            || block.height() != index as u64
            || (index > 0
                && block.raw_block()["parentHash"] != blocks[index - 1].raw_block()["hash"])
        {
            return Err(reject(
                "materialization requires one complete reader history",
            ));
        }
        for event in block.block["events"]
            .as_array()
            .expect("verified 09 event list")
        {
            match event["event"].as_str() {
                Some("cursor_initialized") => {
                    let genesis = parse_cursor(&event["cursor"])?;
                    let position = log_position(&event["position"])?;
                    if initialized
                        .replace((genesis.height, genesis, position))
                        .is_some()
                    {
                        return Err(reject("duplicate Inbox genesis"));
                    }
                }
                Some("cursor_set") => {
                    let position = log_position(&event["position"])?;
                    let control = block
                        .overrides()?
                        .into_iter()
                        .find(|control| control.set == position)
                        .ok_or_else(|| reject("unpaired CursorSet"))?;
                    if !materialized.is_empty() {
                        historical = std::mem::take(&mut materialized);
                    }
                    if !matches!(missing.last(), Some(MissingSource::ResumeCut { control: old }) if *old == control)
                    {
                        missing.push(MissingSource::ResumeCut { control });
                    }
                }
                Some("cursor_advanced") if missing.is_empty() => {
                    let position = log_position(&event["position"])?;
                    let record = block
                        .accepted
                        .iter()
                        .find(|record| record.position() == position)
                        .ok_or_else(|| {
                            reject("no authenticated accepted record at actual advance")
                        })?;
                    let Some((_, previous, previous_position)) = &mut initialized else {
                        return Err(reject("accepted advance before genesis"));
                    };
                    let batch = record.submission();
                    if &batch.prev != previous {
                        return Err(reject("materialization cursor discontinuity"));
                    }
                    let gpos = decode_hex(&batch.gpos, 1024, "gpos")?;
                    let query = SourceQuery {
                        gpos: &gpos,
                        first: batch.first_height,
                        last: batch.last_height,
                        cursor_written_at: *previous_position,
                        required_resume: None,
                    };
                    let replayed = match replay_range(source, &query, previous, configuration) {
                        Ok(replayed) => replayed,
                        Err(ReadError::Wait(reason)) => {
                            missing.push(MissingSource::Gpos {
                                position,
                                gpos: batch.gpos.clone(),
                                reason,
                            });
                            continue;
                        }
                        Err(error) => return Err(error),
                    };
                    let payload =
                        encode_closed(&replayed.records).map_err(|e| reject(&e.to_string()))?;
                    if commitment(&payload) != record.commitment()
                        || replayed.records.last().map(|record| record.epoch_num)
                            != Some(batch.next.origin)
                    {
                        return Err(reject(
                            "gpos source contradicts the accepted commitment or cursor",
                        ));
                    }
                    if let Some((state_root, native_hash)) = materialized_tip {
                        if replayed.virtual_blocks[0].pre_state_root != state_root
                            || replayed.native_parent != native_hash
                        {
                            return Err(reject(
                                "gpos source contradicts the already materialized prefix",
                            ));
                        }
                    }
                    let applied = replayed
                        .audit
                        .last()
                        .expect("replayed range includes one or more heights");
                    materialized_tip = Some((applied.post_state_root, applied.native_hash));
                    materialized.extend(replayed.virtual_blocks);
                    *previous = batch.next.clone();
                    *previous_position = position;
                }
                _ => (),
            }
        }
    }
    let (genesis_height, previous, _) =
        initialized.ok_or_else(|| ReadError::Wait("Inbox genesis unavailable".into()))?;
    if missing.is_empty() && &previous != cursor_read.cursor() {
        return Err(reject(
            "complete materialization does not match current cursor",
        ));
    }
    Ok(AcceptedView {
        cursor_read,
        genesis_height,
        virtual_prefix: materialized,
        historical_prefix: historical,
        missing_sources: missing,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn later_global_cursor_still_requires_exact_paired_resume() {
        let set = json!({"block":7,"tx":2,"log":0,"rpc_log_index":4,"tx_hash":"fixture"});
        let resume = json!({"block":7,"tx":2,"log":1,"rpc_log_index":5,"tx_hash":"fixture"});
        let control_block = json!({"number": 7,"events":[
            {"event":"cursor_set","position":set,"resume":resume},
            {"event":"resume","position":resume,"set":set,"seq":9}
        ]});
        let expected = ResumeRequirement {
            set: L1Position {
                block: 7,
                tx: 2,
                log: 0,
            },
            resume: L1Position {
                block: 7,
                tx: 2,
                log: 1,
            },
            control_seq: 9,
        };
        for written_at in [
            expected.set,
            L1Position {
                block: 8,
                tx: 0,
                log: 0,
            },
        ] {
            let result = latest_resume_in_history(written_at, |h| {
                Ok(match h {
                    7 => control_block.clone(),
                    8 => json!({"number":8,"events":[]}),
                    _ => panic!("cannot skip past required RESUME"),
                })
            })
            .unwrap();
            assert_eq!(result, Some(expected));
        }
        assert!(matches!(
            latest_resume_in_history(
                L1Position {
                    block: 8,
                    tx: 0,
                    log: 0
                },
                |h| {
                    if h == 8 {
                        Ok(json!({"number":8,"events":[]}))
                    } else {
                        Err(ReadError::Wait("missing complete finalized history".into()))
                    }
                }
            ),
            Err(ReadError::Wait(_))
        ));
    }
}
