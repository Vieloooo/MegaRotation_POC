# Historical task service: scheduler, P6 and Handler integration

Date: 19 October 2026. Frozen design note for the runtime portion of JG2 in `historical_service_fresh_check.md`. This is a proposed replacement of the Force selector and its runtime dependencies, not a change already implemented in v3.9. The Model, v3.9 sources and the three historical-service notes were not edited.

## 1. Result and exact scope

A historical appointment can use a deterministic persistent queue without inspecting the growing catalogue on every retry. Keep one FIFO list per physical region, compare their heads when a new appointment is needed, and retain an eligible unserved appointment across fresh-beta retries. Journal catalogue changes separately. Once such an appointment is fixed, later registrations cannot change its retry decision; materializing their queue effects is therefore not a prerequisite for that retry. Commit accepted DATA and its exact-task discharge before any subsequent catalogue or boundary wait.

This resolves an algorithmic dependency in JG2. It does **not** establish an unconditional numerical response bound for an arbitrary implementation. The existing regional complete-message bound yields the old form of `L_(i,r)` only if the additional retained lookups, small journal transactions and pin checks are part of that same admitted complete-handler operation. The frozen Model does not separately bound database operations, catalogue processing, CPU contention or arbitrary catch-up. Section 8 states the exact supported timing implication and the remaining conformance obligation.

The following two inputs are explicit conditions, not conclusions of this note:

1. **Canonical registration input.** There is an authenticated, append-only stream of initial-authority registration events with a specified canonical subposition, finite prefixes, deterministic ordering and a completion criterion for every requested fixed cut. Each positive registration supplies the actual immutable initial AUTH and its authentication evidence. Later discoveries never insert a task before an already finalized registration cut. The input may be the public-boundary subset derived in `historical_inventory_adapter.md`. Coverage of all speculative or zero-output AUTH is not assumed or proved. An unknown AUTH inventory is not an empty one.
2. **Historical service scope.** A task used by the progress argument remains in the original fixed-snapshot fault, correct-message, authority/native preparation and regional-progress domain when addressed. Its correct members retain the necessary keys, state, owners and handlers. AUTH authentication and a governance ON do not establish this condition. A theorem quantifying over all queue blockers must cover every possible incumbent, including a younger task appointed while an older task was ineligible.

Controller arbitration, health-lease journaling and eventual scheduling-MSIG submission are a separate component. Section 7 gives its exact runtime interface. No controller decision deadline or new publication obligation is used here.

## 2. Public state and indexed queue

Within the fixed chain domain, the task is `(i,r)`: region `i` under immutable `AUTH[r,0]`. Its AUTH hash is a useful cache-integrity field, but not another protocol identity. Engine identity remains `(beta,r)`; no additional session identifier is introduced.

Every registered positive-weight task has exactly one outstanding service token. The token has a **birth event** in the canonical scheduler-event order. This is queue age, not an engine or signing domain. An implementation may assign consecutive integer serials when it materializes those events. The birth event is the authoritative ordering key; serial assignment must not depend on callback completion time.

The persisted public state contains:

| State | Meaning |
|---|---|
| Registration journal and its verified frontier | Ordered fixed-cut registration descriptors, authenticated AUTH bytes once acquired, and unresolved source obligations; duplicate AUTH registrations have no second effect |
| Scheduler-event journal | First registration, accepted-task discharge/requeue and their exact canonical subpositions; each event is committed once |
| Queue projection frontier | Last scheduler event fully materialized, plus one FIFO list per fixed physical region and a direct index from `(i,r)` to its current node |
| Service counters `s[i,r]` | Number of newly accepted DATA blocks of that exact task after its registration; updated by DATA acceptance, never by selection or receipts |
| Optional pin | Its `(i,r)`, outstanding-token birth, service count at appointment, original appointment evidence and current assigned boundary, if any |
| Saved selection snapshots | Canonical cut, controls/cooldowns, queue-head evidence when materialized, and pin state; a deferred snapshot is recorded as unresolved, not as an empty queue |
| Existing replay state | Exact subposition, beta, G with provenance, pending boundary, duty/window state, source/control cursors and Closed gates |

