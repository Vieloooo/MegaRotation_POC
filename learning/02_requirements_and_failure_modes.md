# Requirements and failure modes for sequencer rotation

Reading date: 16 September 2026. This note answers the requirements and misbehavior questions in [the meeting notes](../meetup_record/260916.md), using [the proposed architecture](../meetup_record/rotation_overview.jpg), the local MegaETH specification, Zotero, academic papers, and protocol-community discussions.

**Main conclusion:** rotating the party that chooses transaction order addresses only one part of the system. The design must separately preserve the chosen order, execute it correctly, make its data available, complete unfinished blocks, publish batches, and advance settlement. Each obligation needs an identified actor and an explicit failure response.

Statements about the existing protocol link to the specification. The requirements, catalogue, and recommendations below are **design analysis for the proposed architecture**, not claims that MegaETH already implements these mechanisms.

## 1. Start with the correct responsibility boundary

The diagram separates an **order-only sequencer** from a **pre-confirmer that executes and publishes**. The sequencer chooses an allowed L1 origin and a transaction list, signs its order, and need not execute transactions. The pre-confirmer produces execution results and batches; the state proposer submits state claims and the relevant proof material.

This is an architectural proposal. The local [sequencing specification](../megaeth_specs/specs/protocol/sequencing.md) does not define this split as a protocol interface. Its externally visible producer is the authorized mini-block signer. Moreover, the existing [mini-block signature](../megaeth_specs/specs/protocol/blocks.md#mini-block-header-signing) covers transaction and **receipt** roots and gas used. A party that signs only an unexecuted transaction order is making a different commitment from a party that signs those execution-related fields.

The proposed design therefore needs two explicit statements:

1. **Order commitment:** “I authorize these transaction bytes, in this order, for this chain, round, block context, and L1 origin.”
2. **Execution commitment:** “Applying that authorized input under the specified rules and parent state produces these results.”

The first signature cannot establish the second statement. Verifying it protects users against an executor silently changing the input only if the client also checks that the returned result is bound to that input. Execution correctness still needs re-execution, an appropriate proof, or an explicitly trusted execution service.

Likewise, an executor signature proves who made a claim; it does not itself prove the claim true. A validity proof proves only the relation encoded in its circuit. If authorized ordering, origin selection, or mandatory inclusion is absent from that relation, execution validity alone does not enforce those rules.

## 2. State the assumptions before calling a requirement a guarantee

Use the following initial threat model, then refine it when choosing a rotation protocol:

- Candidates, the active sequencer, executor, publisher, and state proposer can fail independently or collude. One machine serving several roles creates correlated failures.
- Users and searchers may submit conflicting, invalid, or costly requests. A correct service must survive adversarial input without promising to process unlimited traffic.
- The network may delay or partition messages. State explicitly whether eventual progress assumes eventual synchrony, and what timing assumptions support any fixed deadline.
- Ethereum may reorganize before finality. Ethereum censorship or loss of finality is a separate dependency, not a sequencer offense by definition.
- DA retrieval, proof generation, monitoring, key custody, and administrator availability each have their own assumptions.
- Cryptographic primitives, the modeled execution rules, and the adjudication contracts are assumed correct for the protocol analysis. Implementation bugs, compromised dependencies, and upgrades require additional review.

“All possible misbehaviors” is meaningful only relative to specified roles, interfaces, and assumptions. The catalogue below covers these interfaces systematically; it is not a claim to enumerate every software exploit or future economic attack.

## 3. The properties we actually need

| Property | Meaning for this design | Example of failure |
| --- | --- | --- |
| Authorization validity | Accepted commitments come from the actor authorized for their exact round and block context. | Yesterday's leader is still accepted after replacement. |
| Input and execution validity | Mandatory inputs and resource rules hold; state transitions follow the execution specification. | Deposits are omitted, or an executor invents a balance. |
| Safety / consistency | Honest consumers do not accept incompatible histories at the same stated assurance level. | Two conflicting orders are both treated as irreversibly committed. |
| Progress liveness | Under the stated network and availability assumptions, complete blocks and later settlement stages continue advancing. | Mini-blocks stream forever but no complete EVM block is published. |
| Inclusion liveness | An eligible request reaching the specified submission channel is eventually included, or meets a stated deadline. | Blocks continue while one user's eligible requests are permanently excluded. |
| Availability / recoverability | Required transaction data, state, and evidence can be obtained by the parties that must verify or resume service. | The next leader has a root but cannot recover the corresponding data. |
| Ordering fairness | An explicit policy constrains which of several otherwise valid orders is allowed. | A promised arrival-order rule is violated for a bribe. |
| Accountability | A violated obligation can be attributed using evidence that the specified adjudicator can evaluate. | Everyone suspects censorship, but nobody can prove receipt or the applicable deadline. |

These properties are distinct. Valid empty blocks can violate inclusion liveness. Two different forks can each execute validly yet violate agreement. An objectively punishable double-sign can still cause losses before punishment. A fair order need not be profitable or efficient.

**Fairness is currently a design choice.** MegaETH's local sequencing specification constrains deposits and L1 attributes relative to ordinary transactions, but leaves ordinary transaction ordering unconstrained. A sandwich or priority purchase may be undesirable without violating that existing ordering rule. Before punishing “unfair order,” define whose observations count, which requests are eligible, and how conflicting observations are resolved. Themis illustrates that fair ordering adds assumptions and protocol machinery beyond ordinary agreement; its receiver-order criteria should not be replaced with an undefined global first-arrival clock. [Themis, paper and definitions](https://eprint.iacr.org/2021/1465.pdf)

**Finality also needs labels.** Keep separate: an order signature, an execution preconfirmation, a complete unsafe block, a block reconstructed from canonical published inputs, finalized input history, and a state claim accepted for withdrawals. The OP derivation specification explicitly distinguishes input-chain finality from output-claim finalization. The local MegaETH block specification likewise warns that a mini-block signature is not a finality proof. [OP derivation](https://specs.optimism.io/protocol/derivation.html#about-reorgs-post-merge); [MegaETH blocks](../megaeth_specs/specs/protocol/blocks.md#security-considerations)

## 4. A practical enforcement vocabulary

Use these labels when turning a failure into a protocol rule:

- **Reject:** a validator can refuse a bad message using authenticated context and deterministic checks.
- **Prove:** transferable evidence establishes a defined offense. Slashing additionally requires a bond, an adjudicator, and a complete dispute procedure.
- **Recover:** a timeout, alternative provider, or handoff restores service. Recovery need not establish malicious intent.
- **Measure:** telemetry or complaints identify poor service without proving an offense.
- **Govern:** intervention rests on a documented governance judgment rather than a fully objective protocol predicate.

These categories can overlap. In particular, **a timeout suitable for changing leaders is not automatically evidence suitable for slashing**. Local silence may reflect a network partition. Research on accountable liveness shows that blame guarantees depend on the communication model; its specialized theorems do not justify punishing every missed wall-clock deadline in an arbitrary network. [Accountable Liveness](https://arxiv.org/abs/2504.12218)

## 5. Failure catalogue

The following tables are original design analysis. “Evidence” describes what would be needed; it does not assert that an existing MegaETH contract accepts that evidence. For each rule, preserve the protocol version, authority snapshot, canonical-context reference, and signed bytes needed to interpret it.

### 5.1 Ordering and transaction admission

| Failure / responsible actor | Property and useful evidence | Detection, remedy, and enforcement class |
| --- | --- | --- |
| Unauthorized or stale leader signs an order | Authorization; signature plus canonical authority/round state. | Check at every consuming boundary; fence obsolete rounds. **Reject**; **Prove** only if an offense is defined. |
| Leader signs incompatible commitments for the same exclusive position | Consistency; two signed payloads and the rule identifying the same position/context. | Gossip evidence; stop treating the branch as settled; choose one canonical branch through the recovery protocol. **Prove + Recover**. |
| Leader signs malformed bytes, unsupported transaction types, or a structurally impossible list | Input validity; signed bytes and applicable parser/resource rules. | Reject deterministically; retain evidence; replace persistently faulty leader. **Reject**, sometimes **Prove**. |
| Leader orders transactions that are invalid in the resulting sequential state | Validity/progress; committed parent context, list, and execution/admission trace. | Apply the predetermined reject/filter rule. Accountability depends on what an order-only leader promised. **Reject/Recover**; not automatic slashing. |
| Leader omits or misorders required L1-derived inputs | Derivation validity/inclusion; canonical origin data and supplied list. | Independently derive mandatory prefix; reject contradiction. **Reject + Prove** if the signed obligation includes this prefix. |
| Leader excludes an ordinary user indefinitely | Inclusion; admission receipt or publicly ordered queue, eligibility, available capacity, and deadline. | Without those facts, complaints are observations. Add a forced path or receipt-backed rule. **Measure/Recover**; potentially **Prove**. |
| Leader reorders for MEV, sells priority, or exploits private order flow | Fairness/privacy; observed order alone rarely proves a receipt-order policy was violated. | Decide the policy first; consider observable admission, encryption, or fair-order consensus. **Measure/Govern**, or **Prove** under an added rule. |
| Leader chooses an impermissible, stale, or noncanonical L1 origin | Derivation validity/progress; parent origin, referenced L1 headers, timestamps, and canonicality context. | Derivation checks distinguish allowed discretion from rule violation; handle genuine L1 reorg separately. **Reject/Recover**. |
| Users flood admission/execution with invalid requests or expensive failed attempts | Capacity/inclusion; load traces and reproducible validity behavior. | Bound preprocessing and queues; align admission with execution rules; meter costly work. Usually **Measure + Recover**, not a leader offense. |

The order-only boundary makes the fourth row especially important. Suppose Alice has nonce 7 and sends transactions with nonces 7 and 8. Listing nonce 8 first may be inadmissible in that state even though both signatures are authentic. Two individually affordable transactions may also become unaffordable in sequence. Decide whether the leader guarantees admissibility, obtains executor feedback before signing, or commits an input log that a deterministic filter maps to executable transactions.

If filtering is permitted, specify exactly when entries are skipped and how users verify that decision. Otherwise “the executor may remove invalid transactions” becomes an unaccountable censorship interface. If filtering is forbidden, a single inadmissible entry must not create an undefined permanent halt.

**A reverted transaction is different.** A transaction admitted under the protocol may execute and return failure, consume gas, and produce a failure receipt. That is not necessarily an invalid transaction or dishonest execution. MegaETH-specific resource-limit exceptions need their own rules. “Every transaction succeeds” is not an appropriate generic validity requirement.

The danger of cheap rejected work is supported by a concrete study of admission/execution mismatches in other rollups. It reports denial-of-sequencing attacks against then-vulnerable implementations; this does **not** establish a corresponding MegaETH vulnerability. [Li et al., CCS 2025](https://zzzihao-li.github.io/papers/ccs25.pdf)

### 5.2 Execution, preconfirmation, and withholding

| Failure / responsible actor | Property and useful evidence | Detection, remedy, and enforcement class |
| --- | --- | --- |
| Executor inserts, deletes, or reorders authorized entries | Order integrity; sequencer commitment, full input, and executor's authenticated output binding. | Compare against the specified order-to-execution mapping. **Reject + Prove** when evidence is complete. |
| Executor reports wrong receipts, gas, logs, or state | Execution validity; authenticated claim and correct execution evidence. | Independent execution or applicable proof/dispute system. **Reject/Prove**; operator replacement restores service. |
| Executor shows incompatible results to different clients | Consistency; conflicting signed execution claims for the same context. | Cross-client monitoring; suspend the affected assurance level; adjudicate defined conflicts. **Prove + Recover**. |
| Leader reveals only an order root, withholding transaction bytes | Availability/progress; signed root is not evidence that any particular recipient received bytes. | Require retrievable data or acknowledged replication before commitment; use retrieval deadlines for recovery. **Recover**; **Prove** needs an added availability obligation. |
| Executor receives the order but withholds execution results or preconfirmations | Realtime liveness/fair exchange; delivery acknowledgment and promised service deadline, if any. | Alternative executor or explicit failover; avoid blaming the sequencer for the executor's silence. **Measure/Recover**, sometimes **Prove**. |
| Executor streams some mini-blocks but never completes the EVM block | Block and settlement progress; signed prefix plus completion obligation. | Apply an explicit finish-or-abort rule and preserve evidence. **Recover**; blame requires defined responsibility and timing. |
| A provider releases information early to favored searchers | Fairness; corroborated signed release records may help, but private delivery is hard to observe. | Define dissemination obligations; monitor multiple viewpoints. Usually **Measure/Govern**. |

A signed root, delivery to the executor, realtime publication, DA dispersal, L1 publication, and long-term retrievability are different events. Record them separately. A single “published” flag obscures both the failure location and the responsible party.

Ethereum Research's preconfirmation discussions separate inclusion promises from execution promises and highlight withholding and dependence on a delegated preconfirmer. They are design discussions, not proofs about MegaETH. Their practical lesson here is that rotating the orderer cannot repair a permanently unavailable sole executor. [Strawmanning Based Preconfirmations](https://ethresear.ch/t/strawmanning-based-preconfirmations/19695); [Preconfirmation Fair Exchange](https://ethresear.ch/t/preconfirmation-fair-exchange/21891)

### 5.3 Rotation, membership, and recovery

| Failure / responsible actor | Property and useful evidence | Detection, remedy, and enforcement class |
| --- | --- | --- |
| Two controllers elect different replacements after a partition | Safety; conflicting activation certificates and their authority sets. | One verifiable authority rule and compatible quorum/locking rules; local timeout alone must not authorize a competing committed history. **Reject/Recover**. |
| Old leader continues after handoff | Authorization; message round and activation boundary. | All executors, clients, and publishers reject obsolete authority. **Reject**; signing may be **Prove** under explicit rules. |
| New leader begins from an incompatible prefix | Consistency; handoff certificate, previous commitments, and parent context. | Bind activation to a recoverable checkpoint and treatment of outstanding promises. **Reject/Recover**. |
| Old leader withholds handoff state or acknowledgment | Progress; available checkpoint and failed retrieval attempts. | Recovery must work without the failed leader's cooperation; replicate required data beforehand. **Recover**, not necessarily objective blame. |
| Repeated false complaints force leader churn | Progress/accountability; complaints, certificates, and rate history. | Authenticate and bound complaints; require the chosen recovery threshold; reject forged evidence. **Reject/Measure/Recover**. |
| Candidate manipulates membership, selection, or exit | Decentralization/incentives; membership snapshot, randomness/auction transcript, bond state. | Analyze Sybils, correlated ownership, grinding, bribery, and withdrawal delay under the chosen selection mechanism. **Reject/Prove** where rules are objective. |
| Predictable next leaders are attacked, or all backups share a dependency | Progress; operational and network evidence. | Independent infrastructure, prepared backups, and an ultimate fallback. **Measure + Recover**. |

For MegaETH, handoff granularity is constrained by the local rule that every mini-block of an EVM block uses the same authorized key. Switching that key halfway through a block is not a permitted transparent implementation change. Moreover, existing mini-block headers have no parent hash or state root, so an isolated signed header is insufficient to establish a complete recoverable execution prefix. [Sequencing authority](../megaeth_specs/specs/protocol/sequencing.md#sequencer-authority); [mini-block structure](../megaeth_specs/specs/protocol/blocks.md#mini-block-header)

The proposed order-only leader could have a separate rotation schedule, but then the protocol must specify how order rounds map to EVM blocks and the executor's signing authority. A recovery rule must say what happens to already promised transactions when the last block is incomplete. Slashing the departing leader does not make this choice for us.

### 5.4 DA, settlement, and the ultimate fallback

| Failure / responsible actor | Property and useful evidence | Detection, remedy, and enforcement class |
| --- | --- | --- |
| Publisher delays batches or publishes different data | Publication progress/order integrity; batch commitment and accepted execution history. | Compare commitments; allow a defined replacement publisher; preserve derivation rules. **Reject/Recover**, sometimes **Prove**. |
| DA certificate is invalid, or data cannot be retrieved | Derivation/availability; certificate verification and retrieval protocol evidence. | Separate cryptographically invalid certificates from unavailable retrieval; define fallback/timeout semantics. **Reject/Recover**; certificates alone do not settle every availability dispute. |
| State proposer publishes a false claim | State validity; claim and execution proof/dispute evidence. | Apply the actual fault/validity-proof mechanism. **Reject/Prove**. Rotating the orderer is insufficient. |
| Proposer, prover, or challenger stops working | Settlement/withdrawal progress; missing required actions and deadline state. | Replacement permissions, monitoring incentives, funded proof/dispute operation. **Recover**. |
| Attacker delays or censors challenge transactions on L1 | Settlement security/progress; L1 history, remaining dispute time, and transaction-delivery evidence. | Size dispute procedures and budgets for the assumed censorship model. **Recover** may be limited by L1. |
| Entire candidate set or shared execution service fails | Inclusion/exit liveness; no operational fast path. | A specified L1 submission, alternate execution, or escape mechanism with required data and funding. **Recover/Govern**, depending on design. |

A forced-inclusion route and an escape hatch solve different problems: the former supplies an alternative way to get an eligible transaction processed; the latter lets users recover assets or state when normal operation fails. Their usefulness depends on derivation, data, proof, and withdrawal dependencies. The OP specifications describe deposit-derived inclusion; this must not be copied into a MegaETH latency guarantee without verifying its actual configuration and deltas. [OP protocol overview](https://specs.optimism.io/protocol/overview.html); [Ideal Properties of Rollup Escape Hatches](https://dicg-workshop.github.io/2022/papers/gorzny.pdf)

The local [DA](../megaeth_specs/specs/rollup/data-availability.md), [derivation](../megaeth_specs/specs/rollup/derivation.md), [batch-submission](../megaeth_specs/specs/rollup/batch-submission.md), and [fault-proof](../megaeth_specs/specs/rollup/fault-proofs.md) pages are stubs. They identify topics to verify; they do not establish complete failure semantics. In particular, a reference to ZK fault proofs is not enough to infer that every state update requires an upfront validity proof.

## 6. What the literature changes in the design

Three lessons deserve attention before designing rewards:

**Accountability is a separate protocol product.** CFT-Forensics studies cryptographic blame for otherwise crash-tolerant protocols. Its relevance is methodological: retain signed, interpretable evidence instead of assuming operational logs suffice. It does not turn ordinary Raft-style failover into Byzantine-safe consensus. [CFT-Forensics](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.AFT.2024.3)

**Someone must check even when nothing appears wrong.** Proof of Diligence studies incentives for monitoring rollup assertions. The implication for this project is to budget for detection, evidence retention, and challenge submission, separately from rewards for ordering. Economic Censorship Games studies challenge completion under paid L1 censorship, showing why “an honest challenger exists” is an incomplete operational assumption. [Proof of Diligence](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.AFT.2024.5); [Economic Censorship Games](https://arxiv.org/abs/2502.20334)

**Governance rules and objective slashing are different enforcement mechanisms.** Optimism's Law of Chains describes governance intervention against prohibited sequencer behavior. This is useful as a model for discretionary service obligations, but it is not a cryptographic proof that a sequencer received a transaction on time. Decide which guarantees are protocol-enforced and which depend on this kind of judgment. [Law of Chains v0.1](https://gov.optimism.io/t/final-law-of-chains-v0-1/6514/1)

## 7. Zotero search and reading record

The local Zotero library was searched without modifying records or attachments. Searches used top-level item metadata; selected records' abstracts were read when present. Public originals were then checked for the substantive claims above. This is a targeted review, not a full reading of every search result or every PDF.

| Query | Top-level matches |
| --- | ---: |
| sequencer | 2 |
| rollup | 19 |
| fairness | 11 |
| accountability | 1 |
| liveness | 1 |
| MEV | 43 |
| preconfirmation | 5 |
| censorship | 7 |
| rotation | 0 |
| Aequitas | 0 |

Sets overlap. Zero matches mean that search found no matching top-level metadata, not that no relevant attachment exists. The identifiers below are **Zotero item keys, not BibTeX citation keys**. Authors are distinguished from editors/contributors; missing or incomplete local metadata is noted.

| Zotero key | Work; authors; year | Reading scope and relevance |
| --- | --- | --- |
| `KY4KEGPP` | [Attacks on Rollups](https://openreview.net/forum?id=hiE6kZXhu8), Adrian Koegl, Zeeshan Meghji, Donato Pellegrino, Jan Gorzny, Martin Derka; 2023 | Local abstract and public indexed abstract: motivates looking beyond invalid execution to network and L1-linked attacks. Direct PDF access met a browser-verification gate; no full-paper review claimed. |
| `XUIN3GW8` | [Ideal Properties of Rollup Escape Hatches](https://dicg-workshop.github.io/2022/papers/gorzny.pdf), Jan Gorzny, Po-An Lin, Martin Derka; 2022 | Local abstract and public paper: recovery when normal operators disappear. The local record renders the second name as Lin Po-An. |
| `4U44KAC9` | [CFT-Forensics](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.AFT.2024.3), Weizhao Tang, Peiyao Sheng, Ronghao Ni, Pronoy Roy, Xuechao Wang, Giulia Fanti, Pramod Viswanath; 2024 | Local/public abstracts and publication metadata: accountability is distinct from fault tolerance. |
| `QZIUVMLR` | [Accountable Liveness](https://arxiv.org/abs/2504.12218), Andrew Lewis-Pye, Joachim Neu, Tim Roughgarden, Luca Zanolini; 2025 | Local/public abstracts: timing assumptions constrain attributable liveness failures. Local metadata lacks the date and last two authors; public record supplies them. |
| `99LT22K7` | [Proof of Diligence](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.AFT.2024.5), Peiyao Sheng, Ranvir Rana, Senthil Bala, Himanshu Tyagi, Pramod Viswanath; 2024 | Local/public abstracts: incentives to perform monitoring work, including during honest operation. |
| `MDL2XWWC` | [Economic Censorship Games in Fraud Proofs](https://arxiv.org/abs/2502.20334), Ben Berger, Edward W. Felten, Akaki Mamageishvili, Benny Sudakov; 2025 | Local metadata; public abstract: dispute time and budgets interact with censorship. |
| `4Z4VRL8D` | [Themis](https://eprint.iacr.org/2021/1465.pdf), Mahimna Kelkar, Soubhik Deb, Sishan Long, Ari Juels, Sreeram Kannan; CCS 2023, preprint 2021 | Local abstract and public model/definitions: receiver-order fairness requires an explicit observation and consensus model. |
| `Z3Y48XJP` | [FairPoS](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.AFT.2023.10), James Hsin-yu Chiang, Bernardo David, Ittay Eyal, Tiantian Gong; 2023 | Local/public abstracts: input privacy/fairness has permissionless participation and committee assumptions; ordering alone is not the whole MEV problem. |
| `WKHMKDE5` | [Universally Composable Transaction Order Fairness: Refined Definitions and Adaptive Security](https://eprint.iacr.org/2025/1527), Michele Ciampi, Aggelos Kiayias, Yu Shen; 2025 preprint | Local/public abstracts: separates receiver order, input causality, and the role of fees. Local record has no year. |
| `DU2JZK5I` | [Preconfirmation Fair Exchange](https://ethresear.ch/t/preconfirmation-fair-exchange/21891), Conor McMenamin, Lin Oshitani; 2025 | Local abstract and public forum post: timely delivery and payment exchange need their own enforcement. Local record lacks creators. |

Additional Zotero metadata/abstracts screened: `G9GVE4W6` (bounded unfairness), `X3ZAGBL7` (based rollups), `ZC4VF9V6` (atomic execution and arbitrage), `KXU2DQDK` (verifiable dispersal), `VAMTD42I` (Pod), and `XZ27GS6X` (restaked preconfirmation implementation). These were not treated as evidence of current MegaETH behavior. Web-only additions include the CCS 2025 denial-of-sequencing paper and the Ethereum/Optimism forum discussions linked above.

## 8. Decisions needed before incentive design

For every promised property, complete one sentence: **“Actor A must perform action B for eligible input C by condition D; observer E detects failure using evidence F; authority G applies response H.”**

The next design session should settle: whether the sequencer guarantees executable inputs; what an execution preconfirmation promises; how order commitments bind to EVM blocks; which prefix survives a failed handoff; how the executor and publisher can be replaced; which submission path gives enforceable inclusion; and which offenses have objective evidence. Reward and slashing equations should follow those answers, because different responsibilities create different costs, opportunities to deviate, and limits on attributable blame.
