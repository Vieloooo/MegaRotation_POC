# Summary

**Final Verdict:** No Critical Error or Justification Gap was established in this independent model, service, and interface review of the frozen v3.11 snapshot, under its stated abstract complete-message/native interface and the two exact human clarifications. This is a scoped conditional verification, not a verdict on the entire protocol or completion of the overall goal.

**List of Findings:** None classified as a Critical Error or Justification Gap in the reviewed interfaces. The timing and service limitations below are material to the scope of this verdict.

The complete Model section is byte-for-byte equal to both frozen v3.1 and `baseline/model_and_assumptions.tex`; its SHA-256 is `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387`. Literal equality is a separate fact from semantic faithfulness. The latter requires the endpoint, receipt, retention, packet, and handler checks described below.

Pre-review manifest verification passed **26/26** entries with no mismatch. The reviewed main hash is `de2e36905fef1e2de7bcc53f0cd8f4a95eca92e4561d347b33bc260e7dc58bef`. Post-review verification is recorded at the end.

# Detailed Verification Log

Locations below are snapshot-relative; line numbers refer to the frozen source. I read only the authorized review scope, manifest, snapshot, and proof-verifier skill. I did not consult live author sources, author notes, prior reviews, peer conclusions, or repairs.

## 1. Frozen premises and exact human clarifications

**Locations:** `baseline/model_and_assumptions.tex:167–179, 293–319`; main `:137–158`.

> “The delay bound includes delivery and processing of complete validation dependencies.”

> “covers old and new correct members of that physical region”

The clarified endpoint scope removes the former ambiguity about correct endpoints from different AUTH snapshots in one physical region. It does not supply a cross-region link or make a failed/departed machine correct. The ingress clarification supplies regional receipt at every party, from which at least one correct-member receipt follows under the regional fault bound. It supplies neither a delivery deadline nor receipt by every future roster before preparation.

The candidate uses exactly those distinctions. Required authorities and recoverable state remain eventual preparation inputs; the original progress premises remain conditional, and original publication starts at availability of a complete regional result.

## 2. Finite, known historical AUTH directory

**Locations:** `sections/rotation_v3_10_ownership.tex:14–56`; `sections/rotation_v3_11_liveness.tex:105–150`.

> “Obtain missing records by their known keys $(k,0)$ from this lineage.”

For an actually admitted slot, genesis starts at zero, a positive-output/regular advancement increments the responsible slot, and a zero-output retry retains it. Consequently the protocol-generated lineage requires each initial authority through the admitted slot. The union of these finitely many finite rosters is finite, as is the maximum of their positive regional allowances. An authorized checkpoint is explicitly required to retain the relevant identities and authentication dependencies.

This is not enumeration of unknown future L1 records or an absence oracle. Retrieval is not assigned a uniform deadline. The proof does not take a maximum over all future configurations. The use of `d_i(r)` is confined to the known finite physical-region prefix and the clarified correct-member service.

This reasoning applies to the specified authenticated lineage/checkpoint interface. It does not establish that an arbitrary older checkpoint lacking that information can immediately support the new directory.

## 3. Persistent service is an explicit correct-node duty

**Locations:** `sections/rotation_v3_10_ownership.tex:58–108, 419–444`; `sections/rotation_v3_11_retirement.tex:122–150`; main `:1669–1676`.

> “Closing its production program does not close that service.”

> “the last correct original owner of a still-pending request cannot leave merely because its AUTH is old”

The protocol specifies actual retained bytes, exact immutable cuts, receive routes, contexts, raw handles, cursors, shares, certificates, and outgoing actions. Termination does not replace these by a private promise or an unowned retrieval task. Physical departure requires exhaustion of the relevant duties or continuation of the same logical owner's state and authenticated route in the same region.

Under the clarified communication premise for correct old members, these are algorithmic duties rather than a newly assumed external archival party. Their cost is real: the source does not promise bounded storage or an independent physical-exit deadline. A failure that removes the last correct pending owner is outside this proof's continuing-service scope. Worker retirement alone is not such a failure.

## 4. Receipt, raw storage, canonicalization, and admission are distinguished

**Locations:** `sections/rotation_v3_11_requests.tex:4–38`; `sections/rotation_v3_10_ownership.tex:62–108`; `sections/rotation_v3_11_input_selection.tex:39–50, 88–124`; `sections/rotation_v3_10_public_requests.tex:47–107, 119–129`.

> “Processing the last required fragment appends the handle … it does not run canonicalization, application admission or a whole-pool scan.”

An ordinary complete authenticated receipt persists its request and journal entry before acknowledgement. Hydration fragment completion installs a raw handle before releasing the receive obligation. Correct opening selection freezes both the raw-journal prefix and canonical-store version after checking hydration, then actually performs canonicalization, authentication, exact-parent admission, ancestry exclusion, and minimum selection.

Thus deferred ordinary canonicalization cannot omit an already complete hydration record from the opening cut. Public import separately records held-source work, extracts actual typed batch entries, and validates identities before deduplication; an invalid asserted identity cannot suppress a later genuine request. An unscanned public root is correctly not treated as an already extracted request handle.

