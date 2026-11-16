# Independent slice A harness — prepared, not run

Written against frozen H3-A-1 and verifier source/wire vectors, without reading the in-flight Builder implementation. **NOT_RUN / not compiled until a named candidate is delivered.** The harness uses real03 registry/PoP, Plan, metadata/Merge inputs, real06 file Publisher/ReaderState/SimEspresso, and the declared07/H3 API.

All final proofs and historical-context callbacks are explicitly synthetic test capabilities. The proof callback accepts only registered exact descriptor/block/proof tuples. The Native profile tests production routing/profile policy, not BLS validity. No test here closes the actual H1 cryptographic or healthy15-worker gate.

`fixture-provenance.json` records copied accepted03/07 fixtures from baseline a1bee707. No production algorithms were copied. Existing seven-transfer fixture helpers are retained for provenance, but these tests use zero or one transfer within the A profile. Expected public positions, phase transitions and codec bytes come from the verifier's frozen vectors, not candidate output.

The accepted07 dependency lock is retained as `Cargo.lock.preparation-seed`. Resolve this workspace's actual `Cargo.lock` with the named candidate's manifest **after** candidate delivery; doing so now would inspect an in-flight production manifest. Then run the locked test under the global heavy lock and task07 external target/artifact settings. Record exact candidate SHA/dirty state, tool/lock hashes and stdout. No build was performed merely to create these files.

```sh
cargo generate-lockfile --offline --manifest-path poc/verification/publication-gfr-healthy/slice-a/independent-harness/Cargo.toml
flock /tmp/megaroation-poc-heavy.lock cargo test --locked --offline --manifest-path poc/verification/publication-gfr-healthy/slice-a/independent-harness/Cargo.toml -- --test-threads=1
```

Use Rust1.95.0, `CARGO_BUILD_JOBS=2`, and the approved task07 data-disk cargo/target location. Actual commands/environment and any required external-path escalation are recorded at execution, not fabricated here. Existing07 M1/M2 independent regression should run on the same final candidate after extraction.
