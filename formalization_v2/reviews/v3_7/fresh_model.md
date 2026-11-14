# Summary

**Final Verdict:** Within this independent semantic-assumption, transport and objective audit, the public FINAL kernel and logical-carrier arguments are valid under their stated original fault, validation and source premises; I found no Critical Error or Justification Gap in those arguments, but the full request-inclusion objective remains unsatisfied by an admitted in-model execution.

**List of Findings:** No new proof defect in the assigned kernel/carrier/prepared-dependency scope. The material limitation is the expressly acknowledged request-starvation execution at `sections/rotation_v3_7_liveness.tex:442–478`. This is an **objective shortfall with an in-model counterexample**, not a false claim hidden inside a theorem that asserts request liveness. Concrete encoder compatibility, native adapter performance and arbitrary-observer processing are **unproved implementation obligations outside the stated theorem**, not counterexamples to its abstract conditional claim.

All line references below are relative to `snapshot/` unless marked otherwise. I read the complete current main source, all nine active section fragments, the figure sources, the complete frozen v3.1 main source and Part I core source. I did not read other review reports or prior verdicts. All 23 entries in the parent `manifest.json` matched their SHA-256 hashes. The main hash is `440d639a3f9a7a0a236836153880171db40c904a1862eef2444b8313c9d2b476`. The current Model section is byte-identical to both the corresponding v3.1 section and `baseline/model_and_assumptions.tex`; the semantic conclusions below rely on the actual dependencies, not that equality alone. No protocol source was changed.

# Detailed Verification Log

## 1. Frozen premises and the original external obligation

> “A verifier outside the region checks condition (iii) from evidence.” — `baseline/sync_hotstuff_rr_part2_rotation_v3_1.tex:823–825`.

The baseline explicitly required external evidence-based block validation, including remote admission witnesses, authority, parent QCs, input ancestry and application validity (`baseline/...v3_1.tex:803–825,881–896,1363–1375`). Therefore v3.7 is an actual public-checker change. It cannot be certified merely by saying that the Model section is unchanged.

The relevant inherited premises are stronger than bare signature authenticity: every weight->F certificate contains a correct signer; the bound includes all snapshot keys compromised while those certificates remain acceptable; signatures are unforgeable and hashes binding (`...v3_7.tex:360–378`; baseline `...v3_1.tex:328–346`). The model also assumes sound regional and application-validity interfaces and separately sound execution/DA correspondence (`...v3_7.tex:510–516`). Correct regional Vote validates the complete current-session evidence before starting a timer (`...v3_7.tex:941–955,1123–1153`). These are the premises needed for the new implication.

**Classification: valid algorithmic refinement under the stated model.** The change replaces one algorithm for establishing intrinsic validity with a sufficient cryptographic certificate plus an operational soundness proof. It does not establish the baseline's stronger, different statement that any publisher-selected auxiliary admission witness has itself been checked successfully.

## 2. RawFINAL does not assume the receipt validity it must prove

> “This is a cryptographic and type check only: it does not include … (iii).” — `sections/rotation_v3_7_receipt_kernel.tex:24–31`.

The raw predicate binds the supplied DATA bytes/hash/height, complete signed domain, assigned region, producing pair and fixed configuration under the authentic incoming context. The context comes from the authenticated boundary and immutable initial authority, not an outer sender hint (`receipt_kernel:15–31`). An attacker cannot choose another snapshot to lower the weight threshold.

> “The lifetime compromise condition and signature unforgeability make its signature an actual correct FINAL action.” — `receipt_kernel:52–56`.

This inference is sound: weight strictly greater than F contains a key outside the entire relevant compromise budget. That signature therefore originated in a correct protocol action. The argument does not reason from “valid receipt” to “valid receipt.”

> “A commitment triggered by a receipt, including Completion, emits no new FINAL.” — `receipt_kernel:73–75`.

