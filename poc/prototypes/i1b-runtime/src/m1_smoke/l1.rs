use super::{M1Result, check, problem, profile::PublicInputs};
use poc_types::domain::{
    FinalizedL1Reader, Hash32, InputMissing, InputStatus, L1BlockRef, L1Header, L1Position,
};
use poc_viewer_gfr::{ControlRef, L1ControlReader};
use serde_json::{Value, json};
use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

pub(super) const SOURCE: &[u8] = b"M2/healthy-force/anvil-l1/v1";
pub(super) const SUBMIT_US: u64 = 11_500_000;
pub(super) const ACT_US: u64 = 22_000_000;
pub(super) const L1_SLOT_US: u64 = 2_000_000;
pub(super) const FINALITY_US: u64 = 1_500_000;
// Reader A completes dispatch at the finalization tick. The worker projection
// becomes readable one tick later, after all same-time scheduler waves.
pub(super) const WORKER_VISIBILITY_LAG_US: u64 = 1;

pub(super) struct Port {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
}

impl Port {
    pub fn start(root: &Path, config: Hash32) -> M1Result<Self> {
        let mut child =
            Command::new(env::var_os("M2_PYTHON").ok_or_else(|| problem("M2_PYTHON missing"))?)
                .arg(env::var_os("M2_L1_PORT").ok_or_else(|| problem("M2_L1_PORT missing"))?)
                .args(["--anvil", &env::var("M2_ANVIL")?])
                .args(["--driver", &env::var("M2_DRIVER")?])
                .args(["--contracts", &env::var("M2_CONTRACTS")?])
                .arg("--output")
                .arg(root.join("l1"))
                .args([
                    "--config-ref",
                    &format!("0x{}", poc_types::hex(config.as_bytes())),
                ])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(File::create(root.join("l1-port.stderr"))?)
                .spawn()?;
        let input = child.stdin.take();
        let output = BufReader::new(
            child
                .stdout
                .take()
                .ok_or_else(|| problem("missing L1 port output"))?,
        );
        Ok(Self {
            child,
            input,
            output,
        })
    }

    pub fn step(&mut self, command: Value) -> M1Result<Value> {
        let input = self
            .input
            .as_mut()
            .ok_or_else(|| problem("L1 port is closed"))?;
        serde_json::to_writer(&mut *input, &command)?;
        input.write_all(b"\n")?;
        input.flush()?;
        let mut line = String::new();
        check(
            self.output.read_line(&mut line)? != 0,
            "L1 port exited; inspect l1-port.stderr",
        )?;
        Ok(serde_json::from_str(&line)?)
    }

    pub fn finish(mut self) -> M1Result<()> {
        drop(self.input.take());
        check(
            self.child.wait()?.success(),
            "L1 port failed to flush evidence",
        )
    }
}

impl Drop for Port {
    fn drop(&mut self) {
        drop(self.input.take());
        // The child observes EOF and tears down its Anvil instance and driver.
        let _ = self.child.wait();
    }
}

#[derive(Default)]
pub(super) struct Reader {
    headers: Vec<L1Header>,
    controls: Vec<Vec<ControlRef>>,
}

