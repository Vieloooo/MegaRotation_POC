# Fresh adversarial verification: recurrent-identity v3.8 trace

Date: 19 October 2026. Reviewed argument: `v39_recurrent_identity_trace.md` only. Protocol authority: `manifest.json` and its frozen `snapshot/` only. The proof-verifier workflow was applied. A separate fresh sub-review checked the progress-premise quantifiers against those same inputs. No other design or review notes supplied premises for this report.

## Summary

**Final Verdict:** **valid as a conditional abstract v3.8 request-starvation counterexecution under the trace's stated fixed-snapshot communication domain and pointwise eventual native-preparation scope. No Critical Error was found in its timing, actual identity recurrence, authority use, debt transitions, or complete-result accounting.** The unformalized service prose does not establish a universal successful-attempt guarantee for every informed identity or selected snapshot. Under such a stronger interpretation the trace would violate that premise, so this is not a certification against every possible reading of that prose.

The verified outcome is precise: every born correct identity becomes an actual initializer, DATA proposer and self-voter infinitely often; every physical region contributes accepted DATA infinitely often; every original complete result is promptly published; and `x` nevertheless never occurs in an original complete regional result or in `G`. The trace does not refute v3.8's explicitly conditional accepted-report-awareness theorem. It also establishes no impossibility for other protocols or for a native adapter with stronger proved preparation behavior.

**List of Findings:** no Critical Error or missing execution step breaking the stated conditional construction was found. One scope caveat must remain attached to the verdict:

- **Location:** trace lines 7 and 120–128, especially “Enough uninterrupted physical-regional service.” **Issue:** the frozen phrases “some sustained service interval” and “working regions enough uninterrupted service time” are not formal quantifiers over identity/cohort occurrences. The trace satisfies their physical-region/existential reading and expressly admits that a stronger per-informed-cohort reading excludes it. Removing that qualification would create a **Justification Gap** in an unconditional all-readings claim; it is not a defect in the qualified claim actually audited.

The 23 manifest-listed snapshot files matched their expected SHA-256 values before review and still match afterward. The manifest and proposed trace also remained byte-identical. The complete audit is recorded below. Only this report was created; no repair or protocol edit was performed.

## Detailed Verification Log

### 1. Claim, model, and dependency map

> “The request never enters G and never occurs in a complete regional result.” — trace line 7

The argument has five dependent steps:

1. Different immutable slots may install different all-correct triples, while each session and each zero-output retry obeys its own fixed AUTH.
2. A new exact native continuation may first become ready at a finite late time; a complete ReadyBundle then receives the original bounded handler treatment.
3. The informed appearance reaches one DATA self-vote before AUTO, but no peer DATA vote. Its surviving timer therefore yields only one FINAL.
4. A fresh uninformed triple supplies the next accepted empty terminal DATA, receives `x` afterward, and remains informed forever. A second fresh success restores the owed-round invariant.
5. Dovetailing the informed appearances gives every individual infinitely many real proposer entries while never creating an accepted aware report or a complete `x`-bearing carrier.

The result is request starvation with continuing global height growth. It is neither a safety violation nor a failure of the global-DATA or repeated-physical-region-service conclusions. Those conclusions hold in this execution.

### 2. Fixed weights, authority, and infinite identity supply

> “A cohort's three members are all correct, have unit weight” and “an AUTO retry at that slot reads the same record.” — trace lines 15–21

With `W=3` and `F=1`, the required strict threshold `weight > F` is two unit signatures. Actual Byzantine weight zero is permitted by the bound `0 <= F < W/2`; the model does not require faults to occur. Membership and weights are fixed for an entire session and its recoveries, not for all future slots (`snapshot/baseline/model_and_assumptions.tex:30–44,135–150`).

The supplied authority assignment is consistent: AUTH at slot `r` names an informed A triple and a newborn B triple before the attack begins; AUTO changes beta and region but reads this same record. Later slots may name different triples and different initializers. The schedule at fixed `(r,g,t,region)` is independent of beta, as required by main lines 1834–1840. All unused recovery positions can contain correct members, so inherited schedule fairness is satisfied without manufacturing a Quit or treating an unentered recovery generation as an actual turn.

