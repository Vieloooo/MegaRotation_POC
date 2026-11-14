# Summary

**Final Verdict:** The first-public-carrier import proof and its integration with the conditional REPORT, prepared regional-response, global DATA-progress and repeated-regional-service arguments are valid at the stated abstract-protocol scope; this review finds no Critical Error or Justification Gap in those scoped claims, and full request liveness remains **OPEN**.

**List of Findings:** None within the stated scope. The following are explicit hypotheses or withheld conclusions, not newly established facts:

- Import starts with an original complete public carrier and actual acquisition/retention of its request bytes. Neither an unresolved reference nor visibility alone establishes pending admission.
- A qualifying prepared parent includes the actually available, retained immutable native read context needed for the finite check. A live executable engine, a state root, or unfinished snapshot construction alone does not qualify.
- The owner must continue the prescribed local work. The protocol now specifies that duty; it does not derive a throughput bound, an import deadline, or finite storage from it.
- REPORT and the numerical DATA-response bounds retain the stated complete-handler/batch-processing and prepared-native-interface conditions. No concrete codec, native adapter or CPU-scheduling benchmark was supplied or verified.
- Accepted-report awareness is a separate hypothesis. The import proposition does not establish first-carrier formation or sufficiently early awareness in disjoint future rosters.

**Frozen artifact and method.** The reviewed main file is `snapshot/sync_hotstuff_rr_part2_rotation_v3_9.tex`, SHA-256 `63dd6770de6aaed0f442c53c6fcd3685fb5a37b478140d67970ee2c4719058cb`. All 24 manifest file hashes passed before review. The literal Model section in the main file equals `snapshot/baseline/model_and_assumptions.tex` byte for byte as text (16,456 characters). This is an independent source/proof review using proof-verifier discipline; only `review_scope.md`, `manifest.json`, and the frozen `snapshot/` were read as review evidence. No other review, author note, live source or previous verdict was consulted. No protocol source was edited and no model checker or executable implementation test was run. Final hash verification is recorded below.

# Detailed Verification Log

Locations below are relative to `snapshot/`. Quoted statements are followed by their verification and scope.

## 1. Dependency map and exact claim

> “If a correct node follows and retains those actual bytes and continues the local loop, it eventually retains \(x\) in \(U_v\) … [at a qualifying prepared parent] the request eventually enters the pending store unless its identity has already entered \(G\).”

Location: `sections/rotation_v3_9_public_requests.tex:134–145`.

The two conclusions have different hypotheses. The first is actual-byte import from a complete original carrier; the second additionally requires a prepared context in which the request passes the ordinary checks. The proof dependencies are:

1. Actual logical DATA occurrence and acquired bytes → durable source job.
2. Fixed finite source job plus FIFO local service → exact bytes in the unvalidated candidate store.
3. Candidate/prepared-parent trigger plus retained immutable context → fixed finite checking job.
4. FIFO checking service plus a successful ordinary check → authenticated pending admission, unless already globally included.
5. Pending admission → ordinary relay, exact-parent eligibility and retention; it does not itself imply accepted-report awareness or inclusion.

This is a conditional local theorem. The wording does not promise that a carrier forms, that an absent referenced target is fetched by this loop, or that any parent qualifying for successful validation is eventually prepared. Those omissions are expressly within the statement's scope, not missing premises needed to prove an unconditional theorem.

## 2. Source retention and actual acquisition hooks

> “append its actual retained root and dictionary cut … Do the same when an exact public reference target is subsequently acquired. On first enabling this rule, enqueue one fixed finite cut of the already retained roots”

Location: `sections/rotation_v3_9_public_requests.tex:23–31`.

The initial-work case is present. Starting the rule after bytes were acquired cannot leave the entire old store permanently unscanned. The fixed initial cut avoids a moving-startup-target argument; concurrent acquisitions create separate jobs rather than extending that cut indefinitely.

> “Durably record the retained root and dictionary cut … Acquisition of an actual exact public target records the same enqueue obligation. These are descriptors, not an in-handler scan or request check.”

Location: `sections/rotation_v3_9_source_step.tex:33–46`; also `149–164` and main file `1632–1636`.

These are actual store/receive obligations, not merely a premise appearing in the proposition. Raw retention precedes availability-cursor advancement and buffer release. SOURCESTEP sends before waiting for P6, and a target acquisition invokes its own enqueue/resumption obligation. Rejection or branch closure does not remove the retained bytes. Thus an unresolved edge can be skipped by an earlier job without losing the later acquired target.

