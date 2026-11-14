# Consensus sources and consequences for the rotation outline

Reviewed 2026-09-25 against `formalization/outline.md`. This is a source and design-input note, not an audit of the eventual `main.tex`.

## Primary-source facts

### PBFT reference for the conservative completion

Castro and Liskov, *Practical Byzantine Fault Tolerance*, OSDI 1999, pp. 173–186: [official USENIX publication record](https://www.usenix.org/conference/osdi-99/practical-byzantine-fault-tolerance), [Section 4, The Algorithm](https://www.usenix.org/legacy/events/osdi99/full_papers/castro/castro_html/node4.html), [MIT-hosted PDF](https://pdos.csail.mit.edu/6.824/papers/castro-practicalbft.pdf).

Section 4.2 distinguishes pre-prepare, prepare, and commit; the latter stages protect ordering across views. Section 4.4 specifies view-change messages carrying a stable checkpoint and prepared proofs, a new-view quorum, and selection of the highest-view prepared value at each sequence number. Null requests fill unprepared gaps and still run through consensus. Recipients validate the new-view selection. Section 4.5 explains quorum intersection and timeout growth; Section 4.6 requires deterministic validation of externally supplied values. The original uses equal-weight replicas, so a stake-weighted adaptation must state its own weighted quorum thresholds and intersection argument.

### GeoBFT

Gupta, Rahnama, Hellings, and Sadoghi, *ResilientDB: Global Scale Resilient Blockchain Fabric*, PVLDB 13(6), 868–883 (2020): [paper](https://www.vldb.org/pvldb/vol13/p868-gupta.pdf), [DOI](https://doi.org/10.14778/3380750.3380757).

- Section 2.1, Remark 2.1: each cluster has `n > 3f`; the paper explicitly distinguishes this regional assumption from a global fault bound.
- Section 2.2, Figure 3: local replication uses PBFT proposal, prepare, and commit messages. The exported certificate contains `n-f` commit signatures.
- Section 2.3, Figures 5–7: the optimistic sender sends a certified request to `f+1` receivers in another cluster, which forward it locally. A receiver cluster can request replacement of the sender cluster's primary; requests include round/view information to prevent replay and duplicate changes.
- Section 2.4: every cluster contributes a request, and all replicas deterministically order the resulting set. There is no single globally active cluster in this baseline.
- Section 2.5: no-op requests undergo local replication and need commit certificates. Replication, inter-cluster sharing, and execution for different rounds can overlap.
- Safety does not assume bounded message delays; liveness needs periods of reliable bounded-delay communication (Sections 2.1–2.3).

The outline is therefore **GeoBFT-inspired**, rather than an instantiation of GeoBFT: it proposes one active region and a separate globally weighted rotation/finality layer.

### Ethereum validator assignments

Official specification, [Validator assignments](https://ethereum.github.io/consensus-specs/specs/phase0/validator/#validator-assignments), [Attesting](https://ethereum.github.io/consensus-specs/specs/phase0/validator/#attesting), and [Attestation data](https://ethereum.github.io/consensus-specs/specs/phase0/validator/#attestation-data): a validator has an assigned committee and slot in each epoch and creates its attestation at that assigned slot. The attestation separately identifies its head vote and FFG source/target checkpoints. Therefore, the outline should not say Ethereum requires every validator to vote in every slot.

The current specifications retain committee-aware aggregation: [Electra Attestation](https://ethereum.github.io/consensus-specs/specs/electra/beacon-chain/#attestation), [get_attesting_indices](https://ethereum.github.io/consensus-specs/specs/electra/beacon-chain/#modified-get_attesting_indices), and [EIP-7549](https://eips.ethereum.org/EIPS/eip-7549) document the committee bitfield and aggregation across committees. This note uses inherited mechanism definitions, not a claim that Phase 0 is the currently deployed fork.

### Ethereum checkpoint identity and slashing

Official [Checkpoint](https://ethereum.github.io/consensus-specs/specs/phase0/beacon-chain/#checkpoint) contains both `epoch` and `root`. [process_slot](https://ethereum.github.io/consensus-specs/specs/phase0/beacon-chain/#slot-processing) caches the latest block header root even during skipped slots. [is_slashable_attestation_data](https://ethereum.github.io/consensus-specs/specs/phase0/beacon-chain/#is_slashable_attestation_data) identifies distinct votes with the same target epoch, or strict source/target interval nesting, as slashable. Epoch finality accounting uses active effective balances and explicit justification/finalization state transitions in [Justification and finalization](https://ethereum.github.io/consensus-specs/specs/phase0/beacon-chain/#justification-and-finalization).

Consequently, a same-root checkpoint in a later epoch is not the same checkpoint object. An actual self-loop with unchanged epoch cannot substitute for an ordinary forward finality link.

### Gasper and Casper FFG

Buterin et al., *Combining GHOST and Casper*, [arXiv:2003.03052v3](https://arxiv.org/pdf/2003.03052v3): Sections 4.1 and 4.4 explicitly use block/epoch pairs; Figure 5 gives a case in which one block occupies two epoch-boundary pairs. Definitions 4.5–4.9 distinguish a supermajority link, justification, and finalization. Section 4.6 warns that naively combining a latest-justified anchor with GHOST can induce accidental surround violations; its filtered hybrid fork choice resolves the stated problem. Sections 5–7 give distinct safety, plausible-liveness, and probabilistic-liveness results under their respective assumptions. This is an idealized protocol paper, not a complete description of current deployed Ethereum.

Buterin and Griffith, *Casper the Friendly Finality Gadget*, [arXiv:1710.09437v4](https://arxiv.org/pdf/1710.09437v4): Section 2 defines stake-weighted supermajority links, justification, adjacent-child finalization, double voting, and surround voting. Theorem 1 is accountable safety; Theorem 2 is plausible liveness. Section 1.1 explicitly explains that finality liveness depends on the proposal mechanism. A finality gadget alone does not make an unavailable sequencer produce blocks or transmit missing data.

Official [Fork choice](https://ethereum.github.io/consensus-specs/specs/phase0/fork-choice/#get_head): GHOST starts from the justified checkpoint and traverses a filtered viable tree using weighted latest messages and a deterministic tie-break. A head choice is not a commit certificate. The specification also contains additional operational rules beyond this short description.

## Design consequences and proposed repairs

The following are deductions or proposed choices, not claims about the source protocols.

1. **Use intra-region for local communication.** Use inter-region or cross-region for communication between regions. The outline currently uses “inter-region” for both concepts.

2. **State two fault conditions.** Let regional stake be `W_i` and faulty regional stake be `B_i`. A local quorum must have weight `>2W_i/3`, and locally certified safety needs `B_i<W_i/3`. A global bound `sum B_i < (sum W_i)/3` does not imply this. A small region can be entirely Byzantine while the global bound holds. Either require regional eligibility constraints or explicitly weaken local certificates to provisional statements that global voters independently validate.

3. **Quorum signatures do not automatically give reliable-broadcast totality or cross-view safety.** A proposal and one voting round can establish same-height certificate uniqueness when honest voters sign at most once for that domain. They do not, by themselves, specify certificate dissemination, recovery after split votes, or locking across views. A parent selected as “latest majority-voted” must come with a precisely ranked certificate, a new-view quorum, and a safe-value rule. Use a defined BFT primitive or write these transitions explicitly.

4. **Do not derive a committed empty tip from a local timeout.** Missing a proposal is a local observation. A timeout message can initiate a consensus-approved no-op, skip, or view change; a local no-op vote cannot itself advance the committed tip. Distinguish a valid zero-transaction miniblock from a certified leader-timeout record. Otherwise, an idle workload becomes indistinguishable from leader failure.

5. **Separate checkpoint time from payload progress.** A possible notation is `C_e=(e,H_e,A_e)`, where `H_e` commits to a control-history node and `A_e` is its adopted miniblock tip. Consecutive failure records can have `A_e=A_{e-1}` while `H_e!=H_{e-1}` and epochs advance. The failure label and rotation decision must be signed fields. A raw equality of data roots is insufficient evidence of an unhealthy region, since healthy idle operation may produce the same application state.

6. **Keep the FFG source justified.** Replace `C_{e-1}->C_{e-1}` by a forward target at epoch `e` carrying the unchanged accepted data tip and an explicit failure status. The source must be the applicable justified checkpoint, which may not be epoch `e-1` after missed quorums. A target failure record is justified by its link; it becomes finalized under the chosen adjacent-epoch rule only after the required next link. Two failure epochs need an exact counter/state transition and reset policy.

7. **Subjective satisfaction is not deterministic validity.** If honest global voters independently select “satisfied” or “unsatisfied” without a common decision rule, neither alternative need gather a quorum. Define a deterministic baseline health predicate and a consensus mechanism for selecting the control record. A later incentive module may choose parameters or submit authenticated observations without silently changing vote validity. A minority refusal to support progress cannot be treated as a supermajority failure certificate.

8. **A disconnected region cannot certify its own exit.** If its stake is `W_i`, then a protocol requiring a global quorum must assume the remaining connected honest stake is `>2W/3` during emergency rotation. Global Byzantine stake `<W/3` alone does not ensure this when honest stake is also disconnected. If failover depends on a sealed checkpoint from the old region, complete disconnection can block failover regardless of the remaining stake. State this limit or introduce externally enforceable leases and a data-recovery rule with a separate proof.

9. **Preserve or explicitly qualify locally certified progress during handoff.** If the next region starts at the last global checkpoint, locally committed miniblocks after that checkpoint can be lost. Calling them final conflicts with no-rollback goals. To preserve them, the old region must seal a certified cut and transfer sufficient data and certificates, or some other protocol must ensure a uniquely recoverable tip. Availability and safety obligations remain when the old region is unreachable.

10. **Define stake/configuration snapshots and signing domains.** Include protocol identifier, configuration version, region, height/slot, view, phase, and payload hash as appropriate. Fix stake weights for each certificate domain. A signature counted under changing stake totals can invalidate quorum-intersection reasoning. Membership changes need an authenticated handoff, not just a fresh scheduling-function evaluation.

11. **Do not import surround voting into ordinary local votes without a definition.** Local equivocation rules naturally apply to conflicting votes in the same height/view/phase; whether votes across views are slashable depends on the recovery algorithm. FFG surround conditions instead compare source and target epochs. Slashing a validator for two different values at one height across different views can punish a recovery step that the protocol explicitly permits.

12. **Keep proposer and availability assumptions visible.** Sequencer rotation cannot force a Byzantine sole proposer to batch, upload data, or submit L1 commitments. Settlement liveness needs an honest eventual proposer, a replacement rule, or a permissionless fallback. A hash commitment and signatures do not imply data availability or state-transition correctness unless the model supplies a corresponding validation/retrieval assumption.
