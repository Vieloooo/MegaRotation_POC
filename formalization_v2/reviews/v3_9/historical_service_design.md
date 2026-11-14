# Historical AUTH service debt: a conditional design

Date: 19 October 2026. Design note, not a protocol revision or a completed proof review. The v3.1 Model is frozen. No main source or frozen source was edited.

## Decision

A fresh pair `(beta_new, r_old)` under the unchanged `AUTH[r_old,0]` is a viable identity mechanism. To defeat the recurrent late-preparation execution, however, the scheduling obligation must be **accepted service of the exact historical voting task**, not selection, Ready, physical-region service, or one unaccepted complete result.

The concrete candidate below uses a fair persistent queue of tasks `(AUTH[r,0], i)`. Once a task receives a Force-created turn, it remains the target of zero-output retries until its own DATA is accepted, apart from temporary ineligibility. This gives a fixed authority and fixed exact anchor while that task is unserved. The existing finite-preparation and growing-beta AUTO argument can then be applied to that task. A mandatory use of the existing MSIG action prevents an infinite successful REGULAR chain from avoiding the task queue forever.

This is a **conditional design**, with two material obligations left open:

1. A historical task must still lie in the original correct-identity, communication, preparation, and regional-progress domain when addressed. Historical AUTH authenticity and ON do not prove this.
2. The task catalogue must be derived from a complete, canonical finite authority inventory. An implementation cannot substitute observers' locally discovered AUTH sets. The frozen abstract AUTH premise gives the natural source for such an inventory, but the current v3.9 interpreter does not yet specify its canonical discovery transition, particularly for a zero-output speculative session.

Under lifetime fixed-snapshot service coverage and a correct canonical inventory adapter, the candidate has a route to all four goals. Under a reading that permits an old quorum to leave that service domain, insisting on its debt can stop global DATA. Letting arbitrary other tasks keep advancing G instead restores the late-preparation starvation mechanism. This is a boundary of this candidate, not a universal impossibility result for protocol redesigns or a declaration that historical nonresponse is definitely allowed by the frozen Model.

## Source facts used

Paths are relative to `formalization_v2/`.

- `revisions/v3_1_frozen/model_and_assumptions.tex:30–44`: AUTH is immutable, fixes the snapshot and quota, and is indexed by r rather than speculative branch. A new beta may reuse r with the same AUTH and a different engine.
- The same file, lines 73–84 and 137–153: eligibility, old-certificate fault accounting, retained protocol state, and fixed snapshot weights. Neither positive weight nor an old certificate certifies future participation.
- The same file, lines 159–179, 220–253 and 295–318: authenticated L1 authority/control history, same-region complete-message processing, online Espresso readers, eventual preparation, actually entered correct turns, and request ingress. These do not supply a uniform delay for preparing every new exact anchor.
- `sync_hotstuff_rr_part2_rotation_v3_9.tex`, “The target slot of a Force”: current zero-output retries retain r, positive-output Force uses r+1, REGULAR uses r+1. The historical selector below changes the Force target rule; it is not already authorized by that code.
- `sections/rotation_v3_6_control.tex`: strict pre-payload cutoff, saved historical selection snapshots, `M = 1 + beta`, conditional cooldown, and original target-beta MSIG matching.
- `sections/rotation_v3_5_health.tex`: durable per-tip MSIG/lease/CLEAR credits, independent restoring ON before a health OFF, and per-region lease credits. The latter are insufficient to protect a particular old roster while other rosters of the same region keep producing.
- `sections/rotation_v3_9_liveness.tex`: the finite-response proof after fixed-slot/fixed-anchor preparation and the quadratic-versus-linear AUTO argument. Its finite maximum concerns one installed configuration, not all future or all historical memberships.
- `sections/rotation_v3_9_requests.tex`: persistent request owners and same-snapshot relay, immutable reports, finite canonical-key predecessors, and the conditional accepted-report-awareness proof.
- `sections/rotation_v3_9_retirement.tex`: Closed currently prohibits every fresh old-pair action; retained timers and existing messages can leave only one FINAL signature. Termination does not authorize stake withdrawal or deletion of request/data duties.
- `sections/rotation_v3_9_public_requests.tex`: import from a genuine first public carrier is eventual local work; it neither creates the carrier nor guarantees import before a particular first report.
- `reviews/v3_8/v39_historical_authority_scope.md` and `v39_recurrent_identity_trace.md`: the unresolved temporal scope of old identities and the concrete recurrent-cohort preparation/closure execution.

