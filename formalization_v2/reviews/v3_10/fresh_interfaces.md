# Summary

**Final Verdict:** The model/service/interface argument is viable under the explicitly stated complete-message/native interface and the human clarification, but the advertised reduction from the frozen regional ingress sentence to a correct owner's authenticated receipt has one narrow **Justification Gap** in its operational meaning. I found no Critical Error in the audited ownership, certificate, publication, timing, or conditional service chains. A separate formula typo does not invalidate those chains.

This is a fresh independent interface audit, not a whole-goal completion claim or a deployment certification. I read only `review_scope.md`, `manifest.json`, the frozen `snapshot/`, and the proof-verifier skill. No source repairs were made.

**List of Findings:**

- **JG-I1 — recipient-level meaning of original ingress.** Location: `snapshot/baseline/model_and_assumptions.tex:310-312`, “Correct clients' admissible requests reach the regions that continue serving,” versus `snapshot/sections/rotation_v3_10_liveness.tex:593-596`, “x reaches a correct regional request owner p ... Its authenticated membership has some finite index k.” The latter is the precise event needed by the proof, but the former does not define which recipient or receive/admission event constitutes reaching a region. This is a specification/quantifier gap, not a demonstrated failure after an actual correct-owner receipt. If the original regional ingress statement already means a complete authenticated receipt at a correct member of every continuing region, the gap disappears and the rest of this reduction checks out. Neither a new delivery deadline nor delivery to every future roster is needed.
- **T-I1 — typographical/formula defect, not a substantive proof failure.** Location: `snapshot/sections/rotation_v3_10_ready_bundle.tex:203`, `s_v+1+mathbf 1_{k_v\ne\tagg{EMPTY}}`. The indicator's `mathbf` lacks its command backslash. The displayed byte count therefore contains ordinary mathematical letters rather than the intended indicator notation. The surrounding framing description and subsequent resource qualifications make its intended role clear.

**Integrity:** All 26 manifest hashes matched before review. The main-source SHA-256 is `1555702839d5d117a95428953d91efcd28154c96a719119a2d178086d7e45bab`. The 16,456-byte Model and assumptions block in the main source equals the frozen `baseline/model_and_assumptions.tex` byte for byte. Post-review integrity verification is recorded at the end.

# Detailed Verification Log

## 1. Frozen model and the meaning of a correct continuing owner

> “Within region i, the actual delay of a correct-to-correct message is ...”; “The delay bound includes delivery and processing of complete validation dependencies.”

The frozen network premise, together with the exact human answer in the scope, covers correct historical and current AUTH members in the same physical region. I did not require roster overlap and did not restrict service to members of one AUTH. Conversely, nothing here makes a failed machine correct or gives cross-region delivery a regional deadline.

Ownership lines 35-43 and 419-444 explicitly preserve the original owner's service until pending/outgoing duties end, or until the same logical owner and its authenticated receive route actually survive a local handoff. Retirement lines 135-162 require actual retained state, sidecars, unfinished sends and dispatch ownership before the worker releases them. This is implementable as a continuing node-level protocol duty under the clarified endpoint scope; it is not an undeclared external archive or arbitrary substitute custodian. A worker's `Terminated` state is not physical departure. No bounded physical-exit or storage claim follows.

## 2. AUTH-prefix existence, retrieval, and context preparation

> “Admission of r has a finite predecessor lineage containing the immutable AUTH slots 0,...,r.” — ownership:25-26.

The relevant index is an actually admitted target. Genesis admits zero; the specified regular/positive-output Force transition advances by one; a zero-output retry preserves the index. This supports the finite initial-authority lineage used by `D_i(r)`. Speculative successors do not erase the authorities through which they were admitted, and the candidate explicitly requires later checkpoints to retain that lineage's identities and authentication dependencies. The known-key retrieval does not enumerate all L1 authorities or wait for absent future keys. Its eventual availability is a preparation dependency, not a new bounded L1 lookup.

