# Feature 16 candidate review

2026-11-16, Asia/Shanghai. **PASS for candidate `8750535d8f02e920353c1ca827c78046e72d578a` under the frozen trusted-local-API compatibility profile.** No actionable implementation finding remains. This verdict authorizes Tester to run the official local-node matrix on the same candidate; actual official publication/requery evidence is still a separate G-API gate at the time of this review.

## Independence and revision

The verifier derived [expectations](expectations-v1.md) and [literal vectors](vectors-v1.json) from v10/v2, the accepted 06 API and pinned official Espresso source before implementation. Preparation commit: `89925a6`; interface commit reviewed read-only: `9747268e52410c1e7ce6b79d96df2ca858598a78`. The tested implementation HEAD is exactly `8750535d8f02e920353c1ca827c78046e72d578a`; implementation/scenario paths remained clean. Only this verification directory was written, with no source changes or verifier commits. New harness/review files remain untracked until the final documentation delivery so Tester can retain the exact candidate HEAD.

Frozen expectation hashes remain unchanged:

- `expectations-v1.md`: `63fe4a579d40279e5b54987b956ab68f8f7c773fb90275e141903a9f834abf7d`.
- `vectors-v1.json`: `fd5df91059168c8b31929a46368d313606ef11eea0c0ee7b011f1e07b6e0cc4a`.

The named `bft_verifier` role is configured as GPT-6 Astra/xhigh. That is role/configuration evidence, not an independently observed runtime-model attestation. Worktree: `/home/jy/.codex/worktrees/d2be/megaroation_poc`; branch: `codex/poc-espresso-compatibility`. Rust: `rustc 1.95.0 (59807616e 2026-04-14)`. Build jobs2; all Rust builds/tests and the offline matrix used `flock /tmp/megaroation-poc-heavy.lock`. Seeds: N/A, literal fixture schedules. No other feature or Regional BFT source was edited.

## Source review

Reviewed the complete `reader.rs`, `bundle.rs`, CLI, pinned image preparation, isolated local launcher and capture script against the frozen contract and pre-code expectations.

- Source identity binds the exact official commit/image/protocol/API, run, genesis and namespace. Native flat positions and compact namespace projection positions stay distinct. Only a complete q0-based prefix is exposed, including after out-of-order delivery. Height hints and submit ACKs cannot advance it.
- Response parsing checks fixed header version/height, checked seconds-to-microseconds conversion, exact block/leaf header agreement, transaction count/index/hash/namespace and payload bounds. Conflicts reject before installation. Repeats preserve the first references. The pinned local API remains the declared authority for finality/count/list; this code does not verify QC/VID commitments.
- Digest bytes follow the frozen EC16-AUDIT-1 recipe and exclude reader/wall/evidence metadata. All native transactions, including foreign namespaces and duplicates, remain covered. Unknown times remain a type with only the Unknown variant; PE-LOG and PE-READER exports return Unsupported.
- Bundle loading validates pin/source/record ordering, hashes every referenced file, rejects missing or escaped paths including symlink escape, and applies raw/manifest/unique-file aggregate bounds before unbounded reads. Official mode binds recorded origins to the configured task-local endpoints and pinned OCI metadata; manifests are not signatures.
- The launcher checks the exact clean source candidate, verifies the official image artifacts, uses isolated user/network/PID namespaces with only loopback, explicitly launches its own Anvil, records its own child PIDs, and terminates only those process groups. Native execution is labelled as such. The capture script makes new connections, retains failures/requeries, stops owned services before replay, and does not claim actual proposal/live/finalization times.

## Independent execution results

| Check | Result and evidence |
| --- | --- |
| External Rust harness | **7/7 PASS**, 7.12s test execution. Uses literal expected bytes and independent delivery/mutation sequences. |
| Frozen audit vectors | All five prefix byte strings and hashes match exactly, including empty prefix and q gaps. Four-block digest `98a1a83ae5f930ef101e6babcec2a28475c855d94c48f9ecf759e5ff3b4c6f39`. |
| Independent offline/bundle/CLI matrix | **30/30 PASS** in final run `offline-02`. Includes two replay pauses0/17ms, separate reader replay, exact public-object comparison, metadata-only changes, logical gaps, corruption and path/HTTP/field negatives. |
| Independent source oracle | Python reconstructs native tuples directly from raw block/leaf/per-position responses, separately decodes the pinned namespace/transaction-table format, and checks raw payload/count/order/bytes against transaction responses before computing audit bytes. It uses no candidate library or candidate output as its expected result. All four fixture blocks and all three successful replays match. |
| Native payload/count bounds | 65,536 bytes accepted and65,537 rejected, including foreign namespace; 1,024 native positions accepted and count1,025 rejected; duplicate records do not recount retained bytes. |
| Selected envelope bound | Frozen116-byte overhead +16 entries, fifteen65,536-byte payloads +65,292-byte final payload equals1,048,576; final payload65,293 rejects. No fake PELG block is constructed. |
| Distinct-height/retention bounds | 4,096 pending heights accepted,4,097 rejected; exactly64MiB of retained native payload accepted and one additional byte rejected. |
| Recording bounds | Raw request, raw response and manifest each accept16MiB and reject16MiB+1. Unique raw files plus manifest accept256MiB, including duplicate-reference deduplication, and reject256MiB+1. |
| Timing/PE capability | Unknown proposal/finalization/live values and explicit unsupported exports PASS. No official timing, virtual-clock, E-TS or T_E claim follows. |

