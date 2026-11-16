#![allow(dead_code)]
//! Fixed independent inputs shared by the component acceptance tests.
use poc_types::domain::*;
use serde_json::Value;
use std::collections::BTreeMap;

pub fn inputs() -> Value {
    serde_json::from_str(include_str!("m2-inputs.json")).unwrap()
}
pub fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
pub fn hash(v: &Value) -> Hash32 {
    Hash32::new(unhex(v.as_str().unwrap()).try_into().unwrap())
}
pub fn golden(name: &str) -> Hash32 {
    let g: Value = serde_json::from_str(include_str!("m2-golden-vectors.json")).unwrap();
    hash(
        &g["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == name)
            .unwrap()["expected_hash"],
    )
}
pub fn metadata(v: &Value) -> Metadata {
    Metadata {
        height: v[0].as_u64().unwrap(),
        timestamp_us: v[1].as_u64().unwrap(),
        origin: v[2].as_u64().unwrap(),
    }
}
pub fn state(m: &Value, a: &Value) -> ExecutionState {
    ExecutionState {
        metadata: metadata(m),
        accounts: a
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
            .collect::<BTreeMap<_, _>>(),
    }
}
pub fn anchor(v: &Value) -> Anchor {
    Anchor {
        block: BlockRef {
            height: v["height"].as_u64().unwrap(),
            origin: v["origin"].as_u64().unwrap(),
            block_id: hash(&v["block_id"]),
        },
        timestamp_us: v["timestamp_us"].as_u64().unwrap(),
    }
}
pub fn config() -> PublicConfig {
    let i = inputs();
    let v = &i["public_config"];
    PublicConfig {
        chain_id: v["chain_id"].as_u64().unwrap(),
        rule_version: 1,
        verifier_version: 1,
        registry_hash: hash(&v["registry_hash"]),
        dep_fixture_hash: hash(&v["dep_fixture_hash"]),
        l1_fixture_hash: hash(&v["l1_fixture_hash"]),
        genesis: anchor(&v["genesis"]),
        genesis_state_root: hash(&v["genesis_state_root"]),
        genesis_native_hash: hash(&v["genesis_native_hash"]),
        l2_slot_us: 1000000,
        l1_genesis_timestamp_us: 0,
        l1_block_us: 2000000,
        t_dep_us: 5000000,
        t_drift_us: 15000000,
        metadata_window_us: 200000,
        byte_limit: 65536,
    }
}
pub fn execution_anchor() -> ExecutionAnchor {
    let i = inputs();
    ExecutionAnchor {
        anchor: config().genesis,
        state: state(&i["anchor"]["metadata"], &i["anchor"]["accounts"]),
        native_hash: hash(&i["anchor"]["native_hash"]),
    }
}
pub fn transfer(v: &Value) -> Vec<u8> {
    Transfer {
        from: v[0].as_u64().unwrap() as u8,
        to: v[1].as_u64().unwrap() as u8,
        nonce: v[2].as_u64().unwrap(),
        amount: v[3].as_u64().unwrap(),
    }
    .encode_raw()
}
pub fn credit(v: &Value) -> Credit {
    Credit {
        to: v[0].as_u64().unwrap() as u8,
        amount: v[1].as_u64().unwrap(),
    }
}
pub fn dep_fixture() -> DepFixture {
    let i = inputs();
    DepFixture {
        first_height: 1,
        rows: i["dep_fixture"]["credits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r.as_array().unwrap().iter().map(credit).collect())
            .collect(),
    }
}
pub fn l1_fixture() -> L1Fixture {
    let i = inputs();
    L1Fixture {
        rows: i["l1_rows"]
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
    }
}
pub fn physical3() -> PhysicalBlock {
    let i = inputs();
    let s = &i["steps"][2];
    PhysicalBlock {
        parent: hash(&i["anchor"]["physical_id"]),
        metadata: metadata(&s["metadata"]),
        pre_state_root: golden("state-h2"),
        transactions: s["transactions"]
            .as_array()
            .unwrap()
            .iter()
            .map(transfer)
            .collect(),
    }
}
