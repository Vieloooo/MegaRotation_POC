# Independent self-containedness review — final PBFT revision

## Summary

**Final verdict: PASS for self-containedness and logical flow under the manuscript's stated model.** All nine findings in `self-contained-review-initial.md` are resolved. The final recheck found no remaining Critical Error, Justification Gap, undefined operational parameter, or missing transition needed by the reviewed arguments.

This verdict follows a full reread of the polished manuscript and both included figures, followed by inspection of every final edit. It concerns the specification and supplied proof flow; compilation, rendered layout, and the independent security audit are separately recorded by the root reviewer. No source edit was made by this reviewer, and `main_v1.tex` was not written.

## Exact reviewed sources

Final copies are retained in `self-contained-final-frozen/`. The final manuscript has 1,497 lines. The hashes were verified after the last changes:

| Source | SHA-256 |
| --- | --- |
| `formalization/main_PBFT.tex` | `42ae5d3fc3b94b84f5a12d71a910909ec045e07363129ce8b4aa43e399f41eeb` |
| `formalization/figures/service-architecture-pbft.tex` | `dcd8e440a75caf1dcbefef99f1ac197c50fbfebdc2d68591d742f69701c5aa13` |
| `formalization/figures/stream-pipeline-pbft.tex` | `cd9e32402772307b8e05391892b7e1cf9792f1a5bbe63db896673579db26a244` |

All main-source line numbers below refer to this final hash.

The final layout-only update was compared directly against the substantively reviewed `f15fc125239c815a14fa06aa1d8733dbfa7a30c684220ca5d3f7c738b2902142` snapshot. The diff contains exactly two changes: line 574 replaces `\Needspace{34\baselineskip}` with `\Needspace{.95\textheight}`, and line 775 adds `\Needspace{.65\textheight}` before the existing finality subsection. No semantic text, line count, or figure source changed. The verdict and source-line references therefore remain valid; the frozen final source was updated to the new hash.

## Resolution of initial findings

| Finding | Final evidence and result |
| --- | --- |
| SC-1: proof-dependent semantic hashes | Lines 279–295 define canonical activation `A`, detached `pi_A`, and genesis `A_0`; `R_0` hashes the semantic activation. Lines 360–364 canonicalize NEWVIEW summaries. Lines 395–399 explicitly detach signatures, signer subsets, and activation witnesses from checkpoint/report identifiers. Lines 752–758 define the pending package's sender-ordered READY identities and detached reports. Resolved. |
| SC-2: partial state assignments and undefined reason | Lines 731–734 copy every effective-parent field before the branch-specific updates. Lines 757–758 give RECOVERY precedence when both guards hold. Genesis is fully initialized at lines 383–390. Resolved. |
| SC-3: incomplete pull timing premise | Lines 188–193 define the live consumer's complete-object path and require request deadlines to cover a complete request and response. Protocol D selects `theta_P` for consumer requests; the complexity section defines `F_P` using that deadline and uses it in consumer recovery. Resolved. |
| SC-4: missing report certificate-signature accounting | Lines 1250–1256 define `lambda_R` and `lambda_G`, including original/inherited evidence and regional commit/finality signatures. Only newly added phase/forwarding-chain signatures and bulk bodies are excluded as stated. Lines 1284–1290 define generic `lambda` before the AB cost formulas. Resolved. |
| SC-5: certificate assembly event | Lines 502–510 explicitly collect a COMMIT quorum, assemble `RC_u`, and accept old-retry votes for assembly without reopening old-retry voting. Resolved. |
| SC-6: acronym, finality, and initialization definitions | The abstract expands L1, PBFT, and FFG; EVM is expanded at its first explanatory use. Setup states positive timing/size limits and includes `K_G,N_G` in `Theta`. Lines 383–394 specify the genesis state, time/epoch convention, trusted base case, justification, finalization, and ancestor finalization before global algorithms use them. The undeclared `pi_r` assignment is gone. Resolved. |
| SC-7: symbol collisions | Metadata uses `lambda_R,lambda_G`, adoption backlog uses `Q_adopt`, and appendix signature chains use `Gamma_r` with `Gamma_0=()`. Regional/geographic latencies retain `L_R,L_G`; `C_0` is solely the genesis checkpoint. Resolved. |
| SC-8: figure/text mismatch | The architecture caption describes role/group boxes. Pipeline line 39 now says “no commit certificate by deadline”; line 71 says “Carry justified control state into the next epoch”. Both figures preserve the Preview/Deliver distinction and original producer on retry. Resolved. |
| SC-9: unexplained legacy comparison | The regional protocol describes its own three phases directly. The comparison at lines 1320–1324 independently defines the alternative per-batch signed-broadcast completion and counts it using Appendix A. No live LMD-GHOST rule or unnamed previous-protocol dependency remains. Resolved. |

## Detailed verification log

### Definitions, roles, and initialization