The queue projection uses an intrusive doubly linked FIFO list for each physical region. First registration appends a node; discharge removes that task's indexed node and appends its successor token to the same region's tail. Each list is in global birth-event order. Removal of an interior task is allowed: REGULAR DATA can serve a task that is not a head. OFF and cooldown do not remove, rotate or re-age its node.

All registered nodes have positive weight under their own initial AUTH. For a fixed region, governance and automatic cooldown eligibility are common to its nodes. Consequently the oldest eligible token is the minimum birth event among the heads of eligible nonempty region lists. This examines at most the fixed region-cycle length, independently of the number of historical AUTH records. A pin test examines its one task and current regional eligibility.

Persistent node versions or a durable write-ahead event journal preserve previous snapshots. A source-position index retrieves saved snapshots; pending selection keeps a durable next-position cursor and visits positions in order rather than rescanning from genesis. A historical checkpoint must authenticate the materialized queue state/frontier, pending ordered descriptors, service counters, pin and existing replay state, or provide the complete authenticated replay route that reconstructs them. A sender-selected queue root is not a new trust anchor.

This is an indexed algorithm, not a constant-time storage theorem. Dictionary access, pointer persistence, hashing and checkpoint maintenance have implementation costs. In particular, using a persistent balanced map does not make its growing lookup/update cost a proven constant. No timing conclusion below relies merely on calling these operations atomic.

### Logical events and deferred materialization

The logical scheduler applies journal events in canonical order. The queue projection may lag. A new or unpinned selection fixes the required cut and must finish all events through that cut before using any head. Work beyond the cut cannot enlarge or restart that job. The worker services the needed prefix before later jobs. It never selects from a partial prefix and later revises the choice.

The pending selection job persists its boundary/cause, exact historical position, post-control state, fixed catalogue cut and projection cursor. If that job lacks evidence, P6 pauses at that substep. The independent source-availability reader still retains and relays later inputs, but P6 does not apply their later Forces over this unresolved earlier state. It first derives the earlier canonical selection, including any pin, and then processes later Force records in order. A queued Force cannot cancel or replace this job with a newer catalogue cut. If the completed job proves no candidate, P6 may advance; the next historical position creates a new fixed job, with the completed predecessor retained. Thus each finite cut is serviced before moving on. Finiteness of a repeatedly replaced cut would not suffice.

If a registration descriptor requires unavailable source evidence, its fixed-cut job remains pending. A local successful lookup cannot create a different public registration position. Under the conditional registration contract, that fixed job eventually obtains its finite source set; executing its finite ordered work then terminates. This is eventual finite work, with no uniform completion time. Without the contract's completion criterion, the inventory barrier remains exactly JG1.

The pin projection is maintained directly by canonical P6 events, independently of this tail materialization. For an already appointed task, P6 knows its token and service count from a verified completed selection cut. It can check that no DATA of that task has discharged the token. New registrations only append; they cannot invalidate this fact. When a queue-dependent decision is later required, replaying the deferred events reconciles the two projections before that decision.

## 3. Exact public event order

The registration adapter must identify one of the following canonical event positions. An unspecified local arrival or a completed background callback is not a position.

1. At genesis, authenticate the required initial AUTH, register its positive-weight tasks in the fixed region order, then resolve the synthetic boundary.
2. At an explicitly defined source registration position supplied by the conditional adapter, journal that position and its finite fixed-cut job. Within an event, order initial authorities by the adapter's specified order and their tasks by the fixed region order. Such a source event may precede a block's prelude only if the adapter actually specifies that order. It cannot be retrospectively inserted there.
3. At a public REGULAR boundary's authority-resolution substep, authenticate its already required `AUTH[r+1,0]`, then journal its registration before that boundary's public resolution completes. This occurs **after** terminal DATA commits. If the historical REGULAR selection header is earlier, registration still occurs at this resolution substep; it is not backdated to the header. A private speculative handoff does not register a task.

