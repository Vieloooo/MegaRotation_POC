# v3.2 preflight: frozen-premise liveness scope

Date: 2026-10-18. This is a read-only analysis of the v3.1 protocol and the v3.2 outline, not the later fresh adversarial audit of a completed v3.2 draft. No protocol draft was edited.

Sources below use `II:Lx-Ly` for `formalization_v2/sync_hotstuff_rr_part2_rotation_v3_1.tex`, `I:Lx-Ly` for `formalization_v2/sync_hotstuff_rr_part1_core.tex`, and `O:Lx-Ly` for `formalization_v2/sync_hotstuff_rr_part2_v3_2_revision_outline_zh.md`. Line numbers refer to the following inspected source hashes:

| Source | SHA-256 |
|---|---|
| Part II v3.1 | `483bdfbf3668a0f892af38df264655e54e0dbd73a1c26fbd45111d1730cec034` |
| Part I | `00533f01567a90cd034e6c3869dfa561b7c5cf476312fc0ce8313b7060f654d3` |
| v3.2 outline | `3e7d6b10bbb9baa07faecf3e978c6f235eb64e30534eb330c257ba8beccb514a` |

## 1. Verdict and scope

The full frozen v3.1 premises support conditional prefix safety and several local/eventual progress statements. They do **not** establish either (a) unbounded global DATA-height growth from one finite successful service interval, or (b) service for every continuously healthy and eligible region. There are concrete v3.1 executions for both gaps below. The current text largely acknowledges the global-liveness limitation rather than asserting a false global theorem; “Critical Error” below means an obstruction to the stronger requested v3.2 goal if the affected rule is retained.

The familiar singleton deadlock is a genuine mismatch with the requested one-live-region goal, but it is **not** a counterexample satisfying all frozen progress premises: `II:L500-L502` expressly assumes that a waiting boundary eventually has **another** eligible region. Similarly, permanently missing authority, lost globally accepted DATA, a Byzantine multisig, or an arbitrary asynchronous correct regional replica must not be counted as in-model attacks.

The two cleanest in-model obstructions are:

1. A finite good service prefix followed by automatic replacement before every new session can produce its first DATA. This is possible with entirely honest regions, a normal Espresso clock, immediately available state and authority, and bounded raw publication. See C1.
2. A/B keep a fixed historical selection reference under which C was OFF. C subsequently remains healthy and ON forever, but A/B continue serving and C is never selected. All accepted-output service can work indefinitely. See C2.

“Live” needs a definition. Governance ON, eligibility, the availability of a state image, and a region satisfying its synchronous BFT assumptions are distinct predicates. The existing model bounds Byzantine/faulty voting weight in **each** region, not only one region (`II:L328-L346`). Treating all other regions as entirely crashed or Byzantine would change the frozen regional model unless a separate abstraction explicitly permits their temporary nonparticipation. An OFF flag alone is not evidence of such a failure and does not change historical voters.

## 2. Complete frozen-premise inventory

This inventory deliberately includes substantive prose outside `assumption` environments.

