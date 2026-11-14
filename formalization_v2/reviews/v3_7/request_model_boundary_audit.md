# Request liveness: frozen-model and protocol boundaries

Status: read-only source audit, 19 October 2026. This report changes no manuscript, model, algorithm, or user decision. It distinguishes what the sources already require from behavior that a future protocol extension would have to implement and prove.

## Result

1. **Part II's first-receipt FIFO and oldest-fitting batch selection are executable protocol rules.** The frozen Model separately assumes request ingress to continuing serving regions, individual batch fit, and finitely many local predecessors. Part I's common admission order cannot be imported: Part II explicitly allows different node orders and redirects Propose to its own request section. Authorization to evolve the protocol therefore does not, by itself, freeze the FIFO algorithm; neither does it authorize assuming a stronger common input order.
2. **Worker termination does not terminate the correct replica or its retained obligations.** A phase-independent pending-request handler at a correct old member that still owns the responsibility can be an algorithmic extension using the existing intra-region message bound. It must specify the retained payload, owner, receive route, and actual send. This is different from assuming that every former member stays online forever, that arbitrary archives respond within `Delta`, or that old members acquire future voting/leadership rights.
3. **No source supports one finite time by which all present and future roster members know `x`.** Even pointwise eventual delivery to every future member would not imply that uniform time, or a common finite predecessor set. Part II's literal ingress condition is at region granularity; its all-member implementation across changing snapshots still needs an explicit mechanism.
4. **The all-correct churn example exposes a real finite-service/FIFO obstruction under class-wise or eligibility-scoped actual-turn fairness.** It is not a proof under every possible reading of the unquantified sentence “Correct leaders receive repeated turns.” A reading that requires each particular correct identity to receive infinitely many future entered turns would exclude permanent identity replacement. The frozen source does not formalize that reading, and it must not be silently adopted or silently dismissed. No universal FIFO impossibility follows.

## Sources and identity checks

All line numbers below refer to these files under `formalization_v2/`:

| Alias | Exact source |
|---|---|
| M | `revisions/v3_1_frozen/model_and_assumptions.tex` |
| B | `revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex` |
| I | `revisions/v3_1_frozen/sync_hotstuff_rr_part1_core.tex` |
| V | `sync_hotstuff_rr_part2_rotation_v3_7.tex` |
| R | `sections/rotation_v3_4_retirement.tex`, included by V:2153 |
| S | `sections/rotation_v3_7_source_step.tex`, included by V:1621 |
| L | `sections/rotation_v3_7_liveness.tex` |
| E | `sections/rotation_v3_7_exports.tex` |
| D | `reviews/v3_7/request_fifo_rotation_design.md` |

Direct comparisons confirmed that the entire current v3.7 Model section equals M, the current request/retained-data subsection equals B's corresponding subsection, and current Part I equals frozen I. The audit read the full frozen Model, the request and retention clauses, the current retirement and source-retention handlers, the relevant Part I assumptions/proof, and the current liveness argument. These are current-source conclusions, not an older review verdict.

## 1. Environmental input conditions versus executable request rules

| Clause | Source | Boundary |
|---|---|---|
| Fixed committee and static Byzantine set in the standalone core | I:151–156 | Part I's original scope. Part II replaces the voting configuration with session-fixed weighted regional snapshots; it does not make the union of future memberships finite. See M:135–150 and L:56–59. |
| Correct requests eventually reach all correct replicas; common admission order; finite predecessors; individual batch fit | I:208–220 | Part I's input-fairness environment. Its request proof uses that common finite predecessor order at I:935–944. |
| Correct leaders select oldest fitting pending requests, skip ancestry, retain pending requests and validation data | I:216–220; I:362–369 | Normative correct-process behavior, although written inside the same assumption box. Propose explicitly calls that behavior. Being inside an assumption box does not turn leader code into a separate transport service. |
| Requests reach regions that continue serving, fit in DATA batches, and have finitely many older requests in each local queue | M:310–312; B:503–505; V:535–537 | Frozen Part II input/progress conditions. They do not state a common order or a uniform predecessor bound across different queues. |
| Enqueue in first-receipt order; duplicates do not move position; later arrivals do not overtake; node orders may differ | B:1841–1845; V:2223–2227 | Part II queue algorithm. The explicit permission for different orders overrides the stronger standalone Part I common-order condition for this composition. |
| Skip requests in proposed ancestry and fill with oldest remaining requests that fit | B:1845–1848; V:2227–2230 | Part II correct-leader selection algorithm. V:1101–1111 explicitly redirects Propose to this section, rather than to I's common-order assumption. |
| Remove a request finally only after inclusion in `G`; discarded-suffix requests remain pending | B:1850–1853; V:2232–2235 | Protocol state/retention duty. Regional commitment and handoff are not discharge events. |