Duplicates are resolved by first canonical registration of an initial AUTH. The logical stream contains one first-registration event for every positive-weight `(i,r)` in that AUTH and no event for a zero-weight region. Arbitrary optional AUTH names or proof sidecars do not create registration jobs.

For each authenticated Espresso block `J_q`, the public interpreter performs the following ordered transitions. The complete namespace and applicable controls remain genuine source prerequisites. Catalogue independence below does not permit skipping either.

**A. Resume and apply independent controls.** Resume the recorded canonical subposition; authenticate every applicable control; apply due independent STATUS/CLEAR in the inherited `(activation timestamp, control sequence)` order. Keep OFF tokens at their old queue ages. ON does not clear cooldown. Current eligibility changes, but a fixed handoff is not revoked or rewritten.

**B. Match MSIG against the entry beta.** Apply only the first due matching MSIG, including its authorized legacy attached configuration, close the entry beta and consume all other due MSIGs under the existing stale semantics. A stale record applies no attached configuration and is neither refunded nor retargeted. REGULAR has not changed beta. A matching MSIG cancels the entry duty's window and skips AUTO settlement.

**C. Otherwise settle windows.** Use only already accepted DATA counts and the original startup/running, half-open-window and batch-settlement rules. On the threshold, establish the AUTO cause. Missing raw source or controls cannot establish zero output. One prelude has at most one effective Force.

**D. Record closure before successor waits.** For either Force, persist the old-beta Closed gate, exact pre-payload `G` and its original provenance, cause/header, captured source context, and selection lower bound `q`; archive only the unconfirmed suffix; increment beta exactly once. Preserve the unserved pin as a candidate, not as permission to reopen the old engine. These facts do not wait for catalogue completion. The historical selector replaces v3.9's Force target-slot rule: the selected task supplies `r`; no successor AUTH is guessed from the source index while selection is unresolved.

**E. Resolve the forced appointment at the first eligible historical position.** At `q` or the next unexamined authenticated position, use that position's post-control/pre-payload eligibility. If the retained pin is outstanding and eligible, reappoint its exact task without materializing later tail events. Otherwise drop the appointment preference, retaining the old token's age, and complete the queue projection through this position's fixed selection cut. Choose the oldest eligible head as in Section 2. If the complete cut proves no candidate, keep the boundary unassigned, persist that fact and continue source replay. If evidence is missing, keep the persistent fixed-cut wait described in Section 2; later raw Force inputs cannot leap over it. A later Force reached in canonical order supersedes the pending boundary's beta/cause while keeping its captured context and any then-valid pin; it does not fabricate an empty export session or restart an unfinished older selection job.

For AUTO, conditional cooldown is computed before this selection using its permitted candidates. An eligible unserved pin is the sole permitted candidate, so that AUTO does not newly cool its region. If no eligible pin exists, use the complete fixed-cut head view; cool the captured source region only if another eligible candidate in a different physical region exists. Use the old source duty's `C_auto`, preserve an older cooldown when the condition fails, and then select under the resulting eligibility. A required catalogue wait postpones cooldown/selection computation, not the established cutoff. MSIG adds no automatic cooldown.

The opening boundary fixes `(beta_new,r)` and region `i`, fresh `g=t=0`, anchor exactly equal to the retained pre-payload `G`, endpoint `h(G)+TB[r]`, the initial AUTH, and the first eligible selection position. Set `q_open` to that position and `lambda_open` to the maximum of the anchor's original L1 reference height and `AUTH[r,0]`'s publication height. Open the startup window there with `M=1+beta_new`, zero misses and zero accepted count. A timestamp jump does not settle this new window again in the same prelude. No new-version DATA may enter G in the Force block.

