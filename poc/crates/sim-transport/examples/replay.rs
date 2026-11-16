use serde::Serialize;
use serde_json::{Value, json};
use sim_transport::*;
use std::{
    env,
    error::Error as StdError,
    fs,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

const SENDER: Endpoint = Endpoint::Service(6);
const READER_A: Endpoint = Endpoint::Reader(1);
const READER_B: Endpoint = Endpoint::Reader(2);
const CHANNEL: ChannelId = ChannelId(7);
const START: Tick = 1_000_000;

struct Args {
    seed: u64,
    reverse: bool,
    wall_pause_ms: u64,
    output: PathBuf,
}

impl Args {
    fn parse() -> Result<Self, Box<dyn StdError>> {
        let mut seed = None;
        let mut reverse = None;
        let mut wall_pause_ms = None;
        let mut output = None;
        let mut args = env::args().skip(1);
        while let Some(option) = args.next() {
            let value = args.next().ok_or("missing option value")?;
            match option.as_str() {
                "--seed" if seed.is_none() => seed = Some(value.parse()?),
                "--declaration-order" if reverse.is_none() => {
                    reverse = Some(match value.as_str() {
                        "forward" => false,
                        "reverse" => true,
                        _ => return Err("declaration order must be forward or reverse".into()),
                    });
                }
                "--wall-pause-ms" if wall_pause_ms.is_none() => {
                    let pause = value.parse()?;
                    if ![0, 2].contains(&pause) {
                        return Err("wall pause must be 0 or 2 milliseconds".into());
                    }
                    wall_pause_ms = Some(pause);
                }
                "--output" if output.is_none() => output = Some(PathBuf::from(value)),
                _ => return Err(format!("unknown or repeated option {option}").into()),
            }
        }
        Ok(Self {
            seed: seed.ok_or("--seed is required")?,
            reverse: reverse.ok_or("--declaration-order is required")?,
            wall_pause_ms: wall_pause_ms.ok_or("--wall-pause-ms is required")?,
            output: output.ok_or("--output is required")?,
        })
    }
}

#[derive(Serialize)]
struct Trace {
    schema: u32,
    scenario: &'static str,
    seed: u64,
    kernel_config_hash: Id,
    normalized_config: Value,
    entries: Vec<Value>,
}

impl Trace {
    fn push(&mut self, name: &str, data: impl Serialize) -> Result<(), serde_json::Error> {
        self.entries
            .push(json!({"name": name, "data": serde_json::to_value(data)?}));
        Ok(())
    }
}

fn binding(endpoint: Endpoint) -> EndpointBinding {
    EndpointBinding {
        endpoint,
        incarnation: 1,
    }
}

fn config(args: &Args) -> TransportConfig {
    let permit = ChannelPermit {
        channel: CHANNEL,
        contexts: vec![],
    };
    let mut config = TransportConfig {
        schema: 1,
        run: [1; 32],
        config: [2; 32],
        seed: args.seed,
        limits: Limits::default(),
        endpoints: vec![
            EndpointRegistration {
                binding: binding(SENDER),
                send: vec![permit.clone()],
                receive: vec![],
            },
            EndpointRegistration {
                binding: binding(READER_A),
                send: vec![],
                receive: vec![permit.clone()],
            },
            EndpointRegistration {
                binding: binding(READER_B),
                send: vec![],
                receive: vec![permit],
            },
        ],
        channels: vec![ChannelSpec {
            id: CHANNEL,
            context: ContextPolicy::Forbidden,
            reliability: Reliability::Reliable,
        }],
        rules: vec![
            LinkRule {
                source: EndpointSelector::Any,
                destination: EndpointSelector::Any,
                channel: ChannelSelector::Any,
                profile: LinkProfile::default(),
            },
            LinkRule {
                source: EndpointSelector::Exact(SENDER),
                destination: EndpointSelector::Exact(READER_A),
                channel: ChannelSelector::Exact(CHANNEL),
                profile: LinkProfile {
                    latency_us: 500,
                    jitter_us: 200,
                    offline: vec![OfflineInterval {
                        start_us: START + 20_000,
                        end_us: START + 30_000,
                    }],
                    faults: vec![
                        FaultEntry {
                            send_ordinal: 1,
                            drop: true,
                            duplicate_delay_us: None,
                            extra_delay_us: 0,
                        },
                        FaultEntry {
                            send_ordinal: 2,
                            drop: false,
                            duplicate_delay_us: Some(0),
                            extra_delay_us: 25,
                        },
                    ],
                    ..LinkProfile::default()
                },
            },
        ],
    };
    if args.reverse {
        config.endpoints.reverse();
        config.rules.reverse();
        config.rules[0].profile.faults.reverse();
    }
    config
}

struct Service {
    output: Message,
}

impl GrantConsumer for Service {
    type Error = &'static str;

    fn on_grant(&mut self, grant: ServiceGrant) -> Result<Vec<ServiceEffect>, Self::Error> {
        if grant.target != binding(SENDER)
            || !matches!(grant.input, ServiceInput::External { kind: 1, .. })
        {
            return Err("unexpected service grant");
        }
        Ok(vec![ServiceEffect::Send(self.output.clone())])
    }
}

struct Reader {
    endpoint: Endpoint,
    receipts: Vec<Id>,
}

impl GrantConsumer for Reader {
    type Error = &'static str;

    fn on_grant(&mut self, grant: ServiceGrant) -> Result<Vec<ServiceEffect>, Self::Error> {
        let ServiceInput::Message(delivery) = grant.input else {
            return Err("unexpected reader grant");
        };
        if grant.target != binding(self.endpoint)
            || delivery.envelope.message.destination != self.endpoint
        {
            return Err("delivery targets a different reader");
        }
        self.receipts.push(delivery.envelope.message.logical_id);
        Ok(vec![])
    }
}

fn confirm_mock(
    mailbox: &mut ReliableMailbox,
    action: MailboxAction,
    trace: &mut Trace,
) -> Result<CommittedCustody, Box<dyn StdError>> {
    let plan = mailbox.plan(action)?;
    let record = std::str::from_utf8(plan.persistence_bytes())?;
    let receipt = DurabilityReceipt {
        owner: plan.owner(),
        base_revision: plan.base_revision(),
        transition_id: plan.transition_id(),
        candidate_record_hash: plan.candidate_record_hash(),
        storage_receipt: framed_hash(
            "SIMT/replay-mock-storage/v1",
            &[&plan.candidate_record_hash()],
        ),
    };
    trace.push(
        "mock_persistence",
        json!({ "record": record, "receipt": receipt }),
    )?;
    let committed = mailbox.confirm(plan, receipt)?;
    trace.push("custody_committed", &committed)?;
    Ok(committed)
}

fn make_clock(
    config: &TransportConfig,
    endpoint: Endpoint,
    trace: &mut Trace,
) -> Result<ClockTracker, Box<dyn StdError>> {
    let domain = [5; 32];
    let identity = ClockIdentity {
        run: config.run,
        config: config.config,
        endpoint,
        incarnation: 1,
        domain,
        epsilon_us: 0,
    };
    let mut tracker = ClockTracker::new(identity.clone())?;
    let request_id = tracker.request()?;
    trace.push(
        "clock_request",
        json!({"identity": identity, "request_id": request_id, "reference_tick_us": START - 500}),
    )?;
    let sample = ClockSample {
        run: config.run,
        config: config.config,
        endpoint,
        incarnation: 1,
        domain,
        rate_profile: framed_hash("SIMT/rate/v1", &[b"fixed-rate-one-v1"]),
        request_id,
        sequence: 1,
        as_of_tick_us: START - 250,
        local_at_as_of_us: START as i64 - 250,
        uncertainty_us: 0,
        lease_end_tick_us: START + 100_000,
    };
    trace.push("clock_sample", &sample)?;
    tracker.install(START, sample)?;
    trace.push("clock_delivery", tracker.view(START)?)?;
    Ok(tracker)
}

fn route(
    kernel: &mut RouteKernel,
    trace: &mut Trace,
    step: StepStamp,
    commands: &[RouteCommand],
    pause: u64,
) -> Result<Vec<RouteEvent>, Box<dyn StdError>> {
    if pause != 0 {
        thread::sleep(Duration::from_millis(pause));
    }
    trace.push("route_input", json!({ "step": step, "commands": commands }))?;
    let result = kernel.commit(kernel.plan(step, commands)?)?;
    trace.push("route_committed", &result)?;
    Ok(result)
}

fn attempt(
    sender: &mut ReliableMailbox,
    id: Id,
    now_us: Tick,
    trace: &mut Trace,
) -> Result<Envelope, Box<dyn StdError>> {
    let committed = confirm_mock(
        sender,
        MailboxAction::Attempt {
            logical_id: id,
            now_us,
            destination: binding(READER_A),
        },
        trace,
    )?;
    let [CustodyRelease::Send(envelope)] = committed.releases.as_slice() else {
        return Err("attempt did not produce one send".into());
    };
    Ok(envelope.clone())
}

fn replay(args: &Args) -> Result<Trace, Box<dyn StdError>> {
    let config = config(args);
    let mut kernel = RouteKernel::new(config.clone())?;
    let mut trace = Trace {
        schema: 1,
        scenario: "reliable-duplicate-after-hold",
        seed: args.seed,
        kernel_config_hash: kernel.config_hash(),
        normalized_config: serde_json::from_slice(&config.canonical_bytes()?)?,
        entries: Vec::new(),
    };
    let sender_clock = make_clock(&config, SENDER, &mut trace)?;
    let reader_clock = make_clock(&config, READER_A, &mut trace)?;

    let mut sender =
        ReliableMailbox::new(config.clone(), SENDER, ReliableLimits::default(), 20_000)?;
    let mut receiver = ReliableMailbox::new(config, READER_A, ReliableLimits::default(), 20_000)?;
    let mut message = Message {
        run: [1; 32],
        config: [2; 32],
        source: SENDER,
        destination: READER_A,
        channel: CHANNEL,
        context: None,
        codec: [4; 32],
        origin: Origin {
            step_id: 1,
            effect_index: 0,
            sent_tick_us: START,
            source_incarnation: 1,
        },
        payload: b"abc".to_vec(),
        payload_hash: payload_hash(b"abc"),
        logical_id: [0; 32],
    };
    message.logical_id = message_id(&message);
    let id = message.logical_id;
    let mut service = Service { output: message };
    let mut a = Reader {
        endpoint: READER_A,
        receipts: vec![],
    };
    let b = Reader {
        endpoint: READER_B,
        receipts: vec![],
    };

    let service_grant = ServiceGrant {
        target: binding(SENDER),
        step: StepStamp {
            logical_step_id: 1,
            tick_us: START,
            event_id: 5,
            causal_wave: 0,
        },
        clock: sender_clock.view(START)?,
        input: ServiceInput::External {
            kind: 1,
            bytes: b"abc".to_vec(),
        },
    };
    trace.push("service_grant", &service_grant)?;
    let effects = service.on_grant(service_grant)?;
    trace.push("service_intents", &effects)?;
    let [ServiceEffect::Send(message)] = effects.as_slice() else {
        return Err("service did not produce one send intent".into());
    };
    confirm_mock(
        &mut sender,
        MailboxAction::Enqueue {
            message: message.clone(),
            now_us: START,
        },
        &mut trace,
    )?;

    let mut step_id = 1;
    for (tick_us, event_id) in [(START, 4), (START + 20_000, 3)] {
        let envelope = attempt(&mut sender, id, tick_us, &mut trace)?;
        step_id += 1;
        let step = StepStamp {
            logical_step_id: step_id,
            tick_us,
            event_id,
            causal_wave: 1,
        };
        route(
            &mut kernel,
            &mut trace,
            step,
            &[RouteCommand::Send(envelope)],
            args.wall_pause_ms,
        )?;
    }
    while let Some(tick_us) = kernel.next_wakeup_us() {
        step_id += 1;
        let step = StepStamp {
            logical_step_id: step_id,
            tick_us,
            event_id: step_id + 8,
            causal_wave: 0,
        };
        let events = route(
            &mut kernel,
            &mut trace,
            step,
            &[RouteCommand::Drain],
            args.wall_pause_ms,
        )?;
        for event in events {
            let RouteEventKind::Delivered(delivery) = event.kind else {
                continue;
            };
            let clock = reader_clock.view(tick_us)?;
            let committed = confirm_mock(
                &mut receiver,
                MailboxAction::Receive {
                    envelope: delivery.envelope.clone(),
                    received_tick_us: tick_us,
                    received_local_us: Some(clock.local_time_us()),
                    authenticated_source: binding(SENDER),
                },
                &mut trace,
            )?;
            for release in committed.releases {
                match release {
                    CustodyRelease::InboxAvailable { logical_id } => {
                        step_id += 1;
                        let grant = ServiceGrant {
                            target: binding(READER_A),
                            step: StepStamp {
                                logical_step_id: step_id,
                                tick_us,
                                event_id: step_id + 8,
                                causal_wave: 1,
                            },
                            clock: clock.clone(),
                            input: ServiceInput::Message(delivery.clone()),
                        };
                        trace.push("reader_grant", &grant)?;
                        trace.push("reader_intents", a.on_grant(grant)?)?;
                        confirm_mock(
                            &mut receiver,
                            MailboxAction::MarkConsumed { logical_id },
                            &mut trace,
                        )?;
                    }
                    CustodyRelease::ReceiveAck(ack) => {
                        confirm_mock(
                            &mut sender,
                            MailboxAction::Acknowledge {
                                ack,
                                authenticated_receiver: binding(READER_A),
                            },
                            &mut trace,
                        )?;
                    }
                    _ => return Err("unexpected receiver release".into()),
                }
            }
        }
    }
    trace.push("final_observations", json!({
        "reader_a_logical_receipts": a.receipts, "reader_b_logical_receipts": b.receipts,
        "receiver_pending": receiver.pending_inbox(), "sender_retry_due_us": sender.retry_due_us(id),
        "route_state": serde_json::from_slice::<Value>(&kernel.snapshot_bytes()?)?,
    }))?;
    if a.receipts != [id]
        || !b.receipts.is_empty()
        || !receiver.pending_inbox().is_empty()
        || sender.retry_due_us(id).is_some()
    {
        return Err("unexpected final custody or reader state".into());
    }
    Ok(trace)
}

fn main() -> Result<(), Box<dyn StdError>> {
    let args = Args::parse()?;
    fs::create_dir(&args.output)?;
    let started = Instant::now();
    let trace = replay(&args)?;
    let logical = serde_json::to_vec(&trace)?;
    let diagnostics = json!({
        "declaration_order": if args.reverse { "reverse" } else { "forward" },
        "wall_pause_ms": args.wall_pause_ms,
        "wall_elapsed_us": started.elapsed().as_micros(),
        "active_native_engines": 0,
        "native_worker_processes": 0,
        "storage_mode": "mock_receipt",
    });
    fs::write(args.output.join("logical-trace.json"), logical)?;
    fs::write(
        args.output.join("diagnostics.json"),
        serde_json::to_vec(&diagnostics)?,
    )?;
    Ok(())
}
