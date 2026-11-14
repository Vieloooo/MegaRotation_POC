# Protocol-design advisory: local BFT and global rotation

This is a design stress test, not a verification of a finished protocol. It uses the current `outline.md`, the September 16/22 meeting requirements, and the parent task's chosen direction. The user has confirmed that S supplies ordered candidate transactions and need not execute; P executes. S may execute privately to optimize its ordering.

## Recommended conservative scope

Use a recognized, fully specified local BFT state machine for ordered input records, and a much slower, serialized global BFT state machine for checkpoint and authorization records. A Casper-style two-link finalization condition may sit on the unique global record chain as the settlement-eligibility condition. This is a deliberately conservative hybrid; it is not a proof that local LMD-GHOST plus global Casper inherits Ethereum/Gasper guarantees.

The fast path need not wait for a global quorum per mini-block. Consequently, its strongest immediate certificate is **local confirmation**. Preserve every locally committed input during an ordinary local leader rotation. Permit rollback of the local-only suffix after a whole-region failure, while preserving every globally ordered input checkpoint. P's execution of that suffix is speculative; irreversible publication must be limited to the eligible globally finalized input prefix, with execution verification handled separately.

Global BFT decision and FFG finalization are different predicates. In this hybrid, a global BFT decision is already irreversible under the global quorum assumptions. Therefore a geographical handoff must extend the latest globally **ordered** checkpoint, including an ordered checkpoint not yet FFG-finalized. The FFG lag can gate publication; it cannot justify forgetting an already ordered global decision.

## Unavoidable regional-failure tradeoff

Consider two executions indistinguishable to the backup regions. In one, the active region issued no new local certificate. In the other, it issued a certificate and disclosed its data only within that region or to a faulty P. The whole region then becomes unreachable. Backups have the same information in both executions. They cannot both guarantee bounded takeover and reconstruct/preserve an arbitrary undisclosed committed extension.

Thus an unconditional combination of (i) local-RTT irrevocability, (ii) no outside durability condition before irrevocability, and (iii) bounded recovery after all local storage becomes inaccessible is unavailable. The honest options are:

1. Treat local confirmation as provisional across geographical failure, as recommended above.
2. Preserve local confirmation unconditionally but wait for its data/certificates to become recoverable; geographical liveness is conditional.
3. Require external archival acknowledgments before a prefix becomes region-failure-stable. This need not be global *consensus* per block, but it is an external durability/latency condition. An archive threshold must imply that at least one surviving honest archive stores both bytes and certificate, and the takeover must collect its report before choosing a base.

Do not silently claim option 1's latency together with option 3's permanence.

## Local safety and view change

### Do not use the outline's bare two-round rule

Voting for a proposal only when it extends one's latest committed tip does not specify safe recovery from partial voting. A leader can distribute different candidates, different honest voters can see different certificate subsets, and replacing a missing proposal by locally constructed empty blocks can split votes. A new leader also may not know a certificate held by another honest node. Quorum intersection alone does not supply the missing view-change protocol.

### Minimal PBFT-style construction

For each log height and local view, distinguish `PROPOSE`, `PREPARE`, and `COMMIT`. A prepare certificate contains more than two thirds of the region's weight. An honest member sends COMMIT only after obtaining a valid prepare certificate and storing the complete proposal bytes. A local commitment certificate contains more than two thirds COMMIT weight. Messages bind the chain/domain, membership version, geographical authorization generation, local view, height, parent commitment, and proposed record hash.

For a view change, members report their last committed prefix and highest prepared certificate for every unresolved log position in the permitted pipeline window. A NEW-VIEW proof contains a regional quorum of these reports. The new leader deterministically preserves committed positions and selects the highest-view prepared value for each unresolved position, following the full PBFT selection rule. Recipients validate the report set and selected values before voting. A mere quorum of highest committed tips is insufficient: a commitment can be hidden while honest commit signers still retain its prepared certificate.

