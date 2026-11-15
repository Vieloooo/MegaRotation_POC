use super::*;

#[test]
fn kv01_02_golden_hash_enum_namespace_and_wrapper_independence() {
    let g = golden();
    let m = msg();
    assert_eq!(hex(payload_hash(b"abc")), g["payload_hash"]["hex"]);
    assert_eq!(hex(message_id(&m)), g["message_id"]["hex"]);
    m.validate_hashes().unwrap();
    assert_eq!(
        serde_json::to_string(&Endpoint::Replica(0)).unwrap(),
        g["enum_literals"]["tuple"]
    );
    assert_eq!(
        serde_json::to_string(&OrderMode::Fifo).unwrap(),
        g["enum_literals"]["unit"]
    );
    assert_eq!(
        serde_json::to_string(&RouteCommand::Rebind {
            endpoint: Endpoint::Replica(0),
            expected_incarnation: 1,
            new_incarnation: 2
        })
        .unwrap(),
        g["enum_literals"]["named"]
    );
    let mut e = envelope();
    e.attempt = 4;
    e.source_incarnation = 3;
    e.destination_incarnation = 9;
    assert_eq!(e.message.logical_id, m.logical_id);
    assert_eq!(message_id(&e.message), m.logical_id);
    let mut c = config();
    c.endpoints.clear();
    let permit = ChannelPermit {
        channel: ChannelId(7),
        contexts: vec![[51; 32]],
    };
    for endpoint in (0..15)
        .map(Endpoint::Replica)
        .chain([Endpoint::Service(0), Endpoint::Reader(0)])
    {
        c.endpoints.push(EndpointRegistration {
            binding: EndpointBinding {
                endpoint,
                incarnation: 1,
            },
            send: vec![permit.clone()],
            receive: vec![permit.clone()],
        });
    }
    assert_eq!(c.clone().normalize().unwrap().endpoints.len(), 17);
    RouteKernel::new(c).unwrap();
    assert_ne!(Endpoint::Replica(0), Endpoint::Service(0));
    assert_ne!(Endpoint::Replica(0), Endpoint::Reader(0));
    // Independently calculated from the API's tag-u8/id-u32BE encoding.
    for (source, expected) in [
        (
            Endpoint::Service(0),
            "23d43a55c15bf2fa9cdec69abf15555293d702e69220eb8a92f4b13922711fc1",
        ),
        (
            Endpoint::Reader(0),
            "8f9745558b8b94c38c3bd1a0a65e0b9d7c4e6d668ae7cf9150c41218f0d05dff",
        ),
    ] {
        let mut message = msg();
        message.source = source;
        assert_eq!(hex(message_id(&message)), expected);
    }
}

#[test]
fn kv03_route_single_field_rejection_is_atomic() {
    let k = RouteKernel::new(config()).unwrap();
    let before = k.snapshot_bytes().unwrap();
    let cases: Vec<(Envelope, Error)> = {
        let e = envelope();
        let mut v = Vec::new();
        let mut x = e.clone();
        x.message.run = [99; 32];
        v.push((x, Error::ConfigMismatch));
        let mut x = e.clone();
        x.source_incarnation = 2;
        v.push((x, Error::BindingMismatch));
        let mut x = e.clone();
        x.message.context = Some([99; 32]);
        v.push((x, Error::ContextMismatch));
        let mut x = e.clone();
        x.message.payload.push(9);
        v.push((x, Error::HashMismatch));
        let mut x = e.clone();
        x.message.origin.sent_tick_us = 1_000_001;
        x.message.logical_id = message_id(&x.message);
        v.push((x, Error::InvalidOrigin));
        let mut x = e.clone();
        x.attempt = 0;
        v.push((x, Error::InvalidAttempt));
        v
    };
    for (e, want) in cases {
        error(k.plan(step(1, 1_000_000), &[RouteCommand::Send(e)]), want);
        assert_eq!(before, k.snapshot_bytes().unwrap());
    }
    let mut c = config();
    c.endpoints[0].send.clear();
    let k = RouteKernel::new(c).unwrap();
    error(
        k.plan(step(1, 1_000_000), &[RouteCommand::Send(envelope())]),
        Error::PermissionDenied,
    );
}

