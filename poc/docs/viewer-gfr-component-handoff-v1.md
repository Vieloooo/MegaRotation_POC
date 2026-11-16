# 07 Viewer GFR pure component handoff v1

2026-11-16, Asia/Shanghai. **Authorized pure-rule component stage complete: M1/M2 verification and G1/G2 Tester matrix PASS. Full feature 07 remains OPEN.** This report records the pure-rule component stage, not full native or system acceptance. No complete-feature acceptance count, main merge or push follows from this milestone.

## Revisions, workspace and roles

| Boundary | Revision / result |
| --- | --- |
| Clean intake | `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b` |
| Accepted 03 dependency, merged first | `e9c15e04bc83b335668f3ecd837367aa87344a89` |
| 06 component dependency, merged second | `4b034dfbb6c3e1c7143208347b546daab8dfe298` |
| Normal merged baseline | `335553359cd39a3a7494fbd9760436cdb2c41a33` |
| Frozen contract and independent source expectations | `186501dbb8156a52bde1832dfaaa1a714c52bf68` |
| Independent exact interface vectors | `673d118fde34557f06f0bd3c447b4c799f601c7a` |
| First consumable M1 interface, independently checked | `e41b81a8d68d67216ea940beeb614135e808e95d`, 167 checks PASS |
| M1 report delivery (same crate source) | `4d83cb2e4d4b9fb532b110cd05d09144b251db2c` |
| Complete M2 source candidate | `de628c2e3536c6d7b10ae2f8513b7014623a0e45`, 117 independent checks PASS |
| Integrated test revision | **`8907ad64d882673d6ef2142defdf6bff2e208a71`**, same candidate/harness source; locked rerun 117 PASS, identical trace |
| Tester revision | **`8907ad64d882673d6ef2142defdf6bff2e208a71`**, 12/12 runs and 6/6 pause pairs PASS; no source changes |

Working checkout is `/home/jy/.codex/worktrees/1e2f/megaroation_poc`, branch `codex/poc-viewer-gfr`. The chat's existing managed worktree was reused. `.build` points to `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07`; cargo, target and artifacts are isolated there. The Regional BFT gitlink remains `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`. Commonware is 2026.10.0; Rust/Cargo are 1.95.0. The source checkout and initialized submodule were read-only references, except authorized Git metadata and the task's data directory. No reset, unrelated 09 merge, native recovery edit, or authored commander/shared Cargo/types/core change occurred.

Actual named roles were `bft_spec` and `bft_verifier` (GPT-6 Astra/xhigh), one `bft_builder` and `bft_simulator` (GPT-6.1 Sol/high). The main chat requested Astra/ultra; available execution tools do not independently expose its model metadata. No silent model substitution or extra user chat was used. The order was source contract → independently derived expectations → single Builder → independent candidate review → integrated revision verification → Tester. One writer owned each source/report path, and all heavy work used the host lock with two build jobs.

## Implemented and consumable

The standalone `poc-viewer-gfr` / `poc_viewer_gfr` workspace at `poc/crates/viewer-gfr/` consumes the actual 03 types, StaticScheduleProvider, metadata, DEP/execution and Merge, and the actual 06 finalized log reader. Its own manifest and lockfile avoid shared workspace edits.

- `init`, `advance_global`, `chain`, `responsibility`, and typed public snapshots/checkpoints implement the pure regular stage.
- GFR-POS-1 orders controls together by exact L1 position, then Expire, SessionEnd and Espresso at equal public key. GFR-LIFE-1 identifies complete lifecycle bodies without collapsing distinct same-pc events. GFR-PUBLIC-1 carries exact existing 03 canonical bytes with opaque final proof bytes.
- Complete Espresso watermark and L1 control coverage establish event order. Later L1 headers used for verification are tracked separately from control coverage. No wall clock or scheduler fault table drives the production rule modules.
- `Wait(None)` marks an unresolved input frontier. Once the next event is proved, missing actual material yields `Wait(Some(original_position))`. State, record, pc and all lifecycle effects commit together, including multiple handoffs within one Espresso event.
- Public absence of proof/body leaves a result ineligible; absent local delivery of an already public message waits. `q_avail` retains original public positions and requires every physical body from the selected anchor. Descendant proofs can cover ancestors, and extra proof variants do not move first completeness.
- Acceptance uses the original public deadline and the current Espresso key for every added virtual height, including padding. Actual 03 Merge validates native metadata/execution and retains raw transactions separately from Kept.
- Full-window and SessionEnd regular paths preserve their distinct final anchors, the predecessor's initial reference anchor, and the fixed successor window. Expire preserves G and enters HALT. Unsupported Force/RESUME stops at its exact position and exposes no partial transition.
- Two real component reader instances own separate cursors, public state and delivery schedules. Same-pc comparison excludes private receipt times and ahead downloads.
- Checkpoint initialization replays and checks the complete public projection and exact supporting inputs, reconstructing internal cursors and anchor materials. Its private initialization binding protects the typed in-memory configuration/profile/parameter identity. A bound earlier than current pc returns `HistoricalBound`.

