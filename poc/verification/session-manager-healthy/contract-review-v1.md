# H2 preparation draft — independent contract review v1

2026-11-16. Reviewer: assigned bft_verifier, GPT-6 Astra / xhigh.

**Final disposition: suitable for preparation-only delivery at final draft2 SHA-256 `837ed8f317e890c3b7a8e393f768cc69c62f7bcf4cec54790816ccc311340754`; implementation-contract freeze remains OPEN.** The small draft1 clarifications below are resolved in that final document; the H1 semantic obligation is explicitly OPEN rather than asserted satisfied. This is a document/source review. No candidate implementation was reviewed, no test/build/simulation was run, and no H2/native/M1 PASS is asserted. The draft correctly labels itself DRAFT / NOT FROZEN / NOT IMPLEMENTATION AUTHORIZATION.

## Reviewed artifacts and source-first order

1. Independently derived and sealed before reading the new draft:
   - `independent-expectations-v1.md`: `11f745e81faa46c03d85d2ee1ed722a842f8631ef62e96dd556e8610cece7ceb` — 30 planned acceptance cases.
   - `hand-vectors-v1.yaml`: `9bda2836ba7a2029e53f79c070a49cb83a217d60f21e8e7e893c367bf7337fbf` — 11 hand-authored vectors, all NOT RUN.
   - `source-manifest-v1.json`: `45cb6839949ad32bdd2284a7d90b22adebcda345d0482bc3d59deabb3af37477`.
2. After notifying lead of those hashes and receiving permission:
   - `/home/jy/.codex/worktrees/a6cf/megaroation_poc/poc/docs/contracts/h2-session-manager-healthy-preparation-v1.md`, 234 lines, SHA-256 `bf7f9e8da4242e40837b5657d59d2c7358d204f0c8a29cb183a8aab3efc7bb11`.
   - `/tmp/h2-session-manager-healthy-preflight/handoff-draft.md`, SHA-256 `6e2df782691e02f20982bcc81a3e429598628ec0d00a82c36fe0615527f21058`.

Frozen independent artifacts were not changed. Candidate implementation files, native prototypes and candidate diffs were not opened. The draft's implementation-location inventory at lines63–90 is the Docer/lead's read-only inventory; this review checks its contract meaning against accepted O3/O7 documents, and does not independently certify those implementation line references.

## Findings and clarifications

### CR-01 — clarify lifecycle_id's typed argument

- **Location:** draft line88; related interface inventory line67.
- **Source rule:** accepted `788291735ce6db3c3eb78b4594a28d730cce71cd:poc/docs/contracts/task-contract-viewer-gfr-v1.md`:64 declares `lifecycle_id(value: &LifecycleEvent) -> Result<Hash32, GfrError>`; lines123–134 define the full event projection and hash preimage.
- **Finding:** “lifecycle_id 的输入是现成 projection” conflates the existing function's typed argument with the bytes it internally hashes. The projection is the canonical hash preimage; the public API consumes the event.
- **Impact:** a consumer could request an unnecessary bytes API, bypass the checked event projection, or create a duplicate lifecycle hash implementation. This is an interface-description mismatch, not an observed implementation defect.
- **Minimal correction:** say “调用既有 `lifecycle_id(&LifecycleEvent)`；该函数按既有 `lifecycle_projection` 生成完整 projection，再使用 `PoC/GfrLifecycle/v1` 与既有 framing 计算身份；另保存完整 event/projection 供审计。”
- **Replay/evidence:** no execution. Hand vector `V-LIFE-SAME-PC` independently specifies the literal position/SessionClosed projection and two distinct same-pc events; subsequent contract tests must pass typed events to the existing API and compare independent projection bytes/identity behavior.
- **Disposition:** clarify before interface freeze; does not prevent an explicitly labeled preparation delivery.

### CR-02 — public selection must permanently exclude B

- **Location:** draft line104: “GFR 后来选 A…Pause B/排除 B”; draft lines148 and178 already give the correct Excluded/Terminated behavior.
- **Source rule:** v10 `03_session_manager_zh_v0_1.md`:224 requires mismatching same-session candidates to become Excluded and Close(EXCLUDED); lines263–265 distinguish preselection local Pause from permanent exclusion by public choice.
- **Finding:** the slash wording permits reading Pause and exclusion as interchangeable after GFR has fixed A.
- **Impact:** implementing only Pause could retain B as resumable after its public eligibility ended, weakening the terminal/candidate guard. Other draft clauses contradict that reading, so this is a local ambiguity rather than an overall source disagreement.
- **Minimal correction:** “GFR 后来选 A 时，将 B 置为 Excluded，并先持久 Terminated 再 Close(EXCLUDED)；若 A 未终止则恢复原 A 的调度。只有公开选择之前的本地偏好切换才将仍可能获选的旧候选置为 Paused。”
- **Replay/evidence:** no execution. `V-ABA` expects B Terminated/Excluded on the public selection step and A's exact prior handle/history; `V-SELECT-BEFORE-READY` expects the same exclusion even while A remains Preparing.
- **Disposition:** resolve wording before implementation freeze. The already-correct A05/§6 assertions should remain.

