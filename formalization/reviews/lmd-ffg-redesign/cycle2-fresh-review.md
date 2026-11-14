# Fresh verification of cycle 2

## Summary

**Final Verdict:** The certified-prefix safety, authority, snapshot-packing, and conditional progress arguments check out under the stated synchronous and capacity assumptions; the full quantitative account is not yet valid because the epoch payload bound contains a **Critical Error**, and the candidate-validation cost has a **Justification Gap**.

Reviewed source: `formalization/build/lmd-redesign-baseline/cycle2-main.tex`, SHA-256 `4e246692d857d8269b5e6e0b195e98f67a03a30c523f30c54fbc31530685ba35`. The working manuscript had the same hash when this review began. Line references below refer to this immutable source. No earlier review files were consulted. The manuscript was not edited.

**No confirmed attack on globally certified prefix safety was found.** The two remaining findings concern resource accounting, not conflicting finality. Arbitrary overload must not be presented as an attack within an envelope that expressly assumes timely processing of all admitted prescribed traffic.

### List of Findings

1. **F1 — Critical Error in the epoch payload bound.** Location: lines 1085–1088, “An epoch may expose up to $NB_G$ distinct relative report bytes.” The final AB output vector has at most one non-bottom report per origin, but AB can fetch, validate, retain, and honestly forward several distinct valid reports for an equivocating origin before that origin outputs bottom. The stated bound is false for all report bodies exposed during an epoch. A concrete four-member execution below exposes six disjoint report bodies from one faulty origin alone. The packing claim about the final report vector at lines 614–616 and 760–777 remains valid.
2. **F2 — Justification Gap in admitted-candidate validation accounting.** Location: lines 1073–1076, the endpoint and forwarding-hop verification bounds; lines 1165–1172, “Verify the chain and complete witness” before retaining at most two semantic values. The cap of two limits retained values and honest forwards, not all expensive candidate validations. The text refers to configured admission quotas but gives no quantitative limit on direct Byzantine candidate envelopes or identifies when that limit is charged. Consequently the displayed verification bound, interpreted as all admitted verification work, is not derived. The bounds are valid for the prescribed honest-forwarded messages alone. Total admitted work needs an explicit quota parameter or a clearly restricted accounting scope.

## Checked assumptions and dependency map

The verification uses the manuscript's actual strong assumptions:

- Fixed configuration, binding canonical hashes, unforgeable transferable signatures, and persistent honest signing histories; Byzantine regional weight is strictly below one third, hence so is global Byzantine weight.
- A fixed globally responsive honest set has weight strictly greater than two thirds throughout the execution under analysis. Complete admitted objects, their needed ancestry, retrieval, and verification meet the stated delivery budgets. These are assumptions about actual concurrent traffic, not consequences of the message counts.
- A healthy eligible region contains a regional supermajority from that responsive global set and satisfies regional delivery. An eventual continuously healthy eligible region exists for the service-progress theorem.
- Epoch and slot boundaries have the stipulated alignment. A node lacking a due parent/activation pauses origination until synchronized. Honest nodes do not retrospectively produce for expired slots.
- Every nonempty honest selected report path has a nonempty fitting whole-batch prefix, as expressly required at lines 370–373. Merely allowing some small legal batch to fit would not suffice; the stated nonempty-prefix requirement must hold for the path actually selected. Eligible-tree depth is bounded, while retained history is not deleted.
- Honest nodes retain and serve admitted/endorsed/global-report data for the configuration. Consumers have the stated serving paths and bounded repair attempts, and execute the prescribed repair and ordered-delivery procedure.
- The tenure inequality is imposed. No claim about eventual inclusion or finalization of an individual preview is assumed.

Dependencies checked:

`authority + deterministic Head + cap-two authenticated broadcast -> common snapshot transition -> one certified control chain -> delivered-prefix safety and activation safety`.

`retained nonempty eligible tree + cutoff-first snapshot + honest-origin broadcast validity + zero-weight descent + no rotation during the interval -> height growth`; together with READY availability, cyclic successor selection, and minimum tenure, this gives unbounded adopted heights and subsequent finalized delivery.

The payload/accounting findings do not break either logical dependency chain when the complete admitted-traffic envelope is assumed. They do prevent using the current numerical traffic claims to establish that envelope.

## Detailed Verification Log

### 1. Producer authority, branches, and latest messages

Lines 697–712: “exactly one configured producer is authorized in each slot.” The activation time, region list, and local-term formula determine the producer uniquely. Complete signatures, parent digests, and consecutive heights authenticate the ordered bytes. A Byzantine producer can still create multiple valid branches; the lemma correctly does not infer uniqueness of a batch.

