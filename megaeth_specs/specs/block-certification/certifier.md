---
description: The block certifier — the attested pure function that checks consecutive channel frames against selected records and binds their submission payload to a data availability commitment.
spec: TBD
---

# Block Certifier

{% hint style="warning" %}
This page specifies unstable behavior.
Block certification is not activated on any MegaETH network.
Parameters marked `TBD` remain unassigned.
Test vectors for the cursor encoding, the attestation digest and the frame chain will be added when the constants are assigned.
{% endhint %}

The **block certifier** gates settlement on certification: each call receives one or more consecutive new [channel frames](https://specs.optimism.io/protocol/derivation.html#frame-format) from one channel, together with any previously accepted prefix, and attests that the complete L2 blocks they carry match the [selected record sequence](block-records.md#canonical-record-sequence) continuing from a fixed position.
One data availability commitment binds all the new frames in that call, which are posted through one batch transaction.
Its verification logic is a pure function of supplied data, with no I/O or verification state between calls, and can serve as the statement proved by a future proof system.
Replacing the current TEE attestation would additionally require a specified proof format, binding of public inputs, and L1 verification and authorization path; this specification does not define that replacement.
This page specifies the Espresso verification the certifier relies on, the [`Cursor`](#cursor) that carries its position across calls, the certifier core's inputs, outputs, and verification procedure, and what its attestation proves.
On this page, the _block certifier_ is the whole component.
The _certifier core_ (or _the core_) is the attested pure function it runs inside an enclave.
The _certifier host_ (or _the host_) is the untrusted process around it that assembles the core's inputs, disperses to EigenDA and returns the commitment the batcher submits to L1.
The host is run by an _operator_ trusted for nothing, unless it can change the release policy of a sealed signing key, which [Attestation](#attestation) permits and qualifies.

## Motivation

Certification fixes what the content of each L2 height is, but it does not by itself constrain what settles.
A sequencer that publishes one transaction list to Espresso and batches a different one to L1 produces a chain whose settled content contradicts the records selected from Espresso's finalized history.
Nothing in the settlement path notices: [derivation](../rollup/derivation.md) reads batches from L1 and has no knowledge of Espresso, so it will derive whatever it was given.
Certification would then be an advisory feed rather than a guarantee — observers could detect the contradiction after the fact, but the contradicting content would already be the chain.

Closing this requires a check that stands between the batcher and L1, and the check cannot be performed by the party being checked.
It also cannot be performed by the L1 contract directly: establishing that a frame's bytes match what Espresso finalized requires verifying Espresso consensus signatures over the blocks spanning that content, which is far beyond what is affordable to re-execute on Ethereum for every batch.
The certifier resolves this by moving the verification off-chain into an environment whose execution can be attested, and leaving L1 with only a signature to check.

## Espresso Verification

The certifier authenticates the Espresso data it is given rather than trusting whoever supplied it.
This section defines finality verification, quorum transitions, and namespace completeness, followed by their common trust anchor.
These operations are defined by [Espresso](https://docs.espressosys.com/network/learn/what-is-espresso), not by this specification; what follows fixes the properties this design depends on, and the mechanisms behind them are Espresso's to specify.

A **quorum** is the validator set governing one Espresso epoch, together with each validator's stake weight and the threshold that speaks for the network, the supermajority of their total stake; see Espresso's [glossary](https://docs.espressosys.com/network/glossary/glossary) for the formal definitions.

```text
verify_finalized(header, proof, quorum) -> bool
```

The core MUST return true if and only if `proof`, anchored in the trusted `quorum`, establishes that `header`'s block is finalized under the commit rule for the Espresso protocol version `v` that `header` declares.
The version thresholds are Espresso's [`EPOCH_VERSION`](#constants) and [`NEW_PROTOCOL_VERSION`](#constants), with versions compared by major and then minor number:

- **`v` ≥ `NEW_PROTOCOL_VERSION`.** `proof` MUST demonstrate one `Certificate2` signed by `quorum` whose leaf commitment, view number and block number equal those of `header`'s block.
- **`EPOCH_VERSION` ≤ `v` < `NEW_PROTOCOL_VERSION`.** `proof` MUST demonstrate two quorum certificates of consecutive views: a committing certificate signed by `quorum` over `header`'s block in that block's view, and a deciding certificate in the next view.
  The second certificate is what establishes that the network moved past the block rather than abandoning it; a block that never receives one was never committed.
  Every certificate in the chain MUST itself be of a protocol version below `NEW_PROTOCOL_VERSION`, so a chain spanning that version's activation fails, because certificates past it do not imply finality under this rule.
  When `header`'s block is the last of its epoch, the deciding certificate is signed by the next epoch's quorum, which the proof MUST authenticate from `quorum` as the rules below require.
- **`v` < `EPOCH_VERSION`.** The core MUST return false; certification never covers Espresso blocks that predate epochs.

This is the whole meaning of a block being authentic and committed, and the certifier core checks nothing else about it.
These commit rules are Espresso's, as its light client implements them in [`LeafProof::verify`](https://github.com/EspressoSystems/espresso-network/blob/d9df43a3f8a78d97883278c8a663e916598f7529/light-client/src/consensus/leaf.rs#L112-L210); this specification fixes only which rule applies to each declared version.
Verification MUST also apply [`ESPRESSO_FINALITY_PROFILE`](#constants)'s certificate-binding and epoch-transition rules, including authentication of every additional quorum needed by the proof.
`proof` MUST supply all evidence those checks require; missing or invalid evidence MUST fail verification.
The supplied evidence MUST NOT introduce a new trust anchor.
The rest of the profile remains unassigned; this helper does not select an Espresso network.

```text
resolve_quorum(quorum, quorum_steps) -> quorum
```

Returns the quorum for the furthest epoch reachable from an already-trusted `quorum` by following `quorum_steps`.
Each step MUST authenticate the quorum governing the next epoch, including its validator public keys, stake weights and ordering, relative to the quorum trusted before that step and under [`ESPRESSO_FINALITY_PROFILE`](#constants)'s epoch-transition rules.
The evidence format and verification mechanism are defined by that profile.
The core MUST reject missing or invalid transition evidence and MUST NOT substitute a stale or fallback quorum or accept a new trust anchor from the supplied evidence.
Espresso's validator set evolves as validators join, exit, and restake, changing at most once per epoch; this operation is how trust crosses those boundaries.

```text
verify_namespace(payload, proof, header) -> bool
```

Returns true if and only if `payload` is exactly this namespace's content in `header`'s block, or `payload` is empty and `proof` demonstrates that the namespace is genuinely absent from it.
Distinguishing genuine absence from mere omission is essential rather than incidental: [record selection](block-records.md#canonicality) chooses the first valid record for a height, so a host that could omit part of a block's namespace payload undetected could suppress the record that should have won and promote a later one in its place.

The chain of trust these operations build terminates at the authentic quorum snapshot whose digest is [`INITIAL_QUORUM_HASH`](activation.md#constants), chosen by the admin under [Activation Schedule](activation.md#activation-schedule).
Every quorum the certifier uses MUST be reachable through valid transitions under `resolve_quorum` from the quorum matching the scheduled `INITIAL_QUORUM_HASH`.
Each call MUST match its supplied quorum to the cursor's anchor and verify the required forward transitions, as the [verification procedure](#verification-procedure) specifies.
The [initial cursor](activation.md#initial-cursor) supplies the scheduled trust anchor, and normal attested cursor transitions preserve that provenance.
After an [emergency override](batch-inbox.md#emergency-cursor-override), the anchor's provenance instead depends on the admin satisfying the override's correctness obligations; the call does not re-prove its reachability from the scheduled trusted quorum.
Matching a supplied quorum's digest to the stored anchor does not itself establish that provenance.

## Cursor

Integer expressions and fixed-width fields on this page follow [Integer Semantics](block-records.md#integer-semantics).

A **`Cursor`** records a scan resume point and the current channel.
A cursor returned by a local certifier call is a proposed transition; only inbox acceptance commits that transition, while an override sets a resume point without making skipped records canonical.
It combines a [record position](block-records.md#record-position) in Espresso's finalized history with the state of the channel currently being built and the quorum anchored at that position:

```text
Cursor {
    height:               uint64,   // Espresso block containing the record's span start
    index:                uint32,   // span-start transaction index within that block's namespace payload
    block_number:         uint64,   // that record's L2 height
    epoch_num:            uint64,   // that record's L1 origin
    frames_digest:        bytes32,  // the frames of the currently open channel
    quorum_snapshot_hash: bytes32,  // digest of the quorum anchored at `height`
}
```

`height` and `index` form the inclusive [record position](block-records.md#record-position) at which the next scan starts: the span start of the last consumed record, which is its record-batch transaction or the index-0 fragment of its completing attempt.
`block_number` is that record's L2 height, so the next expected height is `block_number + 1`.
The cursor MUST use the same tuple and encoding for both publication types; it MUST NOT require a record offset, transaction-type field or persisted fragment nonce.
The core MUST start every scan under [Sync Start](block-records.md#sync-start), for every supplied cursor including bootstrap and override cursors.
The scan starts at the cursor's position, and [Sync Start](block-records.md#sync-start) decides from the authenticated publication there alone: when that transaction is an index-0 record fragment for `block_number`, the scan re-selects that consumed record through the ordinary pipeline without emitting it, before any record for the next height can be selected; otherwise it starts at `block_number + 1` and re-selects nothing.
The rule is the same for bootstrap, accepted and override cursors, and needs no activation parameters.
The core MUST rebuild all fragment-attempt and nonce state from authenticated history rather than from host caches or enclave-local state.

`epoch_num` is that record's L1 origin, copied from the record rather than computed; it is not the origin independently chosen by derivation for that height.
Before ordinary accepted transitions, [bootstrap](activation.md#initial-cursor) or an [override](batch-inbox.md#emergency-cursor-override) supplies the preceding-origin anchor under its own rules.
It is the OP Stack [L1 origin](https://specs.optimism.io/glossary.html#l1-origin) number, unrelated to Espresso's epochs, the validator-set periods that `quorum_snapshot_hash` refers to.
The [exclusive window](batch-inbox.md#exclusive-window) uses this anchor to determine when alternatives may submit, measured in L1 blocks from `cursor.epoch_num`.
During ordinary accepted transitions it tracks the origins of records passed by the inbox, not whether their ordinary transactions successfully entered the derived chain.

`frames_digest` is a hash chain over whatever prefix of the currently open channel's frames has already been attested.
OP Stack batches L2 blocks into [channels](https://specs.optimism.io/protocol/derivation.html#channel-format) and splits each channel into one or more frames.
The certifier permits submitting a channel in consecutive groups of frames or submitting the complete channel in one call.
Chaining the frames is what prevents a host from presenting one frame to the certifier while posting a different one to L1.

`quorum_snapshot_hash` is the digest of a quorum governing the epoch that `height` falls into or an earlier one — never of some later quorum a call happened to resolve — carried so that calls can reuse a trusted anchor rather than re-deriving the chain from the scheduled trusted quorum.
The anchor's provenance follows the bootstrap, attested-transition and override premises under [Espresso Verification](#espresso-verification).
Every cursor the core emits anchors it at `height`'s own epoch exactly.
The [initial cursor](activation.md#initial-cursor) anchors at the quorum matching `INITIAL_QUORUM_HASH`, which may govern an earlier epoch.
The digest is the quorum digest fixed under [Encodings](#encodings), and the [verification procedure](#verification-procedure) both checks the supplied quorum against it on entry and re-derives it for the resulting cursor's own `height`.

For attested transitions, the first four fields MUST advance only when the submission closes its channel.
A channel may carry several L2 blocks, so a closing submission may advance `block_number` by more than one; a non-closing submission MUST leave all four unchanged and update `frames_digest` according to the [verification procedure](#verification-procedure).
The quorum hash MUST follow that procedure independently of whether the submission closes the channel.
This coupling prevents partial-channel activity from advancing the record position or origin anchor; only [inbox acceptance](batch-inbox.md#acceptance) commits a proposed transition.
Closing a channel need not advance those fields when no new record is matched, and accepted advancement does not establish successful derivation of the records' ordinary content.

## Certifier Core

The **certifier core** is the attested pure function.
This section defines the [encodings](#encodings) of the hashes the core computes, [what it receives and emits](#inputs-and-outputs), the [verification it MUST perform](#verification-procedure), and then the [cursor advance](#advancing-the-cursor) that verification depends on.

### Encodings

This section defines the `Cursor` encoding for the [batch inbox's stored cursor](batch-inbox.md#contract-state), the submission payload, the frame hash chain carried inside that cursor, and the attestation digest used to verify `certifier_sig` on L1.
Each MUST be computed exactly as specified here.
A core that agreed with every check on this page but disagreed on one of these byte layouts would produce attestations that no other party's cursor can chain from and that the L1 verifier cannot recover a registered key from.
That would silently reduce the certifier to whichever implementation was deployed first, contradicting the property [Attestation](#attestation) establishes, that the role is one any party can occupy.

A `Cursor`'s canonical encoding MUST be:

```text
enc(c) = abi.encode((height, index, block_number, epoch_num, frames_digest, quorum_snapshot_hash))
```

the standard Solidity ABI encoding of the tuple `(uint64, uint32, uint64, uint64, bytes32, bytes32)` with the fields in the order [`Cursor`](#cursor) declares them: six 32-byte words, each integer left-padded with zero bytes to 32 bytes and each `bytes32` field as its own 32 bytes, 192 bytes in all.
`enc` is `abi.encode`, never `abi.encodePacked`, and it is the same tuple shape the [batch commitment](batch-inbox.md#batch-commitment) carries in calldata and the [cursor-advance event](batch-inbox.md#events) emits, so one schema serves the contract, the events, the attestation digest, and every implementation that reads or writes a cursor.
The two digests the core takes over its own values MUST be:

```text
digest(c) = keccak256(enc(c))
digest(q) = keccak256(DOMAIN_QUORUM || serialize(q))
```

where `||` is byte concatenation, `digest(c)` applies to a `Cursor` wherever one is hashed rather than carried in full, and `digest(q)` applies to a quorum, which is the form `quorum_snapshot_hash` holds and the anchor check compares against.
[`DOMAIN_QUORUM`](#constants) is a fixed domain-separation string.
`serialize(q)` is the big-endian concatenation of the quorum's epoch number (`u64`), the number of validators (`u64`), and, for each validator in Espresso's stake-table order, the length of its BLS public key (`u64`), the key bytes, and its stake (`U256`, 32 bytes).
Each BLS public key MUST use the 64-byte compressed BN254 G2 encoding defined by [Espresso's pinned public-key serialization](https://github.com/EspressoSystems/espresso-network/blob/d2df74deec38c17b9e47fe9bab7cf0964b9a12a2/crates/hotshot/types/src/signature_key.rs#L95-L104), with the Jellyfish and Arkworks versions in that revision's [dependency lockfile](https://github.com/EspressoSystems/espresso-network/blob/d2df74deec38c17b9e47fe9bab7cf0964b9a12a2/Cargo.lock).
The key bytes MUST contain only this binary encoding, retaining its internal byte order without a tag or text wrapper.
The core MUST reject key bytes unless they decode to a curve point in the correct subgroup and re-encode byte-for-byte identically under this encoding.
Espresso's stake-table order is the order the quorum certificate's signer bitmap indexes.
The quorum's threshold is not serialized; it is the supermajority of the total stake and is recomputed from these entries.
This layout is MegaETH's canonical encoding for the quorum commitment stored in its cursor.
Espresso defines the validator-set and proof semantics under [Espresso Verification](#espresso-verification); this encoding does not change Espresso's network-message format or consensus-verification rules.

**Quorum commitment test vector.**
For epoch `1` and a single validator with stake `1` whose public key is the standard BN254 G2 generator (secret scalar `1`), the encoding and digest MUST match the following values.
This is an encoding test vector, not a deployment validator set.

```text
public_key = 0xedf692d95cbdde46ddda5ef7d422436779445c5e66006a42761e1f12efde0018c212f3aeb785e49712e7a9353349aaf1255dfb31b7bf60723a480d9293938e19
serialize(q) = 0x000000000000000100000000000000010000000000000040edf692d95cbdde46ddda5ef7d422436779445c5e66006a42761e1f12efde0018c212f3aeb785e49712e7a9353349aaf1255dfb31b7bf60723a480d9293938e190000000000000000000000000000000000000000000000000000000000000001
digest(q) = 0xad9f242d039be54157baa42a689ce58270b55229c36eb4253d1ac2c221158204
```

A **submission payload** MUST consist of the derivation version byte followed by the encodings of one or more consecutive new frames of the same channel, in frame-number order, with no additional version bytes between frames.
The version and frame encoding MUST follow [`CHANNEL_PARAMS`](#constants), and the complete payload MUST fit the payload limits of [`EIGENDA_CODEC`](#constants).
The payload MUST NOT include frames already represented by the input cursor's `frames_digest`; those frames are supplied separately as a prefix witness under [Inputs and Outputs](#inputs-and-outputs).
A submission MAY contain a single frame or every frame of a complete channel, subject to those limits.
The payload dispersed to EigenDA MUST be the entire submission payload verbatim, including its single version byte, so that the certificate's blob commitment, which `eigenda_commitment` hashes, commits to exactly those bytes.

The frame chain MUST be:

```text
rolling_hash([])          = bytes32(0)
rolling_hash(f_0..f_n)    = keccak256(rolling_hash(f_0..f_{n-1}) || keccak256(f_n))
```

where `f_i` is the derivation version byte followed by the exact encoding of the `i`-th frame, and the empty chain is the 32 zero bytes, which is also the value a closing submission resets `frames_digest` to.
Each frame hash includes that version byte, even when the submission payload carries it only once for several frames.
The frame chain therefore depends on the ordered frames, independently of how they are grouped into submissions; the data availability commitment instead binds the exact payload for one submission.
A different chaining rule yields a different `frames_digest` and prevents implementations from continuing each other's channels.
Each extension hashes one frame and one 32-byte accumulator, keeping `frames_digest` at 32 bytes regardless of the number of frames.

The attestation digest MUST be:

```text
registry_evidence_hash = keccak256(DOMAIN_REGISTRY || abi.encode(registry_evidence))
attest_digest = keccak256(DOMAIN_ATTESTATION || CHAIN_ID || batch_inbox_address
                          || enc(prev_cursor) || eigenda_commitment
                          || keccak256(eigenda_cert) || registry_evidence_hash
                          || enc(cursor))
certifier_sig = ECDSA_sign(CERTIFIER_KEY.sk, attest_digest)
```

where [`DOMAIN_ATTESTATION`](#constants) is a fixed domain-separation string, [`CHAIN_ID`](block-records.md#constants) is MegaETH's chain identifier encoded as a big-endian `uint64`, and `batch_inbox_address` is the L1 address of the [batch inbox contract](batch-inbox.md) this attestation is addressed to, taken from the chain's rollup configuration.
This address MUST be the [fixed batch inbox proxy](batch-inbox.md#proxy-address-and-upgrade), not its implementation address.
The address MUST contribute exactly its 20 address bytes to the concatenation, without ABI padding or a length prefix.
`certifier_sig` MUST be a 65-byte ECDSA signature over secp256k1 on `attest_digest`, encoded as `r || s || v` with `v` in {27, 28} and `s` at most half the curve order, the same form and bound [`publisher_sig`](block-records.md#encoding) uses.
The [acceptance check](batch-inbox.md#acceptance) that recovers it MUST reconstruct the digest by this same rule and MUST reject any other form.
The chain identifier and the inbox address MUST both be covered: without them, deployments admitting the same signing key and reconstructing the same remaining signed fields would compute the same digest, as the [replay-isolation discussion](#security-considerations) explains.
`keccak256(eigenda_cert)` and `registry_evidence_hash` MUST both be covered as well.
The first names the one certificate the core verified.
The second names the EigenDA operator-set data — the [registry evidence](#inputs-and-outputs) — it verified that certificate's signature against, which is what lets the [inbox](batch-inbox.md#acceptance) rebuild that data from EigenDA's registries and compare hashes instead of re-verifying the signature.
[`DOMAIN_REGISTRY`](#constants) is a fixed domain-separation string, and `abi.encode(registry_evidence)` is the standard ABI encoding of the `RegistryEvidence` structure as [Inputs and Outputs](#inputs-and-outputs) declares it, so that the core and the contract hash identical bytes.
The encoding MUST treat the complete structure as one tuple argument, including the leading 32-byte offset word whose value is `0x20` because the tuple is dynamic.
It MUST NOT omit that word or encode the structure's fields as separate top-level arguments.
All subsequent ABI offsets MUST use the bases specified by the standard ABI encoding, including those for the dynamic quorum array, its dynamic tuple elements and the non-signer arrays.

#### Digest Encoding Vectors

These vectors use the explicitly supplied sample domains, not assigned values of `DOMAIN_REGISTRY` or `DOMAIN_ATTESTATION`; the protocol constants remain TBD.
They test serialization and hashing only, not registration, cursor-transition validity or certificate acceptance.
The sample `eigenda_cert` bytes are not a valid EigenDA certificate.
Hexadecimal strings denote bytes without a terminator; the sample domain strings are the exact bytes shown, without padding.
A conforming encoder MUST reproduce the following registry-evidence encoding and digest for the supplied sample domain and fields:

```json
{
  "DOMAIN_REGISTRY": "0x535045432d52454749535452592d4558414d504c45",
  "registry_evidence": {
    "reference_block": 1,
    "quorums": [
      {
        "quorum_number": 0,
        "apk": {
          "X": 1,
          "Y": 2
        },
        "total_stake": 100,
        "non_signer_stakes": [],
        "confirmation_threshold": 67
      }
    ],
    "non_signer_quorum_bitmaps": []
  },
  "abi.encode(registry_evidence)": "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000004300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
  "registry_evidence_hash": "0x15178fbdc36f71a688a445141e9a1c520035b59397daab56743bd439ee0ca9b9"
}
```

A conforming encoder MUST reproduce the following attestation preimage and digest for the supplied sample domain and fields, using the registry-evidence hash from the preceding vector:

```json
{
  "DOMAIN_ATTESTATION": "0x535045432d4154544553542d4558414d504c45",
  "CHAIN_ID": 1,
  "batch_inbox_address": "0x1234567890123456789012345678901234567890",
  "prev_cursor": {
    "height": 1,
    "index": 0,
    "block_number": 10,
    "epoch_num": 100,
    "frames_digest": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "quorum_snapshot_hash": "0x1111111111111111111111111111111111111111111111111111111111111111"
  },
  "eigenda_commitment": "0x2222222222222222222222222222222222222222222222222222222222222222",
  "eigenda_cert": "0x010203",
  "eigenda_cert_hash": "0xf1885eda54b7a053318cd41e2093220dab15d65381b1157a3633a83bfd5c9239",
  "registry_evidence_hash": "0x15178fbdc36f71a688a445141e9a1c520035b59397daab56743bd439ee0ca9b9",
  "cursor": {
    "height": 2,
    "index": 0,
    "block_number": 11,
    "epoch_num": 101,
    "frames_digest": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "quorum_snapshot_hash": "0x1111111111111111111111111111111111111111111111111111111111111111"
  },
  "preimage": "0x535045432d4154544553542d4558414d504c450000000000000001123456789012345678901234567890123456789000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000011111111111111111111111111111111111111111111111111111111111111112222222222222222222222222222222222222222222222222222222222222222f1885eda54b7a053318cd41e2093220dab15d65381b1157a3633a83bfd5c923915178fbdc36f71a688a445141e9a1c520035b59397daab56743bd439ee0ca9b900000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b000000000000000000000000000000000000000000000000000000000000006500000000000000000000000000000000000000000000000000000000000000001111111111111111111111111111111111111111111111111111111111111111",
  "attest_digest": "0xe105a01958e3cc630411983c5941bac716949872bf1afdef14e81386e551fd1f"
}
```

### EigenDA Certificate and Payload

`EIGENDA_CERT_FORMAT` MUST be the standard ABI encoding of the single [`EigenDACertV3` tuple](https://github.com/Layr-Labs/eigenda/blob/26709ca468f176eb23c09f52a3122e5e18681c7d/contracts/src/integrations/cert/EigenDACertTypes.sol), with the nested types, field order and integer widths at that revision.
`eigenda_cert` MUST contain exactly `abi.encode(cert)`, without a function selector or transport prefix, and decoding followed by re-encoding MUST reproduce the input bytes exactly.
The proxy's prefixed RLP transport representation is not an accepted `eigenda_cert` encoding.
Certificate decoding does not establish certificate validity; the [verification procedure](#verification-procedure) supplies the required checks.

`EIGENDA_CODEC` MUST use EigenDA V2's [`PayloadEncodingVersion0` framing](https://github.com/Layr-Labs/eigenda/blob/bfa451e1aaa5eb53ec250758054bbc3267e66208/api/clients/v2/coretypes/payload.go#L15-L46) and [`PolynomialFormEval` conversion](https://github.com/Layr-Labs/eigenda/blob/bfa451e1aaa5eb53ec250758054bbc3267e66208/api/clients/v2/coretypes/encoded_payload.go#L81-L105), including that revision's field-element byte order, evaluation ordering and inverse-FFT roots.
The framing consists of a header symbol carrying the encoding version and big-endian payload byte length, followed by payload chunks padded into field symbols, with the header and body together zero-padded to a power-of-two symbol count.
The G1 commitment MUST be the BN254 KZG commitment to the resulting coefficient polynomial using [`EIGENDA_SRS`](#constants); the length compared by `payload_matches` MUST be that padded symbol count, not the payload byte length.
The core MUST fail the call if the complete submission payload exceeds [`EIGENDA_MAX_PAYLOAD_BYTES`](#constants).
The same codec, setup and length definition MUST apply to [derivation's payload check](../rollup/data-availability.md#batch-commitment-resolution).
`commitment_of(cert)` remains the hash of the whole ABI-encoded `BlobCommitment` defined in the verification procedure, not the raw G1 point.

**Encoding test vectors.**
For the following ASCII payload, the codec and certificate hashes MUST match these values.
This tests the EigenDA encoding only; the payload is not an OP Stack frame submission.
The certificate input is the exact byte sequence in the pinned [ABI certificate fixture](https://github.com/megaeth-labs/mega-block-certifier/blob/f396fb98bc6316414e9458125cdf901c9d3bfae4/crates/block-certifier-server/testdata/eigenda_cert_v3.abi), whose SHA-256 is shown below.
`G1.X` and `G1.Y` are the unsigned, big-endian coordinates, each occupying one ABI word.
Successful decoding of this fixture does not bypass reference-block, registry or certificate-validity checks in an actual submission.

```text
payload = ASCII("block-certifier eigenda cert fixture v1")
payload_byte_length = 39
blob_length = 4
G1.X = 0x26a58b041df9851c07e57a475d4f16b45dd8a31c7c75d4f00998bd264d23d244
G1.Y = 0x1d00c54fae15a334568a92b4caef7058e7e981961598b38cd9b8be551e27076f
certificate_byte_length = 2144
sha256(eigenda_cert) = 0xcf68e78c709590f583fd968cc8c5ab0bca5fcf4faa818bef18cd029ad28aff89
keccak256(eigenda_cert) = 0x943e9423af92f7254891580c977e11fec6f2522b257026069ef70724dbfbce19
commitment_of(decode_cert(eigenda_cert)) = 0xfe6301130cfdcd28d5a76f86231180bcb5696ca26149e06180b98a57bb5e9d54
```

### Inputs and Outputs

```text
EspressoBlockData {
    header:     bytes,   // one Espresso block header
    proof:      bytes,   // finality proof for that header
    ns_payload: bytes,   // this namespace's content in that block, possibly empty
    ns_proof:   bytes,   // ties ns_payload to the header
}

QuorumEvidence {
    quorum_number:          uint8,     // an EigenDA quorum the certificate is signed for
    apk:                    G1Point,   // the quorum's aggregate public key at reference_block
    total_stake:            uint96,    // the quorum's total stake at reference_block
    non_signer_stakes:      uint96[],  // one per certificate non-signer; zero for a non-member of this quorum
    confirmation_threshold: uint8,     // signed-stake percentage the quorum must reach, from the verifier governing reference_block
}

RegistryEvidence {
    reference_block:          uint32,            // L1 block at which operator state is authenticated
    quorums:                  QuorumEvidence[],  // one per signed EigenDA quorum, in ascending quorum_number
    non_signer_quorum_bitmaps: uint256[],         // one per certificate non-signer, in the certificate's order
}

CertifierInput {
    previous_frames:   bytes[],            // accepted prefix, each entry version || one frame encoding
    payload:           bytes,              // version || consecutive new frame encodings of that channel
    cursor:            Cursor,
    espresso_blocks:   EspressoBlockData[], // ascending and contiguous from cursor.height
    quorum:            bytes,               // the quorum already anchored at cursor.height
    quorum_steps:      bytes[],             // steps reaching every epoch espresso_blocks spans
    eigenda_cert:      bytes,               // the certificate EigenDA issued for dispersing payload
    registry_evidence: RegistryEvidence,    // the operator-set data the certificate is verified against
}

CertifierOutput {
    eigenda_commitment:     bytes,    // commitment the entire submission payload was dispersed under
    eigenda_cert_hash:      bytes32,  // keccak256(eigenda_cert)
    registry_evidence_hash: bytes32,  // the digest of registry_evidence fixed under Encodings
    prev_cursor:            Cursor,   // the cursor this call started from
    cursor:                 Cursor,   // the resulting cursor
    quorum:                 bytes,    // the quorum backing the resulting cursor; the next call supplies it as `quorum`
    certifier_sig:          bytes65,
}
```

`previous_frames` MUST contain the complete accepted prefix of the channel being continued, starting at frame 0, or be empty when opening a channel.
It is a witness for the core, not part of the new submission payload, and MUST NOT be reposted as new data.
`payload` MUST encode the non-empty new suffix under [Encodings](#encodings).
`eigenda_cert` is the certificate the [EigenDA](https://docs.eigencloud.xyz/eigenda/core-concepts/overview) disperser issued for that entire payload, so the payload is dispersed before the core is called and the core verifies the certificate rather than predicting it.
`registry_evidence` is the EigenDA operator-set data the certificate's aggregate signature and stake thresholds are checked against, in the form [`EIGENDA_CERT_FORMAT`](#constants) fixes for the certificate and the structure above fixes for the evidence.
`G1Point` is an EigenDA BN254 G1 point encoded for this ABI as `(uint256 X, uint256 Y)` in that order.
For certificate non-signer `j`, bit `q` of `non_signer_quorum_bitmaps[j]` MUST be one exactly when that operator belongs to EigenDA quorum `q` at `reference_block`.
Every quorum's `non_signer_stakes` MUST have one entry per certificate non-signer, in the same order; entry `j` MUST be that operator's historical stake in the quorum if its membership bit is set, and zero otherwise.
A zero stake MUST NOT be used to infer absence of membership.
The [inbox](batch-inbox.md#acceptance) authenticates these values from EigenDA's registries; the core's supplied evidence alone does not authenticate them.
The core cannot read L1, so this data is supplied to it and the [inbox](batch-inbox.md#acceptance) authenticates it afterwards by rebuilding it from EigenDA's registries and comparing `registry_evidence_hash`.

The core MUST NOT retrieve any input itself.
Every Espresso block, proof, quorum step, certificate and registry datum it evaluates MUST come from `CertifierInput`, and the core MUST hold no verification state across calls — `cursor` supplies both the position already consumed and the next L2 height expected.
Given the same `CertifierInput` and fixed protocol and deployment parameters, any party can reproduce the verification result, the unsigned output fields and the [attestation digest](#encodings).
`certifier_sig` additionally depends on the enclave's `CERTIFIER_KEY`, generated under [Attestation](#attestation) and absent from `CertifierInput`; it is not covered by that reproducibility claim.
Different enclaves need not produce identical signature bytes, even when they attest the same cursor transition and commitment.

`quorum_steps` is ordinarily empty, because a quorum changes at most once per Espresso epoch and a single call ordinarily spans far less Espresso history than one epoch.
A call MAY nonetheless need them, and `quorum_steps` MUST then carry exactly the steps from the epoch the anchored quorum governs to each epoch the supplied blocks enter, in ascending epoch order, and no others.
For the first call after deployment, this includes the walk from the scheduled trusted quorum's epoch to `DEPLOYMENT_HEIGHT`'s epoch and any further transitions required by the supplied blocks.
Additional quorum-authentication evidence needed only to establish a supplied block's finality MUST be carried in that block's `proof`, as [Espresso Verification](#espresso-verification) requires.
Verifying that evidence MUST NOT advance the scan's governing quorum or cursor; `quorum_steps` advances the governing quorum when the scan enters another epoch.

### Verification Procedure

```python
def certifier_core(inp):
    # 0. parse all newly submitted frames, then prepend the accepted prefix witness
    new_frames = decode_submission_payload(inp.payload)
    assert new_frames != []
    frame_payloads = inp.previous_frames + new_frames
    frames = [decode_frame_payload(payload) for payload in frame_payloads]
    c      = inp.cursor
    last   = frames[-1]

    # 1. authenticity: anchor on the quorum the cursor committed to, then check every
    #    supplied block against the quorum governing that block's own epoch, stepping
    #    the quorum forward as the blocks cross epoch boundaries
    assert digest(inp.quorum) == c.quorum_snapshot_hash
    assert inp.espresso_blocks != []
    assert contiguous(inp.espresso_blocks, starting_at=c.height)
    quorum, steps, governing = inp.quorum, list(inp.quorum_steps), {}
    for blk in inp.espresso_blocks:
        while not governs(quorum, blk.header):
            assert steps != []
            quorum = resolve_quorum(quorum, [steps.pop(0)])
        assert verify_finalized(blk.header, blk.proof, quorum)
        assert verify_namespace(blk.ns_payload, blk.ns_proof, blk.header)
        governing[height_of(blk.header)] = quorum
    assert steps == []

    # 2. exactly one channel, with no gaps, duplicates, frames after closure, or a raw size over the channel bound
    assert all(f.channel_id == last.channel_id for f in frames)
    assert all(f.number == i for i, f in enumerate(frames))
    assert all(not f.is_last for f in frames[:-1])
    assert sum(len(f.frame_data) + FRAME_OVERHEAD for f in frames) <= MAX_RLP_BYTES_PER_CHANNEL
    if inp.previous_frames != []:
        assert c.frames_digest != bytes32(0)
        assert rolling_hash(inp.previous_frames) == c.frames_digest
    else:
        assert c.frames_digest == bytes32(0)

    # 3. every complete block matches the record selected from the cursor
    end = advance_selected(c, decode_complete_blocks(
        concat(f.frame_data for f in frames), closed=last.is_last), inp.espresso_blocks)

    # 4. a non-closing submission permits posting; closure also commits the run
    out = copy(end if last.is_last else c)
    out.frames_digest = bytes32(0) if last.is_last else rolling_hash(frame_payloads)
    out.quorum_snapshot_hash   = digest(governing[out.height])

    # 5. the certificate certifies the entire new payload, signed by the operator set
    #    the evidence describes, at the thresholds the deployment requires
    cert = decode_cert(inp.eigenda_cert)
    assert payload_matches(cert, inp.payload)
    commitment = commitment_of(cert)
    assert verify_cert(cert, inp.registry_evidence)

    cert_hash     = keccak256(inp.eigenda_cert)
    evidence_hash = registry_evidence_hash(inp.registry_evidence)
    return CertifierOutput(commitment, cert_hash, evidence_hash, c, out, governing[out.height],
                           sign(CERTIFIER_KEY.sk,
                                attest_digest(c, commitment, cert_hash, evidence_hash, out)))
```

The core MUST perform every check above, and MUST NOT emit a `CertifierOutput` when any of them fails.
Each `assert` is normative: it states a condition that MUST hold of the input for an attestation to be produced.

The helpers the procedure composes are:

- `decode_submission_payload(payload)` — consumes one derivation-version byte and every following frame encoding under [`CHANNEL_PARAMS`](#constants), returning one byte string per new frame in wire order: that version byte followed by the frame's exact original encoding.
  An empty payload, a version byte with no frames, an unsupported version, a truncated or malformed frame, an invalid boolean encoding, an exceeded frame or payload limit, or trailing bytes that do not form a complete frame MUST fail the entire call without producing a `CertifierOutput`.
  The helper MUST consume and validate the entire payload before returning; it MUST NOT return a valid prefix after a later frame fails parsing.
- `decode_frame_payload(payload)` — consumes the derivation-version byte and exactly one OP Stack [frame](https://specs.optimism.io/protocol/derivation.html#frame-format), using the version and size limits pinned by [`CHANNEL_PARAMS`](#constants), and returns its `channel_id`, `number` (the encoded `frame_number`), `frame_data`, and `is_last`.
  An unsupported version, truncated field or declared data, invalid boolean encoding, exceeded frame limit, or any bytes after that frame MUST fail the entire call without producing a `CertifierOutput`.
  This single-frame helper parses prefix-witness entries and the entries returned by `decode_submission_payload`, not the multi-frame submission payload itself.
  Parsing MUST preserve the original frame encodings: `rolling_hash` consumes the version-prefixed frame entries verbatim, while `payload_matches` consumes `inp.payload` verbatim.
- `digest(x)` — the hash over `x`'s canonical encoding fixed under [Encodings](#encodings), for a `Cursor` and for a quorum respectively.
- `contiguous(blocks, starting_at)` — true if and only if `blocks` is non-empty, in strictly ascending header-height order, and covers every height from `starting_at` through the last supplied block with no gaps.
- `height_of(header)` — the Espresso block height `header` declares.
- `governs(quorum, header)` — true if and only if the Espresso block `header` describes falls in the epoch `quorum` governs.
- `rolling_hash(frames)` — the frame hash chain fixed under [Encodings](#encodings), chained incrementally so that extending it by one frame never requires rehashing the frames before it.
- `decode_complete_blocks(bytes, closed)` — returns complete decoded blocks in order from the concatenated `frame_data` fields, without frame headers, under the OP Stack [channel](https://specs.optimism.io/protocol/derivation.html#channel-format) and [batch decoding](https://specs.optimism.io/protocol/derivation.html#batch-format) rules selected by [`CHANNEL_PARAMS`](#constants), subject to [Closed-Channel Decoding](#closed-channel-decoding) when `closed` is true and the provisional prefix rules below otherwise.
  - Decoding rules:
    After reading a batch's version byte, the helper MUST reject any version other than the singular batch version pinned by `CHANNEL_PARAMS`, failing the entire call without producing a `CertifierOutput`, even if earlier batches matched selected records.
    This version check precedes decoding the batch contents and MUST NOT be handled as a recoverable decoding error.
    Likewise, before decompressing, the helper MUST reject a channel whose compression type byte does not identify the compression algorithm `CHANNEL_PARAMS` pins, failing the entire call without producing a `CertifierOutput`.
    This check covers a byte naming an algorithm OP Stack derivation accepts but `CHANNEL_PARAMS` does not pin, such as brotli when zlib is pinned, and MUST NOT be handled as a recoverable decoding error.
    When `closed` is true, the helper MUST validate the complete compressed stream under [Closed-Channel Decoding](#closed-channel-decoding) before decoding any batch.
    When `closed` is false, at end of input, a decompression error in a stream of the pinned algorithm, or a batch-encoding error, the helper MUST stop and return the complete blocks already decoded from the provisional prefix, without emitting the incomplete or malformed batch or searching the remaining bytes for another batch.
    These provisional results MUST NOT establish that the eventual closed stream is valid; closure MUST re-evaluate the complete bytes, including the accepted prefix witness.
    If decoding cannot emit any block, the result MUST be empty.
    The core MUST decode under the pinned [`CHANNEL_PARAMS`](#constants), which is the same pinning [batch submission](../rollup/batch-submission.md#provider-substitution) requires of the batcher that constructed the channel.
    A core decoding under a different compression algorithm, batch version byte, or maximum frame size than the batcher encoded under would either fail to decode a well-formed channel or decode a different block sequence from the same bytes.
    In the second case it would attest content no observer reproduces.
  - What the core does with the result:
    When `closed` is false, an incomplete suffix MAY be completed by later frames; the intermediate output extends `frames_digest` over the supplied frames and MUST NOT advance the record position, L2 height or L1 origin.
    When `closed` is true, the undecoded suffix MUST be discarded, and the output MUST advance only through the returned blocks after all of them pass [comparison against the selected records](#advancing-the-cursor), resetting `frames_digest` as step 4 specifies.
    A decoded block that fails comparison against the selected record MUST fail the entire call; it MUST NOT be treated as a decoding error that merely ends the returned sequence.
    The channel's height boundary and the whole-channel failure outcome are specified under [Activation Channel Boundary](activation.md#channel-boundary).
    Non-closing attestations do not authorize derivation to release a channel prefix before the whole-channel boundary check.
- `decode_cert(bytes)` — parses an EigenDA certificate under the pinned [`EIGENDA_CERT_FORMAT`](#constants).
  It yields the batch the certificate belongs to, the blob's inclusion proof in that batch, the blob's commitment, declared length and the quorum numbers its blob header declares, the L1 reference block its operator set is taken at, the quorums it is signed for and their aggregate G1 public keys, the aggregate signature and aggregate G2 public key, and the public keys of the operators that did not sign.
  Any other form MUST fail the entire call.
- `payload_matches(cert, payload)` — true if and only if the BN254 G1 KZG commitment and the length that [EigenDA](https://docs.eigencloud.xyz/eigenda/core-concepts/overview) dispersal of the complete submission payload produces, both computed from `payload` under the pinned [`EIGENDA_CODEC`](#constants), equal respectively the G1 commitment and the length in the certificate's blob commitment.
  The certificate's G2 length commitment and G2 length proof are not recomputed, for the reason the [Rationale](#rationale) gives.
  The core MUST compute both values under the pinned `EIGENDA_CODEC`: a core computing under a different payload-to-blob codec than the disperser uses would never find a certificate whose blob commitment matches the very bytes it attested, and could attest nothing.
- `commitment_of(cert)` — `keccak256(abi.encode(cert.BlobCommitment))`, the hash of the standard ABI encoding of the certificate's blob commitment structure — its G1 commitment, G2 length commitment, G2 length proof and length — as [`EIGENDA_CERT_FORMAT`](#constants) lays it out; the same function the inbox applies.
- `verify_cert(cert, evidence)` — true if and only if all of the following hold:
  - the certificate's blob is included in its batch, that is, the blob certificate hashes into the batch root at the position the inclusion proof names;
  - `cert.reference_block == evidence.reference_block`;
  - the signed EigenDA quorum numbers are non-empty and strictly increasing and match `evidence.quorums` exactly in number and order;
  - the blob header's quorum numbers are non-empty and strictly increasing and are each among the signed quorum numbers, so that `blob quorums ⊆ signed quorums`; this is the core's half of the `required ⊆ blob ⊆ confirmed` rule EigenDA's [certificate verification](https://github.com/Layr-Labs/eigenda/blob/bfa451e1aaa5eb53ec250758054bbc3267e66208/contracts/src/integrations/cert/libraries/EigenDACertVerificationLib.sol#L115-L121) requires, and the inbox checks the required-quorum half at [acceptance](batch-inbox.md#acceptance);
  - the certificate's non-signers are strictly increasing by EigenDA operator ID, `keccak256(abi.encode(X, Y))` of the non-signer G1 public key, and `evidence.non_signer_quorum_bitmaps` and every `non_signer_stakes` array have exactly one entry per non-signer;
  - every stake entry whose membership bit is unset is zero;
  - the certificate's aggregate BLS signature verifies once against the combined signing public key defined below, including the aggregate G1/G2 key-consistency check; and
  - for every quorum the blob header declares, `total_stake > 0`, and `signed_stake = total_stake - sum(non_signer_stakes)` is non-negative and satisfies `100 * signed_stake >= confirmation_threshold * total_stake`, where `confirmation_threshold` is the value carried in that quorum's `QuorumEvidence` and MUST be in the inclusive integer range `1..100`; the inbox supplies it from the certificate verifier governing `cert.reference_block` and authenticates it through `registry_evidence_hash`.

  Let `Q` be the signed quorum numbers, `P[j]` the certificate's non-signer G1 public keys, and `bitmap[j] = evidence.non_signer_quorum_bitmaps[j]`.
  The combined signing public key MUST be computed with BN254 group operations as:

  ```text
  signed_quorum_bitmap = sum(1 << q for q in Q)
  multiplicity[j] = popcount(bitmap[j] & signed_quorum_bitmap)
  signing_apk = sum(evidence.quorums[i].apk for i in range(len(Q)))
                - sum(multiplicity[j] * P[j] for j in range(len(P)))
  ```

  Here `popcount` counts set bits, and multiplication by `multiplicity[j]` is scalar multiplication of the public point.
  A non-signer belonging to two signed quorums is subtracted twice; one belonging to neither is not subtracted.
  The core MUST apply the aggregate-signature and G1/G2 key-consistency checks of EigenDA's [pinned BLS signature checker](https://github.com/Layr-Labs/eigenlayer-middleware/blob/2f7c93e38f56f292f247981a52bd3619a16b9918/src/BLSSignatureChecker.sol#L259), using `signing_apk`, the certificate's aggregate G2 key and signature, and the signed batch digest defined by [`EIGENDA_CERT_FORMAT`](#constants).
  Invalid group encodings, failed group operations or failed pairing checks MUST fail the call.
  The combined signature MUST NOT be verified separately against each quorum's key.
  Stake sums and percentage comparisons MUST use mathematical integer arithmetic without overflow or underflow.
  The signature authenticates the combined signer set, every signed quorum included; stake thresholds are checked separately for each quorum the blob header declares, and a signed quorum the blob does not declare need not reach the threshold, as in EigenDA's certificate verification, where it is simply not confirmed.
  A quorum the blob header declares with zero total stake MUST fail verification; it MUST NOT satisfy the threshold vacuously.
  These bounds do not prohibit zero stake for an individual member and do not replace any stronger security-parameter constraints of the pinned EigenDA profile.
  The [inbox](batch-inbox.md#acceptance) authenticates the registry values and membership bitmaps used by these checks through `registry_evidence_hash`; the remaining certificate encoding and validation rules are those of the pinned `EIGENDA_CERT_FORMAT`.

- `registry_evidence_hash(evidence)` — the digest of the registry evidence fixed under [Encodings](#encodings).
- `attest_digest(prev_cursor, eigenda_commitment, cert_hash, evidence_hash, cursor)` — the attestation digest fixed under [Encodings](#encodings).
- `sign(sk, msg)` — an ECDSA signature by the enclave's own [`CERTIFIER_KEY`](#constants).

Step 1 binds the supplied quorum to the cursor's anchor before using it: `digest(inp.quorum) == c.quorum_snapshot_hash` is checked first, so the host cannot substitute a different starting quorum.
This checks consistency with the cursor, not the anchor's reachability from the scheduled trusted quorum; that trust is inherited from bootstrap and normal attestations or depends on a correctly supplied override.
Resolving a quorum before that check would invert the order the trust argument depends on, because `resolve_quorum` establishes only that its result is reachable from its starting point, never that the starting point was legitimate.
The quorum is then stepped forward exactly when the block under verification falls outside the epoch the current quorum governs, and each block is verified against the quorum governing its own epoch.
A run verified wholly against one quorum would reject every block past a boundary if that quorum were the earlier one, and accept blocks against a validator set that did not govern them if it were the later one.
The closing `assert steps == []` requires every supplied step to have been consumed, so an input cannot carry a quorum the blocks never needed; an unconsumed step is evidence that the host and the core disagree about which epochs the run spans, which is not a condition an attestation should be produced under.

The supplied blocks MUST always include the block at `cursor.height`, which is why step 1 rejects an empty `inp.espresso_blocks` outright rather than leaving the case to `contiguous`.
That block is both where the scan of the [selected record sequence](block-records.md#canonical-record-sequence) resumes and the block whose governing quorum the resulting cursor reports, so a call supplying no blocks has no quorum to anchor its output on.
This does not prevent a vacuous attestation — a call whose frames complete no L2 block still attests, as stated below — it requires only that the Espresso evidence covering the cursor's own position is present.

`out.quorum_snapshot_hash` MUST be the digest of the quorum governing the epoch that `out.height` falls into, which is what keeps the [`Cursor`](#cursor) invariant — that `quorum_snapshot_hash` is anchored at `height` — true of every cursor the core emits, and therefore keeps the next call's anchor check satisfiable.
For a non-closing submission, `out.height == c.height`; the quorum hash remains unchanged when the input quorum already governs that height's epoch, even if the call resolves quorums for later supplied blocks.
If the input anchors an earlier epoch, including an initial scheduled anchor before the deployment epoch, the output MUST instead carry the digest of the quorum reached by verified transitions that governs `c.height`.
This updates the quorum anchor without advancing the record position, L2 height or L1 origin.

Step 2's `channel_id` equality is load-bearing and is not implied by the frame numbering or the digest chain.
Under [Holocene frame ordering](https://github.com/ethereum-optimism/specs/blob/42b446f5891dd5a232cfc346201a613c1bba3f50/specs/protocol/holocene/derivation.md#frame-queue), derivation immediately drops a non-first frame whose channel ID differs from the preceding frame's.
Without the equality check, the core could concatenate such frames into a complete channel while derivation never completes that channel, leaving an accepted cursor advance past content that did not derive.
Step 2 requires numbering from zero across the prefix witness and the new suffix, so a continuation starts exactly after the previously accepted frames and no frame follows a closing frame.
With an empty `previous_frames`, the first new frame MUST be numbered 0 and `c.frames_digest` MUST be zero, so a channel opens only when none is open.
An open channel stops being open only by closing or, at the inbox, by timing out, after which the inbox checks the submission against a zeroed digest, per [Recovery by an Alternative Batcher](batch-inbox.md#recovery-by-an-alternative-batcher).
With a non-empty prefix witness, its digest MUST equal the cursor's non-zero digest; a host cannot omit, alter, reorder or replay accepted frames as the new suffix.
Step 2's raw size bound mirrors the rule derivation's [channel assembler](https://specs.optimism.io/protocol/holocene/derivation.html#channel-bank) applies after adding each frame: a channel whose raw size — the sum over its frames of `frame_data` length plus [`FRAME_OVERHEAD`](#constants) — exceeds [`MAX_RLP_BYTES_PER_CHANNEL`](#constants) is dropped whole before any batch is read, and every later frame of it is dropped with it.
That rule is distinct in condition and in effect from the [closed-stream decompressed-byte bound](#closed-channel-decoding): exceeding the raw bound fails the certifier call, whereas exceeding the decompressed bound makes a closing channel yield no blocks.
Without the raw check a channel over that bound could be attested and advanced past while derivation discarded its frames before decoding.
One check on the total over the prefix witness and the new frames is equivalent to derivation's per-frame check, because frame sizes are non-negative and derivation drops the channel at whichever frame crosses the bound.
The inbox accepts one cursor transition for the entire submission, and derivation receives every new frame from its committed payload; neither receives the prefix witness as new data.

A non-closing submission MUST compare every complete decoded block against its selected record, but MUST NOT advance the record cursor.
If no complete block is decoded, no record-content comparison is required.
The output commits to the accumulated frames through `frames_digest`; channel closure MUST re-evaluate the complete channel before committing record progress.
A call whose frames yield no complete decoded L2 block attests vacuously: step 3 advances no record, and step 4 extends `frames_digest` over the supplied frames for an open channel or resets it to the 32 zero bytes for a closed channel.

Step 5 divides certificate verification between the core and the inbox so that each check is performed exactly once, by the party positioned to perform it.
The core checks blob inclusion, the aggregate signature and the stake thresholds, which need no L1 state beyond the operator-set values `registry_evidence` supplies; the inbox rebuilds those values from EigenDA's registries at `cert.reference_block`, takes the confirmation threshold and required quorums from the certificate verifier governing that block, and checks the reference block's freshness against `block.number`, none of which the core can see.
Because the attestation covers `keccak256(eigenda_cert)` and `registry_evidence_hash`, the inbox accepts only the certificate the core verified and only if the evidence the core was given is what the registries hold; a host that skipped the core, or fed it fabricated stakes or keys, produces a submission the inbox rejects.

### Closed-Channel Decoding

For a closed channel, the core MUST validate decompression of the complete concatenated `frame_data` bytes before decoding any batch or comparing any block against a selected record.
The existing [compression-type check](#verification-procedure) MUST precede this validation and MUST retain its whole-call failure outcome.
After that check, a decompressed byte string MUST exist only when the channel begins with one complete stream of the algorithm pinned by [`CHANNEL_PARAMS`](#constants), passes all of that stream's format and integrity checks, and produces at most [`MAX_RLP_BYTES_PER_CHANNEL`](#constants) decompressed bytes.
For zlib, validation MUST apply [RFC 1950's format and compliance checks](https://www.rfc-editor.org/rfc/rfc1950.html#section-2), including the compression method, window-size field, header check and Adler-32 trailer, and MUST reject a stream using a preset dictionary.
Bytes following the first complete valid stream MUST be ignored, including any second compressed stream.

A truncated stream, malformed stream, failed integrity check or output exceeding the decompressed-byte bound MUST yield an empty block sequence for the entire channel, regardless of any bytes a streaming decompressor produced before detecting the failure.
The helper MUST NOT salvage or clip a prefix of such a stream, and MUST NOT search later bytes for a replacement stream.
These failures MUST NOT alone fail the certifier call; if all other checks pass, it MUST attest a vacuous close, leaving the record position, `block_number` and `epoch_num` unchanged and resetting `frames_digest` to zero under [Verification Procedure](#verification-procedure).
The quorum anchor MUST retain that procedure's ordinary update rules.

After successful stream validation, the core MUST decode batches from the resulting complete byte string in wire order.
At end of input or a batch-encoding error, it MUST retain only preceding complete batches, discard the incomplete or malformed suffix, and MUST NOT search that suffix for another batch.
The existing unsupported-batch-version and selected-record comparison checks MUST retain their whole-call failure outcomes; they MUST NOT be converted into recoverable batch-encoding errors.
The [activation channel-boundary check](activation.md#channel-boundary) MUST remain applicable to the complete decoded sequence.

The result MUST depend only on the complete channel bytes and pinned parameters, not on input chunking, output buffering or the decompressor's timing of error delivery.
A streaming implementation MAY validate incrementally, but MUST NOT commit or release any block before complete-stream validation succeeds.
Conformance vectors MUST cover a corrupt integrity trailer after complete batches, a truncated stream, output exactly at and above the decompressed-byte bound, trailing bytes after a valid stream, a malformed batch suffix within a valid stream, and different input and output chunk sizes for the same channel bytes.

### Advancing the Cursor

`advance_selected` threads a `Cursor` through the [selected record sequence](block-records.md#canonical-record-sequence), checking each decoded block against the record that sequence yields next:

```python
def advance_selected(cursor, blocks, authenticated_espresso_history):
    selected = next_selected(cursor, authenticated_espresso_history)
    for b in blocks:
        r = next(selected, None)
        assert r is not None
        assert (r.transactions, r.epoch_num) == (b.transactions, b.epoch_num)
        assert r.block_number >= genesis_block_number
        assert b.timestamp == (genesis_time
                               + (r.block_number - genesis_block_number) * l2_block_time)
        height, index = position_of(r)
        cursor = Cursor(height, index, r.block_number, r.epoch_num,
                        cursor.frames_digest, cursor.quorum_snapshot_hash)
    return cursor
```

The comparison is a byte equality over the record's two pinned content fields together with an arithmetic check on the height the batch claims, and involves no execution.
Whether the block's transactions apply against the parent state is not something the core establishes, and therefore a TEE attestation alone does not establish the full node's certified status.

`b` is an OP Stack [`SingularBatch`](https://specs.optimism.io/protocol/derivation.html#batch-format), which carries five fields, and the checks above read three of them: `transactions`, `epoch_num`, and `timestamp`.
The timestamp check is not implied by the height the cursor already tracks, because `block_number` never travels inside a batch.
`timestamp` is the field [derivation](../rollup/derivation.md) reads to decide which L2 height a batch fills.
Binding it to the record's own `block_number` through the [protocol-fixed bijection](block-records.md#fields) `timestamp = genesis_time + (block_number - genesis_block_number) * l2_block_time` is therefore what ties the selected record's content to the height that record was selected for.
The parameters are those defined under [Block Record Fields](block-records.md#fields), taken from the rollup configuration rather than the certification activation checkpoint.
The check is stated against that bijection rather than against the preceding batch because the bijection is an OP Stack consensus rule and the record's `block_number` is the authority for which height the content belongs to.

The core places no bound of its own on `epoch_num`, for two reasons.
The rule that a record's L1 origin does not decrease is a condition of [Record Validity](block-records.md#record-validity), evaluated against the `cursor.epoch_num` the core passes the scan as its preceding origin.
The OP Stack rule that a block's origin advances by at most one is derivation's own, applied against the derived parent.
`next_selected` therefore cannot yield a record that violates the rule, and a redundant assert here would be unreachable.

`parent_hash` and `epoch_hash` are deliberately not checked by the core, and the reason is that neither is computable on the verification path.
Checking `parent_hash` would require the hash of the canonical L2 parent, which requires executing the chain up to that point, and the core performs no execution.
Checking `epoch_hash` would require knowing which L1 block is canonical at `epoch_num`, which requires authenticated L1 data the core is not supplied.
Moving either check into the record does not help: pinning the origin's hash there would reintroduce exactly the L1-reorganization brittleness that [keeping the origin's hash out of the record](block-records.md#rationale) avoids, which is a separate and still-valid argument about the record's contents rather than about what the core can compute.
The OP Stack baseline requires a batch's `parent_hash` to equal the hash of the canonical L2 parent and its `epoch_hash` to equal the hash of the canonical L1 block at `epoch_num`, and drops a batch failing either without deriving anything from it.
Under the certified rules, derivation [ignores the batch's `parent_hash`](../rollup/derivation.md#batch-parent-hash) and [resolves the canonical L1 origin from `epoch_num`](../rollup/derivation.md#canonical-l1-origin-resolution), ignoring the batch's `epoch_hash`.
What that leaves unpreventable is stated under [Security Considerations](#security-considerations).

`next_selected` is the [record selection pipeline](block-records.md#record-selection-pipeline)'s entry point, defined under [Scan Pseudocode](block-records.md#scan-pseudocode).
When invoked from the core, `next_selected` MUST draw its records from `inp.espresso_blocks` — the blocks step 1 already authenticated — and MUST NOT query Espresso, which the core cannot do.
`authenticated_espresso_history` MUST be the contiguous Espresso blocks authenticated by the core, from the cursor's `height` through the last position the scan needs, scanned from the cursor's position under [Sync Start](block-records.md#sync-start).
`position_of(r)` MUST be the [record position](block-records.md#record-position) associated with the selected record: its span start, which is its batch transaction or the index-0 fragment of its completing attempt.
The core MUST retain that span start even when the attempt completes in a later Espresso block; the emitted quorum hash MUST remain anchored at the resulting cursor's own `height`, which is the span start's height, not the completion height.
The scan MUST follow the record selection pipeline, rebuilding fragment-attempt and nonce state from history and yielding only complete records.
Before comparing any decoded L2 block against its selected record, the core MUST complete the re-selection of the consumed record that [Sync Start](block-records.md#sync-start) requires for the starting cursor.
If the supplied history is insufficient for that re-selection or for selecting a record needed by the comparison, the core MUST fail without emitting an attestation.
When no complete L2 block is decoded, insufficient history for re-selecting the consumed record alone MUST NOT fail the call; all other verification requirements remain applicable.
The publication scanner's nonce and fragment timeout MUST NOT be conflated with `frames_digest` or the OP Stack settlement channel's timeout.

Conformance vectors MUST cover a call decoding no complete L2 block that supplies only the required cursor-height Espresso evidence, with history insufficient to re-select the consumed record, and succeeds without advancing the record position, L2 height or L1 origin when all other checks pass.
They MUST also cover a call decoding at least one complete L2 block that fails without an attestation when evidence needed to re-select the consumed record or to select a required record is missing.

## Attestation

The core's output is only as trustworthy as the evidence that the core is what actually produced it.
That evidence is a hardware attestation over the program's measurement, and its essential property is that the program does not get to make the claim about itself.

The certifier operator is untrusted; the trusted components are the authorized program, the TEE's execution and key protection, and the hardware attestation system.
A deployment that seals its signing key, as the key-sealing rule below permits, adds the sealing service and every party able to change its release policy to those components, and the operator is then trusted for that policy whenever it is such a party.
Hardware attestation binds the enclave's signing key to the authorized image; the core's signed outputs then attest individual submissions.
Trust in those outputs depends on the specified attestation and signature checks, not on the identity or honesty of the operator, apart from that release policy where the key is sealed.

An [AWS Nitro enclave](https://docs.aws.amazon.com/enclaves/latest/user/set-up-attestation.html) is launched from an image file, and the Nitro hypervisor — outside the enclave, part of the platform rather than the workload — hashes what it loads into Platform Configuration Registers.
`PCR0` is the measurement of the complete image: kernel, ramdisk, and application together.
The enclave generates `CERTIFIER_KEY` inside itself and asks the Nitro Security Module to attest it, receiving a [signed attestation document](https://github.com/aws/aws-nitro-enclaves-nsm-api/blob/main/docs/attestation_process.md) that carries the PCR values, the supplied public key, and a certificate chain terminating at the AWS Nitro Enclaves Root CA ([`NITRO_ROOT_CA`](batch-inbox.md#constants)) — signed by a key the enclave never holds.

The enclave MUST place `CERTIFIER_KEY`'s public key in the document's `public_key` field using the encoding specified by [Certifier Key Registration](batch-inbox.md#certifier-key-registration).

The document therefore asserts, on the platform's authority rather than the enclave's, that an image measuring `PCR0` was loaded and that the code running inside it asked to bind this public key.
Establishing that the intended program is running reduces to comparing `PCR0` against the measurement of the authorized build.
An operator cannot forge the measurement, because it does not produce the signature, and cannot substitute a different key, because the key is inside the signed payload.

Authorization MUST attach to the measurement rather than to the operator: a key is eligible only while its registered measurement belongs to the current [authorized-measurement set](batch-inbox.md#certifier-key-registration), and never because of who runs the machine.
[`CERTIFIER_PCR0`](#constants) is the initial entry in that set; governance may authorize other measurements under the same trust premises.
This is what makes the certifier a role any party can occupy rather than an identity the sequencer holds, which the inbox's [certifier key registration](batch-inbox.md#certifier-key-registration) depends on.
Withdrawing authorization for a measurement MUST invalidate every key registered under it, on the terms the [certifier key registration and revocation rules](batch-inbox.md#certifier-key-registration) specify.

A certifier key MUST be accepted only if an attestation document that binds that key to its registered measurement verifies against a certificate chain terminating at [`NITRO_ROOT_CA`](batch-inbox.md#constants), and that fact MUST be established on L1 rather than taken from any off-chain party.
The registered measurement MUST remain authorized when the key is used, per [Certifier Key Registration](batch-inbox.md#certifier-key-registration).
Establishing it on L1 means either verifying the document there or verifying there a proof of the document's validation whose public output carries the measurement and the key; a zero-knowledge proof of the validation is such evidence, and an off-chain verifier's word is not.
Establishing it on-chain is what makes the certifier role permissionless in fact and not only in principle.
Any party able to launch the authorized image may register the key its own enclave generated, in a single transaction, with no approval step and no operator to petition.
This is because the only thing standing between the document and the registry is evidence the contract checks itself.
The two halves of that authorization live in different places and are governed differently.
The authorized measurements are L1 system configuration under the same governance as the chain's other configuration, while the keys registered under them are not curated at all.
The procedure joining them, covering how a document is submitted, what it is checked against, and what withdrawing a measurement invalidates, is specified by the [certifier key registration and revocation rules](batch-inbox.md#certifier-key-registration).

### Key Sealing

An implementation MAY persist `CERTIFIER_KEY.sk` outside the enclave, but only encrypted under a sealing service whose release policy binds that key to the exact measurement of the image that generated it and under which it is registered.
Before initial registration, the release policy MUST already be restricted to that generating image's measurement.
The service MUST release the key only to an enclave attesting that exact measurement; authorization of another image MUST NOT permit that image to obtain the key.
A policy MUST NOT permit multiple measurements to recover the same signing key, and MUST NOT be widened or retargeted to reuse the key during an image upgrade.
A restarted enclave of the same image MAY recover the same sealed key and keep signing under its existing registration while that measurement remains authorized.
`CERTIFIER_KEY.sk` MUST NOT exist outside an enclave of its bound measurement except in this sealed, encrypted form.
An upgrade changing the image's measurement MUST generate a fresh `CERTIFIER_KEY` inside the new image and register it under the new measurement before its signatures are admissible.
The new image MUST NOT import or reuse a signing key generated under another measurement; [Certifier Key Registration](batch-inbox.md#certifier-key-registration) enforces the corresponding permanent on-chain binding.
When sealing is used, the sealing service and every party able to change its release policy are trusted components alongside the authorized program, the TEE's execution and key protection, and the hardware attestation system.
Removing the measurement condition from the policy would release the key to arbitrary code, which could then sign attestations the inbox accepts with no hardware attestation involved.
With sealing, the certifier operator, or whoever else administers the sealing key's policy (typically the operator's cloud account), is therefore trusted for that policy.
The statement that the certifier operator is untrusted holds only for deployments that do not seal, or whose release policy the operator cannot change; the [Security Considerations](#security-considerations) state what a weakened policy costs.

Every authorized image MUST be bit-reproducible from the published source of the certifier core.
Without reproducibility, the authorized `PCR0` is a value only its publisher can vouch for: another party could run the image but could not establish that it is the program this page specifies, and every guarantee above would reduce to trusting the publisher.

## Constants

| Constant                    | Value                                               | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| --------------------------- | --------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `CERTIFIER_KEY`             | `per enclave`                                       | Each enclave's secp256k1 signing keypair, not a protocol constant. `CERTIFIER_KEY.sk` is generated inside the enclave and never leaves it except in sealed form, as [Key Sealing](#key-sealing) permits; the 20-byte address of the public half is registered on L1.                                                                                                                                                                                                                                                                                                                                                                               |
| `CERTIFIER_PCR0`            | TBD                                                 | The initial authorized measurement of the certifier enclave image. Subsequent key eligibility depends on the current [authorized-measurement set](batch-inbox.md#certifier-key-registration).                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| `EPOCH_VERSION`             | `(0, 3)`                                            | Espresso major/minor version threshold below which finality is not accepted and at which the two-certificate rule begins, per [Espresso Verification](#espresso-verification); fixed by the [pinned Espresso version definitions](https://github.com/EspressoSystems/espresso-network/blob/d9df43a3f8a78d97883278c8a663e916598f7529/crates/versions/src/lib.rs).                                                                                                                                                                                                                                                                                   |
| `NEW_PROTOCOL_VERSION`      | `(0, 6)`                                            | Espresso major/minor version threshold at which the single-`Certificate2` rule begins, per [Espresso Verification](#espresso-verification); fixed by the [pinned Espresso version definitions](https://github.com/EspressoSystems/espresso-network/blob/d9df43a3f8a78d97883278c8a663e916598f7529/crates/versions/src/lib.rs).                                                                                                                                                                                                                                                                                                                      |
| `ESPRESSO_FINALITY_PROFILE` | TBD                                                 | Pinned Espresso network, upgrade configuration, finality-proof encoding and the certificate-verification rules beneath the version-dependent commit rule of [Espresso Verification](#espresso-verification), including authenticated epoch transitions; the public-key serialization reference fixes only the key bytes used in the quorum commitment.                                                                                                                                                                                                                                                                                             |
| `CHANNEL_PARAMS`            | TBD                                                 | Pinned OP Stack channel decoding rules and parameters: frame-payload derivation version, compression format, the singular batch format (version byte `0`; span batches are not permitted), maximum frame size, decompressed-byte limit, complete-stream validation for closed channels, and batch-prefix/error handling.                                                                                                                                                                                                                                                                                                                           |
| `MAX_RLP_BYTES_PER_CHANNEL` | TBD                                                 | OP Stack derivation's per-channel byte bound, 100,000,000 bytes from Fjord, which derivation applies twice: as the raw channel size bound its [channel assembler](https://specs.optimism.io/protocol/holocene/derivation.html#channel-bank) drops a channel for exceeding — the sum over the channel's frames of `frame_data` length plus `FRAME_OVERHEAD` — mirrored by [step 2](#verification-procedure), and as the decompressed-byte bound under [Closed-Channel Decoding](#closed-channel-decoding), where exceeding it yields no blocks rather than clipping a prefix. MUST equal the value derivation applies for the chain's active forks. |
| `FRAME_OVERHEAD`            | 200                                                 | The per-frame overhead, in bytes, that OP Stack derivation's [channel bank](https://specs.optimism.io/protocol/derivation.html#channel-bank) adds to `frame_data` length when measuring a channel's raw size.                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| `EIGENDA_CODEC`             | V2, `PayloadEncodingVersion0`, `PolynomialFormEval` | The pinned [payload framing and polynomial conversion](#eigenda-certificate-and-payload) used to recompute the G1 commitment and symbol count.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `EIGENDA_CERT_FORMAT`       | `EigenDACertV3`, canonical ABI                      | The pinned [certificate layout and canonical encoding](#eigenda-certificate-and-payload) accepted by the core and parsed by the inbox.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `EIGENDA_SRS`               | EigenDA G1 setup prefix                             | The ordered G1 points in `g1.point` with SHA-256 `8f18b9c04ed4bddcdb73001fb693703197328cecabdfa9025f647410b0c50d7f`, identified by the pinned [EigenDA setup manifest](https://github.com/Layr-Labs/eigenda/blob/bfa451e1aaa5eb53ec250758054bbc3267e66208/resources/srs/srs-files-16777216.sha256).                                                                                                                                                                                                                                                                                                                                                |
| `EIGENDA_MAX_PAYLOAD_BYTES` | `16_252_897`                                        | Maximum bytes in one submission payload, including its derivation version byte; the pinned setup covers the header and padded payload at this bound.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `DOMAIN_REGISTRY`           | TBD                                                 | Fixed byte string prefixing every registry-evidence digest, separating it from any other hash the attestation digest is composed from.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `DOMAIN_ATTESTATION`        | TBD                                                 | Fixed byte string prefixing every attestation digest, separating certifier attestations from any other signature this key could make.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `DOMAIN_QUORUM`             | `"BLKC-LC-STATE-01"`                                | Fixed byte string prefixing every quorum digest, separating a quorum digest from any other hash a cursor field could be compared against. Its value is the 16 ASCII bytes shown.                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |

## Rationale

**Why a closed stream is validated before batch decoding.**
Keeping whatever batches a decompressor delivered before a checksum or format error makes the accepted prefix depend on buffering and error reporting.
Validating the complete stream first gives the certifier and derivation the same byte string or the same empty result.
A vacuous close clears an unusable channel without passing its records, allowing a fresh channel to retry the unresolved content; failing only the closing transaction could leave accepted prefix frames waiting for correction or timeout.
Batch-prefix recovery remains useful after successful decompression because the complete validated byte string supplies a common batch-decoding input.

**Why one attestation can cover consecutive frames of one channel.**
Grouping frames reduces the number of attestations and L1 batch transactions without waiting for channel closure when earlier submission is useful.
A complete channel can also be submitted in one payload when it fits the pinned limits; channel boundaries remain the batcher's choice.
Restricting a submission to one channel keeps one open-channel digest and one cursor transition sufficient, while `frames_digest` binds the accepted prefix across submissions.
Partial submission does not advance the record cursor or remove the need to close the channel; recovery of an abandoned channel follows the inbox's existing reset rules.

**Why the cursor advances only on channel closure.**
A channel that never closes cannot supply its batches to derivation.
Allowing intermediate frames to advance `block_number` and `epoch_num` would let incomplete submissions consume records and refresh the [exclusive window](batch-inbox.md#exclusive-window)'s origin anchor without delivering a complete channel.
Requiring closure prevents that partial-channel advancement, but neither closure nor an accepted cursor advance proves successful delivery or execution.
The inbox deliberately permits passing records whose batches are already past the sequencing window, as its [expiry rationale](batch-inbox.md#rationale) explains; those heights can remain deposit-only.
A closing channel matching no new record leaves the record fields unchanged, under [Advancing the Cursor](#advancing-the-cursor).

**Why `block_number` names the last record checked at channel closure.**
Three of the [`Cursor`](#cursor)'s other fields describe one specific record: `height` and `index` are that record's span start in Espresso's finalized history, from which a resumed scan continues (re-selecting a fragmented record), and `epoch_num` is the L1 origin it carries.
Defining `block_number` as the next expected height instead would make it the only one of the four describing a different record from the rest, and would leave `epoch_num` as the origin of a block the cursor does not name — a cursor whose fields no longer agree on what they are about.
The [exclusive window](batch-inbox.md#exclusive-window) reads `cursor.epoch_num`, which names the same record's L1 origin.
Before any record is checked, the [initial cursor](activation.md#initial-cursor) names the activation checkpoint and a supplied preceding-origin anchor, which need not equal the derived checkpoint origin.
The checkpoint precedes certification and needs no record, so the first expected height is the checkpoint height plus one, without a sentinel value for `block_number`.
The initial `index` is 0 because the [record position](block-records.md#record-position) includes the publication transaction it names, so no sentinel is needed.

**Why the batch format is pinned to singular batches.**
The core compares each decoded block with one record, so a format that carries one block per batch is the natural unit.
[Span batches](https://specs.optimism.io/protocol/delta/span-batches.html) can reduce encoded size by reducing repeated block metadata and reorganizing transaction fields; grouping frames for attestation does not remove those potential savings.
Supporting spans would require reconstructing transaction bytes from their columnar form exactly as derivation does and adapting the `parent_check` and `l1_origin_check` checks to avoid reintroducing the batcher-chosen drop conditions the [derivation deltas](derivation-pipeline.md) remove for singular batches.
The singular-only restriction forgoes those potential encoding savings in exchange for simpler block-to-record comparison and avoiding that additional decoding and derivation adaptation.

**Why the attestation binds payload bytes through the DA commitment.**
The signed message is [`attest_digest`](#encodings), which covers the previous cursor, the resulting cursor, and `eigenda_commitment` under the deployment's own domain separation, and contains no separate hash of the submission payload.
The payload never reaches L1, so the contract would have nothing to recompute a payload hash against; `eigenda_commitment` already binds the payload's bytes and length, through the G1 commitment and length the core checked against the payload, and the certificate the core verified, which the attestation names by hash, ties that commitment to what derivation will actually fetch.
Putting the payload itself on L1 would place the data on Ethereum and defeat the purpose of using a separate data availability layer.

**Why the commitment is the certificate's blob commitment rather than recomputed in full.**
An EigenDA blob commitment contains the G1 KZG commitment to the blob polynomial, `lengthCommitment`, `lengthProof`, and the declared length.
Under EigenDA's pinned [commitment construction](https://github.com/Layr-Labs/eigenda/blob/bfa451e1aaa5eb53ec250758054bbc3267e66208/encoding/v2/kzg/committer/committer.go#L178-L209), `lengthCommitment` commits to that same polynomial in G2, and `lengthProof` uses shifted G2 setup powers to prove the polynomial's degree bound.
Recomputing these G2 fields requires the corresponding public G2 setup points, including the trailing powers used for the length proof; the selected enclave carries only the G1 setup needed to recompute the payload's G1 commitment.
The core therefore checks the G1 commitment and length against the payload and takes `eigenda_commitment` as the hash of the certificate's whole blob commitment, relying on the assumed honest EigenDA quorum for correct construction of the G2 fields.
Under the KZG binding assumption, the pinned codec and the checked length, a different payload cannot feasibly pass `payload_matches` against the same G1 commitment.
For a fixed payload, codec and setup, correctly constructed G2 fields are deterministic, so the whole blob commitment is determined by the payload.
A dishonest quorum could sign inconsistent G2 fields, violating the quorum-correctness assumption; a valid quorum signature would not make those fields valid commitments or proofs for that polynomial.
The G2 values are in any case bound by the attestation through `keccak256(eigenda_cert)`.

**Why the cursor lives on L1 rather than inside the certifier.**
The core holds no verification state between calls, so the cursor must come from somewhere; making the L1 contract its source of truth means any number of certifier instances stay consistent without coordinating, each reading the same public value as its starting point.
The attestation verifies a transition from the supplied cursor; the inbox checks that it extends the expected cursor, avoiding a scan from the scheduled trusted quorum on every call.
Verification cost still depends on the Espresso history and namespace data traversed from that cursor; the cursor does not bound how much can accumulate before the next advance.

**Why the core performs no I/O.**
Keeping retrieval outside the core makes the evidence an explicit input and separates verification from network availability during execution.
Given the same supplied inputs, the verification result can be reproduced without fetching data again.
Authenticity comes from the finality, namespace and anchored-quorum checks under [Espresso Verification](#espresso-verification); fetching the data inside the core would not by itself require trusting its source.

**Why the core verifies the EigenDA certificate and L1 only authenticates its operator set.**
EigenDA's own on-chain certificate verifier checks the aggregate BLS signature by pairing, which would be the most expensive operation an inbox submission carries.
The core performs that check inside the enclave — blob inclusion, the aggregate signature, and the stake thresholds — where it costs nothing on L1, and reports the operator-set data it used as `registry_evidence_hash`.
L1 keeps the part only it can do: it reads EigenDA's registries at the certificate's reference block, which the core cannot see, rebuilds the same evidence and compares hashes, checks the required quorums the certificate verifier governing that block names, and bounds the reference block's age against `block.number`, which the core cannot know.
Every check is thus assigned to exactly one of the two: a host cannot bypass the core, because the attestation names the certificate by hash, and cannot feed the core fabricated stakes or keys, because the inbox would rebuild different evidence and the signature would fail.
The cost is that certificate verification joins the enclave's trusted computing base, which the [Security Considerations](#security-considerations) state.

**Why the component that assembles inputs is not specified.**
The input assembler is not trusted to establish authenticity: the core rejects inputs that fail its verification checks, while the [inbox](batch-inbox.md#acceptance) separately checks cursor continuity, sender authorization and certificate freshness, and authenticates the registry evidence.
An internally consistent input with a stale cursor can still produce an attestation; the inbox rejects the submission if its `prev_cursor` no longer matches the expected cursor.
Withholding inputs or delaying submission can prevent timely delivery, so liveness still depends on the [delivery and availability assumptions](full-node.md#conditions-under-which-it-holds).
Retrieval, input assembly, dispersal and submission mechanisms can differ between implementations that produce the same observable protocol behavior, so those mechanisms are not specified here.

**Why execution verification is outside the current core.**
The canonical parent's hash commits to execution-derived fields, including the state root and receipts root, that the ordinary transaction list alone does not establish.
A proof of the current core's content-matching checks would establish the same checks, not execution validity or agreement with the actual derived parent.
Proving execution would additionally need to bind the authenticated parent state, L1 execution inputs and resulting execution; that proof relation and its protocol integration are not specified here.

**Why key sealing is permitted.**
A key that exists only in enclave memory is lost on every enclave restart, and settlement through that certifier then waits for a fresh registration, which needs a new attestation proof on L1.
Sealing lets a restarted enclave of the same image recover its registered key and avoid that wait.
Its cost is the trust expansion the [Security Considerations](#security-considerations) state: the sealing service and whoever can change its release policy join the trusted components.
Sealing is permitted rather than required, so a deployment that wants its operator fully untrusted can decline to seal and accept re-registration after each restart.

**Why an image upgrade changes the signing key.**
Sharing a key across measurements lets an older image keep signing through a newer image's registration after the older image is revoked.
A permanent registration binding and a sealing policy admitting only that bound measurement make revocation contain the affected signing identities.
Same-image restarts retain sealed-key reuse; changing the image requires fresh key generation and registration for each certifier switching measurements.
Neither rule removes the existing trust in parties able to weaken the sealing policy, which L1 cannot inspect.

## Security Considerations

**Malformed compressed streams cannot advance past undelivered content.**
A batcher controls compressed channel bytes and can corrupt a late integrity trailer while retaining a decodable prefix.
If the core and derivation keep different prefixes, an accepted cursor can pass content derivation never receives.
Both MUST apply the same complete-stream validation before committing or releasing blocks; a failed stream contributes no blocks even when some output was available before the error.

**Assets and threat model.** This section protects two properties: that nothing advances the inbox's cursor unless the certifier core has checked it against the selected records, and that the core's verdict is a pure function of the data it is handed, so that an attestation proves what the core checked and nothing more.
The core trusts the image associated with its currently [authorized measurement](batch-inbox.md#certifier-key-registration), the TEE's execution and key protection, and the Nitro attestation that binds [`CERTIFIER_KEY`](#constants) to that image, together with, where a deployment seals that key, the sealing service and its release policy; it trusts [`INITIAL_QUORUM_HASH`](activation.md#constants) as the digest of the admin-approved Espresso anchor and, through the registry evidence the inbox rebuilds, the EigenDA operator set and thresholds it verifies the certificate against.
The certifier host, and the operator running it, are trusted for nothing except, where the key is sealed and the operator can change the release policy, that policy: every input the host assembles is authenticated against the cursor's anchor or against the batcher's own submission, and a wrong input yields no attestation the inbox accepts.
The batcher is untrusted, which is why the frames it presents are chained through `frames_digest` and compared with the selected records rather than believed.
The [Roles and Trust](overview.md#roles-and-trust) table records these premises; the paragraphs below state what each one costs when it fails.

**Input authentication depends on a trusted cursor anchor.**
Given a correct cursor and trusted quorum anchor, the core verifies Espresso finality relative to that quorum, namespace completeness against the authenticated header, and frame continuity against the cursor.
A compromised host cannot replace the anchored quorum merely by supplying different quorum bytes, but the hash comparison does not detect an untrusted anchor already installed by an incorrect override.
Signatures and namespace proofs relative to such an anchor do not by themselves establish genuine Espresso history.
Content authenticity therefore depends on the [pinned publisher key](block-records.md#authorized-publisher-keys), the [quorum trust premises](#espresso-verification), and the corresponding verification checks, rather than on the operator's honesty.

**A partial namespace payload would break canonicality.**
If a host could present part of an Espresso block's namespace payload and have it accepted, it could hide the record that first claimed a height and cause the core to select a later record instead, producing an attestation over content that contradicts what every other observer computes.
`verify_namespace` is what forecloses this, and it MUST be checked for every supplied block including those whose namespace payload is empty — an unverified claim of emptiness is exactly the suppression this prevents.

**Interleaved channels would advance the cursor past blocks that never settle.**
Without step 2's `channel_id` equality, frames belonging to two channels could be presented as one, and the core would decode a coherent block sequence from their concatenation and advance the cursor past it.
For example, a non-closing frame 0 from channel A followed by a closing frame 1 from channel B could pass the numbering and closure checks while supplying a complete byte stream to the core.
Holocene derivation immediately drops B's non-first frame because its channel ID differs, leaving A incomplete until it times out or a new first frame replaces it; the [channel bank](https://github.com/ethereum-optimism/specs/blob/42b446f5891dd5a232cfc346201a613c1bba3f50/specs/protocol/holocene/derivation.md#channel-bank) holds only one staging channel.
Once the record cursor has advanced past that content, later ordinary submissions cannot reinsert the skipped heights.
They may still continue from the advanced cursor if it otherwise admits valid attestations; loss of that content does not itself make the cursor unextendable.

**A quorum anchor ahead of the cursor's epoch can halt ordinary submissions.**
The output rule anchors `quorum_snapshot_hash` to the quorum governing `out.height`'s own epoch, not the furthest epoch inspected by the call.
If a faulty core instead retains an earlier `out.height` while writing a later epoch's quorum digest, and the inbox accepts that cursor, the next call cannot verify the block at the stored height: the earlier quorum fails the digest check, while resolution from the anchored later quorum cannot move backwards.
Crossing an epoch boundary alone does not cause this failure; the output position and quorum anchor must be inconsistent in this way.
Ordinary submissions cannot repair the accepted cursor because `prev_cursor` must equal the stored cursor.
An [emergency cursor override](batch-inbox.md#emergency-cursor-override) can restore progress if the admin supplies a correct recovery position satisfying the override's constraints, including a strictly greater `block_number`; [recovery S8](recovery.md#s8-a-previous-override-left-the-cursor-at-an-unreachable-position) describes the repair bounds and loss of skipped content.
The stored cursor is public on L1, and its mismatch can be diagnosed against authenticated Espresso epoch and quorum data.

**Selected content would otherwise be settleable at a height it was never selected for.**
`b.timestamp` is the only field in a batch that decides which L2 height derivation fills with that batch's content, so a core comparing transactions and L1 origin while leaving the timestamp free would place no constraint at all on where the content it certified ends up.
Suppose the transactions of record 101 are attested inside a batch whose timestamp corresponds to height 106: the core advances its cursor past record 101, and derivation, which under [Holocene](https://specs.optimism.io/protocol/holocene/derivation.html) drops a future batch together with the rest of its channel, never derives that content at all.
Record 101's genuinely selected content can then never settle, because the cursor has already passed it and no submission may re-attest a position behind the cursor.
A pipeline without Holocene's rule would instead hold the batch until height 106 and settle there content selected for a different height, which is precisely the third outcome the [Certification Claim](overview.md#the-certification-claim) rules out.
Omitting the check is also a source of cross-implementation divergence rather than merely a local weakness — an implementer who reads the [height-to-timestamp bijection](block-records.md#fields) will add it and one who reads only the comparison as written will not, and the two cores then accept different sets of submissions for the same Espresso history.

**Bounding `epoch_num` does not prove a fixed rescue margin.**
[Record Validity](block-records.md#record-validity) prevents selected-record origins from decreasing within a scan segment, and the inbox's [acceptance bound](batch-inbox.md#acceptance) prevents an accepted cursor from naming an origin beyond the containing L1 block.
Without the latter bound, a publisher could name a far-future origin and, if the submission were accepted, postpone alternative-batcher eligibility until that origin plus the exclusive window.
These checks constrain the accepted record's origin, not the origin derivation uses after substituting a deposit-only block.
The [exclusive window](batch-inbox.md#exclusive-window)'s fixed rescue margin additionally requires the cursor origin not to exceed the epoch governing the missing height's timeout; an accepted origin ahead of the derived chain can violate that premise.

**Batch-drop conditions that neither a record nor the core can be checked against are a residual exposure.**
The OP Stack baseline drops a batch outright, rather than rejecting its payload, when its `parent_hash` is not the hash of the canonical L2 parent, when its `epoch_hash` is not the hash of canonical L1 at `epoch_num`, when it is submitted outside the [sequencing window](https://specs.optimism.io/protocol/derivation.html#overview) for the height it claims, or when it violates the sequencer-drift rules.
These four baseline conditions are not the four conditions [Δ5, substitution instead of dropping](derivation-pipeline.md#delta-5-substitute-instead-of-dropping) converts, which overlap them only in the drift condition.
None of the four is checkable against a [block record](block-records.md#record-validity), which carries neither hash and says nothing about where or when a batch is submitted, and none is checkable by the core, which performs no execution and is supplied no authenticated L1 data.
The selected record fixes the transaction list, `epoch_num` and height-derived timestamp, but neither `parent_hash` nor `epoch_hash`.
Before the affected frames are accepted and while the inbox cursor still permits their height, a batcher can correct those hashes and obtain a fresh commitment and attestation without changing the selected record.
Once an accepted cursor transition has passed the height, ordinary submissions cannot reinsert it; this restriction comes from cursor progress, not record immutability.
Correcting batch-only hashes cannot change a record-fixed value or restore an expired sequencing window.
[Δ3](../rollup/derivation.md#batch-parent-hash) and [Δ4](../rollup/derivation.md#canonical-l1-origin-resolution) remove the two batch-hash conditions under the certified rules.
[Δ5](../rollup/derivation.md#same-height-deposit-only-substitution) converts the drift rejection conditions into same-height substitution once the required inputs are available; the sequencing-window condition stays, and so does the exposure this paragraph describes for it.
Channel-level conditions are handled elsewhere: the raw channel size bound by the core, which under [step 2](#verification-procedure) fails any call whose channel exceeds [`MAX_RLP_BYTES_PER_CHANNEL`](#constants) and decodes under the same [`CHANNEL_PARAMS`](#constants) as derivation, and the [channel timeout](https://specs.optimism.io/protocol/derivation.html#channel-bank) by the [inbox](batch-inbox.md#acceptance), which refuses a frame derivation would discard; neither is a residual exposure.
An L1 reorganization can make an honestly supplied `epoch_hash` stale, so the removed hash check could reject a batch without any party misbehaving.
Under [Δ4](../rollup/derivation.md#canonical-l1-origin-resolution), the canonical L1 block at `epoch_num` supplies all origin-dependent inputs instead.
This preserves canonical input selection, while relinquishing the unchanged batch's binding to its named L1 branch.
Execution can remain valid with different results or fail and enter the deposit-only escape hatch, as [recovery S3](recovery.md#s3-an-l1-reorganization) describes.

**Certificate verification is inside the enclave's trusted computing base.**
Under the split the [Rationale](#rationale) describes, L1 authenticates the operator set, the non-signer quorum memberships, the stakes and the thresholds the core used and the freshness of the reference block, but not the aggregate signature itself.
A core that skipped the signature check, or a compromised enclave, could therefore attest a certificate no operator quorum signed, and the inbox would accept it if the evidence hash matched; the availability of that payload would then rest on nothing.
This adds data availability to what [Attestation](#attestation) already places on the measurement's integrity — the premise under which the core's content checks are trusted — and adds no new party: an honest core rejects an unsigned or under-signed certificate exactly as EigenDA's verifier would, and the [full-node guarantee](full-node.md#conditions-under-which-it-holds) lists the premise.

**Replay isolation is limited to the signed deployment domain.**
The [attestation digest](#encodings) binds `DOMAIN_ATTESTATION`, MegaETH's `CHAIN_ID` and the inbox proxy address, so changing the chain identifier or proxy address changes the signed message.
Without those identifiers, two deployments could reuse an attestation when they admit the same signing key and reconstruct the same remaining signed fields; sharing a measurement alone does not establish either condition.
The domain does not separately name the L1 chain, implementation address or upgrade generation, so environments reusing the complete domain are not distinguished by it alone.
An implementation upgrade behind the [fixed proxy](batch-inbox.md#proxy-address-and-upgrade) does not automatically invalidate existing attestations when the domain and digest scheme remain unchanged.
An old attestation remains acceptable only if every current [acceptance check](batch-inbox.md#acceptance) passes, including signing-key and measurement eligibility, cursor continuity, certificate freshness and matching registry evidence.
Neither a public initial cursor nor a shared measurement implies that two deployments have identical cursors or satisfy those other checks.

**An aborting decoder is a permanent denial of service.**
The namespace is permissionless and its payloads are attacker-chosen, and the core's signature filter runs only after decoding.
A decoder that could fail on a crafted payload would abort the enclave on every call that scans the Espresso block containing it, and because the cursor only advances past that block once a channel closes — which requires completing the scan — the failure would recur on every retry indefinitely.
The core therefore MUST decode every namespace payload under the total decoding that [Well-Formedness](block-records.md#well-formedness) fixes, so that no payload can abort a call.

**Timely catch-up depends on verification capacity.**
Per-object decoding limits do not bound the aggregate history-verification work of a call: the core verifies contiguous Espresso history starting at `cursor.height`, including namespace data that yields no selected record.
Normal cursor transitions do not checkpoint scan progress independently of selected record progress, so a long gap can require substantial verification before the next advance.
Timely catch-up and submission depend on sufficient capacity to process that history within the applicable submission deadlines; the specification does not establish a fixed bound on that work.
This is a liveness limitation, not permission to omit history or weaken verification.

**The attestation is only as strong as the measurement it reports.**
If an image other than the authorized build could produce an accepted attestation, the certifier's entire verification could be skipped while its signature remained valid, and arbitrary content could be settled as though certified.
This places the AWS Nitro attestation infrastructure inside the trusted computing base alongside `INITIAL_QUORUM_HASH`, and makes reproducible builds load-bearing rather than good practice: without them no party can independently confirm that each authorized measurement corresponds to the specified core, and the permissionlessness of the certifier role is nominal.

**Compromise of `CERTIFIER_KEY` permits arbitrary settled content.**
An attacker able to sign as a registered certifier can attest any frame against any cursor, including content Espresso never finalized, and the L1 contract will accept it because the attestation is the only evidence it has.
This is the strongest single failure in the design, and it is bounded only by revocation — withdrawing authorization for the compromised measurement invalidates every key registered under it in one action, on the terms the [certifier key registration and revocation rules](batch-inbox.md#certifier-key-registration) specify.
It does not permit forging transactions, which remain signed by their senders, and it does not alter the records selected from Espresso's finalized history — only what settles.

**Key sealing adds the sealing service and its policy administrators to the trusted set.**
A [sealed](#key-sealing) key is protected by the sealing service's release policy as well as by the TEE.
That policy restricts recovery to the key's exact registered measurement, and [registration](batch-inbox.md#certifier-key-registration) prevents assigning the key to a different measurement.
These rules contain an old image's key when that measurement is withdrawn, even if an enclave already retains the key; they do not erase the key or make a weakened release policy observable on L1.
Any party able to change that policy — the service itself, or whoever administers the sealing key, typically the certifier operator's cloud account — can drop the measurement condition, have the key released to arbitrary code, and sign attestations the inbox accepts with no hardware attestation involved, which is the key compromise the preceding paragraph describes.
L1 cannot observe the release policy: registration establishes that the key was generated inside an authorized image, not how it has been stored since, so no contract check detects a weakened policy.
Because the inbox accepts every key registered under an authorized measurement, one registered key held under a weak policy exposes the whole chain, not only the deployment that sealed it; withdrawing that measurement remains the bound.

**Ignoring the batch's `parent_hash` prevents a batcher from forcing rejection through that field.**
Under the OP Stack baseline, an authorized batcher can submit otherwise correct selected content with a deliberately wrong `parent_hash`, causing the batch to be dropped even when no earlier height has lost its content.
An earlier deposit-only substitution can instead change the derived parent, so subsequent batches built against the original chain can fail the same comparison and propagate the loss.
The current core neither executes L2 nor receives an authenticated derived parent, so it cannot establish whether the batch's `parent_hash` matches that parent.
Under the certified rules, [Δ3](../rollup/derivation.md#batch-parent-hash) removes both causes of rejection by ignoring the batch field and deriving on the actual parent.
Actual parent-link and execution validation remain required; ignoring the batch field does not make execution valid after the parent state changes.

## Spec History

Block certification has not been activated on any network.