If presentation space is tight, instantiate a named, specified BFT replicated-log primitive and state its input/output and view-change contracts rather than writing a partial replacement algorithm. A single-height explanation is sound only if the actual protocol either serializes heights or imports the full multi-position rule. A pipeline figure must not imply unproved cross-height pipelining.

### Empty records and timeout evidence

An empty record must have a canonical digest and the same consensus/parent-selection rules as a nonempty record. Do not let each member invent an empty parent from its private tip. Its production can demonstrate a reachable quorum and absence of an accepted leader proposal, but does not objectively demonstrate malicious behavior.

The simplest liveness path is a quorum timeout certificate triggering a local view change; an empty record need not be committed first. If retaining the outline's consecutive-empty-block trigger, define exactly which global slot IDs are counted, require canonical committed empty records, and explain how members agree on each empty record despite a silent leader. Usually that already needs the timeout/view-change primitive. Presenting empty voting as a free substitute for leader recovery would be circular.

## Global records, repeated roots, and finalization

Use a globally unique record/checkpoint identity such as

\[
 R_e=(\mathsf{domain},m,e,H(R_{e-1}),g_e,x_e,\mathsf{control}_e),
 \qquad C_e=(e,H(R_e)),
\]

where `m` is the membership version, `g_e` the authorization generation and `x_e` the ordered-input prefix commitment. Adjacent records may have the same `x_e`, while their epoch and record hash remain distinct. Define a no-progress epoch by `x_e=x_{e-1}`, rather than literally equating checkpoint identities.

If the protocol implements Casper source/target votes, bind both source and target epoch plus checkpoint roots. A voter must not issue two differing votes for the same target epoch, and must not issue strictly surrounding source/target pairs. A same-input-root link from epoch `e-1` to epoch `e` is a real inter-epoch vote, not a vote from one checkpoint to itself. Monotonically increasing epochs remain necessary during empty service intervals.

On a consecutive unique record chain, two quorum links `C_(e-1) -> C_e` and `C_e -> C_(e+1)` can define finalization of `C_e`. This is a protocol definition plus inherited global-record safety; it must not be sold as an independent full Gasper theorem. If `C_e` is a unique PBFT output, FFG contributes a timing/settlement policy rather than a second, necessary ordering safety argument.

Two no-progress epochs and two-link finalization are also different conditions. State the exact indices: e.g. no progress in epochs `e-1` and `e` means `x_(e-2)=x_(e-1)=x_e`. If the rotation instruction is first written into `R_e` and activation requires `C_e` finalization, activation awaits the next link. This extra epoch must appear in recovery bounds and diagrams.

## Global validity, data, and control-plane liveness

A global checkpoint is valid only if it extends the latest globally ordered prefix, binds a valid local certificate under the applicable authorization, and has recoverable complete input bytes. Honest global voters should obtain and retain the complete newly checkpointed input before approving the record, or validate an explicit DA condition that guarantees later retrieval. Signing an unavailable hash is not sufficient for recoverable handoff. The retention period and release condition must be stated.

Membership and voting weights must be frozen for a specified configuration period. Global Byzantine weight below one third does not imply regional Byzantine weight below one third. Every region eligible for the fast path needs its own local fault assumption. Also, after the whole active region becomes unavailable, global liveness requires the **union** of faulty and unavailable voting weight to remain below one third. Summing overlapping faulty/unavailable sets double counts; a union condition is cleanest.

A subjective satisfaction oracle can be an abstract policy input, but honest nodes evaluating different private histories may split votes forever. To claim global liveness, global BFT must decide among deterministically valid records, with a valid no-progress fallback available to an honest leader. A policy can choose whether to propose a progress checkpoint or a health/rotation action; it must not cause honest validators to reject all otherwise valid decisions merely because they preferred a different service score. If satisfaction disagreement is permitted to prevent voting, state liveness conditionally on eventual policy agreement.

