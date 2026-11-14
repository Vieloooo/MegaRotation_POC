# Summary

**Final Verdict:** The reviewed conditional safety argument is valid within its expressly stated abstract regional, complete-handler/Ready, cryptographic, source-history, and validation-interface premises; I found no Critical Error or Justification Gap in the REPORT-to-FINAL-to-global-prefix/retirement chain checked here. This is not an unconditional deployment verdict or a proof of full request liveness.

**List of Findings:** None in that conditional safety claim. The outstanding arbitrary-roster request-awareness obligation is expressly acknowledged in the candidate, rather than silently used as a proved premise. It remains an unfulfilled part of the broader request-inclusion objective. Concrete native-handler timing, codecs, storage capacity, and service implementations are not established by this source review.

## Review identity and boundaries

- Fresh adversarial review of the sealed English v3.8 snapshot. Read only `review_scope.md`, `manifest.json`, files under `snapshot/`, and the requested proof-verifier skill. No author notes, other review reports, or prior verdicts were consulted. No protocol repair was performed.
- Before reviewing, independently recomputed all 23 manifest SHA-256 hashes; all matched. Main SHA-256: `c8611baf07c74f34512a182b0b3ace81d6f8f3cf65fb0cb567af696ec8a73574`. The candidate's complete Model and assumptions section is byte-for-byte equal to `snapshot/baseline/model_and_assumptions.tex`. Literal equality alone is not the semantic verdict.
- The principal objects checked were REPORT formation and historical validation; original proposal/evidence identities and timers; FINAL origins and public kernel soundness; public `R0` versus regional `R+`; recovery locks; receipt-driven Completion; retirement; deterministic Force and regular-handoff races; and the use of these facts in the conditional request and DATA-progress arguments.
- Relevant inherited Part I dependencies were read in the supplied baseline, especially admission/evidence handling, terminal-QC dissemination, correct-leader deadlines, protecting QCs, recovery locks, unique extension, and the protected initializer window. Original native DATA/DRAIN soundness and service interfaces remain stated premises; this review does not claim a new independent proof of those interfaces.
- Validation method: source-level dependency checking and explicit analytic adversarial traces. No protocol implementation, model checker, runtime benchmark, or deployed binary codec was executed. The hash and literal-equality checks are executable checks; the protocol traces below are mathematical case analyses.

# Detailed Verification Log

All paths below are relative to `snapshot/`; `main` denotes `sync_hotstuff_rr_part2_rotation_v3_8.tex`.

## 1. Frozen assumptions and dependency map

**Location:** `baseline/model_and_assumptions.tex`, Stake and faults: “Every regional certificate needs matching signatures from distinct members with total weight greater than F”; “The bound F_i counts every key ... compromised while its certificates are still accepted.”

The threshold supplies at least one actual correct signer and lets correct weight alone form a certificate because `W-F>F`. It does not supply intersecting honest quorums. The candidate correctly uses synchrony, forwarded signed evidence, timers and locks for agreement, rather than replacing that reasoning with a threshold-intersection claim. The lifetime-compromise qualification is essential for the historical FINAL inference.

The checked dependency chain is:

1. Authentic boundary and immutable snapshot fix the regional context.
2. Full own-session validation, including REPORT witnesses and ancestry, precedes Vote and its timer.
3. Evidence-first handling plus the regional timing/lock argument makes surviving direct timer commitments compatible; retirement needs its separate early/late analysis.
4. Correct FINAL signatures originate only in such timer commitments. Receipt-driven commits emit no new FINAL.
5. A raw FINAL set therefore establishes existence of an actual valid historical input witness; it does not endorse an arbitrary publisher-selected history.
6. The public kernel plus independent current-branch, exact-ancestry, reference and Force-cutoff checks permits only the next global DATA.
7. Deterministic source replay and append-only transitions preserve the global prefix.

The kernel proof's provenance branch does not assume regional agreement to prove that a correct signer performed full validation. Agreement is used separately when competing admissible committed DATA must be identified.

## 2. REPORT eligibility, context and emission

**Location:** `sections/rotation_v3_8_requests.tex:58-66`: “The ancestry includes the inherited prefix and uncommitted speculative blocks”; “The index is never reused for another parent hash or native state.”

The eligible set is defined against the complete exact parent. A pending request already in an uncommitted parent cannot be selected again as primary. A parent change requires new filtering; an index from a different native state cannot be reused. This is sufficient for the primary's ancestry exclusion and immediate admission test at the abstract level. It supplies no additional measured processing bound.

