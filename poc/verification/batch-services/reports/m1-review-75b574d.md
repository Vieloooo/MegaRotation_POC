# M1 independent review — 75b574d

Date: 2026-11-16. Candidate: `75b574d40930b7c9c748a4c6b6e381117c937537`. Frozen source expectations and contract: `8fb46c296808dbe97c95df7a2761678412b95f62`; integrated dependency baseline: `c9805f7a818bc23d30018ce4dc6f1ee45f92e3ac` (03 `e9c15e0`, 09 `c443ae9`). Worktree `/home/jy/.codex/worktrees/f025/megaroation_poc`, branch `codex/poc-batch-services`. Reviewer: named `bft_verifier`, configured Astra/xhigh. Main model/effort was separately verified by lead as Astra/ultra.

**PASS for M1 frame/crypto interface release. No blocking source findings.** The accepted-reader interface passed static boundary review; its actual Anvil execution and receipt/DA integration are **INCONCLUSIVE / not yet run**, assigned to M2. Durable actors, real process-crash matrix and full feature11 are not included in this PASS.

## Source review

The reviewer derived semantic expectations and golden byte/signature vectors before Builder implementation. The candidate was committed and Builder stopped writing before this review. `git diff 75b574d... -- poc/crates/batch-services` remained empty during verification; later dirty paths are only independent verifier consumers/results.

`frame.rs` implements the single BS-FRAME-v1 layout, uses 03 `validate_transactions` for raw Transfer validation, bounds allocations/counts before iteration, rejects trailing and partial data, uses checked height continuity, verifies channel ID and full Keccak payload commitment separately, and preserves empty ordinary-transaction records. It introduces no second 03 transaction codec and no source-dependent origin validation.

`crypto.rs` matches frozen 09 type strings/domain/ABI and separates authentication from P0 structural validity. Actual secp256k1 recovery rejects malformed lengths, invalid v, zero/out-of-range scalars and high-s signatures. Full prev/next/source bytes, DA certificate, range and gpos are bound. ABI calldata matches independent encoding. Public `SigningKey` is a cryptographic primitive; persistent authorization/output gates remain M2 actor responsibilities.

`accepted_view.rs` owns a concrete `Driver<AnvilRpc>`. The only AcceptedRecord constructor is private and reached through a real driver Read command. AcceptedRecord and VerifiedDaObject have private fields and no Deserialize constructor; serialized evidence cannot be re-imported directly as trusted input. The gateway checks named-reader visibility/finality, exact block provenance, configured caller/target/emitter, successful receipt, transaction/hash/index links, original local log/RPC log index, canonical calldata and signatures. It relies on fixed 09 to validate complete receipt history and event/calldata pairing. Arbitrary public EventRecord/JSON cannot be passed directly to its private promotion method. Loopback endpoint restriction and fresh-genesis ownership come from 09's configured local RPC boundary; no Ethereum consensus/MPT proof claim is made.

`load_payload(None)` and commitment mismatch return WAIT at the accepted position. A matching commitment with malformed frame rejects; a valid frame must match signed first/last height and terminal origin. These conditions were source-reviewed; actual AcceptedRecord construction and loading via a running Anvil still require M2 evidence. Repeated reads intentionally return the same stable acceptance identity; they do not themselves maintain a durable consumer dedup table.

## Independent execution

The separate consumer lives in `poc/verification/batch-services/consumer-m1/`; it imports the candidate as a library and uses independently authored input vectors. Production code was not modified. Its own locked dependency graph is retained. Rust/Cargo are 1.95.0; all heavy work used the shared host flock and `CARGO_BUILD_JOBS=2`.

| Check | Result |
| --- | --- |
| Six frozen valid frames plus 64 independently generated cases | PASS, exact encode/decode/channel/commitment comparison |
| 19 frozen malformed frame cases | PASS, decode and verify-payload reject |
| Every truncated prefix of the 70 valid cases | PASS, 9,011 rejected prefixes; trailing-byte rejection also checked |
| Maximum two records × 1024 transfers | PASS, exact 47,175-byte payload and independent commitment |
| Oversize payload/count and max-u64 height wrap | PASS, explicit rejection |
| Two independent real offline signatures | PASS, production verification |
| 22 domain/field mutations | PASS, independently frozen digests/recovered addresses and production field mutation checks; extra DA-certificate byte rejects its required 65-byte shape |
| Malformed signature suite plus zero/out-of-range r and extra v boundaries | PASS, 13 checks; wrong expected signer also rejects |
| Production-generated DA and Batch signatures | PASS, independently recovered by coincurve/libsecp256k1 using independent digests |
| Production `submitBatch` calldata | PASS, all 1,060 bytes match independent ABI reference |

Random-frame seed: `1112753713` (`0x42534631`). These are format-only vectors: arbitrary origins/raw transfers do not claim valid G or application execution. Offline production-signing keys were locally generated in a private `/tmp` file, excluded from repository/evidence. Public signatures are reproducibly verified without the key. Other cases use literal inputs, no scheduling seed.

Final run command, from the worktree root:

```sh
export PATH=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:$PATH
export CARGO_HOME="$PWD/.build/cargo"
export CARGO_TARGET_DIR="$PWD/.build/target/batch-services"
export CARGO_BUILD_JOBS=2
flock /tmp/megaroation-poc-heavy.lock cargo run --locked --offline \
  --manifest-path poc/verification/batch-services/consumer-m1/Cargo.toml -- \
  "$PWD/poc/verification/batch-services" /tmp/batch-services-m1-verifier-signing.key
/home/jy/miniconda3/bin/python3 poc/verification/batch-services/check_m1_output.py \
  .build/artifacts/m1-verifier-75b574d-run2/consumer-output.json
```

Actual stdout/stderr/revision/toolchain/worktree state were saved under `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11/artifacts/m1-verifier-75b574d-run2/`. Run1 is preserved, with the earlier consumer before adding direct per-field mutation comparisons. Final raw consumer output SHA-256: `c8b63a6427ec5fe0e8fb1660712426582e75f67bc7889a22ee28fc5d27ad9217`. See `m1-manifest-75b574d.json` for all evidence/source hashes and `m1-independent-results-75b574d.json` for the compact result.

## Consumable API and limits

Feature12 can depend on the standalone `batch-services` crate at this candidate and call `frame::{BlockRecord,ClosedFrame,encode_closed,decode_closed,commitment,verify_payload}`. `crypto::{InboxDomain,InboxSigners,check_da,verify_batch,validate_submission,submit_batch_calldata}` retains the 09 wire types and domain. `InboxGateway::{connect,apply,read_accepted_block}` and the read-only AcceptedRecord/AcceptedPayload accessors define the concrete reader boundary. A running system must assign one ordered driver command stream and independent reader identities; another gateway cannot attach a fresh 09 driver to an already advanced Anvil.

This release is useful for frame/commitment consumers now. M2 still owes actual sample DA/receipts, runtime gateway tests, independent Apply/Kept source checks, immutable DA persistence, Certifier/Batcher recovery, unknown-submit reconciliation, real service-process crash tests and the two-wall-pause component matrix. Explicit service delay tests will not substitute for crash tests. Persistent BFT workers may run regular/Force/RESUME independently of the unfinished 01 recovery gate; no BFT crash/restart is inferred from service-crash coverage.

Full public G/gpos, G′, D/ResumeCut and 02 grant/ack integration remain separate gates. M1 does not increase the count of complete accepted features. Tester should run any M1 interface checks on this exact source SHA; production changes require affected re-review.