## Handoff and fencing

Every geographical handoff needs one globally ordered control record binding the old and new generation, new region, inherited globally ordered input prefix, activation boundary, and the allowed disposition of the local-only suffix. Start the new region from this exact certified prefix. Advance the authorization generation even if the same geographical region later regains responsibility.

Include the generation in every local vote and certificate. A delayed old-generation certificate may authenticate an old tentative suffix; it cannot extend the newly authorized stream or override the handoff. An observer that has received the control decision rejects old-generation input beyond the prescribed boundary.

If the document promises that **all** honest consumers reject the old producer immediately, add a common lease/boundary rule and a dissemination allowance under the synchrony/clock model. An emergency certificate unknown to an isolated consumer cannot instantaneously revoke that consumer's information. The simpler safe claim is that globally accepted history follows unique authorization records and temporary local observations remain provisional. Epoch-boundary authorizations or hard leases are natural ways to bound stale local production without a per-block L1 transaction.

Switching ordering leaders does not inherently switch the native MegaETH mini-block signer. If a native signer changes, preserve the existing one-authorized-key-per-EVM-block boundary. The order-log handoff must additionally record the execution/publication cursor and any open EVM block context; input prefix identity alone does not describe P's pending execution duties.

## Order-only S / executing P

Call S's output an ordered candidate-input record or an ordering mini-block, and reserve native MegaETH mini-block for the executed object with receipt commitment. This is an explicit new interface layered before the native format.

Specify deterministic execution/filtering as a function of prior execution state, ordered candidate bytes, and fixed execution context. It returns the native executed transaction subsequence, receipts/new state, and a rejection trace binding candidate positions and deterministic reasons. P cannot replace invalidity with unrestricted discretionary omission. Rejection-trace correctness requires execution verification; local ordering voters need not execute, so their signatures do not prove that correctness.

Separate (a) progress of the ordered candidate log, (b) useful/executable transaction progress, and (c) publication progress. A malicious S could produce endless syntactically valid but non-executable candidate lists. Ordering liveness by itself does not promise execution liveness or censorship resistance. The health rule must identify which progress measure it observes. Likewise P's refusal to execute must not automatically count as regional sequencing failure.

## Claims supportable at this stage

* **Local consistency and continuity:** under regional quorum assumptions, the chosen BFT primitive preserves every locally committed record across local views in one authorization generation.
* **Global prefix safety:** the serialized global BFT record chain never orders conflicting input prefixes, and handoff extends that chain.
* **Conditional ordering liveness:** with available payloads, synchrony, an eventual honest eligible leader, sufficient responsive regional/global weight as applicable, and a live global validity/policy rule, the respective state machine progresses.
* **Qualified disaster recovery:** geographical failure can discard a local-only suffix; globally ordered records and their data survive under the stated global retention/DA assumption.
* **Conditional publication:** requires a live/replaced P plus DA/L1 progress and independently valid execution. Rotation of S alone cannot repair a faulty P.

Do not claim transaction inclusion, unconditional region-disaster losslessness, execution correctness from sequencing votes, automatic native MegaETH compatibility, a proven modified-Gasper protocol, or exact latency numbers without the corresponding assumptions and proof.

## Presentation checks for the eventual draft

Use separate symbols for mini-block slot, local view/leader term, global checkpoint epoch, membership period, and authorization generation. Put local and geographical rotations into separate transition tables. The pipeline figure should show speculative local delivery, global checkpoint/availability, finalization eligibility, P execution, and DA/L1 publication as distinct events. Label the rollback boundary at the latest globally ordered prefix, and the publication boundary at the finalized execution-eligible prefix.

The main technical change from the outline should be stated early: a specified local BFT primitive replaces the informal two-round/Gasper combination, and global record serialization supplies a conservative control plane. LMD-GHOST can be omitted; if retained, restrict it to a clearly speculative proposal-head heuristic with no independent finality guarantee.