| Premise | Exact source and operative words | What it does not supply |
|---|---|---|
| Immutable authority and fixed session configuration | `II:L223-L237`: “at most one immutable authority”; fixed membership, weights, `F_i`, `Delta_i`, `K`, quota, positive window lengths/targets, and cooldown; “Nodes wait for required authority evidence.” | No numeric upper bound on initial/recovery authority availability is given here; no lower bound relates the positive public windows to bootstrap time. |
| Regional faults and evidence | `II:L330-L346`: `F_i<W_i/2`; Byzantine weight at most `F_i`; each certificate needs weight `>F_i`; correct state/data retention, unforgeability, binding hashes. | Does not permit an adversary to switch off an honest quorum or corrupt arbitrarily many retired keys while old certificates remain accepted. |
| Complete finalized L1 control history | `II:L352-L357`: common, authenticated, immutable records; “a complete control prefix, not an arbitrary subset.” | A valid proof of one event cannot establish absence of other controls. |
| Regional synchrony and clock | `II:L360-L372`: correct-to-correct delay at most `Delta_i`; “All nodes share a global clock”; bound includes delivery and processing of complete validation dependencies. | Does not directly bound cross-region state restoration before Ready or catch-up with an arbitrarily long history. |
| Bounded publication | `II:L374-L389`: a complete result available at real time `s` is submitted as an Espresso-admissible complete record by `s+T_pub`; working path and fees exist. Crucially: “retries needed for acceptance into G are a separate task.” | Does not bound the time to form a receipt, restore state, or win a sequence of stale-base retries. |
| Honest multisig | `II:L392-L399`: “one honest, coordinated control service”; only authorized immutable future-dated controls; required advance announcement; no old-anchor choice. | No formal health-selection policy or per-region scheduling fairness is stated. The committee is not an adversarial source that may violate these conditions. |
| L1 control service | `II:L401-L408`: an actually submitted valid control is finalized, delivered, retrieved, and verifiable to every correct online participant by `s+T_L1`. | The text scopes this bound to control operations; do not silently turn it into a bound for every newly needed `AUTH` or native state. |
| Espresso history, retention, clock lead | `II:L411-L427`: compatible finalized histories, complete payload or genuine absence, retrievable replay evidence; accepted DATA recoverable without its original region; timestamps nondecreasing and `theta_q <= t_E(q)+epsilon_E`. | No lower clock-lag bound, no maximum timestamp jump, no fixed catch-up bound. |
| Espresso inclusion and visibility | `II:L429-L447`: each admissible complete submitted record included and verifiable within `T_E`, including visibility of empty relevant namespaces. “It does not promise that an EXPORT passes this chain's acceptance rules.” | Inclusion is not accepted-chain progress. |
| Advance announcement and verification interfaces | `II:L465-L484`: `theta_a > s+T_L1+epsilon_E+eta`; missing control history pauses; regional receipt and application checks “are assumed sound”; sound execution and DA correspondence checks also required. | A cryptographic/application-invalid receipt attack is outside these interface premises unless the task is explicitly to justify their implementation. |
| Entered correct-leader fairness and exclusion | `II:L490-L493`: “Correct leaders receive repeated turns that are actually entered,” including recovery initializers and DATA-producing turns across sessions; exposed equivocators permanently excluded eventually. | Does not say every eligible region is selected. Fair schedules on unused views are insufficient. |
| Ongoing Espresso service | `II:L494-L495`: retrievable history with “growing timestamps” and inclusion of this chain's valid exports. | Unbounded timestamp growth gives eventual crossing, not a fixed real-time crossing bound or acceptance of closed-version data. |
| Complete results and a successful service interval | `II:L496-L499`: bounds hold; “Retried EXPORTs for the next required DATA are accepted in time during **some sustained service interval**.” | The quantifier is existential. It does not say every later interval, infinitely many intervals, or an eventually permanent interval. |
| Authority, state, successor, committee service | `II:L500-L502`: required authorities/native state become available; “A waiting boundary eventually has another eligible region”; “The committee leaves working regions enough uninterrupted service time.” | No fixed failover/startup bound; one permanently eligible region alone does not satisfy the successor clause. The committee clause must be respected, but it is not a guarantee that AUTO deadlines accommodate all future startups. |
| Client delivery/queue conditions | `II:L503-L505`: requests reach continuing regions, fit batches, and have finitely many older requests in each local queue. `II:L1841-L1858`: oldest fitting requests, no queue-position reset, removal only on global inclusion, discarded suffix requests stay pending, evidence retained. | Infinite empty DATA alone does not prove request inclusion. Regional inclusion alone does not discharge a request. |
| Extra progress qualifiers | `II:L507-L512`: bounds do not bound recovery, state restoration, repeated rejected exports; window success needs the serving interval and “a compatible Espresso clock profile”; otherwise repeated AUTO may prevent progress. `II:L2113-L2119`: “timely startup for some serving interval”; one-eligible-region indefinite wait acknowledged. | These phrases do not give a numeric compatibility definition or a universal/recurring service quantifier. They cannot silently be strengthened for a new theorem. |

