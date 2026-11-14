# Request progress after the v3.5 review

Status: design research, not an accepted revision or a whole-protocol proof. This note changes no manuscript source. The immediate v3.6 publication repair and the future request-progress extension should be reviewed separately.

## Conclusion

The v3.5 request problem is not repaired by increasing miss tolerance, retaining a slot after one more kind of failure, or extracting requests only from late complete results. These mechanisms do not yet ensure that an honest request-bearing proposal becomes a complete result or that its contents become an enforceable public obligation.

A promising future extension is an **input-report witness on the first DATA of each term**. A proposer collects signed request reports of total weight greater than `F`, includes the minimum reported request as an actual transaction, and supplies the reports as validation evidence. A correct report contains a pending request selected under an order with finitely many predecessors. This makes even Byzantine-proposed accepted DATA supply input progress. It uses existing regional communication and complete-result publication; it does not publish a bare client request through an assumed external request service.

There is a conditional request-inclusion argument for this extension, below. The input-size envelope, exact changing-membership dissemination semantics, and integration into regional validity need resolution and an independent review. This is an explicit change to proposal construction and validity, including a reserved request ahead of ordinary FIFO filling. It must not be described as the existing batching algorithm or silently included in a minimal publication fix.

## Frozen source boundary

Paths below are relative to `snapshot/`. `main` means `sync_hotstuff_rr_part2_rotation_v3_5.tex`; `liveness` means `sections/rotation_v3_5_liveness.tex`. I read the completed `fresh_assumptions.md`, the immutable source and relevant fragments, the complete v3.1 Part II baseline, and the complete Part I baseline. I did not consult running reviewers' drafts.

The constraints relevant to this design are:

- Complete regional results are DATA plus a direct or covering FINAL receipt and validation dependencies. `T_pub` begins only when this complete object exists; an already recorded result can satisfy the promise. Neither a private proposal, an isolated FINAL, nor an input report receives this guarantee. See main:405–420 and baseline Part II:312–336, 374–391.
- Every regional QC/receipt requires distinct-member weight greater than `F`; this guarantees a correct signer. Correct members alone have enough weight. Membership is fixed within a session, not explicitly across every future slot. See baseline Part II:338–357.
- An effective Force retains pre-payload `G`, closes the old version immediately, and leaves surviving old timers to drain. Retirement prevents new votes and entry after verified closure; it does not manufacture the missing FINAL signatures. See main:1046–1060, 1133–1139, 2161–2195.
- Authority and native-state availability are eventual. There is no uniform startup/recovery delay across changing slots and anchors. See baseline Part II:496–511; main:1786–1819, 2031–2054, 2087–2108.
- Correct-client requests remain pending until they enter `G`; speculative inclusion and regional commitment do not remove them. Local FIFO orders need not agree. See main:2249–2262 and baseline Part II:1840–1859.
- Part I's proof of request inclusion uses infinitely many **successful correct proposals**, not merely infinitely many entered correct turns. Its fixed-committee request argument and its input-fairness assumption are at baseline Part I:208–223, 930–943. The Part II serving conditions are at baseline Part II:487–511.

The publication-bound error identified by `fresh_assumptions.md` is an independent upstream issue. Every implication here that begins with continuing accepted DATA remains conditional on repairing and proving that global-progress property.

There is a precise assumption-versus-algorithm distinction here. Part I places both environmental fairness and correct-leader input behavior inside `ass:fairness`: common admission order, oldest-pending selection, and pending-until-commit. Part II expressly specifies per-node FIFO instead, with removal only at `G`. Therefore one cannot import Part I's common queue order or all-members wording without reconciling the Part II interface. The proposed primary-request reservation changes the inherited **behavioral** FIFO clause. It does not strengthen network, authority, client-delivery, or publication service. If "preserve the v3.1 assumptions" includes preserving that exact correct-leader FIFO behavior as a hard constraint, this candidate is outside that constraint and must not be adopted. In the more permissive interpretation that allows an explicit input-selection algorithm change while retaining environmental premises, it remains a future extension subject to review. This note does not decide that scope question on the user's behalf.

## A stronger obstruction to late-result carryover

The existing liveness fragment gives a trace in which `x` receives enough votes before Force, the complete receipt forms afterward, and strict cutoff prevents its DATA from entering `G` (liveness:347–383). Extracting a request obligation from that late complete result would address that particular trace. It does not address the following variant.

