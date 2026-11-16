# H3 Slice A acceptance

**PASS** for H3-A-1 at `e59d026e8c08b64ff744b57c63cc7756f0b34019` on `codex/h3-publication-gfr`. All five agreed executions exited 0. The capture recorded zero timeouts and zero incomplete cases. HEAD and the 125 captured input hashes matched before and after the batch. [Capture index](manifest.json)

| Run | Seed | Wall pause | Checks passed | Local wall elapsed |
| --- | --- | --- | --- | --- |
| Independent Slice A harness | N/A, fixed inputs | N/A | 14/14 | 104.607 s |
| G1 | 42 | 0 ms | 7 common replay positions | 65.832 s |
| G1 | 42 | 2 ms | 7 common replay positions | 1.018 s |
| G2 | 42 | 0 ms | 10 common replay positions | 0.616 s |
| G2 | 42 | 2 ms | 10 common replay positions | 1.067 s |

The harness used the actual 06 file Publisher, SimEspresso source and reader, with 03 registry and execution fixtures. `SYNTHETIC_NATIVE_BOUNDARY_NOT_CRYPTO` labels the exact-tuple final-proof and context callbacks, including cases routed through the Native profile. The fixed source prefixes extend through q2, with L1 heights 0–30 and DEP heights 1–40. The frozen upper bounds are 49,199 encoded proof bytes and 51,560 encoded attempt bytes. [Frozen contract](../../../../docs/contracts/h3-publication-gfr-slice-a-v1.md)

The replay finality profile was `gfr-07-noncryptographic-rule-fixture-v1`. The original 07 checker’s `semantic_trace` validated the raw source objects, execution, lifecycle order and A/B state by completed public position. Every stdout stream, including the source row, matched the corresponding seed-42 capture at accepted 07 `8907ad64d882673d6ef2142defdf6bff2e208a71`. Each scenario also produced byte-identical stdout under 0 and 2 ms local pauses. [Accepted 07 summary](../../../viewer-gfr/simulation/summary.json)

- **G1** reached height 20 and HALT. It reported unsupported RESUME at synthetic L1 `(26,1,0)`, public key `64,000,000 µs`, retaining q4 at `61,000,000 µs` as the last completed replay position. The fixture source covered q0–q5.
- **G2** retained height 11 and HALT through q7 at public key `52,000,000 µs`. The successor became publicly complete at q5 and entered G at q6 after SessionEnd at `24,000,000 µs`. The fixture source covered q0–q7.

The table measures local elapsed time around each Cargo invocation within the acquired heavy-task lock. The first harness and replay commands included builds. Public keys and simulated proposal/finalization ticks remain separate fields in the JSONL. Real-network measurements were absent.

Actual H1 final-proof cases, historical validation after Close/ClockUnsafe with genuine evidence, controlled H1 runtime and H2 integration, and healthy M1 remain **OPEN**. This batch covers finite component and replay fixtures only. It gives no end-to-end timing or liveness result.

## Replay recipe

From the same worktree at the verified commit, use a fresh task07 artifact basename. The runner uses Rust 1.95.0, two build jobs, and locked offline Cargo. It uses one host lock for all five commands and gives each process a 180 s watchdog. `manifest.json` in the artifact directory contains the exact Cargo arguments, environment, binary hashes, source hashes, statuses and raw output hashes. [Runner](run.py)

```sh
flock --exclusive --timeout 60 --conflict-exit-code 75 /tmp/megaroation-poc-heavy.lock \
  python3 -B poc/verification/publication-gfr-healthy/slice-a/tester/run.py \
  h3-a-e59d026-acceptance-replay-v1
```
