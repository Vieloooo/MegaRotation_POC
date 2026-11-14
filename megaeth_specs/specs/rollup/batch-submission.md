---
description: Batcher behavior deltas over the OP Stack, including the block certifier as alt-DA provider under block certification.
spec: TBD
---

# Batch Submission

{% hint style="warning" %}
This page is incomplete.
It currently specifies only the deltas [block certification](../block-certification/overview.md) introduces, which are unstable and not activated on any network.
The remaining deltas over the [OP Stack batcher specification](https://specs.optimism.io/protocol/batcher.html) — the validation gate bounding submission to validated L2 blocks, and MegaETH's batch size limits — have not been written.
{% endhint %}

The MegaETH batcher is the [OP Stack batcher](https://specs.optimism.io/protocol/batcher.html) operating in the [alt-DA](https://specs.optimism.io/experimental/alt-da.html) mode it already supports, with the [block certifier](../block-certification/certifier.md) as its configured provider in place of a data availability layer addressed directly.
It uses the baseline channel and frame formats, subject to the activation boundary below; the provider performs certification verification.
This page specifies that substitution and the constraints certification places on channel construction.

## Motivation

Certification requires every frame reaching L1 to have been checked against the records selected from Espresso's finalized history, and requires that check to be performed by something other than the party that benefits from skipping it.
A batcher that performed the check itself would be checking its own output, so the check has to sit outside it.

The alt-DA provider interface is already exactly this shape: the baseline batcher hands a payload of frame data to a provider and receives a commitment it posts to L1, without interpreting either.
Placing the certifier at that interface keeps certification verification outside the batcher; channel construction still has to respect the activation boundary.

Activation scheduling, the bootstrap submission and channel boundaries are specified exclusively by [Block Certification Activation](../block-certification/activation.md).

## Provider Substitution

This section and [Failure Handling](#failure-handling) apply to channels containing post-checkpoint blocks.

A batcher MUST operate in alt-DA mode with the [block certifier](../block-certification/certifier.md) as its provider.
It MUST submit each [submission payload](../block-certification/certifier.md#encodings) to the certifier and MUST post the returned [batch commitment](../block-certification/batch-inbox.md#batch-commitment) to the [L1 batch inbox](../block-certification/batch-inbox.md) as the calldata of a batch transaction.
Each payload MUST contain one or more consecutive new frames from one channel, encoded with a single derivation version byte.
A batcher MUST NOT combine frames from different channels in one payload, post a payload without a returned commitment, or disperse it to a data availability layer directly.

A submission MAY contain a single frame, a consecutive group, or an entire channel, within the pinned frame and DA payload limits.
One attestation, EigenDA certificate and batch transaction cover the whole payload; a batcher MUST NOT split or regroup its frames while reusing that commitment.
When a channel spans submissions, the inbox's accepted [`Cursor`](../block-certification/certifier.md#cursor) binds its accepted prefix, which the core receives separately as a witness and which is not dispersed again as part of the new payload.

A batcher MUST construct channels using the pinned [`CHANNEL_PARAMS`](../block-certification/certifier.md#constants): the compression format, the maximum frame size, and the singular batch format — a batcher MUST NOT encode blocks as span batches.
The batcher constructs under the same pinned parameters the certifier decodes under, which is what makes the [certifier core](../block-certification/certifier.md#certifier-core)'s [decode of the batcher's frames](../block-certification/certifier.md#verification-procedure) well defined.
The batcher MAY choose its compression level, provided the resulting channels and frames satisfy the pinned decoding rules and all applicable size limits.
Pinning these fixes what derivation must be able to decode and keeps frames within size limits; it does not require any two batchers to produce identical bytes for the same blocks, and nothing in this specification compares one batcher's output against another's.

A batcher MAY partition the post-checkpoint selected block sequence into channels however it chooses, subject to [Activation and Channel Construction](../block-certification/activation.md#channel-boundary), and MAY close a channel at any point.
No rule requires reproducing another batcher's packing.

## Failure Handling

When the certifier returns no commitment for a payload, the batcher MUST NOT post it to L1 or retry through another path.
Failure of any frame check rejects the whole call; no prefix of that payload is authorized for submission.
A batcher SHOULD retry the payload later when Espresso has not yet finalized the records covering its content.

The two causes are deliberately indistinguishable to everything downstream.
Whether the payload's content permanently contradicts the records selected from Espresso's finalized history or merely has no selected record yet, the observable result is identical: no valid batch reaches the inbox, and if none reaches it before the [sequencing window](https://specs.optimism.io/protocol/derivation.html#overview) elapses, derivation substitutes a deposit-only block for the affected heights by the ordinary rule.

A batcher that has fallen behind MUST NOT adjust what it submits on account of how late it is.
It MUST continue submitting the content the inbox's current [`cursor`](../block-certification/batch-inbox.md#contract-state) expects next, however far past that content's own sequencing window it has fallen, and MUST NOT attempt to determine whether that window has elapsed.
A batcher whose batch transaction reverts MUST re-read the inbox's `cursor` and build its next submission against it: a continuation while its own channel is open and within [`CHANNEL_TIMEOUT`](../block-certification/batch-inbox.md#constants), and a fresh channel from frame 0 once no channel is open or the open one has timed out.
Lateness is decided solely by [derivation](derivation.md#unchanged-behavior); a batcher that skipped ahead on the assumption that a height was already lost would leave a gap the cursor can never pass.

## Rationale

**Why certification is a provider substitution rather than a batcher modification.**
Every check certification introduces is a function of the frame bytes and public data, and none of it requires anything the batcher holds.
Performing those checks at the alt-DA interface allows any batcher that respects the channel constraints to use the block certifier as provider through the same interface.
An alternative batcher need not reproduce the primary batcher's packing.

**Why a stalled batcher resumes rather than skips.**
The cursor advances strictly in sequence, so content cannot be skipped: a batcher that jumped ahead to the current height would produce a channel whose first block is not the one the cursor expects, and the certifier would reject it, permanently.
Resuming from where the cursor stands is therefore the only behavior that makes progress, and it costs nothing when the skipped heights are already lost, since derivation will have filled them with deposit-only blocks whether or not a batch for them eventually arrives.

## Security Considerations

**Bypassing the provider would defeat certification entirely.**
The certifier is the only component that checks a frame against the records selected from Espresso's finalized history, so a batcher that dispersed a frame directly and posted the resulting commitment itself would produce a submission carrying no attestation.
The [L1 batch inbox](../block-certification/batch-inbox.md#acceptance) rejects such a submission, which bounds the damage — but only in combination with derivation's [exclusion of reverted transactions](derivation.md#batcher-transaction-selection), without which the rejected calldata would be ingested anyway.

## Spec History

No batch submission deltas have been activated on any network.
