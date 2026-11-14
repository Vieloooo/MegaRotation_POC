# xuelab intake / preflight — 2026-11-15

This report continues the 2026-11-14 intake; see the adjacent JSON for machine-readable observations. No PoC implementation test passed during intake.

- Explicit working directory: `/mnt/reth/jy/regional_BFT_sim/poc/sequencer_rotation_v2`.
- Clean receiving snapshot: `8086ce146776b88ea917986a521b0587765e1de2`. New implementation branch: `codex/poc-v2-i1`.
- Source clock-skew HEAD: `476ee764ebd38f820f7aab7d20100436d45c140c`, plus supplied working modifications. It is distinct from the receiving snapshot and parent BFT revision.
- All **1341** `handoff_provenance/SHA256SUMS` entries passed. v2 SHA-256: `95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab`.
- Parent BFT HEAD remains `2b508adb9dec368f8a22021129361d22b746ffce`. Its modified README and untracked examples/visualization remain user-owned and read-only.
- Configuration requests GPT-6 Astra / ultra. Runtime model/effort metadata is not exposed; neither TOML nor the receipt proves actual Ultra execution. Named role tools explicitly provide Astra/xhigh for spec/verifier and Sol/high for builder/simulator. Seven total concurrency slots are exposed.
- Linux x86_64, 48 logical CPUs, ~125 GiB RAM, ~1.56 TB free on the working filesystem at intake. Limit Cargo jobs to two; only one build/heavy run at once.
- Direct compiler/cache probe confirmed Rust/Cargo 1.95.0. Noninteractive PATH lacks their shims; local bootstrap selects the installed compiler directly. Parent registry was copied to `.build/cargo` so lock/cache writes remain isolated.
- Node 22.19.0, npm 11.8.0, Python 3.10.12 and Git 2.34.1 available. Foundry/Anvil/protoc missing from PATH. Tonic/protoc will be pinned and downloaded locally. No global tool installation or proxy change.
- Docker client 28.4.0 exists; daemon remains permission denied outside the sandbox. I1 uses direct children and needs no Docker.
- Sandbox `ss` could not inspect sockets. Runtime must bind an available loopback port and record it; never stop another service to acquire a port.

## Frozen workflow

Spec and verifier independently inspected v2/v10 and pinned Commonware 2026.10.0 (`92a2f68a0e131c8b69de41e84aec37da6280f212`) before implementation. Lead owns schema/dependencies/integration; one builder owns runtime/hub/worker. Verifier then checks a named candidate; simulator replays that exact verified revision. Two real-process fixture I1a evidence and actual Commonware runtime I1b evidence are reported separately; overall I1 requires both before three-region handoff claims.

Artifacts: `../contracts/task-contract-i1-v1.md`, `../contracts/i1-interface-annex-v1.md`, `../contracts/i1-independent-expectations-v1.md`.
