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
