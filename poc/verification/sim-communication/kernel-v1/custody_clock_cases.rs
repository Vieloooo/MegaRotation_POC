use super::*;

fn assert_record(plan: &CustodyPlan, name: &str) {
    let g = golden();
    let row = &g["custody_records"][name];
    assert_eq!(
        std::str::from_utf8(plan.persistence_bytes()).unwrap(),
        row["record_canonical_utf8"]
    );
    assert_eq!(hex(plan.transition_id()), row["transition_id_hex"]);
    assert_eq!(hex(plan.candidate_record_hash()), row["record_hash_hex"]);
}

#[test]
fn kv26_four_complete_golden_records_and_receive_original_watermark() {
    let mut sender = mailbox(Endpoint::Replica(0));
    let mut receiver = mailbox(Endpoint::Replica(1));
    for name in ["enqueue", "attempt"] {
        let p = sender.plan(action(name)).unwrap();
        assert_record(&p, name);
        let r = receipt(&p, 0x55);
        sender.confirm(p, r).unwrap();
    }
    let p = receiver.plan(action("receive")).unwrap();
    assert_record(&p, "receive");
    assert!(
        std::str::from_utf8(p.persistence_bytes())
            .unwrap()
            .contains("\"last_action_tick_us\":0")
    );
    let r = receipt(&p, 0x66);
    let committed = receiver.confirm(p, r).unwrap();
    assert_eq!(committed.releases.len(), 2);
    let ack = match &committed.releases[1] {
        CustodyRelease::ReceiveAck(a) => a.clone(),
        other => panic!("expected receive ACK, got {other:?}"),
    };
    assert_eq!(ack, parse::<ReceiveAck>(&golden()["receive_ack"]));
    let p = sender.plan(action("delete")).unwrap();
    assert_record(&p, "delete");
    let r = receipt(&p, 0x55);
    sender.confirm(p, r).unwrap();
    assert_eq!(sender.retry_due_us(msg().logical_id), None);
}

#[test]
fn kv27_wrong_receipts_never_install_or_release() {
    for field in 0..6 {
        let mut m = mailbox(Endpoint::Replica(0));
        let p = m.plan(action("enqueue")).unwrap();
        let before = p.persistence_bytes().to_vec();
        let mut r = receipt(&p, 0x55);
        match field {
            0 => r.owner = Endpoint::Reader(99),
            1 => r.base_revision = 99,
            2 => r.transition_id = [9; 32],
            3 => r.candidate_record_hash = [9; 32],
            4 => r.storage_receipt = [0; 32],
            5 => r.owner = Endpoint::Replica(1),
            _ => unreachable!(),
        };
        error(m.confirm(p, r), Error::InvalidReceipt);
        assert_eq!(m.revision(), 0);
        assert_eq!(m.retry_due_us(msg().logical_id), None);
        assert_eq!(
            before,
            m.plan(action("enqueue")).unwrap().persistence_bytes()
        );
    }
}

#[test]
fn kv28_29_durability_transitions_do_not_release_during_plan() {
    let mut sender = mailbox(Endpoint::Replica(0));
    let p = sender.plan(action("enqueue")).unwrap();
    assert_eq!(sender.revision(), 0);
    assert_eq!(sender.retry_due_us(msg().logical_id), None);
    let r = receipt(&p, 0x55);
    let c = sender.confirm(p, r).unwrap();
    assert!(matches!(
        c.releases.as_slice(),
        [CustodyRelease::Enqueued { .. }]
    ));
    let p = sender.plan(action("attempt")).unwrap();
    assert_eq!(sender.retry_due_us(msg().logical_id), Some(1_000_000));
    let r = receipt(&p, 0x55);
    let c = sender.confirm(p, r).unwrap();
    assert!(matches!(c.releases.as_slice(), [CustodyRelease::Send(_)]));
    assert_eq!(sender.retry_due_us(msg().logical_id), Some(1_020_000));
    let mut receiver = mailbox(Endpoint::Replica(1));
    let p = receiver.plan(action("receive")).unwrap();
    assert!(receiver.pending_inbox().is_empty());
    let r = receipt(&p, 0x66);
    let c = receiver.confirm(p, r).unwrap();
    assert!(matches!(
        c.releases.as_slice(),
        [
            CustodyRelease::InboxAvailable { .. },
            CustodyRelease::ReceiveAck(_)
        ]
    ));
    assert_eq!(receiver.pending_inbox().len(), 1);
}

