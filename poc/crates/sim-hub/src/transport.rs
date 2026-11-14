use poc_types::{
    hub_frame::Frame as Hub,
    runtime_server::{Runtime, RuntimeServer},
    worker_frame::Frame as Worker,
    *,
};
use serde_json::{Value, json};
use sim_runtime::{Identity, Launch, Result, invalid, require};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    net::TcpListener,
    process::{Child, Command},
    sync::mpsc,
    task::JoinHandle,
};
use tokio_stream::wrappers::{ReceiverStream, TcpListenerStream};
use tonic::{Request, Response, Status, Streaming, transport::Server};

const WATCHDOG: Duration = Duration::from_secs(30);
type StreamSender = mpsc::Sender<std::result::Result<HubFrame, Status>>;

pub enum Incoming {
    Ready(RegisterRequest, StreamSender),
    Ack(u32, u64, Ack),
    Disconnected(u32, u64, String),
}

#[derive(Clone)]
struct Service {
    expected: Arc<Mutex<[Registration; 2]>>,
    incoming: mpsc::Sender<Incoming>,
}

struct Registration {
    identity: Identity,
    incarnation: u64,
    claimed: Option<RegisterRequest>,
    attached: bool,
}

#[tonic::async_trait]
impl Runtime for Service {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> std::result::Result<Response<RegisterReply>, Status> {
        let register = request.into_inner();
        let mut expected = self.expected.lock().expect("registration mutex");
        let node = expected
            .get_mut(register.node_id as usize)
            .ok_or_else(|| Status::invalid_argument("node"))?;
        let id = &node.identity;
        if register.schema_version != SCHEMA_VERSION
            || register.run_id != id.run_id
            || register.config_hash != id.config_hash
            || register.incarnation != node.incarnation
            || register.incarnation == 0
            || register.recovered_state_hash.len() != 32
            || node.claimed.is_some()
        {
            return Err(Status::invalid_argument("Register identity or duplicate"));
        }
        let reply = RegisterReply {
            schema_version: SCHEMA_VERSION,
            run_id: id.run_id.clone(),
            node_id: id.node_id,
            incarnation: node.incarnation,
            clock_domain: id.clock_domain(),
            config_hash: id.config_hash.clone(),
        };
        node.claimed = Some(register);
        Ok(Response::new(reply))
    }

    type ControlledRunStream = ReceiverStream<std::result::Result<HubFrame, Status>>;

