# I1 v1 — Controlled two-worker runtime

Date: 2026-11-14. Status: **behavioral contract accepted by the lead; implementation and verification pending**.

This contract freezes the I1 behavior and acceptance cases. The lead must freeze the exact protobuf schema, canonical encoding vectors and dependency manifest annex before the corresponding builder handoff/build. Those annexes must not change the behavior below without a versioned contract amendment.

## 1. Gates and claims

- **I1a: infrastructure fixture.** Run one hub and two real worker subprocesses with controlled Clock RPC, timers, work, messages and recoverable disk journals. Use a clearly labeled SignedIntent fixture to test guards and publication durability. Passing I1a establishes only the exercised infrastructure behavior.
- **I1b: Commonware integration.** Adapt and independently verify the actual Commonware 2026.10.0 runtime/p2p surfaces, including task supervision, scheduling, clock-dependent operations, storage and all observable outputs. Replay the affected I1 cases through those actual adapters.
- **Overall I1 requires both I1a and I1b.** I1a alone does not authorize proceeding to 15-node integration/result claims or handoff statistics. This preserves v2 §§5.2 and 13.1, which require the controlled runtime and Commonware bridge.

Neither gate establishes BFT quorum, cryptographic finality, complete Session Program conformance or production clock synchronization. SignedIntent is not a BLS signature or Simplex vote/certificate. Infrastructure progress, consensus progress evidence, finality evidence and validated contiguous block delivery remain distinct observations. I1a emits no Final, Finalization or DeliveredBlock claim.

## 2. Authority and source mapping

Paths are relative to the isolated checkout unless explicitly absolute.

| Source/version | Applicable rule |
| --- | --- |
| formalization_v10/poc_design_zh_v2.md, 2026-11-14, §§4.1–4.4 | Integer microsecond time; separate virtual/reference time, node clock, RPC delay, public time and wall time; sample/lease bridge; grants cannot renew leases; ClockUnsafe terminates the current cid. |
| Same, §§5.1–5.3 | One scheduler; real subprocesses; complete grant/Ack output batches; explicit work events; controlled timers/network/storage; scripted versus infrastructure failure. |
| Same, §§6.1–6.2 | Independent readiness/Enter; repeated Enter preserves state; stopped instances do not resume. |
| Same, §§12.2 I1/C1/C2 and 13.1–13.3 | Two-worker gate, wall-pause replay, staged implementation and evidence. |
| formalization_v10/00_arch_zh_v0_1.md §2.3 | C2 error bound, C3 monotonic continuous clock model, C5 shared node clock. |
| formalization_v10/02_session_program_zh_v0_1.md, 2026-11-09, §§3.1–3.2, 3.4–3.8, 4.9 | Durable signing history and first receive time; Create/Enter/Close; irreversible Terminated; signing-time stop guard; historical outputs survive closing. |
| Parent /mnt/reth/jy/regional_BFT_sim/DESIGN.md, 2026-11-07, §§1–2.1, 3.1, 5 | Durable recovery, native adapter obligations and evidence/delivery separation. Parent benchmark committee parameters are not this PoC's parameters. |
| formalization_v10/poc_v2_xuelab_handoff_zh.md §6 and poc_v2_xuelab_setup_zh.md §§5–7 | First infrastructure gate; isolated checkout; one writer; pinned dependencies and independent verification. |

The upstream source is Commonware **2026.10.0**, upstream VCS revision **92a2f68a0e131c8b69de41e84aec37da6280f212**, cached read-only at:

~~~text
/mnt/reth/jy/regional_BFT_sim/.tooling/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/
~~~

Relevant source paths below are relative to that cache root:

| Source location | Obligation or gap |
| --- | --- |
| commonware-runtime-2026.10.0/src/lib.rs:525 | Clock::current() is synchronous; sleep/sleep_until return futures. Clock also requires the governor clock traits. |
| commonware-runtime-2026.10.0/src/lib.rs:358 | Spawner includes task supervision and abort semantics. |
| commonware-runtime-2026.10.0/src/lib.rs:719 and :922 | Storage/Blob integration; completed writes are distinct from completed durable sync. |
| commonware-runtime-2026.10.0/src/deterministic.rs:557 | The deterministic executor advances its own time. Separate autonomous runners inside workers violate I1. |
| commonware-p2p-2026.10.0/src/lib.rs:58–211 | Submission feedback does not prove delivery; offline recipients may lose messages; Sender includes LimitedSender/CheckedSender behavior. |
| commonware-p2p-2026.10.0/src/simulated/mod.rs:1–11 | Per-link order and coupling to the deterministic runtime. |
| commonware-consensus-2026.10.0/src/simplex/actors/voter/actor.rs:313–375, :943–986, :1450–1455 | Append, sync, then publish; recovery must not forget an externally published vote. |

