# Strict-cutoff next design: concrete candidate and remaining obligations

Date: 2026-10-18. Research specification only. The user's cutoff choice is still unanswered. This document does not change effective Force timing, accept old-beta output, edit a protocol source, or claim a complete proof.

Inspected main source: `formalization_v2/sync_hotstuff_rr_part2_rotation_v3_4_1.tex`, SHA-256 `67cf324bcda71a8ee2bad74772ef618a4c2000b002eef31390c2fcb468e3b133`. Frozen model: `formalization_v2/revisions/v3_1_frozen/model_and_assumptions.tex`, SHA-256 `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387`. `V:Lx-Ly` below refers to the main source; `M:Lx-Ly` refers to that frozen model; `I:Lx-Ly` refers to `sync_hotstuff_rr_part1_core.tex`.

## 1. Recommended candidate, with an explicit incompleteness boundary

The most promising strict-cutoff candidate is **stable-slot retry with persistent preparation, increasing tolerance for failed windows, canonical selector advancement, and a bounded hard-intervention submission policy**. A service-round debt rule can then turn global progress into useful regional fairness within the full regional model. A separate rescued-request rule can address the documented censorship trace, but it changes batch validity and still needs an ingress argument.

There are two material unresolved issues before this becomes a theorem:

1. **Authority reuse and exclusion.** Replaying immutable `AUTH[r,0]` after every beta change may replay a previously exposed Byzantine initializer. The frozen exclusion/fairness wording must support the precise finite usable-generation argument, or the protocol needs an explicit admission/scheduling repair. It is not permissible to rebind AUTH, silently substitute a leader, or reuse an old-beta Quit as a new-beta admission proof.
2. **Complete request ingress.** A rescued inbox can force future DATA to include a request once that request appears in an authenticated complete regional result. The frozen premises do not yet prove that every admissible correct-client request reaches such a result before retirement. Direct request publication is not covered automatically by `T_pub`.

The rules below make the candidate concrete enough to review, while marking these unresolved points instead of assuming them away.

## 2. Premises that remain frozen

The full model, not merely its assumption environments, is retained. In particular:

- One immutable authority per `(r,g)`; `AUTH[r,0]` fixes all regional snapshots, weights, delays, K, quota and positive window/cooldown parameters; later generations change only leader schedule; a reused r uses the same authority (`M:L30-L44`).
- Per-snapshot Byzantine weight below half, a correct signer in every certificate, fixed membership through a session's recoveries, correct state/data retention, unforgeability and binding hashes (`M:L135-L153`).
- Correct-to-correct regional delivery and complete validation processing within Delta, and a shared global clock (`M:L167-L179`).
- `T_pub` begins only with a complete regional result. A result already raw-published satisfies that service; accepted-chain retries are separate (`M:L181-L196`).
- Honest coordinated future-dated control service; the actual-submission L1 bound; compatible complete Espresso histories; long-term recoverability; nondecreasing timestamps with upper lead only; actual-submission Espresso inclusion/visibility bounds, conditional on the stated online/keep-up and load conditions (`M:L199-L269`).
- Control first crossing is applied before payload, with STATUS/CLEAR then matching MSIG then AUTO (`M:L61-L78`); advance announcement, sound regional/application/DA interfaces, and complete controls remain required (`M:L272-L291`).
- Actually entered correct-leader turns, permanent exclusion of exposed equivocators, growing retrievable Espresso history, complete results, a successful **some sustained service interval**, eventual required authority/state, eventual another eligible region at a waiting boundary, the committee's service-time condition, and client queue/delivery conditions all remain (`M:L293-L320`).

No constant-time restoration, empty-Espresso-block generation interval, uniform future-authority latency, finite union of all future configurations, unbounded bandwidth, or new funded request/READY publication guarantee is added.

## 3. State split for the candidate

Persistent replay state, in addition to current G and beta:

| State | Meaning |
|---|---|
| `r` and current duty anchor A | The immutable authority/configuration slot and exact opening anchor. |
| `accepted_in_duty` | Number of this duty's DATA already accepted into G; derived from replay. |
| `nu` | Global effective-Force counter. Increment on every effective Force, never on stale/duplicate controls; do not reset on ordinary DATA or normal handoff. |
| `miss_count` | Consecutive failed base windows of the current startup/running phase. |
| `round_debt` | Optional finite set of regions still owed useful service in the current fair-service round. |
| `msig_submissions[G_digest]` | Honest-controller policy ledger, distinct from the interpreter's due-record ledger. |

Persistent local shared work:

- `Prep[(anchor producing pair, anchor hash, cfg/state-format, region)]`: native state restoration and its validation evidence. Session cancellation detaches a consumer; it does not delete useful work or restart an already completed fixed-anchor task.
- `AuthCache[(r,g)]`: validated immutable authorities. Keep future recovery authorities learned during any beta.
- Source verification checkpoints, complete regional results, pending client requests, and complete publication/retry obligations remain durable.

Each new `(beta,r)` still has fresh core locks, votes, entry times and timers, starting at `g=t=0`. Shared native state and authority evidence are not shared BFT state. This candidate does **not** smuggle old-generation votes, Quit proofs or timer state into the new branch.

## 4. Rule S1 — retain the authority slot on a zero-output Force

At an effective Force, run the existing old-beta check, MSIG priority, pre-payload cutoff, G preservation, and beta increment unchanged.

Choose the target numeric slot as follows:

```text
if an assigned duty has accepted_in_duty == 0:
    target_r = current_r
else if an assigned duty has accepted positive DATA:
    target_r = current_r + 1
else:
    retain the pending boundary's already determined target_r
```

The target anchor is still exactly pre-payload G. The selected region follows the canonical selector below. New beta distinguishes the new engine from the closed one even when r is retained. `AUTH[r,0]` is read unchanged; no authority is rebound. The witness explicitly binds whether this is a zero-output retry or a new-slot handoff, its source context, and target r.

**Regular completion remains r+1.** A normal boundary still inherits the completed terminal DATA and does not change beta.

This is an explicit change to the Force target-index rule, currently summarized at `V:L203-L205` and in the handoff definitions. It is not merely an implementation detail. If G eventually becomes constant, after at most one positive-output transition the retry slot and its configurations stabilize. That removes the fresh-`AUTH[r,1]` treadmill from the previous strict-cutoff counterexample.

## 5. Rule S2 — move strict advancement from DATA readiness to the next regular selector

The present `V:L638-L639` requires terminal `xi > q_open`, and `V:L1973-L1977` requires a later header **before Ready**. No frozen premise bounds the next empty Espresso block's generation time. The first later header can itself close this attempt before any engine enters. A larger window does not repair that circular dependency by itself.

Use this canonical replacement:

1. Initial Ready needs the current opening header and its complete evidence, but not a header after it.
2. DATA keeps the original check `height(xi) >= q_open` for the first block, monotonicity along ancestry, and strict precedence of xi before its containing EXPORT block. Remove the special terminal `xi > q_open` check.
3. At regular completion, set the next search lower bound to

   `q0_regular = max(height(xi(T)), q_open_of_completed_session + 1)`.

4. Search the same authenticated historical snapshots from this q0 onward. Force still starts its search from the triggering block's height. All boundaries and selections remain deterministic.

Then consecutive regular opening heights strictly increase by a **verifiable selector rule**, rather than by a claim that a leader chose its freshest local view. A correct proposer may use its newest verified header as an operational policy, but a verifier must not pretend to prove that it was locally newest.

If a later header already exists, a direct terminal receipt can still open the successor before terminal publication. If no later header exists, the completed source can form and offer its terminal result without waiting for one. That complete-result publication is already covered by `T_pub`; its Espresso inclusion within `T_E` supplies a later finalized block. The wait has moved after completion, where an existing covered input can drive it, rather than before the first possible covered input.

An EXPORT containing the terminal DATA can still accept that DATA while the successor waits. If its own block is the first possible successor selection position, new successor DATA cannot be accepted in that same EXPORT: the successor's reference lower bound would equal the containing block, violating the existing strict-precedence check. Multi-session EXPORTs remain possible when the successor was selected at an earlier authenticated position. This corner case must be specified explicitly.

S2 changes selector/Ready rules, not Force timing, old-output rejection, or the regional commit algorithm. It avoids requiring a new READY publication service.

## 6. Rule S3 — persistent tolerance for failed authority-defined windows

Do not rewrite a fixed AUTH window length. Continue checking its startup/running windows and positive targets exactly as base monitoring windows. Change only when repeated misses authorize AUTO:

```text
M = 2^nu                          // illustrative deterministic unbounded function
on a failed base window:
    miss_count += 1
    if miss_count < M:
        record the miss; open the next base window in the same phase
    else:
        execute one AUTO at this crossing, using the existing cutoff and cooldown
on a successful base window:
    miss_count = 0
    apply the existing startup-to-running / running progression
on duty replacement:
    miss_count = 0                // nu does not reset
```

