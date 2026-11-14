# Summary

**Final Verdict:** The v3.11 conditional safety argument is valid within its stated original regional-validation, complete-message, executable-state, source-history and cryptographic interfaces; this fresh review found no Critical Error or Justification Gap in the inspected safety chain. This is not an unconditional deployment-soundness verdict or a certification of the separate end-to-end liveness theorems.

**List of Findings:** None in the inspected safety claims. In particular, the review did not find an OC-cancellation, nonselected-body, recovery-to-opening, receipt-provenance, retirement-window, or public-cutoff counterexample that satisfies the frozen premises.

The exact dependency limits matter. The snapshot expressly makes regional retirement safety conditional on its complete Ready/Enter installation contract and baseline DATA/DRAIN validation. A valid state proof alone does not establish bounded installation or packet processing. The public FINAL kernel certifies an actual correct signer's validated input history; it does not certify a publisher-selected alternate witness, arbitrary executable state, execution output or DA correspondence. Those restrictions are present in the statements and accompanying text and therefore are not classified as missing assumptions or proof defects here.

Review isolation: I read the supplied `review_scope.md`, `manifest.json`, frozen `snapshot/` sources and the proof-verifier skill only. I did not consult live sources, author integration notes, prior/peer reports or repairs. I used both supplied human clarifications: original communication covers correct old/new AUTH members in one physical region, and sending to a region means all parties in the region receive it. Neither clarification was expanded into a cross-region bound, future-roster delivery guarantee, failure recovery guarantee or native-work deadline. No source repairs were made.

Integrity: all **26/26** manifest entries matched SHA-256 before review and after substantive review. The main source hash was unchanged:

`de2e36905fef1e2de7bcc53f0cd8f4a95eca92e4561d347b33bc260e7dc58bef`

The entire `Model and assumptions` block is byte-identical to the frozen v3.1 main source and `baseline/model_and_assumptions.tex`, with SHA-256:

`1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387`

# Detailed Verification Log

## 1. Assumptions, authentication and dependency map

Location: main source, lines 253–574; `baseline/model_and_assumptions.tex`; `baseline/sync_hotstuff_rr_part1_core.tex`, lines 734–832.

> “The bound $F_i$ counts every key of the snapshot that is compromised while its certificates are still accepted.”

Together with unforgeability, distinct-signer checking, immutable authority and the fixed-session fault bound, weight greater than $F_i$ establishes an actual correct signer. It does **not** establish honest intersection between two such certificates. The inspected OC, REPORT and public-receipt arguments do not substitute threshold intersection for the separate dissemination and timer/lock arguments.

The safety dependency chain is:

1. Authenticated fixed context and accepted-lifetime key protection identify actual correct signed actions.
2. Full own-session validation and genuine Vote/Commit events establish intrinsic validity of timer-committed DATA and its ancestors.
3. Original equivocation inspection, protecting-QC/entry-lock reasoning, complete receipt propagation and phase-independent closure establish regional prefix compatibility.
4. Correct FINAL provenance and induction over earlier execution events establish the historical public FINAL implication.
5. Common source normalization, complete applicable controls, authentic boundaries, current append authority and strict source/cutoff rules establish deterministic append-only public DATA.
6. Durable manager, evidence and request owners preserve the actions and bytes used in those arguments through worker cancellation and termination.

The network abstraction includes delivery and processing of complete validation dependencies. The later text explicitly distinguishes this abstraction from an arbitrary CPU/storage/native-import deadline. I retain that distinction throughout this review.

## 2. Original regional timer and lock argument

Location: main source, lines 791–1050 and 1127–1380; Part I, lines 734–832.

> “A correct replica casts at most one fresh vote per $(\beta,r,g,t,j)$ and one ANCHOR vote per $(\beta,r,g)$.”

Full signed context, immutable session membership, fixed anchor/quota and DATA/DRAIN parent rules preserve the identity needed by Part I. Fresh proposals receive timers only after actual full validation and the first vote. A duplicate or changed QC encoding cannot restart a timer. QCs and ANCHOR votes themselves start no timer. Ordinary term advancement does not cancel a previously established timer.

