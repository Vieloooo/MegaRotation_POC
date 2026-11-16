use batch_services::{
    crypto::{
        BatchWire, CursorWire, InboxDomain, SigningKey, check_da, decode_fixed, decode_hex,
        recover_signer, validate_submission, verify_batch,
    },
    frame::{
        BlockRecord, CodecError, MAX_PAYLOAD, commitment, decode_closed, encode_closed,
        verify_payload,
    },
};
use poc_types::domain::{Hash32, Transfer};
use serde_json::Value;

fn vectors(name: &str) -> Value {
    let data = match name {
        "frame" => include_str!("../../../verification/batch-services/frame-golden-vectors.json"),
        "signature" => {
            include_str!("../../../verification/batch-services/signature-golden-vectors.json")
        }
        _ => panic!("unknown vector file"),
    };
    serde_json::from_str(data).unwrap()
}

fn string(value: &Value) -> &str {
    value.as_str().unwrap()
}
fn integer(value: &Value) -> u64 {
    value.as_u64().unwrap()
}
fn bytes(value: &Value) -> Vec<u8> {
    decode_hex(string(value), MAX_PAYLOAD + 100, "vector").unwrap()
}
fn fixed<const N: usize>(value: &Value) -> [u8; N] {
    decode_fixed(string(value), "vector").unwrap()
}

