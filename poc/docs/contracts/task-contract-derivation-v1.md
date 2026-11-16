# Task 12 — independent D and exact ResumeCut, v1

Date: 2026-11-16 (Asia/Shanghai). Status: **FROZEN Stage A by task lead after independent source preparation**. This document resolves the Stage-A decisions in `derivation-preparation-v1.md`. The verifier derived expectations before any task-12 implementation existed and agreed G0, L0 and the separate origin rules; the lead selects atomic whole-marker staging. It does not accept the complete feature. Rust API spelling is recorded in the first interface candidate without changing these semantics.

## 1. Scope, ownership and dependencies

Stage A is a small complete component: certified-from-PoC-genesis initialization, canonical finalized L1 input coverage, no-batch sequencing-window fallback, accepted-data WAIT, ordered CursorSet fill/reset, immutable position-bound cuts, and validated checkpoint/restart. It has no batch-frame encoder or decoder. Stage B consumes feature 11's sole frame/DA codec and real accepted payloads, then adds ordinary batch adoption, substitution and the actual-batch integration matrix. Stage B remains an explicit gate.

The assigned implementation is the independent workspace `poc/crates/derivation/`, with its own lockfile. Independent expectations and verification belong to `poc/verification/derivation/`; this contract and task-specific reports are the documentation paths. No edits to root Cargo manifests, shared types/core, other features, native recovery, `regional_BFT_sim`, or `poc/docs/commander/` are authorized here. One writer owns a path at a time.

| Dependency | Exact revision / responsibility |
| --- | --- |
| 03 | `e9c15e04bc83b335668f3ecd837367aa87344a89`; accepted canonical objects, execution, fixed deposits and fixtures |
| 09 | `c443ae925ed549b046731c2d8ef3dd9ce0311d24`; real successful receipt events and explicit local Anvil driver; A01–A17 component, not full-system acceptance |
| Integrated baseline | `37b4f2b8a6d67b303ba90b1d4d1981c93ed8dd12`, normal merges preserving both histories |
| 11 | Exact interface SHA pending. Sole owner of frame/payload/DA encoding, commitment resolution and actual accepted-record interface |
| Runtime integration | Future controlled grant/ack integration; this pure/read-only component cannot claim runtime or 15-worker completion |

Use the existing worktree `/home/jy/.codex/worktrees/29df/megaroation_poc`, branch `codex/poc-derivation`. Regional BFT remains `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`; Commonware remains 2026.10.0. Do not push or merge main. Heavy commands use `flock /tmp/megaroation-poc-heavy.lock`, two build jobs, and this task's `.build` on the data disk. Public services run only on task-owned local ports; source checkout and old artifacts are read-only.

## 2. Authority and explicit substitutions

