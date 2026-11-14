# Cycle 2: liveness repair verification and cost audit

Authoritative input: `formalization/main.tex`, SHA-256 `4e246692d857d8269b5e6e0b195e98f67a03a30c523f30c54fbc31530685ba35`; identical frozen input: `formalization/build/lmd-redesign-baseline/cycle2-main.tex`. Both hashes were checked. Line references below refer to this frozen revision. This is a repair recheck, not a fresh-context audit. The manuscript was not edited.

## Summary

**Final Verdict:** Both cycle-1 liveness findings are closed, and the fixed-anchor progress, tenure, and conditional recovery arguments remain valid under the stated premises; a newly checked payload-complexity claim is false if read as an upper bound on the entire epoch protocol rather than only its final common report vector.

**List of Findings:**

- **Closed — report cutoff.** Lines 359–374 explicitly filter the candidate tree to completed slots before running LMD or truncating its result. Newer HEAD records have zero snapshot weight without reviving old votes. The previous early current-slot batch schedule no longer creates an invalid honest report.
- **Closed — repair timeout premise.** Lines 958–969 define complete GET–DATA round-trip bounds, including service, evidence, verification, and consumer routes, and require each timeout to dominate its corresponding bound. This supplies the missing last-attempt premise of the repair lemma.
- **Remaining — Critical Error in the protocol-wide payload upper bound.** Lines 1085–1088 say, “An epoch may expose up to $NB_G$ distinct relative report bytes.” Authenticated broadcast can disseminate several valid semantic report values for one equivocating origin before returning bottom. The cap of two values is per honest receiver and does not limit each origin to one globally exposed report. The $NB_G$ upper bound is valid for the final common vector, but not for AB witness exposure, verification, or retention during the epoch. This does not invalidate the report-agreement or liveness proofs because they explicitly assume enough capacity for prescribed honest traffic.

## Detailed Verification Log

### 1. Completed-slot snapshot repair

> Lines 359–368: define $\bar u_e=\lfloor((e-1)T_G-t_{\mathrm G})/\tau_R\rfloor$, filter batches to slots at most $\bar u_e$ before LMD, zero later HEAD records, and exclude pending objects for the opening slot even if their bytes arrived early.

For slot $u$, the end boundary is $t_{\mathrm G}+u\tau_R$. Therefore $u\leq\bar u_e$ is exactly the completed-slot condition at epoch start. Since batch slots strictly increase along a branch, this filtering is prefix-closed: removing a too-new batch also removes every descendant and does not leave disconnected paths. At activation $\bar u_e=0$, so a pre-signed slot-1 batch cannot enter that view's first snapshot.

The new order closes the cycle-1 counterexample. It also avoids a subtler ineffective repair that would first select a current-slot fork and only then truncate it to the root, despite another reportable old branch being available. Here the old branches participate in the actual snapshot fork choice.

Zeroing a record whose HEAD vote slot exceeds the cutoff, without falling back to an earlier record, is consistent with the manuscript's latest-message bookkeeping. A Byzantine signer can thereby remove its snapshot weight, but cannot remove complete past-slot branches. Even if every usable weight is zero, `Head` descends into a child of a nonempty filtered tree. The repair consequently introduces no new quorum or convergence dependency.

### 2. Fixed-anchor progress after filtering

> Lines 876–884: in a full producer cycle either a complete extension already exists or a responsive honest producer constructs one.
>
> Lines 886–894: at the next aligned epoch snapshot the available batch's slot has ended; the old branch remains in the filtered tree and yields a valid nonempty report.

These steps are valid together. A prescribed honest producer cannot be denied its unique slot by predecessor silence. Its complete admitted proposal arrives within the regional bound. At the next global boundary, its slot has ended because all epoch boundaries are slot boundaries. If the producer instead finds a full selected tree, a past-slot extension already exists and remains reportable. The fixed anchor and absence of a specified arbitrary candidate-eviction rule preserve this branch until the snapshot.

