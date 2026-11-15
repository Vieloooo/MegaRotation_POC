use crate::{
    ChannelId, Endpoint, EndpointBinding, Envelope, Error, Id, Instance, LocalTime, Message,
    Origin, Tick, TransportConfig, framed_hash,
    identity::{MessageHeader, rebind},
    snapshot::{SCHEMA, decode_snapshot, encode, encode_snapshot, schema, strictly_ordered},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReliableLimits {
    pub outbox_records: u32,
    pub inbox_records: u32,
    pub retained_payload_bytes: u64,
}

impl Default for ReliableLimits {
    fn default() -> Self {
        Self {
            outbox_records: 1024,
            inbox_records: 1024,
            retained_payload_bytes: 4 * 1024 * 1024,
        }
    }
}

impl ReliableLimits {
    fn validate(&self) -> Result<(), Error> {
        let max = Self::default();
        if !(1..=max.outbox_records).contains(&self.outbox_records)
            || !(1..=max.inbox_records).contains(&self.inbox_records)
            || !(1..=max.retained_payload_bytes).contains(&self.retained_payload_bytes)
        {
            return Err(Error::InvalidConfig);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiveAck {
    pub run: Id,
    pub config: Id,
    pub logical_id: Id,
    pub source: Endpoint,
    pub destination: Endpoint,
    pub channel: ChannelId,
    pub context: Option<Id>,
    pub codec: Id,
    pub origin: Origin,
    pub payload_hash: Id,
    pub receiver_incarnation: u64,
    pub custody_transition_id: Id,
    pub storage_receipt: Id,
}

impl ReceiveAck {
    fn header(&self) -> MessageHeader {
        MessageHeader {
            run: self.run,
            config: self.config,
            source: self.source,
            destination: self.destination,
            channel: self.channel,
            context: self.context,
            codec: self.codec,
            origin: self.origin,
            payload_hash: self.payload_hash,
            logical_id: self.logical_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum MailboxAction {
    Enqueue {
        message: Message,
        now_us: Tick,
    },
    Attempt {
        logical_id: Id,
        now_us: Tick,
        destination: EndpointBinding,
    },
    Receive {
        envelope: Envelope,
        received_tick_us: Tick,
        received_local_us: Option<LocalTime>,
        authenticated_source: EndpointBinding,
    },
    MarkConsumed {
        logical_id: Id,
    },
    Acknowledge {
        ack: ReceiveAck,
        authenticated_receiver: EndpointBinding,
    },
    Rebind {
        expected_incarnation: u64,
        new_incarnation: u64,
    },
}

/// The adapter supplies this assertion after durable persistence. The mailbox
/// verifies the binding to the plan, while the adapter vouches for the storage.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DurabilityReceipt {
    pub owner: Endpoint,
    pub base_revision: u64,
    pub transition_id: Id,
    pub candidate_record_hash: Id,
    pub storage_receipt: Id,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InboxItem {
    pub message: Message,
    pub first_receive_tick_us: Tick,
    pub first_receive_local_us: Option<LocalTime>,
    pub consumed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum CustodyRelease {
    Enqueued { logical_id: Id, retry_due_us: Tick },
    Send(Envelope),
    ReceiveAck(ReceiveAck),
    InboxAvailable { logical_id: Id },
    Acknowledged { logical_id: Id },
    Rebound(EndpointBinding),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommittedCustody {
    pub transition_id: Id,
    pub revision: u64,
    pub releases: Vec<CustodyRelease>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OutboxItem {
    header: MessageHeader,
    payload: Option<Vec<u8>>,
    last_attempt: u32,
    retry_due_us: Tick,
    last_envelope: Option<Envelope>,
    acknowledged: bool,
}

impl OutboxItem {
    fn matches_message(&self, message: &Message) -> bool {
        self.header == message.header()
            && self
                .payload
                .as_ref()
                .is_none_or(|payload| payload == &message.payload)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MailboxSnapshot {
    schema: u32,
    kernel_config_hash: Id,
    owner: Endpoint,
    incarnation: u64,
    revision: u64,
    limits: ReliableLimits,
    retry_interval_us: u64,
    last_action_tick_us: Tick,
    outbox: Vec<OutboxItem>,
    inbox: Vec<InboxItem>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum ReleaseIntent {
    Enqueued { logical_id: Id, retry_due_us: Tick },
    SendEnvelope { envelope: Envelope },
    AckFor { logical_id: Id },
    InboxAvailable { logical_id: Id },
    Acknowledged { logical_id: Id },
    Rebound { binding: EndpointBinding },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistenceRecord {
    schema: u32,
    owner: Endpoint,
    base_revision: u64,
    transition_id: Id,
    action: MailboxAction,
    candidate: MailboxSnapshot,
    release_intents: Vec<ReleaseIntent>,
}

#[derive(Debug)]
pub struct ReliableMailbox {
    instance: Instance,
    config: Arc<TransportConfig>,
    state: MailboxSnapshot,
}

impl Clone for ReliableMailbox {
    fn clone(&self) -> Self {
        Self {
            instance: Instance::new(),
            config: Arc::clone(&self.config),
            state: self.state.clone(),
        }
    }
}

#[derive(Debug)]
pub struct CustodyPlan {
    instance: Instance,
    record: PersistenceRecord,
    bytes: Vec<u8>,
    hash: Id,
}

impl CustodyPlan {
    pub fn owner(&self) -> Endpoint {
        self.record.owner
    }

    pub fn base_revision(&self) -> u64 {
        self.record.base_revision
    }

    pub fn transition_id(&self) -> Id {
        self.record.transition_id
    }

    pub fn candidate_record_hash(&self) -> Id {
        self.hash
    }

    pub fn persistence_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl ReliableMailbox {
    pub fn new(
        config: TransportConfig,
        owner: Endpoint,
        limits: ReliableLimits,
        retry_interval_us: u64,
    ) -> Result<Self, Error> {
        let config = config.normalize()?;
        let incarnation = config.endpoint(owner)?.binding.incarnation;
        limits.validate()?;
        if retry_interval_us == 0 {
            return Err(Error::InvalidConfig);
        }
        let state = MailboxSnapshot {
            schema: SCHEMA,
            kernel_config_hash: config.normalized_hash()?,
            owner,
            incarnation,
            revision: 0,
            limits,
            retry_interval_us,
            last_action_tick_us: 0,
            outbox: Vec::new(),
            inbox: Vec::new(),
        };
        encode_snapshot(&state)?;
        Ok(Self {
            instance: Instance::new(),
            config: Arc::new(config),
            state,
        })
    }

    pub fn plan(&self, action: MailboxAction) -> Result<CustodyPlan, Error> {
        let mut candidate = self.state.clone();
        candidate.revision = candidate
            .revision
            .checked_add(1)
            .ok_or(Error::ArithmeticOverflow)?;
        let release_intents = candidate.apply(&self.config, &action)?;
        candidate.check_capacity(0)?;
        let base_revision = self.revision();
        let owner = self.state.owner;
        let record = PersistenceRecord {
            schema: SCHEMA,
            owner,
            base_revision,
            transition_id: transition_id(&self.config, owner, base_revision, &action)?,
            action,
            candidate,
            release_intents,
        };
        let bytes = encode_snapshot(&record)?;
        let hash = framed_hash("SIMT/custody-record/v1", &[&bytes]);
        Ok(CustodyPlan {
            instance: self.instance.for_plan(),
            record,
            bytes,
            hash,
        })
    }

    pub fn confirm(
        &mut self,
        plan: CustodyPlan,
        receipt: DurabilityReceipt,
    ) -> Result<CommittedCustody, Error> {
        self.instance.verify(&plan.instance)?;
        if plan.record.candidate.kernel_config_hash != self.state.kernel_config_hash {
            return Err(Error::ConfigMismatch);
        }
        if plan.base_revision() != self.revision() {
            return Err(Error::StalePlan);
        }
        verify_receipt(&plan.record, plan.hash, &receipt)?;
        let committed = plan.record.committed(&receipt)?;
        self.state = plan.record.candidate;
        Ok(committed)
    }

    pub fn pending_inbox(&self) -> Vec<InboxItem> {
        self.state
            .inbox
            .iter()
            .filter(|item| !item.consumed)
            .cloned()
            .collect()
    }

    pub fn retry_due_us(&self, logical_id: Id) -> Option<Tick> {
        let item = self.state.outbox_item(logical_id).ok()?;
        (!item.acknowledged).then_some(item.retry_due_us)
    }

    pub fn revision(&self) -> u64 {
        self.state.revision
    }

    pub fn restore(
        config: TransportConfig,
        record_bytes: &[u8],
        receipt: DurabilityReceipt,
    ) -> Result<(Self, CommittedCustody), Error> {
        let record: PersistenceRecord = decode_snapshot(record_bytes)?;
        schema(record.schema)?;
        schema(record.candidate.schema)?;
        let config = config.normalize()?;
        if record.candidate.kernel_config_hash != config.normalized_hash()? {
            return Err(Error::ConfigMismatch);
        }
        record.validate(&config)?;
        verify_receipt(
            &record,
            framed_hash("SIMT/custody-record/v1", &[record_bytes]),
            &receipt,
        )?;
        let committed = record.committed(&receipt)?;
        let mailbox = Self {
            instance: Instance::new(),
            config: Arc::new(config),
            state: record.candidate,
        };
        Ok((mailbox, committed))
    }
}

impl MailboxSnapshot {
    fn outbox_index(&self, id: Id) -> Result<usize, usize> {
        self.outbox
            .binary_search_by_key(&id, |item| item.header.logical_id)
    }

    fn inbox_index(&self, id: Id) -> Result<usize, usize> {
        self.inbox
            .binary_search_by_key(&id, |item| item.message.logical_id)
    }

    fn outbox_item(&self, id: Id) -> Result<&OutboxItem, Error> {
        let index = self.outbox_index(id).map_err(|_| Error::UnknownMessage)?;
        Ok(&self.outbox[index])
    }

    fn inbox_item(&self, id: Id) -> Result<&InboxItem, Error> {
        let index = self.inbox_index(id).map_err(|_| Error::UnknownMessage)?;
        Ok(&self.inbox[index])
    }

    fn check_capacity(&self, additional_bytes: usize) -> Result<(), Error> {
        let outbox_bytes = self
            .outbox
            .iter()
            .filter_map(|item| item.payload.as_ref())
            .map(Vec::len)
            .sum::<usize>();
        let inbox_bytes = self
            .inbox
            .iter()
            .map(|item| item.message.payload.len())
            .sum::<usize>();
        if self.outbox.len() > self.limits.outbox_records as usize
            || self.inbox.len() > self.limits.inbox_records as usize
            || (outbox_bytes + inbox_bytes + additional_bytes) as u64
                > self.limits.retained_payload_bytes
        {
            return Err(Error::ResourceLimit);
        }
        Ok(())
    }

    fn apply(
        &mut self,
        config: &TransportConfig,
        action: &MailboxAction,
    ) -> Result<Vec<ReleaseIntent>, Error> {
        match action {
            MailboxAction::Enqueue { message, now_us } => self.enqueue(config, message, *now_us),
            MailboxAction::Attempt {
                logical_id,
                now_us,
                destination,
            } => self.attempt(config, *logical_id, *now_us, *destination),
            MailboxAction::Receive {
                envelope,
                received_tick_us,
                received_local_us,
                authenticated_source,
            } => self.receive(
                config,
                envelope,
                *received_tick_us,
                *received_local_us,
                *authenticated_source,
            ),
            MailboxAction::MarkConsumed { logical_id } => {
                let index = self
                    .inbox_index(*logical_id)
                    .map_err(|_| Error::UnknownMessage)?;
                self.inbox[index].consumed = true;
                Ok(Vec::new())
            }
            MailboxAction::Acknowledge {
                ack,
                authenticated_receiver,
            } => self.acknowledge(config, ack, *authenticated_receiver),
            MailboxAction::Rebind {
                expected_incarnation,
                new_incarnation,
            } => {
                self.incarnation =
                    rebind(self.incarnation, *expected_incarnation, *new_incarnation)?;
                Ok(vec![ReleaseIntent::Rebound {
                    binding: EndpointBinding {
                        endpoint: self.owner,
                        incarnation: self.incarnation,
                    },
                }])
            }
        }
    }

    fn check_enqueue(
        &self,
        config: &TransportConfig,
        message: &Message,
        now: Tick,
    ) -> Result<(), Error> {
        config.check_message(message, true)?;
        if message.source != self.owner {
            return Err(Error::BindingMismatch);
        }
        if message.origin.sent_tick_us > now || message.origin.source_incarnation > self.incarnation
        {
            return Err(Error::InvalidOrigin);
        }
        Ok(())
    }

    fn enqueue(
        &mut self,
        config: &TransportConfig,
        message: &Message,
        now_us: Tick,
    ) -> Result<Vec<ReleaseIntent>, Error> {
        if now_us < self.last_action_tick_us {
            return Err(Error::TimeRegression);
        }
        self.check_enqueue(config, message, now_us)?;
        self.last_action_tick_us = now_us;
        let logical_id = message.logical_id;
        let index = match self.outbox_index(logical_id) {
            Ok(index) => {
                if !self.outbox[index].matches_message(message) {
                    return Err(Error::MessageConflict);
                }
                return Ok(Vec::new());
            }
            Err(index) => index,
        };
        if self.outbox.len() >= self.limits.outbox_records as usize {
            return Err(Error::ResourceLimit);
        }
        self.check_capacity(message.payload.len())?;
        self.outbox.insert(
            index,
            OutboxItem {
                header: message.header(),
                payload: Some(message.payload.clone()),
                last_attempt: 0,
                retry_due_us: now_us,
                last_envelope: None,
                acknowledged: false,
            },
        );
        Ok(vec![ReleaseIntent::Enqueued {
            logical_id,
            retry_due_us: now_us,
        }])
    }

    fn check_attempt_destination(
        &self,
        config: &TransportConfig,
        item: &OutboxItem,
        destination: EndpointBinding,
    ) -> Result<(), Error> {
        let initial = config
            .endpoint(item.header.destination)?
            .binding
            .incarnation;
        if destination.endpoint != item.header.destination
            || destination.incarnation < initial
            || item
                .last_envelope
                .as_ref()
                .is_some_and(|last| destination.incarnation < last.destination_incarnation)
        {
            return Err(Error::BindingMismatch);
        }
        Ok(())
    }

    fn attempt(
        &mut self,
        config: &TransportConfig,
        logical_id: Id,
        now_us: Tick,
        destination: EndpointBinding,
    ) -> Result<Vec<ReleaseIntent>, Error> {
        if now_us < self.last_action_tick_us {
            return Err(Error::TimeRegression);
        }
        let index = self
            .outbox_index(logical_id)
            .map_err(|_| Error::UnknownMessage)?;
        let item = &self.outbox[index];
        self.check_attempt_destination(config, item, destination)?;
        let payload = item.payload.as_ref().ok_or(Error::UnknownMessage)?;
        if now_us < item.retry_due_us {
            return Err(Error::RetryNotDue);
        }
        let attempt = item
            .last_attempt
            .checked_add(1)
            .ok_or(Error::ArithmeticOverflow)?;
        let retry_due = now_us
            .checked_add(self.retry_interval_us)
            .ok_or(Error::ArithmeticOverflow)?;
        let envelope = Envelope {
            message: item.header.with_payload(payload.clone()),
            attempt,
            source_incarnation: self.incarnation,
            destination_incarnation: destination.incarnation,
        };
        let item = &mut self.outbox[index];
        item.last_attempt = attempt;
        item.retry_due_us = retry_due;
        item.last_envelope = Some(envelope.clone());
        self.last_action_tick_us = now_us;
        Ok(vec![ReleaseIntent::SendEnvelope { envelope }])
    }

    fn check_receive(
        &self,
        config: &TransportConfig,
        envelope: &Envelope,
        received_tick_us: Tick,
        authenticated_source: EndpointBinding,
    ) -> Result<(), Error> {
        let message = &envelope.message;
        config.check_message(message, true)?;
        if message.destination != self.owner
            || envelope.destination_incarnation != self.incarnation
            || envelope.source_incarnation < config.endpoint(message.source)?.binding.incarnation
            || authenticated_source
                != (EndpointBinding {
                    endpoint: message.source,
                    incarnation: envelope.source_incarnation,
                })
        {
            return Err(Error::BindingMismatch);
        }
        if envelope.attempt == 0 {
            return Err(Error::InvalidAttempt);
        }
        if message.origin.sent_tick_us > received_tick_us
            || message.origin.source_incarnation > envelope.source_incarnation
        {
            return Err(Error::InvalidOrigin);
        }
        Ok(())
    }

    fn receive(
        &mut self,
        config: &TransportConfig,
        envelope: &Envelope,
        received_tick_us: Tick,
        received_local_us: Option<LocalTime>,
        authenticated_source: EndpointBinding,
    ) -> Result<Vec<ReleaseIntent>, Error> {
        self.check_receive(config, envelope, received_tick_us, authenticated_source)?;
        let message = &envelope.message;
        let logical_id = message.logical_id;
        let index = match self.inbox_index(logical_id) {
            Ok(index) => {
                if self.inbox[index].message != *message {
                    return Err(Error::MessageConflict);
                }
                return Ok(vec![ReleaseIntent::AckFor { logical_id }]);
            }
            Err(index) => index,
        };
        if self.inbox.len() >= self.limits.inbox_records as usize {
            return Err(Error::ResourceLimit);
        }
        self.check_capacity(message.payload.len())?;
        self.inbox.insert(
            index,
            InboxItem {
                message: message.clone(),
                first_receive_tick_us: received_tick_us,
                first_receive_local_us: received_local_us,
                consumed: false,
            },
        );
        Ok(vec![
            ReleaseIntent::InboxAvailable { logical_id },
            ReleaseIntent::AckFor { logical_id },
        ])
    }

    fn check_ack(
        &self,
        config: &TransportConfig,
        item: &OutboxItem,
        ack: &ReceiveAck,
        authenticated_receiver: EndpointBinding,
    ) -> Result<(), Error> {
        if ack.run != config.run || ack.config != config.config {
            return Err(Error::ConfigMismatch);
        }
        let initial = config
            .endpoint(item.header.destination)?
            .binding
            .incarnation;
        if authenticated_receiver
            != (EndpointBinding {
                endpoint: item.header.destination,
                incarnation: ack.receiver_incarnation,
            })
            || ack.receiver_incarnation < initial
            || item
                .last_envelope
                .as_ref()
                .is_some_and(|last| ack.receiver_incarnation < last.destination_incarnation)
        {
            return Err(Error::BindingMismatch);
        }
        if ack.header() != item.header {
            return Err(Error::MessageConflict);
        }
        if ack.storage_receipt == [0; 32] || ack.custody_transition_id == [0; 32] {
            return Err(Error::InvalidReceipt);
        }
        Ok(())
    }

    fn acknowledge(
        &mut self,
        config: &TransportConfig,
        ack: &ReceiveAck,
        authenticated_receiver: EndpointBinding,
    ) -> Result<Vec<ReleaseIntent>, Error> {
        let index = self
            .outbox_index(ack.logical_id)
            .map_err(|_| Error::UnknownMessage)?;
        self.check_ack(config, &self.outbox[index], ack, authenticated_receiver)?;
        let item = &mut self.outbox[index];
        if item.acknowledged {
            return Ok(Vec::new());
        }
        if item.last_attempt == 0 || item.last_envelope.is_none() {
            return Err(Error::InvalidReceipt);
        }
        item.payload = None;
        item.last_envelope = None;
        item.acknowledged = true;
        Ok(vec![ReleaseIntent::Acknowledged {
            logical_id: ack.logical_id,
        }])
    }

    fn validate(&self, config: &TransportConfig) -> Result<(), Error> {
        self.limits.validate()?;
        self.check_capacity(0)?;
        if self.incarnation < config.endpoint(self.owner)?.binding.incarnation
            || self.retry_interval_us == 0
            || !strictly_ordered(&self.outbox, |i| i.header.logical_id)
            || !strictly_ordered(&self.inbox, |i| i.message.logical_id)
        {
            return Err(Error::CorruptSnapshot);
        }
        for item in &self.outbox {
            config.check_header(&item.header, true)?;
            if item.header.source != self.owner
                || item.header.origin.source_incarnation > self.incarnation
                || item.header.origin.sent_tick_us > self.last_action_tick_us
            {
                return Err(Error::CorruptSnapshot);
            }
            if item.acknowledged {
                if item.payload.is_some() || item.last_envelope.is_some() || item.last_attempt == 0
                {
                    return Err(Error::CorruptSnapshot);
                }
                continue;
            }
            let payload = item.payload.as_ref().ok_or(Error::CorruptSnapshot)?;
            config.check_message(&item.header.with_payload(payload.clone()), true)?;
            match &item.last_envelope {
                None if item.last_attempt == 0 && item.retry_due_us <= self.last_action_tick_us => {
                }
                Some(envelope)
                    if item.last_attempt != 0
                        && envelope.attempt == item.last_attempt
                        && item.matches_message(&envelope.message) =>
                {
                    self.check_attempt_destination(
                        config,
                        item,
                        EndpointBinding {
                            endpoint: envelope.message.destination,
                            incarnation: envelope.destination_incarnation,
                        },
                    )?;
                    if envelope.source_incarnation < item.header.origin.source_incarnation
                        || envelope.source_incarnation > self.incarnation
                        || item
                            .retry_due_us
                            .checked_sub(self.retry_interval_us)
                            .is_none_or(|attempt_at| {
                                attempt_at > self.last_action_tick_us
                                    || attempt_at < item.header.origin.sent_tick_us
                            })
                    {
                        return Err(Error::CorruptSnapshot);
                    }
                }
                _ => return Err(Error::CorruptSnapshot),
            }
        }
        for item in &self.inbox {
            config.check_message(&item.message, true)?;
            if item.message.destination != self.owner
                || item.message.origin.sent_tick_us > item.first_receive_tick_us
            {
                return Err(Error::CorruptSnapshot);
            }
        }
        Ok(())
    }
}

impl PersistenceRecord {
    fn validate(&self, config: &TransportConfig) -> Result<(), Error> {
        if self.owner != self.candidate.owner
            || self.base_revision.checked_add(1) != Some(self.candidate.revision)
            || transition_id(config, self.owner, self.base_revision, &self.action)?
                != self.transition_id
        {
            return Err(Error::CorruptSnapshot);
        }
        self.candidate
            .validate(config)
            .map_err(|_| Error::CorruptSnapshot)?;
        self.validate_action(config)
            .map_err(|_| Error::CorruptSnapshot)
    }

    // Only checks consequences visible in the candidate and its ordered release
    // intents. Authenticating the previous state and record chain is external.
    fn validate_action(&self, config: &TransportConfig) -> Result<(), Error> {
        let state = &self.candidate;
        let consistent = match &self.action {
            MailboxAction::Enqueue { message, now_us } => {
                state.check_enqueue(config, message, *now_us)?;
                let logical_id = message.logical_id;
                let item = state.outbox_item(logical_id)?;
                let new = [ReleaseIntent::Enqueued {
                    logical_id,
                    retry_due_us: *now_us,
                }];
                item.matches_message(message)
                    && state.last_action_tick_us == *now_us
                    && (self.release_intents.is_empty()
                        || (self.release_intents == new
                            && !item.acknowledged
                            && item.last_attempt == 0
                            && item.retry_due_us == *now_us))
            }
            MailboxAction::Attempt {
                logical_id,
                now_us,
                destination,
            } => {
                let item = state.outbox_item(*logical_id)?;
                state.check_attempt_destination(config, item, *destination)?;
                let envelope = item.last_envelope.as_ref().ok_or(Error::CorruptSnapshot)?;
                state.last_action_tick_us == *now_us
                    && now_us.checked_add(state.retry_interval_us) == Some(item.retry_due_us)
                    && envelope.source_incarnation == state.incarnation
                    && envelope.destination_incarnation == destination.incarnation
                    && matches!(self.release_intents.as_slice(),
                        [ReleaseIntent::SendEnvelope { envelope: intent }] if intent == envelope)
            }
            MailboxAction::Receive {
                envelope,
                received_tick_us,
                received_local_us,
                authenticated_source,
            } => {
                state.check_receive(config, envelope, *received_tick_us, *authenticated_source)?;
                let logical_id = envelope.message.logical_id;
                let item = state.inbox_item(logical_id)?;
                let ack = ReleaseIntent::AckFor { logical_id };
                item.message == envelope.message
                    && (self.release_intents == [ack.clone()]
                        || (self.release_intents
                            == [ReleaseIntent::InboxAvailable { logical_id }, ack]
                            && !item.consumed
                            && item.first_receive_tick_us == *received_tick_us
                            && item.first_receive_local_us == *received_local_us))
            }
            MailboxAction::MarkConsumed { logical_id } => {
                state.inbox_item(*logical_id)?.consumed && self.release_intents.is_empty()
            }
            MailboxAction::Acknowledge {
                ack,
                authenticated_receiver,
            } => {
                let item = state.outbox_item(ack.logical_id)?;
                state.check_ack(config, item, ack, *authenticated_receiver)?;
                item.acknowledged
                    && (self.release_intents.is_empty()
                        || self.release_intents
                            == [ReleaseIntent::Acknowledged {
                                logical_id: ack.logical_id,
                            }])
            }
            MailboxAction::Rebind {
                expected_incarnation,
                new_incarnation,
            } => {
                let initial = config.endpoint(state.owner)?.binding.incarnation;
                *expected_incarnation >= initial
                    && expected_incarnation.checked_add(1) == Some(*new_incarnation)
                    && *new_incarnation == state.incarnation
                    && self.release_intents
                        == [ReleaseIntent::Rebound {
                            binding: EndpointBinding {
                                endpoint: state.owner,
                                incarnation: *new_incarnation,
                            },
                        }]
            }
        };
        if !consistent {
            return Err(Error::CorruptSnapshot);
        }
        Ok(())
    }

    fn committed(&self, receipt: &DurabilityReceipt) -> Result<CommittedCustody, Error> {
        let mut releases = Vec::with_capacity(self.release_intents.len());
        for intent in &self.release_intents {
            let release = match intent {
                ReleaseIntent::Enqueued {
                    logical_id,
                    retry_due_us,
                } => CustodyRelease::Enqueued {
                    logical_id: *logical_id,
                    retry_due_us: *retry_due_us,
                },
                ReleaseIntent::SendEnvelope { envelope } => CustodyRelease::Send(envelope.clone()),
                ReleaseIntent::AckFor { logical_id } => {
                    let m = &self.candidate.inbox_item(*logical_id)?.message;
                    CustodyRelease::ReceiveAck(ReceiveAck {
                        run: m.run,
                        config: m.config,
                        logical_id: m.logical_id,
                        source: m.source,
                        destination: m.destination,
                        channel: m.channel,
                        context: m.context,
                        codec: m.codec,
                        origin: m.origin,
                        payload_hash: m.payload_hash,
                        receiver_incarnation: self.candidate.incarnation,
                        custody_transition_id: self.transition_id,
                        storage_receipt: receipt.storage_receipt,
                    })
                }
                ReleaseIntent::InboxAvailable { logical_id } => CustodyRelease::InboxAvailable {
                    logical_id: *logical_id,
                },
                ReleaseIntent::Acknowledged { logical_id } => CustodyRelease::Acknowledged {
                    logical_id: *logical_id,
                },
                ReleaseIntent::Rebound { binding } => CustodyRelease::Rebound(*binding),
            };
            releases.push(release);
        }
        Ok(CommittedCustody {
            transition_id: self.transition_id,
            revision: self.candidate.revision,
            releases,
        })
    }
}

fn transition_id(
    config: &TransportConfig,
    owner: Endpoint,
    base_revision: u64,
    action: &MailboxAction,
) -> Result<Id, Error> {
    Ok(framed_hash(
        "SIMT/custody-transition/v1",
        &[
            &config.run,
            &config.config,
            &owner.hash_bytes(),
            &base_revision.to_be_bytes(),
            &encode(action)?,
        ],
    ))
}

fn verify_receipt(
    record: &PersistenceRecord,
    hash: Id,
    receipt: &DurabilityReceipt,
) -> Result<(), Error> {
    if receipt.owner != record.owner
        || receipt.base_revision != record.base_revision
        || receipt.transition_id != record.transition_id
        || receipt.candidate_record_hash != hash
        || receipt.storage_receipt == [0; 32]
    {
        return Err(Error::InvalidReceipt);
    }
    Ok(())
}