#[test]
fn kv30_duplicate_first_receipt_consumed_history_and_receive_time_rule() {
    let mut m = mailbox(Endpoint::Replica(1));
    confirm(&mut m, action("receive"));
    let mut duplicate = action("receive");
    if let MailboxAction::Receive {
        received_tick_us,
        received_local_us,
        ..
    } = &mut duplicate
    {
        *received_tick_us = 1_050_000;
        *received_local_us = Some(1_051_000);
    }
    let c = confirm(&mut m, duplicate);
    assert!(matches!(
        c.releases.as_slice(),
        [CustodyRelease::ReceiveAck(_)]
    ));
    let item = &m.pending_inbox()[0];
    assert_eq!(item.first_receive_tick_us, 1_020_000);
    assert_eq!(item.first_receive_local_us, Some(1_021_000));
    let c = confirm(
        &mut m,
        MailboxAction::MarkConsumed {
            logical_id: msg().logical_id,
        },
    );
    assert!(c.releases.is_empty());
    assert!(m.pending_inbox().is_empty());
    let c = confirm(&mut m, action("receive"));
    assert!(matches!(
        c.releases.as_slice(),
        [CustodyRelease::ReceiveAck(_)]
    ));
    assert!(m.pending_inbox().is_empty());
    // The frozen API's Receive historical position does not advance the owner watermark.
    let mut outgoing = changed_message(msg(), Endpoint::Replica(0), 77, 1000, b"reply".to_vec());
    outgoing.source = Endpoint::Replica(1);
    outgoing.logical_id = message_id(&outgoing);
    confirm(
        &mut m,
        MailboxAction::Enqueue {
            message: outgoing,
            now_us: 1000,
        },
    );
}

#[test]
fn kv31_32_ack_delete_tombstone_and_reenqueue_are_durable_and_idempotent() {
    let mut active = mailbox(Endpoint::Replica(0));
    confirm(&mut active, action("enqueue"));
    let rev = active.revision();
    assert!(confirm(&mut active, action("enqueue")).releases.is_empty());
    assert_eq!(active.revision(), rev + 1);
    let mut m = mailbox(Endpoint::Replica(0));
    confirm(&mut m, action("enqueue"));
    confirm(&mut m, action("attempt"));
    let p = m.plan(action("delete")).unwrap();
    assert_eq!(m.retry_due_us(msg().logical_id), Some(1_020_000));
    let r = receipt(&p, 0x55);
    m.confirm(p, r).unwrap();
    assert_eq!(m.retry_due_us(msg().logical_id), None);
    for a in [action("delete"), action("enqueue")] {
        let rev = m.revision();
        let c = confirm(&mut m, a);
        assert_eq!(m.revision(), rev + 1);
        assert!(c.releases.is_empty());
        assert_eq!(m.retry_due_us(msg().logical_id), None);
    }
    let p = m.plan(action("enqueue")).unwrap();
    let bytes = std::str::from_utf8(p.persistence_bytes()).unwrap();
    assert!(bytes.contains("\"payload\":null"));
    assert!(bytes.contains("\"last_envelope\":null,\"acknowledged\":true"));
}

