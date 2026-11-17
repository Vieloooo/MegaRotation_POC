//! Durable batch construction, signing outbox, and receipt reconciliation.

use crate::{
    CrashHook, FaultPoint, ServiceConfig,
    accepted_view::{
        CurrentCursor, FinalizedBlock, ReadError, authenticate, parse_batch, parse_cursor,
    },
    certifier::{CertificationRequest, check_current},
    crypto::{
        BatchWire, CursorWire, SigningKey, check_da, cursor_hash, hex, submit_batch_calldata,
        uint_word,
    },
    frame::{commitment, decode_closed, encode_closed},
    source::{AppliedBlock, BatchSource, replay},
    storage::{CommandBook, Journal, Root, no_crash, reject, wait},
    transaction::{LegacyTerms, PreparedTransaction, sign_legacy},
};
use poc_types::domain::Hash32;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobPhase {
    Built,
    DaReady,
    Certified,
    SubmittedUnknown,
    Accepted,
    Reverted,
    StalePrev,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum JobTerminal {
    Accepted { evidence: Value },
    Reverted { receipt: Value },
    StalePrev,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchJob {
    id: Hash32,
    prev: Value,
    payload: Vec<u8>,
    gpos: Vec<u8>,
    source: Hash32,
    commitment: Hash32,
    audit: Vec<AppliedBlock>,
    da_certificate: Option<Vec<u8>>,
    certified: Option<Value>,
    raw: Option<PreparedTransaction>,
    terminal: Option<JobTerminal>,
}

impl BatchJob {
    pub fn id(&self) -> Hash32 {
        self.id
    }
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
    pub fn execution_audit(&self) -> &[AppliedBlock] {
        &self.audit
    }
    pub fn commitment(&self) -> Hash32 {
        self.commitment
    }
    pub fn transaction(&self) -> Option<&PreparedTransaction> {
        self.raw.as_ref()
    }
    pub fn terminal(&self) -> Option<&JobTerminal> {
        self.terminal.as_ref()
    }
    pub fn phase(&self) -> JobPhase {
        match self.terminal {
            Some(JobTerminal::Accepted { .. }) => JobPhase::Accepted,
            Some(JobTerminal::Reverted { .. }) => JobPhase::Reverted,
            Some(JobTerminal::StalePrev) => JobPhase::StalePrev,
            None if self.raw.is_some() => JobPhase::SubmittedUnknown,
            None if self.certified.is_some() => JobPhase::Certified,
            None if self.da_certificate.is_some() => JobPhase::DaReady,
            None => JobPhase::Built,
        }
    }
    pub fn certification_request(&self) -> Result<CertificationRequest, ReadError> {
        let da_certificate = self
            .da_certificate
            .clone()
            .ok_or_else(|| wait("batch DA receipt unavailable"))?;
        Ok(CertificationRequest {
            request_id: self.id,
            prev: parse_cursor(&self.prev)?,
            payload: self.payload.clone(),
            commitment: self.commitment,
            da_certificate,
            gpos: self.gpos.clone(),
        })
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct State {
    commands: CommandBook,
    observed: u64,
    jobs: BTreeMap<String, BatchJob>,
}

pub struct Batcher {
    journal: Journal<State>,
    reader: String,
    config: ServiceConfig,
    key: SigningKey,
    available: bool,
    crash: CrashHook,
}

impl Batcher {
    pub fn open(
        root: &Path,
        config: ServiceConfig,
        reader: &str,
        key: SigningKey,
    ) -> Result<Self, ReadError> {
        Self::open_with_hook(root, config, reader, key, no_crash)
    }

    pub fn open_with_hook(
        root: &Path,
        config: ServiceConfig,
        reader: &str,
        key: SigningKey,
        crash: CrashHook,
    ) -> Result<Self, ReadError> {
        if key.address() != config.submitter || reader.is_empty() {
            return Err(reject("invalid Batcher submitter or reader"));
        }
        let journal = Journal::open(Root::open(root, &format!("batcher:{reader}"), &config)?)?;
        Ok(Self {
            journal,
            reader: reader.into(),
            config,
            key,
            available: true,
            crash,
        })
    }

    pub fn is_available(&self) -> bool {
        self.available
    }

    pub fn set_available(&mut self, available: bool) {
        self.available = available;
    }
    pub fn job(&self, id: Hash32) -> Option<&BatchJob> {
        self.journal.state.jobs.get(&id_key(id))
    }

    pub fn build(
        &mut self,
        id: u64,
        cursor: &CurrentCursor,
        prev: &CursorWire,
        gpos: &[u8],
        first: u64,
        last: u64,
        source: &dyn BatchSource,
    ) -> Result<BatchJob, ReadError> {
        let mut state = self.begin(id, json!(["build", prev, gpos, first, last]))?;
        check_current(cursor, &self.config, &self.reader, state.observed)?;
        state.observed = cursor.as_of();
        self.journal.commit(state.clone())?;
        if cursor.cursor() != prev {
            return Err(reject("StalePrev"));
        }
        if prev.height.checked_add(1) != Some(first) {
            return Err(reject("build is not the next cursor range"));
        }
        let replayed = replay(source, cursor, gpos, first, last, &self.config)?;
        let payload =
            encode_closed(&replayed.records).map_err(|e| reject(&format!("build frame: {e}")))?;
        let d = commitment(&payload);
        let mut preimage = b"PoC/BatchJob/v1".to_vec();
        preimage.extend(uint_word(self.config.inbox.chain_id));
        preimage.extend(self.config.inbox.inbox);
        preimage.extend(self.config.inbox.config_ref.as_bytes());
        preimage.extend(cursor_hash(prev)?.as_bytes());
        preimage.extend(d.as_bytes());
        let job_id = commitment(&preimage);
        let prev = json!(prev);
        let key = id_key(job_id);
        if let Some(job) = state.jobs.get(&key) {
            if job.prev != prev
                || job.payload != payload
                || job.gpos != gpos
                || job.source != replayed.fingerprint
            {
                return Err(reject("batch task identity conflict"));
            }
            return Ok(job.clone());
        }
        let job = BatchJob {
            id: job_id,
            prev,
            payload,
            gpos: gpos.into(),
            source: replayed.fingerprint,
            commitment: d,
            audit: replayed.audit,
            da_certificate: None,
            certified: None,
            raw: None,
            terminal: None,
        };
        state.jobs.insert(key, job.clone());
        self.journal.commit(state)?;
        (self.crash)(FaultPoint::BatcherBuiltDurable);
        Ok(job)
    }

    pub fn record_da(
        &mut self,
        id: u64,
        job_id: Hash32,
        d: Hash32,
        receipt: &[u8],
    ) -> Result<BatchJob, ReadError> {
        if receipt.len() != 65 {
            return Err(reject("invalid DA receipt width"));
        }
        let mut state = self.begin(id, json!(["da", job_id, d, receipt]))?;
        let job = find(&mut state, job_id)?;
        if job.commitment != d {
            return Err(reject("DA commitment does not match job"));
        }
        check_da(d, receipt, &self.config.inbox, self.config.signers.da)?;
        if let Some(old) = &job.da_certificate {
            if old != receipt {
                return Err(reject("DA receipt conflicts with durable result"));
            }
        } else {
            ensure_pending(job)?;
            job.da_certificate = Some(receipt.to_vec());
        }
        let output = job.clone();
        self.journal.commit(state)?;
        (self.crash)(FaultPoint::BatcherDaDurable);
        Ok(output)
    }

    pub fn record_certified(
        &mut self,
        id: u64,
        job_id: Hash32,
        batch: &BatchWire,
    ) -> Result<BatchJob, ReadError> {
        let mut state = self.begin(id, json!(["certified", job_id, batch]))?;
        let job = find(&mut state, job_id)?;
        let request = job.certification_request()?;
        let frame = decode_closed(&job.payload).map_err(|e| reject(&e.to_string()))?;
        let first = frame.records.first().expect("frame is nonempty");
        let last = frame.records.last().expect("frame is nonempty");
        if batch.prev != request.prev
            || batch.request_id != id_key(job_id)
            || batch.commitment != id_key(job.commitment)
            || batch.da_certificate != hex(&request.da_certificate)
            || batch.gpos != hex(&job.gpos)
            || batch.first_height != first.block_number
            || batch.last_height != last.block_number
            || batch.next.origin != last.epoch_num
        {
            return Err(reject("certified result differs from durable job"));
        }
        authenticate(batch, &self.config.inbox, self.config.signers)?;
        let signed = json!(batch);
        if let Some(old) = &job.certified {
            if old != &signed {
                return Err(reject("certified result conflicts with durable result"));
            }
        } else {
            ensure_pending(job)?;
            job.certified = Some(signed);
        }
        let output = job.clone();
        self.journal.commit(state)?;
        (self.crash)(FaultPoint::BatcherCertifiedDurable);
        Ok(output)
    }

    /// Durably stores the only permissible outbox transaction for this task before returning it.
    pub fn prepare_transaction(
        &mut self,
        id: u64,
        job_id: Hash32,
        terms: LegacyTerms,
    ) -> Result<PreparedTransaction, ReadError> {
        let mut state = self.begin(id, json!(["prepare_transaction", job_id, terms]))?;
        if state.jobs.values().any(|job| {
            job.id != job_id && job.raw.as_ref().is_some_and(|tx| tx.nonce() == terms.nonce)
        }) {
            return Err(reject("submitter nonce already bound to another job"));
        }
        let job = find(&mut state, job_id)?;
        ensure_pending(job)?;
        let certified = parse_batch(
            job.certified
                .as_ref()
                .ok_or_else(|| wait("batch attestation unavailable"))?,
        )?;
        let tx = sign_legacy(
            &self.key,
            self.config.inbox.chain_id,
            Some(self.config.inbox.inbox),
            terms,
            &submit_batch_calldata(&certified)?,
        )?;
        if let Some(old) = &job.raw {
            if old != &tx {
                return Err(reject("pending raw transaction cannot be replaced"));
            }
        } else {
            job.raw = Some(tx.clone());
        }
        self.journal.commit(state)?;
        (self.crash)(FaultPoint::BatcherRawDurable);
        Ok(tx)
    }

    pub fn pending_transaction(&self, job_id: Hash32) -> Result<&PreparedTransaction, ReadError> {
        if !self.available {
            return Err(wait("Batcher unavailable"));
        }
        let job = self
            .job(job_id)
            .ok_or_else(|| reject("unknown batch job"))?;
        ensure_pending(job)?;
        let tx = job
            .transaction()
            .ok_or_else(|| wait("batch outbox is not yet prepared"))?;
        if commitment(tx.raw()) != tx.hash() {
            return Err(reject("stored raw transaction is corrupt"));
        }
        Ok(tx)
    }

    /// Reconcile the original Advanced or a definite EVM failure in a verified complete prefix.
    pub fn reconcile(
        &mut self,
        id: u64,
        job_id: Hash32,
        blocks: &[FinalizedBlock],
    ) -> Result<JobTerminal, ReadError> {
        let mut state = self.begin(
            id,
            json!([
                "reconcile",
                job_id,
                blocks
                    .iter()
                    .map(|block| &block.raw_block()["hash"])
                    .collect::<Vec<_>>()
            ]),
        )?;
        let job = find(&mut state, job_id)?;
        if let Some(terminal) = job.terminal.clone() {
            self.journal.commit(state)?;
            return Ok(terminal);
        }
        let tx = job
            .raw
            .as_ref()
            .ok_or_else(|| wait("outbox not prepared"))?;
        let tx_hash = id_key(tx.hash());
        let signed = parse_batch(
            job.certified
                .as_ref()
                .ok_or_else(|| reject("pending job missing certification"))?,
        )?;
        let mut actual_receipt = None;
        let mut original = None;
        for (height, block) in blocks.iter().enumerate() {
            if block.domain() != self.config.inbox
                || block.reader() != self.reader
                || block.height() != height as u64
            {
                return Err(reject(
                    "reconciliation requires the configured reader's complete ordered prefix",
                ));
            }
            if height > 0
                && block.raw_block()["parentHash"] != blocks[height - 1].raw_block()["hash"]
            {
                return Err(reject("reconciliation blocks do not form one chain"));
            }
            for record in block
                .accepted()
                .iter()
                .filter(|record| record.request_id() == job_id)
            {
                if record.submission() != &signed || original.is_some() {
                    return Err(reject("accepted request identity conflicts"));
                }
                original = Some(record);
            }
            for receipt in block.raw_receipts() {
                if receipt["transactionHash"] == tx_hash {
                    actual_receipt = Some(receipt);
                }
            }
        }
        let terminal = if let Some(record) = original {
            JobTerminal::Accepted {
                evidence: json!(record),
            }
        } else if let Some(receipt) = actual_receipt.filter(|receipt| receipt["status"] == "0x0") {
            JobTerminal::Reverted {
                receipt: receipt.clone(),
            }
        } else {
            self.journal.commit(state)?;
            return Err(wait(
                "SubmittedUnknown: original acceptance or definite finalized failure unavailable",
            ));
        };
        job.terminal = Some(terminal.clone());
        self.journal.commit(state)?;
        (self.crash)(FaultPoint::BatcherAcceptedDurable);
        Ok(terminal)
    }

    /// Retire only jobs whose raw transaction has never been exposed.
    pub fn retire_stale(
        &mut self,
        id: u64,
        job_id: Hash32,
        current: &CurrentCursor,
    ) -> Result<BatchJob, ReadError> {
        let mut state = self.begin(id, json!(["retire_stale", job_id, current]))?;
        check_current(current, &self.config, &self.reader, state.observed)?;
        state.observed = current.as_of();
        let job = find(&mut state, job_id)?;
        if job.raw.is_some() {
            return Err(wait("reconcile issued outbox before retiring"));
        }
        if parse_cursor(&job.prev)? == *current.cursor() {
            return Err(reject("job is still current"));
        }
        if !matches!(job.terminal, Some(JobTerminal::StalePrev)) {
            ensure_pending(job)?;
            job.terminal = Some(JobTerminal::StalePrev);
        }
        let output = job.clone();
        self.journal.commit(state)?;
        Ok(output)
    }

    fn begin(&mut self, id: u64, input: Value) -> Result<State, ReadError> {
        if !self.available {
            return Err(wait("Batcher unavailable"));
        }
        let mut next = self.journal.state.clone();
        next.commands.register(id, input)?;
        self.journal.commit(next.clone())?;
        Ok(next)
    }
}

fn id_key(hash: Hash32) -> String {
    hex(hash.as_bytes())
}

fn find(state: &mut State, id: Hash32) -> Result<&mut BatchJob, ReadError> {
    let job = state
        .jobs
        .get_mut(&id_key(id))
        .ok_or_else(|| reject("unknown batch task"))?;
    if job.id != id || commitment(&job.payload) != job.commitment {
        return Err(reject("durable batch task identity mismatch"));
    }
    Ok(job)
}

fn ensure_pending(job: &BatchJob) -> Result<(), ReadError> {
    if job.terminal.is_some() {
        return Err(reject("batch task is terminal"));
    }
    Ok(())
}
