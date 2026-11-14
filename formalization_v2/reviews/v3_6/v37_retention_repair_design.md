# v3.7 design: retain public evidence bytes and expose the remaining L1 dependency obligation

## Status and scope

This is a protocol-design note, not a revised manuscript or a completed liveness certification. It answers the old-public-object cache-miss finding in `fresh_liveness.md`. All source references below are relative to `snapshot/`, unless explicitly stated otherwise.

**The concrete retention change below eliminates the reported old Espresso object `X` retrieval trace for prepared, continuously online regional members. It does not, by itself, prove that every operation before first DATA acceptance is dependency-complete.** A distinct L1-authority case remains under the v3.6 public validator: a genuine current-session FINAL root can be accompanied by an alternative, invalid proposal/admission proof whose first undecided test requires a previously unseen authority beyond the prepared finite prefix. Sections 7–9 identify that obligation rather than assigning this fetch a fictitious `T_E` bound. Section 11 assesses the root's subsequent, stronger proposal: derive intrinsic DATA validity from the existing honest-FINAL guarantee and remove publisher-selected admission paths from the public validator's required dependencies. That explicit algorithm change can resolve this corner without a new environmental bound.

No uniform archive, L1 AUTH, native reconstruction, storage import, or external-observer processing bound is added. `T_E`, `T_pub`, and `Delta_i` retain their frozen scopes. Initial hydration is eventual and finite for a fixed finite prefix; the quantitative response argument starts afterward. There are no edits to the frozen Model section or the manuscript in this task.

Relevant source points:

- `sections/rotation_v3_6_source_step.tex:4–5,14–35,75–79,138–160`: a retained index is currently substituted for retained object bytes in the source-packet timing argument.
- Main `436–472`: old history is retrievable, while `T_E` concerns the newly available record/namespace and Espresso evidence.
- Main `395–397,926–935`: the regional bound processes complete validation dependencies; it does not acquire arbitrary missing historical objects before a complete send exists.
- Main `1733–1742` and `sections/rotation_v3_6_liveness.tex:60–96`: persistent fetching covers each finite required prefix of a fixed slot's authority sequence, without a uniform delivery constant.
- Main `614–650`: DATA contains both an L1 header/registry reference and an Espresso history reference. These are different source types.

## 1. Persistent state: content, provenance, and replay progress are separate

Add a node-level `PublicEvidenceStore` shared by the source reader, interpreter, regional engines, manager, and publication workers. Its lifetime is not a session lifetime. Every reference counted below has the full chain/protocol domain, source identity, and object commitment; an index entry without its content is not a retained object.

| State | Required retained content | Role |
|---|---|---|
| `ERaw[q]` | Complete relevant namespace bytes, or authenticated absence; full header; finality and linkage evidence; canonical transaction/record order | Preserves all actual source bytes, including malformed or rejected EXPORTs |
| `EObject[source-position, frame-path, commitment]` | Actual framed-object bytes and a binding path to `ERaw[q]`; optional parsed representation | Finds an exact old public object without an archive read |
| `L1Raw[id]` | Every obtained finalized L1 header, authority, control record, registry/completeness proof, and its authentication dependencies | Preserves L1 objects once obtained; does not claim that unreceived L1 objects are available |
| `AuthCache[r,g]` | Full immutable AUTH and verification evidence, not merely its hash or the sequential fetch cursor | Supplies fixed-slot recovery authorities after preparation |
| `ValidatedBundle[statement-id]` | Full DATA, proposal/admission witnesses, ancestry, native evidence and source proofs used by a correct validation/commit or complete receipt relay | Prevents an existing genuine regional witness from requiring reconstruction or refetch at its later send |
| `AnchorPrepared[anchor-id,region,format]` | Exact executable continuation object and its full validated provenance | Supplies the already prepared Ready attachment, without importing old consensus state |
| `ReplayCheckpoint` | Exact canonical subposition, public interpreter state, applicable-control state and its proof closure, including pending-boundary stage | Fixes the deterministic continuation, without replacing raw history |
| `RetainedCoverage` | Exact authenticated source intervals and checkpoint dependency objects whose bytes are resident | Allows a checkable hydration condition; does not itself constitute bytes |
| `PendingFetches` | Typed unresolved source requests with original canonical position and target identity | Supports eventual resumption; never erases a real wait |

