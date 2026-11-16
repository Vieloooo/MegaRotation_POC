use super::*;
use sim_transport::{
    ChannelId, ClockIdentity, ClockSample, ClockTracker, Delivery, Endpoint, EndpointBinding,
    Envelope, Message, Origin, PacketRef, ServiceGrant, ServiceInput, StepStamp, framed_hash,
    message_id, payload_hash,
};

fn route() -> ReassemblyRoute {
    ReassemblyRoute {
        run: RUN,
        config: CONFIG,
        source: EndpointBinding {
            endpoint: Endpoint::Service(9),
            incarnation: 1,
        },
        destination: EndpointBinding {
            endpoint: Endpoint::Reader(1),
            incarnation: 1,
        },
        channel: ChannelId(3),
        context: Some([4; 32]),
        codec: [5; 32],
        payload_limit: 65_536,
    }
}

fn controlled_fixture(route: &ReassemblyRoute, frame: Vec<u8>, i: u32) -> Delivery {
    let mut message = Message {
        run: route.run,
        config: route.config,
        source: route.source.endpoint,
        destination: route.destination.endpoint,
        channel: route.channel,
        context: route.context,
        codec: route.codec,
        origin: Origin {
            step_id: 7,
            effect_index: i,
            sent_tick_us: 1,
            source_incarnation: route.source.incarnation,
        },
        payload_hash: payload_hash(&frame),
        logical_id: [0; 32],
        payload: frame,
    };
    message.logical_id = message_id(&message);
    Delivery {
        packet: PacketRef {
            logical_id: message.logical_id,
            attempt: 1,
            copy_index: 0,
            packet_seq: i as u64,
        },
        envelope: Envelope {
            message,
            attempt: 1,
            source_incarnation: route.source.incarnation,
            destination_incarnation: route.destination.incarnation,
        },
        raw_due_us: 2,
        eligible_tick_us: 3,
        delivered_tick_us: 3,
    }
}

struct KnownDelivery(Vec<Delivery>);
impl DeliveryVerifier for KnownDelivery {
    fn verify_delivery(&self, item: &Delivery) -> Result<(), BridgeError> {
        if self.0.contains(item) {
            Ok(())
        } else {
            Err(BridgeError::SourceMismatch)
        }
    }
}

#[test]
fn literal_bindings_and_frame_bounds() {
    let message = sim_espresso::LogMessage {
        codec: 0x47465201,
        bytes: b"abc".to_vec(),
    };
    let payload = attempt_payload_id(b"E", &[message.clone()]).unwrap();
    assert_eq!(
        poc_types::hex(payload.as_bytes()),
        "4d0c1b07e6b5f149b1cb4b56f5a1026b6c4f5a08a839125ec94ddd7057ac0e7a"
    );
    let attempt = poc_publisher::AttemptId {
        namespace: b"P".to_vec(),
        task_number: 1,
        attempt_number: 2,
    };
    let key = ResultKey {
        cid: Cid::new(Hash32::new([0x33; 32])),
        block_hash: Hash32::new([0x44; 32]),
    };
    let binding =
        submission_binding_id(RUN, CONFIG, b"E", &attempt, key, payload, 0, message.codec).unwrap();
    assert_eq!(
        poc_types::hex(binding.as_bytes()),
        "f7ae0c2f6af604cb1fab9791814380435866804cc0d56eb40325e67ed5ea4e14"
    );
    let frame = fragment(0, binding, b"abc", 65_536).unwrap();
    assert_eq!(
        poc_types::hex(&frame[0][39..71]),
        "d06849efe5c03e9b648a4dc16324a49b06b56e55c72b82d756d16980676b334c"
    );
    let id = finalized_block_binding_id(RUN, CONFIG, b"E", 5).unwrap();
    assert_eq!(
        poc_types::hex(id.as_bytes()),
        "8568b9f102344973e7d3b4fe649003eff7c888066654c3e7757885bd29c5915f"
    );
    assert!(fragment(1, id, &[], 65_536).is_err());
    assert!(fragment(1, id, b"a", 83).is_err());
    assert_eq!(
        fragment(1, id, &vec![3; 1024 * 1024], 65_536)
            .unwrap()
            .iter()
            .map(Vec::len)
            .sum::<usize>(),
        1_050_070
    );
}

