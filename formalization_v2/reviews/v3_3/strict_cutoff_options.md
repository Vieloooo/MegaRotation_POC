# Strict-cutoff liveness options: narrowed analysis

Date: 2026-10-18. Read-only design research while the user is deciding the Force-semantics question. This report does not choose an alternative or edit a protocol. It replaces broad impossibility language with explicitly scoped obstructions and candidates.

References `V33:Lx-Ly` mean the inspected `formalization_v2/sync_hotstuff_rr_part2_rotation_v3_3.tex`; `RB:Lx-Ly` means `formalization_v2/sections/rotation_v3_3_ready_bundle.tex`; `I:Lx-Ly` means `formalization_v2/sync_hotstuff_rr_part1_core.tex`. The v3.3 model section is inherited from v3.1; in particular its control first-crossing semantics are already **inside the frozen Model section** (`V33:L274-L284`).

Observed SHA-256: v3.3 main source `e69ba625229741039d704943d297830476242dadddbf473ae8cb3375f24b4f88`; ReadyBundle fragment `b5347497e6b925c15cd22bd78285c36fbf31ec1ff3d2f20fda3a770295f89858`.

## 1. Main conclusion

There is no demonstrated impossibility for **every** protocol-only repair preserving strict no-old-output-after-effective-Force. The earlier fresh-delay argument is not sufficient once a protocol shares preparation for the same fixed G across retries. A correct strict-cutoff repair may preserve useful state, authority preparation, and scheduling progress across attempts while still rejecting every old-beta DATA append after closure.

However, the current v3.3 fixes and larger AUTO windows alone do not establish the goal. A sharper counterexample below defeats globally increasing finite per-duty budgets even with perfectly shared native state, fixed finite membership, regular clocks, and infinitely many actually entered correct producing turns. It uses **new recovery-authority availability waits at each new r**, not repeated restoration of the same G.

The strongest unresolved strict-cutoff path combines:

1. Persistent state preparation keyed by exact anchor, independent of speculative session cancellation.
2. No repeated loss of authority/scheduling progress at a fixed global frontier; potentially a stable authority epoch and persistent recovery cursor across zero-output retries, or a canonical authority-wait gate.
3. A globally persistent adaptive AUTO allowance, with a proof that it eventually dominates a **derived** finite response time once G is fixed.
4. An explicit honest-committee submission policy that prevents infinitely many preempting MSIG cutoffs at a stalled frontier, while every MSIG that actually is submitted retains its old-beta check, priority, and first-crossing effect.
5. Separate fair-service and request-service reasoning; the fixed-G argument for global progress is not itself a fairness proof.

Items 2–4 are substantial design work, not a completed theorem. An arbitrary stream of authorized hard-Force requests cannot simply be presumed to obey item 4. Restricting an honest controller's submissions is different from queuing an already-effective Force, but the permitted operational policy must be explicit.

## 2. What v3.3 already repairs, and what remains unchanged

| Interface or rule | Inspected source | Consequence |
|---|---|---|
| Historical handoff validity separated from current permission to enter | `V33:L1854-L1911` | A later closure does not invalidate retained-anchor evidence, but prevents reopening the closed target. |
| Exact executable ReadyBundle and atomic initial Enter | `RB:L11-L47`, `RB:L60-L101` | State preparation and bundle formation occur before first Enter. Cache sharing is possible, but the first construction has no fixed bound. |
| Conditional regional timing implication | `RB:L103-L140` | `e_v <= e+Delta` only when adoption/Enter is genuinely within the modeled complete-message handler. Native database import is not magically bounded. |
| No new public readiness service | `RB:L144-L148` | ReadyBundle is not a quorum certificate, an Espresso record, or a public-window reset. |
| Strict terminal reference advancement | `V33:L629-L645` | Infinite normal handoffs cannot keep using the same old selection height. The selection proposition expressly requires an infinite successful sequence and fixed eventual eligibility (`V33:L2638-L2673`). |
| Same-region fallback | `V33:L1768-L1782` | The source can renew when no alternative is eligible; AUTO cooldown and governance OFF still apply. |
| Retirement stops fresh activity but preserves surviving timers | `V33:L1033-L1041` | A late receipt may still form after closure, but cannot restore old-beta append permission. |
| Strict Force and priority | `V33:L2150-L2162`, `V33:L2171-L2207` | Every matching effective MSIG acts at first crossing before AUTO/payload; new-version DATA cannot appear in the triggering block. |

