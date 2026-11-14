# v3.9 design audit: carrying pending requests through existing complete results

Date: 2026-10-19. Scope: the frozen v3.1 model, current v3.8 source, and a possible request-transfer extension. This note changes no protocol source or frozen assumption.

## Finding

There is a usable **conditional existing carrier**: a request whose full authenticated bytes already occur in ordinary DATA, or genuinely required original evidence, of a complete regional result `R0` can become publicly readable through the existing Publish Layer. This remains true when Force prevents that result from extending `G`. A reader can retain the request for a later proposal without accepting the old DATA.

I did not find a construction using the existing v3.8 session allocation that takes **every request known only to old correct members** to such an `R0`. The missing step is formation of a request-bearing complete result before the old roster loses its available DATA positions. Neither full-result preservation nor existing evidence fields supply that formation step. A subsequently proposed **fresh historical-AUTH session** is a substantive different route: frozen authority semantics permit it, subject to new scheduling and existing-snapshot service obligations detailed in section 7. The negative conclusions below concern the existing allocation and the named carrier/continuation candidates, not a universal impossibility theorem.

## 1. Exact inherited interface

The frozen model defines the input as DATA with a direct or covering FINAL receipt and its required ancestry, boundaries, and source evidence. A proposal QC or a receipt missing payload is insufficient (`revisions/v3_1_frozen/model_and_assumptions.tex:114–126`). The bounded service begins only after that input is available (`181–196`). The frozen EXPORT is a nonempty DATA interval plus the evidence required to validate it; DRAIN may occur in supporting ancestry but is not exported as application DATA (`revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex:1338–1357`).

The current distinction is explicit:

\[
R_+=(R_0,W_R),\qquad \operatorname{PublishInput}(R_+)=R_0.
\]

`W_R` contains new regional INPUT_REPORT sidecars. Public erasure leaves zero extra REPORT bytes; the chosen primary consumes ordinary DATA capacity (`sections/rotation_v3_8_requests.tex:157–199`). Logical normalization preserves the old objects and source denotations, but creates no payload, receipt, authority, or append permission (`sections/rotation_v3_8_carrier.tex:17–29,108–141`). In particular, an old tuple position is not spare capacity for a newly required object.

All network charges below use a named fixed AUTH snapshot. There is no old-only to new-only direct-delivery charge to `max(Delta_old,Delta_new)`.

## 2. A concrete restricted construction that does work

Call a complete result **x-bearing** when its unchanged public `R0` contains the full canonical authenticated request `x` in an original required object. The strongest case is an ordinary DATA batch in its direct or covering DATA ancestry. Another possible case is an actual required old recovery/quit witness containing an admissible signed DATA proposal whose batch contains `x`. This second case is incidental evidence carriage: it requires that particular witness to be part of the original complete input, not an optional attachment invented for transport.

The sender and reader actions can be specified as follows.

1. The old correct holder constructs and offers the ordinary complete `R0` through its existing publication obligation. It supplies the same old DATA, receipt, and required evidence; it adds no request field or sidecar.
2. The ordinary service publishes that exact logical result within its original size/load conditions. If `R0` was already publicly recorded, use its authenticated occurrence; do not demand a corrected wrapper or another publication deadline.
3. At complete namespace acquisition, a reader retains the existing raw source objects. A request-import handler enumerates actual DATA batch objects in the supplied old structure and any publicly resolved original dependencies. It authenticates each request, deduplicates by its logical identity, and retains its bytes. Application eligibility is checked against the reader's exact prospective parent before reporting or batching. Merely importing `x` does not execute it or import the old speculative state.
4. The handler does not condition this retention on the containing EXPORT advancing `G`. It can import a request from closed-version evidence while leaving all current-version, exact-parent, quota, and strict Force-cutoff checks unchanged. Inclusion in `G` remains the only final removal event.

The reader rule is a proposed local protocol addition, not a claim that v3.8 already imports requests from every such source occurrence. The existing `PublicStore` already retains complete raw namespaces, including rejected records, and hydrates the legally referenceable history (`sections/rotation_v3_7_source_step.tex:5–23,33–80`). This supplies the bytes; request indexing and the order of import relative to REPORT emission must still be implemented. No arbitrary old-to-new RPC is needed.

