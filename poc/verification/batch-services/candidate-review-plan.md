# Candidate review and evidence plan

Frozen contract commit: `8fb46c296808dbe97c95df7a2761678412b95f62`. Prepared before reading feature-11 candidate source. This plan extends the already committed source expectations without changing them.

## Interface slice M1

Candidate source must be committed before review. Record exact HEAD and clean/dirty status, independent expectations SHA and tool versions. Review only assigned feature-11 source and read-only dependencies. Run one heavy command under `/tmp/megaroation-poc-heavy.lock`, two build jobs, `--locked`; inspect the actual dependency resolution rather than presuming an offline build is possible.

1. Check public exports and negative construction: a caller-supplied EventRecord/JSON/driver-like response must not create trusted AcceptedRecord without the configured real driver gateway and reader delivery. Public raw bytes may be decoded as candidates, not automatically promoted to trusted acceptance.
2. Run exact independent `frame-golden-vectors.json` payloads through the production encoder/decoder and compare bytes, lengths, channel IDs, commitments, decoded records and expected failures. Exercise the maximum 47175-byte recipe and oversized/overflow bounds. The independent frame decoder imports no feature-11 code.
3. Verify the two real offline signatures and every negative domain/field/format vector with production crypto. Independently recover signatures produced by production crypto; do not rely on production verify(sign()) round trips alone. Include wrong expected signer, DA↔Batch type confusion, high-s aliases, zero r/s and v=0/1/29.
4. Inspect the exact domain, type strings, dynamic-field hashes, cursor/source encoding, range checks and fixed complete-frame profile against frozen 09. Check origin/range mismatch independently from cryptographic validity.
5. Candidate API test hooks should accept explicit inputs and produce public JSON/bytes. Verifier may build a separate consumer in this verification path; production must not depend on these Python helpers. A temporary test inside the crate source is not needed.

## Complete component M2

The main evidence bundle uses 09's existing `raw-rpc-bundle.json` shape (`config`, full `blocks`, `receipts`, optional historical reads). Preserve `driver-trace.json` with command, reply and RPC transcript. For each actual accepted commitment, keep immutable DA payload bytes and a small index mapping the commitment hex to its relative payload path. `check_accepted_evidence.py` independently decodes/replays actual receipts, rechecks signatures, opens every accepted payload and checks frame range/origin. It does not trust the production accepted-record export as its expected result.

The final evidence set also needs:

| Boundary | Evidence and required comparison |
| --- | --- |
| G fixture | Immutable config/anchor/state/metadata/DEP/raw ordinary inputs, explicit `source_mode=fixture` and opaque gpos mapping. Independently derive the §9 Kept/post-state values. Certifier's own source must fail on deliberate missing/conflicting inputs. |
| DA durability | Actual process identity and exit status, exact injected boundary, file hashes before/after restart, returned receipt bytes and retrieved payload. No acknowledged object may disappear or change. Directory/config mismatch must reject. |
| Certifier persistence | Original request, current finalized cursor and latest relevant RESUME, source readiness, persisted signature, reply loss and replay. Cached output must still reject stale prev or WAIT for missing recovery progress. |
| Batcher task | Stable job/request identity, all phase transitions, durable raw tx/hash/nonce before send, receipt-unknown state, restart reconciliation and one original accepted position. Duplicate command versus same-id conflict tested separately. |
| Actual Inbox | At least two new accepted batches with real signatures and nonempty state-changing Kept; actual finalized/delivered provenance and raw EVM receipts. Wrong signer/commitment/stale prev after Override must include mined status=0 where possible. |
| Reader gateway | Separate included/finalized/visible frontiers, Read before/after Finalize/Deliver, wrong reader/address, historical event after later cursor progress, and duplicate receipt replay. No-Advanced successful retry is not a new accepted record. |
| Missing DA/source | AcceptedRecord stays known while AcceptedPayload waits at its exact input. Missing/corrupt bytes do not become empty fallback. G′ reports actual materialization, not cursor height. CursorSet awaits the exact ResumeCut. |
| Timing and independence | Same explicit script with distinct wall pauses produces identical protocol trace; process/role fault boundaries are independent. No public rule reads wall time or scheduler future/fault flags. |

Each crash result distinguishes process termination from object reopen, and data-write durability from receipt-return durability. OS crash tests cannot prove arbitrary filesystem/power-loss behavior; report the exercised process and fsync boundaries precisely. The accepted raw bytes/receipt sample package must be directly usable by feature12 through the single production codec.

## Defect priority

P1 before candidate PASS: trusted acceptance from fabricated/nonfinal/undelivered events; source/RESUME/cache bypass; receipt before durable DA; acknowledged state rollback; mutable DA; signed-field omissions; malformed/overflow decoder acceptance; blind new tx identity after unknown outcome; duplicate accepted records; accepted missing data interpreted as absent batch.

P2 before component completion: incomplete promised fault matrix, missing raw evidence or independent comparison, missing diagnostic distinction, unbound persistence configuration, unsupported advertised API, or incorrect report scope. Any unavailable required case is INCONCLUSIVE, not PASS.

No test count proves full feature11 or full protocol safety. Keep 02/07/08/12/13/full-system gates explicit. The simulator receives the same reviewed source SHA; source fixes require affected independent review and tests again.
