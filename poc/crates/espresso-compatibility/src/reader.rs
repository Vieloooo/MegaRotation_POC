use std::{
    collections::BTreeMap,
    ops::Bound::{Excluded, Unbounded},
    sync::Arc,
};

use base64::{Engine, engine::general_purpose::STANDARD};
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};
use sim_espresso::{
    MAX_BLOCKS, MAX_LOG_BYTES, MAX_MESSAGE_BYTES, MAX_MESSAGES, WireError, validate_message_list,
};

use crate::{
    CaptureKind, CaptureRecord, CaptureRecordRef, Error, EspressoPosition, FIXTURE_CODEC,
    FinalityBasis, LogMessage, MAX_CAPTURE_FILE_BYTES, MissingData, NativeBlock, NativePosition,
    NativeTransaction, PUBLICATION_NAMESPACE, ProjectedMessage, PublicTimeUs, SourceBinding,
    UnknownTiming, UnsupportedTimingEvidence, export_pe_log, export_pe_reader, sha256_hex, tagged,
    valid_id,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReceiveOutcome {
    Stored,
    Duplicate,
    Unavailable,
    Submitted { transaction_hash: String },
    SubmitRejected { status: u16 },
    HeightHint(u64),
    Diagnostic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Endpoint {
    Block(u64),
    Leaf(u64),
    Transaction(u64, u64),
    TransactionByHash,
    Height,
    Submit,
    DevInfo,
    L1,
}

pub(crate) fn endpoint(method: &str, path: &str) -> Result<Endpoint, Error> {
    fn number(value: &str) -> Result<u64, Error> {
        let number = value
            .parse::<u64>()
            .map_err(|_| invalid("invalid API position in request path"))?;
        if number.to_string() != value {
            return Err(invalid("noncanonical API position in request path"));
        }
        Ok(number)
    }
    if path.len() > 2_048 {
        return Err(invalid("request path too long"));
    }
    let parts: Vec<_> = path.split('/').collect();
    match (method, parts.as_slice()) {
        ("GET", ["", "v1", "availability", "block", q]) => Ok(Endpoint::Block(number(q)?)),
        ("GET", ["", "v1", "availability", "leaf", q]) => Ok(Endpoint::Leaf(number(q)?)),
        (
            "GET",
            [
                "",
                "v1",
                "availability",
                "transaction",
                "hash",
                hash,
                "noproof",
            ],
        ) if tagged(hash, "TX~") => Ok(Endpoint::TransactionByHash),
        ("GET", ["", "v1", "availability", "transaction", q, i, "noproof"]) => {
            Ok(Endpoint::Transaction(number(q)?, number(i)?))
        }
        ("GET", ["", "v1", "node", "block-height"]) => Ok(Endpoint::Height),
        ("POST", ["", "v1", "submit", "submit"]) => Ok(Endpoint::Submit),
        ("GET", ["", "api", "dev-info"]) => Ok(Endpoint::DevInfo),
        ("POST", ["", ""]) => Ok(Endpoint::L1),
        _ => Err(Error::UnsupportedRepresentation(format!(
            "unpinned API endpoint {method} {path}"
        ))),
    }
}

fn invalid(reason: &str) -> Error {
    Error::InvalidSource(reason.into())
}

/// Parse one document, disallowing ambiguous repeated keys even in the opaque
/// header, leaf and transaction fields compared on repeat reads.
pub(crate) fn strict_json(bytes: &[u8]) -> Result<Value, serde_json::Error> {
    struct Strict(Value);

    impl<'de> Deserialize<'de> for Strict {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct StrictVisitor;

            impl<'de> Visitor<'de> for StrictVisitor {
                type Value = Strict;

                fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    formatter.write_str("a JSON value with unique object keys")
                }

                fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
                where
                    A: MapAccess<'de>,
                {
                    let mut values = Map::new();
                    while let Some((key, Strict(value))) = access.next_entry()? {
                        if values.insert(key, value).is_some() {
                            return Err(de::Error::custom("duplicate JSON object key"));
                        }
                    }
                    Ok(Strict(Value::Object(values)))
                }

                fn visit_seq<A>(self, mut access: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let mut values = Vec::new();
                    while let Some(Strict(value)) = access.next_element()? {
                        values.push(value);
                    }
                    Ok(Strict(Value::Array(values)))
                }

                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
                    Ok(Strict(Value::from(value)))
                }

                fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
                    Ok(Strict(Value::from(value)))
                }

                fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
                    Ok(Strict(Value::from(value)))
                }

                fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                    Ok(Strict(Value::from(value)))
                }

                fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                    Ok(Strict(Value::from(value)))
                }

                fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
                    Ok(Strict(Value::from(value)))
                }

                fn visit_unit<E>(self) -> Result<Self::Value, E> {
                    Ok(Strict(Value::Null))
                }
            }

            deserializer.deserialize_any(StrictVisitor)
        }
    }

    serde_json::from_slice(bytes).map(|Strict(value)| value)
}

