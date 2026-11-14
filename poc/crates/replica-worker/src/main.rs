use poc_types::{
    hub_frame::Frame as Hub, runtime_client::RuntimeClient, worker_frame::Frame as Worker, *,
};
use sim_runtime::{
    Launch, Result, invalid,
    journal::{Journal, StepResult},
    require,
};
use std::{
    fs::File,
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    match run().await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("replica-worker: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    require(
        args.len() == 2,
        "usage: replica-worker ENDPOINT LAUNCH_JSON",
    )?;
    let launch: Launch = serde_json::from_reader(File::open(&args[1])?)?;
    require(launch.incarnation > 0, "invalid worker incarnation")?;
    let identity = &launch.identity;
    let mut journal = Journal::open(&launch.journal, identity.clone(), launch.incarnation == 1)?;
    let mut client = RuntimeClient::connect(args[0].clone())
        .await?
        .max_decoding_message_size(MAX_FRAME_BYTES)
        .max_encoding_message_size(MAX_FRAME_BYTES);
    let register = RegisterRequest {
        schema_version: SCHEMA_VERSION,
        run_id: identity.run_id.clone(),
        node_id: identity.node_id,
        incarnation: launch.incarnation,
        config_hash: identity.config_hash.clone(),
        recovered_journal_seq: journal.sequence(),
        recovered_state_hash: journal.state().hash()?,
    };
    let reply = client.register(register.clone()).await?.into_inner();
    require(
        reply.schema_version == SCHEMA_VERSION
            && reply.run_id == identity.run_id
            && reply.node_id == identity.node_id
            && reply.incarnation == launch.incarnation
            && reply.config_hash == identity.config_hash
            && reply.clock_domain == identity.clock_domain(),
        "invalid Register reply",
    )?;

    let (sender, receiver) = mpsc::channel(4);
    sender
        .send(WorkerFrame {
            frame: Some(Worker::Hello(register)),
        })
        .await
        .map_err(|_| invalid("worker Hello channel closed"))?;
    let mut stream = client
        .controlled_run(ReceiverStream::new(receiver))
        .await?
        .into_inner();
    let pause = Duration::from_millis(launch.wall_pause_ms);
    let mut retry = launch.retry;
    while let Some(message) = stream.message().await? {
        let frame = message.frame.ok_or_else(|| invalid("empty hub frame"))?;
        let grant = match frame {
            Hub::Shutdown(shutdown) => {
                if shutdown.reason == "scripted-after-hub-commit" {
                    std::process::exit(sim_runtime::Checkpoint::AfterHubCommit.exit_code());
                }
                require(
                    shutdown.reason == "complete" || shutdown.reason == "scripted-offline",
                    "unknown shutdown reason",
                )?;
                return Ok(());
            }
            Hub::Grant(grant) => grant,
        };
        tokio::time::sleep(pause).await;
        let fault = match grant.input.as_ref().and_then(|i| i.kind.as_ref()) {
            Some(input::Kind::WorkCompleted(work)) => launch
                .faults
                .iter()
                .find(|(slot, _)| *slot == work.slot)
                .map(|(_, fault)| *fault),
            _ => None,
        };
        let started = Instant::now();
        match journal.process(&grant, launch.incarnation, retry.take().as_ref(), fault)? {
            StepResult::Crash(checkpoint) => {
                eprintln!(
                    "scripted checkpoint {:?} for step {}",
                    checkpoint, grant.logical_step_id
                );
                std::process::exit(checkpoint.exit_code());
            }
            StepResult::Ack(ack) => {
                eprintln!(
                    "step {} journal {} apply_and_sync_us {}",
                    ack.logical_step_id,
                    ack.durable_journal_seq,
                    started.elapsed().as_micros()
                );
                tokio::time::sleep(pause).await;
                sender
                    .send(WorkerFrame {
                        frame: Some(Worker::Ack(ack)),
                    })
                    .await
                    .map_err(|_| invalid("Ack channel closed"))?;
            }
        }
    }
    Err(invalid("stream closed without shutdown"))
}