#[test]
fn authorized_reverse_delivery_and_verified_credit_release() {
    let route = route();
    let mut log =
        SimEspresso::new(SOURCE.to_vec(), TimestampPolicy::Fixed { theta_us: 0 }).unwrap();
    let message = sim_espresso::LogMessage {
        codec: 1,
        bytes: vec![7; 65_536],
    };
    log.propose(TickUs(0), false, vec![message]).unwrap();
    let wire = encode_finalized(SOURCE, log.finalize(0, TickUs(0)).unwrap()).unwrap();
    let id = finalized_block_binding_id(RUN, CONFIG, SOURCE, 0).unwrap();
    let controls: Vec<_> = fragment(1, id, &wire, route.payload_limit)
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(i, frame)| controlled_fixture(&route, frame, i as u32))
        .collect();
    let verified = KnownDelivery(controls.clone());
    let mut reassembler = Reassembler::new(vec![route.clone()], &verified).unwrap();
    reassembler.register_finalized(&route, SOURCE, 0).unwrap();
    assert_eq!(reassembler.receive(&controls[1]).unwrap(), None);
    assert_eq!(reassembler.receive(&controls[1]).unwrap(), None);
    let mut unauthorized = controls[0].clone();
    unauthorized.packet.packet_seq += 1;
    assert!(reassembler.receive(&unauthorized).is_err());
    assert_eq!(
        reassembler.receive(&controls[0]).unwrap().unwrap().bytes,
        wire
    );
    assert_eq!(reassembler.receive(&controls[0]).unwrap(), None);

    let service = route.source;
    let domain = [6; 32];
    let mut clock = ClockTracker::new(ClockIdentity {
        run: RUN,
        config: CONFIG,
        endpoint: service.endpoint,
        incarnation: service.incarnation,
        domain,
        epsilon_us: 0,
    })
    .unwrap();
    let request_id = clock.request().unwrap();
    clock
        .install(
            1,
            ClockSample {
                run: RUN,
                config: CONFIG,
                endpoint: service.endpoint,
                incarnation: service.incarnation,
                domain,
                rate_profile: framed_hash("SIMT/rate/v1", &[b"fixed-rate-one-v1"]),
                request_id,
                sequence: 1,
                as_of_tick_us: 1,
                local_at_as_of_us: 1,
                uncertainty_us: 0,
                lease_end_tick_us: 2,
            },
        )
        .unwrap();
    let grant = ServiceGrant {
        target: service,
        clock: clock.view(1).unwrap(),
        input: ServiceInput::External {
            kind: 0,
            bytes: vec![],
        },
        step: StepStamp {
            logical_step_id: 7,
            tick_us: 1,
            event_id: 7,
            causal_wave: 0,
        },
    };
    let messages = controls
        .iter()
        .map(|row| row.envelope.message.clone())
        .collect::<Vec<_>>();
    let mut window = CreditWindow::new();
    let port = SyntheticCredit {
        shared_available: false,
        plan: None,
        release: messages[0].logical_id,
    };
    assert!(window.plan(&grant, messages.clone(), &port).is_err());
    let mut port = SyntheticCredit {
        shared_available: true,
        ..port
    };
    let plan = window.plan(&grant, messages.clone(), &port).unwrap();
    assert_eq!(window.in_flight(), (0, 0));
    assert_eq!(plan.effects().len(), 2);
    port.plan = Some(plan.candidate_hash());
    window.confirm(plan, RECEIPT, &port).unwrap();
    let byte_count = messages.iter().map(|m| m.payload.len()).sum::<usize>();
    assert_eq!(window.in_flight(), (2, byte_count));
    assert!(window.release(port.release, b"fake ack", &port).is_err());
    assert_eq!(window.in_flight(), (2, byte_count));
    window.release(port.release, RECEIPT, &port).unwrap();
    assert_eq!(window.in_flight(), (1, messages[1].payload.len()));
}

struct SyntheticCredit {
    shared_available: bool,
    plan: Option<Hash32>,
    release: Id,
}
impl CreditVerifier for SyntheticCredit {
    fn verify_available(&self, _: &ServiceGrant, _: &[Message]) -> Result<(), BridgeError> {
        if self.shared_available {
            Ok(())
        } else {
            Err(BridgeError::Capacity)
        }
    }
    fn verify_charge(&self, hash: Hash32, receipt: &[u8]) -> Result<(), BridgeError> {
        if self.plan == Some(hash) && receipt == RECEIPT {
            Ok(())
        } else {
            Err(BridgeError::SourceMismatch)
        }
    }
    fn verify_release(&self, id: Id, receipt: &[u8]) -> Result<(), BridgeError> {
        if self.release == id && receipt == RECEIPT {
            Ok(())
        } else {
            Err(BridgeError::SourceMismatch)
        }
    }
}
