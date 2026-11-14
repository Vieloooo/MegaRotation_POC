# Part II v4 validation

Date: 2026-10-19.

## Scope

Created the English formal TeX/PDF and the corresponding Chinese Markdown from v3.7 under the final hard-force-only v4 revision plan. This is author proofreading and build verification, not an independent whole-protocol security audit or a model-checking result.

## Completed checks

- Preserved ten top-level sections and six numbered protocols.
- Compared Protocols 1 and 2 and the DATA/quota/DRAIN definition section byte-for-byte with v3.7: unchanged.
- Checked all 14 files in the active TeX input closure for missing references, duplicate labels and obsolete AUTO/Owed/CLEAR runtime symbols: none.
- Read the source changes for explicit MSIG-only activation, independent ON/OFF, old-beta matching, strict pre-payload cutoff, exact retained G, no output/Ready/gap prerequisite, and correct unassigned-boundary handling.
- Aligned Chinese examples, state lifecycle, Ready scope and governance liveness boundaries with the English manuscript.
- Checked the Markdown links, equation/fence delimiters and source whitespace.
- Compiled with latexmk/pdflatex in an isolated temporary output directory: 59 pages; no LaTeX warnings, unresolved references/citations, multiply defined labels, or overfull/underfull boxes in the final log.
- Rendered all 59 pages and inspected seven contact sheets; inspected the new architecture and Force figures in detail. Fixed the architecture label overlap, rebuilt, and inspected the changed final pages.
- Verified SHA-256 preservation of all 15 baseline source dependencies/readers, including v3.7 and the existing arch_v4.tex.

## Model and claim boundaries

- Ready has a separate bounded-service assumption; it does not require a block between Forces.
- No automatic Force fallback remains. Offline recovery requires governance response and a usable candidate.
- Continuous Hard Force may prevent preparation or progress. The global-prefix safety argument does not claim that such an execution remains live.
- Existing regional, retirement, FINAL and application/state verification interfaces remain conditional on their stated assumptions; they were not independently re-proved from scratch in this revision.
- Cross-region fairness, eligibility restoration and client-request inclusion are not protocol guarantees.

## Deliverable hashes

- sync_hotstuff_rr_part2_rotation_v4.tex: 27d6aa71738be38b7a34a4386c2c0925bdc73f437f5a49ffddbac6dbf40b266c
- sync_hotstuff_rr_part2_rotation_v4_zh.md: 79a4d14c2f93c473a42d78695f9986381b96ebf49e5b7056a6294317ded3d928
- sync_hotstuff_rr_part2_rotation_v4.pdf: f8d0974363ec115a706589de19659198a5fecde7f1cb2b75cb1196ef9ef11ef9