### Authority, record, capacity, and future bytes

| Obligation | Existing justification for this restricted construction |
| --- | --- |
| Authority | The producing pair's actual admitted boundary, immutable `AUTH[r,0]`, and entered view/recovery authority; a pair label alone is insufficient. |
| Valid record | Ordinary DATA below the fixed endpoint, with a genuine direct or same-session covering FINAL receipt and the original complete validation evidence. |
| Request fit | `x` is an ordinary admissible batch entry that fits the DATA batch; the complete resulting `R0` must satisfy the original complete-result limit. Individual request fit alone is not a proof of total result admission. |
| Extra public bytes | Zero relative to that same old-format `R0`. No new reporting, ownership, or request-list field is supplied. This is not a claim that an x-bearing DATA costs the same as an empty DATA. |
| Publication start | The first time the entire original `R0` is available, not request receipt, one vote, a QC, or one local commitment. |
| Readability | Covered online history-following readers receive the complete result by `s + T_pub + T_E`. Later readers use retained authenticated history under the original eventual retrieval premise, without a constant catch-up bound. |
| Global permission | None is added. Closed-version DATA remains unappendable; an imported request may be executed only in a fresh valid current DATA proposal. |

### A sufficient complete-result formation interval

There is a concrete favorable case, rather than merely an invocation of publication. Suppose an authorized correct leader completely broadcasts a fresh valid DATA proposal containing `x` at time `t0` in its fixed snapshot. Suppose all correct members remain able to execute their original fresh Vote action during its dissemination, the proposal has its original complete dependencies, and no genuine Quit cancels its timers before expiry. The usual nonconflict and entry conditions of that valid proposal are required.

All correct members vote by `t0 + Delta`, their timers expire by `t0 + 3 Delta`, and their FINAL messages reach the correct collectors by `t0 + 4 Delta`. The correct members alone have weight greater than `F`, so these messages form a complete receipt. If Completion cancels a timer because a terminal receipt already exists, its complete ancestor coverage supplies the needed result instead. With the original full evidence already available, an x-bearing `R0` is available by that last bound.

A Force learned after the needed votes have occurred need not prevent this favorable case: it does not itself cancel an existing timer. The eventual publication may be after the external cutoff and hence carry historical evidence only. This interval proves the actual formation step for this restricted case. It does not show that an arbitrary pending `x` obtains the fresh position or the uninterrupted interval.

Nor does public availability alone prove the REPORT theorem's premise. Future members must import `x` before enough of their immutable reports. A finite historical hydration/import requirement through an authenticated opening cut is a possible downstream design, using existing public bytes; it needs a separate scheduling and startup proof. It cannot repair a request that never reached any public `R0`.

## 3. Can old required evidence serve as a general pending-request channel?

The answer is conditional by record type.

| Candidate field or object | What is actually authorized | Why it does not generally carry pending-only `x` |
| --- | --- | --- |
| DATA batch | Execute authenticated requests under the exact parent, within quota and application capacity. | This is the working path, but first requires a remaining authorized proposal position, selection of `x`, and complete-result formation. |
| DATA ancestry | Preserve the actual hash-linked history needed by the result. | A pending request cannot be inserted into an already signed ancestor without changing its hash and obtaining the required new protocol evidence. |
| Signed PROPOSE/QC in required evidence | Preserve the actual proposal and its justification. | A QC alone is not a publication input. A new x-bearing proposal is still subject to admission, quota, signing gates, and subsequent completeness. |
| Recovery STATUS/NEWVIEW | Entry lock, highest certified opening, actual entry proof, and their validation dependencies. | These messages do not have a pending-request payload. A required x-bearing historical proposal can carry `x` incidentally, but such a proposal need not exist. |
| Quit/equivocation proof | Genuine matching BLAME signatures or actual conflicting admissible signed objects. | Correct nodes cannot manufacture equivocation; no faulty signer is required to place `x` in the proof. A BLAME statement carries no request body. |
| FINAL | Session, fixed configuration, DATA height, DATA hash. | The receipt attests existing committed DATA; it is not an additional application payload or a signature over a pending pool. |
| DRAIN | No batch; same DATA height and terminal references; FINAL is sent for terminal DATA. | Adding `x` violates DRAIN semantics. Repeated DRAIN generations do not create request capacity. |
| L1/source/native evidence | The actual authority, reference, registry proof, or native validation context required by the original interface. | Source fields cannot be redefined as request containers. Application-specific spare witness material is not guaranteed for every admissible request/application. |
| INPUT_REPORT | New regional sidecar, erased from the public `R0`. | Even a public key/hash would not supply the request preimage; the unchanged service does not transport this sidecar. |