`ValidatedBundle` and `L1Raw` may contain objects absent from a particular EXPORT's permitted public dependency basis. Keep **possession**, **authentication**, **regional validity**, and **public admissibility at record position p** as separate predicates. A private bundle can supply the bytes of an already admissible exact public reference; it cannot invent that reference's public occurrence or change the meaning of an earlier incomplete record.

The source-availability cursor and interpreter cursor are different. The former can record a fully retained namespace while the latter remains paused on a boundary. Neither cursor advances past a durable byte insertion that it purports to certify.

### Required retention actions

1. On acquisition of a complete new Espresso namespace and its source evidence, retain the **entire raw namespace** in the source-availability event, before advancing its availability cursor, sending a packet that relies on it, rejecting any record, or releasing the input buffer. Retain rejected records and independently framed objects inside invalid containers. Do not wait for P6 acceptance to decide what to keep.
2. On acquisition of a finalized L1 source object, retain its full bytes and authentication proof before recording the corresponding cache hit or advancing its fetch cursor. Preserve pending controls as well as applied controls. Do not make Espresso ingestion conditional on eagerly obtaining every arbitrary L1 name appearing in its payload.
3. On successful regional validation, retain the dependency-complete witness actually used. Correct proposal/FINAL/receipt senders form their required complete bundle from this retained witness. A newly constructed reference is allowed only under the existing rule that the receiver can resolve it within `Delta_i` (main `933–935`); otherwise send the bytes.
4. On beta replacement, session closure, completion, termination, or publication-worker exit, detach the consumer but keep these node-level objects. Transfer outstanding sends and retention responsibility durably before resource release. A tombstone or `Closed[beta]` does not discharge the store's obligations.
5. A restart restores the store and exact replay subposition. A process that lost required bytes must rehydrate before claiming the prepared continuously-online timing guarantee. It may not claim that an existing index makes it prepared.

This is explicit protocol storage behavior. It is not a new claim that all physical disks or storage APIs finish arbitrary work within a constant time. The abstract message implementation must represent these already retained bytes as validation inputs; an implementation that performs an external archive read or native import at this step has not implemented the prepared interface.

## 2. Initial hydration: choose a finite cut, not a moving target

Choose a fixed authenticated source cut `q0` at which the node begins the new retention discipline. Hydrate:

- every raw namespace/header/evidence object in the finite historical interval that remains legally referenceable from future EXPORTs;
- every object designated by the starting checkpoint, with its source identity and complete bytes;
- the exact replay and anchor provenance needed at that checkpoint;
- the finite fixed-slot authority prefix and executable anchor state used by the prepared-tail witness.

Under the current rule allowing references into arbitrary earlier rejected raw records, the first item generally means all relevant raw namespaces back to the authorized source origin, **not only the objects referenced by accepted DATA**. A checkpoint containing only `G`, cursors, or an index is insufficient. If a checkpoint deliberately exposes a smaller legal reference basis, that restriction must already be part of the authorized protocol semantics; it cannot be inferred from local garbage collection.

Continue retaining every newly available namespace after `q0` while hydration runs. The startup task never restarts at beta change and never chases a new cut in place of completing the old finite cut. Hydration is complete when the fixed old interval and checkpoint dependencies are resident; subsequent inputs are covered by ongoing retention.

For a fixed finite source interval, there are finitely many finite admitted namespace objects. Their eventual retrieval gives a finite completion time by taking a finite maximum. The fixed anchor and finite AUTH prefix have their separate eventual-availability premises. Taking their maximum defines a finite preparation witness. It gives no numerical constant for archive access or L1 authority delivery and no uniform preparation time for all future slots.

Do not expand this finite maximum to “every future AUTH that some adversarial proof may name.” That set is not the finite set in the existing regional-response proposition.

## 3. Checkpoints and pruning

