# Independent mechanical verification of 5c978db

Candidate: `5c978db105d52e8df3b7f5104175e0552a37263c`.
Checkout: `/home/jy/.codex/worktrees/5d7a/megaroation_poc`.
Result: **CHANGES REQUESTED for mechanical acceptance**. Two reproducible evidence-validation findings below. This candidate never emitted overall M1 PASS in the checked cases; protocol capabilities remain NOT_RUN.

The candidate and frozen artifacts stayed read-only. At the final recorded observation, HEAD remained the named SHA and `git status --porcelain=v1` was empty. Dirty/stale tests used `/tmp/m1-merge-verification-5c978db/isolated-repo` only. No Builder test was read or used as expected truth. All writes, fixture copies, scripts, raw command outputs and reports are in this dedicated `/tmp` directory.

## Sources and expected results fixed before inspection

- Mechanical expectations: `/tmp/m1-merge-mechanical-expectations.json`, SHA-256 `241faf3827b54f8dc8d853e68d4be17003f9cf04e8c7ae4dac70470c2eeab167`, 22 case groups derived before candidate inspection.
- Frozen preparation baseline: `3f4b671224c0b68bbd2ede40934822c33eafcbe8`.
- Contract SHA-256: `29e05ab86d156eab0473fd6a59a08cf2a5f962dfa3bf5c171d70dca941aeb647`.
- Source expectations SHA-256: `2abf64ce7000c272354490e70d6b28983a6e92ce84a8782684701735f34a13d3`.
- M0 `35d382ac445cc8bc779f0d18fd23cb57e54680da` is an ancestor. Dispatch document bytes from the candidate and exact `d55d39f73593676325bdc0fbb7ba3845ca93025f` source match SHA-256 `4a211cd38a5db108dc3b8548a87c19e599cb3870e534e513b41e487b4c5cc8b6`.
- Gitlink remains Regional BFT `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`. Successful preparation probes record locked Commonware 2026.10.0 and Rust/Cargo 1.95.0; only version probes ran, not builds.

## V1 [P2] Recorded Git tree is not authenticated against the named candidate

**Rule:** M1-HARNESS-1 §6 requires the exact named candidate and source identity; BundleIntegrity includes candidate/tool/input/dependency identity. §5.2 requires actual source hashes, and §6 says input manifest claims are not verification evidence.

**Location:** `poc/verification/m1-healthy-regular/check.py:154` reads the recorded `git ls-tree` output, and lines 155–160 use its claimed object IDs as the entire source binding. `poc/scenarios/m1-healthy-regular/evidence.py:443` verifies a copied source against that supplied blob ID, but neither checks the recorded tree against an independently obtained tree for the named commit.

**Smallest observed transformation:** Starting from a mechanically valid preparation bundle, append one comment to `source/poc/scenarios/m1-healthy-regular/run.py`. Replace that path's blob ID in the bundle's recorded `ls-tree` stdout; update affected FileRefs and manifest seal. Leave candidate SHA, Git command argv, revision and ancestor claims unchanged.

The actual candidate blob is `e6c4ca6cfe9c190c47f01cab6660451a73c767df`; the altered blob is `c443fd7ed1a48d83a7d895a1c696fe3f6d7fab74`. The actual Git object was independently read using `git rev-parse 5c978db105d52e8df3b7f5104175e0552a37263c:poc/scenarios/m1-healthy-regular/run.py`.

**Expected:** reject invalid candidate/source evidence, exit 2. **Observed:** exit 3 with `BundleIntegrity=PASS` and all protocol gates NOT_RUN. Impact is an incorrect positive source-provenance result: a self-consistent rewritten bundle can attribute code to a commit that never contained it. This does not presently bypass the missing protocol oracle or produce M1 PASS.

**Replay** (use a fresh output path):

```bash
python3 -B poc/verification/m1-healthy-regular/check.py \
  --bundle /tmp/m1-merge-verification-5c978db/bundles/resealed_candidate_source \
  --expected /tmp/m1-merge-verification-5c978db/inputs/allnull/expected.json \
  --output /tmp/m1-v1-new-report.json
```

Raw command/result: `raw/M17_resealed_tree_run/command.json`; output: `outputs/M17_resealed_tree_run.json`; actual/altered Git IDs: `raw/M17_resealed_tree/source_binding.json`. Recommended correction is a real read-only binding to the named local Git commit or independently verifiable Git object evidence, not merely another hash of the rewritten stdout. If commit identity cannot be independently verified, source provenance cannot be marked PASS.

## V2 [P2] H09 nested source reports bypass typed and FileRef validation

