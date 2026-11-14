# Fresh v3.8 progress verification

## Summary

**Final Verdict:** The reviewed local REPORT timing, finite prepared regional response, global DATA progress, repeated accepted regional service, and conditional request-inclusion arguments are valid under their expressly stated abstract complete-handler and environmental premises; no Critical Error or Justification Gap was identified in those conditional claims. Full request inclusion across arbitrary disjoint future AUTH rosters remains unproved, as the candidate explicitly states.

**List of Findings:** None in the stated conditional claims reviewed here.

**Scope limits, not theorem errors:**

- The conditional request theorem assumes awareness in infinitely many genuine accepted report witnesses. Neither repeated regional DATA service nor pointwise eventual request arrival proves that premise for arbitrary changing rosters.
- The regional timing conclusions require the specified complete message, exact-parent eligibility-index, ReadyBundle, and prepared SOURCESTEP handlers. This review does not certify a deployed native adapter, an extra deferred pool scan, CPU/bandwidth performance, a binary codec, or a broader publication interface.
- This was a source-level adversarial proof review with hash verification, not executable protocol testing or model checking. It does not upgrade acknowledged open scope into a completed user-level request-liveness result.

**Frozen evidence:** Before review, all 23 entries in `manifest.json` matched their SHA-256 hashes. The main source was `c8611baf07c74f34512a182b0b3ace81d6f8f3cf65fb0cb567af696ec8a73574`. The nested baseline manifest names the same four baseline files and hashes. Hashes were checked again after writing this report. Only `review_scope.md`, `manifest.json`, and files under `snapshot/` were used as task evidence; no author design notes, other review reports, prior verdicts, or Chinese companion outside this snapshot were consulted.

In the log below, `main` means `snapshot/sync_hotstuff_rr_part2_rotation_v3_8.tex`; section filenames are under `snapshot/sections/`, and `Part I` means `snapshot/baseline/sync_hotstuff_rr_part1_core.tex`.

## Detailed Verification Log

### 1. Dependencies and the scope of the inherited assumptions

> “The delay bound includes delivery and processing of complete validation dependencies.” — `snapshot/baseline/model_and_assumptions.tex:177–179`.
>
> “The quantitative source argument concerns prepared regional processing that witnesses canonical G progress, not every observer's wall clock.” — `rotation_v3_8_liveness.tex:14–15`.

The dependency chain is local report emission/collection → original fresh-position deadlines and protected initializer window → finite regional result bound R_i → prepared public response bound L → global DATA growth → repeated regional accepted service. Conditional request inclusion has a separate chain: genuine historical full validation → an aware correct reporter in the hypothesized witness → an executed bounded-key primary → strict decrease of a finite potential. The latter awareness premise is not derived from the former DATA-progress chain.

The fixed-snapshot fault premise gives correct weight at least W−F>F; it supplies both enough correct reports/votes and a correct signer in any weight->F witness. The proofs do not require intersection between two arbitrary weight->F sets. Fair schedules, permanent exclusion, eventual finite authority/state preparation, publication, and source visibility remain explicit premises. The finite regional maximum is taken only after fixing a stalled frontier and its installed slot; it does not assume a finite union of all future rosters.

### 2. Request identity, retention, FIFO remainder, and the eligibility projection

> “A request is finally removed only when its identity occurs in G.” — `rotation_v3_8_requests.tex:13–20`.
>
> “Parent and request processing completes the required admission and ancestry filtering before exposing this projection to the report action.” — `rotation_v3_8_requests.tex:58–75`.

Retention through speculative commitment and discarded branches is necessary and present. The exact-parent eligible set excludes identities in both inherited and uncommitted ancestry, checks immediate application admissibility, and requires singleton fit. It is not a stale projection reused across a different parent or native state.

The primary executes first. Remaining capacity and later DATA positions use local FIFO, excluding ancestry/batch duplicates and rechecking validity after earlier batch entries. Thus a primary that changes application state does not authorize an invalid FIFO remainder. If only the primary fits, singleton fit is already sufficient; an empty remainder creates no dependency on another request. All-EMPTY witnesses permit ordinary valid batches, including newly arrived requests, without requiring their inclusion from Byzantine leaders.

