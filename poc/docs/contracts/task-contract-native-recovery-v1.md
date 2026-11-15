# NATIVE-RECOVERY/TIMING v1 — bounded same-cid process recovery

Date: 2026-11-15. Status: **FROZEN by lead; not implemented or passed**. Documentation writer owns this file only. No code, build or commit is part of this handoff.

This slice follows the Fresh formal matrix, reported by the lead as 66 passing cases at fec7ade. The implementation-boundary review used 6686a00. Neither revision implements the recovery described here. Overall I1b still requires independently verified recovery and an executed native resolver-repair case. This contract makes no 15-node, v10 admission/GFR, or BFT-5/6 proof claim.

## 1. Scope and source authority

Retain the Fresh profile: five equal-weight PoP-verified members, quorum four, two real OS workers0/1 and explicitly scripted members2–4; epoch1, Genesis floor/view0, native views1–8, retention16, no pruning, Dynamic elector with rotating terms, no early skip/forwarding/stable-term/Stage optimization. Maximum fixture physical height is two. Member, signer-index and process identity remain distinct. Scripted peers are not a live quorum claim.

Same logical run/config/cid/epoch and signing namespace survive physical restart. Worker incarnation and storage recovery generation change; they do not create a new signing domain. The native namespace remains H(I1b/voter-namespace/v1, run, config, cid, epoch-u64BE). Recovery profile configuration and optional session-start floor are fixed before genesis and cannot change during a run.

Authority: v2 §§4–6/13; v10 Session Program §§3.1–3.8 and 4.9; parent DESIGN.md §§2–2.1; task-contract-i1b-v1.md; accepted NATIVE-STORAGE/WITNESS contracts; task-contract-native-engine-fresh-v1.md. Commonware stays 2026.10.0, upstream 92a2f68a0e131c8b69de41e84aec37da6280f212, with separately hashed opt-in runtime and consensus patches. The parent checkout and cached upstream source remain read-only.

| Source anchor in pinned Commonware | Required recovery hook/interpretation |
| --- | --- |
| consensus/src/simplex/actors/batcher/actor.rs around 704 | Selected leader proposal is forwarded before all signature verification has completed. Carry its opaque ingress provenance at this boundary; selection is not signature verification. |
| consensus/src/simplex/actors/voter/ingress.rs; voter/actor.rs around 887 | Preserve ordinary Proposal mailbox/API; add a feature-gated recoverable variant carrying the input reference and validation level. |
| voter/slot.rs:37–155 | Proposal/status and request flags are volatile. Recreate claims through native handlers; do not write requested flags without their operation ownership. |
| voter/state.rs:376–394 | enter_view computes deadlines from current time; restore original timing instead of replay-created deadlines. |
| voter/state.rs:1067–1091, 1131, 1166, 1273 | Use proposed, set_proposal, try_verify and verified with their captured-parent checks. |
| voter/actor.rs:430–589 | Claim/dispatch/reconcile and Waiting/Closed cancellation semantics need plain operation/phase records. |
| voter/actor.rs:1270–1405 | Record actual native response consumption, including Wait/Closed branches that continue before on_end. |
| voter/actor.rs:313–375, 1433–1460 | Persist preparation before native artifact durability and before notify; native append/sync precedes publication. |
| voter/actor.rs:1107–1239 | Journal replay omits auxiliary state; reconstruct before resolver.replayed, normal batcher processing, app reconciliation and next_timeout. |
| voter/round.rs:514–600, around 787 | Retain deadlines/latch; replayed Nullify retry=None must not become recovery-now+retry. |
| commonware-resolver/src/p2p/fetcher.rs around 417 | First transient request ID uses next_u64; restore role RNG before constructing new request generation. |

The actual 6686a00 boundaries are native_application.rs::Archive::{queue,start_ready,complete}, App::{start,persist}; native_replica.rs::{start,step,deliver}; native_step_journal.rs::StepJournal::{create,commit,cached}; native_network.rs::{NativeRx,PacketFactory}; rng.rs::RngCheckpoint. WorkCompleted currently removes Pending before helper body persistence and final native reply. StepJournal currently creates a fresh file and retains cached Acks in memory. Those APIs need the explicit extensions below; their existing audit records are not a recovery implementation.

## 2. Ownership and exact allowed implementation files

Lead owns approval, profile/wire field numbering and hash vectors, dependency/lock integration, source copying and final integration. One builder owns only assigned files from this allowlist. No simultaneous writers without the approved worktree workflow. Verifier/simulator output paths are independently assigned.

Prototype files under poc/prototypes/i1b-runtime/:

~~~text
src/native_recovery.rs                 new coordinator and plain records
src/native_recovery_ledger.rs          new anchored ledger
src/native_application.rs
src/native_replica.rs
src/native_step_journal.rs
src/native_network.rs
src/native_scheme.rs
src/native_hub.rs
src/native_config.rs
src/driver.rs
src/context.rs
src/clock.rs
src/rng.rs
src/lib.rs
src/bin/native-worker.rs
tests/native_recovery.rs               builder acceptance
tests/native_recovery_faults.rs        builder fault fixtures
~~~

Lead-owned integration files: poc/proto/native.proto, prototype src/native_wire_support.rs, Cargo manifests/locks and build wiring. Existing I1a code/proto is unchanged. The lead copies the pinned consensus crate to poc/vendor/commonware-consensus; permitted consensus patch files are:

~~~text
Cargo.toml                            feature native-recovery
src/simplex/mod.rs
src/simplex/recovery.rs                new public plain types/hooks
src/simplex/engine.rs
src/simplex/actors/batcher/mod.rs
src/simplex/actors/batcher/actor.rs
src/simplex/actors/batcher/round.rs       provenance carry only
src/simplex/actors/batcher/verifier.rs    provenance carry only
src/simplex/actors/voter/mod.rs
src/simplex/actors/voter/ingress.rs
src/simplex/actors/voter/actor.rs
src/simplex/actors/voter/state.rs
src/simplex/actors/voter/round.rs
src/simplex/actors/voter/slot.rs
src/simplex/actors/resolver/mod.rs
src/simplex/actors/resolver/actor.rs
~~~

Batcher changes are provenance/startup-fence hooks only; resolver changes are startup-fence/request-generation observation only. No native vote/certificate/Artifact encoding, cryptography, quorum or lock rule changes. Ordinary Engine::new and the ordinary mailbox path keep upstream behavior. Existing runtime whole-job/panic/storage/witness guards may not be weakened. No new dependency is required beyond the existing futures BoxFuture and serialization/crypto primitives.

## 3. Frozen encoding, bounds and identities

H uses the existing SHA-256 framing: domain UTF-8 and each component separately prefixed by u64 big-endian length. Scalars passed directly to H use their declared big-endian width. Canonical record payload is compact serde JSON over the exact ordered fields below; IDs are lowercase 64-character hex, raw bytes lowercase hex, enums exact uppercase strings, optional values JSON null, and ordered collections arrays sorted by their stated key. No unordered maps/floats enter identities.

Bounds: 16 MiB auxiliary/root frame, 64 KiB input envelope, 4096-byte native Artifact frame, 64 KiB per selected native blob, 1024 proposal/operation records per cut, 8192 input/auxiliary events per worker run, 4096 retained signed subjects, 64 pending helper operations. Existing 1 MiB gRPC, selected page256/logical244, cache4 pages/write488/replay256, 30-second infrastructure watchdog and finite poll bounds remain. Exceeding a bound is explicit failure, not truncation or a new committee assumption.

Logical operation IDs already emitted by Archive::queue are preserved byte-for-byte. A separate reconnect_generation increments per logical operation; it is not part of native subject, original WorkBinding or work identity. Physical incarnation is checked old+1 with overflow rejection. Storage recovery generation follows existing Witness old+1 rules. No counter wraps.

New domains:

- I1b/recovery-record/v1: canonical auxiliary Record bytes, excluding its own record_hash.
- I1b/recovery-head/v1: run, config, worker-u32BE, preceding 56 head bytes.
- I1b/recovery-input/v1: run, config, worker-u32BE, original incarnation-u64BE, input_sequence-u64BE, grant_hash, exact canonical Grant bytes.
- I1b/recovery-cut/v1: canonical RecoveryCut bytes, excluding cut_hash.
- I1b/recovery-state/v1: canonical native recovery-sensitive state projection.
- I1b/recovery-body-manifest/v1: canonical ArchiveManifest bytes.
- I1b/recovery-cache/v1: canonical signer-cache entry bytes.
- I1b/recovery-ready/v1: cut_hash, actor tag UTF-8, restored state hash.

The auxiliary ledger uses u64 length, u64 complement, payload, 32-byte record hash; Record fields are schema, sequence, previous_record_hash, grant_intent_ref, kind, data. Head is 88 bytes: magic I1RCHEAD, offset-u64BE, sequence-u64BE, last hash32, bound checksum32. Create/sync files and directory; append+sync; create/sync pending head, rename, sync directory. The same root-owned pending/Ready custody and protected-history rules as accepted Witness apply. No possibly published/acknowledged history is truncated. One fully verified descendant above the head can be promoted only under hub-issued-grant authorization; corrupt complete records or missing committed bytes fail closed.

## 4. Exact plain records

All records include schema=1 and bind run/config/worker/cid/epoch through their ledger header/cut. Time fields are signed local_us or unsigned reference_tick_us as named. Native SystemTime/Duration conversions must be exact integer microseconds; reject unsupported precision rather than round.

### 4.1 Write-ahead input

~~~text
GrantIntent {
  input_id, input_sequence, original_incarnation,
  grant_id, grant_hash, canonical_grant_bytes,
  predecessor_root_receipt, predecessor_cut_hash,
  input_kind, source_member?, channel?, exact_envelope_bytes?,
  native_bytes?, original_receive_tick_us,
  original_receive_local_us?, transport_binding_verified,
  decoded, elected_leader_checked, signature_state
}
~~~

signature_state is NOT_APPLICABLE, PENDING, VERIFIED or REJECTED. Transport identity means the existing simulated authenticated source binding, not native signature verification. A missing usable local clock leaves local receipt null and input unadmitted. First receipt is fixed at the original applicable ingress event and never replaced by body download, replay or reconnect time.

Persist GrantIntent after envelope/binding validation but before driver.grant_async exposes that input to protocol tasks. This includes the current unacknowledged grant. Root-step after-the-fact storage alone is insufficient because the grant may cause a native barrier before its Ack.

NativeRx registers a consumed input reference before returning the native (public key, IoBuf). The opt-in batcher hook claims the next consumed input with exact channel, source and byte equality, in receive order; mismatch fails closed. It carries that opaque reference in feature-gated RecoverableProposal mailbox messages. Existing ordinary Proposal messages remain unchanged. Selection and later signature-validation outcome are separately appended records. Duplicate/conflicting proposals retain separate ingress references; no arbitrary proposal-hash lookup substitutes for this carry.

