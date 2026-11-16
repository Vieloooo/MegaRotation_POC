//! Independent contract-driven M1 replay-interface checker.
//! Expected bytes come from frozen verifier vectors, never candidate encoders.
use poc_types::domain::{
    Anchor, CanonicalObject, CloseReason, Descriptor, Hash32, L1Position, LifecycleBody,
    LifecycleEvent, Metadata, PhysicalBlock, ReplayIndex, ReplayKind, ReplayPosition, SessionId,
    VirtualBlock,
};
use poc_viewer_gfr::{
    PublicObject, compare_positions, decode_position, decode_publication, encode_position,
    encode_publication, lifecycle_id, lifecycle_projection,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, panic::AssertUnwindSafe};

const VECTORS: &str = include_str!("../../interface-vectors-v1.json");

#[derive(Default)]
struct Report {
    passed: usize,
    failed: usize,
}

impl Report {
    fn check(&mut self, case: &str, pass: bool, expected: Value, observed: Value) {
        if pass {
            self.passed += 1;
        } else {
            self.failed += 1;
        }
        println!(
            "{}",
            json!({"case":case,"result":if pass {"PASS"} else {"FAIL"},
                              "expected":expected,"observed":observed})
        );
    }
}

fn unhex(s: &str) -> Vec<u8> {
    assert!(s.len().is_multiple_of(2), "invalid independent hex fixture");
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|b| format!("{b:02x}")).collect()
}

fn array(v: &Value) -> &[Value] {
    v.as_array().expect("fixture array")
}
fn text(v: &Value) -> &str {
    v.as_str().expect("fixture string")
}
fn num(v: &Value) -> u64 {
    v.as_u64().expect("fixture u64")
}
fn hash(v: &Value) -> Hash32 {
    Hash32::try_from(unhex(text(v)).as_slice()).unwrap()
}

fn position(key_us: u64, kind: &str, index: &[Value]) -> ReplayPosition {
    let (kind, index) = match kind {
        "Force" | "Resume" => (
            if kind == "Force" {
                ReplayKind::Force
            } else {
                ReplayKind::Resume
            },
            ReplayIndex::L1(L1Position {
                block: num(&index[0]),
                tx: num(&index[1]),
                log: num(&index[2]),
            }),
        ),
        "Expire" => (ReplayKind::Expire, ReplayIndex::Height(num(&index[0]))),
        "SessionEnd" => (
            ReplayKind::SessionEnd,
            ReplayIndex::Session(SessionId {
                beta: num(&index[0]),
                r: num(&index[1]),
            }),
        ),
        "Espresso" => (ReplayKind::Espresso, ReplayIndex::Espresso(num(&index[0]))),
        _ => panic!("invalid independent kind fixture"),
    };
    ReplayPosition {
        key_us,
        kind,
        index,
    }
}

fn tuple_position(v: &Value) -> ReplayPosition {
    position(num(&v[0]), text(&v[1]), array(&v[2]))
}

fn reason(s: &str) -> CloseReason {
    match s {
        "Time" => CloseReason::Time,
        "Complete" => CloseReason::Complete,
        "Force" => CloseReason::Force,
        "Resume" => CloseReason::Resume,
        "Expired" => CloseReason::Expired,
        "Excluded" => CloseReason::Excluded,
        "Incompatible" => CloseReason::Incompatible,
        _ => panic!("invalid independent reason fixture"),
    }
}

fn virtual_block(v: &Value) -> VirtualBlock {
    VirtualBlock {
        metadata: Metadata::decode(&unhex(text(&v["metadata_hex"]))).unwrap(),
        dep: array(&v["dep_hex"])
            .iter()
            .map(|x| unhex(text(x)))
            .collect(),
        pre_state_root: hash(&v["pre_state_root"]),
        transactions: array(&v["transactions_hex"])
            .iter()
            .map(|x| unhex(text(x)))
            .collect(),
    }
}

