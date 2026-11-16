# H3 Slice B — public stop, record handoff and bounded transport

2026-11-16. ID **H3-B-1**. **FROZEN by lead after independent pre-code review PASS, including exact parameter binding and owner-verified restore.** Authority/ownership: [implementation contract](h3-publication-gfr-implementation-v1.md). The single Builder proceeds after the named A verification/test handoff. This slice implements H3-owned pure plans, bounded storage projections and intents; actual H1 root/aux/port integration remains a local gate. No copied `healthy-session-api`, second Manager, direct network send, forged root acknowledgement or06 change.

S: v10 03 §§3.10,4.1 permanent closure,04 §§3.4–3.5/PUB-2–3 public-only stop and durable result obligations,09 §§3.2–3.4,4.1,4.7–4.8 public records/complete prefix. A: existing02 custody/route/service contracts,06 §4.3–4.4 intent/stop/receipt,07 POS/LIFE/PUBLIC and full-record atomicity. New dispatch decisions2–5 resolve Manager committed-vs-applied and monotonic termination; they supersede the preparation's unresolved safety-fence choice.

## 1. Trusted record capture and Manager handoff

Only capture a record through an existing sealed07 `ReplayState`, whose `public_state()` is read-only. Do not accept an arbitrary mutable `PublicReplayState`/ReplayRecord as authenticated public history. The source identities, config and expected Native profile must match the bridge's immutable binding. A record capture names its exact ordinal and checks position/events against the sealed state. **This establishes a complete algorithmic replay event, not H1 root durability.** RecordDelivery capture is pure immutable evidence/intent; external release and Manager delivery still require the actual H1 committed root/Ack. `from_replay` performs no persistence or delivery and cannot manufacture a runtime commit. A fixture/synthetic binding cannot be represented as genuine H1 evidence merely by calling this constructor; actual source provenance remains recorded.

```rust
pub struct RecordDelivery {
    /* private captured record, source binding, exact07 GfrParameters and identity */
}
impl RecordDelivery {
    pub fn from_replay(state: &poc_viewer_gfr::ReplayState, ordinal: usize)
        -> Result<Self, BridgeError>;
    pub fn id(&self) -> Hash32;
    pub fn record(&self) -> &poc_viewer_gfr::ReplayRecord;
    pub fn event_ids(&self) -> &[Hash32];
    pub fn parameters(&self) -> poc_viewer_gfr::GfrParameters;
}
pub struct MaterialReference {
    pub anchor_id: Hash32, pub object_hash: Hash32, pub reference: Vec<u8>,
}
```

The immutable record retains full input_ref/effects/events, exact GFR-POS-1 position and all GFR-LIFE-1 identities/projections in original order. `from_replay` also captures the exact07 `GfrParameters` from that sealed state, including announcement_us, grace_us and expiry_us; `parameters()` returns that retained value. These parameters are outside03 config_ref. Before any `plan_record` dedupe, scope derivation or state change, require exact equality with `StopLedgerBinding.parameters` in addition to config/source/profile equality. The eight-part record ID remains unchanged; delivery and ledger binding enforce this separate parameter equality, so an identical record ID does not bypass it. `MaterialReference` is only a bounded fetch/audit reference (≤4,096 bytes,≤32 refs per delivery), never a worker Ready flag/token or proof that execution material exists. Actual Plan/anchor/state inputs remain existing03/H1 objects. Same pc may contain close+assign and several regular handoffs; dedupe by full record/event identities, not position alone.

Private record identity uses existing03 `hash_parts`: SHA-256 of `u64BE(len(domain)) || UTF8(domain)`, then for **each separately listed part** `u64BE(len(part)) || part`. Never concatenate adjacent parts before this outer framing. Define `frame(b)=u32BE(len(b))||b`. Raw integer widths below have no embedded version; existing canonical encodings retain their own version bytes.

`H3/Record/v1` has exactly8 parts, in order: (1) `config.reference().encode()` using03 Canonical ConfigRef (34 bytes), (2) raw Espresso source bytes, (3) raw L1 source bytes, (4) raw u16BE Native protocol_version, (5) raw u16BE Native verifier_version, (6) raw UTF8 Native source_revision, (7) exact07 `encode_position(record.position)` bytes, (8) the complete private record projection defined next. Profile equality/validity is checked separately; the hash does not authenticate a revision string.

