# Scheduled L1 status and mini-block timestamps — 29 September 2026

## Scope and frozen artifacts

The user authorized revising `main_v3.tex` to schedule status updates at future
L1 heights and include L1 heights in the mini-block stream as timestamps.
The prior working source and active figures were preserved under
`../build/pre-scheduled-status-20260929/`; pre-existing edits in that source
were retained. Other manuscript versions and presentation/research artifacts
were not edited by this revision.

- Revised source SHA-256: `389430cb9337486a09a5aa1b1b8cec7184fe9243421d1e7332ae5b509515be58`.
- Delivered PDF SHA-256: `ec5d6bb3fcd28414aee4e5ee274402d1c278bc903cfac0fb51e1765af73a403a`.
- Output: 30 pages and nine native TikZ figures.

## Changes and dependency map

| Layer | Revised rule | Downstream dependency |
|---|---|---|
| Mini-block identity, §2.3 | ORDER and the header digest bind finalized L1 height `L_h` and canonical block hash `r_h`; heights do not decrease | Historical source verification, same-batch reference equality, slashing identity |
| Session, §2.5 | Opening context includes `L_0` and effective version `j(L_0)`; new batch timestamps are at least `L_0` | New-region continuation cannot precede its authorizing control boundary |
| Status registry, §4.1 | `E_j=(a_j,chi_j)` announces a strictly later activation height; inclusion height `u_j` satisfies the configured minimum lead distance | Installed entries can remain pending; application cutoff is not an entry field |
| Scheduling, §4.2 | Announcement, preparation and effective application are separate steps | Classification uses the certified authority at activation, not at announcement |
| Effective map, §4.3 | `j(L)=max{j:a_j<=L}`; complete canonical registry prefix authenticates due and pending entries | Producer and consumer cannot substitute another map at a fixed signed height |
| Recovery, §§4.3,5.3 | Independent L1 control progress latches retirement; a separate witness fixes application cut `beta_*` | Stalled application production does not prevent activation; old final work is retained |
| Derivation, §6 | Verify timestamped references, sequential due entries, opening bounds, historical source and immutable boundary witnesses | Rollup relation binds the same data, authority trace and execution claim |
| Guarantees, §7 | Composition remains conditional on regional safety and authenticated closure/order; progress includes L1/control/proof progress | Delay and timestamps are not claimed to construct a closed cut |
| Costs, §8 | Forced path includes `T_act`; `T_cut` is residual recovery after preparation; uncached proofs cost `s_ref` | No hidden L1 wait, proof traffic or claimed numerical recovery bound |

Updated TikZ inputs: architecture, regular rotation, forced rotation, derivation,
and the terminal-drain reference label. Regional voting thresholds, commit timers
and native view-change messages remain unchanged.

## Summary of scoped proof verification

**Final verdict:** the revised composition arguments are consistent with the
explicit regional service and control/recovery premises; this is a scoped
verification of the changed dependencies, not an independent certification of
the entire Sync HotStuff adaptation or an implementation of the recovery service.

**List of findings:** no remaining contradictory activation/cutoff or timestamp
rules were found in the revised scope. The obligations below are explicit
assumptions, not missing steps silently filled by this review:

- Authenticated ordering of control advances relative to sealed handoffs, and
  native-engine-compatible fencing, remain in Assumption 4.2.
- A block-height lead distance alone does not prove usable preparation time;
  §4.2 states the required finalized-notice and delivery condition separately.
- Recovering all protected final prefixes and their bytes remains conditional;
  a schedule, timestamp or elapsed window does not establish completeness.

## Detailed verification log

1. **“a finalized L1 block height and its canonical block hash” (§2.3).** Both
   fields enter the signed header and `q_h`; the reference is immutable across
   re-certification. Nondecreasing heights allow multiple mini-blocks per L1
   block. The logical height is explicitly distinct from an EVM timestamp.
2. **“Installed entries with a_j>L_b remain pending” (§4.3).** Registry root
   authentication covers the installed prefix `J_b`, while activation derives
   `j(L_b)`. An announcement does not prematurely change authority, and a proof
   cannot omit an intermediate due update by showing only the last map.
