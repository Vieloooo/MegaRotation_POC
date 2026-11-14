# v3.7 carrier representation refinement design

This is a design response to T1 in `fresh_transport.md`, not a fresh verifier report. It does not edit the manuscript or frozen snapshot. Source paths below are relative to `snapshot/` unless otherwise stated.

## Conclusion

**A lossless normalization theorem is available for the frozen specification's logical EXPORT tuples.** It can preserve every complete regional-result subgraph in the DATA and evidence fields, including ancestry-only results, independently of outer chain/version/base/interval hints. This is a representation refinement and adds no network access or timing assumption. It does not require an already published tuple to be republished.

**The present v3.6 wording does not yet state that theorem correctly.** It asserts a pre-existing binary framing convention and restricts every non-inline dependency to an exact commitment and source position in a particular public basis. Neither convention is supplied by the frozen v3.1 text. Remove those assertions, define normalization on logical values, and treat a concrete codec's preservation of those values as an implementation obligation.

**This does not prove universal compatibility with unspecified legacy cache references or arbitrary Espresso namespaces.** The source supports an abstract complete-result transport interpretation, but it does not supply a deployed encoder, a universal reference resolver, or a discovery rule for arbitrary unrelated namespaces. A claim quantified over all such concrete interpretations remains a gap. The formal representation theorem must not be used to hide that gap by defining “carrying” to mean “the new decoder extracts it.”

There are therefore two defensible statements, with different scope:

1. The mathematical protocol operates on the logical EXPORT values already specified by v3.1. The new inventory is a conservative representation of those values. This can be proved now, without an environmental strengthening.
2. Every previously deployable carrier/reference encoding allowed by every reading of v3.1 is handled by this particular decoder at the old service deadline. This cannot be proved from the supplied sources. It needs a fixed old encoding/resolution specification and a compatibility proof, or an explicit resolution of the underspecified transport interface.

The first statement is enough to avoid inventing a new framing assumption in an abstract protocol theorem. It is not enough to advertise the second statement. Also, this is **not** a trace-refinement claim that v3.7 accepts exactly the records v3.1 accepted: ignoring a stale outer base and committing individual DATA deliberately changes acceptance behavior. Only the representation of result evidence is conservative.

## 1. Frozen source contract

| Frozen source | Consequence for this design |
|---|---|
| `baseline/model_and_assumptions.tex:114–119` | A complete result contains DATA, a direct or covering FINAL receipt, and the ancestry, boundaries and source evidence required for validation. A QC, a block hash, or missing payload is insufficient. Completeness is a semantic validation property, not a particular framing format. |
| `baseline/sync_hotstuff_rr_part2_rotation_v3_1.tex:1338–1345` | EXPORT is a structured record: chain identifier, branch version, expected global base, ordered DATA interval, and evidence. The evidence explicitly includes receipts, ancestry, REGULAR boundaries, historical authorities and native context. No binary grammar is given. |
| Baseline main `1347–1357` | A record can span sessions, and a rejected/stale EXPORT can still supply regional evidence. Evidence is not made inaccessible merely because the outer acceptance predicate fails. Each item still needs its own validity checks. |
| Baseline main `749–768,791–825,842–856,881–896` | Signed objects carry their actual domain/session context; complete receipt validation includes admission and ancestry; covering receipts preserve same-session ancestors only. Dependencies may come from a message or validated local cache, subject to the stated receiver-resolution condition. |
| `baseline/model_and_assumptions.tex:181–196,236–265` | The service preserves the result's DATA and commitment evidence and makes the submitted complete result available to online, history-following readers under the declared admission/load limits. Already recorded results discharge delivery; no corrected-wrapper deadline follows. |
| `baseline/model_and_assumptions.tex:128–133,159–165`; baseline main `587–620,1496–1504,1809–1813` | L1 supplies authority/control proofs separately; regional blocks reference authenticated L1 and Espresso histories. A historical L1 authority need not occur in an earlier Espresso record. |
| `baseline/model_and_assumptions.tex:218–250`; baseline main `1853–1858,2207–2212` | Required authenticated Espresso history remains retrievable. Public payload/evidence and archival coverage must survive producing-region retirement. Long catch-up is not assigned a uniform bound. |
| Baseline main `2148–2152,2186–2205` | The Publish Layer implementation must meet the service bound; its sketch is not a proof of that bound. Bounded complete-result delivery does not bound stale-base retries. |

The complete frozen Model section, exporter/acceptance section, receipt and validation rules, boundary/Ready rules, external-observer input rules, and Publish Layer implementation section were read for this analysis. The v3.6 `exports` and `source_step` fragments were read in full, together with their relevant main-source interfaces and liveness use.

