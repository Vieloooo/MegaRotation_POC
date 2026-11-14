# Final validation of the standalone service rewrite

Date: 2026-09-25.

## Delivered artifact

- `main.tex`: complete proposed service protocol, with 10 protocol boxes, two native TikZ figures, three worked examples, the model/service contract, self-contained correctness arguments and case-by-case complexity analysis.
- `main.pdf`: 21 A4 single-column pages. It matches the default `build/main.pdf` byte for byte.
- The service boundary excludes consumer execution, EVM block assembly, DA/L1 publication and economic penalties.
- Original user outline retained; previous manuscript and PDF preserved under `build/pre-rewrite-20260925/`.

## Verification

The independent correctness audit repaired its original findings (exact batch PROPOSE authentication; mandatory honest READY behavior; the local/global responsive-quorum relationship). The final delta audit in `rewrite-correctness-v3.md` found no remaining Critical Error or Justification Gap under the stated model. The independent complexity audit in `rewrite-complexity-v3.md` closed its findings about propagation delay, intermittent faults, monotone receiver authority, payload duplication, report packing and multi-chunk state transfer.

Both reviews cover frozen `build/rewrite/review-v3.tex`. Subsequent main-source changes are presentation-only: compact introduction, figure/table/algorithm placement, PDF metadata, bibliography spacing, and explicit theorem/lemma reference labels. Protocol guards, mathematical statements and proof arguments have not changed since that review snapshot.

The result is an independently audited proposed protocol under its explicit assumptions, not a machine-checked proof or a deployed implementation. In particular, synchrony, fixed membership, responsive weight, retained data, healthy-successor availability and bounded workload remain premises.

## Build and visual checks

- Default build: `latexmk -pdf -interaction=nonstopmode -halt-on-error main.tex` succeeded.
- No undefined references/citations, LaTeX warnings, overfull or underfull boxes in the final log.
- Final 21-page PDF rendered and all pages inspected; architecture, pipeline and the complete authenticated-broadcast box also inspected at full-page resolution.
- Algorithm boxes remain intact across pages; theorem/lemma references use the correct names.
- Only native TikZ figure inputs are used by the manuscript.

## Explicit design decisions

- Regional Preview and globally finalized Deliver carry different persistence promises.
- The two-delay regional fast decision does not remove mandatory completion cost.
- Global synchronous report agreement is included as a concrete convergence mechanism before FFG voting, with its communication cost charged.
- Leader push and content-addressed peer pull use the same verification predicates; raced and repeated payload traffic is accounted for.
- A whole-region outage preserves the globally adopted prefix; an unretained regional preview may be cancelled.

## SHA-256

- `main.tex`: `1fc6cbfa619412b506d0d4f6d9d8b911e89609c8c03cdf1f858e9db148ed549e`
- `main.pdf`: `4a77f6dcf0b5e83dde52289d2acb93b9e2b06c36eb58ffc3b3a6081c8d45f9c0`
- `build/main.pdf`: `4a77f6dcf0b5e83dde52289d2acb93b9e2b06c36eb58ffc3b3a6081c8d45f9c0`
- `figures/service-architecture.tex`: `2b6d0654254dd63e5f729f6487f9a2f04bc221945003802eea1b9370bf20dfe8`
- `figures/stream-pipeline.tex`: `29f8c1919b02a4652b2e12e6afa8d3354e457db04b9f7c073b788309c54012b8`
- `build/rewrite/review-v3.tex`: `09f280cdf9caedda6e642c4f8641a4365fd6eec64bb24ab9e432b7a8da19a2d5`