These distinctions follow from the actual signed fields and recovery rules (`sync_hotstuff_rr_part2_rotation_v3_8.tex:795–809,837–887,1256–1327`) and the old carrier denotations (`sections/rotation_v3_8_carrier.tex:31–58,85–97`). Padding, alternate signature encodings, freely chosen hashes, or attaching a request list to a native proof are not a general solution: the frozen interface supplies neither arbitrary semantic payload space nor its capacity. Hash binding supplies authentication, not reconstruction of absent bytes.

An extra x-bearing object placed beside a valid `R0` also does not solve the issue merely because the outer EXPORT grammar is structured. The guarantee concerns the original complete result; preservation of that result is compatible with omission of unrelated new attachments. A required evidence object that already contains `x` is different from making a formerly unnecessary object required in order to smuggle `x` into the guarantee.

## 4. Continuing an old snapshot after Force or quota

### Existing v3.8 behavior

The current gate prohibits fresh old-version DATA/DRAIN proposals, votes, reports, terms, and recovery entries once authenticated Closed is recorded. It allows old timers to expire and existing signatures and complete results to be collected and relayed (`sync_hotstuff_rr_part2_rotation_v3_8.tex:1066–1078`; `sections/rotation_v3_8_retirement.tex:84–110`). Thus it can finish the favorable pre-existing-work case in section 2, but cannot build a new proposal for a pending-only request after closure.

The `4 Delta` drain is explicitly not a complete-result formation theorem. Some timers may be canceled by genuine Quit, and too few matching FINAL signatures may remain. The retirement theorem expressly declines to claim old-session progress or complete-receipt formation (`sections/rotation_v3_8_retirement.tex:201–252`). Persistent ownership keeps the data available; it does not grant a fresh DATA position.

### A deliberate redesign allowing historical continuation

One must distinguish the current gate from the frozen environmental model. The frozen v3.1 text does not say Force itself cancels regional timers, changes locks, or supplies Quit (`revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex:129–132`). A new protocol could investigate continuing already authorized historical work while keeping its output externally unappendable. It would be inaccurate to reject that entire design space solely by citing the later v3.8 gate.

Nevertheless, merely removing that gate does not give the requested general construction:

1. The old session's endpoint is fixed. Recovery does not reset its quota, including speculative DATA already in the proposal ancestry.
2. At the endpoint, no next DATA term exists. Recovery creates only DRAIN, which carries no requests. Completion stops recovery and production.
3. A new `(beta,r)` requires an actual authorized boundary and selected region. Historical `AUTH[r,0]` is not a permit to invent extra branches or standalone request-bearing sessions. A new slot's AUTH can contain a disjoint roster.
4. A historical continuation still needs genuine recovery authorities, entered correct leadership, its regional commit interval, and an original-size complete result. Existence of one currently live serving region is not a proof that every old request owner has an authorized remaining result and finishes one.
5. Continuing many old engines creates a publication/concurrency obligation. Finite work per engine alone does not show that aggregate input load remains inside the frozen service envelope.

Accordingly, a limited “finish at most the already available DATA suffix” extension is a plausible restricted design project. It is neither an existing v3.8 action nor a general pending-request transfer proof. Extending the quota, adding a request-bearing DRAIN type, or manufacturing new same-roster sessions would change the constraints posed for this task.

## 5. Minimal capacity/arrival case