## 1. Task identity and persistent debt

Define a task

`tau = (chain, r, Hash(AUTH[r,0]), i)`

for each region i with positive weight in that immutable authority. The AUTH hash is redundant under uniqueness but makes the binding explicit. Task identity is not `(physical region i)` and not an old `(beta,r)` engine. A service occurrence creates a new pair under the current fresh beta; it never reuses old votes, locks, reports, timers, or Quit proofs.

Maintain a canonical queue of task-turn tokens. At most one outstanding token for a task is needed. New tasks enter at the tail in a fixed authority/region order. After a task's turn is discharged, its next token goes to the tail. A token has a durable serial number, so retransmission or a fresh beta cannot create a second token or reset its position. At every finite source position, the queue prefix and the set of newly registered tasks are finite. New arrivals never overtake a waiting token.

A turn is discharged only by a newly accepted positive DATA block produced under its exact task. Empty application batches still count as DATA, as in v3.9. Selection, Ready, entry, partial FINALs, complete receipts, raw publication, import, and DATA from another AUTH of the same physical region do not discharge it. One accepted DATA discharges at most one token. The already assigned session may complete its ordinary remaining quota afterward.

The queue head means the oldest currently eligible token. Eligibility uses governance, existing cooldown, and the task's own old AUTH, not a newer roster's positive weight. OFF temporarily parks a token without forgetting its age. If eligibility returns, its old priority returns. A permanent administrative removal is outside that task's eventual-service claim; a healthy unrelated task must not be required to wait forever for an administratively OFF token.

This replaces the Force selector's physical-region debt filter with task debt. The existing physical-region accounting can still be maintained for reporting, but its current fairness proof cannot simply be cited unchanged. The new implication is task service implies physical-region service, as made precise below. REGULAR remains unrestricted `NextLive` at r+1.

## 2. Canonical discovery, including zero output and speculation

The intended registration source is **initial AUTH records in authenticated finalized L1 history**, whether or not their sessions ever contributed DATA. Register every positive-weight regional task of a discovered initial AUTH. This deliberately does not require a request to be publicly visible or a committee to know which voter holds it.

An adequate inventory interface takes a fixed authenticated L1 cut Lambda and returns the complete finite set

`A(Lambda) = { AUTH[r,0] finalized at or before Lambda }`.

It must provide completeness, not just individually valid records. Inventory cuts must be fixed by canonical source evidence and processed monotonically. For example, an authenticated L1 publication cut of an already processed control gives a fixed public cut; processing its authority inventory is data discovery, not applying a stale control's Force or attached configuration. Any implementation using this route must specify exactly which existing evidence fixes the cut, how the complete initial-authority list is obtained, and at which canonical transition its new tasks join the queue.

The frozen L1 assumption says that authority records and control history through a finalized height are common and authenticated. A finite-cut inventory is the natural interpretation of that abstract input. It supplies neither a new deadline nor permission to infer missing entries from one endpoint's response. If the concrete interface offers only authenticated lookup for a known r and no complete inventory, this adapter is **not yet derived** from that interface. The candidate must then establish a separate discovery algorithm from the existing source records before claiming request liveness.

The following shortcuts are invalid:

- Register only producing pairs accepted into G. A zero-output or rejected speculative session can already have a correct pending-request owner.
- Register only r values occurring in public FINAL roots. An entered zero-output session can have no complete result and hence no FINAL root.
- Treat private installation of AUTH, a ReadyBundle, a pending request, or an observer's cache as a canonical public scheduling event. Different observers can see those in different orders, or never see them.
- Fetch arbitrarily many publisher-named AUTH references on the main acceptance path and call that a finite prepared scan. This reintroduces an uncharged dependency and malformed-reference obstruction.

A speculative regular session at r>0 required a complete predecessor terminal result at r-1 for its own valid entry. This is useful evidence that its lineage is real: a correct holder of that original complete result offers it under the existing publication interface, even if its branch later closes. But the predecessor's result need not contain the child's AUTH[r,0]. Nor does terminal output by itself prove that a not-yet-issued successor authority exists. Thus “register r+1 for every terminal result and wait for its AUTH” is not a complete substitute for inventory; it can create an obligation to a nonexistent AUTH. It is a possible discovery aid, not the catalogue proof.

