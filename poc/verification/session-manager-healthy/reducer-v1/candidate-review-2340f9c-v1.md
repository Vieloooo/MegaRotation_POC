# H2 reducer independent candidate review — 2340f9c v1

2026-11-16. Assigned bft_verifier, GPT-6 Astra / xhigh.

**Verdict: PASS for the frozen, trusted-input pure Manager reducer component and the finite traces below. No actionable conformance finding in the reviewed production code. Native Manager/H1 integration and M1 remain OPEN.**

The exact verified candidate is **`2340f9c1c8cbc3bc4be57a5c18f884cc74390563`**, checkout `/home/jy/.codex/worktrees/a6cf/megaroation_poc`. Git status was empty before and after the verification command. The `poc/crates/session-manager` Git tree is `78a9250a93da0c5db78578f19951bad5c04f6b80`, identical to initial candidate `129c6674356a98a908289a94df07802808603ff1`. C2 adds the independent capacity/overflow tests and coverage documentation, without production changes.

## Source-first and read-only workflow

The original 30-case oracle and 11 hand vectors remain unchanged. The reducer increment and 36-row case matrix were sealed before reading its contract or implementation:

| Artifact | SHA-256 |
| --- | --- |
| original independent expectations | `11f745e81faa46c03d85d2ee1ed722a842f8631ef62e96dd556e8610cece7ceb` |
| reducer source-first increment | `9ad71186b41e85a5e30100a7fd63d9a009435e12e4459046e3056ad8cd7965e5` |
| reducer case matrix | `4d78b0c12f65f7be5f0c5a2da0048b7815f3ef9e027290e942960ac39cd87395` |
| frozen H2-REDUCER-1 contract | `fe2f962d8480a0cb6b5d0bbb26ff6748d3e7149dfa6e6b05a9732c3c91c70a96` |
| independent.rs, sealed before candidate inspection | `d5178e8c262efb2c04f6301fa64094be5f9978fee68babbf31712ebcecbdba20` |
| fixtures.rs | `39b2b698b7cfde2116c7e89e11b3abbe0d1182d92a650ef0a10c5a96f4d2b2bf` |
| capacity_edges.rs, accepted C2 addition | `c98f3a81cac0b81ccaa32a5304c2b3708e92a9debd02274903432d6f251af46a` |
| independent harness Cargo.lock | `4c3c65a917b537a1fa3c941122db46c833c5c7d3e1846a09c52faeff6ade607d` |

Authority remains v10 02/03/04/09 plus explicit v2 mocks and the implementation dispatch at `d55d39f73593676325bdc0fbb7ba3845ca93025f`. Commonware remains2026.10.0/upstream `92a2f68a0e131c8b69de41e84aec37da6280f212`; Regional BFT gitlink remains `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`. Exact source contents/hashes and source-line mapping are in the preserved source manifest and oracle; no upstream code or public codec was changed by this review.

The independent harness was authored in `/tmp`, then the lead copied it into the candidate before testing. Its dependency paths were made relative; input/test bytes remained sealed. Only public API/getter declarations were read during call alignment. Full production source inspection began after candidate129c667 was named. This reviewer wrote no production or repository acceptance files and ran no merge/commit. New capacity tests were prepared separately, handed back, committed by lead as C2, and then executed on that exact SHA. No oracle was changed to accommodate candidate behavior.

## Executed evidence

Final command (the manifest records the full absolute cargo binary):

```text
flock /tmp/megaroation-poc-heavy.lock \
  /mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/cargo \
  test --locked --offline \
  --manifest-path poc/verification/session-manager-healthy/reducer-v1/independent-harness/Cargo.toml \
  -- --nocapture --test-threads=1
```

Explicit environment:

```text
RUSTC=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc
CARGO_HOME=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/h2-manager/cargo
CARGO_TARGET_DIR=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/h2-manager/target
CARGO_BUILD_JOBS=2
```

Actual versions: `rustc 1.95.0 (59807616e 2026-04-14)`, `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`. The host default was1.90.0 and was not used for the builds. The default sandbox initially rejected creating the assigned data-disk artifact directory as read-only; the authorized scoped execution escalation succeeded. There was no automatic-approval rejection, workaround to another service, or protocol/source change.

| Run | Result | Artifact and manifest hash |
| --- | --- | --- |
| C1 `129c667…`, `--test independent` | 22 passed,0 failed, exit0;0.18s test execution | `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/h2-manager/artifacts/independent-129c667-v1`; manifest `46a52396968104d859069b1b9d217f902b98af31b13a672fde0738d6f74bcb9d` |
| C2 `2340f9c…`, complete harness | **27 passed,0 failed, exit0**:5 capacity/overflow tests plus22 original tests;0.06s+0.18s test execution | `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/h2-manager/artifacts/independent-2340f9c-v1`; manifest `c351fdfbbea1f318d99fbe113e3e0bf49fe7bead8e7a3904c33398aeaaf2b566` |

