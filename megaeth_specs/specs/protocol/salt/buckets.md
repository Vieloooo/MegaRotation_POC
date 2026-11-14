---
description: Normative definition of SALT buckets — key mapping, bucket capacity, metadata, and growth semantics.
spec: Rex5
---

# Bucket Model

SALT maps every account and storage slot to a location inside a fixed space of 16,777,216 **buckets**, and stores it there as an entry of a resizable, open-addressed hash table with a canonical, insertion-order-independent layout.
This page specifies that mapping, the internal structure and growth of a bucket, and the canonicalization step that keeps a bucket's final layout independent of the order in which an [EVM block](https://docs.megaeth.com/spec/glossary#evm-block)'s updates were applied to it.

## Motivation

A standard Merkle-Patricia trie commits to state as a sparse, variable-depth radix trie: each read or write touches a chain of trie nodes proportional to key length, and each trie node's storage location is effectively random, so computing a new state root after a batch of writes requires random disk I/O proportional to the number of touched trie nodes.
That cost is what SALT exists to remove: a state representation whose every commitment fits in memory, and whose commitment updates require no random disk access at all.

A second problem is representation stability.
If a key-value store's on-disk or in-memory layout depended on the order updates were applied — insert A then B differently laid out than insert B then A — two nodes that received the same net set of changes through different batching or delivery order could compute different commitments over identical logical state, breaking the very property a state root exists to provide.
SALT's bucket layout is therefore required to be **strongly history-independent**: a canonical function of what the bucket currently holds — its entry set, at its current capacity and nonce — never of the sequence of operations that produced it.

## Bucket Structure

SALT's bucket space is fixed at `NUM_BUCKETS` buckets (16,777,216; see [Constants](#constants)): that count never changes, though an individual bucket's own capacity does.
A bucket is an open-addressing hash table over a resizable array of **slots**, each slot holding at most one entry.
A **data bucket**'s occupied slot holds one piece of MegaETH state — one account's record or one storage slot's value — which is exactly the chain data a node needs to answer any read and that a state root must authenticate; [Bucket Metadata Storage](#bucket-metadata-storage) introduces the other kind of bucket, whose slots hold something else entirely.

