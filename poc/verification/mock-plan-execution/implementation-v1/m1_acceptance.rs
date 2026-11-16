//! Independent M1 API acceptance. Expected data predates the implementation.
use poc_types::domain::{
    Canonical, Cid, ConfigRef, DomainError, Hash32, L1BlockRef, L1Position, MemberId, Pid,
    RegionId, ResultKey, SignerIndex,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{any::TypeId, fmt::Debug, hash::Hash};

fn vectors() -> Value {
    serde_json::from_str(include_str!("m1-vectors.json")).unwrap()
}

fn compatibility() -> Value {
    serde_json::from_str(include_str!("i1-hash-compatibility.json")).unwrap()
}

fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(text, 16).unwrap()
        })
        .collect()
}

fn hash(value: &Value) -> Hash32 {
    Hash32::try_from(unhex(value.as_str().unwrap()).as_slice()).unwrap()
}

fn check_value<T>(actual: T, expected: &[u8])
where
    T: Canonical + Clone + Copy + Debug + Eq + Ord + Hash + Serialize + DeserializeOwned,
{
    assert_eq!(actual.encode(), expected);
    assert_eq!(T::decode(expected).unwrap(), actual);
    let json = serde_json::to_value(actual).unwrap();
    assert_eq!(serde_json::from_value::<T>(json).unwrap(), actual);
}

#[test]
fn fixed_canonical_bytes_round_trip_through_actual_api() {
    for case in vectors()["objects"].as_array().unwrap() {
        let expected = unhex(case["expected_hex"].as_str().unwrap());
        let value = &case["value"];
        match case["type"].as_str().unwrap() {
            "Pid" => check_value(Pid::new(value.as_u64().unwrap() as u8).unwrap(), &expected),
            "RegionId" => check_value(
                RegionId::new(value.as_u64().unwrap() as u8).unwrap(),
                &expected,
            ),
            "MemberId" => check_value(
                MemberId::new(value.as_u64().unwrap() as u8).unwrap(),
                &expected,
            ),
            "SignerIndex" => check_value(
                SignerIndex::new(value.as_u64().unwrap() as u8).unwrap(),
                &expected,
            ),
            "Hash32" => check_value(hash(value), &expected),
            "Cid" => check_value(Cid::new(hash(value)), &expected),
            "ConfigRef" => check_value(ConfigRef::new(hash(value)), &expected),
            "L1Position" => check_value(
                L1Position {
                    block: value["block"].as_u64().unwrap(),
                    tx: value["tx"].as_u64().unwrap(),
                    log: value["log"].as_u64().unwrap(),
                },
                &expected,
            ),
            "L1BlockRef" => check_value(
                L1BlockRef {
                    height: value["height"].as_u64().unwrap(),
                    hash: hash(&value["hash"]),
                },
                &expected,
            ),
            "ResultKey" => check_value(
                ResultKey {
                    cid: Cid::new(hash(&value["cid"])),
                    block_hash: hash(&value["block_hash"]),
                },
                &expected,
            ),
            other => panic!("unknown fixture type {other}"),
        }
    }
}

#[test]
fn strict_decode_errors_and_precedence() {
    for case in vectors()["invalid_encodings"].as_array().unwrap() {
        let bytes = unhex(case["encoded_hex"].as_str().unwrap());
        let kind = case["type"].as_str().unwrap();
        let result = match kind {
            "Pid" => Pid::decode(&bytes).map(|_| ()),
            "RegionId" => RegionId::decode(&bytes).map(|_| ()),
            "MemberId" => MemberId::decode(&bytes).map(|_| ()),
            "SignerIndex" => SignerIndex::decode(&bytes).map(|_| ()),
            "Hash32" => Hash32::decode(&bytes).map(|_| ()),
            "Cid" => Cid::decode(&bytes).map(|_| ()),
            "ConfigRef" => ConfigRef::decode(&bytes).map(|_| ()),
            "L1Position" => L1Position::decode(&bytes).map(|_| ()),
            "L1BlockRef" => L1BlockRef::decode(&bytes).map(|_| ()),
            "ResultKey" => ResultKey::decode(&bytes).map(|_| ()),
            other => panic!("unknown fixture type {other}"),
        };
        let expected = match case["expected_error_class"].as_str().unwrap() {
            "InvalidLength" => DomainError::InvalidLength {
                expected: match kind {
                    "Pid" | "RegionId" | "MemberId" | "SignerIndex" => 3,
                    "Hash32" | "Cid" | "ConfigRef" => 34,
                    "L1Position" => 26,
                    "L1BlockRef" => 42,
                    "ResultKey" => 66,
                    _ => unreachable!(),
                },
                actual: bytes.len(),
            },
            "UnsupportedVersion" => DomainError::UnsupportedVersion {
                version: u16::from_be_bytes([bytes[0], bytes[1]]),
            },
            "InvalidPid" => DomainError::InvalidPid { value: bytes[2] },
            "InvalidRegion" => DomainError::InvalidRegion { value: bytes[2] },
            "InvalidMember" => DomainError::InvalidMember { value: bytes[2] },
            "InvalidSignerIndex" => DomainError::InvalidSignerIndex { value: bytes[2] },
            other => panic!("unknown error {other}"),
        };
        assert_eq!(result, Err(expected), "{}", case["id"]);
    }
}

