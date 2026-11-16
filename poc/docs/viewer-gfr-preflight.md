# 07 Viewer GFR intake — 2026-11-16

Status: source/contract preparation. This report is not feature acceptance.

- Working checkout: `/home/jy/.codex/worktrees/1e2f/megaroation_poc`; the chat-provided managed worktree is reused. Git metadata points to `/mnt/reth/jy/megaroation_poc/.git/worktrees/megaroation_poc7`.
- Clean starting commit: `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b` (detached). Created requested branch `codex/poc-viewer-gfr`.
- Normal merges, in order: 03 `e9c15e04bc83b335668f3ecd837367aa87344a89` (fast-forward), then 06 `4b034dfbb6c3e1c7143208347b546daab8dfe298` (merge). Clean integration baseline: `335553359cd39a3a7494fbd9760436cdb2c41a33`. No 09 merge, reset, push, or main integration.
- Regional BFT gitlink remains `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`. The initialized source submodule at `/mnt/reth/jy/megaroation_poc/regional_BFT_sim` has that same HEAD and is used read-only.
- v2 design SHA-256: `95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab`.
- Dispatch read from `/mnt/reth/jy/megaroation_poc/poc/docs/commander/dispatch-independent-wave-20261116.md`; dependency-plan-v2 and task 07/workflow handoffs read. The dispatch's final supplement accepts the exact 03 component; neither dependency is claimed merged into main.
- `.build` is a symlink to `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07`. It was absent before setup. No existing artifacts were replaced. Intake free space: root 12 GiB, data volume 1.4 TiB.
- Git metadata and the authorized data-disk directory required sandbox escalation because both are outside writable roots. Escalation succeeded; no user approval question was needed.

## Roles and ownership

The main-chat requested model/effort is GPT-6 Astra / ultra; the execution tools do not expose a main-chat model getter, so this report does not independently attest runtime model metadata. Named tools expose and launch `bft_spec` and `bft_verifier` as GPT-6 Astra / xhigh and `bft_builder` and `bft_simulator` as GPT-6.1 Sol / high. No substitute model or extra user chat is used.

- Docer: only `poc/docs/contracts/task-contract-viewer-gfr-v1.md`.
- Independent Verifier: only `poc/verification/viewer-gfr/`; derives expectations from sources before candidate inspection.
- One Builder, after contract freeze: only `poc/crates/viewer-gfr/`, including its independent workspace/lockfile and builder tests.
- Lead: dependency integration, interface decisions, this task's intake/release reports and commits. No authored changes to commander, shared types/root Cargo declarations, other functions, native recovery, or submodule.
- Tester, after independent candidate verification: new task-specific evidence directories and assigned simulation report.

All heavy builds/tests use `flock /tmp/megaroation-poc-heavy.lock`, two build jobs, the explicit Rust 1.95.0 toolchain, and 07's target/artifacts. No service or external endpoint is required for the pure-rule stage.

## Stage gates

First freeze source-derived contract and independent expectations, then publish a small interface/vector commit, implement pure rules, commit the candidate, independently verify it, and run the agreed finite matrix on the same verified source SHA. Scope includes actual 06 component readers and 03 Plan/Merge. Fixture proof inputs remain explicitly non-cryptographic. Native 04 FinalVerifier, Force/RESUME behavior, and complete system integration remain separate gates; unsupported controls must stop at their original position without partial effects.

Future RESUME lifecycle delivery must stage its position-bound cut/state and all Plan prerequisites before new-instance allocation/Enter, with atomic event/batch visibility. v10 Manager and GFR list lifecycle delivery in different orders; this stage records that integration issue rather than rewriting either source. Current regular/Expire behavior does not require a resolution of the future dispatch order.
