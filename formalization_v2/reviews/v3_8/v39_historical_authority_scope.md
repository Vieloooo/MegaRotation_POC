# Independent scope audit: fresh sessions under historical authority

Date: 2026-10-19. This is a source and assumption audit, not an implementation or a whole-protocol verification. No other current review reports were read. The frozen model was compared with the Model section of the current v3.8 main source and is byte-identical.

## Summary

**Final verdict: conditionally admissible; not established by historical AUTH validity and current physical-region eligibility alone.** A fresh `(beta_new, r_old)` is not a reopening of the closed old pair. The frozen authority and per-pair quota definitions permit that kind of identity separation. There is no identified cryptographic or quota impossibility specific to reusing an old authority. Current v3.8's deterministic target-slot rule nevertheless does not select an arbitrary historical index: a new selection rule and its proof would be a protocol change.

The environmental issue has two distinct readings, which must not be conflated:

1. If the original fixed-snapshot correct-member and communication assumptions continue to cover the historical roster whenever it is addressed—including its persistent owners after worker retirement—then retaining a receive handler and invoking the existing fresh-session Ready/Enter code is an admissible protocol obligation. The request and ReadyBundle messages can stay entirely inside that historical snapshot. No cross-snapshot transport premise or new trusted AUTH issuer is needed merely for this operation.
2. If those assumptions cover only currently serving snapshots, and historical voters may permanently cease signing/participating after their old duties have been transferred, old AUTH plus ON/positive weight does not restore their service. A liveness proof relying on them would need historical-roster availability, or an explicit restriction to historical rosters still satisfying the original serving premises.

The source does **not** justify choosing the second reading as an unconditional frozen-model fact: its network clause is unqualified, and current v3.8 already invokes old-snapshot receive/relay obligations after termination. Conversely, the current conditional liveness theorem does **not** establish the first reading for every historical roster: it explicitly assumes online installed cohorts and excludes permanently missing correct quorums. Thus this audit does not rule out the independent strict-domain route. It identifies exactly what must be established for that route to inherit, rather than enlarge, the original environment.

Findings:

- **Justification Gap:** “valid historical AUTH + current region eligibility implies a serving historical quorum.” Eligibility has no availability test.
- **Conditional admissibility:** fresh-pair reactivation can be code under lifetime fixed-snapshot correctness and synchrony; code alone cannot make a departed/offline endpoint available under an active-snapshot-only interpretation.
- **Protocol incompatibility with current v3.8, not an impossibility:** arbitrary historical selection differs from the current zero-output/current-slot versus positive-output/next-slot rule.
- **Unverified downstream work:** historical selection recurrence, unchanged health-control semantics, progress after changing the stable-slot rule, and full request inclusion still need proofs. This audit certifies none of them.

## Source map

All line references below are to current local sources:

- **M:** `revisions/v3_1_frozen/model_and_assumptions.tex`.
- **P:** `sync_hotstuff_rr_part2_rotation_v3_8.tex`.
- **R:** `sections/rotation_v3_8_retirement.tex`.
- **H:** `sections/rotation_v3_5_health.tex`.
- **L:** `sections/rotation_v3_8_liveness.tex`.
- **Q:** `sections/rotation_v3_8_requests.tex`.
- **B:** `sections/rotation_v3_5_ready_bundle.tex`.
- **C:** `sections/rotation_v3_6_control.tex`.
- **I:** `revisions/v3_1_frozen/sync_hotstuff_rr_part1_core.tex`.

Paths are relative to `formalization_v2/`. The main file embeds M unchanged at P:228–548. Its included retirement, health and liveness fragments are the current files above.

## Detailed verification log

### 1. Authority establishes an immutable historical voting domain

M:30–44 says AUTH fixes the members, weights, fault bound, delay bound, schedule and quota; later recovery authorities change only the schedule. In particular:

> “A new branch version that reuses r uses the same authority at that index. Its pair (beta,r) identifies a different engine.”

This explicitly supports a fresh pair using an unchanged old authority. It does not authorize substituting current members for the old roster, changing old weights, resetting the old pair, or rebinding an old AUTH record.

