# Hydration to Ready and REPORT: a concrete selected-input adapter

Date: 2026-10-19. This is a protocol construction and manual proof audit for the bounded task in `goal_blocker_revalidation.md`. It changes no manuscript or frozen assumption. Paths and source line numbers below are relative to `formalization_v2/`.

## 1. Result and precise scope

**A concrete adapter avoids the unbounded pending-index operation on the timed path, but it changes REPORT's selection rule.** Before Ready, members actually finish a finite, exact-opening-state selection task and certify one selected request. A Ready receiver checks that certificate and one DATA-sized request; it need not finish its private pool scan. The input obligation applies to the **first own-session DATA**, and is inherited through that DATA's ancestry during recovery and later terms. REPORT becomes a bounded selection from this prepared certificate at the opening parent, and EMPTY after the obligation has already been discharged in the parent ancestry.

This preserves the round schedule giving ordinary-term `4 Delta` and initial/recovery-term-zero `6 Delta` first-vote upper bounds **at the original complete-handler interface**. It never assigns a whole-pool scan, source catch-up, new native restoration, or arbitrary deferred job a Delta deadline. It also supplies a request-inclusion proof based on infinitely many accepted session-first DATA, replacing the current proof's infinitely many aware term-first reports.

There remain two precisely separate limits:

1. The ownership candidate's mixed-snapshot delivery and continuing-owner premises are assumed **only conditionally**. Nothing here derives them from frozen v3.1.
2. The source has no concrete codec, message-size budget, signature implementation, or aggregate dispatch profile from which to prove that an enlarged Ready message fits the old service envelope. Section 9 specifies its exact additional objects and processing calls, and the inequalities an implementation must satisfy. The inherited native ReadyBundle limitation is also retained. Thus this note closes the algorithmic dependence on an arbitrary pending-pool rebuild; it does **not** certify an unconditional deployment timing theorem or the full unchanged-model goal.

The construction uses the original fixed regional snapshot and its `>F` threshold. It creates no new committee, trusted party, public request service, public activation certificate, or cross-region transport promise. The new preparation signatures are protocol assertions by those existing members, with the same correct-signer reasoning used for current INPUT_REPORT. They are not BFT votes and carry no rank, lock, entry time, or finality.

## 2. Why preserving the existing minimum rule is still unproved

Current requests:58–75 requires `min E_v(P)` over every locally retained pending request eligible at the exact parent. The source explicitly says an extra deferred pool scan or eligibility rebuild is outside its timing claim. Ownership v3 establishes raw bytes, and explicitly leaves exact-parent processing open. ReadyBundle:90–105 permits adoption of an already executable received state; it does not construct an arbitrary eligibility projection over the recipient's different private pool.

For example, let H0 complete its own native state and index, and send the first ReadyBundle at e. H1 already has all required raw records because HYDRATED completed, but its finite private pool is much larger. H1 can adopt H0's valid executable state through the original Ready handler while its own canonicalization/admission scan is unfinished. No raw-byte delivery event establishes when H1 finishes that scan. A `>F` prepared-index acknowledgement would still not establish that a different correct reporter outside its signer set finished. Copying H0's index would describe H0's cut, not H1's required `E_H1(P)`.

Precomputing only for the initial anchor does not preserve that rule at later arbitrary parents either. The application can make many previously invalid requests valid, or the reverse, after a new DATA. The current generic application interface contains no bounded incremental update theorem for an arbitrary local pool. This is an exact missing derivation, not an impossibility theorem for every implementation: an implementation may already support the required old abstract batch operation. HYDRATED itself does not prove that it does.

The adapter below removes this operation from REPORT's specification. It does not rename the operation, assert that an index is free, or assert that every member has the same queue.

## 3. Typed state and contexts

Let `N=V_i(r)` be the target AUTH0 snapshot, with its original weights and fault bound, and let A be the exact **effective DATA opening anchor** for a session. Preserve the canonical encoding, logical identity, and order from requests:5–11,48–56:

`key(x) = (|enc(x)|, Hash(enc(x)))`, with EMPTY above every request key.

Use the existing hash-binding and canonicalization rules; alternate wrappers and proof encodings do not create new requests. The preparation key is

`K = (chain, protocol, PREPARED_INPUT, i, r, hash(AUTH[r,0]), A.original_pair, A.hash, A.DATA_height, A.physical_position, native_validation_domain)`.

The last field is the existing application/native validation domain, not a new state commitment. The original sound state evidence still connects the actual executable object to A. Do not replace it with a claimed root. K intentionally excludes beta, generation, term, QC encoding, public receipt time, and a local queue root. Reuse is allowed only when every field of K matches. An actual REPORT retains the complete original `RC`, including beta, g, t, opening identity and exact parent.

The node-level owner persists:

| Object | Meaning and update rule |
| --- | --- |
| `RawJournal` | Finite append-only journal of complete immutable raw-record handles. A handle names exact stored bytes, length, digest and authentication material. Incremental fragment bookkeeping marks completion and appends the handle when the last required fragment has been processed. It performs no whole-pool scan or application admission. |
| `Canonical[id]` | Canonical authenticated request bytes, key, identity and original first-receipt metadata. A duplicate keeps the existing entry and FIFO position. Invalid raw material never becomes a valid canonical request. |
| `Pending[id]` | The current pending view; only verified inclusion in G authorizes removal. Inadmissibility at a speculative parent, recovery, or worker exit does not remove the retained request. |
| `Hydration[i,r]` | A fully checked HYDRATED certificate for the ownership candidate's exact common context. All its original serving and relay obligations persist. |
| `VerifiedState[K]` | The fully constructed executable A-state and its original sound native/source/authority evidence, obtained locally or by a pre-Ready STATE_CANDIDATE. It grants no entry or timer. |
| `SelectionJob[K,v]` | One finite immutable raw/canonical cut, the executable A-state dependency, a cursor, a best candidate, and durable work status. At most one such job per K and member; beta cannot restart it. |
| `Offer[K,v]` | One durable signed `OPENING_OFFER(K, key or EMPTY)`, plus its full request body if nonempty. Immutable after creation. |
| `OpeningCert[K]` | A complete certificate described below, plus its selected full request body. Different valid variants are allowed; local attachment does not make one variant a private proposal-validation requirement. |
| `InputAttachment[beta,r]` | One validated matching OpeningCert and the original ReadyBundle executable state, attached at initial Enter. It is not a pointer to an unfinished local job. |

Raw receipt and canonical insertion are separate transitions. A hydration packet whose raw bytes are complete is immediately represented in RawJournal, even if canonicalization is not yet complete. This uses the candidate's already stated incremental storage/fragment-completion interface. If an implementation instead does an unbounded final record join or scan before exposing any handle, that operation must be completed as actual preparation; this note gives it no Delta bound.

For a request already accepted in G, a verified acceptance record may mark its journal handle unnecessary for a new selection job. It must not remove bytes pinned by existing evidence, selection work, an outgoing obligation, or a regional certificate. Raw journal metadata may remain after global acceptance. A request still absent from G has a retained raw or canonical handle. This covers a record that was canonicalized before hydration as well as one canonicalized afterward.

## 4. Finite pre-Ready selection and certification

These are preparation actions, run by the node-level manager independently of any production worker. They have eventual/actual finite completion, not a uniform Delta completion bound.

### 4.0 State can be shared before any Ready

There is no requirement that every offer signer independently reconstruct state or wait for the first Ready sender. As soon as a correct member has constructed and validated the original executable A-state, it creates and actually multicasts

`STATE_CANDIDATE(K, original complete Ready dependency set)`.

This is a separately typed preparation message containing the full executable object and all the same exact-anchor, native, authority, handoff and source dependencies; it has no OC and grants no entry. The receiver authenticates and validates those original dependencies and caches `VerifiedState[K]`. It may use that executable object for its finite selection job. It does not set an entry flag, send STATUS/ANCHOR/votes, start progress timers, or run the Ready receive transition. Public/control evidence still goes to the original manager handler, so preparation does not suppress learned closure.

Creation, finite transport/fragmentation if needed, actual validation and native preparation of this candidate occur before Ready and have no new Delta deadline. Every correct native preparer performs this multicast without waiting for offers, OC, Ready or another preparer's acknowledgement. A correct receiver can validate the supplied executable object with the existing native interface rather than reconstructing a different state database. An implementation that needs additional conversion performs it as actual preparation here; a certificate never certifies that such work is complete at a different node.

The original eventual native/state-availability premise is still needed for some correct preparation to finish, just as in the original Ready proof. After one finishes, its direct candidate distribution supplies the dependency needed by other correct offer signers. This breaks the possible circular wait `offer -> OC -> first ReadyBundle -> other signers' state -> offer`. The timed first-Ready receive path later retains its separate original direct-adoption requirement. Fragmented eventual preparation does not establish that timed requirement.

### 4.1 One correct member's actual computation

For K, a correct member v runs the following procedure:

1. Authenticate the exact target AUTH and opening-anchor context, and start the original executable A-state/native preparation or receive a STATE_CANDIDATE as above. These tasks run concurrently with hydration; neither a claimed future handoff nor completion of this work grants Enter.
2. Obtain a full valid `Hydration[i,r]`. Only after its complete receive event, atomically freeze the actual finite RawJournal prefix and canonical-store version for this job. The frozen cut includes every complete retained raw record already received, regardless of whether v's local canonicalization queue has processed it. Canonical records not represented by a new raw handle are included through the frozen canonical-store version. Taking the cut uses maintained finite counters/version handles, not a new scan. New records never enlarge this job.
3. Once `VerifiedState[K]` is actually available, iterate the already frozen finite cut. For each record, perform canonical encoding and authentication checks; merge duplicate logical identities without changing FIFO metadata; persist newly valid canonical entries; and retain still-pending requests even when they are ineligible at A. Using the already verified exact A-state and complete A ancestry, evaluate the **existing** request-admission predicate, individual DATA fit, and ancestry exclusion. Constructing any ancestry index or evaluating all these records is part of this actual preparation. Skip malformed records and already-in-A identities. Keep the least eligible key found. Do not wait for a future body, external request fetch, or expansion of the frozen cut.
4. Finish every record and persist the result. Emit exactly one `OPENING_OFFER(K,k_v)`, with `k_v` this minimum or EMPTY. If nonempty, the full canonical authenticated body accompanies the offer. Persist before sending. Actually multicast that complete contribution to all target members; unfinished sends are durable duties. A correct member may not sign before its finite job finishes.

The individual first receipt and relay rules still persist a valid request and deduplicate it. The selection job adds a specific completion event, `SelectionDone[K,v]`, at which canonicalization, exact-A admission, ancestry filtering and minimum computation for **this fixed cut** are finished. It does not assert that the live pending queue or another parent projection is complete.

All admitted complete objects and the frozen cut are finite, and each specified local verification is an actual terminating computation, as in the ownership candidate. A very large Byzantine finite cut or record can make preparation very slow. No practical CPU, memory or denial-of-service bound follows. No subsequent incoming request can restart the job or enlarge its frozen work list.

### 4.2 Collecting a concrete certificate

Each manager keeps a collector for K. For a received contribution, it checks signer membership and signature, K, and, if nonempty, the complete body, canonical key, individual fit, admission immediately after A and absence from A ancestry. Count at most one complete valid contribution per member. An equivocation does not enlarge that member's weight or restart any work. This is a pre-Ready collector, with no timed success claim.

As soon as retained contributions have weight `>F`, build

`OC = (K, {(member, signed key)}_S, selected_body)`.

The certificate keeps the signed keys of distinct members. If at least one key is nonempty, `selected_body` is the complete body for their minimum; otherwise it is absent and the certificate is EMPTY. All correct contributions by themselves have weight `W-B >= W-F > F`, so no Byzantine cooperation or identifiable "all correct replies" event is needed. A complete certificate is durably stored, made available to the Ready manager, and actually relayed to all members. It can be transported in the same fixed finite fragment procedure **before Ready** if required. This does not make that multi-packet transfer one Delta.

The common predicate `CheckOC(A,AUTH,OC)` checks:

1. exact K and the original target AUTH;
2. distinct snapshot member signatures of weight `>F` on their individual signed keys;
3. the minimum of those **supplied keys**, requiring no private queue access;
4. for a nonempty minimum, its full canonical authenticated body, matching key, admission immediately after A, single-request fit and absence from A ancestry; for an empty certificate, that all signed keys are EMPTY.

It need not obtain nonselected bodies, the signers' cuts, their private queues, or their HYDRATED certificates. The correct-offer algorithm, combined with one correct signer in OC, provides the assertion about the cut. This is the same type of honest-signer argument as requests:138–142; CheckOC never purports to prove a minimum over an unknown pool by cryptographic inspection alone.

**The Ready input gate is changed:** OC replaces the candidate's raw HYDRATED certificate as the transmitted input-preparation gate. A correct offer can exist only after its signer checked HYDRATED and finished selection. Thus a valid OC implies the existence of that causal preparation chain. There is no second HYDRATED signature set or historical directory capsule inside ReadyBundle. Those remain pre-Ready dependencies at the offer signers. This is deliberate certificate composition using the same fault-bounded snapshot, not an assumption that a sender's unsupported state assertion is true.

A member may adopt a valid OC and cancel its redundant unfinished selection job. It preserves raw/canonical/pending data and all promised service, fragment, share and certificate relay duties. An OC does not need the adopter's own offer. Unless a valid OC already exists, correct members continue their finite jobs and dispatch their offers, so adoption cannot prevent certificate formation.

### 4.3 Coverage lemma

Fix a target r outside the ownership candidate's finite exceptional set for a request x, and suppose x stays globally unaccepted. Every valid HYDRATED certificate for this target contains a correct signer whose first query was after x's original receipt. The ownership proof therefore puts complete x bytes at **every correct target** strictly before that certificate exists.

Every correct OPENING_OFFER for K freezes its cut after checking such a certificate. Therefore its cut contains x as raw or canonical data unless x has already entered G. If A is an eventually accepted parent and x remains admissible there, fits alone and is absent from A ancestry, the actual completed selection examines x. Hence its nonempty key satisfies `k_v <= key(x)`.

Every valid OC has at least one such correct offer. Its selected key is no greater than that offer, and CheckOC supplies a real eligible request with that key. Thus **every valid OC for this covered target and accepted opening anchor is nonempty and selects y with `key(y) <= key(x)`**. The proof tolerates different HYDRATED certificates, different frozen cuts, private supersets, FIFO orders and OC signer sets. It requires no common mempool root and no honest intersection between different certificates.

