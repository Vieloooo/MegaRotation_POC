# Fresh conditional verification of the historical-owner candidate

## Summary

**Final Verdict:** The candidate is viable but incomplete due to two procedure-level **Justification Gaps**; no **Critical Error** was found in its conditional ownership, hydration, or finite-exception argument.

This is a fresh, source-level verification of `reviews/v3_7/v38_ownership_candidate.md`, treated as the proposed algorithm and argument. It is not a verification of another review and is not a replacement protocol. No manuscript changes or protocol repairs were made.

**Conditional premise accepted for this audit.** I grant the candidate's broad mixed-snapshot interpretation: a persistent correct historical owner and a correct future member of the same region have the claimed bounded complete-message communication while their protocol duties continue, with the candidate's finite-prefix allowance `d_i(r)`. This permits examination of the algorithm beyond the already-known transport issue. It does **not** establish that interpretation from the frozen source. Frozen Model lines 167–179 state a within-region bound, whereas the current request and retirement procedures explicitly transport within a fixed installed snapshot. Their use of managers after worker termination does not by itself establish a bound between disjoint snapshots. The candidate remains conditional on that interpretation.

The audit also uses the candidate's express scope: persistent correctness of the original owner, globally locally finite execution events, finite installed rosters and quotas, ongoing correct handlers, inherited individual message and request admissibility, and continued accepted service at the region. It does not grant an unstated physical-departure deadline or a uniform bound over all future configurations.

### List of Findings

- **F1 — Justification Gap: the fragmented-record completion branch has no specified immutable finite slot budget.** **Location:** candidate line 74: “If one request plus authentication material needs multiple allowed messages, use explicit, individually bounded fragment slots and their complete processing”; line 155: “ensure finite per-request slot schedules.” The descriptor commits a journal length and root, but the supplied algorithm does not define how the receiver obtains a finite authenticated fragment count or equivalent stopping bound for each indexed record. Finite journal length bounds the number of records, not the number of attempted fragments. A complete-record timeout of `2d` proves the unfragmented case; the stated fragmented alternative needs an actual fixed finite schedule and rules for a missing, inconsistent, or changing size declaration. This is an acknowledged implementation obligation, but it is also a dependency of the abstract termination claim in lines 104 and 151. Conditional downstream analysis below assumes that obligation is met; it does not supply the missing procedure.
- **F2 — Justification Gap: signature production is specified, but certificate dissemination/collection is not an executable rule.** **Location:** candidate lines 86–90: “may v sign” and “A target member collects distinct matching signatures”; line 104: “the correct-only HYDRATED threshold forms.” There is no stated destination, dispatch event, persistent outgoing duty, or collector rule for a newly produced HYDRATED share. The weight inequality proves that sufficient correct signing weight exists, not that one process obtains it. For example, with three unit-weight correct members and `F=1`, all can finish and retain their own signature locally while no one has weight `>F`. This is a permitted missing-action execution of the literal procedure, not an impossibility result for its intended implementation. If “collects” is intended to include a reliable share-distribution procedure, that procedure is an unexpressed premise; after granting it, the threshold argument is correct.

Neither finding is the mixed-snapshot-model caveat. Neither shows that the theorem is false under a completed implementation satisfying the stated finite-transfer and certificate-delivery obligations. They prevent certifying the candidate's claim that the actual finite completion procedure is already complete.

The concrete codec, aggregate dispatch/load envelope, and report/Ready integration remain expressly open in the candidate. Those scope limits are recorded below rather than counted again as independently discovered errors.

## Detailed Verification Log

### 1. Frozen inputs and verification boundary

The following SHA-256 values were recorded before substantive verification. The frozen Model's working copy and the v3.8 review snapshot were byte-identical.

| Source | SHA-256 |
| --- | --- |
| `formalization_v2/reviews/v3_7/v38_ownership_candidate.md` | `047741b4c05cd6cacf9d638027270db2f9935d7f1f9f5fab26b831ef91844680` |
| `formalization_v2/revisions/v3_1_frozen/model_and_assumptions.tex` | `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387` |
| `formalization_v2/reviews/v3_8/snapshot/baseline/model_and_assumptions.tex` | `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387` |
| `formalization_v2/sync_hotstuff_rr_part2_rotation_v3_8.tex` | `c8611baf07c74f34512a182b0b3ace81d6f8f3cf65fb0cb567af696ec8a73574` |
| `formalization_v2/sections/rotation_v3_8_requests.tex` | `dd588900506e2ef73e89a20eb57588976c2e8a9875f6258e268a87531bf98150` |
| `formalization_v2/sections/rotation_v3_8_retirement.tex` | `984482c763cce1a9f324e416e78d1708f553acbe4a1798f7cafdc46ef289c238` |
| `formalization_v2/sections/rotation_v3_8_liveness.tex` | `480cea02b3499f98d9e2b4b9b8eaaa2afd17ec55409c901b465b31758ff98541` |
| `formalization_v2/sections/rotation_v3_5_ready_bundle.tex` | `4048a979add0bfe1662af309d5b30f7e72b105e8d5b6fda80de415ba43330152` |

