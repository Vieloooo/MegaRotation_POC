# Version 4: regular local rotation discussion

Created 29 September 2026 from the then-current working copy of `main_v3.tex`, including its tentative quota-triggered BLAME condition. This is a discussion extension, not an implemented or proved replacement protocol.

## Reading order

- [Version 4 source](../../main_v4.tex) and [PDF](../../main_v4.pdf).
- [New Section 9](../../sections/v4-rotation-discussion.tex): production comparisons, Candidate A, optional A+, Candidate B, costs and proof obligations.
- [Chinese reading note](discussion-zh.md).
- [Protected baseline hashes](baseline-sha256.json) and [source-code manifest](sources/code-manifest.json).
- [Validation](validation.md).

## Preservation and scope

The source snapshot is [baseline/main_v3.tex](baseline/main_v3.tex). The original `main_v3.tex`, `main_v3.pdf`, and all nine active v3 figure inputs are protected by before/after hashes. Version 4 uses separate copies of those figure inputs in `figures/v4/base/`; future edits to v3 figures therefore do not silently change v4.

The inherited specification, result statements and proofs are preserved. Changes outside the appended discussion are version/date metadata, figure-input paths, two explicit scope notes, and new bibliography entries. The notes explain that the tentative quota-BLAME branch invalidates the original progress proof's “honest leaders are never blamed” step; they do not silently remove the user's experiment or claim to have repaired it.

Candidate A uses fixed application segments and waits for terminal finality before ordinary initialization. Candidate A+ sketches a quorum-backed opening certificate. Candidate B replaces the regional engine with one supporting a pipeline across leaders. No new safety/liveness theorem is asserted for these candidates.

## Reproducible implementation evidence

The public code files were retrieved and retained under `sources/` for narrow read-only comparison:

| Project | Explicit branch | Commit | Commit timestamp (UTC) | Files checked |
|---|---|---|---|---|
| Flow | `master` | `4315a94dce27b0a5da53103bd1f1f519ba4506ca` | 2026-09-25 16:38:28 | HotStuff README, pacemaker, safety rules |
| Aptos | `main` | `a19dae54b5657cdf7abbd9907f465553cdff07f4` | 2026-09-29 05:04:24 | round state, round manager, consensus README |

The code manifest contains original URLs and SHA-256 hashes. These repository snapshots are not claims about deployed mainnet commit IDs or feature flags. An initial GitHub API query using the literal ref `HEAD` returned an older Aptos commit; the comparison instead uses the explicitly resolved `main` commit above.

### Claim-to-source map

- **Flow QC versus TC entry:** [`ProcessQC` / `ProcessTC`](https://github.com/onflow/flow-go/blob/4315a94dce27b0a5da53103bd1f1f519ba4506ca/consensus/hotstuff/pacemaker/pacemaker.go#L93-L149); [entry-evidence checks](https://github.com/onflow/flow-go/blob/4315a94dce27b0a5da53103bd1f1f519ba4506ca/consensus/hotstuff/safetyrules/safety_rules.go#L339-L374).
- **Flow vote routing and finalization:** [consensus README](https://github.com/onflow/flow-go/blob/4315a94dce27b0a5da53103bd1f1f519ba4506ca/consensus/hotstuff/README.md#L138), especially the `forks` description at line 246; [official Jolteon article](https://flow.com/engineering-blogs/jolteon-advancing-flows-consensus-algorithm).
- **Aptos standard round transition:** [`process_certificates`](https://github.com/aptos-labs/aptos-core/blob/a19dae54b5657cdf7abbd9907f465553cdff07f4/consensus/src/liveness/round_state.rs#L245-L288).
- **Aptos vote routing and optimistic proposal materialization:** [routing including broadcast configuration](https://github.com/aptos-labs/aptos-core/blob/a19dae54b5657cdf7abbd9907f465553cdff07f4/consensus/src/round_manager.rs#L1551-L1573), [QC round and parent-identity checks](https://github.com/aptos-labs/aptos-core/blob/a19dae54b5657cdf7abbd9907f465553cdff07f4/consensus/src/round_manager.rs#L965-L986).
- **Aptos deployment context:** [Baby Raptr announcement, 20 June 2025](https://aptosnetwork.com/currents/baby-raptr-lands-on-mainnet). This establishes a dated production upgrade, not deployment of every later Raptr design.
- **Monad normal path and finality distinctions:** [official MonadBFT documentation](https://docs.monad.xyz/monad-arch/consensus/monad-bft), accessed 29 September 2026.
- **Monad backup collection and recovery:** [Category Labs update, 21 November 2025](https://www.category.xyz/blogs/monadbft-update-fast-recovery-leader-fault-isolation), with [MonadBFT paper v4, 21 July 2026](https://arxiv.org/abs/2502.20692v4). The original no-endorsement/reproposal path remains relevant; an ordinary tail that was not final is not a finalized-prefix safety violation.
- **Protocol-level references:** [Jolteon and Ditto v3](https://arxiv.org/abs/2106.10362v3); [Optimal Good-Case Latency for Rotating Leader Synchronous BFT](https://doi.org/10.4230/LIPIcs.OPODIS.2021.27). The latter's count-based model is not silently substituted for the draft's weighted assumptions.

## Build

From `formalization/`:

```sh
latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/main-v4-rotation main_v4.tex
```

The delivered PDF is copied from this isolated build. New prose and diagrams are proposals; compilation and layout checks do not certify their mathematical correctness.
