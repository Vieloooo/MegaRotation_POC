# Multi-agent model policy — 2026-11-16

User override for all subsequent PoC work; supersedes older handoff model instructions.

| Role | Model | Reasoning effort |
| --- | --- | --- |
| Main coordinator / feature lead / merge lead | gpt-6.1-sol | ultra (unchanged) |
| Docer / bft_spec | gpt-6-astra | high |
| Independent Verifier / bft_verifier | gpt-6.1-sol | xhigh (unchanged) |
| Builder / bft_builder | gpt-6.1-sol | high (unchanged) |
| Tester / bft_simulator | gpt-6.1-sol | high (unchanged) |

Other non-Docer agents use gpt-6.1-sol and retain their previously assigned effort. Preserve independent source-first expectations, one writer per path, frozen contracts and same-revision verification. Five active user-chat slots and the heavy-build lock are unchanged.

Do not restart stopped/idle chats merely to propagate this policy. Existing execution records remain historical facts, not rewritten model claims. Local config changes are defaults for future loading, not proof that running agents switched models. At the next authorized turn/spawn, explicitly select the new model/effort and verify actual availability. If a named role is still loaded with the old fixed model/effort, use a generic agent with the same role instructions and explicit requested model/effort (fork_turns=none where required), or reload through supported means; do not claim the old named role has already changed.

Track local .codex changes where those files are already versioned; other worktrees may retain ignored local copies. This tracked policy travels with handoffs. Worktrees lacking local role files must read the source checkout role instructions and this policy. Preserve user-stopped chats until the user requests resumption or explicitly identifies them for continuation.

Application note: the still-active H3 chat notification/model-override call failed with an MCP transport error. Its live model switch is not confirmed; stopped chats were not messaged or restarted. The current session role catalog has refreshed to Docer Astra/high and Verifier Sol/xhigh.
