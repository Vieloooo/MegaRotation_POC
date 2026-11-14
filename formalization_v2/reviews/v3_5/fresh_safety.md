# Summary

**Final Verdict:** The formal conditional safety arguments are viable: this independent review found no in-model conflicting-commit, global-prefix rollback, or cache-dependent replay counterexample. The packet has one nonblocking specification-consistency **Justification Gap** in a retained figure. Regional retirement safety remains conditional on the explicitly stated complete Ready/Enter handler contract; this review does not certify a concrete native adapter.

**List of Findings:**

- **FS-1 — Justification Gap; normative inconsistency, not a demonstrated protocol fork.** **Location:** `snapshot/figures/sync_hotstuff_rr_rotation_v3_1/observer-finality.tex:81–84`: “Either Force … opens session $(\beta+1,r_c+1)$.” This general statement in panel (c) disagrees with the zero-accepted-DATA retry rule in `snapshot/sync_hotstuff_rr_part2_rotation_v3_5.tex:1844–1868` and Protocol 5 at `snapshot/sections/rotation_v3_5_control.tex:235–244`. The precise formal rule reuses `r_c` after zero accepted DATA and retains the stored target when unassigned. The figure is actively included at main lines 2486–2493. Its numerical panel (b) is valid under the main text's explicit `beta=0`, positive-output example, but that does not make panel (c)'s universal target formula correct. Consequence: a reader implementing the figure obtains a different target authority after a zero-output Force. I do **not** count that implementation as a compliant execution of the explicit Protocol 5 rule, or propagate this presentation inconsistency as a proved failure of the prefix theorem.

No safety **Critical Error** was established. No additional substantive proof gap was found in the conditional safety/isolation/retirement branches reviewed below. Source-step latency, global progress, repeated regional service, and request inclusion are separate conclusions; the discussion below checks their safety interfaces without issuing an independent complete liveness certification.

All paths below are relative to `formalization_v2/reviews/v3_5/`. `main` abbreviates `snapshot/sync_hotstuff_rr_part2_rotation_v3_5.tex`; every other cited name identifies its exact snapshot file.

## Input identity and review method

- Read `review_scope.md` and the proof-verifier skill before reviewing; no other reviewer report or prior review conclusion was read.
- SHA256 checked all **20** files in `manifest.json`: no mismatch. Main SHA256: `2f47bca40d7ebdf80bb68a49a5fb8c479c4a50484a753d0134defc9381831d6b`.
- Compared the v3.1 main source with v3.5, including the baseline's handoff, bootstrap, safety, progress, implementation, and reference paragraphs outside Model. The Model section is byte-identical; that check was not used as a semantic correctness verdict.
- Read the inherited Part I model and timer/lock safety chain, the active v3.5 control/source/Ready/health fragments, and the retained retirement proof. Inspected the included figure source where it states control semantics.
- This was a source-level adversarial proof review. No manuscript/snapshot edits, protocol repair, model checking, or executable implementation certification were performed. A small arithmetic enumeration checked only the empty-window shortcut (470,400 integer parameter cases); it is not a protocol test or proof.

## Dependency map

| Claim | Necessary dependencies | Review disposition |
|---|---|---|
| One fixed handoff per pair; one initial entry | Immutable `AUTH[r,g]`, complete historical boundary verification, deterministic selection, fresh beta domain, serialized durable entry flag | Checked conditionally; no conflicting authorized pair construction found |
| Same public replay result at the same completed source position | Complete canonical L1/Espresso input, canonical record-completeness basis, deterministic prelude/debt/window transitions, atomic EXPORT acceptance | Checked; local caches and future-control queues are correctly excluded from equality |
| Preservation of accepted DATA and provenance | Exact expected base, contiguous append, strict pre-payload closure, closed-beta rejection | Checked independently of local program phases and regional liveness |
| Regional safety while retiring | Complete Ready/Enter contract; phase-independent complete-close relay; evidence-first handling; unchanged timer/lock rules; receipt-to-timer provenance | Checked under the stated abstract contract; a native adapter with residual unbounded installation is outside the proved branch |
| Native execution/DA correctness | Sound native validation and publication interfaces | Assumed interface, not proved here; input-prefix finality is not promoted into execution finality |

