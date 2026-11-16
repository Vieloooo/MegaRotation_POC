# Independent verification — d0afeee, Stage A

Date: 2026-11-16 (Asia/Shanghai). Verifier role: `bft_verifier`, configured GPT-6 Astra / xhigh. Result: **PASS for the frozen Stage-A component; no actionable source finding**. Full feature 12 and Stage B remain open.

Candidate: `d0afeee9363ef6703f51eecfd742915f83aa27f2`. Interface: `3b1b34fce6f3a3bac5099d53bdf5f7df56079f80`. Independent expectations preceded implementation and were included at `4a86f8d`. Baseline after normal dependency merges: `37b4f2b8a6d67b303ba90b1d4d1981c93ed8dd12`. Worktree: `/home/jy/.codex/worktrees/29df/megaroation_poc`, branch `codex/poc-derivation`.

The verifier first read the frozen sources and prepared 14 semantic fixtures, then reviewed the interface, and finally reviewed this committed candidate. The four builder unit tests and full real-run script were executed only after source review started. Independent checks below add a receipt-based oracle and a public-API harness; they do not rely on the builder's pass report.

## Results and evidence

Fresh original run directory:

`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/12/artifacts/derivation-verifier-d0afeee-run1`

The run started on a clean candidate. Its `revision.txt`, empty `worktree-status.txt`, `toolchain.txt`, `binary-sha256.txt`, `validation.log`, and per-run raw bundles are retained. A first sandboxed launch failed before creating that directory because the data disk was read-only in the sandbox. The authorized escalated launch succeeded. No alternate candidate or pre-existing run was substituted.

The full script passed formatting, four Rust unit tests, locked/offline builds, pinned Foundry compilation and ABI comparison, eight fresh private-Anvil runs, A/B replay, wall-pause comparison, same-block checkpoint replay in a separate process, and the missing-receipt/reconciliation profile.

| Fresh case, each at 300 and 650 ms wall pause | L1 blocks | Receipts | Final D height | Cuts | Key observation |
| --- | ---: | ---: | ---: | ---: | --- |
| window | 52 | 6 | 24 | 1 | D20 at frontier49; block50 adds 21/origin10 and22/origin11; block51 adds23 and24. |
| ahead | 47 | 8 | 14 | 3 | At block45, first target8 preserves D10; second target11 fills D11; end of block adds D12, then block46 adds13–14. |
| cap | 5 | 6 | 30 | 1 | Block3 target30 fills only through origin3, including output beyond ordinary drift and including-L1 time. |
| accepted_wait | 42 | 7 | 0 | 0 | Actual opaque acceptance at `(40,0,0)` blocks the later marker and leaves complete frontier39. |

The new independent `check_raw.py` reuses the already-independent feature-09 ABI/signature receipt checker and the frozen feature-03 execution/encoding reference. It consumes full raw blocks and receipts, reconstructs acceptance and Set/Resume order, then independently computes Stage-A fallback/fill, state, native bytes/hash, physical parent links, virtual pre-roots, and exact cuts. It imports no feature-12 production code and no builder checker.

Across eight runs it checked 292 raw blocks and 54 raw receipts, and compared 136 derived blocks and 10 cuts **per reader** (272 block comparisons and 20 cut comparisons across A/B). Altering a native hash or cut target caused rejection; removing a receipt caused the independent receipt checker to reject. This rechecks the real Certifier/DA signatures on the opaque acceptance, but does not verify a feature-11 payload or DA persistence.

The new public-API Rust harness independently replays captured genuine RPC replies through the actual feature-09 `Driver` and then invokes the candidate library. It restored all 111 action checkpoints of the ahead trace against a longer verified input prefix. Twenty-one separate checkpoint mutations were rejected: epoch, output flag, action count, window, source genesis hash, genesis native hash, genesis-origin header, last log, covered hash/gap/order, open cursor index/stamp, execution state, native parent/hash, physical parent, virtual pre-root, cause position, cut digest and cut position. Three genesis changes and wrong version/zero window/corrupt deposits also rejected. Missing replay input returned WAIT; changing source-window identity rejected without changing state; restoration using reader B agreed with A.

The harness additionally used only the accepted finite 03 fixture's rows 0–5 for public `FixtureSource` tests: no input waits, header5 without complete controls stops at frontier4, supplying its committed empty controls produces origins3 then4, and a changed header rejects. These are explicitly bounded fixtures, not extra real Anvil receipts or extended verified 03 history.

Independent result files:

- `raw-check-d0afeee.json`: eight raw-source checks and negative controls.
- `api-check-d0afeee.json`: public API checks and mutation names.
- `evidence-manifest-d0afeee.json`: absolute paths, sizes and SHA-256 hashes of raw artifacts and review support files.

## Source-to-code review

`source.rs:26` binds the 03 config to the configured 09 source identity, reader and genesis. `source.rs:64` admits only successful named-reader complete-block queries within finalized/visible coverage; `VerifiedInputs` has no public event/JSON constructor. The finite fixture path remains separate. Source authenticity retains the authorized trusted private-Anvil/deployment-config premise; this is not Ethereum consensus/MPT verification.

`derive.rs:111` checks full configured genesis, state/native hash, config/DEP binding and canonical origin. `derive.rs:398` processes protocol events before marking a block complete and attempting fallback. `derive.rs:480` uses inclusive complete-window coverage, a required next origin, actual parent and first-output evidence. There is no local clock, G, Espresso or hidden scheduler input in these paths.

