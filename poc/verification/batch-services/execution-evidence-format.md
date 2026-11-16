# Independent execution evidence: BS-EXECUTION-EVIDENCE-v1

Prepared before feature11 M2 candidate review. Authority: fixed 03 `e9c15e04bc83b335668f3ecd837367aa87344a89`, its contract v2 §5.1 and canonical hash tables, and frozen batch-services r1 §9. The checker imports no production Apply, frame, signature or receipt code. It separately consumes the independent accepted evidence result, so it does not implement receipt/DA acceptance a second time.

Use **contract:B01–B18** when reporting feature11 acceptance. The older `source-expectations.md` cases are separately named **source:B01**, **source:B05**, etc.; they are semantic preparation IDs and must not be counted as contract IDs. For example, source:B05 is transfer filtering; contract:B05 is DA crash persistence.

## Public source artifact

Export `execution-source.json` as:

```json
{
  "schema": "BS-EXECUTION-EVIDENCE-v1",
  "source_mode": "fixture",
  "fixture_id": "<immutable public fixture identity>",
  "config_ref": "0x<32 bytes, equal to actual Inbox domain configRef>",
  "byte_limit": 65536,
  "snapshots": [{
    "gpos": "0x<exact opaque bytes used in the attestation>",
    "initial": {
      "metadata": {"height": 0, "timestamp_us": 0, "origin": 0},
      "accounts": [[1, 0, 10]],
      "native_parent": "0x<32 bytes>"
    },
    "blocks": [{
      "metadata": {"height": 1, "timestamp_us": 1000000, "origin": 0},
      "dep": ["0x<03 Attr raw>", "0x<optional 02 Credit raw>"],
      "raw_transactions": ["0x<all original 01 Transfer bytes, including discarded ones>"],
      "pre_state_root": "0x<32 bytes from source block>",
      "observed": {
        "kept": ["0x<production observed Kept bytes>"],
        "post_accounts": [[1, 1, 7], [2, 0, 3]],
        "post_state_root": "0x<production observed state root>",
        "native_hash": "0x<production observed native hash>"
      }
    }]
  }]
}
```

The example contains placeholders, not executable evidence. Account rows are `[id:u8, nonce:u64, balance:u64]`, sorted by id, unique, excluding explicit nonce=balance=0 entries. The initial native hash is an explicit fixture anchor, not derived from G. Each snapshot has one unique nonempty gpos (at most 1024 bytes), a contiguous list of source virtual heights, actual metadata, complete DEP and all raw ordinary inputs. No guessed Kept-only input is sufficient to prove filtering.

`observed` is optional. When present, all four output fields are compared to the independent result. When omitted, the checker still derives canonical state/native hashes and compares actual accepted DA records to Kept, but reports zero corresponding observed poststates compared; this cannot be described as validation of production poststate output. Include it for the intended M2 acceptance evidence.

The checker validates 03's transaction/DEP formats and bounds before execution; credits precede ordinary transfers. Invalid nonce, insufficient balance, sender nonce overflow and recipient overflow are filtered without state changes. Self-transfers retain balance and increment nonce, with no fake recipient-overflow rejection. Zero transfers may create only a sender nonce entry. Credit overflow or malformed input rejects the entire step and returns no state. State and native hashes use the frozen 03 SHA-256 framing, not DA Keccak.

Fixture ID/gpos bind the exported source used for the comparison. ConfigRef equality binds the source label to the actual Inbox evidence. This is not independent validation of a complete PublicConfig hash, dep-fixture content address, genesis authenticity or GFR consensus; those limitations are explicit in output.

## Invocation and output

First run the existing independent accepted evidence checker on actual receipt/DA artifacts, then:

```sh
/home/jy/miniconda3/bin/python3 poc/verification/batch-services/check_execution_evidence.py \
  --source /absolute/run/execution-source.json \
  --accepted-result /absolute/run/independent-accepted-result.json
```