3. **“This historical validity test is identical at all replicas” (§4.3).**
   An asynchronously newer local L1 head is not an extra native rejection or
   quit rule. This avoids invalidating the native temporal argument that a
   valid forwarded proposal is processed consistently within the regional bound.
   Coordinated retirement still depends on the explicitly stated closure contract.
4. **“A regular handoff initially uses the terminal batch's L_b” (§4.3).**
   Terminal evidence and ordered control extensions determine one selection
   height. Lemma 7.5 uses that height, the unique registry schedule and NextLive.
   If no successor exists, later effective entries can complete that same
   boundary without changing the terminal header or creating competing sessions.
5. **“The recovery witness determines a separate cut beta_*” (§5.3).**
   Application cutoff and L1 activation height have different units. The witness
   is bound to the original trigger/session and must include all protected old
   commitments, including in-flight and notice-window work. A later recovery
   update cannot erase the latched retirement or change an accepted cut.
6. **Cross-region prefix safety (Theorem 7.6).** Within sessions, regional safety
   is assumed. Normal boundaries preserve the uniquely certified anchor; forced
   boundaries preserve every protected prefix and fence old authority under
   Assumption 4.2. New opening-height and continuity checks then preserve the
   application prefix and logical timestamp order. The induction does not infer
   fencing from the clock or the committee signature.
7. **Composed progress (Theorem 7.7).** L1 control progresses independently of
   APP creation, so a failed sequencer is not needed to reach the activation
   event. Finite activation waits, recovery/data progress, eligible successors
   and sufficiently long serving intervals are explicit premises.
8. **Faithful derivation (Theorem 7.8).** Registry/source/rotation checks bind
   the canonical timestamped authority trace; input/DA/execution checks bind to
   the same data. This does not claim a proof circuit or hidden-suffix detector.
9. **Timing and communication (§8).** Cached-reference steady-state bounds
   retain the native pipeline. The forced sum adds scheduled activation and
   charges only residual cut work afterward; preparation is not counted twice.
   New reference proofs are accounted for in the forwarded proposal bit bound
   and in separately fetched/disseminated evidence.

## Boundary-case consistency checks

These are specification-level trace checks, not implementation tests.

| Case | Required outcome checked against the text |
|---|---|
| `E_7` installed, activation 1050, batch stamped 1049 | Earlier effective map; `J_b` may exceed `j(L_b)` |
| Batch stamped exactly 1050 | Apply `E_7`; due updates precede a quota boundary at that height |
| Two consecutive mini-blocks stamped 1049 | Allowed; strict increase is not required |
| Timestamp falls from 1050 to 1049 | Reject the continuation |
| Different L1 references inside one batch | Reject the batch |
| Same L1 height with a different noncanonical hash | Reject its L1 proof |
| Timestamp jumps across an off update and a later live update | Replay both; a latched retirement cannot be erased |
| APP stream stops at batch 100 | L1 control can still reach activation and initiate recovery |
| Notice-window production finalizes through batch 105 | Cut 100 is inadmissible; preserve the higher prefix |
| Old receipt arrives after activation | Verify its historical source and retained cut, not the latest map |
| A hands to B before an update turning A off | At activation, classify against B; do not reselect the sealed A-to-B handoff |
| No eligible successor until a recovery entry activates | Keep the existing pending anchor/obligation; no new failure term |
| Retired session produced no APP | Reuse the authenticated initial anchor without inventing a receipt |
| New session opens at control height 1050 | First fresh batch must have timestamp at least 1050 and extend the cut |

## Build and visual validation

- Built with `latexmk -pdf -interaction=nonstopmode -halt-on-error
  -outdir=build/main-v3-scheduled-status main_v3.tex` from `formalization/`.
- Final log: no undefined references, LaTeX warnings, overfull or underfull boxes.
- Structural checks: every referenced label resolves; no duplicate labels;
  all nine active figure inputs remain present.
- Active source/figure terminology scan: no remaining `beta_j` status-entry
  cutoff, unspecified “configured origin rule”, or old entry/cutoff identity.
- Rendered all 30 pages and inspected five contact sheets; reviewed the header
  definition, effective-map discussion, handoff diagrams and timing table at
  readable size. Shortened diagram labels to keep action boxes separated and
  avoid awkward hyphenation; re-rendered the affected pages afterward.
- `git diff --check` passes. The delivered `main_v3.pdf` is copied from the
  validated build output.
