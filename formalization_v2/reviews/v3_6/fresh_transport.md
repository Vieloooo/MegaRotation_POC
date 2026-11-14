# Summary

**Final Verdict:** The supported-syntax normalization and per-DATA acceptance branch survives the concrete transport cases checked here, but the claim of liveness under exactly the frozen v3.1 assumptions is **viable but incomplete due to Justification Gaps** concerning transport-format coverage and the complete-handler timing/native-state interface. I found no demonstrated **Critical Error** within the explicitly supported decoder and bounded-handler interpretation. The broader objective with only one working region and permanently unavailable correct quorums elsewhere, and request inclusion, remains unproved.

This is a fresh transport/model/scope audit, not a whole-protocol certification or executable/model-checking result. No earlier review report or verdict was consulted. No manuscript repair is proposed or performed.

**List of Findings:**

- **T1 — Justification Gap: supported normalization is not yet shown equivalent to the frozen complete-result carrier contract.** Location: `sections/rotation_v3_6_exports.tex:32–39,198–224`, especially “Both the existing v3.5 EXPORT body/dependency form and this revision's form use this interpretation” and “Here ‘carrying’ uses the supported decodable body/dependency syntax above”; `sections/rotation_v3_6_source_step.tex:14–35`; `sections/rotation_v3_6_liveness.tex:195–203`. The decoder repairs a record whose independently framed result objects and required public references satisfy the new predicate. The supplied frozen v3.1 text, however, does not specify that framing or the exact-position public-reference grammar. A compatibility/equivalence argument is missing. This is not a claim that omitted ancestry or arbitrary ciphertext is automatically a complete result, and it is not an established counterexample using the supported framing.
- **T2 — Justification Gap for the unchanged-model claim; explicit conditional interface for the narrower theorem.** Location: `sections/rotation_v3_5_ready_bundle.tex:107–144`, especially “If validation, adoption and the atomic Enter transition are the complete message-processing operation covered by [the network assumption], then”; `sections/rotation_v3_6_source_step.tex:151–178`; `sections/rotation_v3_6_liveness.tex:164–177,237–243,440–444`. The uniform public-response bound needs whole ordered source replay, normalization, durable acceptance, and executable-state attachment to fall within the inherited regional processing budget. The frozen model bounds complete validation dependencies and leaves state restoration unbounded; it does not independently prove this broader interface or its implementation. The candidate correctly exposes the condition, but that disclosure does not establish semantic equality with all frozen-model executions. The dependent global/service proofs remain conditional on it.
- **S1 — Scope limitation: “one eligible working region” is insufficient for the proof when other selectable owed regions can have permanently missing correct quorums.** Location: `sections/rotation_v3_6_liveness.tex:156–161,164–165,251–256,312–318`. The proof takes preparation/response maxima over every selectable region. It explicitly states that it does not cover permanently missing correct quorums in other assigned snapshots. Such an execution violates those regional serving premises, so it is not an in-model counterexample; it is an uncovered part of the broader failover objective.
- **S2 — Scope limitation, with a concrete acknowledged failure of the broader property: request inclusion remains open.** Location: `sections/rotation_v3_6_liveness.tex:394–436`. Empty accepted DATA can grow the global chain and discharge a region's debt while each honest proposal containing a pending request is closed before its complete receipt is available. The text does not claim request liveness, so this is not a Critical Error in a stated request-liveness theorem. It prevents a completed full-objective verdict.

## Snapshot and audit basis

All paths in the log below are relative to `formalization_v2/reviews/v3_6/snapshot/`.

- All **21** file hashes in the parent `manifest.json` matched, including included PDFs, figure sources, active section fragments, translations, and the nested baseline manifest.
- All **4** file hashes in `baseline/manifest.json` matched.
- Main SHA-256: `ef6d858b8ead057dd77f1a43e4198d47ceceb016e5c7b826eb941878b59d96d5`.
- The main source's Model section, lines **219–539**, is byte-for-byte equal to `baseline/model_and_assumptions.tex`.
- Read the full v3.1 baseline and Part I, and the full current main content using a line-preserving comparison against that baseline plus direct reads of every changed span. Read all seven active section fragments. Hash verification, rather than PDF rendering, was used for the figures. The supplied translations were integrity-checked but were not treated as the authoritative theorem statements.
- Applied the `proof-verifier` skill in verifier-only mode. The review stayed inside the frozen source package and parent scope; no external implementation or prior verdict supplied a missing premise.

