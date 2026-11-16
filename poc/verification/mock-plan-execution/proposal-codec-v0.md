# MPX-PREP-LE64-SHA256-v0 — independent preparation fixture

Status: **P / PROVISIONAL**, 2026-11-15. This is an executable byte illustration for review, not an approved PoC codec, Commonware signing format, Solidity ABI, or native Ethereum block encoding. No implementation may import this profile without the coordinator's explicit interface decision. The source fixes semantic fields and binding obligations; it does not fix these bytes.

`u8`, `u32le`, and `u64le` are unsigned integers of the stated width; overflow and non-integral/negative values reject. A digest is exactly 32 bytes. `LP(x)=u32le(len(x)) || x`; `List(xs)=u32le(count) || LP(x0) || ...`. Types are disjoint ASCII domains terminated by NUL. SHA-256 is used only for this proposal. JSON files are transport for the fixture and are never themselves protocol hash input.

| Object | Exact proposal byte sequence |
| --- | --- |
| Metadata `M` | `u64le(height) || u64le(timestamp_us) || u64le(origin)` |
| Transfer `T` | `01 || u8(from) || u8(to) || u64le(nonce) || u64le(amount)` |
| Credit `D` | `02 || u8(to) || u64le(amount)` |
| Attributes `A` | `03 || M` |
| State | `MPXPREP/STATE/v0\0 || M(current_context) || u32le(account_count) ||` accounts in ascending id order, each `u8(id) || u64le(nonce) || u64le(balance)` |
| Physical block | `MPXPREP/PHYSICAL/v0\0 || physical_parent_digest || M || pre_state_root || List(raw_ordinary_txs)` |
| Native block | `MPXPREP/NATIVE/v0\0 || native_parent_digest || M || List(DEP) || List(Kept) || post_state_root` |
| CID illustration | `REGIONAL\0 || u64le(beta) || u64le(r) || anchor_id || LP(program_config_bytes)` |

The CID example treats `program_config_bytes` as an opaque, fixed input (`"proposal-config-v0"`). It exercises input binding only. It does not supply a complete ProgramConfig/Plan/registry codec. Anchor exact state is not a Descriptor identity field (v10 02 §3.2), but Enter must verify that state against its fixed anchor.

The execution fixture has accounts 0=A, 1=B, 2=C. All three are explicitly present. A transfer has valid fixed width and opcode before semantic validation. A sender nonce mismatch or insufficient balance skips the transfer with no state change. A successful transfer moves the amount and increments sender nonce. Begin replaces the context with the actual metadata, then applies ordered credits; the attributes transaction is first in DEP. Kept contains only executed ordinary transactions, preserving input order. The fixed positive cases use distinct sender/receiver, positive amounts, existing accounts, and no overflow.

Unknown accounts, zero-value/self transfers, account/nonce/balance widths, overflow behavior, system transaction errors, maximum payload/list sizes, and context's inclusion in the actual state-root codec are **not decided by this fixture**. The independent acceptance table reserves them for the interface decision. In this illustration malformed transfer encoding rejects before Apply; a prior state with an already-applied height cannot be silently treated as the state before that height. Re-evaluation from the same immutable parent must return identical results; caching across a different parent/cid needs an identity-bound key.

For the fixed gap example, anchor height 0 has metadata `(0,10000000,3)` and state A=(nonce 0,balance 100), B=(0,0), C=(0,7). Physical height 3 skips physical heights 1 and 2. Each virtual height still runs Begin/DEP. Its physical parent is `11` repeated 32 bytes. The native parent before height 1 is `aa` repeated 32 bytes; subsequent native parents are the actual preceding native hash. These are deliberately distinct identifiers.

Golden bytes in `golden-vectors.json` are frozen fixture literals generated from the hand-written states in `semantic-vectors.json` by a separate preparation calculation. The checker has no update/generate option and does not load production modules. It decodes the fixed transaction bytes, independently applies them, compares manual state/Kept expectations, constructs bytes, and verifies the saved literals and SHA-256 values. Changing expectations requires a reviewable fixture revision.

The proposal has no BLS implementation, public-key registry, PoP verifier, native signature subject, or certificate serialization. Synthetic key-order labels in the identity vectors are not valid BLS keys. Certificate vectors specify obligations for task 04; their actual cryptographic outcome remains INCONCLUSIVE until genuine public fixtures and the agreed verifier are available.
