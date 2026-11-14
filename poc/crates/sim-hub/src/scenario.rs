use poc_types::{input::Kind as I, *};
use serde::{Deserialize, Serialize};
use sim_runtime::{Checkpoint, Result, invalid, require};

pub const START: u64 = 1_000_000;
pub const NAMES: &[&str] = &[
    "healthy",
    "clock-faults",
    "lease-expiry",
    "timers",
    "close-work",
    "restart-before-ack",
    "restart-after-expiry",
    "network-faults",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub seed: u64,
    pub start_tick_us: u64,
    pub epsilon_us: [i64; 2],
    pub clock: Clock,
    pub links: Vec<Link>,
    pub watchdog_ms: u64,
    pub actions: Vec<Action>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clock {
    pub default_request_us: u64,
    pub default_response_us: u64,
    pub lease_us: u64,
    pub refresh_us: Option<u64>,
    pub refresh_until_tick_us: Option<u64>,
    pub overrides: Vec<ClockOverride>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClockOverride {
    pub node: u32,
    pub request_id: u64,
    pub request_us: u64,
    pub response_us: u64,
    pub response: SampleReply,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SampleReply {
    Valid,
    Lost,
    InvalidThenValid,
    Replay,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Link {
    pub channel: u32,
    pub delay_us: u64,
    pub jitter_us: u64,
    pub capacity: usize,
    pub offline: Vec<[u64; 2]>,
    pub duplicate_delay_us: Option<u64>,
    pub retry_delay_us: Option<u64>,
    pub overflow: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    pub tick_us: u64,
    pub node: u32,
    #[serde(flatten)]
    pub kind: ActionKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action", content = "value")]
pub enum ActionKind {
    Input(Input),
    Packet {
        channel: u32,
        length: usize,
        value: u8,
    },
    FaultWork {
        work: Work,
        checkpoint: Checkpoint,
    },
    Restart,
    Disconnect,
    ReplayIntent {
        slot: u64,
    },
}

pub fn cid(label: &str) -> Vec<u8> {
    hash_parts("I1/cid/v1", &[label.as_bytes()])
}

impl Scenario {
    pub fn new(name: &str, seed: u64) -> Result<Self> {
        require(NAMES.contains(&name), "unknown scenario")?;
        require([7, 42, 99].contains(&seed), "seed must be 7, 42 or 99")?;
        let mut s = Self {
            name: name.to_owned(),
            seed,
            start_tick_us: START,
            epsilon_us: [1000, -1000],
            clock: Clock {
                default_request_us: 250,
                default_response_us: 250,
                lease_us: 100_000,
                refresh_us: None,
                refresh_until_tick_us: None,
                overrides: vec![],
            },
            links: vec![Link {
                channel: 0,
                delay_us: 20_000,
                jitter_us: 0,
                capacity: LINK_QUEUE_BOUND,
                offline: vec![],
                duplicate_delay_us: None,
                retry_delay_us: None,
                overflow: "drop-newest".to_owned(),
            }],
            watchdog_ms: 30_000,
            actions: vec![],
        };
        s.fixture(
            0,
            0,
            "first",
            if name == "close-work" {
                1_022_000
            } else {
                1_300_000
            },
        );
        s.fixture(1, 2_000, "first", 1_300_000);
        match name {
            "healthy" => {
                s.clock.refresh_us = Some(20_000);
                s.clock.refresh_until_tick_us = Some(1_045_000);
                for node in 0..2 {
                    s.input(10_000, node, I::ReadClock(c("first")));
                    s.input(15_000, node, I::BeginWork(work("first", 1, 5000)));
                    s.timer(16_000, node, "first", 1, 1, 1_050_000);
                    s.packet(24_000, node, 0, 32, node as u8);
                    s.input(70_000, node, I::Close(c("first")));
                }
            }
            "clock-faults" => {
                s.clock.overrides = vec![
                    ClockOverride {
                        node: 0,
                        request_id: 1,
                        request_us: 50,
                        response_us: 250,
                        response: SampleReply::Valid,
                    },
                    ClockOverride {
                        node: 1,
                        request_id: 1,
                        request_us: 1000,
                        response_us: 10_000,
                        response: SampleReply::Valid,
                    },
                    ClockOverride {
                        node: 0,
                        request_id: 2,
                        request_us: 250,
                        response_us: 250,
                        response: SampleReply::Replay,
                    },
                    ClockOverride {
                        node: 0,
                        request_id: 3,
                        request_us: 250,
                        response_us: 250,
                        response: SampleReply::InvalidThenValid,
                    },
                ];
                s.input(14_000, 1, I::Enter(c("first")));
                s.input(15_000, 0, I::ReadClock(c("first")));
                s.input(15_000, 1, I::ReadClock(c("first")));
                s.input(18_000, 0, I::RequestSample(Empty {}));
                s.input(20_000, 0, I::RequestSample(Empty {}));
                s.input(22_000, 0, I::ReadClock(c("first")));
            }
            "lease-expiry" => {
                s.clock.overrides.push(ClockOverride {
                    node: 0,
                    request_id: 2,
                    request_us: 250,
                    response_us: 250,
                    response: SampleReply::Lost,
                });
                s.input(20_000, 0, I::RequestSample(Empty {}));
                for delta in [50_000, 80_000, 99_000] {
                    s.input(delta, 0, I::ReadClock(c("first")));
                }
                // The first request is at START+1, so its lease ends at START+100251.
                for (slot, due) in [(1, 100_250), (2, 100_251), (3, 100_252)] {
                    s.input(due - 5000, 0, I::BeginWork(work("first", slot, 5000)));
                }
                s.input(103_000, 0, I::RequestSample(Empty {}));
                s.input(104_000, 0, I::Enter(c("first")));
                s.input(
                    105_000,
                    0,
                    I::Create(Fixture {
                        cid: cid("next"),
                        stop_local_us: 1_300_000,
                    }),
                );
                s.input(106_000, 0, I::AnchorReady(c("next")));
                s.input(107_000, 0, I::Enter(c("next")));
                s.input(108_000, 0, I::BeginWork(work("next", 1, 5000)));
            }
            "timers" => {
                s.timer(10_000, 0, "first", 1, 1, 1_050_000);
                s.timer(10_000, 1, "first", 1, 1, 1_050_000);
                s.timer(11_000, 0, "first", 2, 1, 1_060_000);
                s.input(12_000, 0, I::CancelTimer(t("first", 2, 1, 1_060_000)));
                s.timer(13_000, 0, "first", 2, 2, 1_070_000);
                s.timer(14_000, 0, "first", 3, 1, 1_090_000);
                s.input(20_000, 0, I::RequestSample(Empty {}));
                s.action(65_000, 0, ActionKind::Disconnect);
                s.action(71_000, 0, ActionKind::Restart);
                s.input(75_000, 0, I::Close(c("first")));
            }
            "close-work" => {
                s.input(10_000, 0, I::BeginWork(work("first", 1, 5_000)));
                s.input(16_000, 0, I::BeginWork(work("first", 2, 5_000)));
                s.input(18_000, 0, I::BeginWork(work("first", 3, 5_000)));
                s.input(17_000, 0, I::Enter(c("first")));
                s.input(
                    18_500,
                    0,
                    I::Create(Fixture {
                        cid: cid("first"),
                        stop_local_us: 1_022_000,
                    }),
                );
                s.input(21_000, 0, I::Close(c("first")));
                s.input(30_000, 0, I::Close(c("first")));
                s.action(31_000, 0, ActionKind::ReplayIntent { slot: 1 });
                s.input(16_000, 1, I::BeginWork(work("first", 1, 5_000)));
                s.input(21_000, 1, I::Close(c("first")));
            }
            "restart-before-ack" => {
                s.timer(5_000, 0, "first", 1, 1, 1_050_000);
                for (slot, checkpoint) in [
                    (1, Checkpoint::BeforeAppend),
                    (2, Checkpoint::PartialTail),
                    (3, Checkpoint::AfterSyncBeforeAck),
                    (4, Checkpoint::AfterHubCommit),
                    (5, Checkpoint::AfterRecordSyncBeforeHead),
                ] {
                    s.action(
                        slot * 9_000,
                        0,
                        ActionKind::FaultWork {
                            work: work("first", slot, 5000),
                            checkpoint,
                        },
                    );
                }
                s.input(43_000, 0, I::Enter(c("first")));
                s.input(44_000, 0, I::BeginWork(work("first", 1, 5000)));
                s.input(51_000, 0, I::BeginWork(work("first", 6, 5000)));
            }
            "restart-after-expiry" => {
                s.timer(6_000, 0, "first", 1, 1, 1_115_000);
                s.input(10_000, 0, I::BeginWork(work("first", 1, 5000)));
                s.packet(14_000, 1, 0, 32, 91);
                s.action(80_000, 0, ActionKind::Disconnect);
                s.action(112_000, 0, ActionKind::Restart);
                s.input(114_000, 0, I::Enter(c("first")));
                s.input(115_000, 0, I::BeginWork(work("first", 2, 5000)));
            }
            "network-faults" => {
                s.links[0].offline.push([1_019_000, 1_022_000]);
                s.links[0].retry_delay_us = Some(8_000);
                s.links[0].duplicate_delay_us = Some(2_000);
                s.links.push(Link {
                    channel: 1,
                    delay_us: 1_000,
                    jitter_us: 20,
                    capacity: LINK_QUEUE_BOUND,
                    offline: vec![],
                    duplicate_delay_us: None,
                    retry_delay_us: None,
                    overflow: "drop-newest".to_owned(),
                });
                s.packet(10_000, 0, 0, 32, 1);
                s.packet(1_500, 0, 0, 32, 2);
                s.packet(11_000, 0, 1, 32, 3);
                s.packet(12_000, 0, 1, MAX_PAYLOAD_BYTES, 4);
                s.packet(13_000, 0, 1, MAX_PAYLOAD_BYTES + 1, 5);
                for value in 10..75 {
                    s.packet(40_000, 0, 0, 16, value);
                }
            }
            _ => return Err(invalid("unknown scenario")),
        }
        Ok(s)
    }

    fn fixture(&mut self, node: u32, at: u64, label: &str, stop_local_us: i64) {
        self.input(
            at,
            node,
            I::Create(Fixture {
                cid: cid(label),
                stop_local_us,
            }),
        );
        self.input(at + 1, node, I::RequestSample(Empty {}));
        self.input(at + 800, node, I::AnchorReady(c(label)));
        self.input(at + 900, node, I::Enter(c(label)));
    }

    fn action(&mut self, delta: u64, node: u32, kind: ActionKind) {
        self.actions.push(Action {
            tick_us: START + delta,
            node,
            kind,
        });
    }

    fn input(&mut self, delta: u64, node: u32, kind: I) {
        self.action(delta, node, ActionKind::Input(Input { kind: Some(kind) }));
    }

    fn packet(&mut self, delta: u64, node: u32, channel: u32, length: usize, value: u8) {
        self.action(
            delta,
            node,
            ActionKind::Packet {
                channel,
                length,
                value,
            },
        );
    }

    fn timer(
        &mut self,
        delta: u64,
        node: u32,
        label: &str,
        id: u64,
        generation: u64,
        deadline: i64,
    ) {
        self.input(delta, node, I::ArmTimer(t(label, id, generation, deadline)));
    }
}

fn c(label: &str) -> Cid {
    Cid { cid: cid(label) }
}

fn work(label: &str, slot: u64, duration_us: u64) -> Work {
    Work {
        cid: cid(label),
        slot,
        subject: hash_parts("I1/cid/v1", &[&slot.to_be_bytes()]),
        duration_us,
    }
}

fn t(label: &str, timer_id: u64, generation: u64, local_deadline_us: i64) -> Timer {
    Timer {
        timer_id,
        generation,
        cid: cid(label),
        local_deadline_us,
    }
}
