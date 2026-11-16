# Positioned RESUME replay vectors

The fixtures exercise production GFR AdvanceGlobal and the Session Manager plan/commit/action entry. They use the previously independent native execution fixture and exact-subject structural Final witnesses. The verified-reader seam supplies synthetic positioned cuts. Its fixture digest binds the full synthetic D history, while the projection exposes D through the target. Real DA and Anvil receipt authentication, D execution with actual accepted payloads, and native 15-worker proofs are covered by the other M4 acceptance runs.

The source-first numerical cases are in [expectations.md](expectations.md). The independent suite covers the following boundaries.

- Exact L1 and Espresso coverage, positioned cut, DEP and Plan WAIT with no partial GFR or lifecycle advance. Two independent readers agree once prerequisites are complete.
- Native-common h1 is retained, while conflicting h2 adopts DER provenance and D.h3 above the target is excluded from the projection. The continuation anchor uses the derived identity even when the final shared native block retains its old physical provenance.
- Two same-key RESUMEs execute by L1 tx/log position before Expire. A same-key Force reads the effective derived anchor. An exact timer applies after the sole control, while strictly past Expire and SessionEnd align and halt within the control without Plan. Later regular handoff inherits the activated RESUME floor below the control's inclusion height.
- Checkpoint continuity detects missing or conflicting historical cuts. Mutations of the position, matching CursorSet position, including block, transaction, configuration or target reject atomically.
- Manager waits on missing cut material, rejects mismatched digest and event order, stages old Running, Preparing and Paused terminal intents before Close release, and prepares the replacement from the same cut anchor. Old outstanding Enter, repeated Enter and delayed readiness preserve the permanent guards.

From the named clean M4 worktree, use the M4 isolated cache and target and the host-wide lock.

```sh
m4_job=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m4-resume
m4_rust=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin
PATH="$m4_rust:$PATH" CARGO_HOME="$m4_job/cargo" CARGO_TARGET_DIR="$m4_job/target" \
  CARGO_BUILD_JOBS=2 RUSTC="$m4_rust/rustc" RUSTDOC="$m4_rust/rustdoc" \
  flock /tmp/megaroation-poc-heavy.lock cargo test --locked --offline \
  --manifest-path poc/verification/m4-resume/replay-vectors/Cargo.toml -- --nocapture --test-threads=1
```

Expected result is 10 passed. Prep uses staged source and is not formal candidate evidence.
