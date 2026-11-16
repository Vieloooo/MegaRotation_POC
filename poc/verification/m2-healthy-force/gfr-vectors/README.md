# M2 Healthy Force public replay vectors

The suite calls the production GFR initialization, advance and checkpoint-restore entry points. The independent fixture supplies the public input schedule, exact-subject regional witness and execution oracle. The M2 smoke and L1 tests supply the real process, signature and transaction evidence.

| Contract | Test |
| --- | --- |
| HF04 | Both NEXT contribution cases, inherited L1 floor, and a Force from a regular PENDING boundary |
| HF05 | Force invalidates the old SessionEnd and rejects a valid historical h10 witness first public at the equal key |
| HF06 | Expire for the still-missing h2 closes the new branch after an equal-key Force |
| HF07 | Sorted same-block controls by transaction and local log, branch revalidation, repeated delivery, identical replay after checkpoint restore |
| HF10 | Opposite L1/Espresso delivery order, missing control coverage, missing Force Plan input, independent readers agree |
| HF11 | Stale target while NEXT and current target after HALT |
| HF14 | SessionEnd selects h9, leaves h10 absent, then h11 admission appends h10 as padding and h11 as physical in one event |

The fixture derivation is recorded in [expectations.md](expectations.md). Tests print replay positions and the observed Force assignments or WAIT reasons. No simulated wall-clock or RNG inputs apply.

From the clean M2 candidate checkout, with the M2 Cargo cache and target directory already provisioned:

```sh
M2_JOB=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m2-healthy-force
M2_RUST=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin
env PATH="$M2_RUST:$PATH" RUSTC="$M2_RUST/rustc" RUSTDOC="$M2_RUST/rustdoc" \
  CARGO_HOME="$M2_JOB/cargo" CARGO_TARGET_DIR="$M2_JOB/target" CARGO_BUILD_JOBS=2 \
  flock -w 900 /tmp/megaroation-poc-heavy.lock "$M2_RUST/cargo" test --locked --offline \
  --manifest-path poc/verification/m2-healthy-force/gfr-vectors/Cargo.toml \
  -- --nocapture --test-threads=1
```
