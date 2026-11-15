//! External conformance assertions derived before reading the implementation.
//! Mock storage receipts exercise typed boundaries; they do not prove fsync.
use serde::de::DeserializeOwned;
use serde_json::Value;
use sim_transport::*;

mod custody_clock_cases;
mod route_cases;

fn save_evidence(name: &str, bytes: &[u8]) {
    if let Ok(directory) = std::env::var("SIMT_VERIFY_ARTIFACTS") {
        let directory = std::path::Path::new(&directory);
        std::fs::create_dir_all(directory).unwrap();
        std::fs::write(directory.join(name), bytes).unwrap();
    }
}

fn golden() -> &'static Value {
    static GOLDEN: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    GOLDEN.get_or_init(|| serde_json::from_str(include_str!("golden-vectors.json")).unwrap())
}
fn parse<T: DeserializeOwned>(value: &Value) -> T {
    serde_json::from_value(value.clone()).unwrap()
}
fn config() -> TransportConfig {
    parse(&golden()["normalized_transport_config"])
}
fn msg() -> Message {
    parse(&golden()["message"])
}
fn envelope() -> Envelope {
    parse(&golden()["envelope_attempt1"])
}
fn hex(id: Id) -> String {
    id.iter().map(|b| format!("{b:02x}")).collect()
}
fn id(s: &str) -> Id {
    assert_eq!(s.len(), 64);
    std::array::from_fn(|i| u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap())
}
fn error<T>(result: Result<T, Error>, expected: Error) {
    assert_eq!(result.err(), Some(expected));
}
fn step(n: u64, tick: u64) -> StepStamp {
    StepStamp {
        logical_step_id: n,
        tick_us: tick,
        event_id: 100_000 - n,
        causal_wave: 0,
    }
}
fn route(k: &mut RouteKernel, n: u64, tick: u64, commands: &[RouteCommand]) -> Vec<RouteEvent> {
    let p = k.plan(step(n, tick), commands).unwrap();
    k.commit(p).unwrap()
}
fn changed_message(
    mut m: Message,
    dest: Endpoint,
    origin_index: u32,
    tick: u64,
    payload: Vec<u8>,
) -> Message {
    m.destination = dest;
    m.origin.effect_index = origin_index;
    m.origin.sent_tick_us = tick;
    m.payload = payload;
    m.payload_hash = payload_hash(&m.payload);
    m.logical_id = message_id(&m);
    m
}
fn packet(index: u32, tick: u64) -> Envelope {
    Envelope {
        message: changed_message(msg(), Endpoint::Replica(1), index, tick, b"abc".to_vec()),
        ..envelope()
    }
}
fn plain_config() -> TransportConfig {
    let mut c = config();
    c.rules[0].profile.jitter_us = 0;
    c.rules[0].profile.latency_us = 20;
    c
}
fn delivered(events: &[RouteEvent]) -> Vec<Delivery> {
    events
        .iter()
        .filter_map(|e| match &e.kind {
            RouteEventKind::Delivered(d) => Some(d.clone()),
            _ => None,
        })
        .collect()
}
fn receipt(p: &CustodyPlan, storage: u8) -> DurabilityReceipt {
    DurabilityReceipt {
        owner: p.owner(),
        base_revision: p.base_revision(),
        transition_id: p.transition_id(),
        candidate_record_hash: p.candidate_record_hash(),
        storage_receipt: [storage; 32],
    }
}
fn confirm(m: &mut ReliableMailbox, a: MailboxAction) -> CommittedCustody {
    let p = m.plan(a).unwrap();
    let r = receipt(&p, 0x55);
    m.confirm(p, r).unwrap()
}
fn mailbox(owner: Endpoint) -> ReliableMailbox {
    ReliableMailbox::new(config(), owner, ReliableLimits::default(), 20_000).unwrap()
}
fn action(name: &str) -> MailboxAction {
    parse(&golden()["custody_records"][name]["action"])
}
fn clock_identity(epsilon: i64) -> ClockIdentity {
    ClockIdentity {
        run: [17; 32],
        config: [34; 32],
        endpoint: Endpoint::Replica(0),
        incarnation: 1,
        domain: [85; 32],
        epsilon_us: epsilon,
    }
}
fn sample(
    identity: &ClockIdentity,
    request: u64,
    sequence: u64,
    asof: u64,
    end: u64,
) -> ClockSample {
    ClockSample {
        run: identity.run,
        config: identity.config,
        endpoint: identity.endpoint,
        incarnation: identity.incarnation,
        domain: identity.domain,
        rate_profile: id(golden()["rate_profile"]["hex"].as_str().unwrap()),
        request_id: request,
        sequence,
        as_of_tick_us: asof,
        local_at_as_of_us: (i128::from(asof) + i128::from(identity.epsilon_us))
            .try_into()
            .unwrap(),
        uncertainty_us: identity.epsilon_us.unsigned_abs(),
        lease_end_tick_us: end,
    }
}
fn replace_once(bytes: &[u8], from: &str, to: &str) -> Vec<u8> {
    let s = std::str::from_utf8(bytes).unwrap();
    assert_eq!(
        s.matches(from).count(),
        1,
        "mutation must target exactly one coordinate: {from}"
    );
    s.replacen(from, to, 1).into_bytes()
}
