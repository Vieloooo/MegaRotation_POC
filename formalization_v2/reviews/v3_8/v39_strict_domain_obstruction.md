# Strict snapshot communication: obstruction and construction boundary

Date: 19 October 2026. Scope: independent review of the frozen v3.1 model and the current v3.8 main, requests, liveness, and retirement rules. No manuscript source was changed. This note uses the strict interpretation that the guaranteed correct-to-correct delay applies only when both endpoints belong to the same authenticated session snapshot. It does not infer a physical-region transport service.

## Summary and verdict

**Final Verdict:** v3.8 does not derive full request inclusion from the unqualified frozen ingress prose. Its accepted-report-awareness proposition has a valid finite-predecessor argument under its explicit awareness premise. A fresh-roster starvation execution demonstrates the missing implication under aggregate actual-turn fairness. It is **not** a universal impossibility theorem under every possible interpretation of the frozen repeated-turn clause.

The following distinctions are essential:

1. A request that has entered an original complete regional DATA result already has a promised cross-roster byte path: the original Publish Layer followed by Espresso. Global acceptance of that particular result is not necessary for public availability of its bytes.
2. A request known only to members whose authorized DATA opportunities have ended has no such promised path merely because they retain it.
3. Rejecting a disjoint future AUTH can enforce an algorithmic admission predicate. It does not prove that an immutable, arbitrary AUTH input sequence ever supplies an acceptable replacement.
4. Permanent disjointness of all future rosters is incompatible with an interpretation requiring each particular correct leader identity to enter infinitely many future turns. A counterexample may not assume both and then claim to satisfy all premises.

**List of Findings:**

- **Justification Gap — full request inclusion:** requests fragment lines 291–301 and liveness fragment lines 466–495 correctly leave cross-roster awareness unresolved. The frozen premises have not been connected to the awareness premise at requests lines 245–250.
- **Scope error if asserted — universal impossibility:** a diagonal trace in which every identity has a finite service lifetime does not satisfy a strong per-identity infinite-turn reading of frozen main lines 490–493. Such a trace proves neither impossibility for that reading nor impossibility of every algorithmic AUTH policy.
- **Scope error if asserted — publication repair:** delivery of an original complete regional result does not grant delivery of a pending-request supplement attached after that result is complete or already published.

The current sources therefore support an explicit unresolved boundary, not an unqualified impossibility claim. No independent complete repair under all readings of the frozen premises is established by this note.

## Source contract and dependency map

Paths below are relative to `formalization_v2/`.

| Contract | Source | Consequence used here |
|---|---|---|
| Immutable authority; fixed members, weights, quota; recovery changes only schedule | `revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex:223–237` | Future records cannot be silently rebound; a later snapshot can differ from the earlier one. |
| Complete regional result is DATA plus direct/covering FINAL and necessary evidence | Same file, lines 299–326 | A bare pending request is not a publication-service input. Direct cross-region delivery is optional. |
| Fixed session membership and retained state | Same file, lines 328–346 | Retention preserves knowledge at an old correct node; it does not identify a live future recipient. |
| Regional synchronous delivery | Same file, lines 360–372 | Under the strict interpretation used here, old-to-old and new-to-new deliveries are bounded; disjoint old-to-new deliveries are not thereby promised. |
| Original complete-result publication bound | Same file, lines 374–389 | The clock starts with a complete original result; a previously recorded result already satisfies this guarantee. |
| Espresso complete-record visibility and replay | Same file, lines 411–458 | Complete result bytes cross regional boundaries once actually submitted; archival catch-up has no uniform bound. |
| Actual correct turns, sufficient service, request reach, finite older queue | Same file, lines 486–505 | These premises must be checked separately; none can be replaced by schedule names alone. |
| FIFO and removal only on inclusion in G | Same file, lines 1839–1858 | A retained request has a finite local rank; the text does not impose a common queue or a global membership-transfer protocol. |
| Within-snapshot retained request relay | `sections/rotation_v3_8_requests.tex:23–44` | Receipt at one correct member informs that snapshot within its existing delay; no disjoint-roster bound is asserted. |
| Immutable exact-context INPUT_REPORT | Same fragment, lines 77–104 | A later arrival does not amend an already signed EMPTY report. |
| Original public result and report erasure | Same fragment, lines 157–199 | New report sidecars are not part of the promised public result; selected request bytes must occupy normal DATA capacity. |
| Conditional request theorem | Same fragment, lines 243–301 | Inclusion follows from infinitely many genuinely aware accepted report witnesses. |
| Retirement gates and persistent owners | `sections/rotation_v3_8_retirement.tex:84–110,130–174` | Retention and relay survive exit; fresh old-session DATA, votes, and reports do not. |