# Detailed Verification Log

## 1. Frozen assumptions and inherited scope

**Text checked:** “Every regional certificate needs … total weight greater than `F_i`” and “correct members alone can form one” (`main:359–377`); “The delay bound includes delivery and processing of complete validation dependencies” (`main:391–403`).

The threshold argument uses `W_i-F_i>F_i`; it does not require quorum intersection between arbitrary certificates. The synchronous forwarding/timer argument must provide safety instead. I retained the stated fixed-session weights, authenticated channels, binding hashes, durable correct state, and complete-message processing bound. A permanently absent correct quorum or a correct-to-correct complete message delayed beyond `Delta_i` is not an admissible attack under these premises.

**Text checked:** “A verifier obtains a complete control prefix” (`main:383–389`), advance announcement (`main:496–507`), compatible Espresso prefixes and retrievable namespace evidence (`main:442–478`).

These premises exclude a verifier legally confirming an interval using a cherry-picked control subset. The timestamp upper-lead bound and future announcement imply visibility before the crossing becomes final. Deliberate use of an omitted applicable control would violate the verification premises, not demonstrate a legal local-arrival ordering.

**Baseline check:** `snapshot/baseline/sync_hotstuff_rr_part2_rotation_v3_1.tex:2103–2125` explicitly retains fair schedules, exclusion, uninterrupted commit windows, eventual startup dependencies, and the distinction between delivery and useful acceptance. Part I's authority/fairness statements are at `snapshot/baseline/sync_hotstuff_rr_part1_core.tex:173–223`; its safety chain is at lines 735–830. These were considered separately from the byte-identical Model section. Fair scheduling and successful service intervals are not used as substitutes for safety.

## 2. Authority reuse, beta/g/t separation, and DATA/DRAIN validity

**Text checked:** “A new branch version that reuses `r` uses the same authority” (`main:254–268`); signatures bind the session, configuration, region, and message type (`main:785–809`); “QCs are ranked only within the same session” (`main:811–830`).

A retry of `(beta,r)` as `(beta+1,r)` cannot reuse a vote, Quit, entry lock, or timer as new-session evidence. The message domain changes even if the membership and leader schedule do not. Authority caching at `main:1810–1819` explicitly fetches evidence without admitting a new generation; `main:1870–1876` keeps the schedule beta-independent while preserving fresh core state. An old-beta Quit therefore cannot move the new engine directly to an advantageous recovery generation.

**Text checked:** Fixed endpoint and DATA/DRAIN rules (`main:688–745`), base justification (`main:822–830`), and receipt coverage (`main:887–901`).

The new initial endpoint is computed from the exact retained anchor plus the unchanged slot quota. Same-slot reuse can therefore change the endpoint without rebinding the authority; this is not a quota reset inside one session. Covering evidence remains attached to the original producer and justifies exactly the retained ancestor. It does not import the covering descendant as the new anchor, import its QC into the new rank order, or treat DRAIN as application DATA.

## 3. Regional receipts and inherited timer rules

**Text checked:** A correct FINAL is sent only after a timer or ancestor commitment (`main:832–884`); the Vote step starts exactly one timer (`main:1109–1139`); Completion relays the complete receipt, commits terminal ancestry, and then cancels timers (`main:1158–1170`).

The original regional invariant needs more than receipt signature syntax. The strengthened complete-receipt relay is relevant to the retirement proof because a receipt-based Completion can suppress votes and timers. The rule now disseminates that cancellation cause before using Completion; it is not merely a best-effort publication action.

The unchanged DATA/DRAIN parent rules and fixed-position assignment retain the Part I chain-or-equivocation argument. Genuine Quit cancels a generation's timers; ordinary term rotation does not. Force closes the fresh-action gate but does not itself cancel existing timers, rewrite locks, or manufacture a Quit/Completion event (`main:1005–1059`, `main:1188–1317`).

## 4. Canonical source completeness versus private caches

**Text checked:** “The public dependency basis consists of the authenticated source prefix strictly before `p`” and the canonically identified authorized checkpoint (`snapshot/sections/rotation_v3_5_source_step.tex:12–28`); “A private cache hit cannot make an otherwise incomplete EXPORT valid” (lines 30–49).

