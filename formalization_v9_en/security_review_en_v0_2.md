# v9 Protocol Security Review (v0.2): Core Counterexamples and Open Questions

Date: 2026-11-03. Previous version: [v0.1](security_review_en_v0_1.md). Scope: the v0.1 specifications 00–09; line numbers cited below refer to the Chinese source; [calibration pipelines v0.2](10_pipelines_en_v0_2.md) are illustrative examples, not new protocol rules.

## 0. Changes in this version

- **v0.1 items retained verbatim.** Sections 1–11 (reading guide, calibration boundaries, S1–S9) reproduce v0.1. Entries affected by the second round receive only a “v0.2 note” in §13; the original text is unchanged.
- **Second-round findings.** Four subagents (consensus and handoff, GFR, certification and derivation, recovery and governance) continued checking after reading v0.1. The lead agent checked each against its cited provisions; the results are S10–S27 (§12). Overlaps with v0.1 record only new material and identify the relationship.
- **New sections.** Section 14 summarizes two cross-cutting repair proposals; §15 lists statements in pipelines v0.2 conflicting with new findings (errata, without changing the original); §16 gives a review order covering all entries; Appendix A maps findings to sources and merged items.
- **Baseline.** After v0.1 recorded hashes, [00 §1.2](00_arch_en_v0_1.md) deleted two passages: the sentence saying that contract state and event lists are uniquely determined by the prefix, and a paragraph explaining prefix comparability. 01–09 are unchanged. This version checks current files; hashes are in the [v0.2 baseline](checks/security_review_v0_2_baseline.json). The deletion does not change any conclusion here; however, S1 and the argument for determinism of G both use the property that the same finalized L1 prefix gives the same contract state. Please confirm the deletion was not intended to abandon it.

This report does not classify “a candidate regional result is ultimately not selected” or “RESUME replaces a suffix of G not yet derived unchanged” as BFT safety vulnerabilities in themselves; both are permitted by the current design. The priorities are whether the same public history has a unique interpretation, whether source proofs support the next module's conclusions, and whether the liveness premises can all be satisfied. (This paragraph reproduces v0.1.)

## 1. Reading guide and baseline

Each finding gives triggering capability, relevant rules, a minimal trace, impact, repair options, and items for manual review. **High priority** means a potential failure of consistent public interpretation or cross-module guarantees, or a prolonged stall; **medium priority** means local liveness, a formalization gap, or a claim requiring explicit qualification. These are revision priorities, not CVSS scores for vulnerabilities in deployed code.

Statuses are **confirmed** (the text directly implies a counterexample or gap), **conditional** (holds under stated conditions, without implementation evidence), and **not an in-model vulnerability** (allowed semantics or violation of an explicit assumption). Where a transition is undefined, one can confirm only that the current specification does not uniquely determine the outcome, not that an existing implementation actually forks.

SHA-256 hashes of 00–09 and the old pipelines v0.1 are in the [review baseline](checks/security_review_baseline.json). This round did not write repair options back into the protocol rules. Native comparisons use only files actually read in the repository's `megaeth_specs`; they do not establish the deployed version.

## 2. Boundaries confirmed in calibration

[Pipelines v0.2 §2–§9](10_pipelines_en_v0_2.md) corrected the following examples affecting security analysis:

1. **Separate ledger position from real time.** An Espresso timestamp can precede the real time at which it becomes finally readable; a RESUME ordering key is not the time of Ready or first block production. The 19.2 min is this example's L1 finalization-plus-reading assumption, not an unconditional bound derived from Ethereum slot/epoch lengths.
2. **Check all three deadlines.** A regional result is subject first to the session public deadline, then per-height Expire; native deadlines also depend on the particular origin, channel completion position, and actual derived parent epoch. Ten hours is not the waiting time available to every regional result.
3. **Native window boundary.** Only `epoch_num + sequence_window_size < inclusion_block_number` triggers window expiry; equality alone does not. Fallback at a missing height uses the actual derived state, not the origin claimed by an unadopted regional block.
4. **Acceptance is not execution.** An unfinalized acceptance receipt is not yet in the finalized acceptance view here. An accepted late batch may still be discarded by derivation; cursor height and consistency of derived content are separate matters.
5. **A parent-chain fork does not mean all later batches are discarded.** Local native certified derivation ignores batch parent_hash and continues assessing later contents on the actual derived parent state; results produced on an old parent need not execute unchanged, but a different parent_hash alone does not show that all are discarded.
6. **Recovery work depends on the cut.** The padding count is $\max\{0,h_{\rm res}-h_D^-\}$, where $h_D^-$ is the derived tip before the relevant override. 44,700 applies only to the cut selected in the draft; completion in 48 s needs about 931.25 blocks/s, and in 5 min needs 149 blocks/s. Neither has been measured.

The arithmetic script checks these numerical relationships only; it is not a protocol implementation or formal verifier. Design changes below are suggestions for your decision.

## 3. S1: Missing explicit boundary between pre-read publications and replay position

**High priority; specification gap confirmed, divergent trace depends on an implementation interpretation that must be ruled out; medium confidence.** Proposed by the GFR subagent and retained after the lead agent checked the acceptance conditions.

Location: [09 §3.2](09_finality_rules_en_v0_1.md) line 102 defines the publication index over “public history read in full”; [09 §4.4](09_finality_rules_en_v0_1.md) lines 263–274 list acceptance conditions but do not explicitly require `first_complete_q` to be no later than the **currently processed** Espresso height. The read cache may be ahead of pc; these prefixes must be distinguished.

Use small parameters $\mathit{ts}^*(h)=h$, $W_{\rm exp}=10$, current session window [91,100], $T_{\rm grace}=10$, so the public deadline is 111; $G$ reaches 94.

| Same public log | Reader A: processed prefix only | Reader B: pre-indexes a longer downloaded prefix |
| --- | --- | --- |
| $J_q$ has key 100 and no messages | No extension | Has downloaded complete Res(95) in $J_{q+1}$ |
| Process $J_q$ | $G$ remains at 94 | Checks `100 < 95+10` using current key 100 and `106 < 111` using first public key 106; both pass and 95 is appended early |
| Next $J_{q+1}$ has key 106; Res(95) first becomes fully public | First executes Expire(95) at key 105, HALT at 94 | First executes Expire(96) at key 106, HALT at 95 |

No forged regional proof is needed: B uses a genuine object from a future event prematurely. If the author has defined “process in event order” to prohibit such indexing, B is illegal; the interface should still state that premise so consumers do not fill it in differently.

**Suggestion.** Retain the pre-read cache but separately maintain a processed Espresso frontier; while processing $J_q$, use only results fully public by $q$. Non-Espresso events should also read that frontier, not the end of the download cache. Restrict by height/message position, not timestamp alone, because Espresso timestamps can be equal.

**Manual confirmation:** [ ] State that the index updates only with events; or [ ] separate the cache from the admissible frontier.

## 4. S2: Certification and native takeover use different prev values after channel timeout

**High priority; cross-specification interface conflict confirmed; high confidence.** Proposed by the certification and derivation subagent. This applies if v9 inherits the native channel timeout rules in the current repository; the deployed adapter has not been verified.

