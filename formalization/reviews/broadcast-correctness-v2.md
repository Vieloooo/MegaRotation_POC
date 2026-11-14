# Verification of the corrected synchronous-broadcast draft

Reviewed: 25 September 2026.

Frozen artifact: `formalization/main.tex`, SHA-256
`eeefda017320e101cb5be660ce2b8132844daa829b541ba3c36ae8a25188328d`.
All line numbers in this report refer to that exact artifact. A snapshot
was retained at `/tmp/broadcast-review-eeefda017320.tex`. This reviewer did
not edit the manuscript.

This is a follow-up to [broadcast-correctness-v1.md](broadcast-correctness-v1.md),
which remains frozen against its earlier SHA. The complete diff was checked,
the changed export/voting/progress rules were reread in context, and their
effects on the earlier verified dependencies were checked. The earlier
unrelated consensus reviews are not used as endorsements.

## Summary

**Final Verdict:** The supplied conditional protocol arguments are valid
under their stated modular assumptions; both findings from the first
broadcast review are resolved, and no remaining Critical Error or
Justification Gap was found in the audited claims.

**List of Findings:** None remaining. The validated weighted BA,
start-coordination service, common admissible FFG targets and productive
adjacent links, retention/retrieval, deterministic adapter, native signing,
and sound settlement remain explicit assumptions or deferred
instantiations. This verdict is not a claim of complete implementation,
unconditional global convergence, dynamic-membership safety, deployed
MegaETH compatibility, or an incentive equilibrium.

## Detailed Verification Log

### 1. Export transmission and serial spacing: F2 resolved

> Lines 412–413: “Immediately after its BB decision, a member signs and
> multicasts to every regional member and P.”

This supplies the missing dissemination obligation. Every honest member
now receives every honest DECIDED endorsement within one regional delay
after the latest honest decision. Since honest weight exceeds two thirds,
each can independently form the same-result certificate; there is no
mandatory collector-then-redistribution hop.

> Lines 440–446: the next instance waits for previous BA completion and
> regional evidence; `τ_C ≥ 4Δ_loc + 3σ + L_BA`.

Let `S` be the earliest scheduled local start in an instance. The latest
start is at most `S+σ`; the latest BA invocation is at most
`S+3Δ+3σ`. The stipulated BA bound places all completions by
`T=S+3Δ+3σ+L_BA`. Immediate all-to-all DECIDED dissemination supplies
honest-supermajority evidence everywhere by `T+Δ`. The displayed spacing
therefore covers the stated serial reference schedule, with the existing
boundary-processing convention and common-start/parent premises.
The independent completion/evidence barrier still prevents an unresolved
instance from being skipped. External P delivery remains outside this
purely regional bound.

### 2. FFG source selection and conditional progress: F1 resolved

> Lines 571–574: the source is the highest-epoch known justified ancestor;
> old epochs are not backfilled and future epochs are not signed.

The first review's all-genesis-source trace no longer obeys the honest
source rule once the previous target's justification is known. Target
selection and source selection are now separately specified, and persistent
double-target/surround/monotonicity checks still apply.

> Lines 622–625: after `C_e` is justified, that proof is common before the
> next cutoff; the common next target is its direct child `C_(e+1)`, with
> source equal to `C_e`.

This supplies the exact premise absent from the earlier implication.
Responsive honest weight exceeds two thirds. The first productive round
therefore supplies a link from a justified source to `C_e`; the second
supplies the actual link `C_e -> C_(e+1)`. The direct-child and consecutive-
epoch requirements make that second link a finalizing link. Merely having
consecutive targets is no longer treated as sufficient.

> Lines 989–1002: Proposition `prop:live` invokes the condition of the FFG
> section and explicitly uses the newly justified checkpoint as the next
> source.

The corrected proof follows from the corrected condition. Applying the
productive pair to valid HOLD records can establish the specified health
guard; applying it to a valid ROTATE and CONFIRM child finalizes ROTATE.
Readiness and the explicitly required initialization/start coordination
then permit successor service. The result remains conditional on common
eligible proposals and admissible signing histories; it does not prove
that the target selector or health reports converge in all executions.

This liveness correction does not weaken accountable safety. The original
fixed-weight FFG slashing conditions, exact hashes, justification recursion,
and adjacent-child finalization remain in force.

### 3. Earlier safety dependencies: retained

The complete diff makes no weakening change to the following verified
arguments from the first report:

- **Regional agreement:** weighted quorum intersection prevents two vote
  QCs. A fast decision at local `2Δ+σ` is forwarded to every honest member
  by local `3Δ+2σ`; processing at the equality boundary precedes the BA
  snapshot. Unanimity of the validated weighted BA preserves it. If there
  is no fast decision, BA agreement supplies the result. Fast deciders
  continue participating through termination.
- **Good-case latency:** the actual-start-skew premise `≤δ` and buffering
  imply honest proposal processing by `s_L+δ` and vote delivery by
  `s_L+2δ`. The advertised `2δ` remains internal confirmation from proposal,
  excluding batch accumulation, export, processing, global finality, and
  publication.
- **Portable decisions and local continuity:** DECIDED endorsements
  certify actual decisions. A late raw vote QC cannot overturn a bottom
  BA decision. Serial completed instances share their exact parent and
  deterministic authority; local changes happen after completion.
- **Geographic safety:** fixed-weight FFG excludes conflicting finalized
  checkpoints. ROTATE is immutable pending state; its CONFIRM child is
  valid before ROTATE finalizes. A later strict descendant carries the
  existing finality proof and consumes exactly the pending transition.
  Finalized input is retained; only provisional tails may be cancelled.
- **S/P and publication:** S provides ordered inputs and may execute
  privately; P must execute the deterministic adapter. Only completed
  A4 block ranges are eligible for public job activation. The endpoint
  `K_B floor(k/K_B)` avoids publishing an incomplete finalized block.
  Correct execution acceptance additionally depends on the stated sound
  settlement verifier, and progress depends on the named service premises.

The primary-source checks supporting the inherited audit are [the selected
v3 broadcast paper, Appendix C.1/Figure 10/Theorem
16](https://arxiv.org/pdf/2102.07240v3#page=26) and [Casper FFG, Section
2/Theorem 1](https://arxiv.org/pdf/1710.09437). The weighted application
adaptation and the export, rotation, and publication composition are
manuscript arguments, not extra theorems attributed to those papers.

### 4. Other changes

The diff additionally removes an unused macro, makes the subsection title
safe for PDF metadata, names the separate schedule components, defines
candidate fields and the post-activation generation, and renames the FFG
source-epoch variable. These changes do not alter the checked proof
dependencies. The later report intentionally supersedes the first review's
verdict only for the frozen SHA recorded above.

The last checked diff also makes order-packet terminology explicit, fixes
the reference dependent-instance pipeline depth at one, and strengthens
the general `K_geo` guard: every counted HOLD has reports and justification,
epochs are consecutive, and every successive pair has the adjacent link.
These requirements support the stated penultimate-HOLD finalization and
exclude a gap for `K_geo > 2`; they do not create an extra authority path.
