---
description: The EigenDA-based data availability layer — how an accepted batch commitment resolves to the frame bytes derivation ingests; dispersal, certificate format, and retrieval failure semantics are still to be written.
spec: TBD
---

# Data Availability

{% hint style="warning" %}
This page is incomplete.
It currently specifies only how [derivation](derivation.md) resolves an accepted [batch commitment](../block-certification/batch-inbox.md#batch-commitment) to frame bytes, which is unstable and not activated on any network.
Dispersal to EigenDA, the certificate format, retrieval beyond the check below, and behavior when the data availability layer is unavailable have not been written.
{% endhint %}

MegaETH posts batch data to [EigenDA](https://docs.eigencloud.xyz/eigenda/core-concepts/overview) and places on L1 only a commitment to it.
This page specifies how a commitment the [L1 batch inbox](../block-certification/batch-inbox.md) accepted resolves to the bytes derivation ingests.

## Motivation

The OP Stack [alt-DA](https://specs.optimism.io/experimental/alt-da.html) pipeline reads a commitment from a batcher transaction and asks a data availability provider for the bytes behind it, trusting the provider to return the bytes the commitment names.
Under [block certification](../block-certification/overview.md) the bytes behind a commitment are also what the [block certifier](../block-certification/certifier.md) attested, so the step from commitment to bytes is where the certification gate either holds or fails.
A node that ingested bytes other than the attested ones would settle content no attestation covers, with the inbox's `cursor` advancing exactly as it does for an honest submission.
This page fixes that step so that every node ingests the same bytes, and only the attested ones.

## Batch Commitment Resolution

For each transaction selected under the [certified rules](derivation.md#batcher-transaction-selection), a node MUST resolve the calldata to frame bytes as follows, and MUST NOT ingest any other bytes for that transaction.
Under the legacy rules, transactions retain their pre-certification data-resolution rules under [Activation and Channel Boundary](../block-certification/activation.md#derivation-transition).

1. The calldata is [`BATCH_TX_PREFIX`](../block-certification/batch-inbox.md#constants) followed by `abi.encode(c)` for a [batch commitment](../block-certification/batch-inbox.md#batch-commitment) `c`, which the inbox verified before emitting the acceptance event; the node's commitment decoder dispatches on the prefix's second byte, [`CERTIFIED_COMMITMENT_TYPE`](../block-certification/batch-inbox.md#constants), decodes `c` from the bytes after the prefix, and MUST NOT decode calldata carrying any other commitment type as a batch commitment.
2. The node retrieves from EigenDA the payload that `c.eigenda_cert` certifies.
3. The node recomputes that payload's G1 KZG commitment and length under [`EIGENDA_CODEC`](../block-certification/certifier.md#constants), as the certifier's [`payload_matches`](../block-certification/certifier.md#verification-procedure) does, and MUST NOT ingest the payload unless both equal those in the blob commitment of `c.eigenda_cert`, whose hash the inbox established equals `c.eigenda_commitment`.
4. The payload is a [submission payload](../block-certification/certifier.md#encodings) — one derivation version byte followed by one or more consecutive frames of the same channel — and the node MUST pass it intact to baseline frame parsing, ingesting every frame in wire order exactly once for that transaction.
   The prefix witness supplied to the certifier is not part of this payload and MUST NOT be ingested again.

A node that cannot retrieve the payload, or retrieves one whose G1 commitment or length differs, MUST NOT substitute other bytes for it and MUST NOT derive past that transaction until a matching payload is obtained, for the reason the [derivation page](derivation.md#security-considerations) gives for receipts: skipping would leave the inbox's `cursor` past a frame the chain never received.
The [retention assumption](../block-certification/overview.md#confirmation-stages) is what makes such a payload obtainable; whether a node may ever stop waiting, and what it does then, is not yet specified.

## Constants

This page defines no constants of its own.
[`BATCH_TX_PREFIX`](../block-certification/batch-inbox.md#constants) and [`EIGENDA_CODEC`](../block-certification/certifier.md#constants) are defined on the pages that own them.

## Rationale

**Why the node re-verifies the commitment rather than trusting the certificate.**
The certifier verified `c.eigenda_cert`, the inbox authenticated its operator set and established that its commitment equals `c.eigenda_commitment`, and the certifier's signature covers both the certificate and the commitment; none of that establishes that the bytes a retrieval returns are the bytes committed to.
A retrieval path is an unauthenticated service from the node's point of view, so the commitment check is what carries the certifier's attestation from the commitment on L1 to the bytes in the node's memory.
KZG commitments are binding, so under `EIGENDA_CODEC` and a given length the G1 commitment determines the payload bytes; the certifier checked the same G1 commitment and length against the payload it attested, so a payload that passes the check is the one the certifier saw.

## Security Considerations

**Assets and threat model.** This section protects one property: that the bytes a node ingests for an accepted commitment are the bytes the certifier attested, so that every node ingests the same bytes and only those.
The node relies on the checks upstream of it — the certifier core's comparison against selected records and its verification of `c.eigenda_cert`, and the inbox's authentication of that attestation — but those are statements about `c.eigenda_commitment` and the certificate, not about any payload a retrieval returns.
EigenDA is trusted for the availability of certified payloads, which is what makes a matching payload obtainable at all.
The retrieval path is untrusted: it is an unauthenticated service from the node's point of view, and the commitment check is the only step that separates attested bytes from any others.
The [Roles and Trust](../block-certification/overview.md#roles-and-trust) table records these premises.

**Bytes ingested must equal bytes attested, and the commitment check is the only place that holds.**
Every earlier check — the certifier's comparison against Espresso records and its certificate verification, the inbox's signature and registry checks — is stated over `c.eigenda_commitment` and the certificate, not over bytes a node retrieves later.
A node that skipped the check, or recomputed the G1 commitment and length under a codec other than `EIGENDA_CODEC`, would accept whatever a retrieval returned, and a compromised or faulty retrieval path could then settle content no certifier attested while L1 looked exactly as it does for an honest submission.
The check fails closed: a node that cannot obtain a payload matching the commitment ingests nothing and does not advance, which is the liveness cost the [Certification Claim](../block-certification/overview.md#the-certification-claim) prefers over settling content no selected record carries.

## Spec History

No data availability rules have been activated on any network.
