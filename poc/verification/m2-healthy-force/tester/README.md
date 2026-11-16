# M2 Healthy Force Tester preparation

Runner and evidence commands will use the exact clean SHA named in the independent Verifier review. Seed 42 and the frozen M2-HEALTHY-FORCE-1 profile apply to both main executions. The second execution adds a bounded, deliberate wall delay at a known boundary without changing the protocol clock or virtual input sequence. This file contains no observed scenario outcome.

| Contract cases | Execution | Evidence |
| --- | --- | --- |
| HF01, HF03, HF09 | 15-worker Healthy Force over real Anvil | Full raw trace, receipts, process exits, exact anchor/cid and h16 G acceptance, late old proof/public position |
| HF02 | Identical semantic run at same SHA with wall pause | Two raw directories, declared and observed pause, normalized full protocol trace hashes and first mismatch if any, exact protocol frame hashes where available |
| HF04–HF08, HF10–HF11, HF14 | Exact reviewed production-entry rule vectors | Bound named test functions to each case, raw command/stdout/stderr and exact assertions. Record HF14 as rule-vector scope if it has no 15-worker entry |
| HF12 | Real Anvil transaction controls | Full txs, receipts and exact event count. Use the same tested source revision for the Authority and its decoder |
| HF13 | M1 same-SHA 15-worker smoke, original profile | Sealed M1 manifest, h10/h20 handoffs and h21 acceptance, archived independent public replay and complete controlled-frame/teardown checks |

Run one heavy command at a time under the host flock. Test cases executed by a runner which holds the flock should be invoked directly. Any shared offline checker binary is read-only. Build paths and fresh artifacts remain under the M2 data directory.

## Same-revision M1 regression and independent oracle

From the M2 worktree, pass the exact reviewed SHA in `M2_CANDIDATE`. The wrapper overrides only the committed M1 launcher's build/cache location. The launcher holds the heavy lock across the offline locked build and smoke and performs clean Git checks before and after. The wrapper's original argv is captured in the normal evidence package.

```bash
M2_BUILD_ROOT=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m2-healthy-force
M1_CHECKER=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m1-regular/artifacts/independent-verification-91547857-seed42-run01/checker
M1_TEST_OUT="$M2_BUILD_ROOT/artifacts/tester-$M2_CASE_ID-m1"
python3 -B poc/verification/m2-healthy-force/tester/m1_regression.py \
  --worktree "$PWD" --build-root "$M2_BUILD_ROOT" \
  --candidate "$M2_CANDIDATE" --output "$M1_TEST_OUT"
bash "$M1_CHECKER/reproduce.sh" "$M1_TEST_OUT/raw" "$M1_TEST_OUT-independent" "$PWD"
```

The archived `reproduce.sh` uses the existing M1 verifier executable at `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m1-regular/target/debug/m1-smoke-independent`. Verify its SHA-256 before execution against the archived verifier report's checksum inventory. The expected binary SHA-256 is `10f564d4a9e00a46aff8d39b499da6c9241209eea649d69ecb194aa85a14437e`. The sidecar `checker-sha256.txt` identifies the independent Rust source, Python controlled-frame oracle, lockfile and compatible native.proto. Use the M2 candidate native.proto only if it is byte-identical to archived hash `981ed3b55639aa7ead414bf7bb5059804323eecfd8f8f9541f2d7caf891d0cd2`. The archived supplemental Tester `check_run.py` requires branch `codex/poc-m1-regular` and is unsuitable unchanged for the M2 branch. `checker/reproduce.sh` has no branch requirement.

## Repeat normalization

The M1-derived trace format has run diagnostics in `detail`. Disregard only `os_pid`, `proc_stat`, and `wall_time_ns` on `WorkerSpawn`, and `os_pid` and `wall_time_ns` on `WorkerExit`. Preserve logical `pid` and every virtual/public/local time, sequence, event identity, clock sample, RPC delay, Grant/Ack binding, protocol output, and exit status. Exclude an added M2 field only if the reviewed schema identifies it as a host-specific diagnostic. Freeze the exact event/field exclusions before the first official run. The unmodified trace and diagnostic process/RPC files remain evidence. Compare canonical JSON for every event, in order. Compare the raw framed Grant/Ack stream by SHA-256 when it contains only protocol bytes. Wall time is a real host measurement and does not establish simulated protocol deadlines.

## Tool and failure provenance

Required artifacts include command argv and cwd, exit/timeout, exact before/after Git status and HEAD, profile and seed, tool/binary/lock SHA-256s, and indexed raw files. Report a failed or incomplete run at its precise cutoff. Use scoped escalation for the official loopback-only Anvil invocation because the ordinary sandbox does not permit socket creation. Use a fresh output for each attempt and preserve any failure diagnostics.
