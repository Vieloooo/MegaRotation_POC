# Complexity review of integrated main_v3, version 1

Target: `formalization/main_v3.tex`, read as a 1,449-line version on 2026-09-28. Scope: §8 against the actual regional, DRAIN, regular-rotation and forced-rotation algorithms. Read-only manuscript review; the proof-verifier workflow was used. No claim is made about full protocol correctness or the unresolved implementation of admissible recovery cuts.

## Summary

**Final verdict:** the constants, steady-state communication bounds and amortized-throughput derivation are valid under their stated good-case premises, but the handoff timing table has one **Justification Gap**: it does not represent the explicit wait for an eligible successor permitted by R3/F4, or say that the displayed formulas exclude that waiting case.

There is no identified critical algebraic error. The terminal-drain communication and authority-history acquisition need clearer accounting for the reader, but are already parameterized rather than incorrectly claimed free.

### Findings

1. **Justification Gap — eligible-successor wait is absent from the displayed handoff clocks.**
   - Location: §8.2, `H_ij = F_i + D_i + X_ij + A_j`, the normal row of Table `tab:operating-paths`, and `T_ij^acc`; compare R3 “If i′=⊥ … Process later admissible entries …” and F4 “If no successor exists … Later passive recovery may enable this same handoff.”
   - `D_i` is explicitly delay to the old terminal receipt, `X_ij` is transfer to the successor and `A_j` is successor initialization. None is defined to include waiting for a new eligible successor. A perfectly healthy term can reach its terminal receipt while every other region is off, so the first new receipt can be delayed arbitrarily beyond the displayed expression. The accident paragraph says a live successor is necessary, but does not anchor the formula to its availability time.
   - Consequence: normal/forced timing formulas are verified for the case in which an eligible successor is already available in the effective snapshot and remains available through initialization; they are not established for the full R3/F4 waiting branch. The waiting branch remains conditionally live under the paper's progress assumptions, without a numerical bound.

2. **Accounting/presentation clarification — `D_i=0` does not mean zero DRAIN messages.**
   - Location: §8.2 “It is zero on the healthy direct-commit path”; §8.3 “If terminal drain emits d additional control blocks, charge their proposal and vote exchanges separately”; compare Q2–Q4.
   - The APP terminal's timer can finish without an additional finalization round, yet the native pipeline keeps creating DRAIN children while that timer runs and until closure. Therefore healthy `D_i=0` can coexist with many control blocks. Under a deliberately fixed ideal proposal cadence of `I_i=2δ_R`, the control-block count during a `Θ(Δ_R)` timer interval can scale as `Θ(Δ_R/δ_R)`. No unconditional O(1) control-block count follows from the stated upper bounds on message delay.
   - Consequence: §8's parameterized communication statement is correct, but the reader needs a term-level accounting expression or an explicit warning that normal-boundary traffic includes these control blocks. Do not let “zero additional finalization round” be read as “zero extra communication.” Also use `d_ctrl` rather than `d` here because `d` already denotes the application quota count.

3. **Accounting/presentation clarification — authority-history acquisition is cached in the regional steady-state bounds.**
   - Location: §8.1 physical-block bound and §8.3 normal outer-transfer bound; compare terminal validation of the finalized L1 origin, complete entry prefix and registry proof in §4.3.
   - The normal outer transfer correctly carries `s_L1`, but regional validators must also obtain that history before approving a terminal APP block. If they have not already cached it, its acquisition/dissemination is additional traffic and may add readiness delay. It need not occur on every batch or require a fresh L1 transaction.
   - Consequence: retain the displayed steady-state bound for cached authority context, and explicitly charge missing registry evidence to retrieval/control acquisition. This is not a reason to add a new L1 finality wait to every normal rotation.

## Detailed verification log

### 1. Regional commitment and transferable receipts

Claim: `C_i=δ_R+2Δ_R`, `F_i=C_i+δ_R`, and external receipt by `F_i+δ_P`.

S1 transmits a ready proposal at time zero. In the stated honest-leader good case, all correct recipients receive the complete valid proposal by `δ_R`, vote and start the unchanged `2Δ_R` timer. The inherited safety argument excludes a valid conflicting quit in this path. Therefore all correct replicas commit by `C_i`. All honest stake exceeds the post-commit receipt threshold, and their matching CONFIRM endorsements reach a correct collector within another `δ_R`; hence `F_i` is a valid conservative bound. Sending the complete receipt and body to P adds `δ_P`. The direct-endorsement improvement is consistent with the actual “peers and subscribed consumers” transmission rule. **Verified.**

The endpoint is sequencer commitment/receipt delivery, not L1 settlement. §8.4 explicitly keeps these separate. **Verified.**

### 2. Pipeline interval and sustained rates

Claim: `I_i=2δ_R` between ready physical proposals and batch rate `1/I_i` inside an application segment.

One complete proposal hop and one return vote hop suffice for the honest leader to collect a >2/3 QC. S3 permits a child immediately on that QC, independently of running commitment timers. Offered load, construction and bandwidth are explicitly qualified. Within the K-APP prefix this is an application-batch interval; after quota exhaustion the same physical interval may produce DRAIN rather than application batches. §8's normal-boundary gap accounts for the application pause. **Verified.**

`u/I_i` is correctly described as aggregate mini-block throughput, with no claim of independently spaced finality for the u mini-blocks in one batch. **Verified.**

### 3. In-region replacement constant

Claim: `T_i^view=4Δ_R+2δ_R` from first correct quit to first fresh proposal, followed by `F_i` to a transferable receipt.

