//! Regular replay adapter for the finite, independently delivered 03 L1 fixture.

use poc_core::fixture::{FinalizedFixtureRow, FixtureReader};
use poc_types::domain::{DomainError, FinalizedL1Reader, L1BlockRef, empty_control_commitment};
use sim_espresso::MAX_SOURCE_BYTES;

use crate::{ControlRef, GfrError, InputError, InputStatus, L1ControlReader, L1Header};

pub struct FixtureL1Reader {
    source_id: Vec<u8>,
    reader: FixtureReader,
}

impl FixtureL1Reader {
    pub fn new(source_id: Vec<u8>, reader: FixtureReader) -> Result<Self, GfrError> {
        validate_source_id(&source_id)?;
        Ok(Self { source_id, reader })
    }

    pub fn deliver_header(
        &mut self,
        row: &FinalizedFixtureRow,
        received_at_us: u64,
    ) -> Result<(), InputError> {
        self.reader.deliver_header(row, received_at_us)
    }

    pub fn deliver_empty_control(
        &mut self,
        row: &FinalizedFixtureRow,
        received_at_us: u64,
    ) -> Result<(), InputError> {
        self.reader
            .deliver_empty_control(row, empty_control_commitment(), received_at_us)
    }
}

impl FinalizedL1Reader for FixtureL1Reader {
    fn header(&self, height: u64) -> InputStatus<L1Header> {
        self.reader.header(height)
    }

    fn require_control_coverage(&self, through_height: u64) -> InputStatus<()> {
        self.reader.require_control_coverage(through_height)
    }

    fn finalized_tip(&self) -> Option<L1BlockRef> {
        self.reader.finalized_tip()
    }
}

impl L1ControlReader for FixtureL1Reader {
    fn source_id(&self) -> &[u8] {
        &self.source_id
    }

    fn controls_at(&self, height: u64) -> InputStatus<Vec<ControlRef>> {
        match self.reader.require_control_coverage(height) {
            InputStatus::Ready(()) => InputStatus::Ready(Vec::new()),
            InputStatus::Wait(missing) => InputStatus::Wait(missing),
            InputStatus::Reject(error) => InputStatus::Reject(error),
        }
    }
}

pub(crate) fn validate_source_id(id: &[u8]) -> Result<(), GfrError> {
    if id.is_empty() {
        return Err(GfrError::InvalidSource);
    }
    if id.len() > MAX_SOURCE_BYTES {
        return Err(DomainError::LimitExceeded {
            limit: MAX_SOURCE_BYTES,
            actual: id.len(),
        }
        .into());
    }
    Ok(())
}
