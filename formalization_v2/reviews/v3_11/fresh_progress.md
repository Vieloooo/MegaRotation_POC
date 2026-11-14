# Fresh v3.11 liveness verification

## Summary

**Final Verdict:** The supplied end-to-end liveness argument is valid within its stated abstract complete-message, native-validation, persistent-service and progress premises. This review found **0 Critical Errors and 0 Justification Gaps** in the checked chain. This is a conditional mathematical verdict on the frozen specification, not a claim of implementation timing, executable model checking, or unconditional service when its premises fail.

**List of Findings:** None.

The result is not based merely on the absence of a local counterexample. The dependency checks below cover actual finite ownership transfer and universal certificate coverage, universal OC formation after selective delivery/cancellation, finite exact-anchor preparation, regional response and public acceptance, unbounded DATA, repeated regional service, and the finite-potential request-inclusion conclusion. The last conclusion uses the clarified original regional ingress premise to obtain one actual correct authenticated owner; it does not assume initial awareness at all future rosters or infer inclusion from empty DATA progress.

### Frozen evidence and verification method

- Read only `review_scope.md`, `manifest.json`, files under `snapshot/`, and the proof-verifier skill. No previous report, integration note, live source, or peer conclusion was used. No source was repaired.
- All 26 manifest-listed snapshot hashes matched before review and again after writing this report.
- Main source SHA-256: `de2e36905fef1e2de7bcc53f0cd8f4a95eca92e4561d347b33bc260e7dc58bef`.
- The entire `Model and assumptions` section was compared byte-for-byte with the v3.1 section and with `baseline/model_and_assumptions.tex`; both comparisons were equal. Its SHA-256 is `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387`.
- Manual proof/dependency and adversarial-case review only. No compiler, model checker, deployment measurement, or empirical capacity test was used as proof evidence.

### Assumptions used, rather than proved

The frozen assumptions supply authenticated same-physical-region correct-to-correct delivery and complete-dependency processing, global clock, fixed-snapshot faulty weight below half, binding hashes and unforgeable signatures, immutable authenticated authorities/control history, sound application/native validation, eventual required authority/native-state availability, complete original-result publication, the stated Espresso history/visibility service and upper timestamp-lead bound. The original fair schedules and permanent equivocation exclusion are also used. A failed or permanently absent required correct quorum is not reclassified as a serving snapshot.

The two human clarifications in the scope are used exactly at their endpoints: correct historical and current AUTH members in the same physical region share the regional communication service; sending a request to a region delivers it to every party there. Neither supplies a cross-region delay, bounded client latency, initial future-roster awareness, or a native-restoration deadline.

Protocol duties, rather than external service assumptions, supply actual local finite-work continuation, byte retention, timed-out Byzantine acquisition slots, once-per-context dispatch, journal hooks, pacing, permanent ownership while pending, and phase-independent receive routes. The proof is conditional on correct participants executing those duties. Its regional timing additionally uses the explicitly stated directly executable state/complete-handler abstraction. Residual database import, unsupported packet sizes or load, arbitrary history catch-up, and arbitrary external-observer CPU deadlines are not conclusions of this proof.

## Detailed Verification Log

All locations below are relative to `snapshot/`.

### 1. Stable retry context and finite authority dependency

**Quoted claim:** “A zero-output retry reads the same `AUTH` records” and “Their keys exclude beta” (`sections/rotation_v3_11_liveness.tex:38–63`).

The underlying transition is explicit in the main source at lines 1868–1900 and in `sections/rotation_v3_6_control.tex:240–249`: zero accepted output preserves the responsible numeric slot; positive accepted output advances it; a pending unassigned boundary preserves its stored target. Force increments beta, retains exact pre-payload `G`, and creates fresh core state. It does not transfer old Quit proofs or old clocks. The background AUTH cursor and exact-anchor preparation are independently retained (main source 1813–1843).

In a putative fixed-`G` tail, at most one later Force can close the duty containing the last positive contribution. The next unresolved/assigned target is fixed after that finite transition. A pending regular successor is already a fixed next slot. Thus a finite installed roster/configuration and a finite region cycle, rather than all future memberships, suffice for the preparation maxima.