Consider one governance-ON physical region, so the system always has a live region and regular selection may return it. Its current slot has `TB_r = 1`; the next slot's immutable roster is disjoint. All current members can be correct and all old-to-old and new-to-new communication can satisfy their respective bounds.

1. The sole old DATA proposal `B` has already been signed and voted, with an ordinary batch that does not contain `x`. Its first-DATA reports were issued earlier.
2. A correct old member receives admissible `x` while `B`'s commit timers are still pending, persists it, and dispatches the specified within-snapshot relay. `x` fits alone in a DATA batch and has finitely many queue predecessors.
3. `B` commits and obtains a complete `R0`; publication and global acceptance can succeed normally. The session completes at its fixed endpoint. Its unchanged required witness need not contain `x` anywhere.
4. The next slot installs the disjoint roster. An old-to-new packet may be arbitrarily delayed under the stipulated fixed-snapshot network scope. Nothing in the existing publication guarantee applies to the old pending queue entry.

This is a finite local execution exposing the unavailable carrier. It needs no fault, unavailable publisher, full-pool overload, or Force. No deadline or finite finishing policy can edit `B` after its signatures were fixed, and DRAIN adds no application payload.

With Force instead, the same issue arises when `x` arrives before closure but after the last relevant old proposal is fixed. Closure may even leave no complete result. If a complete old result exists, it still need not contain `x`.

This finite case does not by itself prove permanent noninclusion: the client or a delayed direct message can eventually deliver `x` to a later serving member. The current v3.8 liveness discussion already separates that eventual delivery from pre-report awareness (`sections/rotation_v3_8_liveness.tex:466–495`). Repeating a late-arrival pattern across arbitrary disjoint rosters additionally requires checking the exact identity-level meaning of the frozen fair-leader premise; this note does not claim an unconditional infinite counterexecution without that check.

## 6. Timing, reservation, and admission gates

**A fixed wait before reporting or closing.** Any finite old/new transfer wait has no covered cross-roster delivery bound. For same-snapshot receipt, a wait of `Delta` can disseminate an already known `x`, but an arbitrary valid request can arrive after the intake cut or after the last proposal was fixed. This is a gap in coverage, not a reason to alter Delta.

**Wait for an empty pending queue before terminal DATA.** An empty queue at one node says nothing about late arrival elsewhere. A closing snapshot can synchronize a finite intake cut, but the frozen request interface allows arbitrary admissible requests to arrive later. Those requests still need an owner and a carrier. Continuous arrivals can also keep a moving empty-queue condition false indefinitely.

**Reserve final DATA capacity.** A real reserve is useful for a known finite set that fits. It cannot cover every old-only pending request under a finite quota: a finite burst can exceed the reserve, and a late request can arrive after its batch is signed. “Each request fits alone” does not imply “all pending requests fit in the remaining suffix.” The task supplies no new reserve capacity.

**Do not acknowledge a request until its DATA/receipt is public.** This can define a stronger acknowledgement contract, but it does not transport the bytes of an already received admissible request. Restricting the liveness target to acknowledged requests makes public carriage a premise of admission and drops the very pending requests at issue. It is not a repair of the unchanged target.

**Freeze ingress, flush, then permit the new roster.** This needs a rule for requests that arrive after the freeze, a proof that the finite old cohort fits the remaining quota and forms complete results, and a path from post-freeze old receivers to the next roster. It cannot delay a scheduled Force's strict external cutoff. If the successor's Ready or first DATA waits for an unguaranteed old transfer, even a single live successor may stop producing forever. Timeout release preserves base DATA progress but restores the missing-awareness case.

**Use advance announcement to finish the flush.** The frozen announcement rule gives prospective control visibility. It supplies no lower bound tailored to the old pending backlog, remaining quota, recovery-authority preparation, or completion of a particular x-bearing result. A timely announced Force is not a scheduled request-publication grant.

**Force until the old roster receives a new attempt.** Zero-output retries can reuse a slot and its AUTH, but they do not make a departed roster eligible at every later slot. Positive accepted DATA advances the target slot; the next AUTH may be disjoint. Forcing otherwise healthy growth to preserve an old request owner is not implied by the existing selector or fairness proof.

