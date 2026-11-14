# NATIVE-WITNESS v1 — frozen bounded durability/outbox gate

Authority: PoC v2 §§4–5 and13, task-contract-i1b-v1, and pinned Commonware2026.10.0/upstream92a2f68a0e131c8b69de41e84aec37da6280f212. NATIVE-STORAGE primitive behavior is independently accepted (implementation4f889af, integrated5073e3a; diagnostic temp-allocation fix6c7465f). This contract does not claim full I1b, native live consensus or v10 admission/liveness.

## Scope / ownership

One builder owns prototype src/{witness.rs,witness_ledger.rs,witness_recovery.rs,storage.rs,driver.rs,lib.rs}, builder tests/support, and poc/docs/native-witness-implementation.md. Lead owns dependencies, configuration/hash/wire choices and integration. No vendor, independent-test, I1a or manifest/lock changes without explicit lead review.

Implement a bounded selected-journal witness policy, durable eligible output records and copied-snapshot recovery. Use existing actual native Blob operations and root-owned completion custody. Do not yet implement engines, GuardedScheme, resolver/body transport or RPC. Preserve all68 existing tests. No additional dependency is required: native Artifact/Vote/Certificate codecs, Crc32, serde and SHA-256 are pinned already.

## Frozen profile

One real worker's selected voter partition, one cid, epoch1, native sections/views1–2, no pruning. Use the real five-key/PoP registry and PoP namespace from NATIVE-STORAGE, with explicit member-to-signer-index mapping and verifier-only policy construction. Witness fixtures use the explicitly context-bound voting namespace below; create fresh signatures under it, never reinterpret the earlier storage-only fixture signatures. Physical page256/logical244; compressionNone; native Blob default layoutV1, application version0. Maximum selected blob data64KiB, native frame payload4096B, decoded frames1024 per blob, ledger frame16MiB. These are explicit fixture bounds, not changes to general Storage semantics.

The policy fixes run_id/config_hash/cid (32 bytes), worker_id/member_id u32, epoch/generation u64, selected logical partition/name set, registry and namespace hashes. Host paths, PIDs and wall pauses are diagnostics and excluded from identities. Generation is a physical recovery identity, not a new signing context. Initial policy registration is before the first grant in this slice; multi-cid dynamic registration remains a later extension.

H(domain,parts) uses the existing framing: SHA256 of domain UTF-8 and each component, each preceded by u64BE byte length. Integer identity components are BE at the declared width. Hash domains:

- `I1b/blob/v1`: run, config, worker(u32), generation(u64), partitionUTF8, raw name, open_id(u64).
- `I1b/voter-namespace/v1`: run, config, cid, epoch(u64); the 32-byte result is the native Scheme voting namespace. Config binds the declared chain/genesis/program profile. This is an explicit PoC namespace choice; native per-kind signed subjects are unchanged.
- `I1b/barrier-bytes/v1`: exact covered range bytes for range-SYNC, or exact complete image bytes for full sync. This supplies the covered-image/value hash below.
- `I1b/native-artifact/v1`: exact canonical native Artifact bytes.
- `I1b/native-wire/v1`: channel(u32; VOTE=1,CERTIFICATE=2), canonical native wire bytes.
- `I1b/native-barrier/v1`: blob_id, io_id(u64), kind(u8:range1/full2), offset(u64), length(u64), covered-image/value hash.
- `I1b/artifact-witness/v1`: run, config, cid, epoch(u64), blob_id, section(u64), logical frame offset(u64), artifact_hash, lexicographically sorted unique 32-byte barrier IDs concatenated (no further separator).
- `I1b/outbox-payload/v1`: run, config, cid, epoch(u64), source member(u32), channel(u32), wire_hash. This is an eligible payload, not a delivery or submission.
- `I1b/outbox/v1`: eligible_payload_id, destination member(u32). Stable across retry and recovery; no incarnation, attempt or resend time in the identity.

Canonical metadata uses ordered serde structs, sorted maps/ranges and bounded integers; no floats/unordered hashing. Before verification freeze exact struct fields in the implementation handoff and supply literal vectors. No implicit change to native signature domains.

## Root completion/evidence custody

