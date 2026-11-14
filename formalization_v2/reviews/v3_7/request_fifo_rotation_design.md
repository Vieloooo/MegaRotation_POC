# FIFO request inclusion through authority rotation: scoped design analysis

Status: design exploration only, 19 October 2026. The v3.7 manuscript and the frozen v3.1 Model/Part I files are unchanged. This is not a complete request-liveness repair or a new theorem for the current manuscript.

## Conclusion

A persistent per-region initializer cursor addresses a real defect in the present counterexample: it can prevent every fresh slot from restarting with an arbitrarily chosen Byzantine initializer. There is also a useful stronger regional lemma: when a session's initial leader is correct, its first proposed DATA cannot be bypassed by that session's later accepted DATA, even when equivocation cancels its initial commit timer. Thus a correct initial leader with oldest pending request `x` can make **positive accepted service of that session** imply inclusion of `x` in the accepted ancestry.

Two distinct obligations prevent claiming the requested repair under the entire frozen model:

1. The current protocol does not specify a programmable producer of `AUTH[r,0]` that must follow such a cursor. Defining that producer would be an explicit algorithmic extension. Merely declaring global initializer fairness would strengthen the assumptions.
2. Locking a finite roster only until each initial leader has received one positive accepted session does **not** suffice under arbitrary allowed future membership changes and per-node FIFO. Fresh all-correct rosters can each have enough finite local predecessors of `x` to finish their finite service round without including `x`. A concrete execution below needs neither Force nor slow authority/native preparation.

The scoped positive result is therefore: an implemented source-authority cursor gives FIFO request liveness for a request that reaches a correct identity receiving infinitely many credited initial-leader sessions, with retained local queue state. Fixed continuing membership is one sufficient case. The frozen text does not establish this identity-level continuity for unrestricted future memberships. This is not an impossibility theorem for every FIFO-preserving protocol or every membership policy.

## Sources and frozen boundaries

The analysis uses the current sources, not an earlier memory-derived protocol verdict:

- `sync_hotstuff_rr_part2_rotation_v3_7.tex:238–281`: immutable `AUTH[r,g]`, session snapshots, and numeric-`r` authority identity; an already assigned authority cannot be rebound.
- `...v3_7.tex:360–378,518–537,2221–2240`: session-fixed weights/fault bound, actual correct turns, eventual authority/native preparation, request dissemination, and local first-receipt FIFO with removal only at `G`.
- `...v3_7.tex:691–719,919–955,1061–1069,1207–1302`: DATA quota, initial/recovery admission, complete validation, retirement gate, and recovery locks.
- `...v3_7.tex:1746–1766,1791–1823`: persistent preparation and authority fetching; zero-output Force reuses `r`, while positive accepted output moves to `r+1`.
- `sections/rotation_v3_7_liveness.tex:17–59,63–154,291–440,442–478`: inherited fairness, fixed-slot progress, repeated regional service, and the present request counterexample.
- `sync_hotstuff_rr_part1_core.tex:173–220,268–316,431–525,870–944`: authority/fairness scope, evidence-first quit handling, entry locks, protected correct initializers, and the original request argument.
- `sections/rotation_v3_5_health.tex:3–16,18–45,100–115`: the existing honest committee is explicitly programmed to issue controls, with durable replay and adoption rules; this does not already grant it authority-issuance powers.

The frozen baseline Model is `revisions/v3_1_frozen/model_and_assumptions.tex`, including its progress paragraph at lines 295–312. Its source describes an eventual AUTH service, not an AUTH delay bound. It does not impose a finite union of all future memberships. Part II's local FIFO expressly permits different node orders; Part I's stronger common admission order cannot silently replace it.

## Why initial leadership is materially different from late recovery leadership

The current counterexample first accepts Byzantine empty DATA, then delays a fresh recovery authority until a correct recovery initializer has too little time before Force. The correct proposal can be discarded because the duty has already obtained positive credit and moves to a fresh authority namespace.

For a **correct initial leader** `L(beta,r,0,0)=h`, the argument can instead use ancestry. Let `B` be its first DATA, and suppose the session eventually contributes its first DATA to `G`.

