# Fresh safety review of the v3.3 immutable snapshot

## Summary

**Final Verdict:** The retirement proof is viable but incomplete because of one material `Justification Gap`: closure-based prohibition of an unentered member does not have a fully specified regional propagation path. Under the literal silent-termination path, the early protecting-QC inference is false and admits the regional safety trace below. The independent global-prefix, program-isolation and selection arguments pass at their stated conditional interface scope. This is **not** an independent certification of the baseline DATA/DRAIN engine, an unconditional native-adapter safety claim, or a claim that the complete cross-region progress goal has been achieved.

**List of Findings:**

- **J1 — Unentered closure is absent from the propagated-close invariant.** Location: “For an entered program … issue … RequestClose” (`M:1667–1671`), “For an unentered program … records Terminated” (`M:2066–2069`), versus “If s+Δ<c … Every correct member therefore has cast, or now casts, the matching … vote” (`R:211–219`). The proof's c is the first correct RequestClose, but a correct unentered member can have already stopped without this event. The conditional Ready timing statement explicitly excludes intervening closure (`B:103–117`). An unspecified “Notify regional nodes” (`M:2274`) does not establish an all-members complete-message multicast obligation for this cancellation path. Classification: material `Justification Gap` because notification/RequestClose scope is ambiguous; under the literal direct-termination interpretation, the inference is a `Critical Error` and the trace below breaks local committed-log safety.

The indispensable scope qualifications are:

- Regional receipt and application validation are assumed sound in the frozen model (`M:498–504`). The retirement proposition additionally names baseline regional validation/DATA/DRAIN soundness and the complete-message Ready/Enter contract (`R:155–164`). I checked the retirement extension's timer and lock reasoning against those premises; I did not replace the baseline assumption with an independent full reproof.
- The Ready timing implication is conditional on adoption and Enter actually being the bounded complete-message handler (`B:103–140`). A native adapter with residual installation delay does not obtain the needed entry bound from sound state proofs. The source explicitly acknowledges this limitation, so it is not a concealed proof assumption or a false unconditional theorem.
- The broader progress goal remains open in the snapshot itself (`M:2675–2758`). Neither the independent passes nor resolution of J1 would establish that goal.

The work was reviewer-only. I read the snapshot's sources, Part I and frozen v3.1 baseline, but no previous, preflight, or other current review report. No protocol source was modified. Verification was manual, with hash and exact-text checks; no executable model check or implementation test was performed.

## Snapshot and evidence boundary

All paths below are relative to `formalization_v2/reviews/v3_3/snapshot/`. Every listed SHA-256 was recomputed and matched `manifest.json`.

| Alias | Source | SHA-256 |
|---|---|---|
| M | `sync_hotstuff_rr_part2_rotation_v3_3.tex` | `e69ba625229741039d704943d297830476242dadddbf473ae8cb3375f24b4f88` |
| R | `sections/rotation_v3_3_retirement.tex` | `e024b736ad043c4912d8ae0d2f15072e3cd0cf2f0f1f42e35897971568d1bb58` |
| B | `sections/rotation_v3_3_ready_bundle.tex` | `b5347497e6b925c15cd22bd78285c36fbf31ec1ff3d2f20fda3a770295f89858` |
| I | `sync_hotstuff_rr_part1_core.tex` | `00533f01567a90cd034e6c3869dfa561b7c5cf476312fc0ce8313b7060f654d3` |
| F | `revisions/v3_1_frozen/sync_hotstuff_rr_part2_rotation_v3_1.tex` | `483bdfbf3668a0f892af38df264655e54e0dbd73a1c26fbd45111d1730cec034` |
| A | `revisions/v3_1_frozen/model_and_assumptions.tex` | `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387` |

The model extracted from M, from `\section{Model and assumptions}` through the byte before `\section{Regional engine: definitions}`, is byte-identical to A and to the corresponding embedded section in F. Its 16,456 bytes hash to A's SHA above. Both proof-relevant input fragments are included at `M:1952` and `M:2086`.

## Dependency map