**Quoted claim:** “These records are known to exist” (`liveness:108–115`).

Genesis establishes slot zero; every admitted positive/regular advance moves by one; zero-output reuse does not skip a key. The authenticated lineage therefore contains `AUTH[k,0]` for every `0 <= k <= r`. Known-key historical retrieval is finite. This avoids an all-AUTH enumeration or an absent-future-key oracle. Recovery schedules are a separate sequential task whose eventual availability is a stated inherited premise.

For finite installed identity set `E_i`, permanent exclusion supplies finite `G_{i,p}`. Taking their maximum and then a correct initializer index `g_i*` uses the frozen initializer fairness (`liveness:86–103`; `baseline/sync_hotstuff_rr_part1_core.tex:185–211,852–866`). Repeated beta values do not reseed those schedules. A proof for another pair or a lower generation cannot newly freeze the chosen generation. No finite-time exclusion rate is inferred.

### 2. Actual ownership acquisition terminates against Byzantine sources

**Quoted claim:** “Every referenced validation dependency is actual supplied content” and “Merely queuing a send does not define a” (`sections/rotation_v3_10_ownership.tex:130–171`).

The preparer obtains a finite authentication capsule before the timed exchanges, actually sends every chunk, then sends validation triggers after the strict `a+2d` barrier. A correct source has complete cached bytes by `a+d`; query sending starts only after `a'+2d` and the pacing release. Thus old owners can validate the target without their own new-AUTH discovery or a hidden background authentication completion. Authentication of a complete supplied context is charged to the original source-validation interface; reconstruction/catch-up is excluded from that timed handler.

The journal is finite at every reached local handler, and cuts copy maintained length/root rather than scan it (`ownership:58–89`). The argument does not need finitely many aggregate events throughout the system before an arbitrary real time. Correct-message processing excludes postponing a required query behind an infinite unfinished local handler sequence.

**Quoted claim:** “Later roots or larger counts cannot replace it” (`ownership:184–191`) and “never add slots, enlarge L, replace H or restart” (`ownership:193–231`).

Each directory identity has one descriptor deadline; a Byzantine source can choose an enormous but finite count, equivocate, send malformed metadata, or remain silent, but cannot enlarge the already frozen work. Each record has one metadata slot and a fixed finite fragment budget derived from one accepted finite length. Missing/invalid replies terminate or consume the remaining existing budget. Wrong-sender and wrong-context traffic cannot fail a correct owner's active record. Due messages precede deadline expiry. Whole-record checking is actual terminating preparation, not charged to one regional delay. The durable cursor advances without waiting for Byzantine cooperation (`ownership:233–241`).

This establishes eventual fixed-job completion, without asserting a practical storage/DoS bound or bounded total pool transfer. Unbounded later arrivals do not enlarge the job. Tombstones are harmless for the proof precisely when inclusion has already occurred; a correct source cannot tombstone a still-pending `x`.

### 3. Every HYDRATED certificate covers the needed bytes

**Quoted claim:** “Only at or after b_v+2d may v create its signed share” (`ownership:243–271`).

The definition of `b_v` uses the last actual metadata/chunk send to every target. Incremental reception exposes a complete raw handle by `b_v+d`, strictly before the share; it need not canonicalize or run application admission at the last fragment. The empty-pool case has a defined completion timestamp. The node-level receiver runs during unfinished acquisition and after worker exit.

Shares are persisted, inserted into the local collector and actually multicast. Complete certificates are formed at distinct-member weight above `F`, persisted, and first-possession relayed; no pointer counts as a full certificate (`ownership:273–299`). If there is no earlier certificate, every correct preparer reaches its share and correct weight `W-B >= W-F > F` forms one. If a certificate exists first, adoption is accompanied by durable relay and preserves already promised service/sends. Cancelling redundant acquisition therefore cannot eliminate the first-certificate formation route.

