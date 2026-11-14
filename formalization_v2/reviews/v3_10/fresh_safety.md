# Summary

**Final Verdict:** The reviewed v3.10 safety arguments are valid under their stated original regional, source-verification, cryptographic, native-validation and complete-message/Ready-handler premises; this independent review found no in-model `Critical Error` or unresolved `Justification Gap` in the safety chain examined.

**List of Findings:** None.

This is a conditional safety verdict, not completion of the full end-to-end liveness task. In particular, it does not certify a deployed native adapter, establish a bound for work outside the original complete-message interface, prove that physical machines may leave while retaining pending duties, or independently certify every arrow of the global/service/request-liveness theorem. No model checker, implementation execution, benchmark or machine-checked proof was supplied or run.

The review used only `review_scope.md`, `manifest.json`, the frozen `snapshot/`, and the requested proof-verifier skill. No live source, author notes, earlier review or peer conclusion was read. All 26 manifest entries matched before review and again after the report was written. The frozen main-source SHA-256 is:

`1555702839d5d117a95428953d91efcd28154c96a719119a2d178086d7e45bab`

The complete Model and assumptions section is byte-for-byte identical both to `snapshot/baseline/model_and_assumptions.tex` and to that section in the frozen v3.1 source: 16,456 bytes, SHA-256 `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387`.

The human clarification “覆盖同一地区的新旧正确成员” was applied exactly: the original correct-to-correct within-physical-region service covers old and new AUTH members whose duties continue. It supplies neither cross-region synchrony nor correctness of a failed owner, and it does not turn finite native work into bounded work.

# Detailed Verification Log

All locations below are relative to `snapshot/`. Quoted fragments identify the checked rule; the accompanying analysis records the proof obligation and its scope.

## 1. Dependency map and assumption filter

**Location:** Main source, Model and assumptions, especially stake/faults, `ass:network`, and Verification interfaces: “Every regional certificate needs … total weight greater than (F_i)” and “The bound (F_i) counts every key … compromised while its certificates are still accepted.”

**Check:** (B_i\le F_i<W_i/2) gives both implications needed below: every threshold certificate has a correct signer, and correct-only weight satisfies (W_i-B_i\ge W_i-F_i>F_i). This does **not** give honest intersection between two threshold sets. The new proofs use signer provenance and actual dissemination, not such intersection. The accepted-lifetime compromise condition covers reliance on retained old signatures; it cannot be replaced by a snapshot-time corruption bound.

The safety dependency chain is: fixed authentic context and genuine complete regional validation → actual timer-origin FINAL → sound public kernel → independent public branch/ancestry/reference checks → deterministic per-DATA acceptance. Regional agreement additionally uses the Part I timer/lock argument, with the explicit retirement replacement for its universal protecting-QC claim. Entry isolation and persistent closure prevent callbacks or preparation from bypassing that chain.

The original sound application/receipt interfaces, authenticated complete controls, immutable L1 authorities, compatible Espresso history, hash binding and signature unforgeability remain premises. Attacks that omit an applicable finalized control, exceed the original faulty weight, forge a receipt, or stop the service of a still-correct obligated endpoint are not executions of those premises.

## 2. Persistent owners, authenticated contexts and transfer limits

**Location:** `sections/rotation_v3_10_ownership.tex`, Directory and communication scope, Persistent state and finite cuts, and Complete context before the first query: “This is finite known-key retrieval” and “Every referenced validation dependency is actual supplied content.”

**Check:** The directory is the authenticated finite union through an actually admitted numeric slot. Genesis starts at zero; regular and positive-output transitions advance by one; zero-output retries preserve the slot. Thus the construction uses known existing predecessor AUTH keys, not an oracle for absent or arbitrary future authorities. A checkpoint must retain their authentication dependencies. Persistent identity and the authenticated sender prevent one participant from creating another correct target member's cut.

The capsule transfer actually sends bytes and freezes its length, digest and chunk count. The context-validation trigger follows the final send by (2d); the first query follows the last trigger send by another (2d). Under the stated complete-handler scope, required correct owners have the context before the query even at equal timestamps. The proof does not charge the whole arbitrary capsule transfer or arbitrary native work to one (d).

Descriptor, metadata and fragment deadlines freeze finite budgets. Byzantine silence, changed roots, excessive finite lengths and equivocation cannot create new slots or restart an existing job. Such behavior can increase the actual finite preparation cost; it is not shown to have a practical resource bound. Wrong-sender/wrong-context traffic cannot fail a correct owner's active record. A correct tombstone is safe only for already accepted identities; an adversarial tombstone cannot erase the correct owner's independently retained request.

