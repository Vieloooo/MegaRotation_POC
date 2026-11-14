# Specification — Writing Rules

This file governs everything under `specs/`. Read the repo root [`AGENTS.md`](../AGENTS.md) first for repo-level context.

This directory is the **formal MegaETH specification** — the complete, normative definition of MegaETH's verifiable behavior.
It is not limited to the EVM; it covers the entire set of behaviors that define a correct MegaETH node: block structure, sequencing, state representation, state sync, stateless validation, the L1↔L2 rollup and settlement pipeline, resource metering, system contracts, oracle services, and protocol upgrades.
It targets protocol implementers, auditors, and anyone who needs to verify or reproduce MegaETH's exact behavior.

**Scope principle**: If a behavior affects whether a node produces correct outputs given the same inputs, it belongs in this spec.
A useful boundary test: if two independent implementations that are indistinguishable to any external observer (same blocks, same signatures, same on-chain effects) could use entirely different internal mechanisms for something, that something is implementation, not spec.
Sequencer high-availability machinery (e.g., a Raft cluster) fails this test and is out of scope; the authority rule for who may sign a block does not, and is in scope.

## Page Structure

Every page MUST include YAML frontmatter with a `spec` field — the latest stable spec that this page's main content describes (e.g., `Rex3`).
This field is the single source of truth for which spec version the page reflects.
When the latest stable spec changes, update this field and the page content together.

