# Fresh verification: model fidelity and public carriage, v3.8

## Summary

**Final Verdict:** The reviewed model-fidelity, publication-projection and public-FINAL arguments are valid within their expressly stated abstract complete-handler scope; I found no **Critical Error** or **Justification Gap** in those arguments, but they do not establish the unresolved request-liveness objective for arbitrary changing memberships.

**List of Findings:** No theorem error or missing inference was established in the assigned scope. The following are material limits of this verdict, already stated in the candidate, rather than defects silently repaired by this review:

- The request-inclusion proposition assumes infinitely many accepted first-DATA witnesses with an aware correct reporter **at its report event**. The frozen ingress prose does not imply this condition for arbitrary disjoint rosters. This remains an unresolved obligation for the overall objective, not a counterexample to that conditional proposition.
- The regional timing claims apply to the specified complete-handler interface, with eligibility processing completed in the existing abstract message/batch-processing scope. Neither an eligibility index nor a finite input proves a concrete CPU, bandwidth, native-import or backlog bound. No implementation demonstration of that interface was supplied.
- Logical carrier preservation is not a deployed codec compatibility theorem. No concrete codec, private resolver, native adapter or benchmark was verified.

The review used only `review_scope.md`, `manifest.json`, the frozen `snapshot/` tree, and the required proof-verifier skill. It did not consult design notes or other review reports. All **23/23** manifest hashes matched before review and again after the report was written. The main snapshot hash is `c8611baf07c74f34512a182b0b3ace81d6f8f3cf65fb0cb567af696ec8a73574`. No protocol or proof source was edited. This was a source-level verification, not an executable protocol simulation or model-checking run.

All locations below are relative to `snapshot/`. “Main” denotes `sync_hotstuff_rr_part2_rotation_v3_8.tex`.

## Detailed Verification Log

### 1. Frozen assumptions and dependency map

> “The Model section is unchanged from v3.1.” — Main, lines 124–133.

The main Model section, beginning at line 228 and ending before line 549, is byte-for-byte equal to `baseline/model_and_assumptions.tex`. This literal check is useful but insufficient by itself. The semantic dependencies checked below are:

1. The fixed-snapshot fault and lifetime-compromise bounds identify an actual correct FINAL signer.
2. The signer’s timer originates in a full regional Vote, including the new REPORT rule; this establishes a genuine historical witness.
3. Erasing only REPORT sidecars maps the augmented regional bundle to the entire original publication input.
4. Logical normalization preserves that original input’s public kernel without creating missing objects or source denotations.
5. Public acceptance still checks the authenticated incoming boundary, current permission, exact extension, reference precedence and Force cutoff separately.
6. Prepared regional progress uses the revised first-DATA timings; public response then uses the unchanged complete-result publication and visibility bounds.
7. Global DATA growth and repeated regional service remain distinct from request inclusion, whose additional awareness premise is explicit.

The new selection, relay, retention and handler rules are algorithm rules. They are not consequences already supplied by the old fairness prose. The candidate describes the change that way: `sections/rotation_v3_8_requests.tex`, lines 13–21, expressly replaces first-position FIFO selection; `sections/rotation_v3_5_health.tex`, lines 3–8, expressly says committee honesty alone did not imply the new finite-credit policy.

### 2. Original complete-result contract versus R0/R+

> “A complete regional result is DATA with its direct or covering FINAL receipt and the ancestry, boundaries and source evidence needed to validate it.” — `baseline/model_and_assumptions.tex`, lines 114–119.

> “Every relevant ancestor includes its signed proposal and block data” — `baseline/sync_hotstuff_rr_part2_rotation_v3_1.tex`, lines 887–896.

> “Evidence includes regional receipts, ancestry, REGULAR boundaries, historical authorities and native application context.” — the same baseline, lines 1338–1345.

These original requirements are not replaced by a receipt/hash-only carrier. `sections/rotation_v3_8_requests.tex`, lines 159–173, defines `R0` to retain every original DATA, FINAL, proposal, QC, ancestry, boundary, authority, native-validation and public-source object required by the old interface. `R+=(R0,WR)` adds the REPORT sidecars needed by the new regional validation rule. The distinction matters because a public `R0` can be adequate for the new public checker while still being insufficient for augmented **own-session** Completion.

