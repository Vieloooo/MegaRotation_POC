# Figures for the major v2 revision

All files are native TikZ inputs. The preamble needs `tikz` with `arrows.meta` and `calc`;
the surrounding document supplies captions and labels. No manuscript macros are
required. The notation is `S_{i,j}`, `V_i`, `P`, `M_h`, `B_k`, `Q_v(B)`, `pi`, and
`Delta_R` (with the corresponding TeX commands).

- `architecture.tex`: preserves the application layout of
  `../v2/architecture.tex`: users, active leader and regional peers, proposer
  push/pull paths, other regions, and L1. The outer CFT remains dashed and
  unfinished. The receipt `pi` is intentionally independent of a BFT format.
- `regional-pipeline.tex`: all-to-all vote/echo, certified-parent chaining,
  overlapping commitment timers, and a receipt to the proposer. Horizontal
  positions show causal order and are not a quantitative time scale.
- `blame-communication-pipeline.tex`: active view-change communication figure,
  showing representative members in the same region: BLAME exchange or
  equivocation evidence, quit-proof relay, each member's one-delay wait, STATUS,
  the new leader's two-delay wait, NEWVIEW, unchanged-anchor re-certification,
  and the first fresh proposal. BLAME requires weight greater than one third.
  No timer starts on anchor re-certification.
- `view-change.tex`: earlier compact recovery overview, retained as a source
  reference; it is no longer an input of `main_v2.tex`.
- `regional-communication-pipeline.tex`: the detailed steady-state message
  flow, with vote certification (weight greater than two thirds) separated
  from post-commit receipts (weight greater than one third).
- `retrieval.tex`: leader push and non-leader poll-fetch, followed by verification
  and contiguous delivery. Fetching does not change the committed log.

The prior `figures/v2/` files are unchanged.
