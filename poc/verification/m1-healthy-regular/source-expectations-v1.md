# M1 merge independent expectations — 2026-11-16

Status: source-derived verifier preparation; NOT a candidate acceptance report.
Checkout: `/home/jy/.codex/worktrees/5d7a/megaroation_poc`.
Observed checkout HEAD before candidate inspection: `35d382ac445cc8bc779f0d18fd23cb57e54680da` (M0).
Accepted dispatch: `d55d39f73593676325bdc0fbb7ba3845ca93025f:poc/docs/milestones/M1-implementation-dispatch-20261116.md`.
This file was written before reading any M1 candidate implementation, builder explanation, or new harness contract. Repository and frozen artifacts remained read-only. No build or simulation has run in this preparation.

## Source and profile lock

Read AGENTS, root DESIGN, v2 handoff/setup/design, v10 02/03/04/09, milestone README/M1, and accepted dispatch above. Applicable source locations below are checkout-relative.

The checkout gitlink is `regional_BFT_sim@691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`; its worktree is uninitialized here (`git submodule status` begins `-`). Read the exact pinned DESIGN object without modifying it:

`git -C /mnt/reth/jy/megaroation_poc/regional_BFT_sim show 691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6:DESIGN.md`

Pinned DESIGN identifies Commonware 2026.10.0 release `92a2f68`, Simplex paper eprint 2023/463 and the release's deviations. It does not identify a separate dated Simplex paper revision. For the executable Simplex profile, the locked release rules take precedence over the explanatory paper's 2f+1 shorthand. Read cached `commonware-utils-2026.10.0/src/faults.rs` and `commonware-consensus-2026.10.0/src/simplex/mod.rs` under `/mnt/reth/jy/regional_BFT_sim/.tooling/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`: f=floor((n-1)/3), q=n-f; n=5 gives f=1,q=4. Notarization is not finalization; certification is required before normal advancement/finalize, and its terminal predicate must be deterministic across honest participants. Missing/local admission conditions are not automatically global false.

M1 has 15 continuously live real worker processes, three independent committees of five, static schedule i=(beta+r)%3 and leader pid=5*i+(v-1)%5, equal weights, four distinct legitimate signers, TB=10. No OS restart, same-cid reconstruction, scripted remote voters, Force, RESUME, D/DA/batching, Web, or timing proof from the official node.

## Independent assertions and smallest rejecting traces

### E01 — process and identity provenance

Sources: v2 §§2.2,3.2–3.3,5.1–5.3; M1 main scenarios; dispatch reuse boundary.
Precondition: fixed run manifest and 15 public identities/PoPs. Trigger: worker admission, engine creation, signed output, exit.
Expected: unique stable `(run_id,node_id,OS_pid,process_start_identity)` for all node IDs 0..14 through the observation interval; R0={0..4}, R1={5..9}, R2={10..14}; engine instances and sign events exist in the appropriate processes; each emitted signature maps to that worker's registered key and controlled step. PID reuse cannot hide a restart. A worker need not sign every certificate or Enter every candidate, but all 15 stay alive and each region's five real engines are exercised.
Reject: duplicate OS process identity, only two real workers plus scripted signers, unexplained process exit, duplicate/reassigned key, signer outside selected committee, certificate signers absent from per-worker sign traces. Registry/PoP and raw proofs, not a summary `workers=15`, are evidence.

### E02 — cryptographic finality and domain

Sources: v2 §3.3; v10 02:240 (§3.10),319–340 (BFT-2/3/4); pinned DESIGN §§2.1,3.1,5.
Trigger: observed proof/Final/attempted G admission.
Expected: parse actual native proof bytes; independently rebuild namespace/context, subject/block digest, kind, view and ordered participant mapping; verify registered PoPs, signer bitmap uniqueness/membership and aggregate signature with threshold fixed at 4. Full-body ancestry and application validity are separate checks. Direct finalization or correctly linked descendant coverage is required. Shared audited crypto/codec primitives are permitted; candidate GFR result or a caller-provided `verified=true` is not expected truth.
Reject controls: three valid signers; four entries with one repeated signer; wrong region; valid notarization/nullification supplied as Final; flipped block/parent/body/view/cid/domain/config bytes; malformed/trailing encoding as specified by the frozen codec; invalid PoP. Exactly four and five distinct valid signers are positive boundaries. Actual command and artifacts await H1 verifier API.