**Location:** Same file, Actual all-member dissemination and HYDRATED: “Only at or after (b_v+2d) may (v) create its signed share.”

**Check:** A correct share has a causal all-target byte-delivery predecessor. The last-fragment handler installs a complete immutable raw handle before the share can exist; it need not perform canonicalization. HYDRATED signatures bind chain, region, numeric slot, initial AUTH hash and the canonical AUTH-prefix commitment. Beta independence permits exactly the stated reuse while preventing reuse at another authority or target slot. Threshold provenance therefore establishes hydration for **every** valid certificate, including one assembled later from old shares. It is not a statement that the signature alone encodes a common pool.

Release pacing includes equality in the exceptional set. A pre-request correct query can occur only at one of finitely many numeric slots for that request. The accepted-count rule prevents positive-output beta retries from repeatedly spending the same numeric slot. These facts support the opening-coverage premise; they are not a claim of global selection fairness.

## 3. Executable preparation, immutable selection and OC coverage

**Location:** `sections/rotation_v3_10_input_selection.tex`, State distribution before any Ready: “Every correct native preparer sends without waiting for an offer, certificate, Ready or acknowledgement.”

**Check:** There is no certificate/state bootstrap cycle. A genuine exact-anchor executable state and its original verification dependencies are distributed before OC formation. They confer no entry, lock, timer or ANCHOR permission. New material can be cached without changing an installed session anchor.

**Location:** Same file, A fixed finite selection job and Opening certificate: “Later arrivals never enlarge or restart it”; “It does not read a private queue”; “A locally attached certificate is not exclusive.”

**Check:** The preparation key binds the original producing pair and exact anchor/hash/positions, authority, region, slot and native domain. Excluding beta is safe only when all these fields are equal; the source enforces that equality. The job freezes raw and canonical handles after checking a complete hydration certificate, then actually checks authentication and admission at the exact opening state before offering. Invalid raw claimed identities do not reserve a canonical duplicate identity.

Every valid OC has a correct offer signer. A Byzantine contributor can advertise a false minimum only by supplying an eligible authenticated selected body with that minimum; otherwise `CheckOC` rejects the certificate. Nonselected body availability is unnecessary for the minimum inequality used by coverage. A valid certificate need not contain a particular receiver's offer or match its local certificate. This avoids a private-pool or local-OC veto.

The hydration-to-offer argument uses actual delivery to every correct target before any correct offer cut. It remains valid when the correct HYDRATED signer and correct OC signer are disjoint. No honest-intersection inference was found.

## 4. First-own-session DATA, REPORT and original evidence handling

**Location:** `sections/rotation_v3_10_requests.tex`, The first-DATA report: “A speculative first DATA discharges this obligation only on its own descendants.”

**Check:** The obligation is defined by the complete own-session ancestry, not a local commit bit or current (G). Returning to the opening anchor reimposes it. Retaining a descendant carries the actual first-DATA witness. ANCHOR-only history cannot discharge it; DRAIN requires the positive DATA quota to have been reached. Full Vote, recovery histories, own-session receipt processing and Completion all preserve the predicate. The first primary is checked at the exact opening, fits by itself, and cannot occur twice in the batch.

**Location:** Same file, Context, Emission and Collection; main source lines 1320–1325: “Check the original NEWVIEW and ANCHOR prerequisites independently of the new input sidecars.”

**Check:** The initializer advertisement binds the exact historical report context and OC digest/key. A relay cannot substitute another certificate. A Byzantine initializer can split reports by advertising different valid OCs; the text correctly treats this as possible stalling, not a new Quit predicate. REPORT is a separate signature with no QC rank, lock, timer or entry power.

Original signed material is inspected before completeness rejection or duplicate suppression. Invalid/missing sidecars block REPORT or augmented DATA validation, but cannot suppress an otherwise valid original ANCHOR vote, NEWVIEW forwarding or original equivocation proof. A different selected request changes the DATA hash and is covered by the original proposal-equivocation rule; witness variants for the same DATA do not restart its timer. Earlier-term proposals remain votable under the original current-generation rule.

**Location:** Same file, Local report timing: “All correct members vote ordinary first DATA by (4\Delta) … initial/recovery term zero retains (6\Delta).”

**Check:** In the stated active, unclosed complete-message scope, ordinary entry/QC dissemination consumes (2\Delta), REPORT return one, and first proposal delivery one. Term zero uses the existing (2\Delta) leader wait and prepared entry skew; NEWVIEW reaches all by (e+4\Delta), matching ANCHOR votes and reports return by (e+5\Delta), and DATA reaches all by (e+6\Delta). No post-Enter live-pool scan is hidden in this sequence. Message-first processing handles equality with the (p=1) progress check. This validates the dependency needed by the unchanged timers within the stated interface; it is not a bound for an arbitrary native adapter outside that interface.

