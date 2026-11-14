# Independent liveness and timing audit

Source: frozen `source/main.tex`, SHA-256 `325ab6a05c0fd903f0f999a1d474218d122a302a54aa9c65a744a5dc989a4223`. Line references below refer to that frozen file. No manuscript changes were made.

## Verdict and scope

I did not establish a counterexample to the stated height-growth theorem while retaining its continuous global delivery envelope, honest responsive supermajority, admitted-traffic capacity, positive report capacity, minimum tenure, and eventual continuously healthy eligible region. The liveness argument deliberately requires only some new mini-blocks, including empty ones; losing particular previews or transactions does not refute it.

There is a concrete permanent-stall execution after a single temporary global partition. It is a practical recovery limitation, not an in-model refutation: lines 1145–1147 expressly exclude the failed synchrony premise and say recovery from arbitrary global epoch stalls is unspecified. The execution below shows why merely restoring connectivity does not resume this protocol.

## L1 — A transient partition can leave an absorbing global stall

**Classification:** Confirmed execution outside the continuous-synchrony envelope; severe availability/recovery limitation. It preserves the cryptographic and honest-weight premises. It does not violate the delivered-prefix safety claim.

**Targeted practical expectation:** A service resumes after a temporary partition, without changing configuration or overriding honest persistent signing history. The manuscript does not promise this expectation.

**Exact relevant rules:**

- Global reports start with a justified `C_(e-1)` and complete parent data (556–558).
- The signed broadcast fixes its output after `N` rounds (566–570; 1184–1205).
- A packet not completely valid by its chain-length round deadline is ignored (1192–1195).
- The checkpoint identity commits to the complete report vector (336–340).
- The global target is voted with source epoch `e-1` and target epoch `e` (571–574).
- Honest nodes persist at most one FFG statement per target epoch; the construction uses only adjacent links (625–628).
- A checkpoint needs a quorum strictly above two thirds of total weight (122; 152–154).
- Missing the boundary's required checkpoint prohibits new batch, HEAD, or REPORT origination from a stale anchor (531–538).
- Arbitrary global epoch stall recovery is explicitly unspecified (1145–1147).

### Configuration and timing

Take two regions `V_1={A,B}` and `V_2={C,D}`, with all four nodes honest and each of weight one; no Byzantine signer or forged message is needed. Thus `W=4`, and a global quorum requires at least three distinct signatures. Region 1 is active. Start from the common justified/finalized genesis checkpoint `C_0` at time zero.

For a concrete otherwise compliant timing configuration, set

\[
\Delta_R=\Delta_G=1,\quad \tau_R=3,\quad T_G=9,
\quad N_R=1,\quad N_G=K_G=5.
\]

Then `τ_R>2Δ_R`, `T_G≥(N+3)Δ_G=7`, and `T_G/τ_R=3`. Also

\[
L_*=((2)(1)+1)(3)+1=10,
\qquad \min\{N_G,K_G\}T_G=45>10+27.
\]

Choose legal payload/report capacities. Genesis activates the first global view at epoch 1 with zero counters. All first reports have empty branches because the activation snapshot cutoff is zero (367–368). All nodes have the genesis prefix and may truthfully set READY.

The sole fault is a communication partition separating `{A,B}` from `{C,D}` throughout epoch 1, `[0,9)`. Deliver all messages within each side in their normal time bounds, including the active region's production and HEAD traffic. Hold all cross-partition control traffic until just after time 9. The nodes remain honest and their signing histories survive. Only the global complete-object delivery premise needs to fail; both regions can retain their local delivery bounds throughout.

### Execution

1. **Time 0:** Every node signs its prescribed empty report `r_(S,1)`. The reports all refer to `C_0`. The signed broadcasts begin normally.
2. **Time 4:** The four-round broadcasts end. Nodes `A,B` have common vector

   \[
   \mathbf r^L=(r_{A,1},r_{B,1},\bot,\bot),
   \]

   while `C,D` have

   \[
   \mathbf r^R=(\bot,\bot,r_{C,1},r_{D,1}).
   \]

   Every non-bottom report is valid and completely available inside its component. Remote origins supply no timely packets and correctly return bottom under the local algorithm.
3. **Time 4:** Both components derive HOLD with the same adopted tip, but distinct checkpoint identities: `Hash(r^L)≠Hash(r^R)` under binding hashing, hence `g_1^L≠g_1^R`. The report-vector commitment alone suffices to distinguish the targets even though their adopted data and control state agree.
4. **After derivation:** `A,B` persist and broadcast votes for `(0,g_0,1,g_1^L)`; `C,D` persist and broadcast votes for `(0,g_0,1,g_1^R)`. Each target gets exactly two signatures. Neither can get the required three-signature certificate.
5. **Time 9:** Epoch 2's initial justified parent `C_1` does not exist. Nodes cannot begin its prescribed global instance. Their required boundary checkpoint is missing, so fresh regional origination from `C_0` is also prohibited.
6. **Time 9+ε:** Heal the network permanently. Release every delayed message and thereafter respect all stated delay bounds.