For an actually installed old snapshot, its initial AUTH was finalized at some finite L1 height. A complete canonical inventory eventually passing that height discovers it even if the session produced zero DATA, its speculative suffix was discarded, or its local worker has terminated. No old quorum response is used to authenticate this discovery.

Inventory work uses frozen finite cuts. Later authority issuance appends later work; it cannot enlarge a current scan forever. Crucially, an already assigned task's zero-output retry does not wait for the catalogue to expand or move to a newly discovered task. This prevents an unbounded stream of new AUTH records at fixed G from destroying the finite-task preparation argument. Arbitrary archive/inventory catch-up is still only eventual, not part of the prepared retry's constant L.

## 3. Force appointment and mandatory service opportunity

Each effective Force closes the entry beta exactly as now, preserves exact pre-payload G and its original provenance, and increments beta once. The selector then appoints the oldest eligible task token, unless a currently pinned unserved task remains eligible; that task retains the appointment. Its target is `(beta_new, tau.r)` in `tau.i`, with precisely `AUTH[tau.r,g]` and quota `TB[tau.r]`.

The initial engine starts at g=t=0 with fresh state. Its endpoint is

`h(G_at_Force) + TB[tau.r]`.

That is the new pair's endpoint, not a reset of the old pair's quota. The old beta remains Closed. Any later REGULAR successor is still `(beta_new, tau.r+1)` anchored at this new session's terminal DATA and follows all existing DATA/DRAIN rules. Within one beta, these ordinary successors still increase r; changing to an older r occurs only with a fresh beta.

Before the appointed task contributes accepted DATA, contiguous acceptance cannot advance through one of its regular successors. If it is the only permitted retry task, no other task can advance G in its place. A zero-output AUTO or matching MSIG retries this exact task with another fresh beta and exact unchanged G. Do not reinterpret zero output as permission to choose a newer AUTH merely because it is easier to prepare.

Conditional AUTO cooldown is evaluated among the candidates actually permitted by this task turn. In the pinned eligible case, the old source is the only permitted candidate, so AUTO does not newly cool it down. An older cooldown is not cleared; it expires under the existing time rule. If OFF or a pre-existing cooldown leaves the task ineligible, the queue may appoint the next eligible task while retaining the old token's age. The health restriction below is needed to prevent that exception from being exploited forever against one task.

### A Force must eventually occur when REGULAR keeps succeeding

A Force-only selector is insufficient by itself. With no misses and no committee interventions, an infinite regular chain can avoid the historical queue forever.

Add the following honest-controller signing obligation using the **existing** MSIG operation and existing per-tip credit f[kappa]: when an eligible pending token is not receiving its task's accepted service, and no equivalent scheduling operation is pending, the controller eventually reserves the next unused f[kappa] and submits an ordinary future-dated MSIG targeting its observed beta, with empty attached configuration. It observes authenticated public history and the same canonical task catalogue. This is a protocol work obligation of the existing coordinated committee, not an accurate failure oracle or a new publication service. There is no fixed committee decision deadline claimed.

The public selector, not the MSIG payload, chooses the task. The record carries no discretionary retained height, AUTH replacement, or private target task. Its activation follows the original first crossing and advance-announcement inequality. If REGULAR handoffs continue under the same beta, the old target still matches. If an earlier AUTO or MSIG made it stale, it is consumed without closing the current beta or applying its attached configuration; that earlier effective Force already passed through the task selector. Do not retarget, refund, delay, or revoke the issued record.

If a control consumed the current f credit but was stale, continuing accepted DATA eventually exposes a later unused tip credit. If no DATA follows, the existing per-tip policy gives only finitely many MSIGs, and AUTO drives the pinned retry argument. Pending legacy operations retain their semantics and form a finite prefix. This describes the progress obligation, but a complete revision still needs explicit arbitration between ordinary health H1 requests and the mandatory scheduling action so that optional H1 traffic cannot reserve every useful future credit first. Reserving the first available credit for the pending scheduling duty is sufficient.