The baseline one-generation argument remains applicable: different generation openings yield the original signed NEWVIEW conflict; a first divergent fresh position under one opening yields signed proposal equivocation. The original rank order and lower-ranked parent descent are unchanged. Added input sidecars neither create QC rank nor alter the original signed proposal identity. Thus the new validity predicate cannot create a second independently authorized chain at one fresh position.

The retirement extension correctly does not assume the baseline universal protecting-QC conclusion without checking the missing-vote cases; those cases are addressed separately below.

## 3. Durable OC state and selected-body-only validation

Location: `sections/rotation_v3_11_input_selection.tex`, lines 20–60 and 125–189.

> “The common predicate ... checks the exact $K$, membership, distinctness, signatures, weight, minimum of the supplied keys and the selected body's canonical key and eligibility at $A$.”

The key binds the chain/protocol/type, physical region, slot, immutable initial AUTH hash, original anchor identity/hash/positions and native-validation domain. Omitting the target beta allows intentional reuse of the same prepared input across equal-context retries. It does not permit reuse at a different anchor/authority/domain, and it supplies no fresh-session entry permission: fresh REPORT context and the manager activation gate bind the actual pair separately.

`CheckOC` requires only the minimum selected body's canonical authenticated bytes and eligibility. A Byzantine nonselected signer may supply a key without supplying its body to an OC recipient; this does not invalidate an otherwise valid OC or block its relay. Conversely, an invented minimum key without its required matching eligible selected body fails the common predicate. The local collector's stronger rule of checking complete contributions before forming its own certificate does not silently become an extra requirement on received certificates.

Because weight exceeds $F$, any valid OC contains a correct offer. The correct offer is generated only after a completed fixed-cut job at the verified exact anchor. This gives causal provenance, not a public proof of a minimum over an unknown mempool. The proof uses the minimum of supplied keys, not agreement between private queues or signer-set intersection.

## 4. First-possession relay under selective delivery and cancellation

Location: `sections/rotation_v3_11_input_selection.tex`, lines 152–217; main source, lines 1631–1636 and 1813–1847; `sections/rotation_v3_11_retirement.tex`, lines 122–179.

> “On first validation of any complete OC for $K$, a correct member durably stores that exact certificate and records an outstanding multicast to every target member.”

The store and outgoing duty precede preparation cancellation and Ready notification. One `OCRelay[K]` retains the selected copy and dispatch cursor; duplicates resume unfinished work. Later valid OC variants do not require a second first-possession multicast, but remain admissible for later NEWVIEW/proposal validation. This avoids both a silent cancellation path and a private-certificate veto.

Selective-delivery case split:

- If a correct target ever possesses a valid OC, its persistent node-level handler retains and actually sends a complete certificate to all target members. Closed, stale handoff, cancellation and Terminated do not suppress this relay. Required dependencies remain owned. Ready need never occur for this branch of the argument.
- If no correct target possesses one, none can cancel its own finite selection on that basis. Correct members finish and dispatch their complete offers; correct weight alone exceeds $F$, so a correct collector can form one. The resulting OC takes the same first-possession path.

The proof does not need a correct node to identify an “all correct replies” event. It needs the specified continuing correct-node duties and admitted communication/processing scope. A finite fragmented transfer is not silently treated as a single $\Delta$ message.

The FinishClose transfer explicitly includes the certificate, relay cursor, receive route and unfinished multicasts. A worker may not replace them with an unresolved retrieval promise. Consequently the proof's outgoing dependency survives its worker, rather than depending on a future successful Ready callback.

## 5. First own-session DATA across every relevant history

Location: main source, lines 718–775 and 968–987; `sections/rotation_v3_11_requests.tex`, lines 44–165.

> “A speculative first DATA discharges this obligation only on its own descendants. Recovery returning to $A$ reenforces it with a matching opening certificate.”

The discriminator is the exact validated parent ancestry, not a local commit flag, current $G$, generation number or previously attempted first proposal. This covers:

- Initial DATA: the own-session list is empty, so a nonempty OC's selected body must execute first.
- Recovery returning to the fixed opening: the list is again empty and the opening requirement applies again, using a matching cached or supplied OC.
- Recovery retaining an own-session descendant: validation includes the earlier first-DATA witness; a local assertion of discharge cannot replace it.
- Ordinary later terms and positions: the parent chain already contains the first own-session DATA and retains its full witness.
- ANCHOR-only histories: they contain no own-session DATA and cannot discharge the obligation.
- DRAIN and repeated DRAIN recovery: DRAIN is allowed only at the fixed terminal DATA height; the positive quota $\TB_r\ge1$ means terminal ancestry contains own-session DATA. DRAIN preserves that ancestry, carries no requests and cannot substitute for the missing first DATA.
- Own-session receipts and Completion: full regional validation still checks the applicable ancestry and augmented witnesses before a receipt-driven commit.

Historical validity uses the actual historical parent and supplied common witness; current mempool contents and a different local attached OC cannot veto it. Different witness encodings for the same block remain variants of the same proposal identity. A different selected request changes the DATA block and is governed by the unchanged proposal/equivocation rules.

## 6. REPORT binding and original ANCHOR/Quit independence

Location: main source, lines 1093–1125 and 1305–1380; `sections/rotation_v3_11_requests.tex`, lines 65–165 and 208–248.

> “The advertisement binds the initializer's exact choice during forwarding.”

The report context includes the chain, session, region, configuration, authority, view, generation opening and exact effective parent. At an opening parent, the initializer's signature binds the exact canonical OC digest and selected key; reports acknowledge that same advertised choice. A relay cannot replace the certificate while preserving the signed choice. At a descendant parent, reports acknowledge the validated inherited first-DATA witness. Correct members sign at most once per context and never produce a historical signature merely because a later validator needs one.

> “Check the original NEWVIEW and ANCHOR prerequisites independently of the new input sidecars; a malformed sidecar cannot suppress original signed evidence.”

The independent path is explicit in the core and REPORT rules. An invalid or absent new sidecar may block REPORT or an augmented DATA vote, but cannot suppress the original ANCHOR action, NEWVIEW forwarding or admissible equivocation/Quit inspection. New advertisements and reports create no extra Quit type or QC rank. Delayed complete sidecars may complete REPORT only while its original fresh-action gate remains open; no entry/progress timer is reset.

Under the stated admitted-handler scope, the timing arithmetic is consistent: ordinary-term common ancestry by $e+2\Delta$, reports by $e+3\Delta$, votes by $e+4\Delta$; term-zero NEWVIEW by $e+3\Delta$, independent ANCHOR votes/reports by $e+4\Delta$, both inputs at the leader by $e+5\Delta$, first votes by $e+6\Delta$. At equality the message-first rule applies. The report handler performs no post-Enter private-pool scan. This check verifies the conditional arithmetic, not an unconditional resource bound for all native implementations.

## 7. Ready, callback isolation and resource limits

Location: main source, lines 1745–1847 and 1960–2133; `sections/rotation_v3_11_ready_bundle.tex`, lines 66–184 and 186–267; `sections/rotation_v3_11_source_step.tex`, lines 173–201.

> “The durable entry flag, fixed state attachment and entry time are recorded atomically before signatures are dispatched.”

A complete ReadyBundle supplies an actually executable state object, its exact-anchor evidence and a valid matching OC. The recipient does not need to finish an independent selection job, fetch the signer's hydration history or agree with its private OC attachment. Historical handoff verification is separate from current activation, and current activation checks Closed, fixed handoff, selected region, prior entry and tombstone. A stale callback can contribute cached material but cannot overwrite the fixed anchor or run another initial Enter.

The pre-Ready STATE_CANDIDATE path supplies state independently of OC/Ready formation and itself grants no votes, entry, lock or timer. It does not smuggle installation or source work into a post-Enter callback.

The $e+\Delta$ entry-skew inference is expressly conditional on complete validation, direct adoption and Enter belonging to the original modeled message handler. The text gives the residual-delay decomposition $e_v\le e+\Delta+u_v$ when native import remains. It also lists the actual OC/advertisement/REPORT bytes, signature checks and required packet/load inequalities. The indicator notation in the OC byte expression correctly charges nonempty key descriptors and the selected body only. These provisions delimit, rather than prove, deployment capacity. I do not treat unspecified capacity/benchmarks as either evidence of implementation soundness or a new formal task requirement.

