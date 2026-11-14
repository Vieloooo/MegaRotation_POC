# Summary

**Final Verdict:** The submitted proof package is invalid as written because the universal conclusion of `prop:opening-formation` has a **Critical Error**: selective adoption of an OC can cancel the remaining correct offer without a specified dissemination action for that adopted OC. This is a failure of that local proposition, not a counterexample proving the final ingress-to-`G` theorem false. The full liveness dependency chain was examined; its downstream progress claims are not certified here by silently replacing the failed proposition with a weaker one.

**List of Findings:**

- **CE-OC — universal OC availability does not follow from early adoption.** Location: `sections/rotation_v3_10_input_selection.tex:144–155,169–185`, especially “A correct member may adopt any valid matching certificate and cancel redundant unfinished selection” and “a complete opening certificate is eventually available at every correct target member.” The only explicit OC multicast is the collector's action after obtaining weight `>F` of complete valid contributions. First possession of an already-valid OC has no corresponding mandatory multicast. Such a certificate can contain signed nonselected keys without their bodies, so possession need not satisfy the separate complete-contribution collection condition. After closure, Ready cannot provide the omitted relay. A legal selective-delivery execution is detailed below.

**Freeze and provenance.** Only `review_scope.md`, `manifest.json`, the frozen `snapshot/`, and the proof-verifier skill were consulted. No earlier reviews, integration notes, live source, or peer verdicts were read. All **26/26** snapshot file hashes matched the manifest before review and again after writing this report. Main SHA-256: `1555702839d5d117a95428953d91efcd28154c96a719119a2d178086d7e45bab`. The complete candidate Model section is byte-identical to the corresponding v3.1 section (16,456 characters); its stripped text also equals the saved `baseline/model_and_assumptions.tex`.

The authorized human answer, “覆盖同一地区的新旧正确成员”, was used exactly as a within-physical-region correct-endpoint scope across AUTHs. The former missing-answer issue is not a finding. This review adds no cross-region, archive, native-restoration, whole-pool-selection, or deployment-capacity deadline.

# Detailed Verification Log

## 1. Statements, assumptions, and dependency map

The principal conclusions examined are finite prepared regional response, continuing global accepted DATA (including a singleton eligible region), repeated accepted service for a persistently eligible region, and eventual request inclusion under original ingress and continued admission. The relevant chain is:

1. An installed numeric slot has a finite known initial-AUTH lineage; fixed-slot preparation survives beta changes.
2. Correct persistent owners answer fixed finite cuts; finite reply slots and actual all-target dissemination establish HYDRATED formation and coverage.
3. A correct post-HYDRATED finite selection examines the exact opening state; an OC contains a correct offer and a real eligible selected body. OC formation/dissemination is the failed universal subclaim CE-OC.
4. Complete prepared Ready/NEWVIEW/REPORT inputs retain the original entry and fresh-vote deadlines; finite recovery-prefix traversal reaches an uninterrupted correct initializer or an earlier positive complete result.
5. Retained source bytes, the original full publication value, public FINAL provenance, and per-DATA commit yield prepared global growth or closure.
6. Fixed-slot retry tolerance plus the timestamp upper-lead bound contradict indefinitely stalled global DATA; debt and historical cyclic selection then yield repeated regional service.
7. Global growth in a finite physical cycle supplies a recurring accepted region. Original ingress supplies one correct owner receipt there. Positive release pacing and monotone numeric slots leave finitely many exceptional accepted sessions.
8. Each remaining accepted session-first DATA spends a previously absent logical request identity with key at most that of the waiting request. A finite nonincreasing potential cannot decrease infinitely without inclusion.

The original assumptions used, rather than proved, include authenticated channels and a global clock; `F<W/2`, fixed session membership and the accepted-lifetime compromise bound; retained correct state; sound regional/native input-validation interfaces; authenticated complete L1 and Espresso histories; eventual required authority and executable-state availability; original complete-result publication and online visibility services under their stated admission/load scope; growing Espresso timestamps with only the stated upper clock-lead bound; regional fair schedules and permanent exclusion; the original client ingress and continued-admission hypotheses. The finite health policy, persistent work, pacing, custody, and dissemination rules are protocol behavior, not new external service oracles.

