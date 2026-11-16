# M2 independent component review — e5f77d2

Date: 2026-11-16. **PASS for the frozen component scope, contract:B01–B16; no blocking code findings.** Candidate and independently exercised source: `e5f77d27edde91a61f7400d1a61961e4cbe09c9c`. Contract r2: `b9b8b15` (`BS-COMPONENT-v1`); source-first expectations originated at `8fb46c296808dbe97c95df7a2761678412b95f62`. M1 codec/crypto interface candidate was `75b574d40930b7c9c748a4c6b6e381117c937537`; those two production source files have no M2 diff. Integrated baseline is `c9805f7a818bc23d30018ce4dc6f1ee45f92e3ac`, consuming 03 `e9c15e0` and 09 `c443ae9` without rewriting their histories.

Worktree: `/home/jy/.codex/worktrees/f025/megaroation_poc`; branch: `codex/poc-batch-services`. Reviewer: named `bft_verifier`, configured Astra/xhigh. Main Astra/ultra was verified separately by lead. Builder stopped writing before this review. The formal run began with a clean worktree; subsequent differences contain only independent verification artifacts. No production source was edited by this reviewer.

**This is not complete feature11 acceptance.** Contract:B17's actual feature12 execution and contract:B18's full 02/07/08/12/13/system integration remain open. Public G/gpos inputs are explicitly fixtures. No GFR proof, full G′, D, ResumeCut or native BFT crash/restart result is inferred from this component PASS.

## Independent run and observations

The reviewer reran the candidate's full component validation in a new directory, rather than adopting Builder's preflight. Both private manual-mining Anvil runs used the same explicit script and imposed wall pauses of 300 ms and 650 ms. Per run:

- 7 complete headers (genesis plus six mined blocks), 8 actual receipts.
- Two original successful CursorAdvanced events: L2 heights1–2 at L1 `(2,0,0)`, and height3 at `(3,0,0)`.
- One successful same-request retry with no new Advanced event.
- One actual RESUME/Override at block5; Set `(5,0,0)` is paired with Resume `(5,0,1)`, control sequence0.
- Three mined status0 transactions at block6: stale prev, wrong Certifier signer, and changed commitment with incompatible DA certificate. Diagnostic eth_call revert selectors supplement, but do not replace, those receipts.
- 119 explicit driver commands, 20 matching mutating RPCs (8 sends, 6 timestamp assignments, 6 mines), 8 independently verified EIP-155 raw transaction signatures.
- Ten real child process `_exit(91)` injections and one SIGKILL after actual broadcast before reply. The driver/Anvil and other actors continue running; the targeted role is reopened on the same durable root in a different process.
- Final virtual reference14,000,000μs; included/finalized and all three reader-visible frontiers are L1 height6. This is a bounded scripted component trace, not a timing/liveness theorem.

Independent raw-receipt/ABI/signature/DA checking passed for both runs. Independent 03 execution checking matched all three accepted heights' Kept records and all four source heights' observed poststate/native hashes. h1 keeps transfers0 and3 and filters nonce/balance failures1 and2; h2 is an empty ordinary record; h3 is executed from the state reached through h2. h4 is a source/attestation negative-test height, not a fourth accepted height.

The independent driver audit reconstructs each frontier transition, verifies three distinct WAIT states before inclusion/finality/delivery, compares every returned block/receipt to the raw bundle, and pairs actual mutating RPCs with commands. It independently verifies persisted job IDs, raw transaction bytes/hash/nonce/calldata, three saved Certifier signatures, and final task states **Accepted, Accepted, Reverted**. Every crash's after/reopened file hash map matches; identity files remain unchanged. The DA bytes-only checkpoint precedes receipt existence; acknowledged payload/receipt hashes remain equal to final files. The send/SIGKILL boundary leaves the persisted outbox unchanged and preserves its original hash/nonce.

Protocol trace, driver trace, raw RPC bundle, accepted records and source/execution artifacts match byte-for-byte across both wall pauses. Crash traces match after removing OS-assigned pid/reopen_pid. These pauses are real wall delays; their values do not enter protocol timestamps or source selection.

## Source review and contract coverage

The source/independent expectations preceded implementation. Coverage uses **contract:Bxx** below; early `source:Bxx` semantic-vector names are a different namespace.

