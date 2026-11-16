//! PE16 fixture. Wall pauses live in this driver, outside component inputs.
#[path = "../tests/support/mod.rs"]
mod support;

use std::{fs, path::PathBuf, thread, time::Duration};

use poc_publisher::*;
use sim_espresso::*;
use support::*;

fn phase(phase: Phase) -> &'static str {
    match phase {
        Phase::Verifying => "VERIFYING",
        Phase::Queued => "QUEUED",
        Phase::Submitted => "SUBMITTED",
        Phase::Recorded => "RECORDED",
        Phase::Stopped => "STOPPED",
    }
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}

fn log_task(event: &str, tick: u64, publisher: &FixturePublisher) {
    let task = publisher.task(&KEY).unwrap();
    let outcome = phase(task.phase());
    let generation = publisher.generation();
    let attempts = task.attempts.len();
    println!(
        "{{\"event\":\"{event}\",\"tick\":{tick},\"phase\":\"{outcome}\",\"generation\":{generation},\"attempts\":{attempts}}}"
    );
}

fn log_reader(event: &str, tick: u64, reader: &ReaderState) {
    let tip = reader
        .tip()
        .map_or("null".to_owned(), |(q, _)| q.to_string());
    let receipt_ticks = reader.tip().map_or_else(Vec::new, |(q, _)| {
        (0..=q)
            .map(|q| reader.receipt(q).unwrap().reader_received_at_us.0)
            .collect::<Vec<_>>()
    });
    println!(
        "{{\"event\":\"{event}\",\"tick\":{tick},\"tip\":{tip},\"receipt_ticks\":{receipt_ticks:?}}}"
    );
}