### 4.2 Proposal, operation and actor phase

~~~text
ProposalFact {
  fact_sequence, native_proposal_bytes, selected_input_ref,
  selection_sequence, first_receive_input_ref,
  original_receive_tick_us, original_receive_local_us?,
  transport_binding_verified, decoded, elected_leader_checked,
  signature_state, verifying_parent_digest?,
  application_result_ref?, observed_slot_status,
  conflicting_input_refs[]
}

OperationRecord {
  operation_id, original_work_binding, kind,
  canonical_native_context_bytes?, epoch, view,
  parent_view?, parent_digest?, requested_payload?,
  phase, helper_phase, claimed_sequence,
  original_start_tick_us?, original_due_tick_us?,
  missing_dependencies[], offered_outcome?, outcome_evidence_refs[],
  offered_sequence?, consumed_sequence?, cancelled_sequence?,
  cancellation_reason?, reconnect_generation
}

ActorPhase {
  pending_proposal_operation?, pending_proposal_context?,
  proposal_phase, pending_verify_operation?,
  verifying_proposal_bytes?, verifying_parent_digest?,
  outstanding_certify_operations[], committed_relay_refs[]
}
~~~

kind is PREPARE, PROPOSE, VERIFY or CERTIFY. phase is CLAIMED, OFFERED, CONSUMED or CANCELLED. helper_phase is WAIT_INPUT, WORK_SCHEDULED, WORK_READY, BODY_PERSISTING, READY_TO_OFFER or RETIRED. proposal_phase is NONE, PREPARING, PROPOSING, WAITING or CLOSED. Stage/Held is rejected in the main profile. Array ordering is fact/event order, except dependency/certification ID sets sorted by ID.

original_work_binding preserves the actual 6686a00 fields: work, member, epoch, view, parent_view, parent_digest, requested_payload, request_generation, request_tick_us. Do not call ordinary Archive::queue during reconnection: that would allocate a new work generation and identity.

offered_outcome is typed DIGEST(value), BOOL(value), HANDOFF_WAIT or RESPONSE_CLOSED. Work-ready Option<Block> is a helper fact, not an offered native result. For propose/certify success, record completed body durability before READY_TO_OFFER. Append OFFERED durably before resolving the actual native sender. Record CONSUMED only at the native actor response branch, with whether the handler applied or discarded the result and its exact binding. Record CANCELLED when native reconciliation drops/abandons the receiver, not merely when a later helper notices it.

### 4.3 Archive, signer cache and timing

~~~text
ArchiveManifest {
  blocks[], durable_body_receipts[], proofs[], native_parents[],
  body_requests[], body_retry_deadlines[], delivered_digest,
  delivered_height, emitted_delivery_ids[], work_generation,
  packet_attempt_counters[], blocked_members[]
}

SignerCacheEntry {
  cid, epoch, member, signer_index, kind, view,
  canonical_subject_bytes, canonical_attestation_bytes,
  original_crypto_invocation_id, original_grant_ref,
  originating_artifact_ref?, cache_entry_hash
}

TimingSnapshot {
  current_view, last_finalized_view, stall_anchor,
  native_state_projection_hash, rounds[]
}
round timing = {
  view, entered_at_local_us?, proposed_at_local_us?,
  comparison_decision_phase,
  leader_deadline_local_us?, certification_deadline_local_us?,
  latch_time_local_us?, latch_reason?, latch_cause_ref?,
  retry_deadline_local_us?, retry_arm_local_us?,
  stall_deadline_local_us?
}
~~~

Archive arrays are sorted by their native map key; events/receipts keep sequence order. Revalidate every block/proof/reference. Durability is established from recorded receipts and the accepted new-generation recovery procedure, not from an in-memory durable set or readable bytes alone. Unresolved work may legitimately lack a body. An offered/consumed positive result must retain its exact required evidence, from durable body files or bound write-ahead input bytes which are revalidated and newly persisted before reuse.

The signer cache is staged synchronously at actual native signing, then must be durably included in preparation before the corresponding native barrier/authorization or outer Ack. Scheme::sign cannot wait on an unowned network/disk future. Native pre-append/barrier hooks and root final flush persist staged entries through the root-owned store. A failed persistence forbids external release. Restore verified cache entries before enabling the signer. A recovered native own-vote artifact may supply missing cache bytes without any crypto call, with an explicit recovery-origin record. Conflicting cache/native histories fail closed. A pre-durability crash with no published history may have lost an invocation; never report that as a cache hit.

Native state projection is comparison-only: current/finalized view, retained round IDs, native proposal identities/status, captured verifying-parent bindings, local decision kinds, certificate/finality identities and certification states that determine timer eligibility. It excludes futures, metrics, addresses and newly recomputed timestamps. Never use the projection to overwrite votes, locks, certificates or finality.

### 4.4 Shared preparation and final cut

~~~text
RecoveryPreparation {
  preparation_sequence, grant_intent_ref,
  auxiliary_frontier, signer_cache_frontier,
  prospective_native_artifact_hashes[],
  actor_phase_ref, timing_snapshot_ref, role_rng_checkpoints[]
}

RecoveryCut {
  schema, cut_sequence, previous_cut_hash,
  run, config, worker, cid, epoch, profile_hash,
  grant_intent_frontier, auxiliary_frontier,
  proposal_frontier, operation_frontier, signer_cache_frontier,
  native_journal_boundary, witness_receipt,
  archive_manifest_ref, actor_phase_ref, timing_snapshot_ref,
  lifecycle_ref, role_rng_checkpoints[], runtime_timer_checkpoint,
  quota_checkpoint, intended_root_step_sequence, cut_hash
}
~~~

