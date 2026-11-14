# v3.8 starvation with recurrent correct identities

Date: 19 October 2026. This is a new independent source-level trace audit. It does not amend `v39_strict_domain_obstruction.md` or any manuscript source. Communication is interpreted strictly within an authenticated snapshot; no old-only to new-only physical-region delivery guarantee is imported.

## Summary and verdict

**Final Verdict:** the proposed construction can be made into a source-admissible v3.8 starvation execution under the written eventual native-preparation premise. Every born correct identity actually re-enters DATA-producing turns infinitely often and is their correct initializer infinitely often. Both physical regions contribute accepted DATA infinitely often. All qualifying original complete regional results are published and finalized within their service bounds. The request never enters G and never occurs in a complete regional result.

The mechanism is a correlation between **finite but unbounded first preparation of a new exact anchor** and the AUTO deadline. Informed recurrent cohorts become Ready just early enough to propose and cast one DATA vote, but too late for any peer vote before authenticated closure. Each yields one surviving FINAL signature, below the receipt threshold. Fresh cohorts receive the request after selecting their sole DATA and then complete normally. The old request is retained and repeatedly proposed; the construction does not rely on losing it or refusing the holders future turns.

This is a counterexample to deriving full request inclusion for **v3.8** from aggregate regional successful service plus per-identity repeated entered turns. It is not an impossibility theorem for every protocol redesign. It also does not satisfy a stronger, additional requirement that every informed identity or every individual selected snapshot eventually receives enough post-Ready time to form a complete request-bearing result. The frozen prose supplies eventual preparation and service to working regions, but does not explicitly impose that latter per-identity coupling.

## Fixed parameters and participants

There are two physical region identifiers, A and B, in that cyclic order. Both remain governance-ON and have positive designated-authority weight. The committee issues no controls. Each region has countably many disjoint cohort triples, numbered by birth order. A cohort's three members are all correct, have unit weight, and use

`W=3`, `F=1`, `Δ=1`, `TB_r=1`, `K=1`,

so a certificate or FINAL receipt needs at least two distinct signatures. The fault upper bound is respected even though actual Byzantine weight is zero. Every recovery schedule, including those never entered, contains only these correct members. No equivocation or exclusion event occurs.

Set `W_start=W_run=20`, `m_start=q_min=1`, and `C_auto=1` at every slot. Authorities are immutable and available before they are needed. At a slot, AUTH contains one cohort for A and one for B; an AUTO retry at that slot reads the same record. Different slots may assign different cohorts. A reused cohort has the same retained node-level request state, but a fresh session and engine.

Take fixed positive service bounds large enough for the timely submissions below, for example `T_pub=2`, `T_E=1`. Choose actual Espresso finalization/visibility faster than those bounds when needed. Use strictly growing timestamps equal to real finalization time, so the clock-lead condition holds with `ε_E=0`. Empty Espresso blocks make all deadlines and selection references available; no namespace or control evidence is withheld. Every participant follows and retains the public history.

The sole client request `x` fits alone, is authenticated, and remains admissible after every global parent used here. There are no older requests ahead of it at any cohort. Accepted DATA batches can be empty. Distinct accepted blocks have distinct exact anchor identities and continuation/provenance targets even when their application batch is empty.

## Local timeline for one informed appearance

Let an informed cohort be assigned a public export duty in version β, at slot r, with opening timestamp `o`. Its fixed tolerance is `M=1+β`. No DATA from that duty has entered G. Its Mth startup miss will be settled at

`D = o + 20M`.

Keep every public dependency and authority available, but let the first member finish constructing a directly executable continuation for this new exact anchor at

`e = D - 4.5`.

That member is the assigned initializer L. It immediately constructs/sends its complete ReadyBundle and enters; the other members receive/adopt/enter by `e+1`. This is an actual entry, not a schedule name or a future callback. All members already retain `x` from their birth session and relay it within their snapshot.

Choose actual regional delays as follows. Every indicated peer-message delay is exactly one; self-processing is immediate. Earlier public-state and closure messages may be processed faster, still within the same bound.

