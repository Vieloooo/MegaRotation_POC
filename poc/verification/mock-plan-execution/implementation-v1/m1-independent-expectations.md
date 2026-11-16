# M1 independent expectations — public domain interface v1

2026-11-15. This is new implementation-stage material. The parent directory's LE64 preparation profile remains historical and is **not adopted**. [Dependency plan v2](../../../docs/commander/dependency-plan-v2.md), merged at `548cbb24b390fedf2ddfd60f3aab0de770e24491`, authorizes feature03 independent components without waiting for all of01. Full feature acceptance and runtime integration remain separate.

These expectations were derived before inspecting the new implementation. Only v10/v2, the accepted lead/Docer interface decision, and the existing I1 public types/protobuf definitions were read. There is no import of production code into the reference generator/checker. Native BLS and PoP work belongs to the next03 component verification, and no heavy command has run for M1 preparation.

The approved M1 interface is a new `poc_types::domain` module. It must preserve all existing root `wire::*` exports and I1 helpers; the old protobuf `poc_types::Cid` must not be replaced by `poc_types::domain::Cid`. `hash_parts` keeps its existing signature, return type and exact length framing. M1 introduces no implicit method that re-hashes an opaque Cid into a different identity.

## Fixed domain and encoding cases

| Type / operation | Expected behavior |
| --- | --- |
| `Pid`, `RegionId`, `MemberId`, `SignerIndex` | Private u8 storage. Checked construction and serde deserialization accept respectively0–14,0–2,0–4,0–4; boundary+1,255,negative and256 cannot create a valid value. |
| pid mapping | `pid=5*region+member`; round-trip all15. SignerIndex is a different type from MemberId; no implicit identity mapping is assumed. |
| `Hash32`, `Cid`, `ConfigRef` | Exactly32 raw bytes; reject lengths0,1,31,33,64. Zero bytes are allowed as opaque data, and do not certify valid configuration or descriptor provenance. |
| `L1Position` | Three u64 fields block/tx/log. Lexicographic comparison uses all three fields, including max-u64 and same-block/same-tx distinct logs. |
| `L1BlockRef` | Height:u64, hash:Hash32. Both fields participate in equality/encoding. |
| `ResultKey` | Cid and physical block hash. Both independent32-byte components survive equality/encoding. |
| `Canonical` | `encode()->Vec<u8>`, `decode(&[u8])->Result<Self,DomainError>`. Every object starts u16BE version1, followed by fixed fields. Identity uses u8, position/height use u64BE, hash fields are raw32B. Nested fields do not repeat version headers. Unknown version, truncation and trailing bytes reject. |
| Type/domain scope | Identical raw payloads can have identical M1 bytes in different typed wrappers; M1 is typed data. It does not declare those wrappers interchangeable or invent a new certificate/signing domain. M2 object hashing will use explicit versioned domains under existing `hash_parts`. |

The valid/negative examples in [m1-vectors.json](m1-vectors.json) were materialized before implementation:43 fixed objects,68 invalid encodings, all15 pid mappings and five shuffled L1 positions. SHA-256 fields labelled `encoding_sha256` fingerprint the fixture bytes; they are not implicit Cid hashes or new protocol IDs. Length failures return exactly InvalidLength with expected/actual sizes. The approved precedence is total length, then version, then field domain. The length-before-version cases deliberately exercise both faults; other malformed examples isolate one fault. Stable on-wire error numbers remain out of scope.

The independent checker verifies fixed literals and their arithmetic/shape. Actual public-API checks must additionally prove private checked construction, serde range guards, cross-type separation, stable imports and exact production encode/decode output; these cannot be passed by Python reference checks alone.

## I1 compatibility expectations

[i1-hash-compatibility.json](i1-hash-compatibility.json) contains nine fixed hashes from the public rule: SHA-256 over the UTF-8 domain and every part, each independently preceded by its u64BE byte length. Empty list differs from one empty part. `[ab,c]` differs from `[a,bc]`. Domain length counts UTF-8 bytes. The new module must not change any of these outputs.

Two fixed protobuf messages exercise existing `grant_hash` and `ack_hash`: canonical bytes omit incarnation and the corresponding self-hash field, while keeping all other listed fields. Production tests must vary incarnation and self-hash without changing the expected result; mutating the logical event, input or protected state must change it. This is I1 compatibility checking, not authorization of a new protobuf schema.

Root constants remain `SCHEMA_VERSION=1`, `MAX_FRAME_BYTES=1048576`, `MAX_PAYLOAD_BYTES=65536`, `LINK_QUEUE_BOUND=64`. Existing `Grant`, `Ack`, `wire::Cid`, digest/hash helpers and their public signatures remain available. Full I1 execution/regression is a later integration gate; M1 preservation tests do not establish01 recovery acceptance.

## Before coding / verification gate

Freeze the public-interface document and these fixture bytes before M1 implementation. Run the small checker with Python3 only:

```sh
python3 /home/jy/.codex/worktrees/1c43/megaroation_poc/poc/verification/mock-plan-execution/implementation-v1/check_m1_vectors.py
```

It has no update mode and no RNG (`seed=null`). Production validation will name an exact candidate SHA, compare actual library output to these immutable expected bytes, and report separate component versus integrated statuses. All Rust/heavy commands require the shared lock and `CARGO_BUILD_JOBS=2`, with task03's data-disk target/artifact directory.
