# v9 Protocol Security Review: Core Counterexamples and Open Questions

Date: 2026-11-02. Scope: the v0.1 specifications 00–09; line numbers cited below refer to the Chinese source; [calibration pipelines v0.2](10_pipelines_en_v0_2.md) are illustrative examples, not additional protocol rules.

At your request, this round used four independent subagents to review GFR replay, regional consensus and handoff, certification and derivation, and governance and recovery. This record presents their findings after the lead agent checked each one. An agent interrupted in the previous round did not produce a complete report and is excluded from these conclusions.

This report does not classify “a candidate regional result is ultimately not selected” or “RESUME replaces a suffix of G not yet derived unchanged” as BFT safety vulnerabilities in themselves; both are permitted by the current design. The priorities are whether the same public history has a unique interpretation, whether source proofs support the next module's conclusions, and whether the liveness premises can all be satisfied.

Recommended first reading: **S5 (incorrect Kill propagation) → S1 (publication visibility boundary) → S2 (timeout cursor)**, followed by the properties and trust boundaries in S3–S4. S6–S9 concern recovery feasibility and model contracts.

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

## 12. Items checked but not elevated to new vulnerabilities

- **Different candidate cids decide different content.** BFT agreement is explicitly limited to one cid; fast confirmation across candidates needs an additional product commitment. Different FINALs from two candidates do not establish that a double spend has entered D.
- **An Espresso block from before downtime finalizes only after recovery.** This trace is allowed by the model; E-TS constrains proposal time, not when it later becomes readable. A′ causes G to be revoked on RESUME, but v9 §7 already permits this. Correct the interpretation that “downtime never revokes G”; do not portray it as a new consensus vulnerability.
- **Accepted DA data is unretrievable.** v9 explicitly says WAIT; it cannot be treated as an absent batch and filled with empty content. It blocks derivation and recovery; forced-inclusion liveness does not apply if DA-RET is violated, and no legal entry point bypassing WAIT was found.
- **Watermark with equal timestamps.** Current `≥x` suffices to exclude future Espresso blocks with `<x`; non-Espresso categories precede same-key Espresso events and the deadline uses strict `<`. No vulnerability requiring `>x` instead was confirmed.
- **Governance can select a target too far in the future.** The contract has only a cursor-monotonicity guard; resource and liveness effects of such an authorized input need explanation, but it violates the off-chain timing procedure. Downtime caused by arbitrary malicious governance was not treated as an ordinary Byzantine region's capability.
- **Repeated occupation and abandonment by the primary Batcher.** Native permissions/timeout may prevent an honest backup from satisfying the delivery budget; v9 explicitly says A-DELIVER does not follow from “an honest Batcher exists.” S2 identifies a more specific interface conflict; a general 1 h delivery guarantee remains a service premise to be implemented.
- **Slashing and stake release.** Release delegates conditions to abstract P_stake without specifying an exit quarantine or historical-evidence deadline. This omission proves neither that an attacker can evade penalties nor that v9 establishes “all historical liability is covered before release.” If needed, the abstract contract and implementation basis must be supplied.
- **O1–O3 and no false evidence convictions.** This round did not confirm failure of the fixed-origin construction under its stated premises; the concrete BFT evidence relation depends on a specified verifier, and no implementation proof was performed.

## 13. Suggested manual review order and verification status

| Order | Item | First decision |
| --- | --- | --- |
| 1 | S5 incorrect Kill propagation | Does the successor actually inherit the conflicting prefix? |
| 2 | S1 publication visibility boundary | Are download progress and processed frontier distinct? |
| 3 | S2 timeout cursor | How does native takeover map to v9? |
| 4 | S4, S6 governance inputs | Safety trust scope; treatment of low targets and late inclusion |
| 5 | S3 CERT-4 | Add the initial RESUME cut as the induction base |
| 6 | S7–S9 | Control quiet period, historical-state preparation, cross-session key model |

**Verification status.** All four new subagents returned candidate findings, and the lead agent checked the minimal traces and provisions cited above. The governance agent did not exhaust every recovery path; this report is not a complete security proof or implementation audit. No real BFT, Ethereum, Espresso, or DA network tests were run. The [example arithmetic script](checks/calibrate_261027.py) and [seven minimal predicate counterexample calculations](checks/security_counterexamples.py) were run; the latter checks only equations, inequalities, and empty-set quantifiers in the report's traces, not a complete state-machine reproduction. At review completion, hashes of 00–09 and old pipelines v0.1 matched the review baseline. All repair directions still await your manual confirmation.
