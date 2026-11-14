# Related sequencing systems: what they enforce, and what they pay for

**Reading date: 16 September 2026.** This note answers the related-project and incentive questions in [the meeting record](../meetup_record/260916.md). Public documentation was checked on this date. “Documented as live” means the project says a feature is live; this review did not independently inspect deployed contract storage, validator membership, or transaction traces. Research proposals are identified separately.

The central lesson is that **choosing the next sequencer, agreeing on a transaction order, checking execution, making data available, and paying operators are different mechanisms**. A project can decentralize one while retaining a trusted operator for another. MegaETH's rotation design needs an explicit answer for each.

## 1. Start with the service being decentralized

For this comparison, separate five jobs:

| Job | Output | Question to ask |
| --- | --- | --- |
| Ordering | A sequence of transaction bytes | Who chooses it, and who must agree? |
| Execution | State changes and receipts | Who computes the result, and who checks it? |
| Publication and data availability (DA) | Recoverable input data | Can another node reconstruct and continue the chain? |
| Settlement verification | An accepted state commitment | What stops an incorrect result becoming authoritative? |
| Authority transfer | A new authorized producer and starting point | What prevents two owners of the same production interval? |

These are conceptual roles, not a requirement to run five programs. A single operator may perform several jobs.

The [local MegaETH sequencing specification](../megaeth_specs/specs/protocol/sequencing.md) authorizes one key per EVM block; every mini-block in that EVM block uses that key. It constrains deposits and L1-attributes to precede ordinary transactions, but otherwise leaves ordinary transaction ordering unconstrained. Thus, a proposal to enforce arrival ordering, fair ordering, or a particular auction policy adds a new requirement. It cannot be justified by saying that existing execution validation already checks it.

The same local document deliberately leaves the sequencer's internal decomposition unspecified. Importing an OP-style internal interface or a shared-sequencer architecture is an engineering proposal, not a statement about MegaETH's current normative architecture.

## 2. Comparison at a glance

The detailed sources and limits appear in the following sections.

| System or mechanism | What changes hands or reaches agreement? | Enforcement | Incentive mechanism | Evidence status |
| --- | --- | --- | --- | --- |
| OP Stack plus OP Conductor | Active producer in a trusted high-availability cluster | Raft coordination; rollup derivation and execution rules remain separate | Operator economics; Conductor does not define a permissionless reward market | Documented tool; explicitly not Byzantine fault tolerant |
| Arbitrum BoLD | State assertions are disputed | Bonded challenges and onchain dispute resolution | Assertion/challenge bonds | Documented live; does not elect a sequencer |
| Arbitrum Timeboost | Temporary express-lane access | Auction contract plus trusted sequencer ordering policy | Auction payments for priority | Documented live; decentralized version is a separate specification |
| Espresso / HotShot | Agreement on ordered commitments | BFT certificates, DA mechanisms, rollup integration | Stake-based participation and proposer/delegator rewards in documentation | Production network documented; individual release and slashing claims require care |
| Astria | Ordered opaque rollup data | CometBFT agreement, rollup derivation, Celestia publication | Sequencing fees; exact production validator economics not established here | Architecture verified; no current validator/deployment audit |
| Taiko | L1 proposal ordering plus an offchain preconfirmation turn | L1 contracts, proofs, fast-path authorization, forced inclusion | Transaction fees and proving payments; planned stake-backed preconfs | Current docs describe whitelisted preconfs; permissionless stake stage planned |

## 3. OP Stack: distinguish censorship resistance from producer failover

### 3.1 What the rollup rules enforce

In OP's documented transaction flow, sequencing, batch publication, and state-commitment publication are different duties. The batcher compresses transaction data and publishes it to L1; the proposer publishes commitments to L2 state. This distinction matters when asking whether “rotate the sequencer” also rotates the batcher and proposer. It does not follow automatically. See [Transaction flow](https://docs.optimism.io/op-stack/transactions/transaction-flow).

