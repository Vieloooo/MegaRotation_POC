# I1b v1 — Native Commonware execution under controlled grants

Date: 2026-11-15. Status: **accepted task contract; I1b is not implemented or passed**.

This contract completes the I1b scope defined by [I1 v1](task-contract-i1-v1.md). It incorporates the inspected native cancellation, constructor and panic boundaries. I1a candidate 3d413c4 is a separate infrastructure-fixture candidate; its existence is not I1b verification evidence. No implementation, build or successful runtime result is asserted here.

## 1. Objective and gate

Run two actual worker subprocesses with the native Commonware Simplex engine, controlled task execution, Clock/governor, seeded RNG, durable Storage/Blob, p2p channels, application callbacks, relay and reporter. Every protocol constructor, future poll, cancellation and protocol-bearing disposal must be attributable to a controlled grant or explicit failure-cleanup permit.

Overall I1 requires both I1a and I1b. Passing a trait-shaped fixture or an in-process prototype does not pass I1b. I1b requires real subprocess integration, independent checks and replay on the integrated revision before proceeding to the 15-node stage. Full five-live-replica committee and regular-handoff claims require their own next-stage verification.

Retain five registered equal-weight participants and native N3f1 quorum four. The two actual engines may receive explicitly labeled scripted peer transcripts from the other three fixture identities to exercise certificate, finalize and resolver paths. This is finite test input, not evidence of a live four-node quorum. Two active engines without such input cannot claim quorum finality.

Infrastructure progress, native progress evidence, verified finality evidence and delivery of a contiguous validated block prefix remain distinct. No quorum, signing domain, validity rule or finality rule is relaxed by this contract.

## 2. Source and version authority

| Source | Applicable sections/boundary |
| --- | --- |
| formalization_v10/poc_design_zh_v2.md, 2026-11-14 | §§4.1–4.4: clock/sample/lease bridge; §§5.1–5.3: controlled execution and Commonware integration; §§6.1–6.2: lifecycle; §13.1: implementation gate. |
| formalization_v10/00_arch_zh_v0_1.md | §2.3: C2/C3/C5 clock assumptions. |
| formalization_v10/02_session_program_zh_v0_1.md, 2026-11-09 | §§3.1–3.8 and 4.9: persistent state, Create/Enter/Close, pending validation and signing-time shutdown. |
| /mnt/reth/jy/regional_BFT_sim/DESIGN.md, 2026-11-07 | §§2–2.1 and 3.1: concrete adapter boundaries, deterministic application validity versus local admission, native Simplex evidence and recovery. |
| task-contract-i1-v1.md | Existing I1 guards, twenty acceptance cases, bounds, seeds and evidence package remain binding. |
| i1-interface-annex-v1.md | Lead-owned common wire/encoding/dependency interface. Integration changes require a versioned lead-owned annex amendment. |

Pinned release: **Commonware 2026.10.0**. Upstream VCS revision: **92a2f68a0e131c8b69de41e84aec37da6280f212**. Source inspection root, read-only:

~~~text
/mnt/reth/jy/regional_BFT_sim/.tooling/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/
~~~

Upstream paths below are relative to that root; line numbers identify the inspected release, not future local-patch positions.

