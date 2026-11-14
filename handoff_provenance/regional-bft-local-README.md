# Regional BFT simulation workspace

This workspace compares a 15-member committee using the released Commonware Simplex engine and native Kudzu v2 and Minimmit v5 adapters built from the pinned papers. All backends use the Commonware 2026.10.0 primitives and a seeded virtual network.

The current integration includes native coded delivery and actual shared `BatchVoteVerifier` paths for Simplex5/15, Kudzu6/15 and Minimmit A3/15. Minimmit A reconstructs and validates the body/ancestry before an ordinary or owed M-supplement vote. Existing full profiles remain available. [Feature and ownership handoff](verification/sr-batch-2/handoff.md) · [Minimmit independent review](verification/sr-batch-2/minimmit-coded-implementation-review.md)

The separately frozen shared-verification measurements contain Kudzu42 runs/21 replay pairs, Simplex62/31 and Minimmit64/32. All passed their declared protocol checks; Minimmit's eight BodyWithhold runs still have817 overdue frames and transport bound violated. These per-profile reports retain their original revisions and do not constitute a new same-build three-algorithm performance ranking. Coded + shared verification + limited-bandwidth composition remains a separate gate. [Minimmit measurement](artifacts/sr-batch-2/minimmit-simulation-summary-03e1245.md)

Native coded CLI runs use `run-data <simplex|kudzu|minimmit> coded <scenario> <seed> <new-artifact.json>`, followed by a separate `check` command. This CLI selection alone does not enable shared verification. Preserve the reported transport status: Minimmit BodyWithhold can complete consensus while its whole-trace transport bound is violated.

The [verified local comparison](docs/local-comparison-results.md) contains56 runs,28 exact replay pairs, configuration and measurement rules. The verified code revision is `e674d941555129f6ce91077c5a49b3bec4e326ff`; main revision `8602857` has the identical Git tree. Later documentation and tooling defaults do not change the protocol code. [Independent review](verification/integrated-8891cf8-review.md) · [Machine-readable results](artifacts/comparison/e674d94/summary.json)

## Reproduce the comparison

Run from the project root, using new output paths so existing evidence is retained:

```sh
CARGO_BUILD_JOBS=2 ./scripts/bootstrap.sh test --locked --offline -- --test-threads=1
./scripts/bootstrap.sh run --locked --offline -- run simplex healthy 42 artifacts/replay/simplex-healthy-42.json
./scripts/bootstrap.sh run --locked --offline -- run kudzu healthy 42 artifacts/replay/kudzu-healthy-42.json
./scripts/bootstrap.sh run --locked --offline -- run minimmit healthy 42 artifacts/replay/minimmit-healthy-42.json
./scripts/bootstrap.sh run --locked --offline -- check artifacts/replay/kudzu-healthy-42.json
python3 scripts/run_matrix.py --output artifacts/replay/core42 --seed 42 --repeat 2 \
  --scenario healthy --scenario one-offline --scenario two-offline \
  --scenario leader-offline --scenario partition-delayed
```

The matrix runner executes sequentially and saves the exact command, source and
binary hashes, seeds, raw trace, public-checker output, wall time and replay
comparison. CLI exit0 means complete; exit2 means valid evidence with unfinished
work; exit1 means an error. The collector preserves incomplete cases even though
its aggregate exit is nonzero. A first setup needs network access; omit
`--offline` until the pinned dependencies are cached.

All formal runs use the shared schema2 wrapper. The checker validates native
certificate domains and quorums, full ancestry, observer-local availability and
actual transport receipts. Cryptographic certificate observation, completed
decision, eligible prefix and delivered prefix have separate timestamps. Results
compare the common-third delivery event, and count traffic through that same
event, rather than each runner's later stop poll. Client-submission latency is
unavailable for this on-demand synthetic workload.

Simplex uses quorum11 with f=4; Kudzu uses f=4,p=1, Notar/Final10, First14 and6/15
authenticated coding shards; Minimmit uses f=2, M/nullify5, L13 and the base
full-body profile. Enabled links have10 ms delay, zero jitter and no bandwidth
cap; the controlled executor cycle is1 ns. Exact receiver times include
scheduling overhead and are checked against the declared250 ms bound. Reported
transport bytes include the29-byte experiment envelope and exclude underlying
P2P framing/connection overhead. See the report for native channel compositions.

Each run uses isolated deterministic storage and reproducible test keys.
Counterfactual runs have separate histories; certificates from different runs
must not be combined. Native restart tests cover actor recovery with retained
external endpoints and the approved strict journal policy. Simplex's adapter
has no restart scenario. Lossy recovery is empirical. The historical Kudzu
body-withhold diagnostic reached its15 s cutoff and exceeded the128 MiB formal
artifact limit; its full compressed trace and source recipe are retained in
[the diagnostic directory](artifacts/diagnostics/kudzu-body-withhold/README.md).
These finite checks and virtual timings do not establish production Session
Program integration, a safety proof, steady-state throughput or AWS timing.

## Setup and migration

Copy the project or check it out on the new machine. Run this from the project root:

```sh
./scripts/bootstrap.sh
```

Prerequisites:

| Host | Requirements |
| --- | --- |
| macOS on Apple Silicon or Intel | Bash, curl, `shasum` or `sha256sum`, and the Xcode Command Line Tools (`xcode-select --install` if needed). |
| glibc Linux on ARM64 or x86-64 | Bash, curl, CA certificates, `sha256sum`, a native C/C++ toolchain and make. For Debian/Ubuntu, install `build-essential curl ca-certificates`. If the AWS-LC native build needs its CMake fallback, install `cmake` as well. |
| First setup | Network access to `static.rust-lang.org`, `index.crates.io` and `static.crates.io`, with enough disk space for the Rust toolchain, crate cache and build output. |

The bootstrap and simulation checks were exercised on macOS ARM64. The Linux and Intel macOS rows describe the intended host setup.

The script downloads the pinned Rustup installer and checks its SHA-256 against the checksum published alongside the binary. It keeps Rustup and Cargo caches under `.tooling/` and uses `rust-toolchain.toml` for Rust 1.95.0 and `rustfmt`. `Cargo.toml` and `Cargo.lock` pin Commonware 2026.10.0 and the dependency graph. The script runs the formatting check and tests with the lockfile enforced. It uses the host C toolchain to compile native crate dependencies.

Transfer the project inputs, including `Cargo.lock` and the executable bit on `scripts/bootstrap.sh`. Leave out `.tooling/` and `target/`; the target machine builds its own native dependencies. Set `REGIONAL_BFT_TOOLING_DIR` to an absolute writable directory to reuse a toolchain and crate cache across worktrees or checkouts. Cargo also honors `CARGO_TARGET_DIR` for build output.

## Run a scenario

These are the preserved, explicitly labelled **shared-body smoke** commands.
Use the `run <backend> ...` commands above for networked payload experiments.

```sh
./scripts/bootstrap.sh run --locked -- healthy 42
./scripts/bootstrap.sh run --locked -- leader-offline 42
./scripts/bootstrap.sh run --locked -- partition-heal 42
```

The scenario and unsigned seed are optional, defaulting to `healthy` and `42`. Each successful run prints the committee and active-node counts, elapsed virtual milliseconds, the deterministic runtime audit digest, and the common finalized proposals. The three scenarios check:

- `healthy` finalizes three consecutive proposals at all 15 nodes. The test replays seed 42 and compares the audit digest, virtual duration and finalization records.
- `leader-offline` starts 14 engines in the same 15-member committee and leaves the initial leader offline. Every active node sees a view-1 nullification and subsequent finalizations.
- `partition-heal` starts the network in an 8/7 split. The simulation checks for an empty finalization history before healing and then requires all 15 nodes to finalize a common chain segment.

## What the smoke test measures

The engine exchanges real Simplex votes, certificates and resolver messages through `commonware_p2p::simulated`. It uses Commonware's BLS12-381 `MinSig` multisignatures with deterministic test keys and the upstream mock application and reporter. The mock application shares bodies in memory and uses synthetic application processing delays. Virtual time exercises the integration and scheduling, while the block body path, production key generation and AWS network measurements belong to subsequent work.

The harness uses the public fixtures exercised by [Commonware's Simplex tests at v2026.10.0](https://github.com/commonwarexyz/monorepo/blob/v2026.10.0/consensus/src/simplex/mod.rs). `./scripts/bootstrap.sh test --locked --offline` reruns the checks after the dependencies and toolchain have been cached.

## Codex workflow

Open this folder as its own project in Codex. When you trust the checkout, use Codex's project trust setting so it can load [the project configuration](.codex/config.toml). Transfer `AGENTS.md`, `.codex/`, and `.agents/` along with the Rust sources to another device.

| Stage | Agent | Default model and effort | Instructions |
| --- | --- | --- | --- |
| Coordination and integration | Main chat | GPT-6 Astra · xhigh | [Project workflow](AGENTS.md) |
| Doc | `bft_spec` | GPT-6 Astra · xhigh | [Conformance skill](.agents/skills/regional-bft-conformance/SKILL.md), plus `protocol-spec-writer` when available |
| Implement | `bft_builder` | GPT-6.1 Sol · high | Accepted task contract and locked Rust commands |
| Review and verify | `bft_verifier` | GPT-6 Astra · xhigh | Conformance skill, the paper, and independent acceptance cases |
| Simulation | `bft_simulator` | GPT-6.1 Sol · high | Verified revision, scenario list, and the scenario runner |

The configuration allows six open subagent threads per session in addition to the main chat, following the user's updated limit. The available tool may impose a lower simultaneous limit. Start with a single writer and a verifier preparing cases. Run builds and larger simulations one at a time. With a cached toolchain, set `CARGO_BUILD_JOBS=2` when a build strains laptop memory. Use separate worktrees for concurrent writers once the project has a Git baseline.

If your account exposes different models, update the model and effort fields in the project and agent TOML files. The project skill travels with the checkout, and the personal specification skill is optional. Custom agents inherit the main chat's permissions. The role instructions specify which artifacts each agent owns.

For a first run, ask Codex to use `bft_verifier` to check the existing Simplex partition-heal scenario, then hand the same revision to `bft_simulator` to replay seed 42 and report the command, finalized views, and audit digest. Run formal comparisons only after the relevant backend and its data path pass the checks in `DESIGN.md`.