## 8. Closure observability, protecting QCs and retirement drain

Location: `sections/rotation_v3_11_retirement.tex`, lines 24–242 and 248–406.

> “Closed prohibits every old-version ... new commit timer ... term entry and generation entry.”

Closed is persistent and branch-wide. It gates fresh actions even when the old worker never existed, was refused entry, has terminated or is reached by a delayed callback. An unknown snapshot postpones the complete per-session close event, not the gate. The first complete old-session packet supplies the historical context and triggers the closure multicast before refusal or duplicate/tombstone return. Thus a closed/unentered member cannot silently suppress a vote while leaving the proof's earliest complete-close time falsely late.

For first complete-close time $c$, the actual snapshot-bearing relay closes correct members by $c+\Delta$. The last allowed timer expires by $c+3\Delta$, and its generated FINAL messages arrive by $c+4\Delta$. Each local $c_v+4\Delta$ is conservative. FinishClose additionally checks that timers are settled and all storage/dispatch responsibilities have transferred; elapsed grace alone cannot terminate a live obligation.

For a direct timer started at $s$ that survives to $s+2\Delta$:

- A correct Quit or Completion at or before $s+\Delta$ would reach the committer by expiry and cancel the timer. Complete terminal-receipt multicast is essential here and is explicitly required before Completion.
- In the early case $s+\Delta<c$, closure observability excludes a missing vote caused by Closed. Complete proposal/admission dependencies and original recovery waits make the proposal processable. With no early Quit/Completion, every correct member votes by $s+\Delta$, so the protecting QC is known by expiry before every later live recovery lock snapshot.
- In the late case $c\le s+\Delta$, every correct entry gate closes by $c+\Delta\le s+2\Delta$, while an entry into the next generation would have to be strictly after $s+2\Delta$. No later-generation QC can exploit a missing protecting QC because it needs a correct signer and no such entry can occur.

Same-generation incompatible histories expose the unchanged original equivocation proof before a surviving timer expires. In the early case the ranked-QC descent therefore has its real prerequisites; in the late case all later correct recovery participation is blocked. DRAIN participates in this argument with its own physical position and parent justification. A canceled DRAIN timer does not acquire a fictitious direct commitment merely because Completion committed terminal DATA.

The example in which one late correct voter commits without enough other votes to form a QC or receipt is compatible with the theorem's exact scope. The theorem does not promise a protecting QC for that case, old-session progress or a complete publication result. This is an appropriate restriction, not a safety counterexample.

## 9. Actual FINAL provenance and public-kernel induction

Location: main source, lines 862–935 and 1190–1225; `sections/rotation_v3_10_receipt_kernel.tex`, lines 15–176.

> “A commitment triggered by a receipt, including Completion, emits no new FINAL.”

Raw FINAL verification is genuinely separated from the definition of a fully valid regional receipt. Its threshold identifies a correct signer without assuming the admission property to be proved. The signer's actual FINAL action traces to a prior timer-origin commit, directly or through ancestry. The corresponding Vote had full own-session original and augmented validation, including the OC and required advertisement/REPORT witness. This establishes the existence of a genuine historical witness; it does not authenticate an alternate publisher-supplied recovery path.

Inherited-anchor receipt dependencies already existed before the newer Vote. Induction over execution events from the trusted checkpoint therefore justifies their historical kernel conclusions without a self-supporting receipt cycle, including an unaccepted REGULAR terminal anchor. Exact body hashes and parent links identify the data on that earlier checked history. Same-session coverage includes required physical DRAIN links; another session's receipt cannot cover missing earlier-session DATA.

Receipt-driven Completion cannot generate the first correct FINAL or escape that provenance chain. The regional retirement proof uses the same causal origin to show that receipt-based commits only add previously protected DATA prefixes.

`CheckFINAL` establishes intrinsic input validity only. It does not select a global branch, authenticate an unbound state object or supply complete public controls. Those checks remain independent.

## 10. Entire original publication value and source retention

