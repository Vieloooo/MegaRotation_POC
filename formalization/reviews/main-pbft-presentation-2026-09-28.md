# main_PBFT presentation and consistency review — 28 September 2026

Target: [main_PBFT.tex](../main_PBFT.tex). Scope: presentation only, primarily Sections 3 and 4. The current PBFT manuscript is the authority; no protocol changes from `main_v1.tex` were imported.

## Result

The independent comparison found no material semantic drift or inconsistency between the revised exposition and the existing protocol, service definition, proof dependencies, and figures. This was a presentation and manuscript-consistency review, not a new proof certification or implementation verification.

## Presentation changes

- Explain the Preview and Deliver evidence packages at their first use, then describe their components in Section 4.5 using the existing consumer checks.
- Introduce the mini-block, committed regional-result, and global-checkpoint histories before their detailed formulas.
- Explain semantic values, hashes, detached witnesses, fixed type tags, and the verified context implicitly supplied to the existing abbreviated predicates.
- Separate activation and regional predecessor records from the mini-block/batch definitions. Explain `R_0`, `R_u`, and `p_u` before using them in the batch description.
- Add the sequence-7 / blocks-101–103 example, explaining that `R_6` is the committed outcome of sequence 6, not mini-block 6.
- Distinguish original-author view `alpha`, current PBFT retry `v`, and the prepared retry reported in VIEWCHANGE. Explain that selection compares the prepared retry and preserves original authorship.
- Distinguish prepared certificates, semantic committed results, commit witnesses, VIEWCHANGE bundles and the primary's signed NEWVIEW.
- Explain global state fields in a table, the pending package `p` versus regional predecessor digest `p_u`, checkpoint actions, activation timing, effective-state report context, and report-vector ordering.
- Explain the transport message fields as abstract interfaces without inventing a serialization or additional protocol requirement.

## Protected-content checks

Direct comparison against the frozen source confirms byte-for-byte preservation of:

| Protected content | Verified unchanged |
|---|---:|
| Preamble and Sections 1–2 | 12,439 bytes |
| Formal `Authorized, verifiable stream` definition | 1,197 bytes |
| Section 5 through end of file, including all protocols, proofs, bounds and references | 63,800 bytes |
| Original displayed mathematical blocks throughout the manuscript | All 29 retained verbatim |
| Original Section 4 displayed mathematical blocks | All 12 retained verbatim |
| Both active PBFT figure sources | Exact file hashes match |
| Other active manuscript variants `main.tex` and `main_v1.tex` | Exact file hashes match |

In particular, the revision retains the absence of regional GAP records, commit-gated sequence advancement, highest-prepared NEWVIEW selection, immutable original-author signatures, all quorum thresholds, and the original counter/timer/activation rules. Portable certificates acquire no requirement to include prepared certificates or recursive NEWVIEW histories beyond the predicates already specified.

## Build and rendering

- Fresh isolated `latexmk` build succeeds: 27 pages.
- Final log has no LaTeX warnings, undefined references, overfull boxes or underfull boxes.
- All 66 labels are unique and all 47 explicit source references resolve.
- Rendered all final pages for visual inspection and inspected the revised Section 4 pages at higher resolution. Equations, the state-field table and figures are legible without clipping or overlap. Existing protocol-box pagination was preserved with the protected source.
- Updated output: [main_PBFT.pdf](../main_PBFT.pdf).
- Exact presentation diff: [main-pbft-presentation-2026-09-28.diff](main-pbft-presentation-2026-09-28.diff).

## Source identification

Before SHA-256:

`42ae5d3fc3b94b84f5a12d71a910909ec045e07363129ce8b4aa43e399f41eeb`

Reviewed final SHA-256:

`55c4da499645268f72b89259fc6a0e58a8c6a5dd2c5f4bbc0ec60a4211684ff3`
