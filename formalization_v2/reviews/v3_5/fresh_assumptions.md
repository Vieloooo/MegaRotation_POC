# Fresh v3.5 assumption and objective-scope audit

## Summary

**Final Verdict:** The bounded public-response proof contains a **Critical Error** in its use of the frozen publication guarantee. The global-progress and repeated-regional-service proofs depending on that bound are consequently unverified. Independently, the candidate explicitly leaves request inclusion unresolved and supplies a feasible starvation pattern, so it does not complete the full user objective. The new finite health policy and stable preparation rules are identifiable protocol changes, rather than an implicit accurate-failure oracle or a new uniform authority-availability bound. Native handler timing remains an explicit implementation obligation.

This is an independent verification report, not a repair. I did not read other review reports or prior review conclusions. All locations below are relative to `snapshot/`; `main` means `sync_hotstuff_rr_part2_rotation_v3_5.tex` and `liveness` means `sections/rotation_v3_5_liveness.tex`.

### List of Findings

1. **Critical Error — the exact-base publication used to derive `L` is not covered by the frozen bound.**
   - **Location:** liveness:183–193, especially “The Publish Layer and Espresso bounds add … Submit the result with the exact current base”; contrast main:405–414, “A result already recorded in Espresso satisfies this delivery guarantee; retries needed for acceptance into `G` are a separate task,” and main:2647–2656.
   - **Issue:** A current result can already be recorded in an Espresso-admissible but globally rejected EXPORT. The guarantee then supplies no deadline for a corrected EXPORT. Constancy of `G` eliminates a *race changing a correct base*; it does not turn the first recorded EXPORT into an exact-base one or give the acceptance retry a deadline. The claimed finite `L` and the use of `a[n+1]-a[n] <= L` are unsupported, and an execution with an eventually successful but later retry contradicts the intermediate bound. The dependent global and regional service conclusions are unverified by these proofs; this report does not claim that every possible proof of those conclusions is impossible.
2. **Explicit objective limitation — regional DATA service is weaker than request inclusion.**
   - **Location:** liveness:341–383, particularly “Empty DATA can discharge a regional debt” and “No full request-liveness or completed-goal claim is made here”; main:2251–2262.
   - **Issue:** The supplied positive-progress/late-recovery pattern can keep an oldest, admissible, disseminated request out of `G` while height and regional credits grow. This is not a false request theorem: the candidate candidly disclaims it. It does prevent treating this candidate as completion of useful client service under the user's full objective.
3. **Explicit model/implementation limitations — one physically live region and an arbitrary native adapter are not the proved scope.**
   - **Location:** liveness:198–203, 259–265, 387–391; `rotation_v3_5_ready_bundle.tex`:107–144; `rotation_v3_5_source_step.tex`:127–169.
   - **Issue:** The theorem requires continuing eligible regions under installed authorities and the inherited correct-weight/communication premises. It does not extend to permanently missing correct quorums in other selected snapshots. The entry-time bound also requires directly executable prepared state and a handler covered by the existing complete-message bound. The document states these limitations; silently omitting them would strengthen its conclusions.

## Frozen artifacts and dependency map

The SHA256 of the main file is the requested `2f47bca40d7ebdf80bb68a49a5fb8c479c4a50484a753d0134defc9381831d6b`. Every file listed in the review manifest matches its recorded SHA256. The main file's six active section inputs, three inline figure inputs, and two included figure PDFs are present and manifest-matched. The full source section from `\\section{Model and assumptions}` to `\\section{Regional engine: definitions}` is byte-equal to the v3.1 baseline. I also read the relevant full-baseline progress and request sections and inherited Part I authority, accountability, fairness, and progress arguments; model-section equality alone was not used as the semantic verdict.

The relevant dependency chain is:

- Immutable slot, finite roster, inherited initializer fairness and generation-indexed exclusion, eventual authority/state availability, and retained preparation → proposed finite regional response `R_i` (liveness:55–142).
- Source availability plus the complete SOURCESTEP and ReadyBundle/native-state contracts → proposed bounded entry after a public opening (source step:127–164; ready bundle:107–144).
- Those bounds plus **bounded submission of an exact-base acceptable current result** → proposed public response `L` (liveness:146–194). The emphasized premise is the failing inference identified above.
- Finite health interference, stable zero-output slot, nonempty continuing eligibility, conditional cooldown, and adaptive miss tolerance, together with `L` → quadratic-timestamp contradiction proving global growth (liveness:196–257).
- Global growth plus finite owed sets, strict cutoff, continuity, quotas, and cyclic regular selection → repeated accepted regional DATA (liveness:292–339).
- There is no final implication to request inclusion; the document provides an explicit counterexample pattern instead (liveness:347–383).

