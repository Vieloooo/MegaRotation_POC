# REPORT evidence, the FINAL kernel, and the frozen delivery contract

Status: design analysis only. This note changes no protocol source or assumption, does not assume permission to replace FIFO batching, and does not establish request liveness. It separates three questions that must not be conflated: what FINAL proves, what the publication service must carry, and what the regional delay bound covers.

The narrow answer is: **REPORT need not be a public acceptance dependency, but a kernel-only publication is not a refinement of v3.7's literal full-result carriage contract.** A different, potentially useful construction preserves the entire original complete public result and keeps only the newly introduced REPORT evidence regional. That construction has a clean zero-additional-public-bytes lemma, provided the old service input type is explicitly preserved. It is not the claim that the existing service already accepts or delivers only a kernel.

Source abbreviations below refer to the live v3.7 files in `formalization_v2/`: `main` is `sync_hotstuff_rr_part2_rotation_v3_7.tex`; `kernel`, `carrier`, and `exports` are the corresponding `sections/rotation_v3_7_*.tex` fragments. `scope` is the adjacent `request_report_scope_analysis.md`. `Part I` is `sync_hotstuff_rr_part1_core.tex`. The current sources and the frozen snapshot use the relevant definitions quoted below.

## 1. Four objects with different obligations

Let `C` be the authentic incoming boundary and fixed initial authority of a producing session. Distinguish:

| Object | Meaning | What availability of this object alone permits |
| --- | --- | --- |
| `O=(D, Sigma, path)` | Exact DATA, direct or covering FINAL signatures, and the necessary exact hash-linked bodies | The cryptographic commitment/input-validity judgment, after authenticating `C` and the independent public gates |
| `R0=(O,W0)` | An **original complete regional result**: every ancestry, signed proposal, boundary, authority/source and native-validation object required by the frozen full-result interface, with the original public source denotations | The original `T_pub` service, subject to its original size/load limits |
| `R+=(R0,WR)` | An augmented regional bundle with the new REPORT certificate(s) needed for all relevant own-session first-DATA validation | Full validation under the proposed REPORT algorithm; its publication status is a separate interface question |
| `K(R0)` | The `CheckFINAL` projection plus the caller's required public source context | Public acceptance, if the exact extension, duty, quota, reference and Force checks also pass |

The selected primary's bytes occur once in the ordinary application batch. `WR` need contain report signatures, keys, contexts and member identities, not duplicate copies of selected request bytes. Nonselected full candidate bodies are required by the proposed correct leader's collection rule but are not inputs to the other validators' report-certificate check; they need not become public result evidence merely because the leader received them.

`R0` is deliberately stronger than `K(R0)`. The word "original" means the complete old interface, not whichever subset the new checker happens to accept. A reference to a private regional object is not a public representation of it (`carrier:27–54`).

## 2. The strengthened honest-FINAL implication is valid in principle

The existing proof starts with raw matching FINAL weight greater than `F`, identifies an actual correct signer, and traces that signature to a timer-origin commitment and an earlier fully validated Vote. Receipt-driven commitment cannot generate the first correct FINAL. Hash binding identifies the supplied DATA and exact ancestry with that genuine history (`kernel:33–85`; `main:835–891,941–955`). It does not certify a publisher's chosen alternative witness.

Suppose a future amended regional algorithm adds the REPORT predicate to **every** relevant own-session proposal/ancestry validation path before Vote, recovery validation and receipt-driven Completion. Correct REPORT signers obey the specified selection rule, correct FINAL generation remains unchanged, and all correct signers of the session run this amended algorithm. Let

`ValidPlus(C,H,W0,WR) = Valid0(C,H,W0) and ReportValid(C,H,WR)`.

Here `H` is a genuine historical proposal/ancestry witness. `ReportValid` checks the authentic term and exact parent, distinct-member weight `>F`, minimum signed nonempty key, the selected request's matching bytes, admission, capacity, absence from parent ancestry, and first execution in DATA. It also requires all relevant first-DATA ancestors to have their own valid evidence. DRAIN creates no new primary.