### Initial-leader ancestry lemma

Under the existing complete Ready/entry and regional communication rules, every DATA of this session that can become globally accepted descends from `B`. Consequently, if `x` is in `B` or in the inherited ancestry of `B`, positive accepted service of the session includes `x`.

The proof is not “a correct proposal necessarily survives its timer”:

1. A quota is at least one, so an initially entered session begins with DATA rather than DRAIN. The correct initial NEWVIEW/ANCHOR exchange and proposal obey the existing protected-entry timing. With no earlier genuine quit, its DATA reaches each correct member before the applicable term-zero progress check. Message-before-timer handling covers equality.
2. A correct initial leader cannot produce DATA or NEWVIEW equivocation. Term-zero BLAME cannot legitimately freeze this protected correct execution. Before a first quit, a later term is admitted only through the terminal QC of term zero. That QC has `B` in its exact ancestry, whether or not any timer for `B` has expired.
3. Therefore any first genuine quit that could interrupt this initial execution has an admitted later-term witness containing that terminal QC. A correct replica that enters recovery has validated the quit's own admission witness and knows a QC descending from `B`. Evidence-first processing may prevent a fresh DATA vote or cancel a timer; it does not erase the QC needed to authenticate the quit. The highest-QC entry lock is at least this rank.
4. Induct on later generations. Every already existing higher-rank DATA/DRAIN QC descends from `B`. An ANCHOR QC has a correct signer, and that signer's lock test prevents returning to the lower initial anchor or another QC below the retained rank. Thus every newly certified opening and fresh descendant still extends `B`. An early equivocation of a later recovery initializer does not erase locks from previous generations.
5. A FINAL receipt is sound for its hash-bound regional ancestry. Contiguous global acceptance cannot accept a later session DATA while omitting its first DATA `B`. A DRAIN contributes no global block; a DRAIN receipt covers terminal DATA whose ancestry still includes `B`.

If Force closes the version before such acceptance, the lemma asserts no accepted service. The first DATA is then globally rejected under the unchanged cutoff. With zero accepted session output the retry retains the same numeric `r` and the same per-region initial-leader assignment, while receiving fresh `(beta,r)` state. If Force selects another region, that other region uses its own assignment and cannot discharge this region's initializer token.

This coverage lemma is stronger than a timer-survival statement and does not require a new publication input. Its eventual-service application still depends on the existing global/regional progress premises. It must not be generalized to arbitrary correct recovery or ordinary-term leaders: their initial ancestry can already contain positive output chosen by earlier Byzantine leaders.

### FIFO consequence for one persistent correct identity

Suppose correct identity `h` has received persistently admissible, individually fitting `x`, retains its local queue, and has only finitely many local predecessors of `x`. Consider its credited initial-leader sessions after this receipt. If `x` is not already in accepted ancestry, the first DATA either includes `x`, or includes at least one still-unaccepted local predecessor that prevents `x` from fitting. A predecessor already in the proposed ancestry is also accepted before this session can receive its first accepted-DATA credit.

The ancestry lemma therefore makes each such credit accept `x` or discharge a distinct member of a fixed finite predecessor set. New arrivals cannot join that set. Infinitely many credits for this same correct queue owner force `x` into `G`. No common queue order, authority delay bound, or acceptance deadline for every individual attempt is used.

## A concrete source-authority cursor, if that extension is authorized

This is a proposed algorithm for a designated authority producer, not a property that the present external source is already assumed to satisfy.

For each region `i`, retain a finite membership-round identifier `e`, a fixed roster/configuration for that round, a deterministic finite list of its eligible initial-leader identities, and a cursor `k`. The pair `(e,k)` is an **assignment token**, not a replacement for the engine identity `(beta,r)`. The producer must also retain its authenticated canonical replay cursor and an immutable journal mapping each issued `AUTH[r,0]` and region to its assignment token.