> “Erasure removes only the new separately framed sidecars; it preserves all old required objects, signatures and established public denotations.” — `sections/rotation_v3_8_requests.tex`, lines 175–182.

The projection is well-defined at the logical interface used here. Original DATA/PROPOSE/vote/QC/FINAL signed objects and their hashes do not include a newly inserted REPORT field (lines 122–126); the primary request already occupies ordinary application payload capacity. For fixed DATA and original witness, deletion of `WR` therefore adds no public bytes and removes no old validation object. This does **not** claim that the regional messages have zero added bytes: complete `R+` forwarding and retention are explicitly required.

> “They offer it as soon as its original dependencies are available” and “does not promise Tpub for a weaker kernel-only input” — `sections/rotation_v3_8_exports.tex`, lines 203–214.

Thus the original availability event starts the publication clock; report acquisition is neither a new publication input requirement nor grounds to delay an already available original result. A bare request, report, READY message, hash or receipt lacking payload does not invoke the service. This agrees with the baseline’s `Tpub` origin and its stated input-size/load scope (`baseline/model_and_assumptions.tex`, lines 181–196). No extra public size headroom or bare-request publication promise was used.

### 3. Historical FINAL soundness without signed generation or term

> “This is a cryptographic and type check only” — `sections/rotation_v3_8_receipt_kernel.tex`, lines 15–31.

`RawFINAL` checks the authenticated incoming context and distinct signer weight greater than the fixed fault bound. It deliberately does not build the desired validity conclusion into its premise. The baseline fault/lifetime conditions (`baseline/model_and_assumptions.tex`, lines 137–153) and the main signed context (lines 800–814) make at least one signature an actual correct signing action for the exact session and DATA hash.

> “A correct replica generates FINAL only after Commit’s timer event” — `sections/rotation_v3_8_receipt_kernel.tex`, lines 59–75.

The main Vote rule requires complete validation before starting the timer (lines 1133–1155); Commit emits FINAL for the timed DATA and its newly committed same-session DATA ancestors (lines 1158–1164). Main lines 888–893 distinguish receipt-driven commits, which emit no new FINAL. The REPORT rule applies to first-DATA proposals, relevant own-session ancestry, recovery and own-session receipt handling (`sections/rotation_v3_8_requests.tex`, lines 128–148). Therefore the correct signature has a genuine augmented-valid historical origin.

The inherited-anchor step is not circular: source FINAL signatures used at a new Vote already exist before that Vote; their timer origins are earlier execution events. The induction from the trusted checkpoint in the kernel proof, lines 77–89, also applies to an unaccepted REGULAR anchor. It does not assume that such an anchor has entered `G`.

> “FINAL deliberately binds no generation or term. The rule does not certify that alternate path” — the kernel, lines 121–130.

This restriction is essential and respected. The proof establishes **existence** of the historical report/admission witness checked by a correct signer. It does not authenticate a publisher’s alternate `g`, `t`, QC, REPORT set or recovery AUTH path. As a result, a publisher cannot force an arbitrary recovery-authority fetch merely by attaching an alternate witness. The request proposition uses genuine historical witnesses, not a publicly claimed term annotation (`sections/rotation_v3_8_requests.tex`, lines 274–281).

The kernel does not turn an unbound native state/output/DA sidecar into a proved claim. Its native conclusion covers only input-validity predicates actually checked for the hash-bound DATA; exact executable state, Ready and `Cpub` correspondence remain independent (`sections/rotation_v3_8_receipt_kernel.tex`, lines 155–174). This preserves the baseline sound-interface assumption without inferring a new native-processing deadline.

### 4. Public evidence preservation and private dependencies

> “A reference retains its baseline denotation” — `sections/rotation_v3_8_carrier.tex`, lines 31–58.

