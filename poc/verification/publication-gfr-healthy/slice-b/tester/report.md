# H3 Slice B acceptance

**PASS** for the bounded H3-B-1 component cases at `d289066082e0b89964e16653360a5c72356142f8` on `codex/h3-publication-gfr`. The locked B harness passed 19/19, the affected locked A harness passed 14/14, and the independent stdlib vector check passed. No final case was incomplete and no command timed out. [Capture manifest](manifest.json)

| Execution | Result | Local wall elapsed |
| --- | --- | --- |
| B independent harness | 9 record/frame, 3 credit, 4 reassembly, 3 service tests passed | 106.283 s |
| A independent regression | 14/14 passed | 77.080 s |
| B stdlib vectors | Record, binding, frame, parameter, and descriptor-scope checks passed | 0.046 s |

The initial locked A invocation exited 101 before running tests because B added a normal `sim-transport` dependency. The authorized offline workspace update added the `sim-transport` package and the bridge's dependency edge to the A harness lockfile. Existing package versions, B's lockfile, both harnesses' test sources, and production sources retained their hashes. The initial and resolved A locks and exact diff are in the capture. This was the sole change among 144 compared inputs. The HEAD SHA was the same before and after. [Runner](run.sh)

## Observed fixture boundaries

- **Ledger and stop.** The closed-session fixture captures q0 at public key `14,000,000 µs`, then SessionEnd for `(β=0,r=0)` at `24,000,000 µs` with two ordered lifecycle members. A registered synthetic receipt installs the close before any task exists. The scoped height-3 result is stopped and the open session's height-11 result remains eligible. The watermark test leaves the task eligible at q0 with public key `23,999,999 µs` and stops at q1 with public key and deadline both `24,000,000 µs` after receipt confirmation. [Record cases](../independent-harness/tests/core.rs) · [Watermark cases](../independent-harness/tests/services.rs)
- **Custody and public receipt.** One immutable three-message attempt publishes ordinals 0, 2, 1 at q0, q1, q2, respectively. The proof appears at `(1,0)`. The missing final body keeps the Publisher at `NotYetPublic` through q1. At q2 the first-complete q is 2, the proof publication remains at q1, and the Publisher transitions to `Recorded`. [Custody cases](../independent-harness/tests/services.rs)
- **Reassembly and credits.** The actual 02 `ClockTracker`, `RouteKernel` and `ReliableMailbox` run with synthetic registered owner ports. The route fixture uses seed 7, initial virtual tick `1,000,000 µs`, and `20 µs` modeled link latency. The 140-byte transport profile exercises reverse arrival, duplicates and q1-before-q0 prefix WAIT. A third partial object for one destination is rejected, preserving the first. Four maximal frames charge `246,092` bytes including headers, and five frames or three recipients reject. Missing ACK preserves charged credit. [Route fixture](../independent-harness/tests/route_fixture.rs) · [Reassembly cases](../independent-harness/tests/reassembly.rs) · [Credit cases](../independent-harness/tests/credits.rs)
- **Independent vectors.** The Python checker recomputes both record projections and IDs, all four binding/digest literals, the exact 86-byte frame, and the seven segmentation boundaries. A 1 MiB object takes 18 chunks with 4,096 bytes in the last chunk. Its framed size is 1,050,070 bytes, so four copies exceed 4 MiB. The checker models the three frozen shorter-bound parameter perturbations with unchanged record ID, and independently derives the four unchanged-key scope attacks against existing 03 canonical goldens. The Rust harness exercises the corresponding rejection paths on actual local objects, using +1 µs perturbations for its three record-parameter cases. [Vector checker](check-vectors.py)

The frozen bounds include 8 pending attempts and 1 MiB custody storage, 2 partial objects per destination, 4 globally, 1,024 chunks per object, and 4,096 retained completion IDs. Shared-queue arithmetic and the synthetic credit-unavailability control cover the checked boundaries. The captured executions use fixed fixtures, with no stochastic matrix or real-network measurement. Local wall durations include Cargo and lock acquisition. Protocol ticks and public keys are recorded separately in the fixture source.

The owner, finality and historical-context capabilities are synthetic. Actual H1 root/aux durability, grant/Ack integration, genuine Native certificates, H2 action application, and healthy 15-worker M1 remain **OPEN**. The older A and C capture manifests retained their recorded hashes. [Frozen contract](../../../../docs/contracts/h3-publication-gfr-slice-b-v1.md)

## Replay details

Run from the verified checkout with Rust 1.95.0 and the task07 Cargo paths recorded in the manifest. Both Cargo test commands use `--locked --offline`, two build jobs, and the shared lock. The recorded lock wait was bounded at 60 s and each locked Cargo command had a 180 s watchdog. With the resolved A harness lock in place, replay the two recorded test commands and the stdlib checker. The original-lock failure and offline setup commands are retained for provenance. [Capture manifest](manifest.json)