**F. Save pre-payload snapshots, then process payload in order.** Retain the post-prelude eligibility and scheduler cut/pin evidence. Later DATA or registration does not alter these snapshots. P6 processes logical kernels, strict references and independent append gates exactly as in v3.9. Closed-version DATA in this block cannot append. Neither a public registration nor an old result grants append permission.

**G. Commit each accepted DATA and its task event together.** For a verified next DATA produced by region `i` under slot `r`, atomically append DATA to G, perform existing duty/window/request accounting, increment `s[i,r]`, journal discharge of that task's current outstanding token and requeue at this DATA transaction's subposition, clear a matching pin, and expose the committed prefix and cursor. Empty application batches count as DATA; DRAIN, duplicates, FINALs and unaccepted results do not. One DATA discharges one current token. Later DATA of a continuing duty may discharge its successor token, but cannot discharge another task. That duty keeps its original remaining quota.

This commit records a deterministic dequeue/requeue descriptor; it does not first walk or finish the queue projection. The accepted producer's initial AUTH was already required by its own public boundary. Accepted service is therefore not waiting for discovery of a new authority. Service counters can be updated by task identity without resolving the birth ordinal of an unmaterialized requeue. Before a future queue-dependent selection, ordered materialization resolves those events exactly once.

**H. Resolve terminal successors only afterward.** If the DATA is terminal, the same transaction records the pending REGULAR boundary. Expose it and the DATA before acquiring successor AUTH or processing catalogue work. REGULAR keeps beta, uses `r+1`, inherits exactly the terminal DATA, and uses its unchanged historical unrestricted `NextLive` search. Register the required successor AUTH at the substep above; no task queue filter is added to REGULAR. Its DATA/DRAIN endpoint, completion, recovery and speculation rules are unchanged. A missing successor input pauses this later operation, not the completed DATA transaction.

If successor resolution or later records remain unfinished, a checkpoint may attest the committed prefix together with the exact pending stage and unresolved queue descriptors. It may not assert an invented completed queue, selected successor or absence of candidates.

## 4. Handoff verification and the local Handler

The Force handoff verifier replaces only the old `r` derivation/NextOwed check with replay of this task-selector transition. It verifies the old-beta MSIG match or AUTO threshold, strict pre-payload anchor, completed selection cut **or** an authenticated prior appointment with preserved outstanding pin, current historical eligibility, first eligible position, exact task and initial AUTH. The packet may name the token birth for lookup, but the verifier derives it from authenticated replay. The packet cannot choose a task or replace an installed handoff.

The pin route requires the original selection evidence plus every intervening relevant canonical transition: no discharge, eligibility at this selection, and no replacement appointment. A continuously following prepared member maintains this proof incrementally in its verified state. It need not replay the whole retry chain on every message. A returning member or one missing that state hydrates it before using the prepared timing claim. An arbitrary unverified checkpoint or a sender's assertion that a pin survived is insufficient.

Handler inputs remain the existing public source, regional result and material-available events. The Handler consumes the replay-selected task and fixed boundary; it cannot appoint a different task, advance beta, move the cutoff or apply a controller suggestion directly. It prepares an exact local target and uses historical verification plus the live activation predicate.

At a dependency-complete event, validate the fixed handoff, exact anchor provenance and native object, retained AUTH and opening evidence; atomically test the existing durable entered/Closed/tombstone gates; attach already executable state; perform Ready and initial Enter once; and send the complete ReadyBundle. No separate catalogue worker, controller callback or polling tick lies between this event and Enter.

Waiting, Ready, Active, Frozen, Completed and Terminated remain the six program phases. Queue/pin state is public scheduler state. A fresh `(beta,r)` receives no old locks, reports, votes, Quit proof, entry time or timer. Closure persists before per-program cleanup, and every old-beta callback observes it. Missing old snapshot context preserves the closure-relay obligation. The existing complete-close event, `c_v`, `4 Delta` drain and ownership-transfer rules are unchanged. A queue node never authorizes an old program to sign again.