fn json(bytes: &[u8]) -> Result<Value, Error> {
    strict_json(bytes).map_err(|err| Error::InvalidSource(err.to_string()))
}

fn object<'a>(value: &'a Value, key: &str) -> Result<&'a Value, Error> {
    value
        .get(key)
        .filter(|field| field.is_object())
        .ok_or_else(|| Error::InvalidSource(format!("missing or invalid object {key}")))
}

fn number(value: &Value, key: &str) -> Result<u64, Error> {
    value[key]
        .as_u64()
        .ok_or_else(|| Error::InvalidSource(format!("missing or invalid integer {key}")))
}

fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str, Error> {
    value[key]
        .as_str()
        .ok_or_else(|| Error::InvalidSource(format!("missing or invalid string {key}")))
}

fn tagged_string<'a>(value: &'a Value, key: &str, tag: &str) -> Result<&'a str, Error> {
    let value = string(value, key)?;
    if !tagged(value, tag) {
        return Err(Error::InvalidSource(format!("invalid tagged {key}")));
    }
    Ok(value)
}

/// Parse the selected 0.3 header fields. The complete header is retained as JSON.
#[derive(Clone, Copy)]
struct Header {
    theta: PublicTimeUs,
    seconds: u64,
    millis: Option<u64>,
}

impl Header {
    fn parse(raw: &Value, q: u64) -> Result<Self, Error> {
        let version = object(object(raw, "version")?, "Version")?;
        if number(version, "major")? != 0 || number(version, "minor")? != 3 {
            return Err(Error::UnsupportedRepresentation(
                "expected Espresso header version 0.3".into(),
            ));
        }
        let fields = object(raw, "fields")?;
        if number(fields, "height")? != q {
            return Err(invalid("header height does not match request"));
        }
        let seconds = number(fields, "timestamp")?;
        let theta = PublicTimeUs(
            seconds
                .checked_mul(1_000_000)
                .ok_or_else(|| invalid("timestamp seconds overflow microseconds"))?,
        );
        let millis = fields
            .get("timestamp_millis")
            .map(|_| number(fields, "timestamp_millis"))
            .transpose()?;
        Ok(Self {
            theta,
            seconds,
            millis,
        })
    }
}

struct Observed<T> {
    raw: Value,
    data: T,
    reference: CaptureRecordRef,
}

struct Block {
    header: Header,
    hash: String,
    count: u32,
}

#[derive(Default)]
struct Slot {
    block: Option<Observed<Block>>,
    leaf: Option<Observed<Header>>,
    txs: BTreeMap<u32, Observed<NativeTransaction>>,
}

impl Slot {
    fn theta(&self) -> Option<PublicTimeUs> {
        self.block
            .as_ref()
            .map(|block| block.data.header.theta)
            .or_else(|| self.leaf.as_ref().map(|leaf| leaf.data.theta))
    }

    fn missing(&self) -> MissingData {
        MissingData {
            block: self.block.is_none(),
            leaf: self.leaf.is_none(),
            transaction_indices: self
                .block
                .as_ref()
                .into_iter()
                .flat_map(|block| 0..block.data.count)
                .filter(|i| !self.txs.contains_key(i))
                .collect(),
        }
    }

    fn complete(&self) -> bool {
        self.block
            .as_ref()
            .is_some_and(|block| block.data.count as usize == self.txs.len() && self.leaf.is_some())
    }

