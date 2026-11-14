# Independent revalidation of the v3.9 dependency

Date: 2026-10-19. Scope: source identity, frozen assumptions, and whether work on the full request-inclusion goal actually requires a model clarification. This is a manual dependency audit using the proof-verifier workflow, not a new protocol proof, universal impossibility result, or whole-protocol approval. Only this report was written.

## Summary

**Verdict: the original communication/temporal-service scope is a genuine dependency for an unconditional v3.9 theorem on either currently developed route. However, it is not yet the sole remaining task: a concrete, independent ownership-to-Ready/report integration audit remains explicitly unperformed.** Consequently, the source evidence supports “full integration cannot honestly be certified yet,” but does not by itself support “no meaningful protocol/proof work can continue until the user replies.” Repetition of the unanswered question does not resolve that distinction.

The mixed-snapshot route needs an old-owner/new-member communication edge. The historical-AUTH route avoids that edge but needs the original historical voting identities to satisfy the original serving premises when revisited, and needs a new recurrent selector. Neither requirement follows merely from retaining data or possessing a valid AUTH. Conversely, the frozen text does not expressly expire correct-to-correct service at worker retirement, so historical reactivation must not be rejected by silently adding that expiry.

There is no established impossibility for all protocols under the frozen model. The recurrent-identity execution refutes the present v3.8 inference under its stated scope. It does not exhaust protocol changes, and its fresh review explicitly retains the service-quantifier caveat.

## Exact source dependencies

All paths below are relative to `formalization_v2/`. Let **M** denote `revisions/v3_1_frozen/model_and_assumptions.tex`, **Q** denote `sections/rotation_v3_8_requests.tex`, and **L** denote `sections/rotation_v3_8_liveness.tex`.

1. **Communication endpoint domain.** M:172–179 says:

   > “Within region $i$, the actual delay of a correct-to-correct message is $\delta\le\Delta_i$”

   and includes “delivery and processing of complete validation dependencies.” M:32–33 fixes members and delay parameters through each AUTH; M:145–146 fixes membership for a session. These clauses do not explicitly say which delay applies to an old-only/new-only endpoint pair. They also contain no explicit active-session expiry. The current Q:38–44 derives its relay bound only within one installed snapshot; L:480–486 deliberately leaves the broader domain unresolved. This is a **Justification Gap in an unconditional transfer proof**, not a demonstrated false communication assumption.

2. **Temporal availability is a separate question.** M:152 says “Correct replicas keep their protocol state and committed blocks.” M:147–150 bounds old-key compromise during certificate acceptance; it does not separately promise future signatures from every uncorrupted historical key. M:223–224 explicitly makes accepted DATA recoverable “without the original producing region staying online.” L:358–364 confines progress to snapshots with their required correct voting weight and communication service. None of these clauses authorizes ignoring an outstanding correct handler, and none alone identifies the future serving domain of every historical roster.

   The original progress text also says “Correct leaders receive repeated turns that are actually entered” (M:297–300), while “Required authorities and recoverable native state become available” and the committee leaves “working regions enough uninterrupted service time” (M:307–309). The precise coupling of identity, preparation and productive service matters. The recurrent trace already grants every born correct identity infinitely many actual entered proposer turns; those turns alone do not create a complete request-bearing result under the trace's pointwise preparation reading.

3. **Ownership v3 is conditional by construction.** `v39_ownership_candidate_v3.md`:23–27 expressly grants the mixed-snapshot service and continuing serving owner. Its fresh review accepts the finite-cut, dissemination and finite-exception proofs *at those declared interfaces*, not the derivation of that service from M. The finite maximum `d_i(r)` cannot manufacture a missing endpoint edge. Protocol retention is useful and necessary, but cannot decide whether the original environment continues covering the named endpoint.

4. **Historical AUTH is a genuine alternative, with a different dependency.** M:41–44 expressly permits a new branch version to reuse an unchanged authority index and identifies a different engine by `(beta,r)`. The old-pair closure prohibition therefore does not forbid a genuinely fresh pair. Current retirement:146–152 prohibits fresh signing for the old pair; it does not prove fresh historical-pair service impossible. However, current control:240–249 retains `r` after zero output and advances to `r+1` after positive output. An arbitrary historical visit needs changed selection code. As `v39_historical_authority_scope.md` correctly separates, old AUTH plus eligibility is not a serving-quorum proof, while lifetime fixed-snapshot coverage would make the same-snapshot handler route admissible without an old/new transport assumption.