| Source anchor | Finding |
| --- | --- |
| commonware-consensus-2026.10.0/src/simplex/engine.rs:20, :180 | Exact engine context bounds and three native p2p channel pairs. |
| commonware-runtime-2026.10.0/src/lib.rs:276, :358, :470, :525 | Supervisor, Spawner, Metrics and Clock contracts. |
| commonware-runtime-2026.10.0/src/lib.rs:719, :922 | Storage/Blob, reopen durability and asynchronous sync semantics. |
| commonware-runtime-2026.10.0/src/tokio/runtime.rs:212, :415 | catch_panics configuration and Panicker construction. |
| commonware-runtime-2026.10.0/src/tokio/runtime.rs:485–505 | Runner root execution, root-tree abort and task-tracker drain. |
| commonware-runtime-2026.10.0/src/tokio/runtime.rs:545–615 | Eager factory, native Handle construction, outer panic/tracker wrapper and executor dispatch. |
| commonware-runtime-2026.10.0/src/utils/handle.rs:192, :210–236 | Crate-private task Handle constructor; Abortable; result send; inner panic consumption. |
| commonware-runtime-2026.10.0/src/utils/handle.rs:250–280, :357 | Public completion handles do not cancel spawned work. |
| commonware-runtime-2026.10.0/src/utils/handle.rs::Panicker::notify | With catch enabled, panic is logged without interrupting Runner root. |
| commonware-p2p-2026.10.0/src/lib.rs:58–211, :390 | Limited/checked senders, receiver and Blocker; accepted submission is not delivery. |
| commonware-cryptography-2026.10.0/src/certificate.rs:245–489 | Verifier/Scheme traits, synchronous sign and native assembly/verification. |
| commonware-consensus-2026.10.0/src/simplex/actors/voter/state.rs:554, :751, :800 | Native nullify/notarize/finalize signing; issuance assumes signing refusal means a verifier. |
| commonware-consensus-2026.10.0/src/simplex/actors/voter/actor.rs:313–375, :943–986, :1450–1455 | Native append, sync and publication ordering. |
| commonware-consensus-2026.10.0/src/lib.rs:144–200, :259–396 | Pending propose/verify/certify responses; separate relay/reporter boundaries. |
| commonware-macros-2026.10.0/src/lib.rs:47 | Biased select; preserve native source-order choice. |

The regional-bft-conformance skill frames the cases. protocol-spec-writer was unavailable during this source review. The parent adapter and cache remain read-only references; the existing parent benchmark is not an external controlled runtime.

## 3. Ownership, allowed paths and patch provenance

All implementation paths must be inside the isolated checkout. The lead assigns a single writer and exact paths before coding. Concurrent writers require an approved worktree; a failed managed-worktree operation does not authorize a bypass.

| Responsibility | Owner/scope |
| --- | --- |
| Contract, common interfaces, schema, dependencies, lockfile and integration | Lead. |
| Controlled runtime/engine implementation | One assigned builder in lead-approved poc/crates/ or a separately assigned poc/prototypes/i1b-runtime/ package. |
| Local runtime patch | One assigned owner in a lead-selected poc/vendor/ directory; never the parent registry cache. |
| Independent expectations/checker/tests | Verifier in assigned poc/verification/i1b/ paths. |
| Verified-revision replay | Simulator in fresh poc/artifacts/{verification,simulation}/i1b/ directories. |
| This documentation handoff | Only poc/docs/contracts/task-contract-i1b-v1.md. |

The minimal local runtime patch is restricted to its Cargo.toml feature declaration and src/tokio/runtime.rs, with focused tests. Add an opt-in feature such as controlled-spawn. Ordinary native Spawner::spawn behavior remains unchanged. No consensus/cryptography/quorum algorithm patch is authorized by this contract.

Use a lead-owned Cargo patch override. Keep package version 2026.10.0 and identify the runtime as **2026.10.0 plus local patch i1b-controlled-spawn-v1**. Record upstream crate/source hashes, exact patch bytes and SHA-256, resulting vendored-source hash and lockfile. Do not describe the patched runtime as unmodified upstream. The patch hash is pending until implementation; this contract does not invent one.

## 4. Actual engine and runtime boundaries

The engine requires:

~~~rust
E: Clock + rand_core::CryptoRng + Spawner + Storage + Metrics
S: simplex::scheme::Scheme<D>
L: simplex::elector::Config<S>
B: commonware_p2p::Blocker<PublicKey = S::PublicKey>
D: commonware_cryptography::Digest
A: CertifiableAutomaton<
    Context = simplex::types::Context<D, S::PublicKey>, Digest = D>