Location: `sections/rotation_v3_11_requests.tex`, lines 167–204; `sections/rotation_v3_10_exports.tex`, lines 201–238; `sections/rotation_v3_8_carrier.tex`, lines 1–181; `sections/rotation_v3_11_source_step.tex`, lines 2–171.

> “Write $R_0$ for the original complete regional result ... It is not merely the public kernel.”

Public erasure removes only the new separately framed regional selection sidecars. Original DATA, signed proposal/NEWVIEW, votes, QCs, FINAL, ancestry, boundaries, authorities and original source denotations remain. The selected request consumes ordinary DATA capacity. The proof does not extend $T_{\rm pub}$ to bare requests, Ready, partial FINALs or a weaker kernel-only input.

The logical-carrier lemma preserves actual structured objects and exact reference denotations without treating a private cache or unresolved name as public payload. Optional publisher-named recovery/native witnesses do not impose new public fetch dependencies. Missing required public objects remain missing; normalization cannot fabricate them.

Raw namespaces, rejected-record objects, historical source mappings and actual regional witnesses are retained independently of interpretation outcome, beta and worker lifetime. Source intake records import work without performing a whole-pool scan. The deferred import/checking journals preserve their exact bytes and fixed read contexts and do not gate P6, Ready, report emission or source relay. This separates later request recovery from the safety of the currently accepted prefix.

## 11. Current append authority, per-DATA transactions and strict cutoff

Location: `sections/rotation_v3_10_exports.tex`, lines 54–199; `sections/rotation_v3_6_control.tex`, lines 222–325; main source, lines 2240–2302 and 2541–2602.

> “if Force created its version in $J_{q_f}$, then $q>q_f$.”

The public interpreter independently checks the current open version, authentic incoming boundary and export duty, exact extension of current $G$, next DATA height, same-session coverage/quota, and strictly preceding Espresso selection references. An old receipt's intrinsic validity cannot override any of these gates.

The canonical prelude precedes payload. Independent controls, matching MSIG with old-beta check, and AUTO have the stated order. A Force retains exactly pre-payload $G$, increments beta once, and rejects old-version appends and new-version DATA in that triggering block. Equal timestamps do not erase block/record order. OFF or CLEAR do not rewrite an already fixed historical regular handoff or roll back accepted DATA.

`CommitNextDATA` exposes each DATA and accounting durably before later DATA or successor resolution. Missing successor authority pauses the subsequent canonical resolution; it cannot roll back or withhold the earlier terminal transaction. Private caches affect retrieval, not the common public object basis. A completed past record is not revisited to append DATA retroactively, while its retained objects may serve as evidence for a later authorized record.

These deterministic operations support the stated induction over common source subpositions. Observers at different subpositions hold prefixes of the same append-only DATA sequence. Session-manager phases, late receipts, old-program retirement and speculative preparation do not add another mutation path for $G$.

## 12. Persistent original-owner lifetime and limits of this verdict

Location: `sections/rotation_v3_10_ownership.tex`, lines 58–108 and 419–444; `sections/rotation_v3_11_retirement.tex`, lines 122–179.

> “the last correct original owner of a still-pending request cannot leave merely because its AUTH is old, the successor is Ready, or a HYDRATED certificate exists.”

Pending bytes, immutable cuts, raw/canonical handles, authentication context, transfer state and outgoing duties belong to a node-level service. Inclusion in $G$ is the pending-removal condition; speculative commitment, acknowledgement and handoff are insufficient. A physical exit requires no remaining duty or continuation of the same authenticated logical owner with actual state and receive route in the same physical region. A new custodian's promise or unresolved archive task does not satisfy it.

These are explicit correctness/lifetime rules, so cancellation and arbitrary AUTH changes do not implicitly delete the owners required by OC and request evidence. The human endpoint clarification makes the specified old/new correct-member communication relevant; it does not keep a failed machine correct or provide an unmentioned cross-region path.

This review checks safety and the OC-formation/retention dependencies needed for that safety chain. It does not independently certify the full global-service, recurring-region or end-to-end request-inclusion theorem. No executable model checker, deployment test or native adapter benchmark was supplied or run. No compilation result is used as proof evidence.
