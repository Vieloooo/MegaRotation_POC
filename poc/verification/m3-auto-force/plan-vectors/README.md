# M3 historical Plan acceptance vectors

These deterministic vectors call the production GFR initialization, advance and checkpoint restore entry points and the authorization provider's historical Plan entry points. They reuse the independent M2 input and execution fixtures. They compute the expected authorization digest from the frozen preimage and compare the selected descriptor, including its committee, leader rule and CID.

| Contract | Evidence in this suite |
| --- | --- |
| AF09 | Mask and order, first eligible region without a source, completed and missing historical inputs, zero mask, malformed policy, and the contribution-sensitive R0 to R1 transition from an accepted h1 |
| AF10 | Policy inactive until the public activation key, same-block L1 ordering with stale targets on either side, HALT protection, earlier Plan and CID retained after later policy activation, and regular selection from the policy active at that replay position |
| AF11 | Separate readers with opposite delivery orders, immutable historical checkpoint cuts, worker-facing basis verification with individual corruptions, and GFR admission of the correct historical descriptor and body |

The [numerical predictions](expectations.md) precede the implementation review. Headers and controls are deterministic finalized fixture inputs. The focused publication check uses the exact-subject noncryptographic fixture witness. The separate M3 integration evidence runs real workers, native Simplex Final and EVM transactions. No simulation seed or wall-clock assumption applies to these focused vectors.

Run from the named candidate worktree. The same lock is shared with the primary scenario and other PoC heavy work.

```sh
M3_JOB=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m3-auto-force
M3_RUST=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin
env PATH="$M3_RUST:$PATH" RUSTC="$M3_RUST/rustc" RUSTDOC="$M3_RUST/rustdoc" \
  CARGO_HOME="$M3_JOB/cargo" CARGO_TARGET_DIR="$M3_JOB/target" CARGO_BUILD_JOBS=2 \
  flock -w 900 /tmp/megaroation-poc-heavy.lock "$M3_RUST/cargo" test --locked --offline \
  --manifest-path poc/verification/m3-auto-force/plan-vectors/Cargo.toml \
  -- --nocapture --test-threads=1
```
