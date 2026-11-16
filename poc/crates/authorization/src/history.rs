//! The fixed-registry policy selection and its finalized L1 basis.

use poc_types::{domain::{Hash32, L1Position, RegionId}, hash_parts};

use crate::PlanError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegionalPolicy {
    pub enabled_mask: u8,
    pub order: [u8; 3],
    pub signed_floor: u64,
}

impl RegionalPolicy {
    pub const GENESIS: Self = Self { enabled_mask: 7, order: [0, 1, 2], signed_floor: 0 };

    pub fn from_parts(bytes: [u8; 4], signed_floor: u64) -> Result<Self, PlanError> {
        let [enabled_mask, a, b, c] = bytes;
        let order = [a, b, c];
        let mut sorted = order;
        sorted.sort_unstable();
        if enabled_mask > 7 || sorted != [0, 1, 2] {
            return Err(PlanError::PlanMismatch);
        }
        Ok(Self { enabled_mask, order, signed_floor })
    }

    pub fn mask_and_order(self) -> [u8; 4] {
        let [a, b, c] = self.order;
        [self.enabled_mask, a, b, c]
    }

    pub fn select(self, source: Option<RegionId>) -> Result<Option<RegionId>, PlanError> {
        Self::from_parts(self.mask_and_order(), self.signed_floor)?;
        let start = source.map_or(0, |region| {
            self.order.iter().position(|id| *id == region.get()).expect("validated permutation") + 1
        });
        let selected = (0..3).map(|offset| self.order[(start + offset) % 3])
            .find(|id| self.enabled_mask & (1 << *id) != 0);
        Ok(selected.map(RegionId::new).transpose()?)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActivePolicy {
    pub policy: RegionalPolicy,
    pub force: L1Position,
    pub control_seq: u64,
    pub floor: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoricalPlan {
    pub active: ActivePolicy,
    pub height: u64,
    pub block_hash: Hash32,
}

impl HistoricalPlan {
    pub fn bind(self, legacy_ref: Hash32, region: RegionId) -> Result<Hash32, PlanError> {
        let a = self.active;
        let tx = u32::try_from(a.force.tx).map_err(|_| PlanError::PlanMismatch)?;
        let log = u32::try_from(a.force.log).map_err(|_| PlanError::PlanMismatch)?;
        let hash = hash_parts("PoC/HistoricalPlan/v1", &[
            legacy_ref.as_bytes(), &a.force.block.to_be_bytes(), &tx.to_be_bytes(),
            &log.to_be_bytes(), &a.control_seq.to_be_bytes(), &a.policy.mask_and_order(),
            &a.policy.signed_floor.to_be_bytes(), &a.floor.to_be_bytes(), &self.height.to_be_bytes(),
            self.block_hash.as_bytes(), &[region.get()],
        ]);
        Ok(Hash32::new(hash.try_into().expect("SHA-256 length")))
    }

    pub fn encode(self) -> Vec<u8> {
        let a = self.active;
        [
            a.force.block.to_be_bytes().as_slice(), &a.force.tx.to_be_bytes(), &a.force.log.to_be_bytes(),
            &a.control_seq.to_be_bytes(), &a.policy.mask_and_order(), &a.policy.signed_floor.to_be_bytes(),
            &a.floor.to_be_bytes(), &self.height.to_be_bytes(), self.block_hash.as_bytes(),
        ].concat()
    }

    pub fn decode(mut bytes: &[u8]) -> Result<Self, PlanError> {
        fn take<const N: usize>(bytes: &mut &[u8]) -> Result<[u8; N], PlanError> {
            let (part, rest) = bytes.split_at_checked(N).ok_or(PlanError::PlanMismatch)?;
            *bytes = rest;
            Ok(part.try_into().expect("fixed slice length"))
        }
        let force = L1Position {
            block: u64::from_be_bytes(take(&mut bytes)?),
            tx: u64::from_be_bytes(take(&mut bytes)?),
            log: u64::from_be_bytes(take(&mut bytes)?),
        };
        let control_seq = u64::from_be_bytes(take(&mut bytes)?);
        let mask_and_order = take(&mut bytes)?;
        let policy = RegionalPolicy::from_parts(mask_and_order, u64::from_be_bytes(take(&mut bytes)?))?;
        let floor = u64::from_be_bytes(take(&mut bytes)?);
        let height = u64::from_be_bytes(take(&mut bytes)?);
        let block_hash = Hash32::new(take(&mut bytes)?);
        if !bytes.is_empty() {
            return Err(PlanError::PlanMismatch);
        }
        Ok(Self { active: ActivePolicy { policy, force, control_seq, floor }, height, block_hash })
    }
}
