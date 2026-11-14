# Summary

**Final Verdict:** The reviewed v3.7 public FINAL kernel and conditional prefix-safety/lifecycle arguments are valid under their expressly stated fixed-snapshot, accepted-lifetime key-compromise, complete-validation, source-authentication and complete-message premises; this fresh review found no Critical Error or Justification Gap in those claims.

**List of Findings:** None in the assigned kernel/safety scope. This is a conditional mathematical verdict, not a certification of a concrete codec, native-state adapter, implementation, or full request liveness.

The request-inclusion objective remains unresolved, and the manuscript explicitly gives an execution withholding a persistent request from `G`. Passing the safety/kernel claims does not complete that objective. The public kernel establishes hash-bound historical input validity; it does not certify an unbound state snapshot, native execution output, DA correspondence, or a publisher's alternate admission sidecar.

## Frozen artifact and verification coverage

- Read `review_scope.md` and the parent `manifest.json` before review. No other review report or prior verdict was read.
- Recomputed all **23/23** parent-manifest SHA-256 hashes successfully. Main SHA-256: `440d639a3f9a7a0a236836153880171db40c904a1862eef2444b8313c9d2b476`.
- Independently checked all four entries of `snapshot/baseline/manifest.json`: all matched.
- Extracted the candidate's entire Model section and verified exact byte equality with `baseline/model_and_assumptions.tex`, SHA-256 `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387`. This establishes preservation, not semantic adequacy by itself.
- Read the full v3.7 main source and every active textual input, including the nested carrier input. The recursive input closure has 13 source files and six Protocol environments. Also read the two supplied standalone architecture-figure sources; their compiled PDFs passed the manifest hash check.
- Read the complete inherited Part I source. Covered the complete v3.1 baseline by comparing its unchanged spans against the fully read candidate and reading all 592 differing/deleted baseline lines, including its complete original EXPORT, handoff, Force, observer and guarantee sections. The baseline is used as source context, not as a trusted prior verdict.
- Verification was manual proof and state-machine analysis plus hash/input-closure checks. No executable/model-checking result, LaTeX build result, or PDF-layout certification is claimed. No snapshot source was modified.

All locations below are relative to `formalization_v2/reviews/v3_7/snapshot/`. `main` denotes `sync_hotstuff_rr_part2_rotation_v3_7.tex`; `kernel`, `carrier`, `exports`, `source`, `retirement`, `control`, `ready`, `health`, and `liveness` denote the corresponding active files in `sections/`.

# Detailed Verification Log

## 1. Assumptions and dependency map

**Location:** `main:362–378`: “The bound (F_i) counts every key of the snapshot that is compromised while its certificates are still accepted.”

This is the appropriate cumulative bound for historical FINAL verification. A set of distinct snapshot signers with weight greater than (F_i) contains a key outside the entire accepted-lifetime compromised set. Unforgeability therefore identifies an actual correct signing action, not merely a key that happened to be honest at an unrelated time. Later slashing cannot reweight the certificate. A mobile adversary that eventually compromises more than (F_i) old-snapshot weight while those certificates remain acceptable violates this explicit premise; it is not an in-model kernel counterexample. The assumption can be demanding when historical certificates remain useful indefinitely, but its scope is stated.

**Location:** `main:255–269`: “For (g>0), only the leader schedule changes”; “A new branch version that reuses (r) uses the same authority at that index.”

The authority namespace is immutable. A new beta changes the engine identity, not the recorded authority or old certificate weights. Every signature also binds the assigned region and configuration (`main:793–812`). Reusing a numeric slot therefore does not permit importing an old beta's votes, locks or quit proof.

The safety dependency chain is:

1. Authenticated incoming boundary and `AUTH_{r,0}` fix the producing context.
2. Raw FINAL threshold plus the cumulative compromise bound identifies a correct FINAL action.
3. The regional generation rule identifies its earlier fully validated timer commit.
4. Induction through earlier historical-anchor events establishes intrinsic input validity without assuming the public rule's own soundness.
5. Exact same-session coverage transports this conclusion to the candidate DATA.
6. Independent public boundary, ancestry, quota, control, source-order and Force gates authorize an append.
7. Deterministic per-DATA transitions yield common append-only public output.

