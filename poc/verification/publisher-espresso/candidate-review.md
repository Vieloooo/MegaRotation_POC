# Candidate review — 8ba78f1

Date: 2026-11-16 (Asia/Shanghai). Verifier: named `bft_verifier`, configured GPT-6 Astra/xhigh; no separate runtime-model attestation was exposed. Candidate: **`8ba78f1ace3b383caa1c4360401c645d25907326`**. Frozen contract/independent-expectation baseline: `4334f4f`; source baseline `0a951c482a9712678ab9de34765b781b9c3c9f42`. Worktree `/home/jy/.codex/worktrees/1fe1/megaroation_poc`, branch `codex/poc-publisher-espresso`.

**PASS — bounded independent component review. No actionable blocking finding. Ready for the agreed Tester matrix on the same candidate SHA.** This does not mark the complete 06 feature accepted. PE16 was deliberately not executed during this review.

## Method and source coverage

The verifier derived [independent expectations](expectations-v1.md) from the pinned sources before reading implementation, then reviewed the exact candidate's complete `publisher/src/{lib,codec,store}.rs`, its private recovery-fault tests, and the SimEspresso implementation. The earlier interface slice `9d3dc6c` was reviewed read-only; the final delta only adds the validated message-list wrapper. The candidate sources were unchanged before/after execution; the only working-tree additions were this verifier's files.

| Source requirement / expectation | Candidate check and observed evidence |
| --- | --- |
| E-S/E-TS; T01–T06/L03 | `sim-espresso/src/lib.rs` timestamp/propose/finalize and `wire.rs` bounds reviewed. External harness checks two prewritten exact wire literals, ten malformed cases, fixed/equal/regressing keys, full-deviation E-TS classification, non-live proposal semantics, checked arithmetic, and duplicate-finalize tick invariance. PASS. |
| Complete independent prefixes; L02–L05/M01–M02 | `reader.rs` fragment validation, atomic clone/install, neighbor checks, promotion and snapshot decoder reviewed. External harness obtains B tips None/0/0/2, restores actual serialized pending bytes, rejects causal/conflicting fragments, preserves first receipt, and compares A/B public blocks at equal q. PASS. |
| PUB-1; P01/P11 | `publisher/src/lib.rs:551` refuses receipt verification before a full Offer payload exists; `codec.rs:332` validates this invariant on persistence/recovery. External fixture counts record-verifier invocations: zero for VERIFYING and unverified STOPPED; only after full Offer verification may a stopped task record. Actual file reopen preserves stop and receipt. PASS. |
| PUB-2; P02/P03/P09 | Offer (`lib.rs:344`) preserves the first complete payload across valid proof variants; confirmation (`lib.rs:539`) checks key/source, actual index, complete-through bound and prefix identity (`lib.rs:647`). External harness exercises six false receipt projections, a different reader, q1 proof with q2 body, stable intent replay, ACK state and duplicate confirmation. PASS. |
| PUB-3; P06–P08 | Public stop and watermark (`lib.rs:596/622`) preserve history; prepare (`lib.rs:433`) disallows stopped/recorded output. External harness checks unrelated keys, cutoff 104 versus 105, header-only 105, prepared-intent suppression after stop, late history confirmation and late ACK. PASS for these transitions. Local TIME has no Publisher mutation API. |
| Real persistence; P04/P05/P10 | `store.rs:127` writes/syncs temp, renames and syncs directory before memory installation (`lib.rs:690`). External harness creates a real `state.tmp` directory obstruction, proves main bytes unchanged, then reopens the actual file; it also exercises writer exclusion, binding mismatch, corruption, missing main and ignored stale temp. Four separately reviewed recovery tests cover all durable task transitions and fault stages. PASS. |
| Commit unknown / prepared replay | `store.rs:167` poisons on post-rename directory-sync failure; every mutation/output path checks health first, including `prepare_attempt` at line 438 before its existing-PREPARED fast path. The recovery test begins with an actual saved PREPARED intent, faults a stop commit after rename, observes Poisoned on both replay and dispatch, drops/reopens, and reads durable STOPPED generation 3. PASS. |

The new external harness uses its own finite non-cryptographic fixture and codec, not Builder test support. It reads publication/body bytes from the supplied prefix; deliberately malformed receipt projections test core guards. No successful fixture is represented as cryptographic validation.

## Executed commands and results

