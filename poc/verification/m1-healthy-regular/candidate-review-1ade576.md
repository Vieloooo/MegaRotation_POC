# Independent mechanical recheck — 1ade576

Candidate: **`1ade576f2312a75c3b9652eb8147d47d0ad6cb09`**.
Checkout: `/home/jy/.codex/worktrees/5d7a/megaroation_poc`.
Conclusion: **ACCEPT for the frozen mechanical preparation scope; ready for Simulator to rerun that scope on this exact SHA.** V1 and V2 from `5c978db` are closed by the checked repair. No new blocking finding was observed. **M1 remains NOT_RUN.**

The independent expected outcomes remain those frozen before initial candidate inspection in `/tmp/m1-merge-mechanical-expectations.json` (SHA-256 `241faf3827b54f8dc8d853e68d4be17003f9cf04e8c7ae4dac70470c2eeab167`). Contract/source-expectations hashes remain `29e05ab86d156eab0473fd6a59a08cf2a5f962dfa3bf5c171d70dca941aeb647` and `2abf64ce7000c272354490e70d6b28983a6e92ce84a8782684701735f34a13d3`.

I read the actual repair diff in `evidence.py`, `run.py` and `check.py`; I did not read or run Builder's `mechanical_tests.py`. Runtime behavior was compared with the frozen contract and independent fixtures, not Builder tests. Old artifacts remain intact under `/tmp/m1-merge-verification-5c978db/`.

## Results

**101 recorded candidate commands, all 101 matched expected exits and verdict boundaries.** `results.json` contains each exact argv, cwd, exit code, expected exit, output path, status/reason and measured wall duration. `raw/<case>/` retains stdout/stderr and command record.

| Checked set | Commands | Outcome |
| --- | ---: | --- |
| Frozen mechanical baseline variants | 67 | Correct schema/type/path/hash/size/output/pin/gate refusal; null preparation, opaque bindings, fake PASS and fake-PASS compare remain NotReady. |
| Isolated clean/dirty/stale guards | 7 | Clean temporary checkout → 3; tracked/untracked dirty and stale preflight/run → 2. Shared checkout never mutated. |
| Exact three original counterexamples | 3 | All → 2. Old V1 lacks mandatory commit evidence; old V2 rejects malformed report contents before stale-candidate checking. |
| New-format commit/tree/source mutations | 6 | All → 2 for the actual Git object binding violation. |
| New-format H09 source-report controls/negatives | 18 | Legal preflight and run → 3; 16 frozen-format type/path/duplicate-key mutations → 2. |

No Rust build, worker, official node, signature generation, crypto oracle, execution oracle or GFR replay ran. The opaque-binding execution sentinel remains absent. Source inspection shows fixed read-only Git/tool probes only; checker starts no processes. Actual preparation probes capture Rust/Cargo 1.95.0, Commonware 2026.10.0, Regional-BFT gitlink and candidate/source identities. This is a finite mechanical result, not protocol evidence.

## V1 closure: commit → root tree → PoC subtrees → source blobs

Source rule: M1-HARNESS-1 §6 candidate/source identity and non-authoritative manifest claims; §5.2 actual source evidence.

Repair reviewed at `poc/scenarios/m1-healthy-regular/evidence.py:561` (`git_object_id`), `:587` (`verify_tree`), and `:595` (`candidate_sources`); the offline checker consumes the recorded commit/root/subtree evidence at `poc/verification/m1-healthy-regular/check.py:89`. It recomputes the commit ID and every relevant tree ID before accepting blob IDs. The commit's first tree header binds the root; the root binds the complete PoC directory tree and Regional-BFT gitlink; copied source bytes are checked against those authenticated blob IDs.

The old forged bundle is rejected because its old evidence format lacks the commit probe. **This alone was not counted as sufficient closure.** Fresh `1ade576` bundles were generated successfully and passed the mechanical inventory control; then independent copies were modified, with all touched FileRefs and manifest seals recomputed:

| Case | Rewritten material | Actual rejection reason |
| --- | --- | --- |
| `V1_new_commit_bytes` | Raw commit bytes | `Git commit object ID mismatch` |
| `V1_new_root_entry` | One root-tree object ID | `Git tree object ID mismatch: /` |
| `V1_new_poc_entry` | One PoC-tree blob ID | `Git tree object ID mismatch: poc` |
| `V1_new_source_plus_leaf_tree` | Copied run.py and its leaf blob ID in the recursive tree | `Git tree object ID mismatch: poc/scenarios/m1-healthy-regular` |
| `V1_new_source_plus_all_trees` | run.py, all affected PoC subtree IDs, root-tree entry | `Git tree object ID mismatch: /` |
| `V1_new_source_plus_all_trees_and_commit` | Above plus a rewritten commit pointing at the rewritten root | `Git commit object ID mismatch` |