## 4. Health credits must protect a task, not just its region

The current u_i[d_i] lease credit prevents endlessly excluding a region that produces no DATA. It does not prevent endlessly excluding a particular old AUTH task while other AUTH snapshots of the same region replenish d_i. That gap matters before a task becomes continuously pinned.

A sufficient stricter signing policy adds a durable lease credit

`u_tau[s_tau]`,

where s_tau counts accepted service discharges of that exact task. Issuing a health lease for region i must satisfy both the existing credits and an unused task credit for every registered outstanding task of i. Reserve these task credits before releasing the OFF signature, alongside the existing journal reservations. Accepted DATA from another task does not renew them. The finite set examined is the complete canonical catalogue at the controller's fixed processed cut. A later discovered task inherits any finite outstanding lease backlog; its later credits cannot refund prior records.

Therefore, after a particular tau has been registered and while it remains unserved, only finitely many further health OFFs can affect its region: at most its one newly available task credit, plus the finite already reserved backlog. Each issued OFF has the existing pre-finalized finite future ON. Growing Espresso timestamps eventually leave the task governance-ON, unless an explicit permanent administrative removal takes it outside the claim. This conclusion does not certify that its old voters are online.

MSIG and CLEAR retain their existing per-tip credits. While a pinned task has no accepted DATA, G is fixed and those credits yield finitely many matching MSIG interventions. The scheduler must never treat task registration, selection, Ready, closure, receipt formation, or raw publication as a new accepted-tip credit. No accepted DATA is withheld to protect an issued MSIG; its original target-beta semantics are unchanged.

This extra lease restriction changes committee code, not the Model's service assumptions or L1 record format. Its exact replay/journal integration is part of the proposed implementation, not a property of present v3.9.

## 5. Conditional progress argument

State the temporal domain explicitly. For each task counted by the theorem, suppose its historical fixed snapshot is covered, when addressed, by the original correct-member fault bound and complete-message contract. The correct identities retain the original signing/manager capability required by the new protocol and run its handlers. The original eventual authority/native-state preparation and regional fair/exclusion/entered-turn conditions apply in that old r namespace. Do not infer any of these facts merely from `AUTH[r,0]` or current physical-region eligibility.

Under a lifetime fixed-snapshot reading, these are inherited environmental premises and keeping the handlers/keys is added protocol work. Under a currently-serving-snapshots-only reading, this is an additional historical-service premise. The frozen text has not selected between those readings, so the theorem remains conditional here.

### One continuously eligible head receives accepted DATA

Fix one task tau that is oldest and eventually eligible, with the above service coverage. Once it is appointed and remains unserved, its exact retained anchor G* and old AUTH namespace are fixed. Finite task-specific health interference ends. Its finite old roster gives a finite recovery prefix through a correct initializer after exposed equivocators' exclusion generations. Persistent fetching and preparation retain that prefix and the exact executable anchor across beta changes.

There is therefore a finite preparation time and a finite prepared response bound L_tau of the same form as v3.9. The proof is local to this one old configuration; it takes no maximum over all AUTH history. If the new retry is not closed, its first positive complete regional result is offered and its first DATA becomes accepted within L_tau. It is an original R0, not a weaker kernel or a request-only submission.

After the finite control and old-cooldown prefix, every effective AUTO immediately reassigns tau. Let a_n be real opening finalization time, theta_n its timestamp, and W_tau its unchanged positive startup base-window length. If no DATA is accepted, the same argument yields

`a_(n+1) - a_n <= L_tau`,

`theta_(n+1) - theta_n >= (1 + beta_0 + n) W_tau`,

and `theta_n <= a_n + epsilon_E`.

The accumulated lower bound on theta_n is quadratic while its upper bound is linear. Thus endless zero-output retry is impossible under these premises. A timestamp jump does not defeat the proof: there is still only one Force in the prelude and the new window starts at the realized opening timestamp. No bounded timestamp lag or fixed initial-restoration deadline is added.

The recurrent trace's escape was that other cohorts repeatedly advanced G, making each informed cohort's next exact anchor new and its preparation delay independently unbounded. A pinned task cannot be bypassed that way after the finite interference prefix.

### Queue fairness and continuing DATA

