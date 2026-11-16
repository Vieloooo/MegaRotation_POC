# Feature 11 intake — 2026-11-16

The authorized stage is the independent Batcher/Certifier/DA service component, with a reusable frame/DA/accepted-record interface for feature 12. This does not accept full feature 11 or complete G′. No main merge or push is authorized here.

- Worktree: `/home/jy/.codex/worktrees/f025/megaroation_poc`; branch: `codex/poc-batch-services`.
- Clean detached start: `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`.
- Normal merges, in order: accepted 03 `e9c15e04bc83b335668f3ecd837367aa87344a89`, fixed 09 `c443ae925ed549b046731c2d8ef3dd9ce0311d24`.
- Integrated baseline: `c9805f7a818bc23d30018ce4dc6f1ee45f92e3ac`. No unrelated 06 dependency was merged.
- Regional BFT gitlink: `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`; no submodule mutation.
- v2 design SHA-256: `95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab`.

The initiating dispatch was read at `/mnt/reth/jy/megaroation_poc/poc/docs/commander/dispatch-independent-wave-20261116.md` (management commit `ed186199fc64bf8e96935a5598bdbee61af00352`) alongside dependency-plan-v2 and the task handoff. Source checkout files remain read-only references. Git operations in the managed worktree necessarily update its shared Git metadata; the sandbox-approved branch and normal merges do not modify the source checkout's files or rewrite history.

## Roles and ownership

The actual main turn context `01a1e1cc-4fd7-7843-b3c5-a0c538eba150` reports `gpt-6-astra`, effort `ultra`. Named tool roles are available: `bft_spec` and `bft_verifier` use Astra/xhigh; `bft_builder` and `bft_simulator` use GPT-6.1 Sol/high. No role substitution is planned.

Docer owns `poc/docs/contracts/task-contract-batch-services-v1.md`; Verifier owns independent expectations, vectors and reviews under `poc/verification/batch-services/`; one Builder owns `poc/crates/batch-services/` including its independent workspace/lockfile. Lead owns this intake, release reports, interface decisions and Git integration. Tester receives a separate report/artifact subdirectory after the same committed source candidate passes review. No changes to shared root Cargo/types/core declarations, feature09, native recovery, commander documents, or Regional BFT.

## Environment

`.build` links to `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11`. Its Cargo cache copies only the public registry from feature03, preserving existing files. Target/artifacts/PIDs are feature-local. Root filesystem had 12 GiB available; the data disk had 1.4 TiB. Rust is `1.95.0 (59807616e 2026-04-14)` from `/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin`. Foundry tools are in `/mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0`; `/home/jy/miniconda3/bin/python3` has coincurve 20.0.0 for independent signature checks.

Every build/heavy test uses `flock /tmp/megaroation-poc-heavy.lock`, `CARGO_BUILD_JOBS=2`, an independent target and a new run directory. Runs may need targeted sandbox escalation for the authorized data-disk path and private loopback services. This intake is environment evidence, not component acceptance.

## First gate and explicit limits

Freeze source-derived task contract and independent expectations before implementation. Reuse 03 canonical values/transactions and 09 exact EIP-712 domains, signatures, commitment and receipt semantics. Feature11 owns one P0 frame encoding. Full ReplayPosition semantics/encoding remain feature07-owned; opaque fixture gpos must not be presented as real GFR replay. Preserve DA bytes and signed/pending task state across crashes, keep missing inputs at WAIT, and require actual successful finalized Inbox receipts for acceptance evidence. Real 07/08 G, 12/13 ResumeCut, and 02 controlled service integration remain separate gates.
