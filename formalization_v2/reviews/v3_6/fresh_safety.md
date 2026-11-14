# Fresh v3.6 safety and replay verification

## Summary

**Final Verdict:** The conditional global-prefix, handoff-isolation, and regional-retirement safety arguments are valid at the specified abstract state-machine level; this review found no Critical Error or material Justification Gap in those arguments and no model-compliant fork or rollback trace. This is not an unconditional implementation or full-liveness certification.

**List of Findings:** None in the reviewed safety and replay claims. The transport grammar and complete-message/native-state contracts below are explicit scope boundaries, not properties established by a hash check or by an Espresso finality certificate. Request inclusion remains unresolved in the candidate itself.

**Frozen artifact and method.** Reviewed only `snapshot/` and the review scope. All 21 file hashes in the parent `manifest.json` match, including main SHA-256 `ef6d858b8ead057dd77f1a43e4198d47ceceb016e5c7b826eb941878b59d96d5`; all four nested baseline-manifest hashes also match. Read the full active main source, all seven active section fragments, the five active figure-source files, full inherited Part I, and full v3.1 baseline. For v3.1, exact line matching identified shared text already read in the candidate, and every differing baseline passage was read separately. The frozen Model fragment is an exact substring of the candidate. The two included figure PDFs were hash-verified; this was not rendered-layout QA. No source changes, executable protocol tests, model checking, or external-reference assumptions were introduced.

All line references below are relative to `snapshot/`. Abbreviations: `main` is `sync_hotstuff_rr_part2_rotation_v3_6.tex`; `exports`, `source`, `control`, `retirement`, `ready`, `health`, and `liveness` are the corresponding active files in `sections/`.

## Detailed Verification Log

### 1. Assumptions and dependency map

> “Regional receipt validity and the application-validity checks below are assumed sound under the stated regional model.” (`main:503–509`)

The prefix theorem is conditional on authenticated compatible source histories, complete applicable controls, advance announcement, binding hashes/unforgeable signatures, correct-weight bounds, and sound regional/application validation. The regional bound includes delivery and processing of complete validation dependencies (`main:353–371,377–397`). I did not construct attacks by withholding a correct-to-correct complete message beyond that bound, forging finalized control history, changing immutable authority, exceeding the snapshot fault bound, or allowing a valid timer to survive a timely genuine Quit or Completion.

The relevant dependency chain is:

1. Fixed authenticated incoming boundary and immutable session authority.
2. Regional DATA/DRAIN safety, supplemented by the retirement proof when its gate suppresses new votes.
3. At least one correct FINAL signer and same-session receipt coverage.
4. Common source-object inventory and closure-completeness predicate.
5. Deterministic next-height validation, atomic DATA/accounting commit, and deterministic boundary/control transitions.
6. Prefix comparability and irreversible content/provenance at all committed subpositions.

The candidate explicitly retains the conditional nature of the regional interface rather than claiming the global-prefix proof independently re-proves all of Part I (`main:2538–2544`). The retirement proposition separately assumes the complete-message Ready/Enter installation contract (`retirement:238–247`). These premises must remain visible in any final verdict.

### 2. Inner signed domain versus outer EXPORT hints

> “Signed result contents and authenticated source context determine the chain, producing pair, ancestry and append permission.” (`exports:7–12`)

This rule is consistent with the signature domain in `main:784–803`: signatures bind chain, protocol, pair, configuration, region and message type, and the boundary must actually authorize the pair. The acceptance gate checks that signed domain and the current open version, then the incoming duty and exact current ancestry (`exports:84–98`). A false outer chain/version/base/interval value cannot grant authority. Conversely, the outer hint is not an acceptance veto.

For a v3.5-style supported bad-wrapper record already present in the chain namespace, the same framed DATA and receipt objects are inventoried, and no corrected publication is needed merely to repair a hint (`exports:32–39`; `main:2588–2603`). This is a change to interpretation of existing bytes, not an assumption that every raw inclusion is valid.

Checked adversarial cases: current inner result with stale outer beta; wrong inner chain with correct-looking outer chain; stale outer expected base with a compatible signed suffix; correct outer base with incompatible signed ancestry; closed inner beta relabelled as current. The first and third can contribute the actual next compatible DATA. The others cannot pass the signed-domain/current-duty/path checks.