Full API/codec semantics are frozen in [GFR-07-v1](contracts/task-contract-viewer-gfr-v1.md). The [M1 release](viewer-gfr-interface-release-v1.md) identifies the first independently consumable interface. The crate [README](../crates/viewer-gfr/README.md) gives runner behavior and bounds. Later Publisher/10/11/12/13 consumers must reuse the exported position, lifecycle and publication codecs.

## Verification and limits

| Evidence | Result and scope |
| --- | --- |
| M1 independent vectors/static review | 167/167 PASS, including 1,511 systematic truncated prefixes and a 64 MiB lifecycle capacity rejection; exact codec/API checks |
| M2 independent source-derived harness/static review | 117/117 PASS on `de628c2`; no blocking finding, no production or harness source fixes |
| Integrated verification | 19 candidate/harness source hashes matched; one `--locked --offline` rerun on `8907ad6` passed 117/117 with byte-identical output; lock unchanged |
| Builder checks | Locked check, all-target tests 2/2 and separate G1/G2 seed 7, pause 0 smoke passed; these are preliminary builder evidence |
| Final Tester | G1/G2 × seeds 7/42/99 × pauses 0/2 ms: 12/12 PASS, 6/6 complete stdout pairs byte-identical, no timeouts; raw-input/transition/lifecycle/execution checker PASS |

The Verifier froze 40 VG source cases and contract V01–V23. Codec V01 is carried from the unchanged M1 codec. Its historical M2 coverage records the wall-pause part of VG35 pending; the completed Tester matrix now closes that part for this bounded fixture profile. Unavailable real native V22 and future RESUME material-staging VG39 remain INCONCLUSIVE/OPEN. `VG40 PASS` means the explicit verification capability boundary behaves correctly; it does not mean native finality passed. The 117 checks are repeated on the integrated revision, not counted as 234 distinct cases.

The independent fixture extends finite mock L1/DEP inputs using fresh configuration hashes and 03's explicit finalizer/delivery capabilities. It is not an extension of accepted production L1 data or actual Anvil receipts. Final-proof verdicts are explicitly non-cryptographic and bind exact inputs. Real registry/Plan/Merge/reader components do not turn those verdicts into native certificates.

Independent review reports: [M1](../verification/viewer-gfr/m1-review.md), [M2](../verification/viewer-gfr/m2-review.md), [integrated identity and rerun](../verification/viewer-gfr/integrated-review-8907ad6.md), and [Tester matrix](../verification/viewer-gfr/simulation/simulation-report.md). New raw evidence is retained under:

```text
/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07/artifacts/verification/m1-e41b81a-independent-v1
/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07/artifacts/verification/m2-de628c2-independent-v1
/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07/artifacts/verification/m2-8907ad6-integrated-v1
/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07/artifacts/simulation/gfr-8907ad6-g1-g2-v1
```

M1 raw trace SHA-256 is `94f6f65dad04932d5c946992016844e76d2fd4d536cbd3e5a1b9e392a3aba415`; both M2 traces are `6084c4ae54d826a177ceb2a4d5eba718a74f5ee9aba3e838d0c28bffd5dbbc6c`. Machine-readable manifests pin source/harness/lock/raw files. The lead independently checked all 14 M1 raw-file hashes, 14 M2 raw-file hashes plus 19 source/harness hashes, and 15 integrated raw-file hashes, including actual byte equality of both M2 traces.

