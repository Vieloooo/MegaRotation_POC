//! Independent production-BE codec vectors, fixed before the M2 implementation.
//! The committee bytes here exercise codec binding only, not crypto validation.
use poc_types::domain::*;
use serde_json::Value;
use std::{collections::BTreeMap, fmt::Debug};

fn inputs() -> Value {
    serde_json::from_str(include_str!("m2-inputs.json")).unwrap()
}
fn goldens() -> Value {
    serde_json::from_str(include_str!("m2-golden-vectors.json")).unwrap()
}
fn unhex(s: &str) -> Vec<u8> {
    assert_eq!(s.len() % 2, 0);
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn bytes(name: &str) -> Vec<u8> {
    let g = goldens();
    unhex(
        g["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["id"] == name)
            .unwrap()["hex"]
            .as_str()
            .unwrap(),
    )
}
fn expected_hash(name: &str) -> Hash32 {
    let g = goldens();
    hash(
        &g["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["id"] == name)
            .unwrap()["expected_hash"],
    )
}
fn hash(value: &Value) -> Hash32 {
    Hash32::new(unhex(value.as_str().unwrap()).try_into().unwrap())
}
fn meta(value: &Value) -> Metadata {
    Metadata {
        height: value[0].as_u64().unwrap(),
        timestamp_us: value[1].as_u64().unwrap(),
        origin: value[2].as_u64().unwrap(),
    }
}
fn state(metadata: &Value, rows: &Value) -> ExecutionState {
    let accounts: BTreeMap<_, _> = rows
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r[0].as_u64().unwrap() as u8,
                Account {
                    nonce: r[1].as_u64().unwrap(),
                    balance: r[2].as_u64().unwrap(),
                },
            )
        })
        .collect();
    ExecutionState {
        metadata: meta(metadata),
        accounts,
    }
}
fn anchor(value: &Value) -> Anchor {
    Anchor {
        block: BlockRef {
            height: value["height"].as_u64().unwrap(),
            origin: value["origin"].as_u64().unwrap(),
            block_id: hash(&value["block_id"]),
        },
        timestamp_us: value["timestamp_us"].as_u64().unwrap(),
    }
}
fn config(value: &Value) -> PublicConfig {
    PublicConfig {
        chain_id: value["chain_id"].as_u64().unwrap(),
        rule_version: value["rule_version"].as_u64().unwrap() as u16,
        verifier_version: value["verifier_version"].as_u64().unwrap() as u16,
        registry_hash: hash(&value["registry_hash"]),
        dep_fixture_hash: hash(&value["dep_fixture_hash"]),
        l1_fixture_hash: hash(&value["l1_fixture_hash"]),
        genesis: anchor(&value["genesis"]),
        genesis_state_root: hash(&value["genesis_state_root"]),
        genesis_native_hash: hash(&value["genesis_native_hash"]),
        l2_slot_us: value["l2_slot_us"].as_u64().unwrap(),
        l1_genesis_timestamp_us: value["l1_genesis_timestamp_us"].as_u64().unwrap(),
        l1_block_us: value["l1_block_us"].as_u64().unwrap(),
        t_dep_us: value["t_dep_us"].as_u64().unwrap(),
        t_drift_us: value["t_drift_us"].as_u64().unwrap(),
        metadata_window_us: value["metadata_window_us"].as_u64().unwrap(),
        byte_limit: value["byte_limit"].as_u64().unwrap() as u32,
    }
}
fn check<T: CanonicalObject + PartialEq + Debug>(name: &str, actual: &T) {
    let encoded = bytes(name);
    assert_eq!(actual.encode_checked().unwrap(), encoded, "{name}");
    assert_eq!(T::decode(&encoded).unwrap(), *actual, "{name}");
    let mut tail = encoded.clone();
    tail.push(0);
    assert!(T::decode(&tail).is_err(), "tail {name}");
    assert!(
        T::decode(&encoded[..encoded.len() - 1]).is_err(),
        "truncated {name}"
    );
    let mut version = encoded;
    version[1] = 2;
    assert!(T::decode(&version).is_err(), "version {name}");
}
fn transfer(value: &Value) -> Transfer {
    Transfer {
        from: value[0].as_u64().unwrap() as u8,
        to: value[1].as_u64().unwrap() as u8,
        nonce: value[2].as_u64().unwrap(),
        amount: value[3].as_u64().unwrap(),
    }
}
fn credit(value: &Value) -> Credit {
    Credit {
        to: value[0].as_u64().unwrap() as u8,
        amount: value[1].as_u64().unwrap(),
    }
}

