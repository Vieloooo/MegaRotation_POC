# Cycle 1: independent liveness and recovery verification

Authoritative input: `formalization/main.tex`, SHA-256 `97d709fcfcfe9e72e93d05dfca9da679d79c82a282b27e41dbcce2412ef7f5e9` (verified before review). This review read the current manuscript and the proof-verifier instructions, not earlier review files. It does not edit the manuscript. The epoch-boundary concern was communicated by the coordinating reviewer and then checked independently against the text.

## Summary

**Final Verdict:** The central fixed-anchor and cyclic-recovery arguments are viable, but verification is incomplete because the report snapshot's slot cutoff is not explicitly reconciled with current-slot fork-choice eligibility, and the numerical repair lemma assumes an unstated bound on successful fetches relative to their timeouts.

No counterexample to unbounded adopted-height growth was established under a coherent completed-slot snapshot interpretation. In particular, this review does not assume that ordinary HEAD messages finalize batches, that regional heads converge, or that any chosen speculative batch must survive.

**List of Findings:**

1. **Location:** lines 353–363, 451–459, and 534–546: “All its producer slots end no later than the epoch's start”; “An honest active member reports the longest fitting complete prefix of its locally selected LMD branch at this boundary”; “If the selected branch is nonempty, the admitted prefix must be nonempty”; and “Buffer future-slot objects without selecting or voting for them.” **Issue — Justification Gap:** a prebuffered Byzantine batch for the slot that opens at the epoch boundary is no longer a future-slot object, but it is still forbidden by the report cutoff. The snapshot has no explicit completed-slot eligible tree or ordering rule that excludes that batch. “Snapshot before local production” does not exclude data signed and disseminated earlier by a Byzantine producer. The nonempty-prefix requirement can conflict with the cutoff when only a current-slot branch is selected.
2. **Location:** lines 825–835: “its repair time after the first request is at most $(a+1)\theta$” and “the next admitted honest response verifies within that deadline.” **Issue — Justification Gap:** neither the transport rule nor the timing assumptions state that the timeout dominates the complete successful request/response/verification delay. The proof cannot infer the latter sentence merely from responsiveness. This affects the stated numerical repair bound and its substitutions, not the logical fixed-anchor height-growth argument.

## Detailed Verification Log

### 1. Clock authority, silent leaders, and epoch alignment

> Lines 407–425: slots and producers are calculated from the activation time and the configured list; a successor needs “no release signature from its predecessor, no local quorum, and no certified outcome for the missing slots.”

This is sufficient to bypass a silent scheduled producer. A term is a fixed number of clock slots; silence cannot delay the next term. A node that needs recovery first obtains history and joins a future slot, so the proof does not obtain progress by permitting retroactive honest signing. The bound $L_i=(n_iN_R+1)\tau_R+\Delta_R$ is conservative for reaching a responsive honest producer's next scheduled slot plus an alignment slot and dissemination.

The epoch/slot alignment requirement is also sufficient: $T_G/\tau_R$ is an integer, and every non-genesis activation occurs at an epoch boundary. No fractional-slot handoff is needed. The next epoch boundary after receipt of a proposal is a boundary at which that proposal's producer slot has ended, except for the separate current-slot selection ambiguity described below.

### 2. Report cutoff versus the boundary's current regional head

> Lines 353–354: “All its producer slots end no later than the epoch's start.”
>
> Lines 359–362: “An honest active member reports the longest fitting complete prefix of its locally selected LMD branch at this boundary. If the selected branch is nonempty, the admitted prefix must be nonempty.”
>
> Lines 451–452: “Buffer future-slot objects without selecting or voting for them.”
>
> Lines 541–546: “Active-region members re-root local trees before snapshotting. ... Local production in the new slot then proceeds independently.”

**Finding 1: Justification Gap.** The temporal rules do not explicitly freeze report selection to batches from completed slots. At an epoch boundary, the opening slot is current, so R2's future-slot exclusion alone no longer excludes it. The order in G1 constrains honest local production, but cannot constrain when a Byzantine producer signed or disseminated a valid current-slot batch.

A concrete boundary schedule is:

1. Use an eligible region with four equal-weight members, one Byzantine; let the Byzantine identity be first in the configured producer list. All honest members remain responsive, satisfying both regional and global weight premises.
2. A finalized rotation is due to activate this region at the start of epoch $a$. Its adopted anchor and activation proof are available before that boundary, as required by the global envelope.
3. The Byzantine first producer signs a legal one-batch extension for slot $u=1$ in the new global view and disseminates its complete bytes before activation. The receivers buffer it while it is future-slot data; there is no honest-signing rule restricting this Byzantine act.
4. At activation, the effective anchor is installed and slot 1 becomes current. If buffered data is released before report selection, the one-batch tree is complete, current-view, within depth and candidate caps, and its single branch is selected even at zero HEAD weight.
5. The epoch report must exclude this batch because slot 1 ends after the snapshot boundary. Reporting it violates `ValidReport`; reporting the empty branch contradicts the unconditional nonempty-selected-branch requirement quoted above.

An implementation can resolve this by assigning a particular order or by interpreting “fitting” to include the cutoff, but neither reading follows unambiguously from the present definition. In the second reading, the unconditional nonempty-prefix statement is still wrong in this schedule. The proof therefore needs an explicit bridge between the tree used for the snapshot and the `ValidReport` cutoff. This is an underspecified honest transition, not a demonstrated infinite execution in which honest nodes obey mutually inconsistent requirements.

The dependency affected is `honest local selected branch -> honest nonempty valid REPORT -> honest-origin AB validity -> snapshot growth` (lines 864–872). The snapshot-selection lemma itself remains sound for a common vector of already valid reports. A completed-slot snapshot interpretation also preserves the intended progress proof: an already received fixed-anchor extension remains in the completed-slot tree at the next snapshot.

### 3. Latest HEAD behavior, missing data, and depth backpressure

> Lines 320–325: a newer HEAD replaces the older record; latest equivocation, missing data, or an incompatible branch supplies zero weight, without resurrecting the earlier vote.
>
> Lines 432–436: fork choice continues even when all child weights are zero.

The proposed attacks based solely on erasing latest-message weight fail. A Byzantine can remove its own usable weight, release a missing-body tip, or change forks, but none of these actions removes a previously admitted complete branch from a fixed-anchor tree. `Head` still descends into some child. There is no proof dependence on a majority having identical latest-message maps.

> Lines 312–318: speculative depth is bounded by $d_R$, a full branch can still be reported, and global validation is independent of a receiver's local candidate cap.

Depth backpressure does not by itself break progress. If an honest scheduled producer finds a full selected branch, that branch is already a nonempty candidate; if it finds no branch, the mandatory production rule creates one. The local two-candidate cap cannot exclude a prescribed honest producer's unique batch under the explicit reserved-capacity premise. An alternative branch that changes the selected head cannot change a nonempty finite tree into an empty tree. This conclusion uses the protocol's specified anchor pruning and does not authorize arbitrary garbage collection of live candidates.

Missing ancestry is explicitly inside the complete-object delivery assumptions. A missing-body HEAD is excluded from weight; it is not treated as availability evidence. Thus a withholding attack cannot be used while simultaneously assuming that its unavailable object was an admitted complete report with a bounded delivery time.

### 4. Fixed-anchor progress, conditional on a valid snapshot definition

> Lines 860–867: within a full local producer cycle, either an extension exists or the responsive honest producer creates one; retained data and a fixed anchor preserve a candidate.

This step is valid under the explicit synchronized-production and admitted-workload premises. It does not establish preservation of a particular preview, nor does it need to. After a nonempty completed-slot tree exists, `Head` supplies a nonempty selected path independently of current HEAD weights.

> Lines 869–873: the next snapshot is within $T_G$; its valid honest report is not bottom; without rotation the common tree selects an extension and adopts it within another $T_G$.

Once Finding 1's snapshot ambiguity is resolved, these deductions are valid. The capacity assumption explicitly requires a report to fit one whole legal batch plus relative authorization evidence. The selected leaf in the union of report paths is an endpoint of a reported path, so its branch is not an arbitrary concatenation exceeding the report budget. An honest valid report survives authenticated broadcast. A common nonempty report tree produces a strict height increase even if every endorsement happens to have zero weight under some unrelated local HEAD map.

### 5. Minimum tenure and eventual healthy selection

> Lines 878–880: $\min\{K_G,N_G\}T_G>L_*+3T_G$.

The inequality has enough margin, including the apparent counter off-by-one. If activation occurs at the start of epoch $a$, the first no-progress trigger can occur at the snapshot of epoch $a+K_G-1$, after $(K_G-1)T_G$ from activation. The displayed requirement implies