5. **The public carrier does not remove the first-result problem.** M:114–119 and 181–196 guarantee publication only after a complete regional DATA/FINAL result exists. M:126 says “There is no separate cross-region transport service in this model.” A pending request, archive pointer, HYDRATED record, or lone FINAL signature cannot be promoted to that service input. No new publisher guarantee or trusted AUTH issuer is justified.

## Concrete work that can still proceed without choosing a model reading

The ownership v3 candidate's remaining obligations are substantive, not merely editorial. Lines 31, 141–145, 184 and 204–206 explicitly leave the actual source-handler, augmented ReadyBundle, exact-parent eligible projection, and unchanged size/load integration open. The fresh ownership review repeats that it does not discharge those interfaces. In particular, **all-recipient raw byte availability before HYDRATED is not yet a proof that every relevant INPUT_REPORT sees a validated eligible `x` at its exact parent**.

A bounded next task is therefore available now:

1. Specify the handoff from the hydrated record store into Q's pending store and canonical identity/index, including the completion event for canonical validation and the treatment of deduplicated, accepted and still-pending requests.
2. Trace that event through the existing complete parent handler, Q:58–75, and immutable report emission, Q:87–104. Prove that a still-admissible hydrated `x` is in `E_v(P)` at every counted report, or identify the exact additional processing predicate for which the frozen interface supplies no bound. Do not assign a whole-pool scan, native import, or arbitrary deferred job a Delta deadline.
3. Give the exact Ready admission/receive rule and certificate carriage. Check that the certificate and its validation fit the old complete-message contract, preserve the first-correct-entry skew, and leave the original immediate ANCHOR vote independent. It is not sufficient to say that the augmented bundle “must fit.” If the source lacks a concrete encoding/load profile needed for a deployment claim, retain that separate limitation rather than inventing headroom.
4. Review the resulting finite-preparation/fixed-slot argument and request implication against those actual rules. Keep the environmental edge explicitly conditional throughout. This can discharge or falsify a remaining protocol interface without answering whether that edge is inherited.

This task materially advances the full goal because it tests a necessary integration layer of the developed request solution and prevents a later favorable model clarification from being mistaken for a complete proof. It does **not** authorize presenting a conditional v3.9 as fulfillment of the user's unchanged-assumption goal. An EN/ZH full-proof manuscript and fresh whole-protocol approval should follow only after both environmental coverage and these integration obligations are settled.

Historical-task scheduling is another research direction, but no complete selection/availability/base-progress proof exists in the audited artifacts. A proposed scheduler must preserve service for a live current cohort while preventing permanent starvation of a relevant historical task; assuming every historical quorum online, or pinning an unavailable one indefinitely, would not discharge that obligation. This audit does not offer that open research direction as a completed escape from the dependency.

## Source identity and validation

The exact Model section extracted from both the frozen v3.1 main file and the live v3.8 main file is byte-identical to M, SHA-256 `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387`.

| Reviewed artifact | SHA-256 |
| --- | --- |
| `reviews/v3_8/v39_ownership_candidate_v3.md` | `a71b64b8bb845463987de8d79b92fed6d8ef61db7a60eec0719dfe1b77a67bea` |
| `reviews/v3_8/v39_fresh_ownership_v3_review.md` | `4008b823b71e316520cc3fa4a50d8e636d1db1f146599012adc633560302b2a1` |
| `reviews/v3_8/v39_historical_authority_scope.md` | `fef9e4ac8851594ba8bd38533feeffe44a6e248523661e45d2dea3268a50bc75` |
| `reviews/v3_8/v39_recurrent_identity_trace.md` | `a47bd3e59f93015f92cbb5e09e3d3d8bf18afcc084cdef718ffe42aadd805275` |
| `reviews/v3_8/v39_fresh_recurrent_trace_review.md` | `66aeb96c9aab02bb1f1d6b0edb760deb07e840e5aaeb4f87a41a1c0826bec885` |

The ownership candidate hash matches its fresh review's input record. The live Q, retirement, liveness and ReadyBundle hashes also match that review's recorded source versions. Checks were manual source/dependency reading and byte/hash comparison. No compile, executable protocol test, model checker, codec audit or performance measurement was run. No source, candidate, frozen assumption, or existing review was edited.
