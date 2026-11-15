# D derivation preparation — not an implementation contract

Date: 2026-11-15. Read-only source preparation by the spec agent and lead. Native recovery R1–R18 remains the active gate; no D implementation or final derivation contract is authorized by this note.

## Authority and source pins

Use v2 §§10.2–10.3 and v1 §§11.1–11.6 with the delivered MegaETH snapshot `a738f9b14c7880953d8cff8788db14e8e3e79124` from `handoff_provenance/manifest.json`. The inspected derivation, activation, Inbox, DA and certification-pipeline files match their manifest SHA-256 values. The root README's historical `59797b…` reference is different; do not substitute it for the delivered manifest pin. The v1 relative links to root `megaeth_specs` resolve correctly.

Relevant materialized sources:

* `megaeth_specs/specs/rollup/derivation.md`: Batcher Transaction Selection, Batch Parent Hash, Canonical L1 Origin Resolution, Same-Height Deposit-Only Substitution, Emergency Cursor Overrides and Unchanged Behavior.
* `megaeth_specs/specs/block-certification/derivation-pipeline.md`: Δ1, Δ3–Δ6, Δ8 and batch-validity versus execution rejection.
* `megaeth_specs/specs/block-certification/activation.md`: Activation Deposit-Only Fill.
* `megaeth_specs/specs/block-certification/batch-inbox.md`: Acceptance, Events and Emergency Cursor Override.
* `megaeth_specs/specs/rollup/data-availability.md`: Batch Commitment Resolution.
* v10 `06` §3.4, `07` §§3.2–3.7, `08` §§3.4–3.6/4, `09` §§5.1–5.5.

The MegaETH pages label the design incomplete, unstable, unactivated and spec:TBD. The PoC must identify its explicit substitutions and cannot claim deployed native-spec conformance.

