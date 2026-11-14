---
description: Derivation pipeline deltas — the changes block certification requires of the OP Stack derivation pipeline, and why each one is required.
spec: TBD
---

# Derivation Pipeline Deltas

{% hint style="warning" %}
The behavior summarized on this page is unstable.
Block certification is not activated on any MegaETH network.
It summarizes Δ1 and Δ3–Δ9, whose normative rules live on the [derivation page](../rollup/derivation.md) and [activation page](activation.md); Δ2 is folded into Δ1.
{% endhint %}

This page summarizes the derivation changes required by block certification and the rationale for assigning each check to its protocol role.

## Motivation

Block certification fixes what each L2 height's content is, but it cannot by itself make that content settle.
Whether a height settles as its selected record's content or as a [deposit-only block](https://specs.optimism.io/protocol/derivation.html#batch-queue) is decided entirely by [derivation](../rollup/derivation.md), and for several of derivation's conditions a certifier attestation cannot pre-empt that decision, because the certifier holds no L1 data and performs no execution.
The deltas below identify the change, pipeline stage, rationale, trade-off, and consensus impact.

[Δ1](#delta-1-batch-transaction-selection-keys-on-the-acceptance-event) is already specified normatively on [the derivation page](../rollup/derivation.md#batcher-transaction-selection), and is summarized here rather than restated; Δ2, once a separate receipt-status rule, is folded into it.
Δ3–Δ6 and Δ8–Δ9 are likewise specified normatively on the derivation page and summarized here; Δ7 is specified on the dedicated [activation page](activation.md).
[The Two Kinds of Failure](#the-two-kinds-of-failure) and [The Governing Principle](#the-governing-principle) below are the shared justification all the deltas rest on, and are stated once, before the deltas themselves.

## Summary of Deltas

| Delta                                                                   | Pipeline stage                                                                                                                                                                         | Upstream behavior                                                                                                                                                               | MegaETH behavior                                                                                                                                                                                                             |
| ----------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [Δ1](#delta-1-batch-transaction-selection-keys-on-the-acceptance-event) | Batcher transaction selection (L1 retrieval)                                                                                                                                           | Sender must equal the single `batcherAddr` in `SystemConfig`; receipt status is not consulted                                                                                   | Selection requires the fixed inbox recipient, an allowed outer transaction type, a successful receipt and the inbox's `CursorAdvanced` event; the inbox authorizes the sender                                                |
| Δ2                                                                      | —                                                                                                                                                                                      | —                                                                                                                                                                               | Folded into [Δ1](#delta-1-batch-transaction-selection-keys-on-the-acceptance-event)                                                                                                                                          |
| [Δ3](#delta-3-remove-the-parent_hash-check)                             | [Batch queue](https://specs.optimism.io/protocol/derivation.html#batch-queue) and [attributes queue](https://specs.optimism.io/protocol/derivation.html#payload-attributes-derivation) | a `parent_hash` mismatch drops the batch (batch queue) or resets the pipeline (attributes queue)                                                                                | Neither site checks `parent_hash`; the field is not read at all. The timestamp reset is retained                                                                                                                             |
| [Δ4](#delta-4-resolve-the-l1-origin-from-epoch_num)                     | Batch queue and payload-attributes construction                                                                                                                                        | `epoch_hash` is compared against the origin, and the origin is then fetched by hash to scan for deposits                                                                        | `epoch_hash` is not compared; the origin is resolved from `epoch_num` against the canonical L1 chain the pipeline already holds                                                                                              |
| [Δ5](#delta-5-substitute-instead-of-dropping)                           | Batch queue                                                                                                                                                                            | Timestamp-against-origin, sequencer-drift and origin-continuity violations drop the batch, and the height fills with a deposit-only block only after the full sequencing window | All four conditions yield an immediate same-height deposit-only substitution, the substituted block being the one [the deposit-only fallback](https://specs.optimism.io/protocol/derivation.html#batch-queue) would generate |
| [Δ6](#delta-6-keep-the-channel-after-an-engine-rejection)               | Engine result handling                                                                                                                                                                 | An `INVALID` engine result substitutes a deposit-only block and discards the rest of the channel                                                                                | The deposit-only block is substituted and the rest of the channel is processed in order against it                                                                                                                           |
| [Δ7](#delta-7-activation-boundary)                                      | L1 retrieval, channel assembly and batch release                                                                                                                                       | One transaction protocol, batches released as decoded                                                                                                                           | Switch at the canonical bootstrap position, reject channels crossing the checkpoint, fill missing legacy heights immediately, and discard legacy assembly state                                                              |
| [Δ8](#delta-8-emergency-cursor-overrides)                               | Ordered L1 control inputs and deposit-only construction                                                                                                                                | Missing heights wait for sequencing-window expiry; no certified-cursor override                                                                                                 | Process `CursorSet` in order, reset channel state, and fill missing heights through the supplied target while preserving the safe prefix                                                                                     |
| [Δ9](#delta-9-validate-the-complete-compressed-stream)                  | Channel reader and batch release                                                                                                                                                       | Batches decoded from a streaming decompressor; a later error preserves the earlier prefix and the decompressed-byte limit clips available input                                 | Validate the first complete compressed stream before releasing batches; stream failure or excess decompressed output yields no blocks                                                                                        |

## The Two Kinds of Failure

Derivation refuses a batch in two structurally different ways, and the distinction is what every decision on this page turns on.

**Batch validity failure.** The batch is dropped without deriving anything from it, and the pipeline goes on waiting for a batch at that height, filling it with a deposit-only block only once the sequencing window elapses.
Because a later record cannot replace the record selected by the current scan, no corrected batch can be built for that height without an override, so waiting alone does not repair the mismatch.
This is the expensive failure: it costs a full sequencing window of stalled derivation before it costs the height.

**Engine rejection.** The batch is accepted, the engine executes it and returns `INVALID`, and derivation immediately substitutes a deposit-only block at the same height.
Upstream this also discards the rest of the channel the batch came from, so the heights that channel still held wait out a sequencing window.
[Δ6](#delta-6-keep-the-channel-after-an-engine-rejection) removes that automatic discard; subsequent batches are still evaluated against the replacement chain and may fail their own checks.

A condition worth pre-empting is therefore one that would otherwise cause the first kind.
A condition that causes the second needs no handling at all.

## The Governing Principle

> Pre-empt a condition if and only if the party controlling the offending value can still produce a corrected submission.

Values fixed by the selected record — the transaction list, `epoch_num`, and the timestamp through the height bijection — cannot be changed by a replacement batch under the same scan.
Checking them before selection is fixed, at [Record Validity](block-records.md#record-validity), leaves the height claimable by a corrected record.
Checking them afterwards, in the [certifier core](certifier.md#advancing-the-cursor) or at the [inbox](batch-inbox.md#acceptance), turns a height that would have settled deposit-only into a cursor that can never pass it — a permanent halt recoverable only by [emergency override](batch-inbox.md#emergency-cursor-override).
For these values, checking late is strictly worse than not checking at all.

Values the batcher chooses per submission are correctable, so pre-empting them is sound.
The better move is to stop derivation reading them, after which there is nothing to pre-empt.
Where derivation must keep reading such a value — the L1 block that includes each frame, which the [channel timeout](https://specs.optimism.io/protocol/derivation.html#channel-bank) bounds — the [inbox](batch-inbox.md#acceptance) pre-empts the condition instead, refusing a frame derivation would discard while the content is still resubmittable.

The table below lists every derivation condition this section's principle is applied to, where derivation checks it, what happens upstream when it fails, what happens under certification, and which party pre-empts it, with links to the rule that decides each row.

| Condition                                                                  | Where derivation checks it    | Upstream outcome                                                                              | Under certification                                                                                   | Pre-empted by                                                                                                                   |
| -------------------------------------------------------------------------- | ----------------------------- | --------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| Empty transaction entry                                                    | Batch queue                   | Batch dropped                                                                                 | Never reaches derivation                                                                              | [Record Validity](block-records.md#record-validity) content conditions                                                          |
| Deposit-type transaction entry                                             | Batch queue                   | Batch dropped                                                                                 | Never reaches derivation                                                                              | [Record Validity](block-records.md#record-validity) content conditions                                                          |
| Non-empty Interop activation block                                         | Batch queue                   | Batch dropped                                                                                 | Never reaches derivation                                                                              | [Record Validity](block-records.md#record-validity) content conditions                                                          |
| `parent_hash` mismatch                                                     | Batch queue; attributes queue | Batch dropped during validation; pipeline reset if a mismatch reaches attributes construction | Field not read, [Δ3](#delta-3-remove-the-parent_hash-check)                                           | Nothing needed                                                                                                                  |
| `epoch_hash` mismatch                                                      | Batch queue                   | Batch dropped                                                                                 | Field not read; origin resolved from `epoch_num`, [Δ4](#delta-4-resolve-the-l1-origin-from-epoch_num) | Nothing needed                                                                                                                  |
| Timestamp before the origin's timestamp                                    | Batch queue                   | Batch dropped; deposit-only fallback after the sequencing window                              | Same-height substitution ([Δ5](#delta-5-substitute-instead-of-dropping))                              | Nobody; fixed by the selected record                                                                                            |
| Sequencer drift exceeded with user transactions                            | Batch queue                   | Batch dropped; deposit-only fallback after the sequencing window                              | Same-height substitution ([Δ5](#delta-5-substitute-instead-of-dropping))                              | Nobody; fixed by the selected record                                                                                            |
| Empty batch exceeds drift and unnecessarily retains its origin             | Batch queue                   | Batch dropped; deposit-only fallback after the sequencing window                              | Same-height substitution ([Δ5](#delta-5-substitute-instead-of-dropping))                              | Nobody; fixed by the selected record                                                                                            |
| `epoch_num` below the derived parent's origin                              | Batch queue                   | Batch dropped; deposit-only fallback after the sequencing window                              | Same-height substitution ([Δ5](#delta-5-substitute-instead-of-dropping))                              | Nobody; fixed by the selected record                                                                                            |
| `epoch_num` more than one above the derived parent's origin                | Batch queue                   | Batch dropped; deposit-only fallback after the sequencing window                              | Same-height substitution ([Δ5](#delta-5-substitute-instead-of-dropping))                              | Nobody; fixed by the selected record                                                                                            |
| Origin older than `seq_window_size` L1 blocks before the including block   | Batch queue                   | Batch dropped; height filled by the deposit-only fallback                                     | Unchanged; not converted to same-height substitution ([Δ5](#delta-5-substitute-instead-of-dropping))  | Nobody, deliberately; [not bounded at the inbox](batch-inbox.md#rationale)                                                      |
| Channel open past `CHANNEL_TIMEOUT`                                        | Channel bank                  | Channel dropped; its later frames dropped                                                     | Unchanged                                                                                             | The inbox refuses the late continuation, [Acceptance](batch-inbox.md#acceptance)                                                |
| Channel raw size over `MAX_RLP_BYTES_PER_CHANNEL`                          | Channel bank                  | Channel dropped                                                                               | Unchanged                                                                                             | The certifier core fails the call, [step 2](certifier.md#verification-procedure)                                                |
| Decompressed bytes over `MAX_RLP_BYTES_PER_CHANNEL`                        | Channel reader                | Channel clipped to its first `MAX_RLP_BYTES_PER_CHANNEL` bytes                                | Entire channel yields no batches, [Δ9](#delta-9-validate-the-complete-compressed-stream)              | The core attests a vacuous close without advancing records, [Closed-Channel Decoding](certifier.md#closed-channel-decoding)     |
| Closed compressed stream is malformed, truncated or fails integrity checks | Channel reader                | Previously decoded batches may survive a later read error                                     | Entire channel yields no batches, [Δ9](#delta-9-validate-the-complete-compressed-stream)              | The core validates the complete stream before comparing blocks, [Closed-Channel Decoding](certifier.md#closed-channel-decoding) |
| Non-singular (span) batch version                                          | Batch decoding                | Decoded as a span batch since Delta                                                           | Singular batches only, [`CHANNEL_PARAMS`](certifier.md#constants)                                     | The certifier core fails the call, [`decode_complete_blocks`](certifier.md#verification-procedure)                              |
| Engine `INVALID`                                                           | Engine result handling        | Deposit-only replacement; rest of the channel dropped                                         | Deposit-only replacement; channel kept, [Δ6](#delta-6-keep-the-channel-after-an-engine-rejection)     | Nobody; a full node can [anticipate it by executing](full-node.md#surface-what-it-has-determined)                               |
| Channel straddling the activation checkpoint                               | Batch release                 | No such condition upstream                                                                    | Whole channel rejected, [Activation and Channel Boundary](activation.md#derivation-transition)        | The batcher, under [Activation and Channel Construction](activation.md#channel-boundary)                                        |
| Out-of-order, gapped or duplicate frames                                   | Frame queue; channel bank     | Frame dropped; an open channel is dropped with it                                             | Unchanged                                                                                             | The certifier core requires contiguous numbering from zero, [step 2](certifier.md#verification-procedure)                       |

## Delta 1: Batch-Transaction Selection Keys on the Acceptance Event

**What changes.** The baseline's sender condition — sender equals the single `batcherAddr` in `SystemConfig` — is replaced by an acceptance condition, while selection still requires the fixed inbox recipient and a permitted outer transaction type.
The [certified selection rules](../rollup/derivation.md#batcher-transaction-selection) explicitly admit type 4, exclude blob-carrying type 3, and require a successful receipt carrying the inbox's [`CursorAdvanced`](batch-inbox.md#events) event.
Sender authorization and the direct, code-empty EOA submission requirement are enforced by the [inbox](batch-inbox.md#acceptance).
These selection changes apply under the [certified rules](activation.md#derivation-transition); under the legacy rules, transactions retain their pre-certification selection and can supply only heights at or below the checkpoint.
Under the certified rules, the sender is not evaluated, the [batcher registry](batch-inbox.md#batcher-registry) is inbox state that only the contract reads, and `SystemConfig`'s batcher fields are unchanged.
Blob-carrying (type 3) transactions are removed from the permitted types, and the bytes ingested are the transaction's calldata alone, because the calldata is what the inbox checked.
Type `4` ([EIP-7702](https://eips.ethereum.org/EIPS/eip-7702)) transactions, absent from the baseline's list, are admitted, for the reason [Batcher Transaction Selection](../rollup/derivation.md#batcher-transaction-selection) gives: a code-empty registered sender may submit with one, and derivation must not discard a submission the inbox accepted.
This delta is specified normatively under [Batcher Transaction Selection](../rollup/derivation.md#batcher-transaction-selection).

**Where in the pipeline.** Batcher transaction selection, in L1 retrieval, before channel assembly.

**Why it is required.** Two reasons, one per assumption the baseline makes.
Certification's recovery path needs more than one permitted sender: the [exclusive window](batch-inbox.md#exclusive-window) admits an alternative batcher, and a pipeline filtering on one configured address would ignore every batch that batcher submits, so a submission accepted on L1 would never reach the L2 chain.
And the inbox is a contract that can refuse: upstream selection consults only type, recipient and sender.
A reverted inbox call — or a successful call to some other function of the contract — would therefore be consumed anyway, and content the inbox refused would settle on L2 while the contract's [`Cursor`](certifier.md#cursor) correctly declined to advance.
Matching on the acceptance event closes both at once, because the event is the contract's decision and exists only in a successful receipt.
Excluding type 3 closes a third: the baseline reads a blob transaction's sidecar rather than its calldata, so a transaction could be accepted on its calldata while derivation ingested its blob.

**What it gives up.** Selection is no longer decidable from the transaction alone: it depends on the transaction's receipt, so a node must hold the L1 block's receipts, which it already fetches for deposit logs, and must stall rather than degrade when it cannot.
The event's ABI becomes consensus-critical, which is why the [batch inbox page](batch-inbox.md#events) pins its signature.

**Consensus-visible.** Yes.
Nodes disagreeing on which transactions carry the event — or one node applying a sender rule while another applies the event rule — derive different L2 chains from identical L1 data.

## Delta 3: Remove the parent_hash Check

**What changes.** Under the certified rules, derivation ignores the batch's `parent_hash` throughout batch validation and payload-attributes construction, as specified normatively under [Batch Parent Hash](../rollup/derivation.md#batch-parent-hash).
A retained comparison at either stage would still let the batcher force rejection or a reset through a field the certifier does not bind.
Batch decoding, timestamp checks, the actual block's parent-link validation and execution checks remain in force.
Legacy heights retain their existing rules.

**Where in the pipeline.** The batch queue and the attributes queue.

**Why it is required.** `parent_hash` is the only `SingularBatch` field the batcher is free to choose once the certifier pins the transaction list, `epoch_num`, and the timestamp against the selected record.
A registered alternative batcher can therefore submit a batch whose content is entirely correct and whose `parent_hash` is garbage; the certifier attests it, the inbox accepts it, and derivation drops it — turning the recovery role into a settlement-halting capability whose cost to the attacker is gas.
Removing the check retires the attack rather than defending against it: the batch then derives the selected content onto the actual parent, so the freedom the attacker held stops being read by anyone.
The field carries no information derivation needs — it is otherwise only logged, and does not feed payload attributes, which are built on the [safe head](https://specs.optimism.io/glossary.html#safe-l2-head).

**Why the removal preserves chain validity.** Only the comparison with the batch-supplied `parent_hash` is removed: the next block still extends derivation's current parent, and validation of the actual block's parent and execution remains in force.
Under the [certifier trust premises](full-node.md#conditions-under-which-it-holds), the core binds the ordinary transaction list to the selected record and its height through [canonical comparison](certifier.md#advancing-the-cursor); the retained timestamp checks keep that content at the corresponding derivation height.
Holding the remaining inputs fixed, changing the ignored batch field cannot select another parent or alter the derived result.
This preserves chain validity and content binding, not the original execution result across a changed parent state or L1 history.
After a reorganization or an earlier fallback changes those inputs, the same transactions can execute validly with different results; an invalid block instead follows the specified [deposit-only fallback](#the-two-kinds-of-failure).
This is the [existing guarantee boundary](full-node.md#what-it-does-not-cover), not permission to accept an invalid state transition.

**What it gives up.** The check is also an early warning that the batcher's history has diverged, and the response it triggers — waiting out a sequencing window before committing the deposit-only block — is a window in which an operator could still intervene.
The [full-node page](full-node.md#surface-what-it-has-determined) notes that intervention on a doomed height is time-bounded by the sequencing window.
The trade is a certain attack surface for a buffer that only matters once something else has already failed.

**Consensus-visible.** Yes.
A batch an unmodified OP Stack node drops is derived by a MegaETH node, and the two chains diverge at that height.

## Delta 4: Resolve the L1 Origin from epoch_num

**What changes.** Under the certified rules, derivation ignores the batch's `epoch_hash` and resolves the [L1 origin](https://specs.optimism.io/glossary.html#l1-origin) from `epoch_num` against canonical L1, as specified normatively under [Canonical L1 Origin Resolution](../rollup/derivation.md#canonical-l1-origin-resolution).
The resolved block supplies all origin-dependent inputs, including its header, deposit receipts and applicable configuration updates; unavailable required inputs cause waiting, and legacy heights retain their existing rules.

**Where in the pipeline.** Batch filtering, validation and payload-attributes construction, including origin-dependent data retrieval.
Deleting the comparison while continuing to retrieve inputs by the batch-supplied hash could import an orphaned block's deposits or configuration.

**Why it is required.** The certifier binds `epoch_num` to the selected record but leaves the batcher free to choose `epoch_hash`.
Rejecting a mismatch would let an authorized batcher discard otherwise correct selected content, while a reorganization can make an honestly supplied hash stale.
Given the same canonical L1 view, origin height and other derivation inputs, nodes resolve the same origin and derive the same result without consulting the supplied hash.
The remaining origin-continuity, timestamp and execution-validity checks still apply.

**What it gives up.** OP Stack rejects an unchanged batch whose named origin belongs to a displaced L1 branch; this delta deliberately allows its transactions to be executed against the replacement canonical origin at the same height.
The original branch binding is unnecessary for agreement on the derived chain under a common canonical L1 view, but its removal does not preserve execution results across a reorganization.
The same transactions can execute successfully with different results, in which case no deposit-only substitution occurs.
If the replacement inputs instead make the block invalid, the deposit-only escape hatch follows [recovery S3](recovery.md#s3-an-l1-reorganization).
The upstream protection binds the batch, not the users' transaction signatures: an authorized batcher can construct a new batch naming the replacement origin.
Here the publisher's record does not include the origin hash either; certification fixes the selected transaction content and origin height, while canonical L1 determines the execution context.

Removing the hash check also removes the [upstream causal premise](https://github.com/ethereum-optimism/specs/blob/42b446f5891dd5a232cfc346201a613c1bba3f50/specs/protocol/derivation.md#overview) that a batch cannot be included in its own L1 origin block because it must already contain that block's hash.
An origin height can be declared in advance, so data included in that origin can contain batches that validly extend a parent with the same origin.
Discarding them solely because the pipeline has reached the parent's origin would lose otherwise valid selected content; their timestamps and other validity conditions still determine whether they can extend the chain.

**Consensus-visible.** Yes.
A batch with a mismatching `epoch_hash` can be processed where the upstream rule rejects it, and a replacement origin can change deposits, other execution inputs and results.

## Delta 5: Substitute Instead of Dropping

**What changes.** Under the certified rules, four batch-validity rejection conditions produce a same-height deposit-only substitution under [Same-Height Deposit-Only Substitution](../rollup/derivation.md#same-height-deposit-only-substitution): the timestamp-against-origin condition, both rejection branches of the sequencer-drift condition, and the two origin-continuity conditions.
The drift branches include both non-empty batches beyond the bound and empty batches that unnecessarily retain the current origin; allowed empty cases and waiting for missing origin data remain unchanged.
The replacement uses the actual derived parent, the next L2 height and timestamp, and the canonical origin selected by the deposit-only fallback's one-step rule; it does not inherit an invalid origin claim from the rejected batch.
Required inputs are awaited with the current batch and channel suffix retained, subject to existing reset rules; once available, there is no additional sequencing-window wait.
The remaining batches are then processed against the substituted block as their parent.
Other rejection outcomes, including sequencing-window expiry and past or future timestamps, retain their own rules; this delta does not make every invalid batch an immediate substitution.

**Where in the pipeline.** The batch queue, at the validity checks that compare the block's timestamp and `epoch_num` against the derived parent's L1 origin.

**Why it is required.** All four conditions are determined by the selected record together with the derived chain's own history, so no party can correct them: by [the governing principle](#the-governing-principle) they must not be pre-empted — not at the inbox, and not in the certifier, which holds no L1 data and cannot evaluate them.
They should instead stop causing the expensive failure.
The substituted block cannot blindly inherit the record's origin: an origin or timestamp incompatibility can remain after the ordinary transactions are removed.
It follows the deposit-only fallback's rule instead, and this specification defines no origin rule of its own.
The derived chain's origin trajectory is the ground truth, and a record's `epoch_num` is the sequencer's claim about it.
A mismatched claim can lose its height's content, while later execution and record selection remain subject to the [S4 recovery conditions](recovery.md#s4-a-height-settles-as-a-deposit-only-block).

The origin-continuity conditions are converted because filling a height without its batch can separate the record and derived origin trajectories.
[Record Validity](block-records.md#record-validity) requires only that a record's origin not decrease from the preceding selected record's, while the batch queue requires a batch's origin to equal the derived parent's or exceed it by exactly one.
The deposit-only fallback can advance the derived origin before the already-published records do, but it can also choose the same origin as the record.
An execution-rejection replacement under [Δ6](#delta-6-keep-the-channel-after-an-engine-rejection) retains the origin that already passed batch validity checks, so deposit-only replacement does not itself imply origin divergence.
Records already finalized for later heights can then carry origins below the derived ones.
Upstream recovers from the same divergence by having the sequencer reorganize onto the derived chain and produce those heights again with matching origins, and under certification that path does not exist: the records are final, first-valid-wins forbids a second record for a resolved height, and the batcher cannot resubmit past the cursor.
Under the baseline's drop rule, a mismatched batch and the rest of its channel are discarded under [Holocene's strict ordering](https://specs.optimism.io/protocol/holocene/derivation.html#batch-queue), leaving the missing height to sequencing-window fallback.
If later records remain incompatible, repeated drops and sequencing-window waits can prolong deposit-only operation.
Substitution handles each origin mismatch at its own height; a later record with a compatible origin remains subject to all other validity and execution checks.
A record produced on the derived chain can pass selection however far its origin has advanced beyond the scan's preceding-origin anchor, because Record Validity permits any non-decreasing origin.
If an erroneous or malicious record instead placed that anchor ahead of the derived origin, returning to the derived chain is insufficient: the lower-origin record is not selected, and [S4](recovery.md#the-publisher-keeps-publishing) describes the existing override's recovery conditions and limits.
The gain for the timestamp conditions is the cascade length.
A dropped batch produces its deposit-only block only after a full sequencing window, during which the cursor keeps advancing over heights whose batches will also be dropped, while a substitution produces it once its required L1 inputs are available, without the additional sequencing-window delay.
The affected range still depends on input availability, detection and publication timing.

Every substitution path gates the deposit-only request on the Holocene upgrade being active; without it the engine's rejection reverts the pending safe head and retries rather than a substitution, and this delta does not work at all.
That is not an additional assumption: fork ordering requires Holocene no later than [Isthmus](https://specs.optimism.io/protocol/isthmus/overview.html), and [the activation premise](overview.md) this section already relies on — that certification activates at a height by which Isthmus is active — therefore establishes it.

**Why the sequencing-window condition on the same axis is left alone.** Derivation also drops a batch whose L1 origin is older than `seq_window_size` L1 blocks before the block including it, and that condition is deliberately not converted.
It differs in kind from the four above rather than in degree: those four lose a height that would otherwise have settled, whereas this one recognizes a height the sequencing window has already elapsed on.
The deposit-only fallback that fills such a height runs off the same arithmetic — the epoch's number plus `seq_window_size`, compared against the pipeline's own L1 origin.
So by the time this condition can fire, the height is either already filled, and caught earlier by the timestamp comparison against the safe head, or about to be filled by that fallback.
The one gap between them is that the condition measures against the L1 block that included the batch while the deposit-only fallback measures against the origin the pipeline has reached, which differ while the pipeline lags the L1 head.
In that window the condition fires first, the batch is dropped, and the deposit-only fallback fills the height once the pipeline advances past the expiry.
Converting it would therefore only make the same outcome arrive slightly sooner, at the price of a further divergence from upstream.
This reading of the gap is reasoned from the arithmetic of the two paths rather than verified by test, and is recorded as such.

**What it gives up.** Once the substitution is applied, the height's ordinary content is lost without the additional sequencing-window wait that upstream's drop leaves for some other batch to fill the height.
Under certification that window is worthless, because no corrected batch for a finalized record can exist — but the substitution does forfeit it, and it forfeits the operator-intervention buffer for the same reason Δ3 does.

**Consensus-visible.** Yes.
The height settles as a deposit-only block where an unmodified OP Stack node would settle it as deposit-only a full sequencing window later, and every height in between differs.
For the origin conditions the difference is larger still: an unmodified OP Stack node stays deposit-only until the sequencer produces those heights again, which under certification never happens.

## Delta 6: Keep the Channel After an Engine Rejection

**What changes.** Under the certified rules, an execution `INVALID` verdict substitutes a deposit-only block while retaining the remaining channel batches, as specified under [Channel Retention After Execution Rejection](../rollup/derivation.md#channel-retention-after-execution-rejection).
The replacement preserves Holocene's construction inputs, including the already resolved origin, and filters the transaction list to deposits; it does not reselect the origin as [Δ5](#delta-5-substitute-instead-of-dropping) does for a batch-validity rejection.
After replacement, each remaining batch is validated and executed against the updated derived chain, just as after a Δ5 substitution.

**Where in the pipeline.** The handling of an `INVALID` engine result under Holocene, which today requests deposit-only attributes and flushes the channel the batch was decoded from.

**Why it is required.** Upstream discards the rest of the channel "for consistency reasons": the remaining batches were built on the block that was just replaced, so their `parent_hash` no longer matches and re-deriving them would only fail that check later.
Under certification the reason is gone — [Δ3](#delta-3-remove-the-parent_hash-check) stops derivation reading `parent_hash` — and the remaining batches are selected content that only this channel can supply.
The inbox's cursor has passed every block in it, so no other submission can carry them, and a discarded remainder settles deposit-only after a full sequencing window even where its transactions would have executed.
Discarding that remainder would make channel boundaries determine additional content loss from this engine rejection, including heights whose transactions remain valid under [S4](recovery.md#s4-a-height-settles-as-a-deposit-only-block).
Retaining it removes that particular source of packing-dependent loss, without removing other batch-validity, channel or delivery conditions.
Each remaining batch is still subject to the applicable validity checks; if it reaches execution, a batch the [nonce cascade](recovery.md#why-a-height-cannot-be-repaired-in-place) dooms is rejected and substituted in turn, while a valid batch retains its ordinary content.

**What it gives up.** Nothing certification could use: the discarded remainder could only ever have been resubmitted in a new channel, which the cursor forbids.
It is one more point at which the pipeline departs from upstream and must be re-examined on every rebase.

**Consensus-visible.** Yes.
After an engine rejection an unmodified OP Stack node settles the rest of the channel deposit-only a sequencing window later, while a MegaETH node derives each remaining batch on its own merits.

## Delta 7: Activation Boundary

**What changes.** The admin schedules the first certified L2 height and Espresso trust anchor; the legacy batcher's first accepted certified submission fixes the L1 transition position.
Legacy content is restricted to lower heights, and a channel spanning the boundary is rejected in full.
At the bootstrap transaction the node processes preceding eligible legacy inputs, discards remaining legacy assembly state, and immediately fills missing legacy heights as deposit-only before processing certified frames.
A partial-channel bootstrap activates selection without releasing a certified block before channel completion.
This delta is specified normatively under [Activation and Channel Boundary](activation.md#derivation-transition).

**Where in the pipeline.** L1 retrieval, ordered activation-marker handling, channel assembly, batch release and deposit-only block construction.

**Why it is required.** Classifying the same submission by the next derived height can make continuous processing discard data that replay from a later safe head accepts.
The canonical bootstrap position supplies one transition order regardless of node progress; omitted legacy content has a deterministic fill rather than blocking activation for a sequencing window.

**What it gives up.** The legacy batcher can abandon missing pre-activation content by bootstrapping; subsequent legacy publication cannot restore it.
A missing bootstrap stalls derivation at the last legacy height; a partial bootstrap remains subject to channel completion and availability.

**Consensus-visible.** Yes.
Selecting activation from a local safe head or allowing prefetch to overtake preceding legacy inputs can produce different chains from identical L1 history.

## Delta 8: Emergency Cursor Overrides

This delta is specified normatively under [Emergency Cursor Overrides](../rollup/derivation.md#emergency-cursor-overrides).

**Where in the pipeline.** Ordered L1 event handling, channel assembly and deposit-only block construction.

**Why it is required.** Advancing only the contract cursor leaves derivation expecting heights that subsequent certified submissions have already passed.
Processing the override marker supplies a shared gap-fill boundary and prevents later frames from entering the abandoned channel state.

**What it gives up.** The admin can abandon missing ordinary content without waiting for sequencing-window expiry.
The operation is advance-only and preserves the established safe prefix, including finalized history; it supplies neither batch content nor independent finality.

**Consensus-visible.** Yes.
Ignoring the event or applying it out of order can produce different chains from identical L1 history.

## Delta 9: Validate the Complete Compressed Stream

This delta is specified normatively under [Closed-Channel Decoding](../rollup/derivation.md#closed-channel-decoding), using the same [stream-validation rules as the certifier](certifier.md#closed-channel-decoding).

**Where in the pipeline.** Channel reading, decompression and batch release.

**Why it is required.** A corrupted integrity trailer can make different streaming decoders return different batch prefixes before reporting the error.
A certifier that returns more blocks than derivation can advance the accepted cursor past undelivered ordinary content.
Complete-stream validation makes both return the same byte string or no blocks, independently of buffering.

**What it gives up.** A valid batch prefix cannot be salvaged from a malformed, truncated, integrity-failing or oversized compressed stream.
Successful decompression still permits recovery of complete batches preceding a batch-encoding error.
The batcher can retry unresolved records in a fresh channel after a vacuous close.

**Consensus-visible.** Yes.
The upstream prefix can contribute blocks that the certified rule discards, so the change applies only under certified selection; legacy decoding retains its inherited behavior.

## Consequences Elsewhere

With Δ3 and Δ4 applied, every `SingularBatch` field is pinned: the transaction list, `epoch_num`, and the timestamp by the certifier against the selected record, and `parent_hash` and `epoch_hash` by no longer being read.
[Span batches](https://specs.optimism.io/protocol/derivation.html#span-batches), which carry the further batcher-chosen fields `parent_check` and `l1_origin_check`, are excluded by [`CHANNEL_PARAMS`](certifier.md#constants), so no delta is needed for them.
The batcher becomes a transport with no influence over settled content, which is the premise the [Independent Finality](full-node.md#the-guarantee) property rests on.

What remains unreachable is `parent_hash` as a _correctness_ condition rather than an attack surface: establishing that a batch builds on the canonical L2 parent requires the parent's hash, which requires executing the chain.
This is the boundary between attesting ordering and content, which the certifier does, and attesting execution, which it does not — and it is exactly the condition a zero-knowledge circuit over execution would close.

## Divergence Register

The eight deltas above — Δ1 and Δ3 to Δ9, Δ2 being folded into Δ1 — are the complete set of consensus-level divergences block certification introduces into the derivation pipeline.
There are no others: no further delta is planned, and any additional divergence discovered later belongs in this register before it is implemented.

Once they are applied, a MegaETH node and an unmodified OP Stack node derive different L2 chains from identical L1 data.
That is true of each delta individually, not only of the set.
Δ1 changes which L1 transactions are ingested, Δ3 changes which batches survive validity checking, and Δ4 changes which L1 origin a batch is built against and therefore its deposit set.
Δ5 changes when and how a height without valid content is filled, Δ6 changes what becomes of the rest of a channel after an engine rejection, Δ7 changes how the pipeline reads L1 across activation, Δ8 applies forward override gap filling in L1 order, and Δ9 validates the complete compressed stream before releasing batches.

The cost of each is permanent maintenance and audit surface rather than a one-time change.
Every one of them is a point at which upstream changes cannot be merged without a decision, and a point an auditor must re-examine on every upstream rebase.
This is the reason the set is enumerated in one place and kept small.

## Constants

| Constant                    | Definition                                         |
| --------------------------- | -------------------------------------------------- |
| `CHANNEL_TIMEOUT`           | [Batch inbox constants](batch-inbox.md#constants). |
| `MAX_RLP_BYTES_PER_CHANNEL` | [Certifier constants](certifier.md#constants).     |
| `CHANNEL_PARAMS`            | [Certifier constants](certifier.md#constants).     |

## Spec History

Block certification is not activated on any network.