**Rule:** M1-HARNESS-1 §§5.1–5.2 freeze M1-CHECK-1, strict JSON types and safe FileRefs. H09 `source_run.checker_report` is a referenced source report, not an opaque production port. §6 requires invalid evidence to take precedence over generic NotReady.

**Location:** `poc/scenarios/m1-healthy-regular/evidence.py:310`–328. `source_run()` checks the report's top-level key set, schema/operation, and a manifest hash match in `bundles`, but ignores the validity of `status`, `expected`, `checker`, `commands`, `gates` and `reason`.

**Reproducers:** A valid mechanical H09 source input (with clearly labelled non-protocol fixture payload files) returns NotReady as expected. Two independent copies change only the source report, then recompute its FileRef:

1. Set the report's `status` to JSON `true`.
2. Set `expected` to `{"path":"../outside.json","sha256":"0000000000000000000000000000000000000000000000000000000000000000","bytes":false}`.

**Expected:** exit 2 for invalid source evidence. **Observed:** both preflights return exit 3 / NOT_RUN without reporting the malformed report. Impact is incomplete source-report validation: invalid typed verdicts and forbidden references can be sealed into an otherwise accepted preparation bundle. No outside file read or binding execution was observed, and no overall PASS occurred.

**Replay** (fresh output directories):

```bash
python3 -B poc/scenarios/m1-healthy-regular/run.py preflight \
  --candidate 5c978db105d52e8df3b7f5104175e0552a37263c \
  --input /tmp/m1-merge-verification-5c978db/inputs/source_run_report_bool_status/input.json \
  --output /tmp/m1-v2-bool-new

python3 -B poc/scenarios/m1-healthy-regular/run.py preflight \
  --candidate 5c978db105d52e8df3b7f5104175e0552a37263c \
  --input /tmp/m1-merge-verification-5c978db/inputs/source_run_report_bad_expected_ref/input.json \
  --output /tmp/m1-v2-ref-new
```

Raw results are under `raw/M20_H09_report_bool_status/` and `raw/M20_H09_report_bad_expected_ref/`. `M20_H09_control` establishes that the base source_run is accepted as legal NotReady preparation. Validate the frozen nested report structure/references without executing its claimed commands, trusting its PASS, or interpreting unknown production binding schemas.

## Coverage and limits

`results.json` records **78 concrete candidate-command results: 75 matched, three mismatches representing V1 and the two V2 variants**. The frozen 22 groups were exercised with these concrete variants:

- M01–M06: null preparation and run → 3; real source expectations do not provide an oracle; opaque JSON/non-JSON bindings never execute; hash-consistent fake PASS and identical fake-PASS comparison remain NOT_RUN.
- M07–M12: duplicate keys at two levels; true/false as integer, negative/float/string integer; unknown/missing schema fields; lexical `..`, normalized traversal, absolute FileRefs; leaf/intermediate symlinks pointing inside/outside; byte/hash/size/missing/directory failures → 2.
- M13–M14: duplicate inventory rejected, legal shared reference retained; existing empty/content/symlink outputs protected for all four operations.
- M15–M17: invalid/short/mismatched/stale candidate and tracked/untracked dirty isolated checkout rejected; actual candidate/source/contract/dispatch/expectation pins checked. V1 extends the source-identity case with a coordinated self-consistent source/tree mutation found during code review.
- M18–M22: required-gate deletion/duplication/unknown/extra gate refused; separate bundle/expected/output roots resolve correctly and emitted FileRefs were rechecked; corruption wins over NotReady; H09 null source/capability remains NotReady. V2 extends report-integrity/type cases through the explicitly frozen H09 source-report path.

Exact argv, cwd, exit, stdout/stderr, output paths and reasons are in `raw/*/command.json`. Source scripts are `review_cases.py`, `review_remaining.py`, and `review_provenance_edges.py`. A reviewer bookkeeping exception initially tried to JSON-parse an intentionally preserved non-JSON existing output; it was fixed in the `/tmp` helper and that existing-output subset was rerun. A second helper log-directory collision was fixed before the provenance cases ran. These are retained reviewer harness artifacts, not candidate findings. No Builder mechanical test was run or read.

No unknown production binding/schema was implemented; no worker, official node, BLS verification, nonce/balance execution, GFR replay, Rust build or heavy simulation ran. The binding sentinel file stayed absent. Source inspection shows only fixed Git/tool-version probes in the launcher and no subprocess execution in the checker. This is evidence for finite mechanical refusals, not the M1 15-process or protocol matrix. All protocol oracles, real H1/H2/H3 integration and official same-byte gates remain open. A corrected named candidate requires focused recheck before Simulator reruns it.