#[test]
fn active_ack_requires_attempt_and_supports_new_receiver_incarnation() {
    let mut unattempted = mailbox(Endpoint::Replica(0));
    confirm(&mut unattempted, action("enqueue"));
    let before = unattempted
        .plan(action("enqueue"))
        .unwrap()
        .persistence_bytes()
        .to_vec();
    error(unattempted.plan(action("delete")), Error::InvalidReceipt);
    assert_eq!(
        before,
        unattempted
            .plan(action("enqueue"))
            .unwrap()
            .persistence_bytes()
    );
    for receiver_inc in [1, 2] {
        let mut m = mailbox(Endpoint::Replica(0));
        confirm(&mut m, action("enqueue"));
        confirm(&mut m, action("attempt"));
        let mut a = action("delete");
        if let MailboxAction::Acknowledge {
            ack,
            authenticated_receiver,
        } = &mut a
        {
            ack.receiver_incarnation = receiver_inc;
            authenticated_receiver.incarnation = receiver_inc;
        }
        let c = confirm(&mut m, a);
        assert!(matches!(
            c.releases.as_slice(),
            [CustodyRelease::Acknowledged { .. }]
        ));
    }
    let mut m = mailbox(Endpoint::Replica(0));
    confirm(&mut m, action("enqueue"));
    let mut a = action("attempt");
    if let MailboxAction::Attempt { destination, .. } = &mut a {
        destination.incarnation = 2;
    }
    confirm(&mut m, a);
    // API §7 classifies an obsolete physical binding as BindingMismatch.
    error(m.plan(action("delete")), Error::BindingMismatch);
}

#[test]
fn snapshot_tombstone_preserves_possible_original_retry_deadline() {
    let g = golden();
    let row = &g["custody_records"]["delete"];
    let original = row["record_canonical_utf8"].as_str().unwrap().as_bytes();
    let changed = replace_once(original, "\"retry_due_us\":1020000", "\"retry_due_us\":0");
    let r = DurabilityReceipt {
        owner: Endpoint::Replica(0),
        base_revision: 2,
        transition_id: id(row["transition_id_hex"].as_str().unwrap()),
        candidate_record_hash: framed_hash("SIMT/custody-record/v1", &[&changed]),
        storage_receipt: [0x55; 32],
    };
    save_evidence("tombstone-valid-record.json", original);
    save_evidence("tombstone-impossible-retry-record.json", &changed);
    save_evidence(
        "tombstone-mutant-mock-receipt.json",
        &serde_json::to_vec(&r).unwrap(),
    );
    let result = ReliableMailbox::restore(config(), &changed, r);
    if let Ok((m, release)) = &result {
        eprintln!(
            "MALFORMED_TOMBSTONE_ACCEPTED revision={} releases={release:?}",
            m.revision()
        );
    }
    // Even without a prior state, one Attempt after origin 1_000_000 with
    // retry interval 20_000 cannot leave an original retry deadline of zero.
    error(result, Error::CorruptSnapshot);
}

#[test]
fn kv33_restore_preserves_record_release_identity_and_original_retry() {
    for name in ["enqueue", "attempt", "receive", "delete"] {
        let g = golden();
        let row = &g["custody_records"][name];
        let bytes = row["record_canonical_utf8"].as_str().unwrap().as_bytes();
        let owner = parse(&row["record"]["owner"]);
        let r = DurabilityReceipt {
            owner,
            base_revision: row["record"]["base_revision"].as_u64().unwrap(),
            transition_id: id(row["transition_id_hex"].as_str().unwrap()),
            candidate_record_hash: id(row["record_hash_hex"].as_str().unwrap()),
            storage_receipt: [0x66; 32],
        };
        let (m, replay) = ReliableMailbox::restore(config(), bytes, r.clone()).unwrap();
        assert_eq!(replay.transition_id, r.transition_id);
        assert_eq!(replay.revision, m.revision());
        let (again, again_replay) = ReliableMailbox::restore(config(), bytes, r).unwrap();
        assert_eq!(replay, again_replay);
        assert_eq!(m.pending_inbox(), again.pending_inbox());
        if name == "attempt" {
            assert_eq!(m.retry_due_us(msg().logical_id), Some(1_020_000));
        }
        if name == "receive" {
            assert_eq!(m.pending_inbox()[0].first_receive_tick_us, 1_020_000);
        }
    }
}

