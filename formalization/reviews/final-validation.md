# Final validation

Date: 2026-09-25.

The delivered design uses local LMD-GHOST, Appendix C.1 / Figure 10's
synchronous 2δ-BB over batched order packets, and cross-region Casper FFG.
The earlier PBFT substitution has been removed. The user's outline was
preserved, and the explicit order-only sequencer clarification is retained.

## Artifacts

- `main.tex`: SHA-256
  `eeefda017320e101cb5be660ce2b8132844daa829b541ba3c36ae8a25188328d`.
- `build/main.pdf`: 17 pages, SHA-256
  `17803ca8cd176a8811fd02e84c366a4cd2f3850fc9260f36667c12d32c125d75`.
- Three vector PDF figures with matching editable draw.io files in `figures/`.

## Checks

- `latexmk -interaction=nonstopmode -halt-on-error main.tex` succeeded.
- Final LaTeX log has no warnings, undefined references, overfull boxes, or
  underfull boxes.
- [Independent correctness recheck](broadcast-correctness-v2.md) found no
  remaining Critical Error or Justification Gap in the stated conditional
  claims. It checks the exact source hash above.
- [Presentation review](broadcast-presentation-v1.md) identified notation and
  export-rule issues that were repaired. The [final visual review](broadcast-presentation-v2.md)
  passed the exact PDF hash above, covering all 17 pages and final diagram labels.

## Guarantee boundary

Local fast confirmation depends on the synchronous instance model and the
validated weighted BA contract; BA completion and external DECIDED delivery
remain required. Global finality safety is distinguished from the explicit
common-target/source conditions needed for progress. Geographic recovery
preserves finalized input, with provisional suffix rollback. Concrete BA,
start coordination, optimized overlap, native adapters, dynamic membership,
and incentive equilibria are not claimed implemented or independently proved.