The existing slow-session example uses a fixed startup budget and remains useful (`V33:L2683-L2719`). It does not establish impossibility of adaptive retry design. The present analysis must not overextend it.

## 3. Correction to the repeated-restoration argument

Suppose G stops at a particular anchor A. A protocol keeps one native-state restoration task per actual region/required state representation, keyed by A's producing identity and hash. A Force changes the target session, but does not cancel that task or invalidate the verified application state at A.

If the relevant set of regions/configurations is finite and stays fixed, and each required restoration finishes eventually, then all of these finitely many tasks finish by some finite time. The protocol need not know that time. An adversary cannot thereafter choose a fresh arbitrary restoration delay merely because beta changed: that would ignore the retained completed work.

Consequently, “choose restoration delay larger than budget on every attempt” only refutes a protocol that actually creates independent new obligations or discards useful preparation. It does **not** refute a protocol with successful fixed-anchor coalescing. The correction matters even though every individual first restoration can take arbitrarily long.

There are limits to this reduction:

- At a moving G, infinitely many distinct anchors may require work. A fixed-G global-stall contradiction does not imply that a slow healthy region catches every moving anchor.
- Different immutable authority snapshots may require different members and different executable-state formats/adapters. Sharing one state object is not automatically complete readiness for infinitely many fresh configurations.
- New r-specific authority evidence is not the same object as the old state's verification proof. Caching state does not create `AUTH[r,g]` before L1 publishes it.
- Coalescing does not preserve old voting locks or authorize an old-beta append. New-session core state remains separate, and strict cutoff can remain intact.

## 4. An explicit adaptive-budget counterexample using recovery authority

This execution refutes the following **specific** policy family: at each new assigned duty, choose a finite public AUTO deadline; a failed attempt moves to a fresh r; the deadline is not suspended or extended because that engine is waiting for a required recovery authority. The allowance may grow globally without resetting on region or beta changes. It may be arbitrarily large.

Use two regions A and B. Each has the same three members for the entire execution: correct H0 and H1 and one silent Byzantine B, with unit weights, `W=3`, `F=1`, `Delta=1`. All native state for the fixed G is already cached. All initial authorities `AUTH[r,0]` are ready. Every session has quota one and begins with silent Byzantine initializer B. Correct-to-correct message delays can equal one. Espresso timestamps equal real time. Use a finite good prefix to meet the existential successful-service-interval premise.

Consider later attempt n, opening at a_n with adaptive deadline D_n. Discard a finite prefix so `D_n-a_n` is large enough for the timeline below (e.g. at least 14); if budgets never become large, the fixed-window examples already apply.

1. A finalized header after the opening selection height appears immediately. Both correct members enter generation 0 with the full cached state and valid initial authority. B sends no NEWVIEW. Correct BLAME is generated at the first progress check, and a matching weight-two proof freezes generation 0 by approximately `a_n+7`.
2. The engine must wait for `AUTH[r,1]` after its Delta recovery wait (`V33:L1214-L1228`). Let this authority finalize and become available simultaneously to all correct members at

   `d_n = D_n - 5.5`.

   This is finite and later than the initial freeze/wait. It designates a correct recovery initializer, alternating H0 and H1 across attempts. Authority records are immutable. There is no stated upper bound requiring this authority to have finalized earlier. The construction even satisfies Part I's stronger simultaneous finite availability form for this authority (`I:L173-L180`).
3. Both correct members enter recovery at `e=d_n`. The correct initializer waits two, sends NEWVIEW at `e+2`, receives the other correct member's ANCHOR vote at `e+4`, and proposes the terminal DATA immediately. Its own DATA vote is at `e+4=D_n-1.5`; the other correct member votes at `e+5=D_n-0.5`.
4. The two commit timers expire at `e+6=D_n+0.5` and `e+7=D_n+1.5`. With the Byzantine member withholding FINAL, no complete weight-two FINAL receipt exists before `D_n+1.5`.
5. At D_n, AUTO closes the version with zero accepted DATA. Both correct DATA votes already occurred, so retirement does not cancel their timers. The late receipt and complete result form, are retained/multicast, and are submitted/included within the stated raw service bounds. Strict cutoff rejects their old-beta append.
6. The next attempt has a new r and a new, individually finite `AUTH[r,1]` publication time chosen in the same way. Cooldown can be a fixed value shorter than the time until that region is selected again. All necessary later unused authorities can also be made eventually available and their schedules fair.

