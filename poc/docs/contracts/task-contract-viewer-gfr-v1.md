# Viewer GFR contract v1 — pure replay and independent readers

Date: 2026-11-16. Contract ID **GFR-07-v1**. Status: **FROZEN for M1/M2 by the feature 07 lead on 2026-11-16**. Independent source-first review approved the semantics; its conditions are resolved: immutable expected verification profile even without a binding, explicit per-input verification prerequisite WAIT before the bool checker, actual `sim_espresso` crate name, and one ReplayRecord per atomic event. This is the authorized pure component stage of feature 07; passing it does not mark all of 07, native certificate integration, or the 15-worker regular pipeline accepted. Subsequent interface changes require a lead-approved versioned revision and affected independent re-verification. Documentation ownership transfers from Docer to lead with this freeze.

## 1. Authority, pins, and ownership

Working checkout: `/home/jy/.codex/worktrees/1e2f/megaroation_poc`, branch `codex/poc-viewer-gfr`. Integrated baseline: `335553359cd39a3a7494fbd9760436cdb2c41a33`, containing exact 03 delivery `e9c15e04bc83b335668f3ecd837367aa87344a89` followed by 06 delivery `4b034dfbb6c3e1c7143208347b546daab8dfe298`. No merge, reset, commit, or shared-file edit is assigned to Docer.

The governing task scope is [07 handoff](../handoffs/07-viewer-gfr-zh.md), [workflow](../handoffs/workflow-zh.md), and the read-only source documents `/mnt/reth/jy/megaroation_poc/poc/docs/commander/dependency-plan-v2.md` and `dispatch-independent-wave-20261116.md`. The dispatch replaces the old `core/gfr` path with the isolated workspace below. Root [DESIGN](../../../DESIGN.md), [v2 handoff](../../../formalization_v10/poc_v2_xuelab_handoff_zh.md), [setup](../../../formalization_v10/poc_v2_xuelab_setup_zh.md), and [design](../../../formalization_v10/poc_design_zh_v2.md) were read. The [regional-bft-conformance skill](../../../.agents/skills/regional-bft-conformance/SKILL.md) supplies the acceptance-case method. `protocol-spec-writer` was not present in the available skill catalog or searched local skill roots.

| Writer | Allowed paths and responsibility |
| --- | --- |
| Docer | Only `poc/docs/contracts/task-contract-viewer-gfr-v1.md`; source mapping, contract, revisions requested by lead. |
| One Builder | `poc/crates/viewer-gfr/` including its own workspace, lockfile, exported API, pure reducer, reader adapter, checkpoints, component tests and examples. |
| Independent Verifier | `poc/verification/viewer-gfr/`; independent vectors/expectations, harness and review. Expectations precede implementation inspection. |
| Tester, after verification | Run agreed cases on the exact verified integrated candidate; evidence under task-specific `.build/artifacts/` and reports in its assigned verification paths, handed over serially. |
| Lead | Freeze/version this contract, integration and milestone commits, task-specific reports, path ownership transfers. |

Forbidden writes: `poc/crates/types/`, `poc/crates/core/`, authorization, 06 crates, root Cargo files, native recovery/runtime source, `poc/docs/commander/`, formalization sources, and Regional BFT submodule/worktree/history. Consume 03 objects, Plan, metadata, execution and Merge; propose any required shared change to its owner. Do not fork these implementations. One writer per path; retain others' work.

### Source map

**S** denotes a source rule. **A** denotes this contract's adapter choice; A does not amend v10.