#[test]
fn kv34_stale_and_cross_instance_custody_confirmation() {
    let mut m = mailbox(Endpoint::Replica(0));
    let stale = m.plan(action("enqueue")).unwrap();
    let stale_receipt = receipt(&stale, 0x55);
    let retained_record = stale.persistence_bytes().to_vec();
    confirm(&mut m, action("enqueue"));
    error(m.confirm(stale, stale_receipt.clone()), Error::StalePlan);
    // Already-persisted bytes are still recoverable; a StalePlan is not durable cancellation.
    let (_, recovered) =
        ReliableMailbox::restore(config(), &retained_record, stale_receipt).unwrap();
    assert!(matches!(
        recovered.releases.as_slice(),
        [CustodyRelease::Enqueued { .. }]
    ));
    let p = m.plan(action("enqueue")).unwrap();
    let r = receipt(&p, 0x55);
    let mut clone = m.clone();
    error(clone.confirm(p, r), Error::CrossInstance);
    let p = m.plan(action("enqueue")).unwrap();
    let r = receipt(&p, 0x55);
    let record = &golden()["custody_records"]["enqueue"];
    let restore_receipt = DurabilityReceipt {
        owner: Endpoint::Replica(0),
        base_revision: 0,
        transition_id: id(record["transition_id_hex"].as_str().unwrap()),
        candidate_record_hash: id(record["record_hash_hex"].as_str().unwrap()),
        storage_receipt: [0x55; 32],
    };
    let (mut restored, _) = ReliableMailbox::restore(
        config(),
        record["record_canonical_utf8"].as_str().unwrap().as_bytes(),
        restore_receipt,
    )
    .unwrap();
    error(restored.confirm(p, r), Error::CrossInstance);
}

#[test]
fn kv35_retry_due_attempt_progress_and_rebind_preserve_origin() {
    let mut m = mailbox(Endpoint::Replica(0));
    confirm(&mut m, action("enqueue"));
    confirm(&mut m, action("attempt"));
    let a = MailboxAction::Attempt {
        logical_id: msg().logical_id,
        now_us: 1_019_999,
        destination: EndpointBinding {
            endpoint: Endpoint::Replica(1),
            incarnation: 1,
        },
    };
    error(m.plan(a), Error::RetryNotDue);
    confirm(
        &mut m,
        MailboxAction::Rebind {
            expected_incarnation: 1,
            new_incarnation: 2,
        },
    );
    assert_eq!(m.retry_due_us(msg().logical_id), Some(1_020_000));
    let c = confirm(
        &mut m,
        MailboxAction::Attempt {
            logical_id: msg().logical_id,
            now_us: 1_020_000,
            destination: EndpointBinding {
                endpoint: Endpoint::Replica(1),
                incarnation: 2,
            },
        },
    );
    match &c.releases[0] {
        CustodyRelease::Send(e) => {
            assert_eq!(e.attempt, 2);
            assert_eq!(e.source_incarnation, 2);
            assert_eq!(e.message, msg());
        }
        _ => panic!(),
    };
    assert_eq!(m.retry_due_us(msg().logical_id), Some(1_040_000));
}

#[test]
fn kv36_mailbox_capacity_counts_tombstones_consumed_and_payload() {
    let mut m = ReliableMailbox::new(
        config(),
        Endpoint::Replica(0),
        ReliableLimits {
            outbox_records: 1,
            ..ReliableLimits::default()
        },
        20_000,
    )
    .unwrap();
    confirm(&mut m, action("enqueue"));
    confirm(&mut m, action("attempt"));
    confirm(&mut m, action("delete"));
    error(
        m.plan(MailboxAction::Enqueue {
            message: changed_message(msg(), Endpoint::Replica(1), 3, 1_000_000, b"new".to_vec()),
            now_us: 1_000_000,
        }),
        Error::ResourceLimit,
    );
    let mut m = ReliableMailbox::new(
        config(),
        Endpoint::Replica(1),
        ReliableLimits {
            inbox_records: 1,
            ..ReliableLimits::default()
        },
        20_000,
    )
    .unwrap();
    confirm(&mut m, action("receive"));
    confirm(
        &mut m,
        MailboxAction::MarkConsumed {
            logical_id: msg().logical_id,
        },
    );
    let mut a = action("receive");
    if let MailboxAction::Receive { envelope, .. } = &mut a {
        envelope.message =
            changed_message(msg(), Endpoint::Replica(1), 3, 1_000_000, b"new".to_vec());
    }
    error(m.plan(a), Error::ResourceLimit);
    let m = ReliableMailbox::new(
        config(),
        Endpoint::Replica(0),
        ReliableLimits {
            retained_payload_bytes: 2,
            ..ReliableLimits::default()
        },
        20_000,
    )
    .unwrap();
    error(m.plan(action("enqueue")), Error::ResourceLimit);
}

