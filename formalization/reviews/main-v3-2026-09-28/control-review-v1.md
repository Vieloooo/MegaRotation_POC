# main_v3 control review v1

Reviewed the 1,449-line draft on 2026-09-28. Scope: Section 2 interfaces, terminal Drain, Sections 4–5 control/rotation, and Section 7 composition. No manuscript changes made. This is a bounded logical review of the conditional model, not a claim that the external recovery/control mechanism is implemented.

## Summary

The conditional architecture is coherent after the pending-obligation correction below. The manuscript now explicitly distinguishes the trusted/admissible recovery cut from what a committee signature proves; no additional closed-cut mechanism is demanded by this review. The main actionable gap is that aggregating multiple status entries can erase a forced-handoff obligation. Two smaller corrections concern the committee signing rule and a lemma's stated dependencies.

The original Regional live stream definition is preserved **byte for byte**: both v2 and v3 contain the same 658-byte definition, SHA-256 `fbd6cd16242a26303d653fcae9ab024cea85d69d8233c91b2c46bcdc27b61770`.

## Required corrections

### 1. Latch a forced obligation while processing entries individually

**Location:** boundary rule, lines 779–781; forced protocol F1, lines 845–849; F2's use of `beta_j`.

> “apply its ordered control prefix first. If the active region is off, use the forced path; otherwise ...”

> “If they leave the active region live, apply them passively and return.”

**Classification:** Justification Gap / executable-rule inconsistency. Other text already says a pending forced handoff does not revive old authority, but these instructions can be read as looking only at the final map after a whole prefix is applied.

Example: active A is marked off by entry j, then marked live by entry j+1 before a delayed verifier processes either. A receiver that processed j immediately records a pending retirement; a receiver processing both together can see A live and continue the original session. This contradicts the persistent-pending semantics and makes replay depend on prefix delivery grouping.

**Correction:** Process entries one at a time. At the first effective update leaving the current active region off, latch `pending=(force,j_star,beta_star,retired_session)`; later updates may change successor eligibility but cannot cancel this obligation, replace its cutoff, or revive that session. Force has priority whenever this pending obligation exists, even if the latest map now marks the old region live. F2 must refer to the recorded `beta_star`, not a variable `j` that may have advanced during waiting. F4/F5 clear it only after installing its one successor.

The regular-waiting path should use the same per-entry replay. “First prefix enabling completion” is then deterministic under the existing common-order premise.

### 2. Committee signing uniqueness must precede installation

**Location:** lines 657–660.

> “Correct committee members endorse only one installed successor of a given entry.”

**Classification:** Justification Gap in the immediately following threshold argument. A member could endorse two candidates before either is installed without literally violating “one installed successor.” The ordered L1 registry may accept only one, but that does not establish the stated uniqueness of certified candidates.

**Correction:** Say “Correct committee members sign at most one candidate successor for each predecessor entry and version, retaining that signing lock across restarts.” The existing >2/3 quorum intersection argument then follows under <1/3 faulty committee weight. Keep the separate coordination/liveness assumption.

### 3. Name the control-order premise in the normal-successor lemma

**Location:** lemma “A fixed anchor determines one normal successor,” lines 1158 onward.

The statement names regional safety and the authenticated snapshot rule, while the proof invokes `ass:recovery` for unique completion of a pending boundary.

**Classification:** Justification Gap in the lemma's assumption list; the cross-region theorem itself already includes the needed premise.

**Correction:** Add the admissible control-order premise to the lemma statement, or restrict the lemma to already sealed regular handoffs and state pending uniqueness separately. The first edit is shorter.

## Precision improvement

The recovery condition at lines 733–735 preserves every delivered prefix or prefix for which an accepted final receipt can already be assembled. The adjacent explanation instead says every already finalized batch is protected. If “finalized” includes an honest member's completed local commitment before enough CONFIRM endorsements exist, these are different conditions. Either strengthen the assumption to cover every already completed honest application commitment, or explicitly reserve the cross-region word “finalized” for the verifiable regional final-receipt frontier. The stronger cut premise is the simpler choice if the intended promise is that no honest regional commitment is ever abandoned. This is a wording/scope clarification, not a request to implement a new recovery protocol.