The index and canonical minimum are protocol-local work. The wording expressly does not prove a deadline for work deferred outside the completed handler. I used the stated abstract handler premise rather than inferring a CPU bound from a finite pool or an index.

### 3. Same-snapshot relay and persistent ownership

> “For each authenticated installed snapshot of its region in which it is a member, it dispatches the full request to that snapshot's members.” — `rotation_v3_8_requests.tex:23–44`.

One correct member's first complete receipt immediately schedules full-byte delivery to its installed snapshot. Under that snapshot's existing complete-message bound, every correct recipient processes it within Δ. Duplicate delivery preserves the original FIFO position. The request store and outstanding relay actions survive Waiting, Ready, Active, Frozen, Completed, and Terminated, so worker exit does not silently drop this established relay obligation.

This action creates no vote, entry, Ready gate, QC, or timer. The bound is correctly limited to the dispatched within-snapshot message. It is not a bound on discovery of a future roster, dispatch of an arbitrary backlog, or communication from an old-only member to a new-only member.

### 4. Exact REPORT context and live trigger

> “A correct member emits its report in the ordinary-term entry handler for t>0 … For t=0, it emits on complete validation of the NEWVIEW in the entered generation.” — `rotation_v3_8_requests.tex:77–96`.
>
> “After a new ordinary-term entry, run the independent REPORT trigger … A QC … never changes a recorded entry time.” — `main:1166–1183`.

The context includes authenticated session/region/configuration/AUTH, generation, term, j=0, opening identity, and exact parent hash/height/physical position. A pair or publisher annotation alone is insufficient. Alternate QC encodings do not change that context. A persisted report is immutable; resending repeats it.

The ordinary trigger is tied to actual terminal-QC admission and entry, rather than receipt of the successor's first proposal. That removes a possible collection/proposal circular dependency. The term-zero trigger is tied to the fully validated opening in an entered, current, unfrozen generation. It therefore cannot select against an unknown opening. A DRAIN-only generation is outside the DATA-producing trigger.

Historical validation does not synthesize a fresh report. This is compatible with the claim being proved: a correct initializer's NEWVIEW is broadcast after its original 2Δ entry wait, so under the prepared Δ entry skew it reaches a cohort whose required entries already exist. Old historical contexts need no newly signed reports for the timing theorem.

### 5. Early, missing, late, or incomplete NEWVIEW

> “Future-generation messages wait for entry.” — `main:971–979`.
>
> “Schedule the check for p=1 at entry, even if the opening is not yet known.” — `main:1226–1234`.
>
> “After validating an admitted DATA-opening NEWVIEW, run the independent REPORT trigger … Never delay the original ANCHOR vote while gathering or sending reports.” — `main:1289–1307`.

For a future-generation NEWVIEW, the existing buffering/entry rule applies and the fresh-action gate is rechecked on callbacks. An old-generation or historical validation path cannot fabricate an emission. A missing or incomplete NEWVIEW does not postpone the initial progress check. A NEWVIEW that is too late because Quit or authenticated closure has already shut the gate cannot revive fresh signing or reset the entry clock.

For the claimed correct-initializer bound, an early-before-entry NEWVIEW is not a missing case: its send time is at least first entry+2Δ, while every prepared correct entry occurs by first entry+Δ. Complete validation triggers the report while ANCHOR processing stays independent. Consequently no extra “wait for all reports before forwarding NEWVIEW” round is hidden in the proof.

### 6. Partial contributions and collection head-of-line attacks

> “Ignore incomplete or invalid contributions; count a member at most once.” — `rotation_v3_8_requests.tex:106–120`.

At a correct leader, signature/context checks alone do not count a nonempty contribution: the full selected body must also pass its key, admission, singleton-fit, and ancestry checks. A Byzantine sender's valid signed key with a missing body therefore cannot occupy an unavoidable wait in front of correct reports. An invalid first contribution does not consume a correct member's counted slot. The leader can proceed as soon as complete valid contributions exceed F; correct members alone provide this weight.

The proof uses “all correct reports have arrived by the deadline” only as an upper-bound argument. The algorithm does not need to identify an event named “all correct reports received,” or wait for Byzantine reporters. A Byzantine reporter may choose its own valid contribution adversarially, but it cannot make the leader select an unavailable fictitious body after the specified checks.

