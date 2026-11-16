# Independent M3 acceptance evidence

The checks use the frozen M3 contract and the source-first expectations committed before implementation. They bind the clean candidate and exact inputs to the sealed manifests, decode the actual Anvil constructor and Force calldata, recover the committee signatures, and check the driver's virtual submit, manual inclusion, finalization and delivery actions. They compare the retained governance decisions to each signer's verified public positions, and trace the unresolved submission through reloading the original signed transaction.

The independent Rust projection reuses the M2 public PELG decoder, BLS verification and execution oracle. Its optional M3 input supplies historical header hashes from verified actual Anvil receipts. It recomputes policy selection, the Plan basis, the canonical CID, and the complete G append including padding. The Python oracle uses the existing raw native frame and worker journal checker. The primary M3 fixture expects a real h1 followed by planned R0 exits and a first new R1 h18. Focused protocol and Solidity vectors are recorded in the matrix alongside the main scenario.

Run the checker from the clean tested worktree. Use a fresh output directory for each report.

```sh
M3_JOB=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m3-auto-force
M3_RUST=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin
export PATH="$M3_RUST:$PATH"
export CARGO_HOME="$M3_JOB/cargo"
export CARGO_TARGET_DIR="$M3_JOB/target"
export CARGO_BUILD_JOBS=2
flock /tmp/megaroation-poc-heavy.lock cargo build --locked --offline \
  --manifest-path poc/verification/m2-healthy-force/public/Cargo.toml

/home/jy/miniconda3/bin/python3 -B poc/verification/m3-auto-force/check.py \
  --repository "$PWD" --candidate "$(git rev-parse HEAD)" \
  --mode outage --run "$M3_RUN" --repeat "$M3_REPEAT" \
  --public-checker "$CARGO_TARGET_DIR/debug/m2-force-independent" \
  --output "$M3_REPORT"
```

Run the command with `--mode healthy`, `--mode reader-lag`, `--mode ordering-stopped` or `--mode watermark-frozen` to check the corresponding sealed run. Omit `--repeat` unless a 650 ms repeat of the same recovery scenario exists. The M2 healthy Force regression uses its existing [offline checker](../m2-healthy-force/README.md) against the same M3 candidate. Both replay modes use the same independent Rust binary.

The checks apply to seed 42 and the declared mock governance clocks, simulated delivery, fixture committee and controlled service schedule. Receipts and history are checked against the actual local Anvil RPC trace. The verifier does not establish Ethereum light-client verification or a full governance-process restart. The submitted-unknown case exercises reloading the prepared transaction file and querying or retrying the original driver command.