R: Relay<Digest = D, PublicKey = S::PublicKey,
         Plan = simplex::Plan<S::PublicKey>>
F: Reporter<Activity = simplex::types::Activity<S, D>>
T: commonware_parallel::Strategy
~~~

Engine::start takes separate vote, certificate and resolver (Sender, Receiver) pairs scoped to the epoch. Full-body relay uses another controlled channel. Use Sequential strategy. Runtime Network, DNS Resolver, Strategizer, Rayon and authenticated raw TCP are not required for this minimum, because the adapter supplies the p2p channels.

The worker uses a private native Tokio context for native Handle/supervision, Metrics and filesystem storage. SimContext supplies controlled clock, RNG, stop timeout, Storage/Blob completion visibility and p2p. The raw context is never exposed to consensus/application code. Engine::new and application construction also occur under a valid grant; deferring spawn constructors alone does not control earlier construction.

There is no worker-owned deterministic Runner and no self-advancing virtual clock. Native Tokio hosts infrastructure; the controlled driver owns and polls protocol jobs. Shared/dedicated placement requests are recorded hints and do not bypass the controlled scheduler in this simulation profile.

## 5. Minimal controlled-job API and exact repair

Public external completion-handle constructors cannot implement native task cancellation faithfully. Gating only the user future also leaves native Abortable and supervision cleanup outside the gate. Therefore add this opt-in native handoff:

~~~rust
pub type ControlledJob =
    Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub fn spawn_controlled<F, Fut, T, S>(
    self,
    f: F,
    failure: Arc<AtomicBool>,
    submit: S,
) -> Handle<T>
where
    F: FnOnce(Self) -> Fut + Send + 'static,
    Fut: Future<Output = T> + Send + 'static,
    T: Send + 'static,
    S: FnOnce(ControlledJob);
~~~

The shared failure flag starts false and never resets during a run. Its observer is an atomic store, not a user callback that could panic or emit protocol effects. Driver, signer and publication guards observe it. The submit callback is trusted, nonpanicking registry enrollment: immediately take custody, register initial readiness and return; never poll the job inline.

Preserve native metrics, supervision registration and task admission. Defer the user constructor and observe constructor/poll panics before Handle::init's inner catch:

~~~rust
let user_failure = failure.clone();
let deferred = async move {
    match AssertUnwindSafe(async move { f(self).await })
        .catch_unwind()
        .await
    {
        Ok(output) => output,
        Err(panic) => {
            user_failure.store(true, Ordering::SeqCst);
            resume_unwind(panic);
        }
    }
};
~~~

Pass deferred into native Handle::init. Retain the existing full native panic/task-tracker wrapper around the resulting unit job. In that wrapper's controlled-path outer catch, set failure before calling Panicker::notify:

~~~rust
Err(panic) => {
    failure.store(true, Ordering::SeqCst);
    panicker.notify(panic);
    Poll::Ready(())
}
~~~

That observation is mandatory: cancellation-future and undeliverable-return-value destruction can panic outside the user-future observer but inside this native catch.

Immediately before the current dedicated/shared executor dispatch branches, the controlled method instead calls submit(Box::pin(full_native_job)) and returns the native Handle. Do not submit merely the user future or wrap the complete job in tokio::spawn. Tokio-owned wrappers can be destroyed without polling during shutdown.

The handoff includes deferred construction, Abortable, native result send, supervision epilogue, native panic handling and task-tracker release. Rejected admission/closed-scope behavior stays upstream-compatible; the call and any rejected-capture disposal still occur under a grant. Ordinary Spawner::spawn remains unchanged.

## 6. Polling, custody, quiescence and failure ownership

Each job receives a deterministic task ID from logical parent/spawn order. Each user/native-job poll requires a single-use (run, incarnation, grant, task, poll-sequence) permit. Proxy wakers only mark logical readiness; they do not poll work. Preserve native biased-select semantics and deterministic queue order.

