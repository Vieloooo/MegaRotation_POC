# Protocol-only liveness repairs for a later revision

Date: 2026-10-18. Design analysis only; no draft, formal proof, or protocol source was authored or edited. The source and line-number conventions are those in `preflight_liveness_scope.md`. This document is not the later fresh adversarial review.

## 1. The design choice that cannot be avoided

The original goals remain:

- Unbounded accepted global DATA progress whenever at least one actually healthy region can serve.
- Service for every persistently healthy, eligible region, if the protocol claims this stronger fairness property.

The frozen environmental premises cannot simply be strengthened to recurring successful intervals, bounded state recovery, bounded authority availability, bounded Manager scheduling, bounded timestamp lag, or a stronger committee fairness promise. The phrase “some sustained service interval” at `II:L496-L499` remains existential.

There is a **conditional impossibility**, not an impossibility for every conceivable protocol: retain immediate irreversible branch revocation at every valid Force, finite expiring service attempts, no bound on preparation/recovery per attempt, and only the existing finite-interval service premise. Then no choice of finite window constants, or adaptive finite timeouts alone, can guarantee the goals. A lawful infinite sequence of replacements can make every result obsolete before acceptance. The all-correct construction in the preflight already establishes this for the present fixed windows.

A protocol-only repair must therefore change at least one of those operational rules. The strongest promising directions are:

1. A **protected service obligation**: once an eligible session receives a particular retained anchor, subsequent controls cannot keep canceling that obligation before a result is accepted. Controls can request or prepare the next switch, but their effective replacement is progress-gated or otherwise preserves a completable obligation.
2. **Durable acceptance permission** for already authorized results: a compatible result remains admissible after a control change, subject to the current global prefix, so infinite Force churn cannot invalidate all work.

Both change the semantics of Force. Neither weakens the requested liveness goal. If immediate hard revocation is itself non-negotiable, the stronger liveness goal cannot be obtained from the frozen premises. Calling a new recurring-service premise a “protocol condition” would not resolve that conflict.

## 2. Keep three interpretation questions explicit

### 2.1 Actually healthy is not the ON flag

The existing model has a Byzantine-weight bound and correct-to-correct synchronous processing in **every** regional snapshot (`II:L330-L372`). Required authorities and recoverable state eventually become available (`II:L500-L501`). A wholly dead regional honest quorum, permanent loss of its required state, or arbitrary asynchronous correct replicas are not valid counterexamples under that entire model.

Thus there are two distinct target readings:

- **Within the full frozen model:** one eligible serving region may be enough operationally, but every regional engine that is assigned and continues to execute still has the frozen fault/network/authority/state/fair-schedule premises. A slow eligible region is eventually usable; there is no fixed time bound.
- **A broader one-healthy-region failure model:** every other eligible region may be permanently crashed, permanently unsynchronized, or never recover state. This is not currently licensed by the universal regional premises. No proof under the frozen model establishes that broader fault tolerance merely by using the word “healthy.”

A repair may provide useful behavior in the broader setting, but must not certify it using premises the setting violates. Conversely, do not use a completely dead eligible region as an in-model impossibility example when all frozen premises are being claimed.

### 2.2 A healthy region must have authority to serve

A region can be healthy but permanently governance OFF. If every healthy region is OFF forever, a protocol that continues honoring OFF cannot guarantee production merely from physical health. The existing progress premise supplies an eventual **other eligible** region at a waiting boundary; it does not identify that region with the healthy one in the broader failure model.

The one-live theorem must say whether live means “healthy and eventually/persistently eligible” or whether governance OFF is deliberately overridden for emergency service. This is a real authorization choice. Silently equating health with ON or silently ignoring OFF is not a repair.

### 2.3 Selection fairness is weaker than useful service

Choosing C infinitely often does not imply that C ever contributes a globally accepted DATA block. Startup can always lose a finite deadline, or faster competing candidates can always win. A fair-service claim must identify its event: e.g. a completed protected quota or at least one globally accepted DATA block from each persistently healthy eligible region. Keep this separate from chain-height growth and request inclusion.

## 3. Obstructions that parameter tuning cannot remove

### I1. Finite service does not supply infinite progress

All frozen premises can hold through an arbitrarily long finite successful prefix. Afterward, give each attempt a finite completion time later than its finite expiry. Old engines still finish eventually, and raw publication still obeys `T_pub+T_E`; their closed-version output is rejected. The committee can issue no interfering MSIG at all in the fixed-window example. This does not require a failed source service.

