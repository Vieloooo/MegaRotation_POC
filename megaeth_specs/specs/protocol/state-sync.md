---
description: Normative wire protocol for state sync — stream and payload types, payload authentication, encodings, and mini-block and EVM block assembly and validity rules.
spec: Rex5
---

# State Sync

MegaETH nodes do not re-execute blocks to follow the chain: the [sequencer](https://specs.optimism.io/glossary.html#sequencer) publishes each [mini-block](https://docs.megaeth.com/spec/glossary#mini-block)'s transactions, receipts, and state deltas as separately signed **payloads**, and every consuming node reconstructs mini-blocks and [EVM blocks](https://docs.megaeth.com/spec/glossary#evm-block) from them.
This page specifies the payload types and their encodings, how each payload is authenticated against the active sequencer key, how a node subscribes to a stream, the assembly and validity rules that turn a payload stream into mini-blocks and EVM blocks, and the pull-based requests used to backfill a gap, resolve a reorg, and bootstrap a node with no state.

How peers are discovered and ranked, and which peer a node subscribes to for a given stream, are not specified here.
Two implementations that exchange the payloads defined on this page while choosing their peers differently are indistinguishable to any external observer of correctness, so peer selection is an implementation concern per the [Normativity Boundary](overview.md#normativity-boundary).
Everything on this page is defined in terms of the payloads themselves, the messages that carry them, and the order in which a node hands them to assembly.

## Transport Binding

State sync runs as an [RLPx](https://github.com/ethereum/devp2p/blob/master/rlpx.md) sub-protocol whose capability is `mega` at version `1`, claiming `33` (`0x21`) message ids.
The ids in [Wire Envelope](#wire-envelope) are offsets within the contiguous id block that [capability multiplexing](https://github.com/ethereum/devp2p/blob/master/rlpx.md#capability-messaging) assigns to `mega/1` when peers exchange capability lists in the `Hello` handshake, so a frame carries `base_offset + message_id`.

## Motivation

A node that follows the chain by re-executing EVM blocks cannot see a transaction's result until the block containing it is built and executed locally, which reduces the value of MegaETH's ~10 ms [mini-block](blocks.md#block-hierarchy) cadence to nothing: the data exists on the sequencer within milliseconds but is invisible to consumers for a full EVM block time.

## Trust Model

A node that follows the chain through state sync trusts the [sequencer](https://specs.optimism.io/glossary.html#sequencer).
Every rule on this page establishes that received data is the data the sequencer published; no rule establishes that what the sequencer published is correct.
A node that requires that guarantee obtains it elsewhere: [Stateless Validation](stateless-validation.md) re-executes the block and compares the outputs, and [Fault Proofs](../rollup/fault-proofs.md) make an incorrect state transition challengeable on L1.

A node MUST NOT extend that trust to the peer that supplied the data.
A node MUST apply the verification in [Verification](#verification) to every payload identically, whatever its source, and MUST NOT accept a payload, or any part of a [fetch response](#fetch-responses), because of which peer sent it.
The sequencer's signature over a payload is the only evidence of sequencer origin this protocol provides; no property of a connection or a peer carries that evidence.

Applying that verification depends in turn on a node's view of the [SequencerRegistry](../system-contracts/sequencer-registry.md), which determines the key authorized to sign for a given block.

## Streams and Payloads

The sequencer publishes four independent **streams**.
Each stream carries exactly one payload type, and each payload is individually signed (see [Payload Authentication](#payload-authentication)).

| Stream        | Wire id | Mask bit | Payload type                                    | Published           | Carries                                                                                         |
| ------------- | ------- | -------- | ----------------------------------------------- | ------------------- | ----------------------------------------------------------------------------------------------- |
| `StateChange` | `1`     | `0x01`   | [State-change fragment](#state-change-fragment) | Once per mini-block | SALT state deltas, newly deployed bytecode, buckets to canonicalize, and the block context      |
| `Transaction` | `2`     | `0x02`   | [Transaction fragment](#transaction-fragment)   | Once per mini-block | The mini-block's transactions with their recovered senders, and the mini-block header signature |
| `Receipt`     | `3`     | `0x04`   | [Receipt fragment](#receipt-fragment)           | Once per mini-block | The mini-block's receipts with their bloom filters                                              |
| `Header`      | `4`     | `0x08`   | [Signed header](#signed-header)                 | Once per EVM block  | The sealed EVM block header and the block metadata that binds its mini-blocks to it             |

The wire id and the mask bit encode two distinct field types, which appear in different messages and MUST NOT be substituted for one another: the value `4` selects the `Header` stream as a wire id, but the `Receipt` stream as a mask bit.

- The **wire id** encodes a `stream_id` field, which names exactly one stream, as an RLP integer.
  It appears in the fragment-range request specified in [Backfill](#backfill) and in the subscription messages specified in [Stream Subscription](#stream-subscription).
  A node MUST reject a message whose `stream_id` field decodes to a value other than `1`, `2`, `3`, or `4`.
- The **mask bit** encodes a `stream_mask` field, which names any subset of streams as a single RLP integer formed by the bitwise OR of the selected streams' mask bits.
  It appears in the partial-block-range request specified in [Backfill](#backfill) and in the child-block request specified in [Reorg Resolution](#reorg-resolution), which define what selecting a stream means for those responses.
  A node MUST ignore a set bit that the table above does not define, and MUST ignore the `Header` mask bit, since a response carrying blocks always includes each block's signed header.

The data format of each payload type is specified in [Payload Encodings](#payload-encodings).
A mini-block is streamed as three fragments, one on each per-mini-block stream, i.e., state change, transaction, and receipt, so an EVM block containing `n` mini-blocks is streamed as `3n` fragments plus one signed header.

## Wire Envelope

Every payload and every request or response defined on this page is carried in the same envelope:

```text
message = message_id ++ payload

message_id = 1 byte, raw (not RLP-encoded)
payload    = the RLP encoding of the message body
```

where `++` is byte concatenation.

| `message_id` | Message                                                                          | Body                                                                             |
| ------------ | -------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| `0x11`       | Heartbeat                                                                        | `rlp([payload_id])` — see [Payload Authentication](#payload-authentication)      |
| `0x12`       | [Signed header](#signed-header)                                                  | The payload — see [Signed header](#signed-header)                                |
| `0x13`       | [State-change fragment](#state-change-fragment)                                  | The payload — see [State-change fragment](#state-change-fragment)                |
| `0x14`       | [Transaction fragment](#transaction-fragment)                                    | The payload — see [Transaction fragment](#transaction-fragment)                  |
| `0x15`       | [Receipt fragment](#receipt-fragment)                                            | The payload — see [Receipt fragment](#receipt-fragment)                          |
| `0x16`       | Subscribe request — see [Stream Subscription](#stream-subscription)              | `rlp([request_id, [stream_id, tree_id, start, end]])`                            |
| `0x17`       | Subscribe response — see [Stream Subscription](#stream-subscription)             | `rlp([request_id, accept, latest])`                                              |
| `0x19`       | Prune — see [Stream Subscription](#stream-subscription)                          | `rlp([stream_id, tree_id, ending])`                                              |
| `0x1A`       | Bucket-range request — see [State Snapshot Bootstrap](#state-snapshot-bootstrap) | `rlp([request_id, [version, start, end]])`                                       |
| `0x1B`       | Fragment-range request — see [Backfill](#backfill)                               | `rlp([request_id, [stream_id, start, end]])`                                     |
| `0x1C`       | Partial-block-range request — see [Backfill](#backfill)                          | `rlp([request_id, [start, end, stream_mask]])`                                   |
| `0x1D`       | Child-block request — see [Reorg Resolution](#reorg-resolution)                  | `rlp([request_id, [parent_number, parent_hash, parent_timestamp, stream_mask]])` |
| `0x1E`       | Ancestor-headers request — see [Reorg Resolution](#reorg-resolution)             | `rlp([request_id, [start_number, start_hash]])`                                  |
| `0x1F`       | [Fetch response](#fetch-responses)                                               | `rlp([request_id, type_id, ...])`                                                |

`request_id` is defined in [Requests and Responses](#requests-and-responses), which also gives the rules that correlate a response with the request it answers; those rules apply to every body above that begins with `request_id`.

Message ids not listed above are either reserved or carry messages outside the scope of this specification (propagation-topology control messages, and sequencer-internal control traffic; see the [Normativity Boundary](overview.md#normativity-boundary)).
A node MUST reject, and MUST NOT act on, a message whose `message_id` byte it does not recognize.

## Payload Authentication

Every payload is authenticated on its own, by a sequencer signature that binds the payload's contents to its position in its stream and to the block it describes.
A node consuming a payload reproduces all three of those — the content hash, the position, and the key the block authorizes — and rejects the payload if any one of them disagrees with what the payload claims.

### Payload Numbering

Each stream is a sequence, and a node consuming one needs to know where in that sequence a payload belongs: a payload's contents alone do not say whether it is the next one due, one already applied, or one from far ahead.
A payload's position in its own stream is its `payload_number`.

`payload_number` is bound into the payload's signature, so a validly signed payload cannot be presented as a position in the stream other than the one the sequencer signed it for.

`payload_number` is scoped per stream; the four streams do not share one numbering space:

- On the `StateChange`, `Transaction`, and `Receipt` streams, a payload's `payload_number` is the chain-wide [mini-block `number`](blocks.md#mini-block-header) of the mini-block it describes, carried explicitly as the payload's `fragment_number` field.
- On the `Header` stream, a payload's `payload_number` equals its `block_number`.

The three per-mini-block streams' `payload_number` values therefore advance in lockstep, while the `Header` stream's advances once per EVM block.

### Content Hash and Signature

Each payload carries a content hash, `payload_hash`, computed over its own fields, and a `signature` that binds `payload_hash` to the payload's `block_number` and `payload_number`.
Together, these four values are a payload's authenticated identity, the tuple

```text
payload_id = (block_number, payload_number, payload_hash, signature)
```

and this same tuple's encoding, `rlp([block_number, payload_number, payload_hash, signature])`, is what a node exchanges when it announces its stream position in a heartbeat, wrapped in one more list as `rlp([payload_id])` per the [Wire Envelope](#wire-envelope) — the heartbeat body is a list containing that single nested list, not a flat 4-item list.

A node MUST compute `payload_hash` and `signature` as follows, where `hash` is [BLAKE3](https://github.com/BLAKE3-team/BLAKE3-specs/blob/master/blake3.pdf) with a 32-byte output:

1. `payload_hash = hash(encode(payload))`, where `encode(payload)` is the payload's own wire encoding, defined per type in [Payload Encodings](#payload-encodings), computed with its `payload_hash` field set to 32 zero bytes and its `signature` field set to the empty byte string.
   The [signed header](#signed-header) is the exception: its content hash is defined in that section.
2. `sign_hash = hash(block_number_be ++ payload_number_be ++ payload_hash)`, where `block_number_be` and `payload_number_be` are the 8-byte big-endian encodings of `block_number` and `payload_number`.
3. `signature` is a recoverable secp256k1 ECDSA signature over `sign_hash`, with no message prefix, serialized as the 65 bytes `r ++ s ++ v`, where `r` and `s` are the two 32-byte big-endian signature scalars and `v` is `27 + y_parity`.
   `y_parity` is the recovery bit that secp256k1 signing returns alongside `r` and `s` — `0` or `1`.

### Verification

A node MUST apply the following rules, in order, to every payload it receives, whether the payload arrived on a stream or in a response to a request:

- `payload.block_number == 0` -> `accept` iff `payload.payload_number == 0`: the genesis block has no fragment, so the only well-formed payload at block `0` is the zero-numbered one and its signature is not checked.
- `hash(encode(payload with payload_hash and signature cleared)) != payload.payload_hash` -> `reject`: the payload's content hash does not commit to its own contents.
- `signature` does not recover the key that the [SequencerRegistry](../system-contracts/sequencer-registry.md) authorizes for `payload.block_number` -> `reject`.
- Otherwise -> `accept`.

The authorized key for a payload is the key `sequencerAt(payload.block_number)` returns, exactly as for [mini-block header signing](blocks.md#mini-block-header-signing); a payload for block `N` MUST be signed by the key authorized for block `N`, not by the key authorized at the time the payload is relayed.
A node whose view of `SequencerRegistry` does not yet cover `payload.block_number` cannot decide the third rule; it MUST NOT accept the payload.

A node MUST apply the same three rules to a `payload_id` that arrives on its own — in a heartbeat, or in any other announcement of a peer's stream position — before treating the claimed position as true.

{% hint style="info" %}
The payload content hash and its signature are BLAKE3-based and are never verified inside the EVM, unlike the [mini-block header signature](blocks.md#mini-block-header-signing), which uses `keccak256` precisely so that `ecrecover` can reproduce it.
The two signatures are independent and cover different data: one authenticates a streamed payload, the other authenticates the mini-block header a consumer reconstructs from payloads.
{% endhint %}

## Payload Encodings

`++` is byte concatenation, and `rlp([...])` denotes a standard [RLP](https://ethereum.org/en/developers/docs/data-structures-and-encoding/rlp/) list.
Integer fields are encoded as standard RLP integers (minimal-length big-endian, no leading zero bytes) unless stated otherwise; `bytes32` fields are encoded as fixed 32-byte RLP strings.
Where a payload's encoding is a bare concatenation of RLP items rather than an RLP list, no list header is written and the payload is decoded positionally — a decoder MUST NOT expect an enclosing list header for those payloads.
`encode(x)` denotes the complete wire encoding defined below for `x`'s type: an `rlp([...])` list for some payload types, a bare concatenation of RLP items for others.
Transactions and receipts appear inside RLP lists in their network encoding: a typed ([EIP-2718](https://eips.ethereum.org/EIPS/eip-2718)) item is the RLP byte string `rlp(type_byte ++ payload)`, and a legacy item is its bare RLP list.

{% hint style="info" %}
Four values this page treats as given are defined by [SALT](salt/overview.md) rather than here: the [wire encoding of `salt_deltas`](salt/trie.md#encoding), the [rule for merging a block's per-mini-block deltas](salt/trie.md#salt-deltas) into the single value `salt_deltas_hash` commits to, the [bucket contents and metadata a bucket-range response carries](salt/buckets.md#bucket-range-encoding), and the [derivation of a state root](salt/trie.md#state-root) from applied deltas.
{% endhint %}

Every payload carries a `block_number`, the number of the EVM block it describes: on the three per-mini-block streams, the block containing the mini-block the payload describes; on the `Header` stream, the block the header seals.
`block_number` is what selects the key a payload is authenticated against, per [Payload Authentication](#payload-authentication).

### State-Change Fragment

A state-change fragment carries the SALT state transition of one mini-block.

| Field                     | Type                                      | Description                                                                                                                                                                            |
| ------------------------- | ----------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `block_number`            | `uint64`                                  | Number of the EVM block the mini-block belongs to.                                                                                                                                     |
| `fragment_number`         | `uint64`                                  | Chain-wide [mini-block `number`](blocks.md#mini-block-header) of the mini-block this fragment describes; this payload's `payload_number`, per [Payload Numbering](#payload-numbering). |
| `salt_deltas`             | SALT deltas                               | The mini-block's state deltas, encoded per [SALT delta encoding](salt/trie.md#encoding).                                                                                               |
| `new_bytecodes`           | Account-bytecode map                      | Bytecode deployed in this mini-block, keyed both by account and by code hash. See below.                                                                                               |
| `buckets_to_canonicalize` | list of bucket ids                        | The [SALT buckets](salt/buckets.md#canonicalization) this fragment names for canonicalization; the block-wide union is what consumers act on.                                          |
| `timestamp`               | `uint64`                                  | The mini-block's `timestamp`, in the raw unit of the active spec — see [Mini-Block Timestamps](blocks.md#mini-block-timestamps).                                                       |
| `payload_hash`            | `bytes32`                                 | Content hash, per [Content Hash and Signature](#content-hash-and-signature).                                                                                                           |
| `signature`               | 65-byte string                            | Payload signature, per [Content Hash and Signature](#content-hash-and-signature).                                                                                                      |
| `block_ctx`               | [Block context](#block-context), optional | Present only on the fragment of the first mini-block of each EVM block.                                                                                                                |

A node MUST encode the fragment as:

```text
encode(state_change_fragment) =
    rlp(block_number) ++ rlp(fragment_number) ++ encode(salt_deltas) ++ encode(new_bytecodes)
    ++ rlp(buckets_to_canonicalize) ++ rlp(timestamp) ++ rlp(payload_hash) ++ rlp(signature)
    ++ block_ctx_flag ++ block_ctx?
```

Neither `encode(salt_deltas)` nor `encode(new_bytecodes)` is a single RLP item: each is its own bare concatenation, not wrapped in an RLP string or list.
`salt_deltas`'s bare-concatenation encoding is defined by [SALT delta encoding](salt/trie.md#encoding), not by this page; `new_bytecodes`'s is defined immediately below.
`block_ctx_flag` is a single raw byte, `0x01` if a block context follows and `0x00` if not, and `block_ctx?` is the [block context](#block-context) encoding, present if and only if the flag is `0x01`.
The `block_ctx` field is encoded last, after the signature, even though it is described above in field order.

`new_bytecodes` is encoded as two length-prefixed sequences of RLP items, in ascending key order:

```text
encode(new_bytecodes) =
    rlp(account_count) ++ (rlp(address) ++ rlp(code_hash))*
    ++ rlp(code_count)  ++ (rlp(code_hash) ++ rlp(bytecode))*
```

where `*` means zero or more of the item to its left, `account_count` and `code_count` are the number of entries in the following sequence, the first sequence is ordered by ascending `address`, and the second by ascending `code_hash`.

A node MUST include a block context on the state-change fragment whose `fragment_number` is the first mini-block number of its EVM block, and MUST omit it on every other fragment; a node consuming the stream MUST ignore a block context found on any other fragment.

### Transaction Fragment

| Field              | Type                     | Description                                                                                                                                                                            |
| ------------------ | ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `block_number`     | `uint64`                 | Number of the EVM block the mini-block belongs to.                                                                                                                                     |
| `fragment_number`  | `uint64`                 | Chain-wide [mini-block `number`](blocks.md#mini-block-header) of the mini-block this fragment describes; this payload's `payload_number`, per [Payload Numbering](#payload-numbering). |
| `transactions`     | list of transactions     | The mini-block's transactions, in execution order, each [EIP-2718](https://eips.ethereum.org/EIPS/eip-2718)-encoded per [Transactions](transactions.md).                               |
| `senders`          | list of `address`        | The sender of each transaction, in the same order.                                                                                                                                     |
| `timestamp`        | `uint64`                 | The mini-block's `timestamp`.                                                                                                                                                          |
| `payload_hash`     | `bytes32`                | Content hash, per [Content Hash and Signature](#content-hash-and-signature).                                                                                                           |
| `signature`        | 65-byte string           | Payload signature, per [Content Hash and Signature](#content-hash-and-signature).                                                                                                      |
| `header_signature` | 65-byte string, optional | The [mini-block header signature](blocks.md#mini-block-header-signing) of this mini-block, serialized as the same 65 bytes `r ++ s ++ v` as `signature` above.                         |

A node MUST encode the fragment as:

```text
encode(transaction_fragment) = rlp([block_number, fragment_number, transactions, senders, timestamp, payload_hash, signature, header_signature])
```

`header_signature` is a trailing optional field: when it is absent the list ends after `signature`, so a fragment without it encodes exactly as a seven-item list, byte-identical to the encoding used before the field existed.
`senders` MUST have the same length as `transactions`, and its `i`-th entry MUST be the sender of the `i`-th transaction.

The sender list is a convenience that lets a consumer skip signature recovery; it is not a trust anchor.
A consumer that relies on `senders` without recovering them itself is trusting the payload signature to bind them, which it does, since `senders` is covered by `payload_hash`.

### Receipt Fragment

| Field             | Type             | Description                                                                                                                                                                            |
| ----------------- | ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `block_number`    | `uint64`         | Number of the EVM block the mini-block belongs to.                                                                                                                                     |
| `fragment_number` | `uint64`         | Chain-wide [mini-block `number`](blocks.md#mini-block-header) of the mini-block this fragment describes; this payload's `payload_number`, per [Payload Numbering](#payload-numbering). |
| `receipts`        | list of receipts | The mini-block's receipts, in execution order, each [EIP-2718](https://eips.ethereum.org/EIPS/eip-2718)-encoded with its bloom filter.                                                 |
| `timestamp`       | `uint64`         | The mini-block's `timestamp`.                                                                                                                                                          |
| `payload_hash`    | `bytes32`        | Content hash, per [Content Hash and Signature](#content-hash-and-signature).                                                                                                           |
| `signature`       | 65-byte string   | Payload signature, per [Content Hash and Signature](#content-hash-and-signature).                                                                                                      |

A node MUST encode the fragment as:

```text
encode(receipt_fragment) = rlp([block_number, fragment_number, receipts, timestamp, payload_hash, signature])
```

Each receipt's `cumulative_gas_used` is cumulative over the containing **EVM block**, not over the mini-block, and is the value a consumer uses to derive per-transaction gas (see [Mini-Block Assembly](#mini-block-assembly)).

### Signed Header

A signed header carries the sealed EVM block header together with the **block metadata** that binds the block's mini-blocks and streamed state to it.

| Field            | Type                              | Description                                                                          |
| ---------------- | --------------------------------- | ------------------------------------------------------------------------------------ |
| `header`         | EVM block header                  | The sealed [EVM block](blocks.md#evm-block) header, in standard Ethereum header RLP. |
| `block_metadata` | [Block metadata](#block-metadata) | Fixed-length metadata for this block, including the payload signature.               |
| `payload_hash`   | `bytes32`                         | Content hash, defined below.                                                         |

A node MUST encode the signed header as:

```text
encode(signed_header) = rlp([header, block_metadata, payload_hash])
```

where `block_metadata` is written as `BLOCK_METADATA_LENGTH` raw bytes inside the list payload, with no RLP string header of its own, and is decoded positionally by its fixed length: the list's second item is therefore not self-delimiting the way a standard RLP item is, and a decoder MUST know `BLOCK_METADATA_LENGTH` out of band to parse it — this is not strict RLP framing.

The signed header's content hash and signature deviate from the general rule in [Content Hash and Signature](#content-hash-and-signature) in two ways: the content hash is computed over the header hash rather than over the header itself, and the signature is carried inside `block_metadata` rather than as a separate field.
A node MUST compute them as:

```text
payload_hash = hash(block_hash ++ metadata_without_signature)
sign_hash    = hash(block_number_be ++ block_number_be ++ payload_hash)
```

where `block_hash` is the header's hash, `metadata_without_signature` is the [block metadata encoding](#block-metadata) truncated to exclude its trailing 65 signature bytes, and `block_number_be` is the 8-byte big-endian block number, written twice because a header's `payload_number` equals its `block_number`.

A node MUST recompute `payload_hash` from the header hash it derived when decoding the header, MUST reject the payload if the recomputed value differs from the `payload_hash` field, and MUST then verify `block_metadata.signature` over `sign_hash` against the key authorized for `block_number`.

### Block Metadata

Block metadata is encoded as a fixed-length byte string of exactly `BLOCK_METADATA_LENGTH` bytes, with every integer big-endian and fixed-width — not as RLP items.
A node MUST reject a signed header whose metadata field is not exactly `BLOCK_METADATA_LENGTH` bytes, or whose trailing signature bytes do not form a well-formed recoverable signature.

| Bytes        | Field                | Type      | Description                                                                                                                          |
| ------------ | -------------------- | --------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `[0, 8)`     | `tx_offset`          | `uint64`  | Chain-wide transaction number of this block's first transaction.                                                                     |
| `[8, 16)`    | `mini_block_offset`  | `uint64`  | Chain-wide mini-block number of this block's first mini-block.                                                                       |
| `[16, 24)`   | `mini_block_count`   | `uint64`  | Number of mini-blocks in this block.                                                                                                 |
| `[24, 32)`   | `safe.number`        | `uint64`  | Number of the block the sequencer considered safe when this block was published.                                                     |
| `[32, 64)`   | `safe.hash`          | `bytes32` | Hash of that block.                                                                                                                  |
| `[64, 72)`   | `finalized.number`   | `uint64`  | Number of the block the sequencer considered finalized when this block was published.                                                |
| `[72, 104)`  | `finalized.hash`     | `bytes32` | Hash of that block.                                                                                                                  |
| `[104, 136)` | `salt_deltas_hash`   | `bytes32` | `hash(encode(salt_deltas))` over this block's merged SALT deltas — the commitment that binds the streamed state deltas to the block. |
| `[136, 137)` | `withdrawal_updated` | `uint8`   | `0x00` if this block did not modify the `L2ToL1MessagePasser` predeploy, non-zero if it did.                                         |
| `[137, 202)` | `signature`          | 65 bytes  | The [signed header](#signed-header) payload's signature, serialized as `r ++ s ++ v`.                                                |

`safe` and `finalized` are the sequencer's view at publication time, not a value any consumer re-derives; a consumer MUST NOT treat them as a verified L1 finality claim.
`mini_block_count` is the value against which assembly decides a block's fragments are complete, and `salt_deltas_hash` is the value against which it checks the merged state deltas; both are specified in [EVM Block Assembly](#evm-block-assembly).
`mini_block_offset` and `tx_offset` locate the block's mini-blocks and transactions in the chain-wide numbering, and duplicate the corresponding [block context](#block-context) fields that arrive earlier on the `StateChange` stream.
`withdrawal_updated` is advisory: it reports whether the withdrawal commitment can have moved in this block, and no rule on this page depends on it — the withdrawals-root check in [EVM Block Assembly](#evm-block-assembly) is applied whether or not it is set.

## Stream Subscription

A node does not receive a stream by being connected to a peer; it receives it by asking that peer for it.
Each stream is distributed over `TREE_NUMBER` independent broadcast trees, and a node subscribes per tree.

Which tree carries a payload is fixed by its number: a payload belongs to tree `payload_number % TREE_NUMBER` of its own stream.
A node subscribed to only some of a stream's trees therefore receives only the payloads whose numbers land on those trees — with `TREE_NUMBER = 3`, subscribing on two trees loses every third payload of that stream, forever.
A node that wants a complete stream MUST subscribe on all `TREE_NUMBER` trees of it.

**Subscribe request** (`rlp([request_id, [stream_id, tree_id, start, end]])`) asks a peer to forward one tree's payloads, where `stream_id` is the stream's wire id per [Streams and Payloads](#streams-and-payloads), `tree_id` identifies the tree within that stream, and `start` and `end` bound the subscription in `payload_number`, `end` inclusive.
`end = 0` requests an open-ended subscription.
A request MUST satisfy all of the following; a responder MUST NOT answer one that does not, since such a request is malformed rather than merely unservable:

- `tree_id < TREE_NUMBER`.
- `start % TREE_NUMBER == tree_id`, and, unless `end` is `0`, `end % TREE_NUMBER == tree_id`: both bounds are payload numbers that land on the subscribed tree.
- `end` is `0`, or greater than `start`.

**Subscribe response** (`rlp([request_id, accept, latest])`) answers it, where `accept` is an RLP boolean — `0x01` for true, the empty string `0x80` for false — and `latest` is the responder's own [`payload_id`](#content-hash-and-signature) on the subscribed tree, as the nested list `[block_number, payload_number, payload_hash, signature]`.
Like a [fetch response](#fetch-responses), and unlike a request, its fields are flat.
A responder MAY refuse any subscription, MUST set `accept` to false when it does, and MUST carry `latest` either way.
A requester MUST apply the following rules, in order:

- `accept` is false -> no subscription exists; the requester MUST NOT expect payloads on that tree and MUST subscribe elsewhere for it.
- `latest` fails the `payload_id` verification of [Verification](#verification) -> `reject`.
- `latest.payload_number % TREE_NUMBER` does not equal the requested `tree_id` -> `reject`: the responder is not serving the tree it accepted.

An accepted subscription forwards payloads as the responder itself receives them, and MUST NOT be read as a request for history: a payload the responder forwarded before the subscription existed is never sent, whatever `start` says.
A requester whose `start` is behind the responder's `latest` therefore has a gap that only [Backfill](#backfill) closes.

**Prune** (`rlp([stream_id, tree_id, ending])`) ends a subscription from the subscriber's side, carries no `request_id`, and is not answered.
`ending` is the last `payload_number` the subscriber still wants, and `0` ends the subscription at once.
A responder MUST stop forwarding once `ending` is passed, and MUST ignore a prune from a peer holding no subscription on that tree.

## Payload Delivery

Assembly consumes each stream as an ordered sequence, so a node MUST NOT hand a payload to assembly until every lower-numbered payload of the same stream has already been handed to assembly, and MUST NOT hand more than one payload for any one `payload_number` to assembly.
These two obligations are what assembly relies on; how a node achieves them — by receiving payloads in order, by buffering and reordering them, or by pulling the missing ones per [Backfill](#backfill) — is not specified.

A node MUST verify a payload per [Verification](#verification) before handing it to assembly, and MUST discard an unverifiable payload without handing it on.
A node that discards a payload MUST NOT advance its own stream position past that payload's number, since doing so would make the gap unrecoverable through in-order delivery.

## Block Context

The block context carries the block-level values a consumer needs in order to reconstruct mini-block headers and transaction metadata before the EVM block's own header exists.

| Field                      | Type                | Description                                                                                                                                                    |
| -------------------------- | ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `parent_block_hash`        | `bytes32`           | Hash of the parent EVM block.                                                                                                                                  |
| `block_number`             | `uint64`            | Number of this EVM block.                                                                                                                                      |
| `timestamp`                | `uint64`            | Timestamp of this EVM block, in seconds.                                                                                                                       |
| `tx_number_offset`         | `uint64`            | Chain-wide transaction number of this block's first transaction.                                                                                               |
| `mini_block_number_offset` | `uint64`            | Chain-wide mini-block number of this block's first mini-block.                                                                                                 |
| `base_fee`                 | `uint64`            | Base fee of this block.                                                                                                                                        |
| `suggested_fee_recipient`  | `address`           | Fee recipient of this block.                                                                                                                                   |
| `prev_randao`              | `bytes32`           | `prevRandao` value of this block.                                                                                                                              |
| `gas_limit`                | `uint64`            | Gas limit of this block.                                                                                                                                       |
| `parent_beacon_block_root` | `bytes32`, optional | Parent beacon block root, when the active spec provides one.                                                                                                   |
| `extra_data`               | byte string         | The block's `extraData`, carrying the encoded [EIP-1559 parameters](https://specs.optimism.io/protocol/holocene/exec-engine.html#dynamic-eip-1559-parameters). |

A node MUST encode the block context as:

```text
block_ctx =
    rlp(parent_block_hash) ++ rlp(block_number) ++ rlp(timestamp) ++ rlp(tx_number_offset)
    ++ rlp(mini_block_number_offset) ++ rlp(base_fee) ++ rlp(suggested_fee_recipient)
    ++ rlp(prev_randao) ++ rlp(gas_limit) ++ beacon_root_flag ++ parent_beacon_block_root?
    ++ rlp(extra_data)
```

where `beacon_root_flag` is a single raw byte, `0x01` if a parent beacon block root follows and `0x00` if not.

A block context is delivered exactly once per EVM block, on the state-change fragment of that block's first mini-block, per [State-Change Fragment](#state-change-fragment); every other mini-block of the block omits it.
It is not itself part of any mini-block: it is context shared by every mini-block of the EVM block it belongs to, used to reconstruct each of them from that mini-block's own three fragments.
A node that has not received an EVM block's context therefore cannot reconstruct any of that block's mini-blocks, no matter how many of their fragments have already arrived; see [Mini-Block Assembly](#mini-block-assembly).

## Mini-Block Assembly

The three fragments describing one mini-block MUST all carry the same `block_number` and the same `payload_number`, and a node MUST group a mini-block's fragments by that pair.
A mini-block is reconstructible once those three fragments and the block context for its EVM block have all been delivered.
A node MUST NOT reconstruct any mini-block of an EVM block before receiving that block's context, because the context supplies values no fragment carries.

Given the block context `ctx` and the mini-block's three fragments, all sharing `payload_number`, a node MUST derive the mini-block's zero-based position within its EVM block as `i = payload_number - ctx.mini_block_number_offset`, and MUST reconstruct the [mini-block header](blocks.md#mini-block-header) as:

| Header field      | Value                                                                                                                                |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `block_number`    | `ctx.block_number`                                                                                                                   |
| `block_timestamp` | `ctx.timestamp`                                                                                                                      |
| `index`           | `i`                                                                                                                                  |
| `number`          | `payload_number`                                                                                                                     |
| `timestamp`       | The state-change fragment's `timestamp`, interpreted in the unit active at `ctx.timestamp`                                           |
| `gas_used`        | The sum, over the mini-block's transactions, of each transaction's gas used, derived as below                                        |
| `tx_root`         | Recomputed over the mini-block's transactions, carried by the transaction fragment, per [Mini-Block Body](blocks.md#mini-block-body) |
| `receipt_root`    | Recomputed over the mini-block's receipts, carried by the receipt fragment, per [Mini-Block Body](blocks.md#mini-block-body)         |

The header MUST be reconstructed rather than taken from the wire: no payload carries a mini-block header, and the mini-block header signature is verified against the header the consumer derived, so a consumer that derived it incorrectly rejects the signature rather than accepting a header the sequencer never signed.

Per-transaction gas is derived from the receipts' block-cumulative gas.
A node MUST maintain a running `block_cumulative_gas_used`, initialized to `0` at the start of each EVM block, and for each transaction in order MUST compute `tx_gas_used = receipt.cumulative_gas_used - block_cumulative_gas_used` and then set `block_cumulative_gas_used = receipt.cumulative_gas_used`.

A node MUST apply the following rules when reconstructing a mini-block, and MUST NOT reconstruct or expose it if any rule rejects:

- The transaction fragment's `transactions` and the receipt fragment's `receipts` have different lengths -> `reject`: every transaction in a mini-block has exactly one receipt.
- Some `receipt.cumulative_gas_used < block_cumulative_gas_used` -> `reject`: cumulative gas within an EVM block is non-decreasing, so a lower value means the receipts are not the receipts of this position in this block.
- The state-change fragment's `salt_deltas` cannot be applied to the state the node holds after the preceding mini-block -> `reject`: the deltas are not the deltas of this position in this chain.
- The active spec at `ctx.timestamp` requires a [mini-block header signature](blocks.md#mini-block-header-signing) and the transaction fragment carries no `header_signature`, or the carried signature does not recover the key authorized for `ctx.block_number` -> `reject`.

A node that rejects a mini-block MUST NOT reconstruct any later mini-block of the same EVM block: the running cumulative-gas and state-application state that the rejected mini-block would have advanced is a prerequisite for its successors, so continuing would produce mini-blocks that are individually well-formed and collectively wrong.
The EVM block itself is still assembled and checked per [EVM Block Assembly](#evm-block-assembly), which is where such a failure is detected against the sequencer's own commitments.

State deltas MUST be applied in ascending mini-block order, and MUST be applied exactly once each; applying a delta twice, or out of order, produces a state that no signed commitment covers and that the [EVM block](#evm-block-assembly) state-root check will reject only after the incorrect state has already been exposed.

## EVM Block Assembly

An EVM block is assembled from its signed header plus the accumulated contents of its mini-blocks, carried by their fragments: the concatenation, in ascending payload-number order, of every mini-block's transactions and receipts, and the merge, in the same order, of every mini-block's SALT deltas, new bytecode, and buckets to canonicalize.

A node MUST NOT assemble an EVM block before receiving its signed header, and MUST consider the block's fragments complete when, for each stream it requires, it has received `block_metadata.mini_block_count` fragments for that block.

A node MAY additionally treat a block's fragments as complete once, for every stream it requires, it has received a fragment for a **higher** block number: a stream that has moved on will never deliver another fragment for the earlier block, so waiting for the missing fragments cannot succeed.
A node that completes a block this way MUST NOT present the block's contents as complete; it MUST instead treat the missing contents as empty, so that the consistency rules below reject the block rather than committing a truncated one.

A node MUST apply the following rules to an assembled EVM block, in order, and MUST NOT commit the block if any rule rejects:

- `header.parent_hash` does not equal the hash of the last block the node committed -> `reject` as a reorg: see [Reorg Resolution](#reorg-resolution).
- The transactions root recomputed over the accumulated transactions does not equal `header.transactions_root` -> `reject` as a reorg.
- The receipts root recomputed over the accumulated receipts does not equal `header.receipts_root` -> `reject` as a reorg.
- `hash(encode(merged_salt_deltas))` does not equal `block_metadata.salt_deltas_hash` -> `reject` as a reorg: a fragment was lost, duplicated, corrupted, or merged out of order.
- The state root computed by applying the merged deltas to the node's state does not equal `header.state_root` -> `reject`: the streamed state transition is not the one the sequencer committed to.

Where a block's `buckets_to_canonicalize` is non-empty, canonicalization changes the delta representation, so a node MUST recompute the merged deltas from the resulting state updates before hashing them for the `salt_deltas_hash` comparison; comparing the pre-canonicalization merge would reject a valid block.

A node MUST NOT commit an EVM block that carries no state deltas: a consumer of this protocol derives the block's post-state from the streamed deltas and does not re-execute the block's transactions to obtain it.

The state root check is the strongest check in this protocol — it validates the entire streamed state transition against a value the sequencer signed — and it is the only one that a node reconstructing state from deltas cannot substitute anything weaker for.

## Requests and Responses

Every stream defined earlier is push-only: the sequencer publishes at its own pace, and a node has no way to ask it to repeat, skip ahead, or resolve a conflict.
That is enough for a node that is already caught up and never loses data, but not for one that fell behind, discarded a payload, detected a divergent chain, or started with no state at all — [Backfill](#backfill), [Reorg Resolution](#reorg-resolution), and [State Snapshot Bootstrap](#state-snapshot-bootstrap) each define request/response pairs for exactly one of those situations.
This section specifies the mechanism they share: how a request and its response are correlated, and the shapes a response can take.

`request_id` is distinct from [`message_id`](#wire-envelope): `message_id` is the one-byte wire tag identifying a message's shape and is the same for every instance of that message type, while `request_id` is a per-request value inside the payload, chosen by the requester to correlate a specific response with the specific request that triggered it.

`request_id` is always the first item of a request or response body; a node MUST echo the `request_id` of the request in the response that answers it, and MUST NOT treat a response whose `request_id` matches no outstanding request as an answer to any request.

### Requests

Every request body has the same general form, `rlp([request_id, request])` — a two-item list, where `request` is a nested RLP list holding the fields specific to the request type, defined where that type is specified, one level deeper than `request_id`.
A [fetch response](#fetch-responses) is shaped the other way — its fields are flat, in the same list as `request_id` — so neither shape may be inferred from the other.
Five request types take this form and are each answered by exactly one [fetch response](#fetch-responses):

| Request                     | `message_id` | Specified in                                          |
| --------------------------- | ------------ | ----------------------------------------------------- |
| Bucket-range request        | `0x1A`       | [State Snapshot Bootstrap](#state-snapshot-bootstrap) |
| Fragment-range request      | `0x1B`       | [Backfill](#backfill)                                 |
| Partial-block-range request | `0x1C`       | [Backfill](#backfill)                                 |
| Child-block request         | `0x1D`       | [Reorg Resolution](#reorg-resolution)                 |
| Ancestor-headers request    | `0x1E`       | [Reorg Resolution](#reorg-resolution)                 |

The [subscribe request](#stream-subscription) takes the same form but is answered by a subscribe response instead.

### Fetch Responses

Every response is a single flat RLP list, `rlp([request_id, type_id, ...])`, where `type_id` is encoded as an RLP integer — not a raw byte, unlike [`message_id`](#wire-envelope) — and selects one of six shapes for the fields that follow:

| Type id | Response                    | Answers                                                         |
| ------- | --------------------------- | --------------------------------------------------------------- |
| `1`     | Bucket range with proof     | Bucket-range request                                            |
| `2`     | Signed header range         | Fragment-range request on the `Header` stream                   |
| `3`     | State-change fragment range | Fragment-range request on the `StateChange` stream              |
| `4`     | Transaction fragment range  | Fragment-range request on the `Transaction` stream              |
| `5`     | Receipt fragment range      | Fragment-range request on the `Receipt` stream                  |
| `6`     | Partial block range         | Partial-block-range, child-block, and ancestor-headers requests |

A requester MUST reject a response whose type id does not correspond to the request it answers, and MUST reject a response carrying a type id outside `1`–`6`.
The order in which a response's items arrive is fixed by the request that asked for them, and is specified with that request.

#### Bucket Range with Proof

Type `1` answers a [bucket-range request](#state-snapshot-bootstrap) and carries one contiguous run of SALT buckets together with the signed header that dates them.

| Field          | Type                                      | Description                                                                                                                                           |
| -------------- | ----------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `bucket_range` | Bucket range, tabulated below             | The buckets and the bytecode they reference.                                                                                                          |
| `header`       | [Signed header](#signed-header), optional | The signed header of the block whose post-state the buckets reflect. A responder holding nothing for the requested range omits it.                    |
| `proof`        | byte string                               | Reserved for a proof binding the buckets to that block's state; no such proof is defined — see [State Snapshot Bootstrap](#state-snapshot-bootstrap). |

`bucket_range` carries:

| Field       | Type             | Description                                                                                                                                             |
| ----------- | ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `version`   | `uint64`         | Number of the block whose post-state these buckets reflect; equals `header`'s block number.                                                             |
| `start`     | `uint32`         | First bucket id in the range.                                                                                                                           |
| `end`       | `uint32`         | Last bucket id in the range, inclusive.                                                                                                                 |
| `buckets`   | SALT buckets     | The buckets' key-value contents and per-bucket metadata — a SALT-defined structure, see [Bucket Range Encoding](salt/buckets.md#bucket-range-encoding). |
| `bytecodes` | list of bytecode | The bytecode the returned buckets reference.                                                                                                            |

A node MUST encode the response as:

```text
encode(fetch_response) = rlp([request_id, 1, encode(bucket_range), has_header_flag, header?, proof])

encode(bucket_range) = rlp([version, start, end, buckets, bytecodes])
```

where `has_header_flag` is a single raw byte, `0x01` if a signed header follows and `0x00` if not.
Type `1` is the only response whose fields are a bare concatenation rather than one nested list.
`encode(bucket_range)` is nonetheless a plain RLP list, and the `buckets` item inside that list is a bare SALT-defined structure rather than a self-delimiting RLP item, so a decoder needs the SALT bucket encoding to parse the list at all.

The checks a requester applies to this response are specified in [State Snapshot Bootstrap](#state-snapshot-bootstrap).

#### Signed Header Range

Type `2` answers a [fragment-range request](#backfill) on the `Header` stream: `rlp([request_id, 2, rlp([item, ...])])`, where each item is a [signed header](#signed-header).

#### State-Change Fragment Range

Type `3` answers a [fragment-range request](#backfill) on the `StateChange` stream: `rlp([request_id, 3, rlp([item, ...])])`, where each item is a [state-change fragment](#state-change-fragment).

#### Transaction Fragment Range

Type `4` answers a [fragment-range request](#backfill) on the `Transaction` stream: `rlp([request_id, 4, rlp([item, ...])])`, where each item is a [transaction fragment](#transaction-fragment).

#### Receipt Fragment Range

Type `5` answers a [fragment-range request](#backfill) on the `Receipt` stream: `rlp([request_id, 5, rlp([item, ...])])`, where each item is a [receipt fragment](#receipt-fragment).

#### Partial Block Range

Type `6` answers a [partial-block-range request](#backfill), a [child-block request](#reorg-resolution), and an [ancestor-headers request](#reorg-resolution): `rlp([request_id, 6, rlp([item, ...])])`, where each item is a partial block.

A partial block carries a block's signed header plus whichever of that block's contents the answered request selected.

| Field          | Type                            | Description                                                                                                                                     |
| -------------- | ------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| `header`       | [Signed header](#signed-header) | The block's signed header, encoded exactly as the `Header` stream carries it.                                                                   |
| `transactions` | list of transactions, optional  | The block's transactions, in execution order, with no sender list. Present only if the request selected them.                                   |
| `receipts`     | list of receipts, optional      | The block's receipts, in execution order, **without** their bloom filters. Present only if the request selected them.                           |
| `salt_deltas`  | SALT deltas, optional           | The block's merged state deltas, encoded per [SALT delta encoding](salt/trie.md#encoding). Present only if the request selected them.           |
| `bytecodes`    | list of bytecode, optional      | The bytecode the block's new code hashes refer to, one entry per new code hash in `salt_deltas`, in the same order. Present exactly when it is. |

A node MUST encode a partial block as:

```text
encode(partial_block) =
    encode(signed_header) ++ has_tx_flag ++ transactions? ++ has_receipt_flag ++ receipts?
    ++ has_state_changes_flag ++ salt_deltas? ++ bytecodes?
```

where each `has_*_flag` is a single raw byte, `0x01` if the field or fields that follow it are present and `0x00` if not.
`salt_deltas` and `bytecodes` share one flag and are written as two separate items, not as a nested pair; each other optional field has its own flag.

A partial block does not carry its transactions and receipts the way the [transaction fragment](#transaction-fragment) and [receipt fragment](#receipt-fragment) carry theirs, and the two MUST NOT be treated as interchangeable:

- `transactions` carries no sender list, so a consumer MUST recover each sender itself rather than reading one off the response.
- `receipts` carries each receipt **without** its bloom filter, so a consumer MUST derive each bloom from that receipt's logs before it can recompute a receipts root.

## Backfill

A node that is behind, or that discarded a payload, recovers by requesting the missing data directly.
Two request forms exist, and both are answered by a [fetch response](#fetch-responses).

**Fragment-range request** (`rlp([request_id, [stream_id, start, end]])`) asks for the payloads of the stream identified by `stream_id` (per [Streams and Payloads](#streams-and-payloads)) whose `payload_number` lies in `[start, end]` — `start` and `end` are themselves `payload_number` values, per [Payload Numbering](#payload-numbering).
It is answered by one of four [fetch responses](#fetch-responses) — type id `2`, `3`, `4`, or `5` depending on which stream was requested — each an RLP list of that stream's payload type, already defined in [Payload Encodings](#payload-encodings).
A responder MAY return fewer payloads than requested, and MUST return them in ascending payload-number order starting at `start`.
A requester MUST apply the following rules to the response:

- The response's stream does not equal the requested `stream_id` -> `reject`.
- The response is empty -> `reject`: an empty response is not evidence that the range does not exist.
- Some payload's `payload_number` does not equal the next expected number, starting at `start` and incrementing by one -> `reject`: the returned payloads MUST be contiguous from `start`.
- Some payload's `payload_number` exceeds `end` -> `reject`.
- Some payload fails [Verification](#verification) -> `reject`.

**Partial-block-range request** (`rlp([request_id, [start, end, stream_mask]])`) asks for whole blocks rather than fragments, for block numbers in `[start, end]`, where `stream_mask` is the bitmask (per [Streams and Payloads](#streams-and-payloads)) of which block contents the requester needs.
It is answered by the partial-block-range [fetch response](#fetch-responses) (type id `6`).
A responder MAY return fewer blocks than requested and MUST return them in ascending block-number order starting at `start`.
A requester MUST apply the following rules:

- The response is empty -> `reject`.
- The response contains more blocks than the requested range holds -> `reject`.
- Some block's `header.block_number` does not equal the block number implied by its position in the response, starting at `start` -> `reject`.
- Some block's signed header fails [Verification](#verification) -> `reject`.
- `stream_mask` requests a content field that a returned block omits -> `reject`.
- `stream_mask` requests transactions and the transactions root recomputed over the returned transactions does not equal `header.transactions_root` -> `reject`.
- `stream_mask` requests receipts and the receipts root recomputed over the returned receipts does not equal `header.receipts_root` -> `reject`.
- `stream_mask` requests state changes and `hash(encode(salt_deltas))` does not equal `block_metadata.salt_deltas_hash` -> `reject`.
- `stream_mask` requests state changes and the returned bytecode list does not have one entry per new code hash in `salt_deltas`, in the same order, each hashing to its code hash -> `reject`.

A block obtained this way is already verified against its signed header when the response is validated, so a node MAY skip the transactions-root and receipts-root rules of [EVM Block Assembly](#evm-block-assembly) when committing it; it MUST still apply the parent-hash and state-root rules, which the response validation does not cover.

A node MUST NOT advance its committed position from a rejected response, and MUST treat a rejected response as a failure of that request rather than as evidence about the data itself.

## Reorg Resolution

An EVM block whose `parent_hash` does not match the node's last committed block hash, or whose contents do not match its signed header, means the node's committed chain and the block's chain differ.
A node MUST NOT commit such a block, and MUST resolve the divergence before resuming, using two request forms.

**Child-block request** (`rlp([request_id, [parent_number, parent_hash, parent_timestamp, stream_mask]])`) asks a peer for the block whose parent is `(parent_number, parent_hash)`, where `stream_mask` is the bitmask (per [Streams and Payloads](#streams-and-payloads)) of which block contents the requester needs and `parent_timestamp` is reserved and unread.
A responder that holds a block at `parent_number` with hash `parent_hash` MUST answer with the block at `parent_number + 1` if it has one.
A responder that holds a **different** block at `parent_number` MUST instead answer with that block's header followed by its ancestors' headers in descending block-number order, which tells the requester its own chain diverged.

A requester MUST interpret the response as follows, in order:

- The response is empty -> the peer knows neither the child nor a competing block at that height; the divergence is unresolved and the requester MUST NOT change its committed chain.
- The response holds exactly one block, whose `header.block_number == parent_number + 1` and whose `header.parent_hash == parent_hash` -> the child block, which the requester MUST validate as a partial-block response per [Backfill](#backfill) before committing.
- The response's first header has `header.block_number == parent_number`, a hash different from `parent_hash`, and a valid payload signature -> a competing chain, whose remaining headers the requester MUST validate as an ancestor-header response, below, starting from the expected pair `(parent_number - 1, first_header.parent_hash)`.
- Otherwise -> `reject`.

**Ancestor-headers request** (`rlp([request_id, [start_number, start_hash]])`) asks for the header at `(start_number, start_hash)` and its ancestors, in descending block-number order.
The request carries no count: a responder MAY return anywhere from one header (the starting one alone) to as many contiguous ancestors as it holds and chooses to include, which is why reaching the common ancestor may take more than one request.
A requester MUST apply the following rules to the returned header sequence, walking it in order and starting from the expected `(start_number, start_hash)`:

- The response is empty -> `reject`.
- A header's block number or hash does not equal the expected pair -> `reject`.
- A header fails [Verification](#verification) -> `reject`.
- Otherwise, the next expected pair is `(header.block_number - 1, header.parent_hash)`, and the walk continues.

A node MUST continue walking back — issuing further ancestor-header requests as needed — until it reaches a header whose hash matches the block it holds at that number; that block is the common ancestor.
The node MUST then discard every block it committed above the common ancestor and resume from it.
A node MUST NOT resume from a block whose ancestry it has not verified back to a block it already holds, since doing so would splice an unrelated chain onto its own.

## State Snapshot Bootstrap

A node with no state does not need to replay the chain from genesis; it can download a snapshot of the state itself and then follow the stream.
The snapshot is transferred as [buckets](salt/buckets.md), each labelled with a [version](#bucket-range-with-proof).

Buckets can arrive from different peers at different versions over the course of a download, so a node MUST NOT count a bucket as held unless its [version](#bucket-range-with-proof) matches every other bucket it already holds: a bucket that arrives at a higher version than what's already held MUST NOT be counted until the older buckets are brought forward to match.
How a node brings its held buckets to one consistent version — by consuming the `StateChange` and `Header` streams concurrently with the download, by re-fetching stale buckets afterward, or by any other means — is not specified; only the completeness check below is normative.

**Bucket-range request** (`rlp([request_id, [version, start, end]])`) asks for the buckets with ids in `[start, end]` at a version no older than `version` — typically the version of the buckets the requester already holds.
The response carries the buckets, the bytecode they reference, and the signed header of the block whose post-state they reflect.

A requester MUST apply the following rules to the response:

- The response carries no signed header -> `reject`: without it, the buckets' version is unauthenticated.
- The buckets' `start` does not equal the requested `start`, or their `end` exceeds the requested `end` -> `reject`.
- The signed header's block number does not equal the buckets' version, or is below the requested `version` -> `reject`.
- The signed header fails [Verification](#verification) -> `reject`.
- The number of bucket key-value sets does not equal the number of bucket metadata entries -> `reject`.
- The set of code hashes referenced by the buckets does not equal the set of hashes of the returned bytecode -> `reject`.

The response additionally carries a proof field, reserved for a proof that the returned buckets belong to the state the signed header commits to.
No such proof is defined by this specification, and the rules above deliberately do not check one: the only thing that binds the downloaded buckets to a committed state is the state-root comparison performed once the snapshot is complete, below.

A node MUST NOT consider the snapshot complete until it holds every non-metadata bucket of the [SALT bucket space](salt/buckets.md#bucket-structure).
On completion, a node MUST rebuild the SALT commitment structures from the downloaded buckets and MUST verify that the resulting state root equals the `state_root` of the signed header at the version reached; a mismatch means the snapshot is not a valid state and the node MUST NOT treat itself as synced.
Where the active spec defines a [withdrawals root](https://specs.optimism.io/protocol/isthmus/exec-engine.html#l2tol1messagepasser-storage-root-in-header) in the block header, a node MUST perform the same check for the withdrawal commitment it rebuilds, against that header field.

A node that has bootstrapped MUST NOT serve, or claim to hold, the history preceding its snapshot version: it never received that history, and the state it holds is a snapshot, not a replay.

## Constants

| Constant                | Value | Description                                                                                                  |
| ----------------------- | ----- | ------------------------------------------------------------------------------------------------------------ |
| `BLOCK_METADATA_LENGTH` | `202` | Length in bytes of the fixed-length [block metadata](#block-metadata) encoding.                              |
| `TREE_NUMBER`           | `3`   | Number of broadcast trees each stream is distributed over — see [Stream Subscription](#stream-subscription). |

Stream identifiers and mask bits, wire message ids, and response type ids are structural wire enumerations rather than configurable values, and are tabulated in [Streams and Payloads](#streams-and-payloads) and [Wire Envelope](#wire-envelope).
Response size caps, request concurrency limits, retry counts, and the fan-out and peer-selection parameters of the propagation topology are implementation choices and are deliberately not given values here; see the [Normativity Boundary](overview.md#normativity-boundary).

## Rationale

**Why the state transition is streamed instead of the block being re-derived by execution.**

Streaming the deltas makes a consumer's cost proportional to the size of the state change rather than to the cost of computing it, and makes sub-block-time visibility available to consumers without high requirement of computation power as sequencer needs.
The cost is that the streamed deltas must be independently authenticated and independently checked against the block's committed state root, which is what [Payload Authentication](#payload-authentication) and [EVM Block Assembly](#evm-block-assembly) do.

**Why one mini-block is split across three separately signed payloads rather than being sent as one message.**

Different kinds of node need different parts of a mini-block.
A node serving the full RPC surface subscribes to every stream; a node that only tracks state takes state changes and headers; a node that only follows transactions takes transactions and receipts and never applies a delta.
Splitting the mini-block lets each subscribe to exactly what it serves — which is what the four independent streams of [Streams and Payloads](#streams-and-payloads) and the `stream_mask` of a [partial-block-range request](#backfill) exist to express — where one message would force every consumer to carry all of it on the hottest path in the protocol.

Signing each fragment separately is what makes the split usable rather than merely cheaper.
Each fragment authenticates on its own, so a consumer holding one of the three can verify it without waiting for the other two, and a gap on one stream stalls that stream's delivery rather than all three; [Payload Delivery](#payload-delivery)'s in-order obligation is per stream for the same reason.
The cost is that nothing binds a mini-block's three fragments to each other beyond their shared `payload_number`, so a mini-block is only as consistent as the checks in [Mini-Block Assembly](#mini-block-assembly) make it, and a mismatch that survives those is caught by the block's own commitments one EVM block later.

**Why payload signatures use BLAKE3 while the mini-block header signature uses `keccak256`.**

A payload signature is verified only by consuming nodes, never inside the EVM, so its hash is chosen for speed: a consumer verifies three payload signatures per mini-block on the ~10 ms cadence that decides how quickly streamed state becomes visible, and BLAKE3 is faster than `keccak256`.

The [mini-block header signature](blocks.md#mini-block-header-signing) may have to be verified on chain, and the EVM's only signature primitive, `ecrecover`, is defined over a `keccak256` digest — any other hash would make that signature uncheckable there, and would break the Ethereum tooling that assumes `keccak256` throughout.

**Why `safe` and `finalized` travel in block metadata.**

A node that follows the chain through this protocol has no L1 view of its own.
It never derives from L1, so it cannot work out which L2 blocks a finalized L1 block covers, and would otherwise have no safe or finalized marker to expose at all.
The sequencer does hold that view, and the [signed header](#signed-header) is a payload every consumer already fetches for every block, so carrying the two markers there costs one more field on a message that must arrive anyway rather than a second channel to keep in step with the first.

Putting them inside the signed metadata keeps a relaying peer from altering them in transit: the signature establishes that the sequencer published these values, not that they are true, which is why [Block Metadata](#block-metadata) forbids reading them as a verified L1 finality claim.
A consumer that has its own L1 view has no use for them and ignores the fields.

## Security Considerations

**An unauthenticated payload is a state-corruption primitive, not just bad data.**

A consumer of this protocol adopts streamed SALT deltas as its state without re-executing anything.
A node that applies a state-change fragment before completing [Verification](#verification) has applied an arbitrary attacker-chosen state transition — a credited balance, a rewritten storage slot — and will serve it over RPC.
The [EVM block state-root check](#evm-block-assembly) detects the corruption at the end of the block, after the incorrect state has already been visible for up to the whole block.
Verifying every payload per [Verification](#verification) before applying it, not detecting corruption afterwards, is what bounds this.

**A malicious sequencer can show different nodes different chains.**

Two payloads signed by the authorized key for the same stream position both pass [Verification](#verification), and a node keeps whichever arrived first, so nodes can diverge.
Nothing here detects the conflict or records evidence of it, and [Reorg Resolution](#reorg-resolution) will follow either chain, since the sequencer can sign valid headers for both.
Which chain is canonical is settled only on L1, by the [rollup and settlement pipeline](../rollup/overview.md).

**Backfill and reorg responses come from peers, not from the sequencer, so their shape must be checked against the validity rules in [Backfill](#backfill) and [Reorg Resolution](#reorg-resolution) before their contents are used.**

A peer answering a request controls how many items it returns and what they are.
A requester that does not check contiguity and range bounds can be walked past the range it asked for; one that does not verify each item's payload signature can be handed a fabricated chain; one that accepts an empty response as authoritative can be convinced that data which exists does not.
The [ancestor-header walk](#reorg-resolution) is the sharpest case: a node that accepts a hash-linked header chain without checking that it terminates at a block the node already holds can be induced to discard its own committed chain and adopt an unrelated one.

**A bootstrapped node's state is only as good as its final root check.**

Snapshot buckets arrive from arbitrary peers, and the per-response checks only bind a bucket range to a signed header's version — they do not prove the bucket contents belong to that state.
The rebuild-time comparison of the reconstructed state root against the signed header's `state_root`, required by [State Snapshot Bootstrap](#state-snapshot-bootstrap), is the single check that makes the whole snapshot sound.
A node that skips it, or that treats a mismatch as recoverable, has adopted an attacker-chosen state at a legitimately signed block number.

**This protocol carries no finality guarantee.**

Beyond the limits the [Trust Model](#trust-model) states, nothing here establishes that what the sequencer published will remain canonical.
A consumer that treats an assembled mini-block or a committed EVM block as final is relying on sequencer behavior; finality still depends on the containing block reaching finality on L1 via the [rollup and settlement pipeline](../rollup/overview.md).

## Spec History

- [Rex5](../upgrades/rex5.md) — Added the trailing `header_signature` field to the [transaction fragment](#transaction-fragment), carrying the [mini-block header signature](blocks.md#mini-block-header-signing) that consumers verify against the header they reconstruct, and changed the unit of the `timestamp` field carried by every per-mini-block fragment from milliseconds to microseconds. Before Rex5 the field is absent and mini-blocks are not signed.
