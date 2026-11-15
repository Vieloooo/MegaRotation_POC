//! Independent native fresh codec and cryptographic controls.
#[path = "crypto.rs"]
#[allow(dead_code)]
mod crypto;
use commonware_codec::{Copying, DecodeExt, Encode};
use commonware_consensus::{
    simplex::{
        scheme::bls12381_multisig::Scheme,
        types::{Certificate, Finalization, Finalize, Notarization, Notarize, Proposal, Vote},
    },
    types::{Epoch, Round, View},
};
use commonware_cryptography::{
    Signer as _,
    bls12381::primitives::{ops, variant::MinSig},
    ed25519::{PrivateKey, PublicKey},
    sha256::Digest,
};
use commonware_parallel::Sequential;
use commonware_utils::{
    iter::NonEmpty,
    ordered::{BiMap, Quorum},
};
use poc_native_runtime::{native_wire as p, native_wire_support};
use prost::Message;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use serde_json::{Value, json};
type S = Scheme<PublicKey, MinSig>;
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn fixture() -> (Vec<S>, S, Value, Vec<u8>) {
    let namespace = vec![0x74; 32];
    let mut rng = ChaCha20Rng::from_seed([42; 32]);
    let mut keys = Vec::new();
    let mut members = Vec::new();
    let mut secrets = Vec::new();
    for member in 0..5u32 {
        let identity = PrivateKey::decode(Copying(&[16 + member as u8; 32]))
            .unwrap()
            .public_key();
        let (secret, public) = ops::keypair::<_, MinSig>(&mut rng);
        let pop = ops::sign_proof_of_possession::<MinSig>(&secret, b"I1b/storage/pop/v1");
        keys.push((identity.clone(), public.clone()));
        members.push(json!({"member":member,"identity":identity.encode().to_vec(),"bls_key":public.encode().to_vec(),"pop":pop.encode().to_vec(),"signer_index":0}));
        secrets.push(secret);
    }
    let registry = BiMap::try_from(keys).unwrap();
    for member in &mut members {
        let identity = PublicKey::decode(Copying(
            &serde_json::from_value::<Vec<u8>>(member["identity"].clone()).unwrap(),
        ))
        .unwrap();
        member["signer_index"] = json!(registry.keys().index(&identity).unwrap().get());
    }
    let verifier = S::verifier(&namespace, registry.clone());
    let signers = secrets
        .into_iter()
        .map(|secret| S::signer(&namespace, registry.clone(), secret).unwrap())
        .collect();
    (signers, verifier, json!(members), namespace)
}

#[test]
fn independent_hash_framing_literal() {
    assert_eq!(
        crypto::hash(
            "I1b/fresh-tie/v1",
            &[&7u64.to_be_bytes(), &2u64.to_be_bytes()]
        ),
        [
            0xaa, 0x58, 0xf1, 0x89, 0x96, 0x53, 0xc6, 0xb9, 0xeb, 0x0d, 0x65, 0xec, 0x5a, 0xb5,
            0x2e, 0x89, 0x68, 0x19, 0xc4, 0xd5, 0x1f, 0x17, 0x0a, 0x9a, 0x59, 0x6c, 0x21, 0xbd,
            0x99, 0x7b, 0x87, 0x5f
        ]
    );
}

