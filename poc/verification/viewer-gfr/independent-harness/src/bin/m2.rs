//! Source-derived transition checks; candidate outputs never compute expectations.
#[path = "../m2_fixture.rs"]
mod fixture;
#[path = "../m2_world.rs"]
mod world;
use fixture::*;
use poc_types::domain::*;
use poc_viewer_gfr::*;
use serde_json::{Value, json};
use sim_espresso::ReaderState;
use std::collections::BTreeMap;
use world::*;

#[derive(Default)]
struct Report {
    passed: usize,
    failed: usize,
    coverage: BTreeMap<String, String>,
}
impl Report {
    fn check(&mut self, id: &str, cases: &[&str], pass: bool, observed: Value) {
        if pass {
            self.passed += 1
        } else {
            self.failed += 1
        }
        for case in cases {
            let old = self
                .coverage
                .entry(case.to_string())
                .or_insert("PASS".into());
            if !pass {
                *old = "FAIL".into();
            }
        }
        println!(
            "{}",
            json!({"check":id,"cases":cases,"result":if pass{"PASS"}else{"FAIL"},"observed":observed})
        );
    }
    fn require(&mut self, id: &str, cases: &[&str], pass: bool) {
        self.check(id, cases, pass, json!(pass));
    }
}
fn head(s: &ReplayState) -> u64 {
    s.public_state()
        .entries
        .last()
        .map_or(0, |e| e.block.virtual_block.metadata.height)
}
fn next(s: &ReplayState) -> Option<&Descriptor> {
    match &s.public_state().tail {
        Tail::Next(d) => Some(d),
        _ => None,
    }
}
fn qavail(s: &ReplayState, k: &ResultKey) -> Option<u64> {
    s.public_state()
        .publications
        .results
        .get(k)
        .and_then(|r| r.q_avail)
}
fn waiting(s: &ReplayStatus, p: &ReplayPosition) -> bool {
    matches!(s,ReplayStatus::Wait {next_position:Some(x),..}if x==p)
}
fn rejected_init(result: &Result<InitResult, GfrError>) -> bool {
    matches!(
        result,
        Err(_)
            | Ok(InitResult {
                status: ReplayStatus::Reject(_),
                ..
            })
    )
}
fn state_trace(s: &ReplayState) -> Value {
    json!({"head":head(s),"pc":format!("{:?}",s.public_state().pc),"tail":format!("{:?}",s.public_state().tail),"records":s.public_state().records.len(),"closed_sessions":s.public_state().closed_sessions})
}

fn exact_chain(
    r: &mut Report,
    id: &str,
    s: &ReplayState,
    f: &Fixture,
    physical: &[PhysicalBlock],
    tx: Option<u64>,
    cases: &[&str],
) {
    let expected_physical: BTreeMap<_, _> = physical
        .iter()
        .map(|b| (b.metadata.height, b.digest().unwrap()))
        .collect();
    let good = s.public_state().entries.iter().all(|entry| {
        let h = entry.block.virtual_block.metadata.height;
        entry.block == f.expected_entry(h, expected_physical.get(&h).copied(), tx)
    });
    r.check(id,cases,good,json!({"heights":s.public_state().entries.len(),"native_and_virtual_and_accounts_match_hand_derived":good}));
}

fn genesis_cases(f: &Fixture, r: &mut Report) {
    let w = World::new(f, b"genesis");
    let s = w.init();
    let d = f.descriptor(0, &f.genesis().anchor, &f.genesis().anchor);
    r.require(
        "genesis-ready",
        &["VG01", "V02"],
        head(&s) == 0
            && s.public_state().pc.is_none()
            && next(&s) == Some(&d)
            && s.public_state().records.is_empty(),
    );
    let before = s.clone();
    for _ in 0..3 {
        let c = chain(&s);
        let duty = responsibility(&s);
        r.require(
            "pure-chain-responsibility",
            &["VG38"],
            c.pc == duty.pc && c.entries.is_empty() && duty.tail == &Tail::Next(d.clone()),
        );
    }
    r.require("queries-no-mutation", &["VG38"], s == before);
    w.plan.wait.set(Some(0));
    let blocked = w.init_input(&w.genesis_input()).unwrap();
    r.check(
        "genesis-plan-wait",
        &["VG02", "V02"],
        matches!(blocked.status, ReplayStatus::Wait { .. }) && blocked.state.is_none(),
        json!(format!("{:?}", blocked.status)),
    );
    w.plan.wait.set(None);
    r.require("genesis-plan-retry", &["VG02"], w.init() == s);
    w.plan.none.set(Some(0));
    let pending = w.init_input(&w.genesis_input()).unwrap();
    r.require(
        "genesis-plan-none",
        &["VG03", "V02"],
        matches!(
            pending.state.as_ref().map(|s| &s.public_state().tail),
            Some(Tail::Pending(_))
        ),
    );
    let mut bad = w.genesis_input();
    if let InitInput::Genesis { genesis, .. } = &mut bad {
        genesis.state.accounts.get_mut(&0).unwrap().balance += 1;
    }
    r.require(
        "genesis-state-mismatch",
        &["V02"],
        rejected_init(&w.init_input(&bad)),
    );
}