| Time | Event and consequence |
|---|---|
| `e` | L completes first native preparation, enters, sends ReadyBundle. |
| `e+1` | The other two members finish its complete receive handler and enter. Their unfinished independent reconstruction is no longer needed. |
| `e+2` | L's mandated two-unit wait ends. It sends NEWVIEW, processes it locally, and produces its own ANCHOR vote and `x` report. |
| `e+3` | Both peers validate NEWVIEW and independently send ANCHOR votes and `x` reports. |
| `e+4` | L receives enough ANCHOR votes and reports. It constructs the sole DATA with primary `x`, broadcasts it, and immediately processes its own Vote. Its commit timer expires at `e+6`. |
| `D=e+4.5` | An authenticated Espresso block crosses the Mth missed startup deadline. AUTO closes β before its payload. All three members process authentic closure before `e+5`; complete close bundles are also relayed within Δ. |
| `e+5` | Peer DATA/forwarded-proposal deliveries arrive on time. The durable Closed/fresh-action gate suppresses both peer DATA votes and both peer commit timers. Each archives the material and services the required closure response. |
| `e+6` | L's existing timer survives: Force is not Quit or Completion. L commits its DATA and sends its one FINAL signature to the fixed membership and collection interface. |
| `e+7` onward | All peers/collectors receive the one FINAL and all accompanying bytes. They cannot form a weight-greater-than-one receipt. Their old branch is closed, so they cannot create the missing votes, timers, or fresh recovery actions. |

There is no DATA QC for this proposal and no covering FINAL receipt. ANCHOR votes do not count as DATA votes or FINAL signatures. Repeating this process in other pairs cannot combine signatures: FINAL binds the branch/session, configuration, height, and hash. No public-service antecedent for a complete `x`-bearing result ever arises.

The first progress check would be at local entry plus six. Authentic closure already gates that fresh BLAME action. No genuine Quit proof is invented to cancel L's timer, and no terminal Completion is invented from its lone signature. Retirement drains the actual timer and preserves all data and pending request duties.

## Local timeline for a newborn cohort

A newborn cohort's three identities have not yet received `x` and have never shared a snapshot with an informed cohort. Its exact anchor is prepared promptly. Use the same ordinary initial bootstrap timeline through first DATA at `b+4`, where b is first correct Enter. At each report event the local eligible set is empty; the reports and sole DATA therefore have no request.

Deliver the client request to L at `b+4.25`, after the proposal and its self-vote. Relay it to the peers within one unit. Thus every member receives and retains `x` while the session is still active and before Completion. The peers' later Vote on the existing empty proposal is valid: the report predicate is the immutable exact-context predicate, not a comparison with their updated private queues.

There is no closure during this service. Peer DATA votes occur by `b+5`; L commits at `b+6`, peers at `b+7`, and a complete FINAL receipt exists by `b+8`. Publish the original complete result immediately and include/accept it by `b+9`. This is within the original bounds and well before even the smallest startup deadline at b plus approximately twenty. Every complete dependency and any publication retry needed for the accepted extension is supplied. Since the quota is one, no later request-bearing DATA position exists in this session.

The newborn cohort is now informed forever. It will later reappear infinitely many times under the preceding informed-appearance timeline.

## Global slot, owed-round, and cooldown schedule

A finite seeding prefix gives one newborn A cohort and one newborn B cohort an ordinary successful session each, after which both know `x`. At the next public duty boundary the owed set is `{A,B}`. The seed uses ordinary accepted empty DATA and the newborn timing above.

The following macrocycle alternates its names A and B on every repetition. Begin with owed set `{A,B}`, current version β, and an ordinary duty for an informed A cohort at slot r. AUTH[r,0] names this informed A cohort and a newborn B cohort.

| Step | Authorized session and public effect | Owed set after accepted DATA / boundary |
|---|---|---|
| 1 | Informed A starts `(β,r)` late and follows the informed timeline. It has zero accepted DATA. | `{A,B}` |
| 2 | AUTO at D retains r, increments β, and selects newborn B in the **same** AUTH[r,0]. A is cooled until `D+1`. | `{A,B}` |
| 3 | Newborn B promptly completes `(β+1,r)` and its one DATA enters G. | `{A}` |
| 4 | The ordinary successor uses AUTH[r+1,0], whose A roster is another newborn cohort. Historical NextLive selects A. That A cohort promptly completes and its DATA enters G. | Empty, then refilled to `{A,B}` at the next public boundary |
| 5 | The ordinary successor at r+2 is an informed B cohort. AUTH[r+2,0] names it and a newborn A cohort. | `{A,B}`; next macrocycle starts with A/B exchanged |