Part I has stronger explicit formulations that should not be imported invisibly. Its authority becomes simultaneously available at a finite `d(g)` to all correct replicas, including frozen ones (`I:L173-L180`); its fixed finite Byzantine identity set cannot replenish (`I:L185-L198`); its request fairness uses a common admission order (`I:L214-L220`). Part II replaces several objects, adds finite quotas, and states its own conditions. Part I's unbounded-height theorem (`I:L896-L927`) is about one unbounded engine, not a global chain that closes finite sessions before their output is accepted. A weighted/finite-session adaptation and a composition proof are required.

## 3. C1 — finite success followed by perpetual AUTO churn

**Classification:** Critical Error relative to unbounded global-height progress under the literal frozen progress premises; not a refutation of the existing conditional prefix-safety claim.

Construct this execution:

1. Use two regions A and B, all voters correct, `Delta_A=Delta_B=1`, quorum thresholds satisfying the fixed stake model, and quota one in every session. Required authorities and the retained anchor/state are available immediately. All initializers are correct. Use ordinary timestamps `theta=t`, so the clock lead is zero and there are no clock anomalies.
2. For an arbitrarily long but finite prefix of indices, fixed authorities have generous startup windows. Regional results are accepted before their deadlines. This provides a sustained successful service interval, and can include arbitrarily many successful sessions.
3. All subsequent immutable authorities have `W_start=1`, positive target one, and `0<C_auto<1`. This is permitted by the explicit positive-parameter requirements in `II:L225-L229`; the existing text imposes no all-indices lower bound on `W_start`.
4. At each public opening time `e`, the selected region immediately reaches Ready and enters. Protocol 4 still requires its correct initializer to wait `2 Delta=2` before NEWVIEW (`II:L1590-L1603`). Consequently no fresh DATA can be produced, much less committed and accepted, before the public deadline `e+1`.
5. At `e+1`, AUTO closes the branch before the payload, records the source cooldown, and selects the other region (`II:L1662-L1673`, `II:L1732-L1750`). The former source is eligible again before it is next needed. Thus A and B alternate forever, each entering its new correct-initializer turn immediately.
6. Old v3.1 engines keep running despite Force (`II:L1559-L1564`, `II:L937-L939`), eventually produce their quota-one result, and the working publication service submits and includes it within `T_pub+T_E`. Such records are raw authenticated Espresso bytes from a closed version and cannot advance `G` (`II:L1415-L1419`, `II:L2192-L2198`). The offered load can be a fixed finite session/result rate, so no unbounded-load attack is required.

After the finite good prefix, `G` stays constant forever. There is no bad clock, malicious committee, lost authority, lost state, Byzantine leader, missing entered leader turn, or violation of raw publication. The committee issues no interrupting controls at all. Every temporary boundary has another eligible region. The service bounds continue to hold; it is **acceptance before these future windows**, not raw inclusion, that fails.

This is exactly the distinction made at `II:L496-L499`, `II:L507-L512`, and `II:L2113-L2116`: success/timely startup is stipulated for **some** serving interval, not for the entire suffix. If “compatible clock profile” is intended to exclude this execution even though `theta=t`, the missing condition is a universal compatibility requirement on **window sizes and end-to-end startup**, not a clock property. Such a requirement would strengthen the frozen premises and cannot be smuggled in through terminology.

Even if a minimum window accommodates the nominal bootstrap, the separate unbounded pre-Ready restoration/recovery allowance remains: each newly selected session can take a finite time longer than its finite budget to become usable. Thus choosing a large constant is not a general repair under eventual-only restoration. A startup gate/adaptive mechanism must be proved from existing premises, rather than assumed to finish within the window.

