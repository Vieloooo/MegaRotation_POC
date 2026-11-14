# Fresh verification of the conditional historical-task integration

Date: 19 October 2026. Independent adversarial source review; verifier mode only.

## Summary

**Final Verdict:** The runtime's local scheduler invariants and the controller's finite-credit and local-arbitration arguments are valid under their expressly stated interfaces. Their joint progress implication is viable but remains conditional on registration coverage/completion, historical service coverage, prepared-handler conformance, and actual valid control submission. This review does not establish the unconditional historical-task, request-liveness, or whole-protocol safety goal.

**List of Findings:** No Critical Error or additional Justification Gap was found in the local claims at the reviewed scope. The conditions left open by the candidate are material: none is discharged by this verdict. In particular, finite historical jobs do not establish a uniform response time, and eventual local dispatch does not establish external admission.

For the control activation and restoration conclusions, this verdict uses the original progress condition that Espresso continues with growing timestamps (`revisions/v3_1_frozen/model_and_assumptions.tex:301–302`). In particular, the threshold-crossing step of controller Proposition 3 is read with that original condition; accepted DATA growth alone is not used to infer timestamp growth. Its displayed list of local conditions is not a standalone replacement for the original progress premises.

### Frozen scope and independence

I first read the proof-verifier skill and original frozen/current protocol sources. Only after receiving the final hashes did I open these three candidate files. I did not read the prior reviews, the inventory-adapter note, or the older historical-authority/identity analyses cited by them. I made no source repairs.

The exact candidate files, with SHA-256 values checked before and after review, are:

| File under `formalization_v2/` | SHA-256 |
|---|---|
| `reviews/v3_9/historical_service_design.md` | `bbfdcc3f29b395196e0871b115f98e9859d86bb291881c3da49ca37c882a972d` |
| `reviews/v3_9/historical_runtime_integration.md` | `e383e49f572892cdeb2b7ec5d6a6dd9773f985502bbff2225337c1c6b1551cb3` |
| `reviews/v3_9/historical_control_integration.md` | `3b4528a9e720e20d3e786aff321dcb5ad74f1499e8bbe91b960f848f500537db` |

Original sources inspected include the frozen v3.1 Model and Part I, v3.9 main source, and its current source-step, health, control, liveness, retirement, ReadyBundle, logical-carrier, export, request and public-import interfaces. A direct comparison confirmed that the v3.9 Model text equals the frozen Model after trimming boundary whitespace.

### Dependency map

The conclusions have separate dependencies:

1. Canonical registration events and complete fixed cuts support deterministic queue projection and head selection. They do not prove discovery of every private/speculative AUTH.
2. Selection fixes an authenticated pin. Canonical DATA transactions maintain its service count and release it. An eligible unserved pin makes later registration tails irrelevant to its retry selection.
3. Historical service coverage plus finite fixed-anchor preparation supplies the regional response argument. Extended complete-handler conformance is additionally required for a uniform `L_(i,r)`.
4. Durable controller credits give finite interference at fixed G and for a finitely served task. Actual valid submission, growing Espresso time and canonical processing give a public control opportunity.
5. Queue fairness and request inclusion additionally need coverage/service for every possible blocking incumbent. The local lemmas do not independently establish those end-to-end conclusions.

## Detailed Verification Log

### 1. Identity, registration and lazy queue projection

Runtime lines 20–22 state that task identity is `(i,r)`, engine identity remains `(beta,r)`, and the authoritative queue-age key is a canonical birth event. This is compatible with immutable AUTH indexed by r. A materializer-assigned serial must represent that event order; neither callback order nor private observation may create an alternative order. Controller token serials are therefore read as this canonical token identity, not an independently assigned local number.

Runtime lines 36–38 say, “Removal of an interior task is allowed” and select the minimum birth event over eligible regional heads. Both steps are justified. Every registered task has positive weight in its own immutable AUTH; governance and cooldown are common to tasks of one physical region. Hence either all nodes of that regional list satisfy those common eligibility gates, or none does. Removing a served interior node and appending its replacement leaves every remaining node in birth order. A non-head REGULAR contribution therefore cannot accidentally discharge or re-age the head.

The induction at runtime line 132 correctly gives one outstanding token per task and identical projections at an identical completed cut. Repeated DATA does not generate another event. Repeated valid contributions of one continuing duty do generate distinct exact-task discharge/requeue events, consistently with the service counter and fixed remaining session quota. A different AUTH of the same physical region receives no such credit.

Runtime lines 46–50 require finishing a fixed prefix before a head-dependent selection and explicitly forbid replacing that job with a later cut. This rules out the moving-target failure. In particular, a later raw MSIG/AUTO input is retained without being applied over unresolved earlier canonical scheduler state. Once a completed selection proves there is no candidate, advancing to the next historical position is permitted; missing evidence is never that proof. Each next job has a fixed target and retains its predecessor's completed work.

