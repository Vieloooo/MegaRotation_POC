---
description: Activation of block certification — administrative scheduling, the first certified submission, and the ordered transition from legacy derivation.
spec: TBD
---

# Block Certification Activation

This page specifies the complete transition from legacy batch submission and derivation to block certification.
Ordinary [certifier verification](certifier.md#verification-procedure), [inbox acceptance](batch-inbox.md#acceptance) and [certified derivation](../rollup/derivation.md) remain defined by their respective pages.

{% hint style="warning" %}
This procedure is unstable and is not activated on any network.
Network-specific parameters remain unassigned.
{% endhint %}

## Motivation

A future L2 activation height does not identify where in L1 history a node should change its input interpretation.
Selecting interpretation from a node's current derived head can make continuous processing discard a certified submission that replay from a later safe head accepts.
An explicit bootstrap position supplies a common transition, while deterministic filling lets the legacy batcher omit pre-activation content without making nodes choose different outcomes.

## Activation State

The inbox MUST distinguish unscheduled, scheduled-but-inactive and active states.
Scheduling fixes the first certified L2 height, Espresso scan start and trusted quorum hash, and configures the initial bootstrap batcher; it leaves the certification cursor unset.
Only an accepted bootstrap submission MUST move the inbox into the active state and establish its cursor.
The cursor MUST NOT be read as an active position before that transition.
Installation of the certified implementation MUST NOT activate certification or choose the transition position.
The fixed [inbox proxy address](batch-inbox.md#proxy-address-and-upgrade) MUST remain the same throughout the transition.

## Activation Schedule

The **activation height** is `ACTIVATION_BLOCK_NUMBER`, the first L2 height eligible for certified content; the **activation checkpoint** is its preceding height.
The `admin` MUST schedule activation through an authorized L1 operation fixing `ACTIVATION_BLOCK_NUMBER`, `DEPLOYMENT_HEIGHT`, `INITIAL_QUORUM_HASH` and `bootstrap_batcher`.
`bootstrap_batcher` MUST equal the legacy batcher authorized by the chain's pre-certification configuration when scheduling executes; later registry changes MUST NOT replace this scheduled address.
The contract MUST reject scheduling by any other caller, a second schedule, or scheduling after activation.
`ACTIVATION_BLOCK_NUMBER` MUST exceed the rollup's L2 genesis height, and its timestamp under the [genesis-relative mapping](block-records.md#fields) MUST exceed the scheduling block's timestamp.
The admin MUST choose an activation height beyond every L2 height that legacy derivation can establish from canonical L1 inputs preceding the scheduling transaction.
The contract need not verify this derivation boundary; correct scheduling is an administrative trust premise.
The future-timestamp check MUST NOT be treated as proof of that boundary, because the inherited sequencer-drift rules permit L2 timestamps ahead of their L1 origin.
The contract MUST emit [`ActivationScheduled`](#events) and set `scheduled` without setting `active` or installing a cursor.
The scheduled activation height, Espresso scan start and trusted quorum hash MUST NOT be replaced or cancelled under this interface.
Only the bootstrap batcher address MAY be updated under [Bootstrap Batcher Replacement](#bootstrap-batcher-replacement).

The admin MUST choose `INITIAL_QUORUM_HASH` as the [canonical digest](certifier.md#encodings) of an authentic quorum snapshot of the Espresso network fixed by `ESPRESSO_FINALITY_PROFILE`.
The snapshot MUST govern the epoch containing `DEPLOYMENT_HEIGHT` or an earlier epoch from which valid forward transitions can reach it.
The contract MUST NOT accept a quorum hash other than the scheduled one at bootstrap.
It does not authenticate the administrative trust anchor itself; an authentic choice is a trust premise of [Espresso Verification](certifier.md#espresso-verification).
`DEPLOYMENT_HEIGHT` fixes the scan start so that the bootstrap batcher cannot skip earlier records by choosing a different resume position.
The [publisher](block-records.md#publication) MUST ensure that a record for the activation height is finalized at or after this Espresso start height, republishing it if an earlier attempt finalized below it.
The publisher MUST NOT sign any record fragment for the activation checkpoint height `ACTIVATION_BLOCK_NUMBER - 1`; it MAY publish that record only in a record batch, or not at all, as an [exception to its publication obligation](block-records.md#publication).

## Bootstrap Batcher Replacement

While the inbox is scheduled but inactive, the `admin` MAY replace `bootstrap_batcher` through an authorized L1 operation.
The contract MUST reject replacement by any other caller, before scheduling, after activation, or with the zero address.
A successful replacement MUST update only `bootstrap_batcher` and emit [`BootstrapBatcherUpdated`](#events) with the previous and new addresses; it MUST NOT change the activation height, Espresso scan start, trusted quorum hash, activation status or cursor.
A failed replacement MUST leave state unchanged and emit no replacement event.
Ordinary batcher-registry changes MUST NOT update `bootstrap_batcher`.
Bootstrap authorization MUST use the most recently configured address at the bootstrap transaction's position in canonical L1 order; after a replacement, the previous address MUST NOT retain bootstrap rights unless appointed again.
A node MUST reconstruct these replacements from canonical receipt events in L1 order, including on replay or reorganization.
Replacing the bootstrap batcher MUST NOT change historical legacy sender authorization or permit legacy content at or above the activation height.

## Initial Cursor

The **bootstrap submission** is the first accepted certified submission following the [activation schedule](#activation-schedule).
It MUST use the ordinary [batch commitment](batch-inbox.md#batch-commitment) encoding, EigenDA certificate, certifier signature and verification rules; it requires no separate activation transaction.
Only the currently configured `bootstrap_batcher` MUST be permitted to submit it, replacing ordinary primary/alternative-batcher authorization for this submission.
Its signed `prev_cursor` MUST satisfy the following constraints, replacing comparison with a stored cursor:

| `Cursor` field         | Bootstrap constraint                          |
| ---------------------- | --------------------------------------------- |
| `height`               | Equals `DEPLOYMENT_HEIGHT`                    |
| `index`                | Equals zero                                   |
| `block_number`         | Equals `ACTIVATION_BLOCK_NUMBER - 1`          |
| `epoch_num`            | Supplied at bootstrap; at most `block.number` |
| `frames_digest`        | Equals zero                                   |
| `quorum_snapshot_hash` | Equals `INITIAL_QUORUM_HASH`                  |

`epoch_num` is the preceding-origin anchor for the selected-record scan and the initial exclusive window; it MUST NOT be treated as authentication of the checkpoint's actual derived origin.
The initial cursor MUST NOT authenticate a checkpoint block or state, which a node obtains through [activation derivation](#derivation-transition) or its existing snapshot trust premise.
The first record the scan expects MUST be at `ACTIVATION_BLOCK_NUMBER`; records below it are not required.
This cursor names no consumed record, and [Sync Start](block-records.md#sync-start) needs no activation parameters to start the scan at `ACTIVATION_BLOCK_NUMBER`: it re-selects only when the transaction at the cursor is an authenticated index-0 fragment for `block_number`, and because the publisher signs no fragment for the checkpoint height, the transaction at `(DEPLOYMENT_HEIGHT, 0)` never is one, even when signed bytes are copied into the namespace.
Whatever else is published at that position therefore cannot halt activation; the residual risk of a compromised publisher key signing such a fragment is stated under [block records' security considerations](block-records.md#security-considerations).
The supplied starting quorum MUST match `INITIAL_QUORUM_HASH`, and the core MUST verify all forward transitions needed to authenticate the supplied Espresso blocks.

The contract MUST reject bootstrap before scheduling, or when the activation height's mapped L2 timestamp exceeds the including L1 block's timestamp.
The submission MAY contain only an initial channel fragment or yield no complete block, as an ordinary certified submission may; activation MUST occur upon acceptance, and derivation MUST wait for channel completion before releasing certified blocks.
On success the contract MUST atomically set `active`, store the resulting cursor, emit [`CertificationActivated`](#events) with the signed initial cursor, and then emit the ordinary `CursorAdvanced` event.
A failure MUST leave the schedule, activation status and cursor unchanged and emit neither event.
After activation, every submission MUST use the ordinary stored-cursor matching and batcher-authorization rules; the bootstrap exception MUST NOT be reusable.

`primary_batcher` and `certifier_verifier` MUST be configured before bootstrap acceptance; `alternative_batchers` MAY initially be empty.
Certifier keys MAY be registered before activation under the ordinary registration rules.
The first accepted frame initializes channel ownership and its opening block under [Acceptance](batch-inbox.md#acceptance); no channel state is inherited from legacy derivation.
Until an authorized batcher successfully bootstraps, derivation MUST stop at the activation checkpoint; [batcher replacement](#bootstrap-batcher-replacement) MUST NOT itself activate certification or resume derivation.

## Bootstrap Acceptance

For a decoded ordinary [batch commitment](batch-inbox.md#batch-commitment) `c`, the inactive inbox MUST validate its starting cursor and sender as follows, instead of applying ordinary stored-cursor matching and primary/alternative-batcher authorization:

```python
def bootstrap_initial_cursor(prev, sender, l1_number, l1_timestamp):
    require scheduled and not active
    require sender == bootstrap_batcher
    require timestamp_at(ACTIVATION_BLOCK_NUMBER) <= l1_timestamp
    require prev.height == DEPLOYMENT_HEIGHT
    require prev.index == 0
    require prev.block_number == ACTIVATION_BLOCK_NUMBER - 1
    require prev.epoch_num <= l1_number
    require prev.frames_digest == 0
    require prev.quorum_snapshot_hash == INITIAL_QUORUM_HASH
    return copy(prev)
```

`timestamp_at` MUST use the [genesis-relative mapping](block-records.md#fields), and arithmetic MUST NOT wrap.
All other ordinary acceptance checks MUST remain applicable, including direct submission, canonical calldata, the resulting origin bound, EigenDA verification and the certifier signature over both cursors.
Only the starting cursor and sender checks differ; the bootstrap MUST NOT use a separate commitment format or omit its EigenDA certificate.
The contract MUST accept no further bootstrap after success.

## Derivation Transition

Let `H = ACTIVATION_BLOCK_NUMBER`, the first certified L2 height.
The activation checkpoint is `H - 1`; the height-to-timestamp mapping MUST retain the rollup's genesis baseline.
A node MUST reconstruct the schedule, [bootstrap batcher replacements](#bootstrap-batcher-replacement) and the successful [bootstrap submission](#initial-cursor) from their canonical L1 receipt events.
The bootstrap is identified by `CertificationActivated` and its following `CursorAdvanced`, emitted by the fixed inbox proxy in the same successful transaction; its selection MUST satisfy [Batcher Transaction Selection](../rollup/derivation.md#batcher-transaction-selection).
A node MUST NOT choose how to interpret that transaction from its current derived height, an unaccepted certified-format prefix, or the proxy implementation's installation time.

Before bootstrap, a node MUST apply the legacy selection, resolution, batch-validity and execution rules to preceding inputs, with legacy content restricted to heights below `H`.
It MUST retain ordinary sequencing-window fallback below `H`, but MUST NOT derive height `H` or later before bootstrap.
Upon reaching `H - 1`, it MUST continue examining canonical L1 inputs for the activation event while excluding all further legacy content.
Legacy sender authorization MUST retain its historical configuration; a refused certified attempt MUST NOT activate the transition.

A node MUST process the transition at the bootstrap transaction's position in L1 order, after processing preceding eligible legacy inputs and before processing its certified frames or any subsequent transaction.
Fetching or preprocessing later inputs MUST NOT move the transition ahead of earlier inputs.
A node MUST resolve preceding eligible legacy inputs and process the channels they complete under the inherited resolution, failure, validity and execution rules before deciding which heights are missing.
Locally unavailable data or an unfinished retrieval MUST NOT alone make a height missing; required unavailable inputs MUST retain the inherited waiting or failure outcome.
After those preceding inputs have been resolved under their applicable rules, a node MUST NOT wait for frames included after the transition or for sequencing-window expiry before applying the activation fill.
At the transition it MUST preserve the already-derived prefix, discard remaining legacy frames, channels and decoded batches, and fill every missing height through `H - 1` under [Activation Deposit-Only Fill](#activation-deposit-only-fill).
It MUST then apply certified rules to the bootstrap and subsequent inputs, and MUST NOT admit legacy content after the transition, including content claiming a height below `H`.
The bootstrap's initial `epoch_num` MUST NOT determine replacement origins or authenticate the checkpoint state.

The first submission MAY contain only an initial channel fragment; a node MUST retain it under certified channel assembly and MUST NOT release a block before the channel is complete.
Accepted-payload unavailability MUST retain the [commitment-resolution waiting rules](../rollup/data-availability.md#batch-commitment-resolution), not cause legacy processing to resume or the accepted input to be skipped.

## Channel Boundary

A batcher MUST use legacy encoding for content below `H` and the [certified provider path](../rollup/batch-submission.md#provider-substitution) for content at or above `H`, using the fixed inbox proxy for both.
It MUST NOT construct a channel spanning `H` or submit frames of one channel through both paths; a channel constructed across the boundary MUST be repacked before submission.
Within each side, a batcher MAY choose channel boundaries subject to the applicable format and size rules.
Publication of every legacy batch MUST NOT be a prerequisite for activation; missing heights are handled by the transition's fill.
No later legacy submission can restore ordinary content abandoned at that transition.

Before releasing any batch from an assembled channel, a node MUST inspect every complete decoded batch through the applicable decoder's end-of-input or decoding-error stopping point.
For a certified channel, it MUST first apply [complete-stream validation](../rollup/derivation.md#closed-channel-decoding); a failed stream contributes no decoded batches to this boundary check.
For each represented block whose timestamp maps to an integral height at or above genesis, it MUST use that height for the boundary check; a timestamp outside the mapping MUST retain its normal invalidity treatment and MUST NOT be rounded.
For a legacy [span batch](https://specs.optimism.io/protocol/derivation.html#span-batches), the check MUST cover every represented block.
A node MUST reject the entire channel without releasing any batch if a legacy channel contains a block at or above `H`, or a certified channel contains a block below `H`.
It MUST NOT salvage a wrong-side channel by filtering its batches before this check.
An incomplete or malformed suffix MUST retain the applicable decoder error handling; channels yielding no complete batches contribute no blocks.
Channels passing this check MUST retain all other applicable validity, size and timing rules.

## Activation Deposit-Only Fill

Let `L` be the bootstrap's canonical including L1 block, `P` the actual derived parent, `e` its L1 origin number, and `t = P.timestamp + l2_block_time` the next mapped L2 timestamp.
For each still-missing height below `H`, a node MUST construct the next block on `P` at height `P.number + 1` and timestamp `t`, with no ordinary transactions.
It MUST select canonical origin `e + 1` if that height is representable and at most `L.number` and its timestamp is at most `t`; otherwise it MUST retain origin `e`.
The node MUST apply the inherited deposit, L1-attributes, configuration-update, block-construction and execution rules using that origin and actual parent.
Every replacement origin MUST be at or before `L`; the fill MUST NOT use the bootstrap's supplied origin to choose its L1 inputs.
Required headers or receipts being unavailable MUST cause waiting and retry at the same height, not guessed inputs, skipped deposits or progression past the unresolved height.
A deposit-only replacement failing execution MUST retain the inherited critical-error outcome.
Height and timestamp arithmetic MUST NOT wrap.
The fill MUST NOT wait for sequencing-window expiry, and MUST finish before the node processes certified content or later L1 inputs.
The inbox's [bootstrap timestamp check](#initial-cursor) bounds `H`'s timestamp by `L.timestamp`, so the fill cannot force a future L2 timestamp.

## Replay, Reorganization and Finality

On synchronization or reset, a node MUST recover the canonical schedule, bootstrap batcher replacements and bootstrap position even if its normal L1 replay range begins after their transactions.
Recovery MUST reproduce the activation interpretation and derived outputs of processing the same canonical L1 inputs continuously under these rules.
The choice of replay starting point and the mechanism for locating earlier canonical events are implementation details.
A replay beginning before bootstrap MUST recognize it under the same rules independently of the safe head; a replay beginning after bootstrap MUST already apply certified selection.
A node whose safe head precedes `H - 1` but whose replay begins after bootstrap MUST recover the preceding eligible legacy inputs and reproduce the transition before advancing beyond the checkpoint; it MUST NOT guess missing content.
Replaying the transition MUST fill only heights above the existing safe head and MUST NOT rewrite its established prefix.
An L1 reorganization removing the bootstrap MUST remove its activation effects and apply the inherited rollback and re-derivation rules; moving it MUST recompute the transition at its replacement position.
Bootstrap inclusion MUST NOT independently confer finality; safe and finalized status MUST follow the inherited rules.
Until an authorized batcher successfully bootstraps, derivation MUST stop at `H - 1`.

## Events

The contract MUST emit `ActivationScheduled(uint64 activation_block_number, uint64 espresso_start_height, bytes32 initial_quorum_hash, address bootstrap_batcher)` exactly once, on successful scheduling.
It MUST emit `BootstrapBatcherUpdated(address previous_batcher, address new_batcher)` on every successful bootstrap batcher replacement.
It MUST emit `CertificationActivated(Cursor initial_cursor)` exactly once, immediately before the bootstrap transaction's `CursorAdvanced` event.
All parameters of these events MUST NOT be indexed; their types and order MUST match the declarations above, with `Cursor` encoded in the [Cursor declaration](certifier.md#cursor)'s tuple order.
The activation event MUST NOT be emitted by ordinary submissions, overrides or administrative configuration changes.
Its canonical signature is `CertificationActivated((uint64,uint32,uint64,uint64,bytes32,bytes32))`; the schedule event's is `ActivationScheduled(uint64,uint64,bytes32,address)`.
The replacement event's canonical signature is `BootstrapBatcherUpdated(address,address)`.
Their `topic0` values MUST be the keccak of those strings.
A node MUST reconstruct the [activation boundary](#derivation-transition) from these events in canonical L1 history, not from its local derived height.

The bootstrap MUST also emit the ordinary [`CursorAdvanced`](batch-inbox.md#events) event after `CertificationActivated`, with `prev` equal to the signed initial cursor and `next` equal to the stored resulting cursor.
The activation event's tuple MUST use the [Cursor declaration](certifier.md#cursor)'s field order and types, and the event topic and ABI test vectors MUST cover scheduling, batcher replacement and activation.

## Constants

| Constant                    | Value                                                       | Description                                                                                                                                                       |
| --------------------------- | ----------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ESPRESSO_FINALITY_PROFILE` | Defined under [Certifier Constants](certifier.md#constants) | Espresso network and pinned verification profile to which the administrator's quorum snapshot belongs.                                                            |
| `DEPLOYMENT_HEIGHT`         | TBD                                                         | Espresso start height configured for certification; the `height` of the [initial cursor](#initial-cursor), independent of when the proxy was originally deployed. |
| `ACTIVATION_BLOCK_NUMBER`   | Scheduled by the admin                                      | First certified L2 height; the initial cursor names its preceding height.                                                                                         |
| `INITIAL_QUORUM_HASH`       | Scheduled by the admin                                      | Canonical digest of the trusted Espresso quorum snapshot from which bootstrap verification advances.                                                              |

## Rationale

**Why bootstrap is an ordinary certified submission.**
Using the same commitment, EigenDA certificate and attestation checks avoids a second verification path and a separate activation transaction.
The legacy batcher supplies the origin only when bootstrapping; the administrator fixes the scan position and trust anchor beforehand.

**Why the L1 bootstrap position controls interpretation.**
A scheduled L2 height fixes the content boundary, while the accepted transaction fixes the input transition independently of local node progress.
Missing legacy content is abandoned through a deterministic fill rather than through a requirement that every legacy batch be published first.

**Why the administrator fixes the quorum and scan start.**
A batcher-selected quorum could authenticate fabricated history under attacker-controlled keys, and a batcher-selected scan start could omit an earlier selected record.
A known authentic snapshot suffices; the anchor need not be Espresso's genesis quorum because the certifier authenticates forward transitions from it.

## Security Considerations

**Bootstrap does not prove the checkpoint.**
The bootstrap signature authenticates certified content from the activation height, not the preceding legacy state.
Missing legacy heights are filled deterministically by [activation derivation](#derivation-transition); withholding their batches loses ordinary content without changing the transition rule.
The supplied initial origin affects selected-record validity and the exclusive window, but does not select the origins or state of those replacement blocks.
A false administrative quorum anchor instead permits fabricated Espresso history to pass the enclave's genuine signature checks; bootstrap therefore matches the scheduled anchor rather than accepting a batcher-selected one.

**An unspecified initial cursor has no safe default.**
The continuity chain is an induction and the [initial cursor](#initial-cursor) is its base case, so leaving the base case to implementers leaves the whole chain undetermined.
Two deployments of this specification whose cursors differ in even one field are mutually unusable: the certifier signs over `prev_cursor`, so an attestation produced against one deployment's cursor fails the digest check at the other, and the divergence surfaces as every submission being rejected rather than as any detectable disagreement about content.
An initial `index` above `0` would place the first payloads of `DEPLOYMENT_HEIGHT` behind the resume point, so if one of them carried the first post-checkpoint record, that record would never be found and every call would expect the same height forever.
Certification would make no progress from the activation height, and ordinary submissions could not restore the skipped record to the scan.

**Scheduling cannot revise an established legacy prefix.**
The contract's future-timestamp check does not establish that the chosen height is beyond legacy derivation progress.
A schedule below an already-established legacy head violates the administrative scheduling premise and can make continuous processing and replay classify earlier content differently.

**Ordered inputs and replay.**
Prefetching the bootstrap ahead of eligible legacy data can replace content with deposit-only blocks on one node while another preserves it.
A reset that forgets the canonical activation position can reinterpret the same submission under different rules.
An unavailable accepted bootstrap payload stalls certified derivation rather than authorizing a return to legacy processing.

**Progress depends on bootstrap.**
A missing bootstrap leaves derivation at the last legacy height.
The administrator can replace a failed bootstrap batcher while the inbox remains inactive, but recovery depends on administrative action and a replacement able to submit a valid certified commitment.
Replacement does not bound activation latency or bypass unavailable certifier or EigenDA services; deposits cannot progress beyond the checkpoint until bootstrap succeeds.
Allowing replacement after activation would reopen the one-time bootstrap authority, so the inactive-state restriction is essential.

## Spec History

Block certification activation is not active on any network.
