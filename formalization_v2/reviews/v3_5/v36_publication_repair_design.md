# Minimal v3.6 publication and terminal-commit repair

This is a proposed protocol repair after the completed v3.5 review, not a verification verdict. It uses `fresh_assumptions.md` and the frozen v3.5 snapshot. No running review drafts were consulted. The snapshot's entire Model and assumptions section is byte-equal to `snapshot/baseline/model_and_assumptions.tex`; this proposal leaves it unchanged.

The narrow repair is to derive appendable DATA from authenticated regional result contents, rather than authorize or veto that DATA through a sender's expected-base wrapper, and to commit a terminal DATA before resolving its successor. The bounded public-response proof should then use the first recorded complete result, including an adversary's earlier publication, rather than assume a bounded corrected EXPORT. This does not repair the separately admitted request-starvation problem.

## 1. The two dependencies that must be removed

The frozen publication premise says that a complete regional result is carried in an Espresso-admissible record within `T_pub`, and expressly lets a result already recorded in Espresso satisfy that premise. It assigns no bound to acceptance retries. In v3.5 an incorrect expected base rejects the entire EXPORT, even though its DATA, receipt, ancestry, and dependencies are genuine. Constancy of `G` therefore does not establish the needed response bound. See main lines 337–343, 405–420, 1412–1482, 2642–2658 and liveness lines 183–193.

Deleting expected-base equality alone is insufficient. A sender could put the same complete result under a wrong outer branch label, omit it from the declared output interval while placing it in ancestry, or append an unrelated invalid item that causes the all-or-nothing outer batch to fail. The promised delivery must have useful semantics at the granularity of its authenticated contents.

There is an independent terminal dependency. Acceptance currently includes all normal-boundary and owed-ledger transitions atomically; a new public boundary calls `PrepareOwed(r+1)`, whose first step waits for `AUTH_(r+1,0)`. That authority need not be boundedly available. With quota `Q=1`, the first positive DATA is terminal, so making the whole transaction wait would prevent even the first growth of `G` within `L`. A terminal result's own validation needs its own authority and incoming boundary, not the authority of a successor it has not yet produced. See main lines 1451–1475 and 2225–2236, control lines 111–137.

## 2. Separate transport metadata, evidence, and append permission

Keep the existing EXPORT transport and its chain namespace. Give it these explicit interpretation rules outside the frozen Model section:

1. The expected base, declared outer beta, suggested interval and batching order are transport hints. They are neither authority to append nor a veto on a complete authenticated regional result carried by the record. They can be retained for sender diagnostics.
2. The signed chain/protocol domain, `(beta,r)`, block hash, DATA ancestry, incoming boundary, historical authority and direct/covering FINAL receipt determine the result's identity and validity. An outer label cannot relabel a signed object. Every signature already binds the chain and protocol domain (main line 791).
3. Parse the complete record's typed objects, including objects supplied as supporting ancestry/evidence. A receipt root and its dependency closure are an independently checkable result. Identify covered DATA by their producing pair and block hash, not by sender, receipt signer set, wrapper ID or declared interval. A later session's receipt still cannot cover an earlier session's DATA.
4. Invalid or extraneous objects do not invalidate a distinct complete valid result. Missing or invalid material actually needed by that result does invalidate that result. A valid receipt does not make a malformed ancestor, unauthorized boundary or invalid native input acceptable.

This is an interpretation change, not a new certificate, publisher role, native execution primitive or service deadline. It deliberately changes outer-batch atomicity. It does not change regional commitment, DATA/DRAIN, quotas, REGULAR selection, immutable authorities or receipt coverage.

The transport grammar must be explicit about independently framed typed result/evidence objects. Completeness cannot depend on an untrusted outer interval enumeration. For compatibility, a well-framed v3.5 EXPORT must expose DATA and receipt objects in its body and dependency fields even when its expected base or other hints are wrong. Merely defining a new v3.6 wrapper and ignoring all already recorded older wrappers does **not** repair the frozen already-recorded case.

## 3. Common completeness and canonical extraction