The theorem correctly invokes the hypothesis of actual acquisition for the late-target case (`public_requests.tex:153–154`). It does not infer acquisition from saving an unresolved edge. No historical quorum, new request query or import-specific fetch service is introduced.

## 3. Finite typed traversal, malformed objects and cycles

> “extracts the actual entries of \(\mathit{batch}\) only when the local structure identifies a DATA body … Decode independently represented typed children even if a sibling or enclosing proof fails.”

Location: `sections/rotation_v3_9_public_requests.tex:33–55`; `sections/rotation_v3_8_carrier.tex:17–39,60–106`.

The new import does not reinterpret arbitrary witness strings or request hashes as bodies. Its DATA positions come from the original logical fields, and the carrier specification explicitly distinguishes graph denotation from acquisition. A false outer hint, rejected proof wrapper or closed version cannot hide independently represented DATA; malformed bytes with no such logical structure yield no invented request.

> “at its fixed dictionary cut, visiting each actual object once … saves an unresolved edge without waiting for or fetching its target”

Location: `sections/rotation_v3_9_public_requests.tex:47–51`.

The cut is finite and fixed, inline objects are finite, and each actual object is visited only once. Shared objects and cycles therefore do not create an infinite traversal. Subsequent dictionary growth does not enlarge the running job. Optional AUTH/native/source-history fetches cannot block its completion. This proves finiteness at the logical graph level; it is not a binary-decoder compatibility or bounded parsing-cost theorem, consistently with `rotation_v3_8_carrier.tex:150–181`.

## 4. Candidate separation and poison-resistant deduplication

> “Before successful request validation, deduplicate only identical bytes … A claimed client identifier creates no duplicate marker.”

Location: `sections/rotation_v3_9_public_requests.tex:57–64,82–94`.

Exact-byte equality or a binding commitment can only suppress a byte string already retained. A forged body asserting another client's identifier cannot reserve that identifier before successful authentication and canonicalization. A failed check is scoped to the actual candidate bytes and parent. Consequently neither malformed encodings nor failed authentication can suppress a later valid request by an asserted identity alone.

Canonical retransmission merging occurs only after the original interface establishes the actual logical identity. An unvalidated candidate is not eligible for REPORT, authenticated relay or execution. The proof's transition from a successful check to `enc(x)` and pending admission therefore does not use an unauthenticated identity assumption.

## 5. Prepared-parent triggers and retained read contexts

> “When … completely prepared an exact parent \(P\), enqueue a checking job for the finite current cut of \(U_v\). When a new candidate is retained and such a parent is already prepared, enqueue a check of that candidate at that parent.”

Location: `sections/rotation_v3_9_public_requests.tex:66–80`.

Both event orders are covered. A candidate already in the cut is scheduled by the prepared-parent event; a later candidate is scheduled by its own arrival while the qualifying parent is available. A new parent can schedule a candidate whose earlier check failed. Duplicate checks for the same bytes and exact context may be omitted without changing the result of the finite ordinary check.

> “The native interface has actually supplied an exact parent and retained read context. Record fixed candidate-check work … do not scan it here.”

Location: main file `1601–1603,1632–1636`; also `126–129`.

This is the necessary qualification of “completely prepared.” The interface event requires the actual retained context; it does not identify ordinary Ready with successful snapshot construction. A pointer to mutable engine state is explicitly insufficient. Required construction must finish before notification, with neither an instantaneous-copy assertion nor a new Ready wait. The theorem's prepared-parent premise therefore carries genuine availability rather than inferring it from a nominal pointer.

Each job retains that context until completion. Active-parent changes do not retarget or cancel the job. The proof thus cannot fail merely because \(G\), beta, a session anchor or an active speculative parent changes faster than background checks run. Conversely, the text proves no eventual availability of such immutable contexts in an arbitrary native implementation; that would be a stronger claim than this theorem makes.

## 6. FIFO service and persistence across lifecycle events

> “continuously alternates finite local steps on the oldest unfinished source job and the oldest unfinished checking job … New jobs go to the tail and cannot overtake a fixed job.”

Location: `sections/rotation_v3_9_public_requests.tex:96–107`.

At insertion, a fixed job has finitely many predecessors. Every predecessor and the job itself has finite work. Alternating nonempty journals ensures that an infinite stream of source arrivals cannot starve checking and an infinite stream of checks cannot starve source work. Tail arrivals do not change the finite predecessor set. Under the expressly prescribed continued local-work duty, both source retention and successful checking eventually complete. No wall-clock bound is obtained from this induction.

> “journals, bytes, read contexts and pending enqueue obligations belong to the node … worker exit transfers them durably and a restart resumes their exact cursors.”

