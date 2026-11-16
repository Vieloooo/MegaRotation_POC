# M1 narrow integration intake — f9220e

**Verdict: ACCEPT this H2 pure-component intake for Simulator replay on exact integrated SHA `f9220e7811b42fe137a96f93143b38246d201565`.** No blocking integration finding was observed. Approved next scope is the two H2 Cargo suites plus a fresh mechanical M1 preflight/check at this SHA. **M1 remains NOT_RUN; no production port is approved by this intake.**

This was read-only review in `/home/jy/.codex/worktrees/5d7a/megaroation_poc`. HEAD matched the named SHA and Git status was empty. I ran no Rust build/test, worker, official node or repeated 101-case mechanical matrix. The accompanying machine-readable comparison is `/tmp/m1-integration-evidence-f9220e.json`.

## Source-first boundary

Before inspecting the integration delta, I reused the already frozen M1 independent expectations E04 (exact readiness/once Enter/same-instance switchback), E05 (monotonic close/old task retention) and E12 (complete ReplayRecord and pending/applied obligations), then read H2's `H2-REDUCER-1` contract and its pre-implementation source-first expectations. Their SHA-256 values match the accepted H2 evidence:

- Contract: `fe2f962d8480a0cb6b5d0bbb26ff6748d3e7149dfa6e6b05a9732c3c91c70a96`.
- Source-first reducer increment: `9ad71186b41e85a5e30100a7fd63d9a009435e12e4459046e3056ad8cd7965e5`.
- Independent matrix: `4d78b0c12f65f7be5f0c5a2da0048b7815f3ef9e027290e942960ac39cd87395`.

The accepted slice is explicitly a deterministic in-memory reducer under trusted validated-input/commit-attestation assumptions. It checks identity, lifecycle, record/cursor and action/receipt logic; it does not authenticate native execution or make an input hash/CommitRef into proof of durability. This intake checks preservation and consumability of that reviewed slice, and does not duplicate the existing protocol review.

## Exact history and content equivalence

The merge parents are exactly:

1. Previously verified mechanical harness: `1ade576f2312a75c3b9652eb8147d47d0ad6cb09`.
2. H2 delivery: `b639076fb50f5fcbc06e309f67b1fb6125697242`.

M0 `35d382ac445cc8bc779f0d18fd23cb57e54680da` is an ancestor of H2's actual tested candidate `2340f9c1c8cbc3bc4be57a5c18f884cc74390563`; tested → release → integration ancestry is intact. The release adds only five report/handoff files relative to that tested candidate, with no source, test, contract or lock change.

| Object | Actual tested 2340f9c | Release b639076 | Integrated f9220e |
| --- | --- | --- | --- |
| `poc/crates/session-manager` tree | `78a9250a93da0c5db78578f19951bad5c04f6b80` | identical | identical |
| `poc/verification/session-manager-healthy/reducer-v1/independent-harness` tree | `0903cacaf9ac8ac603ce56fa22d66d2ce89a32ed` | identical | identical |

The delta `1ade576..f9220e` contains **35 added files only**, all within H2-owned crate/verification paths or its four designated contract/preparation/handoff documents. No existing path was modified, removed or renamed.

The complete Git objects for `poc/crates/types`, `poc/crates/core`, `poc/crates/viewer-gfr`, root PoC `Cargo.toml`/`Cargo.lock`, `poc/proto`, the i1b runtime lock and Regional-BFT gitlink are identical across M0, tested H2, release H2, prior M1 and this integration. Existing shared types/core/GFR/lock/proto behavior is therefore unchanged by this merge. The gitlink remains `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`.

All five previously verified M1 harness files have identical Git blob IDs and SHA-256 bytes at `1ade576` and `f9220e`: scenario README, `evidence.py`, `run.py`, verification `check.py`, and `mechanical_tests.py`. No Builder test source was used as an independent oracle in this intake. M1 contract and source-expectations hashes are unchanged.

Both new H2 manifests are standalone workspaces using relative dependencies on the existing public crates; root workspace manifests/locks were not edited. Their Commonware lock entries are all 2026.10.0. These added manifests/locks will be captured by the existing M1 mechanical source inventory at the new commit, so a fresh preflight/check is appropriate even though the harness bytes are unchanged.

## Existing test evidence received, without relabeling its execution SHA

