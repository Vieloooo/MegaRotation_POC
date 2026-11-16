# Simulation transport kernel v1

This independent Rust workspace implements the frozen [kernel contract](../../docs/contracts/task-contract-sim-transport-kernel-v1.md) and [consumer API](../../docs/sim-transport-api-v1.md).

The controller prepares a `RoutePlan` against a single `RouteKernel` revision and commits it to obtain the ordered transport events. It supplies virtual ticks and grant identities. `GrantConsumer` accepts those grants and returns service intents for the root adapter to commit.

The mailbox controller holds exclusive custody from `plan` through persistence, confirmation, and root output commit. The adapter authenticates current bindings and confirms actual persistence using a receipt bound to the candidate record. Active sender entries require a confirmed attempt before acknowledging custody. The authenticated receiver can acknowledge after rebinding to a newer incarnation. A duplicate confirmation against a tombstone releases no effect. After recovery, the adapter uses the persisted record, receipt, and root history to deduplicate release identities by transition ID and release index. If a persisted plan becomes stale, the adapter fails closed and reconciles that record through recovery.

From the repository root on xuelab, select the pinned toolchain and task 02 data volume.

```sh
export PATH="/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:$PATH"
export CARGO_HOME=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/cargo
export CARGO_TARGET_DIR=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/target/sim-transport
export CARGO_BUILD_JOBS=2

flock /tmp/megaroation-poc-heavy.lock cargo test \
  --manifest-path poc/crates/sim-transport/Cargo.toml --locked --offline --all-targets
```

Run the component replay with a fresh output directory whose parent exists.

```sh
flock /tmp/megaroation-poc-heavy.lock cargo run \
  --manifest-path poc/crates/sim-transport/Cargo.toml --locked --offline --example replay -- \
  --seed 7 --declaration-order forward --wall-pause-ms 0 \
  --output /mnt/reth/jy/megaroation_poc/.build/commander-jobs/02/artifacts/replay-7-forward-0
```

`logical-trace.json` records virtual steps, transport events, ClockView observations, service results and custody transitions. `diagnostics.json` records declaration order, wall settings and runtime counters. Compare logical trace bytes within each seed across `forward` and `reverse` declaration order and wall pauses of `0` and `2` ms.

The replay uses in-process components, mock receipts and zero native workers. Real root persistence, transport authentication, Clock RPC and native message integration remain gates for feature 02. Public log interpretation and canonical business identities belong to the service and protocol modules.