| Source/version | Sections and decisions derived from it |
| --- | --- |
| [v10 09, working draft v0.1, 2026-11-10](../../../formalization_v10/09_finality_rules_zh_v0_1.md) | S: §§3.1–3.4 public event order, coverage, original-position WAIT, earliest complete publication; §§4.1–4.3 atomic replay, acceptance, both regular paths; §4.5 Expire/HALT; §§4.7–4.8 records, lifecycle, checkpoint and API; §4.6 future RESUME gate. SHA-256 `6922b77e1de0c8eaddde9f08d08d6734ae63844d406f0e6e32438e33ba0dabcc`. |
| [v10 00](../../../formalization_v10/00_arch_zh_v0_1.md) | S: §§1.2–1.4 complete finalized prefixes, L1 positions, nondecreasing Espresso time and watermark; §§3.2–3.4 physical identity, anchor state, Merge, native execution and origin requirements. |
| [v10 01](../../../formalization_v10/01_l1_authority_zh_v0_1.md), [02](../../../formalization_v10/02_session_program_zh_v0_1.md) | S: 01 §§3.1, 3.12 activation and pure Plan; 02 §3.10 CheckFINAL, §§4.1–4.5 finality and same-chain ancestors; historical public verification does not re-run local receive-time admission. |
| [v10 03](../../../formalization_v10/03_session_manager_zh_v0_1.md), [05](../../../formalization_v10/05_batcher_zh_v0_1.md) | S: 03 §§3.2, 3.10, 4.1 lifecycle identities, material and permanent closure; 05 §§3.1–3.2 G/source/position consumption, actual Kept and downstream delivery distinction. |
| [v10 11](../../../formalization_v10/11_global_chain_safety_liveness_zh_v0_1.md) | S: §§1.1–1.4 G-S1 at equal replay position, fixed anchors, G-S2 without RESUME, G-S3 only from genesis without RESUME; §2 timing claims need their stated premises. |
| [v2](../../../formalization_v10/poc_design_zh_v2.md) | A already authorized by v2: §§3, 4.1, 6.3, 9.1, 10.1–10.2, 11.2, 12.2–12.3: 3×5, four distinct signers, static schedule, 10-height windows, integer microseconds, finite DEP/L1 inputs, pure replay, independent A/B, G1/G2. SHA-256 `95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab`. |
| Regional BFT gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6` | Read-only initialized source at `/mnt/reth/jy/megaroation_poc/regional_BFT_sim`; verified HEAD equals gitlink. DESIGN §§2.1, 3.1, 5 distinguish proof observation, complete validated delivery and admission. Its n=15 benchmark profile is not this n=5 profile. DESIGN SHA-256 `0749ff8bef9ad18b3794788d9f72c887126f4a2368dcf6adcb8587853eee3eae`. |
| Commonware 2026.10.0, release `v2026.10.0` (`92a2f68` as pinned DESIGN records) | Pinned upstream source read locally: `commonware-consensus-2026.10.0/src/simplex/mod.rs`, Protocol Description/Certification, SHA-256 `f2745799e26a93b55fd1aa8178d1252e3a10717eac28a1fcc1f25335bcae829d`; `commonware-utils-2026.10.0/src/faults.rs`, `Faults::quorum`, `N3f1::max_faults`, SHA-256 `f443c640e027d71263cb908fbb8bfd32ccf09d26bad80585f10495179fc2c1f9`. Simplex finalization covers ancestors; notarization/certification/nullification do not alone pass CheckFINAL. n=5 gives f=1, quorum=4. Native bytes/subjects remain owned by 04. |

## 2. Delivery stages and evidence boundaries

**M1, small coherent interface milestone:** exported replay comparison/codec and lifecycle projection, bounded public-object envelope, typed status/reader/verifier boundaries, pure query types, independently authored golden vectors. Commit the interface and vectors before full reducer work. No placeholder function may return success for an unimplemented rule.

**M2, this stage:** Init/AdvanceGlobal/Chain/Responsibility, regular and Expire/HALT, earliest publication completeness, atomic WAIT, historical checkpoint verification, actual 06 reader integration, independent expectation checks and G1/G2 bounded matrix. Force and RESUME stop explicitly at their first required position. No dependency on completion of 01.

**Gates remaining after M2:** 04 actual native FinalVerifier/certificates; actual 09 controls and 12/13 position-bound ResumeCut integration; 08 regular success with real production under controlled grants; complete 07 handoff acceptance. 16 owns official Espresso source verification; use of 06's real component does not claim official Espresso consensus verification.

Keep the following observations separate in artifacts: progress evidence; native final-proof verification; complete hash-linked body delivery/application validation; first public completeness `q_avail`; G acceptance with `pc`; later Inbox acceptance; D derivation. A fixture that says a proof passes exercises only the pure rule interface. It is **non-cryptographic** and cannot satisfy the native gate. No live clock, scheduler fault table, unreceived source object, future finalized block, or shared mutable G may enter the rule path.

## 3. Frozen public interface and capabilities

All 03 types below are imported from `poc_types::domain`: `ReplayPosition`, `ReplayKind`, `ReplayIndex`, `LifecycleEvent`, `LifecycleBody`, `CloseReason`, `Descriptor`, `ResultKey`, `Anchor`, `ExecutionAnchor`, `PhysicalBlock`, `VirtualBlock`, `SessionId`, `PlanInput`, `PlanRecord`, `Window`, `L1Position`, `L1Header`, `ConfigRef`, `Hash32`, `InputStatus`, `InputMissing`, `InputError`, and `FinalVerifier`. Export these by named re-export where helpful; no duplicate domain structures. 03's codecs are structurally checked, not proof/source authentication.

The crate exports these operations (lifetimes may be made explicit without changing behavior):

```rust
pub fn init(input: &InitInput, reads: &ReplayReads<'_>, services: &ReplayServices<'_>)
    -> Result<InitResult, GfrError>;