#[test]
fn custody_message_mutations_wrong_auth_and_best_effort_are_rejected() {
    let mut m = mailbox(Endpoint::Replica(1));
    confirm(&mut m, action("receive"));
    for mutate in 0..3 {
        let mut a = action("receive");
        if let MailboxAction::Receive {
            envelope,
            authenticated_source,
            ..
        } = &mut a
        {
            match mutate {
                0 => envelope.message.payload[0] ^= 1,
                1 => authenticated_source.incarnation = 2,
                2 => envelope.destination_incarnation = 2,
                _ => unreachable!(),
            }
        }
        assert!(m.plan(a).is_err());
        assert_eq!(m.pending_inbox().len(), 1);
    }
    let mut c = config();
    c.channels[0].reliability = Reliability::BestEffort;
    let m =
        ReliableMailbox::new(c, Endpoint::Replica(0), ReliableLimits::default(), 20_000).unwrap();
    assert!(m.plan(action("enqueue")).is_err());
}

#[test]
fn kv37_43_clock_offset_age_and_lease_equality() {
    for (epsilon, want) in [(1000, 1_011_250), (-1000, 1_009_250)] {
        let ident = clock_identity(epsilon);
        let mut c = ClockTracker::new(ident.clone()).unwrap();
        let request = c.request().unwrap();
        c.install(1_010_250, sample(&ident, request, 3, 1_000_250, 1_100_000))
            .unwrap();
        let before = c.snapshot_bytes().unwrap();
        let view = c.view(1_010_250).unwrap();
        assert_eq!(view.local_time_us(), want);
        assert_eq!(view.reference_tick_us(), 1_010_250);
        assert_eq!(view.lease_end_tick_us(), 1_100_000);
        assert!(c.view(1_099_999).is_ok());
        error(c.view(1_100_000), Error::LeaseExpired);
        error(c.view(1_100_001), Error::LeaseExpired);
        assert_eq!(before, c.snapshot_bytes().unwrap());
    }
}

#[test]
fn kv38_39_clock_poisoning_out_of_order_and_wrong_sample_atomicity() {
    let ident = clock_identity(1000);
    let mut c = ClockTracker::new(ident.clone()).unwrap();
    let r1 = c.request().unwrap();
    let r2 = c.request().unwrap();
    let calibration = c.request().unwrap();
    c.install(900, sample(&ident, calibration, 2, 900, 100000))
        .unwrap();
    let before = c.snapshot_bytes().unwrap();
    let mut bad = sample(&ident, r1, 100, 1000, 100000);
    bad.uncertainty_us = 1001;
    error(c.install(1001, bad), Error::InvalidSample);
    assert_eq!(before, c.snapshot_bytes().unwrap());
    c.install(1002, sample(&ident, r2, 3, 1000, 100000))
        .unwrap();
    let r3 = c.request().unwrap();
    c.install(1003, sample(&ident, r1, 5, 1000, 110000))
        .unwrap();
    let before = c.snapshot_bytes().unwrap();
    error(
        c.install(1004, sample(&ident, r3, 4, 1000, 120000)),
        Error::StaleSample,
    );
    assert_eq!(before, c.snapshot_bytes().unwrap());
    assert_eq!(c.view(1004).unwrap().sample_sequence(), 5);
    assert_eq!(c.view(1004).unwrap().lease_end_tick_us(), 110000);
    for kind in 0..5 {
        let mut s = sample(&ident, r3, 6, 1000, 120000);
        match kind {
            0 => s.domain = [7; 32],
            1 => s.rate_profile = [7; 32],
            2 => s.as_of_tick_us = 2000,
            3 => s.lease_end_tick_us = 1004,
            4 => s.local_at_as_of_us = 0,
            _ => unreachable!(),
        }
        assert!(c.install(1004, s).is_err());
        assert_eq!(before, c.snapshot_bytes().unwrap());
    }
}

