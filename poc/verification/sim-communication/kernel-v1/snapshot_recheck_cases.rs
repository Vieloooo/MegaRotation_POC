//! Independent necessary timing bounds, including valid u64 edge controls.
use super::*;

#[test]
fn next_release_mutants_reject_with_nonempty_and_empty_queues() {
    for drain in [false, true] {
        let c = plain_config();
        let mut k = RouteKernel::new(c.clone()).unwrap();
        route(&mut k, 1, 1000, &[RouteCommand::Send(packet(1, 1000))]);
        let (old, bound) = if drain {
            route(&mut k, 2, 1020, &[RouteCommand::Drain]);
            (1023u64, 1020u64 + 65536)
        } else {
            (0, 1000u64 + 65536)
        };
        let snapshot = k.snapshot_bytes().unwrap();
        assert_eq!(
            RouteKernel::from_snapshot(c.clone(), &snapshot)
                .unwrap()
                .snapshot_bytes()
                .unwrap(),
            snapshot
        );
        for value in [bound + 1, u64::MAX] {
            // Match only the used link; all unused links legitimately keep zero.
            let changed = replace_once(
                &snapshot,
                &format!("\"send_ordinal\":1,\"next_release_us\":{old}"),
                &format!("\"send_ordinal\":1,\"next_release_us\":{value}"),
            );
            save_evidence(
                &format!("pacing-drained{drain}-mutant-{value}.json"),
                &changed,
            );
            error(
                RouteKernel::from_snapshot(c.clone(), &changed),
                Error::CorruptSnapshot,
            );
        }
        save_evidence(&format!("pacing-drained{drain}-valid.json"), &snapshot);
    }
}

#[test]
fn valid_near_u64_pacing_and_raw_due_roundtrip_without_bound_overflow() {
    // Actual pacing is one microsecond for an empty frame; theoretical maximum
    // pacing can extend past u64::MAX without making this saved state invalid.
    let mut c = plain_config();
    c.rules[0].profile.latency_us = 0;
    let at = u64::MAX - 2;
    let mut k = RouteKernel::new(c.clone()).unwrap();
    let packets: Vec<_> = (0..2)
        .map(|n| {
            RouteCommand::Send(Envelope {
                message: changed_message(msg(), Endpoint::Replica(1), n, at, vec![]),
                ..envelope()
            })
        })
        .collect();
    route(&mut k, 1, at, &packets);
    assert_eq!(
        delivered(&route(&mut k, 2, at, &[RouteCommand::Drain])).len(),
        1
    );
    let queued = k.snapshot_bytes().unwrap();
    let mut restored = RouteKernel::from_snapshot(c.clone(), &queued).unwrap();
    assert_eq!(restored.next_wakeup_us(), Some(u64::MAX - 1));
    assert_eq!(
        delivered(&route(
            &mut restored,
            3,
            u64::MAX - 1,
            &[RouteCommand::Drain]
        ))
        .len(),
        1
    );
    let empty = restored.snapshot_bytes().unwrap();
    assert_eq!(
        RouteKernel::from_snapshot(c, &empty)
            .unwrap()
            .next_wakeup_us(),
        None
    );
    save_evidence("near-u64-pacing-queued-valid.json", &queued);
    save_evidence("near-u64-pacing-empty-valid.json", &empty);

    // Raw due values stay representable although last_tick+configured delay
    // bounds exceed u64 after a later no-op. Future fault entries are irrelevant.
    let mut c = plain_config();
    c.rules[0].profile.faults = vec![
        FaultEntry {
            send_ordinal: 1,
            drop: false,
            extra_delay_us: 5,
            duplicate_delay_us: Some(17),
        },
        FaultEntry {
            send_ordinal: 9,
            drop: false,
            extra_delay_us: u64::MAX,
            duplicate_delay_us: Some(u64::MAX),
        },
    ];
    let at = u64::MAX - 42;
    let mut k = RouteKernel::new(c.clone()).unwrap();
    let e = Envelope {
        message: changed_message(msg(), Endpoint::Replica(1), 0, at, vec![]),
        ..envelope()
    };
    route(&mut k, 1, at, &[RouteCommand::Send(e)]);
    let before = k.snapshot_bytes().unwrap();
    RouteKernel::from_snapshot(c.clone(), &before).unwrap();
    assert!(
        std::str::from_utf8(&before)
            .unwrap()
            .contains(&format!("\"raw_due_us\":{}", u64::MAX - 17))
    );
    assert!(
        std::str::from_utf8(&before)
            .unwrap()
            .contains(&format!("\"raw_due_us\":{}", u64::MAX))
    );
    route(&mut k, 2, u64::MAX - 10, &[]);
    let advanced = k.snapshot_bytes().unwrap();
    let mut restored = RouteKernel::from_snapshot(c.clone(), &advanced).unwrap();
    assert_eq!(restored.next_wakeup_us(), Some(u64::MAX - 10));
    assert_eq!(
        delivered(&route(
            &mut restored,
            3,
            u64::MAX - 10,
            &[RouteCommand::Drain]
        ))
        .len(),
        1
    );
    assert_eq!(restored.next_wakeup_us(), Some(u64::MAX));
    let state = restored.snapshot_bytes().unwrap();
    RouteKernel::from_snapshot(c, &state).unwrap();
    error(
        restored.plan(step(4, u64::MAX), &[RouteCommand::Drain]),
        Error::ArithmeticOverflow,
    );
    assert_eq!(state, restored.snapshot_bytes().unwrap());
    save_evidence("near-u64-raw-due-valid.json", &advanced);
}

