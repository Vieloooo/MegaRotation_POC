# Formalization v4

本目录用于将 sequencer rotation 整合为单篇、自包含的稿件。

- [详细中文写作提纲](manuscript_outline_zh.md)：逐段内容、协议块、图、接口和证明安排。
- 最终目标仍为一个自包含的 `main.tex`；当前先将第 2、3 节分文件审阅，后续再合并。

提纲采用总—分结构，覆盖 215 个计划段落、16 个协议块、13 张图的设计、12 张表，以及 38 个内部接口。
正文安排为 11 节，附录 A–D 与正文放在同一个 TeX 文件中。

基线为 `formalization_v3` 的 Part II v4.9、Part III v6 和组合式抽象 v2。
L1/Espresso 以假设表达，TEE 作为可信第三方；共同切换边界和前缀保护仍是本协议必须闭合的设计工作。

## 第 2、3 节审阅稿

- [Section 2: Model, Assumptions, and Goals](sections_02_03/section_02_model_assumptions.tex)
- [Section 3: Shared Objects and Contracts](sections_02_03/section_03_shared_objects.tex)
- [审阅编译入口](sections_02_03/main.tex)
- [13 页 PDF 预览](../output/pdf/sequencer_rotation_v4_sections_02_03.pdf)

两节为独立的英文 TeX 正文文件，可以分别手动修改。
`sections_02_03/main.tex` 只提供共用排版设置并载入这两节，保留章节编号 2、3；它不是其他章节的正文草稿。
在编辑器中打开该项目或任一正文文件可查看项目预览；正文修改后应重新编译，已导出的 PDF 不会自行更新。

本轮检查包括来源/术语一致性、组合编译、交叉引用及全部页面的版面检查，不构成完整协议证明验证。
