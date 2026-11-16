# Independent derivation expectations — Stage A

Prepared 2026-11-16 by `bft_verifier` (configured GPT-6 Astra / xhigh), before task-12 implementation existed. Sources were read before the frozen contract; the contract was subsequently reviewed for the explicitly chosen P0 boundaries. Preparation baseline: `37b4f2b8a6d67b303ba90b1d4d1981c93ed8dd12`.

This directory is an independent verifier artifact, not a production codec, source adapter or proof of implementation acceptance. No task-12 source is imported by the reference program. Its semantic inputs are explicitly constructed fixtures, never fabricated verified L1 history. Required fresh Anvil evidence remains a separate gate.

## Source decisions and review

The local sources in `source-manifest.json` match the delivered MegaETH snapshot where applicable. The fixed OP source was independently obtained through Parallel at the URL recorded in that manifest, extract `extract_6b7cb7b2a87402540d875c006632b73b`. The extract was successfully returned; it is not a whole-file byte hash. Local source hashes are whole-file SHA-256.

The frozen `task-contract-derivation-v1.md` is implementable for Stage A. The following are explicit adapter choices, not claims that the upstream source mandates the same initialization or persistence representation:

- G0: genesis is a configured trust anchor, with no emitted-output evidence. `current_epoch` equals actual parent origin. Any emitted block establishes evidence, including an override block. There is no output-free epoch jump. A subsequent non-genesis checkpoint cannot reset that evidence.
- L0: finite 03 fixture provenance and the actual 09 driver source are distinct. Driver source binds public config, chain/contract identity, L1 genesis, timestamp policy and the authorized reader. No finite-fixture commitment authenticates extra driver blocks; no future signed history is hashed into its own configRef.
- CursorSet reset/fill/cut is atomically staged as a whole marker. Missing input or an error during its fill leaves exported pre-marker state and cuts unchanged. Earlier completed inputs remain committed.

The three origin rules must remain distinct. Ordinary fallback requires a complete window and uses first-output evidence. Same-height substitution uses actual parent and next-origin time, without that first-output branch. Override fill is additionally capped by its including L1 block and has no bootstrap target-time restriction. The independent rule vectors below deliberately exercise where these choices differ.

03's `actual_metadata` applies an unconditional drift bound and therefore cannot be the unconditional final gate for D's permitted empty paths. Stage A reuses 03 `Metadata`, actual execution, DEP and native encoding, with the derivation-specific origin guards. This does not authorize editing 03 or copying its production encoding into 12.

## Input and time boundaries

An L1 header, reader-visible complete receipts, and fully processed inputs are separate facts. Fallback is justified only after all relevant inputs through the inclusive boundary are resolved. A pending accepted commitment blocks later controls and the complete frontier, even when its outer cursor appears late. Stage A cannot resolve any accepted payload: the exact accepted position remains `WAIT(Feature11AcceptedPayloadRequired)`.

Process protocol events before the containing block's fallback opportunity. A cut at the first marker in a block cannot contain output caused by a later event or that block's final complete frontier. Transaction-local log indexes define canonical positions; RPC block-global indexes remain evidence only.

Only actual inclusion determines lateness. Wall-clock pauses, download time and header prefetch cannot change it. Independent A/B readers may differ in readiness and local provenance, but at an equal processed position must have equal native bytes/hashes, execution states, cut chain content, target anchors and Set/Resume positions. Reader name, delivery progress, local path and timing must not enter native hashing or the public identity of a cut. Checkpoints may retain local reader binding for authorization, separately from public rule/source identity.

## Executable semantic fixtures

`vectors.json` contains literal preconditions and hand-stated outcomes. `independent_reference.py` executes a bounded fixture model and independently applies the documented 03 big-endian encoding and length-framed SHA-256 domains. It does not invoke Rust, 12 code, a candidate-provided oracle, DA encoding, or an L1 adapter. `reference-results.json` retains complete state/native/anchor byte vectors for candidate comparison.

Numeric header maps and positions in this directory are semantic fixtures. In particular, fixture maps mentioning block 50 do not extend the accepted 03 fixture and do not count as real receipt evidence. The deployment runner must obtain high blocks through a fresh real 09 driver.