### Why healing is insufficient

- **Late reports cannot revise the completed broadcast.** They missed the round deadline in A2; A4 already returned its result. No replay/restart rule for the same epoch is given.
- **Votes for the two targets cannot be pooled.** A QC is a set of signatures on the same statement (152–154).
- **No target can obtain a third honest vote.** Every node has already persisted a different or matching target-epoch-1 statement, and F1 forbids changing it. Since all four nodes are honest, there is no future Byzantine assistance to consider.
- **Advancing the epoch does not supply a parent.** The next instance requires justified `C_1`, and honest vote generation is restricted to adjacent links. There is no rule to skip the missing epoch, vote an old source into a later epoch, or advance by timeout.
- **Geographic rotation cannot repair this state.** ROTATE itself needs the stalled global certificate pipeline. A new L1 configuration or operator recovery procedure would be an external mechanism.

Therefore no `C_1` becomes justified and the prescribed state machine cannot produce later global checkpoints, even with a permanently reliable network after time `9+ε`.

**Effect on the three output levels:** During epoch 1, the active region can still construct and circulate previews, until its depth cap is full. At time 9, the missing-parent rule prevents fresh batch, HEAD, and REPORT origination even though that regional network never failed. No later adoption or newly finalized delivery occurs. Old previews and historical data can still be served, but this is not new stream progress.

**Checks against false positives:** This is not a same-epoch certificate conflict: neither side has a quorum. It uses no dishonest double vote. It does not claim the theorem applies after its continuous global synchrony premise failed. It does establish that the reference protocol has no self-recovery from this ordinary transient failure.

## L2 — LMD balancing and latest-message suppression do not defeat the stated height goal

**Candidate:** A Byzantine member selectively releases competing branches or newer HEADs, double-signs HEADs, points to unavailable branches, or delivers a new-slot HEAD at an epoch boundary. Honest HEAD support might vanish or disagree forever.

**Why it fails against the stated claim:**

1. Latest invalid/missing/equivocating HEADs indeed contribute zero weight (320–326), and future-slot HEADs are rejected for current processing (303–306).
2. `Head` continues down an eligible child even when every child has weight zero (446–449). Manipulating weights cannot make it return the anchor while a complete eligible child remains.
3. The report snapshot first removes new-slot batches and ignores new-slot HEAD weights (359–368). This operation does not delete an older complete candidate.
4. The honest report must include a nonempty fitting prefix of a nonempty selected path; one whole legal batch and authorization evidence must fit (370–374).
5. An honest responsive origin's report survives its authenticated broadcast, so the common report tree has a child. Fresh snapshot votes may choose another branch, but they still select some nonempty extension (590–595).

**Consequence:** These schedules can attack preview stability or individual transaction inclusion. Those properties are explicitly excluded (250–253; 1139–1143). They do not establish permanent height stagnation.

## L3 — Filling the speculative depth budget does not create the claimed deadlock

**Candidate:** Fill every honest tree to `d_R` so honest producers stop, then keep HEADs moving between full branches.

**Why it fails:** A full complete branch remains reportable (309–318). Stopping new production at capacity leaves the nonempty reportable object needed by the global snapshot. Because one legal batch must fit a report, finite packing cannot force an empty honest report. If no rotation is selected, that report causes ADVANCE, changing the anchor and freeing depth. The proof needs no guarantee that a specific full branch wins.

**Boundary:** A deployment that prunes all compatible unadopted candidates, silently drops prescribed honest traffic, or sets `B_G` below one legal batch would invalidate a premise used here. The submitted algorithm does not prescribe such pruning or undersized packing. Retaining bytes is separately specified; an implementation must preserve the eligible candidate behavior used at 883–892.

## L4 — Repeated early replacement is blocked by the stated tenure condition

**Candidate:** Restart each global view at a faulty first producer, then rotate before any honest producer obtains a useful slot/report.

**Checked timing:** Let activation epoch be `a`, at time `(a-1)T_G`. Its own snapshot is empty. With no growth, the no-progress counter first reaches `K_G` in epoch `a+K_G-1`; its earliest corresponding epoch start is `(K_G-1)T_G` after activation. A planned rotation is not due until epoch `a+N_G`, at offset `N_GT_G`.

The stated condition is

\[
\min\{K_G,N_G\}T_G>L_*+3T_G.
\]

It implies

\[
(K_G-1)T_G>L_*+2T_G,\qquad
N_GT_G>L_*+3T_G.
\]

