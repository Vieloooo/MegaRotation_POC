# MegaETH Specification

The formal specification of the MegaETH protocol — the complete, normative definition of how MegaETH produces blocks, processes transactions, manages protocol state, and settles to Ethereum L1.

Published at [docs.megaeth.com/spec](https://docs.megaeth.com/spec/) via GitBook Git Sync.

This repository is separate from the public [documentation repository](https://github.com/megaeth-labs/documentation) because the protocol is not yet audited; visibility of this repository is restricted accordingly.

## Structure

```
specs/                  Specification source
├── SUMMARY.md          Table of contents / navigation
├── overview.md         Landing page
├── hardfork-spec.md    Hardfork and spec versioning
├── glossary.md         Canonical protocol terminology
├── protocol/           L2 node spec: blocks, sequencing, SALT, streaming, validation
├── rollup/             L1↔L2 pipeline spec: derivation, DA, batching, bridges, fault proofs
├── evm/                MegaEVM behavioral spec
├── system-contracts/   System contract specs
└── upgrades/           Per-hardfork (Rex*) delta pages
```

## Linting & Formatting

CI runs three checks on all Markdown files:

| Tool                                                            | What it checks                                             | Config                    |
| --------------------------------------------------------------- | ---------------------------------------------------------- | ------------------------- |
| [lychee](https://lychee.cli.rs/)                                | Internal links resolve to existing files                   | `lychee.toml`             |
| [markdownlint](https://github.com/DavidAnson/markdownlint-cli2) | Markdown structure (blank lines, heading style, bare URLs) | `.markdownlint-cli2.yaml` |
| [Prettier](https://prettier.io/)                                | Consistent formatting (tables, spacing, indentation)       | `.prettierrc.yaml`        |

To run locally:

```bash
# Install tools via mise (one-time setup)
mise install

# Run all checks
mise run lint

# Auto-fix formatting
mise run fmt
```

## Contributing

1. Edit or create Markdown files under `specs/`.
2. Update `specs/SUMMARY.md` if adding or removing pages.
3. Run `mise run fmt` to format, then `mise run lint` to verify.
4. See [`AGENTS.md`](AGENTS.md) and [`specs/AGENTS.md`](specs/AGENTS.md) for writing conventions.