For an adaptive policy that assigns finite budgets `b_1,b_2,...`, choose finite per-attempt preparation times `d_n>b_n`. Eventual state/authority availability holds for every attempt. If each timeout cancels or changes the anchor/session obligation so that completed old work can never be accepted, the adversary does not need a single infinite delay. Taking exponentially increasing `b_n` only defeats a fixed unknown bound, not an unconstrained sequence `d_n`.

This diagonal argument is conditional on genuinely fresh/canceled obligations. If the protocol retains the **same** fixed-anchor preparation task across attempts and eventually admits its result, the construction no longer works; that is why durable preparation/acceptance is a substantive repair.

### I2. Timestamp windows need not provide real-time service

The numeric clock premise is only nondecreasing timestamps and `theta_q <= t_E(q)+epsilon_E` (`II:L417-L420`). At a public opening the timestamp can be far behind real time. The next block can jump forward across any finite timestamp budget while still satisfying the lead bound.

Adaptive larger timestamp windows have the same limitation if opening lag is unbounded and is not tied to a verified real-time anchor. For any finite selected `W_n`, the history can accumulate enough lag before the opening and then catch up. Growing timestamps and bounded inclusion do not prohibit that pattern. Ordinary empty chain-namespace blocks can supply growing authenticated history throughout.

The current protection against recursively expiring every successor **within one block** is correct (`II:L1761-L1767`), but does not prevent one expiry per later jump. The missing property is not solved by processing the same block more carefully.

This clock observation alone is not a proof of infinite global starvation against every adaptive rule. The upper-lead bound constrains cumulative timestamp advancement; a permanently fixed real-time response bound, a budget that never resets across failures, and retained work could support an amortized argument that eventually some real service interval is long enough. The frozen model does not give that fixed end-to-end response bound. The decisive general obstruction is the permitted per-attempt delay sequence in I1, or independent hard revocation in I3, rather than a claim that the clock can spend an infinite initial lag budget for free.

### I3. Infinite valid hard revocation defeats AUTO-only repairs

Suppose AUTO is disabled or delayed until Ready. If every matching MSIG still takes effect at its first timestamp crossing and irreversibly rejects all later old-version output, the committee can lawfully preannounce a sequence of Forces that cuts each new attempt before its usable result is accepted. Advance announcement only makes the cutoff predictable; it does not stop the cutoff from being too early for the new attempt.

The committee's honesty/authorization/announcement premises must all be respected. The only finite-interval service quantifier does not become an infinite one because the committee is honest. If “leaves working regions enough uninterrupted service time” is intended to forbid every such tail, that intended universal or recurring quantifier must be shown in the frozen source; it cannot be introduced now as an unstated meaning of honesty.

Therefore every proposed liveness repair must say what **effective MSIG Force** does to a protected pending obligation. A repair affecting only Soft/AUTO is incomplete.

### I4. Fairness cannot be obtained from fastest-ready races

Under the frozen eventual-only restoration condition, a persistently healthy region may finish any particular fixed-anchor restoration at a finite time without ever catching a moving sequence of newest anchors before faster regions advance it again. Canceling each old candidate and choosing the first latest-anchor Ready response can starve that region forever while G grows.

A fair-service mechanism needs either a protected fixed-anchor reservation after some qualifying event or a way for earlier prepared work to remain useful. “Every region is polled” and “every fetch eventually completes” are insufficient. A new uniform catch-up-rate assumption would be a model strengthening.

In the broader failure model with potentially permanently dead eligible regions, an unconditional wait on the next fair region can destroy global liveness. A readiness certificate plus protected service can help only if that certificate's soundness guarantees the subsequent progress needed for the reservation. Under the full frozen regional premises that continuation may be derivable; under arbitrary post-certificate regional death it is not.

## 4. Candidate R1 — strictly advancing canonical selection references

**Rule direction.** Require the terminal selection reference for a completed regular session to have height strictly greater than its boundary's `q_open`, rather than merely at least it. The next session inherits this strictly later bound. Alternatively, choose the normal successor from the canonical Espresso position where the terminal receipt first becomes public and acceptable. The latter changes the direct fast-handoff path more substantially.

**What it fixes.** The preflight A/B loop using `xi=q0` forever cannot occur. In an infinite normal-session sequence the reference heights cannot remain below a fixed later ON position. Once all references are after that position, the cyclic rule can no longer exclude C solely using its old OFF snapshot.