There are therefore two different changes that must not be conflated:

- Changing the queue/selection/transfer algorithm is a protocol change. Under the stated authorization to evolve the protocol, it is not automatically a change to an environmental assumption. It would nevertheless have to be described explicitly and have its own safety and request-liveness proof. If a particular candidate promises to preserve first-receipt FIFO, moving an inherited request ahead of an existing local predecessor violates that candidate's promise.
- Assuming that clients, an external inbox, or all future members already provide a common globally fair order strengthens the input environment. Neither Part II's local-FIFO clause nor Part I's superseded common-order sentence supplies that fact. Implementing an authenticated order through specified protocol actions would be a different matter, requiring a concrete implementation and proof.

The inherited fair-schedule sentence at B:2103–2105 preserves the regional schedule/exclusion/commit-window conditions. It cannot erase the explicit Part II request-order difference. L:17–32 accordingly spells out the inherited leader-schedule formulas, not a reinstatement of Part I's common request order.

## 2. Historical request owners and the regional communication bound

### What the frozen source already says

M:152 requires correct replicas to keep protocol state and committed blocks. M:167–179 gives an authenticated correct-to-correct message bound within region `i`, including processing of complete validation dependencies. It contains no `Active`-engine or current-session qualification.

The original B already distinguishes production from serving retained data: a Complete engine “only serves data” at B:927–930. Force keeps discarded-suffix requests pending and leaves old engine logs/timers intact at B:1552–1564. B:1850–1858 requires pending-request retention and preservation of required block/receipt copies. B:2207–2212 requires retirement/checkpoint rules to preserve required copies. Physical/staking retirement is expressly unfinished at B:2137–2143; the frozen source does not equate leaving a roster with permission to drop every outstanding duty.

The accepted-data archive has a separate boundary. M:218–234 explicitly makes globally accepted DATA recoverable without the original producing region staying online, with eventual historical retrieval and no constant catch-up bound. This allows an old producer to cease being necessary once its relevant obligations have been discharged or transferred. It does **not** place a never-published pending request into Espresso history.

### What v3.7 makes operational

V:1708–1715 says Completed can still have data duties and Terminated ends BFT timers/signatures while durable workers retain evidence and duties. V:2141–2151 requires responsibility transfer, leaves discarded-suffix requests available for reproposal, and explicitly says Terminated does not authorize stake withdrawal.

R:15–21 retains historical membership context even with no worker or a Terminated record. R:24–41 keeps first complete-receipt relays alive through termination and applies the existing regional message bound to the actual relay. R:57–83 requires complete-close handling for historical snapshots before a tombstone/duplicate response. R:130–156 requires an actual persistent owner with evidence and receive routing before resource release; the transfer may be local to the same correct replica. R:183–210 expressly uses `Delta` delivery to a manager/archive with no worker. R:223–236 denies a bound on historical catch-up, storage copying, or all remaining retirement work. S:33–50 and S:73–81 likewise make retained bytes and durable ownership protocol obligations, with no automatic finite-storage bound.

These clauses already establish that **phase-independent service at the correct historical member is compatible with the existing model**. Adding a pending-request receive/respond/relay handler to that persistent owner need not introduce a new availability premise merely because the owner's engine is Terminated. The new handler would still be new code; the source does not already specify a complete pending-request transfer protocol.

### The precise permissible scope

| Proposed use of an old owner | Source-justified classification |
|---|---|
| Keep request bytes, duplicate identity and its existing queue order in the same correct replica after its worker stops; route incoming requests to its persistent handler | Algorithmic retention/routing extension consistent with B:1850–1858, V:2141–2149, and R:130–156. The original node has not disappeared merely because its BFT program exited. |
| Send a complete protocol message between correct members of the same historical fixed regional snapshot while they retain those duties | Uses the existing intra-region bound, as R:183–210 already does for closure. The pending-request algorithm must name the actual event and provide its dependencies. |
| Transfer a responsibility to another local worker on the same correct replica before releasing the old worker | Existing permitted ownership mechanism, R:149–153. A new service premise is unnecessary. |
| Retain an old queue owner until its captured obligations are discharged, or define an explicit transfer to a continuing owner | A possible algorithmic responsibility/retirement rule, not automatically an environmental stability assumption. Its representation, discharge criterion, fault handling and effect on membership progress remain proof obligations. This report does not select such a rule. |
| Assume any former member is still reachable after its responsibilities were transferred/discharged, or after its physical departure was otherwise allowed | Not implied. Worker-retention rules do not establish perpetual online availability of every historical identity. |
| Charge a remote archive fetch, cross-region send, or arbitrary transfer between unrelated future snapshots to an old snapshot's `Delta_i` | Not established. M:121–126/B:314–319 supplies no separate cross-region transport service; the historical-snapshot argument does not identify a mixed-configuration endpoint set or the applicable bound when `Delta_i` changes with authority. An explicit supported communication path is needed. |
| Restore a huge queue/state database or finish unbounded local work inside one newly named “atomic” handler | Not justified by finiteness. S:238–250 expressly rejects turning arbitrary CPU, database import or historical backlog into a `Delta_i` operation. |
| Let an old owner issue new-session votes, or force its future appointment as leader, solely because it retains a request | Not implied by retention or networking. Membership/authority and current signing gates remain separate; M:30–44, M:145–150, R:84–91 and R:143–147 still apply. |

