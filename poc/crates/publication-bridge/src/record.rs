use poc_types::domain::{
    Anchor, Boundary, Canonical, CanonicalObject, CloseReason, ConfigRef, Descriptor, Hash32,
    L1Position, LifecycleBody, LifecycleEvent, ReplayIndex, ReplayKind, ReplayPosition, ResultKey, SessionId, VirtualBlock, Metadata,
};
use poc_viewer_gfr::{
    GfrParameters, GlobalSource, InputRef, ResumeBasis, ReplayEffect, ReplayRecord, ReplayState, Tail,
    VerificationProfile, encode_position, lifecycle_id, lifecycle_projection,
};

use crate::{
    BridgeError,
    bounded::{Reader, Writer, digest, identity},
};

pub(crate) const MAX_PROJECTION: usize = 1024 * 1024;
const MAX_MEMBERS: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterialReference {
    pub anchor_id: Hash32,
    pub object_hash: Hash32,
    pub reference: Vec<u8>,
}

/// A complete event captured from sealed replay state. Delivery still requires
/// the controlled runtime's committed root and acknowledgment.
#[derive(Clone, Debug)]
pub struct RecordDelivery {
    pub(crate) config: ConfigRef,
    pub(crate) espresso: Vec<u8>,
    pub(crate) l1: Vec<u8>,
    pub(crate) profile: VerificationProfile,
    pub(crate) parameters: GfrParameters,
    pub(crate) position: Vec<u8>,
    pub(crate) projection: Vec<u8>,
    id: Hash32,
    record: ReplayRecord,
    events: Vec<Hash32>,
    materials: Vec<MaterialReference>,
    resume_material: Option<ResumeBasis>,
}

impl RecordDelivery {
    pub fn from_replay(state: &ReplayState, ordinal: usize) -> Result<Self, BridgeError> {
        let public = state.public_state();
        if !matches!(public.verification, VerificationProfile::Native { .. }) {
            return Err(BridgeError::NonNativeProfile);
        }
        public.verification.validate(&public.config)?;
        identity(&public.espresso_source, sim_espresso::MAX_SOURCE_BYTES)?;
        identity(&public.l1_source, sim_espresso::MAX_SOURCE_BYTES)?;
        let record = public
            .records
            .get(ordinal)
            .ok_or(BridgeError::InvalidResult)?;
        let position = encode_position(&record.position)?;
        let projection = record_projection(record)?;
        let config = public.config.reference()?;
        let id = record_id(
            config,
            &public.espresso_source,
            &public.l1_source,
            &public.verification,
            &position,
            &projection,
        )?;
        Ok(Self {
            config,
            espresso: public.espresso_source.clone(),
            l1: public.l1_source.clone(),
            profile: public.verification.clone(),
            parameters: public.parameters,
            position,
            projection,
            id,
            record: record.clone(),
            events: record
                .events
                .iter()
                .map(lifecycle_id)
                .collect::<Result<_, _>>()?,
            materials: Vec::new(),
            resume_material: match &record.input_ref {
                InputRef::L1(control) if control.kind == ReplayKind::Resume => public.resumes.get(&control.position).cloned(),
                _ => None,
            },
        })
    }

    pub fn resume_material(&self) -> Option<&ResumeBasis> { self.resume_material.as_ref() }

    pub fn id(&self) -> Hash32 {
        self.id
    }
    pub fn record(&self) -> &ReplayRecord {
        &self.record
    }
    pub fn event_ids(&self) -> &[Hash32] {
        &self.events
    }
    pub fn parameters(&self) -> GfrParameters {
        self.parameters
    }

    pub fn with_material_references(
        mut self,
        references: Vec<MaterialReference>,
    ) -> Result<Self, BridgeError> {
        if references.len() > 32 || references.iter().any(|item| item.reference.len() > 4096) {
            return Err(BridgeError::Capacity);
        }
        self.materials = references;
        Ok(self)
    }

    pub fn material_references(&self) -> &[MaterialReference] {
        &self.materials
    }
}