This eventual-job conclusion uses the conditional registration contract and continued execution stated in the candidate. Finiteness alone would not prove either retrieval completion or CPU service. The text supplies no deadline for that work and does not place it in a prepared `Delta_i` interval.

### 2. Canonical cuts, control order and historical selection

Runtime lines 58–62 distinguish genesis registration, adapter-defined registration positions, and registration during public REGULAR authority resolution. In particular, a successor AUTH is registered after terminal DATA commits, and is not backdated to an earlier selection header. This is necessary for deterministic queue age and is stated correctly.

Runtime lines 66–80 preserve independent STATUS/CLEAR before the first MSIG matching the entry beta, followed by AUTO only if that MSIG did not act. A REGULAR handoff leaves beta unchanged. Stale MSIGs apply no attached configuration. A matching legacy MSIG retains its original attached-configuration effect. There is at most one effective Force in the prelude.

The closure step fixes exact pre-payload G and its original provenance, closes the old beta and increments beta before successor catalogue waits. The new historical selector changes the Force target-r rule explicitly; it does not claim that the old r/r+1 rule already implements this design. An unresolved selection grants neither an empty candidate set nor a new public opening time.

The cooldown case split is coherent. An eligible unserved pin is the only permitted task, so this AUTO adds no cooldown to its region. In the unpinned case, cooling the source requires an eligible candidate in a different physical region. Cooling the source therefore does not erase all candidates that justified the action. Older cooldowns and OFF remain effective; MSIG creates no automatic cooldown. The opening uses the realized first eligible historical position and is not settled again in the same prelude.

Saved pre-payload snapshots are not rewritten by later registrations, payload credits or lazy materialization. The handoff verifier derives the token, task, anchor and first eligible position from authenticated replay. A packet's token label does not become a new authorization source.

### 3. Atomic DATA service and pin discharge

Runtime line 82 requires DATA append, ordinary accounting, exact-task counter increment, discharge/requeue descriptor, matching-pin clearance and cursor exposure in one transaction. Lines 84–88 place queue materialization and terminal successor authority resolution after this transaction. This preserves the original per-DATA commit boundary, including quota-one output.

A current public producer's initial AUTH was already required by its own fixed boundary. Under the stated registration order, its task has a logical registration before its service event even if that registration's queue node has not yet been materialized. The deferred worker replays those events in the same order as the eager semantics. Thus a direct service-counter update need not know a later numeric ordinal, and it must not overwrite an ahead-of-projection counter when old events are materialized.

No premature release of a pin is specified. Selection, Ready, a receipt or rejected publication does not increment service or clear it. Newly accepted DATA of the pinned task does. A pin that becomes ineligible loses appointment preference at the relevant Force selection, while its queue token keeps its age. A younger eligible incumbent is deliberately nonpreemptive; an older returning token does not silently overwrite that incumbent's fixed handoff.

### 4. Sticky retries, fresh engines and retirement

Runtime lines 94 and 104–128 require original appointment evidence and the intervening canonical proof of preservation. The shortcut is therefore not “trust the cached pin.” A maintained receiver verifies incremental transitions; a returning receiver missing that state hydrates before using the prepared-path timing claim.

The stable-pin argument at runtime lines 136–140 is valid: with no exact-task DATA discharge, the assigned next-height duty prevents acceptance from skipping to a speculative REGULAR successor. G stays at the retained anchor, and fresh-beta retries use the same old AUTH namespace. Tail registration cannot alter that pin decision. The argument explicitly stops when the pin is discharged, replaced or ineligible.

Runtime lines 98–100 retain the historical-verification/live-activation split, durable initial-entry flag, Closed gate, fresh per-pair state, six phases, complete per-session closure event and original `4 Delta` drain. The task queue supplies no authority to sign in a closed old pair. A new `(beta,r)` under an old AUTH remains a distinct engine. Ordinary r+1 DATA/DRAIN, Completion, receipt provenance and old-pair evidence duties retain their original meanings.

The alternative post-Force-completion discussion in the service-design note is not an enabled part of this runtime. It explicitly changes the fresh-action gate if adopted. Its late-arrival, exhausted-quota, Waiting and carrier-only limitations are correctly distinguished from the primary accepted-service design.

### 5. Runtime/controller join and arbitration

Runtime lines 146–152 expose a pending result when queue completion is unavailable. Controller lines 70–83 require eventual completion of each needed fixed view and prohibit signing from a later view after skipping missing registration/service deltas. These interfaces agree provided the controller's fixed-cut view demand is serviced as the stated continued work. The candidate takes that eventual-view interface as a condition; it does not derive it from bounded message delivery.

The controller's `ServingPin` and `Need` definitions agree with runtime line 148. In particular, a younger eligible pin blocks new scheduling even when an older token has become eligible again. The historical service-domain condition correctly covers that younger possible blocker; restricting the condition only to the requested old head would be insufficient.

