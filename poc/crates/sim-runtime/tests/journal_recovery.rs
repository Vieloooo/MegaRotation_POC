use poc_types::{effect::Kind as E, input::Kind as I, *};
use sim_runtime::{
    Checkpoint, Identity, Retry,
    journal::{Journal, StepResult},
    rate_profile,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);
impl TestDirectory {
    fn new() -> Self {
        let id = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("i1-journal-{}-{id}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn id() -> Identity {
    Identity {
        run_id: vec![11; 32],
        config_hash: vec![17; 32],
        node_id: 0,
        epsilon_us: 1000,
    }
}

fn grant(step: u64, inc: u64, tick: u64, kind: I) -> Grant {
    let mut grant = Grant {
        schema_version: SCHEMA_VERSION,
        run_id: id().run_id,
        node_id: 0,
        incarnation: inc,
        logical_step_id: step,
        event_id: step,
        tick_us: tick,
        causal_wave: 0,
        tie_rank: vec![23; 32],
        input: Some(Input { kind: Some(kind) }),
        grant_hash: vec![],
    };
    grant.grant_hash = grant_hash(&grant);
    grant
}

fn ack(journal: &mut Journal, grant: &Grant) -> Ack {
    match journal
        .process(grant, grant.incarnation, None, None)
        .unwrap()
    {
        StepResult::Ack(ack) => ack,
        StepResult::Crash(_) => panic!("unexpected scripted crash"),
    }
}

fn prepare_work(journal: &mut Journal) -> Work {
    let cid = vec![29; 32];
    let work = Work {
        cid: cid.clone(),
        slot: 1,
        subject: vec![31; 32],
        duration_us: 10,
    };
    let sample = good_sample(1, 1, 10);
    for (step, tick, input) in [
        (
            1,
            8,
            I::Create(Fixture {
                cid: cid.clone(),
                stop_local_us: 1200,
            }),
        ),
        (2, 9, I::RequestSample(Empty {})),
        (3, 11, I::Sample(sample)),
        (4, 12, I::AnchorReady(Cid { cid: cid.clone() })),
        (5, 13, I::Enter(Cid { cid })),
        (6, 14, I::BeginWork(work.clone())),
    ] {
        assert_eq!(
            ack(journal, &grant(step, 1, tick, input)).status,
            AckStatus::Quiescent as i32
        );
    }
    work
}

fn good_sample(incarnation: u64, sequence: u64, at: u64) -> ClockSample {
    ClockSample {
        run_id: id().run_id,
        node_id: 0,
        incarnation,
        request_id: sequence,
        clock_domain: id().clock_domain(),
        sequence,
        as_of_tick_us: at,
        node_time_at_as_of_us: 1000 + at as i64,
        uncertainty_us: 1000,
        lease_end_tick_us: at + 100,
        rate_profile_ref: rate_profile(),
    }
}

#[test]
fn committed_truncation_and_length_corruption_fail_closed() {
    let dir = TestDirectory::new();
    let path = dir.0.join("journal");
    let mut journal = Journal::open(&path, id(), true).unwrap();
    let work = prepare_work(&mut journal);
    let prior_len = fs::metadata(&path).unwrap().len() as usize;
    ack(&mut journal, &grant(7, 1, 24, I::WorkCompleted(work)));
    drop(journal);
    let intact = fs::read(&path).unwrap();
    let original = u64::from_be_bytes(intact[prior_len..prior_len + 8].try_into().unwrap());

    let mut wrong_length = intact.clone();
    wrong_length[prior_len..prior_len + 8].copy_from_slice(&(original + 1).to_be_bytes());
    fs::write(&path, &wrong_length).unwrap();
    assert!(Journal::open(&path, id(), false).is_err());
    assert_eq!(fs::read(&path).unwrap(), wrong_length);

    fs::write(&path, &intact[..intact.len() - 1]).unwrap();
    assert!(Journal::open(&path, id(), false).is_err());
    assert_eq!(
        fs::metadata(&path).unwrap().len() as usize,
        intact.len() - 1
    );

    let mut wrong_payload = intact;
    wrong_payload[prior_len + 16] ^= 1;
    fs::write(&path, &wrong_payload).unwrap();
    assert!(Journal::open(&path, id(), false).is_err());
}

#[test]
fn uncommitted_signing_step_can_only_complete_after_fresh_sample() {
    for fault in [Checkpoint::BeforeAppend, Checkpoint::PartialTail] {
        let dir = TestDirectory::new();
        let path = dir.0.join("journal");
        let mut journal = Journal::open(&path, id(), true).unwrap();
        let work = prepare_work(&mut journal);
        let completed = grant(7, 1, 24, I::WorkCompleted(work.clone()));
        assert!(
            matches!(journal.process(&completed, 1, None, Some(fault)).unwrap(), StepResult::Crash(actual) if actual == fault)
        );
        drop(journal);

        let mut recovered = Journal::open(&path, id(), false).unwrap();
        assert_eq!(recovered.sequence(), 6);
        let mut copied = completed.clone();
        copied.incarnation = 2;
        let retry = Retry {
            logical_step_id: 7,
            grant_hash: copied.grant_hash.clone(),
            original_incarnation: 1,
        };
        assert!(recovered.process(&copied, 2, Some(&retry), None).is_err());
        assert!(
            recovered
                .process(&grant(8, 2, 24, I::RequestSample(Empty {})), 2, None, None)
                .is_err()
        );
        assert!(
            recovered
                .process(
                    &grant(8, 2, 24, I::WorkCompleted(work.clone())),
                    2,
                    None,
                    None
                )
                .is_err()
        );

        let recover = ack(&mut recovered, &grant(8, 2, 24, I::Recover(Empty {})));
        assert!(
            matches!(&recover.effects[0].kind, Some(E::RequestSample(request)) if request.request_id == 2)
        );
        ack(
            &mut recovered,
            &grant(9, 2, 27, I::Sample(good_sample(2, 2, 26))),
        );
        let intent = ack(&mut recovered, &grant(10, 2, 27, I::WorkCompleted(work)));
        assert!(
            intent
                .effects
                .iter()
                .any(|e| matches!(e.kind, Some(E::SignedIntent(_))))
        );
    }
}

#[test]
fn synced_record_recovers_exact_ack_with_or_without_updated_head() {
    for fault in [
        Checkpoint::AfterRecordSyncBeforeHead,
        Checkpoint::AfterSyncBeforeAck,
    ] {
        let dir = TestDirectory::new();
        let path = dir.0.join("journal");
        let mut journal = Journal::open(&path, id(), true).unwrap();
        let work = prepare_work(&mut journal);
        let completed = grant(7, 1, 24, I::WorkCompleted(work.clone()));
        assert!(
            matches!(journal.process(&completed, 1, None, Some(fault)).unwrap(), StepResult::Crash(actual) if actual == fault)
        );
        drop(journal);

        let mut recovered = Journal::open(&path, id(), false).unwrap();
        assert_eq!(recovered.sequence(), 7);
        let mut copied = completed.clone();
        copied.incarnation = 2;
        let retry = Retry {
            logical_step_id: 7,
            grant_hash: copied.grant_hash.clone(),
            original_incarnation: 1,
        };
        assert!(recovered.process(&copied, 2, None, None).is_err());
        let StepResult::Ack(replay) = recovered.process(&copied, 2, Some(&retry), None).unwrap()
        else {
            panic!("expected cached Ack")
        };
        assert_eq!(replay.durable_journal_seq, 7);
        assert_eq!(replay.logical_ack_hash, ack_hash(&replay));
        assert!(
            replay
                .effects
                .iter()
                .any(|e| matches!(e.kind, Some(E::SignedIntent(_))))
        );
        assert_eq!(recovered.sequence(), 7);
        let altered = grant(7, 2, 25, I::WorkCompleted(work));
        assert!(recovered.process(&altered, 2, Some(&retry), None).is_err());
    }
}