#[test]
fn kv04_payload_limits_and_empty_payload() {
    for (length, accepted) in [(0, true), (65535, true), (65536, true), (65537, false)] {
        let mut e = envelope();
        e.message = changed_message(
            e.message,
            Endpoint::Replica(1),
            2,
            1_000_000,
            vec![7; length],
        );
        let k = RouteKernel::new(config()).unwrap();
        let r = k.plan(step(1, 1_000_000), &[RouteCommand::Send(e)]);
        if accepted {
            assert!(r.is_ok());
        } else {
            error(r, Error::PayloadTooLarge);
        }
    }
}

#[test]
fn kv05_normalize_golden_and_duplicates() {
    let c = config();
    assert_eq!(
        hex(c.hash().unwrap()),
        golden()["kernel_config_hash"]["hex"]
    );
    assert_eq!(
        String::from_utf8(c.canonical_bytes().unwrap()).unwrap(),
        golden()["normalized_config_canonical_utf8"]
    );
    let mut reverse = c.clone();
    reverse.endpoints.reverse();
    reverse.channels.reverse();
    reverse.rules.reverse();
    assert_eq!(
        reverse.canonical_bytes().unwrap(),
        c.canonical_bytes().unwrap()
    );
    let mut duplicate = c.clone();
    duplicate.endpoints.push(c.endpoints[0].clone());
    error(duplicate.normalize(), Error::DuplicateId);
}

#[test]
fn kv06_profile_priority_ambiguity_missing_and_context_independent_expansion() {
    let mut c = plain_config();
    let mut exact = c.rules[0].clone();
    exact.source = EndpointSelector::Exact(Endpoint::Replica(0));
    exact.channel = ChannelSelector::Any;
    exact.profile.latency_us = 7;
    c.rules.push(exact);
    let mut kinds = c.rules[0].clone();
    kinds.source = EndpointSelector::Kind(EndpointKind::Replica);
    kinds.destination = EndpointSelector::Kind(EndpointKind::Replica);
    kinds.profile.latency_us = 99;
    c.rules.push(kinds);
    let mut k = RouteKernel::new(c).unwrap();
    let e = route(&mut k, 1, 1000, &[RouteCommand::Send(packet(1, 1000))]);
    assert!(matches!(
        e[0].kind,
        RouteEventKind::Queued {
            raw_due_us: 1007,
            ..
        }
    ));
    let mut c = plain_config();
    let mut r = c.rules[0].clone();
    r.source = EndpointSelector::Exact(Endpoint::Replica(0));
    c.rules.push(r.clone());
    r.source = EndpointSelector::Any;
    r.destination = EndpointSelector::Exact(Endpoint::Replica(1));
    c.rules.push(r);
    error(RouteKernel::new(c), Error::AmbiguousProfile);
    let mut c = plain_config();
    c.rules.clear();
    error(RouteKernel::new(c), Error::MissingProfile);
    // Direction permits expand routes even when their allowed contexts are disjoint.
    let mut c = plain_config();
    c.endpoints[1].receive[0].contexts = vec![[99; 32]];
    let k = RouteKernel::new(c).unwrap();
    error(
        k.plan(step(1, 1000), &[RouteCommand::Send(packet(1, 1000))]),
        Error::ContextMismatch,
    );
}

#[test]
fn kv07_fixed_seed_jitter_literals() {
    for (seed, attempt, want) in [
        (7, 1, 4),
        (7, 4, 301),
        (42, 1, 996),
        (42, 4, 48),
        (99, 1, 436),
        (99, 4, 856),
    ] {
        let mut c = config();
        c.seed = seed;
        let mut k = RouteKernel::new(c).unwrap();
        let mut e = envelope();
        e.attempt = attempt;
        let events = route(&mut k, 1, 1_000_000, &[RouteCommand::Send(e)]);
        assert!(
            matches!(events[0].kind,RouteEventKind::Queued{raw_due_us,..} if raw_due_us==1_020_000+want)
        );
    }
}

