# MegaETH source and compatibility review

Reviewed on 2026-09-25 for `formalization/main.tex`; this report does not modify the MegaETH specification.
Labels: **Local specification** means the checked-out normative snapshot; **Public documentation** means an official page retrieved today, not independent deployment verification; **Project requirement** means the user's design material; **Design implication** is our inference or proposed interface.

## Source boundary

- **Local specification:** `megaeth_specs` HEAD is `59797b45b1f59b7c2c147e6f99be39399a1f2b44`, with a clean tracked working tree and remote `https://github.com/megaeth-labs/specification.git`.
  This review did not fetch or alter that checkout.
- The local repository's [AGENTS.md](../../megaeth_specs/AGENTS.md) marks the specification repository as separately restricted; it must not be copied into the public documentation repository.
  The formalization is a local research artifact, not an edit to the normative specification.
- Local `SequencerRegistry`, stateless validation, derivation, DA, batching and fault-proof pages are placeholders/stubs.
  They are not evidence that the live system lacks those functions.
  The current public registry page is substantially newer than the local placeholder.

## Established constraints and consequences

| Topic | Evidence and epistemic status | Consequence for this design |
| --- | --- | --- |
| Native mini-block | **Local specification:** [Blocks and Mini-Blocks](../../megaeth_specs/specs/protocol/blocks.md) defines eight signed header fields: containing block number/time, local index, global mini-block number/time, gas used, transaction root, receipt root. There is no predecessor hash or state root in that header. | A hash-chained consensus envelope and mini-block state commitment, if used, are new objects; do not claim they are existing header fields. |
| Nesting and order | **Local specification:** every EVM block has at least one mini-block; a mini-block never spans EVM blocks; local indices reset to zero; global numbers increase by one. The EVM transaction list concatenates mini-block lists in index order. | Bind consensus position to both block number/index and the chain-wide stream position. Preserve all list bytes and block-boundary context in handoff. |
| Commitments | **Local specification:** transaction/receipt roots are ordered MPT roots over their respective EIP-2718 lists. Whole-block roots are recomputed over concatenated lists, not obtained by hashing mini-block roots together. Receipts carry block-cumulative gas. | Proposer verification must check exact list correspondence and recompute native roots. A stream-prefix hash is an additional research commitment. |
| Signatures | **Local specification:** native mini-block signing uses recoverable secp256k1 over `keccak256(rlp(header))`; [State Sync](../../megaeth_specs/specs/protocol/state-sync.md) separately authenticates state/transaction/receipt fragments and signed EVM headers through BLAKE3-based payload commitments. | Do not conflate a consensus vote, a mini-block signature, a fragment signature, or an execution proof. Specify what each certifies. |
| Key authority | **Local specification:** [Sequencing](../../megaeth_specs/specs/protocol/sequencing.md#sequencer-authority) selects `sequencerAt(block_number)` once before the first mini-block. All mini-blocks in one EVM block must use that same key. | Switching an independent sequencer key in the middle of an EVM block is incompatible with the snapshot. Planned rotation at complete-block boundaries is straightforward; interrupted-block recovery needs an explicit extension. |
| Mandatory input | **Local specification:** supplied deposit and L1-attributes transactions appear before ordinary transactions in the first mini-block. Ordinary transaction ordering is otherwise a producer policy. | Freeze the L1 origin, mandatory prefix and execution context. Weak liveness should allow different eligible packing policies and must not imply fairness or per-transaction inclusion. |
| State sync | **Local specification:** recipients authenticate the producer, enforce fragment order/continuity, apply SALT deltas once, and compare completed-block roots; the trust-model section explicitly says these checks do not establish execution correctness. | A committee that only consumes authenticated deltas cannot certify valid execution. Honest voting must include execution verification or explicitly attest only to availability/order. |
| Native execution boundary | **Local specification:** no externally specified sequencer CL/EL split. The sequencer is opaque for protocol purposes. | The proposed S/P split is a research interface. It cannot inherit existing interoperability or safety guarantees merely by reusing role names. |

**Public documentation:** the current [Mini-Blocks page](https://docs.megaeth.com/mini-block) confirms roughly 10 ms mini-blocks inside roughly 1 s EVM blocks, signed commitments, and EVM-boundary key rotation.
It distinguishes sequencer preconfirmation from L1 finality.
These cadence values are descriptive, not a local-spec enforced slot duration.

**Public documentation:** [Architecture](https://docs.megaeth.com/architecture) describes the native sequencer as ordering, executing, sealing, and streaming blocks; EigenDA receives block data and a batcher posts its certificate to Ethereum.
It describes Kailua dispute resolution and an HA standby, but supplies no cross-operator Byzantine quorum protocol or per-mini-block replication barrier.
The design must state its own consensus and recovery assumptions rather than infer them from HA.

**Public documentation:** the [SequencerRegistry](https://docs.megaeth.com/spec/system-contracts/sequencer-registry) separately tracks the system-transaction address and mini-block signing key.
Rex6 registry version 2 adds a new-key possession proof and minimum scheduling delay; changes apply before a block.
Scheduling is admin-controlled, not a documented stake-weighted rotation election.
The page explicitly identifies deadlock if production stops after activating an unavailable key.
An emergency L1-authorized replacement path and decentralized authority adapter are therefore new integration work, not functionality supplied by this registry interface.

**Public documentation:** [Validator Architecture](https://docs.megaeth.com/node-operation/stateless-validation/validator-architecture) verifies a witness against the parent state root, re-executes transactions and system hooks, and compares post-state, withdrawal, receipt, bloom and gas commitments.
Missing/unwitnessed data cannot silently become empty state.
Witness requests must be pinned to block number and hash.
The validator does not select the canonical fork; execution validity and canonicality are separate properties.

## Outline conflicts that the formalization must resolve

1. **Order-only S versus native executed mini-blocks.**
   [Meeting 0922 requirements](../../meetup_record/meeting_0922_requirements.md) explicitly leaves this open, while [Meeting 0916 response](../../meetup_record/meeting_0916_response.md) gives P execution duties.
   A native receipt-bearing mini-block already presupposes execution.
   Choose either executed mini-blocks produced/checked by the regional service, with P batching/publication, or a distinct order-candidate object executed by P.
   The latter needs deterministic filtering or an execution acknowledgment before exact-output promises.
2. **One global Byzantine bound does not establish local safety.**
   Global adversarial stake below one third permits an entire small region to be adversarial.
   State a local less-than-one-third bound for every safety-relevant regional committee, or weaken local certificates to tentative status until a global quorum validates them.
3. **Terminology is reversed in parts of the outline.**
   Use `intra-region` for the fast local committee and `inter-region` or `global` for cross-region checkpoints.
4. **An empty timeout vote is not a native empty mini-block.**
   A failed leader cannot be relied on to sign a native empty header.
   Timeout certificates should preserve the certified tip and advance a consensus slot/view counter; their objects and success criteria must differ from produced blocks.
   An honest idle leader may emit valid empty mini-blocks, so emptiness alone is not evidence of failure.
5. **Two broadcasts and a supermajority count are not a complete recoverable BFT protocol.**
   Specify locks, persisted votes, view-change evidence, certificate availability and a safe successor-prefix rule.
   A local tip or subjective satisfaction signal is not sufficient to fence a competing partition.
6. **P failure is independent of S rotation.**
   A Byzantine sole P can halt publication despite a healthy stream.
   Settlement liveness must be conditional on responsive publication, or include permissionless relay/reassignment with publication-task inheritance.
7. **Global health is not universal reachability.**
   Heartbeats or a weighted health quorum attest to measured service under a stated predicate, not connectivity to every user/region.
   Subjective dissatisfaction can trigger conservative recovery; it cannot alone justify an objective slash.

## Minimum interfaces to expose in the model

- **Authority:** finalized L1 configuration identifier, eligible operators/regions, snapshot weights, key proofs, activation boundary, deterministic local/global leader schedules and standby list; system-transaction authority stays distinct.
- **Timing:** separate mini-block slot, complete-block cadence, local leader term, global checkpoint epoch, regional term and timeout parameters; example minutes/hours from the outline are not protocol constants.
- **Validity:** a deterministic predicate on ordered bytes, canonical context, predecessor state and mandated L1 inputs; specify whether committee voters execute.
- **Handoff:** retained prefix and certificate, complete-block checkpoint, unfinished-block state, retrievable bytes/witnesses, publication backlog, pending obligations, and an authorization certificate that fences old producers.
- **Publication:** a task binds P, start/end stream positions, end digest, availability evidence, assigned deadline and objective completion event.
  Check signatures, exact content/order, prefix continuity, DA retrieval and the relevant execution/settlement proof separately.
- **Incentives:** selection maps finalized stake and authenticated service history to ordered schedules; parameter maps set timeouts/terms/slash amounts; evidence predicates define objective violations independently of timeout or dissatisfaction triggers.
  Reserve retention and withdrawal-delay parameters so evidence remains enforceable after rotation.

The decisive compatibility test is: A signs the first two mini-blocks of EVM block N, then disappears.
The document must explicitly explain who may finish N, or which tentative content can be abandoned, and how recipients distinguish that decision from an unauthorized fork.
No claim of full fault-tolerant native integration is justified until this case and the authority-update path are specified.

## Memory trace used only to locate and recheck sources

Memory registry lines 214–225 identified the mini-block and handoff boundaries; all substantive source claims above were rechecked from the local files or current official pages.
Relevant memory entry: `MEMORY.md:214-225`, rollout `01a0aa2c-e62e-76b3-ab5e-4c2774f55b2c`.
