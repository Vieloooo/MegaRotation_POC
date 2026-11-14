# Summary

**Final Verdict:** The reviewed safety/validity argument and the restricted first-public-carrier-to-pending implication are valid under the frozen assumptions and the explicitly stated abstract interface contracts; this independent review found no in-model **Critical Error** or **Justification Gap** in those claims.

**List of Findings:** None within that scope. In particular, no concrete identifier-poisoning, stale-context, duplicate-execution, closure-reopening, or public-acceptance counterexample survives the specified validation and lifecycle rules.

**Scope and limits.** This is a manual adversarial verification of the frozen English snapshot, concentrating on the new public request import and the regional/FINAL/global/retirement safety chain. Full request liveness remains **OPEN**: the new proposition neither creates the first request-bearing public result nor proves awareness at sufficiently many future accepted report events. No deployed codec, native adapter benchmark, executable test, or model checker was supplied or run. The verdict does not turn the conditional complete-handler contracts into an unconditional implementation timing guarantee.

**Snapshot integrity.** Before review, all **24/24** manifest file hashes matched. The main file SHA-256 is `63dd6770de6aaed0f442c53c6fcd3685fb5a37b478140d67970ee2c4719058cb`. The main document's entire Model and assumptions section is literally identical to `snapshot/baseline/model_and_assumptions.tex` (16,456 characters). The post-review check recorded below again matched **24/24** hashes. No source was edited. Inputs were restricted to `review_scope.md`, `manifest.json`, `snapshot/`, and the explicitly requested proof-verifier skill; no author notes or other reviews were consulted.

# Detailed Verification Log

All source locations below are relative to `snapshot/`.

## 1. Frozen assumptions and dependency map

> “The bound $F_i$ counts every key of the snapshot that is compromised while its certificates are still accepted.”

Location: `baseline/model_and_assumptions.tex:135–153`.

Every certificate of distinct weight greater than $F_i$ has an actual correct signer under the fixed snapshot and accepted-lifetime compromise bound. This supports provenance, not quorum intersection by itself. I did not count attacks requiring forged signatures, hash collisions, excessive Byzantine weight, retrospective reweighting, incomplete accepted control history, or violation of the specified synchronous complete-message contract as in-model attacks.

> “Regional receipt validity and the application-validity checks below are assumed sound under the stated regional model.”

Location: `baseline/model_and_assumptions.tex:285–291`.

The application interface, request canonicalization/identity semantics, native-state correspondence, and complete dependency validation are abstract interfaces. The current argument must call them on the correct objects and state; it does not derive their implementation. The dependency chain checked here is: retained actual bytes → unvalidated candidate → successful ordinary request validation → authenticated pending entry → exact-parent report/batch validation → correct regional Vote/timer → actual FINAL provenance → public kernel plus independent append permission. Closure preserves or transfers each relevant owner without creating another authority path.

## 2. Source retention and typed extraction

> “Every acquired complete relevant namespace, including rejected or malformed records” and “These are descriptors, not an in-handler scan or request check.”

Location: `sections/rotation_v3_9_source_step.tex:12–17,33–55`; `sections/rotation_v3_9_public_requests.tex:23–55`.

The source owner retains the full acquired namespace and records the import obligation before advancing its availability cursor or releasing the input. P6 rejection and closure therefore cannot discard an otherwise available request carrier. Initial retained roots get a fixed-cut job; subsequently acquired exact reference targets get their own jobs. Thus a target unavailable at an earlier dictionary cut is not incorrectly presumed present, nor is its eventual actual acquisition lost behind the completed earlier job.

Extraction follows actual typed block/body fields and batch positions. A request hash, arbitrary witness string, sender's object index, DRAIN body, or malformed unstructured region of bytes cannot be promoted into an invented DATA entry. Independently represented typed children remain inspectable when an enclosing proof fails. Wrong outer chain/base/version hints do not suppress extraction, but they also confer no validity on extracted requests. Visited-object tracking and fixed finite input exclude unbounded cyclic traversal. These are logical-format conclusions; the carrier source expressly supplies no deployed binary decoder theorem (`sections/rotation_v3_8_carrier.tex:5–29,60–106`).

## 3. Candidate deduplication and authentication

> “Before successful request validation, deduplicate only identical bytes” and “A claimed client identifier creates no duplicate marker.”

Location: `sections/rotation_v3_9_public_requests.tex:57–64,82–94`.

Consider an attacker first placing bytes that assert the identifier of a later valid request. Unless the bytes are identical, the later request remains a distinct candidate. If they are identical, the same candidate bytes already exist. Failed decoding/authentication creates no authenticated identity reservation. Promotion requires chain/domain, client authentication, canonicalization, and application admission; only the resulting actual logical identity may merge retransmissions. Thus invalid asserted-ID poisoning is excluded without relying on the containing DATA's validity.

The same rule blocks cross-domain laundering: a typed DATA entry from a rejected or wrong-domain container is only raw input to the ordinary request checks. It becomes neither a valid relay nor an executable request by virtue of its public occurrence. Different wrappers of the same authenticated request are merged only after validation under the original canonical representation and logical-identity interface.

