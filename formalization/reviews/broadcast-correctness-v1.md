# Fresh verification of the synchronous-broadcast draft

Reviewed: 25 September 2026.

Frozen artifact: `formalization/main.tex`, SHA-256
`69f791f188177c3e45fa7d417e609f71361dedb79406bea73a1b7b63db5fc254`.
All line numbers below refer to that exact artifact. A read-only snapshot was
retained at `/tmp/broadcast-review-69f791f18817.tex`; this review did not edit
`main.tex` or `outline.md`.

The scope is the current **regional Appendix C.1/Figure 10 broadcast,
speculative local LMD, cross-region FFG, and S/P separation**. The older
`correctness-review-v1.md` and `correctness-review-v2.md` concern a superseded
consensus draft and are not endorsements of this artifact. This is a modular
protocol-claim audit, not certification of a complete implementation.

## Summary

**Final Verdict:** The conditional safety composition is supported by the
stated interfaces, but the claimed global conditional-progress implication is
invalid due to one **Critical Error**; the numerical serial-instance spacing
also has one **Justification Gap** in its export communication rule.

**List of Findings:**

- **F1 — Critical Error, global progress only. Location:** lines 605–609,
  “a common justified source” and “Consecutive such rounds give finality”;
  lines 977–979, “The next common adjacent target finalizes it.” **Issue:**
  adjacency of targets does not ensure a link *from* the previous target.
  The source rule permits every vote to keep using genesis. The entire
  common-target/source condition can hold indefinitely while ROTATE never
  finalizes. This does not refute accountable safety.
- **F2 — Justification Gap, serial timing/export liveness. Location:** lines
  408–415, “a member signs” DECIDED and the definition of the exported
  certificate; lines 435–441, “A sufficient conservative spacing”
  `τ_C ≥ 4Δ_loc + 3σ + L_BA`. **Issue:** no recipient/transmission rule is
  stated for DECIDED endorsements. The bound budgets one regional delay
  after the last decision, which is justified by direct endorsement
  dissemination to all participants, but not by an unspecified collector
  and subsequent certificate redistribution. The explicit completion/export
  barrier preserves safety even if the advertised spacing is insufficient.

No other Critical Error was found in the regional agreement proof, portable
decision-evidence argument, local continuity, fixed-weight FFG safety,
finality-gated authority, data-recovery claim, or publication eligibility.
Those conclusions are conditional on the explicitly assumed validated
weighted BA, start coordination, data retention, deterministic execution,
and sound settlement interfaces.

## Detailed Verification Log

### 1. Source and design scope

> Lines 76–83: “The source for the regional broadcast is Appendix C.1,
> Figure 10, and Theorem 16”; instances serialize through completion.

