use batch_services::{crypto as c, frame as f};
use poc_types::domain::{Hash32, Transfer};
use serde_json::{Value, json};
use std::{env, fs};

fn raw(v: &Value) -> Vec<u8> {
    let text = v.as_str().unwrap().strip_prefix("0x").unwrap();
    assert_eq!(text.len() % 2, 0);
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}
fn fixed<const N: usize>(v: &Value) -> [u8; N] {
    raw(v).try_into().unwrap()
}
fn cursor(v: &Value) -> c::CursorWire {
    c::CursorWire {
        height: v[0].as_u64().unwrap(),
        origin: v[1].as_u64().unwrap(),
        frames_digest: v[2].as_str().unwrap().into(),
        source_kind: v[3].as_u64().unwrap() as u8,
        source: v[4].as_str().unwrap().into(),
    }
}
fn batch(fixture: &Value, signature: &Value) -> c::BatchWire {
    c::BatchWire {
        request_id: format!("0x{}", "00".repeat(31) + "01"),
        prev: cursor(&fixture["prev"]),
        next: cursor(&fixture["next"]),
        commitment: fixture["commitment"].as_str().unwrap().into(),
        da_certificate: fixture["da_certificate"].as_str().unwrap().into(),
        first_height: fixture["first_height"].as_u64().unwrap(),
        last_height: fixture["last_height"].as_u64().unwrap(),
        gpos: fixture["gpos"].as_str().unwrap().into(),
        certifier_signature: signature.as_str().unwrap().into(),
    }
}
fn domain(fixture: &Value) -> c::InboxDomain {
    c::InboxDomain {
        chain_id: fixture["chain_id"].as_u64().unwrap(),
        inbox: fixed(&fixture["inbox"]),
        config_ref: Hash32::new(fixed(&fixture["config_ref"])),
    }
}
fn output_frame(vector: &Value) -> Value {
    let records: Vec<f::BlockRecord> = vector["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| f::BlockRecord {
            block_number: r["height"].as_u64().unwrap(),
            epoch_num: r["origin"].as_u64().unwrap(),
            transactions: r["transactions"]
                .as_array()
                .unwrap()
                .iter()
                .map(raw)
                .collect(),
        })
        .collect();
    let encoded = f::encode_closed(&records).unwrap();
    let expected = raw(&vector["payload"]);
    assert_eq!(encoded, expected);
    let digest = Hash32::new(fixed(&vector["commitment"]));
    let decoded = f::verify_payload(digest, &encoded).unwrap();
    assert_eq!(decoded.records, records);
    assert_eq!(
        c::hex(&decoded.channel_id),
        vector["channel_id"].as_str().unwrap()
    );
    assert_eq!(f::commitment(&encoded), digest);
    assert!(matches!(
        f::verify_payload(Hash32::new([0; 32]), &encoded),
        Err(f::CodecError::CommitmentMismatch)
    ));
    for size in 0..encoded.len() {
        assert!(f::decode_closed(&encoded[..size]).is_err());
    }
    let mut extra = encoded.clone();
    extra.push(0);
    assert!(f::decode_closed(&extra).is_err());
    json!({"name":vector["name"], "length":encoded.len(), "commitment":c::hex(digest.as_bytes()), "channel_id":c::hex(&decoded.channel_id), "all_truncations_rejected":encoded.len()})
}
fn main() {
    let args: Vec<_> = env::args().collect();
    let root = &args[1];
    let load = |name: &str| -> Value {
        serde_json::from_slice(&fs::read(format!("{root}/{name}")).unwrap()).unwrap()
    };
    let frames = load("frame-golden-vectors.json");
    let extra = load("random-frame-vectors.json");
    let mut positives = vec![];
    for v in frames["valid"]
        .as_array()
        .unwrap()
        .iter()
        .chain(extra["valid"].as_array().unwrap())
    {
        positives.push(output_frame(v));
    }
    let mut negatives = vec![];
    for v in frames["invalid"].as_array().unwrap() {
        let bytes = raw(&v["payload"]);
        assert!(f::decode_closed(&bytes).is_err());
        assert!(f::verify_payload(f::commitment(&bytes), &bytes).is_err());
        negatives.push(v["name"].clone());
    }
    let maximum: Vec<_> = (1..=2)
        .map(|h| f::BlockRecord {
            block_number: h,
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
        .collect();
    let encoded = f::encode_closed(&maximum).unwrap();
    assert_eq!(encoded.len(), 47175);
    assert_eq!(
        f::commitment(&encoded),
        Hash32::new(fixed(&frames["boundary"]["commitment"]))
    );
    assert_eq!(f::decode_closed(&encoded).unwrap().records, maximum);
    let mut oversized = encoded.clone();
    oversized.push(0);
    assert!(matches!(
        f::decode_closed(&oversized),
        Err(f::CodecError::LimitExceeded)
    ));
    let mut oversized_txs = maximum.clone();
    oversized_txs[0].transactions.push(vec![0; 19]);
    assert!(f::encode_closed(&oversized_txs).is_err());
    let mut wrapped = maximum.clone();
    wrapped[0].block_number = u64::MAX;
    wrapped[1].block_number = 0;
    assert!(matches!(
        f::encode_closed(&wrapped),
        Err(f::CodecError::Overflow)
    ));
    let signatures = load("signature-golden-vectors.json");
    let fixture = &signatures["fixture"];
    let domain = domain(fixture);
    let original = batch(fixture, &signatures["certifier_signature"]);
    let commitment = Hash32::new(fixed(&fixture["commitment"]));
    let da = raw(&fixture["da_certificate"]);
    let cert = raw(&signatures["certifier_signature"]);
    c::check_da(commitment, &da, &domain, fixed(&signatures["da_signer"])).unwrap();
    c::verify_batch(&original, &domain, fixed(&signatures["certifier_signer"])).unwrap();
    c::validate_submission(&original).unwrap();
    assert!(
        c::check_da(
            commitment,
            &da,
            &domain,
            fixed(&signatures["certifier_signer"])
        )
        .is_err()
    );
    assert!(c::verify_batch(&original, &domain, fixed(&signatures["da_signer"])).is_err());
    let mut failures = vec![];
    for v in signatures["mutations"].as_array().unwrap() {
        let message = Hash32::new(fixed(&v["digest"]));
        let (sig, expected) = if v["role"] == "da" {
            (&da, fixed::<20>(&signatures["da_signer"]))
        } else {
            (&cert, fixed::<20>(&signatures["certifier_signer"]))
        };
        let address = c::recover_signer(message, sig).unwrap();
        assert_ne!(address, expected);
        assert_eq!(address, fixed::<20>(&v["recovered"]));
        let mut changed = fixture.clone();
        let label = v["changed"].as_str().unwrap();
        let field = if let Some((which, part)) = label.split_once('.') {
            let index = ["height", "origin", "frames_digest", "source_kind", "source"]
                .iter()
                .position(|x| *x == part)
                .unwrap();
            &mut changed[which][index]
        } else {
            &mut changed[label]
        };
        *field = if let Some(number) = field.as_u64() {
            json!(number + 1)
        } else if matches!(
            label,
            "prev.source" | "next.source" | "gpos" | "da_certificate"
        ) {
            json!(format!("{}01", field.as_str().unwrap()))
        } else {
            let mut bytes = raw(field);
            bytes[0] ^= 1;
            json!(c::hex(&bytes))
        };
        let changed_domain = crate::domain(&changed);
        if v["role"] == "da" {
            let changed_d = Hash32::new(fixed(&changed["commitment"]));
            assert_eq!(changed_domain.da_digest(changed_d).unwrap(), message);
            assert!(
                c::check_da(
                    changed_d,
                    &da,
                    &changed_domain,
                    fixed(&signatures["da_signer"])
                )
                .is_err()
            );
        } else {
            let changed_batch = crate::batch(&changed, &signatures["certifier_signature"]);
            if label == "da_certificate" {
                assert!(changed_domain.batch_digest(&changed_batch).is_err());
            } else {
                assert_eq!(
                    changed_domain.batch_digest(&changed_batch).unwrap(),
                    message
                );
            }
            assert!(
                c::verify_batch(
                    &changed_batch,
                    &changed_domain,
                    fixed(&signatures["certifier_signer"])
                )
                .is_err()
            );
        }
        failures.push(json!({"role":v["role"],"field":v["changed"]}));
    }
    let batch_digest = domain.batch_digest(&original).unwrap();
    assert_eq!(
        batch_digest,
        Hash32::new(fixed(&signatures["batch_digest"]))
    );
    assert_eq!(
        domain.da_digest(commitment).unwrap(),
        Hash32::new(fixed(&signatures["da_digest"]))
    );
    for v in signatures["malformed_signatures"].as_array().unwrap() {
        assert!(c::recover_signer(batch_digest, &raw(&v["signature"])).is_err());
    }
    let mut zero_r = cert.clone();
    zero_r[..32].fill(0);
    assert!(c::recover_signer(batch_digest, &zero_r).is_err());
    let mut high_r = cert.clone();
    high_r[..32].fill(255);
    assert!(c::recover_signer(batch_digest, &high_r).is_err());
    for v in [0, 1, 26, 29, 255] {
        let mut sig = cert.clone();
        sig[64] = v;
        assert!(c::recover_signer(batch_digest, &sig).is_err());
    }
    let key_bytes: [u8; 32] = fs::read(&args[2]).unwrap().try_into().unwrap();
    let key = c::SigningKey::new(key_bytes).unwrap();
    let produced_da = key
        .sign_digest(domain.da_digest(commitment).unwrap())
        .unwrap();
    let produced_cert = key.sign_digest(batch_digest).unwrap();
    assert_eq!(
        produced_da,
        key.sign_digest(domain.da_digest(commitment).unwrap())
            .unwrap()
    );
    let calldata = c::submit_batch_calldata(&original).unwrap();
    println!(
        "{}",
        json!({"status":"PASS","source_revision":"75b574d40930b7c9c748a4c6b6e381117c937537","random_seed":extra["seed"],"positive_frames":positives,"negative_frames":negatives,"maximum_payload_length":encoded.len(),"signature_mutations":failures,"malformed_signatures":signatures["malformed_signatures"].as_array().unwrap().len()+7,"production_signing":{"address":c::hex(&key.address()),"da_digest":c::hex(domain.da_digest(commitment).unwrap().as_bytes()),"da_signature":c::hex(&produced_da),"batch_digest":c::hex(batch_digest.as_bytes()),"batch_signature":c::hex(&produced_cert)},"calldata":c::hex(&calldata)})
    );
}
