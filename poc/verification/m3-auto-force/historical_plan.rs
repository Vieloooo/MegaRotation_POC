//! Independent historical Plan assertions for the bounded M3 evidence replay.

use poc_authorization::{StaticScheduleProvider, ValidatedRegistry};
use poc_types::domain::{
    Descriptor, Hash32, ProgramConfig, RuleSpec, authorization_ref, regional_cid,
};
use serde::Deserialize;
use serde_json::{Value, json};

use super::framed_hash;

#[derive(Clone, Copy, Debug, Deserialize)]
pub(super) struct Policy {
    enabled_mask: u8,
    order: [u8; 3],
}

impl Policy {
    pub(super) fn select(self, source: Option<u64>) -> Option<u64> {
        assert!(self.enabled_mask < 8, "unsupported region mask");
        let mut regions = self.order;
        regions.sort_unstable();
        assert_eq!(regions, [0, 1, 2], "invalid region cycle");
        let first = source.map_or(0, |region| {
            self.order
                .iter()
                .position(|candidate| u64::from(*candidate) == region)
                .expect("source outside the region cycle")
                + 1
        });
        (first..first + self.order.len())
            .map(|index| self.order[index % self.order.len()])
            .find(|region| self.enabled_mask & (1 << region) != 0)
            .map(u64::from)
    }

    fn bytes(self) -> [u8; 4] {
        [
            self.enabled_mask,
            self.order[0],
            self.order[1],
            self.order[2],
        ]
    }
}

#[derive(Clone, Copy)]
pub(super) struct ActivePolicy {
    pub(super) policy: Policy,
    pub(super) position: [u64; 3],
    pub(super) control_seq: u64,
    pub(super) signed_floor: u64,
    pub(super) effective_floor: u64,
}

#[derive(Clone, Copy, Deserialize)]
pub(super) struct Header {
    pub(super) height: u64,
    pub(super) timestamp_s: u64,
    pub(super) hash: [u8; 32],
}

impl ActivePolicy {
    pub(super) fn reference_header(&self, origin: u64, headers: &[Header]) -> Header {
        let height = origin.max(self.effective_floor);
        let header = headers
            .get(usize::try_from(height).expect("historical height range"))
            .expect("missing actual verified historical header");
        assert_eq!(header.height, height);
        *header
    }

    pub(super) fn check_plan(
        &self,
        descriptor: &Descriptor,
        authorization: &StaticScheduleProvider,
        headers: &[Header],
    ) {
        let plan = &descriptor.plan;
        let header = self.reference_header(plan.reference_block.origin, headers);
        let selection = authorization_ref(
            plan.config_ref,
            authorization.registry().digest(),
            &plan.input(),
        )
        .unwrap();
        let tx = u32::try_from(self.position[1]).expect("L1 tx index range");
        let log = u32::try_from(self.position[2]).expect("L1 log index range");
        let digest = framed_hash(&[
            b"PoC/HistoricalPlan/v1",
            selection.as_bytes(),
            &self.position[0].to_be_bytes(),
            &tx.to_be_bytes(),
            &log.to_be_bytes(),
            &self.control_seq.to_be_bytes(),
            &self.policy.bytes(),
            &self.signed_floor.to_be_bytes(),
            &self.effective_floor.to_be_bytes(),
            &header.height.to_be_bytes(),
            &header.hash,
            &[plan.region.get()],
        ]);
        assert_eq!(
            plan.authorization_ref,
            Hash32::new(digest),
            "historical Plan authorization"
        );
        assert_eq!(
            plan.leader_rule,
            RuleSpec {
                rule_id: 2,
                version: 1,
                params: digest.to_vec()
            },
            "leader mode and historical Plan binding"
        );
    }

    pub(super) fn projection(&self, origin: u64, headers: &[Header]) -> Value {
        let header = self.reference_header(origin, headers);
        json!({
            "enabled_mask": self.policy.enabled_mask,
            "order": self.policy.order,
            "historical_height": header.height,
            "historical_hash": header.hash,
            "signed_l1_floor": self.signed_floor,
        })
    }
}

pub(super) fn check_descriptor_base(
    descriptor: &Descriptor,
    authorization: &StaticScheduleProvider,
) {
    let plan = &descriptor.plan;
    let config = authorization.config();
    let registry: &ValidatedRegistry = authorization.registry();
    let program = ProgramConfig::from_plan(config, plan).expect("valid plan configuration");
    assert_eq!(plan.committee, registry.committee(plan.region));
    assert_eq!(
        plan.eligible_proposers,
        plan.committee
            .iter()
            .map(|entry| entry.pid)
            .collect::<Vec<_>>(),
        "all registered regional proposers eligible in M3"
    );
    assert_eq!((plan.total_weight, plan.fault_bound), (5, 1));
    assert!(descriptor.anchor.block.height >= config.genesis.block.height);
    assert!(descriptor.anchor.block.height < plan.window.h_min);
    assert_eq!(
        descriptor.cid,
        regional_cid(&plan.session, descriptor.anchor.block.block_id, &program).unwrap(),
        "canonical historical Plan cid"
    );
}
