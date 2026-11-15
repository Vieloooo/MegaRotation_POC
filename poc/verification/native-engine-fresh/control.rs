//! Independent one-worker tonic authority for completion/cancellation boundaries.
use super::hash;
use p::{
    effect, hub_frame, input,
    native_run_server::{NativeRun, NativeRunServer},
    worker_frame,
};
use poc_native_runtime::native_wire as p;
use prost::Message;
use serde_json::{Value, json};
use std::{
    path::Path,
    process::{Child, Command, Stdio},
    time::Duration,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::{ReceiverStream, TcpListenerStream};
use tonic::{Request, Response, Status, Streaming, transport::Server};
type R<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
type Tx = mpsc::Sender<Result<p::HubFrame, Status>>;
enum Incoming {
    Ready(Tx),
    Ack(p::Ack),
    Lost(String),
}
#[derive(Clone)]
struct Service {
    registration: p::RegisterRequest,
    clock: Vec<u8>,
    bus: mpsc::Sender<Incoming>,
}
#[tonic::async_trait]
impl NativeRun for Service {
    async fn register(
        &self,
        request: Request<p::RegisterRequest>,
    ) -> Result<Response<p::RegisterReply>, Status> {
        if request.into_inner() != self.registration {
            return Err(Status::invalid_argument("identity"));
        }
        Ok(Response::new(p::RegisterReply {
            binding: self.registration.binding.clone(),
            clock_domain: self.clock.clone(),
        }))
    }
    type RunStream = ReceiverStream<Result<p::HubFrame, Status>>;
    async fn run(
        &self,
        request: Request<Streaming<p::WorkerFrame>>,
    ) -> Result<Response<Self::RunStream>, Status> {
        let mut stream = request.into_inner();
        match stream.message().await? {
            Some(p::WorkerFrame {
                frame: Some(worker_frame::Frame::Hello(h)),
            }) if h == self.registration => {}
            _ => return Err(Status::invalid_argument("hello")),
        }
        let (tx, rx) = mpsc::channel(4);
        self.bus
            .send(Incoming::Ready(tx))
            .await
            .map_err(|_| Status::unavailable("driver"))?;
        let bus = self.bus.clone();
        tokio::spawn(async move {
            loop {
                match stream.message().await {
                    Ok(Some(p::WorkerFrame {
                        frame: Some(worker_frame::Frame::Ack(a)),
                    })) => {
                        if bus.send(Incoming::Ack(a)).await.is_err() {
                            break;
                        }
                    }
                    result => {
                        let _ = bus.send(Incoming::Lost(format!("{result:?}"))).await;
                        break;
                    }
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
fn data(v: &Value, key: &str) -> Vec<u8> {
    serde_json::from_value(v[key].clone()).unwrap()
}
struct Driver {
    tx: Tx,
    bus: mpsc::Receiver<Incoming>,
    binding: p::Binding,
    seed: u64,
    sequence: u64,
    frames: Vec<Value>,
}
impl Driver {
    async fn step(&mut self, tick: u64, kind: input::Kind) -> R<p::Ack> {
        self.sequence += 1;
        let id = self.sequence;
        let mut grant = p::Grant {
            binding: Some(self.binding.clone()),
            logical_step_id: id,
            event_id: id,
            tick_us: tick,
            causal_wave: 0,
            tie_rank: hash(
                "I1b/fresh-tie/v1",
                &[&self.seed.to_be_bytes(), &id.to_be_bytes()],
            )
            .to_vec(),
            input: Some(p::Input { kind: Some(kind) }),
            grant_hash: vec![],
        };
        grant.grant_hash = hash("I1b/fresh-grant/v1", &[&grant.encode_to_vec()]).to_vec();
        self.frames.push(json!({"grant":grant}));
        self.tx
            .send(Ok(p::HubFrame {
                frame: Some(hub_frame::Frame::Grant(grant.clone())),
            }))
            .await?;
        let ack = match tokio::time::timeout(Duration::from_secs(15), self.bus.recv()).await? {
            Some(Incoming::Ack(a)) => a,
            Some(Incoming::Lost(s)) => return Err(s.into()),
            _ => return Err("unexpected worker input".into()),
        };
        let mut logical = ack.clone();
        logical.logical_ack_hash.clear();
        logical.durable_step_hash.clear();
        if ack.binding != grant.binding
            || ack.logical_step_id != id
            || ack.grant_hash != grant.grant_hash
            || ack.logical_ack_hash != hash("I1b/fresh-ack/v1", &[&logical.encode_to_vec()])
        {
            return Err("Ack binding/hash".into());
        }
        self.frames.push(json!({"ack":ack}));
        Ok(ack)
    }
}
pub async fn run(mode: &str, worker: &str, template: &str, out: &str) -> R<()> {
    let root = Path::new(out);
    std::fs::create_dir(root)?;
    let work = root.join("worker");
    std::fs::create_dir(&work)?;
    let mut launch: Value = serde_json::from_slice(&std::fs::read(template)?)?;
    let c = launch["resolved"].clone();
    launch["work_dir"] = json!(work);
    launch["wall_pause_ms"] = json!(0);
    let launch_file = root.join("launch-private.json");
    use std::os::unix::fs::OpenOptionsExt;
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&launch_file)?;
    serde_json::to_writer(file, &launch)?;
    let run = data(&c, "run_id");
    let config = data(&c, "config_hash");
    let cid = data(&c, "cid");
    let binding = p::Binding {
        schema_version: 1,
        run_id: run.clone(),
        config_hash: config.clone(),
        worker_id: 0,
        member_id: 0,
        incarnation: 1,
    };
    let clock = hash("I1/clock-domain/v1", &[&run, &config, &0u32.to_be_bytes()]).to_vec();
    let registration = p::RegisterRequest {
        binding: Some(binding.clone()),
        registry_hash: data(&c["base"], "registry_hash"),
        namespace_hash: data(&c, "namespace"),
        profile: "native-engine-fresh-v1".into(),
        recovery: None,
    };
    let (bus, mut rx) = mpsc::channel(8);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let endpoint = format!("http://{}", listener.local_addr()?);
    let service = Service {
        registration,
        clock: clock.clone(),
        bus,
    };
    let server = tokio::spawn(
        Server::builder()
            .add_service(
                NativeRunServer::new(service)
                    .max_decoding_message_size(1048576)
                    .max_encoding_message_size(1048576),
            )
            .serve_with_incoming(TcpListenerStream::new(listener)),
    );
    let mut child = ChildGuard(
        Command::new(worker)
            .arg(endpoint)
            .arg(&launch_file)
            .stdin(Stdio::null())
            .stdout(std::fs::File::create(root.join("stdout"))?)
            .stderr(std::fs::File::create(root.join("stderr"))?)
            .spawn()?,
    );
    let pid = child.0.id();
    let tx = match tokio::time::timeout(Duration::from_secs(15), rx.recv()).await? {
        Some(Incoming::Ready(tx)) => tx,
        _ => return Err("worker did not register".into()),
    };
    let mut d = Driver {
        tx,
        bus: rx,
        binding,
        seed: c["base"]["seed"].as_u64().unwrap(),
        sequence: 0,
        frames: vec![],
    };
    let sample = |request, sequence, as_of| p::ClockSample {
        run_id: run.clone(),
        worker_id: 0,
        incarnation: 1,
        clock_domain: clock.clone(),
        rate_profile: hash("I1/rate/v1", &[b"fixed-rate-one-v1"]).to_vec(),
        request_id: request,
        sequence,
        as_of_tick_us: as_of,
        node_time_at_as_of_us: as_of as i64 + 1000,
        uncertainty_us: 1000,
        lease_end_tick_us: as_of + 100000,
    };
    d.step(1000000, input::Kind::RequestSample(p::Empty {}))
        .await?;
    d.step(1000500, input::Kind::Sample(sample(1, 1, 1000250)))
        .await?;
    let start = if mode == "stop-boundary" {
        d.step(1793250, input::Kind::RequestSample(p::Empty {}))
            .await?;
        d.step(1793750, input::Kind::Sample(sample(2, 2, 1793500)))
            .await?;
        1794000
    } else if mode == "lease-boundary" {
        1095250
    } else {
        1000500
    };
    let entered = d
        .step(
            start,
            input::Kind::Start(p::StartEngine {
                cid: cid.clone(),
                epoch: 1,
                genesis_digest: data(&c, "genesis"),
                registry_hash: data(&c["base"], "registry_hash"),
                namespace_hash: data(&c, "namespace"),
                native_config_hash: data(&c, "native_config_hash"),
                app_fixture_hash: data(&c, "app_fixture_hash"),
                height_limit: c["base"]["height_limit"].as_u64().unwrap(),
                local_stop_us: 1800000,
            }),
        )
        .await?;
    let work = entered
        .effects
        .iter()
        .find_map(|e| match &e.kind {
            Some(effect::Kind::Work(w)) if w.kind == 1 => Some(w.clone()),
            _ => None,
        })
        .ok_or("actual native propose did not start")?;
    if mode == "explicit-close" {
        d.step(
            start + 2500,
            input::Kind::Close(p::Cid { cid: cid.clone() }),
        )
        .await?;
    }
    let completed = d
        .step(start + 5000, input::Kind::WorkCompleted(work))
        .await?;
    let runtime: Value = serde_json::from_slice(&completed.runtime_events_json)?;
    let app: Value = serde_json::from_slice(&completed.application_events_json)?;
    let work_completed = app["work"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event"] == "work_completed")
        .count();
    let work_discarded = app["work"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event"] == "work_discarded")
        .count();
    let persisted = app["work"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event"] == "persisted")
        .count();
    let signs = completed
        .effects
        .iter()
        .filter(|e| matches!(e.kind, Some(effect::Kind::Sign(_))))
        .count();
    let retired = runtime
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["kind"] == "retired");
    d.tx.send(Ok(p::HubFrame {
        frame: Some(hub_frame::Frame::Shutdown(p::Shutdown {
            reason: "complete".into(),
        })),
    }))
    .await?;
    let end = tokio::time::Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.0.try_wait()? {
            break status;
        }
        if tokio::time::Instant::now() > end {
            return Err("worker shutdown timeout".into());
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    };
    server.abort();
    serde_json::to_writer_pretty(std::fs::File::create(root.join("frames.json"))?, &d.frames)?;
    let passed = work_completed == 0
        && work_discarded > 0
        && persisted == 0
        && signs == 0
        && status.success();
    let report = json!({"mode":mode,"scope":"one actual native worker with independent tonic authority; fresh session only","start_tick_us":start,"completion_tick_us":start+5000,"pid":pid,"exit_code":status.code(),"retired_in_completion_grant":retired,"work_completed":work_completed,"work_discarded":work_discarded,"persisted":persisted,"sign_effects":signs,"no_native_consumption_inferred":true,"expected":"obsolete completion discarded, no persistence/signing","status":if passed{"PASS"}else{"FAIL"}});
    serde_json::to_writer_pretty(std::fs::File::create(root.join("report.json"))?, &report)?;
    println!("{report}");
    Ok(())
}
