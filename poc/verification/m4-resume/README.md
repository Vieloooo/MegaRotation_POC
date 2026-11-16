# M4 independent RESUME evidence replay

`check.py` independently decodes the deployed governance committee and exact EIP-712 control and batch receipts. It binds mining, finalized delivery and receipt reads to the recorded virtual driver. The 11 frame oracle opens and verifies the accepted DA objects, and the separate D reference executes on its own predecessor state. Each historic cut is matched to the corresponding ResumeRecorded and paired CursorSet positions. The checker feeds independently verified target state and prefix hashes to the optional M4 mode of the existing native public replay oracle. The oracle checks real BLS, body linkage, ordinary execution, shared G/D prefix, G suffix replacement, historical Plan, and first h21 acceptance. The worker checker verifies durable Grant/Ack and terminal output boundaries. The checker separately compares the 650 ms repeat artifacts.

Build with the tested candidate checkout and pinned Rust 1.95 under the task lock. Use its existing isolated Cargo home and target.

```sh
flock /tmp/megaroation-poc-heavy.lock cargo build --locked --offline \
  --manifest-path poc/verification/m2-healthy-force/public/Cargo.toml

/home/jy/miniconda3/bin/python3 -B poc/verification/m4-resume/check.py \
  --repository "$PWD" --candidate "$M4_CANDIDATE" \
  --run "$M4_MATRIX/primary" --repeat "$M4_MATRIX/primary-pause650" \
  --public-checker "$CARGO_TARGET_DIR/debug/m2-force-independent" \
  --output "$M4_NEW_REPORT_DIR"
```

The CLI verifies clean checkout identity, committed contracts and both handed-off component ancestors. Raw evidence uses the sealed M4 profile, real RPC, DA index/files and compact cut bytes. Focused direct-entry vectors cover negative guards and multiple positions separately from this single positive live scenario. Chain authenticity is scoped to the local captured Anvil RPC and configured verified reader boundary.
