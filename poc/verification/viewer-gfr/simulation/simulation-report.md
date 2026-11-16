# Viewer GFR G1/G2 simulation

**PASS on `8907ad64d882673d6ef2142defdf6bff2e208a71`** under the frozen [GFR-07-v1 contract](../../../docs/contracts/task-contract-viewer-gfr-v1.md). The independent trace checker passed 12 of 12 cases. All six 0/2 ms pause pairs produced byte-identical stdout. There were zero failed executions, watchdog timeouts, or incomplete cases. The fixture matrix closes the wall-pause portion of VG35. [Integrated verification](../integrated-review-8907ad6.md) covers the approved revision and the preceding independent rule checks.

The runs used branch `codex/poc-viewer-gfr` in `/home/jy/.codex/worktrees/1e2f/megaroation_poc`. Every recorded before/after HEAD matched the revision above. Git status showed only the untracked integrated review files and Tester runner during execution. The 40 captured input hashes matched before/after and matched the revision's Git objects when checked after capture. The offline checker also compared the workspace manifest, I1 proto input and public registry fixture to their Git objects. All twelve invocations produced the same replay binary hash.

## Replay

Run the capture wrapper from the verified checkout under Python 3.12.8. It selects the explicit Rust 1.95.0 toolchain, uses the task `.build/cargo` and `.build/target`, sets `CARGO_BUILD_JOBS=2` and serializes all invocations. Use a fresh output directory for a repeat.

```sh
python3 poc/verification/viewer-gfr/simulation/run-matrix.py \
  --output .build/artifacts/simulation/gfr-8907ad6-g1-g2-repeat
python3 poc/verification/viewer-gfr/simulation/check-traces.py \
  .build/artifacts/simulation/gfr-8907ad6-g1-g2-repeat --summary /tmp/gfr-8907ad6-repeat-summary.json
```

The wrapper uses this command for each combination of G1/G2, seed 7/42/99 and pause 0/2 ms. The 60 s lock timeout and 180 s host watchdog bound the capture process.

```sh
flock --exclusive --timeout 60 /tmp/megaroation-poc-heavy.lock \
  cargo run --locked --offline --manifest-path poc/crates/viewer-gfr/Cargo.toml \
  --example replay -- "$scenario" "$seed" "$pause_ms"
```

Raw evidence is in `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07/artifacts/simulation/gfr-8907ad6-g1-g2-v1`. The first line of each `<scenario>-<seed>-<pause>.jsonl` holds the source, including exact PELG block bytes, L1 and DEP, registry, genesis and parameters. The following lines hold each reader's replay steps. Each `.stderr` contains Cargo output and the runner's seed, pause and tips. Each `.command.sh` and `.run.json` contains the command, status, wall measurement and per-run hashes. `capture.json` holds the environment, tool versions, status and input hashes. Its SHA-256 is `22ffb73876e98efc6327ff8adcea648e1734f2e9a874bc7e57e1c783752fcf6c`. The replay binary SHA-256 is `0544600279839b71d89b8a8d363640aea7f4e81af140654ff35b1e321ed76524`. The isolated Cargo.lock SHA-256 is `11a1dd677d7c80da84f2371681e6054d5ac1df0ac72b5bd748d63843f746a6b8`. [Machine-readable results](summary.json)

| Scenario | Seed | Local wall at 0 ms pause | Local wall at 2 ms pause | Shared stdout SHA-256 |
| --- | ---: | ---: | ---: | --- |
| G1 | 7 | 0.616 s | 1.017 s | `0990e115f76aea8e0a07252e0c83f478f026d1ea172cc1637d13e3b71af74457` |
| G1 | 42 | 0.615 s | 0.966 s | `53eabf5b9d4b98ab59cefd3dac8be07ad62ede15d96b04a0d175af2fe979bc5f` |
| G1 | 99 | 0.615 s | 0.966 s | `53eabf5b9d4b98ab59cefd3dac8be07ad62ede15d96b04a0d175af2fe979bc5f` |
| G2 | 7 | 0.615 s | 1.017 s | `6c4ba42753752599226376eb956ccaebf8e6dbaacbd464d09909ef55f5afd371` |
| G2 | 42 | 0.615 s | 1.016 s | `6c4ba42753752599226376eb956ccaebf8e6dbaacbd464d09909ef55f5afd371` |
| G2 | 99 | 0.615 s | 1.016 s | `6c4ba42753752599226376eb956ccaebf8e6dbaacbd464d09909ef55f5afd371` |

## Fixture schedule and time