| Contract IDs | Evidence and conclusion |
| --- | --- |
| B01–B03 | Frozen frame/EIP-712 Rust tests rerun, prior independent M1 byte/recovery/calldata checks remain applicable because frame/crypto source is unchanged, independent actual payload/signature/Kept/poststate checks, and real wrong-signer/DA negative transactions. PASS. |
| B04–B05 | `storage::Root` bounds reads, holds one writer lock, syncs created directory chains, writes/syncs temporary files, publishes immutable DA with non-replacing hard-link semantics, then syncs the directory. DA bytes precede signing; receipt persistence precedes reply. Same bytes/receipt survive actual exits before bytes, after bytes, and after receipt. Missing/damaged backing data waits without a valid DAObject; Disperse refuses to overwrite corruption. PASS within the declared filesystem/process model. |
| B06 | Source identity/config/DEP binding, typed position shape, exact metadata and pre-state checks, own 03 Apply execution, raw→Kept comparison and missing material WAIT. Batcher and Certifier have separate fixture views/readers. The source data and execution checkers independently matched the exported run. A contradictory pre-state fixture rejects. PASS; source validity remains fixture-scoped. |
| B07 | Certifier rechecks its current sealed reader cursor, latest exact paired RESUME, source execution and DA before consulting cached results. Actual cursor advance rejects cached old prev; actual Override yields required Resume log1 even though Set/written_at is log0, and unreplayed recovery waits. Later GLOBAL retaining the prior Resume is a separately labeled history-selection fixture, not an actual resumed G run. PASS for component guards. |
| B08 | Signature persistence and return ordering inspected. Two real child exits before signature persistence and after durable signature, then same-root reopen/replay preserve result identity. Role manifests bind domain/config/role/reader/signer/source fixture. Configuration mismatch rejects. PASS. |
| B09–B10 | Stable job ID derives from domain/full prev/payload commitment; complete command inputs are deduplicated and conflicts reject. Atomic state publication preserves phases/terminal records. Five Batcher persistence exits and the real-send SIGKILL retain raw tx/hash/nonce before exposure, unknown result waits, and reconciliation locates original finalized acceptance or definite failure. Different job reuse of the submitter nonce and replacement of an existing raw outbox reject. PASS. |
| B11 | Two actual Anvil original advances, immutable DA files, full public receipt/calldata/signature artifacts, state-changing ordinary inputs and invalid nonce/balance filtering. Same gpos supplies successive ranges by executing the fixed snapshot prefix to prev, as clarified in r2. PASS. |
| B12 | Actual concrete `Driver<AnvilRpc>` gateway, private constructors/no trusted Deserialize, independent reader/finality/delivery checks, actual status0/no-event-retry exclusions and exact log identities. Additional verifier-owned adaptation of fixed09's reader fault suite replays these new actual receipts: four missing-data cases WAIT, five metadata/emitter/pairing corruptions REJECT, then repaired history advances once; this is explicitly a replay fixture. PASS. |
| B13 | Stale certificate is prepared before actual Override and submitted with a new logical request, yielding mined status0; Certifier independently rejects stale current prev and waits for unreplayed Resume. Same-request successful replay is tested separately. PASS. |
| B14–B15 | Actual accepted records survive DA unavailability/damage; payload loading waits at original position. Exact-gpos materialization compares payload commitments and cross-batch state/native continuity. Missing gpos returns only known materialization; CursorSet without ResumeCut moves the old prefix to history and reports the exact missing cut. No fake D/G′ completion. PASS for supported incomplete-view behavior. |
| B16 | Independent per-role availability controls, actual process exits, and 300/650ms wall-pause comparisons with identical protocol evidence. Source/crypto/state logic reads no wall time or scheduler future faults. PASS for this explicit bounded script. |
| B17 | Single production frame API and actual DA/receipt samples are delivered. Feature12's own D execution and recovery integration: OPEN/INCONCLUSIVE. |
| B18 | Full public G/gpos codec/replay, 02 controlled grant/ack transport, position-bound ResumeCut and system integration: OPEN/INCONCLUSIVE. |

`source.rs` replays from its configured snapshot anchor through the requested last height, checking intermediate inputs before selecting the suffix. Without a required RESUME established by current complete09 history, ordinary receipt advancement does not demand a newer source gpos or source control coverage. With a requirement, both exact processed pair and coverage of its Resume position are required. A mutable alias of one delivered gpos is rejected by FixtureSourceView; future real GFR source immutability/validity remains the provider's integration obligation.

`Batcher::reconcile` consumes a sealed, complete ordered prefix for its own reader/domain. It prefers the original matching accepted request, otherwise adopts only the raw tx's definite failed receipt; absent evidence leaves the same SubmittedUnknown outbox. Terminal replay returns persisted results. Public crypto/transaction helpers are primitives, not independent actor outputs; the production role methods persist before returning host-visible signed data.

No actionable P1/P2 source findings were identified. A preliminary independent-audit attempt represented `evm_setNextBlockTimestamp` as an integer while the fixed driver records the equivalent canonical hex quantity; the verifier adapter was corrected and rerun. This was a checker representation mismatch, not a candidate behavior failure.

## Commands, artifacts and reproducibility