**Location:** `requests:77-104`: the signed context includes the full domain, session, AUTH reference, `(g,t,j=0)`, opening identity, and parent's hash/height/physical position; “Historical validation never creates a missing fresh report.”

The signature prevents replay as a report for a different session, generation, term, opening, or parent. Alternate QC encodings do not create a different context. Fresh emissions require actual current unfrozen entry and the common retirement gate. Ordinary-term entry and complete term-zero NEWVIEW validation supply the two triggers. The bootstrap invokes the same ANCHOR-vote step (`main:2044-2052`), so initial generation zero is covered as well.

A correct member may report after validating a NEWVIEW even when its local lock prevents an ANCHOR vote. This introduces no lock bypass: reports do not form an ANCHOR QC, and the first DATA still needs that QC and all original parent checks. Reports have no entry, commitment, timer or Quit meaning.

## 3. Sidecars need not be included in the original DATA signature

**Location:** `requests:122-155`: “The original signed DATA, PROPOSE, votes, QCs and FINAL objects, and their hash encodings, are unchanged”; “Different sidecar signer sets or encodings for the same proposal are witness variants.”

Consider an adversary replacing sidecar `W1` attached to the same signed proposal with `W2`. Full validation checks `W2`'s signatures, exact historical context, distinct-member threshold, signed minimum, and equality of the selected request with the batch's first authenticated body. If `W2` passes, the unchanged DATA satisfies that witness too. If it fails, it cannot induce a Vote or a timer. This is a witness substitution, not a signed-DATA substitution.

The safety claim requires existence of an actual valid historical witness, not authentication of a particular later sidecar. Consequently the absence of a sidecar commitment in FINAL is not a safety defect in the stated claim. A stronger claim that FINAL authenticates the publisher's particular REPORT set would be false, but the candidate expressly disclaims it (`requests:184-191`; `receipt_kernel:121-130`).

A Byzantine nonselected report can contain an invented key whose full body other validators never fetch. This cannot invalidate the minimum argument: a fictitious minimum must match the actual first request body to pass; a larger unselected key cannot defeat the correct signer's upper bound on the minimum. The correct leader's own collection rule additionally validates every nonempty contribution it retains (`requests:106-120`).

## 4. Historical reports, malformed wrappers and first-DATA ancestry

**Location:** `requests:128-155`; `main:943-979`, `1080-1095`, `1133-1155`, `1313-1337`.

The common predicate checks signed historical reports, not recipients' current private queues or current global prefix. Thus a first-DATA proposal already voted by one correct member remains valid for another member after ordinary term advancement, despite newer request arrivals. Full validation of later DATA, recovery histories, DRAIN and own-session receipts also checks the required first-DATA ancestors. A later valid-looking proposal cannot omit that ancestor's REPORT sidecar.

Adversarial trace: a faulty leader signs conflicting blocks at the same admitted position, attaches a valid report witness to one and a malformed or missing report wrapper to the other, and gives them to different correct replicas. Before rejecting the bad wrapper or deduplicating its enclosing packet, receivers inspect the underlying signed material with its own admission/quota evidence. The original equivocation remains a Quit proof. REPORT invalidity does not hide it. Conversely, two sidecars for the same signed block do not invent equivocation.

Proposal identity remains `(beta,r,g,t,j,Hash(B))`. A duplicate witness cannot restart a timer. An invalid wrapper cannot permissibly suppress a later complete attempt before the replica has voted, because the normal-processing rule requires every admitted, not-yet-voted current-generation proposal to invoke Vote. Earlier terms remain eligible; old generations remain archival; future generations remain buffered. These distinctions preserve the inherited dissemination argument.

## 5. Timing and progress checks under the stated handler scope

**Location:** `requests:203-241`; inherited `baseline/sync_hotstuff_rr_part1_core.tex:630-710`, `870-894`; `main:1224-1241`.

The ordinary-term bound is correctly weakened from Part I's sharper handoff: a correct terminal signer forwards its complete proposal, correct votes make its terminal QC available everywhere by first correct entry plus `2 Delta`, reports reach the next correct leader by plus `3 Delta`, and its complete first DATA reaches all correct members by plus `4 Delta`. This does not require identifying an event called “all correct replies received.” Correct report weight alone exceeds F.

