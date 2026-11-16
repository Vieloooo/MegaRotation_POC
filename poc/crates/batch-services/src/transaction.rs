//! Legacy EIP-155 transport for the private P0 Anvil driver.

use crate::{
    crypto::{Address, CryptoError, SigningKey},
    frame::commitment,
};
use poc_types::domain::Hash32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyTerms {
    pub nonce: u64,
    pub gas_price: u64,
    pub gas_limit: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedTransaction {
    nonce: u64,
    hash: Hash32,
    raw: Vec<u8>,
}

impl PreparedTransaction {
    pub fn nonce(&self) -> u64 {
        self.nonce
    }
    pub fn hash(&self) -> Hash32 {
        self.hash
    }
    pub fn raw(&self) -> &[u8] {
        &self.raw
    }
}

pub fn sign_legacy(
    key: &SigningKey,
    chain_id: u64,
    to: Option<Address>,
    terms: LegacyTerms,
    data: &[u8],
) -> Result<PreparedTransaction, CryptoError> {
    let mut fields = vec![
        uint(terms.nonce),
        uint(terms.gas_price),
        uint(terms.gas_limit),
        rlp_bytes(to.as_ref().map_or(&[] as &[u8], |address| address)),
        uint(0),
        rlp_bytes(data),
    ];
    fields.extend([uint(chain_id), uint(0), uint(0)]);
    let signature = key.sign_digest(commitment(&list(&fields)))?;
    let v = chain_id
        .checked_mul(2)
        .and_then(|base| base.checked_add(u64::from(signature[64]) + 8))
        .ok_or(CryptoError::InvalidDomain)?;
    fields.truncate(6);
    fields.extend([
        uint(v),
        rlp_bytes(strip_zero(&signature[..32])),
        rlp_bytes(strip_zero(&signature[32..64])),
    ]);
    let raw = list(&fields);
    Ok(PreparedTransaction {
        nonce: terms.nonce,
        hash: commitment(&raw),
        raw,
    })
}

/// Address of the contract created by an account/contract at a given nonce.
pub fn create_address(deployer: Address, nonce: u64) -> Address {
    commitment(&list(&[rlp_bytes(&deployer), uint(nonce)])).as_bytes()[12..]
        .try_into()
        .expect("fixed address width")
}

fn uint(value: u64) -> Vec<u8> {
    rlp_bytes(strip_zero(&value.to_be_bytes()))
}

fn list(fields: &[Vec<u8>]) -> Vec<u8> {
    let size: usize = fields.iter().map(Vec::len).sum();
    let mut output = length_prefix(size, 0xc0);
    output.extend(fields.iter().flatten());
    output
}

fn rlp_bytes(bytes: &[u8]) -> Vec<u8> {
    if bytes.len() == 1 && bytes[0] < 0x80 {
        return bytes.to_vec();
    }
    let mut output = length_prefix(bytes.len(), 0x80);
    output.extend(bytes);
    output
}

fn length_prefix(len: usize, base: u8) -> Vec<u8> {
    if len < 56 {
        return vec![base + len as u8];
    }
    let len = len.to_be_bytes();
    let len = strip_zero(&len);
    let mut output = vec![base + 55 + len.len() as u8];
    output.extend(len);
    output
}

fn strip_zero(data: &[u8]) -> &[u8] {
    &data[data
        .iter()
        .position(|&byte| byte != 0)
        .unwrap_or(data.len())..]
}