**Extended kernel lemma.** Under those algorithmic rules and the original cryptographic, fault and sound-validation premises,

`RawFINAL_C(D,Sigma) => exists H,W0,WR: genuine_timer_history(H,D) and ValidPlus(C,H,W0,WR)`.

The proof is the existing proof with one extra conjunct in the actual correct Vote's full validation. No new certificate type or signature field is needed for the *FINAL* implication. An inherited anchor invokes the earlier-event induction exactly as before. A REPORT set greater than `F` contains a correct reporter; this fact concerns a different actual signing event and does not assume honest intersection between two threshold sets.

Consequently, public `CheckFINAL` need not fetch `WR`. Its public input size and verification dependency graph can stay the same. All existing exact DATA/DRAIN path bodies, fixed incoming context, public control history, strict Espresso-reference precedence, branch permission and Force cutoff remain required (`kernel:88–149`).

This is an **existential historical validity** statement. FINAL binds neither generation nor term. It does not authenticate a publisher-named REPORT certificate, a claimed particular `j=0` witness, every possible admission path for the same DATA, or the unpublished bodies of Byzantine nonselected reports. The selected application's exact bytes are hash-bound; the existence of a genuine valid witness is certified. A request-liveness argument must use that genuine witnessed history and its awareness property, not an arbitrary later wrapper.

Nor can the rule be retroactively attributed to old FINAL certificates. The amended lemma applies to sessions known to execute the amended signing/validation algorithm. A transition from old to new rules needs an unambiguous existing authenticated context or an explicit activation design; the unchanged FINAL format alone cannot prove that an old signer enforced a rule that did not yet exist. This note supplies no deployment-upgrade construction.

## 3. Why public acceptance sufficiency is not a full-carriage refinement

The frozen definition is DATA with its direct/covering FINAL receipt and the ancestry, boundaries and source evidence needed to validate it (`main:339–344`). The service submits a record **carrying that result**, preserving DATA and commitment evidence, under fixed input-size/load limits (`main:406–421`). Espresso then makes the complete record available (`main:461–479`).

V3.7 intentionally keeps the old logical transport scope:

- `carrier:17–25` defines a complete result using the delivery and full-admission interfaces, before normalization.
- `carrier:56–66,104–138` preserves every actual complete-result object and dependency edge. Extracting the kernel is a subsequent operation; it is not deletion of the rest of the delivered logical value.
- `carrier:146–165` says the complete logical result is delivered and concrete encoders preserve those values and source denotations.
- `kernel:162–167` expressly says kernel use does not authorize omitting evidence required by delivery/retention obligations.
- `exports:203–234` keeps the original complete-result input and excludes broader transport interpretations that do not provide the original logical complete result from the progress proof.

Consider a correct holder offering complete `R0=(O,W0)` where an original signed own-session proposal or admission dependency `w` in `W0` has no existing authenticated public occurrence. Suppose the service outputs only `K(R0)` and drops `w`. The kernel may pass, but the new record does not carry the old full logical validation witness. A later regional full validator cannot recover `w` from that record or its unchanged source denotations. This violates the carriage/refinement relation without breaking FINAL soundness.

Keeping `w` in the publisher's private archive does not repair the public delivery obligation. Replacing it by a hash or a new private resolver also fails. If `w` already has an established retained public denotation, an exact alias can preserve it; that is existing sharing, not a universal projection theorem. In particular, a first fresh REPORT certificate has no guaranteed prior public occurrence.

Thus the implication

`complete R0 offered at s => kernel-only record is a faithful delivery of R0 by s+T_pub`

does **not** follow from the frozen contract. Offering a stronger input does not license delivery of a weaker logical output. The honest-FINAL lemma proves existence of a valid witness, not recoverability of the offered witness. "Preserves DATA and commitment evidence" is not permission to ignore the surrounding full-result and original-denotation requirements.

## 4. A report-free old-format publication adapter can be defined separately