`derive.rs:558` checks the precise same-transaction Set/Resume pair, stages the entire fill, caps next origin by the including block, selects the target's own state, then commits one immutable cut. `derive.rs:691` uses actual D state and native parent with existing 03 execution and hash code, allowing the specified empty drift exceptions. `derive.rs:188` reconstructs each checkpoint by replay to its exact completed action count and compares every stored field; a local checksum is not treated as semantic validation. Reader name and delivery order do not enter public cut identity or native hashing.

## Contract matrix

All statuses below are Stage A only. A PASS may combine code review, explicitly marked semantic fixtures, fresh raw receipts, and independent API checks; it is not a claim that every malformed-input scenario is representable by the sealed production source.

| ID | Stage-A result | Evidence and boundary |
| --- | --- | --- |
| DA01 | PASS | Public-API genesis/config/DEP negatives, missing initialization inputs, real configured source; constructor/source review. |
| DA02 | PASS, batch clause pending | False genesis evidence and output transitions in unit/reference/finite-source cases; tampered flags reject. Ordinary adoption is Stage B. |
| DA03 | PASS | Actual block49/50 boundary from origin10; missing next header is a semantic fault fixture. |
| DA04 | PASS | Slot2/slot4 reachable first-output cases in frozen oracle and unit tests; real window trace verifies subsequent epoch boundary. |
| DA05 | PASS | Real reader-B gap/reverse reads; genuine receipt withheld only during offline RPC replay; finite header-only source check. |
| DA06 | PASS for gate | Fresh successful opaque acceptance stays at exact position across further delivery and restore. Resolving actual DA is Stage B. |
| DA07 | PASS | Raw receipt checker validates failed/no-event/unrelated logs; Force/init produce no D effect in fresh traces. |
| DA08 | PASS | Independent raw-source recomputation of actual-parent state/native/physical/virtual values, 136 blocks per reader. |
| DA09 | PASS | Finite exhaustion, credit overflow, and window overflow in unit cases; corrupt DEP rejected through public API; source/execution arithmetic review. |
| DA10 | PASS | Actual ordered CursorSet fills in all non-accepted scenarios; cursor origin0 is not blindly selected. |
| DA11 | PASS | Real cap trace: D30/origin3 at including block3; time1030s exceeds including1006s and drift15s. |
| DA12 | PASS | Actual cursor5/D10/target8 at `(45,0,1)` preserves D10 and anchor state 8, independently recomputed. |
| DA13 | PASS for no-batch slice | Distinct actual block45 markers yield prefix 10/anchor 8 and prefix 11/anchor 11; later fallback cannot mutate them. Actual batch between markers is Stage B. |
| DA14 | PASS | Unit semantic pair corruption/missing-pair checks; public exact-query/non-Resume/global-index/future checks; independent raw pair/position checks. |
| DA15 | PASS | Unit semantic missing-header/pending-deposit/exhaustion/credit-failure injections leave pre-marker checkpoint unchanged; actual missing receipt prevents entry until reconciliation. |
| DA16 | PASS | 111 public-API action checkpoints, 21 tamper variants, A/B restore, missing materials WAIT, plus separate-process stored checkpoint replay. |
| DA17 | PASS | A/B canonical equality, repeated accepted WAIT idempotence, two wall pauses, changed source/header rejection and unchanged state. |
| DA18 | PASS | Exact candidate, fresh outputs, commands, tool/binary hashes and this evidence manifest. |

## Commands and scope limits

Working directory for all commands was `/home/jy/.codex/worktrees/29df/megaroation_poc`. Environment:

```sh
export PATH="/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:/home/jy/miniconda3/bin:$PATH"
export CARGO_HOME="$PWD/.build/cargo"
export CARGO_TARGET_DIR="$PWD/.build/target/derivation"
export CARGO_BUILD_JOBS=2
export L1_FOUNDRY=/mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0
flock /tmp/megaroation-poc-heavy.lock bash poc/crates/derivation/tests/validate.sh \
  /mnt/reth/jy/megaroation_poc/.build/commander-jobs/12/artifacts/derivation-verifier-d0afeee-run1
/home/jy/miniconda3/bin/python3 poc/verification/derivation/candidate_review/check_raw.py \
  .build/artifacts/derivation-verifier-d0afeee-run1 \
  poc/verification/derivation/candidate_review/raw-check-d0afeee.json
flock /tmp/megaroation-poc-heavy.lock cargo generate-lockfile --offline \
  --manifest-path poc/verification/derivation/candidate_review/api_harness/Cargo.toml
flock /tmp/megaroation-poc-heavy.lock cargo run --locked --offline \
  --manifest-path poc/verification/derivation/candidate_review/api_harness/Cargo.toml -- \
  .build/artifacts/derivation-verifier-d0afeee-run1 \
  > poc/verification/derivation/candidate_review/api-check-d0afeee.json
python3 poc/verification/derivation/independent_reference.py --check
```

The initial harness lock generation and first run shared one `flock ... bash -c` section; the final harness run used the locked command shown. Tools: Rust/cargo 1.95.0, Python 3.12.8, coincurve 20.0.0, Foundry 1.0.0, solc 0.8.20. Seed: `none (explicit scripts)`. Wall pauses are diagnostics; L1 timestamps and finalized/delivery progress come from explicit commands. The completed source candidate and frozen expectations were not modified; only the newly assigned verification paths were written.

No feature-11 codec, accepted-payload derivation, DA recovery/persistence, ordinary batch adoption or substitution, actual batch between cuts, feature 13 GFR/RESUME integration, or controlled 15-worker runtime is accepted by this report. The opaque accepted receipt proves the Stage-A stop gate only. No Batcher crash was simulated; a delayed/missing receipt profile is labeled as such. D checkpoint/replay and the archived-input process invocation are not BFT same-cid restart evidence. Full feature 12 remains **INCONCLUSIVE pending Stage B and downstream integration**.
