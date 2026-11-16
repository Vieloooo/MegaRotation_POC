#!/usr/bin/env bash
set -euo pipefail

tester_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
worktree=$(cd -- "$tester_dir/../../../../.." && pwd)
cd -- "$worktree"
candidate=d289066082e0b89964e16653360a5c72356142f8
root=poc/verification/publication-gfr-healthy
slice_a=$root/slice-a/independent-harness
slice_b=$root/slice-b/independent-harness
output=$worktree/.build/artifacts/simulation/h3-b-d289066-acceptance-v1
lock=/tmp/megaroation-poc-heavy.lock

export PATH="/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:$PATH"
export CARGO_HOME="$worktree/.build/cargo"
export CARGO_TARGET_DIR="$worktree/.build/target"
export CARGO_BUILD_JOBS=2

capture_revision() {
  [[ $(git rev-parse HEAD) == "$candidate" ]]
  git rev-parse HEAD
  git branch --show-current
  git status --porcelain=v1 --untracked-files=all
}

source_hashes() {
  {
    git ls-files -z poc/crates poc/proto "$root/slice-a/independent-harness" "$root/slice-b" \
      poc/docs/contracts/h3-publication-gfr-{implementation,slice-a,slice-b}-v1.md
    find "$slice_b" -type f -print0
  } | sort -zu | xargs -0 sha256sum
}

if [[ ${1-} == init ]]; then
  mkdir -- "$output"
  capture_revision > "$output/revision.before"
  source_hashes > "$output/sources.before"
  cp -- "$slice_a/Cargo.lock" "$output/slice-a.lock.original"
  rustc -Vv > "$output/rustc-version.txt"
  cargo -V > "$output/cargo-version.txt"
  python3 -V > "$output/python-version.txt"
  flock --version > "$output/flock-version.txt"
  printf 'PATH=%s\nCARGO_HOME=%s\nCARGO_TARGET_DIR=%s\nCARGO_BUILD_JOBS=%s\n' \
    "$PATH" "$CARGO_HOME" "$CARGO_TARGET_DIR" "$CARGO_BUILD_JOBS" > "$output/environment.txt"
  exit 0
fi

if [[ ${1-} == finish ]]; then
  capture_revision > "$output/revision.after"
  source_hashes > "$output/sources.after"
  exit 0
fi

label=${1:?use init, a-original, a-lock, b, a-regression, vectors, or finish}
case "$label" in
  a-original | a-lock | a-regression) manifest=$slice_a/Cargo.toml ;;
  b) manifest=$slice_b/Cargo.toml ;;
  vectors) manifest= ;;
  *) exit 64 ;;
esac

[[ ! -e "$output/$label.command" ]]
capture_revision > "$output/$label.revision-before"
if [[ "$label" == vectors ]]; then
  command=(python3 -B "$tester_dir/check-vectors.py")
elif [[ "$label" == a-lock ]]; then
  command=(flock -x -w 60 -E 75 "$lock" timeout --kill-after=5s 180s cargo update --workspace --offline \
    --manifest-path "$manifest")
else
  sha256sum "$(dirname -- "$manifest")/Cargo.lock" > "$output/$label.lock.sha256"
  command=(flock -x -w 60 -E 75 "$lock" timeout --kill-after=5s 180s cargo test --locked --offline \
    --manifest-path "$manifest" -- --test-threads=1)
fi
printf '%q ' "${command[@]}" > "$output/$label.command"
printf '\n' >> "$output/$label.command"

start=$EPOCHREALTIME
status=0
"${command[@]}" > "$output/$label.stdout" 2> "$output/$label.stderr" || status=$?
printf '%s\n%s\n' "$start" "$EPOCHREALTIME" > "$output/$label.local-wall"
printf '%s\n' "$status" > "$output/$label.exit"
capture_revision > "$output/$label.revision-after"
if [[ "$label" == a-lock && "$status" == 0 ]]; then
  cp -- "$slice_a/Cargo.lock" "$output/slice-a.lock.resolved"
  sha256sum "$slice_a/Cargo.lock" > "$output/$label.lock.sha256"
  diff -u "$output/slice-a.lock.original" "$output/slice-a.lock.resolved" > "$output/slice-a.lock.diff" || [[ $? == 1 ]]
elif [[ -n "$manifest" && "$status" == 0 ]]; then
  while IFS= read -r binary; do
    sha256sum -- "$binary"
  done < <(sed -n 's/.*Running tests\/.* (\(.*\))$/\1/p' "$output/$label.stderr") > "$output/$label.binary.sha256"
fi
printf '%s exit %s\n' "$label" "$status"
exit "$status"