The two required cases are distinct:

1. A publicly referenced dependency exists in the canonical basis but is absent locally: the verifier fetches it and resumes the same source position.
2. A candidate omits a nonpublic required proof or names a future/nonpublic object: the complete namespace contains an invalid candidate, which is rejected without fetching arbitrary future material.

Thus two readers cannot legally obtain different acceptance results merely because one privately cached an authority or proof. A newer local checkpoint also cannot broaden an old record's public basis. Historical source authentication still does not replace regional/application validation.

The shallow domain/base/open-version/append checks precede deep optional archival validation. A stale closed-beta record therefore cannot hold current replay hostage by demanding arbitrary old context. Genuine missing source history or a canonical boundary authority still causes a wait; this is correctly not reclassified as candidate failure or zero weight.

**Dependency limit:** This reasoning relies on the declared common checkpoint/dependency identities and sound validation interface. It does not certify an encoding or checkpoint-authorization implementation that has not been supplied.

## 5. Source-step state mutation and closure delivery

**Text checked:** “The handler … does not change the canonical order” and “does not insert another … replay job” (`snapshot/sections/rotation_v3_5_source_step.tex:4–10`); receive/drain/activate at lines 88–125.

SOURCESTEP supplies authenticated material and runs P6/P5 in contiguous source order. It does not accept a sender's new checkpoint as authoritative, use receive time as the opening timestamp, or give the Session Manager a second beta-increment path. Duplicate source packets do not reset windows or initial entry.

The version-wide Closed gate is persisted without waiting for an old-worker scan. Crucially, the same fragment preserves the complete-close handler and multicast when old-session context becomes available in any phase; it explicitly says the global Closed flag alone does not start `c_v` or authorize early retirement. This is consistent with the separate retirement proof rather than silently replacing complete closure with an unobservable gate.

The prepared-tail timing inference at lines 127–169 is not needed for the global append-only induction. It is explicitly limited to retained checkpoints, prepared executable anchor objects, and the existing complete-handler contract; arbitrary offline catch-up and native import are excluded from that inference.

## 6. Fixed handoff, historical validation, and mutable candidates

**Text checked:** “An unfixed preparing_plan can change … The fixed handoff cannot” (`main:1786–1801`), retry-slot rule (`main:1844–1876`), and historical selection (`main:1878–1907`).

The accepted count deciding `r'` belongs to the captured export duty. It is not inferred from the retained anchor's producer, speculative production height, or number of raw records. With no assigned duty, the stored target is retained across further Forces; hence repeated Forces do not create empty sessions or advance an unopened slot. Beta still changes once, preventing reuse of an already opened pair.

REGULAR keeps its exact terminal anchor and uses `max(height(xi(T)), q_open+1)`; Force uses its triggering height and saved debt snapshot. The later regular selection floor does not require a later header at initial entry. This preserves the possibility of publishing the completed predecessor while awaiting a successor selection header, without permitting a successor's reference to justify itself through the same containing block.

**Text checked:** `VerifyHandoffHistory` versus `CanEnter` (`main:1936–2048`), atomic bootstrap (`main:2060–2084`), and races (`main:2110–2149`).

Historical receipt/anchor validation is allowed after closure, whereas initial entry must pass current open-version and durable-record guards. This avoids both reactivation from a late old receipt and rejection of retained global DATA merely because its producer is closed. A stale callback can contribute independently validated material but cannot overwrite a fixed anchor. Entry is tested and recorded before signatures are dispatched; recovery restores that durable record.

Adversarial orders checked: terminal receipt before Force; Force before terminal receipt; candidate anchor changes from accepted DATA; source duty advances before activation; stale material after a target-region change; repeated callback after entry/restart. None supplies a second authorized handoff or a second initial entry under the stated checks.

## 7. ReadyBundle and the native-state boundary

**Text checked:** Full executable state and native evidence fields (`snapshot/sections/rotation_v3_5_ready_bundle.tex:11–58`); send before/at the first Ready/Enter event and immediate guarded receive processing (lines 60–105).

The sender cannot omit dependencies based on an unknown receiver cache. The receiver validates exact anchor identity/state and current activation independently. It does not import old consensus locks or timers. If closure is already known, a refused bundle still triggers historical-context closure relay, including at Terminated records.

