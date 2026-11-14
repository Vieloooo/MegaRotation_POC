# A formal model for MegaETH sequencer rotation

Read [the pipeline](01_megaeth_pipeline.md) and [the requirements and failure modes](02_requirements_and_failure_modes.md) first.
This note answers how to formalize sequencer behavior and what a rotation protocol must decide.
The message formats and requirements below are **proposed research interfaces**, not existing MegaETH formats or a completed consensus protocol.
They are a starting point for designing and testing the architecture in [your sketch](../meetup_record/rotation_overview.jpg).

## 1. State the objective before choosing an election algorithm

The proposed system gives a changing set of operators responsibility for selecting and ordering transactions, while a separate execution service produces results.
Its central objective should be:

> Replace a failed or malicious ordering operator while preserving an agreed transaction history, preserving correct execution, and keeping the interruption within a stated service target under stated network and infrastructure assumptions.

This contains several independent tasks: selecting an operator, agreeing on its authority, committing an order, executing it, making its data recoverable, and replacing failed components.
A round-robin schedule solves only selection.
A signature identifies a commitment's author; it does not make that commitment unique, executable, available, or final.

There are two natural designs to investigate:

| Design | What the rotation protocol decides | Consequence |
| --- | --- | --- |
| Rotate the ordering role; retain an execution service | The next ordering key and the transaction prefix it inherits | Lower execution requirements for candidates, but executor failure and censorship need a separate recovery path |
| Rotate the complete block-production role | Ordering, execution, streaming, and the corresponding signing authority | Closer to the current external role, but every successor needs execution capacity and usable state |

