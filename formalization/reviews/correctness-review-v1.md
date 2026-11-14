# Independent correctness review v1

Reviewed on 2026-09-25 in verifier-only mode. No changes to `main.tex` were made by this verifier.

**Reviewed artifact:** `formalization/main.tex`, SHA-256 `d936d5e81a8febd276f891ea3c4bfee579397c9fe3a8669b8d2cf50c1c1f6b32` (962 lines). Line references below refer to that snapshot; subsequent author edits require a separate review. The review also read `outline.md` and the source/advisory notes in this directory. The user's confirmed role constraint is authoritative: S must order inputs, may execute privately, and need not execute; P executes.

## Summary

**Final Verdict:** The A2/A3 order-safety, fencing, and conditional faithful-execution arguments are valid as compositions of the stated abstract contracts, but the full correctness package is invalid as written because the publication-liveness inference has a **Critical Error**; local liveness additionally has a **Justification Gap** concerning the application timer and the sufficiently-long-view hypothesis.

This verdict does **not** identify an A2/A3 fork counterexample under the stated BFT/external-validity assumptions. It distinguishes a counterexample to the claimed sufficiency of the publication assumptions from a missing premise in the local-liveness argument.

### List of Findings

1. **Location:** lines 265–268, “The adapter deterministically fixes native block boundaries”; lines 662–671, job endpoints `[k_0+1,k_1]` and “Jobs align with completed native EVM blocks”; line 854, “The proposer and external-service assumptions complete publication.”
   **Issue — Critical Error (publication liveness):** Determinism does not imply that a completed native-block boundary ever coincides with an order-batch endpoint. The job representation only permits whole-batch endpoints. An allowed deterministic adapter can therefore produce infinitely many completed native blocks while no nonempty range is a legal publication job, even with an honest available P, signing service, DA/L1 services, and an indefinitely advancing A4 frontier. The publication inference is false for the stated adapter class.

2. **Location:** lines 333–339, BFT liveness requires “a sufficiently long view”; lines 414–419, a progress timer resets only on DATA and `T_fail` requests a view change; lines 837–842, the liveness proposition lists synchrony, responsive quorums, eventual honest leaders, and common eligibility.
   **Issue — Justification Gap (local liveness):** The application has not shown that these premises provide the sufficiently long honest view required by the imported BFT contract. In particular, the relationship between its DATA-progress timer and PBFT's increasing view timers is unspecified. An application timer that can repeatedly interrupt the BFT pacemaker before commitment defeats the cited liveness argument. The conclusion may hold with the intended pacemaker integration, but that integration is not yet stated.

### Conditional validity scope

The following are verified for fixed configuration, assuming the named BFT module actually meets its stated contract and all application validity predicates are enforced:

- A2 histories are prefix-compatible within an authority generation, including across local terms and views.
- A3 histories are prefix-compatible across geographic generations. ROTATE inherits every globally decided anchor, including those awaiting FFG finalization.
- A4 and A5 order safety inherit A3 safety; FFG is not needed to prove uniqueness of global order in this construction.
- Same-input-root HOLD checkpoints are distinct control checkpoints, so forward links can finalize a HOLD without new transaction progress.
- With the newly explicit monotone target/source signing state, honest FFG voting does not generate a double or surround vote on the unique control chain.
- Availability certificates contain more than one-third honest global acknowledgment weight. Actual future retrieval remains conditional on the retention/reachability assumptions.
- Correct proposers compute identical outputs given identical committed inputs, prior state, and a deterministic adapter; exclusion of incorrect accepted execution additionally depends on the assumed sound settlement verifier.
- No proof is supplied or implied for a deployed MegaETH adapter, weighted PBFT implementation, dynamic membership, arbitrary regional failures, subjective-health convergence, transaction inclusion, or incentive equilibrium.

## Detailed Verification Log

### 1. Role split, source boundary, and outline fidelity

> “Sequencers may execute privately to improve ordering, but execution is not a sequencing obligation.” (lines 61–63)

This matches the confirmed user constraint. The document introduces an order-batch envelope and assigns native execution to P; it does not incorrectly require an order-only S to produce receipt roots. The adapter fixes admission, rejection traces, execution, and context. Therefore an ordered candidate list is distinct from its admitted transaction subsequence, and G3 is faithfulness to the candidate-input adapter rather than a promise that every candidate executes.

> “Global PBFT serializes checkpoint and authority decisions before FFG votes.” (lines 113–118)

The document accurately presents a conservative replacement of the outline's incomplete two-round/Gasper sketch. It preserves the hierarchical service/recovery objective but does not claim to have proved the originally sketched modified Gasper. LMD is explicitly speculative. This change is material and disclosed, rather than silently importing Gasper safety for a different voting population.

### 2. Fault, availability, and execution assumptions

> “Byzantine stake is `B<W/3` globally and `B_i<W_i/3` inside each region whose certificates are accepted.” (lines 164–170)

