# M5 automatic RESUME

Run the seed-42 profile in the candidate checkout. The launcher builds with the pinned offline cache and holds the host build lock while driving its private Anvil instance.

```sh
/home/jy/miniconda3/bin/python3 -B poc/scenarios/m5-auto-resume/run.py \
    --candidate "$(git rev-parse HEAD)" --scenario stopped --output "$NEW_DIRECTORY"
```

Use a fresh directory for each run. The `--wall-pause-ms 650` option adds a wall pause before outbox preparation. Normalize only worker PID and wall diagnostics when comparing virtual traces.

- `stopped` stops Espresso finalized rows from 11.250 to 24.500 virtual seconds.
- `frozen` allows finalized empty rows at the old public time key during that interval. The proposal is marked non-live.
- `slow-rpc` uses the stopped profile with 1,000 µs clock transport per leg and the original clock-offset distribution.
- `late` holds the signed outbox before its first actual driver command until 43.500 seconds.
- `short` restarts ordering before native derivation fallback and observes compatible chain lag.
- `no-target` widens the planned inclusion interval so the integer constraint cannot be satisfied.
- `inputs-wait` withholds post-restart Espresso deliveries from two signer readers.
- `divergence` holds ordering after h2 enters G and checks the independently derived h2 mismatch. Its cutoff follows actual inclusion of the signed request and precedes activation.

The main trace records the fault and restored rows, old result retention, genuine GFR watermark waits, native LocalStop Acks, public HALT, signer deliveries and eligibility, outbox preparation, receipt timing, exact cut reset and activated duty. NativeProposal binds the restored leader’s created Notarize subject to its durable Ack. The Anvil port records the original EIP-712 signatures and raw signed transaction. Named derivation feeds in `raw/derivation/trace.jsonl` include independent A/B and S0/S1/S2 replay, real DA service probes and precise historical cuts.
