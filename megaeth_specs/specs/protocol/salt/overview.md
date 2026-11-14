---
description: Overview of the SALT state representation — the bucket model, cryptographic commitments, state root computation, and the execution witness.
spec: Rex5
---

# State (SALT)

SALT (Small Authentication Large Trie) is MegaETH's state representation, replacing Ethereum's Merkle-Patricia trie with a fixed, shallow, hash-table-backed structure whose commitments fit in memory and update without random disk access.
Every account and storage slot a MegaETH node holds is SALT state; a node's [state root](trie.md#state-root) is derived from the SALT trie's root commitment, and every [state sync](../state-sync.md) payload that carries a state transition carries it as a [SALT delta](trie.md#salt-deltas) paired with the fragment's [`buckets_to_canonicalize`](buckets.md#canonicalization) signal.

## Specifications

- [Bucket Model](buckets.md) — key mapping, the SHI hash table algorithm, bucket metadata and growth, canonicalization, and the bucket range encoding.
- [SALT Trie](trie.md) — the vector commitment scheme, main trie and bucket subtree commitments, state root computation, and SALT delta encoding.
- [Execution Witness](witness.md) — witness contents, membership and non-membership proofs, and the multi-point proof and transcript a verifier checks.

What SALT does not cover — how a witness's claims are used to re-execute a block and check the resulting post-state root against a header — is [Stateless Validation](../stateless-validation.md)'s scope.
How SALT state is streamed between nodes, including the [State-Change Fragment](../state-sync.md#state-change-fragment) that carries a mini-block's SALT delta and the bucket-range request/response used to bootstrap a node with no state, is [State Sync](../state-sync.md)'s scope; these pages define the byte formats [State Sync](../state-sync.md) references but not the transport or delivery mechanics those formats travel over.

## Spec History

No spec-gated changes to the behavior of this section have been identified; verify against [Hardforks and Specs](../../hardfork-spec.md) before treating this as exhaustive.
