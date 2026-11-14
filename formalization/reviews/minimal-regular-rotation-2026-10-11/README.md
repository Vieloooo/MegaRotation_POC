# V4 illustrations and comparison — 11 October 2026

The user explicitly requested additions to the existing
[`minimal_regular_rotation_v4.tex`](../../minimal_regular_rotation_v4.tex).
This is a presentation revision of V4, not a new protocol iteration.

Four inline native TikZ figures were added:

1. Normal proposal/vote/QC pipeline with separate vote-start commit timers.
2. Terminal-QC handoff from Alice to Bob, preserving Alice's pending timer.
3. Different honest BLAME snapshots feeding one common recovery vector,
   with separate next-leader and anchor selections.
4. The `f=2` recovery schedule: parallel authenticated broadcasts, decisions
   at `7 Delta` and `8 Delta`, with identical outputs despite entry skew.

The document now has seven pages. Page 1 retains the two protocol boxes;
pages 2–3 contain the four figures; page 4 contains the comparison table and
explanation. The validation rules and proofs follow on pages 5–7.

The comparison distinguishes unchanged fast-path mechanisms from V4's new
normal handoff, epoch-wide cancellation, common failure selection, fresh
signing identity and recovery-anchor commitment. It does not compare the
baseline's waiting stages with V4's full recovery bound as if they had the
same endpoint. Diagram timing and baseline comparisons were independently
reviewed against the source and the original paper.

Validation:

- Built-in LaTeX compiler: success.
- DVI-only check: seven pages, no overfull or underfull boxes.
- Added pages rendered through the SVG driver and visually inspected as PNGs.
- Original protocol text matches after normalizing one repaired cross-reference
  and trailing whitespace; validation/proof text is byte-identical.
- V1–V3 and the unversioned baseline are unchanged.
- No separate PDF was exported; the current editor was kept open.

The exact previous V4 source is retained in
[`illustration-baseline.json`](illustration-baseline.json). See the
[presentation diff](v4-illustrations.diff) and
[hash/validation record](validation.json).
