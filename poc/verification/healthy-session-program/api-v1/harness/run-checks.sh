#!/usr/bin/env bash
set -euo pipefail
candidate=f9364a2a8e59913f7f9373fab2774be4ed528e57
harness=poc/verification/healthy-session-program/api-v1/harness
artifact_dir=${1:?pass a new raw artifact directory}
test "$(git rev-parse HEAD)" = "$candidate"
git diff --exit-code "$candidate" -- poc/crates/healthy-session-api poc/docs/healthy-session-program/api-slice-v1.md
mkdir -p "$(dirname "$artifact_dir")"
mkdir "$artifact_dir"
git status --porcelain=v1 > "$artifact_dir/git-status-before.txt"
git rev-parse HEAD > "$artifact_dir/candidate.txt"
rustc --version > "$artifact_dir/rustc-version.txt"
cargo --version > "$artifact_dir/cargo-version.txt"
sha256sum "$harness/Cargo.toml" "$harness/tests/independent.rs" "$harness/run-checks.sh" > "$artifact_dir/harness-initial-sha256.txt"
cargo generate-lockfile --manifest-path "$harness/Cargo.toml" --offline 2>&1 | tee "$artifact_dir/generate-lockfile.log"
cargo fmt --manifest-path "$harness/Cargo.toml" --check 2>&1 | tee "$artifact_dir/fmt.log"
cargo test --manifest-path "$harness/Cargo.toml" --locked --offline --test independent -- --test-threads=1 2>&1 | tee "$artifact_dir/independent-tests.log"
sha256sum "$harness/Cargo.toml" "$harness/Cargo.lock" "$harness/tests/independent.rs" "$harness/run-checks.sh" > "$artifact_dir/harness-final-sha256.txt"
git status --porcelain=v1 > "$artifact_dir/git-status-after.txt"
git diff --exit-code "$candidate" -- poc/crates/healthy-session-api poc/docs/healthy-session-program/api-slice-v1.md