Correctly separated. A global bound cannot justify local certificates from an arbitrarily small concentrated-fault region. The regional bound also applies to accepted readiness certificates. The historical-key qualification is necessary for the persistent certificate model; dynamic or mobile corruption outside that qualification is not proved.

> “A live committee operation needs responsive honest weight `>2/3`.” (line 183)

This is stronger than the fault-only safety condition and correctly captures Byzantine withholding plus honest unavailability. Since `U` is explicitly unavailable **honest** stake, `B+U<W/3` does not double-count Byzantine nodes. Losing a region of one-third total stake can prevent global recovery; the document states that limitation.

> “Honest acknowledgment signers retain complete data until verified archival replacement and the required recovery/evidence windows.” (lines 192–199)

Retention and future reachability are explicit assumptions, not consequences of signatures. The data proposition is valid only with those assumptions. Archival replacement is not independently proved sound here. The document properly avoids claiming that a hash reconstructs arbitrary lost data.

> “The reference permits only finalized, monotonically nondecreasing L1 origins...” (lines 240–243)

This removes the immediate conflict between immutable A3 input and a subsequently reorganized tentative L1-origin context. The correctness claim remains conditional on the external meaning of finalized L1 history. It does not establish compatibility with a specific deployed execution adapter.

### 3. PBFT application and local recovery

> “This is a named consensus dependency, not a new proof of PBFT.” (line 339)

This is an admissible conditional abstraction. The weight intersection lemma is correct: two weights strictly above `2W_Q/3` intersect in weight strictly above `W_Q/3`, hence in honest weight under the local committee fault bound. It is not itself a proof of a complete stake-weighted PBFT implementation.

The source PBFT algorithm expressly carries prepared evidence through view changes and fills unprepared gaps with internal null operations. Its liveness argument also uses increasing timers, so importing its safety does not automatically establish the application's separate DATA-progress timer is live. See [PBFT, Sections 4.4–4.5](https://www.usenix.org/legacy/events/osdi99/full_papers/castro/castro_html/node4.html).

> “Application controls and PBFT null operations advance decision height `h` without advancing input position `k`...” (lines 364–366)

Verified. This makes PBFT null semantics compatible with the order-batch state machine. A null is not required to masquerade as a sequencer-signed GAP, and it does not create an unexplained input-hash transition.

> “For a new-view reproposal, check the original envelope's historical authority and the safe-value proof...” (lines 391–394)

Verified. An unchanged prepared operation can survive a view change even though the new primary differs from its original signer. The external-validity rule no longer forces the successor to alter the signed envelope/hash or reject a value it must preserve. Fresh operations still need current authority.

The one-undecided-height restriction, contiguous application, parent commitments, and certified new-view dependency suffice for the stated local prefix proof. A certificate hidden from the successor does not invalidate the argument: honest commit signers retain prepared evidence that must intersect the new-view quorum. A bare successor tip would not suffice, but the document expressly rejects that shortcut.

> “Each member resets its progress timer only on a newly decided leader-produced DATA batch.” (lines 414–415)

Empty DATA correctly counts as service; a quorum-signed timeout does not itself advance input. However, **Finding 2** applies. The theorem's hypotheses do not explicitly entail a sufficiently long honest view once this independent progress timer is introduced. As a diagnostic execution, let all nodes be honest and message delays remain within a fixed bound, while `T_fail` is shorter than the time needed to complete an honest view. If the application can retrigger view change whenever no DATA has committed, honest views can be interrupted indefinitely. Even if its timer restarts on every new view, a fixed insufficient timeout can continue to preempt commitment. This is not a counterexample to PBFT with its proper growing pacemaker; it demonstrates the missing composition premise.

After temporarily assuming that missing premise, the local liveness proof correctly applies the BFT contract to a common valid operation in a sufficiently long honest view.

### 4. Global data acknowledgments and control records

> “Adopt its input tip. P need not yet have executed it.” (line 493)

Correct. Global order/availability is not execution validity. The acknowledgment now binds its base and endpoint, and the certified complete range includes regional certificates and context. Honest acknowledgment weight is strictly greater than `W/3`, by subtracting at most Byzantine weight from a quorum above `2W/3`.

ADVANCE extends the immediately preceding A3 anchor, HOLD preserves it, and ROTATE preserves it while changing authority. Global BFT applies these records in sequence. Therefore induction establishes one monotone A3 input history. Health reports cannot authorize a conflicting prefix. Their convergence remains an explicit liveness assumption; the grace-period baseline alone is not presented here as a theorem for every adversarial arrival schedule.

### 5. FFG source, repeated roots, and offenses

> “where `C_u` is its highest known justified ancestor.” (line 532)

> “Persist the greatest signed target epoch and the greatest justified source; process targets in increasing order and never lower the source.” (lines 539–540)

