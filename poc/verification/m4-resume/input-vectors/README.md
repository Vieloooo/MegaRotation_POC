# M4 input vectors

These vectors exercise private Anvil through one production 09 Driver per chain. The test port reads complete blocks through 11’s named InboxGateway, hands sealed accepted payloads into 12’s GatewaySource, and compares actual D outputs with the independent 03 execution reference. The fixed public profile uses one L2 second per height and two L1 seconds per block. The sequence window has four L1 blocks. The source-first predictions are in [expectations.md](expectations.md).

The same-block case sends two actual RESUME transactions with an accepted ordinary transaction batch between them. The first cut is at `(3,0,1)` with Set `(3,0,0)`. The earlier cut is already available when D encounters the unresolved accepted input at `(3,1,0)`. D stays at that input when the original object is withheld or different bytes are offered. The exact object permits adoption of the h2 ordinary transfer and the second cut at `(3,2,1)` with Set `(3,2,0)`. Later D growth and a checkpoint restore preserve both exact cuts. A and B read the same finalized receipts in different delivery orders. Trying to attach A’s capability as B is rejected.

The native cases cross the exact window end using fresh receipts at L1 blocks 3, 4 and 5. A timely record with an origin timestamp in the future becomes a same-height deposit-only substitute, and its following record is rechecked against the new parent. The future queue case uses a late h3 record and future h4 record in one frame. Completion of the actual window permits fallback h3, then the queued h4 executes. Matching the commitment of a malformed frame establishes its exact bytes and the frame is rejected at the accepted-input boundary. The 09 rollback case independently rechecks real receipt status, same-request retry, three block-specific state reads and exact cursor transition for the test-only outer revert after a real inner Override.

The original 11 archive check pins its delivery seal, verifies both finalized cursor advances, BS-FRAME-v1 bytes, signatures, original Driver schedule and fixture Kept execution. Its actual Ethereum L1 genesis and its G fixture L1 origin use different timestamp coordinates. The 12 typed source rejects that pair of identities under the original signed configuration. The live focused vectors use matching L1 and D coordinates. M4’s separate native 15-worker live run establishes G-backed production and recovery.

## Invocation

Run from the exact candidate worktree. Use the same offline Rust 1.95.0 tools and M4 data cache as the main scenario. The serial matrix calls this runner under the host heavy lock. For standalone verification acquire the same lock around the commands once. Build the current candidate’s `l1-adapter` and Foundry artifacts before launching.

```bash
cargo build --locked --offline --manifest-path poc/verification/m4-resume/input-vectors/Cargo.toml
/home/jy/miniconda3/bin/python3 -B poc/verification/m4-resume/input-vectors/run.py \
  --candidate "$(git rev-parse HEAD)" \
  --anvil "$L1_FOUNDRY/anvil" \
  --driver "$CARGO_TARGET_DIR/debug/l1-adapter" \
  --vector-port "$CARGO_TARGET_DIR/debug/m4-input-vectors" \
  --contracts "$M4_FORGE_OUT" \
  --archive /mnt/reth/jy/megaroation_poc/.build/commander-jobs/11/artifacts/m2-simulator-e5f77d2-run1/pause-300/run \
  --out "$M4_NEW_INPUT_EVIDENCE"
```

The manifest contains the tested candidate, exact child commands, file SHA-256s and per-case conclusions. Raw evidence retains real transactions, receipts, L1 read positions and delivered prefixes, exact DA objects, typed D waits, independently checked native rows and the two historical cuts. Slots are explicit and each fixture uses no random seed. The legacy rollback harness uses its existing 250 ms wall pause. Driver/frontier times are virtual.
