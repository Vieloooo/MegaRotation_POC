# NATIVE-STORAGE v1 — frozen primitive bridge

Status: implementation contract, not a test result. Prerequisite runtime slice is independently accepted at implementation 3ab4549 and integrated revision 7930842. Authority: v2 §§4–5; task-contract-i1b-v1; Commonware2026.10.0/upstream92a2f68a0e131c8b69de41e84aec37da6280f212 Storage/Blob contract, paged writer and segmented variable journal. Full I1b remains pending.

## Scope and ownership

Implement actual Storage and Blob on SimContext/ControlledBlob, plus root-owned deterministic physical-operation completion. Use `<commonware_runtime::tokio::Context as Storage>::Blob` behind Arc; never expose the raw context/blob to protocol code. One builder owns prototype src/{storage.rs,driver.rs,context.rs,lib.rs} and builder tests/support, plus poc/docs/native-storage-implementation.md. Lead owns Cargo manifests/locks and public contract changes. Do not edit vendor runtime or independent verifier sources. Preserve P0 clock/task/RNG behavior and rerun its suites. Parent/cache remain read-only.

This slice DOES NOT implement native-artifact durability proofs, outbox, BLS signing gates, p2p or engines. Actual Journal+real signed Artifact fixtures exercise storage/codec behavior only. No native finality or live BFT claim follows from storing a signed fixture.

## Operations and custody

Implement all actual Storage/Blob requirements, including open_versioned, remove, scan, read_at_buf, read_at, write_at, resize, sync and start_sync; default read_many/start_read_many must remain valid. Copy borrowed names/options/ranges and own buffers before enqueue. A monotonic checked io_id is assigned exactly once when an operation is first issued under an ordinary granted poll/setup. Record run/worker, logical partition/raw name, open-instance generation and owner cid/task. Reopen has a distinct open ID; remove/recreate must not alias an old still-readable handle.

Root grant_async drains runnable jobs, then awaits the smallest issued pending io_id using the ROOT waker, releases its typed result under IO_RELEASE, disposes abandoned results, drains newly ready jobs, and repeats at the same fixed virtual tick. Native I/O completion and wall delays never advance protocol time. No registry lock across poll, await or drop. No protocol proxy waker is handed to native disk futures.

Queued, popped/in-flight operations AND ready-but-unreleased results remain in DRIVER-OWNED slots. A cancellable grant_async/controller future only borrows them. Its Drop marks abandoned control ownership without dropping jobs/resources. The surviving root distinguishes a still-active/reentrant grant from an abandoned async grant, clears abandoned ownership, and drains under cleanup. Repeated calls after failure must not restart cleanup while an outer permit remains active.

IO_RELEASE authorizes result transfer, proxy-wake enrollment and owned-result disposal; it does not authorize new protocol construction/signing/publication. Protocol futures run only in subsequent ordinary polls. Failure cleanup never resumes ordinary work. Poll/disposal panics and broken completion bookkeeping latch the infrastructure flag. All native Storage/Blob method errors, including write and sync failures, remain typed method results; handling one does not itself fail the run. A Step reporting a handled error does not establish a successful durability barrier or publication eligibility.

A dropped/aborted start_sync completion observer does not cancel accepted work: the queue owns native completion. All issued operations use a complete-even-if-observer-abandoned policy. Drop a successful abandoned open result before executing the next open so the wrapper does not invent BlobAlreadyOpen. Metadata registries must not retain unnecessary strong native Blob references.

Sort scan names lexicographically by raw bytes before release. Record canonical operation/result metadata and buffer digests, with no host absolute paths, pointer/PID values or wall durations in logical traces. Native error diagnostics may be retained separately. Preserve exact version/partition/range/open/removal semantics, options and returned bytes.

The synchronous grant/shutdown APIs must never acknowledge quiescence while I/O remains. Freeze their explicit handling in the implementation handoff (reject misuse/fail closed and let the root async owner finish cleanup, or a reviewed resumable API). Top-level run/run_async must keep custody until all accepted I/O/results/jobs are drained or explicitly reported incomplete; no implicit uncontrolled Drop of an outstanding operation.

## Concrete Journal fixture

Use real BLS multisignature types, no shared-body mock app:

- S = simplex::scheme::bls12381_multisig::Scheme<ed25519::PublicKey, MinSig>.
- D = commonware_cryptography::sha256::Digest (not the Sha256 hasher).
- J = commonware_storage::journal::segmented::variable::Journal<SimContext, Artifact<S,D>>.
- Five identities decoded from seeds [16+i;32], i=0..4. BLS keys generated sequentially using ChaCha20Rng seed [42;32] and public ops::keypair. Verify every PoP using namespace `I1b/storage/pop/v1`; native vote namespace `I1b/storage/vote/v1`. Build ordered BiMap and explicit member→signer-index mapping; never assume member0 has signer index0.
- Epoch1; view1 proposal parent-view0, digest [0x44;32]; optional view2 parent-view1, digest [0x55;32]. Native Notarize/Finalize/Nullify and Certification artifacts may be exercised; fixtures are offline test inputs, not protocol-time signatures.
- Partition `i1b-voter-0`, compression None, codec_config from verifier.certificate_codec_config().
- CacheRef::new(paged::page_size(256), NZUsize!(4)): physical pages256 bytes, logical payload244 bytes plus12 CRC bytes.
- write_buffer NZUsize!(488); replay(0,0,NZUsize!(256),ReadOptions::DONT_CACHE). Reopen with the same page size.
- Views/sections1–2, no prune. Each section name is native big-endian u64. Native engine floor is not a Journal Config field; actual Engine integration later uses Genesis/view0 and retained history.
- Finite tests keep blobs <=64KiB and individual encoded artifacts <=4096B. These are test bounds, not invented restrictions on the Storage trait.

No extra dependency or feature is needed for this fixture; use already pinned public key generation and codec APIs. Request lead review rather than adding a mock feature or changing quorum.

## Acceptance

S1: actual open/write/read/resize/sync/remove/scan and all required options through wrapper; issuance/release order exact.
S2: injected bounded physical wall pauses change only wall time; no protocol consumer observes completion before IO_RELEASE.
S3: abandon successful open; following reopen succeeds after controlled result disposal. Include pending/ready abandoned outputs.
S4: duplicate open, version mismatch, invalid partition and missing-range errors retain native variants; handling an expected error need not fail the run.
S5: remove/recreate independent instance; old removed handle remains readable until dropped.
S6: dropped AND aborted start_sync observer still completes actual flush, before later queued operations; completion Handle acceptance is not durability.
S7: different directory creation order yields identical sorted scan output and deterministic subsequent IO IDs.
S8: cancel controller/pump during pending physical operation; inject failure after Ready before release; root retains custody, disposes under cleanup and returns from actual Runner. No successful Step/Ack after failure. Nonreturning native IO remains an external-watchdog/infrastructure condition, never a protocol crash or completed cleanup claim.
S9: actual native variable Journal with real Artifact append/sync/reopen/replay; partial pages, page crossing, repeated partial-page rewrite and multiple sections. Retain exact written/replayed artifacts and native config.
S10: seeds7/42/99 with wall pauses0/2ms yield identical complete logical operation/result traces; exclude only explicit wall diagnostics. Preserve ordinary protocol-level clock/RNG/panic regression suites.

A successful write_at with WriteOptions::SYNC may implement Journal::sync without Blob::sync. Record that exact operation and range now; do not label unrelated dirty bytes durable. Full native witness/outbox acceptance is a following contract, not inferred from these primitive tests.

## Deferred NATIVE-WITNESS choices

The next contract must track byte versions and extent, exact SYNC-write ranges vs full-blob barriers, authoritative CRC page slots and complete varint Artifact frames. Artifact tags must be converted through typed Vote/Certificate encoding. Witness/outbox durability precedes successful barrier-result release for publication eligibility. Recovery preserves raw evidence and committed history, revalidates copied snapshots, and obtains a NEW sync before publishing recovered bytes; readability alone is never a historical barrier proof. Do not run a potentially repairing replay on live files merely to verify them. These obligations remain unimplemented by this slice.

Commands from isolated checkout use `./poc/scripts/bootstrap.sh ... --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline`. One build/heavy run at a time. Handoff candidate/diff, exact checks, source/fixture hashes, traces and remaining limits to independent verifier before using this adapter for engine experiments.

## Lead clarification 1 — native errors and evidence persistence

The lead explicitly distinguishes native method errors from failure to persist the adapter's own required evidence/outbox/Ack ledger. Native Storage/Blob errors stay typed in this primitive slice, even if a protocol caller handles a write or sync error and the controlled Step completes. No such Step implies that the failed mutation or barrier succeeded.

Required adapter-evidence persistence belongs to NATIVE-WITNESS and fails closed. That later slice must invalidate uncertain ranges after failed mutations/barriers, produce no successful witness for them, and prohibit publication without a valid completed barrier. Native Simplex may itself panic on a journal error. This clarification preserves native error semantics and does not weaken signing or publication guards.