Location: [06 CertifyBatch](06_certifier_en_v0_1.md) line 97 requires prev to equal the current cursor read; [07 SubmitBatch](07_batch_inbox_en_v0_1.md) line 141 likewise requires exact equality. Native [batch-inbox Acceptance](../megaeth_specs/specs/block-certification/batch-inbox.md#acceptance) lines 239–251 **temporarily computes** `expected.frames_digest = 0` after channel timeout; line 430 requires a successor to start a new channel using the zeroed prev without first zeroing the stored cursor.

Minimal trace: an unclosed channel is accepted at L1 height $O$, with stored cursor $c=(h,o,f,source)$, $f\ne0$; the primary Batcher then stops. At $O+51$ (timeout 50 L1 blocks in this example), when backup submission permission has opened, an honest successor tries to start frame 0:

- With $c_0=(h,o,0,source)$, native Inbox accepts continuity but v9 Certifier fails `prev == stored cursor`.
- With $c$, the Certifier can pass the first check but native Inbox fails `prev == expected`.

All services may be online, yet the normal takeover path has no prev accepted by both. RESUME provides a separate governance recovery path, but cannot replace the claimed inherited automatic timeout takeover. This does not refute BAT-4 under A-DELIVER; rather, the implementation must explain how it satisfies A-DELIVER.

**Suggestion.** Distinguish stored cursor from effective cursor computed at a verified L1 height, and put channel_open_block, timeout, and zeroing in the shared interface. Certification and contract should check the same effective cursor, including the race where timeout occurs after certification but before inclusion. Alternatively define an explicit on-chain reset transition. Both are design choices.

**Manual confirmation:** [ ] v9 inherits native timeout and adds adaptation; or [ ] v9 uses different reset semantics and separately establishes takeover liveness.

## 5. S3: CERT-4 lacks an induction base after CursorSet

**Medium priority; property false under its original quantifiers; high confidence.** Proposed by the certification and derivation subagent and rechecked by the lead agent. No role must be corrupted.

Location: [06 CERT-4](06_certifier_en_v0_1.md) lines 130–136; [07 acceptance view](07_batch_inbox_en_v0_1.md) line 171.

1. Old $G$ reaches $h$; a legal RESUME targets $N>h$.
2. CursorSet is finalized on L1, and derivation has obtained the relevant cut, so the reader sets $G'=D_L[0:N]$ under the rules.
3. No batch certificates follow this CursorSet yet; GFR lacks the watermark needed for RESUME and stops at position $x$ before processing it.
4. CERT-4 requires $x$ to be no earlier than “the gpos of every certificate in the interval,” with no new RESUME between those gpos and $x$. The certificate set is empty, so these conditions impose no effective lower bound; yet $G'$ is longer than $G_x$, and $G'\npreceq G_x$.

The two replay cuts are misaligned; **this does not prove that the Inbox accepted an incorrect batch**. CERT-3's rejection of old prev values is unaffected.

**Suggestion.** Use the RESUME replay position $r_0$ corresponding to the CursorSet beginning the interval as the induction base; require $x\succeq r_0$, later than every certificate's gpos in the interval, and that $x$ has not yet processed the next RESUME. Use the genesis base point for the genesis interval. State the prefix relation after alignment.

**Manual confirmation:** [ ] Add the $r_0$ lower bound; [ ] identify the cut for every cross-layer prefix statement.

## 6. S4: The claim that governance “affects only liveness” does not cover every RESUME input

**Medium priority; gap in trust scope and state transition confirmed; high confidence.** The certification/derivation and GFR subagents supplied the two counterexamples below. They use inputs accepted by the contract but violating off-chain governance procedure; they do not show that governance following the complete procedure produces them.

[01 §2](01_l1_authority_en_v0_1.md) line 22 says governance honesty is used only for recovery liveness; [01 SubmitResume](01_l1_authority_en_v0_1.md) line 258 checks only that the target exceeds the cursor, leaving stronger target and timing conditions to off-chain procedure in lines 260–271. It must be decided whether safety conclusions also depend on that procedure.

### 6.1 A low target truncates G inside the already-derived common prefix

Suppose cursor.height=0, while prefixes of both $G$ and $D$ reach 2. This state can arise with no accepted batches, empty regional ordinary transactions, and native fallback aligned with the regional origin. Governance signs target $N=1$, satisfying the contract's `1 > 0`. Native derivation retains its established prefix through 2, but [09 §4.6](09_finality_rules_en_v0_1.md) line 323 executes $G\leftarrow D_L[0:1]$, removing block 2 from $G$ even though it is in $D$.

This conflicts with line 326's “common native prefix is retained” and “no block is revoked when old G is a prefix of D_L,” and exceeds [09 §7](09_finality_rules_en_v0_1.md), which describes replacement as an underived suffix. **D does not roll back**; the claim about G is too strong.

### 6.2 Expire for the new next height is generated before pc

Take $\mathit{ts}_{L1}(\ell)=\ell$, $\mathit{ts}^*(h)=h$, $M_a=2$, $W_{\rm exp}=10$, $TB=10$, $T_{\rm grace}=10$. Old $G$ and cursor reach 89, and processing Expire(90) at key 100 produces HALT. L1 height 100 includes a RESUME targeting 90. Once executed at key 102, Expire for the new next height 91 has key 101, already before pc. The new window [91,100] has SessionEnd key 111, so line 332's “cannot start” rule for SessionEnd does not specify how to handle this out-of-order Expire.

Possible choices include immediate HALT, deterministic rejection of the arrangement, or another transition; the draft selects none. One cannot insert the new event back into completed history or silently ignore expiry.

**Suggestion.** First define the governance safety boundary. If safety depends on the procedure, make it an explicit premise. If deterministic semantics are wanted for all authorized inputs, consistently handle low targets, excessively distant targets, actual derivation already past the target, and new event keys earlier than pc. One cannot change only the GFR target without coordinating the Inbox cursor and successor window.

**Manual confirmation:** [ ] Governance honesty is also a safety premise for the retained portion of G; or [ ] state-machine guards bound the effects of all authorized controls.

## 7. S5: Propagating D incompatibility by predecessor cid incorrectly kills a correct successor

**High priority; in-model lifecycle counterexample confirmed; high confidence.** Proposed by the regional consensus subagent. BFT-1 is not violated; the issue is the Manager's closure scope.

Location: [03 ApplyLifecycle](03_session_manager_en_v0_1.md) line 226 closes an incompatible cid and successors whose anchor comes from its decided chain; [03 D-based death determination](03_session_manager_en_v0_1.md) line 275 refers to “some block” in that chain without requiring the successor to inherit the conflicting portion.

Let the predecessor window be [1,100] and successor window [101,200] (heights may all be shifted into a legal time range):

1. The predecessor's legally decided chain contains $B_{98}\prec B_{100}$; $B_{100}$ contains ordinary transactions actually executed. $B_{98}$ enters G in time, but the current Manager obtains the complete proof of $B_{100}$ only after the predecessor's public deadline; this late acquisition does not violate BFT-7.
2. GFR correctly selects $B_{98}$ as regular anchor. Successor $q$ decides $B_{101}$ from it, expanding 99 and 100 as padding; these are delivered in time and become a prefix of D. The successor still runs within its own window.
3. The Manager later verifies late $B_{100}$ and updates the predecessor regional view under [03 ObserveRegional](03_session_manager_en_v0_1.md) line 178. At the next D comparison it finds the predecessor's height-100 hash differs from D[100].
4. The rule generates an IncompatibilityFact against the predecessor cid and closes $q$ because $q$'s anchor $B_{98}$ came from that cid's decided chain.

But $q$ **did not inherit $B_{100}$**, and its own prefix is already in D. The correct rule should close candidates continuing from the conflicting block or its descendants, not all successors of the predecessor cid. A legal configuration assigning consecutive responsibilities to the same region can realize this trace, so Managers of sufficient weight retain both predecessor and successor state; no Byzantine-weight bound is exceeded. Close is irreversible, potentially losing the successor's entire remaining production window.

This also means the late certificate in [pipelines v0.2 §3.4](10_pipelines_en_v0_2.md) cannot be treated solely as “losing a little preconfirmation”; an old instance's late result must also be prevented from triggering an overly broad Kill.

**Suggestion.** Reconstruct an IncompatibilityFact belonging to each successor. If propagation is optimized, prove the successor's verified ancestry contains the candidate block at the conflicting height. Even where a same-chain relation is known, anchor.height at least the conflict height is only part of a necessary check; height cannot replace hash/ancestry.

**Manual confirmation:** [ ] Propagate Kill along the actual conflicting prefix, not the cid source relation as a whole.

## 8. S6: A fixed recovery target depends on an as-yet unknown L1 inclusion time

**Medium-high priority; governance procedure feasibility gap confirmed; high confidence.** Proposed by the governance and recovery subagent.

Location: [01 SubmitResume](01_l1_authority_en_v0_1.md) lines 254–271. Governance fixes $h_{\rm res}$ when signing, but the procedure checks against the timestamp of future actual inclusion block $L$:

$$
\mathit{ts}_{L1}(L)+T_{\rm prep}
\le \mathit{ts}^*(h_{\rm res})
\le \mathit{ts}_{L1}(L)+T_{\rm drift}-\epsilon.
$$

If possible inclusion timestamps lie in $[a,b]$, the same pre-signed target is legal at every possible inclusion position only if

$$
b+T_{\rm prep}\le\mathit{ts}^*(h_{\rm res})\le a+T_{\rm drift}-\epsilon,
$$

that is, $b-a\le T_{\rm drift}-\epsilon-T_{\rm prep}$. The right-hand side is only 3 min in the example.

At 00:00 governance submits a target timestamp 00:25:12, expecting inclusion at 00:00:12; the model nevertheless allows inclusion at 00:10 and finalization by 00:19:12, still within the example's submission-to-finalization budget. The target is then only 15 min 12 s after actual inclusion, below the 25 min preparation requirement; the RESUME key becomes 00:30, later than the target's first block slot. The contract checks only cursor monotonicity and can still accept it.

This is a gap in the sender's ability to guarantee compliance with its own procedure under the current premises, not a safety attack after all procedure conditions hold. Late start does not necessarily prevent recovery forever: the system may jump to the current slot and pad, but the original first-block recovery time guarantee fails.

**Suggestion.** Governance authorizes a relative time offset or finite target selection rule, and the contract determines and records the target from actual inclusion time; or sign a valid inclusion-time interval, have the contract reject out-of-range inclusion, and resubmit. If a fixed target remains without a guard, define a complete recovery path for target failure.

**Manual confirmation:** [ ] Compute the target at execution; or [ ] sign an interval plus on-chain time check; or [ ] accept failure risk and narrow the timing claim.

## 9. S7: Even if every RESUME meets the listed conditions, ordinary blocks may never appear

**Medium-high priority; conditional recovery liveness gap; medium-high confidence.** Proposed by the governance and recovery subagent. The author must clarify whether “appropriate control” implicitly excludes it.

Location: [01 governance procedure](01_l1_authority_en_v0_1.md) lines 260–271 and [09 RESUME](09_finality_rules_en_v0_1.md) lines 319–327.

An emergency exists initially, after which all services work normally. Choose a permitted execution in which every control transaction finalizes within 1 min. Governance submits a new RESUME every 4 min; the $i$th is included at $t_i$ with target timestamp $t_i+25$ min:

- Targets strictly increase and exceed the old cursor, G, and current D. The previous CursorSet is finalized before GFR reaches its announcement key, causing D to move beyond the current production branch, so the apparent “emergency” condition can still hold.
- The $i$th opens an arrangement at key $t_i+20$ min, with its first producible slot around $t_i+25$ min.
- The $i+1$th closes it at key $t_i+24$ min. Every program is closed by the next RESUME before it starts, and no new ordinary block is ever produced.

This does not claim liveness under arbitrary malicious governance; it shows that per-request conditions do not suffice to formalize “honest governance recovers the system.” BFT-5 requires an interval continuously running the same cid, and repeated controls destroy exactly that premise.

**Suggestion.** Explicitly require for recovery liveness an eventual sufficiently long operating interval without conflicting Force/RESUME controls. Define serialization or replacement conditions for an unfinished recovery in governance procedure, and do not mistake the lead of one's own not-yet-replayed CursorSet for a new fault.

**Manual confirmation:** [ ] Add a control-quiet interval; [ ] specify when an unfinished RESUME may be replaced.

## 10. S8: A historical anchor cannot be prepared by applying forward from the latest D state

**Medium priority; operating-procedure gap confirmed; medium confidence.** Proposed by the governance and recovery subagent.

[03 PrepareCandidate](03_session_manager_en_v0_1.md) line 194 specifies Apply height by height to $A.h$ from D's verified tip checkpoint; but ObservePublic advances D before replaying GFR. A catching-up node processing a historical Force can already have $h_D>A.h$. Even if the chains are compatible, forward Apply cannot recover an ancestor state from a descendant's execution state.

The general definitions of ReadyState and StateMaterial already allow historical checkpoints, so this does not establish that an implementation must deadlock. The specific preparation rule should select a checkpoint $C$ on the anchor's verified ancestor chain with $C.h\le A.h$, or directly obtain the exact anchor state and verification evidence. Acquisition and replay of historical inputs belong in the preparation budget. RESUME continues to use the exact ResumeCut.anchor_state.

**Manual confirmation:** [ ] Replace “latest tip” with “verified ancestor checkpoint or exact anchor state.”

## 11. S9: Which Byzantine set accounts for key compromise across sessions?

**Model contract to clarify; no attack confirmed; insufficient to declare BFT-1/BFT-4 invalid.** Proposed by the regional consensus subagent.

[00 §2.1](00_arch_en_v0_1.md) line 223 fixes position public keys; §2.3 lines 286–292 allow Byzantine parties to expose private keys and permit different Byzantine sets in different sessions. If three sessions of the same four-member committee count a, b, and c respectively as the sole Byzantine member, and the adversary retains leaked long-term private keys, nominally “at most one faulty member each time” does not prevent it from accumulating three keys. Cid only separates signature purposes; it cannot stop a key holder generating new signatures.

However, if Byz_s includes every leaked key still capable of impersonating that session identity, the execution exceeds the fault bound; authenticated channels may also be intended to exclude such reuse. No specific BFT verifier is specified, so three keys cannot directly be equated with a valid FINAL. The review concludes only that cross-session corruption, recovery, and key lifetime semantics need clarification.

**Suggestion.** Count identities for which the adversary can generate valid signatures in the session's Byzantine weight; or separately define rekeying, key erasure, and historical proof protection. Do not substitute “static within each session” for a cross-session constraint.

**Manual confirmation:** [ ] Compromised keys continue to count in the fault set; or [ ] provide a recovery and rekeying contract.

## 12. Second-round findings (new in v0.2)

New entries use the same format and status labels as v0.1: each gives the triggering capability, relevant rules, minimal trace, impact, repair options, manual checks, proposing subagent, and relationship to v0.1. Values use the example parameters in pipelines v0.2.

### 12.1 S10: The predecessor's final-block certificate is a “free option” at every handoff

**High priority; design defect and in-model trace confirmed; confidence high for the rule trace, medium for concrete BFT certificate assembly.** Proposed by the consensus/handoff subagent, with a similar conclusion from recovery/governance. The lead agent checked [02 BFT-7](02_session_program_en_v0_1.md) lines 388–390, [09 §4.5](09_finality_rules_en_v0_1.md) line 278, and [03 ApplyLifecycle](03_session_manager_en_v0_1.md) line 226. This elevates pipelines v0.2 variant 1b from a conditional trace to one an adversary can construct, and gives a way to trigger S5 on demand.

**Triggering capability.** Either:

- In BFT where the leader collects votes, a Byzantine leader responsible for the predecessor's last decided view; or
- In BFT where all broadcast votes, that Byzantine leader plus votes decisive for the last block, with total weight at most $F_s$.

Collusion with the Espresso proposer can gain up to another $\Delta_{\rm ts}$.

**Minimal trace** (pipelines v0.2 values):

1. c's Byzantine leader proposes 32,410 at 09:00:10.2 containing its transaction T. It lets only honest weight insufficient to form a certificate alone receive and vote before $\tau^{\rm stop}=$ 09:00:11; honest members receiving it later reject under Accept's timing condition. BFT-7 constrains only honest parties, and CheckFINAL does not check time, so the Byzantine party can complete the certificate at any time.
2. a enters cid(32,409) and regionally decides blocks from 32,411.
3. Before 09:02:05 the Byzantine party chooses:
   - **Publish the certificate** ($\theta\approx$ 09:02:06, before 09:02:11): GFR accepts 32,410 ($h_{\max}$), closes (7,6) as COMPLETE and selects cid(32,410). Roughly 115 blocks already regionally decided by a are excluded and become padding; T executes at the 09:00:10 price.
   - **Withhold the certificate:** 32,410 never existed. Then give Res(32,410) to a's Manager. [03 ObserveRegional](03_session_manager_en_v0_1.md) does not check the public deadline and updates the predecessor view; once $D[32\,410]$ is a's padding, an IncompatibilityFact against (7,6)'s cid closes a's **selected** cid(32,409) under S5's propagation rule. a loses production for the rest of the hour.
4. Variant: leak the certificate to only half of a's honest members, splitting them across two cids within about $T_{\rm pub}+T_E$.

**Impact.**

- Regional fast confirmations in roughly the first $T_{\rm grace}$ plus publication delay of each handoff can be revoked, enabling a double spend against a counterparty relying on them. Inclusion of the last block becomes a two-minute free option (MEV), as well as a low-cost liveness attack.
- Conflicting signatures across cids are not slashable: CheckEvidence checks each descriptor separately (BFT-4 item 4), while honest members also sign multiple candidates legally ([03 §4.2](03_session_manager_en_v0_1.md) line 255).
- The attack can repeat whenever the adversary leads the predecessor's last decided view; frequency is discussed in S17.
- v6 Part 0 S4(b) (fast confirmation consistent with G) has no corresponding claim in v9.

**Suggestions** (design choices for your decision):

- (a) Define “fast confirmation” explicitly as regional finality only in a Selected cid and include this in product semantics.
- (b) Let the successor's first result fix the predecessor's terminal anchor: when Tail is $\mathsf{NEXT}(\nu_s)$ and $\theta_q\ge\tau_s^{\rm stop}$, the first fully public result of a precomputed successor cid(X) closes $s$ with X as anchor; X must extend the prefix of $s$ accepted in $G$, and its chain is published with the result as anchor evidence. Later results of $s$ are ineligible. An honest successor proposes only after $\tau_s^{\rm stop}+\Delta_{i}+\Delta_{\rm cross}$, by which point certificates formed by honest parties have arrived via W26. This compresses the option to one publication delay and removes the variant 1a wait until $\theta_s^{\rm end}$.
- (c) Adopt the S5 repair: propagate Kill only along actual ancestry.

**Manual confirmation:** [ ] Fast-confirmation semantics; [ ] whether a successor result fixes the terminal anchor; [ ] whether propagation of the final-block certificate to sufficient honest weight before stop is a required BFT property.

### 12.2 S11: Certifier dependence on finalized L1 makes multi-submission channels time out and limits settlement throughput

**High priority; design defect confirmed; confidence high for the provisions, medium for throughput estimates.** Independently proposed by certification/derivation and recovery/governance. Unlike S2, which needs the primary Batcher to abandon a channel, this reaches timeout with no failure.

**Location.** [06 CertifyBatch](06_certifier_en_v0_1.md) lines 97–98 require prev to equal the “current cursor in its L1 view” and check recovery alignment using prev's write position $p_{\rm prev}$; [00 §1.2](00_arch_en_v0_1.md) line 68 defines L1 as finalized blocks. Thus the Certifier can certify the next cursor transition only after the previous acceptance finalizes.

**Minimal trace.**

1. A channel needs two submissions; frame 0 is accepted in L1 block $O$.
2. Certification of frame 1 requires $O$ to have finalized (at least 64 slots), so frame 1 lands no earlier than $O+65$, after CHANNEL_TIMEOUT = 50.
3. Both interpretations fail:
   - If [07 SubmitBatch](07_batch_inbox_en_v0_1.md) literally checks only prev, the closing frame is accepted and cursor passes a channel already discarded by derivation. The heights become deposit-only after the native deadline, followed by Kill and RESUME revoking roughly $W_{\rm seq}$ of $G$ (the scale of pipelines v0.2 §6).
   - If 07 inherits native timeout, Inbox expects a prev with zero frames_digest, which Certifier never signs (S2); no one can submit again before RESUME.
4. Throughput: at most one submission per finality cycle and one closed channel per submission. The subagent estimates roughly 10–19 KB/s of compressed data using an EigenDA single-blob limit of about 16 MB (not checked against this repository's specification). Above this load the backlog grows without bound and every height eventually misses its native deadline; cheap L2 spam transactions can increase load.

**Impact.** The premises of A-DELIVER (honest Batcher, C online) hold but delivery fails, leaving BAT-4 inapplicable; under the literal reading, [00 §4.5](00_arch_en_v0_1.md)'s $D\preceq G'$ also fails.

**Suggestion.** Remove CertifyBatch's dependence on an L1 view, consistent with the native certifier core:

- Carry evidence of the accepted prefix in the request and check it against prev.frames_digest.
- Check recovery alignment using prev.source: $\mathrm{RESUME}(c)$ requires Certifier's GFR to have processed $c$; $\mathrm{GLOBAL}(g')$ requires this gpos no earlier than $g'$.
- Leave freshness and timeout to Inbox, and add native effective-cursor timeout to 07.
- Add “each channel completes within CHANNEL_TIMEOUT” and a data-rate bound to A-DELIVER.

The lead agent checked the source-based argument: RESUME always rewrites the cursor in the same transaction, so if prev.source is GLOBAL, no RESUME has taken effect since prev was written; if RESUME lands later, Inbox's prev equality check rejects the old transition, preserving CERT-3. Please confirm the source field's semantics suffice for this role.

**Manual confirmation:** [ ] Whether Certifier depends on finalized L1; [ ] settlement throughput target and DA payload limit.

### 12.3 S12: RESUME targets have no on-chain time bound

**High priority; design defect and regression relative to v6 confirmed; high confidence.** Proposed by certification/derivation and recovery/governance. Extends S4 (low target) and S6 (uncertain inclusion time). v6 relies on contract check C-RES-5 ($\mathit{ts}(L)<t(h_{\rm res})\le\mathit{ts}(L)+D_{\rm drift}$); v9 [01 SubmitResume](01_l1_authority_en_v0_1.md) line 258 checks only that the target exceeds the cursor.

**Three traces.**

1. **Irreversible freeze from a distant target.** Governance mistakenly signs 882,120 instead of 88,212; the contract accepts it (above cursor). Under the [native override rule](../megaeth_specs/specs/rollup/derivation.md#emergency-cursor-overrides), each derivation node must execute 838,608 padding blocks before handling subsequent L1 inputs; at 149 blocks/s this takes about 94 min.
   - Padding origin remains at $L$, with anchor drift about 9.2 days; Enter fails, and Force reuses the anchor.
   - Heights above 882,120 reach their slots only around 10 days later, so ordinary L2 cannot recover before then; L2 block timestamps lead real time by roughly 9 days.
   - A target near $2^{64}$ makes padding effectively unfinishable, permanently stopping derivation and L1 deposits; under [native recovery S8](../megaeth_specs/specs/block-certification/recovery.md#s8-a-previous-override-left-the-cursor-at-an-unreachable-position), once cursor reaches the uint64 maximum the override interface has no repair path.
   - Consequently, forced inclusion (v6 L2-L2) also depends on governance.
2. **RESUME during normal operation also revokes $G$.** Cursor normally lags $G$ by about 20 min. A RESUME targeting $\mathit{ts}(L)+25$ min is included at $L$; at key $\mathit{ts}(L)+20$ min, $G\leftarrow D_L$, revoking about 35–40 min (2,100–2,400 heights) of globally final content although no emergency exists.
3. **Signed control records do not expire.** If SubmitResume's caller is unrestricted, a third party can front-run with an old signature for the same control sequence; after that number is used, the corrected signature fails (conditional on caller rules).

**Impact.** [01 §2](01_l1_authority_en_v0_1.md) line 22's statement that “Authority checks themselves do not depend on $\mathsf M$ being honest” does not hold; both the retained portion of $G$ and forced inclusion depend on honest governance and off-chain procedure.

**Suggestions.**

- Have the contract compute the target from inclusion time: $h_{\rm res}:=\mathrm{slot}(\text{block.timestamp}+T_{\rm prep})$; or check $\text{block.timestamp}+T_{\rm prep}\le\mathit{ts}^*(h_{\rm res})\le\text{block.timestamp}+T_{\rm drift}-\epsilon$. This addresses both S6 and distant targets. With the lower bound, “above cursor” also implies “above $D$'s tip”: $D$'s batch-derived tip is at most cursor, and fallback tip is at most $\mathit{ts}_{\rm L1}-W_{\rm seq}+T_{\rm drift}$.
- On-chain emergency guard: require $\text{block.timestamp}\ge\mathit{ts}^*(\text{cursor.height}+1)+W_{\rm exp}$. It is false in normal operation and true when GFR may HALT. A late accepted batch refreshes cursor (pipelines v0.2 §6.4), so use it with R1 in §14.
- Add expiry to control records or bind them to a recent L1 block hash.
- Add timelocked rotation for $pk_{\mathsf M}$.
- If retaining off-chain procedure, state governance honesty as a safety premise for the retained part of $G$ and forced inclusion (decide together with S4).

**Manual confirmation:** [ ] Contract computes target or checks a time bound; [ ] on-chain emergency guard; [ ] control-record expiry; [ ] whether governance honesty is a safety premise.

### 12.4 S13: Primary Batcher and Certifier keys cannot be replaced

**High priority; trust-boundary gap confirmed; confidence high for missing interfaces, medium for the economics of competition.** Proposed by certification/derivation and recovery/governance. Supplements S2 and v0.1 §12's primary Batcher occupation/abandonment item.

**Location.** [07 §3.1–§3.2](07_batch_inbox_en_v0_1.md) lines 44, 87, 89 fix admin, submission permissions, and certifier_pk; [01 §3.3](01_l1_authority_en_v0_1.md) has no call to change Batcher permissions or rotate $pk_{\mathsf C}$.

**Trace.**

1. The primary Batcher submits a certified, unclosed frame 0 and stops. Under v9's Certifier, nobody can submit after timeout (S2).
2. Even if S2 is fixed, the primary Batcher is exclusive before cursor.origin + EXCLUSIVE_WINDOW and can reopen a channel at each timeout boundary. Each win occupies 10 min; about eight wins make a height entering $G$ only at $\mathit{ts}^*(h)+W_{\rm exp}$ miss its native deadline. Native rules limit this through admin removal of the address; v9 Authority has no such call, while every closing submission reopens the exclusive window.
3. Each stall ends in a missed native deadline, Kill, and a RESUME revoking around 12 h of $G$, repeating indefinitely. The RESUME procedure premise that “Batcher and Certifier have recovered” never holds.
4. $pk_{\mathsf C}$ is fixed. Permanent key loss ($\mathsf C$ may be offline in the model) permanently ends ordinary L2. If compromised, it can sign next.height $=2^{64}-1$: 07 checks no field of next, so a target above cursor can never exist and RESUME is unavailable. The latter exceeds A-CERT, but its damage scope is stated nowhere.

**Impact.** [05 §1](05_batcher_en_v0_1.md)'s claim that “Batcher behavior affects only when delivery completes” fails; the damage scope from compromising $\mathsf C$ is undefined.

**Suggestion.** Add timelocked calls signed by $\mathsf M$ to Authority for replacing primary and backup Batchers and rotating $pk_{\mathsf C}$. Allow RESUME to specify a new primary Batcher and make the override's origin place the exclusive window in the past. Inbox checks $\mathit{ts}^*(\text{next.height})\le\text{block.timestamp}+\varepsilon$.

**Manual confirmation:** [ ] Add permission and key management interfaces; [ ] state the damage scope if $\mathsf C$ is compromised.

### 12.5 S14: A block proposed while Espresso is non-live can use an old timestamp to bring all downtime results into $G$

**High priority; implicit assumption and in-model trace confirmed; medium confidence (HotShot timestamp checks not verified).** Proposed by recovery/governance. Continues v6 issue 3 and is stronger than variant A′: no in-flight block is needed.

**Location.** [00 §1.4](00_arch_en_v0_1.md) line 191 explicitly denies E-TS to blocks proposed during non-live periods; [09 §4.2, §4.4](09_finality_rules_en_v0_1.md) use only $\theta$ for public deadlines and expiry.

**Minimal trace** (pipelines v0.2 §5, downtime from 10:00:01):

1. Late in downtime, a proposer proposes block J with $\theta=$ 10:00:02, containing a's 36,000–36,010 and b's 36,011–39,610 decided during downtime. J finalizes after recovery. E-S requires only nondecreasing timestamps and permits this value.
2. On replay, J's key precedes SessionEnd(7,7) at 10:02:11. GFR accepts a's results through $h_{\max}=36\,010$, closes (7,7) as COMPLETE and selects cid(36,010). In the same Espresso processing loop, (7,8)'s results satisfy $\theta<$ 11:02:11 and $\theta<\mathit{ts}^*(h)+W_{\rm exp}$ and are also accepted (regular successors are not restricted to acceptance in the next block; pipelines v0.2 §3.3 step 8 takes the same reading).
3. 3,611 heights become globally final at D2 00:00. $D$ has been deposit-only since 21:40, and RESUME revokes all of them. Certificates withheld long after stop can also enter this way.

**Impact.** Pipelines v0.2 §5.7 and §7's claim that “without an in-flight block, $G$ is not revoked” fails in the model (erratum E1 in §15); the §4.2 public deadline loses its real-time meaning; S18's stake-release bound becomes unbounded in non-live periods.

**Suggestion.** Adopt v6 repair F1: use as the Espresso block's ordering key the timestamp of a successor proposed while live that proves it finalized. Or make “HotShot validates $\theta$ against local clocks” an explicit Espresso assumption and verify HotShot's implementation.

**Manual confirmation:** [ ] Adopt F1; or [ ] add a HotShot timestamp assumption.

### 12.6 S15: L1 finalization has no margin, and the origin schedule has no fallback

**Medium-high priority; implicit assumption confirmed; high confidence in the mechanism.** Proposed by recovery/governance. Pipelines v0.2 §2.3 records a 48 s margin but does not state the consequences of exceeding it.

**Location.** The schedule and $\omega_A$ in [00 §3.4](00_arch_en_v0_1.md); [02 Build](02_session_program_en_v0_1.md) line 198 executes $\mathrm{Dep}^*_A(h)$ and Accept requires $B.\mathit{os}=\mathit{os}^*_A(B.h)$; R-WAIT in [09 §4.3](09_finality_rules_en_v0_1.md).

**Trace.** The example's $T_{\rm L1}=19.2$ min is exactly the worst value in normal Ethereum operation (a block at an epoch's first slot finalizes after 95 slots). If one epoch fails to justify on time, finalization is 6.4 min later:

- The schedule-selected origin is not finalized; Build and Accept wait about 5.6 min, and skipped slots become padding.
- $M_a$ has only 6 s margin, so R-WAIT also stalls GFR.

**Impact.** Production stops in all regions simultaneously, precisely under a condition BFT-5 assumes away.

**Suggestions.**

- Increase $T_{\rm dep}$ to about 27–28 min. $T_{\rm dep}+b_{\rm L1}\le T_{\rm drift}$ still holds, and the first block after Force or RESUME retains an origin within drift.
- Cost: larger steady-state origin lag; the margin between Expire and native deadline drops from 1 h 40 min to about 1 h 32 min, still covering $M_a+T_{\rm batch}=1$ h 20 min. Recalculate together with $W_{\rm exp}$ and $T_{\rm batch}$.
- Alternatively, when the scheduled origin is not finalized, permit staying on the previous finalized L1 block.
- Call $T_{\rm L1}$ a “typical value,” not an upper bound.

**Manual confirmation:** [ ] Value of $T_{\rm dep}$; [ ] whether origin fallback is allowed.

### 12.7 S16: Tail is undefined after a RESUME arrangement “cannot start”

**Medium priority; specification gap confirmed, implementations may diverge; high confidence.** Proposed by recovery/governance and certification/derivation; same class as S4 §6.2.

**Location.** RESUME step 4 in [09 §4.6](09_finality_rules_en_v0_1.md) fixes the new window at $[h_{\rm res}+1,h_{\rm res}+\mathrm{TB}]$; the unified rule at line 332 outputs only SessionClosed (EXPIRED), specifying neither Tail nor successor assignment.

**Trace.** A compliant RESUME is included at $L=$ D2 00:05:12, with $h_{\rm res}=88\,212$, key 00:25:12, and SessionEnd key 01:32:13 for (8,0).

1. If Espresso produces no block in (00:25:12, 01:32:13] (about 67 min of renewed downtime, or RESUME inclusion about 67 min late; see S6), the arrangement can no longer start when opened.
2. GFR outputs only SessionClosed (EXPIRED), with no successor, and does not HALT until Expire(88,213) at 10:30:13.
3. Under 01's procedure, another RESUME must wait for $D$ fallback to exceed 88,212, around D2 12:05:12, losing about 12 hours.
4. If one implementation sets Tail to HALT and another retains NEXT, they disagree on the validity of a later Force, potentially forking $G$.

**Suggestion.** Open the RESUME window as for Force: $h_{\min}=\max\{h_{\rm res}+1,\mathrm{slot}(\theta_{\rm open}+T_{\rm init})+1\}$. Alternatively, specify that “cannot start” means HALT and adopt R2 in §14 to allow RESUME when HALT and $G\preceq D$.

**Manual confirmation:** [ ] How a RESUME window opens; [ ] Tail after “cannot start.”

### 12.8 S17: Proven Byzantine weight can still vote; the leader rule is unconstrained

**Medium priority; specification gap confirmed (a concrete instance of S9); confidence high for the provisions, medium for impact (the leader rule is unspecified).** Proposed by consensus/handoff.

**Location.** [01 §3.2](01_l1_authority_en_v0_1.md) lines 132–136 (banned parties retain frozen voting weight), §3.12 line 292; [09 §4.5](09_finality_rules_en_v0_1.md) line 287; G3.3.

**Trace.**

- **Weight.** Take $W=100$, $F=33$. K1 (weight 20) is banned for double-signing in $s_1$; K2 (weight 20) is banned for double-signing in $s_2$. Later sessions using the same snapshot still give K1∪K2 frozen weight 40 > F: Plan knowingly assigns a committee with proven Byzantine weight exceeding $F_s$. Keys slashed to zero still vote at zero cost.
- **Timing.** A ban takes effect only when $\ell_s$ is no earlier than the ban height, while $\ell_{s+1}=A_s.\mathit{os}$. For example, (7,7) runs until 10:00:10 using L1 state from 07:40:00.
- **Leader.** Parameters of leader_rule may be derived from selection inputs. If Ref.block_id (the predecessor's last block) is the seed, the predecessor's final leader can grind transaction order to influence the successor's leader sequence. Unweighted rotation also lets weight no greater than $F_s$ split into many positions and lead most views, including the final view determining S10's frequency.

**Impact.** $\mathrm{Byz}_s\le F_s$ can knowingly be violated; effects on G3.3 progress and S10 frequency.

**Suggestion.** Plan assigns zero weight to public keys in $\mathsf{Ban}(\ell_s)$ and recomputes $W_s,F_s,Q_s$ (remaining Byzantine weight is at most $F-b\le F'$, preserving safety). Weight leaders by stake and seed with L1 randomness at $\ell_s$, not block_id; specify a minimum stake per position.

**Manual confirmation:** [ ] Set weight to zero on ban; [ ] constrain leader rule seed and weights.

### 12.9 S18: Missing stake-release and evidence windows

**Medium priority; specification gap confirmed; confidence high for the gap, exploitation requires prolonged downtime.** Proposed by consensus/handoff and recovery/governance. v6 S6 and issue 3 remain unaddressed in v9. v0.1 §12 identified only the contract omission; this gives a concrete path.

**Location.** [01 Release](01_l1_authority_en_v0_1.md) line 236 delegates all conditions to $\mathcal P_{\rm stake}$; [00 §1.4](00_arch_en_v0_1.md) line 191; [09 §4.2, §4.5–§4.6](09_finality_rules_en_v0_1.md); §4.4 line 274 orders eligible extensions by hash.

**Points.**

- v9 has no evidence window $T_{\rm ev}$, exit quarantine, or link between stake release and sessions.
- **Bounded when Espresso is live.** The deadline for a session still containing pid to accept results is bounded: an empty session retains the initial anchor, Expire limits gaps, and Force refreshes Ref through $\mathrm{last}(G)$. Thus $\theta^{\rm end}<\mathit{ts}_{\rm L1}(a_{\rm ex})+T_{\rm drift}+W_{\rm exp}+T_{\rm init}+2\,\mathrm{TB}+T_{\rm grace}\approx 12$ h 34 min, where $a_{\rm ex}$ is activation height of the first snapshot excluding pid; another $\Delta_{\rm ts}+T_E$ later it expires in real time. The v6 formula can be retained.
- **Unbounded when Espresso is non-live.** Downtime starts at 10:01. On day 7.5, a block proposed while non-live with $\theta=$ 10:01:30 carries (7,7) signatures produced only after stake release (a late S10 certificate, or a conflicting chain forged by weight exceeding $F_s$); it finalizes on day 8 and enters $G$ after a seven-day unbonding period. Conflicting eligible extensions are ordered by hash, allowing the attacker to grind priority. L1 cannot observe these.
- While G is stalled, every new session's Ref stays at $\mathrm{last}(G)$ and the committee comes from a snapshot about $W_{\rm exp}+T_{\rm dep}$ old.

**Suggestion.** Restate S6 under E-L/E-TS: release no earlier than the expression above plus $\Delta_{\rm ts}+T_E+T_{\rm ev}$, with $T_{\rm ev}$ measured from finalization of the Espresso block carrying the result. Unconditional S6 requires an L1-verifiable Espresso watermark proving the last session that could use a snapshot containing pid has reached $\theta\ge\theta^{\rm end}$; alternatively adopt S14's F1. The $\mathcal P_{\rm stake}$ unbonding period must cover the bound and pause while the GFR watermark lags L1.

**Manual confirmation:** [ ] Restore S6 and its premises; [ ] relation between unbonding period and watermark.

### 12.10 S19: Consequences of a single region exceeding $F_s$ are unspecified

**Medium priority; implicit assumption confirmed; confidence high for (a) and (b), medium for (c).** Proposed by consensus/handoff. This does not break determinism of $G$ or affect other regions.

**Location.** [02 BFT-4](02_session_program_en_v0_1.md) lines 321–330 and Accept; [09 §4.4](09_finality_rules_en_v0_1.md) lines 263–271; [01 Join](01_l1_authority_en_v0_1.md).

- **(a) No accountable safety.** If $F_s<\mathrm{Byz}<Q_s$, the selected cid may have two decided chains; $G$ takes whichever reaches Espresso first. BFT-4 only prohibits false evidence (“does not require every violation to be discovered”); slashability depends on the chosen BFT's forensic capability.
- **(b) Future timestamps enter $G$.** If $\mathrm{Byz}\ge Q_s$, a block no honest party accepted can finalize. GFR checks deadline, expiry, and origin, but never compares $\mathit{ts}^*(h)$ with $\theta$. Thus a block whose timestamp leads real time by roughly $T_{\rm dep}$ minus actual L1 finalization time (typically 5–7 min; 48 s using 19.2 min) can enter $G$ and $D$; native checks are relative only to height and origin.
- **(c) Attack cost is determined by the smallest region.** Join permits arbitrary region selection: exceeding $F$ requires new weight above $W_i/2$, and reaching $Q$ requires $2W_i$.

**Suggestion.** Add BFT-8 accountable safety: two conflicting decided chains in one cid yield evidence against weight exceeding $F_s$ from two certificates. Add GFR check $\mathit{ts}^*(B.h)\le\theta_{q_{\rm avail}}+\Delta_{\rm ts}$; honest results satisfy it under E-TS. Add a per-region minimum stake or cap in $\mathcal P_{\rm stake}$.

**Manual confirmation:** [ ] BFT-8; [ ] GFR time upper-bound check; [ ] minimum regional stake.

### 12.11 S20: BFT configuration interface implicitly excludes two BFT classes

**Medium priority; implicit assumption, implementations may diverge; medium confidence.** Proposed by consensus/handoff.

**Location.** [02 §3.2](02_session_program_en_v0_1.md) line 117: first_received is keyed by an undefined “proposal identifier”; §3.7–§3.9; BFT-5.

- **(a) Reproposal.** If proposals are distinguished by view, Tendermint-style reproposal of a locked block yields a new $\tau_{\rm recv}\ge\mathit{ts}^*(h+1)+T_{\rm win}$: unlocked honest parties reject it, locked parties reject other values, and the cid deadlocks before $\tau^{\rm stop}$. If distinguished by block hash, a Byzantine leader can compute a height-$h$ block early and send it to some members before $\mathit{ts}^*(h)$, causing them to reject it permanently; once others lock, the same deadlock follows.
- **(b) Chained commitment.** Chained-commit BFT such as HotStuff or Jolteon requires later heights to commit; the window's last one or two heights can never be decided. “$h_{\max}$ has been accepted” never occurs, every handoff is variant 1a, and S10's window opens every hour.

**Suggestion.** Key first_received by $(\mathrm{cid},\mathsf H(B))$, validate reproposals using QC/POL without repeating the time check; or restrict the abstraction to BFT proposing a new block in each view and explicitly finalizing the last block (such as Simplex).

**Manual confirmation:** [ ] Definition of proposal identifier; [ ] applicable BFT family.

### 12.12 S21: CertifyBatch does not require decoding to match derivation

**Medium priority; specification gap; medium confidence (an implementation copying the native certifier is safe).** Proposed by certification/derivation.

**Location.** [06 CertifyBatch](06_certifier_en_v0_1.md) line 101 (“block records fully decodable in this submission”) and CERT-1(4); [05 BlockRecord](05_batcher_en_v0_1.md) (native batches carry timestamp, not block_number); native certifier and [Closed-Channel Decoding](../megaeth_specs/specs/rollup/derivation.md#closed-channel-decoding).

**Trace.**

1. Batcher closes a channel with a corrupted zlib ending, raw size over 100 MB, or timestamp shifted by $k$ slots.
2. A Certifier following v9's text certifies the decodable prefix because those records agree with $G$.
3. Derivation releases none of the channel's contents, or discards the batches as future batches.
4. Cursor passes the lost content, with the same consequences as S11.

**Suggestion.** CERT-1(4) should require Certifier decoding to match derivation exactly: timestamp $=\mathit{ts}^*(h)$, raw-size limit, equal channel_id, complete stream validation and closure.

**Manual confirmation:** [ ] Certifier decoding matches derivation item by item.

### 12.13 S22: A-CUSTODY's release condition cannot serve newly joining nodes

**Low priority; implicit assumption.** Proposed by certification/derivation.

**Location.** [08 A-CUSTODY](08_da_en_v0_1.md) line 26; [09 §3.4](09_finality_rules_en_v0_1.md), where a checkpoint can be verified only against replay of the public prefix; ResumeCut.derived_prefix begins at $h^0$.

**Issue.** Custodians may discard bytes after a checkpoint covers them, but the checkpoint is verified only by replay. A node joining after data is discarded cannot verify any checkpoint or process RESUME.

**Suggestion.** Retain data permanently, or anchor checkpoints to a StateClaim settled on L1.

**Manual confirmation:** [ ] Custody duration and checkpoint trust source.

### 12.14 S23: Acceptance condition 5 depends on the reader's local L1 prefix, so honest readers can compute different $G$

**High priority (under a literal reading, the most serious finding in the report: it breaks agreement on $G$); specification gap and missing parameter constraint confirmed; confidence high for the provisions, medium that implementations follow the literal reading.** Proposed by the GFR subagent. The lead agent checked [09 §4.4](09_finality_rules_en_v0_1.md) line 269 (“and required L1 inputs are finalized” as an acceptance condition), R-WAIT (lines 245–251), and line 251 (“an individual publication not yet fully public is temporarily ineligible and does not block other events”). It also makes v6 issue 3 more serious.

**Root cause.** R-WAIT guarantees only that a reader has an L1 block with timestamp at least $x-M_a$. The origin of a new height $h$ has timestamp no later than $\mathit{ts}^*(h)-T_{\rm dep}$. Pipelines v0.2 sets $T_{\rm dep}=M_a=20$ min; E-TS allows $\theta_q$ to lag proposal time by $\Delta_{\rm ts}=15$ s. Whenever $\theta_q<\mathit{ts}^*(h)$, the origin can lie outside the prefix guaranteed by R-WAIT. Condition 5 then depends on each reader's locally synchronized L1 prefix, while missing data is skipped as “ineligible” instead of WAIT.

**Triggering capability.** An Espresso timestamp behind real time but still within E-TS. The second reader is an honest catching-up node whose L1 sync lags its Espresso sync, such as a restarted Manager, Batcher, or Certifier.

**Minimal trace.**

1. The origin of $h=37\,800$ (10:30:00) is L1 block 3,050 (10:10:00). It is decided at 10:30:00.6 and included in $J_q$ proposed at 10:30:01 with $\theta_q=$ 10:29:47.
2. R-WAIT for $J_q$ requires L1 coverage only through timestamp at least 10:09:47; block 3,049 suffices.
3. Reader P is online, already has 3,050, and accepts 37,800. Reader Q's L1 prefix ends at 3,049; condition 5 fails, so it skips the result without blocking.
4. A Force with $\theta_a=$ 10:29:48 (included no later than block 3,049) executes in $J_{q+1}$ with $\theta=$ 10:29:48; R-WAIT is still satisfied by 3,049.
5. Force anchor $\mathrm{last}(G)$ is 37,800 at P and 37,799 at Q: $\mathrm{id}(A)$ differs, new cids differ, and their $G$ chains diverge permanently.

**Broader variant** (v6 issue 3): an old-timestamp block at $\theta=$ 10:00:01 carries (7,8) results through 39,000, whose origins reach block 3,150, while R-WAIT's cut is around block 2,901. Q skips the results; SessionEnd(7,8) at 11:02:11 fixes a regular anchor around 36,020 or below at Q, but 39,000 at P. This variant needs no Force.

**Impact.** [09 §1](09_finality_rules_en_v0_1.md)'s claim that “any reader … obtains the same $G$” fails; Tail and lifecycle events also differ.

**Suggestions.**

- Evaluate conditions 5 and 6 only on the key-determined cut $C(x)=\{\ell:\mathit{ts}_{\rm L1}(\ell)\le x-M_a\}$, not the local prefix; return WAIT when local data within that cut is missing.
- Add parameter constraint $T_{\rm dep}\ge M_a+\Delta_{\rm ts}$ (e.g. 20 min 15 s) so honest results produced while live are not delayed by it.
- Side effect: a block with key $\theta$ can accept schedule-produced heights only up to $\mathit{ts}^*(h)\lesssim\theta+27$ s. This causal constraint blocks an old-timestamp block from bringing in results for the entire downtime at once (the main part of S14), but not an in-flight block with content predating $\theta^*$ (A′).

**Manual confirmation:** [ ] Evaluate conditions 5 and 6 on the key-determined cut; [ ] WAIT for missing data; [ ] $T_{\rm dep}\ge M_a+\Delta_{\rm ts}$.

### 12.15 S24: first_complete_q is not a function of (result_key, log)

**High priority; specification gap confirmed; medium-high confidence.** Raised by the GFR subagent. The lead agent checked [09 §3.2](09_finality_rules_en_v0_1.md) line 102 and [09 §4.2](09_finality_rules_en_v0_1.md) (“proof, necessary bodies, and context source are all in the public object set”), [04 §3.2](04_publisher_en_v0_1.md) (already published objects may be omitted by reference), and [02 AnchorEvidence](02_session_program_en_v0_1.md).

**Background.** result_key $=(\mathrm{cid},\mathsf H(B))$ does not include $\pi_{\rm fin}$ or $\pi_A$; different publications merge into one entry. The specification does not define which components are “necessary.”

**Traces.**

- **(a) Split at the deadline.** A Byzantine member of (7,7) publishes Res(36 010) at $\theta=$ 10:01:30: $\pi_{\rm fin}$ and $C$ are valid, but $\pi_A$ refers to a proof version for anchor 32 410 that was never published. The member also forwards this Res through W26; the successor Publisher deduplicates by key (PUB-2), retains and republishes this $\pi_A$. An honest replica of a only gets its own version included at $\theta=$ 10:02:20 due to rising fees. A reader taking the union of objects (the anchor is already in $G$) considers the result complete at 10:01:30 and accepts 36 010; a reader requiring every message to carry the source of $\pi_A$ considers it complete only at 10:02:20, past the deadline. Their regular anchors are 36 010 and 36 009 respectively, and $G$ forks.
- **(b) Strict reader stalls.** FORCE and RESUME anchor sources are on L1 or in replay, never on Espresso. A reader requiring these sources to be “in the public object set” can never consider any result of (8,0) after RESUME complete; its $G$ stalls and later reaches Expire while other readers advance.
- **(c) Checkpoint inconsistency.** The same block may have two valid $\pi_{\rm fin}$. Which one $\mathrm{src}_G$ records is unspecified, so a GlobalCheckpoint generated by one implementation will be rejected by another's Init (line 174).

**Recommendation.** Define $q_{\rm avail}(\mathrm{cid},B)$ as the minimum $q$ for which verified objects in $J_0\ldots J_q$ contain a valid proof for $B$ or its descendant under the same cid, and all bodies from $\nu.\mathit{anchor}$ to $B$. The anchor is determined by $\nu$ itself; $\pi_A$ does not participate in the GFR decision. $\mathrm{src}_G$ records the earliest-positioned valid proof, breaking a tie at the same position by smallest hash.

**Manual confirmation:** [ ] definition of $q_{\rm avail}$; [ ] whether $\pi_A$ participates in the GFR; [ ] proof-selection rule for $\mathrm{src}_G$.

### 12.16 S25: An L1 finalization stall makes content that entered $G$ on time miss the native deadline

**Medium-high priority; implicit assumption confirmed; medium confidence (HotShot header L1-field semantics not checked).** Raised by the GFR subagent; supplements the in-flight block item in v0.1 §12 and S14: F1 addresses Espresso timestamp distortion only, not processing delays caused by an L1 finalization stall.

**Trace.** R-WAIT makes every event wait for L1 finalization. The native deadline is about 11 h 40 min after the block slot, while acceptance can occur as late as 10 h after the slot (when filling a gap with padding). The Batcher needs 1 h, leaving about 40 minutes (about 20 minutes after deducting $M_a$) to absorb processing delay. A result filling a gap with padding, with key $\mathit{ts}^*(X)+9$ h 59 min, misses the native deadline if an Ethereum finalization loss delays processing by about one hour (events of about 25 minutes and about one hour occurred in May 2023): $D[X]$ becomes deposit-only, and the next RESUME reverts it from $G$.

**Recommendation.** Add an acceptance guard: $\mathrm{ddl}(h)-\text{l1\_head}(J_{c(q)})\ge\lceil T_{\rm batch}/b_{\rm L1}\rceil$, where l1_head is the L1 head recorded in the Espresso header. This changes “accepted then reverted” into “not accepted.”

**Manual confirmation:** [ ] whether to check remaining native deadline margin at acceptance.

### 12.17 S26: One Espresso proposer determines the order of Force and SessionEnd / Expire in the same period

**Medium priority; design flaw confirmed; high confidence.** Raised by the GFR subagent. The lead agent checked the in-block order in [09 §4.3](09_finality_rules_en_v0_1.md) (Force executes at the beginning of the first block with $\theta_q\ge\theta_a$) and the Force window in §4.6. The outcome is deterministic but manipulable by one proposer.

**Triggering capability.** The proposer of the first Espresso block with $\theta\ge\theta_a$ chooses $\theta$ within E-TS.

**Trace.**

1. SessionEnd(7,7) has key 10:02:11. A Force with $\theta_a=$ 10:02:00 installs the new cyclic order (a, b, c), with all regions enabled. The next block is proposed at 10:02:01.
2. Honest proposer ($\theta=$ 10:02:01): Force executes first. Old NEXT is (7,7), region a, which has contributed a block; source region is a, and Plan selects b.
3. The proposer chooses $\theta=$ 10:02:12 (allowed, no greater than $\tau+15$ s): SessionEnd executes first and assigns (7,8) to b; old NEXT as seen by Force is (7,8), source region is b, and Plan selects c. The window also shifts by 11 s.
4. Applying the same technique to the Expire key immediately after $\theta_a$ can invalidate Force (HALT, entering the dead zone where only RESUME works).

**Impact.** Governance intent can be manipulated.

**Recommendation.** Make FORCE a separate event keyed by $\theta_a$, with category order RESUME ≺ FORCE ≺ EXPIRE ≺ SESSION_END ≺ ESPRESSO. Compute the window from $\theta_a+T_{\rm init}$. R-WAIT requires L1 coverage through $\theta_a-M_a$ and an Espresso watermark no lower than $\theta_a$. Add to the governance procedure $\theta_a\le\mathrm{key}(\mathsf{Expire}(h_G+1))-T_{\rm init}-$ margin.

**Manual confirmation:** [ ] whether Force becomes a separate keyed event.

### 12.18 S27: A few seconds of publication delay at handoff wastes about 2 minutes of work

**Medium priority (griefing); design flaw confirmed; high confidence.** Raised by the GFR subagent. Unlike S10, this requires withholding nothing.

**Location.** [03 §4.3](03_session_manager_en_v0_1.md) lines 259–263 (schedule the highest ready candidate) and W26; [09 §4.5](09_finality_rules_en_v0_1.md) line 278; PUB-4.

**Trace.**

1. (7,6) decides 32 410 at 09:00:10.5; a learns via W26 and enters with the highest candidate cid(32 410).
2. Publication of 32 410 is delayed to $\theta=$ 09:02:15, 4 s after the deadline; 32 409 was published on time.
3. At 09:02:11 the GFR selects cid(32 409), excluding about 120 blocks that a produced on cid(32 410).
4. a re-enters at about 09:02:12; 32 411 through approximately 32 531 become padding.
5. The handoff time is fixed and public; fee increases or namespace congestion can reproduce this at every session boundary.

**Impact.** Flow v0.2 §3.5's conclusion that “handoff has no pause” needs additional conditions.

**Recommendation.** The Manager switches to a higher candidate only after its publication is included with $\theta<\theta_s^{\rm end}$. This costs up to $T_{\rm pub}+T_E$ in confirmation delay instead of 2 minutes of wasted work. The Publisher raises fees for final blocks before $\tau^{\rm stop}$.

**Manual confirmation:** [ ] condition for switching candidates.

### 12.19 GFR subagent's additions to covered items

- **S14 fix F1 requires five synchronized replacements in v9.** Let $\hat\theta_q=\theta_{c(q)}$, where $c(q)$ is the smallest block determined canonically on the finalized chain whose QC chain commits $J_q$. Replace with $\hat\theta$ in: the Espresso event key; the public deadline check and $q_s^{\rm end}$; the watermark; the Force threshold; and the Force window. $\hat\theta$ is nondecreasing, and watermark completeness and same-key rules still hold. E-L's $T_E$ already includes finalization; PUB-4 still gives $\hat\theta\le\tau+T_{\rm pub}+T_E+\Delta_{\rm ts}$, leaving the $T_{\rm grace}$ and $M_a$ budgets unchanged. In A′, the key of $J^*$ moves to about D2 00:00, SessionEnd(7,7) closes (7,7) first, and 36 000 does not enter $G$. Cost: Espresso blocks must expose commit structure, and block $c(q)$ still depends on E-TS. A cheaper approximation is $\hat\theta_q=\theta_{q+k}$, with $k$ at least the commit depth, at the cost of $k$ block times.
- **S16 “cannot start.”** In addition to leaving Tail undefined, this rule is worse than regular SessionEnd replay, which continues to advance. Continuing or recomputing the window as with Force is safe for origin: $\omega_A$ catches up to $\ell^{\rm sch}$ from $A.\mathit{os}=L$, and O1–O3 still hold across padding.
- **Combination of S14 and S23.** Through S23, an old-timestamp block can cause honest readers to disagree about $G$, in addition to causing $G$ to be reverted.

Items confirmed by the GFR subagent: same-key event ordering is strict and complete; Force's $\theta_a\ge\mathit{ts}_{\rm L1}(\ell_U)+M_a$ ensures $\ell_U$ lies within R-WAIT's cut; multiple Forces in one block recheck targets; a Force with the same key as RESUME and a Force preceding RESUME's OPEN step both have determined outcomes; Plans computed in replay satisfy $\ell_s\le x-M_a$ at all three boundaries; resumeReader depends only on L1 and DA before $p_{\rm res}$, so multiple RESUMEs in a block compose correctly; deduplication by hash together with BFT-1 suffices; $q_{\rm avail}$ can only move earlier, and junk messages or split objects cannot push a result past the deadline.

## 13. Items checked but not elevated to new vulnerabilities

### 13.1 v0.1 list (reproduced, with v0.2 notes)

- **Different candidate cids decide different content.** BFT agreement is explicitly limited to one cid; cross-candidate fast confirmation needs an additional product commitment. Two different candidate FINALs do not by themselves establish that a double spend entered D.
  *v0.2 note: S10 shows that an adversary can actively create this divergence and double-spend against regional fast confirmations; see S10 recommendation (a) for whether to make a product commitment.*
- **An Espresso block from before downtime finalizes only after recovery.** This trace is allowed by the model; E-TS constrains proposal time, not when the block later becomes readable. A′ causes G to be reverted at RESUME, but v9 §7 already permits that semantics. Correct the interpretation that “downtime never reverts G”; do not present it as an entirely new consensus vulnerability.
  *v0.2 note: S14 shows that an in-flight block is unnecessary: a block proposed during non-live with an old timestamp can bring the entire downtime period's results into $G$, only for them to be reverted.*
- **Accepted DA data is irretrievable.** v9 explicitly waits; it cannot treat the batch as absent and fill the gap. This blocks derivation and recovery; forced-inclusion liveness does not apply if DA-RET is violated, and no legal path bypassing WAIT was found.
  *v0.2 note: S22 adds the A-CUSTODY duration issue.*
- **Watermark for a timestamp.** The current `≥x` suffices to exclude future Espresso blocks with `<x`; non-Espresso categories precede Espresso at the same key. Together with a strict `<` deadline, no vulnerability requiring a change to `>x` was confirmed.
- **Governance can select a target too far in the future.** The contract has only a cursor monotonicity guard, so the resource and liveness effects of this authorized input do need explanation; however, it violates the off-chain timing procedure. Downtime caused by arbitrary malicious governance was not treated as a capability of ordinary Byzantine regions.
  *v0.2 note: S12 elevates this to high priority: a distant target forces derivation to execute all padding first and can permanently halt forced inclusion; RESUME during normal operation also reverts about 40 minutes of $G$.*
- **Repeated occupation and abandonment by the primary Batcher.** Native permission / timeout rules may prevent an honest alternative from meeting the delivery budget; v9 already states that A-DELIVER does not follow from “an honest Batcher exists.” S2 identifies a more specific interface conflict; the general 1 h delivery guarantee remains a service premise to be realized.
  *v0.2 note: S11 shows a channel can time out even without faults; S13 shows v9 has no interface to remove the primary Batcher or rotate $pk_{\mathsf C}$.*
- **Slashing and stake release.** Release delegates its conditions to abstract P_stake, without an exit isolation period or historical evidence deadline. The omission does not prove an attacker can evade punishment, nor can one claim v9 proves that “all historical liabilities are covered before release.” That property requires an abstract contract and implementation basis.
  *v0.2 note: S17 (banned members retain voting weight) and S18 (specific path for the stake-release deadline).*
- **O1–O3, no false evidence.** This round did not confirm a failure of fixed-origin construction under its stated premises; the concrete BFT evidence relation depends on the specified verifier, and no implementation proof was performed.

### 13.2 Items confirmed after the second round

- **Determinism of $G$.** Under Byzantine regions and Espresso proposers constrained by E-TS, $G$ remains deterministic. A timestamp offset of at most $\Delta_{\rm ts}$ cannot push an honest result timely delivered under PUB-4 beyond $\theta_s^{\rm end}$.
- **Context isolation and slashing within a cid.** Double-signing within a cid is slashable even in an unselected candidate, because SubmitOffense accepts any verified $\nu$. Honest members do not incriminate themselves by signing across cids.
- **Regional instance timing and lifecycle.** With $\le F_s$, Accept, Build, and NotBefore give G1.2 and G1.3. Enter is idempotent, Close irreversible, pause retains locks, and stop is rechecked before resumption.
- **Plan refresh.** Ref is fixed when the predecessor is selected; $\ell_s$ is refreshed through Force ($\mathrm{last}(G)$ and floor) and RESUME. Regions with no eligible proposer are skipped and $3F_s<W_s$ checked. One-level preparation (preparing only the successor of a selected predecessor) cannot be exploited by a single region: the Plan for $s+2$ can be computed about 58 minutes in advance; only GFR stalls caused by R-WAIT have an effect, as recorded in flow v0.2 §5.7.
- **Force.** Force never reverts $G$; a stale Force targeting a closed branch has no effect. O1–O3 still hold after a far-behind regional anchor. Old-branch results cannot enter $G$ ahead of Force in the same $J_q$. v6 issue 1 is fixed in v9: the new window starts after $\theta_q+T_{\rm init}$, and $\omega_A$ starts from $\mathrm{last}(G)$ and satisfies O1–O3.
- **Boundary between RESUME and certification.** RESUME closes the old acceptance window, after which old-branch results cannot enter. Either order of SubmitBatch and CursorSet in the same L1 block is safe: a batch after override fails the prev check, while one before it belongs to $D_L$ bound to the precise RESUME position. The RESUME key requires $L$ to be finalized, so no certificate can cross a RESUME. A stale gpos leaves only S3's induction-base issue. CERT-2 and CERT-3 hold.
- **Deposits.** Neither $\omega_A$ (O1, G2.4), the padding origin rule (at most $L$, timestamp no later than the block time), nor replacement at the same height skips or repeats deposits from any L1 block. A compliant RESUME has anchor origin $L$, and the first new origin $L+1$ is within drift.
- **Forced-inclusion delay.** About $W_{\rm seq}$ under Espresso, regional, or batch-layer failure; exceptions are violation of DA-RET (08 already says D and RESUME wait, with no unsafe bypass) and S12's distant target.
- **Leader discretion.** The leader cannot freely choose timestamp or origin (VP1; Accept checks $\mathit{os}^*$). Clock drift of a minority of honest replicas affects only liveness, but a slow clock can produce late certificates as in S10.
- **Kill.** Apart from the known D lag of about 19 minutes, no missed Kill was found; apart from S5, no false Kill was found.

### 13.3 Known and still present

- **No exit if Espresso is permanently down** (v6 issue 6). Processing RESUME needs an Espresso watermark, so if Espresso never recovers there is no normal recovery path. This premise is still not stated in 00 or 01.

## 14. Cross-cutting repair recommendations

### 14.1 R1: Certifier and Inbox enforce the native deadline

Proposed by the certification and derivation subagent; the lead agent considers the direction sound, with details to check. The native Inbox accepts batches past the window on the rationale that rejecting them “would only stall the cursor” without a remedy ([batch-inbox Rationale](../megaeth_specs/specs/block-certification/batch-inbox.md#rationale)). v9 has RESUME as a remedy, so that rationale no longer applies.

Recommendation: CertifyBatch proves, and SubmitBatch checks, that the L1 block containing the closing submission is no later than the window deadline for the actual parent epoch at the corresponding height (the subagent writes prev.origin + seq_window_size; 6 499 for 36 000 and 6 450 for 35 401). Effects:

- 36 000 in variant A′ and the batches in flow v0.2 §6.4 can no longer advance the cursor.
- A stalled cursor then signifies a fork, making the cursor-based check and “target above cursor” in the 01 procedure meaningful; $D\preceq G'$ holds; CERT-4 is unaffected.

Premise: the invariant holds through the current cursor. Its equivalence to the native Batch Queue's height-by-height check must be checked, including using the earliest height's deadline for a multi-height channel.

### 14.2 R2: Permit RESUME when “already HALT and $G\preceq D$”

The recovery and governance subagent rechecked v6 fix 4 and concluded it holds in v9.

- Once Expire is processed (the watermark is at least its key), Espresso timestamp monotonicity excludes any later block with a smaller $\theta$. Thereafter only RESUME can change $G$, and $D$ only grows. Thus $G\preceq D$ holding when $\mathsf M$ signs still holds at $D_L$, and no blocks are reverted.
- Comparison must use native hashes on finalized $D$, not cursor height (flow v0.2 §6.4).

Benefit: eliminates the approximately two-hour dead zone after HALT caused by a publication failure. Limitations:

- It does not apply when $G$ still has late or old-timestamp results not in $D$ (S14).
- The main cost of publication failure remains $W_{\rm exp}$ (10 h).

HALT also implies S12's on-chain emergency guard and can provide on-chain support for that premise.

## 15. Corrections to flow v0.2

The following statements conflict with findings in this version. Under your working convention, this version does not modify the original flow v0.2; it lists suggested replacements only.

| # | Location | Existing statement | Conflict | Recommendation |
| --- | --- | --- | --- | --- |
| E1 | [Flow v0.2](10_pipelines_en_v0_2.md) §5.7 first item; §7 row “Espresso downtime exceeds about 10 h” | “When there is no in-flight block, all of the original $G$ is retained”; “When there is no in-flight block, $G$ is not reverted” | S14: a block proposed during non-live with an old timestamp can also cause $G$ to be reverted; 00 §1.4 line 191 permits such a block | Change to “when there is no in-flight block and no block proposed during non-live brings in results from the downtime period.” |
| E2 | Same document, §3.4 variant 1b, final paragraph | “This is a conditional trace consistent with BFT-7, not a withholding attack that holds for every implementation” | S10: a Byzantine leader can actively construct the trace by having honest weight insufficient to form a certificate alone vote before stop | Refer to S10 and state “an adversary can construct it; frequency depends on the leader rule.” |

## 16. Suggested manual review order and verification status

Review items sharing a repair direction together.

| Order | Items | What to decide first |
| --- | --- | --- |
| 1 | S23, S24, S1 | Determinism of $G$: evaluate acceptance conditions against the L1 cut determined by the key; definition of $q_{\rm avail}$; separate download progress from the processed frontier |
| 2 | S10, S5, S27 | Handoff: final-block certificate option, Kill propagation only along actual ancestry, candidate-switching condition |
| 3 | S11, S2, S21, R1 | Certifier and Inbox: reliance on finalized L1, timeout effective cursor, decoding consistency, native deadline |
| 4 | S12, S4, S6, S16, R2 | RESUME: contract-computed target or checked time bound, on-chain emergency guard, state after “cannot start,” RESUME after HALT |
| 5 | S13 | Permissions and keys: replace primary Batcher, rotate $pk_{\mathsf C}$ and $pk_{\mathsf M}$ |
| 6 | S14, S25 | Espresso timestamp: F1 or HotShot assumptions; native deadline margin at acceptance |
| 7 | S26 | Whether Force becomes a separate keyed event |
| 8 | S15, S17, S18, S19, S20 | Parameter and model contracts: $T_{\rm dep}$, zero weight upon ban, stake-release deadline, BFT-8, applicable BFT family |
| 9 | S3, S7, S8, S9, S22 | Remaining v0.1 items and custody period |

**Verification status.**

- All four second-round subagents returned reports. The GFR subagent was interrupted once by a rate limit, then resumed from saved progress and completed its work.
- The lead agent checked the cited provisions for every new finding; checked line numbers appear in the relevant “Location” entries.
- The following numbers and claims were not checked against specifications in this repository and are recorded as subagent estimates:
  - S11 throughput numbers (depend on the EigenDA single-blob limit);
  - S20's assessments of Tendermint and HotStuff behavior;
  - Espresso header field semantics used by S25.
- No real BFT, Ethereum, Espresso, or DA network tests were run, and the new counterexamples were not added to [checks/security_counterexamples.py](checks/security_counterexamples.py). This report is not a complete security proof or implementation audit.
- None of the fixes to 00–09 has been written back to the specification; all await your decision.

## Appendix A: Item sources and consolidation

| Item | Source (second-round subagent report number) | Relationship to v0.1 |
| --- | --- | --- |
| S10 | Consensus #1; governance #8 | Elevates flow v0.2 variant 1b; can trigger S5 on demand |
| S11 | Certification #1; governance #3 | Reaches the S2 state without faults |
| S12 | Certification #3, #5 (control-record replay portion); governance #1 | Extends S4 and S6; elevates v0.1 §12's “target too far in the future” |
| S13 | Certification #2; governance #2 | Supplements S2 and v0.1 §12's “primary Batcher” |
| S14 | Governance #4 | Stronger than v0.1 §12's in-flight block; v6 issue 3 |
| S15 | Governance #5 | New |
| S16 | Governance #6; certification #5 | Same kind as S4 §6.2 |
| S17 | Consensus #2 | Concrete instance of S9 |
| S18 | Consensus #3; governance #9 | Elevates v0.1 §12's “slashing and stake release”; v6 S6 |
| S19 | Consensus #4 | New |
| S20 | Consensus #5 | New |
| S21 | Certification #4 | New |
| S22 | Certification #7 | New |
| S23 | GFR #1 | New; concerns determinism of $G$ like S1 |
| S24 | GFR #2 | New; concerns determinism of $G$ like S1 |
| S25 | GFR #3 (L1 stall portion) | Supplements S14 |
| S26 | GFR #4 | New |
| S27 | GFR #5 | New |
| §12.19 | GFR #3 (F1 portion) and notes | Supplements S14 and S16 |
| R1 | Certification #6 | Repairs flow v0.2 §6.4 and variant A′ step 3 |
| R2 | Governance #7 | Rechecks v6 fix 4 and addresses the dead zone |
| §13.2 | “Confirmed” portions of the four subagents' reports | New |
