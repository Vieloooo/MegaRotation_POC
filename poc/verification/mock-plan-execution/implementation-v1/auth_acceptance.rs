//! Independent registry and authorization acceptance using only public test keys.
#[path = "component_fixture.rs"]
mod fixture;
use commonware_codec::{DecodeExt, Encode};
use commonware_cryptography::{
    Signer,
    bls12381::primitives::{ops, variant::MinSig},
    ed25519,
};
use poc_authorization::{
    POP_NAMESPACE, PlanOutcome, StaticScheduleProvider, ValidatedRegistry, schedule,
};
use poc_types::domain::*;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use serde_json::Value;
use sha2::{Digest, Sha256};

fn public_record() -> (RegistryRecord, Vec<[u8; 48]>, Vec<[u8; 48]>) {
    let mut entries = Vec::new();
    let mut wrong_namespaces = Vec::new();
    let mut wrong_kinds = Vec::new();
    for pid in 0..15u8 {
        let identity = ed25519::PrivateKey::from_seed(303000 + u64::from(pid));
        let mut rng = ChaCha20Rng::seed_from_u64(303100 + u64::from(pid));
        let (private, public) = ops::keypair::<_, MinSig>(&mut rng);
        let pop = ops::sign_proof_of_possession::<MinSig>(&private, b"PoC/Registry/PoP/v1");
        wrong_namespaces.push(
            ops::sign_proof_of_possession::<MinSig>(&private, b"PoC/Registry/PoP/v0")
                .encode()
                .as_ref()
                .try_into()
                .unwrap(),
        );
        wrong_kinds.push(
            ops::sign_message::<MinSig>(&private, b"PoC/Registry/PoP/v1", &public.encode())
                .encode()
                .as_ref()
                .try_into()
                .unwrap(),
        );
        entries.push(RegistryEntry {
            pid: Pid::new(pid).unwrap(),
            identity_key: identity.public_key().encode().as_ref().try_into().unwrap(),
            vote_key: public.encode().as_ref().try_into().unwrap(),
            pop: pop.encode().as_ref().try_into().unwrap(),
        });
    }
    (RegistryRecord { entries }, wrong_namespaces, wrong_kinds)
}
fn independently_framed_hash(domain: &str, bytes: &[u8]) -> Hash32 {
    let mut h = Sha256::new();
    h.update((domain.len() as u64).to_be_bytes());
    h.update(domain.as_bytes());
    h.update((bytes.len() as u64).to_be_bytes());
    h.update(bytes);
    Hash32::new(h.finalize().into())
}
fn independently_encoded_registry(record: &RegistryRecord) -> Vec<u8> {
    let mut bytes = vec![0, 1];
    bytes.extend_from_slice(&15u32.to_be_bytes());
    for entry in &record.entries {
        bytes.push(entry.pid.get());
        bytes.extend_from_slice(&entry.identity_key);
        bytes.extend_from_slice(&entry.vote_key);
        bytes.extend_from_slice(&entry.pop);
    }
    bytes
}
struct Reference(InputStatus<Anchor>);
impl PublicReferenceView for Reference {
    fn reference(&self, _: &BlockRef) -> InputStatus<Anchor> {
        self.0.clone()
    }
}
fn ready(outcome: PlanOutcome) -> PlanRecord {
    match outcome {
        PlanOutcome::Ready(plan) => plan,
        other => panic!("expected ready: {other:?}"),
    }
}
fn setup() -> (StaticScheduleProvider, PublicConfig, Reference) {
    let registry = ValidatedRegistry::load(public_record().0).unwrap();
    let mut config = fixture::config();
    config.registry_hash = registry.digest();
    let reference = Reference(InputStatus::Ready(config.genesis.clone()));
    (
        StaticScheduleProvider::new(config.clone(), registry).unwrap(),
        config,
        reference,
    )
}
fn input(config: &PublicConfig, session: SessionId) -> PlanInput {
    PlanInput {
        cause: BoundaryCause::Genesis,
        session,
        source_region: None,
        window: Window {
            h_min: 1,
            h_max: 10,
        },
        reference_block: config.genesis.block.clone(),
    }
}

