# Pinned official dev-node capture

Use an isolated worktree on the reviewed feature 16 revision, with Rust 1.95.0
and the locked crate dependencies in Cargo's cache. The runner expects this
worktree's `.build` directory or symlink to resolve to its task data directory.
It uses Python 3.10 or later, Bubblewrap with user/network/PID namespaces, and
the shared host build lock. On xuelab, the existing OCI cache is
`.build/setup/official-20260918`.

To populate a missing cache, run the following from the worktree root. It
fetches official GHCR image contents by digest and extracts only the configured
regular executable, library and SRS files. Existing files must already match
their digest. The optional acquisition command contacts the registry.

```sh
python3 poc/scenarios/espresso-compatibility/prepare-image.py prepare \
  .build/setup/official-20260918
```

The capture runner verifies the cached OCI index, amd64 manifest, source
revision label, required layers and each extracted file offline before launch.

```sh
poc/scenarios/espresso-compatibility/run-local.sh ec16-run-01 REVIEWED_CANDIDATE_SHA
```

The script checks the reviewed commit and reads the feature's files from that
revision. It builds with Rust 1.95.0 under `flock /tmp/megaroation-poc-heavy.lock`
and uses two build jobs. Call the runner without an enclosing acquisition of
the same lock. It supports overriding `ESPRESSO16_RUST_BIN` and `CARGO_HOME`
for a host with different paths.

The local network namespace contains only its loopback interface. The runner
starts the bundled Anvil on port 18775 and the unmodified official image
binary with its matching runtime libraries on ports 18770 through 18774.
Anvil chain ID 31337 is queried before the official dev-node starts. This is
the dedicated L1 used by the dev-node API. Its upstream configuration builder
also starts a bootstrap Anvil before overriding the L1 URL. A run-local
`bin/anvil` wrapper at the front of the sanitized dev-node PATH records the
original invocation, requested port arguments, namespace PID and parent PID,
and Linux process start ticks. It then execs the same verified image Anvil
through the image loader with the original arguments. The wrapper source and
invocations are retained as bundle artifacts. The runner checks the recorded
process identities after waiting for its owned parents and fails the scenario
if a bootstrap process remains. Bootstrap port arguments document the request,
including port zero when upstream asks the OS to select the port.

The HTTP client opens fresh connections directly to the private local endpoints.
Readiness and capture have finite wall deadlines. Cleanup signals the process
groups created by this invocation and waits for their exit.

The first session submits two distinct opaque fixture payloads to the official
node and repeats the exact first payload. It captures genesis through
observed publication inclusion and queries an out-of-bounds transaction in
genesis. The second session fetches the same complete prefix by height and flat
transaction position. Each session preserves its own HTTP status, body and
wall observation metadata.

The runner records the original executable and image hashes, commands,
response bodies, ACKs, full blocks, leaves and all positional responses in
`.build/artifacts/RUN_ID/bundle/`. It preserves the manifest and raw records
from runs that fail after genesis, when those records remain available.
Before genesis, `partial-configuration.json` and `partial-requests.json`
record the available launch metadata and transcript.
After stopping its owned processes, it replays the bundle twice for the first
session with different pauses and separately for the second session. It
compares the exact EC16-AUDIT-1 bytes and projection across runs, and compares
parsed block, leaf and transaction objects across the two sessions. Inspect
`result.json`, `capture.stderr`, the original HTTP records and `offline-*/`
outputs for the result and its limits. The run has no random seed.

The official API is the stated trust basis for the bounded observed prefix.
Proposal time, consensus finalization time and live at proposal remain
`Unknown`. The native compatibility run provides no virtual timing
evidence and the compatibility crate reports unsupported full PE exports.

The pinned API source also defines availability streams for leaves, headers,
blocks, payloads and transactions. This profile records positional queries
and submit requests only. Stream support and reconnection behavior are beyond
the recorded experiment.
