---
description: MegaETH block structure — EVM blocks and mini-blocks, mini-block header fields, transaction and receipt roots, header signing, and timestamp semantics.
spec: Rex5
---

# Blocks and Mini-Blocks

MegaETH produces two block types at two different cadences: **[mini-blocks](https://docs.megaeth.com/spec/glossary#mini-block)**, sealed roughly every 10 milliseconds and the minimal unit the [state sync](state-sync.md) protocol synchronizes, and **[EVM blocks](https://docs.megaeth.com/spec/glossary#evm-block)**, sealed roughly every second and structurally identical to a standard Ethereum block.
This page specifies the structure of both, the mini-block header signature, and the rules binding a set of mini-blocks to their containing EVM block.

## Motivation

A standard Ethereum block header is designed for one-second-or-slower block times: it is over 500 bytes, carries multiple Merkle roots optimized for light-client proof succinctness rather than header compactness, and encodes timestamps at one-second resolution.

Streaming a header of that size at mini-block cadence does not scale.
At 100 mini-blocks per second, a standard header alone would cost more than 1.57 TB per year of bandwidth and storage on every node that consumes the mini-block stream — a burden disproportionate to what a lightweight, bandwidth-constrained consumer of realtime data needs.
A format designed for realtime propagation needs a smaller header, timestamp resolution matched to sub-block-time confirmation, and a way to make the [sequencer](https://specs.optimism.io/glossary.html#sequencer)'s per-mini-block commitment independently verifiable without waiting for the containing EVM block.

## Block Hierarchy

A MegaETH node MUST produce mini-blocks and EVM blocks as two distinct, nested block types.
Every EVM block MUST be composed of one or more mini-blocks: mini-blocks are the minimal unit the [state sync](state-sync.md) protocol synchronizes, so an EVM block with zero mini-blocks would be unreachable through that protocol.

- Every transaction MUST appear in exactly one mini-block and exactly one EVM block.
- The transactions of a mini-block MUST NOT span more than one EVM block.
- Mini-blocks MUST be ordered within their containing EVM block by a zero-based index that resets to `0` at the start of each new EVM block.
- Mini-blocks MUST additionally be ordered by a chain-wide mini-block number that increases by exactly `1` for every mini-block ever produced and never resets.

A mini-block is therefore identified either by the pair `(block_number, index)`, scoped to its containing EVM block, or by its chain-wide `number` alone; the two are related (both increase by `1` together) but are not the same value, and `number` is not derivable from `(block_number, index)` without external context.

## EVM Block

An EVM block's header and body follow the standard Ethereum block format inherited via the [OP Stack execution engine specification](https://specs.optimism.io/protocol/exec-engine.html), unmodified by MegaETH at the structural level.

- An EVM block's transaction list MUST equal the ordered concatenation, in ascending `index` order, of the transaction lists of every mini-block it contains.
- An EVM block's `timestamp` MUST equal the `block_timestamp` field of every mini-block it contains.
- An EVM block's transactions and receipts roots MUST be computed over that concatenated transaction and receipt list, following the standard Ethereum block root construction; they are not required to equal, and are not derived by combining, any individual mini-block's `tx_root` or `receipt_root`.

## Mini-Block Header

A mini-block header is a flat structure of eight fields.
It contains no reference to a preceding mini-block, no state root, and no bloom filter.

| Field             | Type      | Description                                                                                                  |
| ----------------- | --------- | ------------------------------------------------------------------------------------------------------------ |
| `block_number`    | `uint64`  | Number of the EVM block this mini-block belongs to.                                                          |
| `block_timestamp` | `uint64`  | Timestamp of the containing EVM block, in seconds. MUST equal that EVM block header's `timestamp`.           |
| `index`           | `uint64`  | Position of this mini-block within its containing EVM block. `0` for the first mini-block of each EVM block. |
| `number`          | `uint64`  | Chain-wide mini-block number. Increases by `1` for every mini-block; never resets.                           |
| `timestamp`       | `uint64`  | Timestamp of this mini-block. See [Mini-Block Timestamps](#mini-block-timestamps) for unit and precision.    |
| `gas_used`        | `uint64`  | Sum of gas used by every transaction in this mini-block (see [Mini-Block Body](#mini-block-body)).           |
| `tx_root`         | `bytes32` | Root of the mini-block's transaction trie. See [Mini-Block Body](#mini-block-body).                          |
| `receipt_root`    | `bytes32` | Root of the mini-block's receipt trie. See [Mini-Block Body](#mini-block-body).                              |

A node MUST RLP-encode the header as:

```
rlp([block_number, block_timestamp, index, number, timestamp, gas_used, tx_root, receipt_root])
```

where:

- `block_number`, `block_timestamp`, `index`, `number`, `timestamp`, and `gas_used` are each encoded as a standard [RLP](https://ethereum.org/en/developers/docs/data-structures-and-encoding/rlp/) integer: the minimal-length big-endian byte string with no leading zero bytes, per ordinary RLP integer encoding.
  This applies to `timestamp` regardless of which unit is active (see [Mini-Block Timestamps](#mini-block-timestamps)); the field is a bare `uint64` and carries no unit tag.
- `tx_root` and `receipt_root` are each encoded as a fixed 32-byte RLP string, never integer-shortened even when the root happens to have leading zero bytes.

The encoding is exactly this flat list of eight items: no field is reordered, omitted, or wrapped in a nested list, and no version byte or signature is included.
The mini-block's signature (see [Mini-Block Header Signing](#mini-block-header-signing)) is carried alongside the header, never inside this encoding.

**Example.** A header with `block_number = 42`, `block_timestamp = 1,700,000,000`, `index = 3`, `number = 1000`, `timestamp = 1,700,000,000,123,456`, `gas_used = 21,000`, `tx_root = 0xaa` repeated 32 times, and `receipt_root = 0xbb` repeated 32 times encodes as:

| Bytes              | Meaning                                                    |
| ------------------ | ---------------------------------------------------------- |
| `f857`             | List header: long list, 0x57 (87) bytes of payload follow. |
| `2a`               | `block_number` = 42.                                       |
| `846553f100`       | `block_timestamp` = 1,700,000,000.                         |
| `03`               | `index` = 3.                                               |
| `8203e8`           | `number` = 1000.                                           |
| `87060a2418202240` | `timestamp` = 1,700,000,000,123,456.                       |
| `825208`           | `gas_used` = 21,000.                                       |
| `a0` + 32 × `aa`   | `tx_root`.                                                 |
| `a0` + 32 × `bb`   | `receipt_root`.                                            |

A mini-block header's absence of a predecessor reference is intentional: it is an ephemeral data structure used to stream the containing EVM block at fine granularity, not a self-contained root of trust.
Its position `(block_number, index)` and chain-wide `number` are established by the containing EVM block and by the [state sync](state-sync.md) protocol that delivers it, not by a hash chain inside the header.

## Mini-Block Body

A mini-block's body is the ordered list of transactions executed in it and the receipts produced by that execution.

- `tx_root` MUST be the root of an ordered Merkle Patricia Trie over the mini-block's transactions, each transaction [EIP-2718](https://eips.ethereum.org/EIPS/eip-2718)-encoded and keyed by its RLP-encoded index within the mini-block — the same construction used for a standard Ethereum block's transactions root.
- `receipt_root` MUST be the root of an ordered Merkle Patricia Trie over the mini-block's receipts, each receipt [EIP-2718](https://eips.ethereum.org/EIPS/eip-2718)-encoded with its bloom filter and keyed by its RLP-encoded index within the mini-block — the same construction used for a standard Ethereum block's receipts root.
- `gas_used` MUST equal the sum, over every transaction in the mini-block, of the total gas the transaction consumed.
  This total includes both [compute gas](https://docs.megaeth.com/spec/glossary#compute-gas) and [storage gas](https://docs.megaeth.com/spec/glossary#storage-gas); the two dimensions are not reported separately at the mini-block level.

A mini-block's gas limit is not independently enforced; only the containing EVM block's gas limit bounds the cumulative gas used across all of its mini-blocks.
Beyond the one-or-more minimum in [Block Hierarchy](#block-hierarchy), how many mini-blocks an EVM block is split into, when each one is sealed, and how much content it includes are not specified by this specification at all; see [Sequencing's Block-Production Mechanics](sequencing.md#block-production-mechanics) for why this is deliberately left to sequencer implementation policy.

## Mini-Block Header Signing

A node MUST compute a mini-block's signing digest as `keccak256(rlp(header))`, over all eight fields of the [mini-block header](#mini-block-header).

From [Rex5](../upgrades/rex5.md) onward, a node MUST sign this digest with the active sequencer key using a recoverable secp256k1 ECDSA signature, with no message prefix, and MUST attach the signature to the mini-block as `r`, `s`, and a recovery identifier (`yParity`).

The active sequencer key for a mini-block with a given `block_number` is the key returned by `sequencerAt(block_number)` on the [SequencerRegistry](../system-contracts/sequencer-registry.md) system contract.
A node validating a mini-block MUST recover the signer from the digest and the attached signature and MUST reject the mini-block's signature as invalid unless the recovered address equals `sequencerAt(block_number)`.

## Mini-Block Timestamps

`block_timestamp` MUST equal the `timestamp` field of the containing EVM block's header, in seconds, per standard Ethereum block timestamp semantics.

The mini-block `timestamp` field's unit is spec-dependent at the mini-block's `block_timestamp`: milliseconds before [Rex5](../upgrades/rex5.md), microseconds from Rex5 onward.
A node MUST interpret the raw `uint64` value of `timestamp` according to which of these specs is active at `block_timestamp`, and MUST NOT compare or combine mini-block timestamps from different specs without first normalizing them to a common unit.

Mini-block `timestamp` values are not consensus-enforced to be non-decreasing, either within an EVM block or across the chain.
A node MUST NOT rely on `timestamp` for mini-block ordering; ordering is established solely by `index` (within an EVM block) and `number` (chain-wide).

## Mini-Block Validity Rules

A node validating a received mini-block MUST apply the following rules:

- `header.index == 0` and no prior mini-block exists for `header.block_number` -> `accept`: the first mini-block of an EVM block.
- `header.index != 0` and no mini-block with index `header.index - 1` has been accepted for `header.block_number` -> `reject`: mini-blocks within an EVM block MUST be accepted in index order.
- `header.number` does not equal the previously accepted mini-block's `number + 1` -> `reject`: the chain-wide mini-block number MUST be contiguous.
- `header.tx_root` does not match the root recomputed from the mini-block's transaction list -> `reject`.
- `header.receipt_root` does not match the root recomputed from the mini-block's receipt list -> `reject`.
- `header.gas_used` does not equal the sum of gas used by the mini-block's transactions -> `reject`.
- The active spec at `header.block_timestamp` requires a signature and none is attached, or an attached signature does not recover to `sequencerAt(header.block_number)` -> `reject`.

## Constants

This page defines no additional named constants.
Field widths and encoding rules above are structural, not configurable; the sequencer signing key is defined by the [SequencerRegistry](../system-contracts/sequencer-registry.md) system contract, not by a constant on this page.

## Rationale

**Why the mini-block header omits a parent reference, state root, and bloom filter.**
Each of these fields exists in a standard Ethereum header to let the header stand alone as a root of trust: the parent hash chains blocks without external context, the state root commits to post-execution state, and the bloom filter allows filtering logs without fetching receipts.
A mini-block is not meant to stand alone — its position is established by the containing EVM block and by the state sync protocol that delivers it, and its logs are already available via the receipt list it carries alongside the header.
Dropping these fields removes the largest components of a standard header without removing any guarantee a mini-block is meant to provide.

**Why the signing digest uses `keccak256` rather than an alternative hash function.**
The digest MUST be independently reproducible on-chain via the EVM `ecrecover` precompile, which is defined in terms of a `keccak256` digest.
Using a different hash function for the mini-block digest would make the same signature unverifiable both off-chain and on-chain with a single verification routine.

**Why mini-block ordering does not rely on `timestamp`.**
`timestamp` is a wall-clock sample taken at seal time and is not a consensus-agreed value between the fields already used to order mini-blocks.
Tying validity or ordering to it would make wall-clock skew or adjustment a consensus concern; `index` and `number` are sequencer-assigned, contiguous, and free of that dependency.

**Why every EVM block MUST contain at least one mini-block, but the count and cadence beyond that are unspecified.**
The one-or-more minimum is not a validity nicety: mini-blocks are the atomic unit the state sync protocol delivers, so a sequencer that produced zero mini-blocks for an EVM block would give other nodes no way to receive that block's content through the primary sync path at all.
How many mini-blocks beyond that minimum, and when each is sealed, changes only how finely the content is chunked, never whether it can be synchronized — no node re-derives a specific count or cadence, so constraining it further would remove sequencer implementation freedom for no verifiable benefit, the same reasoning [Sequencing](sequencing.md#block-production-mechanics) applies to sealing cadence generally.

## Security Considerations

**A mini-block signature is an equivocation proof, not a finality proof.**
Because the digest commits to `tx_root`, `receipt_root`, and `gas_used`, a valid signature over a mini-block is a binding commitment: if the sequencer subsequently seals an EVM block whose contents contradict a mini-block it signed, the signed mini-block header is sufficient evidence of that misbehavior.
A node MUST NOT treat a valid mini-block signature as proof of finality; finality of the mini-block's transactions still depends on the containing EVM block reaching finality on L1.

**Pre-Rex5 mini-blocks carry no cryptographic commitment.**
Before Rex5, a mini-block's `signature` is absent, and nothing about the mini-block prevents a sequencer from later sealing an EVM block that disagrees with it.
A consumer that treats an unsigned mini-block as a binding preconfirmation is relying on sequencer behavior, not on a spec-enforced guarantee.

**A mini-block's identity is positional, not hash-chained.**
Because the header carries no reference to its predecessor, a node accepting mini-blocks out of order, or from a source that does not enforce the [Mini-Block Validity Rules](#mini-block-validity-rules), has no header-level defense against a reordered or substituted mini-block at a given `(block_number, index)` position.
This obligation is carried by the [state sync](state-sync.md) protocol's fragment validity rules, not by anything inside the mini-block header itself; an implementation that only checks header well-formedness without enforcing contiguous `index` and `number` acceptance does not meet this specification.

**A sequencer that produces zero mini-blocks for an EVM block causes a sync failure, not an invalid block.**
Nothing in the [Mini-Block Validity Rules](#mini-block-validity-rules) directly checks that an EVM block was preceded by at least one mini-block, because that check happens implicitly: a node relying on the state sync protocol simply never receives that EVM block's content if no mini-block for it was ever streamed.
This makes the [Block Hierarchy](#block-hierarchy) minimum a liveness dependency enforced by absence rather than a rule any single message triggers a rejection for — an implementation that bypasses mini-block streaming entirely for some EVM blocks does not meet this specification, even though no individual mini-block or EVM block it does produce is itself invalid.

## Spec History

- [Rex5](../upgrades/rex5.md) — Introduced mini-block header signing and changed mini-block `timestamp` resolution from milliseconds to microseconds. Before Rex5, mini-blocks carry no signature and `timestamp` is millisecond-resolution.