Location: `sections/rotation_v3_9_public_requests.tex:109–117`; `sections/rotation_v3_9_source_step.tex:48–55`; `sections/rotation_v3_9_retirement.tex:130–161`.

The specification preserves unfinished work, its subposition, required bytes and enqueue obligations independently of the producing engine. FinishClose expressly transfers public-import journals, fixed native read contexts and pending-admission work to an accepting persistent owner before release. Waiting, Completed, Terminated, closure and compaction therefore do not create a missing-worker loophole.

The proof uses these stated durable ownership/update obligations; it does not require a whole scan or snapshot construction to be one instantaneous atomic operation. An implementation that persisted a candidate but lost its required check-enqueue obligation on restart would violate these rules. Crash recovery itself is not a bounded operation or a promise of eventual online service, and the theorem already requires continuation of the loop.

## 7. Pending FIFO admission and exact \(E_v(P)\)

> “preserving an existing entry's FIFO position or assigning the next first-receipt position to a new entry … A successful check at an old parent is not admission at a newer one.”

Location: `sections/rotation_v3_9_public_requests.tex:89–94,119–132`; `sections/rotation_v3_9_requests.tex:5–21,60–79`.

Existing authenticated pending entries keep their position. A new imported request obtains a pending position when successfully admitted; its earlier unvalidated candidate occurrence does not retroactively take a position in the authenticated FIFO. Raw candidate arrival numbers and validated pending order are different data. After admission, later requests cannot displace it; global inclusion, rather than speculative commitment or regular handoff, removes it.

The exact eligible projection still includes every authenticated pending request that passes admission and ancestry exclusion at the actual parent, fits alone and has full bytes. An old-parent import success supplies no new-parent eligibility shortcut. Complete ancestry includes inherited global history and speculative ancestors, and FIFO filling rechecks after earlier entries in the same batch. The report's key minimum is distinct from ordinary FIFO filling, as the protocol explicitly specifies.

The wording also forbids a convenient but invalid implementation interpretation: admitted pending entries cannot be omitted from \(E_v(P)\) merely because an eligibility rebuild is deferred. Parent/request processing completes the projection before exposing it. Reports already emitted remain immutable, so later import only affects later contexts and ordinary filling. This restriction preserves the REPORT proof without asserting a visibility-to-current-report deadline.

## 8. Nonblocking integration and local REPORT timing

> “Ordinary messages, timers, P6, Ready and report emission do not wait for the import loop, its backlog, or an import watermark.”

Location: `sections/rotation_v3_9_public_requests.tex:100–107`; main file `1601–1603,1632–1636`; `rotation_v3_9_source_step.tex:149–154`.

The actual hooks enqueue descriptors, rather than scan the source store or validate candidates in those handlers. Ready's enumerated conditions (main file `2009–2027`) contain no import watermark or pending-request transfer. This supports nonblocking integration at the declared abstract interface.

The more delicate step is exact-parent report selection, rather than raw import. Its work remains inside the explicitly stated completed parent/request/batch-processing abstraction (`rotation_v3_9_requests.tex:60–79`). Queue finiteness or a maintained index does not prove its concrete CPU bound. The text expressly withholds the timing result if an implementation leaves a pool scan, native import or eligibility rebuild outside completed processing. Likewise, ReadyBundle timing is conditional on actual executable-state adoption in the covered handler (`rotation_v3_5_ready_bundle.tex:107–146`). These qualifications prevent a hidden unconditional timing inference.

> “Ordinary first DATA is voted … within \(4\Delta\) … term zero retains \(6\Delta\).”

Location: `sections/rotation_v3_9_requests.tex:207–245`.

For an ordinary term, the independently disseminated terminal QC reaches correct members by \(e+2\Delta\); their reports reach the correct leader by \(e+3\Delta\); the first proposal is processed by \(e+4\Delta\). For term zero, NEWVIEW is sent by \(e+3\Delta\), processed by \(e+4\Delta\), and both the independent ANCHOR votes and reports reach the leader by \(e+5\Delta\), permitting the first vote everywhere by \(e+6\Delta\). Later positions cost at most \(2\Delta\). These satisfy the existing \(e_v+(2p+4)\Delta\) progress checks, with message-before-callback ordering at equality. The sharper old ordinary \(2\Delta\) first-vote claim is explicitly not retained. DRAIN adds no report stage.

## 9. Preservation of prepared regional response and \(L\)

> “\(D_i=(Q+1)(2K+12)\Delta_i,\quad R_i=(g_i^*+1)D_i+2g_i^*\Delta_i+9\Delta_i\)”

Location: `sections/rotation_v3_9_liveness.tex:63–158`.