### E03 — controlled effects, time and durability

Sources: v2:154–283 (§§4–5), especially :250; dispatch decisions 1–3.
Precondition: recorded scheduler input and immutable clock profile. Trigger: Grant → worker work → Ack → hub commit.
Expected: every route send, timer update, durable effect, signature/Final and cross-component publication handoff is attributable to one admitted step; hub commits complete Ack before advancing virtual time. Route/mailbox context admission is one monotone version/hash update preserving old queues/counters/custody. Signing state and safety guards are durable before the corresponding send becomes externally visible. No direct background send or output after Ack outside another grant.
Clock samples retain request/reply sequence, as-of tick, delivery tick, uncertainty, lease end and profile; grants do not renew leases. At every read/sign `abs(C_m-tau)<=1000us`; pairwise bound is 2000us, not 1000us. Manager and Program use the same C_m. RPC latency, public theta, wall pause and virtual tick stay separate. Work processing duration is explicit in virtual inputs.
Reject: orphan output; Ack missing output later emitted directly; signature before durable record; output duplicated as a new logical message; overwritten queues on cid admission; lease changed by grant; stale sample reused as current; wall elapsed added to protocol time; sign after ClockUnsafe or terminal guard.

### E04 — exact readiness, entry and candidate switchback

Sources: v10 02:170–192; 03:64–120,190–230,236–284; dispatch decisions 2–5.
Trigger: Prepare/Ready/Enter/Pause/SelectBack/Close for `(worker,cid)`.
Expected: recompute cid from immutable descriptor; anchor identity and metadata fixed; Ready requires actual post-anchor state reconstructed/validated from a verified checkpoint and ordered inputs. ReadyToken binds worker/cid/anchor/verification version. Missing materials WAIT; known mismatch REJECT. Initial Enter occurs exactly once after Ready, membership, local schedule and clock guards; repeated Enter returns original entry time. A logical session has at most one scheduled cid per worker.
A higher regional anchor may cause Pause but does not itself select a context globally. Resume of a paused candidate reuses original engine/future, mailbox, view/locks, signature history, callbacks and absolute timer deadlines. Include an actual in-process A→B→A switchback without OS restart; B may be a higher still-unpublished regional anchor. GFR selection excludes mismatches permanently; higher physical anchor already admitted to G also excludes lower impossible candidates.
Reject: root-only Ready, Ready bool from Manager, wrong anchor at same height, resetting view/deadline/signature history on select-back, second engine or second Enter, temporarily running excluded candidate while selected state downloads.

### E05 — closed means no new signing; old result remains publishable

Sources: v10 02:186–200,413–420; 03:216–259; 04:95–109,127–139; dispatch decisions 2–3,7.
Trigger: TIME/COMPLETE/EXCLUDED/ClockUnsafe, including Preparing and Paused entries.
Expected: terminal safety guard is installed before Close effect, never rolled back on later failure, and checked again at actual signing after asynchronous work. For each signature, local time is strictly less than stop=ts*(hmax+1), and no earlier Close/guard applies. Old pre-stop signatures may form a proof later. TIME/Pause alone do not discard Publisher tasks, mean public SessionEnd, or make a historically valid proof false. Only verified public exclusion/closure or deadline watermark authorizes publication stop (or finalized recording completes it).
Reject: signing at equality, late async callback signing after closure, closing only Running entries, local TIME deleting pending result, a failed action restoring Running, clock repair reviving terminated cid.

### E06 — slot, first receipt and execution

Sources: v10 02:194–246,319–329,422–455; v2 §§6.3,10.2.
Trigger: Build, first proposal receipt, Accept retry, parent use, Final, virtual expansion.
Expected: proposal height=slot(C_leader at Build), in immutable window and above parent; NotBefore honored; metadata timestamps/origin fixed by selected anchor and public inputs; root is pre-height-execution root after all intervening padding. Execute fixed DEP prefix and then ordinary txs with deterministic filtering, recording Kept/post-state/native hash. At least one kept ordinary transaction changes state in the healthy workload; empty-only traces are insufficient.
Accept boundaries: first_recv >= ts*(h)-2000us; first_recv < ts*(h+1)+T_win; first_recv < stop. If those hold but current C<ts*(h), WAIT(time), no vote and no global invalidity. Retry retains original first_recv. At least one honest ACCEPT precedes first use as parent or Final (BFT-2). Time validation of historical proof does not use observer current time.
Reject: late result manufactured with backdated first_recv; overwritten first_recv after WAIT; post-state root in physical root field; wrong parent/DEP/tx order/native hash; padding beyond supplied physical tip; same chain with two physical blocks from one view. Concrete encoding/transaction goldens need the accepted 03/H1 format; expected execution must not call candidate GFR/Merge.