### 7. Common validation and earlier-term voting

> “Validators do not compare the witness with their own queues, require their own report, or apply current-time/current-G tests to a historical report.” — `rotation_v3_8_requests.tex:128–148`.

A complete valid first-DATA proposal is equally processable by another correct member even if that member's local queue differs or its own report was excluded. Nonselected request bodies are not extra validation dependencies. A malicious smaller reported key must have its actual authenticated body as the first batch entry; otherwise the proposal fails the common predicate. A correct included reporter supplies the local-minimum fact used later by the potential proof.

Later DATA, DRAIN, recovery histories, and own-session receipts require the applicable first-DATA sidecars in their own-session ancestry. The original rule still votes on unseen earlier-term proposals in the current generation, including proposals in attached history and already indirectly committed ancestors. Old generations remain archival and future generations buffered. Thus normal local term advancement does not remove the matching-vote step used by the terminal-QC dissemination and protecting-QC arguments.

### 8. Evidence-first handling, witness variants, and timer identity

> “Report validation must not hide an existing admissible DATA/DRAIN or NEWVIEW equivocation.” — `rotation_v3_8_requests.tex:150–155`.
>
> “Before validating a message or discarding a duplicate, inspect all signed material in it for a quit proof.” — `main:962–968`.

An invalid or incomplete report wrapper does not shelter original signed equivocation evidence. Distinct report signer sets are witness variants of the same proposal; the proposal identity remains (β,r,g,t,j,Hash(B)). An alternate report set or QC encoding cannot create another first vote or restart a timer. Reports themselves create neither Quit evidence nor a QC rank or lock.

The ANCHOR vote is parsed independently if a report shares its packet. Therefore withholding or corrupting a report cannot suppress a valid original ANCHOR vote or original Quit proof.

### 9. Ordinary first-DATA bound: 4Δ

> “The preceding terminal proposal's complete forwarding and correct votes disseminate its QC to all correct members by e+2Δ … Every correct report … reaches a correct leader by e+3Δ.” — `rotation_v3_8_requests.tex:211–217`.

Let e be the first correct entry to the ordinary term, equivalently the first correct complete terminal-QC event for its predecessor. A correct signer of that terminal QC voted at s≤e and forwarded the full proposal. In the stipulated active/unfrozen/unclosed interval, every correct member casts or has already cast its matching vote by s+Δ. Its multicast votes give the QC everywhere by s+2Δ≤e+2Δ, independently of the successor proposal. This is precisely the independent terminal-QC part of Part I's handoff lemma (`Part I:630–657`), now applied to a complete augmented-valid proposal.

Each corresponding local term entry emits the exact-parent report, so a correct successor has sufficient complete correct reports by e+3Δ. It also has the parent/QC by then. The first complete proposal is therefore processed and voted everywhere by e+4Δ. No circular dependence on that proposal is used to establish the reports' existence. The candidate expressly abandons the sharper old ordinary first-vote 2Δ claim (`rotation_v3_8_requests.tex:239–241`).

### 10. Initial/recovery term zero: 6Δ, with no timer reset

> “Both correct-only inputs reach the leader by e+5Δ, and first DATA is voted by e+6Δ.” — `rotation_v3_8_requests.tex:219–228`.

Under the prepared Δ entry skew, the correct initializer enters by e+Δ, waits its original 2Δ, and sends NEWVIEW by e+3Δ. Complete NEWVIEW processing by e+4Δ produces independent ANCHOR votes and reports. Both inputs return by e+5Δ; the proposal reaches and is voted by all correct members by e+6Δ. In recovery, the original STATUS wait still ensures the chosen justification meets every correct entry lock. REPORT introduces no extra step in series after ANCHOR collection.

Each later fresh position adds at most 2Δ. Thus term zero's p-th vote is at most e+(2p+4)Δ, and ordinary terms satisfy the stronger e+(2p+2)Δ bound. Since e≤e_v, both meet the unchanged local deadline e_v+(2p+4)Δ. Messages precede callbacks at equality (`main:1094–1095`). The report event, resend, late request arrival, incomplete packet, and ordinary QC duplicate do not reset e_v, the original BLAME deadlines, or an already running commit timer.

