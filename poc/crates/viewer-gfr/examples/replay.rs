//! Bounded G1/G2 replay with separately delivered actual 06 and 03 readers.

#[path = "replay/fixture.rs"]
mod fixture;
#[path = "replay/trace.rs"]
mod trace;
#[path = "replay/viewer.rs"]
mod viewer;

use fixture::{Fixture, SEC, Scenario};
use poc_viewer_gfr::*;
use sim_espresso::FinalizedLogReader;
use viewer::Viewer;

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(args.len(), 3, "usage: replay G1|G2 7|42|99 0|2");
    let seed: u64 = args[1].parse().expect("seed");
    let pause: u64 = args[2].parse().expect("pause in milliseconds");
    assert!(matches!(seed, 7 | 42 | 99) && matches!(pause, 0 | 2));
    run(&args[0], seed, pause);
}

fn run(name: &str, seed: u64, pause: u64) {
    let fixture = Fixture::new();
    let scenario = Scenario::new(name, &fixture);
    trace::source(&fixture, &scenario);
    let mut a = Viewer::new(&fixture, &scenario, b"A", pause);
    let mut b = Viewer::new(&fixture, &scenario, b"B", pause);
    a.headers(&fixture, 0..=30);
    a.controls(&fixture, 0..=30);
    for block in &scenario.espresso {
        a.block(block);
    }
    assert_eq!(a.state.public_state(), b.state.public_state());
    match scenario.name {
        "G1" => g1(&fixture, &scenario, seed, &mut a, &mut b),
        "G2" => g2(&fixture, &scenario, seed, &mut a, &mut b),
        _ => unreachable!(),
    }
    assert_ne!(a.espresso.encode_snapshot(), b.espresso.encode_snapshot());
    eprintln!(
        "{} seed={seed} wall_pause_ms={pause} A_tip={:?} B_tip={:?}",
        scenario.name,
        a.espresso.tip(),
        b.espresso.tip()
    );
}

fn g1(f: &Fixture, s: &Scenario, seed: u64, a: &mut Viewer<'_>, b: &mut Viewer<'_>) {
    b.headers(f, (0..=10).rev());
    b.controls(f, (0..=9).rev());
    b.fragments(&s.espresso[2], seed, None);
    let missing = (seed as usize) % s.espresso[1].proposal.messages.len();
    b.fragments(&s.espresso[1], seed, Some(missing));
    b.fragments(&s.espresso[0], seed, None);
    let q0 = shared(f, a, b, &s.bound(0), "cache-before-selection");
    assert!(q0.state.public_state().entries.is_empty());
    assert_eq!(b.espresso.tip().unwrap().0, 0);
    let prior = b.state.public_state().clone();
    let q1 = s.bound(1);
    waiting(f, b, &q1, None, None, "partial-public-message-list");
    b.message(&s.espresso[1], missing);
    waiting(
        f,
        b,
        &q1,
        None,
        None,
        "covering-l1-control-list-undelivered",
    );
    b.controls(f, [10]);
    waiting(
        f,
        b,
        &q1,
        None,
        Some(&q1),
        "later-origin-not-in-control-cut",
    );
    b.headers(f, [12, 11]);
    waiting(f, b, &q1, Some(2), Some(&q1), "second-handoff-plan-wait");
    assert_eq!(*b.state.public_state(), prior);
    let admitted = shared(f, a, b, &q1, "two-handoffs-in-one-espresso-event");
    assert_eq!(admitted.added_records.len(), 1);
    assert_eq!(admitted.events.len(), 4);
    assert_eq!(admitted.state.public_state().entries.len(), 20);
    assert_eq!(admitted.state.checkpoint().cuts.l1_headers.len(), 13);
    assert_eq!(admitted.state.checkpoint().cuts.l1_controls.len(), 11);
    let checkpoint = b.state.checkpoint();
    assert_eq!(
        b.restore(f, checkpoint)
            .unwrap()
            .state
            .unwrap()
            .public_state(),
        a.state.public_state()
    );

    b.headers(f, (13..=30).rev());
    b.controls(f, (11..=25).rev());
    shared(f, a, b, &s.bound(2), "old-session-timers-skipped");
    let end = position(
        54 * SEC,
        ReplayKind::SessionEnd,
        ReplayIndex::Session(SessionId { beta: 0, r: 2 }),
    );
    waiting(
        f,
        b,
        &end,
        None,
        None,
        "watermark-one-us-before-session-end",
    );
    b.fragments(&s.espresso[3], seed, None);
    let timer = shared(f, a, b, &end, "session-end-without-new-physical");
    assert_eq!(timer.state.public_state().entries.len(), 20);
    assert_eq!(timer.state.checkpoint().cuts.espresso.len(), 4);
    assert!(timer.state.public_state().publications.ignored.is_empty());
    assert_eq!(
        b.restore(f, timer.state.checkpoint())
            .unwrap()
            .state
            .unwrap(),
        b.state
    );
    shared(
        f,
        a,
        b,
        &s.bound(3),
        "process-watermarker-only-at-its-event",
    );

    b.fragments(&s.espresso[4], seed, None);
    let expiry = position(61 * SEC, ReplayKind::Expire, ReplayIndex::Height(21));
    let halted = shared(f, a, b, &expiry, "expire-preserves-g");
    assert!(matches!(
        halted.state.public_state().tail,
        Tail::Halt { .. }
    ));
    shared(f, a, b, &s.bound(4), "halt-processes-espresso");
    b.fragments(&s.espresso[5], seed, None);
    waiting(
        f,
        b,
        &s.bound(5),
        None,
        None,
        "halt-control-header-without-complete-list",
    );
    b.controls(f, [26]);
    let r_a = a.advance(f, &s.bound(5), None);
    let r_b = b.advance(f, &s.bound(5), None);
    let control = position(
        64 * SEC,
        ReplayKind::Resume,
        ReplayIndex::L1(L1Position {
            block: 26,
            tx: 1,
            log: 0,
        }),
    );
    assert_eq!(
        r_a.status,
        ReplayStatus::UnsupportedControl {
            position: control,
            kind: ReplayKind::Resume
        }
    );
    comparable(
        "full-control-list-resume-precedes-force-and-espresso",
        &r_a,
        &r_b,
    );
}

