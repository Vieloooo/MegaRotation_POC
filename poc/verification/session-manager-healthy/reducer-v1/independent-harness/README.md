# Independent H2 reducer harness

This path is owned by the independent verifier and is separate from production
files. Semantic expectations were sealed before the reducer contract/implementation
was read. Inputs are hand-written trusted-input fixtures with deliberately
non-cryptographic keys/references; they do not prove native source validation.

The package depends on the named checkout's component and existing O3/O7 types.
O3 helpers create descriptor/config inputs only. Expected phases, selection,
cursor/action changes, safe-failure behavior and counts are manually asserted
from the frozen contract; no reducer invocation computes expected outputs.

No build or test has been run during preparation. Run only after the lead names
the candidate, using the host heavy-task lock, Rust1.95.0, jobs2 and assigned
data-disk target/artifacts. Exact command and candidate/contract/source hashes
must be recorded with later results. Public API getter spellings may be aligned
after the candidate's interface is delivered without changing semantic oracles.