M:73–78 defines eligibility solely by governance ON, expired cooldown and positive weight under the designated authority. P:1792–1797 repeats that selector calls use the designated target authority. Therefore a current physical region's roster is not interchangeable with the roster in `AUTH[r_old,0]`; even eligibility must be evaluated using the selected old authority. Neither version of this predicate asks whether any particular historical member is online or able to sign.

H:85–88 is explicit:

> “The lease's ON withdraws temporary exclusion; it does not certify recovery or future synchrony.”

H:171–177 likewise withholds receipt-formation, useful-service and request-acceptance conclusions. An ON event cannot be the missing historical-quorum premise.

### 2. Historical fault protection is not, by itself, a future response guarantee

M:137–150 gives Byzantine weight at most `F < W/2`, a certificate threshold strictly greater than `F`, and correct weight at least `W-F > F`. Its lifetime provision counts every key compromised while certificates from that snapshot remain accepted. This is a restriction on compromise of historical signing keys; it prevents treating old certificates as safe after arbitrarily many old-key corruptions.

That clause alone does not say that every uncorrupted historical key will remain available for future signatures. An uncompromised key stored offline does not become a forged key. If “correct” is interpreted as a permanent, available protocol participant, availability instead comes from that correctness interpretation together with M:167–179, not from certificate validity alone.

The distinction matters for counterexamples. One cannot assume a frozen roster's correct members permanently stop responding and simultaneously claim to satisfy an unqualified original obligation to deliver and process every correct-to-correct message within its `Delta`. If nonresponse is treated as Byzantine behavior of that still-covered member, its weight must be charged to `F`. With at most `F` unresponsive/Byzantine weight, the remaining correct weight is still sufficient. A purported example hiding more missing voting weight outside the fault budget has changed the model domain.

Only correct members need to follow reactivation code. Requiring **all** historical voters, including Byzantine ones, to respond is unnecessary and not supported. The existing BFT proof uses the correct cohort with weight `W-F > F`, its complete-message processing, and its leader/recovery conditions; it does not require Byzantine replies. Merely identifying some responsive weight `>F` is also not automatically enough to reuse every existing proof, since the source's entry and protecting-QC arguments explicitly quantify over the modeled correct members.

### 3. The communication clause does not expressly end at retirement

M:172–179 says:

> “Within region i, the actual delay of a correct-to-correct message is delta <= Delta_i”

and includes processing complete validation dependencies. It contains no “only while Active,” “only under the newest AUTH,” or retirement expiry. Part I is even more explicitly a fixed-membership model: I:153–170 has fixed voters and a fixed Byzantine set with the same correct-message premise. Part II allows session snapshots to change, so the scope over successive snapshots must be resolved from Part II; Part I cannot by itself decide that scope.

Under the strict **fixed-snapshot** reading used by Q:38–44 and L:480–486, old-to-old messages stay in the old snapshot's domain. A reactivation packet sent by an old correct owner to the same old roster does not create the old-only-to-new-only edge that the v3.8 request discussion lacks. Narrowing the domain to same-snapshot communication is therefore not itself an objection to this route.

An additional **temporal** restriction—same-snapshot service stops after the old session's active period—is a different claim. It is not stated in M:167–179 and must not be silently inserted as part of “strict domain.”

### 4. Current retirement preserves owners, but does not reactivate BFT signing

Q:23–44 requires complete first-receipt request relay to every correct member of each installed snapshot containing the sender. The owner persists through Completed and Terminated; the relay uses that snapshot's existing `Delta` bound. Consequently, if a still-pending original correct owner received `x` in such a snapshot and the stated relay premise applies, awareness is not confined to that one owner: its dispatched full request reaches the old correct membership. Q:13–15 removes a request only on its occurrence in `G`.

R:15–21, 43–91 and 112–120 retain historical manager/control records and mandatory receive handlers independently of worker existence. R:140–159 requires accepted durable ownership and receive routing before a worker releases its responsibilities. A mere unresolved retrieval task is not a handoff. The old owners may relay existing signatures and serve data, but R:146–152 expressly prohibits restarting **that old engine**, its timers, or fresh signatures **for that old pair**.