### CR-03 — explicitly carry H1's local-admission/deterministic-validity obligation

- **Location:** draft lines90,104,159 and223–224; current O1/O2 cover native interfaces and retained pending work but do not explicitly list the validity-mapping acceptance obligation.
- **Source rule:** pinned Regional BFT `691cfc14...:DESIGN.md`:47 requires distinction between node-local receive-time admission and context-wide deterministic application validity, checks BFT-2 before first parent use/decision, and says the production adapter needs its own conformance/safety argument. Commonware2026.10.0 `consensus/src/simplex/mod.rs`:108–115 preserves unfinished certification and requires consistent deterministic certification for liveness. v10 SP:215–230 gives local first-receive/time guards, and SP:321–323 requires successful ACCEPT before chain entry.
- **Finding:** H2 correctly delegates native semantics to H1, but an interface SHA and retained callback handle alone do not establish that H1's local-time refusal/pending behavior preserves these upstream properties.
- **Impact:** an H1 adapter may preserve A→B→A mechanically while encoding local admission as globally deterministic invalidity or discarding certification work incorrectly; this can undermine the claimed native safety/liveness evidence. No such faulty implementation was inspected or alleged here.
- **Required freeze gate:** require an accepted H1 mapping of local ingress/firstSeen/clock admission versus `verify/certify` terminal outcomes, including Pause/Close/stop, and evidence that BFT-2 is enforced before parent use/decision on all applicable native paths. Missing/time WAIT must not silently become inconsistent terminal invalidity. Assign to H1/protocol owner; H2 consumes the accepted boundary rather than adding a new signing/consensus rule.
- **Replay/evidence:** no execution. Independent H2-09/14/23/25/28 plus V-ABA/V-LATE11 expose boundaries; exact native vectors depend on the delivered H1 contract. Record unresolved mapping as OPEN/INCONCLUSIVE, never resolve it using a fixture boolean.
- **Disposition:** not a defect in preparation-only scope; explicit safety/liveness assumption escalated to lead and blocking native-interface acceptance until resolved by H1.

### CR-04 — keep subsequent implementation authorization explicit

- **Location:** draft lines5 and234; handoff draft line61.
- **Source/scope:** this assigned round permits documentation and independent verification preparation only, and forbids production modification/builds/simulation. The lead's current workflow also requires M0/H1 delivery, contract freeze and subsequent implementation dispatch.
- **Finding:** “齐备后，lead 才可冻结并交给 Builder” and “实际编码等待 G1/G2 和合同 freeze” can be made unambiguous that satisfying dependencies does not itself start a Builder in this preparation task.
- **Impact:** prevents a preparation artifact being read as automatic permission to begin production work during the current restricted round.
- **Minimal correction:** add “并在后续明确实现授权/dispatch 后开始；本轮保持只做准备”。
- **Replay/evidence:** not a protocol test; review observed only source/document/hash operations. No Builder or Simulator activity was performed by this reviewer.
- **Disposition:** scope clarification for delivery, not a protocol source conflict.

## Source-consistent decisions retained

- Actual ReadyState, exact anchor identity and immutable descriptor are kept distinct from typed decode, matching root, source pointer or caller boolean.
- Selected+Preparing is legal. A pending state download cannot delay a verified public close or permit production by the excluded alternative.
- A→B→A preserves the original engine/entry/view/votes/signatures/deadlines/first-receive/pending work. Both F and actual native gate are present; no fixture claim substitutes for genuine native behavior.
- Closed-scope tombstones are labeled adapter choices implementing source-level permanent closure; Preparing and Paused cannot escape close through late completion.
- Whole ReplayRecord atomic consumption, private action identities, durable intent/receipt/Ack ordering and cursor meaning are clearly proposals awaiting H1/H3 acceptance. Multiple lifecycles at one pc, empty-event records with effects, and same-pc multihandoff are retained.
- Historical proof verification and old task delivery remain independent of stopping new signatures. Actual public exclusion/cutoff may stop new retries; neither Close nor Pause deletes the recorded result/task.
- M1 endpoint remains R2's first eligible result in G through authentic 15-worker→Publisher→SimEspresso→GFR operation. Official Espresso uses the same bytes in a separate compatibility report.
- M2 Force is an input reservation, and D/RESUME/batch/DA/OS restart/Web remain outside this round. Ordinary persistence and signing/grant guards are not waived.