Regional agreement is a separate dependency. It comes from the timer/lock argument and its retirement qualification, not from ordinary quorum intersection at threshold (>F_i).

## 2. Regional bodies, references, quota and full admission

**Location:** `main:612–619`: DATA increments DATA height; DRAIN preserves it while incrementing physical position. `main:727–748`: a drain generation has one DRAIN and no next term.

These type and position rules preserve a strict physical parent chain. DRAIN can provide a timer committing terminal DATA, but cannot become global application DATA or create quota. Repeated interrupted drain generations can lengthen the regional chain; the specification does not claim a fixed total DRAIN count.

**Location:** `main:941–955`: “Regional Vote, recovery and own-session evidence/receipt processing retain this full procedure”; validation checks application validity “throughout the required ancestry.”

The public refinement is not installed into the current session's Vote or recovery verifier. The full procedure still checks actual signed proposals, parent QCs, admission, positions, quota, both reference rules and native input validity. The regional reference path may use a validated cache only under the existing complete-bundle resolution condition. This is the necessary ground for the kernel's later existential-witness conclusion.

**Location:** `main:653–660`: public `CheckFINAL` establishes intrinsic regional reference validity, while global acceptance separately authenticates Espresso context and requires strict precedence.

The separation is coherent. A signature can attest that the original signer validated its historical regional references without proving the current observer's complete control view. The independent public comparison against the containing block remains necessary and is retained.

## 3. Raw FINAL soundness and noncircularity

**Location:** `kernel:15–31`: Raw FINAL checks matching signatures under the authenticated fixed context and “does not include” receipt condition (iii), generation/term, or attached admission-witness validity.

Raw FINAL is genuinely a cryptographic/type predicate. Its context is supplied by authenticated boundary history rather than selected by the sender. No hidden call to “valid regional receipt” assumes the conclusion being proved.

**Location:** `kernel:52–71`: the correct signature is an “actual correct FINAL action”; the signer fully checked its session history before starting the timer.

The threshold implication is valid under the stated cumulative fault bound. Hash binding identifies the later supplied DATA bytes with the exact object signed by that correct replica. The conclusion is existence of at least one genuine historical regional witness. It does not establish that a publisher-selected alternative `PROPOSE`, `AUTH_{r,g}`, QC encoding or native sidecar is valid. This distinction is needed because FINAL deliberately omits generation and term.

**Location:** `main:1147–1153`: Commit sends FINAL for DATA committed by a surviving timer. `main:886–891`: a receipt-triggered commit “sends no FINAL.” `main:1172–1184`: Completion commits and cancels timers without issuing fresh FINAL.

These operational rules support the provenance argument. A receiver cannot bootstrap the first correct FINAL merely by presenting a threshold certificate to Completion and eliciting new signatures. A DRAIN timer can emit FINAL for terminal DATA committed as an ancestor; it never emits FINAL for DRAIN itself. The retirement gate expressly preserves an existing timer's execution and resulting FINAL (`main:1061–1073`), so retirement introduces no alternate signature origin.

**Location:** `kernel:73–85`: inherited-anchor dependencies are justified “by induction over execution events from the trusted initial checkpoint.”

The induction is well-founded in the stated operational model. An inherited anchor's source signatures must already exist when the new Vote uses them; each correct source signature has an earlier timer origin. The new Vote still checks its own-session proposal and exact executable continuation. An unaccepted REGULAR terminal anchor is covered by this reasoning because source commitment precedes the successor's use even though public acceptance need not do so. The proof never uses global acceptance of that unaccepted anchor as its premise.

The original model's general sound-validation interface assumption is not being used here as an assumption that Raw FINAL is sound. It supplies soundness of the actual checks performed by the correct regional signer and of the independent native/source interfaces. The new threshold-to-input-validity implication is separately derived.

## 4. Same-session coverage and limits of native certification

**Location:** `kernel:88–115`: `CheckFINAL(C,B,K)` checks exact body types, hashes, parent links, positions, endpoint equality, same-session coverage and quota. Supporting DRAIN bodies are retained.