At a token's registration or requeue event, it has finitely many predecessors. Later task arrivals go behind it. If all earlier continuously eligible tasks remain within their original service domains, each receives a discharge by the preceding argument; permanently ineligible tasks are parked. The mandatory MSIG policy prevents an infinite regular chain from withholding the next appointment. Hence the token eventually receives service. Requeueing proves repeated exact-task service.

This supplies unbounded accepted DATA while eligible serviceable tasks remain available **under the task-domain condition just stated**. A physical region with at least one continuously eligible covered task is repeatedly served. This is stronger for that task than physical-region fairness, but it is not yet the same theorem as v3.9's statement quantified only over a nonempty serving subset of a currently installed configuration. An unresponsive eligible historical head is precisely where those theorem scopes differ.

### Original request progress

Let x fit alone and remain admissible after the relevant accepted parents. Suppose the frozen ingress condition gives x to a correct member of an installed historical task tau. Its persistent request owner immediately relays x inside that same old snapshot, so every modeled correct member learns it within the existing Delta bound. A retired worker is not required for this relay; the owner persists.

After that finite awareness event, repeated exact-task service supplies infinitely many accepted first-DATA histories whose genuine report sets contain a correct member aware of x, unless x has already entered G. A fresh beta has a new report context; it cannot reuse an old EMPTY report. There are only finitely many earlier pre-awareness contexts, and finite quota cannot turn them into infinitely many accepted first blocks.

The existing finite-predecessor argument now applies: each such accepted first block includes x or spends a previously unaccepted request identity with key at most key(x). The finite potential decreases until x is accepted. Its accepted DATA necessarily has a direct or same-session covering FINAL and an original complete regional result. Thus this argument produces an x-bearing complete R0 and accepted DATA; it does not merely assert that a local owner might publish x someday.

If tau was only privately installed, canonical discovery from Section 2 is essential. If tau's voters are no longer in the original service domain, same-snapshot request retention alone is insufficient. Neither missing point can be filled by the word “fair.”

## 6. Safety, regular execution, and lifecycle obligations

The intended unchanged rules are:

- Force remains an atomic pre-payload operation retaining exact G; old-version payload in the crossing block is rejected, and new-version DATA is first admissible in a later block.
- MSIG checks the entry beta. REGULAR does not invalidate that target. A stale MSIG has no Force or attached-status effect.
- Every fresh pair has one authenticated boundary fixing the old AUTH, selected region, exact G anchor, opening references, and quota. Its handoff verifier must recompute the task appointment and token serial from canonical source state.
- Regular r+1 DATA/DRAIN, terminal receipt, Completion, recovery schedules, and fresh per-pair BFT state are unchanged. An old-g Quit cannot authorize new-beta recovery.
- Waiting, Ready, Active, Frozen, Completed, and Terminated remain the six local phases. The task queue is manager/public scheduler state, not a seventh phase. The original old-beta Closed gate remains intact in this primary design.
- Accepted DATA provenance is never relabeled as belonging to the historical task being scheduled. Old receipts remain evidence for their own producing pair.

These observations make the boundary plausible; they are not a full revised safety proof. The current unique-handoff proof, source-handler dependency accounting, terminal commit order, owed-ledger reasoning, and checkpoint contents must be rewritten for the task selector. A scheduler token cannot privately replace an already installed handoff within the same beta. A finite catalogue scan must not delay an already committed terminal DATA transaction or be silently charged to a prepared source-handler Delta.

## 7. Narrow escape: complete an already selected old program after Force

Only a first x-bearing complete R0 is needed for v3.9's public import path. That suggests letting selected old work finish even though its DATA cannot append after the cutoff. There are two different variants.

### Keeping the present fresh-action gate is insufficient

Retaining the worker longer, extending only the `4 Delta` drain deadline, or preserving its existing timers does not fix the recurrent trace. The correct initializer self-votes an x-bearing proposal, Force arrives before peers' proposal processing, Closed suppresses the peer votes/timers, and the initializer's timer supplies only one FINAL. With W=3 and F=1, no complete receipt exists. More waiting cannot manufacture another signature. The original Publish guarantee has no complete-result input in this case.

### Allowing ordinary post-Force completion is a real, limited repair