pub(crate) fn record_id(
    config: ConfigRef,
    espresso: &[u8],
    l1: &[u8],
    profile: &VerificationProfile,
    position: &[u8],
    projection: &[u8],
) -> Result<Hash32, BridgeError> {
    let VerificationProfile::Native {
        protocol_version,
        verifier_version,
        source_revision,
    } = profile
    else {
        return Err(BridgeError::NonNativeProfile);
    };
    Ok(digest(
        "H3/Record/v1",
        &[
            &config.encode(),
            espresso,
            l1,
            &protocol_version.to_be_bytes(),
            &verifier_version.to_be_bytes(),
            source_revision.as_bytes(),
            position,
            projection,
        ],
    ))
}

fn record_projection(record: &ReplayRecord) -> Result<Vec<u8>, BridgeError> {
    if record.effects.len() > MAX_MEMBERS || record.events.len() > MAX_MEMBERS {
        return Err(BridgeError::Capacity);
    }
    let mut out = Writer::new(MAX_PROJECTION);
    let mut input = Writer::new(MAX_PROJECTION);
    match &record.input_ref {
        InputRef::Espresso(q) => {
            input.tag(0)?;
            input.push(&q.to_be_bytes())?;
        }
        InputRef::SessionEnd(session) => {
            input.tag(1)?;
            input.frame(&session.encode_checked()?)?;
        }
        InputRef::Expire(height) => {
            input.tag(2)?;
            input.push(&height.to_be_bytes())?;
        }
        InputRef::L1(control) => {
            let tag = if control.is_policy_resume() { 5 } else if control.is_policy_force() { 4 } else { 3 };
            input.tag(tag)?;
            input.frame(&control.position.encode())?;
            input.push(&control.control_seq.to_be_bytes())?;
            if tag != 3 {
                input.frame(&control.source_bytes)?;
            } else {
                let branch = control.force_branch().ok_or(BridgeError::InvalidResult)?;
                input.push(&branch.to_be_bytes())?;
            }
        }
    }
    check_input(&input.bytes, &record.position)?;
    out.push(&1u16.to_be_bytes())?;
    out.frame(&input.bytes)?;
    out.count(record.effects.len())?;
    for effect in &record.effects {
        let mut part = Writer::new(MAX_PROJECTION);
        match effect {
            ReplayEffect::Append {
                first,
                last,
                source: GlobalSource::Regional(key),
            } => {
                part.tag(0)?;
                part.push(&first.to_be_bytes())?;
                part.push(&last.to_be_bytes())?;
                part.tag(0)?;
                part.frame(&key.encode())?;
            }
            ReplayEffect::Duty { before, after } => {
                part.tag(1)?;
                part.frame(&tail_projection(before)?)?;
                part.frame(&tail_projection(after)?)?;
            }
            ReplayEffect::Reset { resume_position, target_height, cut_digest } => {
                part.tag(2)?;
                part.frame(&resume_position.encode())?;
                part.push(&target_height.to_be_bytes())?;
                part.push(cut_digest.as_bytes())?;
            }
            ReplayEffect::Append { .. } => return Err(BridgeError::InvalidResult),
        }
        out.frame(&part.bytes)?;
    }
    out.count(record.events.len())?;
    for event in &record.events {
        if event.position != record.position
        {
            return Err(BridgeError::InvalidResult);
        }
        out.frame(&lifecycle_projection(event)?)?;
    }
    Ok(out.bytes)
}

fn tail_projection(tail: &Tail) -> Result<Vec<u8>, BridgeError> {
    let mut out = Writer::new(MAX_PROJECTION);
    match tail {
        Tail::Next(descriptor) => {
            out.tag(0)?;
            out.frame(&descriptor.encode_checked()?)?;
        }
        Tail::Pending(boundary) => {
            out.tag(1)?;
            out.frame(&boundary.encode_checked()?)?;
        }
        Tail::Halt { boundary, anchor } => {
            out.tag(2)?;
            out.frame(&boundary.encode_checked()?)?;
            out.frame(&anchor.encode_checked()?)?;
        }
    }
    Ok(out.bytes)
}

