# MegaETH's pipeline: what exists, and what sequencer rotation would change

This note answers the first questions in [the September 16 meeting notes](../meetup_record/260916.md): what the sequencer and proposer do, how the complete pipeline works, and where the proposed rotation system would enter it.
The key distinction is that the existing protocol exposes **one sequencer role that produces execution-bearing, signed outputs**, while [the proposed architecture](../meetup_record/rotation_overview.jpg) separates transaction ordering from execution.
That separation creates a new protocol interface and new obligations; rotating an existing signing key does not define them.

## How to read the evidence

**Reference date: September 16, 2026.**
Local specification references use `megaeth_specs` commit `59797b45b1f59b7c2c147e6f99be39399a1f2b44`, dated September 14, 2026.
The following labels distinguish evidence from design work:

| Label | Meaning |
| --- | --- |
| **Specified** | Behavior explicitly stated in the local specification snapshot. |
| **Documented** | Described by current official public documentation; deployment details were not independently verified onchain or against running binaries. |
| **OP baseline** | Background inherited from, or compared with, the OP Stack; MegaETH-specific deltas and configuration still matter. |
| **Inferred** | A working reconstruction using the OP baseline or stated MegaETH architecture; needs confirmation against MegaETH-specific rules. |
| **Proposed** | The meeting sketch or a design implication drawn here, not an existing guarantee. |
| **Open** | The available evidence does not determine the answer. |

The local [rollup overview](../megaeth_specs/specs/rollup/overview.md) says MegaETH inherits the OP Stack settlement architecture, but every individual rollup page is currently a stub.
[Stateless validation](../megaeth_specs/specs/protocol/stateless-validation.md) is also a stub, and [SequencerRegistry](../megaeth_specs/specs/system-contracts/sequencer-registry.md) is a placeholder.
Consequently, the L2 wire formats are much better specified here than the settlement and authority-update mechanisms.
Rex5 frontmatter describes the applicable specification version; it is not evidence of activation on a particular network.

## 1. Building blocks and components

### A short glossary

| Term | Meaning in this note |
| --- | --- |
| Transaction | A requested state transition, such as a transfer or contract call. |
| Receipt | Execution output recording success or failure, gas consumption, and logs; it is not the complete resulting state. |
| State delta | Changes to accounts or storage that let a receiving node update its state. |
| Mini-block | An ordered portion of one EVM block, carrying transaction and receipt commitments. |
| EVM block | The containing Ethereum-format block, with a parent hash and aggregate transaction, receipt, and state commitments. |
| L1 origin | The Ethereum block associated with an L2 block's sequencing epoch in the OP baseline. |
| Batch | Encoded transaction input for later chain reconstruction; distinct from an execution result. |
| Data availability, or DA | Ability to retrieve the data needed to reconstruct and check the chain. |
| Derivation | Reconstruction of the canonical L2 input sequence from L1 and the referenced available data. |
| State/output proposal | A claim submitted for settlement about the result of executing a particular chain segment. |
| Preconfirmation | An early sequencer commitment whose precise promise must be stated; it is not automatically L1 finality. |

