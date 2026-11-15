# Independent corrective NATIVE-WITNESS review of 050c59b

**Accepted for the bounded NATIVE-WITNESS slice.** Candidate `050c59b9e446e0aaf295f24e3d3a15357b00c443` passes all 20 independent tests and all 82 prior prototype regressions: **102 passed, zero failed**. The three original semantic counterexamples and the failed-sync uncertainty gap are closed within the stated test scope. No production source was edited by the verifier.

The governing contract includes Lead precision amendment2 at `cee5a3821ad6341bf80972f6fe758936d1cd9c74`; final contract SHA256 is `29676b5721d29b6bf991940b2a16612de7dc090b6f32ef79531b0a9cb95633cb`. New expectations were derived from that amendment before inspecting corrected source. The source basis remains Commonware 2026.10.0/upstream `92a2f68a0e131c8b69de41e84aec37da6280f212` and the primary rules recorded in the original review.

## Closed findings

| Original finding | Corrected rule and independent replay |
|---|---|
| Stale durable byte versions, P1 | `witness_ledger.rs:263` replays ordered observations, and line295 requires current provenance. The unchanged rehashed descendant citing obsolete A-prefix coverage after B-prefix SYNC now rejects with `witness disagrees with current durable byte versions`. Added A→B→A ordinary-write restoration proves old coverage cannot revive without a new barrier; same-value writes retain established coverage. Reused I/O identity mutants reject separately. |
| Artifact substring treated as frame, P2 | `witness_ledger.rs:301` parses the current complete logical prefix from offset 0 and requires an exact decoded frame. The original interior-digest Certification at offset 5 now rejects with `witness does not identify a current canonical journal frame`. No signature or record-identity rejection substitutes for the boundary assertion. |
| Recovery loses route authorization, P2 | `witness_recovery.rs:145` copies and continues the predecessor's intrinsic ledger. Original route 2 recovery is head-idempotent: sequence 10 and its hash remain unchanged. Generation 2 and3 tests preserve exact predecessor record/byte prefixes, remove audit snapshots from their expected paths, and still reopen/authorize using external minimum receipts. Existing route 2/route 3 authorizations append nothing; a new route 4 appends exactly one record. |
| Failed Sync does not latch uncertainty, P2 | `witness.rs:870` records Sync uncertainty; ledger replay preserves it across close/reopen. Both `sync` and awaited `start_sync` receive typed `Error::Closed` from the approved root-only diagnostic. The uncertainty observation commits before typed-result release. Old seals remain, a later same-generation successful sync creates no new witness, and fresh-generation NEW full sync restores eligibility for the complete accepted source. |

The Sync tests are explicitly **lost-sync-response injection after successful native physical sync**, not observed OS disk failure. They exercise error propagation, persistent uncertainty and recovery policy; they do not measure filesystem/hardware failure durability.

## Fixture changes and additional checks

The W9/W11 fixture now locates `GenerationStarted` and counts exactly nine NEW witnesses and one NEW full barrier after it. It also asserts nine historical witnesses and exact predecessor record/byte prefix preservation. New barrier IDs remain distinct from old IDs, complete history retains the same eligible payload set, and all original rejection/idempotence assertions remain in force. This is a representation update for the continued chain, not a weaker acceptance rule.

Three additional tests cover:

- Ordinary overwrite/restoration and monotonic selected-I/O identities, including recomputed record/head hashes.
- Both Sync APIs' tagged lost-response path, successful typed-error handling, persistence-before-release, same-generation uncertainty and new-generation recovery.
- Complete and pending recovery in generations 2 and3. Both just-started and after-new-evidence/before-completion ledgers reject recovered submission without changing their head, even when a prior generation had completed recovery. Complete descendants accept either valid predecessor minimum receipt. Audit directories are renamed into retained evidence, not deleted.

All earlier W1–W12 cases remain covered as described in `REVIEW-add79ab.md`: exact CRC prefix/slot/extent, valid slot fallback/ties, bounded canonical framing, context/PoP/foreign signer negatives, protected same-file reopen, incomplete live tail versus strict recovery, atomic explicit authorization, evidence cut points and cancellation, source-preserving NEW sync recovery, corruption/rollback cases, and the six actual native Journal seed/wall runs. Complete logical evidence, identities, raw files and recovery records remain equal for seeds 7/42/99 under wall pauses 0/2 ms.

## Reproduction and provenance

```text
python3 poc/verification/native-witness/run_acceptance.py
```

The launcher applies an external 60 s cap with 5 s kill grace to the locked offline full prototype suite, creates a fresh atomic output root, and records before/after hashes. Final artifacts are `/tmp/native-witness-independent-65l9bw76`; logs and provenance are `stdout.log`, `stderr.log`, and `run.json`. A durable summary is `candidate-050c59b-report.json` beside this report.

Key replay files under that root:

- `witness/stale-version-9/mutant-outcome.json`
- `witness/nonboundary-frame-18/mutant-outcome.json`
- `witness/recovery-auth-20/duplicate-recovery.json`
- `witness/ordinary-version-{7,8}/` for restoration/I/O-ID mutants
- `witness/lost-sync-response-{10,11}/diagnostic-scope.json`
- `witness/three-generations-26/lineage.json`, with pending-generation ledgers and preserved audit copies

Fixture SHA256: `e1640b62bb12883eebecca915e2dc4ef05ec3bdb67d3203716285ae89c37cf37`. Independent test-binary SHA256: `08668908b9faead02b35d938d00c53f8890a3abdfbacd5ade7464d76524e9d8d`. Candidate source/dependency hashes match before and after execution. Bootstrap formatting check and `git diff --check` pass. Original add79ab reports and failing raw artifacts remain unchanged.

No open finding remains within this bounded contract. This acceptance does not cover real OS failure injection, production-scale journal growth, native live engine signing/timing, resolver/relay transport, native finality or two-process replay. Offline fixture signatures establish no creation-time clock guard. Full I1b remains incomplete, and the lead must rerun the affected checks after integration before using the revision for experiments.
