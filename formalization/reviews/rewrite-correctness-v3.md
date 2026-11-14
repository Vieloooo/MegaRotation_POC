# Summary

**Final Verdict:** The v2-to-v3 changes preserve the previously verified theorem dependencies and clarify the synchronization and payload bounds; no Critical Error or Justification Gap was identified in this bounded delta review.

**List of Findings:** None.

Frozen artifact: `formalization/build/rewrite/review-v3.tex`.

SHA-256: `09f280cdf9caedda6e642c4f8641a4365fd6eec64bb24ab9e432b7a8da19a2d5`.

Scope: the complete diff from frozen `review-v2.tex` to frozen `review-v3.tex`, and its dependencies on the v2 verification recorded in `rewrite-correctness-v2.md`. Unchanged proofs were not independently rerun for this delta. Verifier-only mode; no manuscript edits or additional assumptions were introduced.

# Detailed Verification Log

## 1. Exact proposal authentication remains mandatory

> Lines 318–324: the witness contains the complete mini-block range, producer signatures, and the leader's signature on `(PROPOSE,I,H(x))`; the predicate checks “the leader's PROPOSE signature on this exact batch.”

The prose rewrite preserves the operative requirement that closed v1 F1. A relay cannot change the batch endpoint using only component ORDER signatures. Regional honest-broadcaster validity, completion-witness validity and RC verification continue to use the same authenticated value.

## 2. Target backlog and synchronization time

> Lines 1047–1059: Q begins after an honest participating reporter holds the complete target and its unadopted predecessors; it counts ADVANCE/HOLD adoption epochs, excluding pending ROTATE/CONFIRM; `F_sync` sums the needed per-chunk retrieval bounds.

This removes an ambiguity between data acquisition and report processing. Once the participating reporter holds a complete contiguous target prefix, later outcomes cannot overtake the target, and the longest-fitting-prefix rule makes the stated `Q<=J_0` counting argument valid for adoption epochs. The per-object size assumption remains explicit. Q is still a workload bound rather than a service rate inferred from maximum packet sizes.

The exclusion of pending-rotation epochs is consistent with the scope of the per-target bound: continuing active-generation delivery is analyzed, while an unadopted preview cancelled at handoff has no unconditional final-delivery promise. The startup argument uses the minimum-tenure premise to obtain a reportable adopted batch before rotation guards can preempt that healthy startup.

> Lines 1066–1072: `L_sync=A_G+F_sync+Q T_G`; the minimum-tenure condition uses this L_sync.
>
> Line 1094: `L_final<=A_G+F_sync+(Q+3)T_G+A_P+F_P`.

Both dependent bounds consistently use the full multi-chunk synchronization sum. The change cannot undercount a sequential transfer by charging only one bounded request. It strengthens the startup/tenure requirement when several chunks are needed and preserves the existing alignment, adoption and finalization allowances. F_sync remains conditional on retained reachable holders and finite service deadlines, as in the underlying retrieval lemma.

## 3. Primary push versus ideal content-deduplicated traffic

> Lines 1207–1211: an ideal per-content/per-receiver transfer plan uses M bytes; the actual primary path pushes per mini-block and can repeat identical bodies, so loss-free primary service costs M_0.

This is consistent with the specified transport. Different mini-block positions can contain identical body bytes, so the concrete primary stream need not achieve the ideal distinct-content lower accounting. The general bound `M_0+(a+1)M_r` retains the actual primary-push volume.

> Lines 1218 onward: the simplified `(a+2)M` bound expressly assumes `M_0,M_r<=M`.

That inequality remains conditional and is not asserted for repeated identical-body primary pushes that violate its premise. The repair-race allowance, sequential attempt rule and shared content cache are unchanged.

## 4. Nonmathematical changes and preserved scope

The shortened abstract/introduction preserve the distinction between regional Preview and globally finalized Deliver, and retain the exclusion of consumer execution/publication. Ragged-right algorithm-box formatting does not alter any predicate, timer, message, signature, state transition or proof statement. No consensus rule changed in the inspected diff.

The v2 verification therefore carries forward to this frozen v3 artifact under the same explicit fixed-membership, synchronous-delivery, persistence, healthy-successor and capacity/workload assumptions. This delta does not establish deployment compatibility, implementation correctness or guarantees outside those assumptions.