An early current-slot object can affect ordinary local speculative head selection, but it is excluded from the snapshot tree. Filtering therefore preserves the exact nonemptiness property used by the proof. Whole-batch report capacity ensures a nonempty prefix fits. Valid honest-origin AB input is returned, and the common report transition adopts a strict extension when rotation is excluded.

The previous progress bound $L_i+2T_G$ still has its original meaning: time to a responsive honest producer/candidate, at most one epoch alignment, then one adopting epoch. No additional snapshot epoch was introduced by the repair.

### 3. Tenure and cyclic recovery

> Lines 899–900: $\min\{K_G,N_G\}T_G>L_*+3T_G$.

The no-progress counter first reaches $K_G$ at the snapshot of epoch $a+K_G-1$ if activation is at epoch $a$. Its earliest elapsed trigger time is $(K_G-1)T_G$, and the stated inequality gives

$$
(K_G-1)T_G>L_*+2T_G\geq L_i+2T_G.
$$

The planned trigger requires elapsed time $N_GT_G>L_*+3T_G$. Thus both triggers leave the complete progress interval intact. The mandatory empty report at activation is already covered by this margin.

If adopted heights were bounded, monotonicity would make them eventually constant. Constant height gives a fixed parent prefix; a continuously healthy eligible region can hold and endorse that prefix in each new epoch's READY reports. Cyclic choice cannot jump over that continuously ready target indefinitely. Pending transitions finish under the global envelope, and the newly healthy active region receives enough tenure to contradict stagnation. No persistence of a particular preview or any local agreement claim is used.

### 4. Complete pull-timeout repair

> Lines 958–969: $D_R^{\rm pull},D_G^{\rm pull}$ include the complete admitted GET–DATA round trip, service, relative evidence, response bytes, verification, and the consumer path when applicable; $\theta_R\geq D_R^{\rm pull}$ and $\theta_G\geq D_G^{\rm pull}$.

This closes the numerical gap in lines 840–855. After at most $a$ failed capped attempts, contacting the responsive serving holder produces a verified object by the next timeout. The resulting bound is

$$
a\theta+D^{\rm pull}\leq(a+1)\theta.
$$

The new premise explicitly avoids confusing validator one-way delivery with a consumer's successful fetch. Multiple chunks are still correctly charged as a sum, and $F_P$ must cover the entire missing range rather than just a final object. The result remains conditional on reaching a serving holder and admitted workloads; it does not assert availability of a withheld speculative body.

### 5. Numerical delivery and geographic recovery

The bound $L_i+3T_G+A_P+F_P$ remains conservative. It adds one finalizing epoch to fixed-anchor adoption and then notification plus complete consumer repair. The repaired consumer rule requiring an already-due anchor before delivery does not add an uncharged future boundary: a checkpoint $C_e$ is installed from epoch $e+1$, and the next adjacent link that finalizes it is generated during that epoch. Future rotation activations are applied only when due.

The geographic expression at lines 1013–1032 remains a conditional decomposition. Its $L_{\rm tail}$ term is essential: later old-region reports that genuinely advance height can reset no-progress detection. $T_{\rm ready}$ covers actual synchronization and READY alignment. ROTATE in epoch $r$, CONFIRM/finality in $r+1$, and activation at the start of $r+2$ fit the two-epoch transition allowance. The subsequent $L_*+3T_G$ covers some new adoption and finalization, not a targeted transaction.

These formulas do not infer a finite $L_{\rm tail}$, finite complete-range repair cost, or sufficient transfer capacity merely from nominal batch-size maxima. The manuscript supplies these as separate premises. No additional uncharged delay was established under those premises.

### 6. New payload-complexity counterexample

> Lines 1085–1088: “An epoch may expose up to $NB_G$ distinct relative report bytes. Global verification can require every responsive member to fetch the union, including branches not selected ...”
>
> Lines 1165–1174: receive processing verifies the complete witness before retaining at most two semantic values; each honest member forwards each retained value at most once.

