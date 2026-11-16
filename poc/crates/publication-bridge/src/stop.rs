use std::{collections::BTreeMap, sync::Arc};

use poc_core::metadata::l2_timestamp_us;
use poc_publisher::{LifecycleReason, Publisher, StopDecision};
use poc_types::domain::{
    Canonical, CanonicalObject, Cid, CloseReason, ConfigRef, Descriptor, Hash32, LifecycleBody,
    LifecycleEvent, ProgramConfig, PublicConfig, ReplayKind, ResultKey, regional_cid,
};
use poc_viewer_gfr::{GfrParameters, VerificationProfile, decode_position};
use sim_espresso::{MAX_SOURCE_BYTES, PublicTimeUs, VerifiedPrefix};

use crate::{
    BridgeError, BridgeResult, BridgeStateCodec, NativePublicationVerifier, RecordDelivery,
    bounded::{Reader, Writer, digest, identity},
    record::{MAX_PROJECTION, record_id, restored_events},
};

const HEADER: &[u8] = b"H3SL\0\x01";
const MAX_RECORDS: usize = 4096;
const MAX_SCOPES: usize = 4096;
const MAX_SNAPSHOT: usize = 64 * 1024 * 1024;

type Session = (u64, u64);

#[derive(Clone, Debug)]
pub struct StopLedgerBinding {
    pub config: PublicConfig,
    pub expected: VerificationProfile,
    pub parameters: GfrParameters,
    pub espresso_source: Vec<u8>,
    pub l1_source: Vec<u8>,
    pub publisher_reader: Vec<u8>,
    pub owner_reference: Vec<u8>,
}

/// The bound owner validates the exact candidate and current revision against
/// its own committed storage. The bridge only consumes the resulting receipt.
pub trait DurableCommitPort {
    fn verify_commit(&self, candidate_hash: Hash32, receipt: &[u8]) -> Result<(), BridgeError>;
}

pub struct StopLedger {
    binding: StopLedgerBinding,
    config_ref: ConfigRef,
    encoded_binding: Vec<u8>,
    instance: Arc<()>,
    state: LedgerState,
}

pub struct StopPlan {
    instance: Arc<()>,
    base_revision: u64,
    state: LedgerState,
    bytes: Vec<u8>,
    hash: Hash32,
}

#[derive(Clone, Default)]
struct LedgerState {
    revision: u64,
    records: BTreeMap<Vec<u8>, StoredRecord>,
    branches: BTreeMap<u64, PublicBasis>,
    sessions: BTreeMap<Session, PublicBasis>,
    selected: BTreeMap<Session, (Cid, PublicBasis)>,
    deadlines: BTreeMap<ResultKey, Watermark>,
}

#[derive(Clone)]
struct StoredRecord {
    id: Hash32,
    parameters: GfrParameters,
    projection: Arc<[u8]>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PublicBasis {
    position: Vec<u8>,
    event_ordinal: usize,
    record_id: Hash32,
    event_id: Hash32,
}

#[derive(Clone)]
struct Watermark {
    descriptor: Descriptor,
    q: u64,
    theta: PublicTimeUs,
    deadline: PublicTimeUs,
}

impl StopLedger {
    pub fn new(binding: StopLedgerBinding) -> Result<Self, BridgeError> {
        let config_ref = binding
            .config
            .reference()
            .map_err(|_| BridgeError::InvalidConfig)?;
        let encoded_binding = encode_binding(&binding)?;
        Ok(Self {
            binding,
            config_ref,
            encoded_binding,
            instance: Arc::new(()),
            state: LedgerState::default(),
        })
    }

