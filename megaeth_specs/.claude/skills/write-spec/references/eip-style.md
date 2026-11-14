# EIP Style Reference

Distilled from a read of [EIP-1](https://eips.ethereum.org/EIPS/eip-1) (the meta-EIP defining the EIP format) and four encoding-heavy EIPs: [EIP-2718](https://eips.ethereum.org/EIPS/eip-2718) (Typed Transaction Envelope), [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559) (Fee Market), [EIP-2930](https://eips.ethereum.org/EIPS/eip-2930) (Access List), and [EIP-4844](https://eips.ethereum.org/EIPS/eip-4844) (Blob Transactions).

This is a second styling input alongside [op-spec-style.md](op-spec-style.md).
Where the two agree, treat that as reinforced.
Where they disagree, MegaETH's own template (see the divergence table in op-spec-style.md) still wins — this file exists to add patterns EIPs have that OP Stack's spec site does not, mainly around byte-level encoding rigor.

## Page anatomy

Required preamble fields, in order: `eip`, `title` (≤44 chars, title case), `description` (one sentence), `author`, `discussions-to`, `status`, `type`, `category` (Standards Track only), `created`, `requires` (optional).
Required section order: **Abstract** → **Motivation** (optional, omit if evident) → **Specification** → **Rationale** → **Backwards Compatibility** (mandatory if any incompatibility exists) → **Test Cases** (mandatory for consensus-affecting changes) → **Reference Implementation** (optional) → **Security Considerations** (mandatory) → **Copyright**.

- **Abstract** is a 2-4 sentence technical summary terse enough that reading only the abstract gives the gist — not a teaser, a compressed version of the Specification.
- **Specification** must be "detailed enough to allow competing, interoperable implementations."
- **Rationale** is its own section, segregated from Specification — matches the MegaETH template already.
- **Security Considerations** is mandatory; an EIP without one does not progress. It must name concrete risks and how they're addressed, not just assert safety.
- EIPs are markedly terser than OP Stack pages: EIP-2718 is 79 lines, EIP-2930 is 140 lines, EIP-4844 (the most complex example here) is ~440 lines. No glossary file, no TOC, no per-page background section — the Abstract does that job.

## Normative language

Same RFC 2119 discipline MegaETH already follows, independently confirmed here: EIP-1 states the RFC 2119/8174 boilerplate goes at the top of Specification, and explicitly forbids RFC 2119 keywords **outside** the Specification section.
Bold rendering (`**MUST**`) is common in older EIPs (2718, 1559) rather than plain caps.
MUST/SHOULD density inside Specification is actually low — most binding statements are carried by `assert` statements in pseudocode (see below), not by prose keywords.

## Encoding notation

**RLP list expression**: always a flat `rlp([field1, field2, ...])`, nested structures as nested `[...]`.
Field types are not tagged inline in the expression — a following prose sentence or short Definitions list glosses each field, e.g. (EIP-2718): _"`TransactionType` is a positive unsigned 8-bit number between `0` and `0x7f`... `TransactionPayload` is an opaque byte array."_

**Byte concatenation**: `||`, defined once on first use — "`||` is the byte/byte-array concatenation operator" — then used freely, e.g. (EIP-1559): `0x02 || rlp([chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas, gas_limit, destination, amount, data, access_list, signature_y_parity, signature_r, signature_s])`.

**Variable-length/nested fields** get a literal grammar plus a worked example immediately after, e.g. EIP-2930's access list:

```text
[[{20 bytes}, [{32 bytes}...]]...]
```

with `...` explicitly glossed as "zero or more of the thing to the left," followed by a concrete JSON-shaped example with real hex-shaped placeholder values.
This grammar-then-example pairing is cheap and worth doing whenever we define a wire/witness format.

**Type aliases table** (EIP-4844, for SSZ/complex value types) — a device not present in OP Stack style at all:

| Type            | Base type                                                       | Additional checks                                                              |
| --------------- | --------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `Blob`          | `ByteVector[BYTES_PER_FIELD_ELEMENT * FIELD_ELEMENTS_PER_BLOB]` |                                                                                |
| `KZGCommitment` | `Bytes48`                                                       | Perform IETF BLS signature "KeyValidate" check but do allow the identity point |

Use this shape whenever a type needs both a byte-layout definition and a validity predicate that doesn't fit a plain byte-width note.

## Constants tables

Two-column `Constant | Value`, `ALL_CAPS_WITH_UNDERSCORES` names in backticks, values given with explicit width/type wrappers where relevant: `` `BLOB_TX_TYPE` | `Bytes1(0x03)` ``.
No fork-history column — each table is a flat, single-version snapshot; a later value change lives in a new page, never edited into the old table (see Versioning below).

## Validity rules as code, not tables

EIPs do not use a named-verdict taxonomy (no `-> drop/future` tags like OP).
Validity is instead expressed as literal `assert` statements inside Python pseudocode, and the pseudocode itself is declared authoritative: _"Block validity is defined in the reference implementation below."_ Example (EIP-4844):

```python
assert len(tx.blob_versioned_hashes) > 0
for h in tx.blob_versioned_hashes:
    assert h[0] == VERSIONED_HASH_VERSION_KZG
assert tx.max_fee_per_blob_gas >= get_base_fee_per_blob_gas(block.header)
```

This is tighter than OP's parallel prose-rule-list-plus-illustrative-pseudocode: there is exactly one source of truth for the rule, so prose and code cannot drift apart.
Simpler checks stay as plain prose bullets, e.g.: _"There are an equal number of `tx_payload_body.blob_versioned_hashes`, `blobs`, `commitments`, and `proofs`."_

Formulas follow the same pairing: an inline backtick formula immediately backed by the exact pseudocode that computes it, e.g. (EIP-4844):

`base_fee_per_blob_gas = MIN_BASE_FEE_PER_BLOB_GAS * e**(excess_blob_gas / BLOB_BASE_FEE_UPDATE_FRACTION)`

paired with a full `fake_exponential` Python implementation right below it — never formula-only or code-only.
Rounding/edge-case conventions get a one-line prose aside directly before the code, e.g.: _"Note: `//` is integer division, round down."_

## Security Considerations register

Short, blunt, organized as one `###` subsection per concrete risk rather than narrative risk assessment, e.g. EIP-2930's `### Access list generation` and `### Transaction size bloating` — each names the risk in the heading, then argues in 1-3 sentences why it's bounded ("only a 10% initial discount... almost no cost to not bothering").

## Versioning: new document, not edited baseline

An EIP never edits an earlier Final EIP's text.
Extension happens via the `requires:` frontmatter field plus a prose cross-reference, e.g. EIP-4844 declares `requires: 1559, 2718, 2930, 4895` and its spec text says fields "follow the same semantics as EIP-1559."
The discipline worth stating explicitly: never rewrite the semantics of an already-published spec page in place; a behavior change is a new page (MegaETH already does this via `upgrades/` pages — this is confirmation, not a new rule).

## What this adds beyond op-spec-style.md

1. **`rlp([...])` list-expression convention** for RLP encodings — terser than OP's `++`-concatenation blocks, and the right notation specifically for RLP (as opposed to raw byte-concatenation formats, where `++` remains correct).
2. **Type aliases table** (`Type | Base type | Additional checks`) for value types that need both a byte layout and a non-trivial validity predicate — useful for SALT witness/proof types.
3. **Executable pseudocode as the sole normative statement** of a validity rule (`assert`-as-rule), rather than prose rule + illustrative pseudocode kept in parallel. Prefer this for any rule complex enough that prose and pseudocode could drift apart.
4. **Grammar-then-worked-example pairing** for variable-length/nested encodings — do this for every wire or witness format we define.
5. **Backwards Compatibility as a named, expected section** — not yet part of the MegaETH template; worth adding to the **Upgrade page** template specifically (MegaETH is hardfork-versioned, so this maps naturally onto `upgrades/*.md` rather than onto every concept page).
6. **Test Cases as a named section for consensus-affecting changes** — also not yet in the MegaETH template; worth adding as an optional section on upgrade pages and on pages whose Specification section defines exact byte encodings (inline input/expected-output pairs, no separate test-vectors repo needed).

## Deliberately not adopted

- EIP's low overall MUST/SHOULD density (most binding logic carried by `assert` pseudocode) — MegaETH's Specification sections should still state every rule in prose with RFC 2119 keywords even when pseudocode is also given, per the existing template; pseudocode supplements, it doesn't replace, the normative prose.
- Flat single-version constants tables with no fork-history column — MegaETH concept pages describe the latest stable spec only (matching this), but historical values still belong in Spec History, not silently dropped.
- CC0 copyright waiver, EIP numbering/editor process, `discussions-to` preamble field — process machinery specific to the EIP standardization pipeline, not applicable here.