- **Public virtual inputs.** G1 publishes q0 through q5 at `30_000_000, 31_000_000, 53_999_999, 54_000_000, 61_000_000, 64_000_000` μs. G2 publishes q0 through q7 at `12_000_000, 13_000_000, 14_000_000, 15_000_000, 16_000_000, 23_999_999, 24_000_000, 52_000_000` μs. Separate PELG fields carry virtual proposal ticks `θ_q + 1000q` and finalization ticks another 100 μs later. Finite L1 source heights are 0–30 and finite DEP heights are 1–40. G1 commits seven events before the unsupported control. G2 commits twelve events through q7.
- **Private reader delivery.** A receives all L1 rows and whole Espresso blocks in source order before replay. B initially receives q2 fragments, then incomplete q1, then q0. For each block B reverses the slots, rotates by `seed % (message_count + 1)` and duplicates delivered messages where `(seed + slot) % 3 == 0`. B withholds q1 message 1 in G1 seed 7 and message 0 in seeds 42 and 99. B withholds q1 message 0 in all G2 cases. G1 stages L1 headers and control lists separately. G2 delivers lists and headers in separate reverse passes. Scripted receipt counters start at 100,000,000 for A and 200,000,000 for B and increment on each delivery. Those counters belong to the reader schedule. Per-fragment receipt ticks and private reader snapshots are absent from JSONL. The runner asserts that the final private snapshots differ.
- **Local wall and real network.** The injected pause is a wall sleep at each reader delivery. The table measures each whole locked Cargo invocation with a monotonic host clock, including any lock acquisition and Cargo work. Real network measurements are absent.

The source line is byte-invariant across all seeds and pauses per scenario. Source SHA-256 values are `3f4e63fbf1524be277a57903ba19c285e80abb92878c9357e2dba372cb1674ea` for G1 and `30ac3b29578fefcf80bac4aca790edec45d95068517929354069e0a1645be534` for G2. The checker compared public projections at seven distinct common completed positions in each G1 run and ten in each G2 run, for 102 reader comparisons. It accumulates committed records and lifecycle order before comparing same-position state, excludes reader labels and private diagnostics, and checks projection invariance across seeds. G1's expected seed-dependent withheld-message diagnostic accounts for its two stdout hashes.

## Independent trace checks

**G1 PASS.** q0 caches the successor proof and bodies while G stays at genesis. B returns frontier WAIT for its incomplete Espresso message list and then L1 control coverage through height 10. Once ordering is known, later L1 origin 11 and the second handoff's missing Plan reference both yield WAIT at q1 while the public state and records remain at q0. One completed q1 record accepts h1–10 and h11–20, performs two handoffs and carries four distinct lifecycle events. Their sources retain q_avail 1 and 0 respectively at acceptance key 31,000,000 μs. At 53,999,999 μs B still waits for SessionEnd coverage. SessionEnd at 54,000,000 μs carries h20 forward and assigns the fixed window 31–40 with no append. q3's unknown publication enters diagnostics only at Espresso q3. Expire for h21 at 61,000,000 μs preserves h20 and closes the branch. A complete block-26 control list exposes synthetic RESUME at L1 `(26,1,0)`, key 64,000,000 μs. It stops before Force at `(26,1,1)` and Espresso q5. The last completed pc remains Espresso q4 at 61,000,000 μs.

**G2 PASS.** A public proof without a body at q0 leaves G at genesis. B's locally undelivered public q1 body causes WAIT. q1 establishes q_avail 1 and h1, and the invalid first proof is excluded. A missing physical ancestor at q2 keeps G at h1. When ancestry completes at q3, canonical target order produces h2–3 then h4, including h2 padding. q4 preserves q_avail 3 and all public positions of the two valid proof variants. q5 accepts h9 one microsecond before cutoff and caches successor candidates. SessionEnd at 24,000,000 μs retains h9 and selects window 11–20. At q6 the matching successor retains q_avail 5 and extends G through h11 at the current key, with h10 as padding. The old candidate whose publication equals cutoff and the successor with a different h9 anchor stay excluded. Expire for h12 at 52,000,000 μs closes the branch before same-key q7, which retains HALT and h11.

**Execution PASS in both fixtures.** The checker reads raw body, parent and proof fields from PELG and checks their provenance at each pc. It derives account changes independently from the fixed deposits and ordinary transfers, verifies pre/post roots and reconstructs native hashes. At each physical height, the two raw duplicate-nonce transfers remain in the virtual block and exactly one enters native Kept. Physical heights are `1,10,11,20` for G1 and `1,3,4,9,11` for G2. Other accepted heights have empty ordinary transactions and Kept.

## Scope

The finality label is `gfr-07-noncryptographic-rule-fixture-v1`. L1 control records are synthetic. The component's documented limits are 4,096 source blocks, 64 MiB Espresso source log, 4,096 L1 fixture rows and post-genesis G entries, and 8,193 records. The fixture runs exercised the finite source ranges above. Their coverage supports replay semantics and deterministic wall-pause comparison. Cryptographic validation, real network timing, E-L/E-TS service guarantees and general liveness or safety remain outside this evidence.

**OPEN** includes V22 for actual 04 native certificates, 09 authenticated nonempty L1 control coverage, VG39 and 12/13 position-bound ResumeCut and material staging, 08 controlled real-producer regular integration, downstream compatibility and overall 07 acceptance. This matrix checks the unsupported-control boundary. The separate V18 checkpoint conformance evidence is in the independent M2 review.
