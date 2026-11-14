# I1b runtime slice v1 — lead implementation freeze

Authority: task-contract-i1b-v1.md, including P1–P14 and applicable B1–B4/B13/B14. This slice implements the actual native task, clock, RNG and failure boundary first. Storage/Blob, p2p, GuardedScheme, engines and two-process I1b wire integration follow. This slice alone cannot pass I1b.

## Ownership and baseline

Lead owns manifests/lockfiles, shared API decisions and integration. One builder owns `poc/prototypes/i1b-runtime/src/**`, its `tests/**`, and `poc/vendor/commonware-runtime/src/tokio/runtime.rs` ONLY. The copied runtime baseline is Commonware 2026.10.0/upstream92a2f68a0e131c8b69de41e84aec37da6280f212; original package/file checksums are in the sibling provenance JSON. Lead added `controlled-spawn` to its manifest. tokio/mod.rs already wildcard-reexports the new public ControlledJob alias. No other vendor file change is needed. Do not edit parent/cache or reformat unrelated vendor files.

Implement the exact spawn_controlled API and both permanent, nonpanicking panic observers from the full contract. Keep ordinary Spawner::spawn unchanged. Export a reviewable patch with actual hash and modified-file allowlist. Never describe the patch as unmodified upstream.

## Driver/context requirements

Root-owned ControlledDriver exclusively owns complete native jobs. SimContext contains the non-Clone native Context by value and delegates child/attribute/name/metrics correctly. Use Weak enrollment/proxy wakers to avoid registry ownership cycles. Registration, constructor, poll, cancellation and disposal are auditable stable-ID events. Never hold registry locks while polling/dropping; newly enrolled jobs are ready even if Handles are ignored. Native metrics do not establish quiescence.

Every grant fixes integer reference time. Clock::current and governor::now share the calibrated NodeClock; no OS time or independent advancing Runner. Sleep registers an absolute local deadline and only a granted timer input wakes it. Grant does not renew lease; recovery never resets deadlines. Preserve fixed offsets within ±1000us and the half-open lease. ClockUnsafe blocks ordinary execution/new signing and permanently retires affected cid; recovery may permit a distinct legitimate cid only. Cleanup has restricted permits.

Poll the complete job through tokio::task::unconstrained so host cooperative budget cannot alter ordering. Use stable task-ID readiness, duplicate-wake deduplication and checked IDs. Default bounds: 10,000 logical polls per grant and 30 s external wall watchdog. Failure never advances protocol time. A nonreturning poll requires an external process watchdog later; Tokio timeout cannot preempt it.

Use native catch_panics=true only with BOTH observers. Root keeps ownership after task/pump/syscall/driver failures, revokes ordinary signing/publication and drains jobs before return. No successful Ack after failure. Protocol-bearing Handle<T>/T remains inside controlled jobs; infra may own inert Handle<()> only, disposed during controlled shutdown. Cover missing/retained receivers, DropBombs, never-polled captures and descendant cleanup. Raw RPC/IO pumps never own protocol jobs.

Use public signal::Stopper, retaining first stop value and Signal lifetime. Override stop timeout with controlled time, never native wall-clock stop. Later Engine::new and start must run inside an enclosing session job so actor siblings are supervised together.

## RNG and validation

Pin rand_chacha0.10.0, ChaCha20Rng. Stable role seed derives from run seed, node, cid and role path. Checkpoint seed/derivation reference, stream u64 and word position u128 (wire hi/lo u64, hi<=15); reject exhaustion/wrap. Restore seed, then stream, then position. Check mixed u32/u64/partial fills, role separation and restored continuation. Never use system RNG, PID or task address for protocol randomness.

Compile the actual Clock/governor, Supervisor/Spawner, CryptoRng and Metrics bounds. Complete P1–P14 with independently meaningful sentinels, fixed trace comparisons under wall pauses, clock/stop deadlines and RNG vectors. Prove test cleanup returns from Runner instead of leaking tasks behind asserted flags. Full Storage/Engine/p2p/signing integration stays explicitly pending.

One build at a time, with local cache/target:

    ./poc/scripts/bootstrap.sh test --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline

Request lead review for API/dependency changes. Handoff includes source/diff hashes, exact checks/results, traces and open findings. Independent verifier derives expectations before implementation inspection. No native scenario or I1b PASS until full integration is independently checked.
