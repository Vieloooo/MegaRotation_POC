# 07 Viewer GFR interface release v1

2026-11-16 (Asia/Shanghai). **M1 interface PASS; full feature 07 OPEN.** This release records the first consumable interface milestone and does not increase the complete accepted-feature count. Subsequent M2 verification and Tester results are recorded in the [component handoff](viewer-gfr-component-handoff-v1.md).

## Exact revisions and API

- Worktree `/home/jy/.codex/worktrees/1e2f/megaroation_poc`, branch `codex/poc-viewer-gfr`.
- Clean intake `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`; normal 03 then 06 merges produce `335553359cd39a3a7494fbd9760436cdb2c41a33`.
- Frozen contract/source expectations: `186501dbb8156a52bde1832dfaaa1a714c52bf68`; exact independent interface vectors/review: `673d118fde34557f06f0bd3c447b4c799f601c7a`.
- **Consumable interface and independently tested source SHA: `e41b81a8d68d67216ea940beeb614135e808e95d`.** The verifier recorded an empty diff for the crate before/after its run. Follow-up report/harness-lock commits do not retroactively change the tested revision.
- Package `poc-viewer-gfr`, library `poc_viewer_gfr`, isolated manifest/lockfile at `poc/crates/viewer-gfr/`.

The crate provides `compare_positions`, `encode_position`, `decode_position` (GFR-POS-1), `lifecycle_projection`/`lifecycle_id` (GFR-LIFE-1), and `encode_publication`/`decode_publication` (GFR-PUBLIC-1). It imports the existing 03 typed objects and canonical nested bytes. Force and Resume share the control sorting class and order by exact L1 position; conflicting kinds at one position reject. Lifecycle identity includes position and complete typed body, so multiple events at one pc remain distinct.

The typed boundaries include `ReplayReads`, `L1ControlReader`, `PlanProvider`, `ReplayServices`, `VerificationBasis`, `VerifierBinding`, `VerificationProfile`, `ReplayStatus`, and chain/responsibility/record types. Verification prerequisites have an explicit `InputStatus<()>` boundary before the existing 03 boolean `FinalVerifier`. Genesis is required to bind the expected profile independently of verifier availability. These interface declarations do not yet implement the M2 reducer or claim any native certificate validation.

Exact wire formats, size bounds and public semantics are in [GFR-07-v1](contracts/task-contract-viewer-gfr-v1.md). Consumers must reuse these exported codecs rather than create another position/lifecycle/publication encoding. Publisher wiring and later 10/13 control handlers remain separate integration work.

## Independent evidence

Actual named roles: Docer and independent Verifier `bft_spec`/`bft_verifier` (GPT-6 Astra/xhigh); single Builder `bft_builder` (GPT-6.1 Sol/high). Main requested Astra/ultra; tools here do not independently expose its runtime model metadata. No model substitution or additional user chat was used.

Verifier derived expectations and exact byte vectors before candidate inspection. It then reviewed `e41b81a` and ran the separate harness with Rust/Cargo 1.95.0, `CARGO_BUILD_JOBS=2`, task-local cargo/target, and one `flock /tmp/megaroation-poc-heavy.lock`. **167 passed, 0 failed, exit 0**, including the optional aggregate-capacity case: 67,130,205 projected bytes exceed the 64 MiB bound and are rejected without panic. There is no simulation seed for this deterministic interface vector suite.

Exact command, environment, source/input hashes, lockfile hash, stdout, stderr, exit code and clean candidate diff are retained at:

`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07/artifacts/verification/m1-e41b81a-independent-v1`

The executed harness command inside the recorded lock was:

```sh
cargo run --manifest-path poc/verification/viewer-gfr/independent-harness/Cargo.toml --locked --offline -- --capacity
```

- Raw `harness.jsonl` SHA-256: `94f6f65dad04932d5c946992016844e76d2fd4d536cbd3e5a1b9e392a3aba415`.
- Exact `command.sh` SHA-256: `cbe0de0a114d8faf2a573ea2ca4435b18ffbac104c4cc62733d964e228c584ec`.
- Contract SHA-256: `0a622d8504f48ad63305b917ee30da0d5e07f366512bd10018345ecedd5bfabb`.
- Independent vectors SHA-256: `0409be69fad24e8726df7e952cd0919161e943dbfe05b30431343beba7ce1926`.

M1 tests cover codecs/comparison and capability shapes, not GFR state transitions, checkpoint execution, G1/G2, actual Force/RESUME, Manager delivery, native final certificates, or production integration. The 03 nested descriptor golden used for encoding has its original non-cryptographic caveat.

## Remaining gates and latest routing decision

The subsequent M2 handoff separately records pure-rule verification and G1/G2 on one verified candidate; this M1 evidence alone does not establish those results. 04's real native FinalVerifier/certificates remain unavailable and OPEN; fixtures never satisfy that gate. Unsupported Force/RESUME must be reported at their position rather than skipped. Future RESUME lifecycle batches require complete materials before allocation; actual 13 dispatch/durability remains OPEN.

The coordinator's 2026-11-16 restart-free route (`0c15aac`, handoff update `32a3eb6b31ff8c1dfe2d05dbc2ed26c0c031bb77`) was read from the source checkout without merging unrelated changes. It confirms that this work does not wait for complete 01 recovery. Future genuine 04 certificates may come from a continuously running healthy-worker profile; this does not waive clocks, leases, grant/Ack, signatures, close gates, or independent replay. Current 07 ownership and pure-rule scope are unchanged. No main merge or push has been performed.
