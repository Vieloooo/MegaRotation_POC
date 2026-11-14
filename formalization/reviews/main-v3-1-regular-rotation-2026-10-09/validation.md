# Version 3.1 validation

Completed 10 October 2026.

## Document checks

- The final isolated LaTeX build completed successfully: 33 A4 pages.
- The final log contains no errors, undefined citations/references, LaTeX warnings, overfull boxes or underfull boxes.
- All pages were rendered with Poppler and inspected as contact sheets. Protocol/deadline pages and the revised regional-claim pages were also inspected individually.
- Red text is present in the revised steady-state box on page 8, the new normal-rotation box on page 9, the revised BLAME/recovery box on page 11, and the corresponding derivation check on page 24. Text extraction confirms the red color is present; the figures and prose do not accidentally inherit the red color.
- Page geometry checks found no text outside the checked 25-point side and 20-point top/bottom envelope, and no unresolved `??` markers.
- Source checks confirm that the BLAME protocol has no standalone terminal-QC deadline branch. The view-targeted proposal-progress trigger remains bounded by `K_loc`.
- All 13 protected-file SHA-256 hashes match the task-start manifest: v3/v4 sources and PDFs plus the nine active v3 figure inputs. Only the v3.1 copies of two figure annotations were changed.
- A source diff against the frozen v3 input is saved as `main_v3_1-vs-v3.diff`.

## Focused protocol review

The review follows the user's multicast-vote model. No leader is the unique QC collector. A terminal QC is still required to activate the normal successor, but its absence is not independently blamed. On the uninterrupted ready path, a valid proposal triggers forwarding and enough correct multicast votes; conflicts or quit evidence are handled by the original failure mechanism.

The following integration conditions are explicit in the draft:

- Alice's missing terminal proposal targets Alice's view; only after its terminal QC authenticates Bob's normal entry is Bob's first proposal expected.
- A proposal deadline emits a BLAME signature, not an immediate unilateral switch to Caro.
- Normal advancement does not set the old view's quit flag. A delayed valid quit proof still cancels that old view's pending timers, without cancelling an unrelated successor's timers or rolling the current view back.
- Re-certification is not a fresh proposal and cannot be mistaken for completing a new local quota.
- Normal and recovery entry share one opening identifier and voting history. The leader cannot sign incompatible normal/recovery openings in one view.
- The successor's clock distinguishes normal readiness from predecessor-quit recovery, with only one conservative predecessor-quit adjustment and no resets on duplicate evidence.
- Local physical proposal quota and cross-region application quota are separate. DRAIN never adds an application batch.
- Regional-source verification checks the new opening/entry history; the old view-equivocation description includes conflicting normal/recovery openings.

## Proof verdict

**Viable protocol draft, with Justification Gaps remaining; not a completed safety/liveness certification.**

1. **Mixed-entry preservation.** The modified certificate/entry-lock lemma needs a complete event-order proof covering nodes that have advanced normally while others process a quit. The retained temporal argument and normal terminal-QC case are sketches; the old single-view proof alone is insufficient.
2. **Watchdog soundness and progress.** The isolated three-delay normal timeline and native recovery timeline are explained, but their composition with the one-time predecessor-quit allowance, delayed evidence and fixed openings still needs a full proof. No claim of permanent deadlock or conflicting finalization is established by this review.

Section 7 marks these obligations and makes the downstream regional safety/progress statements conditional. There was no protocol implementation, exhaustive adversarial model check, independent-agent review or deployment. Compilation and redline/layout checks do not establish protocol correctness.
