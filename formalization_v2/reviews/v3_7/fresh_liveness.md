# Fresh independent v3.7 liveness verification

## Summary

**Final Verdict:** The checked abstract proof chain for continuing global DATA progress and repeated accepted regional service is valid under its stated prepared-regional complete-message interface and frozen regional/source assumptions; I found no Critical Error or Justification Gap in that chain. This does **not** complete request liveness: the stated request-starvation execution survives the revision, so the original request-inclusion objective remains unresolved.

**List of Findings:** No mathematical defect was identified in the reviewed liveness chain. The following are material scope limits, not silently supplied premises or proved implementation properties:

- **Request liveness is open.** `sections/rotation_v3_7_liveness.tex:442–478` gives a compatible execution with unbounded accepted empty DATA and repeated regional credits while a pending request is cut off every time. It is not a counterexample to the narrower propositions actually stated at lines 291–299 and 387–394.
- **The two source-handler delays are conditional on the abstract complete-message interface.** `sections/rotation_v3_7_source_step.tex:213–252` and `sections/rotation_v3_5_ready_bundle.tex:107–146` do not prove that arbitrary disk import, native initialization, offline catch-up, or an external observer's application processing finishes within `Delta_i`. The quantitative proof expressly uses the maintained, hydrated regional input path; an implementation must meet that interface.
- **Logical carrier preservation is not arbitrary binary decoding or namespace discovery.** `sections/rotation_v3_7_carrier.tex:146–177` limits the result to the original logical complete-result transport interface. A concrete legacy codec, private-only resolver, or a broader implementation meaning of “already recorded” that discharges delivery without making the result discoverable to the covered reader interface needs a separate compatibility/access argument. None was assumed here.
- **One eligible region suffices only inside the regional model.** `sections/rotation_v3_7_liveness.tex:291–299,354–360` does not prove progress with just one physically healthy region while other selectable assigned snapshots permanently lack their required correct voting weight or communication service. All selected snapshots used by the finite response maximum remain subject to the stated regional premises. A permanently OFF region is outside the eligible-service claim.

## Frozen inputs and method

- Read the v3.7 scope, manifest, frozen main source, included fragments, and the relevant full v3.1/Part I dependencies. No other review or prior verdict was read.
- All **23** manifest file hashes match. The main source hash is `440d639a3f9a7a0a236836153880171db40c904a1862eef2444b8313c9d2b476`.
- The v3.7 Model section is byte-identical both to the v3.1 Model section and to `baseline/model_and_assumptions.tex`. This check establishes textual preservation only.
- This is read-only mathematical verification of the supplied abstract specification, except for this report. No manuscript repair, executable/model-checking validation, native-adapter measurement, or concrete Espresso service certification was performed.

All source line references below are relative to `snapshot/`. `main` denotes `sync_hotstuff_rr_part2_rotation_v3_7.tex`; other names denote the indicated `sections/` or `baseline/` file.

## Detailed Verification Log

### 1. Dependency map and separation of claims

The liveness dependency order is:

1. fixed-slot reuse and persistent preparation;
2. finite raw-byte hydration and ongoing retention;
3. raw FINAL soundness, plus separate mandatory public gates;
4. logical carrier preservation and lazy inventory;
5. finite regional response `R_i` after preparation;
6. immediate complete SOURCESTEP processing and first-result publication, giving `L`;
7. finite health interventions and immediate nonempty AUTO reassignment;
8. the adaptive timestamp contradiction;
9. owed-set/cyclic selection arguments for repeated regional service.

Request inclusion is a separate branch, and is expressly not derived from steps 8–9. Regional agreement and authentic historical boundary selection are dependencies of the liveness argument, not consequences of the arithmetic FINAL threshold alone (`rotation_v3_7_receipt_kernel.tex:128–138`).

The frozen progress text contains the old successful-acceptance/service-interval conditions (`main:518–545`). The new proof explicitly declines to assume the next required DATA is accepted in order to prove acceptance (`rotation_v3_7_liveness.tex:5–15`). Its concrete construction below supplies that step without importing a new acceptance-retry deadline.

### 2. Raw-byte storage, fixed hydration, and retention induction

> “An index without the bytes is not a retained object.”

`rotation_v3_7_source_step.tex:12–29` distinguishes raw Espresso namespaces, object bytes/occurrences, L1 source evidence, regional witnesses, exact executable anchors, and replay state. Lines 33–49 require insertion before advancing availability cursors or releasing inputs, retain rejected/malformed records, and prevent beta replacement, worker exit, checkpoint compaction, or growth of `G` from silently deleting source data.

> “Neither a new beta nor a later source height restarts the fixed old task or changes its target to a moving cut.”