There is no frozen finite union of all future memberships: `rotation_v3_8_liveness.tex:56–59` explicitly rejects that inference. The argument needs only a finite roster at every installed slot and a finite region cycle. The all-correct choice also makes equivocation exclusion vacuous.

### 3. The informed local timeline is executable

> “e = D - 4.5” and “Every indicated peer-message delay is exactly one; self-processing is immediate.” — trace lines 35–39

The initializer is deliberately the earliest entrant. Hence its own two-unit bootstrap wait ends at `e+2`, rather than the conservative worst-case bound `e+3`. Main lines 2035–2054 set that wait from the initializer's own entry, and ReadyBundle lines 60–70 and 90–122 place other entries by `e+1` without a second import or deferred entry event.

The exact event chain checks as follows:

| Time | Checked transition | Why it is allowed |
|---|---|---|
| `e` | L enters and sends a complete ReadyBundle. | Exact native state and all Ready dependencies exist now; entry and dispatch are not delayed afterward. |
| `e+1` | Both peers enter; their STATUS reaches L by `e+2`. | One-unit complete-handler bound. The initial anchor is fixed, so L cannot select a different opening. |
| `e+2` | L sends and self-processes NEWVIEW, ANCHOR vote and report. | Its two-unit wait has ended. The report is an independent immediate trigger. |
| `e+3` | Both peers validate NEWVIEW and send ANCHOR votes and reports. | They are entered, unfrozen and unclosed. No fresh DATA timer is created by ANCHOR voting. |
| `e+4` | L holds weight greater than one for ANCHOR and reports; it proposes the sole DATA containing `x` and self-votes. | `rotation_v3_8_requests.tex:87–120`; main lines 1111–1125. The timer deadline is `e+6`. |
| `e+4.5` | The Mth startup miss closes the branch before payload. | `rotation_v3_6_control.tex:16–22,45–72`; the opening, fixed tolerance and zero accepted count give `D=o+20M`. |
| Before `e+5` | Every peer has authenticated Closed; complete-close dissemination runs. | Public visibility may be faster than its upper bound. Full closure evidence and membership are available; no unauthenticated clock guess is used. |
| `e+5` | The proposed DATA and L's DATA vote reach peers. They retain and validate it but cannot vote or start timers. | `rotation_v3_8_retirement.tex:47–102` gates fresh actions. One-unit delivery is honored despite the refusal to sign. |
| `e+6` | L's preexisting timer commits and emits FINAL. | Force is neither Quit nor Completion; main lines 1158–1164 and retirement lines 92–102 preserve this timer. |

No peer is entitled to backdate a vote to the earlier send time. Vote time is the complete-validation and first-vote event (main lines 1201–1209), which occurs only after closure in their receive handlers. Conversely, closure cannot erase L's earlier valid vote or cancel its timer.

The first BLAME deadline is local entry plus six (main lines 1224–1240). L already has one fresh DATA vote; the two peers have authentic closure before their deadlines, so the fresh-action gate suppresses their BLAME. There is no equivocation, no genuine Quit, and no receipt that could trigger Completion. Thus none of the legal timer-cancellation paths is omitted.

The delayed proposal still triggers any required complete-close reply and archival/evidence work. Retirement cannot terminate L at closure or immediately after its commit: the guard is `t >= c_v+4Δ`, plus drained timers and durable responsibility transfer (`rotation_v3_8_retirement.tex:122–160`). Keeping these owners until that guard holds is consistent with the trace and generates no additional fresh DATA vote.

### 4. One FINAL cannot be upgraded into the missing complete result

> “There is no DATA QC for this proposal and no covering FINAL receipt.” — trace line 53

Correct. Only L casts a DATA vote, so neither the DATA QC nor the FINAL receipt reaches weight greater than one. ANCHOR votes are a separate type and start no timers. FINAL binds `(beta,r,cfg_r,height,hash)` (main lines 837–864). Its omission of generation and term permits combination across recoveries of the same block; it does not permit combination across the distinct branch/session/block identities used by the recurrent appearances (main lines 880–905).

For quota one, the sole DATA is already terminal. Its QC, even if one existed, could not authorize another ordinary term. DRAIN is allowed only when a genuine recovery starts at the endpoint; the initial DATA generation does not append it (main lines 693–742). Here no recovery entry is permitted after closure. Therefore neither an unnoticed second position nor a DRAIN timer supplies a second FINAL.

