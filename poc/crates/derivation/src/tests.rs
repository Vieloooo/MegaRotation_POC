//! Semantic fault fixtures. These maps are never admitted by the public 09 source capability.
use crate::source::{ObservedBlock, digest};
use crate::*;
use l1_adapter::{CursorWire, EventRecord, EventWire, LogWire, PolicyWire};
use poc_core::fixture::ValidatedDepFixture;
use poc_types::domain::{
    Account, Anchor, BlockRef, Credit, DepFixture, ExecutionAnchor, ExecutionState, Hash32,
    InputError, L1Header, L1Position, Metadata, PublicConfig,
};
use std::collections::BTreeMap;

struct Case {
    config: DerivationConfig,
    genesis: ExecutionAnchor,
    deposits: ValidatedDepFixture,
    identity: DerivationIdentity,
    headers: BTreeMap<u64, L1Header>,
    blocks: BTreeMap<u64, ObservedBlock>,
}

impl Case {
    fn new(
        genesis_origin: u64,
        genesis_seconds: u64,
        l1_slot_s: u64,
        l2_slot_s: u64,
        window: u64,
        deposit_rows: Vec<Vec<Credit>>,
    ) -> Self {
        let genesis_us = genesis_seconds * 1_000_000;
        let state = ExecutionState {
            metadata: Metadata {
                height: 0,
                timestamp_us: genesis_us,
                origin: genesis_origin,
            },
            accounts: [(
                1,
                Account {
                    nonce: 0,
                    balance: 10,
                },
            )]
            .into(),
        };
        let genesis = ExecutionAnchor {
            anchor: Anchor {
                block: BlockRef {
                    height: 0,
                    origin: genesis_origin,
                    block_id: Hash32::new([0x55; 32]),
                },
                timestamp_us: genesis_us,
            },
            state,
            native_hash: Hash32::new([0x11; 32]),
        };
        let dep = DepFixture {
            first_height: 1,
            rows: deposit_rows,
        };
        let l1_genesis_s = genesis_seconds
            .checked_sub(genesis_origin * l1_slot_s)
            .unwrap();
        let public = PublicConfig {
            chain_id: 12,
            rule_version: 1,
            verifier_version: 1,
            registry_hash: Hash32::new([9; 32]),
            dep_fixture_hash: dep.digest().unwrap(),
            l1_fixture_hash: Hash32::new([2; 32]),
            genesis: genesis.anchor.clone(),
            genesis_state_root: genesis.state.root().unwrap(),
            genesis_native_hash: genesis.native_hash,
            l2_slot_us: l2_slot_s * 1_000_000,
            l1_genesis_timestamp_us: l1_genesis_s * 1_000_000,
            l1_block_us: l1_slot_s * 1_000_000,
            t_dep_us: 0,
            t_drift_us: 15_000_000,
            metadata_window_us: 30_000_000,
            byte_limit: 250,
        };
        let config = DerivationConfig::new(public, window).unwrap();
        let identity = DerivationIdentity::new(
            &config,
            L1Identity::Fixture {
                commitment: config.public().l1_fixture_hash,
            },
        )
        .unwrap();
        let deposits = ValidatedDepFixture::load(config.public(), dep).unwrap();
        let mut case = Self {
            config,
            genesis,
            deposits,
            identity,
            headers: BTreeMap::new(),
            blocks: BTreeMap::new(),
        };
        for height in 0..=60 {
            let hash = Hash32::new([height as u8 + 1; 32]);
            case.headers.insert(
                height,
                L1Header {
                    height,
                    hash,
                    parent_hash: Hash32::new([height as u8; 32]),
                    timestamp_s: l1_genesis_s + height * l1_slot_s,
                },
            );
        }
        case
    }
    fn input(&self) -> VerifiedInputs<'_> {
        VerifiedInputs::cases(&self.identity, &self.headers, &self.blocks)
    }
    fn init(&self) -> Derivation {
        ready(Derivation::init(
            self.config.clone(),
            self.genesis.clone(),
            self.deposits.clone(),
            &self.input(),
        ))
    }
    fn restore(&self, cp: &Checkpoint) -> Outcome<Derivation> {
        Derivation::restore(
            cp,
            self.config.clone(),
            self.genesis.clone(),
            self.deposits.clone(),
            &self.input(),
        )
    }
    fn complete(&mut self, height: u64, events: Vec<EventRecord>) {
        let mut block = ObservedBlock::empty(self.headers[&height].clone());
        block.stamp.read_evidence_digest = digest(
            "PoC/Derivation/TestFixture/v1",
            &serde_json::to_vec(&events).unwrap(),
        );
        block.events = events;
        self.blocks.insert(height, block);
    }
    fn empty_prefix(&mut self, end: u64) {
        for height in 0..=end {
            self.complete(height, vec![]);
        }
    }
}