The frozen Model still contains the old sustained-service/retried-EXPORT progress wording. The new source-to-`G` argument was checked on its own route through a first logical carrying record; it was not certified by assuming in advance that the next required DATA is accepted. Original regional turns being actually entered are also distinct from schedule fairness alone.

## 2. Known-key ownership discovery and physical custody

> “Obtain missing records by their known keys `(k,0)` from this lineage.”

`ownership.tex:14–56` and main source `1852–1884` give the required numeric lineage. Genesis targets zero; regular succession and positive-output Force advance one; zero-output Force retains the responsible slot; an unassigned boundary retains its stored target. Thus a genuinely admitted `r` has existing predecessor initial authorities through `r`. No absent-index probe or enumeration of all future authorities is needed. A checkpoint is required to retain their identities and authentication dependencies. Immutable authority at reused numeric `r` prevents multiplying unrelated rosters at one exceptional index.

The directory is the finite union of those initial memberships, deduplicated by authenticated persistent identity; the communication maximum is over that finite prefix. Recovery does not change membership. The authorized endpoint clarification applies to correct historical/current members of this same physical region. It does not cover an arbitrary failed machine.

> “Only verified inclusion in `G` permits removal.”

`ownership.tex:58–108,419–444` explicitly keeps receipt journals, pending bytes, raw handles, cuts, registered rosters, outgoing sends, and receive routes outside worker state. Physical exit is prohibited while original pending ownership remains unless the same logical owner's full service survives the handoff. This is sufficient to avoid chasing arbitrary moving custodians. Old owners without a needed pending/outgoing duty may be silent; the acquisition does not require them to remain correct online responders.

A late request at an owner that first appeared at `k` is not promised coverage by `r<k`; those finitely many indices are explicitly exceptional. No finite union of all future memberships is used.

## 3. Finite cuts, context barriers, and Byzantine transfer behavior

> “The journal starts finite … There is no limit transition that installs an infinite journal.”

`ownership.tex:80–89,130–241` supplies finite reached-state journal cuts without assuming a global finite event rate. The cut copies maintained length/root; it is not an uncharged pool scan. Raw complete handles remain distinct from authenticated canonical receipts.