Private record projection is `u16BE(1) || frame(input_projection) || u32BE(effect_count) || frame(effect[0])... || u32BE(event_count) || frame(lifecycle_projection(event[0]))...`. Input projections: Espresso=`u8(0)||u64BE(q)`; SessionEnd=`u8(1)||frame(SessionId.encode_checked())`; Expire=`u8(2)||u64BE(height)`. Append effect=`u8(0)||u64BE(first)||u64BE(last)||u8(0 /* Regional source */)||frame(ResultKey.encode())`; Duty=`u8(1)||frame(tail_before)||frame(tail_after)`. Tail projections: Next=`u8(0)||frame(Descriptor.encode_checked())`; Pending=`u8(1)||frame(Boundary.encode_checked())`; Halt=`u8(2)||frame(Boundary.encode_checked())||frame(Anchor.encode_checked())`. SessionId/Descriptor/Boundary/Anchor use03 CanonicalObject; ResultKey uses03 Canonical. L1/Reset/Resumed are explicitly unsupported in this regular slice. Reuse exact07 LIFE projections; no new public position/lifecycle encoding. Every record field influences the identity. Max256 effects/events each and1MiB projection; overflow is explicit capacity error. Exact duplicates preserve obligations; same position/different record identity rejects.

Composition-only literal vector (not a valid native run): ConfigRef canonical bytes=`0001||11×32`, Espresso/L1 source=`45`/`4c`, protocol/verifier versions1/1, source_revision ASCII`2` repeated40, position=`00010000000000000064040000000000000005` (Espressoq5,key100), no effects/events, input Espressoq5. Projection=`0001000000090000000000000000050000000000000000`; record ID=`9c663cdf791586280d1c28ac39990b87c1504bd996e98fbbefcacda95bbb1809`. A verifier must derive additional nonempty-effect/lifecycle vectors independently; this one pins all eight part boundaries.

H2 consumes full legal records as `ManagerDelta`/action obligations; H1 is the sole worker durable root/aux owner. Dispatch semantics are fixed: after source/structure precheck, close/tombstone, Selected+Preparing, completed-record cursor and action intents commit together as **Committed(actions_pending)**. Material fetch/Prepare/Ready/Enter/Close application is later **ActionsApplied**. Missing successor execution material cannot defer an already known valid close; no Enter before actual H1 ReadyToken. After durable terminal/ClockUnsafe, later failure leaves terminal facts and pending actions intact, never rollback to Running. H3's GFR pc remains committed even if H2 action progress lags. Per-worker progress is independent; no all15 readiness barrier. H3 does not declare H2/H1 actual receipt types before their interface delivery.

## 2. Durable public-stop plans and pre-dispatch guard

H3 StopLedger stores config/source binding, accepted full record IDs/projections, complete session/branch/selected-cid exclusion scopes, verified deadline-watermark facts and pending06 stop projections. Bounds:4,096 records,4,096 scope entries,64MiB snapshot; no silent eviction. Single-owner plans are immutable; persistence is performed by the assigned H1 root/aux or service owner through an injected port. No new service database transaction protocol is introduced.

```rust
pub struct StopLedger { /* private monotonic committed state */ }
pub struct StopPlan { /* private instance/base revision/candidate bytes/digest */ }
pub struct StopLedgerBinding {
    pub config: poc_types::domain::PublicConfig,
    pub expected: poc_viewer_gfr::VerificationProfile,
    pub parameters: poc_viewer_gfr::GfrParameters,
    pub espresso_source: Vec<u8>, pub l1_source: Vec<u8>,
    pub publisher_reader: Vec<u8>, pub owner_reference: Vec<u8>,
}
pub trait DurableCommitPort {
    fn verify_commit(&self, candidate_hash: Hash32, receipt: &[u8])
        -> Result<(), BridgeError>;
}
impl StopLedger {
    pub fn new(binding: StopLedgerBinding) -> Result<Self, BridgeError>;
    pub fn restore(binding: StopLedgerBinding, snapshot: &[u8], receipt: &[u8],
        owner: &dyn DurableCommitPort) -> Result<Self, BridgeError>;
    pub fn plan_record(&self, record: &RecordDelivery) -> Result<StopPlan, BridgeError>;
    pub fn confirm(&mut self, plan: StopPlan, receipt: &[u8], owner: &dyn DurableCommitPort)
        -> Result<(), BridgeError>;
    pub fn public_stop(&self, result: &BridgeResult)
        -> Result<Option<poc_publisher::StopDecision<ResultKey>>, BridgeError>;
    pub fn permit_dispatch(&self, result: &BridgeResult)
        -> Result<bool, BridgeError>;
}
impl StopPlan {
    pub fn candidate_bytes(&self) -> &[u8];
    pub fn candidate_hash(&self) -> Hash32;
}
```

