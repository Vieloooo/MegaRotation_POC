# Sequencer-rotation service specification

## Sync HotStuff RR v4.5

[`sync_hotstuff_rr_v4_5.tex`](sync_hotstuff_rr_v4_5.tex) revises v4.4 for
clarity. Every protocol step, section number, and protocol number is kept.
The revision defines each shared concept once: the Force deletion scope
`Del(F)`, the session ID `σ_r` (which replaces the second meaning of
"opening"), and the branch. It adds a participants table and removes
repeated rationale. It also fixes 16 ambiguous or unhandled cases,
including the L1 ordering of status updates, genesis selection, Forces
whose target a node does not know yet, and receipt checks by external
verifiers.

Blue text marks every change from v4.4, and each tag links to the change
log in Appendix A. Set `\revmarksfalse` for a clean copy. See the
[PDF](../output/pdf/sync_hotstuff_rr_v4_5.pdf), the
[word-level diff from v4.4](../output/pdf/sync_hotstuff_rr_v4_5_diff_v4_4.pdf), and the
[review and revision plan](reviews/sync-hotstuff-rr-v4-5-2026-10-12/revision-plan.md).
Build from this directory with
`latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/sync-hotstuff-rr-v4-5 sync_hotstuff_rr_v4_5.tex`.

## Sync HotStuff RR v4.4

[`sync_hotstuff_rr_v4_4.tex`](sync_hotstuff_rr_v4_4.tex) keeps v4.3's
regional BFT, DATA/DRAIN rules, and regular quota handoff. Sections 5--9
define authorized regional finality, observer global finality, rotation,
and the Submitted / L1-finalized / L1-activated Force handler.

The global wait starts after an observer validates and relays complete
regional commitment evidence. It does not use the block's older L1
reference as its starting time. The model states bounds for monitoring,
the honest committee's decision, and Force finalization. The committee
protects receipt-backed prefixes internally; there is no KEEP message or
new certificate round. One Force remains outstanding until activation.
The draft includes a conditional protection argument, not a new proof of
the weighted regional engine.

See the [PDF](../output/pdf/sync_hotstuff_rr_v4_4.pdf) and the
[complete Chinese translation and explanation of Sections 5--9](sync_hotstuff_rr_v4_4_sections_5_plus_zh.md).
Build from this directory with
`latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/sync-hotstuff-rr-v4-4 sync_hotstuff_rr_v4_4.tex`.

## Sync HotStuff RR v4.3

[`sync_hotstuff_rr_v4_3.tex`](sync_hotstuff_rr_v4_3.tex) rewrites v4.2 for
clarity, using [v3.3](sync_hotstuff_rr_v3_3.tex) as the style baseline.
The protocol boxes now follow v3.3's format, and red marks every
difference from v3.3 (renamings are listed once and not marked). The
old Protocol 3 is split into Protocol 3 (external chain, every node) and
Protocol 4 (bootstrap, successor members). Definitions come before each
pair of boxes; each rule is stated once. The draft defines terms that
v4.2 used without definition, makes the force selection height
deterministic, and fixes the "FORCE additionally requires" wording of
Ready. Two figures show the external session states and the two ways a
session ends. Security and complexity analyses remain deferred.

See the [PDF](../output/pdf/sync_hotstuff_rr_v4_3.pdf) and the
[review and revision plan](reviews/sync-hotstuff-rr-v4-3-2026-10-12/revision-plan.md).
Build from this directory with
`latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/sync-hotstuff-rr-v4-3 sync_hotstuff_rr_v4_3.tex`.

## Sync HotStuff RR v4.2

[`sync_hotstuff_rr_v4_2.tex`](sync_hotstuff_rr_v4_2.tex) simplifies the L1
record fields and the four assumptions. Regional synchrony follows v3.1's
three-bullet wording; the backbone and honest-multisignature assumptions are
one sentence each. Activation, external truncation, and request handling are
specified in the definitions and Protocol 3. Explicit forwarding of complete
bootstrap bundles replaces the separate readiness-service assumption.
Protocols remain marked in red relative to v3.1. Security and complexity
analyses are deferred.

See the [PDF](../output/pdf/sync_hotstuff_rr_v4_2.pdf). Build from this directory
with `latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/sync-hotstuff-rr-v4-2 sync_hotstuff_rr_v4_2.tex`.

## Sync HotStuff RR v4.1

[`sync_hotstuff_rr_v4_1.tex`](sync_hotstuff_rr_v4_1.tex) separates regional
commitment from the external canonical history. Finalized L1 force instructions
truncate a session's external output without changing its local timers, locks,
quota, or commitments. Active-session output is externally provisional; sealed
retained prefixes are final. Section 1.1 gives a simpler five-item account of
what nodes read from L1. The draft clarifies timely authority, trusted control
progress, signing domains, and pending requests after truncation.

