//! Focused R17/R33 additions. These are bounded trusted-input component traces,
//! not evidence that the supplied references originated from real GFR/native I/O.

use h2_reducer_independent::fixtures::*;
use poc_session_manager::*;
use poc_types::domain::*;
use poc_viewer_gfr::{ReplayEffect, ReplayRecord, Tail};

fn new_manager(worker: u8) -> Manager {
    let cfg = config();
    Manager::new(
        binding_for(&cfg, worker),
        cfg.clone(),
        VerifiedGenesis {
            descriptor: genesis(&cfg),
            source_ref: h(0x60),
        },
    )
    .unwrap()
}

fn binding_for(cfg: &PublicConfig, worker: u8) -> ManagerBinding {
    ManagerBinding {
        manager_id: h(0xa0 + worker),
        worker: pid(worker),
        public_config: cfg.reference().unwrap(),
        gfr_source: h(0xb0),
        preparation_version: 1,
    }
}

fn source(seq: u64, record: ReplayRecord) -> ManagerInput {
    ManagerInput::ConsumeRecord(VerifiedReplayInput {
        source: h(0xb0),
        seq,
        record,
        validation_ref: h(0x62),
    })
}

fn install(m: &mut Manager, input: ManagerInput) {
    match m.plan(input, 1).unwrap() {
        PlanOutcome::Planned(d) => {
            let attestation = CommitRef::for_delta(&d, h(0xd0));
            m.accept_commit(d, attestation).unwrap();
        }
        PlanOutcome::Unchanged(_) => (),
        PlanOutcome::Wait(reason) => panic!("unexpected wait: {reason:?}"),
    }
}

fn snap(m: &Manager) -> String {
    let v = m.view();
    format!(
        "{:?}",
        (
            v.revision(),
            v.cursor(),
            v.next_source_seq(),
            v.programs(),
            v.actions(),
            v.guards()
        )
    )
}

#[test]
fn independent_branch_tombstone_exact16_then17_is_atomic_capacity() {
    // Structural scope-capacity fixture. M1's normal run does not activate these
    // additional branches, and this test does not claim Force/RESUME coverage.
    let mut m = new_manager(5);
    for branch in 0..16u64 {
        install(
            &mut m,
            source(branch, expiry(50_000_000 + branch, 1, branch)),
        );
    }
    assert_eq!(m.view().guards().closed_branches().len(), 16);
    assert_eq!(m.view().next_source_seq(), 16);
    let before = snap(&m);
    assert!(matches!(
        m.plan(source(16, expiry(50_000_016, 1, 16)), 1),
        Err(ManagerError::Capacity {
            kind: CapacityKind::Branches,
            limit: 16
        })
    ));
    assert_eq!(snap(&m), before);
    assert_eq!(m.view().guards().closed_branch(16), None);
    println!("PASS R33 branch scope exact16/+1; structural fixture, no M2 activation");
}

#[test]
fn independent_effect_list_exact128_then129_is_atomic_capacity() {
    // Typed source-envelope bound, deliberately using no-op Duty projections.
    // It tests the consumer's list capacity, not that GFR emits redundant Duties.
    let mut m = new_manager(5);
    let d = genesis(&config());
    let effect = ReplayEffect::Duty {
        before: Tail::Next(d.clone()),
        after: Tail::Next(d),
    };
    install(
        &mut m,
        source(0, record(20_000_000, 0, vec![], vec![effect.clone(); 128])),
    );
    assert_eq!(m.view().record(0).unwrap().record.effects.len(), 128);
    let before = snap(&m);
    assert!(matches!(
        m.plan(
            source(1, record(20_000_001, 1, vec![], vec![effect; 129])),
            1
        ),
        Err(ManagerError::Capacity {
            kind: CapacityKind::Effects,
            limit: 128
        })
    ));
    assert_eq!(snap(&m), before);
    println!("PASS R33 effects exact128/+1 and no partial cursor installation");
}

#[test]
fn independent_timestamp_addition_overflow_rejects_public_configuration() {
    let mut cfg = config();
    cfg.genesis.timestamp_us = u64::MAX - 1_000_000;
    let initial = genesis(&cfg);
    let outcome = Manager::new(
        binding_for(&cfg, 0),
        cfg,
        VerifiedGenesis {
            descriptor: initial,
            source_ref: h(0x60),
        },
    );
    assert!(matches!(outcome, Err(ManagerError::Overflow)));
    println!("PASS R33 checked timestamp overflow; no wrapped stop boundary");
}

#[test]
fn independent_height_increment_overflow_rejects_public_configuration() {
    let mut cfg = config();
    cfg.genesis.block.height = u64::MAX - 10;
    let mut p = genesis(&cfg).plan;
    p.window = Window {
        h_min: u64::MAX - 9,
        h_max: u64::MAX,
    };
    let initial = descriptor(&cfg, p, cfg.genesis.clone());
    let outcome = Manager::new(
        binding_for(&cfg, 0),
        cfg,
        VerifiedGenesis {
            descriptor: initial,
            source_ref: h(0x60),
        },
    );
    assert!(matches!(outcome, Err(ManagerError::Overflow)));
    println!("PASS R33 checked h_max+1 overflow; no wrapped session stop");
}

fn unique_anchor(height: u64) -> Anchor {
    let mut a = anchor(height, 0x55);
    let mut id = [0x55; 32];
    id[..8].copy_from_slice(&height.to_be_bytes());
    a.block.block_id = Hash32::new(id);
    a
}

#[test]
fn independent_program_exact64_via_candidates_and_regular_then65_rejects() {
    // Every candidate uses one selected predecessor's fixed10-height window;
    // every new selected session follows the exact regular reference/anchor rule.
    // Native proofs/execution are the stated trusted-input prerequisite.
    let cfg = config();
    let mut m = new_manager(5);
    let mut predecessor = genesis(&cfg);
    let mut seq = 0;
    loop {
        if predecessor.plan.session.r % 3 == 0 {
            for height in predecessor.plan.window.h_min..=predecessor.plan.window.h_max {
                let d = successor(&cfg, &predecessor, unique_anchor(height));
                let input = ManagerInput::RegisterCandidate(VerifiedCandidate {
                    descriptor: d.clone(),
                    predecessor: predecessor.cid,
                    provenance_ref: h(0x61),
                });
                if m.view().programs().len() == 64 {
                    let before = snap(&m);
                    assert!(matches!(
                        m.plan(input, 1),
                        Err(ManagerError::Capacity {
                            kind: CapacityKind::Programs,
                            limit: 64
                        })
                    ));
                    assert_eq!(snap(&m), before);
                    assert!(m.view().program(d.cid).is_none());
                    assert_eq!(m.view().programs().len(), 64);
                    println!("PASS R33 Programs exact64/+1 via six full candidate sets plus four");
                    return;
                }
                install(&mut m, input);
            }
        }
        assert!(
            seq < 24,
            "fixed fixture must reach64 before24 regular records"
        );
        let next = successor(
            &cfg,
            &predecessor,
            unique_anchor(predecessor.plan.window.h_max),
        );
        let effects = vec![append(
            predecessor.plan.window.h_min,
            predecessor.plan.window.h_max,
            predecessor.cid,
        )];
        let rec = record(
            10_000_000 + (predecessor.plan.window.h_max + 1) * 1_000_000,
            seq,
            vec![closed(&predecessor.plan.session), assigned(&next)],
            effects,
        );
        install(&mut m, source(seq, rec));
        seq += 1;
        predecessor = next;
    }
}