fn ready<T>(status: Outcome<T>) -> T {
    match status {
        Outcome::Ready(value) => value,
        Outcome::Wait(wait) => panic!("unexpected WAIT {wait:?}"),
        Outcome::Reject(error) => panic!("unexpected REJECT {error:?}"),
    }
}
fn log(block: u64, tx: u64, log: u64) -> LogWire {
    LogWire {
        block,
        tx,
        log,
        rpc_log_index: 2 * tx + log,
        tx_hash: format!("0x{:064x}", tx + 1),
    }
}
fn pair(block: u64, tx: u64, target: u64) -> [EventRecord; 2] {
    let first = log(block, tx, 0);
    let second = log(block, tx, 1);
    let cursor = |height, seq, source_kind| CursorWire {
        height,
        origin: 0,
        frames_digest: format!("0x{:064x}", 0),
        source_kind,
        source: format!("0x{seq:064x}"),
    };
    [
        EventRecord {
            position: first.clone(),
            event: EventWire::CursorSet {
                old: cursor(0, 0, 0),
                next: cursor(target, tx, 2),
                resume: second.clone(),
            },
        },
        EventRecord {
            position: second,
            event: EventWire::Resume {
                caller: "fixture".into(),
                request_id: format!("0x{:064x}", 22 + tx),
                seq: tx,
                policy: PolicyWire {
                    enabled_mask: 7,
                    order: [0, 1, 2],
                    l1_floor: 0,
                },
                height: target,
                origin: 0,
                act: 0,
                set: first,
            },
        },
    ]
}

#[test]
fn genesis_bindings_and_lateness() {
    let mut case = Case::new(10, 100, 4, 4, 40, vec![vec![]; 8]);
    for (inclusion, late) in [(49, false), (50, false), (51, true)] {
        assert_eq!(case.config.inclusion_is_late(10, inclusion), Ok(late));
    }
    assert!(matches!(
        case.config.window_end(u64::MAX),
        Err(Reject::Arithmetic(_))
    ));
    let mut wrong = case.genesis.clone();
    wrong.state.accounts.get_mut(&1).unwrap().balance += 1;
    assert!(matches!(
        Derivation::init(
            case.config.clone(),
            wrong,
            case.deposits.clone(),
            &case.input()
        ),
        Outcome::Reject(_)
    ));
    let header = case.headers.remove(&10).unwrap();
    assert!(matches!(
        Derivation::init(
            case.config.clone(),
            case.genesis.clone(),
            case.deposits.clone(),
            &case.input()
        ),
        Outcome::Wait(Wait {
            reason: WaitReason::L1Header(10),
            ..
        })
    ));
    case.headers.insert(10, header);
    let d = case.init();
    assert_eq!(d.current_epoch(), 10);
    assert!(!d.epoch_has_output());
}

#[test]
fn complete_boundary_first_epoch_and_another_window() {
    for slot in [2, 4] {
        let mut case = Case::new(10, 100, 4, slot, 40, vec![vec![]; 8]);
        case.empty_prefix(49);
        let mut d = case.init();
        let until = d.advance(&case.input());
        assert!(matches!(
            until.status,
            Outcome::Wait(Wait {
                reason: WaitReason::CompleteBlock(50),
                ..
            })
        ));
        assert!(until.new_items.is_empty());
        assert_eq!(d.complete_frontier().unwrap().height, 49);
        case.complete(50, vec![]);
        let missing = case.headers.remove(&11).unwrap();
        let until = d.advance(&case.input());
        assert!(matches!(
            until.status,
            Outcome::Wait(Wait {
                reason: WaitReason::L1Header(11),
                ..
            })
        ));
        assert!(until.new_items.is_empty());
        case.headers.insert(11, missing);
        let until = d.advance(&case.input());
        let origins: Vec<_> = until
            .new_items
            .iter()
            .map(|it| it.state.metadata.origin)
            .collect();
        assert_eq!(origins, [10, 11]);
        assert_eq!(d.complete_frontier().unwrap().height, 50);
        assert_eq!(d.current_epoch(), 11);
        assert!(d.epoch_has_output());
        let checked = ready(case.restore(&d.checkpoint()));
        assert_eq!(checked.checkpoint(), d.checkpoint());
        assert!(d.advance(&case.input()).new_items.is_empty());
        case.complete(51, vec![]);
        d.advance(&case.input());
        assert_eq!(d.current_epoch(), 12);
    }
}