#[test]
fn checked_identity_construction_and_serde_cannot_bypass_ranges() {
    macro_rules! identity {
        ($ty:ty, $limit:expr, $error:ident) => {
            for value in 0..=u8::MAX {
                if value < $limit {
                    let id = <$ty>::new(value).unwrap();
                    assert_eq!(id.get(), value);
                    assert_eq!(u8::from(id), value);
                    assert_eq!(<$ty>::try_from(value).unwrap(), id);
                    assert_eq!(
                        serde_json::from_value::<$ty>(Value::from(value)).unwrap(),
                        id
                    );
                } else {
                    assert_eq!(<$ty>::new(value), Err(DomainError::$error { value }));
                    assert_eq!(<$ty>::try_from(value), Err(DomainError::$error { value }));
                    assert!(serde_json::from_value::<$ty>(Value::from(value)).is_err());
                }
            }
            for json in ["-1", "256", "1.5", "null"] {
                assert!(serde_json::from_str::<$ty>(json).is_err());
            }
        };
    }
    identity!(Pid, 15, InvalidPid);
    identity!(RegionId, 3, InvalidRegion);
    identity!(MemberId, 5, InvalidMember);
    identity!(SignerIndex, 5, InvalidSignerIndex);
    for case in vectors()["identity_mapping"].as_array().unwrap() {
        let pid = Pid::new(case["pid"].as_u64().unwrap() as u8).unwrap();
        assert_eq!(pid.region().get() as u64, case["region"].as_u64().unwrap());
        assert_eq!(pid.member().get() as u64, case["member"].as_u64().unwrap());
        assert_eq!(Pid::from_region_member(pid.region(), pid.member()), pid);
    }
    assert_ne!(TypeId::of::<MemberId>(), TypeId::of::<SignerIndex>());
    assert_ne!(TypeId::of::<Cid>(), TypeId::of::<ConfigRef>());
    assert_ne!(TypeId::of::<Cid>(), TypeId::of::<poc_types::Cid>());
}

#[test]
fn opaque_hash_width_and_zero_acceptance() {
    for size in [0, 1, 31, 33, 64] {
        assert_eq!(
            Hash32::try_from(vec![0; size].as_slice()),
            Err(DomainError::InvalidLength {
                expected: 32,
                actual: size
            })
        );
        let json = serde_json::to_string(&vec![0u8; size]).unwrap();
        assert!(serde_json::from_str::<Hash32>(&json).is_err());
        assert!(serde_json::from_str::<Cid>(&json).is_err());
        assert!(serde_json::from_str::<ConfigRef>(&json).is_err());
    }
    let zero = Hash32::new([0; 32]);
    assert_eq!(zero.as_bytes(), &[0; 32]);
    assert_eq!(zero.into_bytes(), [0; 32]);
    let cid = Cid::from(zero);
    let config = ConfigRef::from(zero);
    assert_eq!(cid.as_hash(), &zero);
    assert_eq!(config.as_hash(), &zero);
    assert_eq!(cid.as_bytes(), &[0; 32]);
    assert_eq!(config.as_bytes(), &[0; 32]);
}