## 2. The logical object model to specify

Write a frozen logical EXPORT as

\[
 E=(c_E,\beta_E,b_E,I_E,W_E).
\]

Here `I_E` is the DATA-list value and `W_E` is the evidence value specified by the frozen exporter. These are structured protocol values, not a claim about an existing byte-level encoding. Signed DATA and signed evidence retain their original signed bytes, hashes, domain and producing pair. Normalization must never reserialize signed content and thereby change its hash.

Use the constructors already needed to express those values: DATA/DRAIN bodies, signed proposals and votes, QCs, FINAL receipts, ancestry and parent links, admission witnesses, incoming boundaries, authenticated source proofs, authorities, and native application context. Their finite field/list structure supplies a logical path to each occurrence. A list position or a fixed field name is a path component. Sharing can be represented by an object graph. No well-founded validation derivation can be supported solely by an unresolved cycle.

An outer hint with the wrong value is still a field value. An invalid signature or invalid boundary is still a structured evidence object. A missing required object is absent. These three cases must not be collapsed into a single “parse failed” outcome.

At this level, “a result is carried” can be checked **before** defining the new normalizer: its DATA, a direct receipt root, and the finite witness graph used by the frozen validity rules occur in the record's DATA/evidence structure, with any source references interpreted by the existing source interfaces. The original result may occur entirely inside an ancestry field. An outer hint does not change membership in this graph. This is an occurrence-and-validity judgment over the old semantic fields; it does not mention the output of the proposed decoder.

The exact representation of omitted dependencies must remain visible in that judgment. A reference is an edge, not the referenced bytes. A regional receiver's private cache can discharge an edge in a point-to-point message. It does not automatically establish that the same edge is publicly reproducible from an Espresso carrier. Section 4 addresses that distinction.

## 3. Normalization and preservation lemma

### Definition of the map

Define `Flat(E,H)` over the logical tuple and the authenticated source objects `H` admitted by the baseline source interface:

1. Visit every field of `I_E` and `W_E` in a fixed field/list order, including the contents of ancestry, boundary and admission-witness objects. Do not first validate or filter by `c_E`, `beta_E`, `b_E`, interval ordering, a declared output index, or the validity of an enclosing proof.
2. Emit a typed occurrence for every object, preserving its full original content and logical path. Emit its dependency edges from the object's actual fields. Do not trust an auxiliary publisher index.
3. For a shared object, retain one normalized content entry plus its occurrence paths. Different proof encodings for the same DATA remain alternative proof witnesses. Do not merge a bad proof with a good proof merely because they share the DATA hash.
4. Resolve a reference only according to its old authenticated source identity/content semantics. Preserve the actual object and authentication witness. The public-resolution rules in Section 4 determine which references may be used for deterministic global replay.
5. Do not discard all children merely because one parent proposition, one signature, or one unrelated child is invalid. A candidate's required proof graph still fails if one of its required objects fails its own validation. An unresolved edge is never fabricated.

Implementation framing may encode this inventory as two typed sequences, or another faithful representation. Two sequences and byte-length boundaries are not premises of the abstract map. A canonical occurrence path replaces the current “canonical frame path” when ordering alternate proofs. Framing is useful for isolating byte corruption, but corruption isolation requires a concrete decoder specification.

### Lemma: preservation of complete logical result subgraphs

Let `E` be a frozen logical EXPORT and let `R` be a complete regional result carried by its DATA/evidence structure under the frozen validation rules, with its required source objects interpreted from the same baseline source context `H`. Assume the public references used here denote the same authenticated objects in both representations; for fully explicit result evidence this reference clause is empty. Then:

1. `Flat(E,H)` contains an occurrence of every object and dependency edge in `R`'s validation witness.
2. The direct receipt and every same-session ancestor it covers retain exactly the same signed content, producing identity, ancestry and validation judgment.
3. Changing only outer hint values, a declared output interval ordering, or adding unrelated invalid structured evidence cannot remove this witness from the inventory.
4. Normalization supplies no missing object, changes no receipt's scope, and creates no new authority or append permission.