In term zero, prepared entry skew is Delta, NEWVIEW arrives by `e+4 Delta`, independent ANCHOR votes and reports arrive at the correct initializer by `e+5 Delta`, and first DATA votes occur by `e+6 Delta`. Every subsequent position costs at most `2 Delta`. The original first progress deadline is `e_v+6 Delta`; message-first handling covers equality. Reports do not suppress the original checks while the leader is silent or the opening is unknown.

These bounds are conditional on the expressly described complete-handler preparation and active, unfrozen, unclosed interval. They do not prove a bound for an additional request-pool scan, native import or eligibility rebuild left outside that handler (`requests:68-75`; `ready_bundle:107-146`). No unprovided native benchmark was treated as a proved fact.

## 6. Protecting QCs, recovery locks and REPORT changes

**Location:** inherited Part I `733-831`; `main:1133-1183`, `1243-1309`; `requests:231-241`.

Before retirement intervenes, the new report predicate is a common historical validity condition carried in complete bundles. It does not add a private veto to the correct recipient's vote. Thus the original argument applies to an augmented-valid proposal: a correct timer surviving to `s+2 Delta` rules out a correct Quit at or before `s+Delta`; all correct recipients can cast or have already cast its vote by `s+Delta`; the protecting QC arrives before the timer expires. An incompatible signed history would instead expose the original Quit proof in time.

The recovery lock is snapshotted only after Quit's Delta wait and any authority wait. Reports neither rank above that lock nor substitute for ANCHOR approval. DATA/DRAIN parent justifications still descend in rank. Earlier-term votes, prior ancestor commitments and sidecar variants do not change these rules. Completion and closure require the additional argument checked below; I did not apply Part I's universal protecting-QC lemma through those new suppression cases.

## 7. Raw FINAL origins and absence of receipt circularity

**Location:** `sections/rotation_v3_8_receipt_kernel.tex:15-90`; `main:837-893`, `1158-1164`, `1185-1197`.

“This step uses the raw threshold, not the definition of a valid regional receipt” correctly separates signature verification from the validity conclusion. More than F distinct fixed-snapshot weight includes a correct signer. Unforgeability and the lifetime key bound turn that signature into an actual FINAL action on the exact named DATA and full signed session/domain context.

That correct action follows a timer-origin commitment, either directly or as an ancestor. Vote started the timer only after full own-session validation, including all needed REPORT witnesses. Receipt-driven commitment and terminal Completion emit no FINAL. Therefore a cycle consisting solely of receipts and Completion cannot create the first correct signature.

Inherited-anchor uses are well-founded by event order: a source FINAL signature used by a later Vote already existed before that Vote; its timer origin is an earlier event. This covers direct REGULAR handoff before global acceptance as well as accepted inherited anchors. It does not assume global acceptance of the anchor merely to establish its intrinsic historical input validity.

The timer commits the same-session ancestors on its hash-linked path. The public kernel checks actual DATA/DRAIN bodies, positions, links, same-session coverage and quota. A later session's FINAL cannot cover the previous session's DATA. The result establishes hash-bound input validity; it does not validate an arbitrary unbound execution output or state snapshot (`receipt_kernel:155-164`).

## 8. Same DATA body under different generation/term contexts

**Location:** `receipt_kernel:121-142`; `requests:184-191`, `274-289`; `main:795-814`.

FINAL deliberately omits generation and term. If the same DATA body appears under different proposal contexts, its raw receipt consequently does not choose between those contexts. The soundness proof instead selects the actual correct signer's timer history, with its full contextual validations. That is enough for the claimed existential historical witness and for the fixed native input path.

The public interpreter must not infer fresh reports or request awareness from a publisher's term annotation; the candidate does not do so. A different generation, term, parent or session requires a matching contextual REPORT witness for full regional validation. Old all-EMPTY reports remain usable only for their original context; later local request arrivals do not retroactively invalidate them. That bounded historical reuse is compatible with safety and with the conditional request theorem's actual quantifier.

## 9. Regional R+ and public R0

**Location:** `requests:157-199`; `sections/rotation_v3_8_carrier.tex:17-58`, `108-169`; `sections/rotation_v3_8_exports.tex:201-238`.

The erasure operation removes only the newly framed REPORT sidecars. Every original DATA, signature, proposal, QC, ancestry edge, admission/source object and baseline public denotation remains in R0. The service's input is still the original complete regional result, not a bare raw-FINAL kernel. Correct regional senders retain R+ for own-session recovery, receipt processing and Completion, while publication projects R0.