#[test]
fn kv08_10_fifo_and_explicit_unordered() {
    for (order, wanted_first, wake) in [(OrderMode::Fifo, 1, 1020), (OrderMode::Unordered, 2, 1002)]
    {
        let mut c = plain_config();
        let p = &mut c.rules[0].profile;
        p.latency_us = 0;
        p.order = order;
        p.faults = vec![
            FaultEntry {
                send_ordinal: 1,
                drop: false,
                duplicate_delay_us: None,
                extra_delay_us: 20,
            },
            FaultEntry {
                send_ordinal: 2,
                drop: false,
                duplicate_delay_us: None,
                extra_delay_us: 1,
            },
        ];
        let mut k = RouteKernel::new(c).unwrap();
        route(&mut k, 1, 1000, &[RouteCommand::Send(packet(1, 1000))]);
        route(&mut k, 2, 1001, &[RouteCommand::Send(packet(2, 1001))]);
        assert_eq!(k.next_wakeup_us(), Some(wake));
        let d = delivered(&route(&mut k, 3, wake, &[RouteCommand::Drain]));
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].packet.packet_seq, wanted_first);
        let next = k.next_wakeup_us().unwrap();
        let d2 = delivered(&route(&mut k, 4, next, &[RouteCommand::Drain]));
        assert_eq!(d2.len(), 1);
        assert_ne!(d2[0].packet.packet_seq, wanted_first);
    }
}

#[test]
fn kv09_cross_link_tie_literals_and_event_ordinals() {
    for (seed, want) in [(7, vec![3, 2, 1]), (42, vec![2, 3, 1]), (99, vec![2, 3, 1])] {
        let mut c = plain_config();
        c.seed = seed;
        let mut k = RouteKernel::new(c).unwrap();
        let commands: Vec<_> = [
            Endpoint::Replica(1),
            Endpoint::Service(6),
            Endpoint::Reader(1),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, dst)| {
            RouteCommand::Send(Envelope {
                message: changed_message(msg(), dst, i as u32, 1000, b"abc".to_vec()),
                ..envelope()
            })
        })
        .collect();
        route(&mut k, 1, 1000, &commands);
        let events = route(&mut k, 2, 1020, &[RouteCommand::Drain]);
        assert_eq!(
            delivered(&events)
                .iter()
                .map(|d| d.packet.packet_seq)
                .collect::<Vec<_>>(),
            want
        );
        assert_eq!(
            events.iter().map(|e| e.effect_index).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
    }
}

#[test]
fn kv11_12_partition_endpoints_and_immediate_drop_coordinates() {
    for (tick, drop) in [(1009, false), (1010, true), (1029, true), (1030, false)] {
        let mut c = plain_config();
        c.rules[0].profile.offline_mode = OfflineMode::Drop;
        c.rules[0].profile.offline = vec![OfflineInterval {
            start_us: 1010,
            end_us: 1030,
        }];
        let mut k = RouteKernel::new(c).unwrap();
        let events = route(&mut k, 1, tick, &[RouteCommand::Send(packet(1, tick))]);
        if drop {
            assert!(matches!(
                events[0].kind,
                RouteEventKind::Dropped {
                    reason: DropReason::Offline,
                    raw_due_us: None,
                    ..
                }
            ));
        } else {
            assert!(matches!(events[0].kind, RouteEventKind::Queued { .. }));
        }
    }
    let mut c = plain_config();
    c.rules[0].profile.faults = vec![FaultEntry {
        send_ordinal: 1,
        drop: true,
        duplicate_delay_us: None,
        extra_delay_us: 0,
    }];
    let mut k = RouteKernel::new(c).unwrap();
    let events = route(&mut k, 1, 1000, &[RouteCommand::Send(packet(1, 1000))]);
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0].kind,
        RouteEventKind::Dropped {
            packet: PacketRef {
                copy_index: 0,
                packet_seq: 1,
                ..
            },
            reason: DropReason::Fault,
            raw_due_us: None
        }
    ));
}

