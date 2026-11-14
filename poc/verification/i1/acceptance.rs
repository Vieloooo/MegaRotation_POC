//! Independent contract vectors. Production code supplies actual results only.
use poc_types::{effect::Kind as E, input::Kind as I, *};
use prost::Message;
use serde_json::{Value, json};
use sim_runtime::{
    Checkpoint, Identity, Launch, Retry,
    journal::{Journal, StepCommit, StepResult},
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

// Public Hub/Scenario APIs are imported unchanged for actual RPC tests. Their two
// embedded builder tests are incidental and excluded from our independent count.
#[path = "../../crates/sim-hub/src/hub.rs"]
mod hub;
#[path = "../../crates/sim-hub/src/scenario.rs"]
mod scenario;
#[path = "../../crates/sim-hub/src/transport.rs"]
mod transport;

const T: u64 = 1_000_000;
static NEXT: AtomicU64 = AtomicU64::new(0);
fn temp(label: &str) -> PathBuf {
    loop {
        let path = std::env::temp_dir().join(format!(
            "i1-verifier-{label}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&path) {
            Ok(()) => return path,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("reserve temporary directory {}: {error}", path.display()),
        }
    }
}
fn cid() -> Vec<u8> {
    vec![0x33; 32]
}
fn c() -> Cid {
    Cid { cid: cid() }
}
fn work(slot: u64) -> Work {
    Work {
        cid: cid(),
        slot,
        subject: vec![slot as u8; 32],
        duration_us: 5000,
    }
}
fn has(a: &Ack, pred: impl Fn(&E) -> bool) -> bool {
    a.effects.iter().any(|e| e.kind.as_ref().is_some_and(&pred))
}
fn rejected(a: &Ack) {
    assert_eq!(a.status, AckStatus::Rejected as i32);
    assert!(!has(a, |e| matches!(e, E::SignedIntent(_))));
}
fn get_ack(r: StepResult) -> Ack {
    match r {
        StepResult::Ack(a) => a,
        StepResult::Crash(_) => panic!("unexpected crash"),
    }
}

struct Rig {
    id: Identity,
    path: PathBuf,
    j: Option<Journal>,
    step: u64,
    inc: u64,
}
impl Rig {
    fn new(epsilon: i64) -> Self {
        let id = Identity {
            run_id: vec![0x11; 32],
            config_hash: vec![0x22; 32],
            node_id: if epsilon < 0 { 1 } else { 0 },
            epsilon_us: epsilon,
        };
        let path = temp("journal").join("journal");
        let j = Journal::open(&path, id.clone(), true).unwrap();
        Self {
            id,
            path,
            j: Some(j),
            step: 0,
            inc: 1,
        }
    }
    fn grant(&mut self, tick: u64, kind: I) -> Grant {
        self.step += 1;
        let mut g = Grant {
            schema_version: 1,
            run_id: self.id.run_id.clone(),
            node_id: self.id.node_id,
            incarnation: self.inc,
            logical_step_id: self.step,
            event_id: self.step,
            tick_us: tick,
            causal_wave: 0,
            tie_rank: hash_parts(
                "I1/tie/v1",
                &[&7u64.to_be_bytes(), &self.step.to_be_bytes()],
            ),
            input: Some(Input { kind: Some(kind) }),
            grant_hash: vec![],
        };
        g.grant_hash = grant_hash(&g);
        g
    }
    fn apply(&mut self, tick: u64, kind: I) -> Ack {
        let g = self.grant(tick, kind);
        get_ack(
            self.j
                .as_mut()
                .unwrap()
                .process(&g, self.inc, None, None)
                .unwrap(),
        )
    }
    fn state(&self) -> Value {
        serde_json::to_value(self.j.as_ref().unwrap().state()).unwrap()
    }
    fn sample(&self, request: u64, seq: u64, at: u64, end: u64) -> ClockSample {
        ClockSample {
            run_id: self.id.run_id.clone(),
            node_id: self.id.node_id,
            incarnation: self.inc,
            request_id: request,
            clock_domain: self.id.clock_domain(),
            sequence: seq,
            as_of_tick_us: at,
            node_time_at_as_of_us: at as i64 + self.id.epsilon_us,
            uncertainty_us: self.id.epsilon_us.unsigned_abs(),
            lease_end_tick_us: end,
            rate_profile_ref: sim_runtime::rate_profile(),
        }
    }
    fn ready(&mut self) {
        self.ready_with_stop(1_300_000);
    }
    fn ready_with_stop(&mut self, stop: i64) {
        self.apply(
            T,
            I::Create(Fixture {
                cid: cid(),
                stop_local_us: stop,
            }),
        );
        self.apply(T, I::AnchorReady(c()));
        self.apply(T, I::RequestSample(Empty {}));
        self.apply(T + 500, I::Sample(self.sample(1, 1, T, T + 100_000)));
        let a = self.apply(T + 501, I::Enter(c()));
        assert!(has(&a, |e| matches!(e, E::Entered(_))));
    }
    fn reopen(&mut self) {
        drop(self.j.take());
        self.inc += 1;
        self.j = Some(Journal::open(&self.path, self.id.clone(), false).unwrap());
    }
}

#[test]
fn independent_literal_hash_and_protobuf_vectors() {
    for (parts, expected) in [
        (
            vec![],
            "f730b47439b0b89e3d56cd563a2c54df760475d9d244bc51aeee1be7764b49a5",
        ),
        (
            vec![b"".as_slice()],
            "f5211e884dbe1e7a8408825b4bbe41c3f1ad05315d6bc68da66be1751cfdc3c7",
        ),
        (
            vec![b"ab".as_slice(), b"c".as_slice()],
            "7723ee2fac31fa8e287c050898ea054e08e2604a3ad603687bdc9c8babf29e79",
        ),
        (
            vec![b"a".as_slice(), b"bc".as_slice()],
            "caa068eb03c8c08b7ea4a8614714c7c268883bc4572f33a98ce0baf450ba98b5",
        ),
    ] {
        assert_eq!(hex(&hash_parts("I1/test/v1", &parts)), expected);
    }
    let tie = hash_parts("I1/tie/v1", &[&7u64.to_be_bytes(), &9u64.to_be_bytes()]);
    assert_eq!(
        hex(&tie),
        "4a627d335c54960d4e399bfde1204228969592a05258b1180ee5ef798b908736"
    );
    let mut g = Grant {
        schema_version: 1,
        run_id: vec![0x11; 32],
        logical_step_id: 7,
        event_id: 9,
        tick_us: T,
        tie_rank: tie,
        input: Some(Input {
            kind: Some(I::RequestSample(Empty {})),
        }),
        ..Default::default()
    };
    assert_eq!(
        hex(&grant_hash(&g)),
        "6a02bca1613654aedf21d5813767dc844c933ab8fbfa569a7b5ec2e325aa36e3"
    );
    assert!(g.encode_to_vec().ends_with(&[0x52, 2, 0x0a, 0])); // present empty operation
    let a = Ack {
        schema_version: 1,
        run_id: vec![0x11; 32],
        logical_step_id: 7,
        event_id: 9,
        grant_hash: grant_hash(&g),
        effects: vec![Effect {
            kind: Some(E::RequestSample(ClockRequest { request_id: 1 })),
        }],
        durable_journal_seq: 1,
        state_hash: vec![0x55; 32],
        ..Default::default()
    };
    assert_eq!(
        hex(&ack_hash(&a)),
        "029519ded250b4e51d532966aefdbdfdd79c68d220a0a925235ee875baa28ed4"
    );
    g.incarnation = 9;
    assert_eq!(
        hex(&grant_hash(&g)),
        "6a02bca1613654aedf21d5813767dc844c933ab8fbfa569a7b5ec2e325aa36e3"
    );
}

#[test]
fn independent_decode_merge_presence_and_invalid_control() {
    // Input.BeginWork repeated: first embeds slot=7; second duration=5. Merge, not replace.
    let merged = Input::decode([0x2a, 2, 0x10, 7, 0x2a, 2, 0x20, 5].as_slice()).unwrap();
    let Some(I::BeginWork(w)) = merged.kind else {
        panic!("wrong variant")
    };
    assert_eq!((w.slot, w.duration_us), (7, 5));
    let switched = Input::decode([0x2a, 2, 0x10, 7, 0x0a, 0].as_slice()).unwrap();
    assert!(matches!(switched.kind, Some(I::RequestSample(_))));
    assert!(Input::decode([0x2a, 0x80].as_slice()).is_err());
    let mut r = Rig::new(1000);
    let mut g = r.grant(T, I::RequestSample(Empty {}));
    let mut raw = g.encode_to_vec();
    raw.extend([0x08, 2]);
    let changed = Grant::decode(raw.as_slice()).unwrap();
    assert_eq!(changed.schema_version, 2);
    assert!(
        r.j.as_mut()
            .unwrap()
            .process(&changed, 1, None, None)
            .is_err()
    );
    raw = g.encode_to_vec();
    raw.extend([0xc0, 0x0c, 1]); // unknown scalar tag200
    assert_eq!(Grant::decode(raw.as_slice()).unwrap(), g);
    for mode in 0..4 {
        let mut bad = g.clone();
        match mode {
            0 => bad.input = Some(Input { kind: None }),
            1 => bad.node_id = 1,
            2 => bad.event_id = 0,
            _ => bad.run_id = vec![9; 32],
        }
        bad.grant_hash = grant_hash(&bad);
        assert!(r.j.as_mut().unwrap().process(&bad, 1, None, None).is_err());
    }
    g.grant_hash[0] ^= 1;
    assert!(r.j.as_mut().unwrap().process(&g, 1, None, None).is_err());
    assert_eq!(r.j.as_ref().unwrap().sequence(), 0);
}

#[test]
fn historical_sample_and_same_node_clock() {
    for eps in [-1000, 1000] {
        let mut r = Rig::new(eps);
        r.apply(
            T,
            I::Create(Fixture {
                cid: cid(),
                stop_local_us: 1_300_000,
            }),
        );
        r.apply(T, I::RequestSample(Empty {}));
        r.apply(T + 10_000, I::Sample(r.sample(1, 1, T, T + 100_000)));
        let a = r.apply(T + 10_000, I::ReadClock(c()));
        let reads: Vec<_> = a
            .effects
            .iter()
            .filter_map(|e| {
                if let Some(E::ClockRead(x)) = &e.kind {
                    Some((x.caller.as_str(), x.node_time_us))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            reads,
            vec![("Manager", 1_010_000 + eps), ("BFT", 1_010_000 + eps)]
        );
    }
}

#[test]
fn atomic_sample_rejection_and_sequence_poisoning() {
    let mut r = Rig::new(1000);
    r.ready();
    r.apply(T + 2000, I::RequestSample(Empty {}));
    r.apply(T + 2000, I::RequestSample(Empty {}));
    let valid = r.sample(3, 3, T + 2000, T + 102_000);
    for mode in 0..10 {
        let mut bad = valid.clone();
        match mode {
            0 => {
                bad.sequence = 100;
                bad.node_time_at_as_of_us += 1
            }
            1 => bad.run_id = vec![0; 32],
            2 => bad.node_id = 1,
            3 => bad.incarnation = 0,
            4 => bad.clock_domain = vec![0; 32],
            5 => bad.request_id = 99,
            6 => bad.as_of_tick_us = T + 10_000,
            7 => bad.lease_end_tick_us = T + 3000,
            8 => bad.uncertainty_us = 999,
            _ => bad.rate_profile_ref = vec![0; 32],
        }
        let before = r.state();
        let a = r.apply(T + 3000, I::Sample(bad));
        rejected(&a);
        let after = r.state();
        for field in [
            "sample",
            "accepted_sequence",
            "prior_lease_end_tick_us",
            "outstanding_requests",
        ] {
            assert_eq!(before[field], after[field], "mutation {mode}/{field}");
        }
    }
    let a = r.apply(T + 3000, I::Sample(valid.clone()));
    assert!(has(&a, |e| matches!(e, E::SampleAccepted(_))));
    assert_eq!(r.state()["accepted_sequence"], 3);
    assert_eq!(r.state()["outstanding_requests"], json!([2]));
    rejected(&r.apply(T + 3001, I::Sample(valid)));
    rejected(&r.apply(T + 3002, I::Sample(r.sample(2, 2, T + 2000, T + 102_000))));
}

#[test]
fn lease_and_stop_guard_exact_equality() {
    for d in [-1i64, 0, 1] {
        let mut r = Rig::new(1000);
        r.ready();
        let due = (T + 100_000) as i64 + d;
        let w = work(1);
        r.apply((due - 5000) as u64, I::BeginWork(w.clone()));
        let a = r.apply(due as u64, I::WorkCompleted(w));
        assert_eq!(has(&a, |e| matches!(e, E::SignedIntent(_))), d < 0);
        assert_eq!(r.state()["prior_lease_end_tick_us"], T + 100_000);
    }
    let mut r = Rig::new(1000);
    r.ready();
    let w = work(1);
    r.apply(T + 1000, I::BeginWork(w.clone()));
    r.apply(T + 2000, I::Close(c()));
    rejected(&r.apply(T + 6000, I::WorkCompleted(w)));
    let terminal = r.state()["fixtures"][hex(&cid())]["terminal"].clone();
    r.apply(T + 7000, I::Close(c()));
    assert_eq!(terminal, r.state()["fixtures"][hex(&cid())]["terminal"]);
}

#[test]
fn explicit_equal_tick_close_orders_and_local_stop_boundaries() {
    for close_first in [true, false] {
        let mut r = Rig::new(1000);
        r.ready();
        let w = work(1);
        r.apply(T + 1000, I::BeginWork(w.clone()));
        if close_first {
            r.apply(T + 6000, I::Close(c()));
        }
        let a = r.apply(T + 6000, I::WorkCompleted(w));
        assert_eq!(has(&a, |e| matches!(e, E::SignedIntent(_))), !close_first);
        if !close_first {
            r.apply(T + 6000, I::Close(c()));
        }
        assert_eq!(
            r.state()["fixtures"][hex(&cid())]["terminal"]["reason"],
            "Close"
        );
        assert_eq!(
            r.state()["fixtures"][hex(&cid())]["signed_intents"]
                .as_object()
                .unwrap()
                .len(),
            usize::from(!close_first)
        );
    }
    for epsilon in [-1000, 1000] {
        for delta in [-1, 0, 1] {
            let mut r = Rig::new(epsilon);
            r.ready_with_stop(1_022_000);
            let due = (1_022_000 - epsilon + delta) as u64;
            let w = work(1);
            r.apply(due - 5000, I::BeginWork(w.clone()));
            let a = r.apply(due, I::WorkCompleted(w));
            assert_eq!(has(&a, |e| matches!(e, E::SignedIntent(_))), delta < 0);
            if delta >= 0 {
                assert_eq!(
                    r.state()["fixtures"][hex(&cid())]["terminal"]["reason"],
                    "Stop"
                );
            }
        }
    }
}

#[test]
fn absolute_timer_deadline_and_generation() {
    for eps in [-1000, 1000] {
        let mut r = Rig::new(eps);
        r.ready();
        let timer = Timer {
            timer_id: 1,
            generation: 1,
            cid: cid(),
            local_deadline_us: 1_050_000,
        };
        r.apply(T + 1000, I::ArmTimer(timer.clone()));
        r.apply(T + 2000, I::RequestSample(Empty {}));
        r.apply(T + 3000, I::Sample(r.sample(2, 2, T + 2000, T + 102_000)));
        let fire = (1_050_000 - eps) as u64;
        rejected(&r.apply(fire - 1, I::TimerFired(timer.clone())));
        assert!(has(
            &r.apply(fire, I::TimerFired(timer.clone())),
            |e| matches!(e, E::TimerObserved(_))
        ));
        rejected(&r.apply(fire, I::TimerFired(timer.clone())));
        let mut next = timer.clone();
        next.generation = 2;
        next.local_deadline_us = 1_060_000;
        r.apply(fire, I::ArmTimer(next.clone()));
        r.apply(fire, I::CancelTimer(next.clone()));
        rejected(&r.apply(fire, I::TimerFired(next)));
    }
}

#[test]
fn regression_precommit_retry_cannot_bypass_recalibration() {
    for checkpoint in [Checkpoint::BeforeAppend, Checkpoint::PartialTail] {
        for harmless in [false, true] {
            let mut r = Rig::new(1000);
            r.ready();
            let w = work(1);
            r.apply(T + 1000, I::BeginWork(w.clone()));
            let original = r.grant(
                T + 6000,
                if harmless {
                    I::ReadClock(c())
                } else {
                    I::WorkCompleted(w.clone())
                },
            );
            assert!(matches!(
                r.j.as_mut()
                    .unwrap()
                    .process(&original, 1, None, Some(checkpoint))
                    .unwrap(),
                StepResult::Crash(_)
            ));
            r.reopen();
            let mut retry_grant = original.clone();
            retry_grant.incarnation = 2;
            let retry = Retry {
                logical_step_id: original.logical_step_id,
                grant_hash: original.grant_hash,
                original_incarnation: 1,
            };
            assert!(
                r.j.as_mut()
                    .unwrap()
                    .process(&retry_grant, 2, Some(&retry), None)
                    .is_err()
            );
            let direct = r.grant(T + 6000, I::WorkCompleted(w.clone()));
            assert!(
                r.j.as_mut()
                    .unwrap()
                    .process(&direct, 2, None, None)
                    .is_err()
            );
            r.apply(T + 6000, I::Recover(Empty {}));
            rejected(&r.apply(T + 6000, I::ReadClock(c())));
            r.apply(T + 6500, I::Sample(r.sample(2, 2, T + 6000, T + 106_000)));
            assert!(has(&r.apply(T + 6501, I::WorkCompleted(w)), |e| matches!(
                e,
                E::SignedIntent(_)
            )));
        }
    }
}

#[test]
fn durable_ack_replay_and_pre_head_promotion_do_not_resign() {
    for checkpoint in [
        Checkpoint::AfterSyncBeforeAck,
        Checkpoint::AfterRecordSyncBeforeHead,
    ] {
        let mut r = Rig::new(1000);
        r.ready();
        let w = work(1);
        r.apply(T + 1000, I::BeginWork(w.clone()));
        let mut g = r.grant(T + 6000, I::WorkCompleted(w));
        assert!(matches!(
            r.j.as_mut()
                .unwrap()
                .process(&g, 1, None, Some(checkpoint))
                .unwrap(),
            StepResult::Crash(_)
        ));
        r.reopen();
        let before = r.state();
        let seq = r.j.as_ref().unwrap().sequence();
        g.incarnation = 2;
        assert!(r.j.as_mut().unwrap().process(&g, 2, None, None).is_err());
        let retry = Retry {
            logical_step_id: g.logical_step_id,
            grant_hash: g.grant_hash.clone(),
            original_incarnation: 1,
        };
        let a = get_ack(
            r.j.as_mut()
                .unwrap()
                .process(&g, 2, Some(&retry), None)
                .unwrap(),
        );
        assert!(has(&a, |e| matches!(e, E::SignedIntent(_))));
        assert_eq!(
            a,
            get_ack(
                r.j.as_mut()
                    .unwrap()
                    .process(&g, 2, Some(&retry), None)
                    .unwrap()
            )
        );
        assert_eq!(before, r.state());
        assert_eq!(seq, r.j.as_ref().unwrap().sequence());
        let mut altered = g.clone();
        if let Some(I::WorkCompleted(work)) = altered.input.as_mut().and_then(|i| i.kind.as_mut()) {
            work.subject[0] ^= 1;
        }
        altered.grant_hash = grant_hash(&altered);
        let changed_authorization = Retry {
            grant_hash: altered.grant_hash.clone(),
            ..retry.clone()
        };
        assert!(
            r.j.as_mut()
                .unwrap()
                .process(&altered, 2, Some(&changed_authorization), None)
                .is_err()
        );
        assert_eq!(before, r.state());
        r.apply(T + 110_000, I::Recover(Empty {}));
        r.apply(
            T + 110_500,
            I::Sample(r.sample(2, 2, T + 110_000, T + 210_000)),
        );
        rejected(&r.apply(T + 111_000, I::Enter(c())));
        assert_eq!(
            r.state()["fixtures"][hex(&cid())]["signed_intents"]
                .as_object()
                .unwrap()
                .len(),
            1
        );
    }
}

#[test]
fn regression_committed_length_corruption_truncation_and_context_fail_closed() {
    for corruption in 0..7 {
        let mut r = Rig::new(1000);
        r.ready();
        r.apply(T + 1000, I::Close(c()));
        drop(r.j.take());
        let mut bytes = fs::read(&r.path).unwrap();
        let mut start = 0;
        let mut last = 0;
        while start < bytes.len() {
            last = start;
            let n = u64::from_be_bytes(bytes[start..start + 8].try_into().unwrap()) as usize;
            start += n + 48;
        }
        let n = u64::from_be_bytes(bytes[last..last + 8].try_into().unwrap());
        let head = PathBuf::from(format!("{}.head", r.path.display()));
        match corruption {
            0 => bytes[last..last + 8].copy_from_slice(&(n + 1).to_be_bytes()),
            1 => {
                bytes[last..last + 8].copy_from_slice(&(n + 1).to_be_bytes());
                bytes[last + 8..last + 16].copy_from_slice(&(!(n + 1)).to_be_bytes());
            }
            2 => {
                bytes.pop();
            }
            3 => bytes.truncate(last),
            4 => bytes[last + 16] ^= 1,
            5 => fs::remove_file(head).unwrap(),
            _ => r.id.config_hash[0] ^= 1,
        }
        fs::write(&r.path, bytes).unwrap();
        assert!(
            Journal::open(&r.path, r.id, false).is_err(),
            "corruption {corruption}"
        );
    }
}

#[test]
fn journal_rejects_rehashed_ack_state_mismatch() {
    let mut r = Rig::new(1000);
    r.apply(T, I::RequestSample(Empty {}));
    drop(r.j.take());
    let raw = fs::read(&r.path).unwrap();
    let n = u64::from_be_bytes(raw[..8].try_into().unwrap()) as usize;
    let mut record: StepCommit = serde_json::from_slice(&raw[16..16 + n]).unwrap();
    record.ack.state_hash[0] ^= 1;
    record.ack.logical_ack_hash = ack_hash(&record.ack);
    let payload = serde_json::to_vec(&record).unwrap();
    let digest = hash_parts("I1/journal/v1", &[&payload]);
    let size = payload.len() as u64;
    let mut frame = Vec::new();
    frame.extend(size.to_be_bytes());
    frame.extend((!size).to_be_bytes());
    frame.extend(payload);
    frame.extend(&digest);
    fs::write(&r.path, &frame).unwrap();
    let mut head = b"I1HEAD01".to_vec();
    head.extend((frame.len() as u64).to_be_bytes());
    head.extend(1u64.to_be_bytes());
    head.extend(&digest);
    let check = hash_parts(
        "I1/journal-head/v1",
        &[
            &r.id.run_id,
            &r.id.config_hash,
            &r.id.node_id.to_be_bytes(),
            &head,
        ],
    );
    head.extend(check);
    fs::write(format!("{}.head", r.path.display()), head).unwrap();
    let error = Journal::open(&r.path, r.id, false)
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("state hash mismatch"), "{error}");
}

#[test]
fn regression_first_local_receive_is_immutable_and_unsafe_is_not_retroactive() {
    for unsafe_first in [false, true] {
        let mut r = Rig::new(-1000);
        if !unsafe_first {
            r.ready();
        } else {
            r.apply(
                T,
                I::Create(Fixture {
                    cid: cid(),
                    stop_local_us: 1_300_000,
                }),
            );
        }
        let mut e = Envelope {
            run_id: r.id.run_id.clone(),
            src: 0,
            dst: 1,
            channel: 0,
            cid: cid(),
            sent_at_tick_us: T + 1000,
            payload_bytes: vec![7],
            ..Default::default()
        };
        e.payload_hash = sim_runtime::payload_hash(&e.payload_bytes);
        e.message_id = sim_runtime::message_id(&e);
        let a = r.apply(T + 20_000, I::Deliver(e.clone()));
        assert_eq!(
            has(&a, |x| matches!(x, E::MessageReceived(_))),
            !unsafe_first
        );
        let saved = r.state()["first_received"][hex(&e.message_id)].clone();
        assert_eq!(saved["first_tick_us"], T + 20_000);
        assert_eq!(
            saved["first_local_time_us"],
            if unsafe_first {
                Value::Null
            } else {
                json!(1_019_000)
            }
        );
        r.reopen();
        r.apply(T + 21_000, I::Recover(Empty {}));
        let req = if unsafe_first { 1 } else { 2 };
        r.apply(
            T + 21_500,
            I::Sample(r.sample(req, 2, T + 21_000, T + 121_000)),
        );
        e.attempt = 1;
        rejected(&r.apply(T + 22_000, I::Deliver(e.clone())));
        assert_eq!(saved, r.state()["first_received"][hex(&e.message_id)]);
    }
}

#[test]
fn payload_boundary_and_duplicate_identity() {
    let mut r = Rig::new(1000);
    r.ready();
    for len in [65_536usize, 65_537] {
        let mut e = Envelope {
            run_id: r.id.run_id.clone(),
            src: 0,
            dst: 1,
            channel: 0,
            cid: cid(),
            sent_at_tick_us: T + 1000,
            payload_bytes: vec![8; len],
            ..Default::default()
        };
        e.payload_hash = sim_runtime::payload_hash(&e.payload_bytes);
        e.message_id = sim_runtime::message_id(&e);
        let a = r.apply(T + 1000, I::Send(e));
        assert_eq!(has(&a, |x| matches!(x, E::Send(_))), len == 65_536);
    }
}

// Actual gRPC controls below are ignored by default, so the lead can run them
// with socket permission without blocking independent journal/checker work.
fn socket_driver(name: &str, mode: &str, seed: u64) {
    if std::env::var("I1_VERIFIER_CHILD").ok().as_deref() != Some(name) {
        let dir = temp(name);
        let executable = dir.join("acceptance");
        fs::copy(std::env::current_exe().unwrap(), &executable).unwrap();
        let script = format!(
            "#!/bin/sh\nexport I1_WORKER_ENDPOINT=\"$1\" I1_WORKER_LAUNCH=\"$2\"\nexec '{}' --exact helper_worker_entry --ignored --nocapture\n",
            executable.display()
        );
        fs::write(dir.join("replica-worker"), script).unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            dir.join("replica-worker"),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        let out = std::process::Command::new(&executable)
            .args([
                "--exact",
                name,
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("I1_VERIFIER_CHILD", name)
            .env("I1_WORKER_MODE", mode)
            .env("I1_VERIFIER_RUN", dir.join("run"))
            .output()
            .unwrap();
        println!("socket evidence: {}", dir.display());
        assert!(
            out.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        return;
    }
    let output = PathBuf::from(std::env::var("I1_VERIFIER_RUN").unwrap());
    fs::create_dir_all(&output).unwrap();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let mut s = scenario::Scenario::new(
            if mode == "normal" {
                "restart-before-ack"
            } else {
                "healthy"
            },
            seed,
        )
        .unwrap();
        if mode == "normal" {
            for t in [14_000, 23_000, 32_000, 41_000, 50_000] {
                for dt in [0, 1, 250] {
                    s.actions.push(scenario::Action {
                        tick_us: T + t + dt,
                        node: 0,
                        kind: scenario::ActionKind::Input(Input {
                            kind: Some(I::ReadClock(Cid {
                                cid: scenario::cid("first"),
                            })),
                        }),
                    });
                }
            }
        } else if mode == "bad_timer" {
            s.actions.push(scenario::Action {
                tick_us: T + 10000,
                node: 0,
                kind: scenario::ActionKind::Input(Input {
                    kind: Some(I::CancelTimer(Timer {
                        timer_id: 999,
                        generation: 1,
                        cid: scenario::cid("first"),
                        local_deadline_us: 1_060_000,
                    })),
                }),
            });
        }
        let mut h = hub::Hub::new(s, &output, 0).await.unwrap();
        let result = h.run(false).await;
        let err = result.as_ref().err().map(|e| e.to_string());
        let cleanup = h.finish(result.as_ref().err().map(|e| e.as_ref())).await;
        drop(h);
        let rows: Vec<Value> = fs::read_to_string(output.join("virtual-trace.jsonl"))
            .unwrap()
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
        if mode == "normal" {
            assert!(result.is_ok(), "{err:?}");
            assert!(cleanup.is_ok());
            assert!(rows.iter().any(|r| r["kind"] == "RecoveryInputDeferred"));
        } else {
            assert!(result.is_err(), "negative control unexpectedly passed");
            let e = err.unwrap();
            println!("expected failure {mode}: {e}");
            match mode {
                "bad_timer" => assert!(e.contains("timer effect")),
                "altered_duplicate" => assert!(e.contains("altered duplicate Ack")),
                "unsolicited" | "size_exact" => assert!(e.contains("unsolicited Ack")),
                "size_over" => {
                    let d = fs::read_to_string(output.join("transport-diagnostics.jsonl")).unwrap();
                    assert!(e.contains("message") || d.contains("message"));
                }
                "kill" => assert!(e.contains("unscripted")),
                "watchdog" => assert!(e.contains("WallWatchdog")),
                _ => {}
            }
            if mode == "bad_timer" {
                let bad = rows
                    .iter()
                    .find(|r| {
                        r["kind"] == "Grant"
                            && r["detail"]["input"]["kind"].get("CancelTimer").is_some()
                    })
                    .unwrap();
                let step = &bad["detail"]["logical_step_id"];
                assert!(!rows.iter().any(|r| r["kind"] == "AckCommitted"
                    && r["detail"]["ack"]["logical_step_id"] == *step));
            }
            assert!(!rows.iter().any(|r| r["kind"] == "RunComplete"));
        }
        for node in 0..2 {
            let p = output.join(format!("worker-{node}/process-history.json"));
            if p.exists() {
                let v: Vec<Value> = serde_json::from_slice(&fs::read(p).unwrap()).unwrap();
                assert_eq!(
                    v.iter().filter(|v| v["kind"] == "ProcessStarted").count(),
                    v.iter().filter(|v| v["kind"] == "ProcessExited").count()
                );
            }
        }
    });
}

#[test]
#[ignore]
fn socket_recovery_equal_tick_seed7() {
    socket_driver("socket_recovery_equal_tick_seed7", "normal", 7)
}
#[test]
#[ignore]
fn socket_recovery_equal_tick_seed42() {
    socket_driver("socket_recovery_equal_tick_seed42", "normal", 42)
}
#[test]
#[ignore]
fn socket_recovery_equal_tick_seed99() {
    socket_driver("socket_recovery_equal_tick_seed99", "normal", 99)
}
#[test]
#[ignore]
fn socket_invalid_ack_is_atomic() {
    socket_driver("socket_invalid_ack_is_atomic", "bad_timer", 7)
}
#[test]
#[ignore]
fn socket_unsolicited_ack() {
    socket_driver("socket_unsolicited_ack", "unsolicited", 7)
}
#[test]
#[ignore]
fn socket_altered_duplicate_ack() {
    socket_driver("socket_altered_duplicate_ack", "altered_duplicate", 7)
}
#[test]
#[ignore]
fn socket_grpc_exact_1mib() {
    socket_driver("socket_grpc_exact_1mib", "size_exact", 7)
}
#[test]
#[ignore]
fn socket_grpc_over_1mib() {
    socket_driver("socket_grpc_over_1mib", "size_over", 7)
}
#[test]
#[ignore]
fn socket_unscripted_kill() {
    socket_driver("socket_unscripted_kill", "kill", 7)
}
#[test]
#[ignore]
fn socket_wall_watchdog() {
    socket_driver("socket_wall_watchdog", "watchdog", 7)
}

#[test]
#[ignore]
fn helper_worker_entry() {
    let Ok(endpoint) = std::env::var("I1_WORKER_ENDPOINT") else {
        return;
    };
    let launch: Launch =
        serde_json::from_slice(&fs::read(std::env::var("I1_WORKER_LAUNCH").unwrap()).unwrap())
            .unwrap();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            use poc_types::{
                hub_frame::Frame as H, runtime_client::RuntimeClient, worker_frame::Frame as W,
            };
            use tokio_stream::wrappers::ReceiverStream;
            let mut j = Journal::open(
                &launch.journal,
                launch.identity.clone(),
                launch.incarnation == 1,
            )
            .unwrap();
            let id = &launch.identity;
            let mut client = RuntimeClient::connect(endpoint)
                .await
                .unwrap()
                .max_encoding_message_size(MAX_FRAME_BYTES + 4096)
                .max_decoding_message_size(MAX_FRAME_BYTES);
            let reg = RegisterRequest {
                schema_version: 1,
                run_id: id.run_id.clone(),
                node_id: id.node_id,
                incarnation: launch.incarnation,
                config_hash: id.config_hash.clone(),
                recovered_journal_seq: j.sequence(),
                recovered_state_hash: j.state().hash().unwrap(),
            };
            client.register(reg.clone()).await.unwrap();
            let (tx, rx) = tokio::sync::mpsc::channel(4);
            tx.send(WorkerFrame {
                frame: Some(W::Hello(reg)),
            })
            .await
            .unwrap();
            let mut stream = client
                .controlled_run(ReceiverStream::new(rx))
                .await
                .unwrap()
                .into_inner();
            let mode = std::env::var("I1_WORKER_MODE").unwrap();
            if id.node_id == 0
                && ["unsolicited", "size_exact", "size_over"].contains(&mode.as_str())
            {
                let mut a = Ack {
                    schema_version: 1,
                    run_id: id.run_id.clone(),
                    node_id: 0,
                    incarnation: launch.incarnation,
                    logical_step_id: 999,
                    event_id: 999,
                    grant_hash: vec![7; 32],
                    status: 1,
                    effects: vec![Effect {
                        kind: Some(E::Rejected(Notice {
                            cid: vec![],
                            reason: String::new(),
                            request_id: 0,
                        })),
                    }],
                    durable_journal_seq: 1,
                    state_hash: vec![8; 32],
                    logical_ack_hash: vec![0; 32],
                };
                if mode != "unsolicited" {
                    let target = MAX_FRAME_BYTES + usize::from(mode == "size_over");
                    let mut count = target - 200;
                    loop {
                        if let Some(E::Rejected(n)) = a.effects[0].kind.as_mut() {
                            n.reason = "x".repeat(count);
                        }
                        let len = WorkerFrame {
                            frame: Some(W::Ack(a.clone())),
                        }
                        .encoded_len();
                        if len == target {
                            break;
                        }
                        count = (count as isize + target as isize - len as isize) as usize;
                    }
                }
                a.logical_ack_hash = ack_hash(&a);
                tx.send(WorkerFrame {
                    frame: Some(W::Ack(a)),
                })
                .await
                .unwrap();
            }
            let mut retry = launch.retry;
            while let Some(frame) = stream.message().await.unwrap_or(None) {
                match frame.frame.unwrap() {
                    H::Shutdown(s) => {
                        if s.reason == "scripted-after-hub-commit" {
                            std::process::exit(74)
                        }
                        return;
                    }
                    H::Grant(g) => {
                        if id.node_id == 0 && mode == "kill" {
                            let _ = std::process::Command::new("/bin/kill")
                                .args(["-KILL", &std::process::id().to_string()])
                                .status();
                            panic!("kill failed");
                        }
                        if id.node_id == 0 && mode == "watchdog" {
                            std::future::pending::<()>().await;
                        }
                        let fault = if let Some(I::WorkCompleted(w)) =
                            g.input.as_ref().and_then(|x| x.kind.as_ref())
                        {
                            launch
                                .faults
                                .iter()
                                .find(|(slot, _)| *slot == w.slot)
                                .map(|(_, c)| *c)
                        } else {
                            None
                        };
                        match j
                            .process(&g, launch.incarnation, retry.take().as_ref(), fault)
                            .unwrap()
                        {
                            StepResult::Crash(c) => std::process::exit(c.exit_code()),
                            StepResult::Ack(mut a) => {
                                if id.node_id == 0 && mode == "bad_timer" {
                                    if let Some(I::CancelTimer(timer)) =
                                        g.input.as_ref().and_then(|x| x.kind.as_ref())
                                    {
                                        a.status = 0;
                                        a.effects = vec![Effect {
                                            kind: Some(E::CancelTimer(timer.clone())),
                                        }];
                                        a.logical_ack_hash = ack_hash(&a);
                                    }
                                }
                                let mut changed = a.clone();
                                changed.state_hash[0] ^= 1;
                                changed.logical_ack_hash = ack_hash(&changed);
                                if tx
                                    .send(WorkerFrame {
                                        frame: Some(W::Ack(a)),
                                    })
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                                if id.node_id == 0 && mode == "altered_duplicate" {
                                    let _ = tx
                                        .send(WorkerFrame {
                                            frame: Some(W::Ack(changed)),
                                        })
                                        .await;
                                }
                            }
                        }
                    }
                }
            }
        });
}