The publication antecedent is a complete original result `R_0`, with direct or same-session covering FINAL evidence and its required bytes. It is not a proposal, a bare request, a single local commitment or one FINAL (`model_and_assumptions.tex:114–119,181–196`; requests lines 157–182; exports lines 201–214). The collector receives all the actual partial material but cannot sign on behalf of peers. Hence the trace withholds no result to which the bounded publication guarantee applies. It also does not use weakened kernel-only delivery to satisfy a stronger `R_0` obligation.

### 5. The newborn successful appearance does not accidentally include x

> “Deliver the client request to L at b+4.25, after the proposal and its self-vote.” — trace line 61

At the initial reports and at construction of the empty DATA at `b+4`, this triple has not received `x`. The all-EMPTY report set and empty batch are valid. The local FIFO rule cannot include bytes that have not yet arrived. When `x` arrives, it is persisted and relayed; the immutable report is not amended and the already signed proposal is not replaced (`rotation_v3_8_requests.tex:23–44,87–95,106–142`). Peers validate the historical witness, not its consistency with their later private queues.

The ordinary one-unit schedule gives peer votes by `b+5`, L's commitment at `b+6`, peer commitments by `b+7`, and a complete receipt no later than `b+8`. A peer may already assemble a receipt at `b+7` after its own commit and L's arriving FINAL; this only improves the stated upper bound. Prompt publication and acceptance by `b+9` is compatible with `T_pub=2,T_E=1`, choosing actual service delays within those bounds. Complete receipt relay and all original validation dependencies remain available.

The public opening may precede first entry by a small visibility/handler delay; the execution can choose that delay small enough that `b+9` remains strictly before `o+20`. Neither source requires the delays to attain their upper bounds. The trace's “approximately twenty” is therefore an existence choice, not an assertion that first entry resets the public clock. No selected newborn remains stuck in preparation.

With `Q=K=1`, the later request cannot occupy another position in this session. It remains pending after Completion and after any retirement; only identity occurrence in `G` removes it (requests lines 13–20,33–44). A fresh session obtains new locks and votes while preserving that node-level request state.

### 6. Same-r Force, historical selectors, and round accounting

> “AUTO at D retains r, increments β, and selects newborn B in the same AUTH[r,0].” — trace line 76

The following public invariant is consistent after the two successful seed sessions: `O={A,B}`, no surviving earlier cooldown, current informed A duty at `(beta,r)`, zero accepted DATA in that duty, and fixed AUTH already available.

1. At AUTO, `a=0` implies target slot `r`, not `r+1`; beta increments once and `g=t=0` in the new engine. The retained anchor is exactly pre-payload `G` (main lines 1810–1840; control lines 240–249).
2. `PrepareOwed(r)` prunes neither A nor B because both have positive designated weight and governance ON. B is an eligible owed alternative. Conditional AUTO therefore sets A's cooldown to `D+1` and `NextOwed(A,chi,O)=B`. No new round opens at this step (control lines 103–175).
3. B's accepted terminal DATA removes B, leaving `O={A}`. Its terminal transaction cancels the duty's window and fixes pending target `r+1` before successor resolution (exports lines 107–149).
4. B chooses a signed Espresso reference whose finalized header is after `D+1`, is at least one height beyond its opening, and is available before its DATA proposal. Thus the prescribed floor `max(height(xi(T)),q_open+1)` cannot select an earlier cooldown snapshot. Historical `NextLive` selects A under AUTH at `r+1`; it is unrestricted by debt, though A is owed here.
5. A's accepted terminal DATA removes A. At the subsequent public boundary, `PrepareOwed(r+2)` sees an empty set and refills it to `{A,B}`. Historical `NextLive(A,chi)` selects B, starting the names-exchanged next macrocycle.

This verifies the sequence `B,A,A,B,B,A,...` after seeding. The repeated adjacent accepted region names occur across a Force and a new round; they do not violate the cyclic regular selector. The source of the next informed duty is indeed the opposite region from the preceding accepted terminal DATA.

No selection uses current local status in place of its historical saved snapshot. The trace provides the later terminal reference explicitly, which is essential: an earlier reference inside A's cooldown could instead select B again. AUTH availability prevents the debt/selection transition from pausing. No terminal DATA commit is incorrectly delayed behind `PrepareOwed`.