Independently checked the primary [arXiv v3 PDF, Figure 10 and Theorem
16](https://arxiv.org/pdf/2102.07240v3#page=26), including its model of start
skew and BA validity. The draft uses the selected synchronous broadcast,
not an asynchronous reliable-broadcast stopping rule. Its general weighted
quorum, external validity, application parent checks, and exported
endorsements are identified as application extensions. The retained BA and
the distinction between internal confirmation and external evidence are
material requirements of the argument.

The outline and current `synchronous-broadcast-sources.md` and
`ffg-composition-advisory.md` were read as design context, then checked
against the frozen manuscript and primary sources. No claim below relies
on approval of the superseded draft.

> Lines 251–263: “S may execute privately”; P applies the deterministic
> adapter and “cannot insert, permute, or discretionarily filter.”

Correct division of duties. Regional voting validates ordered inputs and
their envelopes, not an EVM execution result. Candidate rejection is a
deterministic adapter operation, and reverted execution is distinguished
from admission rejection. P's execution remains mandatory for publication.

### 2. Quorum and BA dependencies

> Lines 140, 165–170, 361–369 and 869–877: weighted quorum and validated
> weighted BA assumptions.

For committee weight `W_i`, two sets each exceeding `2W_i/3` intersect in
weight exceeding `W_i/3`. Since `B_i < W_i/3`, the intersection contains an
honest signer. Persistent one-value voting therefore excludes two distinct
vote QCs for an instance. Honest weight itself exceeds the quorum threshold,
so Byzantine cooperation is unnecessary for an eligible honest proposal.
These are valid weighted arguments; no identity-count bound is silently
substituted for a stake bound.

The BA contract expressly supplies agreement, termination, honest-input
unanimity, external validity, start-skew tolerance, and proof/data
dissemination. These are assumptions, not proved implementations. They are
sufficient for the stated modular BB proof. In particular, external validity
is needed to exclude an arbitrary unsupported non-bottom BA result. The
manuscript does not claim that the selected single-shot paper supplies a
ready-made weighted application BA.

### 3. Fast cutoff, fallback cutoff, and boundary processing

> Lines 385–400: `t_fast = 2Δ + σ`, `t_BA = 3Δ + 2σ`, boundary-message
> processing precedes the timer, and every member participates in BA.

Let honest party `h` start at real time `s_h` and fast-decide at local time
`t_h ≤ 2Δ+σ`. If party `j` starts at `s_j`, then `s_h-s_j ≤ σ`. The
forwarded certificate reaches `j` no later than local time

`s_h + t_h + Δ - s_j ≤ 3Δ + 2σ`.

Equality is possible. The stated message-before-timer convention is
therefore necessary and supplies the endpoint case: the QC is processed
before `j` snapshots its BA input. Under the idealized message/processing
model, every honest input then equals the unique certified value.
Unanimity preserves the fast decision. If no honest member fast-decides,
BA agreement alone supplies agreement. Late changes to the local lock
cannot alter the frozen BA input or an already chosen outcome.

Processing and payload-transfer costs are expressly excluded from the
idealized two-delay claim at lines 196–197. The result would not directly
cover a real system whose validation queue delays boundary processing;
that implementation is outside the stated model rather than silently
certified here.

> Lines 181–184 and 883–899: actual start skew is at most `δ` in the
> unsynchronized good case, and pre-start messages are buffered.

Correct. If the broadcaster starts at `s_L`, all honest participants start
by `s_L+δ`; the proposal is delivered by that time. Each honest participant
can therefore vote by `s_L+δ`, and all honest votes arrive by `s_L+2δ`.
An early-starting participant's local elapsed time at that real instant is
at most `2δ+σ ≤ 2Δ+σ`, so the votes still meet its fast cutoff. This verifies
the two-delay latency from the broadcaster's proposal. Arbitrary actual
skew bounded only by a much larger `σ` would not supply this good-case
bound; the manuscript correctly states the additional premise.

### 4. Portable evidence and the serial export gap

> Lines 407–430: the external consumer requires a DECIDED quorum and data;
> “A vote QC alone does not prove” timely observation.

The distinction is correct. For a concrete trace, use four equal-weight
members with one Byzantine broadcaster. It obtains votes for `x` from two
honest members and itself, but withholds its own vote from all honest
members until after their fallback snapshots. The third honest member
need not receive a proposal from the faulty broadcaster. Every honest
member sees at most two votes before fallback; all BA inputs are bottom,
so unanimity yields bottom. A later three-vote QC still exists. Its
existence does not certify a decision of `x`.

A DECIDED quorum contains an honest signer, and that signer only endorses
its actual BB decision. BB agreement forces the same outcome for all honest
members. Hashing the exact parent and raw result, then deterministically
deriving the tip and next authority, is sufficient; aggregation encoding
is appropriately excluded from the canonical record hash.

**F2 trace and extent.** Let the earliest current-instance start be `S`.
Allowing worst-case skew, all BA invocations occur by `S+3Δ+3σ`. Taking
`L_BA` as the promised per-invocation completion bound, all honest decisions
exist by `T = S+3Δ+3σ+L_BA`. The displayed spacing leaves exactly one further
delay: `S+τ_C = T+Δ` at equality.

The manuscript specifies signing, but not whom to send DECIDED to. If every
honest member sends its endorsement only to a collector, the collector can
need until `T+Δ` to gather a quorum, and another member can need until
`T+2Δ` to receive the resulting certificate. This uses no faulty member and
violates no network bound. If endorsements are never sent, signing alone
does not make export happen at all. Thus the one-delay export bound needs
a specified dissemination obligation. This is a missing justification of
the latency and progress accounting, not a counterexample to the rule
that the next instance waits for common completion and certificate state.

Assuming that export/common-parent step, the local continuity induction
at lines 911–920 is correct: one completed outcome determines the next
instance's exact parent, input tip, and deterministic leader. Bottom is a
control no-op, not a fabricated packet. Local rotation happens after the
outstanding BA completes, so it does not replace a pending fast decision.
Streaming future packets is explicitly speculative.

### 5. FFG safety versus the source-selection counterexample

> Lines 558–588: exact source/target hashes, ancestor source, double-target
> and surround restrictions, and adjacent-child finalization.

The fixed-weight accountable-safety invocation is appropriate. Independently
checked [Casper FFG, Section 2 and Theorem 1](https://arxiv.org/pdf/1710.09437).
The manuscript's strictly increasing epoch labels, ancestor relation,
justification recursion, and adjacent-epoch finalizing link permit the
same quorum/slashing argument. Gaps in the epoch sequence do not invalidate
that argument. The extra monotone-target/source signing rule only
restricts honest votes further. The proof does not incorrectly claim
that incompatible justified checkpoints are impossible.

**F1 precise admissible trace.** Use a fixed all-honest committee, complete
data, synchronous communication, and a valid ROTATE `R_e` whose planned
guard and readiness condition already hold. Let its ancestry begin at
justified/finalized genesis `C_0`.

1. All members vote `C_0 -> R_e`. This common supermajority justifies
   `R_e`.
2. Propose its direct CONFIRM child `H_(e+1)` with the same pending
   transition and retained input. All members vote
   `C_0 -> H_(e+1)`, rather than `R_e -> H_(e+1)`.
3. Continue a valid consecutive CONFIRM chain. In every later epoch `t`,
   every member votes `C_0 -> H_t`.

Every source is a known justified ancestor. Every target extends finalized
genesis, is fully available, and is on the branch through the highest known
justified target. All target epochs increase; the source epoch stays zero
and therefore never decreases. There is no double target or strict
surround pair because all sources are equal. All honest observers have
the same eligible target and the same justified source, with ample time
for all votes. Thus lines 605–608 and the stated signing/target-selection
rules hold throughout.

Yet no link has source `R_e`, so `R_e` is never finalized. The successor
cannot activate. Adjacent targets alone do not give the link required by
lines 584–587. The inference in lines 977–979 is therefore false under the
stated liveness condition. An assumption establishing the actual adjacent
source-to-target links is missing; this review does not silently import
such an assumption from a desired FFG behavior.

Consequences: the geographic-progress clause of Proposition
`prop:live`, and any unconditional reading of “Consecutive such rounds give
finality,” remain unverified. The regional-progress argument is independent
except for F2's export obligation. Accountable safety, finality-gated
activation, and completed-A4 publication are independent of F1 and remain
supported. This counterexample does not claim that finalization is
impossible under every permitted source choice.

### 6. Pending rotation and authority

> Lines 537–550 and 645–668: ROTATE creates immutable pending state;
> descendants retain its tip until a later ancestor-finality proof is consumed.

These rules avoid a finality/validity cycle. In the successful trace,
justified `R_e` has valid CONFIRM child `H_(e+1)` before any vote finalizes
`R_e`. The actual link `R_e -> H_(e+1)` establishes finality. A strict
descendant `D` can then carry that already established proof, activate
exactly the pending successor, clear the pending field, and check an
ordinary action. Neither `R_e` nor `H_(e+1)` changes hash or derived state.
The proof carried by `D` need not depend on votes for `D`.

Finalizing ROTATE also finalizes its ancestry, so its complete retained tip
is durable. Pending descendants cannot advance the old or new generation.
Different pending rotations may exist provisionally, but conflicting ones
cannot both finalize under the established FFG safety assumptions. Stale
old-region A2 service is explicitly possible; it cannot obtain conflicting
A4 acceptance. The A2/A3 suffix cancellation claim is consequently scoped
correctly and does not erase finalized input.

### 7. Availability, jobs, and execution endpoints

> Lines 502–512 and 944–960: acknowledgers possess the complete exact range
> and local exported decision proofs.

Subtracting Byzantine weight below one third from acknowledgement weight
above two thirds leaves honest acknowledged weight above one third.
Retention plus the explicit reachable-holder assumption supplies recovery.
The certificate alone does not prove perpetual retrieval or execution,
and the manuscript does not assert otherwise.

> Lines 735–747: public job activation, block-aligned endpoints, and a live
> activator; lines 762–773: checked range continuity and reassignment.

The activation event is independently observable and requires no accepting
acknowledgment from P. With packet groups of size `K_B`,
`K_B floor(k/K_B)` is exactly the greatest completed-block endpoint at or
before an A4 frontier `k`. A partial finalized block therefore produces
no premature irreversible job. If a retained geographic tip is inside a
block, the finalized prefix and its fixed first-packet context survive;
publication still waits for a complete finalized group.

The job contract's range/continuity checks, immutable accepted ranges,
deadline rule, and reassignment preserve the certified input obligation.
Faithful A5 execution follows only with the expressly assumed deterministic
adapter and sound proof/dispute verifier. Native signing availability is
an explicit premise, not supplied by rotating the ordering sequencer.
Publication progress is already conditioned on an advancing completed A4
endpoint, a live activator, an eventually correct P, sufficient deadlines,
and the external services. F1 prevents deriving that advancing A4 endpoint
from the present global progress condition alone.

## Dependency and limitation record

The verified safety dependency chain is weighted quorum uniqueness plus
timely QC forwarding and validated BA; then actual DECIDED evidence and
serial exact-parent continuity; independently, fixed-weight FFG accountable
safety; then deterministic pending/activation validity; finally finalized
input ranges and sound deterministic execution settlement.

The draft deliberately leaves the concrete weighted BA, start coordination,
global convergence, health-policy convergence, native adapter, dynamic
membership, and incentive equilibrium outside its present proof. These
explicit interface assumptions were not reclassified as errors merely
because their implementations are deferred. F1 is different: its conclusion
fails even when the presently stated common-view premises hold. F2 concerns
the precise communication obligation needed for the displayed serial bound.

The frozen manuscript remained unchanged through the source/trace checks.
