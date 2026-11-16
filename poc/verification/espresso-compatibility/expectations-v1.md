# Feature 16 independent expectations v1

2026-11-16, Asia/Shanghai. **Preparation PASS; implementation and official-run verdicts not yet issued.** These expectations and literal vectors were derived before any feature-16 candidate existed. The verifier read the pinned official source, v10/v2, the accepted 06 contract/API, and the feature-16 contract. No builder output was used to calculate expected values.

The named role is `bft_verifier`; `.codex/agents/bft_verifier.toml` configures `gpt-6-astra` / `xhigh`. This is configuration evidence, not an independently observed runtime model attestation. Workspace: `/home/jy/.codex/worktrees/d2be/megaroation_poc`; branch: `codex/poc-espresso-compatibility`; preparation baseline: `bcd30b0879e6c17afc177bfea76f7fee170cc069`. No build or simulation was run during preparation. Seeds: N/A, literal cases.

## Source-derived rules and adapter choices

- [v10 architecture §1.4](../../../formalization_v10/00_arch_zh_v0_1.md) defines absolute Espresso height, a complete ordered protocol-message list, nondecreasing public timestamp, and a watermark that follows a complete prefix. E-TS concerns the proposal instant under a public live assumption; reader receipt time cannot supply either fact.
- [v2 §§4.1/4.5/13.1](../../../formalization_v10/poc_design_zh_v2.md) separates wall, virtual, public and local time and treats official dev-node execution separately from deterministic main tests. [v1 §5.1](../../../formalization_v10/poc_design_zh.md) explicitly permits a trusted local API profile, without claiming independent HotShot finality verification.
- [06 contract §§2–3](../../docs/contracts/task-contract-publisher-espresso-v1.md) requires proposal/finalization virtual ticks and live-at-proposal in `FinalizedBlock`/PELG/`ReaderState`. The official profile lacks these facts. `Unknown` must remain unknown; full PE-LOG and PE-READER export must return `UnsupportedTimingEvidence`.
- Official release `20260918`, commit `fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171`, is the source authority. The official source archive read during preparation has SHA-256 `a973cc8954eb6b7252c60a2f8810f2a0434ffdf8b025aa35f7940b7cd2dced3f`. Source working copy: `/tmp/espresso16-source-fd221e8e4b/`.
- [Pinned API routes](https://github.com/EspressoSystems/espresso-network/blob/fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171/crates/espresso/api/src/axum/routes.rs) define `/v1/availability/block/{q}`, `/leaf/{q}` and `/transaction/{q}/{i}/noproof`. [Node API state](https://github.com/EspressoSystems/espresso-network/blob/fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171/crates/espresso/node/src/api/state.rs), `get_transaction_by_position`, uses the payload iterator's flat `nth(i)`. This supports the native flat index, not submission order or transaction-hash sorting.
- [Pinned query types](https://github.com/EspressoSystems/espresso-network/blob/fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171/hotshot-query-service-types/src/availability.rs), `BlockQueryData::new`, computes `num_transactions` from the source payload iterator. `LeafQueryData::new` binds QC and leaf then removes the payload. A leaf alone does not supply a complete list. This adapter trusts the local official API for the count and per-position list; it does not repeat QC/VID or block-commitment verification.
- [Pinned header implementation](https://github.com/EspressoSystems/espresso-network/blob/fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171/crates/espresso/types/src/v0/impls/header.rs) serializes protocol 0.3 as `{"version":{"Version":{"major":0,"minor":3}},"fields":...}`. `QueryableHeader::timestamp()` returns seconds. The task choice is checked `seconds × 1_000_000`, retaining any other raw fields separately.
- [Pinned transaction type](https://github.com/EspressoSystems/espresso-network/blob/fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171/crates/espresso/types/src/v0/v0_1/transaction.rs) encodes payload bytes with base64. Official [submit documentation](https://docs.espressosys.com/network/api-reference/sequencer-api/submit-api) distinguishes submission acknowledgement from sequencing. Official [node documentation](https://docs.espressosys.com/network/api-reference/espresso-api/node-api) defines reported H as latest q+1 even when earlier data is missing; [availability documentation](https://docs.espressosys.com/network/developer/espresso-api/availability-api) permits temporary missing data and leaves duplicate-hash occurrence selection unspecified. The pinned source takes precedence if moving docs differ.

The source identity, trusted-API projection, capacity bounds and EC16-AUDIT-1 digest are adapter choices frozen in [the task contract](../../docs/contracts/task-contract-espresso-compatibility-v1.md), not Espresso consensus rules or 03 canonical object encodings. The native flat index and compact selected-namespace `EspressoPosition.message_index` are separate values. Unknown time blocks complete PE export but does not block this explicitly scoped local API experiment.

## Literal vector baseline

[vectors-v1.json](vectors-v1.json) contains invented, explicitly non-cryptographic API-shaped fixtures and hand-specified expected outcomes. The fixture namespace/transaction-table bytes agree with their listed transactions; their header commitments and QC values are placeholders and must never count as cryptographic or official-node evidence. Schema-minimal fixture headers exercise the adapter's selected fields rather than upstream Rust deserialization.

The four blocks have public `(q, theta_us)` values `(0,0)`, `(1,7000000)`, `(2,7000000)`, `(3,8000000)`. Their native transaction counts are `0,4,1,1`. At q1, native index0 is foreign namespace9; indices1/2/3 contain namespace160016 payloads `41`, `41`, and empty bytes. Their selected message indices are `0,1,2`, preserving native indices `1,2,3`. q2 is a complete nonempty foreign-only block whose selected publication list is empty. q3 contains `42` at both native and selected index0. The fixture codec is `0xEC160001`.

The independently calculated source ID is:

```text
EC16:c3e6b516cf687dbc813d30c0363c2d428f1fec992b6e806edc147ae007cfeac6
```

The JSON stores the exact source preimage, complete EC16-AUDIT-1 byte strings as hex, and these literal SHA-256 results. They were computed with Python `struct.pack`/`hashlib` directly from the frozen recipe and the independent fixture values, not from candidate code.

| Complete prefix block count | Audit byte count | SHA-256 |
| --- | --- | --- |
| 0 | 91 | `69e86a7ce44440ec9de8adc396f247d545d47b977c8bed192000aa00d7722a9a` |
| 1 | 140 | `34b56467ef402f332dcd741be4bda6531fe5ce821c490b0a371f977cad7e8d0d` |
| 2 | 638 | `7cbf120ae32522adecc17943402c367444f0092c7398999e79956f2e279fbd5f` |
| 3 | 792 | `386e795687ea2f091742b780e671e4ae06d33a8fb97cf82aef2664d74efa1cc2` |
| 4 | 952 | `98a1a83ae5f930ef101e6babcec2a28475c855d94c48f9ecf759e5ff3b4c6f39` |

Only the contiguous complete prefix contributes to the digest. Pending q2 with q1 missing cannot change the digest beyond q0. Wall times, replay pauses, reader/session identity, manifest hash and raw evidence references do not appear in these bytes. Source identity and all native transaction contents do appear, including foreign-namespace content and repeated payloads.

## Acceptance matrix and state expectations

| ID | Independent expected behavior |
| --- | --- |
| V01 | Exact release/image/protocol/API/namespace binding; run ID and q0 genesis bind source identity. Mixing records from another source is rejected. |
| V02 | Submit ACK alone leaves publication pending and prefix empty. A matching publication is reported only from an actually complete source block. |
| V03 | q is absolute source height, starting at0. H is a fetch hint. Deliver q2 then q0: tips are None then0; q2 is not readable until q1 arrives. |
| V04 | Exact positions `0..N−1`; count0 requires actual block+leaf. Same bytes at different positions remain distinct. Missing index waits; conflicting or extra index rejects atomically. |
| V05 | Header-only, leaf-only, 404, timeout and missing tx cannot advance prefix/watermark. Supplying the missing material later can advance it. A complete readerA does not fill readerB's gap. |
| V06 | Equal public timestamps are accepted. Decrease and checked multiplication overflow reject without replacing history. `18446744073709` seconds converts to `18446744073709000000`µs; the next second cannot fit u64. |
| V07 | Repeated semantic JSON is idempotent; key order/whitespace/receipt metadata do not create public differences. Any consumed raw-object or public-content conflict preserves the prior prefix and first receipt. |
| V08 | Filter namespace160016 only after all N native transactions are present. Keep both native indices and compact selected indices. Complete foreign-only N>0 yields a valid empty projection. |
| V09 | Two offline runs of one capture, including differing replay wall pauses, yield exact same public objects, audit bytes and digest. This says nothing about deterministic execution of the original node. |
| V10 | Wrong q/index/hash/inner-vs-outer namespace/version, bad JSON/base64, corrupt/missing referenced files, invalid paths including symlink escape, unknown schema and truncated records fail without fabricating empty data or partially installing a rejected operation. |
| V11 | Proposal instant, finalization instant and live-at-proposal remain Unknown; full PELOG and PEReader exports are explicitly Unsupported. No 0, false, receipt time or θ substitution. |
| V12 | Only task-local endpoints and owned processes; external redirects/URLs do not generate requests. Record the native/container execution method and independent ports. |
| V13 | A later verified candidate must perform actual official-binary local submission, enumerate q0 through inclusion, retain raw leaf/block/transaction records, requery on a new connection, and retain an actual missing-data probe. Two distinct publication payloads plus one exact repeat are required. Repeated submission has no assumed occurrence count. |
| V14 | Fixture tests, setup probe and formal official-run evidence remain separately labelled. Missing real execution leaves G-API INCONCLUSIVE even if every fixture passes. |

Capacity equal/one-above cases are recorded explicitly in the vector file: per native payload65,536 (including foreign namespaces), native count1,024, selected PELG-envelope sizing1,048,576 using 06's size validator,4,096 distinct q including pending, cumulative decoded native payload64MiB with no duplicate recount, raw item/manifest16MiB, and unique-file bundle total256MiB. Exceeding these local bounds is `Capacity`, not a claim of invalid Espresso consensus. At exact raw-file/bundle size limits, unrelated schema/hash checks still apply.

For the selected envelope boundary, a69-byte source ID yields116 bytes of fixed PELG overhead. Sixteen selected messages with fifteen lengths65,536 and one length65,292 occupy exactly1,048,576 bytes; increasing the last by1 exceeds the limit. This is only a size calculation; it does not authorize constructing fake PELG timing fields.

## Review scope and next gate

Contract review: **PASS for the declared trusted-local-API profile**, after separating native/selected indices, allowing absent selected namespaces in complete nonempty blocks, excluding observation metadata from audit bytes, and stating capacity accounting. G-PE-TIMING remains explicitly unsupported; G-CRYPTO is outside this profile, not passed by fixtures. Whether the reported compatibility gap is acceptable for complete feature16 acceptance remains the coordinator's decision.

Next: freeze these preparation artifacts, implement in the assigned paths, commit a named candidate, then run an independent harness against that exact revision. Formal tester execution must use the independently reviewed revision, fresh artifact directories, Rust1.95.0/locked dependencies, two build jobs, and the host heavy-work flock. This document is not a candidate implementation verdict or a real-node acceptance report.