fn lifecycle(v: &Value) -> LifecycleEvent {
    let b = &v["body"];
    let body = match text(&b["kind"]) {
        "SessionClosed" => LifecycleBody::SessionClosed {
            session: SessionId {
                beta: num(&b["beta"]),
                r: num(&b["r"]),
            },
            reason: reason(text(&b["reason"])),
        },
        "BranchClosed" => LifecycleBody::BranchClosed {
            branch: num(&b["branch"]),
            reason: reason(text(&b["reason"])),
        },
        "SessionAssigned" => LifecycleBody::SessionAssigned {
            descriptor: Descriptor::decode(&unhex(text(&b["descriptor_hex"]))).unwrap(),
        },
        "Resumed" => LifecycleBody::Resumed {
            control_seq: num(&b["control_seq"]),
            branch: num(&b["branch"]),
            anchor: Anchor::decode(&unhex(text(&b["anchor_hex"]))).unwrap(),
            derived_prefix: array(&b["prefix"]).iter().map(virtual_block).collect(),
        },
        _ => panic!("invalid independent lifecycle fixture"),
    };
    LifecycleEvent {
        position: tuple_position(&v["position"]),
        body,
    }
}

fn publication(v: &Value) -> PublicObject {
    match text(&v["object"]) {
        "Body" => PublicObject::Body(PhysicalBlock::decode(&unhex(text(&v["inner_hex"]))).unwrap()),
        "FinalProof" => PublicObject::FinalProof {
            descriptor: Descriptor::decode(&unhex(text(&v["descriptor_hex"]))).unwrap(),
            target: hash(&v["target_hex"]),
            proof: unhex(text(&v["proof_hex"])),
        },
        _ => panic!("invalid independent publication fixture"),
    }
}

// Treat panics as failed rejection checks, never as successful invalid-input handling.
fn rejected_position(b: &[u8]) -> bool {
    matches!(std::panic::catch_unwind(|| decode_position(b)), Ok(Err(_)))
}
fn rejected_publication(b: &[u8]) -> bool {
    matches!(
        std::panic::catch_unwind(|| decode_publication(b)),
        Ok(Err(_))
    )
}

fn check_positions(v: &Value, r: &mut Report) {
    for p in array(&v["positions"]) {
        let id = text(&p["id"]);
        let want = position(num(&p["key_us"]), text(&p["kind"]), array(&p["index"]));
        let expected = unhex(text(&p["expected_hex"]));
        let encoded = encode_position(&want);
        r.check(
            &format!("position/{id}/encode"),
            encoded.as_ref().is_ok_and(|b| *b == expected),
            p["expected_hex"].clone(),
            json!(
                encoded
                    .as_ref()
                    .map(|b| hex(b))
                    .map_err(|e| format!("{e:?}"))
            ),
        );
        let decoded = decode_position(&expected);
        r.check(
            &format!("position/{id}/decode"),
            decoded.as_ref().is_ok_and(|x| *x == want),
            json!(format!("{want:?}")),
            json!(format!("{decoded:?}")),
        );
        let bad_offsets: Vec<_> = (0..expected.len())
            .filter(|&n| !rejected_position(&expected[..n]))
            .collect();
        r.check(
            &format!("position/{id}/all_truncations"),
            bad_offsets.is_empty(),
            json!({"reject_prefixes":expected.len()}),
            json!({"unexpected_offsets":bad_offsets}),
        );
    }
    for p in array(&v["negative_positions"]) {
        let pass = rejected_position(&unhex(text(&p["hex"])));
        r.check(
            &format!("position/{}/invalid", text(&p["id"])),
            pass,
            json!("REJECT_WITHOUT_PANIC"),
            json!(pass),
        );
    }
    for (i, c) in array(&v["comparisons"]).iter().enumerate() {
        let got = compare_positions(&tuple_position(&c["a"]), &tuple_position(&c["b"]));
        let observed = match got {
            Ok(o) => format!("{o:?}"),
            Err(_) => "REJECT".to_string(),
        };
        r.check(
            &format!("comparison/{i}"),
            observed == text(&c["expected"]),
            c["expected"].clone(),
            json!(observed),
        );
    }
    let good = ReplayPosition {
        key_us: 0,
        kind: ReplayKind::Espresso,
        index: ReplayIndex::Espresso(0),
    };
    let kinds = [
        ReplayKind::Force,
        ReplayKind::Resume,
        ReplayKind::Expire,
        ReplayKind::SessionEnd,
        ReplayKind::Espresso,
    ];
    let indices = [
        ReplayIndex::L1(L1Position {
            block: 0,
            tx: 0,
            log: 0,
        }),
        ReplayIndex::Height(0),
        ReplayIndex::Session(SessionId { beta: 0, r: 0 }),
        ReplayIndex::Espresso(0),
    ];
    for (ki, kind) in kinds.into_iter().enumerate() {
        for (ii, index) in indices.iter().enumerate() {
            if (ki <= 1 && ii == 0)
                || (ki == 2 && ii == 1)
                || (ki == 3 && ii == 2)
                || (ki == 4 && ii == 3)
            {
                continue;
            }
            let bad = ReplayPosition {
                key_us: 0,
                kind,
                index: index.clone(),
            };
            let pass = encode_position(&bad).is_err()
                && compare_positions(&bad, &good).is_err()
                && compare_positions(&good, &bad).is_err();
            r.check(
                &format!("typed_position_mismatch/{ki}/{ii}"),
                pass,
                json!("encoder and both comparator directions reject"),
                json!(pass),
            );
        }
    }
    let bytes = unhex(text(&v["positions"][0]["expected_hex"]));
    let failures: Vec<_> = (5..=255u8)
        .filter(|tag| {
            let mut b = bytes.clone();
            b[10] = *tag;
            !rejected_position(&b)
        })
        .collect();
    r.check(
        "position/all_unknown_tags",
        failures.is_empty(),
        json!("reject 5..255"),
        json!(failures),
    );
}