Native journal boundary is a sorted array of section, validated logical frame end, artifact sequence/hash and witness/barrier receipt. Preparation records precede the native barrier; final cuts refer to completed evidence, avoiding cyclic hashes. Root Step record stores recovery_cut_hash and auxiliary receipt before Ack; Ack returns those receipts. The actual root-step format is versioned, not silently reinterpreted as the old schema.

runtime_timer_checkpoint retains monotonically allocated timer IDs, logical owner/deadline and cancellation state. Reconnected timers receive new physical IDs above the preserved counter and original absolute deadlines; old-incarnation delivery is rejected. quota_checkpoint retains blocked members and each recipient/channel's conservative next-eligible local time and remaining allowance. Recovery never creates extra tokens: until an exact saved bucket can be reinstated, hold its saved debt and resume refill from its saved local timestamp; do not reset to a full burst.

## 5. Public opt-in API and one shared owner

The feature native-recovery adds Engine::new_with_recovery(context, config, options). Engine::new remains the upstream constructor with no recovery behavior. Public generic types use native Digest/PublicKey and Context/Round, not arbitrary serialized futures.

~~~rust
pub struct RecoveryOptions<D: Digest, P: PublicKey> {
    pub store: Arc<dyn RecoveryStore>,
    pub reconnector: Arc<dyn RecoveryReconnector<D, P>>,
    pub startup: Arc<dyn RecoveryStartup>,
    pub session_not_before: Option<SystemTime>,
}

pub trait RecoveryStore: Send + Sync + 'static {
    fn stage(&self, record: RecoveryRecord) -> Result<RecordToken, RecoveryError>;
    fn flush<'a>(&'a self, through: RecordToken)
        -> BoxFuture<'a, Result<RecoveryReceipt, RecoveryError>>;
    fn load<'a>(&'a self, request: RecoveryLoad)
        -> BoxFuture<'a, Result<RecoveryBundle, RecoveryError>>;
    fn checkpoint<'a>(&'a self, cut: RecoveryCut)
        -> BoxFuture<'a, Result<RecoveryReceipt, RecoveryError>>;
    fn claim_ingress(&self, actor: ActorId, channel: u32,
        source_key: Vec<u8>, exact_bytes: Vec<u8>)
        -> Result<InputRef, RecoveryError>;
}

pub trait RecoveryReconnector<D: Digest, P: PublicKey>: Send + Sync + 'static {
    fn reconnect<'a>(&'a self, request: ReconnectRequest<D, P>)
        -> BoxFuture<'a, Result<Reconnected<D>, RecoveryError>>;
}

pub trait RecoveryStartup: Send + Sync + 'static {
    fn ready(&self, actor: ActorId, cut: CutRef, state_hash: [u8; 32])
        -> Result<(), RecoveryError>;
    fn released<'a>(&'a self)
        -> BoxFuture<'a, Result<(), RecoveryError>>;
}
~~~

Reconnected<D> is a typed enum: Prepare(native oneshot Receiver<Handoff<D>>), Propose(Receiver<D>), Verify(Receiver<bool>), Certify(Receiver<bool>), Waiting, Closed or Cancelled. ReconnectRequest contains the original typed native Context/round, exact OperationRecord and separate reconnect generation. Returning an empty receiver whose sender is dropped is forbidden. Reconnector retains the producer/helper before returning promptly; modeled work remains behind the receiver.

There is ONE controller-owned auxiliary store shared by actors, App and root final flush. stage is nonblocking registration with a strictly monotone sequence; it does not claim durability. flush/checkpoint submit owned work into root custody. Queued/inflight/Ready auxiliary I/O and results never live solely in cancellable await locals. Existing full-job panic/failure rules apply. No registry lock across poll/drop/await. Actors and root use the same frontiers; private unrelated checkpoint files are not a substitute.

## 6. Live hooks and coherent commit order

For every grant:

1. Verify binding and persist GrantIntent before native input exposure.
2. NativeRx and batcher preserve exact consumed-input provenance. Record proposal selection/status changes separately from signature verification.
3. Stage CLAIMED before dispatching application work; return a real receiver promptly. Persist claims before any dependent native barrier or completed outer step.
4. Preserve work deadline and helper phase. Stage/flush OFFERED only after its required body/evidence durability and before native sender resolution. On actual actor consumption stage CONSUMED and the applied/discarded result. Stage cancellation/Waiting/Closed/latch facts at those transitions, including continue branches.
5. Before any native artifact append that can reach a durability barrier, flush a RecoveryPreparation covering the resulting native state, staged sign cache, inputs/operations and original timing. Retry-only publication also requires preparation even without a new Artifact append.
6. Native Storage/Witness completes its actual barrier and records the preparation reference before releasing successful I/O. A preparation is not itself a native durability witness.
7. At native pre-select/response boundaries publish the plain ActorPhase/TimingSnapshot. Root final flush requires all dirty actor/app frontiers checkpointed; it cannot call an unresolved callback completed merely to obtain a cut.
8. Root snapshots archive/RNG/timers/counters, commits RecoveryCut, then commits the exact root Step/Ack. Only then may Ack/output leave the worker.

Future construction and user work remain under granted control. Required aux/root persistence failure after native storage success still prevents successful result/Ack/publication. No native signing/private-state callback is invoked under IO_RELEASE or failure-cleanup permission.