For **any** valid certificate, choose one correct signer. The correct source's immutable query cut supplies every still-pending request it held before that signer's first query; the signer's actual all-target dissemination finishes before its share. Consequently every correct target already holds the bytes before that certificate can exist (`ownership:324–342`). This universal quantifier, rather than intersection between signer sets, is what later opening coverage needs.

Old correct shares assembled into a late certificate are handled by their own actual first-query times. A Byzantine query cannot reserve another correct member's cut because query authentication binds the named target member. An early query before release is rejected instead of queued for future authorization. At equal receipt/query clock time the slot is conservatively classified as exceptional.

### 4. Exact-state selection and universal OC formation

**Quoted claim:** “It has no opening certificate” (`sections/rotation_v3_11_input_selection.tex:62–86`).

The complete `STATE_CANDIDATE` transports the original executable anchor object and validation dependencies before waiting for any offer or Ready. It grants no entry or vote. A receiver may use the verified object without separately reconstructing it. This removes the state-to-offer-to-Ready cycle, while leaving the original eventual native-preparation premise intact.

Selection freezes a finite raw/canonical cut only after a checked HYDRATED certificate; records awaiting canonicalization are included (`input_selection:88–124`). It then actually checks every record against the exact executable anchor and full ancestry. Later arrivals, beta changes, and worker replacement neither enlarge nor restart it. There is no offer before terminating local computation. The selected key is the minimum of the actual finite eligible cut, not a cryptographic assertion of a minimum over an unspecified live pool.

**Quoted claim:** “On first validation of any complete OC ... records an outstanding multicast ... before cancelling redundant preparation or notifying Ready” (`input_selection:152–179`).

This is an adequate universal-formation argument for selective Byzantine delivery. If any correct member obtains a valid OC, its persistent first-possession send reaches every target even when that member cancels its redundant selection, its worker closes, its handoff becomes stale, or its program is Terminated. Duplicates resume the original cursor; later valid variants remain admissible. The adopted certificate is checked with `CheckOC`, which requires the selected body only, so a Byzantine nonselected key without a body does not block relay of an otherwise valid OC. This differs from a collector accepting individual offers, where each nonempty contribution must supply its body.

If no correct member has an OC, none may cancel selection on that ground. All correct members finish and send their complete offers; their weight exceeds `F`, so each correct collector can form a certificate. These two exhaustive cases prove availability at **every** correct member (`input_selection:191–217`). There is no need for “all correct replies received” knowledge or for Ready to run before relay.

The receive/retention path is explicit beyond the local rule: `source_step:21–31,53–61`; main source 1824–1832; `retirement:140–157,159–178`. A transfer must give the new owner the actual bytes and route before the old owner releases them. Closure gates new BFT/input-report signatures, not the beta-independent hydration/offer/OC duties.

**Quoted claim:** “every valid opening certificate ... selects ... key(y) <= key(x)” (`input_selection:219–240`).

For a nonexceptional target, universal HYDRATED coverage puts `x` before every correct offer's cut. A valid OC has a correct offer signer because its weight exceeds `F`; that signer's minimum is at most `key(x)`. Taking the minimum of all supplied keys cannot increase it, and `CheckOC` requires a real eligible selected body. Byzantine smaller keys without an eligible body can make a proposed certificate invalid but cannot make a valid certificate evade this bound. No honest-intersection claim, common mempool root, or local-OC veto is used.

### 5. REPORT, recovery and the timed regional response

**Quoted claim:** “Recovery returning to A reenforces it” (`sections/rotation_v3_11_requests.tex:44–63`).

The first-own-session obligation is determined by validated ancestry from the fixed opening, not local `G`, a commit flag, or an earlier discarded speculative execution. Returning to the opening requires the matching OC again. Retaining a descendant retains the prior first-DATA witness. ANCHOR-only histories cannot discharge it; DRAIN requires the positive DATA quota to have been reached. Vote, recovery/NEWVIEW ancestry, own-session receipt processing and Completion all use the rule (`requests:141–165`; main source 968–987).

