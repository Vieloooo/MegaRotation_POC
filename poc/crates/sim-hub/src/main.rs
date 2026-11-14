mod hub;
mod scenario;
mod transport;

use serde_json::json;
use sim_runtime::{Result, invalid, require};
use std::{
    fs::{self, File},
    path::PathBuf,
};

#[derive(Debug)]
struct Options {
    scenario: String,
    seed: u64,
    output: PathBuf,
    wall_pause_ms: u64,
    reverse_start: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    match run().await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sim-hub: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<()> {
    let argv: Vec<_> = std::env::args().collect();
    let options = Options::parse(&argv[1..])?;
    let scenario = scenario::Scenario::new(&options.scenario, options.seed)?;
    fs::create_dir(&options.output)?;
    let output = options.output.canonicalize()?;
    serde_json::to_writer_pretty(File::create(output.join("scenario.json"))?, &scenario)?;
    serde_json::to_writer_pretty(File::create(output.join("commands.txt"))?, &argv)?;
    let mut hub = hub::Hub::new(scenario, &output, options.wall_pause_ms).await?;
    let result = hub.run(options.reverse_start).await;
    let cleanup = hub.finish(result.as_ref().err().map(Box::as_ref)).await;
    let result = result.and(cleanup);
    let (outcome, error) = match &result {
        Ok(_) => ("COMPLETED", None),
        Err(error) => ("INFRASTRUCTURE_FAILURE", Some(error.to_string())),
    };
    serde_json::to_writer_pretty(
        File::create(output.join("manifest.json"))?,
        &json!({
            "gate": "I1a", "fixture": "SignedIntent", "scenario": options.scenario, "seed": options.seed,
            "run_id": hub.run_id, "config_hash": hub.config_hash, "outcome": outcome, "error": error,
            "wall_pause_ms": options.wall_pause_ms, "reverse_start": options.reverse_start,
        }),
    )?;
    result
}

impl Options {
    fn parse(args: &[String]) -> Result<Self> {
        let mut scenario = None;
        let mut seed = None;
        let mut output = None;
        let mut wall_pause_ms = None;
        let mut reverse_start = false;
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            let missing = || invalid(format!("missing value for {arg}"));
            match arg.as_str() {
                "--scenario" if scenario.is_none() => scenario = args.next().cloned(),
                "--seed" if seed.is_none() => {
                    seed = Some(args.next().ok_or_else(missing)?.parse()?)
                }
                "--output" if output.is_none() => output = args.next().map(PathBuf::from),
                "--wall-pause-ms" if wall_pause_ms.is_none() => {
                    wall_pause_ms = Some(args.next().ok_or_else(missing)?.parse()?)
                }
                "--reverse-start" if !reverse_start => reverse_start = true,
                _ => return Err(invalid(format!("unknown or repeated flag {arg}"))),
            }
        }
        let wall_pause_ms = wall_pause_ms.unwrap_or(0);
        require(
            wall_pause_ms <= 2 || wall_pause_ms == 30_050,
            "use a wall pause of 0..2 ms or 30050 ms for the watchdog case",
        )?;
        Ok(Self {
            scenario: scenario.ok_or_else(|| invalid("missing --scenario"))?,
            seed: seed.ok_or_else(|| invalid("missing --seed"))?,
            output: output.ok_or_else(|| invalid("missing --output"))?,
            wall_pause_ms,
            reverse_start,
        })
    }
}
