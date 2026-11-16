//! Finite public inputs with independent finalization and delivery operations.

use crate::metadata::{check_drift, check_l1_header};
use poc_types::domain::{
    Attributes, ConfigRef, DepFixture, ExecutionAnchor, FinalizedL1Reader, Hash32, InputError,
    InputMissing, InputStatus, L1BlockRef, L1Fixture, L1FixtureRow, L1Header, Metadata,
    PublicConfig, empty_control_commitment,
};
use std::sync::Arc;

pub trait DepProvider {
    fn config_ref(&self) -> ConfigRef;
    fn at(&self, metadata: &Metadata) -> InputStatus<Vec<Vec<u8>>>;
}

#[derive(Clone, Debug)]
pub struct CheckedSetup {
    config: PublicConfig,
    genesis: ExecutionAnchor,
    dep: ValidatedDepFixture,
    l1: ValidatedFixture,
}

impl CheckedSetup {
    pub fn config(&self) -> &PublicConfig {
        &self.config
    }

    pub fn genesis(&self) -> &ExecutionAnchor {
        &self.genesis
    }

    pub fn dep(&self) -> &ValidatedDepFixture {
        &self.dep
    }

    pub fn l1(&self) -> &ValidatedFixture {
        &self.l1
    }
}

/// Bind the genesis state and both fixture commitments before constructing live readers.
pub fn check_setup(
    config: &PublicConfig,
    genesis: &ExecutionAnchor,
    dep: DepFixture,
    l1: L1Fixture,
) -> Result<CheckedSetup, InputError> {
    genesis.validate().map_err(InputError::Malformed)?;
    if genesis.anchor != config.genesis
        || genesis.state.root().map_err(InputError::Malformed)? != config.genesis_state_root
        || genesis.native_hash != config.genesis_native_hash
    {
        return Err(InputError::InvalidSource);
    }

    let dep = ValidatedDepFixture::load(config, dep)?;
    let l1 = ValidatedFixture::load(config, l1)?;
    let row = l1.row(genesis.anchor.block.origin)?;
    let l1_timestamp = row.header.timestamp_us().map_err(InputError::Malformed)?;
    check_drift(l1_timestamp, genesis.anchor.timestamp_us, config.t_drift_us)?;
    Ok(CheckedSetup {
        config: config.clone(),
        genesis: genesis.clone(),
        dep,
        l1,
    })
}

#[derive(Clone, Debug)]
pub struct ValidatedDepFixture {
    config_ref: ConfigRef,
    fixture: Arc<DepFixture>,
    last_height: u64,
}

impl ValidatedDepFixture {
    pub fn load(config: &PublicConfig, fixture: DepFixture) -> Result<Self, InputError> {
        let config_ref = config.reference().map_err(InputError::Malformed)?;
        if fixture.digest().map_err(InputError::Malformed)? != config.dep_fixture_hash {
            return Err(InputError::FixtureCorrupt);
        }
        let last_height = fixture.first_height + (fixture.rows.len() - 1) as u64;
        Ok(Self {
            config_ref,
            fixture: Arc::new(fixture),
            last_height,
        })
    }
}

impl DepProvider for ValidatedDepFixture {
    fn config_ref(&self) -> ConfigRef {
        self.config_ref
    }

    fn at(&self, metadata: &Metadata) -> InputStatus<Vec<Vec<u8>>> {
        let first = self.fixture.first_height;
        let height = metadata.height;
        if !(first..=self.last_height).contains(&height) {
            return InputStatus::Reject(InputError::FixtureExhausted {
                requested: height,
                first,
                last: self.last_height,
            });
        }
        let Some(credits) = self.fixture.rows.get((height - first) as usize) else {
            return InputStatus::Reject(InputError::FixtureCorrupt);
        };
        let mut dep = Vec::with_capacity(1 + credits.len());
        dep.push(
            Attributes {
                metadata: metadata.clone(),
            }
            .encode_raw(),
        );
        dep.extend(credits.iter().map(|credit| credit.encode_raw()));
        InputStatus::Ready(dep)
    }
}

