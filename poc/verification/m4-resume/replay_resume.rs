//! Bounded M4 Resume projection using cuts independently checked from receipts and DA.

use std::collections::BTreeMap;

use poc_types::domain::{BoundaryCause, ExecutionAnchor};
use serde::Deserialize;
use serde_json::{Value, json};

use super::{ActivePolicy, Chain, Duty, Header, Policy, PublicConfig, hex, timestamp};

#[derive(Deserialize)]
pub(super) struct ResumeControl {
    pub(super) position: [u64; 3],
    pub(super) seq: u64,
    pub(super) act_us: u64,
    target: u64,
    signed_floor: u64,
    policy: Policy,
    cut_digest: String,
    anchor: ExecutionAnchor,
    prefix: Vec<Derived>,
}

#[derive(Deserialize)]
struct Derived {
    height: u64,
    native: String,
    state_root: String,
    origin: u64,
    kept: usize,
}

pub(super) struct ResumeReplay {
    controls: BTreeMap<[u64; 3], ResumeControl>,
}

impl ResumeReplay {
    pub(super) fn from_controls(controls: Vec<ResumeControl>) -> Self {
        let mut by_position = BTreeMap::new();
        let mut previous = None;
        for control in controls {
            assert!(previous.is_none_or(|(position, act, seq)| {
                control.position > position && control.act_us >= act && control.seq > seq
            }));
            previous = Some((control.position, control.act_us, control.seq));
            assert!(by_position.insert(control.position, control).is_none());
        }
        Self { controls: by_position }
    }

    pub(super) fn next(&self) -> Option<(u64, [u64; 3])> {
        self.controls
            .first_key_value()
            .map(|(&position, row)| (row.act_us, position))
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn apply(
        &mut self,
        global: &mut Chain,
        duty: &mut Duty,
        expected: &mut Vec<Value>,
        config: &PublicConfig,
        headers: &[Header],
        q: u64,
        announcement: u64,
        grace: u64,
        expiry: u64,
    ) -> (Value, bool, u64) {
        let (_, control) = self.controls.pop_first().expect("next Resume");
        let including = &headers[usize::try_from(control.position[0]).unwrap()];
        assert_eq!(including.height, control.position[0]);
        assert_eq!(
            including.timestamp_s.checked_mul(1_000_000).unwrap() + announcement,
            control.act_us
        );
        let old_head = global.head.block.height;
        let prefix_head = control.prefix.last().expect("non-genesis Resume prefix").height;
        let target = usize::try_from(control.target).unwrap();
        let anchor = control.anchor;
        anchor.validate().expect("independently checked cut target metadata");
        assert_eq!(anchor.anchor.block.height, control.target);
        assert_eq!(anchor.anchor.timestamp_us, timestamp(config, control.target));
        assert_eq!(control.prefix[target - 1].native, hex(anchor.native_hash.as_bytes()));
        assert_eq!(
            control.prefix[target - 1].state_root,
            hex(anchor.state.root().unwrap().as_bytes())
        );
        let common = expected
            .iter()
            .zip(control.prefix.iter().take(target))
            .take_while(|(old, new)| old["native"] == new.native)
            .count();
        assert_eq!(expected.len(), usize::try_from(old_head).unwrap());
        let replaced = expected.split_off(common);
        for (offset, entry) in control.prefix.into_iter().take(target).enumerate().skip(common) {
            assert_eq!(entry.height, (offset + 1) as u64);
            expected.push(json!({
                "height": entry.height,
                "native": entry.native,
                "state_root": entry.state_root,
                "origin": entry.origin,
                "kept": entry.kept,
                "source": "DER",
                "position": control.position,
            }));
        }
        let floor = anchor.anchor.block.origin;
        let active = ActivePolicy {
            policy: control.policy,
            position: control.position,
            control_seq: control.seq,
            signed_floor: control.signed_floor,
            effective_floor: floor,
        };
        let old_beta = duty.beta;
        let beta = old_beta.checked_add(1).unwrap();
        let min = control.target + 1;
        let max = control.target + 10;
        let stopped = timestamp(config, min) + expiry < control.act_us
            || timestamp(config, max + 1) + grace < control.act_us;
        let region = (!stopped).then(|| {
            control.policy.select(None).expect("bounded M4 evidence uses an enabled region")
        });
        let projection = (!stopped).then(|| active.projection(floor, headers));
        *duty = Duty {
            beta,
            r: 0,
            cause: BoundaryCause::Resume,
            region: region.unwrap_or(0),
            source_region: None,
            min,
            max,
            reference: anchor.anchor.block.clone(),
            anchor: anchor.anchor.clone(),
            cid: None,
            active_policy: Some(active),
        };
        *global = Chain {
            head: anchor.anchor,
            state: anchor.state,
            native: anchor.native_hash,
        };
        (json!({
            "seq": control.seq,
            "position": control.position,
            "act_us": control.act_us,
            "q_star": q,
            "cut_digest": control.cut_digest,
            "closed_beta": old_beta,
            "old_head": old_head,
            "old_suffix": replaced,
            "common_height": common,
            "target": control.target,
            "prefix_head": prefix_head,
            "anchor": global.head,
            "anchor_root": global.state.root().unwrap(),
            "native_hash": global.native,
            "beta": beta,
            "r": 0,
            "region": region,
            "h_min": min,
            "h_max": max,
            "floor": floor,
            "stopped": stopped,
            "historical_plan": projection,
            "latest_ready_us": timestamp(config, min) - 1000,
        }), stopped, floor)
    }
}