#[test]
fn kv40_clock_request_capacity_and_counter_no_poisoning() {
    let ident = clock_identity(0);
    let mut c = ClockTracker::new(ident.clone()).unwrap();
    for n in 1..=64 {
        assert_eq!(c.request().unwrap(), n);
    }
    let before = c.snapshot_bytes().unwrap();
    error(c.request(), Error::ResourceLimit);
    assert_eq!(before, c.snapshot_bytes().unwrap());
    c.install(1001, sample(&ident, 1, 1, 1000, 2000)).unwrap();
    assert_eq!(c.request().unwrap(), 65);
}

#[test]
fn kv41_clock_rebind_preserves_request_sequence_and_delivery_floor() {
    let mut ident = clock_identity(1000);
    let mut c = ClockTracker::new(ident.clone()).unwrap();
    let request = c.request().unwrap();
    c.install(1_050_000, sample(&ident, request, 3, 1_049_000, 1_100_000))
        .unwrap();
    c.rebind(1, 2).unwrap();
    ident.incarnation = 2;
    error(c.view(1_050_000), Error::Uncalibrated);
    assert_eq!(c.request().unwrap(), 2);
    let before = c.snapshot_bytes().unwrap();
    error(
        c.install(1_049_999, sample(&ident, 2, 4, 1_049_000, 1_100_000)),
        Error::TimeRegression,
    );
    assert_eq!(before, c.snapshot_bytes().unwrap());
    let mut restored = ClockTracker::from_snapshot(ident.clone(), &before).unwrap();
    restored
        .install(1_050_000, sample(&ident, 2, 4, 1_049_000, 1_100_000))
        .unwrap();
    assert_eq!(restored.view(1_050_000).unwrap().sample_sequence(), 4);
}

#[test]
fn kv42_clock_deadline_clamp_offsets_and_checked_projection() {
    for (epsilon, deadline) in [(1000, 1_024_000), (-1000, 1_026_000)] {
        let i = clock_identity(epsilon);
        let mut c = ClockTracker::new(i.clone()).unwrap();
        let r = c.request().unwrap();
        c.install(1000, sample(&i, r, 1, 1000, 2_000_000)).unwrap();
        let v = c.view(1000).unwrap();
        assert_eq!(v.local_deadline_tick_us(1_025_000).unwrap(), deadline);
        assert_eq!(v.local_deadline_tick_us(-2000).unwrap(), 0);
    }
    let i = clock_identity(1000);
    let mut c = ClockTracker::new(i.clone()).unwrap();
    let r = c.request().unwrap();
    c.install(1000, sample(&i, r, 1, 1000, u64::MAX)).unwrap();
    error(c.view(i64::MAX as u64), Error::ArithmeticOverflow);
    assert!(ClockTracker::new(clock_identity(1001)).is_err());
    assert!(ClockTracker::new(clock_identity(-1001)).is_err());
}

#[test]
fn clock_snapshot_canonical_local_invariants() {
    let ident = clock_identity(0);
    let mut c = ClockTracker::new(ident.clone()).unwrap();
    let r = c.request().unwrap();
    c.install(1001, sample(&ident, r, 3, 1000, 2000)).unwrap();
    let bytes = c.snapshot_bytes().unwrap();
    let changed = replace_once(&bytes, "\"accepted_sequence\":3", "\"accepted_sequence\":2");
    error(
        ClockTracker::from_snapshot(ident.clone(), &changed),
        Error::CorruptSnapshot,
    );
    let changed = replace_once(&bytes, "\"request_counter\":1", "\"request_counter\":0");
    error(
        ClockTracker::from_snapshot(ident.clone(), &changed),
        Error::CorruptSnapshot,
    );
    let mut spaced = bytes;
    spaced.insert(1, b' ');
    error(
        ClockTracker::from_snapshot(ident, &spaced),
        Error::NonCanonical,
    );
}

