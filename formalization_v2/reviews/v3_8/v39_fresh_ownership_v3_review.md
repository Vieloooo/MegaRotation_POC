# Fresh independent verification: ownership candidate v3

## Summary

**Final Verdict:** The ownership/discovery and finite-exception arguments are **conditionally valid at the candidate's declared abstract interfaces**; this review finds no Critical Error or additional Justification Gap in those scoped arguments, and does not establish the deferred interface realizations or the complete request-liveness/whole-protocol goal.

**List of Findings:** No in-scope Critical Error or additional Justification Gap found. This is not an unconditional approval of Section 8 or an implementation verdict. The following are explicit limits of the statement being verified, not facts proved by this review:

- **Mixed-snapshot domain.** Candidate lines 23–27 and 202 expressly grant the old-owner/new-member service conditionally. Frozen v3.1's within-region sentence is not used to establish that edge.
- **Complete source-authentication interface.** Candidate lines 35–37, 79–82 and 204 condition the context barrier on the actual supplied source-proof predicate and cached dependencies fitting the original complete-message handler. This review verifies the resulting schedule. It does not prove an arbitrary from-genesis archive validation or catch-up job fits that handler.
- **Ready and report integration.** Candidate lines 31, 141–145, 184 and 206 retain the augmented ReadyBundle and exact-parent report-processing obligations. HYDRATED establishes complete retained bytes before Ready; it does not alone establish the eligible projection used by INPUT_REPORT. The downstream request proof remains conditional on those obligations.
- **Resource and lifecycle realization.** Candidate lines 29–33, 190–208 require admissible message encodings, continued dispatch within the unchanged aggregate envelope, and continuing owners while pending bytes matter. No bounded storage, practical denial-of-service resistance, unconditional physical departure, or uniform future-roster latency follows.

The review applies the `proof-verifier` skill in verifier-only mode. It proposes no repair and edits no manuscript. It reads the v3 candidate itself and the permitted frozen/current source interfaces; it does not read v1/v2 candidates, prior reviews, companion notes, or other agents' findings. The candidate's own account of previous work was not treated as evidence.

## Detailed Verification Log

### 1. Statement, dependencies, and conditional domain

> “A correct target member signs HYDRATED only after its own finite query has obtained every correct historical owner's cut” (candidate line 15).

The literal coverage claim must be read with lines 27, 51, 91 and 106: an owner that retains a globally unaccepted request is a persistently correct serving owner covered by the granted service; a retired empty owner need not answer. A request pending before the correct signer's query is included in its retained journal cut unless it has become globally accepted. The candidate does not claim a cut of all requests received before first DATA acceptance. Line 19 correctly excludes that stronger, causally unsupported guarantee.

The dependency chain is: authenticated finite directory and continuing original owner → completed receiver context → immutable finite cut and fragments → all-target byte dissemination → a valid threshold certificate containing a correct signer → finite exceptional authority set → report-awareness implication, subject to the report interface. The final step is not obtained merely from a threshold acknowledgement.

> “This premise is not derived here from frozen v3.1” (line 23).

Accepted as a conditional premise only. Frozen Model lines 360–372 give authenticated channels, a common clock, and a within-region complete-message delivery/processing bound. They do not independently identify every mixed-snapshot endpoint pair. The finite maximum `d_i(r)` is meaningful only once the stated service covers the exchanges. The candidate explicitly retains this distinction. Nothing here creates cross-region transport or a bare-request publication guarantee; frozen Model lines 307–319 and 374–389 restrict the publication input and cross-region scope.

### 2. Directory, retained ownership, and local finiteness

> “D_i(r) = union of the regional identities in AUTH[k,0], 0 <= k <= r” (line 43).

For a finite numeric r and finite rosters, this is a finite, immutable directory. Frozen Model lines 223–237 make AUTH immutable and independent of beta; recovery authority changes only the schedule. Retaining historical authority evidence permits historical signature checks without a private current-owner oracle. A new roster need not be selected or run BFT merely to appear in the directory.

> “The original owner retains the pending bytes even after one or all target rosters receive them” (line 67).

This is sufficient to avoid the moving-custodian problem. For the fixed request in Section 7, its original correct owner remains a usable source until verified inclusion in G. Copies and acknowledgements cannot remove the last required original pending bytes. This is a protocol duty of the hypothesized continuing owner, not a proof that an arbitrary machine stays online.

