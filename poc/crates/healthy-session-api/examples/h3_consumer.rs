//! Compile-only use of the native binding and replay feed by the publication bridge.

use poc_healthy_session_api::{
    ApiError, ControlledOrigin, EvidenceRef, IdKind, LocalId, NativeVerifierPort, PortResult,
    PublicConfig, RegionalDelivery, RejectReason, ReplayBatch, ReplayCursor, ReplayFeedPort,
    ReplayRecord, VerifierBinding, derive_local_id, lifecycle_id, validate_native_binding,
};

/// Obtain the real verifier and basis from H1. The caller still validates the
/// publication and proof using the historical inputs.
pub fn native_for_delivery<'a>(
    port: &'a dyn NativeVerifierPort,
    config: &PublicConfig,
    delivery: &RegionalDelivery,
) -> PortResult<VerifierBinding<'a>> {
    fn check(
        config: &PublicConfig,
        delivery: &RegionalDelivery,
        binding: &VerifierBinding<'_>,
    ) -> Result<(), ApiError> {
        delivery.validate_shape()?;
        validate_native_binding(binding, config)?;
        if delivery.result.descriptor.plan.config_ref != config.reference()? {
            return Err(ApiError::BindingMismatch("config_ref"));
        }
        if delivery.native_profile != binding.profile {
            return Err(ApiError::BindingMismatch("native_profile"));
        }
        Ok(())
    }

    match port.binding() {
        PortResult::Ready(binding) => match check(config, delivery, &binding) {
            Ok(()) => PortResult::Ready(binding),
            Err(error) => PortResult::Reject(RejectReason::Shape(error)),
        },
        PortResult::Wait(missing) => PortResult::Wait(missing),
        PortResult::Reject(reason) => PortResult::Reject(reason),
    }
}

/// Wrap a record and provenance supplied by the committed replay source.
/// An empty events list still carries the entire record, including its input and effects.
pub fn replay_batch(
    record: ReplayRecord,
    source_record: EvidenceRef,
    sequence: u64,
    previous: Option<LocalId>,
    origin: ControlledOrigin,
) -> Result<ReplayBatch, ApiError> {
    let source = source_record.source;
    let id = derive_local_id(&origin.grant.scope, IdKind::ReplayBatch, sequence, source)?;
    let lifecycle_ids = record
        .events
        .iter()
        .map(lifecycle_id)
        .collect::<Result<_, _>>()?;
    let batch = ReplayBatch {
        id,
        source,
        sequence,
        previous,
        source_record,
        record,
        lifecycle_ids,
        origin,
    };
    batch.validate_shape()?;
    Ok(batch)
}

pub fn read_batch(
    feed: &dyn ReplayFeedPort,
    after: Option<&ReplayCursor>,
) -> PortResult<ReplayBatch> {
    match feed.next(after) {
        PortResult::Ready(batch) => match batch.validate_shape() {
            Ok(()) => PortResult::Ready(batch),
            Err(error) => PortResult::Reject(RejectReason::Shape(error)),
        },
        PortResult::Wait(missing) => PortResult::Wait(missing),
        PortResult::Reject(reason) => PortResult::Reject(reason),
    }
}

fn main() {
    println!("Compile-only H3 consumer. Verifier and replay feed are supplied by their owners.");
}