fn completeness_cases(f: &Fixture, r: &mut Report) {
    let d = f.descriptor(0, &f.genesis().anchor, &f.genesis().anchor);
    let chain3 = f.physical_chain(f.genesis(), &[3], Some(3));
    let b = &chain3[0];
    let k = key(&d, b);
    let mut w = World::new(f, b"proof-body");
    let initial = w.init();
    let p0 = ep(13_000_000, 0);
    let block0 = World::block(0, p0.key_us, vec![w.proof(&d, b)]);
    w.deliver(&block0);
    let first = w.advance(&initial, &p0);
    r.check(
        "proof-only-public-not-complete",
        &["VG08", "VG10", "V06"],
        matches!(first.status, ReplayStatus::Ready)
            && head(&first.state) == 0
            && qavail(&first.state, &k).is_none(),
        state_trace(&first.state),
    );
    let p1 = ep(14_000_000, 1);
    let block1 = World::block(1, p1.key_us, vec![w.body(b)]);
    w.deliver(&block1);
    let second = w.advance(&first.state, &p1);
    r.require(
        "body-later-qavail",
        &["VG08", "V06", "V07"],
        head(&second.state) == 3 && qavail(&second.state, &k) == Some(1),
    );
    r.require(
        "source-padding-and-acceptance",
        &["VG12", "VG17", "V12"],
        second
            .state
            .public_state()
            .entries
            .iter()
            .all(|e| e.source == GlobalSource::Regional(k) && e.position == p1),
    );
    exact_chain(
        r,
        "golden-first3-native",
        &second.state,
        f,
        &chain3,
        Some(3),
        &["V12", "VG12"],
    );
    let before = second.state.public_state().entries.clone();
    let block2 = World::block(2, 15_000_000, vec![w.proof(&d, b)]);
    w.deliver(&block2);
    let third = w.advance(&second.state, &ep(15_000_000, 2));
    r.require(
        "late-extra-proof-no-change",
        &["VG13", "V08"],
        qavail(&third.state, &k) == Some(1) && third.state.public_state().entries == before,
    );
    let again = w.advance(&third.state, &ep(15_000_000, 2));
    r.require(
        "repeated-advance-idempotent",
        &["VG35", "V20"],
        again.state == third.state && again.events.is_empty() && again.added_records.is_empty(),
    );

    let mut local = World::new(f, b"local-incomplete");
    let initial = local.init();
    let block = World::block(0, 13_000_000, local.result_messages(&d, &chain3));
    local.deliver_header(&block);
    local.deliver_message(&block, 1);
    let missing = local.advance(&initial, &p0);
    r.require(
        "local-public-message-missing-waits",
        &["VG09", "V06"],
        matches!(missing.status, ReplayStatus::Wait { .. })
            && missing.state == initial
            && local.tip().is_none(),
    );
    local.deliver_message(&block, 0);
    let complete = local.advance(&missing.state, &p0);
    r.require(
        "local-message-retry-original",
        &["VG09"],
        head(&complete.state) == 3 && qavail(&complete.state, &k) == Some(0),
    );

    let bodies = f.physical_chain(f.genesis(), &[3, 9], Some(3));
    let target = bodies.last().unwrap();
    let kt = key(&d, target);
    let mut ancestors = World::new(f, b"missing-ancestor");
    let initial = ancestors.init();
    let b0 = World::block(
        0,
        20_000_000,
        vec![ancestors.body(target), ancestors.proof(&d, target)],
    );
    ancestors.deliver(&b0);
    let a = ancestors.advance(&initial, &ep(20_000_000, 0));
    r.require(
        "physical-ancestor-not-padding",
        &["VG11", "V07"],
        head(&a.state) == 0 && qavail(&a.state, &kt).is_none(),
    );
    let b1 = World::block(1, 21_000_000, vec![ancestors.body(&bodies[0])]);
    ancestors.deliver(&b1);
    let a = ancestors.advance(&a.state, &ep(21_000_000, 1));
    r.require(
        "descendant-covers-physical-ancestors",
        &["VG12", "V12"],
        head(&a.state) == 9
            && qavail(&a.state, &kt) == Some(1)
            && a.state
                .public_state()
                .entries
                .iter()
                .all(|e| e.source == GlobalSource::Regional(kt)),
    );
    exact_chain(
        r,
        "descendant-native-chain",
        &a.state,
        f,
        &bodies,
        Some(3),
        &["V12"],
    );

    let mut invalid = World::new(f, b"invalid-proof");
    let initial = invalid.init();
    let b0 = World::block(
        0,
        13_000_000,
        vec![invalid.body(b), invalid.invalid_proof(&d, b)],
    );
    invalid.deliver(&b0);
    let a = invalid.advance(&initial, &p0);
    r.require(
        "invalid-proof-not-complete",
        &["VG20", "V07"],
        head(&a.state) == 0 && qavail(&a.state, &k).is_none(),
    );
    let b1 = World::block(1, 14_000_000, vec![invalid.proof(&d, b)]);
    invalid.deliver(&b1);
    let a = invalid.advance(&a.state, &p1);
    r.require(
        "valid-proof-after-invalid",
        &["VG20"],
        head(&a.state) == 3 && qavail(&a.state, &k) == Some(1),
    );
}

fn readiness_cases(f: &Fixture, r: &mut Report) {
    let d = f.descriptor(0, &f.genesis().anchor, &f.genesis().anchor);
    let bodies = f.physical_chain(f.genesis(), &[3], Some(3));
    let p = ep(13_000_000, 0);
    let mut w = World::new(f, b"basis");
    let initial = w.init();
    let block = World::block(0, p.key_us, w.result_messages(&d, &bodies));
    w.deliver(&block);
    w.basis.mode.set(1);
    let out = w.advance(&initial, &p);
    r.check("basis-wait-before-bool",&["VG07","VG40","V21"],waiting(&out.status,&p)&&out.state==initial&&out.events.is_empty()&&w.proofs.calls.get()==0&&w.basis.calls.get()>0,json!({"status":format!("{:?}",out.status),"bool_calls":w.proofs.calls.get(),"basis_calls":w.basis.calls.get()}));
    w.basis.mode.set(0);
    let done = w.advance(&out.state, &p);
    r.require(
        "basis-ready-retry",
        &["VG07", "V21"],
        head(&done.state) == 3 && w.proofs.calls.get() > 0,
    );
    w.binding = false;
    let no_binding_initial = w.init();
    r.require(
        "init-without-binding-retains-expected-profile",
        &["VG40", "V21"],
        no_binding_initial.public_state().verification == profile()
            && no_binding_initial.public_state() == initial.public_state(),
    );
    let out = w.advance(&initial, &p);
    r.require(
        "binding-unavailable-wait",
        &["VG40", "V21"],
        waiting(&out.status, &p) && out.state == initial,
    );
    w.binding = true;
    w.binding_profile = VerificationProfile::NonCryptographicFixture {
        id: "wrong-profile".into(),
    };
    let out = w.advance(&initial, &p);
    r.require(
        "bound-profile-mismatch",
        &["V21"],
        matches!(out.status, ReplayStatus::Reject(_)) && out.state == initial,
    );

    let mut w = World::new(f, b"control-gap");
    w.clear_l1();
    w.deliver_l1(30, 3);
    let initial = w.init();
    let block = World::block(0, 14_000_000, w.result_messages(&d, &bodies));
    w.deliver(&block);
    let p = ep(14_000_000, 0);
    let out = w.advance(&initial, &p);
    r.require(
        "header-not-control-coverage",
        &["VG06", "V04"],
        matches!(
            out.status,
            ReplayStatus::Wait {
                next_position: None,
                ..
            }
        ) && out.state == initial,
    );
    w.deliver_l1(30, 30);
    let out = w.advance(&out.state, &p);
    r.require("control-coverage-retry", &["VG06"], head(&out.state) == 3);

    let mut w = World::new(f, b"later-origin");
    w.params.announcement_us = 20_000_000;
    w.clear_l1();
    w.deliver_l1(3, 0);
    let initial = w.init();
    let block = World::block(0, 13_000_000, w.result_messages(&d, &bodies));
    w.deliver(&block);
    let p = ep(13_000_000, 0);
    let out = w.advance(&initial, &p);
    r.require(
        "origin-later-than-controls-waits",
        &["VG07", "V05"],
        waiting(&out.status, &p) && out.state == initial,
    );
    w.deliver_l1(4, 0);
    let out = w.advance(&out.state, &p);
    r.check(
        "later-header-does-not-demand-later-controls",
        &["VG06", "VG07", "V04", "V05"],
        head(&out.state) == 3 && matches!(out.status, ReplayStatus::Ready),
        state_trace(&out.state),
    );
    if head(&out.state) == 3 {
        let restored = w.init_input(&InitInput::Checkpoint(out.state.checkpoint()));
        r.require(
            "separate-header-control-cut-checkpoint",
            &["VG30", "V18"],
            restored
                .as_ref()
                .ok()
                .and_then(|o| o.state.as_ref())
                .is_some_and(|s| s.public_state() == out.state.public_state()),
        );
    }
}