# Detailed Verification Log

## 1. What the frozen contract actually gives

**Text checked:** `baseline/model_and_assumptions.tex:114–119`: “A complete regional result is DATA with its direct or covering FINAL receipt and the ancestry, boundaries and source evidence needed to validate it.”

This is a semantic completeness condition. A QC, missing DATA payload, or absent required ancestry is insufficient. It supplies no successor-authority requirement after a terminal result: the authority necessary to validate that result is its own historical authority and incoming boundary. The v3.6 terminal split is consistent with this distinction.

**Text checked:** `baseline/model_and_assumptions.tex:181–196`: the service submits an “Espresso-admissible record carrying that result,” preserves DATA and commitment evidence, has a working funded path, and “A result already recorded in Espresso satisfies this delivery guarantee; retries needed for acceptance into G are a separate task.”

The delivery task can already be discharged by an adversary's prior carrier. No proof may replace that carrier with a correctly based wrapper or infer a bounded retry. This audit therefore inspected extraction from the original bytes. The proof's two cases at `sections/rotation_v3_6_liveness.tex:195–203` correctly distinguish a not-yet-recorded result from an already finalized result, once that original record is within the supported decoding class.

**Text checked:** `baseline/model_and_assumptions.tex:236–265`: Espresso bounds record inclusion/finality/visibility and each newly finalized relevant namespace; it “does not promise that an EXPORT passes this chain's acceptance rules.”

Transport is not application acceptance. The candidate does not use Espresso inclusion as a substitute for a FINAL receipt, historical authority, ancestry, or current append permission. The algebraic delivery bounds remain conditional on the existing admission and load limits and on continuously online participants keeping up with history. Fees, working submission access, admissibility, and data availability are inherited premises, not consequences of the retry algorithm.

**Text checked:** `baseline/model_and_assumptions.tex:229–234,285–291,303–320`.

Long-term retrieval, sound regional/application validation, and recoverable native state are existing premises. Arbitrarily long catch-up, native reconstruction, and acceptance retries have no uniform bound. None should be hidden in a new constant named “handler.” The frozen progress paragraph also contains a successful-acceptance-interval condition. The new proof expressly avoids using that condition to prove itself (`sections/rotation_v3_6_liveness.tex:5–12`), which is the correct noncircular objective.

## 2. Result normalization against the requested bad-wrapper cases

**Text checked:** `sections/rotation_v3_6_exports.tex:7–12`: the outer chain/version/base/interval fields are “hints”; “A false hint neither authorizes invalid DATA nor vetoes a complete valid result carried in the same record.”

For independently decodable objects, the following repairs are substantive protocol behavior changes:

| Case | Relevant rule | Audit result |
| --- | --- | --- |
| Wrong outer expected base | `exports:85–102` derives the base from current G and signed ancestry | Does not block a compatible next DATA. |
| Wrong outer branch version | `exports:7–12,87–88`; `source_step:44–46` | Outer hint is ignored; the signed producing version must still be current and open. |
| Wrong outer chain/routing domain | `exports:7–12,32–37,87–88` | An outer label cannot veto a valid decoded result or change its signed chain/protocol domain. A genuinely foreign signed domain still fails. |
| Wrong declared interval/order | `exports:21–26,52–54,63–76` | Inventory and next-height extraction do not depend on the declared output list. |
| Next DATA only in ancestry | `exports:21–22,41–54`; main `2248–2252` | A complete same-session covering receipt can yield the next ancestor. Earlier sessions still require their own receipt. |
| Extra malformed independent object | `exports:23–30,56–61` | A known frame boundary isolates it. A bad proof cannot veto a separate complete proof. |
| Duplicate receipt/signature encoding | `exports:47–61,73–75,104–109` | Identity is signed chain/session/hash, and only an actual new DATA transaction counts. A failed first encoding does not create a permanent rejection marker. |
| Signed closed-version result | `exports:87–98,188–194` | Correctly rejected for current append, even though its historical evidence can remain valid. |
| Genuine missing predecessor | `exports:74–75,89–94,188–194` | Cannot skip a DATA height or retroactively reopen a completed past record. No bounded acceptance retry is inferred. |