Independent control-policy, identity, and scope checks below remain meaningful even though the public-response proof fails.

## Detailed Verification Log

### 1. Frozen model and baseline progress premises

> “The regional engine keeps its original fair-schedule, exclusion and uninterrupted-commit-window conditions.” — baseline Part II:2104–2105.

The inherited Part I assumptions actually state both initializer fairness and within-generation term fairness (baseline Part I:208–223), finite availability of each authority independent of L2 commitment (173–183), and permanent *generation-indexed* exclusion of each exposed equivocator (185–205). Thus the formulas quoted at liveness:11–26 are grounded in more than a generic assertion of honesty. The baseline additionally demands turns actually entered (baseline Part II:490–493; main:521–524); the candidate explicitly retains that condition. A fair infinite schedule alone would not establish that a finite-quota engine reaches every scheduled correct term.

The original authority/state and publication clauses are eventual or conditional, not numeric end-to-end startup bounds. Baseline Part II:496–511 and 2108–2117 already distinguish successful acceptance from transport. Therefore the new argument must genuinely derive its constant response bound; it cannot borrow the older “accepted in time during some sustained service interval” condition and call that a derived result. The publication finding below does not assume an execution that can never satisfy this older condition.

### 2. Finite health policy: protocol restriction, not a new accuracy premise

> “This restricts which new controls it issues; honesty alone did not imply these restrictions.” — health:3–8.

The distinction is correct. Durable credits keyed to the observed accepted tip, persistent per-region accepted-output counts, no refund after stale targets or uncertain submission, and a monotone cursor are all explicit algorithmic obligations (health:18–45). They do not follow from the old honest-committee assumption, but can be prescribed behavior of that already-honest committee. No accuracy or timeliness of health reports is used.

For a stalled accepted frontier, the number of distinct observable tips is at most `h* - h0 + 1`; the credit proof at health:120–143 is valid given the finite prior journal. A regular handoff does not change beta or invalidate a pending MSIG; the policy explicitly preserves that fact (health:10–16). Target mismatches consume their reserved credit; no beta reset yields a new credit without accepted DATA.

The OFF lease first finalizes a later independent ON, then releases an earlier OFF only if the announcement inequality can still hold (health:60–76). Sorted independent-control application makes OFF precede ON when the same timestamp jump crosses both. Empty MSIG attached configuration is essential and is stated, because independent ON followed by a matching attached OFF in the same prelude would otherwise reverse the restoration. The candidate expressly retains legacy record semantics (health:93–118; control:189–193).

The finite-restoration result therefore holds for its stated genesis/managed-policy scope. It does not cover unreconciled legacy OFFs, arbitrary permanent administrative removal, or an unlimited emergency signing bypass. Those exclusions are explicit scope limits, not hidden benign exceptions. Administrative removal cannot be counted as successful service to the removed region.

### 3. Stable slots, immutable authority, preparation, and stale inputs

> “The first case is a retry using the same immutable authority namespace.” — main:1844–1876.

A Force after zero accepted DATA retains `r`; a Force after positive accepted DATA advances it. Both create a new beta with fresh consensus state. Pending unassigned boundaries retain the already determined target slot, so repeated Force does not manufacture successive missing slots merely by recomputing `r_b+1`. In a truly fixed-`G` tail, at most one positive-duty transition remains; the later slot and anchor are fixed. This reduction is sound.

The caches retain validated executable anchor state and unfinished work for that exact anchor, not old locks, votes or timers (main:1786–1819). The authority cursor continues over immutable recovery records across beta changes. With the inherited eventual availability of each finite authority prefix, its finite-prefix conclusion is reasonable and does not require a uniform L1 authority bound.

The finite excluded-identity set is taken within one installed finite roster and authority namespace (liveness:67–93). Part I exclusion is explicitly indexed by generation, not by elapsed wall time. Reusing a low-generation faulty assignment on another beta does not by itself contradict that inherited definition; the proposed response argument must traverse the finite low-generation prefix again. No authority may be rewritten merely because beta changed. The main file and liveness argument preserve these distinctions.