The inherited OP source is pinned to [derivation.md at42b446f5891dd5a232cfc346201a613c1bba3f50](https://raw.githubusercontent.com/ethereum-optimism/specs/42b446f5891dd5a232cfc346201a613c1bba3f50/specs/protocol/derivation.md). The lead fetched its Batch Queue section through Parallel, extract `extract_7f4e7acc5a4c49058c357277a0033026`. A later focused refetch hit the service limit; no authentication/setup change was attempted.

## Minimal PoC boundaries

Inbox acceptance retains real submitter authorization, exact previous-cursor comparison, immutable DA commitment/receipt binding and an actual Certifier signature covering the PoC attestation. Only finalized successful configured-Inbox receipt events become D inputs. Submitted calldata, sender identity or a local cursor update cannot substitute for successful acceptance. Acceptance is separate from execution and D output.

The single complete frame/closed channel carrying1–2 contiguous heights is an explicit codec simplification. Records carry source Kept transactions, not DEP or a direct executable copy of G. DA acknowledges only after immutable bytes persist; retrieval verifies commitment. An accepted payload whose data is unavailable or mismatched causes WAIT at its input position, including before a later dependent CursorSet. It is not a missing batch. The main crash-only profile retains acknowledged data across restart.

D reads finalized L1, actual Inbox events, verified DA, configuration-bound `dep[h]`, canonical metadata and its derivation config. It does not consult G/Espresso/leader to choose output. Execute adopted records on the actual D parent. Attributes come from actual metadata; deposits remain the v2 height-indexed abstraction, with no claim to verify real origin-based deposit allocation.

## Timing boundaries and unresolved initialization

The pinned OP queue rejects lateness only when inclusion is strictly beyond origin plus the window. Fallback requires no admissible batch, complete processing through that boundary and an authenticated next epoch. It retains the current origin when the next L2 time is earlier than the next origin, or the current epoch still needs its first output; otherwise it selects the next epoch. Excess drift forbids ordinary transactions; an empty same-origin batch requires next-origin knowledge and cannot retain the origin once its timestamp permits advancing. These are predicate rules from the [pinned Batch Queue](https://raw.githubusercontent.com/ethereum-optimism/specs/42b446f5891dd5a232cfc346201a613c1bba3f50/specs/protocol/derivation.md).

Literal inclusion vectors: origin10/window40 accepts inclusion49 or50 under the lateness predicate alone, while51 is late. Retrieval delay never changes inclusion. Fallback with that epoch waits if processing reaches only49, if accepted DA at50 is unresolved, or if the next canonical header is absent. At current-origin time100 and next-origin time104, next L2 time102 retains the current origin; at104, output history determines the first-output versus advance branch. These vectors still require all earlier validation checks and a reachable history.

Keep queue current_epoch, actual derived parent/origin, complete processed-input frontier and epoch-output evidence separate. Loading a header does not prove that epoch produced output; do not default epoch_has_output or blindly carry it across epochs. Actual output establishes evidence, and recovery must reconstruct that evidence. Genesis/checkpoint initialization, ordinary-acceptance queue advancement, and exact continuation after an empty-path epoch advance remain explicit owner decisions requiring source-supported reachable-state vectors before freeze. Do not silently resolve these by an implementation shortcut.

## Actual parent and substitution

The certified MegaETH delta ignores source batch parent_hash and epoch_hash values, resolves the numbered origin canonically and builds on actual D. A changed parent after fallback therefore does not automatically invalidate later content.

Immediate same-height deposit-only substitution applies only after earlier checks pass and timestamp is the next expected timestamp, for the enumerated origin-below-parent, origin-above-parent+1, timestamp-before-origin and drift violations, retaining the stated empty-batch exceptions. Use actual parent and mapped next timestamp to select parent origin or its next canonical origin; missing required headers cause WAIT. Consume that substituted record, then recheck the remaining channel record against the replacement parent.

Late-window rejection, old/future timestamps and malformed inputs do not all become this substitution. Execution-engine INVALID is a separate native branch; P0 does not implement general EVM/engine invalidity. Well-formed invalid-nonce/balance ordinary transactions are filtered by light Apply, not classified as whole-block INVALID. Any revert opcode needs its own explicit nonce/fee/state/Kept rules.

## CursorSet and immutable ResumeCut

Process successful CursorSet by exact canonical block/transaction/log position and bind it to the same-transaction RESUME. Resolve preceding accepted data/channels, then reset unfinished pending state. Missing required earlier DA continues to block progress.

If D is below targetN, fill immediately throughN without waiting for an ordinary sequencing window. Each fill uses actual parent origin e and next timestamp t; advance to e+1 only if representable, at or below including L1 blockL, and its canonical timestamp≤t. The supplied cursor origin is not the fill rule. If D already reaches/exceedsN, reset without rewinding it. Override can target a timestamp beyond including L1 time; do not copy bootstrap's timestamp restriction onto Override.

Save a separate historical ResumeCut at each exact RESUME position. Its derived_prefix is D at that CursorSet and may extend beyondN; GFR adopts only throughN. A later D head or end-of-containing-L1-block state cannot answer the earlier cut. Two RESUMEs in one L1 block require distinct historical cuts.

Compare G/D at common heights using the PoC native hash of actual parent, metadata, actual DEP, actual Kept and post-state root. Physical hash, height, cursor or pre-state root alone is insufficient. Derived anchors use the v10 identity built from height and native hash. D lag or greater height alone is not common-height conflict.

## Future acceptance and decisions

Cover successful/failed real receipt controls, inclusion equality/+1 with different download delays, accepted missing DA, complete-window fallback, valid changed-parent content, precise immediate-substitution cases followed by a second channel record, missing canonical headers, D-ahead CursorSet (cursor5/D10/target8 leaves D10), same-block positional RESUMEs, crash/retry and stale-prev rejection across Override.

Before implementation, freeze certified-from-PoC-genesis initialization versus native activation; exact PoC ABI/attestation/DA/cursor encoding; the unresolved epoch-state transitions; light execution/native hash vectors; and immutable accepted-data/historical-cut storage. Upstream's six-field Cursor, EigenDA/TEE checks and ABI are not automatically the v10 logical PoC interfaces.
