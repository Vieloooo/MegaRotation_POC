# H3 Slice C acceptance

**PASS** for the bounded H3-C-1 artifact-input and offline readback scope at `bca02b095bac1e5701eb78e352d605c0ed887e10` on `codex/h3-publication-gfr`. The 52 independent cases, eight Rust replays and five legacy prefix vectors passed. The extra wrong-namespace check rejected the altered POST at the namespace guard. No command timed out and no profile was incomplete. [Capture index](manifest.json)

| Capture | Input ordinals | Submit records | Source prefix | Rust replays | Audit and projection |
| --- | --- | --- | --- | --- | --- |
| Synthetic single | 1 | 2 | q0–q1 | First 0/37 ms, requery | Byte-identical across replays |
| Synthetic duplicates | 11 | 12 | q0–q1 | First 0/37 ms, requery | Byte-identical across replays |
| Saved official markers | N/A | 3 in saved run | q0–q5 | First, requery | Byte-identical to the saved outputs |

The two new captures used the unchanged pre-code input manifests and their 105-byte synthetic GFR Body. A test driver connected the candidate recorder to the verifier's independent synthetic HTTP model. Both captures retain every input ordinal and the repeat-first request. The ordinal and repeat receipts all point to one observed transaction at `(q=1, flat index=1)`, with namespace-local and projected index 0. The source includes a preceding nonmatching namespace. The Python checker compared original inputs, retained copies, POST bytes and raw positional readback. Rust receipts independently matched every first-reader submit ID, payload, ACK and native position. Manifest identity retains codec `0x47465201`, and the opaque EC16 audit retains `0xEC160001`. [Frozen contract](../../../../docs/contracts/h3-publication-gfr-slice-c-v1.md)

The synthetic producer SHA is the fixture value `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`. The recorder retains it separately from the candidate SHA. The matrix exercised invalid paths, size and hash checks, ACK-only input, incomplete prefixes and synthetic native-tag metadata. Its native verification disposition is `NOT_PERFORMED`. [Independent matrix](../independent-harness/README.md)

The saved official marker capture was read using its original recorder revision `6248072b3fd0eee1ae468f0961a4d3589291cba6`. Rust and the independent Python oracle reproduced its six complete blocks, including the published audit and projection. The 89 saved manifest, bundle and replay files checked before and after the batch retained their hashes. [Accepted Feature 16 report](../../../espresso-compatibility/simulation-report.md)

Inputs were fixed, so no seed was used. Local elapsed time was 2.620 s for the matrix and 9.140 s for the locked Rust build. Driver recordings took approximately 0.215 s each. The 37 ms replay pause was local. Proposal, consensus finalization and live-at-proposal timing were `Unknown`, and full PE export was `Unsupported`. This batch recorded no real-network measurements. C07 genuine H1 result publication, native proof verification, a fresh official run, QC/VID and healthy M1 remain **OPEN**.

## Replay recipe

Run from the same checkout at the verified commit using a fresh task07 artifact basename. All 22 command records retain argv, exits, timeouts, local elapsed time and raw stdout/stderr hashes. The raw manifest captures the concurrent Slice B documentation changes in the worktree status. The 34 Slice C and replay inputs matched before and after. Rust 1.95.0 and two build jobs were used under the shared host lock. [Runner](run.py)

```sh
flock --exclusive --timeout 60 --conflict-exit-code 75 /tmp/megaroation-poc-heavy.lock \
  python3 -B poc/verification/publication-gfr-healthy/slice-c/tester/run.py run \
  h3-c-bca02b-acceptance-replay-v1
```
