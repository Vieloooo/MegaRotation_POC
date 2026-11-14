# Native runtime slice — finite acceptance PASS

Implementation candidate: `3ab4549e48bf6e3e8fe49beed828430c974dba5e`. This is the native task/clock/RNG slice specified by `contracts/i1b-runtime-slice-v1.md`, not full I1b.

The independent verifier executed 27 tests twice, at 0 and 2 ms injected wall pauses, under an external 30-second process cap. All 54 invocations passed; all 80 saved complete logical JSON artifacts were byte-identical. Internal replay covers seeds7/42/99, 300 ready Tokio channel receives (cooperative-budget pressure), 100 duplicate wakes, clock and RNG vectors. Actual native Runner return is checked after cancellation and panic cleanup.

The reentrant control/failure cleanup and same-cid Sleep transfer findings have independent passing regressions. The suite checks lazy construction, unawaited panics, future/result destructors, supervision, repeated failure handling, operation bounds, native Metrics, virtual Stopper/timeout behavior, signed deadline ceiling, atomic sample rejection and per-role RNG continuity. No open source findings remain for the recorded slice. The opt-in native patch leaves ordinary Spawner and its suffix unchanged; vendor modifications are restricted to Cargo.toml and tokio/runtime.rs.

Source and exact commands/outcomes: `poc/verification/i1b-runtime/verification-report.json` and `execution.json`. Raw final evidence is copied to `poc/artifacts/verification/i1b-runtime/3ab4549/{pause0,pause2}`. An earlier stale-binary attempt after a fixture compile error is explicitly excluded from accepted evidence in `execution-stale-build-rejected.json`.

Full I1b and overall I1 remain pending controlled Storage/Blob, native transport/quota, signing, actual engines and two-process integration. Terminal state is in-memory in this slice. The external test cap protects the test process; it is not a claim that arbitrary nonreturning Rust code can be preempted or cleaned up by the driver.
