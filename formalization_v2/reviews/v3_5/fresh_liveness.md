# Fresh independent liveness review of v3.5

## Summary

**Final Verdict:** The liveness argument is viable but incomplete because the prepared-public-response proposition has two **Justification Gaps**; the supplied proof does not yet establish its numerical bound, and the global-progress and repeated-service proofs depend on that bound. I found no independent Critical Error establishing that either of those two eventual-progress theorems is false. The proposed execution excluding a pending request from global DATA is valid under the stated model and does not contradict the narrower DATA/service conclusions.

**List of Findings:**

- **L1 — Justification Gap.** Location: `sections/rotation_v3_5_liveness.tex:183–193`, particularly “The Publish Layer and Espresso bounds add … Submit the result with the exact current base.” The frozen service assumption explicitly allows an already recorded result to discharge publication service and leaves acceptance retries separate (`sync_hotstuff_rr_part2_rotation_v3_5.tex:405–420`). The proof needs a bounded correctly based EXPORT, rather than only bounded delivery of the result. It does not discharge this exception when a third party records the same complete result first inside a rejected EXPORT. The leader-first implementation discussion is relevant, but the proof does not establish the required obligation for every earlier-completion outcome covered by the regional proposition.
- **L2 — Justification Gap.** Location: `sections/rotation_v3_5_liveness.tex:86–93,161–193`, together with `sections/rotation_v3_5_control.tex:111–125` and `sync_hotstuff_rr_part2_rotation_v3_5.tex:1457–1475,2226–2233`. A terminal next result, necessarily the first positive result when the quota is one, invokes the next public boundary and `PrepareOwed` under `AUTH_(r*+1,0)`. The finite preparation set explicitly proved available contains the fixed slot's recovery authorities and anchor, not this next-slot authority. The text does not resolve this dependency against atomic EXPORT acceptance and its canonical wait on missing boundary authority. Consequently the stated constant acceptance bound has not covered this case.

The local regional response calculation, finite health-control results, window arithmetic, upper-lead clock contradiction, and owed-set argument are checked below. The last three establish their intended conclusions **conditional on** the missing prepared-public-response step. The native executable-state/complete-handler contract is an explicit scope condition; these results do not certify an arbitrary database implementation.

### Frozen input and review scope

All locations below are relative to `formalization_v2/reviews/v3_5/snapshot/`. Main source SHA256:

`2f47bca40d7ebdf80bb68a49a5fb8c479c4a50484a753d0134defc9381831d6b`.

The outer manifest's 20 file hashes and the nested baseline manifest's four source hashes all match. The complete Model and assumptions section is byte-identical to v3.1. I independently read the full active main source and all six included protocol fragments, the complete inherited Part I source, the full baseline comparison including baseline-only passages outside Model, and the included figure sources. No other review or previous verdict was read. No snapshot file was changed. This is a manual mathematical/protocol review, not executable or model-checking certification.

The frozen baseline matters beyond Model: `baseline/sync_hotstuff_rr_part2_rotation_v3_1.tex:2103–2119` retains the original regional fair-schedule/exclusion/commit-window conditions, distinguishes transport from acceptance, and explicitly notes that v3.1's “other region” selector can stall with one eligible region. Candidate same-region renewal is therefore a substantive protocol change, not a consequence of Model byte equality.

### Dependency map

| Step | Dependencies | Status in this review |
|---|---|---|
| Stable stalled slot and retained preparation | Zero/positive accepted-count rule; persistent native cache and authority cursor | Verified locally |
| Finite exclusion generation and correct initializer | Finite installed roster; inherited generation-index exclusion and initializer fairness | Verified locally |
| Actual regional response within `R_i` | Current-pair entry, complete regional messages, finite cached authority prefix, original DATA/DRAIN and BLAME rules, no external closure | Verified at the stated abstract-handler scope |
| Opening block to first Enter | Source visibility, immediate source relay, retained replay cursor, already executable state, ordered atomic handler | Verified at the stated scope |
| Complete result to accepted next DATA or closure within `L` | Correct-base publication within a service bound; complete acceptance/boundary dependencies | **L1 and L2 unresolved** |
| Finite committee interference at a fixed `G` | Durable credits, finite prior journal, immutable control semantics | Verified locally |
| Eventual immediate AUTO reassignment | Stable authority/governance/owed set, finite old cooldowns, conditional new cooldown | Verified locally |
| Continuing global DATA | Prepared public response plus increasing miss tolerance and timestamp upper-lead inequality | Unverified downstream of L1/L2; algebra checked conditionally |
| Repeated accepted regional service | Continuing global DATA, finite owed round, strict cutoff, advancing regular selection heights | Unverified downstream of global progress; combinatorial argument checked conditionally |
| Failure of a full request-inclusion conclusion | Positive empty DATA per slot, eventual but unbounded new-slot recovery authority, legal votes/timers straddling AUTO | Verified counterexample to the stronger request target |

