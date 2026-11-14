# Independent fresh check: historical AUTH service design

Reviewed: 19 October 2026.

Artifact: `historical_service_design.md`, 204 lines, SHA-256
`bbfdcc3f29b395196e0871b115f98e9859d86bb291881c3da49ca37c882a972d`.

This review uses the proof-verifier workflow. I read the frozen v3.1 Model and Part II source, the current v3.9 main source, and its required selection, health, source-retention, public-kernel, request, and retirement sections. I did not read the prior author/reviewer reports cited by the note, including the two reports named at its line 32. The reviewed artifact was not edited.

## Summary

**Final Verdict:** The candidate is viable but incomplete due to the acknowledged **Justification Gaps** in canonical task discovery and integration of the scheduler with the prepared-response/control machinery; I found no **Critical Error** in the local conditional arguments at their stated scope.

The historical service-domain condition is an explicit hypothesis of the candidate, not a conclusion supplied by AUTH authentication. Its unavailability is not, by itself, a counterexample to the conditional theorem or a universal impossibility result. Conversely, the note does not establish that this condition is inherited without qualification from the frozen Model.

**List of Findings:**

- **JG1 — Canonical discovery remains an unimplemented theorem dependency.**
  - **Location:** Lines 58–60: “Inventory cuts must be fixed by canonical source evidence and processed monotonically” and “this adapter is **not yet derived**”; lines 71–73; lines 197–198.
  - **Issue:** A complete answer for each fixed L1 cut is insufficient by itself. The candidate still needs its specified canonical transition, eventual passage beyond every relevant installed AUTH publication height, deterministic token registration, and nonblocking treatment of unfinished inventory work. Without those components, the request proof cannot establish that its particular historical task ever joins the scheduler. This is an explicitly acknowledged **Justification Gap**, not a false assertion that the present interpreter already has the interface.

- **JG2 — The prepared response and fairness arguments are conditional on scheduler integration that is not yet specified or verified.**
  - **Location:** Lines 97, 113, 125, 141, 166, and 199–200: “there is … a finite prepared response bound L_tau of the same form as v3.9”; “source-handler dependency accounting … must be rewritten”; the acknowledged controller arbitration and journal obligations.
  - **Issue:** Fixing the selected task's AUTH and anchor removes the moving preparation target, but does not alone prove that every mandatory operation before its first accepted DATA fits the inherited prepared processing path. The new queue, catalogue cuts, task-credit updates, historical appointment verification, and controller arbitration need a concrete integration that preserves that path. The original v3.9 bound cannot simply be imported as a theorem about unspecified new transitions. Subject to that integration and the stated service domain, the subsequent AUTO contradiction and finite-predecessor argument are sound. This is an acknowledged **Justification Gap** in the full candidate, not a demonstrated failure of the proposed local mechanism.

No additional Critical Error was established. Neither the safety discussion nor the carrier-only escape is presented as a completed whole-protocol proof, and this review does not certify either as one.

## Detailed Verification Log

### 1. Claims, hypotheses, and dependencies

> Lines 11–16: “This is a **conditional design**” with historical service coverage and a canonical inventory adapter left open.

This accurately scopes the result. The frozen Model supplies immutable per-r authority, fixed-snapshot certificate accounting, synchronous correct-member communication, bounded delivery of complete results, and eventual preparation. It does not turn an authenticated old roster or an ON flag into evidence that every required old voter is presently running the proposed handlers. The note makes the required temporal service scope explicit at lines 117–119 rather than using it silently.

The dependencies of the main route are:

1. Canonical discovery registers the relevant exact-AUTH task.
2. Controller and queue rules eventually appoint it, accounting for eligibility and any incumbent pinned task.
3. After finite interference, no other task advances its retained anchor before its discharge.
4. Persistent preparation and an adapted complete-source path supply a finite task-specific response bound.
5. The growing-beta argument forces an accepted exact-task DATA.
6. Repeated service plus same-snapshot awareness supplies the accepted report events needed by the existing request-potential proof.

JG1 concerns step 1. JG2 concerns the uncompleted integration needed for steps 2 and 4. The later steps were checked conditionally, rather than treated as disproved by those gaps.

### 2. Exact task identity and service debt

> Lines 38–44: `tau = (chain, r, Hash(AUTH[r,0]), i)`; discharge requires newly accepted DATA “under its exact task.”

Correctly distinguishes authority identity, physical region identity, and engine identity. Frozen authority is indexed by r and can be reused by a fresh beta without rebinding its members, schedule, or quota. Counting accepted DATA from a different roster of the same physical region would not establish service of the request holders in this task. Requiring an exact-task discharge prevents that inference.

New task arrivals at the tail cannot overtake an existing token. A finite quota also prevents one finite stock of old report contexts from supplying infinitely many accepted first-DATA events. Acceptance of empty application batches can discharge service debt, but does not independently establish request inclusion; the later report argument is necessary.

> Lines 46–48: temporarily ineligible tokens retain their age; REGULAR remains unrestricted.