The operational premise is present in the main rules (`...v3_7.tex:886–891`) and was already present in v3.1 (`baseline/...v3_1.tex:834–839`). FINAL originates at Commit's surviving DATA/DRAIN timer event (`...v3_7.tex:1147–1153`). The timer began only after complete current-session validation. For an inherited anchor validated through an earlier public kernel, the signatures exist before the new Vote event. Induction on these causally earlier actual signing events reaches the trusted initial checkpoint (`receipt_kernel:58–85`). An unaccepted REGULAR anchor causes no circular dependence on global acceptance.

**Classification: verified.** The result is existence of a genuine historical validation witness for the exact hash-bound DATA. The correct signer need not have checked a later publisher's alternate PROPOSE/QC/recovery path. If more than F weight is compromised, if correct members omit full validation, or if the native validity interface is unsound, the premise has been removed; those cases are not in-model attacks on this lemma.

## 3. CheckFINAL and the independent gates remain distinct

> “No publisher-selected generation, term or admission path is an input.” — `receipt_kernel:111–115`.

The checker requires actual root DATA, its raw FINAL set, and exact same-session hash-linked bodies connecting the root to the candidate, including supporting DRAIN where relevant (`receipt_kernel:88–110`). It checks type, hashes, links, positions, endpoint, same-session coverage and quota. Hash binding identifies that exact supplied path with the correct signer's history. An unresolved hash is not payload; a later session's receipt cannot cover earlier-session DATA.

The caller still checks the authentic incoming boundary, current branch and export duty, exact extension of G, next height, quota, historical Espresso reference context with strict predecessor position, and q greater than the Force-creation block (`sections/rotation_v3_7_exports.tex:78–100`). The public control prefix is independently required (`receipt_kernel:128–138`). Neither FINAL nor a native TEE authenticates completeness of the observer's control view. The weight threshold establishes one correct signer, while regional safety supplies agreement; the text correctly does not use >F alone as a quorum-intersection argument.

**Stress cases:** A valid old-version kernel after Force, a true certificate under the wrong producing boundary, a current-pair block with the wrong parent of G, a future-height root without its required next path, and a same-block selection reference all fail the independent gates. A bad optional witness does not invalidate an independently complete genuine kernel. These cases are distinguished by the specified algorithm and do not expose a gap.

## 4. Native input validity does not swallow Ready, execution or DA

> “It does not authenticate an arbitrary unbound native context, supplied state object, claimed execution output, or later C_pub publication.” — `receipt_kernel:151–160`.

The certified proposition is about the input history actually checked by the honest signer under its authenticated anchor. Own-session Vote, recovery, receipt processing and Completion keep full validation (`...v3_7.tex:941–955`; `receipt_kernel:3–13,140–149`). Ready still requires the exact anchor's recovered and verified executable continuation, target authority and all entry dependencies (`...v3_7.tex:1983–2008`). Native state for another height or merely a serialized snapshot requiring further restoration cannot satisfy that predicate (`rotation_v3_5_ready_bundle.tex:25–58,99–105`).

Faithful later publication must still match the ordered G interval, native execution evidence and DA payload, including a legal continuation from a partial-EVM anchor (`...v3_7.tex:2244–2252`). This preserves the baseline distinction between input-prefix finality and state/withdrawal claims (`baseline/...v3_1.tex:1422–1427,1862–1870`).

**Classification: verified scope separation.** The kernel is not a proof for an unbound native sidecar. It covers a particular native claim only when that claim is hash-bound and checked by the assumed sound regional interface. No new native implementation or performance theorem follows.

## 5. Logical EXPORT normalization preserves evidence, not baseline acceptance traces

> “The signed objects retain their original bytes and hashes under normalization.” — `sections/rotation_v3_7_carrier.tex:12–15`.

The baseline EXPORT has chain/version/base/ordered-DATA/evidence fields, with receipts, ancestry, boundaries, authorities and native context (`baseline/...v3_1.tex:1338–1345`). Normalization operates on that structured logical value. It preserves object bytes, shared edges, occurrence paths and the original authenticated reference denotations; it does not reserialize signed objects (`carrier:56–66`). The finite witness induction is valid as a representational argument (`carrier:104–138`).

