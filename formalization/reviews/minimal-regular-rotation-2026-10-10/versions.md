# Regular rotation: numbered protocol iterations

The unversioned document that was open when the user requested versioning is
preserved byte for byte. Numbered protocol sources are retained independently.
The 10 October source hashes are recorded in
[`versioned-sources-sha256.json`](versioned-sources-sha256.json). On 11 October,
the user explicitly requested illustrations and a vanilla-protocol comparison
inside V4. Its protocol logic and proofs remain unchanged; the previous V4
source, presentation diff, and current hash are preserved in the
[dated illustration record](../minimal-regular-rotation-2026-10-11/README.md).

| Version | Source | Status |
|---|---|---|
| V1 | [First-received quit proof](../../minimal_regular_rotation_v1.tex) | Snapshot of the prior open draft. Fails the requested common-next-leader property; later-proof/re-entry behavior is incomplete. |
| V2 | [Local minimum after one delay](../../minimal_regular_rotation_v2.tex) | Implements the proposed minimum BLAME view with local committed-view cutoff. A legal delayed-certificate execution yields different leaders even with identical committed cutoffs. |
| V3 | [Agreed recovery construction](../../minimal_regular_rotation_v3.tex) | Preserves vote-start timers and immediate normal rotation. Supplies fresh recovery epochs, a common recovery vector and a recovery-anchor commit. Review requested explicit rank, proposal duty and complete-bundle delivery. |
| V4 | [Reviewed proof-bearing version](../../minimal_regular_rotation_v4.tex) | Makes the V3 specifications explicit. Contains safety, common recovery leader, and committed-height progress proofs under the stated idealized synchronous model. |
| V5 | [Explicit equivocation draft](../../minimal_regular_rotation_v5.tex) | Intermediate audit revision; retained separately. The paper-notation presentation below is the requested formal deliverable. |

The standalone [Sync HotStuff RR](../../sync_hotstuff_rr.tex) rewrites the
count-based V4 construction using the original paper's two-figure structure
and message notation. Red text and its final table identify the protocol
differences. It leaves V4 and the weighted regional manuscript unchanged.

## Findings carried into V4

- A local delay does not close a common set of evidence: Byzantine members
  can withhold the shares that complete an older certificate.
- A local committed-view cutoff can differ across replicas. Even identical
  cutoffs do not fix the delayed-certificate example in V2.
- Full proposal/admission histories, immediate normal catch-up, global
  same-epoch fork/quit handling, and continued QC assembly while frozen allow
  the original proposal/vote-start `2 Delta` timer to be retained.
- One composite view identity `(epoch, term)` determines one scheduled
  leader. Recovery changes the epoch, so returning to a term index never
  reuses an old signing identity.
- A concrete failure-only authenticated broadcast construction supplies the
  same vector, minimum blamed term and highest admissible QC to all correct
  nodes. Its rounds explicitly account for entry-time skew.
- Recovery commits its agreed safe anchor. This avoids indefinite finality
  starvation from successive malicious leaders cancelling all pending timers.

## Validation and limits

V2, V3 and V4 passed the built-in standalone LaTeX compiler. The original V4
had four pages; its illustrated presentation has seven. The two main protocol
boxes remain together on page 1. A DVI-only layout check
reported no overfull or underfull boxes; no separate PDF was exported and no
new editor tab was opened. V1 is byte-identical to the preserved open document.

Independent reviews of the construction and exact V3 source found no
substantive attack under the stated model, but identified the specifications
clarified in V4. A further exact-V4 review confirmed those gaps were closed
and found no new material issue. These are mathematical reviews, not a
machine-checked proof or exhaustive executable model check.

The model is fixed-membership authenticated synchrony with `n = 2f+1`, complete
messages and validation within `Delta`, and finite processing in bounded time.
Liveness is unbounded committed-height progress; transaction inclusion also
requires fair request selection. The result does not automatically cover
stake weights, mobile sluggish nodes, data-fetch delays beyond the stated
bound, or Byzantine wire flooding.

The normal path retains immediate terminal-QC handoff and the original timed
commit. Failure recovery costs `f+1` authenticated-broadcast rounds, padded
to `2 Delta` each. Every correct node decides within `(2f+4) Delta` of the
first freeze. This is a conservative reference implementation; no optimality
or minimum-recovery-cost claim is made.
