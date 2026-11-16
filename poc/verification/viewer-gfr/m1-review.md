# Independent M1 review — PASS

Candidate: `e41b81a8d68d67216ea940beeb614135e808e95d`. Reviewer: `bft_verifier`, GPT-6 Astra / xhigh. Contract and expectations were frozen before candidate inspection. Candidate files were read-only throughout review; only the verifier-owned harness lockfile and review/run artifacts were added. No commits were made by the verifier.

**Verdict: PASS for M1 codecs and capability interfaces. No blocking findings.** The independent executable returned **167 passed, 0 failed**, exit 0, including the aggregate capacity case. The lead was informed that M2 implementation may proceed; this result does not certify M2 behavior or complete feature 07.

## Static source review

| Rule | Candidate locations | Review conclusion |
| --- | --- | --- |
| v10 09 §3.3 and contract GFR-POS-1: public key, joint Force/Resume control class, class index | `poc/crates/viewer-gfr/src/codec.rs:34` | Typed index validation precedes comparison; mixed controls compare by L1 position. Conflicting control kinds at one L1 position reject. Wire tags do not determine class order. |
| Contract GFR-POS-1 exact BE encoding and strict decode | `poc/crates/viewer-gfr/src/codec.rs:56`, `:86` | Existing 03 types preserved; fixed-width fields/version/tags match independent bytes. Truncation, unknown tags and trailing bytes reject. |
| v10 09 §4.7 full lifecycle identity; contract GFR-LIFE-1 | `poc/crates/viewer-gfr/src/codec.rs:124`, `:164`, `:187` | Complete position and body enter projection and domain-framed identity hash; canonical nested 03 fields retained. Different same-pc bodies remain distinct. DEP/transaction validation and count/byte bounds enforced. |
| Contract GFR-PUBLIC-1; v10 09 §3.2 proof/body distinction | `poc/crates/viewer-gfr/src/codec.rs:194`, `:237`, `:314` | Exact canonical 03 payload is wrapped without alternate object encoding; proof is opaque. Message/frame bounds checked before copied proof bytes; malformed inner versions reject. No proof verification is claimed by decoding. |
| Contract explicit verifier prerequisites and provenance | `poc/crates/viewer-gfr/src/api.rs:136`, `:167`, `:171` | Separate profile, readiness capability and native bool checker are exposed. M1 has no reducer call site, so WAIT-before-check/profile matching behavior is reserved for M2 tests. Native label validation alone is not native acceptance. |
| Existing 03/06 capability reuse | `poc/crates/viewer-gfr/src/api.rs:108`, `:121`, `:177` | Actual 06 FinalizedLogReader and 03 Plan/metadata/DEP types used; no second domain or clock introduced. State-machine routines are not falsely stubbed as successful. |

## Replay evidence

Evidence directory: `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07/artifacts/verification/m1-e41b81a-independent-v1`.

Invocation:

```sh
bash poc/verification/viewer-gfr/independent-harness/run-m1-review.sh
```

The saved `command.sh` captures the exact environment and commands: Rust 1.95.0, task `.build/cargo` and `.build/target`, `CARGO_BUILD_JOBS=2`, one `flock /tmp/megaroation-poc-heavy.lock`, independent-workspace `cargo generate-lockfile --offline`, then `cargo run --locked --offline -- --capacity`. Tool versions were `rustc 1.95.0 (59807616e 2026-04-14)` and `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`. The run is deterministic literal-driven and has no random seed. Candidate source diff at completion was empty.

`harness.jsonl` records expected and observed values for all 167 checks. The suite covers 9 positive/29 negative position vectors, 9 comparison pairs, 18 lifecycle vectors, 2 positive/11 negative publication vectors, systematic truncated prefixes, unknown tags, invalid typed position/index pairs, repeat identity, nested 03 version rejection, proof limit 32768/32769, Resumed prefix limit 4096/4097, malformed DEP/ordinary transactions, oversized message and aggregate lifecycle bounds. The aggregate fixture independently projects to 67,130,205 bytes; candidate correctly rejected it against 67,108,864 bytes without a panic.

`m1-result.json` records raw evidence hashes, candidate, command, exact truncated-prefix count, scope and remaining gates. `input-hashes.txt`, `candidate.txt`, `candidate-diff.txt`, `status-before.txt`, `status-after.txt`, `lock-hash.txt`, `harness.stderr` and `exit-code.txt` preserve provenance and build/run evidence. New independent Cargo.lock is a harness dependency artifact, not a candidate source change.

## Coverage and limits

Contract V01 is checked for the implemented M1 codecs. Frozen VG04 ordering and VG33 same-position lifecycle identity are checked at the value-interface level, not by a running reducer. Native gate VG40/V22 remains OPEN. VG39 future RESUME material-before-allocation remains an explicit deferred integration gate. Other VG/V state-transition cases are not asserted passed by this M1 review and remain for M2.

The inherited 03 Descriptor golden is encoding-only, not a cryptographically valid registry. Resumed values here test a comparison projection, not actual RESUME acceptance or a validated derived chain. Typed capability declarations are present, but their event-atomic invocation, WAIT behavior and checkpoint/profile enforcement require M2 execution. No G1/G2, real controls, 15-worker behavior, restart safety, official Espresso finality, eventual progress or protocol safety theorem is established by these finite traces.

The coordinator's restart-free handoff directive was read: full 01 recovery is not a prerequisite for this pure component. A future healthy native 04 profile can close the certificate integration gate only after delivery and independent review; none was supplied in this run.