The value `TBD` is permitted, and means that the page's subject is not activated on any network and therefore has no stable spec yet.
A page carrying `spec: TBD` MUST also carry the page-level unstable warning described under [Spec Versioning](#spec-versioning), so that the declaration in the frontmatter is visible to a reader of the rendered page.
The field is replaced with the spec's name when the behavior activates, together with the removal of that warning.

**Exception**: Upgrade pages under `upgrades/` do not need a `spec` frontmatter field.
Their spec association is conveyed by the page title and filename.

Every page that defines behavior MUST follow this section order:

```
# Page Title
1-2 sentence abstract of what this page specifies.

## Motivation
The problem statement: what problem exists that this spec solves.
Describes the concrete failure modes or limitations that necessitate this behavior.
This section explains WHY the spec exists, not how it works.

## <Behavioral Section 1>
## <Behavioral Section 2>
...
The normative behavioral definition, one top-level `##` section per logical component.
Do not wrap these in an umbrella "## Specification" heading — the whole page is already understood to be normative, so that heading adds nesting without adding information.
All behavioral rules use MUST/MUST NOT/SHALL/SHOULD/MAY per RFC 2119.

## Constants
Table of all named constants with values and descriptions.
Every constant referenced in the behavioral sections above MUST appear here.
Placed after those sections so readers encounter the behavioral rules first
and can reference constants as needed.

## Rationale
Design decisions: why this specific solution over alternatives.
Each decision is a named paragraph explaining the trade-off.
This section explains WHY specific choices were made.

## Security Considerations
What could go wrong if this spec is implemented incorrectly or incompletely.
Concrete attack vectors, economic risks, or safety invariants that implementers must preserve.

## Spec History
Links to upgrade pages showing how this behavior evolved across specs.
```

**Sections may be omitted** when they genuinely don't apply (e.g., a glossary page has no Constants or Motivation).
But for any page that defines behavioral rules, the full structure SHOULD be followed.
Security Considerations SHOULD be included on any page that defines gas accounting, resource limits, state mutation, signing/authority rules, or economic rules, but it is not required.
It MAY be omitted for pages that are purely structural (e.g., glossary, overview indexes).
When omitted deliberately (not forgotten), state so explicitly rather than silently dropping the section — e.g. "This page has no security considerations."

### Spec History Format

For **overview pages**, the Spec History section SHOULD be a simple list of specs linking to their upgrade pages, each with one short sentence summarizing the main change.
Do not use tables or detailed changelog breakdowns there.

Example:

```markdown
## Spec History

- [MiniRex](upgrades/minirex.md) — Introduced the initial MegaEVM execution model.
- [Rex](upgrades/rex.md) — Revised gas forwarding and storage-gas semantics.
- [Rex1](upgrades/rex1.md) — Fixed detained compute-gas reset behavior.
```

For **concept pages**, the Spec History section MAY summarize what changed at each spec if that helps explain the evolution of the behavior.

Rules for the Spec History section:

- Only list specs that changed behavior relevant to this page. Do not list specs that inherited behavior unchanged (unless noting "No changes" is clarifying).
- Link each spec name to its upgrade page under `upgrades/`.

**Overview-page exception**: overview/index pages that summarize and organize links to authoritative subpages SHOULD omit Motivation and Rationale, using a lighter structure: Abstract → Specifications (grouped links, with any out-of-scope notes after the list) → optional Spec History.
The explicit-omission note required above (and under [Security Considerations Section Rules](#security-considerations-section-rules)) does not apply to sections this lighter structure already omits by design.

## Progressive Disclosure

A page MUST be ordered so a reader going top to bottom never needs a concept the page hasn't introduced yet.
This applies within a page's behavioral sections, and within a single section's subsections, not just at the page-structure level above.

- **Order by dependency, not by importance.**
  If section B's rule reads a value section A defines, A comes first — even when B is the more central mechanism.
  (E.g., a section defining a data structure's fields belongs before the section defining the algorithm that reads those fields on every step.)
- **Don't borrow a sibling page's vocabulary before this page needs it.**
  A shape/overview section should describe its subject in terms this page itself defines.
  When a full explanation depends on a concept that belongs to a different page or a later section (a cryptographic scheme, an encoding format, a mechanism not yet introduced), state the plain structural fact and link forward for the mechanism and the "why" — don't pull in that other page's terms just to make this section's explanation feel complete.
- **Preview multi-part sections.**
  A section covering more than one sub-topic in sequence SHOULD open with a one- or two-sentence roadmap naming each part in the order it appears, linked to its heading, so the reader knows what's coming.
- **A heading is a claim; keep it accurate.**
  Do not name a section for the full outcome of a multi-step process when the section only performs part of it (e.g. a section that resolves _which container_ a key maps to should not be named as if it also resolves the key's exact position within that container, if that position is only assigned by a later section).
- **Fold in, don't spin out.**
  A short corollary that is tightly coupled to one specific rule (true only as a consequence of that rule, not an independent topic) belongs as a continuation of that rule's own paragraph, not as its own subsection.
  A subsection heading is a claim that the topic is big enough to stand alone.
- **Don't duplicate a list that exists elsewhere on the page.**
  If two sections would enumerate the same sub-pages or sub-topics, keep the enumeration in one place and have the other reference it.
- **Match sibling presentation.**
  Parallel procedures on the same page (e.g. several operations of one algorithm) should share the same structure — if one is written as a bulleted step list, its siblings should be too.
- **Simulate a first-time reader when restructuring.**
  Read the page top to bottom as if seeing it for the first time, and flag every term used before it has been defined on this page or explicitly deferred ("specified next in ...", "see ... below").
  A forward reference is fine when it is explicit about being one; a term simply assumed is not.

## Accuracy

- **Verify constants against source code.**
  Every numeric value (gas limits, storage gas bases, detention caps, contract addresses) MUST be verified against the implementation source (`mega-evm`, `mega-reth`, `mega-optimism`, `salt`, `mega-kailua`, `mega-eigenda`, etc.) before writing or updating.
  Do not copy values from other documentation pages without cross-checking.
- **Attribute behavior to the correct spec.**
  If a behavior was introduced in Rex3, reference Rex3 — not Rex4, even if Rex4 inherits it.
  Each spec's contribution must be precisely scoped.
- **Do not invent guarantees.**
  This specification MUST NOT introduce constraints or behavioral rules that are not verified against the implementation.
  If the implementation does not enforce something the spec text is about to claim as guaranteed, say so plainly instead — do not paper over the gap.
  If uncertain, surface the ambiguity to the user rather than guessing.

## Tone & Language

- **Normative and precise.** Use MUST, MUST NOT, SHALL, SHOULD, MAY per RFC 2119 when defining behavior. Every behavioral rule in a page's behavioral sections MUST use normative language.
- **Exhaustive.** Cover every corner case. Readability may be sacrificed for completeness.
- **No developer guidance.** Do not include "tips", "best practices", "how to use", or recommendations like "Use `eth_estimateGas`" or "Use transient storage". That belongs in the public documentation repository's developer docs, not here.
- **No user-facing language.** Do not address the reader as "you" in a page's behavioral sections. Do not explain "what this means for you". That belongs in the public documentation repository's user or developer docs, not here.
- **Self-contained.** This specification never links to the public documentation repository's user or developer docs. It may link to external references (EIPs, the Ethereum Yellow Paper, OP Stack specs at specs.optimism.io) and to other pages within `specs/`.
- **No implementation details.** Do not reference specific code patterns, function names, or implementation strategies (e.g., `spec.is_enabled(MINI_REX)`). Describe the required behavior, not how to implement it.

## Behavioral Section Rules

### Normative Language

- **Uppercase only.**
  Per RFC 8174, only UPPERCASE keywords (MUST, SHOULD, MAY, etc.) carry normative weight.
  Lowercase "must", "should", "may" are plain English with no special meaning.
  The formal BCP 14 boilerplate lives on the [Specification overview page](overview.md) — individual pages do not repeat it.
- Use "A node MUST..." for required behavior.
- Use "A node MUST NOT..." for prohibited behavior.
- Use "SHOULD" only when non-compliance is acceptable in defined circumstances.
- Do not use normative keywords outside a page's behavioral sections and its Security Considerations section.
  Motivation, Rationale, and Spec History use plain English.
- Descriptive prose (background, context) does not require normative keywords.

### Constants

- Every numeric value used in a page's behavioral sections MUST be defined as a named constant in the Constants table.
- Do not embed magic numbers in formulas — reference the constant name.
- Each constant row MUST include: name, value, and a one-line description.

### Formulas

- Express formulas as inline code blocks: `` `total_gas = compute_gas + storage_gas` ``.
- Define every variable immediately after the formula.
- For complex logic, use pseudocode in fenced code blocks.

### Edge Cases

- State edge cases explicitly as normative rules (e.g., "For state that does not yet exist, the node MUST...").
- Do not leave behavior undefined — if the spec doesn't say what happens, implementers will guess differently.

### Charging Lifecycle

- For any cost or fee, specify WHEN it is charged: before execution, at the opcode, or post-execution.
- Specify what happens on failure: is the cost consumed, refunded, or rolled back?

### Spec Versioning

- Main content in concept pages MUST describe the latest stable spec's behavior only.
  Previous spec behavior belongs in the Spec History section or upgrade pages.
  Unstable spec behavior MUST be placed in `<details>` blocks, never in main prose or tables.
- **The latest spec is implicit.**
  Do not mention the latest spec name in the Specification, Constants, or Motivation sections.
  The `spec` frontmatter field declares which spec the page describes.
  Readers should be able to read the main content without encountering "as of Rex3" or "in Rex3" qualifiers.
  The Rationale section MAY reference spec names when explaining historical design decisions.
- Wrap unstable (not-yet-activated) spec content in `<details>` blocks with a clear label (e.g., "SpecName (unstable): ...").
- Unstable content MUST still use normative language within the `<details>` block.
- **Wholly unstable pages carry a page-level warning instead of `<details>`.**
  The `<details>` requirement above applies to unstable behavior appearing on a page whose main content is stable, where the block's job is to mark off the exception from the stable surroundings.
  A page or section whose entire subject is unstable has no stable surroundings to be marked off from, and instead carries a page-level `{% hint style="warning" %}` stating that the behavior is unstable and not activated on any network, paired with `spec: TBD` in its frontmatter; its body is not wrapped in `<details>`.
  Wrapping such a page block by block would collapse the whole page behind toggles, which distinguishes nothing and leaves the page unreadable.
- **Glossary exception**: glossary entries for unstable-spec terms do not use `<details>` blocks.
  Instead, mark them with inline text (e.g., "_(SpecName, unstable)_") at the start of the definition.

## Motivation and Rationale Section Rules

### Motivation

- Describe the concrete problem: what breaks, what is underpriced, what attack becomes possible.
- Use specific numbers where possible (e.g., "base fee of 0.001 gwei", "up to 10 billion gas per block").
- Do NOT describe the solution — that is the job of the page's behavioral sections.

### Rationale

- Each design decision is a **named paragraph** starting with bold text (e.g., "**Why `base × (multiplier − 1)` instead of `base × multiplier`?**").
- Explain the trade-off: what was considered, what was rejected, and why.
- Reference historical changes where applicable (e.g., "MiniRex used X, Rex changed to Y because...").

## Security Considerations Section Rules

This section exists because EIP-1 makes Security Considerations a blocking requirement for any Ethereum specification, and RFC 2119 §7 warns that "the effects on security of not implementing a MUST or SHOULD may be very subtle."

- Describe **concrete** risks, not generic warnings.
  State what goes wrong: underpriced operations, denial-of-service vectors, state corruption, economic exploits, equivocation.
- Frame each risk as a consequence of incorrect implementation.
  "If a node fails to charge storage gas on contract creation, an attacker can exhaust state storage at compute-gas cost only."
- Do not repeat the page's behavioral sections.
  Those sections say what MUST happen; Security Considerations says what breaks if it does not.
- Name the invariants that the spec preserves.
  "This spec preserves the invariant that total detained gas never exceeds the block gas limit."
- If no security considerations apply, state so explicitly: "This page has no security considerations."
  Do not silently omit the section — that is ambiguous (forgot vs. none).

## Upgrade Page Rules

Upgrade pages under `upgrades/` are the authoritative record of what changed at each spec.
They complement concept pages: concept pages describe the current behavior, upgrade pages describe the delta.

### Required Structure

Every upgrade page MUST follow this structure:

```markdown
---
description: <SpecName> network upgrade — <one-line summary>.
---

# <SpecName> Network Upgrade

This page is an informative summary of the <SpecName> specification.

## Summary

2-4 paragraphs: what changed, what problem it solves, developer impact.

## What Changed

### <Change Name>

#### Previous behavior

- Precise description of behavior before this spec.
- Include concrete values, formulas, or rules that were in effect.

#### New behavior

- Precise description of behavior introduced by this spec.
- Include the new values, formulas, or rules.

### <Next Change Name>

...

## Developer Impact

What contract authors, integrators, and tooling authors need to know.

## Safety and Compatibility

Backward-compatibility boundaries, failure-mode differences.

## References

Links to the implementation repo, related EIPs, or other specs.
```

### Precision Requirements

- Every "Previous behavior" section MUST state the exact prior values, formulas, or rules — not just "it was different."
  A reader should be able to implement the previous spec from the "Previous behavior" text alone.
- Every "New behavior" section MUST state the exact new values, formulas, or rules.
- Do not merge changes so aggressively that the previous/new mapping becomes unclear.
  Each behavioral change gets its own subsection under "What Changed."
- If a change fixes a bug in a prior spec, state what the buggy behavior was and what the corrected behavior is.

## What Belongs Here

- Block and mini-block structure, sequencing rules, state (SALT) representation, state sync, stateless validation — the L2 node itself.
- The L1↔L2 rollup and settlement pipeline: derivation, data availability, batch submission, deposits, withdrawals, L1 contracts, fault proofs — specified as deltas over the OP Stack specifications where MegaETH modifies standard OP Stack behavior.
- EVM behavioral definitions (gas costs, opcode semantics, resource limits).
- System contract specifications (addresses, interfaces, execution semantics).
- Oracle service specifications (storage layout, timing guarantees, gas detention).
- Hardfork/spec progression and per-upgrade behavioral deltas.
- Glossary of protocol terms.

## What Does NOT Belong Here

- Developer tips or best practices → developer documentation, in the public `documentation` repo.
- Code examples showing how to use a feature → developer documentation.
- User-facing explanations → user documentation, in the public `documentation` repo.
- Integration configuration → developer documentation.
- "Why should I care about this?" framing → developer documentation.
- Implementation-specific code patterns or API references → developer documentation.
- Recommendations ("Use X instead of Y") → developer documentation.
- Sequencer high-availability internals, broadcast-tree topology, deployment/nginx/RPC-gateway details — implementation and operations concerns, not verifiable behavior. See the scope principle above.
- Engine API interaction between a rollup node and an execution engine, or any other consensus-layer/execution-layer split. Unlike vanilla OP Stack and Ethereum, MegaETH does not specify a protocol-level CL/EL boundary — treat "the sequencer" as a single opaque role unless a page can show a genuine cross-implementation interoperability need for that internal boundary. See [`AGENTS.md`](../AGENTS.md#scope)'s scope principle.

## Source of Truth

The user is the ultimate source of truth.
This specification is the canonical written specification of MegaETH's verifiable behavior, but user instructions override both this spec and the implementation when there is ambiguity or conflict.

The implementation repositories (`mega-evm`, `mega-reth`, `mega-optimism`, `salt`, `mega-kailua`, `mega-eigenda`, `mega-eigenda-proxy`, `mega-op-contracts`, `stateless-validator`, and others) are side channels for verification only.
Agents MUST inspect the relevant implementation to confirm whether this specification matches the current implementation, but implementation code is not the final authority.

If an agent finds or suspects any discrepancy between user intent, this specification, and implementation behavior, the agent MUST NOT silently resolve it.
Instead, the agent MUST surface the discrepancy clearly and ask the user to confirm the intended behavior before changing the spec text.

## Linking Rules

See the [`write-spec` skill's Terminology Discipline section](../.claude/skills/write-spec/SKILL.md#terminology-discipline) for the full rule on keeping terms consistent, defined, and linked. Summary:

- **Link glossary terms on first use, per major section.** The first mention of a MegaETH-specific term within each `##`/`###` section SHOULD link to its glossary entry (or the spec page that owns the concept).
- **A term has exactly one canonical (bolded) definition site: the page whose concept it actually belongs to** — usually wherever its full significance is explained, not just wherever it's first mentioned in reading order.
  Every other page that uses the term links to that definition instead of bolding and re-defining it a second time, even if that other page needed the term earlier.
- **Link spec names to their upgrade pages.**
  When a spec is mentioned by name (e.g., MiniRex, Rex, Rex3), link it to the corresponding upgrade page under `upgrades/`.
- **Anchor targets must be headings.**
  Any content that is the target of a `#fragment` link MUST be a markdown heading (`##`, `###`, etc.), not bold text or other inline formatting.
  GitBook generates anchors only from headings. When renaming a heading, grep for every existing link to its old anchor and update them.
- **Cross-link to OP Stack specs with absolute URLs** (e.g., `https://specs.optimism.io/protocol/derivation.html`) for delta pages under `rollup/`.
- Use relative links with `.md` extensions for pages within `specs/` (e.g., `[Rex5](../upgrades/rex5.md)`), not absolute site URLs — those are only needed for the transitional exception in the `write-spec` skill (linking a term whose home page hasn't migrated yet).
- **Cross-references MUST be rich, not bare.**
  Never link with generic text like "here" or "this page" — link text is either the term itself (per Terminology Discipline) or a short phrase naming what the destination actually specifies (e.g. "the [state sync](protocol/state-sync.md) protocol's fragment validity rules", not "see state sync").
  When a rule on the current page depends on, is constrained by, or is enforced by another page, say so explicitly inline at the point of dependency and link the exact section that defines it — not just the page, and not only as a "See Also" list at the end.
  A reader following the spec's link graph should be able to reconstruct why a claim is true without re-deriving it themselves.

## Formatting Preferences

- Use tables for structured data (gas costs, opcode lists, resource limits, constants).
- Use unambiguous table values. Write "Unlimited", "No limit", or "N/A" — never use bare dashes ("—") which are ambiguous (not applicable? unknown? unlimited?).
- Use `<details>` blocks for unstable (not-yet-activated) features.
- Use `{% hint style="info" %}` sparingly — only for non-normative notes that help implementers understand design intent. Never for developer tips.
- Use `{% hint style="warning" %}` for unstable spec warnings.
- Do NOT use `{% hint style="success" %}` in spec pages — it implies developer guidance.
- Do NOT use `{% hint style="danger" %}` for normative rules — normative rules belong in plain prose with MUST/MUST NOT. Reserve `{% hint style="danger" %}` only for deprecation notices.
- **One sentence, one line.** Put each sentence on its own line, regardless of resulting line length — see the repo root [`AGENTS.md`](../AGENTS.md#markdown-conventions) for the full rationale.
- **Diagrams use Mermaid.** Use a fenced ` ```mermaid ` block for any diagram (state machines, sequence flows, pipeline stages); do not embed images or ASCII art.