There is a narrower construction that avoids additional public REPORT bytes **without deleting any required frozen evidence**. It retains `R0` on the public wire, rather than publishing only `K(R0)`.

Define the publication service domain to remain exactly its **original** complete-result type `C0`. The amended regional algorithm constructs and retains `R+=(R0,WR)`, but presents the old service with `R0`. The new report certificates are additional regional proposal-validation evidence, not new fields in the original publication result. The service still carries all of `R0` through the original logical tuple/source interface. Public acceptance extracts `K(R0)` and uses the extended kernel lemma.

This split must be explicit in the amended algorithms. It cannot be obtained by leaving "complete result" overloaded and then assuming that the old word both includes and excludes `WR`. If the amended full-result definition instead says that every new regional validation dependency is part of the object the Publish Layer must carry, then its input is `R+`, REPORT is required carriage, and this construction is unavailable under that definition.

The required erasure conditions are:

1. **Old completeness.** Every offered `R0` satisfies the original full evidence obligations. The old validity predicate is a conjunct of amended validation. A raw kernel that was formerly incomplete is never promoted into the service domain.
2. **Original signed objects.** DATA, its ordinary batch, FINAL signatures, signed original proposals, old QCs, authorities, boundaries, ancestry and original source denotations remain byte-for-byte or logically unchanged as applicable. `WR` is a separately framed validation sidecar. It must not be inserted into the signed DATA body, a changed hash encoding, or an original signed proposal field that cannot be erased while preserving its signature.
3. **Public completeness.** Publication sends every old required object inline or through an unchanged established public reference. It does not use REPORT's irrelevance to `CheckFINAL` to omit unrelated original evidence.
4. **Regional completeness.** REPORT evidence remains available for the augmented own-session validation, receipt/Completion, recovery, evidence inspection and retirement duties that require it. Public old-format results alone need not satisfy those augmented regional duties. A separate reliable regional transfer/retention path must implement them; no new public archive guarantee is inferred.
5. **Same timing origin and profile.** `T_pub` starts when the complete `R0` is actually available to the original service. REPORT acquisition, signature collection, forming `R+`, or an unfinished erasure operation does not move an earlier event into that service's domain. No extra on-wire metadata, extra public submission round or new source lookup is needed.

**Old-interface erasure lemma.** Let `erase(R+)=R0` satisfy conditions 1–5. For any original service output `E0` that carries `R0` under its original source context, the unchanged structural normalization preserves the original complete result and its public kernel. The public checker accepts exactly the same hash-bound DATA and obeys the same global gates. In an amended-session execution, the extended honest-FINAL lemma additionally supplies `ReportValid` for the genuine historical witness. The original publication premise applies at the availability time of `R0`, to the same complete input type and same old-format output.

**Proof.** Amended full validation implies original full validation, so `R0` is in the old service domain. Apply the existing service contract to `R0`, then the existing carrier-preservation lemma to the actual `E0`. Its public kernel is unchanged because the newly removed sidecar is outside that kernel and no old required object or denotation is removed. Apply the extended signing lemma for the stronger historical predicate. This proof neither asks the service to accept an incomplete old input nor asks it to deliver less than its original complete logical result.

This lemma gives a possible algorithmic interface refactor under the frozen environment. It is not an already-established property of v3.7, and it does not permit the full-output-to-kernel-only mapping rejected above. If preservation of the existing single augmented "full regional result" interface is mandatory, the exact remaining obstruction is the required REPORT carriage and its unproved fit in the old public size envelope.

## 5. What can and cannot be concluded about bytes

Fix the same DATA payload and original witness `W0`, not an unrelated old FIFO execution with a shorter batch. Let `E0` be its original complete EXPORT representation, and let `B_old` be the old admitted encoded-record limit. If the adapter emits exactly the same old-format `E0`, then

`bytes(E_new) = bytes(E0) <= B_old`.

At the abstract object level, the report-augmented local result has

`bytes(R+) = bytes(R0) + bytes(WR) + local framing`,

but that extra local component is not input to the original publication service under the explicit split. Likewise,