The last two mutations recompute ancestor tree IDs independently using Git object encoding. They demonstrate that consistency between rewritten source files and their own tree records cannot substitute for the named commit. Rejections are substantive object-ID failures, not version or missing-field failures.

Example direct replay, using a fresh output path:

```bash
python3 -B poc/verification/m1-healthy-regular/check.py \
  --bundle /tmp/m1-merge-verification-1ade576/bundles/V1_source_plus_all_trees_and_commit \
  --expected /tmp/m1-merge-verification-1ade576/inputs/allnull/expected.json \
  --output /tmp/m1-recheck-v1-new-report.json
```

Expected exit is 2. Equivalent commands and outputs for all six mutations are stored in their `raw/V1_new_*/command.json` and `outputs/V1_new_*.json` files.

## V2 closure: shared frozen run/report/type/reference checks

Source rule: M1-HARNESS-1 §§5.1–5.2 typed M1-RUN-1/M1-CHECK-1 and FileRef rules; §6 invalid evidence precedes generic NotReady.

Repair reviewed at `poc/scenarios/m1-healthy-regular/evidence.py:395` (`run_fields`), `:435` (`check_report_fields`) and `:461` (`source_run`). Both direct bundles and H09 nested source reports use the shared validators. These checks inspect frozen report structures without treating a claimed PASS as oracle execution and without executing claimed commands.

The original `status=true` and `expected={path:"../outside.json",bytes:false,...}` inputs now return 2 due to report validation, before the otherwise-stale original candidate SHA could be checked.

For the new format, a source manifest/report plus explicitly labelled mechanical payload fixtures is accepted as legal **NotReady** preparation in both preflight and run. It neither starts source-run workers nor claims real official publication. Sixteen independent copies then reject: boolean status; expected parent/absolute path or boolean size; boolean command time/exit; integer timeout; string argv; relative checker root; traversing checker source reference; boolean gate-evidence size; boolean manifest event ID; traversing manifest source reference; malformed tool hash; duplicate report keys; duplicate manifest keys.

Example positive and rejecting replays (fresh output directories):

```bash
python3 -B poc/scenarios/m1-healthy-regular/run.py preflight \
  --candidate 1ade576f2312a75c3b9652eb8147d47d0ad6cb09 \
  --input /tmp/m1-merge-verification-1ade576/inputs/V2_control/input.json \
  --output /tmp/m1-recheck-v2-control-new

python3 -B poc/scenarios/m1-healthy-regular/run.py preflight \
  --candidate 1ade576f2312a75c3b9652eb8147d47d0ad6cb09 \
  --input /tmp/m1-merge-verification-1ade576/inputs/V2_expected_parent/input.json \
  --output /tmp/m1-recheck-v2-invalid-new
```

Expected exits are 3 and 2 respectively.

## Commands, state and limits

The independent scripts were run with:

```text
M1_REVIEW_ROOT=/tmp/m1-merge-verification-1ade576
M1_CANDIDATE=1ade576f2312a75c3b9652eb8147d47d0ad6cb09
python3 -B /tmp/m1-merge-verification-1ade576/review_cases.py
python3 -B /tmp/m1-merge-verification-1ade576/focused_recheck.py
```

The two environment values were supplied to each invocation. Recreating fixtures requires a new empty `M1_REVIEW_ROOT`; replaying individual retained fixtures uses the exact commands in raw records and fresh output paths. All preparation inputs use seed 7/forward/pause 0 where those fields are applicable; no virtual protocol trace was produced, and this does not exercise seeds 42/99 or the 24-run M1 matrix.

`verification-state.json` records final HEAD exactly equal to `1ade576f2312a75c3b9652eb8147d47d0ad6cb09`, empty Git status, 101 matched outcomes and unchanged frozen hashes. The only dirty/stale writes occurred in this artifact directory's temporary isolated Git clone. Candidate source and frozen acceptance artifacts stayed read-only. Evidence is sealed by `artifact-seal.json`; the reconstructable isolated checkout is excluded from that file inventory, with its setup commands and all observed results retained.

This closes the two mechanical findings only. H1/H2/H3 production APIs, real 15-worker traces, native BLS/proof checking, exact state execution, independent GFR oracle, H01/H02 matrix and official same-byte publication remain unexecuted/unverified here. The Simulator may reproduce this accepted mechanical scope at this exact SHA; it cannot report M1 PASS from these outcomes.