**Proof.** Fix the finite validation derivation for `R` under the frozen rules. For explicit objects, induct over its typed structural occurrences. A leaf retains its entire content and path by step 2. For a compound value, step 1 visits every required child independently of both outer hints and the validity of sibling propositions; the induction hypothesis retains those children, and step 2 retains the actual dependency edges. For shared subgraphs, apply the induction to first visitation; step 3 retains all incoming references and occurrence paths. For a reference edge, the stated unchanged source interpretation supplies the same authenticated object, to which the previous cases apply. Thus every object used by the frozen derivation remains present. The original validity predicate operates on the same signed content, authority, ancestry and source witnesses, so its conclusion is unchanged. Coverage of an ancestor uses the same direct receipt and same verified same-session ancestry, as in baseline main `842–856`. No step adds an absent leaf or changes a signed field, proving the converse non-fabrication claim. Outer hints and unrelated objects do not occur in the traversal's inclusion conditions, which proves item 3. ∎

This induction is substantive even though simple: it establishes that the new algorithm does not omit an old evidence position, stop at a bad wrapper, take a publisher's list as exhaustive, or conflate failed and successful proofs. It is not the circular assertion that “supported carriers are the carriers this algorithm supports.”

**Consequent extraction statement.** If such a complete result contains the next DATA for the current unchanged duty, its signed ancestry extends current `G`, and its independent admission/application/source/current-version checks succeed, then the existing per-DATA scan will eventually consider a valid proof of that DATA. This is a finite logical-step statement. Its processing time is separate; finiteness alone does not prove a `Delta_i` bound. It also does not repair a genuinely missing predecessor, reopen a closed version, or satisfy a successor-authority dependency.

### What the lemma does not prove about bytes

A deployment must fix `encode` and `decode` and establish at least:

\[
 \operatorname{decode}(\operatorname{encode}(E))=E
\]

for every permitted logical EXPORT and every encoding it accepts as representing that EXPORT. It must additionally preserve independently represented complete subgraphs when the enclosing application-level hints or other objects are invalid. If multiple old codecs are supported, each needs this statement; a new encoder alone does not prove backward compatibility.

The frozen package supplies neither those functions nor legacy fixture bytes. Consequently “both existing v3.5 and v3.6 forms use this interpretation” (`exports:32–39`) is unsupported as a binary compatibility assertion. The admissible replacement is a statement about the common logical EXPORT interface and a named codec implementation obligation. No hypothetical ciphertext or corrupted length prefix is needed to expose this distinction.

## 4. References: what is inherited and what must change

### 4.1 Point-to-point cache references are explicitly allowed

Baseline main `798–801` permits a receipt to carry its DATA/ancestry or refer to a validated local cache. Baseline main `887–896` permits dependencies from the message or a validated cache, but a reference is allowed only if the receiver can resolve it within the regional delay contract. These provisions apply directly to regional validation bundles and early handoff; an outside receipt verifier also uses the same historical validity rules (`823–825,881–896`).

Therefore it would be inaccurate to say that v3.1 had no references, or that every cache-referencing receipt is incomplete. A correct same-region recipient already possessing the exact bytes is a legitimate example. The more permissive v3.6 typed handoff packet (`main:1832–1855`) likewise does not establish a public EXPORT reference format: unresolved handoff references withhold Ready instead of silently satisfying it.

### 4.2 EXPORT has a stronger public transport purpose, but no fixed wire reference syntax

The frozen exporter expressly supplies the evidence needed for its DATA interval (`1338–1345`). Complete-result visibility supplies the result to every covered online reader (`model:236–265`), and after publication authenticated history is the common retrieval path (`main:1853–1858,2207–2212`). Thus a pointer resolvable only in one sender's private memory, with no delivered or publicly recoverable object, cannot establish that this **public complete-result transport obligation** was met. This conclusion uses the old delivery interface, not the new decoder.

That does not imply the sender had to include an explicit source position beside every reference. The baseline says no such thing. Possible faithful representations include inline contents, a hash resolved against authenticated retained contents, or the semantic source identity already carried by an L1/authority proof. An exact location is an optimization for finding an object; it is not automatically part of the object's semantic identity.

If the proposed theorem instead quantifies over a concrete legacy carrier whose private cache references were treated as discharging the old “already recorded” exception despite public readers lacking the object, the source does not give an automatic repair. Completing that carrier from later private material or requiring a new wrapper would change the premise or require another delivery argument. Do not claim that the flattening lemma handles this case.

### 4.3 A conservative reference normalization rule

Replace the exact-position requirement at `source_step:14–35` with the following principles:

- Preserve every source identity and authenticated content reference the baseline already uses. For a source object, use its own domain and identity: e.g. L1 finalized block hash/height and registry proof; immutable `AUTH_(r,g)` with its authenticated publication evidence; Espresso header hash/height and authenticated namespace occurrence. Do not require an L1 object to be embedded in an earlier Espresso record.
- A reference to a content commitment may match an object in the record or the interpreter's authenticated retained public dictionary. Exact occurrence positions can be supplied, but absence of a redundant position must not reject a uniquely identified available object. If the proof order needs an occurrence, derive a canonical representative from actual authenticated occurrences, rather than require the sender to supply it.
- Include all authenticated prior raw Espresso records in the dictionary, even records rejected for append, and all authenticated source evidence actually supplied through the authorized L1 interface. Include only checkpoint objects established by the checkpoint's specified trust/authentication contract; a private checkpoint cannot enlarge that contract.
- Preserve the original temporal rules. In particular, signed Espresso selection references must precede the containing EXPORT block (`baseline main:618–621,1371–1373`). This is distinct from the location of an inline ancestry object or independently authenticated L1 proof. Do not generalize that Espresso rule into “every source proof needs an earlier Espresso position.”
- Treat a cache as a storage optimization for a publicly specified object, not as a new source of authority. Bytes privately available but not justified by the record/source/checkpoint interface cannot change public completeness. Conversely, failure of one reader to have cached a publicly specified object creates a retrieval wait; it does not make the object semantically absent.

This removes an unnecessary representational restriction. It does **not** create a deadline for retrieving an arbitrary new L1 proof or scanning arbitrary history. If a missing item is required by the canonical boundary/prelude, retain the existing wait. For candidate-only incomplete or invalid material, skip that proof choice under the common semantic predicate; do not turn its unsupported name into a new canonical wait.

The difficult boundary remains visible: an arbitrary name can denote a genuine previously unknown L1 authority in some execution. Authentication and required-object status determine whether it is relevant; merely labelling all unseen names bogus is unsound. Bounded handling of these cases belongs to the retention/preparation repair and T2, not to the carrier-representation lemma.

### 4.4 Reference coverage obligation, stated without circularity

For any by-reference EXPORT representation retained by the formal specification, specify its old denotation first: which object identity it names, which authenticated source establishes the bytes, and what the old complete-result delivery contract makes available. Then prove the normalization rule preserves that denotation. The criterion is not “does the new decoder find it?”

For explicit logical evidence, the preservation lemma already proves coverage. For an accepted old content-hash reference into the same authenticated source inventory, the same proof works after reference resolution. For an unspecified cache/URL resolver, no universal coverage proof exists until its old denotation and access contract are fixed. Do not resolve the ambiguity by silently choosing the narrower v3.6 grammar.

## 5. Opaque bytes and namespace cases

| Case | Correct classification |
|---|---|
| Complete structured result in the logical evidence field; wrong outer base/version/chain/interval hint | Covered by the representation lemma. Signed domain remains authoritative. The record must still reach the interpreter through the inherited transport interface. |
| Complete next DATA occurs only in a typed ancestry subgraph | Covered. All old evidence positions are visited; no fresh wrapper or corrected output list is needed. |
| Wrong signature or unrelated invalid structured child | Does not remove a different complete witness. A required invalid child still invalidates its own proof. |
| Missing payload or required ancestry, with only a hash and no permitted resolution | Already excluded by the frozen complete-result/validation definitions. No strengthened assumption is needed to reject it. |
| Arbitrary ciphertext with no decoding key or defined EXPORT semantics | Not an instance of the frozen logical EXPORT tuple. The baseline does not claim that arbitrary byte containment equals a complete regional result. |
| A documented reversible encoding of the full old tuple, but unsupported by the chosen new decoder | A genuine concrete compatibility gap. Calling it opaque does not discharge it. Prove the old codec adapter or limit the implementation claim. |
| An outer chain hint is false, while signed result objects have this chain's domain and the physical record is in the supplied relevant namespace | Covered. Do not prefilter by the hint before inventory. |
| A full typed EXPORT is put in an unrelated namespace with no locator/discovery path supplied to this interpreter | Not resolved by framing. The frozen relevant-namespace visibility clauses do not prove bounded discovery of it. Nor is the result itself necessarily semantically incomplete. If the unqualified “already recorded in Espresso” exception includes this occurrence, the delivery theorem needs an interface clarification or stronger access premise. |
| A full typed EXPORT in another namespace is actually delivered with its authenticated namespace proof through an already specified service path | Its location alone does not corrupt the result. Coverage depends on that existing service path and its old bound; do not impose an outer routing-label veto. |

The last two rows prevent an overclaim in both directions. One cannot treat every arbitrary Espresso occurrence as discoverable merely from finality. One also cannot prove that a logically complete result is incomplete solely because its namespace is inconvenient. The frozen abstract transport interface can consistently be read as delivering the complete logical result to the chain's reader interface, as its visibility paragraphs say. If “already recorded” is given a broader concrete meaning, the omitted discovery contract must be acknowledged.

