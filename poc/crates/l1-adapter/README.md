# Anvil L1 adapter

`l1-adapter` accepts explicit commands from one ordered owner. It runs against a private loopback Anvil process started with `--no-mining`. Versioned scope and the ABI are in [the feature09 contract](../../docs/contracts/task-contract-anvil-l1-v1.md) and [the exported Solidity ABIs](../../contracts/abi/). This workspace builds independently with Rust 1.95 and the included lockfile.

The component begins with a complete genesis block. `Tick` advances the relative virtual clock in microseconds. `Submit` broadcasts a signed raw transaction. `Mine` sets a slot timestamp and mines a single block. It verifies the header, all transactions and their receipts before advancing inclusion. `Finalize` advances independently after the simulated slot delay. `Deliver` advances a named reader only across the blocks it has received. Reader queries cover a full block, a specific transaction-local event, or Inbox cursor state at a named block. Cursor results cross-check historical `eth_call` at the exact block hash against the event history. A result includes the transaction and receipt that last changed the cursor.

The adapter retains both the transaction-local log index and the RPC block-global log index. It emits both members of a RESUME pair only after verifying their shared successful receipt. It exposes L1 wire evidence for the shared type adapter and the future derivation reader. It relies on the private Anvil node for canonical execution and block receipt availability.

An exact replay of a resolved command id returns the original output. To reconcile a pending mutation, resend the original command. The adapter checks the transaction hash or canonical block head and receipts without performing the mutation again. A new submit or mine waits while reconciliation is pending. This standalone process starts at genesis and retains command state in memory. SimHub grant/ack and the feature03 shared object mapping are integration gates. Fixture results cover only the explicit command component.

The `l1-adapter LOOPBACK_ADDR CONFIG_JSON [DROP_MINE_ANSWER_NUMBER]` runner reads one JSON command per stdin line and writes one response line after a startup line. The third argument is a test switch that drops the response after the selected real mine has executed. The Rust state machine sees the loss while the transcript retains the actual response. See the scenario for a complete deployment and command sequence.

Run on xuelab from the checkout root with the task's private Cargo cache prepared.

```sh
export PATH="/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:$PATH"
export L1_FOUNDRY=/mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0
export CARGO_HOME="$PWD/.build/cargo"
export CARGO_TARGET_DIR="$PWD/.build/target/l1-adapter"
export CARGO_BUILD_JOBS=2
flock /tmp/megaroation-poc-heavy.lock bash poc/crates/l1-adapter/tests/validate.sh \
    "$PWD/.build/artifacts/anvil-l1-$(git rev-parse HEAD)-run1"
```

The validation script builds both contracts and the adapter with the pinned tools. It runs the real Anvil scenario twice with 300 ms and 650 ms wall pauses, compares the full virtual traces, and replays the real receipt bundle through the same Rust coverage code with selective RPC faults. It requires Python with `coincurve` for test fixture signing. The fixture's derived keys are local test identities. The output contains a command trace, separate read-only diagnostic calls, public signatures, complete raw RPC bundles, the build revision, tool hashes and the virtual trace comparison.
