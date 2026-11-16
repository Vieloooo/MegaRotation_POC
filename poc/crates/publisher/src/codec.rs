use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use sim_espresso::{
    EspressoPosition, LogMessage, MAX_MESSAGE_BYTES, MAX_MESSAGES, MAX_SOURCE_BYTES, PublicTimeUs,
    TickUs,
};

use crate::{
    Attempt, AttemptId, AttemptState, LifecycleReason, MissingInput, PublicStop, PublicationTask,
    RecordedAt, Snapshot, StateCodec, StoreBinding, StoreError, store::MAX_SNAPSHOT_BYTES,
};

const HEADER_BYTES: usize = 14;
const MAX_TASKS: usize = 4_096;
const MAX_ATTEMPTS: usize = 4_096;
const MAX_MISSING: usize = 1_024;
const MAX_DOMAIN_RESULT_BYTES: usize = 4 * 1024 * 1024;
pub(crate) const MAX_REFERENCE_BYTES: usize = 65_536;

pub(crate) fn validate_binding(binding: &StoreBinding) -> Result<(), StoreError> {
    for value in [
        &binding.namespace,
        &binding.configuration,
        &binding.source_id,
        &binding.reader_id,
    ] {
        if !(1..=MAX_SOURCE_BYTES).contains(&value.len()) {
            return Err(StoreError::InvalidState("invalid store identity length"));
        }
    }
    Ok(())
}

pub(crate) fn encode_snapshot<K, R>(
    codec: &impl StateCodec<K, R>,
    binding: &StoreBinding,
    snapshot: &Snapshot<K, R>,
) -> Result<Vec<u8>, StoreError>
where
    K: Ord,
{
    if snapshot.tasks.len() > MAX_TASKS {
        return Err(StoreError::Capacity);
    }
    let mut out = Encoder(Vec::new());
    for value in [
        &binding.namespace,
        &binding.configuration,
        &binding.source_id,
        &binding.reader_id,
    ] {
        out.bytes(value, MAX_SOURCE_BYTES)?;
    }
    out.u64(snapshot.generation);
    out.u32(snapshot.tasks.len() as u32);
    for task in snapshot.tasks.values() {
        validate_task(task, binding, snapshot.tasks.len())?;
        out.bytes(&codec.encode_key(&task.key), MAX_REFERENCE_BYTES)?;
        out.bytes(&codec.encode_result(&task.result), MAX_DOMAIN_RESULT_BYTES)?;
        out.u64(task.task_number);
        out.u32(task.missing.len() as u32);
        for entry in &task.missing {
            out.u32(entry.code);
            out.bytes(&entry.reference, MAX_REFERENCE_BYTES)?;
        }
        if let Some(deadline) = task.verified_deadline {
            out.tag(1);
            out.u64(deadline.0);
        } else {
            out.tag(0);
        }
        if let Some(payload) = &task.verified_payload {
            out.tag(1);
            out.bytes(
                task.validation_basis.as_ref().expect("verified"),
                MAX_REFERENCE_BYTES,
            )?;
            out.u32(payload.len() as u32);
            for message in payload.iter() {
                out.u32(message.codec);
                out.bytes(&message.bytes, MAX_MESSAGE_BYTES)?;
            }
        } else {
            out.tag(0);
        }
        out.u32(task.attempts.len() as u32);
        for attempt in &task.attempts {
            out.u64(attempt.created_tick.0);
            match &attempt.state {
                AttemptState::Prepared => out.tag(0),
                AttemptState::Dispatched {
                    controlled_delivery_ref,
                } => {
                    out.tag(1);
                    out.bytes(controlled_delivery_ref, MAX_REFERENCE_BYTES)?;
                }
                AttemptState::ServiceAcknowledged {
                    controlled_delivery_ref,
                    service_ref,
                } => {
                    out.tag(2);
                    out.bytes(controlled_delivery_ref, MAX_REFERENCE_BYTES)?;
                    out.bytes(service_ref, MAX_REFERENCE_BYTES)?;
                }
            }
        }
        if let Some(receipt) = &task.recorded_at {
            out.tag(1);
            out.u64(receipt.publication_position.q);
            out.u32(receipt.publication_position.message_index);
            out.u64(receipt.complete_through_q);
            out.bytes(&receipt.source_id, MAX_SOURCE_BYTES)?;
            out.bytes(&receipt.verification_basis, MAX_REFERENCE_BYTES)?;
        } else {
            out.tag(0);
        }
        match &task.stop_reason {
            None => out.tag(0),
            Some(PublicStop::Lifecycle {
                reason,
                public_basis,
            }) => {
                out.tag(match reason {
                    LifecycleReason::SessionClosed => 1,
                    LifecycleReason::BranchClosed => 2,
                    LifecycleReason::OtherCidAssigned => 3,
                });
                out.bytes(public_basis, MAX_REFERENCE_BYTES)?;
            }
            Some(PublicStop::Deadline { q, theta, deadline }) => {
                out.tag(4);
                out.u64(*q);
                out.u64(theta.0);
                out.u64(deadline.0);
            }
        }
        if out.0.len() > MAX_SNAPSHOT_BYTES - HEADER_BYTES {
            return Err(StoreError::Capacity);
        }
    }
    let body = out.0;
    let mut envelope = Vec::with_capacity(HEADER_BYTES + body.len());
    envelope.extend_from_slice(b"PESP");
    envelope.extend_from_slice(&1u16.to_le_bytes());
    envelope.extend_from_slice(&(body.len() as u32).to_le_bytes());
    envelope.extend_from_slice(&crc32(&body).to_le_bytes());
    envelope.extend_from_slice(&body);
    Ok(envelope)
}

