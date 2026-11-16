# Independent M1 review — 0e72dc2

2026-11-15. Named reviewer: `bft_verifier`, GPT-6 Astra / xhigh. Candidate: **`0e72dc247de22afdc8d6681713b3d15f4aa05262`**. Result: **PASS for the frozen M1 public-interface component**. No actionable implementation finding remains in this reviewed scope. This does not accept all of03,01 recovery,04 native certificate verification, or runtime/15-worker integration.

Independent expectations and JSON literals were frozen before inspecting the new `domain.rs` implementation. The reviewed contract is [mock-plan-public-interface-v1.md](../../../docs/contracts/mock-plan-public-interface-v1.md); the independent source/compatibility expectations are [m1-independent-expectations.md](m1-independent-expectations.md). Expected inputs were not regenerated from production output:

- `m1-vectors.json`: SHA-256 `8b02a346161e3e4597d339818338ab1434a77796acdfd45d14ae412ff82fbf6c` —43 valid encoded objects,68 malformed encodings,15 identity mappings and exact L1 order.
- `i1-hash-compatibility.json`: SHA-256 `19614c686c75c2fcb032b25b76b504812f1dbedc22cb9af9c664c48e7aa8813f` —9 fixed framed hashes including actual I1 Grant/Ack expected bytes.

Read-only inspection of the named candidate confirmed private checked identity constructors and checked Serde conversion; exact32B opaque references; total-length→version→field guard order; fixed BE encoding without repeated nested versions; u64 L1 positions and lexicographic equality/order; typed separation of member and signer index; and no implicit opaque Cid rehash. The only change to `types/src/lib.rs` is `pub mod domain;`; the existing wire export, I1 constants and hash helper bodies are unchanged. The manifest addition only registers the independent integration test. No protobuf or existing I1 encoding was changed by M1.

The independent actual-library run passed **7/7 tests**, including exhaustive u8 identity constructor/Serde bounds, fixed byte/decode/error vectors, hash-length/zero handling, all reference fields, I1 helper/import compatibility and Grant/Ack incarnation/self-hash exclusion. The reference Python suite separately passed233 assertions. No broader Rust test matrix was run because this component change supplied no additional unresolved concern requiring it.

Exact Rust command, from `/home/jy/.codex/worktrees/1c43/megaroation_poc`:

```sh
flock /tmp/megaroation-poc-heavy.lock env REGIONAL_BFT_RUST_BIN=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin CARGO_BUILD_JOBS=2 /home/jy/.codex/worktrees/1c43/megaroation_poc/poc/scripts/bootstrap.sh test -p poc-types --test mock_plan_m1_independent --locked --offline
```

Rust1.95.0; Python3.12.8 runner; seed=null; task03's own Cargo cache/target through the `.build` link. HEAD was the candidate before and after the run, with no tracked implementation modifications. A prior sandbox attempt could not create the task data-volume evidence directory (`EROFS`); it yielded no usable evidence and was not a code verdict. The same bounded command then ran with approved data-volume write access.

Raw evidence directory:

`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03/artifacts/verification/m1/0e72dc247de22afdc8d6681713b3d15f4aa05262/`

| File | SHA-256 |
| --- | --- |
| `stdout.txt` | `63921f824e0fee5ffafe950c5eab3985e8e18b769fa6a662a6cad253bba27d0c` |
| `stderr.txt` | `fc4c591913f740fff70d04aef88caaa5184dc8d9319b2ff5f055c47d8609e27f` |
| `run-manifest.json` | `89b601c3a158db6fbe2c912fb797fb2cd7cddd534f6a58eaa41fe133c133c896` |

The manifest records exact argv/CWD, toolchain, candidate, statuses, vector hashes and raw-output hashes. Stderr contains ordinary Cargo completion lines, not test failures. M1 is suitable for downstream component development under its versioned contract. Registry/PoP, Plan/Descriptor/physical codecs, execution/Merge, L1 provenance and native certificate adaptation still require their own later component acceptance.