## 5. Concrete Ready send and receive rule

Before its initial Ready/Enter event, a correct sender holds:

- the original full ReadyBundle, including the actually executable A-state and its complete original native/source/authority evidence;
- a complete OC satisfying CheckOC for the exact K of that bundle.

Define `ReadyBundle+ = (ReadyBundle_original, OC)` with normalized dependencies supplied once. The selected request is present as full bytes if OC is nonempty. A reference to a pending pool, missing request, selection task, state import, historical directory or future certificate is not complete.

At a first Ready/Enter event, the sender atomically performs the existing guarded entry transition, attaches the validated executable A-state and OC, records the original entry time, and actually multicasts the full ReadyBundle+ at that event. It does not begin selection or bundle construction after recording entry. Attaching OC is a convenience for sending later complete inputs, not a decision fixing a unique session primary. A later valid NEWVIEW/proposal may use a different valid OC for the same K; validators apply CheckOC and never veto it merely because their local attachment differs.

A receiver handles the packet in this order:

1. Run the original evidence-first inspection and full ReadyBundle authentication, historical handoff, exact-anchor, authority, reference and native-state checks. Reject a mismatching K/OC under CheckOC. No private queue/index is consulted.
2. Apply the original current activation, fixed-handoff, Closed, RequestClose, tombstone and already-entered tests. A stale valid certificate remains historical preparation evidence; it cannot reopen a closed pair or reset a timer. A closed packet still triggers the existing phase-independent closure response.
3. For an authorized unentered pair, adopt the received already executable state by the **original ReadyBundle meaning of adopt**, attach OC, and perform Ready/Enter atomically in the same complete handler. The receiver does not wait for its own selection job or native reconstruction job. It can discard those redundant jobs only after the received executable object has independently passed the old native predicate. A late job callback cannot overwrite the attached state or OC or change entry time.
4. Preserve and relay the complete packet and needed sidecars. The received selected body may additionally enter the local canonical pending service, but report correctness does not depend on inserting or reindexing the recipient's whole private pool. Selected-body authentication/validation has already occurred in CheckOC.

This neither assumes that the sender knows which recipient is slower nor treats a serialized database image as an executable state. If the receiver needs native conversion/import outside the existing handler, the original residual delay `u_v` from ReadyBundle:124–144 remains. OC does not eliminate it. Conversely, a recipient with a slower private reconstruction can use the fully executable received object under the old contract; no completion event from that private job is required.

## 6. Changed REPORT and first-own-session validation

### 6.1 An obligation that recovery can carry

Let `OwnDATA(P)` be the DATA blocks of the target session on the fully validated ancestry from its fixed opening anchor A through the effective parent P. The original ancestry/parent rules establish whether this list is empty; it is not determined by a local current G or private commit flag.

Every own-session DATA, including historical recovery/Completion ancestry, obeys:

- If `OwnDATA(P)` is empty, this is the **first own-session DATA**. Its effective execution parent is A by the original parent rules. It must carry a valid matching OC. If OC is nonempty, its selected request is the first batch entry, admitted at A, with no duplicate identity later in the batch. If OC is EMPTY, ordinary valid batching is allowed, including an empty batch.
- If `OwnDATA(P)` is nonempty, full ancestry validation must include its first own-session DATA and that block's valid OC. No fresh pool-selection task is introduced at P.

This check applies in Vote, NEWVIEW histories, recovery QC histories, own-session receipts, Completion and required DRAIN ancestry. A speculative first DATA only discharges the obligation for descendants **on its branch**. A recovery returning to A has no such ancestor and must discharge it again. If recovery keeps a descendant, that descendant carries the original certificate and selected request. Discarded speculative branches spend no global inclusion potential.

The regional sidecar containing OC is retained with those objects exactly as W_R is retained now. It is not a new public dependency. The original public R0 keeps the same original payload/ancestry/evidence; the selected request occupies normal DATA application capacity. Erasure removes only the regional sidecar, and the honest-FINAL argument must trace a correct Vote that checked this updated predicate. Publisher-supplied alternate hints still cannot replace that actual historical validation.

### 6.2 REPORT emission and validation

Keep the existing RC, immutable one-report-per-RC rule, fresh-action/retirement gate, ordinary entry emission time, NEWVIEW emission time and immediate independent ANCHOR vote. At the opening parent, the initializer chooses **one prepared OC in its NEWVIEW sidecar**. It does not ask every reporter to choose a different private certificate.

For `OwnDATA(P)` empty, the initializer persistently selects a valid matching OC and emits the separate signed preparation-sidecar record

`INPUT_ADVERTISE(RC, Hash(canonical_OC_bytes), OC.selected_key_or_EMPTY)`.

