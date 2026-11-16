//! Deterministic application of a block to an immutable parent state.

use poc_types::domain::{
    Account, Credit, DomainError, ExecutionState, MAX_TRANSACTION_BYTES, Metadata, Transfer,
    validate_dep, validate_transactions,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplyOutput {
    pub state: ExecutionState,
    pub kept: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionError {
    Malformed(DomainError),
    CreditOverflow { account: u8 },
}

impl From<DomainError> for ExecutionError {
    fn from(error: DomainError) -> Self {
        Self::Malformed(error)
    }
}

pub fn apply(
    parent: &ExecutionState,
    metadata: &Metadata,
    dep: &[Vec<u8>],
    transactions: &[Vec<u8>],
) -> Result<ApplyOutput, ExecutionError> {
    apply_with_byte_limit(parent, metadata, dep, transactions, MAX_TRANSACTION_BYTES)
}

pub fn apply_with_byte_limit(
    parent: &ExecutionState,
    metadata: &Metadata,
    dep: &[Vec<u8>],
    transactions: &[Vec<u8>],
    byte_limit: usize,
) -> Result<ApplyOutput, ExecutionError> {
    parent.validate()?;
    validate_dep(metadata, dep)?;
    validate_transactions(transactions, byte_limit)?;

    let mut state = parent.clone();
    state.metadata = metadata.clone();
    // validate_dep already checked the first item against metadata and all remaining credits.
    for raw in &dep[1..] {
        let Credit { to, amount } = Credit::decode_raw(raw)?;
        let mut account = account(&state, to);
        account.balance = account
            .balance
            .checked_add(amount)
            .ok_or(ExecutionError::CreditOverflow { account: to })?;
        set_account(&mut state, to, account);
    }

    let mut kept = Vec::new();
    for raw in transactions {
        if execute_transfer(&mut state, Transfer::decode_raw(raw)?) {
            kept.push(raw.clone());
        }
    }
    Ok(ApplyOutput { state, kept })
}

fn account(state: &ExecutionState, id: u8) -> Account {
    state.accounts.get(&id).cloned().unwrap_or(Account {
        nonce: 0,
        balance: 0,
    })
}

fn set_account(state: &mut ExecutionState, id: u8, account: Account) {
    if account.nonce == 0 && account.balance == 0 {
        state.accounts.remove(&id);
    } else {
        state.accounts.insert(id, account);
    }
}

fn execute_transfer(
    state: &mut ExecutionState,
    Transfer {
        from,
        to,
        nonce,
        amount,
    }: Transfer,
) -> bool {
    let mut sender = account(state, from);
    if nonce != sender.nonce || amount > sender.balance {
        return false;
    }
    let Some(next_nonce) = sender.nonce.checked_add(1) else {
        return false;
    };
    sender.nonce = next_nonce;

    if from != to {
        let mut recipient = account(state, to);
        let Some(next_balance) = recipient.balance.checked_add(amount) else {
            return false;
        };
        recipient.balance = next_balance;
        sender.balance -= amount;
        set_account(state, to, recipient);
    }
    set_account(state, from, sender);
    true
}
