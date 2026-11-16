# M3 Authority vectors

This fixture exercises AF07 and the on-chain part of AF08 against the named candidate. It starts two private Anvil chains with manual two-second L1 slots, deploys the actual committee contract with thresholds 2 and 1, and sends the transactions in [the source-derived matrix](expected.md) through the existing Rust L1 driver. The public vectors use fixed local identities, a ten-second announcement, and no random seed.

The driver evidence records EVM execution and reader delivery separately. Independent ABI decoding and EIP-712 recovery check successful raw receipts and their finalized viewer-facing positions. Rejected transactions must have status 0 and no logs. The fixture reads the sequence after each phase and calls accepted requests at later historical heights to check their original return values.

From the candidate checkout, use the L1 driver and Forge artifacts built from that revision. Set `M3_FORCE_BUILD` to the job with the candidate driver under `target/debug/l1-adapter`. Set `M3_FORCE_CONTRACTS` to the candidate Forge artifact directory. Choose a fresh path for `M3_FORCE_L1_EVIDENCE`.

```sh
flock /tmp/megaroation-poc-heavy.lock /home/jy/miniconda3/bin/python3 -B \
  poc/verification/m3-auto-force/l1-vectors/run.py \
  --candidate "$(git rev-parse HEAD)" \
  --anvil /mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0/anvil \
  --driver "$M3_FORCE_BUILD/target/debug/l1-adapter" \
  --contracts "$M3_FORCE_CONTRACTS" --out "$M3_FORCE_L1_EVIDENCE"
```

`manifest.json` records the exact command, clean Git revision, provenance hashes, and each profile's checked Force positions. Each threshold directory contains full RPC blocks and transaction receipts, the driver command sequence, the separate diagnostic RPC trace, and decoded case results. The M3 automatic Force scenario supplies the AF08 governance journal and ambiguous broadcast recovery evidence.