#[test]
fn kv44_grant_consumer_reader_isolation_and_opaque_effects() {
    struct Reader {
        target: EndpointBinding,
        received: Vec<Vec<u8>>,
    }
    impl GrantConsumer for Reader {
        type Error = &'static str;
        fn on_grant(&mut self, g: ServiceGrant) -> Result<Vec<ServiceEffect>, Self::Error> {
            if g.target != self.target {
                return Err("target");
            }
            match g.input {
                ServiceInput::Message(d) => {
                    if d.envelope.message.destination != self.target.endpoint {
                        return Err("destination");
                    }
                    self.received.push(d.envelope.message.payload);
                    Ok(vec![])
                }
                _ => Err("message expected"),
            }
        }
    }
    let mut c = plain_config();
    let reader2 = EndpointRegistration {
        binding: EndpointBinding {
            endpoint: Endpoint::Reader(2),
            incarnation: 1,
        },
        ..c.endpoints[3].clone()
    };
    c.endpoints.push(reader2);
    let mut k = RouteKernel::new(c).unwrap();
    let e = Envelope {
        message: changed_message(msg(), Endpoint::Reader(1), 1, 1000, b"opaque q2".to_vec()),
        ..envelope()
    };
    route(&mut k, 1, 1000, &[RouteCommand::Send(e)]);
    let deliveries = delivered(&route(&mut k, 2, 1020, &[RouteCommand::Drain]));
    assert_eq!(deliveries.len(), 1);
    let mut ci = clock_identity(0);
    ci.endpoint = Endpoint::Reader(1);
    let mut clock = ClockTracker::new(ci.clone()).unwrap();
    let req = clock.request().unwrap();
    clock
        .install(1000, sample(&ci, req, 1, 1000, 2000))
        .unwrap();
    let mut a = Reader {
        target: EndpointBinding {
            endpoint: Endpoint::Reader(1),
            incarnation: 1,
        },
        received: vec![],
    };
    let b = Reader {
        target: EndpointBinding {
            endpoint: Endpoint::Reader(2),
            incarnation: 1,
        },
        received: vec![],
    };
    a.on_grant(ServiceGrant {
        target: a.target,
        step: step(2, 1020),
        clock: clock.view(1020).unwrap(),
        input: ServiceInput::Message(deliveries[0].clone()),
    })
    .unwrap();
    assert_eq!(a.received, vec![b"opaque q2".to_vec()]);
    assert!(b.received.is_empty());
}

#[test]
fn kv46_canonical_record_semantic_mutations_and_receipt_scope() {
    let g = golden();
    let row = &g["custody_records"]["receive"];
    let original = row["record_canonical_utf8"].as_str().unwrap().as_bytes();
    for changed in [
        replace_once(original, "\"revision\":1", "\"revision\":2"),
        replace_once(
            original,
            "\"first_receive_tick_us\":1020000",
            "\"first_receive_tick_us\":999999",
        ),
        replace_once(
            original,
            "\"retry_interval_us\":20000",
            "\"retry_interval_us\":0",
        ),
    ] {
        // Rehashing the deliberately malformed record isolates structural validation;
        // this mock receipt is not a real storage/cryptographic authentication claim.
        let r = DurabilityReceipt {
            owner: Endpoint::Replica(1),
            base_revision: 0,
            transition_id: id(row["transition_id_hex"].as_str().unwrap()),
            candidate_record_hash: framed_hash("SIMT/custody-record/v1", &[&changed]),
            storage_receipt: [0x66; 32],
        };
        assert!(serde_json::from_slice::<Value>(&changed).is_ok());
        error(
            ReliableMailbox::restore(config(), &changed, r),
            Error::CorruptSnapshot,
        );
    }
    let mut spaced = original.to_vec();
    spaced.insert(1, b' ');
    let r = DurabilityReceipt {
        owner: Endpoint::Replica(1),
        base_revision: 0,
        transition_id: id(row["transition_id_hex"].as_str().unwrap()),
        candidate_record_hash: framed_hash("SIMT/custody-record/v1", &[&spaced]),
        storage_receipt: [0x66; 32],
    };
    error(
        ReliableMailbox::restore(config(), &spaced, r),
        Error::NonCanonical,
    );
}
