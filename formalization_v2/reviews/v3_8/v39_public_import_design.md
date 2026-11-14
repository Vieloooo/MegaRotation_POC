# v3.9 local import from retained public DATA

Date: 2026-10-19. Scope: new `sections/rotation_v3_9_public_requests.tex` only. The fragment is intended after the v3.8 installed-snapshot relay and before `INPUT_REPORT`. No frozen Model or v3.8 source is edited by this task.

## Result and limits

The fragment adds an explicit local path from already retained public DATA bytes to unvalidated request candidates and then, after the existing request checks, to the ordinary pending FIFO. Its first-carrier theorem is conditional on an original public `R0` actually containing the request and the node following and retaining its bytes. Rejected EXPORTs and closed branches do not suppress extraction. Import gives no old DATA append permission and never updates `G`.

The fragment does not prove formation of the first request-bearing `R0`, universal pre-first-report awareness of future rosters, or full request liveness. It creates no bare-request publication obligation, Ready hydration gate, global pacing rule, old-to-new message bound, or historical quorum service duty. No new wall-clock bound is claimed.

## Inputs, ownership and scheduling

The persistent node-level request owner holds two append-only journals and an unvalidated byte store. Source jobs consume actual already retained roots at a fixed dictionary cut. Their finite traversal follows original logical typed fields, including DATA bodies inside original proposals or ancestry. Unresolved references are recorded but never fetched by this worker; acquisition of an actual target independently supplies a later job. A fixed startup scan covers the finite store already held when the rule begins. Its target does not move with later source arrivals.

The candidate store records exact request entry bytes, occurrence paths and local arrival numbers. Its entries cannot be used as authenticated requests, reports or executable input. Before validation, deduplication is by exact bytes only. A forged claimed client ID cannot poison a future genuine request. Only a successful ordinary authentication/canonicalization/identity check can merge retransmissions by logical identity.

Checking jobs use an already completely prepared exact parent and its retained read context. A parent-preparation event schedules the finite candidate cut then held; a new candidate schedules its own check when a parent is already prepared. This avoids assuming that every application's client authentication can be separated from native-state admission. Jobs do not acquire AUTH, native state or source objects. A failed check applies to that byte string and parent only. Future prepared parents get their own checks.

The local loop alternates finite steps on the FIFO heads. New arrivals append at the tail. A fixed parent job retains its read context and completes even when the active parent changes. These are explicit protocol processing duties, not a claim that data structure choice proves a CPU, bandwidth, storage or real-time bound. Empty or unresolved external work cannot pin either queue. The loop is worker-independent: restart, closure, completion and retirement preserve its bytes, jobs, context and cursors.

## Compatibility with the current pending set

The v3.8 definition of `E_v(P)` ranges over *pending* requests, not every arbitrary byte string held by a node. The new raw candidate store is a separate pre-validation state. Successful checking invokes the ordinary request handler and enters its authenticated pending store; normal exact-parent filtering must finish before that updated eligible projection is exposed. No authenticated pending request may be skipped merely because it came from import.

For a new pending entry, its FIFO position is the first successful complete request-check event, as for the existing first complete receipt of an authenticated admissible request. Repeated valid receipt preserves the existing position. A successful check against an older parent does not certify the request at a newer parent. REPORT and FIFO filling continue to check current exact-parent native admission, fit and complete ancestry; global and speculative duplicates are excluded as before. Earlier entries in a proposed batch still affect the FIFO remainder's application validity.

This separation deliberately permits a report to precede import of a raw candidate. It does not permit indefinite omission: if the candidate reaches a completely prepared parent at which its ordinary checks succeed, a fixed finite job must finish and promote it. If the request remains admissible at later relevant parents, later report processing sees it through the unchanged pending-store interface. Immutable reports already signed are not amended. This is the same late-arrival boundary that v3.8 already exposes, not a new argument that every fresh roster is aware in time.

## Required integration hooks

1. **Request fragment:** include the new fragment immediately after the installed-snapshot relay and before `The first-DATA report`. Keep `E_v(P)`'s exact-parent native, fit and ancestry checks. A short cross-reference may identify `U_v` as explicitly outside pending until ordinary validation succeeds.
2. **Raw source retention / SOURCESTEP:** on retaining a complete raw namespace, durably append its retained root and fixed dictionary cut to the import source journal. Do this independently of whether P6 accepts any DATA. The synchronous part is recording the descriptor/obligation; do not await scanning or checking before SOURCESTEP relay, P6 or Ready.
3. **Object dictionary:** acquisition of an actual exact public target must append a source job for it. Enqueue actual objects, including original structured DATA children of rejected records; do not use sender object indexes or require optional witness validation first. An unresolved target creates no bytes and no wait.
4. **Already held store:** initialize one finite retained-root cut and keep new acquisitions on their normal append path. This is a scan of held bytes and must not be converted into a new Ready hydration wait or moving historical target.
5. **Native parent preparation:** when an exact parent is completely prepared under the original interface, record the candidate cut and retain the already available read context for the finite checks. On candidate arrival, schedule its check at the currently prepared parent if there is one. Advance of a parent must not cancel jobs or invalidate their retained read context. No absent dependency is fetched by these jobs.
6. **Request success:** after the actual client/domain/canonicalization/admission checks, merge only a proven retransmission, create or preserve the ordinary pending FIFO entry unless its identity is already in `G`, and invoke the existing installed-snapshot relay. Complete ordinary exact-parent eligible-index maintenance before exposing its update.
7. **Lifecycle:** the node-level owner preserves import journals, candidate bytes, pending enqueue obligations and active read contexts across worker exit, session termination, Closed and compaction. `G` membership may end an active pending entry; it does not authorize deletion of raw public evidence.
8. **Progress text:** cite `prop:v39-public-request-import` only as a conditional first-carrier-to-local-retention/pending result. The existing `prop:v38-request-inclusion` still needs accepted-report awareness. Do not remove that condition or claim a bound on import completion from `T_pub + T_E`.

## Source contracts checked

- `sections/rotation_v3_8_requests.tex`: canonical request encoding, original pending FIFO, exact-parent `E_v(P)`, immutable reports, regional sidecars and unchanged public `R0`, conditional accepted-awareness theorem.
- `sections/rotation_v3_7_source_step.tex`: retained raw namespaces including rejected records, independent source and replay cursors, actual dictionary objects, persistence across closure/compaction, and existing immediate source relay.
- `sections/rotation_v3_8_carrier.tex`: original typed DATA/evidence structure, source denotations, inline and retained-target inventory, optional-reference nonblocking rule and wrong-hint resilience.
- `sections/rotation_v3_8_exports.tex`: public-kernel validation and independent append gates; failed outer wrappers cannot hide independently represented objects.
- `sync_hotstuff_rr_part2_rotation_v3_8.tex`: the existing DATA tuple has a typed application batch; DRAIN has none.
- `revisions/v3_1_frozen/model_and_assumptions.tex`: original complete-result transport, separate native preparation, fixed-snapshot network guarantee, online/history-following public visibility, and the distinction between complete-result availability and first-result formation.

## Validation

Manual source-level consistency review. An isolated temporary copy of the v3.8 main and request fragment, with this fragment inserted at the proposed location, compiled successfully with `latexmk -pdf -interaction=nonstopmode -halt-on-error` (79 pages). The final log has no warnings, undefined references, or overfull/underfull boxes. The test changed no v3.8 source or frozen source. Its temporary output is `/tmp/v39-public-import-check-8qra63pr/main.pdf`. The actual integrated v3.9 candidate still needs its own compile. No page-render inspection, executable protocol test, benchmark, or full-protocol liveness certification is claimed.
