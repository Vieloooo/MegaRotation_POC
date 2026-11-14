# Regional proof verification — main_v2, review v1

Verifier mode: the proof-verifier skill was read and applied. This report does not edit or repair the manuscript.

Reviewed source: `formalization/main_v2.tex`, SHA-256 `50d483baef03c60cab036e5b8ea79e8c96da36289b81dc5edb04125d5d02a6e6`. Line references below refer to that snapshot. A temporary review copy is `/tmp/main-v2-regional-review-v1-source.tex`.

## Summary

**Final Verdict:** The regional certificate-comparability and frozen-watermark safety argument is valid under the stated synchronous fixed-session envelope, but the draft as a whole is not yet valid as a complete SMR/evidence specification: one literal slashing predicate has a Critical Error, and deterministic application semantics, proposal scheduling, and the advertised per-view workload bound have Justification Gaps.

No counterexample was found to the regional prefix-agreement theorem under its stated hypotheses. The slashing finding is independent of that theorem.

### Findings

1. **Critical Error — cross-session false-positive slash.** Location: lines 541–542, “same-region, same-local-view PROPOSE or VOTE statements for incomparable consensus branches.” The compared statements need not have the same global generation. F1 at 534–535 verifies a fixed configuration and domains, but does not impose generation equality. An honest identity can sign local view zero in two authorized sessions of one configuration; the session roots differ, so their branches are incomparable. The literal predicate accepts these lawful signatures as slashable evidence. This conflicts with the lawful-view-change qualification at 565–567. Scope: slashing interface, not the fixed-session safety theorem.

2. **Justification Gap — the supplied deterministic transition function is underdefined.** Location: 216–223, “An Append(r) command appends its authenticated bytes at the next position”; 194–198 define deterministic state from a command sequence. The state contains the mini-block sequence/tip, but the command carries only one request. Partitioning two identical ordered requests into one mini-block or two produces different headers, heights, prefix digests and possibly returned positions. No deterministic partitioning/framing rule is part of `T(s,Append(r))`. Identical committed batch prefixes certainly determine identical execution, but that does not yet instantiate the stated request-level state machine. Consequently the full deterministic-state/linearizability claim at 619–620 and 662–663 is incomplete, although ordering safety remains verified.

3. **Justification Gap — timed proposal issuance is used in the progress proof but not imposed by the algorithm.** Location: S1–S2 at 399–407 versus 635–639. The proof starts the first proposal at `A_v` and sends proposal j by `A_v+2(j−1)Δ_R`. S1 does not explicitly state its scheduled send event, and S2 says the leader “may immediately” propose and must stop “sufficiently early to finish their good-case timers.” These are not the complete deterministic/event-driven timing obligations used by the worst-case calculation. The strict safety cutoff is explicit and correct; the liveness calculation is valid conditional on its assumed prompt send rule. The later request-completion theorem inherits this gap.

4. **Justification Gap — N_R does not bound honest work under a Byzantine leader.** Location: 383–407 and 750–752, “The proposal cap N_R ... limit[s] per-view honest work.” The cap is defined for an honest leader; S3–S4 at 408–419 do not bound the number of valid same-branch proposals/QCs an honest member processes or votes for under a Byzantine leader. An arbitrarily fast permitted network can carry arbitrarily many sequential certified descendants inside a fixed active window. The object-size bound is per batch, not per view. Thus the claimed per-view work limit does not follow from the specified admission rule. The per-batch message/bit counts are unaffected; synchronous safety is conditional on the complete-object envelope and is not refuted by this observation.

## Detailed Verification Log

### 1. Model, thresholds and source scope

**Lines 146–163:** all nonfaulty regional members participate throughout the analyzed session; complete prescribed messages and processing meet a known bound; logical schedules are shared. This is a strong standard-synchrony model. Whole-region recovery and arbitrary mobile sluggishness are explicitly excluded, so the proof does not silently assume a fault model more permissive than its hypotheses.

**Lines 179–187:** weighted quorum facts are correct. Two sets of weight greater than `2W_i/3` intersect in weight greater than `W_i/3`, exceeding the total faulty weight. Honest weight itself exceeds `2W_i/3`.

**Lines 372–380:** the Sync HotStuff attribution is appropriately qualified as a standard-core adaptation with a replacement pacemaker and stronger quorum threshold. The paper does not claim that this exact fixed-window construction, its stake adaptation or its theorem was given by the source. Original Sync HotStuff §II is the appropriate source; the chosen `2Δ_R` timer and certified-parent pipeline are consistent with that source. Its all-to-all communication is correctly retained rather than borrowing the linear complexity of partially synchronous HotStuff.

### 2. SMR semantics and requests

**Lines 190–203:** prefix agreement, validity, deterministic state/output consistency, response-after-commit and eventual admitted-request completion are a suitable SMR interface. The text correctly distinguishes request completion from infinite empty-block growth.

**Lines 205–236:** authenticated request identifiers, duplicate suppression, retransmission and finite predecessor sets provide the ingredients for admitted-request liveness. The fair-service assumption explicitly covers the finite union over honest queues. Byzantine producer fairness is not required for safety. Finding 2 concerns the remaining relationship between request commands and batch framing.

**Lines 240–252:** requests remain pending after tentative-branch changes, preventing a request selected on an abandoned branch from disappearing. Execution/output is restricted to committed content. These are correct obligations.

### 3. Attempts, signed payloads and ancestry

**Lines 270–279:** the nested Cantor pairing is injective for nonnegative arguments. With region indices starting at one, local views starting at zero and batch positions starting at one, `u` identifies `(i,v,k)`. A replacement at the same batch position in a new local view gets a distinct attempt. Reusing the original ORDER/PROPOSE/VOTE/DECIDED comparison domains therefore does not itself slash lawful within-session recovery.

