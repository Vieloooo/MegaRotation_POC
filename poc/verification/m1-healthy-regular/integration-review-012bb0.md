# M1 narrow H3 Slice A integration intake — 012bb0

**Source identity / composition verdict: ACCEPT.** The integrated candidate is exactly **`012bb0b7d9efe5bd22ec544321eed8c12219b9d8`**, with parents `f9220e7811b42fe137a96f93143b38246d201565` and H3 delivery `f0ad79e33933f23eb527af62c6bc5096a77844fb`.

This approves the affected replay matrix below. **Do not seal this integration stage until those checks pass on this exact SHA. M1 remains NOT_RUN.** This review ran no Rust build, heavy simulation, worker or official node, and made no repository edits. HEAD matched the named candidate and Git status was empty.

Machine-readable comparisons, changed paths, hashes and original recorded commands are in `/tmp/m1-integration-evidence-012bb0.json`.

## Source-first scope

Before reviewing the integrated delta, I reused M1's frozen E07/E08/E13 expectations (public completeness versus local delivery, public event ordering/admission, and actual-result/readback provenance), then read H3-A-1 and H3's pre-code `source-expectations-v1.md`.

The consumable slice is one extracted GFR/Publisher completeness evaluator, a bounded private result store/canonical public-message adapter, and a Native-profile dependency-injection boundary. Complete public evidence remains separate from selection, deadline/Expire checks and G admission. Full Offer still requires context/anchor/execution validation. Actual H1 final-proof and context-provider correctness remain separate gates.

The owner review/report explicitly labels finality/context callbacks **SYNTHETIC_NATIVE_BOUNDARY_NOT_CRYPTO**: they match an exact descriptor/block/proof tuple. Actual03 registry/PoP/Merge use does not turn those synthetic final proofs into native H1 certificate evidence. No whole-protocol review was repeated here, and no Native profile tag was treated as cryptographic proof.

## Exact content and history checks

`f9220e..012bb0` contains **66 changed files: 62 additions and four modifications**, all in H3-authorized publication-bridge/viewer-gfr or H3 docs/verification paths. The only existing files modified are:

- `poc/crates/viewer-gfr/examples/replay/viewer.rs`
- `poc/crates/viewer-gfr/src/lib.rs`
- `poc/crates/viewer-gfr/src/publications.rs`
- `poc/crates/viewer-gfr/src/replay.rs`

| Production object | Actual source/tested e59d026 | Delivery f0ad79e | Integrated 012bb0 |
| --- | --- | --- | --- |
| `poc/crates/publication-bridge` | `ed28e3518d238dbb9f6193a4eb0a7324fa4a64d0` | identical | identical |
| `poc/crates/viewer-gfr` | `a75b17bbc41c675550389a71d5e0a7768694c333` | identical | identical |

Actual source/tested SHA is `e59d026e8c08b64ff744b57c63cc7756f0b34019`; it is an ancestor of delivery and integration. There was no merge-local production change. Release additions include reports, the executed independent lock, and B/C frozen documents/vectors/fixtures. **B/C production implementation is not part of this delivery.** The official compatibility implementation and other shared modules were not modified.

The following complete Git objects are unchanged from `f9220e`: H2 crate and whole H2 verification tree; types/core/authorization/Publisher/SimEspresso; root PoC manifest/lock; proto; Regional-BFT gitlink; and original07 verification tree. The BFT gitlink remains `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`.

M1 harness README, evidence.py, run.py, check.py and mechanical_tests.py have identical blob IDs and SHA-256 at verified `1ade576`, `f9220e` and this integration. All 22 H2 recorded own-input hashes still match. **H2's GFR dependency has changed**, despite its own source/tests remaining identical; its two suites therefore need a composition rerun.

## Received evidence and independent lock provenance

I read the H3 handoff, source-candidate independent review and Tester report, then checked their file identities:

- All **125** original Tester input hashes match the integrated bytes; before/after maps are identical.
- All **22** independent-review production source hashes, **six** contract/expectation hashes and **two** lock hashes match.
- All **13** archived independent-review raw files rehash to their recorded values.
- Original Tester manifest at `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07/artifacts/simulation/h3-a-e59d026-acceptance-v1/manifest.json` matches SHA-256 **`4d32a56fc24e1217b97de73854dbc0f2c65bdc7f7d5ab63bf898d42a6551aa95`**.

The new H3 independent `Cargo.lock` was **not tracked at e59d026**. It was resolved/executed by the owner and captured before delivery. Its SHA-256 is **`27fa993956b33e3afdf8e9647ca1619159e9d8b4486f03a046343a83c5add118`**, matching both independent and Tester evidence and the delivered/integrated file. Release/integration blob ID is `e1d4aeb47d8d70e23ab99df21d57b1b8ee473e4e`. All other files in that independent harness match the source/tested commit exactly. This is a separately evidenced lock addition, not an assertion that the whole harness tree was already tracked identically at e59d026. Reuse it with `--locked`; **do not regenerate it**.