The retained dictionary represents actual authenticated public objects; a regional cache reference cannot make a private-only object public. This is consistent with the baseline’s separate L1 and Espresso source obligations. In particular, L1 evidence is not required to have appeared earlier in Espresso, and no redundant occurrence-position field is imposed on a valid original reference.

> “Fix the finite validation witness of a complete result in the original logical fields.” — the carrier proof, lines 120–141.

The structural proof preserves leaves, compound children, shared edges and reference denotations. Keeping an unresolved edge proves representation preservation, not target availability. The distinction is maintained in the subsequent extraction claim: required DATA/FINAL/path/context bytes must already be supplied through the inherited public input interface. Optional recovery AUTH or native proofs are excluded from that projection; they do not conceal required payload by being called optional. Inline structured children remain inventoried despite a bad enclosing proof.

No equivalence to the old all-or-nothing acceptance trace is claimed (carrier, lines 144–148). Bad outer hints are deliberately no longer acceptance vetoes. Conversely, normalization neither manufactures omitted bytes nor repairs an unspecified compressed/opaque encoding. The precise compatibility limitation is stated at lines 161–181.

> “No publisher-selected generation, term or admission path is an input.” — `sections/rotation_v3_8_receipt_kernel.tex`, lines 103–119.

The public kernel remains separated from full regional validation. Its operation still checks actual DATA/DRAIN types, hashes, links, endpoint, same-session coverage and quota. The public caller additionally checks exact current `G`, the incoming duty, authentic selection context, reference precedence and the strict post-Force containing-block cutoff (`sections/rotation_v3_8_exports.tex`, lines 78–100). A valid historical receipt does not itself choose a branch or authorize a current append.

### 5. Request processing, awareness and timing charges

> “Maintain a key-ordered index … for the exact parent.” — `sections/rotation_v3_8_requests.tex`, lines 58–75.

The parent’s ancestry and native state affect eligibility, so an index from another parent cannot be reused unchecked. The candidate requires filtering to finish before exposing the projection to REPORT. It explicitly withholds the old timing conclusion if an implementation leaves an additional deferred pool scan, native import or eligibility rebuild outside completed processing. This is a conditional abstract batch-processing interface, not a derivation of constant computational cost from indexing.

The same limitation applies to input and load growth. `R+` carries additional regional signatures and contextual dependencies, and retained pending sets/histories can grow. The frozen message premise already covers complete validation dependencies; the regional timing proofs only apply to operations satisfying that premise. They do not supply measured feasibility for arbitrary traffic or database representations. Likewise, public normalization is finite but is not declared fast solely because it is finite (carrier, lines 144–148; `sections/rotation_v3_7_source_step.tex`, lines 238–250).

> “For one installed snapshot” — `sections/rotation_v3_8_requests.tex`, lines 23–44.

The relay’s `Delta_i` bound applies to a dispatched full request within that authenticated snapshot. The pending store and transport duties survive worker exit. Learning a snapshot schedules retained individual requests but does not create a bound for discovering a roster, dispatching an arbitrary backlog or transferring between disjoint memberships. No mixed-snapshot `Delta` or all-current/all-future awareness was imported.

The ordinary report timing uses the retained terminal-QC dissemination argument: all correct members enter by `e+2Delta`, reports reach the correct leader by `e+3Delta`, and the first proposal is voted by `e+4Delta`. In term zero, NEWVIEW processing and independent ANCHOR/report messages reach the leader by `e+5Delta`, so the first vote is by `e+6Delta` (`sections/rotation_v3_8_requests.tex`, lines 203–241). These fit the original first progress deadline `e_v+6Delta`; the older ordinary `2Delta` first-vote latency is explicitly relinquished. Reports do not suspend the original progress checks or ANCHOR action. This is sufficient for the cited prepared response chain, conditional on the complete-handler and unfrozen/unclosed scope.

### 6. Preservation of DATA/global/regional progress scope

> “All finite sets used below belong to this installed configuration” — `sections/rotation_v3_8_liveness.tex`, lines 44–59.