Final stdout SHA-256: `f7e7941e8438818856096a5cdd1eede1577a896fe1a4d2bb81329a85d676af3d`. Final stderr SHA-256: `0d1c313f355611a5160ffa78b344539152e08fd414311162ca36c65645ac3d15`. Manifests preserve exact commands, candidate/tree IDs, input hashes, versions, exit codes and clean states. Test durations are wall diagnostics, not protocol virtual latency. Inputs are deterministic hand-written fixtures; no random seed or native simulation was used.

## Rule-to-code audit and replay evidence

All locations below refer to C2's unchanged production tree. These are inspected conformance locations, not reported defects.

| Source rule | Candidate location | Impact checked / independent replay evidence |
| --- | --- | --- |
| SP170–192; SM198/246–257: unique descriptor, once Enter, fixed state identity | `src/lib.rs`:459,493,553,710; `src/actions.rs`:152,271,419 | Same descriptor retains one Create/Enter; conflicting metadata/cid/lineage or PreparedRef bindings reject. Actual entry is recorded only by matching receipt. Unique-descriptor/preparation, fixed ActionId, wrong-predecessor/window tests passed. |
| SM214/224/257/263; dispatch30: highest Ready, local pause vs public exclusion, original instance | `src/actions.rs`:202–307; `src/replay.rs`:249–284 | Higher Preparing leaves A selected locally; Ready B must park A; unconfirmed Enter also requires parking. SelectBack retains original handle/entry/action history. Publicly chosen terminal A still excludes B. Candidate-switch and pending-entry tests passed. |
| SM225/230/240–242/259; V2:216; dispatch29/31: terminal/unsafe monotonicity | `src/lib.rs`:579–677; `src/actions.rs`:11–76; `src/state.rs`:94–99 | All relevant phases close; stop equality blocks dispatch; Unsafe uses private cause and fixed session fence. Late prepared/entered receipts cannot resurrect; local clock does not invent public cursor/lifecycle. Stop/unsafe/late-receipt/scope tests passed. |
| GFR219/379/418; O7 LIFE-1; dispatch28: atomic complete record and exact identity | `src/replay.rs`:19–208; `src/lib.rs`:187–209,219–254 | Whole record is staged in a copied state; malformed last event/effects/input_ref/source/gap cannot partly advance cursor. Same-pc events remain distinct; intermediate closed assignment creates no engine; empty records still advance. Record/multihandoff/replay tests passed. |
| SM275–276: accepted predecessor physical tip rules out lower anchors | `src/replay.rs`:74–95,308–359; `src/lib.rs`:516–535 | Trusted APPEND source updates tip and excludes lower candidates atomically; later low candidate cannot reopen. A higher regional candidate alone does not publicly kill lower A. APPEND-tip/barrier test passed. |
| dispatch28–29: immutable plan, exact commit, sticky failure guards | `src/lib.rs`:176–290,293–385 | Cause binds input and local time; base/context identity rejects foreign/stale plans; exact commit replay idempotent. Failure latch keeps terminal/tombstone while cursor remains unaccepted, disables dispatch and has no recovery entry. Commit/time/fault-latch tests passed. |
| SP once-enter; dispatch28; PUB135–139: stable obligations and correlated receipts | `src/actions.rs`:79–149,310–445; `src/lib.rs`:388–423,680–706 | Typed IDs retain original revision; failed pending actions retry same ID; completed/cancelled stale failures are unchanged; conflicting success rejects; delayed success records history without reopening. Publish identity survives local stop. Receipt/typed-ID/task tests passed. |
| fixed checked bounds in contract§7 | `src/lib.rs`:24–33,459/691/710/745–773; `src/actions.rs`:84–92; `src/replay.rs`:25/56–64/157–205/218–222 | Explicit Capacity/Overflow rather than silent eviction/wrap; tested boundaries below. Native resource behavior is not inferred. |

Code review also confirmed no reducer networking, signing, native engine execution, filesystem commit, background scheduling or second public wire codec. `CommitRef::for_delta` and private observations are explicitly trusted assertions, not authentication. The externally supplied H1 owner must make their facts true.

## Capacity coverage and conditional reachability

These cases are distinguished rather than all labeled simply “untested”:

| Bound | Independent evidence / limitation |
| --- | --- |
| Programs64 | Actual pure reducer trace: six sets of10 legal typed successor candidates under successive selected predecessors, then four of the seventh set;65th rejects atomically. No private-state injection. |
| selected/closed sessions128 | Exactly128 combined states accepted;129th rejects with cursor/state unchanged. |
| branch tombstones16 | Exactly16 typed Expire scope facts accepted;17th rejects. This is a structural consumer-capacity fixture; it does not claim M1 activates16 branches or exercise Force/RESUME. |
| complete source records2048 | Exactly2048 accepted;2049th rejects with no pruning or cursor change. |
| ResultKey tasks256 | Exactly256 accepted; exact duplicate retains identity;257th distinct task rejects atomically. |
| lifecycle events64 |64 accepted;65 rejected before partial effects. |
| effects128 |128 typed no-op Duty projections accepted;129 rejected before cursor change. This is an envelope-bound fixture, not a claim that actual GFR emits redundant Duties. |
| time/height arithmetic | Public input makes computed timestamp overflow; separate near-u64-max genesis/window makes h_max+1 overflow. Both return Overflow instead of wrapping. |
| descriptor16KiB / lifecycle projection1MiB | Guards inspected at lib725–730 and replay197–205. In valid O3's fixed3×5 profile, all descriptor fields are fixed-width except vectors with fixed counts; the largest Handoff descriptor is960 bytes. The largest supported M1 assignment projection is998 bytes, so64 events are at most63,872 bytes. These larger byte caps cannot be reached by a valid supported descriptor/event list; RESUME variable-prefix input is explicitly unsupported. No invalid constructor/private-state bypass was added just to force a byte cap. |
| dependencies64 / action ledger4096 | Allocation guards inspected at actions84–92. Under the trusted **one fixed Plan per session, one same-chain anchor per physical height, monotonic public choice** premise, a predecessor admits at most10 physical anchors plus its initial anchor; each program retains only its current pause and close barrier. The supported activation dependency set is therefore below64. The normal lifecycle allocates Create/Prepare/Enter once, monotonic candidate preference parks a cid at most once before its unique public selection, SelectBack does not repeat Enter, and Close is at most initial plus one late-Create followup. This gives a conservative7 intents/program plus256 Publish tasks,704 at Programs64, below4096. These are conditional state-machine bounds, not proof against forged trusted inputs or future profiles; the defensive capacity guards remain. No executable exact4096/65-dependency boundary is claimed. |

There is no independent arbitrary-state exploration or global unreachability proof. Named map/list caps are not a claim of an overall perpetual-service memory bound: the exact-commit history is retained in `Manager.commits` (`lib.rs`:40,283–288) and is not separately bounded by this contract. The evidence covers finite supplied histories; future service/profile claims need an explicit history-retention bound.

## Open production assumptions and handoff

1. Real H1 API/runtime integration remains unverified. The lead's dependency inventory reports that `healthy-session-api` has not been delivered as a compilable crate, while the current H1 API document is still an untracked freeze candidate. This review did not inspect that separate implementation. No dependency on completion of01 is introduced.
2. H1 must authenticate ReadyToken/PreparedRef facts with actual exact-anchor execution, persist terminal/scheduling/root/aux state before effect release, preserve the original engine/futures/mailbox/absolute deadlines and original firstSeen, and enforce clock/stop guards at the real signature出口. The component cannot establish these facts from Hash32 or typed Eq.
3. H1's accepted deterministic payload validity versus node-local admission/WAIT/certification mapping remains necessary, as stated in pinned Regional BFT DESIGN:47 and Commonware simplex/mod.rs:108–115. Pure fixtures neither settle it nor license an always-true verifier.
4. H3 must supply complete authenticated local GFR records with continuous private source sequence and truthful APPEND source/physical-target semantics. H2 does not reconstruct that public history or independently validate final proofs. Source cursor acceptance never rolls H3 pc back.
5. Pending Pause/Close/Enter/Publish obligations require correlated terminal outcomes from their owners. The reducer deliberately waits on safe parking/closing dependencies; progress of those external receipts is an integration premise, not proven by the pure component.
6. The early-h11 and late-lawful-h14 tests check distinct reducer context/intent/reference traces with fixed11–20 windows. They do not generate real h11/h14 blocks, execute Accept, verify signatures, determine real q_avail or append to G. Real early h11 production/admission and the R2-first-G endpoint remain required at M1.
7. Native/OS restart, D/RESUME, batch/DA, Force/M2, Web, actual15-worker scheduling, real publication and official Espresso same-byte compatibility are not exercised. Ordinary terminal/signing durability and controlled grant/Ack remain requirements for later integration.

No source requirement was waived and no safety/liveness assumption was silently closed. Tester may now replay the agreed component suite on **the same C2 SHA**. A documentation-only later commit may cite this exact tested SHA, but cannot be relabeled as newly executed native or system evidence.