pub(crate) fn decode_snapshot<K, R>(
    codec: &impl StateCodec<K, R>,
    binding: &StoreBinding,
    bytes: &[u8],
) -> Result<Snapshot<K, R>, StoreError>
where
    K: Ord + Clone,
{
    if bytes.len() > MAX_SNAPSHOT_BYTES {
        return Err(StoreError::Capacity);
    }
    let mut envelope = Decoder(bytes);
    if envelope.take(4)? != b"PESP" {
        return Err(StoreError::BadMagic);
    }
    if envelope.take(2)? != 1u16.to_le_bytes() {
        return Err(StoreError::UnsupportedVersion);
    }
    let len = envelope.u32()? as usize;
    let checksum = envelope.u32()?;
    let body = envelope.take(len)?;
    if !envelope.0.is_empty() {
        return Err(StoreError::Malformed);
    }
    if crc32(body) != checksum {
        return Err(StoreError::Checksum);
    }
    let mut read = Decoder(body);
    for expected in [
        &binding.namespace,
        &binding.configuration,
        &binding.source_id,
        &binding.reader_id,
    ] {
        if read.bytes(MAX_SOURCE_BYTES)? != expected {
            return Err(StoreError::BindingMismatch);
        }
    }
    let generation = read.u64()?;
    let task_count = read.count(MAX_TASKS)?;
    if generation < task_count as u64 {
        return Err(StoreError::InvalidState("generation precedes task count"));
    }
    let mut tasks = BTreeMap::new();
    let mut numbers = BTreeSet::new();
    for _ in 0..task_count {
        let key_bytes = read.bytes(MAX_REFERENCE_BYTES)?;
        let key = codec.decode_key(key_bytes)?;
        if codec.encode_key(&key) != key_bytes
            || tasks.last_key_value().is_some_and(|(last, _)| last >= &key)
        {
            return Err(StoreError::InvalidState(
                "noncanonical or duplicate task key",
            ));
        }
        let result_bytes = read.bytes(MAX_DOMAIN_RESULT_BYTES)?;
        let result = codec.decode_result(result_bytes)?;
        if codec.encode_result(&result) != result_bytes {
            return Err(StoreError::InvalidState("noncanonical result"));
        }
        let task_number = read.u64()?;
        if !numbers.insert(task_number) {
            return Err(StoreError::InvalidState("duplicate task number"));
        }
        let missing_count = read.count(MAX_MISSING)?;
        let mut missing = Vec::new();
        for _ in 0..missing_count {
            missing.push(MissingInput {
                code: read.u32()?,
                reference: read.reference()?,
            });
        }
        let verified_deadline = if read.present()? {
            Some(PublicTimeUs(read.u64()?))
        } else {
            None
        };
        let (validation_basis, verified_payload): (_, Option<Arc<[LogMessage]>>) =
            if read.present()? {
                let basis = read.reference()?;
                let count = read.count(MAX_MESSAGES)?;
                let mut messages = Vec::new();
                for _ in 0..count {
                    let codec = read.u32()?;
                    let bytes = read.bytes(MAX_MESSAGE_BYTES)?.to_vec();
                    messages.push(LogMessage { codec, bytes });
                }
                (Some(basis), Some(Arc::from(messages)))
            } else {
                (None, None)
            };
        let attempt_count = read.count(MAX_ATTEMPTS)?;
        let mut attempts = Vec::new();
        for index in 0..attempt_count {
            let exact_payload = verified_payload
                .clone()
                .ok_or(StoreError::InvalidState("unverified attempt"))?;
            let created_tick = TickUs(read.u64()?);
            let state = match read.tag()? {
                0 => AttemptState::Prepared,
                1 => AttemptState::Dispatched {
                    controlled_delivery_ref: read.reference()?,
                },
                2 => AttemptState::ServiceAcknowledged {
                    controlled_delivery_ref: read.reference()?,
                    service_ref: read.reference()?,
                },
                _ => return Err(StoreError::Malformed),
            };
            let number = index as u64 + 1;
            attempts.push(Attempt {
                stable_id: AttemptId {
                    namespace: binding.namespace.clone(),
                    task_number,
                    attempt_number: number,
                },
                number,
                exact_payload,
                created_tick,
                state,
            });
        }
        let recorded_at = if read.present()? {
            Some(RecordedAt {
                publication_position: EspressoPosition {
                    q: read.u64()?,
                    message_index: read.u32()?,
                },
                complete_through_q: read.u64()?,
                source_id: read.bytes(MAX_SOURCE_BYTES)?.to_vec(),
                verification_basis: read.reference()?,
            })
        } else {
            None
        };
        let stop_reason = read_stop(&mut read)?;
        let task = PublicationTask {
            key: key.clone(),
            result,
            task_number,
            missing,
            verified_deadline,
            verified_payload,
            validation_basis,
            attempts,
            recorded_at,
            stop_reason,
        };
        validate_task(&task, binding, task_count)?;
        tasks.insert(key, task);
    }
    if !read.0.is_empty() {
        return Err(StoreError::Malformed);
    }
    Ok(Snapshot { generation, tasks })
}