1. Fixed source histories and controls (`M:372–392, 431–496`), immutable authorities (`M:243–257`), authenticated signatures/hashes and the regional fault threshold (`M:348–366`) supply the public and regional trust roots.
2. Baseline complete validation, admission evidence, evidence-first processing, and current-generation vote processing (`M:889–945`) supply the regional operational rules. Baseline DATA/DRAIN validation soundness remains an explicit premise, as in F's qualification at `F:2097–2101`.
3. Complete ReadyBundle relay and atomic state adoption (`B:60–117`) give first-generation entry availability only under their stated handler interpretation. This replaces use of an unexplained initial-entry skew.
4. Complete quit relay, mandatory recovery waits, complete receipt multicast, and receipt-triggered Completion (`M:1143–1154, 1201–1230`; `R:11–28`) support the retirement proof's cancellation arguments.
5. Early-window protecting votes and late-window exclusion of recovery (`R:211–237`) support the adapted ranked-QC argument. The baseline proof templates are one generation/one chain (`I:735–747`), protecting timer (`I:749–776`), recovery locks (`I:778–793`), and ranked descent (`I:795–815`). The universal protecting-QC conclusion of Part I is **not** imported after close.
6. Historical handoff verification plus separate current activation (`M:1838–1944`) supplies unique fixed local entry; canonical prelude and atomic EXPORT checks (`M:1418–1456, 2171–2209, 2264–2287`) supply the global-prefix induction.

## Detailed Verification Log

### 1. Historical evidence remains valid without granting current entry

> “It authenticates what the boundary established; it does not require its version to remain open now and grants no entry permission.” (`M:1840–1846`)

The REGULAR case recursively validates the predecessor in the same historical mode (`M:1853–1859`). Force derives the retained anchor, old/new versions and source context from canonical replay (`M:1860–1878`). Thus a new Force can inherit old-session DATA without requiring that DATA's producing session to be currently enterable. The original producer and direct/covering receipt remain attached (`M:1881–1886`; `M:873–887`).

> “All initial entry paths use both predicates.” (`M:1904–1906`)

`CanEnter` separately requires the current open version, applicable fixed boundary, selected region and absence of prior entry, close or exit (`M:1894–1902`). A late old receipt can still validate retained evidence but cannot reopen an old engine or append an old suffix. Cache misses follow the same historical procedure (`M:1907–1911`); validity is not predicated on having cached the pre-Force verdict.

Conditional pass: the interface correctly separates proof of an old fact from permission for a new action. No counterexample based merely on learning a valid receipt after Force survives these checks.

### 2. Fixed pair, exact anchor, callback routing and local races

> “The fixed handoff cannot [change].” (`M:1744–1753`)

The fixed fields include pair, region, anchor, source context, selection and authority. The inherited anchor is a producing-pair/height/hash object, not merely a DATA height (`M:1792–1797, 1823–1834`). Material callbacks carry both their target and requested anchor (`M:1615–1617, 1634–1641, 2035–2042`). Ready checks state against this exact anchor (`M:1930–1938`).

The manager serializes control changes and rechecks the entry flag and public validity in the same event (`M:1599–1601, 1672–1682`). The durable flag is installed before signatures (`M:1960–1967`). Initial-entry, authority-arrival, recovery, terminal-QC and buffered-message paths share the close gate (`M:1033–1044`; `R:48–55`). This covers the material-available/Force race and duplicate callbacks without requiring an additional session identifier.

The one-handoff argument is conditional on regional uniqueness of the terminal DATA, including its signed selection reference (`M:2577–2587`). It does not establish that uniqueness from receipt signatures alone. Same-region renewal preserves isolation because it creates a new pair (`M:1768–1774`).

Conditional pass. The proof correctly limits authorization to the replica's processed public prefix (`M:2603–2609`); it does not falsely promise simultaneous real-time observation of Force.

### 3. Mandatory receipt dissemination and the Completion case

> “A terminal receipt is multicast no later than its local Completion event.” (`R:14–25`)

This obligation is present in the normal receipt rule (`M:824–832`), manager routing (`M:1623–1632`), Completion itself (`M:1143–1154`), and the retirement contract. It survives completed/archival ownership and deduplication. A holder cannot Complete privately while leaving peers dependent on collection of individual FINALs.

For a chosen direct timer voted at s and expiring at s+2Δ, Completion at b≤s+Δ relays the complete terminal receipt by b; the committer processes it by s+2Δ and cancels that timer. Equality is covered by message-first processing (`M:1060–1061`; `R:75–77`). The argument applies to DATA and DRAIN timers. It relies only on the specified cancellation action, not on already assuming that the Completion commitment is safe (`R:179–191`).

The unentered-cache exception is addressed: the first correct initial entry is e₀; a complete own-session receipt requires a correct FINAL originating no earlier than a timer expiry e₀+2Δ. Under the complete-handler entry contract, members that can participate have entered by e₀+Δ (`R:172–177`). Closure is treated separately by the early/late case distinction. An unentered or terminated cache does not silently execute Completion.

Conditional pass. Receipt formation itself remains unbounded and is not inferred from T_pub.

### 4. Retirement drain and early/late safety split