Step 2 is exactly the zero-output retry rule: no r increment, no AUTH rebinding, and fresh core state at g=t=0 in the new β. There is an eligible other member of the owed set, so conditional AUTO cooldown may cool A and NextOwed must select B.

For step 4, choose newborn B's terminal DATA selection reference at a finalized Espresso header later than `D+1`, and at least one height beyond its opening. Such a header is available before its normal proposal at approximately `D+4`. The first eligible historical NextLive position therefore sees A's cooldown expired and selects A. The public terminal DATA commit precedes PrepareOwed and cannot be delayed by it. Authorities are already available. No stale reference to the cooldown interval is used.

After newborn A's accepted block, the ordinary successor B is selected using unrestricted NextLive. The empty owed set refills when this new public duty is resolved. Both regions have just received accepted credit, so the next informed interruption begins with the invariant `{A,B}` again. The accepted region sequence is `B,A,A,B,B,A,...` after the seed; both regions are served infinitely often. Temporary cooldowns never suppress a region indefinitely.

There is no uncontrolled speculative pipeline. Arrange optional direct handoff delivery so that each new first Enter is no earlier than its public duty opening. This is allowed because direct cross-region transfer has no guarantee and the Espresso path suffices. An informed successor may know its authenticated handoff in advance, but its first full native preparation is the finite late event specified above. It cannot advance the speculative production branch before Ready.

## Scheduling every identity infinitely often

For each physical region separately, use its successive informed macrocycle appearances to dovetail over all cohorts already born there. One explicit stage schedule is: at stage k, schedule every cohort among the first k cohorts once for each of its three initializer choices. New cohorts born during that finite stage join later stages. A finite seed supplies the initially available cohort.

Each stage is finite; each constituent macrocycle has finite duration. There are infinitely many stages and every cohort has a finite birth index, so every cohort appears in all sufficiently late stages. Cycling the three initializer choices makes every identity the actual initializer L infinitely often. Every such occurrence executes Enter, NEWVIEW, a DATA proposal containing `x`, and its own DATA Vote. These are actual DATA-producing turns, not unused authorities or entries to a permanently blocked Ready queue.

All other identities in that selected cohort also enter the actual session within Δ and participate in the ANCHOR/report steps. The frozen initializer/schedule fairness formulas hold because all scheduled leaders are correct. No recovery-generation position is falsely counted as an entry; this all-correct execution needs no genuine recovery. Every born correct identity nevertheless has infinitely many actual initial and DATA-leader entries across sessions.

## Why finite late preparation is allowed by the written code

This is the key admissibility check. The construction does **not** delay a complete received ReadyBundle, a validated MaterialAvailable callback, or a complete public source handler.

- `rotation_v3_5_ready_bundle.tex:60–70` places gathering, cross-region fetching, native reconstruction, and first bundle construction before first Enter and explicitly assigns them no fixed time bound. Once a bundle exists, it must be sent in that same event. The trace does so at e.
- The same fragment, lines 90–122, requires atomic adoption and Enter within the complete receive handler. The trace enters every peer by `e+1`, and inserts no post-Ready import.
- Main v3.8 lines 1674–1684 requires immediate entry when Ready passes and proper asynchronous dispatch. Before e the exact executable object is unavailable; at e the trace enters immediately. The task may be active throughout the wait; lack of a uniform completion bound permits each distinct reconstruction to finish finitely but late.
- `rotation_v3_7_source_step.tex:171–176` attaches an executable continuation **when already prepared**, and explicitly performs no new reconstruction there. Full public history and a correct public interpreter therefore do not by themselves imply Ready.
- Main lines 1763–1772 retain completed and unfinished preparation for the **same exact anchor**, keyed also by configuration, native format, and region. No cache is discarded in this execution. Each recurrence has a new exact G anchor after further accepted DATA; a cached object for an earlier height/hash cannot be declared ready for it. Main lines 2021–2025 expressly distinguishes cached state at different anchors.
- `rotation_v3_8_liveness.tex:44–59,94–105` obtains finite preparation only in a tail with fixed G and fixed installed slot. This execution advances G twice per macrocycle and moves to new slots. It never creates the fixed-anchor tail to which the finite-maximum argument applies. The larger `M=1+β` is matched by a larger, still finite first-preparation delay at a new target.