1. **Issue.** For each not-yet-assigned numeric slot `r`, produce `AUTH[r,0]` whose region-`i` initial leader is the current token's identity. Keep that region's round snapshot until its round finishes. Existing positive finite quotas and DATA/DRAIN rules remain unchanged. Complete later-term and recovery schedules so that the original fairness and permanent-exclusion conditions still hold. Do not wait for this slot's own accepted output before issuing its authority.
2. **Credit.** Only the first DATA of a session actually accepted into canonical `G` is eligible. It can credit the token written into that session's `AUTH[r,0]` for its assigned region. Credit a token at most once, and only while it is the current token. Ready, a proposal/QC, regional commitment, a complete but rejected result, Force, or DRAIN alone does not count.
3. **Advance.** On that credit, advance to the next identity of the same finite round. Changes of `beta`, speculative regular successors, cooldown, and administrative status do not silently advance or delete an unfinished correct identity's token. An exposed identity may be excluded in future authorities according to the original exclusion rule; this must be authenticated and cannot become discretionary pruning of correct identities.
4. **Finish the round.** Only when every non-pruned token in the finite round has been credited may a future, not-yet-issued authority adopt a new membership round. This is finite-round membership retention, not a permanent global membership freeze.

The assignment fields could be committed inside authority records or derived from an authenticated issuance journal. The design must choose one concrete representation. A local unverifiable cursor is insufficient to define what every source record means.

### Pre-issued authorities and source ordering

`AUTH[r,0]` fixes schedules for every region before the selected region is known, and normal production can run ahead of `G`. Consequently, the producer must not overwrite an already finalized `AUTH[r,0]` when the cursor changes. Multiple speculative slots may carry the same token. Only the first accepted session credits it; a later session carrying a stale token does not advance the new token.

For a positive progress argument, a producer must eventually process each accepted-service event and issue new authorities from its updated durable cursor. At any finite processing time, the finalized/pre-issued backlog is finite in the ordinary non-Zeno execution model. Thus old-token assignments create a finite lag, not an infinite reset. This requires an actual progressing producer action; “choose the next fair initializer” is not a proof. A permanently lagging producer that keeps issuing old-token authorities would defeat the proposed property.

Do not make `AUTH[r+1,0]` wait for the first accepted DATA of session `r+1`, which would be circular. Nor is it necessary to wait for session `r` to become global before normal handoff: future authorities can keep assigning the same outstanding token until canonical credit is observed. This preserves speculative regular operation, while allowing duplicate tokens in a finite backlog.

If a region is continuously eligible and satisfies the serving premises, existing repeated regional service plus finite stale-assignment lag can credit its current token. A nonempty unfinished finite round can then advance through its remaining identities. A region removed permanently by governance is outside that regional-service conclusion; its cursor must not be treated as serviced. Pending STATUS, CLEAR and MSIG records retain their actual first-crossing semantics and never disappear merely because a membership round is locked.

### Why this is not already supplied by the model

The Model specifies what immutable authorities contain and that required evidence eventually arrives. It does not identify an honest programmable issuer that reads canonical Espresso acceptance and controls future initial leaders and membership admission. The multisignature assumption covers authorized STATUS, CLEAR and MSIG FORCE, not AUTH generation. The existing health policy's verified cursor provides a possible pattern, not a grant of this extra authority.

There are two legitimate implementation routes, each requiring explicit scope:

- Program the actual authorized source issuer to execute the cursor and document how its observed accepted history and issuance journal are authenticated.
- Program an authority contract that checks authenticated evidence for the relevant acceptance and assignment transitions. The means by which that contract authenticates Espresso/canonical interpreter state must itself be specified; it does not follow from the fact that L1 is a bulletin board.

Neither route needs a numerical authority delay bound merely to express the algorithm. Both add real protocol behavior. Rejecting noncompliant authorities downstream, while leaving the original producer unconstrained, is not equivalent: the original source can finalize an immutable noncompliant record, after which receivers wait forever for an impossible replacement. An existing deployment also needs a migration point and a finite inventory of pre-issued authorities; their identity and old membership cannot be retrospectively rewritten.

## Finite membership rounds still admit FIFO starvation

