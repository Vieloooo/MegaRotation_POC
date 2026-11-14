# Version 3 manuscript figures

The manuscript uses nine native inputs. Section 3.4 adds
`terminal-drain-pipeline.tex`, a three-batch example showing quota enforcement,
timer-free NEWVIEW re-certification, a fresh DRAIN child's commit timer, and
terminal receipt export and retirement. In addition to the four files described
below, `regional-pipeline.tex`, `regional-communication-pipeline.tex`,
`blame-communication-pipeline.tex`, and `retrieval.tex` were copied from the
latest v2 figure set without modifying their originals. The BLAME and CONFIRM
thresholds in that source are greater than one third; vote QCs remain greater
than two thirds. The scheduled-status revision updates the architecture, regular/forced
rotation, derivation and terminal-reference labels. Current validation is
recorded in [the revision note](../../reviews/main-v3-scheduled-status-2026-09-29.md);
the [initial validation](../../reviews/main-v3-2026-09-28/validation.md) is historical.

All four files are native TikZ inputs, use only the TikZ libraries already loaded by `main_v2.tex`, and define their styles locally. Widths are approximately 15.2–15.6 cm, suitable for the one-column 11 pt article without `resizebox`. Their internal text uses `footnotesize` (9 pt) and remains actual selectable PDF text.

Suggested captions:

- **architecture.tex:** “The cross-region live-stream service wraps regional SMR interfaces. Deterministic scheduling and authenticated status entries determine regional authority; the committee neither runs the normal sequencing path nor certifies each stream tip. Proposer P verifies the resulting stream before downstream publication.”
- **regular-rotation.tex:** “Regular cross-region rotation after a fixed quota of application batches. Time flows down; spacing indicates causal order, not elapsed time. The successor starts ordering only after it verifies the old region’s terminal finalized batch and the effective map at its recorded L1 control height. A normal handoff needs no new committee entry.”
- **forced-rotation.tex:** “Forced cross-region rotation when a scheduled update becomes effective and marks the then-active region unavailable. Time flows down. The entry supplies a future L1 activation height and status map; a separate recovery witness fixes the application cut and regional evidence resolves its anchor. Safe-cutoff and old-authority-closure conditions are required separately before successor production. An inactive region’s recovery only changes eligibility.”
- **derivation.tex:** “Verification obligations for proposer publication. Status history and regional evidence determine the authorized ordered stream; DA and execution remain separate obligations, all bound to the same complete input. The relation can be enforced directly or by a selected fraud-proof or validity-proof system.”

Notation used in figures: `K` is the application-batch quota, `b` a global application-batch height, `A_b` a finalized anchor, `F_b` a regional final receipt, `L_h`/`L_b` a signed L1-height timestamp, `rho_b` the batch L1 reference, `E_j=(a_j,chi_j)` a scheduled status entry, and `beta_*` the separate application recovery cut. No extra consensus layer or speculative/conditional-session stage is shown.

Presentation advice for inherited v2 figures: the five-lane regional communication plot is much wider than the manuscript column and uses scriptsize text. Prefer a full-width native redesign or split pipeline overview from detailed all-to-all communication if the original requires shrinking. The v3 figures deliberately stay within four lanes and place caveats in the caption or a compact note rather than long arrow labels.