### 3. Closure extraction, ancestry, corruption, and duplicates

> “The decoder inventories all such objects, including DATA and receipts supplied as ancestry; the sender's declared output interval is not this inventory.” (`exports:14–30`)

The scan examines the actual object graph, recomputes identities/edges, rejects unresolved cycles as complete proofs, and selects only height `h(G)+1`. Its candidate order is fixed by signed pair, DATA hash, direct-receipt-root hash and canonical frame path (`exports:63–82`). A declared interval beginning above a gap cannot conceal a complete earlier DATA and its own commitment evidence carried as ancestry. After each commit, the next-height scan is recomputed.

Same-session coverage is sound: a correct signer that committed the root committed its ancestors, but a later-session FINAL does not cover an earlier session (`main:881–895`; `exports:41–61`). Thus a later-session closure can contribute an earlier next DATA only when it contains the earlier session's own direct/covering proof. It cannot replace that proof with the later receipt.

Independently framed malformed extras, an invalid alternative proof, and a bogus dependency outside the chosen closure do not poison a complete closure. A malformed required object invalidates that proof choice. Trying another proof before marking anything committed avoids an invalid first wrapper suppressing a later valid proof (`exports:56–61`). Different receipt signer sets/encodings do not create new DATA identities; already accepted heights are evidence only and cannot receive another count (`exports:47–54,73–75,104–120`). DRAIN remains supporting regional ancestry, never application DATA (`exports:99–102`).

The proof does not need a candidate-attempt counter in the public cursor: failed proof choices have no public mutation, and a resumed scan can recompute the same finite order. Successful DATA commits do change the cursor and current height.

### 4. Public dependency basis and cache independence

> “A verifier's newer private checkpoint does not change the basis.” (`source:14–24`)

The relevant basis is the authenticated source prefix before the record and the checkpoint authorized by that context. A dependency must be supplied inline or identify actual bytes by commitment and position in that basis. Earlier rejected raw records may provide evidence, but are not retroactively accepted. Current-record objects may be shared across closures.

The explicit distinction in `source:26–47` is essential and sufficient for the claimed cache-independent logical scan: a cache hit cannot turn a publicly incomplete closure into a complete one; a cache miss retrieves the specified public object and resumes. A bare hash/URL/future authority name is not an available public dependency. A candidate-local missing proof is skipped without blocking other closures. A missing namespace/control prefix or genuine canonical boundary dependency pauses the required operation instead.

Accordingly, two observers with different private caches cannot accept different DATA solely because one privately knows a proof that the other cannot derive from the record's common basis. Optional historical validation of a closed-beta object does not become a replay barrier. This verification concerns the specified common basis; it does not prove a particular implementation's retrieval performance or checkpoint authorization mechanism.

### 5. Per-DATA commit, terminal DATA, and canonical resumption

> “Durably expose the updated G and accounting before attempting a later DATA or successor operation.” (`exports:104–120`)

The atomic unit includes the appended DATA, producing-session accepted count, regional service credit, owed-set discharge, and window accounting. A terminal commit additionally cancels the old duty's window and records its pending REGULAR boundary. Thus an invalid later DATA cannot roll back an earlier accepted DATA or leave a counted-but-uncommitted/committed-but-uncounted result.

> “This transaction requires the completed session's incoming boundary and authority, not AUTH[r+1,0].” (`exports:122–139`)

The stored terminal boundary has the terminal anchor/provenance, source context, unchanged beta, target slot `r+1`, selection reference and lower bound. Only the later resolution operation obtains the target authority and runs `PrepareOwed`. This addresses the quota-one case directly: the first DATA can already be global while successor-authority retrieval remains unbounded. Whole-record preliminary authentication is expressly forbidden from reversing that dependency order (`exports:154–159`; `main:2134–2175`).

For the public cursor `(q,n,j,nu,stage)`, the record position and committed-DATA count determine the next height; `scan`, `resolve`, and `done` distinguish scanning, a canonical pending-boundary operation, and record completion (`exports:161–196`). The pending-boundary record retains the original resolution context. Duplicate source delivery resumes the persisted state, so it neither reruns a committed credit nor repeats the block prelude. The specification's abstract prelude/boundary steps and their persisted effects must be implemented as stated; the report is not a crash-recovery implementation test.

