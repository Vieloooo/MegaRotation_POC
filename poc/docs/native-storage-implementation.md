# Native storage bridge

This slice implements Commonware 2026.10.0 Storage and Blob through the native Tokio filesystem backend. Protocol code receives a controlled context and blob handles. Each physical operation owns its arguments and receives a checked I/O ID during the ordinary grant. The root driver holds queued, active and completed slots. Its async grant waits on one physical operation at a time with the root waker, then transfers the typed result under an I/O release permit. Only a subsequent ordinary poll can continue a protocol future.

A dropped result receiver leaves the operation in the driver. Its eventual release disposes the result before the next dispatch, including a successful unobserved open that still owns the native name. The detached `start_sync` completion handle observes a queued operation. The driver awaits the native flush even after the observer is dropped or aborted. Native errors retain their original type. Scan names sort by raw bytes before release. Blob identity includes the open generation, and current issuer identity comes from the current controlled session. A legitimate later session on the same driver can use a transferred blob handle.

A failed or canceled async grant keeps accepted I/O in root-owned slots. The grant's cancellation guard clears its marker and latches failure without dropping operations. Async cleanup finishes accepted work and discards its results without polling protocol consumers. `run` and `run_async` always run this cleanup after the controller returns or panics. `grant` and `shutdown` return errors if they encounter outstanding physical I/O, and the async owner drains it. `shutdown_async` can perform that drain directly. The controlled runtime preserves the 30-second external watchdog obligation for I/O that does not return.

The diagnostic predispatch hold and fail-before-release use an explicit I/O ID configured before the first grant. Tests can retain an active operation before dispatch, cancel controller ownership, then allow root cleanup to complete the operation. An unreleased hold requires the external watchdog. Wall pauses are diagnostic. I/O trace data records run, node, incarnation, cid, task, partition, raw name, generation, grant and tick, operation details, and result metadata. Data digests use SHA-256 over the bytes. Native error variants appear as stable names in the trace.

## Validation

Run from the isolated checkout root.

```sh
./poc/scripts/bootstrap.sh fmt --manifest-path prototypes/i1b-runtime/Cargo.toml -- --check
./poc/scripts/bootstrap.sh test --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline
I1B_STORAGE_TEST_EVIDENCE=/tmp/rotation-i1b-native-storage-builder-journal ./poc/scripts/bootstrap.sh test --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline --test native_journal
```

The full prototype run passed 57 tests, including 49 existing runtime checks and 8 new storage checks. The replay test wrote and reopened a real Commonware variable journal with five BLS identities, verified PoPs, and three signed vote artifacts among 27 records. It covered two sections, repeated partial-page writes and a page boundary. It compared both encoded artifacts and their reconstructed vote wire bytes on reopen. The fixture uses 256-byte physical pages, 244-byte payload pages, cache capacity 4, write buffer 488, replay buffer 256, and no compression or pruning.

| Contract case | Builder test evidence |
| --- | --- |
| S1 | Native open, scan, remove, read, buffered read, batched reads, write, resize and sync round trip. Options and operation order checked. |
| S2 | Complete fixed-tick traces match under 0 and 2 ms wall pauses. Pre-release failure prevents any protocol consumer poll after Ready. |
| S3 | Abandoned receiver and unobserved native task result release exclusive open ownership before another open. |
| S4 | Typed duplicate-open, version mismatch, invalid partition and missing partition. A read beyond EOF returns the native error. |
| S5 | Recreate uses a distinct open ID. Old handle remains readable across remove and later legitimate cid use. |
| S6 | Dropped and aborted completion observers leave native sync running, with ready/discard ordering before later I/O. |
| S7 | Distinct creation orders yield identical scan and subsequent operation traces. |
| S8 | Deterministic predispatch hold, canceled controller future, and Ready-before-release failure all drain from native Runner. Synchronous misuse also drains without a successful Ack. |
| S9 | Real BLS artifacts cross a page boundary and retain exact artifact and vote bytes across close and reopen. |
| S10 | Write/reopen traces and raw section files match pairwise across wall-pause variants at each seed. |

The optional diagnostic export writes fresh run directories. Builder evidence from the listed command used `/tmp/rotation-i1b-native-storage-builder-journal`. Each pair had 141 write-run events, 38 reopen-run events and 27 records. Logical trace SHA-256 values were `a7861e49f2e0a9a87abb435e17dea4bb5923651c9dfc2a56e2faac92c7acc848` for seed 7, `8bdf45fdc1a000936923458c2a3b445154b8859e1ada6cbd11aad7116f6f97a1` for seed 42 and `828199d302edb2733037f1b9e0f4aad32b275cec36d563de6f61e85d1e5a3cb7` for seed 99.

The fixture demonstrates the primitive bridge, including actual native SYNC writes. It does not establish a versioned durability witness for individual artifacts, an outbox, guarded BLS signing, native Simplex protocol execution, or the subprocess I1b gate.