$$
(K_G-1)T_G>L_*+2T_G\geq L_i+2T_G.
$$

The planned-rotation trigger is no earlier than epoch $a+N_G$, after $N_GT_G>L_*+3T_G$. Hence neither trigger replaces a newly activated healthy region before the fixed-anchor progress window. The fact that a rotation retains the parent instead of its candidate does not invalidate this argument: the trigger itself is excluded throughout the needed window.

> Lines 898–907: bounded monotone heights would eventually be constant; a continuously healthy region keeps producing READY; cyclic selection reaches it; sufficient tenure then contradicts stagnation.

This is a valid contradiction structure, subject to the snapshot qualification. A bounded integer-valued monotone height sequence is eventually constant, and a positive-length batch is the only way to increase the adopted height. Once constant, late data cannot keep resetting the no-progress counter by actually advancing height. With a continuously ready target region, each selection of a different region before that target moves forward along the finite cyclic interval toward it; variable readiness of intervening regions cannot skip the target forever. Same-region restart cannot be chosen while a different continuously ready target exists. Each pending rotation finishes after its stipulated CONFIRM/finality/activation sequence under the global envelope.

Every adopted checkpoint is finalized by the next adjacent epoch link. Consequently, unbounded adopted heights imply unbounded finalized heights. This remains a claim about some new data, with no transaction-inclusion or particular-batch persistence promise.

### 6. Repair timeout qualification

> Lines 825–827: “If a bounded requested object has $a$ failed attempts before a responsive honest holder, its repair time after the first request is at most $(a+1)\theta$.”
>
> Lines 833–835: “Each failed attempt costs at most $\theta$, and the next admitted honest response verifies within that deadline.”

**Finding 2: Justification Gap.** The network model supplies complete-object delivery bounds $\Delta_R,\Delta_G$ (and a consumer path bound $\Delta_P$), while D2 independently introduces attempt timeouts $\theta_R,\theta_G$. No inequality or premise says that a successful bounded GET/DATA operation, including verification, fits within its timeout.

For example, let a timeout be one time unit and let an immediate request to the first responsive honest holder return a completely verified object after two time units, within a stated complete-object bound of three. There are no bad holders ahead of it, but responsiveness alone does not make this request successful within one unit. Thus the proof's last-attempt inference is unavailable. If $a$ instead counts actual timed-out attempts, including attempts at responsive holders, that meaning and the handling of late successful replies must be supplied; the existence of a responsive holder is not by itself the assertion that the next attempt succeeds by its deadline.

The protocol says to validate and cache complete replies and does not expressly discard late replies, so this observation does not establish permanent consumer starvation. It does prevent verification of the numerical repair formula as written. The same qualification applies when $F_R,F_G$, successor synchronization, or $F_P$ are obtained by substituting this formula. It does not affect a timing claim that separately assumes a valid complete-range repair bound as an input.

### 7. Outage and delivery bounds

> Lines 981–1000: the outage bound separately includes late adopted progress, detection/alignment, readiness, two handoff epochs, new progress/finality, and consumer repair.

The decomposition is conservative once its component bounds are available. It correctly does not treat the first local silence as the start of a universal fixed timeout: an old-region report can still advance the adopted height and reset $z$. $L_{\rm tail}$ is an additional conditional bound on the last such advance, not a consequence of mere outage. Likewise, $T_{\rm ready}$ includes synchronization and report alignment; a readiness signature only proves possession when signed, and the theorem permits an already selected region to fail before activation.

The two epochs for ROTATE/CONFIRM/activation match the specified boundary: ROTATE in epoch $r$, CONFIRM/link in $r+1$, activation at the start of $r+2$. The extra epoch from adoption to finalization is also accounted for. No shorter per-mini-block finality claim has been used. The unresolved items affecting a fully verified delivery bound are the report-cutoff definition in Finding 1 and the component repair-time premise in Finding 2.

## Limits of this review

The verdict treats complete-object synchrony, admitted workload, permanent retention, and a continuously responsive global supermajority as premises supplied by the manuscript. It does not certify that a deployment can realize those capacity or storage assumptions. No demand for per-batch agreement, regional quorum certificates, LMD convergence, individual inclusion, or bounded reorganization persistence was introduced.