#[test]
fn positions_and_references_bind_all_components() {
    let v = vectors();
    let positions: Vec<_> = v["l1_position_input"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| L1Position {
            block: p["block"].as_u64().unwrap(),
            tx: p["tx"].as_u64().unwrap(),
            log: p["log"].as_u64().unwrap(),
        })
        .collect();
    let mut actual = positions.clone();
    actual.sort();
    let expected: Vec<_> = v["l1_position_expected_order_indices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| positions[i.as_u64().unwrap() as usize])
        .collect();
    assert_eq!(actual, expected);
    let a = Hash32::new([1; 32]);
    let b = Hash32::new([2; 32]);
    assert_ne!(
        L1BlockRef { height: 7, hash: a },
        L1BlockRef { height: 7, hash: b }
    );
    assert_ne!(
        ResultKey {
            cid: Cid::new(a),
            block_hash: a
        },
        ResultKey {
            cid: Cid::new(a),
            block_hash: b
        }
    );
    assert_ne!(
        ResultKey {
            cid: Cid::new(a),
            block_hash: a
        },
        ResultKey {
            cid: Cid::new(b),
            block_hash: a
        }
    );
}

#[test]
fn i1_hash_parts_and_public_exports_remain_compatible() {
    for case in compatibility()["cases"].as_array().unwrap() {
        let owned: Vec<_> = case["parts_hex"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| unhex(v.as_str().unwrap()))
            .collect();
        let parts: Vec<&[u8]> = owned.iter().map(Vec::as_slice).collect();
        let actual: Vec<u8> = poc_types::hash_parts(case["domain"].as_str().unwrap(), &parts);
        assert_eq!(
            actual,
            unhex(case["expected_hash"].as_str().unwrap()),
            "{}",
            case["id"]
        );
    }
    assert_eq!(poc_types::SCHEMA_VERSION, 1);
    assert_eq!(poc_types::MAX_FRAME_BYTES, 1_048_576);
    assert_eq!(poc_types::MAX_PAYLOAD_BYTES, 65_536);
    assert_eq!(poc_types::LINK_QUEUE_BOUND, 64);
    let _: poc_types::Cid = poc_types::wire::Cid::default();
    assert_eq!(poc_types::hex(&[0, 15, 255]), "000fff");
    assert_eq!(
        poc_types::digest("I1/empty/v1", &poc_types::Grant::default()),
        poc_types::hash_parts("I1/empty/v1", &[&[]])
    );
}

#[test]
fn i1_grant_and_ack_canonical_hashes_preserve_incarnation_exclusion() {
    let compat = compatibility();
    let expected = |name| {
        unhex(
            compat["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|case| case["id"] == name)
                .unwrap()["expected_hash"]
                .as_str()
                .unwrap(),
        )
    };
    let mut grant = poc_types::Grant {
        schema_version: 1,
        run_id: vec![1, 2],
        node_id: 14,
        incarnation: 3,
        logical_step_id: 7,
        event_id: 9,
        tick_us: 1500,
        causal_wave: 1,
        tie_rank: unhex("deadbeef"),
        input: None,
        grant_hash: vec![5; 32],
    };
    assert_eq!(poc_types::grant_hash(&grant), expected("grant"));
    grant.incarnation = 999;
    grant.grant_hash = vec![9; 32];
    assert_eq!(poc_types::grant_hash(&grant), expected("grant"));
    grant.event_id += 1;
    assert_ne!(poc_types::grant_hash(&grant), expected("grant"));
    let mut ack = poc_types::Ack {
        schema_version: 1,
        run_id: vec![1, 2],
        node_id: 14,
        incarnation: 3,
        logical_step_id: 7,
        event_id: 9,
        grant_hash: unhex("aabb"),
        status: 1,
        effects: vec![],
        durable_journal_seq: 11,
        state_hash: unhex("ccdd"),
        logical_ack_hash: vec![5; 32],
    };
    assert_eq!(poc_types::ack_hash(&ack), expected("ack"));
    ack.incarnation = 999;
    ack.logical_ack_hash = vec![9; 32];
    assert_eq!(poc_types::ack_hash(&ack), expected("ack"));
    ack.state_hash[0] ^= 1;
    assert_ne!(poc_types::ack_hash(&ack), expected("ack"));
}