**J1: missing closure propagation for unentered programs.** The review below first exposes the missing prerequisite, then temporarily assumes it when checking the remaining algebra and independent cases.

The main invocation rule calls RequestClose for entered programs only (`M:1667–1671`). A Waiting program whose target becomes impossible instead follows definitive invalidation and direct termination (`M:1661–1666, 1736–1739, 2066–2069`; `R:108–113`). Only RequestClose explicitly says to relay the complete closure bundle to the fixed membership and record c_v (`R:30–47`). The unentered path has no corresponding mandatory relay. `B:80–85` correctly refuses initial Enter when a newer closure is already verified, so the Ready contract cannot make this member enter after the fact.

`M:2274` says “Notify regional nodes of any derived forced boundary.” If that sentence is intended to require immediate complete-proof multicast to *every* fixed regional member even when the relevant local program never entered, it could supply the missing premise. It currently specifies neither that recipient set nor that complete-message transport obligation, and can naturally mean the interpreter notifying its local regional manager. The explicit RequestClose relay cannot silently be extended to every definitive-cancellation path in the proof.

The following trace demonstrates why the distinction matters; all times use Δ=1. It is admissible on the literal direct-termination path and is excluded if the stronger complete regional closure notification is intended.

1. One selected region has correct H₀,H₁ and Byzantine Z, all unit weight, W=3,F=1. The session quota and K are both 2. H₀ is the correct generation-0 initializer; generation-1 initializer Z has a finalized immutable authority. All required history, state and a qualifying later Espresso header are available to H₀.
2. At time 0, H₀ performs Ready/Enter and sends its complete ReadyBundle. Its delivery to H₁ takes the permitted one unit. An authorized, sufficiently advance-announced MSIG becomes effective in a finalized Espresso block at time ε with 0<ε<1. H₁ obtains that complete public history immediately, while still Waiting, and directly terminates its unentered old-version target. H₀ obtains the triggering Espresso block only at time 20, within an allowed T_E=20 visibility bound. H₁'s cancellation sends no regional closure bundle on the literal path. This is not a regional message delay greater than Δ: the missing object was never required to be sent on that path.
3. At time 1, H₁ processes H₀'s ReadyBundle but cannot enter a verified closed version. This obeys the bundle's activation check. At time 2, H₀ finishes its correct initializer wait. Z promptly provides an ANCHOR vote, so H₀ obtains the generation-0 ANCHOR QC for inherited anchor A. H₀ proposes fresh DATA B at height h(A)+1 and votes for it. Z withholds its B vote. H₁ archives B without voting. No B QC forms.
4. At time 4, H₀'s B timer expires and commits B. Its FINAL alone cannot form a receipt, and B is nonterminal. There is no Completion or quit proof. At time 8, H₀ has cast only one of the two term-position votes, so its original p=2 progress check produces BLAME. Z supplies the matching BLAME; H₀ quits generation 0 at time 8 and enters generation 1 after the mandated wait at time 9.
5. H₀'s highest old QC is the ANCHOR QC for A, because no QC for B exists. Z sends a generation-1 NEWVIEW justified by that old ANCHOR QC. H₀'s ANCHOR lock test passes; Z and H₀ form the new ANCHOR QC. A fresh DATA B′, also at height h(A)+1 but with different valid content, is proposed and voted by H₀. Its timer expires by time 12 with no quit or Completion, committing a conflicting block. (Small positive message delays can replace the instantaneous exchanges without approaching time 20.)
6. H₀ first calls RequestClose at time 20. Thus c=20 and B's vote satisfies s+Δ=3<c, yet the claimed early-case protecting QC never existed. Its later recovery entry was possible because H₁'s earlier nonentry prohibition was omitted from c and from closure dissemination.

No forged signature, excessive faulty weight, premature control, inconsistent finalized source history, failed correct-to-correct delivery, unsound application validation, or delayed complete-result publication is required. Neither B nor B′ needs a complete FINAL receipt, so the frozen sound-receipt interface is not contradicted. The extra Ready timing premise also holds in its stated conditional form: an actual closure intervenes for H₁, so it does not promise H₁'s entry. Both commitments occur only in an already globally closed suffix; the separate global-prefix proposition is therefore not refuted. Fair future schedules and publication service can continue after this finite prefix.

Consequences: the all-correct vote inference in `R:217–219` and the dependent cross-generation conclusion in `R:251–262` remain unverified until the scope of closure propagation is established. This report does not assume that obligation as an unspoken environmental premise.

> “Every correct member has closed by c+Δ.” (`R:117–138`)