#[derive(Clone, Debug)]
pub struct ValidatedFixture {
    config_ref: ConfigRef,
    fixture: Arc<L1Fixture>,
}

impl ValidatedFixture {
    pub fn load(config: &PublicConfig, fixture: L1Fixture) -> Result<Self, InputError> {
        let config_ref = config.reference().map_err(InputError::Malformed)?;
        if fixture.digest().map_err(InputError::Malformed)? != config.l1_fixture_hash {
            return Err(InputError::FixtureCorrupt);
        }
        for row in &fixture.rows {
            check_l1_header(config, row.header.height, &row.header)?;
        }
        Ok(Self {
            config_ref,
            fixture: Arc::new(fixture),
        })
    }

    /// Make the source actor that accepts explicit finalization events.
    pub fn finalizer(&self) -> FixtureFinalizer {
        FixtureFinalizer {
            source: self.clone(),
            finalized_at: Vec::new(),
        }
    }

    /// Start an independent reader with an empty delivery prefix.
    pub fn reader(&self) -> FixtureReader {
        FixtureReader {
            source: self.clone(),
            receipts: vec![RowReceipt::default(); self.fixture.rows.len()],
            header_prefix_len: 0,
            control_prefix_len: 0,
        }
    }

    fn index(&self, height: u64) -> Result<usize, InputError> {
        let last = (self.fixture.rows.len() - 1) as u64;
        if height > last {
            return Err(InputError::FixtureExhausted {
                requested: height,
                first: 0,
                last,
            });
        }
        Ok(height as usize)
    }

    fn row(&self, height: u64) -> Result<&L1FixtureRow, InputError> {
        let index = self.index(height)?;
        self.fixture
            .rows
            .get(index)
            .ok_or(InputError::FixtureCorrupt)
    }
}

/// Per-run authority for finalizing the mock source. Finalization follows the source prefix.
pub struct FixtureFinalizer {
    source: ValidatedFixture,
    finalized_at: Vec<u64>,
}

/// Evidence produced by an explicit accepted finalization event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FinalizedFixtureRow {
    config_ref: ConfigRef,
    header: L1Header,
    finalized_at_us: u64,
}

impl FinalizedFixtureRow {
    pub fn header(&self) -> &L1Header {
        &self.header
    }

    pub fn finalized_at_us(&self) -> u64 {
        self.finalized_at_us
    }
}

impl FixtureFinalizer {
    pub fn finalize(
        &mut self,
        header: &L1Header,
        finalized_at_us: u64,
    ) -> Result<FinalizedFixtureRow, InputError> {
        let index = self.source.index(header.height)?;
        let row = self.source.row(header.height)?;
        if row.header != *header {
            return Err(InputError::Conflict);
        }
        if finalized_at_us < row.finalized_at_us {
            return Err(InputError::InvalidSource);
        }

        let actual_finalized_at = if let Some(previous) = self.finalized_at.get(index) {
            *previous
        } else {
            if index != self.finalized_at.len()
                || self
                    .finalized_at
                    .last()
                    .is_some_and(|last| finalized_at_us < *last)
            {
                return Err(InputError::InvalidSource);
            }
            self.finalized_at.push(finalized_at_us);
            finalized_at_us
        };
        Ok(FinalizedFixtureRow {
            config_ref: self.source.config_ref,
            header: header.clone(),
            finalized_at_us: actual_finalized_at,
        })
    }
}

#[derive(Clone, Debug, Default)]
struct RowReceipt {
    finalized_at_us: Option<u64>,
    header: bool,
    controls: bool,
}

/// Reader state advances only through accepted, explicit delivery events.
#[derive(Clone, Debug)]
pub struct FixtureReader {
    source: ValidatedFixture,
    receipts: Vec<RowReceipt>,
    header_prefix_len: usize,
    control_prefix_len: usize,
}

