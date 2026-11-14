# main_v2: separate BLAME, vote and confirmation thresholds

Date: 2026-09-28.

## Change

The user authorized lowering BLAME and post-commit receipt thresholds to signed weight greater than W_i/3.
Ordinary vote certificates continue to require weight greater than 2W_i/3.
The active manuscript, both communication diagrams and figure documentation now make this distinction.
CONFIRM statements are explicitly domain-separated from VOTE and are signed only after local commitment.

## Reasoning and scope

- Faulty stake remains strictly below W_i/3 in the analyzed regional session.
- Each accepted BLAME certificate includes an honest signer. The unchanged deadline bound prevents honest members from blaming an honest leader, so Byzantine members cannot remove that leader on their own.
- Quit-proof propagation, canceled old-view timers, the post-quit wait, STATUS and NEWVIEW rules remain unchanged; the threshold change does not select a new history by itself.
- Each accepted receipt includes an honest post-commit endorsement. The existing SMR safety argument supplies compatibility with other honest commitments.
- Honest members can form both types of evidence without Byzantine cooperation; configuration weights stay fixed.
- The proof paragraphs now state these threshold dependencies. No claim is made outside the manuscript's regional synchrony and fault assumptions, and no committee or cross-region protocol has been changed.

## Validation

- Fresh build: `latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/main-v2-thresholds-20260928 main_v2.tex`.
- Result: 11 pages, with no LaTeX warnings, unresolved references or overfull/underfull boxes.
- All pages rendered; overview checked and the two communication figures, receipt definition and revised proofs inspected in detail.
- Static source comparison confirmed the steady-state, slashing and retrieval protocol boxes are unchanged; the view-change box differs only in its BLAME threshold.
- Static comparison confirmed each affected figure differs only in its intended threshold label; the ordinary vote-QC label is unchanged.
- The verified build was copied to [main_v2.pdf](../main_v2.pdf).
- No protocol implementation tests or independent proof certification were performed for this document-only update.

## Reproducibility

- Baseline source SHA-256: `d201da53bceb0df51f8be46784f9f7b576f8ce1f8bb512526c15e23d30f08e35`.
- Updated source SHA-256: `24fcf93848b06cce8df91a321200b4291b3b6b9818938b723254f10d283f1eb6`.
- Delivered PDF SHA-256: `454d814fa55a57742d5aad46bf4708d8bce913f3f8cdeac3b3b6bb2cdf25d135`.
- [Source diff](main-v2-thresholds-2026-09-28.diff).
- Baseline source, PDF and affected figures are preserved in `formalization/build/main-v2-thresholds-20260928/baseline/`.
