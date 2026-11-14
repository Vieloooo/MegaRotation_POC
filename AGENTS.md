# Sequencer Rotation PoC v2

Read formalization_v10/poc_v2_xuelab_handoff_zh.md, poc_v2_xuelab_setup_zh.md, poc_design_zh_v2.md and DESIGN.md first.
Work explicitly in this isolated checkout; new implementation belongs in poc/. Do not edit/reset/clean the parent /mnt/reth/jy/regional_BFT_sim working tree or its README/examples/visualization changes. Reference its pinned backend read-only.
P0 mocks staking: 3 regions x 5 equal-weight replicas, 10-height windows, <=1 ms clock error with separate RPC delay, staggered Enter, regular/Force/RESUME. No staking or TLA.
The user authorized multi-agent work following Regional BFT README. Lead owns interfaces, dependencies, integration. Use bft_spec/bft_builder/bft_verifier/bft_simulator instructions in .codex/agents/. Main effort ultra; respect actual model availability and concurrency slots, do not silently downgrade.
Freeze the task contract before coding. Start one writer plus independent verification preparation. One owner per path; if concurrent writers are needed, use managed worktrees after a Git baseline. Verifier derives expectations from sources before reading implementation. Simulator runs agreed cases on the verified integrated revision, records commands/seeds/traces/limits. Initially one build/heavy simulation at a time.
First intake/preflight, then two-worker Clock RPC/timer/message/journal gate I1. No PoC implementation/test pass is claimed. Preserve terminal/signed state across crashes; separate virtual/wall/public/local time; no outputs bypass controlled grant/ack.
