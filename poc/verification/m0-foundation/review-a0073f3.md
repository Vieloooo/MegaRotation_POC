# M0 independent integration source review — a0073f3

2026-11-16, Asia/Shanghai. Named `bft_verifier`, GPT-6 Astra/xhigh. **PASS for source, dependency, harness provenance and preserved historical evidence identity at `a0073f336357aa840467cf623ba98334416787ee`; no blocking source integration finding. M0 runtime acceptance is still pending the frozen same-revision Tester matrix.** This is not a newly executed component test result or M1/M2 acceptance.

## Independence and candidate

The verifier derived and recorded `expectations-v1.md` from the pinned source rules, original contracts and independent component reports before inspecting the integrated tree or Builder explanation. The frozen M0 contract at `poc/docs/milestones/results/M0-task-contract-v1.md` agrees with those expectations and the authorized scope changes: final management input d5cbecb, primary milestone plan ancestor 4a9a6b0, full07 interface checker plus M2 and representative G1/G2, three16 recording replays,09 default tests/ABI, and native healthy seed7 pause0/2.

Candidate worktree: `/home/jy/.codex/worktrees/440b/megaroation_poc`, branch `codex/poc-m1-foundation`. The source candidate's first-parent chain has the contract freeze plus seven ordinary merge commits in the approved order, with all seven exact inputs as ancestors. No source conflict resolution, dependency upgrade or protocol implementation change was introduced. Existing independent workspaces and their lockfiles remain unchanged.

Role metadata was separately checked by the lead from each role's latest own runtime turn context, not inherited parent context: lead Astra/ultra; Docer/Verifier Astra/xhigh; Builder/Tester GPT-6.1 Sol/high. The provided metadata file `/tmp/m0-model-evidence.json` has SHA256 `2cd55eaab2220256da21d218fea89d8af013b10fdb4322643aee469cfd253071`. This review uses that explicit lead-provided attestation and does not infer actual execution from repository configuration alone.

## Read-only identity and evidence audit

Command, from the candidate worktree:

```sh
python3 /tmp/m0-verification/audit_source_binding.py
```

The script only reads Git objects, existing files and historical evidence, and writes under `/tmp/m0-verification`. It runs no build, simulation, network service, Git mutation or submodule operation. Its machine-readable `source-tree-binding.json` records per-directory tree/blob identifiers, per-file SHA256, all comparisons and original artifact paths. **675 identity/input checks and 1,092 historical artifact hash checks passed; zero failed.** These are integrity checks, not1,767 protocol test cases. 1,571 distinct filesystem files were hashed including checkout input comparison.

| Component | Actual historical test source | Integration result |
| --- | --- | --- |
|03|e9c15e04bc83b335668f3ecd837367aa87344a89|Types/auth/core/root manifest+lock/proto/vendor and root workspace dependencies equal tested and delivered source.32 feature cases/52 workspace passes/11 ignored remain the historical counts. Final verifier and Tester manifest hashes and their declared artifacts/input vectors match.|
|02|4fd6f115bd969b0a8f252d7933edd21c27f00a8f|Production/harness source equals e1a4357 delivery and M0. Production tree remains3d2ff54ca9231d2611e424599b0db73170faa9a2. Independent46 and12-run matrix evidence manifests/declared hashes preserved.|
|06|8ba78f1ace3b383caa1c4360401c645d25907326|Publisher/SimEspresso production manifests/locks/source unchanged. The independent harness was untracked during original tests and sealed later: its Rust test, manifest and lock exactly match the original report's hashes. Actual-file recovery evidence49 entries and matrix121 entries preserved. Legacy wrapper packaging differences are not called source changes or newly executed tests.|
|07|8907ad64d882673d6ef2142defdf6bff2e208a71; same reviewed source as de628c2|Production and transitive03/06 dependencies plus existing independent harness unchanged. Nineteen implementation/harness hashes from M2 record match,40 captured runtime inputs plus supplemental inputs match. Later sealed G1/G2 checker/runner bytes match historical summary hashes.117-check trace and12 raw replay outputs/stderr hashes preserved.|
|16|6248072b3fd0eee1ae468f0961a4d3589291cba6|Crate/scenarios and06 dependency unchanged. Later sealed independent Rust harness/lock and final Python oracle/matrix hashes equal the reviewed versions. All seven preserved checksum manifests and690 entries match, including failed first official attempt, final component matrix, wrapper checks and successful official recording.|
|09|87daee8e9e77bbe27c337383d873a5eaf4a694ba|All Solidity/ABI/driver and lock sources unchanged. Sealed reports/helpers match their original hashes. Actual Tester artifacts and independent receipt trace files match. A18/A19 stay unresolved.|