A missing genuine boundary authority pauses later replay after the accepted terminal DATA. Complete evidence proving no candidate instead completes that attempt and permits later source blocks to continue the historical search (`exports:133–152`). These are different cases. The former cannot substitute a later local receipt time for the original opening timestamp; the latter can legitimately open only at the first actual eligible historical position. Completed old records never regain append permission (`exports:188–196`).

### 6. Historical authority versus present permission

> “It authenticates what the boundary established; it does not require its version to remain open now and grants no entry permission.” (`main:1859–1867`)

`VerifyHandoffHistory` recursively checks the original fixed boundary, exact quota, terminal receipt, signed selection reference, or canonical Force transition (`main:1868–1918`). The separate `CanEnter` predicate requires the current open version, applicable boundary, selected local region, no previous entry/close/tombstone, and no authenticated Closed record (`main:1920–1940`). This correctly preserves a retained old anchor's evidence without reopening its old program.

The REGULAR search starts at `max(height(xi(T)),q_open+1)` and uses the first historical `NextLive` result under the target authority (`main:1801–1830`). Later OFF does not change an established historical boundary. Missing authority/history is a wait, not permission to skip a candidate. `ResolveBoundary` can use a containing terminal-publication header as the first selection position, but a successor DATA needing that header cannot pass the strict reference-before-containing-block check in that same block.

Zero-output Force retries retain the current numeric slot, positive-output switches advance it, and unassigned boundaries retain their stored target (`main:1767–1799`). Beta changes isolate the reused slot's messages, locks, timers and proofs. No old Quit or vote admits a new-beta generation. The at-most-once Enter argument in `main:2502–2535` follows from the fixed boundary, exact callback identity, serialized gate and durable entered flag.

### 7. Owed-set and historical snapshot timing

> “Later payload credits or round openings do not rewrite the saved pre-payload snapshots.” (`control:152–164`)

The mutable owed set O and saved O_E(q) have distinct roles. DATA credits mutate O once; they do not mutate O_E(q). During a continuing duty, an emptied O is not refilled merely for another DATA. A public boundary resolution obtains the designated authority, prunes ineligible debt, then refills only if empty (`control:101–141`). A speculative REGULAR handoff never performs those mutations.

Force selects from the saved historical O_E and eligibility snapshot, while REGULAR remains unrestricted historical `NextLive`. Therefore a post-prelude payload discharge or terminal-induced round refill cannot retroactively change the Force successor chosen from that prelude. Repeated evidence delivery cannot refill another round because it resumes the already persisted transition. Conditional AUTO cooldown checks another currently eligible owed candidate before adding the source cooldown; it neither clears an earlier cooldown nor overrides OFF (`control:166–175`).

### 8. Same-block Force order and cutoff

> “Apply only the first whose target matches the entry version.” (`control:207–249`)

The entry beta is captured before the prelude. Due independent STATUS/CLEAR run first in `(activation,sequence)` order. Only the first due matching MSIG has an effect and applies its attached configuration; other due MSIGs are consumed without attached effects. A matching MSIG skips AUTO. Thus a second instruction targeting the freshly created beta cannot cause a same-block cascade. This also handles a stale MSIG beside an independently effective STATUS.

AUTO settles due half-open windows before payload. Deadline-equal DATA cannot repair the old window; at most one AUTO closes the entry duty, even across many crossed deadlines (`control:45–90,229–249,273–284`). The new duty starts at the realized opening timestamp and is not settled again in that prelude. Equal-timestamp later blocks do not repeat already applied controls.

Both Forces retain the entire pre-payload G with original provenance. Acceptance independently rejects both closed-version DATA and DATA of a Force-created version in its triggering block (`exports:87–98`). The combination rules out a late old successor appending through a new outer hint, or a new session sneaking into the block whose prelude created it.

### 9. Regional retirement dependency checked against Part I

> “This does not assert a protecting QC for every late surviving timer.” (`retirement:238–247`)

The adaptation correctly avoids importing Part I's universal protecting-QC conclusion after retirement suppresses votes. I checked the Part I chain—same-generation chain-or-proof, protecting QC, recovery lock, ranked descent, direct/ancestor comparability (`baseline/sync_hotstuff_rr_part1_core.tex:733–831`)—against the modified gate.