### 11. Protected first commit and faulty-term progress

> “A correct initializer still proposes by e+5Δ, retains its protected commit window and forms a complete receipt by e+9Δ.” — `rotation_v3_8_requests.tex:231–241`.

Part I's protected-initializer proof continues to apply (`Part I:870–893`). Before the first proposal there is no admitted later term. A correct initializer meets term-zero's BLAME deadlines. If its first proposal is sent/voted at s, any later term's entry is at u≥s, so its earliest correct BLAME is u+6Δ≥s+6Δ. With equivocation excluded in this generation, no first freeze can interrupt the s+3Δ all-correct commit window. Thus proposal by e+5Δ, all correct commits by e+8Δ, and sufficient FINAL delivery by e+9Δ remain justified.

For a faulty term, report collection does not disable the original checks. A first correct terminal vote still forwards the entire validation bundle, giving matching correct votes/QC within the original bound unless Quit/Completion intervenes. If no correct terminal vote exists by the last-check bound, no terminal QC could have advanced anyone; correct members' matching BLAMEs force Quit. A Byzantine leader that withholds complete reports or their resulting proposal is therefore handled by the existing progress path.

### 12. Regional result bound R_i remains finite

> “D_i=(Q+1)(2K+12)Δ_i, R_i=(g_i^*+1)D_i+2g_i^*Δ_i+9Δ_i.” — `rotation_v3_8_liveness.tex:107–157`.

The finite roster and permanent exclusion yield a finite generation threshold beyond every exposed equivocator; inherited initializer fairness provides a finite correct-initializer index g_i*. Persistent fetching eventually caches the finite authority prefix through that index. Exact-anchor preparation and fixed-cut hydration also finish eventually; no common deadline over all future configurations is asserted.

For a term beginning at first correct entry u, the latest original progress check is at most u+(2K+6)Δ, allowing the ordinary 2Δ entry skew. The proof then allows 2Δ for terminal-QC dissemination or matching BLAME dissemination and up to 4Δ for endpoint commits/FINAL collection. The (2K+12)Δ allowance is conservative. At most Q DATA terms, or one DRAIN term, fit in a generation. Each recovery adds at most 2Δ from first Quit when its authority is cached. The displayed R_i consequently covers the finite prefix and the protected correct initializer window. A DRAIN at the endpoint still covers positive session DATA because the fresh retry's quota Q≥1.

The report modification changes neither these deadlines nor their finite authority/preparation argument. It does not require a new successful request-acquisition service: correct reports may all be EMPTY, and DATA progress can still occur.

### 13. Original public result and no new publication premise

> “R_+=(R_0,W_R) … The Publish Layer's input remains exactly R_0.” — `rotation_v3_8_requests.tex:157–182`.
>
> “They offer it as soon as its original dependencies are available; publication does not wait for an unrelated missing REPORT sidecar.” — `rotation_v3_8_exports.tex:201–214`.

The full regional bundle retains REPORT sidecars for regional validation, history forwarding, receipt-driven Completion, and archival ownership. The publication projection deletes only those newly separately framed sidecars. Every original required DATA, signed object, QC, receipt, ancestry, boundary, source denotation, and native evidence object remains in R_0; it is not replaced by a smaller public kernel as the service input.

Consequently the publication service's original complete-input starting event, size/load restrictions, and old-format value are preserved. The primary uses ordinary batch capacity, while REPORT bytes do not consume extra public-record headroom. No T_pub guarantee is inferred for bare requests/reports, a missing payload, or a kernel-only input. The logical-carrier preservation lemma retains its representational scope; it does not manufacture a deployed codec or fetch guarantee.

### 14. Honest FINAL and genuine contexts without g/t in FINAL

> “FINAL still binds no generation or term and does not authenticate a publisher's chosen alternate witness.” — `rotation_v3_8_requests.tex:184–191`.
>
> “A correct replica generates FINAL only after Commit's timer event … checked … every required first-DATA REPORT witness.” — `rotation_v3_8_receipt_kernel.tex:53–89`.