Thus the defensible answer is neither “old members are unavailable” nor “all old members are available forever.” An unfulfilled duty survives the **worker**. A protocol may continue using its correct, retained owner within the already modeled communication domain. It must identify an actual duty owner and cannot silently extend that domain, resurrect departed endpoints, or infer future leadership from storage responsibility.

## 3. Request dissemination under unlimited roster changes

The literal frozen condition is M:310–312: requests reach the **regions that continue serving**. It does not explicitly quantify over each future authority roster or explain whether a regional ingress event means receipt by a particular member, all correct members, or a durable regional inbox. The all-correct-members wording in I:214 concerns its fixed committee and cannot alone settle this changing-membership interface.

For a known finite snapshot, a protocol can specify that a correct recipient multicasts the complete request to that snapshot. The existing correct-to-correct bound then gives a finite delivery time to that finite recipient set, if the message is in the modeled domain. The act of sending, correct intake, request identity/admission checks, duplicate handling, and ownership across later changes must be stated; eventual authority availability alone performs none of them.

Even granting the stronger pointwise property

```
for each future serving snapshot r and each correct member v in that snapshot,
there exists a finite time t(x,r,v) at which v receives x,
```

does not give

```
there exists one finite T after which every present and future serving
member has already received x and retains one common finite predecessor set.
```

The quantifiers cannot be exchanged over an unlimited sequence of new identities. Future identities may not exist at `T`; their local finite predecessor sets can differ and be arbitrarily large. A finite maximum is justified only for a fixed finite set of recipients/tasks. The current proof explicitly confines its finite maxima to one installed slot and the finite region cycle, and denies a finite union of all future memberships at L:44–59 and L:94–103. S:66–71 makes the same finite-task distinction for hydration and authority preparation.

The region-level source condition also does not guarantee that a particular `x`-aware identity remains scheduled after roster replacement. Conversely, it does not forbid the protocol from making retained request obligations survive that replacement. Dissemination, persistent responsibility, and accepted service are three different steps; none should be inferred from the other two without the corresponding handler/proof.

## 4. Audit of the all-correct FIFO churn execution

### What the trace actually establishes

D:96–108 installs fresh finite all-correct rosters, gives each member the same finite local prefix before `x`, and allows each identity only a finite number of credited initial-leader sessions before the next roster. With batch capacity and quota one, each credit discharges one local predecessor. The roster can finish its entire prescribed finite service round while `x` is still pending. The next roster brings a different finite predecessor set.

The queue arithmetic is sound. There is no need for unbounded predecessors at one node, undelivered `x`, Byzantine behavior, discarded request bytes, or a failure of publication. The old nodes can remain correct and retain `x` indefinitely without receiving another leadership appointment. Retention alone does not make their queue the selection queue of a new member.

The single-region construction is mechanically compatible with **current v3.7**: V:1775–1786 explicitly allows same-region renewal when no eligible alternative exists. In frozen v3.1, B:1443–1449 instead requires another region and B:2118–2119 warns that one eligible region can wait indefinitely. That is an algorithm-version difference, not a reason to reject the current trace or to alter the frozen Model. If the older selection algorithm were being tested instead, two persistently eligible alternating regions could instantiate the same per-roster queue obstruction.

Unlimited fresh all-correct identities violate no static-fault restriction by themselves: each session has a fixed roster, no identity is corrupt, and the explicit current argument does not assume a finite union of future memberships. Sources: M:135–150; L:56–59. Authorities can remain immutable and be supplied before use; the example needs no authority-rebinding or delayed-authority premise.

### Actual-turn fairness is the unresolved interpretive boundary

The explicit inherited formulas are:

```
for every g0, some g >= g0 has a correct initializer;
for every fixed g and t0, some t >= t0 has a correct term leader.
```

They appear at I:208–211 and L:17–32. They quantify over **correctness of scheduled positions**, not infinitely many appointments of each identity. All-correct rosters satisfy them. An unentered scheduled position is still not an actual service event.

