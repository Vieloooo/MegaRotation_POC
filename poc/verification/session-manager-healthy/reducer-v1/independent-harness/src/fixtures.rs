//! Fixed hand-written inputs. Key bytes are deliberately non-cryptographic.
//! O3 encoding helpers construct inputs only; expected reducer outputs never
//! call the reducer or its private ID generator.

use poc_types::domain::*;
use poc_viewer_gfr::{GlobalSource, InputRef, ReplayEffect, ReplayRecord};

pub fn h(byte: u8) -> Hash32 {
    Hash32::new([byte; 32])
}

pub fn pid(value: u8) -> Pid {
    Pid::new(value).unwrap()
}

pub fn sid(beta: u64, r: u64) -> SessionId {
    SessionId { beta, r }
}

pub fn anchor(height: u64, identity: u8) -> Anchor {
    Anchor {
        block: BlockRef {
            height,
            origin: 3,
            block_id: h(identity),
        },
        timestamp_us: 10_000_000 + height * 1_000_000,
    }
}

pub fn config() -> PublicConfig {
    PublicConfig {
        chain_id: 42,
        rule_version: 1,
        verifier_version: 1,
        registry_hash: h(0x33),
        dep_fixture_hash: h(0x34),
        l1_fixture_hash: h(0x35),
        genesis: anchor(0, 0x11),
        genesis_state_root: h(0x36),
        genesis_native_hash: h(0x37),
        l2_slot_us: 1_000_000,
        l1_genesis_timestamp_us: 0,
        l1_block_us: 2_000_000,
        t_dep_us: 5_000_000,
        t_drift_us: 15_000_000,
        metadata_window_us: 200_000,
        byte_limit: 1024,
    }
}

fn plan(
    cfg: &PublicConfig,
    session: SessionId,
    cause: BoundaryCause,
    source_region: Option<RegionId>,
    window: Window,
    reference: BlockRef,
) -> PlanRecord {
    let region = RegionId::new(((session.beta + session.r) % 3) as u8).unwrap();
    let pids: Vec<Pid> = (0..5).map(|j| pid(5 * region.get() + j)).collect();
    let committee = pids
        .iter()
        .map(|p| CommitteeEntry {
            pid: *p,
            identity_key: [p.get() + 1; 32],
            vote_key: [p.get() + 0x80; 96],
            weight: 1,
        })
        .collect();
    PlanRecord {
        session,
        cause,
        source_region,
        region,
        committee,
        total_weight: 5,
        fault_bound: 1,
        eligible_proposers: pids,
        leader_rule: RuleSpec::static_schedule(),
        window,
        reference_block: reference,
        config_ref: cfg.reference().unwrap(),
        authorization_ref: h(0x44),
    }
}

pub fn descriptor(cfg: &PublicConfig, plan: PlanRecord, anchor: Anchor) -> Descriptor {
    let program = ProgramConfig::from_plan(cfg, &plan).unwrap();
    let cid = regional_cid(&plan.session, anchor.block.block_id, &program).unwrap();
    Descriptor { plan, anchor, cid }
}

pub fn genesis(cfg: &PublicConfig) -> Descriptor {
    let p = plan(
        cfg,
        sid(0, 0),
        BoundaryCause::Genesis,
        None,
        Window {
            h_min: 1,
            h_max: 10,
        },
        cfg.genesis.block.clone(),
    );
    descriptor(cfg, p, cfg.genesis.clone())
}

pub fn successor(cfg: &PublicConfig, predecessor: &Descriptor, a: Anchor) -> Descriptor {
    let p = plan(
        cfg,
        sid(
            predecessor.plan.session.beta,
            predecessor.plan.session.r + 1,
        ),
        BoundaryCause::Handoff,
        Some(predecessor.plan.region),
        Window {
            h_min: predecessor.plan.window.h_max + 1,
            h_max: predecessor.plan.window.h_max + 10,
        },
        predecessor.anchor.block.clone(),
    );
    descriptor(cfg, p, a)
}

pub fn espresso_position(key: u64, q: u64) -> ReplayPosition {
    ReplayPosition {
        key_us: key,
        kind: ReplayKind::Espresso,
        index: ReplayIndex::Espresso(q),
    }
}

pub fn event(position: &ReplayPosition, body: LifecycleBody) -> LifecycleEvent {
    LifecycleEvent {
        position: position.clone(),
        body,
    }
}

pub fn record(
    key: u64,
    q: u64,
    bodies: Vec<LifecycleBody>,
    effects: Vec<ReplayEffect>,
) -> ReplayRecord {
    let position = espresso_position(key, q);
    let events = bodies
        .into_iter()
        .map(|body| event(&position, body))
        .collect();
    ReplayRecord {
        position,
        input_ref: InputRef::Espresso(q),
        effects,
        events,
    }
}

pub fn session_end(key: u64, session: SessionId, bodies: Vec<LifecycleBody>) -> ReplayRecord {
    let position = ReplayPosition {
        key_us: key,
        kind: ReplayKind::SessionEnd,
        index: ReplayIndex::Session(session.clone()),
    };
    let events = bodies
        .into_iter()
        .map(|body| event(&position, body))
        .collect();
    ReplayRecord {
        position,
        input_ref: InputRef::SessionEnd(session),
        effects: vec![],
        events,
    }
}

pub fn expiry(key: u64, height: u64, branch: u64) -> ReplayRecord {
    let position = ReplayPosition {
        key_us: key,
        kind: ReplayKind::Expire,
        index: ReplayIndex::Height(height),
    };
    let close = event(
        &position,
        LifecycleBody::BranchClosed {
            branch,
            reason: CloseReason::Expired,
        },
    );
    ReplayRecord {
        position,
        input_ref: InputRef::Expire(height),
        effects: vec![],
        events: vec![close],
    }
}

pub fn append(first: u64, last: u64, predecessor: Cid) -> ReplayEffect {
    ReplayEffect::Append {
        first,
        last,
        source: GlobalSource::Regional(ResultKey {
            cid: predecessor,
            block_hash: h(0x55),
        }),
    }
}

pub fn assigned(d: &Descriptor) -> LifecycleBody {
    LifecycleBody::SessionAssigned {
        descriptor: d.clone(),
    }
}

pub fn closed(session: &SessionId) -> LifecycleBody {
    LifecycleBody::SessionClosed {
        session: session.clone(),
        reason: CloseReason::Complete,
    }
}
