mod governance;
mod hub;
mod l1;
mod m4;
mod m5;
pub(crate) mod profile;
mod public;
mod worker;

use poc_types::domain::Pid;
use std::{fmt::Debug, path::PathBuf};

type M1Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn problem(text: impl Into<String>) -> Box<dyn std::error::Error> {
    std::io::Error::other(text.into()).into()
}

fn checked<T>(result: std::result::Result<T, impl Debug>) -> M1Result<T> {
    result.map_err(|error| problem(format!("{error:?}")))
}

fn check(value: bool, message: impl Into<String>) -> M1Result<()> {
    if value { Ok(()) } else { Err(problem(message)) }
}

pub fn run_m4_driver(args:&[String])->M1Result<()> {m4::run_driver(args)}

pub fn run(args: &[String]) -> M1Result<()> {
    match args {
        [mode, pid, output] if matches!(mode.as_str(), "--worker" | "--auto-force-worker") => {
            let pid = Pid::new(pid.parse()?)?;
            worker::run(pid, PathBuf::from(output), mode == "--auto-force-worker")?;
            Ok(())
        }
        [seed_flag, seed, output_flag, output]
            if seed_flag == "--seed" && seed == "42" && output_flag == "--output" =>
        {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?
                .block_on(hub::run(PathBuf::from(output), false, 0, None,false,None))
        }
        [mode, seed_flag, seed, output_flag, output, pause_flag, pause_ms]
            if mode == "--healthy-force" && seed_flag == "--seed" && seed == "42"
                && output_flag == "--output" && pause_flag == "--wall-pause-ms" =>
        {
            let pause_ms = pause_ms.parse::<u64>()?;
            check(pause_ms <= 60_000, "wall pause must be at most 60 seconds")?;
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?
                .block_on(hub::run(PathBuf::from(output), true, pause_ms, None,false,None))
        }
        [mode, scenario, seed_flag, seed, output_flag, output, pause_flag, pause_ms]
            if mode == "--auto-force" && seed_flag == "--seed" && seed == "42"
                && output_flag == "--output" && pause_flag == "--wall-pause-ms" =>
        {
            let pause_ms = pause_ms.parse::<u64>()?;
            check(pause_ms <= 60_000, "wall pause must be at most 60 seconds")?;
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?
                .block_on(hub::run(PathBuf::from(output), true, pause_ms, Some(governance::Scenario::parse(scenario)?),false,None))
        }
        [mode,seed_flag,seed,output_flag,output,pause_flag,pause_ms]
            if mode=="--preset-resume" && seed_flag=="--seed" && seed=="42"
                && output_flag=="--output" && pause_flag=="--wall-pause-ms" => {
            let pause_ms=pause_ms.parse::<u64>()?;
            check(pause_ms<=60_000,"wall pause must be at most 60 seconds")?;
            tokio::runtime::Builder::new_current_thread().enable_all().build()?
                .block_on(hub::run(PathBuf::from(output),true,pause_ms,None,true,None))
        }
        [mode, scenario, seed_flag, seed, output_flag, output, pause_flag, pause_ms]
            if mode == "--auto-resume" && seed_flag == "--seed" && seed == "42"
                && output_flag == "--output" && pause_flag == "--wall-pause-ms" =>
        {
            let pause_ms = pause_ms.parse::<u64>()?;
            check(pause_ms <= 60_000, "wall pause must be at most 60 seconds")?;
            tokio::runtime::Builder::new_current_thread().enable_all().build()?
                .block_on(hub::run(PathBuf::from(output), true, pause_ms, None, true,
                    Some(m5::Mode::parse(scenario)?)))
        }
        _ => Err(problem("usage: m1-smoke --seed 42 --output NEW_DIRECTORY or m1-smoke --healthy-force --seed 42 --output NEW_DIRECTORY --wall-pause-ms N")),
    }
}