Controller lines 148–150 and 181–214 give mandatory scheduling priority over optional H1 at each newly observed usable f credit. The credits are shared, not doubled by purpose. An actually submitted current-beta health MSIG is an equivalent opportunity because it uses the same empty configuration and public selector. A merely reserved optional operation cannot occupy the mandatory scheduling slot forever. A stale/expired operation does not refund its old credit; growing accepted DATA can provide later keys. Under the stated eventual local-work duty, optional health input therefore cannot starve reservation or dispatch.

A Force that makes a pending MSIG stale already supplies a selection opportunity. REGULAR alone cannot make it stale. This establishes the stated opportunity argument; it does not guarantee that the motivating token defeats all older eligible tokens.

### 6. Task-specific health and legacy leases

Controller lines 150 and 158–174 require every registered task of the region in a lease reservation, including parked and non-head tasks. Service and requeue are atomic, so there is no absent-token interval in which a lease can evade a task's credit. New registration during an outstanding lease inherits a spent current task bit. Multiple existing hazards remain a finite separately counted backlog rather than minting several new bits.

The finite-count proofs in Propositions 1 and 2 follow from these monotone keys. At fixed actual G, the controller can visit only its remaining finite ancestor-tip keys; f is shared by health and scheduling. For a task with final count `s*`, only keys from observed `s0` through `s*` can fund later new leases. Other rosters' DATA cannot replenish them. Unreleased reservations and uncertain/legacy operations are included in the backlog rather than discarded.

An actual new OFF requires the already finalized independent later ON, with `s_off + D < theta_off < theta_on`. Same-block threshold crossing applies OFF then ON. The legacy section correctly separates finite count from restoration: an attached legacy OFF can overwrite an independent ON in the same prelude, so it requires reconciliation and a corrective ON after the relevant canonical processing. The restoration conclusion is explicitly contingent on actual submission of that corrective ON and growing timestamps. No old voter availability follows from governance ON.

### 7. Actual submission is a separate premise

Controller lines 224–265 correctly distinguish exact reservation, signature release, local dispatch, actual interface admission and finalization. The frozen `T_L1` guarantee starts at actual valid submission. Choosing an activation threshold at an earlier signing/RPC event does not by itself establish the strict advance-announcement inequality at admission.

The unchanged-byte retry and expiry rules preserve operation identity and prevent an uncertain old record from being re-signed as a refunded retry. A released uncertain operation remains a reconciliation obligation even when active first-submission attempts must stop. An invalid late attempt is not counted as success.

Proposition 3's public-opportunity conclusion explicitly requires an actual valid submission or an equivalent already submitted record. Under the original growing-time progress premise, a finite threshold is eventually crossed while the no-intervening-Force case has constant beta. That completes the conditional argument. Local nonstarvation alone does not complete it.

An abstract model in which correct `Submit` is an enabled action may provide the needed interpretation. The candidate expressly leaves establishment of that interpretation open. I neither infer external admission from the conditional finalization bound nor posit an arbitrary unavailable external service as an attack satisfying all intended frozen progress conditions.

### 8. Timing and the unproved end-to-end boundaries

Runtime lines 166–182 impose extended complete-handler conformance explicitly. With that condition and the original regional response premises, its arithmetic is the original decomposition: opening visibility and handler, regional response, original complete-result publication, and result visibility and handler give

`L_(i,r) = 2 T_E + 2 Delta_i + R_(i,r) + T_pub`.

The first DATA is exposed before any later terminal-successor wait. The publication input remains complete original `R0`; no bare-task/request or kernel-only service is assumed. All arbitrary initial archive, native and authority preparation remains outside this uniform bound.

The residual-delay paragraph correctly withholds a fixed L when conformance is absent. An indexed list, fixed finite job or atomic descriptor does not establish constant CPU/disk cost. Increasing finite residual delays across retries can invalidate the linear real-time upper bound required by the quadratic AUTO contradiction. The text identifies this exact limitation rather than hiding it inside `Delta_i`.

Under conformance and the stated stable-tail conditions, summing startup timestamp gaps `(1+beta_n)W_start` versus bounded real opening gaps is valid using only the original upper clock-lead bound; no bounded timestamp lag is introduced. Finite controller interference and the pin-specific no-new-cooldown rule are essential to immediate stable-tail reassignment.

The service-design request argument is consequently conditional on repeated accepted service of the covered task. Same-snapshot retention/relay supplies awareness only in that service domain. Fresh report contexts and finite quota prevent a finite stock of pre-awareness reports from supplying infinitely many accepted first blocks; the original finite-key potential then applies. Canonical registration of that owner, actual serviceability of every blocking historical incumbent, and enough public scheduling opportunities remain antecedents. None is proved by the local queue or controller lemmas alone.

## Validation and limits

This was manual source-level verification of the exact frozen candidate and its joins to the original protocol. No executable simulation, model checking, TeX build, benchmark, concrete submission test or native-adapter validation was performed. No protocol source, candidate or prior review was modified. The candidate hashes remained unchanged after review.