#[test]
fn execution_state_physical_native_and_raw_bytes_match_frozen_goldens() {
    let input = inputs();
    let initial = state(&input["anchor"]["metadata"], &input["anchor"]["accounts"]);
    check("state-h0", &initial);
    assert_eq!(initial.root().unwrap(), expected_hash("state-h0"));
    let mut pre = initial.root().unwrap();
    let mut native_parent = hash(&input["anchor"]["native_hash"]);
    for step in input["steps"].as_array().unwrap() {
        let metadata = meta(&step["metadata"]);
        let h = metadata.height;
        check(&format!("metadata-h{h}"), &metadata);
        let attr = Attributes {
            metadata: metadata.clone(),
        };
        assert_eq!(attr.encode_raw(), bytes(&format!("dep-h{h}-0")));
        assert_eq!(Attributes::decode_raw(&attr.encode_raw()).unwrap(), attr);
        let mut dep = vec![attr.encode_raw()];
        for (index, row) in step["credits"].as_array().unwrap().iter().enumerate() {
            let value = credit(row);
            assert_eq!(
                value.encode_raw(),
                bytes(&format!("dep-h{h}-{}", index + 1))
            );
            assert_eq!(Credit::decode_raw(&value.encode_raw()).unwrap(), value);
            dep.push(value.encode_raw());
        }
        let txs: Vec<_> = step["transactions"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let value = transfer(row);
                let raw = value.encode_raw();
                assert_eq!(raw, bytes(&format!("transfer-{index}")));
                assert_eq!(Transfer::decode_raw(&raw).unwrap(), value);
                raw
            })
            .collect();
        let after = state(&step["metadata"], &step["expected_accounts"]);
        check(&format!("state-h{h}"), &after);
        assert_eq!(after.root().unwrap(), expected_hash(&format!("state-h{h}")));
        let kept: Vec<_> = step["expected_kept_indices"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| txs[i.as_u64().unwrap() as usize].clone())
            .collect();
        let native = NativeBlock {
            parent: native_parent,
            metadata: metadata.clone(),
            dep,
            kept: kept.clone(),
            post_state_root: after.root().unwrap(),
        };
        check(&format!("native-h{h}"), &native);
        assert_eq!(
            native.digest().unwrap(),
            expected_hash(&format!("native-h{h}"))
        );
        if h == 3 {
            let physical = PhysicalBlock {
                parent: hash(&input["anchor"]["physical_id"]),
                metadata,
                pre_state_root: pre,
                transactions: txs,
            };
            check("physical-h3", &physical);
            assert_eq!(physical.digest().unwrap(), expected_hash("physical-h3"));
            let filtered = PhysicalBlock {
                transactions: kept,
                ..physical.clone()
            };
            check("physical-h3-only-kept", &filtered);
            assert_ne!(physical.digest().unwrap(), filtered.digest().unwrap());
            assert_eq!(pre, expected_hash("state-h2"));
            assert_ne!(pre, after.root().unwrap());
            assert_ne!(physical.parent, native.parent);
        }
        pre = after.root().unwrap();
        native_parent = native.digest().unwrap();
    }
    assert_eq!(
        derived_anchor_id(3, native_parent),
        expected_hash("derived-anchor-h3")
    );
    let execution_anchor = ExecutionAnchor {
        anchor: anchor(&input["public_config"]["genesis"]),
        state: initial,
        native_hash: hash(&input["anchor"]["native_hash"]),
    };
    check("execution-anchor", &execution_anchor);
    let mut bad = execution_anchor.clone();
    bad.state.metadata.height = 1;
    assert!(bad.encode_checked().is_err());
}

