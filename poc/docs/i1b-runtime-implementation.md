# I1b controlled runtime prototype

The prototype uses Commonware 2026.10.0 with local patch `i1b-controlled-spawn-v1` from upstream revision `92a2f68a0e131c8b69de41e84aec37da6280f212`. The feature-gated patch submits a complete native job to the root-owned driver. Its constructor and native Abortable, Handle completion, supervision and tracker epilogues run under the driver's poll or cleanup permits. The driver applies both panic observers and fences publication after permanent failure. Normal Commonware spawn remains byte-for-byte the upstream implementation.

`ControlledDriver::run` and `run_async` start the native Runner with panic catching enabled. The controller borrows the root-owned registry. Protocol work receives only `SimContext`. The driver uses stable task-ID readiness, deduplicates controlled wakes and polls each job through Tokio's unconstrained wrapper. A live wake outside a same-thread permit fails closed. Transport pumps and, in the next slice, physical storage completion release must enter the driver through a controlled input or operation. Bare Tokio sleep, deferred yield and direct physical I/O futures are outside the runtime profile. The pinned Simplex non-test actor paths use biased selection and context clocks. Native Tokio storage uses blocking operations and requires its completion bridge before engine integration.

`SimContext` directly implements Commonware Clock, Supervisor, Spawner and Metrics, governor Clock and ReasonablyRealtime, and rand_core CryptoRng through TryRng and TryCryptoRng. A delivered, identity-bound sample calibrates the fixed-rate oscillator for a half-open lease. Clock reads and all tasks in one grant share the same projected time. Sleep preserves its signed microsecond deadline and waits for a matching granted timer, including after a same-cid transfer to another task. Close, stop-time equality and lease expiry retire a cid and dispose its work. Graceful `Spawner::stop` instead uses the native Stopper and an optional controlled sleep. Terminal state is in memory for this slice. The ChaCha20 adapter can checkpoint and restore a role's seed, stream and word position.

## Checks

Run from the isolated checkout root.

```sh
./poc/scripts/bootstrap.sh fmt --manifest-path prototypes/i1b-runtime/Cargo.toml -- --check
./poc/scripts/bootstrap.sh test --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline
```

The tests cover constituent native paths for P1–P14. The checks include constructor deferral, abort before and after first poll, unobserved constructor/poll panic, abandoned output and retained output destruction, complete-job disposal panic, parent cancellation, a canceled transport-only pump, reentrant/failed driver steps, fixed-point disposal, absence of Step/Ack on failure, and normal and over-bound cleanup. The replay fixture compares canonical event bytes with and without a wall pause. Clock and RNG checks include ±1000 µs, historic projection, recovery after expiry with a distinct cid, local stop and lease equality, native timeout bias and Stopper lifetime, sub-microsecond rounding, timer transfer, known ChaCha20 literals, role separation, and checked restore/exhaustion.

No native-Storage/Blob completion or durability contract, p2p, BLS GuardedScheme, native Simplex engine, two-process native wire, native sign/publication, or independent I1b acceptance result is included in this prototype. The process watchdog and crash recovery are later integration obligations.

## Patch provenance

The upstream packaged archive SHA-256 is `d520c22dc2c94764ee836d12ea9ee4f68e5c0dfb78071f3c7ef5fdc5ddfd88d6`. Upstream per-file hashes are recorded in `../vendor/commonware-runtime.provenance.json`. Comparing the vendored files to those hashes found exactly two modified paths, the lead-owned `Cargo.toml` feature declaration and `src/tokio/runtime.rs`. Reverse applicability of the complete upstream-relative patch was checked with `git apply --check --reverse`.

| Artifact | SHA-256 |
| --- | --- |
| `../patches/commonware-runtime-controlled-spawn-v1.patch` | `fcded76efe22885dde5e6cde4596ee70eeddbe83305150967fdbd92355f1e8f4` |
| Patched `src/tokio/runtime.rs` | `319e8f9bdb4698e48a7a045a338298c2f37691fcf36d65819b783ec222386db7` |
| Patched runtime `Cargo.toml` | `98d5d6be8e2225a25c09df7d5b03505fa3216541acf28e715ea616f3a98b7244` |
| Prototype `Cargo.lock` | `dae3a3a7e9123ed17c705a0690b004c5af10f10ac9c06610203688592bbc33ca` |