The existing no-wait relay and ordinary publication path preserve the current base DATA-progress argument because they introduce no new blocking handoff. Every stronger gate proposed above needs an independent termination argument under the unchanged services; the existence of at least one live region does not supply it.

## 7. Separate constructive route: a fresh session under historical AUTH

The following direction was raised after the first carrier audit: do not first transfer `x` to a disjoint new roster. Instead, authorize a genuinely new session `(beta_new,r_old)` under an already known historical `AUTH[r_old,0]`, anchored at the current pre-payload `G`, and arrange recurrent accepted service for historical snapshot tasks.

### Authority and DATA semantics permit this form

The frozen authority paragraph expressly says that authority is indexed by `r`, a new branch version reusing `r` uses its unchanged authority, and the pair `(beta,r)` identifies a different engine (`model_and_assumptions.tex:30–44`). It does not state a model-level monotonicity axiom for `r` across all beta changes. Thus a revised deterministic Force-boundary algorithm may choose historical `r_old`, provided it actually authenticates and selects that target and retains the fixed old AUTH. This changes the current target-slot algorithm; it is not available merely by sending a historical authority record to current `CanEnter`.

For the new pair the initial anchor is the current retained `G`, with its actual old producing provenance and commitment evidence. Its endpoint is

\[
H_{\beta_{\rm new},r_{\rm old}}=h(G)+TB_{r_{\rm old}}.
\]

Consequently it has ordinary fresh DATA capacity. It does not reset the quota or locks of `(beta_old,r_old)`, reopen that pair's Closed record, reuse its vote position, or turn DRAIN into DATA. All old pairs stay closed. An authorized MSIG remains a future-dated Force with the existing target-beta/control fields; the choice of historical slot must be derivable from the new public scheduling algorithm, rather than secretly adding an unauthenticated target field to its payload.

Each such new version begins strictly after its Force block, and every proposed block uses the new beta, actual historical configuration, exact fresh boundary and current anchor. These facts make original-size complete regional results possible. Ordinary R0 publication then has exactly the carrier proof in section 2. The frozen model therefore does **not** justify declaring this route impossible on the grounds of old quota exhaustion or Closed alone.

### A conditional formation and request-inclusion argument

Let a task identify a particular historical regional snapshot `(i,r_old)` containing a correct owner of `x`. Suppose the owner's within-snapshot relay has completed, the correct recipients retain the request, and this same snapshot meets the original regional communication/serving premises whenever its task is served. Suppose its task remains eligible and receives infinitely many fresh visits that contribute accepted term-first DATA, while `x` remains admissible after the relevant current parents and fits alone.

Every sufficiently late genuine report in those visits has an aware correct signer: the same old correct membership already holds `x`, and no old/new communication edge is required. The first-DATA report rule therefore executes `x` or an as-yet-unaccepted request with key at most `key(x)`. The finite-predecessor potential of v3.8 decreases on each such accepted visit. Hence `x` eventually enters ordinary DATA accepted into `G`. Its ordinary complete `R0` supplies the desired public bytes; disjoint later rosters can recover them through existing history. This is a conditional positive theorem, not just an empty-result carrier.

“Revisit once” is insufficient if the old queue contains smaller requests. Likewise, raw publication without accepted service does not automatically spend the v3.8 potential: repeatedly discarded predecessors may recur. A scheduler should discharge its historical service debt on actual accepted DATA, and give each still-relevant historical task repeated successful service, or supply a different proved request-selection potential.

### What is code, and what still needs a premise audit

