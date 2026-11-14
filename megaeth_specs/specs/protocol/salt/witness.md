---
description: Normative definition of the SALT execution witness format for stateless verification.
spec: Rex5
---

# Execution Witness

A SALT witness lets a verifier that holds only a state root — not the state itself — check the pre-state values a block execution reads and prove the absence of keys it does not touch, cryptographically bound to that root by a single aggregated opening proof.
This page specifies what a witness contains, how membership and non-membership are each proved, and the exact multi-point proof and Fiat-Shamir transcript a verifier reproduces to check one.

How a witness's contents are used to re-execute a block and how the resulting post-state root is checked against a block header is [Stateless Validation](../stateless-validation.md)'s scope, not this page's: this page ends at "the witness is internally consistent and its claims are bound to `pre_state_root`," and Stateless Validation begins at "re-executing the block against those claims produces the committed post-state."

## Motivation

A [stateless validator](../stateless-validation.md) re-executes a block without holding the chain's state, so it needs, for every key the block's execution reads, either that key's value or a proof that the key does not exist — and it needs both bound to the specific state root the block claims to build on, so that a witness constructed against a different (possibly attacker-chosen) state cannot be substituted.
Because [SALT's bucket layout](buckets.md#shi-hash-table) is a hash table rather than a sorted structure, proving a key's absence is not simply "show me the leaf" the way it is in a sorted trie: absence has to be demonstrated by exhibiting enough of the bucket's probe sequence to rule the key out, per the ordering property [Insertion](buckets.md#insertion) maintains.

## Witness Contents

A **`SaltWitness`** has two fields:

| Field   | Type                                     | Description                                                               |
| ------- | ---------------------------------------- | ------------------------------------------------------------------------- |
| `kvs`   | map of `SaltKey` to optional `SaltValue` | Every slot the witness makes a claim about, in ascending `SaltKey` order. |
| `proof` | [`SaltProof`](#saltproof)                | The cryptographic proof binding `kvs`'s claims to a state root.           |

Each `kvs` entry is one of two kinds of claim:

- **`Some(value)`**: the slot is occupied by `value` — a **membership** claim.
  A bucket's own [`BucketMeta`](buckets.md#bucket-metadata) slot is a special case of this kind: per [Bucket Metadata](buckets.md#bucket-metadata), an unwritten metadata slot's value is the implicit default `BucketMeta`, never an absent value, so a `BucketMeta` entry in `kvs` MUST always be `Some`, never `None`.
- **`None`**: the slot is proven empty — a **non-membership** claim, one entry of the [probe sequence](buckets.md#shi-hash-table) that establishes the target key's absence, not necessarily the slot the target key's own probe sequence starts at.

A witness builder MUST include a data bucket's own `BucketMeta` entry in `kvs` whenever any claim about that bucket depends on its `nonce` or `capacity` — most notably [a non-membership claim](#membership-and-non-membership-proofs), whose probe sequence cannot be recomputed without them — and a verifier MUST derive `nonce`/`capacity` from that entry, never from any source outside the witness.
The **witness builder** is whichever component re-executes the block against its complete pre-state to produce the witness — a role distinct from the sequencer's: its only inputs from the sequencer are the block and the pre-state it builds on, both obtained through [State Sync](../state-sync.md), and everything the witness contains is derived from the builder's own re-execution, which is what [Bucket Model](buckets.md#security-considerations)'s security argument relies on.
A witness builder MUST also include every slot — occupied and empty — of any bucket that applying the block's insertions to the pre-state resizes, so that a verifier's occupied-slot count for that bucket is available ([Resizing](buckets.md#resizing)) and it reaches the same resize decision the builder's replay did; a verifier whose count for such a bucket is unavailable skips the resize and computes a different post-state root.
A `SaltKey` absent from `kvs` entirely carries no claim at all: a verifier MUST NOT treat an absent key as either present or absent, and MUST reject any attempt by a consumer of the witness to read a key `kvs` does not cover.
This three-state semantics — witnessed-present, witnessed-absent, and unwitnessed-and-therefore-inaccessible — is what makes a witness safe to execute against: an implementation that treats an unwitnessed key as implicitly absent would let a witness builder hide state from the verifier without the verifier detecting it.

## SaltProof

A **`SaltProof`** has three fields.

| Field                 | Type                                                                             | Description                                                                                                                                                                                                                                                                                        |
| --------------------- | -------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `parents_commitments` | map of [`NodeId`](trie.md#nodeid) to [`Commitment`](trie.md#commitment-encoding) | The union, over every witnessed slot, of the [commitments](trie.md#node-commitments) on that slot's [authentication path](#authentication-path), keyed by `NodeId` so a trie node shared by more than one witnessed slot's path (for example, the main trie root) appears once, not once per slot. |
| `proof`               | [`MultiPointProof`](#multi-point-proof)                                          | A single aggregated opening proof covering every witnessed slot's [`kv_hash`](trie.md#slot-hashing) against its bucket's commitment path.                                                                                                                                                          |
| `levels`              | map of [`BucketId`](buckets.md#mapping-state-to-buckets) to `uint8`              | For every bucket with at least one witnessed slot, one more than [Bucket Subtrees](trie.md#bucket-subtrees)'s subtree depth above the segment level for that bucket's capacity — `1` if the bucket has no subtree — needed to interpret its trie nodes' `NodeId`s.                                 |

`parents_commitments` uses one unified structure for both main-trie nodes and bucket-subtree nodes — the same `NodeId` space, the same commitment encoding, the same vector-commitment mechanics defined in [SALT Trie](trie.md#node-commitments) — so a verifier does not need separate logic for a witnessed key whose path stays entirely within the main trie versus one that descends into the subtree of [a bucket whose capacity exceeds `MIN_BUCKET_SIZE`](trie.md#bucket-subtrees).
`levels[b]` is never `0`: a bucket at `MIN_BUCKET_SIZE` capacity — including every metadata bucket, which never resizes — has no subtree at all, and `1` is the value that case MUST use, since a subtree depth of `0` levels above the segment level is still one level (the segment level itself).
Nor does `levels[b]` ever exceed [`MAX_SUBTREE_LEVELS`](trie.md#constants), the frame's own level count; a verifier MUST reject a `SaltProof` whose `levels` carries any value outside `[1, MAX_SUBTREE_LEVELS]` before reconstructing any `NodeId` from it, since [Reconstructing a Witnessed Slot's NodeIds](#reconstructing-a-witnessed-slots-nodeids)' climb is undefined outside that range.

### Authentication Path

A witnessed [slot](buckets.md#bucket-structure)'s **authentication path** is the ordered chain of [trie nodes](trie.md#node-commitments) from that slot's own segment commitment up to the main trie root — the parent-of relationship [Node Commitments](trie.md#node-commitments) defines, followed from leaf to root rather than root to leaf.
`parents_commitments[0]`, the main trie root's commitment, MUST be present in every `SaltProof`, since every authentication path terminates there.
A `SaltWitness` whose `kvs` is empty is invalid: it carries no claim for a proof to bind, and [Verifying a Witness](#verifying-a-witness)'s state-root check has no authentication path to run against — a verifier MUST reject an empty-`kvs` witness rather than treat it as vacuously valid.

### Reconstructing a Witnessed Slot's NodeIds

For a witnessed slot in bucket `b` with `slot_id`, the exact chain of `NodeId`s from that slot's segment up to the main trie root MUST be reconstructed as follows.

- If `levels[b] = 1` (bucket `b` has no subtree), the slot's leaf commitment is addressed directly at `bucket_root_node_id(bucket_id)` ([NodeId](trie.md#nodeid)): skip straight to the main-trie climb below.
- Otherwise, the slot's segment `NodeId` is `(b << BUCKET_SLOT_BITS) | (STARTING_NODE_ID[MAX_SUBTREE_LEVELS - 1] + (slot_id div MIN_BUCKET_SIZE))` — [NodeId](trie.md#nodeid)'s subtree construction, with a `local_number` from the fixed frame every bucket's segments use regardless of that bucket's own actual depth — and its `local_number` MUST be climbed to its parent (`parent_number` of [NodeId](trie.md#nodeid), applied to the `local_number` with the `bucket_id` prefix kept) exactly `levels[b] - 2` times, reaching the highest subtree-addressed node on the slot's path: the direct child of the bucket's own actual subtree root, which is itself never subtree-addressed ([NodeId](trie.md#nodeid)).
- From there, the climb continues not with a further `local_number`, but from the bucket's own main-trie leaf, `bucket_root_node_id(bucket_id)` ([NodeId](trie.md#nodeid)) — the actual subtree root's main-trie address — up through the remaining `MAIN_TRIE_LEVELS - 1` main-trie levels to the root, via `parent_number` exactly as above; this main-trie climb is also where the `levels[b] = 1` case rejoins.

Each resulting `NodeId`'s commitment MUST be looked up in `parents_commitments`.

### Query Construction

A **query** is a single claim about an already-established vector commitment — `commitment` commits to a value `result` at position `point` — the same `(i, v_i)` pair [Node Commitments](trie.md#node-commitments)'s vector-commitment formula takes as input, now being opened rather than committed.
[Multi-Point Proof](#multi-point-proof) below authenticates a whole set of queries with one aggregated proof rather than one opening proof per query, built from every witnessed slot's `NodeId` chain above.
Two disjoint collections MUST be built, each a map of `NodeId` to a set of `point` values queried at that `NodeId`'s commitment:

- **`internal_nodes`**: one entry per parent-child edge on every witnessed slot's climb from its segment to the main trie root — the edges [Reconstructing a Witnessed Slot's NodeIds](#reconstructing-a-witnessed-slots-nodeids) walks, not the slot's own segment or leaf.
  For each edge, the parent's `NodeId` (see below for the one exception) MUST gain the child's `vector_position` ([Node Commitments](trie.md#node-commitments)) as a `point`.
  For a bucket with a subtree (`levels[b] >= 2`), the edge from the highest subtree-addressed node on its climb into the bucket's own actual subtree root, `bucket_root_node_id(bucket_id)` — the one edge that crosses from subtree addressing into main-trie addressing — is keyed not by the plain `bucket_root_node_id`, but by:

  ```text
  encode_parent(parent, level) = parent | (level << ENCODED_PARENT_LEVEL_SHIFT)
  ```

  where `parent = bucket_root_node_id(bucket_id)`, `level = levels[b]`, and `ENCODED_PARENT_LEVEL_SHIFT` is in [Constants](#constants); this composite key sorts every such crossing edge strictly between the (always-small) plain main-trie-internal `NodeId`s and the (always-large, `bucket_id`-shifted) subtree-internal `NodeId`s, and among themselves by `levels[b]` first, so the ordering below is well-defined without inventing a second `NodeId` space.
  A no-subtree bucket (`levels[b] = 1`) contributes no crossing edge at all: its witnessed slots' only `internal_nodes` edges are the plain main-trie ones climbing from `bucket_root_node_id(bucket_id)`, and the slots themselves appear only in `leaf_nodes` below.

- **`leaf_nodes`**: one entry per witnessed slot, keyed by the `NodeId` [Reconstructing a Witnessed Slot's NodeIds](#reconstructing-a-witnessed-slots-nodeids) computes for that slot's segment (or leaf), gaining `slot_id mod MIN_BUCKET_SIZE` as a `point`.

Two witnessed slots whose climbs share an edge contribute that edge's `(NodeId, point)` pair only once: both collections' `point` sets MUST de-duplicate by ordinary set membership.
The de-duplication has effect only in `internal_nodes` — distinct witnessed slots in one segment still contribute distinct `slot_id mod MIN_BUCKET_SIZE` points to that segment's `leaf_nodes` entry — so the total query count is the number of distinct climb edges plus the number of distinct witnessed slots.

The final query list MUST be assembled in the fixed order `internal_nodes` before `leaf_nodes`, each map's entries in ascending `NodeId` order (using the encoded key above where it applies) and, within one `NodeId`, its `point`s in ascending order.
For an `internal_nodes` entry, `commitment` is the parent's own commitment, `point` is the queried child index (as a scalar), and `result` is that child's `child_scalar` ([Node Commitments](trie.md#node-commitments)).
For a `leaf_nodes` entry, `commitment` is the segment's (or, for a no-subtree bucket, the bucket's own leaf's) commitment, `point` is the queried slot index (as a scalar), and `result` is `kv_hash(slot)` ([Slot Hashing](trie.md#slot-hashing)).

## Multi-Point Proof

`SaltProof.proof` is a single **`MultiPointProof`** authenticating every witnessed slot's claimed `kv_hash` against its segment's commitment, aggregated into one opening rather than one proof per slot.
It MUST be constructed and verified using the [IPA](https://eprint.iacr.org/2017/1066.pdf)-based [multi-point protocol](https://dankradfeist.de/ethereum/2021/06/18/pcs-multiproofs.html) below, over the [commitment scheme](trie.md#commitment-scheme)'s CRS and Banderwagon group, with `DOMAIN_SIZE = 256` (see [Constants](#constants)) — the same `256` as [`TRIE_WIDTH`](buckets.md#constants), since a `MultiPointProof` opens the same 256-scalar vectors [Node Commitments](trie.md#node-commitments) commits to; `DOMAIN_SIZE` names that quantity specifically in its role as the IPA protocol's evaluation-domain size.

### Transcript

Every step of proof generation and verification interacts with a single **Fiat-Shamir transcript**: a byte buffer, initialized to a domain-separator label, into which points and scalars are appended (each prefixed by its own short label, with no length prefix or delimiter) before a challenge is drawn — drawing a challenge digests and then resets the buffer, per `challenge_scalar` below.
The transcript's five operations MUST be implemented exactly:

```text
new(label):                    state = label
domain_sep(label):             state = state ++ label                   # no length prefix, same as the other operations' label prefix
append_point(label, P):        state = state ++ label ++ compress(P)     # 32-byte compressed point, per Commitment Encoding
append_scalar(label, s):       state = state ++ label ++ le32(s)         # 32-byte little-endian scalar
challenge_scalar(label):
    state = state ++ label
    digest = SHA256(state)
    state = []                                                            # buffer cleared before folding the challenge back in
    c = reduce(digest)                                                    # per SALT Trie's Slot Hashing: little-endian bytes, reduced mod the scalar field order
    append_scalar(label, c)
    return c
```

where `compress` is the 32-byte Banderwagon point encoding from [Commitment Encoding](trie.md#commitment-encoding), `le32` is the scalar's canonical 32-byte little-endian encoding, and `reduce` is the same byte-string-to-scalar reduction [Slot Hashing](trie.md#slot-hashing) defines.
Every label below is the exact ASCII byte string; these literal labels MUST be used, since the transcript's output depends on them exactly as written.

### Proof Generation and Verification

The evaluation domain is fixed as the first `DOMAIN_SIZE` scalar field elements: domain point `j` is the scalar `j`, for `j` in `[0, DOMAIN_SIZE)` — the same integers [Query Construction](#query-construction)'s `point`s already are — and every evaluation form, Lagrange basis polynomial, and barycentric weight below is taken over exactly these points, not over roots of unity or any other domain.
Given the **queries** [Query Construction](#query-construction) builds, in the order specified there — each a `(commitment, point, result)` triple — the following MUST be run, where `f_i` is query `i`'s underlying commitment's full 256-value vector in evaluation form over that domain (the list `f_i(0), ..., f_i(255)`), and `(z_i, y_i)` is that query's `(point, result)`:

```text
transcript = new(b"st")
transcript.domain_sep(b"multiproof")
for (commitment, point, result) in queries, in order:
    transcript.append_point(b"C", commitment)
    transcript.append_scalar(b"z", point)
    transcript.append_scalar(b"y", result)
r = transcript.challenge_scalar(b"r")

# g(X) = sum_i r^i * (f_i(X) - y_i) / (X - z_i): the genuine aggregated quotient,
# each division computed directly in evaluation form via the domain's precomputed
# barycentric weights; commit to it as g_x_comm
transcript.append_point(b"D", g_x_comm)
t = transcript.challenge_scalar(b"t")

# g1(X) = sum_i (r^i / (t - z_i)) * f_i(X): a fresh linear combination of the
# queries' own f_i, weighted by t-dependent scalars -- NOT g(X)'s quotient, despite
# the superficial resemblance. Commit to it as g1_comm
transcript.append_point(b"E", g1_comm)
g3_comm = g1_comm - g_x_comm

# IPA opening of g3 = g1 - g at point t, claiming inner_product_result = g3(t)
transcript.domain_sep(b"ipa")
transcript.append_point(b"C", g3_comm)
transcript.append_scalar(b"input point", t)
transcript.append_scalar(b"output point", inner_product_result)
w = transcript.challenge_scalar(b"w")
q = w * Q                                               # the CRS's extra point, scaled once by w
a = g3's 256 evaluation-form values                    # the vector being folded
b = [L_0(t), ..., L_255(t)]                             # Lagrange coefficients at t
G = [G_0, ..., G_255]                                   # the CRS basis points
for k in 0..log2(DOMAIN_SIZE):                          # 8 rounds, DOMAIN_SIZE = 256
    a_L, a_R = a[:len(a)/2], a[len(a)/2:]               # split each in half
    b_L, b_R = b[:len(b)/2], b[len(b)/2:]
    G_L, G_R = G[:len(G)/2], G[len(G)/2:]
    L_k = <a_R, G_L> + <a_R, b_L> * q
    R_k = <a_L, G_R> + <a_L, b_R> * q
    transcript.append_point(b"L", L_k)
    transcript.append_point(b"R", R_k)
    x_k = transcript.challenge_scalar(b"x")
    a = a_L + x_k * a_R                                 # new a, half the length
    b = b_L + x_k^-1 * b_R                               # new b, half the length
    G = G_L + x_k^-1 * G_R                               # new G, half the length
# after 8 rounds, a, b, and G each have length 1
```

where `<u, v>` for equal-length vectors is the inner product `sum_j u_j * v_j`, `<u, H>` for a vector `u` of scalars and `H` of points is the multi-scalar multiplication `sum_j u_j * H_j`, `L_i(t)` is the `i`-th Lagrange basis polynomial (over the `DOMAIN_SIZE`-point domain fixed above) evaluated at `t`, and `Q` is the CRS's extra point ([Commitment Scheme](trie.md#commitment-scheme)) — every round's `L_k`/`R_k` folds in the `w`-scaled `q = w * Q`, while the final check below uses the unscaled `Q` with its `w` factor written out explicitly, the same binding expressed from each side.
A prover MUST run this to derive `g_x_comm` — from its own knowledge of every query's full `f_i`, since `g(X)` is a genuine quotient no party lacking every `f_i` can construct — `g1_comm`, and each `(L_k, R_k)`, ending with `a`'s single remaining scalar.
A verifier MUST run the identical sequence of transcript operations, but computes `g1_comm` (and therefore `g3_comm`) directly from the queries and the two challenges `r`, `t` — `g1_comm = sum_i (r^i / (t - z_i)) * commitment_i`, valid because commitment is linear ([Commitment Scheme](trie.md#commitment-scheme)) — and `inner_product_result = sum_i (r^i / (t - z_i)) * y_i` from the queries' own claimed results, rather than reading either from the proof; from the proof's encoding it reads only `g_x_comm`, each round's `L_k`/`R_k`, and the final folded scalar `a`, deriving the same `r`, `t`, `w`, and each `x_k` a conforming prover would.
Once the transcript is exhausted, letting `x_k^-1` be each round's inverse challenge and, for `i` in `[0, DOMAIN_SIZE)`, `s_i = -1 * product of x_k^-1 over every round k whose bit is set in i`'s binary representation (most-significant round first), a verifier MUST accept if and only if:

```text
(sum_k x_k * L_k) + (sum_k x_k^-1 * R_k) + g3_comm + w * (inner_product_result + a * b_0) * Q + a * (sum_{i=0}^{255} s_i * G_i) = O
```

where `O` is the Banderwagon group identity, `a` is the proof's final folded scalar, and `b_0 = sum_{i=0}^{255} L_i(t) * s_i` — the same Lagrange-coefficient vector `b`'s initial value above, evaluated against `s`: a single scalar, not that vector's first element.
The proof MUST be rejected if this final check fails, or if the transcript's domain-separator labels, append order, or challenge order deviates from the sequence above at any step — because the transcript is deterministic, any deviation between prover and verifier produces different challenges and, except with negligible probability, a proof that fails to verify rather than one that verifies incorrectly.

`MultiPointProof` MUST be encoded as:

```text
encode(multi_point_proof) = compress(g_x_comm) ++ compress(L_0) ++ ... ++ compress(L_7) ++ compress(R_0) ++ ... ++ compress(R_7) ++ le32(a)
```

32 bytes each, for a total of `32 * (2 * log2(DOMAIN_SIZE) + 2) = 576` bytes, fixed regardless of how many slots the proof's queries cover: the transcript-folding cost is `log2(DOMAIN_SIZE)` rounds independent of query count, so aggregating more witnessed slots into one `SaltProof` grows only `parents_commitments` and `kvs`, not the multi-point proof itself.
The encoding's internal layout is not self-describing: nothing in the bytes marks where the `L_k` points end and the `R_k` points begin, so a decoder MUST know `DOMAIN_SIZE` out of band to partition them; how the 576 bytes themselves are framed when a witness travels on the wire is [Wire Encoding](#wire-encoding)'s scope, below.

## Wire Encoding

Unlike every other wire format on this page and in [State Sync](../state-sync.md), a `SaltWitness` and its `SaltProof` MUST be encoded as [bincode](https://github.com/bincode-org/bincode) (fixed-integer-width, little-endian configuration) rather than RLP: a witness moves only between the witness builder that produces it and the [stateless validators](../stateless-validation.md) that check it — over whatever channel connects them, which this specification does not fix — never across the heterogeneous peer set a block, delta, or bucket range reaches, so this page documents the reference implementation's actual bincode contract rather than a from-scratch RLP scheme no implementation produces.

- Every variable-length collection — `kvs`, `parents_commitments`, `levels`, and the `proof` byte vector below — MUST be prefixed by its entry count (byte count, for the byte vector) as an 8-byte little-endian integer; a fixed-width field carries no prefix of its own — a `SaltKey` or `NodeId` as 8 little-endian bytes, a `levels` entry's `BucketId` key as 4 little-endian bytes and its `uint8` value as 1 byte, a compressed `Commitment` as 32 bytes, and a `SaltValue` as its `MAX_SALT_VALUE_BYTES`-byte form.
- `kvs` and `parents_commitments` MUST each be encoded in ascending-key order — `SaltKey` and `NodeId` respectively — which for both fields is a property of the ordered map type carrying them, not a rule imposed at encode time.
- `levels` MUST be encoded with its entries sorted into ascending `BucketId` order at encode time, since its underlying map type carries no natural order on its own; a decoder MUST reject a duplicate `BucketId` entry.
- A `kvs` entry's `Some`/`None` `SaltValue` MUST be distinguished by bincode's own `Option` encoding: a one-byte discriminant (`0` for `None`, `1` for `Some`) followed by the payload if present — the payload being the full padded `MAX_SALT_VALUE_BYTES`-byte form ([Encoding State](buckets.md#encoding-state)).
  A decoder MUST reject a `Some` payload whose declared `2 + key_len + value_len` exceeds `MAX_SALT_VALUE_BYTES`, before the entry is hashed or otherwise used: [Slot Hashing](trie.md#slot-hashing)'s `salt_value.key ++ salt_value.value` is undefined for such an entry, so it can carry no checkable claim.
- Each `Commitment` in `parents_commitments` MUST be encoded compressed (32 bytes, per [Commitment Encoding](trie.md#commitment-encoding)), and MUST be subgroup-checked on decode exactly as every other Banderwagon point on this page is.
- The `proof` field MUST carry [Multi-Point Proof](#multi-point-proof)'s fixed 576-byte encoding as a length-prefixed byte vector per the first rule above — an 8-byte little-endian byte count, always `576` at the current `DOMAIN_SIZE`, then the bytes — not written inline the way a fixed-width `Commitment` is: the fixed length is a property of `DOMAIN_SIZE`, not of the framing.

Unlike `levels`, neither `kvs` nor `parents_commitments` is rejected for carrying a duplicate key: a decoder MUST resolve a duplicate to its last-appearing value, so the decoded result is deterministic — this affects wire compactness, not verification correctness, since two conforming decoders resolve the same duplicate the same way.

## Membership and Non-Membership Proofs

A membership claim for `plain_key` is a `kvs` entry `salt_key -> Some(salt_value)` where `salt_key` is wherever the SHI algorithm actually placed `plain_key` — not computable from `plain_key` alone, since [Mapping State to Buckets](buckets.md#mapping-state-to-buckets)'s `slot_id` depends on the bucket's contents at insertion time — together with `parents_commitments` covering the path from that slot's bucket leaf (or subtree segment) to the root.
A verifier MUST resolve a membership claim's slot from the witness itself: each non-metadata `Some` entry embeds its own plain key ([Encoding State](buckets.md#encoding-state)), those embedded keys form the witness's `plain_key -> salt_key` mapping, and reading `plain_key` against the witness means looking it up in exactly that mapping — never walking [Lookup](buckets.md#lookup)'s probe sequence, whose earlier slots a membership claim does not cover.
A `plain_key` that mapping does not cover falls through to the non-membership walk below, which — per [Witness Contents](#witness-contents)' three-state rule — either proves absence from covered slots or fails on the first slot `kvs` does not cover, so a present-but-unwitnessed key is inaccessible rather than misread as absent.
A verifier accepts a membership claim once [Multi-Point Proof](#multi-point-proof) verification succeeds for that slot's claimed `kv_hash` — [Slot Hashing](trie.md#slot-hashing)'s `reduce(blake3(salt_value.key ++ salt_value.value))` — against the committed path.

A non-membership claim for `plain_key` is the [probe sequence](buckets.md#shi-hash-table) `probe(plain_key, 0), probe(plain_key, 1), ...`, each visited slot included in `kvs` (`Some` if occupied, `None` if empty), continuing until either an empty slot or a slot whose stored key is lexicographically less than `plain_key` is reached — the stopping rules [Lookup](buckets.md#lookup) uses, minus the equal-key stop, which cannot fire for a key that is genuinely absent — or, in a completely full bucket none of whose occupants' keys is less than `plain_key`, until all `capacity` slots of the probe sequence have been included, [Lookup](buckets.md#lookup)'s exhaustion case.
Every slot on this probe sequence up to and including the stopping slot MUST be included when building a witness: a verifier concludes non-membership only by walking this same probe sequence against `kvs` itself, in order, and — per [Witness Contents](#witness-contents)'s three-state rule — that walk MUST fail the moment it reaches a slot `kvs` does not cover, rather than concluding absence from an incomplete prefix.

## Verifying a Witness

A verifier checking a `SaltWitness` against an expected `pre_state_root` MUST first derive the state root `parents_commitments[0]` claims — `child_scalar(parents_commitments[0])` ([Node Commitments](trie.md#node-commitments), applied exactly as [State Root](trie.md#state-root) applies it to the main trie's root commitment) — and MUST reject the witness if that value does not equal `pre_state_root`.
This is the check that makes `kvs`'s claims actually bound to a specific, externally-trusted state, per this page's own scope; the multi-point check alone only proves internal consistency between `parents_commitments` and the claimed `(kvs, levels)` — nothing about it ties `parents_commitments[0]` to any particular root a verifier did not already supply.

Beyond the multi-point check and the root check above, `parents_commitments` and `levels` together MUST also be checked, when verifying a `SaltProof` against a set of claimed `kvs`, to contain exactly the trie node set the claimed `kvs` requires — every trie node on every witnessed slot's authentication path, and no additional trie node — and the proof MUST be rejected if any required trie node's own commitment is missing or if either map contains an entry no witnessed slot's path requires.
This closes a gap the multi-point check alone does not: a proof that is cryptographically valid for the trie nodes it does include says nothing about a trie node it omits, so completeness must be checked structurally, against the shape of the claimed `kvs`, not cryptographically.

## Constants

| Constant                     | Value       | Description                                                                                                                              |
| ---------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `DOMAIN_SIZE`                | `256`       | Vector length of every commitment, main trie or bucket subtree, in its role as the IPA protocol's evaluation-domain size.                |
| `ENCODED_PARENT_LEVEL_SHIFT` | `33`        | Bit offset at which `encode_parent` ([Query Construction](#query-construction)) embeds `levels[b]` into a crossing edge's composite key. |
| Multi-point proof length     | `576` bytes | `32 * (2 * log2(DOMAIN_SIZE) + 2)`, fixed regardless of witnessed slot count.                                                            |

## Rationale

**Why bincode instead of RLP for [`SaltWitness`/`SaltProof`'s wire encoding](#wire-encoding), unlike every other format on this page.**
Every other SALT wire format exists to let heterogeneous, independently-developed peers exchange consensus-relevant data, which is what justifies RLP's implementation-neutral discipline.
A witness has a narrower job: it moves only from its builder to the stateless validators that check it, between parties that today share an implementation, so bincode's Rust-struct-derived layout costs nothing extra — the tradeoff is that this page's wire contract is only as portable as that shared implementation, not independently reproducible from a struct-free description the way RLP formats are.

**Why one aggregated multi-point proof instead of one opening proof per witnessed slot.**
An opening proof's cost is dominated by the `log2(DOMAIN_SIZE)`-round folding argument; the multi-point protocol pays that cost once for the whole batch of queries rather than once per query, which is exactly what makes [Multi-Point Proof](#multi-point-proof)'s proof size independent of witnessed slot count.

**Why non-membership requires a probe-sequence prefix rather than a single "this slot is empty" claim.**
[Insertion](buckets.md#insertion)'s SHI algorithm only guarantees that a stored entry is preceded, along its own probe sequence, by entries with strictly greater keys — not that a key's own ideal slot is where lookup would find it: a key can be displaced arbitrarily far down its probe sequence by prior insertions.
A single empty-slot claim at the key's ideal position is not always available, and is not always the _first_ stopping condition reached: per [Lookup](buckets.md#lookup), the walk stops at the first empty slot _or_ the first occupied slot holding a smaller key, whichever comes first, and any occupied slots holding greater keys before that point must still be exhibited so a verifier can confirm the walk did not stop early — the full prefix, not just its final slot, is what a verifier can check without re-deriving the bucket's entire contents.

**Why `SaltProof` addresses main-trie and bucket-subtree nodes through one `NodeId` space instead of two separate schemes.**
A witnessed slot's authentication path routinely crosses from a bucket subtree into the main trie (or never enters a subtree at all, for a `MIN_BUCKET_SIZE` bucket); giving both kinds of trie node the same commitment representation and the same map key type lets [Multi-Point Proof](#multi-point-proof) treat every trie node on every path uniformly, rather than requiring a verifier to special-case where a subtree ends and the main trie begins.

## Security Considerations

**An unwitnessed key must be inaccessible, not silently treated as absent.**
[Witness Contents](#witness-contents)'s three-state semantics is a security boundary, not just a convenience, for exactly the reason given there: silently treating an unwitnessed key as absent would let a witness builder hide state from the verifier undetected.
An implementation MUST error on access to an unwitnessed key rather than substitute a default.

**Transcript label or ordering deviations fail closed.**
Because the [transcript](#transcript) is a deterministic function of the exact append sequence, any implementation bug that appends the wrong label, the wrong point encoding, or operations in the wrong order produces a transcript that diverges from a conforming implementation's at the first deviation, which cascades into different challenges and, overwhelmingly, a proof that fails the final verification check rather than one that passes incorrectly.
This does not excuse imprecision when implementing [Multi-Point Proof](#multi-point-proof): a subtly wrong implementation still fails to interoperate with a conforming one, causing every proof it produces to be rejected and every proof it tries to verify from a conforming peer to be rejected too, a liveness failure discovered only in production if the implementation was not checked byte-for-byte against this page during development.

**Proof-shape completeness must be checked independent of the cryptographic check.**
[Verifying a Witness](#verifying-a-witness)'s structural check exists for exactly the reason given there: a cryptographically valid multi-point proof makes no claim whatsoever about a trie node it omits, so completeness cannot be established cryptographically and must be checked structurally instead.

**Point decompression must reject non-subgroup values, per [Commitment Encoding](trie.md#commitment-encoding).**
Every commitment in `parents_commitments` and every point in a `MultiPointProof` arrives from untrusted input; the subgroup check that page requires applies here without exception, since accepting an invalid point anywhere in a proof's commitment path undermines the entire proof's binding guarantee, not just the single trie node it appears at.

## Spec History

No spec-gated changes to the behavior on this page have been identified; verify against [Hardforks and Specs](../../hardfork-spec.md) before treating this as exhaustive.
