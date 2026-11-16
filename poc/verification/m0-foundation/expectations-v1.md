# M0 independent expectations v1

2026-11-16, Asia/Shanghai. Prepared by `bft_verifier` (named role: GPT-6 Astra/xhigh), before reading an integrated candidate or Builder explanation. This is a frozen acceptance checklist, **not a PASS result**. The verifier applied regional-bft-conformance and read the source checkout's AGENTS/DESIGN, v2 handoff/setup/design, milestones README/M0 plan, pinned Regional BFT DESIGN, and the original component contracts/source-first expectations and independent reports. No implementation or Git state was modified during preparation. No separate runtime-model attestation is exposed by these tools.

## Authority, inputs and scope

M0 combines reusable components. It does not implement M1's 15 healthy workers/three regular rotations or M2's active healthy Force. P0 remains 3 committees of 5, 4 distinct valid signers, fixed 10-height windows, bounded 1 ms clock error separated from RPC latency. Commonware is 2026.10.0; Rust is 1.95.0. The pinned BFT source is `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`.

Old main: `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`. Expected component ancestors, in normal merge order:

| Input | Delivery SHA | Actual source revision behind recorded component evidence |
| --- | --- | --- |
| 03 | e9c15e04bc83b335668f3ecd837367aa87344a89 | e9c15e04bc83b335668f3ecd837367aa87344a89, final verifier and Tester; preceding reviewed source e30f5ed3b3352d58005f672bfebf22e9810552fb |
| 02 | e1a435712ec492cae2bbff120a51ffd507eebf45 | 4fd6f115bd969b0a8f252d7933edd21c27f00a8f |
| 06 | 4b034dfbb6c3e1c7143208347b546daab8dfe298 | 8ba78f1ace3b383caa1c4360401c645d25907326 |
| 07 | 788291735ce6db3c3eb78b4594a28d730cce71cd | 8907ad64d882673d6ef2142defdf6bff2e208a71, source/harness identical to de628c2e3536c6d7b10ae2f8513b7014623a0e45 |
| 16 | 5c34b400ba5245d7775a281566e21bf59708b147 | 6248072b3fd0eee1ae468f0961a4d3589291cba6; Rust component evidence inherited from source-identical 8750535d8f02e920353c1ca827c78046e72d578a |
| 09 | c443ae925ed549b046731c2d8ef3dd9ce0311d24 | 87daee8e9e77bbe27c337383d873a5eaf4a694ba |
| Current management documents | d5cbecb24836af80f8cd5414052043a17ca2ae17 | Authorized latest commander state; retains primary milestone plan ancestor 4a9a6b0a92a11e5778833988bb6524d929f4608e |

07 already contains 03 and 06 as ancestors. 16 contains 06. Check actual ancestry; no cherry-pick duplication. No merge of 01 recovery WIP, 11 or 12 branch increments. The old-main native prototype may already contain recovery-named files; preservation of baseline files is not authorization to import later recovery changes. Source checkout, other worktrees and submodule history/worktree stay untouched. Commander state/plan files must equal the d5cbecb versions in the integrated tree, not stale copies inherited from component branches. The authorized update from primary-plan ancestor 4a9a6b0 to d5cbecb changes only `poc/docs/commander/state.json`, independently confirmed using `git diff --name-only`; no feature source changed. Preserve 4a9a6b0 as the primary-plan ancestor.

## Source identity and evidence binding

For each delivery, record `git diff --name-status TESTED DELIVERY -- PATHS` and the per-path Git blob/tree identifiers. Separately compare DELIVERY to integrated candidate. A component crate tree alone is insufficient: include its manifest/lock, transitive local dependencies, build scripts/proto, and the harness/checker/input files actually run. Record wrapper-only packaging differences separately. Any behavior-affecting conflict resolution requires review and rerun of its affected consumers.

Required path groups:

- 03: `poc/crates/{types,authorization,core}`, `poc/Cargo.toml`, `poc/Cargo.lock`, `poc/proto`, `poc/verification/mock-plan-execution`. Final independent manifest SHA256 `a25ee38b4ea5eefd36e4302eedca04e5cc282f00008e537a7803b98ccc8c4da7`; final Tester manifest `47da8a35da3f22b4295916865c6d30b1f66db5be05ed9c625a71c47f072ebfa1`. Both live under `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03/artifacts/`, respectively `verification/full03/e9c15e04bc83b335668f3ecd837367aa87344a89/run-manifest.json` and `simulation/e9c15e04bc83b335668f3ecd837367aa87344a89/run-manifest.json`. The original final feature subtotal was 32 actual-library tests; workspace 52 passed/11 ignored, not 63 passed.
- 02: `poc/crates/sim-transport`, `poc/verification/sim-communication/kernel-v1`. Production tree at tested/delivery revisions is `3d2ff54ca9231d2611e424599b0db73170faa9a2`. Historical independent 46/46 and 12-run fixture matrix; native workers=0. Mock DurabilityReceipt is not real fsync/root grant evidence.
- 06: `poc/crates/{publisher,sim-espresso}`, independent harness and frozen vectors. Historical independent six cases plus four actual-file recovery cases. Publisher/reader runtime adapter, real FINAL, canonical domain adapter and GFR integration remain separate gates.
- 07: `poc/crates/viewer-gfr`, independent harness, simulation checker/input files, and transitive 03/06 paths. Historical 117 checks, 19 pinned implementation/harness hashes; full 12-run G1/G2 matrix at 8907ad6. Fixture profile is explicitly `gfr-07-noncryptographic-rule-fixture-v1`.
- 16: `poc/crates/espresso-compatibility`, `poc/scenarios/espresso-compatibility`, independent harness, vectors/checkers. Official fixed-source recording is `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/ec16-official-6248072-run02/bundle`. Original recording candidate must remain 6248072; current replay executable belongs to M0 SHA. Do not rewrite recording provenance. Official release source fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171, protocol 0.3/API v1/namespace160016.
- 09: `poc/contracts`, `poc/crates/l1-adapter`, independent ABI/receipt checkers and literal inputs. Historical A01–A17 are finite trusted local Anvil component results, with A18/A19 unresolved. This task does not start its live control chain.

Audit historical manifests/artifact hashes without overwriting them. Historical source-identical results may support bounded compatibility, but cannot be relabeled as a new M0 run or expanded to end-to-end acceptance.

## Source-derived invariants at integration boundaries

1. 03 retains one canonical checked public identity/Plan/descriptor/execution/Merge API. Real PoP checks are not FINAL certificate checks. `FinalVerifier` has no default-true production implementation. Shared objects/encoding are not forked by merging.
2. 02 opaque delivery, independent readers, sample freshness/lease, timers and durable custody remain separate from public ordering and signature validation. Pure component confirmation cannot be reported as real controlled worker grant/Ack integration.
3. 06 only records a result after a complete validated Offer and actual complete publisher reader receipt. ACK/server finalization alone cannot record. STOP and recorded history survive reopen; public stop inhibits fresh or prepared dispatch. Incomplete public result differs from missing local public bytes. Reader A cannot fill B's missing prefix.
4. 07 follows public key → controls → Expire → SessionEnd → Espresso. Watermark and complete L1 controls precede event processing. WAIT at a known blocked event leaves all that event's mutations/lifecycle/pc atomic; absent ordering coverage may produce explicit frontier WAIT. q_avail is first public complete proof+physical-body position; acceptance key may be later. Every new virtual height, including padding, must pass Expire. Full-window successor and SessionEnd preserve fixed windows and reference/anchor separation. Unsupported control stops at its exact position. No wall time triggers GFR.
5. G1 oracle: q1 can accept h1–10 plus cached h11–20 with four distinct lifecycle records in one atomic event; original q_avail values 1 and 0 survive. Missing origin or successor Plan gives WAIT at q1 with no partial effects. SessionEnd carries h20 and fixed window31–40; Expire preserves h20/HALT; synthetic Resume stops before later Force/espresso. G2 oracle: proof-only q0 gives no accept; q1 establishes q_avail1/h1; missing physical ancestry waits for public completeness at q3; canonical extensions include h2 padding; SessionEnd at cutoff retains h9 and window11–20; cached successor q_avail5 accepts h11 with h10 padding; Expire preserves h11/HALT before same-key q7. A/B semantic projections at same pc agree and full stdout pairs for pause0/2 agree bytewise.
6. 16 preserves source q/theta/native transaction indices and complete independent reader prefixes. Proposed/finalized virtual ticks and live_at_proposal remain Unknown/Unsupported. Recording/replay public audit is expected SHA256 `43f106c905bb0877c2131bd7926eb099701d4e3936c0845e96ec7fb36a4e7cf5` (889 bytes; six complete blocks, two native publications). HTTP and payload hash consistency do not prove QC/VID/Regional FINAL, E-TS/PUB bounds or virtual-clock support.
7. 09 ABI and exact L1 position/receipt/finality/delivery boundaries stay intact. Contract inclusion is not GFR activation. No invented ResumeCut or D. Driver durability/root integration is not inferred from in-memory reconciliation.