It supplies this signed record and the complete OC with the original NEWVIEW. A correct initializer signs one such advertisement per RC and resends it unchanged. The separate signature binds the certificate choice during forwarding: a Byzantine relay cannot replace the sidecar of a correct initializer's original NEWVIEW and make recipients report another certificate. The original signed NEWVIEW format and its equivocation predicate are unchanged. Different advertisements by a Byzantine initializer may split reports and stall its term, but are not a new Quit proof. Existing progress checks still run.

Each correct reporter uses:

```
On the existing REPORT event for completely validated parent P:
    if OwnDATA(P) is empty:
        check the initializer's signed INPUT_ADVERTISE for exact RC
        check its complete OC for exact K and its digest/key match
        emit INPUT_REPORT(RC, OPENING, OC_digest, advertised_key)
    else:
        emit INPUT_REPORT(RC, DISCHARGED, EMPTY)
    persist before send; all resends repeat this exact report
```

An invalid or missing advertisement/OC prevents only REPORT. The original NEWVIEW forwarding, ANCHOR vote and evidence handling run independently once their original conditions pass. A delayed valid advertisement can complete this report action while the original fresh-action gate still permits it; it cannot reset a timer. For a correct initializer the complete advertisement and OC arrive with the original NEWVIEW, so there is no extra network round.

For initial term zero, the initializer has OC from its Ready attachment. A correct recovery initializer returning to A also has that cached OC and supplies it in NEWVIEW. A recovery history with own-session DATA carries its historical OC through the ancestry and needs no new advertisement. An ordinary term after a terminal DATA always has nonempty OwnDATA. A future/old context never creates a missing fresh report. A correct node signs at most one report for RC even if a Byzantine initializer sends different valid advertisements.

At the opening parent, a correct leader collects only distinct-member reports for the exact OC digest/key it advertised. It checks RC and signatures; it has already validated that OC and its one selected body. Once the reports have weight `>F` and the original parent/ANCHOR QC is available, it uses OC's selected request as primary, or ordinary valid batching if OC is EMPTY. It fills remaining capacity by the original local FIFO. At a descendant parent it collects `DISCHARGED, EMPTY` reports and uses ordinary valid FIFO batching. Reports transmit no pool, body or duplicate OC back to a correct leader.

Common validation of the first own-session DATA checks its one OC, the initializer's matching advertisement and a `>F` report sidecar whose members all sign that same RC, mode, OC digest and key. A nonempty OC's body is the first batch entry; an EMPTY OC allows ordinary batching. Common validation of a descendant-parent term-first DATA checks `>F` matching DISCHARGED reports and the inherited first-own-session witness. Thus the final sidecar contains one OC, one advertisement and one report-signature set, not one OC per reporter. It never compares them against a validator's local queue.

A Byzantine low key cannot pass without its matching valid OC/body. For a covered target, an EMPTY OC cannot exist because every valid OC contains a correct nonempty offer. An untrusted relay cannot redirect a correct initializer's advertisement. Correct-only reports suffice without Byzantine cooperation because every correct recipient of a correct initializer receives and checks the same signed advertisement. The extra advertisement is a preparation-sidecar signature, not an ANCHOR vote or consensus decision.

Different OC variants, advertisement/report signer sets and witness encodings remain witness variants. They do not change the original proposal identity `(beta,r,g,t,j,Hash(B))`, QC rank or timer identity. Different selected requests change B and are handled by the original proposal/equivocation rules. Invalid sidecars cannot suppress inspection of original signed equivocation material. All OC/advertisement/report sidecars are retained and forwarded in complete regional validation dependencies.

This is a substantive replacement of requests:58–75,87–95 and the term-level inclusion argument, not a claim that a frozen cut equals the current live `E_v(P)`. It also replaces the current live minimum-across-report-keys rule: the minimum is computed once inside OC before Ready; live REPORT acknowledges the initializer's fully supplied choice of that certificate. Post-cut arrivals remain pending and use the continuing push path. They do not rewrite OC or an old report. Later roster preparations cover them by the ownership candidate's paced finite-exception proof.

## 7. Timing proof and exact absence of private work

Use Delta of the **target snapshot**, not the historical maximum d used during ownership transfer. Assume the same uninterrupted, activatable, current/unfrozen interval and complete-handler scope as the current local timing proposition. Section 9 lists the concrete operations added to that scope and the remaining resource audit.

### Ready skew

Let e be the first correct initial Enter. That sender actually sends its complete ReadyBundle+ at e. Every correct recipient processes the original native/authority/state dependencies, the finite signed-key OC and one selected request, and then enters in the same handler, or entered earlier. Under the original complete-message contract for that **admitted concrete packet**, every correct entry is at most `e+Delta`.

There is no outstanding `min E_v(A)` action in this implication. The receiver attaches an already selected witness; it does not prove or recreate the honest offer signer's minimum. This reasoning also does not establish the original native predicate's implementation bound if it was absent before the change. With an external native residual delay u_v, only the original residual-delay decomposition is available.