Use three equally weighted members, correct `H0,H1` and Byzantine `B`, with certificate threshold two signatures and `Delta=1`. In each fresh authority slot, a Byzantine initializer first contributes valid empty DATA to `G`, leaving positive accepted count and unused quota. Delay the next recovery authority until an honest recovery initializer `H0` enters near the final AUTO crossing. All previous client dissemination and FIFO conditions hold; `x` is oldest at every correct member.

Let the correct initializer first send and self-vote for the proposal containing `x` at time `s`. This can follow the ordinary NEWVIEW/ANCHOR exchange: `H1` has already supplied its ANCHOR vote. Choose a small `epsilon>0` with `epsilon<1/3`.

1. Delay `H0`'s DATA proposal/forward to `H1` until `s+1`, within the regional bound.
2. Finalize the AUTO crossing at `s+epsilon`. Deliver and verify its complete closure evidence at `H1` before `s+1`. The model supplies upper delay bounds, so a faster source/closure delivery is permitted.
3. `H1` closes its signing gate before processing that DATA proposal. It preserves and forwards evidence but casts no new DATA vote and starts no timer for `x`.
4. `H0`'s existing timer survives Force and can commit at `s+2`, producing one FINAL. `B` withholds its vote and FINAL. `H1` has no timer to expire and no receipt to process. Thus no two-signature FINAL receipt for this request-bearing DATA forms.
5. The earlier accepted empty DATA makes the current duty positive. Force advances to a fresh `r+1`, where the next eventual-but-unbounded authority wait recreates the pattern.

No equivocation is needed. Correct initializer entry, proposal creation, correct-to-correct delays, closure processing, pending-request retention, and strict cutoff all hold. High unused generations can satisfy the schedule assumptions. Repeatedly rotate the actually entered correct initializers as required. A lone old timer does not complete a regional result, and old-generation retirement cannot be treated as permission to collect fresh votes afterward.

Consequently, a rule that scans only complete results for carryover has no input containing `x` to scan in this execution. Calling the lone FINAL, DATA QC, or signed proposal a Publish Layer input would enlarge the frozen service domain. This is an obstruction to that repair, not an impossibility theorem for all algorithmic repairs.

## Why narrower scheduling and accounting changes are insufficient so far

| Change | What it fixes | Remaining obligation |
| --- | --- | --- |
| Increase `M(beta)` faster | Enlarges each finite retry budget. | A fresh eventual authority delay can exceed every chosen finite budget; the existing positive-progress trace still applies. |
| Retain `r` after positive DATA | Avoids immediately creating a fresh authority namespace. | `G` can still change and require a different exact native anchor at a different region. A fixed-slot cache proof is not a fixed-anchor preparation proof. Accepted empty DATA also replenish v3.5 health credits. A new stabilization argument is required. |
| Retain the last recovery index across Force | Avoids resetting to a low-generation initializer. | A new pair has fresh locks and admission proofs. An old Quit is not its new entry proof. Reusing an authority schedule as a scheduling cursor is possible only after specifying how bootstrap differs from recovery; it cannot copy old consensus state or assert that an actually entered honest turn survives cutoff. |
| Clear regional debt only after a nonempty batch | Stops empty DATA from earning that credit. | Byzantine leaders can include their own unrelated requests forever. Nonempty is not fair inclusion. |
| Clear debt only after an honest proposal | Would express the desired progress. | Honesty is not publicly decidable from a valid receipt. All receipts have a correct voter; that voter did not choose the proposal's batch. |
| Require many distinct proposers | May force a correct proposer in a fixed roster if weight exceeds `F`. | Frozen scheduling fairness does not promise a turn to every identity required by an arbitrary coverage target. Nor does a correct proposal imply a complete result before cutoff. |
| Reject a proposal omitting each validator's oldest request | Can stop the all-correct-known empty-block example. | Two correct validators can have different oldest requests, each of full batch size. A correct leader cannot satisfy both; Byzantine votes may be withheld. The original correct-leader progress theorem then fails. |
| Extract canonical debt from late complete results | Gives a public queue once a request-bearing result exists. | The no-complete-result execution above prevents that ingress step. |