fn main() {
    let mut args = std::env::args_os().skip(1);
    let seed = args
        .next()
        .expect("seed")
        .to_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let pause = args
        .next()
        .expect("wall_pause_ms")
        .to_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let dir = PathBuf::from(args.next().expect("new artifact directory"));
    assert!(
        args.next().is_none(),
        "usage: scenario_trace SEED WALL_PAUSE_MS NEW_ARTIFACT_DIR"
    );
    assert!(
        matches!(seed, 7 | 42 | 99),
        "supported matrix seeds are 7, 42 and 99"
    );
    fs::create_dir(&dir).expect("new artifact directory");

    let pause_wall = || {
        if pause > 0 {
            thread::sleep(Duration::from_millis(pause));
        }
    };
    eprintln!("fixture=non-cryptographic seed={seed} wall_pause_ms={pause}");
    println!("{{\"event\":\"scenario\",\"seed\":{seed},\"fixture\":\"non-cryptographic\"}}");
    let state = dir.join("publisher");
    let mut publisher = create(&state);
    let inputs = FixtureInputs::default();

    let result = FixtureResult::valid(KEY);
    let wait_inputs = FixtureInputs {
        ancestor: false,
        ..Default::default()
    };
    assert_eq!(
        publisher.offer(result.clone(), &wait_inputs).unwrap().phase,
        Phase::Verifying
    );
    log_task("offer_wait", 100, &publisher);
    pause_wall();
    drop(publisher);
    publisher = reopen(&state);
    assert_eq!(
        publisher.offer(result.clone(), &inputs).unwrap().phase,
        Phase::Queued
    );
    log_task("offer_queued", 101, &publisher);

    let first = publisher.prepare_attempt(&KEY, TickUs(102)).unwrap();
    log_task("attempt_prepared", 102, &publisher);
    let first_bytes = fs::read(state.join("state.snapshot")).unwrap();
    fs::write(dir.join("prepared.snapshot"), &first_bytes).unwrap();
    pause_wall();
    drop(publisher);
    publisher = reopen(&state);

    let replay = publisher.prepare_attempt(&KEY, TickUs(103)).unwrap();
    assert_eq!(replay, first);
    let namespace = hex(&first.stable_id.namespace);
    let task_number = first.stable_id.task_number;
    let original_tick = replay.created_tick.0;
    println!(
        "{{\"event\":\"prepared_replay\",\"tick\":103,\"task_number\":{task_number},\"attempt\":1,\"namespace_hex\":\"{namespace}\",\"original_tick\":{original_tick}}}"
    );
    publisher
        .mark_dispatched(&replay.stable_id, b"fixture-controlled-1".to_vec())
        .unwrap();
    log_task("dispatch_committed", 104, &publisher);
    pause_wall();

    // The first ACK was lost. A later explicit retry retains the same task/payload.
    let retry = publisher.prepare_attempt(&KEY, TickUs(106)).unwrap();
    assert_eq!(retry.stable_id.attempt_number, 2);
    assert_eq!(retry.exact_payload, first.exact_payload);
    println!(
        "{{\"event\":\"attempt_retry\",\"tick\":106,\"task_number\":{task_number},\"attempt\":2,\"namespace_hex\":\"{namespace}\"}}"
    );
    publisher
        .mark_dispatched(&retry.stable_id, b"fixture-controlled-2".to_vec())
        .unwrap();
    publisher
        .mark_service_acknowledged(
            &retry.stable_id,
            b"fixture-controlled-2",
            &retry.exact_payload,
            b"fixture-service-ack".to_vec(),
        )
        .unwrap();
    log_task("service_acknowledged", 108, &publisher);

    let mut log = log();
    let rows = [
        produce(&mut log, 110, 104, vec![proof(KEY, 1)]),
        produce(&mut log, 112, 104, vec![body(KEY)]),
        produce(&mut log, 114, 104, vec![ancestor(KEY)]),
        produce(&mut log, 116, 105, vec![]),
    ];
    for row in &rows {
        let wire = encode_finalized(SOURCE, row).unwrap();
        let q = row.proposal.q;
        let theta = row.proposal.theta_us.0;
        let proposed_at = row.proposal.proposed_at_us.0;
        let finalized_at = row.finalized_at_us.0;
        fs::write(dir.join(format!("q{q}.pelg")), &wire).unwrap();
        println!(
            "{{\"event\":\"finalized\",\"q\":{q},\"theta_us\":{theta},\"proposed_at_us\":{proposed_at},\"finalized_at_us\":{finalized_at},\"wire_hex\":\"{}\"}}",
            hex(&wire),
        );
    }
    pause_wall();
    drop(publisher);
    publisher = reopen(&state);
    assert_eq!(publisher.task(&KEY).unwrap().phase(), Phase::Submitted);
    assert!(publisher.task(&KEY).unwrap().recorded_at.is_none());
    log_task("finalized_unreceived_reopen", 120, &publisher);

    let mut a = reader(b"fixture-observer-A");
    let mut b = reader(READER);
    for (index, row) in rows.iter().enumerate() {
        let tick = 130 + index as u64;
        a.receive_block(SOURCE, row.clone(), TickUs(tick)).unwrap();
        log_reader("reader_a", tick, &a);
        pause_wall();
    }
    assert!(b.tip().is_none());

    let order = match seed {
        7 => [2, 0, 1],
        42 => [2, 1, 0],
        99 => [0, 2, 1],
        _ => unreachable!(),
    };
    for (index, q) in order.into_iter().enumerate() {
        let tick = 140 + index as u64;
        b.receive_block(SOURCE, rows[q].clone(), TickUs(tick))
            .unwrap();
        println!("{{\"event\":\"reader_b_delivered\",\"tick\":{tick},\"q\":{q}}}");
        log_reader("reader_b", tick, &b);
        if index < 2
            && let Some((tip, _)) = b.tip()
        {
            let outcome = publisher
                .confirm_recorded(&KEY, &b.require_prefix_through(tip).unwrap(), &inputs)
                .unwrap();
            assert!(matches!(outcome, ConfirmOutcome::Wait { .. }));
            log_task("public_material_wait", tick, &publisher);
        }
        if index == 0 {
            let bytes = b.encode_snapshot();
            fs::write(dir.join("reader-b-pending.snapshot"), &bytes).unwrap();
            b = ReaderState::decode_snapshot(
                SOURCE,
                READER,
                &fs::read(dir.join("reader-b-pending.snapshot")).unwrap(),
            )
            .unwrap();
        }
        pause_wall();
    }

    assert_eq!(b.tip(), Some((2, PublicTimeUs(104))));
    assert!(b.require_watermark(PublicTimeUs(105)).is_err());
    let prefix = b.require_prefix_through(2).unwrap();
    assert_eq!(
        prefix.blocks(),
        a.require_prefix_through(2).unwrap().blocks()
    );
    let receipt = match publisher.confirm_recorded(&KEY, &prefix, &inputs).unwrap() {
        ConfirmOutcome::Recorded { receipt, .. } => receipt,
        other => panic!("unexpected receipt {other:?}"),
    };
    assert_eq!(
        (receipt.publication_position.q, receipt.complete_through_q),
        (0, 2)
    );
    println!(
        "{{\"event\":\"recorded_receipt\",\"tick\":144,\"q\":{},\"message_index\":{},\"complete_through_q\":{}}}",
        receipt.publication_position.q,
        receipt.publication_position.message_index,
        receipt.complete_through_q
    );
    log_task("recorded", 144, &publisher);

    b.receive_block(SOURCE, rows[3].clone(), TickUs(145))
        .unwrap();
    log_reader("reader_b", 145, &b);
    assert_eq!(
        publisher
            .apply_watermark(&KEY, &b.require_watermark(PublicTimeUs(105)).unwrap())
            .unwrap(),
        WatermarkOutcome::Applied
    );
    drop(publisher);
    publisher = reopen(&state);
    let saved = publisher.task(&KEY).unwrap();
    assert_eq!(saved.recorded_at, Some(receipt));
    assert!(matches!(
        saved.stop_reason,
        Some(PublicStop::Deadline { q: 3, .. })
    ));
    assert!(matches!(
        publisher.prepare_attempt(&KEY, TickUs(146)),
        Err(PublisherError::SubmissionUnavailable(Phase::Recorded))
    ));
    log_task("recorded_and_stopped_reopen", 146, &publisher);
    fs::write(dir.join("reader-a.snapshot"), a.encode_snapshot()).unwrap();
    fs::write(dir.join("reader-b.snapshot"), b.encode_snapshot()).unwrap();
}
