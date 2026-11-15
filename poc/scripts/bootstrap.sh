#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
export CARGO_HOME="$PWD/../.build/cargo"
export CARGO_TARGET_DIR="$PWD/../.build/target"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
export RUST_TEST_THREADS="${RUST_TEST_THREADS:-1}"
# Optionally select an installed Rust 1.95.0 toolchain without changing the host cache.
if [[ -n "${REGIONAL_BFT_RUST_BIN:-}" ]]; then
    export PATH="$REGIONAL_BFT_RUST_BIN:$PATH"
fi
if (( $# )); then exec cargo "$@"; fi
cargo fmt --all --check
cargo test --locked --offline
