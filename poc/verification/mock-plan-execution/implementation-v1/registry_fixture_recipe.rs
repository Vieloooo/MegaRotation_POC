//! Independent public fixture recipe using pinned upstream primitives only.
//! Test-only deterministic seeds; private bytes are never serialized or printed.
use commonware_codec::Encode;
use commonware_cryptography::{
    Signer,
    bls12381::primitives::{ops, variant::MinSig},
    ed25519,
};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn generate_public_registry_fixture_from_primitives() {
    let mut entries = Vec::new();
    for pid in 0..15u8 {
        let identity = ed25519::PrivateKey::from_seed(303_000 + u64::from(pid));
        let mut rng = ChaCha20Rng::seed_from_u64(303_100 + u64::from(pid));
        let (vote_private, vote_public) = ops::keypair::<_, MinSig>(&mut rng);
        let pop = ops::sign_proof_of_possession::<MinSig>(&vote_private, b"PoC/Registry/PoP/v1");
        let wrong_namespace =
            ops::sign_proof_of_possession::<MinSig>(&vote_private, b"PoC/Registry/PoP/v0");
        let wrong_kind = ops::sign_message::<MinSig>(
            &vote_private,
            b"PoC/Registry/PoP/v1",
            &vote_public.encode(),
        );
        assert!(
            ops::verify_proof_of_possession::<MinSig>(&vote_public, b"PoC/Registry/PoP/v1", &pop)
                .is_ok()
        );
        assert!(
            ops::verify_proof_of_possession::<MinSig>(
                &vote_public,
                b"PoC/Registry/PoP/v1",
                &wrong_namespace
            )
            .is_err()
        );
        assert!(
            ops::verify_proof_of_possession::<MinSig>(
                &vote_public,
                b"PoC/Registry/PoP/v1",
                &wrong_kind
            )
            .is_err()
        );
        entries.push(serde_json::json!({
            "pid": pid,
            "identity_key": hex(&identity.public_key().encode()),
            "vote_key": hex(&vote_public.encode()),
            "pop": hex(&pop.encode()),
            "wrong_namespace_pop": hex(&wrong_namespace.encode()),
            "wrong_kind_signature": hex(&wrong_kind.encode())
        }));
    }
    let value = serde_json::json!({
        "schema": "public-registry-primitive-fixture-v1",
        "commonware": "2026.10.0",
        "namespace": "PoC/Registry/PoP/v1",
        "identity_seed_base": 303000,
        "vote_seed_base": 303100,
        "rng": "rand_chacha::ChaCha20Rng::seed_from_u64, per pid",
        "security": "Deterministic public test fixture; never use these keys for production",
        "entries": entries
    });
    println!("REGISTRY_JSON={}", serde_json::to_string(&value).unwrap());
}