> “Let a be the time of the last actual envelope send ... At or after a+2d ... Begin the first CUT_QUERY ... s_v >= max{a'+2d,release(r)}.” — ownership:144-162.

The context barrier uses actual sends, not queue insertion. For each required correct owner, all fixed chunks arrive/process by `a+d`; the validation trigger starts after that; the trigger finishes by `a'+d`, strictly before the query. The maximum `d_i(r)` covers only the finite historical/target prefix under the clarified physical-region communication scope. It is not a maximum over arbitrary future configurations.

This reasoning uses the original complete-dependency source-authentication operation. It does not derive a deadline merely from the capsule's finiteness. The source explicitly places missing-history acquisition, native reconstruction and any whole-object work outside that operation before the trigger. Under that declared abstract interface I found no extra clock/service premise in this barrier. It would not prove that an adapter with residual unbounded validation/import work satisfies that interface.

## 3. Raw, authenticated, and admissible receipts

> “ReceiptJournal is an append-only sequence of first complete authenticated request receipts”; “RawJournal records complete immutable raw-object handles.” — ownership:65-73.

The two stores are distinct. Complete authenticated receipts enter the immutable query journal; raw hydration handles enter the raw store before ordinary canonicalization. Selection freezes both the raw prefix and the canonical-store version after checking HYDRATED, then actually performs canonicalization, authentication, exact-parent admission and minimum search. No certificate treats an unvalidated identifier as a canonical duplicate or an unscanned public namespace as a request handle.

The public-import loop separately retains source descriptors, scans finite fixed inputs and queues finite checks at retained exact parent contexts. Its invalid/unvalidated records cannot suppress a later valid identity. It is eventual and nonblocking: source relay, P6, Ready, REPORT and timers do not wait for its backlog. The direct owner-to-target argument does not depend on a public carrier for the request having already formed.

### JG-I1: the first actual receipt is still the input premise

> “Apply the original ingress statement to this continuing region C: x reaches a correct regional request owner p at some finite actual receipt time t_x.” — liveness:593-595.

This is the only arrow in the audited request chain whose endpoint is more precise than its cited original wording. The frozen text says that requests reach continuing **regions**; the proof needs receipt of the complete authenticated request by a **correct authenticated member** with a usable journal event. The human clarification resolves old/new correct-member communication, not this initial recipient predicate.

Under a literal weaker interpretation that physical delivery to any member constitutes reaching a region, a client can deliver only to a Byzantine member of an otherwise serving region. That member withholds the request while all correct replicas continue producing valid empty DATA. All subsequent owner-transfer rules are inapplicable because no correct owner ever received the bytes. This is a diagnostic of the ambiguous premise, not a claimed attack satisfying an intended correct-recipient definition of ingress.

The supplied Part I is more explicit: `baseline/sync_hotstuff_rr_part1_core.tex:214-220` assumes requests eventually reach all correct replicas. Importing that entire fixed-committee input-fairness premise into arbitrary future Part II rosters would itself change the present proof's claimed ingress scope; Part II also intentionally uses different local FIFO orders and global-removal semantics. The present proof should therefore be evaluated against its stated regional ingress meaning, rather than silently borrowing the stronger Part I quantifier.

**Dependency consequence:** the actual-correct-owner conditional theorem in requests:250-295 and the hydration/opening propositions remain checked. The final theorem's claim to follow from the original ingress sentence alone is unverified until that sentence's recipient-level meaning is fixed. Conditional on the intended correct-owner receipt, continue through the remaining steps below; no new bounded client retry service is required.

## 4. Immutable transfer jobs and every-certificate coverage

> “Later roots or larger counts cannot replace it”; “never add slots, enlarge L, replace H or restart.” — ownership:187,217.

Every directory query has its own recorded actual send time and descriptor deadline. The first valid descriptor freezes a finite record count; each record freezes metadata, length and fragment count. Silence, malformed replies and Byzantine equivocation can fail or enlarge the initial finite job, but cannot perpetually extend an already frozen job. Wrong-sender and out-of-context packets cannot fail a correct owner's record. Equality processes due replies before expiry. Local whole-record checks are actual terminating preparation without a newly asserted `d` deadline.

> “Only at or after b_v+2d may v create its signed share.” — ownership:258-259.

`b_v` is the last actual metadata/chunk send to any target, and target receivers run independently of their production or acquisition jobs. Thus every correct target has each complete raw handle by `b_v+d`, strictly before a correct share. A threshold greater than F supplies at least one such correct signer in every valid HYDRATED certificate. Mandatory share collection, full-certificate formation and relays make the finite formation proof operational. Cancellation requires an already valid certificate and retains outstanding duties.

The coverage result concerns **every** valid HYDRATED certificate, including one assembled later from old shares. Positive `release(r)` makes all correct first queries at or before a fixed receipt time lie in a finite set of numeric slots; equality is deliberately exceptional. Nonexceptional cuts include the retained original owner. A fixed numeric slot can contribute positive accepted DATA from only one pair before the public transition advances it. Hence zero-output beta retries cannot multiply its quota, and the finite-exception claim does not assume a finite union of future rosters or a global event-rate limit.

## 5. Executable state, opening offers, OC and REPORT

> “Every correct native preparer sends without waiting for an offer, certificate, Ready or acknowledgement.” — input_selection:77-78.

STATE_CANDIDATE breaks the potential state/offer/Ready cycle. Its actual construction, validation and finite transfer have no new native deadline. After hydration, each correct offer comes from a fixed finite raw/canonical cut and an actual exact-anchor eligibility computation. Equal-context beta retries preserve the cut, work, executable state and certificates.

Every valid OC has a correct offer signer. Every correct offer's cut follows a HYDRATED certificate whose prior all-target dissemination already supplied x in the nonexceptional case. Therefore every valid OC selects a body with key at most x's key when x remains eligible. This uses neither honest intersection across HYDRATED/offer/REPORT sets nor agreement on private pool roots. A collector need not fetch nonselected bodies to validate OC, and a local attached OC does not veto another valid one.

REPORT acknowledges the initializer's signed advertised OC at an opening, or an already validated first-own-session ancestry at a descendant. It does not run a live-pool search. Original ANCHOR voting, forwarding and admissible-equivocation inspection remain independently triggered even when the sidecar is absent or bad. A Byzantine advertisement split can stall its own term without creating a spurious new Quit proof.

The mandatory first-own-session predicate covers first DATA after recovery to the opening; retained descendants retain the historical witness. DRAIN requires prior positive quota and cannot discharge that obligation by itself. Receipt and Completion validation preserve the regional sidecars, while receipt-driven commitment emits no fresh FINAL.

## 6. Timing and original resource scope

> “If validation, adoption and the atomic Enter transition are the complete message-processing operation covered by ass:network...” — ready_bundle:139-140.

The entry-skew inference is explicitly conditional on an actually executable state object and its real supplied validation dependencies. The document separately gives the residual-delay expression when external native work remains and explicitly withholds the stronger claim in that case. It does not infer zero installation time from calling an operation atomic.

The added active footprint is specified: one OC, one selected body, one initializer advertisement and one report signature set, with normalized dependency sharing. Ready does not retransmit the historical owner directory or HYDRATED signature set and does not run a private scan. Under the original admitted complete-handler scope, the ordinary-term dependency path is entry/ancestry by `e+2Δ`, reports by `e+3Δ`, proposal votes by `e+4Δ`; term zero is NEWVIEW by `e+3Δ`, complete votes/reports by `e+4Δ`, correct inputs at the leader by `e+5Δ`, DATA votes by `e+6Δ`.

The supplied framing does not prove headroom in every numerical deployment envelope. The source expressly says this, and neither the audit task nor the abstract protocol requires a deployed codec benchmark. I therefore do **not** classify the lack of a benchmark as a proof gap. Conversely, these conditional inequalities cannot be quoted as an unconditional guarantee for a concrete adapter whose packets or native work fall outside the original interface.

**T-I1:** ready_bundle:203 has the malformed `+mathbf 1_...` term described in the Summary. No timing bound is derived from numerically evaluating that malformed term; the semantic framing and the conditional resource obligations are separately stated.

## 7. R0 publication, honest FINAL provenance and public source processing

> “The Publish Layer input remains exactly R0. Erasure removes only the new separately framed regional sidecars.” — requests:181-182.

The public input remains the entire original complete result: original DATA, FINAL, proposal/QC, ancestry, boundary, authority and source/native evidence required by the original interface. The amendment does not give bare requests, OC, HYDRATED or a kernel-only record a publication bound. Correct regional validators keep the augmented bundle; public erasure removes only the new sidecars.

Raw FINAL validity supplies a real correct signer under the accepted-lifetime fault model. Tracing that actual signature to a timer-origin Vote yields a genuine full regional witness, including the opening/REPORT rule. Receipt-based Completion emits no FINAL, avoiding circular self-certification. The public theorem concludes existence of the genuine witness, not validity of a publisher-selected replacement or arbitrary native claim. Independent global ancestry, boundary, branch, reference and cutoff checks remain mandatory.

The logical carrier preserves original structured objects and source denotations despite outer bad hints; it does not manufacture missing bytes or promise discovery in arbitrary unrelated namespaces. PublicStore retains rejected raw records before advancing availability, enabling later exact references. Source callbacks enqueue import work without waiting for it. Mandatory source dependencies remain distinct from optional recovery/native sidecars.

P6 commits and exposes each DATA before successor-authority resolution; a terminal DATA is not held behind `AUTH_{r+1,0}`. Canonical subpositions preserve the original timestamp, and a completed rejected record is not revisited to gain append permission. Source handlers are charged to the original complete-message abstraction for maintained, prepared regional replay, not to arbitrary observer catch-up or unbounded native reconstruction.

## 8. Global service and conditional end-to-end request argument

The stalled-frontier proof fixes one slot/configuration and a finite physical region cycle. Historical AUTH, native state, request hydration, selection and fixed-cut public hydration are completed at finite, nonuniform times and preserved across beta retries. Its preparation maximum does not range over every future membership.

The finite health-controller credits leave finitely many matching interventions at a fixed frontier. Conditional AUTO cooldown does not remove the last eligible owed source. Once the finite prefix has passed, each zero-output AUTO immediately opens the next duty. The increasing `1+beta` tolerance gives quadratic required timestamp growth, whereas the prepared response bound and the original upper clock-lead premise give only linear elapsed-time growth in the no-DATA contradiction. No lower timestamp-lag assumption or public-window pause is introduced.

The regional-service proof counts accepted DATA rather than selection: finite owed debts cannot support infinitely many productive Force versions without serving a persistent debt; without infinitely many Forces, finite quotas and increasing regular opening heights let cyclic selection reach a persistently eligible region. This remains conditional on the region's serving premises and does not make a permanently missing correct quorum healthy. Empty DATA is not treated as request inclusion.

Conditional on JG-I1's actual correct-owner receipt, unbounded global DATA and a finite region cycle supply a region with infinitely many accepted contributions. Removing finitely many exceptional numeric slots leaves infinitely many aware accepted session-first blocks. Persistent admissibility at their exact accepted opening states is explicit. Each such block spends a new logical identity of key at most x's key; bounded canonical encoding length gives a finite potential, and accepted-prefix extension prevents spending the same identity twice. Speculative/discarded execution spends nothing. This completes the owner-receipt-to-inclusion implication without future-roster awareness or new public-request delivery assumptions.

## Post-review integrity

All 26 manifest hashes matched again after writing this report; there were zero mismatches. The main-source SHA-256 remains `1555702839d5d117a95428953d91efcd28154c96a719119a2d178086d7e45bab`. No compilation, model checker, execution test, deployment benchmark or source repair was performed in this verifier run.
