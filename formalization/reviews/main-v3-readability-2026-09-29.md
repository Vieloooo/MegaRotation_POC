# main_v3 readability revision

Date: 29 September 2026.

The revision edits [main_v3.tex](../main_v3.tex) and two native TikZ inputs
for presentation and notation. The updated [PDF](../main_v3.pdf) has 28 pages.

## Presentation changes

- Explain the actors, sessions, anchors and three transition types in the
  introduction, followed by a reading guide.
- Introduce cryptographic notation and missing interface arguments before use.
  Define regional membership counts, status versions, committee weight,
  snapshot origins, cursors, sealed boundaries and recovery cuts.
- Add a compact guide distinguishing physical block position, application-batch
  height, mini-block height, local view, cross-region term and application count.
- Explain how batch receipts and mini-block receipts relate, and distinguish
  proposal certification, commitment and exported finality evidence.
- Define the publication interval and execution states before the rollup claim.
- Replace unexplained symbols in the architecture figure with descriptive
  labels; introduce the entry-lock certificate symbols used in the view-change figure.
- Use “forced rotation” consistently and keep handoff figures within their
  respective subsections. Keep key definitions and their explanations together.

## Timing notation

The timing formulas retain their meanings and coefficients. Descriptive time
symbols avoid overloading application batches, anchors, receipts and weights.

| Previous symbol | Revised symbol | Meaning |
| --- | --- | --- |
| $C_i$ | $T_i^{\rm commit}$ | Regional commitment latency |
| $F_i$ | $T_i^{\rm receipt}$ | Transferable receipt latency |
| $X_{ij}$ | $T_{ij}^{\rm xfer}$ | Anchor transfer and verification |
| $A_j$ | $T_j^{\rm init}$ | Successor initialization |
| $D_i$ | $T_i^{\rm drain}$ | Additional terminal drain delay |
| $W_{ij}$ | $T_{ij}^{\rm wait}$ | Waiting for an eligible successor |
| $T_{ij}^{\rm acc}$ | $T_{ij}^{\rm forced}$ | Forced-rotation latency |

The signing domain now uses the already defined configuration and session
identifiers, `cfg` and `s`, instead of introducing the aliases `cfgId` and `sid`.

## Validation and scope

Compared with the source saved before this revision, all 4 definition blocks,
4 assumption blocks, 3 lemma statements, 5 theorem statements, 8 proofs and
9 protocol blocks are byte-identical. The editorial changes preserve the
thresholds, timers, quotas, recovery premises and conditional guarantee scope.
This was a presentation review, not a new correctness audit.

The isolated build completed with no LaTeX warnings, undefined references,
overfull boxes or underfull boxes. All 54 labels are unique, all referenced
labels resolve, and all eight figure inputs exist. All 28 rendered pages
were inspected in contact sheets, with detailed checks of definitions,
protocols, diagrams and performance tables. The final figure-notation edit
was rebuilt and its affected page was inspected again.

Build from `formalization/`:

```sh
latexmk -pdf -interaction=nonstopmode -halt-on-error \
  -outdir=build/readability-20260928 main_v3.tex
```

The pre-edit source and PDF, build log and rendered checks are retained under
the ignored `build/readability-20260928/` directory. Earlier review records
remain historical records of their own source snapshots.
