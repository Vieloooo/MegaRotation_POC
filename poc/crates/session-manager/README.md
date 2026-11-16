# Session Manager reducer

`poc-session-manager` implements [H2-REDUCER-1](../../docs/contracts/h2-session-manager-reducer-v1.md) as one in-memory Manager per worker. It accepts the existing 03 domain values and complete 07 replay records. Its outputs are state projections and typed action intents.

## Consuming a decision

1. Initialize `Manager::new` with the trusted binding, public config and genesis descriptor. For a local committee member the initial view includes one revision-zero Create intent. The adapter owns the bootstrap commit.
2. Call `plan(input, local_time_us)` using the worker's NodeClock. For a `Planned(delta)`, use `delta.view()` and `delta.guards()` to install the proposed state and guards in the controlled commit. `delta.intents()` lists the new obligations in stable ID order.
3. After the state and guards are durable, create the attestation with `CommitRef::for_delta(&delta, durable_ref)` and call `accept_commit`. If commit fails after any guard has been installed, call `latch_commit_failure(&delta, guard_ref)` and stop the run. Keep the runtime's installed guards in force.
4. Poll `dispatchable(local_time_us)` during the controlled step. Use the committed guards and release only eligible intents through the grant/Ack path. Each intent's dependencies are fixed. The current schedule and outstanding parking/closing obligations also gate reused Enter intents. A matching `Entered` receipt records the first entry time and a matching `PreparedRef` marks preparation complete.
5. Submit matching receipts through `plan` and the same commit path. Replay or retry the original action ID. A repeated commit reports the current status of that decision's obligations. `view().status()` reports the state of all actions.

`delta.view()` and `ManagerView` borrow read-only state. Public assignment and local scheduling have separate getters. The complete accepted record retains the source sequence, public position, original ordered effects and events, validation reference, and the 07 lifecycle IDs.

The [component tests](tests/reducer.rs) contain executable consumption examples using the M0 object and key fixtures. Run them with Rust 1.95.0, the existing Cargo cache, and the assigned data-disk target directory.

```sh
flock /tmp/megaroation-poc-heavy.lock cargo test --locked --offline \
  --manifest-path poc/crates/session-manager/Cargo.toml --test reducer
```

The tests are trusted-input fixtures. The runtime adapter remains responsible for validating real execution/proofs, installing root and auxiliary state and sticky guards, preserving native engine instances during Pause/SelectBack, and releasing outputs through controlled grant/Ack. The GFR adapter owns verified replay provenance and the publisher owns public recording and acceptance. These are M1 integration gates.