> “Initialize ReceiptJournal as a finite sequence … Each complete first-request handler appends at most one finite entry” (line 61).

The stated local transition invariant is valid. A reached local state starts finite; a first-receipt append, a duplicate, a frozen cut, and an indexed record access preserve finiteness. The maintained length/root makes taking a cut an indexed-state operation, rather than a queue scan charged to the response slot. There is expressly no limit transition installing an infinite journal after infinitely many local predecessors. This is an ordinary finite-state-object operational boundary, not global temporal local finiteness. It does not prove bounded memory or bound the number of events at unrelated nodes.

The correct-query processing bound is relevant here: a required correct-to-correct handler cannot be deferred behind an unfinished infinite local prefix while still satisfying the assumed complete-message service. Conversely, the candidate is not proved for an alternative machine model admitting infinite initial/reached local objects. Lines 63 and 204 state that limit explicitly.

### 3. Full context transfer and the source-authentication barrier

> “Every referenced validation dependency is supplied as bytes” (line 77).

The capsule construction is an actual pre-query task, with no uniform acquisition deadline. It includes the initial trusted checkpoint dependencies, finalized source evidence, AUTH prefix through r, historical identities, and exact target roster. No fresh receiver cache is assumed. The candidate requires a completed finite capsule before starting its timed schedule. Frozen Model lines 280–297 provide the checkpoint basis; lines 424–427 warn that arbitrary historical catch-up has no constant-time bound.

> “Let a be v's clock time of the last actual capsule-envelope send” (line 78).

Given the fixed finite length, fixed chunk count, and continued actual dispatch, all correct covered owners have all raw capsule bytes by `a+d`. The definition includes the final envelope to every directory owner. It needs no receipt or acknowledgement, so a Byzantine directory member cannot block a by withholding a reply. Raw authenticated storage is explicitly allowed before target-authority validation; it is not entry authorization.

> “At or after a+2d … sends … CONTEXT_VALIDATE” (line 79).

Since `d>0`, raw delivery/processing precedes each validation trigger strictly. The operation specified at the receiver is source-proof authentication over already cached complete dependencies. It does not include executable-state restoration, application replay, a missing-source fetch, a database import, or a hidden current-owner search. Current retirement lines 57–83 provide a relevant existing pattern: a phase-independent manager authenticates a supplied historical snapshot/context, even without a worker or prior context cache. ReadyBundle lines 38–47 and 74–79 likewise require full source dependencies.

The necessary limitation is precise. The frozen complete-message sentence is not a complexity theorem for an arbitrary capsule format or arbitrarily large verification job. Lines 35–37 and 82 explicitly require that this actual source-authentication predicate and its cached dependencies are admitted by the old complete-message interface. Under that condition, the check can be charged to d. If a concrete adapter performs additional unbounded history validation outside that interface, the argument stops at this step; chunking its bytes does not repair its verification deadline. The candidate does not assert otherwise. I therefore accept this as a conditional interface application, not as proof of every concrete source-check implementation.