## 7. Restore algorithm and startup fence

TrustedRecoveryFloor contains run/config/worker/cid, root-step receipt, auxiliary receipt/cut hash and witness receipt, supplied from the hub outside the worker file fault domain. Recover exact prefix receipts and valid descendants. An intrinsically valid older ledger/head replacement below the external floor is rejected.

Select the highest complete cut_sequence whose chain extends all floors and whose input/aux/body/native boundaries can be validated. A prepared descendant beyond the last Ack is eligible only when it belongs to a hub-issued outstanding grant and its required evidence is present. Never select an arbitrary lower cut to hide a mismatch. Missing/corrupt required state fails closed, but ordinary missing input for a correctly recorded pending operation remains pending.

Restore in this order:

1. Validate roots/receipts and preserve source files. Restore original local stop, terminal state and old lease end. Check the conservative expired-during-downtime rule. Clock recalibration never revives a terminated cid.
2. Perform accepted Witness copied-source/fresh-generation recovery with NEW barrier evidence; retain exact old authorizations and bytes. Reopen root/aux ledgers with anchored validation, not create_new/empty reset.
3. Restore archive/body references, original receipts, operation records, signer cache, counters, role RNG and timer identity state. Reverify exact bytes and reconstruct body persistence where required from bound bytes. Do not mark pending body data present.
4. Initialize all native actors behind the startup fence. Replay native journal authority. No normal actor loop, helper completion, new signature, resolver request or publication is enabled.
5. Reapply compatible auxiliary proposal facts in recorded selection order and at the recorded validation level using existing State handlers. A feature-gated RecoverableProposal carries its ingress reference; it does not mark pending signatures verified. Native certificates/votes/finality remain authoritative.
6. Reconnect operations and restore actor phases as §8 specifies. Native journal evidence may supersede old auxiliary requests; apply native cancellation/reconciliation policy and record the cancellation rather than overwriting native authority.
7. Restore timing AFTER proposal/operation facts determine timer eligibility. Check the reconstructed state projection. Restore original deadlines/anchors only; never replace native votes/locks or current/finalized view from the snapshot.
8. Each of voter, batcher, resolver, application operation registry, archive and root driver reports ready for the same cut. Required Clock calibration, witness/root/aux frontiers, signer cache and helper registrations must all be valid. Commit a StartupReleased record binding their readiness hashes before releasing the fence.

The main engine session is itself a controlled job, keeping actor siblings in one scope. No Scheme-only gate substitutes for actor fencing: native issuance flags can change before Scheme.sign. Replay/restore may stage historical observations but cannot publish before release. An unresolved body/operation can be ready for startup as explicitly pending; it must not block startup until a nonexistent result appears.

If native reconstruction plus bound auxiliary transitions cannot reproduce the required eligible state, report the specific incompatibility. Do not silently switch to fresh deadlines or discard a legitimate selected proposal. The positive no-Artifact pending-verification case in R1 is mandatory.

## 8. Operation and phase reconstruction

Use State::set_proposal -> try_verify -> verified and try_propose -> proposed, preserving native captured-parent checks. Never write requested_build/requested_verify/Verified flags directly. Native own-vote replay may already supply Verified; recognize it without a second application result or sign. Original fixture-validity evidence is not a claim that v10 Accept was implemented.

| Committed state | Required restoration |
| --- | --- |
| CLAIMED/WAIT_INPUT | Reconnect same operation with retained producer and fresh receiver. Keep exact missing dependencies/first receipt; no new generation or 5ms budget. |
| CLAIMED/WORK_SCHEDULED | Preserve original start/due. Restore one hub work completion under the new physical binding. |
| WORK_READY/BODY_PERSISTING | Restore helper continuation from plain result/body evidence. Finish required durability before offering native success. No opaque helper future is restored. |
| OFFERED, not consumed | Reconnect and offer exact stored outcome once in controlled order. Do not rerun build to produce a different digest. |
| CONSUMED | Reconstruct only the recorded applied transition with its bound evidence, or recognize native replay already supplied it. No redelivery. |
| CANCELLED | Restore cancellation; no producer reconnection or acceptance of late results. |
| WAITING | Retain exact captured parent/request. Once it certifies/finalizes, use native release/reconcile; view nullification/supersession still cancels normally. |
| CLOSED | Retain native forfeiture/timeout semantics. Do not turn it into Wait or retry. |

For in-flight variants, claimed native state must have a live reconnected receiver/producer. Waiting/Closed are deliberately receiverless actor phases with an identified captured request, not stray flags.

Already-due helper work gets an explicit RecoveryWorkCompleted event containing operation_id, original_due_tick_us, reconnect_generation and cut_hash. It is eligible at max(current tick, original due), only after startup release. Archive::complete must validate that authorization rather than requiring old due==new tick or assigning tick+5000. Ordinary completion still requires its originally scheduled coordinates. Native callback consumption retains the original first receipt.

## 9. Timing policy

Snapshots retain actual entered_at/proposed_at, leader/certification deadlines, latch time/reason/cause, nullify retry absolute/original anchor and supported stall fields. An existing retry cycle may not restore retry=None and arm from recovery-now.

Opt-in native checkpointing materializes the current Nullify retry deadline at the live arm event before durability/publication, recording retry_arm_local_us and retry_deadline=arm+configured retry. Repeated observation never changes it. A genuine subsequent nullify retry starts its next interval at that live event and checkpoints it. Native ordinary disabled-path behavior is unchanged.