The raw certificate identifies its direct root, while the exact hash-linked path identifies the covered ancestor. This does not reinterpret signatures on a descendant as direct signatures on the ancestor. The operational ancestor-commit rule transfers commitment and input validity along that path. A later session's receipt cannot substitute for the earlier producer's receipt; this is stated both here and in `main:894–908`.

**Location:** `kernel:151–160`: the kernel does not authenticate an “arbitrary unbound native context, supplied state object, claimed execution output, or later (C_{\rm pub}) publication.”

This correctly limits the implication to claims bound by the DATA and actually checked by the sound regional input-validity interface. Ready still requires recovered, verified exact native continuation (`main:1983–2007`). Faithful later publication separately checks order, multiplicity, execution and DA correspondence (`main:2244–2252`). No general executable-state or output theorem follows solely from FINAL.

## 5. Historical receipts, REGULAR boundaries and current permission

**Location:** `main:1883–1948`: `VerifyHandoffHistory` authenticates historical fixed fields and “grants no entry permission.” `main:1951–1971`: `CanEnter` requires the current open version and checks durable close/entry records.

This separation prevents the historical-verification deadlock in which an old retained anchor becomes unverifiable merely because its producer is now closed. A valid old receipt can support an exact retained anchor without reopening the old engine or granting its suffix append permission. REGULAR verification still recursively authenticates the producer's initial boundary, fixed quota, exact terminal DATA and historical selection. The producer's optional recovery-admission sidecar is not required by the public kernel, but the target's authority, actual state and own-session admission remain required.

Adversarial trace checked: retain DATA (B) from beta 7 after Force; present its old receipt in a beta 8 handoff; separately replay a beta 7 successor receipt. The first can establish historical anchor provenance, but `CanEnter` rejects re-entry to beta 7 and public acceptance rejects any new beta 7 DATA. The beta 8 target must inherit exactly retained `G`, with the original producing pair/hash. No relabeling is authorized.

## 6. Logical carrier normalization and common public completeness

**Location:** `carrier:17–54`, `carrier:56–102`: actual field/list objects and reference denotations are preserved; “Private caches implement this public object lookup; they do not add new public denotations.”

The preservation result is a statement about the specified structured logical EXPORT and its actual evidence graph. Children remain inventoried even if their enclosing proof or an outer hint is invalid. Shared objects retain their incoming edges, and source references retain their authenticated denotations. Unresolved edges do not create objects. The specification explicitly excludes a blanket compatibility result for an unspecified binary codec or private-only resolver; that is a documented implementation limit rather than a failed mathematical claim.

**Location:** `carrier:68–93`: unresolved optional edges are retained without acquiring their targets; required DATA/FINAL projection is resolved through the public input interface.

An adversary cannot make an optional recovery authority or native sidecar into a mandatory public fetch merely by attaching it. The actual DATA/FINAL objects must already be supplied inline or through their legally available public denotations. The text does not claim that omitted required bytes can be extracted from an unknown optional blob.

**Location:** `source:83–126`: the common basis excludes private newer checkpoint additions; a specified legal source-object cache miss requires retrieval, while absent required DATA does not complete a kernel.

Adversarial trace checked: observer A privately holds a body absent from the record and all permissible earlier public sources; observer B does not. A's private copy cannot complete the record, so it cannot accept a block B must reject. If the body actually has a legal earlier public occurrence, B retrieves/resumes rather than reinterpreting the record as permanently absent. Full raw retention includes rejected records, preventing session exit or earlier application rejection from changing the public basis.

**Location:** `source:44–81`, `source:191–210`: beta replacement, checkpoint compaction and worker exit do not delete referenceable bytes; hydration uses a fixed finite cut.

The retention induction is valid at the abstract level and makes no finite-storage claim. Eventual retrieval of a fixed finite historical set gives a finite completion time without a uniform retrieval bound. New namespaces are retained while this fixed task runs, avoiding a moving hydration target. This is not a claim that a hash or eventually accessible archive pointer supplies resident bytes.

## 7. Per-DATA acceptance and durable resume

**Location:** `exports:78–100`: candidate acceptance requires the current producing version, fixed incoming duty, exact extension of `G`, next height, `CheckFINAL`, quota, strictly earlier Espresso references and the post-Force containing-block cutoff.