    pub fn restore(
        binding: StopLedgerBinding,
        snapshot: &[u8],
        receipt: &[u8],
        owner: &dyn DurableCommitPort,
    ) -> Result<Self, BridgeError> {
        let mut ledger = Self::new(binding)?;
        if snapshot.len() > MAX_SNAPSHOT {
            return Err(BridgeError::Capacity);
        }
        let mut input = Reader(snapshot);
        if input.take(HEADER.len())? != HEADER
            || input.frame(MAX_SNAPSHOT)? != ledger.encoded_binding
        {
            return Err(BridgeError::SourceMismatch);
        }
        owner.verify_commit(snapshot_hash(snapshot), receipt)?;
        let revision = input.u64()?;
        let mut state = LedgerState {
            revision,
            ..LedgerState::default()
        };
        for _ in 0..input.count(MAX_RECORDS)? {
            let position = input.frame(35)?;
            if state
                .records
                .last_key_value()
                .is_some_and(|(last, _)| last.as_slice() >= position)
            {
                return Err(BridgeError::InvalidEncoding);
            }
            let id = Hash32::new(input.fixed()?);
            let parameters = read_parameters(&mut input)?;
            if parameters != ledger.binding.parameters {
                return Err(BridgeError::SourceMismatch);
            }
            let projection = input.frame(MAX_PROJECTION)?;
            if id != ledger.record_identity(position, projection)? {
                return Err(BridgeError::InvalidEncoding);
            }
            let decoded_position = decode_position(position)?;
            let events = restored_events(projection, &decoded_position)?;
            state.apply_events(&ledger, position, id, &events)?;
            state.records.insert(
                position.to_vec(),
                StoredRecord {
                    id,
                    parameters,
                    projection: Arc::from(projection),
                },
            );
        }
        for _ in 0..input.count(MAX_SCOPES)? {
            let key = ResultKey::decode(input.frame(66)?)?;
            if state
                .deadlines
                .last_key_value()
                .is_some_and(|(last, _)| *last >= key)
            {
                return Err(BridgeError::InvalidEncoding);
            }
            let descriptor = Descriptor::decode(input.frame(MAX_PROJECTION)?)?;
            ledger.verify_descriptor(&descriptor)?;
            let (q, theta, deadline) = (
                input.u64()?,
                PublicTimeUs(input.u64()?),
                PublicTimeUs(input.u64()?),
            );
            if key.cid != descriptor.cid
                || deadline != ledger.deadline(&descriptor)?
                || theta < deadline
            {
                return Err(BridgeError::InvalidEncoding);
            }
            state.deadlines.insert(
                key,
                Watermark {
                    descriptor,
                    q,
                    theta,
                    deadline,
                },
            );
        }
        input.finish()?;
        state.check_scopes()?;
        // Each record or watermark is first accepted in a distinct revision.
        if usize::try_from(revision).ok() != Some(state.records.len() + state.deadlines.len()) {
            return Err(BridgeError::InvalidEncoding);
        }
        ledger.state = state;
        Ok(ledger)
    }

    pub fn plan_record(&self, record: &RecordDelivery) -> Result<StopPlan, BridgeError> {
        if record.config != self.config_ref
            || record.espresso != self.binding.espresso_source
            || record.l1 != self.binding.l1_source
            || record.profile != self.binding.expected
            || record.parameters != self.binding.parameters
        {
            return Err(BridgeError::SourceMismatch);
        }
        if let Some(prior) = self.state.records.get(&record.position) {
            if prior.id != record.id() || prior.projection.as_ref() != record.projection {
                return Err(BridgeError::InvalidResult);
            }
            return self.plan(self.state.clone());
        }
        if self.state.records.len() >= MAX_RECORDS {
            return Err(BridgeError::Capacity);
        }
        let mut candidate = self.state.clone();
        candidate.apply_events(self, &record.position, record.id(), &record.record().events)?;
        candidate.records.insert(
            record.position.clone(),
            StoredRecord {
                id: record.id(),
                parameters: record.parameters,
                projection: Arc::from(record.projection.as_slice()),
            },
        );
        candidate.revision = candidate
            .revision
            .checked_add(1)
            .ok_or(BridgeError::Capacity)?;
        self.plan(candidate)
    }

    pub fn plan_watermark(
        &self,
        publisher: &Publisher<NativePublicationVerifier<'_>, BridgeStateCodec>,
        key: &ResultKey,
        prefix: &VerifiedPrefix<'_>,
    ) -> Result<Option<StopPlan>, BridgeError> {
        if prefix.source_id() != self.binding.espresso_source
            || prefix.reader_id() != self.binding.publisher_reader
        {
            return Err(BridgeError::SourceMismatch);
        }
        let task = publisher.task(key).ok_or(BridgeError::InvalidResult)?;
        let descriptor = &task.result.result().descriptor;
        self.verify_descriptor(descriptor)?;
        if task.key != *key || task.result.key() != *key {
            return Err(BridgeError::InvalidResult);
        }
        let Some(deadline) = task.verified_deadline else {
            return Ok(None);
        };
        if deadline != self.deadline(descriptor)? {
            return Err(BridgeError::SourceMismatch);
        }
        let (q, theta) = prefix.tip();
        if theta < deadline {
            return Ok(None);
        }
        if self.state.deadlines.contains_key(key) {
            return self.plan(self.state.clone()).map(Some);
        }
        let mut candidate = self.state.clone();
        candidate.deadlines.insert(
            *key,
            Watermark {
                descriptor: descriptor.clone(),
                q,
                theta,
                deadline,
            },
        );
        candidate.revision = candidate
            .revision
            .checked_add(1)
            .ok_or(BridgeError::Capacity)?;
        candidate.check_scopes()?;
        self.plan(candidate).map(Some)
    }

    pub fn confirm(
        &mut self,
        plan: StopPlan,
        receipt: &[u8],
        owner: &dyn DurableCommitPort,
    ) -> Result<(), BridgeError> {
        if !Arc::ptr_eq(&self.instance, &plan.instance) || plan.base_revision != self.state.revision
        {
            return Err(BridgeError::SourceMismatch);
        }
        owner.verify_commit(plan.hash, receipt)?;
        self.state = plan.state;
        Ok(())
    }