fn regular_cases(f: &Fixture, r: &mut Report) {
    let d0 = f.descriptor(0, &f.genesis().anchor, &f.genesis().anchor);
    let old = f.physical_chain(f.genesis(), &[3, 10], Some(3));
    let a10 = f.execution_anchor(old.last().unwrap(), Some(3));
    let d1 = f.descriptor(1, &a10.anchor, &f.genesis().anchor);
    let successor = f.physical_chain(&a10, &[11], Some(3));
    let mut w = World::new(f, b"cached-successor");
    let initial = w.init();
    let b0 = World::block(0, 21_000_000, w.result_messages(&d1, &successor));
    let b1 = World::block(1, 22_000_000, w.result_messages(&d0, &old));
    w.deliver(&b0);
    w.deliver(&b1);
    let first = w.advance(&initial, &ep(21_000_000, 0));
    r.require(
        "ahead-download-cannot-leak",
        &["VG14", "V09"],
        head(&first.state) == 0 && qavail(&first.state, &key(&d0, old.last().unwrap())).is_none(),
    );
    let out = w.advance(&first.state, &ep(22_000_000, 1));
    let p = ep(22_000_000, 1);
    r.check(
        "cached-successor-same-event",
        &["VG14", "VG21", "V09", "V13"],
        head(&out.state) == 11
            && next(&out.state) == Some(&d1)
            && qavail(&out.state, &key(&d1, &successor[0])) == Some(0)
            && out.added_records.len() == 1
            && out.events.len() == 2,
        state_trace(&out.state),
    );
    r.require("same-event-lifecycle-and-fixed-duty",&["VG21","VG33","V13","V20"],out.events.iter().all(|e|e.position==p)&&out.events.iter().any(|e|matches!(e.body,LifecycleBody::SessionClosed {ref session,reason:CloseReason::Complete}if session.r==0))&&out.state.public_state().entries.last().is_some_and(|e|e.position==p));
    let all = [old.clone(), successor.clone()].concat();
    exact_chain(
        r,
        "handoff-native-execution",
        &out.state,
        f,
        &all,
        Some(3),
        &["V13"],
    );
    let before = out.state.public_state().records.len();
    let b2 = World::block(2, 24_000_000, vec![]);
    w.deliver(&b2);
    let out2 = w.advance(&out.state, &ep(24_000_000, 2));
    r.require(
        "old-end-no-second-handoff",
        &["VG24", "V16"],
        next(&out2.state) == Some(&d1)
            && out2.events.is_empty()
            && out2.state.public_state().records.len() == before + 1,
    );

    for later_wait in [false, true] {
        let mut w = World::new(
            f,
            if later_wait {
                b"metadata-rollback"
            } else {
                b"plan-rollback"
            },
        );
        let initial = w.init();
        let mut m = w.result_messages(&d0, &old);
        if later_wait {
            m.extend(w.result_messages(&d1, &successor));
            w.wait_metadata = Some(11);
        } else {
            w.plan.wait.set(Some(1));
        }
        let b = World::block(0, 22_000_000, m);
        w.deliver(&b);
        let p = ep(22_000_000, 0);
        let out = w.advance(&initial, &p);
        r.check(
            if later_wait {
                "whole-event-metadata-rollback"
            } else {
                "whole-event-plan-rollback"
            },
            &["VG26", "V15"],
            waiting(&out.status, &p)
                && out.state == initial
                && out.events.is_empty()
                && out.added_records.is_empty(),
            json!({"status":format!("{:?}",out.status),"unchanged":out.state==initial}),
        );
        w.wait_metadata = None;
        w.plan.wait.set(None);
        let out = w.advance(&out.state, &p);
        r.require(
            "whole-event-retry-one-record",
            &["VG26", "V15"],
            head(&out.state) == if later_wait { 11 } else { 10 }
                && out.added_records.len() == 1
                && out.events.len() == 2,
        );
    }

    let old9 = f.physical_chain(f.genesis(), &[3, 9], Some(3));
    let a9 = f.execution_anchor(old9.last().unwrap(), Some(3));
    let dnext = f.descriptor(1, &a9.anchor, &f.genesis().anchor);
    let mut w = World::new(f, b"end-at-nine");
    let initial = w.init();
    let b0 = World::block(0, 20_000_000, w.result_messages(&d0, &old9));
    w.deliver(&b0);
    let s = w.advance(&initial, &ep(20_000_000, 0)).state;
    let b1 = World::block(1, 24_000_000, vec![]);
    w.deliver(&b1);
    let out = w.advance(&s, &end(24_000_000, 0));
    r.check(
        "session-end-h9-fixed-window-reference",
        &["VG22", "V14"],
        head(&out.state) == 9 && next(&out.state) == Some(&dnext) && out.events.len() == 2,
        state_trace(&out.state),
    );
    let mut w = World::new(f, b"end-no-block");
    let initial = w.init();
    let b = World::block(0, 24_000_000, vec![]);
    w.deliver(&b);
    let out = w.advance(&initial, &end(24_000_000, 0));
    let expected = f.descriptor(1, &f.genesis().anchor, &f.genesis().anchor);
    r.require(
        "session-end-no-result-no-padding",
        &["VG23", "V14"],
        head(&out.state) == 0 && next(&out.state) == Some(&expected),
    );
}