pub fn advance_global(state: &ReplayState, reads: &ReplayReads<'_>,
    services: &ReplayServices<'_>, through: Option<&ReplayPosition>) -> ReplayResult;
pub fn chain(state: &ReplayState) -> ChainView<'_>;
pub fn responsibility(state: &ReplayState) -> ResponsibilityView<'_>;
pub fn compare_positions(a: &ReplayPosition, b: &ReplayPosition)
    -> Result<std::cmp::Ordering, GfrError>;
pub fn encode_position(value: &ReplayPosition) -> Result<Vec<u8>, GfrError>;
pub fn decode_position(bytes: &[u8]) -> Result<ReplayPosition, GfrError>;
pub fn lifecycle_projection(value: &LifecycleEvent) -> Result<Vec<u8>, GfrError>;
pub fn lifecycle_id(value: &LifecycleEvent) -> Result<Hash32, GfrError>;
pub fn encode_publication(value: &PublicObject) -> Result<Vec<u8>, GfrError>;
pub fn decode_publication(bytes: &[u8]) -> Result<PublicObject, GfrError>;
```

`InitInput` is `Genesis { config: PublicConfig, genesis: ExecutionAnchor, parameters: GfrParameters, expected_verification: VerificationProfile } | Checkpoint(GlobalCheckpoint)`. `GfrParameters { announcement_us, grace_us, expiry_us }` uses positive announcement/grace/expiry; window length is the already-bound 03 configuration's 10 heights. Parameters, source IDs, genesis/config and expected verification profile are bound in state/checkpoint even when the verifier capability is unavailable. A later supplied binding must match that profile exactly; mismatch rejects, and absence never erases the expected profile. Never silently replace these values on continuation. All addition, multiplication, conversion and height increments use checked arithmetic. Overflow is explicit error, not saturation or wrap.

`ReplayReads` holds `espresso: &dyn sim_espresso::FinalizedLogReader` and `l1: &dyn L1ControlReader`. `ReplayServices` holds the existing 03 `PublicConfig`, a plan capability, 03 `MetadataProvider`, 03 `DepProvider`, and `finality: Option<VerifierBinding<'_>>`. Immutable configuration equality is checked at use. All reader/service operations are read-only; repeated queries are deterministic for the supplied delivered inputs.

The plan capability exposes `resolve(&PlanInput, &dyn PublicReferenceView) -> PlanOutcome` and `descriptor(&PlanRecord, &Anchor) -> Result<Descriptor, PlanError>` with the exact 03 outcome/error types. The production implementation delegates to `poc_authorization::StaticScheduleProvider`. References are established by checked genesis or accepted replay records, not an unverified caller map. `Ready` means selected, `None` means PENDING, `Wait` blocks the event, and `Reject` reports invalid input. The static provider normally never returns None; a named fixture capability may test the PENDING branch without claiming real static schedule emits it.

`VerifierBinding { verifier: &dyn FinalVerifier, basis: &dyn VerificationBasis, profile: VerificationProfile }` consumes the existing `check(&Descriptor, &PhysicalBlock, &[u8]) -> bool`. The viewer-owned trait `VerificationBasis` exposes `require(&self, descriptor: &Descriptor, block: &PhysicalBlock) -> InputStatus<()>`; it establishes availability and validity of the historical public configuration/registry/native verifier basis for that exact input before the bool call. Wait/Reject propagates without calling check. Only Ready permits a true/false result; a missing native basis must never be silently mapped to false. `VerificationProfile` is `NonCryptographicFixture { id: String } | Native { protocol_version: u16, verifier_version: u16, source_revision: String }`. The profile is provenance, not a proof that an implementation is native. A native acceptance claim additionally needs 04's reviewed exact implementation/candidate and actual certificate artifacts. No default verifier and no production always-true verifier. Missing binding is `WAIT(FinalVerifierUnavailable)` for an otherwise viable selected candidate; unknown/incompatible verifier versions are errors. Known malformed/invalid proofs return false and are excluded. The viewer independently checks Plan, CID, target digest, ancestry and Merge.

The minimal L1 extension is:

```rust
pub trait L1ControlReader: poc_types::domain::FinalizedL1Reader {
    fn source_id(&self) -> &[u8];
    fn controls_at(&self, height: u64) -> InputStatus<Vec<ControlRef>>;
}
pub struct ControlRef {
    pub position: L1Position,
    pub kind: ReplayKind, // only Force or Resume
    pub control_seq: u64,
    pub source_bytes: Vec<u8>, // opaque authenticated upstream record
}
```

`controls_at(h)` yields the **complete ordered control list**, including verified empty lists, and is consistent with `require_control_coverage(h)`. Positions must belong to h, be unique and ordered; sequence/identity conflict or disagreement with authenticated source is Reject. 07 does not assign 09 ABI or policy bytes. For M2 regular cases implement an adapter over 03's actual `FixtureReader`: return empty only after its header and empty-control commitment/list delivery pass. Do not turn missing controls into empty. Separate explicitly named control fixtures exercise unsupported Force/RESUME and full-block coverage; they are not real Anvil receipts.

`InitResult` contains `state: Option<ReplayState>` and a status. Genesis initialization has `pc=None`, no fabricated replay event, and no invented SessionAssigned position; its selected initial duty is available through Responsibility. `ReplayResult { state, added_records, events, status }` returns a new owned state without mutating the caller's state. `ChainView` exposes G, per-height sources, per-height acceptance position/native block/state evidence and pc; `ResponsibilityView` exposes Tail and the same pc. Queries perform no replay, download, or callback.

Status is `Ready | Wait { next_position: Option<ReplayPosition>, missing: Vec<Missing> } | UnsupportedControl { position: ReplayPosition, kind: ReplayKind } | Reject(GfrError)`. Missing is structured, sorted and deduplicated: Espresso next q/header/message list/watermark; L1 header/control coverage; public object location/hash; inherited 03 input missing; final verifier; future resume cut. Display text is not protocol identity.

`Some(p)` in WAIT means coverage has established p as the next event and only its required material remains absent; retry stays at p. `None` explicitly means a source frontier is incomplete and the exact next event cannot yet be proved. Diagnostics may name a tentative timer, but must not label it completed or commit it. Once the frontier is filled, earlier newly revealed events are ordered normally. This is an A-level representation of v10's missing-log case, not permission to move a determined event. A finite supplied prefix need not be the network's latest prefix.

`through` is an optional inclusive comparison bound for historical queries/finite runs. It does not create an event or move pc. Return Ready after all events through that bound have been proven covered and processed, or when no event is pending in the supplied covered horizon. If the request requires an event or coverage not supplied, WAIT. Without a bound a live NEXT/PENDING may WAIT on its future timer; HALT still processes supplied Espresso records and must stop on unsupported controls. No hidden event-count limit may return Ready while eligible events remain.

## 4. Replay position encoding and lifecycle identity

**GFR-POS-1 (A)** wraps the existing 03 typed skeleton with free functions. Encoding: `u16BE(1) || u64BE(key_us) || u8(kind) || index`, kind tags Force=0, Resume=1, Expire=2, SessionEnd=3, Espresso=4. Index is raw `u64BE(block)||u64BE(tx)||u64BE(log)` for either control; raw `u64BE(height)` for Expire; raw `u64BE(beta)||u64BE(r)` for SessionEnd; raw `u64BE(q)` for Espresso. No nested versions. Exact lengths: controls 35, Expire/Espresso 19, SessionEnd 27 bytes. Reject wrong kind/index, unknown version/tag, truncation and trailing bytes. No consensus ordering by raw encoded bytes.

Order is lexicographic `(key_us, class, index)`, classes control=0, Expire=1, SessionEnd=2, Espresso=3. Controls compare mixed Force/Resume **by L1 position before kind**. Two control kinds at the same L1 position are conflicting identities and comparison/ingestion rejects them; no invented Force-before-Resume tie-break. Session index compares `(beta,r)`. Equal theta Espresso blocks compare q. `pc: Option<ReplayPosition>` is the last **fully committed** event, with None before any event. Inner repeated admissions never invent subpositions.

Independent M1 vectors must include these literal bytes (spaces are notation):

```text
Expire(key=100,h=9):
0001 0000000000000064 02 0000000000000009
Espresso(key=100,q=7):
0001 0000000000000064 04 0000000000000007
SessionEnd(key=100,s=(2,3)):
0001 0000000000000064 03 0000000000000002 0000000000000003
Resume(key=100,p=(1,2,3)):
0001 0000000000000064 01 0000000000000001 0000000000000002 0000000000000003
```

At key 100: Resume(1,0,0) precedes Force(1,0,1), which precedes Expire(9), then SessionEnd(2,3), then Espresso(7), then Espresso(8). Force(1,0,0) versus Resume(1,0,0) is a conflict. Independent vectors cover u64 limits and malformed typed pairs as well as byte errors.

**GFR-LIFE-1 (A)**: dedupe uses **position + kind + complete canonical body**, not position alone or session height. `kind()` is derived from the existing `LifecycleBody`; a separately supplied conflicting kind is invalid. Preserve event list order within a record. An exact duplicate causes no second allocation or reopening. Distinct SessionClosed and SessionAssigned events at one Espresso pc remain distinct.

For canonical comparison bytes define `frame(b)=u32BE(len(b))||b`, `list(x)=u32BE(count)||frame(x[0])||...`; lengths are checked. Projection is `u16BE(1)||frame(encode_position(position))||u8(body_tag)||body`. Body tags/fields:

| Body | Tag and projection |
| --- | --- |
| SessionAssigned | 0; `frame(Descriptor::encode_checked())`. |
| SessionClosed | 1; raw beta, r as u64BE, then reason u8. |
| BranchClosed | 2; raw branch u64BE, then reason u8. |
| Resumed | 3; raw control_seq, branch u64BE; `frame(Anchor::encode_checked())`; ordered list of virtual block projections. Reserved for future real RESUME transitions; codec support does not implement RESUME. |

Reason tags Time=0, Complete=1, Force=2, Resume=3, Expired=4, Excluded=5, Incompatible=6. A virtual block projection is `frame(Metadata::encode_checked()) || list(dep raw bytes) || raw32(pre_state_root) || list(transactions raw bytes)`, retaining the original ordinary transactions. This is a lifecycle comparison projection of the 03 VirtualBlock skeleton, not a replacement physical/native encoding or hash. Reuse checked 03 metadata/transaction/DEP validation and limits. A Resumed prefix has at most 4096 blocks, and projection at most 64 MiB. `lifecycle_id` is existing `poc_types::hash_parts("PoC/GfrLifecycle/v1", &[projection])` converted to Hash32; hash framing is the existing u64BE framing. Retain projection/full event for audit; a hash is not its verification material.

Golden lifecycle vectors independently cover both close variants, reason tags, two different bodies at one pc, exact retry equality, changed anchor/descriptor inequality, and length/version bounds. Large Descriptor examples reuse 03 goldens without inventing new descriptor encoding.

### Future RESUME atomic material boundary — lead decision

v10 09 §4.6 describes SessionAssigned in step 4 and Resumed in step 5, while v10 03 §3.10 ApplyLifecycle (the Resumed paragraph at source line 226) expects recovery StateMaterial before allocating the new assignment. **Retain the source-listed event records/order; do not silently edit or reorder v10.** A committed replay event is an atomic lifecycle batch. Before allocating or entering any new cid, a future Manager adapter stages and validates all required materials for the entire batch, including the precise ResumeCut, recovered anchor state and Plan inputs, then applies externally visible effects atomically. Lack of any input exposes none of that batch. 13 owns its integration implementation/review, including durability and acknowledgment. Current UnsupportedControl(Resume) emits no lifecycle batch and performs no partial G replacement. This decision does not assert that 13's gate is passed.

## 5. Public object adapter and canonical public comparison

06 `PE-LOG-1`, `PE-LOG-WIRE-1`, `PE-READER-1` are real consumed component APIs. Its `LogMessage{codec,bytes}` is opaque; no existing 03 production publication envelope was delivered. **GFR-PUBLIC-1 (A)** fills that boundary in 07 with exported functions, suitable for later Publisher/11/12 consumers. It does not redefine 03 object codecs or 06 PELG.

`PUBLICATION_CODEC: u32 = 0x47465201`. `PublicObject` is `Body(PhysicalBlock) | FinalProof { descriptor: Descriptor, target: Hash32, proof: Vec<u8> }`. Payload bytes: ASCII `GFRP`, `u16BE(1)`, then tag 0 for `frame(PhysicalBlock::encode_checked())`, or tag 1 for `frame(Descriptor::encode_checked()) || raw32(target) || frame(proof)`. Proof bytes are opaque and preserved. Target digest plus descriptor cid yields 03 ResultKey. Each payload must fit 06's 65,536-byte message limit; proof length at most 32,768, descriptor frame at most 16,384; 03's stricter object limits also apply. Reject over-limit construction before allocating. No fragmentation/second envelope is introduced in this stage.

Unknown protocol codec/version or malformed GFR payload cannot become a valid candidate. The block still has an ordered public message list: classify such objects as ignored/rejected publication diagnostics and continue, unless the **source envelope/completeness itself** is invalid. Never silently reinterpret an unknown codec as GFR-PUBLIC-1. Proof-only means the FinalProof message is fully delivered but target/ancestor Body messages may be absent from public history. Missing one of the source's declared messages means the Espresso prefix itself is incomplete, not a proof-only result.

Each indexed object retains exact `(q,message_index)` and exact canonical payload. Body identity is the 03 physical digest. Evidence may have several distinct valid proofs for a ResultKey; do not reject them as a conflict or overwrite earlier public locations with receive times. Optional external AnchorEvidence does not enter `q_avail` once the descriptor's anchor has been selected by replay.

Canonical public-state comparison at the **same pc** includes G virtual/native blocks and post-state evidence, sources, branch, Tail, selected descriptors, closed sets, committed record/effect/lifecycle order, and publication/result completeness as of processed Espresso history. Normalize maps by semantic keys and object locations by `(q,index)`; normalize each proof/body object by its bytes plus all original locations. It excludes receipt clocks, download queues, optional cache entries, future downloaded objects, and a chosen display proof. Keep all valid proof/location pairs, or compare their sorted full set; no first-arriving representative proof determines consensus identity. `q_avail` is proof-existence based, so adding a later proof cannot increase it. No compression to one representative proof is required.

A reader may have fetched a later watermark block to authorize a timer; its later public messages cannot affect an earlier Espresso event or its cached candidate set. Index publication objects only when their Espresso event is processed (with that whole event staged). Future download knowledge does not leak backward into G/source/q_avail. At equal pc, supporting prefix lengths may differ; compare the canonical public state, not private reader snapshots.

## 6. Event selection, coverage, and atomic progress

Derive timers from committed state only: NEXT/PENDING has Expire at `ts*(h_G+1)+expiry_us`; NEXT has SessionEnd at `ts*(h_max+1)+grace_us`; HALT has neither. Espresso event key is theta_q; control key is actual inclusion-header timestamp plus announcement_us. `proposed_at`, `finalized_at`, receipt tick and wall time never replace theta. E-TS excess is a diagnostic/premise failure, not a different ordering rule; descending theta or nonextending finalized history is Reject.

Before committing the earliest event with key x:

1. Obtain 06 complete finalized prefix through a block with theta≥x. Espresso(q)'s complete prefix through q suffices for its own key; no mandatory later block. Same-key future q is later in order.
2. Obtain complete finalized L1 headers/control lists through the first header with timestamp≥x−announcement_us, **including every control in that block**. For x below announcement_us, compare in a nonnegative domain and require the genesis coverage witness; no unsigned underflow. Under 03 fixed pace derive the covering height with checked ceiling arithmetic, never floor. Larger supplied coverage is allowed but must not pre-apply later controls.
3. Check every earlier event and all same-key controls are available. Missing list entries/coverage leaves frontier unresolved. Once selection is established, missing actual Plan, origin/metadata, DEP, state or verifier inputs yields WAIT(Some(p)). Header/control coverage does not imply these extra inputs exist.
4. Stage all effects of this event, including candidate indexing, all additions, same-block repeated regular handoffs, Plan resolutions, records and lifecycle. Commit the new state, record and pc together only when the entire event succeeds. A later substep WAIT discards all staged effects of that event; earlier fully committed events from this call remain in returned state/added_records/events. Do not publish partial lifecycle or partially accepted intervals.

Validate supplied prefix identity/continuation before applying it. A fork, regression, changed prior message/header, conflicting control identity or corrupted fixture is Reject, with no acceptance using the bad extension. Verified prefix cuts/checkpoints are not permission to ignore original authenticated message counts. A supplied result with invalid proof/body/window/continuity/metadata is excluded with reproducible reason, rather than changing G or waiting for a known contradiction to disappear. Validated but incompatible same-cid final chains violate the finality premise: stop the event with explicit `InconsistentFinality` instead of choosing a branch and calling agreement verified.

At an ordered Force or RESUME in this stage return `UnsupportedControl{position,kind}`, preserve pc and all pre-event state, and return prior completed records. Do not skip the record, consume a timer/Espresso behind it, interpret it as invalid Force, or claim READY. This applies even to a Force whose full future handler might judge it ineffective. The extension point retains authenticated position/control bytes for 10/13.

## 7. Complete result and G acceptance

For selected descriptor ν, anchor A, target B and ResultKey `(ν.cid,H(B))`, compute `q_avail` as the least q whose processed public prefix contains both (i) at least one proof accepted by the pinned FinalVerifier for ν and B, and (ii) every canonical hash-linked physical body strictly after A through B. Store original object locations; do not use local cached but unpublished bodies. A complete later descendant result can cover same-chain physical ancestors without separate ancestor proofs, but its own full ancestry and target body are still required. Missing physical ancestors are never padding.

When the complete known public prefix has no required proof/body, the candidate is **not publicly complete** and does not block unrelated events. A source message already committed at a public position but not delivered/read is a missing-input WAIT before its event. Likewise, otherwise viable selected candidates waiting for actual origin/DEP/state/verification inputs block the event. Check cheap definitive exclusions first; an unselected cid or malformed proof cannot force arbitrary state downloads. Preserve cached preselection objects and their historical q; recompute completeness from that history when the descriptor becomes selected. A body's/proof's later local arrival never changes its original public q.

For each selected context, process candidate target **03 canonical PhysicalBlock digests in ascending byte order** (A clarification of v10's canonical-encoding-hash order), not proof hash or receive order. Dedupe same ResultKey, remove already accepted overlap, and use updated h_G after each extension. For an extension to B at Espresso key x, require:

| Guard | Required check |
| --- | --- |
| Responsible | Exact selected cid/descriptor, open session and branch, Tail NEXT. |
| Final and connected | At least one valid target proof, full physical ancestry rooted at exact selected anchor, continuation of already accepted same-cid physical prefix. |
| Window | Each new physical height in fixed session window. Virtual padding can precede h_min when selected anchor is older. |
| Native compatibility | Call 03 RegionalMetadata/Merge and DEP/execution over selected ExecutionAnchor, checking metadata, origin not-ahead/drift, pre-state roots, parents and actual execution. Do not substitute recalculated fields for invalid body fields. |
| Public cutoff | `theta[q_avail] < theta_session_end`; equality rejects. |
| Whole interval expiry | For **every** new virtual h in `[h_G+1,B.h]`, `x < ts*(h)+expiry_us`, including padding before first physical block. Equality rejects the entire extension. |

On success append the full contiguous interval with `GlobalSource::Regional(ResultKey)` for each new height and its acceptance pc. Padding uses the covering result source. Preserve existing per-height sources when a later proof overlaps them. Save virtual block, exact native block/hash, post-state and physical digest supplied by 03 Merge; raw ordinary transactions and actual Kept have distinct meanings. G never manufactures a tail of padding without a decided physical target. No proof observation alone changes G.

## 8. Regular, PENDING and HALT

On acceptance of a physical block at current fixed h_max, perform immediate regular within the **same** Espresso event. Otherwise execute SessionEnd only after its public key is covered and earlier events finish, provided the current NEXT still belongs to that session. An invalidated old timer causes no second handoff; generate current timers instead of retaining arbitrary stale future queue entries.

Both paths close the predecessor with `SessionClosed{reason:Complete}`. The production anchor is the last accepted physical block of that session; if none, use its initial anchor. Do not pick the highest known unaccepted block or move the anchor merely because local time passed. The successor is `(beta,r+1)`, window `[old.h_max+1,old.h_max+10]`, source region=predecessor Plan region, reference block=**predecessor initial anchor**. Reference block and production anchor can differ. Call the existing Plan/descriptor constructor with these exact objects. Ready produces SessionAssigned and NEXT; None produces PENDING with the complete boundary; Wait rolls back this entire event.

After a full-window handoff, reevaluate the new NEXT against already public cached results within the same Espresso(q), at the same x and pc, until no eligible extension remains. Successor assignment and its first acceptance have distinct ordered effects/lifecycle entries within the **one ReplayRecord** for that event. Source and effect order must be deterministic. Stop condition follows finite objects and strictly advancing physical/window heights, not a wall timer.

PENDING has no SessionEnd and accepts no results; Expire still targets h_G+1. A valid Expire closes the branch with reason Expired, saves the recovery boundary/session information, sets HALT, and preserves all G/source/selected/closed history. It neither deletes accepted G nor fills a missing block. HALT has no subsequent Expire/SessionEnd and no Espresso acceptance. Only future implemented RESUME can reactivate it; current Force/RESUME is explicitly unsupported.

ReplayRecord follows v10: `position,input_ref,effects,events`; effects are ordered `Append{first,last,source}` and `Duty{before,after}` in this stage; reserve `Reset{resume_position,target_height}` for future RESUME without executing it. One fully completed public event produces one record, even if it has zero accepted results or several regular handoffs. All lifecycle members carry that record's exact position.

## 9. Checkpoint and independent reader boundary

A GlobalCheckpoint contains the complete replay state, immutable parameters/config/profile/source identities, pc, selected/closed data, publication provenance/q_avail, records/events, anchor execution evidence, and supporting L1/Espresso cuts. A standalone head/hash or trusted JSON object is not a verified checkpoint. Init(checkpoint) replays from checked genesis to the saved pc using the supplied exact public inputs (or an independently established equivalent verification method), compares the entire canonical public projection, and checks supporting cuts. Missing verification inputs returns WAIT without an initialized state; mismatch, altered source/cut, forged q_avail/closed set/lifecycle or altered execution state returns Reject. No fresh empty state fallback. Fixture→Native promotion requires native revalidation, never profile relabeling.

For this stage a typed in-memory checkpoint is sufficient; if persisted, its versioned envelope must retain complete state and reject corruption/truncation/unsupported versions before exposing it. The independent verifier tests restored continuation through the next handoff, not just a serialization roundtrip. No 01 process crash/disk durability claim follows from this component checkpoint.

Each viewer owns an actual 06 `ReaderState` with distinct reader_id, an independent 03 fixture L1 reader, its own ReplayState, cursor, object cache, and download schedule. Their immutable source fixtures may be shared. Deliver public headers/messages using 06 receive_header/receive_message/receive_block; only read `FinalizedLogReader` complete prefixes/watermarks. A receives q faster than B without modifying B. Do not wrap a shared G, clone A's computed result into B, or let either reader inspect SimEspresso's undispatched data. Receipt tick and private `PERS` snapshots may differ; G-S1 projection at equal pc must agree.

Expose the actual rule API and deterministic snapshots to 08/14. A/B agreement alone is insufficient: independently derived source vectors check the complete transition list and outputs, including native execution data, for each reader.

## 10. Independent acceptance matrix

Each case records trigger, pre-state, delivered public objects, coverage, guard, expected next state/record/output and exercised assumption. Independent expected values must not call the reducer to compute themselves. Native-specific rows stay OPEN until 04 is available; all pure rows use an explicitly labeled fixture profile if needed.

| ID | Trigger/pre-state and boundary | Expected result |
| --- | --- | --- |
| V01 | M1 position/lifecycle/publication codecs; enum/index mismatch, max integer, bad/truncated/extra bytes, over-limit frame | Exact independently supplied bytes; correct mixed control ordering; typed errors; no allocations from unbounded lengths. |
| V02 | Genesis, Plan ready/None/Wait; wrong config/genesis anchor/state/reference | NEXT/PENDING/WAIT respectively; no fabricated pc/lifecycle; mismatches reject. |
| V03 | Next timer x, Espresso theta<x despite increasing q; then theta=x | No timer before watermark; after coverage, same public timer runs regardless of wall/receipt time. |
| V04 | Espresso/timer x; L1 header covers x−M_a but last block's control list missing; control in its last log | WAIT frontier, never treat header as complete controls; after list delivery control wins same-key ordering. |
| V05 | Valid selected result with full control coverage but missing later L1 origin/DEP/anchor state | WAIT(Some(original Espresso position)), no partial additions/events; filling actual input yields original expected transition. |
| V06 | Whole messages/header missing locally versus fully read proof-only q0, body first public q2 | Former is WAIT; latter allows q0 processing with no G append, becomes complete at q2. Cached unpublished body does not help. |
| V07 | Proof-first, body-first, missing actual physical ancestor, body digest/parent mismatch | Earliest full proof+ancestry q; missing public ancestor remains ineligible; known contradiction excluded; no missing ancestor converted to padding. |
| V08 | Two valid proofs for same ResultKey at different positions, opposite download order, extra late optional evidence | Same q_avail, sources and normalized proof set at same pc; no conflict from valid proof-byte difference. |
| V09 | Cached successor published before selection; exact anchor match and same-height wrong-anchor control | Original q_avail retained, current x used for acceptance; wrong cid/anchor never enters G. |
| V10 | Public completeness immediately before cutoff and exactly at cutoff; same-key SessionEnd/Espresso | Strict-before eligible; equality excluded/old session closed. No current wall clock affects historic proof validity. |
| V11 | A result expands padding plus physical target; any new h has expiry==x or <x; target's own expiry later | Reject entire extension/no suffix-only append. Integrated Expire precedes Espresso at equality and HALTs. Standalone guard test verifies every new h. |
| V12 | Final descendant covers several actual ancestors and padding; overlaps an earlier accepted prefix | Full body path checked; one descendant proof sufficient; contiguous suffix only; old sources retained, new heights share covering ResultKey. |
| V13 | h10 accepted; cached eligible successor h11 present in same Espresso block | Complete closure, fixed [11,20], production anchor h10, reference initial predecessor anchor; successor h11 appended at same pc. Repeat to two handoffs in one event. |
| V14 | h9 last physical then SessionEnd; separately no physical before SessionEnd | Production anchor h9 or initial anchor; [11,20] unchanged; no invented h10; next physical h11 may cover h10 padding subject to its expiry. |
| V15 | Plan needs material after an h10 append, or later same-block successor needs missing metadata | Entire current Espresso event staged and rolled back on WAIT; earlier committed event prefix preserved. Retry emits one record/batch, no duplicate closures. |
| V16 | NEXT/PENDING Expire; stale prior-session timer; HALT with later Espresso, Force and Resume | G preserved; branch closed once; no stale second handoff; no HALT acceptance; explicit unsupported control stop, never wake/skip. |
| V17 | Supplied E/L1 forks, theta regression, conflicting same-position controls, changed earlier message | Reject without using invalid extension. Equal theta advancing q is accepted. E-TS excess only limits timing claims. |
| V18 | Checkpoint at a pc; missing validation material; altered sources/q_avail/closed data/native state | Verified restore equals genesis replay; missing gives WAIT; any forged public-state field rejects. Restored and uninterrupted continuations agree. |
| V19 | Actual 06 A/B ReaderState schedules: ordered versus reverse/duplicate header/message delivery; L1 data delayed independently | Different private frontiers/receipts; same public projections at every common completed pc; no reader state leak. Source-derived checker agrees with each. |
| V20 | Multi-proof and candidate sorting permutations; two different lifecycles at one pc; exact retry | Deterministic target hash order and record/effect list, distinct lifecycle IDs, retry idempotence. |
| V21 | Missing verifier/basis versus fixture versus native profile, native input version mismatch, initialization with absent binding followed by matching/mismatching binding | Expected profile persists without capability; missing gives named WAIT for viable result, prerequisite Wait/Reject never calls bool checker; fixture artifacts explicitly noncrypto; native label alone never marks gate passed; mismatch rejects. |
| V22 | Actual 04 certificates, when delivered: 3 vs 4 distinct signers, duplicate/wrong signer, wrong cid/view/kind/target/domain, malformed bytes; certificate before full ancestry | Invalid final proof excluded; exact valid quorum verifies finality; notarization/nullification/progress never suffice; body delivery and G admission separately gated. **OPEN for M2.** |
| V23 | Fixture exhaustion and all public arithmetic/byte limits | Explicit bounded-input/overflow/capacity failure; never synthesize L1/DEP, wrap, silently truncate history or report complete progress. |

Minimum named G1 scenario: V03–V05, V13–V20 using independently delivered actual 06/03 readers. Minimum G2: V06–V12 and V15 with q_avail and x both printed, including ordinary state-changing transactions and physical gaps. Fixed fixture variants/seeds **7, 42, 99**, each under wall pause **0 and 2 ms**, select documented delivery schedules; the source public log stays identical. Compare canonical public traces byte-for-byte across pause pairs and against independent expectations. State finite height/q/L1 ranges, event and capacity bounds; report no wider liveness/safety theorem.

Build/test runs use the task's Rust 1.95.0, own manifest/lockfile, `--locked --offline`, `CARGO_BUILD_JOBS=2`, and one global `flock /tmp/megaroation-poc-heavy.lock` around each heavy command (never nested). Record actual commands, candidate SHA/dirty state, fixture/hash/seed, raw traces, expected/observed results and missing gates. Tester runs the exact independently verified integrated candidate; code fixes require affected re-verification before rerunning that matrix.

## 11. Open decisions and completion conditions

No unresolved protocol choice blocks this frozen pure stage. Independent source-first expectations are at `poc/verification/viewer-gfr/source-first-expectations-v1.md`, with literal semantic vectors in `literal-vectors-v1.json`; the Verifier supplies additive interface wire vectors for M1. Exact source pins and 03/06 boundaries above are available. The following remain explicit integration gates:

1. 04 native verifier implementation/witness format and its exact reviewed SHA; 07 neither invents a new final certificate nor counts fixture proof verdicts as crypto acceptance.
2. 09 authenticated nonempty L1 control adapter, with complete block coverage, exact positions and ABI/source validation. Current regular adapter verifies finite empty lists; control fixtures stop unsupported. Force handler belongs to 10, RESUME to 13.
3. 12/13 historical ResumeCut and lifecycle batch material staging/durable delivery, preserving source-listed records. The material-before-allocation decision in §4 must be exercised in that integration; it does not modify v10 text.
4. Publisher use of exported GFR-PUBLIC-1 codec and downstream use of GFR-POS-1/lifecycle projection require their consumers' compatibility checks; no second encoding may be introduced privately. Envelope size limits can exclude otherwise structurally valid maximal 03 bodies; report explicit capacity and version any future fragmentation extension.
5. Controlled service effects/Manager allocation, real producer regular success, observer SSE and official Espresso validation are separate owners' gates. This crate returns pure values and does not bypass grant/Ack to publish network or lifecycle side effects.

M1 completion requires a committed consumable API and independent vectors with exact revision references. M2 completion requires all applicable V01–V21/V23, independent candidate review, G1/G2 on that same candidate, exact raw artifacts and an honest OPEN native gate. Overall 07 stays open until its real-certificate and downstream integration acceptance cases pass. No automatic main merge or push is part of this contract.
