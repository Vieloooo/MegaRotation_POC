# Fresh independent verification of the v3.9 ownership candidate v2

## Summary

**Final Verdict:** The conditional construction is viable but incomplete due to a procedure-level **Justification Gap** in the authority/context preparation barrier; its arbitrary-roster inclusion argument also requires global temporal local finiteness, which it explicitly invokes but does not establish from the frozen Model. No **Critical Error** was identified in the fixed fragment budgets, completed dissemination, HYDRATED collection, or finite-predecessor argument under their stated prerequisites.

This review grants the candidate's broad mixed-snapshot same-region communication premise for the algorithm audit. Its absence from the frozen Model is a separate scope boundary, not a counterexample to that conditional audit. The review does not certify full request liveness under the frozen environment, the integration of the new Ready gate into the whole protocol, a concrete transport implementation, or achievement of the overall goal.

**List of Findings:**

- **F1 — Justification Gap: the timed query has no specified implementable context-readiness barrier.** **Location:** candidate line 63, “Begin the CUT_QUERY deadline only after the complete context has reached the relevant correct handlers under the conditional service,” and line 89, “Context transfers finish, and their actual local validation completes, before the timed cut query that depends on them.” **Issue:** when an owner's large-context validation is actual terminating work outside the bounded per-envelope handler, there is no specified action by which the hydrator knows when the required correct owners are ready. Eventual termination supplies an existential time, not the local start guard of an algorithm. Starting the `2d` query from a locally known send-completion time can miss a correct owner's descriptor at the irreversible freeze; waiting for every directory identity can block on a silent Byzantine or retired identity. If all owner-side context validation is instead inside the old bounded handler, that is a materially different interface interpretation that must be identified rather than inferred from eventual whole-object validation. Coverage at line 87 and the subsequent hydration and inclusion conclusions remain unverified at this dependency.
- **F2 — Justification Gap in the connection to the frozen assumptions, not in the locally finite conditional argument.** **Location:** candidate lines 19 and 138, “only finitely many different roster preparations can have started before one fixed request's receipt time in a locally finite execution,” and lines 173–181, which list the remaining boundary without listing this execution restriction. **Issue:** the required finiteness is global across all target preparations, potentially at arbitrarily many identities, in a bounded real-time prefix. The frozen Model's upper delivery bounds and per-node finite older queues do not establish it. The candidate does acknowledge the restriction locally and explicitly excludes Zeno executions at line 142; it is therefore inaccurate to call the premise wholly unacknowledged. Nevertheless, granting only the broad communication domain does not prove the additional restriction from frozen v3.1. Sections 7–9 remain conditional on it, and the distinction must survive any claimed promotion to the unchanged-environment theorem.

The revised finite fragment budget and explicit HYDRATED dispatch/collection are substantive improvements and pass the local checks below. This verdict does not rely on any earlier candidate, audit, or author notes.

## Detailed Verification Log

### 1. Scope, statements, and dependency map

> “The broad mixed-snapshot communication service remains an explicit conditional premise, not a guarantee established from frozen v3.1.” — candidate line 3

The subject is an abstract conditional algorithm. The broad premise is granted in this review. Frozen `model_and_assumptions.tex:167–179` says that correct-to-correct messages within region `i` have bounded delivery and processing, but does not explicitly enumerate disjoint historical/future-snapshot endpoints. Current `rotation_v3_8_liveness.tex:480–495` preserves that unresolved boundary. Retirement's phase-independent handlers concern known fixed snapshots (`rotation_v3_8_retirement.tex:15–20, 43–83, 112–120`); they do not independently add a new communication edge. The candidate correctly separates these facts.

The proof dependencies are:

1. persistent original-owner bytes plus an authenticated finite historical directory;
2. complete authority/context preparation at all required handlers;
3. fixed descriptor, record, and fragment work yielding correct-owner cut coverage;
4. actual all-target byte dissemination strictly before a correct HYDRATED signature;
5. correct-share dispatch and collection, or adoption of an already valid certificate;
6. a pre-Ready hydration gate compatible with the existing complete-handler interface;
7. globally finite pre-request preparations plus finite accepted contribution per old slot;
8. exact-parent REPORT validity and a finite-predecessor potential;
9. conditional discharge of a finite pending set before physical departure.

F1 is at dependency 2. The following checks of dependencies 3–9 temporarily assume the required contexts really are ready at query start, as the proof-verifier workflow permits after a Justification Gap. F2 concerns the premise of dependency 7. These conditional checks are not an alternative proof repairing either gap.

### 2. Communication, processing, and resources

