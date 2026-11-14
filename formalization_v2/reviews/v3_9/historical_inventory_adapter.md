# Historical task inventory: existing-source adapter and interface boundary

Date: 19 October 2026. Bounded follow-up to `historical_service_design.md`, item 2. The frozen design, protocol sources, and frozen Model are unchanged. This note concerns discovery and canonical registration only; it proposes no other scheduler changes.

## Result

The specified sources support a canonical finite catalogue of initial AUTH records already required by **canonical public duty boundaries**. That adapter covers public zero-output sessions and introduces no all-authority scan or new P6 fetch.

They do **not yet establish** a complete catalogue of every previously installed speculative AUTH. The precise missing operation is not merely “get a finalized L1 height.” It is deciding, with authenticated completeness at that fixed height, which initial AUTH keys exist, and giving those keys a common registration event. The explicit completeness interface covers the control log; the existing reference and authority interfaces do not explicitly return a complete AUTH list, an AUTH high-water mark, or a terminating authenticated nonmembership answer for a candidate key.

The frozen statement that authority records through each finalized height are “common and authenticated” admits a strong abstract reading in which the complete authority set is already an available input. Under that reading a finite-cut scan is permissible code. The concrete interfaces retained in v3.1/v3.9 do not select or implement that reading. Replacing those interfaces with `EnumerateAllAUTH(Lambda)` would therefore assume the exact operation this follow-up was asked to derive. No claim that a deployment cannot supply it is made here.

## 1. Exact source contract

Paths below are relative to `formalization_v2/`.

| Source | What it supplies | What it does not specify |
|---|---|---|
| `revisions/v3_1_frozen/model_and_assumptions.tex:5–6` | Authentication of finalized headers, authorities, and a complete ordered control prefix | A complete ordered authority prefix |
| Same file, `30–44` | At most one immutable AUTH per `(r,g)`; waiting for required authority evidence | Contiguous initial AUTH issuance, maximum issued r, ordering of different initial AUTH publications, or a successful absent-key lookup |
| Same file, `46–52` | Control sequence number and binding control-history commitment | A commitment whose enumerated leaves are every AUTH record |
| Same file, `159–164` | Common authenticated authority/control history at each finalized height; expressly complete control prefix | A concrete AUTH inventory request, enumeration proof format, or distinction between complete-set delivery and point evidence |
| Same file, `208–214` | Bounded finalization/visibility of a valid **control operation actually submitted** | Bounded discovery of unrelated AUTH records or a full L1 block/receipt archive |
| Same file, `307–309` | Eventual availability of required authorities and native state | An algorithm identifying every historical required AUTH or proving a queried authority never existed |
| `revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex:587–596` | `rho` contains finalized L1 height/hash and a registry proof for **control records** up to that height | An inventory of initial authorities at that height |
| Same file, `1769–1791` | Refreshing the complete applicable control view and rejecting omitted controls | An all-authority completeness check |
| `sections/rotation_v3_9_source_step.tex:18–23,40–46,59–76` | Retain every **obtained** AUTH; fixed-cut Espresso hydration and the separately required finite AUTH prefix | Automatic acquisition of every authority below a finalized L1 cut |
| `sync_hotstuff_rr_part2_rotation_v3_9.tex:1783–1793` | Persistent sequential recovery-authority fetching over g for a known active r | Enumeration of initial authorities over r |
| `sections/rotation_v3_8_carrier.tex:4–55` | Original complete results retain the historical authorities needed to validate that result; exact public aliases use their original L1 interface | Publication of every authority known to the sender, including a future successor's AUTH |

The Model's common-history statement gives a unique mathematical set

`A_Lambda = { AUTH[r,0] finalized at or before finalized L1 height Lambda }`.

Uniqueness of this set and the ability to authenticate a supplied member are different from an executable procedure that returns the complete finite set and terminates. The latter is the missing-interface boundary. This distinction does not weaken the Model statement or assert that authenticating an old AUTH is impossible; it prevents a completeness operation from being hidden inside a local scan.

No existing bound should be reassigned to that operation. In particular, T_L1 applies to the submitted control, T_E to its declared Espresso source, and the regional Delta to actual complete messages. None supplies an unstated full-L1-authority enumeration deadline.

## 2. A supported restricted adapter

The following event rule is canonical using already required public inputs.