## 4. C2 — a permanently eligible healthy region is never selected

**Classification:** Critical Error relative to per-region service fairness. This trace satisfies even indefinitely successful accepted-output service.

Let the fixed cyclic list be A, B, C. At finalized position `q0`, A and B are eligible and C is OFF. At a later position `q1`, an authorized, correctly announced ON makes C eligible permanently. All three regions are healthy, authority/state are available, and no Force occurs. Arrange all successors after the first to inherit/open with selection lower bound `q0`.

Every A/B terminal DATA legally uses `xi(T)=q0`. The reference is authentic, its height is nondecreasing, it is at least that session's `q_open=q0`, and it precedes the containing EXPORT (`II:L608-L623`, `II:L1506-L1510`). These are all current reference checks. There is no strict-advance or freshness rule.

For every regular boundary, the search begins at `q0`; its minimum eligible historical position is again `q0` (`II:L1468-L1479`). Therefore:

`A -> B -> A -> B -> ...`

C was OFF in that fixed historical snapshot, so it is skipped forever. C is eligible at **every actual snapshot from q1 onward**, not merely intermittently. A/B produce and publish on time indefinitely; all selected leaders are correct; the honest committee leaves all regions ON and undisturbed. Correct-leader fairness quantifies actual turns and does not force C to acquire one.

This is stronger than merely choosing a currently OFF historical successor: it proves infinite starvation of a permanently eligible healthy region while the global chain makes unbounded progress. It is permitted by the explicitly acknowledged historical-reference behavior (`II:L630-L645`, `II:L1490-L1494`).

An implementation choosing a fresh reference voluntarily does not repair the protocol guarantee: a compliant leader is presently permitted to keep `q0`, and a verifier must accept it.

## 5. Other candidate attacks and their correct classification