The argument obtains a finite authority prefix from the fixed installed roster, eventual permanent exclusion and initializer fairness; persistent fetching and exact-anchor preparation survive beta changes. Preparation is a finite maximum of eventual completion times, not a uniform archive/native deadline. REPORT keeps the original progress-check ceilings and protected initializer window, so the conservative per-term \((2K+12)\Delta_i\), per-generation \(D_i\), recovery allowances and final \(9\Delta_i\) receipt allowance remain available. An import backlog is not a new prerequisite anywhere in this chain.

> “\(L=\max_i\{2T_E+2\Delta_i+R_i+T_{\rm pub}\}\)”

Location: `sections/rotation_v3_9_liveness.tex:162–293`.

The two source stages use retained old Espresso bytes, bounded new source visibility, complete regional source handlers and prepared exact anchor/authority state. Optional candidate witness targets, native request contexts and import checks do not enter the public kernel or these handlers. The result offered to publication remains the original complete \(R_0\), including its required old evidence. The service clock is not moved to a bare request, partial FINAL or kernel-only submission.

The closing alternative remains essential: the result extends the fixed stalled frontier if the retry is still open, or a Force has already closed it. Terminal DATA is committed before a successor authority wait, so quota one does not hide the first-growth event. The two \(\Delta_i\) allowances remain conditional regional handler bounds, not arbitrary-observer replay or native-restoration bounds.

## 10. Global and repeated regional DATA progress

> “\(\theta_{n+1}-\theta_n\ge M_nW_{\min},\quad a_{n+1}-a_n\le L\)”

Location: `sections/rotation_v3_9_liveness.tex:295–356`.

Under the specified stable-slot and health-controller conditions, the no-growth tail has finite preparation and eventually immediate AUTO reassignment. With \(M_n=1+\beta_0+n\), accumulated timestamp spacing is quadratic while the regional response alternative and upper clock-lead bound make elapsed first-finalization time at most linear. The contradiction uses no lower-lag bound and no import completion event. Thus the new local import rules do not alter this DATA-progress argument or its prerequisites.

> “A region \(C\) … contributes DATA to \(G\) repeatedly.”

Location: `sections/rotation_v3_9_liveness.tex:391–444`.

The owed-set proof retains its two cases. Infinitely many productive Force-created versions would discharge the finite outstanding debts, while an eventually unproductive Force tail contradicts global DATA progress. With finitely many Forces, unbounded accepted DATA and finite quotas yield infinitely many regular sessions; strictly increasing selection heights eventually pass the permanent-eligibility position and old cooldown expiry, and cyclic selection cannot skip an eligible \(C\) forever. Ordinary REGULAR selection remains unfiltered by the owed set. Empty DATA may discharge a debt, so this proves regional DATA service rather than request inclusion.

## 11. Accepted-report awareness remains independent

> “infinitely many accepted term-first DATA have a genuine report witness containing a correct signer whose eligible retained set contained \(x\) at its report event”

Location: `sections/rotation_v3_9_requests.tex:247–293`.

The potential argument is valid under this exact hypothesis. A correct aware signer reports a key at most that of \(x\); the certificate minimum is an actual executable primary with no occurrence in the exact accepted ancestry. Acceptance spends a previously unaccepted identity from a finite set of canonical byte strings whose keys are at most \(\operatorname{key}(x)\). Monotone \(G\) prevents spending it twice. Infinitely many such decreases while \(x\) remains absent are impossible.

The same-snapshot relay (`rotation_v3_9_requests.tex:23–44`) provides the stated bound only after actual full authenticated receipt/dispatch in that installed snapshot. Learning a new snapshot schedules retained relays but introduces no arbitrary-backlog or disjoint-roster deadline. Public import may complete after a roster's immutable reports; first-carrier formation is also not supplied. The text expressly keeps these gaps outside the proved theorem (`rotation_v3_9_liveness.tex:446–504`). Therefore passing this review must not be reported as solving unconditional request liveness.

## 12. Verification limits and final integrity check

This review checks the stated logical protocol, source hooks, retained-job proof, exact-parent REPORT semantics and their claimed liveness dependencies. It does not certify a deployed codec, state representation, crash-recovery implementation, scheduler, finite-storage implementation or measured processing time. It imports no stronger availability condition than the text's qualified prepared-parent and prepared-handler conditions.

**After-review integrity:** all 24 manifest file hashes were rechecked after creating this review and passed; the main hash remains `63dd6770de6aaed0f442c53c6fcd3685fb5a37b478140d67970ee2c4719058cb`. The only created artifact is this review file.
