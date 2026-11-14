# Sources and scope for the regional PoS/SMR abstraction in `main_v2`

Prepared 28 September 2026. Source-frozen application context: `formalization/main_v1.tex`. This research note recommends definitions and interface obligations; it does not implement the future cross-region CFT protocol.

## Main recommendation

Specify **an L1-authorized, stake-weighted regional SMR under a fixed finalized configuration**. The L1 selects identities, public keys, weights, regional assignments, and deterministic leader rotation lists. A regional protocol then orders batches of the existing mini-block objects. This is a defensible, narrow PoS construction: stake supplies Sybil resistance and voting authority, while the stated fault bound and synchronous network model supply the consensus theorem.

Do not call this a complete dynamically reconfiguring permissionless PoS system unless validator admission/exit, stake changes, state transfer, epoch agreement, and historical-key security are also implemented. Local leader rotation is not validator-set reconfiguration. Cross-region rotation is a separate outer protocol and is deliberately unfinished here.

## Primary sources

### Standard state machine replication

Fred B. Schneider, **Implementing Fault-Tolerant Services Using the State Machine Approach: A Tutorial**, ACM Computing Surveys 22(4), 299–319, 1990.

- Author-hosted paper: <https://www.cs.cornell.edu/fbs/publications/SMSurvey.pdf>
- DOI: <https://doi.org/10.1145/98163.98167>
- Especially §1, §3, §4, and §7.

Schneider defines commands as deterministic atomic state transitions and coordinates replicas so they process a common request sequence. Identical initial state and command order give identical outputs. His treatment also separates combining replica outputs from coordinating replicas and discusses reconfiguration. This supports defining the regional state machine explicitly, rather than treating a batch-agreement oracle as the entire abstraction.

Recommended formalization for this manuscript (application design):

\[
\mathcal A_i=(\Sigma_i,s_{i,0},\mathcal C_i,\mathcal O_i,\delta_i),
\qquad
\delta_i:\Sigma_i\times\mathcal C_i\to\Sigma_i\times\mathcal O_i.
\]

Each correct replica maintains a committed command log \(L_{i,j}\). Require:

1. Prefix consistency across correct replicas and all times.
2. Local append-only committed logs.
3. Integrity: an authenticated request identifier executes at most once; committed commands are valid for their predecessor state.
4. Deterministic execution and matching responses for the same log prefix.
5. Request liveness during the required fault/network envelope: every continuously admissible request submitted to correct replicas is eventually executed and answered, under the stated admission/fairness bounds.

For a sequential API, completed responses must be returned only after the corresponding command is committed. If linearizability is claimed, also show that an operation invoked after another operation completes cannot be ordered before it. Prefix agreement alone does not prove real-time order. A log append service with replies containing the committed index and with correctly serialized requests is a simple starting point.

**Important v1 difference:** growing the chain using empty mini-blocks while making no guarantee about submitted transaction selection proves chain growth, not standard per-request SMR liveness. A clean specification distinguishes admission to the sequencer's transaction pool from submission of an admitted command to the regional SMR. State exactly which commands get liveness and what fair selection rule supplies it. An honest leader may not starve an admitted command forever while the theorem still claims standard request liveness.

### The generic PoS compiler the user likely remembers

Jovan Komatovic, Andrew Lewis-Pye, Joachim Neu, Tim Roughgarden, Ertem Nusret Tas, **From Permissioned to Proof-of-Stake Consensus**, AFT 2025, LIPIcs 354, 18:1–18:26.

- Published article and BibTeX: <https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.AFT.2025.18>
- Published PDF: <https://drops.dagstuhl.de/storage/00lipics/lipics-vol354-aft2025/LIPIcs.AFT.2025.18/LIPIcs.AFT.2025.18.pdf>
- Full version: <https://arxiv.org/abs/2506.14124>

The paper gives a generic transformation from permissioned consensus to quasi-permissionless PoS. Its preservation statements concern properties satisfied by the input protocol **under partial synchrony**. It preserves consistency, liveness, optimistic responsiveness, composable log-specific safety properties, and message-complexity order; its added certification gives accountability. The transformation runs permissioned instances in epochs, certifies logs, and uses agreed epoch-ending transactions. Validators wait long enough before issuing those transactions to preserve an overlap interval for request liveness. The next committee derives from the completed epoch's log. These are substantive mechanisms, not a relabeling of votes as stake. See §1.3, §4, and Theorem 12.

