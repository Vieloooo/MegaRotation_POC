# AGENTS.md

This file provides guidance to AI agents (e.g., Claude Code, Codex, Cursor, etc.) when working with content in this repository.

## Project Overview

MegaETH Specification — the formal, normative specification of the MegaETH protocol.
It is published at [docs.megaeth.com/spec](https://docs.megaeth.com/spec/) via GitBook Git Sync, as a GitBook space separate from the public [documentation site](https://docs.megaeth.com) and its [repository](https://github.com/megaeth-labs/documentation).

This repository is **not the public documentation repository**.
It exists separately because the MegaETH protocol is not yet audited; visibility of this repository is restricted independently of the public docs site.
Do not copy or move content from here into the `documentation` repository, and do not assume this repo's visibility matches that repo's.

Specification content lives under `specs/`, written in GitBook-flavored Markdown.
For GitBook syntax, custom blocks, configuration, and best practices, follow the [official GitBook skill file](https://raw.githubusercontent.com/GitbookIO/public-docs/refs/heads/main/skill.md).
The writing rules for spec content itself — page structure, normative language, accuracy requirements — live in [`specs/AGENTS.md`](specs/AGENTS.md); read it before editing anything under `specs/`.

## History

This specification originated in the [mega-evm repository](https://github.com/megaeth-labs/mega-evm) (`docs/spec/`), covering only MegaEVM, System Contracts, and Network Upgrades.
It was briefly staged as a `spec/` folder inside the `documentation` repo, and briefly laid out flat at this repo's root, before settling on this repository with content nested under `specs/` — mirroring how `documentation` nests its content under `docs/` and keeps project tooling at the root.

Do not describe this specification as "mirrored from mega-evm/docs" or reference the earlier transitional layouts as if they were still current.
This repository, with content under `specs/`, is the canonical source.

## Scope

This is the **formal MegaETH specification** — the complete, normative definition of MegaETH's verifiable behavior.
It covers the entire set of behaviors that define a correct MegaETH node: block structure, sequencing, state representation, state sync, stateless validation, the L1↔L2 rollup and settlement pipeline, EVM execution, system contracts, and protocol upgrades.
It targets protocol implementers, auditors, and anyone who needs to verify or reproduce MegaETH's exact behavior.

**Scope principle**: if a behavior affects whether a node produces correct outputs given the same inputs, it belongs in this spec.
A useful test for the boundary: if two independent implementations that are indistinguishable to any external observer (same blocks, same signatures, same on-chain effects) could use entirely different internal mechanisms for something, that something is implementation, not spec.
Sequencer high-availability machinery (e.g., a Raft cluster) fails this test and is out of scope; the authority rule for who may sign a block does not, and is in scope.

**MegaETH does not specify a protocol-level CL/EL boundary.**
Unlike vanilla OP Stack and Ethereum, where the consensus-layer/execution-layer split (and the Engine API between them) is a real protocol interface because independently developed clients on each side must interoperate, MegaETH's sequencer is not decomposed this way for protocol purposes: whether an implementation internally separates block-building from execution, and how the two communicate, is invisible to every other node.
Treat "the sequencer" as a single opaque role that produces signed mini-blocks and EVM blocks; do not write MUST rules for an internal component boundary (rollup node ↔ execution engine, or any other CL/EL-shaped split) unless a page can show a genuine cross-implementation interoperability need for that specific boundary.

**Applying the test to a specific behavioral rule you're about to write.**
Before writing a MUST rule, ask: does any validity or verifier-checked rule elsewhere in this spec actually depend on this behavior happening, or happening a particular way?
If yes, it's in scope — write it.
If no, ask a second question before concluding it's out of scope: does varying this behavior between implementations change something _consensus-visible_ (part of a committed root, part of what another node's state transition depends on), even though no verifier independently re-derives or checks it?
Be skeptical of your own answer here — it is tempting to justify keeping a rule by asserting the reference behavior itself "is what makes the protocol interoperable" (e.g., an early draft of the Sequencing page argued mini-block sealing thresholds were in scope because they keep the realtime stream's fragment size predictable for consumers); check whether that claimed interoperability need is actually real and load-bearing elsewhere in the spec before accepting it, not merely plausible-sounding.
A rule is genuinely out of scope when a differently-policied implementation is not just unchecked but actually equally valid for every purpose this spec cares about — i.e., it is a policy layered on top of the protocol (an economic or UX choice, like when to refresh an oracle value, or an internal scheduling/architecture choice, like when to seal a mini-block or whether to split block-building from execution) rather than a piece of the protocol itself.
Writing "no validity rule re-checks this" in a Security Considerations paragraph is a signal to stop and explicitly re-run this test, not license to keep the MUST rule anyway.
When the answer is genuinely ambiguous, do not guess: surface the ambiguity to the user per the Source of Truth rule in [`specs/AGENTS.md`](specs/AGENTS.md#source-of-truth) instead of defaulting to writing everything as a MUST.

## Markdown Conventions

- **One sentence, one line.** Put each sentence on its own line in every Markdown file in this repository, including this file and `README.md`. This applies regardless of resulting line length. It keeps diffs reviewable — a one-sentence edit shows as a one-line diff, not a reflow of the whole paragraph.
- **Diagrams use Mermaid.** When a page needs a diagram, use a fenced ` ```mermaid ` block rather than an embedded image or ASCII art. GitBook renders Mermaid natively, and Mermaid source stays diffable and editable in place, unlike an image.

## Project Structure

```
.
├── .gitbook.yaml               # GitBook config (root: ./specs/)
├── AGENTS.md                   # This file — repo-level guidance
├── CLAUDE.md                   # Claude-specific overrides (symlink to AGENTS.md)
├── README.md                   # Repo-level README (structure, linting, contributing)
├── specs/                      # Specification source (GitBook-flavored Markdown)
│   ├── AGENTS.md                # Spec-wide writing rules (page structure, normative language, accuracy)
│   ├── SUMMARY.md                # Table of contents / navigation
│   ├── overview.md               # Spec landing page
│   ├── hardfork-spec.md          # Hardfork and spec versioning
│   ├── glossary.md               # Canonical protocol terminology
│   ├── protocol/                 # L2 node spec: blocks, sequencing, SALT, streaming, validation
│   ├── rollup/                   # L1↔L2 pipeline spec: derivation, DA, batching, bridges, fault proofs
│   ├── evm/                      # MegaEVM behavioral spec (published under megaevm/)
│   ├── system-contracts/         # System contract specs
│   └── upgrades/                 # Per-hardfork (Rex*) delta pages
├── .claude/skills/write-spec/  # Skill for writing/updating spec pages
├── .claude/skills/review-spec/ # Skill for reviewing spec pages
└── .sisyphus/plans/            # Planning docs (gitignored, not committed)
```

## Version Control

The main branch is `main`.
Prefer making changes via PRs on GitHub once the repo has real reviewers; early bootstrap commits may land directly on `main`.

### Branch naming convention

The naming convention for git branches is `{developer}/{category}/{description}`, where:

- `{developer}` is the (nick)name of the developer.
- `{category}` should indicate the type of modification, e.g., `spec`, `fix`, `ci`.
- `{description}` is a short description of the changes (a few words, hyphen-separated).

Example: `william/spec/add-sequencing-page`, `alice/fix/wrong-gas-constant`.

## Workflows

### Editing the specification

1. Edit or create Markdown files under `specs/`.
2. Read [`specs/AGENTS.md`](specs/AGENTS.md) for the writing rules, and use the [`write-spec` skill](.claude/skills/write-spec/SKILL.md) — it has the full page templates, style reference, and terminology discipline rules.
3. Update `specs/SUMMARY.md` if adding or removing pages.
4. Commit source changes only.

### Reviewing the specification

1. Use the [`review-spec` skill](.claude/skills/review-spec/SKILL.md) — it holds the ranked review criteria, the attack method, the findings-register format, and the rule for tracking the author's concurrent edits by file hash.
2. Keep the register under `.sisyphus/plans/<review-name>/` (gitignored); the review edits `specs/` only for structure-and-presentation fixes, as exact replacements that change no rule, and applies design changes only after the author decides them.
3. Log specification-versus-implementation discrepancies in the register's implementation register, never as specification defects, and surface them to the author.

### Committing changes

When requested to commit changes, first review all changes in the working tree, regardless of whether they are staged.
Only commit source Markdown and configuration changes.
Keep commit messages simple — no co-author information or "generated with" footers.

### Creating PRs

When a PR creation is requested, the agent should:

1. Check if the repo is on a branch other than `main`; if not, create and checkout a new branch and inform the user.
2. Commit all source changes.
3. Push to the remote.
4. Use the `gh` CLI tool to create a PR with a `Summary` section at the top of the description.