The current parent adapter is src/backends/simplex.rs: one deterministic runner near line 647, fixed committee assumptions and engine startup near line 970. Parent src/checker.rs also has fixed committee assumptions. These files are read-only references, not an external-worker runtime or a completed v10 adapter.

The regional-bft-conformance skill supplies the acceptance-case method. protocol-spec-writer was not found in the available skill catalog/local search. No paper quorum or finality rule is modified by this infrastructure contract.

## 3. Ownership and allowed paths

All writes belong to the isolated checkout /mnt/reth/jy/regional_BFT_sim/poc/sequencer_rotation_v2.

| Owner | Writable assignment |
| --- | --- |
| Lead | Contract approval, poc/proto/**, shared interface definitions, poc/Cargo.toml, poc/Cargo.lock, dependency/toolchain decisions and integration. |
| One builder | Explicitly assigned implementations under poc/crates/{sim-hub,sim-runtime,replica-worker}/, builder tests and poc/scripts/. |
| Verifier | Explicitly assigned poc/verification/i1/**. |
| Simulator | Fresh run directories under poc/artifacts/{verification,simulation}/i1/. |
| Spec writer for this handoff | Only poc/docs/contracts/task-contract-i1-v1.md; other annexes require a separate assignment. |

Generated build/cache material uses checkout-local paths designated by the lead. The parent checkout, its tracked/untracked work and its .tooling are read-only. Do not invoke the parent bootstrap: it changes directory to the parent and may write parent tooling. Preserve other agents' work. At this stage there is one writer at a time; concurrent code writers require the approved worktree workflow.

## 4. Frozen adapter choices and bounds

I1 runs one hub and **two simultaneously existing OS worker subprocesses**, each with independent memory, journal and identity. A restarted worker has a new PID/incarnation and reopens its original journal.

The following are accepted I1 adapter choices, not newly asserted source theorems:

1. Half-open leases: valid exactly when tick_us < lease_end_tick_us.
2. Baseline oscillator: rate exactly one with fixed epsilon_us in [-1000, 1000].
3. Sequential grants and deterministic causal tie ordering.
4. Full-step durable records with replayable Ack batches.
5. Conservative termination when an entered cid's prior lease expires during downtime.
6. Multiplexed Clock operations on the controlled tonic/protobuf RPC stream.

| Bound/configuration | Frozen value |
| --- | --- |
| Maximum encoded gRPC message | 1 MiB = 1,048,576 bytes, enforced on send and receive |
| Maximum envelope payload | 64 KiB = 65,536 bytes |
| Per-directed-link/channel queue capacity | 64 envelopes |
| Wall watchdog | 30 seconds; infrastructure failure only, never protocol time or timeout evidence |
| Approved seeds | 7, 42, 99 |
| Baseline clock error | At most 1,000 microseconds relative to the reference; pairwise error may be 2,000 microseconds |

A complete Ack must fit the message bound; silent truncation or partial Ack commitment is forbidden. Exceeding an output/resource bound reports an explicit failure. Per-link overflow policy is fixed in the scenario before execution; baseline uses drop-newest with a visible QueueOverflow event. No overflow silently becomes successful delivery.

## 5. Wire contract

Use **tonic and protobuf over loopback**, with bounded messages and an explicit schema version. Diagnostic stdout/stderr does not transport protocol effects.

~~~text
Register(RegisterRequest) -> RegisterReply
ControlledRun(stream WorkerFrame) -> stream HubFrame
~~~

Clock sampling and timers are typed RPC operations carried in Ack/Grant. This batches the design's conceptual ClockService interface without blocking an outstanding grant on a response requiring future virtual time. Transport receipt alone never installs a sample or wakes a protocol task.

The lead freezes protobuf field numbers, enum numbers, input/effect payload definitions and canonical test vectors before the builder handoff. Required semantic fields are:

~~~text
RegisterRequest {
  schema_version: u32,
  run_id: bytes32,
  node_id: u32,
  incarnation: u64,
  config_hash: bytes32,
  recovered_journal_seq: u64,
  recovered_state_hash: bytes32
}

RegisterReply {
  schema_version: u32,
  run_id: bytes32,
  node_id: u32,
  incarnation: u64,
  clock_domain: bytes32,
  config_hash: bytes32
}

Grant {
  schema_version: u32,
  run_id: bytes32,
  node_id: u32,
  incarnation: u64,
  logical_step_id: u64,
  event_id: u64,
  tick_us: u64,
  causal_wave: u64,
  tie_rank: bytes32,
  input: oneof,
  grant_hash: bytes32
}

Ack {
  schema_version: u32,
  run_id: bytes32,
  node_id: u32,
  incarnation: u64,
  logical_step_id: u64,
  event_id: u64,
  grant_hash: bytes32,
  status: enum { QUIESCENT, REJECTED },
  effects: ordered repeated Effect,
  durable_journal_seq: u64,
  state_hash: bytes32,
  logical_ack_hash: bytes32
}
~~~

Grant input variants:

~~~text
CreateFixture, AnchorReady, EnterFixture, BeginWork,
WorkCompleted, ClockSampleReply, TimerFired,
DeliverEnvelope, CloseFixture, PauseFixture, ResumeFixture,
RecoverPending, ScriptedFaultCheckpoint
~~~

Effect variants:

~~~text
ClockSampleRequest, ArmTimer, CancelTimer, WorkStarted,
SendEnvelope, ClockRead, SampleAccepted, SampleRejected,
Entered, SignedIntent, Terminated, MessageReceived,
TimerObserved, DiagnosticRejection
~~~

Clock and timer objects:

~~~text
ClockSampleRequest {
  request_id: u64
}

ClockSampleReply {
  run_id: bytes32,
  node_id: u32,
  incarnation: u64,
  request_id: u64,
  clock_domain: bytes32,
  sequence: u64,
  as_of_tick_us: u64,
  node_time_at_as_of_us: i64,
  uncertainty_us: u64,
  lease_end_tick_us: u64,
  rate_profile_ref: bytes32
}

Timer {
  timer_id: u64,
  generation: u64,
  cid: optional bytes32,
  local_deadline_us: i64
}
~~~

ClockSampleRequest inherits run/node/incarnation from its enclosing Ack. TimerFired carries the complete timer identity/generation and original deadline. ClockUnsafe is a diagnosed transition resulting in Terminated, with the cause and relevant lease/tick recorded; it cannot be merely an unexamined queued notification.

Network object:

~~~text
Envelope {
  run_id: bytes32,
  src: u32,
  dst: u32,
  channel: u32,
  message_id: bytes32,
  cid: optional bytes32,
  sent_at_tick_us: u64,
  payload_hash: bytes32,
  payload_bytes: bytes,
  attempt: u32
}
~~~

Counters cannot wrap. Decoders reject unknown schema versions, malformed identifiers and oversized payloads. The lead's canonical protobuf encoding has **no maps** and no floating-point fields. Ordered repeated fields preserve semantic order. Freeze unknown-field, duplicate-field and presence handling in the schema annex so noncanonical encodings cannot acquire ambiguous identities.

Domain-separated hashes use SHA-256 over length-prefixed byte strings and fixed-width big-endian integers. The schema annex freezes domain tags, field order, widths, length-prefix width, optional-field tags and canonical encoded-message vectors. Do not hash ad hoc JSON, debug formatting or map iteration order.

Logical step/output identities survive worker restart. Transport incarnation is excluded from the logical Ack digest. Recovery may wrap the same logical Ack in a new transport envelope without altering its effects or logical identity.

## 6. Scheduler and output guards

Only the hub advances virtual time. Queue order is lexicographic:

~~~text
(tick_us, causal_wave, tie_rank, event_id)
~~~

- Scenario events have stable declaration-derived IDs.
- Derived events receive IDs deterministically from committed effects.
- tie_rank is the domain-separated SHA-256 of the I1 tie domain, seed and event ID under §5's encoding.
- An event generated at its parent's tick has causal_wave = parent.wave + 1; future-tick events begin at wave zero.
- IDs and ranks never depend on OS completion order, PID or wall-clock arrival.
- Seeds 7, 42 and 99 exercise different legitimate same-time orderings; checker invariants must hold for each ordering.

One grant is outstanding globally. At a grant, virtual time is fixed until the worker reaches quiescence and returns its entire Ack, or an explicitly configured fault checkpoint is reached. Runnable jobs use stable FIFO/task-ID ordering. Async work waiting on time or input becomes a registered wait, not a wait for future virtual time inside the step.

The hub validates the whole Ack before committing effects, commits it exactly once, then queues derived events. Identical duplicates are idempotent. An altered duplicate, unsolicited Ack, wrong event/incarnation or observable output outside a granted batch fails the run as an infrastructure error. A rejected application input may produce a REJECTED Ack without becoming infrastructure failure; malformed control framing may not.

Transport threads may buffer frames. They cannot mutate fixture state, install samples, fire timers, sign or send envelopes independently. All observable storage effects must be attributable to the current step/recovery operation and represented in its durable evidence. No output thread continues protocol work after Ack.

The 30-second wall watchdog detects infrastructure stalls. A watchdog firing stops that run with INFRASTRUCTURE_FAILURE and never advances virtual time or invents a protocol crash. Wall-pause comparison cases remain below the watchdog; a separate case deliberately exceeds it.

## 7. Clock guards

The baseline projection is:

~~~text
C_m(tick) = sample.node_time_at_as_of
            + tick - sample.as_of_tick
~~~

It projects the fixed continuous function tick + epsilon; it does not install the historical returned timestamp as current time. Local times use signed integer microseconds to avoid underflow with a negative offset. All conversions use checked arithmetic. The runtime may know the reference tick internally; fixture protocol callers receive only their NodeClock value, not another node's clock or the fault script.

ClockSampleRequest commitment schedules request transit, service sampling and response delivery as separate deterministic events. Record all three times and configured delays. Actual RPC/CPU wall delay does not alter them. Sample sequence allocation follows deterministic service order.

A sample becomes usable only through its response-delivery grant and complete validation:

1. Run, node, incarnation, domain and rate profile match.
2. The request is outstanding and not already accepted.
3. Sequence strictly exceeds the durable accepted-sequence watermark.
4. as_of_tick_us <= delivery_tick_us < lease_end_tick_us.
5. Baseline projection agrees with the fixed oscillator profile.
6. uncertainty_us <= 1000 and covers the claimed baseline offset.
7. Installing the sample preserves the model's continuity; no max(last, new) correction is allowed.

Rejection is atomic: no accepted-sequence, lease, NodeClock or unrelated-request mutation. In particular, rejecting invalid sequence 100 must not prevent accepting valid sequence 3. Record the reason and sample identity. A rejected response does not count as calibration or consume a different request.

Only an accepted Clock RPC reply changes calibration/lease state. Grant ticks do not renew leases. Baseline perfect-rate projection accounts for response age explicitly; RPC delay is not added to epsilon or treated as proof of offset accuracy. Nonzero slew/rate profiles require a later contract specifying the u + rho*a error budget and endpoint checks.

At every protocol clock read, Enter and actual SignedIntent creation:

1. Check calibration and lease.
2. Check permanent Terminated.
3. Check C_m < stop_local_us.
4. Apply the fixture-specific eligibility guard.

Lease equality is unsafe, independently of whether a queued ClockUnsafe event has run. ClockUnsafe produces durable termination of the entered fixture cid and blocks new Enter until calibration recovers. Recovery never resurrects that cid.

The fixture Manager and fixture BFT callers share the same NodeClock. Trace both caller identities and values. Monitor the actual error at every read/sign-intent point and profile endpoint; an undetected clock error is a failed premise, not healthy-node evidence.

## 8. Timers, work, lifecycle and routing

Timers retain **absolute local deadlines**, IDs and generations. Renewal never restarts a sleep duration. For rate one, the hub schedules the first allowed tick with C_m >= deadline; already-past deadlines are immediately eligible. Cancellation invalidates the generation. A generation fires at most once. The schema annex distinguishes cancellation, delivery and suppressed stale delivery in trace records.

A restarted worker restores deadlines. An already-due timer becomes eligible after safe recovery with its original deadline. Terminal-cid timers are cancelled visibly rather than used to resume consensus. Renewal cannot resurrect a cancelled timer generation.

At least one fixture work item has nonzero configured virtual duration:

~~~text
BeginWork -> WorkStarted -> queued WorkCompleted
~~~

WorkCompleted rechecks clock, local stop and terminal guards immediately before creating a SignedIntent. Wall execution cost is separately diagnostic.

Create allocates a fixture descriptor without starting a view. Enter waits for anchor readiness and calibration, rejects terminated/expired instances, and records the first entry time. Repeated Create/Enter preserves the original descriptor, entry time, current view and signing history. Separate worker events must demonstrate staggered readiness/Enter; no all-workers-ready barrier supplies simultaneous starts. Fixture view progress is not BFT progress.

Every envelope comes from a committed Ack and is routed through the hub. Validate source ownership, run, payload hash, cid binding, maximum size and stable message identity. No worker-to-worker socket is allowed. First receive time is preserved across duplicates/retries and restart.

Each link records latency, jitter, online intervals, queue bound, overflow behavior and retry policy. Baseline links preserve order per directed link/channel. Admission, drop, delivery and receiver processing are separate events. Submission acceptance never implies receipt. Payload/retry identity remains stable across retries; attempt is a transport coordinate, not a new logical decision.

I1a does not promise cross-region reliable outbox delivery or indefinite raw-vote retention. Those need later acceptance cases. Closing a cid retains previously created output; it does not convert the network API into a reliable-delivery guarantee.

## 9. Journal, restart and termination

Use a real disk journal per worker. A complete step record contains:

~~~text
StepCommit {
  journal_seq,
  previous_record_hash,
  logical_step_id,
  grant_hash,
  resulting_state,
  ordered_effects,
  logical_ack_hash,
  checksum
}
~~~

Durable state includes:

- Fixed run/config/node identity and fixture descriptors.
- Enter/termination records, stop deadlines and current fixture view.
- SignedIntent coordinates and exact bytes.
- First receive timestamps.
- Absolute timer deadlines/generations.
- Sample request counter, accepted sequence and prior lease end.
- Cached complete logical Ack batches and their binding to the original grant.

Append and complete sync_all or an equivalent durability barrier **before exposing Ack**. Creating a new journal also requires a documented directory durability boundary. Append success alone is insufficient. A fixture signing coordinate is scoped by cid, message kind and view; replaying its exact bytes is distinct from creating a conflicting new intent at that coordinate.

| Injected boundary | Required behavior |
| --- | --- |
| Before complete durable StepCommit | No Ack/output release; recovery may retry the step. |
| After durable StepCommit, before Ack | Recover state and replay the exact cached logical Ack. |
| After hub commitment | Duplicate Ack cannot duplicate logical effects. |
| Torn tail | Discard only an incomplete uncommitted tail under the frozen framing rules. |
| Corrupt complete committed record or mismatched context | Fail closed; never initialize an empty replacement state. |

Crash injection inside a grant uses a named scripted checkpoint while that grant owns the step. It is not an OS-timing race disguised as a virtual event. Unscripted panic, disconnect, corruption or watchdog expiration produces INFRASTRUCTURE_FAILURE, not a simulated protocol crash. The supervisor manages only this run's recorded child processes.

On restart, the hub authorizes RecoverPending for a known durable step/hash. This permits replay of a pre-crash logical Ack under a new transport incarnation; arbitrary prior-incarnation frames remain invalid. The logical replay references the original grant/hash and cannot become a new signing event. The hub retains committed-step deduplication state throughout worker recovery. Hub crash recovery is outside I1a and must not be claimed.

A fresh incarnation needs fresh Clock RPC calibration before new signing. Preserve the previous lease end. If it expired during downtime, permanently terminate the old entered cid before renewal can enable work. This is an explicit conservative I1 recovery policy, not a new v10 theorem. Recovery completed before the old expiry may continue a nonterminal cid with its original state. In both cases, local stop must be checked before resumed scheduling.

Close is durable and idempotent. It stops new signing and retains existing signed intents, blocks/proofs if any later integration stores them, and pending outputs. Repeated Close returns the original terminal record. Replaying output created before Close is permitted without creating a new signature. No new intent is permitted at local stop equality even if a Close notification has not been delivered.

## 10. Acceptance matrix

All cases begin from independently specified fixtures and expected results. The verifier derives expectations from the sources and this contract before reading the candidate implementation. Run the approved seed set 7, 42 and 99 where ordering/links are seeded; fixed adversarial vectors are additionally identified by name. I1b reruns all applicable cases through its actual Commonware adapters and records explicit rationale for any fixture-only case that needs a corresponding adapter-level replacement.

| ID | Trigger/pre-state | Required observation |
| --- | --- | --- |
| I1-01 | Start two workers | Distinct simultaneously live PIDs, independent journals and actual RPC; no shared protocol memory. |
| I1-02 | Same seed/config, varied process-start order and wall pauses | Canonical virtual traces identical; wall/PID diagnostics may differ. |
| I1-03 | Worker waiting for sample/input/timer | No worker time advance; waits become registered pending work and do not obstruct hub scheduling. |
| I1-04 | Offsets -1000/+1000; one-way RPC delays 50/250 and 1000/10000 microseconds | Offset remains bounded independently of latency; pairwise difference may equal 2000 microseconds; same-node callers agree. |
| I1-05 | Delayed but unexpired sample | Project from as-of to delivery tick; never install an old timestamp as current. |
| I1-06 | Replay, wrong run/domain/incarnation, unknown request, future as-of or expired response | Atomic rejection; no renewal, counter poisoning or unrelated-request mutation. |
| I1-07 | Invalid sequence 100 followed by valid sequence 3 | Sequence 3 still accepted when otherwise valid. |
| I1-08 | Lost refresh; repeated grants; signing at lease end minus one, equal, plus one | First may sign under remaining guards; equality/later cannot; grants do not extend lease. |
| I1-09 | Calibration recovery following ClockUnsafe | Old cid remains terminal; only a separately permitted subsequent fixture cid can Enter. |
| I1-10 | Timer renewal, cancellation/rearm and restart | Original deadline survives; generations deduplicate; already-due and terminal-cancellation behavior follows §8. |
| I1-11 | Close or stop while work pending | Completion creates no intent after termination or at local stop equality, including both equal-tick orderings. |
| I1-12 | Duplicate Enter/Create/Close; staggered workers | Entry/view/signing history preserved; original termination returned; no global startup barrier masks readiness. |
| I1-13 | Crash at each journal checkpoint; torn tail and committed corruption controls | No externally released non-durable intent; cached Ack recovers once; no conflicting fixture intent; corruption fails closed. |
| I1-14 | Restart/calibration before versus after prior lease expiry | Before: safe continuation possible. After: old cid permanently terminates. Absolute timers and request/sequence state survive. |
| I1-15 | Duplicate grant/Ack, altered duplicates, wrong identity and unsolicited frames | Exact duplicates idempotent; altered/unsolicited frames fail closed; recovery replay is explicitly authorized. |
| I1-16 | Delayed/duplicate/offline traffic and 65th queued envelope; payload/message size boundary | Admission/drop/delivery separated; queue and size bounds enforced; stable IDs; no direct route. |
| I1-17 | Unscripted kill/watchdog versus declared checkpoint crash | Infrastructure failure distinguished from intentional protocol fault; wall time never becomes a protocol clock. |
| I1-18 | Close and work completion at equal ticks under approved seeds | Trace order governs Close race; lease/stop guards hold for every ordering; same-tick causal order is explicit. |
| I1-19 | Checker receives deliberately mutated sample/send/journal trace | Rejects grant-based lease extension, missing sync-before-send, altered duplicate and output outside grant. |
| I1-20 | Terminal cid retains cached pre-stop output | Replay allowed without increasing SignedIntent creation count or erasing terminal state. |

No two-worker case establishes a four-of-five quorum or bounded BFT liveness. A fixture timer firing or message arriving is not a finality proof or block delivery event. A finite passing suite provides evidence only for the recorded scenarios and bounds.

## 11. I1b Commonware adapter obligations

I1a provides a small executable harness. I1b must close the following gaps before overall I1 passes:

| Boundary | Required I1b integration evidence |
| --- | --- |
| Clock and governor | Implement the actual Clock trait, governor clock traits and relevant timeout/stop/rate-limit paths. current is synchronous, nonblocking and reads the grant's fixed NodeClock; timer futures wake only on granted events. |
| Spawner/Supervisor/executor | Actual spawning, child supervision, abortion and select/waker behavior stay inside controlled execution. Stable ordering reaches quiescence without background consensus output after Ack. Test work spawned but not explicitly awaited. |
| RNG | Per-role seeded state is reproducible and recovered as needed; process-start order and uncontrolled thread scheduling cannot consume random values differently. |
| Storage/Blob/journal | Actual storage calls and the upstream journal's append/sync/publication sequence obey the controlled boundary. Acknowledging acceptance of asynchronous sync is not acknowledging durable completion. |
| p2p | Actual LimitedSender/CheckedSender/Sender, UnlimitedSender where used, and Receiver contracts are mapped, including feedback, rate limits, receiver capacity, offline/drop semantics and message bounds. recv wakes only through grants. |
| Simplex-owned work and outputs | Enumerate actual engine context bounds and every used task, application, relay, reporter, vote, certificate, resolver and storage path. Each path must either be exercised through the controlled adapter or remain an explicit blocker to overall I1. |
| Recovery and signing | Identify where real native signatures are created, persisted and published and where lifecycle guards must be checked. I1a's SignedIntent cannot substitute for this evidence. Full v10 validity and signing conformance remains a subsequent Session Program gate. |

Replacing only Clock::current or starting independent deterministic runners does not pass I1b. A compatible trait-shaped wrapper without exercised behavior does not pass. Capture compile checks against the pinned types and behavioral tests on the integrated candidate. If upstream surfaces cannot support the required control, report the exact gap and a versioned minimal alternative; do not silently relax the gate to in-process function calls.

## 12. Dependencies and evidence package

Minimum environment: Rust **1.95.0**, a C toolchain where dependencies require it, pinned tonic/prost/Tokio, a pinned protobuf generation toolchain, hashing/serialization and Python for independent checking. Commonware remains **2026.10.0**; do not upgrade it to solve an adapter issue silently.

Read-only cache inspection found Tokio 1.54.1, prost 0.14.4, serde 1.0.229 and serde_json 1.0.154. Tonic/build and vendored protoc were not found in that inspection. This is cache evidence, not a successful compile or an approved dependency set. Exact tonic/protoc and transitive pins belong to a **lead-owned dependency manifest annex frozen before build**. Do not silently replace protobuf/gRPC with JSON/TCP. Do not infer an offline build succeeded from an existing cache.

Build in the isolated workspace with a local target/cache policy; run one Rust build/heavy simulation at a time. Verification output and simulator output use separate fresh directories. Run affected checks on the integrated revision before formal simulation. Source configuration describing Ultra or a role model is not runtime model/effort evidence.

Minimum evidence package:

~~~text
manifest.json
source-and-spec-hashes.json
scenario.json
virtual-trace.jsonl
transport-diagnostics.jsonl
worker-0/journal
worker-0/process-history.json
worker-1/journal
worker-1/process-history.json
checker-report.json
commands.txt
~~~

Record candidate commit plus dirty diff/hash, contract/schema/dependency hashes, tool versions, seeds, all clock/link/work/fault settings, exact commands, process identities, checker outcomes and failure output. Virtual trace includes the causally ordered grants, reads, sample events, timer/work transitions, journal durability boundaries, Ack commitments and routed messages. Diagnostics retain actual PID/wall-time/transport facts separately.

Canonical replay comparison excludes wall elapsed, PIDs and ephemeral ports; it does not discard protocol-visible outcomes to force equality. Use the same logical run ID/configuration in paired runs, or a frozen explicit normalization for run IDs. An independent checker consumes raw evidence rather than calling the implementation's state-transition routines. Report PASS, FAIL or INCONCLUSIVE for each gate and case; infrastructure failure cannot be reported as a protocol PASS.

## 13. Remaining lead-owned annexes and handoff

Accepted behavioral choices are multiplexed tonic/protobuf Clock operations, half-open leases, conservative expired-during-downtime termination, SignedIntent-only I1a scope, full Commonware I1b prerequisite, the bounds in §4 and seeds 7/42/99.

The following remain to be frozen by the lead before the affected implementation/build:

1. Exact protobuf field/enum numbers and payload schemas; canonical bytes/hash vectors and duplicate/unknown-field handling.
2. Dependency/protoc manifest, exact locked versions, writable cache/target paths and reproducible commands.
3. Journal framing, checksum/canonical record bytes, directory-sync procedure and exact fault checkpoint hooks.
4. Concrete scenario files for the approved seed set, including refresh/lease lengths, link delays, work durations, stop times and sub-watchdog wall pauses.
5. I1b path ownership and actual Commonware integration candidate, with its distinct review/evidence record.

These annexes complete concrete implementation choices; they cannot lower a guard, alter a signing/finality rule, remove a case or redefine I1a as overall I1. Any such change requires a versioned contract amendment approved by the protocol/lead owner.

Each handoff records contract version/hash, candidate revision or diff, exact checks/outcomes, artifact paths and open findings. This document is a contract, not evidence that any PoC implementation/test passed.