impl Reader {
    pub fn receive(&mut self, value: &Value, inputs: &PublicInputs) -> M1Result<L1Header> {
        let height = field(value, "number")?;
        check(
            height == self.headers.len() as u64,
            "noncontiguous finalized L1 delivery",
        )?;
        let header = L1Header {
            height,
            hash: bytes32(&value["hash"])?,
            parent_hash: bytes32(&value["header_and_transactions"]["parentHash"])?,
            timestamp_s: field(value, "timestamp")?,
        };
        let previous = self.headers.last().map_or(Hash32::new([0; 32]), |h| h.hash);
        check(
            header.parent_hash == previous,
            "actual L1 header ancestry mismatch",
        )?;
        let projected = inputs
            .l1
            .rows
            .get(height as usize)
            .ok_or_else(|| problem("L1 metadata projection out of range"))?;
        check(
            projected.header.height == height && projected.header.timestamp_s == header.timestamp_s,
            "actual L1 header disagrees with worker metadata projection",
        )?;
        let mut controls = Vec::new();
        for event in value["events"]
            .as_array()
            .ok_or_else(|| problem("missing L1 events"))?
        {
            match event["event"].as_str() {
                Some("cursor_initialized") => (),
                Some("force") => {
                    let policy = &event["policy"];
                    check(
                        field(policy, "enabled_mask")? == 7
                            && policy["order"] == json!([0, 1, 2])
                            && field(event, "act")? == header.timestamp_s + 10,
                        "non-static Force or inconsistent activation",
                    )?;
                    let position = &event["position"];
                    let at = L1Position {
                        block: field(position, "block")?,
                        tx: field(position, "tx")?,
                        log: field(position, "log")?,
                    };
                    controls.push(ControlRef::static_force(
                        at,
                        field(event, "seq")?,
                        field(event, "branch")?,
                    ));
                }
                _ => return Err(problem("unexpected event in healthy Force L1 prefix")),
            }
        }
        self.headers.push(header.clone());
        self.controls.push(controls);
        Ok(header)
    }
}

impl FinalizedL1Reader for Reader {
    fn header(&self, height: u64) -> InputStatus<L1Header> {
        self.headers.get(height as usize).cloned().map_or(
            InputStatus::Wait(InputMissing::L1Header(height)),
            InputStatus::Ready,
        )
    }

    fn require_control_coverage(&self, through_height: u64) -> InputStatus<()> {
        if through_height < self.controls.len() as u64 {
            InputStatus::Ready(())
        } else {
            InputStatus::Wait(InputMissing::ControlCoverage(through_height))
        }
    }

    fn finalized_tip(&self) -> Option<L1BlockRef> {
        self.headers.last().map(|h| L1BlockRef {
            height: h.height,
            hash: h.hash,
        })
    }
}

impl L1ControlReader for Reader {
    fn source_id(&self) -> &[u8] {
        SOURCE
    }

    fn controls_at(&self, height: u64) -> InputStatus<Vec<ControlRef>> {
        self.controls.get(height as usize).cloned().map_or(
            InputStatus::Wait(InputMissing::ControlCoverage(height)),
            InputStatus::Ready,
        )
    }
}

fn field(value: &Value, name: &str) -> M1Result<u64> {
    value[name]
        .as_u64()
        .ok_or_else(|| problem(format!("missing L1 integer {name}")))
}

fn bytes32(value: &Value) -> M1Result<Hash32> {
    let text = value
        .as_str()
        .and_then(|v| v.strip_prefix("0x"))
        .ok_or_else(|| problem("expected hexadecimal L1 hash"))?;
    check(
        text.len() == 64 && text.is_ascii(),
        "unexpected L1 hash length",
    )?;
    let bytes = (0..32)
        .map(|i| u8::from_str_radix(&text[i * 2..i * 2 + 2], 16))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Hash32::try_from(bytes.as_slice())?)
}

#[derive(Clone, Copy)]
pub(super) enum Event {
    Genesis,
    Deploy,
    Submit,
    Retry,
    Mine(u64),
    Finalize(u64),
    Deliver(u64, &'static str),
}

impl Event {
    pub fn command(self, tick_us: u64) -> Value {
        let (op, height, reader) = match self {
            Self::Genesis => ("genesis", 0, "A"),
            Self::Deploy => ("deploy", 0, "A"),
            Self::Submit => ("submit", 0, "A"),
            Self::Retry => ("retry", 0, "A"),
            Self::Mine(height) => ("mine", height, "A"),
            Self::Finalize(height) => ("finalize", height, "A"),
            Self::Deliver(height, reader) => ("deliver", height, reader),
        };
        json!({"op": op, "height": height, "reader": reader, "tick_us": tick_us})
    }
}
