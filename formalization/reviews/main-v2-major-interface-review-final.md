# Final interface review closure

File: `formalization/main_v2.tex`.

SHA-256: `c0340188273735d89f094e39599129b00f95d1d6c34ce4346bb62667477a1fe4`.

**Verdict:** no remaining findings within this interface-and-scope review. The three issues in `main-v2-major-interface-review.md` are resolved in the inspected revision.

Verified corrections:

- Section 2 explicitly identifies mini-block values by header and transaction bytes; alternative valid signature/receipt encodings are evidence for the same value. Prefix comparability therefore concerns semantic entries.
- The SMR/application paragraph now specifies that correct submissions remain available until confirmation, aligning the definition with the persistent-input condition used in the liveness argument.
- Section 3 defines the ORDER comparison domain as the same session, producing view, and mini-block height. Lawful re-proposal in another view is excluded from this predicate.

The further revisions are consistent with this contract: fresh batches contain a nonempty mini-block range (an idle mini-block can contain zero transactions), NEWVIEW validates inherited data and an earlier-view certificate with an explicit initial-checkpoint exception, and the latency table labels its recovery bound as handover to an honest successor.

The six-section organization follows the approved outline. Its public stream contract hides BFT-stage details, the batch-content command remains distinct from its consensus placement, and the post-commit receipt correctly realizes the public confirmation semantics. Request-level liveness is explicitly conditional on the stated input-service premise; permissionless admission is delegated to L1 while the proof covers one activated snapshot. Cross-region CFT and global finality remain open.

This closure verifies the interface, scope and prior local findings. It does not replace the separate core-BFT proof review or claim a dynamic-membership/cross-region theorem. No manuscript edits were made by this reviewer.
