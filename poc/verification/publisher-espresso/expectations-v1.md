# Publisher/Espresso independent expectations v1

Prepared 2026-11-15 by the named `bft_verifier` role, before any Builder implementation or explanation was read. This is an acceptance oracle for `publisher-espresso-component-v1`, not an implementation result. Status: **EXPECTATIONS FROZEN FOR LEAD REVIEW; no candidate checked**.

## Provenance and scope

The verifier first read v10 [architecture §1.4](../../../formalization_v10/00_arch_zh_v0_1.md), [Publisher §§3–4](../../../formalization_v10/04_publisher_zh_v0_1.md), [GFR §§3.2–3.5, 4.7](../../../formalization_v10/09_finality_rules_zh_v0_1.md), v2 [§§4.5, 5.1, 5.3, 12.2](../../../formalization_v10/poc_design_zh_v2.md), root AGENTS/DESIGN, v2 handoff/setup, workflow, dependency-plan-v2 and the 06/07/08/16 handoffs. Only afterward did the verifier read [the component contract](../../docs/contracts/task-contract-publisher-espresso-v1.md). The conformance skill was applied to distinguish proof observation, complete material, delivery and persistence.

The source checkout's Regional BFT HEAD was independently read as `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`; its DESIGN pins Commonware `2026.10.0`. This component does not change a BFT rule or verify a native quorum. P0 remains 3 independent committees of 5, 4 distinct signers, 10-height windows. The actual named role is `bft_verifier`; its declared configuration is GPT-6 Astra/xhigh. The available tool supplied no separate model-runtime attestation.

`S` below denotes a source requirement. `A` denotes the explicitly chosen engineering contract: integer representation, bounds, wire codec, store transaction algorithm and pure-input API. Neither class permits redefining 03 domain objects. Fixtures must be explicitly non-cryptographic and implement the same verifier trait used by production adapters. Their PASS is not a CheckFINAL/BLS PASS.

The adjacent [vectors-v1.json](vectors-v1.json) contains literal expectations and wire bytes. Those values come from the written rules and field layout, not an implementation encoder. Timestamp units are microseconds. Seeds are N/A for fixed arithmetic/wire cases; only D01 uses 7, 42 and 99 for externally chosen delivery schedules.

## Independent matrix and contract mapping

