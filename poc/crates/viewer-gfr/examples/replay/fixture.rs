use std::collections::BTreeMap;

use poc_authorization::ValidatedRegistry;
use poc_core::{
    execution::apply,
    fixture::{CheckedSetup, DepProvider, FinalizedFixtureRow, check_setup, reg_a_l1_fixture},
    merge::merge,
    metadata::{MetadataProvider, RegionalMetadata},
};
use poc_types::{domain::*, hash_parts};
use poc_viewer_gfr::*;
use serde_json::Value;
use sim_espresso::{FinalizedBlock, LogMessage, SimEspresso, TickUs, TimestampPolicy};

pub const SEC: u64 = 1_000_000;
pub const ESPRESSO_SOURCE: &[u8] = b"gfr-07/noncrypto-espresso";
pub const PROFILE: &str = "gfr-07-noncryptographic-rule-fixture-v1";
pub const ANNOUNCEMENT: u64 = 12 * SEC;
pub const EXPIRY: u64 = 30 * SEC;

pub struct Fixture {
    pub setup: CheckedSetup,
    pub l1_source: L1Fixture,
    pub dep_source: DepFixture,
    pub l1_rows: Vec<FinalizedFixtureRow>,
    pub plan: StaticScheduleProvider,
}

impl Fixture {
    pub fn new() -> Self {
        let registry_json: Value = serde_json::from_str(include_str!(
            "../../../../verification/mock-plan-execution/implementation-v1/public-registry.json"
        ))
        .unwrap();
        let entries = registry_json["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| RegistryEntry {
                pid: Pid::new(u8::try_from(entry["pid"].as_u64().unwrap()).unwrap()).unwrap(),
                identity_key: from_hex(entry["identity_key"].as_str().unwrap()),
                vote_key: from_hex(entry["vote_key"].as_str().unwrap()),
                pop: from_hex(entry["pop"].as_str().unwrap()),
            })
            .collect();
        let registry = ValidatedRegistry::load(RegistryRecord { entries }).unwrap();
        let dep_source = DepFixture {
            first_height: 1,
            rows: (1..=40)
                .map(|height| {
                    vec![Credit {
                        to: 2,
                        amount: height,
                    }]
                })
                .collect(),
        };
        let l1_source = reg_a_l1_fixture();
        let anchor = Anchor {
            block: BlockRef {
                height: 0,
                origin: 3,
                block_id: Hash32::new([0x21; 32]),
            },
            timestamp_us: 10 * SEC,
        };
        let genesis = ExecutionAnchor {
            state: ExecutionState {
                metadata: anchor.metadata(),
                accounts: BTreeMap::from([(
                    1,
                    Account {
                        nonce: 0,
                        balance: 100,
                    },
                )]),
            },
            anchor,
            native_hash: Hash32::new([0x42; 32]),
        };
        let config = PublicConfig {
            chain_id: 7,
            rule_version: 1,
            verifier_version: 1,
            registry_hash: registry.digest(),
            dep_fixture_hash: dep_source.digest().unwrap(),
            l1_fixture_hash: l1_source.digest().unwrap(),
            genesis: genesis.anchor.clone(),
            genesis_state_root: genesis.state.root().unwrap(),
            genesis_native_hash: genesis.native_hash,
            l2_slot_us: SEC,
            l1_genesis_timestamp_us: 0,
            l1_block_us: 2 * SEC,
            t_dep_us: 5 * SEC,
            t_drift_us: 15 * SEC,
            metadata_window_us: 200_000,
            byte_limit: 65_536,
        };
        let setup = check_setup(&config, &genesis, dep_source.clone(), l1_source.clone()).unwrap();
        let plan = StaticScheduleProvider::new(config, registry).unwrap();
        let mut finalizer = setup.l1().finalizer();
        let l1_rows = l1_source
            .rows
            .iter()
            .map(|row| {
                finalizer
                    .finalize(&row.header, row.finalized_at_us)
                    .unwrap()
            })
            .collect();
        Self {
            setup,
            l1_source,
            dep_source,
            l1_rows,
            plan,
        }
    }

    pub fn descriptor(
        &self,
        r: u64,
        source_region: Option<RegionId>,
        reference: &Anchor,
        anchor: &Anchor,
    ) -> Descriptor {
        let plan = self.plan.resolve(
            &PlanInput {
                cause: if r == 0 {
                    BoundaryCause::Genesis
                } else {
                    BoundaryCause::Handoff
                },
                session: SessionId { beta: 0, r },
                source_region,
                window: Window {
                    h_min: 10 * r + 1,
                    h_max: 10 * r + 10,
                },
                reference_block: reference.block.clone(),
            },
            &FixtureReference(reference),
        );
        let PlanOutcome::Ready(plan) = plan else {
            panic!("fixture Plan must resolve: {plan:?}");
        };
        self.plan.descriptor(&plan, anchor).unwrap()
    }

    pub fn candidate(
        &self,
        anchor: &ExecutionAnchor,
        descriptor: &Descriptor,
        heights: &[u64],
    ) -> (Vec<PhysicalBlock>, ExecutionAnchor) {
        let mut delivered = self.setup.l1().reader();
        for row in &self.l1_rows {
            delivered
                .deliver_header(row, row.finalized_at_us())
                .unwrap();
        }
        let metadata = RegionalMetadata::new(self.setup.config(), &delivered).unwrap();
        let mut execution = anchor.state.clone();
        let mut physical_parent = anchor.anchor.block.block_id;
        let mut blocks = Vec::new();
        for height in anchor.anchor.block.height + 1..=*heights.last().unwrap() {
            let InputStatus::Ready(meta) = metadata.at(&anchor.anchor, height) else {
                panic!("fixture origin unavailable");
            };
            let InputStatus::Ready(dep) = self.setup.dep().at(&meta) else {
                panic!("fixture DEP unavailable");
            };
            let txs = if heights.contains(&height) {
                let nonce = execution.accounts[&1].nonce;
                let transfer = |nonce| {
                    Transfer {
                        from: 1,
                        to: 2,
                        nonce,
                        amount: 3,
                    }
                    .encode_raw()
                };
                let txs = vec![transfer(nonce), transfer(nonce)];
                let block = PhysicalBlock {
                    parent: physical_parent,
                    metadata: meta.clone(),
                    pre_state_root: execution.root().unwrap(),
                    transactions: txs.clone(),
                };
                physical_parent = block.digest().unwrap();
                blocks.push(block);
                txs
            } else {
                Vec::new()
            };
            execution = apply(&execution, &meta, &dep, &txs).unwrap().state;
        }
        let InputStatus::Ready(result) = merge(
            anchor,
            &blocks,
            &descriptor.plan.window,
            &metadata,
            self.setup.dep(),
            self.setup.config(),
        ) else {
            panic!("fixture chain construction");
        };
        let end = result.blocks.last().unwrap();
        let anchor = Anchor {
            block: BlockRef {
                height: end.state.metadata.height,
                origin: end.state.metadata.origin,
                block_id: physical_parent,
            },
            timestamp_us: end.state.metadata.timestamp_us,
        };
        (
            blocks,
            ExecutionAnchor {
                anchor,
                state: end.state.clone(),
                native_hash: end.native_hash,
            },
        )
    }
}

