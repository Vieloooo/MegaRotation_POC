# M2 independent expectations — production BE profile

2026-11-15. Authority: [implementation contract v2](../../../docs/contracts/task-contract-mock-plan-execution-v2.md), SHA-256 `a18d2b33f9c541abc81b4fead601c3ccaead6221c90dc43345d52a850429cc15`, together with the pinned v10/v2 sources. New expected data was derived before reading M2 implementation. M1 code was reviewed separately only after its own expectations were fixed. No historical LE proposal bytes were rewritten or adopted.

`m2-inputs.json` and `m2-golden-vectors.json` fix40 production-BE records: Metadata, sparse ExecutionState, raw Transfer/Credit/Attr, PhysicalBlock, NativeBlock, ExecutionAnchor, DepFixture, bounded L1Fixture, empty-control commitment, PublicConfig, PlanInput, Boundary, PlanRecord, ProgramConfig, authorization/cid preimages, Descriptor and derived anchor identity. Hashes use the frozen `PoC/<Type>/v1` domain names with existing u64BE length framing, not the old preparation domains.

The manually derived main execution values remain A=(nonce2,balance90), B=(2,15), C=(0,19), with Kept raw indices `[1,3,4,6]`. The initial state excludes the explicit zero B account as required by the production sparse codec. Each saved state retains the metadata of the height actually executed. Physical h3 pre-root is h2 post-root, before current Begin/DEP. Physical anchor id is `11`×32; native genesis parent is `aa`×32. Neither is substituted for the other.

The complete domain/hash preimages were independently materialized with standard Python integer-to-byte conversion from the manual inputs, then checked by a separate parser/encoder/execution oracle using `struct`. `check_m2_vectors.py` has no expected-update mode and imports no production code. It currently checks170 assertions,40 byte records and14 hand-calculated execution boundary cases. These are reference checks; actual implementation status is reported separately by candidate review.

| Frozen value | Expected production hash |
| --- | --- |
| h3 pre-root / h2 post-root | `df9c31e30169fdcf76b8539ec33fa9c400278362d415d8c670d61ee34083758a` |
| h3 post-root | `76bffa42f643077ad9e15c1f7a098c03cfe5e5562b791f7ef932493c83360349` |
| physical h3 | `806e5423463b9326eefc8771b2f0f8b6ff953509e714045d3a8e7bd4b3431cac` |
| native h3 | `bb6b678ab60e51eb8125a7c3279a6fa3f856d9611c8b7a297376a52a35077d85` |
| example PublicConfig | `564fecf01e8c3a00990c91291c695285dea08416fa34ed4c852090694e5ec659` |
| example cid | `001ebf7d42684ada032b8091c56d73c1c25e22daed0cc9731f9968ba96e55ae7` |

The example Plan's committee bytes and registry-hash reference deliberately isolate **structural codec/preimage binding**. They are not a cryptographically validated registry and must not pass registry/Plan source validation. Actual crypto uses a separate public fixture generated only with pinned Commonware primitives, using `registry_fixture_recipe.rs`: per-pid identity seed303000+pid, BLS seed303100+pid, ChaCha20Rng and namespace `PoC/Registry/PoP/v1`. The recipe emits only public keys, PoPs and negative signatures; it never serializes private bytes. These deterministic test keys are explicitly public test material, not operational keys. Production outputs are never used to choose the expected registry mapping or digest.

`registry-negative-vectors.json` separately fixes strict identity rejection: canonical Ed25519 identity and order4 point, noncanonical encoding of identity, mixed-torsion point, off-curve encoding, and valid prime-subgroup basepoint. `check_identity_vectors.py` independently verifies those expectations with finite-field Edwards arithmetic; it is a small offline oracle, not a constant-time crypto implementation. BLS zero/infinity and width negatives, wrong PoP namespace/key/kind, duplicate identities/vote keys and normalization remain mandatory actual crypto tests. Commonware's Ed25519 ZIP215 decode alone is insufficient for the stricter registry contract; BLS G1/G2 decode does have the required infinity/subgroup guards.

`execution-boundaries.json` fixes zero/self/unknown-account behavior, nonce and receiver overflow skips, credit-overflow atomic rejection, malformed-tail rejection, no zero-credit account creation and config byte-limit rejection. The configured byte limit is enforced through `apply_with_byte_limit`/Merge; a global-limit unconfigured apply helper must not pretend to consume a missing config. DEP must start with exactly one matching Attr, followed only by Credit. All input decoding precedes any committable result; no failure yields a partially applicable state.

`m2-component-scenarios.json` fixes the remaining actual component gates. In particular: source Finalize and reader Deliver are separate explicit inputs; advancing a clock does not expose an on-disk row; each reader's header prefix and control coverage are distinct; out-of-order input may stage without becoming visible; wrong source/conflict preserves prior state. Metadata computes the fixed scheduled origin and anchor clamp before requesting that actual origin from the reader, and cannot replace missing input with the reader's last available header. Merge has an explicit fixed Window, expands only through its supplied physical tip, and allows covered padding below a successor's physical minimum.

Exact reference commands (CWD `/home/jy/.codex/worktrees/1c43/megaroation_poc`, Python3.12.8, seed=null):

```sh
python3 poc/verification/mock-plan-execution/implementation-v1/check_m2_vectors.py
python3 poc/verification/mock-plan-execution/implementation-v1/check_identity_vectors.py
```

`m2_codec_acceptance.rs` is the independent actual-library test written from this contract and these immutable inputs. It constructs the public structs independently and compares actual checked bytes/hashes, including malformed lengths/counts, invalid sparse/config objects and strict raw transaction parsing. It does not perform PoP, Apply, Merge, L1 state transitions or native final certificates; those require the later component harness on a named candidate. The source/expectation distinction and each limit must remain explicit in final reports.