> “A finite maximum cannot create a communication edge absent from the premise.” — candidate line 25

Correct. For fixed `r`, a finite maximum of applicable regional delay allowances is finite. That algebra is useful only after the mixed-snapshot service is granted for the actual endpoint pairs. No maximum over future configurations and no cross-region bound are justified or used.

> “Its complete authenticated envelope ... must be admissible under the unchanged complete-message size/processing profile.” — candidate line 29

This is an explicit implementation condition. A positive payload `b` with all metadata/signature overhead inside the old profile has not been constructed by the candidate. The note does not silently count an entire unbounded pool as one message or increase a publication size limit. Frozen publication bounds apply to complete regional results under their stated limits (`model_and_assumptions.tex:181–196, 236–254`), not to bare requests or HYDRATED objects.

> “Any separate whole-object validation or local preparation is performed as actual terminating work ... It is not assigned an unproved Delta deadline.” — candidate line 31

This is the correct limitation for arbitrary local preparation. It also matters to F1: such work at a remote historical owner cannot be presumed to have finished just because the last individual context envelope was delivered. Incremental per-envelope authentication/storage and completion of a whole authenticated authority context are different predicates unless the interface explicitly makes them the same completed handler.

Finite work per preparation is not a global aggregate-load proof. The candidate appropriately leaves concurrent dispatch, storage, representation, and old profile compliance open at lines 33 and 178. This review assumes admitted traffic where the granted message guarantees actually apply; it does not certify an overload-resistant implementation.

### 3. Directory, ownership, and late requests

> “`D_i(r) = union of the regional identities in AUTH[k,0], 0 <= k <= r`.” — candidate line 39

For finite `r` and finite individual rosters this is a finite set, even when the union of every future roster is infinite. Immutable AUTH records determine its context; recovery authorities change only the schedule. Frozen Model lines 30–44 support those authority properties. Deduplication by persistent node identity does not discard the historical authority needed to authenticate that node.

> “The original owner retains the pending bytes even after one or all target rosters receive them.” — candidate line 55

This avoids a chain of private custody transfers. A later query can use the same persistently correct owner while the request remains absent from `G`. Current request semantics likewise remove requests only on global inclusion and preserve pending-service owners across worker phases (`rotation_v3_8_requests.tex:13–21, 33–44`). A node not yet in the applicable AUTH prefix is deliberately outside earlier-roster discovery coverage; the candidate states this at line 57.

The journal and cut state separate live arrivals from a fixed query job. Registration before the atomic cut also covers the arrival race: a request already in the cut is queried; a new request after registration is pushed. Beta does not reset the key `(i,r,v)`. Correct physical retirement of an empty owner is compatible with the procedure's use of timeouts rather than waiting for every directory identity.

The asserted finite journal prefix is itself an execution-finiteness condition. Local queue finiteness of older pending requests does not, by itself, prove finiteness of an append-only journal that also retains globally included history.

### 4. Preliminary context and CUT_QUERY

> “Only an authenticated query by the named target member v can create its cut.” — candidate line 61

This prevents a Byzantine sender from pre-creating another member's stale cut merely by putting that member's identity into a query. Legitimate earlier queries by the correct member remain possible and are handled by the later finite-exception argument.

> “Any large context is transferred ... followed by actual terminating local validation.” — candidate line 63

**F1 applies here.** The context can be a multi-envelope object, and whole-object validation is not automatically assigned the per-envelope deadline. The protocol then instructs `v` to start only after the relevant correct handlers have complete context, without specifying a locally observable completion condition. The distinction matters because `v` cannot identify the correct subset of its directory.

For example, under the allowed unbounded terminating-work interpretation, a correct owner can finish its context validation after a finite time later than a query deadline chosen from local sends. Its delayed descriptor is then discarded permanently at line 72, although it holds a still-pending request. This illustrates the missing implication; it is not a counterexample to a different algorithm that already has the required readiness barrier. Conversely, the note's permitted silent/retired directory members prevent replacing the missing rule by an unstated wait for every member.

> “At `s_v+2d`, after processing all messages due at that time, v freezes at most one complete valid descriptor per directory identity.” — candidate line 72

Conditioned on complete context already being ready, this bound is correct: a query sent at `s_v` is processed by a correct owner by `s_v+d`, the immediate signed reply by `s_v+2d`, and processing-before-expiry handles equality. A Byzantine source cannot increase the frozen record count or replace the accepted root later. Different Byzantine choices at different hydrators are harmless because correctness requires coverage of correct owners, not consensus on a pool root.

### 5. Immutable record metadata, fragment budgets, and reconstruction