fn boundary_cases(f: &Fixture, r: &mut Report) {
    let d = f.descriptor(0, &f.genesis().anchor, &f.genesis().anchor);
    let bodies = f.physical_chain(f.genesis(), &[3], Some(3));
    for (key, expected) in [(23_999_999, 3), (24_000_000, 0)] {
        let mut w = World::new(f, b"public-cutoff");
        let initial = w.init();
        let b = World::block(0, key, w.result_messages(&d, &bodies));
        w.deliver(&b);
        let out = w.advance(&initial, &ep(key, 0));
        r.check("public-cutoff-strict",&["VG15","V10"],head(&out.state)==expected,json!({"public_key":key,"end":24_000_000,"head":head(&out.state),"expected_head":expected}));
    }
    let mut w = World::new(f, b"watermark");
    let initial = w.init();
    let b0 = World::block(0, 23_999_999, vec![]);
    w.deliver(&b0);
    let out = w.advance(&initial, &end(24_000_000, 0));
    r.require(
        "timer-watermark-one-short",
        &["VG05", "VG36", "V03"],
        matches!(out.status, ReplayStatus::Wait { .. })
            && out.events.is_empty()
            && next(&out.state).unwrap().plan.session.r == 0,
    );
    let b1 = World::block(1, 23_999_999, vec![]);
    w.deliver(&b1);
    let out = w.advance(&out.state, &end(24_000_000, 0));
    r.require(
        "frozen-theta-not-clock",
        &["VG36", "V03"],
        matches!(out.status, ReplayStatus::Wait { .. }) && out.events.is_empty(),
    );
    let b2 = World::block(2, 24_000_000, vec![]);
    w.deliver(&b2);
    let out = w.advance(&out.state, &end(24_000_000, 0));
    r.require(
        "watermark-equality-allows-end",
        &["VG05", "V03"],
        next(&out.state).unwrap().plan.session.r == 1 && out.events.len() == 2,
    );

    let old = f.physical_chain(f.genesis(), &[3, 9], Some(3));
    let anchor = f.execution_anchor(old.last().unwrap(), Some(3));
    let d3 = f.descriptor(3, &anchor.anchor, &anchor.anchor);
    let later = f.physical_chain(&anchor, &[31], Some(3));
    for (key, expected) in [(49_999_999, 31), (50_000_000, 9)] {
        let mut w = World::new(f, b"padding-expiry");
        let initial = w.init();
        let b0 = World::block(0, 20_000_000, w.result_messages(&d, &old));
        w.deliver(&b0);
        let state = w.advance(&initial, &ep(20_000_000, 0)).state;
        let b1 = World::block(1, key, w.result_messages(&d3, &later));
        w.deliver(&b1);
        let out = w.advance(&state, &ep(key, 1));
        r.check("padding-expiry-whole-segment",&["VG16","VG17","V11"],head(&out.state)==expected,json!({"acceptance_key":key,"first_padding_expiry":50_000_000,"target_expiry":71_000_000,"head":head(&out.state),"expected_head":expected}));
        if expected == 9 {
            r.require(
                "expire-equality-halts-before-target",
                &["VG16", "VG27", "V11"],
                matches!(out.state.public_state().tail, Tail::Halt { .. }),
            );
        } else {
            exact_chain(
                r,
                "padding-range-native",
                &out.state,
                f,
                &[old.clone(), later.clone()].concat(),
                Some(3),
                &["VG17"],
            );
        }
    }
}

fn halt_and_controls(f: &Fixture, r: &mut Report) {
    for kind in [ReplayKind::Force, ReplayKind::Resume] {
        let mut w = World::new(f, b"halt-control");
        w.params.expiry_us = 3_000_000;
        let initial = w.init();
        let b0 = World::block(0, 14_000_000, vec![]);
        w.deliver(&b0);
        let halted = w.advance(&initial, &expiry(14_000_000, 1));
        r.require(
            "expire-preserves-chain-halts",
            &["VG27", "V16"],
            head(&halted.state) == 0
                && matches!(halted.state.public_state().tail, Tail::Halt { .. })
                && halted.state.public_state().closed_branches.contains(&0)
                && halted.events.len() == 1,
        );
        let control = ControlRef {
            position: L1Position {
                block: 5,
                tx: 0,
                log: 0,
            },
            kind,
            control_seq: 1,
            source_bytes: b"EXPLICIT-NONCRYPTO-CONTROL-FIXTURE".to_vec(),
        };
        w.l1.controls.insert(5, vec![control.clone()]);
        let b1 = World::block(1, 16_000_000, vec![]);
        w.deliver(&b1);
        let out = w.advance(&halted.state, &ep(16_000_000, 1));
        let expected = ReplayPosition {
            key_us: 16_000_000,
            kind,
            index: ReplayIndex::L1(control.position),
        };
        r.check("halt-unsupported-control-no-wake",&["VG28","V16"],matches!(&out.status,ReplayStatus::UnsupportedControl {position,..}if *position==expected)&&head(&out.state)==0&&matches!(out.state.public_state().tail,Tail::Halt {..})&&out.events.is_empty(),json!(format!("{:?}",out.status)));
    }
    let mut w = World::new(f, b"pending-expire");
    w.params.expiry_us = 3_000_000;
    w.plan.none.set(Some(0));
    let initial = w.init_input(&w.genesis_input()).unwrap().state.unwrap();
    let b = World::block(0, 14_000_000, vec![]);
    w.deliver(&b);
    let out = w.advance(&initial, &expiry(14_000_000, 1));
    r.require(
        "pending-expire",
        &["VG03", "VG27", "V16"],
        matches!(out.state.public_state().tail, Tail::Halt { .. }),
    );
    let mut w = World::new(f, b"mixed-controls");
    let initial = w.init();
    let resume = ControlRef {
        position: L1Position {
            block: 4,
            tx: 0,
            log: 3,
        },
        kind: ReplayKind::Resume,
        control_seq: 1,
        source_bytes: vec![1],
    };
    let force = ControlRef {
        position: L1Position {
            block: 4,
            tx: 1,
            log: 0,
        },
        kind: ReplayKind::Force,
        control_seq: 2,
        source_bytes: vec![2],
    };
    w.l1.controls.insert(4, vec![resume.clone(), force]);
    let b0 = World::block(0, 13_000_000, vec![]);
    let b1 = World::block(1, 14_000_000, vec![]);
    w.deliver(&b0);
    w.deliver(&b1);
    let out = w.advance(&initial, &ep(14_000_000, 1));
    r.check("mixed-controls-original-position",&["VG04","VG28","V04","V16"],matches!(&out.status,ReplayStatus::UnsupportedControl {position,kind:ReplayKind::Resume}if position.index==ReplayIndex::L1(resume.position))&&out.state.public_state().pc==Some(ep(13_000_000,0))&&out.added_records.len()==1&&out.events.is_empty(),state_trace(&out.state));
}