The request theorem starts from the clarified delivery of a correct client's complete admissible request and the ordinary receipt rule. It does not replace that premise by a client send attempt, a hash, an unauthenticated wrapper, a partial object, or a Byzantine-only receipt.

## 5. Context barriers and finite transfer budgets

**Locations:** `sections/rotation_v3_10_ownership.tex:130–241`.

> “Merely queuing a send does not define $a$.”

> “No new $d$ deadline is assigned to this whole-object work.”

The source explicitly transmits every authentication-capsule chunk and records actual send times. The trigger waits strictly beyond full correct-recipient chunk delivery; the first cut query waits strictly beyond the source-authentication trigger's processing deadline. The trigger is limited to the original supplied-dependency source check, with acquisition, native conversion, and whole-object preparation outside it.

The descriptor, record count, record length, digest, and fragment count freeze once. Byzantine silence, equivocation, or malformed metadata cannot enlarge a running job or restart it. Correct replies are protected from unrelated unauthenticated/context-mismatched traffic. Equality is handled by processing due replies before expiry. Whole-object reconstruction and checking are terminating preparation, not charged wholesale to a single delay interval.

The resulting claim is finite completion under continued local execution and admitted message/load scope, not a practical denial-of-service or capacity bound. A finite but enormous Byzantine descriptor remains costly; the source does not deny that cost.

## 6. Actual dissemination precedes every HYDRATED certificate

**Locations:** `sections/rotation_v3_10_ownership.tex:243–342`.

> “all required raw handles are retained at each correct target by $b_v+d<b_v+2d$, strictly before the share”

Every correct signer actually multicasts all fixed metadata and chunk envelopes to the whole target roster, then waits past their processing deadline before signing. Shares are collected and certificates relayed by persistent node-level handlers. Cancellation requires a complete valid certificate and preserves promised sends.

A valid certificate of weight greater than `F` has a correct signer. Its completed dissemination therefore puts the covered request at **every** correct target before that certificate can exist. This is the necessary causal bridge between unrelated hydration and later offer signer sets; no honest-quorum intersection is inferred.

## 7. Opening formation, selective delivery, and OC relay

**Locations:** `sections/rotation_v3_11_input_selection.tex:62–86, 126–240`.

> “On first validation of any complete OC for $K$ … records an outstanding multicast to every target member.”

State-candidate distribution precedes offers and does not wait for Ready, removing a state/OC/Ready circular dependency. Each uncancelled correct member completes its fixed finite selection and sends a complete offer. Correct weight alone exceeds `F`.

If a Byzantine sender first selectively delivers an otherwise valid OC to one correct member, that member durably retains and relays it before cancelling selection or notifying Ready. This relay survives Closed/Terminated and does not need live entry. If no correct member has a certificate, none can cancel on that basis and correct offers eventually suffice. These two cases support universal formation.

An already valid OC is checked with `CheckOC`, which requires only its selected body, not all nonselected Byzantine bodies. Therefore selective nonselected-body withholding does not defeat the first-possession relay. Alternative valid OCs remain admissible; the retained relay copy is not a private veto.

## 8. Every-certificate coverage and finite exceptional slots

**Locations:** `sections/rotation_v3_10_ownership.tex:366–417`; `sections/rotation_v3_11_input_selection.tex:219–240`; `sections/rotation_v3_11_liveness.tex:513–572`.

> “Equality is included, so no equal-time receipt/query order is needed.”

The positive `release(r)` pacing gives only finitely many numeric slots with a correct first query at or before the actual receipt time. The finite set below the owner's first AUTH slot is also excluded. Every other target includes the owner and has all correct query cuts strictly after receipt. This covers every valid HYDRATED certificate, including certificates assembled later from old shares.

Positive accepted output advances the canonical numeric slot; zero-output retries do not multiply accepted quota. Thus the finite exceptional slot set contributes only finitely many accepted session-first blocks, despite arbitrary future roster changes or infinitely many zero-output beta retries.

At a covered target, every correct offer's frozen cut contains the request. The threshold supplies a correct offer; the certificate minimum is no larger than that offer's key; `CheckOC` requires an actual eligible selected body. This establishes the claimed universal OC property without mempool-root agreement.

## 9. Timed Ready, REPORT, and SOURCESTEP interfaces

**Locations:** `sections/rotation_v3_11_ready_bundle.tex:66–177, 186–267`; `sections/rotation_v3_11_requests.tex:95–165, 208–248`; `sections/rotation_v3_11_source_step.tex:145–268`; main `:2124–2133`.

> “If validation, adoption and the atomic Enter transition are the complete message-processing operation covered by [the network assumption]”

> “Calling that work one event cannot prove $u_v=0$.”

The stated entry-skew inference is conditional on a complete executable object, full supplied validation dependencies, and direct adoption in the modeled handler. Its continuation with an unbounded residual native-import delay is explicitly withheld. The source inventories the OC bytes, advertisement and report signatures, selected-body checks, durable attachment, and aggregate dispatch. A singleton request fitting DATA alone is explicitly insufficient to prove that an enlarged Ready/proposal envelope fits an existing capacity limit.

