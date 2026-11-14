# Part III 历史版本与审阅记录

当前 **v3 正文及阅读材料已移到 [`formalization_v3`](../../formalization_v3/README.md)**，与 Parts 0、I、II、IV 放在一起；`formalization_v2` 顶层路径保留兼容链接。本目录保留 v1、v2、逐轮审阅材料和旧入口。

- [当前 v3 PDF](../sync_hotstuff_rr_part3_staking_v3.pdf)
- [当前 v3 TeX 正文](../sync_hotstuff_rr_part3_staking_v3.tex)
- [v3 中文快速导读与记号表](../sync_hotstuff_rr_part3_staking_v3_quick_guide_zh.md)
- [中文逐轮修改记录](../sync_hotstuff_rr_part3_staking_changes_zh.md)
- [最终可读性校对](reviews/v3_readability_zh.md)
- [最终问题关闭复核](reviews/v3_protocol_zh.md)
- [完成项核对](reviews/completion_audit_zh.md)

日常编辑请使用 `formalization_v3` 中的 v3 文件。旧 `main.tex` 仍可编译，内容转向顶层正文；`versions/part3_v3.tex`、原导读和原修改日志路径保留兼容链接，不是另存的一份正文。本次操作未移动或删除 Vim 恢复文件。

从 `formalization_v2` 构建当前正文：

```sh
latexmk -pdf -interaction=nonstopmode -halt-on-error sync_hotstuff_rr_part3_staking_v3.tex
```

v1、v2 源码保存在 `versions/`；原 PDF 导出文件继续保留在仓库 `output/pdf/`。`reviews/` 和已有修改日志记录各轮审阅当时的状态，历史哈希不随后来编辑或迁移重写。路径迁移及迁移时哈希见 `relocation_2026_10_20.json`。

本次整理保留了磁盘上最新保存的正文内容，只更新目录和引用路径，包括 Part II 及会议记录中的 Part III 链接。