fn check_lifecycle(v: &Value, r: &mut Report, capacity: bool) {
    let mut ids = BTreeSet::new();
    for p in array(&v["lifecycle"]) {
        let id = text(&p["id"]);
        let event = lifecycle(p);
        let expected = unhex(text(&p["expected_hex"]));
        let projected = lifecycle_projection(&event);
        r.check(
            &format!("lifecycle/{id}/projection"),
            projected.as_ref().is_ok_and(|b| *b == expected),
            p["expected_hex"].clone(),
            json!(
                projected
                    .as_ref()
                    .map(|b| hex(b))
                    .map_err(|e| format!("{e:?}"))
            ),
        );
        let got = lifecycle_id(&event);
        let got_hex = got
            .as_ref()
            .map(|h| hex(h.as_bytes()))
            .map_err(|e| format!("{e:?}"));
        r.check(
            &format!("lifecycle/{id}/id"),
            got_hex.as_ref().is_ok_and(|s| s == text(&p["expected_id"])),
            p["expected_id"].clone(),
            json!(got_hex),
        );
        if let Ok(h) = got {
            ids.insert(hex(h.as_bytes()));
        }
        let repeat = lifecycle_projection(&event);
        r.check(
            &format!("lifecycle/{id}/repeat"),
            repeat.as_ref().is_ok_and(|b| *b == expected),
            json!("same exact bytes"),
            json!(
                repeat
                    .as_ref()
                    .map(|b| b.len())
                    .map_err(|e| format!("{e:?}"))
            ),
        );
    }
    r.check(
        "lifecycle/distinct_bodies_same_pc",
        ids.len() == array(&v["lifecycle"]).len(),
        json!(array(&v["lifecycle"]).len()),
        json!(ids.len()),
    );
    let base = lifecycle(array(&v["lifecycle"]).last().unwrap());
    let LifecycleBody::Resumed {
        control_seq,
        branch,
        anchor,
        derived_prefix,
    } = base.body.clone()
    else {
        panic!("fixture")
    };
    let sample = derived_prefix[0].clone();
    for (name, n, ok) in [
        ("at_prefix_limit", 4096, true),
        ("over_prefix_limit", 4097, false),
    ] {
        let event = LifecycleEvent {
            position: base.position.clone(),
            body: LifecycleBody::Resumed {
                control_seq,
                branch,
                anchor: anchor.clone(),
                derived_prefix: vec![sample.clone(); n],
            },
        };
        let got = lifecycle_projection(&event);
        r.check(
            &format!("lifecycle/{name}"),
            got.is_ok() == ok,
            json!(if ok { "ACCEPT" } else { "REJECT" }),
            json!(got.as_ref().map(|b| b.len()).map_err(|e| format!("{e:?}"))),
        );
    }
    for name in [
        "missing_attributes",
        "attributes_mismatch",
        "bad_transaction",
    ] {
        let mut b = sample.clone();
        match name {
            "missing_attributes" => b.dep.clear(),
            "attributes_mismatch" => b.metadata.height += 1,
            "bad_transaction" => b.transactions.push(vec![0xff]),
            _ => unreachable!(),
        }
        let event = LifecycleEvent {
            position: base.position.clone(),
            body: LifecycleBody::Resumed {
                control_seq,
                branch,
                anchor: anchor.clone(),
                derived_prefix: vec![b],
            },
        };
        let pass = lifecycle_projection(&event).is_err() && lifecycle_id(&event).is_err();
        r.check(
            &format!("lifecycle/{name}"),
            pass,
            json!("REJECT"),
            json!(pass),
        );
    }
    let mut bad = base.clone();
    bad.position.index = ReplayIndex::Height(7);
    r.check(
        "lifecycle/bad_position",
        lifecycle_projection(&bad).is_err() && lifecycle_id(&bad).is_err(),
        json!("REJECT"),
        json!(format!("{:?}", lifecycle_projection(&bad))),
    );
    if capacity {
        check_aggregate_capacity(&base, &sample, r);
    }
}