#[test]
fn native_three_four_distinct_quorum_and_wrong_domain_kind_epoch_controls() {
    let (signers, verifier, members, namespace) = fixture();
    let mut oracle = crypto::Oracle::new();
    oracle
        .dispatch(&json!({"op":"init","members":members,"namespace":namespace,"epoch":1}))
        .unwrap();
    let proposal = Proposal::new(
        Round::new(Epoch::new(1), View::new(1)),
        View::new(0),
        Digest([0x44; 32]),
    );
    let votes: Vec<_> = signers
        .iter()
        .map(|s| Notarize::sign(s, proposal.clone()).unwrap())
        .collect();
    assert!(
        Notarization::from_notarizes(
            &verifier,
            NonEmpty::new(&votes[0], votes[1..3].iter()),
            &Sequential
        )
        .is_err()
    );
    // A mathematically valid aggregate of THREE distinct signatures must still
    // fail the native four-member certificate threshold during verification.
    use commonware_cryptography::{
        bls12381::{
            certificate::multisig::Certificate as AggregateCertificate,
            primitives::{ops::aggregate, variant::Variant},
        },
        certificate::Signers,
    };
    let signatures: Vec<_> = votes[..3]
        .iter()
        .map(|v| v.attestation.signature.get().unwrap())
        .collect();
    let aggregate = aggregate::combine_signatures::<MinSig, _>(NonEmpty::new(
        signatures[0],
        signatures[1..].iter().copied(),
    ));
    let publics: Vec<_> = members.as_array().unwrap()[..3]
        .iter()
        .map(|m| {
            <MinSig as Variant>::Public::decode(Copying(
                &serde_json::from_value::<Vec<u8>>(m["bls_key"].clone()).unwrap(),
            ))
            .unwrap()
        })
        .collect();
    let public = aggregate::combine_public_keys::<MinSig, _>(NonEmpty::new(
        &publics[0],
        publics[1..].iter(),
    ));
    let mut domain = namespace.clone();
    domain.extend_from_slice(b"_NOTARIZE");
    assert!(
        aggregate::verify_same_message::<MinSig>(&public, &domain, &proposal.encode(), &aggregate)
            .is_ok()
    );
    let low = AggregateCertificate::<MinSig> {
        signers: Signers::new(5, votes[..3].iter().map(|v| v.attestation.signer)).unwrap(),
        signature: aggregate.into(),
    };
    let low = Certificate::Notarization(Notarization::<S, Digest> {
        proposal: proposal.clone(),
        certificate: low,
    })
    .encode()
    .to_vec();
    let low = oracle
        .dispatch(&json!({"op":"certificate","bytes":low}))
        .unwrap();
    assert_eq!(low["valid"], false);
    assert_eq!(low["signers"].as_array().unwrap().len(), 3);
    let duplicate = [&votes[0], &votes[1], &votes[2], &votes[2]];
    assert!(
        Notarization::from_notarizes(
            &verifier,
            NonEmpty::new(duplicate[0], duplicate[1..].iter().copied()),
            &Sequential
        )
        .is_err()
    );
    // Four members excluding member0 form a valid native certificate.
    let valid = Notarization::from_notarizes(
        &verifier,
        NonEmpty::new(&votes[1], votes[2..5].iter()),
        &Sequential,
    )
    .unwrap();
    let bytes = Certificate::Notarization(valid.clone()).encode().to_vec();
    let decoded = oracle
        .dispatch(&json!({"op":"certificate","bytes":bytes}))
        .unwrap();
    assert_eq!(decoded["valid"], true);
    assert_eq!(decoded["signers"].as_array().unwrap().len(), 4);
    let wrong_kind = Certificate::Finalization(Finalization::<S, Digest> {
        proposal: valid.proposal.clone(),
        certificate: valid.certificate.clone(),
    })
    .encode()
    .to_vec();
    assert_eq!(
        oracle
            .dispatch(&json!({"op":"certificate","bytes":wrong_kind}))
            .unwrap()["valid"],
        false
    );
    let vote = Vote::Notarize(votes[0].clone()).encode().to_vec();
    assert_eq!(
        oracle.dispatch(&json!({"op":"vote","bytes":vote})).unwrap()["valid"],
        true
    );
    oracle
        .dispatch(&json!({"op":"init","members":members,"namespace":vec![0x75;32],"epoch":1}))
        .unwrap();
    assert_eq!(
        oracle
            .dispatch(&json!({"op":"certificate","bytes":bytes}))
            .unwrap()["valid"],
        false
    );
    oracle
        .dispatch(&json!({"op":"init","members":members,"namespace":namespace,"epoch":2}))
        .unwrap();
    assert_eq!(
        oracle.dispatch(&json!({"op":"vote","bytes":vote})).unwrap()["valid"],
        false
    );
    let mut wrong_pop = members.clone();
    wrong_pop[0]["pop"] = wrong_pop[1]["pop"].clone();
    assert!(
        oracle
            .dispatch(&json!({"op":"init","members":wrong_pop,"namespace":namespace,"epoch":1}))
            .is_err()
    );
    let mut wrong_index = members.clone();
    wrong_index[0]["signer_index"] = wrong_index[1]["signer_index"].clone();
    assert!(
        oracle
            .dispatch(&json!({"op":"init","members":wrong_index,"namespace":namespace,"epoch":1}))
            .is_err()
    );
}

#[test]
fn native_attestation_mapping_and_malformed_wire_controls() {
    let (signers, _, members, namespace) = fixture();
    let mut oracle = crypto::Oracle::new();
    oracle
        .dispatch(&json!({"op":"init","members":members,"namespace":namespace,"epoch":1}))
        .unwrap();
    let proposal = Proposal::new(
        Round::new(Epoch::new(1), View::new(2)),
        View::new(1),
        Digest([0x35; 32]),
    );
    let vote = Finalize::sign(&signers[1], proposal.clone()).unwrap();
    let q = json!({"op":"attestation","kind":3,"bytes":vote.attestation.encode().to_vec(),"subject":proposal.encode().to_vec()});
    let value = oracle.dispatch(&q).unwrap();
    assert_eq!(value["valid"], true);
    assert_eq!(value["signer_index"], members[1]["signer_index"]);
    assert_eq!(value["parent_view"], 1);
    let mut wrong = q.clone();
    wrong["kind"] = json!(1);
    assert_eq!(oracle.dispatch(&wrong).unwrap()["valid"], false);
    for raw in [vec![], vec![99], vec![0, 0xff, 0xff, 0xff], {
        let mut b = Vote::Finalize(vote.clone()).encode().to_vec();
        b.push(0);
        b
    }] {
        let outcome = oracle.dispatch(&json!({"op":"vote","bytes":raw}));
        assert!(outcome.is_err() || outcome.unwrap()["valid"] == false);
    }
}