**Application inference:** the theorem does not immediately establish properties of a synchrony-dependent Sync HotStuff instantiation in the partially synchronous model. Cite it as the systematic general framework and explain that this draft takes the simpler fixed, externally finalized L1-configuration case. Do not claim to instantiate the complete compiler without implementing and proving its epoch mechanism.

### A directly relevant external-L1 stake-table design

Espresso Systems, **The Espresso Sequencer: HotShot Consensus and Tiramisu Data Availability**, 2023.

- Project-hosted paper: <https://github.com/EspressoSystems/HotShot/blob/main/docs/espresso-sequencer-paper.pdf>
- Later paper: **The Espresso Sequencing Network: HotShot Consensus, Tiramisu Data-Availability, and Builder-Exchange**, <https://eprint.iacr.org/2024/1189>.
- Current primary implementation documentation for stake-table interfaces: <https://hotshot.docs.espressosys.com/src/hotshot_types/traits/stake_table.rs.html>.

The 2023 manuscript §2.4 explicitly discusses extending HotStuff to PoS with a stake table maintained on L1, epoch snapshots, reconfiguration, and verifiable weighted certificates. This is an architectural precedent for external L1 stake configuration. It does not make their specific epoch mechanism, DA model, randomness beacon, or long-range argument part of this draft automatically. The later paper distinguishes consensus ordering from execution and identifies stake tables/reconfiguration as additional mechanisms around the consensus core. Use this source to justify the architecture, not to claim this draft is HotShot or inherits its whole security analysis.

### Existing implementation description separating BFT and stake

- Tendermint official validator documentation: <https://docs.tendermint.com/v0.34/tendermint-core/validators.html>.
- Tendermint official protocol overview: <https://docs.tendermint.com/master/spec/>.

The validator documentation explicitly separates consensus from the external process that assigns membership and voting power; collateral may be on-chain or off-chain. The overview describes hash-linked transaction batches committed by weighted validators. This is useful supporting terminology: an engine becomes stake-based through its membership/weighting layer, not because every BFT protocol is intrinsically PoS.

### Sync HotStuff and a warning requiring exact variant selection

- Ittai Abraham, Dahlia Malkhi, Kartik Nayak, Ling Ren, Maofan Yin, **Sync HotStuff: Simple and Practical Synchronous State Machine Replication**, IEEE S&P 2020: <https://eprint.iacr.org/2019/270>.
- Authors' tutorial: <https://decentralizedthoughts.github.io/2019-11-12-Sync-HotStuff/>.
- Jérémie Decouchant, Burcu Kulahcioglu Ozkan, Yanzhuo Zhou, **Liveness Checking of the HotStuff Protocol Family**, PRDC 2023: <https://arxiv.org/abs/2310.09006>.

The tutorial presents the synchronous core with proposal forwarding, votes, a nonblocking \(2\Delta\) commitment timer, and separate view change; the paper also considers mobile-sluggish faults and an optimistic variant. The PRDC article reports a forcelocking failure for an **early** Sync HotStuff ePrint version (20190312:115828); this must not be described as a refutation of the final S&P 2020 text. The manuscript must identify the exact variant and explicitly define its view-change timing, status collection, tie resolution, and recovery. A citation alone is not a correctness proof for a modified weighted protocol.

## Stake configuration and certificate contract for v2

The following are design recommendations from the application context, not quoted compiler guarantees.

Use a single L1-finalized configuration reference \(c\), including the L1 chain identifier, anchor block/hash or equivalent finality reference, region assignments \(V_i\), registered signing keys, integer stake amounts \(\omega_{i,j}>0\), the local cyclic lists, and the future region rotation list. Each local list should enumerate each configured validator once unless weighted scheduling is explicitly desired. Voting power and leader frequency are separate design choices: retaining v1's once-per-member rotation is compatible with stake-weighted voting but does not imply stake-proportional proposer selection.

All protocol messages bind their message type, configuration, global activation generation, region, local view, and appropriate batch/log index. Quorum verification counts each configured signing identity once against the **same fixed snapshot**. For v1-compatible thresholds, \(\operatorname{wt}(Q)>2W_i/3\) and Byzantine weight \(<W_i/3\) give the usual honest intersection. If the chosen Sync HotStuff variant instead uses a different threshold, its weighted proof must be shown explicitly; do not preserve only the numeric threshold while changing the assumptions underneath it.