    fn same_as(&self, other: &Self) -> bool {
        self.block.as_ref().map(|block| &block.raw) == other.block.as_ref().map(|block| &block.raw)
            && self.leaf.as_ref().map(|leaf| &leaf.raw) == other.leaf.as_ref().map(|leaf| &leaf.raw)
            && self
                .txs
                .iter()
                .map(|(i, tx)| (i, &tx.raw))
                .eq(other.txs.iter().map(|(i, tx)| (i, &tx.raw)))
    }
}

/// A contiguous, complete prefix whose trust basis is declared in its binding
/// and every block. This type represents API observations.
#[derive(Debug)]
pub struct NativePrefix<'a> {
    binding: &'a SourceBinding,
    blocks: &'a [NativeBlock],
}

impl<'a> NativePrefix<'a> {
    pub fn binding(&self) -> &'a SourceBinding {
        self.binding
    }

    pub fn blocks(&self) -> &'a [NativeBlock] {
        self.blocks
    }
}

pub struct OfficialCaptureReader {
    binding: Arc<SourceBinding>,
    reader_id: String,
    last_id: Option<u64>,
    slots: BTreeMap<u64, Slot>,
    prefix: Vec<NativeBlock>,
    payload_bytes: usize,
}

impl OfficialCaptureReader {
    pub fn new(binding: SourceBinding, reader_id: impl Into<String>) -> Result<Self, Error> {
        let reader_id = reader_id.into();
        if !valid_id(&reader_id) {
            return Err(Error::InvalidRecording("invalid reader ID".into()));
        }
        Ok(Self {
            binding: Arc::new(binding),
            reader_id,
            last_id: None,
            slots: BTreeMap::new(),
            prefix: Vec::new(),
            payload_bytes: 0,
        })
    }

    pub fn binding(&self) -> &SourceBinding {
        &self.binding
    }

    pub fn reader_id(&self) -> &str {
        &self.reader_id
    }

    pub fn tip(&self) -> Option<(u64, PublicTimeUs)> {
        self.prefix.last().map(|block| (block.q, block.theta))
    }