OP derivation combines sequencer batches with information obtained from L1, including deposits. When the sequencing window expires without an acceptable batch, derivation can construct an empty sequencer batch; this does not remove mandatory deposit processing. The censorship-resistant path is a transaction submitted through L1 and handled by derivation, rather than a promise that every private RPC submission must be included. See [Derivation specification](https://specs.optimism.io/protocol/derivation.html) and [Rollup protocol overview](https://docs.optimism.io/op-stack/protocol/overview).

**Design implication for MegaETH:** define two service levels. The fast path may promise a short receipt latency under a functioning active sequencer. A fallback may promise eventual inclusion through another authority or L1-derived input. Specify different latency bounds, fees, and assumptions. A fallback that takes minutes or hours does not satisfy a millisecond trading service requirement, even if it protects eventual access.

### 3.2 What Conductor contributes

[OP Conductor](https://docs.optimism.io/chain-operators/tools/op-conductor) uses Raft for leadership and unsafe-block coordination, monitors sequencer health, and controls sequencing. Its documentation explicitly assumes honest participants and excludes Byzantine fault tolerance. The implementation description commits unsafe payloads through Conductor before gossiping them; this is useful operational protection against competing producers. See the [Conductor implementation documentation](https://pkg.go.dev/github.com/ethereum-optimism/optimism/op-conductor).

**Design implication:** Conductor is a useful reference for operational takeover: track a precise head, synchronize the replacement, and control publication during transition. Its trust assumption makes it insufficient as the sole mechanism for competing, potentially malicious operators. A malicious operator can ignore a local stop command. MegaETH therefore needs the authority boundary enforced by receivers and canonical derivation, not only by orchestration software.

### 3.3 The OP community already discussed epoch rotation

The 2022 Optimism forum thread [Pragmatic first step towards decentralizing sequencer](https://gov.optimism.io/t/pragmatic-first-step-towards-decentralizing-sequencer/2767) proposed a governance whitelist, epoch rotation, a reserve producer, possible bonds, and later auctions. The technical reply emphasized sharing pending transactions to avoid losing them during switches. This is a historical proposal, not evidence of deployment.

Its most relevant lesson is practical: preserving the chain head is only half of handoff. Pending requests, issued promises, and unfinished publication work also need owners. A transaction accepted just before the boundary should not disappear because it was never copied to the replacement.

## 4. Arbitrum: three mechanisms that should not be conflated

### 4.1 Sequencer feed and forced inclusion

Arbitrum's sequencer publishes a fast feed and later batches. The feed gives provisional ordering information; posting data to the parent chain and settling the resulting state are distinct milestones. Its delayed inbox and force-inclusion mechanism provide a bypass for censorship or outages. Current documentation also describes a censorship-timeout buffer that changes the force-inclusion delay. Exact delay parameters must be read from the relevant deployment. See [The Sequencer and Censorship Resistance](https://docs.arbitrum.io/how-arbitrum-works/deep-dives/sequencer).

For MegaETH, the transferable pattern is a canonical input channel that remains usable when the preferred producer stops cooperating. The open design issue is how that channel interacts with an unfinished rotation: can it trigger takeover, add inputs to the next producer's obligations, or both?

### 4.2 BoLD validates state, not the sequencer's business policy

[Overview of BoLD](https://docs.arbitrum.io/how-arbitrum-works/bold/gentle-introduction), updated 11 September 2026, documents BoLD as active on One, Nova, and Sepolia. Bonded validators post state claims and challenge incorrect ones. BoLD addresses dispute delay attacks and permits permissionless validation. The paper [BoLD: Fast and Cheap Dispute Resolution](https://arxiv.org/abs/2404.10491) develops the dispute mechanism.

This does not mean sequencers are permissionless, or that all their ordering decisions are checked by BoLD. Consider a sequencer that receives Alice's trade, excludes it, and executes Bob's trade correctly. The resulting state can satisfy the state-transition rules. To challenge Alice's omission, the protocol would need an additional rule and evidence proving that Alice was owed inclusion.

Similarly, an economically bonded state asserter and a bonded sequencer need not be the same actor. Slashing the former for a false state claim does not punish the latter for an unfulfilled receipt unless the system explicitly connects those liabilities.

### 4.3 Timeboost sells priority, not the sequencer machine

[How Timeboost works](https://docs.arbitrum.io/how-arbitrum-works/timeboost/gentle-introduction), also updated 11 September 2026, says Timeboost is live on One and Nova. A sealed-bid second-price auction allocates an express lane for a round; the documented defaults are a 60-second round and a 200-millisecond delay for ordinary submissions. The controller receives a time advantage, rather than ownership of the sequencer or unrestricted control over all transaction ordering. The documentation explicitly describes the sequencer as trusted to implement the ordering policy.

Offchain Labs separately publishes a [Decentralized time boost sequencer specification](https://github.com/OffchainLabs/decentralized-timeboost-spec). It describes a threshold-honest committee, certificates, clock assumptions, and optional threshold encryption. Treat that as a distinct design artifact; the existence of this specification does not establish that the live chains use it.

**Design implication:** decide whether MegaETH will auction production responsibility, some transaction-ordering privilege, or both. These are different products with different risks. A winner that only submits priority transactions need not maintain the full execution state. A winner that takes over the complete current sequencer role needs execution, synchronization, publication, and handoff capacity. In the sketch's order-only design, it needs the agreed ordering history and assigned publication duties, while execution readiness remains the executor's responsibility. Auction revenue alone cannot establish readiness for either role.

## 5. Espresso: shared confirmation does not mean shared execution

The paper [The Espresso Sequencing Network: HotShot Consensus, Tiramisu Data-Availability, and Builder-Exchange](https://eprint.iacr.org/2024/1189.pdf) separates block building, agreement on ordered commitments, and data availability. Its shared-sequencing layer does not execute every participating rollup's state machine. The builder-exchange problem concerns exchanging block information and payment commitments between mutually distrustful participants.

The [Espresso node documentation](https://docs.espressosys.com/network/concepts/the-espresso-network/internal-functionality/sequencer-node) similarly describes an L2-agnostic service: rollup-specific executors and provers interpret the ordered data. Therefore, integrating Espresso does not by itself verify MegaETH execution, preserve MegaETH state-diff continuity, or deliver a synchronized replacement execution server.

### 5.1 Keep three kinds of assurance separate

1. **Agreement:** a quorum has committed to an ordering under the consensus fault assumptions.
2. **Availability:** the committed data can be retrieved under the DA mechanism's assumptions.
3. **Execution validity:** applying that data to the agreed prior state produces the claimed next state.

[Espresso's DA process](https://docs.espressosys.com/network/concepts/the-espresso-network/properties-of-hotshot/espresso-data-availability-layer/how-it-works) combines committee availability with verifiable information dispersal and a retrievability certificate. It also explains that a chain can use Espresso confirmations while choosing Ethereum or another provider for its DA arrangement. The [light-client contract](https://docs.espressosys.com/network/concepts/the-espresso-network/internal-functionality/light-client) tracks HotShot commitments on L1; its consensus proof should not be mistaken for a proof of a particular rollup's execution.

**Design implication:** if MegaETH uses a committee only to certify a rotation checkpoint, specify exactly what members attest to. “I saw a hash,” “I have the input data,” and “I verified the resulting state” impose different duties and costs. A certificate for one cannot silently stand in for the others.

### 5.2 Incentives and release boundaries

The [Mainnet 0 release page](https://docs.espressosys.com/network/releases/mainnet-0) documents the October 2024 production release with a fixed operator set. Later [Mainnet 1 node instructions](https://docs.espressosys.com/network/releases/mainnet-1/running-a-mainnet-1-node) describe stake registration and a top-100 active set. The [rewards documentation](https://docs.espressosys.com/network/concepts/the-espresso-network/internal-functionality/stake-table/staking-rewards-calculation) describes proposer rewards shared with delegators, commission, and delayed stake activation.

Do not combine every release's features into one asserted live configuration. In particular, the generated [StakeTable contract documentation](https://espresso-network.docs.espressosys.com/contracts/src/StakeTable.sol/contract.StakeTable.html) calls its light-client reference currently unused and reserved for future slashing. This review does not establish a live slashing mechanism. Stake-based voting, reward allocation, and executable punishment are separate claims.

For MegaETH, this suggests a useful split: pay the active producer for delivery, pay standby infrastructure for readiness, and define punishments only after defining verifiable evidence. “Stake required” is not a complete enforcement specification.

## 6. Astria: a concrete example of ordering without rollup execution

[The Astria Sequencing Layer](https://docs.astria.org/overview/components/the-astria-sequencing-layer) describes CometBFT consensus with rotating proposers. Astria agrees on rollup data and executes its own native operations, such as value transfers and validator-set updates; it does not execute every rollup transaction. Validators receive data during consensus, and a relayer publishes data to Celestia.

The [transaction-flow documentation](https://docs.astria.org/overview/transaction-flow) separates a Composer that submits rollup data, the sequencer network, a Relayer, and a Conductor that supplies the rollup execution node. It distinguishes a sequencer commitment from the later Celestia-backed commitment used by its derivation flow. Its numerical timings are development-configuration examples and should not be read as current production service guarantees.

The [Composer documentation](https://docs.astria.org/overview/components/composer) explains how an operator can pay sequencing fees on behalf of ordinary rollup users. Sophisticated users can submit directly and express bundle preferences. The Composer's local arrival-order batching is not a global fairness guarantee. This review did not establish current production validator admission, stake distribution, slashing rules, or reward splits; those are unknown here.

**Design implication:** Astria illustrates how a distributed ordering service can have lower duties than MegaETH's active execution sequencer. Such a service can tell the next operator what to execute, but cannot replace the execution state that operator needs. For MegaETH, separately measure time to obtain the ordered log, time to reconstruct state, and time to serve useful receipts. A quick leader election can coexist with a slow execution takeover.

Also distinguish names: Astria's Conductor connects ordering/DA to a rollup engine. OP Conductor manages high availability among sequencer instances. They solve different problems despite sharing a name.

## 7. Taiko and based rollups: useful rotation ideas, with rollout qualifications

The original research post [Based rollups—superpowers from L1 sequencing](https://ethresear.ch/t/based-rollups-superpowers-from-l1-sequencing/15016) describes sequencing driven by L1 proposers and their builder/searcher ecosystem. This is an architectural family, not a statement that every deployed “based” system has identical admission or preconfirmation rules.

Taiko's current [protocol overview](https://docs.taiko.xyz/protocol/overview) identifies Unzen as the current fork, using Shasta's proposal/checkpoint architecture. It describes a preconfirmation-whitelisted fast proposing path, forced inclusion, and composed execution proofs including at least one ZK proof. Ethereum orders proposal transactions, while proving advances protocol finalization. The L1 inclusion of a proposal and proof-backed state finalization remain distinct events.

The [preconfirmation page](https://docs.taiko.xyz/protocol/preconfirmations) describes a producer that constructs and gossips blocks before proposing them to L1, with epoch turns. It labels whitelisted preconfirmations current and permissionless stake-backed participation planned. Some later lifecycle descriptions discuss that planned stage, so they should not all be treated as current deployment facts. This note uses the explicit rollout labels rather than assuming a roadmap target has shipped.

The [economics page](https://docs.taiko.xyz/protocol/economics) describes proposer revenue from transaction fees and costs for L1 publication and proving. It also states that the live Unzen liveness-bond parameter is zero. This is a useful warning against equating the existence of a bond mechanism in a design with a nonzero financial deterrent in production.

The research post [Based preconfirmations](https://ethresear.ch/t/based-preconfirmations/17353) discusses signed promises, distinct safety and liveness faults, and collateralized penalties; later discussion considers conditioning promises on L1 state to handle reorgs. These are research ideas, not automatically Taiko or Ethereum consensus rules.

**Design implication:** make every MegaETH receipt describe its scope: promised transaction, execution result if any, parent/checkpoint, producer epoch, deadline, and treatment of L1 reorganization. Determine whether the successor must honor the predecessor's outstanding promises. If so, the successor must receive their data and liabilities before taking over; otherwise the promise should explicitly end at a well-defined boundary.

## 8. Why neither optimistic nor validity proofs is the whole enforcement system

An execution proof answers a conditional question: given these inputs and this prior state, is this resulting state correct? An optimistic dispute mechanism answers the same type of question through challenges and adjudication. Their exact reach depends on the rules included in the verified computation.

Neither mechanism automatically answers whether a request was received in time, whether an operator deliberately delayed it, whether the incoming operator was ready, or whether ordinary transactions were ordered fairly. A producer can execute an empty or selectively populated block correctly. It can also stop producing altogether, leaving no false state transition to challenge.

This is not an argument that inclusion or ordering rules cannot be verified. They can become verifiable if the protocol supplies public inputs and defines a checkable rule: an L1 inbox, a signed inclusion promise, a certified receipt, or a specified ordering procedure. The economic and communication costs then become part of the design.

[Order-Fairness for Byzantine Consensus](https://eprint.iacr.org/2020/269.pdf), by Kelkar, Zhang, Goldfeder, and Juels, explicitly treats order-fairness as an additional property beyond agreement and liveness. Consequently, “all validators agreed on the block” is not evidence that the leader selected a fair order. For MegaETH, choose a concrete ordering policy before attempting to punish violations of it.

## 9. An incentive checklist for MegaETH rotation

The following are design recommendations derived from the comparison, not claims about existing MegaETH mechanisms.

### 9.1 Pay for distinct obligations

An integrated active producer incurs execution, bandwidth, publication, and staffing costs. In the proposed split, allocate each cost to the responsible orderer, executor, or publisher instead of charging all of them to the orderer. A standby executor may incur substantial synchronization costs without collecting active-period transaction fees; an order-only standby has a different cost structure. If only the active period is paid, an operator may rationally economize on its assigned readiness duties and cause a long takeover delay.

Separate compensation for delivered blocks, successful DA publication, readiness, and completed handoff where evidence is available. Avoid paying simply for self-reported uptime. A readiness payment needs an observable test tied to the state the operator must actually serve, while accounting for the cost of maintaining unused capacity.

### 9.2 Specify evidence before choosing a bond

| Alleged violation | Possible evidence | Main difficulty |
| --- | --- | --- |
| Equivocation | Conflicting signatures for the same defined position | Define position, epoch, and permitted reorg conditions |
| Breaking an inclusion promise | Signed receipt plus canonical history after its deadline | Establish admissibility, deadline, and exceptions |
| Unauthorized production | Signature and authoritative producer schedule | Ensure all verifiers use the same schedule version |
| Missing publication duty | A public publication obligation and expired deadline | Distinguish operator omission from unavailable external infrastructure |
| Poor readiness | A failed protocol-defined challenge or takeover | Prevent challengers from manufacturing failures |
| Unfair ordering | Evidence required by the chosen ordering rule | Private local arrival times alone do not establish a global order |

A larger bond does little against a deviation that cannot be proved. Conversely, an overly broad fault rule can penalize honest operators during partitions and drive them out of the market. Define recoverability and blame separately: switching away from an unreachable leader need not immediately prove that it deserves slashing.

### 9.3 Model profitability and deviations separately

A starting accounting identity is

\[
\text{operating profit} = \text{fees and rewards} + \text{allowed MEV}
 - \text{execution and readiness costs} - \text{DA/L1 costs}
 - \text{auction payment} - \text{capital cost}.
\]

Positive honest profit is a participation condition. Incentive compatibility additionally requires that each feasible deviation does not improve expected utility. For an identified deviation, compare its additional revenue or saved cost against expected enforceable penalties and lost future earnings. This is a modeling checklist, not an equilibrium theorem: detection depends on the evidence channel, collectable penalties depend on the bond and withdrawal delay, and future earnings depend on re-entry and identity rules.

An auction payment allocates a right; a bond secures an obligation. Keep them separate. Also specify who pays if the outgoing producer collects fees while the incoming producer must finish publication or proving. Unassigned residual costs invite strategic behavior at every boundary.

## 10. Focused reading path

Read the sources in this order to answer the meeting's concrete questions. Rows marked “proposal” or “paper” supply design ideas and arguments, not deployment evidence.

| Priority | Primary source | Type | What to extract for MegaETH |
| --- | --- | --- | --- |
| 1 | [OP Derivation](https://specs.optimism.io/protocol/derivation.html) | Protocol specification | Canonical input rules and fallback behavior |
| 2 | [OP Conductor](https://docs.optimism.io/chain-operators/tools/op-conductor) | Operations documentation | Head transfer, monitoring, and the honest-node assumption |
| 3 | [Pragmatic first step towards decentralizing sequencer](https://gov.optimism.io/t/pragmatic-first-step-towards-decentralizing-sequencer/2767) | Optimism forum proposal, 2022 | Epoch rotation, reserve producer, pending-transaction continuity |
| 4 | [BoLD: Fast and Cheap Dispute Resolution](https://arxiv.org/abs/2404.10491) | Research paper, 2024 | State-dispute liveness and economic exposure |
| 5 | [Decentralized time boost sequencer](https://github.com/OffchainLabs/decentralized-timeboost-spec) | Design specification | Committee assumptions and the distinction between priority rights and sequencing |
| 6 | [Espresso sequencing-network paper](https://eprint.iacr.org/2024/1189.pdf) | Research paper, 2024 | Separation of agreement, DA, execution, and builder exchange |
| 7 | [Astria Transaction Flow](https://docs.astria.org/overview/transaction-flow) | Architecture documentation | Ordering commitments versus DA-backed derivation |
| 8 | [Taiko Preconfirmations](https://docs.taiko.xyz/protocol/preconfirmations) | Documentation with rollout stages | Producer turns and early execution receipts |
| 9 | [Based preconfirmations](https://ethresear.ch/t/based-preconfirmations/17353) | Ethereum Research proposal, 2023 onward | Promise scope, safety/liveness faults, L1 conditions |
| 10 | [Order-Fairness for Byzantine Consensus](https://eprint.iacr.org/2020/269.pdf) | CRYPTO 2020 paper | Why order-fairness needs its own definition and mechanism |

Before implementing any borrowed mechanism, write a one-page interface for it: input, output, signer, verifier, deadline, fault assumption, fallback, payment, and evidence of breach. That exercise will reveal which existing mechanism can be reused and which part of MegaETH rotation still needs a protocol design.