struct FixtureReference<'a>(&'a Anchor);

impl PublicReferenceView for FixtureReference<'_> {
    fn reference(&self, reference: &BlockRef) -> InputStatus<Anchor> {
        assert_eq!(*reference, self.0.block);
        InputStatus::Ready(self.0.clone())
    }
}

fn from_hex<const N: usize>(string: &str) -> [u8; N] {
    assert_eq!(string.len(), 2 * N);
    std::array::from_fn(|i| u8::from_str_radix(&string[2 * i..2 * i + 2], 16).unwrap())
}

#[derive(Clone, Copy, Default)]
pub struct NonCryptographicFinality;

impl VerificationBasis for NonCryptographicFinality {
    fn require(&self, descriptor: &Descriptor, body: &PhysicalBlock) -> InputStatus<()> {
        match descriptor.validate().and_then(|()| body.validate()) {
            Ok(()) => InputStatus::Ready(()),
            Err(error) => InputStatus::Reject(InputError::Malformed(error)),
        }
    }
}

impl FinalVerifier for NonCryptographicFinality {
    fn check(&self, descriptor: &Descriptor, body: &PhysicalBlock, proof: &[u8]) -> bool {
        matches!(proof.first(), Some(1 | 2)) && proof == proof_bytes(descriptor, body, proof[0])
    }
}