Raw weight->F FINAL verification under the authentic incoming fixed snapshot identifies an actual correct signature. The signing rule traces it to a preceding timer-origin Vote that fully validated its own-session history and report witnesses. Receipt-triggered commitment and Completion emit no replacement FINAL (`main:888–893`), so the argument does not circularly bootstrap a first correct signature from the receipt whose validity is being proved. Inherited-anchor kernel uses descend to earlier execution events.

This establishes existence of a genuine historical witness for the exact hash-bound DATA and its same-session ancestry. It does not authenticate a later publisher-selected generation, term, or report set. The conditional request theorem is correctly formulated using genuine witnesses, and its proof does not derive awareness or report context from unsigned publisher annotations. Other-session covering receipts do not substitute for this provenance.

### 15. Prepared public response and global DATA growth

> “L=max_i{2T_E+2Δ_i+R_i+T_pub}.” — `rotation_v3_8_liveness.tex:162–176`.
>
> “The left side is quadratic in N, while the right side is linear.” — `rotation_v3_8_liveness.tex:326–355`.

The regional result furnished by R_i still has the unchanged public projection. Persistent raw-byte retention includes rejected-record evidence; optional REPORT, alternate recovery AUTH, and native sidecars are not public-kernel fetch dependencies. The source-handler contract charges source receipt/processing and prepared entry without inserting new report work into Ready. A complete result previously recorded with bad outer hints is still inventoried from that logical carrying record. The proof does not require a corrected publication or infer a bound for retrying rejected exports.

The incoming opening and first result each contribute T_E+Δ_i, with regional response R_i and publication T_pub between them. The caller's genuine boundary, current-pair, next-height, source-reference, and Force-cutoff checks remain mandatory. Terminal DATA commits before successor AUTH acquisition, so quota one is not a blocked edge case. If Force closes the retry in the carrying block's prelude, the proof uses its closure alternative rather than accepting the closed version.

In a hypothetical fixed-G tail, zero-output retry preserves a fixed slot; finite health credits leave a finite control prefix; fixed preparation gives one finite L over installed regions. Conditional AUTO cooldown preserves an eligible owed candidate after old restrictions expire, so successive AUTO closures immediately open the next retry. Each has zero accepted output, hence remains in startup and needs at least M_n W_min additional timestamp progress, with M_n=1+β_0+n. If no DATA grows, the response lemma forces a_{n+1}−a_n≤L. Summation and the original upper-lead condition give the stated quadratic-versus-linear contradiction. No lower-lag bound or extra empty-block-rate premise is used.

### 16. Repeated accepted regional service and unchanged races

> “Only Force selection is filtered by O. REGULAR selection retains its historical NextLive rule.” — `rotation_v3_8_liveness.tex:366–389`.
>
> “Empty DATA can discharge a regional debt, so the proposition does not establish transaction inclusion.” — `rotation_v3_8_liveness.tex:440–444`.

Fixing an unfinished finite round, a persistently eligible owed region cannot be pruned. With infinitely many Forces, each productive Force-created version must first accept DATA from its selected owed session, because acceptance is contiguous and prior-version output is cut off. Every such first contribution either discharges a debt or follows a debt already pruned; only finitely many distinct removals can avoid the fixed region. Having only finitely many productive versions would contradict continuing global DATA growth. With finitely many Forces, unbounded accepted DATA entails infinitely many finite-quota regular sessions; their strictly increasing opening heights eventually pass the region's permanent eligibility position and any old cooldown. Cyclic NextLive then reaches it. The same finite-round argument admits a newly permanent-ON region into a later round and repeats after service.

REPORT does not alter the round ledger, pre-payload snapshots, or selectors. Independent controls still precede matching MSIG, matching MSIG precedes AUTO, and Force keeps exactly pre-payload G. A terminal result accepted first opens its ordinary boundary; a Force in the same containing block acts before payload and rejects the old-version result. Zero-output Force keeps r; positive-output Force targets r+1; an unassigned boundary keeps its stored target. Late sidecars or regional receipts do not change these races or reopen a closed version.

### 17. Conditional request inclusion: exact decrease of a finite potential

> “Suppose infinitely many accepted term-first DATA have a genuine report witness containing a correct signer whose eligible retained set contained x at its report event.” — `rotation_v3_8_requests.tex:243–250`.
>
> “Let Φ_x(G) count their distinct logical identities absent from G.” — `rotation_v3_8_requests.tex:253–271`.