Lines 320–326: “A strictly newer slot replaces the old record” and an unusable latest message “does not resurrect an older vote.” This is a well-defined latest-message rule. Same-slot equivocation causes zero weight; observing a newer unambiguous slot can restore it. Missing data can make the same stored latest record usable after retrieval. None of these actions changes authority. Future-slot messages are separately excluded from current processing.

Lines 445–448 and 715–729: fork choice descends even when all weights are zero. For a finite eligible tree, the digest tie rule makes the choice deterministic, and a nonempty tree cannot return its root. Different node inputs can produce conflicting speculative paths, as the specification explicitly permits.

### 2. Epoch cutoff and whole-batch packing

Lines 359–374 and 558–563: “first filter the candidate tree” and only then run snapshot LMD. This order is essential and is now correct. A just-started-slot batch cannot become the selected snapshot root path and thereby displace an older reportable path. A latest HEAD after the cutoff contributes zero rather than restoring an older HEAD; the tree itself remains available, and zero-weight descent still finds a nonempty path. At activation the cutoff is zero and the first report is empty.

Lines 351–354: the signed report binds its exact parent, current global view, full relative branch, endpoint, size, and completed-slot cutoff. Old-epoch reports cannot be replayed into another snapshot. A truncated honest report endorses its actual truncated endpoint once, not an absent longer branch.

Lines 760–777: “A selected leaf is the endpoint of a reported path.” For the final common vector this is correct. A leaf of the union belongs to at least one included complete root-to-leaf report path. Consequently the selected path fits one report's limits, despite the union containing up to $Nb_G$ entries and $NB_G$ relative branch bytes before deduplication. This reasoning does not bound intermediate AB report variants; that is F1.

### 3. Capped authenticated broadcast

Lines 732–757 and 1157–1181: the two-value retained-state algorithm still provides agreement, honest-origin validity, external validity, and fixed-round termination under its delivery premise.

For each value retained by a responsive honest receiver, either its accepted chain already contains a responsive honest signer whose earlier multicast disseminated that value, or the receiver can append and disseminate it before the last round. If the receiver previously forwarded it, that earlier dissemination suffices. A length-$n$ chain necessarily contains a responsive honest signer because its signers are distinct and include all identities.

At every other responsive honest receiver, a disseminated retained value either becomes retained or encounters a set already containing two values. If one honest retained set contains two distinct values, applying this argument to both forces every honest output to bottom. If all honest retained sets have at most one value, every nonempty one contains the same value. Different two-element sets do not threaten this output agreement.

This argument does **not** imply there are only two semantic values globally for one origin. Different honest receivers may retain different pairs. It also does not limit the number of full witnesses verified before the retention test.

### 4. Global uniqueness, FFG links, and geographic authority

Lines 780–808: induction from the common trusted genesis supplies a common parent and effective activation state. Report agreement and deterministic transition supply a common target. The responsive honest global supermajority can form the adjacent link within the budget. Adjacent links cannot surround each other, and target epochs strictly increase.

Two distinct certificates at one target epoch would have an honest signer in their quorum intersection, contradicting persistent sign-once behavior. With validated immediate-parent ancestry, this also precludes conflicting certified control histories. All transitions retain or extend the adopted prefix.

Lines 632–641: activation consumes a finalized strict-ancestor ROTATE at its predetermined future boundary. It does not modify an already hashed checkpoint or depend on a proof of the checkpoint currently being constructed. ROTATE retains the parent tip covered by READY; CONFIRM retains that tip; the successor inherits it and receives a fresh view domain. Same-region restart is covered by the same argument.

READY proves possession when signed, not ongoing availability. The text correctly handles subsequent unavailability through later global epochs rather than claiming the certificate prevents a failure.

### 5. Consumer stale-state catch-up and final delivery

Lines 509–525: “Before appending, install its already-due anchor and all intervening due activations, unless a newer extending anchor is installed.” This closes the case in which a consumer learns finality for a branch while retaining an incompatible speculative suffix under an older anchor. Complete data/proof verification and anchor catch-up occur before append and further preview output. A proof whose boundary is still in the future is queued; replay cannot decrease the anchor.

Lines 816–837: delivered prefixes therefore lie on the unique certified control chain and append in order exactly once. The installed anchor contains the delivered prefix; preview replacement occurs above it. Neither a local reorganization nor a geographic handoff can withdraw delivered entries. This verifies order and authority, not execution validity.

### 6. Conditional progress, depth, and minimum tenure

