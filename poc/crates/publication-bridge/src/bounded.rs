use poc_types::{domain::Hash32, hash_parts};

use crate::BridgeError;

pub(crate) fn digest(domain: &str, parts: &[&[u8]]) -> Hash32 {
    Hash32::try_from(hash_parts(domain, parts).as_slice()).expect("SHA-256 length")
}

pub(crate) fn identity(value: &[u8], max: usize) -> Result<(), BridgeError> {
    if (1..=max).contains(&value.len()) {
        Ok(())
    } else {
        Err(BridgeError::SourceMismatch)
    }
}

pub(crate) struct Writer {
    pub(crate) bytes: Vec<u8>,
    limit: usize,
}

impl Writer {
    pub(crate) fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
        }
    }

    pub(crate) fn push(&mut self, data: &[u8]) -> Result<(), BridgeError> {
        if data.len() > self.limit - self.bytes.len() {
            return Err(BridgeError::Capacity);
        }
        self.bytes.extend_from_slice(data);
        Ok(())
    }

    pub(crate) fn tag(&mut self, tag: u8) -> Result<(), BridgeError> {
        self.push(&[tag])
    }

    pub(crate) fn count(&mut self, count: usize) -> Result<(), BridgeError> {
        self.push(
            &u32::try_from(count)
                .map_err(|_| BridgeError::Capacity)?
                .to_be_bytes(),
        )
    }

    pub(crate) fn frame(&mut self, data: &[u8]) -> Result<(), BridgeError> {
        self.count(data.len())?;
        self.push(data)
    }
}

pub(crate) struct Reader<'a>(pub(crate) &'a [u8]);

impl<'a> Reader<'a> {
    pub(crate) fn take(&mut self, n: usize) -> Result<&'a [u8], BridgeError> {
        let (head, tail) = self
            .0
            .split_at_checked(n)
            .ok_or(BridgeError::InvalidEncoding)?;
        self.0 = tail;
        Ok(head)
    }

    pub(crate) fn fixed<const N: usize>(&mut self) -> Result<[u8; N], BridgeError> {
        self.take(N)?
            .try_into()
            .map_err(|_| BridgeError::InvalidEncoding)
    }

    pub(crate) fn tag(&mut self) -> Result<u8, BridgeError> {
        Ok(self.fixed::<1>()?[0])
    }

    pub(crate) fn u64(&mut self) -> Result<u64, BridgeError> {
        Ok(u64::from_be_bytes(self.fixed()?))
    }

    pub(crate) fn count(&mut self, max: usize) -> Result<usize, BridgeError> {
        let value = usize::try_from(u32::from_be_bytes(self.fixed()?))
            .map_err(|_| BridgeError::Capacity)?;
        if value > max {
            Err(BridgeError::Capacity)
        } else {
            Ok(value)
        }
    }

    pub(crate) fn frame(&mut self, max: usize) -> Result<&'a [u8], BridgeError> {
        let len = self.count(max)?;
        self.take(len)
    }

    pub(crate) fn finish(&self) -> Result<(), BridgeError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(BridgeError::InvalidEncoding)
        }
    }
}