## Required rerun commands on the named M0 candidate

All commands below use CWD `/home/jy/.codex/worktrees/440b/megaroation_poc`. Set task-local environment once; `M0_DATA` and `M0_SHA` are task-specific variables, not global daemon configuration. `CARGO_HOME` must use the prepared public-cache setup, never credentials. Use a new evidence directory for every attempt and retain failures. The dedicated data path is `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m0-foundation`.

```sh
export REGIONAL_BFT_RUST_BIN=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin
export PATH="$REGIONAL_BFT_RUST_BIN:/home/jy/miniconda3/bin:$PATH"
export CARGO_BUILD_JOBS=2
export CARGO_HOME="$M0_DATA/cargo"
export CARGO_TARGET_DIR="$M0_DATA/target"
export RUST_TEST_THREADS=1
```

Record actual SHA, branch, status, all command argv, return codes, duration, seed/fixture, inputs and logs/binary hashes. Require unchanged tracked source and lockfiles before/after. Do not use wrappers that overwrite target/cache with another component's job directory. `poc/scripts/bootstrap.sh` forces `$checkout/.build/{cargo,target}`; either bind that directory correctly or use direct cargo commands below.

**Root workspace / 03** (root Rust workspace lives at `poc/Cargo.toml`):

```sh
flock /tmp/megaroation-poc-heavy.lock cargo test --manifest-path poc/Cargo.toml --workspace --locked --offline -- --nocapture --test-threads=1
python3 poc/verification/mock-plan-execution/implementation-v1/check_m1_vectors.py
python3 poc/verification/mock-plan-execution/implementation-v1/check_m2_vectors.py
python3 poc/verification/mock-plan-execution/implementation-v1/check_identity_vectors.py
python3 poc/verification/mock-plan-execution/implementation-v1/check_extended_vectors.py
```

Require 03's 32 actual-library cases and original I1 compatibility coverage to pass. Record all ignored cases honestly.

**02 independent kernel**:

```sh
flock /tmp/megaroation-poc-heavy.lock cargo test --manifest-path poc/verification/sim-communication/kernel-v1/Cargo.toml --locked --offline -- --test-threads=1
```

Expected historical suite size46. No native engine/worker claim.

**06 crates and independent reader/publisher**:

Set `PE_VERIFY_DIR` and `PE_TEST_EVIDENCE_DIR` to new separate directories inside the M0 evidence root before running. The latter retains actual durable reopen/fault files.

```sh
flock /tmp/megaroation-poc-heavy.lock cargo test --manifest-path poc/crates/sim-espresso/Cargo.toml --locked --offline -- --nocapture --test-threads=1
flock /tmp/megaroation-poc-heavy.lock cargo test --manifest-path poc/crates/publisher/Cargo.toml --locked --offline -- --nocapture --test-threads=1
flock /tmp/megaroation-poc-heavy.lock cargo test --manifest-path poc/verification/publisher-espresso/independent-harness/Cargo.toml --locked --offline -- --nocapture --test-threads=1
```

Full publisher tests include `recovery_tests::` four cases; report their execution rather than unnecessarily repeat them.

