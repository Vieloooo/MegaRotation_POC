# I1a controlled runtime

I1a uses one hub with two independent worker processes and a local tonic bidirectional protobuf stream per worker. Only the hub advances virtual time. Each worker handles one granted input at a time and synchronizes its step record and committed head before sending its complete Ack. The SignedIntent fixture records the signing guard and publication boundaries. The Commonware 2026.10.0 integration belongs to I1b.

From the isolated checkout root, build with locked dependencies and run a scenario with a new output directory under an existing parent.

```sh
./poc/scripts/bootstrap.sh
./poc/scripts/bootstrap.sh build --locked --offline
.build/target/debug/sim-hub --scenario healthy --seed 7 --output /tmp/rotation-i1a-healthy
.build/target/debug/sim-hub --scenario healthy --seed 7 --output /tmp/rotation-i1a-paused --wall-pause-ms 2 --reverse-start
cmp /tmp/rotation-i1a-healthy/virtual-trace.jsonl /tmp/rotation-i1a-paused/virtual-trace.jsonl
```

The eight scenarios are declared in `crates/sim-hub/src/scenario.rs`. The selected resolved schedule, link and clock profiles, seed and explicit fault plans are saved in `scenario.json`. Seeds are 7, 42 and 99. Diagnostic wall pauses of 0–2 ms affect hub and worker scheduling and leave config identity unchanged. The dedicated 30-second watchdog exercise uses `--wall-pause-ms 30050`.

The virtual JSONL includes event keys, canonical typed grants/Acks, clock service and response transport, lease checks, validated effects, timer suppression, network admission and delivery, and authorized replay. `frames.bin` contains controlled-stream frames with one direction byte (0 hub/1 worker), one node byte, a big-endian u64 body length, and canonical decoded protobuf bytes. Hello records include the repeated Register request. Wall diagnostics include listener endpoint, child PIDs and exits. Worker stderr records apply-and-sync time for steps. Each worker directory contains the journal, committed head, per-incarnation launch configuration, stdout/stderr and process history. The run manifest marks harness completion or infrastructure failure. Independent source/command provenance and checker reports belong with verification or simulation evidence.

## Acceptance-case handoff

The mapping lists candidate exercises and known gaps. Independent verification is pending.

| Case | Candidate exercise or gap |
| --- | --- |
| I1-01 | `healthy` launches and records two live workers, independent journals and two gRPC streams. |
| I1-02 | Compare baseline/reversed-start and wall-pause `healthy` JSONL for each seed. |
| I1-03 | `healthy` has outstanding sample, timer, work and network waits while the hub schedules other work. |
| I1-04 | `clock-faults` uses +1000/-1000 and 50/250, 1000/10000 asymmetric RPC delays, with same-node Manager and BFT reads. |
| I1-05 | `clock-faults` projects historic samples after transit and tests delayed readiness. |
| I1-06 | `clock-faults` includes stale sequence, wrong run/domain/incarnation/request, future and expired replies, and consumed request duplication. Compare journal sample state around rejected replies. |
| I1-07 | `clock-faults` delivers invalid sequence 100 before valid sequence 3 on the same outstanding request. |
| I1-08 | `lease-expiry` loses refresh and grants work completion at lease end −1, equality and +1. |
| I1-09 | `lease-expiry` recalibrates, rejects Enter for the previous terminal cid and enters a separate fixture. |
| I1-10 | `timers` covers offset conversion, renewal, cancellation/rearm, delayed due firing after restart, and terminal cancellation. |
| I1-11 | `close-work` tests stop equality, pending work completion and Close ordering. |
| I1-12 | `healthy` staggers Enter. `close-work` repeats Create/Enter/Close and keeps previous signed state. |
| I1-13 | `restart-before-ack` has five process checkpoints. `journal_recovery` tests frame corruption, committed truncation, uncommitted retry and cached Ack after sync. |
| I1-14 | `timers` and `restart-before-ack` recover before prior expiry. `restart-after-expiry` holds the worker offline past its lease and refuses to revive the cid. |
| I1-15 | `restart-before-ack` and `close-work` resend exact grants. Journal tests reject altered and unauthorized replay. Independent adversarial stream coverage for altered/unsolicited Ack, bad stream identity and wrong framing remains pending. |
| I1-16 | `network-faults` covers offline drop/retry, duplicate delivery, cross-channel order/jitter, overflow and the payload equality/+1 boundary. Explicit 1 MiB gRPC edge-vector injection remains pending. |
| I1-17 | Named process faults are in the restart scenarios. Run the dedicated watchdog option and an externally killed-worker control for full coverage. |
| I1-18 | `close-work` varies equal-tick completion and Close under the approved seeds. Confirm both orderings in the matrix. |
| I1-19 | Independent trace mutation checker belongs to verification. |
| I1-20 | `close-work` replays a cached pre-close intent through the actual stream, with no new StepCommit. |

The model uses process boundaries for deterministic crash injection and a checksummed local filesystem journal for worker recovery. Hub crash recovery, disk power-loss experiments, native consensus work and consensus/finality evidence remain outside this implementation.
