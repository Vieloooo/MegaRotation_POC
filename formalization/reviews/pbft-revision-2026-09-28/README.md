# Regional PBFT revision

The manuscript is [main_PBFT.tex](../../main_PBFT.tex), with a compiled [main_PBFT.pdf](../../main_PBFT.pdf). It is based on the starting `main_v1.tex` snapshot; it does not replace `main.tex` or write to the live original. The new figures have separate `-pbft.tex` filenames.

## Changes

- Removed the speculative head-selection module, HEAD messages/evidence, and fork-choice dependencies from the manuscript and its figures.
- Replaced regional signed-broadcast completion with signed, weighted PBFT: PREPREPARE, PREPARE, COMMIT, explicit certificate assembly, and VIEWCHANGE/NEWVIEW.
- Kept one outstanding batched sequence. A timeout retries that sequence; it never invents a GAP or discards a potentially committed value. A new primary carries the highest required prepared batch with its original producer signatures.
- Defined commit outcomes independently of committing retry and certificate signer subset. Defined activation, report, readiness, and view-change semantic records independently of proof encodings.
- Specified retry timing, complete request-response bounds, persistence, data retention, admission budgets, and geographic recovery of an unfinished regional sequence.
- Reworked all regional proofs, local/global composition, latency/capacity conditions, message counts, signature-bit costs, verification work, and diagrams. The global synchronous report agreement and FFG activation structure remain.

## Communication and timing interpretation

| Component | Result |
| --- | --- |
| Normal regional PBFT phase messages | `O(n_i^2)` per successful batch |
| Mandatory per-batch regional authenticated-broadcast completion | Removed; baseline was `O(n_i^3)` direct messages |
| Full unaggregated certificate relay | Still `O(n_i^2)` messages and `O(n_i^3 kappa)` signature bits |
| View change | Additional certificate bundles, potentially `O(n_i^3 kappa)` control bits and `O(n_i^2 B_R)` body bytes |
| Normal internal commit latency | At most `3 delta_R` from an eligible complete proposal |
| Retry duration | Configured `T_V >= 8 Delta_R` under the stated bounded-entry-skew model |
| Global report agreement | Retained, potentially dominant; no system-wide quadratic-cost claim |

The regional construction is a specified weighted, single-outstanding-sequence PBFT adaptation. Its local safety proof does not depend on delay bounds. Its progress proof uses the stated bounded-delay timer envelope; this document does not claim the complete original PBFT partial-synchrony machinery. Global agreement still requires its continuous synchronous complete-object envelope. PBFT does not fix a stalled global epoch after that envelope is violated. Consumer execution, transaction inclusion/fairness, financial penalties, and arbitrary-partition progress remain outside the claims.

## Independent checks and repair record

1. [Initial security review](security-review-initial.md) found a canonical-state specification gap and an overbroad incoming verification-work bound. Both were revised before the [final security review](security-review-final.md).
2. A fresh agent read the entire protocol and figures for notation, logical flow, operational completeness, and standalone exposition. Its [initial review](self-contained-review-initial.md) drove corrections to activation encoding, state transitions, round-trip deadlines, certificate assembly, evidence costs, notation, and figure labels. See the [final self-containedness review](self-contained-review-final.md).
3. [Finite invariant checks](check_invariants.py) enumerate weighted quorum/carryover cases and bounded-skew retry timing cases. [Results](invariant-results.json) are supporting checks, not a full implementation test or exhaustive asynchronous execution proof.
4. [Final validation](validation.md) records source hashes, build/reference/layout checks, and preservation status.

## Original-source observation

The live `main_v1.tex` changed concurrently during this task, outside the revision's writes. It was not reverted. [Source observation](source-observation.md) records that difference. The starting version is preserved at [source/main_v1.tex](source/main_v1.tex) and its hash matches the baseline captured before authoring. The PBFT revision explicitly retains signing persistence, validation, and data retention requirements needed for its proofs.

## Build

From the `formalization` directory:

```sh
latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/pbft main_PBFT.tex
```

The build uses only `main_PBFT.tex` and the two new native TikZ figure inputs, plus standard LaTeX packages. No original manuscript or original figure is a build dependency.
