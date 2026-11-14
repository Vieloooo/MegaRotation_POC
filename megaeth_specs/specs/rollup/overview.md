---
description: Rollup and settlement specification overview — MegaETH's behavioral deltas over the OP Stack derivation, data availability, batching, bridging, and fault proof specifications.
---

# Rollup & Settlement

{% hint style="warning" %}
This section is under construction — pages are stubs until their normative content is written.
{% endhint %}

This section specifies the L1↔L2 pipeline: how MegaETH blocks are derived from L1 data, how batch data is made available, and how the chain settles to Ethereum.
It targets settlement and bridge implementers and auditors.

MegaETH inherits the OP Stack settlement architecture.
Pages in this section are **delta specifications**: they define only MegaETH's behavioral differences over the corresponding [OP Stack specification](https://specs.optimism.io/), which each page links as its baseline.

- **[Derivation](derivation.md)** — deltas over OP Stack derivation, including data-availability certificate handling and batcher transaction selection under [block certification](../block-certification/overview.md).
- **[Data Availability](data-availability.md)** — the EigenDA-based data availability layer: certificate format, dispersal and retrieval, and failure semantics.
- **[Batch Submission](batch-submission.md)** — batcher behavior deltas, including the pre-submission validation gate and the [block certifier](../block-certification/certifier.md) as alt-DA provider.
- **[Deposits and Force Inclusion](deposits.md)** — L1-initiated transactions and their inclusion guarantees.
- **[Withdrawals](withdrawals.md)** — L2-to-L1 message passing and proving.
- **[L1 Contracts](l1-contracts.md)** — deltas to the OP Stack L1 contract set.
- **[Fault Proofs](fault-proofs.md)** — the ZK fault proof dispute game replacing the OP Stack interactive bisection game.

The L1 batch inbox this pipeline reads from is a fixed-address proxy, whose certification implementation is specified under [Block Certification](../block-certification/batch-inbox.md) along with the rules governing certified submissions.
