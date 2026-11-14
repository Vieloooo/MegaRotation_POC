use crate::{Id, driver::Shared};
use bytes::Buf;
use commonware_runtime::{
    Blob, BlobVersion, Error, Handle, IoBufs, IoBufsMut, ReadOptions, Storage, WriteOptions,
    tokio::Context as NativeContext,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fmt,
    future::Future,
    ops::RangeInclusive,
    pin::Pin,
    sync::{Arc, Weak},
};
use tokio::sync::oneshot;

pub(crate) type NativeBlob = <NativeContext as Storage>::Blob;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct IoIdentity {
    pub io_id: u64,
    pub grant: u64,
    pub tick_us: u64,
    pub run_id: Id,
    pub node: u32,
    pub incarnation: u64,
    pub cid: Id,
    pub task: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct IoTarget {
    pub partition: String,
    pub name: Option<Vec<u8>>,
    pub open_id: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum IoOperation {
    Open {
        min_version: u16,
        max_version: u16,
    },
    Remove,
    Scan,
    Read {
        offset: u64,
        len: usize,
        dont_cache: bool,
        reused_buffer: bool,
    },
    Write {
        offset: u64,
        len: usize,
        digest: Id,
        sync: bool,
        dont_cache: bool,
    },
    Resize {
        len: u64,
    },
    Sync {
        detached: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum IoResult {
    Open { len: u64, version: u16 },
    Names { names: Vec<Vec<u8>> },
    Bytes { len: usize, digest: Id },
    Ok,
    Error { kind: &'static str },
}

pub(crate) type Release = Box<dyn FnOnce(bool) -> bool + Send>;
type Physical = Pin<Box<dyn Future<Output = (IoResult, Release, Option<Arc<NativeBlob>>)> + Send>>;

pub(crate) struct WitnessIo {
    pub target: IoTarget,
    pub operation: IoOperation,
    pub submitted: Option<IoBufs>,
    pub native: Option<Arc<NativeBlob>>,
}

pub(crate) struct IoJob {
    pub identity: IoIdentity,
    pub future: Physical,
    pub witness: Option<WitnessIo>,
}

pub(crate) struct CompletedIo {
    pub identity: IoIdentity,
    pub result: IoResult,
    pub release: Release,
    pub witness: Option<WitnessIo>,
}

#[derive(Default)]
pub(crate) struct DiskQueue {
    pub active: Option<IoJob>,
    pub completed: Option<CompletedIo>,
}

impl IoJob {
    pub fn new<T: Send + 'static>(
        identity: IoIdentity,
        run: impl Future<Output = Result<T, Error>> + Send + 'static,
        sender: oneshot::Sender<Result<T, Error>>,
        summarize: fn(&T) -> IoResult,
        witness: Option<WitnessIo>,
        opened: fn(&T) -> Option<Arc<NativeBlob>>,
    ) -> Self {
        let inspect = witness.is_some();
        let future = Box::pin(async move {
            let result = run.await;
            let summary = match &result {
                Ok(value) => summarize(value),
                Err(error) => IoResult::Error {
                    kind: error_kind(error),
                },
            };
            let native = if inspect {
                result.as_ref().ok().and_then(opened)
            } else {
                None
            };
            let release: Release = Box::new(move |deliver| deliver && sender.send(result).is_ok());
            (summary, release, native)
        });
        Self {
            identity,
            future,
            witness,
        }
    }
}

#[derive(Clone)]
pub(crate) struct StorageAccess {
    native: Arc<NativeContext>,
    shared: Weak<Shared>,
    scope: Option<Id>,
}

impl StorageAccess {
    pub fn new(native: Arc<NativeContext>, shared: Weak<Shared>, cid: Id) -> Self {
        Self {
            native,
            shared,
            scope: Some(cid),
        }
    }

    fn shared(&self) -> Arc<Shared> {
        self.shared
            .upgrade()
            .expect("driver owns storage operations")
    }

    fn issue<T: Send + 'static>(
        &self,
        target: IoTarget,
        operation: IoOperation,
        work: impl FnOnce(IoIdentity) -> Pin<Box<dyn Future<Output = Result<T, Error>> + Send>>,
        summary: fn(&T) -> IoResult,
    ) -> oneshot::Receiver<Result<T, Error>> {
        self.issue_witness(target, operation, work, summary, None, None, |_| None)
    }

    fn issue_witness<T: Send + 'static>(
        &self,
        mut target: IoTarget,
        operation: IoOperation,
        work: impl FnOnce(IoIdentity) -> Pin<Box<dyn Future<Output = Result<T, Error>> + Send>>,
        summary: fn(&T) -> IoResult,
        native: Option<Arc<NativeBlob>>,
        submitted: Option<IoBufs>,
        opened: fn(&T) -> Option<Arc<NativeBlob>>,
    ) -> oneshot::Receiver<Result<T, Error>> {
        let shared = self.shared();
        let identity = shared.issue_io(self.scope, target.clone(), operation.clone());
        if matches!(operation, IoOperation::Open { .. }) {
            target.open_id = Some(identity.io_id);
        }
        let witness = shared
            .witness_partition(&target.partition)
            .then_some(WitnessIo {
                target,
                operation,
                submitted,
                native,
            });
        let (sender, receiver) = oneshot::channel();
        let job = IoJob::new(
            identity.clone(),
            work(identity),
            sender,
            summary,
            witness,
            opened,
        );
        shared.enqueue_io(job);
        receiver
    }

    pub async fn open(
        &self,
        partition: &str,
        name: &[u8],
        versions: RangeInclusive<BlobVersion>,
    ) -> Result<(ControlledBlob, u64, BlobVersion), Error> {
        let target = IoTarget {
            partition: partition.to_owned(),
            name: Some(name.to_vec()),
            open_id: None,
        };
        let operation = IoOperation::Open {
            min_version: versions.start().get(),
            max_version: versions.end().get(),
        };
        let access = self.clone();
        let address = target.clone();
        let receiver = self.issue_witness(
            target,
            operation,
            move |identity| {
                Box::pin(async move {
                    let (native, size, version) = access
                        .native
                        .open_versioned(
                            &address.partition,
                            address.name.as_deref().expect("open names a blob"),
                            versions,
                        )
                        .await?;
                    let target = IoTarget {
                        open_id: Some(identity.io_id),
                        ..address
                    };
                    Ok((
                        ControlledBlob {
                            native: Arc::new(native),
                            access: StorageAccess {
                                scope: None,
                                ..access
                            },
                            target,
                        },
                        size,
                        version,
                    ))
                })
            },
            |(_, len, version)| IoResult::Open {
                len: *len,
                version: version.get(),
            },
            None,
            None,
            |(blob, _, _)| Some(blob.native.clone()),
        );
        receive(receiver).await
    }

    pub async fn remove(&self, partition: &str, name: Option<&[u8]>) -> Result<(), Error> {
        let target = IoTarget {
            partition: partition.to_owned(),
            name: name.map(<[u8]>::to_vec),
            open_id: None,
        };
        let address = target.clone();
        let native = self.native.clone();
        receive(self.issue(
            target,
            IoOperation::Remove,
            move |_| {
                Box::pin(async move {
                    native
                        .remove(&address.partition, address.name.as_deref())
                        .await
                })
            },
            unit,
        ))
        .await
    }

    pub async fn scan(&self, partition: &str) -> Result<Vec<Vec<u8>>, Error> {
        let target = IoTarget {
            partition: partition.to_owned(),
            name: None,
            open_id: None,
        };
        let partition = target.partition.clone();
        let native = self.native.clone();
        receive(self.issue(
            target,
            IoOperation::Scan,
            move |_| {
                Box::pin(async move {
                    let mut names = native.scan(&partition).await?;
                    names.sort_unstable();
                    Ok(names)
                })
            },
            |names| IoResult::Names {
                names: names.clone(),
            },
        ))
        .await
    }
}

/// A single open native blob. Commonware may share it through its standard Arc<Blob>
/// implementation. The adapter's identity metadata holds no additional native open.
pub struct ControlledBlob {
    native: Arc<NativeBlob>,
    access: StorageAccess,
    target: IoTarget,
}

impl fmt::Debug for ControlledBlob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ControlledBlob")
            .field("target", &self.target)
            .finish_non_exhaustive()
    }
}