Read `poc/docs/h2-session-manager-reducer-handoff-v1.md`, the independent candidate review and Tester report. The handoff at line 3 limits PASS to finite trusted-input pure-component traces; lines 23 and 63 explicitly retain real root/aux/guard and H1/H3 obligations as OPEN.

I independently read and rehashed the original evidence files:

- `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/h2-manager/artifacts/independent-2340f9c-v1/manifest.json`: matches `c351fdfbbea1f318d99fbe113e3e0bf49fe7bead8e7a3904c33398aeaaf2b566`.
- `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/h2-manager/artifacts/tester-2340f9c-v1-3/summary.json`: matches `a67ffdedb930c07dba84bb68037662fc72bc3f2da5c029f21ebaef82e291735a`.

All **22 recorded tested input hashes** match actual files in `f9220e`; the recorded before/after input sets are equal. Existing evidence reports component 8/8 and independent 27/27 at `2340f9c`, with the independent suite repeated once at that same SHA. Those are 35 distinct tests, not 62 distinct tests. The earlier interrupted toolchain/lock-wait attempts remain explicitly incomplete. I did not rerun or rename those old results as executions on `f9220e`.

Preserved limitations include trusted preparation/commit observations; no real ReadyState execution, native signatures, physical engine/future/deadline preservation, fsync/root transaction, controlled grant/Ack, H3 public replay, 15-worker system or official publication. The larger action/dependency/byte-cap bounds have the existing conditional/static coverage limits; unbounded long-lived commit-history behavior is not newly certified here.

## Approved Simulator replay scope

Run only on clean `f9220e7811b42fe137a96f93143b38246d201565`, record that actual SHA and fresh output artifacts. Use Rust/Cargo/Rustdoc 1.95.0, explicitly set RUSTC/RUSTDOC and toolchain PATH (the earlier H2 rustup-shim issue must not recur), `CARGO_BUILD_JOBS=2`, and assigned data-disk cargo/target/artifact roots. Acquire the shared heavy lock once per Cargo command:

```text
flock /tmp/megaroation-poc-heavy.lock /mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/cargo test --manifest-path poc/crates/session-manager/Cargo.toml --locked --offline -- --test-threads=1

flock /tmp/megaroation-poc-heavy.lock /mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/cargo test --manifest-path poc/verification/session-manager-healthy/reducer-v1/independent-harness/Cargo.toml --locked --offline -- --nocapture --test-threads=1
```

Expected suite shapes are 8 component tests and 27 independent tests (5 capacity + 22 original), no failures. This is authorization to replay the accepted finite component suites, not a statement they have already run on the integrated revision.

Then prepare a fresh M1 input with candidate exactly `f9220e...` and real copied contract/source expectations, and run:

```text
python3 -B poc/scenarios/m1-healthy-regular/run.py preflight --candidate f9220e7811b42fe137a96f93143b38246d201565 --input INPUT_JSON --output NEW_PREPARATION_DIR
python3 -B poc/verification/m1-healthy-regular/check.py --bundle NEW_PREPARATION_DIR --expected EXPECTATIONS_JSON --output NEW_CHECK_REPORT_JSON
```

The expected mechanical outcome remains exit 3 / NOT_RUN / NotReady; BundleIntegrity may pass once complete evidence is checked. H2's delivered component can be listed with its exact release/review/contract references, but that alone cannot set `h2.manager` Ready. The H1 runtime/session adapter, actual H2 durable-owner bridge, H3 publication/replay bridge and independent production protocol oracle are still absent. No arbitrary binding argv may execute, and no synthetic protocol result may be added to obtain PASS.

## Review commands and limits

Read-only checks used `git status/rev-parse/show`, `git merge-base --is-ancestor`, `git diff --name-status`, exact `rev:path` tree/blob comparisons, SHA-256 of original evidence and every recorded tested input, and TOML inspection of the two H2 manifests/locks. Full changed-path list, object IDs, file hashes, ancestry outcomes and recorded suite argv are in `/tmp/m1-integration-evidence-f9220e.json`.

No source requirement was waived. This narrow intake preserves the previous independent review because the source/test/dependency inputs are unchanged, and authorizes reproduction on the new integration SHA. It does not close M1's real regular/SessionEnd/R2-first-G/official same-byte endpoints, nor Force, OS restart, D/RESUME, batch/DA or Web.