**Lines 284–307:** the semantic proposal includes the instance and explicit local view, certified parent reference and output tips; the instance determines region and batch height. Although the VOTE signs `Hash(x)` rather than the entire redundant envelope, validity fixes all the envelope fields from the signed data and pairing. No unsigned mutable consensus field was identified.

Certified ancestry and increasing batch height support finite parent-chain traversal. Current-view message restrictions at 437–438, shared time and the honest signer in each QC exclude a reachable future-view parent certificate. The proof would be clearer if the parent-view relation were explicit, but its required temporal fact follows in a genuine execution with unforgeable signatures.

**Lines 311–321:** output records identify the immutable accepted block and distinguish a vote QC from an attestation that a regional timer-based commitment occurred. The changed record format and loss of numeric `u−1` parent adjacency are explicit.

### 4. Active protocol and boundaries

**Lines 386–393:** `F_v<A_{v+1}` with a full drain delay before the snapshot. Strict commitment before `F_v` is essential. Evidence is processed before freeze/timer ties, and no decision is permitted at the freeze instant. These details make the later strict inequality `t+Δ_R<S_v` correct.

**S3, 408–415:** a signer persists only one vote per `(v,k)` and stays on a comparable branch within the entire view. The rule is stronger than same-height uniqueness, exactly as the comparable-certificates proof requires. A fresh current-view QC may exceed the frozen entry rank, but its own earliest current-view ancestor had to obtain honest votes under the same guard.

**S4 and S6, 416–430:** any direct local commitment requires the actual complete QC. That certificate has already been multicast by the honest committer (on its first local collection). Thus the handover proof does not rely on the external observer reconstructing a private “absence of equivocation” observation. Certificate encodings are deduplicated by semantic block. An expired/canceled timer does not restart merely because a late QC arrives.

**S5, 420–424:** authenticated producer equivocation cancels pending timers but leaves existing commitments intact. With an honest leader, no such conflicting producer signature can be created under the signature hypothesis.

**H1–H4, 449–464:** all old voting/timers stop at the common boundary; statuses use frozen certificates after the drain; the next honest leader receives every honest status after another full delay. Ignoring late older-view certificates as *current-view voting guards* prevents an adversary from changing recovery requirements after status collection, while retaining the proof for a future snapshot preserves evidence. This is a fully stated protocol rule, not an assumption that Byzantine holders publish promptly.

### 5. Regional safety proof

**Lemma at 575–582:** valid. Any two incomparable same-view QCs intersect at an honest member, contrary to S3. Since all consensus edges increase batch height, equal view/height plus comparability implies the same block.

**Lemma at 584–604:** valid for the stated fixed-session model. If an honest member commits B at `t<F_v`, every honest member knows its QC before `S_v`. The maximum stored certificate cannot have a lower rank. Same-view comparability handles higher-height QCs. For a later-view QC, at least one honest signer enforces the frozen-rank guard; following certified parents to the first block of that view supplies an older-view parent whose rank already dominates the protected certificate. Induction on views gives extension of B. The retained highest state prevents a later snapshot from lowering the protected rank.

The proof relies on the full-QC commitment guard, all-honest multicast delivery before the snapshot, comparable voting, and preservation of old certified state. Each is actually stated. The same argument would not automatically apply to asynchronous recovery, arbitrary key loss or a different unproved pacemaker, and the draft does not claim that it does.

**Theorem at 606–621:** regional batch-prefix safety follows from the two verified lemmas. Same-view direct commits are comparable; later-view direct commits extend earlier ones; ancestor commits inherit that relation. Mini-block integrity follows from the canonical contents, signed authority and hash ancestry. The deterministic application-state portion remains subject to Finding 2.

### 6. Liveness and regional evidence

**Lemma at 623–640:** the arithmetic is correct assuming the send times used by the proof. Quorum replies take at most `2Δ_R`; recipients vote within one delay and their timers expire by `A_v+(2j+1)Δ_R`, strictly before `A_v+(2N_R+4)Δ_R`. The frozen maximum reported to an honest leader meets every honest member's rank guard and preserves their committed prefix. Finding 3 is the missing specification-to-proof timing link.

**Theorem at 642–664:** conditional on an honest view actually executing the prompt proposal schedule and on well-defined batch application semantics, the finite-predecessor argument is sound. A selected ancestry containing r commits r with its child; otherwise fair honest proposals reduce the finite set ahead of r. Pending requests are not dropped on a tentative reorganization. Honest recurring views eventually provide the quorum of DECIDED endorsements. Real-time order follows when a new correct-client request is submitted only at invocation and responses require a committed prefix; request identifiers suppress retransmission execution.

**Proposition at 666–675:** regional evidence soundness follows independently of the false-positive slash finding. Every valid DECIDED quorum has an honest signer; E1 permits that signature only for a locally committed block. The verified prefix-safety argument excludes conflicting certified outputs within one session.

### 7. Costs and outer abstraction

**Lines 726–752:** proposal-to-regional latency `δ_R+2Δ_R`, certificate delivery to P plus `δ_P`, and pipelined interval approximately `2δ_R` are correctly distinguished. Raw vote transmission count is quadratic; unaggregated all-holder QC dissemination can be cubic in signature bits. STATUS is one send per member with an unaggregated certificate and required data. Finding 4 concerns only the additional asserted N_R-based cap under Byzantine leaders.

**Lines 677–724, 754–759:** the outer layer is clearly unfinished. Local DECIDED evidence does not claim outer commitment or survival of a whole-region loss. Durable outer state/output rules and the crash-only refinement boundary are explicit future obligations. No completed CFT theorem is smuggled into the regional proof.