The registry exclusively owns complete native jobs. Release registry locks before polling or dropping a job: constructors, destructors, channels and supervision may enqueue further work. Initial registration always marks a task ready, even when nobody awaits its Handle.

After a job returns Ready, explicitly dispose of it under the same permit before declaring completion. Quiescence requires all runnable work, cancellation/disposal work and granted physical-I/O completions to be accounted for. Parent completion/abort must drain descendant cancellation to a fixed point, including never-polled children. Native running-task gauges are diagnostic, not quiescence evidence; abort can decrement them before destruction finishes.

Configure native Tokio with **with_catch_panics(true)**. The inspected Panicker logs and returns under this setting, preventing a native task panic from interrupting Runner root before the controlled registry can drain. This is permitted only with the mandatory observers in §5; otherwise native catches could silently consume unawaited-task or destructor panics.

Runner root exclusively owns the surviving registry and driver. RPC/controller pumps are separate cancellable transport-only tasks and never own protocol jobs. Their cancellation must not destroy the registry. Root catches driver-loop errors/panics without moving registry ownership into the caught/dropped temporary future.

The scheduler catches complete-job poll and explicit disposal panics. All step/syscall errors follow the same failure path. Check the permanent flag after each poll/disposal and before Ack commitment or output release. On first failure:

1. Revoke ordinary execution, signing and publication permissions.
2. Mark INFRASTRUCTURE_FAILURE; do not emit a successful Ack.
3. Reject new protocol work.
4. Drain cancellation/disposal under explicit failure-cleanup permits while root retains custody.
5. Return only after the recorded cleanup endpoint.

Failure-cleanup permits authorize cancellation/destruction only, never a new constructor, signature, publication or successful Ack. Direct disposal of the complete native job is allowed during failure cleanup; ordinary work need not be polled to obtain cleanup. Catch and record cleanup destruction panics. Newly produced cancellation/disposal work is drained to a fixed point.

Normal shutdown empties the registry before Runner root returns. Native Runner otherwise aborts its tree and waits for tracked jobs which the vanished driver can no longer poll. Arbitrary raw root cancellation is not exposed. Unscripted process/runtime teardown and bounded-cleanup failure are infrastructure failure with no successful-Ack or completed-cleanup claim. The existing 30-second infrastructure watchdog never becomes protocol time or a protocol crash.

### Return-value ownership

Producer-side gating covers an undeliverable result dropped during native sender.send. It does not automatically cover a result retained in Handle<T> and dropped later by its consumer.

Protocol-bearing Handle<T> and returned T remain owned by controlled jobs; their eventual disposal is controlled. Only inert infrastructure Handle<()> may escape, with its lifecycle still managed by the driver. Raw transport tasks may not own protocol-bearing returns. A general-purpose guarantee for arbitrary external Handle<T> disposal would require a larger API and is not claimed here.

## 7. Clock, stop, RNG and Metrics

Implement actual Clock and governor traits. current() is synchronous/nonblocking and reads the fixed grant NodeClock; governor now() reads that identical source. sleep/sleep_until register absolute local deadlines and wake only through controlled events. Test default Clock::timeout, including equal-deadline ordering. No delegation to native SystemTime::now or tokio::time::sleep.

Implement Spawner::stop/stopped with the public signal mechanism and controlled timeout. Preserve first-stop-value and signal-lifetime semantics. Do not delegate a timeout to native stop, which uses the native wall-clock sleep. Native supervision and Handle behavior come from the handoff, not completion-handle emulation.

The I1 sample/lease rules remain binding: only accepted delivered RPC samples renew leases; tick equality is expired; grants never renew; ClockUnsafe/Close/stop guards are checked independently of queued notifications. Manager, application and BFT use one NodeClock. Restart preserves the conservative expired-during-downtime termination policy.

