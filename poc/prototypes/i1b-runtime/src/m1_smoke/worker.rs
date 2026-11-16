use super::profile::{self, Install, PublicInputs, ReadyMaterial};
use crate::{
    ControlledDriver, Error, Result,
    native_config::Resolved,
    native_replica::Replica,
    native_step_journal::StepJournal,
    native_wire::{Ack, Grant},
    native_wire_support::MAX_FRAME,
};
use poc_types::domain::Pid;
use prost::Message;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Serialize, Deserialize)]
pub(crate) struct Launch {
    pub install: Install,
    pub inputs: PublicInputs,
}

fn frame(reader: &mut impl Read) -> Result<Option<Vec<u8>>> {
    let mut len = [0u8; 4];
    let count = reader.read(&mut len[..1]).map_err(io_error)?;
    if count == 0 {
        return Ok(None);
    }
    reader.read_exact(&mut len[1..]).map_err(io_error)?;
    let len = u32::from_be_bytes(len) as usize;
    if len == 0 || len > MAX_FRAME {
        return Err(Error("invalid M1 worker frame length"));
    }
    let mut bytes = vec![0; len];
    reader.read_exact(&mut bytes).map_err(io_error)?;
    Ok(Some(bytes))
}

fn write(writer: &mut impl Write, ack: Ack) -> Result<()> {
    let bytes = ack.encode_to_vec();
    if bytes.len() > MAX_FRAME {
        return Err(Error("oversized M1 worker Ack"));
    }
    writer
        .write_all(&(bytes.len() as u32).to_be_bytes())
        .map_err(io_error)?;
    writer.write_all(&bytes).map_err(io_error)?;
    writer.flush().map_err(io_error)
}

fn io_error(_: std::io::Error) -> Error {
    Error("M1 worker pipe or journal I/O failure")
}

pub(super) const IDLE_RETIRE: &[u8] = b"PoC/M3/idle-retire/v1";

pub(crate) fn run(pid: Pid, root: PathBuf, auto_force: bool) -> Result<()> {
    let mut input = std::io::stdin();
    let frame_bytes = frame(&mut input)?.ok_or(Error("M1 worker exited before assigned"))?;
    if auto_force && frame_bytes == IDLE_RETIRE {
        return durable_file(&root.join("idle-retire.bin"), &frame_bytes);
    }
    let launch: Launch =
        serde_json::from_slice(&frame_bytes).map_err(|_| Error("invalid M1 Launch"))?;
    let plans = launch.inputs.plans()?;
    let cfg = Arc::new(Resolved::normal(&plans, &launch.install.descriptor)?);
    let member = u32::from(pid.member().get());
    let native = cfg.signer(member, profile::fixture_key(pid))?;
    let mut replica = Replica::new(cfg.clone(), member, native)?;
    let material = ReadyMaterial::prepare(launch.inputs, &launch.install, pid)?;
    replica.prepare_normal(material);
    durable_file(&root.join("install.json"), &frame_bytes)?;
    let mut steps = StepJournal::create(&root, &cfg, u32::from(pid.get()))?;
    let storage = root.join("native");
    ControlledDriver::run_async(
        commonware_runtime::tokio::Config::default()
            .with_worker_threads(1)
            .with_storage_directory(storage),
        cfg.profile(member),
        async move |driver| {
            driver.enable_native_clock_audit();
            driver.configure_witness(cfg.witness(member)?, &root.join("witness"))?;
            let mut output = std::io::stdout();
            while let Some(bytes) = frame(&mut input)? {
                let grant = Grant::decode(bytes.as_slice())
                    .map_err(|_| Error("invalid M1 Grant protobuf"))?;
                if grant.tick_us < launch.install.prepared_at_us {
                    return Err(Error("M1 runtime grant preceded installation"));
                }
                let ack = if let Some(ack) = steps.cached(&grant)? {
                    ack
                } else {
                    let ack = replica.step(driver, &grant).await?;
                    steps.commit(grant, ack)?
                };
                write(&mut output, ack)?;
            }
            Ok(())
        },
    )
}

pub(crate) fn durable_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(io_error)?;
    file.write_all(bytes).map_err(io_error)?;
    file.sync_all().map_err(io_error)?;
    File::open(path.parent().ok_or(Error("M1 file has no parent"))?)
        .and_then(|directory| directory.sync_all())
        .map_err(io_error)?;
    Ok(())
}