The required handoff sends can be dispatched when material is available. Their cross-snapshot arrival is not assigned a Delta bound, and the complete public result remains timely. The chosen no-early-Enter schedule therefore needs no violation of mandatory within-snapshot Ready or source relays. If a particular complete executable handoff were received earlier, delaying its handler would be invalid; the trace does not posit that event.

### 7. Dovetail establishes actual per-identity recurrence

> “at stage k, schedule every cohort among the first k cohorts once for each of its three initializer choices.” — trace line 91

Fix a born cohort with finite birth index `n`. Each stage is finite. The seeding and repeated newborn successes provide at least the first `k` cohorts by the time stage `k` needs them. For every sufficiently late stage `k >= n`, the cohort is assigned three distinct informed appearances, one per initializer choice. Region-specific informed appearances alternate globally, so two independent finite stage schedules can be interleaved without altering the public macrocycle.

Every such appearance finishes preparation at finite `e=D-4.5`, enters, produces NEWVIEW, proposes the `x`-bearing DATA and self-votes. Thus each of the triple's identities is an actual initializer and DATA-producing leader infinitely often. The other members actually enter and report as well. This is stronger than simply assigning correct names at unentered schedule positions.

Each macrocycle has positive duration and only finitely many events; later beta raises its finite wait. The construction does not require infinitely many births, authority installations or cohort appearances before one finite time. The growing backlog of born cohorts does not invalidate dovetailing: each fixed birth index is eventually below the stage cutoff, even if that cutoff grows more slowly than the total population.

No recovery with `g>0` occurs because no Quit exists. The frozen fairness formulas quantify correct scheduled initializers and DATA leaders (`liveness.tex:22–32`), and the additional executed-turn prose is met by the real `g=0` entries. Requiring gratuitous actual recoveries in a correct execution would be another quantifier not stated there.

### 8. First native preparation: what is and is not allowed

> “The construction does not delay a complete received ReadyBundle, a validated MaterialAvailable callback, or a complete public source handler.” — trace line 99

This separation is decisive and supported by the frozen code:

- ReadyBundle lines 60–70 place first gathering, reconstruction and bundle construction before first entry and explicitly assign no fixed bound. After those tasks finish, the trace immediately enters and sends.
- ReadyBundle lines 90–122 require complete received state to be validated, adopted and entered in the modeled handler. The peers do so within one unit; no unbounded post-Ready import is introduced.
- Main lines 1674–1684 dispatch asynchronous work and enter atomically when Ready passes. The late event must be an actual completion of first preparation, not an arbitrary delay after all checks already passed.
- Source-step lines 171–176 attach an executable continuation only **when already prepared**. Correct public replay and timely full namespace delivery alone do not construct it.
- Main lines 1763–1772 retain prepared and unfinished work by exact anchor producing pair/hash, format, configuration and region. The trace keeps those caches. Every informed recurrence has a later accepted anchor; main lines 2021–2025 expressly reject using state at one DATA height as readiness for another.
- Liveness lines 44–59 and 94–105 take a finite maximum only after fixing `G*`, slot, installed snapshot and target anchor. There is no fixed-G tail here: each macrocycle advances `G` twice. The same-anchor persistent-retry argument therefore does not force these new targets to become ready early.

The compatible quantifiers are

\[
\forall a\text{ required exact target},\quad \exists T_a<\infty
\quad\text{such that its first preparation finishes by }T_a.
\]

They do not imply a single bound on `T_a - opening(a)` across future targets. For informed targets the trace chooses that finite delay as `20(1+beta)-4.5`. Increasing AUTO tolerance does not dominate an independently unbounded sequence of first-preparation delays. It would dominate repeated attempts after one fixed target has been prepared and retained, which is exactly the different case used in the source's global-progress proof.

Empty DATA is expressly valid and counts as service (Part I lines 210–212; requests lines 112–136; control lines 38–43). Empty application batches can leave application values unchanged, but the target includes the new anchor hash, position and validated continuation/provenance. The abstract interface supplies no universal constant-time conversion from every old exact target to every new one. A concrete adapter could prove that such empty steps need only bounded work; that would exclude this particular scheduling choice. Mere absence of such a proof is not an empirical performance attack. The verdict is about the explicitly eventual abstract interface, not arbitrary slowing of an implementation operation already covered by Delta.