The mini-block and state terms follow [Blocks](../megaeth_specs/specs/protocol/blocks.md), [State Sync](../megaeth_specs/specs/protocol/state-sync.md), and [SALT](../megaeth_specs/specs/protocol/salt/overview.md).
The L1-origin definition follows the [OP glossary](https://specs.optimism.io/glossary.html#l1-origin).

### Separate responsibilities before naming operators

**Specified:** the sequencer is one externally visible role.
It supplies the signed mini-blocks and EVM blocks other nodes consume.
Whether one machine, a standby cluster, or multiple internal processes implement that role is outside the protocol boundary.
MegaETH does **not** specify a consensus-layer/execution-layer interface analogous to Ethereum's Engine API.
[Source: sequencing and its scope boundary](../megaeth_specs/specs/protocol/sequencing.md#block-production-mechanics).

**Documented:** an RPC endpoint accepts a user transaction and forwards it to the sequencer, which orders and executes it.
Replica RPC nodes apply streamed results without re-executing; full nodes independently execute the blocks.
Public documentation describes high-availability standbys, an EigenDA-backed batcher, and Kailua-based dispute proving.
Those are descriptions of the architecture, not evidence of a decentralized leader-election protocol.
[Source: MegaETH architecture](https://docs.megaeth.com/architecture).

For the rotation design, retain the following logical responsibilities even if one operator performs several:

| Responsibility | Question it answers |
| --- | --- |
| Ordering | Which transactions, in which sequence, under which block context? |
| Execution | What receipts and state changes result from that sequence? |
| Streaming | How do consumers receive authenticated results quickly? |
| Validation | Are those results the correct execution of the accepted inputs? |
| Batch publication | Which inputs become recoverable through the settlement path? |
| State proposal | What execution result is claimed to the settlement contracts? |
| Challenging/proving | How is a disputed execution claim checked? |
| Authority management | Who is entitled to issue each kind of commitment? |

This is a design decomposition, not a claim that the deployed network has eight separate operators.
In particular, “proposer” should mean **state proposer** when discussing the bottom box of the sketch; using the same word for the ordering leader would conceal two different responsibilities.

## 2. Block structures: mini-blocks inside EVM blocks

A mini-block is a **MegaETH-specific** structure; the OP Stack provides useful settlement context but does not define MegaETH's mini-block format.
Both formats are specified locally in [Blocks and Mini-Blocks](../megaeth_specs/specs/protocol/blocks.md).
The following table separates header commitments from the actual data they commit to.

| Feature | Mini-block | Containing EVM block |
| --- | --- | --- |
| Identity | Containing `block_number`, within-block `index`, and chain-wide mini-block `number` | EVM block number and block hash |
| Position/linkage | No parent hash; position is checked through ordered delivery and containing-block context | Header contains `parent_hash` |
| Time | `block_timestamp` is the containing block's seconds timestamp; `timestamp` uses microseconds from Rex5 | Header `timestamp` uses seconds |
| Transactions | Ordered transaction list; header `tx_root` commits to it | Body transaction list is the ordered concatenation of all mini-block transaction lists |
| Receipts | Body includes receipts; header `receipt_root` commits to them | Aggregate receipt list has a header commitment; receipts are execution outputs rather than ordinary block-body transactions |
| Gas | `gas_used` sums the mini-block transactions' total gas | Header accounts for total block gas and the block gas limit |
| State | No state root in the mini-block header; deltas travel separately through state sync | Header includes `state_root`, derived from MegaETH's SALT state |
| Logs bloom | No header bloom filter; receipts carry their blooms | Ethereum-format header includes a logs bloom |
| Other header fields | Exactly the eight fields listed below | Ethereum/OP-format fields, whose fork-specific semantics follow the applicable rules |

Every transaction belongs to exactly one mini-block and one EVM block; no mini-block crosses an EVM-block boundary.
An EVM block contains at least one mini-block.
`index` resets to zero in the next EVM block, while the chain-wide mini-block number increments without resetting.
For example, block `1200` may contain transaction lists `[a,b]`, `[c]`, and `[d,e]`; its transaction list is `[a,b,c,d,e]`.
Its transaction root is recomputed over that full list, **not** obtained by combining the three mini-block roots.
The same distinction applies to receipt roots.

A state delta is not a ninth mini-block header field.
[State Sync](../megaeth_specs/specs/protocol/state-sync.md#streams-and-payloads) transports it separately and binds it to the reconstructed EVM block.
Receipt gas counters are cumulative across the EVM block, so computing the gas used by `[d,e]` requires the preceding cumulative value rather than resetting the counter at each mini-block.
[Source: mini-block assembly](../megaeth_specs/specs/protocol/state-sync.md#mini-block-assembly).

### What the mini-block signature actually promises

From Rex5, the signed mini-block header commits to eight fields:

```text
block_number, block_timestamp, index, number,
timestamp, gas_used, tx_root, receipt_root
```

The signature covers `keccak256(rlp(header))` and must recover the key returned by `sequencerAt(block_number)`.
Every mini-block in one EVM block uses the same authorized key; a pending change takes effect before a subsequent block starts.
[Sources: mini-block signing](../megaeth_specs/specs/protocol/blocks.md#mini-block-header-signing), [sequencer authority](../megaeth_specs/specs/protocol/sequencing.md#sequencer-authority).

Three consequences matter immediately:

1. **This signature commits to execution results as well as ordering.**
   `receipt_root` and `gas_used` cannot be obtained from transaction ordering alone.
2. **The header contains no parent hash and no state root.**
   Position and linkage depend on the surrounding protocol and containing EVM block; this is not a self-contained chain of miniature Ethereum headers.
3. **Attribution is different from correctness and finality.**
   A signature identifies the accountable key and binds the signed data, but does not prove that execution was correct or that L1 will retain those transactions.

The [block specification](../megaeth_specs/specs/protocol/blocks.md#security-considerations) explicitly warns against treating the signature as finality.
Nor does describing equivocation evidence establish that a deployed contract automatically slashes it.

## 3. Block-building rules

### 3.1 Intake, admission, and execution are different checks

A transaction pool, often called a mempool, is a producer's collection of pending requests.
**Inferred operational model:** RPC acceptance and pool storage precede final block admission, but neither commits the protocol to include the request.
The local spec does not define a globally agreed mempool or require every candidate sequencer to see identical pending transactions.

For an ordinary transaction, the relevant signature, nonce, and balance must be admissible at its position in the evolving block state.
If an account's next nonce is `7`, a transaction with nonce `8` cannot simply execute first; an earlier transaction may advance the nonce or consume funds needed by a later one.
This follows from the ordinary validity checks identified in [Sequencing](../megaeth_specs/specs/protocol/sequencing.md#transaction-admission-and-ordering).
A signed ordering list therefore cannot treat transaction validity as independent of the chosen prefix.

A valid transaction can nevertheless **revert during execution**: for example, a swap's minimum-output condition may fail.
Its receipt then reports failure, while the transaction still occupies its position in the block.
This differs from an inadmissible transaction that cannot be included in that position under the applicable transaction rules.
An order-only design needs to distinguish these cases before it defines “the executor must follow the order.”

### 3.2 Mandatory prefix and ordinary ordering

**Specified:** every EVM block begins with a first mini-block that places all supplied deposit and L1-attributes transactions before ordinary transactions.
The ordinary transactions' ordering policy is otherwise unconstrained: the spec does not promise arrival ordering, fairness, or inclusion of every transaction seen by an RPC node.
Validation concerns admissible transactions and correct state transitions, rather than reproducing the sequencer's scheduling policy.
[Source: transaction admission and ordering](../megaeth_specs/specs/protocol/sequencing.md#transaction-admission-and-ordering).

Deposited transactions and MegaETH system transactions require separate treatment from ordinary user transactions.
The latter are identified by transaction format, the system signer, and a permitted target; they have special fee and nonce semantics.
Crucially, the **system address rotates independently of the mini-block signing key**.
A future rotation protocol must say whether it changes either authority, rather than assume both move together.
[Source: transaction types and signer authority](../megaeth_specs/specs/protocol/transactions.md).

### 3.3 Origin and time constraints

**OP baseline:** an L2 epoch identifies an L1 origin; its deposits, attributes, eligible batches, and preceding L2 state determine derivation.
For timestamps, the baseline requires:

```text
t = previous_L2_timestamp + block_interval
t >= origin_timestamp
if t > origin_timestamp + drift_bound:
    ordinary_sequencer_transactions = []
```

These are necessary timing conditions, not the complete batch-acceptance algorithm.
Beyond the drift bound, an empty sequencer batch still cannot retain an old origin when the next origin could be adopted without violating the minimum timestamp rule.
The origin hash must match the canonical L1 block, and origin progression is constrained.
The interval concerns EVM blocks, not mini-blocks.
[Source: OP derivation](https://specs.optimism.io/protocol/derivation.html#overview).

**Inferred MegaETH obligation:** origin selection must satisfy the inherited origin/time rules and MegaETH's deltas, rather than choose arbitrary historical L1 blocks.
Exact origin progression, drift parameters, and batch eligibility remain unverified because the local [derivation delta](../megaeth_specs/specs/rollup/derivation.md) is unfinished.

### 3.4 Resources and sealing

**Documented:** MegaETH distinguishes admission-time limits from execution-time accounting.
Its resource dimensions include total gas, compute gas, transmitted data, key/value updates, state growth, encoded transaction size, and DA size.
Some excessive requests are rejected before inclusion; some execution-time excesses produce an included transaction with a failed receipt.
For block-level data size, key/value updates, and state growth, the documented builder can include a final transaction that crosses the threshold, then stop adding transactions.
Thus “the block exceeds a resource number” is not by itself enough to diagnose an invalid block without checking that resource's exact accounting rule.
No numerical deployment limits are assumed here.
[Source: MegaETH resource limits](https://docs.megaeth.com/developer-docs/overview-3/resource-limits).

**Specified:** each EVM block needs at least one mini-block, but mini-block count and sealing cadence are implementation policy.
The public [mini-block guide](https://docs.megaeth.com/mini-block) describes roughly 10 ms mini-blocks and one-second EVM blocks; that operating cadence is not a validity rule requiring every EVM block to contain exactly 100 mini-blocks.
Mini-block wall-clock timestamps are not required to increase monotonically; `index` and `number` determine order.
[Sources: sequencing mechanics](../megaeth_specs/specs/protocol/sequencing.md#block-production-mechanics), [timestamp semantics](../megaeth_specs/specs/protocol/blocks.md#mini-block-timestamps).

## 4. Full lifecycle: from a user request to a settlement claim

### 4.1 Receive, choose context, execute, and seal

The producer receives candidate requests, chooses a valid block context and admissible sequence under section 3, executes transactions, and produces the structures from section 2.
Receipts and state deltas become available incrementally as mini-blocks are formed.
This is the low-latency path; publication and settlement proceed later.
The following steps describe logical dependencies, not a requirement that one thread or one operator perform everything serially.

### 4.2 Stream results and reconstruct state

**Specified:** state sync sends four separately authenticated streams: state changes, transactions, receipts, and EVM headers.
The first three provide one fragment per mini-block; the last supplies the sealed EVM header and metadata.
A consumer reconstructs the mini-block header from fragments and checks the mini-block signature against that reconstruction.
[Source: streams and assembly](../megaeth_specs/specs/protocol/state-sync.md#streams-and-payloads).

There are two distinct signature layers.
Mini-block header signing uses the keccak digest above, whereas stream payload authentication uses BLAKE3-based hashes and binds payload content to its stream position and block number.
Historical payloads are checked against the key authorized for their block, not whichever key happens to be current when they are downloaded.
[Source: payload authentication](../megaeth_specs/specs/protocol/state-sync.md#payload-authentication).

At EVM-block assembly, consumers check the parent link, aggregate transaction and receipt roots, the merged SALT-delta commitment, and the state root obtained by applying those deltas.
These checks detect inconsistent or corrupted delivery.
They do **not** independently establish that the deltas are the correct EVM execution of the transactions: a malicious sequencer can sign mutually consistent but incorrect outputs.
The [state-sync trust model](../megaeth_specs/specs/protocol/state-sync.md#trust-model) states this distinction explicitly.

A witness-based validator instead needs enough authenticated pre-state information to re-execute and compare the resulting outputs.
[SALT witnesses](../megaeth_specs/specs/protocol/salt/witness.md) specify authenticated state information, while the complete [stateless-validation procedure](../megaeth_specs/specs/protocol/stateless-validation.md) remains unwritten.
Thus “replica received the result,” “validator checked execution,” and “settlement accepted the result” are different milestones.

### 4.3 Publish reconstructable inputs through DA and L1

**Documented:** MegaETH sends batch data to EigenDA and posts a corresponding DA certificate through the L1 submission path.
The official integration description distinguishes distributed blob storage from the small onchain reference; Ethereum need not hold the full batch bytes in its own execution storage.
[Source: MegaETH's EigenDA integration](https://www.megaeth.com/blog-news/endgame-how-we-break-performance-limits-with-eigenda).

**OP baseline:** a batch submitter packages L2 inputs for publication, including compression and channel/frame handling.
This is an input-publication task, distinct from a proposer submitting an execution claim.
[Source: OP batch submitter](https://specs.optimism.io/protocol/batcher.html).

**Open:** MegaETH's [batch-submission stub](../megaeth_specs/specs/rollup/batch-submission.md) names a pre-submission validation gate, but does not define its algorithm or which validated head bounds submission.
Its [DA stub](../megaeth_specs/specs/rollup/data-availability.md) leaves certificate validity, retrieval failures, and DA outages unspecified.
Do not turn these titles into assumptions that every published block has already passed a particular validator quorum.

A certificate, retrieved bytes, and execution validity are three different objects.
For rotation, the practical question is whether a replacement can retrieve every input needed to extend the accepted prefix, including inputs for early commitments that have not yet reached durable publication.
A signature over unavailable bytes alone does not provide that ability.

**OP baseline:** execution applies the derived inputs to obtain state; batches do not themselves carry EVM-block execution commitments.
An L1 reorganization can change derivation and make it disagree with the fast feed.
[Source: OP derivation pipeline](https://docs.optimism.io/op-stack/protocol/derivation-pipeline).
This describes the settlement-facing data dependency, not a mandated internal CL/EL split for MegaETH.

### 4.4 Propose state, resolve disputes, and enable withdrawals

**OP baseline:** a state proposer submits a commitment to the execution result of an identified L2 history.
An output commitment can include more than the state root: the OP construction also binds withdrawal-related state and an L2 block hash.
Those exact fields must not be copied into MegaETH without checking its SALT and settlement adaptations.
[Source: OP output commitments](https://specs.optimism.io/protocol/proposals.html#l2-output-commitment-construction).

**Documented:** MegaETH's ZK-fraud-proof explanation describes an optimistic claim about executing a payload from an initial state, with a succinct proof used to resolve a disagreement about the result.
It describes an optimistic challenge model rather than requiring a validity proof for every ordinary state proposal.
The article is dated July 23, 2025; its timing and slashing statements are not independently verified deployment parameters here.
[Source: MegaETH on ZK fraud proofs](https://www.megaeth.com/blog-news/endgame-how-we-use-zk-to-secure-megaeth).

Kailua itself now supports hybrid and validity modes, so its available features do not determine which mode a particular MegaETH deployment uses.
[Source: Kailua repository](https://github.com/boundless-xyz/kailua).
The local [fault-proof page](../megaeth_specs/specs/rollup/fault-proofs.md) leaves the precise proposal/challenge flow and resolution rules open.
Accordingly, the sketch's arrow **“state root + proof”** should be understood as an unresolved interface: it must say whether a proof accompanies every proposal, is produced only in a dispute, or has some other role.

**OP baseline:** a withdrawal begins with an L2 message recorded in the Message Passer, is proven on L1 against an accepted output commitment, and is finalized only after the applicable settlement conditions.
[Source: OP withdrawals](https://specs.optimism.io/protocol/withdrawals.html).
The newer fault-proof bridge integration uses dispute games rather than the legacy output oracle and introduces game-resolution and maturity conditions.
This is why an old `L2OutputOracle` tutorial is insufficient to establish today's MegaETH withdrawal path.
[Source: OP bridge integration](https://specs.optimism.io/fault-proof/stage-one/bridge-integration.html).
MegaETH's [withdrawal delta](../megaeth_specs/specs/rollup/withdrawals.md), including SALT proofs, is still a stub.

### 4.5 A numbered transaction trace

Suppose Alice submits a swap with a minimum-output constraint.
The numbers below are illustrative identifiers, not network parameters.

1. **Before inclusion:** an RPC acknowledgment means her request arrived.
   It does not identify its final position or promise success.
2. **At ordering and execution:** the producer places the swap in mini-block `(1200, 3)` and executes it after the preceding transactions.
   An earlier swap may change the pool price; Alice's call can legitimately revert while still being included in a valid block.
3. **At early delivery:** Alice receives a receipt and state updates, with authenticated commitments.
   She can check that the authorized signer committed to those results, while an independent execution check is a separate task.
4. **At EVM-block assembly:** block `1200` commits to the concatenated transactions and receipts and its resulting state.
   The receiver checks consistency with the mini-block stream.
5. **At DA publication:** batch bytes are dispersed and referenced by a certificate.
   A verifier still needs to retrieve and check the bytes, rather than execute the certificate itself.
6. **At L1 inclusion and derivation:** a recognized batch submission references those inputs on Ethereum; a verifier derives and executes the relevant history.
   This is the transition toward the independently derived, safe chain described in section 5.
7. **At L1 finalization:** the L1 history needed for that derivation becomes finalized, subject to Ethereum's finality and the external-DA assumptions.
8. **At state-claim settlement:** a proposal covering the history is accepted through the applicable proof/dispute rules.
   Proposal submission need not wait until every earlier step in this pedagogical list completes; the concrete game's prerequisites determine scheduling.
9. **For a later withdrawal:** Alice initiates an L2 withdrawal, proves its inclusion against an eligible output, and completes the separate L1 finalization procedure.
   Her original swap did not itself require withdrawing anything.

Now suppose rotation occurs after step 3 but before step 5.
Knowing that block `1200` has an authorized signer is insufficient for a successor orderer: it needs the authenticated promised prefix, relevant ordered inputs, block context, and the executor's authenticated progress to know where ordering may resume.
A replacement executor or full block producer additionally needs execution state or a reconstructable checkpoint; whoever takes over publication needs the outstanding batch backlog.
If it simply starts a different block `1200`, Alice may hold a genuine commitment that the resulting chain does not honor.
This example is a design implication of the commitments above, not a claim that today's high-availability system follows that procedure.

## 5. L1 confirmation and data availability

### 5.1 Which kind of confirmation?

**OP baseline:** independently deriving nodes distinguish the following chain heads.
The terminology below follows [OP Stack transaction finality](https://docs.optimism.io/op-stack/transactions/transaction-finality); applying it to MegaETH requires its external-DA rules.

| Milestone | What has been established | What has not been established by this milestone alone |
| --- | --- | --- |
| Mini-block commitment | A particular authorized key signed particular contents | Correct execution, durable publication, or L1 finality |
| Unsafe EVM block | A producer supplied an L2 block ahead of L1 confirmation | That the same block will be derivable from the canonical L1 history |
| Safe L2 block | A node derives the block from accepted data in the current canonical L1 history | That the supporting L1 history cannot reorganize |
| Finalized L2 block | The required L1 history has finalized | That a bridge's separate output-claim and withdrawal conditions have completed |
| Settled output claim | The applicable settlement mechanism accepts an execution claim | That a particular withdrawal has been proven and finalized |
| Finalized withdrawal | The bridge has completed its required checks and L1 action | A guarantee about unrelated transactions or later blocks |

**Specified caveat:** MegaETH state-sync metadata contains `safe` and `finalized` markers, but these are the sequencer's reported view.
A replica receiving them does not thereby verify L1 finality.
[Source: block metadata](../megaeth_specs/specs/protocol/state-sync.md#block-metadata).
This matters when choosing a takeover checkpoint: a label supplied by the failing leader is different evidence from an independently derived checkpoint.

In the OP baseline, challenging an incorrect output claim does not rewrite the correctly derived L2 chain; it rejects a bad settlement claim about that chain.
The familiar withdrawal delay is consequently distinct from transaction finality.
[Source: OP Mainnet finality clarification](https://docs.optimism.io/op-mainnet/network-information/transaction-finality).
Do not import its exact timing into MegaETH.

### 5.2 What the DA certificate does, and what retrieval requires

**DA background:** EigenDA's whitepaper distinguishes dispersing bytes, verifying a certificate, retrieving bytes, and observing availability.
Its guarantees depend on storage honesty thresholds, an accessible retrieval path, and a defined availability window.
Certificate verification is a cryptographic check, while retrieval is a network operation; a certificate is not indefinite storage and does not prove EVM execution correctness.
[Source: EigenDA whitepaper, sections 1 and 3](https://docs.eigencloud.xyz/assets/files/EigenDA_Whitepaper-c917fb56c146ebd146abfba3c52648a1.pdf).

**Inferred MegaETH pipeline:** a deriving node encounters an accepted L1 reference, checks the applicable certificate rules, retrieves the committed batch bytes, checks their binding and encoding, and then derives the L2 inputs.
This is the natural reconstruction from the documented EigenDA integration and OP derivation baseline, not a completed MegaETH normative algorithm.
The [DA](../megaeth_specs/specs/rollup/data-availability.md) and [derivation](../megaeth_specs/specs/rollup/derivation.md) stubs leave its exact failure branches unresolved.

For design purposes, specify at least three distinct outcomes: a malformed or invalid certificate, a valid-looking certificate whose bytes cannot currently be retrieved, and retrievable bytes that decode to an invalid batch.
They need not trigger the same action.
A temporary retrieval failure must not silently authorize a successor to invent different transactions at already committed positions.
Likewise, finalizing the Ethereum transaction that contains a certificate does not independently establish that every challenger has obtained the external bytes.

### 5.3 A useful OP-derived fallback hypothesis

OP's [derivation-pipeline explanation](https://docs.optimism.io/op-stack/protocol/derivation-pipeline) describes progressing with deposit-only inputs after the sequencing window expires without a valid batch, discarding incompatible unsafe history.
**Inferred research baseline:** MegaETH may preserve an analogous L1-driven escape path, consistent with its public force-inclusion documentation.
Its external-DA timeouts, precise eligibility rules, and recovery implementation still need confirmation.
Treat this as a hypothesis for checking the design, not an established fallback algorithm.
It would also imply that some early ordinary-transaction commitments can fail to enter the derived chain unless the rotation mechanism ensures their timely publication.

## 6. What the proposed order-only sequencer changes

The [meeting sketch](../meetup_record/rotation_overview.jpg) assigns selection, inclusion, ordering, and origin choice to a sequencer, while a pre-confirmer executes, serves results, and publishes batches.
The existing mini-block signature cannot simply be renamed an ordering signature, because it already covers receipts and gas.

| Sketch element | Required design decision |
| --- | --- |
| Sequencer signs a transaction list | Define the signed context, transaction encoding, position, origin, predecessor commitment, and authority epoch. |
| Sequencer need not execute | Define handling of nonce/balance conflicts, resource exhaustion, and transactions that cannot be admitted at their signed positions. |
| Pre-confirmer executes the order | Specify whether it may reject the whole list, omit entries under a deterministic rule, or request a replacement; silent discretionary edits defeat the ordering promise. |
| Users verify the sequencer signature | Separately authenticate execution claims and specify how users check their relationship to the signed order. |
| Pre-confirmer publishes batches | Bind publication to the accepted order, define deadlines, and decide whether another publisher can recover from its failure. |
| State proposer submits results | Bind its claim to the canonical input sequence, including any permitted invalid-entry handling. |
| Another sequencer takes over | Select one successor and one inherited prefix under an explicit authority rule. |

These are proposed requirements, not added descriptions of the current protocol.
Several can be satisfied while preserving today's single active producer abstraction, but exposing separately accountable orderers and executors changes what other nodes must verify.

The current block-boundary authority rule also creates a concrete constraint: switching mini-block signing keys in the middle of an EVM block would require changing that rule.
Alternatively, rotation can take effect at an EVM boundary while a separate recovery procedure handles an incomplete block.
Choosing between those approaches requires specifying which early commitments survive failure.

Finally, replacing only the orderer does not repair a stalled executor, a missing DA publication, or an absent state proposer.
Each dependency needs either its own recovery path or an explicit assumption that it remains available.

## 7. Context still needed before choosing a rotation protocol

The official [force-inclusion guide](https://docs.megaeth.com/user-guide/force-inclusion) documents submitting through Ethereum's portal to bypass the sequencer and states a twelve-hour inclusion bound.
Treat that as a documented user-facing claim until checked against deployed configuration and failure behavior; the local [deposit/force-inclusion spec](../megaeth_specs/specs/rollup/deposits.md) does not supply the proof or mechanism.
Even a working slow L1 escape path would not establish the fast recovery target sought by the sketch.

The next implementation review should resolve these specific gaps:

- **Authority:** who may schedule registry changes, how consumers authenticate registry state, and how changes interact with L1 reorgs.
- **Publication:** who controls the batch-submission authority, what the validation gate checks, and whether a successor can publish inherited data.
- **Execution:** which acceptance and resource rules apply before an order-only commitment becomes binding.
- **Recovery:** which signed prefix a successor must preserve, and what happens when its data is unavailable.
- **Settlement:** the deployed game type, proposal permissions, bonds, proof program, challenge conditions, and withdrawal maturity rules.
- **Failure semantics:** actual behavior under missing DA certificates, unavailable blobs, sequencer outage, or a proposer that stops submitting.

For the meeting's question map: sections 1–5 cover components, block structures, building rules, the full lifecycle, and L1 confirmation/DA, respectively.
Section 6 identifies the changes introduced by the proposed split, and section 7 identifies the evidence needed before assigning liveness or validity guarantees.
The accompanying requirements and literature notes can then compare rotation mechanisms against this concrete pipeline.