The first offline run `offline-01` passed before additional checks were added to the independent Python oracle (manifest path/pre-read length, aggregate payload and projected envelope checks). **The full30-case matrix was rerun into `offline-02` with the final helper version.** `offline-02` is the final helper-version evidence; `offline-01` is retained as prior work, not relabelled with the final helper hash. Frozen vectors were also rerun independently after the oracle change.

Final helper hashes:

| File | SHA-256 |
| --- | --- |
| `check_capture.py` | `e42e68df3f40c991dcce199ba6fa53d03fd2c6d75f243d0bf3ace9c25ead157a` |
| `run_offline_checks.py` | `1a949eb5f9c8eb2b0025fab9b8aacd87598a4432ecd76756a8aa493c38234f34` |
| `independent-harness/tests/independent.rs` | `3e7abd76ac8fec65585f0a98dda3a93b6bbd01bd917783bd5b126444645f4c9f` |
| `independent-harness/Cargo.lock` | `f16db4763b12615cd1764b88b8010c8ac8d619eff9a0813cac47a045bab7ca8b` |
| Independently built candidate executable | `dcffb26c638cf82c75b4ace68b2e888cf5ce38218c66a33c4712c94139d873d1` |

## Reproduction and artifacts

Evidence root:

```text
/mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/verification/8750535d8f02e920353c1ca827c78046e72d578a/
  independent-01/  # exact command, rust/cargo versions, stdout/stderr, input hashes
  offline-02/      # 30 commands/outcomes, raw fixture bundles, oracle outputs, summary
```

Actual external harness command:

```sh
bash poc/verification/espresso-compatibility/run-independent.sh \
  /mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/verification/8750535d8f02e920353c1ca827c78046e72d578a/independent-01 \
  8750535d8f02e920353c1ca827c78046e72d578a
```

The harness generated its own lockfile once offline, then ran `cargo test --locked --offline -- --test-threads=1 --nocapture`. Its exact argv is in `independent-01/command.json`. Candidate CLI build used `cargo build --manifest-path poc/crates/espresso-compatibility/Cargo.toml --locked --offline` with the same Rust/cache/jobs and task-specific target `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/target/independent-verifier`. The final matrix command was:

```sh
flock -w 600 /tmp/megaroation-poc-heavy.lock \
  python3 poc/verification/espresso-compatibility/run_offline_checks.py \
  --binary /mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/target/independent-verifier/debug/espresso-compatibility \
  --output /mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/verification/8750535d8f02e920353c1ca827c78046e72d578a/offline-02 \
  --candidate 8750535d8f02e920353c1ca827c78046e72d578a
```

Use a new output directory for reruns. Capacity fixtures deliberately change from the equality case to the one-above case within a test; their commands, passing outputs, exact equality manifest and deterministic generator preserve both states without mislabelling the final mutated file as the earlier input. They are fixtures, not official network evidence.

For each official reader, Tester must invoke the stable independent oracle after capture (replace placeholders with actual paths):

```sh
python3 poc/verification/espresso-compatibility/check_capture.py BUNDLE \
  --reader first --candidate 8750535d8f02e920353c1ca827c78046e72d578a \
  --audit-file RUN/offline-first/public-audit.bin \
  --projection-file RUN/offline-first/public-projection.json \
  --output NEW-INDEPENDENT-FIRST-REPORT.json
```

Repeat with `--reader requery`, its corresponding audit/projection files, and a new report path. The oracle verifies OCI metadata pins for official packages, reconstructs the raw source list independently, and compares the candidate output only after deriving its own bytes. Any actual-run mismatch requires a finding and affected re-review, not changing frozen expected values.

## Limits and remaining gates

This is finite candidate/component evidence. All input sequences in the independent matrix are labelled non-cryptographic fixtures; the real-node setup probe was not counted as V13. G-API/actual publication and requery remains for Tester on this exact candidate. G-PE-TIMING remains Unsupported until an authorized06 interface change can express absent timing evidence. HotShot QC/VID security, E-TS/E-L guarantees, real Regional FINAL, virtual-time control of the official node and full06 integration were not verified. Full feature16 acceptance is a coordinator decision after the actual-run evidence and this compatibility gap are reported.
