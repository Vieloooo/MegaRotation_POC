# main_v1 notation and consistency review — 28 September 2026

Target: [main_v1.tex](../main_v1.tex), with its two active version-specific TikZ inputs. This review concerns the current manuscript, not `main.tex` or `main_PBFT.tex`.

## Result

No outstanding material inconsistency was found between the revised definitions, algorithms, proof dependencies, and figures within the manuscript's stated fault, synchrony, availability, capacity, and tenure assumptions. This is a manuscript consistency review, not an implementation verification or a new full security certification.

## Changes

- Section 3 defines the roles of the regional and global evidence packages at their first use; Section 4.5 states their contents and dependence on a trusted anchor.
- Section 4 distinguishes the mini-block, regional-outcome, and global-checkpoint histories. It defines regional outcomes before using their hashes in proposals and includes the instance-7 / blocks-101–103 example explaining `R_6`.
- Defined fixed type tags, canonical encoding, complete mini-block fields, quorum certificates, and the difference between VOTE and DECIDED evidence.
- Made verification context and witness inputs explicit. Bounded messages carry new objects and relative witnesses; a fresh receiver fetches missing ancestry separately.
- Separated global state `Sigma` from node identity `S`, regional and global failure counters, completion inputs, and the stored decision digest from the protocol's proposal-valued decision.
- Defined `R_0` as an initialization variant authorized by genesis or an activation proof, with no regional `RC_0`.
- Made activation references and readiness-report references semantic fields, with signatures and certificate encodings detached. This prevents arbitrary proof encodings from changing agreed checkpoint identities.
- Defined report-vector ordering, pending-rotation fields, transport object references, and local-state value types.
- Removed the LMD-GHOST / HEAD state, protocol, conflict rules, explanatory references, and communication-cost term. Regional Preview and global Deliver remain distinct certified outputs.
- Replaced the active figure inputs with [service-architecture-v1.tex](../figures/service-architecture-v1.tex) and [stream-pipeline-v1.tex](../figures/stream-pipeline-v1.tex). Shared figures and other manuscript variants were not edited.

## Consistency repairs found during review

1. At an activation boundary, the no-extension transition must retain the **effective state's** fresh `R_0`, not the old checkpoint's regional cursor. Pending rotation likewise retains the parent data tip together with the effective regional cursor.
2. Specified preservation of fields not explicitly updated and the global no-progress counter's increments and activation reset.
3. Made persistent ORDER and PROPOSE sign-once rules explicit so that the equivocation-evidence proposition has matching algorithmic rules.
4. Specified that an FFG ancestry path consists of valid checkpoint transitions connecting its source to its target.
5. Defined the backbone arc count `E_B` and the per-arc forwarding convention used by the communication bounds, avoiding collision with the completion support set `E`.
6. Clarified positive parameter domains and the responsive-consumer premise used by delivery bounds.

## Validation

- Independent read-through of the revised object definitions and their use in regional broadcast, completion, consumer verification, local/global transitions, finality, safety/liveness proofs, and complexity analysis.
- No LMD-GHOST, HEAD, speculative-head algorithm, stale `S_e`, `z_S`, `readyProof`, or undefined activation-proof variable remains in the active source closure.
- All 69 source labels are unique; all 51 explicit source references resolve. The compile also resolves citations and generated references.
- Fresh isolated `latexmk` build succeeds: 24 pages, no LaTeX warnings, undefined references, overfull boxes, or underfull boxes in the final log.
- Rendered all 24 final pages for visual inspection; inspected the revised definitions in detail and verified both new figures. Kept Section 3's output notation with its explanation and the finality heading with its protocol.
- The compiled output is [main_v1.pdf](../main_v1.pdf).

## Source identification

SHA-256 of the source before this revision:

`8b35398116f4b744556058129713a73f309fac65a22f6fb4cce01fbb4fcdf905`

SHA-256 of the reviewed final `main_v1.tex`:

`f918927188fe07fd5962e0b2b65abde65565e727f1dbb811b2e49db8386cb13e`

Figure hashes:

- `service-architecture-v1.tex`: `62a8864286a337cc68e703ca1d0adcbe75bb74c9bd2e6b9ec8ce6d532ca8a6a5`
- `stream-pipeline-v1.tex`: `4ed6a8268967c0ad3412e531a1eb671b575c1f07504b5a41e312f938eb315dc4`
