//! One ordered writer per role. Each published file and its directory are synced.

use crate::{
    accepted_view::ReadError,
    crypto::{Address, InboxDomain, InboxSigners},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceConfig {
    pub inbox: InboxDomain,
    pub signers: InboxSigners,
    pub submitter: Address,
    pub source_fixture: String,
}

impl ServiceConfig {
    fn validate(&self) -> Result<(), ReadError> {
        self.inbox.separator()?;
        if self.source_fixture.is_empty()
            || self.submitter == [0; 20]
            || self.signers.da == [0; 20]
            || self.signers.certifier == [0; 20]
            || self.signers.da == self.signers.certifier
        {
            return Err(reject("invalid service configuration"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FaultPoint {
    DaBeforeBytes,
    DaBytesDurable,
    DaReceiptDurable,
    CertifierBeforeSignaturePersist,
    CertifierSignatureDurable,
    BatcherBuiltDurable,
    BatcherDaDurable,
    BatcherCertifiedDurable,
    BatcherRawDurable,
    BatcherAcceptedDurable,
}

pub type CrashHook = fn(FaultPoint);
pub fn no_crash(_: FaultPoint) {}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Identity {
    version: String,
    role: String,
    config: ServiceConfig,
}

pub(crate) struct Root {
    pub(crate) path: PathBuf,
    _lock: File,
}

impl Root {
    pub(crate) fn open(path: &Path, role: &str, config: &ServiceConfig) -> Result<Self, ReadError> {
        config.validate()?;
        create_dirs(path).map_err(io_error)?;
        let parent = path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        sync_dir(parent).map_err(io_error)?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path.join("actor.lock"))
            .map_err(io_error)?;
        lock.try_lock()
            .map_err(|error| ReadError::Infrastructure(format!("role root is locked: {error}")))?;
        let root = Self {
            path: path.into(),
            _lock: lock,
        };
        let identity = Identity {
            version: "BS-COMPONENT-v1/BS-FRAME-v1".into(),
            role: role.into(),
            config: config.clone(),
        };
        if let Some(saved) = root.read_json::<Identity>("identity.json")? {
            if saved != identity {
                return Err(reject("durable identity mismatch"));
            }
            root.confirm("identity.json")?;
        } else {
            for entry in fs::read_dir(path).map_err(io_error)? {
                let name = entry.map_err(io_error)?.file_name();
                if name != "actor.lock" && !name.to_string_lossy().starts_with(".tmp-") {
                    return Err(reject("nonempty durable root without identity"));
                }
            }
            root.publish("identity.json", &json_bytes(&identity)?, false)?;
        }
        Ok(root)
    }

    pub(crate) fn read(&self, name: &str, limit: u64) -> Result<Option<Vec<u8>>, ReadError> {
        let file = match File::open(self.path.join(name)) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(io_error(error)),
        };
        let mut bytes = Vec::new();
        file.take(limit + 1)
            .read_to_end(&mut bytes)
            .map_err(io_error)?;
        if bytes.len() as u64 > limit {
            return Err(reject("durable record exceeds size limit"));
        }
        Ok(Some(bytes))
    }

    pub(crate) fn read_json<T: DeserializeOwned>(
        &self,
        name: &str,
    ) -> Result<Option<T>, ReadError> {
        self.read(name, 64 * 1024 * 1024)?
            .map(|bytes| {
                serde_json::from_slice(&bytes)
                    .map_err(|error| reject(&format!("invalid durable record {name}: {error}")))
            })
            .transpose()
    }

    pub(crate) fn publish(&self, name: &str, bytes: &[u8], replace: bool) -> Result<(), ReadError> {
        static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
        let (temp, mut file) = loop {
            let temp = self.path.join(format!(
                ".tmp-{}-{}",
                std::process::id(),
                NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
            ));
            match OpenOptions::new().write(true).create_new(true).open(&temp) {
                Ok(file) => break (temp, file),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(io_error(error)),
            }
        };
        let result = (|| {
            file.write_all(bytes)?;
            file.sync_all()?;
            let output = self.path.join(name);
            if replace {
                fs::rename(&temp, &output)?;
            } else {
                fs::hard_link(&temp, &output)?;
                fs::remove_file(&temp)?;
            }
            sync_dir(&self.path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(temp);
        }
        result.map_err(io_error)
    }

    /// Re-establish the durability barrier before reusing a file after an interrupted publish.
    pub(crate) fn confirm(&self, name: &str) -> Result<(), ReadError> {
        File::open(self.path.join(name))
            .and_then(|file| file.sync_all())
            .map_err(io_error)?;
        sync_dir(&self.path).map_err(io_error)
    }
}

fn create_dirs(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        return Ok(());
    }
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    create_dirs(parent)?;
    match fs::create_dir(path) {
        Ok(()) => (),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists && path.is_dir() => (),
        Err(error) => return Err(error),
    }
    sync_dir(path)?;
    sync_dir(parent)
}

fn sync_dir(path: &Path) -> io::Result<()> {
    File::open(path)?.sync_all()
}

pub(crate) fn reject(reason: &str) -> ReadError {
    ReadError::Reject(reason.into())
}
pub(crate) fn wait(reason: &str) -> ReadError {
    ReadError::Wait(reason.into())
}
fn io_error(error: io::Error) -> ReadError {
    ReadError::Infrastructure(error.to_string())
}

pub(crate) fn json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, ReadError> {
    serde_json::to_vec(value).map_err(|error| ReadError::Infrastructure(error.to_string()))
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub(crate) struct CommandBook(BTreeMap<u64, Value>);

impl CommandBook {
    pub(crate) fn register(&mut self, id: u64, complete_input: Value) -> Result<(), ReadError> {
        if let Some(existing) = self.0.get(&id) {
            return if existing == &complete_input {
                Ok(())
            } else {
                Err(reject("command id content conflict"))
            };
        }
        if self.0.last_key_value().is_some_and(|(&last, _)| id <= last) {
            return Err(reject("command id out of order"));
        }
        self.0.insert(id, complete_input);
        Ok(())
    }
}

pub(crate) struct Journal<T> {
    pub(crate) root: Root,
    pub(crate) state: T,
    poisoned: bool,
}

impl<T: Default + Clone + Serialize + DeserializeOwned> Journal<T> {
    pub(crate) fn open(root: Root) -> Result<Self, ReadError> {
        let state = if let Some(state) = root.read_json("state.json")? {
            root.confirm("state.json")?;
            state
        } else {
            if root.read("initialized", 8)?.is_some() {
                return Err(reject("initialized role state missing"));
            }
            let state = T::default();
            root.publish("state.json", &json_bytes(&state)?, false)?;
            state
        };
        if root.read("initialized", 8)?.is_none() {
            root.publish("initialized", b"1", false)?;
        }
        Ok(Self {
            root,
            state,
            poisoned: false,
        })
    }

    pub(crate) fn commit(&mut self, next: T) -> Result<(), ReadError> {
        if self.poisoned {
            return Err(ReadError::Infrastructure(
                "durable journal must be reopened after a write failure".into(),
            ));
        }
        let bytes = json_bytes(&next)?;
        if bytes.len() > 64 * 1024 * 1024 {
            return Err(reject("journal capacity exceeded"));
        }
        if let Err(error) = self.root.publish("state.json", &bytes, true) {
            self.poisoned = true;
            return Err(error);
        }
        self.state = next;
        Ok(())
    }
}
