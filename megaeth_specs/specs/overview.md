---
description: MegaETH formal specification — normative behavioral definitions for the protocol, rollup and settlement pipeline, EVM execution, system contracts, and network upgrades.
---

# MegaETH Specification

This is the formal specification of MegaETH's verifiable behavior — the complete, normative definition of how MegaETH produces blocks, processes transactions, manages protocol state, and settles to Ethereum L1.

Any node implementation that produces the same outputs given the same inputs for every case defined in this specification is a correct MegaETH node.

MegaETH is built on the OP Stack.
All standard OP Stack protocol semantics and Ethereum EVM semantics are inherited unless explicitly overridden by this specification.
Where a section covers a MegaETH-native subsystem (mini-blocks, SALT state, state sync, stateless validation), it is specified here in full, as is [block certification](block-certification/overview.md) in its own section — apart from the two neighbouring behaviors it specifies as deltas over the OP Stack under [Rollup & Settlement](rollup/overview.md).
Where a section modifies a standard OP Stack component (derivation, data availability, batch submission, fault proofs), it specifies only the behavioral deltas over the [OP Stack specifications](https://specs.optimism.io/).

## Organization

The specification is organized by component, targeting the implementer of each part of the system:

- **[Protocol](protocol/overview.md)** — the MegaETH L2 node itself: block and mini-block structure, sequencing rules, SALT state, state sync, and stateless validation.
- **[Rollup & Settlement](rollup/overview.md)** — the L1↔L2 pipeline: derivation, data availability, batch submission, deposits, withdrawals, L1 contracts, and fault proofs.
- **[Block Certification](block-certification/overview.md)** — publishing each L2 block to an external consensus network and gating settlement on what was published, making a block's ordering irreversible and its data public within seconds.
- **[MegaEVM](evm/overview.md)** — EVM execution semantics: the dual gas model, multidimensional resource limits, gas detention, and other execution-layer differences from standard Ethereum.
- **[System Contracts](system-contracts/overview.md)** — pre-deployed protocol contracts: oracle storage, high-precision timestamps, keyless deployment, access and limit control, and the sequencer registry.
- **[Network Upgrades](upgrades/overview.md)** — the versioned spec progression: each hardfork's exact behavioral deltas.
- **[Glossary](glossary.md)** — canonical definitions of MegaETH-specific terms.

Concept pages describe the behavior of the latest stable spec; upgrade pages record the delta introduced by each hardfork.
Start with [Hardforks and Specs](hardfork-spec.md) to understand how behavior is versioned.

---

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this specification are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) ([RFC 2119](https://www.rfc-editor.org/rfc/rfc2119), [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174)) when, and only when, they appear in all capitals, as shown here.