    pub fn prefix(&self) -> NativePrefix<'_> {
        NativePrefix {
            binding: &self.binding,
            blocks: &self.prefix,
        }
    }

    pub fn read(&self, q: u64) -> Result<&NativeBlock, Error> {
        usize::try_from(q)
            .ok()
            .and_then(|q| self.prefix.get(q))
            .ok_or_else(|| self.wait())
    }

    pub fn require_prefix_through(&self, q: u64) -> Result<NativePrefix<'_>, Error> {
        self.read(q)?;
        let n = usize::try_from(q).expect("read accepted q") + 1;
        Ok(NativePrefix {
            binding: &self.binding,
            blocks: &self.prefix[..n],
        })
    }

    pub fn require_watermark(&self, theta: PublicTimeUs) -> Result<NativePrefix<'_>, Error> {
        if self
            .tip()
            .is_some_and(|(_, last_theta)| last_theta >= theta)
        {
            Ok(self.prefix())
        } else {
            Err(self.wait())
        }
    }

    pub fn wait(&self) -> Error {
        let next_q = self.prefix.len() as u64;
        Error::Wait {
            next_q,
            missing: self.slots.get(&next_q).map_or_else(
                || MissingData {
                    block: true,
                    leaf: true,
                    transaction_indices: Vec::new(),
                },
                Slot::missing,
            ),
        }
    }

    pub fn receive(&mut self, record: CaptureRecord<'_>) -> Result<ReceiveOutcome, Error> {
        if record.source_id != self.binding.source_id() {
            return Err(invalid("record source ID does not match reader"));
        }
        if record.reader_id != self.reader_id {
            return Err(Error::InvalidRecording(
                "record delivered to wrong reader".into(),
            ));
        }
        if self.last_id.is_some_and(|last| record.id <= last) {
            return Err(Error::InvalidRecording("nonincreasing record ID".into()));
        }
        if record.response.len() > MAX_CAPTURE_FILE_BYTES {
            return Err(Error::Capacity("raw response"));
        }

        let endpoint = endpoint(record.method, record.path)?;
        let outcome = match record.status {
            None => ReceiveOutcome::Unavailable,
            Some(200) => self.receive_success(endpoint, record)?,
            Some(status @ 400..=599) if endpoint == Endpoint::Submit => {
                ReceiveOutcome::SubmitRejected { status }
            }
            Some(404 | 408 | 429 | 500..=599) => ReceiveOutcome::Unavailable,
            Some(status) => {
                return Err(Error::InvalidRecording(format!(
                    "unexpected HTTP status {status} at {}",
                    record.path
                )));
            }
        };
        self.last_id = Some(record.id);
        self.advance();
        Ok(outcome)
    }

    fn receive_success(
        &mut self,
        endpoint: Endpoint,
        record: CaptureRecord<'_>,
    ) -> Result<ReceiveOutcome, Error> {
        let raw = json(record.response)?;
        let reference = CaptureRecordRef {
            reader_id: self.reader_id.clone(),
            record_id: record.id,
            path: record.path.into(),
        };
        match endpoint {
            Endpoint::Block(q) => self.receive_block(q, raw, reference),
            Endpoint::Leaf(q) => self.receive_leaf(q, raw, reference),
            Endpoint::Transaction(q, i) => self.receive_tx(q, i, raw, reference),
            Endpoint::Submit => {
                let transaction_hash = raw
                    .as_str()
                    .filter(|hash| tagged(hash, "TX~"))
                    .ok_or_else(|| invalid("invalid submit ACK transaction hash"))?
                    .to_owned();
                Ok(ReceiveOutcome::Submitted { transaction_hash })
            }
            Endpoint::Height => Ok(ReceiveOutcome::HeightHint(
                raw.as_u64()
                    .ok_or_else(|| invalid("invalid block-height hint"))?,
            )),
            Endpoint::TransactionByHash | Endpoint::DevInfo | Endpoint::L1 => {
                Ok(ReceiveOutcome::Diagnostic)
            }
        }
    }

    fn require_slot(&self, q: u64) -> Result<Option<&Slot>, Error> {
        let slot = self.slots.get(&q);
        if slot.is_none() && self.slots.len() == MAX_BLOCKS {
            return Err(Error::Capacity("distinct block heights including pending"));
        }
        Ok(slot)
    }

    fn receive_block(
        &mut self,
        q: u64,
        raw: Value,
        reference: CaptureRecordRef,
    ) -> Result<ReceiveOutcome, Error> {
        let slot = self.require_slot(q)?;
        if let Some(prior) = slot.and_then(|slot| slot.block.as_ref()) {
            return repeat(&prior.raw, &raw);
        }
        let raw_header = object(&raw, "header")?;
        let header = Header::parse(raw_header, q)?;
        let hash = tagged_string(&raw, "hash", "BLOCK~")?.to_owned();
        if q == 0 && hash != self.binding.identity().genesis_block_hash {
            return Err(invalid("genesis block hash does not match source binding"));
        }
        let payload = object(&raw, "payload")?;
        let payload_bytes = STANDARD
            .decode(string(payload, "raw_payload")?)
            .map_err(|_| invalid("invalid raw block payload base64"))?;
        let ns_table = object(payload, "ns_table")?;
        let table_bytes = string(ns_table, "bytes")?;
        STANDARD
            .decode(table_bytes)
            .map_err(|_| invalid("invalid namespace table base64"))?;
        let header_table = object(&raw_header["fields"], "ns_table")?;
        if string(header_table, "bytes")? != table_bytes
            || number(&raw, "size")? != payload_bytes.len() as u64
        {
            return Err(invalid(
                "block payload size or namespace-table metadata conflict",
            ));
        }
        let count = number(&raw, "num_transactions")?;
        if count > MAX_MESSAGES as u64 {
            return Err(Error::Capacity("transactions in one block"));
        }
        let count = u32::try_from(count).expect("bounded transaction count");
        self.check_order(q, header.theta)?;
        if let Some(slot) = slot {
            if slot
                .leaf
                .as_ref()
                .is_some_and(|leaf| &leaf.raw["leaf"]["block_header"] != raw_header)
            {
                return Err(invalid("block/leaf header conflict"));
            }
            if slot
                .txs
                .iter()
                .any(|(&i, tx)| i >= count || tx.data.block_hash != hash)
            {
                return Err(invalid(
                    "pending transaction conflicts with block count or hash",
                ));
            }
        }
        self.slots.entry(q).or_default().block = Some(Observed {
            raw,
            data: Block {
                header,
                hash,
                count,
            },
            reference,
        });
        Ok(ReceiveOutcome::Stored)
    }

    fn receive_leaf(
        &mut self,
        q: u64,
        raw: Value,
        reference: CaptureRecordRef,
    ) -> Result<ReceiveOutcome, Error> {
        let slot = self.require_slot(q)?;
        if let Some(prior) = slot.and_then(|slot| slot.leaf.as_ref()) {
            return repeat(&prior.raw, &raw);
        }
        let leaf = object(&raw, "leaf")?;
        let raw_header = object(leaf, "block_header")?;
        let header = Header::parse(raw_header, q)?;
        let qc = object(&raw, "qc")?;
        let data = object(qc, "data")?;
        tagged_string(data, "leaf_commit", "COMMIT~")?;
        number(qc, "view_number")?;
        if qc.get("signatures").is_none() {
            return Err(invalid("missing QC signatures field"));
        }
        self.check_order(q, header.theta)?;
        if slot
            .and_then(|slot| slot.block.as_ref())
            .is_some_and(|block| &block.raw["header"] != raw_header)
        {
            return Err(invalid("block/leaf header conflict"));
        }
        self.slots.entry(q).or_default().leaf = Some(Observed {
            raw,
            data: header,
            reference,
        });
        Ok(ReceiveOutcome::Stored)
    }

    fn receive_tx(
        &mut self,
        q: u64,
        i: u64,
        raw: Value,
        reference: CaptureRecordRef,
    ) -> Result<ReceiveOutcome, Error> {
        let slot = self.require_slot(q)?;
        if i >= MAX_MESSAGES as u64 {
            return Err(invalid("transaction index outside supported block range"));
        }
        let i = u32::try_from(i).expect("bounded transaction index");
        if let Some(prior) = slot.and_then(|slot| slot.txs.get(&i)) {
            return repeat(&prior.raw, &raw);
        }
        let tx = transaction(&raw, q, i)?;
        if q == 0 && tx.block_hash != self.binding.identity().genesis_block_hash {
            return Err(invalid("genesis transaction disagrees with source binding"));
        }
        if let Some(slot) = slot {
            if slot
                .block
                .as_ref()
                .is_some_and(|block| i >= block.data.count || tx.block_hash != block.data.hash)
            {
                return Err(invalid("transaction conflicts with block count or hash"));
            }
            if slot
                .txs
                .first_key_value()
                .is_some_and(|(_, prior)| prior.data.block_hash != tx.block_hash)
            {
                return Err(invalid("transaction conflicts with pending block hash"));
            }
        }

        let payload_bytes = self
            .payload_bytes
            .checked_add(tx.payload.len())
            .filter(|&bytes| bytes <= MAX_LOG_BYTES)
            .ok_or(Error::Capacity("retained native transaction payload bytes"))?;
        if tx.namespace == PUBLICATION_NAMESPACE {
            let selected: Vec<_> = slot
                .into_iter()
                .flat_map(|slot| slot.txs.values().map(|tx| &tx.data))
                .chain(std::iter::once(&tx))
                .filter(|tx| tx.namespace == PUBLICATION_NAMESPACE)
                .map(|tx| LogMessage {
                    codec: FIXTURE_CODEC,
                    bytes: tx.payload.clone(),
                })
                .collect();
            check_selected(self.binding.source_id(), &selected)?;
        }

        self.slots.entry(q).or_default().txs.insert(
            i,
            Observed {
                raw,
                data: tx,
                reference,
            },
        );
        self.payload_bytes = payload_bytes;
        Ok(ReceiveOutcome::Stored)
    }

    fn check_order(&self, q: u64, theta: PublicTimeUs) -> Result<(), Error> {
        let before = self
            .slots
            .range(..q)
            .rev()
            .find_map(|(_, slot)| slot.theta());
        let after = self
            .slots
            .range((Excluded(q), Unbounded))
            .find_map(|(_, slot)| slot.theta());
        if before.is_some_and(|before| before > theta) || after.is_some_and(|after| after < theta) {
            return Err(invalid("public timestamp regresses in height order"));
        }
        Ok(())
    }

    fn advance(&mut self) {
        let mut q = self.prefix.len() as u64;
        while let Some(slot) = self.slots.get(&q).filter(|slot| slot.complete()) {
            let block = slot.block.as_ref().expect("complete block");
            let leaf = slot.leaf.as_ref().expect("complete leaf");
            let finality_basis = match self.binding.capture_kind() {
                CaptureKind::Official => FinalityBasis::TrustedLocalOfficialApi {
                    binding: Arc::clone(&self.binding),
                    leaf_record_ref: leaf.reference.clone(),
                    block_record_ref: block.reference.clone(),
                },
                CaptureKind::Fixture => FinalityBasis::Fixture {
                    binding: Arc::clone(&self.binding),
                    leaf_record_ref: leaf.reference.clone(),
                    block_record_ref: block.reference.clone(),
                },
            };
            self.prefix.push(NativeBlock {
                q,
                theta: block.data.header.theta,
                timestamp_seconds: block.data.header.seconds,
                timestamp_millis: block.data.header.millis,
                block_hash: block.data.hash.clone(),
                ordered_transactions: slot.txs.values().map(|tx| tx.data.clone()).collect(),
                complete_count: block.data.count,
                finality_basis,
                timing: UnknownTiming::default(),
                transaction_refs: slot.txs.values().map(|tx| tx.reference.clone()).collect(),
            });
            q += 1;
        }
    }

    pub fn project_publication_messages(
        &self,
        q: u64,
        codec: u32,
    ) -> Result<Vec<ProjectedMessage>, Error> {
        let block = self.read(q)?;
        if codec == 0 {
            return Err(Error::UnsupportedRepresentation(
                "publication codec zero".into(),
            ));
        }
        Ok(block
            .ordered_transactions
            .iter()
            .filter(|tx| tx.namespace == PUBLICATION_NAMESPACE)
            .enumerate()
            .map(|(index, tx)| ProjectedMessage {
                position: EspressoPosition {
                    q,
                    message_index: u32::try_from(index).expect("bounded transaction count"),
                },
                source_position: tx.position,
                message: LogMessage {
                    codec,
                    bytes: tx.payload.clone(),
                },
            })
            .collect())
    }

    /// Compare complete block, leaf and transaction JSON values, independent of
    /// per-connection receipts and JSON presentation. Never borrows evidence
    /// from the other reader to fill pending entries.
    pub fn compare_public_objects(&self, other: &Self) -> Result<(), Error> {
        if self.binding.identity() != other.binding.identity()
            || self.binding.capture_kind() != other.binding.capture_kind()
        {
            return Err(invalid("different reader sources"));
        }
        if self.prefix.len() != other.prefix.len() {
            return Err(invalid("different complete prefix lengths"));
        }
        for block in &self.prefix {
            if !self.slots[&block.q].same_as(&other.slots[&block.q]) {
                return Err(Error::InvalidSource(format!(
                    "requeried public objects conflict at q={}",
                    block.q
                )));
            }
        }
        Ok(())
    }

    pub fn public_audit_bytes(&self, codec: u32) -> Result<Vec<u8>, Error> {
        if codec == 0 {
            return Err(Error::UnsupportedRepresentation(
                "publication codec zero".into(),
            ));
        }
        let mut out = b"EC16-PUBLIC-1\0".to_vec();
        lp(&mut out, self.binding.source_id().as_bytes())?;
        count(&mut out, self.prefix.len())?;
        for block in &self.prefix {
            out.extend(block.q.to_le_bytes());
            out.extend(block.theta.0.to_le_bytes());
            lp(&mut out, block.block_hash.as_bytes())?;
            count(&mut out, block.ordered_transactions.len())?;
            for tx in &block.ordered_transactions {
                out.extend(tx.position.source_transaction_index.to_le_bytes());
                out.extend(tx.namespace.to_le_bytes());
                out.extend(tx.pos_in_namespace.to_le_bytes());
                lp(&mut out, tx.transaction_hash.as_bytes())?;
                lp(&mut out, &tx.payload)?;
            }
            let selected = self.project_publication_messages(block.q, codec)?;
            count(&mut out, selected.len())?;
            for message in selected {
                out.extend(message.position.message_index.to_le_bytes());
                out.extend(
                    message
                        .source_position
                        .source_transaction_index
                        .to_le_bytes(),
                );
                out.extend(message.message.codec.to_le_bytes());
                lp(&mut out, &message.message.bytes)?;
            }
        }
        Ok(out)
    }

    pub fn public_sha256(&self, codec: u32) -> Result<String, Error> {
        self.public_audit_bytes(codec)
            .map(|bytes| sha256_hex(&bytes))
    }

    /// Match ACK hash, namespace and exact payload only in the visible prefix.
    pub fn find_publication(&self, transaction_hash: &str, payload: &[u8]) -> Vec<NativePosition> {
        self.prefix
            .iter()
            .flat_map(|block| &block.ordered_transactions)
            .filter(|tx| {
                tx.namespace == PUBLICATION_NAMESPACE
                    && tx.transaction_hash == transaction_hash
                    && tx.payload == payload
            })
            .map(|tx| tx.position)
            .collect()
    }

    pub fn export_pe_log(&self) -> Result<Vec<u8>, UnsupportedTimingEvidence> {
        export_pe_log()
    }

    pub fn export_pe_reader(&self) -> Result<sim_espresso::ReaderState, UnsupportedTimingEvidence> {
        export_pe_reader()
    }
}

