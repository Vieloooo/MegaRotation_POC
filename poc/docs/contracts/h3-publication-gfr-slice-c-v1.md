# H3 Slice C — official runner artifact input

2026-11-16. ID **H3-C-1**. **FROZEN by lead after independent pre-code review PASS.** [Dispatch/ownership](h3-publication-gfr-implementation-v1.md) formally transfers only `poc/scenarios/espresso-compatibility/{capture.py,run-local.sh,README.md}` and `poc/verification/espresso-compatibility/check_capture.py` for this minimal extension. The single Builder proceeds after the named A verification/test handoff.16 crate, launcher/isolation design, pins and old evidence remain unchanged. No H1 API is needed to implement/independently test input handling; genuine regional verification remains a separate actual-input gate.

## Source and interface

S/A:16 frozen contract §§2–4 and fixed official release20260918/commit `fd221e8e4b51f5ecec0ae44bc99f6b7f82cd1171` define namespace160016, HTTP `/v1`, complete native flat-index reads, trusted local API and Unknown timing. New dispatch decision10 requires the same M1 bytes through this runner, not marker substitution. Existing marker defaults and offline audit behavior remain compatible.

Add optional `--payload-manifest PATH` to `capture.py` and `check_capture.py`. Extend shell invocation to `run-local.sh RUN_ID EXPECTED_CANDIDATE_SHA [--payload-manifest PATH]`. No argument preserves the exact two-marker+first-marker-repeat mode. The optional path is resolved/read before node launch; shell forwards it as a separately quoted argv value to existing capture, never via eval or shell interpolation of file contents. Reuse existing Bubblewrap, fixed binary verification, private Anvil wrapper, task PID cleanup and host lock; no second process launcher.

**H3-PUBLICATION-ARTIFACT-1** is a bounded audit/input manifest, not another public encoding:

```json
{
  "schema": "H3-PUBLICATION-ARTIFACT-1",
  "evidence_kind": "synthetic-adapter",
  "producer_candidate_sha": "<40 lowercase hex>",
  "run_id": "<1..64 existing-safe-id characters>",
  "result_key_hex": null,
  "native_evidence": null,
  "messages": [
    {"ordinal": 0, "codec": 1195790849,
     "path": "payload-000.bin", "length": 123, "sha256": "<64 lowercase hex>"}
  ]
}
```

Codec decimal1195790849 is existing `0x47465201`; use the constant's checked numeric equality. `evidence_kind` is `synthetic-adapter` or `native-regional`. Native mode additionally requires nonnull `result_key_hex` containing the exact03 canonical66-byte ResultKey (132 lowercase hex characters), and `native_evidence={"h1_revision":<40hex>,"report_path":<relative path>,"report_sha256":<64hex>}`. This is provenance, **not native verification by Python**. The actual H1 proof/body/anchor check must be rerun by the reviewed H3/H1 verifier against the exported bytes before claiming real-regional publication PASS; retaining a report or selecting this tag alone cannot close that gate. Synthetic adapter tests do not require H1 and remain clearly labeled.

Strict JSON rejects duplicate keys, unknown fields, noninteger lengths/ordinals/codecs (including bool), and nonfinite numbers. Manifest≤65,536 bytes;1..11 messages, contiguous ordinals from0, each payload1..65,536 bytes, aggregate payload≤65,536 bytes; native report≤65,536 bytes. These limits admit A's corrected≤51,560-byte encoded full attempt (≤51,169 bytes of actual message payloads) and do not relax06 bounds. Read each file with bounded reads, verify exact length/SHA before launch, and retain immutable in-memory bytes for submission. Relative paths must resolve to regular non-symlink files under the manifest directory; reject absolute paths, `..`, escape, missing files and conflicts. Do not discover or execute commands from the manifest.

## Capture and independent readback