#[test]
fn kv11_late_hold_drain_and_adjacent_intervals() {
    for (mode, expected_end) in [(OfflineMode::Hold, 1300), (OfflineMode::Drop, 0)] {
        let mut c = plain_config();
        c.rules[0].profile.offline_mode = mode;
        c.rules[0].profile.offline = vec![
            OfflineInterval {
                start_us: 1100,
                end_us: 1200,
            },
            OfflineInterval {
                start_us: 1200,
                end_us: 1300,
            },
        ];
        let mut k = RouteKernel::new(c).unwrap();
        route(&mut k, 1, 1000, &[RouteCommand::Send(packet(1, 1000))]);
        let e = route(&mut k, 2, 1150, &[RouteCommand::Drain]);
        if expected_end > 0 {
            assert!(e.is_empty());
            assert_eq!(k.next_wakeup_us(), Some(expected_end));
            let d = delivered(&route(&mut k, 3, 1300, &[RouteCommand::Drain]));
            assert_eq!(d.len(), 1);
            assert_eq!(d[0].raw_due_us, 1020);
            assert_eq!(d[0].delivered_tick_us, 1300);
        } else {
            assert!(matches!(
                e[0].kind,
                RouteEventKind::Dropped {
                    reason: DropReason::Offline,
                    raw_due_us: Some(1020),
                    ..
                }
            ));
            assert_eq!(k.next_wakeup_us(), None);
        }
    }
}

#[test]
fn kv13_late_drain_charges_actual_tick_and_empty_frame() {
    for (payload, spacing) in [(b"abc".to_vec(), 3), (vec![], 1)] {
        let mut k = RouteKernel::new(plain_config()).unwrap();
        let mut first = packet(1, 1000);
        first.message = changed_message(
            first.message,
            Endpoint::Replica(1),
            1,
            1000,
            payload.clone(),
        );
        let mut second = first.clone();
        second.message = changed_message(second.message, Endpoint::Replica(1), 2, 1000, payload);
        route(
            &mut k,
            1,
            1000,
            &[RouteCommand::Send(first), RouteCommand::Send(second)],
        );
        assert_eq!(
            delivered(&route(&mut k, 2, 1100, &[RouteCommand::Drain])).len(),
            1
        );
        assert_eq!(k.next_wakeup_us(), Some(1100 + spacing));
        assert!(route(&mut k, 3, 1100, &[RouteCommand::Drain]).is_empty());
    }
}

