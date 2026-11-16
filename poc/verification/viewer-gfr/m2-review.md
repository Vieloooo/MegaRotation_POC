# Independent M2 candidate review — PASS

Reviewed candidate **`de628c2e3536c6d7b10ae2f8513b7014623a0e45`**, by `bft_verifier` (GPT-6 Astra / xhigh), 2026-11-16. Expectations and executable harness were committed before reducer inspection. Candidate files and frozen harness source remained unchanged during review. The independent workspace lockfile gained three explicit dependency names already present transitively; no dependency version changed. The verifier made no commit.

**No blocking findings. Independent harness: 117 passed, 0 failed, exit 0.** This is approval of the candidate for the agreed Tester matrix, not completion of full feature 07. No harness/API fix was needed. The only compiler warning was an unused verifier fixture helper; it does not affect the evidence.

## Reviewed rules and implementation

| Source / accepted rule | Candidate location | Checked behavior |
| --- | --- | --- |
| v10 09 §§3.3–3.4; GFR-07-v1 §6 | `poc/crates/viewer-gfr/src/frontier.rs:100`, `:198`, `:427` | Public key/class/index order, mixed controls, complete watermark and ceiling-rounded L1 control coverage, explicit unknown frontier; same-key unreceived q cannot falsely satisfy a later bound. |
| v10 09 §§3.4,4.8 | `poc/crates/viewer-gfr/src/frontier.rs:28`, `:56`, `:281` | Existing supporting prefixes cannot regress or change. Header and control frontiers remain separate; later metadata headers do not demand later control lists. Supporting watermark is retained without early publication indexing. |
| v10 09 §§4.1,4.8; whole-event atomicity | `poc/crates/viewer-gfr/src/replay.rs:263`, `:337`, `:377` | Clone/stage one event; a later Plan/metadata WAIT discards all its appends, publication updates, handoffs and effects. Earlier committed events remain. Missing input returns the determined original pc when ordering is established. |
| v10 09 §3.2 and §4.2 | `poc/crates/viewer-gfr/src/publications.rs:80`, `:149`, `:256`, `:318` | Original public object positions, target proof/body ancestry, earliest q_avail, candidate caching, canonical target order, separate proof readiness, invalid proof exclusion, descendant coverage and conflict-premise rejection. |
| v10 09 §4.2; v10 00 §§3.3–3.4 | `poc/crates/viewer-gfr/src/publications.rs:228`, `:375`; `src/replay.rs:416` | Actual 03 metadata/Merge/native execution; exact selected anchor and already accepted prefix; public deadline uses q_avail, every new virtual height uses current acceptance key, including padding. |
| v10 09 §§4.3,4.5 | `poc/crates/viewer-gfr/src/replay.rs:502`, `:559` | Full-window and SessionEnd regular paths; production anchor versus initial reference; fixed successor window; multiple same-event handoffs; Expire/HALT preserves G and closes once. |
| Dispatch Force/RESUME scope | `poc/crates/viewer-gfr/src/replay.rs:307` | First unsupported control stops at its exact position; no skip, partial branch mutation or HALT wakeup. |
| v10 09 §§4.7–4.8 and accepted checkpoint clarifications | `poc/crates/viewer-gfr/src/replay.rs:158`; `src/state.rs:43`, `:84` | Full public-state and supporting-cut replay comparison; immutable typed genesis/profile/parameter binding; missing reads WAIT; 23 public-field/cut tamper variants reject. Restore rebuilds cursor/anchors and continues once. |
| Frozen verifier capability rule | `poc/crates/viewer-gfr/src/replay.rs:208`; `src/publications.rs:269` | Profile remains bound without capability; mismatch rejects; basis WAIT occurs before bool checker. The native label does not establish cryptographic acceptance. |
| v2 independent readers and time separation | `poc/crates/viewer-gfr/src/reader.rs:13`; `examples/replay/viewer.rs:12`, `:106`, `:166` | Per-viewer actual 06 ReaderState, delivered 03 L1 reader and ReplayState. Wall sleep affects harness pacing only; explicit delivered ticks/public theta drive inputs. Rules contain no wall/local-clock trigger or uncontrolled output side effect. |

The M1 codec source is byte-for-byte unchanged from reviewed `e41b81a8d68d67216ea940beeb614135e808e95d`; its API file only adds the approved HistoricalBound error variant. Existing M1 exact vectors remain applicable. No M2 implementation success is inferred from matching A/B outputs alone: the independent harness checks literal transition guards and a hand-derived account/Kept/native-content oracle.

## Reproducible evidence

Absolute evidence directory:

`/mnt/reth/jy/megaroation_poc/.build/commander-jobs/07/artifacts/verification/m2-de628c2-independent-v1`

```sh
bash poc/verification/viewer-gfr/independent-harness/run-m2-de628c2-review.sh
```

The saved command fixes Rust 1.95.0, task `.build/cargo` and `.build/target`, two build jobs and one host `flock /tmp/megaroation-poc-heavy.lock`; it generates only the independent lockfile offline, then runs `cargo run --bin m2 --locked --offline`. This harness uses fixed fixtures and has no random seed. Raw command, compiler versions, input hashes, stdout JSONL, stderr, exit code, lock hash and before/after status are retained. `candidate-diff.txt` is empty. `m2-result-de628c2.json` records their hashes, every rule mapping and source identities for the final integrated-revision comparison.

The 117 checks include source-derived native content at heights1..3,9,11,20,31; first-public versus acceptance positions; proof/body/public versus local missing data; readiness call counters; exact profile binding; full-event rollback; strict cutoff/expiry equality; fixed-window regular; synthetic unsupported controls; nonextending source rejection; 23 checkpoint tamper cases; watermark/message separation; historical bound rejection; finite DEP exhaustion/overflow; canonical target sorting; and independent actual 06 A/B readers with opposite valid-proof arrival order.

## Scope and unresolved gates

The checked public histories are finite. L1 uses actual 03 finite mock rows0..30 with explicit finalization/delivery. DEP is the documented verifier-generated configuration-bound range1..40, plus an exhaustion variant ending at3. Proofs match exact descriptor/body/proof fixtures and are **not cryptographic certificates**. Synthetic nonempty controls are **not Anvil receipts**. The two-handoffs-at-one-pc case explicitly uses non-live public timestamp lag and supports no E-L/E-TS timing claim.

The full VG/V mapping is in the result JSON. V01 is carried from unchanged M1 codec evidence; VG40 passes only the unavailable-capability boundary. VG35 has repeated-call evidence but its wall-pause matrix remains for Tester. V22 actual native certificates and VG39 future RESUME material-before-allocation are **INCONCLUSIVE/deferred**, never counted as native/RESUME acceptance.

Remaining gates are the agreed G1/G2 seeds7/42/99 × pauses0/2ms Tester run on the final exact integrated revision; 04 native FinalVerifier/certificate vectors; actual 09 controls; 12/13 position-bound ResumeCut and lifecycle material staging; and controlled 15-worker regular integration. Typed in-memory checkpoint conformance is not disk durability or OS-restart evidence. These finite traces establish no general safety or liveness theorem and do not mark full 07 accepted.