## 5. Fixed-G preparation and complete dependency accounting

Fix an eligible unserved pin `(i,r)` after the last interruption that can replace it, and suppose its retained anchor remains `G*`. Prepare and keep the following independently of each fresh beta:

| Dependency | What is retained / when it is obtained |
|---|---|
| Exact anchor | Full DATA, original producing pair/hash/height, original direct or covering commitment evidence, required ancestry and source provenance |
| Native continuation | Actually executable state for this exact anchor, configuration, region and native format; native validation evidence and all proof inputs; no pending database import |
| Initial and recovery AUTH | Initial AUTH with fixed old snapshot/quota/windows; persistent sequential fetch cursor and full authentication bytes through the finite recovery index needed for this task's regional response |
| Public source | Authorized maintained replay checkpoint, exact subposition, all legally referenceable raw Espresso bytes through a fixed hydration cut, dictionary/source-occurrence mappings, and every subsequent acquired namespace including rejected records |
| Controls | Complete applicable control prefix and pending records, authentication dependencies, current governance/cooldown and window state; advance-announced controls are checked before their crossings |
| Scheduler proof | Verified original appointment cut, exact pin/token and service count, incremental preservation state, actual retained supporting source and queue evidence, plus deferred tail descriptors and their frontier |
| Current opening | New finalized opening header/namespace, cause and current beta, fresh immutable handoff/reference fields, Closed routing update and durable program entry state |
| Regional execution | Genuine full ReadyBundle, fresh reports/proposals/recovery proof and receipt dependencies under this same historical snapshot; persistent request owners remain separate from catalogue processing |

Hydration and preparation have a fixed target. Later beta changes retain the cache and unfinished fetches. They do not restart native preparation, AUTH fetching or catalogue-origin validation. Later registration jobs are retained without becoming prerequisites for preservation of this pin. Any actually missing original pin evidence is initial hydration; it is not exempted from verification.

For each opening or result-source event, the mandatory path is exhausted by the following cases:

1. **New raw input:** acquire the complete namespace/header/linkage/finality and applicable complete controls; retain bytes and enqueue import/registration descriptors before releasing the buffer. Immediate SOURCESTEP relay does not await local P6, native preparation, catalogue expansion, request import or controller work. Unavailable genuine boundary evidence is an explicit incomplete-operation state, not a complete packet.
2. **Prelude:** the prepared pin route reads its resident task/pin proof, current controls and window state. It does not enumerate AUTH history, inspect other queued rosters, scan workers, or fetch a new successor authority. An OFF/ineligible pin or missing original evidence takes the general waiting path and lies outside this stable-tail case.
3. **Public result kernels:** inline objects and retained legal public aliases supply DATA/FINAL and ancestry; fixed AUTH authenticates the correct-signer kernel. Wrong optional PROPOSE, recovery-AUTH, L1 sidecar or native-output targets add no required fetch. Valid neighbors remain independently inspectable. Missing genuine source is still a wait.
4. **Append:** check exact current boundary/G, quota, strict source precedence and actual linked DATA. Queue registration never upgrades absent/private evidence. Intrinsic regional validity uses the existing honest-FINAL argument; execution/DA publication for `C_pub` remains separate.
5. **First DATA transaction:** update the already known task counter, journal its discharge/requeue, clear pin and commit G. No queue expansion or successor resolution precedes exposing this growth, even when quota is one. Later payload work may wait without undoing it.
6. **Ready and regional execution:** use the prepared exact native object and full evidence, current opening header and guarded fresh entry. No future header, new READY certificate, publication of a bare task/request or all-catalogue snapshot is required. Recovery uses only this fresh pair's proofs and the retained finite AUTH prefix. Complete-result offering remains the original service input.