This is not acceptance-trace equivalence: v3.1 rejected an entire record if any acceptance condition failed, while v3.7 uses hints and commits per DATA (`baseline/...v3_1.tex:1350–1357`; `carrier:140–144`; `exports:107–123`). The text explicitly states this difference, so success of a bad-hint record is a protocol change rather than a purported consequence of the original all-or-nothing interpreter.

**Stress cases:** A correct result occurring only in supplied ancestry survives inventory. Wrong outer chain/version/base/interval hints cannot erase independently represented objects. An invalid sibling or enclosing proof cannot suppress valid inline children. Alternative proof choices remain distinct, so one invalid wrapper cannot establish a permanent duplicate marker (`carrier:17–25,68–79,95–102`; `exports:35–50`). An absent required DATA leaf or cyclic unresolved reference is not manufactured.

## 6. Reference denotations, lazy inventory, and the already-recorded exception

> “A reference retains its baseline denotation … It is an edge to an object, not the object's bytes.” — `carrier:27–35`.

Public completeness uses the current record's actual objects, permissible earlier authenticated source objects and the objects established by an authorized checkpoint. A private cache does not enlarge this common basis; a newer private checkpoint cannot change an old record's public completeness (`rotation_v3_7_source_step.tex:83–105`). Content-identity aliases do not acquire a new requirement for a redundant explicit occurrence index. Conversely, the dictionary must establish the actual object and a permissible occurrence. L1 objects remain authenticated through L1 and need not have appeared in Espresso first.

> “Preserve an unresolved reference as a typed edge; do not wait for its target merely to finish this traversal.” — `carrier:68–79`.

The algorithm inventories actual supplied objects incrementally and resolves the DATA/FINAL/covering-path/fixed-context projection. It does not first fetch a union of every optional sidecar target. A recovery authority denotes its authority fields, not a hidden DATA carrier; a native proof sidecar is not absent DATA/FINAL bytes (`carrier:81–93`). This addresses both direct optional lookup and an inventory-stage lookup disguised as preprocessing.

> “A result already recorded in Espresso satisfies this delivery guarantee; retries needed for acceptance into G are a separate task.” — `...v3_7.tex:408–415`, unchanged from baseline `...v3_1.tex:376–383`.

The public-response argument correctly handles an adversary's earlier complete carrying record, including wrong outer hints, rather than assuming a later corrected publication deadline (`liveness:248–256`). For a fresh forced retry, a correct FINAL signature arises after a finalized selection reference at or after the retry opening; immutable Espresso order places its carrying record strictly later (`liveness:258–265`). Thus this first result cannot already be trapped in a pre-opening record for the same new pair. A genuinely missing predecessor, payload or closed version is still not rescued retroactively (`exports:183–199`).

**Access boundary:** The argument requires the unchanged complete logical result to be supplied through the modeled public reader interface. This is supported by the inherited complete-result and full-record visibility premises (`...v3_7.tex:339–358,461–490`), not by defining “carrying” as success of Flat. A private-only resolver, arbitrary ciphertext, a deployed codec that hides required DATA behind an additional accessor, or an occurrence outside the specified public reader interface has no compatibility/discovery proof here (`carrier:146–177`). Treating such an occurrence as sufficient to discharge publication would require a separate argument. I do not import a bounded discovery guarantee for unrelated namespaces or a fresh decoder assumption. Conversely, an actually supplied logical result with a wrong chain hint remains covered and cannot be dismissed on that basis.

**Classification:** Logical preservation is verified; broader binary/access compatibility is expressly unclaimed. No particular legacy binary codec is specified in the baseline, so a hypothetical codec incompatibility is not a demonstrated mathematical counterexample.

## 7. Full-byte retention and fixed hydration do not create an archive bound

> “Raw Espresso source: Every acquired complete relevant namespace, including rejected or malformed records.” — `source_step:12–17`.

