#!/usr/bin/env bash
# Invoke this script under the global heavy-run lock.
set -euo pipefail
: "${L1_FOUNDRY:?set L1_FOUNDRY to the pinned Foundry+solc directory}"
: "${CARGO_HOME:?use feature-local Cargo cache}"
: "${CARGO_TARGET_DIR:?use feature-local Rust target}"
[[ "${CARGO_BUILD_JOBS:-}" == 2 ]]
[[ "$#" == 1 && "$1" == /* ]]
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
    "$L1_FOUNDRY/anvil" --version
    "$L1_FOUNDRY/forge" --version
    "$L1_FOUNDRY/solc" --version
    sha256sum "$L1_FOUNDRY"/{anvil,forge,solc}
} > "$out/toolchain.txt"
set -x
"$L1_FOUNDRY/forge" build --root poc/contracts --use "$L1_FOUNDRY/solc" --offline --threads 2 \
    --out "$out/forge-out" --cache-path "$out/forge-cache"
cargo test --locked --offline --manifest-path poc/crates/batch-services/Cargo.toml
cargo build --locked --offline --manifest-path poc/crates/batch-services/Cargo.toml --example run_component
sha256sum "$CARGO_TARGET_DIR/debug/examples/run_component" > "$out/component-binary-sha256.txt"
rg --files --hidden poc/crates/batch-services -g '!target/**' | LC_ALL=C sort | xargs sha256sum > "$out/component-source-sha256.txt"
for pause in 300 650; do
    python3 poc/crates/batch-services/tests/anvil_component.py --anvil "$L1_FOUNDRY/anvil" \
        --driver "$CARGO_TARGET_DIR/debug/examples/run_component" \
        --artifact "$out/forge-out/ControlAuthority.sol/ControlAuthority.json" \
        --out "$out/pause-$pause" --wall-pause-ms "$pause"
done
for name in protocol-trace driver-trace raw-rpc-bundle accepted-records execution-source; do
    cmp "$out/pause-300/run/$name.json" "$out/pause-650/run/$name.json"
done
python3 - "$out" <<'PY'
import json
from pathlib import Path
import sys

root = Path(sys.argv[1])
def stable_crashes(pause):
    data = json.loads((root / f"pause-{pause}/run/crash-trace.json").read_text())
    assert len(data) == 11
    assert sum(case.get("signal") == "SIGKILL" for case in data) == 1
    assert all(case["pid"] != case["reopen_pid"] for case in data)
    return [{key: value for key, value in case.items() if key not in {"pid", "reopen_pid"}}
            for case in data]
assert stable_crashes(300) == stable_crashes(650)
PY

find "$out" -path '*/run/*' -type f -print0 | sort -z | xargs -0 sha256sum > "$out/evidence-sha256.txt"