The initializer advertises one exact certificate digest/key under the full report context. Correct signers acknowledge that common input or a discharged ancestor. Sidecar splitting by a Byzantine initializer may stall that faulty term, but cannot create a new Quit type or suppress the independent original NEWVIEW/ANCHOR/equivocation processing (`requests:65–127`). Different valid OC encodings are witness variants; a different selected transaction changes the block and remains subject to the original block equivocation rule.

For ordinary terms the preceding terminal QC and ancestry are everywhere by `e+2Delta`; reports reach a correct leader by `e+3Delta`, and the first DATA is voted by `e+4Delta`. For term zero, prepared initial/recovery entry skew is `Delta`; NEWVIEW is sent by `e+3Delta`, independent ANCHOR votes and matching reports are sent by `e+4Delta`, received by `e+5Delta`, and first DATA is voted by `e+6Delta` (`requests:208–248`). Later positions retain two-delay spacing and message-first equality handling. No private-pool scan sits in these stages.

The original terminal-QC **entry-skew** argument survives because complete proposal forwarding and correct votes still disseminate the QC even when the successor is Byzantine. The old faster correct-successor proposal bound is not silently reused for the amended REPORT path. A correct initializer still proposes by `e+5Delta`, preserving the protected initial commit window from Part I (`baseline/...core.tex:630–657,870–894`).

The conservative regional bound uses at most `Q` DATA terms or one DRAIN term per generation, finite authority prefix through `g_i*`, and the bounded progression/freeze/result alternative per term (`liveness:152–203`). If a terminal correct vote exists, its complete forwarding either leads to a QC and advancement/result or a Quit intervenes; otherwise the final checks produce matching BLAME. At `g_i*`, exclusion rules out equivocation interruption and the correct initializer's protected window yields all-correct commitment by `e+8Delta` and full FINAL collection by `e+9Delta`. Endpoint recovery through DRAIN still completes positive DATA after the opening. This proves actual progress to the chosen initializer, rather than merely locating it in a schedule.

**Timing scope:** `ready_bundle:134–177,186–267` explicitly distinguishes direct adoption of an already executable object from an unbounded residual native import. It lists OC/advertisement/REPORT bytes and checks and does not infer fixed delay from finiteness or singleton DATA fit. At the formal complete-handler abstraction, the dependency list and handler satisfy the stated conditional arithmetic. The proof makes no unconditional implementation claim where those concrete packets/checks are outside the original admitted profile. Missing deployment benchmarks therefore are not a Justification Gap in this stated formal theorem; they remain implementation obligations, and no larger envelope or native deadline was assumed in this review.

### 6. Complete original result to canonical first-DATA acceptance

**Quoted claim:** “The Publish Layer input remains exactly R_0” (`requests:167–204`).

Erasure removes only new regional OC/advertisement/REPORT sidecars. It retains original DATA, FINAL, proposal/QC/ancestry, authority, boundary, native-validation and public-source evidence. Thus the proof neither starts `T_pub` on a partial receipt nor grants a new publication service to bare requests or a weaker public kernel.

The public FINAL kernel has a genuine correct signature at weight above `F`; a correct FINAL is generated by the timer Commit action, not by receipt-driven Completion (`receipt_kernel:33–92`). Its timer-origin Vote checked full own-session input ancestry, including the first-DATA OC and required report witness. Earlier inherited anchors use causally earlier kernel conclusions. A publisher's alternate recovery or native sidecar is not substituted for that genuine historical witness. Independent append/context/reference/cutoff gates remain mandatory.

Logical carrier normalization preserves the original complete-result subgraph and its public DATA/FINAL projection even under false outer hints, ancestry-only placement, or invalid neighboring evidence (`carrier:108–159`). It supplies no absent bytes or new codec/discovery service. Retained rejected namespaces and the complete old source cut prevent a hidden Espresso archive fetch in the prepared path (`source_step:38–93,203–223`). Optional high-generation AUTH/native sidecars cannot block inventory or public-kernel processing.

Source availability records raw bytes and the import descriptor and immediately relays before waiting for P6, native preparation, or deferred request import (`source_step:145–171`; `public_requests:96–107`). The prepared source handler advances canonical prelude and per-DATA transactions, then attaches already executable state/OC for Enter; later callbacks retain exact target identity. Its timing assertion is the stated regional handler scope, not arbitrary external observer processing (`source_step:225–268`).