The immediately safe policy is **no semantic deletion of publicly referenceable raw bytes**. Storage can grow with the admitted source history. This note proves no finite-storage bound.

Allowed representation changes are deduplication of equal immutable bytes, lossless compression with the same abstract-access contract, replacement of parsed indexes by a reconstructible index, and deletion of duplicate physical copies while at least one dependency-complete logical copy remains in the node's prepared store. Retain source-position and frame-path mappings when equal bytes occur at multiple positions; source order is part of admissibility.

A checkpoint may compact replay computation. It does not erase an old raw object's future evidentiary use. For every object it replaces, retain either the exact bytes and authentication path or a protocol-authorized proof object that is explicitly sufficient for **every** later allowed use. A commitment alone is insufficient when a later closure needs the committed bytes. An external archive that is only eventually reachable is insufficient for the prepared timing invariant.

Before deleting any object, establish all of the following: it is not legally referenceable by any future supported record; no retained checkpoint/boundary/anchor/receipt or pending validation uses it; no live or retired engine's evidence/publication duty uses it; and no promised complete message depends on it. Current v3.6 gives no general finite horizon satisfying the first condition for earlier raw records. Therefore ordinary session retirement or an advancing `G` is not a pruning proof.

## 4. Source packet formation must not prevalidate arbitrary candidate proofs

Replace the ambiguous all-dependencies packet requirement with a staged, typed construction:

```text
OnEspressoAvailable(q, header, namespace, sourceProof):
    authenticate the source envelope and complete namespace/absence
    persist ERaw[q] and its source evidence
    index actual locally framed bytes; do not chase arbitrary external names
    retain any supplied L1/evidence bytes with their own provenance
    send SOURCESTEP(q, exact raw input, maintained checkpoint identity,
                    available applicable controls and fixed-boundary evidence)
    invoke ordered replay when the current operation is dependency-complete

OnAdditionalDependencyAvailable(id, bytes, proof):
    authenticate and retain the exact typed source object
    satisfy only matching PendingFetches / complete-message dependencies
    resume the saved canonical operation; do not restart prior commits
```

All required old Espresso references are available through `ERaw/EObject` after hydration. The packet can transmit their bytes or use the existing bounded-reference contract when recipients already maintain the same retained basis. A sender must not omit bytes merely because it has a cache hit and hopes recipients have one. Newly active or returning recipients outside the hydrated cohort use full packets or initial catch-up; their backlog is not covered by a constant `Delta_i` claim.

The immediate raw-source relay is an actual send even if some later candidate-local L1 authority is missing. It preserves availability and prevents packet formation itself from fetching the union of all adversarial proof branches. **It does not magically make the eventual replay operation complete.** A dependency-complete packet/operation is claimed only after all dependencies of that operation have been supplied or already retained under the existing complete-message contract.

Canonical prelude/boundary dependencies remain mandatory. In particular, do not classify missing applicable controls or the designated current `AUTH[r*,0]` as an incomplete unrelated candidate. `CommitNextDATA` still precedes `ResolveBoundary`: successor `AUTH[r*+1,0]` is never included as a prerequisite for the first terminal DATA commit.

## 5. Candidate order: reject irrelevant material before external acquisition

Preserve independent framing, all embedded DATA/receipt inventory, and the rule that false outer hints do not veto a valid result. Inspection of a rejected outer root must still discover independently framed neighboring roots.

For each actual proof root, use the following order:

