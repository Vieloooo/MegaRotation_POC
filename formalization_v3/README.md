# 当前形式化文档（formalization_v3）

这里集中放置从 `formalization_v2` 整理出的各 Part 最新正文与 PDF（2026-10-20）。目录名 v3 表示新的工作目录；各文档保留自己的版本号。

| Part | 内容 | 最新版本 | PDF | TeX |
| --- | --- | --- | --- | --- |
| 0 | 原始 Sync HotStuff | 原版 | [阅读](sync_hotstuff_part0_original.pdf) | [源码](sync_hotstuff_part0_original.tex) |
| I | 区域内核心协议 | 当前版 | [阅读](sync_hotstuff_rr_part1_core.pdf) | [源码](sync_hotstuff_rr_part1_core.tex) |
| II | Sequencer Rotation | v4.6 | [阅读](sync_hotstuff_rr_part2_rotation_v4_6.pdf) | [源码](sync_hotstuff_rr_part2_rotation_v4_6.tex) |
| III | Staking and Authority | v3 | [阅读](sync_hotstuff_rr_part3_staking_v3.pdf) | [源码](sync_hotstuff_rr_part3_staking_v3.tex) |
| IV | Force Handler and Publish Layer | v0.1 | [阅读](sync_hotstuff_rr_part4_force_v0_1.pdf) | [源码](sync_hotstuff_rr_part4_force_v0_1.tex) |

## 配套阅读

- [组合式抽象 v2：Global Finality Rule 形式化](composability_abstraction_v2_zh.md)（2026-10-25；新修订，§5 给出状态、接纳谓词、各模式转移与契约条件）
- [组合式抽象：外部功能、模块接口与恢复](composability_abstraction_zh.md)（2026-10-25；基于 Part II v4.9、Part III v6 的接口设计稿，含与 mega-espresso 的对接与待定契约）
- [Part II v4.6 修改说明](rotation_v4_6_changes_zh.md)
- [Part III v3 中文快速导读与记号表](sync_hotstuff_rr_part3_staking_v3_quick_guide_zh.md)
- [Part III 逐轮修改记录](sync_hotstuff_rr_part3_staking_changes_zh.md)

日常阅读和编辑使用本目录。旧版本、架构历史和审阅记录保留在 `../formalization_v2/`；已迁移文件的原路径保留兼容链接，指向这里的同一份文件。

## 构建

在本目录运行 `./build.sh`（需要 TeX Live 和 latexmk）。脚本按 Part 0、I、II、III、IV 的顺序构建，将 PDF 放到本目录，中间文件放到 `build/`。Part IV 的跨文档引用依赖 Part II 生成的辅助文件，因此先构建 Part II。

图文件在 `figures/` 中。迁移前文件哈希见 `relocation_2026_10_20.json`；整理不修改协议正文。

迁移后五份 PDF 均已重新构建。当前构建仍有两处未解析的章节引用：Part II 的 `sec:late-receipt` 和 Part IV 的 `II-sec:transport-scope`；本次整理未修改这些正文引用。