fn invalid_results(f: &Fixture, r: &mut Report) {
    let d = f.descriptor(0, &f.genesis().anchor, &f.genesis().anchor);
    let original = f.physical_chain(f.genesis(), &[3], Some(3))[0].clone();
    for case in [
        "parent",
        "root",
        "metadata",
        "origin",
        "window",
        "wrong_cid",
        "wrong_anchor",
    ] {
        let mut w = World::new(f, b"invalid-results");
        let initial = w.init();
        let mut b = original.clone();
        let mut descriptor = d.clone();
        match case {
            "parent" => b.parent = Hash32::new([0xfe; 32]),
            "root" => b.pre_state_root = Hash32::new([0xfe; 32]),
            "metadata" => b.metadata.timestamp_us += 1,
            "origin" => b.metadata.origin = 20,
            "window" => {
                b.metadata = Fixture::metadata(11);
                b.pre_state_root = Fixture::expected_state(10, Some(3)).root().unwrap();
            }
            "wrong_cid" => descriptor.cid = Cid::new(Hash32::new([0xfe; 32])),
            "wrong_anchor" => descriptor.anchor.block.block_id = Hash32::new([0xfd; 32]),
            _ => unreachable!(),
        }
        let block = World::block(0, 13_000_000, w.result_messages(&descriptor, &[b]));
        w.deliver(&block);
        let out = w.advance(&initial, &ep(13_000_000, 0));
        r.check(
            &format!("invalid-result/{case}"),
            &["VG18", "VG19", "V07", "V09"],
            head(&out.state) == 0,
            json!({"head":head(&out.state),"status":format!("{:?}",out.status)}),
        );
    }
}

fn checkpoint_and_prefix(f: &Fixture, r: &mut Report) {
    let d = f.descriptor(0, &f.genesis().anchor, &f.genesis().anchor);
    let bodies = f.physical_chain(f.genesis(), &[3], Some(3));
    let k = key(&d, &bodies[0]);
    let mut w = World::new(f, b"checkpoint");
    let initial = w.init();
    let b = World::block(0, 13_000_000, w.result_messages(&d, &bodies));
    w.deliver(&b);
    let accepted = w.advance(&initial, &ep(13_000_000, 0)).state;
    let cp = accepted.checkpoint();
    let restored = w.init_input(&InitInput::Checkpoint(cp.clone()));
    r.require(
        "checkpoint-verified-restore",
        &["VG30", "V18"],
        restored
            .as_ref()
            .ok()
            .and_then(|x| x.state.as_ref())
            .is_some_and(|s| s.public_state() == accepted.public_state()),
    );
    let mut cases: Vec<(&str, GlobalCheckpoint)> = vec![];
    macro_rules! tamper {
        ($name:expr,$edit:expr) => {{
            let mut c = cp.clone();
            ($edit)(&mut c);
            cases.push(($name, c));
        }};
    }
    tamper!("config", |c: &mut GlobalCheckpoint| c
        .state
        .config
        .chain_id += 1);
    tamper!("genesis", |c: &mut GlobalCheckpoint| c
        .state
        .genesis
        .state
        .accounts
        .get_mut(&0)
        .unwrap()
        .balance += 1);
    tamper!("parameters", |c: &mut GlobalCheckpoint| c
        .state
        .parameters
        .expiry_us += 1);
    tamper!("profile", |c: &mut GlobalCheckpoint| c.state.verification =
        VerificationProfile::NonCryptographicFixture {
            id: "relabel".into()
        });
    tamper!("espresso_source", |c: &mut GlobalCheckpoint| c
        .state
        .espresso_source
        .push(1));
    tamper!("l1_source", |c: &mut GlobalCheckpoint| c
        .state
        .l1_source
        .push(1));
    tamper!("branch", |c: &mut GlobalCheckpoint| c.state.branch = 4);
    tamper!(
        "tail",
        |c: &mut GlobalCheckpoint| if let Tail::Next(d) = &mut c.state.tail {
            d.anchor.block.block_id = Hash32::new([3; 32]);
        }
    );
    tamper!("pc", |c: &mut GlobalCheckpoint| c.state.pc = None);
    tamper!("selected", |c: &mut GlobalCheckpoint| c
        .state
        .selected
        .clear());
    tamper!("closed_sessions", |c: &mut GlobalCheckpoint| {
        c.state.closed_sessions.insert((0, 0));
    });
    tamper!("closed_branches", |c: &mut GlobalCheckpoint| {
        c.state.closed_branches.insert(0);
    });
    tamper!("native_hash", |c: &mut GlobalCheckpoint| c.state.entries
        [0]
    .block
    .native_hash =
        Hash32::new([4; 32]));
    tamper!("post_state", |c: &mut GlobalCheckpoint| c.state.entries
        [0]
    .block
    .state
    .accounts
    .get_mut(&0)
    .unwrap()
    .balance += 1);
    tamper!("virtual_root", |c: &mut GlobalCheckpoint| c.state.entries
        [0]
    .block
    .virtual_block
    .pre_state_root =
        Hash32::new([4; 32]));
    tamper!("source", |c: &mut GlobalCheckpoint| c.state.entries[0]
        .source =
        GlobalSource::Regional(ResultKey {
            cid: k.cid,
            block_hash: Hash32::new([4; 32])
        }));
    tamper!("acceptance_position", |c: &mut GlobalCheckpoint| c
        .state
        .entries[0]
        .position
        .key_us +=
        1);
    tamper!("records", |c: &mut GlobalCheckpoint| c
        .state
        .records
        .clear());
    tamper!("q_avail", |c: &mut GlobalCheckpoint| c
        .state
        .publications
        .results
        .get_mut(&k)
        .unwrap()
        .q_avail = Some(9));
    tamper!("publication_index", |c: &mut GlobalCheckpoint| c
        .state
        .publications
        .results
        .clear());
    tamper!("header_cut", |c: &mut GlobalCheckpoint| c.cuts.l1_headers
        [0]
    .hash =
        Hash32::new([5; 32]));
    tamper!("control_cut", |c: &mut GlobalCheckpoint| {
        c.cuts.l1_controls.clear();
    });
    tamper!("espresso_cut", |c: &mut GlobalCheckpoint| {
        c.cuts.espresso.clear();
    });
    for (name, cut) in cases {
        let got = w.init_input(&InitInput::Checkpoint(cut));
        r.check(
            &format!("checkpoint-tamper/{name}"),
            &["VG31", "V18"],
            rejected_init(&got),
            json!(format!("{:?}", got.as_ref().map(|x| &x.status))),
        );
    }
    let saved_e = std::mem::replace(
        &mut w.e,
        ReaderState::new(E_SOURCE.to_vec(), b"checkpoint-missing".to_vec()).unwrap(),
    );
    let missing = w.init_input(&InitInput::Checkpoint(cp.clone()));
    r.require(
        "checkpoint-missing-material-wait",
        &["VG32", "V18"],
        missing
            .as_ref()
            .is_ok_and(|x| x.state.is_none() && matches!(x.status, ReplayStatus::Wait { .. })),
    );
    w.e = saved_e;
    let b1 = World::block(1, 24_000_000, vec![]);
    w.deliver(&b1);
    let live = w.advance(&accepted, &ep(24_000_000, 1));
    if let Ok(InitResult {
        state: Some(restored),
        ..
    }) = restored
    {
        let resumed = w.advance(&restored, &ep(24_000_000, 1));
        r.require(
            "checkpoint-next-handoff-continuation",
            &["VG30", "V18"],
            live.state.public_state() == resumed.state.public_state()
                && live.events == resumed.events,
        );
    }
    let replacement = World::block(0, 13_000_000, vec![]);
    w.e = ReaderState::new(E_SOURCE.to_vec(), b"replacement-reader".to_vec()).unwrap();
    w.deliver(&replacement);
    let bad = w.advance(&accepted, &ep(13_000_000, 0));
    r.require(
        "nonextending-espresso-prefix",
        &["VG29", "V17"],
        matches!(bad.status, ReplayStatus::Reject(_)) && bad.state == accepted,
    );
}

