#!/usr/bin/env bash
# Invoke this script under the global heavy-run lock.
set -euo pipefail

: "${L1_FOUNDRY:?set L1_FOUNDRY to the pinned Foundry+solc directory}"
: "${CARGO_HOME:?use the feature-local Cargo cache}"
: "${CARGO_TARGET_DIR:?use the feature-local Rust target}"
: "${CARGO_BUILD_JOBS:?set CARGO_BUILD_JOBS=2}"
[[ "$CARGO_BUILD_JOBS" == 2 ]]
[[ "$#" == 1 && "$1" == /* ]] || { echo 'usage: validate.sh NEW_ABSOLUTE_OUTPUT_DIR' >&2; exit 2; }
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
"$L1_FOUNDRY/forge" build --root poc/contracts --use "$L1_FOUNDRY/solc" --offline --threads 2 \
    --out "$out/forge-out" --cache-path "$out/forge-cache"
python3 poc/crates/l1-adapter/tests/check_abi.py "$out/forge-out"
cargo build --offline --locked --manifest-path poc/crates/l1-adapter/Cargo.toml
sha256sum "$CARGO_TARGET_DIR/debug/l1-adapter" > "$out/driver-sha256.txt"
for pause in 300 650; do
    python3 poc/crates/l1-adapter/tests/run_anvil.py --anvil "$L1_FOUNDRY/anvil" \
        --driver "$CARGO_TARGET_DIR/debug/l1-adapter" --contracts "$out/forge-out" \
        --out "$out/pause-$pause" --wall-pause-ms "$pause"
done
cmp "$out/pause-300/virtual-trace.json" "$out/pause-650/virtual-trace.json"
L1_ADAPTER_RPC_BUNDLE="$out/pause-300/raw-rpc-bundle.json" \
    cargo test --offline --locked --manifest-path poc/crates/l1-adapter/Cargo.toml \
    --test receipt_faults -- --ignored
sha256sum "$out"/pause-*/{driver-trace,raw-rpc-bundle,virtual-trace,result}.json > "$out/evidence-sha256.txt"