`K_new(R+) = K_old(R0)`.

There are zero additional REPORT bytes in the public envelope, and the selected primary is already in the ordinary DATA batch. This does not reduce its application capacity. It also does not establish that every possible original complete result was within the old admitted profile: that inherited premise still applies, including ancestry and original evidence costs.

For a hypothetical deletion-monotone codec one also has `bytes(encode(K(R0))) <= bytes(encode(R0))`, possibly with framing qualifications. That is only a size statement. The left-hand object generally fails the original complete-carriage relation. A smaller unauthorized output does not close the service proof.

If the unified augmented full-result interface is retained, with `q` first-DATA REPORT witnesses in the required history and roster size at most `n`, then a basic explicit encoding adds approximately

`sum_(a=1..q) R(n_a),  R(n)=O(n*(signature_bytes + key_bytes + member_id_bytes) + context_bytes)`.

The relevant obligation is `bytes(E_plus) <= B_old`, not merely `R(n)=O(n)`. No numerical headroom or concrete codec is specified in the sources. One cannot assume that an application request fitting alone leaves REPORT capacity, or reduce the original maximum request size to make room. Repeated history/certificate copying must be counted even where a single new REPORT certificate is small.

## 6. Regional REPORT traffic and processing

Assume the roster and DATA batch capacity are the installed finite values. For one first-DATA context, let `b` bound one admissible request body and let `R(n)` be the report certificate's metadata size.

| Operation | Additional traffic/work | Scope of the bound |
| --- | --- | --- |
| One REPORT from each member to its leader | `O(n)` point-to-point messages and distinct signatures; at most `O(n*b + R(n))` candidate/body bytes into a correct leader | One candidate per reporter; no whole-pool transfer |
| Correct leader validates reports | At most one checked eligible body per retained member report, `O(n)` signatures, context/weight checks and minimum comparison | Byzantine malformed traffic needs normal rejection/deduplication; no promised completion of a Byzantine reply |
| Certificate in first-DATA proposal | `O(R(n))` extra dependency bytes per copy, selected body already in DATA | Other validators do not need full bodies of nonselected reports |
| Original forwarding of complete proposals | The added certificate follows existing dissemination paths | With `O(n^2)` correct forwards, the conservative extra traffic is `O(n^2 R(n))`, potentially `O(n^3 signature_bytes)`; saying merely "O(n) extra messages" hides this |
| Required report evidence in recovery/receipt ancestry | The sum of the previously needed certificates, each dependency represented once per complete bundle | Per-operation history size can grow; caching can avoid repeat transfer only when recipients can resolve exact objects within the original bound |

The original Part I cost accounting already distinguishes message count, QC bytes, dependency bytes, and history/retrieval amplification (`Part I:1028–1086`). Adding `WR` to a complete regional bundle changes its dependency-byte term even if the number of Vote messages is unchanged. The public erasure construction does not erase these regional costs.

The literal regional communication assumption bounds delivery **and processing of complete validation dependencies** for correct-to-correct messages (`main:392–404,941–955`). Thus, at the abstract protocol level, a complete REPORT-bearing proposal can use the existing `Delta`; it is not necessary to invent an extra `Delta` merely to verify an `O(n)` certificate. This is a use of the already universal complete-message premise, not a measured performance claim.

Conversely, `O(n)` is not a proof of unchanged concrete CPU/bandwidth/load limits. The sources give no numerical old operating profile demonstrating the added body intake, signature checks and repeated certificate forwarding fit. Nor does the communication premise bound work before a message is sent, an unbounded local queue scan at a timer callback, cross-region owner discovery, a new native-state import, or an indefinitely incomplete request-acquisition job. An implementation must place actual processing in the admitted complete-message operation or maintain the needed state/index in advance. It cannot call arbitrary preparatory work "processing" and thereby derive an independent service bound.

## 7. Timing fits only when the acquisition contract is real

The timing calculation in `scope:75–114` remains correct as a conditional schedule. Let `e` be first correct term entry and `e_v` each local entry.