## 5. FINAL provenance and the public kernel

**Location:** `sections/rotation_v3_10_receipt_kernel.tex:33–94`, Soundness of the public FINAL kernel: “This step uses the raw threshold, not the definition of a valid regional receipt.”

**Check:** The proof begins with independently checkable signatures under authentic incoming context (C). At least one signature is an actual correct FINAL action. Correct FINAL originates only from Commit's timer, directly or by ancestor commitment. Vote precedes that timer and checks the full genuine augmented witness. Receipt-triggered commit and Completion emit no new FINAL, so they cannot create a self-supporting cycle of receipts without a timer origin.

The induction over earlier source signatures is well-founded: an inherited-anchor FINAL must already exist before the later Vote uses it. This covers an unaccepted regular anchor without assuming its global acceptance. Hash binding transfers the conclusion only to the exact supplied DATA and exact same-session path. A receipt from another session cannot cover this session's DATA.

The public proof establishes existence of the actual correct signer's historical witness. It does not authenticate a publisher's alternative recovery generation, OC, report set, native state or execution output. Since FINAL deliberately omits (g,t), the public algorithm correctly avoids treating a publisher-selected recovery path as mandatory or certified. Current branch permission, authentic initial context, exact extension, source order and Force cutoff remain independent checks.

## 6. Whole original publication value and retained public sources

**Location:** `sections/rotation_v3_10_requests.tex:169–204`: “The Publish Layer input remains exactly (R_0). Erasure removes only the new separately framed regional sidecars.”

**Check:** The text preserves original DATA, FINAL, signed proposal/NEWVIEW/vote bytes, QCs, ancestry, authority, native and source denotations. It removes only OC/advertisement/REPORT sidecars from public carriage. Regional senders continue retaining and forwarding the augmented witness required for own-session validation. The public kernel is a sufficient checking projection, not a substitute for the publication service's complete-result input and not a new service for bare requests.

**Location:** `sections/rotation_v3_8_carrier.tex`, Structural normalization; `sections/rotation_v3_10_source_step.tex`, Common public-kernel completeness.

**Check:** Normalization preserves actual typed objects and baseline public references even inside ancestry or an invalid enclosing wrapper. It does not fabricate bytes from a hash, a private cache, an unresolved edge or a cycle. Optional recovery/native annotations cannot impose a new required public fetch or suppress an independent valid kernel. Legal public objects and applicable controls remain genuine dependencies. Retention includes rejected raw records and source-occurrence identities; public import is separately queued and has no append, lock or entry authority.

The representation lemma is correctly limited to the specified logical values. Its proof is not a universal claim about an unspecified legacy binary codec or external namespace discovery.

## 7. Ready/Enter, retries and delayed callbacks

**Location:** Main source, `CanEnter`, Ready and Protocol 4; `sections/rotation_v3_10_ready_bundle.tex`, Send and receive events.

**Check:** Historical handoff validity is separated from permission to enter now. The former may remain useful after Force; the latter requires the current open version, exact still-applicable fixed handoff, correct selected region, no close request or tombstone and no earlier initial entry. The durable flag and exact attachment are recorded atomically before signatures. A late callback retains the original target and anchor identity and cannot replace a fixed anchor or reset a recorded entry time.

The ReadyBundle carries an actual verified executable state, its provenance, complete target evidence and one valid OC. Correct entry immediately sends it to the fixed membership. A recipient may use this already valid state/certificate without finishing redundant local selection. Its certificate is a cache, not a local consensus veto. Reusing hydration/OC across an equal-context beta retry imports no old BFT locks, votes, timers or Quit proof.

**Material limit:** The text explicitly conditions the (e+\Delta) entry-skew conclusion on validation, adoption and atomic Enter being the original complete-message handler. It explicitly withholds that conclusion if residual native installation remains outside the handler. I have not replaced this formal interface condition by a new numerical benchmark demand, nor treated a real unbounded import as zero time.

## 8. Retirement and the inherited regional safety argument

**Location:** `sections/rotation_v3_10_retirement.tex`, Manager closure and FinishClose: “Closed prohibits every old-version … new commit timer” and “observing Closed alone does not cancel a timer.”

**Check:** The gate covers buffered proposals, terminal-QC rotation, authority callbacks, STATUS, NEWVIEW, INPUT_ADVERTISE/REPORT and initial/recovery entry. Existing timers retain their deadlines; genuine Quit and Completion still cancel them. Complete evidence and receipts are processed before simultaneous timer/entry/retirement events. The manager/archival owner supplies closure response even without a worker, after a tombstone, on duplicate delivery and after restart.