* **Persistent pending owners and fresh-pair dispatch are protocol choices.** A retired worker need not be the process that receives the new assignment. Keeping its node-level owner, historical membership, bytes, source receive route, and a fresh-pair constructor does not by itself add a new channel. New member identities can coexist with the old historical task; no overlap is needed.
* **The fixed-snapshot Delta is applicable to the replayed roster.** Every actual regional message is still old-member to old-member in the exact unchanged AUTH. No physical-region union or cross-roster bound is used.
* **The accepted-lifetime fault clause must be checked, not automatically replaced by a new forever-honest-key premise.** The frozen bound counts compromises while that snapshot's certificates remain accepted (`model_and_assumptions.tex:145–150`). If those certificates have remained accepted as historical/anchor evidence throughout, replay uses that already ongoing lifetime condition. If a design had actually ended their accepted lifetime, it cannot resurrect old keys after arbitrary intervening compromise and pretend the original bound still applies. Old stake withdrawal is not authorized merely by worker termination.
* **Possession of an old AUTH is not itself voting availability.** The frozen model explicitly makes global DATA recoverable without the original producing region staying online (`model_and_assumptions.tex:218–224`). Its delivery services have online-reader conditions. Current liveness explicitly does not cover permanently missing correct quorums in assigned snapshots (`sections/rotation_v3_8_liveness.tex:358–362`). Therefore “at least one live current region exists” is not, by itself, a proof that every historical roster can resume voting. If the modeled correct historical members are required to retain their active node-level service and fresh-pair duties while pending requests persist, that is a candidate protocol lifecycle rule. Its compatibility with the task's live-region meaning must be established; it must not be presented as a theorem derived from storage retention alone.
* **Governance eligibility remains real.** Historical task selection cannot bypass an effective OFF under the frozen eligibility rules. A request confined to a permanently ineligible historical region is not serviced merely because another region is live. For the original “requests reach regions that continue serving” premise, the design must identify a suitable eligible historical owner snapshot, rather than assuming all historical regions remain ON.
* **Fair scheduling is an algorithm obligation.** A growing historical task set needs an explicit enumeration/dovetail policy; new tasks cannot permanently overtake a fixed old task. Positive-output transitions may change tasks, whereas zero-output retries need a fixed chosen slot to retain the existing finite-parameter progress argument. A permanently unserviceable historical task cannot pin all base DATA forever when only an unrelated snapshot is live. Conversely, skipping it by timeout does not prove request progress for its owner. The scheduler must prove both properties under the precise inherited serving conditions.

The concrete formation proof for one replayed task can reuse the fixed-slot regional response after preparation: the historical AUTH sequence is immutable, the current anchor is finite and recoverable, and original DATA/FINAL generation produces an ordinary complete result. The existing global proof cannot simply be cited for the entire new scheduler, because its stalled-frontier argument relies on a fixed target slot and a finite installed parameter set. That invariant must be re-established.

**Assessment:** historical AUTH replay is a viable protocol-design direction under the frozen authority and carrier interfaces. It escapes the old-quota example by creating an authorized new engine, rather than by publishing a bare request. It does not inherently add a permanent-membership, old/new transport, or publication-capacity assumption. A full construction still needs an explicit fair scheduler, a consistent historical-node lifecycle/availability interpretation, and base-progress proof. Under the weaker interpretation that only some current snapshot is live while any needed historical quorum may be permanently absent, this route alone does not establish the requested general liveness.

## 8. Result to carry forward

The useful verified boundary is:

\[
\begin{aligned}
&\text{original admissible complete }R_0\text{ contains the full bytes of }x\\
&\quad\Longrightarrow\quad
\text{existing bounded publication and retained public history can deliver }x.
\end{aligned}
\]

The missing general implication is:

\[
\text{old correct member retains admissible }x
\quad\Longrightarrow\quad
\text{an authorized x-bearing }R_0\text{ eventually forms}.
\]

Full original evidence can sometimes carry `x`; it cannot be treated as unlimited request capacity. Continuation of the same historical pair can sometimes finish such a result; it cannot reset that pair's completed quota. Timing and admission sequencing can cover a declared finite cohort; they do not cover every pending request without solving late-arrival ownership and transport. A fresh historical-AUTH pair is a different, conditionally constructive route that can supply new ordinary DATA capacity and keep all messages within the original snapshot. No complete v3.9 scheduler/lifecycle/progress proof is established here.

Validation: source inspection and explicit local construction/failure cases only. No executable model check, transport/capacity benchmark, LaTeX compile, or independent whole-protocol verification was performed. Only this review note was written.
