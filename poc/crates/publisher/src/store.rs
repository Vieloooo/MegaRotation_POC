use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use crate::CodecError;

const LOCK: &str = "writer.lock";
const MAIN: &str = "state.snapshot";
const TEMP: &str = "state.tmp";
pub(crate) const MAX_SNAPSHOT_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitStage {
    CreateDirectory,
    OpenLock,
    ReadMain,
    OpenDirectory,
    OpenTemp,
    WriteTemp,
    SyncTemp,
    Rename,
    SyncDirectory,
}

#[derive(Debug)]
pub enum StoreError {
    Io {
        stage: CommitStage,
        source: io::Error,
    },
    WriterActive,
    CommitUnknown(io::Error),
    Poisoned,
    Capacity,
    BindingMismatch,
    BadMagic,
    UnsupportedVersion,
    Malformed,
    Checksum,
    InvalidState(&'static str),
    DomainCodec(CodecError),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for StoreError {}

impl From<CodecError> for StoreError {
    fn from(error: CodecError) -> Self {
        Self::DomainCodec(error)
    }
}

pub(crate) struct FileStore {
    path: PathBuf,
    _lock: File,
    poisoned: bool,
    #[cfg(test)]
    pub(crate) fail_once: Option<CommitStage>,
}

impl FileStore {
    pub fn create(path: &Path) -> Result<Self, StoreError> {
        fs::create_dir(path).map_err(|source| io_error(CommitStage::CreateDirectory, source))?;
        let store = Self::acquire(path, true)?;
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        File::open(parent)
            .and_then(|dir| dir.sync_all())
            .map_err(|source| io_error(CommitStage::SyncDirectory, source))?;
        Ok(store)
    }

    pub fn open(path: &Path) -> Result<(Self, Vec<u8>), StoreError> {
        let store = Self::acquire(path, false)?;
        let main = File::open(path.join(MAIN))
            .map_err(|source| io_error(CommitStage::ReadMain, source))?;
        let mut bytes = Vec::new();
        main.take((MAX_SNAPSHOT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|source| io_error(CommitStage::ReadMain, source))?;
        if bytes.len() > MAX_SNAPSHOT_BYTES {
            return Err(StoreError::Capacity);
        }
        Ok((store, bytes))
    }

    fn acquire(path: &Path, create: bool) -> Result<Self, StoreError> {
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(create)
            .open(path.join(LOCK))
            .map_err(|source| io_error(CommitStage::OpenLock, source))?;
        match lock.try_lock() {
            Ok(()) => (),
            Err(std::fs::TryLockError::WouldBlock) => return Err(StoreError::WriterActive),
            Err(std::fs::TryLockError::Error(source)) => {
                return Err(io_error(CommitStage::OpenLock, source));
            }
        }
        Ok(Self {
            path: path.to_path_buf(),
            _lock: lock,
            poisoned: false,
            #[cfg(test)]
            fail_once: None,
        })
    }

    pub fn check_healthy(&self) -> Result<(), StoreError> {
        if self.poisoned {
            Err(StoreError::Poisoned)
        } else {
            Ok(())
        }
    }

    pub fn commit(&mut self, bytes: &[u8]) -> Result<(), StoreError> {
        self.check_healthy()?;
        if bytes.len() > MAX_SNAPSHOT_BYTES {
            return Err(StoreError::Capacity);
        }
        let dir = File::open(&self.path)
            .map_err(|source| io_error(CommitStage::OpenDirectory, source))?;
        let temp_path = self.path.join(TEMP);
        self.checkpoint(CommitStage::OpenTemp)
            .map_err(|source| io_error(CommitStage::OpenTemp, source))?;
        match fs::remove_file(&temp_path) {
            Ok(()) => (),
            Err(error) if error.kind() == io::ErrorKind::NotFound => (),
            Err(source) => return Err(io_error(CommitStage::OpenTemp, source)),
        }
        let mut temp = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(|source| io_error(CommitStage::OpenTemp, source))?;
        #[cfg(test)]
        if self.fail_once == Some(CommitStage::WriteTemp) {
            temp.write_all(&bytes[..bytes.len() / 2])
                .map_err(|source| io_error(CommitStage::WriteTemp, source))?;
            self.checkpoint(CommitStage::WriteTemp)
                .map_err(|source| io_error(CommitStage::WriteTemp, source))?;
        }
        temp.write_all(bytes)
            .map_err(|source| io_error(CommitStage::WriteTemp, source))?;
        self.checkpoint(CommitStage::SyncTemp)
            .and_then(|()| temp.sync_all())
            .map_err(|source| io_error(CommitStage::SyncTemp, source))?;
        drop(temp);
        self.checkpoint(CommitStage::Rename)
            .and_then(|()| fs::rename(&temp_path, self.path.join(MAIN)))
            .map_err(|source| io_error(CommitStage::Rename, source))?;
        if let Err(error) = self
            .checkpoint(CommitStage::SyncDirectory)
            .and_then(|()| dir.sync_all())
        {
            self.poisoned = true;
            return Err(StoreError::CommitUnknown(error));
        }
        Ok(())
    }

    fn checkpoint(&mut self, _stage: CommitStage) -> io::Result<()> {
        #[cfg(test)]
        if self.fail_once == Some(_stage) {
            self.fail_once = None;
            return Err(io::Error::other(format!("injected failure at {_stage:?}")));
        }
        Ok(())
    }
}

fn io_error(stage: CommitStage, source: io::Error) -> StoreError {
    StoreError::Io { stage, source }
}