| Independent ID | Contract | Trigger / pre-state | Expected result and invariant |
| --- | --- | --- | --- |
| T01 | PE02 | Live proposal, deviation at and one unit beyond ±delta | Equality is WithinBound. Beyond bound remains an ordered log entry with OutsideBound carrying the full absolute deviation, not merely excess over the bound. Compare proposal tick, never finalize/receipt tick. S: architecture E-TS. |
| T02 | PE01/PE02 | Different consecutive q have equal theta | Accept both, preserve q and message order; do not deduplicate messages or report a fork. S: E-S and GFR §3.3. |
| T03 | PE03 | Fixed theta while proposal tick/q increase | q advances, watermark does not exceed theta. With delta=1000, theta=10000 at proposal 10000/11000/11001 yields Within/Within/Outside(deviation=1001). S: v2 §4.5. |
| T04 | PE02/PE04 | Nondecreasing timestamp outside E-TS; quantized offset | OutsideBound is a timing-premise failure, not InvalidSource. Offset then floor-quantization is exact. Invalid quantum, negative result and overflow reject atomically without consuming q. S+A. |
| T05 | PE05 | theta regresses after an accepted proposal | InvalidSource/Regressed; no clipping, q consumption, policy mutation or tick-frontier mutation. Already finalized/read prefix survives. S: E-S; A: atomic operation. |
| T06 | PE05 | Proposal is non-live, finalization later occurs while live | Assessment remains NotAssumedNonLive. Later finalization does not retroactively establish E-TS. S: architecture §1.4. |
| L01 | PE06 | Proposed but not finalized; attempt out-of-order finalize; repeat finalized q | No reader-visible block before finality. Cannot finalize over a missing predecessor. Exact repeat retains original finalization record and timestamp and does not advance the event-tick frontier. S+A. |
| L02 | PE08 | Reader gets q2 before q0, then incomplete q1 | Buffered future blocks remain unavailable through read/prefix/watermark. Visible tips: None, q0, q0, q2 when last q1 message arrives. S: complete prefix; A: fragment API. |
| L03 | PE01/PE07 | Empty message list, repeated messages, canonical wire | Empty list is complete only with its explicit count. Repeated messages retain separate indices. Exact wire bytes round-trip; malformed/truncated/oversized input fails. A: PE-LOG-WIRE-1. |
| L04 | PE08/PE10 | Independent A/B delivery, duplicates and conflicts | A progress never changes B. Exact duplicate retains first receipt; conflict/wrong source/out-of-range index cannot overwrite an accepted prefix. When both reach the same q, public bytes and object locations agree; local receipt times may differ. S+A. |
| L05 | PE10/PE15 | Message before header; reader snapshot/reopen with pending fragments | No inferred count. Later header must detect any already-buffered out-of-range index or receipt tick earlier than finalization. All reader receipts must be at/after the corresponding finalized tick. Recovery preserves visible prefix, pending bytes and first receipt. No local cache/global future read may fill a missing message. A. |
| M01 | PE09 | Entire proof-only publication read, body absent from public history | The byte-complete block advances log prefix; semantic result remains publicly incomplete. This is not a missing-log WAIT. S: GFR §§3.2/3.4. |
| M02 | PE09/PE13 | Hash reference with local body but no public body position | Local body cannot create public completeness or Recorded. Later actually published body retains its own q/index. S: Publisher §3.2/GFR §3.2. |
| M03 | PE09/PE12 | Multiple legal proofs / optional anchor material / late reader | Preserve original locations and original bytes. Receipt read time, extra proof or optional old anchor proof must not redefine first public completeness. q_avail and G admission remain 07 computations. S. |
| P01 | PE11 | Offer with missing input, known invalid input, complete valid fixture | Missing ancestor/anchor/config remains VERIFYING+missing and emits no submission. Sufficiently known wrong cid/body/proof/ancestry rejects. Only complete validated result becomes QUEUED. Core fixture tests invocation/state propagation, not cryptography. S: PUB-1. |
| P02 | PE12 | Duplicate key with identical/alternative legal proof; invalid replacement | At most one task. Legal proof bytes may differ for the same key. Preserve first verified result/payload, attempts and terminal facts. Invalid re-Offer/reverification cannot corrupt the existing task. Different keys are independent. S: PUB-2; A: first payload retained. |
| P03 | PE13 | Prepare, dispatch, service ACK, server finalize, publisher reader confirm | PREPARED intent is durable before effect; phase initially QUEUED. Dispatch makes SUBMITTED. Service ACK and globally finalized but undelivered block remain SUBMITTED. Only verified complete publisher-reader receipt makes RECORDED. S+A. |
| P04 | PE13/PE15 | Crash before/after prepare persistence; dispatch escaped but mark absent | Reopen the actual file. Before persistence: no new intent/effect. After persistence: replay same stable ID and exact payload. Externally escaped but unmarked PREPARED intent is replayed under the same ID; later new retry increments number only after prior dispatched/ACK and explicit input. A. |
| P05 | PE13/PE15 | Crash after server finalization but before reader delivery or receipt commit | Task remains SUBMITTED until actual reader-prefix verification. Pre-commit crash has no receipt; post-commit reopen preserves receipt. Repeated confirmation emits no second Recorded effect or task. S: PUB-2/3. |
| P06 | PE14 | Program TIME/Close, Manager change, restart | No task deletion, phase downgrade or cancellation from these local events. They provide no public stop authority. S: PUB-3. |
| P07 | PE14 | Matching public closure/selection; unrelated selection; cutoff equality | Only exact projected affected keys change. Watermark theta=end-1 does not stop; theta=end does, but only from a complete reader prefix. STOP is durable and prevents new/replayed PREPARED output. Unrelated session assignment does not stop the old task. S+A. |
| P08 | PE13/PE14 | Recorded then STOP/late ACK; STOP then historical confirmed receipt | `phase==RECORDED` iff receipt exists. Historical receipt and stop reason both survive. STOPPED may become RECORDED after valid receipt, while stop remains and retries stay forbidden. Late ACK cannot downgrade. S: PUB-2/3; A: race resolution. |
| P09 | PE13 | Verifier receipt with wrong key/source, missing range, invalid index | Core rejects unless key/source match and publication q <= complete_through_q <= publisher reader tip, with actual message index in range. No receipt may rely on global future data. proof@q1/body@q2 must retain complete_through_q>=2. A; source completeness obligation. |
| P10 | PE15 | Snapshot write/sync/rename/directory-sync failure, corrupted main | No effect before durable success; no half-installed memory. Pre-rename failure retains old main. Post-rename parent-sync failure poisons handle (commit unknown), requires reopen. Bad main/namespace/version is error, not empty initialization; temp is never auto-promoted. A. |
| P11 | PE11/PE13/PE14 | VERIFYING or STOPPED task without a fully validated Offer receives valid public receipt | Receipt alone cannot make RECORDED: complete Offer result/payload must already be durably verified, or the same transition must first complete verify_offer. Missing local anchor/config/ancestry remains WAIT even if the public publication is available. S: PUB-1. |
| D01 | PE16 | Fixed schedules for seeds 7/42/99 with 0/2ms wall pause | Identical canonical virtual traces, public bytes, prefixes, task phases, receipt/stops and attempt IDs for corresponding runs. Wall diagnostics excluded. Pure functions do not read wall time or automatically advance/submit. S: v2 §5.1. |

