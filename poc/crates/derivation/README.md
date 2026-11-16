# Independent D derivation

This standalone Rust workspace implements Stage A of the [frozen task-12 contract](../../docs/contracts/task-contract-derivation-v1.md). It binds the configured 03 genesis and fixed deposits to the finalized 09 reader, scans receipt events in transaction-local log order, and executes deposit-only fallback and CursorSet fills on the actual D parent. It records each ResumeCut immediately after the exact paired CursorSet. The cut includes the complete historical prefix at that marker and the execution anchor at the target height.

`DriverSource` owns an instance of the checked 09 driver. Feed it ordered `Command`s and use the `VerifiedInputs` read view for `Derivation::init`, `step`, `advance` and `restore`. The source collects successful full block reads from its configured named reader. Bind `expected_l1_genesis` to the canonical L1 genesis hash in the caller's source configuration. `FixtureSource` is a separate capability for the bounded 03 receipt-free fixture. It uses explicit finalization and delivery for the header and committed empty receipt history.

`Derivation::derive_at_resume` uses 03's `(block, tx, log)` position, where `log` is transaction-local. The RPC block-global log index is kept as evidence. `Checkpoint` is a serializable local state claim. `restore` replays the available verified inputs to the claimed action, validates the checkpoint and reconstructs all historical cuts. It can restore from a longer available L1 prefix without including later activity in the checkpoint or cut. Use `native_hash_at` when comparing a specific height with an independent view of another chain.

Real receipt gates use private manual-mining Anvil with the 09 contracts and driver, then replay the raw RPC and ordered commands through a fresh driver before deriving D. Tests compare two named readers with different deliveries. A separate process reads the suspended checkpoint between the two markers at block 45. The missing receipt case injects data unavailability during an offline replay of a fresh Anvil transcript and reconciles the same pending mine when the genuine receipt becomes available. Semantic fault fixtures and the local archived-input runner exercise no BFT process restart or controlled runtime integration.

From the repository root, using the task's assigned private cache and data directory:

```sh
export PATH="/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:/home/jy/miniconda3/bin:$PATH"
export CARGO_HOME="$PWD/.build/cargo"
export CARGO_TARGET_DIR="$PWD/.build/target/derivation"
export CARGO_BUILD_JOBS=2
export L1_FOUNDRY=/mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0
flock /tmp/megaroation-poc-heavy.lock bash poc/crates/derivation/tests/validate.sh \
    "$PWD/.build/artifacts/derivation-new-run"
```

The matrix uses explicit scripted actions and no seed. It writes inputs, driver transcripts, derivation snapshots, cuts, and a SHA-256 manifest in the new output directory. Pauses of 300 and 650 milliseconds occur at the same point in the Anvil harness. The driver uses explicit reference ticks. The comparisons cover the virtual traces and canonical derivation results independently of wall-clock delays.

Stage B still requires feature 11's verified accepted-frame interface and real payloads. In Stage A every accepted batch holds the exact position with `Feature11AcceptedPayloadRequired`. The opaque receipt fixture demonstrates this boundary without claiming a verified feature-11 payload.