1. At genesis, after the specified initial `AUTH[0,0]` is authenticated, register its positive-weight tasks `(r=0, Hash(AUTH[0,0]), i)` in the fixed region order.
2. Whenever the public interpreter resolves a new canonical public boundary and validates its already required target `AUTH[r,0]`, register every positive-weight regional task of that AUTH if the authority has not previously been registered. Use the boundary's canonical source position and its deterministic substep as the registration event. Registration is not indexed by the callback's local completion time.
3. Deduplicate by initial AUTH identity. A later beta or a repeated witness cannot register it again. Persist the registry update with the same canonical boundary state and checkpoint it.
4. For a scheduler snapshot at canonical source subposition p, use exactly the registry prefix through p. This is a finite cut. Later boundaries append future entries and never retarget that cut.

The registration metadata is finite and uses the same target AUTH and finite region cycle that the boundary already validates. It does not fetch a further AUTH, enumerate earlier L1 blocks, or scan publisher-selected optional proof trees. If the boundary lacks its required AUTH, it already waits under the original protocol; this adapter does not invent a new wait. For REGULAR, terminal DATA and its accounting remain committed and exposed before successor resolution and registration.

Two observers with the same complete source prefix obtain the same public boundaries, target authorities, registration positions, and task order. Different retrieval/handler timing does not change those logical positions. A supplied later checkpoint must preserve this registry or an authenticated replay route for it; otherwise it cannot claim the same scheduler state.

### Coverage

- A public duty can be assigned and subsequently close with zero DATA. Its AUTH was already required for that public boundary, so it is registered.
- An unassigned public boundary whose target authority is eventually obtained is registered at its canonical resolution substep. Learning the AUTH earlier in a private cache does not create an earlier event.
- Registering every positive-weight region of that AUTH includes its nonselected regional rosters too. No request byte, request-owner report, or voting quorum is needed for catalogue membership.
- An AUTH used only by an off-chain speculative boundary need not pass through any of these public resolution events. This adapter alone does not cover it.

One may additionally extract fully supplied authenticated AUTH objects from actual canonical public source occurrences, if a precise finite parsing/authentication rule is specified. That enlarges the supported subset but does not create a guarantee that every speculative authority has such an occurrence. Optional references must remain optional; a catalogue extension must not turn every named AUTH edge into a mandatory P6 lookup.

## 3. Why a zero-output speculative child is the hard case

Consider a valid REGULAR successor at slot r whose predecessor is at r-1.

1. The predecessor has a genuine terminal complete result R0. The selected successor obtains and validates that result, its own finalized `AUTH[r,0]`, and its native state through the existing Ready inputs.
2. The complete ReadyBundle contains the child's AUTH and is sent within the child's selected snapshot (`rotation_v3_5_ready_bundle.tex:11–47,60–70`). This is authenticated same-snapshot transport. It is not an Espresso publication event.
3. The child receives and retains x, but Force closes the branch before the child has any complete regional result. No missing quorum, lost state, or violation of correct-message delay is needed for this finite execution prefix.
4. Correct owners retain x and the old authority. The predecessor's R0 is offered and eventually public under the original complete-result service. That R0 validates predecessor output; it need not contain the child's distinct AUTH. Its logical source fields contain required historical authority, not every future authority known to a holder.

Thus the public result can identify candidate index r without supplying `AUTH[r,0]` or proving whether that key existed at a chosen earlier L1 cut. The same predecessor terminal result is also possible when no child authority was ever issued and no child entered. Terminal output therefore cannot itself authenticate a positive task at r.

This is a coverage witness for the proposed inventory operation, not a claim that baseline v3.9 can keep r hidden forever. In the baseline, future canonical public boundaries might eventually reach r and register it. A historical scheduler that can repeatedly return to older r values must separately prove such future discovery; it cannot inherit it from unbounded DATA height alone. No alternative scheduler rule is proposed here.

## 4. Existing fields fix some cuts, but not their contents

### `rho` and opening L1 bounds

A DATA block's signed `rho` fixes a finalized height/hash and authenticates the control prefix there. Its opening bound includes the initial AUTH's publication height, so a valid child's first DATA would indirectly identify a sufficiently late L1 cut. A child with zero DATA has no such signed block to publish. Its boundary and ReadyBundle can remain regional material.

Even when a public `rho` supplies a late enough Lambda, its stated registry proof is for controls. A header hash can commit to an underlying L1 state without disclosing all keys of the application's authority registry. No AUTH range proof, finite-key traversal procedure, or nonmembership result is specified by this reference field.

### MSIG/control publication heights

