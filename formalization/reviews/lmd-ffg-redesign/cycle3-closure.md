# Cycle 3 targeted closure review

## Summary

**Final Verdict:** Both substantive cycle-2 accounting findings are closed: the revised report-body bounds and parameterized candidate-verification bound are valid, and the previously verified core safety/progress arguments are unchanged. One narrow **Justification Gap in wording/scope** remains if “complete-witness processing/transfer budget” is intended to include all control evidence rather than relative report-body bytes.

Reviewed artifact: `formalization/build/lmd-redesign-baseline/cycle3-main.tex`, SHA-256 `3b19bd5ef9fa7b12c43a7cbddd201e4c49e93d71fae0ee0e84583d7c7f1004d2`. `formalization/main.tex` matched this hash when checked. The complete diff from the frozen cycle-2 source was inspected. Besides a layout-only `Needspace`, changes are confined to resource-accounting prose. The protocol algorithms and correctness proofs have not changed. No manuscript edits were made.

### List of Findings

- **C1 — Scope clarification for the complete-witness bound.** Location: cycle-3 lines 1105–1106, “the coarse complete-witness processing/transfer budget is $O(J_GB_G)$”. **Issue: Justification Gap if interpreted as all witness/envelope bytes.** The stated $B_G$ limit applies to relative report-branch bytes, while lines 1036–1038 separately define report metadata/history evidence of size $L_G^c$, excluding the new AB signature chain of size $O(N\kappa)$. Thus $O(J_GB_G)$ is a valid relative report-body byte bound. It is not an unqualified bound on every complete envelope's bytes without also accounting for metadata/history and signatures. This is a naming/scope issue, not a further cap-two error, a safety attack, or a need to change the protocol. Calling it the “relative report-body byte budget” would remove the ambiguity; a bound on all admitted complete envelopes would instead include $O(J_G(B_G+L_G^c+N\kappa))$ before retries and physical routes, with consistent units.

No other remaining finding was identified in the targeted review. The verification does not establish that a deployed network can sustain a chosen admission budget; the manuscript correctly makes the actual admitted-traffic delay envelope an assumption.

## Detailed Verification Log

### 1. Prescribed honest traffic versus Byzantine offers

Lines 1074–1079 explicitly restrict the earlier transmission/signature counts to prescribed honest protocol traffic and introduce $J_G$ as a bound on **all candidate report-envelope receptions admitted for complete verification at responsive honest endpoints**. It includes direct Byzantine offers and reports discarded only after verification.

This closes the prior unsupported inference from a two-value retained-state cap to a total validation cap. The actual counted work is now bounded by a stated parameter. The required reservation for all prescribed honest packets is preserved, so the accounting does not silently alter the broadcast algorithm by discarding later necessary honest forwarding.

### 2. Verification costs

Lines 1080–1087: each admitted candidate needs at most $O(N)$ signature-chain checks plus at most $\nu_G$ checks for its underlying report evidence. Multiplying by the total endpoint reception budget gives $O(J_G(N+\nu_G))$.

Each honest node forwards at most two semantic values per origin, so across $N$ origins and $N$ possible honest nodes there are $O(N^2)$ honest logical multicasts, each delivered to at most $N$ endpoints. Hence prescribed honest multicasts induce $O(N^3)$ endpoint receptions. The older bound follows only if the admission policy supplies a compatible $J_G=O(N^3)$ budget, as the new text now states. It is not claimed to follow from cap-two retention itself.

The separate $O(EN^2(N+\nu_G))$ forwarding-path term applies to honest-generated traffic. Additional admitted adversarial relay work is expressly outside this term and requires its own admission accounting. This division is consistent with the endpoint definition of $J_G$.

### 3. Final-vector and intermediate-AB body sizes

Lines 1097–1102 now distinguish three different quantities correctly:

1. The final common vector has at most one non-bottom report per origin, so its relative report-body total is at most $NB_G$.
2. Honest cap-two sets have at most $2N^2$ `(honest participant, origin, retained value)` memberships. The number of distinct semantic report values occurring in those sets is no larger. Their report-body union is therefore $O(N^2B_G)$.
3. Replicating that body union uncached across at most $N$ endpoints costs $O(N^3B_G)$ aggregate endpoint bytes before retransmissions and routes.

These bounds permit different honest receivers to retain different pairs for an equivocating origin. They therefore correctly cover the six-distinct-body example, and do not mistake common bottom output for absence of intermediate data.

### 4. Further admitted candidates and the residual scope issue

Lines 1103–1107 correctly state that direct Byzantine candidates ignored only after complete verification are not bounded by cap-two retention. Under a total $J_G$ admitted-candidate reception budget, each candidate's relative branch body costs at most $B_G$, so the coarse relative-body-byte bound is $O(J_GB_G)$.

The phrase “complete-witness processing/transfer budget” is broader than that derivation. Control metadata/history and the new AB signature chain already have separate size variables. C1 records this limited scope ambiguity; the underlying body accounting is correct.

The following repair equation separately accounts for failed capped responses. Physical link-byte costs separately multiply transfers by route length. Thus the revision does not confuse unique bodies, endpoint copies, retries, and physical-link traffic.

### 5. Consistency with the existing core proof

The resource-accounting change does not alter:

- cutoff filtering before report snapshot fork choice;
- latest-HEAD bookkeeping without reviving an older vote;
- fixed authority and complete whole-batch validation;
- cap-two broadcast agreement and honest-origin validity;
- common-vector GHOST selection and whole-report packing;
- adjacent certified checkpoints and fixed finalized handoff boundaries;
- consumer anchor/activation installation before finalized append;
- fixed-anchor progress, cyclic readiness selection, or minimum tenure.

Consequently the cycle-2 verification of these properties carries over unchanged under the same explicit assumptions. No certified-prefix safety counterexample or new conditional-liveness defect was found. No claim about regional head convergence, individual transaction inclusion, per-preview finality, or arbitrary-stall recovery is introduced by the revised accounting.

## Final-artifact closure addendum

**Final scoped verdict: Valid under the stated assumptions; no remaining finding from this review.**

Verified final artifact: `formalization/build/lmd-redesign-baseline/final-main.tex`, SHA-256 `a1697f0842252a3af2d2a60cbb268ee73b3d00d8034629b8059762a8c635615b`. The working `formalization/main.tex` has the same hash. The complete diff from the cycle-3 frozen source contains only the requested accounting-scope clarification.

The revised paragraph correctly names $O(J_GB_G)$ the **relative report-body byte budget** and separately gives $O(J_G(8B_G+L_G^c+N\kappa))$ **bits** for complete endpoint envelopes. The factor eight converts the body-byte limit to bits; metadata/history evidence and the new AB signature chain are now included in their own terms. Retransmissions and physical routes remain separately accounted for. This closes C1 without changing any algorithm, proof, or admission assumption.

The original cycle-3 findings above remain as a record of their original target. Both substantive cycle-2 findings and the final cycle-3 scope clarification are closed for the final hash identified here. This is a conditional formal verification result within the manuscript's explicit synchrony, availability, admission, retention, and tenure assumptions; it does not supply an implementation benchmark or an additional guarantee outside those assumptions.