#[test]
fn independent_closed_frame_vectors() {
    let data = vectors("frame");
    for vector in data["valid"].as_array().unwrap() {
        let records = vector["records"]
            .as_array()
            .unwrap()
            .iter()
            .map(|record| BlockRecord {
                block_number: integer(&record["height"]),
                epoch_num: integer(&record["origin"]),
                transactions: record["transactions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(bytes)
                    .collect(),
            })
            .collect::<Vec<_>>();
        let payload = bytes(&vector["payload"]);
        let expected_d = Hash32::new(fixed(&vector["commitment"]));
        assert_eq!(
            encode_closed(&records).unwrap(),
            payload,
            "{}",
            vector["name"]
        );
        let actual = verify_payload(expected_d, &payload).unwrap();
        assert_eq!(actual.records, records, "{}", vector["name"]);
        assert_eq!(actual.channel_id, fixed(&vector["channel_id"]));
        assert_eq!(
            verify_payload(Hash32::new([0; 32]), &payload),
            Err(CodecError::CommitmentMismatch)
        );
    }
    for vector in data["invalid"].as_array().unwrap() {
        let payload = bytes(&vector["payload"]);
        assert!(decode_closed(&payload).is_err(), "{}", vector["name"]);
        assert!(
            verify_payload(commitment(&payload), &payload).is_err(),
            "{}",
            vector["name"]
        );
    }
    let records = (1..=2)
        .map(|block_number| BlockRecord {
            block_number,
            epoch_num: 0,
            transactions: (0..1024)
                .map(|nonce| {
                    Transfer {
                        from: 0,
                        to: 1,
                        nonce,
                        amount: 0,
                    }
                    .encode_raw()
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    let payload = encode_closed(&records).unwrap();
    assert_eq!(payload.len(), MAX_PAYLOAD);
    assert_eq!(
        MAX_PAYLOAD as u64,
        integer(&data["boundary"]["payload_length"])
    );
    let expected_d = Hash32::new(fixed(&data["boundary"]["commitment"]));
    assert_eq!(
        verify_payload(expected_d, &payload).unwrap().records,
        records
    );
}

fn cursor(data: &Value) -> CursorWire {
    CursorWire {
        height: integer(&data[0]),
        origin: integer(&data[1]),
        frames_digest: string(&data[2]).into(),
        source_kind: integer(&data[3]) as u8,
        source: string(&data[4]).into(),
    }
}

fn attestation(data: &Value, signature: &Value) -> BatchWire {
    BatchWire {
        request_id: format!("0x{}", "00".repeat(32)),
        prev: cursor(&data["prev"]),
        next: cursor(&data["next"]),
        commitment: string(&data["commitment"]).into(),
        da_certificate: string(&data["da_certificate"]).into(),
        first_height: integer(&data["first_height"]),
        last_height: integer(&data["last_height"]),
        gpos: string(&data["gpos"]).into(),
        certifier_signature: string(signature).into(),
    }
}

fn domain(data: &Value) -> InboxDomain {
    InboxDomain {
        chain_id: integer(&data["chain_id"]),
        inbox: fixed(&data["inbox"]),
        config_ref: Hash32::new(fixed(&data["config_ref"])),
    }
}

#[test]
fn independent_ethereum_attestation_vectors() {
    let data = vectors("signature");
    let fixture = &data["fixture"];
    let expected_domain = domain(fixture);
    let da_signer = fixed(&data["da_signer"]);
    let certifier = fixed(&data["certifier_signer"]);
    let d = Hash32::new(fixed(&fixture["commitment"]));
    let da = bytes(&fixture["da_certificate"]);
    let cert = bytes(&data["certifier_signature"]);
    let batch = attestation(fixture, &data["certifier_signature"]);
    validate_submission(&batch).unwrap();
    assert_eq!(
        expected_domain.da_digest(d).unwrap(),
        Hash32::new(fixed(&data["da_digest"]))
    );
    let batch_digest = expected_domain.batch_digest(&batch).unwrap();
    assert_eq!(batch_digest, Hash32::new(fixed(&data["batch_digest"])));
    check_da(d, &da, &expected_domain, da_signer).unwrap();
    verify_batch(&batch, &expected_domain, certifier).unwrap();
    assert!(check_da(d, &cert, &expected_domain, certifier).is_err());
    assert_ne!(recover_signer(batch_digest, &da).unwrap(), da_signer);

    for vector in data["mutations"].as_array().unwrap() {
        let mut mutated = fixture.clone();
        let changed = string(&vector["changed"]);
        let value = if let Some((which, field)) = changed.split_once('.') {
            let index = ["height", "origin", "frames_digest", "source_kind", "source"]
                .iter()
                .position(|name| *name == field)
                .unwrap();
            &mut mutated[which][index]
        } else {
            &mut mutated[changed]
        };
        *value = if let Some(v) = value.as_u64() {
            (v + 1).into()
        } else if ["prev.source", "next.source", "gpos", "da_certificate"].contains(&changed) {
            format!("{}01", string(value)).into()
        } else {
            let mut raw = bytes(value);
            raw[0] ^= 1;
            batch_services::crypto::hex(&raw).into()
        };
        let mutated_domain = domain(&mutated);
        let (digest, signature, signer) = if vector["role"] == "da" {
            let mutated_d = Hash32::new(fixed(&mutated["commitment"]));
            assert!(
                check_da(mutated_d, &da, &mutated_domain, da_signer).is_err(),
                "{changed}"
            );
            (mutated_domain.da_digest(mutated_d).unwrap(), &da, da_signer)
        } else {
            let mutated_batch = attestation(&mutated, &data["certifier_signature"]);
            assert!(
                verify_batch(&mutated_batch, &mutated_domain, certifier).is_err(),
                "{changed}"
            );
            if changed == "da_certificate" {
                // The submitted certificate has a strict 65-byte shape.
                assert!(mutated_domain.batch_digest(&mutated_batch).is_err());
                continue;
            }
            (
                mutated_domain.batch_digest(&mutated_batch).unwrap(),
                &cert,
                certifier,
            )
        };
        assert_eq!(digest, Hash32::new(fixed(&vector["digest"])), "{changed}");
        let recovered = recover_signer(digest, signature).unwrap();
        assert_eq!(recovered, fixed(&vector["recovered"]), "{changed}");
        assert_ne!(recovered, signer);
    }
    for vector in data["malformed_signatures"].as_array().unwrap() {
        assert!(
            recover_signer(batch_digest, &bytes(&vector["signature"])).is_err(),
            "{}",
            vector["name"]
        );
    }
}

#[test]
fn local_signing_matches_ethereum_recovery() {
    let data = vectors("signature");
    let fixture = &data["fixture"];
    let domain = domain(fixture);
    let d = Hash32::new(fixed(&fixture["commitment"]));
    // Public synthetic test key, checked against Ethereum's known address for scalar 1.
    let mut scalar = [0; 32];
    scalar[31] = 1;
    let key = SigningKey::new(scalar).unwrap();
    let signer =
        decode_fixed("0x7e5f4552091a69125d5dfcb7b8c2659029395bdf", "test address").unwrap();
    assert_eq!(key.address(), signer);
    let digest = domain.da_digest(d).unwrap();
    let signature = key.sign_digest(digest).unwrap();
    assert_eq!(signature, key.sign_digest(digest).unwrap());
    check_da(d, &signature, &domain, signer).unwrap();
}