The scan's ordering chooses among proof encodings, not between authorized forks: the current incoming boundary fixes the producing session, and the proof invokes regional receipt safety to make valid next DATA unique (`exports:78–82`). This remains conditional on the separately audited regional/retirement safety premises; this transport audit does not independently certify those premises.

### Finding T1: the repair's carrier coverage is not established by the frozen text alone

**Text checked:** `exports:15–20`: “The existing EXPORT body and dependency syntax has two finite sequences,” each object with its own decodable type/length boundary. `exports:32–39` applies that interpretation to v3.5 records. `exports:200–224` makes it the meaning of “carrying” for the progress proof.

The frozen v3.1 specification at `baseline/sync_hotstuff_rr_part2_rotation_v3_1.tex:1338–1357` defines the semantic fields of an EXPORT and its evidence; it does not define a serialization grammar, per-object frame boundaries, or how a decoder must find nested evidence after malformed enclosing metadata. The supplied snapshot also contains no independently frozen v3.5 encoding definition against which the backward-compatibility assertion can be checked.

Similarly, `source_step:14–35` requires dependencies to be inline or identified by exact commitment **and source position** in a common public basis. That is a sound determinism rule, but the baseline allows receipt/validation references to a validated local cache (`baseline/...v3_1.tex:798–801,887–896`) and does not state this exact public-reference representation. The audit therefore cannot infer equivalence merely from byte equality of the Model section.

This gap matters specifically to “already recorded”: if the old contract regards a previously carried complete result as delivery-complete but the new supported decoder does not inventory its encoding or references, no bounded corrected publication follows. Stating that unsupported carriers are excluded makes the narrowed scope visible; it does not prove that no previously permitted complete carrier is excluded.

There is no need to count omitted required ancestry as a legitimate complete result: it violates the frozen semantic definition. Nor have I shown that arbitrary ciphertext must count as a result. The unresolved obligation is narrower and more precise: a format/refinement argument connecting every permitted old complete carrier to the newly inventoried framed objects and public dependency predicate. Without that argument this is a **Justification Gap**, not an established false theorem or a fabricated supported-syntax attack.

## 3. Canonical public dependencies and cache independence

**Text checked:** `source_step:14–35`: the basis is the authenticated source prefix before p and its authorized checkpoint; private newer checkpoints do not change it; earlier rejected raw records may provide evidence.

Conditional on that common basis and specified framing, the completeness rule is observer-independent. A private cache cannot turn an absent public dependency into a valid candidate; a cache miss retrieves specified bytes without choosing a different DATA or timestamp. Raw historical evidence can be reused without retroactively accepting its DATA. A hash/URL/future authority name alone is insufficient.

**Text checked:** `source_step:37–54`; `exports:188–196`.

The distinction between a malformed/incomplete candidate and missing canonical source/boundary evidence is explicit. The former is skipped without blocking other complete closures. The latter pauses the canonical operation. Checking signed domain and open-version permission before deep archival validation prevents irrelevant closed-version dependencies from creating a public replay wait. These rules address private-cache-dependent acceptance and arbitrary future-authority fetching by malformed batch items.

This is a protocol refinement, not inherently an environmental strengthening. The outstanding issue is the **coverage** of this new predicate by the old delivery contract, as recorded in T1, and the time needed to obtain/validate its dependencies, as recorded in T2. A bounded operation cannot be inferred solely from a finite scan when its admissible size/load envelope has not been instantiated.

## 4. Terminal acceptance, exposure, and cursor authority

**Text checked:** `exports:104–120`: “Durably expose the updated G and accounting before attempting a later DATA or successor operation.”

Each accepted DATA atomically changes G, the producing session count, window count where applicable, regional credit, and owed discharge. Invalid later material cannot erase an earlier accepted DATA or its credit. The saved pre-payload eligibility/owed snapshots are unchanged. This repairs whole-EXPORT rollback as an obstacle to the first growth event.

**Text checked:** `exports:122–159`: terminal commit “requires the completed session's incoming boundary and authority, not AUTH[r+1,0].”

