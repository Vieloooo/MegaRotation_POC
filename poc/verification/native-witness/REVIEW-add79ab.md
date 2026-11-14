# Independent NATIVE-WITNESS review of add79ab

**Not accepted.** Candidate `add79aba803de747e47c37b9d1e7a1d6a057fef4` has three independently reproduced semantic failures and one source-confirmed uncertainty gap. All 81 previous tests pass. Of 17 new independent tests, 14 pass and three fail. No production source was changed by the verifier.

Contract: `poc/docs/contracts/task-contract-native-witness-v1.md`, SHA256 `26e9da2be69a5914dd13aa05141d2ad09c0b01ebb3663c5f830938a92487f86f`. Expectations were derived before candidate inspection from that contract and pinned Commonware 2026.10.0 (`92a2f68a0e131c8b69de41e84aec37da6280f212`), particularly native page-slot/CRC, Artifact/Vote/Certificate, Blob SYNC and voter/resolver ordering rules.

## Findings

1. **P1 — replay accepts superseded durable bytes for a new witness.** `poc/prototypes/i1b-runtime/src/witness_ledger.rs:316` selects referenced historical barriers; lines343–347 accept any matching byte. An actual native run writes page A, range-SYNCs A's prefix, overwrites it with page B, then range-SYNCs B's conflicting prefix. The real external minimum receipt is sequence4. A rehashed descendant adds only A's footer/extent barrier and a witness citing A's obsolete prefix. Anchored open accepts it. The contract requires current durable byte versions at witness creation and invalidation after overwrites; immutable already-sealed historical witnesses are a separate case. This can manufacture a durability claim that the retained barrier chronology contradicts. Repro: `w2_w10_rehashed_stale_durable_version_is_rejected` (`acceptance.rs:582`). Raw replay: `witness/stale-version-7/{witness,mutant,mutant.head,mutant-outcome.json}` under the artifact root below.

2. **P2 — evidence does not prove a journal frame boundary.** `poc/prototypes/i1b-runtime/src/witness.rs:267` validates a caller-selected offset, reconstructs its substring and canonically decodes that substring. It does not establish that the offset follows native framing from the beginning of the logical journal. A genuine signed Finalize fixture contains `04 02 01 01 01` inside its payload digest at logical offset5. The real decoder finds one Finalize. A rehashed descendant ledger claims a separate Certification at offset5, referencing the genuine full-page SYNC, and anchored open accepts it. The contract explicitly rejects substring evidence. This repro manufactures protected history; Certification has no wire output, so it does not demonstrate a forged signature or network publication. Repro: `w4_w10_rehashed_artifact_substring_is_not_a_journal_frame` (`acceptance.rs:1661`), raw `witness/nonboundary-frame-14/mutant-outcome.json`.

3. **P2 — fresh recovery forgets previously committed route authorizations.** `poc/prototypes/i1b-runtime/src/witness_recovery.rs:133` retains old eligible payloads but not submissions; line150 creates an empty active ledger. `witness_ledger.rs:481` consequently treats a previously authorized destination as new. Source generation already authorizes one payload to member2. Recovery preserves source bytes and builds a valid new witness, but its active ledger contains no inherited authorization. Calling `submit_recovered` for the same route increments the head from4 to5. The frozen stable authorization/history rule requires reuse without another authorization; this is independent of later transport retries. Repro: `w6_w9_recovery_preserves_committed_authorization_history` (`acceptance.rs:535`), raw `witness/recovery-auth-16/duplicate-recovery.json`.

4. **P2, source-only — selected Sync Err does not mark uncertainty.** `poc/prototypes/i1b-runtime/src/witness.rs:919` handles native Error outcomes, but lines920–925 mark only Write/Resize uncertain. A Sync/start_sync error leaves the section eligible for later ordinary sealing. The frozen conservative V1 rule requires fresh-generation recovery after a failed selected barrier; already-sealed history remains intact and the native Err remains a typed result. Public diagnostics expose evidence/release failures, not a native sync Err, so this branch has not been independently injected. A narrow injection or private unit regression is still required. This limitation is not a passing W3 claim.