No other review, previous verdict, or current agent's notes were read. The proof-verifier skill supplied the verification procedure; no domain reference or external protocol claim was needed.

The dependency chain under review is:

1. A persistent original owner and a finite authenticated authority directory give a fixed discovery point.
2. Bounded descriptor replies and a finite timed record procedure give a finite acquired cut.
3. Actual all-target dissemination followed by a delivery wait gives one signer's coverage fact.
4. A `>F` certificate authenticates at least one such completed fact.
5. Pre-Ready completion places coverage before the term reports without adding a whole-pool delay after first Ready.
6. Globally local finiteness plus finite per-slot quota bounds exceptional old cuts.
7. The current first-DATA report rule converts repeated accepted aware opportunities into inclusion of a fixed admissible request.

F1 concerns step 2 and F2 concerns step 4's eventual formation. In accordance with verifier mode, each missing conclusion is assumed temporarily for the independent downstream checks; it is not silently proved or repaired.

### 2. The claimed communication domain

> “A historical correct owner and a future correct member of the same physical/logical region are therefore endpoints of that regional service while they have a protocol duty.” (candidate line 25)

This is accepted **conditionally** here. Frozen Model lines 172–179 give bounded correct-to-correct delivery and complete-dependency processing within a region. They do not explicitly define mixed-snapshot endpoints or select a delay parameter for such a pair. Current retirement lines 15–19, 57–68 and 203–215 identify the session's fixed membership; current requests lines 38–44 expressly distinguish within-snapshot delivery from disjoint-roster transfer. Thus existing archival handlers support phase independence, but do not independently settle the membership-domain question.

> “Taking a maximum would not repair an interpretation under which old and new same-region endpoints have no communication guarantee at all.” (line 27)

Correct. With the broad service premise granted, `d_i(r)` is finite for a finite authenticated prefix and may be used consistently in the candidate's cut and multicast analysis. It cannot create a missing edge, and this review makes no such inference. A maximum over all future rosters is unnecessary.

### 3. Directory, ownership, and finite retained cuts

> “D_i(r) = union of the regional identities in AUTH[k,0], 0 <= k <= r.” (line 37)

For fixed finite `r`, finite rosters give a finite directory. Immutable AUTH ordering fixes membership without consulting a private current-owner oracle. Correctness does not require every listed identity to have been selected, to run an engine, or to answer. Repeated identities can be deduplicated while their historical authentication evidence is retained. Recovery authorities change schedules only, so excluding `g>0` from ownership epochs agrees with frozen Model lines 30–44.

Obtaining all required old AUTH evidence has eventual, not constant, timing. The frozen progress premises explicitly assume required authorities become available (lines 307–309). The candidate adds these finite prefixes to the required preparations. It cannot obtain a numerical prefix-acquisition bound from the regional timeout.

> “The original owner retains the pending bytes even after one or all target rosters receive them.” (line 53)

This supplies the needed stable discovery invariant. A future target need not discover private successive custodians. Pending bytes are removed only after verified membership in monotone `G`; an acknowledgement or HYDRATED certificate is not a removal event. This is sufficient for any fixed globally unaccepted `x` held by the persistently correct original owner.

> “Requests first delivered to a member of a future AUTH are covered once that identity enters the relevant authenticated prefix.” (line 55)

The later proof correctly introduces a finite index `k` containing that identity and discards targets `r<k`. The scheme does not establish ingress to an identity outside all applicable authenticated membership. That is an explicit boundary, not a missed discovery guarantee.

> “Repeated queries for the same key reuse the cut; beta never restarts it.” (line 47)

An immutable `(i,r,v)` cut prevents fresh arrivals and beta changes from extending an in-progress task. The intended authenticated request is the query of target member `v`; this review reads that identity as the authenticated requester, not an attacker-controlled claim about a different requester. No second independent preparation for that same correct member/key is needed by the algorithm.

`ReceiptJournal` may grow without bound over the whole execution. Each captured finite-time prefix is finite under the candidate's local-finiteness premise. The persistent prefix metadata allows a descriptor response without a full scan. The actual indexed-read implementation remains part of the candidate's explicit mechanics obligation; no bounded database rebuild follows merely from naming this data structure.

