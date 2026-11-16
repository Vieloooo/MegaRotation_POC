use poc_core::metadata::l2_timestamp_us;
use poc_types::domain::*;
use poc_viewer_gfr::{GlobalSource, LifecycleBody, RegionalPolicy, ReplayState, ReplayStatus, Tail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sim_espresso::{FinalizedLogReader, PublicTimeUs, ReaderState};

use super::{M1Result, checked, l1, profile::{SECOND, SOURCE}, public::Public};

pub(super) const OUTAGE: u64 = 11_250_000;
pub(super) const HEARTBEAT: u64 = 250_000;
pub(super) const POLL: u64 = 100_000;
pub(super) const SIGN_US: u64 = 10_000;
pub(super) const BROADCAST_US: u64 = 10_000;
const STALL: u64 = 2 * SECOND;
const MAX_HEAD_AGE: u64 = SECOND;
const RECOVERY_BUDGET: u64 = 6_750_000;
pub(super) const THRESHOLD: usize = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Scenario {
    Outage,
    Healthy,
    ReaderLag,
    OrderingStopped,
    WatermarkFrozen,
}

impl Scenario {
    pub fn parse(s: &str) -> M1Result<Self> {
        match s {
            "outage" => Ok(Self::Outage),
            "healthy" => Ok(Self::Healthy),
            "reader-lag" => Ok(Self::ReaderLag),
            "ordering-stopped" => Ok(Self::OrderingStopped),
            "watermark-frozen" => Ok(Self::WatermarkFrozen),
            _ => Err(super::problem("unknown M3 scenario")),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Outage => "outage",
            Self::Healthy => "healthy",
            Self::ReaderLag => "reader-lag",
            Self::OrderingStopped => "ordering-stopped",
            Self::WatermarkFrozen => "watermark-frozen",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub(super) struct Request {
    pub request_id: u64,
    pub control_seq: u64,
    pub target_beta: u64,
    pub enabled_mask: u8,
    pub order: [u8; 3],
    pub signed_floor: u64,
    pub cid: Cid,
    pub head_height: u64,
}

#[derive(Clone, Copy)]
pub(super) struct Advertised {
    pub l1: Option<u64>,
    pub espresso: Option<(u64, PublicTimeUs)>,
}

fn espresso_json(tip: Option<(u64, PublicTimeUs)>) -> Value {
    tip.map_or(Value::Null, |(q, theta)| json!({"q":q, "theta_us":theta.0}))
}

#[derive(Clone, Copy, Serialize)]
struct ClockSample {
    as_of_us: u64,
    received_us: u64,
    offset_us: i64,
    lease_end_us: u64,
}

impl ClockSample {
    fn usable(self, now: u64) -> bool {
        self.as_of_us <= self.received_us && self.received_us <= now
            && now < self.lease_end_us && self.offset_us.unsigned_abs() <= 1_000
    }
}

pub(super) struct Signer {
    pub l1: l1::Reader,
    pub espresso: ReaderState,
    replay: ReplayState,
    previous: Option<(u64, PublicTimeUs)>,
    id: u8,
    clock: Option<ClockSample>,
}

impl Signer {
    pub fn new(id: u8, genesis: &ReplayState) -> M1Result<Self> {
        Ok(Self {
            l1: l1::Reader::regional(),
            espresso: checked(ReaderState::new(SOURCE.to_vec(), format!("M3/signer/{id}").into_bytes()))?,
            replay: genesis.clone(),
            previous: None,
            id,
            clock: None,
        })
    }

    pub fn sample_clock(&mut self, as_of_us: u64, received_us: u64) {
        let offset_us = i64::from(self.id) * 500 - 500;
        self.clock = Some(ClockSample {
            as_of_us, received_us, offset_us, lease_end_us: as_of_us + 180_000,
        });
    }

    pub fn poll(
        &mut self,
        public: &Public,
        advertised: Advertised,
        sample_us: u64,
        unresolved: bool,
        verify_motion: bool,
    ) -> M1Result<(Value, Option<Request>)> {
        let l1_tip = self.l1.finalized_tip().map(|row| row.height);
        let espresso_tip = self.espresso.tip();
        let prior = self.previous;
        let mut complete = l1_tip == advertised.l1 && espresso_tip == advertised.espresso;
        let clock_good = self.clock.is_some_and(|clock| clock.usable(sample_us));
        if let Some((q, theta)) = advertised.espresso
            && complete
        {
            let bound = ReplayPosition {
                key_us: theta.0,
                kind: ReplayKind::Espresso,
                index: ReplayIndex::Espresso(q),
            };
            let advanced = public.advance_reader(&self.replay, &self.espresso, &self.l1, &bound)?;
            if matches!(advanced.status, ReplayStatus::Reject(_) | ReplayStatus::UnsupportedControl { .. }) {
                return Err(super::problem(format!("governance reader {}: {:?}", self.id, advanced.status)));
            }
            self.replay = advanced.state;
            complete = advanced.status == ReplayStatus::Ready
                && self.replay.public_state().pc.as_ref() == Some(&bound);
            if complete && verify_motion {
                self.previous = espresso_tip;
            }
        } else {
            complete = false;
        }
        let state = self.replay.public_state();
        let head_height = state.entries.last().map_or(state.genesis.anchor.block.height,
            |row| row.block.state.metadata.height);
        let active = state.policies.get(&state.branch).map_or(RegionalPolicy::GENESIS, |row| row.policy);
        let mut detail = json!({
            "signer": self.id, "l1_advertised": advertised.l1, "l1_verified": l1_tip,
            "espresso_advertised": espresso_json(advertised.espresso),
            "espresso_verified": espresso_json(espresso_tip), "previous": espresso_json(prior),
            "replay_position": state.pc, "branch": state.branch, "head_height": head_height,
            "clock_good": clock_good, "clock_sample_us": sample_us, "clock_sample": self.clock, "eligible": false,
            "tail": match &state.tail { Tail::Next(_) => "NEXT", Tail::Pending(_) => "PENDING", Tail::Halt { .. } => "HALT" },
        });
        let Tail::Next(descriptor) = &state.tail else {
            detail["reason"] = json!("NO_OPEN_DUTY");
            return Ok((detail, None));
        };
        let start = l2_timestamp_us(&state.config, descriptor.plan.window.h_min)?;
        let stop = l2_timestamp_us(&state.config, descriptor.plan.window.h_max + 1)?;
        let assignment = state.records.iter().rev().find_map(|record| {
            record.events.iter().any(|event| {
                matches!(&event.body, LifecycleBody::SessionAssigned { descriptor: assigned } if assigned.cid == descriptor.cid)
            }).then_some(record.position.key_us)
        }).unwrap_or(0);
        let begin = start.max(assignment);
        let progress = state.entries.iter().rev().find_map(|row| {
            matches!(row.source,GlobalSource::Regional(key) if key.cid==descriptor.cid)
                .then_some(row.position.key_us)
        }).unwrap_or(begin);
        let observation = state.records.iter().rev().find(|record|
            matches!(record.position.kind, ReplayKind::Espresso)
        ).map_or(0, |record| record.position.key_us);
        let advancing = matches!((prior, espresso_tip), (Some((q0,t0)),Some((q1,t1))) if q1>q0 && t1>t0);
        let new_mask = active.enabled_mask & !(1 << descriptor.plan.region.get());
        let expiry = l2_timestamp_us(&state.config, head_height + 1)? + state.parameters.expiry_us;
        let inclusion = (sample_us + SIGN_US + BROADCAST_US) / l1::L1_SLOT_US * l1::L1_SLOT_US
            + l1::L1_SLOT_US;
        let timely = inclusion + state.parameters.announcement_us + RECOVERY_BUDGET < expiry;
        let replayed_seq = state.records.iter().filter_map(|record| {
            if let poc_viewer_gfr::InputRef::L1(control) = &record.input_ref {
                Some(control.control_seq)
            } else {
                None
            }
        }).max();
        let known_seq = self.l1.latest_control_seq();
        let reason = eligibility(complete, clock_good, verify_motion, advancing,
            sample_us, observation, begin, stop, progress, timely, unresolved || known_seq > replayed_seq);
        detail["reason"] = json!(reason);
        detail["cid"] = json!(descriptor.cid);
        detail["region"] = json!(descriptor.plan.region);
        detail["progress_us"] = json!(progress);
        detail["observation_us"] = json!(observation);
        detail["predicted_inclusion_us"] = json!(inclusion);
        detail["expiry_us"] = json!(expiry);
        detail["policy"] = json!({"enabled_mask":new_mask, "order":active.order, "signed_floor":state.l1_floor});
        if reason != "ELIGIBLE" {
            return Ok((detail, None));
        }
        let control_seq = known_seq.unwrap_or(0).checked_add(1)
            .ok_or_else(|| super::problem("control sequence exhausted"))?;
        detail["eligible"] = json!(true);
        Ok((detail, Some(Request {
            request_id: control_seq, control_seq, target_beta: state.branch, enabled_mask: new_mask,
            order: active.order, signed_floor: state.l1_floor, cid: descriptor.cid, head_height,
        })))
    }
}

#[allow(clippy::too_many_arguments)]
fn eligibility(
    complete: bool, clock_good: bool, verify_motion: bool, advancing: bool,
    sample: u64, observation: u64, begin: u64, stop: u64, progress: u64,
    timely: bool, unresolved: bool,
) -> &'static str {
    if !complete { return "INPUTS_NOT_READY"; }
    if !clock_good { return "CLOCK_UNSAFE"; }
    if (verify_motion && !advancing) || observation > sample || sample - observation > MAX_HEAD_AGE {
        return "ORDERING_NOT_ADVANCING";
    }
    if observation < begin || observation >= stop { return "OUTSIDE_OPEN_DUTY"; }
    if observation.saturating_sub(progress) < STALL { return "PROGRESS_WITHIN_THRESHOLD"; }
    if !timely { return "RECOVERY_BUDGET_EXCEEDED"; }
    if unresolved { return "CONTROL_IN_FLIGHT"; }
    "ELIGIBLE"
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        m1_smoke::{
            profile::{self, ReadyMaterial, Reference},
            public::initial_descriptor,
        },
        native_config::Resolved,
    };
    use commonware_codec::Encode;
    use commonware_consensus::{
        simplex::types::{Finalization, Finalize, Proposal},
        types::{Epoch, Round, View},
    };
    use commonware_cryptography::sha256::Digest;
    use commonware_parallel::Sequential;
    use commonware_utils::iter::NonEmpty;
    use poc_authorization::PlanOutcome;
    use poc_publication_bridge::publication_messages;
    use sim_espresso::{
        BlockHeader, FinalizedBlock, LogMessage, SimEspresso, TickUs, TimestampPolicy,
    };
    use std::{fs, path::PathBuf};

    struct Fixture {
        root: PathBuf,
        public: Public,
        signer: Signer,
        ordering: SimEspresso,
    }

    impl Fixture {
        fn new(name: &str, l1_height: u64, disable_regions: bool) -> Self {
            let root =
                std::env::temp_dir().join(format!("poc-m3-signer-{}-{name}", std::process::id()));
            fs::create_dir(&root).unwrap();
            let public = Public::new(&root, true, true).unwrap();
            let mut signer = Signer::new(0, &public.replay).unwrap();
            for row in public.inputs.l1.rows.iter().take(l1_height as usize + 1) {
                let header = &row.header;
                let events = if disable_regions && header.height == 4 {
                    json!([{
                        "event": "force",
                        "branch": 0,
                        "seq": 1,
                        "act": header.timestamp_s + 10,
                        "position": {"block": 4, "tx": 0, "log": 0},
                        "policy": {"enabled_mask": 0, "order": [0, 1, 2], "l1_floor": 3},
                    }])
                } else {
                    json!([])
                };
                let block = json!({
                    "number": header.height,
                    "hash": format!("0x{}", poc_types::hex(header.hash.as_bytes())),
                    "timestamp": header.timestamp_s,
                    "header_and_transactions": {
                        "parentHash": format!("0x{}", poc_types::hex(header.parent_hash.as_bytes()))
                    },
                    "events": events,
                });
                signer.l1.receive(&block, &public.inputs).unwrap();
            }
            let ordering = SimEspresso::new(
                SOURCE.to_vec(),
                TimestampPolicy::Offset {
                    offset_us: 0,
                    quantum_us: 1,
                },
            )
            .unwrap();
            Self {
                root,
                public,
                signer,
                ordering,
            }
        }

        fn finalize(&mut self, tick: u64, messages: Vec<LogMessage>) -> FinalizedBlock {
            let q = self
                .ordering
                .propose(TickUs(tick), true, messages)
                .unwrap()
                .q;
            self.ordering.finalize(q, TickUs(tick + 1)).unwrap().clone()
        }

        fn deliver(&mut self, block: &FinalizedBlock) {
            self.signer
                .espresso
                .receive_block(SOURCE, block.clone(), block.finalized_at_us)
                .unwrap();
        }

        fn poll(&mut self, block: &FinalizedBlock) -> (Value, Option<Request>) {
            let now = block.finalized_at_us.0 + profile::CLOCK_TRANSIT_US;
            self.signer.sample_clock(block.finalized_at_us.0, now);
            self.signer
                .poll(
                    &self.public,
                    Advertised {
                        l1: self.signer.l1.finalized_tip().map(|header| header.height),
                        espresso: Some((block.proposal.q, block.proposal.theta_us)),
                    },
                    now,
                    false,
                    true,
                )
                .unwrap()
        }

        fn publication(
            &self,
            descriptor: &Descriptor,
            chain: &[PhysicalBlock],
            proof: Vec<u8>,
        ) -> Vec<LogMessage> {
            let result = self
                .public
                .result(descriptor.clone(), chain.to_vec(), proof)
                .unwrap();
            publication_messages(&result, SOURCE).unwrap()
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn certify(public: &Public, descriptor: &Descriptor, block: &PhysicalBlock) -> Vec<u8> {
        let resolved = Resolved::normal(&public.finality.0, descriptor).unwrap();
        let height = block.metadata.height;
        let proposal = Proposal::new(
            Round::new(Epoch::new(1), View::new(height)),
            View::new(height - 1),
            Digest(*block.digest().unwrap().as_bytes()),
        );
        let mut signers = (0..4).map(|member| {
            let pid = Pid::new(descriptor.plan.region.get() * 5 + member).unwrap();
            resolved
                .signer(u32::from(member), profile::fixture_key(pid))
                .unwrap()
        });
        let first = signers.next().unwrap();
        let votes = NonEmpty::new(
            Finalize::sign(&first, proposal.clone()).unwrap(),
            signers.map(|signer| Finalize::sign(&signer, proposal.clone()).unwrap()),
        );
        let proof = Finalization::from_owned_finalizes(&first, votes, &Sequential)
            .unwrap()
            .encode()
            .to_vec();
        assert!(public.finality.check(descriptor, block, &proof));
        proof
    }

    #[test]
    fn signer_advances_progress_only_for_accepted_results() {
        let mut fixture = Fixture::new("progress", 7, false);
        let descriptor = initial_descriptor(&fixture.public.replay).unwrap().clone();
        let install = profile::Install {
            descriptor: descriptor.clone(),
            execution: fixture.public.inputs.genesis.clone(),
            history: vec![],
            prepared_at_us: 13 * SECOND,
            projected_l1_lag_us: None,
            plan_basis: None,
            resume_cut: None,
            resume_cut_digest: None,
        };
        let material = ReadyMaterial::prepare(
            fixture.public.inputs.clone(),
            &install,
            Pid::new(0).unwrap(),
        )
        .unwrap();
        let mut chain = Vec::new();
        for height in 1..=3 {
            let parent = chain
                .last()
                .map_or(descriptor.anchor.block.block_id, |block: &PhysicalBlock| {
                    block.digest().unwrap()
                });
            let InputStatus::Ready(block) = material.build(&chain, parent, height) else {
                panic!("fixture build did not produce h{height}");
            };
            chain.push(block);
        }
        let first_proof = certify(&fixture.public, &descriptor, &chain[0]);
        let first = fixture.publication(&descriptor, &chain[..1], first_proof.clone());
        let valid = fixture.finalize(11_073_000, first.clone());
        fixture.deliver(&valid);
        let (detail, request) = fixture.poll(&valid);
        assert_eq!(detail["head_height"], 1);
        assert_eq!(detail["progress_us"], 11_073_000);
        assert!(request.is_none());

        let mut input = descriptor.plan.input();
        input.session.r += 1;
        let plan = fixture
            .public
            .finality
            .0
            .resolve(&input, &Reference(&descriptor.anchor));
        let PlanOutcome::Ready(plan) = plan else {
            panic!("fixture alternate region");
        };
        let unselected = fixture
            .public
            .finality
            .0
            .descriptor(&plan, &descriptor.anchor)
            .unwrap();
        let other_proof = certify(&fixture.public, &unselected, &chain[1]);
        let other = fixture.publication(&unselected, &chain[..2], other_proof);

        // The certificate is genuine, but its proposal binds h1, not the h2 target.
        assert!(
            !fixture
                .public
                .finality
                .check(&descriptor, &chain[1], &first_proof)
        );
        let invalid = fixture.publication(&descriptor, &chain[..2], first_proof);
        let third_proof = certify(&fixture.public, &descriptor, &chain[2]);
        let proof_only = fixture
            .publication(&descriptor, &chain, third_proof)
            .pop()
            .expect("publication ends in FinalProof");
        for (tick, messages) in [
            (11_250_000, first),
            (12_050_000, invalid),
            (12_100_000, other),
            (13_000_000, vec![proof_only]),
        ] {
            let block = fixture.finalize(tick, messages);
            fixture.deliver(&block);
            let (detail, request) = fixture.poll(&block);
            assert_eq!(detail["head_height"], 1);
            assert_eq!(detail["progress_us"], 11_073_000);
            assert_eq!(detail["observation_us"], tick);
            assert_eq!(detail["reason"], "PROGRESS_WITHIN_THRESHOLD");
            assert!(request.is_none());
        }

        let second_proof = certify(&fixture.public, &descriptor, &chain[1]);
        let second = fixture.publication(&descriptor, &chain[..2], second_proof);
        let block = fixture.finalize(13_075_000, second);
        let received = block.finalized_at_us;
        let q = block.proposal.q;
        fixture
            .signer
            .espresso
            .receive_header(SOURCE, BlockHeader::from(&block), received)
            .unwrap();
        let (proof, bodies) = block.proposal.messages.split_last().unwrap();
        for (index, body) in bodies.iter().enumerate() {
            fixture
                .signer
                .espresso
                .receive_message(SOURCE, q, index as u32, body.clone(), received)
                .unwrap();
        }
        let (wait, request) = fixture.poll(&block);
        assert_eq!(wait["reason"], "INPUTS_NOT_READY");
        assert_eq!(wait["head_height"], 1);
        assert_eq!(wait["progress_us"], 11_073_000);
        assert_eq!(wait["observation_us"], 13_000_000);
        assert!(request.is_none());

        fixture
            .signer
            .espresso
            .receive_message(SOURCE, q, bodies.len() as u32, proof.clone(), received)
            .unwrap();
        let (ready, request) = fixture.poll(&block);
        assert_eq!(ready["head_height"], 2);
        assert_eq!(ready["progress_us"], 13_075_000);
        assert_eq!(ready["reason"], "PROGRESS_WITHIN_THRESHOLD");
        assert!(request.is_none());
    }

    #[test]
    fn signer_cannot_vote_in_pending_or_halted_replay() {
        for (name, l1_height, disable, timestamp, tail) in [
            ("pending", 4, true, 18 * SECOND, "PENDING"),
            ("halted", 16, false, 41 * SECOND, "HALT"),
        ] {
            let mut fixture = Fixture::new(name, l1_height, disable);
            let block = fixture.finalize(timestamp, vec![]);
            fixture.deliver(&block);
            let (detail, request) = fixture.poll(&block);
            assert_eq!(detail["tail"], tail);
            assert_eq!(detail["reason"], "NO_OPEN_DUTY");
            assert!(request.is_none());
        }
    }

    #[test]
    fn threshold_and_preconditions() {
        let verdict = |observation, complete, moving, clock, timely, pending| {
            eligibility(
                complete,
                clock,
                true,
                moving,
                observation,
                observation,
                SECOND,
                10 * SECOND,
                SECOND,
                timely,
                pending,
            )
        };
        assert_eq!(
            verdict(3 * SECOND - 1, true, true, true, true, false),
            "PROGRESS_WITHIN_THRESHOLD"
        );
        assert_eq!(
            verdict(3 * SECOND, true, true, true, true, false),
            "ELIGIBLE"
        );
        assert_eq!(
            verdict(3 * SECOND, false, true, true, true, false),
            "INPUTS_NOT_READY"
        );
        assert_eq!(
            verdict(3 * SECOND, true, false, true, true, false),
            "ORDERING_NOT_ADVANCING"
        );
        assert_eq!(
            verdict(3 * SECOND, true, true, false, true, false),
            "CLOCK_UNSAFE"
        );
        assert_eq!(
            verdict(3 * SECOND, true, true, true, false, false),
            "RECOVERY_BUDGET_EXCEEDED"
        );
        assert_eq!(
            verdict(3 * SECOND, true, true, true, true, true),
            "CONTROL_IN_FLIGHT"
        );
    }
}