## Literal material and delivery oracle

Use opaque fixtures named `test-codec-1`, `cid-C`, anchor `A`, physical `B1(parent=A)`, `B2(parent=B1)`, target proof `P2(B2)`. These labels do not define 03 hashes or any signature validation.

Public history: q0 contains P2; q1 contains body B2; q2 contains body B1. Each block has an explicit complete list. A consuming q0 has a complete byte-log prefix while the target result lacks B1/B2 bodies. Through q1 it still lacks B1. Through q2 all required bodies and the proof are publicly available. The **source-derived handoff oracle** for 07 is earliest complete q=2; 06 must expose the positions that allow 07 to compute it and must not implement G acceptance. Optional old anchor evidence at q4 or a second valid P2 at q5 does not move the source-defined earliest q. A reader that locally cached B1 before reading q2 does not thereby acquire its public position.

For fragment delivery, q0 has zero messages, q1 has two distinct messages (indices 0/1), q2 has one. B receives complete q2, complete q0, q1 header+index0, then index1. Expected B tips are [None,0,0,2]. Before the final step read(2) is WAIT, even though q2 is buffered. A may have received all three immediately; its state cannot fill B's gap. Run each exact delivery twice and retain the initial receipt time. A conflicting duplicate cannot overwrite the existing public bytes. Repeat after serializing/recovering B while q1 is incomplete.

## Persistence oracle: real files, not cloned memory

Each injection starts from a named complete durable generation and captures the old main-file bytes/hash. The candidate test must close/drop the old store and open a new handle from the actual path. Assertions compare decoded stable task identity/payload/attempt/terminal state plus generation; a clone of an in-memory store does not satisfy the case.

| Durable mutation | Before-commit recovery | Successful-commit recovery |
| --- | --- | --- |
| Create/WAIT/QUEUED | Original store; no new durable task or output | Same keyed task and missing/validated material |
| Prepare intent | No new attempt or effect | Same PREPARED stable ID, number and exact payload; phase preserved |
| Mark dispatch | Original PREPARED can replay same ID | DISPATCHED, SUBMITTED; explicit next retry may create next number |
| Service ACK | DISPATCHED, SUBMITTED | SERVICE_ACKNOWLEDGED, SUBMITTED |
| Receipt | No new receipt; pending task retained | RECORDED with immutable publication position/complete-through/basis |
| Public STOP | Old retry permission | Durable stop, no fresh or prepared output; preserve task and any receipt |

Exercise failure before writing temp, during temp write, temp sync, rename, and parent-directory sync. Any failure known to occur before rename leaves the prior main authoritative. Parent sync failure after rename is **INDETERMINATE COMMIT**, not a normal rollback: current handle must reject further mutation/output until reopen. A test injecting only an early error is insufficient to cover this boundary. Reopen may discover the committed next main generation; assertions must follow actual complete main state. Do not silently install/return effects from an uncertain in-memory generation.

Test an old uncommitted temp beside valid main, wrong namespace/config, truncated/checksum-corrupt/unsupported-version main, and an already-existing store whose main was lost. Explicit fresh-store creation is separate from recovery; none of those recovery errors may become an empty queue. The contract does not specify a particular checksum algorithm or file-byte layout; tests must attack the chosen candidate format and retain raw mutated files/reopen errors.

## Test execution and remaining gates

Candidate review will name an exact committed SHA and inspect code read-only. Test commands are to be bound to the exported API after its interface mapping exists; no executable names are invented here. All Rust build/heavy tests use `flock /tmp/megaroation-poc-heavy.lock`, `CARGO_BUILD_JOBS=2`, Rust 1.95.0, task-owned cargo/targets and `--locked --offline`; no nested lock. Evidence records exact commands, input/vector hash, candidate SHA/dirty state, raw file paths, trace digests, bounds, seeds and PASS/FAIL/INCONCLUSIVE.

Component acceptance does not satisfy D02 controlled grant/Ack/transport integration, D03 canonical object/codec/validation adapter, D04 real final certificates, D07 GFR/stop-scope projection, D16 official dev-node finality, or D06 integrated crash/reader cases. No native quorum, 15-worker run, official Espresso consensus or timing/liveness theorem is claimed here.
