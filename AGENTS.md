# Sequencer Rotation PoC v2

Read formalization_v10/poc_v2_xuelab_handoff_zh.md, poc_v2_xuelab_setup_zh.md, poc_design_zh_v2.md and DESIGN.md first.
Work explicitly in this checkout; new implementation belongs in poc/. Regional BFT is pinned in the regional_BFT_sim submodule. Keep its worktree and Git history intact unless the task explicitly includes changes there.
P0 mocks staking: 3 regions x 5 equal-weight replicas, 10-height windows, <=1 ms clock error with separate RPC delay, staggered Enter, regular/Force/RESUME. No staking or TLA.
The user authorized multi-agent work following Regional BFT README. Lead owns interfaces, dependencies, integration. Use bft_spec/bft_builder/bft_verifier/bft_simulator instructions in .codex/agents/. Main effort ultra; respect actual model availability and concurrency slots, do not silently downgrade.
Freeze the task contract before coding. Start one writer plus independent verification preparation. One owner per path; if concurrent writers are needed, use managed worktrees after a Git baseline. Verifier derives expectations from sources before reading implementation. Simulator runs agreed cases on the verified integrated revision, records commands/seeds/traces/limits. Initially one build/heavy simulation at a time.
The current I1a and bounded native fixture results are recorded in poc/docs and README.md. Overall I1 remains open. Preserve terminal/signed state across crashes; separate virtual/wall/public/local time; no outputs bypass controlled grant/ack.

Model policy (user updated 2026-11-16): only Docer/bft_spec uses gpt-6-astra with high effort. Main/feature/merge leads use gpt-6.1-sol/ultra; Verifier uses gpt-6.1-sol/xhigh; Builder and Tester use gpt-6.1-sol/high. Other efforts unchanged. Read poc/docs/commander/model-policy-20261116.md. Do not restart user-stopped chats to apply configuration.