## 4. Fixed native context and stale-parent attacks

> “A pointer to a subsequently mutated engine state is not such a context” and “Changing the active parent does not retarget or cancel an old job.”

Location: `sections/rotation_v3_9_public_requests.tex:66–94`; main source `1601–1603,1632–1645`.

A checking job is created only after actual preparation of an exact parent and its retained read context. Snapshot construction must finish before the notification. It is neither inferred to be instantaneous nor performed implicitly inside the import loop. A later engine mutation, Force, change of $G$, or different current parent cannot change the state against which that job authenticates/admission-checks its bytes.

A success at an old parent may enter the ordinary pending store, but explicitly does not establish validity at a new parent. A failed check is local to that byte/parent pair and does not permanently reject the asserted identifier. The job acquires no absent native state or authority. A stale callback can supply useful retained material but cannot replace the session anchor or authorize Enter. This prevents the import worker from becoming an alternative Ready or native-state installation path.

## 5. Pending state, ancestry exclusion, and stateful batches

> “At report emission and FIFO filling, use the completely validated actual parent, its native state and complete ancestry” and “Earlier entries in a batch still affect admission of the remainder.”

Location: `sections/rotation_v3_9_public_requests.tex:119–132`; `sections/rotation_v3_9_requests.tex:5–20,60–79`.

The successful import does not bypass the exact eligible projection $E_v(P)$. That projection excludes identities in inherited global history and in speculative ancestors, and it is not reused for another hash or native state. Consequently an old successful check cannot justify a request already executed on the current parent ancestry or a request invalidated by intervening state changes. FIFO filling also checks batch-local duplicates and state changes induced by earlier entries. The distinction between an authenticated pending request and an eligible request at this parent is preserved.

Pending requests remain until their identity enters $G$, so discarding an unaccepted speculative suffix does not erase them. If the identity is already globally accepted when a job finishes, the outcome is recorded without adding an active pending entry. Public replay and pending removal cannot create a second global occurrence because subsequent batch validation still excludes complete ancestry. Source-evidence retention is separate from active pending membership.

## 6. FIFO work, persistence, and the new proposition

> “New jobs go to the tail and cannot overtake a fixed job” and “session changes and repeated new arrivals cannot suspend it indefinitely.”

Location: `sections/rotation_v3_9_public_requests.tex:96–117,134–165`.

For a fixed acquired source job, the stated execution discipline supplies finitely many predecessor jobs, finite input, saved local progress, and eventual completion of finite local work. It therefore reaches each actual batch position. An unresolved reference does not itself supply a payload; the proposition separately assumes acquisition of the actual bytes, whose target-acquisition hook supplies the later job. Exact-byte deduplication can only redirect to bytes already retained.

The prepared-parent event/candidate-arrival hook similarly records a finite checking job with all inputs held. Its successful check eventually promotes the request unless already globally accepted. Neither future arrivals nor a moving active parent can cancel that fixed obligation. Journals, cursors, read contexts, outcomes, and pending enqueue obligations survive worker exit, phase transitions, checkpoint compaction, and $G$ advancement. `FinishClose` explicitly transfers them to a persistent owner (`sections/rotation_v3_9_retirement.tex:130–158`).

This verifies the stated conditional proposition. Eventual local work is an explicit protocol duty, not a derived wall-clock bound or an archive/network service. It does not prove that any suitable parent will ever be prepared, that the first carrier forms, or that import finishes before any particular report. No finite storage or throughput bound follows.

## 7. REPORT interaction and regional evidence

> “Later arrivals do not amend it” and “Reports create no QC rank, lock, timer, commitment, entry permission or new Quit proof.”

Location: `sections/rotation_v3_9_requests.tex:81–108,110–159`.

Reports are persisted once for the authenticated complete context, including exact parent and opening. A late import therefore cannot rewrite an existing EMPTY or nonempty report. It is usable in later report contexts and ordinary batching. Correct leaders check report signatures, fixed membership, complete context, body/key consistency, admission, fit, and ancestry exclusion. Regional validators independently check the selected body's authenticated minimum and full batch validity. A fictitious selected key cannot become executable without its matching valid body.

An invalid report wrapper does not suppress the original ANCHOR vote or admissible equivocation evidence. Sidecar variants do not change proposal identity or restart a timer. Own-session Vote, recovery histories, receipt processing, and Completion retain the required sidecars. The correct reporter's local-minimum assertion is tied to its historical report event, not to the validator's later local queue.

## 8. Regional commitment, FINAL, and public R0 scope

> “A commitment triggered by a receipt, including Completion, emits no new FINAL.”

Location: `sections/rotation_v3_8_receipt_kernel.tex:33–90`; main source `1133–1197`.

The raw FINAL threshold selects an actual correct FINAL signer. That action traces to a timer started only after full own-session validation, including the historical REPORT predicate. Receipt-triggered commitments do not manufacture a new first correct signature. Inherited-anchor kernel dependencies precede the new Vote event, so causal induction from the trusted checkpoint is well founded. Hash binding identifies the actual DATA and supplied hash-linked ancestors; same-session coverage does not become cross-session coverage.

