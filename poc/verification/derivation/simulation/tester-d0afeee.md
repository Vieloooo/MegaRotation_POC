# Feature 12 Stage A formal test report

**Stage A PASS on `d0afeee9363ef6703f51eecfd742915f83aa27f2`.** The four Rust tests, eight fresh private Anvil runs, raw receipt checker, and public API harness all passed. Full feature 12 remains **INCONCLUSIVE** pending Stage B and downstream integration.

| Record | Value |
| --- | --- |
| Date | 2026-11-16, Asia/Shanghai |
| Tester | `bft_simulator`, GPT-6.1 Sol / high |
| Worktree | `/home/jy/.codex/worktrees/29df/megaroation_poc` |
| Branch | `codex/poc-derivation` |
| Integrated baseline | `37b4f2b8a6d67b303ba90b1d4d1981c93ed8dd12` |
| Interface | `3b1b34fce6f3a3bac5099d53bdf5f7df56079f80` |
| Contract and independent expectations | `4a86f8d5c6f2d39e52994bb7c862fcc7313ccf8c` |
| Candidate / Verifier / Tester source | `d0afeee9363ef6703f51eecfd742915f83aa27f2` |
| Seed | `none (explicit script)` |

The tracked worktree matched the candidate at launch and at completion. The launch status shows the Verifier's untracked support files under `candidate_review/`. This report and its manifest are in `simulation/`. The exact source and expected results are identified in the [frozen contract](../../../../poc/docs/contracts/task-contract-derivation-v1.md) and [independent review](../candidate_review/review-d0afeee.md).

## Fresh receipt matrix

Each row ran with 300 ms and 650 ms injected host wall pauses. Counts include genesis where applicable and are per run. Reader A read the delivered history in ascending order. Reader B first encountered a delivery gap at block 1, then read in descending order. Their canonical outputs, checkpoints, and action traces agreed in every run. The two pauses in each row produced byte-identical `virtual-trace.json`, `d-a.json`, and `d-b.json`.

| Scenario | Result | L1 blocks / receipts | Final D / cuts | Observed transition |
| --- | --- | ---: | ---: | --- |
| `window` | PASS | 52 / 6 | 24 / 1 | With origin 10 and a 40-block window, frontier 49 held D20. Complete block 50 added D21/origin10 and D22/origin11. Block 51 added D23 and D24. |
| `ahead` | PASS | 47 / 8 | 14 / 3 | RESUME `(45,0,1)` preserved prefix D10 with anchor/state D8. RESUME `(45,1,1)` filled through D11 and captured its own prefix. Ordinary growth reached D14 while both historical cuts retained their original contents. A fresh process restored the checkpoint between the two markers and reproduced the continuous replay. |
| `cap` | PASS | 5 / 6 | 30 / 1 | CursorSet in L1 block 3 filled D30/origin3 at timestamp 1030 s. The including L1 timestamp was 1006 s and the ordinary drift bound was 15 s. |
| `accepted_wait` | PASS for the Stage A WAIT gate | 42 / 7 | 0 / 0 | The actual opaque Inbox acceptance at `(40,0,0)` left frontier 39 and D0. The later CursorSet and RESUME did not produce a cut, even after block 41 delivery. |

Both `window` runs also replayed captured 09 RPC with the genuine block 12 receipt temporarily withheld. Mine command 30 returned WAIT and derivation remained at D0. Supplying the genuine receipt reconciled command 30 and made the exact `(12,0,1)` cut available at target D20/origin10. The two `receipt-gap-replay.json` outputs matched byte for byte. This profile injects new unavailability during offline receipt replay. The `ahead` checkpoint and resumed replay files likewise matched across wall pauses.