For a concrete native adapter, some empty blocks may permit immediate reuse of existing execution state plus cheaply validated new provenance. Such an adapter could prevent this particular delay choice. The frozen abstract model and v3.8 code do not prove a uniform completion bound for that operation, nor do they require every online cohort to have the latest exact executable continuation before selection. Thus the abstract trace uses the explicitly eventual first-construction interface. It must not be advertised as a measured attack on an adapter that has independently proved stronger preparation behavior.

The minimum material distinction is exact executable continuation, not missing Espresso bytes. All raw source namespaces, finality evidence, accepted DATA, immutable AUTH and control history arrive and are retained within their existing promises. Previous producers may possess a complete continuation, but a direct delivery from their disjoint snapshot to the recurrent cohort is outside the strict bounded domain. Their complete original result's public availability does not automatically instantiate the directly executable object at every reader.

## Premise audit

| Premise or invariant | Witness in this execution |
|---|---|
| Authenticated Δ-bounded snapshot communication | Every ReadyBundle, report, vote, FINAL, complete-close relay, and peer proposal is processed within one unit. Closure precedes the scheduled peer proposal by half a unit. |
| Immutable AUTH and fixed positive quota | Each record is used unchanged; both regions' rosters are fixed at the slot, Q=1; the zero-output retry uses the same r and record. |
| Correct repeated actually entered turns | Dovetail assigns every born identity infinitely many genuine initializer/proposer/self-vote occurrences. |
| Exclusion and regional fairness | All actors are correct, all schedules correct, no equivocation; no omitted Byzantine response is needed. |
| Native state and required authorities eventually available | Authorities are timely. Every selected exact continuation finishes at a finite time, including each late informed event. |
| Enough uninterrupted physical-regional service | Every macrocycle contains complete timely service by a newborn cohort of each physical region, and each result enters G. The committee never interrupts anything. Informed sessions are closed only by the specified automatic deadline. |
| Eventual client ingress | Every newborn cohort receives the full x before Completion, relays it within Δ, and retains it forever. Every recurrent cohort is already informed. |
| Fit, finite queue rank, continuing admissibility | x fits alone, has no older client request ahead, and remains admissible. |
| Original complete-result publication and Espresso service | Every newborn complete result is submitted and accepted timely with full evidence. The informed occurrence has only one FINAL, so no x-bearing complete-result antecedent is suppressed. |
| Increasing timestamps and accepted service | Time grows without bound, each finite macrocycle gives two accepted DATA, both regions get repeated credit, and no old G is rolled back. |
| No hidden deletion or illicit reopening | Closed partial results, committed x, reports, pending x and work ownership remain retained; later appearances use new authorized pairs. |

The uninterrupted-service row is a precise scope statement. If “enough uninterrupted service” is instead defined as a promise that each recurrent informed cohort eventually has a complete post-Ready receipt window, the trace violates that stronger premise and cannot be used against it. That definition would need to be present in the authority text; it cannot be obtained merely from repeated physical-region DATA service or from the existing eventually-available native state condition.

## Consequence and limits

In every accepted newborn result the genuine first-DATA report witness was issued before its cohort received x. In every informed recurrent appearance the witness names x, but its DATA never obtains a complete regional receipt and never becomes accepted. Consequently the v3.8 conditional awareness theorem is not contradicted: it has no infinite sequence of **accepted** aware witnesses in this execution.

The construction closes the earlier per-identity recurrence gap. It also shows why “the holders will return later” is insufficient without coupling their return to a complete result or a guaranteed transport opportunity. Original Publish/Espresso can carry requests present in complete DATA results, but the automatic retirement pattern prevents that first carrier here.

This is only a v3.8 counterexample and a boundary for repairs retaining its startup/retirement behavior. A redesigned AUTH policy, a different algorithmic preparation/activation rule, or a rigorously derived first-carrier guarantee may exclude it. No universal impossibility claim follows.

Validation: manual exact-handler/timing/slot/owed-ledger audit. No executable model checker, native implementation experiment, TeX compilation, or independent second review was run.