    /// A retained lifecycle scope applies even if the result was offered later.
    pub fn public_stop(
        &self,
        result: &BridgeResult,
    ) -> Result<Option<StopDecision<ResultKey>>, BridgeError> {
        let descriptor = &result.result().descriptor;
        self.verify_descriptor(descriptor)?;
        let session = (descriptor.plan.session.beta, descriptor.plan.session.r);
        let Some((basis, reason)) = self.state.stop_for(session, descriptor.cid) else {
            return Ok(None);
        };
        let mut reference = Writer::new(65_536);
        reference.push(b"H3STOP\0\x01")?;
        reference.frame(&self.config_ref.encode())?;
        reference.frame(&self.binding.espresso_source)?;
        reference.frame(&self.binding.l1_source)?;
        write_parameters(&mut reference, self.binding.parameters)?;
        reference.frame(&basis.position)?;
        reference.push(basis.record_id.as_bytes())?;
        reference.push(basis.event_id.as_bytes())?;
        Ok(Some(StopDecision {
            affected_keys: vec![result.key()],
            reason,
            public_basis: reference.bytes,
        }))
    }

    /// Lifecycle and deadline facts remain effective while the service owner
    /// retries their 06 projection. Dispatch must recheck in the same grant.
    pub fn permit_dispatch(&self, result: &BridgeResult) -> Result<bool, BridgeError> {
        Ok(
            self.public_stop(result)?.is_none()
                && !self.state.deadlines.contains_key(&result.key()),
        )
    }

    pub fn deadline_watermark(
        &self,
        result: &BridgeResult,
    ) -> Result<Option<(u64, PublicTimeUs, PublicTimeUs)>, BridgeError> {
        self.verify_descriptor(&result.result().descriptor)?;
        Ok(self
            .state
            .deadlines
            .get(&result.key())
            .map(|fact| (fact.q, fact.theta, fact.deadline)))
    }

    fn verify_descriptor(&self, descriptor: &Descriptor) -> Result<(), BridgeError> {
        descriptor.validate()?;
        if descriptor.plan.config_ref != self.config_ref {
            return Err(BridgeError::SourceMismatch);
        }
        let program = ProgramConfig::from_plan(&self.binding.config, &descriptor.plan)?;
        if regional_cid(
            &descriptor.plan.session,
            descriptor.anchor.block.block_id,
            &program,
        )? != descriptor.cid
        {
            return Err(BridgeError::InvalidResult);
        }
        Ok(())
    }

    fn deadline(&self, descriptor: &Descriptor) -> Result<PublicTimeUs, BridgeError> {
        let next = descriptor
            .plan
            .window
            .h_max
            .checked_add(1)
            .ok_or(BridgeError::InvalidResult)?;
        let timestamp = l2_timestamp_us(&self.binding.config, next)?;
        timestamp
            .checked_add(self.binding.parameters.grace_us)
            .map(PublicTimeUs)
            .ok_or(BridgeError::InvalidResult)
    }

    fn record_identity(&self, position: &[u8], projection: &[u8]) -> Result<Hash32, BridgeError> {
        record_id(
            self.config_ref,
            &self.binding.espresso_source,
            &self.binding.l1_source,
            &self.binding.expected,
            position,
            projection,
        )
    }

    fn plan(&self, state: LedgerState) -> Result<StopPlan, BridgeError> {
        let mut out = Writer::new(MAX_SNAPSHOT);
        out.push(HEADER)?;
        out.frame(&self.encoded_binding)?;
        out.push(&state.revision.to_be_bytes())?;
        out.count(state.records.len())?;
        for (position, record) in &state.records {
            out.frame(position)?;
            out.push(record.id.as_bytes())?;
            write_parameters(&mut out, record.parameters)?;
            out.frame(&record.projection)?;
        }
        out.count(state.deadlines.len())?;
        for (key, fact) in &state.deadlines {
            out.frame(&key.encode())?;
            out.frame(&fact.descriptor.encode_checked()?)?;
            out.push(&fact.q.to_be_bytes())?;
            out.push(&fact.theta.0.to_be_bytes())?;
            out.push(&fact.deadline.0.to_be_bytes())?;
        }
        let bytes = out.bytes;
        Ok(StopPlan {
            instance: Arc::clone(&self.instance),
            base_revision: self.state.revision,
            state,
            hash: snapshot_hash(&bytes),
            bytes,
        })
    }
}

impl StopPlan {
    pub fn candidate_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn candidate_hash(&self) -> Hash32 {
        self.hash
    }
}

impl LedgerState {
    fn check_scopes(&self) -> Result<(), BridgeError> {
        if self.branches.len() + self.sessions.len() + self.selected.len() + self.deadlines.len()
            > MAX_SCOPES
        {
            return Err(BridgeError::Capacity);
        }
        Ok(())
    }