pub fn verification() -> VerificationProfile {
    VerificationProfile::NonCryptographicFixture { id: PROFILE.into() }
}

fn proof_bytes(descriptor: &Descriptor, body: &PhysicalBlock, variant: u8) -> Vec<u8> {
    let subject = hash_parts(
        PROFILE,
        &[
            &descriptor.encode_checked().unwrap(),
            body.digest().unwrap().as_bytes(),
            &[variant],
        ],
    );
    [&[variant][..], PROFILE.as_bytes(), &subject].concat()
}

pub fn body(block: &PhysicalBlock) -> LogMessage {
    publication(PublicObject::Body(block.clone()))
}

pub fn proof(descriptor: &Descriptor, block: &PhysicalBlock, variant: u8) -> LogMessage {
    publication(PublicObject::FinalProof {
        descriptor: descriptor.clone(),
        target: block.digest().unwrap(),
        proof: proof_bytes(descriptor, block, variant),
    })
}

fn publication(object: PublicObject) -> LogMessage {
    LogMessage {
        codec: PUBLICATION_CODEC,
        bytes: encode_publication(&object).unwrap(),
    }
}

pub struct Scenario {
    pub name: &'static str,
    pub parameters: GfrParameters,
    pub espresso: Vec<FinalizedBlock>,
    pub controls: BTreeMap<u64, Vec<ControlRef>>,
}

impl Scenario {
    pub fn new(name: &str, fixture: &Fixture) -> Self {
        match name {
            "G1" => g1(fixture),
            "G2" => g2(fixture),
            _ => panic!("expected G1 or G2"),
        }
    }

    pub fn l1_source(&self) -> Vec<u8> {
        format!("gfr-07/noncrypto-l1-{}", self.name).into_bytes()
    }

    pub fn bound(&self, index: usize) -> ReplayPosition {
        let block = &self.espresso[index].proposal;
        ReplayPosition {
            kind: ReplayKind::Espresso,
            key_us: block.theta_us.0,
            index: ReplayIndex::Espresso(block.q),
        }
    }
}

fn g1(fixture: &Fixture) -> Scenario {
    let genesis = fixture.setup.genesis();
    let initial = fixture.descriptor(0, None, &genesis.anchor, &genesis.anchor);
    let (current, a10) = fixture.candidate(genesis, &initial, &[1, 10]);
    let next = fixture.descriptor(1, Some(initial.plan.region), &initial.anchor, &a10.anchor);
    let (successor, _) = fixture.candidate(&a10, &next, &[11, 20]);
    let controls = [ReplayKind::Resume, ReplayKind::Force]
        .into_iter()
        .enumerate()
        .map(|(i, kind)| ControlRef {
            position: L1Position {
                block: 26,
                tx: 1,
                log: i as u64,
            },
            kind,
            control_seq: i as u64 + 1,
            source_bytes: format!("explicit-non-ABI-control-fixture-{i}").into_bytes(),
        })
        .collect();
    Scenario {
        name: "G1",
        parameters: GfrParameters {
            announcement_us: ANNOUNCEMENT,
            grace_us: 13 * SEC,
            expiry_us: EXPIRY,
        },
        espresso: finalize(vec![
            (
                30 * SEC,
                vec![
                    proof(&next, &successor[1], 1),
                    body(&successor[1]),
                    body(&successor[0]),
                ],
            ),
            (
                31 * SEC,
                vec![
                    body(&current[1]),
                    proof(&initial, &current[1], 1),
                    body(&current[0]),
                ],
            ),
            (54 * SEC - 1, Vec::new()),
            (
                54 * SEC,
                vec![LogMessage {
                    codec: 42,
                    bytes: vec![42],
                }],
            ),
            (61 * SEC, Vec::new()),
            (64 * SEC, Vec::new()),
        ]),
        controls: BTreeMap::from([(26, controls)]),
    }
}