Lines 868–894: under a fixed anchor and healthy active region, a responsive honest producer receives a slot within a complete schedule cycle. It either creates an extension or already has one in its eligible tree. A full depth-limited tree already contains reportable data and thus is not a liveness obstruction. Retention under a fixed anchor preserves a nonempty candidate tree.

The subsequent cutoff-first snapshot contains a completed-slot extension. Its honest active reporter supplies a non-bottom AB result with a nonempty fitting prefix. Global Head chooses some nonempty branch independently of regional convergence or vote concentration. With rotation excluded, ADVANCE follows.

Lines 905–930: if adopted heights were bounded, integer monotonicity would make them eventually constant. Then a continuously healthy region repeatedly supplies READY for the fixed prefix. Selecting the first ready region after the current region cannot cyclically bypass this permanently ready region indefinitely. On activation, its counters reset; the tenure condition permits the fixed-anchor progress interval before another replacement. This contradicts permanent height stagnation.

The theorem establishes growth of some adopted/finalized data. It does not establish persistence of any chosen batch, preview, or transaction. No stronger claim was used.

### 7. Timing and repair

Lines 958–977: the repair timeout now covers the complete GET–DATA round trip, service, bytes, and verification. The per-object $(a+1)\theta$ bound follows from sequential capped attempts. Sequential chunks incur summed bounds, as stated; it is not a one-way delay bound incorrectly used as a round-trip timeout.

The local proposal bound, $L_i+2T_G$ adoption bound, one following epoch for finalization, and separate consumer announcement/repair additions are conservative under their stated alignment and synchronization premises. The outage bound explicitly conditions on a finite late-data tail, readiness/synchronization time, and a healthy selected successor. None yields an unconditional bound for arbitrary global stalls.

### 8. F1: intermediate report bodies exceed the final-vector bound

A concrete execution satisfies the fault model and cap-two algorithm:

1. Let $N=4$ with equal weights, three honest responsive members and one Byzantine member $b$, all in the active region. The Byzantine weight is $1/4<1/3$.
2. In a completed slot for which $b$ was the authorized producer, it signs six distinct valid branches from the common anchor. Choose disjoint transaction bodies so content caching cannot identify them, with each report's relative body size arbitrarily close to $B_G$. Global report validity is explicitly independent of local candidate caps.
3. In round 1 of $b$'s report AB invocation, send two different valid origin-signed reports to each honest receiver, with the three receiver pairs disjoint. Each honest receiver verifies and retains its own two values.
4. In round 2, the three honest receivers forward their retained pairs. Thus six distinct report bodies are honestly forwarded. The complete-witness receive rule causes honest endpoints to obtain and verify the other pairs even though their retained sets are already full.
5. All honest outputs for origin $b$ are bottom, so its final-vector payload contribution is zero. Nevertheless, the execution has exposed nearly $6B_G$ distinct relative report bytes, exceeding $NB_G=4B_G$.

This needs only two initial Byzantine candidates per receiver, so even a natural cap of two on that direct sender would not restore the claimed global $NB_G$ bound.

For the semantic values actually forwarded by honest nodes, at most two per honest node per origin gives an $O(N^2B_G)$ bound on the distinct body union over all origins, and an $O(N^3B_G)$ bound on uncached endpoint acquisition across all $N$ receivers. These are accounting observations about the supplied algorithm, not changes to it. Caching can reduce the actual bytes, but cannot restore $NB_G$ in the worst case. The final report vector and selected branch retain their separate smaller bounds.

### 9. F2: candidate admission and verification work

Lines 1165–1172 place complete chain/witness validation before the retain/ignore test. A faulty origin or sender may therefore supply further valid candidate reports that are never retained and never honestly forwarded, yet still cost body retrieval and verification if admitted. The generic instruction to “enforce configured limits and admission quotas” excludes unlimited flooding conceptually but does not quantify this traffic in the displayed bounds.

For example, if an explicit quota $Q$ bounds candidate envelopes admitted for expensive processing per `(epoch, origin, authenticated logical sender, receiver)`, total endpoint candidate count is at most $O(QN^3)$, and the corresponding uncached validation term is $O(QN^3(N+\nu_G))$. The manuscript currently has no such stated parameter, charging point, or equivalent aggregate bound. Any concrete quota must also preserve admission of prescribed honest forwarding; silently cutting off all later messages after two locally retained values would not be the supplied broadcast algorithm or its proof.

The $O(N^2)$ bound on signatures generated by honest AB participants is valid. Their two-forward cap also bounds prescribed honest logical multicasts and their control bytes. The stated total-verification accounting needs to distinguish that honest-generated traffic from additional admitted Byzantine candidate traffic. This gap is confined to quantitative resource justification; it is not evidence of double finalization under the assumed envelope.