Invocation from the worktree:

```sh
bash /home/jy/.codex/worktrees/1fe1/megaroation_poc/poc/verification/publisher-espresso/run-independent.sh
```

The checked-in script pins the candidate, rejects source differences, sets task-specific `CARGO_HOME`, `CARGO_TARGET_DIR`, `CARGO_BUILD_JOBS=2`, and runs these two non-nested lock commands:

```sh
flock /tmp/megaroation-poc-heavy.lock cargo test --manifest-path poc/verification/publisher-espresso/independent-harness/Cargo.toml --locked --offline -- --nocapture --test-threads=1
flock /tmp/megaroation-poc-heavy.lock cargo test --manifest-path poc/crates/publisher/Cargo.toml --locked --offline --lib recovery_tests:: -- --nocapture --test-threads=1
```

Toolchain: `rustc 1.95.0 (59807616e 2026-04-14)`, `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`. External harness: **6 passed, 0 failed**. Reviewed recovery-fault tests: **4 passed, 0 failed**. Both commands exited 0. Fixed cases use seed **N/A**; no randomized scheduler or wall-pause comparison was run. Reported test durations 0.10s/0.14s are wall execution cost, not protocol latency.

Raw evidence root:

`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/06/artifacts/verification/8ba78f1ace3b383caa1c4360401c645d25907326/independent-run1`

It contains environment/dirty-state evidence, both raw test logs, independent pending-reader/snapshot files, corrupt files, and recovery-boundary snapshots. `SHA256SUMS` covers 49 evidence files. The commit-unknown evidence is under `recovery-faults/pe-indeterminate-3256345-3/` (before snapshot and reopened publisher main); pre-rename snapshots and each task-transition snapshot are retained in sibling directories.

| Artifact | SHA-256 |
| --- | --- |
| `independent-harness.log` | `d52300d8324daa7da7e485a38a84728b36e0c7f2b60b96ba5d46782f3284a1e1` |
| `reviewed-recovery-tests.log` | `3ae096d70d8233fce0f494838ecff5fa3079b5369c515844b9bcc8b37ba34f9a` |
| raw `SHA256SUMS` | `881b45c9920eed281b0d83a730a690b6b30328bd1e5df65b61b8706df5dc6df0` |
| external `tests/independent.rs` | `c9b20f972051e09e475caf39d50fc72af190d9c685a1b1f5fb492c3d7ea9d4f9` |
| external `Cargo.toml` | `35f63f4c95ac3c39d74570944fbcc87f0b78eb26f9117cb11e3bb09fa2ef4939` |
| external `Cargo.lock` | `870e83af3e2133e3a06e570b05782a00e8ef0823f923dccd626680c1f00ddb4f` |
| `run-independent.sh` | `2ee6f4de268442dfd25bdebe5a2eada766d781e8b4480a66db6ea86b8393bb67` |

## Limits and handoff

This evidence is finite, process-local component testing with real files and explicitly injected commit failures. It is not a physical power-loss experiment, a multiprocess controlled runtime test or a proof of all schedules. The private fault hooks were reviewed and executed, not replaced by cloned-memory assertions; actual files are reopened after failures. The external harness independently exercises the public APIs and real filesystem obstruction.

PE16 seeds 7/42/99 and 0/2ms wall-pause pairing remain for Tester on **the same `8ba78f1` SHA**. No source or frozen oracle file was edited. Verification additions are deliberately uncommitted until Tester finishes; HEAD was not advanced.

Delivery note added by the lead after Tester completed: [the matrix](simulation-report.md) passed on the same candidate. The delivered `run-independent.sh` additionally supports source-identical documentation commits and resolves its checkout from its own path; `--exact-head` restores the original strict HEAD guard. It records the actual HEAD, rejects modified or extra component source files, and requires a fresh output directory. The wrapper hash above is the historical script used for `independent-run1`; this packaging adjustment does not change that run's SHA, logs, harness or test commands.

Full 06 remains **INCONCLUSIVE / pending integration**: D02 controlled grant/Ack transport and durable custody; D03 canonical object/codec and adapter validation; D04 native final certificates; D07 lifecycle-scope projection and independent q_avail/GFR; D16 official Espresso source verification. Late offers require the 07 adapter to re-project historical public exclusion. No crypto, native quorum, 15-worker, G acceptance or official dev-node claim is made.
