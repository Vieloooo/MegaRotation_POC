# Request reports: scope, timing, and membership handoff

Status: a future design candidate, not an implemented or verified amendment. The user has not yet selected whether to modify FIFO, preserve FIFO, or limit the objective. This note does not assume that decision and changes no manuscript source.

## What is established and what remains open

The completed v3.7 reviews support the stated abstract safety, accepted-DATA progress, and repeated regional DATA service results. Those results do not imply request inclusion: the acknowledged empty-DATA/late-honest-proposal execution remains possible. This note develops a separate candidate in which the first DATA of each DATA-producing term contains an enforceable request-selection witness.

The candidate has a short conditional inclusion proof and can fit the original regional progress deadlines **if** request acquisition fits the specified query/reply interface. The principal remaining gap is making future voting members sufficiently aware of a retained request under the frozen, changing-membership model. Existing synchrony and pending-request retention do not by themselves specify a bounded complete-pool query or a complete membership handoff. The message and complete-result size profile also remains unchecked.

The candidate is therefore reviewable, but is not yet a full request-liveness solution under the literal frozen assumptions. No authority/native preparation bound, external bare-request publication service, or exception to strict Force cutoff is introduced here.

All source locations below are relative to `snapshot/`. `main` is `sync_hotstuff_rr_part2_rotation_v3_7.tex`; `live` is `sections/rotation_v3_7_liveness.tex`; `retirement` is `sections/rotation_v3_4_retirement.tex`; `Part I` is `baseline/sync_hotstuff_rr_part1_core.tex`.

## 1. The pending FIFO decision is substantive

| User choice | Consequence for this design |
| --- | --- |
| Permit an explicit FIFO modification | Investigate the direct-primary witness below. Its mandatory primary precedes ordinary FIFO filling. |
| Preserve the current FIFO batching behavior | Do not adopt direct-primary. A separate mechanism must ensure that enough existing FIFO-selected honest proposals are accepted, or supply a different compatible inclusion rule. This note proves neither. |
| Limit the objective to accepted DATA / repeated regional DATA | v3.7's reviewed result can be described at that scope, while request inclusion remains explicitly unproved. No report extension is required for that narrower claim. |

Part I's `ass:fairness` mixes environment conditions with specified correct behavior: schedule fairness, eventual delivery to all correct replicas, common admission order, oldest-pending batching, and pending-until-commit (Part I:208–223). Part II already adapts this interface: its actual request section specifies local first-receipt FIFO, allows different queue orders, and removes a request only at `G` (main:2221–2239). Its Model progress paragraph instead says requests reach regions that continue serving and have finitely many older local predecessors (main:535–537).

The new rule would replace part of the **correct-leader batching behavior** in Part II. It does not merely add a proof of the old rule, and retaining the Model section byte-for-byte would not hide that semantic change. Conversely, reserving a primary request is an algorithm change, not a stronger network or publication service assumption. An amendment would need to identify explicitly which Part I behavioral clause Part II supersedes. If the user freezes that inherited behavior as well as the environment assumptions, this candidate is outside the permitted scope.

The report witness does not require a common local FIFO order. Each node retains its existing first-receipt order for ordinary filling; report choice uses the separate finite-predecessor order below. Imported requests cannot move ahead of already queued requests in the ordinary FIFO by inventing an older local receipt time.

## 2. Minimal report and first-DATA interface

### 2.1 Identity and finite-predecessor order

Let `enc(x)` be the canonical authenticated bytes of one logical client request, including the client's request identity and payload. Re-delivery does not create a new identity. Let

`key(x) = (byte_length(enc(x)), H(enc(x)))`.

Compare keys lexicographically; put `EMPTY` after every nonempty key. Hash binding is the existing cryptographic premise, not a new source of agreement. For every fixed `x`, every actual request preceding `x` has byte length at most `length(enc(x))`. There are finitely many such byte strings. This finite-predecessor fact does not rely on claiming that an infinite mathematical execution contains only finitely many hash outputs. Hash binding is needed to identify the selected body, not to make the request-string predecessor set finite.

