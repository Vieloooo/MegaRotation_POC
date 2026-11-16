# M1 harness contract independent review — 2026-11-16

Reviewer: `/root/m1_verify`, bft_verifier (Astra/xhigh), using regional-bft-conformance.
Checkout: `/home/jy/.codex/worktrees/5d7a/megaroation_poc`.
Review type: read-only contract review and source-derived verification preparation. No candidate implementation or builder explanation was used to derive expectations. No build, worker execution, simulation, BLS test or candidate checker test has run.

## Named artifacts

- Initial source baseline: M0 `35d382ac445cc8bc779f0d18fd23cb57e54680da`.
- Current integration baseline reported/read during contract review: `daebb514d4bf1ea596b828e320257bf54b68ceaa` (M0 plus implementation decision, no H1/H2/H3 runtime implementation delivered here).
- Accepted implementation decision: `d55d39f73593676325bdc0fbb7ba3845ca93025f:poc/docs/milestones/M1-implementation-dispatch-20261116.md`.
- Independent expectations: `/tmp/m1-merge-verifier-expected.md`, SHA-256 `2abf64ce7000c272354490e70d6b28983a6e92ce84a8782684701735f34a13d3`. Frozen before reading the new contract; do not rewrite it in response to the candidate.
- Contract: `poc/docs/contracts/m1-healthy-regular-harness-v1.md`, M1-HARNESS-1.
- Initial review snapshot: `/tmp/m1-contract-reviewed-initial.md`, SHA-256 `e502efc9cd347cd9f885fcfa9b0fa2b7ad53423f095eebe8d89a45f0958117d7` (contains author's first source-alignment amendments).
- Final contract hash, independently read after Docer stopped writing: **`29e05ab86d156eab0473fd6a59a08cf2a5f962dfa3bf5c171d70dca941aeb647`**. Final diff was read against the initial snapshot; this review binds that exact hash.

## Outcome

The final contract is consistent with the source-derived M1 success endpoints, q4 profile, true h9→h11 preproduction/caching requirement, complete-result publication, public replay ordering and official same-byte readback limits. All contract clarifications below are resolved in the named final hash. **No blocking objection remains to the locally frozen mechanical harness/preflight scope.** This is not M1 PASS and is not acceptance of a future runtime or protocol checker.

Strong boundaries already present: separate BundleIntegrity / ProcessAndCustody / RegionalFinalAndDelivery / PublicReplayAndLifecycle / OfficialSameBytes; exact components and explicit NotReady ports; no candidate GFR expected-output generator; A/B equality alone insufficient; actual raw signatures/proofs/ancestry; no production always-true verifier; delayed publication does not prove PUB-4; official Unknown/Unsupported and unverified QC/VID retained; exact q4/q5 positive and subquorum/domain negatives; full matrix and finite bounds recorded as planned, not executed.

## Findings sent to author and lead

### CR1 — specify all workers' local replay/Manager provenance before topology freeze

- Source: v2 design §2.2 (each worker has independent Manager/BFT/GFR/cache/journal), v10 03 §1 and §2 (per-replica Manager; lifecycle comes from local public-rule replay, not a remote asserted choice).
- Reviewed location: contract §5.3 `readers` / `gfr_observed`, initial lines 135–137; §6 checker step 4, initial line 155. These explicitly describe A/B state but do not unambiguously require per-worker replay-state provenance.
- Impact if left ambiguous: a central mutable GFR/Manager or shared observer snapshot could provide decisions to all 15 signers and pass the described A/B comparison without implementing the required independent local protocol state.
- Required clarification: each worker has independent Manager and locally verified public-replay state/cache (or the lead separately approves and records a precisely equivalent deployment). Record reader/worker identity, input prefix/pc → complete ReplayRecord → Manager delta → durable receipt/action. Reject shared mutable ReplayState and unverified remote selection.
- Minimal future replay: give worker 5 a delayed public prefix while A is ahead; it must retain its own pc and cannot apply an unverified remote assignment. Submit a bundle claiming 15 Managers but providing only A/B shared state; the corresponding capability/evidence gate cannot PASS.
- Evidence status: source/contract static review only; no candidate trace exists. Not a reproduced implementation defect.

### CR2 — make independent PASS derivation and missing-oracle behavior explicit

- Source: v2 §13.3 (independent checker from original inputs; same-implementation viewers insufficient), accepted dispatch reuse/verification boundaries, skill independent expected-result requirement.
- Reviewed location: contract §4 status semantics; §5.2 manifest `status/gates/ports`, initial lines 115–117; §6 checker/oracle, initial lines 150–160.
- Impact if left implicit: a mechanical checker could hash a complete directory, read `verified=true` / `status=PASS` / ready-port declarations and accidentally promote unexecuted crypto/execution/replay checks to protocol PASS.
- Required clarification: all manifest verdict/readiness fields are claims, not evidence. Checker emits its own per-layer verdict only after the pinned corresponding verifier actually executes over required raw inputs. Missing H1/H2/H3 capability or independent crypto/execution/replay oracle stays `status=NOT_RUN`, `reason.code=NotReady`; no summary-only fallback. Missing/corrupt required raw evidence is reported explicitly, never as PASS. BundleIntegrity PASS cannot discharge another layer or M1.
- Minimal future replay: a structurally complete synthetic directory with internally matching file hashes and all manifest gates set PASS, but no independently implemented protocol oracle. Expected overall NOT_RUN/NotReady, zero protocol PASS gates. A copied manifest-only change from FAIL/NOT_RUN to PASS must not alter checker-derived verdict.
- Evidence status: contract-level false-PASS prevention; mechanical rejection test is planned, not yet executed.

### CR3 — distinguish preparation schema from runnable input (lead-requested clarification)

- Source: accepted dispatch permits incremental preparation before H1/H2/H3 interfaces/fixtures arrive; it forbids fabricating runtime evidence.
- Reviewed location: contract §5.1 input fields and §4 preflight.
- Impact: mandatory concrete fixtures/profile/expectations for preparation could pressure the shell to invent placeholder protocol inputs merely to report dependencies missing.
- Required clarification from lead: preparation may represent genuinely unprovided fixtures/profile/expectations as null and report precise NotReady reasons; real `run` requires all concrete inputs and valid bindings. Null never stands for an empty protocol fixture or zero value.
- Minimal future replay: minimal preparation with component/binding/fixture/profile/expectation nulls succeeds only as NOT_RUN/NotReady; same payload submitted for `run` cannot launch/sign/report PASS.
- Evidence status: requested schema clarification, not an executed defect.

### CR4 — close two minor run/schema ambiguities

- Contract §5.1 uses closed-schema unknown-field rejection while H09 requires referring to an already validated H01 source run. Specify where the source-run manifest/artifact provenance lives in input so H09 does not require an invented unsupported parameter or untracked external byte source.
- Contract §7 says H03 is a supplemental semantic run, but final release requires all mandatory gates. Explicitly say whether H03 is required once or only diagnostic; it must never substitute for H02.
- These do not change protocol rules or block independent preparation work once the author resolves the wording.

## Source-alignment amendments already observed during review

Final resolution check: CR1 is explicit at final contract line 29 and raw reader/GFR roles at lines 144–145; CR2's fail-closed verdict aggregation is at line 169, with the fake-PASS bundle case in H07; CR3's nullable preparation versus runnable-input requirement is at lines 101–112; CR4's H09 `source_run` is line 109 and H03 is explicitly non-mandatory diagnostic at line 189. The new M1-EXPECTATIONS-1 envelope (line 116) only references the existing independent source expectations and an initially null oracle binding; its existence does not assert an executable oracle. No new protocol encoding or guessed production API was introduced.

The author has already made these clarifications in the reviewed file:

1. Independent execution expected results cannot import the candidate GFR/Merge transition implementation.
2. At SessionEnd in H02, G remains at h9. Only a subsequent Espresso event (possibly the same public key, ordered later) can append padding10 + physical11, and its current admission key must be below both heights' expiration keys.
3. H05 has both exactly four and five valid distinct signer positive boundaries, alongside the rejecting vectors.

The proposed H02 sequence is source-consistent in principle: h9 candidate A can produce valid h11 before delayed regional h10 reaches the successor; higher candidate B can then become Ready/scheduled while h10 remains unpublished, so A is paused without G-based exclusion; SessionEnd selecting h9 resumes A's original state and excludes B. Exact microsecond timing, common-cid quorum intervals, transfer/processing bounds and the real API are still unprovided and must be checked before execution.

## Open implementation/safety gates escalated to lead

- H1: real 15-process/5-engine runtime and signed-step evidence; native proof namespace/PoP/participant mapping; exact ReadyToken/anchor-state verification; absolute timer and same-instance pause/select-back; controlled route/mailbox admission; local Accept WAIT/receive-time rules versus Commonware deterministic certify semantics.
- H2: complete lifecycle-record identity, immutable ManagerDelta/intents, single durable owner receipts, pending/applied distinction and terminal guard monotonicity.
- H3: bounded actual publication bytes, raw original positions and q_avail inputs, separate local/public incompleteness, native verifier binding, per-reader replay inputs, official actual-byte artifact interface.
- Independent checker: cryptographic verification, actual state-changing execution, and public replay expected-state derivation must exist and run; a mechanical shell cannot satisfy them.
- Source pin: the local submodule directory is uninitialized; expectations used the exact git object from the other initialized local clone without modifying either tree. Final candidate must record exact source/lock identities. No paper/whole-protocol proof is supplied by the finite M1 scope.

## Scope and replay evidence

Read-only commands used include source `cat/sed/rg`, `git show d55d39f...:<dispatch>`, `git ls-tree HEAD regional_BFT_sim`, `git submodule status regional_BFT_sim`, exact pinned Regional-BFT `git show`, cached Commonware 2026.10.0 quorum and protocol docs, and `nl -ba <contract>` for source locations. The only writes were separately assigned `/tmp` expectation/review snapshots.

No Rust build, real worker, deterministic candidate replay, cryptographic verification or official-node request has been executed in this preparation. Therefore no seed or runtime trace outcome is claimed. Planned mandatory system matrix is 24 H01/H02 runs with seeds 7/42/99 × OS startup orders forward/reverse × wall pauses 0/2 ms, independent negative checks/replay comparisons and one official same-byte run, with the contract's finite bounds. H03 is optional diagnostic and cannot replace H02. Future heavy operations require the lead's named candidate and shared `flock /tmp/megaroation-poc-heavy.lock`, jobs=2.