For quota one, the very first accepted DATA commits before target authority, `PrepareOwed`, and successor selection. A missing next-slot authority can therefore block later replay but cannot withhold the already achieved first growth. No result-completeness test is allowed to eagerly require the next authority. The independent source-handler and observer rules repeat this ordering (`source_step:49–54`; main `2136–2184`).

**Text checked:** `exports:161–186`; main `1408–1434`.

The durable cursor includes block, record, number of DATA commits, and stage; a pending boundary retains its canonical resolution position. Global finality is defined at the committed DATA subtransition, not at completion of the outer record/block. Replay resumes without rerunning counts or prelude, while prior DATA remains exposed. This directly removes a terminal-successor wait from the public-response conclusion.

**Text checked:** main `1861–1940`.

`VerifyHandoffHistory` establishes historical authority without demanding that the old version remain live; `CanEnter` separately requires current permission and a fresh local entry. A retained old result can therefore validate as history while its producing session cannot reopen. Current EXPORT append permission is checked independently. The separation avoids rejecting retained-anchor evidence merely because a Force closed its producer.

## 5. Causality and the strict Force cutoff

**Text checked:** `sections/rotation_v3_6_liveness.tex:205–214`: a correct FINAL signer validated a finalized selection reference before signing/committing the complete result.

For a result formed in this retry, its carrying finalized Espresso block cannot be the already finalized reference block containing its own future result. Immutability and the correct signer's causal validation order therefore place it after the reference and, for a Force-created retry, after the opening block. This argument uses the actual signed result, not honest publisher hints. A privately assembled receipt still includes at least one correct FINAL signer.

**Text checked:** `control:207–264`; `exports:95–98`; `liveness:243–246`.

Independent controls precede the first matching MSIG; otherwise AUTO settles delivery before payload. A Force retains pre-payload G and rejects new-version DATA in the triggering block. If the carrying block itself closes the retry, the public-response proposition's closure alternative holds; its payload cannot rescue the old version. No deadline-equal or same-block acceptance loophole was found in this branch.

## 6. Public-response budget and semantic assumption comparison

**Text checked:** `source_step:83–117,136–166`; `liveness:164–185,237–243`.

The new handler does remove an unconstrained protocol queue: source availability immediately relays an authenticated complete packet, ordered reception resumes predecessors, and ready state enters in the same serialized event. Those are genuine behavior refinements. They do not add a READY certificate, a second publication obligation, or a separate network round for each predecessor.

Given the bounded complete-handler interpretation, the displayed budget is coherent: opening-source availability costs at most T_E, its regional handler at most Delta_i; R_i starts from first actual Enter and already covers the required regional entry/protocol work; complete-result transport costs T_pub + T_E; the resulting complete source handler costs another Delta_i. Thus `2 T_E + 2 Delta_i + R_i + T_pub` is a conservative envelope. ReadyBundle's all-member entry interval is a premise used inside R_i, rather than a third independent allowance that necessarily has to be added to this expression.

### Finding T2: the interface condition is disclosed, but not derived from the frozen model

**Text checked:** `ready_bundle:112–144` explicitly makes the entry-skew implication conditional and states that residual installation delay u_v is not uniformly bounded. `source_step:168–178` excludes serialized snapshots requiring import, copying, or runtime initialization. `liveness:237–242` says the complete replay/normalization transaction remains an implementation obligation.

The frozen network premise (`baseline/model_and_assumptions.tex:167–179`) bounds regional delivery and processing of complete validation dependencies. It does not independently establish that all canonical replay, database/state adoption, durable state changes, or per-retry native continuation work are just those bounded validation operations. The frozen native-state premise is eventual; it is not a uniform per-retry installation bound.

A model-permitted native adapter could need finite but unbounded work tied to a newly known session/branch after reading the opening. Retaining a recovered anchor does not by itself prove that this work disappears. The candidate's directly executable persistent object and atomic attachment contract excludes that case, explicitly. Likewise, collecting every source predecessor by a common availability deadline proves availability; finishing all required replay within one Delta_i also needs the whole handler to lie inside the stated processing envelope. Calling steps atomic specifies their ordering but, without the abstract processing interpretation, does not independently supply a physical-time bound.