fn g2(f: &Fixture, s: &Scenario, seed: u64, a: &mut Viewer<'_>, b: &mut Viewer<'_>) {
    b.controls(f, (0..=30).rev());
    b.headers(f, (0..=30).rev());
    b.fragments(&s.espresso[2], seed, None);
    b.fragments(&s.espresso[1], seed, Some(0));
    b.fragments(&s.espresso[0], seed, None);
    let proof_only = shared(f, a, b, &s.bound(0), "proof-only-public-history");
    assert!(proof_only.state.public_state().entries.is_empty());
    waiting(
        f,
        b,
        &s.bound(1),
        None,
        None,
        "public-body-locally-undelivered",
    );
    b.message(&s.espresso[1], 0);
    let first = shared(
        f,
        a,
        b,
        &s.bound(1),
        "first-completeness-at-body-publication",
    );
    assert_eq!(first.state.public_state().entries.len(), 1);
    let ancestor = shared(
        f,
        a,
        b,
        &s.bound(2),
        "target-with-missing-public-physical-ancestor",
    );
    assert_eq!(
        ancestor.state.public_state().entries,
        first.state.public_state().entries
    );
    b.fragments(&s.espresso[3], seed, None);
    let covered = shared(
        f,
        a,
        b,
        &s.bound(3),
        "descendant-and-canonical-target-order",
    );
    assert_eq!(covered.state.public_state().entries.len(), 4);
    assert!(
        covered.state.public_state().entries[1]
            .block
            .physical_digest
            .is_none()
    );
    assert_eq!(
        covered.state.public_state().entries[0]
            .block
            .native_block
            .kept
            .len(),
        1
    );

    b.fragments(&s.espresso[4], seed, None);
    let evidence = shared(
        f,
        a,
        b,
        &s.bound(4),
        "multiple-proof-variants-retain-first-q",
    );
    assert_eq!(
        evidence.state.public_state().entries,
        covered.state.public_state().entries
    );
    let retry = b.advance(f, &s.bound(4), None);
    assert!(retry.added_records.is_empty() && retry.events.is_empty());
    trace::step("exact-retry-idempotent", "B", &retry);
    b.fragments(&s.espresso[5], seed, None);
    let before = shared(
        f,
        a,
        b,
        &s.bound(5),
        "strict-before-cutoff-and-cached-successor",
    );
    assert_eq!(before.state.public_state().entries.len(), 9);

    b.fragments(&s.espresso[6], seed, None);
    let end = position(
        24 * SEC,
        ReplayKind::SessionEnd,
        ReplayIndex::Session(SessionId { beta: 0, r: 0 }),
    );
    let timer = shared(f, a, b, &end, "session-end-after-h9");
    let Tail::Next(next) = &timer.state.public_state().tail else {
        panic!("NEXT after h9");
    };
    assert_eq!(next.anchor.block.height, 9);
    assert_eq!(
        next.plan.window,
        Window {
            h_min: 11,
            h_max: 20
        }
    );
    assert_eq!(next.plan.reference_block.height, 0);
    let checkpoint = b.state.checkpoint();
    b.restore(f, checkpoint).unwrap();
    let successor = shared(
        f,
        a,
        b,
        &s.bound(6),
        "cutoff-equality-and-successor-at-current-key",
    );
    assert_eq!(successor.state.public_state().entries.len(), 11);
    assert!(
        successor.state.public_state().entries[9]
            .block
            .physical_digest
            .is_none()
    );
    let GlobalSource::Regional(key) = successor.state.public_state().entries[10].source else {panic!("expected regional source")};
    assert_eq!(
        successor.state.public_state().publications.results[&key].q_avail,
        Some(5)
    );
    assert_eq!(
        successor.state.public_state().entries[10].position,
        s.bound(6)
    );

    b.fragments(&s.espresso[7], seed, None);
    let expiry = position(52 * SEC, ReplayKind::Expire, ReplayIndex::Height(12));
    shared(f, a, b, &expiry, "expire-before-equal-key-espresso");
    shared(f, a, b, &s.bound(7), "halt-retains-accepted-prefix");
}

