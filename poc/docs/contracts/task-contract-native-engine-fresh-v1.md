# NATIVE-ENGINE/FRESH v1 — frozen vertical integration

Authority: v2 §§3–6/13, full I1b contract, and Commonware2026.10.0 plus the accepted opt-in runtime patch. Use accepted Clock/Storage/Witness foundations. This slice runs TWO actual OS native workers and genuine Simplex engines, but does NOT claim same-cid process recovery, v10 windows/Accept/GFR or full I1b. Native recovery/timing plus auxiliary state remains a mandatory following gate.

## Ownership / scope

Lead owns proto/native.proto, native_wire_support.rs, manifests/locks and integration. Builder owns new native_{config,scheme,elector,network,application,replica,hub,step_journal}.rs modules, bin/native-{worker,hub}.rs, builder tests, and poc/docs/native-engine-fresh-implementation.md. Existing runtime src/{driver,context,clock,lib}.rs may gain explicit authorities/audit/output hooks; no guard weakening. Witness hooks may gain read-only access to committed IDs/receipts and actor staging of the existing authorization API. No vendor, independent-test or I1a code edits. New dependencies/features require lead approval.

Actual Engine generic context is SimContext; actual native Storage/Blob, Spawner/Handle, Clock/governor, RNG and Metrics must be used. Native application/body path is private per worker, not a shared-memory relay between workers. Root-owned controlled data/archive service retains old proofs/bodies after consensus Close; it may verify historical proof without a current BFT admission. No new signatures from closed/unsafe cid.

## Fixed registry / configuration

Five equal-weight members0–4, distinct verified identity/BLS keys/PoPs. Real worker0/member0 and worker1/member1; members2–4 are explicitly scripted endpoints. Setup may generate deterministic test keys, but running scripted signer service loads/uses ONLY keys2–4. It cannot produce worker0/1 signatures; checker requires their actual worker SignAudit/journal evidence. Scripted peers never Register or receive worker clock leases and are not counted as live replicas.

Reuse Witness public-registry framing and context-bound voting namespace. Native signer index is sorted BiMap identity-key position, independent from logical member. Custom elector selects logical (view−1)%5 and maps that member to native index. Use rotating terms (one view/term), Dynamic mode ignoring certificate contents for the deterministic formula, prepare=Wait, no skip/forward/optimistic stable-term optimization. Keep native quorum4.

Canonical resolved base profile contains seed, public registry hash, code/profile versions, epoch, fixture/transport/work/timeout settings and scripted behavior; excludes paths, PIDs, wall pauses and process-start order. Derive in order (H is existing u64BE length framing):

- config_hash=H(`I1b/fresh-config/v1`,[canonical base-profile JSON,registry_hash]);
- run_id=H(`I1b/fresh-run/v1`,[config_hash]);
- genesis_digest=H(`I1b/fresh-genesis/v1`,[config_hash]); trusted external fixture anchor at height0;
- cid=H(`I1b/fresh-cid/v1`,[run_id,genesis_digest,epoch u64BE]);
- native Scheme namespace=H(`I1b/voter-namespace/v1`,[run_id,config_hash,cid,epoch u64BE]).

After namespace derivation, native_config_hash=H(`I1b/fresh-native-config/v1`,[config_hash,registry_hash,namespace_hash]); app_fixture_hash=H(`I1b/fresh-app/v1`,[cid,genesis_digest,height_limit u64BE,payload_size u32BE=256,work_duration u64BE=5000]). These derived values are not fed back into the base profile. No cyclic hash inputs. Namespace per-kind native subjects unchanged. Clock domains/rate references reuse the existing Profile methods (I1 clock-domain/rate domains).

## Numeric profile (virtual, not measured performance)

Reference start1,000,000us; epoch1; worker offsets+1000/−1000us. Lease100ms, refresh20ms; request/response each250us. Native link latency20ms, zero jitter baseline; per-directed-link/channel queue64, drop-newest overflow. Recipient/channel quota64messages/s, burst64; explicit stress variants may use unlimited quota to isolate router overflow. Receiver/mailbox capacity64. gRPC1MiB, envelope payload64KiB. Build/verify/certify each5ms modeled WorkStarted/WorkCompleted. Native synchronous crypto virtual cost0 explicitly; wall cost remains diagnostic. Leader200ms, certification400ms, nullify retry100ms, resolver timeout100ms/retry20ms. Local stop1,800,000us; cutoff1,900,000us. Seeds7/42/99; diagnostics0/2ms wall pause and reversed process start.

Real process registration may finish before scheduling, but program/calibration/start are independent: clock kick at start for worker0 and start+20ms for worker1; start engine only after its own calibration and material readiness. No all-Ready barrier. Ledger/actor IDs never depend on PID/arrival order. Native selected journal sections are explicitly1–8, retention16/no prune; this extends the earlier witness FIXTURE range only, not its coverage rules. Unexpected view exhaustion is incomplete/failure, never silently dropped.

## Fixture application and archive

Maximum scenario height1 or2. This is physical-count fixture height, NOT v10 slot/window semantics. Canonical block bytes: version u8=1, cid32, height u64BE, parent_digest32, payload_length u32BE, payload. Payload exactly256 bytes, byte i=(height+i) mod256. Digest=H(`I1b/fixture-block/v1`,[canonical bytes]). Height must be parent.height+1; complete ancestry reaches configured genesis. No genesis body is needed: it is the trusted input anchor.