Protocols 1–2 retain red changes relative to RR v3.1, and new Protocol 3 is red.
Security and complexity analyses remain deferred. See the
[PDF](../output/pdf/sync_hotstuff_rr_v4_1.pdf). Build from this directory with
`latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/sync-hotstuff-rr-v4-1 sync_hotstuff_rr_v4_1.tex`.

## Sync HotStuff RR v4

[`sync_hotstuff_rr_v4.tex`](sync_hotstuff_rr_v4.tex) extends the RR v3.1
protocol with stake-weighted voting, explicit L1 control objects, global
views `(r,g,t)`, a regional DATA quota, and one recovery-only DRAIN position
per generation. Patched Protocols 1–2 show changes in red; the new Protocol 3
continuously watches the stream and finalized L1 status, then bootstraps from
a regular or activated forced-handoff event. Security and complexity analyses
are deferred. Control ordering, forced closure, and external anchor retrieval
remain explicit service assumptions.

See the [PDF](../output/pdf/sync_hotstuff_rr_v4.pdf). Build from this directory
with `latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/sync-hotstuff-rr-v4 sync_hotstuff_rr_v4.tex`.

## Staking and rotation module

[`staking_rotation_module_v2.tex`](staking_rotation_module_v2.tex) is the current
six-page standalone high-level design for permissionless whole-position joins
and exits, frozen epoch configurations, region and in-region leader orders,
multisignature eligibility, and timeout behavior. Regional BFT view change is
an external service; safe cross-session activation additionally requires its
terminal-closure interface. Every cross-region handoff is serialized on L1.

The design--review--patch sequence preserves
[`v1`](staking_rotation_module_v1.tex), its
[security verdict](reviews/staking-rotation-module-2026-10-10/review_v1.tex), and
the [v2 recheck and iteration record](reviews/staking-rotation-module-2026-10-10/review_v2.tex).
Three independent v2 reviews found no remaining critical errors or justification
gaps within the explicitly conditional interface scope. See the
[final PDF](../output/pdf/staking_rotation_module_v2.pdf).

Build from this directory with
`latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/staking-rotation-module-v2 staking_rotation_module_v2.tex`.

## Minimal standalone rotation candidate

[`minimal_regular_rotation.tex`](minimal_regular_rotation.tex) is a three-page,
self-contained candidate based on raw Sync HotStuff. It combines terminal-QC
normal entry, a local view-entry clock for the original BLAME deadline, and
the original failure-recovery waits in one protocol box. The explicit choice
to finish an ongoing exit wait before normal entry, together with the raw
commit timer without a QC guard, admits the safety counterexample in the
document. This is a rejected candidate, not a verified replacement for v3.1.

See the [PDF](minimal_regular_rotation.pdf) and
[verification record](reviews/minimal-regular-rotation-2026-10-10/README.md).

## Version 3.1: terminal-QC normal local rotation

[`main_v3_1.tex`](main_v3_1.tex) implements the vanilla normal-rotation idea
as a protocol draft. Protocol-box additions and changes are red. Terminal QC
enables the next leader; BLAME identifies the view whose proposal is missing.
There is no separate missing-QC BLAME condition or extra handoff ACK round.
The draft distinguishes normal advancement from quit, includes the
three-delay normal proposal timeline, and makes remaining mixed-entry proof
obligations explicit. Original v3 and v4 files are preserved.

See the [PDF](main_v3_1.pdf) and [revision/validation record](reviews/main-v3-1-regular-rotation-2026-10-09/README.md).
Build with
`latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/main-v3-1-rotation main_v3_1.tex`.

## Version 4: local rotation discussion

[`main_v4.tex`](main_v4.tex) preserves the current v3 specification and adds
Section 9 on normal in-region leader rotation and further optimization.
It compares Flow, Aptos and MonadBFT using official sources and pinned Flow/Aptos
implementation snapshots. The discussion separates fixed-boundary completion,
optional READY/OPEN initialization, and an engine with a pipeline across leaders.
The inherited quota-BLAME experiment is retained with an explicit progress-proof
scope note; none of the candidates is presented as a proved protocol.

See the [PDF](main_v4.pdf), [Chinese reading note](reviews/main-v4-rotation-2026-09-29/discussion-zh.md),
and [source/validation record](reviews/main-v4-rotation-2026-09-29/README.md).
Version 4 has independent figure copies; v3 sources, figures and PDF remain untouched.

Build with
`latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/main-v4-rotation main_v4.tex`.

## Version 3

[`main_v3.tex`](main_v3.tex) composes the preserved regional live-stream interface
into a cross-region service. Each mini-block header binds a finalized L1 height and block hash. Status
entries schedule future L1 activation heights, with separate announcement
and effective versions. It includes fixed-quota and active-failure handoffs,
and a proof-system-neutral rollup derivation relation.
Inactive-region status updates only change eligibility; the schedule skips
unavailable regions. Section 8 compares finality, pipeline interval, messages
and bits across four operating paths.