Within the original abstract complete-message interpretation, the reported `4Delta`/`6Delta` schedule has no new post-entry pool scan: ordinary-term ancestry is discharged, while term zero checks the initializer's advertised OC. ANCHOR forwarding/voting and original evidence inspection remain independent of report-sidecar validity. A valid alternative OC cannot be rejected because of local preparation choices.

SOURCESTEP retains and sends raw source input before P6 or request import. Its prepared handler uses resident source objects, fixed required authority, public kernel checks, guarded DATA commits, and an already executable state/OC attachment. It does not wait on owner discovery, a future roster, native reconstruction, or public import. Terminal DATA commits before successor-authority resolution.

This supports the **conditional abstract handler** claim reviewed here. It does not prove an implementation's packet admission inequalities or bound arbitrary native imports, historical backlogs, codec work, or external-observer application processing. No deployment benchmark is required to assess the formal claim, and this report does not treat the absence of one as a proof defect. Conversely, literal Model equality alone cannot establish those concrete adapter obligations.

## 10. Original public result, source scope, and honest-FINAL provenance

**Locations:** `sections/rotation_v3_11_requests.tex:167–204`; `sections/rotation_v3_10_receipt_kernel.tex:33–92, 157–176`; `sections/rotation_v3_8_carrier.tex:17–58, 108–181`; `sections/rotation_v3_10_exports.tex:201–238`.

> “The Publish Layer input remains exactly $R_0$.”

The erasure removes only the new regional OC/advertisement/report sidecars. Original DATA, signed proposals, QCs, FINAL, ancestry, authority, native-validation objects, and public-source denotations remain in `R0`. The selected request consumes ordinary DATA capacity. No bare request, weaker public kernel, hydration certificate, or Ready message is granted the original publication deadline.

The public kernel's validity argument uses an actual correct FINAL signature and traces it to the timer-origin Vote that performed augmented own-session validation. Receipt-triggered commits emit no new FINAL, so the provenance argument does not create circular receipt validity. A publisher's alternate sidecar is not authenticated by that argument. Execution output, exact native state, and `C_pub` remain separate obligations.

Logical normalization preserves objects and denotations; it does not manufacture bytes, make privately cached evidence public, find objects in unrelated namespaces, or prove compatibility for an unspecified binary encoding. Source retention includes rejected records. Public import and handoff work do not gate raw source intake. Required canonical boundary evidence remains an explicit wait, and per-DATA commits preserve the original append authority and strict version cutoff.

## 11. Global, regional-service, and request quantifiers

**Locations:** `sections/rotation_v3_11_liveness.tex:70–203, 205–419, 421–499, 574–697`.

The finite preparation maxima range over the fixed stalled-frontier slot, its finite historical AUTH prefix, and the finite physical-region cycle. They do not imply a uniform bound for all future memberships. The global contradiction then uses its explicitly conditional prepared regional response and source-handler bound. A nonempty eligible set can include a singleton; this is not a claim that the protocol tolerates arbitrarily missing correct quorums in otherwise assigned snapshots.

Repeated regional service is stated for a region that eventually remains governance-ON with positive authority weight and satisfies the serving premises. It does not promise service to an administratively permanent-OFF region. Empty DATA can discharge that service debt, so the request proof correctly supplies a separate argument.

For requests, unbounded global DATA over a finite region cycle supplies a region with infinitely many accepted contributions. Applying the original ingress statement to a continuing serving region supplies one actual correct owner, not future-roster awareness. Finite quotas yield infinitely many accepted session-first opportunities; removing finitely many exceptional slots leaves infinitely many covered opportunities. Persistent admissibility at the relevant accepted openings and singleton fit are explicitly retained. Each such accepted block spends a previously unaccepted canonical identity no larger than the request's key. The bounded-length set is finite, so the nonincreasing potential cannot decrease infinitely often without inclusion.

This verifies the reviewed awareness and service-interface arrows conditionally on the global progress and safety dependencies. It is not an independent certification of every regional timer/rank/recovery or public-state-machine proof.

# Limitations and integrity

- No executable model checker, adversarial simulator, compiler run, or deployment measurement was performed. Source inspection and mathematical/interface checking are the evidence for this report.
- No concrete native adapter, binary codec, packet-capacity profile, or storage handoff implementation is supplied by this snapshot. Their conformance is not proved by this review.
- No claim is made for physically departed last correct owners, permanent-OFF regions, inadmissible requests, arbitrary missing correct quorum service, unknown future authorities, or unbounded native work inside a claimed bounded handler.
- The scoped no-finding result is not a whole-protocol or whole-goal completion claim.

Post-review manifest verification: **26/26 entries matched, zero mismatches**. Main SHA-256 remained `de2e36905fef1e2de7bcc53f0cd8f4a95eca92e4561d347b33bc260e7dc58bef`.
