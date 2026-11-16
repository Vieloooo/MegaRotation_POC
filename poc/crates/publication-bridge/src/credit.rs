use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use poc_types::domain::Hash32;
use sim_transport::{EndpointBinding, Id, Message, ServiceEffect, ServiceGrant};

use crate::{
    BridgeError,
    bounded::{Writer, digest},
};

const MAX_FRAMES: usize = 4;
const MAX_BYTES: usize = 262_144;
const MAX_RECIPIENTS: usize = 2;

/// The controller validates real remaining link/global/custody capacity,
/// including faults that duplicate a route. This includes the sender's byte
/// budget with copy overhead. The owner verifies charge at commit, and release
/// only once the actual outcome covers the whole logical send.
pub trait CreditVerifier {
    fn verify_available(
        &self,
        grant: &ServiceGrant,
        messages: &[Message],
    ) -> Result<(), BridgeError>;
    fn verify_charge(&self, plan_hash: Hash32, receipt: &[u8]) -> Result<(), BridgeError>;
    fn verify_release(&self, logical_id: Id, receipt: &[u8]) -> Result<(), BridgeError>;
}

pub struct CreditWindow {
    instance: Arc<()>,
    revision: u64,
    sender: Option<(Id, Id, EndpointBinding)>,
    charge: BTreeMap<Id, usize>,
}

pub struct CreditPlan {
    instance: Arc<()>,
    base_revision: u64,
    sender: (Id, Id, EndpointBinding),
    effects: Vec<ServiceEffect>,
    new_charge: BTreeMap<Id, usize>,
    hash: Hash32,
}

impl Default for CreditWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl CreditWindow {
    pub fn new() -> Self {
        Self {
            instance: Arc::new(()),
            revision: 0,
            sender: None,
            charge: BTreeMap::new(),
        }
    }

    pub fn plan(
        &self,
        grant: &ServiceGrant,
        messages: Vec<Message>,
        owner: &dyn CreditVerifier,
    ) -> Result<CreditPlan, BridgeError> {
        let first = messages.first().ok_or(BridgeError::InvalidResult)?;
        let sender = (first.run, first.config, grant.target);
        if self.sender.is_some_and(|current| current != sender) {
            return Err(BridgeError::SourceMismatch);
        }
        if messages.len() > MAX_FRAMES - self.charge.len() {
            return Err(BridgeError::Capacity);
        }
        let mut charged = self.charge.clone();
        let mut recipients = BTreeSet::new();
        for message in &messages {
            if message.run != first.run
                || message.config != first.config
                || message.source != grant.target.endpoint
                || message.origin.source_incarnation == 0
                || message.origin.source_incarnation > grant.target.incarnation
                || message.origin.step_id == 0
                || message.origin.step_id > grant.step.logical_step_id
                || message.origin.sent_tick_us > grant.step.tick_us
                || message.destination == grant.target.endpoint
            {
                return Err(BridgeError::SourceMismatch);
            }
            if !message.payload.starts_with(b"H3TF\0\x01")
                || !(84..=61_523).contains(&message.payload.len())
            {
                return Err(BridgeError::InvalidEncoding);
            }
            message
                .validate_hashes()
                .map_err(|_| BridgeError::InvalidResult)?;
            if charged
                .insert(message.logical_id, message.payload.len())
                .is_some()
            {
                return Err(BridgeError::InvalidResult);
            }
            recipients.insert(message.destination);
        }
        if recipients.len() > MAX_RECIPIENTS || charged.values().sum::<usize>() > MAX_BYTES {
            return Err(BridgeError::Capacity);
        }
        owner.verify_available(grant, &messages)?;
        let mut identity = Writer::new(1024);
        identity.push(&first.run)?;
        identity.push(&first.config)?;
        identity.push(&self.revision.to_be_bytes())?;
        for value in [
            grant.target.incarnation,
            grant.step.logical_step_id,
            grant.step.tick_us,
            grant.step.event_id,
            grant.step.causal_wave,
        ] {
            identity.push(&value.to_be_bytes())?;
        }
        identity.count(self.charge.len())?;
        for id in self.charge.keys() {
            identity.push(id)?;
        }
        identity.count(messages.len())?;
        for message in &messages {
            identity.push(&message.logical_id)?;
        }
        let hash = digest("H3/CreditPlan/v1", &[&identity.bytes]);
        let effects = messages.into_iter().map(ServiceEffect::Send).collect();
        Ok(CreditPlan {
            instance: Arc::clone(&self.instance),
            base_revision: self.revision,
            sender,
            effects,
            new_charge: charged,
            hash,
        })
    }

    pub fn confirm(
        &mut self,
        plan: CreditPlan,
        receipt: &[u8],
        owner: &dyn CreditVerifier,
    ) -> Result<(), BridgeError> {
        if !Arc::ptr_eq(&self.instance, &plan.instance) || plan.base_revision != self.revision {
            return Err(BridgeError::SourceMismatch);
        }
        let revision = self.revision.checked_add(1).ok_or(BridgeError::Capacity)?;
        owner.verify_charge(plan.hash, receipt)?;
        self.sender = Some(plan.sender);
        self.charge = plan.new_charge;
        self.revision = revision;
        Ok(())
    }

    pub fn release(
        &mut self,
        logical_id: Id,
        receipt: &[u8],
        owner: &dyn CreditVerifier,
    ) -> Result<(), BridgeError> {
        if !self.charge.contains_key(&logical_id) {
            return Err(BridgeError::InvalidResult);
        }
        let revision = self.revision.checked_add(1).ok_or(BridgeError::Capacity)?;
        owner.verify_release(logical_id, receipt)?;
        self.charge.remove(&logical_id);
        self.revision = revision;
        Ok(())
    }

    pub fn in_flight(&self) -> (usize, usize) {
        (self.charge.len(), self.charge.values().sum())
    }
}

impl CreditPlan {
    pub fn effects(&self) -> &[ServiceEffect] {
        &self.effects
    }
    pub fn candidate_hash(&self) -> Hash32 {
        self.hash
    }
}