This preserves original public size and load scope for the same original value; it does not assume extra public capacity for REPORT bytes. The selected primary occupies ordinary batch capacity. Logical normalization preserves actual objects and source meanings, does not manufacture absent payloads, and does not let wrong outer hints or unrelated invalid proofs hide a complete neighboring kernel. Optional regional witnesses are not public-kernel fetch gates.

No deployed byte codec is specified. Therefore the structural claim is correctly limited to the defined logical value. A private-only cache reference does not become public merely because a regional receiver can resolve it. These are material scope conditions, not evidence that a concrete codec or deployment has passed review.

## 10. Completion cannot silently defeat the early vote window

**Location:** `sections/rotation_v3_8_retirement.tex:24-41`, `243-279`, `333-367`.

Fix a direct timer voted at s that actually survives until `s+2 Delta`. If a correct member Completes at `b <= s+Delta`, its mandatory complete terminal-receipt relay reaches the committer by `b+Delta <= s+2 Delta`; message-first processing invokes Completion and cancels the chosen timer. That contradicts survival. The proof uses receipt delivery and cancellation, not a prior assertion that receipt-based commitments are already safe.

The entered-cache issue is separately addressed: complete Ready/Enter delivery bounds entry by first entry plus Delta; a correct FINAL cannot exist earlier than a timer at first entry plus `2 Delta`. This step remains explicitly conditional on the Ready contract. Receipt-based commits subsequently inherit the already established direct-timer provenance. A canceled DRAIN timer does not introduce a fresh DRAIN commitment.

## 11. Closed/RequestClose races and surviving timers

**Location:** `retirement:43-120`, `176-241`, `281-331`, `375-401`.

The distinction between branch-wide Closed and the first complete per-session RequestClose time c is essential. The manager retains complete closure evidence even without a worker. If an arriving complete proposal first supplies the fixed snapshot, the receive handler forms and relays the complete close bundle before rejecting entry or returning a tombstone. Thus suppressed matching votes by `s+Delta` imply `c <= s+Delta`; an unknown old worker cannot silently disappear from the protecting-QC case.

Early case, `s+Delta < c`: closure cannot suppress the matching vote, and survival excludes early Quit and Completion. Correct votes supply the protecting QC before later live recovery locks.

Late case, `c <= s+Delta`: a protecting QC need not form. Nevertheless any later-generation correct entry requires first Quit at `a > s+Delta`, and hence entry strictly after `s+2 Delta`. The complete close relay has shut all correct entry gates by `c+Delta <= s+2 Delta`. Consequently that later entry and its correct signatures cannot occur. No later QC can exploit the missing protecting QC.

Concrete analytic negative control: with three unit-weight members and F=1, H0 closes at c=0, H1 receives closure at Delta, and a Byzantine leader gives H1 an otherwise valid proposal at `0<s<Delta`. H1 can vote and later commit while H0 declines its matching vote. With Byzantine votes/FINAL withheld, no QC or receipt need form. The candidate explicitly permits this trace and preserves the local commitment; it does not falsely invoke complete-result publication or Completion.

Conflicting same-generation correct votes still forward incompatible original signed histories. Evidence-first handling and the retained owner deliver the contradiction by the surviving timer's expiry. Closure blocks fresh signing but does not suppress evidence inspection or genuine Quit.

## 12. Retirement drain, sidecar retention and worker exit

**Location:** `retirement:122-174`, `201-241`; `main:1066-1078`; `requests:23-44`, `163-199`.

Complete close relay shuts all correct gates by `c+Delta`; an already-started timer therefore expires by `c+3 Delta`, and its actual FINAL sends arrive by `c+4 Delta`. A local `c_v+4 Delta` drain threshold is conservative. It is not a receipt-formation, storage-copy, publication or acceptance deadline.

FinishClose additionally requires timer resolution and durable ownership of REPORT sidecars, required validation evidence, outgoing relays, receipts and publication duties. A bare Closed observation before complete per-session context does not start the drain clock. Late messages still reach persistent owners. Fresh REPORT signatures use the same gate as other fresh actions; forwarding an existing report or request does not reopen that gate. The once-per-request/snapshot relay survives worker exit and is distinct from a fresh BFT signature.

## 13. Independent public checks and Force/regular races

**Location:** `exports:54-155`, `164-199`; `sections/rotation_v3_6_control.tex:207-295`; `main:1808-1871`, `1900-1988`, `2081-2122`, `2473-2578`.