For a record at canonical position `p`, retain the v3.5 public dependency basis: the authenticated source prefix strictly before `p`, together with the protocol-authorized checkpoint fixed by that source context. This basis includes authenticated objects in earlier raw records even if their outer EXPORT was rejected; global acceptance is not a prerequisite for using an object as evidence. Every result dependency must be supplied as bytes in the current record or resolvable by exact commitment and source identity in that common basis. Current-record bytes may be shared by several result closures.

Apply that predicate to each result closure, not to every unrelated item in the containing batch. Use a normalized index of actual parsed objects, rather than treating the sender's index as authoritative. A sender's unsupported claim that another object exists does not create it. A private cache can accelerate validation but cannot supply a missing public dependency. A publicly available object missing only from a verifier's cache is retrieved at the same canonical position.

An incomplete namespace or applicable control history still prevents processing the block. An independently incomplete result inside a complete namespace is skipped without fetching arbitrary future authorities or following arbitrary URLs. A false dependency on `AUTH_(r+100,0)` attached to an unrelated item cannot block a complete result for the current duty. The current result's genuinely necessary authority or incoming boundary is still mandatory.

Maintain outer namespace order. Inside each record, derive candidate next DATA from all its complete result closures using a canonical ordering, for example `(DATA height, producing pair, block hash, supporting-root hash)` with canonical object-position tie breaks. Recompute the next-height candidates after each accepted DATA. The outer sender's suggested order does not override DATA ancestry. Different encodings of the same authenticated object normalize to the same identity.

Candidates for different global heights are never alternatives to one another: only `h(G)+1` is considered. If several valid results support that height, the current fixed duty/boundary and regional safety should imply that their DATA identity is the same. State and prove that uniqueness obligation; canonical ordering must not be advertised as a replacement for regional safety. Try another independently complete proof when one purported proof of the same identity is invalid. Do not poison a hash-level dedup entry merely by observing an invalid first wrapper or signature set.

## 4. The acceptance transaction

After the unchanged block prelude, process records in namespace order. For each record, repeatedly perform the following transaction while it supplies a verifiable next DATA:

```text
DeriveNextDATA(S, record_at_p):
    extract result closures from actual typed objects, independently of wrapper hints
    find supported DATA B at height h(G)+1
    require signed chain/protocol domain and current append permission
    derive B's producing region and session from the fixed incoming boundary
    require exact ancestry compatibility with G, including provenance
    require own-session direct/covering commitment evidence
    require all admission, quota, parent, hash and native validity checks
    require every relevant Espresso reference to precede the containing block
    reject new-beta DATA in the same block that created beta by Force
    return B and the evidence actually needed for these checks, or none

CommitNextDATA(S, B):
    append B to G, preserving its signed content and producing identity
    increment its producing session and regional accepted-service counts once
    remove its producing region from the current owed set if present
    if B is terminal:
        cancel the completed duty's window and retain its completed counts
        install a pending REGULAR boundary anchored at B, target slot r+1
        commit this state before attempting successor authority/selection
    otherwise:
        keep the same duty and window, with the new accepted count
    durably expose the resulting accepted DATA prefix
```

Compatibility means that the result's authenticated path contains the exact current anchor/tip, with matching hashes, positions and original producing identities, and that its next DATA is the unique continuation authorized by the current boundary. A physically present ancestor is not accepted merely because a descendant's receipt exists: apply all required session-specific commitment and application checks. DRAIN remains supporting regional evidence and never becomes application DATA.

The interpreter derives the actual base from `G`. It does not reinterpret the sender's false claimed base as true. An overlapping record can contribute a still-unaccepted compatible suffix; accepted ancestors are evidence and receive no second credit. A conflicting path or closed-version suffix cannot contribute new DATA.

Atomicity is now per accepted DATA transaction, including its count, owed discharge, and pending-boundary creation. An implementation may batch a completely validated sequence as an optimization only if the outcome and observable committed prefixes are identical to these transactions. A later invalid item or unresolved successor must not roll back an earlier transaction or postpone its finality. The sender cannot choose the all-or-nothing group that controls progress.

