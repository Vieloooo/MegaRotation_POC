# Independent broadcast-draft presentation review

Reviewed: 25 September 2026. Target: `formalization/main.tex`, 1,124 lines, SHA-256 `69f791f188177c3e45fa7d417e609f71361dedb79406bea73a1b7b63db5fc254`.

Scope: current source only, with fresh checks of the two primary theorem sources. No old PBFT review was used. No changes were made to `main.tex`. Figure content and final PDF layout are excluded because replacement diagrams are being generated separately. Line numbers below refer to this reviewed snapshot.

## Summary

**Final verdict:** The current draft consistently implements the requested LMD-GHOST / synchronous regional broadcast / cross-region FFG hierarchy. Its conditional safety argument is viable; the principal presentation fixes are undefined global-record fields and one missing explicit dissemination instruction needed by the stated export-liveness accounting. No critical mathematical error was identified in this review.

The structure is suitable for a reader: architecture and source boundary → system model → acceptance levels and goals → regional protocol → global FFG → rotation → proposer duties → incentive interfaces → scoped verification. The draft need not be redesigned or reorganized wholesale.

### Findings worth addressing

1. **Global checkpoint fields are not self-contained.**
   - **Location:** lines 487–500: `C=(c,e,g_par,a,k,q_k,mu,AC,rho,alpha)`.
   - **Issue:** `mu` and `rho` are never defined; `alpha` is first explained at lines 661–664. This obscures how an action, health/readiness evidence, and activation proof are carried by the object whose validity is specified next. This is a notation/specification gap, rather than a demonstrated safety failure.
   - **Minimal suggestion:** Immediately after the tuple, identify the action field, action-specific evidence/package, and optional activation proof. Also say which fields are absent for HOLD/CONFIRM and where the ROTATE package `A_R` sits. A short sentence or a compact field list suffices.

2. **DECIDED endorsements are signed but have no explicit send rule.**
   - **Location:** lines 407–415: “After its BB decision, a member signs …”; lines 435–441 and 994–996 rely on certificate dissemination.
   - **Issue/classification:** **Justification Gap** for the operational export-progress claim. Signing alone does not cause another member or P to obtain the endorsements. The serial-start spacing and one-hop export estimate implicitly assume prompt dissemination by honest deciders.
   - **Minimal suggestion:** State that after deciding, each member promptly broadcasts its endorsement to the regional committee; explain how P obtains the resulting certificate. This remains after the local decision and does not add a round before the `2δ` fast decision.

3. **Two compact notational links would remove avoidable rereading.**
   - **Location:** `q(C)` at lines 596–597; `L_c` at lines 154 and 629 versus the schedule family at lines 787–789.
   - **Issue:** The accessor `q(C)` and the contents of the aggregate schedule `L_c` are only inferable. These are editorial notation gaps.
   - **Minimal suggestion:** Define `q(C)` as a checkpoint's input-tip commitment, and explicitly identify `L_c` with the geographic, regional, and proposer schedules returned by selection.

4. **The source-epoch index shadows the regional-instance index.**
   - **Location:** notation table lines 222–224 and FFG vote formula lines 558–561.
   - **Issue:** `u` first means a regional broadcast instance and then means a global source epoch. The scopes are recoverable, so this is not an error, but the notation table suggests a fixed meaning.
   - **Minimal suggestion:** Use a source-epoch symbol such as `e_s` in the FFG formula, or explicitly announce the local reuse. This is optional if avoiding a formula edit is preferable.

5. **The general health guard could identify its certificate object more precisely.**
   - **Location:** lines 622–627: `K_geo >= 2`, followed by the explicit two-HOLD rule and “Records without certificates … do not satisfy this guard.”
   - **Issue:** The reader must infer whether “certificates” means health-report quorums, FFG justification links, or the finalization proof of the first HOLD. The `K_geo=2` example is explicit; the general guard is less executable.
   - **Minimal suggestion:** Name the required proofs for each counted HOLD and retain the stated adjacent-link finalization requirement. This need not change the guard's intended behavior.

## Checks against the requested corrections

| Requirement | Current evidence | Result |
|---|---|---|
| One active region with local LMD-GHOST | Lines 87–93, 331–344 | Clear. LMD selects speculative order; BB votes determine confirmation. |
| Appendix C.1 / Figure 10 regional broadcast | Lines 75–83, 371–405, 879–904 | Correct source and algorithm family. |
| Approximately ten order packets per checkpoint | Lines 222–223, 352–355, 433–445, 1027–1037 | Clear example, separated from native EVM block grouping and from sustained throughput. |
| Cross-region FFG, no PBFT | Lines 108–121, 478–614 | No PBFT stage remains. The only PBFT mention explicitly excludes it. |
| S only needs to order; optional private execution | Lines 62–65, 146–150, 239–263 | Clear. S's private execution does not become a voting requirement. |
| P must execute and publish faithfully | Lines 726–775 | Clear obligation, public assignment, deterministic adapter, and settlement verification. |
| Internal `2δ` versus external decision proof | Lines 280–294, 407–430, 986–998 | Clear and repeated at useful boundaries. Raw vote QC is not advertised as a portable decision proof. |
| Mandatory BA completion | Lines 361–369, 392–400, 435–445 | Clear. Fast deciders still run BA; serial reference excludes unproved overlap. |
| A3 justified is provisional; A4 finalized is durable | Lines 285–294, 582–603, 684–704 | Clear. Publication gate uses A4. |
| ROTATE stays pending until finalized | Lines 537–550, 645–674 | Clear. Finality proof activates a strict-ancestor pending transition in a later descendant. |
| Fixed weights and conditional progress only | Lines 159–170, 605–614, 717–724, 962–984 | Scope retained. Concrete BA and global convergence remain explicit dependencies. |