**What it preserves.** Historical deterministic selection; signed source references; no receive-time-local choice; no assumption that all observers have the same latest cache.

**What it does not fix.** Same-region exclusion, selected-region startup loss, infinite Force churn, useful-service fairness, or real-time deadline calibration.

**Proof obligations.** Specify exactly which terminal block is subject to the strict bound, including recovery/DRAIN and quotas. Correct leaders must wait for or use a qualifying finalized header without applying a nondeterministic local freshness test. Eventual Espresso growth can provide a later header, but not a fixed wait bound. A full fairness proof must include Force selection and useful accepted output, not just infinitely increasing normal-reference heights.

## 5. Candidate R2 — same-region fallback and non-destructive cooldown

**Rule direction.** Search the finite cyclic list for other eligible regions first; if none exists, consider the source itself. For an ordinary quota-completion boundary, immediately renew a new logical session when the source is the sole eligible region. For AUTO, preserve the source's actual cooldown and allow self-renewal after its canonical expiry if no alternative exists. Do not reset or extend a cooldown merely by replaying a duplicate or by resolving the same pending boundary again.

**What it fixes.** The rule-level singleton deadlock and the fact that CLEAR/expiry cannot currently overcome the “other region” exclusion. New sessions retain fresh identities and the exact anchor; old voting state is not reused.

**What it does not fix.** An infinitely renewed source can still lose every short startup window; OFF cannot be overridden implicitly; global progress does not follow if all healthy sources remain ineligible. Finite cooldown expiration is eventual under unbounded timestamp growth, not bounded in real time.

**Frozen-premise issue.** The old “eventually another eligible region” condition may remain a sufficient legacy premise, but the new singleton theorem must not rely on it. Proving a property using fewer existing premises does not require changing the old assumptions. Claiming a broader fault model does.

## 6. Candidate R3 — overlap-tolerant acceptance with a proved retry loop

**Rule direction.** Permit an EXPORT whose expected base is an authenticated ancestor of current G, provided its overlapping DATA has exactly the same hashes, positions, session provenance, and valid evidence as G. Validate the full record; append only the as-yet-unaccepted continuous suffix. Count each newly appended DATA once. A conflicting overlap is rejected. A record with no new DATA is a no-op, not a clock reset.

**What it fixes.** Competing honest publishers need not rebuild a complete export merely because another submission accepted its matching prefix first. A full future interval may become usable without requiring one more base-update round if its overlap remains exact.

**Stable-base argument to develop.** Either other acceptable records advance G infinitely often, already establishing chain progress, or G eventually stabilizes. In the latter case, correct workers eventually see the stable base and can submit its next continuous complete interval; inclusion/visibility is bounded for that submitted record. This is more informative than assuming all stale-base retries eventually win.

**What it does not fix.** Under existing closure rules, a closed branch remains ineligible even when its data overlap agrees with G. Overlap tolerance alone cannot rescue the C1 results. It also cannot turn a fork at the next height into a valid extension.

**No hidden service extension.** Keep `T_pub` attached to complete regional results. An acceptance retry obligation is protocol work; do not rewrite the frozen service assumption so it automatically bounds the whole retry sequence. Correct worker scheduling/persistence needs a concrete state machine using already available processing obligations, not an unexplained fresh fairness axiom.

## 7. Candidate R4 — progress-protected service and queued Force

This is the most direct candidate **within the full frozen regional model**, where an assigned continuing engine has eventual authority/state/leader progress. It is not yet a certified design.

**Canonical structure.**

1. Preserve one replay-defined protected obligation `(ticket, region, authority, retained anchor, required positive output)` at a time. Its anchor and authorization cannot be continually replaced by later forecasts or Soft failures. A quota is finite and at least one DATA is required.
2. Treat additional Force records as ordered pending switch requests. Honor their finalized authorization and target checks, but define their **effective replacement** at a progress boundary or another explicitly proved transition. Repeated requests cannot continually invalidate the same protected obligation.
3. Prepare potential successors concurrently. Old-engine cleanup never gates preparation or activation of an independent new obligation.
4. Upon completing the positive protected output, advance G and the fair-service cursor, then apply the canonical next authorized switch/selection. Every protected completion is actual global progress.

