# Independent correctness review v2 — superseded PBFT draft

**Status:** Review stopped after the user rejected PBFT for the fast regional path and clarified that a simpler synchronous-broadcast protocol from the good-case-latency source is intended. This report preserves completed observations about the superseded PBFT composition only. It does not verify the intended replacement protocol.

**Reviewed snapshot:** `formalization/main.tex`, SHA-256 `2e20dcbada19e8316ed8ac9bcaaf90b4f0fd7b568394b39a19f87eb6d5f9fc04`, 1019 lines, read on 2026-09-25. Subsequent source edits are outside this report. The verifier did not edit `main.tex`.

## Summary

**Final Verdict:** Both findings in v1 were repaired in the inspected PBFT snapshot, but the publication-liveness statement retained a **Justification Gap** about eventual job activation; the review was stopped before completion because the user superseded the regional PBFT design.

### List of Findings

- **Location:** lines 714–721, “Activation is a public contract event or a uniquely derivable certified event recognized by that contract”; line 910, “The proposer and external-service assumptions complete publication.”
  **Issue — Justification Gap:** The rules constrain a job once activated but do not require an eligible completed A4 range eventually to receive a job and an available correct assignee. Correct P, signing, DA, L1, and proof services cannot complete a job that is never activated. A contract that indefinitely creates no jobs satisfies the written activation restrictions. Thus the sentence is justified for an already activated eligible job under its additional service assumptions, but not as a proof of end-to-end publication from an available A4 prefix. This issue is independent of whether the regional consensus is PBFT or synchronous broadcast.

### Resolved v1 findings

1. **Publication endpoint mismatch — repaired.** Lines 269–281 now group a fixed `K_B` consecutive order batches into each native EVM block, count both DATA and GAP, prohibit batches from crossing native block boundaries, and identify completed-block endpoints as multiples of `K_B`. Lines 717–721 use those same multiples for jobs and define the publishable A4 endpoint as `K_B floor(k/K_B)`. The v1 countermodel with transaction-derived native boundaries strictly inside every order batch is excluded.

2. **Application timeout versus PBFT pacemaker — repaired for the PBFT draft.** Lines 383–389 retain increasing protected view windows. Lines 447–451 initialize the service timer on a new view and prevent either `T_fail` or planned handoff from interrupting its protected duration. Lines 894–895 explicitly include the sufficiently-long-honest-view premise in the conditional-liveness statement. Lines 920–925 replace the detection term by `T_detect`, which includes the residual protected window. These edits close the exact premise mismatch and associated latency omission identified in v1. They say nothing about the latency or recovery proof of the intended non-PBFT replacement.

## Detailed Verification Log

### 1. Native grouping and the publication boundary

> “Completed-block input positions are exactly multiples of `K_B`.” (line 273)

> “If A4 currently ends at `k`, its publishable endpoint is `K_B floor(k/K_B)`.” (lines 719–720)

These statements agree. For any continuing input sequence, every next group of `K_B` batches closes a native block. Advancing A4 eventually exposes additional completed endpoints, provided A4 input itself continues advancing. A4 can still end partway through a native block, in which case waiting is explicitly permitted. Grouping continues across ordering-authority changes because `k` continues from the inherited anchor; native signing authority remains a distinct service. Context for the first batch of a group is inherited when a handoff occurs inside that group.

This verifies only the new grouping contract. The remaining native execution/signing adapter is still explicitly uninstantiated against MegaETH's complete rules.

### 2. Protected view windows and conditional times

> “Application timers may request recovery but cannot interrupt that protected view window.” (lines 385–386)

The application no longer contradicts the sufficiently-long-view hypothesis of its named PBFT contract. The updated theorem explicitly takes that hypothesis. G4 now promises eventual A2 service and states that a uniform deadline additionally requires bounds on all attempts and transfers. The revised symbolic latency expression includes protected-view waiting. These are coherent conditional claims within the superseded PBFT instantiation.

### 3. Epoch identity, data acknowledgments, and authority

> “Control epoch `e` is a sequential control round, with nominal cadence `T_E`.” (lines 507–510)

Round overruns delay future openings rather than fabricating missing wall-clock checkpoints. This makes the required `g_(e-1)` parent and consecutive control indices consistent with recovery from a slow consensus round.

> “The field `a` names the generation active immediately before applying the record...” (lines 521–522)

This resolves the old/new generation interpretation of ROTATE. The successor is activated by applying the unique rotation record, and `e_act` is its control epoch rather than an independent clock command. ADVANCE acknowledgments bind the previous adopted anchor; HOLD and ROTATE set `AC_e` to the empty value while preserving existing retention obligations. These changes preserve the global-prefix induction used in the PBFT draft.

### 4. FFG and ancestor finalization

> “Finalization includes its ancestors in the control chain and their ordered-input prefixes.” (lines 577–578)

> “It can promote an already globally ordered prefix to A4.” (lines 597–599)

The new text correctly distinguishes no new **ordered** input from no newly **finalized** input. A HOLD may retain the A3 frontier while making that existing frontier A4. The monotone target/source persistence from v1 remains in place, so the previously identified delayed-vote surround trace remains excluded on this unique control chain.

### 5. Unresolved publication activation

> “The proposer and external-service assumptions complete publication.” (line 910)

The new endpoint rule establishes a representable eligible range; it does not establish eventual activation of a job for that range. For example, let `K_B=1`, let all inputs reach A4, and let every external service and P be correct and available. An activation contract that never emits an assignment violates no stated “only for A4” or job-format restriction. No publication follows from the written job rules alone. This is a missing activation/assignment liveness premise, not a fork or execution-faithfulness counterexample.

No further review of the PBFT draft was performed after the user's protocol correction. A fresh verification must begin from the exact synchronous-broadcast source, its assumptions and guarantees, and the revised regional protocol; this report cannot be used as approval of that replacement.