fn independent_readers(f: &Fixture, r: &mut Report) {
    let d = f.descriptor(0, &f.genesis().anchor, &f.genesis().anchor);
    let bodies = f.physical_chain(f.genesis(), &[3], Some(3));
    let mut a = World::new(f, b"reader-A");
    let mut b = World::new(f, b"reader-B");
    let ia = a.init();
    let ib = b.init();
    let q0 = World::block(
        0,
        13_000_000,
        vec![a.proof(&d, &bodies[0]), a.proof(&d, &bodies[0])],
    );
    let _ = b.proof(&d, &bodies[0]);
    let _ = b.proof(&d, &bodies[0]);
    let q1 = World::block(1, 14_000_000, vec![a.body(&bodies[0])]);
    let q2 = World::block(2, 14_000_000, vec![]);
    for q in [&q0, &q1, &q2] {
        a.deliver(q);
    }
    let a0 = a.advance(&ia, &ep(13_000_000, 0));
    let a1 = a.advance(&a0.state, &ep(14_000_000, 1));
    let a2 = a.advance(&a1.state, &ep(14_000_000, 2));
    b.deliver(&q2);
    let blocked = b.advance(&ib, &ep(13_000_000, 0));
    r.require(
        "reader-B-future-only-waits",
        &["VG34", "VG37", "V19"],
        blocked.state == ib
            && b.tip().is_none()
            && matches!(blocked.status, ReplayStatus::Wait { .. }),
    );
    b.deliver_header(&q0);
    b.deliver_message(&q0, 1);
    r.require(
        "later-proof-arrives-first-not-complete-prefix",
        &["VG13", "VG34", "V08"],
        b.tip().is_none(),
    );
    b.deliver_message(&q0, 0);
    let b0 = b.advance(&blocked.state, &ep(13_000_000, 0));
    r.require(
        "readers-equal-at-q0",
        &["VG13", "VG34", "V08", "V19"],
        a0.state.public_state() == b0.state.public_state() && head(&b0.state) == 0,
    );
    b.deliver_header(&q1);
    let blocked = b.advance(&b0.state, &ep(14_000_000, 1));
    r.require(
        "reader-B-declared-message-gap",
        &["VG09", "VG34", "V19"],
        blocked.state == b0.state
            && b.tip() == Some(0)
            && matches!(blocked.status, ReplayStatus::Wait { .. }),
    );
    b.deliver_message(&q1, 0);
    let b1 = b.advance(&blocked.state, &ep(14_000_000, 1));
    let b2 = b.advance(&b1.state, &ep(14_000_000, 2));
    r.require(
        "readers-equal-at-q1-q2",
        &["VG34", "V19"],
        a1.state.public_state() == b1.state.public_state()
            && a2.state.public_state() == b2.state.public_state()
            && head(&b2.state) == 3,
    );
    exact_chain(
        r,
        "reader-independent-native-oracle",
        &b2.state,
        f,
        &bodies,
        Some(3),
        &["VG34", "V19"],
    );
}