Stale material is not an activation command. Target pair and anchor identity are bound to tasks, fixed handoffs cannot be replaced, and historical evidence validity is separate from current `CanEnter` permission (main:1673–1676, 1683–1724, 1786–1795, 2000–2017). A cached state for a different anchor or a valid old-beta receipt cannot reopen the target. The phase-independent Closed gate and closure relay address delayed ReadyBundle, authority, and other old-session callbacks (source step:110–125; retirement:43–108). I found no assumption-level route by which those stale inputs legitimately reseed the current authority or bypass closure.

### 4. Source-handler and Ready timing

> “Source availability invokes the handler … not … an independently scheduled replay job.” — source step:4–10.

> “If validation, adoption and the atomic Enter transition are the complete message-processing operation covered by [the network assumption] …” — ready bundle:107–121.

The original bound already includes delivery and processing of complete validation dependencies (main:391–403; baseline Part I:159–171). A complete packet, immediate correct regional relay, deterministic ordered handler, and direct attachment of an already executable validated state are a coherent abstract-message design under that bound. No additional cross-region network delay or new public READY certificate is silently introduced. The source packet's first relay does not wait for local P6 replay or native preparation, preventing a circular wait for the selected region to be known.

This does **not** prove a performance bound for arbitrary state restoration, copying, database import, or an independent replay job. The candidate explicitly gives the residual-delay alternative `e_v <= e + Delta + u_v` and says no uniform `u_v` follows from the frozen model (ready bundle:124–144). Source step:159–169 and main:2087–2096 repeat this boundary. Therefore an abstract result conditional on that handler contract is not automatically an assumption-drift defect, but a concrete implementation cannot inherit the constant entry bound merely by naming its operation “adopt” or “handler.” Establishing that adapter contract remains outside this verification.

The record-completeness gate separates a genuinely absent canonical source dependency from a malformed candidate demanding arbitrary future material (source step:12–49). The former pauses at its canonical position; the latter is rejected without blocking the entire replay. This prevents using a malicious bare future-authority reference as an in-model unbounded dependency of an otherwise complete namespace. Missing *actual* designated authority still causes waiting, as it should.

### 5. Critical publication-bound inference

> “The Publish Layer and Espresso bounds add `T_pub + T_E`. Submit the result with the exact current base `G*` …” — liveness:183–193.

The implication does not follow from the actual service contract. The service promises an Espresso-admissible record containing the result, expressly allows an already recorded result to satisfy the promise, and separates retries for `G` acceptance (main:405–420). “Espresso-admissible” is not “accepted by this rollup”: main:475–478 and 1424–1431 explicitly distinguish them. The interpreter rejects a wrong expected base (main:1433–1438), and later availability does not retroactively accept a rejected record (main:1477–1480).

A concrete bounded-response witness uses a retry after all finite preparation and the last health intervention. Choose an already large beta and window values so the next AUTO crossing lies after `a+L+1`; this is consistent with `M=1+beta`, positive fixed base windows, and a finite prior execution. Let Espresso timestamps equal real time, so there is no clock-lead or lower-lag violation. Let the regional engine quickly form a complete current-beta result. A Byzantine participant may collect a valid receipt and record that result in an Espresso-admissible EXPORT with a deliberately wrong expected base. The DATA, receipt and dependencies are genuine; the expected-base acceptance check fails, leaving `G` unchanged.

Once that result is already recorded, the frozen publication guarantee is satisfied. The correct retry worker eventually builds/submits the corrected record, but the frozen contract assigns no constant deadline to that acceptance retry; choose its successful recording at `a+L+1`, before AUTO. All regional communication, Espresso inclusion/visibility, eventual authority/state and subsequent successful-service conditions can hold. At `a+L`, neither global growth nor an effective Force has occurred, contrary to the public-response proposition. No permanently absent correct quorum, inaccurate clock, unannounced control, or infinite refusal of all future service is needed.

The abstract contract is the relevant premise here. Main:2603–2607 introduces the immediate leader-first strategy as an implementation sketch whose service bound must be justified, rather than making a stronger acceptance-retry bound part of the frozen service assumption. Its existence is not a proof that every implementation satisfying the stated abstract contract submits the corrected record by `T_pub`.

This is a **Critical Error in the stated inference/bounded-response claim**, not merely a complaint that publication implementation details are absent. The subsequent global proof's inequality `a[n+1]-a[n] <= L` (liveness:235–250) depends directly on that invalid bound. Under the proof-verifier rule, the global-progress and repeated-service conclusions are marked unverified, rather than repaired here.

### 6. Clock and control argument after provisionally granting `L`

> “The left side is quadratic … the right side is linear.” — liveness:227–256.