Normative project rules are v10 [`09` §§5.1–5.5](../../../formalization_v10/09_finality_rules_zh_v0_1.md#5-derivation-rule), v2 §§10.2–10.3 and v1 §§11.1–11.6 as retained by v2. The fixed native delta is the delivered MegaETH snapshot `a738f9b14c7880953d8cff8788db14e8e3e79124`, not a moving upstream branch. Its pages expressly describe an incomplete, unstable, unactivated design. This component claims the named P0 subset, not deployed MegaETH conformance.

| Source | Used rule | SHA-256 of materialized file |
| --- | --- | --- |
| `megaeth_specs/specs/rollup/derivation.md` | selection, canonical origin, batch checks, substitution, ordered override | `ba73e945918d935fef3e3e34c07da0f798589fcb1e550d2eae97859d00dbe140` |
| `megaeth_specs/specs/block-certification/activation.md` | actual-parent fill, including-block cap, non-wrapping arithmetic | `2522bf7243124f7f90d7a88f39f43bd8f7cd7e5e62a37d599b2f86ecb9ab505b` |
| `megaeth_specs/specs/block-certification/derivation-pipeline.md` | certified deltas Δ3–Δ6 and separation from engine INVALID | `b5ef11bb70cdfb50b54766f38463decf16858d17712c637080eb4f062b2b6a25` |
| `megaeth_specs/specs/block-certification/batch-inbox.md` | successful acceptance and override events | `e150f038b2bf2473aace803f20edba011731b851a9513c8bde7d176816787710` |
| `megaeth_specs/specs/rollup/data-availability.md` | accepted commitment resolution and waiting | `d2782bd7fac9f7df23dc8db3b7e3ec97ebe839ce2ce280824dde52fd5b7e9ed2` |

The inherited [OP Batch Queue source](https://raw.githubusercontent.com/ethereum-optimism/specs/42b446f5891dd5a232cfc346201a613c1bba3f50/specs/protocol/derivation.md#batch-queue) is pinned to `42b446f5891dd5a232cfc346201a613c1bba3f50`. Docer independently retrieved its complete Batch Queue excerpt on this task through Parallel, extract `extract_c570c06ac9a4813f0491b832989087ec`. The explicit ordered queue rules take precedence over the overview's half-open interval shorthand. The local source content and independent expectations, not an implementation's explanation, determine the assertions.

P0 choices are: certified interpretation starts immediately above the configured genesis anchor; no legacy/bootstrap activation implementation; the feature-11 single complete frame (its exact codec remains feature 11's responsibility); fixed height-indexed `dep[h]`; feature-03 light Apply/native encoding; append-only finalized local L1 with a trusted private Anvil driver. No L1 consensus/MPT verification, reorganization of finalized history, native EVM engine INVALID, real deposit allocation, TLA, staking or slashing is claimed. No G, Espresso, leader, wall clock, or scheduler future state is a derivation input.

## 3. Canonical objects and API boundary

Reuse feature-03 `L1Position`, `L1Header`, `L1BlockRef`, `Metadata`, `ConfigRef`, `ExecutionAnchor`, `ExecutionState`, `VirtualBlock`, `PhysicalBlock`, `NativeBlock`, their checked encodings/digests and `derived_anchor_id`. Reuse `poc_core::execution::apply_with_byte_limit`, `DepProvider` and `l2_timestamp_us`. Do not copy their encoding into this crate.

Feature-03 regional origin selection and its unconditional drift checker are not D's origin-selection algorithm. D constructs canonical `Metadata` from the rules below and checks the applicable ordinary or deposit-only conditions. In particular, a source-authorized empty drift exception or CursorSet fill beyond the including L1 timestamp must not be rejected by applying the ordinary regional bound afterwards. The canonical header still must be available and the L2 timestamp cannot precede the selected origin timestamp.

**P0 source-binding choice L0:** distinguish the bounded 03 fixture profile from the real 09 driver profile. A task-owned `DerivationConfig` binds the 03 `PublicConfig`/reference, positive sequencing-window size and profile version. For the driver profile it additionally binds 09 chain ID/configRef, Inbox/Authority/submitter addresses, actual L1 genesis hash/timestamp, slot parameters, named reader and adapter/interface revision. The 09 configRef used by contracts/driver must equal the selected 03 public-config reference. The existing `PublicConfig.l1_fixture_hash` remains its original finite-fixture commitment; it does not authenticate arbitrary future Anvil blocks. Do not hash the complete signed future L1 history back into that field, which would create a circular binding. Driver-profile initialization may verify genesis and `ValidatedDepFixture` directly, without invoking the 03 all-fixture `check_setup` path. Later headers/receipts are authenticated through the separately bound 09 source capability. This is a local PoC integration boundary, not a new L1 consensus proof.

Reuse feature-09 `BlockWire`, `EventRecord`, `EventWire`, `LogWire`, `CursorWire` and `BatchWire` where applicable. Its `LogWire.log` is the transaction-local index, while `rpc_log_index` is block-global. The mapping to 03 `L1Position` is exactly `(block, tx, log)`. A JSON adapter may strictly convert existing driver results; it is not a new receipt or batch wire format. Successful `Driver::Read` output must remain bound to its reader, configuration, finalized visible block hash and complete transaction/receipt bundle. A caller's `finalized=true`, chosen height or hand-constructed event is not real receipt evidence.

The production source capability is a named 09 `Driver<R>` reader query (or deterministic replay of its complete RPC/command evidence through the 09 driver checks), not a public constructor that trusts a deserialized `BlockWire`/JSON archive. Tests may have an explicitly fixture-only source, whose results must be labeled as such. The fresh no-frame runner deploys its own contracts and supplies empty blocks plus RESUME transactions; it must not reuse the old 09 scenario's already-accepted opaque batch as if it were feature-11 data.

The public semantic interface, with Rust spelling to be finalized by the lead, is:

```text
Init(config, configured_genesis, dep_provider, verified_l1)
  -> State | WAIT(missing) | REJECT(reason)
Advance(state, verified_l1)
  -> {state, new_derived_items, new_cuts, status}
DeriveAtResume(state_or_verified_history, exact L1Position)
  -> immutable ResumeCut | WAIT(next_position, missing) | REJECT(reason)
Checkpoint(state) -> nonrecursive checkpoint
Restore(checkpoint, configured_genesis, verified_l1, required_inputs)
  -> replay-validated State | WAIT(missing) | REJECT(reason)
```

`ResumeCut` carries the exact RESUME position, matching CursorSet position, target height, immutable D prefix as of that CursorSet, target `ExecutionAnchor`, and input/configuration evidence references. The prefix may extend above target; the target anchor's state is the state at target, not the cut head. `DerivedItem` carries canonical block material, actual post-state/root, native hash, and why that block was emitted. These are task-owned semantic containers around 03 objects; no shared encoding is introduced. `DeriveAtResume` is read-only and cannot rewind live D.

For Stage A, encountering any real `CursorAdvanced` stops at that position with `WAIT(Feature11AcceptedPayloadRequired)` regardless of whether arbitrary caller bytes are offered or the external cursor fields appear late. There is no public fixture resolver that can certify those bytes. Pure predicate/test fixtures may describe records without defining a serialized payload or claiming L1/DA verification; actual batch implementation and its accepted-data completion path are Stage B.

## 4. State, initialization and output evidence

Persist or deterministically reconstruct all of:

* configuration and genesis bindings; actual D parent/native hash and execution state; complete derived prefix and per-height state needed for historical anchors;
* verified canonical header/receipt coverage, the exact last completed input position, fully scanned block frontier, and unresolved input position;
* queue `current_epoch`, actual parent's origin, and first-output evidence; these are not interchangeable with downloaded-header height;
* buffered/unfinished channel state and reset occurrences (empty in Stage A), accepted commitment obligations, immutable cuts and their evidence references.

`Init` checks the configured genesis anchor, state metadata/root/native hash and config/DepProvider binding using 03 objects. It obtains the canonical genesis-origin header and validates the initial metadata. It does not import G state. **P0 choice G0:** the configured genesis is a trust anchor, not an output emitted by this derivation process. Initial `current_epoch = genesis.origin`; `epoch_has_output = false` has exactly this provenance. This is an explicit adapter decision, not an OP requirement about genesis.

At every completed output, `current_epoch` becomes that output's actual origin; output evidence is recorded for that origin and `epoch_has_output` becomes true. Ordinary adoption, substitution, fallback and override output all establish this evidence. Merely loading a header, advancing an L1 cursor or restoring unchecked fields does not. There is no empty queue-epoch advancement disconnected from an actual output in this P0 state machine. After an output at `e+1`, subsequent rules use that output as parent and `e+1` as current epoch, with first-output evidence already present.

The first-output guard applies only to ordinary sequencing-window fallback. An otherwise valid ordinary batch may advance from the genesis origin to `e+1`; the genesis choice does not add a new batch rejection rule. A non-genesis checkpoint cannot choose an arbitrary first-output flag. Replay reconstructs the evidence, then compares it with the checkpoint.

All height, origin, timestamp and window arithmetic is checked. Overflow is a typed rejection/terminal configuration error, never wraparound, saturation or guessed progress. A finite known fixture exhaustion is reported explicitly, not converted to a fictitious verified header or empty deposits. Truly pending required input returns WAIT.

## 5. Complete input frontier and event order

Read and process canonical finalized L1 in `(block, transaction, transaction-local log)` order. Preserve each block hash, transaction hash, actual receipt status and event pair. Failed submissions, successful calls without the configured acceptance event and unrelated log emitters create no accepted batch. They nevertheless belong to complete receipt coverage. Initialized cursor and Force records do not create D blocks; successful initialization is validated as input history, and Force cannot reset D or create a ResumeCut.

Separate three facts: (a) a canonical header is known, (b) the complete transactions/receipts and relevant events of that block are verified and available to this reader, and (c) replay has fully scanned/resolved those inputs. A high header or end-of-block cursor alone does not prove (b) or (c). Mark block `B` fully scanned only after all of its inputs have been visited in order and no unresolved accepted input at/before its end remains. A `CursorAdvanced` with unavailable interface/data prevents this frontier from crossing its position, including before a later CursorSet in the same block.

Process a supplied block's protocol events before its end-of-block fallback opportunity. Thus material loaded for a later event cannot leak into an earlier cut; fallback based on complete block `B` cannot appear before an earlier CursorSet in `B`. Canonical headers may be prefetched, but that does not advance the processed-input frontier. Input delivery timing changes WAIT duration only, not inclusion, ordering, output or cuts.

Same exact canonical inputs are idempotent. Conflicting headers/events at a previously bound position, nonextending finalized history, wrong configured contract/configuration, malformed position conversion or inconsistent pair material is REJECT without rewriting the established prefix. A single incomplete action is atomic: preserve all state and outputs completed before it, but do not publish a partial CursorSet effect or cut. A retry of that action occurs at the same position.

## 6. Ordinary no-batch fallback

Let `P` be the actual D parent, `e = P.origin`, `t = P.timestamp + l2_slot`, and `W = sequence_window_blocks` (a positive derivation configuration value bound alongside the 03 config). After evaluating available candidates, fallback requires no acceptable candidate, the complete processed input frontier through **block `e + W` inclusive**, and authenticated canonical header `e+1`. An unresolved accepted input forbids calling the window empty. Header availability alone or an elapsed local timeout cannot authorize fallback.

Choose origin `e` if `t < header[e+1].timestamp` **or** there is no derived-output evidence for the current epoch; otherwise choose `e+1`. Construct exactly height `P.height+1`, timestamp `t`, empty ordinary transactions, and the selected origin's canonical metadata/required inputs. The first branch is not changed into an ordinary drift rejection. Execute, emit and then reevaluate from the new parent/epoch. Following a transition to `e+1`, the next ordinary fallback requires the complete frontier through `(e+1)+W`; the prior window is not reused as if the epoch had not changed.

Reachable examples, in seconds for readability (implementation uses checked microseconds):

* Genesis origin 10 at time 100; header 10 time 100, header 11 time 104; L2 slot 4. With full processed input through 50 and `W=40`, first fallback is height 1/time 104/origin 10 because genesis has no emitted-output evidence. Next fallback is height 2/time 108/origin 11. Another fallback waits until block 51 is complete.
* With the same origin/header times and slot 2, first fallback is time 102/origin 10; the next at time 104 advances to origin 11 because the preceding output supplies evidence. Restoring after time 102 must preserve this choice.
* With complete frontier only 49, missing canonical 11, or an unresolved accepted payload at block 50, no first fallback is allowed. Each has a distinct missing-input reason.

These are semantic vectors. Tests requiring L1 heights outside the accepted 03 finite fixture must use fresh feature-09 driver-generated local inputs. They cannot append fabricated rows and label them verified 03 data.

## 7. Execution and derived identity

For any permitted output, fetch `dep` for its actual chosen `Metadata` from the config-bound 03 `DepProvider`, then invoke 03 Apply on the **actual D parent's state**. Attributes come from chosen metadata, while deposit credits remain the v2 fixed-height abstraction. Pending metadata/deposits return WAIT; an authenticated corrupt or exhausted fixed fixture returns its explicit error. No empty-DEP substitution is allowed.

Construct 03 `NativeBlock` with actual previous D native hash, actual metadata, actual DEP, Apply's actual `kept`, and actual post-state root; use its existing canonical digest. Store the corresponding virtual/physical content and pre-state root consistently with v10 §5.1. D's executed ordinary content is Apply's Kept, and source-record transactions (in Stage B) remain separately attributable if some were filtered. The derived anchor identity is 03 `derived_anchor_id(height, native_hash)`. A cursor hash, physical hash or equal height does not identify the D native block.

No G parent, source G state, source native hash or source batch parent is an execution shortcut. Ordinary wrong-nonce/insufficient-balance transfers are filtered by 03 Apply, not converted into whole-block INVALID. Deposit arithmetic failure is an explicit critical execution error; it does not authorize skipping the height or advancing the input.

## 8. CursorSet and exact historical cuts

Use the successful same-receipt pair that feature 09 verifies: `CursorSet.resume` identifies one concrete `Resume`, whose `set` points back to the same concrete CursorSet. Require equal block/transaction/transaction hash, exact log positions, target agreement and configured emitter/context; preserve both local and RPC log indexes. Missing needed pairing evidence is WAIT; contradictory supplied pairing is REJECT. Never pair by latest sequence, target height, end-of-block cursor or block height alone.

At the CursorSet position:

1. Resolve all preceding accepted inputs and process complete channels under their rules. Stage A therefore waits at any preceding CursorAdvanced. Future/unfinished batch state is abandoned only under the eventual Stage-B reset rule, not by treating unavailable bytes as absent.
2. Stage a reset of unfinished channel/pending state. With P0 complete single frames this normally resets an empty state; record the reset nevertheless.
3. Let `N = next` cursor height and `L` be this CursorSet's including block. If D is below `N`, fill consecutively through `N`. For each actual parent origin `e` and next mapped time `t`, choose `e+1` only when representable, `e+1 <= L`, and its canonical timestamp `<= t`; otherwise keep `e`. When the potentially eligible next header is missing, WAIT. If `e+1 > L`, do not require that future header merely to retain `e`. Always require the selected origin's inputs.
4. Each fill uses empty ordinary transactions and the execution rule in §7. It does not wait for sequencing-window expiry. The cursor's supplied origin is not the fill origin. Override target time may exceed `L.timestamp`; bootstrap's timestamp bound does not apply here.
5. If D already reaches/exceeds `N`, preserve its entire established head/prefix; do not fill or rewind. For cursor height 5, actual D height 10 and target 8, the cut prefix ends at 10 and its anchor/state are from height 8.
6. Atomically commit reset/fill, processed position and an immutable cut under the **RESUME** position. Cut identity includes the paired Set position and evidence, even though its snapshot is the effect at Set. A later replay of the same pair returns the same cut. Different material at that key is an error, not replacement.

If any fill input is missing, none of this CursorSet's staged fills/reset/cut is externally committed; retry from the pre-marker state. Implementation may compute privately until ready, but its exported state cannot show half a marker. This is a P0 transactional API choice consistent with exact cuts and repeatable retries; it does not change the final canonical chain.

Two RESUMEs in one block are distinct. Feature 09 requires increasing cursor targets, so real P0 vectors obey that acceptance rule. Intervening inputs affect only the later cut. After a marker, subsequent processing starts above the actual D head, not above the supplied target. `DeriveAtResume(p)` returns the immutable cut, or replays verified history to that exact marker without changing live D. A not-yet-covered position returns WAIT; a fully covered position proven not to be a RESUME returns REJECT. An empty block number alone is not a query key.

## 9. Checkpoint and restart validation

A checkpoint records the configuration/genesis binding, input position and complete frontier, actual parent/state/prefix, epoch output evidence, pending obligations and reset state. It does not recursively embed checkpoints or a self-referential cuts map. Store cuts separately or refer to canonical input positions; restoration must reconstruct/validate every previously exposed cut and its binding from verified history. A local digest verifies byte integrity only, not semantic validity.

For Stage A, the simplest accepted validation is deterministic replay from the configured genesis to the checkpoint's exact completed frontier/position using the same verified L1 inputs, followed by comparison of all consensus-relevant fields and historical cuts. Missing replay materials return WAIT. A changed prefix, state, epoch flag, claimed complete block, skipped acceptance/override or config binding returns REJECT. Do not turn an unverified arbitrary anchor into a fresh genesis. Resume live processing only after verification; the restored next-step result and each older cut must equal uninterrupted replay.

The component's local save/load or returned bytes are not controlled external protocol outputs. Future service integration must route publication through the authorized grant/ack boundary; this stage adds no bypass to native runtime.

## 10. Stage-B rules frozen as obligations, not Stage-A PASS claims

Feature 11 must supply one verified accepted-frame path whose completion is bound to the actual successful finalized `CursorAdvanced`, immutable DA commitment and its single codec. Stage A must not preempt its encoding, silently accept guessed payloads, or claim a fixture resolver verifies signatures/DA. A mismatching offered DA object is rejected as an object; the accepted input remains unresolved at the same position and cannot become an empty batch. Retrieval delay never changes its actual inclusion block.

On actual typed records from that path, preserve OP check order with MegaETH's deltas: future timestamp waits in the queue, old timestamp drops; ignore supplied parent/hash as prescribed; lateness is strictly `origin + W < inclusion`; then evaluate origin continuity, canonical origin time and drift. Ordinary transaction syntactic validity uses the 03 light opcode rules. Malformed input is not same-height substitution. The first acceptable record in canonical inclusion/wire order wins. Missing canonical inputs retain the record and suffix rather than marking them invalid.

Only after the next-timestamp and earlier guards pass, origin below actual parent, origin above parent+1, timestamp before canonical origin, or the specified excess-drift failure becomes immediate same-height deposit-only substitution. Empty advancing-origin and empty same-origin-with-next-time-later drift exceptions remain. Replacement origin is actual parent origin `e` when `t < header[e+1].timestamp`, otherwise `e+1`; missing required headers cause WAIT. This rule does not inherit the fallback first-output guard. Consume the substituted record; recheck the next record against the replacement parent. Late, malformed, old/future and unrelated errors do not become substitutes. P0 does not implement native engine-level INVALID.

Stage B must run real feature-11 DA/receipt vectors for inclusion 49/50/51 with origin10/window40, reader delay, changed-parent adoption after fallback, sequential substitution/revalidation, missing/mismatched accepted data before Set, and two same-block cuts with actual batches between them. Component A completion does not imply these pass.

## 11. Acceptance matrix and evidence

The independent verifier derives concrete values before reading candidate code. Builder commits the source candidate before review; Tester uses that same verified SHA, with a new SHA/review after changes. Distinguish pure/fixture vectors from real fresh driver receipts, and report seed `none (explicit script)` where applicable.

| ID | Stage-A required observation |
| --- | --- |
| DA01 | Bound genesis/state/config/dep accepted; wrong binding rejected; missing genesis-origin input waits |
| DA02 | Genesis first-output evidence false; header download does not change it; real output changes it; ordinary future adoption is not constrained by fallback's flag |
| DA03 | Full-window boundary 49 waits / 50 permits for origin10/W40, with complete actual receipt coverage; missing 11 waits |
| DA04 | Reachable slot4 first-output-retain and slot2 time-retain/then-advance histories match §6; new epoch needs its own window |
| DA05 | Header-only, incomplete receipts and reader-hidden blocks cannot advance complete frontier; later delivery yields the same D as continuous reading |
| DA06 | Actual CursorAdvanced stops Stage A at its exact position; later Set/fallback cannot overtake it; offered arbitrary bytes cannot satisfy the gate |
| DA07 | Failed/no-event/unrelated-emitter input does not become acceptance; initialized/Force input does not derive/reset D |
| DA08 | Deposit-only Apply uses actual metadata/parent and config-bound deposits; independent 03 native bytes/hash/state agree; no G input exists |
| DA09 | Missing/finite-exhausted fixture and overflow are explicit; no invented continuation or skipped deposit |
| DA10 | Real paired Set below target fills immediately using actual parent and including-block cap; supplied cursor origin is not selected blindly |
| DA11 | Set target beyond including timestamp is allowed; empty fill is not rejected by ordinary regional drift; e+1 above L requires no future header |
| DA12 | D ahead case cursor5/D10/target8 keeps head10 and cut-prefix10, anchor/state8; marker still resets |
| DA13 | Two actual RESUMEs in one block yield distinct exact-position cuts; later growth cannot mutate either |
| DA14 | Wrong/missing same-transaction pairing, log-index confusion, non-RESUME and not-yet-covered queries have distinct reject/wait outcomes |
| DA15 | Required input missing during marker causes no externally partial reset/fill/cut; retry equals uninterrupted result |
| DA16 | Checkpoint/restart after output and after each same-block marker reproduces future D and earlier cuts; forged epoch/frontier/prefix/config rejects |
| DA17 | Identical delivery/replay is idempotent; conflicting canonical data rejects; two reader delivery orders/pauses converge to byte-identical canonical outputs |
| DA18 | Candidate/commands/tool versions/raw receipts/traces/hash manifest kept; limits and Stage-B dependency are explicit |

DA02's ordinary-adoption clause and the full batch predicates may be independently specified now, but their executed batch portion is **Stage B INCONCLUSIVE** until feature 11 is consumed. DA06 is an interface-unavailability test in Stage A, not proof that unavailable DA later recovers. DA03 at high L1 heights and DA10–DA13 require a fresh 09 local driver scenario; hand-created events remain fixtures only. DA13's no-batch/fill version is Stage A; its intervening actual-batch version is Stage B.

Final reporting includes baseline/interface/candidate/test SHAs, actual role/model settings, worktree/branch, exact commands, configuration and seeds, absolute raw-evidence paths and SHA-256 summaries, per-ID PASS/FAIL/INCONCLUSIVE, and the open real-11/13/runtime integration gates. Stage A PASS is a component milestone and does not increase the commander's count of accepted complete features.
