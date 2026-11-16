# Independent slice A review — e59d026

**PASS for slice A's implemented component scope. No candidate defect was found in the inspected paths and executed cases.** Candidate `e59d026e8c08b64ff744b57c63cc7756f0b34019` stayed at HEAD throughout the runs. Production code, legacy acceptance artifacts and the14-test harness source match that commit. Untracked B/C documentation/preparation and this review's new lock/evidence were separate from the production candidate.

Source expectations and literal vectors preceded candidate inspection. The review used the frozen H3-A-1 rules and exact dispatch; it did not derive expected outcomes from Builder explanations. No production files were changed. The independent harness compiled and passed on its first execution with **no test-source fixes**.

## Executed evidence

| Command recipe | Result | Scope |
| --- | --- | --- |
| `bash .../e59d026e8c08b64ff744b57c63cc7756f0b34019/run-independent.sh` | **14/14 PASS**, exit0 | Actual03 registry/PoP/Plan/metadata/Merge,06 file Publisher and independent source readers, declared07/H3 adapters; explicitly synthetic exact-tuple final-proof/context capabilities. |
| `bash .../e59d026e8c08b64ff744b57c63cc7756f0b34019/run-legacy.sh` — codec binary with `--capacity` | **167/167 PASS**, exit0 | Existing independent07 position/lifecycle/publication codecs and capacity vectors. |
| Same legacy recipe — `m2` rule binary | **117/117 PASS**, exit0 | Existing independent07 pure replay/completeness/regular/Expire/WAIT/checkpoint/readers regressions with their original explicit noncrypto limits. |

The exact executed scripts and raw outputs are archived at `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07/artifacts/verification/h3-a-e59d026-independent-v1`. Its relocation manifest retains original executed workspace paths/commands; all13 raw files were copied and hash-verified before their redundant new workspace copies were removed. No test was rerun for relocation. Only this versionable review and result manifest remain here.

The archived recipes contain exact commands, absolute workdir, approved task07 Cargo/target paths, Rust1.95.0 and2 build jobs. Every heavy command acquired `/tmp/megaroation-poc-heavy.lock`; runs were serial. The harness's own lock was resolved offline and then used with `--locked --offline`. The frozen legacy lock was reused without regeneration. Tool versions, lock hashes, source/harness hashes, post-run binary hashes, raw stdout/stderr and exit files are sealed in `result.json`.

## Source-rule and code audit

| Rule | Candidate location | Impact checked / replay evidence |
| --- | --- | --- |
| v10 09 §§3.2/3.4: earliest proof/body completeness, original positions and no future-data leakage | `viewer-gfr/src/publications.rs:102`, shared evaluator`:361` | Fresh wrapper accepts sealed06 prefix plus explicit cut, reindexes only that cut and does not trust external verdicts. Public-cut, reverse delivery and original-position tests pass. GFR and Publisher call the same evaluator. |
| Frozen07 §§3/9: immutable verification profile, missing-basis distinction and trusted historical state | `publications.rs:425`; `publication-bridge/src/verifier.rs:100` and`:241` | Basis WAIT/REJECT makes no bool-checker calls. Native mismatch rejects and production constructor rejects fixture profile. Sealed internal GFR cache advances an empty later q with missing optional binding, while the fresh public wrapper returns WAIT. |
| v10 04 §3.4/PUB-1;03 Ready/provenance and execution rules | `publication-bridge/src/verifier.rs:155`, Merge at`:262` | Complete Offer checks subject, context/anchor and actual03 execution. Missing L1/context preserves Verifying; synthetic-valid proof with incorrect pre-state root rejects. This is adapter evidence, not proof that a future H1 ContextVerifier is correct. |
| v10 04 §3.5/PUB-2 vs09 responsibility | `publication-bridge/src/verifier.rs:332`, invalid-proof mapping`:365` | Unselected candidate may be RECORDED while actual GFR stays unchanged. q1 proof/q2 body yields receipt positionq1 and complete_through_q2. An invalid public proof leaves NotYetPublic; a later valid variant records successfully without poisoning the task. Wrong source/reader cannot record. |
| Frozen06 durable task/StateCodec and H3-A-1 bounds | `publication-bridge/src/result.rs:37`, decoder`:123`, ancestry`:177`, public emission`:238` | Partial private store roundtrips; real file reopen retains stable attempt and first complete payload. Same-key proof variant does not replace it. Independent store/public literals and malformed versions/lengths/counts reject as expected. Four-transfer body emits197 bytes; fifth transfer and overlong proof reject. |

The candidate retains Native-only production dependency injection without an exported permissive/default verifier. Historical checking has no live-engine/current-clock/lease/Selected dependency. The public helper performs no G mutation and adds no submit/network path. Existing acceptance projections passed all checked legacy regressions.

## Limits and remaining gates

The14 new tests use fixed inputs, not randomness; seed is N/A. Their source logs have at most q0–q2, selected results through physical3 and an unselected successor physical11. Actual03 finite L1 input is0–30 and DEP input1–40. Private codec literals are structural fixtures, not signed results. The old suites retain their own finite bounds and explicit coverage statuses in their raw JSONL.

Final proofs are **synthetic exact descriptor/block/proof tuples**, even where the production Native profile is exercised. Real registry/PoP checking does not substitute for real H1 final-certificate verification. No real3/4-signature quorum case,15-worker process, root grant/Ack custody, H1 ReadyToken, B stop/transport path, official node, wall-pause matrix, Force, D/RESUME, OS crash/recovery or bounded-liveness claim was run here. Legacy V22/native and future RESUME/wall-pause gates remain explicitly INCONCLUSIVE.

This PASS permits the Tester to run agreed slice A cases on the **same exact candidate SHA**. It is not healthy M1 or full H3 acceptance. Any production fix or interface change requires affected independent re-verification before that handoff.