There is no need for an unbounded global reconsideration queue. Each source record is interpreted at its canonical position. Rejected candidate outputs are not retroactively accepted after later public transitions. Public evidence can remain cached and be referenced by a later record under the common-basis rule; that later reference has its own canonical acceptance position. For the bounded-response lemma, the complete result for the current prepared retry already contains the next DATA and its ancestry, so no such later record is required.

## 5. Terminal DATA and successor resolution are different transactions

The pending REGULAR boundary installed by terminal acceptance contains everything already fixed without the successor authority: the terminal anchor and receipt, source context, target slot `r+1`, unchanged beta, signed selection reference and historical selection lower bound. It does not fabricate a successor, a zero-weight result, an empty owed set, or a new window.

Only the subsequent `ResolveBoundary` operation obtains `AUTH_(r+1,0)`, runs `PrepareOwed(r+1)`, authenticates the historical selection, and assigns the public successor duty when possible. Missing authority leaves this operation pending. It does not undo the terminal append or restore the predecessor's window. Once evidence is available, selection and owed updates resume at their original canonical transition, exactly once; retrieval time does not become an opening timestamp.

Protocol 6's input-authentication stage and the source handler must use the same dependency boundary: they may require the controls and authorities needed for the prelude and the DATA transaction being verified, but must not eagerly require every authority that a later operation in the record might use. Otherwise an early whole-record completeness wait would recreate the terminal dependency before the new commit rule is reached.

The owed set immediately after terminal append is the old set minus the producing region, with no speculative refill. This is a legitimate intermediate public state. Debt pruning and any next-round initialization under the target authority occur only in `ResolveBoundary`. A pending boundary is different from a proved absence of an eligible successor, but neither runs a DATA window before a duty is assigned.

Keep the current historical REGULAR rule and public window-opening convention: an already selectable successor opens publicly at the terminal acceptance block's timestamp; a boundary with no candidate opens at its first eligible selection block. A delayed proof resumes that same computation and never starts a fresh grace period. An earlier speculative selected handoff is checked and reused, not recomputed from local time or current membership.

The external interpreter must be able to persist and expose the DATA-commit substate even if later selection prevents completion of the enclosing record or block. Thus its durable cursor/checkpoint needs a canonical subposition, such as `(q, namespace-record-index, accepted-DATA-index, pending-boundary-stage)`. Global finality refers to a committed accepted-DATA subtransition in an authenticated finalized source prefix, not to successful completion of every later operation in that block. A full-block-only exposure API would reintroduce the same `Q=1` delay.

A conservative implementation may pause subsequent canonical replay at the unresolved boundary. This introduces no new deadline for successor assignment; the already committed `G` is permanent and already observable. If implementation instead continues processing independent controls, it must separately specify and prove the equivalence of that continuation. The minimal repair does not need such out-of-order continuation.

When successor evidence is already present, normal multi-session batches remain possible: commit terminal A, resolve its ordinary boundary, then consider B's next DATA. If B's opening reference is the containing block, B still cannot append in that block. No DATA/DRAIN or REGULAR selection rule changes.

## 6. Force cutoff and prepublication cases

The entire original prelude still runs before any extraction can append DATA. It applies independent controls, then a matching MSIG or AUTO with the existing priority, records closure, retains exactly pre-payload `G`, and forbids the new beta from exporting in that same block. Result normalization is never a reason to inspect or credit the payload before this prelude.

