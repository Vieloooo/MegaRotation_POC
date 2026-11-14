# Focused independent review of the hydration-to-Ready adapter

Date: 2026-10-19.

This file exports the already-completed independent manual review. It does not revise the adapter or extend the review to later changes. Paths below are relative to `formalization_v2/`; adapter line references identify the reviewed version.

- Reviewed artifact: `reviews/v3_8/v39_hydration_ready_adapter.md`.
- Reviewed adapter SHA-256: `574ec470c8f7809f1b2d24c92831324bd0325e94c9150df4afe3006383238221`.
- Later adapter SHA-256 supplied with the export request: `efbd9c31860f891c7aa1c298b4428644dd7710354b4e2df17b884afc7eb0d3c7`. That version includes appended review-record prose. This report's verdict applies to the reviewed hash above; it does not claim a fresh audit of the later version.

## Summary

**Final Verdict: Scoped acceptance.** No concrete Critical Error or Justification Gap was found in the conditional adapter's local argument. This is a manual dependency audit, not whole-protocol approval or deployment-timing certification.

**List of Findings:** None within the stated scope. The mixed-snapshot delivery and continuing-owner premises, original native direct-adoption limitation, and concrete packet/processing/load obligations are explicit conditions rather than conclusions established by this adapter.

The checked source interfaces were:

- `sections/rotation_v3_8_requests.tex`: exact-parent request eligibility, report contexts and emission, independent ANCHOR processing, regional sidecar carriage, and local report timing.
- `sections/rotation_v3_5_ready_bundle.tex`: complete executable-state dependencies, send/receive events, logical adoption, and conditional entry-skew scope.
- `revisions/v3_1_frozen/model_and_assumptions.tex`: fixed snapshot thresholds, soundness and persistence, communication scope, and eventual preparation conditions.
- `sync_hotstuff_rr_part2_rotation_v3_8.tex`: anchors, quota and DRAIN rules, admission, complete regional validation, parent selection, recovery, handoff and Ready interfaces.
- `sections/rotation_v3_8_liveness.tex`: fixed-slot and exact-anchor persistent preparation across beta retries.
- `reviews/v3_8/v39_ownership_candidate_v3.md`: the conditional hydration construction, causal all-target delivery, and paced finite-exception argument used as dependencies.

No other review report was used as evidence for this verdict.

## Detailed Verification Log

### 1. Scope and the replaced operation

Adapter lines 7–14 state that the adapter “changes REPORT's selection rule” and “does not certify an unconditional deployment timing theorem or the full unchanged-model goal.” Lines 20–26 distinguish raw-byte hydration from completion of each reporter's exact-parent live-pool projection.

This accurately identifies the source operation in requests lines 58–75 and the limitation of ReadyBundle adoption at lines 90–105. The construction is evaluated as a replacement selection rule, not as a proof that a frozen cut equals the old live `E_v(P)`.

### 2. Pre-Ready state sharing and finite selection

Adapter lines 64–72 require `STATE_CANDIDATE` distribution before waiting for offers, OC or Ready: “Every correct native preparer performs this multicast without waiting for offers, OC, Ready or another preparer's acknowledgement.”

That dependency order breaks the proposed circular wait. The state candidate still requires the original executable object and sound validation evidence; it grants no entry or timer. Its actual preparation and transport have no new Delta deadline.

Lines 79–85 freeze a finite raw/canonical cut only after a complete valid HYDRATED receive event, examine the fixed cut using the verified exact A-state, and require actual completion before signing. New arrivals cannot enlarge the job. Raw receipt is distinguished from canonical insertion, so an unfinished canonicalization queue does not by itself exclude already delivered request bytes.

### 3. Certificate formation and causal coverage

Lines 91–108 construct OC from distinct-member signed keys of weight `>F`, validate its selected body, and permit redundant-job cancellation only after a valid OC exists. Correct members alone have weight `W-B >= W-F > F`, using frozen model lines 137–150. Cancellation therefore does not create a certificate-formation circularity.

Lines 112–116 state that hydrated bytes reach “every correct target strictly before that certificate exists,” and that every correct offer freezes its cut afterward. For a covered request x that remains globally unaccepted and eligible at A, the correct offer's selected key is at most `key(x)`. Every valid OC contains a correct offer, so the minimum supplied key is also at most `key(x)`; `CheckOC` requires the selected key to have a real eligible body.

This composes the ownership candidate's causal all-target delivery with the later frozen cut. It does not assume honest intersection between HYDRATED, offer or REPORT signer sets. Lines 97–104 correctly avoid claiming that cryptographic inspection proves a minimum over an unknown private pool.

### 4. Ready adoption and private-index independence