The main dependency chain is:

`client receipt -> within-snapshot awareness -> a genuine request-bearing DATA result -> original Publish/Espresso -> future-roster awareness -> accepted report witnesses -> inclusion in G`.

v3.8 proves a bounded local implication for the second arrow and a conditional finite-potential implication for the last arrow. The absent general implication is between retained knowledge in an expiring snapshot and a useful future DATA/report opportunity. Public result delivery cannot be invoked before its qualifying input exists.

## Detailed verification log

### 1. The conditional finite-potential proof

The proposition assumes “infinitely many accepted term-first DATA” whose genuine witness includes a correct signer retaining eligible `x` at its report event (requests lines 245–250).

At each such witness, its correct signer reports a request with key at most `key(x)`. The certificate minimum executed by that DATA has key no larger. If `x` has not entered G, exact-parent ancestry exclusion ensures that this executed identity was not already in the accepted prefix preceding that DATA. The finite set of canonical byte strings with key at most `key(x)` supplies a strictly decreasing finite potential. Acceptance is prefix extending, so discarded speculative branches cannot reset the potential. This reasoning is valid under the stated admissibility, fit, canonical identity, genuine-witness, and awareness conditions.

The proposition does not prove its awareness condition. It explicitly allows arbitrary roster changes **conditional on** that condition; this is not a derivation of the condition for arbitrary roster changes.

### 2. Why the publication bound is not an ingress bound

The frozen service begins when a correct participant has an original complete regional result available (frozen lines 374–389). If `x` is only a retained pending request after the snapshot has consumed its quota, this antecedent is false for any result carrying `x`.

All complete empty DATA results may nevertheless form, be submitted within `T_pub`, become visible within `T_E`, and enter G. This satisfies the original service in full. It does not permit adding `x` to an arbitrary sidecar, changing the certified DATA body, reopening a completed quota, or claiming that the service must republish an already recorded result with newly attached content.

### 3. Why repeating regional service need not repeat awareness

Repeated accepted DATA from a region is a statement about a region identifier and its successive authorized sessions. It does not by itself imply that the same node, a knowledgeable node, or a node reachable from a knowledgeable old snapshot participates in those sessions. A finite old local queue is useful only if that queue obtains later relevant execution opportunities or a guaranteed transfer to a queue that does.

Conversely, if “correct leaders receive repeated turns” means that a particular knowledgeable identity enters infinitely many future DATA-producing turns, a permanently disjoint-roster execution does not meet that premise. This is a necessary scope check, not a small technicality.

## A precise v3.8 starvation execution under aggregate actual-turn fairness

This execution is a counterexample to the inference from pointwise eventual arrival plus repeated **aggregate** correct service to accepted-report awareness. It is not presented as satisfying the stronger per-identity interpretation discussed below.

Use two physical region identifiers A and B, both continuously ON. Normal rotation alternates between them. Set every session quota to a finite positive integer, for example four DATA blocks, with `K=1`; all startup/running windows are sufficiently generous for the actual processing and publication schedule. Supply all required immutable authorities and native states in time. Every new slot has fresh snapshot identities, disjoint from all earlier slots. The relevant snapshot has enough correct weight to form every certificate. All actual correct-to-correct messages inside each snapshot obey its fixed Δ.