**Critical Error in the protocol-wide upper-bound reading.** The final vector has at most one non-bottom report per origin, so its union is bounded by $NB_G$. The AB execution preceding that vector has a different bound because values from an equivocating origin may differ at different honest receivers.

A concrete admitted schedule uses four equal-weight global participants, one Byzantine active-region origin $B$ and three honest responsive participants $H_1,H_2,H_3$:

1. The Byzantine origin creates six valid reports $r_1,\ldots,r_6$ for the same parent and epoch. Each report contains a different complete legal past-slot branch with disjoint body bytes approaching the configured report limit. Byzantine producer equivocation permits these alternative branches; `ValidReport` is explicitly independent of each receiver's local two-candidate cap.
2. In round 1, $B$ sends $r_1,r_2$ only to $H_1$, $r_3,r_4$ only to $H_2$, and $r_5,r_6$ only to $H_3$. Each honest receiver validates and retains two values, within its cap.
3. At the round boundary, each honest receiver forwards its two retained values, exactly as A3 prescribes. Every other honest receiver must process those complete honest-forwarded witnesses in round 2, even though its already-full retained set ultimately ignores their values.
4. The origin's final AB output is bottom everywhere. Nevertheless, six distinct report bodies have been exposed and checked. For $N=4$, these can exceed $NB_G=4B_G$ in total distinct relative body bytes.

This is not an attack relying on excluded unlimited Byzantine flooding. The extra round-2 traffic is mandatory honest forwarding and is reserved by the admission premise. The broadcast proof itself explicitly allows different honest nodes to retain different pairs under equivocation.

For the number of distinct semantic values that are retained and then forwarded by honest nodes, a coarse bound is $2N$ per origin and $O(N^2)$ across all origins. Their body union can therefore be $O(N^2B_G)$ rather than $NB_G$, with uncached reception/verification at all $N$ endpoints potentially costing $O(N^3B_G)$ aggregate body bytes. These are coarse protocol-traffic bounds, before route multiplication, retries, and caching improvements. Additional rejected Byzantine-origin traffic remains governed by the configured admission quotas and the manuscript's flood exclusion.

The unqualified sentence at lines 1085–1088 should therefore not be used to size the global complete-object capacity premise. The same warning applies to any engineering interpretation that sizes all intermediate report-body verification from the final vector alone. The global liveness proof remains conditional on a capacity envelope that actually covers these prescribed forwards; it does not establish that envelope from $NB_G$.

### 7. Other complexity checks

For prescribed control traffic, each honest participant forwards at most two semantic values per origin and at most once per value. Across $N$ origins this gives $O(N^2)$ logical multicasts, $O(EN^2)$ backbone arc transmissions, and the stated $O(EN^2(L_G^c+N\kappa))$ control-bit expression. The chain has at most $N$ signatures. The $O(N^3(N+\nu_G))$ endpoint verification count is a conservative uncached control/evidence-check count: $O(N^2)$ logical messages, $N$ recipients, and up to $O(N+\nu_G)$ checks each. These coarse control counts already allow different retained values across receivers; the payload statement is the inconsistent part.

The local $O(n_i^2)$ HEAD count and absence of additional local-rotation control messages are consistent with the algorithms. The retention discussion correctly distinguishes bounded eligible depth from unbounded historical storage. The body-repair expression is conditional on capped failed responses, and physical route bytes must be charged separately, as the text states. These formulas are not unconditional upper bounds on arbitrary rejected Byzantine input traffic.

## Closure status

Cycle-1 report-cutoff finding: **closed**.

Cycle-1 repair-timeout finding: **closed**.

Core progress/tenure and conditional recovery recheck: **passed under the manuscript's explicit network, workload, retention, and availability premises**.

New issue requiring correction or explicit narrowing: **the $NB_G$ relative report-body exposure bound describes the final report vector, not the entire authenticated-broadcast execution**.
