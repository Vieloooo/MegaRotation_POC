# Validation record

Date: 29 September 2026.

## Scope and preservation

- The v3 source captured at task start has SHA-256 `d2bd13a00d6de27197a3015a30083ead97955dc6f575b228362b2d4312c89a18`.
- Before/after SHA-256 comparison passed for all 12 protected files: `main_v3.tex`, `main_v3.pdf`, the v3 figure README, and all nine active v3 figure inputs. Their pre-existing working-tree changes were not reverted or edited.
- All nine `figures/v4/base/` copies match their v3 inputs byte for byte.
- Normalizing the declared header/date metadata, figure paths, two scope notes and appended discussion input makes v4's entire pre-bibliography inherited body match the frozen v3 body exactly. Existing definitions, protocol rules, result statements and proofs were not rewritten.
- New bibliography entries link official documentation, pinned Flow/Aptos source files and research papers. The tentative quota-BLAME clause remains visible and is explicitly excluded from the inherited progress proof's scope.

## Build and layout

- `latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/main-v4-rotation main_v4.tex` completed successfully.
- Final PDF: **37 A4 pages**, with **10 native TikZ figures** (nine inherited and one new).
- Section 9 begins on **page 31**. The implementation comparison is on pages 31–32; Candidate A on pages 32–33; A+ on pages 33–34; Candidate B on pages 34–35; costs on page 35; proof obligations on pages 35–36; references on pages 36–37.
- Final log has no errors, undefined references/citations, LaTeX warnings, overfull boxes or underfull boxes.
- All 37 pages were rendered with Poppler and inspected as page contact sheets. Pages 31–37 were additionally inspected individually for the new text, table, algorithm sketch, formulas, diagram and bibliography.
- Automated text geometry checks found no words outside a 25-point side/20-point top-bottom page envelope and no unresolved `??` markers.
- Rendered QA pages and the geometry result are under the ignored `build/main-v4-rotation/qa/` directory. The delivered `main_v4.pdf` is copied from the inspected isolated build.

## Research checks and limits

- Flow source inspection covered the QC/TC pacemaker paths, evidence for view entry, next-view vote routing and the documented finalization condition.
- Aptos source inspection covered the standard `QCReady`/`Timeout` distinction, optional vote broadcasting versus next-proposer routing, and the parent-QC checks in optimistic proposal materialization. The discussion does not assert that these functions exhaust every current configuration.
- Monad comparisons use its official documentation, dated recovery update and versioned paper; no independent audit of Monad implementation correctness was performed.
- Repository snapshots do not establish which binaries or feature flags are deployed on each mainnet. The source record explicitly distinguishes code observations, dated deployment announcements and candidate designs.
- This is a design/research and document-validation pass. No candidate A/A+/B implementation, formal proof certification, model-checking run, benchmark or independent-agent review is claimed. Timing for the proposed READY/OPEN fast path is labeled a target, and the weighted leader-schedule issue is an open obligation.
