# Feature03 independent component verification — e30f5ed

Date: 2026-11-16 (Asia/Shanghai). Reviewer: named `bft_verifier`, GPT-6 Astra / xhigh. Candidate: **`e30f5ed3b3352d58005f672bfebf22e9810552fb`**. Result: **PASS for the complete authorized feature03 component scope; ready for the Tester matrix on the sealed same-code candidate. No open finding remains in that scope.** This is not acceptance of04 native final certificates,02 runtime integration,09 Anvil, or15-worker handoff.

The independent run passed **32 actual Rust tests**:7 M1 public-interface/I1-compatibility,5 production-codec,6 registry/authorization,1 upstream public-key/PoP recipe,13 execution/metadata/Merge/L1 tests. Four Python reference checks also passed. Raw actual component output records three cases: independently progressing readers, the main state-changing Merge vector, and the4096/4097 expansion boundary. No test expected value was copied from the production implementation.

## Sources and expectation independence

- Baseline preparation: `183430e71e5d61533f54a76b2d7d22c5021f1704`; the new dependency plan authorizes independent03 implementation without waiting for all of01.
- M1 contract/bytes were committed at `dae686b7152a4b8dabcc670fa08e9abd88bf276d` before M1 implementation; M2 contract/bytes were committed at `862cc1657cf81afcdcf6548a197b003f47fd979b` before M2 implementation.
- Frozen implementation contract: [v2](../../../docs/contracts/task-contract-mock-plan-execution-v2.md), SHA-256 `a18d2b33f9c541abc81b4fead601c3ccaead6221c90dc43345d52a850429cc15`. It retains v10/v2 semantic authority, Commonware2026.10.0 and Regional BFT pin `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`.
- [M1 expectations](m1-independent-expectations.md), [M2 expectations](m2-independent-expectations.md), fixed JSON byte/state vectors and the independent tests were prepared before reading their corresponding implementation. The historical LE64 profile remains untouched and is not the production codec.
- Real public registry inputs came from the separately run pinned upstream primitive recipe, not `poc_authorization` output. The public record, canonical registry bytes/hash and sorted identity mapping were then frozen independently. Only public keys, PoPs, negative signatures and test seed recipes are stored; no private-key bytes are stored or reported.

## Review and actual coverage

| Area | Evidence and conclusion |
| --- | --- |
| Public M1 compatibility | Private checked u8 identities; exact32-byte opaque values; BE version/field/length guards; L1 position ordering and all reference fields. All root I1 exports remain. Against the preparation baseline, `types/src/lib.rs` only adds `pub mod domain`; protobuf and the old helper bodies are unchanged. Actual fixed Grant/Ack/hash_parts vectors pass. |
| Production canonical objects |40 immutable byte records and exact domain/preimage hashes; no repeated nested version; checked count/byte bounds; sparse state ordering; invalid objects cannot use checked digest methods. The Plan codec example is explicitly not a verified registry and is never promoted to one. |
| Registry/PoP |15 real Ed25519 identities and BLS MinSig vote keys/PoPs; strict canonical, non-small-order, torsion-free Ed gate before native decode; native G1/G2 zero/infinity/width failures; wrong key, namespace, kind, bit mutation and duplicates reject. Native signer mapping follows actual identity-key order, not pid or vote-key order. |
| Static authorization | Three regions, five members and quorum parameters remain fixed. Maximum-u64 schedule and view0 boundary; explicit reference WAIT/REJECT; full Plan re-derivation; authorization/config mutation; regular reference0 versus production anchor9 and successor window11–20; equal-height distinct anchors yield different cids. |
| Apply | Ordered DEP and Kept, successful balance changes, nonce/balance filtering, zero/self/unknown accounts, nonce/receiver overflow skip, credit overflow rejection, malformed suffix rejection and immutable parent state. Config-limited apply is tested separately from the global-limit helper. |
| Metadata/Merge | Actual metadata and clamped fixed origin with missing-input WAIT; actual physical versus native parents; pre-root after padding/before current Begin; exact state/Kept/native hashes; anchor-only empty increment; bounded ancestry, missing body/state, invalid inputs, resource bound and atomic failure. |
| Finalized L1 fixture | Config/source/genesis binding; explicit source Finalize distinct from reader Deliver; actual late finalization cannot be backdated; independent readers; staged out-of-order headers; separate empty-control coverage; duplicates, foreign-source tokens, corruption and exhaustion. This is the shared fixture interface, not an Ethereum finality proof. |

The real registry digest is `82e053c4537f6482bc4b10abcf27882b6abb23bd2f72e012bbb3ded329f285d9`. Signer-index→pid maps are R0=`[4,3,1,0,2]`, R1=`[7,6,5,9,8]`, R2=`[13,11,14,10,12]`. These were derived by sorting the fixed public identity bytes independently and checked against the actual validated registry.

In the main actual Merge trace, h1/h2 are padding and h3 is physical. Final accounts are A=(nonce2,balance90), B=(2,15), C=(0,19); Kept corresponds to raw indices `[1,3,4,6]`. The h3 pre-root is `df9c31e30169fdcf76b8539ec33fa9c400278362d415d8c670d61ee34083758a`, post-root `76bffa42f643077ad9e15c1f7a098c03cfe5e5562b791f7ef932493c83360349`, physical hash `806e5423463b9326eefc8771b2f0f8b6ff953509e714045d3a8e7bd4b3431cac`, native hash `bb6b678ab60e51eb8125a7c3279a6fa3f856d9611c8b7a297376a52a35077d85`.