Thus both replacement triggers occur after the fixed-anchor progress bound `L_i+2T_G`, including the one-epoch shift for the initially empty report. The activation rule resets `z=0` and `e_(v_G)=a` (634–643). Old counters do not force immediate replacement. This candidate would become real if an implementation weakens tenure or fails to reset the counters, but it is not a break of the frozen rules.

## L5 — Dynamic READY sets cannot cyclically exclude a continuously ready healthy region

**Candidate:** Other regions alternate between ready and unavailable, repeatedly attracting rotations and starving one healthy successor.

**Checked argument:** Once heights are permanently constant, let `j` be a continuously healthy eligible region. Its responsive honest intersection has regional weight greater than two thirds, possesses the unchanged parent prefix, and must set READY on each fresh parent report (180–185; 377–381). Global honest-origin validity preserves those reports.

For a current region `i≠j`, define `d(i,j)∈{1,…,m-1}` to be the forward cyclic distance to `j`. A selected first-ready successor is either `j`, or is strictly before `j` in that scan. In the latter case its forward distance to `j` is strictly smaller. It cannot jump across a continuously ready `j`. A same-region restart is unavailable when `j≠i` is ready. Therefore at most `m-1` such changes reach `j`, regardless of the time-varying readiness of other regions. Once there, the tenure calculation in L4 allows progress.

**Boundary:** A READY quorum promises current possession, not persistent availability (647–651). A selected region may fail before activation. This can delay recovery and force more transitions, but it does not defeat the eventual-continuously-healthy premise or the cyclic argument after readiness stabilizes.

## L6 — Late old data and empty output do not refute the theorem

**Candidate:** A failed or malicious active region releases one valid withheld batch whenever the no-progress timer is about to expire, resetting it forever; alternatively produce only empty mini-blocks.

**Why this does not refute height liveness:** Every reset must accompany an actual adopted-height increase (590–595), and every mini-block contains a new height even if its transaction list is empty. Infinitely many successful resets already imply unbounded adopted height. Finitely many resets have a last one, after which the no-progress trigger fires. In addition, the planned `N_G` trigger is independent of the no-progress counter.

**What can fail:** Useful throughput, fair inclusion, a particular transaction's inclusion, or survival of a particular preview. These are excluded service properties. A bound for outage recovery must explicitly include the late-data tail `L_tail` (1015–1034); the theorem does not offer a uniform bound on that tail without more assumptions.

## L7 — Cold recovery is conditional, not an autonomous synchronizer

The regional recovery rule restores persistent history and retrieves activation/anchor data before rejoining a future slot (426–429). The consumer and regional output rules likewise pause when a required proof or body is missing (509–538). READY requires the entire parent prefix, not merely its digest (377–381).

These are safe admission rules, but they do not establish progress from an arbitrary snapshot lacking a common justified parent. The global protocol begins from a common parent and its proof inducts from genesis using uninterrupted delivery budgets (556–558; 792–806). The model assumes a fixed responsive honest global quorum throughout the execution (180–185), permanent retention of admitted/endorsed/adopted data (187–192), and bounded complete-object retrieval (158–171). Losing these premises, deleting historical keys/signing state, or requiring a recovering node to catch an unbounded moving prefix is outside the proved envelope.

For the stated height theorem, the bounded-height contradiction fixes the parent prefix eventually. A healthy member that is already in `H_G` participates in the continuous global protocol and holds this prefix; the proof does not need to prove arbitrary state transfer under overload. No separate in-model cold-start counterexample was established.

## L8 — Healthy operation can have repeated global-scale live-stream pauses

**Classification:** Concrete permitted all-honest execution, not a liveness-theorem violation. It matters if “live stream” is intended to mean a sustained low-latency cadence of newly signed batches.

Take one region with four always-responsive honest equal-weight nodes. Set

\[
\tau_R=1,\quad \Delta_R=0.1,\quad \Delta_G=1,
\quad T_G=7,\quad N_R=1,\quad N_G=K_G=5,
\quad d_R=b_G=b_R=1.
\]

Assume one legal batch fits the byte limits. All stated timing constraints hold; `L_*=5.1` and `35>5.1+21`, so the minimum tenure also holds. Since there is only one region, the planned timer alone cannot select another region; the no-progress counter below never reaches 5.

