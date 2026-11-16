# Source-first acceptance expectations for feature 03

Candidate to inspect: e9c15e04bc83b335668f3ecd837367aa87344a89.
Sources read before candidate implementation/reports: root AGENTS/DESIGN and regional-bft-conformance; v2 handoff/setup and relevant v2 §§3,10.2; v10 architecture; frozen task-contract-mock-plan-execution-v2 §§1–7 and mock-plan-public-interface-v1.

Acceptance is for the authorized pure/shared 03 implementation, not I1 or 15-worker integration. Required evidence:
1. Exact final candidate SHA in independent verification and simulator runs, clean/unchanged candidate, pinned BFT/Commonware/toolchain and tracked source identity.
2. M1/M2 production BE checked canonical encodings and errors, immutable old I1 wire/hash compatibility, independent fixed expectations committed before implementation; no LE proposal substituted as production evidence.
3. Real cryptographic 15-member registry/PoP validation with sorted identity participant mapping, invalid point/namespace/key/duplicates rejected. Final quorum proof adaptation belongs to 04 and is not claimed by PoP success.
4. Static schedule/Plan/descriptor/cid validated against approved source and config; actual anchor/reference distinction, domain separation and checked range arithmetic.
5. Pure deterministic execution with well-formed invalid transfer filtering, malformed input atomic rejection, canonical state, actual metadata/DEP and checked arithmetic.
6. Merge on actual native parent with padding pre-root before current Begin/DEP, exact roots/hashes/Kept, WAIT for missing and REJECT known invalid input, bounded expansion; no partial output or clock-driven tail fill.
7. Explicit Finalize then independent reader delivery, actual visible contiguous prefixes, independent control coverage WAIT, conflict rejection without state change and finite fixture errors.
8. Every original 03 gate covered by real-library tests/reference vectors on exact final SHA, adequate raw logs/hashes; clearly scoped ignored I1 socket tests and no false claim for downstream final proof/Anvil/GFR/ResumeCut/runtime effects.

This checklist is fixed before candidate implementation and existing verification reports are inspected. Review may accept existing reproducible evidence without redundant heavy compilation; any discovered substantive missing gate is not excused by test count.