Constructor requires valid03 config, compatible Native profile, positive07 parameters, source/reader IDs of1..256 bytes and owner_reference of1..4,096 bytes; it creates only an empty uncommitted local ledger. The port is bound to the expected real owner/run and validates an actual owner-provided durability receipt covering exact snapshot/candidate bytes, identity and revision; a nonempty byte string or recomputed checksum is not a root receipt. Snapshot hash is `hash_parts("H3/StopSnapshot/v1", [exact_snapshot_bytes])` with exactly one part; the snapshot retains the complete checked binding, including all three exact07 GfrParameters and each retained delivery's parameter binding. **Restore requires this matching owner receipt through the port in addition to version/bounds/integrity/binding validation.** It compares snapshot/delivery parameters against the supplied StopLedgerBinding alongside config/source/profile, rejecting any mismatch even when03 config_ref and record ID match. Arbitrary caller bytes with a new hash cannot install public scopes. No default port or unchecked restore exists. Tests may use explicitly synthetic ports that recognize only registered committed candidate hashes; production durability PASS waits for actual owner integration. Cross-instance/stale plan/receipt mismatch rejects before state change. Corruption never resets to empty. These are pure local plans until a real owner receipt is verified; no second database or H1 root persistence implementation is created here.

Scopes derive solely from captured public events: SessionClosed matches all that session's results; SessionAssigned excludes other cids in that same session; BranchClosed (existing Expire in this slice) matches that branch. Retain scope even with no present task. Before `public_stop` or `permit_dispatch`, require the result plan's config_ref to match the ledger configuration, construct existing03 `ProgramConfig::from_plan`, and recompute `regional_cid(session, anchor.block.block_id, program)`; reject a CID mismatch before any projection or task mutation. BridgeResult proves bounds only, so its caller-supplied session/window cannot otherwise authorize a stop of an existing key. This cheap identity check does not replay signatures or claim native validity. Reject other-session overreach, conflict and caller-invented scope. Program local TIME/Pause/ClockUnsafe is not public exclusion. Watermark stop uses actual06 VerifiedPrefix with matching source/reader and the actual Native Publisher task's verified deadline; theta≥deadline (including equality) stops. No caller-invented deadline, header-only or wall timer substitute.

The verified task deadline must also equal the existing07 deadline formula evaluated using this ledger's configuration/parameters and that checked descriptor window; a task from an adapter with a shorter grace cannot authorize an earlier stop. This is the same exact-parameter binding applied to records, not a new timeout rule.

Serialize Offer→scope check→06 apply_public_stop→prepare into one controlled actor turn, rechecking before any undispatched intent release. Late new keys inherit prior scopes. A preexisting prepared-but-undispatched attempt stops; already committed in-flight submissions remain historical. Stop cannot erase results/attempts; later verified Recorded is permitted but does not reopen retry/G eligibility. Persist ledger scope before acknowledging its record as accepted; if06 task update then fails, ledger dispatch guard still fails closed and retries the pending task update. No rollback of durable exclusion on later failure.

## 3. Bounded service custody and split-publication schedule

Consume06 immutable `Submission< ResultKey >` / AttemptId and exact messages, not a new task ID. Service custody binds source/run/config, AttemptId and full ordered payload. Admit at most8 pending A-profile attempts and at most1MiB retained payload+metadata; identical attempt retry is idempotent, same identity/different bytes rejects. No full Offer bypass. A partial H1 proof observation never enters the mempool.

Store each message's original ordinal and state pending/proposed-at(q,index)/finalized witness. A controlled proposal plan may choose any explicit bounded subset/order of an accepted attempt's existing messages (e.g. proof atq1, bodies atq2); validate proposed list with06 bounds. Record this schedule as input. All original messages remain one immutable attempt, including duplicates. Do not invent extra attempts to span q, silently omit body obligations, or recompute payloads. Replayed proposal intent is idempotent by exact attempt/ordinal and proposal-intent identity. Public repeated publication, when intentionally scheduled, has an explicit new service proposal occurrence while preserving the original message identity.

Pure service plans return proposed06 operations and candidate custody bytes; actual SimEspresso propose/finalize and emitted messages execute through existing02 ServiceGrant/root controlled output machinery. A service receipt proves only actual durable custody; root Ack proves controlled output commit; neither proves finalized publication. Delivery and finalized reader prefix are separate controlled inputs. No caller can mark Recorded using a custody ACK. H1 supplies real ports/endpoints/channel admission later; this does not block pure mempool/plan tests.

## 4. Transport-only frame and controlled credits

