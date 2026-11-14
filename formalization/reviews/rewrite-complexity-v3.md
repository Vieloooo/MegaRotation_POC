# Final complexity and accounting review — rewrite v3

**Verdict: PASS for the reviewed complexity, latency-accounting, retention and receiver-interface scope.** No unresolved findings remain from the v1/v2 audit. This is a targeted audit of the stated conditional bounds, not a separate proof certification of the entire protocol.

- Reviewed snapshot: `formalization/build/rewrite/review-v3.tex`.
- SHA-256: `09f280cdf9caedda6e642c4f8641a4365fd6eec64bb24ab9e432b7a8da19a2d5`.
- Method: compared the complete v2→v3 diff, read the changed sections in their current context, and checked their use in the synchronization, minimum-tenure and finalized-delivery formulas. The unchanged protocol/cost components retain the conclusions recorded in the v2 review.
- Manuscript modifications by this reviewer: none.

## Residual v2 findings resolved

1. **Actual primary traffic is now distinguished from ideal deduplicated traffic.** Lines **1204–1211** state that an ideal once-per-digest transfer plan uses `M`, whereas actual primary push can repeat identical bodies at different positions and costs `M_0`. Lines **1212–1223** retain the sound general response bound `M_0+(a+1)M_r`, explicitly condition the `(a+2)M` specialization on `M_0,M_r <= M`, and account for in-flight push/pull races. Thus the manuscript no longer claims that receiver caching alone prevents repeated primary bytes from crossing the network.

2. **Backlog adoption and data acquisition now have separate premises and costs.** Lines **1047–1058** start the `Q <= J_0` argument only after an honest participating reporter possesses the complete target and predecessor objects. They count ADVANCE/HOLD adoption epochs, excluding pending ROTATE/CONFIRM intervals. At least one complete pending outcome fits by the stated individual-outcome/evidence condition (**1061–1063**), so longest-prefix reporting removes at least one predecessor per such epoch, while later arrivals cannot overtake the target. `F_sync` explicitly sums the required per-chunk acquisition bounds. Both `L_sync` (**1066**) and finalized delivery (**1094**) use `F_sync`; one bounded-request `F_G` is no longer silently charged for arbitrary multi-request synchronization.

## Final scope of the accepted bounds

- Regional `2 delta_R` remains a complete-object good case, separate from producer accumulation, scheduling, consumer transfer and mandatory serialized completion (**1077–1089**).
- The minimum-tenure condition includes the conservative planned-tenure leader bound, data acquisition, report backlog and additional epoch alignment (**1064–1075**).
- Finalized delivery is conditional on a continuing eligible generation; a preview discarded at a handoff has no finalized-delivery guarantee (**1091–1099**).
- The v2 corrections remain unchanged: announcement propagation budgets; continuous-failure versus intermittent-Byzantine local bounds; monotone receiver activation; READY's connection to a globally responsive regional quorum; interval costs for HEAD, announcements, pulls and evidence; and explicit retained history size.
- Direct versus backbone costs, payload versus control costs, and signature-chain versus semantic-evidence sizes remain separated. The bounds continue to rely on the specified delivery, service-capacity, storage and workload assumptions; they do not claim an unconditional or optimal performance guarantee.