From a first correct quit at t0, its complete quit proof reaches all correct participants by `t0+Δ_R`. Each waits `Δ_R`; hence the honest next leader has entered by `t0+2Δ_R`. Its V3 deadline is at most `t0+4Δ_R`. The NEWVIEW broadcast and return votes consume at most `2δ_R` in the good case, giving a current-view anchor certificate and allowing a fresh child. This reasoning assumes the next leader is honest and needed ancestry is available, as §8 says. Detection and failed intermediate leaders are explicitly outside this clock. **Verified.**

The `6Δ_R` first-proposal deadline is the p=1 instance of `(2p+4)Δ_R`. It is not presented as a uniform failure-detection bound from every later point in the view. **Verified.**

### 4. Initial successor startup

Claim: `A_j≤2Δ_R+2δ_R`, once the correct successor participants possess the verified common anchor with the required readiness skew, and the initial leader is honest.

The manuscript actually preserves the initial V3–V4 NEWVIEW path. Measured from common readiness, the leader's `2Δ_R` wait followed by a `2δ_R` announcement/vote exchange is a conservative bound. Successor-side distribution is explicitly part of `X_ij`. The text does not silently presume `A_j=0`. **Verified.**

A Byzantine initial leader can require additional local view changes; the formula is expressly a good-case startup bound. The broader conditional-progress theorem need not inherit this constant. **Verified.**

### 5. Terminal drain and first-new-receipt timing

Claim: `H_ij=F_i+D_i+X_ij+A_j` from old terminal proposal to first new proposal; add `F_j` for the first new transferable receipt.

The algorithm waits for the old terminal APP receipt, transfers it and initializes the successor before any new APP proposal. In a healthy view, the terminal APP's own timer suffices; Q2's control descendants are not a further required finalization stage. If a view change re-certifies that terminal as an unchanged anchor, Q3's fresh DRAIN starts a new commit timer and can finalize the terminal ancestor. Parameterizing any extra delay as `D_i` is correct. **Verified, conditional on the eligible-successor issue in Finding 1.**

There is no reintroduction of a separate StreamFinal state or speculative cross-region promotion: all extra terms correspond to receipt transport or initialization in the stated algorithms. **Verified.**

### 6. Amortized throughput across K-batch terms

Claim: `I_bar=((K−1)I+H)/K` and batch rate `1/I_bar`.

Between successive first proposals of homogeneous healthy terms, there are K−1 internal APP proposal gaps and one terminal-to-next-first APP gap H. This interval produces K application batches. Adding the first new batch's `F_j` to H would double-count the pipeline offset when averaging a repeated schedule. Startup/drain endpoints for finite observation windows are acknowledged. **Verified, conditional on the same boundary hypotheses used to define H.**

The interpretation of “timeout cross-region” as fixed-quota term expiry is consistent with the latest accepted outline and explicit manuscript text. This is a scope choice, not a derived equivalence to wall-clock expiry. No numeric wall-clock rotation bound has been proved. **Verified against current scope.**

### 7. Accident expression

Claim: detection + safe cut/closure + committee certification + L1 wait + complete anchor transfer + successor initialization + first fresh receipt.

These are all necessary named components of the displayed forced path. Their sum is correctly called a sequential upper expression rather than a tight latency or lower bound; some overlap is possible. Safe-cut recovery and L1 progress remain explicit assumptions. The formula does not wrongly claim that a status certificate proves absence of a hidden finalized suffix. **Verified as a phase accounting, subject to Finding 1's eligibility wait.**

An implementation must determine how its cut-discovery and fencing waits interact with L1 finality; no actual finite bound on `T_cut` or numerical Ethereum recovery time follows here. §8 states this limitation. **Verified.**

### 8. Message and bit counts

Full proposal forwarding plus votes gives O(n_i²) correct-party transmissions per physical block. Explicit parent QCs of O(n_i κ) bits replicated over O(n_i²) forwarding messages yield O(n_i³ κ) control bits, while complete new bodies yield O(n_i² ℓ). Post-commit endorsements are lower-order O(n_i² κ), with O(n_i κ) control bits for each external subscribed receiver. **Verified, with missing ancestry/authority proofs charged separately.**

BLAME-certificate relay, STATUS, NEWVIEW forwarding and recertification can be bounded by O(n_i²) transmissions and O(n_i³ κ) control bits for one replacement, with missing ancestry separate. **Verified.**

The O(n_j) normal outer-transfer claim explicitly assumes an assembled receipt at one reachable holder and direct delivery to each successor member. Its O(n_j(a_ij+n_i κ+s_L1)) byte bound includes old receipt signatures, missing anchor data and registry evidence. This does not erase quadratic successor bootstrap. **Verified.**

A fixed-index m-entry status map costs O(m) bits and its explicit c-member certificate costs O(cκ). The text correctly leaves internal committee agreement at M_C/B_C rather than deriving consensus costs from a threshold signature. L1 transaction payload and dissemination are distinguished from committee internals. **Verified.**

### 9. Proof-system-neutral rollup cost

The section does not claim constant L1 cost, a concrete gas number or a settlement delay from the abstract verification relation. It states relevant input dimensions and leaves proving/challenging/verification to the implementation. **Verified.**

## Validation boundary

This review validates the displayed phase arithmetic and asymptotic accounting against the manuscript algorithms. It does not establish committee observation accuracy, safe-cut implementability, snapshot serialization, a particular L1's timing, network provisioning or a circuit/fraud-proof implementation.