Implement rand_core::TryRng<Error = Infallible> and TryCryptoRng, yielding the blanket CryptoRng implementation. Seed a pinned cryptographic PRNG from run seed, node, cid and stable role path. Do not delegate to native system randomness. Preserve/reconstruct approved RNG state on restart; record deterministic draw order. No PID, wall time, task address or host-start order affects draws.

Delegate actual Metrics registration/encoding and Supervisor naming/attributes to native facilities. Detached metrics are not a registration implementation. Wall-cost telemetry is diagnostic and excluded from canonical protocol trace comparisons.

## 8. Actual Storage/Blob and recovery

Wrap native Storage/Blob to retain filesystem semantics, while controlling when protocol tasks observe physical completion. Required actual methods include Storage::open_versioned/remove/scan and Blob::read_at_buf/read_at/write_at/resize/sync/start_sync; test read_many/start_read_many where used.

Minimum I/O profile: serialize issued physical operations in deterministic operation-ID order at the current virtual tick. Real disk latency delays wall completion, never virtual time. Buffer results until controlled release. Pending physical I/O is distinct from waiting for a future virtual event; do not Ack merely because user tasks are temporarily unrunnable.

Preserve upstream requirements: open/reopen readable data is crash-durable; version and partition checks; one live open per blob; removed handles remain readable; write/resize differs from sync; WriteOptions::SYNC is not a barrier for unrelated earlier operations. start_sync acceptance is not durability; its Handle resolves only after durability. Dropping that completion Handle does not cancel the flush. No previous-incarnation I/O survives into restarted protocol execution.

Retain the native Simplex voter journal and the outer controlled-step/Ack ledger as separate durable objects. The latter stores adapter state/output identity, not a fictional serialization of private Simplex futures. Recover native state through native journal replay, then recover/deduplicate external outputs using the outer ledger.

Test the boundary where native vote sync succeeds but the outer Ack record does not. No output may have escaped; replay may republish the recovered exact native vote identity. Application certify(true) requires its necessary local body/state durability before the native finalize vote can proceed.

## 9. p2p, application and evidence boundaries

Implement actual LimitedSender/CheckedSender/Sender, Receiver and Blocker. Include receiver capacity, local queue feedback, controlled rate-limit clock, peer blocking/subscriptions and three native epoch-scoped channels. UnlimitedSender is required only if a selected path uses it.

Send feedback means local staging acceptance. Hub offline/overflow drop may follow. All outbound bytes remain staged in Ack; no direct worker-to-worker socket exists. Quota checks use the shared governor clock; helper futures must not introduce independent real-time sleeps. Bounds remain 1 MiB gRPC, 64 KiB payload and 64 envelopes per directed link/channel.

Use a small full-body application corpus with independent local storage. propose/verify/certify return native one-shot receiver types. Missing body/state leaves a response pending; abandoned work is cancelled. Relay bodies through the hub separately from control messages. Reporter stages activities in the current controlled output batch.

Reporter activity is not automatically verified finality or delivery. Independently decode/verify native certificates, then separately check complete validated body/ancestry. Certification is not finality, notarization is not finality, and proof observation with missing data remains pending delivery.

## 10. Synchronous native signing guard

GuardedScheme delegates native BLS multisignature verification/assembly, participant ordering, certificate types, subjects and N3f1 threshold. Its actual certificate Scheme::sign<D>(&self, subject) -> Option<Attestation<Self>> checks immediately before invoking the native signer:

~~~text
active run/grant/poll permission and no permanent failure
correct node/cid/epoch
usable delivered Clock sample and tick < lease_end
not durably Terminated
same NodeClock and C_m < local_stop
native subject/context binding unchanged
~~~

Cover native Notarize, Nullify and Finalize, including local-proposal-related notarization. A send/relay/reporter-only guard is too late. Record actual NativeSignCreated separately from native append, durable sync and external publication. No new signature/publication is allowed under failure-cleanup permissions.