This is a substantive change to Force selection, not a consequence of current physical-region owed debt. The note says so. Parking an OFF task avoids requiring service while it is administratively excluded. The nonpreemptive pinned-task rule means that queue fairness must cover every task that can block a token, including a younger task appointed while the older token was parked. The note's global lifetime service-coverage reading supplies that condition. The narrower statement “this target and its originally earlier tokens are serviceable” would not by itself suffice if an uncovered younger incumbent were pinned. I do not read the broader claim as making that weaker assertion.

### 3. Discovery of zero-output and speculative tasks

> Lines 52–60: register tasks from a complete finite inventory of initial AUTH records, not from a local cache.

The proposed source is coherent with the abstract statement that finalized L1 authority records are common and authenticated. Nevertheless, abstract authenticity is not an implementation of enumeration or canonical registration. JG1 remains material.

> Lines 64–69: accepted output and public FINAL roots are insufficient; a predecessor terminal result need not contain the child's AUTH.

Correct. A valid initial entry can create a request owner without the session ever forming a complete result. A rejected speculative lineage may likewise leave no accepted producer identity. REGULAR terminal output establishes a predecessor boundary opportunity; it does not assert that the successor AUTH already exists. Requiring an unconditional fetch of every r+1 suggested by a terminal object could wait for a nonexistent authority.

> Lines 71–73: a complete inventory eventually passing an installed AUTH's height finds it; fixed-cut work does not grow indefinitely.

This implication is valid with the stated cofinal discovery condition. A fixed finite scan and tail-enqueued later work prevent moving-target starvation of that scan. This supplies no uniform scan duration, and it cannot be inserted into every timed source operation merely by calling each input finite.

### 4. Appointment, exact anchors, and strict cutoff

> Lines 77–85: a Force creates `(beta_new, tau.r)`, retains exact pre-payload G, and sets the endpoint to `h(G_at_Force) + TB[tau.r]`.

This is a coherent fresh-engine identity rule. It neither resets the old pair's consumed quota nor relabels old accepted DATA. Ordinary r+1 progression within a beta can remain unchanged. The current historical handoff verifier would require a real revision to verify this different target rule and its token serial; the note explicitly reserves that work.

Before the appointed task's first accepted DATA, its speculative regular successors cannot bypass it through contiguous acceptance. Closed-version DATA remains rejected, including output in the crossing block, and new-version DATA remains inadmissible in that same block. Those rules support the claimed fixed-anchor reasoning; the queue does not grant old DATA new append permission.

The anchor is fixed in the tail after the task is persistently eligible and pinned. It need not remain fixed across an earlier OFF interval in which the rule appoints another task. Lines 87, 123, and 137 must be read together with their finite-interference prefix. The proof below uses that tail, not an invariance claim covering temporary eligibility fallback.

### 5. Mandatory MSIG scheduling and health credits

> Lines 91–97: REGULAR can avoid a Force-only scheduler; the committee must eventually use an existing MSIG opportunity.

Correct. Successful regular sessions need not create AUTO. A duty that exists only at Force boundaries therefore needs an actual Force-generation obligation. Using the existing MSIG format preserves its target-beta semantics: REGULAR keeps beta unchanged, whereas an earlier effective Force makes a later record stale. A stale record cannot be retargeted or refunded.

The note correctly identifies arbitration as unfinished: optional H1 traffic cannot be permitted to consume every opportunity before a mandatory scheduling obligation acts. Existing per-tip credits limit interventions at fixed G; they do not independently provide priority for this new duty. This is part of JG2.

> Lines 103–111: health leases consume `u_tau[s_tau]`; another AUTH's accepted DATA cannot renew the task credit.

For a registered unserved task and a faithful controller journal, this gives only finitely many further OFF leases affecting it. The existing lease policy supplies a finalized finite future ON, and growing Espresso timestamps cross it. The finite reserved backlog must be included, as the note does. Legacy OFFs without restoration still require the baseline adoption reconciliation; a finite legacy backlog alone does not prove ON restoration. Unlimited administrative OFF remains outside the managed-task claim.

No claimed bound on old voters' participation follows from this control result. The note correctly separates the two.

### 6. Fixed-task preparation and the AUTO contradiction

> Lines 123–125: the old roster supplies a finite recovery prefix, and persistent preparation retains that prefix and exact anchor.

Under the explicitly stated old-namespace service conditions, the finite-roster/exclusion argument is local to one immutable configuration. It does not require a finite union of all past or future memberships. Persisting the fixed native-state target and authority-fetch tasks across beta changes removes the recurrent reset of that preparation target.

The new source-processing dependency accounting is still JG2. Assuming it supplies the claimed finite `L_tau`, the subsequent inequalities are valid.

> Lines 127–135: `a_(n+1)-a_n <= L_tau`, `theta_(n+1)-theta_n >= (1+beta_0+n) W_tau`, and `theta_n <= a_n+epsilon_E`.