An existing finalized control's publication proof can supply an authenticated finite L1 cut without changing its signed payload. Processing a stale MSIG still authenticates that source record; it must not execute its Force or attached changes. This is a possible **cut naming** mechanism only. The record's fields do not announce a list of AUTH keys, a maximum r, or completion of an authority inventory.

Consequently the frozen design's suggestion “use the publication cut of a processed control” solves the common-cut problem only after a complete-authority retrieval interface is supplied. It does not by itself solve task discovery. A controller's private knowledge that it has checked all relevant authorities is not a public completeness proof and is not encoded by the target-beta MSIG instruction.

### “Full L1 prefix” in control ingestion

The subsection's surrounding requirements explicitly concern control sequence, control-history commitment, applicable controls, and omitted-control rejection. It would be possible to implement it by reading full underlying L1 blocks, but that implementation is not required by the abstract interface. The text does not presently promise complete L1 block bodies and receipts, a fixed AUTH-emission address/encoding, or exhaustive contract-state traversal.

Reading that phrase as a full executable L1 archive must therefore be stated as an interface interpretation and checked against a concrete source implementation. It cannot silently turn a complete control prefix into a complete authority inventory.

## 5. Why nonblocking point queries do not finish the proof

Natural-number r values make background dovetailing attractive. It can be useful for locally discovering valid existing authorities, but the following distinctions remain.

| Rule | Remaining failure |
|---|---|
| Stop at the first missing initial AUTH | Initial AUTH keys/publication order are not stated contiguous; absence may mean unavailable evidence rather than no record |
| Query all r up to a public upper bound | No existing public upper bound covers every speculative child's AUTH; the last child can be zero output |
| Add each AUTH when a local lookup succeeds | Canonical queue membership/order becomes local-arrival dependent |
| Order late discoveries by their L1 publication height | Sorting eventually known records does not undo earlier Force choices made before a missing older record was known |
| Freeze a finite L1 cut and wait until all authorities through it are found | Finite existence does not supply a termination criterion; there is no specified complete list/range proof |
| Treat an unanswered lookup as absence after a timeout | No such response/absence deadline exists |
| Publish the old ReadyBundle or a bare AUTH discovery announcement | These are not original complete regional results; original Publish/Espresso service does not promise their delivery |

Even a finite known candidate set, such as “r+1 after this genuine terminal result,” needs a way to decide each candidate at the chosen cut. A successful AUTH membership proof can settle a positive candidate. The existing interfaces do not state a terminating authenticated negative answer for a never-issued candidate. Leaving that lookup pending is safe for a background cache, but it does not yield a complete canonical set that a Force can select from.

There is no contradiction between keeping such queries nonblocking and retaining agreement: they may populate private caches without changing public scheduler state. The missing step is a source-grounded canonical registration rule for the positive replies. Making registration depend on a shared fixed cut returns to the completeness/absence problem; making it depend on the local reply returns to arrival-order dependence.

## 6. Precise boundary for the frozen candidate

The supported public-boundary adapter can be adopted as an explicit finite subset catalogue. It meets the constraints in this follow-up, but its coverage theorem stops at authorities that obtain a canonical public boundary or an otherwise guaranteed canonical public source occurrence.

To upgrade it to all historical installed AUTH while keeping the signed formats unchanged, one of the following must be **derived from an existing concrete L1 interface**, not merely asserted:

- complete authenticated authority-emission records through a fixed finalized cut, with their finite traversal and completion condition; or
- a finite authenticated candidate-key bound together with terminating membership/nonmembership proofs at the cut; or
- an existing mandatory public occurrence for every installed speculative authority, including a zero-output child, plus a canonical finite occurrence-processing rule.

These alternatives describe the missing interface; they are not proposals to add a new enumeration service, publication class, timestamp bound, or signed control field. The current exact sources establish none of them explicitly. Under the strong complete-set reading of the frozen common-authority premise, the first is the intended abstraction and a finite-cut adapter can be written once its canonical cut-admission event is fixed. Under the explicit point-evidence/control-prefix interface, the full inventory proof remains open.

Until that reading or concrete interface is established, the correct status is: **canonical public-boundary inventory derived; complete speculative/zero-output historical inventory not derived**. Fixed-cut journaling prevents an infinite moving tail but cannot supply absent source completeness. No new catalogue work should be inserted into already prepared P6 or charged to its existing complete-message bound to conceal that gap.

Validation: direct inspection of the frozen Model, original v3.1 reference/control interface, and current v3.9 retention, authority-fetch, handoff, ReadyBundle, and logical-carrier definitions. This is a source/interface derivation, not an executable experiment or a whole-protocol review.