The sender actually transmits the capsule's metadata and fixed chunks before defining its last-send time. The validation triggers follow `a+2d`, and actual first queries follow `a'+2d` and the release guard. Consequently required correct owners have bytes before validation and validated context before the query, with strict inequalities that avoid an equal-clock ordering assumption. Sender identity authenticates who may create a target's cut.

Each owner descriptor is frozen once at its actual query-send deadline. Each retained descriptor fixes finite record indices; each metadata slot fixes finite length/hash/chunk count. Silence, malformed replies, conflicting expected-owner replies, or oversized but finite declarations cannot add slots or restart a completed one. Wrong-sender/context traffic cannot fail a correct owner's transfer. Due replies are processed before expiry. Whole-object canonical/authentication checks run as actual terminating preparation, not as an inferred one-`d` operation.

For a correct owner holding an unaccepted request at its query cut, every required reply is available under the clarified original endpoint service and complete-message scope. A verified global-inclusion tombstone is harmless; speculative execution or a different roster's acknowledgement is insufficient to discard the request. These steps establish finite fixed-slot acquisition under the stated local continuation and admitted-load behavior. They do not prove a practical storage or denial-of-service bound.

## 4. HYDRATED coverage, offers, and the OC finding

> “All required raw handles are retained at each correct target by `b_v+d<b_v+2d`, strictly before the share.”

`ownership.tex:243–342` explicitly multicasts every frozen pool record to every target, stages out-of-order fragments, and exposes a raw handle before releasing fragment receive duties. Receivers run without a production worker and while their own acquisition is unfinished. A correct share is created only after the last actual all-target send plus `2d`.

The HYDRATED collector has the important complete dispatch rule at `283–287`: on first possession of a complete valid certificate, actually multicast it, and each receiver performs the same once-per-context relay. Therefore cancellation of private acquisition on adoption does not strand the certificate. If none exists, all correct members continue; correct weight is `W-B>=W-F>F`. The formation and every-certificate coverage claims are justified.

The coverage quantifier is genuinely universal. Any valid certificate has a correct signer; that signer's actual dissemination precedes its signature, hence precedes every certificate containing that signature. Every correct target receives the request. No honest intersection between different certificate signer sets is assumed.

> “The cut includes every complete retained raw record already received and every retained canonical entry.”

`input_selection.tex:39–124` independently distributes `STATE_CANDIDATE` before offers or Ready. Each correct selector freezes its raw/canonical cut after checking HYDRATED, then actually assembles, canonicalizes, authenticates, checks ancestry/admission/fit at the exact opening `A`, and finds its minimum. Native conversion, ancestry indexing, and the scan are preparation. New arrivals cannot enlarge the cut. No Ready/offer cycle or post-Enter pool scan is inserted.

The OC key includes the exact producing anchor identity, authority and native domain, but excludes target beta; equal-context retry reuse is valid. A different exact anchor starts different work. `CheckOC` needs the selected real body and minimum signed key, not each unselected body or a private queue. Thus an OC with `>F` target signatures contains a correct offer, and its selected eligible key is no larger than that correct offer's key. Combined with universal HYDRATED coverage, `prop:opening-coverage` is locally valid even for Byzantine certificate assembly. A local OC is not an exclusive choice.

### CE-OC: selective OC adoption does not imply all-target availability

The collection/adoption text states:

> “Each correct collector checks complete offers, including each nonempty contribution's body and eligibility … As soon as retained valid contributions exceed `F`, form, persist and actually relay the complete certificate to all target members.” (`144–148`)

> “Every recipient verifies and retains the complete certificate. A correct member may adopt any valid matching certificate and cancel redundant unfinished selection … Absent an already valid certificate, it continues its finite job and offer dispatch.” (`150–155`)

The proposition then asserts:

> “a complete opening certificate is eventually available at every correct target member” (`169–173`), with the proof saying “Early adoption can only make a certificate available sooner” (`184`).

The asserted universal conclusion has the following allowed execution.

1. Use three unit-weight target members `H0,H1,B`, with `H0,H1` correct and `F=1`. Ownership and exact native preparation finish as required. `H0` completes its finite selection and multicasts its nonempty valid offer with body `x0`. Delay `H1`'s own finite selection; it has not signed an offer yet.
2. `B` signs a nonempty offer key `kB>key(x0)` but supplies no corresponding request body. It constructs `OC={H0's signed key,B's signed key; selected body x0}`. This is a valid OC under `CheckOC`: signer weight is two, `x0` is the minimum supplied key and is eligible, and the definition explicitly does not obtain nonselected bodies (`137–141`). The construction does not require a false correct signature or an invalid selected request.
3. Close the relevant production pair before any correct Ready/Enter. Then `B` supplies this valid OC only to `H1`. `H1` adopts it and permissibly cancels its unfinished selection. It retains the certificate and all already-promised duties. The specified adoption action does not initiate an OC multicast.
4. `H0` receives no offer from `H1` and no complete valid contribution from `B`. It cannot collect weight greater than `F` of complete contributions. Receiving `H0`'s already-multicast offer at `H1` does not turn `B`'s bodyless nonselected key into the complete contribution required by the collector. Thus the explicit collector multicast need not occur at either correct member.
5. Closed blocks the independent ReadyBundle relay. The slot can subsequently produce DATA at another physical region and advance, so the old exact target need never become activatable again. The finite-work and retention duties continue as prescribed; neither requires retransmitting an adopted OC. `H0` can remain without an OC forever.

This is an adversarial-scheduling/dispatch counterexample to the **local universal proposition**. It is not cured by correct-to-correct synchrony: no relevant OC send from `H1` to `H0` is specified. It is not the HYDRATED issue, whose first-possession relay is explicit. It also does not assert failure of the OC's request-coverage predicate; the supplied OC can be perfectly valid.

The failed conclusion is stronger than mere existence of a complete OC at one correct member. The counterexample therefore does not, on its own, falsify eventual global progress or eventual request inclusion. Nevertheless `liveness.tex:127–145` cites the formation proposition in establishing finite preparation, and subsequent public/global/service proofs cite that result. This report does not replace that dependency with an unstated weaker formation-and-later-Ready argument or repair the specification.

## 5. REPORT, first-own DATA, recovery, and prepared timing

The following checks are independent conditional checks of the supplied prepared-input interfaces; they do not repair CE-OC.

> “If the list is empty, the next DATA is the first own-session DATA … It must execute a nonempty opening certificate's selected body first.”

`requests.tex:44–165` bases discharge on validated ancestry from the fixed opening, not local `G`, regional commitment, or an arbitrary EMPTY report. Returning to `A` reenforces the obligation; a retained descendant carries the historical first-DATA witness. ANCHOR alone cannot discharge it. DRAIN requires a positive DATA quota already reached and cannot be an empty session substitute. Full Vote, own-session receipt, recovery, Completion and later-position ancestry checks all retain these witnesses.

The initializer signs an exact context/OC digest/key advertisement. Correct reporters acknowledge that advertised certificate or genuine discharged ancestry; they do not compare private queues or attached local OC variants. A Byzantine initializer can split advertisements, but this only stalls its term, while original progress checks still fire. The original signed NEWVIEW/PROPOSE rules, rank, locks, and evidence inspection are unchanged. Independent ANCHOR voting and forwarding are not gated on sidecar success.

The arithmetic in `requests.tex:208–248` checks out **for the stated complete-handler interface**. In an ordinary term, terminal QC/ancestry reaches all by `e+2Delta`, reports reach a correct leader by `e+3Delta`, and first DATA is voted by `e+4Delta`. For term zero, entry skew is `Delta`; correct NEWVIEW goes by `e+3Delta`, complete processing/report and ANCHOR votes by `e+4Delta`, both inputs reach the leader by `e+5Delta`, and first DATA is voted by `e+6Delta`. Later positions retain `2Delta`. Message-first equality preserves the original progress deadlines. No private minimum or new state restoration occurs on this path.

The ReadyBundle actually contains executable state, its original validation dependencies, and one complete OC. The handler checks and attaches those inputs and enters atomically; it does not fetch a pool. `ready_bundle.tex:134–177,186–267` correctly distinguishes this abstract contract from a residual physical import `u_v`. The report does not infer `u_v=0` from finiteness or demand a concrete codec benchmark for the stated formal task. The stated timing remains conditional on those concrete logical dependencies and operations being admitted by the original complete-message/native interface; it is not an unconditional timing claim for arbitrary adapters.

## 6. Finite regional response and original publication/source path

`liveness.tex:86–115` uses only the fixed roster's finite set of ever-exposed equivocators and their finite exclusion indices. Immutable schedules across beta retries allow a common finite good initializer generation `g_i*`; an old-beta Quit cannot create entry in the new pair. The background fetch task preserves its cursor and obtains a fixed finite AUTH prefix. Required lineage authorities are existing known keys, as checked above.

Conditionally on the prepared OC/Ready inputs, the supplied term/gen bound is conservative. Last progress checks run even when an opening is missing. A terminal vote triggers complete forwarding and correct votes, or Quit intervenes; without any correct terminal vote no terminal QC can have advanced the term, and matching correct BLAME eventually freezes it. A DATA generation has at most its finite quota in terms; DRAIN has one. Cached recovery authorities leave the original recovery waits. Hence the argument proves actual entry to the good initializer instead of substituting fair schedule names. The correct initializer's protected first-proposal/commit/FINAL timeline gives `e+5Delta`, `e+8Delta`, `e+9Delta`; a DRAIN receipt covers positive own-session DATA because the quota is at least one.

`receipt_kernel.tex:33–92` traces a raw `>F` FINAL set to an actual correct timer-origin Vote. Receipt-driven Completion emits no new FINAL and cannot be a circular first cause. The actual signer checked genuine full regional OC/report/ancestry inputs; publisher-selected alternative witnesses are not used for this inference. Inherited-anchor conclusions use earlier signature events. This supplies intrinsic input validity, not arbitrary native output, executable state, or public append authority.

The public projection remains the **entire original `R0`** (`requests.tex:167–204`, `exports.tex:201–238`). Only new regional sidecars are erased. No bare request, Ready, HYDRATED, or kernel-only object obtains a new publication deadline. Logical normalization preserves existing structured DATA/FINAL objects despite bad outer hints, without treating missing payload, private-only pointers, arbitrary ciphertext, or undiscoverable namespaces as complete public input.

`source_step.tex` retains actual rejected and accepted namespaces before cursor advance and starts independent import work by descriptors. Fixed-cut hydration does not chase a moving endpoint. Optional recovery/native/L1-reference sidecars cannot create an external lookup on the current public kernel path. P6 commits and exposes each accepted DATA before successor `AUTH_{r+1,0}` or `PrepareOwed` can wait. In particular quota-one first DATA is not held behind successor preparation. Source intake/relay does not await P6, native work, ownership work, or deferred import completion.

Under the prepared source-handler contract, the two source stages and the original complete-result service give the stated `2TE+2Delta_i+Ri+Tpub`. A genuine result's correct FINAL signature postdates validation of its finalized opening-or-later Espresso reference; immutable history puts its carrying record strictly later. An already-carried result need not obtain a corrected wrapper. Strict pre-payload cutoff, same-session coverage, exact next-height extension, and the independent global gates remain necessary. An effective Force supplies the closure alternative; no old completed record is reopened to rescue a closed version.

## 7. Global progress, singleton selection, and per-region service

These are dependency checks conditional on the prepared-response result, not unconditional certificates after CE-OC.

At a stalled accepted frontier, at most one positive-output duty transition remains before a fixed numeric slot. The finite health-controller journal allows only finitely many further matching MSIGs/status changes at that frontier. Cooldowns existing in that finite prefix expire because Espresso timestamps grow. `NextOwed` and conditional AUTO cooldown preserve a candidate once fixed eligibility and owed state are reached: cooling the source requires another currently eligible owed candidate; if the source lies outside the owed set, such a candidate already exists. A singleton source in the owed set is not cooled. No arbitrary unassigned real-time gaps are smuggled into the consecutive-retry argument.

For zero-output consecutive retries, `M_n=1+beta_0+n` and unchanged positive base windows give `theta_(n+1)-theta_n>=M_n W_min`. Prepared response gives closure by `a_n+L` under the assumed absence of growth. Thus the quadratic sum of timestamp increments is bounded above by a linear real-time term plus the original clock lead, a contradiction. No lower clock-lag bound or fixed empty-block rate is used. If an earlier cold assigned zero-output duty persisted forever, its finite fixed miss threshold and growing timestamps would themselves eventually trigger AUTO; initial authority/state/hydration waits are not assigned a uniform failover bound.

The repeated-service proof distinguishes two cases correctly. Infinitely many productive Force-created versions must first accept DATA from their initially selected owed regions; with an unprunable fixed debt, only finitely many other members can be discharged/pruned. Finitely many productive versions plus infinitely many Forces would contradict global progress. With finitely many Forces, finite quotas imply infinitely many accepted regular sessions; the strict `q_open+1` floor eventually passes any fixed eligibility/cooldown position, and cyclic NextLive cannot skip an eligible fixed region forever. Regular selections need not belong to the owed set. The old-round argument ensures newly persistent eligible regions eventually join a round. Empty DATA may discharge regional debt, so none of this is substituted for request inclusion.

The scope is the original regional serving premises. It is not a theorem for arbitrary permanently missing correct quorums in other assigned snapshots, a permanently OFF region, or unlimited administrative/emergency intervention outside the finite policy.

## 8. End-to-end ingress, finite exceptions, and potential

The final proof's quantifiers were checked through to accepted request identity; they were not replaced by local conditional awareness alone.

If global accepted DATA is unbounded, finiteness of the physical cycle gives at least one region contributing infinitely often. The proof applies the original ingress premise to that continuing region and needs one finite actual authenticated receipt at a correct regional owner, with some finite membership index `k`. It does not require delivery before every future roster begins. Continued correct-owner custody, not generic physical health, keeps this particular receipt available. If “reach the regions that continue serving” were interpreted only as delivery to Byzantine endpoints, it would not be the correct-owner ingress used here; the source uses the ordinary correct-protocol-receipt interpretation.

For fixed `x` received at `t_x`, actual query pacing gives the finite set

`E_x = { r >= 0 : tau_0 + r tau_p <= t_x }`.

Equality is conservatively exceptional. After removing `r<k` as well, each correct first query is strictly after the receipt and has the owner in its authenticated directory. Every valid HYDRATED certificate therefore covers `x` before any correct offer's cut. A late certificate assembled from an old early share belongs to an already-excluded slot; a new beta cannot create another correct first query for the same key. Missing/unauthenticated target indices add no cuts.

Numeric `r` cannot decrease in canonical accepted history. Zero-output retries add no DATA; positive-output Force and regular completion advance `r+1`; strict cutoff prevents the old pair contributing again. Thus each exceptional installed slot contributes at most its fixed finite quota, regardless of arbitrary changing future memberships or infinitely many zero-output retries. Repeated accepted DATA in the recurring region therefore entails infinitely many nonexceptional accepted session-first blocks.

For each such first block, contiguous acceptance includes its own first DATA before descendants, even within one EXPORT. Its actual opening `A` is the accepted predecessor; unaccepted `x` remains absent from that ancestry. The theorem explicitly assumes `x` remains admissible at these exact accepted openings and fits alone. Universal OC coverage then selects a real eligible `y` with `key(y)<=key(x)`, and genuine full regional first-own-DATA validation requires `y` first and absent from `A`'s ancestry. Byzantine proposers and public sidecar erasure do not remove the genuine correct FINAL signer's earlier check.

The length-then-binding-hash ordering gives finitely many canonical byte strings at most `key(x)`, hence finitely many represented logical identities. Prefix extension makes their absent-count potential nonincreasing. Each counted accepted first DATA spends a previously unaccepted identity and strictly decreases it. Discarded/speculative blocks spend nothing; Force never rolls accepted `G` back. This final potential implication is sound conditional on the preceding global-progress dependency. No separate future-roster-awareness assumption remains in that implication.

## 9. Cold cases, retirement, and validation limits

The following adversarial cases were checked directly: initial zero-output retries; receipt/query equality; an owner receiving `x` after early shares/cuts; wholly changed future rosters; arbitrarily large but finite Byzantine metadata and fragments; sources silent after legitimate physical exit; redundant acquisition cancellation; recovery to the opening versus retention of first-DATA descendants; DRAIN-only recovery at the positive endpoint; quota-one terminal commit; bad-hint records and delayed required source objects; old/closed worker callbacks; and an OC learned only after closure. The last case yields CE-OC; the others do not introduce an additional identified issue in the checked liveness chain.

Retirement transfers actual bytes, receive routes, pending work, receipt/closure relays, ownership, native candidates, and selection work to persistent managers before release. Existing commit timers and evidence processing survive the fresh-signature gate; original complete-close `4Delta` is a drain threshold, not completion of all storage/publication duties. Source/closure callbacks can run without reviving a worker. None authorizes an old version to append.

No executable model checker, deployment measurement, or benchmark was supplied or run. No source was repaired. Compilation was not used as proof evidence. Sound native validation and the admitted complete-message/source-processing abstraction remain stated interfaces; arbitrary database import, archive catch-up, unbounded envelopes outside that abstraction, and observer application-processing deadlines are not established. These accurately delimited obligations are not reported as new missing original-model assumptions. The one reported mathematical defect is the OC dissemination proposition, and the final request theorem is **not declared false** solely because of that local defect.