For selected partitions, retain exact submitted write bytes (`IoBufs::clone` is available), operation options/ranges and same-open NativeBlob reference. Root owns all queued/inflight/Ready results as in NATIVE-STORAGE. Between IoReady and IoReleased, run any required snapshot validation and ledger persistence in a SECOND root-owned evidence slot. A cancellable grant/controller only borrows it. Neither completed native Result nor evidence Future may live solely in an await-local.

Use root/infrastructure wakers for physical work. No registry lock across await/poll/drop. Evidence I/O is dedicated root filesystem work, never recursively submitted through the SimBlob whose successful completion it is acknowledging. IO_RELEASE still cannot authorize new ordinary constructors, signing or publication. Dispose abandoned results under release/cleanup before the next operation.

A failure of this required witness/evidence ledger after native success is infrastructure failure: no successful native-result release, Step/Ack or submission follows. Native method Err itself remains a typed result per Clarification1; it produces no successful barrier and marks the attempted selected ranges/extent uncertain. V1 requires validated fresh-generation recovery before clearing uncertainty. Preserve already sealed history; never invent a witness from an Err or from mere readable bytes.

## Byte/version and barrier model

Track current observed bytes, checked extent and known durable byte versions per open instance. Initial readable bytes are not evidence of a particular historical barrier. Ordinary successful writes/resize update visible state, not durability. Empty writes do not grow extent or certify unrelated data. A successful SYNC write covers ONLY the exact submitted range/values. Full sync and completed start_sync cover preceding established mutations on THAT blob, including known extent. An unrelated blob's barrier is never usable.

Any later overwrite invalidates the corresponding current coverage unless its equality/preservation is explicitly established; sealed historical witnesses remain immutable. Failed attempted mutation makes its possible overlap/extension uncertain. V1 may conservatively refuse sealing until explicit recovery rather than guess unknown extent or partial-write results. Bounded readback, when needed, reads the SAME open native Blob and known extent; never reopen a live handle or run repairing Journal replay to verify it.

Before destructive mutation of protected history, reject/fail closed: no remove, shrinking below a protected frame's physical page, or changing a sealed frame's logical varint/payload bytes in this no-prune profile. Footer updates and append padding may change according to the actual native writer. Verify protected artifacts remain decodable after a successful selected mutation; unexpected corruption cannot become a successful evidence claim. General nonselected Storage behavior is unchanged.

## Exact page/frame/Artifact decoder

A physical page is244 payload bytes plus two footer slots, each `u16BE len || u32BE CRC32C`. Whole partial-page rewrites are256 bytes. Length0 is NEVER valid. A slot is valid only if 0<len<=244 and public Crc32::checksum(payload[..len]) matches. Choose the valid larger length; slot0 wins a tie; fall back to the other valid slot. All pages except the last must be full. Bytes beyond the selected logical prefix are ignored. Preserve physical/logical offset distinction; Blob offsets exclude the container header.

Concatenate selected logical prefixes and parse bounded canonical commonware_codec UInt<u32> length + exact Artifact<S,D> bytes. Require complete framing, canonical decode/re-encode equality, epoch and section/view binding, valid configured signatures/certificates where present, bounded signer/bitmap parsing and no trailing item bytes. Invalid complete pages, oversized frames, wrong scope and noncanonical encodings fail closed in V1. Ordinary buffering may leave an incomplete UNSEALED logical tail: protected-history checks must accept that tail while checking every protected frame. Barrier processing may seal complete prefix frames only, with full selected-prefix/slot/extent coverage; the incomplete frame itself never becomes eligible. Strict snapshot recovery still rejects incomplete source framing in V1, preserving it for diagnosis. No silent source truncation/repair. These rejected cases are not promised automatic recovery.

An artifact witness requires known durable coverage for the ENTIRE selected CRC-authenticated prefix [0,selected_length) of every involved page (not merely the target frame), the selected six-byte CRC slot, and a durable extent lower bound through that complete 256-byte physical page. Evidence contains immutable exact artifact bytes, page bytes/selected slots, logical/physical mapping and actual barrier references. Substring matching or a global last-sync counter is not evidence.

Decode native Artifact and construct native Vote or Certificate before encoding wire bytes. Artifact tags0/1/2/3/4/5/6 are Notarize/Notarization/Certification/Nullify/Nullification/Finalize/Finalization; Vote and Certificate each have tags0/1/2 for their respective variants. Certification has no network payload. Use typed codecs, not tag-number copying. Local vote eligibility additionally requires the configured local signer index.

## Eligible outbox and native path policy