fn g2(fixture: &Fixture) -> Scenario {
    let genesis = fixture.setup.genesis();
    let initial = fixture.descriptor(0, None, &genesis.anchor, &genesis.anchor);
    let (chain, _) = fixture.candidate(genesis, &initial, &[1, 3, 4, 9, 10]);
    let (_, a9) = fixture.candidate(genesis, &initial, &[1, 3, 4, 9]);
    let next = fixture.descriptor(1, Some(initial.plan.region), &initial.anchor, &a9.anchor);
    let (successor, _) = fixture.candidate(&a9, &next, &[11]);
    let mut wrong_anchor = a9.clone();
    wrong_anchor.anchor.block.block_id = Hash32::new([0x33; 32]);
    let wrong = fixture.descriptor(
        1,
        Some(initial.plan.region),
        &initial.anchor,
        &wrong_anchor.anchor,
    );
    let (wrong_successor, _) = fixture.candidate(&wrong_anchor, &wrong, &[11]);
    let mut simultaneous = vec![
        (&chain[1], proof(&initial, &chain[1], 1)),
        (&chain[2], proof(&initial, &chain[2], 2)),
    ];
    simultaneous.sort_by_key(|(block, _)| std::cmp::Reverse(block.digest().unwrap()));
    let mut q3 = vec![body(&chain[1])];
    q3.extend(simultaneous.into_iter().map(|(_, message)| message));
    Scenario {
        name: "G2",
        parameters: GfrParameters {
            announcement_us: ANNOUNCEMENT,
            grace_us: 3 * SEC,
            expiry_us: EXPIRY,
        },
        espresso: finalize(vec![
            (
                12 * SEC,
                vec![proof(&initial, &chain[0], 3), proof(&initial, &chain[0], 1)],
            ),
            (13 * SEC, vec![body(&chain[0])]),
            (
                14 * SEC,
                vec![proof(&initial, &chain[2], 1), body(&chain[2])],
            ),
            (15 * SEC, q3),
            (
                16 * SEC,
                vec![
                    proof(&initial, &chain[2], 2),
                    proof(&initial, &chain[2], 1),
                    body(&chain[3]),
                ],
            ),
            (
                24 * SEC - 1,
                vec![
                    proof(&initial, &chain[3], 1),
                    body(&successor[0]),
                    proof(&next, &successor[0], 1),
                    body(&wrong_successor[0]),
                    proof(&wrong, &wrong_successor[0], 1),
                ],
            ),
            (
                24 * SEC,
                vec![body(&chain[4]), proof(&initial, &chain[4], 1)],
            ),
            (52 * SEC, Vec::new()),
        ]),
        controls: BTreeMap::new(),
    }
}

pub fn finalize(blocks: Vec<(u64, Vec<LogMessage>)>) -> Vec<FinalizedBlock> {
    let mut source = SimEspresso::new(
        ESPRESSO_SOURCE.to_vec(),
        TimestampPolicy::Fixed { theta_us: 0 },
    )
    .unwrap();
    blocks
        .into_iter()
        .enumerate()
        .map(|(q, (theta_us, messages))| {
            source.set_policy(TimestampPolicy::Fixed { theta_us });
            let proposal = theta_us + (q as u64) * 1000;
            source.propose(TickUs(proposal), true, messages).unwrap();
            source
                .finalize(q as u64, TickUs(proposal + 100))
                .unwrap()
                .clone()
        })
        .collect()
}