**Why it can address finite-vs-infinite service.** The protocol stops relying on every external finite window being lucky. Eventual state/authority/entered-correct-leader progress can complete each fixed obligation; raw bounded publication and a correct stable-base acceptance path can then advance G. Repeating the protocol, rather than assuming repeated lucky intervals, is the intended source of arbitrarily many completions.

**The explicit cost.** A submitted MSIG's advertised timestamp becomes an earliest eligibility/request time, or later MSIGs are queued behind the protected action. It is no longer universally true that every matching MSIG irreversibly closes its target at the first crossing. That semantic change must be visible and reviewed. The existing numeric AUTH window fields can remain immutable monitoring thresholds, but cannot retain their original universal immediate-revocation effect if they are allowed to be too small.

**The hard case.** In a broader model where an assigned eligible region may be permanently dead, protecting it forever would block the system. It is invalid to cite full frozen regional liveness to solve that case. A protocol needs a qualified readiness/result event before protection, or a durable-output failover design such as R5. The readiness event must actually guarantee what the protected action needs under the intended fault model.

**Fairness.** Use a persistent cyclic/FIFO service obligation derived from authenticated eligibility, rather than “first Ready always wins.” Do not reset a healthy region's waiting credit after unrelated Force or candidate changes. A complete proof must show that every persistent healthy eligible region can acquire a qualifying ticket, the ticket cannot be canceled forever, and its completion contributes globally accepted DATA. This also requires removing historical-reference starvation through R1 or another canonical fresh selection point.

**Why this is not a new assumption.** Restricting how an honest protocol interpreter applies controls is a protocol change. Assuming that the committee happens to stop issuing inconvenient controls is an environmental strengthening. Keep that distinction explicit.

## 8. Candidate R5 — durable authorized work across control changes

This direction targets both AUTO and MSIG churn without assuming every attempt finishes before its deadline.

**Rule direction.** A control change prevents new assignments under the obsolete production plan, but does not automatically invalidate every already-authorized finite output obligation. Keep a replay-authenticated ledger of durable grants. A later EXPORT can satisfy such a grant if its ancestry and overlap exactly extend current G, all original receipt/authority/application checks succeed, and its remaining quota/grant has not been consumed. Canonical Espresso order decides which compatible result appends first. Subsequent incompatible results fail the overlap check.

**Why the global-stall argument changes.** If G stops advancing and at least one continuing healthy region retains a complete-able grant at that fixed anchor, its eventual complete result cannot be killed solely by infinite control churn. It becomes a candidate for acceptance under the unchanged raw delivery bounds. If another grant repeatedly wins first, G is advancing instead.

**Necessary durable preparation.** Do not cancel the only healthy region's fixed-anchor state restoration each time a new candidate version appears. Cache by anchor identity, retain the finite grant, and permit the resulting valid work to discharge that grant. A bare cache is insufficient if the acceptance rule still rejects the associated authorization after every Force.

**What this changes.** The current statement “closed-version outputs remain archival” (`II:L1559-L1564`) is no longer universally true for outstanding grants. A forced source or an OFF region may still contribute previously authorized work after the control transition. That is a policy/authorization tradeoff, even if deterministic prefix safety can be maintained. Do not describe this as merely a retry optimization.

**Security obligations.** Define grant creation/consumption, allowed block identities, original-snapshot verification, upper quotas, replay order, and overlap checks. Preserve the current G exactly. An arbitrary old regional receipt is not enough: its grant must authorize this particular extension. Multiple preauthorized branches cannot be treated as one regional engine; locks and identities stay separate. Existing key-compromise/retention premises continue to apply while grants remain acceptable.

**Fairness limitation.** First-valid-wins durable grants give a plausible global-progress argument, but can still let a fast region win every race. Add a persistent fair service mechanism or protected qualified ticket if per-region useful service is required. Do not claim that retaining C's candidate implies C's output will ever be selected.

**Resource obligation.** Avoid retaining an unbounded number of active BFT engines merely because controls arrive forever. Coalesce compatible preparations by exact anchor, preserve a bounded set of current per-region obligations where possible, and keep archival proof/data duties separate from live engines. The proof cannot assume infinite processing capacity while simultaneously relying on the frozen complete-message/load bounds.

## 9. Candidate R6 — readiness-based timing and real-time attestations

Deferring AUTO until authenticated readiness can eliminate the unbounded **pre-Ready** part of a startup budget. It is not a complete liveness design on its own.