On a successful relevant barrier: validate coverage, seal new witnesses and exact eligible VOTE/CERTIFICATE payload records, durably commit the ledger, then release successful native I/O. Eligible does NOT mean submitted/delivered. A later explicit matching send request or authorized recovery submission expands to actual configured destinations, persists that submission, and only then returns a releasable output. Duplicate logical IDs never create a new signature. Do not auto-publish every journal item.

The real source distinguishes output paths:

- Voter own votes, retries and voter certificate-channel broadcasts follow the native journal barrier. Require exact matching durable history for those paths.
- Resolver may forward already-verified Finalization/Nullification before local voter sync; raw Notarization is servable there only after native local certification. Do not invent a universal local-artifact barrier for resolver responses.
- Relay Propose/Prepare/Forward and resolver request/error are not own votes; no fictitious vote witness prerequisite.

This slice exercises voter VOTE/CERTIFICATE eligibility and explicit test submission only. Resolver/body gates and actual transport remain deferred. Future bridge uses All excluding self, preserves native bytes/priority and treats accepted feedback as submission only. SignedIntent/fixture metrics must never be called native finality.

## Durable ledger

Use an independent append-only ledger under the task/run directory. Record schema1, sequence, previous_record_hash, identity and ordered typed entries. Entries include policy seal, barriers/witnesses/eligible payloads, submissions and recovery generation links; preserve sealed history. Canonical JSON payloads, header u64BE length + u64BE bitwise complement, footer H(`I1b/witness-record/v1`,[payload]). Bound frames16MiB. Hash-chain and decoded identity/sequence must verify.

Committed head is88 bytes: ASCII `I1WHEAD1`, offset u64BE, sequence u64BE, last record hash32, and H(`I1b/witness-head/v1`,[run,config,worker u32BE,previous56 head bytes]). Initial empty hash/offset/sequence zero. Sync newly created ledger/head and directory. Append+sync ledger, write+sync head.pending, rename to head, sync directory, THEN expose committed records or successful completion. Only incomplete trailing bytes above a trusted committed floor may be truncated. Complete corrupted/identity-mismatched records, missing committed bytes/head, rollback or inconsistent sequence fail closed. Exactly one complete valid uncommitted final batch may be explicitly promoted and re-synchronized; it cannot silently erase history.

Persist complete page/frame evidence inside records for this bounded profile; do not introduce missing external snapshot dependencies. Raw full-file copies may be retained separately as audit artifacts.

## Recovery: preserved source, fresh generation, NEW barrier

Before opening a source with native storage, copy bounded raw selected files and ledger/head bytes read-only. Native Runner/open/replay may synchronize or repair; do not use them to establish the original snapshot. V1 raw container header: `CWIK`, layout u16BE1, application version u16BE0, CRC32C(first8) at bytes8..12, zero reserved padding to4096; data begins4096. Validate before interpreting pages.

Compare decoded source artifacts with all protected sealed history. Missing/conflicting protected bytes fail closed. Never truncate source or overwrite old output/history. V1 accepts only a fully valid bounded snapshot; incomplete/corrupt source requiring repair is rejected and retained.

Create a FRESH target generation/directory (existing/nonempty or source-equal target rejected). Copy accepted canonical native page bytes through actual native Blob operations with the same version/page profile, obtain a NEW full sync, validate again, then persist recovery witnesses/outbox links. Preserve original raw hash and old ledger chain. Recovered wire bytes must exactly equal their typed original encoding; use public verifier-only inputs, no private key/sign call. Do not claim source readability or target Runner's incidental filesystem flush proves an old barrier.

## Acceptance cases

