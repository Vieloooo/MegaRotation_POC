# main_v2: equivocation slashing and blame communication

Date: 2026-09-28.

## Authorized scope

This update implements only items 1 and 2 of the user's request in [main_v2.tex](../main_v2.tex): explicit leader double-head slashing and a regional blame communication pipeline.
The committee, cross-region rotation, fixed batch quotas and terminal-drain proposals remain in the separate design documents.

## Changes

- Section 3.4 now defines double-head evidence using the same chain/deployment, configuration, region, session, original producing view and mini-block height, two valid original signatures, and distinct canonical header digests.
- The L1 procedure checks the current chain/adjudicator and authorized historical records, verifies evidence, deduplicates by production slot, and applies the configured penalty without changing the current session's quorum weights.
- Existing VOTE, PROPOSE/NEWVIEW and confirmed-prefix evidence is made explicit; lawful view changes, unchanged-anchor re-certification and forwarding are excluded from false attribution.
- Figure 4 now uses [native TikZ communication lanes](../figures/v2-major/blame-communication-pipeline.tex), showing BLAME, quit-proof relay, the separate waits, STATUS, NEWVIEW, anchor re-certification and fresh-child resumption.

## Independent review

A separate reviewer checked the slashing predicates and their compatibility with V1--V4.
Its requested fix, explicit chain/deployment checks and resolution through authorized L1 configuration records, was incorporated.
A separate agent prepared and refined the diagram; the integrated PDF was subsequently inspected by the main agent.
This is a scoped manuscript and consistency review, not an implementation audit or a new proof certification.

## Validation

- Fresh build: `latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/main-v2-slashing-blame-20260928 main_v2.tex`.
- Result: 11-page PDF; no LaTeX warnings, overfull/underfull boxes or unresolved references in the final log.
- Rendered all pages; inspected the overall layout and the changed figure/evidence pages in detail. Final figure, equations and protocol box have no clipping or text overlap.
- Source comparison confirms the earlier consensus algorithms and Sections 4 onward are unchanged.
- Updated [main_v2.pdf](../main_v2.pdf) from the verified build.
- No runtime tests were applicable to this document-only change.

## Reproducibility

- Baseline source SHA-256: `e61c6e5083979464cef09f86d101a62940d040bd8735673fcad03a705deeb919`.
- Updated source SHA-256: `d201da53bceb0df51f8be46784f9f7b576f8ce1f8bb512526c15e23d30f08e35`.
- Updated figure SHA-256: `80515b3ecc23680650fb3683c1242449c7fdab55999f4364b21771ceea0b618f`.
- Delivered PDF SHA-256: `c85ddb0e28d5d027b0fc0c3264123cd71ca6e3a6f5add9bd34cc82e2d314e130`.
- [Scoped source diff](main-v2-slashing-blame-2026-09-28.diff).
- Original source and PDF preserved under `formalization/build/main-v2-slashing-blame-20260928/baseline/`.
