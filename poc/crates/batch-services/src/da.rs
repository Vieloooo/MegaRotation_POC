//! Immutable local P0 DA custody and signing.

use crate::{
    CrashHook, FaultPoint, ServiceConfig,
    accepted_view::{ReadError, VerifiedDaObject},
    crypto::{Signature, SigningKey, check_da, hex},
    frame::{MAX_PAYLOAD, commitment},
    storage::{CommandBook, Journal, Root, no_crash, reject, wait},
};
use poc_types::domain::Hash32;
use serde_json::json;
use std::path::Path;

pub struct DaStore {
    journal: Journal<CommandBook>,
    config: ServiceConfig,
    key: SigningKey,
    available: bool,
    crash: CrashHook,
}

impl DaStore {
    pub fn open(root: &Path, config: ServiceConfig, key: SigningKey) -> Result<Self, ReadError> {
        Self::open_with_hook(root, config, key, no_crash)
    }

    pub fn open_with_hook(
        root: &Path,
        config: ServiceConfig,
        key: SigningKey,
        crash: CrashHook,
    ) -> Result<Self, ReadError> {
        if key.address() != config.signers.da {
            return Err(reject("DA key does not match configured signer"));
        }
        Ok(Self {
            journal: Journal::open(Root::open(root, "da", &config)?)?,
            config,
            key,
            available: true,
            crash,
        })
    }

    pub fn set_available(&mut self, available: bool) {
        self.available = available;
    }

    pub fn disperse(
        &mut self,
        command_id: u64,
        payload: &[u8],
    ) -> Result<(Hash32, Signature), ReadError> {
        if !self.available {
            return Err(wait("DA unavailable"));
        }
        if payload.len() > MAX_PAYLOAD {
            return Err(reject("DA payload exceeds P0 limit"));
        }
        let d = commitment(payload);
        let name = hex(d.as_bytes());
        let bytes_name = format!("{name}.bin");
        let receipt_name = format!("{name}.receipt");
        let mut next = self.journal.state.clone();
        next.register(command_id, json!(["disperse", hex(payload)]))?;
        self.journal.commit(next)?;
        let root = &self.journal.root;
        let old_receipt = root.read(&receipt_name, 65)?;
        match root.read(&bytes_name, MAX_PAYLOAD as u64)? {
            Some(saved) if saved == payload => root.confirm(&bytes_name)?,
            Some(_) => {
                return Err(reject(
                    "immutable DA object is corrupt or conflicts with input",
                ));
            }
            None if old_receipt.is_some() => {
                return Err(reject("acknowledged DA object is missing"));
            }
            None => {
                (self.crash)(FaultPoint::DaBeforeBytes);
                root.publish(&bytes_name, payload, false)?;
            }
        }
        (self.crash)(FaultPoint::DaBytesDurable);
        let receipt = if let Some(old) = old_receipt {
            check_da(d, &old, &self.config.inbox, self.config.signers.da)?;
            root.confirm(&receipt_name)?;
            old.try_into()
                .map_err(|_| reject("stored receipt length"))?
        } else {
            let receipt = self.key.sign_digest(self.config.inbox.da_digest(d)?)?;
            root.publish(&receipt_name, &receipt, false)?;
            receipt
        };
        (self.crash)(FaultPoint::DaReceiptDurable);
        Ok((d, receipt))
    }

    pub fn retrieve(&mut self, command_id: u64, d: Hash32) -> Result<VerifiedDaObject, ReadError> {
        if !self.available {
            return Err(wait("DA unavailable"));
        }
        let name = hex(d.as_bytes());
        let mut next = self.journal.state.clone();
        next.register(command_id, json!(["retrieve", name]))?;
        self.journal.commit(next)?;
        let read = |name: String, limit| {
            self.journal
                .root
                .read(&name, limit)
                .map_err(|error| wait(&format!("DA integrity unavailable: {error}")))?
                .ok_or_else(|| wait("DA bytes or receipt missing"))
        };
        let payload = read(format!("{name}.bin"), MAX_PAYLOAD as u64)?;
        let signature = read(format!("{name}.receipt"), 65)?;
        if commitment(&payload) != d
            || check_da(d, &signature, &self.config.inbox, self.config.signers.da).is_err()
        {
            return Err(wait(
                "DA integrity unavailable: commitment or receipt mismatch",
            ));
        }
        let receipt = signature
            .try_into()
            .map_err(|_| wait("DA integrity unavailable: receipt width"))?;
        for file in [format!("{name}.bin"), format!("{name}.receipt")] {
            self.journal
                .root
                .confirm(&file)
                .map_err(|error| wait(&format!("DA durability unavailable: {error}")))?;
        }
        Ok(VerifiedDaObject {
            commitment: d,
            payload,
            receipt,
        })
    }
}