The [PDF](main_v3.pdf) is 30 pages with nine native TikZ figures in
[`figures/v3/`](figures/v3/README.md). Section 3.4 includes a worked
three-batch example and a terminal-drain pipeline figure showing quota
enforcement, recovery across a local view change, and final receipt export.
The [scheduled-status revision](reviews/main-v3-scheduled-status-2026-09-29.md)
updates definitions, handoffs, derivation checks, conditional guarantees and
costs together. L1 activation heights and application recovery cuts remain
distinct; a preparation window does not replace safe closure or data recovery.
The
[readability revision](reviews/main-v3-readability-2026-09-29.md) adds first-use
definitions, a counter guide, consistent timing notation and improved page
breaks, while preserving every formal definition, assumption, result, proof
and protocol block. See the earlier
[validation and independent-review record](reviews/main-v3-2026-09-28/validation.md)
for preserved definitions, build/render checks and conditional recovery scope.

Build from this directory with
`latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/main-v3 main_v3.tex`.

## Version 2 and earlier drafts

[`main_v2.tex`](main_v2.tex) is the 28 September major revision, organized into
six sections: two-layer structure; model and service definitions; regional PoS
SMR; stream delivery and retrieval; security; and complexity. Its stream
interface is independent of the BFT message format. The regional protocol uses
the May 2020 Sync HotStuff revision, including native event-driven view change
and NEW-VIEW anchor re-certification, with L1 stake-weighted certificates.
Cross-region CFT, rotation and global finality remain unfinished.

The incremental slashing/blame update adds an explicit leader double-head
evidence predicate, L1 verification and per-slot penalty deduplication in
Section 3.4, and a detailed regional blame/view-change communication figure.
See [`reviews/main-v2-slashing-blame-2026-09-28.md`](reviews/main-v2-slashing-blame-2026-09-28.md)
for the scope, independent review and PDF validation.

BLAME certificates and post-commit receipts now require weight greater than
one third; ordinary vote QCs retain the greater-than-two-thirds threshold.
See [`reviews/main-v2-thresholds-2026-09-28.md`](reviews/main-v2-thresholds-2026-09-28.md)
for the threshold rationale and updated build checks.

The compiled copy is [`main_v2.pdf`](main_v2.pdf). The five active TikZ figures
are in [`figures/v2-major/`](figures/v2-major/README.md); earlier `figures/v2/`
files are historical and are not dependencies of this revision. See
[`reviews/main-v2-major-validation.md`](reviews/main-v2-major-validation.md)
for the pinned source, independent reviews and build/render checks.

Build with `latexmk -pdf -outdir=build/main-v2-major main_v2.tex`.

[`main.tex`](main.tex) is the proposed protocol rewritten from the current service-level discussion. The service provides an L1-authorized, verifiable mini-block stream, local failover and geographic failover. Producer execution, EVM block assembly, DA/L1 publication and economic penalties are outside its interface.

The manuscript includes the model and service contract, mathematical message and certificate objects, protocol boxes, worked traces, correctness proofs and latency/communication accounting. The data path uses leader push, regional pull repair and backbone/pull synchronization. The global and local view numbers are denoted by `v_G` and `v_L`; `P` is the external proposer.

Regional production follows clock slots and scheduled leader terms. A leader may propose one signed mini-block batch per slot; an empty slot does not block later slots. Members exchange ordinary HEAD votes and use LMD-GHOST to choose provisional branches. `SetPreview` may replace the speculative suffix, including after a local leader change. There is no local VOTE quorum, DECIDED certificate, GAP outcome, or completion for each batch, and no mini-block finality guarantee.

The global layer retains signed report agreement at epoch boundaries. Fresh active-region REPORT head endorsements select a common LMD branch from the reported candidate union. Adjacent Casper FFG links justify and finalize checkpoints and authorize global handoff at fixed boundaries. A new justified adopted prefix is installed at the next epoch boundary and then constrains regional branches; finalized `Deliver` outputs are append-only. The main guarantees concern global prefix safety and eventual adopted-height progress, not identical local heads or persistence of an individual preview.

This design applies LMD-GHOST and Casper FFG ideas to the stated model; it is not an exact description of Ethereum's deployed consensus. Epoch-level signed report agreement remains expensive. Its cost, together with replication, synchrony, quorum, retention and workload assumptions, is included in the analysis.

Build in this directory:

```sh
latexmk -pdf -interaction=nonstopmode -halt-on-error main.tex
```

The default output is `build/main.pdf`; `main.pdf` is the delivered copy. Both active figures are native TikZ inputs in `figures/`. See the [LMD/FFG redesign review record](reviews/lmd-ffg-redesign/README.md) for the independent proof, flaw-finding, revision and validation cycle.

The original user outline is preserved. The pre-rewrite manuscript and PDF are retained under the ignored `build/pre-rewrite-20260925/` directory. Earlier figure exports and reviews are historical and are not dependencies of this manuscript.