1. Save exact manifest bytes, its SHA and a bounded copy of every input payload/native report into capture bundle artifacts. Record a deterministic ordinal→copied-path/length/hash mapping and classification in configuration. Existing raw request and response capture remains unchanged; retain failure evidence. Producer SHA and current runner SHA are separate fields and need not be equal.
2. Submit every list entry in order using unchanged HTTP namespace160016 and its exact payload. Preserve duplicate entries/bytes. Repeat the first payload as the existing idempotence observation. Distinct payloads must not be required when using a manifest; matching entries may share an ACK/hash and one actual occurrence. Record every submit record ID and its originating ordinal; do not silently dedupe the manifest.
3. Await complete actual public prefix using existing bounded wall deadlines. Match each input ordinal by exact bytes, namespace and returned ACK hash. ACK alone is Submitted. Save all native `(q,source_transaction_index)` occurrences and namespace-local/projected indices; duplicate inputs may reference the same observed occurrence without claiming separate inclusion. Raw native positions and namespace filtering rules remain unchanged.
4. Reuse current complete block/leaf/all-index capture, second-reader requery, actual404 check and offline replay. The unchanged16 CLI always emits its EC16-AUDIT-1 projection with `FIXTURE_CODEC=0xEC160001`. Preserve that tag and old audit bytes/convention even in manifest mode; it labels an opaque audit projection, not the HTTP payload's protocol. Independently validate manifest/input-identity metadata against actual `GFR_PUBLIC=0x47465201` and compare original GFR payload bytes through POST/readback. HTTP transactions have namespace/payload, not a codec field to rewrite. Do not edit16 crate or substitute the GFR codec into its legacy audit; a separate input-identity section binds each manifest GFR codec/payload to native positions. No official16 reader/PE export is passed into GFR.
5. Extend independent `check_capture.py` with the same optional manifest flag and independent validation (do not import candidate capture validation). Reconstruct source transactions with its existing raw payload/namespace-table oracle; require identity from input file→recorded manifest copy→decoded POST payload→decoded native positional transaction. Preserve old no-flag vector/audit checks exactly. Cross-check source manifest/report copies by hash; report native cryptographic verification as separately required, never performed by this Python hash check.

`result.json` and independent report distinguish `marker-default`, `synthetic-adapter` and `native-regional` input mode; include input manifest SHA, producer/runner revisions, per-ordinal submit/occurrence mapping and byte-equality results. Native mode without actual H1 revalidation remains **byte-publication-only**, not genuine regional or M1 PASS. Official wall duration, proposed/finalized/live evidence Unknown, unsupported PE exports, trusted API and unverified QC/VID remain explicit. Never feed the official capture into deterministic GFR timing proof.

## Independent cases and gates

| ID | Input | Expected |
| --- | --- | --- |
| C01 | No new argument, old fixtures/vectors | Existing marker defaults, audit and launcher behavior unchanged. |
| C02 | One and11 synthetic entries; duplicate payload; different producer/runner SHA | Exact ordered bytes submitted/mapped; duplicates retained; producer SHA not overwritten. Synthetic classification visible. |
| C03 | Missing/escaping/symlink payload, changed length/hash, duplicate JSON key/unknown field, bool length, limits+1 | Fail before node launch; no shell execution or partial submit. |
| C04 | Actual ACK only, wrong namespace/bytes, missing earlier q/index | No complete receipt; bounded WAIT/incomplete evidence preserved, no fabricated positions. |
| C05 | Original input→copied artifact→POST→raw source transaction with independent full-prefix oracle | Every selected ordinal matches exact bytes/hash; all original occurrences retained. |
| C06 | Native tag without actual reviewed H1 proof check; official timing fields unavailable | No native/M1 claim from tag/report/hash; Unknown/Unsupported and QC/VID limits unchanged. |
| C07 | Later genuine M1 artifact, same reviewed H1 revalidation, actual official capture/requery | Separate genuine-result byte-publication PASS with native verifier evidence and finite positions; still no official deterministic timing claim. |

C01–C06 input handling may be tested offline/synthetically without launching a node or waiting for H1. C07 waits only for genuine artifact/native checker and an authorized bounded official run. Builder/Verifier document exact commands and fixture classification; current annex is not execution evidence.