fn historical_frontiers(f: &Fixture, r: &mut Report) {
    let mut w = World::new(f, b"timer-checkpoint-watermark");
    let initial = w.init();
    let initial_checkpoint = initial.checkpoint();
    let d1 = f.descriptor(1, &f.genesis().anchor, &f.genesis().anchor);
    let body = f.physical_chain(f.genesis(), &[11], Some(11));
    let watermark = World::block(0, 24_000_000, w.result_messages(&d1, &body));
    w.deliver(&watermark);
    let genesis_restored = w.init_input(&InitInput::Checkpoint(initial_checkpoint));
    r.require(
        "genesis-checkpoint-ahead-inputs-stays-genesis",
        &["VG30", "V18"],
        genesis_restored
            .as_ref()
            .ok()
            .and_then(|x| x.state.as_ref())
            .is_some_and(|s| s.public_state() == initial.public_state()),
    );
    let timer = w.advance(&initial, &end(24_000_000, 0));
    r.require(
        "timer-watermark-messages-not-indexed",
        &["VG14", "VG30", "V18"],
        head(&timer.state) == 0
            && qavail(&timer.state, &key(&d1, &body[0])).is_none()
            && timer.state.public_state().pc == Some(end(24_000_000, 0)),
    );
    let cut = timer.state.checkpoint();
    r.require(
        "timer-checkpoint-retains-watermark",
        &["VG30", "V18"],
        cut.cuts.espresso.iter().any(|b| b.proposal.q == 0),
    );
    let restored = w.init_input(&InitInput::Checkpoint(cut.clone()));
    r.require(
        "timer-checkpoint-does-not-process-watermark-message",
        &["VG30", "V18"],
        restored
            .as_ref()
            .ok()
            .and_then(|x| x.state.as_ref())
            .is_some_and(|s| s.public_state() == timer.state.public_state()),
    );
    if let Ok(InitResult {
        state: Some(restored),
        ..
    }) = restored
    {
        let after = w.advance(&restored, &ep(24_000_000, 0));
        let again = w.advance(&after.state, &ep(24_000_000, 0));
        r.require(
            "restored-watermark-next-event-once",
            &["VG30", "VG35", "V18"],
            head(&after.state) == 11
                && after.added_records.len() == 1
                && again.state == after.state
                && again.added_records.is_empty()
                && again.events.is_empty(),
        );
        exact_chain(
            r,
            "restored-new-window-native",
            &after.state,
            f,
            &body,
            Some(11),
            &["V18"],
        );
    }
    w.e = ReaderState::new(E_SOURCE.to_vec(), b"missing-watermark".to_vec()).unwrap();
    let absent = w.init_input(&InitInput::Checkpoint(cut));
    r.require(
        "timer-checkpoint-needs-supporting-watermark",
        &["VG32", "V18"],
        absent
            .as_ref()
            .is_ok_and(|x| x.state.is_none() && matches!(x.status, ReplayStatus::Wait { .. })),
    );

    let mut w = World::new(f, b"same-theta-frontier");
    let initial = w.init();
    for q in 0..=10 {
        w.deliver(&World::block(q, 24_000_000, vec![]));
    }
    let out = w.advance(&initial, &ep(24_000_000, 20));
    r.require(
        "same-theta-through-unseen-q-not-ready",
        &["VG36", "V03", "V19"],
        matches!(out.status, ReplayStatus::Wait { .. })
            && out.state.public_state().pc == Some(ep(24_000_000, 10)),
    );
    let earlier = w.advance(&out.state, &end(24_000_000, 0));
    r.require(
        "through-before-current-explicit-reject",
        &["VG38"],
        matches!(
            earlier.status,
            ReplayStatus::Reject(GfrError::HistoricalBound)
        ) && earlier.state == out.state,
    );
}

fn sorting_and_limits(f: &Fixture, r: &mut Report) {
    let d = f.descriptor(0, &f.genesis().anchor, &f.genesis().anchor);
    let bodies = f.physical_chain(f.genesis(), &[3, 9], Some(3));
    let k3 = key(&d, &bodies[0]);
    let k9 = key(&d, &bodies[1]);
    let early_source = if k3.block_hash < k9.block_hash {
        k3
    } else {
        k9
    };
    let mut outputs = vec![];
    for reverse in [false, true] {
        let mut w = World::new(
            f,
            if reverse {
                b"sort-reverse"
            } else {
                b"sort-forward"
            },
        );
        let initial = w.init();
        let mut messages = vec![
            w.body(&bodies[0]),
            w.body(&bodies[1]),
            w.proof(&d, &bodies[0]),
            w.proof(&d, &bodies[1]),
        ];
        if reverse {
            messages.reverse();
        }
        w.deliver(&World::block(0, 20_000_000, messages));
        let out = w.advance(&initial, &ep(20_000_000, 0));
        r.require(
            "target-canonical-hash-order-and-overlap",
            &["VG25", "V20", "V12"],
            head(&out.state) == 9
                && out.state.public_state().entries.iter().all(|entry| {
                    entry.source
                        == GlobalSource::Regional(
                            if entry.block.virtual_block.metadata.height <= 3 {
                                early_source
                            } else {
                                k9
                            },
                        )
                }),
        );
        outputs.push(out.state.public_state().entries.clone());
    }
    r.require(
        "target-input-permutation-stable-chain-source",
        &["VG25", "V20"],
        outputs[0] == outputs[1],
    );

    let mut w = World::new(f, b"conflicting-finality");
    let initial = w.init();
    let mut conflict = bodies[0].clone();
    conflict.transactions = vec![
        Transfer {
            from: 0,
            to: 1,
            nonce: 0,
            amount: 1,
        }
        .encode_raw(),
    ];
    let messages = vec![
        w.body(&bodies[0]),
        w.proof(&d, &bodies[0]),
        w.body(&conflict),
        w.proof(&d, &conflict),
    ];
    w.deliver(&World::block(0, 13_000_000, messages));
    let out = w.advance(&initial, &ep(13_000_000, 0));
    r.require(
        "inconsistent-finality-premise-explicit-reject",
        &["V17"],
        matches!(
            out.status,
            ReplayStatus::Reject(GfrError::InconsistentFinality(_))
        ) && out.state == initial,
    );

    let mut reader = World::new(f, b"reader-invalid-source");
    let q0 = World::block(0, 13_000_000, vec![]);
    reader.deliver(&q0);
    let q1 = World::block(1, 12_000_000, vec![]);
    let bad = reader
        .e
        .receive_block(E_SOURCE, q1, sim_espresso::TickUs(14_000_000));
    r.require(
        "reader-rejects-regressing-public-time",
        &["VG37", "V17"],
        bad.is_err() && reader.tip() == Some(0),
    );
    let bad = reader
        .e
        .receive_block(b"wrong-source", q0, sim_espresso::TickUs(14_000_000));
    r.require(
        "reader-rejects-source-mismatch",
        &["VG37", "V17"],
        bad.is_err() && reader.tip() == Some(0),
    );

    let short = Fixture::with_dep_limit(3);
    let d = short.descriptor(0, &short.genesis().anchor, &short.genesis().anchor);
    let bodies = short.physical_chain(short.genesis(), &[3, 9], Some(3));
    let mut w = World::new(&short, b"dep-exhaustion");
    let initial = w.init();
    let block = World::block(0, 20_000_000, w.result_messages(&d, &bodies));
    w.deliver(&block);
    let out = w.advance(&initial, &ep(20_000_000, 0));
    r.require(
        "finite-dep-exhaustion-explicit-error",
        &["V23"],
        matches!(out.status, ReplayStatus::Reject(_)) && out.state == initial,
    );

    let mut w = World::new(f, b"arithmetic-overflow");
    w.params.expiry_us = u64::MAX;
    let result = w.init_input(&w.genesis_input());
    let rejected = match result {
        Err(_) => true,
        Ok(InitResult {
            status: ReplayStatus::Reject(_),
            ..
        }) => true,
        Ok(InitResult {
            state: Some(state), ..
        }) => {
            w.deliver(&World::block(0, 13_000_000, vec![]));
            matches!(
                w.advance(&state, &ep(13_000_000, 0)).status,
                ReplayStatus::Reject(_)
            )
        }
        _ => false,
    };
    r.require(
        "timer-arithmetic-overflow-explicit-error",
        &["V23"],
        rejected,
    );
}

