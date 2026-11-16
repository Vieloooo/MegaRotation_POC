mod hub;
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

pub fn run(args: &[String]) -> M1Result<()> {
    match args {
        [mode, pid, output] if mode == "--worker" => {
            let pid = Pid::new(pid.parse()?)?;
            worker::run(pid, PathBuf::from(output))?;
            Ok(())
        }
        [seed_flag, seed, output_flag, output]
            if seed_flag == "--seed" && seed == "42" && output_flag == "--output" =>
        {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?
                .block_on(hub::run(PathBuf::from(output)))
        }
        _ => Err(problem("usage: m1-smoke --seed 42 --output NEW_DIRECTORY")),
    }
}
