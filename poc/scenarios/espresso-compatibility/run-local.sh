#!/usr/bin/env bash
set -euo pipefail

scenario=$(cd "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo=$(cd "$scenario/../../.." && pwd)
cd "$repo"

if [[ $# != 2 || ! $1 =~ ^[A-Za-z0-9_-]{1,64}$ || ! $2 =~ ^[a-f0-9]{40}$ ]]; then
  echo "usage: $0 RUN_ID EXPECTED_CANDIDATE_SHA" >&2
  exit 2
fi
run_id=$1
candidate=$2
owned=(poc/crates/espresso-compatibility poc/scenarios/espresso-compatibility)
if [[ $(git rev-parse HEAD) != "$candidate" ]] ||
  [[ -n $(git status --porcelain --untracked-files=all -- "${owned[@]}") ]]; then
  echo "espresso-compatibility: candidate SHA or owned paths differ from the reviewed revision" >&2
  exit 2
fi

data=$(realpath -- "$repo/.build")
image="$data/setup/official-20260918"
run="$data/artifacts/$run_id"
target="$data/target"
mkdir -p "$data/artifacts"
mkdir -- "$run"

rust_bin=${ESPRESSO16_RUST_BIN:-/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin}
if [[ -d $rust_bin ]]; then
  export PATH="$rust_bin:$PATH"
fi
if [[ $(rustc --version) != "rustc 1.95.0 "* ]]; then
  echo "espresso-compatibility: Rust 1.95.0 required" >&2
  exit 2
fi
export CARGO_HOME=${CARGO_HOME:-/mnt/reth/jy/megaroation_poc/.build/cargo}
export CARGO_TARGET_DIR="$target"
export CARGO_BUILD_JOBS=2

exec 9>>/tmp/megaroation-poc-heavy.lock
flock -w 600 9

build=(cargo build --manifest-path poc/crates/espresso-compatibility/Cargo.toml --locked --offline)
printf '%s\n' "${build[*]}" > "$run/build-command.txt"
"${build[@]}" > "$run/build.stdout" 2> "$run/build.stderr"
python3 "$scenario/prepare-image.py" verify "$image" > "$run/image-verification.json"
command=(
  timeout --signal=TERM --kill-after=15s 250s
  bwrap --unshare-user --unshare-net --unshare-pid --die-with-parent --new-session
  --ro-bind / / --bind "$run" "$run" --tmpfs /tmp --proc /proc
  /usr/bin/python3 "$scenario/capture.py"
  --image "$image" --run "$run" --run-id "$run_id" --candidate "$candidate"
  --replay-binary "$target/debug/espresso-compatibility"
)
python3 -c 'import json,sys; print(json.dumps(sys.argv[1:], indent=2))' "${command[@]}" > "$run/launch-command.json"
"${command[@]}" > "$run/capture.stdout" 2> "$run/capture.stderr"
cat "$run/capture.stdout"