Retaining `r` is not ruled out as part of a larger repair. For example, a single fixed region with a prepared anchor and a cached finite authority prefix is a different case from repeatedly changing anchor/recipient pairs. The claim here is only that slot retention alone does not establish the missing fixed-preparation premise.

At the level of the old public accounting rules, a pending `x` does not distinguish an accepted Byzantine empty DATA from the same DATA in an execution with no client requests. Counters based only on accepted height, count, region, and quota cannot certify request service. Any proof using those counters must separately establish surviving honest input selection or add an enforceable input witness.

## Candidate: a request witness for term-first DATA

This section is a concrete future extension, not a change already authorized for the manuscript. It preserves the DATA/DRAIN distinction, the quota, existing commit timers, strict Force cutoff, and the immutable authority namespace. It changes the batch-selection rule and adds a validation witness.

### Request identity and order

Give every request a canonical byte encoding `enc(x)`, including the client's authenticated request identity and payload. Re-delivery preserves this identity. Define

`key(x) = (byte_length(enc(x)), H(enc(x)))`,

ordered lexicographically, with `EMPTY` after every nonempty key. Hash binding supplies the same no-collision premise already used for block identities. Every fixed request has only finitely many possible preceding canonical byte strings: they have length at most its own. Thus the proof does not need a finite union of the local FIFO predecessor sets of every future membership.

Using fixed-length hash order alone also gives a finite key domain, but the length-first order makes the finite-predecessor argument about actual request strings explicit. Neither order supplies a useful worst-case latency bound; the predecessor count can be enormous. This is eventual fairness, not an efficiency recommendation.

### Report production

For the first DATA position of each DATA-producing term, let `P` be its exact parent/anchor. A report binds:

`chain, protocol domain, cfg, assigned region, beta, r, g, t, parent_hash(P), key(x)`.

It is a separately tagged signature; it is not a DATA vote, ANCHOR vote, FINAL, entry proof, or publication result.

A correct member reports once for this exact context. It chooses the minimum-key request in its retained pending pool that is application-admissible after `P`, fits alone in a batch, and is absent from `P`'s ancestry. It reports `EMPTY` only if that set is empty. It does not remove a request merely for reporting it or for placing it on a speculative branch.

- For term zero, send this signed subrecord with the ANCHOR-vote message, after validating NEWVIEW and its exact parent. The original ANCHOR vote retains its matching signed fields. This avoids choosing a request against an unknown recovery anchor.
- For ordinary `t>0`, send the complete report and request bytes to the designated successor when local terminal-QC processing enters that term. The parent is already fixed by that QC.
- DRAIN needs no report: it has no application payload and exists only to commit its DATA ancestry.
- Apply the existing fresh-signing/closure gate. An archived report is evidence only; its context cannot authorize another pair or parent.

The report subrecord is processed independently of its carrier's original consensus action. A missing or malformed report must not hide a valid Quit proof or invalidate an otherwise valid ANCHOR vote; it merely fails to supply an input report for the proposer. Correct senders provide complete report bodies, and the correct-only collection path remains available.

Correct reports include their complete request bytes for the collecting leader. The proposer may use any distinct-member report set of weight greater than `F`, but a correct proposer only uses nonempty reports for which it has the complete bytes and has checked their key, admission, size, and absence from the exact parent ancestry. It can gather enough reports from correct members alone.

### Proposal rule and public validation

The first DATA proposal carries a report set of weight greater than `F` for its exact context. If the set contains a nonempty report, choose the least reported nonempty key. Its full request is the block's **primary request**, executed first in the application batch. Fill remaining capacity using the existing local FIFO rule. If all reports are `EMPTY`, there is no mandatory primary; ordinary admissible batching, including an empty batch, remains possible.

Every validator checks the same supplied witness:

1. Distinct valid member signatures, current snapshot weight, report domain, exact view, first-DATA position, and exact parent.
2. The declared primary is the minimum nonempty key in that set, or all reports are `EMPTY`.
3. The full primary bytes match their declared length and hash, are application-admissible after the exact parent, fit the batch, and do not already occur in its ancestry.
4. The primary is actually executed in the DATA batch. The subsequent application sequence is valid under the existing rules.

Validators **do not** require their own report or their own oldest request to appear in the certificate. They do not recompute the reported choices from their private queues. Consequently two different local queues do not create two different validity predicates for one complete proposal.