## Detailed Verification Log

### 1. Inherited quantifiers and the stable-slot reduction

> “A name in a schedule is not evidence that its turn has entered or its DATA has become global.”

`sections/rotation_v3_5_liveness.tex:11–36` correctly separates catalog fairness from actual entry. Part I gives `forall g_0 exists finite g >= g_0` with a correct initializer, and `forall fixed g,t_0 exists finite t >= t_0` with a correct term leader (`baseline/sync_hotstuff_rr_part1_core.tex:208–223`). Its exclusion threshold is a generation index, not an elapsed time or a new promise for each beta (`baseline/sync_hotstuff_rr_part1_core.tex:185–204`). The candidate also retains the separate actual-entered-turn condition at main source `517–536`.

The new finite-response proof does not infer actual entry from either schedule quantifier. Its term/generation exhaustion argument supplies actual entry unless a complete positive result already forms. Term fairness by itself is insufficient in a finite-quota session; the proof appropriately uses an eventual correct initializer and the finite generation prefix.

> “After it, the zero-output rule keeps a fixed slot.”

Liveness `38–51` follows from main `1844–1876` and control `235–255`: if accepted `G` stops changing, only an already productive current duty can cause one final positive-count slot advance. Later duties have zero accepted count and retain the same numeric slot, even if physical regions change. An unassigned boundary retains its designated target. All these retries have fresh beta-bound locks, votes, timers and Quit admission proofs. The immutable `(r,g,t)` schedule is not reseeded by beta.

The shared native cache and sequential authority fetch cursor are concrete rules at main `1803–1819`. Retaining unfinished work matters: merely caching completed work while canceling every pending fetch on Force would not establish the proposition. The proof only takes finite maxima for one installed slot and its finite region cycle; it does not require a finite union over all future configurations.

### 2. Finite authority prefix and actual regional response

> “The finite roster makes the maximum finite; inherited initializer fairness makes `g_i^*` finite.”

Liveness `67–84` uses the finite set of identities ever exposed in this fixed namespace. Each has a finite permanent exclusion threshold. Above their finite maximum no admissible exposed leader equivocation can occur: such a signer would itself be in the exposed set and forbidden by its threshold. A proof for an older generation or another beta cannot freeze the current generation. The quantifier is over generation indices, and the argument remains valid even if the same low-generation faulty assignment recurs in many fresh betas. This is the same finite-union argument proved in Part I `852–868`.

> “Eventual authority availability and the persistent fetching task make all its records available after a finite time.”

For `AUTH_(r*,g)`, `0 <= g <= g_i^*`, the set is finite. Eventual availability plus a fetch cursor that does not restart gives finite completion of this set. Taking the maximum with eventual restoration of one exact fixed anchor adds no uniform authority or restoration-delay assumption. The native object must already be executable, as required by main `1803–1819` and ReadyBundle `25–56,99–105,124–146`; an arbitrary serialized database is not enough. This verifies the stated fixed-slot preparation, subject to L2 when it is later used for terminal global acceptance.

> `D_i=(Q+1)(2K+12)Delta_i`, `R_i=(g_i^*+1)D_i+2g_i^*Delta_i+9Delta_i`.

Liveness `95–141` is conservative under its explicit no-external-closure condition:

1. Initial correct entry skew is at most `Delta` under the complete ReadyBundle handler. Cached recovery authorities remove authority skew, so Quit propagation plus the original wait gives at most `Delta` skew for a recovery. Ordinary term-entry skew is at most `2Delta` by Part I's terminal-QC handoff argument (`630–657`).
2. If `u` is first correct term entry, all correct last checks occur by `U=u+(2K+6)Delta`: the per-member last deadline is `e_v+(2q+4)Delta`, `q <= K`, and term-entry skew is at most `2Delta`. Main `1197–1205` explicitly schedules the first check even before the opening is known and runs overdue later checks immediately. Thus a silent initializer cannot postpone BLAME by withholding its opening.
3. If a correct terminal vote exists by `U`, its complete forward and the correct members' matching votes produce a QC within another `2Delta`, unless Quit or an already existing complete result intervenes. Below the endpoint this advances the term. At the endpoint, surviving timers and one FINAL-delivery round give a complete positive result within the remaining allowance. Completion is already a complete-result outcome, not a reason to wait for more signatures.
4. If no correct terminal vote exists by `U`, no correct member has advanced on its terminal QC, since every QC contains a correct signer. Correct members that remain in the term have too few votes at their last checks. Their common BLAME statement and correct-alone quorum form/propagate a Quit within the extra allowance. Different openings or signed conflicting positions provide an earlier Quit, not an indefinite blockage.
5. Hence `(2K+12)Delta` safely bounds a term's advance, Quit or complete-result outcome. A DATA generation has at most `Q` terms because every term adds at least one DATA height; a DRAIN generation has one term. `D_i` overcounts these possibilities and is therefore conservative.
6. Each inter-generation transition with cached authority costs at most `2Delta` from the first correct Quit. The finite prefix reaches the correct initializer `g_i^*`, unless an earlier complete positive result has already formed. This establishes actual entry; it does not assume it from fairness.
7. At `g_i^*`, no admissible equivocation can interrupt the generation. Part I's protected-initializer argument (`870–894`) supplies the first fresh proposal by `e+5Delta`, all correct commitments by `e+8Delta`, and a full FINAL receipt by `e+9Delta`. A later faulty leader's earliest BLAME is too late to destroy this initial commit window. If the fresh block is DRAIN, it commits terminal DATA after the session's original anchor. Since `Q >= 1`, the result still contains positive DATA.

The displayed `R_i` has excess slack; I found no counterexample to these regional constants using the specified regional rules, cached finite authority prefix and complete-handler interpretation. This checks regional result formation/offering, not acceptance of the result by the external interpreter.

### 3. Opening source input to Ready/Enter

> “They give first Ready/Enter by `a+T_E+Delta_i`.”

Liveness `161–181` and source-step `127–169` explicitly use continuously online participants with maintained predecessor cursors. All headers at or before the opening finalized no later than `a`; their complete source evidence is available by `a+T_E`. Applicable controls are available before crossings by advance announcement. The handler sends the source packet before waiting for its own replay (`source_step:78–86`), drains an ordered complete prefix on receive, and invokes Manager and initial Enter in the same dependency-complete event (`88–108`). Thus one regional `Delta_i`, rather than a new network round per prior height, is justified by the abstract complete-processing contract.

ReadyBundle `60–96,107–146` separately ensures that the first correct actual entrant sends a complete executable-state bundle immediately. It supplies all other correct entries by one more `Delta_i` unless valid closure prevents them. Main `2038–2044` requires the opening header, not a post-opening empty block. The proof therefore introduces no hidden block-generation rate or lower timestamp-lag assumption.

This verification relies on the stated logical state-object adoption contract. It gives no bound on an external database import, storage copy, arbitrary offline catch-up or native adapter that still has residual installation work. The candidate expressly withholds those claims; I do not classify that explicit implementation limitation as a new proof defect.

### 4. L1: publication of the result does not yet prove timely publication of the accepting record

> “The Publish Layer and Espresso bounds add `T_pub+T_E`. Submit the result with the exact current base `G*`…”

**Location:** Liveness `183–193`; compare main `405–420,475–489,1406–1431,2641–2661`.

**Classification: Justification Gap.** The fixed-base argument correctly eliminates an honest sender's race with an actual accepted predecessor: if another accepted EXPORT changes `G*`, progress has already happened. It does not eliminate the different event expressly covered by the assumption: a result already appears as raw Espresso bytes in an EXPORT that does not enter `G`.