#[test]
fn real_pop_registry_hash_normalization_and_identity_index_mapping() {
    assert_eq!(POP_NAMESPACE, b"PoC/Registry/PoP/v1");
    let (record, _, _) = public_record();
    let registry = ValidatedRegistry::load(record.clone()).unwrap();
    let expected =
        independently_framed_hash("PoC/Registry/v1", &independently_encoded_registry(&record));
    assert_eq!(registry.digest(), expected);
    assert_eq!(registry.record(), &record);
    let mut reordered = record.clone();
    reordered.entries.reverse();
    assert_eq!(
        ValidatedRegistry::load(reordered).unwrap().digest(),
        expected
    );
    for r in 0..3u8 {
        let region = RegionId::new(r).unwrap();
        let mut entries: Vec<_> = record
            .entries
            .iter()
            .filter(|e| e.pid.region() == region)
            .collect();
        entries.sort_by_key(|e| e.identity_key);
        assert_eq!(registry.participants(region).len(), 5);
        for (index, entry) in entries.iter().enumerate() {
            let idx = SignerIndex::new(index as u8).unwrap();
            assert_eq!(registry.signer_index(entry.pid), idx);
            assert_eq!(registry.pid_at(region, idx), entry.pid);
            let identity = ed25519::PublicKey::decode(entry.identity_key.as_slice()).unwrap();
            assert_eq!(registry.pid_for_identity(&identity), Some(entry.pid));
        }
    }
}

#[test]
fn registry_rejects_wrong_pop_key_kind_namespace_and_duplicates() {
    let (record, wrong_namespace, wrong_kind) = public_record();
    for pop in [wrong_namespace[0], wrong_kind[0], record.entries[1].pop] {
        let mut bad = record.clone();
        bad.entries[0].pop = pop;
        assert!(ValidatedRegistry::load(bad).is_err());
    }
    let mut bad = record.clone();
    bad.entries[0].pop[7] ^= 1;
    assert!(ValidatedRegistry::load(bad).is_err());
    let mut bad = record.clone();
    bad.entries[14].identity_key = bad.entries[13].identity_key;
    assert!(ValidatedRegistry::load(bad).is_err());
    let mut bad = record.clone();
    bad.entries[14].vote_key = bad.entries[13].vote_key;
    bad.entries[14].pop = bad.entries[13].pop;
    assert!(ValidatedRegistry::load(bad).is_err());
    let mut bad = record.clone();
    bad.entries[14].pid = bad.entries[13].pid;
    assert!(ValidatedRegistry::load(bad).is_err());
    let mut bad = record;
    bad.entries.pop();
    assert!(ValidatedRegistry::load(bad).is_err());
}

#[test]
fn strict_edwards_and_bls_zero_infinity_negatives_reach_actual_loader() {
    let (record, _, _) = public_record();
    let vectors: Value =
        serde_json::from_str(include_str!("registry-negative-vectors.json")).unwrap();
    for case in vectors["identity_cases"].as_array().unwrap() {
        let mut changed = record.clone();
        changed.entries[0].identity_key =
            fixture::unhex(case["identity_key_hex"].as_str().unwrap())
                .try_into()
                .unwrap();
        assert_eq!(
            ValidatedRegistry::load(changed).is_ok(),
            case["expected_accept"].as_bool().unwrap(),
            "{}",
            case["id"]
        );
    }
    for case in vectors["bls_cases"].as_array().unwrap() {
        let mut changed = record.clone();
        let raw = fixture::unhex(case["hex"].as_str().unwrap());
        if case["target"] == "vote_key" {
            changed.entries[0].vote_key = raw.try_into().unwrap();
        } else {
            changed.entries[0].pop = raw.try_into().unwrap();
        }
        assert!(ValidatedRegistry::load(changed).is_err(), "{}", case["id"]);
    }
    let json = serde_json::to_value(&record.entries[0]).unwrap();
    for (field, lengths) in [
        ("identity_key", vec![31, 33]),
        ("vote_key", vec![95, 97]),
        ("pop", vec![47, 49]),
    ] {
        for length in lengths {
            let mut changed = json.clone();
            changed[field] = serde_json::json!(vec![0u8; length]);
            assert!(serde_json::from_value::<RegistryEntry>(changed).is_err());
        }
    }
    let mut weight = json;
    weight["weight"] = serde_json::json!(1);
    assert!(serde_json::from_value::<RegistryEntry>(weight).is_err());
}