The publication-bridge and independent-harness lock entries retain Commonware2026.10.0. The old07 independent lock is unchanged, SHA-256 `f94c45a60b060f8e261d846dc8c0d115283907cf107679ae67365d69dfc5302d`.

Existing evidence reports 14 independent Slice A tests, 167 codec/capacity vectors, 117 pure-rule vectors, and G1/G2 seed42 × wall pause0/2ms (four finite fixture replays). Those executions remain attributed to e59d026, not to 012bb0. No actual H1 n5/q4 certificate or healthy M1 execution is inferred.

## Minimal affected commands for Simulator

Use clean `012bb0b7d9efe5bd22ec544321eed8c12219b9d8`, fresh assigned data-disk output/cargo/target paths, Rust/Cargo/Rustdoc1.95.0, `CARGO_BUILD_JOBS=2`, and a single shared-lock owner per command. Explicitly set RUSTC/RUSTDOC/PATH to the installed toolchain so the previous rustup-shim issue cannot recur. Record actual argv, source/lock/binary hashes, outputs and before/after revision.

Below `cargo` means `/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/cargo`. Prepend each Cargo invocation with `flock /tmp/megaroation-poc-heavy.lock` exactly once.

```text
# H2 consumer regression: expect 8 and 27 tests respectively.
cargo test --manifest-path poc/crates/session-manager/Cargo.toml --locked --offline -- --test-threads=1
cargo test --manifest-path poc/verification/session-manager-healthy/reducer-v1/independent-harness/Cargo.toml --locked --offline -- --nocapture --test-threads=1

# H3 Slice A: expect 14 independent tests, with the delivered lock unchanged.
cargo test --locked --offline --manifest-path poc/verification/publication-gfr-healthy/slice-a/independent-harness/Cargo.toml -- --test-threads=1

# Original07 regressions: expect 167 codec/capacity and 117 pure-rule PASS cases.
cargo run --locked --offline --manifest-path poc/verification/viewer-gfr/independent-harness/Cargo.toml --bin viewer-gfr-independent-harness -- --capacity
cargo run --locked --offline --manifest-path poc/verification/viewer-gfr/independent-harness/Cargo.toml --bin m2

# Preserve each raw stdout as G1-42-0.jsonl, G1-42-2.jsonl, G2-42-0.jsonl, G2-42-2.jsonl.
cargo run --locked --offline --manifest-path poc/crates/viewer-gfr/Cargo.toml --example replay -- G1 42 0
cargo run --locked --offline --manifest-path poc/crates/viewer-gfr/Cargo.toml --example replay -- G1 42 2
cargo run --locked --offline --manifest-path poc/crates/viewer-gfr/Cargo.toml --example replay -- G2 42 0
cargo run --locked --offline --manifest-path poc/crates/viewer-gfr/Cargo.toml --example replay -- G2 42 2
```

The historical `m2` binary name denotes the existing pure-rule suite, not M2 Force acceptance. No additional source-interface expansion is authorized here.

For each of the four replay outputs, invoke the unchanged original07 `semantic_trace(raw_bytes, scenario, 42)` from `poc/verification/viewer-gfr/simulation/check-traces.py`. Require its checked fields to match the corresponding accepted case in the existing H3 Tester manifest, require full stdout SHA-256 to equal that case's `accepted_raw_trace_sha256` and `stdout_sha256`, and require each pause pair's full stdout bytes to match. The owner recipe also checks the source row and case-specific stderr; retain that check and the actual binary hash. Expected fixture limits are G1: seven common positions, h20/HALT with unsupported RESUME remaining unprocessed; G2: ten common positions, h11/HALT with completeness q5 versus admission q6. These are old noncryptographic rule fixtures, not new system traces.

Do **not** invoke the old H3 tester/run.py or archived run-independent.sh unchanged: they pin e59d026 and its old branch/workdir, and the archived independent recipe contains initial lock generation. Simulator should wrap the commands above for the current exact candidate, keep original07 semantic checking, and write new evidence without changing historical acceptance files.

A fresh lightweight M1 preparation/check at 012bb0 is also appropriate because the captured candidate tree has changed. Use actual current SHA and copied frozen M1 input/expectation files. Expected result stays exit3 / NOT_RUN / NotReady; a component delivery may populate component provenance, but it cannot make H1 runtime, the actual H2 bridge, H3 B/C transport/official ports or the independent production protocol oracle Ready. No full101-case mechanical rerun is requested because its code and inputs are unchanged.

## Stage limits and acceptance condition

The source-identity/composition check is complete; execution at 012bb0 is still pending. Only after H2 two suites, H3 independent14, legacy167/117, four fixture replays with original semantic/hash/pause checks, and applicable new-SHA mechanical preparation checks pass should this integration stage be sealed.

Neither this review nor those component/fixture passes close real final-proof verification, actual historical context provenance, controlled grant/Ack/root custody, H1/H2/H3 runtime wiring, 15-worker regular/SessionEnd production, R2-first-G, real official same-byte publication, B/C, Force, D/RESUME or recovery. M1 is explicitly NOT_RUN throughout this limited intake.
