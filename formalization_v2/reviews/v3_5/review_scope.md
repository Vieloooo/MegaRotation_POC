# v3.5 frozen review input

This is a candidate, not an accepted protocol. The review snapshot is immutable.

- Main: `snapshot/sync_hotstuff_rr_part2_rotation_v3_5.tex`.
- All active TeX input fragments and included figure sources/assets are captured under snapshot/.
- Complete v3.1 baseline and inherited Part I source: snapshot/baseline/.
- Compare the full baseline, including references and progress paragraphs outside Model; do not treat byte equality of Model as a semantic verdict.
- Verify file SHA256 against manifest.json before reviewing.
- Preserve strict pre-payload Force cutoff, original DATA/DRAIN, MSIG priority and old-beta matching when interpreting the candidate. Do not silently repair a missing step.
- Review source-step and ReadyBundle complete-processing/native-state boundaries explicitly.
- Verify regional finite-response constants, actual entry rather than catalog fairness, stable-slot preparation, adaptive miss settlement, finite health interventions, owed-set progress and scope of at-least-one-eligible-region claims.
- Distinguish global DATA progress, repeated accepted regional output, and request inclusion. The candidate explicitly leaves request inclusion open and supplies a proposed counterexample; verify its scope.
- Apply the existing assumption filter before presenting an attack. Do not use permanent missing correct quorum as an in-model execution if it violates inherited regional communication/fault assumptions. Do identify any theorem or objective scope that the model does not cover.
- Reports must not modify the snapshot or infer correctness from prior reviews. Report exact source locations, dependency map, Critical Error vs Justification Gap, and independent branches that remain verified.

The author has not supplied a model-checking or executable protocol certification. Figures and successful compilation are readability checks only.