fn check_input(bytes: &[u8], position: &ReplayPosition) -> Result<(), BridgeError> {
    let mut input = Reader(bytes);
    let matched = match (input.tag()?, position.kind, &position.index) {
        (0, ReplayKind::Espresso, ReplayIndex::Espresso(q)) => input.u64()? == *q,
        (1, ReplayKind::SessionEnd, ReplayIndex::Session(session)) => {
            SessionId::decode(input.frame(MAX_PROJECTION)?)? == *session
        }
        (2, ReplayKind::Expire, ReplayIndex::Height(height)) => input.u64()? == *height,
        (3, ReplayKind::Force, ReplayIndex::L1(at)) => {
            let encoded = L1Position::decode(input.frame(MAX_PROJECTION)?)?;
            input.u64()?; // control sequence
            input.u64()?; // target branch
            encoded == *at
        }
        (4, ReplayKind::Force, ReplayIndex::L1(at)) => {
            let encoded = L1Position::decode(input.frame(MAX_PROJECTION)?)?;
            let control_seq = input.u64()?;
            let source_bytes = input.frame(64)?.to_vec();
            let control = poc_viewer_gfr::ControlRef {
                position: encoded,
                kind: ReplayKind::Force,
                control_seq,
                source_bytes,
            };
            encoded == *at && control.regional_policy().is_some()
        }
        (5, ReplayKind::Resume, ReplayIndex::L1(at)) => {
            let encoded = L1Position::decode(input.frame(MAX_PROJECTION)?)?;
            let control = poc_viewer_gfr::ControlRef {position:encoded,kind:ReplayKind::Resume,
                control_seq:input.u64()?,source_bytes:input.frame(128)?.to_vec()};
            encoded == *at && control.resume_control().is_some()
        }
        _ => false,
    };
    if !matched {
        return Err(BridgeError::InvalidResult);
    }
    input.finish()
}

/// Restore LIFE members from the exact stored record projection. The
/// existing encoder confirms the version, framing and canonical nested bytes.
pub(crate) fn restored_events(
    projection: &[u8],
    position: &ReplayPosition,
) -> Result<Vec<LifecycleEvent>, BridgeError> {
    let mut row = Reader(projection);
    if row.fixed::<2>()? != 1u16.to_be_bytes() {
        return Err(BridgeError::InvalidEncoding);
    }
    check_input(row.frame(MAX_PROJECTION)?, position)?;
    for _ in 0..row.count(MAX_MEMBERS)? {
        check_effect(row.frame(MAX_PROJECTION)?)?;
    }
    let count = row.count(MAX_MEMBERS)?;
    let mut events = Vec::with_capacity(count);
    for _ in 0..count {
        let bytes = row.frame(MAX_PROJECTION)?;
        let mut event = Reader(bytes);
        if event.fixed::<2>()? != 1u16.to_be_bytes()
            || event.frame(35)? != encode_position(position)?
        {
            return Err(BridgeError::InvalidEncoding);
        }
        let body = match event.tag()? {
            0 => LifecycleBody::SessionAssigned {
                descriptor: Descriptor::decode(event.frame(MAX_PROJECTION)?)?,
            },
            1 => LifecycleBody::SessionClosed {
                session: SessionId {
                    beta: event.u64()?,
                    r: event.u64()?,
                },
                reason: reason(event.tag()?)?,
            },
            2 => LifecycleBody::BranchClosed {
                branch: event.u64()?,
                reason: reason(event.tag()?)?,
            },
            3 => {
                let control_seq=event.u64()?;
                let branch=event.u64()?;
                let anchor=Anchor::decode(event.frame(MAX_PROJECTION)?)?;
                let count=event.count(poc_types::domain::MAX_FIXTURE_ROWS)?;
                let mut derived_prefix=Vec::with_capacity(count);
                for _ in 0..count {
                    let mut block=Reader(event.frame(MAX_PROJECTION)?);
                    let metadata=Metadata::decode(block.frame(MAX_PROJECTION)?)?;
                    let mut dep=Vec::new();
                    for _ in 0..block.count(MAX_PROJECTION/4)? {dep.push(block.frame(MAX_PROJECTION)?.to_vec());}
                    let pre_state_root=Hash32::new(block.fixed()?);
                    let mut transactions=Vec::new();
                    for _ in 0..block.count(MAX_PROJECTION/4)? {transactions.push(block.frame(MAX_PROJECTION)?.to_vec());}
                    block.finish()?;
                    derived_prefix.push(VirtualBlock{metadata,dep,pre_state_root,transactions});
                }
                LifecycleBody::Resumed{control_seq,branch,anchor,derived_prefix}
            },
            _ => return Err(BridgeError::InvalidEncoding),
        };
        event.finish()?;
        let parsed = LifecycleEvent {
            position: position.clone(),
            body,
        };
        if lifecycle_projection(&parsed)? != bytes {
            return Err(BridgeError::InvalidEncoding);
        }
        events.push(parsed);
    }
    row.finish()?;
    Ok(events)
}