Toolchain actually used: Rust/Cargo1.95.0, Python3.12.8 at `/home/jy/miniconda3/bin/python3`, Foundry1.0.0-stable commit `e144b82070619b6e10485c38734b4d4d45aebe04`, solc0.8.20 `a1b79de6`; secp256k1 Rust dependency is pinned by the candidate lockfile. Tool binary hashes are in the raw `toolchain.txt`. Commonware2026.10.0 and Regional BFT gitlink `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6` remain unchanged.

Formal run command from the worktree root:

```sh
export PATH=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:/home/jy/miniconda3/bin:$PATH
export CARGO_HOME="$PWD/.build/cargo"
export CARGO_TARGET_DIR="$PWD/.build/target/batch-services"
export CARGO_BUILD_JOBS=2
export L1_FOUNDRY=/mnt/reth/jy/megaroation_poc/.build/tools/foundry-1.0.0
flock /tmp/megaroation-poc-heavy.lock bash poc/crates/batch-services/tests/validate.sh \
  "$PWD/.build/artifacts/m2-verifier-e5f77d2-run1"
```

For each `pause` in `300 650`, independent commands were:

```sh
/home/jy/miniconda3/bin/python3 poc/verification/batch-services/check_accepted_evidence.py \
  --bundle .build/artifacts/m2-verifier-e5f77d2-run1/pause-$pause/run/raw-rpc-bundle.json \
  --da-index .build/artifacts/m2-verifier-e5f77d2-run1/pause-$pause/run/da-index.json
/home/jy/miniconda3/bin/python3 poc/verification/batch-services/check_execution_evidence.py \
  --source .build/artifacts/m2-verifier-e5f77d2-run1/pause-$pause/run/execution-source.json \
  --accepted-result poc/verification/batch-services/reports/m2-accepted-$pause-e5f77d2.json
/home/jy/miniconda3/bin/python3 poc/verification/batch-services/check_m2_driver_and_crash.py \
  .build/artifacts/m2-verifier-e5f77d2-run1/pause-$pause/run
```

Their stdout is retained in this report directory as `m2-{accepted,execution,driver-crash}-{300,650}-e5f77d2.json`. The verifier-owned reader fault consumer used its own lockfile and the same two-job heavy lock:

```sh
export L1_ADAPTER_RPC_BUNDLE="$PWD/.build/artifacts/m2-verifier-e5f77d2-run1/pause-300/run/raw-rpc-bundle.json"
flock /tmp/megaroation-poc-heavy.lock cargo test --locked --offline \
  --manifest-path poc/verification/batch-services/consumer-m2/Cargo.toml \
  --test reader_faults -- --ignored
```

The scenario seed is `none (explicit script)`. Test signer keys follow the visible synthetic scalar recipe1–4 in the candidate harness; they are local PoC identities, not user credentials. Existing frozen/reference vectors have their separately recorded seeds. No network production service or external user data was involved.

Absolute main evidence root: `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11/artifacts/m2-verifier-e5f77d2-run1`. Additional reader fixture log: `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/11/artifacts/m2-verifier-e5f77d2-reader-faults/test.log`. `m2-manifest-e5f77d2.json` inventories every file under both evidence roots and the independent reports/source checkers with SHA-256.

Actual reusable DA samples under either run's `run/da/`:

| Heights / L1 position | Keccak commitment | Bytes / payload SHA-256 |
| --- | --- | --- |
| 1–2 / `(2,0,0)` | `3719037b49943e3c4dfdb7488fd54eb94cbfa8b849e4e8339ea5f03cc7ca3ef2` | 117 / `f84fd2986e31829e19bbad01784c5d4701d43febea0d42ed8fd3877073437e5c` |
| 3 / `(3,0,0)` | `5629543d719715cc831f03dd53c111bd0112681dd05eee06ae084f92f1e9448d` | 74 / `dc09763e46982ed64c6f28e94af3163e6cbad48fefc4a8b6413f16e732aa7b93` |

## Limits and handoff

Actual crashes are Linux process termination/reopen on the same host and filesystem. The fsync ordering was reviewed; the tests do not simulate power loss, kernel crash, storage hardware failure, Byzantine actors or indefinite retention. Missing/corrupt DA injection is separately labeled and never counted as an ordinary crash or absent batch. Delay does not substitute for process failure.

The contract's restart-free BFT worker profile may support regular/Force/RESUME without waiting for full01 native recovery. These service tests do not execute those full handoffs or establish BFT crash safety. G fixtures are bounded to four heights; no later source heights or completed ResumeCut are invented. Finite tests do not establish A-DELIVER/CERT-5/DA-RET timing or retention guarantees.

Tester may now run its agreed component matrix on **the same candidate SHA**. Any production fix or interface change invalidates affected verification and requires re-review. Do not count this component PASS as a complete accepted feature or merge it to main without the coordinator's separate process.