These clauses prove two limited facts:

- A correct original owner cannot discard a still-pending request or silently abandon a required relay just because its BFT worker exits.
- The current protocol has no rule that turns those archival/data duties into a fresh historical-authority BFT service. Such a rule would be added code.

The retained owner may be a local service, and certain duties can be transferred. A data owner holding `x` is not automatically the original voting identity with a usable signing key. Conversely, if a protocol explicitly requires that original correct identity to retain its key and manager and the environment continues covering that identity, retaining this capability is a protocol-state obligation. It need not create a new trusted service.

R:173–174 and P:2166–2168 say termination does not authorize stake withdrawal. Therefore “FinishedClose, hence legally withdrew stake and disappeared” is not a source-supported counterexample. The specification supplies no withdrawal protocol from which to derive either release of the historical voting duty or compulsory perpetual BFT service. Physical process exit, stake withdrawal, archival data service and fresh voting are separate questions.

### 5. Espresso archive recoverability neither summons nor forbids old voters

M:220–234 explicitly requires accepted DATA to remain recoverable without the original producing region staying online. It permits the **archive service** to survive that producer's absence and puts no constant-time bound on a returning node's catch-up. M:238–249 gives bounded Espresso visibility only to correct participants that stay online and keep up; M:210–214 similarly qualifies L1 control visibility by online participation.

This is evidence that historical-data validity/retrievability is independent of live historical voting. It is not a theorem that the original region will return. It is also not a universal exception allowing a still-covered correct historical replica to violate an outstanding complete-message obligation. A producer-offline execution may satisfy archive safety while falling outside the premises of a regional progress theorem.

Thus the tempting counterexample “all old voters go offline, Espresso archives remain, new roster progresses” is a valid separation of archive recoverability from historical availability. It is **not** an unconditional counterexample to a protocol whose original fixed-snapshot correct-message domain still includes those voters and whose receive owners remain obligated. Such a counterexample would need the active-snapshot-only interpretation as an explicit scope premise.

### 6. Exact conditional admissibility argument

Fix an old authority `A = AUTH[r_old,0]` and a selected region with old roster `V`, total weight `W`, and fault bound `F`. Suppose:

1. The original fault/correctness and complete-message assumptions apply to this historical roster when the fresh session is activated. Its correct identities retain their required state, keys and receive path. These are the same identities, not replacement members authenticated by a new issuer.
2. The canonical interpreter deterministically creates a genuinely fresh `(beta_new,r_old)` from current pre-payload `G`; the original force cutoff, matching MSIG/AUTO cause, applicable eligibility/owed-state checks, and source evidence are preserved under the revised selector.
3. At least one correct old member eventually obtains the complete new opening context and verified executable state. The frozen eventual authority/state-preparation conditions and existing online source services cover that task; no fixed catch-up deadline is inferred.
4. Correct old members run a manager handler for this new pair, retain the original fresh-action and closure gates, and perform the complete ReadyBundle handler within the original complete-message contract. The required recovery authorities/schedules and the original fair/exclusion/entered-turn conditions cover this old `r` namespace when reused.

Then its first correct Ready/Enter sends a complete bundle to old correct recipients in the same snapshot. B:107–122 gives their entry by `e+Delta` while activation remains valid. Correct weight is `W-F>F`; the existing ANCHOR, DATA/REPORT and FINAL paths can use the old authority without any new signature domain or cross-snapshot request edge. Fresh pair identity prevents importing old locks, old Quit proofs or old votes. If the old cohort already knows a still-admissible pending `x`, its new reports use that retained knowledge under Q's original parent-specific checks.

This is an admissibility argument, not a complete unbounded request-service proof. Selecting the old roster once, providing a schedule name, and making a future MSIG possible do not establish infinitely many productive informed witnesses. The selector/control policy must still supply the service recurrence actually needed by the request theorem.

Premise 1 is the exact fork in interpretation. Under lifetime fixed-snapshot correctness and synchrony, it is inherited and adding the handler is code. Under an active-snapshot-only environment, it is additional historical service coverage. Merely writing “every old voter shall retain a handler forever” cannot resolve which endpoints the environment keeps available, especially if a permitted departed participant is outside that environment's current correct cohort.