Evidence submission must not silently subtract stake from \(W_i\) mid-view. Freeze voting power for the current configuration. Penalties may be adjudicated against the L1 bond; a new validator set or weight map requires a separately authorized reconfiguration. The L1 configuration source resolves membership agreement, not automatic state transfer or long-range authentication. Retain the v1 historical-honest-key assumption or specify another mechanism; slashing alone does not prevent forged old histories after exits/key compromise.

## Retaining v1 slash conditions correctly

The actual v1 rules are in `main_v1.tex`, §“Equivocation and invalid inputs” (approximately lines 952–979). The draft defines objective conflict evidence but leaves penalty amounts and adjudication mechanisms outside the service. It does **not** already implement an L1 slashing contract.

| v1 evidence | Recommended v2 treatment |
|---|---|
| ORDER: same \((c,v_G,u,h)\), different mini-block header digests | Preserve for an actual sign-once batch identity. If \(u\) becomes a revisable log height across local views, either include \(v_L\) in the comparison key or retain a distinct proposal-attempt identifier. Never punish an honest legitimate re-proposal merely because a tentative branch changed. |
| PROPOSE / VOTE: same instance \(I\), different proposal digests | Rebind the key to the Sync HotStuff instance: same configuration, region/global generation, local view, and proposal slot. Different local views may legitimately vote differently while preserving the safe committed prefix. |
| DECIDED: same \(I\), different outcomes, including different parents | Preserve as incompatible committed-output evidence. The comparison domain is the immutable regional log position/generation. Distinct certificates or signer sets on the same semantic output are not conflicts. |
| REPORT: same checkpoint-report domain, different semantic reports | Historical v1 condition only. The outer CFT layer is unfinished, so v2 should not invent a live REPORT protocol to keep this offense active. |
| FFG: double target or surround vote | Historical v1 condition only. CFT outer rotation has not defined FFG voting and cannot inherit these offenses automatically. |

Additional same-view chain-fork evidence is possible if the implemented voting rule explicitly forbids a signer from endorsing incomparable branches. Specify the exact signed pair and ancestor witnesses needed to check incomparability before adding that predicate. A verifier must distinguish a new signature over the same semantic object from a conflicting statement.

Do not slash for a local timeout, unavailable DATA response, a legitimate view change, or a recovery relay of another node's signed conflicting messages. Silence is not positive cryptographic evidence. A claim that every safety failure identifies a particular stake fraction is a separate accountable-safety theorem and must not follow merely from listing double-sign predicates.

## Making one region look like one non-Byzantine process

This is an interface proof obligation, not an automatic consequence of a region producing a consistent block chain.

1. Let the inner SMR execute the future outer algorithm's **state transitions**, including term changes, election votes, replicated-log writes, acknowledgments, outbox updates, and input deduplication.
2. Treat external messages and timeout events as commands to the regional state machine, so all correct replicas order and apply them consistently. Local nondeterministic timers must not directly cause different outer votes.
3. Release an outer message only with a regional certificate over its exact destination, outer instance/term, message type and payload, tied to the committed transition/output identifier. A local leader's signature alone is insufficient.
4. Give every emitted message a deterministic identifier. Receivers accept duplicates at most once; replaying the same message is permitted and supports recovery.
5. Persist committed log/state, last signed votes, and outer outbox state across local crashes. Whole-region crash recovery must recover the same prefix before producing new outputs.
6. Describe the failure envelope precisely. Whole-region network isolation can yield delay/omission; an arbitrary breakdown of the inner synchrony or stake assumption does not automatically yield a crash-only outer node. With timing-dependent inner safety, this is especially significant.

Within these conditions, every accepted output corresponds to one deterministic transition in one compatible regional log. Delayed or omitted outputs then present a crash/recovery/omission interface suitable for an outer CFT design. The future CFT algorithm must still supply unique active-region authority, cross-region log replication, commit/leader election rules, data retention, fencing and safe handoff. Regional commit alone does not prove global irreversible confirmation for proposer \(P\).

## Suggested bibliography keys

- `schneider1990smr`: standard deterministic SMR definition and replicated output.
- `synchotstuff2020`: actual regional consensus source (state exact variant).
- `permissioned2pos2025`: general compiler and its explicit model boundary.
- `espresso2023`: external L1 stake-table architecture.
- `hotstuffliveness2023`: reason to make pacemaker/view-change details explicit.

For a lean manuscript, the first four suffice in the main exposition, with the liveness-checking source attached to an implementation/variant note. v1 should remain unchanged.