### Ordinary terms

The preceding complete terminal proposal/votes disseminate its QC and validated ancestry by `e+2 Delta`, as in the current proof. That ancestry contains own-session DATA. Each correct entry emits EMPTY immediately; no application request search or new state scan is required. Reports reach a correct leader by `e+3 Delta`. It has correct weight `>F`, and the complete valid first proposal is processed and voted everywhere by `e+4 Delta`.

### Initial/recovery term zero

The prepared-entry skew is at most Delta under the existing Ready/recovery interface. A correct initializer sends its full NEWVIEW by `e+3 Delta`. If its chosen parent is A, it supplies its cached OC and its signed INPUT_ADVERTISE. Otherwise its full validated ancestry supplies the historical first-DATA OC. Complete processing everywhere by `e+4 Delta` emits the original ANCHOR vote **independently** and emits the bounded REPORT action above. Both correct-only inputs reach the leader by `e+5 Delta`, and the full first DATA is voted everywhere by `e+6 Delta`.

This adds no report collection round before ANCHOR, no AcquirePending fetch, and no per-parent pool rebuild. The only application check for a new opening candidate is the same single-request check already required for DATA; the later batch retains ordinary application validation. DRAIN creates no new report.

Subsequent positions retain `2 Delta`; the p-th fresh vote is no later than `e+(2p+4) Delta <= e_v+(2p+4) Delta`. Messages due at equality precede progress callbacks. Reports do not pause BLAME or faulty-term checks. Original complete forwarding carries all new required regional sidecars, so the protecting-QC and terminal-forwarding arguments refer to the same common valid proposal at every correct replica. The `e+9 Delta` correct-initializer receipt bound retains its former derivation once these same complete packet checks are admitted.

## 8. Finite preparation, retries and inclusion proof

### Fixed-slot preparation

In the stalled-G argument, the installed r, target snapshot and exact A are fixed. Ownership v3 gives finite completed hydration conditionally on its stated communication/owner premises. Original native/authority preparation gives the exact executable A-state after finite actual preparation. Every correct selection job freezes one finite cut and performs a finite computation, then actually sends its offer. The finitely many correct members eventually supply weight `>F`; each correct collector can construct an OC. This adds an actual finite completion time to the existing preparation maximum.

The selection key K and immutable job survive beta retries. No retry replays the whole cut or reimports state. The beta-specific Ready packet binds K to the new valid handoff; its current activation checks remain separate. OC and its offer signatures are beta-independent and require no re-signing or recollection on such a retry. Only the ordinary fresh RC-bound INPUT_ADVERTISE and REPORT signatures are created in the later timed NEWVIEW/report handlers; they read the cached certificate and do not rescan a growing pool. A changed exact anchor or AUTH/native domain creates a different K and a genuinely new preparation task. A speculative recovery parent does not create a new K: it either retains the already certified first DATA, or returns to the same A and reuses OC. No public export window, pacing release, post-Enter timer or Force cutoff is reset by this work.

Consequently, in the fixed-G tail, completed selection belongs to the finite pre-Ready preparation maximum and does not add a new unbounded summand to R_i. This is conditional on the original native direct-adoption and concrete packet-resource scope; it is not a claim that every new A is prepared within Delta.

### Request inclusion using session-first DATA

Fix x with the candidate's actual-receipt/continuing-owner hypothesis, single-DATA fit and continued admission at the relevant accepted parent states. Assume x never enters G and the region has infinitely many accepted positive-output sessions. Apply the ownership candidate's paced finite-exception theorem, with no change to its conditional communication premise. Except for finitely many authority indices, every correct offer's hydration precedes its cut and covers x.

For each nonexceptional accepted positive-output session, contiguous accepted ancestry contains that session's first own-session DATA B. Let A be its fixed opening parent. A is an accepted ancestor when B is accepted, even if it and B first become accepted within one EXPORT. Since x remains absent from G, it is absent from A; by hypothesis it is admissible immediately after A. The coverage lemma makes every valid OC for this K select some y with `key(y) <= key(x)`. The mandatory first-own-session rule makes B execute such a y, with its identity absent from A.

At the canonical acceptance of B, y therefore spends a previously unaccepted logical identity in the finite set of canonical encodings with key at most key(x). The existing global extension rule prevents accepting the same ancestor a second time as fresh DATA. Later ancestry exclusions prevent spending y again in a later accepted first DATA. Speculative/discarded executions do not count.

There are finitely many such predecessor identities. Infinitely many distinct nonexceptional accepted positive-output sessions would force infinitely many strict decreases of the finite potential `Phi_x(G)`, a contradiction. Thus x enters G.

The passage from ongoing accepted DATA service to infinitely many such first blocks uses the existing finite quota per session and fixed-slot advancement rules: each session has finitely many DATA; zero-output retries contribute none; a positive-output closure or regular completion advances r. Finitely many exceptional r therefore contribute only finitely many accepted first own-session DATA. This argument does not require a finite union of future rosters or a global finite event rate.

