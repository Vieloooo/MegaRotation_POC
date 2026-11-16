# Publisher component

PE-PUB-1 is represented by `Publisher`, `PublicationTask`, `PublicationVerifier` and `StateCodec`. The verifier supplies the key, validation verdict, verified cutoff and publication messages. Use the 03 `poc_types::domain::ResultKey` from M1 `0e72dc247de22afdc8d6681713b3d15f4aa05262` and its canonical codec when wiring a real result adapter. This crate currently depends only on the SimEspresso interface. It leaves native CheckFINAL and anchor verification to the 03/04 adapter under the frozen contract.

Use `Publisher::create` with a new directory or `Publisher::open` for recovery. Both require the same stable namespace, configuration, source and designated reader. The store acquires an exclusive writer lock and commits an entire bounded snapshot by syncing its temporary file, renaming it to the main file and syncing the containing directory. The state envelope uses `PESP` v1 and a CRC32 for storage corruption. The caller's `StateCodec` owns the embedded canonical key and result bytes. Directory sync failure after rename poisons the current writer, including replay of an already prepared submission. Reopen inspects the committed main file. An old temp file has no recovery authority.

`offer` durably retains a validated result or pending materials under one key. A later successful complete result keeps the first verified payload for that key. `prepare_attempt` commits a prepared intent before returning a `Submission`. It replays a prepared submission under the original identity, bytes and tick until `mark_dispatched` records the controlled commit reference. An explicit later attempt keeps the original task and exact payload. `mark_service_acknowledged` validates the attempt ID, payload and controlled reference. The 02 integration must commit the returned submission as a controlled effect before network output.

`confirm_recorded` consumes a `VerifiedPrefix` from the designated reader and retains the first complete receipt, including its publication position and the actual prefix used to verify completeness. `apply_watermark` checks this reader's complete prefix against the task's verified cutoff. `apply_public_stop` applies a precise 07 lifecycle projection to existing tasks and preserves history. If a task arrives after a public exclusion, the 07 adapter must project that history onto the late Offer before permitting output. Program TIME and Manager changes create no public stop input. A verified late receipt can record a stopped task while retaining its stop reason.

The component tests use the verifier and codec traits with explicitly non-cryptographic inputs. The replay example uses fixed delivery schedules for seeds 7, 42 and 99. Its virtual JSONL stdout can be compared byte for byte for each seed across wall pauses of 0 and 2 ms. The wall pause is applied by the example driver and appears only on stderr. Every run writes a fresh file-backed publisher store, reader checkpoints and raw PELG records to the supplied new directory.

```sh
flock /tmp/megaroation-poc-heavy.lock cargo test --locked --offline --manifest-path poc/crates/sim-espresso/Cargo.toml
flock /tmp/megaroation-poc-heavy.lock cargo test --locked --offline --manifest-path poc/crates/publisher/Cargo.toml
flock /tmp/megaroation-poc-heavy.lock cargo run --locked --offline --manifest-path poc/crates/publisher/Cargo.toml --example scenario_trace -- 42 2 /absolute/new-artifact-dir
```

With Rust 1.95.0, use the task's data disk for Cargo home, build outputs and traces, and set `CARGO_BUILD_JOBS=2`. Set `PE_TEST_EVIDENCE_DIR` to an existing task artifact directory if the test runner should retain fault injection and file reopen evidence.

| Contract cases | Test target |
| --- | --- |
| PE01–PE10 | SimEspresso `tests/contract.rs` |
| PE11–PE14 | Publisher `tests/contract.rs` |
| PE15 | Publisher `src/recovery_tests.rs` |
| PE16 | Publisher `examples/scenario_trace.rs` |