**Text checked:** “If validation, adoption and the atomic Enter transition are the complete message-processing operation covered by [the network assumption]” (lines 107–144).

Under that exact contract, a first entrant's complete send at `e` yields every still-authorized correct entry by `e+Delta`; the conclusion is not inferred from a bare snapshot hash. If database import/runtime initialization remains after message validation, the text explicitly admits residual `u_v` and withholds the bound. A concrete native adapter has not been furnished here. Consequently I do not strengthen the conditional retirement theorem into safety for arbitrary delayed native installation, nor infer state/DA correctness from Espresso finality.

## 8. Owed-set snapshots and deterministic control transitions

**Text checked:** Owed updates and PrepareOwed (`snapshot/sections/rotation_v3_5_control.tex:101–159`); conditional cooldown (lines 161–170); Protocol 5 (lines 202–276).

The public interpreter alone mutates `O`. Speculative REGULAR handoffs do not discharge debt or start a round. Accepted payload DATA removes its producer once; the saved `O_E(q)` remains the pre-payload value. Normal successor assignment runs PrepareOwed but retains its historical unrestricted NextLive choice. Force alone filters by NextOwed. These distinctions prevent a local preparation or a later payload credit from rewriting historical Force selection.

PrepareOwed waits for the designated target authority rather than interpreting absence as no eligible member. OFF prunes debt; ON does not insert into an unfinished round; cooldown does not remove debt. Conditional AUTO cooldown tests for another eligible owed member before applying the new cooldown. Its omission is not CLEAR and does not override an existing OFF/cooldown.

The exact successor at a pending boundary is the least historical eligible position under the saved snapshots. Repeated preparation at an unresolved canonical transition cannot use later local arrival time as a new opening timestamp. No observer-dependent choice was found in these transitions.

## 9. MSIG priority, stale targets, window settlement, and strict cutoff

**Text checked:** Protocol 5 steps 1–4 (`snapshot/sections/rotation_v3_5_control.tex:202–244`).

The entry beta is captured once. Due independent STATUS/CLEAR effects run first; the first due MSIG matching that entry beta applies, including its authorized attached configuration. Other due MSIGs are consumed without attached effects. A matching MSIG skips AUTO. A prior AUTO making an announced MSIG stale therefore neither closes the new branch nor imports the stale record's attached OFF. A regular handoff keeps beta and does not cancel a scheduled matching MSIG.

**Text checked:** Consecutive miss settlement (same file, lines 45–90).

Success resets misses; failed startup remains startup; windows advance from their deadlines; a timestamp jump settles all crossed empty windows up to the single triggering AUTO. The arithmetic shortcut uses exactly the same number of failures and last failed deadline as iterative settlement. A fresh duty opens at the realized selection timestamp and is not settled again in that prelude. Equal timestamps cannot repeat consumed controls or undo the first crossing.

Both Forces retain **pre-payload** `G`, close old beta, and prohibit new-beta exports in that triggering block. A terminal DATA in the crossing payload is too late to cancel a Force already triggered by the prelude. Positive-output versus zero-output slot selection also uses the captured pre-payload count. These facts survive the adaptive tolerance without changing DATA/DRAIN or granting a late-result grace period.

FS-1 is the retained figure's inconsistent target formula, not an ambiguity in these explicit steps.

## 10. Complete closure relay and the retirement proof

**Text checked:** Complete close bundle and phase-independent processing (`snapshot/sections/rotation_v3_4_retirement.tex:43–120`); FinishClose (`same file:122–169`).

Closed blocks fresh actions immediately. A missing fixed-session context leaves a durable relay obligation and does not assign `c_v`. A complete old-session proposal or refused ReadyBundle must resolve that context, multicast closure, and reply before ordinary processing or a tombstone return. Existing timers/evidence processing cannot be released merely because the global gate is set. The transfer recipient must already possess the evidence and receive route; a future fetch promise is insufficient.

**Text checked:** No silent closure and drain propositions (same file, lines 171–236).