Assume x never enters G. At each counted accepted first-DATA event, take the genuine aware witness required by the theorem. Its correct reporter selected y with key(y)≤key(x). The common predicate makes the witness minimum an actual first executed request z with key(z)≤key(y), whose identity is absent from its exact parent ancestry. If x were in that accepted ancestry, contiguous acceptance would already have included x, contrary to the assumption.

At the acceptance event, the exact parent ancestry is the preceding global prefix, including ancestors accepted earlier in the same EXPORT. Thus z has an identity newly entering G at that event. Bounded canonical byte length gives only finitely many canonical strings with key at most key(x), hence only finitely many corresponding logical identities. The potential counting those identities absent from G is finite, monotone nonincreasing, and strictly decreases at every counted event while x remains absent. Infinitely many such decreases are impossible.

This argument is unaffected by changed parent, recovery generation, term, session, roster, or branch: each witness is checked in its own authentic context and G never rolls back. A discarded speculative occurrence does not decrease the potential; an accepted occurrence prevents reusing that identity as a later primary because later accepted ancestry contains it. Retransmissions, alternate transport/authentication-proof encodings, duplicate deliveries, receipt signer variants, and duplicate exports do not create a fresh logical identity or a second accepted DATA count. No local FIFO ordering agreement is used.

### 18. Immutable EMPTY reports, speculative ancestry, and the explicit open boundary

> “A late request can leave an old EMPTY certificate usable for its exact term, but that context has only finitely many DATA positions.” — `rotation_v3_8_requests.tex:283–289`.
>
> “The full request-liveness goal remains open even if the conditional REPORT proof passes review.” — `rotation_v3_8_liveness.tex:466–495`.

A request arriving after immutable reports does not retrospectively change them, and the protocol does not claim otherwise. One old context can authorize only finitely many DATA positions. Unbounded accepted DATA therefore requires infinitely many genuine term-first ancestors; changing their displayed context cannot turn one accepted block into multiple events. Unaccepted speculative paths cannot be skipped to claim a decrease of the global potential.

This finite-context observation is not itself a cross-roster dissemination theorem. The candidate's stated adversarial order remains possible under unqualified pointwise eventual delivery: each disjoint roster first emits EMPTY reports, later receives/relays x while still finishing its service, and hands off to another roster that has not yet received x. Old owners preserve x but their within-snapshot relay does not deliver it to the new-only members. Repeating that order need not yield the infinitely many aware accepted witnesses required by the conditional theorem.

I therefore classify arbitrary-roster awareness as an acknowledged unresolved objective, not a counterexample to the conditional implication. The proof does not silently replace that missing premise with repeated regional DATA service, “every certificate has a correct signer,” or the existence of a correct leader somewhere in a schedule.

### 19. Retirement compatibility and retained sidecars

> “Before any … INPUT_REPORT … require that the manager has no authenticated Closed[β] …” — `main:1066–1075`.
>
> “required regional REPORT sidecars … pending-request relays … [owners] cannot … issue fresh … INPUT_REPORT signatures” — `rotation_v3_8_retirement.tex:130–152`.

Fresh reports share the same closure gate as fresh votes and entry. A late callback cannot reopen it. Already existing reports may be retained and relayed, and pending request transport remains a persistent data duty. FinishClose transfers all report sidecars needed for regional validation and existing timer/evidence obligations before the worker exits; erasing the public sidecar is not permission to delete its last required regional copy.

The timing/progress claims reviewed above expressly concern an uninterrupted active interval. They do not assert complete-receipt formation for every surviving late timer after closure. Thus the report extension does not revive the universal protecting-QC claim that retirement itself disclaims. Within the unclosed vote window, the common augmented predicate and complete forwarding preserve the original matching-vote argument; late-window retirement uses its separate no-later-entry reasoning.

### 20. Final frozen-snapshot check

All top-level manifest entries and all four nested baseline entries were rehashed after this review. No snapshot/protocol file was edited. The main SHA-256 remains `c8611baf07c74f34512a182b0b3ace81d6f8f3cf65fb0cb567af696ec8a73574`.