fn position(key_us: u64, kind: ReplayKind, index: ReplayIndex) -> ReplayPosition {
    ReplayPosition {
        key_us,
        kind,
        index,
    }
}

fn shared(
    f: &Fixture,
    a: &mut Viewer<'_>,
    b: &mut Viewer<'_>,
    bound: &ReplayPosition,
    label: &str,
) -> ReplayResult {
    let r_a = a.advance(f, bound, None);
    let r_b = b.advance(f, bound, None);
    assert_eq!(r_a.status, ReplayStatus::Ready, "{label}");
    assert_eq!(r_a.state.public_state().pc.as_ref(), Some(bound));
    comparable(label, &r_a, &r_b);
    r_a
}

fn comparable(label: &str, a: &ReplayResult, b: &ReplayResult) {
    assert_eq!(a.status, b.status, "{label}");
    assert_eq!(a.state.public_state(), b.state.public_state(), "{label}");
    assert_eq!(a.added_records, b.added_records, "{label}");
    assert_eq!(a.events, b.events, "{label}");
    trace::step(label, "A", a);
    trace::step(label, "B", b);
}

fn waiting(
    f: &Fixture,
    b: &mut Viewer<'_>,
    bound: &ReplayPosition,
    wait_plan: Option<u64>,
    expected: Option<&ReplayPosition>,
    label: &str,
) {
    let before = b.state.clone();
    let wait = b.advance(f, bound, wait_plan);
    let ReplayStatus::Wait {
        next_position,
        missing,
    } = &wait.status
    else {
        panic!("{label}: {:?}", wait.status);
    };
    assert_eq!(next_position.as_ref(), expected, "{label}");
    assert!(!missing.is_empty());
    assert_eq!(wait.state, before, "{label}");
    assert!(wait.added_records.is_empty() && wait.events.is_empty());
    trace::step(label, "B", &wait);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_g1_and_g2_replay() {
        run("G1", 7, 0);
        run("G2", 7, 0);
    }

    #[test]
    fn whole_interval_expiry_includes_padding() {
        let fixture = Fixture::new();
        let params = Scenario::new("G2", &fixture).parameters;
        assert!(!interval_unexpired(fixture.setup.config(), params, 10, 11, 50 * SEC).unwrap());
        assert!(interval_unexpired(fixture.setup.config(), params, 11, 11, 50 * SEC).unwrap());
        assert!(interval_unexpired(fixture.setup.config(), params, 10, 11, 50 * SEC - 1).unwrap());
    }
}