The independent checker reconstructed the log order and D from 292 raw L1 blocks and 54 raw receipts, then compared 136 derived blocks and 10 cuts per reader. Its negative controls reject changed native hashes, changed cut targets, and missing receipts. The public API harness restored 111 action checkpoints, rejected 21 checkpoint mutations and 3 genesis mutations, covered invalid config/DEP and bounded 03 fixture inputs, and distinguished unknown future positions from known non-RESUME positions. The 03 fixture check covers only committed rows 0–5. Together with the independently reviewed source, these runs cover the frozen Stage A portion of DA01–DA18, including its named fixtures and acceptance WAIT boundary.

## Reproduction and evidence

All commands ran in `/home/jy/.codex/worktrees/29df/megaroation_poc`. The build and simulation used the host-wide lock with two Cargo jobs. The test used Rust/cargo 1.95.0, Python 3.12.8, coincurve 20.0.0, Foundry 1.0.0-stable, and solc 0.8.20. Full command expansion, compiler and tool versions, binary hashes, and run profiles are in the evidence directory.

```sh
export PATH="/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:/home/jy/miniconda3/bin:$PATH"
export CARGO_HOME="$PWD/.build/cargo"
export CARGO_TARGET_DIR="$PWD/.build/target/derivation"
export CARGO_BUILD_JOBS=2
export L1_FOUNDRY=/mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0
export PYTHONDONTWRITEBYTECODE=1
out=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/12/artifacts/derivation-tester-d0afeee-run1

test "$(git rev-parse HEAD)" = d0afeee9363ef6703f51eecfd742915f83aa27f2
git diff --exit-code HEAD -- .
time flock /tmp/megaroation-poc-heavy.lock bash poc/crates/derivation/tests/validate.sh "$out"
python3 poc/verification/derivation/candidate_review/check_raw.py "$out" "$out/raw-check-d0afeee.json"
flock /tmp/megaroation-poc-heavy.lock cargo run --locked --offline \
    --manifest-path poc/verification/derivation/candidate_review/api_harness/Cargo.toml -- \
    "$out" > "$out/api-check-d0afeee.json"
sha256sum --check --quiet "$out/evidence-sha256.txt"
```

The validation command took 21.406 s host wall time including the lock, builds, and scripted pauses. Final virtual reference times were 104, 94, 10, and 84 s for `window`, `ahead`, `cap`, and `accepted_wait`, respectively. L1 used a 1000 s genesis timestamp, 2 s block slots and 2 s modeled finality delay. Anvil RPC ran on private loopback ports. Network latency was outside the measured scope. All runs exited successfully and their Anvil child processes were gone after completion.

The [evidence manifest](evidence-manifest-d0afeee.json) covers 126 new files totaling 31,628,496 bytes. Per-file hashes, configuration inputs, and byte equality checks are recorded there. The runner's 95-file checksum list passed `sha256sum --check`.

| Evidence | SHA-256 |
| --- | --- |
| `evidence-manifest-d0afeee.json` | `ec08d4380ee37a405ea627b65c12599667541ba5ed2be4364e92c4665793d2bf` |
| `evidence-sha256.txt` | `3a22217a9e2d97ca174f5360ce086d73e73ac3d0fc94f563abad2712025608b4` |
| `raw-check-d0afeee.json` | `c0b08d03d9ad0b7cda07923d8f60c69142b6204fad9f53a89dd905de159803d9` |
| `api-check-d0afeee.json` | `364ed3fbd62971e76412d9663d295f6309faced70e69364f5d18d999a16c82df` |

Raw receipts, logs and checker results are in the [fresh run directory](/mnt/reth/jy/megaroation_poc/.build/commander-jobs/12/artifacts/derivation-tester-d0afeee-run1).

## Open acceptance gates

Stage B remains INCONCLUSIVE for actual feature-11 frames and DA commitment verification, unavailable or mismatched payload recovery, ordinary batch adoption, inclusion lateness over real payloads, substitution and second-record revalidation, and actual batch input between two same-block cuts. Feature 13 integration, controlled 15-worker runtime, Batcher crash/restart, and BFT same-cid recovery remain outside the observed Stage A cases.
