# H2 pure reducer — independent source-first increment v1

2026-11-16. Assigned bft_verifier, GPT-6 Astra / xhigh. **Expectations prepared before reading the new reducer contract, its implementation, or builder explanation. No executable cases have been run.** This freezes an independent oracle increment, not a production API or implementation contract.

## Authority and provenance

Current checkout `/home/jy/.codex/worktrees/a6cf/megaroation_poc`, branch `codex/h2-session-manager-implementation`, observed HEAD `dd3110f77724a273a4e7982c05d6c4b36dea4b9d`. Delivered M0 is `35d382ac445cc8bc779f0d18fd23cb57e54680da`.

New authority **D**: `/mnt/reth/jy/megaroation_poc/poc/docs/milestones/M1-implementation-dispatch-20261116.md`, exact Git object at `d55d39f73593676325bdc0fbb7ba3845ca93025f`; live-file bytes and exact object independently hash to `4a211cd38a5db108dc3b8548a87c19e599cb3870e534e513b41e487b4c5cc8b6`. Read in full before drafting. D:3/43/47 authorizes the independently frozen pure reducer slice without waiting for H1 runtime; old preparation-only implementation restrictions no longer apply to this slice. Native integration remains a separate gate.

Retained source-first oracle **B** (unchanged, checked SHA-256):

- `/tmp/h2-session-manager-healthy-verifier/independent-expectations-v1.md`: `11f745e81faa46c03d85d2ee1ed722a842f8631ef62e96dd556e8610cece7ceb`.
- `hand-vectors-v1.yaml`: `9bda2836ba7a2029e53f79c070a49cb83a217d60f21e8e7e893c367bf7337fbf`.
- `source-manifest-v1.json`: `45cb6839949ad32bdd2284a7d90b22adebcda345d0482bc3d59deabb3af37477`.

B's old authorization/gate status is historical; D supersedes only the identified scope/adapter decisions, not source protocol invariants. The original files are not rewritten.

