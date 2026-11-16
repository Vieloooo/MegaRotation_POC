# Espresso compatibility

Independent workspace for the feature 16 pinned local-node profile. See the
[frozen contract](../../docs/contracts/task-contract-espresso-compatibility-v1.md)
and [independent expectations](../../verification/espresso-compatibility/expectations-v1.md).

`SourceIdentity::pinned(run_id, genesis_block_hash) -> SourceIdentity` supplies the
release, image, protocol, API and namespace fields. `SourceIdentity::source_id(&self)
-> Result<String, Error>` validates those fields and derives the source identifier.
`SourceBinding::new(identity, capture_manifest_sha256, capture_kind)
-> Result<SourceBinding, Error>` associates a recording and its official or fixture
classification with that identity.

`CaptureRecord<'a>` exposes the source, reader, request method and path, record ID,
optional HTTP status and raw response bytes. `NativeBlock` contains a complete list
of `NativeTransaction` values with the original positions in `NativePosition`.
`ProjectedMessage` combines a 06 `LogMessage` and compact `EspressoPosition` with
its original `NativePosition`. Timing evidence stays `Unknown` and
`FinalityBasis` declares the local API or fixture basis alongside record references.

`export_pe_log() -> Result<Vec<u8>, UnsupportedTimingEvidence>` and
`export_pe_reader() -> Result<sim_espresso::ReaderState, UnsupportedTimingEvidence>`
both report the missing proposal, finalization and live-at-proposal evidence.