Every correct identity is an actually entered correct recovery initializer and DATA producer infinitely often, not merely a name in an unused schedule. Both correct voters commit infinitely often through surviving timers. The Byzantine member never equivocates, so permanent equivocator exclusion is irrelevant. No new corrupt identities, changing membership, state-restoration restart, clock anomaly, private control omission, or raw-publication failure is used. No committee control interrupts the tail.

The delayed authority can be submitted sufficiently late for any applicable actual-submission service bound to be satisfied; no frozen premise requires its early submission. Part II's bounded L1 control-operation service is not itself a deadline for producing future recovery schedules.

This is **not** a counterexample to every adaptive design. It fails if the protocol prevents the fresh-authority treadmill, or canonically suspends the relevant AUTO exposure until current required authority is available and then preserves enough recovery progress. Those repairs need actual public state-machine definitions. Simply saying “increase W globally” does not address this execution.

## 5. Why global adaptive AUTO is nevertheless a serious candidate

Once a fixed-G repair derives a finite uniform response bound L for all sufficiently late attempted duties, the existing Espresso **upper-lead** bound can support an amortized argument without adding a lower-lag bound.

For intuition, let theta_n be opening timestamps and t_n their real finalization times. If only AUTO continues at a stalled G and the effective timestamp allowances are b_n, then successive failed attempts satisfy

`theta_(n+1) >= theta_n + b_n`.

If every sufficiently late attempt would accept positive output within L real time unless it is closed first, continued failure forces

`t_(n+1) <= t_n + L`.

Together with `theta_n <= t_n + epsilon_E`, cumulative failed timestamp budgets cannot exceed an affine function of the number of attempts. Therefore an allowance sequence whose cumulative average tends to infinity cannot sustain those failures forever. An arbitrarily large but finite initial clock lag is absorbed in the additive constant. This is a proposed proof route, not a complete theorem for v3.3.

The hard part is deriving L from frozen premises rather than assuming it. Fixed-G state coalescing may remove restoration from the tail. A stable finite configuration and a preserved scheduling/recovery cursor may leave only finitely many required authorities before a correct usable initializer. Their finite availability times then have a finite maximum. But fresh r, changing snapshots, erased recovery progress, unbounded new authority waits, and independent MSIG interruptions prevent that reduction in the present protocol.

Thus unbounded Espresso lag alone does not establish impossibility of adaptive AUTO. Conversely, no amortized clock calculation supplies the missing regional/authority bound by itself.

## 6. Authority and scheduling continuity while keeping strict cutoff

A promising research direction is to distinguish a **retry** at unchanged G from a new service/configuration epoch. At a stalled frontier, preserve an authority namespace and progress toward a usable leader instead of incrementing into unrelated fresh schedules after every failed output attempt. Every effective Force can still increment beta, reject all old-beta output, preserve G, and start a separately identified engine.

The current model already permits a new beta to reuse a numeric r and requires it to read the same immutable authority (`V33:L254-L257`). However, the current Force successor rule advances r once a duty was assigned (`V33:L1784-L1797`). Reusing an authority epoch for zero-output retries would therefore be an explicit protocol/indexing change, not an implementation cache optimization.

Do **not** implement the idea merely by restarting `(same r,g=0)` forever:

- It can repeatedly select the same silent Byzantine initializer instead of preserving recovery progress.
- If an initializer equivocates and is exposed, blindly reusing an immutable old assignment forever conflicts with permanent exclusion from later leader schedules.
- A quit proof from old beta cannot automatically be reused as the new engine's admission proof. New bootstrap authorization must specify how a retained scheduling cursor selects an immutable authority while using fresh locks/votes and the exact retained G.

A viable design must identify which state is portable: cached authority evidence, the scheduling cursor, proven exclusions, and native state are candidates; old commit timers, votes and unsafe locks are not. It must keep the fixed session membership/quota obligations and explain how the new branch obtains its initial authorization without rebinding an existing AUTH record.

