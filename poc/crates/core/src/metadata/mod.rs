//! Fixed L2 clock, production origin selection, and actual-origin validation.

use poc_types::domain::{
    Anchor, ConfigRef, DomainError, FinalizedL1Reader, InputError, InputStatus, L1Header, Metadata,
    PublicConfig,
};

pub trait MetadataProvider {
    fn config_ref(&self) -> ConfigRef;
    fn at(&self, anchor: &Anchor, height: u64) -> InputStatus<Metadata>;
}

pub struct RegionalMetadata<'a> {
    config: &'a PublicConfig,
    config_ref: ConfigRef,
    l1: &'a dyn FinalizedL1Reader,
}

impl<'a> RegionalMetadata<'a> {
    pub fn new(
        config: &'a PublicConfig,
        l1: &'a dyn FinalizedL1Reader,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            config,
            config_ref: config.reference()?,
            l1,
        })
    }

    fn candidate(&self, anchor: &Anchor, height: u64) -> Result<Metadata, DomainError> {
        let advance = height
            .checked_sub(anchor.block.height)
            .ok_or(malformed("height before anchor"))?;
        if anchor.timestamp_us != l2_timestamp_us(self.config, anchor.block.height)? {
            return Err(malformed("anchor timestamp"));
        }
        if advance == 0 {
            return Ok(anchor.metadata());
        }

        let timestamp_us = l2_timestamp_us(self.config, height)?;
        let scheduled = scheduled_origin(self.config, height)?;
        let anchor_origin = anchor.block.origin;
        let origin_limit = anchor_origin
            .checked_add(advance)
            .ok_or(DomainError::Overflow)?;
        let origin = origin_limit.min(anchor_origin.max(scheduled));
        Ok(Metadata {
            height,
            timestamp_us,
            origin,
        })
    }
}

impl MetadataProvider for RegionalMetadata<'_> {
    fn config_ref(&self) -> ConfigRef {
        self.config_ref
    }

    fn at(&self, anchor: &Anchor, height: u64) -> InputStatus<Metadata> {
        let metadata = match self.candidate(anchor, height) {
            Ok(metadata) => metadata,
            Err(error) => return InputStatus::Reject(InputError::Malformed(error)),
        };
        if metadata.height != anchor.block.height {
            match visible_origin(self.config, self.l1, &anchor.metadata()) {
                InputStatus::Ready(()) => {}
                InputStatus::Wait(missing) => return InputStatus::Wait(missing),
                InputStatus::Reject(error) => return InputStatus::Reject(error),
            }
        }
        finish_metadata(self.config, self.l1, metadata)
    }
}

pub fn l2_timestamp_us(config: &PublicConfig, height: u64) -> Result<u64, DomainError> {
    let delta = height
        .checked_sub(config.genesis.block.height)
        .ok_or(malformed("height before genesis"))?;
    let elapsed = delta
        .checked_mul(config.l2_slot_us)
        .ok_or(DomainError::Overflow)?;
    config
        .genesis
        .timestamp_us
        .checked_add(elapsed)
        .ok_or(DomainError::Overflow)
}

pub fn scheduled_origin(config: &PublicConfig, height: u64) -> Result<u64, DomainError> {
    let earliest = config
        .l1_genesis_timestamp_us
        .checked_add(config.t_dep_us)
        .ok_or(DomainError::Overflow)?;
    let eligible_elapsed = l2_timestamp_us(config, height)?
        .checked_sub(earliest)
        .ok_or(malformed("no scheduled L1 origin"))?;
    eligible_elapsed
        .checked_div(config.l1_block_us)
        .ok_or(malformed("zero L1 block interval"))
}

/// Validate a caller-selected origin, such as an origin selected by L1 derivation.
pub fn actual_metadata(
    config: &PublicConfig,
    reader: &dyn FinalizedL1Reader,
    previous: &Metadata,
    height: u64,
    origin: u64,
) -> InputStatus<Metadata> {
    if let Err(error) = config.validate() {
        return InputStatus::Reject(InputError::Malformed(error));
    }
    if previous.height.checked_add(1) != Some(height)
        || !origin
            .checked_sub(previous.origin)
            .is_some_and(|advance| advance <= 1)
    {
        return InputStatus::Reject(InputError::Malformed(malformed("origin continuity")));
    }
    match l2_timestamp_us(config, height) {
        Ok(timestamp_us) => finish_metadata(
            config,
            reader,
            Metadata {
                height,
                timestamp_us,
                origin,
            },
        ),
        Err(error) => InputStatus::Reject(InputError::Malformed(error)),
    }
}

pub(crate) fn check_l1_header(
    config: &PublicConfig,
    height: u64,
    header: &L1Header,
) -> Result<u64, InputError> {
    if header.height != height {
        return Err(InputError::InvalidSource);
    }
    let expected = height
        .checked_mul(config.l1_block_us)
        .and_then(|elapsed| config.l1_genesis_timestamp_us.checked_add(elapsed))
        .ok_or(InputError::Malformed(DomainError::Overflow))?;
    let timestamp_us = header.timestamp_us().map_err(InputError::Malformed)?;
    if timestamp_us != expected {
        return Err(InputError::InvalidSource);
    }
    Ok(timestamp_us)
}

pub(crate) fn check_drift(
    l1_timestamp_us: u64,
    l2_timestamp_us: u64,
    max_drift_us: u64,
) -> Result<(), InputError> {
    if !l2_timestamp_us
        .checked_sub(l1_timestamp_us)
        .is_some_and(|drift| drift <= max_drift_us)
    {
        return Err(InputError::Malformed(malformed(
            "origin timestamp outside drift bound",
        )));
    }
    Ok(())
}

fn visible_origin(
    config: &PublicConfig,
    reader: &dyn FinalizedL1Reader,
    metadata: &Metadata,
) -> InputStatus<()> {
    let header = match reader.header(metadata.origin) {
        InputStatus::Ready(header) => header,
        InputStatus::Wait(missing) => return InputStatus::Wait(missing),
        InputStatus::Reject(error) => return InputStatus::Reject(error),
    };
    match check_l1_header(config, metadata.origin, &header).and_then(|timestamp_us| {
        check_drift(timestamp_us, metadata.timestamp_us, config.t_drift_us)
    }) {
        Ok(()) => InputStatus::Ready(()),
        Err(error) => InputStatus::Reject(error),
    }
}

fn finish_metadata(
    config: &PublicConfig,
    reader: &dyn FinalizedL1Reader,
    metadata: Metadata,
) -> InputStatus<Metadata> {
    match visible_origin(config, reader, &metadata) {
        InputStatus::Ready(()) => InputStatus::Ready(metadata),
        InputStatus::Wait(missing) => InputStatus::Wait(missing),
        InputStatus::Reject(error) => InputStatus::Reject(error),
    }
}

fn malformed(reason: &'static str) -> DomainError {
    DomainError::Malformed { reason }
}