The store retains bytes, source identities, occurrence mappings, authentication evidence and cursors, not just accepted ancestry or hashes. Insertion precedes cursor advance and rejection; beta changes, G growth, checkpoint compaction and worker exit cannot delete referenceable content (`source_step:33–50`). This prevents a later alias into rejected public history from causing a new archive access in the prepared cohort.

> “Choose a fixed authenticated source cut q_0 … Retain every newly available namespace beyond q_0 while hydration runs.” — `source_step:52–71`.

A finite old interval with finite admitted payloads has a finite hydration completion time under eventual retrieval. Taking the maximum with finitely many authority and native-preparation completions yields an execution-dependent finite preparation time, not a uniform deadline. New acquisitions do not move the old hydration target. The argument does not hydrate every arbitrary future L1 sidecar, and it makes no bounded-storage claim (`source_step:73–81`).

The retained-source proposition is an induction over acquisitions plus the inherited T_E deadline for newly finalized relevant namespaces (`source_step:191–210`). It is not a deadline for L1 authorities, arbitrary public objects, application execution or returning-node catch-up. Loss of required bytes requires rehydration before prepared timing resumes (`source_step:44–50`). **Classification: verified with these explicit scopes.**

## 8. Mandatory dependency audit before the first positive DATA

> “No optional proof choice creates an uncharged external lookup.” — `liveness:185–228`.

The case split is complete for the stated fixed-G tail:

| Operation or dependency | Available reason | Remaining limitation |
| --- | --- | --- |
| Opening/prelude, zero-output retry target, pending selection | Prepared AUTH at the fixed slot; current replay state; retained E history; applicable controls available before crossing | Genuine missing current-boundary evidence pauses replay; it is not an empty set |
| New complete namespace and E evidence | Original T_E source guarantee | No arbitrary application-processing guarantee follows from T_E |
| Old E aliases, including rejected-record evidence | Fixed hydration and persistent complete raw bytes | An unresolved private-only or absent object is not public evidence |
| Current DATA/FINAL and same-session covering path | Supplied inline/public objects; signature and path checks under prepared AUTH0 | Missing required candidate bytes do not become complete through another observer's private cache |
| Publisher-selected recovery AUTH, regional rho proof or native-output sidecar | Excluded from the public kernel; intrinsic validity follows from the actual correct signing event | They remain relevant to their separate full regional/native interfaces |
| Initial Enter | Exact prepared executable anchor and full Ready inputs | No residual unbounded state import may be hidden in the attachment |
| Regional Vote/recovery and FINAL formation | Full complete bundles plus the finite cached authority prefix used in R_i | No fixed bound is asserted before preparation |
| Successor AUTH after a quota-one first DATA | Only ResolveBoundary needs it, after durable terminal CommitNextDATA | Successor resolution may wait indefinitely long within its eventual premise; accepted DATA remains exposed |

The terminal split is explicit in `exports:125–162` and P6 (`...v3_7.tex:2195–2202`), so an eager whole-record lookup of AUTH(r+1,0) is not authorized to withhold the current terminal commit.

## 9. What the regional timing statement actually establishes

> “The charged complete-message operation consists of … lookup of resident exact bytes … and the guarded attachment of already executable state.” — `source_step:238–250`.

The original communication premise includes delivery and processing of complete validation dependencies (`baseline/...v3_1.tex:360–372`; `baseline/...part1_core.tex:159–171,283–293`). The new protocol defines an immediate correct-to-correct SOURCESTEP relay with actual required bytes and a complete receiver operation; it does not defer its action to an unrelated polling or native reconstruction task (`source_step:133–178`). For the maintained prepared regional input path under its admitted input/load limits, charging that operation to the inherited regional bound supports Enter by a+T_E+Delta_i, or intervening closure. The ReadyBundle skew is similarly conditional on actual bounded validation/adoption/Enter, and explicitly leaves residual import delay u_v unbounded when that interface is not met (`ready_bundle:107–146`).