### E07 — public completeness and independent readers

Sources: v10 04:45–69,95–109; 09:92–109,135–148,221–244,336–353.
Precondition: original finalized log bytes, each public object's `(q,message_index)`, and each reader's delivery events.
Expected: q_avail is the minimal public prefix containing >=1 valid target final proof plus every actual physical body from selected anchor to target. Public hash references resolve only to bodies with public origin positions. Additional proof/anchor evidence does not delay q_avail. A descendant proof covers ancestors only after the full linked target chain is available. Padding lacks a separate body obligation.
Publicly absent proof/body makes that result temporarily ineligible but need not stop unrelated events. An already-public object/L1 verification input missing locally requires WAIT at the original event; no cursor advance or permanent skip. A/B use separate replay states and distinct read schedules; compare all state/Rec/effects/lifecycle/source fields at equal positions, then compare against independent expected state. Different wall receive times do not change public keys or q_avail.
Reject: certificate-only G append; local cache counted as published; missing physical ancestor treated as padding; q_avail assigned by reader arrival/selection/last optional proof; advancing past locally missing verified input; A/B same shared mutable state; heads-only comparison presented as independence.

### E08 — public event order and G admission

Sources: v10 09:111–148,177–244,314–422.
Expected: independently merge public inputs by `(key, category, index)` with controls < Expire < SessionEnd < Espresso. M1 control fixture is empty, but its verified complete L1 prefix still covers x-M_a; all actual metadata/DEP inputs must also be available. For each G append, current selected cid matches, context is open, proof/chain and fixed window valid, public completeness key is strictly before session end, and current acceptance key x is strictly below Expire(h) for EVERY new virtual height including padding. Append is all-or-nothing and uses deterministic extension hash order; re-evaluate after G grows. Recurse into new NEXT within the same Espresso event when it has cached qualified extension.
Only accepted Espresso extensions change G in M1; SessionEnd/Expire/Manager clocks do not generate blocks. With no RESUME, G is contiguous and append-only. Source covers every appended height. Missing earliest padding height cannot be skipped even if final target height itself is unexpired.
Reject: local clock triggers public handoff; q increases at frozen theta and triggers future timer; h deadline equality accepted; target-only expiration check; partial append then WAIT/fail; incorrect tie class; late/stale SessionEnd executes twice.

### E09 — full-window R0→R1→R2 oracle

Sources: v10 09:246–263; v2 §§3.2,6.3,9.1,12.3; milestone M1.
Healthy full-window profile fixes beta=0 and sessions r=0,1,2, regions 0,1,2, windows [1,10],[11,20],[21,30]. Declared successful full-window trace must actually produce/admit predecessor h10 then h20 physical blocks; it cannot merely increment a session after ten views or count ten produced blocks with moved heights.
At h10 admission: close (0,0) COMPLETE, anchor exact h10 identity/post-state, r→1, fixed [11,20]. At h20: close (0,1), anchor exact h20, r→2, [21,30]. Reference block for successor Plan is predecessor's INITIAL anchor, which differs from production anchor. After exact selected R2 result (normally h21 under the frozen healthy workload) is actually appended to independently reconstructed G, regular success holds. Regional Final/submitted/recorded/SessionAssigned alone are insufficient.
Reject: hmax chosen by block count or view, successor starts anchor.h+1 when predecessor window ended higher, reference block silently replaced by final production anchor, mismatched anchor identity at equal height, success before R2 G accept.

### E10 — SessionEnd main case h9→h11