The public kernel is only one acceptance premise. The interpreter also fixes the incoming boundary/region, current open branch, exact extension of G, next DATA height, quota, strict precedence of selection references, and the post-Force containing-block cutoff. Altering a report wrapper, choosing a different FINAL encoding, or supplying an alternative generation cannot override those checks.

Analytic race checks:

- Terminal DATA accepted before Force advances the public duty and its accepted-count context; the later Force uses that actual context and all of pre-payload G.
- Force in the containing block's prelude closes the old version before its payload. A late terminal receipt or speculative regular successor cannot append its closed suffix. New-version DATA also cannot be accepted in that triggering block.
- With zero accepted DATA, Force retries the same slot; with positive accepted DATA it targets the next slot. Repeated Forces while unassigned preserve the stored target rather than inventing empty sessions.
- A due matching MSIG takes precedence over AUTO. Other due MSIGs are consumed as stale; stale attached configuration has no effect. Equal timestamps do not repeat controls, and a new duty is not settled again in its opening prelude.
- A regular boundary remains authenticated historical evidence after OFF or closure, but it is not live entry permission. The separate CanEnter/Closed checks block reopening. Cached old callbacks cannot rewrite a fixed new anchor or reset entry time.

Per-DATA durable commits occur before successor-authority resolution. A later invalid candidate or genuine boundary wait cannot roll back already exposed G. Common canonical source order and deterministic subtransitions therefore give comparable prefixes even when observers stop at different subpositions within a record. Retirement does not itself mutate G or select a second retained anchor.

## 14. Conditional request potential and the unrepaired full objective

**Location:** `requests:243-301`; `sections/rotation_v3_8_liveness.tex:446-495`.

The request theorem assumes infinitely many accepted term-first DATA with a genuine witness containing an aware correct signer. If x remains absent from G, that signer selects y with `key(y) <= key(x)` and the witness minimum z satisfies `key(z) <= key(y)`. Full validation requires z's actual first body and excludes its identity from exact parent ancestry. Acceptance includes that ancestry, so this spends a previously unaccepted identity.

There are finitely many canonical byte strings with length at most x's length, hence finitely many relevant logical identities. Their number absent from G is a nonincreasing finite potential, strictly decreasing at each counted accepted first DATA while x remains absent. Discarded speculative output does not spend it. An accepted predecessor cannot be spent again because every later accepted parent includes it. Force never rolls back G.

The bounded K positions of a genuine term prevent an infinite accepted DATA chain from being supported by only finitely many first-DATA ancestors. FINAL's missing generation/term does not authorize using an arbitrary publisher annotation in this counting; actual signer histories supply the genuine witnesses.

However, repeated accepted regional DATA service does not imply that these witnesses contain informed correct signers. With successive disjoint rosters, each may issue immutable EMPTY reports before later receiving x, then finish its service before the next disjoint roster learns x. Old owners' retention and same-snapshot relay do not supply that next roster. The candidate explicitly leaves this obligation open. It is not a Critical Error in the stated conditional theorem; it prevents claiming the broader request-liveness objective has been achieved.

## 15. Effect on abstract DATA progress and final scope

**Location:** `liveness:61-158`, `162-356`, `391-444`, `497-516`.

The REPORT rule preserves the original fresh-position deadlines in the prepared correct-leader cases and leaves progress checks running in faulty terms. A correct terminal vote still forwards every validation dependency, including historical REPORT evidence, so the terminal-QC/advance-or-Quit analysis remains available. The protected correct initializer still obtains its first proposal by `e+5 Delta` and a complete receipt by `e+9 Delta` under the stated uninterrupted assumptions.

The global argument uses original complete-result publication, retained public bytes, prepared source handlers, and the strict Force cutoff; no request-awareness fact is substituted for those premises. The quadratic timestamp-versus-linear-real-time contradiction and owed-service arguments concern accepted DATA, including empty DATA. They do not imply request inclusion. Their source-handler and Ready timing remain abstract conditional contracts, not a proof for arbitrary native import or historical replay costs.

**Disposition:** Conditional safety passes this fresh review. No protocol edits are proposed or performed by this verifier. Full arbitrary-roster request liveness remains open, and concrete implementation/service validation remains outside the evidence reviewed.

## Final snapshot integrity check

After completing the analysis and writing this report, independently recomputed every manifest hash again: **23/23 match**. The main remains `c8611baf07c74f34512a182b0b3ace81d6f8f3cf65fb0cb567af696ec8a73574`. The report itself is outside the sealed snapshot. No snapshot file changed during this review.