Optional session_not_before is fixed in profile and defaults to None for Fresh-compatible runs. On EVERY genuinely new view entry:

~~~text
entered_at = actual NodeClock.current()
base = max(entered_at, session_not_before) if configured, else entered_at
leader_deadline = base + leader_timeout
certification_deadline = base + certification_timeout
~~~

Restored deadlines bypass this calculation. Latches/retries are never floored. Main profile has no stall deadline. No Clock.current impersonation, backward time, all-ready barrier or shifted Enter timestamp is allowed.

The existing 200/400ms normal timeout profile is retained for this physical-count fixture; the optional future-start test fixes its own start in config and no v10 slot inference follows. Later slot-aware timeouts must explicitly cover b_L2+2delta_G and declared work/data/network paths. That later BFT-5/6 mapping is not proved by this slice.

## 10. Resolver, transport and RNG continuity

Restore every RoleRng checkpoint using existing ChaCha20 seed/stream/word-position fields before actor reconstruction can draw randomness. No new seed or cursor zero. Persist the post-construction resolver checkpoint before releasing a fresh native request.

Outstanding native resolver requests are transient: record cancellation of old request generation, reconstruct a new native Fetcher request, and label its new ID/timer. This is not restoration of an old request timer and does not reset view/nullify timers. Preserve body-request IDs/retry deadlines separately; they are explicit Archive state, not native Fetcher state.

Hub cancels old-incarnation runtime timers/work deliveries and reissues restored logical obligations under a new physical binding. Reject stale timer/work/response association; retain certificate/body cryptographic validation regardless of request matching. Preserve packet-attempt counters, blocklist and quota debt; attempts do not become new logical messages.

Exact cached root Ack recovery uses a new RecoveredAck transport envelope containing current binding, original canonical Ack bytes, original grant hash, root receipt and cut hash. Do not modify/re-hash old logical Ack binding or execute the step again. The hub accepts it only for a known outstanding/committed logical step and deduplicates its exact transport attempts. An interrupted uncommitted step is identified explicitly; any reissued input/new step retains original authenticated receipt/operation identity and cannot reset timing.

Native recovery adds a versioned native-recovery-v1 wire/profile. Semantic added objects are TrustedRecoveryFloor, RecoverEngine, RecoveryWorkCompleted, RecoveryReady and RecoveredAck with the fields fixed here. Lead assigns protobuf numbers without changing these semantics. Fresh schema/profile and I1a remain independently replayable.

## 11. Failure cutpoints and modeled boundaries

All injected failures occur at named controlled boundaries; wall pauses alone never choose a protocol crash point. Add these hooks:

~~~text
after_grant_intent_sync_before_input
after_proposal_fact_sync_before_verify_claim
after_claim_sync_before_receiver_install
after_work_ready_before_body_sync
after_body_sync_before_offered
after_offered_sync_before_native_send
after_native_consume_before_preparation_sync
after_preparation_sync_before_native_barrier
after_native_barrier_before_final_cut
after_final_cut_sync_before_root_record
after_root_record_sync_before_root_head
after_root_head_before_ack
after_actor_restore_before_timing
after_timing_restore_before_startup_release
~~~

No Ack or publication escapes its required completed root/evidence commits. Missing required prepared state after an injected precommit crash may fail closed only when no authorized recoverable cut covers the durable native state; completed-Ack states must recover. Never treat this allowance as permission to omit R1/other legitimate pending-state persistence.

Root owns queued/inflight/Ready aux/native I/O and results. Controller/pump cancellation cannot destroy them. IO_RELEASE allows proxy wakes and owned-result disposal, not protocol construction/signing/publication. Ordinary jobs resume under later ordinary permits. Permanent failure revokes signing/publication and drains under restricted cleanup; no successful Ack follows. The accepted whole-native-job panic observers remain in force, including cancelled future and undeliverable result destructors.

## 12. Executable acceptance R1–R18

Every case records source/candidate/patch hashes, exact command, resolved profile, seed, fault checkpoint, trusted floors, raw files and expected event ordering. Run ordering-sensitive cases for seeds7/42/99 and wall pauses0/2ms. Real process-restart cases use a new worker PID/incarnation; in-process reconstruction alone is insufficient.