| Scenario | Verdict under the full frozen premises | Reason / exact boundary |
|---|---|---|
| Only A remains eligible forever; A completes quota and NextLive excludes A | Out-of-model as a full-premise counterexample; real one-live-goal mismatch | `NextLive` explicitly selects the “first other” region (`II:L1443-L1449`), but the progress condition requires eventual **another** eligible region (`II:L500-L502`). `II:L2118-L2119` already acknowledges the indefinite wait. |
| AUTO cools down the only source; cooldown later expires, but it still cannot select itself | Same classification; cooldown expiry alone is not renewal | The captured source remains fixed while Waiting (`II:L1451-L1458`); repeated Force does not change it. CLEAR cannot fix the exclusion of the source. |
| Terminal `xi` selects B after B's later OFF | In-model selection behavior, not by itself a permanent-stall proof | Source explicitly gives this example (`II:L630-L645`). OFF changes subsequent historical-snapshot selections, not an already fixed boundary (`II:L273-L277`). If B violates regional assumptions by disappearing, that disappearance is not automatically in-model. If B is merely OFF, it may still correctly serve its historical session. |
| Espresso timestamps stop growing forever | Out-of-model for progress | Violates `II:L494-L495`; cannot be used while claiming all frozen progress premises. |
| Espresso has arbitrarily large lag or jumps forward while respecting its upper lead | Allowed by the explicit numeric clock assumptions; end-to-end claim needs a compatibility proof | `theta<=t_E+epsilon_E` bounds lead only (`II:L417-L420`). A timestamp far behind real time can jump across a deadline almost immediately in real time. A regular positive timestamp window alone is not a guaranteed real-time service duration. |
| An old scheduled Force unexpectedly becomes applicable before correct online nodes could have obtained it | Out-of-model | With the actual advance-announcement and lead bounds (`II:L465-L476`), that trace violates the L1/clock/announcement premises. |
| A single timestamp jump recursively times out all successors in the same block | Not an attack on these rules | New startup windows open at the realized selection timestamp, not the old missed deadline (`II:L1761-L1767`); their positive lengths prevent same-block recursion. Later jumps can still cause new failures. |
| Data are permanently lost after global acceptance | Out-of-model | Explicit independent recoverability/retention premise (`II:L413-L427`). |
| A valid next result is recorded raw once, rejected behind a gap, and never retried | Justification/implementation gap if retry duty omitted; not a valid correct-service trace for the written worker rule | Publication alone is satisfied, but `II:L2192-L2196` directs workers to track, rebuild and resubmit. A proof must use that protocol action and the finite-interval acceptance premise, not equate raw inclusion with acceptance. |
| Perpetual stale-base rejection with a fixed open branch and permanently fixed G despite correct fully informed retrying workers | Not yet a demonstrated in-model trace | With a fixed base and all dependencies, a correctly rebuilt next interval has no stale-base reason to fail. To sustain the attack, specify changing branch/base, missing dependencies, unfair scheduling, or another actual acceptance obstacle. A changed base itself is global progress. |
| Every current branch closes before its valid result is accepted | In-model after only a finite successful interval; C1 | Timely inclusion of closed-version bytes does not undo closure. |
| Required recovery authority never appears | Out-of-model for eventual progress | Violates `II:L500-L501`. The absence of a fixed bound still allows arbitrarily long finite Frozen periods. |
| No correct initializer is ever entered | Out-of-model | Violates explicit actually-entered fairness (`II:L490-L493`), even if unused schedule positions name correct leaders. |
| One honest leader's term is followed by equivocation that cancels pending timers | In-model finite interruption; not a permanent liveness counterexample by itself | `II:L1297-L1305` explains why one correct leader is insufficient; permanent exclusion and fair entered turns must also be respected. An infinite replay of the same exposed equivocator violates that premise. |
| Force sets an old engine to Frozen or cancels all timers | Not v3.1; proposed-rule error if introduced | Only Quit/Completion have those effects (`II:L916-L939`, `II:L1092-L1102`, `II:L1148-L1159`). The outline correctly flags early termination as new behavior (`O:L377-L396`). |
| Correct Manager leaves a valid Ready session queued forever | Justification Gap in a new lifecycle, or a violation of retained entered-turn fairness, depending on the quantified claim | The outline chooses atomic Ready/Enter and warns that asynchronous delay requires a new bound (`O:L285-L289`). Do not add a new scheduler fairness axiom; define prompt local processing within the inherited complete-message model. |

## 6. Clock and publication deductions that can safely be used

For a complete regional result available at real time `s`, the sound unconditional transport deduction is:

`submitted <= s+T_pub`, and `raw result + finality evidence visible <= s+T_pub+T_E`.

This is stated explicitly at `II:L449-L462`. It is not a global-acceptance bound. To derive a window bound, the proof must account for (i) public opening versus first Ready, (ii) bootstrap/commit/FINAL formation, (iii) a valid next interval and its actual acceptance position, and (iv) mapping of real time into Espresso timestamps.

For example, if an opening has timestamp `theta0` and a particular complete result is accepted in a block finalized by real time `u`, then the lead bound only gives that block's timestamp at most `u+epsilon_E`. To guarantee its timestamp is **strictly** below `theta0+W`, one needs `u+epsilon_E < theta0+W`. Neither `T_pub+T_E` alone nor a nondecreasing clock provides that inequality: `s-theta0` can include arbitrary pre-Ready delay and timestamp lag. Equality is too late because windows are half-open (`II:L1670-L1673`).

Growing timestamps imply that any fixed finite threshold is eventually crossed if “growing” means unbounded; this itself should be made explicit rather than conflated with a quantitative rate. With no lag/rate bound, failover has no uniform wall-clock bound. The source already states no unconditional failover/withdrawal bound (`II:L2121-L2125`).

## 7. Theorems that are candidates, and the theorem that is not yet justified