fn check_effect(bytes: &[u8]) -> Result<(), BridgeError> {
    let mut part = Reader(bytes);
    match part.tag()? {
        0 => {
            if part.u64()? > part.u64()? || part.tag()? != 0 {
                return Err(BridgeError::InvalidEncoding);
            }
            ResultKey::decode(part.frame(66)?)?;
        }
        1 => {
            check_tail(part.frame(MAX_PROJECTION)?)?;
            check_tail(part.frame(MAX_PROJECTION)?)?;
        }
        2 => {
            L1Position::decode(part.frame(MAX_PROJECTION)?)?;
            part.u64()?;
            part.fixed::<32>()?;
        }
        _ => return Err(BridgeError::InvalidEncoding),
    }
    part.finish()
}

fn check_tail(bytes: &[u8]) -> Result<(), BridgeError> {
    let mut part = Reader(bytes);
    match part.tag()? {
        0 => {
            Descriptor::decode(part.frame(MAX_PROJECTION)?)?;
        }
        1 => {
            Boundary::decode(part.frame(MAX_PROJECTION)?)?;
        }
        2 => {
            Boundary::decode(part.frame(MAX_PROJECTION)?)?;
            Anchor::decode(part.frame(MAX_PROJECTION)?)?;
        }
        _ => return Err(BridgeError::InvalidEncoding),
    }
    part.finish()
}

fn reason(tag: u8) -> Result<CloseReason, BridgeError> {
    match tag {
        0 => Ok(CloseReason::Time),
        1 => Ok(CloseReason::Complete),
        2 => Ok(CloseReason::Force),
        3 => Ok(CloseReason::Resume),
        4 => Ok(CloseReason::Expired),
        5 => Ok(CloseReason::Excluded),
        6 => Ok(CloseReason::Incompatible),
        _ => Err(BridgeError::InvalidEncoding),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_empty_record_projection_and_identity() {
        let position = ReplayPosition {
            key_us: 100,
            kind: ReplayKind::Espresso,
            index: ReplayIndex::Espresso(5),
        };
        let record = ReplayRecord {
            position: position.clone(),
            input_ref: InputRef::Espresso(5),
            effects: vec![],
            events: vec![],
        };
        let bytes = record_projection(&record).unwrap();
        assert_eq!(
            poc_types::hex(&bytes),
            "0001000000090000000000000000050000000000000000"
        );
        let profile = VerificationProfile::Native {
            protocol_version: 1,
            verifier_version: 1,
            source_revision: "2".repeat(40),
        };
        let id = record_id(
            ConfigRef::new(Hash32::new([0x11; 32])),
            b"E",
            b"L",
            &profile,
            &encode_position(&position).unwrap(),
            &bytes,
        )
        .unwrap();
        assert_eq!(
            poc_types::hex(id.as_bytes()),
            "9c663cdf791586280d1c28ac39990b87c1504bd996e98fbbefcacda95bbb1809"
        );
    }
}
