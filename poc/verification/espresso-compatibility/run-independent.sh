#!/usr/bin/env bash
set -euo pipefail
repo=$(cd "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
cd "$repo"
if [[ $# != 2 || ! $2 =~ ^[a-f0-9]{40}$ ]]; then
  echo 'usage: run-independent.sh NEW_OUTPUT EXPECTED_CANDIDATE_SHA' >&2
  exit 2
fi
out=$1
candidate=$2
[[ $(git rev-parse HEAD) == "$candidate" ]]
[[ -z $(git status --porcelain --untracked-files=all -- poc/crates/espresso-compatibility poc/scenarios/espresso-compatibility) ]]
mkdir -p -- "$(dirname -- "$out")"
mkdir -- "$out"
out=$(realpath -- "$out")
export PATH=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:$PATH
export CARGO_HOME=/mnt/reth/jy/megaroation_poc/.build/cargo
export CARGO_TARGET_DIR=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/16/target/independent-verifier
export CARGO_BUILD_JOBS=2
export EC16_VERIFY_OUTPUT="$out"
exec 9>>/tmp/megaroation-poc-heavy.lock
flock -w 600 9
git rev-parse HEAD > "$out/candidate.txt"
git status --short > "$out/worktree-status.txt"
rustc --version > "$out/rust-version.txt"
cargo --version > "$out/cargo-version.txt"
manifest=poc/verification/espresso-compatibility/independent-harness/Cargo.toml
if [[ ! -f poc/verification/espresso-compatibility/independent-harness/Cargo.lock ]]; then
  cargo generate-lockfile --manifest-path "$manifest" --offline > "$out/lock.stdout" 2> "$out/lock.stderr"
fi
command=(cargo test --manifest-path "$manifest" --locked --offline -- --test-threads=1 --nocapture)
python3 -c 'import json,sys; print(json.dumps(sys.argv[1:],indent=2))' "${command[@]}" > "$out/command.json"
"${command[@]}" > "$out/tests.stdout" 2> "$out/tests.stderr"
cat "$out/tests.stdout"
sha256sum poc/verification/espresso-compatibility/{expectations-v1.md,vectors-v1.json,run-independent.sh} \
  poc/verification/espresso-compatibility/independent-harness/{Cargo.toml,Cargo.lock,tests/independent.rs} > "$out/input-sha256.txt"
[[ $(git rev-parse HEAD) == "$candidate" ]]
[[ -z $(git status --porcelain --untracked-files=all -- poc/crates/espresso-compatibility poc/scenarios/espresso-compatibility) ]]