### T1 — prefix safety and monotonicity

Under the frozen source-history, announcement, regional-validity and verification-interface premises, correct observers starting from the same authenticated checkpoint interpret a common complete history identically, and `G` only extends. This is the intended conditional argument at `II:L2064-L2101`. This preflight did not independently verify every regional safety lemma or every proposed v3.2 termination rule.

### T2 — resolution of an individual waiting boundary

For a fixed pending boundary whose branch is not superseded, if the complete relevant history/authority evidence becomes available and there is an eventual authenticated position with another eligible region, historical `NextLive` has a least such position and fixes one successor. Eventual state availability plus continuing correct local processing permits Ready; a fixed time bound does not follow. This uses the frozen successor condition rather than asserting it from “one region is live.” A Force that supersedes the boundary creates a different obligation.

### T3 — local bootstrap bound from first Ready

With the existing atomic Ready/Enter interpretation, complete-bundle regional relay, a correct initializer and no Quit, `II:L1607-L1617` yields entry by `e+Delta`, first fresh DATA proposal/votes by `e+6Delta`, and then the applicable uninterrupted timer/receipt reasoning. Do not measure this from public assignment, candidate creation, or start of state transfer. An asynchronous v3.2 Ready queue invalidates direct reuse of these bounds unless the queue is eliminated or its delay derived from already frozen processing premises.

### T4 — progress during a qualifying finite service interval

If a specific interval contains a usable current duty, its required complete next results, and their acceptance before the public deadlines, its successful windows add their positive targets and terminal acceptance completes finite quotas. This is an interval theorem. Its conclusion has the same finite/existential scope as its service premise; it does not quantify over all future heights.

### T5 — regional recovery / finite-quota completion, conditional on continued execution

A Frozen generation waits for `a_v+Delta` and the next finalized authority, then re-enters unfrozen (`II:L1161-L1176`). Eventual authority availability rules out remaining Frozen solely for a permanently missing authority. A finite-quota regional progress argument can adapt Part I's entered-fairness/exclusion/commit-window reasoning and use DRAIN when the terminal DATA is certified but lacks a receipt. It must explicitly establish delivery/entry-skew and receipt aggregation under the Part II authority interface; do not simply cite the unbounded Part I theorem. Once v3.2 legally retires an old engine, any continued-execution regional liveness guarantee must be scoped to engines that remain assigned to execute.

### Not established — unbounded G and fair service for every healthy eligible region

Neither `for every height h, eventually height(G)>=h` nor `every permanently healthy eligible region eventually contributes globally accepted DATA` follows from T1–T5. C1 refutes the first implication under the current literal service quantifier; C2 refutes the second even with unbounded G. Adding “at least one live region” does not resolve either trace: C1 has two healthy regions, and C2 has three.

Global request liveness is a separate corollary requiring infinitely many successful globally accepted service opportunities by correct leaders and the retained pending-request/queue rules. `II:L1841-L1858` supports such a proof, but the finite interval alone does not. Empty DATA gives chain-height progress without request inclusion (`II:L2115-L2117`).

## 8. Protocol changes compatible with frozen environmental premises

These are repair directions and proof obligations, not certified implementations. They change protocol rules, not the assumed external fault, timing, authority, service or cryptographic model.

1. **Permit same-region renewal as a fallback.** Search other eligible regions first, then allow the source itself if no other region is eligible. After AUTO, respect the source's cooldown until expiry; after expiry it may reopen a new logical session with the preserved `G`, fresh session identity and the same applicable authority rules. The ordinary singleton quota-completion path can renew without imposing a spurious cooldown. This fixes the singleton exclusion without changing fault bounds or assuming a second healthy region. It does not solve C1 by itself. The frozen “another eligible region” condition remains a stronger legacy sufficient condition; a one-live theorem for the revised protocol must not rely on it as if it followed from one live region.

