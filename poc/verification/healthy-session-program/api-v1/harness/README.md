# Independent H1 API verifier harness

This standalone workspace depends directly on the named candidate production API and the authoritative03/07 types. It does not copy production helpers or the builder tests. The consumer examples are included from their production paths.

All proofs, keys, Ready tokens, execution references and receipts constructed in tests are synthetic shape fixtures. Counters and test ports demonstrate explicit call boundaries only; they are not a simulated runtime and cannot establish cryptography, execution, durability or consensus progress.

The reviewed candidate is f9364a2a8e59913f7f9373fab2774be4ed528e57. run-checks.sh checks this HEAD and refuses to overwrite its raw artifact directory. Use the exact outer flock command recorded in ../candidate-f9364a2-result.json; it fixes Rust1.95, data02 cache/target and jobs2. No nested flock is used.

A later final-SHA recheck must first bind the unchanged production tree, frozen contract and these harness inputs to the new named revision, then run its explicitly recorded commands. Do not weaken the candidate check or silently relabel the original run.

Original oracle material is ../source-recipes.json and ../goldens.json. Historical source-first files are left untouched. The additional coverage report distinguishes tested shape rules from future owner-ledger/runtime obligations.