#[test]
fn block_and_proto_hash_literals_exclude_only_declared_self_hashes() {
    let mut parent = vec![0x44; 32];
    let mut expected = vec![
        "a136eba973294e8e10edbe13e76fcdd722d6dfc74df0ab23e356aff38aad9122",
        "a30dd7cfc7214f925987ac7b2506727e38cc1b1e04820dc9d00598531e21f1be",
    ]
    .into_iter();
    for height in [1u64, 2] {
        let mut raw = vec![1];
        raw.extend_from_slice(&[0x33; 32]);
        raw.extend_from_slice(&height.to_be_bytes());
        raw.extend_from_slice(&parent);
        raw.extend_from_slice(&256u32.to_be_bytes());
        raw.extend((0..256u16).map(|i| (height as u8).wrapping_add(i as u8)));
        assert_eq!(raw.len(), 333);
        parent = crypto::hash("I1b/fixture-block/v1", &[&raw]).to_vec();
        assert_eq!(hex(&parent), expected.next().unwrap());
    }
    let binding = p::Binding {
        schema_version: 1,
        run_id: vec![0x11; 32],
        config_hash: vec![0x22; 32],
        worker_id: 0,
        member_id: 0,
        incarnation: 1,
    };
    let mut grant = p::Grant {
        binding: Some(binding.clone()),
        logical_step_id: 1,
        event_id: 2,
        tick_us: 1000000,
        causal_wave: 0,
        tie_rank: crypto::hash(
            "I1b/fresh-tie/v1",
            &[&7u64.to_be_bytes(), &2u64.to_be_bytes()],
        )
        .to_vec(),
        input: Some(p::Input {
            kind: Some(p::input::Kind::RequestSample(p::Empty {})),
        }),
        grant_hash: vec![],
    };
    let expected = crypto::hash("I1b/fresh-grant/v1", &[&grant.encode_to_vec()]);
    assert_eq!(
        hex(&expected),
        "87b4e717fd216c521a4e48f29658db63b365fadb548dc515d813846bab69f11d"
    );
    assert_eq!(native_wire_support::grant_hash(&grant), expected);
    grant.grant_hash = vec![1; 32];
    assert_eq!(native_wire_support::grant_hash(&grant), expected);
    let mut ack = p::Ack {
        binding: Some(binding),
        logical_step_id: 1,
        event_id: 2,
        grant_hash: expected.to_vec(),
        status: 1,
        effects: vec![p::Effect {
            kind: Some(p::effect::Kind::ClockRequest(p::ClockRequest {
                request_id: 1,
            })),
        }],
        runtime_events_json: b"[]".to_vec(),
        application_events_json: b"[]".to_vec(),
        witness_receipt: Some(p::Receipt {
            sequence: 3,
            hash: vec![0x55; 32],
        }),
        durable_step_sequence: 1,
        ..Default::default()
    };
    let expected = crypto::hash("I1b/fresh-ack/v1", &[&ack.encode_to_vec()]);
    assert_eq!(
        hex(&expected),
        "d517471ea2fdb03b5282f88527b91143a845d146c73986dcc602e67c1469fa67"
    );
    assert_eq!(native_wire_support::ack_hash(&ack), expected);
    ack.durable_step_hash = vec![7; 32];
    ack.logical_ack_hash = vec![8; 32];
    assert_eq!(native_wire_support::ack_hash(&ack), expected);
    ack.durable_step_sequence = 2;
    assert_ne!(native_wire_support::ack_hash(&ack), expected);
}

#[test]
fn body_protobuf_unknown_fields_and_repeated_message_merge_semantics() {
    let mut oracle = crypto::Oracle::new();
    let raw = vec![0x0a, 3, 0x0a, 1, 0x33, 0x0a, 4, 0x12, 2, 1, 2, 0x98, 6, 1];
    let decoded = oracle.dispatch(&json!({"op":"body","bytes":raw})).unwrap();
    assert_eq!(decoded["kind"]["Push"]["cid"], json!([0x33]));
    assert_eq!(decoded["kind"]["Push"]["block_bytes"], json!([1, 2]));
    let decoded = oracle
        .dispatch(&json!({"op":"body","bytes":[0x0a,0,0x12,2,8,1]}))
        .unwrap();
    assert_eq!(decoded["kind"]["Request"]["request_id"], 1);
}