W1 plain write/readability alone gives no eligible output; exact successful range-SYNC does, only where coverage is complete.
W2 unrelated SYNC range/blob cannot cover prior dirty frame bytes; full sync covers established prior mutations; emptySYNC is not full sync.
W3 failed mutations/barriers create no successful witness; uncertain state needs recovery, protected history retained.
W4 exact native pages/varints/artifacts across boundaries and partial rewrites; zero/oversized lengths, wrong slot/CRC, noncanonical/incomplete frames rejected; valid fallback slot accepted.
W5 epoch/cid/registry/domain/signer/section mismatch and Artifact-vs-wire tag confusion rejected; Certification cannot become a vote.
W6 exact vote and voter certificate eligibility; no submission before ledger commit; explicit duplicate send/recovery retains logical ID and exact bytes.
W7 deterministic faults after native completion/before ledger, after ledger frame sync/before head, after head/before result release; no successful release/Ack on failure, protected head/tail rules hold.
W8 controller cancellation while evidence Future or Ready result is retained; root ownership and restricted cleanup, no ordinary consumer poll/output after failure.
W9 copied snapshot recovery into fresh generation creates NEW barrier and exact old output; no signer/private key, no mutation of source files or old history.
W10 missing/corrupted committed ledger/head, corrupt protected source history and unsafe target paths fail closed without empty-state reset/truncation.
W11 seeds7/42/99 with wall pauses0/2ms: complete logical evidence/eligibility/submission traces and canonical identities match.
W12 all68 prior tests remain passing; native behavior ordering is not tightened for resolver/relay paths that this slice does not implement.

Record commands, configs, seeds, raw files/ledger/heads, exact metadata/hash vectors, source/patch revisions and failure artifacts. Independent verifier derives expectations before inspecting the new implementation. No full I1b PASS until subsequent timing, guarded signing, p2p, real engines and native two-process replay pass.

## Lead precision amendment1 — source-derived verifier questions

Registry hash is H(`I1b/voter-registry/v1`, flattened parts): for each member in ascending member-ID order append member u32BE, canonical native identity-key bytes, BLS public-key bytes, and verified PoP bytes as FOUR separately length-framed components. Require exactly members0–4, distinct identity keys AND distinct BLS vote keys, and valid PoP. Native signer order remains sorted BiMap identity-key order. `namespace_hash` is exactly the already-derived32-byte voting namespace, not another hash. Raw source-file identity is H(`I1b/source-raw/v1`, [whole raw file including V1 header]).

Coverage must include every byte participating in the selected CRC, even bytes belonging to another frame or an incomplete unsealed tail. A matching SYNC write ending at slot0 byte250 does NOT establish extent256 by itself. An extent lower bound may come from a successful full sync, or from a successful nonempty range-SYNC whose checked end reaches the needed physical-page end; emptySYNC creates no new extent bound. Track later shrink/overwrite invalidation. Ledger validation must independently enforce these coverage/extent conditions, not trust a producer's Boolean.

A valid foreign-member individual vote in the selected LOCAL voter journal is rejected as an invalid selected artifact, not merely hidden from wire eligibility. Quorum certificates may have any authorized signer set, including one without the local member. Certification remains non-wire data.

For this bounded VOTE/CERTIFICATE API, priority is fixed true. Destinations are a set normalized into ascending unique IDs; empty sets, unknown members or any explicit self member reject the WHOLE request before staging. The later native Sender filters self before calling this gate. A `Submission` here means a durable payload+destination AUTHORIZATION; it is not a transport attempt or delivery. Exact duplicate authorizations reuse stable IDs/bytes and do not create another ledger authorization or signature. Actual transport retries/attempt IDs and Step/Ack attempt dedup belong to later p2p/worker integration. Do not claim that duplicate authorization suppresses all future native retransmissions.

Grant-time submission APIs may return IDs/receipts, never a newly releasable packet before successful Step/evidence commit. Offline `submit_recovered` is permitted only on an anchored, fully completed recovered generation; it may return the already-durably-authorized records/bytes, and records remain explicitly offline—no Clock sample, fake protocol grant/tick, signing, Step/Ack or actual transport is claimed. If implementation uses reserved zero coordinates internally, annotate them as OFFLINE_RECOVERY and never expose them as a real clock/grant event.

Rollback detection requires an EXTERNAL trusted minimum receipt, outside the worker ledger/head fault domain: identity, sequence, record hash (and offset where recorded). Opening/recovery must verify the exact prefix receipt and accept a valid later descendant head; it must reject heads below/conflicting with that floor. This permits witness commits made before a lost Ack. An unanchored forensic inspection can check intrinsic framing/chain only and cannot authorize recovery publication or claim detection of a self-consistent old ledger+head replacement. Unit parent receipts model the future hub's acknowledged receipt. The head is checksummed and identity-bound, not a MAC against an attacker who can rewrite every trust anchor.

Recovery targets must not exist at all; reject empty existing directories, symlinks, source aliases or overlapping source/target paths. Generation must be exactly checked old_generation+1 and all signing/context parameters remain identical. Always preserve source copies and committed authorization history.
