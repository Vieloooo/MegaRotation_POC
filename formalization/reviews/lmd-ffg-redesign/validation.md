# Final validation

Date: 26 September 2026 (Asia/Shanghai).

## Artifacts

- Main source: `formalization/main.tex`
  - SHA-256: `325ab6a05c0fd903f0f999a1d474218d122a302a54aa9c65a744a5dc989a4223`
- Delivered PDF: `formalization/main.pdf`, 20 A4 pages, 522,505 bytes
  - SHA-256: `abe56efb43ad3d6adb5515d174f427b6bd10e80af9cd9d0e09e18044c16a3dc9`
- Isolated build: `formalization/build/lmd-ffg-final/main.pdf`
  - Byte-identical to the delivered PDF.
- Independently reviewed mathematical text:
  `formalization/build/lmd-redesign-baseline/final-main.tex`
  - SHA-256: `a1697f0842252a3af2d2a60cbb268ee73b3d00d8034629b8059762a8c635615b`
  - The sole subsequent source change is `\Needspace{25\baselineskip}` before
    the consumer subsection. This is layout only.

## Proof and model checks

- Three freshly initialized proof reviewers were used across two independent
  draft-review rounds, followed by targeted repair checks.
- The [final closure addendum](cycle3-closure.md#final-artifact-closure-addendum)
  reports no remaining findings under the manuscript's stated assumptions.
- First-cycle cutoff, request-timeout and consumer-anchor issues were repaired.
- Second-cycle equivocation-body and candidate-verification accounting issues
  were repaired. Relative body bytes and complete envelope bits are explicitly
  distinguished in the final equation.
- `python3 reviews/lmd-ffg-redesign/model_checks.py` passes:
  - 1,296 conflicting report vectors, each checked in 24 origin orders;
  - two whole-batch packing budgets for each vector;
  - 12 latest-message, clock, cutoff, re-anchor and equivocation checks.
- These are written mathematical reviews and finite abstract checks, not a
  machine-checked distributed-system proof or a deployment certification.

## Build and presentation

Executed from `formalization/`:

```sh
latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/lmd-ffg-final main.tex
```

- Build succeeded from an isolated output directory.
- Final log contains no LaTeX warnings, undefined references/citations,
  overfull boxes or underfull boxes.
- Source-level cross-reference check found no missing label targets.
- Both active TikZ figures compile as part of the manuscript. Pipeline labels
  now distinguish regional HEAD/SetPreview from global report/FFG/Deliver.
- All pages were rendered and visually inspected using page contact sheets;
  architecture, protocol and pipeline pages were additionally inspected at
  larger size. A small diagram width overflow and orphan subsection heading
  were corrected. No clipped text or overlapping diagram elements remained.
- `git diff --check` passes.

## Scope and preservation

The design retains costly **epoch-level** report agreement, removes only
regional per-batch agreement, and openly distinguishes itself from deployed
Ethereum Gasper. Global finality and liveness claims require the explicit
quorum, complete-object delivery, admission, retention and tenure assumptions.
No individual speculative mini-block finality or survival claim is made.

The user's starting main source and active figure sources are preserved under
`build/lmd-redesign-baseline/`; any prior delivered `main.pdf` was preserved
there before replacement. Existing unrelated workspace changes, historical
figures, outlines, meeting files and reviews were not reset. No commit or push
was performed.