A result is DATA plus receipt/ancestry, whereas an EXPORT additionally chooses the expected global base and the interval (`338–343,1412–1419`). Therefore the same valid result can already be recorded in an EXPORT with an incorrect expected base. Main `413–414` then treats the result as delivered, even though main `1437–1438,1477–1480` rejects the record and never revisits it. The fixed `G*` observation does not make those earlier bytes accepted.

An admissible test prefix exposes the missing obligation. Use a Byzantine proposer and a quota-one session. All correct voters commit the valid terminal DATA normally. Deliver the required FINAL signatures quickly to the Byzantine collector and more slowly, still within `Delta`, to the correct collectors. The collector can submit the complete valid result first in an Espresso-admissible record with a wrong expected base. Espresso finalizes that record before a correct collector obtains the complete receipt. All correct collectors subsequently offer the correct result as specified, but the delivery assumption's already-recorded exception has already been satisfied. The engine has Completed, so the regional proof's early-Completion branch does not force a later correct initializer to produce a different result. The next session may still be restoring its different anchor state. A later correct-base retry remains necessary. This prefix uses no missing correct quorum, signature forgery, indefinite authority failure, or violation of a regional message bound.

This is a counter-schedule to the *service-bound inference*, not a claimed complete execution refuting the eventual theorem. In particular, the entire Model still includes an acceptance-during-service condition that the candidate explicitly declines to use as its proof premise (`517–541`; liveness `5–9`). Importing that condition would not verify the candidate's claimed derivation.

The implementation discussion cannot be ignored: main `2610–2621` prescribes immediate submission by the proposing leader and says implementation backup delays/retries must fit in `T_pub`. For a correct proposer and an actually issued exact-base EXPORT, that path suffices. However, the regional proposition also stops at an earlier result/Completion produced by a Byzantine proposer, and correct replicas are required to **offer** that result, not themselves to make a newly bounded acceptance retry. Section 10 explicitly describes an implementation of the preceding abstract service and disclaims a proof of its bound (`2603–2607`). The proof needs to connect these sentences to the already-recorded case and identify the exact bounded obligation it invokes. I cannot silently treat the sketch as a stronger replacement for the frozen service assumption.

Downstream effect: the numerical prepared-public-response proposition is unverified. Assuming its conclusion temporarily, the later clock and owed-set arguments remain checkable and are checked below.

### 5. L2: terminal acceptance can introduce an unprepared next-slot authority

> “At a new public duty boundary/selection invoke PrepareOwed with the designated target authority … Commit the whole update only if it passes.”

**Location:** Main `2226–2233`, main `1457–1475`, control `111–125`; compare liveness `86–93,161–193`.

**Classification: Justification Gap.** The prepared regional proposition caches the authorities through `g_i^*` at `r*`, and state/provenance for the fixed retained anchor. But terminal DATA cancels the current duty and reaches a normal boundary for `r*+1`. `PrepareOwed` explicitly obtains `AUTH_(r*+1,0)` and its complete evidence before pruning/refilling debt; missing evidence waits at this canonical transition. Source-step `38–45` also distinguishes a genuinely missing canonical-boundary authority from a malformed record that may simply be rejected.

Take the allowed quota `Q=1`. The first positive regional result is terminal, so the public-response proof cannot avoid this case by exporting a nonterminal positive prefix. All fixed-slot authorities and the exact opening state can be prepared, while `AUTH_(r*+1,0)` is available only at an arbitrarily later finite time. The frozen model permits this eventual authority delay. P6 then invokes a transition whose prerequisite is absent, despite the fixed-slot preparation used to justify `L` being complete.

Main `1465–1475` says terminal DATA may enter `G` when no successor is eligible, but expressly distinguishes known absence from missing authority. It does not state an independently committed terminal append followed by a deferred `PrepareOwed` transition that preserves P6's atomic semantics. Conversely, if next-slot authority is considered part of the complete terminal EXPORT's validation dependencies, the preceding regional `R_i` argument has not bounded its availability. Either reading needs a stated dependency argument before the claimed public bound follows.

This is not an infinite authority-withholding attack, nor a demonstrated false eventual theorem. The missing record is one finite next-slot object and can eventually arrive. The issue is that the proof's explicit finite preparation set and acceptance transition do not yet match. I do not strengthen the model with a bound on its arrival or supply a repair.