The packet's new scheduler evidence is either already retained under the recipient's complete-message resolution contract or supplied in full. A sender's cache hit cannot justify omitting evidence at its recipient. A prepared receiver validates new steps against its maintained state; it does not import an arbitrary historical checkpoint or scan an ever-growing registration journal inside the timed operation.

## 6. Local arguments

**Queue agreement and one token per task.** Induct on canonical scheduler events. First registration appends exactly one token unless that AUTH has appeared already. A discharge deletes the unique outstanding token and appends exactly one replacement; a duplicate DATA has no event. Other transitions leave token membership/age unchanged. Consequently any two complete projections through the same cut have the same lists, heads and historical service counts. Direct P6 counters may be ahead of that projection; replay checks their event history and never overwrites them with older projected values. Logical birth events give a strict common order even if one implementation assigns numeric serials later. Deferred materialization cannot change the logical result.

**Head selection is complete.** Within a region, every node has the same governance/cooldown eligibility and positive task-specific weight. If the region is eligible its head is its oldest eligible token; otherwise none of its tokens is eligible. The minimum over the finite eligible region heads is therefore the global oldest eligible token. No historical AUTH scan is needed at selection. Indexed interior deletion preserves list order.

**Tail independence of a retained pin.** The priority rule consults an outstanding eligible pin before heads. At fixed G there are no discharge/requeue events. New registration adds only tail tokens, and changes neither the pin's old AUTH, its anchor nor regional eligibility. Therefore any two completions of the unmaterialized registration tail produce the same retry appointment. The interpreter may commit that decision without asserting either completion. This argument stops if the pin is discharged, replaced or ineligible; the next head-dependent choice must finish its fixed cut.

**Exact-task discharge does not depend on catalogue completion.** The producer task is already identified and authenticated by its public boundary. Its service count advances exactly once with each new DATA, and its deterministic journal event can be retained before queue replay. In the eager semantics that event removes and requeues the same unique task token; in the deferred semantics the worker performs that operation later in the identical order. The visible G prefix and pin release agree. Thus delaying the projection cannot delay the already committed first DATA or provide a second credit.

**Fixed anchor on the stable pinned tail.** Every zero-output Force retains pre-payload G and reappoints the same task. Contiguous acceptance cannot skip its next height and accept a speculative REGULAR successor first. If its first DATA is accepted, the service obligation is discharged; otherwise G remains the exact old anchor. Controls before this tail can replace the pin or let another task move G, so the claim deliberately starts after that finite prefix.

**Unique historical appointment.** Given an authorized checkpoint and complete registration/source histories, the controls and MSIG match, Force cause, retained anchor, eligibility, pin preservation and first completed head choice are deterministic. Missing evidence leaves a pending transition at the same position rather than skipping a candidate. Immutable saved pre-payload snapshots prevent later payload service from changing this choice. The Handler's separate live gate then prevents historical validity from reviving a Closed pair. These are local invariants, not a whole revised safety proof.

## 7. Controller interface and fairness boundary

After each committed public subtransition expose a read-only `ScheduleView` containing its canonical cursor/cut, accepted tip, beta, current duty/boundary, pin and direct outstanding/service state. Include the oldest eligible task only when the relevant fixed queue cut is complete; otherwise expose an explicit pending result. Registration/discharge descriptors are durable replay input for the controller's own journal. No private controller cache changes public queue order.

The scheduling need is: a complete view contains an eligible outstanding task and there is no eligible outstanding pin currently assigned to the active public duty. An older token becoming eligible does not preempt a younger eligible pin already being served. Exact discharge or an effective Force triggers recomputation; an issued old-target MSIG retains its original immutable semantics even if the need subsequently changes.