impl Blob for ControlledBlob {
    async fn read_at_buf(
        &self,
        offset: u64,
        len: usize,
        bufs: impl Into<IoBufsMut> + Send,
        options: ReadOptions,
    ) -> Result<IoBufsMut, Error> {
        let bufs = bufs.into();
        let op = read_op(offset, len, options, true);
        let native = self.native.clone();
        receive(self.access.issue(
            self.target.clone(),
            op,
            move |_| Box::pin(async move { native.read_at_buf(offset, len, bufs, options).await }),
            read_result,
        ))
        .await
    }

    async fn read_at(
        &self,
        offset: u64,
        len: usize,
        options: ReadOptions,
    ) -> Result<IoBufsMut, Error> {
        let op = read_op(offset, len, options, false);
        let native = self.native.clone();
        receive(self.access.issue(
            self.target.clone(),
            op,
            move |_| Box::pin(async move { native.read_at(offset, len, options).await }),
            read_result,
        ))
        .await
    }

    async fn write_at(
        &self,
        offset: u64,
        bufs: impl Into<IoBufs> + Send,
        options: WriteOptions,
    ) -> Result<(), Error> {
        let bufs = bufs.into();
        let submitted = self
            .access
            .shared()
            .witness_partition(&self.target.partition)
            .then(|| bufs.clone());
        let op = IoOperation::Write {
            offset,
            len: bufs.len(),
            digest: digest(&bufs),
            sync: options.contains(WriteOptions::SYNC),
            dont_cache: options.contains(WriteOptions::DONT_CACHE),
        };
        let native = self.native.clone();
        let inspect = native.clone();
        receive(self.access.issue_witness(
            self.target.clone(),
            op,
            move |_| Box::pin(async move { native.write_at(offset, bufs, options).await }),
            unit,
            Some(inspect),
            submitted,
            |_| None,
        ))
        .await
    }

