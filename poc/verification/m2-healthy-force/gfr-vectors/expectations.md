# Source-first GFR boundary expectations

Expected event sequences follow v10 GFR §§3.2–3.4, 4.2–4.5 and M2-HEALTHY-FORCE-1 §§3–6. The fixtures supply validated control projections and structural regional finality witnesses. Real Anvil provenance and native regional signatures are checked by the separate main-run verifier. Public time has no dependence on wall time. All vectors are deterministic and use no random seed.

## Time arithmetic

Use the frozen profile with L2 genesis at 10 s, 1 s L2 slots, 2 s L1 slots, 10 s announcement, 3 s preparation, 3 s grace and 8 s expiry. L2 height h has timestamp (10+h) s. L1 height ℓⱼ has timestamp 2ℓ s and controls in it activate at (2ℓ+10) s. The genesis session is (0,0) in R0, window [1,10]. Its SessionEnd key is 24 s, and Expire(h) is at (18+h) s. For these fixture keys, the slot-derived first height is the integral activation time in seconds minus six. Take the maximum with h_G+1 and extend the window nine more heights. Observe G, branch, active descriptor or HALT, replay positions, record effects, and lifecycle separately.

| Case | Prestate and minimum new public input | Expected |
| --- | --- | --- |
| HF04 unused NEXT | Empty G. Valid Force target 0 at 18 s (L1 height 4), Espresso watermark 18 s. | Branch 1, r0, R1, source R0, genesis anchor and reference, window [12,21], floor max(genesis origin,4). G unchanged. BranchClosed then SessionAssigned. |
| HF04 inherited floor | Empty G, genesis floor 3. Valid Force target 0 in L1 height 2 at 14 s. | Branch 1/r0/R1, reference and anchor genesis, window [8,17]. Floor stays 3. |
| HF04 contributed NEXT | G physical head h1 from (0,0), before Force target 0 at 18 s. | Branch 1, r1, R2, source R0, exact h1 anchor/reference, window [12,21]. G unchanged. |
| HF04 PENDING | Plan provider returns None for the regular successor of h9 at cutoff 24 s. Force target 0 at 26 s (L1 height 8), before Expire(h10) at 28 s. | Branch 1, saved r1 and source R0, R2, exact h9 anchor/reference, window [20,29], floor 8. G unchanged. |
| HF05 | Physical h1–h9 accepted before 24 s. Old complete h10 proof and body first publicly available at 24 s. Same-key Force targets branch 0. | Force then Espresso, old SessionEnd no longer applies. Anchor remains h9, branch 1/r1/R2, window [18,27]. No G append from old h10. |
| HF06 | h1 accepted, h2 missing. Force target 0 at 20 s. | Force then Expire(h2) then equal-key Espresso. Force assigns branch 1/r1, then Expire closes branch 1, Tail HALT. G unchanged. |
| HF07 | h1 accepted and h2 pending. Three controls in L1 height 4 at distinct sorted transaction/local log positions, target 0 then 0 then 1, shared act 18 s. | First creates branch 1/r1, second has empty effects/lifecycle, third sees branch 1 and creates branch 2/r1. All three positions precede Espresso. Delivery replay with the identical prefix has no new records or lifecycle. |
| HF10 | Empty G and Force key 18 s. Reader A sees complete L1 through height 4 first. Reader B sees complete Espresso watermark first. | Each waits at the missing input boundary with original committed state. Neither sees a Force effect early. With both prefixes, a Plan WAIT still leaves the original state unchanged. With the plan available, A/B agree at the same replay position on G, closed branches, Tail/descriptor and lifecycle. |
| HF11 stale | Target 7 while branch 0 is active, Force key 18 s. | Exactly one Force record with no effects/lifecycle. G, branch, floor, Tail unchanged. |
| HF11 HALT | Empty G reaches Expire(h1) at 19 s, then Force targets current beta at 20 s. | Force record has no effects/lifecycle, preserves HALT and G. |
| HF14 | Physical h1–h9 before old SessionEnd; old h10 is unavailable before 24 s. Then qualified h11 from new session. | At cutoff (0,1) in R1, window [11,20], chosen physical anchor h9, Plan reference genesis. G remains h9. Only h11 admission appends [10,11] atomically, with h10 padding and h11 physical. |