| Fixture family | Independent expected result |
| --- | --- |
| `window_49`, `header_only_50` | No output; the complete frontier is below the boundary. |
| `slot4_first_output`, `slot2_time_boundary` | First output retains origin 10; the next output advances to 11; a third waits for complete block 51. |
| `new_epoch_boundary` | Completion through 51 permits the next origin transition, then waits for 52. |
| `missing_next_origin` | No output despite complete window; required canonical next origin is missing. |
| `accepted_blocks_frontier` | Accepted position at block 50 remains pending; a later marker cannot overtake it. |
| `override_cap_and_drift` | Actual origin advances to 4 then 5, stays at 5 through height 30; no header 6 required when including block is 5. Target timestamp 40 exceeds including timestamp 10 and ordinary drift. |
| `two_same_block_cuts` | Targets 2 and 4 produce distinct cuts at `(12,0,1)` and `(12,1,1)`; their prefix heads remain 2 and 4. |
| `ahead_of_target` | Reachable prior override plus fallback yields D10/cursor5; target8 keeps head10, anchor8. End-of-block growth to 11 cannot change that cut. |
| `atomic_missing_dep`, `atomic_exhausted_dep` | A failure at fill height 3 publishes none of the marker's staged heights 1–2, reset or cut. |
| `real_plan_window50` | With L1 slot 2s, L2 slot 1s and genesis origin 0, a block-12 override fills to D20/origin10; complete block49 cannot extend it, block50 adds heights21/origin10 and22/origin11. |
| `real_plan_ahead_two_markers` | Block3 target5, then complete blocks42–44, reach D10/cursor5. At block45 target8 preserves prefix10/anchor8, target11 produces prefix11/anchor11; only block end then adds height12/origin6. |
| `actual_parent_transfer_filter` | Source nonce1 transfer is filtered on actual D nonce0; the next nonce0 transfer succeeds. Source G state/hash is absent. This execution reference is Stage B preparation only. |

The two real-run plans use an explicit synthetic base timestamp of 1000 seconds. Their planned Set/Resume indexes are typical direct-call receipt indexes, not evidence that Anvil emitted those positions; real tests must consume actual 09 positions. Reference `WAIT_WINDOW` and similar labels describe the stopped predicate, not a newly mandated Rust enum spelling. There are 14 executable Stage-A semantic cases and a small Stage-B predicate/execution preparation only.

## Candidate acceptance plan

The frozen contract's DA01–DA18 remain authoritative; the reference script alone does not mark any candidate row PASS.

1. **DA01–DA02:** independently vary config, genesis metadata/state/native hash, DEP digest and source binding. Missing canonical genesis input waits. Header prefetch cannot set output evidence. Restore a non-genesis checkpoint with false evidence: reject. Ordinary adoption's ability to advance directly from genesis is specified but execution remains Stage B.
2. **DA03–DA05:** use fresh real 09 blocks through the configured 40-block window. Before complete boundary coverage, no output. Remove a receipt, hide a delivered block from one reader, or omit next-origin data: wait without invented defaults. Deliver the missing material and compare the exact resulting native chain.
3. **DA06–DA07:** a real accepted batch blocks Stage A at its transaction-local position regardless of supplied arbitrary bytes; failed transactions, absent acceptance events and unrelated emitters do not. A Force or initialization event alone creates no D block/reset/cut. Whole-block receipt completeness includes these transactions.
4. **DA08–DA09:** compare actual metadata, DEP, pre-root, Kept, post-state/root, native parent/bytes/hash and derived anchor against independent vectors. Test pending DEP, finite exhaustion, credit overflow and checked height/time/window arithmetic. No partial invalid block is exposed.
5. **DA10–DA13:** fresh real Set/Resume pairs exercise immediate fill, including-block cap, over-drift empties, target below actual D, and two increasing targets in one block. Retain actual successful receipts, all transaction/log indexes and both exact pair pointers. Compare target state with the state at target, not the cut's later head.
6. **DA14–DA15:** mismatch the same-transaction pair, substitute a global log index, query a fully covered non-Resume or unknown future position, and remove a required fill input. Distinguish REJECT from WAIT. During marker WAIT, state/reset/cut equality must be byte-for-byte unchanged.
7. **DA16–DA17:** checkpoint immediately after each same-block marker and after a fallback. Resume using the same public history; compare with uninterrupted replay and all earlier cuts. Tamper epoch, frontier, input position, native parent, state, cut prefix/anchor, config and source identity individually. Missing replay material waits; contradictory state rejects. Repeat input is idempotent, conflicting canonical history rejects. A/B canonical output excludes local reader diagnostics.
8. **DA18:** record exact committed candidate, commands, explicit-script seed, tool versions, fresh raw receipts/traces and hashes. Verifier and Tester must name the same source candidate. After repairs, recheck affected behavior on the new revision.

Stage-B obligations remain INCONCLUSIVE: real feature-11 payload/commitment opening, ordinary batch adoption, 49/50/51 actual accepted inclusions, recovery after missing/mismatched DA, sequential substitution with a second retained record, actual batch between same-block markers, and real downstream 13/runtime integration. The lateness values in `vectors.json` are rule calculations, not accepted batch receipts or cryptographic validation.

## Reproduction

From this checkout:

```sh
python3 poc/verification/derivation/independent_reference.py --check
```

`--write` intentionally regenerates the reference result artifact; use it only after reviewing a vector/source change. This is a lightweight deterministic calculation, seed `none (explicit fixtures)`, and performs no heavy build, service launch, network access or runtime timing experiment.