Source abbreviations remain those in B: **SP** v10 Session Program, **SM** v10 Session Manager, **PUB** v10 Publisher, **GFR** v10 finality rules, **V2** v2 design, **O3/O7** accepted public object/replay contracts. Exact versions/hashes are recorded in the unchanged source manifest. Regional BFT remains gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`; Commonware `2026.10.0`, upstream `92a2f68a0e131c8b69de41e84aec37da6280f212`. No upstream implementation is modified or revalidated by this increment.

## What is independently decidable in this slice

The subject is a deterministic, side-effect-free local state reducer with immutable proposed state/delta and action intents. It can verify logical identity/binding, source cursor and full-record consistency at the declared validated-input boundary, planning/commit simulation, monotonic lifecycle, action identities, receipt handling and retained obligations. It must not itself send network messages, create/drive native engines, sign, arm real timers, write a second service database, assert cryptographic verification, compute a competing public encoding or advance GFR public state.

H1 owns exact native state verification, ReadyToken issuance, engine handles/guards and the actual single root/aux durable transaction. H3 supplies complete locally verified ReplayRecords and publication services. Until their accepted API is available, the reducer can use explicitly internal input/capability references and named fixture adapters. An internal or test-issued capability is a prerequisite declaration, not evidence that the referenced checkpoint/proof/state was truly verified. No production constructor that accepts `ready=true`, a state root alone or an arbitrary unbound token may be described as actual readiness verification. Native integration must consume H1's eventual shared types, not preserve a parallel public API.

## Source-derived rules and dispatch decisions

| Rule | Sources | Independent expectation |
| --- | --- | --- |
| R-ID | SP:78–119,170–184; SM:198; O3 descriptor/cid contract | Fixed descriptor per cid, exact anchor identity, unique instance reference. Repeated same preparation does not overwrite progressed phase/entered/terminal state; same-height different anchor is distinct. Recompute/check existing O3 identity under the appropriate verified descriptor boundary. |
| R-READY | SM:58,81–83,198,246–255; D:31 | Preparing and Selected are independent. Readiness requires an applicable H1-issued capability bound to worker/cid/exact anchor/verification version and evidence identity. The pure fixture may model this dependency; native correctness remains OPEN. Wrong binding is REJECT; missing capability is WAIT; Ready must not arise from source selection alone. |
| R-SCHEDULE | SM:214,224,257,263–265; D:30 | Highest legal Ready candidate before public selection, one scheduled cid per logical session. Higher Preparing does not displace lower Ready. Public choice excludes alternatives even when chosen candidate lacks state or is already terminal. Locally pausing A preserves its identity; selecting back entered A requests Resume/SelectBack, never another Enter. |
| R-CLOSE | SM:225,230,240–242,259; SP:192,416; D:29–31 | Close/stop/unsafe cannot be undone. Persisted closed scopes prevent late creation/reactivation. Time uses the original stop boundary; ClockUnsafe is a private local cause and guard, never a fabricated public Time event or enum extension. Preserve original terminal fact and separately retain later closure evidence. |
| R-RECORD | GFR:219,379,418; O7 lifecycle/atomic record contract; D:28 | Whole verified ReplayRecord is the consumption unit. Verify full record identity/continuity and complete ordered lifecycle list; same pc can contain several events. Close facts, Selected+Preparing, cursor and pending intents commit together logically. Missing successor execution state does not delay a known valid close; invalid/unverified record cannot masquerade as committed. GFR pc is independent and is never rolled back when Manager waits. |
| R-COMMIT | V2:239–252; SM:230,240; D:28–29 | Proposed delta is immutable and inert before external commit. A rejected, wholly uncommitted plan changes no accepted logical state or cursor. Once an actual H1 terminal/unsafe guard is durable, later failure cannot restore Running or permit signing. Retain that safe fact and pending obligations; stop/fail the step if needed. No success cursor or action completion is fabricated to conceal partial failure. |
| R-ACTION | D:28–30/38; SP one-time Enter; PUB task identity | IDs must be deterministic and stable for retransmission of the same logical intent, distinct for semantically distinct commands/subjects and appropriately isolated by run/worker/source binding. Retry attempts/time must not invent a new intent. Retain exact action bytes/parameters and pending status until an applicable receipt; `Committed(actions_pending)` is different from `ActionsApplied`. Exact formula is a contract choice to review after this seal. |
| R-RECEIPT | D:28–29; SM:240; PUB:135–139 | Receipt is correlated to an existing exact intent and its expected target/outcome. Unknown/mismatched/conflicting receipts cannot clear obligations or resurrect entries. Successful exact replay is idempotent. Failed/incomplete action retains the same retry identity. Later terminal/selection guards dominate any stale Enter/Resume receipt. |
| R-HISTORY | SP:222,246,420; SM:118,188; PUB:101,135–139; D:33 | Pending historical result/Offer/evidence identities survive Pause/TIME/ClockUnsafe/local Close. Genuine public exclusion/cutoff may stop new retries by H3's rule; retain history. A receipt alone is not public recording or global acceptance. Reducer does not re-run historical Final through current local Accept. |
| R-H11 | SP:215–230; GFR:109,231–263; D:35 | Early cached h11 and late legal-slot h14 are separate cases. Both retain window11–20; late legal production is permitted before stop but cannot be labeled h9→h11 or given forged firstSeen. Pure intent trace cannot prove native production or G acceptance. |

## Required state/observation distinctions

These are semantic distinctions for review, not new Rust/API schemas:

1. Proposed vs externally accepted state/delta. A caller cannot treat a plan or serializable JSON as a durable H1 commit receipt. Base revision/hash and immutable binding must reject stale/incompatible commits.
2. Public Selected vs local Ready vs Enter requested vs actual Enter applied. An Enter intent awaiting receipt cannot be advertised as an executed native Enter. If the implementation tracks desired scheduling separately, it must label that state honestly and recheck guards before effect release.
3. Pending intent vs durable obligation vs acknowledged application. Cursor says the whole record's obligations were accepted, not that an engine action succeeded. No production operation is executed by this component itself.
4. Public closed scope vs private local terminal cause. ClockUnsafe does not close the public session/branch or move GFR; public Time must not be emitted merely to reuse an enum. The exact private scope of ClockUnsafe remains an explicit contract choice; at minimum it terminates the affected current cid and blocks new Enter until an accepted safe-clock transition.
5. Ordinary snapshot rejection vs irreversible guard preservation. The dispatch permits rejecting a plan before effects. It forbids rollback across a guard already installed durably. A guard-ledger/receipt overlay or equivalent fail-closed state must preserve this distinction; cannot simply restore a pre-step cloned Running state.
6. Immutable source truth vs private delivered/consumed cursor. A complete source record can be available while Manager is pending. H2 never asks H3 to rewind its committed GFR pc. A read-only source record's original identity and lifecycle projections survive retry.
7. Native identity/history opaque to H2. Reducer preserves handle/capability reference and original first-entry data. It does not manufacture view/locks/signatures, change timer deadlines, overwrite first ingress or use a fictional new handle to represent restoration.

## Concrete independent miniature traces

### C1: commit rejection before any guard

Base revision r0 has A Running, B Preparing, cursor p0. A complete source record p1 selects B and closes A; successor state is not yet Ready. Planning returns an immutable proposal that would produce A Excluded+Terminated, B Selected+Preparing, cursor p1 and Close(A)/Prepare(B) obligations. Base r0 remains byte-for-byte unchanged while planning. External commit reports “not installed, no guards durable.” Reject/discard this proposal: accepted state remains r0 and no output is released. The source GFR may remain at p1; it is not rewound. Retry of the same source input yields the same logical intents/IDs under the same base.

### C2: guard persisted, later failure

Use C1's proposal, but H1 reports A's terminal guard durably installed before a later assignment/root step fails. Expected state must retain A as non-signable, even if record p1/cursor/new assignment remains pending. Never restore A Running via rollback. Preserve source input and obligation identities so retry cannot drop Close or duplicate Enter. A stale r0 proposal cannot supersede the newer safety fact. This is logical fail-closed simulation only; genuine fsync/root/guard atomicity is a future native integration gate.

### C3: same-pc batch with no intermediate engine execution

One valid Espresso ReplayRecord p contains `[SessionClosed(S0), SessionAssigned(A:S1), SessionClosed(S1), SessionAssigned(C:S2)]`. All events have p but distinct full lifecycle IDs. Logical processing retains the complete ordered identities and final public selection/closed facts. A is already closed at the end of the transaction: no executable Enter/Resume(A) may escape an intermediate assignment. Intent coalescing vs retained cancelled intent is a contract choice; resulting externally releasable effects must be equivalent. An empty-events record with a valid G APPEND still advances its exact cursor and applies applicable lower-anchor exclusion.

### C4: action receipt race

Plan Enter(A) as intent E with fixed id and original parameters; do not claim applied entry. Before E is released/receipted, accept a newer public exclusion or local stop. The state becomes terminal/non-signable and pending E is blocked/cancelled according to the frozen semantics. A late success-looking E receipt cannot set phase Running, clear the terminal cause, allocate a new handle or create another entry attempt. Retained history may record the reported older action if valid, but latest guard dominates.

### C5: early h11 vs later lawful h14

Use the unchanged independent oracle's ts*(h)=h seconds, T_win=0.2s, grace=3s, expiry=30s, local offset0, S0 window1–10, S1 window11–20, S0 stop11s/public end14s and S1 stop21s. Exact A9 successor is prepared before11s.

- Early path: B11 first received11.020s, authentic native final/body publicly complete11.300s; A9 selected14s. Original firstSeen and q_avail survive; later public acceptance can append padding10+physical11, subject to every height's expiry. Reducer does not generate these proofs.
- Late path: no earlier B11 exists; A9 becomes Ready at14s and enters while C_m<21s. It may request lawful production in the current slot14; B14 first received14.020s satisfies h14 interval `[13.998s,15.2s)`. A real B14 anchored A9 would cover padding10–13 and physical14 after H1/GFR checks. This is not h9→h11 success; no backdated B11, shifted window or fabricated firstSeen is allowed.
- Negative: first receiving newly generated B11 at14.020s violates its strict upper bound12.2s. The reducer cannot “repair” it by changing timestamps or windows. This test observes immutable context/intent metadata in the component; native Accept is still H1's responsibility.

## Review checklist before contract freeze

- Explicit internal capability issuer/validation assumptions, bindings, fixture labeling and how real H1 shared types will replace adapter placeholders. No bool readiness or production always-true validator.
- Exact action-ID formula/domain/encoding and collision-isolation inputs, ordinals and stable retries; input canonicalization order and integer bounds. Reuse O3/O7 public encoding; private reducer encoding must not be advertised as another public protocol.
- Transition table for Enter requested/applied, Pause/Resume ordering, failed and duplicate receipts, stale startup suppression, stable handle identity and phase labels.
- Commit proposal acceptance/rejection API, base-state binding, guard persistence evidence and fail-closed status, disallowing stale-plan rollback of irreversible safety facts.
- Full-record preflight/continuity and source identity; exact duplicate handling even after later cursor advancement, contradictory event/record identity rejection, no external intermediate effects.
- ClockUnsafe private cause/scope and safe recovery; Time stop equality; all terminal phases monotonic. GFR/public selection unchanged by local clock.
- Bounds/overflow behavior, no silent candidate eviction that would permit reallocation, outstanding task preservation and an explicit unsupported RESUME boundary.

## Component evidence and future gates

`case-matrix-v1.md` lists deterministic planned cases and their source rules. Later independent harness uses fixed manual inputs and expected transitions, separate from production tests and implementation-derived outputs. It may use fixture capabilities and synthetic source records only when explicitly marked. Golden private action IDs can be added after the formula is frozen, derived with an independent encoder; that does not alter this semantic oracle.

Record named candidate SHA/dirty state, contract/oracle hashes, tool/lock versions, exact command, fixed inputs/case IDs, outcome details and limits. Only after contract freeze and candidate naming may actual checking/build occur. Heavy commands require `flock /tmp/megaroation-poc-heavy.lock`, `CARGO_BUILD_JOBS=2` and the assigned data-disk target/artifacts; this preparation runs none.

A component PASS means finite pure logical traces passed against the frozen reducer contract. It does not verify actual H1 ReadyToken production, registry/final signatures, native Pause/SelectBack/deadlines, write-before-sign/root durability, OS recovery, native Accept/certify mapping, 15 workers, real Publisher/GFR pipeline or M1's R2-first-G endpoint. All corresponding native/system gates remain OPEN. M2 Force control activation, D/RESUME, batch/DA and Web remain outside this slice.