Reuse02 Message/Envelope/Delivery identities and route/mailbox capacity/custody. Never embed fragments as Espresso/GFR public objects. Object kinds are SubmissionMessage0 and FinalizedLogBlock1; the first carries one existing LogMessage's exact payload, with attempt/ordinal/codec manifest bound by `binding_id`; the second carries exact06 `encode_finalized` bytes. No fabricated proposed/finalized tick for an unproposed submission.

**H3-TRANSPORT-1** frame: ASCII `H3TF`,u16BE1,kind:u8,binding_id:32 raw bytes,object_digest:32 raw bytes,total_len:u32BE,chunk_index:u16BE,chunk_count:u16BE,chunk_len:u32BE,chunk bytes. Header83 bytes. `object_digest=hash_parts("H3/TransportObject/v1",[one raw kind byte,raw32 binding_id,exact object bytes])`, exactly3 separately framed parts. The complete object must be nonempty; zero total_len, zero chunk_count or empty chunk is rejected. Both valid GFR message payloads and06 encoded finalized blocks are nonempty, including an Espresso block with zero messages.

Transport binding hashes use the same outer u64BE hash_parts framing. `run` and `config` below are02 Id values as raw32 bytes, not03 ConfigRef encodings; all listed u32/u64 fields are raw BE integers. `source_id`/namespace are raw bytes, with their normal checked bounds.

- `attempt_payload_id = hash_parts("H3/AttemptPayload/v1", [source_id, u32BE(message_count), u32BE(0), u32BE(codec[0]), payload[0], ..., u32BE(n−1), u32BE(codec[n−1]), payload[n−1]])`. There are exactly`2+3n` parts. Messages are the complete immutable06 Submission list in original order; no hashing of fragments or selected subsets.
- SubmissionMessage binding_id uses domain`H3/SubmissionBinding/v1` and exactly10 parts: `[run, config, source_id, AttemptId.namespace, u64BE(task_number), u64BE(attempt_number), ResultKey.encode(), raw32(attempt_payload_id), u32BE(message_ordinal), u32BE(message.codec)]`. ResultKey is existing03 canonical66 bytes. Receiver checks the bound complete-attempt manifest and exact ordinal/codec before release; no new identity replaces06 AttemptId.
- FinalizedLogBlock binding_id uses domain`H3/FinalizedBlockBinding/v1` and exactly4 parts: `[run, config, source_id, u64BE(q)]`. On completion require exact06 decode to the same source/q. Same binding with another object_digest/length is conflict, not another reassembly slot. Destination-specific delivery/incarnation remains in the existing02 reassembly key; this immutable block binding need not differ for each reader.

Composition-only hash vector: run=`01×32`,config=`02×32`,source=`45`,AttemptId.namespace=`50`,task1,attempt2,ResultKey canonical=`0001||33×32||44×32`,one message ordinal0/codec`0x47465201`/payload ASCII`abc` (synthetic, not a valid GFR publication). attempt_payload_id=`4d0c1b07e6b5f149b1cb4b56f5a1026b6c4f5a08a839125ec94ddd7057ac0e7a`; Submission binding_id=`f7ae0c2f6af604cb1fab9791814380435866804cc0d56eb40325e67ed5ea4e14`; Submission object_digest=`d06849efe5c03e9b648a4dc16324a49b06b56e55c72b82d756d16980676b334c`. Same run/config/source with finalizedq5 gives binding_id=`8568b9f102344973e7d3b4fe649003eff7c888066654c3e7757885bd29c5915f`. These pin widths/part boundaries and imply no native signature or block validity.

```rust
pub fn fragment(kind: u8, binding_id: Hash32, object: &[u8], payload_limit: usize)
    -> Result<Vec<Vec<u8>>, BridgeError>;
pub struct Reassembler { /* private run/config/endpoints and bounded partials */ }
pub struct Reassembled { pub kind: u8, pub binding_id: Hash32, pub bytes: Vec<u8> }
impl Reassembler {
    pub fn receive(&mut self, delivery: &sim_transport::Delivery)
        -> Result<Option<Reassembled>, BridgeError>;
}
```

Bind reassembly to actual validated02 run/config/source/destination/incarnations/channel/context/codec plus kind/binding/digest. Each input must originate from actual controlled route/custody delivery; public struct construction alone is not production delivery evidence. H1 owns channel registration, context admission, root grants and authentication; H3 must not impersonate them. Small-profile object≤1MiB; data chunk=`min(61,440, actual_payload_limit−83)` with checked subtraction and nonzero result. M1 payload_limit65,536 gives≤18 chunks for1MiB; final chunk4,096. Lower configured payload limits are supported only when resulting count≤1,024; otherwise explicit capacity failure. Check exact count/offset/final length before allocation.