#[test]
fn schedule_boundaries_and_actual_plan_membership_are_deterministic() {
    let v: Value = serde_json::from_str(include_str!("../semantic-vectors.json")).unwrap();
    for c in v["schedule"].as_array().unwrap() {
        let s = SessionId {
            beta: c["input"][0].as_u64().unwrap(),
            r: c["input"][1].as_u64().unwrap(),
        };
        let (r, p) = schedule(&s, c["input"][2].as_u64().unwrap()).unwrap();
        assert_eq!(r.get() as u64, c["expected"][0].as_u64().unwrap());
        assert_eq!(p.get() as u64, c["expected"][2].as_u64().unwrap());
    }
    assert!(schedule(&SessionId { beta: 0, r: 0 }, 0).is_err());
    let (provider, config, reference) = setup();
    for r in 0..3u64 {
        let plan = ready(provider.resolve(&input(&config, SessionId { beta: 0, r }), &reference));
        assert_eq!(plan.region.get() as u64, r);
        assert_eq!(plan.total_weight, 5);
        assert_eq!(plan.fault_bound, 1);
        for view in 1..=11 {
            assert_eq!(
                provider.leader(&plan, view).unwrap().get() as u64,
                5 * r + (view - 1) % 5
            );
        }
        assert!(provider.leader(&plan, 0).is_err());
        assert_eq!(ready(provider.validate_plan(&plan, &reference)), plan);
    }
}

#[test]
fn public_reference_missing_conflicting_and_tampered_plan_are_rejected() {
    let (provider, config, reference) = setup();
    let query = input(&config, SessionId { beta: 0, r: 0 });
    let missing = Reference(InputStatus::Wait(InputMissing::Reference(
        query.reference_block.clone(),
    )));
    assert!(matches!(
        provider.resolve(&query, &missing),
        PlanOutcome::Wait(_)
    ));
    assert!(matches!(
        provider.resolve(
            &query,
            &Reference(InputStatus::Reject(InputError::InvalidSource))
        ),
        PlanOutcome::Reject(_)
    ));
    let mut wrong = config.genesis.clone();
    wrong.block.block_id = Hash32::new([0x99; 32]);
    assert!(matches!(
        provider.resolve(&query, &Reference(InputStatus::Ready(wrong))),
        PlanOutcome::Reject(_)
    ));
    let plan = ready(provider.resolve(&query, &reference));
    let mut changed = plan.clone();
    changed.authorization_ref = Hash32::new([0; 32]);
    assert!(matches!(
        provider.validate_plan(&changed, &reference),
        PlanOutcome::Reject(_)
    ));
    let mut changed = plan.clone();
    changed.committee[0].vote_key[3] ^= 1;
    assert!(matches!(
        provider.validate_plan(&changed, &reference),
        PlanOutcome::Reject(_)
    ));
    let mut changed = plan;
    changed.config_ref = ConfigRef::new(Hash32::new([0; 32]));
    assert!(matches!(
        provider.validate_plan(&changed, &reference),
        PlanOutcome::Reject(_)
    ));
    let mut invalid = query;
    invalid.window.h_max = 9;
    assert!(matches!(
        provider.resolve(&invalid, &reference),
        PlanOutcome::Reject(_)
    ));
}

#[test]
fn regular_reference_and_production_anchor_remain_separate_and_cid_bound() {
    let (provider, config, reference) = setup();
    let query = PlanInput {
        cause: BoundaryCause::Handoff,
        session: SessionId { beta: 0, r: 1 },
        source_region: Some(RegionId::new(0).unwrap()),
        window: Window {
            h_min: 11,
            h_max: 20,
        },
        reference_block: config.genesis.block.clone(),
    };
    let plan = ready(provider.resolve(&query, &reference));
    let a = Anchor {
        block: BlockRef {
            height: 9,
            origin: 7,
            block_id: Hash32::new([0x44; 32]),
        },
        timestamp_us: 19000000,
    };
    let mut b = a.clone();
    b.block.block_id = Hash32::new([0x55; 32]);
    let da = provider.descriptor(&plan, &a).unwrap();
    let db = provider.descriptor(&plan, &b).unwrap();
    assert_eq!(da.plan, db.plan);
    assert_eq!(da.plan.reference_block.height, 0);
    assert_eq!(
        da.plan.window,
        Window {
            h_min: 11,
            h_max: 20
        }
    );
    assert_ne!(da.cid, db.cid);
    assert!(provider.validate_descriptor(&da).is_ok());
    assert!(provider.validate_descriptor(&db).is_ok());
    let mut wrong = da;
    wrong.cid = db.cid;
    assert!(provider.validate_descriptor(&wrong).is_err());
    let bad = Anchor {
        block: BlockRef {
            height: 11,
            ..a.block
        },
        timestamp_us: 21000000,
    };
    assert!(provider.descriptor(&plan, &bad).is_err());
}
