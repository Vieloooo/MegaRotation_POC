# Viewer GFR

`poc-viewer-gfr` consumes the 03 public types, Plan and execution and the 06 finalized reader interfaces. The [Viewer GFR contract](../../docs/contracts/task-contract-viewer-gfr-v1.md) defines ordering, acceptance, codec bytes and integration gates.

The exported codecs compare and encode replay positions, derive lifecycle identities from full events, and wrap checked 03 objects for 06 messages. `init` binds the public configuration, execution genesis, timing parameters, source identities and expected verification profile. `advance_global` commits one ordered record per complete event. A frontier WAIT has no next position. After the frontier proves which event comes next, a missing verification input returns WAIT at that position and rolls back the entire event, including any same-block handoffs. Force and RESUME stop at their exact ordered control positions.

`chain` and `responsibility` read a completed position without consulting the inputs. `ReplayState::public_state` provides the deterministic typed projection for comparing independent readers at the same position. `publications.results` retains raw proof payloads and locations for audit. It evaluates proof sets and first completeness while the matching descriptor holds NEXT. Late or unrelated evidence can remain unchecked. Replay uses an inclusive `through` position as a comparison bound. A bound behind `pc` returns `HistoricalBound`; use a checkpoint or replay from genesis for an earlier position.

`ReplayState::checkpoint` exports the projection and exact supporting cuts. The timer watermark can extend past the last indexed publication, and L1 headers used by Merge can extend past the control cut. Restoring a checkpoint replays the inputs and compares the complete projection and cuts. Its private initialization binding preserves the typed in-memory parameters, profile and source identities. A durable checkpoint format and its authentication are separate work.

`FixtureL1Reader` wraps the actual 03 reader and returns an empty control list only after finalized header and empty-list delivery establish coverage. The examples bind separate 06 `ReaderState` and 03 readers for A and B and use fixtures that visibly label their non-cryptographic proof verdicts. Actual native final certificates require the 04 integration. G1 supplements 03 headers with explicitly synthetic nonempty control lists to exercise the unsupported-control boundary. Real L1 controls require the 09 adapter.

## Reproduce the bounded fixture traces

Use Rust 1.95.0 with the assigned offline Cargo cache and target directory. Take the shared heavy-job lock once per command.

```sh
export CARGO_BUILD_JOBS=2
flock /tmp/megaroation-poc-heavy.lock cargo test --all-targets --locked --offline --manifest-path poc/crates/viewer-gfr/Cargo.toml
flock /tmp/megaroation-poc-heavy.lock cargo run --locked --offline --manifest-path poc/crates/viewer-gfr/Cargo.toml --example replay -- G1 7 0 > g1-7-0.jsonl
```

Run both `G1` and `G2` with each seed `7`, `42`, `99` and each wall pause `0`, `2` ms. A receives whole blocks in source order. B receives q2 before q0 and q1, reverses and rotates fragment order by seed, repeats selected deliveries and delays one message from q1. In G1 it also stages L1 headers and controls separately. The source data is identical across schedules and wall pauses. Stdout is JSONL containing all raw source objects and typed results per reader and completed bound. The seed and pause are logged on stderr. Compare stdout byte-for-byte within each seed's pause pair.

G1 has six Espresso blocks `q0..q5`. It reaches G height 20 through two immediate handoffs at q1, checks an empty-session fallback, then expires h21. The first synthetic control at L1 height 26 is RESUME, which stops replay ahead of the later Force record and q5. G2 has eight blocks `q0..q7`. It exercises absent public ancestry, delayed local data, multiple proofs, cutoff equality, a cached successor and expiry. Its G reaches h11, with h10 as padding and no additional height accepted at q7. Both use L1 heights `0..30` and DEP heights `1..40`. Native execution includes ordinary transfers and rejected duplicate-nonce transfers, with both raw transactions retained and one transfer in Kept per physical fixture block.

Replay retains up to the 06 bounds of 4,096 Espresso blocks and 64 MiB source log, 4,096 L1 fixture rows and 4,096 post-genesis G entries, and 8,193 records. A required input beyond a finite bound returns an explicit error. The codec limits proof bytes to 32,768, descriptor frames to 16,384 and each message to 65,536 bytes. The lifecycle codec limits a recovered prefix to 4,096 blocks and caps projection output at 64 MiB. These runs exercise the specified finite public histories.