| ID | Fixture and trigger | Required assertions |
| --- | --- | --- |
| R1 | Commit Ack with elected leader proposal Unverified, verification waiting for body, and no voter Artifact; restart before lease/stop. | Successful restore of same proposal/operation/receipt with fresh live receiver; original certification deadline selected, not leader deadline. |
| R2 | R1 with native signature still pending, plus duplicate/conflicting ingress. | Opaque ingress reference/order preserved; no pending-to-verified relabel; original first receipt unchanged; normal native conflict behavior. |
| R3 | Commit pending build before its 5ms due; restart. | Same operation_id/original WorkBinding and captured parent; no work-generation increment; one completion at original due. |
| R4 | Crash after work-ready/before body sync and after body sync/before offered. | Helper state reconstructed; no native success before required durability; exact block/digest retained. |
| R5 | Crash after OFFERED persistence but before send/consumption, for propose and verify. | Exact stored outcome offered once to a retained receiver; no rebuild/reverification result substitution. |
| R6 | Restore CONSUMED result, both with and without a corresponding native Artifact. | Evidence-supported native transition restored; native journal remains authoritative; no double consumption or new signature from replay. |
| R7 | Cancel pending helper by view exit/Close, then deliver old WorkCompleted after restart. | Cancellation survives; stale completion discarded; no empty channel treated as successful recovery. |
| R8 | Already-due helper work at safe recovery, and future-due control. | Explicit authenticated RecoveryWorkCompleted at current tick for past due; original deadline recorded; future due unchanged, never recovery-now+5ms. |
| R9 | Restart before/at/after leader, certification and Nullify retry deadlines; explicit latch fixture. | Original absolutes/arm/latch cause persist; expired deadline immediately eligible after fence; no recovery-duration extension. |
| R10 | Staggered pre-window Enter with optional session floor; multiple new views before start. | Real entered_at differs per worker; every new normal deadline uses configured floor; restored absolutes, latches and retries unchanged; no all-ready barrier/timewarp. |
| R11 | Native barriers/root Ack across all named failure cuts. | Prepared/cut/root receipt relations valid; latest compatible cut extends trusted floors; no publication before required commits; exact cached Ack recovery. |
| R12 | Replace ledger/head with old consistent pair; remove/corrupt required operation/body/cut; wrong cid/config. | Anchored fail-closed without reset/truncation. Ordinary missing input in R1 remains a positive recovery case, not corruption. |
| R13 | Hold one actor/helper readiness after other actors restored; inject queued requests/results. | No new crypto/private-key invocation or publication before whole-engine StartupReleased; staged observations remain staged. |
| R14 | Restart after Close, at original local stop, or after old lease expiry. | Original terminal/stop/expiry policy retained; fresh sample cannot revive cid; archived existing data remains independently available. |
| R15 | Native vote durably created and authorized before lost Ack; retry/restart and Nullify cache path. | Restore exact signature/cache/outbox/attempt bytes; distinguish CREATED/CACHE_HIT; no primitive signing for old-output recovery. |
| R16 | Drop a real native certificate response, restart requester, then complete native resolver repair. | Old transient request cancelled; new native request ID/generation and persisted post-draw RNG; actual RESOLVER request/response consumed; view deadline unchanged; no BODY-only substitute. |
| R17 | Dedicated Waiting/Closed handoff-phase fixtures built through native handlers. | Waiting releases once on exact-parent certification; nullification/supersession cancels; Closed keeps forfeiture. Fixtures are explicitly Scheduled/rotating phase tests, never Stage and never relabeled as the main Dynamic profile. |
| R18 | Repeat real-process recovery matrices with seed/wall/start-order variants and malicious trace mutations. | Canonical logical cuts/deadlines/signatures/outcomes match per fixed seed; checker rejects premature offer/consume/sign/publication, false provenance and wrong deadline. Native resolver executed-repair evidence is present before overall I1b approval. |

Checker derives expected transitions from source/contract and reads raw input/WAL/body/native/witness/root records independently. It does not accept an implementation-produced valid=true as proof. Compare evidence, native final proof and validated delivery separately. Retain failure artifacts and classify infrastructure failure/premise failure without a fabricated liveness success.

## 13. Completion and limits

All semantic choices in this proposal are fixed; protobuf numbering, vendored source copy and generated code are lead integration work, not permission to invent alternate recovery semantics. Any guard/record/ordering change requires a versioned amendment. Register exact local patch hashes after implementation; no hash is invented here.

Completion requires R1–R18, relevant prior regressions and affected checks on the integrated revision, followed by same-revision simulation. Native resolver repair must actually execute. A compile check, empty recovery receiver, fresh deadline, fresh Archive, fabricated old barrier or in-memory restart is not a substitute.

The fixture still does not implement v10 node-local Accept/first-receive admission, slot windows, GFR or handoff semantics. Its validated bodies and historical proofs do not prove BFT-5/6. Do not proceed to 15-node/v10 claims by renaming this bounded gate. Preserve the Doc → Implement → independent Verify → Simulation handoff with contract/candidate/checks/artifacts/open findings.


## Lead wire numbering annex — frozen

Existing Fresh field numbers and bytes are retained; new fields are omitted in Fresh mode. native.proto adds RegisterRequest.recovery=5; WorkerFrame.recovered_ack=3; Input.recover/recovery_work/arm_fault=10/11/12; WorkKind.PREPARE=4; Effect.recovery_ready/recovery_cut=10/11; Ack.recovery_receipt=13. The added message definitions in proto/native.proto are authoritative. Scope profile is `native-recovery-v1`; new physical incarnation must match a hub-scripted restart and trusted floor. Fresh mode rejects recovery-only inputs/frames rather than silently enabling restart. Existing logical Ack hash naturally binds the optional recovery receipt; it still clears only its own hash and durable_step_hash. RecoveredAck carries exact old Ack bytes and is never substituted for a newly computed normal Ack. Receipt sequence/hash is a trusted minimum-prefix anchor, not necessarily the current head.

Control fault arming is root/scenario-only, named and occurrence-bounded. It is never exposed as a worker clock-advance API. Fault evidence is infrastructure diagnostics plus the durable checkpoint files and hub schedule; it cannot release protocol effects outside the Ack. The implementation handoff must document exact deterministic replay/scenario commands and report unfinished recovery as failure/inconclusive, never a Fresh fallback.

## Lead amendment 1 — authenticated empty native history

Frozen on 2026-11-15 after the independent source-first R1 review. A selected foreign proposal with pending Verify may have no voter Artifact and no voter partition directory: native journal startup can observe PartitionMissing. This is a positive recovery case, not evidence of corruption by itself.

The builder allowlist additionally includes prototype `src/witness_recovery.rs`, solely for an explicit validated empty-history recovery branch. Existing nonempty snapshot, CRC, frame, barrier, authorization and generation checks remain unchanged. No witness wire encoding changes are authorized.

