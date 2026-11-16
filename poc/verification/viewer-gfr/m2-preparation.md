# Independent M2 harness preparation

Prepared from the frozen source expectations/contract after M1 review, before inspecting the M2 reducer or publication-processing implementation. Only the lead-authorized public state/query types and FixtureL1Reader adapter were read to bind the external harness. No M2 build or simulation has run. Rustfmt 1.95.0 parsed the new source successfully; this is not type-check or conformance evidence.

Owned source: `independent-harness/src/bin/m2.rs`, `src/m2_fixture.rs`, `src/m2_world.rs`; isolated harness Cargo manifest/lockfile. The existing M1 main binary remains the default. M2 will run with `cargo run --manifest-path poc/verification/viewer-gfr/independent-harness/Cargo.toml --bin m2 --locked --offline` only after a named candidate, under the global host lock, Rust 1.95.0 and two jobs. The harness lockfile will be refreshed offline then because new explicit dependencies were added; no other lockfile is owned here.

## Fixture identity and oracle

- Registry: unchanged accepted 03 `implementation-v1/public-registry.json`, loaded through actual `ValidatedRegistry` with key/PoP checking; actual `StaticScheduleProvider` constructs plans/descriptors. Regional final proofs remain explicitly **non-cryptographic exact descriptor/body/proof fixtures**. Their check counter distinguishes readiness from a bool verdict.
- L1: actual 03 `reg_a_l1_fixture()`, finite heights **0..30**. Hash at height h is 32 repetitions of `0x40+h`; parent is the previous row hash (zero for h0); timestamp is `2*h` seconds; allowed finalization is timestamp + 1.5 seconds. Empty-control commitment is the existing 03 canonical empty commitment. Every visible row is obtained by explicit `FixtureFinalizer` finalization and separate header/control delivery. Nothing outside this finite fixture is synthesized.
- DEP: verifier-owned generated extension, heights **1..40**, freshly committed into PublicConfig. Original first-three-height credits stay unchanged: h1 account0 +5, h2 account1 +2, h3 account0 +10; h4..40 have no credits. A separate finite-exhaustion fixture ends at h3. These are generated pure-rule mocks, not additional accepted production input or Anvil evidence.
- Execution: unchanged original genesis at height0, timestamp10 seconds, origin3, accounts `(0,nonce0,balance100)` and `(2,nonce0,balance7)`, original physical/native anchor IDs. Fixed schedule gives `timestamp(h)=10s+h*1s` and origin `(h+5)/2` for h≥1. L1 pace2 seconds, Tdep5 seconds, drift15 seconds.
- The seven ordinary transactions are copied from the accepted 03 h3 vector. Hand-computed retained indices are **1,3,4,6** and post-execution accounts are `(0,2,90),(1,2,15),(2,0,19)`. Before ordinary execution, credit-only balances are h1 `(105,0,7)`, h2 `(105,2,7)`, h3+ `(115,2,7)` with zero nonces. A late first physical target at h11 uses the same ordinary list after those credits.
- Expected virtual/native blocks are constructed from these literal account/Kept outcomes, explicit metadata/DEP, and existing canonical 03 hashing. **The expected-output path does not call Viewer, Apply or Merge.** The candidate uses actual 03 metadata/Merge/execution, so native content is checked against a separate account/Kept oracle. Generation of descriptor identity/encoding is delegated to already accepted 03 APIs rather than inventing another codec.
- Nonempty controls are explicitly named unauthenticated pure-rule fixture inputs at exact L1 positions. They exercise the unsupported-control boundary only, not 09 receipts or Force/RESUME execution.

## Source-derived event expectations

| Boundary | Expected transition |
| --- | --- |
| Genesis | G h0, pc None, NEXT session(0,0), window1..10; Plan Wait yields no state, None yields PENDING. |
| Proof public at q0/key13s; body q1/key14s | q0 commits with G h0; q1 appends h1..3, q_avail1 and acceptance pc q1, including padding. |
| B9 proof/target body but actual B3 missing | G stays h0; publishing B3 later makes descendant complete and appends contiguous h1..9. |
| Basis not ready / binding missing | WAIT at the proven Espresso position, zero bool calls for basis-WAIT, no partial state/effects. |
| Header reaches coverage but controls stop one block short | Unknown-frontier WAIT; filling complete controls allows original transition. |
| M_a20s, event13s, header needed through4, controls only0 | Missing origin4 gives exact-event WAIT; delivered header4 allows append without demanding controls1..4. Its checkpoint restores under the same split frontiers. |
| Successor B11 public q0/key21s, predecessor h10 q1/key22s | q0 contributes no G; q1 accepts through h11 in one event, successor keeps q_avail0 and acceptance q1; reference remains predecessor initial anchor. |
| Plan or successor metadata waits after tentative h10 append | Entire Espresso event rolls back; retry produces one record and one closure/assignment batch. |
| h9 at key20s then SessionEnd24s | G remains h9; successor anchor B9, reference A0, fixed window11..20. No terminal padding. |
| No physical result by SessionEnd24s | Successor anchor A0/window11..20; G stays h0. |
| Publication at23,999,999µs versus24,000,000µs | Strict-before accepted; equality closes old session first and cannot append old result. |
| G h9, later session window31..40, physical B31 | Key49,999,999 accepts padding h10..30 and h31; key50,000,000 expires h10 first and HALTs with G h9. |
| Expire14s (short Wexp3s) | Branch closes once, G retained, HALT. A Force/Resume at16s returns explicit unsupported at its position and cannot wake/skip. |
| Two complete windows in one Espresso event | Four distinct lifecycle members remain at one pc and one ReplayRecord; tested with non-live timestamp lag, not an E-TS/bounded-liveness claim. |
| Timer pc24s requires q0 watermark24s containing successor result | Timer checkpoint retains q0 as support but does not index/accept its messages until Espresso(q0); restoration has the same separation. |
| through=(24s,Espresso20), delivered tip=(24s,q10) | WAIT, because unseen q11..20 may share the key; never falsely READY. |

## Additional lead decisions

These are additive adapter clarifications, not edits to frozen v10 sources:

1. A `through` bound below the current pc returns `GfrError::HistoricalBound` and leaves the state unchanged. No rewind or misleading historical READY.
2. The typed in-memory GlobalCheckpoint exporter retains a private immutable binding of original genesis/profile/parameters. Tests obtain a checkpoint via `state.checkpoint()` and mutate public state/cuts; they do not construct a checkpoint literal. This prevents a modified parameter/profile from becoming its own trusted reinitialization basis when an early history would otherwise replay identically. No persistence claim follows.

## Coverage plan and limits

The harness emits individual check results plus a mapping of every VG01–VG40 and V01–V23. Cases not actually exercised are INCONCLUSIVE. V01 remains covered by the separate M1 result; VG39 is always marked deferred for future RESUME material staging; V22 is always open for actual native certificates. VG40 tests unavailable capability semantics only, not a native acceptance gate.

Coverage includes 23 checkpoint public-field/cut tamper variants, verified restore and next-handoff continuation, private A/B readers with different raw 06 delivery order, source regression/gaps, canonical target sorting with per-height source expectations, ordinary state-changing execution, and explicit bounded-input/arithmetic errors. G1/G2 seeded pause matrix remains the separate Builder/Tester agreed runner at seeds7/42/99 and pauses0/2ms; no matrix result is claimed during preparation.

This source-only preparation has no candidate verdict. Compile/API discrepancies may be corrected in the harness after a named candidate without changing frozen expected semantics. Any observed protocol mismatch must be reported with source, candidate line and replay evidence before repair.
