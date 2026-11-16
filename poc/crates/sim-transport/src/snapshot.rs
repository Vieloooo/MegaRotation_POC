use crate::Error;
use serde::{Serialize, de::DeserializeOwned};

pub(crate) const MAX_SNAPSHOT: usize = 32 * 1024 * 1024;
pub(crate) const SCHEMA: u32 = 1;

pub(crate) fn encode(value: &impl Serialize) -> Result<Vec<u8>, Error> {
    serde_json::to_vec(value).map_err(|_| Error::CorruptSnapshot)
}

pub(crate) fn encode_snapshot(value: &impl Serialize) -> Result<Vec<u8>, Error> {
    let bytes = encode(value)?;
    if bytes.len() > MAX_SNAPSHOT {
        return Err(Error::SnapshotTooLarge);
    }
    Ok(bytes)
}

pub(crate) fn decode_snapshot<T: DeserializeOwned + Serialize>(bytes: &[u8]) -> Result<T, Error> {
    if bytes.len() > MAX_SNAPSHOT {
        return Err(Error::SnapshotTooLarge);
    }
    let value = serde_json::from_slice(bytes).map_err(|_| Error::CorruptSnapshot)?;
    if encode(&value)? != bytes {
        return Err(Error::NonCanonical);
    }
    Ok(value)
}

pub(crate) fn schema(version: u32) -> Result<(), Error> {
    if version != SCHEMA {
        return Err(Error::UnsupportedSchema);
    }
    Ok(())
}

pub(crate) fn strictly_ordered<T, K: Ord>(values: &[T], key: impl Fn(&T) -> K) -> bool {
    values.windows(2).all(|pair| key(&pair[0]) < key(&pair[1]))
}