| Case | Required result |
| --- | --- |
| Complete current result, wrong or stale expected-base hint | Derive the compatible next DATA from its signed contents and accept at this record position. |
| Wrong outer beta/domain/interval hint, correct signed result for this chain | Ignore the hint as authority; authenticate the inner domain and actual session. A result from another signed domain remains invalid. |
| Valid result accompanied by unrelated bad DATA or a bogus dependency | Validate its own complete closure and commit it independently. |
| Result overlaps `G` | Deduplicate accepted ancestors and consider only the compatible next DATA. |
| Future-session result whose complete closure supplies the current next DATA | That DATA can be derived from its own valid direct/covering evidence and accepted first; successor material is considered only afterward. |
| Record genuinely lacks the current next DATA or its necessary proof | No gap skipping and no bound on a later acceptance retry is inferred. Such a record does not carry the complete next result needed by the lemma. |
| Old-beta result in a block whose prelude closes that beta | No new old-beta DATA is accepted. The closure is the response alternative. |
| New-beta result in its Force-triggering block | Rejected by the unchanged same-block restriction. |
| Result already globally accepted before the correct holder offers it | The growth alternative already occurred; no second acceptance or credit. |
| Earlier result becomes relevant only after closure or an unrelated later boundary | No retroactive old-record acceptance or reopening; historical evidence alone grants no append permission. |

A malicious holder may possess and publish the genuine complete result before a correct holder obtains the assembled receipt. Under the repaired rule this is useful publication: if it was appendable then, it advanced `G` then. If `G` already advanced or a Force intervened, the public-response alternative already holds. A bad wrapper cannot reserve a result identity or mark it permanently consumed.

For the prepared retry's first positive result, causal validity matters. Its correct FINAL signer validated an incoming boundary and an authenticated finalized Espresso reference at or after the retry opening. A record genuinely containing that signed complete result cannot have been finalized before those dependencies existed. Hence its containing source block is after the reference and after the Force opening block. This needs to be proved from authenticated finalized-reference validation and immutability; it must not be replaced by an assumption that all adversarial publishers choose good wrapper metadata. Regional validity alone must not be silently treated as general append permission for unrelated historical results.

## 7. Why this can use the frozen delivery bound

Let a prepared retry open at real time `a`, and let a correct holder offer its complete first positive result at `s`, within the existing regional bound. Assume neither growth of `G` nor an effective closure has occurred by the time that result's canonical record is processed.

If it was not previously recorded, the frozen service supplies its record within `T_pub`, and Espresso supplies finalized, retrievable complete bytes within `T_E`. If it was already recorded by `s`, no new send is required: apply the same extraction/acceptance semantics to that earlier record. Its complete source input is available within the existing visibility deadline for that record/block, at latest `s+T_E` when “already recorded” means already finalized. The earlier publication can only shorten this arm of the argument.

At that record position, unchanged `G=G*` supplies the same anchor, the retry is still current, the result's signed incoming boundary fixes the same session, and the complete result supplies the first next DATA and validation closure. Wrapper metadata and unrelated batch items cannot veto it. The current retry authority is among the prepared dependencies. If that first DATA is terminal, the commit is independent of successor authority. Thus at least one DATA is appended, contrary to the assumed absence of growth. A Force in its own containing block instead supplies the closure alternative before payload processing.

No line in this argument asks the Publish Layer to submit a corrected expected-base wrapper within `T_pub`. The existing numerical `L = max_i {2 T_E + 2 Delta_i + R_i + T_pub}` is a candidate bound only after establishing the existing complete-handler timing obligation for the new normalization and per-DATA commit step. This design does not prove a new constant bound for arbitrary native replay, an arbitrarily old backlog, or successor-authority acquisition. If the normalization adds processing outside the frozen complete-message contract, that delay must be exposed; it cannot be renamed into `Delta_i`.

The global-progress and repeated-regional-service proofs can be revisited only after this replacement lemma is proved. The request-inclusion theorem remains unresolved because one accepted empty DATA can still discharge regional debt.

## 8. Exact boundary of the frozen-contract claim

The repair covers the review's witness: the genuine complete DATA/result, including its ancestry and source evidence, is in the chain's authenticated Espresso record, but its outer EXPORT metadata prevents acceptance. It also covers adversarial early publication of that same complete result. It does not obtain the repair by declaring that a wrong expected base means the result was never recorded.

The phrases “record carrying that result,” “complete regional result,” and the chain's “complete relevant namespace” must retain their original meaning. The proposal must not silently strengthen them to mean “an exact-base accepted export” or add successor authority to the definition of a complete terminal result. Equally, a bare hash, absent payload or genuinely missing required ancestry was already excluded by the frozen complete-result definition (main lines 337–343).