| Stage | Ordinary term | Initial/recovery term zero under the existing prepared entry contract |
| --- | --- | --- |
| Last correct entry | `e+2Delta` | `e+Delta` |
| A genuinely sufficient entry-started two-message acquisition finishes everywhere | `e+4Delta` | `e+3Delta` |
| Exact parent available for reporting | Original terminal-QC path | Valid NEWVIEW by `e+4Delta` |
| Reports sent everywhere | `e+4Delta` | `max(e_v+2Delta, valid NEWVIEW)` no later than `e+4Delta` |
| Complete correct-only report quorum reaches leader | `e+5Delta` | `e+5Delta`, also the original ANCHOR-QC bound |
| First DATA fully validated and voted everywhere | `e+6Delta` | `e+6Delta` |

The original first-position vote deadline is `e_v+6Delta`; later positions add `2Delta`. At equality, due complete messages precede the progress callback (`Part I:681–709`). REPORT must not delay the original immediate ANCHOR vote. If REPORT is merely packaged alongside that vote, a malformed report subrecord must not suppress the independent valid ANCHOR vote or embedded quit evidence.

The `2Delta` acquisition row cannot be read as a new mempool service assumption:

- Sending at most one locally selected complete candidate in REPORT itself needs no full-pool exchange, but does not prove all-current-roster awareness of an old request.
- If **each** current member queries every current member, this is `O(n^2)` requests/replies, not the `O(n)` report phase. Returning one candidate can be bounded in bytes, but is not automatically sufficient for later parent-dependent filtering.
- An anchor-independent early response `[a]` from a pool `[a,b]` can fail when the eventual recovery parent already contains `a`; `b` can remain unknown. Returning every possible fallback is an uncontrolled full-pool transfer unless a separate bound/protocol is proved.
- Starting an anchor-dependent query after NEWVIEW can give first DATA at `e+8Delta`, beyond the old deadline. Serving queries only after the responder's own ordinary-term entry can likewise miss the early requester's report cutoff.
- Querying current members cannot locate a request held only by a disjoint retired roster. The original `Delta_i` is regional, and does not provide a bounded cross-region or historical-owner discovery operation.

Reports based only on each correct signer's retained local eligible set avoid a new acquisition wait. They still need a concrete way to determine the selection at the reporting event, and leave the changing-membership awareness condition from `scope:129–166` open. A single correct pending owner or a `>F` ownership acknowledgement is insufficient to force every possible later `>F` REPORT certificate to contain an aware correct signer. The existing three-member counterexample remains decisive.

## 8. Result of this design audit

1. **Established at the abstract proof level:** enforcing the rule in genuine regional full validation lets unchanged-form FINAL certify the existence of a valid first-DATA REPORT witness. The public acceptance kernel does not need new REPORT signatures or nonselected bodies.
2. **Rejected as a refinement of the literal v3.7 delivery contract:** offering a complete original result and publishing only its minimal kernel, when original required evidence has no existing public denotation. Verification sufficiency does not imply full-result carriage.
3. **Available as an explicit future interface construction:** offer and publish the entire original complete result `R0`, keep only new REPORT evidence in a separate regional bundle, and use the extended lemma publicly. Under the stated erasure conditions, the public record is the same old-format object and adds zero bytes. This preserves the old service domain rather than extending `T_pub` to formerly incomplete inputs. It requires an explicit algorithm/interface distinction and preservation of regional report duties; it is not yet implemented or integrated.
4. **Still unproved:** concrete regional load compatibility; bounded sufficient acquisition and changing-roster request awareness; preservation of all core/retirement rules after the new gate; and the resulting full request-inclusion theorem. The proposed split removes a possible public-size obstacle only. It does not settle those separate questions or the user's FIFO choice.

Validation consisted of source-level analysis of the current v3.7 delivery, kernel, carrier, export and regional message rules, the existing request-report analysis, and Part I timing/cost definitions. No manuscript edits, codec implementation, deployment benchmark, model checking, or full proof certification were performed.