### 6. Finite health controls and restoration

> “Only a newly observed accepted tip provides a new kappa; only newly accepted DATA from i advances d_i.”

Health `18–45,120–143` correctly ties new credits to a monotone accepted-prefix cursor. At a fixed actual tip of height `h*`, the observed tips from `h_0` through `h*` give at most `h*-h_0+1` additional keys. One durable reservation per key and no refunds give the stated bound, including stale targets and uncertain submissions. Previously reserved MSIGs are correctly accounted for by `N_0`. This assumes a finite preexisting journal, as stated, not one new budget per Force.

Health `60–98,145–169` also verifies restoration: ON is independently submitted/finalized first and has a later finite activation threshold than OFF; all same-block independent controls apply by threshold order. New health MSIGs have empty attached configuration, so they cannot overwrite that ON in the later MSIG phase. A region with only finitely many accepted DATA counts has only finitely many per-region lease credits, even when other regions advance forever. No refund or overlapping lease creates extra OFFs. Growing Espresso timestamps cross the final ON threshold eventually.

The adoption/permanent-administration exceptions at health `100–118` are explicit. Unlimited outside-policy interventions would not satisfy the claimed finite-intervention premise and are not valid counterexamples to this policy result.

### 7. Adaptive windows, cooldowns and the real/Espresso clock argument

> “There is no AUTO-created unassigned interval that could replenish arbitrary real-time clock lag.”

Conditionally on the prepared-public-response proposition, liveness `206–257` is sound. Once `G` is fixed, the slot, installed parameters, governance and authority eligibility stabilize after the finite health prefix. Old finite cooldown thresholds are eventually crossed. The owed set is finite and no accepted DATA changes it in the stalled tail.

The conditional AUTO cooldown rule (`control:161–170`) checks alternatives before setting the new cooldown. If the source is the last eligible owed member it is not newly cooled; if it is outside the owed set, an eligible owed alternative is already present after the finite old-cooldown prefix. Consequently every later AUTO both closes and opens a retry, with no intervening unassigned interval. An existing OFF or cooldown is not silently cleared. This part genuinely permits one eligible region under the regional model.

For a zero-output duty every settled base window fails and remains in startup (`control:45–72`). A timestamp jump can settle multiple failures but creates one Force, opens the replacement at the realized timestamp and does not settle it again in that prelude (`268–276`). Thus the displayed timestamp increment is at least `M_n W_n`, and `M_n=1+beta_0+n`. Fixed-slot positivity gives one `W_min>0`.

Assuming the public-response bound and no global growth, each closure occurs by `a_n+L`; immediate reassignment makes it the next opening. Therefore

`theta_N >= theta_0 + W_min [N(1+beta_0)+N(N-1)/2]`,

while the frozen upper-lead condition gives

`theta_N <= a_N+epsilon_E <= a_0+NL+epsilon_E`.

The contradiction is quadratic versus linear. There is no reversal of the upper-lead inequality and no hidden use of a lower timestamp-lag bound or an empty-block production rate. Finite initial lag changes only the constant. The unresolved point is precisely the premise `a_(n+1)-a_n <= L`, inherited from L1/L2, not this algebra.

The scope clarification at liveness `259–265` is material. Every region included in the response maximum must satisfy its installed Byzantine-weight and communication premises. “At least one eligible region” does not license assigning permanently unavailable correct quorums elsewhere while pretending those snapshots satisfy the model. A physically healthy region permanently OFF is also outside the eligible-region conclusion. No such assumption-violating trace is used here.

### 8. Repeated accepted regional service

> “Only Force selection is filtered by O. REGULAR selection retains its historical NextLive rule.”

Liveness `267–345` respects the actual selector distinction. For an unfinished round containing permanent debt `C`, every other member can be discharged or pruned only once, and new ON events do not join that unfinished round.

In the infinite-Force case, the first accepted DATA in a productive Force-created version must belong to its initially selected owed region: strict old-version cutoff and contiguous acceptance prohibit bypassing that forced session. Each such productive version causes one of the finite round removals, possibly already caused by pruning since selection. If `C` remains owed, infinitely many productive versions are impossible. Infinitely many Forces with only finitely many productive versions would give a globally stalled tail, contradicting the global theorem **if established**.

