# Feature 03 commander acceptance audit

2026-11-16 (Asia/Shanghai). Verdict: **ACCEPT the complete authorized feature 03 component scope at `e9c15e04bc83b335668f3ecd837367aa87344a89`.** No additional feature-03-owned completion gap or actionable finding was identified in this acceptance audit. The commander may count this as one accepted feature after recording its decision; this review does not update commander state or main.

Candidate worktree: `/home/jy/.codex/worktrees/1c43/megaroation_poc`, branch `codex/poc-mock-plan-execution`. HEAD and worktree were checked before/after and remain the exact candidate and clean. No candidate files, frozen expectations, main branch, other worktrees or repository configuration were changed.

## Authority and review order

The source-first expectations were written to `source-first-expectations.md` before implementation and existing verification results were inspected. Sources were root AGENTS/DESIGN and regional-bft-conformance, pinned PoC v2 §§3/10.2, v10 architecture §§3.1–3.4, the approved `task-contract-mock-plan-execution-v2.md` §§1–7, `mock-plan-public-interface-v1.md`, relevant pinned Regional BFT mapping and Commonware 2026.10.0 primitive interfaces.

The complete 03 scope is real registry/PoP, identity mappings, static authorization, checked public objects/codec/hash, pure execution/metadata/Merge/native hash and the explicitly finalized, independently delivered L1 fixture. Contract v2 explicitly assigns native final-proof/3-of-4 quorum adaptation to 04, controlled runtime to 01/02, real L1 ABI/receipts to 09, complete replay/lifecycle to 05/07 and ResumeCut to 12/13. Those are downstream integration obligations, not reasons to keep this authorized 03 feature unaccepted.

## Evidence checked

`audit.py` and `evidence-audit.json` record **79 passing integrity/identity/count/reference checks**, zero failures. These are evidence-audit checks, not 79 additional protocol test cases.

| Check | Verified evidence |
| --- | --- |
| Exact named candidate | Independent final verifier and Tester manifests both bind e9c15e0, clean before/after and unchanged HEAD. |
| Prior review to final seal | `git diff e30f5ed… e9c15e0 -- poc/crates poc/Cargo.toml poc/Cargo.lock poc/proto poc/vendor regional_BFT_sim` is empty. The seal adds reports/reference checks plus a primitive-recipe formatting-only change, and both roles reran at the sealed SHA. |
| Raw results | Independent final stdout sums to 32 passed, 0 failed, 0 ignored. Tester workspace stdout sums to 52 passed, 0 failed, 11 ignored. The feature subtotal is auth6 + recipe1 + core13 + M1 7 + codec5 = 32; existing workspace contributes 20. |
| Test attribution | All named cases are accounted for. The 11 omitted cases are one I1 helper plus ten existing socket/process cases; none are 03 conformance tests and none are counted as passed. |
| Artifact integrity | Every verifier `files` entry and Tester `artifacts` entry exists and matches its recorded SHA-256; Tester file sizes also match. The Tester links the exact independent-final manifest digest. |
| Inputs and versions | Frozen vector and additional input hashes match the checked-in files. The contract/v2/lock hashes match recorded sources. All nine Commonware entries in the workspace lock are 2026.10.0; curve25519-dalek5.0.0 is pinned. BFT gitlink is unchanged 691cfc14…. |
| I1 compatibility | Compared with ada48a5, `types/src/lib.rs` adds only `pub mod domain;`; existing helpers/exports remain. Proto, vendor and BFT pin have no change. Real fixed I1 Grant/Ack/hash compatibility vectors passed in the recorded final runs. |
| Independent reference reproduction | Reran the four immutable Python reference scripts using Python 3.12.8. All returned 0 and stdout hashes equal the final verifier's recorded hashes. No Rust build or heavyweight test was repeated because the sealed same-SHA evidence is intact. |
| Actual component values | Reader progress, state-changing h1–h3 Merge and 4096/4097 bounds match exactly between verifier and Tester when keyed by distinct case name. Ordering within each case's blocks/transactions was preserved. |

Independent final manifest:

`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03/artifacts/verification/full03/e9c15e04bc83b335668f3ecd837367aa87344a89/run-manifest.json`

SHA-256 `a25ee38b4ea5eefd36e4302eedca04e5cc282f00008e537a7803b98ccc8c4da7`.

Tester final manifest:

`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03/artifacts/simulation/e9c15e04bc83b335668f3ecd837367aa87344a89/run-manifest.json`

