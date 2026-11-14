---
description: Normative block production rules — sequencer authority, and transaction admission and ordering.
spec: Rex5
---

# Sequencing

At any moment, exactly one [sequencer](https://specs.optimism.io/glossary.html#sequencer) is authorized to produce MegaETH blocks.
This page specifies who is authorized to sign mini-blocks, and how deposit and L1-attributes transactions must be ordered relative to ordinary transactions; everything about how the sequencer internally produces blocks — mini-block sealing cadence, and whether or how it is internally split into separate components (unlike vanilla OP Stack and Ethereum, MegaETH does not specify a consensus-layer/execution-layer protocol boundary here) — is sequencer implementation and is explicitly out of scope, per the [scope principle](../../AGENTS.md#scope).

## Motivation

Ordinary transactions are unconstrained in the order the sequencer admits them, but the [deposit](https://specs.optimism.io/glossary.html#user-deposited-transaction) and [L1-attributes](https://specs.optimism.io/protocol/deposits.html#l1-attributes-deposited-transaction) transactions that a [rollup node](https://specs.optimism.io/glossary.html#rollup-node) supplies at the start of each EVM block are not: later transactions in that block may depend on state those transactions establish, so their position relative to ordinary transactions must be fixed.

## Sequencer Authority

The active sequencer for a given EVM block is the key returned by `sequencerAt(block_number)` on the [SequencerRegistry](../system-contracts/sequencer-registry.md#read-methods) system contract, as specified in [Mini-Block Header Signing](blocks.md#mini-block-header-signing).
`SequencerRegistry` applies any pending sequencer-key change as a pre-block operation, before the EVM block's first mini-block is built; consequently, `sequencerAt(block_number)` MUST NOT change value between the first and last mini-block of the same EVM block, and every mini-block within one EVM block MUST be signed by the same key.

## Block-Production Mechanics

Every EVM block MUST be composed of one or more mini-blocks, per [Blocks and Mini-Blocks' Block Hierarchy rule](blocks.md#block-hierarchy) — mini-blocks are the minimal unit the state sync protocol synchronizes, so this minimum is a real liveness dependency, not implementation policy.
Beyond that minimum, how many mini-blocks the sequencer produces for a given EVM block, when it seals each one and how much content it includes, and whether and how it is internally split into separate components (for example, a block-building component and an execution component communicating over an internal RPC boundary) are not specified by this page.
No validity rule anywhere in this specification checks any of that remaining freedom, and any internal architecture that produces the same signed mini-blocks and EVM blocks is indistinguishable to every other node, per the [scope principle](../../AGENTS.md#scope); unlike vanilla OP Stack and Ethereum, which specify a consensus-layer/execution-layer split as a protocol interface because independently developed clients on each side must interoperate, MegaETH's sequencer is not decomposed this way for protocol purposes.

The only externally verifiable fact about how the sequencer operates is the identity check specified in [Sequencer Authority](#sequencer-authority): a mini-block's signature MUST recover to the current `sequencerAt(block_number)`, regardless of which physical instance or internal components produced it.

## Transaction Admission and Ordering

The first mini-block of each EVM block MUST include every [deposit](https://specs.optimism.io/glossary.html#user-deposited-transaction) and [L1-attributes](https://specs.optimism.io/protocol/deposits.html#l1-attributes-deposited-transaction) transaction supplied for that block, and MUST order them before any other transaction in that mini-block.

Beyond that constraint, the order in which the sequencer admits and includes ordinary transactions into mini-blocks is unconstrained by this specification: a node validating a block checks that each included transaction is individually valid (correctly signed, admissible nonce, sufficient balance) and that the resulting state transition is correct, but does not re-derive or check the order in which the sequencer chose to include transactions.
Two sequencer implementations that admit and order ordinary transactions by entirely different policies (e.g. arrival order, priority-fee order) are indistinguishable to any external observer as long as each individually produces valid transactions in a valid resulting state, per the [scope principle](../../AGENTS.md#scope); transaction ordering policy is therefore an implementation concern, not specified further here.

## Constants

This page defines no named constants.
Block-production mechanics are sequencer implementation choices, not spec-defined values; see [Block-Production Mechanics](#block-production-mechanics).

## Rationale

**Why transaction ordering is unconstrained beyond the deposit/L1-attributes-first rule.**
Verification checks the result of applying transactions, not the order the sequencer chose them in; constraining ordering further would add a rule no node needs to check, at the cost of removing sequencer implementation freedom for no verifiable benefit.

**Why block-production mechanics — sealing cadence and any internal component split — are not specified.**
Both change only the sequencer's own internal scheduling and architecture, never a check any node applies when validating a received mini-block or EVM block.
Per the same reasoning as transaction ordering, mandating a specific policy here would add rules no node ever checks, at the cost of removing implementation freedom other MegaETH sequencer implementations may need — a different mini-block cadence, a different internal architecture entirely — for no verifiable benefit.
This includes not specifying a consensus-layer/execution-layer boundary the way vanilla OP Stack and Ethereum do: that split exists there to let independently developed clients on each side interoperate, a need MegaETH's sequencer does not have, since no other node ever observes its internal architecture.

## Security Considerations

**A mismatched sequencer-key signature halts block production.**
A node rejects any mini-block whose signature does not recover to the current sequencer key, so a compromised or misconfigured key halts block production outright, rather than degrading silently.

**No block-production mechanic is security-relevant to a validating node.**
A sequencer that seals mini-blocks unpredictably, or is internally architected in any way at all, still produces only validly signed mini-blocks and EVM blocks per [Sequencer Authority](#sequencer-authority).
A consumer relying on predictable mini-block cadence, or any assumption about the sequencer's internal architecture, as a security property rather than a quality-of-service one, is relying on sequencer implementation behavior this specification does not guarantee.

## Spec History

No spec-gated changes to the behavior on this page have been identified; verify against [Hardforks and Specs](../hardfork-spec.md) before treating this as exhaustive.
