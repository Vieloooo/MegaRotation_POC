# Independent contract review

2026-11-16, `bft_verifier`, GPT-6 Astra / xhigh. Result: **PASS for contract freeze**. No 07 implementation or builder explanation was inspected before this decision. This is not candidate or feature acceptance.

Reviewed frozen contract: `poc/docs/contracts/task-contract-viewer-gfr-v1.md`, SHA-256 `0a622d8504f48ad63305b917ee30da0d5e07f366512bd10018345ecedd5bfabb`.

The contract agrees with the frozen source-first expectations on public ordering, complete coverage, per-event atomicity, q_avail, current-key expiry, regular anchors/reference/fixed windows, HALT, independent readers and verified checkpoint restoration. Its unknown-frontier WAIT explicitly avoids promising a determined next event before the public order can be proved. Its publication envelope reuses canonical 03 objects and opaque native proof bytes. Fixture provenance remains distinct from native verification.

Two substantive issues were raised and resolved before freeze:

1. `FinalVerifier::check` returns bool and cannot itself communicate missing historical verification prerequisites. The new separate `VerificationBasis::require(descriptor, block) -> InputStatus<()>` must pass before `check`; WAIT/REJECT must propagate without a bool call. Candidate testing must use a counting verifier to prove this call boundary.
2. An absent verifier must not leave the verification profile unbound. `InitInput::Genesis.expected_verification` now fixes that immutable profile even when `finality=None`; any later binding/profile mismatch rejects. The label Native remains provenance only and cannot attest its own cryptographic correctness.

The crate-name typo and contradictory wording about multiple records within one event were also corrected. One event produces one ReplayRecord with ordered effects/lifecycle members.

**Erratum to the frozen source-first document:** the Manager lifecycle reference is **v10 03 §3.10 ApplyLifecycle**, specifically line 226, not §3.9. The source line and substantive material-before-allocation concern were correct. The original frozen file and its hash remain unchanged. Future RESUME lifecycle delivery is still an OPEN integration gate; the per-event batch decision is accepted as a boundary and is not evidence of implemented Manager behavior.

`interface-vectors-v1.json` adds exact independent contract encodings: 9 positive and 29 negative position cases, 9 comparator pairs, 18 lifecycle projection/id cases, 2 positive and 11 negative publication cases. `generate-interface-vectors-v1.py` uses Python standard integer encoding and SHA-256 only; it never imports production 07. Nested objects reuse the accepted 03 independent BE-v1 goldens, whose encoding-only registry caveat is retained. These vectors cover codec structure and comparison, not native proof verification or actual RESUME execution.

Candidate review must map all 40 frozen VG cases and contract V01–V23 to concrete evidence or explicit INCONCLUSIVE/deferred scope. V22/native and VG39/future RESUME cannot be counted as passed by fixture vectors.