The hydration target at lines 54–71 is a fixed finite source interval, including legally referenceable rejected records back to the source origin. Newly arriving namespaces are retained separately. Eventual retrieval of each member of this fixed finite set gives a finite completion time; the argument does not replace eventual retrieval by a uniform archival bound. The anchor and finite authority prefix are separately prepared.

The induction in lines 191–211 is valid under the online/history-following visibility premise: the base covers the fixed cut; the step retains every newly acquired namespace before any rejection; transitions preserve all legally referenceable bytes. For a block finalized at `a`, every predecessor finalized no later than `a`, so its source bytes are available by `a+T_E`. This is a byte-availability result, not an arbitrary backlog-processing or L1 authority deadline. The storage requirement is unbounded in general, as lines 73–81 correctly acknowledge.

### 3. Raw FINAL kernel and remaining mandatory public inputs

> “This step uses the raw threshold, not the definition of a valid regional receipt.”

`rotation_v3_7_receipt_kernel.tex:15–31,51–56` verifies exact DATA and signatures using the authenticated fixed incoming context and `AUTH_{r,0}`. Weight greater than `F_i`, the fixed-snapshot lifetime compromise bound, and unforgeability yield an actual correct FINAL action.

The next inference is noncircular. A correct FINAL originates only in the timer Commit rule, which follows full regional Vote validation (`main:941–955,1133–1153`). Receipt-triggered commitments and Completion emit no new FINAL (`main:884–891`; kernel lines 73–85). Historical kernel use for inherited anchors is justified by earlier signature/Vote events from the trusted initial checkpoint, including speculative REGULAR anchors; it does not assume those anchors have already entered `G`.

The result establishes existence of a genuine validated regional witness for the exact hash-bound input. It does not validate a publisher's alternative admission witness or an unbound native state/output assertion. Thus arbitrary optional `AUTH_{r,g}`, regional `rho` proof, or native-sidecar reads can be omitted from the public kernel without weakening full regional admission.

The separation is essential and is retained. `CheckFINAL` still needs actual DATA/FINAL/covering-path bytes and authenticated incoming context (`kernel:88–115`). Current branch permission, exact extension of `G`, quota, authentic boundary/initial authority, strict Espresso-reference precedence, and Force cutoff remain independent checks (`kernel:128–149`; `rotation_v3_7_exports.tex:78–100`). Complete applicable controls also remain mandatory. An exact executable anchor and `C_pub` execution/DA correspondence are not supplied by this kernel (`kernel:151–167`).

### 4. Logical preservation versus actual lazy inventory

> “This graph definition is not an instruction to fetch every referenced target before processing the record.”

`rotation_v3_7_carrier.tex:56–102` preserves original signed bytes, field/list positions, occurrence paths, shared-object edges and baseline source denotations. The actual traversal visits supplied inline and retained authenticated objects once, retaining unresolved optional edges without fetching their targets. Inline children remain visible even inside an invalid enclosing proof. Invalid outer hints or siblings do not erase independently represented kernels.

The preservation proof at lines 104–138 is a graph-denotation proof over the finite complete-result witness. The reference case preserves what the old reference denoted; it does not infer a read deadline from an unresolved edge. The actual DATA/FINAL projection is available through the unchanged logical complete-result transport interface and retained Espresso objects. L1 authority fields and native sidecars are not secretly DATA/FINAL carriers under that specified logical object model (lines 81–93).

For preceding malicious records, the required/optional distinction is consistent: unrelated pair/domain/hash/signature roots are rejected using the already fixed current context; genuinely absent required objects make only that kernel incomplete; independent complete neighbors are still inspected; a genuine missing source namespace/control/current-boundary input pauses the appropriate operation (`source_step:83–131`; `exports:45–70,191–199`). An index or private cache assertion cannot fabricate a public object, and a false outer base/version/interval cannot create a permanent duplicate veto.

### 5. Fixed slot, finite authority prefix, and the regional response bound

> “Low-generation faulty assignments can recur on a new beta, but the number of generations before `g_i^*` is fixed.”

Under a last global tip `G*`, at most one remaining Force closes a duty with positive accepted output; all subsequent zero-output retries reuse `r*` and its immutable schedules (`main:1791–1823`; `liveness:44–59`). Persistent native preparation and sequential `AuthCache[r,g]` work survive consumer cancellation and beta changes (`main:1746–1765`). Old-beta Quit proofs do not admit the new engine.

The finite-roster/exclusion argument in `liveness:75–105` is valid: take the maximum exclusion generation over identities ever exposed in this fixed authority namespace, then choose a correct initializer at a finite `g_i^*` by inherited initializer fairness. An admissible high-generation equivocation would contradict that identity's exclusion tail. Delayed revelation of a lower-generation or other-pair proof does not freeze the current generation. All authorities through `g_i^*`, exact anchor preparation and fixed-cut hydration eventually complete; only a finite maximum is taken.

