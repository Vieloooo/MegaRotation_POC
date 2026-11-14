# Control-adapter review for main_v3

Reviewed 2026-09-28 against `formalization/main_v2.tex` (697 lines, SHA-256 `24fcf93848b06cce8df91a321200b4291b3b6b9818938b723254f10d283f1eb6`) and the status-entry outline. This is a design and dependency audit, not an end-to-end proof certification. It does not modify the manuscript.

## 1. Stable interface and state

Keep the existing Regional live stream definition verbatim. Add a Cross-region live stream definition whose wrapper verifies regional outputs and their authority chain. Avoid a second finality stage: a new region starts only after receiving the preceding finalized application anchor and a unique authorization witness.

Use four separate indices: global application batch `b`, mini-block `h`, local physical SMR position `k` in `B_k`, and cross-region term `tau`. A region's application quota is `K`; it is neither a local-view limit nor a bound on physical control descendants.

The committee business entry stays exactly `E_j=(beta_j,chi_j)`. The signed envelope supplies chain/configuration domain, consecutive version `j`, predecessor-entry identifier, and signatures. The verifier state may be richer without enlarging committee business state:

`Gamma=(tau,i,sid,b_start,b,k_app,j,chi,phase,pending)`.

Here `j` is the applied registry cursor, `phase` is active or waiting, and `pending` retains an already-triggered handoff. A passive status update changes only `j,chi`; it does not change `sid`, `tau`, the application quota, or any signed comparison domain.

`NextLive(i,chi)` scans the installed cyclic region list strictly after `i` and returns the first other live region, or bottom if none exists. Waiting is the conservative single-live-region policy. If renewal is later desired, define it as a same-region new term, not an implicit cross-region rotation.

## 2. Terminal Drain: a concrete minimal adapter

The v2 deadline counts fresh physical SMR proposals forever. Stopping application proposals at quota without replacing this workload creates spurious BLAME; merely re-certifying a terminal NEWVIEW anchor does not start a timer. Both defects can be repaired without changing the native commit rule:

1. Extend physical blocks with an application/control type. An application block contributes one batch to the application projection. A `DRAIN` block contributes no application batch, no mini-block, and no transaction. It still has a physical position, parent digest, current-view parent QC, and ordinary PROPOSE/VOTE authentication.
2. Proposal validity counts application blocks in the **proposed ancestry**, not only already committed blocks. Reject an application proposal that would make this term exceed `K`. Otherwise pipelining could issue application batches `K+1,K+2,...` before batch `K` is committed.
3. Once a proposed chain reaches application count `K`, all its fresh descendants are `DRAIN` blocks. They use unchanged forwarding, voting, timers and view-change locks. Both application and DRAIN blocks count as fresh proposals in the native deadline index `p`.
4. View change remains unchanged. If the inherited anchor already has application count `K`, the new leader proposes a fresh DRAIN child after re-certification. Its timer commits the terminal application ancestor. If the inherited prefix is shorter, complete the remaining application batches before draining.
5. Once the terminal application batch has a valid final receipt, disseminate the receipt and close this regional session. Receivers verify and retain the terminal application prefix; no new application or control vote in that session is needed thereafter. Local uncommitted DRAIN blocks have no application output.

This is implementable without inventing a special timer-free commitment rule. Projection of a safe physical SMR log preserves application safety. Under the v2 regional assumptions, an honest-serving view can keep supplying valid fresh blocks, so the native progress argument still applies until the terminal receipt exists. No claim that exactly one DRAIN block suffices through arbitrarily many failed views is justified. In a healthy view the terminal batch's own timer normally completes while control children keep the deadline satisfied; therefore the normal case need not pay an extra DRAIN finalization round.

## 3. Status snapshots need a binding rule

The latest map visible to a receiver is not a protocol state. Two correct receivers can see different finalized L1 prefixes and choose different successors. Bind a status reference to the terminal **regionally finalized object** (or an equivalently finalized boundary descriptor). The reference includes an authenticated finalized L1 origin and the exact registry cursor at that origin; its proof shows no entry within that origin was omitted. Origins and cursors cannot regress.

Every receiver derives the same successor from this reference. Later entries do not reinterpret a completed terminal receipt. A bounded lag / progress policy for origins can improve freshness, but it is a separate requirement. “Monotone origin” alone permits indefinite use of an old map; do not call it prompt revocation.

For passive entries, `beta_j` is an observation/reference height. It does not rewind the application log or create a session. For a forced entry, `beta_j` is a candidate cut and must pass the closed-cut conditions below.

## 4. A genuine race that must be specified

Suppose term A reaches its terminal batch and its final receipt, using old map `chi`, authorizes successor B. Concurrently, a committee entry marks A unavailable at that same batch and its new map would select C. A purely local rule saying “forced takes priority” does not prevent B from already finalizing a child while a second receiver activates C. L1 finality does not itself serialize this off-chain regular handoff with the forced handoff.

Two coherent policies are possible:

- **Regular-first boundary policy.** Finalizing the quota terminal atomically derives the regular successor under its bound snapshot. Later entries are evaluated against that successor as the current active region. An entry marking old A unavailable is then passive. An entry marking current B unavailable may force a zero-application-batch term from the same anchor. Earlier-cut entries may not rewind a completed regular handoff. This changes the outline's force-first tie rule but is straightforward to replay.
- **Force-first under serialized closure.** Retain the outline's tie rule only with a separately stated boundary-serialization/closed-cut condition ensuring that a forced witness cannot coexist with any different irrevocable regular authorization from the same parent. This is a substantive new assumption or implementation obligation, not a consequence of signature verification.

Whichever is selected, state how a stale entry is handled. It must not retroactively reinterpret an already accepted terminal snapshot. If a stale entry still requires retiring the now-current active region, obtaining a current valid cut is necessary; merely applying the old height is unsafe.

**Recommended for the present conditional draft.** Since the manuscript will explicitly assume recovery/control-cut admissibility, retain force priority at the same **unsealed** boundary. Replay the complete consecutive entry prefix designated by that boundary's authenticated control cut against the already certified authority state. If the resulting map leaves its active region unavailable, Force; otherwise, if its application count is `K`, Regular; otherwise, Continue. A sealed normal witness fixes its snapshot permanently. Any later entry is evaluated against its certified successor, not against the former source. The control-cut premise prohibits a conflicting forced cut from coexisting with the sealed normal authority. This gives a readable rule without asserting an unsupported distributed “latest L1 head” oracle.

## 5. Forced cut: what the certificate cannot prove

A committee certificate over `(beta,chi)` proves only that the committee signed those fields. A regional receipt at `beta` proves that one prefix is final. Neither proves absence of a higher finalized suffix or prevents an old session from producing another future receipt.

A safe forced handoff needs all of the following, independently of signature validity:

1. **Prefix inclusion.** Every mini-block already delivered under any affected authorized session is an ancestor of the resolved finalized batch at `beta`. Existing descendants beyond `beta` make that cut inadmissible.
2. **Closure/fencing.** Old authority, including any concurrently issued regular successor authority, cannot subsequently produce accepted application output outside the new continuation.
3. **Unique anchor.** The regional SMR and authenticated authority history yield one finalized batch identity at `beta`. The committee cannot choose between conflicting valid receipts.
4. **Recoverability.** The batch, its ancestry and required data are available to the successor and verifier.
5. **Unique activation.** The applicable committee entry is final in the configured L1 rule, the cursor is consecutive, and one unambiguous boundary transition derives the new authority context.

The all-region crash case exposes an information limit: with unknown surviving receipts, an observer cannot prove that its greatest observed height is the greatest finalized height. A proof system cannot derive that negative fact from a low receipt. Waiting a few L1 confirmations does not create the missing information. The draft should give conditional force-rotation safety/progress and explicitly identify closed-cut installation as an interface assumption until a mechanism is selected.

A possible implementation direction is a low-frequency retirement/recovery procedure that fences sufficient old-region members, collects their locks and stored data, and uses the old SMR's safe recovery to finalize a terminal projection before retiring it. This adds no committee branch-selection right. It is **not yet a complete algorithm**: simple `>2/3` fence ACKs plus `>1/3` final receipts do not automatically give an honest intersection, and the native temporal preservation argument must be used to establish that the gathered locks cover every possible old commitment. If the old group cannot recover enough members, this method does not give unconditional availability.

## 6. Control rules to expose in the paper

| Rule | Guard | State effect |
|---|---|---|
| Passive | Next authenticated entry leaves the currently derived active region live | Apply cursor/map only |
| Continue | Application count below K and no pending forced obligation | Verify region/producer/final receipt; append next batch |
| Regular | Terminal batch finalized; bound snapshot and race policy satisfied | Derive NextLive; install anchor/session; reset application count |
| Force | Next effective entry leaves current active region unavailable and a valid closed cut exists | Retain pending obligation, retire old authority, derive successor from effective map |
| Resume waiting | Pending boundary exists and a later update supplies a successor | Finish that same boundary; do not count a second trigger |

If an active region has already been marked unavailable and the handoff is pending, repeated entries do not create repeated forced terms. Recovery of that same old region does not silently cancel an established retirement; cancellation or same-region renewal needs an explicit policy. Inactive recovery is always passive for a healthy current term.

## 7. Proof boundary and presentation checks

- Existing v2 regional safety requires its synchrony, weighted-fault and retained-state assumptions. Calling a region “crashed” does not prove that a timing assumption violation could not have generated contradictory receipts. The wrapper takes regional stream safety as its black-box premise.
- Cross-region prefix induction is short once each boundary has a unique immutable anchor and unique authority witness. The hard part is supplying these premises at the forced boundary; put it next to the algorithm, not only in a concluding limitation paragraph.
- L1 verification checks the full derivation trace: a non-active recovery event cannot justify an early switch; an ordinary quota switch needs no new committee entry; entries cannot be skipped within the designated registry origin.
- Keep latency clocks explicit. Existing v2 local commitment is `delta_R+2 Delta_R` from ready proposal; the user shorthand `2 Delta_R+delta_X` starts at the terminal vote/timer event and suppresses receipt collection. New-region startup and its first batch's finalization are additional latency. The block interval is a different quantity.

Memory used only for the established separation of authentication, commitment consistency, and execution correctness (`MEMORY.md:279`) and the warning against treating synchrony failure as an automatic crash-only outer fault (`MEMORY.md:266`). Current protocol details were checked from the frozen local sources above.