**07 independent interface checker, M2 harness and four representative G1/G2 replay runs**:

```sh
flock /tmp/megaroation-poc-heavy.lock cargo run --manifest-path poc/verification/viewer-gfr/independent-harness/Cargo.toml --locked --offline -- --capacity
flock /tmp/megaroation-poc-heavy.lock cargo run --manifest-path poc/verification/viewer-gfr/independent-harness/Cargo.toml --bin m2 --locked --offline
flock /tmp/megaroation-poc-heavy.lock cargo run --manifest-path poc/crates/viewer-gfr/Cargo.toml --example replay --locked --offline -- G1 7 0
flock /tmp/megaroation-poc-heavy.lock cargo run --manifest-path poc/crates/viewer-gfr/Cargo.toml --example replay --locked --offline -- G1 7 2
flock /tmp/megaroation-poc-heavy.lock cargo run --manifest-path poc/crates/viewer-gfr/Cargo.toml --example replay --locked --offline -- G2 7 0
flock /tmp/megaroation-poc-heavy.lock cargo run --manifest-path poc/crates/viewer-gfr/Cargo.toml --example replay --locked --offline -- G2 7 2
```

Save each stdout unmodified as G1-7-0.jsonl etc, separate stderr. Run the unchanged original checker function `semantic_trace(raw, scenario, seed)` from `poc/verification/viewer-gfr/simulation/check-traces.py` via importlib for all four files, and byte-compare each pause pair. A small M0 wrapper may import this function and record M0 provenance independently. **Do not run the legacy capture/checker CLI unchanged**: both embed historical 8907ad6/branch, three seeds and old workspace. Do not monkeypatch them to falsely say the old run happened. The semantic function contains the source-derived execution/q_avail/WAIT/lifecycle/A-B oracle and does not need historical HEAD. The interface checker includes its independently sized 64 MiB `--capacity` boundary. Historical expected M2 count117; representative replay scope four cases/two pause pairs, not12/six. Hash the imported checker and actual input files.

**16 offline component, full fixed-input boundary matrix and official recording replay**:

```sh
flock /tmp/megaroation-poc-heavy.lock cargo test --manifest-path poc/crates/espresso-compatibility/Cargo.toml --locked --offline -- --test-threads=1
flock /tmp/megaroation-poc-heavy.lock cargo test --manifest-path poc/verification/espresso-compatibility/independent-harness/Cargo.toml --locked --offline -- --test-threads=1 --nocapture
flock /tmp/megaroation-poc-heavy.lock cargo build --manifest-path poc/crates/espresso-compatibility/Cargo.toml --locked --offline
flock /tmp/megaroation-poc-heavy.lock python3 poc/verification/espresso-compatibility/run_offline_checks.py --binary "$CARGO_TARGET_DIR/debug/espresso-compatibility" --output "$M0_EVIDENCE/ec16-offline" --candidate "$M0_SHA"
flock /tmp/megaroation-poc-heavy.lock "$CARGO_TARGET_DIR/debug/espresso-compatibility" replay /mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/ec16-official-6248072-run02/bundle first "$M0_EVIDENCE/ec16-official-replay"
flock /tmp/megaroation-poc-heavy.lock "$CARGO_TARGET_DIR/debug/espresso-compatibility" replay /mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/ec16-official-6248072-run02/bundle first "$M0_EVIDENCE/ec16-official-replay-paused" --pause-ms 37
flock /tmp/megaroation-poc-heavy.lock "$CARGO_TARGET_DIR/debug/espresso-compatibility" replay /mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/ec16-official-6248072-run02/bundle requery "$M0_EVIDENCE/ec16-official-replay-requery"
python3 poc/verification/espresso-compatibility/check_capture.py /mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/ec16-official-6248072-run02/bundle --reader first --candidate 6248072b3fd0eee1ae468f0961a4d3589291cba6 --audit-file "$M0_EVIDENCE/ec16-official-replay/public-audit.bin" --projection-file "$M0_EVIDENCE/ec16-official-replay/public-projection.json" --output "$M0_EVIDENCE/ec16-recording-independent.json"
python3 poc/verification/espresso-compatibility/check_capture.py /mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/ec16-official-6248072-run02/bundle --reader first --candidate 6248072b3fd0eee1ae468f0961a4d3589291cba6 --audit-file "$M0_EVIDENCE/ec16-official-replay-paused/public-audit.bin" --projection-file "$M0_EVIDENCE/ec16-official-replay-paused/public-projection.json" --output "$M0_EVIDENCE/ec16-recording-independent-paused.json"
python3 poc/verification/espresso-compatibility/check_capture.py /mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/artifacts/ec16-official-6248072-run02/bundle --reader requery --candidate 6248072b3fd0eee1ae468f0961a4d3589291cba6 --audit-file "$M0_EVIDENCE/ec16-official-replay-requery/public-audit.bin" --projection-file "$M0_EVIDENCE/ec16-official-replay-requery/public-projection.json" --output "$M0_EVIDENCE/ec16-recording-independent-requery.json"
```