> “s_v >= max(a'+2d, release(r))” (line 80).

Every trigger has actually been sent by a', hence every required correct receiver has stored ContextReady by `a'+d`, strictly before `s_v`. This proves the start barrier from the correct sender's clock and the stipulated handler contract. No knowledge of which directory members are correct, remote completion notification, or Byzantine ACK is needed. Per-owner digests and persistent ContextReady records bind the target; another target's callback does not satisfy the guard.

### 4. First-query pacing and per-owner descriptor deadlines

> “recording each actual send time s_{v,p}” (line 84).

The distinction between the first multicast send s_v and each owner's actual send time is essential and is present. A delayed send to p receives its own deadline `s_{v,p}+2d`; the candidate does not spend p's reply window while v is still dispatching earlier recipients.

The receiver rejects a query processed before `release(r)` and neither records a cut nor registers its target. A rejected early packet is not retained as deferred authorization. Only the authenticated named v can create `Cuts[(i,r,v)]`, so another sender cannot pre-create a correct member's cut. The sender's clock gate is sufficient for the finite-exception proof; the receiver gate also prevents premature Byzantine registration/cut creation.

> “atomically retains its current finite ReceiptJournal prefix” (line 87).

Registration and cut creation are in the same first valid handler. A request is either in that prefix or is a subsequent live arrival with an already registered target. New arrivals cannot increase m, replace the cut root, or reset the job. Repeated beta attempts reuse the same key.

A correct query is processed by `s_{v,p}+d`; its immediate complete descriptor reply is processed by `s_{v,p}+2d`. Processing messages due at the deadline before expiry admits the endpoint equality. A silent or equivocating Byzantine owner contributes at most one frozen descriptor or none. A finite directory yields finitely many descriptor deadlines. No “all correct replies received” event is required.

### 5. Metadata, fragment, and checking budgets

> “derive n once” (lines 96 and 108).

For a frozen descriptor with finite m, each indexed record has one metadata slot. Its first accepted complete metadata fixes finite L and H, hence `n=ceil(L/b)`. Fragment slots bind the complete cut/owner/index/length/digest/fragment context. The exact payload-length rule covers both nonfinal and final fragments; L=0 receives its explicit final-check branch.

Unrelated, wrong-sender and unauthenticated packets cannot fail an honest owner's record. An authenticated malformed response from the actual expected owner can fail its own Byzantine record, which is an allowed omission. Slots never restart, later roots/counts never replace the frozen descriptor, and late fragments cannot enlarge the attempt. Signed tombstones do not prove new G inclusion and may only remove coverage for a request that is already accepted, or omit a Byzantine source's contribution.

For a still-unaccepted request at a correct owner, every metadata and fragment reply fits its query/reply deadline, so the exact retained bytes reconstruct. The final canonical/authentication checks are actual terminating local work before Pool_v is frozen and disseminated. They are not incorrectly given a new Delta bound.

Large Byzantine finite m or L can cause very large finite work. That does not falsify this eventual finite-job argument, but neither a practical work bound nor overload resistance follows. The same restriction applies to capsules and outgoing records. Finite jobs require the specified continued dispatch and admitted load profile; they do not establish that profile for an implementation.

### 6. Dissemination, shares, and collection

> “v sends its full bytes to every target identity” (line 114).

This is an actual finite multicast of a frozen pool, including each record's complete metadata and fixed fragments. Out-of-order retention and sender/context/index deduplication are specified. A job merely queued at the sender cannot count as a completed send. Recipients keep the bytes at the node-level manager even before entry or after a worker's exit.

> “Only at or after b_v+2d may v create its signed share” (line 118).

Every needed envelope is delivered and processed by `b_v+d < b_v+2d`. Thus byte availability at all correct target members strictly precedes a correct share, and any certificate containing that share. This addresses equal-timestamp ordering without assuming a positive actual network delay. The empty-pool branch defines b_v and remains finite.

This proves retained byte availability under the stipulated per-envelope storage/reconstruction interface. It does not prove arbitrary receiver-side whole-record validation finishes by waiting 2d. Lines 31 and 116 explicitly exclude that inference; application eligibility after an exact future parent remains a separate report obligation.

> “immediately executes an authenticated multicast of the full signed share to every target member” (line 126).

The share dispatch, distinct-signer collectors, threshold formation, and full-certificate once-per-context relay are all actual specified actions. Persistent unfinished sends survive worker exit and beta changes. A bare root is insufficient. Full-certificate fragmentation has a frozen finite budget.

Under the snapshot fault model, every `>F` certificate contains a correct signer. Correct weight is `W-B >= W-F > F`, since `B<=F<W/2` (frozen Model lines 328–343). If no certificate yet exists, each correct target continues its finite job and broadcasts; a finite maximum of their completion and delivery times gives certificate formation without Byzantine cooperation. If a valid certificate exists earlier, adoption cannot invalidate its correct signer's already completed dissemination. Cancellation preserves promised serving/relay/storage duties.

The hydration lemma therefore follows. No honest intersection between the HYDRATED signer set and a later REPORT signer set is required: the correct HYDRATED signer sent the actual bytes to every correct target member first.

### 7. Ready skew, fixed-slot progress, and pushes

> “The first ReadyBundle supplies the complete certificate” (line 141).

The certificate gate is before the first Ready/Enter event. A correct entrant cannot exist without a certificate containing a correct share, whose required byte dissemination is already complete. A slower target's private acquisition is consequently not a missing pool-transfer dependency after first Ready. The complete certificate must accompany the ReadyBundle; a pointer is insufficient.

Under the explicitly retained augmented ReadyBundle processing contract, the first entrant's actual send at e yields other permitted entries by `e+Delta`, as in ReadyBundle lines 107–146 and current main lines 2058–2067. Historical d is spent before Ready; it does not replace the target's Delta. This preserves the *form and scope* of the old skew argument, not unconditional feasibility of a larger encoded bundle or deferred request processing.

> “In the stalled-G proof, r is fixed” (line 145).

The claim is supported by current main lines 1808–1839 and current liveness lines 44–59. For a fixed slot, release(r) is finite; the directory, per-target cuts, metadata counts and local preparations are finite. Their completed hydration can enter the existing preparation maximum, while beta retries retain it. No public window is paused or reset, and no whole-pool delay is inserted into the post-Enter response constant. This does not bound initial context acquisition or native preparation.

Subsequent pushes need not reopen a cut or revoke HYDRATED. They are finite message sequences and eventual continued duties, not one arbitrary whole-object transfer charged to d. The arbitrary-future-roster proof below primarily uses the original retained owner and later cuts, so it does not depend on racing every fresh push ahead of a report.

### 8. Finite early authority indices without global temporal local finiteness

> “tau_0+r*tau_p <= s_v <= t_x” (line 157).

For fixed finite t_x and positive tau_p, this implies the displayed finite set E_x; if `t_x<tau_0`, E_x is empty. Including equality correctly handles a receipt and a first query recorded at the same clock time. There is at most one correct first query per `(i,r,v)`, and every target roster is finite, so the sum of its possible early correct cuts over E_x is finite. Infinite activity at unrelated actors and Byzantine early attempts cannot enlarge this set.

> “Discard targets r<k and those in E_x” (line 165).

Outside this finite set, every correct signer's query begins strictly after t_x, p is in the authenticated directory, and p still retains x under the contradiction hypothesis. Its immutable cut contains x; successful finite acquisition and prior all-target dissemination put x at every correct target before that signer's share. Older correct shares usable in a late certificate can only be exceptional in the same sense.

Current main lines 1815–1839 ensure zero-output Force retries reuse r, while a positive-output Force advances it; normal completion advances r too. Finite quotas therefore imply only finitely many accepted DATA contributions from each exceptional r. Infinitely many zero-output retries contribute none. This establishes all-but-finitely-many accepted-contribution byte availability without a finite union of future memberships, uniform future delay, or a global finite-event-rate premise.

The local journal invariant and the paced-index argument perform different tasks. The former makes each actual cut finite; the latter bounds how many target indices could predate the fixed request. Neither is substituted for the other.

### 9. Request inclusion: correct conditional implication, explicit remaining interface

> “Some correct signer in every report certificate then supplies an eligible candidate y <= x” (line 182).

This implication requires x to be in that signer's exact-parent eligible set at its report event, not merely present as unvalidated raw bytes. Current requests define that set using full bytes, admission immediately after the parent, individual DATA fit, and ancestry exclusion; the parent/request processing must finish those checks before exposing the projection. Candidate lines 31, 141–143, 184 and 206 explicitly retain this integration obligation. This review does not discharge it by calling HYDRATED an application-validity certificate.

Conditional on that interface, x's continued admissibility/fit and absence from G give the required eligibility at each accepted counted parent. If x were already in its accepted ancestry, x would already enter G. The correct reporter chooses y no greater than x. A valid first DATA executes a real fresh minimum primary z no greater than y, regardless of whether the proposer is Byzantine. Ancestry exclusion prevents spending one logical primary twice in accepted G.

The candidate's example order `(canonical length, canonical bytes)` has a finite lower set for every fixed request. The current requests use `(canonical length, hash)` with binding canonical identity; the same bounded-length finiteness property is sufficient. Exact order agreement is an integration detail to retain when applying the change; no new timestamp-comparability premise is needed. Infinitely many fresh accepted predecessors at or below x are impossible. Current requests' request-inclusion proposition and its finite-term counting support this conditional last step.

Thus no further roster-awareness counterexample remains once the stated hydration and report interfaces hold. This is not a whole-protocol certification of report timing, final receipt validity, source-kernel use, or encoding/load headroom. The candidate expressly withholds that certification.

### 10. Physical lifecycle and final scope

> “Worker Terminated never stops this owner service” (line 188).

This agrees with current retirement lines 130–174: worker exit requires persistent data/control/request duties and does not authorize stake withdrawal. The candidate extends the pending-request ownership duty through global inclusion, so a copied pool or certificate cannot authorize disappearance of the still-needed original source.

The shutdown rule first closes ingress atomically, drains complete already-delivered requests, and persists an arrival during shutdown before deciding the exit guard. This rules out losing an actually received request at the worker/node boundary. A failed send to a departed endpoint is expressly outside the actual-receipt premise. Correct-client retries do not become a new bounded ingress service.

For a finite retained pending set after ingress stops, conditional individual inclusion times have a finite maximum and eventually discharge the request-specific exit guard. Other archival duties may still prevent physical departure. Indefinite new arrivals, permanent denial of the serving premise, arbitrary endpoint failure, fixed storage limits, and uniform latency across all future AUTHs are outside the conclusion, as explicitly stated. Those are scope limits, not newly found in-model counterexamples.

### 11. Input integrity and validation record

The table records full-file SHA-256 values before substantive verification and again after reading/verification, before writing this review. The candidate matches the assigned hash exactly. Only the frozen file's Model section (lines 194–514) was used as frozen semantic evidence; source-location searches were used to identify the current interfaces. Current main reads covered stake, lifecycle, Force slot reuse/advance, Ready/Enter, and termination. The current main's Ready input resolves to `sections/rotation_v3_5_ready_bundle.tex`.

| Input under `formalization_v2/` | Before SHA-256 | After SHA-256 |
| --- | --- | --- |
| `reviews/v3_8/v39_ownership_candidate_v3.md` | `a71b64b8bb845463987de8d79b92fed6d8ef61db7a60eec0719dfe1b77a67bea` | `a71b64b8bb845463987de8d79b92fed6d8ef61db7a60eec0719dfe1b77a67bea` |
| `revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex` | `483bdfbf3668a0f892af38df264655e54e0dbd73a1c26fbd45111d1730cec034` | `483bdfbf3668a0f892af38df264655e54e0dbd73a1c26fbd45111d1730cec034` |
| `sync_hotstuff_rr_part2_rotation_v3_8.tex` | `c8611baf07c74f34512a182b0b3ace81d6f8f3cf65fb0cb567af696ec8a73574` | `c8611baf07c74f34512a182b0b3ace81d6f8f3cf65fb0cb567af696ec8a73574` |
| `sections/rotation_v3_8_requests.tex` | `dd588900506e2ef73e89a20eb57588976c2e8a9875f6258e268a87531bf98150` | `dd588900506e2ef73e89a20eb57588976c2e8a9875f6258e268a87531bf98150` |
| `sections/rotation_v3_8_retirement.tex` | `984482c763cce1a9f324e416e78d1708f553acbe4a1798f7cafdc46ef289c238` | `984482c763cce1a9f324e416e78d1708f553acbe4a1798f7cafdc46ef289c238` |
| `sections/rotation_v3_8_liveness.tex` | `480cea02b3499f98d9e2b4b9b8eaaa2afd17ec55409c901b465b31758ff98541` | `480cea02b3499f98d9e2b4b9b8eaaa2afd17ec55409c901b465b31758ff98541` |
| `sections/rotation_v3_5_ready_bundle.tex` | `4048a979add0bfe1662af309d5b30f7e72b105e8d5b6fda80de415ba43330152` | `4048a979add0bfe1662af309d5b30f7e72b105e8d5b6fda80de415ba43330152` |

The exact frozen Model text, lines 194–514 with original line endings, hashes to `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387`. Verifier guidance `/Users/jin/.codex/skills/proof-verifier/SKILL.md` hashes to `f5898b3fe46b49946dc761e66ab0c013b6b7f23e10e8b3a0e8f267605c4ce49c`.

Validation performed: source-grounded manual verification of the stated proof chain, endpoint/equality cases, Byzantine omission/equivocation cases, fixed versus growing budgets, certificate formation/cancellation, beta reuse, and finite-exception counting. No compile, executable test, model check, transport/CPU benchmark, concrete codec audit, or whole-protocol verdict was performed. No manuscript, candidate, frozen assumption, or prior review was modified.