The response calculation in `liveness:107–154` is conservative. Part I supplies recovery entry skew at most `Delta`, terminal-QC term-entry skew at most `2 Delta`, and the protected correct-initializer window (`baseline/sync_hotstuff_rr_part1_core.tex:600–657,681–709,870–894`). The revised initial ReadyBundle supplies the initial `Delta` skew under its stated handler interface.

For an ordinary term, by `u+(2K+6)Delta` either advancement/Quit/Completion has occurred or its final progress checks have run. A correct terminal vote spreads and yields the QC within another `2 Delta`; at the endpoint the additional commit/FINAL allowance is sufficient. Without any correct terminal vote there can be no correct QC advancement, so the matching BLAME checks force a Quit. The unknown-opening first check is explicitly scheduled (`main:1211–1228`). Hence `(2K+12)Delta` bounds the stated term outcomes.

A DATA generation has at most `Q` terms and a DRAIN generation one. Therefore `D_i=(Q+1)(2K+12)Delta_i` safely bounds each failed-prefix generation. Cached-authority recovery costs at most another `2 Delta_i` from first correct Quit. At `g_i^*`, the correct initializer's first proposal is sent by `e+5 Delta`, all correct members commit by `e+8 Delta`, and their FINALs yield a complete result by `e+9 Delta`. DRAIN commits terminal DATA from this same session, and `Q>=1` makes that positive DATA beyond the retry anchor. The displayed `R_i=(g_i^*+1)D_i+2g_i^*Delta_i+9Delta_i` is an overestimate, not an unjustified speedup.

### 6. Complete SOURCESTEP operation and first public DATA

> “On acquiring a complete raw source input, each correct member retains and sends it in that availability event, before waiting for its own P6 result.”

`source_step:133–159` avoids a sender waiting for successful replay, all optional sidecar fetches, native restoration, or a subsequent empty block before dispatching the raw source input. A source relay that still lacks genuine mandatory boundary evidence is explicitly incomplete; later exact-dependency arrival resumes the saved operation. Sender-only cache possession does not excuse omission of bytes required by the receiver.

For the prepared cohort, the source handler uses resident exact bytes, fixed current authority/anchor, advance-announced controls and the new bounded Espresso input. Its receive operation processes a contiguous dependency-complete prefix and performs guarded state attachment/Enter in that same modeled message event (`source_step:161–178,213–252`). The `Delta_i` charge therefore applies only to this specified maintained input path. No receiver-native import or archive task is silently put inside it; those would invalidate the timing interpretation and are explicitly excluded.

The five-case dependency check in `liveness:185–228` covers the prelude, inventory, public kernel, exact append permission and regional initial/recovery handling. In a fixed-`G*` tail before the first positive DATA, current canonical boundaries target the fixed slot; attacker-selected auxiliary admission paths cannot force an arbitrary new recovery authority read. Earlier current-pair candidate records obey the same rules, so a malformed predecessor cannot introduce an optional mandatory wait.

The newly formed complete regional result is the original publication-service input (`liveness:239–246`; `exports:201–210`), not a newly invented kernel-only input. If the result was not recorded at its first correct offer time `s`, visibility follows by `s+T_pub+T_E`; if it was already recorded, the existing record is available by `s+T_E`. Carrier preservation handles wrong outer hints, nested ancestry placement and bad unrelated evidence; no acceptance retry is assumed (`liveness:248–256`).

A genuine correct FINAL cannot exist before its signer's validated reference at or after the opening, so the containing record is later than that finalized reference and later than a forced opening (`liveness:258–265`). With no intervening `G` growth or closure, the first DATA after `G*` therefore passes the independent current-duty gates. A Force in the same containing block supplies the closure alternative because the prelude runs first.

Terminal quota-one DATA is durably committed/exposed before successor `AUTH_{r+1,0}` acquisition or `PrepareOwed` (`exports:107–162`; `main:2195–2202`). A later boundary wait cannot withhold that first growth. This addresses both a standalone terminal record and the first segment of a multi-session record.

The total `L=max_i(2T_E+2Delta_i+R_i+T_pub)` correctly charges the opening source/relay, regional response, first-result transport, and result source/relay. ReadyBundle entry skew is already within `R_i`; it is not omitted or charged twice. The final source allowance concerns witnessing the DATA commit before any later unresolved operation, not finishing an entire multi-session record.

### 7. Independent health-control branch

The finite-control result in `rotation_v3_5_health.tex:120–143` follows from the monotone accepted-tip cursor and one nonrefundable credit of each type per accepted tip. Only `h*-h0+1` new keys remain in a fixed-prefix tail. Previously reserved operations, including not-yet-released signatures, are counted separately. Stale targets and exact retries do not create further effects.