> “The requester freezes the first valid same-context metadata received by the slot deadline.” — candidate line 76

Signed metadata binds the cut, owner, record index, exact finite byte length, and digest. The fixed target context prevents replay across preparations. The acceptance predicate must be read as validating the expected owner's signed same-context response; an unauthenticated packet is not valid metadata. Missing or unusable metadata consumes only its fixed slot. A tombstone removes the record from this acquisition without purporting to prove global acceptance to anybody else.

> “From that frozen L, compute once `n=ceil(L/b)`.” — candidate line 77

For finite `L>=0` and `b>0`, `n` is finite and immutable. `L=0` performs no fragment slots and reaches final checking. A very large Byzantine choice can cause very large finite work but cannot create a dynamically extending fragment budget. Every frozen descriptor has finitely many record indices and every record has finitely many fragment slots; the finite sum terminates under continued execution.

> “Unauthenticated, wrong-sender, wrong-context, and out-of-slot packets are ignored and cannot terminate an honest owner's slot.” — candidate line 78

The explicit filter protects the honest-source path. A matching malformed/conflicting reply authenticated as the expected owner may fail that record; a correct owner does not emit such a reply. Absence expires the slot. Late replies cannot restart an already completed attempt. Whether failure skips the remaining scheduled slots or consumes them without accepting bytes, the work remains bounded by the fixed budget.

The required chunk lengths total exactly `L`: the first `n-1` payloads have length `b`, and the last has length `L-b(n-1)`. Final checking additionally verifies `H`, canonical encoding, and authentication, rather than treating a hash or fragment count as the request. Binding hashes and unforgeable signatures are inherited from frozen Model lines 152–153.

> “A correct owner supplies the actual fixed metadata and every requested fragment of a still-pending request.” — candidate line 83

With F1's prerequisite temporarily granted, each correct query/reply fits its `2d` slot. A request continuously absent from `G` cannot be legitimately tombstoned, so its exact retained bytes reconstruct. If inclusion occurs during acquisition, omitting that already included request is harmless to the still-unaccepted coverage statement. Byzantine tombstones provide no stronger evidence than Byzantine omission.

> “On finishing or timing out a slot, a correct participant schedules the next fixed slot or its finite local check.” — candidate line 85

This specifies continuation rather than reliance on unsolicited retries. New arrivals do not enlarge the job, and beta does not restart it. The resulting pool-coverage statement at line 87 is valid conditional on the context-readiness prerequisite; because F1 is unresolved, that statement is not unconditionally verified for the supplied procedure.

### 6. Actual dissemination and the strict signing guard

> “It actually sends the complete signed metadata and the n complete signed fragment envelopes to every target.” — candidate line 95

The sender already has the frozen finite objects, so this multicast does not wait for recipient queries or acknowledgements. It includes every target identity, not merely members of a storage certificate. Receiver retention of out-of-order fragments and fixed object lengths avoids imposing a fragment-arrival order on the network.

> “Only at or after `b_v+2d` may v create its signed share.” — candidate line 99

For every envelope its actual send time is at most `b_v`; correct recipients finish its covered receive processing by `b_v+d`. Since `d>0`, this is strictly before `b_v+2d`. Consequently a correct signature cannot race an equal-time final envelope handler at another member. A queued send or unfinished sender validation does not establish `b_v`. For an empty pool the same guard is well-defined from acquisition/preparation completion.

This establishes retained complete bytes and validation material under the specified handler scope. It does not itself establish exact-parent application eligibility, a rebuilt request index, or bounded arbitrary whole-object receiver work. Candidate lines 31, 97, and 124 preserve these distinct integration obligations.

### 7. HYDRATED dispatch, formation, relay, and early stop

> “At share creation, v durably stores its signed share, inserts it into its own collector, and immediately executes an authenticated multicast ... to every target member.” — candidate line 107

This supplies the actual missing message-producing action. Local signing alone is not treated as certificate formation. Unfinished sends remain duties across beta changes and worker exit.

> “It caches at most one valid matching share per distinct signer.” — candidate line 108

Same-context checks bind the chain, region, target authority and directory prefix. Duplicate shares cannot increase weight. The full certificate is materialized from actual shares and verified at recipients. Once-per-context certificate relay is finite per holder; an alternate signer-set encoding does not induce unbounded relay loops. Fragmented certificate transport uses a fixed object size rather than a root or retrieval promise.

> “Their weight is `W-B >= W-F > F`.” — candidate line 112

Correct: frozen Model lines 137–143 give `B<=F<W/2`. Once all correct target jobs and actual share sends finish, a finite maximum of their completion/delivery times exists for this finite target roster. Every correct collector has sufficient correct-only weight; no Byzantine cooperation or separate collection service is needed.