> “Erasure removes only the new separately framed sidecars” and “It is not merely the public kernel.”

Location: `sections/rotation_v3_9_requests.tex:161–203`; `sections/rotation_v3_8_exports.tex:201–238`.

The Publish Layer still receives the original complete $R_0$, with original DATA, receipt, ancestry, boundary, authority, native-validation and public-source objects. The public checker may use its sufficient kernel without making a publisher's alternate recovery/native witness mandatory, but publication service is not silently applied to a kernel-only object or bare request. Regional $R_+$ ownership preserves REPORT sidecars; public erasure does not authorize their last-copy deletion. A kernel proves existence of a genuine historical witness, not validity of a publisher-selected alternative or an arbitrary unbound native-output claim.

## 9. Closure and the regional safety chain

> “Only Quit and Completion cancel a timer” and “a force instruction never cancels a timer.”

Location: main source `1066–1078,1158–1164`; `sections/rotation_v3_9_retirement.tex:243–373`.

The retirement proof does not incorrectly reuse the baseline universal protecting-QC lemma for a late vote. For a surviving timer voted at $s$, an early correct Quit or Completion would reach and cancel it by $s+2\Delta$. A closure-suppressed peer vote produces a phase-independent complete-close event by $s+\Delta$. If $s+\Delta<c$, all correct votes are available and the protecting QC precedes later recovery locks. If $s+\Delta\ge c$, complete-close propagation shuts every correct entry gate by $c+\Delta$, while any later recovery entry must be strictly after $s+2\Delta\ge c+\Delta$. Thus the late case cannot exploit a missing protecting QC in a later generation.

Same-generation incompatible signed histories still expose the original admissible equivocation before a surviving timer expires. Across generations the ranked-justification descent has its required protecting-QC premise in the early case. Receipt commits trace back to those direct timer origins. The $c_v+4\Delta$ drain covers existing honest timers and FINAL delivery, while ownership transfer—not merely elapsed grace—preserves later receipt/publication/import duties. Pending request transport neither issues a fresh INPUT_REPORT nor reopens the old BFT worker.

## 10. Global acceptance, immutable slots, and replay

> “Force retains all of $G$ before payload” and “Closed versions are never reopened.”

Location: main source `2488–2542`; `sections/rotation_v3_8_exports.tex:78–123,164–199`; `sections/rotation_v3_6_control.tex:207–284`.

Actual kernel validity remains separate from signed-domain, current open version, authenticated incoming duty, exact extension of $G$, quota/position, strict source precedence, and the post-Force containing-block cutoff. Import does not modify any of these predicates. Preludes process independent controls, then a matching MSIG against the captured entry version, then AUTO only if appropriate. Force retains the exact pre-payload prefix and cannot authorize new-version DATA in the triggering block.

Per-DATA transactions durably expose accepted prefixes before successor-authority waits. Completed old records receive no retroactive append permission; later exact-byte acquisition may import a request without accepting its old DATA. Duplicate source delivery resumes the saved subposition and cannot rerun counts or a prelude. A zero-output retry retains the authority slot, positive accepted output advances it, and an unassigned boundary retains its stored target slot through repeated Force (main source `1817–1849`). None of the request-import events changes those rules.

## 11. Timing and the deliberately unresolved request obligation

> “Their original processing bounds are not bounds on this deferred work.”

Location: `sections/rotation_v3_9_public_requests.tex:96–107`; `sections/rotation_v3_9_source_step.tex:149–164,243–257`; `sections/rotation_v3_9_requests.tex:207–245`.

SOURCESTEP and native-preparation handlers enqueue descriptors; neither scans the candidate journal nor waits for a snapshot/import watermark. Report formation uses the already completed exact-parent projection. Under the expressly qualified complete-handler contract, the added worker therefore introduces no new dependency into P6, Ready, ANCHOR, or the stated report deadlines. The text does not establish that an arbitrary concrete CPU, storage, or native runtime meets those bounds; it expressly withholds that inference.

> “This does not yet derive full request inclusion from the frozen changing-membership ingress prose.”

Location: `sections/rotation_v3_9_requests.tex:247–305`; `sections/rotation_v3_9_public_requests.tex:167–175`.

The accepted-awareness theorem has a valid finite-descent argument: while $x$ remains absent, each counted accepted first DATA spends a previously globally unaccepted identity whose canonical key is at most that of $x$. There are finitely many such canonical byte strings; exact ancestry exclusion prevents spending the same identity twice. Import can supply pending entries useful to this theorem, but gives neither the required infinitely many aware accepted report witnesses nor first-carrier formation. Repeated future rosters may report before their later deliveries/imports. Full request liveness therefore remains **OPEN**, as stated; no ownership/hydration proposal outside this frozen protocol is assumed enabled.

## Post-review integrity check

Recomputed SHA-256 for every path in `manifest.json` after writing this review: **24 files checked, 0 mismatches**. Main source hash remained `63dd6770de6aaed0f442c53c6fcd3685fb5a37b478140d67970ee2c4719058cb`.