All six component deliveries equal the integrated candidate for their production paths, dependencies and complete dedicated verification directories. For06/07/16, some original tests/checkers were untracked at the historical execution SHA; their recorded content hashes, rather than a false claim that they were then committed, establish identity. Every retained historical result keeps its actual original SHA.

The first audit script attempt stopped because03 manifest input keys use basenames rather than repository-relative names. The audit was corrected to resolve those keys under `poc/verification/mock-plan-execution/implementation-v1/`; no expected hash or candidate file changed. `/tmp/m0-verification/audit-attempt01.txt` preserves the tool-level issue. Attempt02 passed; attempt03 adds07 simulation wrapper/checker and supplemental input bindings and passes. This was an audit path-mapping error, not candidate failure.

## Scope and exclusion checks

- Commander directory exactly matches `d5cbecb24836af80f8cd5414052043a17ca2ae17`. Every pre-existing milestone file matches that input; the only additional milestone file is this task's M0 contract. The d5cbecb upgrade from4a9a6b0 changes only commander state. All management inputs remain ordinary ancestors.
- `poc/prototypes`, `poc/proto` and `poc/vendor` exactly equal old main `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`. The BFT gitlink remains `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`. No submodule worktree/history writes occurred.
- None of the four current01 recovery/recovery-resume/11/12 branch tips is an ancestor of M0. `poc/crates/batch-services` and `poc/crates/derivation` are absent. Existing baseline recovery-named files are preserved; later WIP was not imported.
- The old-main prototype differs from historical `fec7ade1675cb60dd16a16fb28ebe81e3f76f508` at17 paths. This is explicitly recorded; the historical66-run/33-pair Fresh PASS does not transfer. The selected current-source native healthy compatibility runs remain mandatory.
- main was still the authorized old value and no worktree had main checked out at this audit. These are observations, not a reservation: they must be checked again immediately before CAS.

## Boundary spot checks

The lack of integration source changes supports reuse of the independent component code reviews. Additional focused reading confirms the scope labels and verifier boundaries survived:

- `poc/crates/types/src/domain/objects.rs:974` only declares `FinalVerifier::check`; it supplies no permissive implementation. Registry/PoP validation does not become a final-certificate proof by merging.
- `poc/crates/viewer-gfr/src/publications.rs:269` waits when finality services are absent, checks prerequisite basis, then actually calls the supplied verifier. `api.rs:138` separates an explicitly named noncryptographic fixture profile from a Native metadata profile. A Native label is not validation evidence; the true04 certificate gate remains open.
- `poc/crates/espresso-compatibility/src/lib.rs:213` retains the Unknown timing type. At:303/:307 the PE-log/reader exports return `UnsupportedTimingEvidence`; no virtual proposal/finalization/live facts are invented from official HTTP capture.
- All public codec/manifests/local dependency trees relevant to these boundaries are unchanged from their reviewed components. No alternate encoding, shared-type fork, new default verifier or Commonware upgrade entered through the merge.

## Gate to Tester and release

The source candidate is eligible for the frozen validation matrix after the new M0 runner is independently reviewed and committed. This report does **not** approve an unreviewed runner or claim that runtime checks already ran. A subsequent report/runner seal must identify its exact SHA and prove production, dependency, original harness and frozen-input identity to a0073f3; Tester then records that exact current SHA, not a0073f3 or any older historical source SHA by convention.

M0 requires all selected locked checks plus bounded current-source native compatibility, actual G1/G2 semantic checking, and all three official-recording replay oracle checks. Failed or incomplete checks keep main unchanged. A later report-only seal can bind the tested revision through exact source/runner/input identity, but must name which SHA literally executed. Before CAS, check main's old value and worktree occupancy again.

Open after this source review: full I1/recovery; actual native q4 FINAL proof adapter; real15 healthy workers/controlled services/Manager/Publisher/GFR wiring; M1 official publication of its actual result; M2 Force activation; D/RESUME/Batcher/DA;16 PE timing, QC and VID. Retained fixture, HTTP and hash evidence cannot close those gates.

## Artifact hashes

- `source-tree-binding.json`: `4a1617aa85ede347114c433097c6caf89276aaf4573bf972f12a75974f33e2cd`
- `audit_source_binding.py`: `6fc5710c0eb08a76f567453e7d4672b6d80949cf88e18f2ca5ea0fc956e0c46e`
- `audit-attempt03.stdout.json` records the final counts; stderr is empty.

The verifier wrote only `/tmp/m0-verification`; the lead may copy these artifacts sequentially into the assigned M0 verification path after the current writer finishes.
