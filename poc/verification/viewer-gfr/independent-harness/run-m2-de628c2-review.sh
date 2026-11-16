#!/usr/bin/env bash
set -euo pipefail
cd /home/jy/.codex/worktrees/1e2f/megaroation_poc
expected_candidate=de628c2e3536c6d7b10ae2f8513b7014623a0e45
test "$(git rev-parse HEAD)" = "$expected_candidate"
git diff --exit-code "$expected_candidate" -- poc/crates/viewer-gfr
export PATH=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:"$PATH"
export CARGO_HOME="$PWD/.build/cargo"
export CARGO_TARGET_DIR="$PWD/.build/target"
export CARGO_BUILD_JOBS=2
evidence_dir="$PWD/.build/artifacts/verification/m2-de628c2-independent-v1"
test ! -e "$evidence_dir"
mkdir -p "$evidence_dir"
cp poc/verification/viewer-gfr/independent-harness/run-m2-de628c2-review.sh "$evidence_dir/command.sh"
git rev-parse HEAD > "$evidence_dir/candidate.txt"
git status --short > "$evidence_dir/status-before.txt"
rustc --version > "$evidence_dir/rustc.txt"
cargo --version > "$evidence_dir/cargo.txt"
sha256sum poc/docs/contracts/task-contract-viewer-gfr-v1.md \
  poc/verification/viewer-gfr/source-first-expectations-v1.md \
  poc/verification/viewer-gfr/independent-harness/src/bin/m2.rs \
  poc/verification/viewer-gfr/independent-harness/src/m2_fixture.rs \
  poc/verification/viewer-gfr/independent-harness/src/m2_world.rs \
  poc/crates/viewer-gfr/src/*.rs > "$evidence_dir/input-hashes.txt"
exec 9>/tmp/megaroation-poc-heavy.lock
flock 9
cargo generate-lockfile --manifest-path poc/verification/viewer-gfr/independent-harness/Cargo.toml --offline \
  > "$evidence_dir/lock.stdout" 2> "$evidence_dir/lock.stderr"
set +e
cargo run --manifest-path poc/verification/viewer-gfr/independent-harness/Cargo.toml --bin m2 --locked --offline \
  > "$evidence_dir/harness.jsonl" 2> "$evidence_dir/harness.stderr"
harness_status=$?
set -e
printf '%s\n' "$harness_status" > "$evidence_dir/exit-code.txt"
git status --short > "$evidence_dir/status-after.txt"
git diff --exit-code "$expected_candidate" -- poc/crates/viewer-gfr > "$evidence_dir/candidate-diff.txt"
sha256sum poc/verification/viewer-gfr/independent-harness/Cargo.lock > "$evidence_dir/lock-hash.txt"
printf 'evidence=%s\nexit=%s\n' "$evidence_dir" "$harness_status"
exit "$harness_status"
