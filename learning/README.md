# Learning guide: designing MegaETH sequencer rotation

[Chinese study package: Restaking services, EigenDA workflows, consensus, and rotation (2026-09-22)](zh-CN/restaking-services/README.md) · [Source index](zh-CN/restaking-services/sources.md)

[中文版 / Chinese version](zh-CN/README.md)

[Follow-up discussion in Chinese: receipts, transaction roots, MegaETH mini-blocks, Flashblocks, and OP Stack Subblocks](zh-CN/05_miniblocks_and_flashblocks.md)

[Chinese learning note: sequencer selection, fees, and tokenomics — OP Mainnet, Base, and MegaETH (2026-09-21)](zh-CN/07_sequencer_selection_fees_and_tokenomics.md)

[Chinese learning note 11: OP Stack vs. MegaETH — architecture, realtime confirmations, and high availability (2026-09-22)](zh-CN/11_op_stack_vs_megaeth_architecture.md)

[Chinese learning note 12: regular leader rotation in Flow, Aptos, and MonadBFT (2026-10-09)](zh-CN/12_hotstuff_regular_leader_rotation.md) · [Pinned implementation sources](sources/2026-10-09-hotstuff-rotation/README.md)

[Chinese learning note 13: Ethereum and restaking entry, exit, churn limits, and epoch-based regional rotation (2026-10-10)](zh-CN/13_staking_membership_and_randomized_rotation.md)