> “No honest quorum intersection is used.” — candidate line 116

Correct. A certificate with weight greater than `F` proves at least one correct signer, not an honest intersection with a later REPORT certificate. The correct signer's completed all-target multicast is the relevant evidence. The example `{H0,B}` versus `{H1,B}` is consistent with this logic.

> “A correct target may adopt a valid complete certificate and stop redundant local acquisition while preserving ... duties.” — candidate line 118

This does not create a circular formation argument. If no certificate exists, no correct participant can invoke this stopping condition. If one exists, it already has a correct signer whose actual dissemination preceded its signature. Cancellation need not recover that signer's pool again and cannot erase retained requests, promised serving work, or already-created share/certificate relays. The proof does not require every correct target to sign after a certificate already exists.

### 8. Ready, retry persistence, and stalled-G preparation

> “Ready requires this certificate in addition to the existing exact-anchor, authority, and native-state prerequisites.” — candidate line 122

This is a proposed protocol change, not a property already present in v3.8. Current main `2000–2018` lists the existing Ready prerequisites and `2035–2043` performs atomic Ready/Enter. The candidate leaves their anchor, authority, closure, and durable-entry checks intact while adding a new preparation gate.

Conditional on F1 and the declared augmented ReadyBundle complete-handler contract, the first correct Ready occurs after every correct target already holds the covered request bytes. Its complete certificate can be adopted by a slower member without waiting for that member's private pool acquisition. The original entry-skew implication still has the same form: the first correct complete bundle is sent at entry and all correct eligible recipients finish their bounded handler by `e+Delta` (`rotation_v3_5_ready_bundle.tex:60–97, 107–144`). An unresolved pool fetch or unbounded post-Ready import would fall outside this implication.

For stalled `G`, the current liveness proof fixes one authority slot and finite roster (`rotation_v3_8_liveness.tex:44–59, 75–105`). Once its hydration procedure actually completes, adding that one finite preparation time to a finite maximum is legitimate. This does not put variable historical hydration inside `R_i`, but also does not prove a concrete unchanged `R_i` if extra validation exceeds the old handler profile. The candidate expressly reserves that integration audit.

Live pushes do not invalidate or enlarge old cuts. At a correct owner queried by the correct certificate signer, roster registration occurred before that cut. Later requests can therefore use the separate finite push path even when the roster's production worker has exited. No single-`d` total-object transfer claim is made.

### 9. Finite exceptions and arbitrary future rosters

> “There are only finitely many target preparations whose first correct query began before `t_x` in a locally finite execution.” — candidate line 138

**F2 applies to model scope.** The proof needs finitely many such preparations globally, not merely finitely many at each one of infinitely many possible identities. Upper network-delay bounds do not impose a positive lower separation between preparation events. Frozen Model lines 310–312 give a finite older queue for each request, not a finite real-time prefix of all roster preparations. Current requests lines 283–289 explicitly reason about a *finite execution prefix*; they do not establish that every bounded real-time interval is such a prefix.

Under the candidate's global temporal local-finiteness restriction, the counting step is correct. Query events exactly at `t_x` also form a finite set under that restriction; the strict-before/strict-after wording leaves that endpoint implicit, but it cannot create infinitely many exceptions under the stated restriction.

Fix a persistently correct original owner `p` that appears at finite authority index `k`. Only finitely many target indices precede `k`. For later queries started after `x` is retained, `p` is in the authenticated finite directory and remains a live pending owner while `x` is absent from `G`. Conditional cut coverage and completed dissemination then give all-correct awareness before a certificate can authorize Ready.

An old certificate adopted after `t_x` may still reflect an earlier cut. This is not a counterexample: its target is among the finite pre-request preparations. Immutable beta-independent hydration keys prevent retries from creating an unlimited number of distinct exceptional target slots. Installed-slot quota accounting and the zero-output/positive-output Force successor rules bound each such slot's accepted contribution: zero-output retries add no accepted DATA; once positive output closes, the successor index advances. Arbitrarily many zero-output beta retries do not create infinitely many accepted old-cut opportunities.

No overlap between successive rosters, uniform future delay bound, or finite union of all future memberships is required by this conditional counting argument. It is not a Zeno-execution theorem and gives no numerical request latency.

### 10. Exact-parent REPORT selection and finite predecessors

> “A report order such as `(canonical_encoding_length, canonical_encoding_bytes)` has finitely many predecessors for every fixed canonical request.” — candidate line 153