At the frozen PoC expansion limit,4096 virtual heights execute successfully with independently expected final native hash `b574146396dc53a3e9099193398887106a874121ed46564d98322c90a9146db3`.4097 rejects with `LimitExceeded{limit:4096,actual:4097}` and no partial result. This explicit resource bound is not a production-liveness failure.

## Findings resolved before this verdict

The applicable source rule is frozen v2 §§2,5.3,6: known invalid/conflicting local inputs reject; missing necessary materials wait; no partial result may be committed. Expectations below were independently specified from that rule, not taken from fixes.

| Finding / affected earlier candidate | Minimal input and impact | Resolution checked at e30f5ed |
| --- | --- | --- |
| Known context hidden by missing tip, identified by Builder and independently checked (`81f84f8`) | Valid selected anchor, missing tip body, but invalid config or mismatched provider config reference. The earlier lookup returned WAIT before a known source conflict, delaying rejection indefinitely. | `merge/mod.rs:203` validates context before fetching. The same valid context still returns WAIT for a genuinely missing body. |
| Supplied execution anchor conflict hidden by missing tip, identified during independent review (`7fe0e85`) | Selected anchor A, supplied execution material for B or locally malformed state/native genesis binding, and missing tip. The conflict was checked only after body collection. | `merge/mod.rs:229` / `:258` validate supplied material before fetching; identity, metadata/state and genesis native-hash negatives pass. |
| Known bad fetched body hidden by a missing earlier ancestor, identified by Builder and independently checked | A correctly hash-bound tip is already available but has a wrong fixed timestamp, exceeds the config byte limit, or spans4097; its parent is missing. Full-chain-only checks could return WAIT before known rejection. | `merge/mod.rs:176` is shared by direct Merge and tip reconstruction before the next ancestor lookup. Valid fetched content with a missing parent remains WAIT. Execution-dependent root/DEP checks still wait for their inputs. |

An earlier test harness compile error used a raw byte slice where Commonware requires `Copying`; only the test API adaptation changed, with fixed expected data unchanged. It was not a production failure. The resource cap was already explicit in the frozen contract and required no protocol change.

## Exact run and evidence

Command, CWD `/home/jy/.codex/worktrees/1c43/megaroation_poc`:

```sh
flock /tmp/megaroation-poc-heavy.lock env REGIONAL_BFT_RUST_BIN=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin CARGO_BUILD_JOBS=2 /home/jy/.codex/worktrees/1c43/megaroation_poc/poc/scripts/bootstrap.sh test -p poc-types -p poc-authorization -p poc-core --locked --offline -- --nocapture
```

Rust1.95.0, Python3.12.8, `seed=null` for deterministic vectors; public crypto seeds are303000+pid and303100+pid with ChaCha20Rng. The run took11.378 wall seconds, including ordinary runner/lock overhead; this is not protocol latency. HEAD remained e30f5ed before and after; tracked implementation/test inputs matched the candidate. The manifest records unrelated documentation/reference-file dirtiness separately.

Raw directory:

`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03/artifacts/verification/full03/e30f5ed3b3352d58005f672bfebf22e9810552fb/`

| Artifact | SHA-256 |
| --- | --- |
| `stdout.txt` | `d7840a10069229f20fea20a773c35f85c4d7e4ae5dd1ebda7bed54d9a26f46eb` |
| `stderr.txt` | `5dcdf859cfc2959508ee6c879167b5d323a3ac153dc0e858304cbfa0654ed0df` |
| `component-evidence.json` | `352b86ded10153a683bf743ee30e03dc23ffce5fe6ab52983e93618014a94753` |
| `run-manifest.json` | `cbe83a448a391fac76c0ff8a3f8c357db390cbeb494e4e8b9cb9ca347f4937e8` |

The raw directory also contains the exact public registry emitted by the primitive recipe and stdout/stderr of `check_m1_vectors.py`, `check_m2_vectors.py`, `check_identity_vectors.py` and `check_extended_vectors.py`; all return0. The last checker was a supplementary untracked reference script at e30f5ed, with its fingerprint recorded, and will be included in the final documentation/verification seal. It rechecks fixed public registry framing/order and4096-height arithmetic without importing production.

After the run, at the lead's explicit request, only `registry_fixture_recipe.rs` was passed through pinned rustfmt: import placement and line wrapping changed; literals, seeds, calls and expected data did not. The final seal must confirm that no production difference was introduced and rerun the same component command on its exact SHA before Tester acceptance.

## Remaining integration scope

PASS covers the authorized03 components and their public interfaces. It does not execute or claim04 native Simplex FinalVerifier/quorum certificate matrices,02 controlled transport/grant-Ack/native recovery integration,09 real Anvil receipts/ABI,07 full replay,12/13 historical ResumeCut, or15-worker regular/Force/RESUME scenarios. The FinalVerifier trait has no permissive default implementation; real native certificate adaptation remains04's work. The Tester still runs the agreed whole-workspace matrix and captures the bounded component evidence on the final exact candidate. Finite tests do not establish a protocol proof or production performance.