[Chinese learning note 14: Espresso fast confirmation in MegaETH specification PR #8 — records, verification, settlement enforcement, and failure conditions (2026-10-17)](zh-CN/14_espresso_fast_confirmation.md)

[Chinese learning note 15: MegaETH and OP Stack L1 force inclusion, withdrawal escape hatches, and Espresso outages (2026-10-24)](zh-CN/15_l1_force_inclusion_and_espresso_outage.md)

[Chinese learning note 16: Simplex and regional BFT selection under one-third Byzantine tolerance (2026-10-28)](zh-CN/16_simplex_and_regional_bft.md) · [Illustrated reading edition](zh-CN/16_simplex_and_regional_bft.html)

These notes answer [the September 16 meeting questions](../meetup_record/260916.md) and develop the context needed for [the proposed rotation architecture](../meetup_record/rotation_overview.jpg).
They explain the existing pipeline first, then its security requirements, a possible formal model, and relevant systems and incentive designs.
They are research and learning notes, not changes to the MegaETH specification.

**Research date: September 16, 2026.**
The local specification snapshot is commit `59797b45b1f59b7c2c147e6f99be39399a1f2b44` in `megaeth_specs`, dated September 14, 2026.
Public documentation and primary literature were consulted alongside the user's Zotero library.
Sources are linked next to the claims they support; the Zotero search record appears in [note 02](02_requirements_and_failure_modes.md).

## Read in this order

| Note | What you should understand afterward |
| --- | --- |
| [01 — MegaETH's pipeline](01_megaeth_pipeline.md) | The components, EVM-block and mini-block structures, production rules, transaction lifecycle, DA publication, L1 derivation, and settlement; what changes when ordering is separated from execution |
| [02 — Requirements and failure modes](02_requirements_and_failure_modes.md) | What validity, safety, liveness, censorship resistance, and accountability each require; how an orderer, executor, publisher, or proposer can prevent the system from meeting them |
| [03 — Formal model and handoff](03_formal_model_and_handoff.md) | How to describe local information, actions, signed orders, execution acknowledgments, authority, inherited prefixes, and failover without hiding unresolved decisions |
| [04 — Related systems and incentives](04_related_systems_and_incentives.md) | What existing projects and research proposals actually decentralize, what their proofs enforce, how their incentives work, and which ideas transfer to this design |

For the five pipeline topics requested in the follow-up, start with note 01:

1. **Building blocks:** distinguish ordering, execution, state streaming, validation, batch publication, state proposals, and proof/dispute resolution.
2. **Block structures:** distinguish an EVM block, its mini-blocks, their signatures and roots, a publication batch, and a DA certificate.
3. **Block-building rules:** identify mandatory transactions, authority checks, state-dependent transaction validity, resource constraints, and the producer's remaining discretion.
4. **Full lifecycle:** follow one transaction from RPC submission through execution and early results to reconstructable history and settlement.
5. **L1 confirmation and DA:** distinguish availability, an L1-recorded input commitment, derivation finality, and an accepted execution claim.

## Direct answers to the meeting questions

| Meeting question | Working answer | Where to develop it |
| --- | --- | --- |
| What does the current proposer do, and what is the full pipeline? | Separate block production from state proposal: the former supplies an ordered executed history; the latter makes a settlement claim about its result. | Note 01 |
| What changes with rotation? | A new operator needs both valid authority and an agreed history to continue; the sketch additionally introduces a separately accountable ordering/execution interface. | Notes 01 and 03 |
| What is the sequencer's role? | The current written protocol exposes a producer of execution-bearing outputs; the sketch assigns the new sequencer only transaction selection, order, and constrained L1-origin choice. | Note 01 |
| What are the liveness and validity requirements? | Define separate obligations for order agreement, executable inputs, correct state transitions, data recovery, timely service, inclusion, and settlement. | Notes 02 and 03 |
| What misbehavior can prevent these goals? | Consider each dependency and each handoff step, including withholding, conflicting commitments, censorship, invalid candidates, incorrect execution, and publication or settlement stalls. | Note 02 |
| How should each sequencer's behavior be formalized? | Use a history-dependent strategy based on the messages and state it actually observes; distinguish Byzantine actions from rational choices and measurable service failures. | Note 03 |
| How do related systems implement decentralization? | They make different choices about leader selection, consensus, order commitments, execution, DA, and forced inclusion; compare these separately. | Note 04 |
| Do optimistic or validity proofs make participants follow the rules? | They enforce the statement encoded in the proof/dispute system; timely inclusion, promised ordering, availability, and leader replacement need their own mechanisms. | Notes 02 and 04 |
| How should sequencers be incentivized? | First define attributable duties and verifiable violations, then allocate fees, operating costs, bonds, rewards, and penalties across the actual roles. | Notes 03 and 04 |

## The most consequential distinctions

**The sketch is a protocol change, not simply an operator schedule.**
The existing [mini-block signature](../megaeth_specs/specs/protocol/blocks.md#mini-block-header-signing) covers transaction and receipt commitments and gas usage.
A new signature issued before execution needs its own meaning, and an executor's results need a verifiable relationship to it.

**Rotation needs an inherited prefix.**
Knowing who goes next does not establish which previously signed transactions survive the handoff.
The successor needs their bytes and authenticated execution progress, plus any publication duties assigned to it.
If execution also changes operators, the replacement executor additionally needs usable execution state or a way to reconstruct it.
The transition model in note 03 is a proposal for organizing these obligations, not a claim that they are already implemented.

**An incomplete EVM block deserves its own failure case.**
The current [authority rule](../megaeth_specs/specs/protocol/sequencing.md#sequencer-authority) keeps the signing key constant throughout an EVM block.
Replacing an unavailable signer halfway through that block cannot be justified by a rule that only changes keys at block boundaries.

**Authentication, correctness, availability, and finality answer different questions.**
The [state-sync trust model](../megaeth_specs/specs/protocol/state-sync.md#trust-model) explicitly distinguishes authenticated sequencer outputs from independently verified execution.
The learning notes maintain that distinction when discussing preconfirmations, DA, and settlement.

## Evidence and inference policy

| Evidence type | How to interpret it |
| --- | --- |
| Local normative text | Evidence of the behavior specified by this checkout; not by itself evidence that every deployment matches it |
| Official public documentation | A first-party description, with deployment parameters and configuration still subject to verification |
| OP Stack baseline | A concrete reference for inherited architecture or a comparison, subject to MegaETH's changes |
| Academic paper | A result or construction under that paper's stated assumptions; no automatic guarantee for the proposed integration |
| Research/governance forum | An attributed proposal or discussion, not proof that a feature shipped |
| Proposed model or inference | Our reasoning about a possible design; explicitly distinguished from current MegaETH behavior |

The permission to use OP Stack as a reference is useful because MegaETH's local [rollup section](../megaeth_specs/specs/rollup/overview.md) is under construction.
It does not justify importing exact OP constants, contract layouts, permissions, or timing guarantees without checking the MegaETH delta.
Mini-blocks are explained using the MegaETH specification; they should not be assumed to be a standard OP Stack block type.

In particular, this source review does not establish deployed registry governance, exact DA failure handling, the batcher validation gate, all proof-system parameters, or a complete interrupted-block recovery algorithm.
Those uncertainties are part of the research agenda, rather than filled with unstated assumptions.

## Decisions to resolve next

The next design discussion can focus on a small set of choices:

1. **What rotates?** Only ordering, or execution and its signing authority as well?
2. **What does an early promise mean?** Receipt, inclusion, exact order, successful execution, or a specified result?
3. **What becomes committed, and by whom?** Leader assignment, transaction prefixes, execution results, or several of these at different times?
4. **What survives a crash?** Which prefix is retained, where its data is stored, and who can finish an interrupted EVM block?
5. **How does an order-only service handle invalid inputs?** Validation before a binding promise, or a deterministic candidate-to-execution mapping?
6. **How is recovery authorized if L2 stops?** A transition that itself requires the stalled leader to produce another block may not be a usable emergency path.
7. **Which failures should trigger replacement, and which justify penalties?** A timeout may justify recovery without proving malicious behavior.
8. **What is the performance target at each stage?** Measure order acceptance, execution response, recovery, publication, and settlement separately.

Once these choices are explicit, the formal model can become a protocol proposal with concrete safety and liveness claims, and the incentive model can address enforceable duties rather than an unspecified idea of “honest sequencing.”