If an **already entered** old pair may continue the ordinary votes, reports, recovery, and timers needed to form a complete result, then a working informed cohort can finish its x-bearing proposal despite global closure. This changes the retirement contract and its fresh-action prohibition; it does not require changing the global cutoff. The result remains historical, is offered as the original complete R0, and can be published under the existing correct-participant service. No new block format, bare-request carrier, or auxiliary consensus is necessary.

It still does not prove the full request goal:

1. **Late arrival and exhausted quota.** Take quota one and a singleton batch capacity. All correct members report EMPTY and the leader fixes the sole terminal DATA. Deliver x after that immutable proposal, even before Completion, and relay it promptly. Completing the session publishes only the already chosen batch. There is no unused DATA position for x. If x arrives after Completion, the same conclusion is immediate. DRAIN is not another DATA capacity. Giving this old pair an extra position or resetting its quota would reopen/extend completed production and is outside this escape.
2. **Selected but not entered.** An old task may have a valid handoff but still be Waiting for first preparation when Force occurs. Allowing only already entered work to finish gives it no fresh entry after Closed. At changing exact anchors, each eventual first preparation can again finish after its selection has closed. Allowing an old closed Waiting pair to enter later would itself change the original no-old-beta-entry rule; it cannot be described as merely draining existing work.
3. **A complete carrier may repeatedly omit x.** Even when recurring entered historical pairs are all allowed to finish, “some complete result per turn” is weaker than x-bearing service. With quota one, capacity one, and a retained smaller-key y that has never entered G, every report can keep choosing y instead of x. Old unaccepted completions do not spend the existing Phi_x(G) potential. A proof needs eventual accepted aware service, or a separately specified exposure-debt rule and its full validation/progress proof. Such a rule is not present in v3.9 and is not silently added here.
4. **Import is not immediate awareness.** Even after an x-bearing R0 exists, v3.9 promises eventual finite-job import after an appropriate prepared-parent event, not import before each new roster's immutable first report. A complete end-to-end argument still needs sufficiently many accepted informed report events. The scheduler above obtains them directly in the old snapshot; the carrier-only escape has not yet done so.

For case 3, a newly imported y might eventually be accepted elsewhere and remove that particular obstruction. The point is that formation of y-only complete results by itself does not establish this acceptance; current public-import timing permits new rosters to make their reports first. This is a missing implication, not a claim that y can never be accepted under any stronger scheduling algorithm.

All variants still require real old voting participation. A valid old certificate, retained request bytes, a data-serving archive, and an available signing quorum are separate resources. Conversely, it is not valid to declare an old correct roster absent merely because its worker is Terminated: current retirement preserves owners and does not authorize stake withdrawal. The temporal-domain question remains conditional in both the scheduler and this escape.

## 8. Frozen candidate and remaining work

The reviewable candidate is: canonical exact-AUTH task inventory; persistent fair task-turn queue; accepted exact-task discharge; sticky fresh-beta retry; mandatory existing-MSIG scheduling; and task-specific finite health exclusion. All DATA, receipts, publication service, strict cutoff, MSIG matching, regular rules, and six phases keep their existing meaning.

Before a manuscript can claim that this closes the goal without changing the Model, it must establish:

1. The original temporal service domain covers the historical voting identities used by the theorem, or state that the conditional theorem does not cover unavailable historical rosters.
2. A canonical complete authority-inventory transition from actual existing inputs, with discovery of zero-output/speculative AUTH and no catalogue-tail or bogus-reference blocking.
3. Durable controller arbitration and task-lease credits, including queued legacy controls, temporary OFF, and catalogue discovery delay.
4. Revised unique-handoff/source-processing proofs, plus the exact queue-fairness and request-awareness arguments, under the unchanged public cutoff and ordinary regional execution.

Without item 1, this candidate cannot simultaneously claim task-pinned request progress and v3.9's broader live-subset global-progress scope. Without item 2, an old owner can remain outside the scheduler forever even though its request is safely retained. The narrower post-Force-completion escape repairs a specific first-carrier failure but leaves late/exhausted requests, pre-entry closure, and carrier-versus-accepted-service gaps.

Validation performed: source-level design and manual invariant/timing analysis against the listed files. No TeX build, executable test, model checking, or independent verification was performed. No theorem in this note is presented as a verified result for current v3.9.
