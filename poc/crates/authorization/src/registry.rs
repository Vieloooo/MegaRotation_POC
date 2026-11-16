use commonware_codec::{Copying, DecodeExt};
use commonware_cryptography::{
    bls12381::primitives::{
        group::{G1, G2},
        ops,
        variant::MinSig,
    },
    ed25519::PublicKey,
};
use commonware_utils::ordered::BiMap;
use curve25519_dalek::edwards::CompressedEdwardsY;
use poc_types::domain::{
    CommitteeEntry, DomainError, Hash32, Pid, RegionId, RegistryEntry, RegistryRecord, SignerIndex,
};
use std::collections::{BTreeMap, BTreeSet};

pub const POP_NAMESPACE: &[u8] = b"PoC/Registry/PoP/v1";
pub type ParticipantMap = BiMap<PublicKey, G2>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryError {
    Malformed(DomainError),
    DuplicateIdentity { pid: Pid },
    DuplicateVoteKey { pid: Pid },
    InvalidIdentity { pid: Pid },
    InvalidVoteKey { pid: Pid },
    InvalidProof { pid: Pid },
}

impl From<DomainError> for RegistryError {
    fn from(error: DomainError) -> Self {
        Self::Malformed(error)
    }
}

/// The input record, checked keys, and native participant ordering share one construction path.
#[derive(Clone, Debug)]
pub struct ValidatedRegistry {
    record: RegistryRecord,
    digest: Hash32,
    identities: BTreeMap<PublicKey, Pid>,
    participants: [ParticipantMap; 3],
    by_signer: [[Pid; 5]; 3],
    signer_indices: [SignerIndex; 15],
}

impl ValidatedRegistry {
    pub fn load(mut record: RegistryRecord) -> Result<Self, RegistryError> {
        record.entries.sort_unstable_by_key(|entry| entry.pid);
        record.validate()?;

        let mut identities = BTreeMap::new();
        let mut vote_bytes = BTreeSet::new();
        let mut by_region: [Vec<(PublicKey, G2)>; 3] = std::array::from_fn(|_| Vec::new());

        for entry in &record.entries {
            let pid = entry.pid;
            let identity = strict_identity(entry)?;
            if identities.insert(identity.clone(), pid).is_some() {
                return Err(RegistryError::DuplicateIdentity { pid });
            }
            if !vote_bytes.insert(entry.vote_key) {
                return Err(RegistryError::DuplicateVoteKey { pid });
            }

            let vote_key = G2::decode(Copying(&entry.vote_key))
                .map_err(|_| RegistryError::InvalidVoteKey { pid })?;
            let proof =
                G1::decode(Copying(&entry.pop)).map_err(|_| RegistryError::InvalidProof { pid })?;
            ops::verify_proof_of_possession::<MinSig>(&vote_key, POP_NAMESPACE, &proof)
                .map_err(|_| RegistryError::InvalidProof { pid })?;
            by_region[usize::from(pid.region().get())].push((identity, vote_key));
        }

        let participants = by_region
            .into_iter()
            .map(|entries| {
                ParticipantMap::try_from(entries).map_err(|_| malformed("participant map"))
            })
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| malformed("region count"))?;
        let mut by_signer = Vec::with_capacity(3);
        let mut signer_indices = [SignerIndex::new(0)?; 15];
        for region_map in &participants {
            let map: &ParticipantMap = region_map;
            let mut region_pids = Vec::with_capacity(5);
            // Read Commonware's actual key order instead of prescribing an independent sort.
            for (index, identity) in map.keys().iter().enumerate() {
                let pid = *identities
                    .get(identity)
                    .ok_or(malformed("missing identity"))?;
                region_pids.push(pid);
                signer_indices[usize::from(pid.get())] = SignerIndex::new(index as u8)?;
            }
            by_signer.push(
                region_pids
                    .try_into()
                    .map_err(|_| malformed("participant count"))?,
            );
        }

        Ok(Self {
            digest: record.digest()?,
            record,
            identities,
            participants,
            by_signer: by_signer
                .try_into()
                .map_err(|_| malformed("region count"))?,
            signer_indices,
        })
    }

    pub fn record(&self) -> &RegistryRecord {
        &self.record
    }

    pub fn digest(&self) -> Hash32 {
        self.digest
    }

    pub fn entry(&self, pid: Pid) -> &RegistryEntry {
        &self.record.entries[usize::from(pid.get())]
    }

    pub fn participants(&self, region: RegionId) -> &ParticipantMap {
        &self.participants[usize::from(region.get())]
    }

    pub fn signer_index(&self, pid: Pid) -> SignerIndex {
        self.signer_indices[usize::from(pid.get())]
    }

    pub fn pid_at(&self, region: RegionId, index: SignerIndex) -> Pid {
        self.by_signer[usize::from(region.get())][usize::from(index.get())]
    }

    pub fn pid_for_identity(&self, key: &PublicKey) -> Option<Pid> {
        self.identities.get(key).copied()
    }

    pub fn committee(&self, region: RegionId) -> Vec<CommitteeEntry> {
        let start = usize::from(region.get()) * 5;
        self.record.entries[start..start + 5]
            .iter()
            .map(|entry| CommitteeEntry {
                pid: entry.pid,
                identity_key: entry.identity_key,
                vote_key: entry.vote_key,
                weight: 1,
            })
            .collect()
    }
}

fn strict_identity(entry: &RegistryEntry) -> Result<PublicKey, RegistryError> {
    let invalid = RegistryError::InvalidIdentity { pid: entry.pid };
    let point = CompressedEdwardsY(entry.identity_key)
        .decompress()
        .ok_or_else(|| invalid.clone())?;
    if point.compress().to_bytes() != entry.identity_key
        || point.is_small_order()
        || !point.is_torsion_free()
    {
        return Err(invalid);
    }
    PublicKey::decode(Copying(&entry.identity_key)).map_err(|_| invalid)
}

fn malformed(reason: &'static str) -> RegistryError {
    RegistryError::Malformed(DomainError::Malformed { reason })
}