These checks remain independent of the kernel's intrinsic-validity implication. False outer chain/version/base/interval hints neither authorize the wrong signed context nor suppress a complete valid candidate. The canonical proof order chooses evidence, not a competing block; compatibility of different valid same-session commitments depends on the separate regional-safety result.

**Location:** `exports:107–162`: “There is no all-or-nothing outer EXPORT transaction”; terminal DATA commits before obtaining the next authority.

Adversarial trace checked: a record contains valid terminal DATA (T), then an invalid successor candidate or a successor requiring unavailable `AUTH_{r+1,0}`. `CommitNextDATA` first persists (T), accounting, the cursor and pending REGULAR boundary. The later failure cannot reverse that prefix or make acceptance depend on the successor authority. This also handles quota one without importing an unbounded next-slot authority wait into the first-DATA acceptance step.

**Location:** `exports:164–199`, `main:2165–2219`: the cursor includes `(q,n,j,nu,stage)` and a pending boundary; duplicate delivery resumes without repeating counts or the prelude.

The cursor and next-height scan distinguish a committed DATA transaction from a later unresolved boundary operation. A crash/resume cannot recount DATA, reset its window, or rerun Force. A completed past record is not reopened to accept later missing DATA retroactively. Its authentic objects may still be evidence for a later record. Genuine namespace/control/boundary absence causes a wait rather than an invented empty history.

## 8. Force, cutoffs and immutable authority reuse

**Location:** `control:207–264`: independent controls precede the first due MSIG matching the entry version; AUTO is evaluated only if that MSIG did not close it; one prelude increments beta at most once.

The event order is deterministic even when activation timestamps are equal or one timestamp jump crosses several thresholds. Other due MSIGs are consumed under the captured entry context, so a second due record targeting the just-created version cannot produce a same-prelude cascade. A stale record's attached configuration is not applied. Missing target-authority evidence pauses the canonical selection computation, not the cutoff.

**Location:** `control:240–249`, `main:1791–1823`: Force retains exact pre-payload `G`; zero accepted output reuses (r), positive accepted output advances to (r+1), and unassigned boundaries retain their stored target slot.

Adversarial trace checked: old-version DATA appears in the triggering block at the exact deadline. The prelude closes its version before the record is inspected, so it cannot repair the expired window. New-version DATA is also forbidden in that triggering block. In later blocks, it must extend the exact retained anchor under the new pair. Repeated zero-output retry cannot import old core state even though it reads the same immutable authority namespace.

The window arithmetic is consistent with half-open windows and positive targets. Empty-window batching computes the same number of consecutive failures as sequential settlement. It does not insert payload credit between crossed deadlines or reset a new duty to an earlier missed deadline.

## 9. Complete-close retirement and inherited regional safety

**Location:** `retirement:43–120`: the persistent manager retains complete closure evidence, relays for known snapshots in every phase, and preserves unresolved snapshot-discovery obligations.

The important case is a member that is closed before learning an old session's snapshot. Its durable Closed flag immediately blocks fresh work, but does not invent a per-session drain clock. A later complete Ready/proposal/history packet supplies that historical context and triggers closure multicast before refusal, duplicate handling or a tombstone response. A never-created or already-terminated worker is therefore not a silent missing voter.

**Location:** `retirement:171–221`: suppression within a proposal's (s+\Delta) window implies a complete close event by that time; all members then close by (c+\Delta), surviving timers expire by (c+3\Delta), and their FINAL messages arrive by (c+4\Delta).

The inequalities follow from actual complete relays under the regional bound. The local threshold (c_v+4\Delta\ge c+4\Delta) is conservative. Message-first processing handles equality. The argument does not assign the drain start to an earlier bare closure observation, and does not bound receipt assembly or ownership transfer.

**Location:** `retirement:238–368`: conditional regional safety uses an early/late timer-window split rather than an unconditional universal-QC lemma.

For a surviving timer voted at (s), an early correct Quit or Completion would arrive and cancel it by (s+2\Delta). In the early-close case (s+\Delta<c), no close can suppress the matching votes, so the protecting QC reaches prospective later entrants before their lock snapshots. In the late-close case (s+\Delta\ge c), all correct entry gates close by the timer expiry, while any next-generation entry would have to occur strictly after that expiry. Thus an absent protecting QC cannot be exploited by a later correct recovery signer. Same-generation incompatible signed histories still expose a quit proof within the vote window. Receipt-based commits trace to earlier direct timer origins rather than being assumed safe circularly.