fn read_stop(read: &mut Decoder<'_>) -> Result<Option<PublicStop>, StoreError> {
    let value = match read.tag()? {
        0 => return Ok(None),
        tag @ 1..=3 => PublicStop::Lifecycle {
            reason: match tag {
                1 => LifecycleReason::SessionClosed,
                2 => LifecycleReason::BranchClosed,
                _ => LifecycleReason::OtherCidAssigned,
            },
            public_basis: read.reference()?,
        },
        4 => PublicStop::Deadline {
            q: read.u64()?,
            theta: PublicTimeUs(read.u64()?),
            deadline: PublicTimeUs(read.u64()?),
        },
        _ => return Err(StoreError::Malformed),
    };
    Ok(Some(value))
}

fn validate_task<K, R>(
    task: &PublicationTask<K, R>,
    binding: &StoreBinding,
    task_count: usize,
) -> Result<(), StoreError> {
    if task.task_number == 0 || task.task_number > task_count as u64 {
        return Err(StoreError::InvalidState("task number is not contiguous"));
    }
    if task.missing.len() > MAX_MISSING || task.attempts.len() > MAX_ATTEMPTS {
        return Err(StoreError::Capacity);
    }
    if let Some(payload) = &task.verified_payload {
        if !task.missing.is_empty()
            || task
                .validation_basis
                .as_ref()
                .is_none_or(|bytes| bytes.is_empty())
            || task.verified_deadline.is_none()
        {
            return Err(StoreError::InvalidState("invalid verified task"));
        }
        validate_payload(&binding.source_id, payload)?;
    } else if task.missing.is_empty()
        || task.validation_basis.is_some()
        || !task.attempts.is_empty()
        || task.recorded_at.is_some()
    {
        return Err(StoreError::InvalidState("missing verified offer"));
    }
    let mut last_tick = None;
    for (index, attempt) in task.attempts.iter().enumerate() {
        let number = index as u64 + 1;
        if attempt.number != number
            || attempt.stable_id.attempt_number != number
            || attempt.stable_id.task_number != task.task_number
            || attempt.stable_id.namespace != binding.namespace
            || task.verified_payload.as_ref().is_none_or(|payload| {
                !Arc::ptr_eq(payload, &attempt.exact_payload) && *payload != attempt.exact_payload
            })
            || (attempt.state == AttemptState::Prepared && index + 1 != task.attempts.len())
            || last_tick.is_some_and(|tick| tick > attempt.created_tick)
        {
            return Err(StoreError::InvalidState("invalid attempt history"));
        }
        match &attempt.state {
            AttemptState::Prepared => (),
            AttemptState::Dispatched {
                controlled_delivery_ref,
            } => validate_ref(controlled_delivery_ref)?,
            AttemptState::ServiceAcknowledged {
                controlled_delivery_ref,
                service_ref,
            } => {
                validate_ref(controlled_delivery_ref)?;
                validate_ref(service_ref)?;
            }
        }
        last_tick = Some(attempt.created_tick);
    }
    if let Some(receipt) = &task.recorded_at {
        if receipt.source_id != binding.source_id
            || receipt.publication_position.q > receipt.complete_through_q
            || receipt.publication_position.message_index as usize >= MAX_MESSAGES
        {
            return Err(StoreError::InvalidState("invalid saved receipt"));
        }
        validate_ref(&receipt.verification_basis)?;
    }
    match &task.stop_reason {
        Some(PublicStop::Lifecycle { public_basis, .. }) => validate_ref(public_basis)?,
        Some(PublicStop::Deadline {
            theta, deadline, ..
        }) => {
            if *theta < *deadline || task.verified_deadline != Some(*deadline) {
                return Err(StoreError::InvalidState("invalid saved deadline stop"));
            }
        }
        None => (),
    }
    Ok(())
}

