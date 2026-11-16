# Independent B harness

Expected behavior comes from frozen H3-B-1, the source-first32-case preparation, B identity/parameter/descriptor vectors and the dispatch's sole-committer decision. Test adapters were completed from the communicated/public signatures; B implementation behavior was not read to derive assertions. **Prepared, not built or run until its named candidate.**

The harness uses actual03 registry/PoP/Plan/execution fixtures, sealed07 replay records,06 file Publisher/SimEspresso/readers, and02 ClockTracker, RouteKernel and ReliableMailbox algorithms. Finality/context capabilities and owner receipts are explicitly synthetic. The H3 receipt ports recognize only registered hashes. The route fixture registers deliveries only after the actual02 route and mailbox transitions; deliberate mutated Delivery structs are negative inputs, not production evidence.

The existing A-owned fixture module is reused read-only. Source snapshots/expected outputs remain distinct from Builder code. This workspace owns its Cargo.lock, resolved offline only for the named candidate. Heavy commands use task07 Cargo/target data, Rust1.95.0,2 jobs and `/tmp/megaroation-poc-heavy.lock`; raw output goes directly into a new task-data artifact directory.

Prepared tests cover full/empty/same-position records, exact parameters/source/profile/config, registered receipt/stale/cross-instance/restore guards, close before Offer, forged same-key scopes, verified deadline and source/reader binding, pure fragment goldens/bounds, actual02-delivered reassembly and original-prefix WAIT, immutable custody/split-q scheduling/repeated finalization, and credit charge/release/backpressure.

No private H3 RecordID is treated as H1 LocalId or an actual root receipt. H1's typed API candidate `f9364a2a8e59913f7f9373fab2774be4ed528e57` was read with `git show`; its ReplayBatch uses full07 record, EvidenceRef, ControlledOrigin and a separately derived LocalId. The actual NativeVerifierPort runtime, genuine certificates, H1 root/aux durability, H2 action execution and15-worker M1 remain open. No H1 type or implementation is copied/merged here.

An unknown deadline is tested using a Verifying task whose historical context is missing and whose verified_deadline is None. A known verified deadline is not made invalid merely because another piece of result material is still pending. Backward-record tests require monotonic exclusion; a local implementation may reject an out-of-order record, but cannot reopen an already committed stop.