The controller must reserve the scheduling action's first available per-tip Force credit ahead of optional H1 traffic, enforce its task-specific lease credits using a complete fixed registration cut, and carry out the existing empty-configuration future-dated MSIG submission duty when required. Its private work is not a P6, SOURCESTEP, Ready or first-DATA dependency. Missing catalogue completeness may delay its decision; it cannot infer no debt from a partial view. An effective Force chooses through public replay, not an MSIG task field. Local nonstarvation of reservation/dispatch alone does not prove a submitted public operation: the runtime fairness implication requires an actual valid submission, or an established enabled-submission interface. The frozen `T_L1` clock starts only after actual submission; this note adds no ingress service assumption.

The runtime lemmas establish no all-task fairness theorem on their own. That theorem still needs the controller's eventual appointment and finite-interference properties, eventual eligibility at the relevant selections, the registration coverage of the target, and the response premise for every task that can block it. Finite predecessors are useful only after those dependencies are established; an uncovered eligible historical incumbent can remain pinned indefinitely. REGULAR remains unrestricted, so a Force-only selector without the mandatory controller obligation is insufficient.

## 8. What timing follows, and what remains open

The frozen Model supplies regional delivery **and processing of complete validation dependencies** within `Delta_i` (Model lines 167–179), complete-result publication under its admitted limits (181–196), online maintained source visibility (236–254), and eventual authority/native availability (307–309). It does not assign a new deadline to the registration adapter or a growing local storage operation. `rotation_v3_9_source_step.tex:243–255` and the ReadyBundle's exact timing paragraph make the same distinction.

For the stable pinned tail, the service-scope condition gives the finite task-specific recovery prefix used in `rotation_v3_9_liveness.tex:63–158`. No maximum over all historical rosters is taken. With `Q=TB[r]`, its unchanged regional bound has the form

`D_(i,r) = (Q+1)(2K+12) Delta_i`,

`R_(i,r) = (g*+1) D_(i,r) + 2 g* Delta_i + 9 Delta_i`.

This bound already presupposes that complete ReadyBundle validation/adoption, report processing and regional operations satisfy the inherited complete-message interface. A retained serialized snapshot requiring a later import does not satisfy it.

Now impose the **implementation conformance condition**, rather than silently deriving it: on the prepared maintained path, both extended SOURCESTEP receive operations—including resident scheduler proof lookup, direct pin checks and durable constant-descriptor updates—are covered by the existing complete-message processing contract under its declared admission/load regime. There is no outside catalogue scan, native import, journal-replay backlog or independently scheduled callback before their guarded final step.

Under that condition, if a retry opens in a block first finalized at real time `a`, the original decomposition applies:

`first correct Enter <= a + T_E + Delta_i`, unless closure already forbids it;

`L_(i,r) = 2 T_E + 2 Delta_i + R_(i,r) + T_pub`.

By `a+L_(i,r)`, prepared regional replay has witnessed its first accepted DATA or the retry has already been effectively closed. The first-DATA transaction can supply this witness while terminal successor resolution remains pending. This is not a bound for arbitrary external observers or initial historical catch-up. A new or unpinned selection may take unbounded eventual preparation time before the stable-tail premise applies.

Without conformance, the available decomposition contains residual local delays: opening-handler residual `u_open`, regional residual `u_reg` if adoption/processing is not covered, and result-handler residual `u_accept`. Conditional on those operations finishing and the pair remaining permitted, the corresponding expression is only

`2 T_E + 2 Delta_i + R_(i,r) + T_pub + u_open + u_reg + u_accept`.

The frozen assumptions give no uniform bound on these residuals across fresh beta attempts. Indexed access removes a catalogue-length scan but does not prove that the residuals vanish or have a fixed maximum. Finite work at every attempt permits a sequence of increasing finite delays. Therefore this algorithm alone cannot justify a fixed `L_(i,r)` or the quadratic-versus-linear AUTO contradiction for such an implementation.

