use crate::{Error, Id, Result, hash_parts};
use commonware_codec::{Copying, Decode, Encode, ReadExt, varint::UInt};
use commonware_consensus::{
    Epochable, Viewable,
    simplex::{
        scheme::bls12381_multisig::Scheme,
        types::{Artifact, Certificate, Vote},
    },
};
use commonware_cryptography::{
    Crc32,
    bls12381::primitives::{
        ops,
        variant::{MinSig, Variant},
    },
    certificate::Verifier as _,
    ed25519::PublicKey,
    sha256::Digest,
};
use commonware_parallel::Sequential;
use commonware_utils::ordered::{BiMap, Quorum};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PAGE_BYTES: usize = 256;
pub const DATA_BYTES: usize = 244;
pub const MAX_BLOB_BYTES: usize = 65_536;
const MAX_ARTIFACT_BYTES: usize = 4096;
const MAX_FRAMES: usize = 1024;

type NativeScheme = Scheme<PublicKey, MinSig>;
type NativeArtifact = Artifact<NativeScheme, Digest>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessConfig {
    pub run_id: Id,
    pub config_hash: Id,
    pub cid: Id,
    pub worker: u32,
    pub member: u32,
    pub epoch: u64,
    pub generation: u64,
    pub partition: String,
    pub sections: Vec<u64>,
}

pub struct WitnessMember {
    pub member: u32,
    pub key: PublicKey,
    pub bls_key: <MinSig as Variant>::Public,
    pub pop: <MinSig as Variant>::Signature,
}

#[derive(Clone)]
pub struct WitnessPolicy {
    pub(crate) seal: PolicySeal,
    scheme: NativeScheme,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicySeal {
    pub config: WitnessConfig,
    pub registry_hash: Id,
    pub namespace_hash: Id,
    pub local_signer: u32,
}

impl WitnessConfig {
    pub fn voting_namespace(&self) -> Id {
        hash_parts(
            b"I1b/voter-namespace/v1",
            &[
                &self.run_id,
                &self.config_hash,
                &self.cid,
                &self.epoch.to_be_bytes(),
            ],
        )
    }
}

impl WitnessPolicy {
    pub fn new(config: WitnessConfig, members: &[WitnessMember]) -> Result<Self> {
        require(
            config.epoch == 1
                && config.generation > 0
                && !config.partition.is_empty()
                && config
                    .partition
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
                && !config.sections.is_empty()
                && config.sections.iter().all(|section| {
                    [1, 2].contains(section)
                        || (config.partition == crate::native_config::PARTITION
                            && config.sections.as_slice() == [1, 2, 3, 4, 5, 6, 7, 8])
                })
                && config.sections.windows(2).all(|pair| pair[0] < pair[1]),
            "invalid witness scope",
        )?;
        let mut members: Vec<_> = members.iter().collect();
        members.sort_by_key(|member| member.member);
        require(
            members.iter().map(|member| member.member).eq(0..5),
            "invalid witness membership",
        )?;
        let mut parts = Vec::new();
        let mut keys = Vec::new();
        for member in members {
            ops::verify_proof_of_possession::<MinSig>(
                &member.bls_key,
                b"I1b/storage/pop/v1",
                &member.pop,
            )
            .map_err(|_| Error("invalid witness proof of possession"))?;
            parts.extend([
                member.member.to_be_bytes().to_vec(),
                member.key.encode().to_vec(),
                member.bls_key.encode().to_vec(),
                member.pop.encode().to_vec(),
            ]);
            keys.push((member.key.clone(), member.bls_key.clone()));
        }
        let registry_hash = hash_parts(
            b"I1b/voter-registry/v1",
            &parts.iter().map(Vec::as_slice).collect::<Vec<_>>(),
        );
        let local_key = keys
            .get(config.member as usize)
            .ok_or(Error("unknown local member"))?
            .0
            .clone();
        let registry = BiMap::try_from(keys).map_err(|_| Error("duplicate witness key"))?;
        let index = registry
            .keys()
            .index(&local_key)
            .ok_or(Error("missing local key"))?;
        let namespace_hash = config.voting_namespace();
        let scheme = NativeScheme::verifier(&namespace_hash, registry);
        Ok(Self {
            seal: PolicySeal {
                config,
                registry_hash,
                namespace_hash,
                local_signer: index.get(),
            },
            scheme,
        })
    }