Conditionally on a valid uniform prepared-tail `L`, the clock algebra uses the correct one-sided clock constraint. A zero-accepted-DATA retry stays in startup, its miss tolerance increases with beta, and fixed installed parameters have a positive minimum window length. Each closure needs at least `M_n W_n` additional Espresso timestamp. Immediate reassignment would bound each real-time opening interval by `L`. Summing gives quadratic timestamp growth against linear real-time growth, contradicting the upper-lead bound. A finite initial timestamp lag affects only an additive constant; no bounded lower lag is required.

The conditional AUTO cooldown only removes the source when another eligible owed candidate exists, so it does not create repeated fresh no-candidate delays after the fixed-prefix restrictions have settled (control:161–170). Earlier cooldowns have finite thresholds; growing timestamps eventually pass them. Missing target authority is a canonical evidence wait, not a license to reset the window or use local recovery time as its opening timestamp (control:224–266; main:2098–2108). These checks support the *conditional* structure of the clock argument, but cannot restore the missing `L` premise.

### 7. Eligibility, “one live region,” and owed service

> “A physically healthy region kept permanently OFF is not an eligible candidate.” — liveness:259–265.

This limitation is correct and material. Eligibility requires governance ON, expired cooldown, and positive weight under the designated authority (main:297–302, 1828–1842). Physical availability does not by itself establish these properties. Conversely, the inherited regional assumptions require correct quorum weight and bounded correct-to-correct communication in every regional instance to which the proposed regional response is applied. The proof cannot be extended to an arbitrarily selected dead region with a permanently absent correct quorum simply because a different region remains healthy. That would leave the frozen model. I therefore do not present such a trace as an in-model attack.

The owed set is finite, only accepted DATA discharges debt, temporary cooldown does not prune it, and only Force selection is filtered (control:101–159). The repeated-service proof properly treats regular historical handoffs separately (liveness:300–345). Its infinite-Force case depends on the first accepted DATA of a new version coming from its initially selected owed session; strict cutoff and continuity supply that premise. Its finite-Force case uses finite quotas, strictly increasing regular selection heights, and cyclic NextLive. Those combinatorial deductions are sensible conditional on continuing global growth. They remain unverified as an unconditional chain because that upstream proof currently fails.

### 8. The admitted request-starvation pattern is materially feasible

> “In each newly reached slot, a Byzantine initializer first produces one valid empty DATA that is accepted globally.” — liveness:349–377.

Take an admissible request `x` disseminated to every correct member, oldest in every correct queue. This even satisfies the stronger common-order case, so different local queue orders are not needed. A Byzantine initializer may choose a valid empty DATA despite `x`; the honest FIFO rule constrains correct leaders, not Byzantine contents beyond application validity. With quota at least two, accepting that block leaves the duty positive but unfinished. Ordinary silence can produce BLAME without equivocation.

For the supplied `Delta=1` timing, a correct recovery initializer entering at `d` can send NEWVIEW at `d+2`, obtain its ANCHOR QC and make a DATA vote at `d+4`, with the other needed correct vote at `d+5`. The commit timers then expire at `d+6` and `d+7`. A legal Espresso crossing at `d+5.5` precedes any complete FINAL receipt. Existing timers survive Force and may emit FINAL while draining (main:1133–1139; retirement:92–105), so late complete receipts and their bounded publication are consistent with the protocol. Strict pre-payload cutoff prevents their old-version DATA from entering `G`.

The earlier accepted empty DATA makes the duty's count positive, so Force advances to a new `r`, and the next slot supplies a new eventual-but-unbounded recovery-authority wait. Finite preparation of one stalled slot cannot remove this pattern because `G` and `r` keep advancing. The controller can issue no MSIG or health OFF at all. Use correct recovery initializers in each slot, rotating identities/regions as needed to satisfy actually-entered fairness. No equivocation is required, every requested authority can appear at a finite time, all correct regional messages meet their bounds, and every result offered to publication can meet its guarantee. Thus this pattern does not evade the frozen assumptions by introducing permanently missing correct members.

The result is unbounded accepted height and repeated regional DATA credits while `x` stays pending forever. The candidate openly identifies that distinction. The remaining user objective is consequently unresolved even if the public-response issue were put aside.

## Validation and scope

This was a source-level adversarial model and proof-scope audit with manifest verification and direct baseline comparison. I did not modify the snapshot, repair proofs, run model checking, certify the regional DATA/DRAIN engine, or establish a concrete native adapter's performance. Independent finite-credit/restoration and identity/eligibility checks above are not an executable whole-protocol certification.