## Numerical examples are intentionally separate profiles

There is no source conflict between draft line166 and sealed V-SE9:

| Quantity | Independent V-SE9 | Draft's illustrative profile |
| --- | --- | --- |
| ts*(h) | h seconds | 10 seconds + h seconds |
| predecessor stop | 11s | 21s |
| SessionEnd public key | 14s | 24s |
| successor window | 11–20 | 11–20 |
| h11 first-receive interval (δ=1ms, T_win=200ms) | [10.998s,12.2s) | [20.998s,22.2s) |
| successor stop | 21s | 31s |

The draft is a 10-second translation of the same absolute-window logic. Neither profile is an accepted native performance configuration yet. Future traces must bind their actual time origin and parameter hashes, not mix 14s and24s in one run. In both, waiting until SessionEnd before newly producing h11 fails its original first-receive deadline. Later legal physical heights under the same fixed window remain possible and do not substitute for the required h9→h11 evidence.

## Freeze gates that remain OPEN

1. Exact M0 integrated SHA, component provenance and compatibility evidence; current preparation HEAD is not M0.
2. Accepted H1 interface SHA/ownership and real Create/Prepare/Enter/Pause/Resume/Close lifetime/identity contract; pending-work/deadline and local-admission obligations above.
3. Precise ReadyState evidence schema/trust anchor/03 execution binding, checked bounds and missing-versus-invalid behavior.
4. ClockUnsafe local reason and scope across Running/Paused/not-yet-entered contexts; unchanged O3/O7 public enums absent owner-approved versioning; recovery cannot revive terminal cid.
5. H3 full committed-record provenance/continuity, same-position views, lifecycle projections, source acknowledgement and old-result Offer/Recorded ownership/receipts.
6. H1/H2 durable state/terminal/scheduling guard/action-intent transaction, effect release and full grant/Ack; a list of fsync calls alone is not an atomicity argument.
7. Accepted finite case bounds, actual native engine configuration and SessionEnd schedule, per-cid trace observability, independent checker and exact run commands/seeds on named candidates.
8. Subsequent explicit implementation dispatch and new independently reviewed candidate. Component/native/M1/official compatibility gates remain distinct.

## Scope and verification limits

Checked: source consistency, accepted O3/O7 interface-contract semantics, lifecycle/admission/publication boundaries, hand-derived numeric translation, gate labels and draft hash. Files and source lines above locate document issues, not candidate defects. No concrete replay was run; all replay references are proposed hand vectors and require later authentic execution evidence. Finite fixtures, actual native traces and integrated M1 results remain future work.

The independent expectations, vectors and source manifest remain sealed at their listed hashes. Any later source/API changes require a versioned contract review and explicit amendment rather than silently changing these expected outcomes.

## Draft2 follow-up (same review, source oracle unchanged)

Read the lead's revised `H2-SM-HEALTHY-v1-draft2`, SHA-256 `0b997022650afadd4e63108218898e7113829d35ec2623e9271799a18bfdb9c0`, after the draft1 findings were reported. Line88 resolves CR-01. Lines5/235 resolve CR-04. O9 at line231 explicitly records CR-03 and is connected to G2/G4 at lines214/216; its substantive acceptance is still OPEN pending H1 evidence, as required.

CR-02 is almost resolved at line104, but the wording “若 A 未终止，则将 B 置 Excluded+Terminated 并关闭，恢复 A 调度” conditionally binds B's exclusion to A being nonterminal. Public exclusion of B is required even when selected A is already TIME-terminated (draft line148 already correctly allows Selected+Terminated). Requested final wording: “GFR 后来选 A 时，将 B 置 Excluded+Terminated 并关闭；若 A 未终止，则恢复 A 调度。” This is a scope clarification to the same CR-02, not a new protocol requirement.

### Final draft2 read-only check

The final file SHA-256 is `837ed8f317e890c3b7a8e393f768cc69c62f7bcf4cec54790816ccc311340754`. Line104 now excludes/closes B unconditionally when public GFR chooses A, and separately conditions A's resumption on nontermination. CR-02's ambiguity is resolved. CR-01/CR-04 are resolved; CR-03 is correctly recorded as OPEN O9, linked to G2/G4. No remaining source conflict found that blocks a preparation-only delivery. M0/H1/H3 exact inputs, O1–O9 decisions, actual native/fixture/system execution and later implementation authorization remain pending.

Recomputed hashes of the three sealed independent files match the original values above. The final check used only text reads and SHA-256; no candidate code or executable fixture was run. Review history retains both draft1 findings and the intermediate draft2 clarification for audit.