After finite matching MSIG/status interference and old cooldown expiry, zero-output AUTO immediately reassigns the same eligible pinned task. If an attempt remained open beyond its prepared response bound, acceptance would contradict the no-growth hypothesis; hence its closing/opening time is at most `L_tau` later. Startup stays in the failed-startup phase, the base window stays positive and unchanged, and beta increments once per effective Force. Summing gives a quadratic lower bound and a linear upper bound on the opening timestamps. The frozen upper clock-lead bound suffices; no lower-lag bound is required. A timestamp jump causes only one effective Force in the prelude and does not restart the new duty at an old deadline.

> Lines 137 and 141–143: bypassing an unserved task lets other cohorts move G; persistent pinning removes that escape.

Correct at the stated scope. Eventual preparation at each separately moving anchor does not give a time by which one of those anchors is prepared before it is superseded. Fixing one task and anchor permits a finite maximum for that task instead. This is a limitation of the fallback argument, not a universal impossibility theorem about all schedulers.

With every possible blocking task covered, finite predecessors and eventual discharge yield eventual service; tail arrivals cannot create new predecessors. Requeueing supplies repeated service. An eligible uncovered historical incumbent can block this candidate, so the result does not recover v3.9's live-subset theorem merely by retaining its name. Lines 143 and 202 explicitly acknowledge that scope difference.

### 7. Request awareness and accepted service

> Lines 147–153: same-snapshot relay gives awareness, repeated exact-task service supplies accepted aware first-DATA histories, and the finite predecessor potential decreases.

Conditional on task discovery and the historical service domain, this chain is valid. The persistent owner relays without requiring an old BFT worker to restart. A fresh pair cannot reuse an immutable EMPTY report from another context. There are only finitely many reports before the finite awareness event, and finite quotas prevent that stock from generating infinitely many distinct accepted term-first DATA blocks.

Each later genuine report witness includes a correct signer from the aware snapshot. Its reported minimum is at most `key(x)` while x remains eligible and absent. The executed primary therefore spends a previously unaccepted identity at most that key or includes x. Exact inherited ancestry prevents spending the same accepted identity twice. The existing finite-potential argument then forces inclusion. This step genuinely needs accepted aware service: a stream of rejected complete receipts does not decrease the potential on G.

### 8. Safety and lifecycle scope

> Lines 157–166: unchanged cutoff, target-beta MSIG semantics, fresh per-pair core state, six phases, and provenance; “not a full revised safety proof.”

The listed invariants are consistent with the intended mechanism, but they are obligations for a revised interpreter rather than a complete proof. In particular, the queue must become authenticated public/checkpoint state, and a token cannot override a handoff already fixed in the same beta. The source-path and terminal-commit cautions accurately prevent an eventual catalogue scan from being hidden inside the old regional Delta bound.

Current retirement keeps Closed independent of worker phase. It blocks fresh old-pair BFT actions, and missing snapshot context leaves a durable relay obligation rather than creating a fictitious complete-close time or a new 4-Delta deadline. Nothing in this scheduler proposal may treat Terminated as authorization to discard that obligation or withdraw old voting stake.

### 9. Carrier-only retirement escape

> Line 174: extending drain alone can leave only one FINAL with W=3 and F=1.

Correct. One correct initializer can vote before learning closure while another correct member processes closure before the proposal and does not vote. The surviving timer commits locally and sends FINAL, but weight one is not greater than F=1. An adversary can withhold its own signatures. More waiting cannot supply the missing signature, and the publication assumption does not apply to this incomplete result.

> Line 178: allowing an already entered old pair to continue ordinary work changes retirement while leaving global cutoff intact.

Correctly classified as a real protocol change. For the stated working informed cohort and proposal, continued ordinary execution can remove that particular first-carrier obstruction. The resulting old DATA can be transported without becoming appendable after closure. A whole regional safety/progress proof for this modified retirement rule is not supplied or silently imported.

The four stated limitations are sound:

- An immutable terminal proposal at quota one can exhaust all DATA capacity before x arrives. DRAIN adds no application DATA capacity.
- A selected but unentered Waiting pair gets no entry permission from allowing only already entered work to finish.
- Repeated rejected y-only results need not reduce the accepted-history potential for a larger-key x. Public import of y may later help another task include it, but carrier formation alone does not establish that event.
- Eventual finite-job import does not imply import before a particular new roster's immutable first report. The current import proposition explicitly separates first-carrier formation and accepted-report awareness.

The note appropriately calls the repeated-y case a missing implication rather than proving that y can never be accepted. Likewise, it does not infer that an old correct roster has disappeared merely because its worker has terminated.

## Validation and limits

This was a source-level, adversarial mathematical/design review. I checked the local dependency, quantifier, signature-threshold, cutoff, and timing arguments against the permitted sources. I did not repair the design, modify the protocol, run a TeX build, execute a protocol simulation, or perform model checking. The report does not resolve the frozen Model's historical participation interpretation or complete the new inventory/controller/source-handler specifications.

The reviewed note's SHA-256 was checked before review and again after writing this report; both checks returned the frozen value recorded above.