The two semantic ledger forgeries recompute entry identities, canonical record hashes, chain links and protected head checksums. They retain the actual external receipt for the unchanged prefix. Their acceptance is not an integrity-check bypass caused by omitting hash updates.

## Executed coverage

| Gate | Independent evidence and result |
|---|---|
| W1 | Plain write/readability gives no witness; subsequent matching complete barrier does. PASS. |
| W2 | Exact CRC-prefix/slot/physical extent; unrelated blob; empty SYNC; full sync; dropped/aborted start_sync observers. Positive and missing-coverage negatives PASS. Superseded-byte mutant FAIL. |
| W3 | Protected overwrite/shrink/remove/fallback rejected before native mutation, including same-file reopen; source bytes retained. Sync Err uncertainty remains source-only OPEN. |
| W4 | Independent CRC32C and literal page/header/hash vectors; larger valid slot, tie0, fallback; bounds, zero/noncanonical/oversized/incomplete framing; physical/logical crossing; live complete-prefix/incomplete-tail acceptance and strict recovery rejection. PASS except the frame-boundary forgery. |
| W5 | Context/domain/epoch/section/local signer/PoP/distinct registry checks; certificate quorum excludes local member; typed wire mapping; Certification non-wire. PASS. |
| W6 | Explicit authorization only; mixed invalid destinations reject atomically; duplicate routes preserve current-generation head; no release before evidence commit; forensic/non-recovery submission rejects. PASS except inherited recovery authorization. |
| W7 | Before-ledger, after-record-sync, after-head-sync cuts; no successful result/Ack; explicit single complete batch promotion. PASS. |
| W8 | Cancel borrowed grant while root owns Ready result and evidence; repeated failed controls preserve custody; root controller panic drains evidence and suppresses consumer. PASS. |
| W9 | Public verifier-only recovery, fresh generation and NEW full native sync; exact wire/eligible IDs and raw files preserved; protected source corruption and target alias/existing-path rejection. PASS except authorization continuity. |
| W10 | Trusted minimum prefix accepts descendants, rejects coherent rollback/conflict; committed framing/checksum/head/truncation errors; incomplete-tail rule; header/layout corruption; five fully rehashed coverage/mapping mutants reject. Two additional rehashed semantic mutants expose findings1–2. |
| W11 | Actual native Artifact Journal with page crossing, buffered incomplete tails and partial-page rewrites; seeds7/42/99, wall pauses0/2ms. Full events, ledger records, identities, source/target raw bytes and recovery records compare equal within each pair. PASS. |
| W12 | All81 previous prototype tests PASS; source and dependency hashes unchanged. No resolver/relay behavior change or universal barrier claim. |

The registry literal independently obtained from the configured native keys/PoPs is `849458372215a66a4a252c576f9615776e3057ee040e85d753635d63fd9c0657`. The namespace, page, header, blob, barrier and witness literals were derived before implementation inspection and are asserted in the fixture.

## Replay and provenance

From the isolated checkout root:

```text
python3 poc/verification/native-witness/run_acceptance.py
```

The launcher uses `timeout --kill-after=5s 60s ./poc/scripts/bootstrap.sh test --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline --no-fail-fast -- --test-threads=1`, atomically allocates fresh artifact roots, preserves prior output, and records complete logs and before/after source hashes.

Final raw artifact root: `/tmp/native-witness-independent-i_57u6cd`. `run.json`, `stdout.log`, and `stderr.log` record exit101, 95 passing tests and three failing tests. Copied provenance: `candidate-add79ab-report.json` beside this report. Before/after source hashes match. Independent fixture SHA256: `a5daa4d32f688b23355736756db89aa6d4fa21db3fad740b65b7051c778f03cb`. Test binary SHA256: `4cdc45c0624b41eba7a376be609d89b4f0702f1c9cbdbed4a9421269db52f41e`.

Formatting and `git diff --check` pass. Only the new verifier directory and the approved Cargo test target were written. Builder-provided raw inputs were not used as expected outputs or modified.

This is bounded durability/witness/authorization verification. Offline signed fixtures do not establish native engine creation-time signing guards, admission, finality, real p2p submission/delivery, timing preservation or two-process replay. Overall I1b remains incomplete. Corrected production source must be named and the affected independent and integrated checks rerun before this slice can be accepted.
