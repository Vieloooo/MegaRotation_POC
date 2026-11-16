# M2 Healthy Force independent check

The checker consumes sealed evidence from [M2-HEALTHY-FORCE-1](../../docs/milestones/M2-healthy-force-task-contract-zh.md) and uses the [source-first expectations](source-expectations-v1.md). It checks the seed-42 live profile, the optional pause repeat, and the M1 healthy full-window regression. Focused rule vectors run separately against production entry points.

The Rust projection loads only the canonical public domain and PELG codecs, the static registry and CID validation, and the pinned Commonware signature implementation from the candidate checkout. The acceptance logic reconstructs ancestry, first complete public positions, global execution and padding, and Force/regular choices from raw bytes and independently verified EVM input. It does not invoke the candidate's GFR transitions, Manager, Publisher, Merge, or FinalVerifier. The Python checker binds the receipt projection to the actual driver commands and raw RPC, checks raw Grant/Ack and durable worker/Manager records, and checks 15 process identities and worker signing. It reuses the versioned Anvil receipt checker and a subset of the previously sealed M1 offline checker.

From the exact candidate checkout, set the existing M2 toolchain/Cargo paths, use the shared lock for the independent binary build, and choose a fresh report directory.

```sh
cd /home/jy/.codex/worktrees/poc-m2-healthy-force/megaroation_poc
M2_DATADIR=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m2-healthy-force
M2_RUSTBIN=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin
export PATH="$M2_RUSTBIN:$PATH"
export CARGO_HOME="$M2_DATADIR/cargo"
export CARGO_TARGET_DIR="$M2_DATADIR/target"
export CARGO_BUILD_JOBS=2
flock /tmp/megaroation-poc-heavy.lock cargo build --locked --offline \
  --manifest-path poc/verification/m2-healthy-force/public/Cargo.toml

/home/jy/miniconda3/bin/python3 -B poc/verification/m2-healthy-force/check.py \
  --repository "$PWD" --candidate "$(git rev-parse HEAD)" \
  --raw "$M2_RUN/raw" --receipts "$M2_RUN/raw/l1/raw-rpc-bundle.json" \
  --driver-rpc "$M2_RUN/raw/l1/driver-trace.json" \
  --hub-l1 "$M2_RUN/raw/l1/hub-l1.jsonl" \
  --public-checker "$CARGO_TARGET_DIR/debug/m2-force-independent" \
  --output "$M2_NEW_REPORT"
```

For the pause comparison, pass `--repeat-raw "$M2_REPEAT_RUN/raw"`. For the M1 full-window regression, pass `--mode regular --raw "$M1_RUN/raw"` and omit `--receipts` and `--driver-rpc`. The input manifests must bind clean candidate revisions and every raw file. The report records the manifest, trace and public replay SHA-256 values. The pause comparison strips process wall diagnostics from WorkerSpawn and WorkerExit, then checks equality of the normalized trace, raw native frames and raw RPC bundle.

This bounded check trusts the local Anvil RPC as the execution source, independently verifies receipt/event consistency and the actual EIP-712 signer, and checks finalization against the explicit virtual driver and reader actions. The live report covers the specified process and slot schedule. It does not imply real-network timing, Ethereum proof verification, automatic failure detection, or recovery across process restart.
