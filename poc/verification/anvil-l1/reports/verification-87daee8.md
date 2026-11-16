# Feature09 independent candidate verification

Date: 2026-11-16 (Asia/Shanghai). Role: named `bft_verifier`, GPT-6 Astra / xhigh. Candidate **`87daee8e9e77bbe27c337383d873a5eaf4a694ba`**, clean worktree before and after verification. Initial frozen interface `daad412e46eeecd64ef9b8af8175e64ec40edc53`, contract `ANVIL-L1-ABI-v1 r1`. Worktree `/home/jy/.codex/worktrees/65ef/megaroation_poc`, branch `codex/poc-anvil-l1`.

**Verdict: PASS for the bounded independent component review and exercised A01–A17 matrix. No blocking implementation finding. Eligible for Tester to run the same candidate. Complete feature09 is INCONCLUSIVE: A18/A19 remain integration gates.**

## Independence and reviewed behavior

Source expectations, literal vectors, crypto/ABI helpers and receipt-checker design were authored before candidate implementation was read. They are committed in `poc/verification/anvil-l1/`. Source authority is v10 00 §1.2/4.2, 01 §3.1/3.9–3.11, 06 §3.2/3.4/CERT-2/3, 07 §3.2–3.6, 08 §3–4 and 09 §5, plus v2 §§3.2/4.1/5.1/7/10.3. Pinned BFT `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6` and Commonware2026.10.0 are unchanged and not exercised here.

Read all four production Solidity files, ABI/interface JSON, Rust lib/history/ABI/RPC/runner and scenario/fault-test source on C. Checked these paths against the frozen expected transitions:

- `TypedData.sol:16/27`: canonical write calldata; exact EIP-712 domain; low-s real ecrecover guards. `ControlAuthority.sol:86`: shared hasControl/sequence and req table; signature before retry, retry before current sequence/target; fixed static policy; source-bound Resume internal call. No arbitrary forwarder in production.
- `BatchInbox.sol:79/117`: immutable fixed permissions/signers, complete prev and source binding, DA and Certifier signatures, 1–2 closed heights, origin bounds, original retry output, and exclusive Authority Override. `RollbackFixture.sol` is a separate test deployment, not production Authority.
- `lib.rs:242/336/371/399`: actual production Driver owns raw submission and manual timestamp/mining; exact cached commands have no repeated side effects; unresolved mutation blocks new Submit/Mine; reconciliation reads canonical head/receipts. `main.rs:26` drops a completed real mine response after Anvil has already executed it.
- `history.rs:254/368` and `abi.rs:164`: complete ordered transactions/receipts, status0 exclusions, canonical ABI, actual tx-local log positions plus original RPC-global indices, staged Set/Resume pairing before inclusion, complete prior cursor and immutable historical writes.
- `lib.rs:470/486/510`: separate simulated finality, per-reader continuous delivery, blocked future reads, exact-block-hash historical eth_call matching receipt-derived cursor. The production adapter trusts the configured private Anvil execution rather than supplying Ethereum proofs; independent receipt checking additionally recovered actual signatures against the declared deployment/signers.

## Commands and environment

The successful full run used one outer global lock and two build jobs:

```sh
env PATH=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:/home/jy/miniconda3/bin:/usr/local/bin:/usr/bin:/bin \
 L1_FOUNDRY=/mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0 \
 CARGO_HOME=/home/jy/.codex/worktrees/65ef/megaroation_poc/.build/cargo \
 CARGO_TARGET_DIR=/home/jy/.codex/worktrees/65ef/megaroation_poc/.build/target/l1-adapter \
 CARGO_BUILD_JOBS=2 \
 flock /tmp/megaroation-poc-heavy.lock bash poc/crates/l1-adapter/tests/validate.sh \
 /home/jy/.codex/worktrees/65ef/megaroation_poc/.build/artifacts/anvil-l1-verifier-87daee8-run2
```

