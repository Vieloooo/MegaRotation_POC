---
name: write-spec
description: Writes and updates pages of the MegaETH Specification, this repository's entire content. Use when authoring a new spec page, filling a spec stub, updating a spec page for a hardfork or behavior change, or when a task involves normative (MUST/SHALL) protocol documentation. Suitable for delegation — a subagent given one spec page and this skill has everything it needs.
---

# Write Specification Pages

Write or update MegaETH Specification pages in this repository: $ARGUMENTS

Parse the arguments to determine scope. Accepted inputs:

- A page path (e.g., `specs/protocol/blocks.md`) — write or update that page.
- A section (e.g., `specs/rollup/`) — work through the pages of that section.
- A change description (e.g., "Rex7 changes oracle detention") — update every page the change touches, plus the corresponding `specs/upgrades/` page.

<!-- TODO(scaffold): This skill is being built up as spec-writing proceeds. Sections marked TODO are placeholders — fill them with lessons learned from writing real pages. -->

## The Spec Flavor

This entire repository is unlike a typical documentation site.
It optimizes for **concision and completeness**, in that order, and deliberately sacrifices readability for them:

- Every behavior is defined exhaustively — every corner case, every failure mode, every charging point. If the spec doesn't say what happens, implementers will guess differently.
- No narrative, no motivation-first buildup, no "for example" scaffolding in normative text. State the rule once, precisely, and move on.
- No developer guidance, tips, or "what this means for you" framing — that belongs in the separate public `documentation` repository's developer docs, never here.
- Normative language per RFC 2119/8174: uppercase MUST / MUST NOT / SHALL / SHOULD / MAY, only in a page's behavioral sections and its Security Considerations section.
- Readers are protocol implementers and auditors. Density is a feature; a page that reads slowly but resolves every ambiguity is a good page.

This repository is also **not public** — it holds pre-audit protocol detail, separated from the public `documentation` repo for that reason. Do not copy its content into the public repo, and do not assume anything written here is safe to quote outside it.

## Page Types

| Type                                      | Where                                                      | Template                                                                                                                                    |
| ----------------------------------------- | ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| Full spec page (MegaETH-native subsystem) | `specs/protocol/`, `specs/evm/`, `specs/system-contracts/` | Abstract → Motivation → one or more behavioral `##` sections → Constants → Rationale → Security Considerations → Spec History               |
| Delta spec page (OP Stack modification)   | `specs/rollup/`                                            | Same template; the behavioral sections state the OP Stack baseline (link to the exact specs.optimism.io section) and define only the deltas |
| Upgrade page (per-hardfork record)        | `specs/upgrades/`                                          | Summary → What Changed (previous/new behavior per change) → Developer Impact → Safety and Compatibility → References                        |
| Overview page                             | `*/overview.md`                                            | Abstract → Specifications (grouped links, with any out-of-scope notes after the list) → optional Spec History                               |

The full page-template rules (frontmatter, section order, normative-language rules, formatting preferences) live in [`specs/AGENTS.md`](../../../specs/AGENTS.md) — read it before drafting if you haven't already.

## Workflow

### Phase 1 — Research

<!-- TODO(scaffold): refine as we learn which sources answer which sections; add per-page source pointers. -->