#[test]
fn queued_fault_bounds_use_past_primary_duplicate_and_unlisted_ordinals() {
    let mut c = plain_config();
    c.rules[0].profile.faults = vec![
        FaultEntry {
            send_ordinal: 1,
            drop: false,
            extra_delay_us: 30,
            duplicate_delay_us: Some(5),
        },
        FaultEntry {
            send_ordinal: 2,
            drop: true,
            extra_delay_us: u64::MAX,
            duplicate_delay_us: None,
        },
        FaultEntry {
            send_ordinal: 7,
            drop: false,
            extra_delay_us: 500000,
            duplicate_delay_us: Some(77),
        },
    ];
    let mut k = RouteKernel::new(c.clone()).unwrap();
    route(&mut k, 1, 1000, &[RouteCommand::Send(packet(1, 1000))]);
    let drop = route(&mut k, 2, 1001, &[RouteCommand::Send(packet(2, 1001))]);
    assert!(matches!(
        drop[0].kind,
        RouteEventKind::Dropped {
            reason: DropReason::Fault,
            ..
        }
    ));
    let snapshot = k.snapshot_bytes().unwrap();
    assert_eq!(
        RouteKernel::from_snapshot(c.clone(), &snapshot)
            .unwrap()
            .snapshot_bytes()
            .unwrap(),
        snapshot
    );
    // Before ordinal3, only past non-drop ordinal1 can explain queued copies.
    for changed in [
        replace_once(&snapshot, "\"raw_due_us\":1050", "\"raw_due_us\":1052"),
        replace_once(&snapshot, "\"raw_due_us\":1055", "\"raw_due_us\":1057"),
        replace_once(&snapshot, "\"raw_due_us\":1050", "\"raw_due_us\":501020"),
    ] {
        error(
            RouteKernel::from_snapshot(c.clone(), &changed),
            Error::CorruptSnapshot,
        );
    }
    route(&mut k, 3, 1100, &[]);
    RouteKernel::from_snapshot(c.clone(), &k.snapshot_bytes().unwrap()).unwrap();
    route(&mut k, 4, 1101, &[RouteCommand::Send(packet(3, 1000))]);
    let mixed = k.snapshot_bytes().unwrap();
    assert_eq!(
        RouteKernel::from_snapshot(c, &mixed)
            .unwrap()
            .snapshot_bytes()
            .unwrap(),
        mixed
    );
    save_evidence("past-fault-original-and-duplicate-valid.json", &snapshot);
    save_evidence("past-fault-plus-unlisted-ordinal-valid.json", &mixed);
}

#[test]
fn tombstone_attempt_count_bounds_keep_old_due_under_near_u64_watermark() {
    let mut sender = mailbox(Endpoint::Replica(0));
    let mut receiver = mailbox(Endpoint::Replica(1));
    confirm(&mut sender, action("enqueue"));
    confirm(&mut sender, action("attempt"));
    let next = confirm(
        &mut sender,
        MailboxAction::Attempt {
            logical_id: msg().logical_id,
            now_us: 1_020_000,
            destination: EndpointBinding {
                endpoint: Endpoint::Replica(1),
                incarnation: 1,
            },
        },
    );
    let sent = match &next.releases[0] {
        CustodyRelease::Send(e) => e.clone(),
        _ => panic!(),
    };
    let received = confirm(
        &mut receiver,
        MailboxAction::Receive {
            envelope: sent,
            received_tick_us: 1_020_000,
            received_local_us: None,
            authenticated_source: EndpointBinding {
                endpoint: Endpoint::Replica(0),
                incarnation: 1,
            },
        },
    );
    let ack = match &received.releases[1] {
        CustodyRelease::ReceiveAck(a) => a.clone(),
        _ => panic!(),
    };
    confirm(
        &mut sender,
        MailboxAction::Enqueue {
            message: changed_message(msg(), Endpoint::Replica(1), 88, 1_000_000, vec![]),
            now_us: u64::MAX - 1,
        },
    );
    let p = sender
        .plan(MailboxAction::Acknowledge {
            ack,
            authenticated_receiver: EndpointBinding {
                endpoint: Endpoint::Replica(1),
                incarnation: 1,
            },
        })
        .unwrap();
    let r = receipt(&p, 0x55);
    let original = p.persistence_bytes().to_vec();
    let (restored, _) = ReliableMailbox::restore(config(), &original, r.clone()).unwrap();
    assert_eq!(restored.retry_due_us(msg().logical_id), None);
    let changed = replace_once(
        &original,
        "\"retry_due_us\":1040000",
        "\"retry_due_us\":1020000",
    );
    let mutant = DurabilityReceipt {
        candidate_record_hash: framed_hash("SIMT/custody-record/v1", &[&changed]),
        ..r
    };
    error(
        ReliableMailbox::restore(config(), &changed, mutant),
        Error::CorruptSnapshot,
    );
    save_evidence("tombstone-new-watermark-valid.json", &original);
    save_evidence("tombstone-two-attempt-one-interval-mutant.json", &changed);
}
