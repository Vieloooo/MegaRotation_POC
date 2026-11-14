# Final bounded control review

Date: 2026-09-28.

Reviewed manuscript snapshot: `formalization/main_v3.tex`, SHA-256 `3b6d7add459896616f381de1a8bf98effa1d83424bd79af9efdcf685894bbed4`.

**Verdict: pass for the bounded control review under the manuscript's declared regional-safety, authenticated-control-order and admissible-recovery-cut premises.** All findings from `control-review-v1.md` are resolved. No outstanding correction remains within that review's scope.

Confirmed corrections:

- Status entries are processed sequentially. A forced obligation records its triggering entry, immutable cutoff and retiring session; later recovery cannot erase it or move the cutoff.
- The derivation protocol honors that latched obligation even if a later map marks the old region live.
- Regular and forced activation atomically install the successor authority, reset its application count, set the phase active and clear pending metadata.
- A waiting forced handoff uses the first admissible enabling control prefix and seals its selected successor.
- Committee members sign at most one candidate successor per predecessor/version, including before installation.
- The normal-successor lemma explicitly names the required control-order premise.
- Forced-cut preservation includes completed honest application commitments, delivered prefixes and assemblable final receipts.
- Terminal receipt verification ends physical Drain/voting without waiting for another live region; evidence and fetch service remain available. The progress proof distinguishes subsequent receipt delivery from a new native timer-free commitment rule.
- Initialization names the genesis anchor, initial entry and first live owner from the predefined region list; initialization waits if none is eligible.

The existing **Regional live stream** definition remains byte-identical to v2: 658 bytes, SHA-256 `fbd6cd16242a26303d653fcae9ab024cea85d69d8233c91b2c46bcdc27b61770` in both files.

This verdict verifies the targeted corrections and their consistency with the conditional specification. It does not instantiate the external closed-cut mechanism, re-prove the external Sync HotStuff paper, certify a concrete proof-system implementation, or replace visual/presentation review.
