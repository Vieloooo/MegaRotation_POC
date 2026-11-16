# P0 batch services

This independent workspace implements the P0 batch format fixed in the
[component contract](../../docs/contracts/task-contract-batch-services-v1.md).
Feature 12 can add `batch-services = { path = "../batch-services" }` to its own
manifest and use the following interfaces.

- `frame::encode_closed(&[BlockRecord])`, `frame::decode_closed(&[u8])`,
  `frame::commitment(&[u8])` and `frame::verify_payload(Hash32, &[u8])` share the
  single BS-FRAME-v1 format. Records preserve 03 raw ordinary transactions.
  `verify_payload` checks the commitment and frame shape before returning any
  records.
- `crypto::InboxDomain` binds chain id, Inbox address and configRef.
  `check_da` verifies a DA receipt and `verify_batch` verifies the certifier's
  signature over the complete feature 09 attestation. Call `validate_submission`
  for the P0 cursor, gpos and range guards. `submit_batch_calldata` emits the
  feature 09 ABI from its `BatchWire`. Successful signature checks establish
  authentication only.
- `accepted_view::InboxGateway::connect(endpoint, config, signers)` opens the
  feature 09 driver on a private Anvil instance at genesis. The host supplies
  its single ordered command stream through `apply(Command)` and assigns
  `read_accepted_block(command_id, reader, height)` an ID in that same stream.
  The reader needs the finalized block delivered to its named prefix before
  the query can return. The method returns the exact driver reply and original
  successful `CursorAdvanced` inputs, in transaction/log order. A visible block
  with zero advances returns an empty list. Repeated queries return the same
  `AcceptanceIdentity` per original log.
- `AcceptedRecord` has a closed constructor and read-only accessors for
  its position, configuration, full `BatchWire` and evidence. The evidence
  retains the original transaction and receipt, inclusion timestamp, exact
  transaction-local `position.log`, RPC log index and driver frontiers.
  Serialized evidence is for audit. `AcceptedRecord::load_payload` takes
  `Option<&[u8]>`, rechecks signatures, checks commitment and frame format,
  and matches the record range and final origin to the signed cursor. Missing
  or corrupt DA bytes return `ReadError::Wait` at that input position.

The component relies on the configured local Anvil RPC and the feature 09
finalization/delivery model. The gpos bytes are opaque here. Feature 12 derives
from accepted L1 evidence and DA independently of G.

Run the locked interface checks from the checkout root with Rust 1.95.0 and
the feature-local cached registry and target:

```sh
export PATH="/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:$PATH"
export CARGO_HOME="$PWD/.build/cargo"
export CARGO_TARGET_DIR="$PWD/.build/target/batch-services"
export CARGO_BUILD_JOBS=2
flock /tmp/megaroation-poc-heavy.lock cargo test --locked --offline \
  --manifest-path poc/crates/batch-services/Cargo.toml
```

The tests consume the independent byte and signature vectors. The real Anvil
receipt and durable service-process matrix is a separate component milestone.

## Durable actor component

`Batcher`, `Certifier` and `DaStore` take separate durable directories and a
`ServiceConfig` for their deployment. A role checks its configured key and
persists command identity before the effects of an accepted command. The host
supplies the next command and handles all transport. `Batcher::prepare_transaction`
persists the signed bytes, hash and nonce before giving the raw outbox to the
host. `Batcher::reconcile` adopts the original `CursorAdvanced` or a definite
failed receipt from a complete delivered prefix. An unknown result waits with
the existing outbox identity. `DaStore::retrieve` supplies the bytes at that
commitment for `AcceptedRecord::load_payload`.

`InboxGateway::read_current_cursor` reads a named reader's current delivered
prefix and finds the last CursorSet at or before the cursor's `written_at`.
The result preserves its paired RESUME position and sequence. Batcher and
Certifier replay independent `FixtureSourceView`s using `poc-core` Apply.
One fixed gpos can supply successive batch ranges after verified execution
through each preceding height. `materialize_accepted_view` exposes the verified
cursor and fixture-materialized prefix. A CursorSet without its ResumeCut
exposes the older materialization as historical and returns the missing cut's
exact position. Real GFR and ResumeCut providers remain integration gates.

Run the locked Anvil, process-exit and wall-pause comparison from the checkout
root after selecting the Rust toolchain and cache as above:

```sh
export L1_FOUNDRY=/mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0
flock /tmp/megaroation-poc-heavy.lock bash poc/crates/batch-services/tests/validate.sh \
  "$PWD/.build/artifacts/batch-services-$(git rev-parse --short HEAD)-run1"
```

Use a fresh output directory for each run. `pause-{300,650}/run/` contains
`raw-rpc-bundle.json` (09's config, complete blocks and receipts), `driver-trace.json`
(the ordered Tick, Submit, Mine, Finalize, Deliver and Read commands and replies),
`rpc-transcript.json`, `accepted-records.json`, the `da-index.json` commitment to
relative immutable payload-file map and its raw DA files, and
`execution-source.json` (raw fixture input and observed execution results).
`crash-trace.json` records actual child PIDs, termination mechanism, persistence
boundary and before/after journal checksums. The single-threaded Linux test
host injects process exit at the documented persistence hooks and SIGKILL
between the real broadcast and the Batcher's reply. Its 09 driver and private
Anvil continue running through those exits. Protocol traces and actual archived
receipts are compared across the two imposed wall pauses. Process metadata is
recorded separately because PIDs are assigned by the operating system.

The scenario covers the contract cases below. Final component status follows
independent review and the Tester run at the candidate revision.

| Cases | Evidence |
| --- | --- |
| B01–B03 | Frozen Rust frame and EIP-712 vectors, production execution audit from the fixture and mined wrong-signer/DA-commitment failures. |
| B04–B05 | Same-content Disperse and Retrieve, independently controlled unavailability, and process exit before bytes, after durable bytes and after durable receipt. A separate test damage/restore of the backing file verifies mismatch WAIT and same-byte recovery. |
| B06–B08 | Certifier uses its own fixture view and reader, tests missing anchor/state/metadata/DEP/height, conflicting pre-state, stale cached prev, exact paired RESUME and two signature process boundaries. The later GLOBAL cursor retaining the earlier RESUME is also covered by a history-selection fixture. |
| B09–B10 | Durable role identities, same-command replay/conflict, five Batcher phase exits and one actual SIGKILL after submission with lost reply. Reopening waits at the unknown transaction then reconciles original finalized acceptance. |
| B11–B13 | Two real Inbox advances from state-changing batches, included/finalized/delivered Read checks, a successful duplicate request with no new advance, and a mined stale-prev failure after Override. |
| B14–B16 | Accepted payload WAIT while DA is missing or corrupted, missing gpos and position-bound missing ResumeCut, and matching protocol, receipt and crash-boundary evidence across 300/650 ms pauses. |
| B17 | Reusable production API and raw receipts and DA artifacts. Feature 12's derivation test is a separate gate. |
| B18 | Real GFR, 02 control transport, historical ResumeCut and full-system integration remain open. |