    async fn resize(&self, len: u64) -> Result<(), Error> {
        let native = self.native.clone();
        receive(self.access.issue_witness(
            self.target.clone(),
            IoOperation::Resize { len },
            move |_| Box::pin(async move { native.resize(len).await }),
            unit,
            None,
            None,
            |_| None,
        ))
        .await
    }

    async fn sync(&self) -> Result<(), Error> {
        let native = self.native.clone();
        let inspect = native.clone();
        receive(self.access.issue_witness(
            self.target.clone(),
            IoOperation::Sync { detached: false },
            move |_| Box::pin(async move { native.sync().await }),
            unit,
            Some(inspect),
            None,
            |_| None,
        ))
        .await
    }

    async fn start_sync(&self) -> Handle<()> {
        let native = self.native.clone();
        let inspect = native.clone();
        let receiver = self.access.issue_witness(
            self.target.clone(),
            IoOperation::Sync { detached: true },
            move |_| Box::pin(async move { native.start_sync().await.await }),
            unit,
            Some(inspect),
            None,
            |_| None,
        );
        Handle::from_receiver(receiver)
    }
}

async fn receive<T>(receiver: oneshot::Receiver<Result<T, Error>>) -> Result<T, Error> {
    receiver.await.unwrap_or(Err(Error::Closed))
}

fn read_op(offset: u64, len: usize, options: ReadOptions, reused_buffer: bool) -> IoOperation {
    IoOperation::Read {
        offset,
        len,
        dont_cache: options.contains(ReadOptions::DONT_CACHE),
        reused_buffer,
    }
}

fn read_result(buf: &IoBufsMut) -> IoResult {
    let mut hash = Sha256::new();
    let mut chunks = vec![std::io::IoSlice::new(&[]); 4];
    loop {
        let count = buf.chunks_vectored(&mut chunks);
        if chunks[..count]
            .iter()
            .map(|chunk| chunk.len())
            .sum::<usize>()
            == buf.len()
        {
            break;
        }
        chunks.resize(chunks.len() * 2, std::io::IoSlice::new(&[]));
    }
    let count = buf.chunks_vectored(&mut chunks);
    for chunk in &chunks[..count] {
        hash.update(&chunk[..]);
    }
    IoResult::Bytes {
        len: buf.len(),
        digest: hash.finalize().into(),
    }
}

fn digest(buf: &IoBufs) -> Id {
    let mut hash = Sha256::new();
    buf.for_each_chunk(|chunk| hash.update(chunk));
    hash.finalize().into()
}

fn unit(_: &()) -> IoResult {
    IoResult::Ok
}

fn error_kind(error: &Error) -> &'static str {
    match error {
        Error::Closed => "closed",
        Error::Aborted => "aborted",
        Error::Exited => "exited",
        Error::ReadFailed => "read_failed",
        Error::WriteFailed => "write_failed",
        Error::PartitionNameInvalid(_) => "partition_name_invalid",
        Error::PartitionCreationFailed(_) => "partition_creation_failed",
        Error::PartitionMissing(_) => "partition_missing",
        Error::PartitionCorrupt(_) => "partition_corrupt",
        Error::BlobAlreadyOpen(..) => "blob_already_open",
        Error::BlobOpenFailed(..) => "blob_open_failed",
        Error::BlobMissing(..) => "blob_missing",
        Error::BlobResizeFailed(..) => "blob_resize_failed",
        Error::BlobSyncFailed(..) => "blob_sync_failed",
        Error::BlobInsufficientLength => "blob_insufficient_length",
        Error::BlobCorrupt(..) => "blob_corrupt",
        Error::BlobLayoutMismatch { .. } => "blob_layout_mismatch",
        Error::BlobVersionMismatch { .. } => "blob_version_mismatch",
        Error::InvalidChecksum => "invalid_checksum",
        Error::OffsetOverflow => "offset_overflow",
        Error::Io(_) => "io",
        _ => "other_native_error",
    }
}