For the first result after fixed `G*`, a correct FINAL can exist only after the retry's finalized opening/reference. Hence its carrying record is strictly later than the required reference and forced opening block. If a Force precedes payload, closure supplies the alternative outcome; otherwise the first DATA is the authorized next height and passes the kernel and independent gates. Wrong hints do not require a corrected publication. With quota one, terminal DATA commits and is exposed **before** successor AUTH or Owed resolution can wait (`exports:107–162`; `liveness:298–347`). The resulting finite prepared response is `L = max_i (2 T_E + 2 Delta_i + R_i + T_pub)`.

### 7. Unbounded global DATA, including singleton eligibility

**Quoted claim:** “There is no AUTO-created unassigned interval” (`liveness:368–410`).

At fixed `G*`, the health journal has finitely many new credits and finite retained prior operations; the last effective health/status prefix can be discarded. Existing cooldowns expire because timestamps grow. `PrepareOwed` prunes OFF/zero-weight members, refills an empty round at a public boundary, and ignores cooldown when recording debts (`health:120–177`; `control:101–175`). With a nonempty eligible serving set, conditional AUTO cooldown never removes its last eligible owed source; if another eligible owed candidate exists, that candidate supports immediate reassignment. Thus singleton self-renewal is included, without clearing OFF or an old cooldown.

After preparation, each retry either grows `G` or closes within the same finite `L`. Under the assumed absence of growth, closures are AUTO and immediately open the next retry; the startup phase persists. Their timestamp spacings are at least `(1+beta_n) W_min`, while real-time spacing is at most `L`. Since beta increases once per effective Force, the sum of timestamp requirements is quadratic, contradicting the frozen upper-lead bound against a linear real-time sum. Timestamp jumps cause one Force and open at their realized timestamp; no same-prelude cascading or replenishing unassigned gap is used (`control:24–90,273–284`). Initial timestamp lag is a finite additive constant, not a new lower-lag assumption.

This proof derives growth without taking the frozen paragraph's already-successful EXPORT-retry interval as the desired conclusion. It remains conditional on serving regional quorums, eventual fixed preparation, original services, and the controller/retention/handler rules; an ineligible or permanently nonserving assigned snapshot is not covered by the finite response premise.

### 8. Repeated accepted service and the exact Owed quantifier

**Quoted claim:** “Only Force selection is filtered by O” (`liveness:421–510`).

The proof respects this distinction. A fixed owed round is finite; each other member can be discharged or pruned once, and ON cannot add new members before the round ends. If Forces are infinite, a productive new version's first accepted DATA must be from its initially selected owed region: strict cutoff and contiguous acceptance cannot skip that session. An unserved persistent debt therefore permits only finitely many other productive-version removals; eventually no growth would contradict the preceding theorem.

If Forces are finite, finite session quotas and unbounded accepted DATA imply infinitely many accepted regular sessions. Their opening/selection floors strictly increase by at least one (`main:1902–1915`), eventually passing permanent eligibility and cooldown expiry. The fixed cyclic `NextLive` scan cannot indefinitely bypass an eligible fixed region even if other regions change eligibility. Contiguous acceptance then requires its selected session's contribution. The same finite-removal argument finishes an earlier round, and subsequent duty boundaries admit a newly permanently eligible region into a later round.

The conclusion is repeated **accepted DATA service**, not transaction inclusion: an empty DATA can discharge a debt. No step treats selection alone or a speculative regional commit as accepted service.

### 9. Original ingress to finitely many missed accepted opportunities

**Quoted claim:** “some region C contributes DATA infinitely often” and “at least one such member p obtains the complete authenticated x” (`liveness:598–619`).