Propose/verify/certify issue stable logical work requests. Missing body/ancestry stays pending; computation starts only when required inputs are available. WorkCompleted is delivered by Hub after5ms; never sleep wall time for protocol work. Persist body/state before returning proposal/certify success. Established deterministic invalidity returns false; missing data is never false. Cancel obsolete/closed operations and discard late completion. Certify must independently require full valid body/ancestry, even if native verify was never called.

Full-body protobuf BodyPush/Request/Reply lives in BODY envelopes. Preserve request IDs across retries; deduplicate received content. Body channels and native control channels can be independently delayed. A cryptographically verified finalization is ProofObserved; delivery occurs only after a contiguous validated body prefix. Each block delivers once. Archive and publication of already-existing evidence may continue after consensus Close; never invoke Scheme.sign for archived replay.

## GuardedScheme and outputs

Use actual native BLS multisig scheme with private key only for the real worker's member. At EVERY Scheme.sign call, check actual ordinary controlled authority, node clock sample/lease, original local stop, cid/epoch and permanent failure/termination. Guard precedes cache hit as well as primitive signing. Cache exact canonical kind+round+subject attestations; a conflicting same-kind/same-round subject fails closed. Preserve native cross-kind rules. CREATED means one actual underlying crypto invocation; CACHE_HIT reuses bytes; DENIED creates none. Public proof verification never requires live clock.

Record each actual Clock::current/governor read and each signing decision with reference tick, local time, sample sequence/lease and grant/task identity. Same node modules use the existing NodeClock. No grant calibration shortcut.

Native VOTE and direct CERTIFICATE channel outputs require exact committed witness/authorization. Existing Witness Submission is a stable route authorization, NOT a transport attempt. Each actual Sender call gets a separate checked attempt/effect identity persisted in the root Step journal; repeated Ack does not repeat attempts. Native Nullify retransmission may reuse authorization/signature while routing another attempt.

RESOLVER forwarding verifies actual native certificate/context and preserves native servability; requests/errors are not votes. BODY/relay is content-bound, not a fictitious vote-witness path. All output is staged under a grant and released only in the durable Ack. No direct worker-to-worker socket or background protocol output. All expands known committee excluding self; unknown peers reject, duplicates do not count twice. Accepted feedback is local staging only, not remote delivery. Scripted provenance is forbidden in worker outputs.

## Transport / journals

Exact numbered schema is proto/native.proto; helpers native_wire_support.rs freeze hashes. Standard protobuf merge/unknown-field semantics match I1a; semantic checks reject absent required oneofs, unknown enums/schema and malformed32-byte IDs. Register/stream hello bind run/config/worker/member/incarnation, registry, namespace and fresh profile. Incarnation1 only in this slice; disconnect/unplanned loss is infrastructure failure, no hidden restart.

Hub owns ordered (tick,causal_wave,tie_rank,event_id) queue and one outstanding global Grant. Every grant can produce native sends. Queue ordering and derived IDs depend only on committed effects/config/seed. tie_rank=H(`I1b/fresh-tie/v1`,[seed u64BE,event_id u64BE]); same-tick derived events use parent.causal_wave+1. TimerFired uses original absolute deadline; grants never extend clock lease. WorkCompleted and native network delivery are explicit events. Clock service samples at its actual virtual service tick, then separately delivers reply.

Ack has full canonical runtime-events JSON plus application-events JSON, and typed actionable effects. Audit JSON is exact ordered typed serialization, not a filtered digest. Logical Ack hash excludes itself and durable_step_hash, avoiding circularity; witness receipt and durable step sequence remain included. Root per-worker step ledger stores the exact grant, logical Ack, sequence and prior hash BEFORE reply, with protected length/complement/checksum+committed-head semantics from I1a under distinct `I1b/root-step-record/v1` and `I1b/root-step-head/v1` domains (head magic I1SHEAD1). Set Ack.durable_step_hash to the completed record hash afterward. This proves output persistence, not reconstructible opaque native state. Exact cached duplicate grant/Ack is idempotent; altered/unrequested frames fail closed.

## Scripted peer policy and cases

Scripts may sign only2–4. They retain deterministic per-kind/view subjects, receive bodies/control through Hub routing, and sign only the declared valid fixture proposal and context. They may assemble certificates using genuine received worker votes. Record each script action and actual native bytes as SCRIPTED, never a worker journal or live quorum.

Fresh cases: healthy(height2), two-real-alone(scripts disabled; noquorum), proof-before-body(height1, hold all BODY destinedworker1 until1,250,000us), close-propose/close-verify/close-certify (close selectedworker2,500us after first matching WorkStarted, before5ms completion), lease-loss(drop renewals after1,050,000us), wrong-context/native-proof negatives, quota and router overflow controls. Freeze resolved schedules in each output. Healthy can close after both deliver limit; negative cases run to declared cutoff. No fabricated guarantee for scenarios outside readiness/participation bounds.

Checker independently decodes raw native Vote/Certificate, PoP/index/namespace/subject/quorum, body bytes/hash/ancestry, every sign clock/permit, witness and root-journal bindings, origin classification, routing/feedback and exact trace replay. Required negatives:3vs4distinctsigners, duplicates, wrongkind/context/epoch/leader/proof, malformed bytes, false witness refs, proof-only delivery, unsolicited/altered Ack, output without grant, expired/closed signing. Save publicgenesis, exact fullconfig/keys-map, native proof/body bytes, scripts, clock/link/work schedule, process PIDs, journals and failure outputs. Keep verification and simulator output separate.

Completion requires two real native processes and same-revision independent checks/replay. This slice is fresh-run integration ONLY. Same-cid crash recovery with auxiliary callback state and original voter deadlines, guarded recovery history and v10 admission remain open; do not advance to15-node/handoff claims by renaming this partial gate overallI1b.
