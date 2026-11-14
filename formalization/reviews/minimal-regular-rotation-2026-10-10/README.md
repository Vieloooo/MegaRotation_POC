# Minimal regular rotation: candidate and safety check

- [Standalone source](../../minimal_regular_rotation.tex)
- [Rendered PDF](../../minimal_regular_rotation.pdf)
- [Bounded trace replay](check_trace.py)
- [Replay output](trace-results.json)
- [Protected-manuscript hashes](protected-sha256.json)

The candidate adds normal entry on a terminal QC and defines the BLAME clock
as actual local view entry. It retains the original exit wait even when a
terminal QC arrives during that wait, and retains timed commit without an
extra QC-at-commit condition. Those explicit composition choices are unsafe:
an early entrant can commit a block without a QC while other members are
still closing the preceding view. Highest-QC recovery can then select a
conflicting continuation.

The exact-rational replay checks the decisive event schedule, voting
eligibility, timers, quorum counts and listed synchronous delivery bounds.
It reproduces conflicting commits for both n=5, f=2, QC=3 and n=7, f=2,
QC=5. An additional QC-at-commit guard blocks this particular trace. The
script is a witness replay, not exhaustive model checking or a safety proof
for the guarded variant. The counterexample does not establish that every
regular-rotation design is unsafe, nor does it invalidate the original
Sync HotStuff protocol or the guarded v3.1 draft.

Validation on 10 October 2026:

- Built-in standalone LaTeX compiler: success.
- Export with latexmk: success, three A4 pages; no LaTeX warnings or
  overfull/underfull box warnings in the final log.
- All three rendered pages visually inspected; the protocol fits one page.
- Replay completed for both unsafe profiles and the diagnostic guarded run.
- Original v3, v3.1 and v4 source/PDF hashes match the captured baseline.

To reproduce, run `python3 check_trace.py` from this directory. Build from
`formalization` using
`latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=build/minimal-regular-rotation minimal_regular_rotation.tex`.

Primary source: [Sync HotStuff, IEEE S&P 2020, Section II, Figures 1–2 and
Lemma 1](https://conferences.computer.org/sp/pdfs/sp/2020/349700a106.pdf).