2. **Make normal selection references advance.** A protocol-level strict lower bound such as a terminal reference later than its session's opening reference removes the constant-`q0` trace. Because block heights are well-ordered, infinitely many completed sessions then cannot remain before a fixed later ON position. This is weaker than a freshness bound and preserves deterministic historical selection. It may require waiting for a new authenticated Espresso position, so the latency effect must be stated. Full service fairness still needs a proof covering Forces, eligibility changes, quotas and candidate activation; fixing this trace is not that proof.

3. **Separate preparation from a timed serving obligation.** Existing eventual-only restoration is incompatible with treating every selected-but-unready candidate as necessarily able to meet a fixed finite startup deadline. A robust protocol must preserve preparation work across discarded forecasts and derive formal activation from authenticated readiness, or use a proved adaptive/search mechanism that eventually gives a healthy prepared region enough service. It must still be able to bypass an unusable candidate; merely disabling all timeouts until that candidate is Ready can deadlock on it. If a new Ready/activation certificate is used, its transport, admission, authority and replay rules must be defined and derived from existing services, not assumed as a new bounded oracle.

4. **Do not hide end-to-end progress in a new assumption.** Merely adding “Manager scheduling is bounded,” “state recovery is bounded,” “all future windows succeed,” or “service intervals recur forever” would strengthen the frozen premises. A minimum startup length derived from the existing `Delta_i` removes the short-bootstrap instance of C1, but eventual-only restoration and accepted-export retries still require protocol work. The existing authority's fixed positive parameters cannot be retroactively rewritten by a handler.

5. **Specify correct retry workers and prove the right progress alternative.** Keep the existing requirement to replay acceptance and rebuild the remaining continuous suffix after a stale base or gap. A useful proof split is: either other valid records keep changing `G`, which is already global progress, or `G` eventually stabilizes, so an informed worker has a stable next base. The remaining obstruction then is branch churn or missing next results, not an unexplained forever-stale base. Any repair must prevent perpetual closure of every prepared result before it can be accepted.

6. **Keep Ready/Enter atomic and do not make new activation wait for old termination.** The outline already has the right direction (`O:L285-L289`, `O:L324-L339`, `O:L396`). The close procedure must preserve required votes/evidence/timers until its proved retirement condition. New-session preparation/activation can run concurrently. A “wait until every old Frozen engine completes” rule would unnecessarily couple useful new work to unbounded old recovery-authority delays.

The outline's proposal to “增加 Manager 处理、状态恢复、持久化和收尾接口的服务条件” (`O:L516`) is the principal assumption-freeze hazard. Under the user's freeze, write operational obligations and derive whatever follows from existing premises; do not add independent scheduling/restoration/retirement guarantees and then call the original assumptions unchanged.

## 9. Required proof decisions before a final v3.2 liveness claim

- Define “live/healthy” operationally and distinguish it from the replay eligibility flag.
- Keep global unbounded progress, per-region service fairness, and client-request liveness as separate target statements; do not replace the stronger requested goal with T4.
- Resolve C1 with an actual activation/window/retry mechanism or prove why the existing frozen conditions already exclude it using an explicit quantifier. The latter cannot be done by silently changing “some” to “every” or “infinitely often.”
- Resolve C2 with a selection rule that cannot stay forever behind a permanent eligibility update; prove fair service, not merely eventual selection.
- State exactly whether revised same-region renewal supplies the one-live case; do not present the legacy another-region premise as satisfying that goal.
- Keep all source-history, sound verification, fault/stake, clock, authority, availability, committee and progress premises in the adversarial audit's assumption filter.
- Re-audit the completed v3.2 source from a fresh context. This report is preflight evidence, not a substitute for that audit or for executable/model-checking validation.

Validation performed: source-level inspection and hand-constructed symbolic executions only. No simulation, model checker, theorem prover, or LaTeX build was run for this preflight. The inspected protocol drafts were not modified.
