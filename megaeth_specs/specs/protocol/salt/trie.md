---
description: Normative definition of the SALT trie — its shape, commitment scheme, state root computation, and delta encoding.
spec: Rex5
---

# SALT Trie

SALT authenticates the [bucket model](buckets.md) with a fixed, shallow tree — the **SALT trie** — built from a homomorphic vector commitment scheme.
Every **trie node** in it — a bucket segment, a bucket subtree node, or a main-trie node — commits to exactly 256 children or slots using the same mechanism.
That uniformity is what lets a state root be recomputed from a changed leaf up to the root in a bounded number of steps, regardless of how much state exists.
This page specifies the shape of that tree, how a trie node in it is addressed, how a slot's contents are hashed and folded into a commitment, how the state root is derived from the root commitment, and the wire encoding and hash of a **SALT delta** — the unit of state change a [mini-block](https://docs.megaeth.com/spec/glossary#mini-block) streams and a block header commits to.

## Motivation

A Merkle-Patricia trie's commitment is a hash chain: changing one leaf requires rehashing every trie node on the path to the root, and because that path's trie nodes are scattered across a sparse, variable-depth structure, each rehash is a random-access read.
A homomorphic vector commitment removes the second half of that cost: because the commitment function is linear in its input vector, the effect of changing one entry on the commitment of a trie node can be computed directly from the old and new entry values, without reading or recomputing the commitments of sibling entries that did not change.
Combined with the fixed, shallow tree shape [Main Trie](#main-trie) and [Bucket Subtrees](#bucket-subtrees) establish below — a small, bounded number of levels regardless of how many [buckets](buckets.md) exist or how large any one of them grows — this keeps state-root recomputation proportional to the number of changed entries and their ancestors, not to the size of the state as a whole.

## Trie Shape

This section defines what the tree looks like, before anything about how its commitments are computed: [Main Trie](#main-trie) is the tree's fixed top structure, and [Bucket Subtrees](#bucket-subtrees) is what a bucket's own leaf expands into once it outgrows a single segment.

### Main Trie

The main trie is a fixed, complete 256-ary tree of `MAIN_TRIE_LEVELS` levels: one root, [`TRIE_WIDTH`](buckets.md#constants) level-1 trie nodes, `TRIE_WIDTH^2` level-2 trie nodes, and `TRIE_WIDTH^3` = [`NUM_BUCKETS`](buckets.md#constants) level-3 leaves, one leaf per [bucket](buckets.md#mapping-state-to-buckets).
Every trie node, at any level, holds exactly one thing: a single commitment value standing in for everything beneath it — its up to 256 children, for the root or an internal trie node, or its one bucket, for a leaf.
The `NUM_BUCKETS` leaves are arranged left to right in ascending `bucket_id` order, so a bucket's own `bucket_id` is also its leaf's position within the whole leaf level — its position among its at most `TRIE_WIDTH` siblings is `bucket_id mod TRIE_WIDTH`, per [Node Commitments](#node-commitments) — and [NodeId](#nodeid), below, uses the level-wide position to address every trie node on the leaf's path to the root.
Each leaf's commitment is its bucket's own commitment: a bucket at `MIN_BUCKET_SIZE` capacity commits directly as that leaf, while a larger bucket's leaf commitment is instead the root commitment of its own [bucket subtree](#bucket-subtrees).
How every trie node — a leaf, an internal trie node, or a bucket subtree node — actually computes its own commitment value is specified in [Node Commitments](#node-commitments), further below; this section establishes only the tree's shape.

### Bucket Subtrees

A bucket whose capacity exceeds [`MIN_BUCKET_SIZE`](buckets.md#constants) is partitioned into `capacity / MIN_BUCKET_SIZE` **segments** of `MIN_BUCKET_SIZE` [slots](buckets.md#bucket-structure) each.
Its commitment is then the root of a private, complete 256-ary subtree built over those segments, up to `MAX_SUBTREE_LEVELS` levels deep — replacing the flat commitment a `MIN_BUCKET_SIZE`-capacity bucket uses directly as its main-trie leaf commitment.
A slot's [`slot_id`](buckets.md#mapping-state-to-buckets) splits into two parts relative to its segment: **segment index** — which of the bucket's segments the slot falls in — is `slot_id div MIN_BUCKET_SIZE`, and the slot's position within that segment is `slot_id mod MIN_BUCKET_SIZE`; a bucket at `MIN_BUCKET_SIZE` capacity has exactly one segment (segment index `0`), so there `slot_id mod MIN_BUCKET_SIZE` is just `slot_id` itself.
Segments are arranged left to right within the subtree in ascending segment-index order — the same convention [NodeId](#nodeid), next, uses to address every trie node in either tree.
The subtree's depth above the segment level is the minimum number of `TRIE_WIDTH`-ary levels needed to cover `capacity / MIN_BUCKET_SIZE` segments.
Because [Resizing](buckets.md#resizing) only ever doubles capacity from `MIN_BUCKET_SIZE`, the segment count is always a power of two.
`MAX_SUBTREE_LEVELS` levels — the segment level plus `MAX_SUBTREE_LEVELS - 1` levels above it — is exactly enough: it covers the largest capacity a `BUCKET_SLOT_BITS`-bit `slot_id` can address, `MIN_BUCKET_SIZE * TRIE_WIDTH^(MAX_SUBTREE_LEVELS - 1) = 2^BUCKET_SLOT_BITS` slots.
The bucket subtree's root commitment — not any segment's or intermediate trie node's commitment — is what replaces the bucket's leaf commitment in the main trie; how a segment's or an intermediate subtree node's own commitment is computed, like every other trie node's, is specified in [Node Commitments](#node-commitments), further below.

## NodeId

With the trie's shape fixed, a **`NodeId`** is a 64-bit value identifying one trie node of either the [main trie](#main-trie) or one bucket's [subtree](#bucket-subtrees), used wherever a specific trie node needs to be addressed individually rather than reached by walking the tree from the root — most notably by [Execution Witness](witness.md)'s [`SaltProof`](witness.md#saltproof), which keys its `parents_commitments` map by `NodeId`.
Its construction differs by tree: a main-trie node's `NodeId` is its breadth-first-search (BFS) number, counting from the main trie's own root; a bucket subtree node's `NodeId` is instead

```text
NodeId = (bucket_id << BUCKET_SLOT_BITS) | local_number
```

where `bucket_id` and `BUCKET_SLOT_BITS` are as defined in [Mapping State to Buckets](buckets.md#mapping-state-to-buckets), and `local_number` is defined below.

A main-trie `NodeId` is assigned by breadth-first search: the root is `0`, and each subsequent level's nodes are numbered left to right, continuing from the previous level's last number — so each level's range picks up exactly where the previous one ends:

| Level      | Node count                   | `NodeId` range                                                                  |
| ---------- | ---------------------------- | ------------------------------------------------------------------------------- |
| `0` (root) | `1`                          | `[0, 1)`                                                                        |
| `1`        | `TRIE_WIDTH`                 | `[1, 1 + TRIE_WIDTH)`                                                           |
| `2`        | `TRIE_WIDTH^2`               | `[1 + TRIE_WIDTH, 1 + TRIE_WIDTH + TRIE_WIDTH^2)`                               |
| `3` (leaf) | `TRIE_WIDTH^3 = NUM_BUCKETS` | `[1 + TRIE_WIDTH + TRIE_WIDTH^2, 1 + TRIE_WIDTH + TRIE_WIDTH^2 + TRIE_WIDTH^3)` |

A bucket's own leaf — its bucket root — sits within level `3`'s range at the position [Main Trie](#main-trie) already assigns it, its own `bucket_id`:

```text
bucket_root_node_id(bucket_id) = (1 + TRIE_WIDTH + TRIE_WIDTH^2) + bucket_id
```

A bucket's leaf-level commitment is addressed this way even when the bucket has a subtree: a bucket root is always a main-trie `NodeId`, never subtree position `0` of its own subtree.

A bucket subtree's `local_number` is _not_ scoped to that one bucket's own subtree size: every bucket, however many segments it actually has, is numbered within the same fixed, maximal frame — a hypothetical complete `MAX_SUBTREE_LEVELS`-level tree, sized for the largest subtree any bucket can ever grow to.
That frame's per-level starting number follows the same cumulative BFS-count pattern as the main-trie table above, extended one level further: `STARTING_NODE_ID = [0, 1, 1 + TRIE_WIDTH, 1 + TRIE_WIDTH + TRIE_WIDTH^2, 1 + TRIE_WIDTH + TRIE_WIDTH^2 + TRIE_WIDTH^3]`.
Its first `MAIN_TRIE_LEVELS` entries are exactly the main-trie table's four range starts; its fifth entry, `16,843,009`, extends the same pattern one level deeper, to the frame's own leaf level.
A bucket's segment at segment index `i` MUST be numbered:

```text
local_number(i) = STARTING_NODE_ID[MAX_SUBTREE_LEVELS - 1] + i
```

— always `16,843,009 + i`, whether the bucket has `2` segments or the maximum `TRIE_WIDTH^(MAX_SUBTREE_LEVELS - 1)`; an internal subtree node's `local_number` above the segment level follows from that base case by the parent climb below.
Climbing one level in either tree's numbering — a subtree `local_number` (with the `bucket_id` prefix carried through unchanged) or a main-trie `NodeId` — MUST map a node's number `n`, at any level other than the root's, to its parent's number as:

```text
parent_number(n) = (n - 1) div TRIE_WIDTH
```

which equals stripping the level's `STARTING_NODE_ID` entry from `n`, taking [`parent_position`](#node-commitments) of the resulting level-relative position, and adding back the parent level's `STARTING_NODE_ID` entry — the three steps collapse to the single division above because every level's starting number is one more than `TRIE_WIDTH` times the previous level's.
Applying `parent_position` to `n` itself, without stripping the level start first, yields the wrong parent whenever `n`'s level-relative position is `TRIE_WIDTH - 1` modulo `TRIE_WIDTH`.
Only the frame levels between the segment level and that particular bucket's own actual root are ever materialized as real subtree nodes; the frame's shallower levels, above the bucket's actual root, never are.
A bucket's own actual root — at whichever frame level its capacity puts it — is itself never addressed by a subtree `local_number`: as already established above, it is always the bucket's main-trie leaf, `bucket_root_node_id`.

A main-trie `NodeId` never reaches `1 + TRIE_WIDTH + TRIE_WIDTH^2 + TRIE_WIDTH^3` (the table above, `16,843,009` for the current constants), and a subtree `NodeId` is never below `NUM_META_BUCKETS << BUCKET_SLOT_BITS` (`2^56`, the smallest `bucket_id` with a subtree, shifted into position) — so the two ranges never overlap, and a `NodeId` MUST be classified as main-trie or subtree from its value alone.
Interpreting a subtree `NodeId`'s `local_number` further still requires knowing that bucket's own subtree depth, given separately by, for example, `SaltProof.levels` in [Execution Witness](witness.md#saltproof).

## Computing Commitments

This section defines how a trie node's own commitment is actually computed, in three layers: [Commitment Scheme](#commitment-scheme) first defines the vector-commitment primitive every trie node uses, [Slot Hashing](#slot-hashing) then defines how a leaf-level slot's contents become one of the scalars that primitive commits to, and [Node Commitments](#node-commitments) ties the two together into the actual commitment of every leaf, segment, and internal trie node in the tree.

### Commitment Scheme

SALT commits to a vector of up to 256 scalar field elements using a [**Pedersen vector commitment**](https://doi.org/10.1007/3-540-46766-1_9) over [Banderwagon](https://hackmd.io/@kevaundray/BJBNcv9fq), the prime-order group built from the [Bandersnatch curve](https://eprint.iacr.org/2021/1152.pdf).
The commitment of a 256-element vector, whose `i`-th scalar-field-element entry is `v_i`, MUST be computed as:

```text
commit(v_0, ..., v_255) = sum_{i=0}^{255} v_i * G_i
```

where `G_0, ..., G_255` are the first 256 points of the **common reference string (CRS)** and `*`/`sum` are scalar multiplication and point addition in the Banderwagon group — so each term `v_i * G_i` multiplies the scalar `v_i` by the group point `G_i`, and the vector itself is nothing more than the ordered list of these 256 scalars.
This formula always receives exactly 256 scalars in SALT's own usage, never fewer: [Slot Hashing](#slot-hashing) fixes an unoccupied leaf or segment slot's scalar to `EMPTY_SLOT_HASH` rather than leaving it out of the vector, and [Node Commitments](#node-commitments) uses a precomputed default commitment for a trie node's missing child rather than shortening the vector, so no caller of this formula ever supplies fewer than 256 entries.

The CRS is 257 Banderwagon points, `G_0, ..., G_255` and one additional point `Q` used only by the [multi-point opening proof](witness.md#multi-point-proof) specified in [Execution Witness](witness.md), not by the commitment formula above.

#### Commitment Encoding

Banderwagon identifies a Bandersnatch point's affine coordinates `(x, y)` with `(-x, -y)` as the same group element — not `(-x, y)`, which is that point's group inverse, a different element.
A Banderwagon point (`Commitment`) MUST be encoded as 32 bytes: whichever of `(x, y)` or `(-x, -y)` has the lexicographically larger `y` — i.e. `y`, if `y` is already the larger of `y` and `-y`, or `-y` otherwise — is the canonical representative, and that representative's `x`-coordinate (`x` or `-x`, respectively) MUST be serialized as a big-endian integer.
A 32-byte encoding MUST be rejected outright, before any curve check, if its big-endian integer value is not strictly less than the Banderwagon base field's order — a non-canonical encoding MUST NOT be reduced into range and accepted, only rejected.
Otherwise, it MUST be rejected on decode unless the resulting `x` yields a valid curve point whose corresponding canonical `y` is on the curve and the resulting point lies in the Banderwagon prime-order subgroup; a point that fails either check MUST NOT be accepted.
This subgroup check is required for every point decoded from data that did not originate from the node's own already-validated storage — in particular, every point carried in a [`SaltProof`](witness.md#saltproof) or reconstructed as part of verifying one.
The prime-order subgroup excludes every point with `y = 0`: the Bandersnatch curve equation forces `y = 0` only for a handful of low-order points outside that subgroup, so the subgroup check above already rules them out — [Node Commitments](#node-commitments)'s `child_scalar` computation, which divides by a commitment's `y`-coordinate, therefore never divides by zero for any point that passed this check.

The CRS MUST be derived deterministically as:

```text
CRS = generate_random_elements(257, CRS_SEED)

generate_random_elements(n, seed):
    points = []
    index = 0
    while len(points) < n:
        digest = SHA256(seed ++ be_bytes(index, 8))
        candidate = reduce_base_field(digest)          # interpret digest as a big-endian integer, reduce modulo the base field order
        point = decode(candidate)                        # per Commitment Encoding, above; None if candidate does not decode to a valid point
        if point is not None:
            points.append(point)
        index += 1
    return points
```

where `be_bytes(x, n)` is `x`'s big-endian encoding as exactly `n` bytes, and `reduce_base_field(digest)` interprets `digest`'s 32 bytes as a **big-endian** integer and reduces it modulo the Banderwagon base field's order, yielding a candidate `x`-coordinate in that field — the opposite byte order from [`reduce`](#slot-hashing)'s little-endian interpretation, since the two reduce into different fields for different purposes and share no encoding convention.
`decode(x)` applies [Commitment Encoding](#commitment-encoding)'s decode checks — canonical `y`, then prime-order-subgroup membership — directly to an already-computed `x`, skipping the byte-parsing step a 32-byte input would otherwise need.
Unlike Commitment Encoding's decode, `decode(x)` here returns no point instead of rejecting when a check fails, since the loop above simply tries the next index.
`CRS_SEED` is the fixed byte string in [Constants](#constants).
This derivation has no trusted-setup toxic waste: any implementation MAY regenerate the CRS from `CRS_SEED` rather than embed it as a constant, and both approaches MUST produce byte-identical points.

### Slot Hashing

Every commitment SALT computes ultimately rests on hashing a bucket [slot](buckets.md#bucket-structure) into a single scalar field element.
A slot's hash MUST be computed as:

```text
kv_hash(slot) = EMPTY_SLOT_HASH,                          if slot is unoccupied
kv_hash(slot) = reduce(blake3(salt_value.key ++ salt_value.value)),  if slot holds salt_value
```

where `salt_value.key` and `salt_value.value` are the plain key and plain value bytes a `SaltValue` encodes (excluding its length-prefix bytes and any unused tail), `blake3` produces a 32-byte digest, `reduce` interprets those 32 bytes as a little-endian integer and reduces it modulo the scalar field order of the [commitment curve](#commitment-scheme), and `EMPTY_SLOT_HASH` is the fixed constant in [Constants](#constants), reduced the same way.
The scheme's soundness rests on `EMPTY_SLOT_HASH` never coinciding with any reachable `blake3` output under this reduction; the constant is fixed precisely so every implementation treats the same value as "empty" without needing to special-case it structurally.
Concatenating `salt_value.key ++ salt_value.value` with no delimiter or length tag is unambiguous only because every [plain key and plain value](buckets.md#encoding-state) pair has a fixed, distinct total length per SALT value type — 60 bytes for a short-form account record (20 + 40), 92 for a long-form account record (20 + 72), 84 for a storage slot (52 + 32), and 12 for a `BucketMeta` record (12 + 0), per [Encoding State](buckets.md#encoding-state) — so no two different (key, value) pairs of different types can concatenate to the same bytes; a future value type without a fixed, distinct total length would need an explicit delimiter to preserve this.
A metadata bucket's slot is never "unoccupied" in the first sense above: per [Bucket Metadata](buckets.md#bucket-metadata), an unpopulated metadata slot's `salt_value` is the implicit default `BucketMeta` (`nonce = 0, capacity = MIN_BUCKET_SIZE`), not an absent value, so `kv_hash` for any metadata bucket slot always takes the second case — `reduce(blake3(...))` of that default record when nothing has been written yet, never `EMPTY_SLOT_HASH`.

### Node Commitments

Every trie node SALT authenticates — a bucket's leaf-level flat commitment (at `MIN_BUCKET_SIZE` capacity), a [bucket subtree](#bucket-subtrees) segment, or an internal trie node of either the main trie or a bucket subtree — commits to a vector of exactly 256 values using the [commitment scheme](#commitment-scheme) above; what differs between them is only what goes into that vector.

A leaf's or a segment's own commitment MUST be computed as the vector commitment of its 256 slots' [`kv_hash`](#slot-hashing) values, `kv_hash(slot_0), ..., kv_hash(slot_255)`, indexed by each slot's position within the segment (see [Bucket Subtrees](#bucket-subtrees)).
An internal trie node's children are not already numbered this way: a child's own **vector position** — the index in `[0, TRIE_WIDTH)` it occupies within that parent's commitment vector — and its parent MUST be derived from its level-relative position `p` (a leaf's `bucket_id`, a bucket subtree segment's segment index, or an already-derived ancestor's position) as:

```text
vector_position(p) = p mod TRIE_WIDTH
parent_position(p)  = p div TRIE_WIDTH
```

repeated once per level climbed, until reaching the main trie's root or, within a bucket subtree, the bucket's own subtree root (see [Bucket Subtrees](#bucket-subtrees)).
Every internal trie node's own commitment MUST be computed as the vector commitment of its 256 children's `child_scalar` values, defined below, each entry placed at that child's `vector_position`.
[Commitment Scheme](#commitment-scheme) defines that vector commitment over 256 _scalar field_ elements, but a child's commitment is a curve _point_, not a scalar — so before it can take its place as one of the 256 entries in its parent's vector, it MUST first be collapsed down to a single scalar field element, `child_scalar`, computed as:

```text
child_scalar(commitment) = reduce_scalar_field(base_field_quotient(commitment.x, commitment.y))
```

where `commitment.x` and `commitment.y` are the child commitment's affine coordinates, and `base_field_quotient(x, y) = x / y`, computed in the curve's base field, collapses the point's two coordinates down to the single base-field element `child_scalar` folds into the parent's vector; `reduce_scalar_field(z)` then serializes that base-field element `z` to 32 little-endian bytes and applies [`reduce`](#slot-hashing) to them — the same little-endian-bytes-to-scalar-field reduction [Slot Hashing](#slot-hashing) defines, just preceded here by a serialization step because its input is a field element rather than already a byte string.
For a trie node whose child does not yet exist (has never been written to), that child's precomputed default commitment MUST be used rather than treating the missing child as an error; these default commitments MAY be precomputed and cached rather than recomputed, since they never change.
A default commitment is defined recursively from the leaf level up, and — because [Slot Hashing](#slot-hashing) gives a metadata slot a different never-written scalar than a data slot — the recursion is rooted differently over the two regions of the bucket space: a never-written data-bucket leaf, or bucket-subtree segment, defaults to `commit(EMPTY_SLOT_HASH, ..., EMPTY_SLOT_HASH)` (256 copies), while a never-written [metadata-bucket](buckets.md#bucket-metadata-storage) leaf defaults to the commitment of 256 copies of the implicit default `BucketMeta` record's `kv_hash` ([Slot Hashing](#slot-hashing)); an internal trie node's default is then the vector commitment of its 256 children's own defaults' `child_scalar` values.
Because the metadata buckets occupy exactly the leaves covered by the main trie's first level-1 node (`NUM_META_BUCKETS = TRIE_WIDTH^2` leaves), every default below the root is pure — derived from one region's leaf default alone — and only the empty root's default mixes the two, combining that one metadata-region child with 255 data-region children.
Every default at every level of either recursion is a specific, fixed, nonzero point, never the Banderwagon group's identity element: both leaf defaults commit to 256 copies of a nonzero scalar ([Slot Hashing](#slot-hashing)), and a commitment to any nonzero vector can equal the identity only by exhibiting a nontrivial discrete-log relation among the hash-derived CRS points — precluded under the same discrete-log assumption the [commitment scheme](#commitment-scheme)'s binding already rests on.

## State Root

The **state root** MUST be computed as:

```text
state_root = child_scalar(main_trie_root_commitment)
```

The state root is serialized as a 32-byte little-endian scalar — the same canonical scalar encoding [Execution Witness](witness.md#transcript) uses wherever a scalar field element is transmitted.
`child_scalar` here is the same computation [Node Commitments](#node-commitments) uses to fold a child's commitment into its parent's input vector, applied once more to the main trie's own root commitment.
Because `child_scalar` already yields a scalar field element, this final step is only serialization, with no further reduction.
Any node that holds the complete, current contents of every bucket in `[0, NUM_BUCKETS)` (including the [metadata buckets](buckets.md#bucket-metadata-storage)) can compute this value directly from [Main Trie](#main-trie) and [Bucket Subtrees](#bucket-subtrees) above, without needing any prior commitment state: the result is a pure function of the bucket contents, independent of the order buckets are read in or the order updates that produced those contents were originally applied in.
This is the check [State Snapshot Bootstrap](../state-sync.md#state-snapshot-bootstrap) requires a node to perform once it holds a complete downloaded snapshot, comparing the result against the signed header's `state_root` field.

Incremental recomputation of the state root from a small set of changed buckets — updating only the commitments on the path from each changed bucket to the root, using the commitment scheme's linearity rather than recomputing every trie node from scratch — produces the identical value and is the mechanism [SALT Deltas](#salt-deltas) below exists to make efficient, but is a performance technique, not a distinct normative computation; the state root MAY be computed either way.

## SALT Deltas

A **SALT delta** (`salt_deltas` in the [State-Change Fragment](../state-sync.md#state-change-fragment), and the value whose hash [Block Metadata](../state-sync.md#block-metadata)'s `salt_deltas_hash` commits to) is the net set of `SaltKey` changes some scope of execution applies, in the four-part shape below.
Two distinct scopes use this shape: an individual mini-block's own `salt_deltas`, as its state-change fragment carries it, and an EVM block's **merged** `salt_deltas` — the net effect, per `SaltKey` and per `codes` entry, of applying every one of its mini-blocks' deltas in ascending mini-block order, plus the additional entries [Canonicalization](buckets.md#canonicalization) contributes once, after the block's last mini-block; this is a genuine merge, not a concatenation — a key a mini-block's delta puts and a later mini-block's delta deletes, for example, nets to no entry at all in the block-level delta, per the net-effect rule below.
[Canonicalization](buckets.md#canonicalization) runs once per EVM block, not once per mini-block, so only the block-level merged delta is ever canonicalized; an individual mini-block's own `salt_deltas` MUST NOT be assumed already canonicalized, and `block_metadata.salt_deltas_hash` commits to the block-level merged and canonicalized form only, never to an individual mini-block's own delta.
Both scopes share the same four disjoint parts, each keyed by `SaltKey` in ascending order:

| Part      | Keyed by              | Contents                                                                                                               |
| --------- | --------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `puts`    | `SaltKey`             | Entries newly occupied by this delta: the inserted `SaltValue`.                                                        |
| `deletes` | `SaltKey`             | Entries newly vacated by this delta: the `SaltValue` that occupied the [slot](buckets.md#bucket-structure) beforehand. |
| `updates` | `SaltKey`             | Entries occupied at both ends of the scope whose occupant changed: a **value delta**, below.                           |
| `codes`   | `bytes32` (code hash) | Every code hash newly assigned to an account touched by this delta's scope, excluding the empty-code hash.             |

A slot that a delta's scope both fills and empties again, or that ends that scope holding the exact `SaltValue` it started with, contributes no entry to any part; a slot occupied at both the start and the end of the scope by different contents contributes a single `updates` entry, even if it was transiently emptied — or emptied and refilled by a different plain key's entry — inside the scope: a SALT delta records only the net effect on the final slot occupant over its own scope, never any intermediate write.
An account's code hash counts as "newly assigned" whenever it differs from what the account held before this delta's scope — contract creation, an [EIP-7702](https://eips.ethereum.org/EIPS/eip-7702) delegation-designator update, and a system-contract code install or upgrade are all instances of this, uniformly; a balance- or nonce-only update to an account whose code hash is unchanged contributes nothing to `codes`, and a code hash that resolves to the empty-code hash is excluded even when it does change.
A `BucketMeta` entry's first-ever write MUST be classified as `updates`, never `puts`: per [Slot Hashing](#slot-hashing), an unwritten metadata slot already holds the implicit default `BucketMeta`, so writing its real value for the first time changes an already-occupied slot's contents rather than occupying an empty one.

An `updates` entry's **value delta** MUST be computed as the byte-wise XOR of the slot's old and new `SaltValue` encodings — both always exactly [`MAX_SALT_VALUE_BYTES`](buckets.md#constants) long before this step — with any leading zero bytes of the XOR result trimmed:

```text
value_delta(old, new) = trim_leading_zeros(old_bytes XOR new_bytes)
```

where `old_bytes` and `new_bytes` are each the slot's full `MAX_SALT_VALUE_BYTES`-byte `SaltValue` encoding.
A node applying an `updates` entry to its own copy of `old` recovers `new` by re-XORing: `new_bytes = old_bytes XOR pad_leading_zeros(value_delta, MAX_SALT_VALUE_BYTES)`.
A `value_delta` longer than `MAX_SALT_VALUE_BYTES` bytes denotes no valid value delta: `pad_leading_zeros` is undefined for a longer input, since no XOR against a `MAX_SALT_VALUE_BYTES`-byte `SaltValue` can ever produce one — [Encoding](#encoding) below states how this bound is, and is not, enforced.

### Encoding

A SALT delta MUST be encoded as a bare concatenation of RLP items, matching the general rule in [Payload Encodings](../state-sync.md#payload-encodings) that `encode(salt_deltas)` is not a single RLP item:

```text
encode(salt_deltas) =
    rlp(len(puts))    ++ (rlp(key) ++ rlp(value))*
    ++ rlp(len(deletes)) ++ (rlp(key) ++ rlp(value))*
    ++ rlp(len(updates)) ++ (rlp(key) ++ rlp(delta))*
    ++ rlp(len(codes))   ++ (rlp(code_hash))*
```

where, within each repeated group, entries appear in ascending `SaltKey` order (ascending code-hash order for `codes`); `key` denotes the entry's `SaltKey`, its underlying `uint64` value, so `rlp(key)` is a standard RLP integer; `value` denotes the entry's full `MAX_SALT_VALUE_BYTES`-byte `SaltValue` encoding, so `rlp(value)` is a single fixed-length RLP string (unlike the [trimmed encoding](buckets.md#bucket-range-encoding) a bucket-range response uses); `delta` denotes the entry's `value_delta` byte string, but `rlp(delta)` is _not_ a bare RLP string — it is a one-item RLP list whose single item is itself an RLP _list of the delta's bytes_, each byte encoded as an RLP integer (`0x00` as `0x80`, `0x01` through `0x7f` as themselves, `0x80` through `0xff` as `0x81` followed by the byte), matching the reference implementation's derived encoding of its delta type; and `code_hash` denotes a 32-byte hash, so `rlp(code_hash)` is a fixed 32-byte RLP string.
The 4-byte `value_delta` `0x0100aa7f` therefore encodes as the 7 bytes `0xc6c5018081aa7f` — an outer one-item list around an inner list of the four integer-encoded bytes — not as the 6 bytes `0xc5840100aa7f` that wrapping the delta as one RLP string would give.
An encoder MUST NOT produce a `puts` or `deletes` entry whose `value` declares `2 + key_len + value_len` greater than `MAX_SALT_VALUE_BYTES`, nor an `updates` entry whose `delta` exceeds `MAX_SALT_VALUE_BYTES` bytes; such an entry denotes no valid `SaltValue` or value delta at all.
These bounds bind the encoder only: no consumer-side rule re-checks them, the reference implementation applies delta entries without validating them, and what a node computes from a violating entry is unspecified — [Security Considerations](#security-considerations) describes the resulting exposure.
These bounds need not be enforced at decode time, unlike [Bucket Range Encoding](buckets.md#bucket-range-encoding)'s `compact_value`: there the declared lengths determine how many bytes the decoder consumes, so an oversized declaration must be rejected at decode to keep the stream parseable, whereas here `rlp(value)` is a fixed-`MAX_SALT_VALUE_BYTES`-byte string and `rlp(delta)` a self-delimiting list, so a violating entry cannot desynchronize decoding.

`salt_deltas_hash`, [Block Metadata](../state-sync.md#block-metadata)'s commitment to the block's merged state deltas, MUST be computed as:

```text
salt_deltas_hash = hash(encode(salt_deltas))
```

using the same BLAKE3 hash function [Payload Authentication](../state-sync.md#content-hash-and-signature) uses for payload content hashes, over the block's fully merged and [canonicalized](buckets.md#canonicalization) `salt_deltas`, exactly as [EVM Block Assembly](../state-sync.md#evm-block-assembly) requires when checking a block's merged deltas against `block_metadata.salt_deltas_hash`.

## Constants

| Constant             | Value                                       | Description                                                                                                                                                           |
| -------------------- | ------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `MAIN_TRIE_LEVELS`   | `4`                                         | Number of levels in the main trie, root through bucket-leaf.                                                                                                          |
| `MAX_SUBTREE_LEVELS` | `5`                                         | Maximum levels of a bucket subtree, including its segment level.                                                                                                      |
| `STARTING_NODE_ID`   | `[0, 1, 257, 65,793, 16,843,009]`           | Per-level starting BFS number of the maximal subtree frame ([NodeId](#nodeid)); its first `MAIN_TRIE_LEVELS` entries are also the main trie's per-level range starts. |
| `EMPTY_SLOT_HASH`    | 32 bytes, each byte `0x01`                  | Fixed slot-hash value for an unoccupied slot, before reduction into the scalar field.                                                                                 |
| `CRS_SEED`           | `"MAKE_ETHEREUM_GREAT_AGAIN"` (ASCII bytes) | Seed for deterministic CRS derivation.                                                                                                                                |
| CRS size             | `257` points                                | `G_0, ..., G_255` (commitment basis) plus `Q` (used only by the multi-point proof).                                                                                   |

## Rationale

**Why a homomorphic vector commitment instead of a standard Merkle hash tree.**
Merkle hashing has no algebraic structure: computing a trie node's new hash after one child changes requires the full hash function over all 256 children again, unlike the linear commitment [Motivation](#motivation) describes, whose incremental-update property — used in [State Root](#state-root)'s incremental recomputation — is exactly what a hash chain cannot offer.

**Why the CRS is derived from a public seed rather than a ceremony.**
Some vector commitment schemes (e.g. KZG) require a structured reference string with discarded secret ("toxic") randomness, where anyone who retained that randomness could forge openings.
IPA-based commitments over Banderwagon require no such secret: [Commitment Scheme](#commitment-scheme)'s `generate_random_elements` derivation is a plain hash-to-curve procedure with no private input, so a fixed public seed is exactly as trustworthy as any other choice of seed, and any implementation can regenerate the identical CRS from the seed alone without trusting a distributed artifact.

**Why `codes` is part of a SALT delta rather than left for a consumer to derive from `puts`/`updates`.**
A consumer holding the pre-state could in principle recover the same set by decoding every touched account record and comparing each account's code hash across the delta's scope — `puts` carry account records outright, and an `updates` entry's XOR delta exposes exactly which record bytes changed.
Carrying `codes` explicitly spares every consumer that decode-and-compare pass over all account-shaped entries, and gives the [backfill](../state-sync.md#backfill) validity rules a direct, self-contained value to check bytecode completeness against, matching [State-Change Fragment](../state-sync.md#state-change-fragment)'s `new_bytecodes` payload one-to-one.

**Why `updates` carries an XOR delta instead of the new value outright.**
An update typically changes only a few bytes of a `SaltValue` (a balance or a single storage word), and leading-zero-trimmed XOR against the old value is usually far shorter than the new value's full fixed-length encoding — `puts` and `deletes` still carry full values because, for those, no prior or resulting value exists on the other side of the change to XOR against economically.

## Security Considerations

**A node that computes a different `child_scalar`, CRS, encoding, or trie node addressing than the reference derivation cannot agree on any state root.**
Every value in this page's [NodeId](#nodeid), [Commitment Scheme](#commitment-scheme), [Node Commitments](#node-commitments), and [State Root](#state-root) sections is a fixed function with no configuration point; an implementation that derives the CRS from a different seed, uses a different curve encoding, or places a child at a different `vector_position` than [Node Commitments](#node-commitments) defines computes commitments that are internally consistent but never equal to any other conforming implementation's, and will reject every block via [EVM Block Assembly](../state-sync.md#evm-block-assembly)'s state-root check.

**Omitting the subgroup or canonical-encoding check on decode fails open, not closed.**
Unlike the self-consistent divergences above, an implementation that skips [Commitment Encoding](#commitment-encoding)'s decode checks on untrusted runtime input computes byte-identical results to a conforming one on every honest input — omitting a validation check there changes nothing the node itself computes — so the omission is invisible in normal operation and surfaces only under attack, as acceptance where a conforming node rejects.
The same is not true of the checks' other call site: [Commitment Scheme](#commitment-scheme)'s CRS derivation filters candidate points through the identical `decode`, so an implementation that drops the subgroup check there admits candidates a conforming one skips, derives a different `G_1` onward, and lands in the self-consistent-divergence case above — rejecting every block — rather than failing open.
The subgroup check is not a redundant well-formedness check: without it, an attacker can supply a point outside the group the commitment scheme's security relies on and construct a value that satisfies a state-root or witness check the reference scheme's algebra was never proven to guarantee, breaking the commitment's binding property.
This check MUST be applied to every point decoded from untrusted input, per [Commitment Encoding](#commitment-encoding).

**An out-of-bounds delta entry is applied, not rejected.**
[SALT Deltas](#salt-deltas)' length bounds bind the encoder only: no decode- or apply-time rule re-derives them, and the reference implementation applies `puts`, `deletes`, and `updates` entries without validating their declared lengths, so a sequencer that signs a violating entry induces unspecified behavior in every consumer instead of a clean rejection.
The guard is the sequencer's signature and its own encoder conformance, not a validity rule; an implementation MAY additionally validate the bounds on receipt, at the cost of diverging from the reference — halting where the reference computes an unspecified result — on input no conforming sequencer produces.

## Spec History

No spec-gated changes to the behavior on this page have been identified; verify against [Hardforks and Specs](../../hardfork-spec.md) before treating this as exhaustive.
