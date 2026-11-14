---
description: Protocol specification overview — component map, normativity boundary, and the OP Stack baseline convention for the MegaETH L2 node.
---

# Protocol

{% hint style="warning" %}
[Stateless Validation](stateless-validation.md) is still a stub; every other page in this section carries its normative content.
{% endhint %}

This section specifies the MegaETH L2 node itself: everything an implementation must do to produce, propagate, and validate MegaETH blocks.
It targets execution-client and validator implementers.

The subsystems in this section are MegaETH-native and are specified here in full:

- **[Blocks and Mini-Blocks](blocks.md)** — the structure of EVM blocks and mini-blocks, transaction and receipt roots, header signing, and timestamp semantics.
- **[Sequencing](sequencing.md)** — block production rules: sequencer authority, and transaction admission and ordering.
- **[Transactions](transactions.md)** — the transaction types that may legitimately appear in a MegaETH block: standard and deposited transactions inherited from the OP Stack, and the MegaETH-native mega system transaction, including its signer authority and processing rules.
- **[State (SALT)](salt/overview.md)** — the SALT state representation: the bucket model, cryptographic commitments, state root computation, and the execution witness format.
- **[State Sync](state-sync.md)** — the protocol by which nodes obtain chain data from the sequencer without re-executing it: the RLPx sub-protocol it binds to, the streamed payload types and their encodings, payload authentication, mini-block and EVM block assembly and validity rules, and the pull-based backfill, reorg-resolution, and state-snapshot bootstrap requests.
- **[Stateless Validation](stateless-validation.md)** — witness-based block verification: witness consumption, re-execution, and validator correctness rules.

## Normativity Boundary

This specification defines verifiable behavior: whatever affects whether a node produces correct outputs given the same inputs.
Mechanisms that only affect availability, performance, or operations — sequencer high-availability machinery, propagation topology and peer selection, deployment architecture, and RPC gateway behavior — are implementation concerns and are not specified here.
For example, the rules governing who may sign a mini-block are normative; the failover mechanism that decides which machine holds the signing key is not.
A transport is normative where two implementations must speak it to exchange data at all, and not otherwise: [state sync](state-sync.md) binds its payloads to a specific [RLPx](https://github.com/ethereum/devp2p/blob/master/rlpx.md) sub-protocol, so that binding — its capability name, version, message ids, and framing — is normative alongside the payload encodings and validity rules, while the topology that relays those payloads between nodes remains an implementation concern.