The witness may contain only signed keys for nonselected reports. Their bodies are not dependencies of the public minimum comparison. The selected key must have a full valid preimage in the DATA batch. A fake smaller key cannot justify a different, larger primary: the minimum check fails. If the fake key itself is selected, its missing or invalid body fails validation. Correct proposers avoid this obstruction by using fully checked reports from correct members, whose combined weight exceeds `F`.

The semantic block identity binds the selected primary and ordinary DATA content. Report signer-set/encoding changes for the same content should be normalized as witness changes, like alternate parent-QC encodings, rather than creating synthetic proposal equivocation. The exact encoding must be specified before implementation.

This reserves a primary request ahead of FIFO filling. It is not literally the old "fill the batch with the oldest remaining requests" rule. The revised proposal rule must say so explicitly; the retained FIFO rule governs the remaining capacity.

### Regional timing calculation

These bounds are for a correct leader while the generation remains unfrozen and unclosed, under the existing complete-message processing contract. They do not bound external authority or native preparation.

For term zero, let `e` be first correct entry. The original schedule gives all entries by `e+Delta`, NEWVIEW by `e+3Delta`, and its delivery by `e+4Delta`. Correct ANCHOR votes with complete input reports reach the leader by `e+5Delta`. It has a correct-only report quorum by the same deadline as its ANCHOR QC. The first proposal reaches all correct members by `e+6Delta`, exactly the old first progress deadline. No new round is inserted here. The candidate must preserve message-before-timer handling at equality.

For an ordinary term, let `q` be the first correct complete receipt of the preceding terminal QC. Part I's terminal-QC handoff gives all correct term entries by `q+2Delta`, and gives the correct successor its parent QC by `q+Delta`. Reports sent on entry all reach it by `q+3Delta`; the first DATA reaches all by `q+4Delta`. Subsequent positions still add at most `2Delta`, so position `i` reaches all by

`q + (2i+2)Delta <= e_v + (2i+4)Delta`,

because every local entry `e_v >= q`. Thus the existing progress-check deadlines have sufficient slack. This calculation uses the proven `2Delta` ordinary entry skew, not the `Delta` recovery-entry skew. See baseline Part I:631–707.

The original protected initializer window starts at its first proposal and uses the fact that later terms cannot be admitted before that proposal. Reports create no new term admissions or Quit proofs. The above term-zero bound therefore supplies a plausible continuation of that argument, but a revised proof must still explicitly cover all new validity dependencies, malicious report handling, closure races, and normalized witness identity.

### Conditional inclusion lemma

Fix a persistently admissible request `x`. Assume the already required serving/dissemination condition has supplied a tail of accepted term-first DATA whose report certificates contain at least one correct member that knows `x`. This is automatic after all correct members of a fixed continuing serving committee have received `x`; changing membership needs the separate clarification below. Assume there are infinitely many such accepted term-first DATA after this dissemination tail begins.

Suppose `x` never enters `G`. Consider any one of those accepted term-first blocks `B` with parent `P`.

1. Its parent ancestry is part of the eventually accepted global chain. Therefore `x` is absent from it, remains admissible there under the theorem's persistent-admissibility condition, and fits alone.
2. At least one correct, `x`-aware report signer chooses a nonempty request `y` with `key(y) <= key(x)`.
3. The selected primary `z` has `key(z) <= key(y) <= key(x)`, because the minimum is taken over a report set containing that correct signer. Application validity and the preimage check establish that `z` is an actual request, not an unsupported identifier.
4. `z` is executed in `B` and is absent from its parent ancestry. Different accepted term-first blocks therefore cannot repeatedly discharge this argument with the same request identity: any earlier primary is already in the later block's ancestry.

Only finitely many canonical request strings have key at most `key(x)`. Infinitely many distinct accepted primary requests from this set are impossible. Hence `x` eventually enters `G`.

Continuing accepted DATA implies infinitely many accepted term-first DATA when the term and session quotas are finite. For service confined to a particular persistent region, use its repeated accepted DATA guarantee: a finite session contributes only finitely many DATA, so repeated unbounded contributions give infinitely many appropriate first-DATA witnesses. First DATA cannot be skipped by contiguous global acceptance. One must also exclude infinitely many protocol events before a finite time in the ordinary execution semantics when discarding pre-dissemination reports; this note does not claim a Zeno-execution result.