The full raw namespaces, accepted DATA, finality evidence, AUTH and control history are delivered and retained. `R_0` native-validation inputs are not generally an already adopted, directly executable continuation at every reader; the receipt-kernel text expressly distinguishes these objects (lines 149–173). The attack cannot hide public bytes, erase caches, restart a fixed hydration target, or defer an already complete source handler. None of those is needed by its stated construction.

### 9. Frozen progress clauses, with their exact scopes

The controlling list is `snapshot/baseline/model_and_assumptions.tex:293–320` (duplicated in main lines 520–547).

| Frozen clause | Result of this audit |
|---|---|
| Correct leaders receive repeated actually entered turns; exposed equivocators are eventually excluded (297–300). | Every born identity has infinitely many real initializer/proposer/self-vote turns. All scheduled leaders are correct. There are no exposed equivocators. |
| Espresso continues retrievable finalization with growing timestamps and includes valid exports (301–302). | The chosen real-time timestamps grow without bound. Empty source blocks provide actual crossings and references. Every complete valid offered result is included promptly. |
| Complete results become available; Publish/Espresso meet their bounds; the next required DATA is accepted during some sustained interval (303–306). | Every macrocycle has two complete successful original results and timely acceptance. At each fixed next global height, a newborn session eventually supplies its next DATA. The clause does not identify every interrupted proposal as that eventual accepted result. The partial informed FINAL never activates the complete-result guarantee. |
| Required authorities and recoverable native state become available (307–308). | AUTH is already available; each selected exact native target completes in finite time. There is no all-future-target maximum in this premise. |
| A waiting boundary eventually has another eligible region (308). | At the relevant force, an eligible owed B exists immediately; subsequent regular selection references occur after cooldown expiry. No boundary is indefinitely unassigned. |
| The committee leaves working regions enough uninterrupted service time (309). | The committee issues no controls. Each physical region repeatedly receives a whole successful startup, receipt and acceptance interval. This supplies the regional/existential reading; it does not supply the stronger per-informed-cohort reading below. |
| Correct client requests reach regions that continue serving, fit, and have finite local predecessors (310–312). | Every newborn serving cohort receives and relays `x` before Completion, and every returning cohort already retains it. `x` is alone and continues to be admissible. No uniform global dissemination cutoff over all future identities is stated. |

The inherited regional window language does not silently undo the retirement gate. Main lines 1358–1362 explicitly restrict commit-window progress statements to sessions with no RequestClose during the interval; closing sessions may retain a local commitment without a complete receipt. Main lines 1405–1409 state eventual window acquisition while the engine continues to run. Baseline Part II lines 2104–2116 require timely startup **for some serving interval**, and lines 2200–2205 distinguish transport from that end-to-end interval. Here the newborn intervals witness it, while the informed intervals are closed.

A precise stronger condition that excludes this trace would couple the quantifiers, for example:

\[
\forall C\text{ that retains }x\text{ and is selected infinitely often},\quad
\exists k:\ C\text{'s selected appearance }k
\text{ has a complete post-Ready result window}.
\]

One may specify the window concretely as no closure through `e_k+9Δ` under the correct-bootstrap conditions. A still stronger version quantifies over every selected snapshot. The trace violates both: every informed occurrence closes at `e_k+4.5Δ`. Infinite actual entry only provides `forall C, infinitely many entries`; pointwise native availability provides `forall target, exists finite completion`; successful physical-region service provides `forall region, infinitely many successes`. None exchanges into `forall informed C, exists a successful aware occurrence`.

The exact numerical window is a sufficient formulation, not a newly proposed repair. Requiring a complete result or successful acceptance directly would also exclude the execution, but importing that condition would change what is being tested. Because the frozen service prose is not formalized at this quantifier level, the appropriate finding is a stated scope boundary, not intuitive acceptance of every strong reading or intuitive rejection based on “honest leaders must eventually succeed.”

### 10. Request conclusion and limits

> “the v3.8 conditional awareness theorem is not contradicted” — trace line 132

Correct. Every accepted term-first DATA uses an all-EMPTY report witness created before its newborn cohort receives `x`. Every aware recurrent witness selects `x`, but that DATA has no complete receipt and is never accepted. Therefore the antecedent of `rotation_v3_8_requests.tex:243–271`—infinitely many **accepted** term-first blocks with a genuinely aware correct signer—is false. The finite-predecessor proof itself is unaffected.

The missing carrier is real for the current v3.8 execution: original Publish/Espresso service has no `x`-bearing complete input to transport, even though all actual holders retain `x` and repeatedly propose it. Repeated accepted physical-region service, repeated individual participation, and full source availability coexist with starvation.

This supports the stated current-protocol boundary only. It does not quantify over all protocols, all authority policies, all native adapters, or algorithms that establish a different first-carrier property. No universal impossibility or concrete implementation exploit has been proved.

## Validation and hash audit

Validation was a manual source-level adversarial trace review, including a fresh independent progress-quantifier sub-review. No executable model checker, native adapter experiment, benchmark, or TeX compilation was run. Those tools would not be substitutes for resolving the prose quantifier boundary.

All paths below are relative to this review directory. Snapshot before-values were verified against the frozen manifest before analysis; after-values were recomputed after writing the report. **25/25 audit inputs unchanged; 23/23 snapshot files match the manifest.**

| Input | Before SHA-256 | After SHA-256 |
|---|---|---|
| `manifest.json` | `2baf4e209aa94e4d13789dfffe04784f01d9e24e2e2c3e3e910b00b97ad0d1f0` | `2baf4e209aa94e4d13789dfffe04784f01d9e24e2e2c3e3e910b00b97ad0d1f0` |
| `v39_recurrent_identity_trace.md` | `a47bd3e59f93015f92cbb5e09e3d3d8bf18afcc084cdef718ffe42aadd805275` | `a47bd3e59f93015f92cbb5e09e3d3d8bf18afcc084cdef718ffe42aadd805275` |
| `snapshot/baseline/manifest.json` | `d95999e9e95a9b958cf710de39b0171ae333b99c35de90c83bfc6fc4e433ca2b` | `d95999e9e95a9b958cf710de39b0171ae333b99c35de90c83bfc6fc4e433ca2b` |
| `snapshot/baseline/model_and_assumptions.tex` | `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387` | `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387` |
| `snapshot/baseline/sync_hotstuff_rr_part1_core.tex` | `00533f01567a90cd034e6c3869dfa561b7c5cf476312fc0ce8313b7060f654d3` | `00533f01567a90cd034e6c3869dfa561b7c5cf476312fc0ce8313b7060f654d3` |
| `snapshot/baseline/sync_hotstuff_rr_part2_rotation_v3_1.tex` | `483bdfbf3668a0f892af38df264655e54e0dbd73a1c26fbd45111d1730cec034` | `483bdfbf3668a0f892af38df264655e54e0dbd73a1c26fbd45111d1730cec034` |
| `snapshot/baseline/sync_hotstuff_rr_part2_rotation_v3_1_zh.md` | `7dbd6035096fabf06f200f25183312c7708bbeab1cfade97fe560eda5184ac40` | `7dbd6035096fabf06f200f25183312c7708bbeab1cfade97fe560eda5184ac40` |
| `snapshot/figures/cross_region_rotation_v3_2/modules.pdf` | `cc16b85e71089d597c6b970b27a1a8c9ee8e80c03d34a39e0f7392ce85dba62d` | `cc16b85e71089d597c6b970b27a1a8c9ee8e80c03d34a39e0f7392ce85dba62d` |
| `snapshot/figures/cross_region_rotation_v3_2/modules.tex` | `5aea9ecc96c9d63b23e2df49c38cdb73bead660f7fd69de1f82c99428fb4a6a5` | `5aea9ecc96c9d63b23e2df49c38cdb73bead660f7fd69de1f82c99428fb4a6a5` |
| `snapshot/figures/cross_region_rotation_v3_5/candidate_ready.pdf` | `9bed3112ec9c77d9c6924241ec89773d6966c4662d251badf50f027b72165b7e` | `9bed3112ec9c77d9c6924241ec89773d6966c4662d251badf50f027b72165b7e` |
| `snapshot/figures/cross_region_rotation_v3_5/candidate_ready.tex` | `c59a0ebda4c8e2f0190186174f7399d3eae53072eebb6cf11d83f49ee0225b16` | `c59a0ebda4c8e2f0190186174f7399d3eae53072eebb6cf11d83f49ee0225b16` |
| `snapshot/figures/sync_hotstuff_rr_rotation_v3_1/regular-handoff.tex` | `2d81433fcb0e5509d4ac3e4912ed85f5af78088c4983bc4b880f7fc7dbf63094` | `2d81433fcb0e5509d4ac3e4912ed85f5af78088c4983bc4b880f7fc7dbf63094` |
| `snapshot/figures/sync_hotstuff_rr_rotation_v3_5/forced-handoff.tex` | `53b6370496f548aabd1286b7c371b686143050ebd72e062b76269928cb64277b` | `53b6370496f548aabd1286b7c371b686143050ebd72e062b76269928cb64277b` |
| `snapshot/figures/sync_hotstuff_rr_rotation_v3_6/observer-finality.tex` | `2707d6823e7a865d47caf10bb949122a17ff122bd1dff2a00e3c78ea8f30a000` | `2707d6823e7a865d47caf10bb949122a17ff122bd1dff2a00e3c78ea8f30a000` |
| `snapshot/sections/rotation_v3_5_health.tex` | `21ce547d20c30e42a61bd7c58a5d45e51b5fa2aeeb71bc84a1a995f93155e12b` | `21ce547d20c30e42a61bd7c58a5d45e51b5fa2aeeb71bc84a1a995f93155e12b` |
| `snapshot/sections/rotation_v3_5_ready_bundle.tex` | `4048a979add0bfe1662af309d5b30f7e72b105e8d5b6fda80de415ba43330152` | `4048a979add0bfe1662af309d5b30f7e72b105e8d5b6fda80de415ba43330152` |
| `snapshot/sections/rotation_v3_6_control.tex` | `37e7c065b33cead1f334878f9fe92e761472464e5a67292947cd9a4f251d2285` | `37e7c065b33cead1f334878f9fe92e761472464e5a67292947cd9a4f251d2285` |
| `snapshot/sections/rotation_v3_7_source_step.tex` | `5291ae3a91e4cfff498950b25a0a8711e1fdd65423b6d2e855e575d7ae445f50` | `5291ae3a91e4cfff498950b25a0a8711e1fdd65423b6d2e855e575d7ae445f50` |
| `snapshot/sections/rotation_v3_8_carrier.tex` | `cb2de3e19a5c69d3a42ef1341eadd533336a3eeb75192813aff015483ed10c72` | `cb2de3e19a5c69d3a42ef1341eadd533336a3eeb75192813aff015483ed10c72` |
| `snapshot/sections/rotation_v3_8_exports.tex` | `06e835d96d595484a4cab19bf79b640a9412cd6c0e29afaea55518cfadaac5b2` | `06e835d96d595484a4cab19bf79b640a9412cd6c0e29afaea55518cfadaac5b2` |
| `snapshot/sections/rotation_v3_8_liveness.tex` | `480cea02b3499f98d9e2b4b9b8eaaa2afd17ec55409c901b465b31758ff98541` | `480cea02b3499f98d9e2b4b9b8eaaa2afd17ec55409c901b465b31758ff98541` |
| `snapshot/sections/rotation_v3_8_receipt_kernel.tex` | `2ff7f8e190ec5f67a3684cd29848a1bddefd7c22f2cbe41dac322924d98b841a` | `2ff7f8e190ec5f67a3684cd29848a1bddefd7c22f2cbe41dac322924d98b841a` |
| `snapshot/sections/rotation_v3_8_requests.tex` | `dd588900506e2ef73e89a20eb57588976c2e8a9875f6258e268a87531bf98150` | `dd588900506e2ef73e89a20eb57588976c2e8a9875f6258e268a87531bf98150` |
| `snapshot/sections/rotation_v3_8_retirement.tex` | `984482c763cce1a9f324e416e78d1708f553acbe4a1798f7cafdc46ef289c238` | `984482c763cce1a9f324e416e78d1708f553acbe4a1798f7cafdc46ef289c238` |
| `snapshot/sync_hotstuff_rr_part2_rotation_v3_8.tex` | `c8611baf07c74f34512a182b0b3ace81d6f8f3cf65fb0cb567af696ec8a73574` | `c8611baf07c74f34512a182b0b3ace81d6f8f3cf65fb0cb567af696ec8a73574` |