- The model introduces participants, weights, strict quorum thresholds, sender order, configuration, delays, capacities, failure assumptions, and persistence before protocol operations. Lexicographic sender order at lines 120–121 supplies an exact interpretation of every later sender-ordered selection.
- The indices distinguish region identity, global authorization view, regional batch sequence, retry, original-author retry, checkpoint epoch, and mini-block height. Local retry preserves sequence and parent; global activation changes the global view and initializes a fresh sequence domain.
- The producer/author distinction is maintained throughout: ORDER/BATCH signatures belong to the immutable original author, while PREPREPARE/NEWVIEW signatures belong to the scheduled current primary.
- The activation record identifies its rotation checkpoint, finalizing child, successor, retained tip, and fixed boundary. Its witness is detached. Checkpoint records and finality predicates are explicitly cross-referenced when activation is introduced, then defined before the global algorithms depend on them.
- Genesis fixes the full retained prefix and `S_0=(0,i_0,A_0,h_0,q_0,R_0,0,1,empty)` at time zero, the start of epoch one. `C_0` is an explicit trusted justified/finalized base case, so initialization does not require a nonexistent predecessor or an unexplained quorum link.
- All validity checks used operationally have supplied definitions: complete batch/body verification, prepared/commit certificates, NEWVIEW selection, report validity, deterministic checkpoint transitions, link verification, and consumer acceptance. The generic authenticated-broadcast primitive is fully supplied in the referenced appendix; it is not imported as an undefined black box.

### Regional operational flow and proof dependencies

- The path from complete PREPREPARE to PREPARE quorum, prepared certificate, COMMIT quorum, assembled regional certificate, export, and next sequence is explicit.
- Retry origins are local elapsed-time origins attached to one unresolved sequence. Retirement is persistent. Future proposals are buffered without advancing the retry; recovered nodes without a trustworthy timer origin abstain until a safe later entry.
- VIEWCHANGE is an immutable snapshot. NEWVIEW verifies a full quorum and selects the highest prepared retry, permitting a fresh batch only when every included report is bottom. This rule is compatible with keeping original-author metadata.
- Same-retry uniqueness follows from quorum intersection and an honest sign-once rule. Carryover uses the intersecting honest COMMIT signer, whose higher VIEWCHANGE must follow its old COMMIT because retirement prohibits the reverse signing order. The earliest-conflicting-prepared-view argument therefore preserves any potentially assembled COMMIT quorum.
- Regional-chain uniqueness is an induction over certified predecessors. It does not depend on timely regional messages. Regional progress separately uses the responsive regional quorum, bounded entry skew, full-object capacity, cyclic primary schedule, and sufficient retry duration.

### Global agreement, finality, and authority

- The model now explicitly identifies its synchrony premise as supporting global **agreement and progress**, rather than describing it as only a progress assumption.
- The global proof has a complete local dependency chain: common trusted genesis/parent and activation context; common report-validity predicate; authenticated-origin broadcast agreement; one common report vector; deterministic transition with complete state assignments; one common target; responsive honest weight sufficient for its FFG link; and a common next-epoch state.
- Canonical activation/report/READY encodings make the equality of checkpoint targets literal despite alternative certificate signer subsets. No proof object is required to hash identically merely because it proves the same fact.
- FFG signing, link verification, justification, parent finalization, and ancestor finalization are specified. The actual epoch algorithm uses adjacent links; it does not rely on an unexplained fork-choice mechanism to reconcile conflicting reports.
- A ROTATE checkpoint and its confirming child are fixed before the new activation record is installed. Activation is applied to a strict ancestor at the fixed start of epoch `r+2`; it changes the effective state for later targets without modifying previously hashed checkpoints.
- The pending state preserves the retained parent tip, so READY evidence corresponds to the state the successor must possess. Recovery in the same region creates fresh signing domains and does not reuse an old unfinished regional instance.

### Consumer verification and progress

- Raw producer pushes alone cause no Preview. Preview requires full regional certification and the current activation context; Deliver requires the global finality chain and complete contiguous data.
- Delivery is append-only. Catch-up processes every intervening geographic activation, preventing an old preview suffix from surviving a skipped truncation when later views reuse heights.
- Live-holder response bounds now apply to complete request-response attempts, including verification. Consumer paths have a separate explicit deadline and are used consistently by the repair bounds.
- Liveness is stated with responsive global weight, an eventual healthy eligible region, bounded admissible backlog, and sufficient tenure. The service does not promise inclusion of a particular transaction or fairness. An unadopted preview discarded at handoff has no finalized-delivery promise.

### Communication, performance, and diagrams

- Normal regional phase message traffic is quadratic. Full unaggregated certificate relay still incurs the separately counted cubic signature-bit term. View-change witnesses, body bytes, inherited proof checking, global signed-broadcast traffic, and consumer repair are not silently treated as constant-size work.
- The cost variables distinguish latency from metadata bits and generic AB witness size. The global report bound includes newly appended regional certificate evidence.
- The manuscript counts admitted protocol work under per-sender quotas and complete-object capacity, while separately excluding unlimited wire flooding. It does not turn a latency upper bound into a throughput upper bound.
- Both figure sources were reread. They represent dependency paths, not equal-duration message rounds; the source contains no performance annotation contradicting the body. Regional retry is conditioned on lack of a commit certificate, and the global path distinguishes justification, finalization, and delivery.

## Residual findings

**None identified in this final source-level review.** Introductory service terminology and expressly cross-referenced object definitions are followed by concrete definitions and algorithms before they are relied upon in the proofs. No additional local definition, transition, or external theorem is needed to reconstruct the stated argument.

The theorem scope remains the manuscript's stated scope: fixed configuration, persistent signing state, cryptographic assumptions, complete admitted objects, continuous global synchrony, and the separately stated progress/capacity/tenure conditions. This verdict does not extend those results to arbitrary global partitions or supply an implementation proof.