## 6. Proposed manuscript edits

These are proposed edits only; none were applied.

1. **Replace `exports:14–39` with a logical-value normalization definition.** Remove “the existing syntax has two finite sequences,” “each object has its own decodable type and length boundary,” and the unproved v3.5 binary compatibility assertion. State the common logical tuple, all evidence constructors/paths traversed, preservation of signed bytes, and the separate codec obligation. Keep wrong-hint independence and non-fabrication. Explain malformed logical evidence separately from unparseable arbitrary bytes.

2. **Insert the preservation lemma immediately before canonical extraction.** Include the induction over explicit objects, compound fields, shared objects and preserved source references. State precisely that this is representation refinement. Retain the independent acceptance checks; do not infer acceptance from delivery.

3. **Change `exports:69–74`.** Replace “canonical frame path” by the canonical occurrence path in the logical EXPORT/source representation. If a concrete framing is specified later, prove it induces that path order. A reference to a duplicate source object uses a deterministically selected authenticated occurrence.

4. **Replace `source_step:14–35`'s exact-position grammar with preserved public-source denotation.** Permit existing authenticated source identities/content commitments and derive location from the retained source inventory when needed. Explicitly distinguish L1 source objects from Espresso-record objects. Maintain observer-independent completeness and the separation between cache misses and semantic absence.

5. **Rewrite `exports:198–224` as a representation and implementation scope paragraph.** Suggested core text:

   > The delivery assumptions range over the logical complete results and EXPORT fields specified in the Model and publication interface. The normalization lemma preserves every such result represented by those fields, including a result in an earlier EXPORT whose outer hints are wrong. No new submission is required. This statement fixes no legacy byte encoding. A concrete codec or by-reference compression must preserve the same logical objects and their authenticated source denotations; the specification does not establish compatibility for an unspecified codec or a private-only resolver. The service's reader interface must supply the record under the inherited namespace/visibility contract. Arbitrary undiscovered occurrences outside that interface are not covered by a bounded-discovery theorem here.

   This paragraph states a mathematical interface and an implementation limit. Do not retain the current circular-looking sentence “Here ‘carrying’ uses the supported ... syntax.”

6. **Update `liveness:195–203,220–224`.** Say “the first record carrying the complete logical result under the inherited public delivery interface,” then invoke the preservation lemma. Delete the unexplained qualifier “supported.” Preserve the two cases for newly submitted and already recorded results. If concrete transport interpretation is intentionally broader than that interface, leave the public-response theorem conditional and say exactly which reference/discovery obligation remains open.

7. **Update main `105–124,2137–2153,2473–2479,2596–2612`.** Replace framing-as-premise statements with logical normalization and its lemma. The abstract safety statement requires equal logical source inputs; codec/source authentication establishes those inputs in a deployment. Do not advertise all legacy byte forms as proved compatible.

8. **Retain explicit timing limits.** Do not use finite flattening or source-dictionary membership to prove all normalization/retrieval completes in `Delta_i`. T2 is separate. Do not add a bound for missing L1 authorities, arbitrary catch-up, native preparation, or an acceptance retry. If the retention/source-handler repair needs a regional relay allowance, expose it in that argument and its displayed budget.

## 7. Interaction with retention repair

The parallel retention design proposes retaining authenticated raw Espresso objects (including rejected records), all obtained finalized L1 objects/proofs, fixed-anchor evidence and full regional bundles independently of session workers. That design can implement the public dictionary above without requiring old objects to be republished.

Three distinctions must survive:

- Retaining an object already authenticated under the old source interface preserves its identity and availability; attaching a new arbitrary public source to an old carrier is a different operation.
- A later regional-only bundle may establish a valid historical receipt, but it cannot silently alter whether an already completed old Espresso record had a publicly specified proof. Reopening a past record or changing its semantic basis would need an explicit protocol rule and a new proof.
- Being entitled to obtain an authentic L1 proof eventually does not imply it was present at the first carrying-record deadline. A normalization lemma cannot close that timing gap.

## 8. Validation status

This note supplies a manual structural argument and concrete source references. No deployed encoder/decoder exists in the reviewed package, so no legacy binary compatibility test was possible. No manuscript, snapshot, manifest, PDF or memory was changed. The only written artifact is this design note. The proposed lemma resolves the abstract framing issue; unresolved concrete reference/discovery scope and the separate complete-handler timing interface must remain visible in the overall v3.7 verdict.