fn check_aggregate_capacity(base: &LifecycleEvent, sample: &VirtualBlock, r: &mut Report) {
    // Layout arithmetic is independent of production. Each large block has
    // 1 attributes + 1024 credits and 1024 ordinary transfers, all valid raw bytes.
    let mut block = sample.clone();
    block.dep.truncate(1);
    let mut credit = vec![2, 0];
    credit.extend_from_slice(&1u64.to_be_bytes());
    block.dep.extend(std::iter::repeat_n(credit, 1024));
    let mut transfer = vec![1, 0, 1];
    transfer.extend_from_slice(&0u64.to_be_bytes());
    transfer.extend_from_slice(&1u64.to_be_bytes());
    block.transactions = vec![transfer; 1024];
    let block_bytes = 30 + 4 + 29 + 1024 * 14 + 32 + 4 + 1024 * 23;
    let base_bytes = 2 + 4 + 19 + 1 + 8 + 8 + 4 + 58 + 4;
    let limit = 64 * 1024 * 1024;
    let count = (limit - base_bytes) / (4 + block_bytes) + 1;
    assert!(count <= 4096);
    let mut event = base.clone();
    if let LifecycleBody::Resumed { derived_prefix, .. } = &mut event.body {
        *derived_prefix = vec![block; count];
    }
    let got = std::panic::catch_unwind(AssertUnwindSafe(|| lifecycle_projection(&event)));
    let pass = matches!(got, Ok(Err(_)));
    r.check(
        "lifecycle/aggregate_64mib_limit",
        pass,
        json!({"reject":true,"independent_bytes":base_bytes+count*(4+block_bytes),"limit":limit}),
        json!({"reject_without_panic":pass}),
    );
}