pub(crate) fn validate_payload(
    source_id: &[u8],
    messages: &[LogMessage],
) -> Result<(), StoreError> {
    if messages.is_empty() {
        return Err(StoreError::InvalidState("empty publication"));
    }
    sim_espresso::validate_message_list(source_id, messages)
        .map_err(|_| StoreError::InvalidState("invalid publication message list"))
}

fn validate_ref(value: &[u8]) -> Result<(), StoreError> {
    if (1..=MAX_REFERENCE_BYTES).contains(&value.len()) {
        Ok(())
    } else {
        Err(StoreError::InvalidState("invalid evidence reference"))
    }
}

struct Encoder(Vec<u8>);

impl Encoder {
    fn tag(&mut self, value: u8) {
        self.0.push(value);
    }
    fn u32(&mut self, value: u32) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }
    fn u64(&mut self, value: u64) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    fn bytes(&mut self, bytes: &[u8], max: usize) -> Result<(), StoreError> {
        if bytes.len() > max || self.0.len() + bytes.len() + 4 > MAX_SNAPSHOT_BYTES - HEADER_BYTES {
            return Err(StoreError::Capacity);
        }
        self.u32(bytes.len() as u32);
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}

struct Decoder<'a>(&'a [u8]);

impl<'a> Decoder<'a> {
    fn take(&mut self, len: usize) -> Result<&'a [u8], StoreError> {
        let (head, tail) = self.0.split_at_checked(len).ok_or(StoreError::Malformed)?;
        self.0 = tail;
        Ok(head)
    }
    fn tag(&mut self) -> Result<u8, StoreError> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, StoreError> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("exact length"),
        ))
    }
    fn u64(&mut self) -> Result<u64, StoreError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("exact length"),
        ))
    }
    fn present(&mut self) -> Result<bool, StoreError> {
        match self.tag()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(StoreError::Malformed),
        }
    }
    fn count(&mut self, max: usize) -> Result<usize, StoreError> {
        let value = self.u32()? as usize;
        if value > max {
            Err(StoreError::Capacity)
        } else {
            Ok(value)
        }
    }
    fn bytes(&mut self, max: usize) -> Result<&'a [u8], StoreError> {
        let len = self.count(max)?;
        self.take(len)
    }
    fn reference(&mut self) -> Result<Vec<u8>, StoreError> {
        Ok(self.bytes(MAX_REFERENCE_BYTES)?.to_vec())
    }
}

/// CRC32 detects torn or accidentally corrupted snapshots. It authenticates no data.
fn crc32(bytes: &[u8]) -> u32 {
    const TABLE: [u32; 256] = crc_table();
    !bytes.iter().fold(!0u32, |crc, byte| {
        (crc >> 8) ^ TABLE[((crc as u8) ^ byte) as usize]
    })
}

const fn crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut index = 0;
    while index < table.len() {
        let mut crc = index as u32;
        let mut bit = 0;
        while bit < 8 {
            crc = (crc >> 1) ^ if crc & 1 == 1 { 0xedb8_8320 } else { 0 };
            bit += 1;
        }
        table[index] = crc;
        index += 1;
    }
    table
}