Together these rules prevent honest surround offenses on the single globally decided chain: if the later signed vote has the larger target, its source cannot be smaller; thus it cannot strictly surround an earlier vote. Increasing targets and one vote per target prevent double votes. Finality remains a delayed eligibility condition on the BFT-ordered chain, not a substitute for BFT order safety. This separation is consistent with the source/target and adjacent-finalization distinctions in [Casper FFG, Section 2](https://arxiv.org/pdf/1710.09437).

An earlier version seen during this review lacked the monotonic signing-state sentence. It permitted the trace `0 -> 3`, followed by delayed `1 -> 2` after learning that checkpoint 1 was justified, which satisfies `0<1<2<3`. The reviewed hash closes this issue; it is not an outstanding finding.

> “Checkpoint identities and epochs differ although input tips agree.” (line 553)

Correct. Distinct `g_e` values commit to distinct control records even when `q_k` is unchanged. Two consecutive successful links finalize the intervening HOLD checkpoint. Merely having two HOLD records does not create those links; the text separately requires the FFG finalization guard and acknowledges missed votes.

FFG progress is conditional on timely certificate dissemination and sufficiently successful consecutive voting rounds. Safety does not require those liveness conditions. No full Ethereum/Gasper liveness result is proved or needed for the A3 order argument.

### 6. Geographic activation, fencing, and preservation

> “is the latest globally decided input anchor, including any checkpoint awaiting FFG finalization.” (lines 588–589)

This is the correct handoff boundary for this construction. Choosing only the latest FFG-finalized anchor would permit forgetting an already ordered A3 extension; the document expressly preserves it instead. Ready signers acknowledge the full history, authority chain, and outstanding jobs, rather than a root only.

> “Activation takes effect for A3 extension when ROTATE is globally decided.” (line 598)

The unique global control order serializes competing rotation/advance proposals. A valid ROTATE fixes the inherited cutoff and successor generation. Fresh old-generation extensions after the cutoff cannot become A3. Isolated clients can still observe old A2 output, and the text correctly does not claim instantaneous remote fencing.

> “The regional SEAL operation records its last input tip and prohibits further DATA/GAP decisions in that generation.” (lines 575–577)

This gives the lossless planned transition a real frozen tail. A planned geographic change must export that tail before activating the successor; emergency recovery instead permits losing only the regional-only suffix. Native block sealing is kept distinct.

The durability-tradeoff proposition is valid for arbitrary unknown payload bytes with no surviving external copy. Signatures/digests can authenticate a surviving candidate but do not provide a bounded reconstruction algorithm for arbitrary vanished payloads. Its conclusion is appropriately limited to the stated failure scenario.

### 7. P, native boundaries, and settlement

> “Jobs align with completed native EVM blocks.” (line 669)

This avoids publishing a partial native block, but **Finding 1** prevents the claimed publication inference. Consider an otherwise healthy execution with a fixed deterministic adapter that closes each native block after two admitted transactions. Let the first order batch contain three admitted transactions, and every later order batch contain two. All transactions are valid. The cumulative admitted lengths at order endpoints are `3,5,7,9,...`, whereas complete native-block endpoints are `2,4,6,8,...`. Thus every order-batch endpoint lies inside a native block. Infinitely many native blocks finish inside batches, all input becomes A4, and every external service is available, yet no positive whole-batch endpoint is a native-block endpoint. No job beginning at genesis can satisfy the stated range/boundary constraints. The permitted deterministic adapter class therefore does not imply publication liveness.

This countermodel does not show that accepted output can be incorrect; it shows that no eligible nonempty publication job need exist. Publication-liveness conclusions depending on the quoted inference are unverified/false for this adapter class; the independent order and execution-safety arguments remain checked above.

> “Ordering-S rotation does not switch this key.” (line 677)

The document explicitly separates ordering authority from the native EVM signing key and assumes native signing availability. It does not falsely claim an independent new signer can finish a block begun under another key. Recovery of a failed key service remains outside the proved integration.

The exact input-range, continuity, authority, A4, DA, and execution-verifier checks support G3 conditionally. Reassignment preserves the same input obligation, and the original P's refusal to acknowledge does not cancel a publicly activated job. Deadline satisfaction still depends on configured external confirmation/proof behavior; the manuscript does not establish numerical deadline adequacy.

### 8. Accountability, proofs, and scope

Local PBFT evidence is phase/view-specific. A different operation at the same height in a later permitted view is not automatically an offense. FFG evidence instead compares source/target epochs and full checkpoint identities. Timeouts and subjective reports authorize recovery without automatically establishing misconduct. These distinctions are sound for the stated interface-level G6.

The weighted intersection lemma is valid. The order/handoff proposition is valid under its explicitly assumed BFT and external-validity contracts. The data/faithfulness proposition is valid under its stated retrieval and soundness assumptions. The conditional-liveness proof has exactly the dependencies identified in Findings 1–2; it is not fully established by the current premises.

The latency inequality is symbolic and conditional on bounds for every term. It correctly does not infer multi-phase BFT latency from one observed network delay. Its applicability also depends on the long-view condition in Finding 2. The geographic example preserves A3 and clearly cancels the A2-only tail. Fixed-configuration analysis correctly leaves dynamic membership and long-range protection outside the result.