For an explicit schedule with real recovery entry, give a snapshot three unit-weight members with Byzantine bound one. Its generation-zero initializer is a silent Byzantine member. Correct BLAME causes genuine recovery; generation one has a correct initializer and thereafter alternating correct leaders. No member equivocates. All later generations also have correct initializers and fair schedules. This satisfies the inherited schedule formulas and permanent exclusion vacuously, and actually enters correct recovery initializers and several correct DATA-producing turns in each session. No speculative schedule name is counted as an entered turn.

Fix a valid authenticated request `x` that fits alone and remains admissible after every empty DATA prefix. There are no other client requests. For session n:

1. The fresh cohort has not received `x`. It follows the normal engine, issues genuine EMPTY first-DATA reports, and constructs/votes every DATA block in its quota with an empty application batch.
2. After the last request-selection/proposal event for the terminal DATA, but before the terminal DATA timer/Completion finishes, deliver the full `x` to a correct member. Thus the arrival can occur while the cohort is still Active, not only at an abandoned archive.
3. Deliver its required same-snapshot relay within Δ. All correct members retain `x`; it has no older request ahead of it. The already proposed terminal DATA is unchanged. Any recovery-only DRAIN carries no requests, and the session has no remaining DATA capacity.
4. Allow all original commitments and complete FINAL results to form. Publish every complete original result within the original service bound, finalize it in Espresso within the original visibility bound, and accept it into G. The terminal result opens the next regular session.
5. Retain `x` at the old correct members and their persistent owners indefinitely. They complete all required old-snapshot relays. Delay any attempted transfer to the disjoint next cohort until after that cohort's last request-selection event. Independently deliver the client's `x` to the next cohort by step 2 of its own session.

Each finite session finishes. Public time and G grow without bound. Both regions contribute accepted DATA infinitely often. Each installed cohort actually receives `x` at a finite time while its session is still active, and every local queue satisfies finite older backlog. All original complete-result publication and visibility promises hold; the adversary does not delay a qualifying public result or suppress its payload. No Force is needed, so there is no shortage of uninterrupted service, no excessive committee interference, and no conflict with v3.8 debt/health rules. The client request remains absent from every DATA body and hence from G forever.

The request's late arrival is not inconsistent with synchronous regional communication: before step 2 no correct member of that snapshot has its bytes, and the model supplies no bound for the client's first ingress. All subsequent same-snapshot relays obey Δ.

This execution makes the acceptance prefix and all report/proposal events indistinguishable from an execution with no client request, until each cohort has exhausted the choices that could include `x`. The original publication service carries only the actual empty results, so it does not distinguish the two executions for the next fresh cohort.

**Exact fairness limitation.** Correct initializers and DATA leaders are actually entered infinitely often in aggregate, in every physical region, with arbitrarily many entered correct turns obtainable by increasing the finite quotas. But each identity has a finite lifetime. If the frozen clause requires every particular correct identity to enter infinitely many later sessions, this execution is not admissible. Increasing its finite quota does not fix that defect. Nor can one repair the trace merely by giving old identities infinitely many schedule positions whose turns never enter.

**Exact algorithmic limitation.** This execution directly applies to v3.8 and to protocols that let a finite-lifetime roster exhaust its request-bearing DATA opportunities without guaranteeing a later transfer. It does not defeat an unspecified protocol that enforces a different AUTH/activation rule. Such a rule must be examined for its own progress and service-input obligations before claiming either success or impossibility.

## Construction boundary using only original complete results

There is a useful algorithmic route after a first carrier exists. The following is a conditional construction boundary, not a completed v3.9 protocol or proof.

1. Inspect authenticated DATA payloads in every original complete regional result encountered in the Espresso namespace, including results whose DATA cannot advance G because their branch closed or their public wrapper was rejected. Recover actual authenticated client requests from those bodies. Do not treat publication as proof of current application admissibility; recheck it for the future parent.
2. Retain each recovered request by logical identity until its inclusion in G. The entire original full-result service remains unchanged: no bare request, report, or new suffix is submitted under its bound.
3. Before a new roster begins the report sequence used for a liveness claim, finish an explicitly fixed finite historical cut sufficient to include every result finalized through that cut. Hydration may be eventual. A rolling target that continually moves forward can starve; a hash-only checkpoint that loses unaccepted request bytes is insufficient.
4. Use the existing verifiable primary-selection rule after that actual awareness. Genuine accepted first-DATA witnesses then spend a finite predecessor potential.

