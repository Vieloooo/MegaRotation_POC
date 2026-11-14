# MegaETH Sequencer Rotation Research

Research notes for understanding MegaETH's transaction pipeline and designing sequencer rotation.

## Start reading

- [当前形式化文档：Parts 0–IV（最新 PDF 与源码）](formalization_v3/README.md)

- [过去一周的设计思路与 Codex 对话索引（2026-09-28）](research-index/2026-09-28-weekly-design/README.md)
- [中文学习指南](learning/zh-CN/README.md)
- [English learning guide](learning/README.md)
- [补充讨论：交易回执、交易根，以及 Mini-block、Flashblocks 与 OP Stack Subblocks](learning/zh-CN/05_miniblocks_and_flashblocks.md)
- [OP Stack：整体架构、交易确认与 Batch 数据往返](learning/zh-CN/06_op_stack_architecture_and_pipeline.md)
- [排序器选择、费用与代币经济：OP Mainnet、Base、MegaETH](learning/zh-CN/07_sequencer_selection_fees_and_tokenomics.md)
- [OP Stack：桥接、争议与故障处理流程](learning/zh-CN/08_op_stack_bridge_and_disputes.md)
- [OP Stack：从错误 root 到 Cannon VM 单步裁决](learning/zh-CN/09_fault_proof_bisection_and_cannon_vm.md)
- [OP Conductor：Sequencer 高可用、权限与激励](learning/zh-CN/10_op_conductor_ha_trust_and_incentives.md)
- [11 — OP Stack 与 MegaETH：架构、实时确认与高可用设计对比](learning/zh-CN/11_op_stack_vs_megaeth_architecture.md)
- [Restaking 服务学习专题：EigenLayer、EigenDA、共识与轮换](learning/zh-CN/restaking-services/README.md)
- [Restaking / EigenDA 资料索引与阅读路线](learning/zh-CN/restaking-services/sources.md)
- [Meeting questions — 2026-09-16](meetup_record/260916.md)
- [Meeting 0916 Response — Sequencer Obligations and Accountability](meetup_record/meeting_0916_response.md)
- [Meeting 0922 — L1 + BFT 架构愿景与本周需求](meetup_record/meeting_0922_requirements.md)
- [Proposed architecture](meetup_record/rotation_overview.jpg)

The notes distinguish local specification requirements, public documentation, OP Stack reference behavior, inferences, and proposed designs.
The initial literature review is dated 2026-09-16; the follow-up discussion was saved on 2026-09-17.

## Specification reference

`megaeth_specs` is a Git submodule pointing to the separate [MegaETH specification repository](https://github.com/megaeth-labs/specification/tree/59797b45b1f59b7c2c147e6f99be39399a1f2b44).
It is pinned to commit `59797b45b1f59b7c2c147e6f99be39399a1f2b44`, matching the snapshot used by the notes.
Access to that repository is controlled separately from access to this notes repository.

When cloning this repository on another machine, use `git clone --recurse-submodules` with this repository's clone URL.
If it is already cloned, run:

```sh
git submodule update --init --recursive
```

The learning notes and meeting materials can be read without initializing the submodule.
Relative links into `megaeth_specs` resolve in the local checkout after submodule initialization; on GitHub, use the pinned specification link above to browse the reference source.