    pub(crate) fn new_m1(
        config: WitnessConfig,
        members: &[WitnessMember],
        registry_hash: Id,
        namespace_hash: Id,
        local_signer: u32,
    ) -> Result<Self> {
        require(
            config.epoch == 1
                && config.generation == 1
                && config
                    .sections
                    .iter()
                    .copied()
                    .eq(1..=crate::m1_smoke::profile::VIEW_MAX)
                && config.partition == format!("normal-v1-voter-{}", poc_types::hex(&config.cid))
                && config.worker < 15
                && config.worker % 5 == config.member
                && members.iter().map(|member| member.member).eq(0..5),
            "invalid M1 witness scope",
        )?;
        let mut entries = Vec::new();
        for member in members {
            ops::verify_proof_of_possession::<MinSig>(
                &member.bls_key,
                poc_authorization::POP_NAMESPACE,
                &member.pop,
            )
            .map_err(|_| Error("invalid M1 witness proof of possession"))?;
            entries.push((member.key.clone(), member.bls_key.clone()));
        }
        let registry = BiMap::try_from(entries).map_err(|_| Error("duplicate M1 witness key"))?;
        let index = registry
            .keys()
            .index(&members[config.member as usize].key)
            .ok_or(Error("missing M1 witness local key"))?;
        require(
            index.get() == local_signer,
            "M1 witness signer-order mismatch",
        )?;
        Ok(Self {
            seal: PolicySeal {
                config,
                registry_hash,
                namespace_hash,
                local_signer,
            },
            scheme: NativeScheme::verifier(&namespace_hash, registry),
        })
    }

    pub fn seal(&self) -> &PolicySeal {
        &self.seal
    }

    pub(crate) fn section(&self, partition: &str, name: Option<&[u8]>) -> Option<u64> {
        if partition != self.seal.config.partition {
            return None;
        }
        let section = u64::from_be_bytes(name?.try_into().ok()?);
        self.seal
            .config
            .sections
            .contains(&section)
            .then_some(section)
    }

    pub fn decode_section(&self, section: u64, image: &[u8]) -> Result<Vec<DecodedArtifact>> {
        self.decode_frames(section, image, false)
    }

