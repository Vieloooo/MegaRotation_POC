use sim_transport::*;

const R0: Endpoint = Endpoint::Replica(0);
const R1: Endpoint = Endpoint::Replica(1);
const CHANNEL: ChannelId = ChannelId(7);
const START: Tick = 1_000_000;

fn binding(endpoint: Endpoint, incarnation: u64) -> EndpointBinding {
    EndpointBinding {
        endpoint,
        incarnation,
    }
}

fn config(profile: LinkProfile) -> TransportConfig {
    let permit = ChannelPermit {
        channel: CHANNEL,
        contexts: vec![[0x33; 32]],
    };
    TransportConfig {
        schema: 1,
        run: [0x11; 32],
        config: [0x22; 32],
        seed: 7,
        limits: Limits::default(),
        endpoints: [R0, R1, Endpoint::Service(6), Endpoint::Reader(1)]
            .map(|endpoint| EndpointRegistration {
                binding: binding(endpoint, 1),
                send: vec![permit.clone()],
                receive: vec![permit.clone()],
            })
            .to_vec(),
        channels: vec![ChannelSpec {
            id: CHANNEL,
            context: ContextPolicy::Required,
            reliability: Reliability::Reliable,
        }],
        rules: vec![LinkRule {
            source: EndpointSelector::Any,
            destination: EndpointSelector::Any,
            channel: ChannelSelector::Exact(CHANNEL),
            profile,
        }],
    }
}

fn message(effect_index: u32, payload: Vec<u8>) -> Message {
    let mut message = Message {
        run: [0x11; 32],
        config: [0x22; 32],
        source: R0,
        destination: R1,
        channel: CHANNEL,
        context: Some([0x33; 32]),
        codec: [0x44; 32],
        origin: Origin {
            step_id: 7,
            effect_index,
            sent_tick_us: START,
            source_incarnation: 1,
        },
        payload_hash: payload_hash(&payload),
        logical_id: [0; 32],
        payload,
    };
    message.logical_id = message_id(&message);
    message
}

fn envelope(message: Message) -> Envelope {
    Envelope {
        message,
        attempt: 1,
        source_incarnation: 1,
        destination_incarnation: 1,
    }
}

fn stamp(logical_step_id: u64, event_id: u64, tick_us: Tick) -> StepStamp {
    StepStamp {
        logical_step_id,
        tick_us,
        event_id,
        causal_wave: 0,
    }
}

fn route(kernel: &mut RouteKernel, step: StepStamp, commands: &[RouteCommand]) -> Vec<RouteEvent> {
    kernel.commit(kernel.plan(step, commands).unwrap()).unwrap()
}

fn golden() -> TransportConfig {
    config(LinkProfile {
        jitter_us: 1000,
        ..LinkProfile::default()
    })
}

