# NATIVE-WITNESS — bounded independent acceptance PASS

Corrected implementation candidate: `050c59b9e446e0aaf295f24e3d3a15357b00c443`. The previous add79ab candidate was rejected; its review and raw counterexamples remain preserved.

The capped full regression completed with102 PASS and0 failures:82 prior/builder tests plus20 independent cases. The original stale-barrier, interior-substring and recovery-authorization failures are closed at their intended invariants. Additional cases check ordinary overwrite/version invalidation, I/O-ID uniqueness, tagged lost-Sync/start_sync responses, persistent uncertainty, and generation2/3 intrinsic authorization history without external audit snapshots.

Recovery fixture counts were updated to distinguish historical witnesses retained in the intrinsic chain from newly established generation witnesses; exact old record/byte-prefix preservation and unchanged duplicate-authorization head are also asserted. Rejection/idempotence expectations were not weakened.

Evidence: `poc/verification/native-witness/REVIEW-050c59b.md`, `candidate-050c59b-report.json` and the raw copy at `poc/artifacts/verification/native-witness/050c59b-accepted`. Earlier failures are at `REVIEW-add79ab.md` and `poc/artifacts/verification/native-witness/add79ab-rejected`.

The gate establishes the recorded bounded native-journal evidence and durable authorization behavior. A sync-response-loss hook is an explicit injected response fault after native success, not an observed OS disk failure. It does not prove power-loss durability. Native recovery/timing with auxiliary callback state, guarded signing, p2p, real engines and native two-process replay remain pending; full I1b/I1 has not passed.