The resulting eventual bound can be astronomically large. This order is a proof device for eventual fairness; it is not a practical latency or priority guarantee.

### 2.2 Message context and request ownership

The first DATA of each term has a fixed parent `P`. A report signs

`INPUT_REPORT(chain, domain, cfg, region, beta, r, g, t, parent_hash(P), key_or_EMPTY)`.

It is distinct from ANCHOR/DATA/DRAIN votes, STATUS, FINAL, and Quit evidence. It creates no timer, entry permission, or public acceptance. A correct member signs once for this exact context and chooses the minimum-key locally available pending request that:

- passes application admission after the exact parent;
- fits alone in a DATA batch; and
- does not already occur in that parent's ancestry.

`EMPTY` means this eligible set is empty. It must not mean that a required acquisition task is incomplete. However, unless the acquisition contract below is actually established, waiting for an indefinitely incomplete pool would itself break the correct-leader progress argument.

The report delivered to its proposer includes the full selected request body. A correct proposer collects a distinct-member report set of total weight `>F` using only reports whose context and signature validate and, for nonempty reports, whose full bodies it locally checked. Correct members alone provide enough weight. A correct proposer never waits for Byzantine replies or for an operationally unrecognizable event called "all correct replies received."

### 2.3 First-DATA rule

For first DATA position `j=0` in a DATA-producing term:

1. Obtain a report set of weight `>F` for the exact view and parent.
2. If any report is nonempty, choose its minimum key and execute that request as the first transaction of the DATA batch. This is the reserved **primary request**.
3. Fill the remaining application capacity using the existing local FIFO rule, skipping requests already in the now-extended ancestry/batch. If all reports are `EMPTY`, use ordinary valid batching, possibly empty.
4. Attach the report signatures and keys as validation evidence. Publish only the selected full request bytes, already present in the actual application batch; nonselected full request bodies need not be included in the result.

Every validator checks that same supplied witness: member identities/weights, exact context and parent, minimum selection, selected-body hash/length, admission, capacity, absence from parent ancestry, and actual execution in the application batch. A validator does **not** require its own report or its own oldest FIFO request to occur in the witness. A local-queue veto would reintroduce incompatible validity conditions and can prevent a correct leader from obtaining votes.

A fake nonselected key cannot justify skipping a smaller selected key: minimum checking is mechanical. A fake key selected as minimum must have a full matching, admissible body or the block fails. A correct leader avoids fake-minimum obstruction by using the complete reports of correct members, whose total weight is `>F`. No public validity claim is made about the unpublished bodies of nonselected Byzantine reports.

Reject a primary request already in the parent ancestry even if the native application would otherwise permit repetition of the same command. This is necessary for the finite-predecessor argument. The exact logical-request identity must distinguish retransmission from a legitimately separate client command.

Alternate signer subsets or encodings for the same semantically valid primary should remain validation-witness variants, like alternate parent-QC encodings; they should not manufacture proposal equivocation by themselves. The primary's identity and bytes are hash-bound in DATA. The final signature/serialization definition requires an integrated audit.

DRAIN is unchanged and carries no primary request. A recovery at the DATA endpoint uses DRAIN, so it cannot be counted as a fresh request-bearing proposal. Its inherited DATA ancestry must already contain the witnesses required when those DATA blocks were proposed.

## 3. Timing with a two-Delta request query

The following is a **conditional timing lemma**, not a derivation of the acquisition contract. Let `e_v` be local term entry, `e=min e_v`, and use the installed regional `Delta`. Assume a correct member can start an appropriate pending-data query at entry and completely process every required correct response by `e_v+2Delta`. All payload and processing/load conditions of the existing regional bound must hold.

### 3.1 Ordinary term

The original terminal-QC dissemination argument gives `e_v <= e+2Delta` without depending on the successor's new proposal (Part I:630–657). A member sends its report when its query stage ends at `e_v+2Delta`. Thus:

| Event | Latest time from first correct entry `e` |
| --- | --- |
| Last correct ordinary-term entry | `e+2Delta` |
| Last required correct query stage completes; last correct report sent | `e+4Delta` |
| Correct leader receives a correct-only report quorum | `e+5Delta` |
| First DATA is completely validated and voted for by every correct replica | `e+6Delta` |

The parent terminal QC reaches the correct successor no later than `e+Delta`; reports are the later prerequisite. Each additional DATA position still adds at most `2Delta`, so position `i` is voted for by

`e + (2i+4)Delta <= e_v + (2i+4)Delta`.

This meets the original progress-check deadline exactly in the worst case. These are **votes at the replicas**, not receipt of those votes by the proposer. At equality, all due complete-message processing must occur before the progress callback. That rule already exists (Part I:355–357, 681–709; main's protocol message-processing order). There is no spare allowance for an uncharged queue scan, response-chunk round, source lookup, native import, or extra delayed dispatcher.

A query responder cannot simply buffer until its own ordinary-term entry. An early requester may enter at `e`, while another correct member enters only at `e+2Delta`; a reply sent then can arrive at `e+3Delta`, after the requester's report cutoff. Either the query carries complete terminal-QC/admission history so normal message processing can admit the receiver before servicing it, or the persistent data-service handler must be explicitly usable before local term entry. The latter transports client data without signing a new BFT report for an unentered view.

### 3.2 Initial and recovery term zero

For `g=0`, the exact anchor is already installed at Ready/Enter (main:2018–2031). Querying can therefore use it immediately. Initial-entry skew `<=Delta` remains conditional on the complete ReadyBundle/adoption/Enter contract and an unclosed handoff (main:2039–2050).

For `g>0`, the chosen recovery anchor is unknown until NEWVIEW. Start an **anchor-independent** data acquisition at entry, then filter the acquired pool against the exact validated NEWVIEW parent. With cached recovery authority, entry skew is `<=Delta` (live:113–139), so query stages finish by `e+3Delta`; the correct initializer's valid NEWVIEW reaches everyone by `e+4Delta`. Let

`report_time_v = max(e_v + 2Delta, validated_NEWVIEW_time_v)`.

Reports are then sent by `e+4Delta`, reach the leader by `e+5Delta`, and first DATA reaches all by `e+6Delta`. The leader also has its ANCHOR QC by `e+5Delta`. For `g=0`, the same conservative bound holds.

Do not replace "cached recovery authority" with a new simultaneous authority-delivery assumption. The v3.7 authority clause states common authenticated finalized records, not a shared local delivery instant. Its prepared-tail timing supplies the cited skew; arbitrary initial authority waits remain eventual and unbounded.

The original ANCHOR-vote action is immediate upon valid NEWVIEW. Preserve it and send REPORT separately when `report_time_v` arrives. Literal piggybacking would otherwise require delaying an early ANCHOR vote, which is another core behavior change. A malformed report subrecord never invalidates an otherwise valid original ANCHOR vote or suppresses embedded Quit evidence.

Starting an anchor-dependent query only after NEWVIEW does not fit: NEWVIEW can arrive at `e+4Delta`; query/reply, report transmission, and DATA delivery can then end at `e+8Delta`. Also, an early raw-pool query returning only its head is not sufficient: from `[a,b]` it might return `a`, and the later selected parent might already contain `a` while `b` remains unknown. A bounded sufficient response must be established separately.

## 4. What current retention and synchrony actually provide

The frozen network condition bounds a correct-to-correct complete message **within region `i`** and includes processing of complete validation dependencies (main:392–404). It does not give a cross-region bound. Healthy regional managers watch public histories without an Active engine (main:1523–1526). Pending requests survive discarded suffixes and are not deleted before `G` (main:2232–2239). Program exit requires durable data/control/publication ownership transfer and preserves late-message routes (main:2139–2149; retirement:130–173).

These are useful ingredients, but they are not yet any of the following:

- A directory that lets a new roster discover every relevant correct pending-request holder before its report cutoff.
- A bound on transferring an entire pending pool, whose size can grow while only individual requests are known to fit a batch.
- A promise that old physical members remain online indefinitely after all specified duties have been handed off.
- A rule transferring every pending request to all correct members of every future `AUTH` roster.
- A bound on an old owner reconstructing the new parent's native state to filter its pool for admissibility.

The existing retirement contract explicitly transfers complete-result publication, closure and receipt duties. Its general request-retention text does not specify the stronger current-roster delivery and service interface above. A correct local persistent owner may be a natural implementation, but naming one does not prove its remote availability, discovery, load bound, or transfer completeness.

## 5. Changing membership and queue handoff

### 5.1 The awareness property actually needed

For a fixed persistently admissible `x`, a sufficient condition is that infinitely many **accepted** term-first DATA have a report certificate containing a correct signer that, when reporting against the accepted parent, has an eligible candidate of key at most `key(x)`. Full awareness of `x` at every correct signer is an easy stronger sufficient condition, but must be proved rather than imported.

After `x` reaches every correct member of one fixed serving roster, retention supplies that condition for future reports from that roster. The finite-predecessor order handles different FIFO orders and arbitrarily many future local predecessor sets. It does not help if each accepted certificate is drawn from new correct members who still do not know `x`.

Part I's "eventually reach all correct replicas" concerns its fixed committee. Part II's "reach the regions that continue serving" is weaker/less explicit about rotating rosters. Per-node eventual delivery alone permits each roster to produce its accepted empty/report-EMPTY result before `x` arrives there, followed by delivery at a finite later time after that roster stops contributing. This is a semantic gap to resolve, not an assumption that this note fills with all-future-member delivery.

### 5.2 Why a greater-than-F handoff receipt is insufficient

The regional threshold ensures one correct signer; it does not ensure that two quorums intersect in a correct member. For three equal members `H0,H1,B` with threshold two, an ownership acknowledgement `{H0,B}` and a later report certificate `{H1,B}` intersect only at Byzantine `B`. Knowing that one correct new owner stores `x` therefore does not prevent a certificate whose correct signer remains unaware of `x`.

After one correct holder reliably relays `x` to **all correct members of a known active roster**, a subsequent report certificate cannot avoid all aware correct members. Under the existing message bound that actual complete relay takes `Delta` within the applicable region. The missing work is ensuring the sender, full bytes, current recipient set, timely dispatch and report ordering exist under roster changes. A quorum acknowledgement is not a substitute for that work.

### 5.3 Candidate handoff contracts and their limits

| Proposed addition | Potential use | What still needs proof |
| --- | --- | --- |
| Current-roster QUERY to all current members at entry | Disseminates a request already held by at least one correct current member before the query. | Cannot discover a request held only by a retired/disjoint roster. A bounded sufficient response is also absent. |
| First-receipt and roster-change push by a durable regional pending owner | Can preserve `x` while voter identities change. | Owner lifetime/correctness, new-roster discovery, mandatory dispatch, bounded per-request load, and awareness before report emission; these do not follow from source-history hydration. |
| Mandatory queue handoff before initial Ready/Enter | Can make request ownership independent of a short production attempt. | Exact sender set and finite queue cut, completeness criterion despite Byzantine withholding, transfer availability, and compatibility with existing Ready skew and fixed-slot preparation. |
| Wait for a `>F` queue-handoff certificate | Gives at least one correct acknowledgement. | Does not establish report-quorum awareness, as the three-member example shows. |
| Copy all correct old queues before entry | Would transfer every old correct holder's `x`. | Correct identities are unknown; waiting for all old members can wait forever on Byzantine members. A complete finite-cut protocol and correct-only completion path are required. |
| Query all historical owner rosters | May cover a disjoint old holder. | History/owner discovery and total response work may grow; old owners may have transferred duties. It cannot be charged to one fixed `2Delta` exchange without an explicit bounded interface. |

A new handoff gate cannot be inserted after Ready while retaining the old immediate Enter proof. If handoff is a preparation stage, its completed pool/state must be part of the actual ready object and be retained across retries. In the fixed-`G`, fixed-slot tail used by v3.7, a **fixed finite** transfer task with an established eventual completion path might be incorporated into finite preparation. A transfer target that continually expands with new pending arrivals need not finish; repeatedly restarting it on beta changes reproduces the original lost-preparation problem. Finite preparation alone also does not prove request awareness across the infinitely many different slots possible while `G` grows.

An eventually completing finite-cut handoff would therefore need at least:

1. A durable named owner set and a precise snapshot cut; later arrivals remain in a separate continuing relay duty.
2. Preservation of every unaccepted request across ownership transfer, including requests arriving during the transfer.
3. A correctness/completeness argument that can finish without Byzantine cooperation and does not rely on incorrect quorum-intersection arithmetic.
4. A recipient dissemination rule strong enough that report certificates cannot avoid all aware correct members.
5. Evidence that the owner/recipient communication is within the original regional interface and input/load envelope, or an explicit acknowledgement that the frozen environment does not cover it.

These are unproved candidate interface obligations. The present source does not supply a complete mandatory-handoff algorithm satisfying them.

## 6. Pseudocode with the unresolved interface exposed

The following makes the dependency visible rather than hiding it in an implicit mempool call. `AcquirePending` is **not a service guaranteed by the current model**. Its coverage and size contract must be implemented/proved, or the liveness theorem must retain the explicit awareness condition.

```text
on term Enter(C), at local time e_v:
    persist e_v and C using the existing core entry rule
    begin AcquirePending(C)             // unresolved coverage/size contract
    schedule ReportGate(C) at e_v + 2*Delta

on complete data QUERY(C, admission_context):
    inspect embedded evidence first
    validate context or use a pre-entry data-service route
    reply with the sufficient request material required by AcquirePending
    // never enter/sign a BFT view merely to answer a data query

ReportGate(C):
    process all messages due at this clock time first
    if closed, frozen, completed, or no longer permitted to report: stop
    if term zero and selected parent is unknown: wait for valid NEWVIEW
    require AcquirePending(C) has met its proved contract
    P := exact first-DATA parent from the valid opening/terminal QC
    if this is a DRAIN generation: stop
    x := minimum key among available pending requests admissible after P,
         fitting alone, and absent from P's ancestry
    persist and send one signed INPUT_REPORT(C, hash(P), key(x) or EMPTY)
    send full x bytes to the leader when x exists

correct leader, first DATA:
    receive reports without waiting for Byzantine respondents
    keep distinct valid context-bound reports with complete checked bodies
    once their weight > F and the existing parent/ANCHOR QC is available:
        choose minimum nonempty report key, if any
        execute its request first; fill remaining capacity by local FIFO
        attach signed report keys; supply only selected full request bytes
        run the unchanged DATA broadcast/vote/timer path

validate first DATA:
    validate context, parent, distinct report signatures, and weight > F
    verify declared primary equals minimum nonempty signed key
    verify primary's full bytes, admission, size, and no ancestry occurrence
    verify primary is executed first and the full application batch is valid
    do not compare with the receiver's private FIFO or require its own report
```

For recovery, `ReportGate` fires at `max(e_v+2Delta, valid NEWVIEW processing)` as above. Send REPORT separately from the immediate original ANCHOR vote. Malformed requests/report wrappers cannot suppress valid embedded Quit evidence or invalidate an independent original ANCHOR vote. Incomplete Byzantine replies do not acquire a delivery/completion guarantee merely because they arrived before the cutoff.

## 7. Conditional request-inclusion proof

Assume the v3.7 accepted-DATA/repeated-region progress result remains valid after integrating the added regional validity rule and its size/processing costs. Fix a persistently admissible request `x` that fits alone. Suppose infinitely many accepted term-first DATA have report sets containing a correct signer whose eligible available set contains `x` (or at least contains a request no greater than `x` under the report order).

If `x` never enters `G`, it is absent from every accepted parent ancestry. Persistent admissibility applies at those accepted parent states, not at an unrelated speculative fork. In each of the identified accepted term-first blocks, the correct report chooses `y <= x`, and the certificate's minimum primary satisfies `z <= y <= x`. The primary preimage and application checks make `z` an actual executed request. The ancestry exclusion means the same logical primary cannot supply this progress infinitely often: after one accepted occurrence it is in every later accepted parent ancestry.

There are finitely many actual canonical byte strings of length at most `length(enc(x))`, hence finitely many possible `z <= x`. Infinitely many distinct accepted primaries from that finite set are impossible. Therefore `x` must enter `G`.

Finite term and session quotas imply that unbounded accepted DATA contains infinitely many accepted term-first DATA. For a request confined to a particular continuing region, use repeated accepted contributions from that region; a finite number of its finite sessions/terms cannot contribute infinitely many DATA. Contiguous acceptance cannot skip the corresponding first DATA. Discarding pre-dissemination reports uses the ordinary locally finite execution semantics; this is not a Zeno-execution theorem.

The accepted proposer can be Byzantine. The witness supplies a correct input-selection contribution without requiring an accepted honest-leader proposal. Force still rejects all later old-version DATA, including late request-bearing receipts. No isolated report, single FINAL, QC, or bare request receives `T_pub`.

This is a modular conditional theorem. The awareness premise is precisely the unresolved membership/ingress obligation; it is not asserted to follow from the current prose. Likewise, the reviewed v3.7 progress proof cannot simply be cited unchanged after adding an unbounded `AcquirePending` wait.

## 8. Exact size and service compatibility

The selected primary reuses its normal application bytes; it is not duplicated as an additional metadata payload. Nonselected report bodies are sent regionally to the leader for collection but need not be published. The published witness adds report signatures, keys, member identifiers and context, bounded by the installed roster size.

Let `B_old` denote the actual previous complete-result size limit, `B_data` its application-byte allowance, `D_old` its other dependency bytes, and `R(n)` the normalized report-witness overhead for an `n`-member roster. A concrete encoding must demonstrate, for every admitted result,

`encoded_result_bytes <= B_old`,

including `R(n)`, all required reports over an exported ancestry, FINAL/QC evidence, and carrier/codec overhead. A statement that `R(n)=O(n)` does not establish that inequality. A request fitting alone in `B_data` proves no spare witness capacity. If witness bytes share the old application cap, reducing transaction capacity can also violate the inherited "every request fits" premise for a maximum-size request.

Regional collection additionally sends potentially one full candidate request per reporting member, plus any query/reply material. The network assumption's complete-processing clause is not a proof that arbitrary new aggregate load, full-pool transfer, queue scanning, or native reconstruction finishes within one `Delta`. The intended interface must stay within the declared operating profile. The sources provide no numerical headroom or concrete codec with which to verify this now.

The Publish Layer remains restricted to complete regional results (main:406–421). Reports become covered only as the defined DATA/validation evidence of such a result and only within that old size/load profile. No raw-report or client-request publication deadline may be inferred from `T_pub` or `T_E`.

## 9. Scope verdict and validation

The direct-primary mechanism is a credible future algorithm candidate: its minimum witness is common and checkable, its predecessor argument tolerates changing local FIFO orders, and a genuine two-message acquisition stage fits the existing progress deadlines under the stated prepared timing contracts. It explicitly changes FIFO batching behavior and first-DATA validity. It does not preserve the regional core unchanged.

The unresolved items are concrete: user authorization for the FIFO change; sufficient request awareness across changing rosters; a complete bounded acquisition/handoff interface; old-profile size/load compatibility; and a fresh integrated safety/liveness/retirement/kernel review. Pending the user's choice and those proofs, the full request objective remains open.

Validation here consisted of source-level analysis against the frozen v3.7 snapshot and completed model review, plus an independent read-only check of the query/report timing and callback order. No manuscript edits, compilation, model checking, deployment benchmark, or whole-protocol certification were performed.