Sources: v10 02 §3.7–3.8; 03 §4.3–4.4; 09:92–109,221–263; dispatch decision 9.
Freeze a legal input schedule where R0 h9 is admitted and h10, whether produced or not, is not completely public before `e0=ts*(11)+T_grace`. R1 candidate anchored at exact R0 h9 is prepared using real state; its h11 is built/received/finalized in the legal h11 slot BEFORE selection and cached (possibly publicly complete already). This may coexist with pause/higher-candidate/select-back when that higher predecessor result has not entered G.
Before public coverage e0, local elapsed time alone does not hand off. At SessionEnd(e0), G remains through h9, predecessor closes COMPLETE, successor selects exact anchor h9 while retaining window [11,20]. At a following Espresso event (possibly same key e0, after SessionEnd), accept cached h11 only if original q_avail<successor e1 and EVERY newly appended h in {10,11} is unexpired at CURRENT acceptance key. Compute padding10 and physical11 with successor's anchor-origin/execution rules; G sources for both refer to the h11 result. No second publication or fresh signatures are required solely because the candidate became selected.
Reject: G adds padding10 at SessionEnd itself; changed window [10,19]; h11 first generated after e0 despite slot no longer 11; overwritten receipt time; prior candidate kill despite no public impossibility; using q_avail instead of acceptance key to evade expiration; accepting old R0 h10 at/e0 to overwrite padding10.

### E11 — separate late-slot/padding control

Same source. If no legal preproduced h11 exists, first successor physical result can be at the actual legal Build slot, e.g. h14. Keep [11,20], anchor h9; only this result's verified extension can append padding10..13 plus physical14, subject to all-height expiration. Report this as the late-slot/padding case. It does not discharge E10's h9→h11 claim.

### E12 — complete replay batch and commit semantics

Sources: v10 09:361–392; 03:216–242; dispatch decisions 2–3.
Trigger: one GFR ReplayRecord with multiple lifecycle events at the same position, including close+assignment.
Expected: Manager preflights complete source/structure then durably accepts all obligations together: terminal guards/tombstones, Selected+Preparing, cursor and immutable action intents. Identity includes position+kind+body, not position alone. Missing successor execution material delays Enter, not known valid closure. `Committed(actions_pending)` is distinct from `ActionsApplied`; H3 replay pc does not roll back because Manager execution lags. A post-guard action failure preserves safety guard and pending work; no forged receipt or silent cursor loss.
Reject: dedup by position drops assignment/close; cursor advances without close obligation; unknown record marks committed; missing Ready suppresses old close; replay retry duplicates Enter; rollback of durable closed state.

### E13 — deterministic replay and official readback

Sources: v2 §§5,12.2,13.3; milestone M1; dispatch decision 10.
For each fixed `(scenario,seed,startup profile)`, compare wall-pause 0 vs 2ms and repeat canonical virtual trace. Exclude OS PIDs/wall durations from canonical digest, but preserve them separately; include virtual ticks/local clocks/signature/proof bytes/route payloads/public positions/lifecycle/G source/native hashes. Different seeds or genuinely different virtual startup order need not have byte-identical traces; they must each satisfy the oracle and declared endpoint. Required seeds are 7/42/99; exact startup profiles, finite cutoff/event budget and processing/link bounds must be frozen before formal runs.
Export byte-identical actual worker-originated regional/publication result, its run/result/proof references and hash. Existing official runner submits and reads back those bytes; compare full bytes/hash and reverify regional proof independently. Official wall timing and API evidence are separate; PE time remains Unknown/Unsupported and QC/VID unverified. Marker-only readback or fresh synthetic proof is not M1 readback evidence.

## Minimal checker and artifact interface (proposed, not duplicate public encoding)

Use a JSON/JSONL evidence envelope around EXISTING raw canonical protocol objects; do not define a second RegionalResult, Plan or proof codec. Bytes may be content-addressed files, with hashes and exact source offsets; reject missing evidence needed for claimed checks. Recommended command surface:

`check-m1 --manifest RUN/manifest.json --raw RUN/raw --trace RUN/trace.jsonl --expected-profile PROFILE --report REPORT`

Modes: `check` (all assertions and scope), `compare` (same input/profile across wall pauses), `mutate-selftest` (derived temporary evidence only), and `check-official` (exact actual-result lineage/readback). Never mutate candidate/frozen input artifacts. Return nonzero on FAIL; missing candidate interface/evidence is INCONCLUSIVE/OPEN, not PASS. Per-check output includes source rule, source/candidate location, input IDs, derived expected/observed values and minimal replay command.

Required envelopes:

1. Manifest: exact candidate/integration/component/API/contract SHAs, source hashes, dirty status, tool/lock versions, seeds per role, scenario and input hashes, genesis/config/registry+PoPs, complete parameters including units, queue/size/timing bounds, finite cutoff, observed processes, commands and artifact digests. Do not persist private signing keys in review output.
2. Worker evidence: process identity and lifetimes; descriptor/engine immutable IDs; Create/Ready/Enter/Pause/Resume/Close; ReadyToken plus checkpoint/input bytes; before/after logical state references sufficient to verify same-instance continuity; receive-first and Accept/work records; sign kind/view/subject/raw signature and durable step ref.
3. Runtime evidence: grant/event/worker/tick/input; Ack with complete outbound/timers/durable roots; hub commit; route/mailbox/custody IDs; clock sample delivery/lease/read observations; context admission revisions. Identity must join stages without trusting matching display labels.
4. Result/publication evidence: exact proof/body/chain/anchor-source bytes, proof-observed vs complete-delivered times, stable result_key, offer/attempt/recorded/stop evidence; public object bytes and original positions; bounded transport chunk/attempt identity if used.
5. Public/replay evidence: complete finalized SimEspresso and static finalized L1 fixture prefixes, reader A/B received positions, observed ReplayRecords/state/source maps and WAIT positions. Independent checker derives sorted events, q_avail and G from raw inputs before comparing observed output.
6. Lifecycle receipt evidence: full record identity/hash, prepared ManagerDelta/intents, committed receipt/cursor/guard references, each action applied/pending outcome, per-worker correspondence.
7. Execution evidence: fixed DEP/workload+initial account state, actual physical chain, verified checkpoint/anchor-state origin, Kept and native bytes/post-root per height. Independent execution expected implementation/handwritten vectors must not import production GFR/Merge transitions.
8. Official evidence: actual run/result artifact provenance, submitted/read-back raw bytes+hashes, API profile/command/endpoints and trusted local-source evidence, separate wall events and explicit Unknown/QC-VID limits.

## API gates for the lead

H1: exact healthy-session-api commit and adapter/CLI; worker process ownership and trace types; actual native FinalVerifier/namespace/profile/PoP/participant mapping; sign/Accept/Ready/engine/timer/clock/grant/Ack witnesses; pause/select-back; dynamic route/mailbox admission receipts; bounded result channel and legal slot workload. Verify local WAIT/late admission versus deterministic Commonware certify behavior on the locked source before calling H1 safety/liveness closed.

H2: exact typed commands and ManagerDelta/action-intent schema, ReadyToken consumer, entire ReplayRecord event identity, committed/actions-applied receipts, candidate selection and same-instance pause/select-back semantics. No speculative Rust signature or duplicate API until H1's interface freezes.

H3: raw publication codec/export and exact bound, public-position/completeness evidence, independent reader/GFR input-output trace schema, verified stop source, production H1 verifier binding and public endpoint routing, official artifact-input command. No production always-true verifier.

Merge lead: exact profile/numeric timing/finite bounds/startup pair and actual candidate SHAs; one real integration writer. Input coverage must support earliest padding deadlines. Freeze what per-worker GFR/Manager state is instantiated (v2 specifies per-worker plus independent observers), and do not substitute a shared object. None of these gates justify claiming a future implementation completed.

## Scope, limits and replay plan

This preparation checks source consistency and defines assertions; it supplies no executed candidate trace, 15-process proof, cryptographic test result, throughput result, or protocol proof. Primary healthy runs and the delayed-publication SessionEnd run have different publication-liveness assumptions. Process health does not imply a fixed common-cid quorum-ready interval; progress budgets start only when a fixed set of >=4 has entered, has materials, is continuously scheduled/healthy and has enough time before stop-delta. First Enter and total online count across split candidates are invalid timing origins.

Negative checker self-tests should corrupt a COPY under `/tmp`: signer count/domain; PID-to-sign provenance; orphan Ack output; late signature; duplicate Enter/reset deadline; wrong anchor/state/window; missing/public-late body; altered q_avail; missing L1 prefix; early/tied SessionEnd; all-height Expire violation; lifecycle batch event omission; altered execution root; official byte mismatch. Cryptographic negatives must actually run crypto verification, not depend on a precomputed boolean. Keep candidate/frozen inputs immutable and record mutation descriptions/hashes.

No heavy task has run. Future build/simulation uses `flock /tmp/megaroation-poc-heavy.lock` and jobs=2, after exact candidate delivery/verification. Formal simulator must rerun agreed matrix on the independently verified integrated revision and record command, seed, traces, bounds and open assumptions.