#[test]
fn fixtures_and_public_config_bind_exact_independent_bytes() {
    let input = inputs();
    let fixture = DepFixture {
        first_height: input["dep_fixture"]["first_height"].as_u64().unwrap(),
        rows: input["dep_fixture"]["credits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row.as_array().unwrap().iter().map(credit).collect())
            .collect(),
    };
    check("dep-fixture", &fixture);
    assert_eq!(fixture.digest().unwrap(), expected_hash("dep-fixture"));
    assert_eq!(empty_control_commitment(), expected_hash("empty-control"));
    let l1 = L1Fixture {
        rows: input["l1_rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| L1FixtureRow {
                header: L1Header {
                    height: r["height"].as_u64().unwrap(),
                    hash: hash(&r["hash"]),
                    parent_hash: hash(&r["parent_hash"]),
                    timestamp_s: r["timestamp_s"].as_u64().unwrap(),
                },
                finalized_at_us: r["finalized_at_us"].as_u64().unwrap(),
                empty_control_commitment: hash(&r["empty_control_commitment"]),
            })
            .collect(),
    };
    check("l1-fixture", &l1);
    assert_eq!(l1.digest().unwrap(), expected_hash("l1-fixture"));
    let config = config(&input["public_config"]);
    check("public-config", &config);
    assert_eq!(
        *config.reference().unwrap().as_hash(),
        expected_hash("public-config")
    );
    assert_eq!(config.dep_fixture_hash, fixture.digest().unwrap());
    assert_eq!(config.l1_fixture_hash, l1.digest().unwrap());
}

#[test]
fn plan_program_authorization_and_cid_use_frozen_preimages() {
    let input = inputs();
    let config = config(&input["public_config"]);
    let session = SessionId { beta: 0, r: 0 };
    let window = Window {
        h_min: 1,
        h_max: 10,
    };
    let plan_input = PlanInput {
        cause: BoundaryCause::Genesis,
        session: session.clone(),
        source_region: None,
        window: window.clone(),
        reference_block: config.genesis.block.clone(),
    };
    check("plan-input", &plan_input);
    check(
        "boundary-no-anchor",
        &Boundary {
            input: plan_input.clone(),
            anchor: None,
        },
    );
    check(
        "boundary-with-anchor",
        &Boundary {
            input: plan_input.clone(),
            anchor: Some(config.genesis.clone()),
        },
    );
    let config_ref = config.reference().unwrap();
    let authorization = authorization_ref(config_ref, config.registry_hash, &plan_input).unwrap();
    assert_eq!(authorization, expected_hash("authorization-preimage"));
    let committee = input["plan"]["committee"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| CommitteeEntry {
            pid: Pid::new(entry["pid"].as_u64().unwrap() as u8).unwrap(),
            identity_key: unhex(entry["identity_key"].as_str().unwrap())
                .try_into()
                .unwrap(),
            vote_key: unhex(entry["vote_key"].as_str().unwrap())
                .try_into()
                .unwrap(),
            weight: 1,
        })
        .collect();
    let plan = PlanRecord {
        session: session.clone(),
        cause: BoundaryCause::Genesis,
        source_region: None,
        region: RegionId::new(0).unwrap(),
        committee,
        total_weight: 5,
        fault_bound: 1,
        eligible_proposers: (0..5).map(|n| Pid::new(n).unwrap()).collect(),
        leader_rule: RuleSpec {
            rule_id: 1,
            version: 1,
            params: vec![],
        },
        window,
        reference_block: config.genesis.block.clone(),
        config_ref,
        authorization_ref: authorization,
    };
    check("plan-record", &plan);
    let program = ProgramConfig::from_plan(&config, &plan).unwrap();
    check("program-config", &program);
    let cid = regional_cid(&session, config.genesis.block.block_id, &program).unwrap();
    assert_eq!(*cid.as_hash(), expected_hash("cid-anchor-11"));
    let other = regional_cid(&session, Hash32::new([0x22; 32]), &program).unwrap();
    assert_eq!(*other.as_hash(), expected_hash("cid-anchor-22"));
    assert_ne!(cid, other);
    check(
        "descriptor",
        &Descriptor {
            plan: plan.clone(),
            anchor: config.genesis.clone(),
            cid,
        },
    );
    let mut bad = plan.clone();
    bad.committee[0].weight = 2;
    assert!(bad.encode_checked().is_err());
    let mut bad = plan.clone();
    bad.eligible_proposers[1] = bad.eligible_proposers[0];
    assert!(bad.encode_checked().is_err());
    let mut bad = plan;
    bad.region = RegionId::new(1).unwrap();
    assert!(bad.encode_checked().is_err());
}