### 4. Descriptor deadlines and correct-versus-Byzantine responses

> “Only after the complete context is available through the corresponding regional receive handler does the timed step begin.” (line 61)

This ordering is necessary: a `2d` exchange cannot include an unresolved AUTH fetch. Subject to the accepted complete-context delivery premise, a correct query sent at `s_v` reaches a correct live owner by `s_v+d`; its maintained-metadata response reaches `v` by `s_v+2d`. Processing due messages before the deadline covers equality. No test identifies correct owners, and no Byzantine response is needed.

> “A silent identity contributes no descriptor … at most the first complete locally accepted descriptor.” (line 72)

Correct. A finite directory and at most one accepted descriptor per identity give a finite descriptor vector. Byzantine equivocation need not be resolved consistently across hydrators. A Byzantine can advertise a large finite integer count, but cannot enlarge an already frozen descriptor. This establishes a finite number of **record slots**, with no useful uniform work or resource bound.

> “A correct indexed reply is either the still-pending full request or a signed tombstone for a request already in G.” (line 70)

Tombstones do not create false global acceptance: they are not used as acceptance proofs. If the owner is correct, a globally unaccepted request is never replaced by a tombstone. If the owner is Byzantine, omitting its slot was already allowed. An owner may delete accepted request bytes without jeopardizing the theorem about a request that remains globally unaccepted.

> “For each slot, send an indexed record request and allow a full 2d query/reply interval … A missing, tombstoned, or invalid response consumes its finite slot and is skipped.” (line 74)

For one admissible complete reply per record, this is a valid finite sequential procedure: each slot consumes a bounded interval, and every correct owner's still-pending record is received. Byzantine silence consumes time but cannot make a slot wait forever.

**F1 occurs at the subsequent fragmented-record sentence.** The fixed descriptor's `length` counts journal records, while a record may require several physical messages. The note does not specify the record's fragment count, when it becomes immutable, how it is authenticated, or what finite stopping rule applies if the purported fragment description itself never completes. A finite count of opaque records does not establish a known finite sequence of fragment attempts. Per-message delay only applies once the relevant actual message is dispatched; it is not a termination rule for an unspecified multipart exchange. Calling for “explicit … fragment slots” names the requirement but does not instantiate it.

This finding is a **Justification Gap**, not an asserted infinite valid request or a refutation of finite fragmentation. The candidate may restrict its certified abstraction to complete single-message records, or discharge its stated fragmented-transfer obligation, but this report does neither. For the remaining checks I assume each acquired record has a fixed finite transfer schedule satisfying the inherited message profile and that required dispatch actually proceeds.

### 5. All-correct multicast and the HYDRATED threshold

> “For every request in Pool_v, v sends the full request to every target identity.” (line 84)

`Pool_v` is the finite acquired cut pool, not a moving copy of all arrivals in the live pending store. Under that reading, new arrivals do not extend this schedule. Each correct target's manager receives and persists these packets even without an engine or with another beta's worker archived. For required bytes sent at time `a<=b_v`, complete processing by `a+d<=b_v+d` follows from the granted communication premise.

> “Only at or after b_v+d … may v sign HYDRATED.” (lines 86–88)

The temporal direction is correct. At the signature event, every correct target already holds the relevant bytes. A certificate containing that signature cannot exist earlier. A local acquisition by another target may remain unfinished without affecting its actual knowledge of those multicast requests. Completion must mean the actual final sends and completed reception processing, not enqueuing a pool-transfer job.

> “A target member collects distinct matching signatures of weight greater than F … Correct target members alone can supply that weight.” (line 90)

The certificate's **soundness** follows from frozen Model lines 137–150: Byzantine weight is at most `F`, so `>F` includes one correct signer; correct total weight is `W-B >= W-F > F`. The signed target authority and directory-prefix commitment prevent conflating different configurations. The proof needs no correct intersection with a later REPORT certificate.

**F2 concerns formation, not soundness.** Producing a signature and having enough potential signing weight do not alone place those shares at any collector. No send or relay event for these new shares is specified. The concrete three-correct-member example in the Summary leaves the new Ready gate permanently unsatisfied while all local hydration procedures finish. This is an omitted action in the written algorithm. Temporarily assuming reliable dispatch and collection, finitely many correct preparations suffice to form a certificate without Byzantine cooperation.

> “Every correct target may adopt the valid HYDRATED certificate and stop redundant local acquisition.” (line 96)

