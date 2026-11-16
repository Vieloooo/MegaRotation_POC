#!/usr/bin/env bash
set -euo pipefail

cd /home/jy/.codex/worktrees/1e2f/megaroation_poc
expected_candidate=e41b81a8d68d67216ea940beeb614135e808e95d
test "$(git rev-parse HEAD)" = "$expected_candidate"
git diff --exit-code "$expected_candidate" -- poc/crates/viewer-gfr

export PATH=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:"$PATH"
export CARGO_HOME="$PWD/.build/cargo"
export CARGO_TARGET_DIR="$PWD/.build/target"
export CARGO_BUILD_JOBS=2

evidence_dir="$PWD/.build/artifacts/verification/m1-e41b81a-independent-v1"
test ! -e "$evidence_dir"
mkdir -p "$evidence_dir"
cp poc/verification/viewer-gfr/independent-harness/run-m1-review.sh "$evidence_dir/command.sh"
git rev-parse HEAD > "$evidence_dir/candidate.txt"
git status --short > "$evidence_dir/status-before.txt"
rustc --version > "$evidence_dir/rustc.txt"
cargo --version > "$evidence_dir/cargo.txt"
sha256sum poc/docs/contracts/task-contract-viewer-gfr-v1.md \
  poc/verification/viewer-gfr/interface-vectors-v1.json \
  poc/verification/viewer-gfr/source-first-expectations-v1.md \
  poc/verification/viewer-gfr/independent-harness/src/main.rs \
  poc/crates/viewer-gfr/src/api.rs poc/crates/viewer-gfr/src/codec.rs \
  poc/crates/viewer-gfr/src/lib.rs > "$evidence_dir/input-hashes.txt"

# One host-wide heavy run. This script is the sole holder; no nested lock.
exec 9>/tmp/megaroation-poc-heavy.lock
flock 9
cargo generate-lockfile --manifest-path poc/verification/viewer-gfr/independent-harness/Cargo.toml --offline \
  > "$evidence_dir/lock.stdout" 2> "$evidence_dir/lock.stderr"
set +e
cargo run --manifest-path poc/verification/viewer-gfr/independent-harness/Cargo.toml --locked --offline -- --capacity \
  > "$evidence_dir/harness.jsonl" 2> "$evidence_dir/harness.stderr"
harness_status=$?
set -e
printf '%s\n' "$harness_status" > "$evidence_dir/exit-code.txt"
git status --short > "$evidence_dir/status-after.txt"
git diff --exit-code "$expected_candidate" -- poc/crates/viewer-gfr > "$evidence_dir/candidate-diff.txt"
sha256sum poc/verification/viewer-gfr/independent-harness/Cargo.lock > "$evidence_dir/lock-hash.txt"
printf 'evidence=%s\nexit=%s\n' "$evidence_dir" "$harness_status"
exit "$harness_status"
