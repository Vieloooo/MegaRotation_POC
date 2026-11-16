# Feature 03 public interface delivery

Owner: feature 03 lead. Date: 2026-11-15 (Asia/Shanghai). This records consumable interface commits under [staged dependency plan v2](commander/dependency-plan-v2.md). It does not count feature 03 as accepted.

| Interface | Exact consumable SHA | Scope and evidence |
| --- | --- | --- |
| M1 | `0e72dc247de22afdc8d6681713b3d15f4aa05262` | Checked pid/region/member/signer indices, opaque Hash32/Cid/ConfigRef, L1Position/L1BlockRef/ResultKey, strict BE codec. Seven independent actual API tests passed; [independent review](../verification/mock-plan-execution/implementation-v1/m1-review-0e72dc2.md). |
| M2 shared objects | `fadde7e84b5eee9dc528b2c318127a7c2c5721df` | Plan/ProgramConfig/Descriptor, state/transaction/physical/native codecs, bounded dep/L1 fixtures, typed reference/reader/final-verifier capabilities and noncanonical evidence/replay/lifecycle skeletons. Five independently authored actual codec tests plus all seven M1 tests passed. The Verifier inspected this exact revision and reported no contract violation; full authorization/core behavior remains pending. |

The M1 contract is [public interface v1](contracts/mock-plan-public-interface-v1.md). M2 uses [implementation contract v2](contracts/task-contract-mock-plan-execution-v2.md), SHA-256 `a18d2b33f9c541abc81b4fead601c3ccaead6221c90dc43345d52a850429cc15`. The contracts and independently derived inputs were committed before their implementations (`dae686b7152a4b8dabcc670fa08e9abd88bf276d` and `862cc16`, respectively). M2's forty fixed BE records are [here](../verification/mock-plan-execution/implementation-v1/m2-golden-vectors.json); their SHA-256 is `5bbd96c791ce2a1db148f8a627b6a8d34070e229a941f564cb45b06f78ea3aca`.

Consumers import `poc_types::domain`; the original root I1 exports, protobuf, hash functions and encodings remain unchanged. M1 uses `Canonical`; bounded M2 objects use `CanonicalObject::encode_checked/decode`. Constructors and structural codecs do not authenticate references, PoPs, final proofs or L1 provenance. Public field structures require their explicit validation step. `FinalVerifier` has no default implementation. Typed replay/lifecycle/evidence values have no approved canonical wire encoding in this increment.

02 can consume identity and opaque context/config references. 06 can consume ResultKey and the concrete public body/result structures while preserving its own log/reader ownership. 09 can consume exact L1 positions/header references and `FinalizedL1Reader`/`InputStatus`; its EVM ABI and real finality adapter remain its responsibility. 04 must supply the actual native final-certificate adapter; 07 owns complete replay semantics/encoding and 12/13 the historical recovery cut.

M2 actual-library command, executed on `fadde7e84b5eee9dc528b2c318127a7c2c5721df` before the authorization/core workspace expansion:

```sh
flock /tmp/megaroation-poc-heavy.lock env CARGO_BUILD_JOBS=2 REGIONAL_BFT_RUST_BIN=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin ./poc/scripts/bootstrap.sh test -p poc-types --locked --offline
```

CWD: `/home/jy/.codex/worktrees/1c43/megaroation_poc`; Rust 1.95.0; no random input (`seed=null`). Raw combined stdout/stderr: `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03/artifacts/m2-types/codec-tests.log`, SHA-256 `e6bca9c19f3e40d6c6452090b10de813f14bf499545315ca12bb811d1a353c18`. This is component evidence, not a true BLS final-certificate, runtime recovery, 15-worker, Anvil or complete feature-03 verdict. Subsequent integration must name and verify its own exact candidate SHA.