    pub(crate) fn decode_frames(
        &self,
        section: u64,
        image: &[u8],
        allow_incomplete_tail: bool,
    ) -> Result<Vec<DecodedArtifact>> {
        require(
            self.seal.config.sections.contains(&section),
            "journal section outside witness scope",
        )?;
        require(
            image.len() <= MAX_BLOB_BYTES && image.len().is_multiple_of(PAGE_BYTES),
            "invalid paged extent",
        )?;
        let mut logical = Vec::new();
        let mut pages = Vec::new();
        for (index, page) in image.chunks_exact(PAGE_BYTES).enumerate() {
            let (slot, length) = page_slot(page)?;
            require(
                length == DATA_BYTES || (index + 1) * PAGE_BYTES == image.len(),
                "nonfinal partial page",
            )?;
            logical.extend_from_slice(&page[..length]);
            pages.push(PageEvidence {
                physical_offset: (index * PAGE_BYTES) as u64,
                selected_slot: slot,
                selected_length: length as u16,
                bytes: page.to_vec(),
            });
        }
        let mut frames = Vec::new();
        let mut offset = 0;
        let mut rng = ChaCha20Rng::from_seed(hash_parts(
            b"I1b/witness-verification-rng/v1",
            &[&self.seal.namespace_hash],
        ));
        while offset < logical.len() {
            let remaining = &logical[offset..];
            let mut reader = Copying(remaining);
            let len = match UInt::<u32>::read(&mut reader) {
                Ok(len) => len,
                Err(commonware_codec::Error::EndOfBuffer) if allow_incomplete_tail => break,
                Err(_) => return Err(Error("invalid native frame varint")),
            };
            use bytes::Buf as _;
            let prefix = remaining.len() - reader.remaining();
            require(
                len.encode().as_ref() == &remaining[..prefix],
                "noncanonical native frame varint",
            )?;
            let size = len.0 as usize;
            require(
                size > 0 && size <= MAX_ARTIFACT_BYTES,
                "invalid native frame size",
            )?;
            let end = offset
                .checked_add(prefix + size)
                .ok_or(Error("frame offset overflow"))?;
            if end > logical.len() {
                if allow_incomplete_tail {
                    break;
                }
                return Err(Error("incomplete native frame"));
            }
            require(frames.len() < MAX_FRAMES, "too many native frames")?;
            let artifact_bytes = &logical[offset + prefix..end];
            let artifact = NativeArtifact::decode_cfg(
                Copying(artifact_bytes),
                &self.scheme.certificate_codec_config(),
            )
            .map_err(|_| Error("invalid native Artifact encoding"))?;
            require(
                artifact.encode().as_ref() == artifact_bytes,
                "noncanonical native Artifact",
            )?;
            require(
                artifact.epoch().get() == self.seal.config.epoch
                    && artifact.view().get() == section,
                "native Artifact epoch or section mismatch",
            )?;
            let wire = self.verify(&mut rng, artifact)?;
            let first = offset / DATA_BYTES;
            let last = (end - 1) / DATA_BYTES;
            frames.push(DecodedArtifact {
                section,
                logical_offset: offset as u64,
                frame_bytes: logical[offset..end].to_vec(),
                artifact_bytes: artifact_bytes.to_vec(),
                pages: pages[first..=last].to_vec(),
                wire,
            });
            offset = end;
        }
        Ok(frames)
    }

    fn verify(
        &self,
        rng: &mut ChaCha20Rng,
        artifact: NativeArtifact,
    ) -> Result<Option<NativeWire>> {
        let signer = self.seal.local_signer;
        let scheme = &self.scheme;
        let out = match artifact {
            Artifact::Notarize(vote) => {
                require(
                    vote.verify(rng, scheme, &Sequential)
                        && vote.attestation.signer.get() == signer,
                    "invalid or foreign notarize",
                )?;
                NativeWire {
                    channel: 1,
                    bytes: Vote::Notarize(vote).encode().to_vec(),
                }
            }
            Artifact::Nullify(vote) => {
                require(
                    vote.verify::<_, Digest>(rng, scheme, &Sequential)
                        && vote.attestation.signer.get() == signer,
                    "invalid or foreign nullify",
                )?;
                NativeWire {
                    channel: 1,
                    bytes: Vote::<NativeScheme, Digest>::Nullify(vote)
                        .encode()
                        .to_vec(),
                }
            }
            Artifact::Finalize(vote) => {
                require(
                    vote.verify(rng, scheme, &Sequential)
                        && vote.attestation.signer.get() == signer,
                    "invalid or foreign finalize",
                )?;
                NativeWire {
                    channel: 1,
                    bytes: Vote::Finalize(vote).encode().to_vec(),
                }
            }
            Artifact::Certification(_, _) => return Ok(None),
            Artifact::Notarization(certificate) => {
                self.certificate(rng, Certificate::Notarization(certificate))?
            }
            Artifact::Nullification(certificate) => {
                self.certificate(rng, Certificate::Nullification(certificate))?
            }
            Artifact::Finalization(certificate) => {
                self.certificate(rng, Certificate::Finalization(certificate))?
            }
        };
        Ok(Some(out))
    }