Actual Python **`/home/jy/miniconda3/bin/python3`**, Python3.12.8, coincurve20.0.0. Rust/cargo1.95.0; Foundry1.0.0-stable `e144b82070619b6e10485c38734b4d4d45aebe04`; solc0.8.20+a1b79de6. Exact tool and driver SHA256 are in raw evidence `toolchain.txt` and `driver-sha256.txt`.

Environment attempt1 omitted `/home/jy/miniconda3/bin` from explicit PATH, selected system Python3.10, and stopped during the version precheck with `PackageNotFoundError: coincurve`. It performed no compilation or Anvil scenario and is **ENVIRONMENT_PREFLIGHT_FAIL, not candidate behavior FAIL**; its untouched log remains in `.../anvil-l1-verifier-87daee8-run1/`. The first automatic approval for corrected run2 timed out before process creation; its single permitted retry ran successfully. This was a timeout, not a safety rejection.

Independent follow-up commands (read raw evidence; no build):

```sh
/home/jy/miniconda3/bin/python3 poc/verification/anvil-l1/receipt_checker.py --bundle .build/artifacts/anvil-l1-verifier-87daee8-run2/pause-300/raw-rpc-bundle.json > /tmp/anvil-l1-verifier-receipts-300.json
/home/jy/miniconda3/bin/python3 poc/verification/anvil-l1/receipt_checker.py --bundle .build/artifacts/anvil-l1-verifier-87daee8-run2/pause-650/raw-rpc-bundle.json > /tmp/anvil-l1-verifier-receipts-650.json
/home/jy/miniconda3/bin/python3 poc/verification/anvil-l1/golden_vectors.py check --cast /mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0/cast
/home/jy/miniconda3/bin/python3 /tmp/anvil-l1-driver-check-87daee8.py > /tmp/anvil-l1-verifier-driver-check.json
```

## Observed matrix and scope

Each real run used seed `none (explicit script)`, blocks0–7 inclusive (8 headers), final relative virtual time16,000,000μs, 195 command/reply records and exactly7 actual mine RPC calls. Each had **145 included receipts:129 status0 and16 status1**. Independent configured-contract checking reconstructed11 protocol events and5 accepted semantic retries. Runs paused300ms versus650ms before the first mine; pending transactions had no receipt and head remained0. Raw driver trace, raw RPC bundle and canonical virtual trace were byte-identical across pauses.

| Cases | Result | Actual evidence |
| --- | --- | --- |
| A01 | PASS | Published compiled ABIs match; independent helper/hash matches on actual deployment, actual ECDSA recovery; fixed golden4 messages/57 mutations/40 malformed signatures/34 Cast checks pass. |
| A02 | PASS | Real wrong chain/address/salt/type/caller/rid and per-field control mutations have status0/no logs. |
| A03 | PASS | First seq0, skipped seq2, repeated/descending rejection, uint64 max acceptance then no reuse. |
| A04 | PASS | Valid original/alternative-signature retries return original outputs without events; bad signature and changed content reject; canonical trailing/aliased calldata have mined status0. |
| A05 | PASS | Only fixed region mask/order; l1Floor90000 is signed/recorded and accepted; unsupported mask/permutation rejects. |
| A06 | PASS | Actual closed two-height then one-height batches; permission/full-prev/source/range/origin negative receipts. |
| A07 | PASS | Real DA/Certifier wrong signer/domain/commitment/certificate replacement and malformed signatures rejected. No boolean verifier stub. |
| A08 | PASS | Equal/lower target, future origin and direct unauthorized Override reject. |
| A09 | PASS | Source RESUME(seq), zero frames, signed origin and same-tx Set→Resume; old pre-Override attestation rejects; post-Override batch succeeds. |
| A10 | PASS | Real Authority writes-before-invalid-Override reverts; independent test harness writes inner cursor then reverts outer transaction; status0/no logs and old cursor/seq/seen state remain; same request later succeeds. |
| A11 | PASS | Block4 contains Force and two separate Resume transactions; same actual act, distinct exact positions and linked pairs. |
| A12 | PASS | Block4 pair positions `(4,1,0)→(4,1,1)` and `(4,2,0)→(4,2,1)`, RPC global indices1/2 and3/4. |
| A13 | PASS | 6 historical cursor reads plus2 exact-event reads, original initialization/written_at, unchanged block3/empty-block history, A/B equality. |
| A14 | PASS |300/650ms wall pause, pending receipt null, no autonomous mine; exact slot timestamps including deployment. |
| A15 | PASS | Included/finalized/delivered frontiers separate; A/B prefixes independent, future read WAIT, out-of-order buffered range only advances after gap filled. |
| A16 | PASS | Same real bundle replayed through Rust coverage path: missing block/full tx/receipt/logs WAIT; wrong parent/index/tx/emitter and missing Resume in complete data REJECT; prefixes unchanged; repaired same command completes without an extra mine. |
| A17 | PASS | Real completed mine command152 loses response; new mine is blocked; original command reconciles without timestamp/mine RPC. Independent method allowlist/state check and whole-trace pause comparison pass. |
| A18 | INCONCLUSIVE | Real02 Clock/transport grant/ack and03 canonical public object mapping are not in this component. |
| A19 | INCONCLUSIVE | Real11 frame/DA/G sourcing,12 historical ResumeCut and13 recovery endpoints are not exercised. |