An alternative is a canonical wait gate for required authority. The full frozen model promises eventual availability, so waiting for a **fixed required record** need not introduce a new environment assumption. But the public replay must know which record is awaited and when the gate ends. Local fetch completion is not a canonical timestamp; absence from an incomplete local view is not a proof of authority absence. Once genuine authority is available, recovery must remain schedulable rather than be immediately replaced at a historical expired deadline.

Either direction can defeat the explicit trace in Section 4. Neither has yet been specified sufficiently to prove the full goal.

## 7. Membership finiteness: what can and cannot be used

Part I explicitly fixes `n=2f+1`, a Byzantine set fixed from the start, and a finite nonreplenishing Byzantine identity set (`I:L153-L154`, `I:L185-L198`). Part II replaces the threshold with weighted regional snapshots and explicitly fixes membership/weights **for a whole session including recovery** (`V33:L350-L363`). Initial authorities fix each r's members and delays (`V33:L244-L250`).

It is reasonable to treat an installed regional roster and the cyclic region list as finite protocol objects; v3.3's selection proof explicitly uses a finite cyclic list (`V33:L2654-L2659`). Nevertheless, the source does not explicitly fix one global voter population, one maximum Delta over all future r, or a finite union of every future snapshot. Do not import such a uniform global bound from Part I without reconciling the r-indexed configuration model.

The adaptive counterexample above does not depend on this ambiguity: it uses a fixed finite population and constant Delta. For a constructive stable-frontier proof, explicitly keeping a finite installed configuration epoch would be useful protocol behavior; assuming that arbitrary future authority configurations happen to stay in a finite uniformly bounded family would be an extra environment restriction.

## 8. Honest HealthPolicy versus extra environmental fairness

The frozen committee is honest and coordinated and submits only authorized, future-dated immutable controls. Current v3.3 gives qualitative OFF/ON/CLEAR/MSIG purposes, but leaves health thresholds operational (`V33:L1576-L1595`). This is not yet an algorithm proving a finite number of harmful cutoffs at a stalled G.

There are three different proposals that must not be conflated:

1. **Extra assumption:** “The committee eventually stops interrupting a working region” or “it grants infinitely many sufficiently long intervals.” This strengthens or resolves the frozen service quantifier in a new way and is not allowed as a silent proof step.
2. **Submission policy:** Define an honest controller algorithm, for example a persistent bounded number of MSIG retry requests per unchanged G, or a monotonically increasing service allowance which it cannot bypass absent a specifically defined new event. A correct committee follows this algorithm. Every submitted MSIG still takes effect exactly at its scheduled crossing, with inherited old-beta matching and MSIG priority. This changes what the protocol authorizes/submits, not when an already submitted Force takes effect.
3. **Deferred effective Force:** Accept a submitted matching Force but postpone its effective cutoff until useful output or quota completion. That changes the frozen first-crossing semantics and is one of the user choices still pending. It must not be hidden inside option 2.

Option 2 is the strongest potential strict-cutoff lever, but requires an explicit authorization/operational decision. If the intended Hard Force interface permits an arbitrary stream of valid operator interventions, an implementation cannot claim those inputs obey a new rate limit unless the protocol actually rejects or declines them. Counting “urgent” exceptions must not quietly restore an unlimited preemption channel.

A bounded-per-G MSIG policy also needs to handle already pending records and STATUS/OFF operations, not only newly generated Force requests. It must not wrongly consume a valid due record, reorder it below AUTO, or ignore a matching target. The unchanged interpreter continues to apply every valid submitted record according to the inherited rules. Health observations alone do not supply an accurate oracle for permanent regional failure.

## 9. Timing anchors without a new publication assumption

The shared global clock and a certificate threshold containing a correct signer may allow a protocol to authenticate a suitable real-time statement. That could help calibrate a deadline independently of Espresso's unbounded lag. But it is a new message/certificate design, not a property of existing selection references or a regional receipt.

The transport gap is concrete: `RB:L144-L148` expressly says ReadyBundle is regional only and supplies no publisher guarantee. The frozen publication service covers a **complete regional result**; Espresso's bounded service starts only after an admissible complete record is actually submitted. A new READY or lease message has no automatically inherited bounded funded submission path.

