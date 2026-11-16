# Independent M1 harness

Prepared from frozen GFR-07-v1 contract and `interface-vectors-v1.json` before inspecting builder code. The input literals are frozen; candidate code supplies observed results only. This workspace owns its own lockfile and never modifies the candidate or shared crates.

The checker covers exact positive/negative codec and comparator vectors, all truncations of valid envelopes, typed position mismatches, unknown tags, lifecycle identities and retries, malformed DEP/transactions, 4096/4097-block prefix limits, and 32768/32769-byte proof limits. `--capacity` additionally checks the aggregate 64 MiB lifecycle limit with a large, independently sized valid input. The separate manifest resolves only after a named M1 candidate exists; no build was run during preparation.

Run from the worktree root using the assigned `.build` cache and target, Rust 1.95, two build jobs and `flock /tmp/megaroation-poc-heavy.lock`. The symlink target lies outside sandbox writable roots, so execution needs the already-authorized escalation. Generate the local lockfile offline once at the named revision; subsequent runs use `--locked --offline`.

```sh
cargo run --manifest-path poc/verification/viewer-gfr/independent-harness/Cargo.toml --locked --offline -- --capacity
```

Capture stdout/stderr, command, candidate SHA, dirty paths, compiler version and fixture hashes under the task `.build/artifacts/` directory. Each JSONL line records expected and observed results. An encoding-only inherited Descriptor fixture is not a verified registry. PASS here is limited to M1 interface conformance; native verification, state transitions and actual RESUME remain untested.