Upstream state toggles issuance before notarize/finalize signing and assumes failure only for a verifier. Therefore initial engine execution waits for calibration/readiness, and a lifecycle/clock refusal is permanent for that cid. Never return None as a temporary request to retry later. Stop/abort the cid; renewed calibration cannot revive it. Preserve native per-kind eligibility rather than adding an invented prohibition across all vote kinds.

## 11. Minimum independent engine acceptance matrix

Approved seeds: **7, 42, 99**. Preserve all twenty I1 cases; the matrix below provides the actual native adapter obligations. Use two real engines in the five-member registry, with explicit scripted peer input only where needed. Per-view rotation and disabled optional optimizations must be frozen in configuration; do not claim the later PoC leader schedule merely from a default elector.

| ID | Case | Required observation |
| --- | --- | --- |
| B1 | Native task control | Engine, voter, batcher, resolver and application constructors/polls/disposals require permits; no ungranted effects. |
| B2 | Wakes and selection | Duplicate/cross-task wakes, initially ready spawn without awaiting, biased-select ties and timer wakes reproduce across approved seeds/wall pauses. |
| B3 | Supervision | Parent completion/abort cancels descendants, including never-polled children; cleanup reaches fixed point before quiescence/Ack. |
| B4 | Clock/governor | Native timeout, quota and Manager calls use one NodeClock; only delivered samples renew leases. |
| B5 | Native quorum | Three distinct signatures fail quorum-four threshold; four verify. Two live engines without scripted quorum input do not claim quorum finality. |
| B6 | Native transport | Votes/certificates/resolver/body/blocking paths are controlled; accepted submission followed by remote drop is distinguished from receipt. |
| B7 | Native asynchronous work | Pending propose/verify/certify handles temporary data delay; completion after Close/stop cannot create a native signature. |
| B8 | Actual signer guard | Notarize/Nullify/Finalize before/equal/after lease and stop; none created at equality/later or after permanent failure. |
| B9 | Native durability | Crash before sync, after sync/before publication, between native sync and outer Ack commit, and after Ack; no non-durable released vote or duplicate new signature. |
| B10 | Native recovery | Same voter partition preserves native signing history plus adapter terminal/first-receipt/output state; mismatched context rejected. |
| B11 | Storage contract | Reopen durability, versions, live-open uniqueness, remove/read semantics, dropped sync-completion Handle and absence of old-incarnation I/O. |
| B12 | Evidence/data separation | Valid finalization before body/ancestry records proof observation only; delivery follows independently verified contiguous data. |
| B13 | RNG and replay | Same config/seed/fault script under different host pauses/start orders yields identical canonical trace and native signature bytes. |
| B14 | Negative checker | Reject ungranted/stale-clock signing, absent durability barrier, false quorum, proof-only delivery and successful Ack after failure. |

### Mandatory constructor, panic and disposal refinements

Each refinement has an independent sentinel/assertion and identifies its enclosing B case; an unawaited Handle must not conceal failure.

| ID | Injected event | Expected result |
| --- | --- | --- |
| P1 | Register a task without any poll permit | Constructor sentinel remains untouched; first authorized poll invokes it once. |
| P2 | Constructor panic, including unawaited Handle | Deferred observer sets permanent failure before native catch consumes panic. |
| P3 | User-future poll panic | Same failure observation; no successful Ack or later sign/publication. |
| P4 | Abort before first poll | Constructor never runs; closure captures dispose under cancellation permission. |
| P5 | Pending-future cancellation destructor panic | Controlled native outer observer or explicit disposal catch records failure. |
| P6 | DropBomb result with receiver already dropped | Panic in native sender.send failure disposal is observed by the patched outer catch before Panicker::notify. |
| P7 | Completed result retained then dropped by consumer | Controlled consumer disposal has a permit; panic is visible even though producer already completed. |
| P8 | Complete-job disposal panic outside user poll | Scheduler disposal catch sets failure; no hidden registry destruction. |
| P9 | Parent completion/abort with never-polled descendants | Descendants cancelled/disposed under control; constructor not spuriously invoked. |
| P10 | RPC/controller pump cancelled or disconnected | Root retains registry custody; same failure/drain protocol applies. |
| P11 | Driver-loop or syscall/step error/panic | Root catches without dropping registry; permissions revoked before cleanup. |
| P12 | Cleanup generates more cancellation/disposal | Drain reaches fixed point; no registry lock held during poll/drop; gauge value cannot substitute for completion. |
| P13 | Failure before queued output/Ack release | No sign/publication/successful Ack after failure; retained evidence reports infrastructure failure. |
| P14 | Normal shutdown and bounded cleanup failure control | Normal registry drains before root returns; incomplete cleanup is infrastructure failure, never passed. |

