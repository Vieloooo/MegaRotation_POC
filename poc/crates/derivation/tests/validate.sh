#!/usr/bin/env bash
# Run with the host-wide heavy lock already held.
set -euo pipefail
: "${L1_FOUNDRY:?set pinned Foundry tool directory}"
: "${CARGO_HOME:?set task-local Cargo cache}"
: "${CARGO_TARGET_DIR:?set task-local Rust target}"
[[ "${CARGO_BUILD_JOBS:-}" == 2 ]]
[[ $# == 1 && "$1" == /* ]] || { echo 'usage: validate.sh NEW_ABSOLUTE_OUTPUT_DIR' >&2; exit 2; }
out="$1"
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../../.."
mkdir -- "$out"
exec > >(tee "$out/validation.log") 2>&1

git rev-parse HEAD > "$out/revision.txt"
git status --porcelain > "$out/worktree-status.txt"
{
    rustc --version
    cargo --version
    python3 --version
    python3 -c 'import importlib.metadata as m; print("coincurve", m.version("coincurve"))'
    "$L1_FOUNDRY/forge" --version
    "$L1_FOUNDRY/anvil" --version
    "$L1_FOUNDRY/solc" --version
    sha256sum "$L1_FOUNDRY/forge" "$L1_FOUNDRY/anvil" "$L1_FOUNDRY/cast" "$L1_FOUNDRY/solc"
} > "$out/toolchain.txt"
set -x
cargo fmt --check --manifest-path poc/crates/derivation/Cargo.toml
cargo test --locked --offline --manifest-path poc/crates/derivation/Cargo.toml
cargo build --locked --offline --manifest-path poc/crates/derivation/Cargo.toml
cargo build --locked --offline --manifest-path poc/crates/l1-adapter/Cargo.toml
"$L1_FOUNDRY/forge" build --root poc/contracts --use "$L1_FOUNDRY/solc" --offline --threads 2 \
    --out "$out/forge-out" --cache-path "$out/forge-cache"
python3 poc/crates/l1-adapter/tests/check_abi.py "$out/forge-out"
drv="$CARGO_TARGET_DIR/debug/l1-adapter"
derive="$CARGO_TARGET_DIR/debug/derivation-replay"
sha256sum "$drv" "$derive" > "$out/binary-sha256.txt"
for case in window ahead cap accepted_wait; do
    for pause in 300 650; do
        run="$out/$case-$pause"
        python3 poc/crates/derivation/tests/run_anvil.py --case "$case" --derivation "$derive" \
            --anvil "$L1_FOUNDRY/anvil" --driver "$drv" --contracts "$out/forge-out" --out "$run" --wall-pause-ms "$pause"
        "$derive" replay "$run" A "$run/d-a.json"
        "$derive" replay "$run" B "$run/d-b.json"
        python3 poc/crates/derivation/tests/check_real_cases.py "$case" "$run" "$derive"
        if [[ "$case" == window ]]; then
            "$derive" receipt-gap "$run" "$run/receipt-gap-replay.json"
        fi
    done
    cmp "$out/$case-300/virtual-trace.json" "$out/$case-650/virtual-trace.json"
    cmp "$out/$case-300/d-a.json" "$out/$case-650/d-a.json"
    cmp "$out/$case-300/d-b.json" "$out/$case-650/d-b.json"
    if [[ "$case" == window ]]; then
        cmp "$out/$case-300/receipt-gap-replay.json" "$out/$case-650/receipt-gap-replay.json"
    fi
done
find "$out" -mindepth 2 -maxdepth 2 -type f -name '*.json' -print0 | sort -z | xargs -0 sha256sum > "$out/evidence-sha256.txt"