Correct over a finite byte alphabet. The current v3.8 implementation uses `(length, hash)` rather than literal bytes (`rotation_v3_8_requests.tex:48–56`); the candidate presents its order as an example, and both have a finite lower set for a fixed canonical request under binding hashes. The proof uses this property rather than globally comparable receipt times or a union of future FIFO queues.

> “Some correct signer in every report certificate then supplies an eligible candidate `y <= x`.” — candidate line 155

Given all-correct awareness, completed eligibility processing, continued admissibility, individual DATA fit, and an accepted exact parent not already containing `x`, the inequality follows from the correct reporter's minimum rule. Current requests `58–75, 87–101, 128–155` provide the exact-parent, immutable report, and common validation predicates. A Byzantine proposer's smaller reported minimum must still provide matching authenticated request bytes and pass application/ancestry checks.

The selected primary is fresh in the exact accepted ancestry. Each accepted primary therefore spends a distinct logical request from the finite lower set of `x`; speculative discarded histories do not spend it twice. The formal potential argument in current requests `243–272` supports this inference. Repeated accepted regional contributions supply infinitely many accepted term-first blocks by the finite term/session bounds (`274–279`), not by counting DRAIN as a fresh request opportunity.

This deduction is valid with its awareness and interface premises. It remains downstream of F1 and of the global local-finiteness restriction in Section 7. The candidate correctly leaves report timing, request-preimage handling, intrinsic FINAL validity, and concrete size/load integration open; this review does not certify them afresh as a whole protocol.

### 11. Physical exit and the limit of termination claims

> “A whole node cannot release its only pending-request service while x remains globally unaccepted merely because a future roster's certificate exists.” — candidate line 161

This preserves the fixed discoverable owner used by the proof. Current retirement only releases the production/recovery worker after durable duty transfer and expressly does not authorize stake withdrawal (`rotation_v3_8_retirement.tex:130–174`). The candidate's node-level continuation is compatible with that distinction.

> “Atomic ingress shutdown precedes release.” — candidate line 163

The stated guard accounts for complete requests already delivered or arriving during shutdown: they are persisted and can invalidate exit readiness. A failed send after transport shutdown is not reclassified as successful receipt by a correct owner. The client retry rule is an algorithm, not a newly assumed bounded ingress service.

For a node with ingress stopped and a finite pending set whose individual requests satisfy all inclusion premises, a finite maximum of their finite inclusion times discharges the request-specific guard. This conditional consequence is sound. It proves neither finite physical departure under perpetual ingress nor a departure deadline independent of request inclusion, nor discharge of unrelated archive/control/publication duties. It remains downstream of the inclusion proof gaps identified above.

### 12. Verification provenance and frozen sources

I read the proof-verifier skill and only the candidate and the permitted current/frozen protocol sources listed below. I did not open prior candidates, prior reviews, or author notes referenced by the candidate. A separate narrow verifier independently checked the finite-exception argument and the context barrier against the same permitted sources; its conclusions agree with F1 and F2. It made no source edits.

The current main's complete Model section was extracted and compared byte-for-byte with `revisions/v3_1_frozen/model_and_assumptions.tex`; they match exactly, including SHA-256 `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387`.

Paths below are relative to `formalization_v2/`. These hashes were recorded before review and rechecked after writing the review.

| Source | SHA-256 |
| --- | --- |
| `reviews/v3_8/v39_ownership_candidate_v2.md` | `ce76b4078e30e9882f4e49677e34af2cc514485052474082b7fca5070e54adfc` |
| `revisions/v3_1_frozen/model_and_assumptions.tex` | `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387` |
| `sync_hotstuff_rr_part2_rotation_v3_8.tex` | `c8611baf07c74f34512a182b0b3ace81d6f8f3cf65fb0cb567af696ec8a73574` |
| `sections/rotation_v3_8_requests.tex` | `dd588900506e2ef73e89a20eb57588976c2e8a9875f6258e268a87531bf98150` |
| `sections/rotation_v3_8_retirement.tex` | `984482c763cce1a9f324e416e78d1708f553acbe4a1798f7cafdc46ef289c238` |
| `sections/rotation_v3_5_ready_bundle.tex` | `4048a979add0bfe1662af309d5b30f7e72b105e8d5b6fda80de415ba43330152` |
| `sections/rotation_v3_8_liveness.tex` | `480cea02b3499f98d9e2b4b9b8eaaa2afd17ec55409c901b465b31758ff98541` |

Validation was a source-grounded manual proof/specification review, a byte-for-byte frozen-Model comparison, and source-hash verification. No manuscript source was changed. No repair, compilation, model checking, concrete network test, codec validation, or whole-protocol certification was performed.