The stalled-frontier proof fixes `G*` and, after at most one positive-output transition, a slot `r*`. Its roster, authority prefix and regional parameter maxima are then finite. This permits a finite preparation time and a fixed response witness without assuming a finite union of every future roster or a uniform `Delta` across all future AUTH records. Historical hydration, authority acquisition and native preparation remain eventual (lines 94–105).

The new REPORT gate does not insert a request-acquisition stage into this prepared proof: correct reports alone exceed `F`, and an empty eligible set yields EMPTY. The resulting first-vote bounds feed the explicit finite response calculation and the correct initializer’s protected receipt window (liveness, lines 107–157). This verifies the dependency on REPORT without claiming all pending requests are globally known.

For public response, the source dependency case split (liveness, lines 189–232) uses retained exact source bytes, the prepared initial authority/anchor, advance-announced controls and actual complete regional messages. The two charged source handlers operate on prepared/resident inputs. The proof does not charge a new archive read, arbitrary recovery AUTH, native-state import or arbitrary external observer’s processing to those `Delta_i` terms (lines 234–292).

The publication step still starts from the original complete result. An earlier adversarial logical carrying record can satisfy delivery, but the carrier argument makes its original kernel discoverable despite bad hints; it does not promise a repaired submission. If Force intervenes, the response’s closure alternative applies. For quota one, terminal DATA commits before successor-authority acquisition. The causal finalized-reference argument excludes publishing genuine current-session FINAL evidence in its own opening/reference block (lines 243–286).

The global contradiction then uses a finite fixed-slot response bound, increasing miss tolerance and the original timestamp upper-lead bound; it does not assume bounded lower timestamp lag (lines 295–364). The repeated-service proof keeps REGULAR selection outside the owed-set filter and uses strict increase of regular opening heights (lines 366–444). Empty accepted DATA can discharge a region’s service debt, so this proof remains a DATA-service result rather than request liveness.

### 7. Retention and acknowledged unresolved objective

> “required regional REPORT sidecars” and “each locally complete original publication result R0 has been offered” — `sections/rotation_v3_8_retirement.tex`, lines 130–152.

Public erasure does not authorize regional evidence deletion. FinishClose transfers sidecars, data, request relays and publication duties to a persistent owner before worker release. REPORT signing requires the current fresh-action gate (`sections/rotation_v3_8_requests.tex`, lines 87–104); post-exit owners may relay existing signatures but may not issue fresh REPORT signatures. Thus a public `R0` record is not incorrectly treated as a replacement for every retained own-session `R+` dependency.

> “Suppose infinitely many accepted term-first DATA have a genuine report witness containing a correct signer whose eligible retained set contained x at its report event.” — `sections/rotation_v3_8_requests.tex`, lines 243–250.

Under this exact premise, the finite-predecessor proof is sound: each counted accepted first block executes a primary of key at most `key(x)` that is absent from its exact parent ancestry; contiguous acceptance makes it a newly spent identity in `G`. The number of canonical strings of bounded length is finite, so infinitely many strict decreases are impossible. Discarded speculative commitments do not spend this potential, and Force does not roll back already spent identities (lines 253–272).

Awareness is evaluated at the report event. A later request cannot retroactively invalidate an old all-EMPTY certificate; the candidate explicitly says so at lines 283–289. The informal “informed” discussion in `sections/rotation_v3_8_liveness.tex`, lines 448–457, must be read with that report-event qualification, rather than as a current-queue veto over a saved historical witness.

The open membership trace is materially different: each disjoint roster can report before receiving `x`, receive and relay it later, and leave the next disjoint roster to repeat that order. Old owners’ retention does not transfer the bytes to the next roster. This explains the failed inference from pointwise eventual ingress to infinitely many aware accepted witnesses (`sections/rotation_v3_8_liveness.tex`, lines 466–495; requests, lines 291–301). The candidate expressly leaves the possible identity-level meaning of repeated correct turns unresolved. I do not silently choose a stronger reading to close that gap, and I do not mark the overall request-liveness objective achieved.

### 8. Final integrity check

The final manifest check again matched all 23 snapshot files, including the main hash stated above. The byte-identical baseline Model check also remained true. Only this Markdown review was added.