impl FixtureReader {
    pub fn deliver_header(
        &mut self,
        finalized: &FinalizedFixtureRow,
        delivered_at_us: u64,
    ) -> Result<(), InputError> {
        let index = self.check_delivery(finalized, delivered_at_us)?;
        self.receipts[index].finalized_at_us = Some(finalized.finalized_at_us);
        self.receipts[index].header = true;
        self.advance_prefixes();
        Ok(())
    }

    pub fn deliver_empty_control(
        &mut self,
        finalized: &FinalizedFixtureRow,
        commitment: Hash32,
        delivered_at_us: u64,
    ) -> Result<(), InputError> {
        let index = self.check_delivery(finalized, delivered_at_us)?;
        if commitment != self.source.fixture.rows[index].empty_control_commitment {
            return Err(InputError::Conflict);
        }
        self.receipts[index].finalized_at_us = Some(finalized.finalized_at_us);
        self.receipts[index].controls = true;
        self.advance_prefixes();
        Ok(())
    }

    fn check_delivery(
        &self,
        finalized: &FinalizedFixtureRow,
        delivered_at_us: u64,
    ) -> Result<usize, InputError> {
        let index = self.source.index(finalized.header.height)?;
        if finalized.config_ref != self.source.config_ref
            || finalized.header != self.source.fixture.rows[index].header
            || delivered_at_us < finalized.finalized_at_us
        {
            return Err(InputError::InvalidSource);
        }
        if self.receipts[index]
            .finalized_at_us
            .is_some_and(|prior| prior != finalized.finalized_at_us)
        {
            return Err(InputError::Conflict);
        }
        Ok(index)
    }

    fn advance_prefixes(&mut self) {
        while self
            .receipts
            .get(self.header_prefix_len)
            .is_some_and(|receipt| receipt.header)
        {
            self.header_prefix_len += 1;
        }
        while self.control_prefix_len < self.header_prefix_len
            && self.receipts[self.control_prefix_len].controls
        {
            self.control_prefix_len += 1;
        }
    }
}

impl FinalizedL1Reader for FixtureReader {
    fn header(&self, height: u64) -> InputStatus<L1Header> {
        match self.source.index(height) {
            Ok(index) if index < self.header_prefix_len => {
                InputStatus::Ready(self.source.fixture.rows[index].header.clone())
            }
            Ok(_) => InputStatus::Wait(InputMissing::L1Header(self.header_prefix_len as u64)),
            Err(error) => InputStatus::Reject(error),
        }
    }

    fn require_control_coverage(&self, through_height: u64) -> InputStatus<()> {
        match self.source.index(through_height) {
            Ok(index) if index < self.control_prefix_len => InputStatus::Ready(()),
            Ok(_) => InputStatus::Wait(InputMissing::ControlCoverage(
                self.control_prefix_len as u64,
            )),
            Err(error) => InputStatus::Reject(error),
        }
    }

    fn finalized_tip(&self) -> Option<L1BlockRef> {
        let index = self.header_prefix_len.checked_sub(1)?;
        Some(self.source.fixture.rows[index].header.reference())
    }
}

/// Default finite REG-A source. The headers' IDs are deterministic fixture values.
pub fn reg_a_l1_fixture() -> L1Fixture {
    let mut parent_hash = Hash32::new([0; 32]);
    let empty_control_commitment = empty_control_commitment();
    let rows = (0..=30)
        .map(|height| {
            let hash = Hash32::new([0x40 + height as u8; 32]);
            let timestamp_s = 2 * height;
            let header = L1Header {
                height,
                hash,
                parent_hash,
                timestamp_s,
            };
            parent_hash = hash;
            L1FixtureRow {
                header,
                finalized_at_us: timestamp_s * 1_000_000 + 1_500_000,
                empty_control_commitment,
            }
        })
        .collect();
    L1Fixture { rows }
}
