//! Verifier-owned finite inputs and a hand-derived execution oracle.
//! No viewer call, Apply, or Merge is used to compute expected state/native blocks.
use poc_authorization::{StaticScheduleProvider, ValidatedRegistry};
use poc_core::{
    fixture::{CheckedSetup, FinalizedFixtureRow, check_setup, reg_a_l1_fixture},
    merge::MergedBlock,
};
use poc_types::domain::*;
use poc_viewer_gfr::{PlanOutcome, VerificationBasis};
use serde_json::Value;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
};

#[path = "component_fixture.rs"]
mod original;

pub const E_SOURCE: &[u8] = b"verifier-gfr-espresso-v1";
pub const L_SOURCE: &[u8] = b"verifier-gfr-mock-l1-v1";
pub const FIXTURE_ID: &str = "verifier-gfr-exact-subject-noncrypto-v1";
pub const LAST_DEP: u64 = 40;

pub struct Fixture {
    pub setup: CheckedSetup,
    pub plan: StaticScheduleProvider,
    pub tokens: Vec<FinalizedFixtureRow>,
}

impl Fixture {
    pub fn new() -> Self {
        Self::with_dep_limit(LAST_DEP)
    }

    pub fn with_dep_limit(last_dep: u64) -> Self {
        assert!((3..=LAST_DEP).contains(&last_dep));
        let json: Value = serde_json::from_str(include_str!(
            "public-registry.json"
        ))
        .unwrap();
        let record = RegistryRecord {
            entries: json["entries"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| RegistryEntry {
                    pid: Pid::new(r["pid"].as_u64().unwrap() as u8).unwrap(),
                    identity_key: original::unhex(r["identity_key"].as_str().unwrap())
                        .try_into()
                        .unwrap(),
                    vote_key: original::unhex(r["vote_key"].as_str().unwrap())
                        .try_into()
                        .unwrap(),
                    pop: original::unhex(r["pop"].as_str().unwrap())
                        .try_into()
                        .unwrap(),
                })
                .collect(),
        };
        let registry = ValidatedRegistry::load(record).unwrap();
        let mut config = original::config();
        config.registry_hash = registry.digest();
        let mut dep = original::dep_fixture();
        dep.rows.resize(last_dep as usize, vec![]);
        let l1 = reg_a_l1_fixture();
        config.dep_fixture_hash = dep.digest().unwrap();
        config.l1_fixture_hash = l1.digest().unwrap();
        let genesis = original::execution_anchor();
        let setup = check_setup(&config, &genesis, dep, l1.clone()).unwrap();
        let mut finalizer = setup.l1().finalizer();
        let tokens = l1
            .rows
            .iter()
            .map(|row| {
                finalizer
                    .finalize(&row.header, row.finalized_at_us)
                    .unwrap()
            })
            .collect();
        let plan = StaticScheduleProvider::new(config, registry).unwrap();
        Self {
            setup,
            plan,
            tokens,
        }
    }

    pub fn config(&self) -> &PublicConfig {
        self.setup.config()
    }
    pub fn genesis(&self) -> &ExecutionAnchor {
        self.setup.genesis()
    }

    /// Pinned schedule at genesis ts=10s, origin=3, Tdep=5s, L1 slot=2s.
    pub fn metadata(h: u64) -> Metadata {
        assert!(h <= LAST_DEP);
        Metadata {
            height: h,
            timestamp_us: 10_000_000 + h * 1_000_000,
            origin: if h == 0 { 3 } else { (h + 5) / 2 },
        }
    }

    /// Literal balances/nonce outcomes from the accepted 03 first-three-height vector.
    pub fn expected_state(h: u64, transaction_height: Option<u64>) -> ExecutionState {
        let changed = transaction_height.is_some_and(|t| h >= t);
        let rows = if changed {
            vec![(0, 2, 90), (1, 2, 15), (2, 0, 19)]
        } else if h == 0 {
            vec![(0, 0, 100), (2, 0, 7)]
        } else if h == 1 {
            vec![(0, 0, 105), (2, 0, 7)]
        } else if h == 2 {
            vec![(0, 0, 105), (1, 0, 2), (2, 0, 7)]
        } else {
            vec![(0, 0, 115), (1, 0, 2), (2, 0, 7)]
        };
        ExecutionState {
            metadata: Self::metadata(h),
            accounts: rows
                .into_iter()
                .map(|(id, nonce, balance)| (id, Account { nonce, balance }))
                .collect(),
        }
    }

    pub fn txs() -> Vec<Vec<u8>> {
        let tuples = [
            (0, 1, 7, 1),
            (0, 1, 0, 30),
            (0, 2, 1, 100),
            (1, 2, 0, 12),
            (0, 1, 1, 5),
            (1, 0, 0, 1),
            (1, 0, 1, 10),
        ];
        tuples
            .into_iter()
            .map(|(from, to, nonce, amount)| {
                Transfer {
                    from,
                    to,
                    nonce,
                    amount,
                }
                .encode_raw()
            })
            .collect()
    }

    pub fn dep(h: u64) -> Vec<Vec<u8>> {
        let mut dep = vec![
            Attributes {
                metadata: Self::metadata(h),
            }
            .encode_raw(),
        ];
        match h {
            1 => dep.push(Credit { to: 0, amount: 5 }.encode_raw()),
            2 => dep.push(Credit { to: 1, amount: 2 }.encode_raw()),
            3 => dep.push(Credit { to: 0, amount: 10 }.encode_raw()),
            _ => {}
        }
        dep
    }