The phase-independent close handler supplies the missing operational bridge. A member suppressing a matching vote because it is Closed must assemble and multicast complete closure from the proposal's historical snapshot context, including when it is Waiting, Completed, Terminated or has no worker (`retirement:43–120,171–194`). It cannot silently disappear from the proof's early vote window. Complete own-session receipts likewise multicast before Completion and continue through persistent owners (`retirement:24–41`).

For a direct timer voted at s that survives to s+2Delta, an early genuine Quit or Completion by s+Delta would arrive in time to cancel it. If `s+Delta<c`, no closure suppressed a required early vote, so the usual protecting QC reaches every possible later live recovery entrant before its lock snapshot. If `s+Delta>=c`, all correct entry gates close by `c+Delta<=s+2Delta`, whereas any entry into the next generation must occur strictly after `s+2Delta`; no correct later-generation signer can exploit a missing protecting QC (`retirement:249–326`). Same-generation conflicting histories still expose evidence before the chosen timer expiry (`retirement:328–338`).

The ranked descent is consequently justified in the early case; the late case excludes higher-generation signatures instead of asserting a nonexistent QC. A FINAL receipt traces to an earlier actual direct timer, since receipt-induced commits emit no FINAL. This makes the receipt-based commitment argument noncircular (`retirement:340–367`). DRAIN commitments use the same physical-position rank and timer rules; they do not export DRAIN or create new DATA quota.

The drain arithmetic is also consistent: close propagation by `c+Delta`, last possible timer expiry by `c+3Delta`, and generated FINAL delivery by `c+4Delta`. `d_v=c_v+4Delta` is conservative, and elapsed time alone is insufficient for FinishClose without durable responsibility transfer (`retirement:122–169,196–236`). This does not prove that every late timer produces a complete receipt; the explicit withholding example at `retirement:370–381` remains allowed.

### 10. Prefix induction and what remains conditional

> “Replaying to the same canonical subposition ... yields the same public replay result.” (`main:2448–2460`)

The induction's compared result excludes future control queues, private caches and speculative programs while including the exact cursor, pending boundary, historical snapshots and accepted DATA (`main:1402–1424`). Given the stated deterministic input grammar/common dependency basis, controls and boundaries select the same duty; the next-height scan selects the same valid DATA; each atomic DATA commit extends G; and every non-DATA transition preserves G. A paused observer therefore has a prefix of an observer that has advanced farther through the same record. The proof at `main:2462–2500` has the needed per-subtransition induction and does not depend on completing the whole EXPORT before exposing terminal DATA.

The causal record/reference argument at `liveness:205–214` is consistent with safety: a complete result includes a correct FINAL signer that validated a finalized reference at/after opening before signing. An immutable already-finalized block cannot later acquire that signature. Thus a supported record containing that result is after its required reference and after its Force-created opening. I found no same-block exception that preserves the honest-signing and finality assumptions.

The bounded public-response and downstream global/service propositions require more than this safety induction. Their stated scope uses supported decodable complete results (`exports:198–224`) and the complete SOURCESTEP/ReadyBundle handlers with already executable native state (`source:136–178`; `ready:107–146`; `liveness:149–247`). Frozen v3.1 described EXPORT fields/evidence and allowed complete validation bundles; it did not give a concrete byte grammar (`baseline/sync_hotstuff_rr_part2_rotation_v3_1.tex:1330–1357`). This review verifies the specified v3.6 interpreter, not backward binary compatibility of an unspecified deployed encoding. Opaque/unlocatable bytes, omitted required ancestry, arbitrary post-validation state import, or arbitrary offline backlog cannot be silently treated as covered by these bounds. The candidate explicitly withholds those claims.

No safety step failed in this review, so no safety-dependent claim is marked unverified because of a discovered Critical Error. The full objective nevertheless remains incomplete: `liveness:400–436` explicitly permits continuing accepted empty DATA while a queued request is repeatedly confined to a closed suffix. Global DATA growth and repeated region credits are therefore not a request-inclusion proof. Concrete source/native adapters and implementation timing are also outside this manual proof verdict.