#[test]
fn atomic_marker_retries_and_rejects_without_partial_fills() {
    let mut case = Case::new(0, 100, 2, 1, 40, vec![vec![]; 6]);
    case.empty_prefix(1);
    case.complete(2, pair(2, 0, 5).to_vec());
    let mut d = case.init();
    ready(d.step(&case.input()));
    ready(d.step(&case.input()));
    ready(d.step(&case.input()));
    ready(d.step(&case.input()));
    ready(d.step(&case.input())); // open marker's block
    let before = d.checkpoint();
    let next = case.headers.remove(&2).unwrap(); // after earlier staged fills, e+1 becomes necessary
    assert!(matches!(
        d.step(&case.input()),
        Outcome::Wait(Wait {
            reason: WaitReason::L1Header(2),
            ..
        })
    ));
    assert_eq!(d.checkpoint(), before);
    case.headers.insert(2, next);
    d.pending_deposit = Some(3);
    assert!(matches!(
        d.step(&case.input()),
        Outcome::Wait(Wait {
            reason: WaitReason::Deposit(poc_types::domain::InputMissing::Dep(3)),
            ..
        })
    ));
    assert_eq!(d.checkpoint(), before);
    d.pending_deposit = None;
    assert!(matches!(
        ready(d.step(&case.input())),
        Step::CursorSet { fill_count: 5, .. }
    ));
    let cut = ready(d.derive_at_resume(L1Position {
        block: 2,
        tx: 0,
        log: 1,
    }));
    assert_eq!(cut.target_anchor.anchor.block.origin, 2);
    assert_eq!(cut.target_height, 5);
    for (deposits, exhaustion) in [
        (vec![vec![], vec![]], true),
        (
            vec![
                vec![],
                vec![],
                vec![Credit {
                    to: 1,
                    amount: u64::MAX,
                }],
            ],
            false,
        ),
    ] {
        let mut failure = Case::new(0, 100, 2, 1, 40, deposits);
        failure.empty_prefix(1);
        failure.complete(2, pair(2, 0, 5).to_vec());
        let mut d = failure.init();
        for _ in 0..5 {
            ready(d.step(&failure.input()));
        }
        let cp = d.checkpoint();
        let failed = d.step(&failure.input());
        if exhaustion {
            assert!(matches!(
                failed,
                Outcome::Reject(Reject::Input(InputError::FixtureExhausted {
                    requested: 3,
                    ..
                }))
            ));
        } else {
            assert!(matches!(failed, Outcome::Reject(Reject::Execution(_))));
        }
        assert_eq!(d.checkpoint(), cp);
    }
}

#[test]
fn pair_and_checkpoint_tampering_are_rejected() {
    let mut case = Case::new(0, 100, 2, 1, 40, vec![vec![]; 8]);
    case.empty_prefix(1);
    let pair = pair(2, 0, 4);
    case.complete(2, vec![pair[0].clone()]);
    let mut d = case.init();
    assert!(matches!(
        d.advance(&case.input()).status,
        Outcome::Wait(Wait {
            reason: WaitReason::ResumePair(_),
            ..
        })
    ));
    let mut conflict = pair.clone();
    conflict[0].position.rpc_log_index = 1;
    case.complete(2, conflict.to_vec());
    assert!(matches!(
        d.advance(&case.input()).status,
        Outcome::Reject(Reject::Source(_))
    ));
    let mut invalid = case.init();
    assert!(matches!(
        invalid.advance(&case.input()).status,
        Outcome::Reject(Reject::Source(_))
    ));
    case.complete(2, pair.to_vec());
    let mut d = case.init();
    for _ in 0..6 {
        ready(d.step(&case.input()));
    }
    let cp = d.checkpoint();
    let from_disk = serde_json::to_value(&cp).unwrap();
    assert!(matches!(
        d.derive_at_resume(L1Position {
            block: 2,
            tx: 0,
            log: 0
        }),
        Outcome::Reject(Reject::NonResume(_))
    ));
    let mut variants = Vec::new();
    for field in [
        "epoch_has_output",
        "current_epoch",
        "last_log",
        "cuts",
        "frontier",
        "prefix_parent",
        "configuration",
        "action_count",
    ] {
        let mut tampered = from_disk.clone();
        match field {
            "epoch_has_output" => tampered[field] = false.into(),
            "current_epoch" => tampered[field] = 99.into(),
            "last_log" => tampered[field]["tx"] = 9.into(),
            "cuts" => {
                tampered[field][0]["evidence_digest"][0] = serde_json::Value::from(
                    (tampered[field][0]["evidence_digest"][0].as_u64().unwrap() + 1) % 256,
                )
            }
            "frontier" => tampered["covered"][0]["reference"]["hash"][0] = 2.into(),
            "prefix_parent" => tampered["derived"][0]["native_block"]["parent"][0] = 18.into(),
            "configuration" => tampered["identity"]["sequence_window_blocks"] = 99.into(),
            "action_count" => tampered["action_count"] = u64::MAX.into(),
            _ => unreachable!(),
        }
        variants.push((
            field,
            serde_json::from_value::<Checkpoint>(tampered).unwrap(),
        ));
    }
    for (field, tampered) in variants {
        assert!(
            matches!(
                case.restore(&tampered),
                Outcome::Reject(Reject::CheckpointMismatch)
            ),
            "{field}"
        );
    }
    case.complete(3, vec![]);
    assert_eq!(ready(case.restore(&cp)).checkpoint(), cp);
    case.blocks.remove(&1);
    assert!(matches!(case.restore(&cp), Outcome::Wait(_)));
}