This is safe after valid certificate delivery. Its correct signer's completed multicast, not an unresolved pool pointer, supplies coverage. The stoppage condition preserves existing serving duties. It cannot create a bootstrap deadlock by itself: if no certificate exists, this stopping rule has not fired; once one exists, redundant work is unnecessary for the asserted coverage.

### 6. Ready skew and beta-independent preparation

> “Ready requires this certificate … The first ReadyBundle supplies the bounded certificate and exact immutable target context.” (line 100)

Under the previous conditional conclusions, all correct target recipients have the required request bytes before the first correct Ready. The first entrant can carry the HYDRATED certificate in its complete ReadyBundle. A slower recipient needs to validate the certificate, not fetch the signer's pool or finish its own historical cut. Therefore the candidate does not intrinsically insert an arbitrary pool wait between the first ReadyBundle's processing and Enter.

Current ReadyBundle lines 60–96 require sending the complete bundle in the Ready/Enter event and entering in the receiving complete handler. Its `Delta` skew conclusion is conditional on that handler's bounded processing (lines 107–144). Adding HYDRATED preserves the shape of that argument only if the augmented certificate and target-context checks fit the same complete-message interface. The candidate expressly reserves that bounded-check/size audit; it has not established a concrete unchanged byte or CPU budget. In particular, the larger historical `d_i(r)` is paid before signing and does not itself replace the target snapshot's post-entry `Delta_i`.

> “Its variable preparation cost is paid before first Ready and retained across every beta retry.” (line 102)

This separation is correct. `(i,r)` and `(i,r,v)` state excludes beta. Under a stalled `G`, the existing retry proof fixes a slot `r*` (current liveness lines 44–59); its directory and correct target roster are finite. Once the finite-transfer and share-delivery gaps are granted, the finite maximum of their preparation times can be included in the existing preparation maximum. No numerical whole-pool term is then required inside the already-prepared regional response `R_i`.

This is not a proof of a startup deadline for arbitrary growing histories, nor does it reset the public export clock. The argument only says that retained preparation eventually finishes for the fixed stalled frontier under the stated availability and continued-dispatch premises.

### 7. Arbitrary future rosters, stale cuts, and duplicate certificates

> “Fix a request x received … by a persistently correct owner p … Let k be a finite authority index placing p in the historical directory.” (line 112)

These are the right quantifiers for the candidate. If `x` is assumed never to enter `G`, owner `p` retains it forever. For any later target `r>=k` and correct signer whose first cut query begins after `t_x`, the owner's query-processing cut contains `x`. The preceding hydration argument gives universal target awareness before HYDRATED authorizes Ready. No roster intersection is needed.

> “There are only finitely many target preparations whose first correct query began before t_x in a locally finite execution.” (line 116)

This inference is valid with **global local finiteness**: a bounded real-time prefix contains finitely many protocol events across all identities. Per-process local finiteness alone would not imply it when the identity universe may grow. The candidate's finite-exception sentence is read as explicitly invoking the former, stronger conventional execution property; that property is not separately stated in the frozen Model and is a premise of this candidate's argument, not a consequence of the network delay bound.

A pre-`x` cut or already-issued HYDRATED certificate can correctly omit `x`. It is not required to be revoked or refreshed. The proof charges its target to the finite early-start set. Such a certificate does not authorize arbitrarily many fresh quota allocations: zero-output beta retries retain the same immutable slot quota; positive output causes the next installed slot at the next boundary. Consequently a finite set of target indices contributes only finitely much accepted DATA, even if it has many speculative retries, delayed certificate encodings, or repeated deliveries.

Duplicate shares and alternate certificate signer sets do not create new target indices. An immutable pre-`x` REPORT for one old parent/term can likewise be reused only within that exact context. Current requests lines 283–289 explicitly account for finite pre-dissemination reports and delayed publication. Neither duplicate evidence nor stale publication converts a finite set of pre-awareness contexts into infinitely many accepted first-DATA opportunities.

> “No finite union of every future roster and no uniform future Delta is assumed.” (line 118)

Correct. The proof uses a fixed original owner, one finite prefix directory per target, and a finite exceptional event prefix. Each later target may have new members and a different finite preparation cost. The argument establishes eventual inclusion by contradiction, not a common deadline across those costs.

The push claim for a target already registered with `p` is also correctly separated: once `p` actually dispatches a complete new request, delivery is bounded by the granted same-region service. An arbitrary backlog or fragmented complete send is not bounded merely from initial application receipt. This distinction matters for the candidate's resource scope but is not needed to revive an old HYDRATED certificate's cut guarantee.

### 8. Conversion of awareness to accepted request inclusion

> “Some correct signer in every report certificate then supplies an eligible candidate y <= x … No logical primary can be fresh in accepted ancestry twice.” (line 133)

