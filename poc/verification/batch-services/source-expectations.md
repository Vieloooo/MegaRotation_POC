# Batch services: source-first independent expectations v1

Date: 2026-11-16. Role: `bft_verifier` (configured Astra/xhigh). Integrated dependency baseline: `c9805f7a818bc23d30018ce4dc6f1ee45f92e3ac`. Sources were read before any feature-11 candidate or Builder explanation. This file fixes semantic expectations; exact frame bytes will be independently derived after the lead freezes the feature-11 codec. It is preparation, not implementation PASS.

Source pins and file SHA-256 values are in `source-manifest.json`. Regional BFT remains `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`, Commonware remains `2026.10.0`; no BFT certificate/quorum rule changes belong to this component. The conformance skill's evidence/delivery separation and write-before-output/restart obligations apply.

## Authority and explicit substitutions

| Authority | Independent requirement |
| --- | --- |
| v10 05 §§3.2–3.5, BAT-1–3 | Encode contiguous records starting at `prev.height+1`, using the same source position's executed `Kept`, excluding DEP and discarded ordinary transactions. Padding retains an empty record. Building or submitting never advances authoritative cursor. Task identity is `(prev,y)`; retry resumes durable steps. |
| v10 06 §§3.2–3.4, CERT-1–3 | Certifier owns its current L1/G view; checks full prev, latest RESUME no later than cursor `written_at`, DA signature/opening, decoded records, and complete source at a specific gpos before signing. Cached results do not bypass current prev/recovery guards. |
| v10 07 §§3.2–3.7 | Accepted input comes from verified finalized configured-Inbox successful receipts and full transaction input; `written_at` is historical metadata, outside the signed cursor. G′ materializes from the certificate's G source or exact ResumeCut, never just cursor height. |
| v10 08 §§3–4 | Immutable commitment-to-payload binding; deterministic public signature verification without store/clock; durable bytes before DA acknowledgment; missing payload is WAIT and never empty replacement. |
| v2 §§10.2–10.3 | Explicit P0: small nonce/balance execution, fixed `dep[h]`, crash-only services, one complete frame/closed channel with 1–2 heights. This does not establish native MegaETH wire compatibility or full GFR/G′/D. |
| 03 object contract v2 §§3, 6, 10 | Reuse `Transfer::encode_raw/decode_raw`, `Metadata`, execution and typed references. `ReplayPosition` has no frozen canonical codec in 03; complete codec/ordering belongs to 07. |
| 09 ABI contract v1 r1 §§2–5 | Ethereum Keccak-256, exact EIP-712 domain and type strings, complete cursor binding, low-s 65-byte secp256k1 signatures, zero frame digests, opaque gpos, 1–2 heights, true receipt/tx/log provenance. |

Current G input must be explicitly a bounded source fixture. Opaque `gpos` is signed exactly but is not proof of a GFR replay. A matching fixture marker, or `bool valid`, cannot be presented as independent consensus/GFR validation. Missing actual source produces WAIT; future 07/08 and 12/13 integration gates remain open.

## Frozen source-derived semantic cases

The machine-readable inputs/outcomes in `semantic-vectors.json` are hand-derived here, before feature-11 implementation. They are expected outcomes, not observed test results.