Count/quota updates and terminal cancellation remain tied to accepted DATA. A miss alone does not become slashable evidence. A matching due MSIG still takes priority. Effective Force increments `nu`; ordinary progress, empty DATA, different source regions and new numeric slots do not decrease it.

This is a substantial change to AUTO sensitivity: it tolerates multiple failed fixed windows, rather than changing the immutable window field or falsely claiming one short window now accommodates arbitrary startup. It provides no fixed failover bound. The illustrative exponential function should be reviewed for operability; the proof condition is increasing cumulative failed allowance, not this particular encoding.

At a stalled G with stable r, base lengths are fixed positive values. If a finite late real-time response bound L can be **derived**, ever-growing cumulative AUTO exposure can defeat clock lag by the amortized argument in the prior report. Neither eventual state availability nor actual leader fairness alone is that derivation.

## 7. The essential authority/exclusion and actual-entry proof obligation

After S1, there are only finitely many fixed regional snapshots at a stalled G. Their native state can be prepared once. It is tempting to conclude that a finite set of recovery authorities eventually makes every retry fast. The following issues must be settled first.

### 7.1 Reused Byzantine initializer

`AUTH[r,0]` may always name a Byzantine initializer. Caching state does not make that leader correct. The unchanged core can blame a silent initializer and recover, so a bounded number of bad **early generations per attempt** is tolerable once their successors' authorities are available.

An equivocating initializer is harder because the same immutable initial assignment is replayed at each new beta. There are two readings of the frozen exclusion condition:

- If “excluded permanently from later leader schedules” prohibits every later actual assignment, a future retry cannot simply run the old excluded initializer again. S1 plus unconditional g=0 restart is incomplete. No immutable AUTH rewrite is allowed, and old-beta equivocation is not automatically a Quit proof for the new beta. An explicit new admission/scheduling rule or a different retry-start design is required.
- If the intended inherited rule is Part I's exclusion from a sufficiently high **authority generation index** (`I:L185-L203`), replaying lower indexed authorities does not erase the exclusion tail. A proof may tolerate replayed bad early generations, but must show each sufficiently long retry reaches a usable generation beyond that tail. This interpretation must be justified from the frozen source; do not silently select it because it makes the proof work.

The same caveat applies to “same authority”: the authority's fixed `(region,g,t)` schedule must be interpreted identically on each beta retry. An authority that encodes an arbitrary beta-dependent fresh schedule does not yield a stable finite usable prefix merely because its bytes are cached. Its exact schedule semantics must be pinned down.

### 7.2 Actually entered fairness, not unused honest names

The frozen progress text says correct initializers/producing turns are **actually entered** (`M:L297-L300`). A paper proof cannot point to an unused future honest entry and declare progress. Conversely, fairness of actual correct turns across repeated beta retries does not automatically show that any one retry reaches a high clean generation: all those turns might be the same early generation that gets interrupted each time.

The needed proof chain is:

1. Establish a finite clean/usable generation index g* in the fixed authority namespace, under the precisely inherited fairness and exclusion predicates.
2. The finitely many required `AUTH[r,g]` up through g* each become available eventually; hence they are all cached after a finite time. No uniform authority-service delay is assumed.
3. Show that, with these authorities cached, an uninterrupted retry either already forms a complete positive result or reaches g* within a finite bound. Use finite quota, fixed Delta, per-term progress checks, complete bundles, and the actual recovery transition. Do not assume a Byzantine term runs promptly.
4. Show that the correct usable initializer actually enters and obtains the uninterrupted commit/FINAL interval. The public timeout proof must allow this interval instead of assuming it occurred.

Part I has explicit quantifiers for correct initializers beyond every generation index and permanent exclusion over a fixed finite identity set (`I:L185-L221`). Part II's summary wording and the retry reset must be reconciled with those quantifiers. If that reconciliation fails, S1–S3 do not yet yield L. This is a precise remaining obstacle, not a generic claim that adaptive windows cannot work.

### 7.3 Sparse recovery waits can defeat the clock argument until this is proved

The previous concrete execution remains the adversarial test: silent generation 0 freezes; a new required recovery authority arrives only near the current cutoff; a correct initializer genuinely enters and casts DATA votes before cutoff, while its timers/complete receipt finish afterward. With fresh r on each failure, this occurs forever even with fixed members and shared native state.

The clock can also stay far behind during each finite authority wait, then cross the finite AUTO threshold between those correct votes and receipt formation. For example, after a long finite wait choose recovery entry d, votes at d+4 and d+5, deadline crossing at d+5.5, and commits at d+6 and d+7. Every wait is finite, growing timestamps/upper lead hold, and each correct identity can take infinitely many real producing turns. The long waits recharge the available clock lag. Summing timestamp budgets does not contradict the unbounded real waiting times.