    fn apply_events(
        &mut self,
        ledger: &StopLedger,
        position: &[u8],
        record_id: Hash32,
        events: &[LifecycleEvent],
    ) -> Result<(), BridgeError> {
        for (event_ordinal, event) in events.iter().enumerate() {
            let basis = PublicBasis {
                position: position.to_vec(),
                event_ordinal,
                record_id,
                event_id: poc_viewer_gfr::lifecycle_id(event)?,
            };
            match &event.body {
                LifecycleBody::BranchClosed {
                    branch,
                    reason,
                } if matches!((event.position.kind, *reason),
                    (ReplayKind::Expire, CloseReason::Expired) | (ReplayKind::Force, CloseReason::Force)) => {
                    first_basis(
                        self.branches
                            .entry(*branch)
                            .or_insert_with(|| basis.clone()),
                        basis,
                    );
                }
                LifecycleBody::SessionClosed {
                    session,
                    reason: CloseReason::Complete | CloseReason::Time,
                } if matches!(
                    event.position.kind,
                    ReplayKind::SessionEnd | ReplayKind::Espresso
                ) =>
                {
                    first_basis(
                        self.sessions
                            .entry((session.beta, session.r))
                            .or_insert_with(|| basis.clone()),
                        basis,
                    );
                }
                LifecycleBody::SessionAssigned { descriptor }
                    if matches!(
                        event.position.kind,
                        ReplayKind::SessionEnd | ReplayKind::Espresso | ReplayKind::Force
                    ) =>
                {
                    ledger.verify_descriptor(descriptor)?;
                    let key = (descriptor.plan.session.beta, descriptor.plan.session.r);
                    let entry = self
                        .selected
                        .entry(key)
                        .or_insert_with(|| (descriptor.cid, basis.clone()));
                    if entry.0 != descriptor.cid {
                        return Err(BridgeError::InvalidResult);
                    }
                    first_basis(&mut entry.1, basis);
                }
                _ => return Err(BridgeError::InvalidResult),
            }
            self.check_scopes()?;
        }
        Ok(())
    }

    fn stop_for(&self, session: Session, cid: Cid) -> Option<(&PublicBasis, LifecycleReason)> {
        if let Some(basis) = self.branches.get(&session.0) {
            return Some((basis, LifecycleReason::BranchClosed));
        }
        if let Some(basis) = self.sessions.get(&session) {
            return Some((basis, LifecycleReason::SessionClosed));
        }
        self.selected
            .get(&session)
            .filter(|(selected, _)| *selected != cid)
            .map(|(_, basis)| (basis, LifecycleReason::OtherCidAssigned))
    }
}

fn first_basis(prior: &mut PublicBasis, candidate: PublicBasis) {
    if candidate < *prior {
        *prior = candidate;
    }
}

fn snapshot_hash(bytes: &[u8]) -> Hash32 {
    digest("H3/StopSnapshot/v1", &[bytes])
}

fn encode_binding(binding: &StopLedgerBinding) -> Result<Vec<u8>, BridgeError> {
    let VerificationProfile::Native {
        protocol_version,
        verifier_version,
        source_revision,
    } = &binding.expected
    else {
        return Err(BridgeError::NonNativeProfile);
    };
    binding.expected.validate(&binding.config)?;
    let p = binding.parameters;
    if p.announcement_us == 0 || p.grace_us == 0 || p.expiry_us == 0 {
        return Err(BridgeError::InvalidConfig);
    }
    let mut out = Writer::new(MAX_SNAPSHOT);
    out.frame(&binding.config.encode_checked()?)?;
    out.push(&protocol_version.to_be_bytes())?;
    out.push(&verifier_version.to_be_bytes())?;
    out.frame(source_revision.as_bytes())?;
    write_parameters(&mut out, p)?;
    for source in [
        &binding.espresso_source,
        &binding.l1_source,
        &binding.publisher_reader,
    ] {
        identity(source, MAX_SOURCE_BYTES)?;
        out.frame(source)?;
    }
    identity(&binding.owner_reference, 4096)?;
    out.frame(&binding.owner_reference)?;
    Ok(out.bytes)
}

fn write_parameters(out: &mut Writer, p: GfrParameters) -> Result<(), BridgeError> {
    for value in [p.announcement_us, p.grace_us, p.expiry_us] {
        out.push(&value.to_be_bytes())?;
    }
    Ok(())
}

fn read_parameters(input: &mut Reader<'_>) -> Result<GfrParameters, BridgeError> {
    Ok(GfrParameters {
        announcement_us: input.u64()?,
        grace_us: input.u64()?,
        expiry_us: input.u64()?,
    })
}