If closure suppresses a vote during the forwarded proposal's `s+Delta` window, the required receive handler yields a complete close event `c <= s+Delta`. From the first complete close multicast, all correct gates close by `c+Delta`; any surviving timer expires by `c+3Delta`, and its honest FINAL delivery completes by `c+4Delta`. The local `c_v+4Delta` threshold is conservative. It does not bound arbitrary Byzantine releases, receipt formation, storage copying, or public acceptance.

**Text checked:** Conditional regional retirement safety proof (same file, lines 238–367).

The proof correctly avoids copying Part I's universal protecting-QC lemma after closing members stop voting. For a surviving timer voted at `s`:

- A correct Quit or Completion by `s+Delta` would disseminate its complete cancellation cause by expiry. Therefore neither can suppress the early matching-vote window of that surviving timer.
- If `s+Delta<c`, no close gate silently suppresses a vote. The complete proposal/admission path and Ready/Enter contract allow correct votes, yielding a protecting QC before later recovery locks.
- If `s+Delta>=c`, every correct entry gate closes by `c+Delta<=s+2Delta`, while any successor-generation entry requires a first Quit later than `s+Delta` and another `Delta` wait. Thus a later-generation correct signature cannot exploit a missing protecting QC.
- Same-generation conflicting correct votes reveal signed equivocation by the surviving timer's expiry, including certificates assembled later from already sent votes. Evidence-first/message-first processing handles equality.
- A receipt-based commitment traces through a correct FINAL signer to an earlier direct timer or ancestor commitment; receipt-triggered commits emit no new FINAL. This avoids circularly assuming receipt safety to prove direct-timer safety.

The proof's premises remain operationally supplied by the new SourceStep and ReadyBundle handlers. I found no new interface that permits a silent closed member, resets an old timer, bypasses the complete-close obligation, or imports an old recovery proof into a fresh beta. The deliberately given single surviving timer without a QC/receipt (lines 370–381) is compatible with this safety argument and is not erroneously promoted to regional progress.

## 11. Global-prefix and local-isolation proofs

**Text checked:** Atomic EXPORT acceptance (`main:1412–1482`) and Protocol 6 (`main:2208–2239`).

Each accepted interval has the exact current base and provenance, continuous DATA heights and parent hashes, per-session commitment evidence, valid normal boundary chain, and references strictly preceding its containing block. Validation occurs on a temporary state including counts and debt changes; rejection does not leave a partially accepted boundary or count. Earlier rejected future-height records are not retroactively accepted.

**Text checked:** Public replay result definition (`main:1484–1504`) and prefix proof (`main:2509–2553`).

The equality statement is appropriately about completed public replay results. It does not falsely require equal future-control queues, private caches, speculative programs, or extra local source checkpoints. Complete common input and deterministic P5 transitions give the same next public state; P6 either appends the same continuous interval or appends nothing. Force, eligibility/debt changes, and waiting never delete `G`. Different verified Espresso prefix lengths are ordered steps of this same append-only sequence.

**Text checked:** Program-isolation proof (`main:2555–2597`).

The regular branch uses the predecessor's unique terminal commitment and signed selection reference; the forced branch uses the unique public closure, accepted count, and retained prefix. Beta increments and immutable pair records prevent target reuse. Durable entry and exact callback contexts establish the local at-most-once claim. The text correctly limits this authorization to each node's processed public prefix; temporary lag can leave old production running, but cannot legalize a closed-version global append.

## 12. Limits of the review conclusion

The safety conclusions do not imply that all correct replicas have committed at receipt time, that a late surviving timer forms a receipt, that raw Espresso inclusion is accepted DATA, or that input-prefix finality proves native execution/withdrawal finality (`main:1348–1399`, `main:1522–1527`, `main:2272–2280`).

The new health policy preserves the legacy order rather than reinterpreting old attached configuration (`snapshot/sections/rotation_v3_5_health.tex:10–16`, `93–118`). Its finite-credit/restoration mechanisms affect progress; they do not acquire an older-anchor selection power. The global-progress and repeated-service statements have their own additional dependencies, and the request-liveness obligation is explicitly unresolved (`snapshot/sections/rotation_v3_5_liveness.tex:347–395`).

The reviewed safety branches remain independent of a failure to complete that request-liveness objective. Conversely, this report supplies no certification of a concrete source adapter, native-state installation latency, state execution proof system, or complete protocol implementation.
