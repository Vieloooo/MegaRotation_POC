# Final validation

## Exact reviewed outputs

- Manuscript: `formalization/main_PBFT.tex`
- SHA-256: `42ae5d3fc3b94b84f5a12d71a910909ec045e07363129ce8b4aa43e399f41eeb`
- Delivered PDF: `formalization/main_PBFT.pdf` (24 pages)
- PDF SHA-256: `c6437899754a29b5220d72be6cd0ed8170100180beba415ad8236ac67b352ce3`
- New figure inputs: `service-architecture-pbft.tex` and `stream-pipeline-pbft.tex`.

Both independent final reviews identify the exact final source hash above. The final source differs from the last substantively reviewed source only by two verified page-placement instructions; reviewers checked and recorded those changes.

## Build and layout

- `latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/pbft main_PBFT.tex` completed successfully.
- No undefined references/citations, LaTeX warnings, overfull boxes, or underfull boxes in the final log.
- All 24 pages rendered and visually inspected; figures, phase algorithms and cost table also inspected at larger scale. The two subsection headings previously separated from their protocol boxes were moved with the boxes and rechecked. No clipping or overlapping text found.
- Source reference scan: 63 labels, 32 reference uses, no missing/duplicate labels.
- Only the two new PBFT figure inputs are included. No LMD-GHOST/HEAD module, regional completion algorithm, or old local GAP-timer parameters remain.

## Security and self-containedness

- Independent security review: no remaining Critical Error or Justification Gap identified under the explicit fixed-fault, persistence, capacity and delivery assumptions.
- Fresh independent self-containedness review: PASS after resolving all nine initial findings.
- Finite checks: 84 weight profiles, 31,606 quorum/carryover cases and 1,476 bounded-entry-skew timing cases; zero failures. These supplement the mathematical arguments and are not a full implementation test or exhaustive asynchronous execution proof.
- Regional safety is independent of synchrony. Regional progress uses the explicit retry envelope. Global agreement/progress still requires continuous synchronous report rounds and a responsive global supermajority. The final manuscript states the global stalled-epoch limitation and does not claim arbitrary-partition recovery.

## Preservation

No task authoring wrote to the live `main_v1.tex`, `main.tex`, or the original figure sources. Original figure hashes match the starting baseline. The live `main_v1.tex` was edited concurrently outside this task; its final observed hash is `ed64009d7395b9a9ff3ef8db1ba10f2017fe56cc085b7c504c2be6e5093dd4f8`. See `source-observation.md` and `source-observed.diff`. The starting source is preserved in `source/main_v1.tex`, matching the pre-authoring baseline hash.

No commit or push was performed.
