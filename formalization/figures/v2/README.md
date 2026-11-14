# Figures for `main_v2.tex`

All files are native TikZ inputs, use monochrome styling, and fit an approximately 159 mm text column without scaling. Required libraries: `arrows.meta`, `positioning`, `fit`, `calc`.

- `architecture.tex`: application-level decentralized-sequencer overview adapted from v1 Figure 1: L1, users, the producing region's leader and peers, proposer P, push/evidence/pull paths, and other regional SMRs. Dotted components mark unfinished cross-region CFT/rotation; dashed arrows are pull requests.
- `pos-adapter.tex`: fixed stake configuration, scheduled leadership, weighted certificate verification, and evidence submission to L1.
- `steady-state.tex`: proposal, all-to-all echo/vote, pipelined next proposal, nonblocking synchronous commit timer, and regional proof export to the proposer.
- `pipeline.tex`: parent-QC proposal gating and overlapping commit timers; shows local commit latency separately from export latency.
- `outer-interface.tex`: the regional logical-agent boundary and the explicitly unspecified outer protocol.
- `handover.tex`: scheduled active, freeze, drain, and status boundaries; next-view first-parent constraint.
- `evidence.tex`: public verification of a conflicting pair of signed statements and L1 adjudication; keeps the running stake snapshot fixed.
- `transport.tex`: client admission, batch construction, regional commit, decision export, and missing-data repair.

The time axes are schematic. In particular, the pipeline figure does not claim finality after two actual network delays.

Protocol coverage: Protocol 1 (admission) and Protocol 5 (export) use `transport.tex`; Protocol 2 (configuration) uses `pos-adapter.tex`; Protocol 3 (regional steady state) uses `steady-state.tex` and `pipeline.tex`; Protocol 4 (scheduled handover) uses `handover.tex`; Protocol 6 (evidence) uses `evidence.tex`. `architecture.tex` and `outer-interface.tex` show the specification boundary and make no claim of a completed outer CFT protocol.