    pub fn expected_native(
        &self,
        h: u64,
        transaction_height: Option<u64>,
    ) -> (NativeBlock, Hash32) {
        assert!(h > 0 && h <= LAST_DEP);
        let mut parent = self.genesis().native_hash;
        let mut last = None;
        for n in 1..=h {
            let kept = if transaction_height == Some(n) {
                let txs = Self::txs();
                [1, 3, 4, 6].into_iter().map(|i| txs[i].clone()).collect()
            } else {
                vec![]
            };
            let block = NativeBlock {
                parent,
                metadata: Self::metadata(n),
                dep: Self::dep(n),
                kept,
                post_state_root: Self::expected_state(n, transaction_height).root().unwrap(),
            };
            parent = block.digest().unwrap();
            last = Some(block);
        }
        (last.unwrap(), parent)
    }

    pub fn physical_chain(
        &self,
        anchor: &ExecutionAnchor,
        heights: &[u64],
        transaction_height: Option<u64>,
    ) -> Vec<PhysicalBlock> {
        let mut parent = anchor.anchor.block.block_id;
        heights
            .iter()
            .map(|&h| {
                let body = PhysicalBlock {
                    parent,
                    metadata: Self::metadata(h),
                    pre_state_root: Self::expected_state(h - 1, transaction_height)
                        .root()
                        .unwrap(),
                    transactions: if transaction_height == Some(h) {
                        Self::txs()
                    } else {
                        vec![]
                    },
                };
                parent = body.digest().unwrap();
                body
            })
            .collect()
    }

    pub fn execution_anchor(
        &self,
        block: &PhysicalBlock,
        transaction_height: Option<u64>,
    ) -> ExecutionAnchor {
        let h = block.metadata.height;
        ExecutionAnchor {
            anchor: Anchor {
                block: BlockRef {
                    height: h,
                    origin: block.metadata.origin,
                    block_id: block.digest().unwrap(),
                },
                timestamp_us: block.metadata.timestamp_us,
            },
            state: Self::expected_state(h, transaction_height),
            native_hash: self.expected_native(h, transaction_height).1,
        }
    }

    pub fn descriptor(&self, r: u64, production: &Anchor, reference: &Anchor) -> Descriptor {
        struct RefView(Anchor);
        impl PublicReferenceView for RefView {
            fn reference(&self, r: &BlockRef) -> InputStatus<Anchor> {
                if self.0.block == *r {
                    InputStatus::Ready(self.0.clone())
                } else {
                    InputStatus::Reject(InputError::InvalidSource)
                }
            }
        }
        let input = PlanInput {
            session: SessionId { beta: 0, r },
            cause: if r == 0 {
                BoundaryCause::Genesis
            } else {
                BoundaryCause::Handoff
            },
            source_region: if r == 0 {
                None
            } else {
                Some(RegionId::new(((r - 1) % 3) as u8).unwrap())
            },
            reference_block: reference.block.clone(),
            window: Window {
                h_min: r * 10 + 1,
                h_max: r * 10 + 10,
            },
        };
        let PlanOutcome::Ready(plan) = self.plan.resolve(&input, &RefView(reference.clone()))
        else {
            panic!("fixture Plan")
        };
        self.plan.descriptor(&plan, production).unwrap()
    }

    pub fn expected_entry(
        &self,
        h: u64,
        physical: Option<Hash32>,
        transaction_height: Option<u64>,
    ) -> MergedBlock {
        let (native_block, native_hash) = self.expected_native(h, transaction_height);
        MergedBlock {
            virtual_block: VirtualBlock {
                metadata: Self::metadata(h),
                dep: Self::dep(h),
                pre_state_root: Self::expected_state(h - 1, transaction_height)
                    .root()
                    .unwrap(),
                transactions: if transaction_height == Some(h) {
                    Self::txs()
                } else {
                    vec![]
                },
            },
            physical_digest: physical,
            state: Self::expected_state(h, transaction_height),
            native_block,
            native_hash,
        }
    }
}

#[derive(Default)]
pub struct ExactProofs {
    entries: RefCell<Vec<(Descriptor, PhysicalBlock, Vec<u8>)>>,
    pub calls: Cell<usize>,
}
impl ExactProofs {
    pub fn register(&self, descriptor: &Descriptor, block: &PhysicalBlock) -> Vec<u8> {
        let proof = format!("NONCRYPTO-EXACT-SUBJECT-{}", self.entries.borrow().len()).into_bytes();
        self.entries
            .borrow_mut()
            .push((descriptor.clone(), block.clone(), proof.clone()));
        proof
    }
}
impl FinalVerifier for ExactProofs {
    fn check(&self, descriptor: &Descriptor, block: &PhysicalBlock, proof: &[u8]) -> bool {
        self.calls.set(self.calls.get() + 1);
        self.entries
            .borrow()
            .iter()
            .any(|(d, b, p)| d == descriptor && b == block && p == proof)
    }
}

#[derive(Default)]
pub struct Basis {
    pub mode: Cell<u8>,
    pub calls: Cell<usize>,
}
impl VerificationBasis for Basis {
    fn require(&self, _: &Descriptor, block: &PhysicalBlock) -> InputStatus<()> {
        self.calls.set(self.calls.get() + 1);
        match self.mode.get() {
            0 => InputStatus::Ready(()),
            1 => InputStatus::Wait(InputMissing::State(block.pre_state_root)),
            _ => InputStatus::Reject(InputError::InvalidSource),
        }
    }
}

pub fn key(descriptor: &Descriptor, block: &PhysicalBlock) -> ResultKey {
    ResultKey {
        cid: descriptor.cid,
        block_hash: block.digest().unwrap(),
    }
}
pub fn literal_account_map(h: u64, tx: Option<u64>) -> BTreeMap<u8, Account> {
    Fixture::expected_state(h, tx).accounts
}