fn two_handoffs_one_event(f: &Fixture, r: &mut Report) {
    let d0 = f.descriptor(0, &f.genesis().anchor, &f.genesis().anchor);
    let old = f.physical_chain(f.genesis(), &[3, 10], Some(3));
    let a10 = f.execution_anchor(old.last().unwrap(), Some(3));
    let d1 = f.descriptor(1, &a10.anchor, &f.genesis().anchor);
    let successor = f.physical_chain(&a10, &[11, 20], Some(3));
    let a20 = f.execution_anchor(successor.last().unwrap(), Some(3));
    let d2 = f.descriptor(2, &a20.anchor, &a10.anchor);
    let mut w = World::new(f, b"two-handoffs");
    let initial = w.init();
    let mut messages = w.result_messages(&d1, &successor);
    messages.extend(w.result_messages(&d0, &old));
    let mut block = World::block(0, 22_000_000, messages);
    // Non-live public timestamp lag is permitted by E-S. This avoids claiming
    // two physical windows finalized before the first window's real deadline.
    block.proposal.proposed_at_us = sim_espresso::TickUs(31_000_000);
    block.proposal.live_at_proposal = false;
    block.finalized_at_us = sim_espresso::TickUs(31_000_010);
    w.deliver(&block);
    let p = ep(22_000_000, 0);
    let out = w.advance(&initial, &p);
    let closed: Vec<_> = out
        .events
        .iter()
        .filter_map(|e| match &e.body {
            LifecycleBody::SessionClosed {
                session,
                reason: CloseReason::Complete,
            } => Some(session.r),
            _ => None,
        })
        .collect();
    let assigned: Vec<_> = out
        .events
        .iter()
        .filter_map(|e| match &e.body {
            LifecycleBody::SessionAssigned { descriptor } => Some(descriptor.plan.session.r),
            _ => None,
        })
        .collect();
    r.check("two-full-handoffs-one-public-event", &["VG21","VG33","V13","V20"],
        head(&out.state)==20&&next(&out.state)==Some(&d2)&&out.added_records.len()==1
        &&out.events.len()==4&&closed==vec![0,1]&&assigned==vec![1,2]&&out.events.iter().all(|e|e.position==p),
        json!({"head":head(&out.state),"closed":closed,"assigned":assigned,"events":out.events.len(),"timing_premise":"non-live; noE-L/E-TSbounded-livenessclaim"}));
    exact_chain(
        r,
        "two-handoffs-native-content",
        &out.state,
        f,
        &[old, successor].concat(),
        Some(3),
        &["V13"],
    );
}

fn main() {
    println!(
        "{}",
        json!({"suite":"GFR-07-M2-independent","evidence":"finite pure-rule noncryptographic exact-subject proof fixture","l1":"actual03reg_a_l1_fixture0..30; explicitfinalizer/delivery; notAnvil","dep":"verifier-generated1..40, creditsat1/2/3 only","oracle":"manualaccounts/Kept and03canonicalhashing; noViewer/Apply/Merge for expected outputs"})
    );
    let f = Fixture::new();
    let mut r = Report::default();
    genesis_cases(&f, &mut r);
    completeness_cases(&f, &mut r);
    readiness_cases(&f, &mut r);
    regular_cases(&f, &mut r);
    boundary_cases(&f, &mut r);
    halt_and_controls(&f, &mut r);
    invalid_results(&f, &mut r);
    checkpoint_and_prefix(&f, &mut r);
    independent_readers(&f, &mut r);
    historical_frontiers(&f, &mut r);
    sorting_and_limits(&f, &mut r);
    two_handoffs_one_event(&f, &mut r);
    for n in 1..=40 {
        r.coverage
            .entry(format!("VG{n:02}"))
            .or_insert("INCONCLUSIVE_NOT_EXECUTED_IN_THIS_HARNESS".into());
    }
    r.coverage.insert(
        "VG39".into(),
        "INCONCLUSIVE_FUTURE_RESUME_MATERIAL_STAGING_GATE".into(),
    );
    r.coverage.insert(
        "V22".into(),
        "INCONCLUSIVE_ACTUAL_NATIVE_CERTIFICATES_UNAVAILABLE".into(),
    );
    if r.coverage.get("VG35").is_some_and(|x| x == "PASS") {
        r.coverage.insert(
            "VG35".into(),
            "INCONCLUSIVE_REPEAT_PASSED_WALL_PAUSE_MATRIX_PENDING".into(),
        );
    }
    for n in 1..=23 {
        r.coverage
            .entry(format!("V{n:02}"))
            .or_insert("INCONCLUSIVE_NOT_EXECUTED_IN_THIS_HARNESS".into());
    }
    println!(
        "{}",
        json!({"coverage":r.coverage,"summary":{"passed":r.passed,"failed":r.failed},"result":if r.failed==0{"PASS_WITH_EXPLICIT_LIMITS"}else{"FAIL"},"limits":"not fullVGcoverage/nativefinality/realcontrols/RESUME/15worker/safetyproof"})
    );
    if r.failed > 0 {
        std::process::exit(1);
    }
}