This proof counts **session-first** opportunities, so it does not invoke the old theorem's now-false premise that every correct term-first reporter minimizes its whole live eligible pool. It still tolerates Byzantine accepted proposers: validity supplies OC, and every OC has a correct offer signer. A later correct REPORT sender can simply reuse the complete certified selected request without processing its own larger private pool.

## 9. Precise resource accounting and the remaining obligation

The construction makes the bounded predicate concrete; it cannot create undocumented capacity in the old service profile.

Let n be the fixed roster size. An OC contains at most n signatures and n keys, one K, a distinct-signer representation and at most **one** canonical request body. No raw pool, journal, historical directory, hydration signature set, list of native jobs, or nonselected request body is a Ready dependency.

One concrete canonical framing is: a typed K record; an n-bit signer bitmap; entries in ascending authenticated roster order, each containing an EMPTY/nonempty tag, the canonical length and fixed-width digest when nonempty, and that member's ordinary signature; then a length-prefixed selected body, absent for EMPTY. Signature encoding is the same ordinary signature scheme already used by the model; no aggregation is assumed. Context K is encoded once and signed by each member together with its entry. The bitmap and entry count must agree. Lengths are canonical unsigned encodings. All field/type/domain tags are included in the signed preimages.

Writing `c(K)` for the encoded typed common record, `s_v` for each ordinary signature's encoded length, `ell(L)` for a canonical encoded length, h for the hash width, and c_frame for the fixed framing, its exact object-size formula is

```
bytes(OC) = c_frame + c(K) + ceil(n/8)
          + sum_{v in S} [s_v + 1 + I_nonempty(v)*(ell(L_v)+h)]
          + I_nonempty(OC)*(ell(L_selected)+L_selected).
```

The K fields already present in the normalized Ready dependencies may be encoded by local indices into those supplied bytes; all such referents remain within this same complete packet. This formula counts them by the chosen concrete framing, rather than assuming arbitrary omissions are free. Each `L_selected` satisfies the existing singleton DATA request-fit test. This does not establish that a singleton request plus a maximal old ReadyBundle fits one old Ready envelope.

The incremental **Ready receive** cost is exactly: parse this finite framing; bind K to already validated target/anchor/native domain; check at most n distinct-member signatures and weights; compute the minimum of at most n supplied keys; authenticate/canonicalize and apply the old single-request admission/fit/ancestry predicate to one selected request; persist/attach that finite witness. No pool minimum, raw journal scan, full-directory verification or new native reconstruction is part of that handler.

The **opening NEWVIEW** additionally carries one OC/body and one initializer advertisement signature. Its receiver validates that OC once and checks the advertisement; it does not validate n different reporters' OCs. An opening REPORT contains the original RC, a mode tag, one OC digest and its selected key, plus one member signature. A correct leader checks at most n such reports against its already validated advertised OC. The final proposal carries one OC, one advertisement and one REPORT signature set; the selected body is already the first batch entry and is encoded once. Descendant REPORT is an EMPTY signed RC with a mode tag, and its original complete ancestry carries the first-DATA OC/advertisement/reports once. Retention, complete forwarding and recovery messages must preserve those objects once per normalized dependency set.

For precision, if the canonical advertisement has fixed framing c_ad, it adds `c_ad + bytes(RC) + h + bytes(key_or_EMPTY) + s_leader`. The REPORT amendment adds its mode tag and h-byte OC digest in the opening case. These are real bytes and checks, not free padding. The active correct-leader path has one OC's at-most-n offer signatures, one advertisement signature, and at-most-n REPORT signatures; it does not hide quadratic validation of a different OC from each reporter.

For an implementation with old limits `B_Ready`, `B_Newview`, `B_Report`, `B_Proposal`, `B_History` and old admitted processing/load region L_old, the actual checks are therefore:

```
bytes(original Ready normalized deps + OC)              <= B_Ready
bytes(original NEWVIEW deps + one OC + advertisement)  <= B_Newview
bytes(RC report + mode + OC digest/key)                 <= B_Report
bytes(original proposal deps + reports + OC + ad)       <= B_Proposal
bytes(original history deps + required input sidecars) <= B_History
the listed signature/body/state/retention handlers and
the actual multicast/collection dispatch loads          belong to L_old.
```

The source supplies neither these numerical capacities nor a codec/processing model from which these inequalities can be proved. In particular, adding finite signatures to a saturated old envelope is not justified merely because they are small, and two separately admissible messages do not automatically complete in the same Delta. Chunking an OC before Ready establishes finite preparation; chunking a first-Ready or live REPORT packet does not by itself preserve the old skew or report clock.