| IDs | Trigger / pre-state / guard | Required outcome |
| --- | --- | --- |
| B01–B04 | Cursor h=0; encode source heights [1], [1,2], [], [1,2,3], [2], [1,3], duplicate [1,1], or overflow from max u64 | One/two contiguous records succeed; empty/oversized/gapped/duplicate/wrapped ranges reject. No cursor change from build. |
| B05–B07 | Account 0 starts nonce=0/balance=10; credit 5; candidates transfer 0→1 `(nonce=0,amount=7)`, repeat nonce 0, insufficient `(nonce=1,amount=9)`, valid `(nonce=1,amount=8)` | Kept contains only first/fourth transfer, resulting account 0 nonce=2/balance=0 and account 1 balance=15. DA records contain their exact 03 ordinary transaction bytes; DEP and rejected candidates absent. Empty ordinary/padding still yields a record. |
| D01–D06 | Disperse/retrieve fixed y; retry/restart; corrupt/missing bytes; wrong key/domain/signature; crash around persistence | Commitment exactly Keccak(y); valid receipt covers commitment under configured Inbox domain. No receipt released before durable bytes. Retry preserves bytes/receipt identity; missing yields WAIT, mismatch yields explicit error without DAObject. No mutation of an existing acknowledged object. |
| C01–C08 | Full cursor mismatch, unavailable independent G/DA, gpos mismatch, record mismatch, last RESUME not replayed, cached prior result after cursor advance | Reject stale cursor or mismatched content; WAIT missing source/DA/recovery. Cached old result cannot authorize signing anew. On valid inputs derive next/range and sign every attestation field. |
| S01–S04 | Change chain, Inbox, config salt, prev/next fields, commitment, exact DA certificate bytes, range or gpos; malformed/high-s/wrong signer | Original signature must fail validation for altered input. DA and Certifier signatures cannot substitute for each other. Keccak must not be replaced by NIST SHA3. |
| L01–L05 | Submitted/pending transaction; included but not finalized; finalized but not reader-delivered; successful configured-Inbox event; failed/wrong-emitter/no-event receipt | Only actual delivered finalized successful `CursorAdvanced` with matching canonical calldata becomes an accepted record. A replay transaction with no new event creates no second accepted record. Keep original inclusion position. |
| R01–R06 | Crashes before/after DA persist, certifier persist, signed-output persistence, tx submission, finalized acceptance persistence; unknown tx result | Acknowledged bytes and signed/task state survive. Recover uses existing identity and bytes. Unknown send outcome must be reconciled before assuming failure. Restart/retry yields at most one new Inbox advance per logical task. Stale prev after Override cannot be accepted as a new batch. |
| V01–V04 | Reader has cursor but missing exact gpos source; available partial source; CursorSet lacking exact ResumeCut | Report authoritative cursor plus actual materialized prefix and missing sources. No copying latest G across RESUME, no inventing blocks through cursor height. D consumes accepted L1+DA without needing G′. |

## Independent byte and signature checks

09's already frozen signature inputs are normative:

- Domain name `SequencerRotationBatchInbox`, version `1`, chain ID, actual Inbox address, configRef as salt.
- `DAReceipt(bytes32 commitment)` and `BatchAttestation(bytes32 prevHash,bytes32 nextHash,bytes32 commitment,bytes32 daCertificateHash,uint64 firstHeight,uint64 lastHeight,bytes32 gposHash)`.
- Cursor struct hash covers height, origin, frame digest, source kind and Keccak(source). Batch hash covers the complete prev/next hashes, Keccak(exact 65-byte DA certificate), range and Keccak(gpos).
- `r[32] || s[32] || v[1]`, `v=27/28`, `0<r<n`, `0<s<=n/2`, nonzero recovered configured signer. Verify using an independent library/reference before asking the deployed contract.
- `next.sourceKind=GLOBAL`, `next.source==gpos`, both frame digests zero; first=`prev.height+1`, last=`next.height`, count 1 or 2.

Frame validation must reject trailing/truncated/noncanonical encodings, invalid complete-frame markers, malformed transaction lengths/opcodes, count/length arithmetic overflow, bounds violations and contradictory decoded record ranges. The codec is a new feature-11 engineering profile, not a second encoding of 03 types.

## Receipt and restart evidence requirements

Acceptance evidence must include actual deployment identity, raw transaction calldata, block/header/receipts, chain-specific finalization and per-reader delivery, exact transaction-local log position plus RPC block-global log index. Reconstruct complete BatchSubmission and compare event prev/next/commitment. Successful status alone or handcrafted events are fixtures, not real Anvil acceptance. Preserve status=0 negative receipts when transactions can be mined.

Persisted state must bind chain, Inbox, config and signer identity; opening a directory under another configuration must fail rather than reinterpret old signatures/jobs. Commit methods need file data plus directory durability at creation/rename boundaries. Injected process exit after writes or before output tests the real persistence boundary; dropping an in-memory object alone is weaker evidence and must be labeled.

No new protocol deadline may depend on `SystemTime`, wall-clock sleep, source scheduler fault flags, or unseen future driver state. Explicit scripted component inputs are permitted, with full 02 grant/ack integration retained as an open gate. Heavy verification uses the shared host flock and two build jobs, on a named committed candidate. Tester receives that exact verified source revision; any source fix invalidates affected review/test results.

## Current status

Source-derived semantic preparation: COMPLETE. Exact frame golden vectors and real offline signature vectors: COMPLETE against reviewed draft-r1; interface freeze review PASS is recorded in `contract-review.md`. Candidate review and real Anvil acceptance: NOT RUN. No feature-11 completion or full G′/D/native compatibility claim is made.