Independent checker negative controls also rejected five deliberate corruptions of the actual bundle: missing final receipt, wrong parent hash, block-global index replaced with local value, mismatched Resume sequence and incorrect original written_at. These are separate checker tests, not assertions that the trusted canonical Anvil emitted corrupt data.

## Evidence paths and digests

Raw evidence actual data directory:

`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/09/artifacts/anvil-l1-verifier-87daee8-run2`

Its worktree alias is `/home/jy/.codex/worktrees/65ef/megaroation_poc/.build/artifacts/anvil-l1-verifier-87daee8-run2`. It includes exact revision/clean-status, validation command log, compiled artifacts, raw RPC, deployment, command traces, diagnostics, case labels, all receipts and hashes. Own Anvil children were terminated by the runner; no global service/port was used.

- driver binary SHA256: `7bf99a50f6f9035e609f5e0df6ea2bb5e2bca8f7447f5119da796fca79126c9e`
- each raw driver trace: `4729922497f770808b8088f596546c13b4ac5ca15ebefd77e8c589586e818561`
- each raw RPC bundle: `c4eee3e00c96b524e31802d71ab883764aa6733beb0686f2ec8b86194eb5426c`
- each virtual trace: `114bb15dc6c444da432ab5841dde21792ab8aa97032d0a7c42eb48a85b20b025`

Independent results:

- `/tmp/anvil-l1-verifier-receipts-300.json`
- `/tmp/anvil-l1-verifier-receipts-650.json`
- `/tmp/anvil-l1-verifier-driver-check.json`, reproduced by `/tmp/anvil-l1-driver-check-87daee8.py`
- `/tmp/anvil-l1-verifier-checker-negatives.json`

## Limits

This is finite component evidence on a fresh private Anvil, not a proof, Ethereum consensus/MPT verifier, public-network integration or completed Force/RESUME production recovery. Driver command/request reconciliation state is in memory; restart/durable recovery is not claimed. Trusted local RPC canonical execution is a declared premise. No malicious RPC Byzantine safety, native EigenDA/TEE/staking or general partial-channel behavior is claimed. Opaque gpos fixtures prove exact bytes/signature binding, not valid canonical GFR replay. The script signs frame commitments but does not prove actual11 DA persistence or materialize G/G′/D. No ResumeCut is fabricated. Arithmetic extremes beyond the specific seqmax vector are supported by source guard review, not an exhaustive runtime exploration.

No production source or checked-in expected artifact was changed during this review; HEAD remains candidateC. Tester must use the same C and record its own commands and evidence. A later report-only commit may reference tested C but must not relabel it as the tested source revision.
