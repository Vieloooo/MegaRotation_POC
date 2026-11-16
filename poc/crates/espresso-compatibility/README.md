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

`OfficialCaptureReader::new(binding, reader_id) -> Result<Self, Error>` starts
with an empty prefix. Its `receive(CaptureRecord) -> Result<ReceiveOutcome, Error>`
takes exact pinned API routes, records only the first observation of each
semantic object, and reports submit acknowledgements as `Submitted`. Conflicting
responses return an error with the previous prefix intact. A block contributes
to the prefix after its matching leaf and all flat transaction positions arrive.
`tip()`, `read(q)`, `prefix()`, `require_prefix_through(q)`, and
`require_watermark(theta)` read that contiguous prefix. Missing input reports
`Wait`, including the first missing height and any known missing positions.

`project_publication_messages(q, codec) -> Result<Vec<ProjectedMessage>, Error>`
selects namespace 160016 and assigns compact message indices. The complete
native list retains the original positions and duplicate transactions.
`find_publication(hash, payload) -> Vec<NativePosition>` requires the namespace,
ACK hash and exact payload bytes to match a transaction in the visible prefix.
`public_audit_bytes(codec)` and `public_sha256(codec)` implement EC16-AUDIT-1.
`compare_public_objects(&other)` compares the full parsed block, leaf and
transaction objects in two independently read complete prefixes.

`CaptureBundle::open(directory) -> Result<Self, Error>` validates a versioned
`manifest.json`, pin and source ID, relative paths, and the size and SHA-256 of
every referenced file. `replay(reader_id) -> Result<ReplaySession, Error>`
feeds only that reader's recorded responses into a fresh in-memory reader.
`replay_bundle(directory, reader_id)` does both. A manifest-referenced file
that cannot be read is `InvalidRecording`. A missing logical position in an
intact bundle remains `Wait`.

For successful lookup responses, the reader matches the full block and leaf
headers and validates the block's base64 payload size and namespace table.
It checks each transaction against the source count, requested height and index,
block hash and namespace. It trusts the pinned local API for finalized data and
flat ordering. It retains QC and commitments as source records.

Lookup transport failure and HTTP 404, 408, 429 or 5xx leave the requested input
unavailable. The live harness retries within its wall budget and preserves the
exact failure and response in the transcript. A submit HTTP 4xx or 5xx is
`SubmitRejected`. Redirects are errors. An ACK advances no public position.

The executable provides two offline commands. They run from a prebuilt binary.

```sh
espresso-compatibility replay BUNDLE READER NEW_OUTPUT [--pause-ms N]
espresso-compatibility compare BUNDLE READER REQUERY_READER
```

Replay creates `public-audit.bin`, `public-projection.json` and `receipts.json`.
The first two files exclude receipt evidence and wall metadata. Receipts list
the reader identity, bundle SHA-256, unavailable records and submit outcomes.
See the [local scenario](../../scenarios/espresso-compatibility/README.md) for
official launch and capture.

The `EC16-CAPTURE-1` manifest has these fields:

| Field | Meaning |
| --- | --- |
| `capture_kind` | `fixture` or `official` |
| `identity`, `source_id` | Validated EC16-SOURCE-1 profile and run/genesis binding |
| `configuration` | Launch commands, local endpoints and ports, verified official OCI artifacts, and process metadata for official mode |
| `artifacts` | Reserved supporting files, each with a relative `path` and lowercase `sha256` |
| `records` | Strictly increasing `id`, `reader_id`, `source_id`, HTTP `method` and `path`, optional `status`, raw `request` and `response` references, and `observation` metadata |

A raw file reference uses the `artifacts` format. A missing HTTP request body
or absent response from a transport failure uses JSON null. Official records
also carry an `observation.origin` matching their configured private API
endpoint. The manifest checksums identify the recorded bytes within a capture.
Trust in the original observation comes from the local launch and capture
procedure.