Repeat timing-sensitive refinements under wall pauses. Do not introduce ordinary polling or signing merely to make cleanup terminate.

## 12. Dependencies, execution order and evidence

Rust remains **1.95.0**. Reuse the lead-approved I1 wire/dependency annex when integrating tonic/protobuf; do not create a competing schema or silently change transport. A separate prototype package, if assigned, has its own workspace/lockfile until lead integration.

Expected native integration dependencies are Commonware runtime/consensus/cryptography/p2p/utils/codec/parallel/actor at =2026.10.0, plus commonware-storage at that version if directly testing the native journal. The approved source-compatible auxiliary set is governor =0.10.4, rand_core =0.10.1, rand =0.10.3 with std_rng, futures =0.3.34 and bytes =1.12.1. Actual lockfile and feature selection are lead-owned and must be checked against the common annex. Metrics can use Commonware's public reexports; direct prometheus-client is not inherently required.

Implementation order:

1. Native controlled-job API and P1–P14 failure/supervision tests.
2. Actual engine type-bound compilation with Clock/governor, stop and RNG adapters.
3. Wrapped storage and B11/cross-ledger durability tests.
4. GuardedScheme, five-member registry and actual engine/app/relay/reporter paths.
5. Scripted peer transcripts and actual hub RPC across two subprocesses.
6. Independent integrated-revision verification, then simulator replay.

One Rust build/heavy simulation at a time. A compile-only or in-process result is a prototype result, not overall I1b PASS. Save contract/annex/patch/source hashes, candidate commit plus dirty diff, commands, tool versions, dependency lock, seeds, clock/link/work/fault configuration, raw native bytes, journal files, process records, virtual trace, separate wall diagnostics and independent checker reports. Include task constructor/poll/cancel/drop and failure-cleanup evidence.

Verification and simulation outputs are separate fresh directories. The simulator uses the independently verified integrated revision. Finite passing cases establish only their recorded scenarios/bounds; failed premises or infrastructure failures are explicit. Configuration naming a model/effort is not runtime execution evidence.

## 13. Remaining decisions and next-stage validity boundary

Lead-owned concrete freezes before builder/integration: vendor directory and patch feature name; actual patch/source hash; dependency/lockfile integration; exact poll/disposal/IO event encoding; concrete native timing/elector settings; scripted transcript fixtures and output ownership; permitted implementation paths. These may refine mechanisms but cannot weaken this contract's guards or acceptance cases.

The later Session Program validity mapping remains unresolved: Commonware Automaton::verify/certify requires consistent context-wide validity, while v10 first-receive admission and local-time conditions are node-local. Missing input/time must remain pending; local abstention must not be silently encoded as deterministic invalidity. A separate accepted contract and safety reasoning are required before regular/v10 claims. I1b does not supply that proof.

Other next-stage obligations include five live workers, the explicit public (beta,r,v) schedule, full Build/Accept/NotBefore and descriptor semantics, body/ancestry delivery, and later GFR/regular/Force/RESUME integration. No existing benchmark, SignedIntent fixture or scripted-peer certificate replaces those obligations.

Every handoff records contract version/hash, patch identity, candidate revision/diff, exact checks/outcomes, artifacts and open findings. I1b implementation and PASS status remain pending until that evidence exists.