An unknown old snapshot does not invent a close time. A later complete old-session packet resolves the context and triggers a complete close multicast before refusing entry or returning from the archive. Required bytes and receive routes must already transfer before worker release. The (4\Delta) drain is not permission to abandon unfinished storage/publication or request duties.

**Location:** Same file, No silent closure, Closure propagation and Conditional regional safety, especially lines 180–229 and 271–370.

**Check:** For a surviving direct timer voted at (s), an early correct Quit or Completion at or before (s+\Delta) would reach the committer by expiry and cancel it. Initial entry comes from the specified Ready relay; recovery waits and complete quit/admission material supply processability by the same vote-window endpoint. An AUTH visibility delay is not silently replaced by globally simultaneous visibility.

Let (c) be the first complete phase-independent close. If (s+\Delta<c), no closed member can silently suppress its matching vote: doing so would force a complete close event by (s+\Delta). Correct votes then form the protecting QC before every possible subsequent recovery-lock snapshot. If (s+\Delta\ge c), all fresh/entry gates close by (c+\Delta\le s+2\Delta), whereas any entry to the next generation requires strictly more than (s+2\Delta). The late case therefore needs no universal protecting QC.

This distinction is necessary and is actually used. The included three-member example correctly permits a late honest local timer commitment with neither a QC nor a threshold receipt. It is not mistakenly promoted to complete-result publication or global progress.

For same-generation conflicts, the first incompatible signed history is forwarded before processing the competing complete history; its admissible equivocation evidence reaches the committer by expiry. The preserved original evidence-first path is therefore essential. Across generations, the early-case protecting QC and highest-rank lock permit the original well-founded parent/ANCHOR descent. The late case excludes the needed later correct signer. DRAIN's same-session positions, endpoint restriction and lower-ranked parent retain this descent. Receipt-based commitments trace back to the already analyzed direct timer events and do not introduce a circular agreement premise.

No new safety-breaking retirement path was found. The proof's statement correctly remains conditional on baseline DATA/DRAIN/application soundness and the precise complete-message Ready contract.

## 9. Force, accounting and public prefix preservation

**Location:** `sections/rotation_v3_6_control.tex`, Protocol 5; `sections/rotation_v3_10_exports.tex`, Acceptance checks and Commit one DATA; main source, Protocol 6 and Common interpretation proposition.

**Check:** Complete due controls are applied in canonical order; the first MSIG matching the entry beta outranks AUTO. Other due MSIGs are consumed as stale without applying attached changes. Force retains exactly pre-payload (G), increments beta once, and grants no new-version DATA acceptance in the triggering block. It does not use the old region's last speculative or locally committed tip as the retained anchor.

Zero-output Force preserves the responsible numeric slot; positive accepted output advances it. Pending boundaries retain their stored target slot across further Forces. Engine identity remains ((\beta,r)), while AUTH remains immutable by (r,g). The strict source-reference-before-containing-block check also prevents same-record self-authorization or a new forced suffix in the triggering block.

P6 inspects candidates only at (h(G)+1), checks the exact hash-linked extension and currently authorized duty, and commits one DATA with its count/cursor atomically. It can use another valid proof when one wrapper fails. A terminal DATA commits and becomes visible before successor-authority resolution; a later unresolved boundary cannot undo it. A durable subposition prevents duplicate prelude or count application. A completed old record is not revisited for retroactive append.

With identical authenticated sources and the same checkpoint, these operations are deterministic. Regional safety supplies uniqueness of a valid next DATA within the fixed duty; ordering alternative proof objects does not choose among forks. Different replay subpositions therefore produce prefix-comparable accepted DATA. Closed-version receipts remain historical evidence but never acquire fresh append permission. STATUS/ON, prepared certificates, late source import and callbacks cannot reopen a closed beta.

## 10. Limits of this verdict

This review checked the new ownership/input mechanisms where they feed validity, original regional timer premises, inherited-context verification, retirement and public append authority. It also checked the finite-slot and first-DATA coverage claims as prerequisites of those paths. It does not turn correct-only certificate weight into quorum intersection or a selection event into accepted service.

The source still assumes sound native/application and DA correspondence interfaces, correct obligated participants' retained service, cryptographic security and authenticated complete source histories. The concrete packet dependencies must actually be admitted by the original complete-message/processing scope; finiteness alone supplies no uniform processing bound. These are the stated boundaries of the formal result, not newly discovered safety failures.

In particular, no safety finding does not establish the full requested end-to-end liveness result. Continuing global DATA, repeated accepted service for every region under its original persistence scope, and ingress-to-recurring-service composition require their own complete quantifier audit. No change was made to the frozen protocol or its proofs during this review.
