# Independent presentation and scope review — v1

Reviewed on 25 September 2026. This review is read-only with respect to `main.tex`. Scope: reader pipeline, notation, claim boundaries, traceability of goals, and preliminary PDF presentation; it is not a replacement for the independent proof review.

Inputs read: the complete `main.tex`, preserved `outline.md`, complete `reviews/ethereum-chat-context.md`, and the design/source reviews. The user's explicit order-only S / mandatory executing P clarification controls the role interpretation. Source line numbers below refer to the concurrent draft read during this review, approximately the 12:09 build; section and quoted-text anchors are supplied because the author is making changes.

## Overall assessment

The reader pipeline is coherent: source choices → model and input/execution distinction → acceptance levels and goals → local protocol → global control/FFG → geographic handoff → P/DA/L1 → economic interfaces → conditional arguments and traces. The main departures from the outline are visible early and repeated where they affect guarantees. In particular, the text does **not** conceal that weighted PBFT replaces regional Gasper, global PBFT makes FFG redundant for order safety, the A2-only tail can be abandoned, and the native execution/signing/L1 adapters remain uninstantiated.

The order-only S boundary is consistently maintained. Regional/global signatures certify order and availability; P executes, while optional private S execution has no consensus obligation. The draft also separates input-log progress, executable progress, and publication. These are substantive strengths and should be preserved through compression.

The findings below concern exact promise wording and transition semantics, not a request for another wholesale reorganization.

## Substantive scope and readability findings

### R1 — Align G4's bounded-time promise with the conditional result

- **Locations:** G4, source approximately 299–302; Proposition “Conditional liveness,” 836–845; “Latency accounting,” 857–873.
- **Issue:** G4 promises that valid batches become A2 and retrievable by P “in bounded time.” The result establishes finitely many view changes assuming the schedule eventually reaches an honest leader; the displayed bound covers one failed leader followed by an honest successor. The text does not give a uniform bound on the number/duration of failed attempts or P retrieval in the general G4 case. Finite eventual recovery and an explicit service bound are distinct reader promises.
- **Suggested repair:** State G4 as eventual progress under the proposition's assumptions, with bounded service only when the schedule, view changes, retrieval, and message-resource terms have configured finite bounds. Alternatively, add the full bounded-attempt hypothesis and formula. Do not broaden the proof by changing only its conclusion.
- **Classification:** Substantive guarantee alignment, moderate priority.

### R2 — Give the activation epoch an operational meaning

- **Locations:** §6.1 activation package, approximately 584–600, `e_act` versus “Activation takes effect ... when ROTATE is globally decided.”
- **Issue:** The message contains an activation epoch, but the transition fires on the BFT decision itself. A reader cannot tell whether `e_act` is merely the ROTATE record's index, a future eligibility boundary, or an additional guard. The generation field `a` in a ROTATE control record also needs a one-line convention if it denotes the departing generation.
- **Suggested repair:** Define `e_act` as the epoch/index of the deciding ROTATE record and remove any suggestion of an additional delay; or specify the delayed activation predicate and authority in the intervening period. The immediate-decision interpretation is consistent with the present prose.
- **Classification:** Substantive transition ambiguity, moderate priority.

### R3 — Mark configuration transfer as a proposed extension of the record grammar

- **Locations:** §5.1 action set, approximately 484; §6.3 “Configuration boundary,” 645–655.
- **Issue:** The fixed-configuration grammar lists only ADVANCE, HOLD, and ROTATE, while §6.3 says the old global committee decides RECONFIG as though it were already in that grammar. The final caveat correctly excludes dynamic membership from the proof, but the opening description sounds more complete than the stated transition system.
- **Suggested repair:** Begin §6.3 with “A future configuration-transfer extension would add a terminal RECONFIG action ...” or explicitly extend the grammar and identify which rules/proofs remain open. The former preserves the deliberately fixed-configuration scope with less text.
- **Classification:** Substantive scope clarity, moderate priority.

### R4 — Express the rollback boundary as a suffix, not “below A3”

- **Locations:** §6.1 approximately 579; after the durability proposition approximately 631.
- **Issue:** “Rollback below A3” means a lower acceptance level to the author, but can mean reverting below the last A3 position to a reader. That latter meaning would contradict the main guarantee. The precise rule is already clear in §6.2: discard only positions after the latest A3 anchor.
- **Suggested repair:** Use “discard only the regional-only suffix beyond the latest A3 anchor” in both places. This is a wording fix with a safety-relevant interpretation.
- **Classification:** Substantive wording ambiguity, moderate priority.

### R5 — Separate unchanged order frontier from newly achieved finality