| Time | Snapshot / global action | Regional production |
|---|---|---|
| `0`, start epoch 1 | Install genesis; cutoff 0 makes report empty. | Then the first producer emits one batch `X`, filling depth 1. |
| `(0,7)` | `C_1` is HOLD. | Every later producer selects a full one-batch path and emits nothing. |
| `7`, start epoch 2 | Install `C_1`, which retains genesis; snapshot reports `X`. | `X` still occupies the entire speculative depth. No producer has room. |
| `(7,14)` | `C_2` adopts `X` and is justified. Its installation boundary is still time 14. | New batches remain prohibited by the full depth cap. |
| `14`, start epoch 3 | Install adopted anchor `X`; re-rooting makes the report snapshot empty. | Then emit `Y`, filling the one available depth again. |
| `21`, start epoch 4 | Snapshot reports `Y`; `C_4` adopts it later in this epoch. | No new batch before time 28. |
| `28`, start epoch 5 | Install `Y`; snapshot empty. | Emit the next one-batch burst. Repeat. |

Thus batches are emitted at `0,14,28,…`, despite one-second local slots and a healthy 0.1-second regional network. An honest producer may wait for the global adopted anchor even though its own slot uses no regional quorum. Global checkpoints continue every epoch; `z` alternates between 1 and 0; some new data is adopted and finalized indefinitely. Hence this is permitted backpressure, not a deadlock or failure of unbounded height growth.

**Rules producing the pause:** A full path cannot be extended (312–314; 456–461); an epoch-start snapshot excludes the batch produced just after it (359–368; 560–565); a newly justified checkpoint is installed only at the next epoch boundary (531–533; 575–579). These three rules together create the two-epoch cycle in this valid configuration.

**Output-level distinction:** Preview is newly extended only once every `2T_G`. Adoption of those batches occurs in alternate epochs. Finalized delivery follows one epoch after each adoption. Existing bytes remain readable throughout, but the newly produced stream pauses for almost `2T_G` after each burst.

## L9 — A regional outage can remove the global quorum required to authorize failover

**Classification:** Concrete availability failure outside the global-responsive-supermajority premise; a geographic failure threshold that the design's regional-failover description can otherwise obscure.

Let a whole region of weight `W_i` become unavailable. Even if every node outside it is honest and available, their total weight is `W-W_i`. If

\[
W_i\geq W/3,
\]

then the survivors have at most `2W/3`, below the strictly-greater-than-two-thirds certificate threshold. They cannot certify HOLD, ADVANCE, ROTATE, or CONFIRM. A healthy successor's READY quorum does not replace that global certificate.

For example, take three equal-weight regions, each internally entirely honest, and lose the active region. The two surviving regions retain two thirds of total weight, which is insufficient. Their regional links may be healthy and their READY proofs valid, but no finalized handoff can be authorized. At the next required checkpoint boundary their members cannot originate fresh messages from the stale anchor either (535–536). Local slot advancement does not solve the missing global parent.

If Byzantine nodes in surviving regions also withhold votes, an outage smaller than one third of total weight can suffice. In general, for an unavailable region `V_i` and Byzantine survivor set `B_out`, a global honest quorum is impossible whenever

\[
W_i+\operatorname{wt}(B_{\rm out})\geq W/3.
\]

Example: four equal-weight regions of four equal-weight nodes each, with one Byzantine per region. Each regional honest weight is `3/4>2/3`; all 12 honest nodes initially form a global quorum. Lose the active region, and let the three surviving Byzantine nodes withhold. Only 9 of 16 total weight remains responsive and honest, well below the required `>32/3`. This does not contradict the explicit global availability premise: it illustrates why that premise is stronger than per-region honest supermajorities plus a healthy alternate region.

**Recovery distinction:** While the affected region is down, this is ordinary insufficient quorum. If the outage also leaves honest nodes locked onto incompatible target-epoch votes, restoring a quorum does not necessarily recover; L1 gives the stronger permanent-stall mechanism. If no conflicting epoch votes were cast, the manuscript still supplies no general recovery procedure from the skipped required epoch.

## Dependency audit

The positive height proof depends on these independent properties, each checked locally rather than assuming ordinary LMD convergence:

1. Every healthy active region has a responsive honest scheduled producer within a full finite cycle (180–185; 413–425).
2. An empty eligible tree is extended by that producer; a full tree already supplies reportable data (456–461; 309–318).
3. A fixed anchor and unchanged validity allow some complete eligible past-slot candidate to survive until the next aligned snapshot (363–374; 883–892).
4. Honest snapshot reports are nonempty when a candidate exists and survive global broadcast.
5. A nonempty common report tree gives a nonempty selected branch without any local common-head claim (590–595; 760–779).
6. Rotation cannot preempt a freshly activated healthy region before its conservative progress window (902; L4 above).
7. Bounded monotone integer heights eventually become constant; then cyclic readiness reaches a healthy region (921–930; L5 above).
8. Under uninterrupted global progress, each adopted checkpoint is finalized by the following epoch and retained data can be delivered to live consumers (782–812; 841–857).

The most consequential practical boundary is L1: continuous synchrony is needed to preserve the unique justified-parent induction. Partial synchrony after a finite fault is not sufficient for recovery in the submitted state machine.