This lemma does **not** assume that an honest leader's proposal is accepted. The accepted proposer may be Byzantine. The report quorum is the observable link to a correct participant's input selection. It also does not convert an isolated report into an externally deliverable request.

## Canonical-debt alternative and its extra costs

Instead of immediately executing the selected primary, a DATA block could carry it as authenticated ingress content. Accepted complete results would append it to a public FIFO debt queue; future DATA would have to execute that queue against a fixed authenticated session-opening snapshot. Late complete closed-version results could also contribute requests if this were explicitly part of deterministic replay, while their old DATA would remain globally rejected.

The report witness is still needed to close the ingress hole. Bare extraction from existing request-bearing results is insufficient. A metadata design additionally needs:

- A rule saying exactly which result contents create debt, how duplicates are keyed, and when invalid/obsolete application requests may be skipped without deleting a persistently admissible request.
- A shared authenticated queue snapshot for proposal validity; checking each receiver's latest private queue is unsafe for progress.
- A proof that opening/reference heights eventually pass every debt-creation position. The v3.5 increasing regular selection floor and strict Force opening order are relevant, but must be integrated with the new replay state.
- Capacity accounting for both ingress bytes and executed transaction bytes. A request fitting alone does not imply that an execution batch plus another full request fits the original publication envelope.
- An explicit revision to the batching discipline; priority for canonical debt is not the unchanged private FIFO rule.

The direct-primary variant avoids this additional public state and duplicate payload capacity. It is the simpler candidate to test first if a future request-progress extension is pursued.

## Obligations before this can become a protocol claim

1. **Global and regional progress remain separate prerequisites.** Fix the complete-result acceptance/publication issue and independently prove continuing accepted global DATA and the required repeated regional service. This note does not reuse the invalid v3.5 constant `L` argument.
2. **Dissemination across future memberships must be grounded in the frozen premise.** Part II says correct-client requests reach regions that continue serving, while its queue rule is per node. The proof needs sufficiently many future accepted certificates to contain a correct `x`-aware reporter. Per-node eventual delivery with arbitrarily fresh disjoint rosters and no inherited pending pool does not by itself imply a common post-dissemination tail. The length/hash order removes the infinite-union-of-FIFO-predecessors problem, but not the problem of repeatedly selecting report signers who have not received `x` yet. Do not add this awareness condition silently. A concrete pending-pool retention/transfer rule can be investigated under the existing regional communication interface; it must be specified and proved, not treated as an external request service.
3. **The service size envelope is not yet checked.** The primary request reuses its actual transaction bytes and still fits alone. Reports add at most one signature/key per regional member, plus context. This is bounded for each installed roster and does not add a second full transaction to the published DATA. Nevertheless the frozen `T_pub` and `T_E` apply only under their declared input-size/load limits. The sources supply no numerical headroom that proves the enlarged complete result lies inside those limits. Regional report collection also carries full request bytes to the proposer. The exact encoding and operating envelope must be checked; unchanged asymptotic order is not sufficient evidence.
4. **Regional validity needs a fresh integrated audit.** Carry the report witness in every complete validation bundle; gate fresh reports on retirement; bind exact parent and view; normalize alternate witness encodings; verify the correct-only report collection path; keep reports out of Quit/FINAL semantics. The timing calculations above are not a full replacement for the Part I safety and liveness proofs.
5. **Application semantics need an exact request identity and admissibility interface.** Repeated delivery must not create fresh identities. Primary duplication must be checked against ancestry even if an application permits repeated identical commands. Persistently admissible requests must be distinguished from requests invalidated by previously accepted transactions. These checks cannot be inferred merely from a hash order.
6. **Do not label this a minimal wrapper repair.** It changes first-DATA validity and batching and adds a regional report action. The frozen environmental assumptions can remain unchanged only after the preceding interface obligations are established. No bounded authority wait, bounded native preparation, accurate health oracle, late-cutoff exception, or external bare-request publication guarantee has been introduced in the candidate argument.

## Validation performed

This is a source-level design and adversarial execution analysis. No manuscript was edited; no compilation, executable model checking, or full proof certification was performed. The stronger no-receipt trace and the report-collection bounds were checked against the frozen quorum, retirement, term-entry, ANCHOR, progress-deadline, and publication definitions. The candidate remains a future extension with the explicit unresolved obligations above.
