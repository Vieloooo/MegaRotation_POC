//! Independent fixture replay and durable Certifier signing.

use crate::{
    CrashHook, FaultPoint, ServiceConfig,
    accepted_view::{CurrentCursor, ReadError, VerifiedDaObject, parse_batch},
    crypto::{BatchWire, CursorWire, SigningKey, check_da, hex, validate_submission, verify_batch},
    frame::{commitment, verify_payload},
    source::{BatchSource, replay},
    storage::{CommandBook, Journal, Root, json_bytes, no_crash, reject, wait},
};
use poc_types::domain::Hash32;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Debug, Serialize)]
pub struct CertificationRequest {
    pub request_id: Hash32,
    pub prev: CursorWire,
    pub payload: Vec<u8>,
    pub commitment: Hash32,
    pub da_certificate: Vec<u8>,
    pub gpos: Vec<u8>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct State {
    commands: CommandBook,
    observed: u64,
    signatures: BTreeMap<String, Signed>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Signed {
    input: Value,
    source: Hash32,
    cursor: Value,
    result: Value,
}

pub struct Certifier {
    journal: Journal<State>,
    reader: String,
    config: ServiceConfig,
    key: SigningKey,
    crash: CrashHook,
    available: bool,
}

impl Certifier {
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
        if key.address() != config.signers.certifier {
            return Err(reject("Certifier key does not match configured signer"));
        }
        if reader.is_empty() {
            return Err(reject("Certifier needs an independent reader"));
        }
        let journal = Journal::open(Root::open(root, &format!("certifier:{reader}"), &config)?)?;
        Ok(Self {
            journal,
            reader: reader.into(),
            config,
            key,
            crash,
            available: true,
        })
    }

    pub fn is_available(&self) -> bool {
        self.available
    }

    pub fn set_available(&mut self, available: bool) {
        self.available = available;
    }

    pub fn certify(
        &mut self,
        id: u64,
        request: &CertificationRequest,
        cursor: &CurrentCursor,
        data: Option<&VerifiedDaObject>,
        source: &dyn BatchSource,
    ) -> Result<BatchWire, ReadError> {
        if !self.available {
            return Err(wait("Certifier unavailable"));
        }
        if request.payload.len() > crate::frame::MAX_PAYLOAD
            || request.gpos.len() > 1024
            || request.da_certificate.len() != 65
        {
            return Err(reject("certification request width"));
        }
        let input = json!(request);
        let mut state = self.journal.state.clone();
        state.commands.register(id, input.clone())?;
        check_current(cursor, &self.config, &self.reader, state.observed)?;
        state.observed = cursor.as_of();
        self.journal.commit(state)?;
        if &request.prev != cursor.cursor() {
            return Err(reject("StalePrev"));
        }
        let frame = verify_payload(request.commitment, &request.payload)
            .map_err(|e| reject(&format!("certification frame: {e}")))?;
        let first = frame
            .records
            .first()
            .expect("closed frame nonempty")
            .block_number;
        let terminal = frame.records.last().expect("closed frame nonempty");
        let last = terminal.block_number;
        if request.prev.height.checked_add(1) != Some(first) {
            return Err(reject("certification is not the next cursor range"));
        }
        let replayed = replay(source, cursor, &request.gpos, first, last, &self.config)?;
        if replayed.records != frame.records {
            return Err(reject(
                "DA payload contradicts independently executed source",
            ));
        }
        check_da(
            request.commitment,
            &request.da_certificate,
            &self.config.inbox,
            self.config.signers.da,
        )?;
        let data = data.ok_or_else(|| wait("Certifier has not retrieved required DA object"))?;
        if data.commitment() != request.commitment
            || data.receipt().as_slice() != request.da_certificate
            || data.payload() != request.payload
        {
            return Err(reject("DA data differs from certification request"));
        }
        // Always recheck current cursor, recovery, source execution and DA before cache use.
        let cache_key = hex(commitment(&json_bytes(&input)?).as_bytes());
        let gpos = hex(&request.gpos);
        let mut result = BatchWire {
            request_id: hex(request.request_id.as_bytes()),
            prev: request.prev.clone(),
            next: CursorWire {
                height: last,
                origin: terminal.epoch_num,
                frames_digest: hex(&[0; 32]),
                source_kind: 1,
                source: gpos.clone(),
            },
            commitment: hex(request.commitment.as_bytes()),
            da_certificate: hex(&request.da_certificate),
            first_height: first,
            last_height: last,
            gpos,
            certifier_signature: String::new(),
        };
        validate_submission(&result)?;
        if let Some(signed) = self.journal.state.signatures.get(&cache_key) {
            if signed.input != input || signed.source != replayed.fingerprint {
                return Err(reject("certification cache identity conflict"));
            }
            let batch = parse_batch(&signed.result)?;
            result.certifier_signature = batch.certifier_signature.clone();
            if batch != result {
                return Err(reject(
                    "cached attestation contradicts the revalidated request",
                ));
            }
            verify_batch(&batch, &self.config.inbox, self.config.signers.certifier)?;
            return Ok(batch);
        }
        result.certifier_signature = hex(&self
            .key
            .sign_digest(self.config.inbox.batch_digest(&result)?)?);
        (self.crash)(FaultPoint::CertifierBeforeSignaturePersist);
        let mut state = self.journal.state.clone();
        state.signatures.insert(
            cache_key,
            Signed {
                input,
                source: replayed.fingerprint,
                cursor: json!(cursor),
                result: json!(result),
            },
        );
        self.journal.commit(state)?;
        (self.crash)(FaultPoint::CertifierSignatureDurable);
        Ok(result)
    }
}

pub(crate) fn check_current(
    cursor: &CurrentCursor,
    config: &ServiceConfig,
    reader: &str,
    observed: u64,
) -> Result<(), ReadError> {
    if cursor.domain() != config.inbox || cursor.reader() != reader {
        return Err(reject(
            "cursor is not from the role's configured reader and Inbox",
        ));
    }
    if cursor.as_of() < observed {
        return Err(reject("reader cursor prefix regressed"));
    }
    Ok(())
}