Given the awareness conclusion, this matches the current v3.8 conditional request theorem (requests lines 243–272). Consider an accepted first DATA while `x` remains absent from `G`. Its exact accepted parent ancestry cannot already contain `x`; otherwise acceptance of that ancestry would already include it. With the candidate's maintained admissibility and individual-fit premises, an aware correct reporter has `x` in its eligible set and reports `y<=x`. Since every `>F` report witness has a correct signer and all correct target members are aware, the witness cannot avoid that bound. The minimum executed primary `z` satisfies `z<=y<=x`, is real, and is absent from accepted parent ancestry.

The candidate's example `(encoding length, encoding bytes)` has a finite lower set. The current source uses `(encoding length, Hash(encoding))`; with its stated binding/canonical-body interpretation, bounded encoding length likewise gives a finite number of possible canonical identities. This difference does not defeat the finite-potential argument. An accepted primary cannot be charged twice because `G` never rolls back and later accepted ancestry includes it.

The inference needs infinitely many accepted term-first opportunities, not merely infinitely many proposals or locally committed speculative blocks. Finite per-slot quota and finite per-term position count provide that counting bridge for infinitely many accepted regional contributions; current requests lines 274–281 make the term-first ancestry argument explicit. A Byzantine accepted proposer does not defeat the mandatory report predicate checked by the genuine correct FINAL signer.

The candidate correctly withholds a whole-protocol verdict for its added Ready gate, report timing and validation integration. The implication is valid conditional on actual augmented-valid accepted first DATA and the preceding ownership obligations. It does not prove application-state permanence for an arbitrary initially admissible request.

### 9. Worker termination, physical exit, and clients

> “Worker Terminated never stops this owner service.” (line 139)

This is a coherent node-level duty. Current retirement lines 130–174 already separate worker release from persistent request/data/control/publication owners and explicitly deny that Terminated authorizes stake withdrawal. The candidate extends request ownership with a stricter original-copy rule; it does not claim a short whole-node termination bound.

> “Atomic ingress shutdown precedes release … a complete admissible request received during shutdown is persisted.” (line 141)

This prevents dropping an already-delivered complete request in the exit race. For a fixed globally unaccepted `x`, its owner cannot pass the stated pending-request exit guard. Empty retired identities may therefore be skipped without losing a required original copy. Any actual remaining consensus, archive or transport duty must still be met; the pending-request condition is a necessary exit guard, not permission to violate other correct-node obligations.

> “Correct clients retain their request until G inclusion and retry against authenticated current regional membership.” (line 143)

This client action is compatible with the theorem's ingress premise but is not a proof of bounded ingress or a guarantee that a retry succeeds. A failed attempt to a physically departed endpoint is not successful receipt by a persistent correct owner. No bare-request Espresso service is imported.

> “For a node that has stopped ingress with a finite pending set … taking their finite maximum eventually discharges the request-specific exit guard.” (line 145)

Conditionally correct: finitely many requests, each satisfying the request theorem, have finitely many finite inclusion times. Their maximum is finite. This only discharges the request-specific guard; ongoing archival or control duties may still prevent physical departure. Infinite new arrivals, permanent lack of regional service, and independent physical-exit deadlines are expressly excluded.

### 10. Resource scope and final validation status

> “Finite transfer proves eventual completion in the abstract synchronous model; it does not establish practical overload resistance, bounded storage … Concurrent preparations must not manufacture aggregate load exceeding the unchanged declared envelope.” (line 156)

This is an appropriate limitation. A finite Byzantine descriptor can cause arbitrarily large finite work and retained metadata. All historical owners answering all future targets may accumulate unbounded storage and outgoing duties over an infinite run. Neither individual request fit nor finite work per preparation proves compatibility of simultaneous preparations with an unchanged aggregate network/load profile. New-request pushes also need actual dispatch under that profile. These are explicit deployment obligations, not benefits supplied by synchrony.

The frozen publication assumption remains restricted to original complete regional results; none of this analysis applies `T_pub` or `T_E` to CUT queries, request bodies, HYDRATED shares, or bare requests. The ownership protocol neither chooses global finality nor changes Force cutoff.

**Validation performed:** manual dependency and adversarial case analysis against the frozen sources listed above, including correct/Byzantine timeouts, tombstones, beta retries, stale cuts/certificates, disjoint future rosters, Ready ordering, report potential, and exit races. No executable model checking, proof assistant, transport benchmark, or compilation was performed. No manuscript or frozen assumption was edited.

**Integrity:** the final SHA-256 comparison after report creation passed: all eight source entries in the manifest were unchanged.