In the finite-Force case, unbounded accepted DATA and finite per-session quotas imply infinitely many regular sessions. Their strictly increasing opening/selection heights eventually pass `C`'s permanent-eligibility position and old cooldown expiry. A forward cyclic `NextLive` scan cannot repeatedly pass an eligible `C`; other regions' eligibility changes do not change that property. Contiguous acceptance must service its selected session before passing it. The same reasoning ends an older unfinished round, so a newly permanent-ON region eventually joins a new one. It then applies again after every contribution.

This combinatorial proof is verified conditional on global progress. Empty DATA discharges debt, exactly as control `38–41,103–109` specifies. It supplies regional accepted output, not request inclusion.

### 9. The proposed request-liveness counterexample is admissible

> “A correct recovery initializer actually enters and proposes the oldest pending request x … its DATA votes precede the AUTO crossing while its complete FINAL receipt follows it.”

Liveness `347–383` can be instantiated without violating the frozen regional assumptions:

- Use three unit-weight regional members, two correct and one Byzantine, `F=1`, `Delta=1`, quota `Q=2`, and `K=2`. The initial generation's Byzantine leader supplies valid empty DATA at its first position, lets it commit and enter `G`, then withholds its second position. Correct voters remain online; BLAME freezes the generation normally.
- Choose startup target one and positive startup/running window lengths large enough for that first empty DATA to succeed before the recovery wait. For example, startup length 20 and running length 10 leave ample room. In version beta, the final failed running-window crossing occurs after `M=1+beta` failed windows. Initial authority for the next slot can already be available.
- Delay only the current new slot's `AUTH_(r,1)` until a finite time `d` chosen so that the final tolerated crossing is `d+5.5`. No fixed authority-delay bound is imposed by the model. Both correct members have already finished their Quit waits and enter recovery at `d` when its authority becomes available.
- The correct recovery initializer sends NEWVIEW at `d+2`. Deliver it to the other correct member at `d+3`; its ANCHOR vote reaches the initializer at `d+4`. The initializer then proposes the oldest pending request `x` and self-votes at `d+4`; the other correct member votes at `d+5`. All these delays are at most one.
- Their original timers expire at `d+6` and `d+7`. With Byzantine FINAL withheld, a complete receipt exists no earlier than the later correct FINAL. AUTO at `d+5.5` therefore precedes any complete receipt. Both correct DATA votes have already occurred. Closure suppresses new actions but does not cancel these timers, so a full late receipt can still form and be published within the ordinary service bounds.
- The earlier empty DATA made this duty's accepted count positive. The Force target is therefore the next slot `r+1`, not a cached zero-output retry. Its newly required recovery authority can be delayed independently, again only to a finite time. With a single eligible owed region the conditional cooldown permits self-renewal; a cyclic multi-region version also works.

Take `x` to have reached all correct participants before this sequence and to have no older pending request. Byzantine leaders may omit it from otherwise application-valid empty DATA. Every correct recovery initializer actually enters and proposes it, satisfying actual correct-turn conditions. Schedule every sufficiently high generation/term fairly; no equivocation is needed, so permanent-exclusion premises hold vacuously. The correct quorum is continuously present. Espresso can use timestamps equal to real finalization times, obeying the upper-lead bound. All complete results are offered and published on time, but late old-version EXPORTs fail the unchanged strict pre-payload cutoff.

Main `2260–2263` keeps `x` pending until global inclusion, even if it commits in an old regional suffix. Thus repeating the execution gives unbounded accepted empty DATA and repeated regional debt discharge while `x` never enters `G`. This is a valid failure of the stronger request-liveness target. It confirms the candidate's explicit limitation; it is not a Critical Error in its narrower DATA/service statements.

### 10. Verification boundary

The identified gaps are confined to the bridge from prepared local response to bounded global acceptance. I have not replaced that bridge with a new retry rule, supplied a new authority-preparation lemma, strengthened delay/fairness assumptions, or relaxed strict Force cutoff. Conditional regional response, health-policy finiteness/restoration and the independent request counterexample remain verified at the stated abstract level. Global DATA progress and repeated accepted regional service remain unverified pending the two recorded dependencies.