S1 and the finite-authority-prefix argument are intended to remove those repeated waits. The sum-of-budgets argument is legitimate only **after** that removal and a derived L covering first usable entry, regional result formation, raw publication and acceptance. It must also account for the existing online/keep-up, complete-message, and declared-load qualifiers; it cannot promise constant catch-up from an arbitrary long history.

## 8. Rule S4 — an explicit strict-cutoff HealthPolicy

One concrete candidate controller policy is:

```text
key = current verified G commitment and provenance
if one MSIG has already been submitted under this key:
    do not submit another discretionary MSIG while the same G remains current
else if the specified authorized health/intervention predicate holds:
    submit one future-dated MSIG targeting the current beta
    record the submission durably under key
```

The predicate and whether operational override is allowed require an explicit product/protocol decision. “Urgent” cannot be an unlimited uncounted exception while a liveness proof assumes the one-submission cap. At a fixed G this bounds future MSIG interruptions; already pending records are still handled exactly by the inherited interpreter. The policy does not revise targets, apply stale attached changes, reorder MSIG below AUTO, or postpone a valid matching submitted Force beyond its first crossing.

This is a submission/authorization policy for the already-honest controller, not an environmental assertion that interventions happen to become fair. It is also not the user's still-pending “queue Force until full quota” alternative. If the intended interface must accept arbitrarily many authorized hard interventions at the same G, this candidate policy is unavailable and the corresponding preemption obstruction remains.

STATUS/ON/OFF/CLEAR still require complete authenticated replay. A global-progress theorem needs a region actually eligible to serve, not a physically healthy region kept OFF forever. For persistent-eligible fairness, the named region cannot be repeatedly removed by OFF under the premise. The policy must not use unavailable/private health information as a canonical trigger or invent a bounded health-observation channel.

## 9. Rule S5 — service-round debt for useful regional fairness

S1–S4 target global progress. They alone do not prevent one slow eligible region's work from continually losing to other regions while G grows.

A concrete optional selector extension is a finite service-round ledger:

1. At a canonical round opening, initialize `round_debt` to the nonempty eligible region IDs at that authenticated position. Regions becoming eligible later join the next round. Membership of the cyclic region list remains the existing finite list.
2. A region discharges its debt only when at least one of its DATA blocks is accepted into G during the round. Selection, Ready, QC, raw inclusion and empty heartbeats do not discharge it. Empty **DATA** may count as useful regional production under the existing rule; request service remains separate.
3. New selections search only regions still owed service and currently eligible, in cyclic order, with self fallback. Current already-open sessions may finish their fixed quotas; no quota is rewritten.
4. Governance OFF removes that region's current debt prospectively. Temporary AUTO cooldown does not erase debt: wait if no owed region is currently eligible. Do not start a new round simply because an owed healthy region is cooling down.
5. When all debts are discharged or removed, open the next round from the current canonical eligibility snapshot. Keep this ledger across Force and beta changes.

Why this helps: if a persistently eligible region C never contributes, every other finite region can receive at most one debt credit in that unfinished round. Their already-open quotas are finite. Continued global growth therefore eventually requires C's credit, provided the global-progress argument applies to the restricted candidate set. New ON events cannot postpone an existing C debt by endlessly resetting the round.

This changes the selector; it is not already proved by NextLive. Under the full regional model, a fixed required region's authority/state/core obligations eventually work and the stable-G retry analysis can be applied. Under a broader model with permanently dead-but-ON regions, a debt can block after healthy regions receive credit. The current frozen universal regional premises do not cover that broader failure. Do not advertise this fairness rule as one-healthy-region fault tolerance against arbitrary dead eligible alternatives.

## 10. Rescued-request inbox: useful but a semantic change and incomplete ingress

The concrete v3.3/v3.4 request trace has x in complete late honest results, while fast Byzantine-led empty DATA advances G. Such results are already covered by raw publication even though their beta is closed. They can recover **requests**, without accepting the old DATA or its state.

A candidate inbox rule is:

1. Scan authenticated Espresso records in canonical position/within-record order. For each historically valid complete regional result, including records rejected for old beta or stale base, extract authenticated client requests and deduplicate by the application's canonical request identity.
2. Store an ordered persistent rescue inbox as replay state. An old receipt supplies provenance for this input recovery only. It never grants old-beta append permission or promotes old execution state.
3. For a proposed DATA B, use the already-finalized history cutoff named by `xi(B)` to determine its required rescue prefix. Verify application admissibility against B's actual parent state and require the oldest pending admissible rescue requests that fit, after removing requests already in the proposed ancestry/global prefix. A conflicting or currently inadmissible request is treated according to a fully specified native admission rule.
4. Keep the source record's strict-before-containing-reference relation. S2 ensures selection lower bounds eventually pass any fixed rescue position, so a Byzantine leader cannot keep an old reference forever to evade a permanent inbox entry while the chain progresses through sessions.

This rule blocks empty Byzantine batches once a required admissible rescue entry is in scope. It can yield the conditional statement: **if x enters the canonical rescue inbox, stays admissible, and G grows without bound through valid DATA, then the finite older rescue prefix cannot hide x forever.** The proof must use finite contents per authenticated prefix and the existing batch/admission constraints, not an unlimited-throughput assumption.

It is **not** merely a publisher optimization. Current batch policy is a correct leader's local queue rule (`V:L2345-L2364`); a Byzantine empty block can still be application-valid. Mandatory rescue inclusion changes DATA/application validity for every proposer and every correct Vote check. Even if implemented through the existing validity callback, the core's semantic contract changes and requires new safety/liveness review. In particular, one must revisit any use of “valid empty blocks” in the Part I liveness proof when the canonical queue is nonempty.

The general ingress gap remains: an actually entered honest producing turn need not form a complete receipt before retirement. A surviving local commit is not always a complete regional result. Thus existing assumptions do not yet establish that **every** correct-client request reaches this inbox. A result containing x may never be complete in executions beyond the specific late-FC trace. Nor does raw delivery of some other empty complete result imply delivery of x.

Appending uncommitted requests to an old EXPORT is not an automatic escape. The frozen service preserves the complete result's DATA/evidence, and an already recorded result satisfies that guarantee (`M:L181-L190`). A new request trailer, a duplicate carrying extra requests, or a standalone request transaction has no automatically inherited funded bounded submission guarantee. Such ingress needs a concrete derivation or a separately authorized change; it cannot be assumed.

There is a further reason not to import Part I's request proof wholesale: the frozen Part II model fixes voters per session, not a finite union of all future voters/local queue orders. “Finitely many predecessors in each local queue” need not give one uniform finite predecessor set across infinitely many new configurations. A canonical public inbox would solve that ordering issue once ingress is established; local queue fairness alone does not automatically do so.

## 11. Proof obligation tree and outcome status

| Obligation | Intended route | Present status |
|---|---|---|
| Deterministic prefix safety / strict cutoff | Recompute target r, selector floor, miss ledger, debts from complete history; beta still increments once and all old-beta appends remain rejected. | Requires fresh review of changed boundaries/index reuse, not a new source/fault assumption. |
| No post-opening-header deadlock | S2 permits complete current output with known opening header; covered publication drives any later header needed for the next regular boundary. | Concrete repair candidate; no new publication type. |
| Eliminate repeated fixed-G state and authority waits | S1 pins r on zero output; retain Prep/AuthCache; derive a finite usable authority prefix. | State coalescing is sound; exclusion/schedule interpretation and actual-entry argument remain unresolved. |
| Infinite AUTO cannot erase all output | After a derived finite L, increasing failed-window allowance and the existing upper-lead bound give the amortized contradiction. | Cannot be invoked before L is derived; sparse authority waits currently defeat it. |
| Infinite hard preemption controlled | S4 bounds submissions per fixed G; all submitted controls keep inherited first-crossing/target/priority semantics. | Requires explicit authorization of the HealthPolicy, not an assumption of environmental kindness. |
| Useful service for each persistent eligible region | S5 debt persists until accepted output; reduce starvation to failure of global progress in a finite restricted set. | Concrete selector candidate; broader dead-but-ON model remains outside scope. |
| Every client request included | Existing pending queues plus accepted honest opportunities, or rescue inbox with proved ingress and mandatory DATA validity. | Not established. Rescue fixes the specific complete-late-result trace but not general ingress. |

This is the best current concrete strict-cutoff rule set, not a complete paper-ready solution. S2 is independently useful. S1/S3 require an exact authority/exclusion/admission resolution. S4 requires an explicit permitted controller policy. S5 and the rescued inbox must not be advertised as preserving all selector/core semantics unchanged. The full goal must remain open until those obligations and a fresh adversarial review are completed.

Only this analysis Markdown file was written. All old protocol sources and frozen assumptions remain unchanged. Validation consists of source inspection, explicit event traces, and proof-obligation analysis; no executable model check or formal proof validation was run.
