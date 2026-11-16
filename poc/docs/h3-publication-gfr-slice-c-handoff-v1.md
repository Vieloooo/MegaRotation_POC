# H3 Slice C official artifact-input handoff

2026-11-16. **Bounded input/readback adapter PASS; genuine H1 publication C07 and healthy M1 remain OPEN.** Exact source, independent review and Tester revision: **`bca02b095bac1e5701eb78e352d605c0ed887e10`**. Worktree `/home/jy/.codex/worktrees/1e2f/megaroation_poc`, branch `codex/h3-publication-gfr`. A remains available at `e59d026e8c08b64ff744b57c63cc7756f0b34019`; C does not alter its code.

Only the four formally transferred files changed: `poc/scenarios/espresso-compatibility/{capture.py,run-local.sh,README.md}` and `poc/verification/espresso-compatibility/check_capture.py`. The 16 crate, fixed source/image pins, launcher/process management, isolation checks and old evidence were preserved. The no-argument two-marker/repeat behavior remains supported. Contract: [H3-C-1](contracts/h3-publication-gfr-slice-c-v1.md).

## Consumer entry

The existing runner accepts:

```sh
poc/scenarios/espresso-compatibility/run-local.sh NEW_RUN_ID EXACT_RUNNER_SHA \
  --payload-manifest /absolute/path/to/publication-manifest.json
```

Use a clean reviewed source revision and a new run directory. The runner already acquires the host flock; do not nest another copy. The optional input parent is mounted read-only at `/tmp/ec16-publication-input` inside the existing Bubblewrap command, preserving the guest's private `/tmp` and final-file symlink rejection. No second launcher is introduced.

The bounded `H3-PUBLICATION-ARTIFACT-1` manifest references one to eleven immutable GFR payload files, exact codec `0x47465201`, contiguous ordinals, lengths and SHA-256. All files are checked before launch; escaping/symlink/invalid/oversized inputs reject. The manifest, payloads and optional native report are copied into the capture bundle and rechecked independently. Duplicate input ordinals remain separate submissions and may legitimately map to one recorded occurrence. Source producer revision and runner revision stay separate.

Both validation implementations expose `load_payload_manifest(path)` for offline callers/tests. The checker accepts optional `--payload-manifest PATH`; it always validates copied input evidence when present and additionally compares an external input when supplied. `configuration.input_identity` binds original input fields, copied paths, actual submit IDs/ACKs, and full q/flat-index/namespace-index/projected-index occurrence mappings. Native-tagged data reports `native_verification=NOT_PERFORMED`; the tag/report hash is provenance only. The actual H1 checker must revalidate genuine exported regional bytes for C07.

The legacy EC16 audit codec remains `0xEC160001`, an opaque projection tag. It does not rewrite the GFR payload bytes sent to HTTP. Actual GFR codec and byte identity are recorded separately. Official proposed/finalized/live evidence stays Unknown, PE exports Unsupported, and local API/QC/VID trust boundaries are unchanged. No official capture enters deterministic GFR timing evidence.

## Independent evidence

- Verifier: **52/52** source-derived offline cases, plus both saved official marker readers' six-block audit/projection compatibility; no source finding or harness fix. [Review](../verification/publication-gfr-healthy/slice-c/bca02b095bac1e5701eb78e352d605c0ed887e10/review.md), [result](../verification/publication-gfr-healthy/slice-c/bca02b095bac1e5701eb78e352d605c0ed887e10/result.json).
- Tester on the same SHA: **52/52**, an extra wrong-namespace rejection, **eight unchanged-Rust-CLI offline replays**, and **five legacy vectors**. Synthetic one-entry/eleven-duplicate captures matched independent raw namespace/POST/position derivation; first/37-ms/requery audit/projection bytes matched. Saved official first/requery bytes matched the original six-block audit/projection. No timeout, incomplete profile, new official node or real H1 proof execution occurred. [Tester report](../verification/publication-gfr-healthy/slice-c/tester/report.md), [manifest](../verification/publication-gfr-healthy/slice-c/tester/manifest.json).
- Raw review evidence: `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07/artifacts/verification/h3-c-bca02b-independent-v1`.
- Raw Tester evidence: `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07/artifacts/simulation/h3-c-bca02b-acceptance-v1`; raw manifest SHA-256 `e70e784b8666ee000779a22597b86c9d24b5d9b8035c81bacb04d052f5a57a9e`.

Tool/source/lock/binary hashes, exact commands and the distinction between synthetic and saved official input are retained in the reports/manifests. Rust 1.95.0 used locked offline dependencies and two jobs under the shared heavy lock. No unobserved service run, native finality, root/aux integration or healthy M1 success is inferred. The next integration supplies actual H1/M1 payload bytes through this implemented entry; B implementation continues independently.