#[test]
fn raw_transaction_parser_rejects_noncanonical_shapes() {
    let t = bytes("transfer-0");
    let c = bytes("dep-h1-1");
    let a = bytes("dep-h1-0");
    for raw in [&t, &c, &a] {
        let mut tail = raw.clone();
        tail.push(0);
        match raw[0] {
            1 => {
                assert!(Transfer::decode_raw(&tail).is_err());
                assert!(Transfer::decode_raw(&raw[..18]).is_err());
            }
            2 => {
                assert!(Credit::decode_raw(&tail).is_err());
                assert!(Credit::decode_raw(&raw[..9]).is_err());
            }
            3 => {
                assert!(Attributes::decode_raw(&tail).is_err());
                assert!(Attributes::decode_raw(&raw[..24]).is_err());
            }
            _ => unreachable!(),
        }
    }
    assert!(Transfer::decode_raw(&c).is_err());
    assert!(Credit::decode_raw(&a).is_err());
    assert!(Attributes::decode_raw(&t).is_err());
    let mut wrong = t;
    wrong[0] = 255;
    assert!(Transfer::decode_raw(&wrong).is_err());
}

#[test]
fn invalid_sparse_state_config_and_lengths_cannot_be_encoded_or_hashed() {
    let input = inputs();
    let mut s = state(&input["anchor"]["metadata"], &input["anchor"]["accounts"]);
    s.accounts.insert(
        1,
        Account {
            nonce: 0,
            balance: 0,
        },
    );
    assert!(s.encode_checked().is_err());
    assert!(s.root().is_err());
    let mut duplicate = bytes("state-h3");
    duplicate[47] = duplicate[30];
    assert!(ExecutionState::decode(&duplicate).is_err());
    let mut descending = bytes("state-h3");
    descending[30] = 255;
    assert!(ExecutionState::decode(&descending).is_err());
    let mut too_many = bytes("state-h0");
    too_many[26..30].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(ExecutionState::decode(&too_many).is_err());
    let mut enormous = bytes("physical-h3");
    enormous[90..94].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(PhysicalBlock::decode(&enormous).is_err());
    assert!(Window { h_min: 1, h_max: 9 }.encode_checked().is_err());
    assert!(
        Window {
            h_min: 1,
            h_max: 11
        }
        .encode_checked()
        .is_err()
    );
    assert!(
        Window {
            h_min: 1,
            h_max: u64::MAX
        }
        .encode_checked()
        .is_err()
    );
    for which in 0..5 {
        let mut bad = config(&input["public_config"]);
        match which {
            0 => bad.l2_slot_us = 0,
            1 => bad.byte_limit = 0,
            2 => bad.byte_limit = 65537,
            3 => bad.t_drift_us = 6000000,
            4 => bad.rule_version = 2,
            _ => unreachable!(),
        }
        assert!(bad.encode_checked().is_err());
        assert!(bad.reference().is_err());
    }
}