M:297–300 adds: “Correct leaders receive repeated turns that are actually entered, including recovery initializers and DATA-producing turns across sessions.” It supplies no identity-level quantifier, persistence condition or treatment of a member after removal. Its possible readings have different consequences:

| Reading | Consequence for D:96–112 |
|---|---|
| Correct leaders as a class receive infinitely recurring actual entered DATA-producing opportunities across sessions | The all-correct trace satisfies it. It produces accepted nonempty DATA forever. |
| Every continuously eligible/continuing identity must repeatedly receive entered turns; removed identities have no continuing appointment entitlement | Finite fresh rosters are not excluded. A particular `x`-aware owner need not be continuously eligible. |
| “Repeated” requires several actual turns for each member during its membership | The literal one-credit trace should be expanded. Give each member any prescribed finite number `k` of turns and use `k*m` distinct predecessors before `x`. The same finite-service obstruction remains. |
| Every particular correct identity must receive infinitely many actual future turns, even after it would otherwise leave the roster | Permanent fresh-roster replacement violates that reading. The trace cannot be called admissible under it. The source does not formalize or reconcile that reading with later snapshot changes. |

The words “including recovery initializers” also must not be read as requiring recoveries in an execution where every session completes correctly without a quit. The literal core schedule assumptions specify available recovery assignments; they do not force needless recovery. If instead the added prose is intended to require actual infinitely recurring recovery entry regardless of faults, that would itself need clarification before classifying a no-recovery execution.

Consequently D:108's blanket assertion that all progress premises hold needs an explicit fairness interpretation. D:112 correctly identifies the absence of an explicit after-removal identity entitlement, but that absence does not prove that the strongest reading was ruled out. The precise conclusion is conditional: **any membership policy allowed by the chosen existing fairness meaning that replaces all queue owners after a request-oblivious finite accepted-service budget admits this local-FIFO obstruction.** The source alone does not establish an unconditional theorem over every interpretation of the prose.

Even identity-level repeated **entry** is weaker than repeated **accepted initial-leader service**. The current v3.7 counterexample at L:447–472 expressly gives correct recovery leaders actual entered proposals whose results miss the cutoff. Hence choosing the stronger fairness reading would not, by itself, discharge the remaining request-liveness proof. The proposed ancestry/cursor argument requires the particular credited initial-leader opportunities described in D:34–56; this audit does not independently certify that new lemma.

## 5. Permissible refinement versus added premise

| Item | Algorithm/refinement that can be investigated without silently changing the frozen environment | Claim that cannot be assumed from the sources |
|---|---|---|
| Local selection | Explicitly revise or retain the first-receipt FIFO code and prove the chosen request property | A common client admission order already holds in Part II |
| Pending requests | Persist payload/order/obligation state beyond a worker; specify receive/relay/transfer/discharge actions | All future members already know every request at one common finite time |
| Historical owners | Use the existing correct replica's persistent owner for actual supported intra-region messages while it owns the duty | Every historically listed node remains online and immediately responsive forever |
| Membership | Define and verify a request-aware admission/retirement policy, including Byzantine blocking and durable transfer cases | Stable membership, a finite future identity universe, or infinite service of a particular request-aware identity without deriving it |
| Authority schedule | Specify an actual authority-producing/checking algorithm with immutable issuance and authenticated service-credit processing | The source issuer automatically follows a new cursor because required AUTH records eventually become available |
| Public ingress | Prove a concrete protocol path using inputs already covered by its actual service assumptions | `T_pub` covers bare requests, READY messages or a weaker input; E:201–210 expressly denies this |
| Acceptance | Prove that the relevant honest request-bearing proposal obtains accepted credit despite cutoff and churn | Repeated regional credits or actual entered correct turns already imply accepted request-bearing proposals; L:436–478 separates them |

For the AUTH row, M:30–44 establishes immutable authority contents, while M:199–206/B:392–399 makes the honest multisignature service responsible for STATUS, CLEAR and MSIG FORCE. It does not name an existing honest programmable AUTH issuer. A concrete issuer or contract implementation is protocol work; asserting an extra honest issuer behavior without implementing/authenticating it is an added premise. Rejecting noncompliant immutable source records downstream also needs a progress argument and cannot alone force the original source to replace them.

This map leaves genuine design work open. It does not prohibit algorithmic request ownership across rotation, adopt permanent membership, change FIFO, or select a stronger fairness meaning for the user.

## Verification scope

This was a source-level boundary audit. Exact comparisons checked the frozen Model, request subsection and Part I identity. No manuscript was edited, no protocol was implemented, and no model checker or full proof verifier was run. The only file written by this audit is this report.