Your sketch chooses the first direction.
Treat it as the main research candidate, while using the second as a comparison baseline.
The [current sequencing spec](../megaeth_specs/specs/protocol/sequencing.md#block-production-mechanics) treats the sequencer as one externally visible role and does not define the proposed ordering-to-execution interface.

## 2. Keep the different clocks and identifiers separate

Use the following notation throughout the design.

| Symbol | Meaning | Why it is separate |
| --- | --- | --- |
| `c` | Chain identifier and protocol-version domain | Prevent interpreting a signature in another chain or protocol |
| `r` | Rotation term: an interval with a fixed candidate/committee configuration | Changes when the membership or planned assignment changes |
| `v` | View: a failover attempt within a term | Changes when a leader is replaced without changing membership |
| `b` | EVM block number | The current MegaETH signing authority is resolved at this granularity |
| `j` | Mini-block index inside `b` | Identifies a fragment of the EVM block |
| `k` | Position in the proposed ordered log | Connects decisions across leader changes |
| `ell = (L1 number, L1 hash)` | L1 origin | Determines relevant L1 context; it is not the rotation term |
| `a` | Authority-configuration identifier | Binds the decision to a particular membership and key schedule |

In OP terminology, a sequencing epoch is associated with an L1 origin.
Use “rotation term” for `r` so that two unrelated meanings of “epoch” cannot silently enter the same rule.
The OP baseline constrains origin progression, origin hashes, and timestamps; the origin is not an arbitrary parameter the leader can choose to its advantage.
Exact MegaETH extensions still require implementation/spec confirmation because its [derivation page](../megaeth_specs/specs/rollup/derivation.md) is a stub.
See the [OP derivation batch-validity rules](https://specs.optimism.io/protocol/derivation.html#batch-queue).

For the first prototype, use a fixed committee during each term and make planned transitions occur at completed EVM-block boundaries.
This reduces the number of simultaneous problems; it does not resolve emergency replacement during an incomplete block.

## 3. Model authorities individually

An operator may run several services, but the model should still distinguish their permissions.

| Authority | Object it authorizes | Rotation question |
| --- | --- | --- |
| Ordering key | Proposed signed transaction order | Which candidate leads in `(r,v)`? |
| Committee voting keys, if used | Consensus votes or transition certificates | What membership and voting weight apply? |
| Execution/mini-block signing key | Execution-related mini-block and stream commitments | Does it remain fixed when the ordering key changes? |
| Mega system transaction key | Privileged maintenance transactions | Is it separately administered and recoverable? |
| Batcher submission authority | Data/certificate submissions recognized by derivation | Can a successor actually publish? |
| State-proposal/proof permissions | Settlement claims and their defense or challenge | Can settlement continue after another role fails? |
| Configuration/update authority | Membership, rules, emergency changes | Can it act when the L2 stops? |

These rows are an analysis decomposition, not a claim that MegaETH currently has seven separate operator organizations or seven separate keys.
In particular, the existing [state-sync authentication](../megaeth_specs/specs/protocol/state-sync.md#payload-authentication) and [mini-block signature](../megaeth_specs/specs/protocol/blocks.md#mini-block-header-signing) use the authorized sequencer key.
Giving an independent executor that authority while introducing an ordering key requires an explicit compatibility design.

One separation already exists in the written spec: the [mega system transaction signer](../megaeth_specs/specs/protocol/transactions.md#mega-system-transaction-signer-authority) is tracked independently of the mini-block signer.
Do not rotate that privilege accidentally when rotating ordering rights.

## 4. Define what the ordering signature promises

### A candidate order message

A research message could have the following logical contents:

```text
Order = (
    chain_and_version, authority_config, rotation_term, view,
    log_position, target_evm_block, parent_evm_block_hash,
    previous_order_digest, l1_origin_number, l1_origin_hash,
    execution_context_digest, mandatory_input_digest,
    ordered_transaction_root, transaction_count,
    block_boundary_marker, promise_conditions
)
OrderSignature = Sign(ordering_key, DomainSeparatedHash(Order))
```

The transaction root commits to **an ordered, length-bound list**, not an unordered set.
The bytes must be retrievable; a root alone cannot be executed.
The precise encoding, hash scheme, inclusion proofs, and domain separator remain design decisions.
Do not reuse this pseudocode as a wire format.

Each group of fields prevents a different ambiguity:

- The authority and domain fields identify who may sign and which rules govern the promise.
- The parent and previous-order fields prevent attaching the same list to a different history.
- The L1 origin and execution context fix inputs that can affect transaction validity and results.
- The mandatory-input commitment binds the order to the required L1-derived prefix.
- The boundary marker determines whether a chunk continues or closes an EVM block.
- The promise conditions state the deadline, guarantee level, and any permitted cancellation or L1-reorg condition.

The execution context needs a canonical definition covering every execution-relevant input, rather than a selected list that omits inconvenient fields.
For example, timestamps, applicable protocol rules, fee parameters, and system-transaction placement cannot remain discretionary after an exact execution promise is made.
The local [block context](../megaeth_specs/specs/protocol/state-sync.md#block-context) is a useful inventory, but it is not already an ordering-contract format.

An order can commit only to ordinary transactions if the protocol also deterministically specifies how mandatory transactions are inserted.
Otherwise “the executor cannot alter the list” is ambiguous: inserting a required deposit might look like a violation.
The current [admission rule](../megaeth_specs/specs/protocol/sequencing.md#transaction-admission-and-ordering) places supplied deposits and L1 attributes before ordinary transactions in the first mini-block.

### An execution response is a different commitment

For discussion, write:

```text
ExecutionAck = (
    order_digest, parent_execution_commitment,
    accepted_transaction_root, receipt_commitment,
    resulting_state_commitment, execution_status
)
```

An acknowledgment needs its own authenticated source and semantics.
The resulting-state field is a proposed commitment, which could be available at a completed EVM block or supplied by an additional construction.
It is **not** an existing mini-block-header field.
The current [mini-block header](../megaeth_specs/specs/protocol/blocks.md#mini-block-header) has transaction and receipt roots and gas usage, but no state root or predecessor hash.

An executor's signature can establish what it claimed, not that execution was correct.
Correctness still needs independent checking or a proof tied to the agreed inputs.
The distinction follows directly from the existing [state-sync trust model](../megaeth_specs/specs/protocol/state-sync.md#trust-model).

## 5. Resolve invalid transactions before promising exact execution

An ordering service that does not execute has incomplete information about state-dependent validity.
Consider two signed transactions from the same account with the same nonce.
Each may look individually plausible at admission, but both cannot be included as ordinary valid transactions in one execution sequence.
Likewise, an earlier transaction may consume funds that a later one needs.

Choose explicitly between these interfaces:

| Interface | Meaning of a signed list | Handling of an invalid candidate |
| --- | --- | --- |
| Exact executable order | Every listed transaction belongs to the promised execution sequence | Obtain a validation acknowledgment before a strong promise; define rejection and recovery if validation fails |
| Ordered candidates | The list is input to a deterministic admission/filtering function | Specify the function, preserve order among accepted candidates, and make every rejection reproducible |

The second design allows a lighter ordering operator but changes the meaning of “include this transaction.”
It also requires agreement on the mapping from ordered candidates to canonical executed transactions and on how settlement validates that mapping.
A private executor policy such as “skip anything troublesome” is not sufficient.

For an exact-order prototype, distinguish a sequencer's preliminary proposal from an execution-backed promise.
A signed rejection should identify the candidate, the relevant predecessor state, and the rejection condition.
Some rejection claims need replay or a proof to adjudicate; a reason string is not evidence.
If the executor simply refuses to answer, swapping ordering leaders will not repair the service.

Also distinguish **invalid transaction** from **valid transaction whose contract call reverts**.
A revert can be the correct execution outcome and consume gas.
An inclusion promise alone does not promise a successful swap, mint, or transfer.
Research on [BFT and proposer-promised preconfirmations](https://ethresear.ch/t/analyzing-bft-proposer-promised-preconfirmations/17963) is useful for separating the promised property from the mechanism enforcing it.

## 6. Formalize each participant's behavior using its information

Let `h_i(t)` be participant `i`'s local history up to time `t`.
It contains messages actually received, the participant's L1 view, any local state or mempool, observed certificates, local timers, and its own prior actions.
It does not contain an omniscient global mempool, other nodes' receipt times, or transactions never delivered to it.

A behavioral strategy is a history-dependent policy:

```text
pi_i(h_i(t)) -> a distribution over feasible next actions
```

Randomization is optional.
Feasibility is constrained by information, cryptography, and available resources; the prescribed honest policy is a separate restriction on which feasible actions follow the protocol.
An ordering operator's action set includes proposing a list and context, signing, publishing bytes to selected recipients, forwarding evidence, acknowledging a handoff, remaining silent, or producing conflicting messages.
For a rational model, also include accepting side payments, choosing infrastructure effort, withholding selected transactions, and deciding whether to join or leave.
For a Byzantine model, allow arbitrary behavior within the cryptographic and corruption assumptions.

Define different policies for the executor, voters, batcher, state proposer, and challengers.
An honest executor checks the applicable order and execution rules; an honest voter checks the predicate its vote actually attests to.
If voters do not execute, an order certificate cannot be described as an execution-validity certificate.

Separate three predicates:

```text
OrderWellFormed(O, history)
Executable(O, predecessor_state, canonical_context)
PublicationComplete(O, retrieval_and_settlement_evidence)
```

The first can cover authentication, authority, list encoding, parent linkage, and admissible context.
The second covers state-dependent validity and execution constraints.
The third depends on a precisely chosen DA and publication policy.
No implication between these predicates should be assumed without a mechanism establishing it.

Keep independent progress records for ordering, execution, data availability, L1 derivation, and settlement.
They may advance at different times; for example, data can be dispersed before a state claim is proposed.
A single variable called `confirmed` conceals this distinction.

## 7. Specify handoff as transfer of authority and an inherited prefix

### Planned handoff

A handoff record should identify the old and new authority configurations, the activation boundary, a digest of the retained prefix, evidence justifying that prefix, and where its bytes can be obtained.
It should also identify the execution checkpoint and outstanding publication obligations.
Include a manifest of outstanding signed promises and acknowledged pending requests, with their deadlines and the actor responsible after handoff.
Requests not transferred need explicit expiry or retry semantics; a change of leader should not silently cancel a binding inclusion promise.

An example is: “candidate B becomes the orderer after completed EVM block 100; the inherited prefix ends at order digest `q`; batches through block 98 are already recorded; blocks 99–100 still require publication.”
The successor needs enough authenticated data to distinguish unfinished work from work already completed.
Repeating publication may be harmless only if the relevant interface defines deduplication and canonical interpretation.

Consumers accept a new leader because they verify the transition under the protocol's authority rule, not because a new endpoint claims leadership.
An old leader may keep sending messages forever; **fencing** means that messages outside its authority are rejected.
Key deletion on the old machine is not the safety mechanism.

### Emergency handoff

A local timeout establishes that a node has not observed progress within a chosen interval.
It does not prove that the leader is malicious, that every other node also timed out, or that no prior commitment exists.
The replacement mechanism therefore needs an agreed transition and a safe prefix-selection rule.

[HotStuff](https://arxiv.org/html/1803.05069v6) provides a primary reference for leader replacement under partial synchrony.
Its safety uses voting, locking, and commit rules; leader replacement carries information about earlier certificates.
For its fixed `3f+1` replica model, certificates use `2f+1` votes.
That threshold is not a stand-alone consensus algorithm: importing only the vote count omits the rules that prevent conflicting decisions across views.

For this project, choose an existing consensus protocol and specify exactly what the application submits to it.
If consensus commits only leader assignments, it does not automatically commit all orders signed by those leaders.
If it commits order prefixes as well, explain when a user may rely on that stronger commitment and how the extra latency is exposed.

A later view may replace an uncommitted proposal under the selected protocol's rules.
It cannot erase an already committed prefix merely because more nodes now prefer a different one.
New committee membership also needs a verified transition from old membership; a certificate under an arbitrary new committee cannot authorize itself.

## 8. The incomplete EVM-block case is a blocking design decision

Suppose operator A signs mini-blocks 0 and 1 for EVM block 101 and then disappears.
Under the current [authority rule](../megaeth_specs/specs/protocol/sequencing.md#sequencer-authority), every mini-block in that EVM block uses the same authorized key.
Operator B cannot simply sign mini-block 2 with a different key and claim compatibility with that rule.
Nor can it skip to block 102 without determining the valid parent block 101.

There are several possible design directions, with different consequences:

1. **Keep the executor/signing authority available while rotating only ordering.**
   The executor can finish the block from an agreed prefix, but the proposed order protocol still needs a rule for changing orderers within it or closing it early.
   Executor failure remains a separate case.
2. **Recover the existing authorized signing service.**
   This may fit operational high availability, but its trust and custody assumptions differ from replacing an independent operator with another independent key.
3. **Define a protocol upgrade for interrupted blocks.**
   Possibilities include explicitly scoped replacement authority or abandoning a tentative suffix and rebuilding from a completed checkpoint.
   Either choice needs changes to acceptance, signing, derivation, and promise semantics; a discarded suffix cannot have been advertised as irrevocable.

For the initial model, keep planned rotation at completed-block boundaries and classify incomplete-block recovery as a separate transition that must be designed before claiming full fault tolerance.
Do not hide this case in “the next sequencer takes over.”

## 9. Turn desired guarantees into precise obligations

The following are proposed properties to prove or test, not established theorems about the sketch.

| Property | Proposed statement | Required qualification |
| --- | --- | --- |
| Authority agreement | Honest participants using the same finalized authority history agree on who may produce each class of message | Specify configuration finality and behavior during L1 reorgs |
| Prefix safety | Histories accepted at the chosen commit level by honest participants are prefix-compatible | Define that commit level; a leader signature alone is insufficient |
| Execution correctness | Each accepted execution result follows the agreed ordered inputs and canonical context | State who checks it and when; filtering, if any, is part of the rule |
| Order preservation | An executor cannot silently insert, remove, or permute ordinary transactions covered by an exact-order promise | Account explicitly for mandatory inputs and invalid-candidate semantics |
| Recovery availability | A correct successor can retrieve enough data and state to resume from the retained prefix | State retention duration, storage roles, and DA/network assumptions |
| Service liveness | The protocol eventually produces another valid block after failures within the assumed bound | Requires responsive ordering, execution, data, and authority-update paths |
| Inclusion liveness | An eligible transaction becomes included, or gets a permitted, verifiable rejection, within a specified bound | Define receipt, eligibility, capacity allocation, deadlines, and fallback route |
| Settlement progress | Produced history can continue through publication, proposal, and final resolution | Requires DA, L1 inclusion, and the relevant proving/challenging services |
| Accountability | A specified violation has attributable evidence and an available adjudication path | Evidence availability, challenge costs, and withdrawal delays matter |

For prefix safety, “prefix-compatible” means one accepted history extends the other; honest nodes may be at different heights.
For inclusion liveness, eligibility cannot mean merely “the user sent a transaction somewhere.”
It must cover validity under the promised admission rule, a verifiable delivery event, available capacity or a defined queue, and protection against indefinite priority starvation.

Use an explicit partial-synchrony assumption when claiming bounded failover: after an unknown stabilization time, relevant communication is bounded and enough required participants are responsive.
Before that point, preserve safety even if service pauses.
The [HotStuff model](https://arxiv.org/html/1803.05069v6#S3) supplies one precise reference model; Ethereum and EigenDA dependencies need their own assumptions rather than inheriting a committee-network bound.

## 10. Connect the protocol model to the later incentive model

First decide which deviations are invalid, which are provable violations, and which remain permitted choices.
Only then define utilities.
One possible accounting decomposition is:

```text
utility = ordering revenue + service rewards + MEV + side payments
          - operating costs - publication/proof costs
          - penalties - lost future business
```

This is a proposed model, not MegaETH's existing payment mechanism.
Costs and revenue belong to the actor that actually bears or receives them.
An orderer paid immediately while an executor pays all DA costs can face very different incentives from an integrated operator.

For a particular deviation, compare incremental gain with expected enforceable loss.
Detection, successful adjudication, and collectible collateral all affect that loss.
The condition “gain is smaller than expected loss” is only a local deterrence check; it is not an equilibrium proof and says little about coalitions, external sabotage benefits, or unbounded cross-chain exposure.
The [related-systems note](04_related_systems_and_incentives.md) develops the mechanism comparisons.

## 11. A concrete next research package

Start with fixed membership, a named Byzantine fault model, one active orderer, and an explicitly modeled execution service.
Keep membership selection and reward auctions outside the first safety exercise.
Select a consensus protocol, define whether its decisions cover authority, order prefixes, or both, and specify the recovery data each decision requires.

Before optimizing throughput, run these adversarial traces against the proposed rules:

| Trace | Expected question the design can answer |
| --- | --- |
| Leader sends different orders to two clients | Which promise is binding, what evidence is retained, and can both clients claim finality? |
| Leader signs a root but withholds all bytes | Can anything be committed, and who supplies recovery data? |
| Leader dies after two mini-blocks | Who can finish the same EVM block under the signature rules? |
| A user receives an inclusion promise just before rotation | Who inherits the request, its deadline, and any liability for omission? |
| Old leader returns after replacement | Which messages are fenced, while valid historical data remains usable? |
| Executor drops one ordered transaction | Is rejection justified, and what establishes the canonical executed list? |
| Executor refuses all valid orders | What component is replaced, rather than cycling innocent orderers? |
| L1 origin is reorganized | Which tentative promises are cancelled and which prefix is recomputed? |
| DA or the only batcher fails | Can publication resume without violating previously issued promises? |
| Two partitions select different successors | Why can they not both commit conflicting histories? |
| Committee changes during an unfinished handoff | Which configuration can certify the transition? |

A useful first deliverable is a small executable state-machine model and these traces, followed by a safety argument for the selected protocol plus the new application rules.
The key measurement is not only leader-switch time: separately measure time to the next executable order, complete EVM block, recoverable data, and L1 publication.
These measurements expose which part of the proposed split actually improves availability and which part remains a dependency.

## Reading onward

Return to [requirements and failure modes](02_requirements_and_failure_modes.md) to connect each transition to an attack, or read [related systems and incentives](04_related_systems_and_incentives.md) to choose concrete protocol families.
The [learning guide](README.md) collects the open decisions that need resolution before this model can become a MegaETH specification.