A complete closure relay at the first correct close c reaches and is processed by every correct member within Δ. The last permissible new vote occurs by c+Δ, so its timer expires by c+3Δ and its generated FINAL reaches peers by c+4Δ. Local d_v=c_v+4Δ is conservative because c_v≥c. Genuine Quit or Completion can cancel but cannot postpone a timer. FinishClose also requires timer exhaustion and acknowledged persistent ownership; the elapsed grace alone is insufficient (`R:79–106, 149–153`).

> “If s+Δ<c … Every correct member therefore has cast, or now casts, the matching … vote.” (`R:211–222`)

For the early case, **assuming the missing J1 propagation invariant**, no correct Quit, Completion or close can suppress voting through s+Δ: Quit/Completion would cancel the chosen direct timer through their complete relays. Earlier-term and already-committed ancestor proposals are still processed (`M:937–945`). The complete proposal then produces matching correct weight at least W−F>F, giving the protecting QC by s+2Δ to members that can later take recovery entry. Without that extra invariant, this is precisely the unsupported step.

First-generation processability depends on the named Ready contract. For recovery, the argument at `R:197–209` uses complete predecessor quit/admission history and the original Δ wait. It must not be read as importing Part I's common authority-availability time d(g), which is used by `I:616–625` but not promised by the frozen Part II model. The supplied inductive argument is adequate for this narrower processability claim: the first correct quit of each predecessor generation relays its admission history; a lagging member can finish the predecessor wait by at most one Δ after the earliest next entry, and the forwarded complete proposal supplies the next authority. No claim of simultaneous authority visibility is necessary.

Any first correct Quit of generation g must occur after s+Δ; hence every entry to g+1 satisfies e_v≥a_v+Δ≥a+Δ>s+2Δ (`R:224–230`). An early-case protecting QC is therefore available before every later live lock snapshot. A member already Completed cannot take such a snapshot and need not be assigned a fictitious unused lock.

> “In the late case s+Δ≥c … Every correct entry gate has closed by then.” (`R:232–237`)

Here timer expiry s+2Δ≥c+Δ is at or after every correct close. The strict recovery-entry inequality places any potential higher-generation entry after that time, where the gate forbids it. A later-generation QC requires a correct signer, so it cannot exploit the absence of a protecting QC. This covers equality s+Δ=c. The document explicitly exhibits a late local commitment with no protecting QC or complete FINAL receipt (`R:281–292`), rather than wrongly applying the universal Part I lemma.

The drain calculation and the late-window inequality pass for the stated RequestClose paths. The early-case safety result remains dependent on J1; it cannot be passed merely by the calculation. No increased quorum, retirement certificate or additional agreement is used.

### 5. Same-generation conflicts, cross-generation locks and receipt provenance

> “A correct signer of an incompatible history either processes B first and cannot vote, or forwards its history … no later than s+Δ.” (`R:239–249`)

Evidence-first processing continues after close, so a closed receiver does not suppress conflict detection merely because it no longer votes. The same-generation signed-history/fork property is the baseline prerequisite. If the conflicting signer acts first, its complete forwarded history reaches the chosen committer by expiry; if B is processed first, the retained evidence prevents its conflicting action. This also excludes a QC assembled later from already cast conflicting votes.

Across generations the late case has no later correct votes. In the early case, the protecting QC exists before every live recovery lock; ANCHOR's older justification must meet that lock, and each DATA/DRAIN parent justification has smaller rank (`R:251–262`; `M:797–816, 977–988, 1219–1223, 1247–1264`). The baseline least-ranked-counterexample descent is therefore applicable. DRAIN's physical position increases even when its DATA height does not; the descent is not incorrectly based on DATA height alone.

> “Committing from another receipt emits no FINAL.” (`R:264–272`; `M:865–870`)

Every receipt has a correct FINAL signer. That signer committed the named DATA by a direct timer or as an ancestor of a direct timer commitment. Receipt-derived commits generate no new FINAL, so receipt provenance cannot form a self-supporting cycle. Completion adds an already protected DATA prefix; cancellation of an unexpired DRAIN timer does not commit that DRAIN. This closes the receipt-based branch of the proof without assuming its conclusion.

The same-generation and receipt-provenance checks pass. The cross-generation conclusion remains unverified because it depends on J1's missing protecting-QC prerequisite; temporarily assuming that prerequisite, the ranked descent is locally valid under the explicitly assumed baseline validation properties.

### 6. Same-region fallback and strict terminal selection reference

> “The terminal DATA additionally requires height(ξ(T))>q_open.” (`M:628–645`)

