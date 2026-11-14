# I1a gate result — PASS within the recorded fixture bounds

Verified integrated revision: `35112ac2188287269d6faefb567466687beefeed` (I1a implementation `3d413c4342b56250bb69a3d822a5693e84c1c2d7`). The task contract and interface annex remain the acceptance authority.

Independent verification passed 13 pure cases and 10 actual loopback/process controls. All five source findings were repaired and independently regressed. The checker rejected six mutations, including three with recomputed state, Ack, journal and committed-head hashes. `./poc/scripts/bootstrap.sh` passed on the integrated revision; socket checks are separately run because they require loopback access.

The simulator replayed eight approved scenarios at seeds 7, 42 and 99, each with baseline and 2 ms wall-pause/reversed-start execution. All 48 runs completed and passed the independent checker. All 24 pairs have byte-identical full virtual traces, unchanged config/run IDs, and unchanged source and executable hashes. There were no timeouts, failures or omitted cases. The matrix includes 14,142 trace rows and 2,496 committed steps.

Evidence: `poc/artifacts/simulation/i1a/35112ac2188287269d6faefb567466687beefeed/matrix.json` (SHA-256 `ee90fde08df00f4ae8230365b2b952efdcf8ccd9595241f79080202e3ef84e40`), with all raw per-run data and collector alongside it. Independent controls and pre-replay verification are under `poc/artifacts/verification/i1/`; source-derived findings and checks are in `poc/verification/i1/verification-report.json`.

The collector used 106.055 seconds of **local wall time**, including checker work. Virtual runs began at 1,000,000 microseconds and ended at scenario-dependent ticks from 1,102,251 to 1,212,250. These are loopback-controlled simulations, not real-network or AWS measurements.

This PASS covers two real worker processes and the **SignedIntent fixture**. It does not establish native BLS/Simplex behavior, complete Commonware runtime integration, 15-node operation, handoffs or protocol proofs. Overall I1 remains pending I1b. Proceed with the frozen native runtime integration contract; do not substitute I1a for the native gate.