This agrees with the inherited Part I steps (`baseline/sync_hotstuff_rr_part1_core.tex:735–831`) where their prerequisites apply. The text appropriately withdraws universal protecting-QC and guaranteed receipt-formation claims for late retiring timers (`retirement:370–381`). For example, one late correct voter can commit after others close without ever obtaining a threshold receipt; the specification preserves that local log and does not call it Completion.

**Location:** `retirement:122–169`: FinishClose additionally requires drained/canceled timers, durable data/evidence and explicit ownership of outstanding control/receipt/publication duties.

Elapsed (4\Delta) alone is insufficient. The complete-close record, responsibility transfer and persistent dispatcher block late callbacks from restarting an engine and retain historical data after worker exit. The protocol does not equate termination with stake withdrawal or deletion of accepted history.

The regional-safety result explicitly depends on the complete Ready/Enter handler contract. `ready:107–146` correctly withholds the (e+\Delta) entry-skew conclusion for implementations that leave an unbounded database import after message validation. Sound native state evidence alone does not prove that implementation timing property. This is an express scope condition, not an unconditional implementation guarantee established by the review.

## 10. Public prefix induction, progress dependencies and remaining objective

**Location:** `main:2483–2538`: common interpretation is indexed by canonical committed subpositions, and distinguishes public replay results from local future-control queues or caches.

The induction uses compatible authenticated histories and deterministic prelude/record/DATA/boundary steps. The common public basis makes cache differences irrelevant to semantic completeness. Every successful DATA step appends the exact next block; every control, Force, selection and lifecycle step preserves previously accepted DATA. Different observers can pause at different subpositions, including immediately after a terminal commit, while their DATA prefixes remain comparable. No local retirement operation mutates `G`.

**Location:** `liveness:63–154`, `liveness:158–289`: the finite regional-response and prepared-public-response arguments retain full own-session processing, finite fixed-slot authority preparation and the original complete-result publication input.

The safety review found no hidden replacement of a complete-result input by a bare kernel. The kernel removes optional public fetches but does not relax actual regional validation or payload retention. The already-recorded carrying case uses the original record, actual objects and causal signature/reference order; a bad outer hint alone does not require a corrected publication. Prepared SOURCESTEP timing is expressly restricted to the regional complete-message handler and admitted input/load limits, not arbitrary external-observer CPU or native restoration.

**Location:** `liveness:291–352`: the stalled-frontier argument compares quadratic timestamp demand with linear real-time retry bounds. `liveness:387–440`: repeated service discharges finite owed debts while preserving unrestricted REGULAR selection.

These branches do not enter the kernel or prefix-safety proof. The claimed inequality uses the stated upper clock-lead bound and increasing miss tolerance; it does not need a lower bound on timestamp lag. The owed-service argument distinguishes Force-selected debt from ordinary historical REGULAR selection and does not silently filter all normal successors through the owed set. No new safety exception is introduced by either mechanism.

**Location:** `liveness:442–478`: “Global progress and repeated regional credits do not yet prove that every admissible correct-client request reaches (G).”

The supplied request execution is consistent with the distinction being made: accepted empty DATA gives positive output and advances the next Force to a new slot; a late authority can allow a correct request-bearing proposal whose receipt forms after cutoff. Repeating finite but unbounded new-slot authority waits can leave that request pending while `G` grows and regional debts are discharged. This is an acknowledged limitation of the overall objective, not a contradiction of the qualified kernel/prefix-safety propositions reviewed above.

## Final scope statement

No in-model Critical Error or Justification Gap was identified in the assigned safety/kernel claims. The verified conclusions depend on the frozen cryptographic and full accepted-lifetime fault assumptions, sound source/native validation interfaces, the specified full regional and closure processing, and the explicit abstract Ready/source-handler contract. This review does not establish concrete transport compatibility, executable performance, bounded storage, arbitrary observer processing deadlines, or request inclusion.