- A selected but never-ready region must not be the only candidate forever in a broader failure model. Conversely, repeatedly replacing every unready candidate still permits finite delayed completions to miss every attempt.
- Initializer fairness and recovery are eventual, not uniformly bounded. A Ready snapshot alone does not imply that the next chosen leader commits within a fixed known BFT budget.
- MSIG can still preempt all Ready attempts unless it also respects the durable/protected work rule.
- A Ready event received locally cannot restart the public clock nondeterministically; replay needs one canonical authenticated event and target anchor.

One possible clock-specific tool is a quorum-signed real-time anchor: the shared global clock and a certificate with weight `>F` ensure at least one correct signer checks a signed timing statement. A protocol can potentially define a future deadline whose lower real-time service duration follows from that statement and the Espresso upper-lead bound. However, this is a new certificate protocol to design and verify, not an existing fact about `xi`, L1 timestamps, or Espresso finality. The source expressly says references are not commit clocks.

There is also a transport trap. The frozen `T_pub` guarantee covers **complete regional results**, not every newly invented Ready/activation message. `T_E` applies once an admissible complete record has actually been submitted. Do not assume a Ready certificate reaches a funded Espresso submission path within `T_pub` without deriving that behavior from the existing model, e.g. by including it in a covered complete-result package or specifying the necessary protocol path. Merely adding a new “bounded Ready service” would violate the assumption freeze.

Therefore real-time certificates can repair the timestamp mapping, but cannot by themselves resolve arbitrary eventual preparation, endless hard revocation, or fair service.

## 10. What not to call a complete repair

| Proposed change | Actual effect | Remaining obstruction |
|---|---|---|
| Set `W_start` to a much larger constant | Removes some short-startup failures | No uniform state/recovery bound; timestamp lag; hard revocation |
| Double W on each failure | Learns an unknown fixed bound if one exists | Frozen premises permit a different larger finite delay/lag on every canceled attempt |
| Start W at local Ready | Gives a locally convenient timer | Not deterministic public replay; no public readiness transport defined |
| Disable AUTO until Ready | Avoids premature AUTO before that event | Dead candidate in broader model; infinite MSIG; eventual-only regional recovery |
| Always pick the fastest Ready region | Helps aggregate throughput | Can starve a persistently healthy slower region |
| Keep downloading old candidates | Avoids some wasted work | Results remain useless if every old grant is revoked before acceptance |
| Permit self in NextLive | Fixes singleton selection exclusion | Same source can still miss every window or stay OFF |
| Strictly advance xi | Fixes permanent historical OFF starvation | Does not guarantee output after selection |
| Accept matching EXPORT overlaps | Removes some stale-base retries | Does not undo closed-version rejection |
| Assume “enough time” every round | Makes a proof straightforward | Changes the frozen existential/unbounded service assumptions |
| Have the honest multisig choose better | May be a sensible implementation policy | Requires explicit protocol rules and justified observations; honesty alone is not the missing recurring-service theorem |

## 11. Recommended design path and remaining impossibility boundary

Implementing R1, R2, and R3 is useful protocol work, but that package alone does not meet the original full goal. The later revision needs an explicit decision between R4, R5, or another mechanism that defeats endless invalidation of the only productive work.

For the **literal full frozen model**, R4 is the most economical conceptual path: every eligible selected engine has eventual state/authority/fair-leader support, so protecting a fixed finite service obligation can use those existing premises instead of inventing a bounded startup premise. A cyclic service schedule plus advancing canonical references can then target useful regional fairness. This still needs a complete regional-to-global composition proof and a concrete receipt/retry/activation state machine.

For a **broader model with only one healthy region and potentially permanently dead eligible alternatives**, R4 needs qualified protection or R5-style durable competing work. The present frozen assumptions do not describe that broader failure envelope or certify a readiness detector for it. A slow-vs-dead ambiguity cannot be solved by relabeling a timeout as proof of regional failure. Any claimed extension must state which new behavior is obtained without relying on violated universal regional premises.

If the user requires both (a) immediate irreversible revocation by every authorized MSIG at its scheduled first crossing and (b) unbounded global progress despite only one finite accepted-service interval and unbounded per-attempt restoration, those requirements are incompatible. The repair must change control semantics, or the premises must change. The user forbids the latter; do not silently weaken the progress goal or report a parameter adjustment as a complete solution.

No final theorem is claimed here. No source files or protocol drafts were modified. Validation consists of source-grounded design analysis and counterexample reasoning, without executable simulation or proof checking.