### 7. Existing liveness results do not settle that fork for all history

L:56–59 takes finite sets only from the installed configuration and expressly avoids a union over all future memberships. L:164–176 bounds response for “continuously online, hydrated members of the installed regional snapshots.” L:295–302 requires installed regions “able to serve under those premises.” L:358–364 explicitly withholds extension to permanently missing correct quorums; L:393–397 requires that a serviced region continue satisfying the regional serving premises.

These conditional statements can be applied to an old roster only after showing it satisfies those same premises. Choosing its old AUTH does not prove that fact. Conversely, if the original model already makes that roster a continuing correct synchronous participant set, no new cross-snapshot premise is needed to check this condition.

The inherited “correct turns that are actually entered” condition (M:297–300; L:17–32) must not be used circularly as “therefore this selected historical roster returns.” Its identity-level scope is itself material; L:476–478 explicitly leaves that meaning unresolved for permanent identity replacement. A future proof must state whether it covers those historical identities and show the new code/selection actually realizes the relevant entries.

### 8. Fresh historical sessions: actual authorization and safety boundaries

There is an immediate **current-code authorization obstacle**: P:1808–1840 and C:240–249 require a Force to target the current `r` on zero accepted output, `r+1` after positive output, or the already stored target while unassigned. P:1941–1946 verifies that precise derivation. An arbitrary older `r_old` is rejected unless it happens to be that prescribed target. The frozen AUTH permission does not by itself override this rule. A proposed v3.9 selector must change the rule consistently; an MSIG is currently a target-version/cause record, not a new AUTH issuer or discretionary retained-anchor selector (C:177–183; M:199–205).

There is no identified independent obstacle from the following unchanged mechanisms:

- **Closed:** R:10–11 and P:1968–1979 block the old beta. A genuinely new current open beta has a distinct pair; the old `Closed[beta_old]` is not `Closed[beta_new]`.
- **Quota:** P:695–704 defines `H[beta,r] = h(A[beta,r,0,0]) + TB[r]`. Thus a fresh pair anchored at current `G` has endpoint `h(G)+TB[r_old]`. This does not reset the old session's endpoint or change `TB[r_old]`.
- **Authority and schedules:** retain exactly the old snapshot, weights, `F`, `Delta`, `K`, quota, window parameters and per-`(r,g)` schedules. A new beta does not reseed them (M:30–44; L:34–42).
- **Provenance and append safety:** retain `G`'s actual producing pair/hash/evidence; do not relabel it. Every new DATA must extend exact `G`, satisfy the unique new boundary, and use the current open beta (P:1941–1965; `sections/rotation_v3_8_exports.tex`:78–90).
- **Namespace isolation:** old QCs/locks/entry times/quit proofs are not new-pair state. P:2536–2568 proves this kind of per-pair isolation under a unique authenticated handoff.

Returning to an older numeric slot also means later REGULAR successors may revisit other old authority indices under the new beta. That is not an immediate pair collision, because beta is fresh; the revised protocol must nevertheless check all successor, debt, selection and stable-retry invariants. The current stalled-frontier proof depends on the zero-output rule fixing one `r*` (L:44–59, 305–311). An arbitrary historical selector cannot simply cite that proof unchanged.

## Scope of conclusion

The proposed route is a real candidate under a lifetime fixed-snapshot interpretation of the original correct-member/service assumptions. The exact source does not support declaring it impossible merely because the authority is old or the old pair is Closed. It also does not support an unconditional request-liveness claim from historical certificate validity, current physical-region eligibility, or one persistent request owner alone.

The decisive proof obligation is temporal service coverage of the old **voting identities**, not authentication of their old roster and not transfer of `x` to a disjoint new roster. If that coverage is already part of the original model, the fresh-session handler can exploit it without a new environmental service. If it is not, the route must be conditional or explicitly strengthen that coverage. No new issuer, executable/model-checking validation, or whole-protocol safety claim is assumed here.