Lines 127–136 make the local OC attachment nonexclusive and require receivers to validate/adopt the supplied executable state through the original ReadyBundle contract. The receiver “does not wait for its own selection job or native reconstruction job,” while the original residual native-conversion delay remains explicit if direct adoption is unavailable.

This removes private-pool completion from REPORT without treating an arbitrary serialized image as an executable state. Late callbacks cannot replace the attachment or reset entry time. The stated behavior matches the original ReadyBundle adoption and atomic-entry interface.

### 5. First-own-session DATA under recovery

Lines 142–149 define the obligation through fully validated ancestry. If there is no own-session DATA, the first own-session DATA executes after A and must carry OC. A speculative first DATA discharges the obligation only for descendants on its branch; returning to A requires it again.

The original anchor definitions at main lines 593–600, quota and DRAIN rules at 696–734, parent rules at 1017–1019, and recovery selection at 1274–1309 support this dichotomy. ANCHOR-only recovery can return to A. A DRAIN requires reaching the positive DATA quota and therefore cannot bypass the first-own-session DATA requirement. Retaining a descendant retains its validated first-DATA witness.

Lines 149–151 apply the check in recovery, receipts, Completion and DRAIN ancestry and preserve regional sidecar ownership. Public erasure is distinguished from the actual historical correct Vote that checked the augmented predicate.

### 6. Initializer advertisement and REPORT binding

Lines 157–182 bind opening reports to the initializer's separately signed `INPUT_ADVERTISE(RC, Hash(canonical_OC_bytes), OC.selected_key_or_EMPTY)`. The initializer chooses one complete OC; correct reporters sign the matching digest/key, and common proposal validation checks that same advertisement and report set.

This prevents an untrusted relay from replacing a correct initializer's certificate choice. Invalid or missing sidecars block only REPORT; line 176 preserves original NEWVIEW forwarding, ANCHOR voting and evidence handling independently. A Byzantine initializer can split its own reports, but the adapter does not claim that this creates an additional Quit proof. Original signed BFT message formats and proposal identities remain unchanged.

### 7. Timing and resource conditions

Lines 192–210 retain the original round arithmetic: ordinary-term reports reach the leader by `e+3 Delta` and first DATA is voted by `e+4 Delta`; initial/recovery NEWVIEW processing produces ANCHOR votes and reports by `e+4 Delta`, both reach the leader by `e+5 Delta`, and first DATA is voted by `e+6 Delta`.

The argument introduces no extra collection round or timed private-pool scan. It is expressly conditional on complete processing of the admitted concrete packets and the original prepared-entry interface.

Lines 240–274 enumerate the OC, advertisement and report objects, their signature/body checks, and packet/handler/load inequalities. Line 274 states that the deadlines follow only if the old abstraction admits those dependencies. The note does not infer old-envelope capacity from finiteness, or infer same-Delta completion from separate message admissibility. The native adoption and concrete capacity obligations remain unresolved as stated.

### 8. Beta retries and stalled-G preparation

Lines 216–220 retain fixed-K jobs, state and OC across beta retries. A changed exact anchor or AUTH/native domain requires a new task; a recovery parent either carries the earlier first DATA or returns to A and reuses its prepared certificate.

This matches the fixed-slot, exact-anchor persistent-preparation argument in liveness lines 44–105. In the stalled-G tail, the new finite selection completion can join the preparation maximum without being charged as a post-entry pool-rebuild operation. The conclusion remains conditional on the original direct-adoption and concrete resource scope.

### 9. Inclusion through accepted session-first DATA

Lines 224–234 count accepted positive-output sessions instead of the replaced live-pool term-first reports. For each nonexceptional accepted session-first DATA, coverage supplies an eligible selected identity with key at most `key(x)`, absent from its accepted opening ancestry. Only actual global acceptance spends the finite predecessor potential; discarded speculative executions do not.

The finite per-session quota and slot-advancement rules support the passage to infinitely many session-first opportunities. Thus the new argument does not silently reuse the old claim that every correct term-first reporter minimizes its entire current eligible pool.

## Conditional limits and validation record

The mixed-snapshot communication and continuing-owner premises were taken conditionally from the ownership candidate, not derived from frozen v3.1. Eventual native preparation and sound executable-state validation were inherited source interfaces. The concrete codec, message capacities, aggregate dispatch/processing profile and native direct-adoption implementation were not certified.

Validation consisted of manual source reading, causal-order and threshold checks, finite-cut and retry analysis, recovery/ancestry checks, timing algebra, and resource-footprint inspection. No executable protocol test, model checker, codec implementation, CPU/transport benchmark or whole-protocol verification was performed. No adapter or source file was changed by this review; this file is only the saved report.