If the conformance condition holds, the fixed AUTH/startup window, retained G, finite control interference and growing `M=1+beta` give the existing conditional contradiction: no-growth retry gaps are at most `L_(i,r)`, while required timestamp gaps grow as `(1+beta) W_start`; summation conflicts with the upper clock-lead bound. No lower clock-lag bound is needed. If conformance is unproved, that quantitative step remains the precise runtime part of JG2 still open. It is not repaired by finite catalogue size, bounded message delivery alone, a hash index or naming the work one event.

## 9. Source anchors and validation

All paths below are relative to `formalization_v2/`.

- `revisions/v3_1_frozen/model_and_assumptions.tex:30–44,61–84,159–179,181–214,236–283,295–318`: immutable task authority, control order, service interfaces and progress scope.
- `sync_hotstuff_rr_part2_rotation_v3_9.tex:1580–1795,1799–2034,2183–2245`: Handler, durable phases/cache, historical/live handoff predicates and ordered P6 commits.
- `sections/rotation_v3_6_control.tex`: the preserved strict cutoff, old-beta match, first-crossing window arithmetic and conditional cooldown; the old physical-region owed selector is the explicitly replaced component.
- `sections/rotation_v3_9_source_step.tex:33–84,88–194,196–257`: byte retention, public-kernel dependencies, immediate relay, close gating and exact timing limitations.
- `sections/rotation_v3_9_liveness.tex:63–158,162–293`: local finite recovery prefix and the original prepared-response decomposition.
- `sections/rotation_v3_5_ready_bundle.tex`: full executable-state/validation-dependency contract and explicit residual-installation limitation.
- `reviews/v3_9/historical_service_design.md`, `historical_service_fresh_check.md` and `historical_inventory_adapter.md`: the candidate, JG2 and the unresolved all-history registration coverage.

Validation performed: direct source inspection and local deterministic-state/dependency/timing arguments. No source edits, TeX build, executable simulation, model checking or independent whole-protocol review were performed. The note specifies a runtime candidate and its conditional lemmas; it does not close the historical service-domain question, JG1, controller proof or implementation timing conformance.

Source SHA-256 values checked before writing and after completing this note:

- `reviews/v3_9/historical_service_design.md`: `bbfdcc3f29b395196e0871b115f98e9859d86bb291881c3da49ca37c882a972d`
- `reviews/v3_9/historical_service_fresh_check.md`: `99fc1e945b682e5151dca7ffe24166a61e97ebd2f7c139d81fc0e9ca8b6624bf`
- `reviews/v3_9/historical_inventory_adapter.md`: `563d8adc2fea2be65eef11f718a7835deca7814aa880e9051ef2fdec7033908b`
- `revisions/v3_1_frozen/model_and_assumptions.tex`: `1309fa5f8db29f082c4757bb07a1c7c4aa6723a277a75cdbf976566e2f68a387`
- `sync_hotstuff_rr_part2_rotation_v3_9.tex`: `63dd6770de6aaed0f442c53c6fcd3685fb5a37b478140d67970ee2c4719058cb`
- `sections/rotation_v3_9_source_step.tex`: `9c25926926439a0010d38a1373d21aba512fef01a6073fae92598dc95c58d310`
- `sections/rotation_v3_9_liveness.tex`: `5b3376a83cee0f6e6133819b383ae4b97236bf5fb612a02bd8ae99cf73eeb4bd`
- `sections/rotation_v3_6_control.tex`: `37e7c065b33cead1f334878f9fe92e761472464e5a67292947cd9a4f251d2285`
- `sections/rotation_v3_5_ready_bundle.tex`: `4048a979add0bfe1662af309d5b30f7e72b105e8d5b6fda80de415ba43330152`

## Freeze record

Frozen 19 October 2026. SHA-256 of the UTF-8 bytes preceding the newline immediately before this heading: `aae5a6f3736749241a47263a17271635b756f06a29687ae248442d52ee705620`. The heading and this paragraph are excluded to avoid a self-referential hash.