fn hash_text(id: Id) -> String {
    id.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn receipt(plan: &CustodyPlan) -> DurabilityReceipt {
    DurabilityReceipt {
        owner: plan.owner(),
        base_revision: plan.base_revision(),
        transition_id: plan.transition_id(),
        candidate_record_hash: plan.candidate_record_hash(),
        storage_receipt: [0x66; 32],
    }
}

#[test]
fn independently_frozen_digests_and_custody_records() {
    let config = golden();
    assert_eq!(
        hash_text(config.hash().unwrap()),
        "c71654ba27624d989e8ac44469b5b055c061980eecf693f67bfb30424359438c"
    );
    let message = message(2, b"abc".to_vec());
    assert_eq!(
        hash_text(message.logical_id),
        "7423e6af569d272d5d2a80634d61b4a84e615a8d5e95ce2c7e32771702206b73"
    );
    let mut sender =
        ReliableMailbox::new(config.clone(), R0, ReliableLimits::default(), 20_000).unwrap();
    let mut receiver =
        ReliableMailbox::new(config.clone(), R1, ReliableLimits::default(), 20_000).unwrap();

    let planned_enqueue = sender
        .plan(MailboxAction::Enqueue {
            message: message.clone(),
            now_us: START,
        })
        .unwrap();
    assert_eq!(planned_enqueue.persistence_bytes().len(), 2709);
    assert_eq!(
        hash_text(planned_enqueue.candidate_record_hash()),
        "3b131d0ce885fd3d740253d12c1f3a4e7256c1f9ea058f2e98b958a466a03a4f"
    );
    let enqueue_receipt = receipt(&planned_enqueue);
    let releases = sender
        .confirm(planned_enqueue, enqueue_receipt)
        .unwrap()
        .releases;
    assert!(matches!(&releases[..], [CustodyRelease::Enqueued { .. }]));

    let planned_attempt = sender
        .plan(MailboxAction::Attempt {
            logical_id: message.logical_id,
            now_us: START,
            destination: binding(R1, 1),
        })
        .unwrap();
    assert_eq!(planned_attempt.persistence_bytes().len(), 3795);
    assert_eq!(
        hash_text(planned_attempt.candidate_record_hash()),
        "4bad46f53c1b857e4ae8a1a99b05c8ce0aa656ceaa3ed768adf5ec9bcdb1572a"
    );
    let attempt_receipt = receipt(&planned_attempt);
    let (restored, recovered_release) = ReliableMailbox::restore(
        config.clone(),
        planned_attempt.persistence_bytes(),
        attempt_receipt.clone(),
    )
    .unwrap();
    let committed = sender.confirm(planned_attempt, attempt_receipt).unwrap();
    assert_eq!(recovered_release, committed);
    assert_eq!(restored.retry_due_us(message.logical_id), Some(1_020_000));
    let [CustodyRelease::Send(envelope)] = committed.releases.as_slice() else {
        panic!("send missing")
    };

    let planned_receive = receiver
        .plan(MailboxAction::Receive {
            envelope: envelope.clone(),
            received_tick_us: 1_020_000,
            received_local_us: Some(1_021_000),
            authenticated_source: binding(R0, 1),
        })
        .unwrap();
    assert!(receiver.pending_inbox().is_empty());
    assert_eq!(planned_receive.persistence_bytes().len(), 3036);
    assert_eq!(
        hash_text(planned_receive.candidate_record_hash()),
        "c49322b4f1e849fd75687cc015320f5b60c074f826e8085d6565d95554d726cb"
    );
    let receive_receipt = receipt(&planned_receive);
    let (_, recovered_release) = ReliableMailbox::restore(
        config.clone(),
        planned_receive.persistence_bytes(),
        receive_receipt.clone(),
    )
    .unwrap();
    let committed = receiver.confirm(planned_receive, receive_receipt).unwrap();
    assert_eq!(recovered_release, committed);
    let [
        CustodyRelease::InboxAvailable { .. },
        CustodyRelease::ReceiveAck(ack),
    ] = committed.releases.as_slice()
    else {
        panic!("inbox and ACK missing")
    };

    let planned_delete = sender
        .plan(MailboxAction::Acknowledge {
            ack: ack.clone(),
            authenticated_receiver: binding(R1, 1),
        })
        .unwrap();
    assert_eq!(planned_delete.persistence_bytes().len(), 3040);
    assert_eq!(
        hash_text(planned_delete.candidate_record_hash()),
        "03b1e24360f9720def7ae717bcea6d24f76a7c9ba6aecc7a9e9cf133181e3468"
    );
    let delete_receipt = receipt(&planned_delete);
    assert!(sender.retry_due_us(message.logical_id).is_some());
    let (_, recovered_release) = ReliableMailbox::restore(
        config,
        planned_delete.persistence_bytes(),
        delete_receipt.clone(),
    )
    .unwrap();
    assert_eq!(
        sender.confirm(planned_delete, delete_receipt).unwrap(),
        recovered_release
    );
    assert!(sender.retry_due_us(message.logical_id).is_none());
}

#[test]
fn invalid_tail_and_cross_instance_plans_do_not_mutate() {
    let cfg = golden();
    let mut kernel = RouteKernel::new(cfg.clone()).unwrap();
    let good = RouteCommand::Send(envelope(message(2, b"abc".to_vec())));
    let mut invalid = envelope(message(3, b"invalid".to_vec()));
    invalid.destination_incarnation = 2;
    let original = kernel.snapshot_bytes().unwrap();
    assert_eq!(
        kernel
            .plan(
                stamp(1, 20, START),
                &[good.clone(), RouteCommand::Send(invalid)]
            )
            .unwrap_err(),
        Error::BindingMismatch
    );
    assert_eq!(kernel.snapshot_bytes().unwrap(), original);

    let plan = kernel
        .plan(stamp(1, 20, START), std::slice::from_ref(&good))
        .unwrap();
    let other = kernel.plan(stamp(2, 19, START), &[]).unwrap();
    let mut clone = kernel.clone();
    assert_eq!(clone.commit(other).unwrap_err(), Error::CrossInstance);
    assert_eq!(clone.snapshot_bytes().unwrap(), original);
    let stale = kernel.plan(stamp(2, 19, START), &[]).unwrap();
    let candidate = plan.candidate_snapshot_bytes().unwrap();
    assert!(
        matches!(&kernel.commit(plan).unwrap()[0].kind, RouteEventKind::Queued { packet, raw_due_us: 1_020_004 }
        if packet.packet_seq == 1)
    );
    assert_eq!(kernel.commit(stale).unwrap_err(), Error::StalePlan);
    assert_eq!(kernel.snapshot_bytes().unwrap(), candidate);

    let mut restored = RouteKernel::from_snapshot(cfg, &candidate).unwrap();
    assert_eq!(restored.snapshot_bytes().unwrap(), candidate);
    let tick = restored.next_wakeup_us().unwrap();
    let delivered = route(&mut restored, stamp(2, 3, tick), &[RouteCommand::Drain]);
    assert!(
        matches!(&delivered[0].kind, RouteEventKind::Delivered(d) if d.delivered_tick_us == tick)
    );
}

#[test]
fn late_hold_and_fifo_recompute_after_actual_drain_tick() {
    let mut kernel = RouteKernel::new(config(LinkProfile {
        latency_us: 1,
        offline: vec![OfflineInterval {
            start_us: START + 50,
            end_us: START + 100,
        }],
        faults: vec![FaultEntry {
            send_ordinal: 1,
            drop: false,
            duplicate_delay_us: None,
            extra_delay_us: 5,
        }],
        ..LinkProfile::default()
    }))
    .unwrap();
    route(
        &mut kernel,
        stamp(1, 7, START),
        &[
            RouteCommand::Send(envelope(message(1, b"abc".to_vec()))),
            RouteCommand::Send(envelope(message(2, vec![]))),
        ],
    );
    assert_eq!(kernel.next_wakeup_us(), Some(START + 6));
    assert!(route(&mut kernel, stamp(2, 6, START + 70), &[RouteCommand::Drain]).is_empty());
    assert_eq!(kernel.next_wakeup_us(), Some(START + 100));
    let first = route(
        &mut kernel,
        stamp(3, 5, START + 100),
        &[RouteCommand::Drain],
    );
    assert!(
        matches!(&first[..], [RouteEvent { kind: RouteEventKind::Delivered(d), .. }] if d.packet.packet_seq == 1)
    );
    assert_eq!(kernel.next_wakeup_us(), Some(START + 103));
    let second = route(
        &mut kernel,
        stamp(4, 4, START + 103),
        &[RouteCommand::Drain],
    );
    assert!(
        matches!(&second[..], [RouteEvent { kind: RouteEventKind::Delivered(d), .. }] if d.packet.packet_seq == 2)
    );
}

#[test]
fn duplicate_copy_consumes_capacity_but_keeps_attempt() {
    let mut kernel = RouteKernel::new(config(LinkProfile {
        capacity: 1,
        faults: vec![FaultEntry {
            send_ordinal: 1,
            drop: false,
            duplicate_delay_us: Some(0),
            extra_delay_us: 0,
        }],
        ..LinkProfile::default()
    }))
    .unwrap();
    let envelope = envelope(message(0, vec![]));
    let events = route(
        &mut kernel,
        stamp(1, 1, START),
        &[RouteCommand::Send(envelope.clone())],
    );
    assert!(matches!(&events[..], [
        RouteEvent { effect_index: 0, kind: RouteEventKind::Queued { packet: first, .. }, .. },
        RouteEvent { effect_index: 1, kind: RouteEventKind::Dropped { packet: duplicate, reason: DropReason::LinkCapacity, raw_due_us: Some(_) }, .. },
    ] if first.attempt == duplicate.attempt && first.copy_index == 0 && duplicate.copy_index == 1
        && first.logical_id == duplicate.logical_id));
    assert_eq!(
        kernel
            .plan(stamp(2, 2, START), &[RouteCommand::Send(envelope)])
            .unwrap_err(),
        Error::InvalidAttempt
    );
}

#[test]
fn receiver_rebind_keeps_original_receipt_and_requires_matching_confirmation() {
    let cfg = golden();
    let mut receiver =
        ReliableMailbox::new(cfg.clone(), R1, ReliableLimits::default(), 20_000).unwrap();
    let message = message(2, b"abc".to_vec());
    let action = MailboxAction::Receive {
        envelope: envelope(message.clone()),
        received_tick_us: START + 5,
        received_local_us: Some(800),
        authenticated_source: binding(R0, 1),
    };
    let plan = receiver.plan(action.clone()).unwrap();
    let mut wrong = receipt(&plan);
    wrong.candidate_record_hash = [1; 32];
    assert_eq!(
        receiver.confirm(plan, wrong).unwrap_err(),
        Error::InvalidReceipt
    );
    assert_eq!(receiver.revision(), 0);
    assert!(receiver.pending_inbox().is_empty());
    let plan = receiver.plan(action).unwrap();
    let receipt1 = receipt(&plan);
    receiver.confirm(plan, receipt1).unwrap();

    let plan = receiver
        .plan(MailboxAction::Rebind {
            expected_incarnation: 1,
            new_incarnation: 2,
        })
        .unwrap();
    let receipt2 = receipt(&plan);
    let (mut receiver, rebind) =
        ReliableMailbox::restore(cfg, plan.persistence_bytes(), receipt2).unwrap();
    assert!(matches!(&rebind.releases[..], [CustodyRelease::Rebound(b)] if b.incarnation == 2));
    let action = MailboxAction::Receive {
        envelope: Envelope {
            destination_incarnation: 2,
            attempt: 2,
            ..envelope(message)
        },
        received_tick_us: START + 40,
        received_local_us: Some(1400),
        authenticated_source: binding(R0, 1),
    };
    let plan = receiver.plan(action).unwrap();
    let receipt3 = receipt(&plan);
    let releases = receiver.confirm(plan, receipt3).unwrap().releases;
    assert!(
        matches!(&releases[..], [CustodyRelease::ReceiveAck(ack)] if ack.receiver_incarnation == 2)
    );
    assert_eq!(receiver.pending_inbox().len(), 1);
    assert_eq!(receiver.pending_inbox()[0].first_receive_tick_us, START + 5);
    assert_eq!(
        receiver.pending_inbox()[0].first_receive_local_us,
        Some(800)
    );
}

#[test]
fn clock_rebind_preserves_success_watermark_and_lease_expires_at_equality() {
    let mut identity = ClockIdentity {
        run: [1; 32],
        config: [2; 32],
        endpoint: R0,
        incarnation: 1,
        domain: [3; 32],
        epsilon_us: 1000,
    };
    let mut clock = ClockTracker::new(identity.clone()).unwrap();
    let sample = ClockSample {
        run: identity.run,
        config: identity.config,
        endpoint: R0,
        incarnation: 1,
        domain: identity.domain,
        rate_profile: framed_hash("SIMT/rate/v1", &[b"fixed-rate-one-v1"]),
        request_id: clock.request().unwrap(),
        sequence: 2,
        as_of_tick_us: START,
        local_at_as_of_us: START as i64 + 1000,
        uncertainty_us: 1000,
        lease_end_tick_us: START + 100,
    };
    clock.install(START + 10, sample.clone()).unwrap();
    assert_eq!(
        clock.view(START + 99).unwrap().local_time_us(),
        START as i64 + 1099
    );
    assert_eq!(clock.view(START + 100).unwrap_err(), Error::LeaseExpired);
    clock.rebind(1, 2).unwrap();
    identity.incarnation = 2;
    let mut clock =
        ClockTracker::from_snapshot(identity, &clock.snapshot_bytes().unwrap()).unwrap();
    assert_eq!(clock.view(START + 11).unwrap_err(), Error::Uncalibrated);
    let next = ClockSample {
        incarnation: 2,
        sequence: 3,
        request_id: clock.request().unwrap(),
        ..sample
    };
    assert_eq!(next.request_id, 2);
    let original = clock.snapshot_bytes().unwrap();
    assert_eq!(
        clock.install(START + 9, next.clone()).unwrap_err(),
        Error::TimeRegression
    );
    assert_eq!(clock.snapshot_bytes().unwrap(), original);
    clock.install(START + 10, next).unwrap();
    let view = clock.view(START + 11).unwrap();
    assert_eq!(view.local_deadline_tick_us(-1001).unwrap(), 0);
    assert_eq!(view.local_deadline_tick_us(1_025_000).unwrap(), 1_024_000);
}

#[test]
fn ack_requires_an_attempt_and_checks_active_destination_incarnation() {
    let cfg = golden();
    let mut sender =
        ReliableMailbox::new(cfg.clone(), R0, ReliableLimits::default(), 20_000).unwrap();
    let mut receiver = ReliableMailbox::new(cfg, R1, ReliableLimits::default(), 20_000).unwrap();
    let message = message(2, b"abc".to_vec());
    let id = message.logical_id;
    let enqueue = MailboxAction::Enqueue {
        message: message.clone(),
        now_us: START,
    };
    let plan = sender.plan(enqueue.clone()).unwrap();
    let confirmation = receipt(&plan);
    sender.confirm(plan, confirmation).unwrap();

    let receive = MailboxAction::Receive {
        envelope: envelope(message),
        received_tick_us: START + 1,
        received_local_us: None,
        authenticated_source: binding(R0, 1),
    };
    let plan = receiver.plan(receive.clone()).unwrap();
    let confirmation = receipt(&plan);
    let committed = receiver.confirm(plan, confirmation).unwrap();
    let [_, CustodyRelease::ReceiveAck(old_ack)] = committed.releases.as_slice() else {
        panic!("receive ACK missing");
    };
    let ack = MailboxAction::Acknowledge {
        ack: old_ack.clone(),
        authenticated_receiver: binding(R1, 1),
    };
    assert_eq!(sender.plan(ack.clone()).unwrap_err(), Error::InvalidReceipt);

    let plan = sender
        .plan(MailboxAction::Attempt {
            logical_id: id,
            now_us: START,
            destination: binding(R1, 2),
        })
        .unwrap();
    let confirmation = receipt(&plan);
    let attempt = sender.confirm(plan, confirmation).unwrap();
    let [CustodyRelease::Send(new_envelope)] = attempt.releases.as_slice() else {
        panic!("expected one send");
    };
    assert_eq!(sender.plan(ack).unwrap_err(), Error::BindingMismatch);
    assert_eq!(sender.retry_due_us(id), Some(START + 20_000));

    let plan = receiver
        .plan(MailboxAction::Rebind {
            expected_incarnation: 1,
            new_incarnation: 2,
        })
        .unwrap();
    let confirmation = receipt(&plan);
    receiver.confirm(plan, confirmation).unwrap();
    let plan = receiver
        .plan(MailboxAction::Receive {
            envelope: new_envelope.clone(),
            received_tick_us: START + 2,
            received_local_us: None,
            authenticated_source: binding(R0, 1),
        })
        .unwrap();
    let confirmation = receipt(&plan);
    let committed = receiver.confirm(plan, confirmation).unwrap();
    let [CustodyRelease::ReceiveAck(ack)] = committed.releases.as_slice() else {
        panic!("expected duplicate ACK only");
    };

    let ack = MailboxAction::Acknowledge {
        ack: ack.clone(),
        authenticated_receiver: binding(R1, 2),
    };
    let plan = sender.plan(ack.clone()).unwrap();
    let confirmation = receipt(&plan);
    sender.confirm(plan, confirmation).unwrap();
    for duplicate in [ack, enqueue] {
        let plan = sender.plan(duplicate).unwrap();
        let confirmation = receipt(&plan);
        assert!(
            sender
                .confirm(plan, confirmation)
                .unwrap()
                .releases
                .is_empty()
        );
        assert_eq!(sender.retry_due_us(id), None);
    }
}

#[test]
fn route_snapshot_rejects_noncanonical_and_inconsistent_packet_data() {
    let cfg = golden();
    let mut kernel = RouteKernel::new(cfg.clone()).unwrap();
    route(
        &mut kernel,
        stamp(1, 1, START),
        &[RouteCommand::Send(envelope(message(2, b"abc".to_vec())))],
    );
    let bytes = kernel.snapshot_bytes().unwrap();
    let mut spaced = bytes.clone();
    spaced.push(b'\n');
    assert_eq!(
        RouteKernel::from_snapshot(cfg.clone(), &spaced).unwrap_err(),
        Error::NonCanonical
    );

    let canonical = String::from_utf8(bytes).unwrap();
    let too_early = canonical.replacen("\"raw_due_us\":1020004", "\"raw_due_us\":1000001", 1);
    assert_ne!(canonical, too_early);
    assert_eq!(
        RouteKernel::from_snapshot(cfg.clone(), too_early.as_bytes()).unwrap_err(),
        Error::CorruptSnapshot
    );
    let wrong_packet = canonical.replacen("\"packet_seq\":1", "\"packet_seq\":2", 1);
    assert_ne!(canonical, wrong_packet);
    assert_eq!(
        RouteKernel::from_snapshot(cfg, wrong_packet.as_bytes()).unwrap_err(),
        Error::CorruptSnapshot
    );
}