Possible derivations to investigate, without asserting them:

- Carry the timing evidence in a genuinely covered complete-result package. This is useful after a result exists, but cannot by itself make the first result beat an earlier startup cutoff.
- Use an existing valid committee control operation with a concrete submission policy. Its already-frozen real-time submission/announcement bound can authenticate a prospective control schedule. Do not assume a bounded time for the committee to learn a private Ready event unless the source interface provides it.
- Avoid a new time certificate entirely if a stable-frontier response bound plus cumulative AUTO-budget reasoning suffices.

Choosing a deadline from an observer's current wall clock or local Ready receipt would give different canonical histories. L1 timestamps are not stipulated as the missing cross-service commit clock. Neither shortcut is justified.

## 10. The one-live-region boundary and the separate fairness problem

Within the full frozen regional model, each assigned continuing session has bounded correct regional communication, a correct certificate-forming weight, eventual required authority/native state, and entered correct-leader fairness. “Only one live region” can describe eligibility or which region currently serves, but cannot simultaneously mean that all other assigned correct quorums permanently violate these premises.

A broader deployment target allowing permanently dead eligible alternatives needs a model of regional unavailability. A ready/health flag alone does not establish that a region will remain operational. An impossibility example based on such permanent failures is outside the current full model; a liveness proof based on every region's eventual recovery does not establish tolerance of them either. The self-fallback is a real rule improvement without resolving this model boundary.

Even if the strict-cutoff global-progress route succeeds, useful service for every persistent healthy eligible region remains a separate issue. Global G may grow while a particular region continually prepares obsolete anchors. Strict xi and cyclic selection fix the old fixed-historical-snapshot problem, but not perpetual post-selection failure. A persistent fair service cursor, retained per-region work, and a proof that each such region receives an actually usable interval are still required.

Request progress is yet another obligation. v3.3 explicitly permits the trace where fast Byzantine-led empty DATA enters G while correct-led requests always miss the cutoff (`V33:L2721-L2739`). A budget policy that resets on any accepted DATA can preserve that starvation. A global monotone allowance, or service credit tied to the right correct/pending work, may help, but must be proved under changing anchors/authorities rather than inferred from aggregate height growth.

## 11. Decision table for the next protocol iteration

| Candidate | Compatible with strict cutoff? | Current assessment |
|---|---|---|
| Keep verified fixed-G state work across candidate changes | Yes | Useful, removes an overbroad repeated-restoration argument; does not solve new authority waits. |
| Strict xi and same-region fallback | Yes; already in v3.3 | Useful partial selection repairs only. |
| Nonresetting adaptive AUTO allowance | Yes in principle | Needs an explicit new AUTO rule and derived stable-frontier response bound; Section 4 defeats the simple fresh-r form. |
| Keep fixed authority epoch / recovery scheduling progress across zero-output retries | Yes in principle | Promising substantive repair; requires new admission/indexing/exclusion definitions and independent verification. |
| Canonically wait for a fixed required authority | Yes in principle | Uses eventual availability, but needs public gating semantics and must avoid immediate expiration at an old timestamp. |
| Concrete honest-committee submission HealthPolicy | Yes if it restricts submissions rather than delays already-effective events | Potentially removes infinite MSIG interference; authorization scope and exception channels must be explicit. |
| New public Ready/timing certificate | Potentially | No automatic bounded publisher path; cannot just assert `T_pub` for the new type. |
| Queue an already due Force until quota completes | No under the presently retained frozen first-crossing rule | User choice pending; not authored here. |
| Accept compatible old-beta output after effective Force | No under strict cutoff | User choice pending; not authored here. |

The strongest defensible status is **a plausible strict-cutoff path with unresolved authority/scheduling, committee-policy, and fairness obligations**, not a universal impossibility and not a completed liveness proof. The existing simple adaptive-window approach has a concrete in-model obstruction. Preserving work and authority progress can remove that obstruction without necessarily changing the cutoff event itself; this deserves further design and fresh review before declaring the strict option infeasible.

Only this report was written. No protocol source, frozen assumption, submitted control rule, or pending-choice-dependent alternative was changed. Validation is source inspection and explicit symbolic execution analysis; no model checker or theorem prover was run.
