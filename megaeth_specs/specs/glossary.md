---
description: Canonical definitions of MegaETH-specific protocol terms.
---

# Glossary

This page defines MegaETH-specific protocol terms and links to the pages that specify their behavior.
Entries marked unstable describe block certification, which is not activated on any MegaETH network.

## Block record

_(Block certification, unstable)_ The per-block content the publisher publishes to Espresso for each L2 block, carrying that block's ordinary transactions in full together with the L1 origin height they attach to.

See [Block Records](block-certification/block-records.md).

## Publisher

_(Block certification, unstable)_ The block certification role authorized to authenticate and publish one block record per L2 block into the Espresso namespace, performed by the sequencer's operator and bound by the publication rules rather than by the operator's identity.

See [Publication](block-certification/block-records.md#publication).

## Record batch

_(Block certification, unstable)_ One signed Espresso namespace transaction containing one or more complete unsigned block records, each authorized by the key governing its own height.

See [Record Batch](block-certification/block-records.md#record-batch).

## Record fragment

_(Block certification, unstable)_ One signed Espresso namespace transaction containing part of one encoded unsigned block record, with its height, attempt nonce, record hash, fragment index and count.

See [Record Fragment](block-certification/block-records.md#record-fragment).

## Record position

_(Block certification, unstable)_ The pair of an Espresso block height and a namespace-transaction index; a selected record's position is the start of its span — the record batch containing it, or the index-0 fragment of the attempt that completed it — from which a resumed scan continues, re-selecting it first only when it was fragmented.

See [Record Position](block-certification/block-records.md#record-position).

## Record selection pipeline

_(Block certification, unstable)_ The scan that computes the selected record sequence from a cursor: Sync Start initializes it once, the Publication Queue decodes and authenticates namespace transactions, the Record Queue produces complete records for the next expected height, and the Record Selector checks their content and origin.

See [Record Selection Pipeline](block-certification/block-records.md#record-selection-pipeline).

## Selected record

_(Block certification, unstable)_ The first valid record for an expected L2 height in Espresso's finalization order, as chosen by the record scan; the selected records of a scan segment in height order form the selected record sequence.

See [Canonical Record Sequence](block-certification/block-records.md#canonical-record-sequence).

## Canonical record

_(Block certification, unstable)_ A selected record through which an accepted L1 inbox submission has advanced the L2 cursor; selection and attestation alone do not confer this status.

See [Canonical Record Sequence](block-certification/block-records.md#canonical-record-sequence).

## Certified

_(Block certification, unstable)_ A full node's local determination, after it has verified a height's selected record, L1-dependent validity and execution, that the height's content will settle without deposit-only fallback under the full-node guarantee's conditions.

See [The Guarantee](block-certification/full-node.md#the-guarantee).

## Resolved height

_(Block certification, unstable)_ An L2 height for which the scan has selected a record, so that later records for it are ignored within that scan segment; resolution does not make the record canonical.

See [Canonicality](block-certification/block-records.md#canonicality).

## Block certifier

_(Block certification, unstable)_ The component that gates settlement on certification by checking consecutive channel frames against the selected record sequence and binding their submission payload to a data availability commitment under an attestation.

See [Block Certifier](block-certification/certifier.md).

## Certifier core

_(Block certification, unstable)_ The attested pure function inside the block certifier's enclave that performs the verification procedure and signs the resulting cursor transition, with no I/O and no state between calls.

See [Certifier Core](block-certification/certifier.md#certifier-core).

## Certifier host

_(Block certification, unstable)_ The untrusted process around the certifier core that assembles the core's inputs, disperses the submission payload to EigenDA and returns the commitment the batcher submits, run by an operator trusted for nothing unless it can change the release policy of a [sealed](block-certification/certifier.md#key-sealing) certifier key.

See [Block Certifier](block-certification/certifier.md).

## `Cursor`

_(Block certification, unstable)_ The structure recording the Espresso scan resume point, preceding L2 height and origin anchor, current channel frame digest and anchored quorum digest.
A certifier output proposes a cursor transition; inbox acceptance commits it, while bootstrap and emergency overrides establish anchors under their own rules.
The recorded height and origin do not independently authenticate a derived block or its state.

See [Cursor](block-certification/certifier.md#cursor).

## Batch commitment

_(Block certification, unstable)_ The structure bundling the certifier's output cursors, commitment and signature with the data availability certificate the certifier verified.
A certified-format batch transaction carries its canonical ABI encoding after the batch-transaction prefix.

See [Batch Commitment](block-certification/batch-inbox.md#batch-commitment).

## Activation checkpoint

_(Block certification, unstable)_ The L2 height immediately preceding the scheduled first certified height, whose state is determined by legacy derivation and activation deposit-only fill.
The bootstrap starting cursor uses this height, but its supplied preceding-origin anchor does not authenticate the checkpoint's actual derived origin or state.

See [Initial Cursor](block-certification/activation.md#initial-cursor).

## Exclusive window

_(Block certification, unstable)_ The period, measured in L1 blocks from the accepted cursor's `epoch_num`, during which only the primary batcher may submit to the L1 batch inbox.

See [Exclusive Window](block-certification/batch-inbox.md#exclusive-window).

## Deposit-only fallback

_(Block certification, unstable)_ The OP Stack derivation rule that fills a height with the empty batch derivation produces when that height's sequencing window elapses without a batch.

See the OP Stack [batch queue](https://specs.optimism.io/protocol/derivation.html#batch-queue) and [The Certification Claim](block-certification/overview.md#the-certification-claim).

## Deposit-only block

_(Block certification, unstable)_ A block carrying only the deposit transactions derived from L1, which derivation produces for a height whose batch never arrives or is rejected.

See [The Certification Claim](block-certification/overview.md#the-certification-claim).

## Legacy rules and certified rules

_(Block certification, unstable)_ The derivation rule sets separated by the accepted bootstrap transaction: preceding legacy inputs may supply heights below the scheduled activation height, and the bootstrap and later certified inputs may supply heights at or above it.
Missing legacy heights are filled as deposit-only at the transition.

See [Activation and Channel Boundary](block-certification/activation.md#derivation-transition).

## Emergency cursor override

_(Block certification, unstable)_ The admin-only operation that sets the L1 batch inbox's cursor directly to a strictly greater height, bypassing the acceptance checks and abandoning any open channel and the skipped heights.

See [Emergency Cursor Override](block-certification/batch-inbox.md#emergency-cursor-override).

## Lossless recovery

_(Block certification, unstable)_ The recovery outcome in which each affected height settles the ordinary transaction list of its selected record at that same height, byte-identically and in the same order.

See [Recovery Outcomes](block-certification/recovery.md#recovery-outcomes).