There is a real interpretive limit that should be stated in the v3.6 proof. If the frozen service is read as discharging its obligation when a result exists only in an undiscoverable unrelated namespace, an undecodable opaque blob, or a record that drops required ancestry and preserves only DATA plus a receipt, this local repair cannot prove the bound. The frozen model then does not give the receiver bounded access to the promised complete result at all. One must not hide that problem by announcing a stricter new transport contract. Under that broader reading an additional assumption would be needed, or the theorem must remain unproved. Under the ordinary existing reading—a complete result carried in a decodable chain record, with complete namespace availability—the new parsing/acceptance rule is an algorithmic repair, with no extra publication promise.

Before claiming a repair, instantiate the extraction function on the exact earlier v3.5-style bad-wrapper witness. Its result bytes and dependency closure must be accepted under v3.6 without a new publication. Test this as a semantic compatibility requirement, not only on freshly generated v3.6 packets.

## 9. Proof obligations and edit map

1. **Extraction completeness.** Every independently complete result physically carried by a supported record yields the same authenticated candidate objects regardless of outer hints, private caches, sender indexes, or unrelated invalid items. Demonstrate this for the already-recorded witness, including a result placed in supporting ancestry rather than the declared DATA interval.
2. **Common dependency closure.** The source prefix/checkpoint and inline-object rule determine one completeness predicate. Missing public bytes pause at the same point; bogus nonpublic references do not block other complete result closures. Previously rejected raw records can supply authenticated evidence without gaining retroactive append permission.
3. **Unique compatible next DATA.** From the fixed incoming boundary and regional safety, at most one DATA identity can pass all next-height checks. Receipt coverage never crosses sessions. Authenticated inner domain and session fields remain authoritative.
4. **Per-DATA prefix safety.** Replacing outer-record atomicity with per-DATA transactions preserves append-only `G`, all count/owed invariants, quota limits, native validity, and immutable provenance. A malformed suffix cannot revoke a committed prefix. Duplicate wrappers, alternate receipts and repeated referenced objects never double count.
5. **Terminal noninterference.** Validate and commit `Q=1` DATA without `AUTH_(r+1,0)`. Persist/expose its accepted substate before the authority wait. Later `PrepareOwed` executes once at the original canonical boundary, with no clock reset or speculative round refill.
6. **Canonical cursor and certifier agreement.** Define partial block/record checkpoints containing the pending boundary and precise subposition. Two correct observers at the same subposition agree; later unresolved selection does not make earlier accepted DATA provisional. Certifier previous/next checkpoints bind this full state.
7. **Strict cutoff.** Induct over prelude then ordered DATA subtransitions. No payload in a closing block repairs the expired window; no new-beta DATA appears in the triggering block; no old-beta candidate is revived by caching, normalization, or late evidence.
8. **First-result acceptance dichotomy.** For the current prepared retry, prove causal precedence of its boundary/reference, complete next-DATA ancestry, and the “already recorded means already accepted or intervening growth/closure” implication. Do not use a corrected-retry deadline.
9. **Timing accounting.** Establish that extraction, validation and durable per-DATA exposure fit the existing complete-source/message handler scope and declared input-size/load limits. Do not turn an arbitrary old history catch-up or expensive native operation into a bounded atomic instruction by definition.
10. **Dependent theorem scope.** Re-run the response, global-growth and regional-service proofs with the repaired lemma. Keep request inclusion, arbitrary dead selected regions, and unbounded initial restoration outside the established conclusion.

The implementation-facing edits belong in the main `sec:exports`, Protocol 6, global-finality/checkpoint definitions, and publication implementation discussion; the source-step fragment's record-completeness gate; the control fragment's public-boundary/`PrepareOwed` call sites; and the public-response proof. The Model section, regional DATA/DRAIN rules, REGULAR historical selector and strict Force cutoff need no change.

Validation performed for this design note: source-level inspection of the frozen definitions and the completed assumption review; byte comparison of the Model section against the frozen baseline. No source implementation, model checking, compilation or independent proof verification was performed.