#[test]
fn kv14_15_duplicate_overflow_and_budget_classes() {
    let mut c = plain_config();
    c.rules[0].profile.capacity = 1;
    c.rules[0].profile.faults = vec![FaultEntry {
        send_ordinal: 1,
        drop: false,
        duplicate_delay_us: Some(0),
        extra_delay_us: 0,
    }];
    let mut k = RouteKernel::new(c).unwrap();
    let e = route(&mut k, 1, 1000, &[RouteCommand::Send(packet(1, 1000))]);
    assert_eq!(e.len(), 2);
    assert!(matches!(
        e[0].kind,
        RouteEventKind::Queued {
            packet: PacketRef {
                copy_index: 0,
                packet_seq: 1,
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        e[1].kind,
        RouteEventKind::Dropped {
            packet: PacketRef {
                copy_index: 1,
                packet_seq: 2,
                ..
            },
            reason: DropReason::LinkCapacity,
            raw_due_us: Some(1020)
        }
    ));
    for reason in [
        DropReason::LinkCapacity,
        DropReason::GlobalCapacity,
        DropReason::PayloadBudget,
    ] {
        let mut c = plain_config();
        match reason {
            DropReason::LinkCapacity => c.rules[0].profile.capacity = 2,
            DropReason::GlobalCapacity => c.limits.queued_copies = 2,
            DropReason::PayloadBudget => c.limits.queued_payload_bytes = 6,
            _ => unreachable!(),
        };
        let mut k = RouteKernel::new(c).unwrap();
        let e = route(
            &mut k,
            1,
            1000,
            &[
                RouteCommand::Send(packet(1, 1000)),
                RouteCommand::Send(packet(2, 1000)),
                RouteCommand::Send(packet(3, 1000)),
            ],
        );
        assert_eq!(e.len(), 3);
        assert!(
            matches!(&e[2].kind,RouteEventKind::Dropped{reason:r,raw_due_us:Some(1020),..} if *r==reason)
        );
    }
}

#[test]
fn kv16_tracked_history_capacity_survives_queue_drain() {
    let mut c = plain_config();
    c.limits.tracked_messages = 1;
    let mut k = RouteKernel::new(c).unwrap();
    route(&mut k, 1, 1000, &[RouteCommand::Send(packet(1, 1000))]);
    route(&mut k, 2, 1020, &[RouteCommand::Drain]);
    let before = k.snapshot_bytes().unwrap();
    error(
        k.plan(step(3, 1021), &[RouteCommand::Send(packet(2, 1021))]),
        Error::ResourceLimit,
    );
    assert_eq!(before, k.snapshot_bytes().unwrap());
    let mut retry = packet(1, 1000);
    retry.attempt = 4;
    route(&mut k, 3, 1021, &[RouteCommand::Send(retry)]);
}

#[test]
fn kv16_literal_v1_maxima_1024_copies_4mib_8192_ids() {
    let mut c = plain_config();
    let template = c.endpoints[0].clone();
    c.endpoints.push(EndpointRegistration {
        binding: EndpointBinding {
            endpoint: Endpoint::Reader(2),
            incarnation: 1,
        },
        ..template
    });
    let endpoints: Vec<_> = c.endpoints.iter().map(|x| x.binding.endpoint).collect();
    let links: Vec<_> = endpoints
        .iter()
        .flat_map(|src| {
            endpoints
                .iter()
                .filter(move |dst| *dst != src)
                .map(move |dst| (*src, *dst))
        })
        .collect();
    let commands: Vec<_> = (0..1025u32)
        .map(|i| {
            let (source, dst) = links[i as usize / 64];
            let mut m = changed_message(msg(), dst, i, 1000, b"abc".to_vec());
            m.source = source;
            m.logical_id = message_id(&m);
            RouteCommand::Send(Envelope {
                message: m,
                ..envelope()
            })
        })
        .collect();
    let mut copies = RouteKernel::new(c).unwrap();
    let e = route(&mut copies, 1, 1000, &commands);
    assert_eq!(
        e.iter()
            .filter(|e| matches!(e.kind, RouteEventKind::Queued { .. }))
            .count(),
        1024
    );
    assert!(matches!(
        e.last().unwrap().kind,
        RouteEventKind::Dropped {
            reason: DropReason::GlobalCapacity,
            ..
        }
    ));

    let mut bytes = RouteKernel::new(plain_config()).unwrap();
    let commands: Vec<_> = (0..64)
        .map(|i| {
            RouteCommand::Send(Envelope {
                message: changed_message(msg(), Endpoint::Replica(1), i, 1000, vec![255; 65536]),
                ..envelope()
            })
        })
        .collect();
    assert_eq!(route(&mut bytes, 1, 1000, &commands).len(), 64);
    let extra = Envelope {
        message: changed_message(msg(), Endpoint::Service(6), 65, 1000, vec![1]),
        ..envelope()
    };
    let e = route(&mut bytes, 2, 1000, &[RouteCommand::Send(extra)]);
    assert!(matches!(
        e[0].kind,
        RouteEventKind::Dropped {
            reason: DropReason::PayloadBudget,
            ..
        }
    ));

    let mut history = RouteKernel::new(plain_config()).unwrap();
    for batch in 0..2 {
        let commands: Vec<_> = (0..4096)
            .map(|i| RouteCommand::Send(packet(batch * 4096 + i, 1000)))
            .collect();
        route(&mut history, u64::from(batch) + 1, 1000, &commands);
    }
    let before = history.snapshot_bytes().unwrap();
    error(
        history.plan(step(3, 1000), &[RouteCommand::Send(packet(8192, 1000))]),
        Error::ResourceLimit,
    );
    assert_eq!(before, history.snapshot_bytes().unwrap());
    let mut known = packet(0, 1000);
    known.attempt = 4;
    assert!(
        history
            .plan(step(3, 1000), &[RouteCommand::Send(known)])
            .is_ok()
    );
}

#[test]
fn kv17_18_checked_time_stale_binding_and_origin_domain() {
    let mut c = plain_config();
    c.rules[0].profile.latency_us = 1;
    let k = RouteKernel::new(c).unwrap();
    let before = k.snapshot_bytes().unwrap();
    error(
        k.plan(step(1, u64::MAX), &[RouteCommand::Send(packet(1, 1000))]),
        Error::ArithmeticOverflow,
    );
    assert_eq!(before, k.snapshot_bytes().unwrap());
    let mut k = RouteKernel::new(plain_config()).unwrap();
    let mut e = packet(1, 1000);
    e.message.origin.step_id = 999;
    e.message.logical_id = message_id(&e.message);
    route(&mut k, 1, 1000, &[RouteCommand::Send(e)]);
    let events = route(
        &mut k,
        2,
        1020,
        &[
            RouteCommand::Rebind {
                endpoint: Endpoint::Replica(1),
                expected_incarnation: 1,
                new_incarnation: 2,
            },
            RouteCommand::Drain,
        ],
    );
    assert!(matches!(
        events[1].kind,
        RouteEventKind::Dropped {
            reason: DropReason::StaleBinding,
            raw_due_us: Some(1020),
            ..
        }
    ));
    let mut e = packet(2, 1021);
    e.destination_incarnation = 2;
    e.message.origin.source_incarnation = 2;
    e.message.logical_id = message_id(&e.message);
    error(
        k.plan(step(3, 1021), &[RouteCommand::Send(e)]),
        Error::InvalidOrigin,
    );
}

#[test]
fn kv19_23_atomic_plan_stale_cross_instance_noop() {
    let c = plain_config();
    let mut k = RouteKernel::new(c.clone()).unwrap();
    let before = k.snapshot_bytes().unwrap();
    let p = k
        .plan(step(1, 1000), &[RouteCommand::Send(packet(1, 1000))])
        .unwrap();
    assert_eq!(before, k.snapshot_bytes().unwrap());
    assert_eq!(k.revision(), 0);
    assert_eq!(p.base_revision(), 0);
    let stale = k.plan(step(2, 1000), &[]).unwrap();
    k.commit(p).unwrap();
    error(k.commit(stale), Error::StalePlan);
    let before = k.snapshot_bytes().unwrap();
    let mut bad = packet(3, 1000);
    bad.message.payload.push(7);
    error(
        k.plan(
            step(2, 1000),
            &[RouteCommand::Send(packet(2, 1000)), RouteCommand::Send(bad)],
        ),
        Error::HashMismatch,
    );
    assert_eq!(before, k.snapshot_bytes().unwrap());
    for mut foreign in [
        RouteKernel::new(c.clone()).unwrap(),
        k.clone(),
        RouteKernel::from_snapshot(c.clone(), &before).unwrap(),
    ] {
        let p = k.plan(step(2, 1000), &[]).unwrap();
        let original = foreign.snapshot_bytes().unwrap();
        error(foreign.commit(p), Error::CrossInstance);
        assert_eq!(original, foreign.snapshot_bytes().unwrap());
    }
    let p = k.plan(step(2, 1000), &[]).unwrap();
    assert!(k.commit(p).unwrap().is_empty());
    assert_eq!(k.revision(), 2);
    error(
        k.plan(
            StepStamp {
                event_id: 0,
                ..step(3, 1000)
            },
            &[],
        ),
        Error::StepRegression,
    );
    error(k.plan(step(3, 999), &[]), Error::TimeRegression);
}

#[test]
fn kv24_snapshot_mutations_keep_canonical_field_order() {
    let c = plain_config();
    let mut k = RouteKernel::new(c.clone()).unwrap();
    route(&mut k, 1, 1000, &[RouteCommand::Send(packet(1, 1000))]);
    let bytes = k.snapshot_bytes().unwrap();
    let mut spaced = bytes.clone();
    spaced.insert(1, b' ');
    error(
        RouteKernel::from_snapshot(c.clone(), &spaced),
        Error::NonCanonical,
    );
    for changed in [
        replace_once(&bytes, "\"next_packet_seq\":2", "\"next_packet_seq\":1"),
        replace_once(&bytes, "\"packet_seq\":1", "\"packet_seq\":0"),
        replace_once(&bytes, "\"copy_index\":0", "\"copy_index\":2"),
    ] {
        assert!(serde_json::from_slice::<Value>(&changed).is_ok());
        error(
            RouteKernel::from_snapshot(c.clone(), &changed),
            Error::CorruptSnapshot,
        );
    }
    let mut other = c.clone();
    other.seed = 42;
    error(
        RouteKernel::from_snapshot(other, &bytes),
        Error::ConfigMismatch,
    );
    let mut unknown = bytes.clone();
    unknown.splice(1..1, b"\"unknown_field\":0,".iter().copied());
    error(
        RouteKernel::from_snapshot(c.clone(), &unknown),
        Error::CorruptSnapshot,
    );
    let mut two = RouteKernel::new(c.clone()).unwrap();
    route(
        &mut two,
        1,
        1000,
        &[
            RouteCommand::Send(packet(1, 1000)),
            RouteCommand::Send(packet(2, 1000)),
        ],
    );
    let duplicate_seq = replace_once(
        &two.snapshot_bytes().unwrap(),
        "\"packet_seq\":2",
        "\"packet_seq\":1",
    );
    error(
        RouteKernel::from_snapshot(c.clone(), &duplicate_seq),
        Error::CorruptSnapshot,
    );
    error(
        RouteKernel::from_snapshot(c, &vec![b' '; 33_554_433]),
        Error::SnapshotTooLarge,
    );
}

#[test]
fn kv25_command_budget_is_checked_before_mutation() {
    let mut k = RouteKernel::new(plain_config()).unwrap();
    let before = k.snapshot_bytes().unwrap();
    error(
        k.plan(step(1, 1000), &vec![RouteCommand::Drain; 4097]),
        Error::ResourceLimit,
    );
    assert_eq!(before, k.snapshot_bytes().unwrap());
    let p = k
        .plan(step(1, 1000), &vec![RouteCommand::Drain; 4096])
        .unwrap();
    assert!(k.commit(p).unwrap().is_empty());
}

#[test]
fn kv25_event_count_exact_boundary_and_one_more_batch() {
    let mut c = plain_config();
    let template = c.endpoints[0].clone();
    c.endpoints = (0..64)
        .map(|n| EndpointRegistration {
            binding: EndpointBinding {
                endpoint: Endpoint::Replica(n),
                incarnation: 1,
            },
            ..template.clone()
        })
        .collect();
    c.rules[0].profile.faults = (1..=70)
        .map(|n| FaultEntry {
            send_ordinal: n,
            drop: false,
            duplicate_delay_us: Some(0),
            extra_delay_us: 0,
        })
        .collect();
    let packets: Vec<_> = (0..64u32)
        .map(|i| {
            let (src, dst) = if i < 63 {
                (Endpoint::Replica(0), Endpoint::Replica(i + 1))
            } else {
                (Endpoint::Replica(1), Endpoint::Replica(0))
            };
            let mut m = changed_message(msg(), dst, i, 1000, b"abc".to_vec());
            m.source = src;
            m.logical_id = message_id(&m);
            Envelope {
                message: m,
                ..envelope()
            }
        })
        .collect();
    let mut exact = RouteKernel::new(c.clone()).unwrap();
    let commands: Vec<_> = (0..4096)
        .map(|i| {
            let mut e = packets[i % 64].clone();
            e.attempt = (i / 64 + 1) as u32;
            RouteCommand::Send(e)
        })
        .collect();
    let events = route(&mut exact, 1, 1000, &commands);
    assert_eq!(events.len(), 8192);
    let mut excess = RouteKernel::new(c).unwrap();
    route(
        &mut excess,
        1,
        1000,
        &packets
            .iter()
            .cloned()
            .map(RouteCommand::Send)
            .collect::<Vec<_>>(),
    );
    let before = excess.snapshot_bytes().unwrap();
    let mut commands = vec![RouteCommand::Drain];
    commands.extend((0..4095).map(|i| {
        let mut e = packets[i % 64].clone();
        e.attempt = (i / 64 + 2) as u32;
        RouteCommand::Send(e)
    }));
    // 64 due deliveries plus 4095*2 queued/drop outcomes exceed8192.
    error(excess.plan(step(2, 1020), &commands), Error::ResourceLimit);
    assert_eq!(before, excess.snapshot_bytes().unwrap());
}

#[test]
fn snapshot_raw_due_respects_fixed_profile_and_last_committed_tick() {
    let c = plain_config();
    let mut k = RouteKernel::new(c.clone()).unwrap();
    route(&mut k, 1, 1000, &[RouteCommand::Send(packet(1, 1000))]);
    let original = k.snapshot_bytes().unwrap();
    let changed = replace_once(
        &original,
        "\"raw_due_us\":1020",
        "\"raw_due_us\":18446744073709551615",
    );
    save_evidence("route-valid-snapshot.json", &original);
    save_evidence("route-impossible-due-snapshot.json", &changed);
    save_evidence("route-mutant-config.json", &c.canonical_bytes().unwrap());
    let result = RouteKernel::from_snapshot(c, &changed);
    if let Ok(restored) = &result {
        eprintln!(
            "MALFORMED_ROUTE_ACCEPTED next_wakeup={:?}",
            restored.next_wakeup_us()
        );
    }
    // A Send no later than last_step.tick=1000, with latency20/jitter0/no
    // fault delay, cannot produce raw_due beyond1020. No full history needed.
    error(result, Error::CorruptSnapshot);
}

#[test]
fn kv25_encoded_event_budget_is_atomic() {
    let mut c = plain_config();
    let template = c.endpoints[0].clone();
    c.endpoints = (0..64)
        .map(|n| EndpointRegistration {
            binding: EndpointBinding {
                endpoint: Endpoint::Replica(n),
                incarnation: 1,
            },
            ..template.clone()
        })
        .collect();
    c.rules[0].profile.latency_us = 0;
    let k = RouteKernel::new(c).unwrap();
    let before = k.snapshot_bytes().unwrap();
    let mut commands = Vec::new();
    // 150 distinct links each deliver a 64KiB byte array; >32MiB canonical events.
    for i in 0..150u32 {
        let source = Endpoint::Replica(i / 63);
        let target0 = i % 63;
        let target = Endpoint::Replica(if target0 >= i / 63 {
            target0 + 1
        } else {
            target0
        });
        let mut m = changed_message(msg(), target, i, 1000, vec![255; 65536]);
        m.source = source;
        m.logical_id = message_id(&m);
        commands.push(RouteCommand::Send(Envelope {
            message: m,
            ..envelope()
        }));
        commands.push(RouteCommand::Drain);
    }
    error(k.plan(step(1, 1000), &commands), Error::ResourceLimit);
    assert_eq!(before, k.snapshot_bytes().unwrap());
}

#[test]
fn kv45_twelve_component_replay_pairs() {
    for seed in [7, 42, 99] {
        let mut traces = Vec::new();
        for reverse in [false, true] {
            for pause in [0, 2] {
                let mut c = plain_config();
                c.seed = seed;
                if reverse {
                    c.endpoints.reverse();
                    c.channels.reverse();
                    c.rules.reverse();
                }
                let mut k = RouteKernel::new(c).unwrap();
                let mut trace = route(&mut k, 1, 1000, &[RouteCommand::Send(packet(1, 1000))]);
                std::thread::sleep(std::time::Duration::from_millis(pause));
                trace.extend(route(&mut k, 2, 1020, &[RouteCommand::Drain]));
                let bytes = serde_json::to_vec(&(trace, k.snapshot_bytes().unwrap())).unwrap();
                save_evidence(
                    &format!("pure-replay-seed{seed}-reverse{reverse}-pause{pause}.json"),
                    &bytes,
                );
                traces.push(bytes);
            }
        }
        assert!(traces.windows(2).all(|p| p[0] == p[1]));
    }
}