    fn certificate(
        &self,
        rng: &mut ChaCha20Rng,
        value: Certificate<NativeScheme, Digest>,
    ) -> Result<NativeWire> {
        require(
            value.verify(rng, &self.scheme, &Sequential),
            "invalid native certificate",
        )?;
        Ok(NativeWire {
            channel: 2,
            bytes: value.encode().to_vec(),
        })
    }
}

pub fn page_slot(page: &[u8]) -> Result<(u8, usize)> {
    require(page.len() == PAGE_BYTES, "invalid physical page size")?;
    let mut selected = None;
    for slot in 0..2 {
        let index = DATA_BYTES + slot * 6;
        let len = usize::from(u16::from_be_bytes(
            page[index..index + 2].try_into().unwrap(),
        ));
        let expected = u32::from_be_bytes(page[index + 2..index + 6].try_into().unwrap());
        if len > 0
            && len <= DATA_BYTES
            && Crc32::checksum(&page[..len]) == expected
            && selected.is_none_or(|(_, old_len)| len > old_len)
        {
            selected = Some((slot as u8, len));
        }
    }
    selected.ok_or(Error("no valid physical page CRC"))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageEvidence {
    pub physical_offset: u64,
    pub selected_slot: u8,
    pub selected_length: u16,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeWire {
    pub channel: u32,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodedArtifact {
    pub section: u64,
    pub logical_offset: u64,
    pub frame_bytes: Vec<u8>,
    pub artifact_bytes: Vec<u8>,
    pub pages: Vec<PageEvidence>,
    pub wire: Option<NativeWire>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Barrier {
    pub blob_id: Id,
    pub io_id: u64,
    pub kind: u8,
    pub offset: u64,
    pub bytes: Vec<u8>,
    pub id: Id,
}

impl Barrier {
    pub(crate) fn new(blob_id: Id, io_id: u64, kind: u8, offset: u64, bytes: Vec<u8>) -> Self {
        let hash = hash_parts(b"I1b/barrier-bytes/v1", &[&bytes]);
        let id = hash_parts(
            b"I1b/native-barrier/v1",
            &[
                &blob_id,
                &io_id.to_be_bytes(),
                &[kind],
                &offset.to_be_bytes(),
                &(bytes.len() as u64).to_be_bytes(),
                &hash,
            ],
        );
        Self {
            blob_id,
            io_id,
            kind,
            offset,
            bytes,
            id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactWitness {
    pub id: Id,
    pub blob_id: Id,
    pub artifact_hash: Id,
    pub barrier_ids: Vec<Id>,
    pub frame: DecodedArtifact,
}

impl ArtifactWitness {
    pub(crate) fn new(
        seal: &PolicySeal,
        blob_id: Id,
        frame: DecodedArtifact,
        barrier_ids: Vec<Id>,
    ) -> Self {
        let config = &seal.config;
        let artifact_hash = hash_parts(b"I1b/native-artifact/v1", &[&frame.artifact_bytes]);
        let ids: Vec<_> = barrier_ids.iter().flatten().copied().collect();
        let id = hash_parts(
            b"I1b/artifact-witness/v1",
            &[
                &config.run_id,
                &config.config_hash,
                &config.cid,
                &config.epoch.to_be_bytes(),
                &blob_id,
                &frame.section.to_be_bytes(),
                &frame.logical_offset.to_be_bytes(),
                &artifact_hash,
                &ids,
            ],
        );
        Self {
            id,
            blob_id,
            artifact_hash,
            barrier_ids,
            frame,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EligiblePayload {
    pub id: Id,
    pub witness_id: Id,
    pub wire_hash: Id,
    pub wire: NativeWire,
}

impl EligiblePayload {
    pub(crate) fn new(seal: &PolicySeal, witness_id: Id, wire: NativeWire) -> Self {
        let cfg = &seal.config;
        let channel = wire.channel.to_be_bytes();
        let wire_hash = hash_parts(b"I1b/native-wire/v1", &[&channel, &wire.bytes]);
        let id = hash_parts(
            b"I1b/outbox-payload/v1",
            &[
                &cfg.run_id,
                &cfg.config_hash,
                &cfg.cid,
                &cfg.epoch.to_be_bytes(),
                &cfg.member.to_be_bytes(),
                &channel,
                &wire_hash,
            ],
        );
        Self {
            id,
            witness_id,
            wire_hash,
            wire,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Submission {
    pub id: Id,
    pub payload_id: Id,
    pub destination: u32,
}

impl Submission {
    pub(crate) fn new(payload_id: Id, destination: u32) -> Self {
        let id = hash_parts(b"I1b/outbox/v1", &[&payload_id, &destination.to_be_bytes()]);
        Self {
            id,
            payload_id,
            destination,
        }
    }
}

pub(crate) fn require(condition: bool, error: &'static str) -> Result<()> {
    if condition { Ok(()) } else { Err(Error(error)) }
}

#[derive(Clone)]
pub(crate) struct Image {
    pub bytes: Vec<u8>,
    covered: Vec<Option<Id>>,
    durable_extent: Option<(usize, Id)>,
    pub uncertain: bool,
}

impl Image {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            covered: vec![None; bytes.len()],
            durable_extent: None,
            bytes,
            uncertain: false,
        }
    }

    pub fn write(&mut self, offset: usize, bytes: &[u8]) -> Result<()> {
        if bytes.is_empty() {
            return Ok(());
        }
        let end = offset
            .checked_add(bytes.len())
            .filter(|end| *end <= MAX_BLOB_BYTES)
            .ok_or(Error("witness write exceeds bound"))?;
        if self.bytes.len() < end {
            self.resize(end)?;
        }
        for (index, byte) in bytes.iter().enumerate() {
            let position = offset + index;
            if self.bytes[position] != *byte {
                self.bytes[position] = *byte;
                self.covered[position] = None;
            }
        }
        Ok(())
    }

    pub fn resize(&mut self, length: usize) -> Result<()> {
        require(length <= MAX_BLOB_BYTES, "witness resize exceeds bound")?;
        self.bytes.resize(length, 0);
        self.covered.resize(length, None);
        if let Some((extent, _)) = &mut self.durable_extent {
            *extent = (*extent).min(length);
        }
        Ok(())
    }

    pub fn cover(&mut self, range: std::ops::Range<usize>, barrier: Id) {
        let end = range.end;
        self.covered[range].fill(Some(barrier));
        if self.durable_extent.is_none_or(|(extent, _)| end >= extent) {
            self.durable_extent = Some((end, barrier));
        }
    }

    pub fn provenance(&self, frame: &DecodedArtifact) -> Option<Vec<Id>> {
        if self.uncertain {
            return None;
        }
        let (extent, extent_id) = self.durable_extent?;
        let mut ids = std::collections::BTreeSet::from([extent_id]);
        for page in &frame.pages {
            let physical = usize::try_from(page.physical_offset).ok()?;
            if physical.checked_add(PAGE_BYTES)? > extent {
                return None;
            }
            let footer = DATA_BYTES + usize::from(page.selected_slot) * 6;
            for byte in (0..usize::from(page.selected_length)).chain(footer..footer + 6) {
                ids.insert(self.covered.get(physical + byte).copied().flatten()?);
            }
        }
        Some(ids.into_iter().collect())
    }
}

pub(crate) type Images = BTreeMap<Id, Image>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_and_bad_crc_reject_and_valid_fallback_and_tie_select() {
        let mut page = [0; PAGE_BYTES];
        page[..24].fill(5);
        assert!(page_slot(&page).is_err());
        for (slot, length) in [(0, 12u16), (1, 24u16)] {
            let start = DATA_BYTES + slot * 6;
            let crc = Crc32::checksum(&page[..usize::from(length)]);
            page[start..start + 2].copy_from_slice(&length.to_be_bytes());
            page[start + 2..start + 6].copy_from_slice(&crc.to_be_bytes());
        }
        assert_eq!(page_slot(&page), Ok((1, 24)));
        page[23] ^= 1;
        assert_eq!(page_slot(&page), Ok((0, 12)));
        let first_slot = page[244..250].to_vec();
        page[250..256].copy_from_slice(&first_slot);
        assert_eq!(page_slot(&page), Ok((0, 12)));
    }
}

use crate::{
    IoIdentity, IoOperation, IoResult,
    storage::{NativeBlob, WitnessIo},
    witness_ledger::{
        BlobSeal, IoObservation, LedgerEntry, ObservedChange, WitnessFault, WitnessLedger,
    },
};
use commonware_runtime::{Blob as _, ReadOptions};
use std::sync::Arc;

pub(crate) fn verify_protected<'a>(
    witnesses: impl Iterator<Item = &'a ArtifactWitness>,
    section: u64,
    image: &[u8],
) -> Result<()> {
    use bytes::Buf as _;
    let protected: Vec<_> = witnesses.filter(|w| w.frame.section == section).collect();
    let Some(end) = protected
        .iter()
        .map(|w| w.frame.logical_offset as usize + w.frame.frame_bytes.len())
        .max()
    else {
        return Ok(());
    };
    let mut logical = Vec::new();
    for index in 0..end.div_ceil(DATA_BYTES) {
        let physical = index * PAGE_BYTES;
        let page = image
            .get(physical..physical + PAGE_BYTES)
            .ok_or(Error("selected mutation removes a protected page"))?;
        let (_, len) = page_slot(page)?;
        require(
            len == DATA_BYTES || physical + PAGE_BYTES == image.len(),
            "selected mutation creates a nonfinal partial page",
        )?;
        logical.extend_from_slice(&page[..len]);
    }
    let mut remaining = logical
        .get(..end)
        .ok_or(Error("protected frame is outside valid CRC prefix"))?;
    let mut offset = 0;
    let mut offsets = BTreeMap::new();
    while !remaining.is_empty() {
        let mut reader = Copying(remaining);
        let len =
            UInt::<u32>::read(&mut reader).map_err(|_| Error("protected native frame varint"))?;
        let prefix = remaining.len() - reader.remaining();
        require(
            (1..=MAX_ARTIFACT_BYTES).contains(&(len.0 as usize))
                && len.encode().as_ref() == &remaining[..prefix],
            "protected native frame prefix changed",
        )?;
        let frame_len = prefix + len.0 as usize;
        let frame = remaining
            .get(..frame_len)
            .ok_or(Error("protected native frame boundary changed"))?;
        offsets.insert(offset, frame);
        offset += frame_len;
        remaining = &remaining[frame_len..];
    }
    require(
        protected.iter().all(|w| {
            offsets
                .get(&(w.frame.logical_offset as usize))
                .is_some_and(|bytes| *bytes == w.frame.frame_bytes)
        }),
        "selected mutation would alter protected history",
    )
}

pub(crate) struct WitnessRuntime {
    pub policy: WitnessPolicy,
    pub ledger: WitnessLedger,
}

impl WitnessRuntime {
    pub fn new(policy: WitnessPolicy, ledger: WitnessLedger) -> Self {
        Self { policy, ledger }
    }

    pub fn preflight(&self, identity: &IoIdentity, io: &WitnessIo) -> Result<()> {
        let cfg = &self.policy.seal.config;
        require(
            identity.run_id == cfg.run_id && identity.node == cfg.worker && identity.cid == cfg.cid,
            "selected I/O does not match the witness context",
        )?;
        if matches!(io.operation, IoOperation::Scan) {
            return Ok(());
        }
        if matches!(io.operation, IoOperation::Remove) && io.target.name.is_none() {
            return require(
                self.ledger.index.witnesses.is_empty(),
                "remove would erase protected witness history",
            );
        }
        let section = self
            .policy
            .section(&io.target.partition, io.target.name.as_deref())
            .ok_or(Error("unexpected selected journal name"))?;
        if matches!(io.operation, IoOperation::Remove) {
            return require(
                !self
                    .ledger
                    .index
                    .witnesses
                    .values()
                    .any(|w| w.frame.section == section),
                "remove would erase protected witness history",
            );
        }
        if !matches!(
            io.operation,
            IoOperation::Write { .. } | IoOperation::Resize { .. }
        ) {
            return Ok(());
        }
        let blob = BlobSeal::new(
            self.policy.seal(),
            io.target
                .open_id
                .ok_or(Error("missing selected open identity"))?,
            section,
        );
        let mut forecast = self
            .ledger
            .index
            .images
            .get(&blob.id)
            .ok_or(Error("selected blob lacks observed extent"))?
            .clone();
        require(
            !forecast.uncertain && !self.ledger.index.uncertain_sections.contains(&section),
            "selected blob mutation requires fresh recovery",
        )?;
        Self::mutate(&mut forecast, io)?;
        verify_protected(
            self.ledger.index.witnesses.values(),
            section,
            &forecast.bytes,
        )
    }

    async fn read_image(native: Option<Arc<NativeBlob>>, length: usize) -> Result<Vec<u8>> {
        require(length <= MAX_BLOB_BYTES, "selected blob exceeds bound")?;
        if length == 0 {
            return Ok(Vec::new());
        }
        let native = native.ok_or(Error("missing same-open native blob"))?;
        let data = native
            .read_at(0, length, ReadOptions::DONT_CACHE)
            .await
            .map_err(|_| Error("cannot read selected same-open image"))?;
        Ok(data.coalesce().as_ref().to_vec())
    }

    fn mutate(image: &mut Image, io: &WitnessIo) -> Result<()> {
        match io.operation {
            IoOperation::Write { offset, .. } => {
                let bytes = io
                    .submitted
                    .as_ref()
                    .ok_or(Error("selected write missing submitted bytes"))?
                    .clone()
                    .coalesce();
                image.write(
                    usize::try_from(offset).map_err(|_| Error("selected write offset overflow"))?,
                    bytes.as_ref(),
                )
            }
            IoOperation::Resize { len } => {
                image.resize(usize::try_from(len).map_err(|_| Error("selected extent overflow"))?)
            }
            _ => Ok(()),
        }
    }

    pub async fn observe(
        mut self,
        identity: IoIdentity,
        mut io: WitnessIo,
        result: IoResult,
        fault: Option<WitnessFault>,
    ) -> (Self, Result<()>) {
        let staged = self.observe_inner(&identity, &mut io, &result).await;
        let result = match staged {
            Ok(entries) => self.ledger.append(entries, fault),
            Err(error) => Err(error),
        };
        (self, result)
    }

    async fn observe_inner(
        &mut self,
        identity: &IoIdentity,
        io: &mut WitnessIo,
        outcome: &IoResult,
    ) -> Result<Vec<LedgerEntry>> {
        let section = self
            .policy
            .section(&io.target.partition, io.target.name.as_deref());
        let observed = |change| {
            LedgerEntry::Observation(IoObservation {
                io_id: identity.io_id,
                change,
            })
        };
        if matches!(io.operation, IoOperation::Remove) {
            return Ok(vec![observed(if *outcome == IoResult::Ok {
                ObservedChange::Remove { section }
            } else {
                ObservedChange::Uncertain {
                    section,
                    reason: "remove".to_owned(),
                }
            })]);
        }
        let Some(section) = section else {
            return Ok(Vec::new());
        };
        let Some(open_id) = io.target.open_id else {
            return Ok(Vec::new());
        };
        let mut blob = BlobSeal::new(self.policy.seal(), open_id, section);
        if let IoResult::Open { len, version } = outcome {
            require(
                *version == 0,
                "unexpected selected blob application version",
            )?;
            blob.initial_bytes = Self::read_image(
                io.native.take(),
                usize::try_from(*len).map_err(|_| Error("selected open size overflow"))?,
            )
            .await?;
            if !self.ledger.index.in_recovery() {
                verify_protected(
                    self.ledger.index.witnesses.values(),
                    section,
                    &blob.initial_bytes,
                )?;
            }
            return Ok(vec![LedgerEntry::Blob(blob)]);
        }
        if !matches!(
            io.operation,
            IoOperation::Write { .. } | IoOperation::Resize { .. } | IoOperation::Sync { .. }
        ) {
            return Ok(Vec::new());
        }
        if matches!(outcome, IoResult::Error { .. }) {
            let reason = match io.operation {
                IoOperation::Write { .. } => "write",
                IoOperation::Resize { .. } => "resize",
                IoOperation::Sync { .. } => "sync",
                _ => unreachable!("selected mutation"),
            };
            return Ok(vec![observed(ObservedChange::Uncertain {
                section: Some(section),
                reason: reason.to_owned(),
            })]);
        }
        require(
            *outcome == IoResult::Ok,
            "selected completion kind mismatch",
        )?;
        let mut image = self
            .ledger
            .index
            .images
            .get(&blob.id)
            .ok_or(Error("missing selected observed blob"))?
            .clone();
        Self::mutate(&mut image, io)?;
        verify_protected(self.ledger.index.witnesses.values(), section, &image.bytes)?;
        let uncertain = image.uncertain || self.ledger.index.uncertain_sections.contains(&section);
        let barrier = match io.operation {
            IoOperation::Write {
                offset,
                len,
                sync: true,
                ..
            } if len > 0 && !uncertain => {
                let start = usize::try_from(offset).map_err(|_| Error("range offset overflow"))?;
                Barrier::new(
                    blob.id,
                    identity.io_id,
                    1,
                    offset,
                    image.bytes[start..start + len].to_vec(),
                )
            }
            IoOperation::Write { offset, len, .. } if len > 0 => {
                let start = usize::try_from(offset).map_err(|_| Error("write offset overflow"))?;
                return Ok(vec![observed(ObservedChange::Write {
                    blob_id: blob.id,
                    offset,
                    bytes: image.bytes[start..start + len].to_vec(),
                })]);
            }
            IoOperation::Resize { len } => {
                return Ok(vec![observed(ObservedChange::Resize {
                    blob_id: blob.id,
                    len,
                })]);
            }
            IoOperation::Sync { .. } if !image.bytes.is_empty() && !uncertain => {
                Barrier::new(blob.id, identity.io_id, 2, 0, image.bytes.clone())
            }
            _ => return Ok(Vec::new()),
        };
        let actual = Self::read_image(io.native.take(), image.bytes.len()).await?;
        require(
            actual == image.bytes,
            "same-open readback does not match tracked bytes",
        )?;
        let start = barrier.offset as usize;
        image.cover(start..start + barrier.bytes.len(), barrier.id);
        let frames = self.policy.decode_frames(section, &image.bytes, true)?;
        let mut entries = vec![LedgerEntry::Barrier(barrier)];
        let mut new_payloads = std::collections::BTreeSet::new();
        for frame in frames {
            let Some(ids) = image.provenance(&frame) else {
                continue;
            };
            if self.ledger.index.witnesses.values().any(|old| {
                old.blob_id == blob.id
                    && old.frame.logical_offset == frame.logical_offset
                    && old.frame.frame_bytes == frame.frame_bytes
            }) {
                continue;
            }
            let witness = ArtifactWitness::new(self.policy.seal(), blob.id, frame, ids);
            let wire = witness.frame.wire.clone();
            let witness_id = witness.id;
            entries.push(LedgerEntry::Witness(witness));
            if let Some(wire) = wire {
                let payload = EligiblePayload::new(self.policy.seal(), witness_id, wire);
                if !self.ledger.index.eligible.contains_key(&payload.id)
                    && new_payloads.insert(payload.id)
                {
                    entries.push(LedgerEntry::Eligible(payload));
                }
            }
        }
        Ok(entries)
    }
}