fn repeat(prior: &Value, new: &Value) -> Result<ReceiveOutcome, Error> {
    if prior == new {
        Ok(ReceiveOutcome::Duplicate)
    } else {
        Err(invalid("conflicting repeated response"))
    }
}

pub(crate) fn decode_payload(encoded: &str) -> Result<Vec<u8>, Error> {
    if encoded.len() > MAX_MESSAGE_BYTES.div_ceil(3) * 4 {
        return Err(Error::Capacity("decoded transaction payload bytes"));
    }
    let payload = STANDARD
        .decode(encoded)
        .map_err(|_| invalid("invalid transaction payload base64"))?;
    if payload.len() > MAX_MESSAGE_BYTES {
        return Err(Error::Capacity("decoded transaction payload bytes"));
    }
    Ok(payload)
}

fn transaction(raw: &Value, q: u64, i: u32) -> Result<NativeTransaction, Error> {
    let tx = object(raw, "transaction")?;
    let namespace = number(raw, "namespace")?;
    if number(raw, "index")? != u64::from(i)
        || number(raw, "block_height")? != q
        || number(tx, "namespace")? != namespace
        || namespace > u64::from(u32::MAX)
    {
        return Err(invalid(
            "transaction position or namespace binding conflict",
        ));
    }
    Ok(NativeTransaction {
        position: NativePosition {
            q,
            source_transaction_index: i,
        },
        namespace,
        payload: decode_payload(string(tx, "payload")?)?,
        transaction_hash: tagged_string(raw, "hash", "TX~")?.to_owned(),
        block_hash: tagged_string(raw, "block_hash", "BLOCK~")?.to_owned(),
        pos_in_namespace: u32::try_from(number(raw, "pos_in_namespace")?)
            .map_err(|_| invalid("namespace position overflow"))?,
    })
}

fn check_selected(source_id: &str, selected: &[LogMessage]) -> Result<(), Error> {
    validate_message_list(source_id.as_bytes(), selected).map_err(|err| match err {
        WireError::BlockTooLarge => Error::Capacity("selected PELG envelope bytes"),
        _ => Error::InvalidSource(format!("invalid projected publication list: {err:?}")),
    })
}

fn count(out: &mut Vec<u8>, num: usize) -> Result<(), Error> {
    out.extend(
        u32::try_from(num)
            .map_err(|_| Error::Capacity("audit field length"))?
            .to_le_bytes(),
    );
    Ok(())
}

fn lp(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), Error> {
    count(out, bytes.len())?;
    out.extend(bytes);
    Ok(())
}
