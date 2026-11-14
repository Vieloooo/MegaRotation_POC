# main_v3 validation and review record

Date: 2026-09-28.

## Delivered artifacts

- [main_v3.tex](../../main_v3.tex)
- [main_v3.pdf](../../main_v3.pdf), 25 pages
- [Native TikZ inputs](../../figures/v3/README.md), eight manuscript figures
- [Machine-readable validation results](validation-data.json)

## Scope and verified preservation

The starting main_v2 snapshot is [source/main_v2.tex](source/main_v2.tex).
Its SHA-256 is `24fcf93848b06cce8df91a321200b4291b3b6b9818938b723254f10d283f1eb6`.
The live main_v2 file was checked against this snapshot and was not modified by this task.

The entire `Regional live stream` definition is byte-identical: 658 bytes,
SHA-256 `fbd6cd16242a26303d653fcae9ab024cea85d69d8233c91b2c46bcdc27b61770`.
Section 2 adds a cross-region service that composes the regional interfaces.
The original VOTE >2W_i/3, BLAME >W_i/3, and CONFIRM >W_i/3 distinctions are preserved.

The draft adds a concrete empty-projection DRAIN adapter, ordered status entries,
regular/forced handoffs, conditional composition results and the proof-system-neutral rollup relation.
Inactive-region recovery is passive; only an update making the active region unavailable triggers a forced obligation.
The trigger and cutoff remain latched until activation, even if later eligibility updates arrive.

“Timeout cross-region rotation” is interpreted as the previously specified fixed-quota term expiry.
No separate wall-clock authority-transfer rule is introduced; a fetch timeout alone is not authority.

## Independent reviews

- [Control design audit](control-design.md)
- [Control review v1](control-review-v1.md)
- [Final bounded control review](control-review-final.md): pass under declared premises
- [Complexity assumptions](complexity-assumptions.md)
- [Complexity review v1](complexity-review-v1.md)
- [Final bounded complexity review](complexity-review-final.md): pass under stated assumptions
- [Presentation review](presentation-review-v1.md): full-PDF inspection and corrected figure layouts

Resolved control findings include persistent forced obligations, explicit clearing at activation,
candidate-certificate uniqueness, and the control-order premise in the normal-handoff lemma.
Terminal receipt verification ends new physical DRAIN traffic while retaining data serving.
Resolved accounting findings include successor waiting, receipt collection, bootstrap,
uncached registry acquisition, failed attempts and control-block traffic.
The four-case timing table and separate message/bit table use consistent measurement points.

## Build and visual validation

The manuscript was compiled using:

```sh
latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/main-v3 main_v3.tex
```

The final log contains no LaTeX warnings, undefined references, overfull boxes or underfull boxes.
All input paths exist, labels are unique, and the preserved definition was compared programmatically.
All 25 pages were reviewed through rendered page/contact-sheet inspection, with protocol,
figure and table pages inspected at full size.
The final small layout changes were recompiled and pages 2, 16 and 19 were checked again.
All eight figures are native TikZ; original v2 figures remain untouched.

## Limits of the validation

These are human-readable design/derivation audits and build/render checks, not machine-checked
protocol verification or implementation tests.
Forced recovery remains conditional on an admissible complete cut, fenced old authority,
recoverable data and compatible control ordering (Assumption 4.2).
The draft does not claim that an entry signature or an arbitrary proof system establishes
absence of a hidden finalized suffix.
Committee coordination and concrete fraud-proof/validity-proof implementations remain interfaces;
their costs are parameterized rather than invented.

Final source SHA-256: `3b6d7add459896616f381de1a8bf98effa1d83424bd79af9efdcf685894bbed4`.
Final PDF SHA-256: `99c07481bdbb4e41b7b56603054891f5090e24dd59106e9e2d8129aa17b2712f`.