For a REGULAR successor, q_open(next)=q_s≥height(ξ(T))>q_open(current). The integer selection heights therefore escape every fixed historical height along an infinite regular sequence (`M:2647–2659`). With a fixed nonempty eligibility set, the cyclic selector visits each eligible member repeatedly; with one eligible member, fallback renews it. The premise concerns infinitely many consecutive regular handoffs and does not imply that they occur or that their output is accepted.

AUTO still excludes its source during cooldown, and OFF is not overridden by expiry or same-region fallback (`M:1776–1782`; `M:2661–2667`). A later Force supersedes a pending boundary rather than letting it mutate into another current boundary. Strict references add a Ready prerequisite for a qualifying finalized header, not a bound on when such a header or state first becomes usable.

Pass for the two explicitly scoped selection claims.

### 7. Pure validator and identical accepted prefixes

> “Replaying the same complete Espresso prefix yields the same interpreter state.” (`M:2527–2535`)

The induction has the necessary canonical inputs and order: complete applicable controls; first matching MSIG relative to the entry version; AUTO only in its absence; one pre-payload snapshot; ordered atomic EXPORTs; and first eligible historical selection (`M:2171–2209, 2264–2287`). Invalid exports do not mutate counts or retry retroactively (`M:1451–1456`). Missing dependencies pause at the same canonical position rather than selecting a later public timestamp (`M:1442–1449`).

Selection references must precede the containing EXPORT, and a Force-created branch cannot export in its triggering block (`M:1430–1434`); neither receipt/history validation nor selection needs the same EXPORT to justify itself. Historical caches provide validated objects, while current append permission is checked independently (`M:1904–1911`). The existing-fixed-record comparison is a consistency guard against replacing canonical fixed fields, not an alternative source of selection authority (`M:1887–1891`). Under the baseline uniqueness premise, valid cache hits and misses derive the same fixed fields.

Force preserves all pre-payload G and its provenance; it cannot select a lower anchor or accept later old-version DATA. Local program phases, closure processing and storage callbacks never mutate G (`M:1686–1691, 2556–2564`). Thus different complete-prefix lengths correspond to different positions on one append-only state sequence.

Conditional pass under the frozen authenticated-history and sound-validation interfaces. This is a source-level deterministic transition argument, not a tested certifier implementation.

## Assumption-preservation audit outside the unchanged Model

| Change or dependency | Assessment |
|---|---|
| Complete receipt multicast | New protocol obligation on already-correct holders, with the existing intra-region complete-message bound. It does not assume that receipts form or that cross-region transport is bounded. |
| Authenticated closure relay and 4Δ drain | New protocol behavior derived from the existing Δ bound. No new public-source visibility deadline or retirement agreement. Transfer remains unbounded. |
| Closure of an unentered selected member | **J1 unresolved:** existing prose does not explicitly establish complete regional relay for the direct-termination path. Treating every such prohibition as if it had a c_v/RequestClose relay would import an unstated protocol obligation into the early-window proof. |
| Atomic ReadyBundle adoption and Enter | Explicit abstract message/representation contract. The timing theorem is valid only if the actual native adapter meets it; `B:120–140` correctly refuses to turn arbitrary database import into a bounded message operation. This is a real proof boundary outside the unchanged Model, not satisfied merely by byte-identical assumptions. |
| Baseline DATA/DRAIN, application and receipt soundness | Assumed interface, not independently proved by this review or by the v3.3 extension. The v3.1 baseline already disclaimed a regional reproof (`F:2097–2101`); the v3.3 retirement statement makes its dependency explicit. |
| Recovery authorities | No simultaneous availability premise is imported from Part I. The retirement argument uses complete relayed admission history and actual mandatory waits, as checked above. |
| Same-region fallback and advancing ξ | Protocol selection/validation changes, not new honesty or availability assumptions. Their limited selection proposition does not strengthen the frozen progress quantifiers. |
| Persistent owners and tombstones | Required local correct-replica state/retention behavior implementing the existing persistence duties. No bounded transfer time, bounded storage size, stake-withdrawal permission, or trusted new controller is asserted. |
| Quorums, control authority, synchrony and global finality | Unchanged. The review rejects attacks requiring forged signatures, more than F Byzantine weight, conflicting finalized source histories, omitted applicable controls, late unauthorized controls, or correct-to-correct delay beyond Δ. Those are premise violations, not counterexamples. |

The source's safety qualification is therefore material and must survive any summary. Even at the abstract complete-message level, J1 must be resolved before passing the retirement extension. The independent conditional passes do not establish an arbitrary native implementation's Ready timing, independently prove the assumed regional engine, or resolve the acknowledged progress failures.