    async fn controlled_run(
        &self,
        request: Request<Streaming<WorkerFrame>>,
    ) -> std::result::Result<Response<Self::ControlledRunStream>, Status> {
        let mut stream = request.into_inner();
        let first = stream.message().await?.and_then(|frame| frame.frame);
        let Some(Worker::Hello(hello)) = first else {
            return Err(Status::invalid_argument("expected Hello"));
        };
        {
            let mut expected = self.expected.lock().expect("registration mutex");
            let node = expected
                .get_mut(hello.node_id as usize)
                .ok_or_else(|| Status::invalid_argument("node"))?;
            if node.claimed.as_ref() != Some(&hello) || node.attached {
                return Err(Status::invalid_argument("unregistered or duplicate Hello"));
            }
            node.attached = true;
        }
        let (tx, rx) = mpsc::channel(4);
        let incoming = self.incoming.clone();
        let (node, incarnation) = (hello.node_id, hello.incarnation);
        incoming
            .send(Incoming::Ready(hello, tx))
            .await
            .map_err(|_| Status::unavailable("hub closed"))?;
        tokio::spawn(async move {
            let reason = loop {
                match stream.message().await {
                    Ok(Some(WorkerFrame {
                        frame: Some(Worker::Ack(ack)),
                    })) => {
                        if incoming
                            .send(Incoming::Ack(node, incarnation, ack))
                            .await
                            .is_err()
                        {
                            return;
                        }
                    }
                    Ok(None) => break "EOF".to_owned(),
                    Ok(Some(_)) => break "invalid worker frame".to_owned(),
                    Err(error) => break error.to_string(),
                }
            };
            let _ = incoming
                .send(Incoming::Disconnected(node, incarnation, reason))
                .await;
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

pub struct Transport {
    service: Service,
    incoming: mpsc::Receiver<Incoming>,
    streams: [Option<StreamSender>; 2],
    processes: [Option<Child>; 2],
    history: [Vec<Value>; 2],
    started: Instant,
    output: PathBuf,
    endpoint: String,
    executable: PathBuf,
    diagnostics: BufWriter<File>,
    server: JoinHandle<std::result::Result<(), tonic::transport::Error>>,
}

impl Transport {
    pub async fn new(output: &Path, identities: [Identity; 2]) -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("http://{}", listener.local_addr()?);
        let (tx, incoming) = mpsc::channel(32);
        let service = Service {
            expected: Arc::new(Mutex::new(identities.map(|identity| Registration {
                identity,
                incarnation: 0,
                claimed: None,
                attached: false,
            }))),
            incoming: tx,
        };
        let handler = RuntimeServer::new(service.clone())
            .max_decoding_message_size(MAX_FRAME_BYTES)
            .max_encoding_message_size(MAX_FRAME_BYTES);
        let server = tokio::spawn(
            Server::builder()
                .add_service(handler)
                .serve_with_incoming(TcpListenerStream::new(listener)),
        );
        let executable = std::env::current_exe()?
            .with_file_name(format!("replica-worker{}", std::env::consts::EXE_SUFFIX));
        require(
            executable.is_file(),
            "build replica-worker alongside sim-hub",
        )?;
        let mut transport = Self {
            service,
            incoming,
            streams: [None, None],
            processes: [None, None],
            history: [vec![], vec![]],
            started: Instant::now(),
            output: output.to_owned(),
            endpoint,
            executable,
            diagnostics: BufWriter::new(File::create(output.join("transport-diagnostics.jsonl"))?),
            server,
        };
        transport.diagnostic("Listen", json!({"endpoint": transport.endpoint}))?;
        Ok(transport)
    }

    pub fn diagnostic(&mut self, name: &str, detail: Value) -> Result<()> {
        serde_json::to_writer(
            &mut self.diagnostics,
            &json!({
                "kind": name, "wall_elapsed_us": self.started.elapsed().as_micros(), "detail": detail,
            }),
        )?;
        writeln!(self.diagnostics)?;
        self.diagnostics.flush()?;
        Ok(())
    }

    pub fn spawn(&mut self, launch: &Launch) -> Result<()> {
        let node = launch.identity.node_id as usize;
        require(self.processes[node].is_none(), "worker already running")?;
        {
            let mut expected = self.service.expected.lock().expect("registration mutex");
            let reg = &mut expected[node];
            require(
                reg.identity == launch.identity
                    && reg.incarnation.checked_add(1) == Some(launch.incarnation),
                "invalid process incarnation",
            )?;
            reg.incarnation = launch.incarnation;
            reg.claimed = None;
            reg.attached = false;
        }
        let dir = self.output.join(format!("worker-{node}"));
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("launch-{}.json", launch.incarnation));
        serde_json::to_writer(File::create(&path)?, launch)?;
        let stdout = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(dir.join(format!("stdout-{}.log", launch.incarnation)))?;
        let stderr = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(dir.join(format!("stderr-{}.log", launch.incarnation)))?;
        let child = Command::new(&self.executable)
            .arg(&self.endpoint)
            .arg(path)
            .kill_on_drop(true)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr)
            .spawn()?;
        let detail = json!({"node_id": node, "incarnation": launch.incarnation, "pid": child.id(), "wall_elapsed_us": self.started.elapsed().as_micros()});
        self.history[node].push(json!({"kind": "ProcessStarted", "detail": detail}));
        self.diagnostic("ProcessStarted", detail)?;
        self.processes[node] = Some(child);
        self.save_history(node)
    }

    pub fn attach(&mut self, node: u32, sender: StreamSender) -> Result<()> {
        require(
            self.streams[node as usize].is_none(),
            "worker stream already attached",
        )?;
        self.streams[node as usize] = Some(sender);
        Ok(())
    }

    pub fn try_receive(&mut self) -> Result<Option<Incoming>> {
        match self.incoming.try_recv() {
            Ok(frame) => Ok(Some(frame)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => {
                Err(invalid("incoming stream channel closed"))
            }
        }
    }

    pub async fn receive(&mut self, deadline: tokio::time::Instant) -> Result<Incoming> {
        tokio::time::timeout_at(deadline, self.incoming.recv())
            .await
            .map_err(|_| invalid("WallWatchdog elapsed (30 seconds)"))?
            .ok_or_else(|| invalid("incoming stream channel closed"))
    }

    pub async fn send(&self, node: u32, frame: Hub) -> Result<()> {
        let sender = self.streams[node as usize]
            .as_ref()
            .ok_or_else(|| invalid("worker stream absent"))?;
        tokio::time::timeout(WATCHDOG, sender.send(Ok(HubFrame { frame: Some(frame) })))
            .await
            .map_err(|_| invalid("WallWatchdog while sending"))?
            .map_err(|_| invalid("worker stream closed"))
    }

    pub async fn stop_and_reap(
        &mut self,
        node: u32,
        reason: &str,
        expected_code: i32,
    ) -> Result<()> {
        self.send(
            node,
            Hub::Shutdown(Shutdown {
                reason: reason.to_owned(),
            }),
        )
        .await?;
        self.reap(node, Some(expected_code), false).await
    }

    pub async fn reap(&mut self, node: u32, expected_code: Option<i32>, kill: bool) -> Result<()> {
        let index = node as usize;
        self.streams[index] = None;
        let mut child = self.processes[index]
            .take()
            .ok_or_else(|| invalid("worker process absent"))?;
        let pid = child.id();
        if kill {
            child.start_kill()?;
        }
        let status = tokio::time::timeout(WATCHDOG, child.wait())
            .await
            .map_err(|_| invalid("WallWatchdog while reaping child"))??;
        let detail = json!({"node_id": node, "pid": pid, "status": status.to_string(), "exit_code": status.code(), "wall_elapsed_us": self.started.elapsed().as_micros()});
        self.history[index].push(json!({"kind": "ProcessExited", "detail": detail}));
        self.diagnostic("ProcessExited", detail)?;
        self.save_history(index)?;
        if let Some(code) = expected_code {
            require(
                status.code() == Some(code),
                &format!("unexpected process exit for worker {node}: {status}"),
            )?;
        }
        Ok(())
    }

    pub fn is_live(&mut self, node: usize) -> Result<bool> {
        Ok(match self.processes[node].as_mut() {
            Some(child) => child.try_wait()?.is_none(),
            None => false,
        })
    }

    pub fn process_exists(&self, node: usize) -> bool {
        self.processes[node].is_some()
    }

    fn save_history(&self, node: usize) -> Result<()> {
        serde_json::to_writer_pretty(
            File::create(
                self.output
                    .join(format!("worker-{node}/process-history.json")),
            )?,
            &self.history[node],
        )?;
        Ok(())
    }
}

impl Drop for Transport {
    fn drop(&mut self) {
        self.server.abort();
    }
}

pub fn deadline() -> tokio::time::Instant {
    tokio::time::Instant::now() + WATCHDOG
}