fn check_publications(v: &Value, r: &mut Report) {
    r.check(
        "publication/codec_tag",
        u64::from(poc_viewer_gfr::PUBLICATION_CODEC) == num(&v["publication_codec"]),
        v["publication_codec"].clone(),
        json!(poc_viewer_gfr::PUBLICATION_CODEC),
    );
    for p in array(&v["publications"]) {
        let id = text(&p["id"]);
        let expected = unhex(text(&p["expected_hex"]));
        let encoded = encode_publication(&publication(p));
        r.check(
            &format!("publication/{id}/encode"),
            encoded.as_ref().is_ok_and(|b| *b == expected),
            p["expected_hex"].clone(),
            json!(
                encoded
                    .as_ref()
                    .map(|b| hex(b))
                    .map_err(|e| format!("{e:?}"))
            ),
        );
        let roundtrip = decode_publication(&expected).and_then(|o| encode_publication(&o));
        r.check(
            &format!("publication/{id}/decode"),
            roundtrip.as_ref().is_ok_and(|b| *b == expected),
            json!("exact independent bytes preserved"),
            json!(
                roundtrip
                    .as_ref()
                    .map(|b| hex(b))
                    .map_err(|e| format!("{e:?}"))
            ),
        );
        let failures: Vec<_> = (0..expected.len())
            .filter(|&n| !rejected_publication(&expected[..n]))
            .collect();
        r.check(
            &format!("publication/{id}/all_truncations"),
            failures.is_empty(),
            json!({"reject_prefixes":expected.len()}),
            json!({"unexpected_offsets":failures}),
        );
    }
    for p in array(&v["negative_publications"]) {
        let pass = rejected_publication(&unhex(text(&p["hex"])));
        r.check(
            &format!("publication/{}/invalid", text(&p["id"])),
            pass,
            json!("REJECT_WITHOUT_PANIC"),
            json!(pass),
        );
    }
    let fixture = array(&v["publications"])
        .iter()
        .find(|p| p["object"] == "FinalProof")
        .unwrap();
    for (name, size, ok) in [
        ("proof_at_limit", 32768, true),
        ("proof_over_limit", 32769, false),
    ] {
        let descriptor = Descriptor::decode(&unhex(text(&fixture["descriptor_hex"]))).unwrap();
        let object = PublicObject::FinalProof {
            descriptor,
            target: hash(&fixture["target_hex"]),
            proof: vec![0xab; size],
        };
        let got = encode_publication(&object);
        r.check(
            &format!("publication/{name}/encode"),
            got.is_ok() == ok,
            json!(if ok { "ACCEPT" } else { "REJECT" }),
            json!(got.as_ref().map(|b| b.len()).map_err(|e| format!("{e:?}"))),
        );
        // Construct expected envelope ourselves to test decoder limit independently.
        let desc = unhex(text(&fixture["descriptor_hex"]));
        let mut wire = b"GFRP\0\x01\x01".to_vec();
        wire.extend_from_slice(&(desc.len() as u32).to_be_bytes());
        wire.extend_from_slice(&desc);
        wire.extend_from_slice(&unhex(text(&fixture["target_hex"])));
        wire.extend_from_slice(&(size as u32).to_be_bytes());
        wire.resize(wire.len() + size, 0xab);
        let result = std::panic::catch_unwind(|| decode_publication(&wire));
        let pass = if ok {
            matches!(result, Ok(Ok(_)))
        } else {
            matches!(result, Ok(Err(_)))
        };
        r.check(
            &format!("publication/{name}/decode"),
            pass,
            json!(if ok { "ACCEPT" } else { "REJECT_WITHOUT_PANIC" }),
            json!(pass),
        );
    }
    let mut bytes = unhex(text(&fixture["expected_hex"]));
    bytes.resize(65537, 0);
    r.check(
        "publication/over_message_limit",
        rejected_publication(&bytes),
        json!("REJECT_WITHOUT_PANIC"),
        json!(rejected_publication(&bytes)),
    );
    let mut inner_bad = unhex(text(&v["publications"][0]["expected_hex"]));
    inner_bad[11] = 0;
    inner_bad[12] = 2;
    r.check(
        "publication/nested_03_version",
        rejected_publication(&inner_bad),
        json!("REJECT"),
        json!(rejected_publication(&inner_bad)),
    );
}

fn main() {
    let capacity = std::env::args().any(|a| a == "--capacity");
    let vectors: Value = serde_json::from_str(VECTORS).unwrap();
    println!(
        "{}",
        json!({"suite":"GFR-07-M1-independent","evidence":"non-cryptographic codec/interface","vectors_sha256":"0409be69fad24e8726df7e952cd0919161e943dbfe05b30431343beba7ce1926","aggregate_capacity_included":capacity})
    );
    let mut report = Report::default();
    check_positions(&vectors, &mut report);
    check_lifecycle(&vectors, &mut report, capacity);
    check_publications(&vectors, &mut report);
    println!(
        "{}",
        json!({"summary":{"passed":report.passed,"failed":report.failed},"result":if report.failed==0 {"PASS"}else{"FAIL"},"limits":"M1 codec/comparison only; no GFR state transition, native certificate, real control, RESUME execution, or integration acceptance"})
    );
    if report.failed > 0 {
        std::process::exit(1);
    }
}