Accordingly the precise remaining deadline obligation is **this bounded OC/one-request validation and transport footprint in the unchanged old interface**, together with the unchanged original executable-state adoption check. It is no longer an unspecified arbitrary pool projection. If the existing complete regional message abstraction admits these explicitly supplied dependencies under its original profile, Section 7 proves the original deadlines without another round. If it does not, this note does not claim those deadlines, enlarge the envelope, or silently insert a new assumption. There is no concrete deployment certification in the available source to resolve that final resource check.

## 10. Edge-case audit and validation record

| Case | Required behavior / proof consequence |
| --- | --- |
| No first ReadyBundle exists yet | STATE_CANDIDATE distributes the executable original state before OC; selection signers never wait circularly for Ready. |
| H0 finishes selection before H1 | H1 validates/adopts OC and the original executable Ready state; its private scan is irrelevant to REPORT. |
| H1 has a larger or differently ordered pool | Its own offer may differ, but any covered correct offer is at most x. Common validation uses the certificate's keys. |
| x arrives during or after a frozen cut | That job is not extended. Push/retention continues; later paced roster preparations cover x. No same-context retroactive report claim is made. |
| Same logical request arrives in several wrappers | Canonicalization/deduplication preserves one logical identity and original FIFO metadata; invalid alternatives are skipped. |
| x is accepted while preparation runs | Its inclusion goal is already satisfied. Pinned evidence stays available; no false new G acceptance is inferred from a tombstone. |
| A changes | Different K; no reuse of eligibility or certificate. Actual new preparation precedes Ready. |
| Recovery chooses A | Full NEWVIEW supplies OC; first own-session DATA again satisfies the opening obligation. |
| Recovery keeps a speculative descendant | Its complete ancestry includes and validates the original first DATA and OC. Global potential is spent only if that branch is accepted. |
| Zero-output beta retry | Reuse the same K/job/OC when exact fields match; fresh RC and original current-entry checks prevent replay as a BFT action. |
| Late native/selection callback | Context/version check prevents overwrite of attached state, OC or durable entry time. |
| Byzantine low key or incomplete body | An advertisement cannot authorize REPORT without a valid matching OC/body, and a proposal must supply the same complete witness. |
| Relay swaps the OC beside a correct NEWVIEW | The initializer advertisement signature binds its exact OC digest/key; the swapped sidecar is rejected independently of the original ANCHOR path. |
| Different valid OC/report signer sets | Witness variants only; no private-queue veto, additional QC rank or timer reset. |
| Closed pair or worker retirement | No fresh REPORT/Enter past the old gate; node-level data, preparation and relay obligations persist as specified. |

The proof-verifier workflow was used as a dependency/local-step audit of this constructed argument; this task explicitly authorized protocol construction. This is not an independent whole-protocol approval. Validation was manual source reading, source hashes, causal-order and threshold checks, finite-cut/retry analysis, timing algebra, and resource-footprint accounting. No compile, executable protocol test, model checker, codec implementation, CPU/transport benchmark or proof of the mixed-snapshot environmental premise was performed.

A separate fresh focused reviewer read the completed adapter and original source interfaces and returned scoped acceptance, with no additional Critical Error or Justification Gap found. It checked pre-Ready state sharing, post-hydration finite cuts, correct-signer OC coverage, nonexclusive Ready attachment, signed-advertisement forwarding, ANCHOR-only recovery/DRAIN ancestry, beta reuse, the session-first potential proof, and the explicitly conditional timing/resource claim. Its reviewed adapter SHA-256 was `574ec470c8f7809f1b2d24c92831324bd0325e94c9150df4afe3006383238221`, before this review-record paragraph was appended. That review made no edits and did not certify the unresolved environment, codec, aggregate-load or native-adoption interfaces.

| Input | SHA-256 |
| --- | --- |
| `reviews/v3_8/goal_blocker_revalidation.md` | `4fd1f41e6cb085c50bd40ed7e39deb7f100449a4d67f11ca44e2aa1ff4c20ade` |
| `reviews/v3_8/v39_ownership_candidate_v3.md` | `a71b64b8bb845463987de8d79b92fed6d8ef61db7a60eec0719dfe1b77a67bea` |
| `reviews/v3_8/v39_fresh_ownership_v3_review.md` | `4008b823b71e316520cc3fa4a50d8e636d1db1f146599012adc633560302b2a1` |
| `sections/rotation_v3_8_requests.tex` | `dd588900506e2ef73e89a20eb57588976c2e8a9875f6258e268a87531bf98150` |
| `sections/rotation_v3_5_ready_bundle.tex` | `4048a979add0bfe1662af309d5b30f7e72b105e8d5b6fda80de415ba43330152` |
| `sync_hotstuff_rr_part2_rotation_v3_8.tex` | `c8611baf07c74f34512a182b0b3ace81d6f8f3cf65fb0cb567af696ec8a73574` |
| `revisions/v3_1_frozen/model_and_assumptions.tex` | `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387` |

Only this adapter note was written. No main file, fragment, frozen assumption, ownership candidate or prior review was changed.
