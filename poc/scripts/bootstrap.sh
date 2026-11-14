#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
export CARGO_HOME="$PWD/../.build/cargo"
export CARGO_TARGET_DIR="$PWD/../.build/target"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
export RUST_TEST_THREADS="${RUST_TEST_THREADS:-1}"
# Read the installed compiler directly. No rustup writes to the parent cache.
export PATH="/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:$PATH"
if (( $# )); then exec cargo "$@"; fi
cargo fmt --all --check
cargo test --locked --offline