The source cut, validation, catch-up, and entry timing would need exact specification and review. In particular, requiring a new Ready gate without proving that it eventually completes is not a repair. A startup bound cannot be inferred for arbitrary archived history. Public ingestion also cannot silently enlarge the old service's record size or load envelope.

Most importantly, this route assumes that some genuine original complete result carrying `x` is eventually formed. A knowledgeable correct leader's schedule position is insufficient. An entered turn interrupted before useful proposal/receipt formation is insufficient. An uncommitted request-bearing proposal is insufficient. A complete result on a later discarded branch is sufficient for **transport**, after which acceptance of `x` still follows from newly proved awareness and accepted-service arguments.

To turn this route into a full theorem, derive the first-carrier event from the frozen actual-turn and uninterrupted-service premises, or implement a protocol rule that guarantees it without changing those premises. This note does not derive that implication for arbitrary immutable future AUTH histories.

## AUTH policies: what is an algorithm and what is a new assumption

An explicit deterministic validity/admission rule is an algorithmic change. Examples include refusing activation until a specified prior result is public, requiring an authenticated continuity witness, or requiring a particular overlap property of consecutive authorized rosters. These are not intrinsically trust assumptions if their predicates are completely checked by the protocol.

The liveness proof must nevertheless address the following separate facts:

- AUTH at an already assigned index is immutable. If the supplied record fails a new predicate, the protocol cannot replace it by relabeling another record or choosing a different membership for that same index.
- The frozen “required authorities become available” wording does not explicitly say that an external authority source will follow a newly invented overlap, request-transfer, or admission policy. Treating that as automatically guaranteed changes the quantified authority histories.
- Rejecting all disjoint future records narrows the histories on which the protocol operates. To claim unchanged liveness, specify who can produce a conforming record, how its availability follows from an existing service, and what happens when the immutable record at the next index is already incompatible.
- Retaining the old roster is not enough if its finite fixed DATA quota is exhausted. Allowing it to produce new request-carrying DATA beyond that quota or to reopen a completed/closed session changes another protocol contract and requires a fresh safety argument.
- Requiring a transfer acknowledgment does not create a reliable transfer channel. Under the strict interpretation, an old-only correct node and new-only correct node may lack any promised direct delivery. A gate waiting on such a message can stall forever.
- Requiring a digest or an availability signature without the full pending request bytes does not create an original complete DATA result and does not establish awareness.

An implementable authority-generation protocol might still be possible. It must be written and proved under the existing L1 authority/control interface. It cannot be justified merely by saying that an honest AUTH issuer chooses appropriate rosters; the frozen model contains no such issuer-honesty or membership-transfer promise.

## What can be concluded about the clarification

The strict snapshot interpretation does not justify importing old-to-new delivery merely because both identities share a physical region label. That communication-domain clarification is material to any proposed direct relay repair.

It is not, by itself, a complete yes/no decision procedure for full request liveness. The statement also needs a precise reading of:

- which actual nodes receive the client's request, and when that obligation applies to a changing region;
- whether repeated entered correct turns are aggregate, per still-serving identity, or per every identity ever correct;
- whether “enough uninterrupted service” guarantees a useful request-bearing result after awareness, rather than only empty DATA progress somewhere in the region;
- whether future AUTH histories are arbitrary immutable external inputs, or outputs of an explicitly specified and justified protocol.

For the present manuscript, it is sound to keep the full request-liveness goal open pending either a source-grounded clarification or a complete algorithmic bridge proof. It is unsound to convert the clean fresh-roster trace above into an unconditional impossibility result. It is equally unsound to claim that a within-region relay or a proposed AUTH restriction already supplies the missing proof.

Validation: manual source-level proof and scope review only; no executable model check, protocol test, or TeX compilation was run. Existing manuscript and other review files were left unchanged.
