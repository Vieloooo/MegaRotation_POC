# Healthy Session API

H1-API-1 defines the typed boundary used by the healthy-session runtime, Session Manager,
and publication bridge. See the [frozen contract](../../docs/healthy-session-program/api-slice-v1.md)
for fields, guard order, and the local control-ID preimage.

- The Manager submits an immutable `ManagerDelta`. `RootCommitted` records the
  accepted revision, replay cursor, and pending obligations. `ActionsApplied`
  records completion against the original commit. `FailClosed` retains known
  guards and any existing commit.
- The runtime supplies `ProgramPort`, `ManagerCommitPort`, and `NativeVerifierPort`.
  It owns program identity, readiness issuance, controlled action execution,
  and the durable root and guards.
- The publication bridge supplies `RegionalResultPort` and `ReplayFeedPort`.
  A `ProofObserved` can precede the target body. A `RegionalDelivery` carries
  the complete typed result for consumer validation. A `ReplayBatch` retains the
  full source record, including its input and effects when it has no lifecycle events.

Shape checks establish structural consistency. Owners validate references, ledger
state, execution, proof validity, and persistence. The crate re-exports the
existing public objects and lifecycle helpers and defines only the local control-ID
hash. The examples compile against injected ports.

From the repository root with Rust 1.95.0 and the dependency cache available:

```sh
flock /tmp/megaroation-poc-heavy.lock cargo check --locked --offline --all-targets --manifest-path poc/crates/healthy-session-api/Cargo.toml
flock /tmp/megaroation-poc-heavy.lock cargo test --locked --offline --all-targets --manifest-path poc/crates/healthy-session-api/Cargo.toml
cargo fmt --check --manifest-path poc/crates/healthy-session-api/Cargo.toml
```
