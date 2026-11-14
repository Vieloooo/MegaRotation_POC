---
description: The L1 batch inbox contract — the batch commitment it accepts, the batcher registry and exclusive window that govern who may submit, and the acceptance rules that admit a batch to L1.
spec: TBD
---

# L1 Batch Inbox

{% hint style="warning" %}
This page specifies unstable behavior.
Block certification is not activated on any MegaETH network.
Parameters marked `TBD` remain unassigned.
Test vectors for the batch commitment calldata and the cursor-advance event topic will be added when the constants are assigned.
{% endhint %}

The **L1 batch inbox** is a proxy at the chain's fixed batch inbox address.
Its certification implementation accepts a batch only when the batch carries a valid [block certifier](certifier.md) attestation, references a real data availability certificate, and continues exactly where the previously accepted batch ended.
Certification begins at the scheduled [activation height](activation.md#activation-schedule); at and above it, [derivation](../rollup/derivation.md#batcher-transaction-selection) ingests a submission only if the inbox emitted its [acceptance event](#events), while earlier heights are derived under the [legacy rules](activation.md#derivation-transition).
This page specifies the commitment the contract accepts, the state it maintains, how certifier keys come to be registered against it, the [batcher registry](#batcher-registry) and [exclusive window](#exclusive-window) that determine who may submit, the acceptance rules themselves, and how an alternative batcher recovers settlement from a stalled primary.

## Motivation

A batcher's submission alone does not establish that its content matches the records selected from Espresso's finalized history.
The inbox gates acceptance on a certifier attestation and cursor continuity, giving derivation an on-chain acceptance signal for exactly the submissions checked by certification.

## Proxy Address and Upgrade

The batch inbox proxy address in the chain's rollup configuration MUST remain unchanged when certification is introduced.
The upgrade MUST replace the implementation behind that proxy; it MUST NOT introduce a different batch inbox address for certified submissions.
`batch_inbox_address` in the [attestation digest](certifier.md#encodings), the recipient checked by derivation, and the emitter matched for the acceptance event MUST all be the proxy address, not the implementation address.

The certification implementation MAY be installed before the scheduled activation height is reached.
Legacy-format submissions are handled under [Legacy-Format Calls](#legacy-format-calls).
The transition is specified by [Block Certification Activation](activation.md).

## Batch Commitment

Integer expressions and fixed-width fields on this page follow [Integer Semantics](block-records.md#integer-semantics), including deadline comparisons in [acceptance](#acceptance).

The calldata of a certified-format batch transaction is a **batch commitment**, bundling the [certifier's output](certifier.md#inputs-and-outputs) with the data availability certificate the certifier verified for the submission payload it attested:

```text
BatchCommitment {
    eigenda_cert:       bytes,    // the certificate the certifier verified, named by hash in certifier_sig
    eigenda_commitment: bytes,    // the attested commitment; covered by certifier_sig
    prev_cursor:        Cursor,   // the cursor the attestation started from
    cursor:             Cursor,   // the resulting cursor
    certifier_sig:      bytes65,  // over keccak256(DOMAIN_ATTESTATION || CHAIN_ID
                                  //   || batch_inbox_address || enc(prev_cursor)
                                  //   || eigenda_commitment || keccak256(eigenda_cert)
                                  //   || registry_evidence_hash || enc(cursor))
}
```

The digest that `certifier_sig` signs, the cursor encoding `enc` that the digest is composed from, and the registry-evidence digest it covers are all defined under the certifier's [attestation encodings](certifier.md#encodings); this contract recomputes that digest exactly as defined there and does not define a second form of it.
`prev_cursor`, `cursor`, `eigenda_commitment`, and `certifier_sig` are reproduced verbatim from the `CertifierOutput` that produced them, and `eigenda_cert` is the certificate that output was produced against: the submission payload is dispersed first, and the certifier verifies the certificate before signing.
The registry evidence itself is not carried; the contract rebuilds it from EigenDA's registries under [Acceptance](#acceptance), so `certifier_sig` verifies only if the evidence the certifier used is what those registries hold.
This is the shape an OP Stack [alt-DA](https://specs.optimism.io/experimental/alt-da.html) provider already returns to a batcher, so the batcher posts it without any knowledge of what it contains.
The calldata MUST be [`BATCH_TX_PREFIX`](#constants) followed by the Solidity ABI encoding of `BatchCommitment` as a single struct argument, in the field order declared above.
The encoding includes the offset word that introduces a dynamic tuple.
Each `Cursor` MUST be encoded as a tuple of its six fields in the order the [`Cursor` declaration](certifier.md#cursor) lists them, the same tuple shape the [cursor-advance event](#events) uses.
The prefix's second byte is [`CERTIFIED_COMMITMENT_TYPE`](#constants), allocated as `0x80` outside the OP Stack [reserved commitment-type range](https://specs.optimism.io/experimental/alt-da.html#input-commitment-submission).
It MUST be distinct from every commitment type the legacy provider's alt-DA commitments carry, so that the contract's legacy-call compatibility handling and a node's commitment decoder can tell the two formats apart at their second byte, before decoding either.
`eigenda_cert`'s own leading bytes, where the certificate format carries a version, belong to the certificate's encoding under [`EIGENDA_CERT_FORMAT`](certifier.md#constants) and are not part of this prefix.
Derivation selects its active protocol by the canonical [bootstrap transition](activation.md#derivation-transition), not by this prefix.
In this ABI encoding, `certifier_sig` MUST be represented as dynamic `bytes`; `bytes65` above constrains the decoded value's length rather than naming an ABI type.
For calldata beginning with the prefix, the contract MUST decode exactly that layout and MUST revert on calldata that does not decode to it or whose decoded `certifier_sig` is not exactly 65 bytes; calldata not beginning with the prefix is a legacy-format submission, handled as [Legacy-Format Calls](#legacy-format-calls) states.
For the decoded commitment `c`, the entire calldata MUST equal `BATCH_TX_PREFIX || abi.encode(c)`, where `||` is byte concatenation and `abi.encode(c)` is the standard Solidity ABI encoding of `c` as a single struct argument.
The contract MUST revert if that equality fails, including for noncanonical offsets, padding, or trailing bytes.
This equality defines the accepted byte strings; an implementation need not perform a re-encoding if it enforces the same condition.
How [derivation](../rollup/derivation.md) resolves accepted calldata to the dispersed frame bytes is specified under [Batch Commitment Resolution](../rollup/data-availability.md#batch-commitment-resolution) on the data availability page.

## Contract State

```text
activation:           ActivationState          // scheduling and phase, defined by the activation page
cursor:               Cursor                   // defined only after bootstrap acceptance
channel_open_block:   uint64                   // the L1 block that included the first frame of the open channel
channel_opener:       address                  // the sender whose submission opened the open channel
primary_batcher:      address                  // the address admitted at any time
alternative_batchers: set[address]             // the addresses admitted once the exclusive window expires
certifier_verifier:   CertifierVerifier        // registers certifier keys and answers whether a
                                               //   signature is an admissible attestation
admin:                address                  // the chain's existing OP Stack super admin
```

`cursor` records certification progress and the channel frame digest after [activation](activation.md#activation-state).
Its `(height, index)` MUST retain the unified [record position](block-records.md#record-position) supplied by the certifier, the span start of the last consumed record, for either a record batch or a record fragment; the acceptance interface requires no publication signatures, fragment assembly state or publisher attempt nonces beyond the certifier attestation.
Scheduling and phase state are defined on that page.
`channel_open_block` and `channel_opener` record the L1 block that included the first frame of the open channel and the address that submitted it, as read by the [channel timeout and ownership checks](#acceptance).
Before activation no certified channel is open; after activation `cursor.frames_digest` is zero exactly when none is.

`certifier_verifier` governs admission rather than content: the semantics it MUST provide, how keys come to be registered with it, and how they stop being admissible are specified under [Certifier Key Registration](#certifier-key-registration); whether it is embedded in this contract or is a separate contract this one calls is not specified.
`primary_batcher` and `alternative_batchers` are the [batcher registry](#batcher-registry), the addresses permitted to submit; derivation never reads them, because it selects on the [acceptance event](#events) instead.
These registry and key-registration values, together with the [authorized-measurement set](#certifier-key-registration), affect acceptance and event emission, and therefore which submissions derivation can ingest.

`admin` MUST be the super admin the chain's OP Stack contract set already designates, and MUST NOT be a role this specification introduces or a second authority held alongside it; the [L1 contract deltas](../rollup/l1-contracts.md) page is where that role set is owned.
It plays no part in ordinary operation — it does not sign or submit — and the operations it authorizes include [activation scheduling](activation.md#activation-schedule), curation of the [batcher registry](#batcher-registry) and the [emergency cursor override](#emergency-cursor-override).

The activation schedule, bootstrap authorization and initial cursor are specified exclusively by [Block Certification Activation](activation.md).

## Certifier Key Registration

The contract decides whether an attestation is admissible by asking its `certifier_verifier`, so what that verifier establishes — how keys come to be registered, and how they stop being admissible — is a normative rule rather than an operational detail.
This section specifies the verifier's interface and the semantics it MUST provide: the authorized-measurement set that registration checks against, the permissionless registration procedure itself, and revocation.
It does not specify where the verifier's state lives: the verifier MAY be embedded in the inbox contract or MAY be a separate contract the inbox calls, provided the semantics and the governance conditions below hold.

```text
CertifierVerifier {
    register(evidence: bytes)                            // records a certifier key from attestation evidence
    verify(signature: bytes, digest: bytes32) -> bool    // whether signature is an admissible attestation over digest
}
```

The set of authorized enclave measurements is held by the verifier, is curated by the chain's existing L1 super admin — the same authority that holds this contract's `admin` role — and [`CERTIFIER_PCR0`](certifier.md#constants) is its initial entry.
There MUST be exactly one authoritative measurement set: no acceptance decision may consult a second list, and an implementation keeping the set in the inbox, in a separate verifier contract, or in `SystemConfig` satisfies this section equally as long as that is the only set `verify` reads.
Governance therefore decides which enclave _images_ may certify, and never which parties may run them, which is the division the certifier's [attestation](certifier.md#attestation) rules require: authorization attaches to the measurement, not to the operator.

Registration MUST be permissionless.
Any party MAY register a certifier public key by calling `register` with evidence that an AWS Nitro attestation document binds it, and before recording the registration the verifier MUST:

- establish that the document's signature and its certificate chain up to [`NITRO_ROOT_CA`](#constants) verify, with every certificate in the chain valid at the document's own timestamp, and that the document satisfies the registration-time bound below;
- require that the measurement the document reports in `PCR0` is a member of the authorized-measurement set; and
- bind the registration to the public key carried in the signed document's `public_key` field, recording the 20-byte Ethereum address of that key against the measurement the document reported, so that `verify` can recover the signer and find its measurement.

The signed document's `public_key` MUST be present and contain exactly the 65-byte [SEC 1 uncompressed encoding](https://www.secg.org/sec1-v2.pdf) `0x04 || X || Y` of a finite secp256k1 public point, with each coordinate encoded as exactly 32 big-endian bytes, including any leading zeroes.
The verifier MUST reject any other length or prefix, coordinates outside the base field, and points not on secp256k1; it MUST NOT reduce out-of-range coordinates modulo the field size.
The registered address MUST be the last 20 bytes of `keccak256(X || Y)`, excluding the `0x04` prefix.
Both direct-document and proof-based registration MUST establish this decoding, point validity and address derivation for the same authenticated `public_key` field.

For each signer address, its first successful registration MUST permanently bind it to the authenticated measurement reported by that registration.
The verifier MUST reject evidence registering that address under any different measurement, even if both measurements are authorized, the new document is newer, or the original measurement has been withdrawn.
Such rejection MUST leave the existing binding unchanged.
Removal or reauthorization of a measurement MUST NOT delete or change any key's measurement binding; replacing or upgrading the verifier MUST preserve those bindings.
The verifier MAY accept renewed registration evidence for the same address and measurement, but MUST retain every ordinary registration check.
A measurement MAY have multiple distinct registered signer addresses; the restriction is one measurement per key, not one key per measurement.
An image upgrade to a different measurement MUST use a freshly generated key registered under that measurement, under [Key Sealing](certifier.md#key-sealing); it MUST NOT migrate an earlier image's key by re-registration.

Let `T` be the signed document's `timestamp` in milliseconds since Unix epoch, as defined by the [Nitro attestation document format](https://github.com/aws/aws-nitro-enclaves-nsm-api/blob/c7ba5f32febe0e9aaf4cc3636f9297aaa6800b42/docs/attestation_process.md#22-attestation-document-specification), `B` the containing L1 block's `block.timestamp` in seconds, and `A` [`ATTESTATION_MAX_AGE`](#constants) in seconds.
Registration MUST satisfy:

```text
0 <= 1000 * B - T <= 1000 * A
```

The verifier MUST evaluate this bound using mathematical integer arithmetic without overflow, underflow or timestamp rounding, including both endpoints.
A document newer than the containing L1 block's timestamp MUST be rejected; it MAY be resubmitted once L1 time catches up, subject to every registration check.
The verifier MUST NOT accept a key on any other basis.
The evidence MAY be the document itself, verified on L1, or a proof whose verification on L1 establishes the same facts.
Both evidence forms MUST establish the same registration-time bound using `T` authenticated by the same document that binds the registered key and measurement.
A zero-knowledge proof of the document's validation, whose public output carries the measurement, the public key and the document's timestamp, is the form the reference implementation uses.
Either way the facts are established by the verifier's own on-chain verification and never by trusting the submitter or an off-chain party.
In particular, it MUST NOT accept a key on the authority of whoever submits the registration, MUST NOT accept a key supplied alongside evidence that attests some other key, MUST NOT accept evidence whose certificate chain terminates anywhere other than [`NITRO_ROOT_CA`](#constants), and MUST NOT accept a document older than `ATTESTATION_MAX_AGE`.
The age bound is what keeps an old document from being replayed indefinitely once its short-lived signing certificate has lapsed; a document not submitted in time is regenerated rather than grandfathered, because an enclave of the same image can request a fresh document for its existing key, subject to the permanent key-to-measurement binding; an image upgrade uses a fresh key.
An alternative batcher's freshly launched enclave is therefore registrable by the alternative batcher itself, within one L1 transaction and without anyone's permission, which is what the [recovery path](#recovery-by-an-alternative-batcher) needs it to be.

Removing a measurement from the authorized set MUST invalidate every key registered under that measurement, in that one action, and MUST NOT require the affected keys to be enumerated.
This is why each registration records the measurement it was made under rather than the key alone, and why that binding MUST NOT be reassigned to another image.
Admissibility is evaluated at the moment of use as a join between the registered keys and the authorized set.
A key therefore becomes unusable the instant its measurement stops being authorized, however many keys that measurement produced and whether or not anyone knows what they are.
Reauthorizing the same measurement MUST restore the eligibility of its previously registered keys without requiring fresh registration or attestation evidence.
Restored key eligibility MUST NOT bypass any other [acceptance](#acceptance) check, including cursor continuity and certificate freshness.

A submission whose `certifier_sig` recovers to a key whose measurement is no longer authorized MUST be rejected.
In-flight submissions need no wind-down and get no grace: they fail the acceptance check exactly as any other inadmissible submission does, `cursor` does not move, and the content they carried remains selected under the unchanged cursor and resubmittable under any key registered against a measurement that is still authorized.
Revocation therefore costs liveness for as long as no authorized enclave is registered, and never costs safety.

The verifier is inside the governance trust boundary.
Whoever can change the authorized-measurement set, replace the verifier, or upgrade its implementation can admit an arbitrary signer.
Every one of those powers MUST therefore be held by the chain's existing L1 super admin and by no other party.
Replacing or upgrading a separate verifier contract carries the same authority requirement as upgrading the inbox implementation under [Proxy Address and Upgrade](#proxy-address-and-upgrade).

The governance trust boundary MUST also include every contract that directly or through delegation determines which registration proofs are accepted, which EigenDA certificate verifier applies, or which security thresholds and required quorums apply.
Each such decision MUST be immutable, controlled exclusively by the chain's existing L1 super admin, or explicitly identified as an external trust dependency together with its controller and the decisions that controller can change.
This requirement MUST apply to delegated proof-verification routes, attestation trust-root and certificate-revocation configuration, and certificate-verifier routing and parameter configuration, wherever those decisions are mutable.
Control of the top-level verifier alone MUST NOT be treated as exclusive control of acceptance when a delegated dependency is independently mutable.
External controllers of registration-proof acceptance are trusted against admitting an arbitrary signer; external controllers of certificate-verifier selection, security thresholds or required quorums are trusted against weakening data-availability acceptance.
EigenDA and EigenLayer operator-registry governance remains an external data-availability trust dependency: its controllers are trusted for the operator keys, stake histories and registry configuration used to authenticate certificate evidence, including the stale-stake switch described under [Security Considerations](#security-considerations).
The [Roles and Trust](overview.md#roles-and-trust) table MUST identify any additional external verification controllers used by the deployment and the acceptance decisions they control.

## Batcher Registry

The batcher registry names every L1 address permitted to send a batch transaction: `primary_batcher`, the one address admitted at any time, and `alternative_batchers`, the addresses admitted once the [exclusive window](#exclusive-window) has expired.
It is this contract's own state, and the [`admin`](#contract-state) MAY set `primary_batcher` and MAY add an address to or remove an address from `alternative_batchers`; the contract MUST reject those operations from any other sender.
Batch submitters MUST be externally owned accounts with empty account code; contract batchers and accounts carrying an [EIP-7702 delegation indicator](https://eips.ethereum.org/EIPS/eip-7702#delegation-indicator) are not supported.
When initializing or setting `primary_batcher`, or initializing or adding an alternative batcher, the contract MUST reject an address whose account code is non-empty.
The [acceptance checks](#acceptance) MUST recheck the sender's code at each submission; registration does not establish permanent eligibility.
`SystemConfig.batcherAddr` is unchanged and retains its historical authorization role for [legacy derivation](activation.md#derivation-transition); it plays no part in certified-submission acceptance or selection under the certified rules.

The registry is distinct from the enclave authorization specified under [Certifier Key Registration](#certifier-key-registration), which governs which _keys_ may sign an attestation rather than which _addresses_ may submit one; an alternative batcher needs standing in both, and neither implies the other.
Curation of the registry is deliberate rather than incidental: a registered address can trigger the frame-0 reset described under [Recovery by an Alternative Batcher](#recovery-by-an-alternative-batcher) below, so open enrolment would convert a recovery path into a griefing lever available to anyone.

The registry is read only by this contract, when it evaluates a submission.
[Certified derivation](../rollup/derivation.md#batcher-transaction-selection) does not read it or evaluate senders: it requires this contract's [acceptance event](#events), so a registry change needs no announcement to derivation.
Acceptance alone does not guarantee ingestion; the activation, recipient, transaction-type and downstream derivation rules still apply.
A membership change therefore takes effect for the submissions executed after it, and removal cannot be retroactive, because a submission already accepted has already emitted its event.

## Exclusive Window

Submission rights are measured in L1 blocks from [`cursor.epoch_num`](certifier.md#cursor), the L1 origin recorded in the accepted cursor.
This is the same axis [`seq_window_size`](https://specs.optimism.io/protocol/derivation.html#overview) already counts on, so the two deadlines are directly comparable rather than approximately related through an assumed block time.

| L1 height `h`                              | Addresses permitted to submit                  |
| ------------------------------------------ | ---------------------------------------------- |
| `h < cursor.epoch_num + EXCLUSIVE_WINDOW`  | The primary batcher only                       |
| `h >= cursor.epoch_num + EXCLUSIVE_WINDOW` | The primary batcher or any alternative batcher |

[`EXCLUSIVE_WINDOW`](#constants) MUST be expressed in L1 blocks and MUST be strictly less than `seq_window_size`.
For a missing L2 height `B`, let `e` be the L1 epoch whose sequencing-window expiry governs derivation's deposit-only fallback at that height.
If `cursor.epoch_num <= e`, exclusivity expires at least `seq_window_size - EXCLUSIVE_WINDOW` L1 blocks before that expiry.
[Record Validity](block-records.md#record-validity) makes selected-record origins non-decreasing within a scan segment, but does not establish this relation to the derived chain.
After a [deposit-only substitution](../rollup/derivation.md#same-height-deposit-only-substitution), the accepted record's origin can remain ahead of the derived origin; alternatives may then become eligible only after the missing height has been filled, as the [origin-divergence limitation](recovery.md#the-publisher-keeps-publishing) explains.
Even when the relation holds, the margin is an interval of eligibility, not a guarantee that registration, attestation, dispersal, and successful channel delivery finish before the deadline.

`EXCLUSIVE_WINDOW` MUST also exceed [`SETTLEMENT_LAG_BOUND`](#constants), the maximum number of L1 blocks by which `cursor.epoch_num` trails the L1 head while the whole pipeline is functioning normally.
The anchor is the L1 origin recorded at channel closure, so the gap includes the entire submission path rather than merely the interval between two channel closures.
`SETTLEMENT_LAG_BOUND` MUST bound the sum of five components: the origin lag the sequencer chose at production time, Espresso finality for the records covering that block, channel accumulation until the frame carrying the block closes its channel, EigenDA dispersal of the payload carrying that frame, and L1 inclusion of the batch transaction committing it.
Sizing the window against only one component does not establish that it exceeds the whole path's lag; a primary settling at the normal pipeline pace could then lose exclusivity before its next completed submission.

## Acceptance

A certified-format batch submission MUST be a transaction sent directly by a code-empty EOA authorized under the bootstrap or ordinary batcher rules to the fixed inbox proxy address.
Forwarded and internal-call submissions MUST be rejected; the proxy's own delegation to its implementation remains permitted.
The contract's fallback function decodes a [batch commitment](#batch-commitment) from its calldata and MUST apply the following checks, in this order.
`bootstrap_initial_cursor` and the activation phase are defined by [Bootstrap Acceptance](activation.md#bootstrap-acceptance).

```python
def fallback():
    if not msg.data.startswith(BATCH_TX_PREFIX):
        require not active
        return              # legacy-format submission before activation: no state or event
    require scheduled
    require msg.sender == tx.origin
    require extcodesize(msg.sender) == 0
    c = decode(msg.data)    # BATCH_TX_PREFIX || abi.encode(BatchCommitment)
    require len(c.certifier_sig) == 65
    require msg.data == concat(BATCH_TX_PREFIX, abi.encode(c))
    require blobhash(0) == 0    # no blob sidecar: derivation reads calldata alone

    # a channel open for more than CHANNEL_TIMEOUT blocks counts as closed here
    bootstrap = not active
    if bootstrap:
        expected = bootstrap_initial_cursor(c.prev_cursor, msg.sender,
                                            block.number, block.timestamp)
    else:
        expected = copy(cursor)
        if cursor.frames_digest != 0 \
                and block.number > channel_open_block + CHANNEL_TIMEOUT:
            expected.frames_digest = 0

    # cheapest checks first: a losing racer reverts here, before any verification
    require c.prev_cursor == expected
    require c.cursor.epoch_num <= block.number
    require expected.frames_digest == 0 \
         or msg.sender == channel_opener    # an open channel belongs to its opener

    if not bootstrap:
        require msg.sender == primary_batcher \
             or (block.number >= cursor.epoch_num + EXCLUSIVE_WINDOW \
                 and msg.sender in alternative_batchers)

    cert = decode_cert(c.eigenda_cert)          # under EIGENDA_CERT_FORMAT; revert on any other form
    require cert.reference_block < block.number \
         and block.number - cert.reference_block <= EIGENDA_REFERENCE_BLOCK_WINDOW
    verifier = EIGENDA_CERT_VERIFIER_ROUTER.getCertVerifierAt(cert.reference_block)
    require set(verifier.quorumNumbersRequired()) <= set(cert.blob_header_quorums)
    evidence = registry_evidence(cert, verifier)    # rebuilt from EigenDA's registries at cert.reference_block
    require certifier_verifier.verify(c.certifier_sig, keccak256(
        DOMAIN_ATTESTATION || CHAIN_ID || batch_inbox_address
        || enc(c.prev_cursor) || c.eigenda_commitment
        || keccak256(c.eigenda_cert) || registry_evidence_hash(evidence)
        || enc(c.cursor)))
    require commitment_of(cert) == c.eigenda_commitment

    if expected.frames_digest == 0:
        channel_open_block = block.number    # this submission opened a channel
        channel_opener = msg.sender
    if bootstrap:
        active = True
        emit CertificationActivated(expected)
    emit CursorAdvanced(expected, c.cursor)
    cursor = c.cursor
```

The contract MUST revert without modifying state when any check fails, and MUST emit the [acceptance event](#events) and set `cursor` to `c.cursor` when all of them pass.
Calldata not beginning with `BATCH_TX_PREFIX` is a [legacy-format submission](#legacy-format-calls), not a batch commitment, and is handled as that section states.
`primary_batcher` and `alternative_batchers` are the [batcher registry](#batcher-registry) as this contract holds it when the transaction executes.
The digest passed to the verifier is the attestation digest the certifier signed, recomputed from the submitted fields under the certifier's [attestation encodings](certifier.md#encodings); a contract that composed those fields any other way would reject every honest submission.

`extcodesize` reads the sender's account-code length at the time of submission, including any EIP-7702 delegation indicator.
The sender checks apply to batch submissions, not to administrative calls or permissionless certifier-key registration.
The existing registry and exclusive-window checks still authorize the sender; `tx.origin` equality does not replace them.

The bound on `c.cursor.epoch_num` rejects a cursor naming an L1 origin that has not occurred.

The channel timeout check mirrors the rule derivation's [channel bank](https://specs.optimism.io/protocol/derivation.html#channel-bank) applies: a channel whose first frame was included in L1 block `O` is discarded once derivation processes a block later than `O + CHANNEL_TIMEOUT`, and every frame of it that arrives afterwards is lost.
Once a channel has been open for more than [`CHANNEL_TIMEOUT`](#constants) blocks the contract treats it as closed: the continuity check then expects a `prev_cursor` whose `frames_digest` is zero.
A frame continuing the channel, whose `prev_cursor` carries the stale digest, is therefore refused, and a frame 0 built against the zeroed cursor is what continues settlement.
No field tells the contract which of the two a submission is.
A submission accepted against a zero `frames_digest` opens a channel — the first frame of a fresh channel, or the reset of [Recovery by an Alternative Batcher](#recovery-by-an-alternative-batcher) — and the contract records its including block and its sender.
While that channel is open and has not timed out, the contract MUST refuse a submission from any other sender: the channel belongs to its opener, which alone may continue it, and it stops being open only by closing or timing out, so no sender, its opener included, can open another channel before then.
A refused frame costs its sender gas and nothing else: `cursor` is unchanged, so the same content is resubmitted in a fresh channel by whoever opens one next, whereas a late frame accepted here would have advanced `cursor` past blocks derivation had already discarded, which no later submission could repair.
A complete channel can open and close in one acceptance whether its payload contains one frame or several; ownership protects channels spread across submissions from competing resets, as the [Security Considerations](#security-considerations) set out.

`blobhash(0) == 0` refuses a transaction that carries a blob sidecar, since `BLOBHASH` returns zero when the transaction has no blobs and the first versioned hash otherwise.
The contract checks calldata alone, but [derivation](../rollup/derivation.md#batcher-transaction-selection) excludes blob-carrying transactions and inherits a baseline that would otherwise read their blobs in place of their calldata.
A blob-carrying submission the contract accepted would therefore advance `cursor` past a frame derivation never ingests.
That is the divergence between acceptance and ingestion the [Security Considerations](#security-considerations) describe, whose lost content no later submission can reinsert.
Refusing it here keeps acceptance and ingestion aligned for the transaction type as the event keeps them aligned for the transaction itself.

`certifier_verifier.verify` MUST establish that `certifier_sig` has the form the certifier's [attestation encodings](certifier.md#encodings) fix: `r || s || v` with `v` in {27, 28} and `s` at most half the curve order.
It MUST establish that `certifier_sig` recovers to a registered address whose recorded measurement is still a member of the authorized-measurement set, per [Certifier Key Registration](#certifier-key-registration).
It MUST NOT require any particular key.
The contract identifies the signing key and its registered measurement, without requiring a particular enclave operator.
An alternative batcher's freshly registered enclave is therefore accepted under the same signature and measurement checks as the primary's.

The contract does not verify the certificate's aggregate signature; the [certifier core](certifier.md#verification-procedure) did, and the attestation names both the certificate, by `keccak256(c.eigenda_cert)`, and the operator-set data the core verified it against, by `registry_evidence_hash(evidence)`.
`verifier` is the EigenDA certificate verifier that [`EIGENDA_CERT_VERIFIER_ROUTER`](#constants) names for `cert.reference_block` through its [`getCertVerifierAt`](https://github.com/Layr-Labs/eigenda/blob/bfa451e1aaa5eb53ec250758054bbc3267e66208/contracts/src/integrations/cert/router/EigenDACertVerifierRouter.sol), resolved when the transaction executes; its security thresholds and required quorums govern every certificate whose reference block it is named for.
The contract MUST require that the quorum numbers the certificate's blob header declares include every quorum in `verifier.quorumNumbersRequired()`, reverting otherwise.
This is the required-quorum half of EigenDA's `required ⊆ blob ⊆ confirmed` rule; the core checks the other half, that every blob quorum is among the signed quorums, under [`verify_cert`](certifier.md#verification-procedure).
`registry_evidence(cert, verifier)` MUST reconstruct the [`RegistryEvidence`](certifier.md#inputs-and-outputs) values authenticated by EigenDA's [pinned registry-checking procedure](https://github.com/Layr-Labs/eigenlayer-middleware/blob/2f7c93e38f56f292f247981a52bd3619a16b9918/src/BLSSignatureChecker.sol#L88), using the certificate's registry indices at `cert.reference_block`.
The signed EigenDA quorum numbers MUST be non-empty, strictly increasing and valid under the pinned EigenDA profile.
When the pinned signature checker's `staleStakesForbidden` flag, read when the transaction executes, is true, the contract MUST require for each signed quorum that the registry coordinator's `quorumUpdateBlockNumber` for that quorum plus the delegation manager's `minWithdrawalDelayBlocks()` exceed `cert.reference_block`, and MUST revert otherwise, as the [pinned checker's stale-stake rule](https://github.com/Layr-Labs/eigenlayer-middleware/blob/2f7c93e38f56f292f247981a52bd3619a16b9918/src/BLSSignatureChecker.sol#L180-L192) specifies.
The non-signers MUST be strictly increasing by the operator IDs derived from their public keys as [certificate verification](certifier.md#verification-procedure) specifies.
For each non-signer, the contract MUST authenticate its historical quorum-membership bitmap against the registry coordinator at `cert.reference_block`, using that operator ID and the corresponding bitmap index, and include it in `non_signer_quorum_bitmaps` in certificate order.
For each signed quorum, in ascending quorum number, the contract MUST authenticate the supplied quorum G1 aggregate public key against the registry's historical APK hash, read its historical total stake, and include `verifier.securityThresholds().confirmationThreshold` as that quorum's `confirmation_threshold`.
It MUST build that quorum's `non_signer_stakes` in certificate non-signer order, reading the historical stake only for non-signers whose authenticated membership bitmap contains that quorum and inserting zero for every other non-signer.
The certificate's per-quorum non-signer stake indices apply to the membership-filtered subsequence, in that same order, as the pinned checker specifies; non-members consume no stake index.
`registry_evidence(cert, verifier)` MUST revert if any required index does not resolve for the specified operator or quorum at that block, an APK hash does not match, or any required registry validation fails.
The membership bitmaps are covered by the same `registry_evidence_hash` as the keys and stakes; they MUST NOT be trusted merely because the submission or host supplied them.
Because the digest the signature is checked against includes `registry_evidence_hash(evidence)`, evidence the core was given that differs from what the registries hold makes the signature fail; no field of the calldata carries the evidence, and the contract MUST NOT take it from the submission.
`decode_cert` parses the certificate under [`EIGENDA_CERT_FORMAT`](certifier.md#constants), the same layout the core accepts, and the freshness bound on `cert.reference_block` is the contract's for the same reason the origin bound is: only L1 knows `block.number`.
The commitment comparison remains a separate check: the core verified that the certificate's commitment equals the attested one, and the contract re-establishes it over the bytes it received, because derivation fetches by the certificate and reads by the commitment, and the two MUST name the same blob.

### Legacy-Format Calls

A **legacy-format submission** is a call whose calldata does not begin with [`BATCH_TX_PREFIX`](#constants).
Before activation, a legacy-format submission MUST succeed without modifying state and without emitting the [acceptance event](#events).
After activation, the contract MUST revert legacy-format submissions.
This compatibility behavior permits gas estimation by legacy batchers; after bootstrap, derivation also excludes later legacy inputs rather than relying on their receipt status.
Under the [legacy rules](activation.md#derivation-transition), derivation evaluates their included data regardless of receipt status.

## Events

Scheduling and bootstrap events are defined exclusively under [Activation Events](activation.md#events).

For every successful [certifier-key registration](#certifier-key-registration), including renewed registration of the same signer and measurement, the contract recording the registration MUST emit an event identifying the signer address and its bound measurement.
For every change to the authorized-measurement set, the contract recording the change MUST emit an event identifying the measurement and whether it was authorized or withdrawn.
For every replacement of `certifier_verifier`, the contract recording the replacement MUST emit an event identifying the previous and replacement verifier addresses.
A reverted operation MUST leave no event from that operation in its receipt.
The names and ABI layouts of these audit events are implementation-defined; their required information MUST be present in the emitted logs.
These events MUST NOT activate certification or advance the cursor, and derivation MUST NOT use them as batch-acceptance signals or cursor-reset inputs.
An implementation MUST retain the independently specified activation and cursor events for those effects.

The contract MUST emit `CursorAdvanced(Cursor prev, Cursor next)` on every submission the [acceptance](#acceptance) rules admit, where `prev` is the `cursor` the submission chained from and `next` is the `cursor` the contract stored.
It MUST emit `CursorSet(Cursor prev, Cursor next)` on every [emergency cursor override](#emergency-cursor-override), where `prev` is the `cursor` replaced and `next` the one the admin supplied.
When the previous channel had timed out, the emitted `prev` carries `frames_digest` zero, the value the submission was checked against, rather than the stale digest the contract stored.
This event is the acceptance signal [derivation](../rollup/derivation.md#batcher-transaction-selection) selects batch transactions by, so its presence in a receipt is consensus-critical: a transaction to this contract is derived from only if its receipt carries this event from this address; the remaining selection conditions are derivation's own.
`CursorAdvanced` MUST be emitted exactly once per accepted submission and only by that path: the contract MUST NOT emit it from the override, from [certifier key registration](#certifier-key-registration) or revocation, or from any change to the [batcher registry](#batcher-registry).
`CursorSet` is an ordered derivation control input under [Emergency Cursor Overrides](../rollup/derivation.md#emergency-cursor-overrides), not a batch acceptance signal: derivation MUST apply its gap-fill and channel-reset effects, and MUST NOT ingest the override transaction's calldata as batch content.
An observer reconstructing [selection and canonicality](block-records.md#canonicality) from L1 MUST use this event to locate the new resume point.
Only a `CursorAdvanced` that increases `block_number` makes the selected records in that advance canonical; `CursorSet` and submissions leaving `block_number` unchanged make no additional record canonical.
A closing submission that yields no complete block leaves `cursor` unchanged and may be accepted, and emit `CursorAdvanced`, more than once while its `prev_cursor` still matches.
A submission that reverts emits nothing, because the revert discards the log along with the state change.

The event's full signature is:

```text
CursorAdvanced((uint64,uint32,uint64,uint64,bytes32,bytes32),(uint64,uint32,uint64,uint64,bytes32,bytes32))
```

Each tuple contains the six fields in the order the certifier's [Cursor](certifier.md#cursor) declares them — `height`, `index`, `block_number`, `epoch_num`, `frames_digest`, `quorum_snapshot_hash` — with the types shown above.
The field order and types MUST match the declaration above.
Neither parameter is `indexed`; both cursors are ABI-encoded in the log's data.
`topic0` is the keccak of the [canonical event signature](https://docs.soliditylang.org/en/latest/abi-spec.html#events), which contains the event name and parameter types, not field names.
Reordering fields of the same type leaves that signature and `topic0` unchanged while changing which field each encoded value represents.
`CursorSet` has the same tuple shape — `CursorSet((uint64,uint32,uint64,uint64,bytes32,bytes32),(uint64,uint32,uint64,uint64,bytes32,bytes32))` — and its `topic0` is likewise the keccak of that string.

## Emergency Cursor Override

The [`admin`](#contract-state), and only the `admin`, MAY set the contract's `cursor` directly, bypassing every check the [acceptance](#acceptance) rules apply.
No submission is involved: the operation carries no batch commitment, no attestation, and no data availability certificate, only the cursor fields the admin supplies.
The contract MUST reject the operation before activation or from any other sender, and MUST NOT provide any path by which `cursor` is written other than an accepted submission or this override.

The supplied cursor MUST satisfy three conditions, and the contract MUST revert without modifying state when any of them fails:

- **The override advances or it does nothing.**
  The supplied `block_number` MUST be strictly greater than the stored `cursor.block_number`.
  The override MUST NOT be able to move the cursor backwards, and MUST NOT be able to leave it where it is.
  At the maximum value of `Cursor.block_number`, no representable target satisfies this rule; [recovery S8](recovery.md#s8-a-previous-override-left-the-cursor-at-an-unreachable-position) specifies the resulting repair limitation.
- **No open channel survives an override.**
  The supplied `frames_digest` MUST be `0`, the value `rolling_hash([])` takes under the certifier's [attestation encodings](certifier.md#encodings).
  Any channel open at the moment of the override is abandoned, so the next submission begins with frame 0 of a channel built from the new position — the same reset an alternative batcher's frame-0 submission relies on under [Recovery by an Alternative Batcher](#recovery-by-an-alternative-batcher), rather than a second mechanism introduced for the override.
- **The origin must already exist.**
  The supplied `epoch_num` MUST satisfy `epoch_num <= block.number`, the same bound [acceptance](#acceptance) applies to a submitted cursor.
  An override therefore cannot park the [exclusive window](#exclusive-window)'s anchor at an L1 origin that has not occurred, which would suspend the recovery path the window exists to open.

The override performs no other validation of the supplied cursor.
It MUST emit [`CursorSet`](#events) with the replaced and the supplied cursor, so that every observer can locate the resume point on L1.
Derivation MUST process that event under [Emergency Cursor Overrides](../rollup/derivation.md#emergency-cursor-overrides), filling missing heights through the supplied `block_number` without replacing the established safe prefix or independently conferring finality.
The `admin` MUST therefore supply values consistent with the selected record sequence:

- a `(height, index)` chosen under one of two cases, as [Sync Start](block-records.md#sync-start) distinguishes them: if it is the index-0 fragment of an attempt for `block_number`, that attempt MUST complete and its record MUST pass the [Record Selector](block-records.md#record-selector) with `prev_epoch = epoch_num`; otherwise, it MUST be a position from which the scan finds the publisher's record for `block_number + 1` first;
- an `epoch_num` that the next record's origin does not go below, nor, in the first case, the re-selected record's, since `epoch_num` is the preceding origin for [Record Validity](block-records.md#record-validity)'s continuity condition; and
- a `quorum_snapshot_hash` that is the digest of a quorum reachable from the scheduled trusted quorum under [Espresso Verification](certifier.md#espresso-verification) and governing `height`'s epoch or an earlier one.

The obligation is the operator's, because the contract cannot hold it to any of these.
An override that violates the first case or the origin obligation stalls certification: in the first case the scan yields no record until it re-selects a passing record for `block_number`, and in either case it selects no next record whose origin lies below `epoch_num`; it does not select content from the wrong position.
An override that violates the second case by naming a position from which the scan finds a different authenticated record for `block_number + 1` first is not detected, as [Security Considerations](#security-considerations) describe.
The stall lasts until the scan reaches records satisfying those conditions or a corrected override, and because every override must advance `block_number`, a correction costs at least one more skipped height.
These obligations are premises of the [Certification Claim](overview.md#the-certification-claim); advancing `block_number` alone does not preserve canonicality or establish the quorum anchor's validity.
The contract cannot verify that the supplied `height` and `index` correspond to a real [record position](block-records.md#record-position), nor that the record at that position is the one carrying `block_number`, because it cannot read Espresso — establishing either would require exactly the Espresso consensus verification the [certifier](certifier.md) exists to keep off L1.
The supplied `quorum_snapshot_hash` is not authenticated by the override.
A trusted quorum from an earlier epoch remains usable when valid `quorum_steps` advance it to the epoch at `height`; if matching quorum bytes or the required transitions cannot be supplied, verification cannot proceed.
An untrusted quorum is a different failure: matching its digest proves consistency with the stored cursor, not authenticity against Espresso history, and proofs signed by that quorum can pass its signature checks without establishing real Espresso finality.
An incorrect anchor can therefore compromise safety rather than merely halt progress; the guarantee relies on the admin preserving the [quorum trust premise](certifier.md#espresso-verification).
This override accepts the admin's recovery position without a proof establishing those obligations.
Its correctness is therefore a governance trust premise, in addition to the three checks the contract enforces.

## Recovery by an Alternative Batcher

Once the window has expired, an alternative batcher assembles a submission entirely from public data.
The cursor is public on L1; the Espresso blocks, proofs, and quorum evidence are public on Espresso; and the frames are built from the [selected record sequence](block-records.md#canonical-record-sequence) rather than reproduced from anything the primary holds.
No rule of this specification requires an alternative batcher to reproduce the primary's channel packing, and none compares a submission against a reference packing — a channel is checked against the selected records, never for conformity to what some other party would have built.
A batcher SHOULD submit a complete channel in one submission when it fits the pinned channel and frame limits, especially during recovery; multi-frame channels remain permitted.

When the primary stalled part-way through a channel, `cursor.frames_digest` is non-zero and the stalled channel cannot be continued: a continuation frame would have to extend the compressed stream the earlier frames began, and the compressor state those frames left behind exists only in the primary's process.
The frames themselves are on EigenDA, but they are not enough.
The alternative batcher MUST instead abandon that channel and repack, which requires no new mechanism:

- It MUST read the current on-chain `cursor` and use it as `prev_cursor` with `frames_digest` set to zero, the value the [acceptance](#acceptance) check expects once the stalled channel has timed out; until then the channel belongs to its opener and no party, its opener included, can reset it.
- It MUST submit a [payload](certifier.md#encodings) beginning with frame 0 of a channel it built itself, whose first L2 block is `cursor.block_number + 1`.

Built against a zeroed cursor, the reset needs no exception in the certifier at all.
The [verification procedure](certifier.md#verification-procedure) accepts an empty prefix witness only for a submission beginning at frame 0 against a zero `frames_digest`, which is exactly the zeroed cursor the inbox checks once the stalled channel has timed out.
The [cursor advance](certifier.md#advancing-the-cursor) resumes from `cursor.block_number`, which intermediate frames never moved.
The reset also restarts the [channel timeout](#acceptance): the frame-0 submission records its own including block as the new channel's opening block.

The stalled channel's already-dispersed frames are orphaned.
Derivation sees an incomplete channel, discards it when the next channel's first frame arrives, or at its timeout, exactly as Holocene's channel bank always has, and derives the alternative batcher's channel instead.

## Constants

| Constant                         | Value    | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| -------------------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `EXCLUSIVE_WINDOW`               | TBD      | Length in L1 blocks of the primary batcher's exclusive submission window, measured from `cursor.epoch_num`. Greater than `SETTLEMENT_LAG_BOUND`, and strictly less than `seq_window_size`.                                                                                                                                                                                                                                                                         |
| `SETTLEMENT_LAG_BOUND`           | TBD      | Maximum number of L1 blocks by which `cursor.epoch_num` trails the L1 head under normal operation, bounding the sum of origin lag, Espresso finality, channel accumulation, EigenDA dispersal, and L1 inclusion of the batch.                                                                                                                                                                                                                                      |
| `CHANNEL_TIMEOUT`                | 50       | The channel timeout derivation's channel bank applies, in L1 blocks: OP Stack's `ChannelTimeoutGranite`, which replaces the rollup configuration's `channel_timeout` from Granite and so applies at every certified height under the [Isthmus activation premise](overview.md#specifications). A frame continuing an open channel beyond it is refused, per [Acceptance](#acceptance). MUST equal the timeout derivation applies, per the [Rationale](#rationale). |
| `NITRO_ROOT_CA`                  | TBD      | The AWS Nitro Enclaves Root CA certificate at which every attestation document's chain MUST terminate, per [Certifier Key Registration](#certifier-key-registration).                                                                                                                                                                                                                                                                                              |
| `ATTESTATION_MAX_AGE`            | TBD      | Maximum age, in seconds, of an attestation document at registration. [Certifier Key Registration](#certifier-key-registration) defines the exact millisecond comparison and rejects future document timestamps.                                                                                                                                                                                                                                                    |
| `BATCH_TX_PREFIX`                | `0x0180` | The two bytes preceding the ABI-encoded [batch commitment](#batch-commitment): the OP Stack alt-DA derivation-version byte `0x01` followed by `CERTIFIED_COMMITMENT_TYPE`.                                                                                                                                                                                                                                                                                         |
| `CERTIFIED_COMMITMENT_TYPE`      | `0x80`   | The commitment-type byte allocated to the certified format, outside the OP Stack reserved range `0x00 .. 0x7f`; it MUST differ from every type used by the legacy provider, per [Batch Commitment](#batch-commitment).                                                                                                                                                                                                                                             |
| `EIGENDA_REFERENCE_BLOCK_WINDOW` | TBD      | Maximum age, in L1 blocks, of a certificate's reference block at acceptance: `block.number - cert.reference_block` MUST NOT exceed it, per [Acceptance](#acceptance).                                                                                                                                                                                                                                                                                              |
| `EIGENDA_CERT_VERIFIER_ROUTER`   | TBD      | L1 address of the EigenDA certificate verifier router whose `getCertVerifierAt` names, for a reference block, the certificate verifier whose security thresholds and required quorums govern certificates at that block, per [Acceptance](#acceptance).                                                                                                                                                                                                            |

## Rationale

**Why more than one address may submit.**
A gate that only one party can operate is a gate that party can close.
Were the inbox to accept batches from a single address carrying attestations from a single enclave, both halves of the admission check would sit with the sequencer.
A sequencer that simply stopped submitting — no crash, no invalid submission, nothing any proof could identify as misbehavior — would then starve settlement for as long as it chose.
Every subsequent L2 height would fall to the [deposit-only fallback](https://specs.optimism.io/protocol/derivation.html#batch-queue), the empty batch derivation produces when a height's sequencing window elapses.
That is a narrower failure than a sequencer that stops producing blocks, and a more troubling one.
When production stops there is nothing for anyone to settle.
When only settlement stops, every input needed to produce a correct batch is already published on Espresso in plain view, and the chain degrades anyway, purely because no one else is permitted to act on them.
Admitting more than one submitter closes that gap; doing so without making the normal path nondeterministic or handing a griefing lever to whoever asks for one is what the [exclusive window](#exclusive-window) and the [registry](#batcher-registry)'s curation answer, and the paragraphs below give the terms of that trade.

**Why the certified format has its own commitment type rather than a third prefix byte.**
The OP Stack alt-DA calldata format is a derivation-version byte, a commitment-type byte, and the commitment, and the legacy provider's commitments use the generic type with the provider's own layer byte inside the commitment.
A distinct commitment type distinguishes the two formats at the second byte, before either is decoded, which is what the contract's legacy-call handling and a node's commitment decoder need; a third byte inside the generic type would leave the first two bytes identical and require decoding, or trial decoding, to tell them apart.
The type value is allocated once and shared by the batcher, the certifier's provider interface, the contract, and the node, and takes effect at the same L2-height boundary as the rest of the certified format.

**Why the contract does not call EigenDA's certificate verifier.**
The certifier core verifies the certificate, and the attestation names it and the operator-set data used, for the reasons the [certifier page](certifier.md#rationale) gives; the contract's part is a handful of registry reads and a freshness check in place of a pairing.

**Why the EigenDA thresholds are read through the router rather than pinned.**
Upstream EigenDA fixes the confirmation threshold and the required quorums in each certificate verifier, and changes them only by adding a new verifier to the router at a future activation block.
A copy pinned in this specification would drift after any such change, either refusing honest certificates that meet the new parameters or accepting certificates the rest of the EigenDA integration treats as under-secured.
Resolving the verifier at `cert.reference_block` judges every certificate by the parameters in force for its own reference block, whichever side of a change it falls on.
The core still checks the threshold, but against a value the inbox reads from that verifier and authenticates through `registry_evidence_hash`, so a host cannot lower it.
The required-quorum check sits in the inbox rather than the core because the core has no authenticated source for the required quorums, and `RegistryEvidence` carries none.
The cost is that the router's owner becomes a trusted role, as the [Security Considerations](#security-considerations) state.

**Why cursor continuity replaces the sender restriction as the safety mechanism.**
The original single-sender check was doing two jobs: keeping derivation's ingestion aligned with the contract's acceptance, and preventing replay of a commitment lifted from the mempool.
Cursor continuity rejects submissions whose `prev_cursor` does not match the cursor expected by [acceptance](#acceptance), while the [acceptance event](#events) identifies the frames derivation ingests.
A closing-frame attestation that strictly advances `block_number` cannot be replayed after acceptance: `block_number` never decreases, so its earlier `prev_cursor` can no longer match.
Closing a channel does not itself imply that advance: the core permits [frames yielding no complete decoded L2 block](certifier.md#verification-procedure), which can leave the entire cursor unchanged.
Such an attestation may be accepted repeatedly while its `prev_cursor` still matches and the sender, signature and certificate continue to satisfy the other acceptance checks; cursor continuity is not a blanket duplicate-rejection rule.
An intermediate-frame attestation can be replayed by a registered sender if the cursor returns to its `prev_cursor`, which frame-0 resets can bring about.
The effect is bounded by channel ownership under [acceptance](#acceptance): a replayed intermediate frame extends only a channel its sender opened, a replayed frame 0 merely opens a channel its sender then owns, and either costs the sender gas and nothing more.
Identity was never what made a submission safe; the attestation and the cursor chain were, which is why widening who may submit changes nothing about what may be submitted.

**Why the continuity check runs before the verifications.**
Once an accepted submission strictly advances `block_number`, competing submissions built against the earlier cursor fail the continuity check; a submission that leaves the cursor unchanged need not invalidate its competitors.
The cost of a failed competing submission affects whether competing is rational at all.
Placing the cursor comparison first makes a loser pay the transaction base cost, its calldata, and one storage read, rather than the registry reads and signature recovery that dominate the function's cost.
The remaining unrecovered cost is the loser's EigenDA dispersal, which is bounded and self-limiting: because under an honest EigenDA quorum the certificate's blob commitment, which `eigenda_commitment` hashes, is a function of the submission payload bytes, duplicate dispersals of identical content converge on the same commitment, so the waste is duplicated fees rather than divergent state.

**Why the registry lives in this contract and derivation reads the acceptance event.**
The registry's hard requirement is not where it is stored but that derivation ingest exactly what the contract accepted, because content the cursor passes without derivation ingesting it can never be reinserted by a later submission.
Two formulations satisfy it: derivation re-derives membership from a registry it tracks and applies the sender rule itself, or derivation keys on the contract's own acceptance event and never evaluates senders.
The first needs the registry to live somewhere derivation already observes in transaction order — which means widening `SystemConfig` — and a transaction-order rule so that a membership change and a submission in the same L1 block are judged identically on both sides.
It also leaves derivation defining "sender" for itself, which the contract's `msg.sender` need not equal.
The second removes that registry dependency: the contract alone evaluates the registry, and certified derivation uses its acceptance event as evidence of authorization while retaining its other selection and validity checks.
`SystemConfig`'s batcher fields are untouched, and legacy selection retains its historical sender check.
Its cost is that the event's ABI becomes consensus-critical, which is why [Events](#events) pins the signature, and that derivation depends on one contract's logs rather than on transaction senders alone — a departure from the baseline's shape that is paid once, in one place.

**Why the attestation document is verified on-chain rather than by governance.**
Having governance register keys after inspecting documents off-chain would be far cheaper on-chain, and it would collapse the design: the certifier would become an allowlist of parties governance has approved rather than a role any party can occupy, which is the property the [recovery path](#recovery-by-an-alternative-batcher) rests on.
If an alternative batcher launches a fresh enclave after the primary stalls, registration and subsequent submission must fit within the affected heights' remaining sequencing windows.
Off-chain governance approval would add delay to that recovery path; on-chain verification removes that approval dependency without guaranteeing timely completion.
The cost of the chosen path is that L1 must verify the attestation — either COSE signature verification and X.509 chain validation to [`NITRO_ROOT_CA`](#constants) performed directly, or a proof of that validation, which is what makes the cost affordable — and it is paid once per registration rather than per submission.

**Why an exclusive window at all, rather than open competition from the start.**
The waste avoided is minor and is not the reason.
The window keeps the normal path deterministic — exactly one party submits, so no honest submitter's in-flight work is invalidated by another's — and it denies a registered but hostile submitter the ability to repeatedly open channels ahead of the primary and let them time out, stalling the primary for a `CHANNEL_TIMEOUT` each time.
Both concerns apply only while the primary is functioning, and both are acceptable during the recovery the window has already been forfeited to.
That the window is actually in force throughout that period is not automatic, however: because the anchor is `cursor.epoch_num`, a primary that is merely slow drags the anchor back with it, so an undersized window expires precisely while the primary is still working normally.
Sizing the window above `SETTLEMENT_LAG_BOUND` is what makes "while the primary is functioning" and "while the window is in force" the same interval rather than two intervals that happen to overlap.

**Why the window is measured in L1 blocks from the cursor's epoch rather than from a stored timestamp.**
A stored timestamp would be additional state, would need to be written on every acceptance, and would measure elapsed time on an axis unrelated to the one `seq_window_size` uses — so the relationship between the two deadlines would depend on an assumed L1 block time.
`epoch_num` is already carried in the cursor for free, is already covered by the attestation, and puts both deadlines on the same axis; the fixed recovery margin additionally requires the origin relation stated under [Exclusive Window](#exclusive-window).

No timestamp is stored and no timer is started.
The anchor rides inside the `cursor` the contract already writes on every acceptance, so the window costs no additional state.
Its integrity is the joint product of three mechanisms, and no one of them is sufficient.
`epoch_num` is part of `Cursor` and `certifier_sig` covers the whole cursor, which prevents the _batcher_ from choosing the anchor it posts.
[Record Validity](block-records.md#record-validity)'s continuity condition prevents the _publisher_ from moving the anchor backwards, because a record whose origin decreases is never selected.
This contract's bound of `cursor.epoch_num` against `block.number` prevents the publisher from naming an origin that does not exist yet.
The batcher and the publisher are the same operator, so binding the anchor to an attestation is by itself a constraint on transport and not on the anchor's value.

**Why channel packing is not constrained.**
Which L2 blocks a channel contains depends on when its producer chose to close it — wall-clock timing, configured channel duration, the compression ratio achieved so far — so two honest submitters running identical code at different moments produce different, equally valid channels.
A public deterministic packing policy could be specified, but leaving boundaries free lets an alternative batcher construct channels without reproducing another producer's timing or configuration.
Outcome equivalence requires the same ordered decoded batches and derivation inputs, with valid channels successfully delivered within the applicable deadlines; packing freedom alone does not establish those conditions.
Waiting to close a larger channel can miss a sequencing deadline that an earlier submission of a smaller channel could meet.
[Δ5, substitution instead of dropping](derivation-pipeline.md#delta-5-substitute-instead-of-dropping) and [Δ6, keeping the channel after an engine rejection](derivation-pipeline.md#delta-6-keep-the-channel-after-an-engine-rejection) preserve the remaining batches for the failure paths they change, not for every cause of channel discard.
A corollary worth stating because it is easy to assume otherwise is that compression determinism is not load-bearing — nothing compares one party's compressed bytes against another's, and `CHANNEL_PARAMS` is pinned so that derivation can decode the result and frames respect size limits, not so that two parties produce identical bytes.

**Why prefer a complete channel per submission.**
Batchers should prefer submitting a complete channel in one submission when it fits the pinned [channel, frame and DA payload limits](certifier.md#constants), especially during recovery.
This means a payload containing frames `0..k` of one channel, with only frame `k` carrying `is_last == true`, and containing at least one complete L2 block matching its selected record.
A successful submission then advances `cursor.block_number` and closes the channel in the same acceptance, leaving no partial channel for a later reset to abandon.
Competing submissions based on the old cursor can still revert, but each accepted submission of this form advances the record cursor.
This is a packing recommendation, not an acceptance restriction: channels spread across submissions remain permitted, and coordinating their completion remains a liveness assumption.
The recommendation does not prevent an opener from wasting its own timeout on a channel it never closes; it does make the common recovery a single submission that opens and closes in one acceptance.

**Why an emergency override exists at all, given cursor continuity.**
Normal submissions extend the cursor expected by [acceptance](#acceptance); they cannot rewind its record progress.
A cursor from which no valid next attestation can be produced under the specified rules, such as one with an unusable quorum anchor, can therefore block further submissions even while record content remains public.
Repairing such cursor fields requires an override, subject to its [repair limits](recovery.md#s8-a-previous-override-left-the-cursor-at-an-unreachable-position), or another authorized repair mechanism; changing the fixed inbox address is not a recovery mechanism specified here.
A valid cursor that has advanced past content derivation did not ingest is a different case: subsequent submissions may extend it, but cannot reinsert the skipped heights through the ordinary submission path.
That content loss does not by itself require an override to resume submissions, and an advance-only override cannot rewind it either.
Continued inbox acceptance does not establish that later content derives successfully; that still depends on the applicable derivation and availability rules.
The authority chosen is one that already exists in the OP Stack contract set, so the override introduces no new role and no party who did not already hold power over the chain's contracts.
Its scope is deliberately narrow: advance-only, with no open channel carried across and the same origin bound acceptance applies.
What it can express is therefore a resume position, and never a settlement.
Its breadth is equally deliberate: the override is insurance against an implementation defect that leaves the cursor in a state no submission can continue from, which may include a wrong quorum anchor, and narrowing what it can rewrite would narrow what it can repair.
The price is the trust premise the [Security Considerations](#security-considerations) state, accepted knowingly.

**Why the cursor-advance event is specified rather than left to the implementation.**
An event is usually telemetry, and telemetry does not belong in a specification.
This one is a consensus rule: [derivation](../rollup/derivation.md#batcher-transaction-selection) ingests a transaction only if its receipt carries this event from this contract, so the signature, the field order and the emitting path are all part of what every node must agree on.
It also serves an observer reconstructing the settled chain's history from L1 alone, who must distinguish an accepted submission from a reverted one and from an [override](#emergency-cursor-override).
A reverted submission emits nothing because the revert discards the log, and an override emits `CursorSet` instead of `CursorAdvanced`.
The presence of the event is therefore exactly the statement that the cursor advanced through the normal path.

**Why batch submissions require a direct call from a code-empty EOA.**
[Derivation](../rollup/derivation.md#batcher-transaction-selection) reads the outer transaction's recipient and calldata, so accepting an internal call could advance the cursor through data it never ingests.
Equality between `msg.sender` and `tx.origin` rejects ordinary forwarded calls, including calls from a contract constructor whose deployed code is still empty.
That equality alone is insufficient under [EIP-7702](https://eips.ethereum.org/EIPS/eip-7702#self-sponsoring-allowing-txorigin-to-set-code): a delegated transaction origin can call the inbox from its own execution context.
Checking for empty account code at submission also rejects that path, including delegation installed after registration.
The inbox proxy's own `delegatecall` preserves the original sender and calldata and does not violate these checks.

**Why registry membership is curated rather than open.**
Open enrolment would let any address acquire the right to trigger frame-0 resets, which orphan dispersal the primary has already paid for.
Curation limits who can reset, and the exclusive window limits when alternatives may submit; neither bounds the duration of contention once the window expires.

**Why the contract does not verify L1 headers.**
The design considered was to have the [certifier](certifier.md) take each L1 origin's header as an input, read the origin timestamp out of it, and commit to `(epoch_num, epoch_hash)` in its attestation, with this contract then verifying that hash against the canonical L1 block.
It is attractive for a specific reason worth recording: because a header's hash is its keccak, a single 32-byte comparison on L1 would validate every piece of the certifier's L1-dependent arithmetic at once, with the header itself never entering calldata.
It was rejected because the conditions it would pre-empt — the origin's timestamp relation and the [sequencer drift](https://specs.optimism.io/protocol/derivation.html#batch-queue) bound — are fixed by the selected record, and refusing such a submission does not save the height.
The record is final, no corrected version of it exists, and so refusal only prevents the cursor from ever passing that height, trading a height that would have settled as a [deposit-only block](https://specs.optimism.io/protocol/derivation.html#batch-queue) for a halt requiring an [emergency cursor override](#emergency-cursor-override).
The general rule this instance follows is the one stated in [the block records page's rationale for checking record-fixed values before selection is fixed](block-records.md#rationale): a condition is worth pre-empting only where the party controlling the offending value can still produce a corrected submission.

**Why direct L1 hash lookup does not cover every recovery submission.**
Direct lookup reaches only a bounded history: 256 blocks through `BLOCKHASH`, or 8191 through the historical-hash contract [EIP-2935](https://eips.ethereum.org/EIPS/eip-2935) introduced in Prague.
A [recovery submission](#recovery-by-an-alternative-batcher) whose origin is older than the available history cannot authenticate that origin through direct lookup alone.
Authenticating older history requires an additional proof mechanism or an authenticated store of historical hashes, neither of which this design specifies; [EIP-2935's rationale](https://eips.ethereum.org/EIPS/eip-2935#rationale) describes the availability of more complex historical proofs.
The limited lookup range therefore constrains the direct-comparison design considered above, without ruling out other header-verification designs.

**Why the contract, not the certifier, bounds `epoch_num` against `block.number`.**
The certifier constrains how fast the anchor may advance, but it cannot perform this check itself: it is a pure function of inputs it is handed, it performs no I/O, and the current L1 height is precisely the kind of fact an untrusted certifier host cannot be trusted to supply.
The contract executes inside an L1 block and knows `block.number` at no cost, which makes it the only party positioned to establish that the origin exists at all.
The check is placed immediately after the continuity check so that it remains a comparison against values already loaded rather than an addition to the verification cost a losing racer pays.

**Why `epoch_num` is not bounded against the sequencing window either.**
[Derivation](../rollup/derivation.md) drops a batch whose origin is older than `seq_window_size` L1 blocks before the block including it, and because this contract executes inside that very block it could reproduce that condition exactly.
It does not, for the same reason the header check was rejected: the origin comes from the selected record, so a rejected submission has no corrected successor, and rejecting would again convert a lost height into a halted cursor.
The existing bound of `epoch_num` against `block.number` under [acceptance](#acceptance) is different in kind — it rejects an origin that does not exist yet, which is not a statement about whether the height can settle.

**Why the channel timeout is enforced here, when the sequencing window is not.**
The two timing rules differ in what a refusal buys.
A batch past its sequencing window is lost whether or not this contract accepts it — the height is filled without it either way — so refusing it would only halt the cursor in front of a height derivation has already given up on, which is why the window is left to derivation.
A frame past the channel timeout is different: derivation has discarded the channel, but the content is still submittable in a fresh channel opened by the same or another batcher, provided the cursor has not moved.
Accepting such a frame is what makes the loss permanent, because acceptance advances the cursor past blocks that will never derive and nothing can bring it back; refusing it keeps the content in play at the cost of one reverted transaction.
This is the [governing principle](derivation-pipeline.md#the-governing-principle) applied at the inbox: pre-empt a condition exactly when the party controlling the offending value can still produce a corrected submission, and here the batcher can.
The rule is a copy of derivation's kept in one constant, and a mismatch between the two copies errs safely in only one direction.
A contract timeout shorter than derivation's refuses frames derivation would still have accepted, costing a resubmission, while a longer one reopens the exposure.
That is why `CHANNEL_TIMEOUT` MUST equal the timeout derivation's channel bank actually applies, `ChannelTimeoutGranite`, and not the rollup configuration's `channel_timeout` field, which derivation no longer reads.

## Security Considerations

**Assets and threat model.** This section protects three properties: that every submission the inbox accepts is one derivation ingests and nothing else is, that `cursor` advances only through a continuous chain of attested steps from the [initial cursor](activation.md#initial-cursor), and that the scheduled bootstrap authority, then the [batcher registry](#batcher-registry) and [exclusive window](#exclusive-window), decide who may submit and when.
The inbox trusts the enclave keys its `certifier_verifier` authenticates as registered under authorized measurements — that is, the certifier core and the Nitro attestation behind it; the administrator who schedules the initial Espresso scan and trusted quorum anchor; and the L1 super admin, who curates the registry, authorizes measurements, performs the [emergency override](#emergency-cursor-override) and owns the EigenDA certificate verifier router that sets the certificate security parameters.
Every batcher, primary or alternative, is untrusted, and every byte it submits is checked against an attestation before it moves the cursor.
The certifier host is untrusted, which is why the contract itself bounds `epoch_num` against `block.number`; the mempool and the submission's calldata are untrusted, which is why cursor continuity defeats replay and registry evidence is rebuilt from EigenDA's registries rather than read from the submission.
The [Roles and Trust](overview.md#roles-and-trust) table records these premises.

**Delegated verification extends the governance trust boundary.**
A proof-verification gateway that delegates to a mutable route can admit a forged registration proof if its controller substitutes an unsound verifier, even when the inbox's own verifier and authorized-measurement set remain unchanged.
The resulting registration can admit an arbitrary signing key under an authorized measurement; subsequent signature checks and the permanent measurement binding do not establish that the initial registration was authentic.
The [governance rules](#certifier-key-registration) therefore follow acceptance decisions through every delegated dependency, rather than stopping at the contract the inbox calls.
A mutable dependency controlled outside the L1 admin adds its controller to the trusted set for the decisions it can change.

**Acceptance and ingestion must agree.**
This specification preserves the invariant that every submission the contract accepts is a submission derivation ingests, and nothing else is.
Because derivation selects on the [acceptance event](#events) alone, the invariant holds by construction so long as every node matches the same event from the same address.
A node that instead evaluated senders against a registry of its own — or that trusted a configured sender without the event, or ingested a successful transaction that carried no event — could ingest what the contract refused or skip what it accepted.
Skipping an accepted frame can leave derivation without content the cursor has already passed; ingesting unaccepted bytes can instead bypass certification.
If a closing submission advances the record cursor past missing content, later ordinary submissions cannot reinsert those heights, but they can still use the stored cursor as `prev_cursor` when a valid continuation exists.
This distinguishes loss of earlier content from an inability to continue submitting; neither outcome follows merely from comparing `prev_cursor` with the stored value.

**A blob-carrying submission would divide acceptance from ingestion at the byte level.**
The contract judges calldata; a type 3 transaction also carries a blob sidecar that the baseline pipeline reads instead of the calldata.
Without the `blobhash` check, a submitter could pass every acceptance check on honest calldata while the transaction's blob held bytes no certifier attested, and whichever side derivation read from, the outcome is a failure: reading the blob silently settles content no certifier attested, reading nothing leaves `cursor` past a frame the chain never receives.
The check and the [derivation-side exclusion](../rollup/derivation.md#batcher-transaction-selection) of type 3 together make the accepted calldata the only bytes a batch transaction carries.

**Registry evidence is rebuilt from the registries, never taken from calldata.**
If the contract accepted the operator-set data from the submission and merely hashed it, the digest would match whatever the host had fed the core, and a fabricated stake table would pass.
Rebuilding the evidence from EigenDA's registries at the certificate's reference block is what makes the attestation's evidence hash a statement about the real operator set; the aggregate signature itself is checked only inside the enclave, on the terms the [certifier page](certifier.md#security-considerations) states.

**The EigenDA security parameters are only as sound as the parties that set them.**
The owner of [`EIGENDA_CERT_VERIFIER_ROUTER`](#constants) can add a verifier with a lower confirmation threshold or fewer required quorums for future reference blocks, and every certificate referencing those blocks is then accepted on those terms.
That owner is therefore a trusted governance role, a premise of the same kind as the [authorized-measurement set](#certifier-key-registration): lowering the parameters lowers the data-availability security of settlement without any acceptance check failing.
The stale-stake switch is controlled by EigenDA's registry-coordinator owner, which is already trusted for the operator registries the evidence is read from; turning it off admits certificates judged against quorum stakes last updated more than the minimum withdrawal delay before their reference block, and adds no party beyond that one.

**Revocation is a liveness cost that MUST NOT be converted into a safety one.**
Withdrawing a measurement invalidates every key registered under it, and an implementation tempted to soften that — honoring in-flight submissions, or grandfathering keys registered before the withdrawal — would leave a compromised measurement's signatures admissible for exactly as long as the grace period lasts, which is the window an attacker holding such a key needs.
The specified behavior is that revoked keys fail the acceptance check like any other inadmissible key, leaving the cursor unmoved and the content resubmittable.
The permanent registration binding prevents an old image's key from remaining admissible through registration under a replacement measurement.
Revocation does not erase a key already held by an enclave; its signatures become inadmissible because the bound measurement is withdrawn.
Reauthorizing the same measurement also restores any compromised key previously registered under it.
Removal followed by reauthorization therefore does not rotate keys or permanently retire old ones; it restores their authorization together, subject to the other acceptance checks.

**Admission and exclusivity bound recovery contention to one channel timeout per occurrence.**
A registered batcher that is hostile rather than merely unhelpful can wait for the window to expire and open a channel it never closes.
It cannot corrupt the chain: each attested block matches the selected record, and successful acceptance of the closing submission makes the records in its cursor advance canonical.
What it costs is one [`CHANNEL_TIMEOUT`](#constants) during which no one else may submit; then the channel counts as closed, any admitted sender opens its own, and a primary that opens and closes a channel in one submission re-arms the [exclusive window](#exclusive-window) against the intruder.
Curation limits who may do this and the timeout limits each occurrence, so the total cost is the number of occurrences the `admin` tolerates before removing the address.

**Alternating resets cannot occur, because an open channel belongs to its opener until it times out.**
Two registered batchers each recovering the same stalled chain would otherwise orphan each other's channels in turn — every submission accepted, every acceptance rewriting only `frames_digest`, `cursor.block_number` never moving — with no misbehaving party.
Under [acceptance](#acceptance) the second batcher's submission is refused while the first's channel is open and within `CHANNEL_TIMEOUT`, so the first has the whole timeout to close its channel; if it does not, the channel counts as closed and the second opens its own.
What the protocol does not ensure is that an opener uses its timeout well; the [complete-channel recommendation](#rationale) makes the ordinary recovery a single submission, and a slow opener costs at most one timeout per channel it abandons.

**A forged future origin would make exclusivity permanent.**
Nothing in a block record's content bounds its `epoch_num` from above.
Without the contract's `c.cursor.epoch_num <= block.number` check, a publisher could therefore name an L1 origin far in the future, settle one block against it, and hold `cursor.epoch_num + EXCLUSIVE_WINDOW` beyond the L1 head indefinitely.
That would reinstate exactly the settlement monopoly the [rationale for admitting more than one submitter](#rationale) exists to remove, and do so with a single accepted submission.
[Record Validity](block-records.md#record-validity)'s continuity condition does not close this alone, because monotonicity says nothing about how far ahead an origin may be named.

**A certified batch may still be discarded, and the contract does not prevent that.**
Advancing `cursor` past an L2 block attests that the block's content is what Espresso finalized; it does not establish that derivation will accept it.
A batch submitted after its own `seq_window_size` deadline can be attested, accepted, and advance the cursor, and still be replaced by a deposit-only block.
This is deliberate for the sequencing window — refusing a batch that is lost either way would only halt the cursor — and it does not weaken the [Certification Claim](overview.md#the-certification-claim), whose outcome is decided solely by derivation.
The channel timeout is the one derivation timing rule this contract does reproduce, under [Acceptance](#acceptance), because there a refusal saves the content.
Without it, a closing frame included late — by congestion, by a slow disperser, or by a primary that delays it deliberately while keeping the [exclusive window](#exclusive-window) fresh with each closing frame — would be accepted.
It would advance the cursor past a channel derivation had discarded, and settle every block in it deposit-only, with nothing on L1 to distinguish the outcome from honest operation.

**Alternative batchers are unincentivized.**
An alternative batcher earns no revenue for submitting and pays gas and dispersal fees to do so, so the recovery path depends on parties with an indirect stake in the chain's liveness choosing to bear that cost within the window.
This is weaker than a cryptographic guarantee and stronger than assuming the sequencer behaves: it requires only that at least one such party acts, not that the party holding the settlement monopoly relinquishes it.
An implementation that assumed recovery is automatic, and sized `EXCLUSIVE_WINDOW` on that assumption, would leave no margin for the case the mechanism exists to cover.

**An override can invalidate an earlier certified determination.**
The [override](#emergency-cursor-override) advances the L2 cursor without making skipped records canonical.
An override whose `height` and `index` skip a previously selected record for a future height can make the resumed scan select a later conflicting record.
Observers following the same L1 history agree on that new selection, but a full node's earlier certified determination may no longer hold.
The contract does not verify the supplied scan position or quorum anchor, so the [Certification Claim](overview.md#the-certification-claim) depends on the admin preserving those premises.
If a skipped height receives no valid batch within its sequencing window, normal derivation fills it deposit-only; this fallback does not protect subsequent heights from an incorrect resume position.
An override issued in error or through a compromised admin key can therefore affect safety as well as liveness.

**Changing `Cursor` can break event recognition or silently change decoded meaning.**
Changing the canonical type sequence changes the [event signature](#events), so a consumer still matching the old `topic0` no longer recognises accepted submissions.
For derivation, this can leave L1 accepting submissions whose data L2 does not ingest, leading to deposit-only fallback.
Reordering fields while preserving the type sequence instead leaves `topic0` unchanged: swapping `block_number` and `epoch_num`, both `uint64`, makes a consumer using the old layout interpret each value as the other field.
Consumers that decode cursor fields can therefore misinterpret an event they still recognise; topic matching alone does not establish layout compatibility.
A change to `Cursor`'s field order or types MUST be treated as an event-interface change — versioned, announced, and coordinated with its consumers — even when `topic0` stays unchanged.

**An override is announced, but never as an acceptance.**
An [override](#emergency-cursor-override) emits [`CursorSet`](#events) rather than `CursorAdvanced`, so the event stream distinguishes "a settlement position was asserted by governance" from "a submission was accepted".
Derivation consumes the override as a control input that resets channel state and fills missing heights, while selecting batch content only through `CursorAdvanced`.
An override MUST NOT replace the established safe prefix or finalized history; its inclusion MUST NOT independently confer finality on the fill.
The signal is what lets an observer reconstruct [record selection and canonicality](block-records.md#canonicality) across the discontinuity, since each override begins a new scan segment; without it an observer would have to poll `cursor` to notice that the value moved and could not tell where the certifier's scan resumed.

**Withholding at the source remains out of scope.**
These rules address a sequencer that certifies blocks and then declines to settle them.
They do not replace a sequencer that stops producing blocks or supply records for the resulting missing heights; that case is a governance question about appointing a new sequencer, not one this contract can resolve.
Already finalized selected records remain submittable under the existing rules while their data and required proofs are available; the source halt alone does not invalidate that backlog.

**L1 rollback restores the cursor but does not guarantee preservation of selected content.**
Orphaning an accepted submission rolls back its inbox state changes, including the cursor update.
If the submission is accepted again on the replacement chain, the proposed [origin-resolution delta](derivation-pipeline.md#delta-4-resolve-the-l1-origin-from-epoch_num) resolves `epoch_num` against the new canonical L1 history.
That removes the stale-hash rejection but does not preserve execution validity: changed deposits or other execution inputs may make the selected record's block invalid.
Derivation then substitutes a deposit-only block and follows [recovery S3](recovery.md#s3-an-l1-reorganization).

**Registration is a precondition of the recovery path, not merely of efficiency.**
An alternative batcher must be in the [batcher registry](#batcher-registry) this contract reads and must have registered a certifier key per [Certifier Key Registration](#certifier-key-registration) before it can submit anything at all.
A full node that can verify a height will settle, but is not registered, cannot act on that knowledge — it depends on some registered party submitting.
Whether registry membership should remain in the `admin`'s hands is therefore a question about the recovery path's availability and not only about who may submit during normal operation.

## Spec History

Block certification has not been activated on any network.