1. Parse the root and its supplied DATA bytes; check types, signed domain, hash consistency, current beta, assigned pair/region/configuration, quota/height range, and compatibility with the current fixed duty. Decode referenced old Espresso frames from the retained store. Reject a syntactic cycle or an absent/future Espresso source position using the common authenticated source basis.
2. Check the root's FINAL signatures, distinct signers and weight `>F_i` under the already prepared fixed snapshot `AUTH[r*,0]`. This step needs no recovery leader schedule: FINAL has no generation/term field. Do not fetch `AUTH[r*,g']` to determine whether a root with insufficient or invalid FINAL signatures is relevant.
3. Identify required evidence by the validation schema and signed object identities. An unsolicited dependency list does not itself make each listed object necessary. Independently validate all complete eligible proof choices; an invalid extra path does not poison a different complete proof. Optional validation of closed-beta or wrong-duty objects is an archival task and cannot stop current replay.
4. For an exact Espresso public reference, perform a retained-store lookup and verify source membership/precedence. A missing legal historical object means the prepared invariant failed and requires rehydration; it is not a fresh `T_E` promise. A reference absent from the common basis is an incomplete proof choice, regardless of a private cache hit.
5. For L1 references, distinguish already retained finalized bytes, supplied bytes with complete source authentication, and a genuinely unresolved source object. Do not infer availability of the third case from Espresso visibility or the current slot's finite recovery prefix.
6. If full evidence for the current candidate is present, perform all original authority, admission, ancestry, reference, native validity and boundary checks, then commit or reject. Preserve evidence-first handling for genuine Quit material independently; a bogus proof with absent admission evidence is not a genuine Quit.

Steps 1–3 eliminate many arbitrary-fetch attacks, including fabricated roots, foreign-session roots and stray dependencies. They do **not** prove that step 5 is never reached with a real cache miss. A genuine public dependency missing locally must retain its canonical pending position under the current semantics, not be silently treated as invalid because of arrival order.

Do not “repair” the latter by completing a record and revisiting it later: `exports:188–193` deliberately forbids retroactive acceptance. Do not choose which proof is publicly complete from the current cache contents.

## 6. What the byte-retention induction actually proves

Define `EComplete(q)` for a prepared regional member to mean that all full raw Espresso objects legally referenceable from processed positions through `q`, their source authentication, and checkpoint-resolved exact objects reside in its logical store.

**Base.** Fixed-cut hydration establishes `EComplete(q0)` before the timed tail. This includes rejected raw records, not merely accepted ancestry.

**Step.** On acquisition of `J[q+1]` and its complete namespace, retain the bytes before advancing the source-availability cursor. Every reference to an earlier position is resolved from the inductive store. Every inline current-record object is in the newly retained namespace. Every absent/future reference is classified from the authenticated position/basis. No beta change, worker exit or checkpoint compaction deletes this content. Therefore `EComplete(q+1)` holds.

**Timing consequence.** If `J[q]` becomes final at real time `a`, all predecessors finalized by `a` have their newly available Espresso bytes and evidence delivered by `a+T_E` to the covered history-following members. For objects before `q0`, bytes were already hydrated. For subsequent objects, insertion preserves the bytes at their original availability event. Hence every **Espresso-sourced** exact public reference needed through `q` is available from the prepared store by `a+T_E`. The old `X` trace cannot occur: there is no permissible transition that replaces the only retained bytes by an eventual archive pointer.

This induction says nothing about an independently finalized L1 object not yet acquired. It also does not convert a retained hash into a full native executable object or give a wall-clock bound for arbitrary physical I/O.

## 7. Dependency audit before the first positive current-pair DATA

| Operation/dependency | Why it can be complete in the stalled prepared tail | Unresolved qualification |
|---|---|---|
| Opening namespace, earlier raw objects, source headers/proofs | Section 6 induction plus `T_E` for newly available Espresso inputs | Requires actual hydration and full-byte retention |
| Applicable L1 controls and completeness evidence | Frozen `T_L1` service and advance announcement, then retention | Controls are not all L1 authority/header objects |
| Fixed current boundary and `AUTH[r*,0]` | Finite fixed-slot preparation | No assertion about every future slot |
| Fixed anchor's provenance and executable continuation | Finite preparation, retained full witness and direct executable object | External database import remains outside `Delta_i` |
| Honest first-result path through `g_i*` | Finite authority prefix plus original regional complete-bundle rules | Finite `g_i*` covers the genuine path, not every malicious alternative proof |
| New valid DATA's L1 reference header/registry proof | Correct validating regional senders possess it and must relay complete validation dependencies | It is not obtained from `T_E`; identify the actual correct regional send |
| Foreign/closed-beta/fabricated receipt root | Early context and FINAL-threshold gates | Embedded independent current-pair roots must still be inspected |
| Authentic current-pair root with alternate proposal/admission proof | Genuine regional evidence exists and is relayable | Public witness substitution or rejection-before-fetch has not been established |
| Terminal successor authority | Not needed before first durable DATA commit | `ResolveBoundary` waits afterward |
| Nonregional observer's application/authority validation | Deterministic eventual replay | No regional multicast deadline for this observer |

For a genuine root already formed by time `b`, a correct signer committed its DATA before signing. The v3.6 Commit rule sends FINAL to the fixed region and the collector (main `1127–1131`); the generic complete-validation rule requires correct senders to supply resolvable complete dependencies (main `926–935`). Complete receipt holders also multicast the full result (main `833–839`). Make the individual FINAL payload explicit in the revision: it carries the full retained validation witness or references already covered by the existing regional delay contract. This is a send rule for evidence already held, not an L1 fetch deadline.

Thus a genuine witness reaches the correct members of that **same fixed region** by `b+Delta_i`, under the original complete-message contract. This is useful even when its L1 reference or generation exceeds a precomputed cache prefix. It does not deliver that witness to arbitrary observers or other regions on the same deadline.

Two care points prevent overclaiming. First, v3.1 receipt prose says FINAL goes to its region (`baseline/...v3_1.tex:791–799`), while its Commit display names the Publish Layer (`1067–1071`); v3.6 explicitly specifies both destinations. Do not claim the Commit displays are identical. Second, obtaining a genuine regional witness does not establish that this witness is permitted in the particular public closure being interpreted.

## 8. The surviving L1 alternative-witness obstacle

The following is the exact missing argument, not a claimed permanent-stall attack:

1. The DATA value `B` contains `(beta,r,k,h,type,x,parentHash,rho,xi)`, without `g,t` (main `605–608`). Its PROPOSE signature separately binds `g,t,j` and parent justification (main `1092–1094`). FINAL signs the DATA hash and session fields without `g,t` (main `826–868`).
2. Therefore authentic threshold FINAL signatures for `B` do not authenticate every proposal/admission witness a publisher chooses to attach to that same `B`.
3. A publisher can present those genuine signatures with an alternative purported proposal/admission path naming `AUTH[r*,g']` beyond the finite prepared prefix. The path may include an authentic Byzantine-member signature whose eligibility as leader cannot be decided without that authority. All supplied Espresso-side raw objects can be retained already; the undecided object is L1-sourced.
4. The root's context and FINAL threshold pass the early gates. No text read here proves that every such invalid alternative can be rejected before its first uncached L1 authority lookup. The finite-prefix lemma does not bound this lookup.
5. The correct signer's genuine validation witness is available through regional relay, but may differ from the particular path exposed in the record/common public basis. Current `source_step:14–35` prohibits private-cache completion; `exports:56–61` invalidates a bad required closure and permits another **complete** proof. The new regional witness is not automatically another publicly complete proof.

A proposal to use any genuine witness for the signed DATA statement may be reasonable, but it needs an explicit semantic refinement: which source identities must already be public at `p`, which bytes are only proof representations, and why replacing the representation preserves the exact old record's completeness and deterministic acceptance. The rule must not let a later regional-only packet turn a previously incomplete earlier EXPORT into an accepted historical record.

Three tempting shortcuts are invalid:

- “All current-pair authorities are cached by preparation”: only a finite genuine prefix was prepared; adversarial paths are not bounded by `g_i*`.
- “The new EXPORT is available by `T_E`, so its named AUTH is too”: L1 supplies its own authority evidence; no uniform AUTH-delivery bound is frozen.
- “Skip an uncached authority and continue”: this makes public completeness depend on local acquisition order unless the protocol first proves that the proof choice is publicly incomplete or irrelevant. If a later result becomes newly accepted at a different source position, window/Force behavior can change.

The safest present change is to retain bytes, implement early gates and explicit full-evidence sends, but leave this case as a genuine wait until the carrier/witness refinement is justified. This preserves correctness and eventual resumption while withholding the old uniform response claim.

## 9. Exact timing scope and conditional accounting

The complete-message operation charged to `Delta_i` must be listed explicitly: authenticate the regional envelope; check included source evidence; resolve already resident exact references; inspect evidence; decode finite admitted frames; execute the dependency-complete relevant validation; run the canonical prelude and per-DATA transaction; durably expose its cursor/counts; and invoke the guarded Manager transition with an already executable anchor object. The existing abstract contract must cover this operation under the stated input/load limits. No extra archive request, external authority lookup, replay of an arbitrary offline backlog, database import or native reconstruction occurs inside this charged step.

Retaining bytes removes an acquisition wait, not all computational costs. The source does not state a general CPU/IO bound for arbitrary accumulated history. Do not prove that an unlimited number of historical transitions fits one `Delta_i` merely by calling them atomic. The quantitative lemma remains limited to the maintained, history-following input path with the stated complete-message interpretation; a concrete implementation must establish that its admitted packet/batch handling meets that interface. Returning-node catch-up is excluded.

If a subsequent witness refinement proves all rows of Section 7 dependency-complete from retained public inputs by each source deadline, the original accounting can be reused:

`L_i = 2 T_E + 2 Delta_i + R_i + T_pub`.

That is a **conditional** formula, not a result established by the storage induction alone.

If the justified refinement instead needs a genuine FINAL signer's regional evidence to finish source assembly, charge that path explicitly. For a root carried in a block finalized at `a`, it existed by `a`, so its genuine correct-signature bundle can reach the same region by `a+Delta_i`. Espresso bytes arrive by `a+T_E`. A conservative complete assembly time is `a+max(T_E,Delta_i)`; an additional complete source relay/handler can then finish by `a+max(T_E,Delta_i)+Delta_i`. The looser allowance `T_E+2 Delta_i` is safe for this stage. It does not presume `Delta_i <= T_E`.

Likewise, for the chosen complete result first offered at `s`, the full-receipt regional relay finishes by `s+Delta_i`, while its newly published source input is available by `s+T_pub+T_E` (or `s+T_E` when already recorded). The relevant complete inputs are available by the maximum, not by assigning the regional leg to `T_E`. A conservative design that charges the extra relay on both opening/prefix and result paths would use

`L_i' = 2 T_E + 4 Delta_i + R_i + T_pub`.

Even this larger number is conditional on public witness equivalence and complete preceding operations. Adding `Delta_i` cannot repair an arbitrary future AUTH fetch or a forbidden private-witness substitution. The finite maximization over the installed region set remains valid only after these individual dependencies are established.

State the timed outcome for continuously online hydrated members of the relevant fixed regional snapshots. Other online observers receive the bounded Espresso source data, and deterministic replay yields the same prefix once dependencies are verified, but the present regional relay gives no matching wall-clock application-validation guarantee to those observers. The wider quantifier in `liveness:153–154` must be narrowed or supported by another actual path already covered by the frozen model.

## 10. Proposed manuscript edit map and completion criterion

Without changing the frozen assumptions, a subsequent revision can make these localized edits:

1. `source_step:4–5`: replace cursor/index-only retention with Section 1's full-byte store and cross-lifecycle ownership rules.
2. `source_step:14–35`: separate exact Espresso object lookup, independently authenticated L1 source objects, and private regional validation evidence. State common source admissibility separately from local possession. Resolve the alternative-witness issue before claiming this paragraph is dependency-complete.
3. `source_step:75–91`: prevent packet assembly from waiting for the union of arbitrary candidate proof dependencies; specify the immediate raw-input relay and the separate complete-operation event.
4. Receipt/Commit rules at main `833–839,1127–1131`: explicitly bind each correct full-evidence send to retained validation bytes and its immediate dispatch event. This introduces neither a new public record nor a new service assumption.
5. Preparation and lifecycle paragraphs at main `1727–1742,2090–2115`: include fixed-cut hydration and permanent retention responsibility in retry/exit behavior; keep native preparation outside the bound.
6. `source_step:138–178` and `liveness:149–245`: replace the blanket source-deadline inference with the Section 6 induction and the complete Section 7 dependency case split; narrow regional scope and charge any real additional relay.

The repair is complete only when the proof can enumerate every operation before the first accepted positive DATA and assign each required object to one of: retained fixed preparation, new bounded Espresso bytes, advance-announced control bytes, or an identified complete correct regional send whose public use is justified. A residual “fetch the specified L1 object and resume” on an adversarially chosen proof branch fails that criterion.

**Retention-only conclusion:** the explicit retention algorithm repairs the reported Espresso `X` cache-miss mechanism and gives a source-byte invariant. The complete public-response theorem, downstream adaptive-window contradiction and repeated accepted-service conclusion remain conditional until the L1 alternative-witness issue and the precise complete-handler scope are discharged. Section 11 supplies a promising additional protocol refinement for the former. Request inclusion remains the separate limitation already identified by v3.6.

## 11. Additional public receipt-kernel refinement proposed by the root

This section was added after the retention-only analysis. It is a different repair from importing a later private witness into an old public record. It changes which evidence the **public** validation algorithm needs, using a lemma already entailed by the frozen fault and regional-correctness assumptions. The regional Vote/admission/equivocation rules keep all their original full-evidence requirements.

### Honest-FINAL validity lemma

Fix the actual boundary-selected session and its authenticated fixed voting snapshot. Suppose a set of distinct valid signatures of total weight `>F_i` signs the same FINAL tuple for DATA hash `hB`, height and pair. Then at least one signer is correct under the frozen fault accounting, and the following chain is noncircular:

1. Signature unforgeability means that correct signer executed an actual FINAL send for this exact tuple.
2. A correct replica emits a fresh FINAL when a Vote-created timer commits DATA or its DATA ancestors (main `1127–1133`). A DRAIN timer emits FINAL for its terminal DATA ancestor, not for DRAIN.
3. Vote creates such a timer only after complete validation of the proposal and required ancestry, including authority/admission, reference, parent/quota and application-validity checks (main `926–935,1103–1124`). Therefore the signed DATA and its same-session ancestry satisfied those intrinsic predicates in the actual correct execution.
4. A commit obtained from an existing receipt emits no new FINAL (main `874–877`). Thus receipt validation cannot fabricate the foundational signature event. No induction of the form “a receipt is valid because another receipt said so” is used.
5. Binding hashes transfer the conclusion to the exact supplied DATA bytes whose recomputed hash is `hB`, and to the actual supplied hash-linked ancestor path. They do not authenticate a publisher's different proposal/admission sidecar merely because it names the same DATA.

The conclusion is an intrinsic valid-DATA statement, not an acceptance or freshness statement. A correct signer may have committed before a Force, and a valid old result may be globally ineligible. The public interpreter still checks its canonical incoming boundary, current beta, exact current `G`, pair/region/configuration, contiguous hash-linked ancestry, height and quota, current source position, all Espresso strict-precedence constraints, and the prelude/cutoff order. In particular, historical regional validity never substitutes for current append permission.

The lemma uses the original `>F_i` threshold and correct-validator assumptions. It introduces neither stronger honest weight nor a new attestor. Its use must nevertheless be declared as an explicit public algorithm revision: main `855–863` presently says the outside verifier checks receipt condition (iii) from detailed evidence. The revised operational explanation would establish that condition from the honest-FINAL lemma, while retaining the condition itself as a semantic property. Do not claim that the existing detailed-validation algorithm already has this behavior.

### Refined required public input

For a current-pair receipt root, the minimal public kernel can contain:

- actual root DATA bytes and actual DATA ancestry needed to link the same-session covering receipt to the next DATA and exact incoming anchor;
- the threshold FINAL signatures and their signed context;
- the actual canonical boundary and fixed `AUTH[r*,0]` snapshot, already prepared for this duty;
- the required source position and Espresso context for the independent public gates.

Admission witnesses, recovery leader AUTH records, proposer-selected QCs and L1-reference proof sidecars are then optional diagnostics for public receipt validation, not prerequisites of this kernel. A supplied invalid alternative cannot veto the valid kernel. The same-session DATA/ancestry **bytes** remain necessary; a threshold signature is not data availability, and an omitted payload cannot be replaced by an existential statement that the signer once possessed it. The source-basis/common-completeness rule still governs every required exact byte reference.

There are two necessary limits:

1. Intrinsic L1-reference and native-validity predicates can be certified only to the extent that the correct Vote actually checked them for the hash-bound DATA. If the public algorithm still insists on separately fetching arbitrary `rho` headers or AUTH recovery witnesses, the original acquisition gap can remain. State the intended reduction consistently instead of retaining that fetch behind a different label.
2. An unbound native sidecar, execution output, state snapshot, partial-EVM continuation or DA-to-output correspondence claim does not inherit validity from FINAL unless the signed DATA hash and the original validity predicate bind precisely that claim. Keep the frozen sound native/DA interface and the Faithful publication obligation (main `2210–2217`) for these claims. Do not make an unbounded `C_pub`/native publication operation a prerequisite for the logical `G` transaction. Initial executable anchor preparation and Ready verification remain separate and unchanged.

No private-witness substitution is needed: the public verifier uses the signature set and bytes already carried by the record or its admissible exact public references. The honest signer's private execution proves a theorem about those signed bytes; the verifier does not import a new external proof branch into the old record. Carrier syntax still needs its own compatibility audit, especially for any semantic requirement that old record completeness includes an unnecessarily specified bad sidecar; this refinement must state that public acceptance extracts the minimal actual valid kernel from independently framed material.

### Completing the dependency case split under this refinement

In a prepared stalled tail, before the first positive current-pair acceptance:

1. Source authentication, applicable controls and canonical boundary use the retained raw evidence, advance-announced controls and prepared fixed `AUTH[r*,0]`.
2. Inventory parses actual bounded source frames and resolves required old Espresso references from the full-byte store. It does not chase optional external admission paths merely to inventory them.
3. Foreign, closed, malformed and insufficient-weight roots are rejected with the already known current context. Embedded independent roots remain visible.
4. A surviving threshold root uses the honest-FINAL lemma for intrinsic regional validity. Its actual DATA hash links and independent global gates are checked using retained/public bytes and the prepared current boundary. A raw lookup that shows an actually absent required DATA byte gives a common incomplete-kernel result, independent of cache arrival order.
5. A root containing a positive valid current-session path either commits the first next DATA or is globally ineligible by the independent source/current-duty rules. There is no required `AUTH[r*,g']` lookup caused by its publisher-selected alternate admission proof. All optional incomplete proof material is retained for the separate regional/archival evidence handler; it is not a source-interpreter blocking dependency. A complete valid current-engine Quit proof still receives its original mandatory evidence-first handling.
6. The first terminal DATA commits before any successor AUTH lookup. Retained anchor/authority preparation handles opening and retry operations. No native reconstruction is placed before this logical first-DATA commit.

This closes the identified **dependency-acquisition** cases at the abstract level if the revised public kernel and its common carrier interpretation are adopted as stated. The original `2 T_E + 2 Delta_i + R_i + T_pub` stage accounting can then use the raw-retention induction: the proof no longer needs a separate regional leg to fetch a publisher-selected public validation witness. The genuine regional full-evidence sends remain necessary for original BFT processing, and are already accounted for in `R_i`.

The quantitative result still has the existing complete-handler, admitted-input/load, directly executable Ready object and regional-cohort scope from Section 9. This lemma does not prove arbitrary native-adapter performance, unrestricted CPU/history processing speed, a nonregional observer deadline, request inclusion, or the carrier-format compatibility argument. A final manuscript theorem should be stated only after those protocol interfaces and the reduction lemma are integrated consistently and freshly reviewed.

## Validation performed

Manual source comparison against the frozen v3.6 review, main dependency/receipt/admission/lifecycle/observer sections, complete source-step/exports fragments, relevant liveness/control/ReadyBundle fragments, and the v3.1 receipt/validation baseline. A separate focused L1 audit checked the distinction between the finite AUTH prefix, genuine regional evidence, and an adversarial alternative witness. No manuscript files were changed; no executable model, native-adapter timing test, or storage performance test was run.