The absent-partition branch requires all of the following before constructing or opening a target generation:

1. The original source storage root, witness ledger and protected head are present, regular as applicable, and verified against the external trusted minimum witness receipt and the original run/config/worker/cid/epoch/generation policy. A missing ledger/head/root is not an empty history.
2. The selected source partition is absent specifically with NotFound. Permission errors, non-directory objects, symlinks and malformed entries are errors. No source directory or blob is fabricated to satisfy the existing copier.
3. The complete validated witness history through the selected source head contains no Blob, Observation, Barrier, Witness, Eligible or Submission entry in any generation. Only policy records and correctly linked empty generation transitions are permitted. Current empty indexes alone are insufficient proof because earlier history may have contained protected bytes. This strict branch does not cover deletion or pruning; this fixture has no pruning.
4. The externally anchored compatible auxiliary cut and source manifest explicitly describe an empty voter file/Artifact/barrier/authorization set for this source generation. Required proposal, operation, first receipt, body and timing records are independently retained and verified. An empty native journal never means an empty application or auxiliary state.
5. The raw snapshot audit records the actual absence and empty source-file list. The target has a genuine old+1 generation transition linked to the old witness head. No old native barrier, Artifact, file or sync observation is invented. If there are no target native bytes to sync, the result contains no invented target barrier either; future native writes receive only their actual observed barriers.

R1 must execute this branch with a real new worker process and recover the pending Verify receiver and original certification deadline. R12 additionally deletes a partition that had a protected Blob/Artifact history, substitutes an empty manifest against a nonempty anchored cut, and removes the witness ledger/head; each is rejected. Repeat empty-to-empty generation recovery is allowed only while the complete historical predicate above remains true. Nonempty native recovery continues through the existing accepted raw-copy and fresh-sync path.

## Lead amendment 2 — prepared cache, recovered effects and clock continuity

Frozen on 2026-11-15 after independent source-first R11/R15 review. These rules refine existing records without changing protobuf numbering or native consensus rules.

**Prepared descendants.** A durable RecoveryPreparation can contain an actual signer invocation even when its prospective Artifact has not yet been appended and the authoritative native journal still matches the preceding complete cut. Recovery validates all compatible durable preparation records in the authorized grant suffix, including their bound input, sequence/hash lineage, exact subject/attestation, signer and namespace. Their signer-cache entries and invocation/counter high-water marks augment the selected complete cut before any signing is enabled. An exact later guarded retry uses the retained attestation and reports CACHE_HIT; no private-key invocation is repeated merely because the Artifact is absent. Restore this monotonic cache knowledge without copying prospective issuance/decision flags into native state. If native execution later legitimately requests that signature, it must still follow its ordinary append/barrier/publication path. An incompatible or conflicting prepared cache is an error, not a reason to discard it and sign again. Only an invocation lost before its required durability, with no published history, may be newly CREATED. The recovered report distinguishes the selected native cut from the adopted durable preparation frontier.

**Recovered Ack effect adoption.** The hub verifies original canonical Ack bytes, original grant identity, completed root receipt/cut and the new physical binding before any adoption. If the original logical Ack was already applied, RecoveredAck has zero new effects. For a completed worker Ack previously lost to the hub, adopt each original logical effect at most once using (worker, logical step, effect index); preserve exact original Send bytes, message ID, authorization and attempt. Existing logical Sign/Proof/Delivered/Lifecycle evidence is historical evidence, not a new crypto call or application transition. Recovered ClockRequest, TimerArm/TimerCancel and Work effects do not directly schedule old physical obligations. Record cancellation of the old incarnation, reconcile their logical obligations with the validated recovered cut, and authorize fresh clock exchange or rebound timer/work separately under the new incarnation. Rebound work retains its original operation ID/due and uses RecoveryWorkCompleted where required. Recovered lifecycle terminal facts apply before scheduling. Repeated RecoveredAck cannot mint packets, work, timers, clock samples or another root step. A stale outstanding request from the old resolver generation remains cancelled even if an old Send is durably known; the hub must record its cancellation instead of releasing it as an active fresh-generation request.

**Explicit clock checkpoint.** RecoveryCut additionally binds `clock_checkpoint` with run_id, worker_id, old_incarnation, clock_domain, rate_profile, last_issued_request_id, last_accepted_sample_sequence, last_observed_reference_tick_us, last_observed_local_us, old_lease_end_tick_us, clock_terminal_state, and outstanding_request_ids with their original incarnation bindings. Monotonic request/sample frontiers include compatible durable prepared descendants just as the other allocation counters do. A new incarnation retains these anti-replay frontiers and lifecycle/old-expiry constraints but starts with no usable calibration and no outstanding request eligible for an old response. Its first request ID is strictly greater than every previously issued/persisted request ID; its accepted sample must pass the existing identity, sequence, age, rate and half-open lease checks under the new incarnation. Do not reinterpret a cached old ClockRequest or ClockSample as new. Retain prior clock observations for continuity/expiry validation, never as a substitute for the new exchange. Overflow, rollback, wrong domain/profile or a terminal old clock/lifecycle state fails closed.

R11/R15 cover a synced prepared cache with absent Artifact, both before and after native persistence, lost versus already-applied Ack, repeated recovered envelopes, and no duplicate physical scheduling. R12/R14 cover clock checkpoint rollback, replayed old replies, request-ID reuse, fresh-incarnation calibration requirement, stop/old-lease equality and terminal retention. These cases remain actual process-restart obligations.
