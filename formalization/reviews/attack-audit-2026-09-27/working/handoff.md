# Independent audit: handoff, consumer output, readiness, and replay

Source frozen at `source/main.tex`, SHA-256 `325ab6a05c0fd903f0f999a1d474218d122a302a54aa9c65a744a5dc989a4223`. Line references below refer to that frozen file. This pass did not use historical reviews or modify the proposed protocol.

## Result and exact target

**No unconditional attack on the handoff or consumer guarantees was found under the stated model in this pass.** The strongest practical hazards are already excluded by unusually strong premises: common clock; delivery and verification of complete admitted objects within the configured bounds; a fixed, continuously responsive honest global supermajority; persistent signing history; permanent retention for the configuration; and fail-closed consumer/producer behavior when a due checkpoint is missing. Removing any of these changes the problem being proved.

The relevant promised properties are:

1. Every accepted branch has complete, consecutively linked bytes signed by the producers authorized for its slots ([241–243](../source/main.tex#L241)).
2. Honest consumers deliver prefix-compatible, append-only logs, each position once; previews have no corresponding agreement or monotonicity guarantee ([244–246](../source/main.tex#L244)).
3. A finalized handoff preserves every **globally adopted** entry; current speculative data uses the current global view, while historical signatures remain verifiable ([247–249](../source/main.tex#L247)).
4. Height and delivery grow under the additional envelope, workload, tenure, and eventual healthy-region conditions; no individual preview/batch/transaction persistence is promised ([250–253](../source/main.tex#L250)).

The target is a proposed sequencing interface, not EVM correctness, DA publication, fairness, an arbitrary-partition consensus service, or accountability for silence. See [1138–1148](../source/main.tex#L1138).

## Precise assumptions used to reject candidate attacks

| Premise | Source | Why it matters here |
|---|---|---|
| Fixed Byzantine set, less than one-third weight in every region; historical honest keys remain uncompromised | [174–178](../source/main.tex#L174) | Prevents a false READY quorum made entirely of Byzantine holders and prevents long-range signature forgery. |
| Persistent signing history, including restart | [177–178](../source/main.tex#L177), [403–407](../source/main.tex#L403) | A crash cannot reset same-slot or same-epoch anti-equivocation rules. |
| A single honest global set of weight greater than two-thirds stays responsive throughout the analyzed execution | [180–185](../source/main.tex#L180) | This is stronger than merely having a fresh quorum at each instant; finalized/adopted data remains served by this set. |
| Complete-object bounds include ancestry, queueing, verification, concurrent traffic, and installation margin | [157–171](../source/main.tex#L157) | Prevents withholding a required body after a digest has already been counted as timely. |
| Honest endorsers and global report acceptors retain and serve data for the whole configuration | [187–193](../source/main.tex#L187) | Fork-choice pruning is not data deletion; discarded local branches do not imply loss of an adopted prefix. |
| Consumers verify exact branch/range identifiers and complete content | [395–401](../source/main.tex#L395), [501–527](../source/main.tex#L501) | Prevents a supplier from satisfying a request with a different fork at the same height. |
| Missing due proof or data pauses current-authority output; replay cannot roll back the anchor | [511–527](../source/main.tex#L511), [531–539](../source/main.tex#L531) | Rejects stale-view preview and catch-up rollback attacks if implemented literally. |

## S1. A brief global deadline violation can permanently freeze every region

**Classification:** permanent deadlock after a timing-premise violation; not an in-model liveness refutation. This is more severe than simply waiting for the network to recover.

Use four nodes and two regions: the active region has honest `H1,H2` of weight one each and Byzantine `B` of weight `0.4`; the inactive region contains honest `H3` of weight one. Each region has more than two-thirds honest weight. Total weight is `3.4`, so a global certificate needs weight greater than `34/15 = 2.266...`. Local active-region traffic can remain timely throughout the trace.

1. All members start epoch `e` from the same justified parent. `H3` originates its valid REPORT `r3` and self-delivers it immediately.
2. Delay the round-one `r3` packets to `H1,H2` until `Delta_G + epsilon`. All other traffic can be timely; `B` supplies no useful forwarding. This is the deliberately violated premise.
3. `H1,H2` ignore the late length-one chain: the packet had to be complete by the round-one deadline. `H3` has already marked its own value forwarded and never sends it a second time under this invocation ([1189–1204](../source/main.tex#L1189)). Later timely rounds do not repair the missed acceptance.
4. `H3` returns `r3` for that origin, while `H1,H2` return bottom. Other origins can have identical outputs. The vectors, and hence their checkpoint hashes, differ even if their selected stream tip and active region happen to agree ([336–342](../source/main.tex#L336)).
5. `H1,H2` sign checkpoint `C` with total weight two; `H3` signs different checkpoint `C'` with weight one. `B` withholds. Neither target can receive a global certificate.
6. After healing, the honest validators cannot retarget epoch `e`: they persist at most one FFG statement per target epoch ([625–628](../source/main.tex#L625)). There is no justified `C_e` from which to start the adjacent epoch `e+1` ([556–579](../source/main.tex#L556)).
7. At the next boundary, all regions fail closed because their required checkpoint is missing. No batch, HEAD, or report may be originated from a stale anchor ([531–539](../source/main.tex#L531)). A rotation is blocked by the same global dependency. Retrying retrieval cannot fetch a certificate that does not exist.

Thus an arbitrarily small excess beyond one required delivery deadline can create an unbounded halt after network health has returned. The text explicitly states that recovery from arbitrary global epoch stalls is unspecified ([1145–1148](../source/main.tex#L1145)). The user-facing finding should emphasize the stall and the exact assumption crossed, rather than imply that FFG safety has failed: no conflicting certificate forms.

An equivalent equal-weight, one-region trace is in `working/liveness_traces.py`: a Byzantine REPORT origin gives a value to one honest receiver, and that receiver's sole length-two multicast is delayed past its deadline for the other two honest receivers. The honest FFG weights split `1/4` versus `1/2`, with the Byzantine quarter withholding.

## S2. Short tenure plus reset-to-first-producer gives perpetual silent rotation

**Classification:** an executable starvation trace when the configured tenure condition is omitted or undersized. All synchrony, quorum, and honest-readiness conditions can hold. This is outside the theorem because it deliberately violates [901–903](../source/main.tex#L901).

Use two regions `V_i = {B_i,H_i}`. In each region, `B_i` is Byzantine with weight one, `H_i` is honest with weight three, and the configured producer list is `[B_i,H_i]`. Both honest nodes remain responsive and both regions remain healthy. Consequently `H_i` alone supplies regional READY weight `3 > 8/3`, and `H1,H2` together supply global voting weight `6 > 16/3`. Byzantine nodes need not send any messages.

Choose the concrete parameters:

| Parameter | Value |
|---|---:|
| `tau_R` | 1 second |
| `Delta_R`, `Delta_G` | 0.1 second each |
| `N_R` | 10 slots per producer term |
| `T_G` | 2 seconds |
| `N_G` | 1 epoch |
| `K_G` | 100 epochs |

The regional inequality is `1 > 2(0.1)`. With four total nodes the global inequality is `2 >= (4+3)(0.1) = 0.7`; epoch boundaries align with slots. Give all legal batch/report limits sufficient capacity, although no batch is ever produced in this trace.

For every activation epoch `a`:

| Epoch | Effective action and stream state |
|---|---|
| `a`, at time `t` | Activate region `i`, reset the producer list to `B_i`, report the empty branch at cutoff zero, and certify HOLD. Both regions are READY. |
| `a+1`, at `t+2` | The planned timer reaches `e-e_g=1`, so choose the other ready region and certify ROTATE. The adopted tip stays unchanged. |
| `a+2`, at `t+4` | Certify CONFIRM and finalize the ROTATE. |
| `a+3`, at `t+6` | Activate the other region and reset its producer list to its Byzantine first member. Repeat. |

Each active residence is only `3T_G = 6` seconds. The first Byzantine producer's term lasts `N_R tau_R = 10` seconds. It stays silent, and each activation happens before any honest producer can obtain its first term. There are forever no mini-blocks, growing previews, adopted height increments, or new delivered entries, although the global checkpoint chain keeps finalizing and every readiness quorum is genuine.

The deliberately failed condition is numerically explicit:

`min(K_G,N_G) T_G = 2`, but `L_* + 3T_G = ((2*10+1)*1 + 0.1) + 6 = 27.1`.

This demonstrates why tenure cannot be inferred merely from honest voting weight or from successful geographic switching. It also shows that a fixed first producer makes repeated handoff/reset a starvation mechanism under bad parameters.

## S3. Honest weight does not bound the wait for an honest producer

**Classification:** arbitrarily long finite live-stream silence across allowed configurations; no contradiction to the stated `n_i`-dependent bound.

A region can contain one responsive honest identity of weight `4/5` and one hundred Byzantine identities sharing weight `1/5`. Place all Byzantine identities before the honest identity in the configured cyclic list. Set `N_R=100`, `tau_R=0.03` seconds, `Delta_R=0.01` seconds, `Delta_G=0.1` seconds, `T_G=10.5` seconds, and `K_G=N_G=40`. The first hundred producer terms can all be silent, yielding exactly `100*100*0.03 = 300` seconds with no batch before the honest producer gets its turn.

All displayed conditions hold: `T_G >= (101+3)*0.1 = 10.4`; `T_G/tau_R = 350`; `L_i = 303.04`; and `40*10.5 = 420 > L_i + 3T_G = 334.54`. The honest identity alone can keep the global chain alive and meet every readiness threshold. Adding more tiny-weight Byzantine identities and scaling valid tenure parameters makes the finite silence arbitrarily large while Byzantine weight stays fixed below one-third.

This is the identity-based producer schedule at [413–428](../source/main.tex#L413) interacting with stake-weighted honesty. The source's bound already depends on the full identity count ([860–868](../source/main.tex#L860)); an operational promise of fast recovery based only on the honest-weight fraction would not follow from it.

## H1. Activate a successor using an unfinalized ROTATE

**Attempt.** An adversary obtains a justified `ROTATE` checkpoint `C_r`, withholds the following finality link, and tells successor region `j` to begin global view `g+1`. Meanwhile the old region continues global view `g`. The adversary tries to have honest consumers accept two current authorities.

**Why it fails.** A new global view needs finalized control authorization ([141–145](../source/main.tex#L141)). Activation requires the specific adjacent link `C_r => C_{r+1}` and occurs only at the beginning of epoch `r+2`, time `(r+1)T_G` ([634–643](../source/main.tex#L634)). A justified `C_r` by itself is insufficient. A pre-boundary successor batch lacks an effective activation; an honest node missing the finality proof cannot start ([636–637](../source/main.tex#L636)). The epoch/slot alignment ensures activation is also a regional slot boundary ([137–139](../source/main.tex#L137)).

**Status:** rejected under the literal verification rules. An implementation that treats justification of ROTATE as activation would break the protocol.

## H2. Use a READY certificate for one prefix, then make the successor inherit a different prefix

**Attempt.** At the start of epoch `r`, successor members sign READY for parent tip `a`. During the same epoch, active-region reports reveal an extension `a -> b`. If the checkpoint both adopted `b` and rotated using the READY certificate for `a`, a cold successor could begin without `b`; an implementation might silently drop `b` or be unable to start.

**Why it fails.** READY binds the exact parent, next global view, destination region, height, and digest ([377–386](../source/main.tex#L377)). When a successor is selected, `ROTATE` explicitly retains the parent tip `a` and adopts no range from the just-computed snapshot ([601–608](../source/main.tex#L601)). `CONFIRM` also adopts no new data ([587–589](../source/main.tex#L587)). Thus the successor inherits exactly the prefix whose possession was promised, and every earlier adopted entry is included in it.

This rule intentionally sacrifices the candidate extension `b`. Its exclusion is not loss of an adopted entry because `b` never became adopted. The worked example states this distinction ([653–661](../source/main.tex#L653)).

**Status:** rejected. Useful regression case: construct nonempty report paths in a ROTATE epoch and assert that the resulting adopted tip remains the parent tip.

## H3. READY quorum signs, then all its useful holders disappear

**Attempt.** Byzantine members lie about having data. Enough honest members genuinely possess the parent and sign READY, but they crash before activation. The selected region has a valid readiness certificate but cannot produce.

**What the certificate actually establishes.** For a region of weight `W_j`, let `Q_j` be READY weight greater than `2W_j/3` and let Byzantine weight be less than `W_j/3`. Then

`wt(Q_j \ B) > W_j/3`.

Consequently at least some honest READY signers possessed and retained the prefix when signing. This proves neither a responsive regional supermajority at activation nor availability of a particular selected producer. The draft explicitly states this limitation ([647–651](../source/main.tex#L647)).

**Why this does not break the stated progress theorem.** Adopted data is also held by the continuously responsive honest global supermajority, independently of the selected region ([842–857](../source/main.tex#L842)). Global epochs continue, detect non-growth, and can select another ready region. The theorem additionally requires an eventual continuously healthy eligible region and sufficient tenure ([899–932](../source/main.tex#L899)). The quoted outage bound only applies when the selected successor is healthy ([1015–1034](../source/main.tex#L1015)).

**Status:** real operational failure episode, expressly modeled and recovered; not a safety attack or a counterexample to the conditional theorem. A claim that READY alone guarantees activation-time availability would be false.

## H4. Toggle readiness to skip a continuously healthy region forever

**Attempt.** Byzantine behavior and intermittent failures make different unhealthy regions ready at strategically chosen epochs. Each is selected, becomes unavailable, and a new unhealthy region is selected before the continuously healthy region can be reached.

**Why it fails under the actual selection rule.** Fix a continuously ready target region `j*` and a current region `i != j*`. Let `d(i,j*)` be the forward distance from `i` to `j*` in the finite cyclic region list. At a due transition, the first ready region after `i` has step size `s` satisfying `1 <= s <= d(i,j*)`, because `j*` itself is ready. If it is not `j*`, the remaining distance is `d(i,j*) - s`, strictly smaller. Thus at most `m-1` intervening selections can precede `j*`; a same-region restart cannot occur while a different continuously ready region exists ([596–608](../source/main.tex#L596)).

If adopted height is forever constant, the no-progress trigger eventually fires; late reports can keep resetting the trigger only by increasing height, which already supplies the theorem's weak progress goal. Once `j*` activates, the tenure inequality protects a sufficiently long production/adoption interval ([901–903](../source/main.tex#L901)).

**Status:** rejected. The finite-ring potential argument supports the fairness sentence in the theorem proof.

## H5. Old global-view producer reuses a height after handoff

**Attempt.** Let old view `g` preview batch `X` at height `h+1`. After handoff, new view `g+1` produces different `Y` at the same height. An attacker serves `X` in place of `Y`, replays an old HEAD, or tries to turn the old preview into a delivered entry.

**Why it fails.** Mini-block headers bind configuration, global/local view, slot, within-batch index, height, parent digest, and transaction-list commitment ([263–288](../source/main.tex#L263)). Current branches require all new batches to belong to the effective global view ([291–296](../source/main.tex#L291)). HEAD domains include the global view and only active-region members contribute weight ([299–307](../source/main.tex#L299)). Retrieval objects bind an exact branch/range rather than height alone ([395–401](../source/main.tex#L395)). Consumers process all intervening activations and identify old preview entries by digest and view ([517–527](../source/main.tex#L517)).

Old batches remain valid historical evidence, but they are not eligible children of the new view's current anchor ([462–465](../source/main.tex#L462), [679–681](../source/main.tex#L679)).

**Status:** rejected. Merely using a height-indexed cache would violate an explicit rule.

## H6. Catch-up installs an old checkpoint and rolls back delivered output

**Attempt.** A consumer first delivers through finalized checkpoint `C_b`. An attacker later supplies authentic older checkpoint `C_a`, with `a < b`, or sends a replayed activation sequence, hoping to reset the speculative anchor, global view, or the delivered-position watermark.

**Why it fails.** The consumer accepts an extending control proof, explicitly refuses a replayed ancestor rollback, preserves delivered entries, and installs due anchors/activations before appending newly covered data ([511–527](../source/main.tex#L511)). Restart retains consumer log, anchor, global chain, and signing/history state ([403–407](../source/main.tex#L403)). New global views reset local slot domains, not the existing checkpoint/signing histories ([638–643](../source/main.tex#L638)).

The relevant invariant is

`delivered_prefix <= installed_anchor_prefix <= unique_global_control_history`.

Installing an extending anchor preserves it. SetPreview can only change data above the installed anchor. Deliver first installs a suitable already-due extending anchor and then appends previously undelivered entries in order. A simple induction therefore prevents both rollback and a preview conflicting with delivered output.

**Status:** rejected under persistent state and verification rules. Restoring an old disk snapshot while treating it as current persistent state falls outside the stated persistence premise.

## H7. Stale proof attacks a consumer that has not heard about a handoff

**Trace for an unsafe implementation.**

1. Consumer `P` has an authentic old checkpoint and activation for view `g`.
2. A handoff finalizes elsewhere and its activation boundary passes.
3. The old producer supplies a correctly signed view-`g` branch and withholds the newer control proof.
4. If `P` merely asks whether its last-known activation verifies, it can emit an old-view preview after authority has changed.

This is the usual distinction between **authentic history** and **fresh authority**. Nothing in an old signature self-revokes it.

**Why this is not an unconditional attack on the literal draft.** D4 says missing proof/data pauses current-authority output ([511–516](../source/main.tex#L511)); the epoch-installation rules require the due parent and forbid stale-anchor origination ([531–539](../source/main.tex#L531)). A common clock and known epoch schedule let an implementation determine that an epoch's required checkpoint is missing even before learning its contents. The protocol also expressly disclaims an isolated old producer's stale previews to stale clients as an extension of the newly accepted control history ([537–539](../source/main.tex#L537)).

**Implementation obligation worth making mechanically explicit:** define a predicate such as `CurrentAnchor(P,t)` that requires the prescribed justified parent for the epoch containing `t`, all due activations, and complete adopted bytes before `SetPreview` or new current-authority origination. Testing only “the latest proof I happen to know is valid” does not implement D4. This predicate should apply to advisory previews as well as locally computed LMD previews.

**Status:** assumption-sensitive implementation hazard / possible specification clarification, not a demonstrated in-model theorem failure. During a control outage, output must pause rather than claim both freshness and availability.

## H8. Former local producer backdates a newly signed batch

**Executable adversarial behavior allowed by the validation predicate.**

1. In active global view `g`, Byzantine producer `b` owns slot `u` and publishes nothing in that slot.
2. A later producer uses slot `u+1`, extending anchor `a` with some speculative branch `X`.
3. While `g` remains active, `b` newly signs a batch `Y` extending `a` but labels it with its earlier authorized slot `u`.
4. `Y` has the correct signatures, parent, increasing slot sequence, and size. It can be admitted or reported after slot `u` has passed. Suitable HEAD/report support can make it replace speculative `X`.

The verifier authenticates the producer assigned to the **claimed slot**, not the physical time at which the private key was used. `ValidBatch` has no externally authenticated signing-time predicate ([283–296](../source/main.tex#L283)). The honest recovering-node rule forbids retroactive signing ([426–429](../source/main.tex#L426)), but Byzantine behavior need not follow that rule. No observer can distinguish a batch signed late from one signed on time and withheld, using these bytes alone.

**Why this does not currently violate the service definition.** The service permits multiple valid forks and reorgable previews ([224–236](../source/main.tex#L224), [699–715](../source/main.tex#L699)). The batch's producer really was assigned slot `u`. It cannot conflict with an installed adopted anchor, impersonate the new producer for slot `u+1`, or be admitted as new current-view data after a geographic handoff. Thus “loses its term” is a limit on which slot labels the identity is authorized for, not cryptographic erasure of its ability to release a historical-slot fork.

**Status:** real admissible behavior, outside the protected goal; useful to explain explicitly. Any stronger claim that an old local producer becomes unable to introduce *any* new valid object after its term would require additional mechanisms or a narrower acceptance window and a revised proof.

## H9. Supplier equivocates about data, equivocating HEAD becomes resurrected, or signature witnesses create false conflicts

These related candidates were checked:

- An exact GET for one fork cannot be satisfied by the same heights from a different fork: IDs bind exact branch/range and receivers check original evidence ([395–401](../source/main.tex#L395)).
- A newer missing-data or incompatible HEAD does not revive a convenient old vote ([320–326](../source/main.tex#L320)). A same-slot double-HEAD receives zero local weight until a newer unambiguous slot; restoration later is an explicit rule, not a failure of equivocation detection.
- Different signature encodings and signer-set witnesses do not create different semantic object identifiers ([152–155](../source/main.tex#L152)); authenticated-broadcast witness variants are likewise not distinct semantic values ([1178–1182](../source/main.tex#L1178)).
- The evidence claim is **soundness of an accepted pair**, not completeness of catching every offense. Withholding one half of a conflict, reorging in different slots, silence, missing DATA, or forwarding an origin's two conflicting AB values is not a counterexample to the stated proposition ([664–682](../source/main.tex#L664), [935–945](../source/main.tex#L935)).
- A valid signature alone does not prove body availability. The transport lemma explicitly makes speculative repair conditional on a reachable holder; globally adopted data obtains the stronger global retention guarantee ([842–857](../source/main.tex#L842)).

**Status:** all rejected as attacks on the stated guarantees.

## Review handoff

1. Report the absence of a found in-model handoff/consumer break honestly; do not turn explicitly permitted preview reorgs into high-severity safety findings.
2. Prioritize S1 as the global-to-local permanent stall dependency, while stating that its trace crosses the continuous timing premise. S2 and S3 separate bad-configuration perpetual starvation from allowed long finite silence.
3. Include H8 as a worked “what authorization does and does not mean” attack trace: the claimed slot is authenticated, signing time is not.
4. Include H3 and H4 together to explain why READY is a possession promise and why the liveness theorem still needs eventual sustained health, global retention, and tenure.
5. H1, H2, H5, and H6 explain why obvious handoff/replay attacks fail on the frozen text. No protocol modification was made or is required by this review.

This pass is an independent symbolic audit and direct source review, not a full formal verification or a claim to enumerate every possible implementation vulnerability.