I do **not** infer a bound merely because inventory is finite, state is called resident, or processing is called atomic. A general CPU/backlog bound does not follow. The mathematical proposition specifies the complete-message interface it uses and limits its claim to members already maintaining that path. A concrete adapter that leaves restoration or external access outside the handler has not implemented that interface; the document itself withholds the corresponding timing claim. Arbitrary external observers receive no regional Delta_i processing guarantee.

On this interpretation, the public-response sum L=max_i(2T_E+2Delta_i+R_i+T_pub) has exactly the opening source/relay, regional response, complete-result publication, and result source/relay stages stated at `liveness:230–288`. Neither optional AUTH acquisition nor successor resolution is charged to those Delta_i terms. **Classification: valid conditional abstract timing; no deployment performance certification.**

## 10. Remaining chain and objective coverage

The finite regional response uses the complete inherited Part I initializer fairness and permanent exclusion tail, not merely a correct name in an unentered schedule (`liveness:17–42,63–154`; `baseline/...part1_core.tex:185–223,870–928`). Finite fixed-slot preparation is legitimate without a uniform bound over all future slots. Health credits permit only finitely many further interventions at a fixed G, and leases have already finalized restoration ON records (`rotation_v3_5_health.tex:120–169`).

Under absence of growth, immediate eligible reassignment after each AUTO is material: it makes successive closing/opening real times at most L apart. Timestamp increments are at least M_n W_min while M_n=1+beta_0+n, so their sum is quadratic. The upper-lead constraint bounds the same timestamp by a_0+NL+epsilon_E (`liveness:301–351`). This comparison uses no lower timestamp-lag or empty-block-rate premise. Owed-set service keeps REGULAR selection unrestricted and filters only Force; empty DATA legitimately discharge regional credits (`liveness:362–440`).

These claims preserve the strict pre-payload cutoff and MSIG priority (`rotation_v3_6_control.tex:207–284`), and all six protocols remain present. Receipt correctness, public DATA progress and repeated regional service do not establish request inclusion. The retirement rules preserve existing timers, require genuine Quit or Completion to cancel them, and separately transfer retention/publication duties; they do not give a complete receipt to every late local commit (`rotation_v3_4_retirement.tex:92–169,370–381`).

> “The accepted empty DATA grow G and discharge regional debts, while strict cutoff rejects every late result containing x.” — `liveness:464–478`.

The disclosed execution is consistent with the model: choose quota at least two; each new slot first accepts a Byzantine leader's valid empty DATA; withhold that slot's recovery authority until a finite late time; allow the correct recovery initializer actually to enter and propose the oldest pending x; place the AUTO crossing after the necessary correct votes but before complete FINAL formation. For Delta=1, votes at d+4 and d+5, a crossing at d+5.5, and timer expiries at d+6 and d+7 respect the stated bounds. Closure does not cancel those timers, and later publication can meet both service bounds while cutoff still rejects that version's x. Positive accepted count moves to a fresh slot, so cached zero-output retry preparation does not prevent the next authority delay. No equivocation, permanently missing quorum, dishonest committee, failed Espresso service or violation of actually entered correct turns is required.

**Objective assessment:** The candidate supports conditional input-prefix preservation, explicit handler/lifecycle behavior, and the stated prepared DATA-progress/repeated-service results. It does not meet the full objective of eventual acceptance of every persistently admissible correct-client request. That shortfall cannot be removed by relabeling empty-DATA growth as request liveness or by adding an unstated deadline for each new authority, canonical request ingress, READY publication, or accepted honest proposals. This review supplies no repair and makes no complete-goal claim.

## Validation limits

This was a manual mathematical and source-semantic audit with SHA-256 and Model-section equality checks. I ran no executable protocol/model checker, no native adapter benchmark, no binary codec compatibility test and no deployment experiment. The complete-result service, cryptographic assumptions and native soundness interfaces were checked as stated premises rather than independently implemented or empirically established.