## Detailed verification log

1. **Weighted intersection, lines 869–877.** “Two quorums … intersect in weight greater than one third.” Correct: the displayed inclusion-exclusion inequality gives strict intersection weight above `W_Q/3`, which exceeds the permitted Byzantine weight. No unweighted count bound is silently substituted.

2. **Broadcast agreement, lines 879–904.** “If some honest member decides … it forwards the QC … before taking their BA inputs.” Correct under the stated common-instance validity, honest forwarding, delay, skew, and boundary-processing premises. At most one value can have a regional vote QC. An honest fast decision forces all honest BA inputs to that value; otherwise BA agreement resolves the common result. The proof treats weighted BA validity as an explicit contract rather than attributing it to the unweighted source theorem.

3. **Good-case latency, lines 176–197 and 897–904.** The draft distinguishes actual delay, known worst-case delay, actual start skew, and the conservative timer allowance. The `2δ` claim is from the broadcaster's proposal/start, matching the source definition. Complete proposal data and idealized processing are stated premises; batch accumulation, export, and mandatory completion are excluded from this internal bound. This matches [Abraham et al., Definition 6, Figure 10, and Theorem 16](https://arxiv.org/pdf/2102.07240v3#page=26).

4. **Portable evidence and local continuity, lines 407–430 and 906–920.** Correct conditional safety argument. DECIDED certificates certify members' actual decisions, and deterministic outcome/authority derivation fixes the next parent. They do not rely on an adversary-supplied receipt timestamp. The send-rule gap above affects progress/accounting, not the inference from an already valid certificate.

5. **FFG finality, lines 552–614 and 922–942.** The checkpoint tree has increasing epoch numbers, justified ancestor sources, supermajority links, persistent double/surround restrictions, and adjacent-child finalization. These support the cited fixed-weight accountable-safety argument. The draft correctly refrains from deriving global convergence from this safety theorem. The source is [Casper FFG, Section 2 and Theorem 1](https://arxiv.org/pdf/1710.09437#page=4).

6. **Activation, lines 537–550 and 653–674.** The pending-state construction avoids making a CONFIRM record depend on the future votes finalizing its parent. A later descendant consumes an ancestor finality proof, clears exactly the pending transition, and then validates its ordinary action. The operational start of the successor and the later global inclusion of its data are distinguished. Compatible finalized rotations preserve the ancestry and input prefix.

7. **Data and execution, lines 944–960.** Correct under the explicit full-range retention, reachability, deterministic execution, and settlement-soundness premises. Availability acknowledgments prove neither correct execution nor unconditional permanent availability; the text says so.

8. **Progress, lines 962–984.** This is deliberately conditional on BA termination, common starts/parents, eligible schedules, common admissible FFG targets/sources, readiness, and external-service assumptions. It is not an unconditional implementation liveness theorem. The excerpt's remaining-choice paragraph is consistent with that scope.

9. **Performance, lines 986–1024.** The separation of `2δ`, external export, serial BA completion, checkpoint throughput, and global-epoch delay is appropriate. The conservative spacing is intelligible once prompt DECIDED dissemination is made explicit. The recovery expression is conditioned on one failed candidate followed by a correct successor and bounded retrieval/start costs; it is not presented as an all-failures universal bound.

## Presentation guidance

Keep the current order of sections and the A1–A5 ladder. The worked traces usefully make the distinction between ten committee members, ten packets per checkpoint, and ten packets per native block explicit. The hidden-late-QC trace is especially valuable because it explains the extra DECIDED certificate without imposing an unexplained extra fast-path round.

After the replacement figures are inserted, perform one final PDF inspection for label legibility, page breaks, and consistency with the four distinctions above: speculative head versus BB decision; member decision versus export; mandatory BA versus fast decision; pending rotation versus finalized activation. This review does not certify those diagrams or the final rendered page count.

## Follow-up on the author's source revisions

The subsequent source now explicitly defines `mu`, `rho`, `alpha`, the effective-generation field, and the `q(C)`/`k(C)` accessors (lines 497–501); names the schedule family including the independent global-control schedule (lines 158–160 and 803–804); uses `e_s` for the FFG source epoch (lines 566–570); and requires immediate DECIDED multicast to every regional member and P (lines 410–417). Findings 1–4 are therefore resolved. The FFG progress condition also now explicitly requires the next adjacent target to use the newly justified checkpoint as its source, which is the link needed for finalization. Finding 5 remains only a request to name the health-guard certificates precisely. Final diagram/PDF review is pending.