The following execution defeats the stated policy class even assuming a perfectly implemented, immediately updated authority producer. It is independent of the delayed-authority counterexample.

Use one continuously eligible region and a batch capacity of one request. Set every session's DATA quota to one. For each membership round `n`, install a fresh roster of `m` correct identities. There are no Byzantine members, no equivocation, no Force, no health control, no unavailable native state, and no delayed authority. Every published result is promptly accepted. All configured fault and certificate bounds hold.

Before the first service of that round, every member has received the pending request `x`. Its local FIFO queue is

```
y[n,1], y[n,2], ..., y[n,m], x
```

The `y[n,j]` are distinct admissible requests, not accepted in any earlier round. They arrive at these new identities before `x`, so this order respects first-receipt FIFO; each queue has only `m` predecessors of `x`. The queues may even agree inside the round.

The cursor assigns each of the `m` correct identities one initial-leader session. The first accepted session includes `y[n,1]`, the next includes `y[n,2]`, and so on. After `m` accepted sessions every token has received positive service and the round closes. At that point `x` is oldest in the old members' queues, but membership changes before they receive another session. The next fresh roster already has its own new finite prefix `y[n+1,1],...,y[n+1,m]` before `x`. Repeat forever.

Every membership round is finite and fully serviced. Every request queue has finitely many predecessors. Request `x` reaches all correct members of every serving roster before their first service. `G` grows forever, all initializers are correct, all actual turns are productive, and all service bounds hold. Yet `x` never enters `G`.

This avoids the weaker objection “the request had not reached the new members yet.” It also avoids treating empty DATA as useful request service: every accepted DATA here contains a genuine request. The obstacle is the absence of a finite predecessor set attached to a queue owner who continues receiving service.

The original sentence “correct leaders receive repeated turns” is not formalized as infinite future service for every identity after it leaves membership. The current fixed-slot proofs expressly avoid a finite union over future configurations. If that sentence is intended to impose identity-level infinite service despite removal, it must be stated and reconciled with membership semantics; it cannot be silently imported to dismiss this execution. Giving each finite roster any fixed finite number of complete cycles only enlarges the finite prefix needed for the same construction.

More generally, this obstruction applies to request-oblivious membership policies that permit an entirely fresh roster after a finite accepted-service budget for the current roster. Choose a finite predecessor prefix exceeding that budget. It does not cover a policy that explicitly retains a correct queue owner until a captured pending-request obligation has been discharged, or a policy with proved FIFO-preserving pending-state transfer. Those would be different algorithms whose request awareness, representation, membership admission, and completion rules must be established.

## What would be sufficient, and what remains open

The cursor plus the ancestry lemma supports a complete FIFO argument under any proved mechanism guaranteeing an `x`-aware correct queue owner infinitely many credited initial-leader sessions. Examples of sufficient conditions are eventual stable membership with continued serving, or a finite recurring identity set with retained local queues and a recurring correct member that receives `x`. These are conditional cases, not newly adopted frozen premises.

A fully algorithmic alternative could investigate membership admission tied to finite pending-queue obligations, or a transfer protocol that preserves the relevant finite predecessor relation across changing identities. Neither is specified here. An arbitrary signed “done” from a member is not enough when that member may be Byzantine; demanding every member certify an empty queue may instead block churn forever. Moving an inherited request ahead of a new node's existing queue would change the existing first-receipt FIFO. Introducing a bare-request public queue would exceed the original complete-result publication guarantee. These issues need separate resolution before such an extension can be called a repair.

Keeping one membership forever would sidestep the counterexample, but is not the proposed finite-round churn policy and is not adopted. Likewise, quota changes, empty/nonempty service labels, or a larger tolerance do not supply the missing identity/queue continuity.

## Verification status

This note checks source-level dependencies and gives explicit executions. The initial-leader ancestry lemma and producer policy remain design claims for independent review; no full protocol/model-checker implementation was run. The only written file is this design report. The manuscript, assumption text, FIFO selection, strict cutoff, normal quotas, authority identity, DATA/DRAIN engine, and pending-control interpretation were not changed.
