---
description: MegaETH deltas over OP Stack derivation, including batcher transaction selection under block certification.
spec: TBD
---

# Derivation

{% hint style="warning" %}
This page is incomplete.
It currently specifies only the deltas [block certification](../block-certification/overview.md) introduces, which are unstable and not activated on any network.
The remaining deltas over the [OP Stack derivation specification](https://specs.optimism.io/protocol/derivation.html) — the handling of invalid or unavailable certificates beyond [Batch Commitment Resolution](data-availability.md#batch-commitment-resolution) — have not been written.
{% endhint %}

MegaETH derivation is the [OP Stack derivation pipeline](https://specs.optimism.io/protocol/derivation.html), unmodified except where this page states otherwise.
It does not verify [Espresso](https://docs.espressosys.com/network/learn/what-is-espresso) proofs or [certifier](../block-certification/certifier.md) attestations: it distinguishes legacy data from data admitted by the [L1 batch inbox](../block-certification/batch-inbox.md), and enforces which L2 heights each source may supply.
The activation procedure is specified by [Block Certification Activation](../block-certification/activation.md).
This page specifies [batcher transaction selection](#batcher-transaction-selection), [closed-channel decoding](#closed-channel-decoding), [batch parent-hash handling](#batch-parent-hash), [canonical L1 origin resolution](#canonical-l1-origin-resolution), [same-height substitution](#same-height-deposit-only-substitution), [channel retention after execution rejection](#channel-retention-after-execution-rejection), [emergency cursor overrides](#emergency-cursor-overrides), and the behavior certification leaves unchanged.

## Motivation

Certification upgrades the implementation behind the fixed batch inbox proxy and widens the permitted submitters from one address to a curated set.
It also makes contract acceptance a prerequisite for post-checkpoint batch content.

The single-sender assumption is explicit in the baseline and simply becomes false: a pipeline that filters on one configured address would ignore every batch an alternative batcher submits, which would make the recovery path the [exclusive window](../block-certification/batch-inbox.md#exclusive-window) opens unusable — settlement would be accepted on L1 and never reach the L2 chain.

Legacy batch selection does not depend on transaction execution success.
That permits legacy data to remain usable even if the upgraded proxy implementation reverts the submission, but carrying the same rule into the certified rules would ingest submissions the certification contract refused.

Under the certified rules, the contract emits an event exactly when it accepts a submission, and derivation ingests only transactions that carry it.
Historical batches still need the legacy rules, so the activation boundary also determines which L2 heights each may supply.

The activation boundary, legacy channel restrictions, bootstrap-triggered fill and replay behavior are specified exclusively by [Block Certification Activation](../block-certification/activation.md).

## Batcher Transaction Selection

The baseline selects a batcher transaction by recipient and sender: a transaction counts if it is sent to the batch inbox address and its sender is the `batcherAddr` held in [`SystemConfig`](https://specs.optimism.io/protocol/system-config.html).
Under the [certified rules](../block-certification/activation.md#derivation-transition), MegaETH replaces the sender condition with an acceptance condition, excludes blob-carrying transactions and explicitly admits [EIP-7702](https://eips.ethereum.org/EIPS/eip-7702) type 4 transactions; every other aspect of selection is retained.
The rules in this section apply to the bootstrap transaction and subsequent inputs; preceding legacy inputs retain their pre-certification selection and data-resolution rules.

For the bootstrap and while the certified rules are active, a node MUST select a transaction if and only if all of the following hold:

- it is sent to the fixed batch inbox proxy address, per the baseline;
- its type is one of `0`, `1`, `2`, `4` or `0x7e` (the baseline's deposited transaction type for nested OP Stack chains): type `3` and all other types are excluded, whatever their calldata or receipts say; and
- **the inbox accepted it** — its receipt reports success and carries the [`CursorAdvanced`](../block-certification/batch-inbox.md#events) event emitted by the batch inbox address, matched by the `topic0` that page fixes.

The sender is not evaluated: neither `batcherAddr` nor any registry confers ingestion, and a transaction from the primary batcher whose receipt carries no acceptance event is not a batcher transaction.
A node MUST NOT ingest the calldata of a transaction whose receipt reports failure, and MUST NOT ingest a successful transaction whose receipt carries no acceptance event, whatever its sender.
The event is an L1 log, so a node reads it from the same L1 receipts it already fetches to extract deposit logs; a node that cannot obtain the receipts of an L1 block MUST NOT derive from that block until it can, and MUST NOT fall back to a sender-based selection.

The inbox's [direct-submission checks](../block-certification/batch-inbox.md#acceptance) reject contract batchers and internal-call submissions, so an accepted batch submission's calldata is the outer transaction's calldata.
The bytes a node ingests from a batcher transaction are exactly its calldata, which is the byte string the inbox checked; a node MUST NOT read a blob sidecar or any other data attached to the transaction.
How that calldata resolves to frame bytes is specified under [Batch Commitment Resolution](data-availability.md#batch-commitment-resolution) on the data availability page.

The [batcher registry](../block-certification/batch-inbox.md#batcher-registry) that decides who may submit is the inbox contract's own state, evaluated by the contract alone; `SystemConfig.batcherAddr` plays no part in selection under the certified rules; selection under the legacy rules retains its historical authorization checks.

## Closed-Channel Decoding

The OP Stack [channel reader](https://github.com/ethereum-optimism/specs/blob/42b446f5891dd5a232cfc346201a613c1bba3f50/specs/protocol/derivation.md#channel-reader-batch-decoding) decodes batches from a decompressed stream and retains batches obtained before a later decompression or decoding failure.
Under the certified rules, a node MUST validate the complete compressed bytes of an assembled closed channel using [Closed-Channel Decoding](../block-certification/certifier.md#closed-channel-decoding) before releasing any batch.
It MUST use the same pinned compression algorithm, stream checks, decompressed-byte bound, first-stream and trailing-byte rules as the certifier.
A truncated, malformed, integrity-failing or oversized compressed stream MUST contribute no batches; the node MUST discard that channel without releasing a prefix or substituting guessed bytes.
A compression type outside the pinned algorithm MUST contribute no batches.
The node MUST NOT retain a batch merely because its streaming decompressor returned that batch's bytes before reporting a later stream error.

After successful stream validation, the node MUST decode the complete resulting byte string in wire order, retaining complete batches preceding a recoverable batch-encoding error and discarding its malformed or incomplete suffix.
It MUST retain the applicable batch-version, [activation channel-boundary](../block-certification/activation.md#channel-boundary), validity and execution rules; stream validation MUST NOT itself imply that any decoded batch is valid or executable.
A stream yielding no batches MUST NOT itself trigger an immediate deposit-only replacement; missing heights MUST follow the applicable fallback or ordered marker-fill rules.
Legacy channels MUST retain their inherited decompression and decoding behavior.

## Batch Parent Hash

The OP Stack [batch-validity rules](https://github.com/ethereum-optimism/specs/blob/42b446f5891dd5a232cfc346201a613c1bba3f50/specs/protocol/derivation.md#batch-queue) reject a batch whose `parent_hash` differs from the derived parent's hash.
Under the [certified rules](../block-certification/activation.md#derivation-transition), a node MUST ignore the value of the batch's `parent_hash` throughout derivation: it MUST NOT select a parent, reject a batch, or reset the pipeline on the basis of that value.
This applies both during batch validation and during payload-attributes construction; retaining the comparison at either stage violates this rule.
The field remains in the batch encoding, and a node MUST retain the applicable batch-decoding checks.
A node MUST derive the block on its current derived parent and MUST retain the timestamp checks that bind the batch to its L2 height, the actual block's parent-link validation, and execution-validity checks.
At and below the activation checkpoint, a node MUST retain the legacy rules, including their batch `parent_hash` check.
The rationale and the validity argument are under [Δ3](../block-certification/derivation-pipeline.md#delta-3-remove-the-parent_hash-check).

## Canonical L1 Origin Resolution

The OP Stack [batch-validity rules](https://github.com/ethereum-optimism/specs/blob/42b446f5891dd5a232cfc346201a613c1bba3f50/specs/protocol/derivation.md#batch-queue) reject a batch whose `epoch_hash` differs from the canonical origin selected by `epoch_num`.
Under the [certified rules](../block-certification/activation.md#derivation-transition), a node MUST ignore the value of the batch's `epoch_hash` throughout derivation and MUST NOT reject or reset on the basis of that value.
When constructing a block from a batch that passes the remaining batch-validity rules, a node MUST resolve its origin to the authenticated canonical L1 block at `batch.epoch_num` in the node's current L1 view.
All origin-dependent inputs MUST refer to that resolved block: its header, the receipts used to derive deposits and applicable system-configuration updates, and the resulting L1 attributes.
A node MUST retain the existing rules for processing those inputs, including when to include deposits; it MUST NOT use the batch-supplied hash to select or retrieve them.
When required canonical headers or receipts are unavailable, a node MUST wait and retry when they become available; it MUST NOT substitute data selected by the batch's `epoch_hash` or treat unavailability alone as batch invalidity.
After an L1 reorganization, a node MUST apply the existing rollback and re-derivation rules and resolve affected origins against the replacement canonical history.
The field remains in the batch encoding, and a node MUST retain the applicable decoding, origin-continuity, timestamp and execution-validity checks, with the specified failure outcomes under [Same-Height Deposit-Only Substitution](#same-height-deposit-only-substitution).
A node MUST NOT reject a batch solely because its `epoch_num` equals the number of its L1 inclusion block.
When the pipeline is reading an L1 block whose number equals the current derived parent's L1 origin number, a node MUST evaluate the available batches under the applicable timestamp and other batch-validity rules; it MUST NOT discard them as past solely on the basis of that equality.
These cases remain subject to the canonical-input waiting rules above.
At and below the activation checkpoint, a node MUST retain the legacy origin-hash rules.
The rationale and the changed branch-binding semantics are under [Δ4](../block-certification/derivation-pipeline.md#delta-4-resolve-the-l1-origin-from-epoch_num).

## Same-Height Deposit-Only Substitution

Under the [certified rules](../block-certification/activation.md#derivation-transition), a node MUST replace the batch-validity rejection outcomes listed below with a deposit-only block at the same L2 height, without waiting for the sequencing window to expire.
A node MUST retain the baseline validity-check order, as modified by this page, and MUST NOT apply this substitution unless the batch's timestamp equals the next expected L2 timestamp and the earlier checks have passed.
In particular, sequencing-window expiry, past or future batch timestamps, malformed input and unrelated rejection causes MUST retain their specified outcomes; they MUST NOT trigger this substitution.
Execution rejection follows [Channel Retention After Execution Rejection](#channel-retention-after-execution-rejection), with different replacement inputs.

Let `P` be the current derived parent, `e` its L1 origin number, and `t = P.timestamp + l2_block_time` the next expected timestamp, using the [rollup-configured block time](../block-certification/block-records.md#fields).
The converted rejection conditions are:

- `batch.epoch_num < e`;
- `batch.epoch_num > e + 1`;
- the batch timestamp is earlier than its canonical origin's timestamp; or
- the batch exceeds the applicable sequencer-drift bound and either contains ordinary transactions, or contains none but retains origin `e` when the known canonical origin at `e + 1` has a timestamp at or before `t`.

A node MUST preserve the [upstream drift exceptions](https://github.com/ethereum-optimism/specs/blob/42b446f5891dd5a232cfc346201a613c1bba3f50/specs/protocol/derivation.md#batch-queue): an empty batch may exceed the bound when advancing the origin, or when retaining the current origin because the next origin's timestamp is later than `t`.
If a required next-origin header is unavailable, a node MUST wait rather than infer a drift violation.

For a converted rejection, a node MUST construct the replacement from `P` and the canonical L1 history using the deposit-only fallback's payload-construction rules, with these inputs fixed:

- L2 height `P.number + 1`, actual parent `P`, and timestamp `t`;
- origin `e` if `t` is earlier than the canonical L1 block at `e + 1`'s timestamp, and origin `e + 1` otherwise; and
- an empty ordinary transaction list.

A node MUST use the resolved canonical origin's hash and header and MUST apply the inherited deposit, L1-attributes and applicable configuration-update rules to that origin and `P`, including when deposits are included.
The replacement MUST satisfy the inherited block-construction and execution-validity rules; the rejected batch's claimed origin MUST NOT determine the replacement's origin or its L1 inputs.
This fixes the fallback at the current parent and height, rather than at a hypothetical later pipeline state after waiting out the sequencing window.

If inputs required to evaluate the condition or construct the replacement are unavailable, a node MUST retain the current batch and the remaining batches of its channel, wait, and retry at the same height against the then-canonical L1 view, subject to existing reorganization and reset rules.
A node MUST NOT treat missing inputs alone as invalidity, guess the replacement origin, proceed to a later batch, or add a sequencing-window wait once the required inputs are available.
After deriving the replacement, a node MUST consume the rejected batch, retain the channel's remaining batches, and process them in order against the updated derived chain, applying all applicable validity and execution checks anew.
This substitution MUST NOT itself flush the channel; later failures remain subject to their own rules.
At and below the activation checkpoint, a node MUST retain the legacy failure outcomes.
The rationale and trade-offs are under [Δ5](../block-certification/derivation-pipeline.md#delta-5-substitute-instead-of-dropping).

## Channel Retention After Execution Rejection

Under the [certified rules](../block-certification/activation.md#derivation-transition), when a regularly derived payload receives an execution-validity verdict of `INVALID`, a node MUST apply the [Holocene deposit-only replacement rule](https://github.com/ethereum-optimism/specs/blob/42b446f5891dd5a232cfc346201a613c1bba3f50/specs/protocol/holocene/derivation.md#engine-queue) without discarding the channel's remaining batches.
The node MUST retain the failed payload's construction inputs other than its transaction list, including its actual parent, height, timestamp and resolved canonical L1 origin, and MUST filter the transaction list to its deposit transactions in their original order.
It MUST NOT reselect the origin using the rule for [batch-validity substitution](#same-height-deposit-only-substitution).
Execution of the replacement determines its resulting state and block commitments; retaining construction inputs does not retain the failed payload's execution outputs.

After successfully deriving the replacement, the node MUST consume the failed batch and process the retained batches in order against the updated derived chain, applying all applicable batch-validity and execution checks anew.
An execution rejection MUST NOT itself flush the channel; later failures remain subject to their own rules, and existing reorganization and reset rules remain applicable.
Retaining a batch does not imply that it remains executable after the replacement.

A node MUST NOT treat an ordinary transaction's EVM revert, unavailable required inputs, or an RPC failure alone as a block-level `INVALID` verdict; these retain their applicable execution, wait or retry behavior.
The node MUST retain Holocene's critical-error handling if the deposit-only replacement itself is invalid; it MUST NOT skip that height or process the channel suffix on the failed replacement.
At and below the activation checkpoint, a node MUST retain the legacy execution-rejection and channel-discard rules.
The rationale and trade-offs are under [Δ6](../block-certification/derivation-pipeline.md#delta-6-keep-the-channel-after-an-engine-rejection).

## Emergency Cursor Overrides

The OP Stack [batch queue](https://specs.optimism.io/protocol/derivation.html#batch-queue) generates missing deposit-only blocks after sequencing-window expiry and has no certified-cursor override input.
Under the certified rules, a node MUST process each successful receipt's [`CursorSet`](../block-certification/batch-inbox.md#events) log emitted by the fixed inbox proxy as an ordered control input, in canonical L1 block, transaction and log order.
It MUST identify the event by the emitting address and specified `topic0`, independently of the outer transaction's recipient or sender; an admin contract MAY call the inbox internally.
It MUST NOT ingest the override transaction's calldata as batch content or treat the event as a `CursorAdvanced` acceptance.

Before applying the override, the node MUST resolve preceding accepted inputs and process the complete channels they supply under their applicable resolution, validity and execution rules.
Unavailable required data MUST retain the applicable waiting or failure outcome; local retrieval status alone MUST NOT turn preceding content into missing heights.
After processing those inputs, the node MUST discard unfinished certified channel assembly and any pending batches from the abandoned channel state, and MUST NOT wait for later frames to complete that state.

Let `N` be the event's supplied `next.block_number` and `L` its canonical including L1 block.
If the actual derived safe head is below `N`, the node MUST fill each remaining height through `N` with a deposit-only block before processing later L1 inputs, without waiting for sequencing-window expiry.
For each fill block, it MUST apply the construction, canonical-origin selection, input-availability, execution-failure and non-wrapping arithmetic rules of [Activation Deposit-Only Fill](../block-certification/activation.md#activation-deposit-only-fill), substituting this event's `L` and the inclusive target `N` for the bootstrap block and checkpoint.
The bootstrap timestamp bound applies only to bootstrap; an override MUST NOT be treated as evidence that the target's timestamp is at or before `L.timestamp`.
The event's supplied `epoch_num` MUST NOT choose replacement origins or authenticate derived state.

The node MUST preserve its established safe prefix in the same canonical L1 history, including finalized blocks, and MUST NOT reorganize that prefix to match the supplied cursor.
If the safe head is already at or above `N`, it MUST perform the channel reset without filling or rewinding.
After applying the override, it MUST evaluate subsequent batches against the next height above its actual derived head, rather than assuming that head equals `N`.
The supplied cursor fixes record-scan resumption; it MUST NOT select a replacement L2 parent.

On synchronization or reset, a node MUST recover relevant canonical override positions and reproduce their effects even when its normal L1 replay range omits the events.
If the recovered safe head precedes an omitted override's target, it MUST rewind L1 replay far enough to process the preceding inputs and override in order, rather than infer missing content from the target alone.
An L1 reorganization removing or moving an override MUST apply the inherited rollback and re-derivation rules to its effects; this MUST NOT authorize governance to roll back finalized history.
Override inclusion MUST NOT independently confer finality on replacement blocks; safe and finalized status MUST follow the inherited rules.

## Unchanged Behavior

The following are inherited from the baseline without modification, and a node MUST NOT vary them on account of certification:

- The [sequencing window](https://specs.optimism.io/protocol/derivation.html#overview) rule and `seq_window_size`, except for the immediate [activation fill](../block-certification/activation.md#activation-deposit-only-fill), [override fill](#emergency-cursor-overrides), and the prohibition on deriving certified heights before bootstrap.
- [Channel timeout](https://specs.optimism.io/protocol/derivation.html#channel-bank) and incomplete-channel discard behavior, except for the activation discard under [Activation and Channel Boundary](../block-certification/activation.md#derivation-transition) and the [override reset](#emergency-cursor-overrides).
- Frame ordering, channel assembly, and batch decoding under the selected rules, except for [complete-stream validation](#closed-channel-decoding) and the whole-channel boundary check before batches are released.
- Derivation of deposit and forced-inclusion transactions from L1, using the origin selected under [Canonical L1 Origin Resolution](#canonical-l1-origin-resolution), [Same-Height Deposit-Only Substitution](#same-height-deposit-only-substitution), [Activation Deposit-Only Fill](../block-certification/activation.md#activation-deposit-only-fill), or [Emergency Cursor Overrides](#emergency-cursor-overrides).

A node MUST NOT apply any timing check of its own to a submission's age, and MUST NOT poll the inbox cursor to choose its activation phase; the canonical activation events determine that phase.
Whether a late batch still counts is decided by the inherited sequencing-window rule, subject to the activation boundary and ordered override effects.

## Rationale

**Why selection keys on the acceptance event rather than on senders.**
For transactions evaluated under the certified rules, selection requires evidence of inbox acceptance: admitting a refused submission could settle content the contract's `cursor` never passed.
Acceptance does not guarantee that data reaches the derived chain: the activation boundary, transaction-type and recipient restrictions, and downstream validity and timing rules still apply.
A sender-based rule can only approximate that invariant, because the contract's decision also depends on the exclusive window, on registry membership at the moment of execution, and on the attestation and certificate checks, none of which a sender tells a node about.
Keeping the two sides aligned would need derivation to track a registry in transaction order and to define "sender" exactly as the contract does.
The acceptance event is the contract's decision itself, emitted in the same transaction and discarded with it on revert, so matching it supplies this evidence without reconstructing authorization in derivation and subsumes the receipt-status condition, since no event survives a failed transaction.
The registry formulation was considered and rejected on those grounds, and because it would have widened `SystemConfig`, whose batcher fields this design leaves untouched.

**Why the event is required of the primary batcher too.**
Exempting the configured `batcherAddr` from the event condition would reopen both failures the condition closes: a successful non-submission call from that address — a key registration, for instance — would be ingested as a batch, and a node without receipts would silently degrade to sender-based selection and diverge from nodes that have them.
One condition for every sender keeps selection a single rule with no privileged path.

**Why blob-carrying transactions are excluded.**
The inbox checks a submission's calldata and nothing else, while the baseline reads a type 3 transaction's blob sidecar and ignores its calldata, so admitting type 3 would let one transaction present honest calldata to the contract and arbitrary bytes to derivation.
Excluding the type keeps the bytes derivation ingests identical to the bytes the contract accepted, which is the same invariant the acceptance event preserves for the transaction as a whole.
The [inbox](../block-certification/batch-inbox.md#acceptance) also refuses a submission that carries blobs, so an excluded transaction never carries the acceptance event either; the derivation rule stands on its own so that selection stays decidable without relying on that refusal.

**Why type 4 transactions remain eligible under certified selection.**
An [EIP-7702 authorization list](https://eips.ethereum.org/EIPS/eip-7702#behavior) can set another account's delegation without adding code to the transaction sender.
A registered sender with empty code can therefore submit directly to the inbox using type 4 and pass the [acceptance checks](../block-certification/batch-inbox.md#acceptance); excluding that type in derivation would discard an accepted submission after the cursor advanced.
Its eligibility does not admit a delegated batcher: the inbox still rejects a sender with non-empty account code at submission, including delegation set during that transaction's authorization processing.
Derivation reads only the accepted transaction's calldata, and this type list does not automatically admit future transaction types.

**Why derivation does not repeat certification verification.**
The inbox gates certified submissions before derivation reads them.
Derivation checks source eligibility and the activation boundary, but does not repeat the certifier's Espresso proof or selected-record checks.

## Security Considerations

**Override order must agree across nodes.**
A node MUST NOT process later accepted frames or infer a missing height before resolving the inputs preceding `CursorSet`.
Ignoring the marker can leave later batches ahead of the expected height; applying it before preceding content is resolved can abandon content another node derives.
The shared L1 event order, actual-parent fill and preservation of the established safe prefix prevent those outcomes.
The admin can abandon missing content and choose a large forward target, but the override provides no authority to replace finalized history.

**Assets and threat model.** This section protects two properties: that above the [activation checkpoint](../block-certification/activation.md#derivation-transition) a node ingests exactly the calldata the L1 batch inbox accepted, and nothing else, and that every node derives the same chain from identical L1 data.
Derivation trusts the inbox's acceptance decision, as the [`CursorAdvanced`](../block-certification/batch-inbox.md#events) event in an L1 receipt reports it, and does not repeat the certifier's checks behind that decision; for heights at or below the checkpoint it applies legacy derivation and the specified activation fill, not certified content verification.
Every transaction sender is untrusted, the configured `batcherAddr` included, as are any bytes a transaction carries other than its calldata.
A node without L1 receipts has no trusted substitute for the event and must stall.
The [Roles and Trust](../block-certification/overview.md#roles-and-trust) table records the wider premises these rules rest on.

**Canonical origin resolution replaces the batch's branch binding.**
Agreement on canonical L1 provides a common origin at each height; it does not authenticate the batch-supplied hash.
Removing the equality check while continuing to use that hash for receipt retrieval could import deposits or configuration from an orphaned block.
All origin-dependent inputs therefore use the resolved canonical block.
An unchanged batch can now be processed against a replacement origin after a reorganization; valid execution may produce different results, while invalid execution follows the applicable fallback rules.
The [full-node guarantee](../block-certification/full-node.md#conditions-under-which-it-holds) remains conditional on unchanged L1 execution inputs.

**Ingesting a transaction the inbox did not accept would nullify the certification gate.**
For post-checkpoint heights, the acceptance condition and source-boundary check prevent ordinary batch content that the certification inbox did not admit from reaching the L2 chain.
Without it, any address — a registered batcher through a reverted submission, or through a successful call to some other function of the contract — could place calldata committing to content no selected record carries in a transaction to the inbox and have derivation ingest the bytes anyway.
The [Certification Claim](../block-certification/overview.md#the-certification-claim) would be false, and the failure would be silent: the contract's `cursor` would correctly refuse to advance while the L2 chain settled that content regardless.

**Ingesting bytes the inbox did not check would nullify it just as completely.**
The acceptance event establishes that the contract admitted the transaction, not which bytes derivation should read from it.
Had type 3 remained permitted, a submission whose calldata is an honest commitment would be accepted, and a node reading its blob sidecar would ingest bytes no certifier attested and no certificate covers, resolving under [alt-DA](https://specs.optimism.io/experimental/alt-da.html) to whatever the submitter dispersed.
The bypass would be silent, because the contract's `cursor` advances exactly as for an honest submission; a node MUST therefore read only the calldata, and MUST treat a blob-carrying transaction as not a batcher transaction even if a receipt carried the event.

**The event's signature and emitting address are consensus-critical.**
Selection matches on `topic0` and on the emitting address, so two nodes that computed the topic from different signature strings, or that matched logs from different addresses, would ingest different transactions from identical L1 data and derive different chains.
Changing `Cursor`'s canonical type sequence changes `topic0`, so nodes matching the old topic no longer select those submissions.
Reordering fields of the same type can instead preserve `topic0` while changing the meaning decoded by consumers; topic matching alone does not establish layout compatibility.
Both cases require the coordinated event-interface upgrade described on [the batch inbox page](../block-certification/batch-inbox.md#security-considerations).

**A node without receipts must stall rather than guess.**
Receipt availability is a required data dependency, and it cannot be degraded: a node that fell back to sender-based selection when receipts were unavailable would ingest a different set of transactions from a node that had them, which is a fork from identical L1 data.

**Applying a timing check in derivation would fork the chain.**
Nodes that rejected a batch for arriving late while others accepted it would derive different chains from identical L1 data.
The sequencing window rule is the single authority on lateness precisely so that this cannot happen, which is why the inbox contract deliberately performs no sequencing-window check of its own; the [channel timeout](../block-certification/batch-inbox.md#acceptance) it does enforce is a copy of derivation's own rule.

## Spec History

No derivation deltas have been activated on any network.