The rest of this page works through that structure in order: [Encoding State](#encoding-state) first defines how a piece of state is packed into the bytes a slot actually stores, [Mapping State to Buckets](#mapping-state-to-buckets) defines which bucket a given piece of state resolves to, [Storing State in a Bucket](#storing-state-in-a-bucket) defines how it is physically placed into and located within that bucket's slots, and the sections after that cover a bucket's growth, the block-level cleanup that growth requires, and the wire format used to transfer many buckets at once.

## Encoding State

This section builds up what a slot actually stores in three steps: the **plain key** every entry is addressed by, the **plain value** it maps to, and finally the `SaltValue` byte string that packs the two together.

Every piece of MegaETH state SALT stores is keyed by a **plain key**, one of:

- an account key, the account's 20-byte address; or
- a storage key, a 52-byte string formed by the account's 20-byte address followed by the 32-byte storage slot.

The plain key's byte encoding MUST be derived as:

```text
plain_key(account)         = address
plain_key(storage, slot)   = address ++ slot
```

where `++` is byte concatenation, `address` is the 20-byte account address, and `slot` is the 32-byte storage slot.
No other plain key shapes exist for MegaETH state — an account's SALT-visible state is exactly its account record and its storage slots, each addressed independently; SALT holds no per-account substructure and no account-scoped storage root — though [Bucket Metadata](#bucket-metadata) below defines one further, SALT-internal plain-key shape, the 12-byte string a `BucketMeta` record is keyed by.

Every plain key maps to a **plain value**.
A plain value MUST be encoded as:

```text
plain_value(account, code_hash == EMPTY_CODE_HASH) = nonce_be ++ balance_be
plain_value(account, otherwise)                    = nonce_be ++ balance_be ++ bytecode_hash
plain_value(storage)                               = value_be
```

where `nonce_be` is the account's nonce as an 8-byte big-endian integer, `balance_be` is the account's balance as a 32-byte big-endian integer, `bytecode_hash` is the 32-byte `keccak256` hash of the account's code, `EMPTY_CODE_HASH` is the `keccak256` hash of the empty byte string (see [Constants](#constants)), and `value_be` is the 32-byte big-endian storage value.
The branch condition is the account's code hash, not any notion of account type: an account whose code hash equals `EMPTY_CODE_HASH` encodes as the 40-byte **short-form account record**, and an account with any other code hash — including an [EIP-7702](https://eips.ethereum.org/EIPS/eip-7702)-delegated EOA, whose code hash covers its delegation designator — encodes as the 72-byte **long-form account record** carrying that code hash.
Which of the two applies is tagged implicitly by `value_len` in the enclosing `SaltValue` below — `40` or `72` — so the encoding is self-describing from that length alone, not from any context external to the record.
Storage is always 32 bytes, never a shorter big-endian-trimmed encoding.

Bytecode itself is not stored as a SALT value: an account record carries only `bytecode_hash`, and the bytecode bytes are carried and addressed separately, keyed by that hash, per [State-Change Fragment](../state-sync.md#state-change-fragment)'s `new_bytecodes`.

A plain key and its plain value are packed together into a **`SaltValue`**, the byte string a bucket's slot actually stores:

```text
SaltValue = key_len ++ value_len ++ key ++ value
```

Despite the name, a `SaltValue` carries the full plain key alongside the plain value, not the value alone: [Storing State in a Bucket](#storing-state-in-a-bucket)'s hash table slot position does not by itself identify which key occupies it, so the occupant must be self-identifying.
`key_len` and `value_len` are each a single byte giving the length of `key` and `value`, and `key` and `value` are the plain key's and plain value's own byte encodings.
A `SaltValue` MUST NOT exceed `MAX_SALT_VALUE_BYTES` total bytes (`2 + key_len + value_len`); the largest legal value is a long-form account record, `2 + 20 + 72 = 94` bytes.
A `SaltValue` MUST be stored as exactly `MAX_SALT_VALUE_BYTES` bytes: the `2 + key_len + value_len`-byte encoding above, right-padded with zero bytes out to that fixed width.
`key_len` and `value_len` alone determine how many of those bytes are meaningful; [Bucket Range Encoding](#bucket-range-encoding) trims this padding for the wire, and the commitment over an occupied slot hashes only the trimmed `key ++ value` bytes — [Slot Hashing](trie.md#slot-hashing) excludes the two length prefixes and the padding — but every other use of a `SaltValue` — a bucket slot's actual content, and the byte string [SALT Trie](trie.md#salt-deltas) XORs to compute an `updates` delta — operates on the full, padded `MAX_SALT_VALUE_BYTES`-byte form.

## Mapping State to Buckets

This section defines which bucket a given plain key resolves to.
A plain key is mapped to a **`SaltKey`**, a 64-bit value combining a 24-bit **bucket id** (`BucketId`) in its high bits and a 40-bit **slot id** in its low bits:

```text
SaltKey = (bucket_id << BUCKET_SLOT_BITS) | slot_id
```

`bucket_id` MUST be computed as:

```text
bucket_id(plain_key) = (ahash(plain_key) mod NUM_KV_BUCKETS) + NUM_META_BUCKETS
```

`ahash` is not the upstream [AHash](https://github.com/tkaitchuck/aHash) crate's default hasher: that hasher picks between multiple internal algorithms based on runtime CPU features, and is not guaranteed stable across platforms, versions, or future AHash releases.
`ahash` is instead one specific, frozen construction — AHash v0.8.12's non-SIMD "fallback" folded-multiply hasher, permanently fixed rather than tracking upstream AHash — keyed with the four fixed seeds `HASHER_SEEDS` (see [Constants](#constants)) and applied to the plain key's raw bytes.
This frozen construction MUST be implemented exactly and reproduced bit-for-bit, since `bucket_id` determines where an entry physically lives, which determines every bucket's contents and therefore every commitment computed over it, per [SALT Trie](trie.md).
As an unambiguous conformance check, `ahash` keyed with `HASHER_SEEDS` MUST map the byte string `hello` to `1027176506268606463` and `world` to `2337896903564117184`.
The `+ NUM_META_BUCKETS` offset means `bucket_id` always falls in `[NUM_META_BUCKETS, NUM_BUCKETS)`; a `bucket_id` below `NUM_META_BUCKETS` never holds plain-key data, only [bucket metadata](#bucket-metadata).

`slot_id` is not a hash of the plain key; it is assigned by the SHI probe sequence at insertion time, specified next in [Storing State in a Bucket](#storing-state-in-a-bucket).

## Storing State in a Bucket

With `bucket_id` fixed, this section covers what remains: [Bucket Metadata](#bucket-metadata) first defines what a bucket tracks about itself and where that tracking data itself lives, and [SHI Hash Table](#shi-hash-table) then defines the algorithm that uses it to place, find, and remove a `SaltValue` within one of the bucket's slots — finishing what [Mapping State to Buckets](#mapping-state-to-buckets) started by assigning `slot_id`, and covering what changes once a bucket outgrows its initial capacity.

### Bucket Metadata

`BucketMeta` is the two pieces of data that a bucket carries about itself, needed to place, find, or remove any entry within it: a **nonce** and a **capacity**.
These are not implicit state a node derives on the fly; they are themselves data a node must store and commit to, specified here.

| Field      | Type     | Description                                                                        |
| ---------- | -------- | ---------------------------------------------------------------------------------- |
| `nonce`    | `uint32` | Selects the probe-sequence hash for this bucket; currently always `0` (see below). |
| `capacity` | `uint64` | Number of slots currently allocated to this bucket.                                |

No operation defined on this page or in [SALT Trie](trie.md) ever changes a bucket's `nonce`: [Resizing](#resizing) and [Canonicalization](#canonicalization) both change only `capacity`, carrying `nonce` through unchanged.
A `nonce` different from `0` is therefore not reachable through any currently-specified protocol behavior; the field exists in `BucketMeta`'s encoding to leave room for a future mechanism to re-key a specific bucket's probe sequence (for example, to recover from the adversarial concentration [Security Considerations](#security-considerations) describes) without that mechanism being specified today.

`BucketMeta` MUST be encoded as a `SaltValue` whose plain key is the 12-byte string `nonce_le ++ capacity_le` — `nonce` and `capacity` each little-endian — and whose plain value is empty (`key_len = 12`, `value_len = 0`, so 14 encoded bytes before padding), stored at the `SaltKey` given by [Bucket Metadata Storage](#bucket-metadata-storage).
A bucket that has never been resized has implicit metadata `nonce = 0, capacity = MIN_BUCKET_SIZE`; an unpopulated metadata slot MUST be treated as this default rather than as an error.

#### Bucket Metadata Storage

A `BucketMeta` record needs a `SaltKey` of its own to be stored under, distinct from any of the data bucket's own slots; this subsection defines where.
The `NUM_BUCKETS` bucket ids are partitioned into two disjoint ranges by numeric value: `[0, NUM_META_BUCKETS)`, the **metadata buckets**, and `[NUM_META_BUCKETS, NUM_BUCKETS)`, the **data buckets** that [Mapping State to Buckets](#mapping-state-to-buckets) assigns plain keys into.
A metadata bucket's slots hold `BucketMeta` records directly, addressed by slot number alone — not by the [SHI Hash Table](#shi-hash-table) algorithm below, which applies only to data buckets — at a fixed `META_BUCKET_SIZE` capacity that never resizes.

Data bucket `bucket_id`'s metadata MUST be stored at:

```text
bucket_metadata_key(bucket_id) = SaltKey(bucket_id = bucket_id >> MIN_BUCKET_SIZE_BITS, slot_id = bucket_id mod MIN_BUCKET_SIZE)
```

so that metadata bucket `m` holds the metadata for the `MIN_BUCKET_SIZE` data buckets `[m * MIN_BUCKET_SIZE, (m + 1) * MIN_BUCKET_SIZE)`, at slot `bucket_id mod MIN_BUCKET_SIZE` within it.
This formula is only ever applied to a data `bucket_id`, so `m` only ever takes values in `[NUM_META_BUCKETS / MIN_BUCKET_SIZE, NUM_META_BUCKETS)` — metadata buckets `0` through `NUM_META_BUCKETS / MIN_BUCKET_SIZE - 1` (256 of them) are never addressed this way, and no operation ever writes to their slots, though like every metadata slot they still commit as the implicit default `BucketMeta` per [Slot Hashing](trie.md#slot-hashing): `NUM_META_BUCKETS = NUM_BUCKETS / MIN_BUCKET_SIZE` provides one metadata slot per _bucket_, data and metadata alike, which takes 256 more metadata buckets than the `NUM_KV_BUCKETS / MIN_BUCKET_SIZE` metadata buckets the data buckets alone require.

### SHI Hash Table

A bucket's slots MUST be populated according to the following **Strongly History-Independent (SHI)** open-addressing algorithm, so that a bucket's final layout is a canonical function of the set of entries currently present, independent of the order they were inserted or deleted in.

Every bucket has a `nonce` and a `capacity`, defined in [Bucket Metadata](#bucket-metadata) above.
Because a bucket is open-addressed, a plain key is not confined to one fixed slot: it has a **probe sequence**, an ordered list of candidate slots that [Lookup](#lookup), [Insertion](#insertion), and [Deletion](#deletion) below each walk one slot at a time to place, find, or remove that key.
A probe sequence starts at a hash-derived slot and moves forward, wrapping past the bucket's last slot back to slot `0`, until it hits a stopping condition of its own; because it visits each of the bucket's `capacity` slots exactly once before repeating, every walk below terminates within at most `capacity` steps — either at a stopping condition (an empty slot, a match, or, for [Lookup](#lookup), a smaller key), or, in a completely full bucket where no stopping condition ever fires, by exhausting all `capacity` slots, an outcome each operation below defines explicitly.
The `i`-th slot of `plain_key`'s probe sequence within a bucket of a given `nonce` and `capacity` MUST be computed as:

```text
probe(plain_key, i) = (ahash(plain_key ++ nonce_le) mod capacity + i) mod capacity,  for i = 0, 1, 2, ...
```

where `ahash` is the same frozen hash function and fixed seeds used for [bucket assignment](#mapping-state-to-buckets), and `nonce_le` is the bucket's nonce as a 4-byte little-endian integer.
So `probe(plain_key, 0)` is the sequence's starting slot, `probe(plain_key, 1)` is the next slot checked if the first does not resolve the operation, and so on.
`probe(plain_key, i)`'s result is a `slot_id` in `[0, capacity)` — the same `slot_id` [Mapping State to Buckets](#mapping-state-to-buckets) combines with the bucket's `bucket_id` to form the `SaltKey` an occupied slot is ultimately stored under; a probe sequence does not introduce a second, distinct notion of "slot," only a specific visiting order over the bucket's existing ones.

#### Lookup

The probe sequence `probe(plain_key, 0), probe(plain_key, 1), ...` MUST be walked, and, for each slot visited, in order:

- The slot is empty -> the key is absent.
- The slot's entry's key equals `plain_key` -> the key is present, with that entry's value.
- The slot's entry's key is lexicographically less than `plain_key` (comparing the `SaltValue` key bytes) -> the key is absent; probing MUST stop.
- Otherwise (the slot's key is present and lexicographically greater than `plain_key`) -> continue probing the next `i`.

If the walk visits all `capacity` slots without any rule above resolving it — possible only in a completely full bucket every one of whose occupants' keys is greater than `plain_key` — the key is absent and probing stops.
The third rule is what makes non-membership provable without probing the full bucket: [Insertion](#insertion) maintains the invariant that every stored entry is preceded, along its own probe sequence from that sequence's first slot up to the slot it occupies, only by entries with strictly greater keys — so if `plain_key` occupied any later slot of the walked sequence, the slot just seen could not hold a key smaller than `plain_key`.

#### Insertion

To insert or update `(plain_key, plain_value)`, the probe sequence `probe(plain_key, 0), probe(plain_key, 1), ...` MUST be walked, in order, and, at each visited slot `i`:

- The slot is empty -> place `(plain_key, plain_value)` there and stop.
- The slot's entry's key equals `plain_key` -> replace its value with `plain_value` and stop.
- The slot's entry's key is lexicographically less than `plain_key` -> place `(plain_key, plain_value)` in this slot, then continue walking the _same_ probe sequence — `probe(plain_key, i + 1), probe(plain_key, i + 2), ...` — to re-insert the entry that was just displaced, as if it were the value now being inserted.
- Otherwise -> continue to the next `i`.

This is a **displacement chain**: an insertion may relocate multiple existing entries, each pushed further along the same probe sequence that satisfies the same rule, preserving the invariant [Lookup](#lookup) relies on — every stored entry is preceded, along its own probe sequence, only by entries with strictly greater keys.

#### Deletion

To delete `plain_key`: if [Lookup](#lookup) finds no occupied slot for it, the deletion is a no-op.
Otherwise, that slot — call it `gap` — MUST be repeatedly closed:

- Scan the slots `(gap + 1) mod capacity, (gap + 2) mod capacity, ...` — physical slot order, not `plain_key`'s own probe sequence — until either an empty slot is reached, or an occupied slot `s` is reached whose entry's key `k` satisfies `rank(k, s) > rank(k, gap)`, where `rank(k, x)` is the `j` such that `probe(k, j) = x`: the number of steps into `k`'s own probe sequence needed to reach slot `x`.
- If the scan reaches an empty slot first — or visits all `capacity - 1` other slots without reaching an empty slot or a qualifying entry, possible only in a completely full bucket — leave `gap` empty and stop: deletion is complete.
- If the scan reaches such a slot `s` first, move `s`'s entry into `gap`, set `gap = s`, and repeat the scan from the new `gap`.

An entry that satisfies `rank(k, s) > rank(k, gap)` is one whose own probe sequence would have allowed it to occupy `gap` — moving it there keeps every remaining entry's position consistent with the [Insertion](#insertion) rule, as if the deleted entry had never displaced it in the first place.
This retraces and undoes exactly the displacement chain that insertion would have created for the current entry set, so that the bucket after deletion is identical to a bucket that never held the deleted entry.

#### Resizing

A bucket MUST resize in either of two cases:

- After an insertion places an entry into a previously empty slot, the bucket's occupied-slot count exceeds `capacity * BUCKET_RESIZE_LOAD_FACTOR_PCT / 100`; an in-place value replacement ([Insertion](#insertion)'s equal-key rule) leaves the count unchanged and never triggers this case.
- An insertion's probe sequence exhausts all `capacity` slots without finding an empty one — meaning the entry currently being placed (the original entry, or, mid-displacement-chain, the most recently displaced one) has no slot to occupy.

The load-factor case MUST be evaluated only when the bucket's true occupied-slot count is available: a node holding the bucket in full always has it, while [Execution Witness](witness.md)'s stateless replay path has it exactly when every one of the bucket's `capacity` slots is witnessed — which [Witness Contents](witness.md#witness-contents) requires a witness builder to arrange for every bucket the block's insertions resize.
When the count is unavailable, the load-factor check MUST be skipped — not approximated from whatever slots are in view, and not treated as an error — leaving the exhaustion case as the only trigger; [Security Considerations](#security-considerations) describes what a skipped check costs.

On resize:

- `capacity` MUST be recomputed by repeatedly multiplying it by `BUCKET_RESIZE_MULTIPLIER` until the new capacity satisfies `occupied * 100 <= new_capacity * BUCKET_RESIZE_LOAD_FACTOR_PCT`, where `occupied` is the bucket's occupied-slot count before this resize — excluding, in the exhaustion case, the still-unplaced entry that triggered it.
- Every other currently-stored entry MUST be extracted and re-inserted into an initially empty table of the new capacity, using the [Insertion](#insertion) rule above; the re-insertion order is immaterial, per the order-independence property below.
- In the exhaustion case only, once the two steps above complete, the previously-unplaced entry MUST then be inserted into the resized bucket, from the start of its own probe sequence, using the same [Insertion](#insertion) rule.
- A bucket's capacity MUST NOT grow beyond `2^BUCKET_SLOT_BITS`: a `slot_id` cannot address a slot beyond that bound, so this case is not a defined protocol behavior.

Rebuilding from an empty table is history-independent whatever order the entries are re-inserted in: the [SHI Hash Table](#shi-hash-table)'s layout is a function of the entry set, `nonce`, and `capacity` alone, so the resized bucket's layout depends only on its final entry set and new capacity — not on the entries' original insertion order, the re-insertion order chosen, or the resize trigger.
(Re-inserting in descending key order additionally places every entry at the first empty slot of its own probe sequence with no displacement at all, since each entry inserted is smaller than every entry already placed; an implementation MAY rebuild that way, and the result is identical to any other order's.)
Growing past `MIN_BUCKET_SIZE` slots allocates capacity in multiples of `MIN_BUCKET_SIZE`: a grown bucket is split into multiple [segments](trie.md#bucket-subtrees) of `MIN_BUCKET_SIZE` slots each, organized as specified in [SALT Trie](trie.md#bucket-subtrees).
Nothing about the SHI lookup, insertion, or deletion algorithm changes when a bucket has grown this way: `capacity` is simply larger, and the probe sequence still ranges over `[0, capacity)`.

## Canonicalization

The [Resizing](#resizing) rule guarantees a canonical layout for a bucket considered in isolation after each individual insertion or deletion, but a [sequencer](https://specs.optimism.io/glossary.html#sequencer) applies every plain-key change belonging to one [EVM block](https://docs.megaeth.com/spec/glossary#evm-block) — across however many [mini-blocks](https://docs.megaeth.com/spec/glossary#mini-block) that block is split into — to the same bucket before any consumer is expected to reconstruct that block's final state.
If a burst of insertions earlier in the block grows a bucket to accommodate a temporary peak occupancy, and later deletions elsewhere in the same block bring occupancy back down, the bucket's capacity does not automatically shrink back down: [Resizing](#resizing) only ever grows a bucket, never shrinks one on its own.
Left uncorrected, the bucket's final capacity would depend on the intermediate path of updates across the block's mini-blocks, not just on the net entry set the block produces — breaking history independence at the EVM-block granularity even though each individual SHI operation, and each individual mini-block's own updates, preserve it locally.

**Canonicalization** removes that path-dependence, and it runs once per EVM block, not once per mini-block.
Once every mini-block of an EVM block has had its updates applied, in mini-block order, every bucket that was resized at least once anywhere in that block MUST be canonicalized — regardless of which of the block's mini-blocks triggered the resize:

1. Extract that bucket's current entry set in full.
2. Reset the bucket to the metadata it held at the start of the EVM block.
3. Re-insert every extracted entry using the [Insertion](#insertion) rule, allowing [Resizing](#resizing) to trigger exactly as it would for a bucket built up from that starting capacity; as with a resize's rebuild, the re-insertion order is immaterial to the resulting layout.

The resulting bucket depends only on the EVM block's net entry set for that bucket, never on how many mini-blocks the block was split into or how many times the bucket happened to resize while applying them.
A sequencer MUST publish, across an EVM block's mini-blocks' `buckets_to_canonicalize` fields ([State-Change Fragment](../state-sync.md#state-change-fragment)), a union equal to exactly the set of buckets [Resizing](#resizing) resized anywhere in that block.
How that set is split across the block's mini-blocks is not constrained — only the union is consensus-visible; the reference sequencer publishes the whole set in the block's final mini-block, leaving the field empty on every earlier one.
Every node that reconstructs the block's state from its transmitted deltas MUST identify the exact set of buckets requiring this treatment as that union: the set is not derivable from the block's merged delta, and recovering it from the per-mini-block deltas instead would require retaining and inspecting every fragment's delta separately ([Rationale](#rationale)).
A node that re-executes the block and applies the resulting updates itself — [stateless validation](../stateless-validation.md)'s path, which never receives the field — instead derives the same set directly, as the buckets its own application resized.
No validity rule cross-checks the transmitted field against an independently derived resize set — the consequences of a mismatch, in each direction, are described in [Security Considerations](#security-considerations).
[SALT Trie](trie.md#salt-deltas) specifies how canonicalization contributes its own additional entries to the state delta a node hashes and commits to.

## Bucket Range Encoding

With a bucket's internal structure and lifecycle fully specified, one thing remains: how a contiguous span of buckets is serialized to move in bulk between nodes.
A **bucket range** is the key-value and metadata content of a contiguous span of buckets `[start, end]`, exchanged during [state snapshot bootstrap](../state-sync.md#state-snapshot-bootstrap).
`[start, end]` MUST lie entirely within `[NUM_META_BUCKETS, NUM_BUCKETS)`: a bucket range only ever covers data buckets, since a data bucket's own [`BucketMeta`](#bucket-metadata) already travels inline with it in the second group below, and metadata buckets are never fetched or transferred as a range of their own.
[Bucket Range with Proof](../state-sync.md#bucket-range-with-proof) notes that this response is the one that does not fit [Fetch Responses](../state-sync.md#fetch-responses)' general nested-list shape, and delegates the encoding of its `buckets` field to this page.

A bucket range MUST be encoded as:

```text
encode(buckets) =
    rlp(bucket_count) ++ (rlp(kv_count) ++ (rlp(salt_key) ++ compact_value)*)*
    ++ (rlp(nonce) ++ rlp(capacity))*
```

where `*` means zero or more of the item to its left, `bucket_count` is the number of buckets the response actually covers, and:

- The first repeated group lists, for each covered bucket in ascending bucket-id order starting at `start`, its `kv_count` occupied slots, `rlp(kv_count)` followed by that many `(salt_key, compact_value)` pairs in ascending `salt_key` order; every `salt_key` in a bucket's group MUST carry that bucket's id in its `bucket_id` field, a producer obligation no decode-time rule re-checks — [State Snapshot Bootstrap](../state-sync.md#state-snapshot-bootstrap)'s completion-time state-root check is what ultimately catches a violating response.
- `salt_key` denotes the occupied slot's `SaltKey`, RLP-encoded as its underlying `uint64` value — a standard RLP integer, not a nested structure.
- `compact_value` is the occupied slot's `SaltValue`, appended as raw bytes with no RLP header of its own: exactly the `2 + key_len + value_len` leading bytes of the layout from [Encoding State](#encoding-state), trimmed to its used length rather than padded to `MAX_SALT_VALUE_BYTES` — a decoder reads `key_len` and `value_len` as the first two raw bytes and derives the remaining length from them.
- The second repeated group lists that same `bucket_count` buckets' [`BucketMeta`](#bucket-metadata) as `bucket_count` consecutive `(nonce, capacity)` pairs, `rlp(nonce) ++ rlp(capacity)`, in the same ascending bucket-id order — after, not interleaved with, the key-value groups.

`end` MUST equal `start + bucket_count - 1` whenever `bucket_count >= 1`; a responder that cannot fit even the range's first bucket within its response-size budget returns `bucket_count = 0` while reporting `end = start`, the encoding's only representation of an empty response, since the inclusive `[start, end]` notation cannot express an empty span.
A decoder MUST therefore take the number of key-value groups and `(nonce, capacity)` pairs from `bucket_count` as transmitted, never derive it as `end - start + 1`.
A decoder MUST reject a `compact_value` string whose declared `2 + key_len + value_len` exceeds `MAX_SALT_VALUE_BYTES`.
A decoder MUST likewise reject, before the entry is stored, hashed, or otherwise used, a `compact_value` whose `(key_len, value_len)` pair is not one of `(20, 40)`, `(20, 72)`, or `(52, 32)` — the only shapes [Encoding State](#encoding-state) defines for a data bucket's slot, and the shapes [Slot Hashing](trie.md#slot-hashing)'s unambiguity argument assumes.
These items are positioned as additional flat elements of the enclosing `encode(bucket_range) = rlp([version, start, end, buckets, bytecodes])` list defined in [Bucket Range with Proof](../state-sync.md#bucket-range-with-proof), not as one nested list item; a decoder reads them positionally, the same way it reads every other field of that response.

## Constants

| Constant                        | Value                                                                | Description                                                                                                                                                                                                |
| ------------------------------- | -------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `BUCKET_ID_BITS`                | `24`                                                                 | Width of the `bucket_id` field within a `SaltKey`.                                                                                                                                                         |
| `BUCKET_SLOT_BITS`              | `40`                                                                 | Width of the `slot_id` field within a `SaltKey`.                                                                                                                                                           |
| `TRIE_WIDTH`                    | `256`                                                                | Branching factor of the main trie and of every bucket subtree level.                                                                                                                                       |
| `NUM_BUCKETS`                   | `16,777,216`                                                         | Total number of buckets (`2^BUCKET_ID_BITS`).                                                                                                                                                              |
| `MIN_BUCKET_SIZE`               | `256`                                                                | Initial and minimum capacity of a data bucket.                                                                                                                                                             |
| `MIN_BUCKET_SIZE_BITS`          | `8`                                                                  | `log2(MIN_BUCKET_SIZE)`.                                                                                                                                                                                   |
| `META_BUCKET_SIZE`              | `256`                                                                | Fixed capacity of a metadata bucket (equal to `MIN_BUCKET_SIZE`; a metadata bucket never resizes).                                                                                                         |
| `NUM_META_BUCKETS`              | `65,536`                                                             | `NUM_BUCKETS / MIN_BUCKET_SIZE`; number of buckets reserved for metadata.                                                                                                                                  |
| `NUM_KV_BUCKETS`                | `16,711,680`                                                         | `NUM_BUCKETS - NUM_META_BUCKETS`; number of buckets holding plain-key data.                                                                                                                                |
| `BUCKET_RESIZE_LOAD_FACTOR_PCT` | `80`                                                                 | Occupancy percentage of capacity that triggers a resize.                                                                                                                                                   |
| `BUCKET_RESIZE_MULTIPLIER`      | `2`                                                                  | Factor a bucket's capacity is multiplied by on each resize step.                                                                                                                                           |
| `MAX_SALT_VALUE_BYTES`          | `94`                                                                 | Maximum encoded length of a `SaltValue`.                                                                                                                                                                   |
| `EMPTY_CODE_HASH`               | `0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470` | `keccak256` of the empty byte string; the code hash selecting the short-form account record in [Encoding State](#encoding-state).                                                                          |
| `HASHER_SEEDS`                  | `[0x921321f4, 0x2ccb667e, 0x60d68842, 0x077ada9d]`                   | The four seeds keying `ahash` for both bucket assignment and probe hashing: the low 128 bits of `keccak256("Make Ethereum Great Again")`, as four big-endian 32-bit words, each zero-extended to `uint64`. |

## Rationale

**Why a hash function determines `bucket_id` rather than a structural derivation from the plain key (e.g. a key prefix, as in a radix trie).**
A hash spreads keys uniformly across the fixed bucket space for plain keys an adversary has not specifically targeted (e.g. sequential contract storage slots), which is what keeps bucket occupancy — and therefore proof size and commitment update cost — bounded under ordinary use.
A structural derivation would let an adversary concentrate many keys into one bucket trivially, by choosing addresses or slots with a common prefix; a hash at least requires searching for a preimage, which [Security Considerations](#security-considerations) discusses.

**Why AHash rather than a cryptographic hash function for bucket and probe-sequence assignment.**
Bucket and slot assignment are not a security boundary by themselves — an entry landing in a predictable or adversarially-influenced bucket does not, on its own, corrupt state or forge a proof, since every entry is still authenticated by the [commitment](trie.md) over its bucket's actual contents.
A fast, non-cryptographic hash keeps the cost of every read, write, and displacement-chain step low, which matters because these operations run per transaction; the frozen construction and its fixed seeds (see [Mapping State to Buckets](#mapping-state-to-buckets)) still make the assignment deterministic and reproducible across implementations, which is the only property consensus requires here.

**Why SHI (Strongly History-Independent) rather than a simpler open-addressing scheme.**
An ordinary linear-probing hash table's layout after a sequence of insertions and deletions depends on that sequence, not just on the final key set — two implementations that apply the same net changes through different batching would generally end up with different physical layouts, hence different bucket commitments, hence different state roots, breaking consensus.
The displacement-chain insertion and retrace-on-deletion rules specified in [SHI Hash Table](#shi-hash-table) are exactly the mechanism (Blelloch & Golovin, FOCS 2007) that guarantees a canonical layout as a pure function of the entry set at a given `nonce` and `capacity`, at the cost of an insertion or deletion potentially touching more than one slot.

**Why buckets are meta-bucket-addressed instead of storing capacity and nonce inline with the main trie structure.**
`BucketMeta` is itself state that must be committed to, since a bucket's capacity determines the size of the vector its commitment covers; storing it as ordinary bucket entries — addressed the same way as any other SALT data — lets metadata reuse the identical commitment machinery specified in [SALT Trie](trie.md) rather than requiring a second, structurally different authentication scheme just for metadata.

**Why canonicalization is a required once-per-EVM-block step rather than an incidental property of the SHI algorithm.**
[SHI Hash Table](#shi-hash-table)'s guarantees are per-operation: each individual insertion or deletion, taken alone, converges on the correct SHI layout for the entry set that operation leaves behind.
They are not automatically block-wide, because resizing only grows a bucket, so a block whose mini-blocks transiently peak a bucket's occupancy and then let it drop leaves that bucket at a capacity reflecting the peak, not the block's net effect — canonicalization exists specifically to remove that dependence on intra-block history, restoring the same canonical-layout guarantee at the granularity (the EVM block, not the mini-block) that [SALT Trie](trie.md#salt-deltas)'s merged, block-level state delta is actually committed at.

**Why `buckets_to_canonicalize` must be transmitted explicitly rather than derived by a receiving node from the deltas it already has.**
An individual resize event is not hidden: it rewrites the resized bucket's [`BucketMeta`](#bucket-metadata) entry, and that write is an ordinary part of whichever single mini-block's `salt_deltas` triggered it — a node that inspected every mini-block's delta separately, one at a time, and never discarded that history, could in principle notice each resize as it happened.
That information genuinely disappears once deltas are merged across mini-blocks, which is exactly what reconstructing an EVM block's committed `salt_deltas` requires.
Capacity only ever grows on its own ([Resizing](#resizing)), and only canonicalization ever shrinks it back down; a bucket that grows mid-block and is later shrunk back to its original capacity by canonicalization therefore nets to _no change at all_ in its `BucketMeta` entry once the block's deltas are merged.
The transient growth, and the resulting need to re-derive the bucket's slot assignments from scratch, leaves no trace in that merged view — even though the bucket's entries can legitimately end up in different physical slots than a from-scratch build of the same final key set would produce.
A node reconstructing a block's `salt_deltas` by merging deltas directly — the natural and efficient way to do it — therefore cannot recover the required bucket set from that merged result alone; recovering it without an explicit signal would instead require retaining and separately inspecting every mini-block's own delta, in full, for the entire block, solely to catch this one case.
The sequencer avoids imposing that cost on every consumer by publishing the set directly: it already has it for free, as the set of buckets its own incremental updates resized.

## Security Considerations

**Uncanonicalized buckets diverge state roots.**
A node that fails to canonicalize the exact set of buckets an EVM block's mini-block fragments' `buckets_to_canonicalize` entries together name computes a bucket layout that differs from the sequencer's, hence a different [commitment](trie.md) and a different state root — every bucket's commitment folds into the root unconditionally, since [Main Trie](trie.md#main-trie) is a complete tree with one leaf per bucket.
This is not a proof-forgery or authentication bypass, but it is a liveness failure for the node: [EVM Block Assembly](../state-sync.md#evm-block-assembly)'s state-root check rejects every subsequent block until the node re-derives the correct layout.

**The canonicalization set itself is trusted, not verified.**
[Canonicalization](#canonicalization) obligates the sequencer to publish the exact resized-bucket set, but no validity rule checks the transmitted `buckets_to_canonicalize` union against the resizes the block's deltas actually imply — a consumer could in principle re-derive the set by inspecting every fragment's delta separately for `BucketMeta` writes ([Rationale](#rationale)), but nothing requires any node to do that work.
A sequencer whose named set disagrees with the set it actually canonicalized before committing its own deltas and state root is caught indirectly — consumers following the named set derive a different merged delta and root, and [EVM Block Assembly](../state-sync.md#evm-block-assembly)'s checks reject the block — but a sequencer that consistently skips both, neither canonicalizing a resized bucket nor naming it, commits a path-dependent capacity that no delta consumer flags; only stateless validation exposes it, under the same conditions as the skipped load-factor resize below — the validator's own replay resizes and canonicalizes the bucket independently of the field, and the roots disagree.
Padding the set is likewise unchecked: naming a bucket the block never resized is a state no-op — rebuilding an already-canonical bucket reproduces the identical layout and contributes no delta entries — but it forces every consumer into that bucket's extract-and-re-insert work, a per-block resource-consumption vector bounded only by the fragment sizes the sequencer can publish.

**Grinding attacks against `bucket_id` are bounded by gas cost, not prevented by the hash.**
`HASHER_SEEDS` are fixed public constants — every node must compute the same `bucket_id` for the same plain key, so the seeds cannot be secret or per-node-random the way an ordinary hash table's flood-resistant seed would be.
An adversary can therefore search for plain keys that all hash to one `bucket_id` (for example, grinding `CREATE2` salts, or choosing storage slots) and concentrate state there, lengthening that bucket's probe chains and enlarging witnesses that touch it.
This costs the adversary one ordinary state-write's gas per colliding key it wants stored, and it only degrades the targeted bucket: [Resizing](#resizing) still bounds that bucket's load factor, and every other bucket is unaffected.
It therefore raises the operating cost of reading or writing the targeted bucket without breaking correctness, forging a proof, or letting the attacker reach unrelated buckets.

**A skipped load-factor resize is invisible to every delta consumer, and is exposed only by stateless validation against a complete witness.**
[Execution Witness](witness.md)'s stateless replay path inserts into a bucket without the occupied-slot count [Resizing](#resizing)'s load-factor trigger needs whenever the witness does not cover that bucket in full; [Resizing](#resizing) directs exactly that case to skip the check, so the resize simply does not happen there.
Skipping the trigger cannot place an entry in the wrong slot or accept a value that does not belong — every placement still follows the same probe-sequence rules, and the committed state root still authenticates the bucket's true key-value contents.
What it breaks is the layout guarantee: a bucket whose threshold-crossing resize never executed is committed, at the end of the block, at a capacity the load-factor bound would have rejected had the count been available — and nothing on the delta path corrects it, because [Canonicalization](#canonicalization) rebuilds only buckets that were actually resized during the block, and nodes consuming the block's deltas reproduce the transmitted layout verbatim without re-running any SHI logic.
Stateless validation is the one check that can expose it: a validator re-executes the block from the pre-state with its own [Resizing](#resizing) logic and compares the resulting root against the header, so a resize the sequencer skipped surfaces as a root mismatch — provided the witness covers every slot of that bucket, which [Witness Contents](witness.md#witness-contents) requires a witness builder to arrange precisely so the count is available.
That check is therefore only as strong as the witness builder's independence from the sequencer: a builder that takes the block's effects from the sequencer instead of re-deriving them, or that leaves a resized bucket's remaining slots unwitnessed, lets the validator skip the same check and accept the over-loaded bucket — so a sequencer willing to violate [Resizing](#resizing)'s load-factor MUST can commit, and keep committing, such a bucket wherever validation is absent or compromised, degrading that bucket's probe-chain lengths and the size of every witness that touches it for as long as it persists.
The backstops are structural rather than checked: an insertion into a completely full bucket has no empty slot to take and forces the exhaustion-case resize, whose rebuild reveals the bucket's true occupancy, and any new-entry insertion applied with the occupancy count available triggers the postponed resize at that point.
The damage is confined to the targeted bucket's probe-chain and proof-size degradation: it cannot produce an invalid key-value pair, and the state root every node accepts commits to the bucket's true contents throughout.

## Spec History

No spec-gated changes to the behavior on this page have been identified; verify against [Hardforks and Specs](../../hardfork-spec.md) before treating this as exhaustive.
