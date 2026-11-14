---
description: The transaction types that may legitimately appear in a MegaETH block — standard and deposited transactions inherited from the OP Stack, and the MegaETH-native mega system transaction.
spec: Rex5
---

# Transactions

A MegaETH block is built from standard Ethereum transactions, OP Stack-inherited deposited transactions, and the mega system transaction, a MegaETH-native mechanism for protocol-managed maintenance writes.
This page enumerates all of them and specifies the mega system transaction in full; the other types are inherited unchanged and specified by reference.

## Motivation

An implementer needs one canonical list of what content a MegaETH block may legitimately contain; without it, the transaction-type rules are scattered across whichever page happens to mention them.

Most of that content already has a canonical format and processing rules defined by the OP Stack, which MegaETH inherits unchanged — restating those rules here would duplicate, and risk drifting from, that baseline.

One class of transaction is MegaETH-native.
Protocol-managed maintenance operations — for example, updating [oracle](../system-contracts/oracle.md)-managed storage — need to execute with fee and validation semantics different from an ordinary call, while remaining part of the same verifiable execution model as every other transaction; an out-of-band, non-transactional mechanism could not offer that.
The mega system transaction provides it.

## Transaction Types

A MegaETH block MAY contain transactions of the following kinds:

- **Standard transactions** — ordinary Ethereum transactions, whether legacy-encoded or an [EIP-2718](https://eips.ethereum.org/EIPS/eip-2718) typed envelope, unmodified from standard Ethereum.
- **[Deposited transactions](https://specs.optimism.io/glossary.html#deposited-transaction)** — inherited unchanged from the OP Stack, including [user-deposited transactions](https://specs.optimism.io/glossary.html#user-deposited-transaction) and the [L1-attributes deposited transaction](https://specs.optimism.io/protocol/deposits.html#l1-attributes-deposited-transaction); see the linked OP Stack specification for their wire format and processing rules.
  Their required position within a block's first mini-block is specified in [Sequencing's Transaction Admission and Ordering](sequencing.md#transaction-admission-and-ordering).
- **Mega system transactions** — a MegaETH-native mechanism, specified in full below.

## Mega System Transaction Identification

A mega system transaction uses the ordinary legacy transaction encoding: it carries no [EIP-2718](https://eips.ethereum.org/EIPS/eip-2718) type-envelope byte, the same encoding client implementations conventionally number as transaction type `0`.
Nothing in a mega system transaction's on-wire bytes, hash, or fields distinguishes it from any other legacy transaction; whether a given transaction is a mega system transaction is a classification a node MUST derive at processing time, not a property carried on the transaction itself.

A node MUST classify a transaction as a mega system transaction if and only if all of the following hold:

- the transaction uses the legacy encoding (no EIP-2718 type byte);
- the transaction's signer equals the current system address (see [Mega System Transaction Signer Authority](#mega-system-transaction-signer-authority));
- the transaction is a `CALL` (not a `CREATE`) to an address in `MEGA_SYSTEM_TX_WHITELIST`.

A `CREATE` transaction from the current system address MUST NOT be classified as a mega system transaction; it MUST be processed as an ordinary transaction, subject to ordinary signature, nonce, and fee rules.

## Mega System Transaction Signer Authority

The signer of a mega system transaction MUST be the system address returned by [`SequencerRegistry`'s read methods](../system-contracts/sequencer-registry.md#read-methods) for the block in which the transaction appears, resolved after all pre-block state changes for that block are committed.
This system address is tracked by `SequencerRegistry` independently of the sequencer key that signs [mini-blocks](blocks.md#mini-block-header-signing), and rotates on its own schedule; a change to one role MUST NOT be construed as a change to the other.

## Mega System Transaction Processing

A node MUST process a transaction classified as a mega system transaction with the following semantics, applied in place of the ordinary transaction-validity rules:

- It MUST bypass nonce validation, though the signer's nonce MUST still increment after execution, as it would for any other transaction from that signer.
- It MUST bypass gas-fee charging and balance checks for gas fees.
- It MUST preserve the current system address as the caller the target contract observes, exactly as an ordinary transaction's sender would be observed.

A mega system transaction's execution MUST still be metered by its `gasLimit` exactly as an ordinary transaction's is, and its `gasUsed` MUST be included in the containing mini-block's and EVM block's cumulative gas accounting, per [Mini-Block Body](blocks.md#mini-block-body).
A mega system transaction MUST NOT cause any state change to the block beneficiary or any fee vault, since no fee is charged.
A mega system transaction's receipt MUST have the same shape (status, cumulative gas used, logs, bloom) as any other transaction's receipt; it is not a distinct receipt type.

## Constants

| Constant                   | Value                                                            | Description                                                              |
| -------------------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------ |
| `MEGA_SYSTEM_TX_WHITELIST` | `{` [Oracle](../system-contracts/oracle.md) contract address `}` | The stable set of contract addresses a mega system transaction MAY call. |

The current system address is not a page-level constant: from Rex5 onward it MUST be resolved per block from `SequencerRegistry`, per [Mega System Transaction Signer Authority](#mega-system-transaction-signer-authority), not from a fixed value.
Its pre-Rex5 fixed value is recorded in [Spec History](#spec-history).

## Rationale

**Why special transaction semantics instead of ordinary calls.**
Protocol-maintenance operations, such as oracle updates, must execute with deterministic behavior that is not coupled to ordinary fee payment and admission semantics.
Giving them distinct execution treatment makes that behavior explicit, while still keeping them fully verifiable as part of ordinary block execution — no out-of-band or non-transactional mechanism is needed.

**Why preserve the caller as the current system address.**
A whitelisted target contract must be able to distinguish a protocol-maintenance call from an ordinary user call.
Preserving the caller identity provides that signal without introducing a separate call context type.

## Security Considerations

**The fee and nonce exemption MUST apply exclusively to addresses in `MEGA_SYSTEM_TX_WHITELIST`.**
If the whitelist check were absent or misconfigured, any legacy transaction from the current system address could bypass gas fees and nonce validation for arbitrary targets, enabling cost-free state manipulation across the entire chain.

**Adding an address to `MEGA_SYSTEM_TX_WHITELIST` is a consensus-critical change.**
Any modification to the whitelist changes which transactions receive privileged processing; such a change MUST be gated by a spec upgrade, so all nodes agree on the new whitelist contents at the same activation point.

**A `CREATE` transaction from the current system address MUST NOT receive privileged processing.**
Allowing fee-exempt contract creation would let the current system address deploy arbitrary code at no cost, outside the intended maintenance scope.

**Signature validation is required for a mega system transaction, not bypassed.**
The identification rule requires the signer to equal the current system address, which can only be established by successfully recovering it from a genuine ECDSA signature; a transaction with no signature, or one that recovers to any other address, MUST NOT be classified as a mega system transaction.
This is what makes forging one require the system address's private key, not merely knowledge of the address — the security of the whole mechanism reduces to the same key-custody and rotation guarantees [`SequencerRegistry`'s change scheduling](../system-contracts/sequencer-registry.md#change-scheduling) provides for the system address.

**Where a mega system transaction originates is invisible to the protocol.**
Because classification depends only on a transaction's own content (encoding, signer, target), not on where it came from, no node validating a block can distinguish a mega system transaction the sequencer itself constructed from one that — hypothetically — arrived through some other channel with a valid signature.
This specification therefore states no origination rule: a sequencer that constructs mega system transactions internally and one that admitted an externally-submitted transaction bearing a valid system-address signature would both produce validly classified, validly executed blocks.
Restricting construction to the sequencer itself, and declining to admit system-address transactions arriving over an externally-facing intake path, is nonetheless sound operational practice — it narrows the blast radius of a leaked system-address key — but it is sequencer-side defense in depth, not a protocol invariant.

**Content integrity beyond signer identity depends on the target contract's own access control, not on a rule specified here.**
This page specifies only who MUST sign a mega system transaction and which contracts it MAY call; whether a correctly-signed, correctly-targeted transaction is further rejected, and what state changes it may make, are properties of whichever system contract it targets, specified on that contract's own page — not an independent check performed by this specification.

## Spec History

- [MiniRex](../upgrades/minirex.md) introduced mega system transactions and the fixed `MEGA_SYSTEM_ADDRESS` constant (`0xA887dCB9D5f39Ef79272801d05Abdf707CFBbD1d`).
- [Rex](../upgrades/rex.md), [Rex1](../upgrades/rex1.md), [Rex2](../upgrades/rex2.md), and [Rex3](../upgrades/rex3.md) retained the same semantics.
- [Rex5](../upgrades/rex5.md) replaced the fixed `MEGA_SYSTEM_ADDRESS` constant with per-block dynamic resolution from `SequencerRegistry`; the identification rule and whitelist are otherwise unchanged.
