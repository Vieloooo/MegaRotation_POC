# Publisher and SimEspresso component matrix

**PASS, PE01–PE16.** On `8ba78f1ace3b383caa1c4360401c645d25907326`, all 15 component tests and six fixed schedule replays passed. Every seed produced identical virtual JSONL, raw public log bytes, and Publisher and reader snapshots with 0 ms and 2 ms local wall pauses. Each replay also matched independent expected events and bytes.

The named `bft_simulator` role ran this matrix on 2026-11-16 (Asia/Shanghai) from `/home/jy/.codex/worktrees/1fe1/megaroation_poc` on `codex/poc-publisher-espresso`. The role used GPT-6.1 Sol with high effort configured. The actual execution HEAD was the verified candidate above. The two component directories were checked before and after execution and matched the candidate. The only worktree additions during execution were untracked verification files, whose input hashes are recorded in `environment.json`. The Regional BFT gitlink was `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`. The frozen contract and independent candidate review are linked here. [Contract](../../docs/contracts/task-contract-publisher-espresso-v1.md) · [Independent review](candidate-review.md)

| Cases | Result | Observations |
| --- | --- | --- |
| PE01–PE10 | PASS, 7 tests | Monotonic q with equal and frozen theta, C3 timestamp classification, atomic rejection, finalize ordering, exact PELG bytes and bounds, independent readers and fragmented-prefix recovery. |
| PE11–PE14 | PASS, 4 tests | Offer validation and same-key deduplication, durable submit/ACK state and reader-bound receipt, scoped stops, cutoff equality and late historical receipts. |
| PE15 | PASS, 4 tests | Real file reopen across task transitions, pre-rename failure, post-rename sync uncertainty, stale temp, corruption, writer exclusion and store identity checks. |
| PE16 | PASS, 6 runs and 3 pairs | Each of three fixed delivery schedules passed its literal oracle at both wall pause settings. All ten compared files matched byte for byte in each pair. |

The replay oracle independently constructs the full expected event sequence from the fixture inputs and independently encodes four PELG records, two Publisher snapshots and three reader checkpoints. Reader A receives `q0..q3` at virtual ticks 130–133. Reader B fills its gaps using the schedule below, then receives q3 at tick 145. All runs replay prepared attempt 1 at tick 103 under its original identity and tick 102, dispatch it, then create attempt 2 under the same task with the same payload. The task remains `SUBMITTED` through service ACK and a reopen after finalization. Confirmation records publication q0/index0 with completeness through q2 at virtual tick 144. After the cutoff at q3/theta 105 and another reopen, generation 9 retains both the receipt and the deadline stop marker.

| Seed | B delivery order before q3 | B contiguous tips before q3 | Canonical JSONL SHA-256 for both pauses |
| --- | --- | --- | --- |
| 7 | 2, 0, 1 | empty, 0, 2 | `a604ebd7e58d22f8c1b1dacac0bbb50c2b19b7c79218df2a41a8c5a8c3d99861` |
| 42 | 2, 1, 0 | empty, empty, 2 | `650e26dc56bb592d4abf52c4c7f476877b9ca0dfadd02386ebc4cce0a2d625a5` |
| 99 | 0, 2, 1 | 0, 0, 2 | `a0b48f40ea0f7eac5c1c7a3fd9d4cb567ca798d7d028aa470f504ec98452174f` |

The compared files include standard output, four PELG records, the pending and final reader checkpoints, and the prepared and final Publisher snapshots. The writer lock file coordinates the local filesystem and carries no protocol state. Standard error and local elapsed time are diagnostic. Each replay covers virtual ticks 100–146, with theta keys `[104, 104, 104, 105]` and finalization ticks `[111, 113, 115, 117]`. Local observation of each Cargo invocation, including lock wait and startup, ranged from 0.067 to 0.095 seconds. The matrix used three explicitly selected schedules and no real network measurements.

From the recorded worktree, the executed command was:

```sh
python3 poc/verification/publisher-espresso/run-matrix.py --exact-head \
  /mnt/reth/jy/megaroation_poc/.build/commander-jobs/06/artifacts/simulation/8ba78f1ace3b383caa1c4360401c645d25907326/matrix-run1
```

For a source-identical delivery commit, omit `--exact-head` and use a fresh directory under the same candidate directory. The runner records the actual HEAD separately and rejects any differences in the two component directories. The following commands were run serially, with Cargo `--locked --offline`, `CARGO_BUILD_JOBS=2`, the assigned `cargo` and `target/simulation` directories, and retained files set through `PE_TEST_EVIDENCE_DIR`. Each command acquired `/tmp/megaroation-poc-heavy.lock` once. Lock acquisition and execution each had a 120-second budget. All eight commands exited 0. The fixed unit and integration tests use no seed.

```sh
flock -w 120 -E 75 /tmp/megaroation-poc-heavy.lock timeout 120 cargo test --locked --offline --manifest-path poc/crates/sim-espresso/Cargo.toml -- --test-threads=1
flock -w 120 -E 75 /tmp/megaroation-poc-heavy.lock timeout 120 cargo test --locked --offline --manifest-path poc/crates/publisher/Cargo.toml -- --test-threads=1
flock -w 120 -E 75 /tmp/megaroation-poc-heavy.lock timeout 120 cargo run --locked --offline --manifest-path poc/crates/publisher/Cargo.toml --example scenario_trace -- "$seed" "$pause_ms" "$new_fixture_directory"
```

Toolchain versions were `rustc 1.95.0 (59807616e 2026-04-14)`, `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`, and Python 3.12.8. The raw evidence directory contains command and environment records, logs, traces, original snapshot files, input hashes and a verified 121-file `SHA256SUMS`. Its SHA-256 is `ef61f46f11461df871771526d8e71b6ffd123591a8f8c54bbbab0fa6f19f4b7e`. [Machine-readable results](simulation-results.json) · [Raw evidence](/mnt/reth/jy/megaroation_poc/.build/commander-jobs/06/artifacts/simulation/8ba78f1ace3b383caa1c4360401c645d25907326/matrix-run1)

Full 06 remains **INCONCLUSIVE / OPEN**. D02 requires the controlled grant/Ack transport and durable custody. D03 requires canonical object, codec and validation adapter wiring. D04 requires real final certificates. D07 requires lifecycle-scope projection and independent GFR replay. D16 requires official Espresso source validation. D06-integrated requires the actual controlled readers, Publisher, replay and crash/restart matrix. This bounded matrix used explicit non-cryptographic component inputs and real file reopen with injected commit failures.