The accepted checker now includes the actual chain/Inbox/configRef in `configuration`; this is an evidence-format addition, with no production/API change. The execution checker requires a complete nonempty PASS payload set. Every accepted record is compared by its exact attested gpos and height; origin and transactions must equal independently executed source results. Missing snapshot/height yields INCONCLUSIVE with the exact accepted position; mismatched data raises a validation error. Output records source/result SHA-256, Kept indices, filter reasons, poststate/native hashes, number of observed states compared and accepted heights covered.

The source-only self-test uses no production candidate or Anvil:

```sh
/home/jy/miniconda3/bin/python3 poc/verification/batch-services/check_execution_evidence.py --self-test
```

It checks 25 cases including r1 §9's h1, h2 and padding state results, Credit+filtering, max-balance self-transfer, overflow filters, zero transfer/account normalization, malformed DEP/ordinary inputs, bounds, rejected-input immutability, source/accepted joins and tampered evidence. Join tests use in-memory synthetic positions and are explicitly reference fixtures, never actual receipt evidence. Results are in `execution-reference-results.json`; this is reference preparation, not contract:B01/B06 implementation PASS.

## Durable/crash review checklist and minimal counterexamples

| Contract case | Independent evidence | Minimal failing example |
| --- | --- | --- |
| B04/B05 | DA payload+receipt returned before crash, process exit boundary, same root after restart, original bytes/receipt and file hashes. File data, object publication and parent-directory durability order source-reviewed. | Emit receipt, then kill before payload/directory sync; restart cannot retrieve its committed bytes. |
| B05 | Inject bytes-written-before-receipt, receipt-persisted-before-reply and lost reply. Recovered object retains configuration and identity. | Store exists after bytes-only crash but service treats missing receipt as permanent corruption instead of completing unacknowledged work. |
| B07 | Current full cursor and historical Set/Resume pairing, source processed set, cache hit attempt. Select CursorSet position `<= cursor.written_at`, then require its paired Resume. | Current Override written at `(b,t,0)`, paired Resume at `(b,t,1)`; comparing Resume directly to written_at misses the current required RESUME. Another case: later GLOBAL cursor still requires the preceding RESUME. |
| B07/B08 | Saved signature bytes/request/source/domain; independent current reader after restart; no new response from stale cache. | Sign at prev h=2, advance/Override cursor, restart and return cached certificate without checking current prev. |
| B08/B09 | Role manifests bind chain/Inbox/config/signer/codec/source identity, separate roots and independent exit/restart evidence. | Reopen same directory with a new key or same signer but different Inbox and silently reuse jobs/signatures. |
| B09 | Stable `(prev,y)` job, complete persisted content/phase/outbox, command ID and content, terminal preservation. | Same command ID with different payload accepted; or restart writes a new BUILT snapshot over terminal ACCEPTED. |
| B10 | Raw signed tx, hash, nonce and request ID persisted before actual send; exact crash location; live reconciliation trace. | Node accepts tx, service loses reply and restarts, increments nonce/request ID and creates a second logical submission instead of reconciling original tx. |
| B11/B12 | Two original Advanced positions and distinct successful receipts; same-request retry receipt no Advanced; stable identity across repeated reads. | Count status=1 no-event retry as a second accepted batch, or mark included-but-unfinalized data Accepted. |
| B13 | Signature obtained before Override; actual new stale-prev submission produces mined status=0; original successful request retry kept distinct. | Test only `eth_call`, or use the previously successful request ID whose idempotent success is correct but proves no stale-new-request rejection. |
| B14/B15 | Original acceptance position remains when bytes unavailable/corrupt; retrieval recovery same bytes; materialization missing source/cut explicit. | Skip unresolved accepted input and synthesize empty/fallback records or report cursor height as full G′. |
| B16 | Distinct wall pauses with identical explicit input script; separate per-role process identity and fault switch; bounded trace comparison. | Relabel sleep/delayed reply as process crash, or let restarting DA implicitly reset Certifier/Batcher/Anvil. |

Actual service-process termination/reopen is distinct from object drop, and delay is distinct from crash. The persistent BFT worker track does not need 01's native recovery completion; service crash evidence does not establish BFT crash/restart. The reviewer will label filesystem/power-loss limits and exact exercised process boundaries. No new heavy build or Anvil run is part of this preparation.