1. Read the target page (or stub) and its section overview to understand scope and the pages it cross-links.
2. Consult the knowledge base (`/knowledge-base <topic>`) for architecture context and source pointers before reading code.
3. Identify the implementation sources of truth. Primary repos by section:
   - `specs/protocol/` — mega-reth (blocks, sequencing, state sync), salt (state, witness), stateless-validator. Note: MegaETH does not specify a protocol-level CL/EL boundary (see [Common Pitfalls](#common-pitfalls)), so how mega-reth and mega-optimism internally communicate is not itself spec content — read them as one combined "sequencer" source, not as evidence of a rollup-node/execution-engine protocol interface.
   - `specs/rollup/` — mega-optimism, mega-op-contracts, mega-kailua, mega-eigenda(-proxy); plus the OP Stack baseline at specs.optimism.io.
   - `specs/evm/`, `specs/system-contracts/` — mega-evm.
4. For delta pages, pin the exact OP Stack spec section being modified and read it first.

### Phase 2 — Draft

1. Read [references/op-spec-style.md](references/op-spec-style.md) — the OP Stack spec style baseline: sentence templates, formatting devices (validity-rule bullets, encoding blocks, per-function MUST lists), glossary and delta conventions, and the table of points where MegaETH deliberately diverges from OP style.
2. For any page defining a byte-level encoding (RLP, wire format, witness format) or a value type with a non-trivial validity predicate, also read [references/eip-style.md](references/eip-style.md) — EIP's `rlp([...])` notation, type-alias tables, and the grammar-then-worked-example pattern; use it alongside, not instead of, op-spec-style.md.
3. Follow the template for the page type above, ordered per [`specs/AGENTS.md`'s Progressive Disclosure rules](../../../specs/AGENTS.md#progressive-disclosure) — dependency order, explicit forward references, no borrowed sibling-page vocabulary.
4. One sentence per line.
5. Every numeric value becomes a named constant in the Constants table; no magic numbers in formulas.
6. Specify the charging/failure lifecycle for every cost and every rule: when it applies, and what happens on failure.
7. Follow [Terminology Discipline](#terminology-discipline) below for every defined term.

### Phase 3 — Verify

1. Verify every constant, address, formula, and behavioral claim against the implementation source — never copy from other docs without checking.
2. If the implementation contradicts the intended behavior or existing spec text, STOP and surface the discrepancy to the user — never silently resolve it.
3. Check every term used against [Terminology Discipline](#terminology-discipline): same term used the same way as other pages, every term defined somewhere, first use per section linked.
4. Update [`specs/SUMMARY.md`](../../../specs/SUMMARY.md) if pages were added or removed.
5. Run `mise run lint` and fix findings.

## Terminology Discipline

A term used loosely across pages is a spec bug, not a style nit — an implementer who reads "sequencer" on one page and "block producer" on another cannot tell if they're the same thing without guessing.

- **Keep terms consistent across pages.** Once a page establishes the name for a concept (e.g. "mini-block", "sequencer key", "compute gas"), every other page MUST reuse that exact term. Do not introduce a synonym for something already named elsewhere in the spec, and do not silently rename an existing term while drafting a new page — if the existing name is wrong, raise it, don't fork it.
- **Every term MUST be defined somewhere, and the definition MUST be linked, not re-explained.** Two cases:
  - MegaETH-specific terms are defined once in [`specs/glossary.md`](../../../specs/glossary.md) (or, for a term that is really a full spec concept rather than a short definition, on the spec page that owns it, e.g. `sequencerAt` on `specs/system-contracts/sequencer-registry.md`). Do not paraphrase the definition inline on a second page — link to the one place that owns it.
  - Terms MegaETH inherits unchanged (EIP names, Ethereum/OP Stack concepts like "state root" or "deposit transaction") are defined externally — link the EIP, the OP Stack spec section, or the Yellow Paper rather than restating the definition.
- **A term has exactly one canonical (bolded) definition site: the page whose concept it actually belongs to** — usually wherever its full significance is explained, not just wherever it happens to be mentioned first in reading order. Every other page that uses the term links to that definition instead of bolding and re-defining it a second time, even if that other page needed the term earlier.
- **Link the first use of a term per section, not just per page.** In a long page, a reader may jump straight to a `###` subsection via an anchor and never see the page's opening paragraph — so re-link a term's definition on its first occurrence within each major (`##`/`###`) section, even if the same page already linked it once higher up. Subsequent uses within that same section don't need to be re-linked.
- **Transitional exception:** while `specs/glossary.md` and other not-yet-migrated pages are still stubs, a term's real definition may temporarily live in the mega-evm-hosted glossary (or, for content not yet migrated from the `documentation` repo's brief `spec/` staging, that old location). Link it there with an absolute `https://docs.megaeth.com/spec/glossary#...` URL, and flag in your summary that the link needs to switch to a relative `glossary.md#...` link once that content migrates. Do not leave a term undefined just because its home page hasn't been migrated yet.
- **Anchor renames break links silently.** If you rename a `##`/`###` heading, grep the whole repo for links to its old anchor (`#old-heading-text`) before moving on — `mise run lint` catches broken anchors via lychee, but only if you run it.

## Conventions

- Concept pages describe the **latest stable spec only**; historical behavior goes in Spec History / upgrade pages; unstable (not-yet-activated) behavior goes in `<details>` blocks.
- The latest spec name is implicit — no "as of RexN" qualifiers in main content; the `spec:` frontmatter field declares the version.
- This repository is self-contained: it does not link to the public `documentation` repo's user/developer docs. Links go inward (other pages here) or outward to external baselines (OP Stack specs, EIPs, Yellow Paper) — see [references/op-spec-style.md](references/op-spec-style.md) for how OP Stack does the latter.

<!-- TODO(scaffold): add a "Delegation" section — how to split a section across parallel subagents, what context each needs, and how drafts are reviewed. -->

## Common Pitfalls

**Writing a full MUST-rule section for something that's actually implementation policy, not protocol.**
It's easy to over-specify: a behavior the reference sequencer/node happens to do gets written up as a normative rule with MUST language, a Rationale paragraph, and a Security Considerations note — all without checking whether any verifier or validity rule elsewhere in the spec actually depends on it.
Before writing a MUST-heavy subsection, run the [scope principle](../../../AGENTS.md#scope) test explicitly, including the follow-up question for behavior no validity rule re-checks: does varying this between implementations change something _consensus-visible_, or is it a policy layered on top of the protocol that any implementation could do differently with zero effect on interoperability?
A real example from this repo: an early draft of `specs/protocol/sequencing.md` specified in detail _when_ and _how often_ a sequencer must inject an oracle-refreshing system transaction — MUST rules, a window-segmentation definition, a Rationale paragraph, a Security Considerations note.
None of it was actually checked by any validity rule (the page's own Security Considerations said so), and any refresh policy — including never refreshing at all — produces an equally valid, equally reproducible block.
That entire behavior was implementation policy (a UX/economic choice of the reference sequencer), not protocol; the page should only ever have specified the one genuinely in-scope fact: who is authorized to _sign_ an oracle system transaction, since that's a security boundary a verifier does check.
The tell to watch for: if you find yourself writing "no validity rule re-checks this" or "a sequencer using a different value would still produce a valid block" in Security Considerations, stop and re-run the scope test on the whole subsection before finishing the page, not after — don't let the disclaimer coexist with a MUST rule it just undermined. If the answer is genuinely ambiguous, surface it to the user rather than guessing, per [`specs/AGENTS.md`](../../../specs/AGENTS.md#source-of-truth).

**Assuming a CL/EL-style protocol boundary that MegaETH doesn't have.**
Vanilla OP Stack and Ethereum specify a consensus-layer/execution-layer split — the rollup node and the execution engine, talking over the Engine API — as a real protocol interface, because independently developed clients on each side must interoperate.
It's easy to import that framing into a MegaETH page by habit (mega-reth and mega-optimism _are_ separate processes in today's architecture, and the OP Stack Engine API docs are a natural-looking source to cite), but MegaETH does not specify a protocol-level CL/EL boundary: whether an implementation splits block-building from execution at all, and how the two halves talk if it does, is invisible to every other node.
A real example from this repo: the same draft of `specs/protocol/sequencing.md` from the pitfall above also specified an "Engine API Payload Submission" section — payload-identifier semantics, a sealing-deadline constant, a pruned-response contract — treating the rollup node and execution engine as two independently-implementable protocol components.
Once the sequencer was correctly treated as a single opaque role (see [`AGENTS.md`](../../../AGENTS.md#scope)), that entire section had nothing left to specify: no other node ever observes that internal boundary, so it came out entirely, not just trimmed down to a signer-identity rule the way the oracle section was.
Read mega-reth and mega-optimism together as one "sequencer" implementation source, not as evidence of a cross-implementation protocol interface, unless a page can show a genuine reason a third party would need to independently implement one side and plug it into the other.