These observations do not refute the theorem **conditional on that interface**. They do prevent claiming that the interface's feasibility/timing has already been proved from the unchanged Model section alone. The missing implication propagates from `prop:v35-public-response` to the fixed-L contradiction in `prop:v35-global-progress` and then to repeated regional service. No independent downstream proof replaces it.

## 7. Other liveness premises and what is legitimately a protocol change

**Text checked:** `liveness:41–54,70–107`; main `1726–1742,1767–1799`.

In a genuinely stalled frontier, zero accepted output keeps the same authority slot after at most one positive-duty switch. A persistent sequential authority fetch, fixed beta-independent schedule, and retained exact anchor allow a finite required authority prefix to be prepared eventually. Taking a maximum over a finite installed roster does not impose a uniform deadline over all future slots. This is different from assuming bounded L1 authority production. It still needs actual eventual availability and reusable prepared state, as explicitly stated.

**Text checked:** `health:3–8,18–45,120–177`.

The finite intervention journal is an algorithm for the already honest coordinated committee, not an assumption that the health reports are accurate. Credits do not replenish on Force or raw publication. At a fixed accepted frontier there are only finitely many new credit keys, and each outstanding control keeps its original semantics. Legacy backlog/restoration is separately limited. This is a legitimate restriction of correct protocol behavior; honesty alone in v3.1 did not already imply it.

**Text checked:** `control:166–175`; `liveness:267–309`.

Conditional cooldown prevents AUTO from newly excluding its last eligible owed source. With all selectable regions meeting the regional serving premises and finite preparation complete, consecutive retries can be immediately reassigned. Under a fixed L, the displayed timestamp increments accumulate quadratically in retry number while real time accumulates at most linearly; the frozen upper-lead inequality then contradicts infinite zero-output retry. This algebra does not need a lower timestamp-lag bound or a periodic empty-block assumption. Its crucial premise is the uniform public-response bound; T2 therefore matters directly.

Service input-size/load/admission conditions remain external. The document does not prove that a concrete deployment's accumulating history, dependency bundles, SourceStep traffic, and native validation stay within its chosen service envelope. This is an implementation/scope obligation, not an independently demonstrated violation of the frozen idealized network assumption. No fee-free path, universal archive-latency guarantee, or unbounded-throughput claim should be inferred from the constants.

## 8. Objective coverage

### S1: all selectable regions versus only one surviving region

**Text checked:** `liveness:156–165,312–318`: L is a maximum over “regions that can be selected,” and the conclusion “does not extend the model to permanently missing correct quorums in other assigned snapshots.”

The proof is about model-compliant regional snapshots, not arbitrary unresponsive regions alongside one good region. This distinction is consequential with owed selection. After a good region discharges its debt, a permanently dead but governance-ON positive-weight region can remain owed; health leases eventually restore ON and AUTO does not discharge debt. The working region's mere existence is not the response premise for the selected dead region. The draft does not claim to handle that broader environment, and it supplies no reduction from it to the all-selected-regions proof.

Permanently dead correct quorums would violate the assumed regional communication/serving conditions, so this is **not** a Critical Error or an in-model adversarial trace. It is an explicit uncovered goal requirement. The theorem's “nonempty set ... able to serve” wording must be read with the later scope paragraph and the maximum actually used by its proof.

### S2: accepted regional credits do not imply request inclusion

**Text checked:** `liveness:394–436`.

The acknowledged trace is consistent with the separation of authority availability, receipt formation, and global cutoff. With Delta = 1, a correct recovery initializer entering at d can wait until d+2, obtain anchor votes and propose at d+4, and obtain the other correct DATA vote at d+5. A Force crossing at d+5.5 precedes the resulting d+6/d+7 timer commitments and complete receipt. Retirement may allow those already started timers to finish, but cannot restore the closed version's global append permission. The earlier accepted empty DATA makes the next Force advance the slot, so each attempt can encounter a new eventual-but-unbounded authority wait.

Thus repeated accepted DATA and regional debt discharge do not establish eventual global inclusion of the oldest pending request. The manuscript correctly leaves that property open. A final project verdict must retain that limitation even if T1 and T2 are resolved.