SHA-256 `47da8a35da3f22b4295916865c6d30b1f66db5be05ed9c625a71c47f072ebfa1`.

## Source-to-code/coverage spot checks

- Contract v2 §4.1 real PoP is implemented at `poc/crates/authorization/src/registry.rs:60–74`: actual Commonware G2/G1 decode and `verify_proof_of_possession::<MinSig>` with the frozen namespace. The actual identity-key-ordered participant map is used for signer indexes. The final six auth tests and upstream primitive recipe include real positive and invalid-key/namespace/point/duplicate cases. This is not a default-true verification marker.
- v10 Apply and frozen contract §5.1 are implemented in `poc/crates/core/src/execution/mod.rs`: decode/validate before cloned-state mutation, DEP credits first, ordered transfers with correct nonce/balance/overflow skipping, self-transfer and sparse state behavior. Fixed real-library tests compare resulting state, Kept and roots against independent values.
- v10 VP3/Merge and contract §5.3 are reflected in `poc/crates/core/src/merge/mod.rs:35–120`: use actual anchor native hash, expand only through the physical tip, check pre-root before current Begin/DEP, and emit native blocks using DEP, Kept and post-root. The final tests include missing/known-invalid separation, no partial output and 4096/4097 expansion bounds. Earlier WAIT-before-known-invalid issues were fixed before the reviewed e30 candidate, and the production code is unchanged in e9.
- Contract §6 uses actual typed finalization receipts with private fields (`poc/crates/core/src/fixture/mod.rs:188`) and explicit source finalizer (`:206`) plus independent reader state (`:151`). Tests exercise late finalization, different reader prefixes, control coverage, foreign-source receipts, duplicate/conflict atomicity and fixture exhaustion.
- `poc/crates/types/src/domain/objects.rs:974` declares `FinalVerifier::check` with no implementation/default body. Typed `RegionalResult`, `ReplayPosition` and lifecycle objects therefore do not falsely claim native proof validation or canonical downstream replay encoding.

## Audit execution notes

The first local audit attempt used the shell's older Python, which lacked `tomllib`; no candidate execution occurred. It was rerun using the documented `/home/jy/miniconda3/bin/python3` (3.12.8).

The first audit compared the outer arrays of independent test-case evidence positionally and observed a mismatch. Inspection showed only two independent case records appeared in a different outer order; all fields and all per-case ordered lists were identical. The corrected comparison requires three unique case identities and compares the full records by identity. `evidence-audit-initial-list-order.json` preserves that initial result. This is not normalization of a protocol event trace, and no candidate result or frozen input was modified.

Reproduce this audit:

```sh
/home/jy/miniconda3/bin/python3 /tmp/commander-feature03-acceptance/audit.py
```

The script writes only this temporary audit directory. Existing raw Rust runs used the shared heavy-job lock and exact locked/offline commands recorded in their manifests; the audit did not need to take that lock for hashing and short Python references.

## Downstream release

Prefer the complete accepted code/interface SHA **`e9c15e04bc83b335668f3ecd837367aa87344a89`** for 02/06/07/09/12. It exports packages `poc-types`, `poc-authorization`, and `poc-core`. Older staged references remain useful: M1 `0e72dc247de22afdc8d6681713b3d15f4aa05262`; M2 types+independent codec `fadde7e84b5eee9dc528b2c318127a7c2c5721df`. Do not mistake either smaller milestone for the full implementation.

- 02: checked public pid/region/member/signer identities and opaque cid/config; transport authentication and durable effects remain 02's work.
- 06/07: Plan/descriptor/block objects, ResultKey and pure execution/Merge. Require actual source/anchor and native final-proof checks from the appropriate adapters; no permissive stand-in should become production validation.
- 09: exact L1Position/L1BlockRef and FinalizedL1Reader semantics. The 03 fixture proves neither real EVM receipts nor finalized Ethereum history.
- 12: shared Apply, DEP, actual metadata and native hash on D's own parent/state. Do not copy G state or force regional scheduled origin onto independent D derivation.

This acceptance is finite, bounded component evidence: three committees of five, fixed P0 mocks, deterministic vector seed=null and public crypto seeds 303000+pid/303100+pid. It is not a safety/liveness proof, performance claim, full I1 completion or an end-to-end 15-worker handoff result. Actual combined commits still require integration checks when consumers are wired.
