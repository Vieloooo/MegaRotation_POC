# OP Stack Spec Style Reference

Distilled from a deep read of `ethereum-optimism/specs` (rendered at [specs.optimism.io](https://specs.optimism.io/)): `protocol/derivation.md`, `exec-engine.md`, `deposits.md`, `batcher.md`, `withdrawals.md`, `messengers.md`, `system-config.md`, `fault-proof/index.md`, `fault-proof/stage-one/fault-dispute-game.md`, `root.md`, `background.md`, `glossary.md`, fork pages (`holocene/`, `isthmus/`, `jovian/`), and repo tooling config.

This is the styling baseline the MegaETH spec draws from.
The final section lists where MegaETH deliberately diverges — those rules win on conflict.

## Site organization

- Component-oriented TOC: pages map 1:1 to the software role an implementer builds (execution engine, rollup node, batcher, proposer, contracts).
- Directory = section (`protocol/`, `fault-proof/`, `experimental/`); each major subsystem gets an `index.md`/`overview.md` landing page before detail pages.
- `root.md` is a minimal front door; `background.md` is the non-normative primer and labels itself as such: "This document provides an overview of the protocol to provide context for the rest of the specification."
- Experimental/in-flight specs are quarantined in an `experimental/` section, cleanly separated from normative content.
- Pages are sized to the topic, not a template: `batcher.md` is 34 lines, `derivation.md` is ~1080. Neither is considered wrong.

## Page anatomy

Every page follows the same opening sequence:

1. `# Title` — noun phrase, sometimes with a "Specification" suffix ("L2 Chain Derivation Specification").
2. Table of contents (auto-generated with doctoc; GitBook generates ours, so we skip this).
3. Glossary link-reference block: `<!-- All glossary references in this file. -->` followed by `[g-derivation]: ../glossary.md#l2-chain-derivation` lines (derivation.md declares ~40 up front).
4. `## Overview` — prose that motivates and situates before any normative content.

Then: parameters/constants → data formats and encodings → architecture/algorithms → edge cases and error handling.
Heading nesting routinely reaches H4, occasionally H5 (`##### Bedrock, Canyon, Delta: API Usage`).
Sections that are cross-reference targets define a reference anchor right under the heading:

```markdown
### Batch Format

[batch-format]: #batch-format
```

## Normative language

OP uses RFC 2119 keywords **sparingly and inconsistently** — capitalized MUST is reserved for interoperability-critical constraints and bullet lists; long stretches are plain declarative present tense ("New frames for timed-out channels are dropped instead of buffered."), and lowercase "must" carries normative weight in prose. There is no BCP 14 boilerplate.
(MegaETH is stricter — see divergences below — but the _sentence templates_ are worth stealing.)

Recurring templates, verbatim:

- Setter triple (contract functions): "MUST revert if called by an address other than the owner." / "MUST update the batcher hash." / "MUST emit a ConfigUpdate event with the UpdateType.BATCHER type."
- Access control: "MUST only be triggerable by the ProxyAdmin or its owner."
- Data-object subject: "The first transaction MUST be a [L1 attributes deposited transaction][l1-attr-deposit], followed by an array of zero-or-more [user-deposited transactions][user-deposited]."
- Unconditional-effect note attached to the rule: "First, the balance of the `from` account MUST be increased by the amount of `mint`. This is unconditional, and does not revert on deposit failure."
- Prohibition: "Deposited transactions MUST never be consumed from the transaction pool."
- Component-actor SHOULD: "The execution engine SHOULD warn the user when the recommended version is newer than the current version supported."
- Implementation freedom: "A verifier may implement this differently, but must be semantically equivalent to not diverge from the L2 chain."
- Input preconditions: "These inputs must satisfy the following conditions:" followed by a numbered list.
- Security property (impersonal form): "It should not be possible to 'double spend' a withdrawal, ie. to relay a withdrawal on L1 which does not correspond to a message initiated on L2."

Subjects of normative sentences are the named component ("the rollup node", "the execution engine", "the batcher"), the data object ("the withdrawals field MUST be nil"), or the contract function (subject elided in bullet lists). Never "implementations MUST"; never "you".

Rationale is interleaved, not segregated: nearly every rule is followed by a "This is to ensure / This enables / to prevent X" sentence, e.g. "We select `0x7E` because transaction type identifiers are currently allowed to go up to `0x7F`. Picking a high identifier minimizes the risk that the identifier will be used by another transaction type."

## Formatting devices

**Constants** — a dedicated table, SCREAMING_SNAKE in backticks, with upgrade-history columns:

```markdown
| Parameter | Bedrock (default) value | Latest (default) value | Changes | Notes |
| `MAX_CHANNEL_BANK_SIZE`| 100,000,000 | 1,000,000,000 | [Fjord](…) | Constant increased with Fjord.|
```

Prose then references the table: "`MAX_CHANNEL_BANK_SIZE` is a protocol constant defined in the [Protocol Parameters table](#protocol-parameters)."

**Encodings** — `text` code blocks with `++` concatenation, then a "where:" list defining every field:

```text
frame = channel_id ++ frame_number ++ frame_data_length ++ frame_data ++ is_last

channel_id        = bytes16
frame_number      = uint16
```

"Where `uint32` and `uint16` are all big-endian unsigned integers." — followed by a per-field bullet gloss.
Byte layouts as markdown tables (`| 1 byte | 11 bytes | 20 bytes |`) or byte-range bullets ("Bytes `[28, 32)`: big-endian `uint32`, encoding …").
RLP/versioned formats as a two-column table keyed by version byte, each version with an `rlp_encode([...])` expression.

**Validity rules** — OP's most distinctive device, `condition -> verdict` bullets, each with an "i.e." gloss and inline rationale:

> - `batch.timestamp > next_timestamp` -> `future`: i.e. the batch must be ready to process.
> - `batch.epoch_hash != batch_origin.hash` -> `drop`: i.e. a batch must reference a canonical L1 origin, to prevent batches from being replayed onto unexpected L1 chains.

A "Definitions:" list precedes the rules; the list is explicitly ordered ("Rules, in validation order:").

**Formulas** — inline backtick code, not LaTeX, with a "Where:" list defining every symbol and precision caveats in prose ("big-int computation, result in Wei and `uint256` range").
LaTeX appears only for game-theoretic formalism (execution trace $(S_0,…,S_n)$, DAG $G=(V,E)$).

**Pseudocode** — short Python-flavored `text` blocks, with honesty notes about abstraction ("While the above pseudocode implies that all batches are known in advance, it is possible to perform streaming compression…").

**Contract pages** — full Solidity `interface` blocks (events + functions, inline comments), NatSpec reproduced only for functions needing deep explanation; then a `## Function Specification` section with one `### functionName` subsection per function: a prose lead-in plus a MUST-bullet list. Getters get one line ("Returns the address of the L1CrossDomainMessenger contract.").
Address constants inline in prose at first mention ("a simple predeploy contract at `0x4200000000000000000000000000000000000016`").
Storage layout specified only where consensus-relevant, with rationale attached.

**Multi-actor protocols** — a Definitions section first (one `###` per term, constants defined as terms), then mechanics; actors introduced with bolded role names ("**Challengers** and **Defenders**"); concrete walkthroughs use Alice/Bob; flows split by domain ("### On L2" / "### On L1") with numbered actor-action steps and failure branches inline ("If proof verification fails, the call reverts. Otherwise the hash is recorded to prevent it from being re-proven.").
No formal state-machine notation, no mermaid on normative pages; diagrams are static images.

**Invariants** — named informally in prose then referenced ("the `L2 time >= L1 time` invariant"); security properties as numbered "Key Properties" in impersonal should-not-be-possible form, with real-incident citations ("For reference, see [this writeup][polygon-dbl-spend] of a vulnerability of this type found on Polygon.").

**Notes** — `> **Note**` blockquotes for non-binding clarifications; labeled asides for audience targeting: "**Vocabulary note**: …", "Note for application developers: …", "Implementers note: …".

## Glossary conventions

Single file, ~70 entries, thematic H1 sections separated by `---`.
Entry = `##` heading + reference-anchor definition + 1–4 short paragraphs:

```markdown
## Sequencing Window

[sequencing-window]: glossary.md#sequencing-window

A sequencing window is a range of L1 blocks from which a [sequencing epoch][sequencing-epoch] can be derived.
```

- First sentence is the definition, "An X is a Y that Z" form.
- Define, then delegate: "Epochs can have variable size, subject to some constraints. See the [L2 chain derivation specification][derivation-spec] for more details."
- Disambiguation lives in the glossary ("This term _deposit_ is somewhat ambiguous… This section disambiguates all deposit-related terms.").
- Spec pages link glossary terms on essentially **every** significant use (not just first), via `[g-…]` reference handles declared at the top of the file.

## Hardfork / delta conventions

- Each fork: a directory with `overview.md` + delta pages only for components that change. The overview is a pure link index grouped by layer (Execution Layer / Consensus Layer / Smart Contracts), no prose.
- Delta pages name each change as an italicized proper noun (_Strict Batch Ordering_, _Fast Channel Invalidation_), then state what is removed/replaced/retained against the named baseline, linking the exact baseline section amended: "So the following changes are made to the [Bedrock Batch Queue](../derivation.md#batch-queue): — The reordering step is removed… — The other rules stay the same, including empty batch generation when the sequencing window elapses."
- Activation always gets its own section with precise trigger semantics: "The new batch rules activate when the _L1 inclusion block timestamp_ is greater or equal to the Holocene activation timestamp. Note that this is in contrast to how span batches activated in [Delta](…)."
- Upgrade transactions follow a rigid per-transaction template: purpose sentence → "A deposit transaction is derived with the following attributes:" → attribute bullet list → reproducible verification blocks with exact `cast`/`jq` commands and expected output, provenance pinned to a commit.
- Baseline pages absorb history inline with a temporal idiom ("Prior to the Canyon network upgrade, … After the Canyon network upgrade, …", "Starting with Holocene, …") and forward-reference fork deltas parenthetically. (MegaETH does this differently — see divergences.)
- Delta pages end with `# Rationale` and `# Security and Implementation Considerations` sections that permit "we" and candid uncertainty: "To the best of our knowledge, we haven't experienced this on OP Mainnet or other mainnet OP Stack chains yet."

## Register and sentence style

- Third-person declarative present tense for rules ("is dropped", "is derived with the following attributes").
- "We" is standard for design rationale ("We do not use the sender's nonce to ensure uniqueness because this would require an extra L2 EVM state read") and for algorithm narration ("In the _L1 Traversal_ stage, we simply read the header of the next L1 block."). Never second person.
- Terms are italicized on first definition, then reused verbatim as proper nouns; synonym collisions addressed head-on ("also referred to as the batcher").
- Honest hedging for known gaps: "(it may not be in rare engine-corruption recovery cases, this is being reviewed)"; candid scope disclaimers: "The exact P2P based sync is out of scope for the L2 specification."
- Medium sentences (15–30 words), short paragraphs (1–4 sentences); long logic chains go into bullet lists, not prose.
- External links: EIPs/RFCs always linked; source code and other specs pinned to commit URLs, sometimes to the line; reference implementations linked, never inlined ("A reference implementation of the deposit contract can be found in [OptimismPortal.sol].").

## Where MegaETH deliberately diverges

Our spec layer rules override OP style on these points; do not import the OP habit:

| Topic                  | OP Stack                                                                                | MegaETH spec                                                                                                                                              |
| ---------------------- | --------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| RFC 2119               | Loose: caps mostly in bullet lists, lowercase "must" normative in prose, no boilerplate | Strict: only UPPERCASE keywords are normative, BCP 14 boilerplate on the overview page, keywords only in Specification / Security Considerations sections |
| Page template          | Free-form, sized to topic                                                               | Fixed section order: Abstract → Motivation → Specification → Constants → Rationale → Security Considerations → Spec History                               |
| Rationale placement    | Interleaved after each rule                                                             | Segregated into the Rationale section (named-paragraph per decision); Specification stays pure                                                            |
| Hardfork history       | Absorbed inline into baseline pages ("Prior to Canyon… After Canyon…")                  | Concept pages describe the latest stable spec only; history lives in Spec History tables and `upgrades/` pages; unstable behavior in `<details>` blocks   |
| Fork organization      | Directory per fork with per-component delta pages                                       | Single upgrade page per spec under `upgrades/` with a What Changed (previous/new) structure                                                               |
| Line format            | Hard-wrapped ~100–120 chars, multiple sentences per line                                | One sentence per line                                                                                                                                     |
| TOC                    | doctoc-generated per page                                                               | GitBook generates navigation; no in-page TOC                                                                                                              |
| Notes/callouts         | `>` blockquotes, `> [!Note]`                                                            | GitBook `{% hint %}` blocks, sparingly; `<details>` for unstable specs                                                                                    |
| Spec versioning marker | None                                                                                    | `spec:` frontmatter field on every concept page                                                                                                           |
| Constants presentation | Parameter table with per-fork value columns                                             | Constants table (name, value, description) after the Specification section                                                                                |

## Worth adopting for the new Protocol / Rollup & Settlement sections

1. **`condition -> verdict` validity-rule bullets** with "i.e." glosses — ideal for fragment validity, batch acceptance, and witness verification rules.
2. **`++` concatenation encoding blocks with "where:" field lists** — for mini-block/fragment wire formats and the witness format.
3. **Definitions-before-mechanics** structure for multi-actor pages (fault proofs, withdrawals) and constants-as-defined-terms.
4. **Per-function MUST-bullet specification** for L1 contract delta pages.
5. **Delta framing idiom** for `rollup/` pages: name each delta, state removed/replaced/retained, link the exact OP baseline section amended, and pin the OP specs version referenced.
6. **Activation-semantics precision** — every behavior change states its exact trigger condition.
7. **Reproducible verification commands** for on-chain artifacts (deployed code hashes, addresses).
8. **Glossary anchor discipline** — reference-style `[g-…]` handles, define-then-delegate entries, disambiguation sections.
9. **Impersonal "It should not be possible to X" phrasing** for security properties, with incident citations where they exist.
10. **Explicit implementation-freedom grants** ("may implement differently, but must be semantically equivalent") wherever the spec constrains outputs, not mechanisms.