- **Location:** §5.2, approximately 548–555, “without finalizing any new transactions.”
- **Issue:** Equal input tips establish that the control links add no ordered inputs. A HOLD link can nevertheless newly finalize the previously A3-only prefix: the first adjacent link finalizes its predecessor. At the second link the same prefix is already final, but the present wording invites the broader, incorrect inference that HOLD processing cannot promote input from A3 to A4.
- **Suggested repair:** Say “without advancing the ordered-input frontier.” Add an explicit sentence that a finalized checkpoint makes its committed input prefix and control ancestors final, if that is the intended A4 definition. This also prevents readers from importing the wrong “all blocks in an epoch” interpretation from Ethereum discussions.
- **Classification:** Substantive acceptance-level clarity, moderate priority.

## Notation and navigation findings

### R6 — Assumptions render as theorems in cross-references

- **Locations:** rendered page 11, Lemma 9.1 and Propositions 9.2–9.3; source labels `ass:fault` and `ass:external`.
- **Observed defect:** The PDF says “under theorem 2.1” and “under theorems 2.1 and 2.3,” although the targets are Assumptions 2.1 and 2.3. This obscures which statements are hypotheses and which are proved.
- **Suggested repair:** Correct the cleveref label types for the shared-counter environments, then re-render to check the actual words. Check definition/proposition/lemma references at the same time rather than changing the global theorem name indiscriminately.
- **Classification:** Reader navigation and epistemic labeling, moderate priority; mechanically small.

### R7 — Normalize availability-certificate subscripts

- **Locations:** §5.1 approximately 471 and 480–492.
- **Issue:** The object is introduced as `AC_k` by input position and inserted as `AC_e` by epoch without defining the conversion. The range/base fields are now explicit, so this is a notation issue rather than an availability-proof finding.
- **Suggested repair:** Define `AC_e := AC(k_{e-1}, k_e, g_{e-1})` for ADVANCE or use one notation consistently; state the retained/empty certificate convention for HOLD and ROTATE.
- **Classification:** Notation clarity, low priority.

### R8 — Make the goal-to-result mapping explicit

This is optional but would make the requested formalization easier to audit. The actual mapping is:

| Goal | Support in the current draft | Exact boundary |
| --- | --- | --- |
| G1 order safety | Proposition “Order and handoff safety” | Fixed configuration; correct underlying BFT; regional A2 within one generation; global A3 across generations. |
| G2 authority/fencing | Same proposition plus §6.1–6.2 validity rules | Fences A3 acceptance; does not instantly inform an isolated A2 consumer. |
| G3 faithfulness | Proposition “Data and faithful execution” plus §7 | Settlement acceptance is sound under the execution/L1 adapter; A2–A4 certificates alone do not certify execution. |
| G4 weak service liveness | Proposition “Conditional liveness” and latency paragraph | Eventual honest leader, sufficient responsive weight, valid available operation, and convergence/readiness where required; see R1 for bounded-time mismatch. |
| G5 recovery | Safety proposition, data proposition, §6 and latency paragraph | Local A2 preservation, global A3 preservation, conditional retrieval and successor readiness; A2-only suffix rollback permitted. |
| G6 accountability interface | §8's domains, evidence predicates, public job records, and abstract mappings | An interface specification, not a proven incentive/equilibrium or complete slashing mechanism. |

A short paragraph at the beginning/end of §9 can cite this correspondence without adding another large table. In G3, “A5-accepted execution” is more precise than the unqualified “accepted execution.”

## Preliminary rendering inspection

- Inspected all 13 pages of `build/main.pdf` produced at approximately 12:09 on 25 September 2026, by rendering with Poppler at 85 DPI and reading the page images.
- No clipped text, overlapping equations, broken table rules, or stray missing glyphs were visible. The current log has no overfull boxes; remaining underfull warnings are bibliography URL wrapping.
- Tables, headings, margins, and page numbering are consistent. The worked traces remain easy to follow.
- The three diagrams are still text placeholders. Final figure readability, their A2/A3/A4 boundaries, caption placement, and any pagination changes remain for the final rereview.
- The assumption/theorem cross-reference problem in R6 was confirmed visually, not inferred solely from source.
- Cosmetic choices such as a contents page, different table spacing, or avoiding one paragraph split across a page are not blockers and are not recommended as additional scope.

## Final-review checklist

Recheck R1–R7 against the actual revised text; verify the goal mapping above still holds; inspect final diagrams and all pages affected by their insertion; confirm that the diagram arrows do not imply global consensus or execution on the A2 critical path; confirm final source references distinguish assumptions from proved results.

Memory was used only for workspace orientation. Current role, native-interface, and claim-boundary judgments above were taken from the current task confirmation and current files, not stale memory.