Unbounded global DATA and the finite physical region cycle give a recurring accepted region independently of request awareness. Under the explicit human clarification, the original premise that the request reaches continuing regions supplies actual receipt at a correct party in one serving authenticated snapshot. Its membership has a finite index `k`. The first authenticated receipt persists bytes and the owner journal before acknowledgement, and that original owner remains responsible while `x` is absent from `G` (`ownership:91–108,419–444`). A send attempt, unauthenticated raw candidate, or Byzantine-only delivery is not substituted for this event. This is exactly the original-ingress step; no all-future-roster awareness premise remains.

**Quoted claim:** “Equality is included” (`liveness:531–583`).

For every correct target query, the actual first send obeys `s_v >= tau_0 + r tau_p`, with `tau_p > 0`. Queries at or before the finite receipt time therefore use the finite set `E_x`. Include slots below owner index `k` as well. A finite sum over the finite authenticated rosters in these finitely many numeric indices bounds all early cuts; no finite union of future memberships or global finite-event-rate premise is needed.

Outside those indices, `p` is in the authenticated directory and every correct first query is strictly later than receipt. Universal HYDRATED coverage thus precedes every correct offer's post-certificate cut. A late certificate formed from an old early correct share belongs to `E_x`; beta reuse creates neither a new cut nor a new release. Every excluded numeric slot has a fixed finite quota and can have only one positive accepted pair: zero-output Force accepts none, whereas positive Force or regular completion advances the numeric slot. Strict cutoff prevents old-pair revival. Hence only finitely many accepted DATA/session-first opportunities can be exceptional.

### 10. Infinitely many aware first blocks force finite-potential inclusion

**Quoted claim:** “Contiguous global acceptance includes every such session's first own-session DATA” and “Phi_x strictly decreases” (`liveness:621–666`; `requests:250–295`).

Each session has finite quota and each fixed slot finite accepted contribution, so a recurring accepted region supplies infinitely many positive sessions and accepted first-own-session blocks. Removing the finite exceptional indices leaves infinitely many covered blocks. DRAIN, ANCHOR-only histories, and recovery to the opening do not substitute for the required first DATA. Several ancestors arriving in the same EXPORT preserve this ordering because acceptance commits per DATA.

For any remaining first block, `x` is in every correct offer's cut, remains absent from accepted opening ancestry, fits alone, and passes admission at that exact opening by the theorem's explicit persistent-admissibility premise. Every valid OC therefore selects a real `y` with `key(y) <= key(x)`. Full regional validation makes even a Byzantine proposer's accepted first DATA execute `y` as primary, and the actual correct FINAL signer establishes that historical fact after public sidecar erasure.

Keys compare finite canonical byte length before binding hash. Consequently the set of canonical logical identities with key at most the fixed `key(x)` is finite, irrespective of future arrivals or adversarial choice. At each counted first-DATA acceptance, its exact opening is the accepted prefix immediately before that block, including earlier commits within the same EXPORT. Ancestry exclusion makes `y` a previously absent identity, so the nonnegative potential decreases strictly. Prefix extension never increases it; Force never rolls back it; discarded speculative executions do not spend it. Infinitely many decreases are impossible. This proves eventual inclusion under the stated persistence/admission premises.

### 11. Adversarial cases and remaining scope

The checks above include cold/late target preparation; absent future authority keys versus known historical keys; Byzantine descriptor growth, equivocation, invalid metadata and silence; actually sent versus merely queued envelopes; source-context validation ordering; equality at timers and query receipt; cancellation on an already valid certificate; old shares assembled late; selective OC delivery; nonselected Byzantine keys without bodies; closure, worker exit and Terminated receive routing; stale beta and repeated numeric slots; recovery returning to the opening; speculative first-DATA rollback; DRAIN; quota one; singleton eligibility; stale/bad-hint public records; optional malicious recovery-authority sidecars; and public-import backlogs.

No surviving in-model counterexample or missing proof arrow was identified in this assigned liveness chain. This verdict retains the source's explicit boundaries: finite preparation is not uniformly bounded restoration, packet finiteness is not capacity headroom, regional DATA growth is not alone request inclusion, and original admissible ingress plus persistent admissibility are substantive hypotheses. In particular the review does not convert the complete-handler contract into a claim about an unspecified native adapter or certify the entire manuscript beyond the checked dependencies.