Expected previous counts crate8, independent7, offline30. The official `--candidate` above deliberately checks the **recording's original provenance**, while M0 wrapper records the **current replay binary SHA**. All three replay audits and public projections must match byte-for-byte; each replay is independently checked against the source recording. No official service is started; M1 separately must publish its actual regional-result bytes.

**09 compile/tests and ABI/interface preservation, no live Anvil needed**:

```sh
flock /tmp/megaroation-poc-heavy.lock cargo build --manifest-path poc/crates/l1-adapter/Cargo.toml --locked --offline
flock /tmp/megaroation-poc-heavy.lock cargo test --manifest-path poc/crates/l1-adapter/Cargo.toml --locked --offline -- --test-threads=1
flock /tmp/megaroation-poc-heavy.lock /mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0/forge build --root poc/contracts --use /mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0/solc --offline --threads 2 --out "$M0_EVIDENCE/forge-out" --cache-path "$M0_EVIDENCE/forge-cache"
python3 poc/crates/l1-adapter/tests/check_abi.py "$M0_EVIDENCE/forge-out"
python3 poc/verification/anvil-l1/golden_vectors.py check --cast /mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0/cast
```

09 historical real receipt matrix is source-identity-checked reused evidence. This run verifies build, default tests and ABI/golden interface only, not newly executed A01–A17. Report ignored real-receipt tests as ignored, not passed; do not start Anvil to satisfy them.

**Ordinary native compatibility:** independently compare `poc/prototypes/i1b-runtime` against old main and historical fec7ade. Old main's source is not the fec7ade source, so its 66-run/33-pair historical PASS never transfers. The lead selected the currently supported ordinary no-restart healthy profile, seed7 with wall pause0/2 ms. Execute those two bounded runs plus relevant Clock/Grant/Ack acceptance at current M0 SHA under the same lock; compare the complete applicable virtual traces, and record the existing CLI pause bound (at most2). Record exact discovered command/profile and whether scripted signers are present. No auto empty-journal restart; no full01 R1–R18 recovery matrix required. If the current interface lacks a supported normal smoke, report the specific gap instead of inventing a command or claiming a historical PASS.

## Release gate

The independent reviewer checks the integrated named SHA and conflict/source identity record before Tester uses that same SHA. Any later runtime/harness change requires affected re-review and rerun. Report-only additions may retain the tested implementation SHA only with an explicit complete source/harness identity audit; do not say a different HEAD was literally executed.

Accept M0 only when the selected reruns pass, original restrictions remain explicit, and the final candidate is reproducible. Before CAS main update, separately require old main unchanged and no worktree with main checked out; use old-value compare update, never force over a changed main. Failure leaves the integration branch and evidence intact and main unchanged. No push, no deployment, no source commander edits.

Open after M0: full I1/recovery; real native FINAL adapter and q4 signature boundary; 15 healthy workers/controlled services; Manager/Publisher/GFR real producer wiring; official M1 result publication; M2 real Force activation; D/RESUME/Batcher/DA; 16 PE timing/QC/VID. These are not M0 regressions merely because they are outside the accepted component scope.