The restoration branch at lines 145–169 is also valid. Each OFF requires a previously finalized independent ON at a finite later threshold; OFF/ON in the same first-crossing block are ordered correctly. Only one nonoverlapping lease is allowed per region per accepted-service count. Thus an indefinitely unserved managed region eventually has no further health OFFs even if other regions keep growing `G`. Empty attached MSIG configuration prevents an MSIG OFF from overwriting the scheduled ON. Administrative removals and unreconciled legacy OFFs are correctly outside this restoration claim (lines 93–118).

These arguments do not themselves establish DATA or request progress, and do not require an accurate bounded-time health oracle.

### 8. Adaptive time and the last eligible region

After the finite health-control prefix and finite preparation, the fixed slot has a finite region/parameter set. Existing cooldowns expire at finite source positions because Espresso timestamps grow. The conditional AUTO rule cools its source only if another currently eligible owed candidate exists (`rotation_v3_6_control.tex:143–175`). Consequently it cannot create a new all-candidates-cooled gap: either another eligible owed candidate remains, or the source renews; if the source is outside the owed set, an eligible owed candidate already exists. This does not clear old cooldowns or override OFF.

With no `G` growth, each retry has zero accepted DATA and stays in startup. Each effective AUTO both closes one retry and immediately opens the next after the above finite prefix. The response proposition implies `a_(n+1)-a_n<=L`; otherwise that attempt would produce `G` growth within `L`. Base-window settlement gives `theta_(n+1)-theta_n >= (1+beta_0+n)W_min`, with `W_min>0`. A large timestamp jump produces only one Force and starts the next duty at the realized timestamp (`control:273–280`).

Thus `theta_0+W_min sum_(n<N)(1+beta_0+n) <= theta_N <= a_0+NL+epsilon_E` (`liveness:322–350`). Quadratic growth contradicts the linear right-hand side. This uses only the frozen upper clock-lead bound. No lower-lag bound or bounded empty-block cadence was introduced; finite initial lag is absorbed in `a_0-theta_0`.

### 9. Owed service: both independent cases

For a permanently eligible owed region `C`, its debt cannot be pruned. Other members of the finite unfinished round can be discharged or pruned only once; later ONs do not join that round (`liveness:364–399`; `control:101–164`).

If infinitely many effective Forces occur, the first accepted DATA of every productive Force-created version must come from its initially selected owed region. Strict version cutoff and contiguous next-height acceptance prevent bypassing that forced session. Each productive version therefore consumes a previously remaining debt or a debt pruned after selection. There can be only finitely many such consumptions while `C` remains unserved. Infinitely many nonproductive versions after the last productive one would contradict continuing global progress. This branch is valid (`liveness:401–412`).

If only finitely many Forces occur, unbounded DATA and finite session quotas imply infinitely many accepted REGULAR sessions. The stricter regular floor makes their selection heights strictly increase. After passing `C`'s permanent eligibility position and all earlier cooldowns, cyclic `NextLive` cannot skip eligible `C` indefinitely, regardless of other regions' eligibility changes. Its assigned session must contribute before contiguous acceptance advances past it (`liveness:414–422`). This argument correctly allows REGULAR selections outside the owed set.

The same finite-removal argument ends an earlier unfinished round; otherwise an unpruned residual member is reached by the infinite-Force or final-version cyclic branch. Further boundaries open the next round and include a newly permanent-ON `C`. Repeating after any contribution proves repeated accepted regional service (`liveness:424–434`). Empty DATA may discharge debt, so this result does not imply request inclusion.

### 10. Request starvation remains a valid separate branch

The counterexample in `liveness:442–478` respects the absence of a fresh-slot authority deadline. A Byzantine initializer contributes one valid empty DATA, then stops. Delaying that slot's recovery authority until shortly before the final failed running-window crossing lets a correct initializer actually enter and propose the oldest pending request, but its complete receipt arrives after Force.

The numeric example has DATA votes at `d+4,d+5`, timers at `d+6,d+7`, and a crossing at `d+5.5`. Force need not cancel either timer; the receipt can still form and meet the original publication bounds afterward. The already accepted empty DATA makes the Force legitimately advance to a fresh slot, so persistent preparation for a zero-output retry does not remove the next new authority wait. No equivocation or schedule-unfairness attack is needed. The request stays pending until global inclusion (`main:2222–2240`), while each late old-version result is cut off.

Accordingly, the abstract DATA-progress and repeated-service results do not establish the complete user-facing request-liveness objective. This is an explicit remaining design obligation, not an implementation timing caveat.