Two incomplete objects/2MiB declared storage per destination; globally at most4 incomplete objects/4MiB declared storage per Reassembler, at most1,024 frame slots per object, and4,096 retained completed-object identities. Frame-slot metadata is additionally bounded by these counts before allocation. Duplicate identical chunks idempotent, conflict/wrong digest/identity rejects, incomplete object never yields a public message/prefix. Retain completed identities to prevent replay delivery; exhausting their capacity gives explicit backpressure, not eviction of unconsumed objects. Reassembled06 wire still passes exact decoder/source checks before reader ingestion. A source hole remains WAIT; reassembly arrival cannot change original `(q,index)` or q_avail.

For M1 schedule at most4 frames or262,144 total in-flight payload bytes per H3 sender, to at most2 recipients per grant, subject to **actual**02 global/link/custody credits and copy overhead. Credits count frame headers: one maximal chunk frame is61,523 bytes; four are246,092 bytes; all18 frames of a1MiB object total1,050,070 bytes before any duplicate link copies.02 default4MiB queue is shared with other traffic; never presume it is reserved for H3. Plans reserve/release credit only on corresponding actual owner outcomes; capacity rejection retains durable unsent obligations and retries under later grant, without rebuilding identity. No uncontrolled15-recipient fanout. Delivery acknowledgements cannot be invented to free credit. Per-reader public state and queues remain independent.

Exact H1 endpoint/channel IDs, port receipts and constructor bindings are a local integration gate; freeze them in a small addendum when delivered. The pure fragmentation/reassembly/credit decisions above can be independently implemented and tested now. A public object larger than06 bound remains invalid even if transport can fragment it.

## 5. Focused independent cases

| ID | Trigger / guard | Expected |
| --- | --- | --- |
| B01 | Capture genuine sealed replay record, same-pc distinct events, forged mutable record | Full ordered IDs/material refs retained; mutable/foreign source does not authenticate; changing any semantic record field changes identity. |
| B02 | Exact record replay, same-position conflict, stale plan, wrong owner receipt/corrupt snapshot; forged scopes with recomputed checksum | Idempotent exact replay; conflict/mismatch/unauthenticated restore rejects; sealed replay capture alone is not H1 root commit or Manager delivery. |
| B02p | Valid sealed replay with identical03 config_ref/source/profile but different announcement/grace/expiry; shorter grace/expiry stop record or restored snapshot | `plan_record` and restore reject before dedupe/scope mutation, including equal record IDs. Parameters outside03 config_ref cannot authorize another configured ledger's stop. |
| B03 | Public close before any Offer; late result; stop between prepare and dispatch | Durable scope survives absent keys; late/undispatched output blocked; existing in-flight facts preserved. |
| B04 | Local TIME/Pause versus public close/watermark equality; unrelated session assignment | Local state alone preserves publication; only correctly scoped public evidence stops; no scope overreach. |
| B04p | Bounds-valid result keeps a legitimate cid/body/key but changes session/window/anchor to a stopped scope | Existing03 config/CID reconstruction rejects before StopDecision or mutation; no forged descriptor can stop the unrelated legitimate task. |
| B05 | Durable close plus successor material WAIT; later action failure/ClockUnsafe | H2 committed obligations and H1 terminal remain; no signing/Enter rollback; ActionsApplied stays pending. Actual H1/H2 port case is gated locally. |
| B06 | Full attempt proofq1/bodyq2, duplicate attempt and conflicting attempt payload | One immutable attempt/custody set; genuine public incompleteness atq1; full prefix atq2 permits receipt; conflict rejected. |
| B07 | Literal record/binding hash vectors; object length0/61,440/61,441/1MiB/+1, reduced payload bound, duplicate/conflicting/reversed chunks | Exact hash parts/segmentation; zero-length/count/chunk rejects; no partial public delivery; same original identities/bytes. |
| B08 | Fifteen recipients, shared4MiB saturation, unrelated traffic, missing ACK | Controlled recipient/frame batches and real credits; pending obligations retained; no bypass or fake public absence/free credit. |
| B09 | Reader A/B distinct message/chunk arrival, earlier q hole, duplicate complete object | Independent prefixes/WAIT; once complete same public state; no shared mutable progress, q rewrite or repeated downstream delivery. |

No crash-recovery expansion is implied by durable intent/storage boundaries. Actual production receipt integration, H1 runtime and M1 end-to-end PASS remain separate, explicitly reported gates; core B implementation proceeds after its local review/freeze.