Tester raw trace hashes are G1 seed 7 `0990e115f76aea8e0a07252e0c83f478f026d1ea172cc1637d13e3b71af74457`, G1 seeds 42/99 `53eabf5b9d4b98ab59cefd3dac8be07ad62ede15d96b04a0d175af2fe979bc5f`, and every G2 run `6c4ba42753752599226376eb956ccaebf8e6dbaacbd464d09909ef55f5afd371`. Each seed's two pause variants have identical bytes. G1's cross-seed difference is a local missing-message diagnostic; source input rows and same-pc public states agree. The lead independently repeated all six byte comparisons and raw-source equality checks.

The independent trace checker compares A/B at **7 distinct completed positions per G1 run and 10 per G2 run**, totaling 102 distinct-position comparisons over the twelve runs. G1 has eight paired scenario labels, but its unsupported RESUME retains the previous pc, so it is not an eighth distinct position. The tested replay binary SHA-256 is `0544600279839b71d89b8a8d363640aea7f4e81af140654ff35b1e321ed76524`. The machine-readable [summary](../verification/viewer-gfr/simulation/summary.json) retains each exact command, seed, pause, result, source/trace hash and local wall duration.

Tester summary SHA-256 is `13963d703e2ce5c7151695ae6ca93ef21ee6063fff145281729e476b0a1b02fb`; its readable report is `6005a4a1c9a41b777335244a053ba8f5850b6948f33e338f5f56fd10611dbe0f`. The lead verified all 24 trace/stderr hashes, capture and binary hashes, twelve source-row hashes, and the 102-position count. The Tester also independently reconstructed accepted native hashes, Kept and account state from raw bodies and deposits. Per-fragment reader receipt ticks and private snapshots are not present in JSONL; the report specifies the deterministic delivery schedule/counters and this limitation. No real network timing is measured.

## Reproduce

From this worktree with the prepared data-disk cache:

```sh
export PATH=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:"$PATH"
export CARGO_HOME="$PWD/.build/cargo"
export CARGO_TARGET_DIR="$PWD/.build/target"
export CARGO_BUILD_JOBS=2
flock /tmp/megaroation-poc-heavy.lock cargo run --locked --offline --manifest-path poc/verification/viewer-gfr/independent-harness/Cargo.toml --bin m2
flock /tmp/megaroation-poc-heavy.lock cargo run --locked --offline --manifest-path poc/crates/viewer-gfr/Cargo.toml --example replay -- G1 7 0
```

The agreed matrix is G1 and G2 × seeds 7, 42, 99 × wall pauses 0 and 2 ms, with six byte-comparison pairs. G1 uses Espresso q0–q5, G2 q0–q7; both use finite L1 heights 0–30 and DEP heights 1–40. G1 reaches h20 and stops at an unsupported RESUME after HALT; G2 reaches h11 with h10 padding and preserves that prefix at Expire. Public timestamps, local delivery ticks and wall pauses are distinct. These finite traces assert no wider BFT safety/liveness or timing theorem.

## Open integration gates

1. **04 real native FinalVerifier/certificates:** unavailable. The public readiness capability must complete before the existing bool checker; an expected profile is bound even while the checker is absent. A `Native` label alone is never acceptance evidence.
2. **Actual controls and recovery:** 09's authenticated nonempty controls, 10 Force and 12/13 historical ResumeCut/RESUME handlers are not implemented here. Current controls are explicit unsupported inputs, never silently skipped.
3. **Future lifecycle dispatch:** v10 GFR lists SessionAssigned before Resumed while Manager §3.10 requires recovery StateMaterial before allocation. Preserve source event records and stage all cut/state/Plan inputs for an atomic batch before allocation/Enter. Actual dispatch, durability and acknowledgments remain a 13 integration gate.
4. **System and durable recovery:** real producer regular success, 15-worker controlled integration, Publisher output/lifecycle wiring and durable process checkpoint recovery remain separate acceptance. Current checkpoint support is typed and in-memory.

The latest restart-free routing decision (`0c15aac`, handoff `32a3eb6b31ff8c1dfe2d05dbc2ed26c0c031bb77`) was read without unrelated merges. This stage does not depend on complete 01 recovery. Future genuine 04 certificates may come from a continuously running healthy-worker profile while still requiring clocks, leases, timers, grants/Acks, actual signatures, preparation and Close gates. That route does not close 01's recovery matrix or make this component full feature 07 accepted.