## Checked dependencies and positive findings

1. **Regional interface.** `Open` takes a finalized anchor and immutable authority; no speculative regional session or StreamFinal promotion stage remains. Global batch, mini-block and physical positions are separated. Passive entries do not mutate the session/signing domain.
2. **Drain.** Quota validation counts proposed ancestry, so pipelining cannot overshoot K application batches. Contentless controls obey the native timers and are counted by native proposal deadlines. A re-certified terminal anchor can acquire a fresh child timer; no new timer-free commitment rule is assumed. Arbitrarily many failed views are correctly excluded from a constant control-count claim.
3. **Regular rotation.** Terminal contents bind a single L1 snapshot, and normal handoff does not demand an extra committee update. Snapshot monotonicity is not mislabeled as prompt revocation. Waiting for an eligible different region is explicit.
4. **Force/regular race.** The admissible-control premise explicitly forbids an alternative force cut from competing with a sealed normal handoff. Later entries are evaluated against the certified successor. This is an assumption-backed composition claim, clearly identified as such.
5. **Forced cutoff.** Hidden finalized suffixes, fencing and recoverable bytes are explicitly separated from signature validity and L1 finality. The draft does not claim that a proof system proves nonexistence of hidden final receipts.
6. **Composition safety.** Given unique authority and anchor at each boundary, prefix induction is valid. The induction does not pretend to repair a broken regional safety contract.
7. **Composition progress.** Its premises require admissible cuts, eligible successors, recoverable data and sufficiently long serving intervals. Those exclude perpetual zero-output rotations and permanent complete-data-loss cases. No unconditional finite accident-recovery bound is claimed.
8. **L1 relation.** Historical source authority remains valid for old publication backlog. An inactive-region recovery does not justify an early rotation, and normal quota rotation needs no fresh committee entry.

## Validation limits

This review did not re-prove the inherited Sync HotStuff lemmas from the external paper, inspect rendered layout, benchmark costs, or instantiate the recovery/control-cut service. Its conclusions rely on the clearly stated regional and control-plane premises. The pending replay correction should be checked in the algorithm, derivation rules and all diagrams/captions that describe trigger behavior.

## Drain-retirement follow-up

It is safe, and simpler for communication accounting, to stop physical Drain/voting once a valid terminal receipt is verified rather than continuing until a successor is available. Require dissemination and retention of the receipt and complete terminal application data, and keep retrieval service available. The receipt includes an honest post-commit endorsement; every future valid physical descendant has empty application projection. The wrapper may therefore wait independently at that terminal prefix.

This termination does not mean every old replica must obtain a new native direct commitment through its own timer. The finite service has ended with a verified final application prefix. Accordingly, qualify the regional progress proof's “committed everywhere through a later descendant” sentence: it applies while the engine continues serving; after terminal retirement, receipt delivery suffices for application-output delivery. Do not introduce a timer-free native commitment rule merely to describe shutdown.

## Correction recheck

The revised manuscript now processes entries individually and latches `(j_star,beta_star,alpha_star)`; uses the stored cutoff in forced preservation, authentication and activation; locks committee signing before installation; lists the control-order premise in the normal-successor lemma; protects completed honest commitments in the cut premise; and retires the physical engine upon verified terminal receipt while retaining retrieval. The regional progress proof distinguishes post-retirement receipt delivery from native commitment. The regional definition remains byte-identical (658 bytes and the same hash above).

One final state-assignment omission remains at this recheck: forced F5 should explicitly clear `pending` and set `phase=active` atomically with replacing the session. Otherwise the newly specified persistent pending guard remains true after successful activation. Normal completion should likewise clear any shared pending-boundary metadata. With that local clarification, the reviewed corrections pass under the declared premises; no new mechanism is requested.
