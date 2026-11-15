use crate::{
    Error, Id, Profile, Result, WitnessConfig, WitnessMember, WitnessPolicy, hash_parts,
    native_wire::*,
};
use commonware_codec::{Copying, DecodeExt, Encode};
use commonware_consensus::simplex::scheme::bls12381_multisig;
use commonware_cryptography::{
    Signer,
    bls12381::primitives::{
        group::Private,
        ops,
        variant::{MinSig, Variant},
    },
    ed25519::{PrivateKey, PublicKey},
};
use commonware_utils::ordered::BiMap;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub(crate) type NativeScheme = bls12381_multisig::Scheme<PublicKey, MinSig>;
pub(crate) const START: u64 = 1_000_000;
pub(crate) const STOP: i64 = 1_800_000;
pub(crate) const CUTOFF: u64 = 1_900_000;
pub(crate) const PROFILE: &str = "native-engine-fresh-v1";
pub(crate) const WORK_US: u64 = 5_000;
pub(crate) const LINK_US: u64 = 20_000;
pub(crate) const REFRESH_US: u64 = 20_000;
pub(crate) const CLOCK_TRANSIT_US: u64 = 250;
pub(crate) const LEASE_US: u64 = 100_000;
pub(crate) const CAPACITY: usize = 64;
pub(crate) const PARTITION: &str = "i1b-native-voter";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PublicMember {
    pub member: u32,
    pub identity: Vec<u8>,
    pub bls_key: Vec<u8>,
    pub pop: Vec<u8>,
    pub signer_index: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BaseProfile {
    pub profile: String,
    pub package: String,
    pub commonware: String,
    pub scenario: String,
    pub seed: u64,
    pub registry_hash: Id,
    pub epoch: u64,
    pub start: u64,
    pub stop: i64,
    pub cutoff: u64,
    pub epsilon: [i64; 2],
    pub lease: u64,
    pub refresh: u64,
    pub clock_transit: u64,
    pub link_delay: u64,
    pub link_jitter: u64,
    pub queue_capacity: usize,
    pub overflowing: String,
    pub quota_per_second: u32,
    pub mailbox_capacity: usize,
    pub grpc_max_bytes: usize,
    pub payload_bound: usize,
    pub work_us: u64,
    pub payload_size: u32,
    pub virtual_crypto_us: u64,
    pub leader_timeout: u64,
    pub certification_timeout: u64,
    pub timeout_retry: u64,
    pub fetch_timeout: u64,
    pub fetch_retry: u64,
    pub height_limit: u64,
    pub sections: [u64; 8],
    pub view_retention: u64,
    pub scripted: bool,
    pub body_hold_until: Option<u64>,
    pub close_on_work: Option<i32>,
    pub lose_clock_after: Option<u64>,
    pub startup_skew: u64,
    pub stress_tick: Option<u64>,
    pub stress_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Resolved {
    pub base: BaseProfile,
    pub members: Vec<PublicMember>,
    pub config_hash: Id,
    pub run_id: Id,
    pub genesis: Id,
    pub cid: Id,
    pub namespace: Id,
    pub native_config_hash: Id,
    pub app_fixture_hash: Id,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Launch {
    pub resolved: Resolved,
    pub member: u32,
    pub secret: Vec<u8>,
    pub work_dir: PathBuf,
    pub wall_pause_ms: u64,
}

fn err(_: impl std::fmt::Debug) -> Error {
    Error("invalid native fixture configuration")
}

impl Resolved {
    pub fn generate(scenario: &str, seed: u64) -> Result<(Self, Vec<Private>)> {
        if ![7, 42, 99].contains(&seed) {
            return Err(Error("unsupported seed"));
        }
        let (height_limit, scripted, body_hold_until, close_on_work, lose_clock_after) =
            match scenario {
                "healthy" => (2, true, None, None, None),
                "two-real-alone" => (2, false, None, None, None),
                "proof-before-body" => (1, true, Some(1_250_000), None, None),
                "close-propose" => (1, true, None, Some(WorkKind::Propose as i32), None),
                "close-verify" => (1, true, None, Some(WorkKind::Verify as i32), None),
                "close-certify" => (1, true, None, Some(WorkKind::Certify as i32), None),
                "lease-loss" => (2, true, None, None, Some(1_050_000)),
                "wrong-context" | "native-proof-negatives" => (1, true, None, None, None),
                "quota" | "router-overflow" => (1, false, None, None, None),
                _ => return Err(Error("unknown native scenario")),
            };
        let mut rng = ChaCha20Rng::from_seed([42; 32]);
        let mut members = vec![];
        let mut secrets = vec![];
        for member in 0..5 {
            let identity = PrivateKey::decode(Copying(&[16 + member as u8; 32]))
                .map_err(err)?
                .public_key();
            let (private, public) = ops::keypair::<_, MinSig>(&mut rng);
            let pop = ops::sign_proof_of_possession::<MinSig>(&private, b"I1b/storage/pop/v1");
            members.push(PublicMember {
                member,
                identity: identity.encode().to_vec(),
                bls_key: public.encode().to_vec(),
                pop: pop.encode().to_vec(),
                signer_index: 0,
            });
            secrets.push(private);
        }
        let mut order: Vec<_> = members
            .iter()
            .map(|item| {
                Ok((
                    PublicKey::decode(Copying(&item.identity)).map_err(err)?,
                    item.member,
                ))
            })
            .collect::<Result<_>>()?;
        order.sort_unstable();
        for (index, (_, member)) in order.into_iter().enumerate() {
            members[member as usize].signer_index = index as u32;
        }
        let registry_hash = registry_hash(&members);
        let base = BaseProfile {
            profile: PROFILE.into(),
            package: env!("CARGO_PKG_VERSION").into(),
            commonware: "2026.10.0+controlled-spawn-v1".into(),
            scenario: scenario.into(),
            seed,
            registry_hash,
            epoch: 1,
            start: START,
            stop: STOP,
            cutoff: CUTOFF,
            epsilon: [1000, -1000],
            lease: LEASE_US,
            refresh: REFRESH_US,
            clock_transit: CLOCK_TRANSIT_US,
            link_delay: LINK_US,
            link_jitter: 0,
            queue_capacity: CAPACITY,
            overflowing: "drop_newest".into(),
            quota_per_second: if scenario == "router-overflow" { 0 } else { 64 },
            mailbox_capacity: CAPACITY,
            grpc_max_bytes: 1 << 20,
            payload_bound: 1 << 16,
            work_us: WORK_US,
            payload_size: 256,
            virtual_crypto_us: 0,
            leader_timeout: 200_000,
            certification_timeout: 400_000,
            timeout_retry: 100_000,
            fetch_timeout: 100_000,
            fetch_retry: 20_000,
            height_limit,
            sections: [1, 2, 3, 4, 5, 6, 7, 8],
            view_retention: 16,
            scripted,
            body_hold_until,
            close_on_work,
            lose_clock_after,
            startup_skew: 20_000,
            stress_tick: matches!(scenario, "quota" | "router-overflow").then_some(1_024_000),
            stress_count: if matches!(scenario, "quota" | "router-overflow") {
                65
            } else {
                0
            },
        };
        Ok((Self::derive(base, members)?, secrets))
    }
    fn derive(base: BaseProfile, members: Vec<PublicMember>) -> Result<Self> {
        let json = serde_json::to_vec(&base).map_err(err)?;
        let registry = base.registry_hash;
        let config_hash = hash_parts(b"I1b/fresh-config/v1", &[&json, &registry]);
        let run_id = hash_parts(b"I1b/fresh-run/v1", &[&config_hash]);
        let genesis = hash_parts(b"I1b/fresh-genesis/v1", &[&config_hash]);
        let cid = hash_parts(
            b"I1b/fresh-cid/v1",
            &[&run_id, &genesis, &base.epoch.to_be_bytes()],
        );
        let namespace = hash_parts(
            b"I1b/voter-namespace/v1",
            &[&run_id, &config_hash, &cid, &base.epoch.to_be_bytes()],
        );
        let native_config_hash = hash_parts(
            b"I1b/fresh-native-config/v1",
            &[&config_hash, &registry, &namespace],
        );
        let app_fixture_hash = hash_parts(
            b"I1b/fresh-app/v1",
            &[
                &cid,
                &genesis,
                &base.height_limit.to_be_bytes(),
                &base.payload_size.to_be_bytes(),
                &base.work_us.to_be_bytes(),
            ],
        );
        Ok(Self {
            base,
            members,
            config_hash,
            run_id,
            genesis,
            cid,
            namespace,
            native_config_hash,
            app_fixture_hash,
        })
    }

    pub fn validate(&self) -> Result<()> {
        // Verify the public configuration without constructing any signing key.
        if self.base.profile != PROFILE
            || ![7, 42, 99].contains(&self.base.seed)
            || self.members.iter().map(|member| member.member).ne(0..5)
            || registry_hash(&self.members) != self.base.registry_hash
            || Self::derive(self.base.clone(), self.members.clone())? != *self
        {
            return Err(Error("invalid resolved native profile"));
        }
        let registry = self.registry()?;
        let _ = self.witness(0)?;
        for key in &self.members {
            let mark = PublicKey::decode(Copying(&key.identity)).map_err(err)?;
            if registry.keys().get(key.signer_index as usize) != Some(&mark) {
                return Err(Error("invalid native signer index map"));
            }
        }
        Ok(())
    }
    pub fn registry(&self) -> Result<BiMap<PublicKey, <MinSig as Variant>::Public>> {
        let keys = self
            .members
            .iter()
            .map(|member| {
                Ok((
                    PublicKey::decode(Copying(&member.identity)).map_err(err)?,
                    <MinSig as Variant>::Public::decode(Copying(&member.bls_key)).map_err(err)?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        BiMap::try_from(keys).map_err(err)
    }
    pub fn public_keys(&self) -> Result<[PublicKey; 5]> {
        self.members
            .iter()
            .map(|m| PublicKey::decode(Copying(&m.identity)).map_err(err))
            .collect::<Result<Vec<_>>>()?
            .try_into()
            .map_err(err)
    }
    pub fn signer(&self, member: u32, private: Private) -> Result<NativeScheme> {
        let signer = NativeScheme::signer(&self.namespace, self.registry()?, private)
            .ok_or(Error("signing key is not in registry"))?;
        use commonware_cryptography::certificate::Scheme as _;
        if signer.me().and_then(|m| m.signer()).map(|m| m.get())
            != self.members.get(member as usize).map(|m| m.signer_index)
        {
            return Err(Error("private key belongs to a different member"));
        }
        Ok(signer)
    }
    pub fn verifier(&self) -> Result<NativeScheme> {
        Ok(NativeScheme::verifier(&self.namespace, self.registry()?))
    }
    pub fn witness(&self, node: u32) -> Result<WitnessPolicy> {
        let members = self
            .members
            .iter()
            .map(|member| {
                Ok(WitnessMember {
                    member: member.member,
                    key: PublicKey::decode(Copying(&member.identity)).map_err(err)?,
                    bls_key: <MinSig as Variant>::Public::decode(Copying(&member.bls_key))
                        .map_err(err)?,
                    pop: <MinSig as Variant>::Signature::decode(Copying(&member.pop))
                        .map_err(err)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        WitnessPolicy::new(
            WitnessConfig {
                run_id: self.run_id,
                config_hash: self.config_hash,
                cid: self.cid,
                worker: node,
                member: node,
                epoch: self.base.epoch,
                generation: 1,
                partition: PARTITION.into(),
                sections: self.base.sections.to_vec(),
            },
            &members,
        )
    }
    pub fn profile(&self, node: u32) -> Profile {
        Profile {
            run_id: self.run_id,
            config_hash: self.config_hash,
            run_seed: self.base.seed,
            node,
            incarnation: 1,
            epsilon_us: self.base.epsilon[node as usize],
        }
    }
    pub fn binding(&self, node: u32) -> Binding {
        Binding {
            schema_version: 1,
            run_id: self.run_id.to_vec(),
            config_hash: self.config_hash.to_vec(),
            worker_id: node,
            member_id: node,
            incarnation: 1,
        }
    }
    pub fn registration(&self, node: u32) -> RegisterRequest {
        RegisterRequest {
            binding: Some(self.binding(node)),
            registry_hash: self.base.registry_hash.to_vec(),
            namespace_hash: self.namespace.to_vec(),
            profile: PROFILE.into(),
        }
    }
    pub fn start(&self) -> StartEngine {
        StartEngine {
            cid: self.cid.to_vec(),
            epoch: self.base.epoch,
            genesis_digest: self.genesis.to_vec(),
            registry_hash: self.base.registry_hash.to_vec(),
            namespace_hash: self.namespace.to_vec(),
            native_config_hash: self.native_config_hash.to_vec(),
            app_fixture_hash: self.app_fixture_hash.to_vec(),
            height_limit: self.base.height_limit,
            local_stop_us: self.base.stop,
        }
    }
}

fn registry_hash(members: &[PublicMember]) -> Id {
    let fields: Vec<_> = members
        .iter()
        .flat_map(|m| {
            [
                m.member.to_be_bytes().to_vec(),
                m.identity.clone(),
                m.bls_key.clone(),
                m.pop.clone(),
            ]
        })
        .collect();
    hash_parts(
        b"I1b/voter-registry/v1",
        &fields.iter().map(Vec::as_slice).collect::<Vec<_>>(),
    )
}
