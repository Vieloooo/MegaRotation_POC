# MegaETH Sequencer Rotation PoC v2

这是 Sequencer Rotation PoC 的独立 Git checkout。目标是在单机运行三个地区、每区五个等权 replica，并用可重放的场景验证 regular、Force 和 RESUME。先看 [PoC v2 设计](formalization_v10/poc_design_zh_v2.md) 和 [v10 协议规范](formalization_v10/README.md)。

## 当前进展

截至 2026-11-15，运行基础已能驱动真实 Simplex 双进程，正在验证同一 `cid` 的崩溃恢复。当前实现基线为 `codex/poc-v2-i1` 上的 `0f013e2`，Commonware 2026.10.0，Rust 1.95.0。

| 范围 | 已有证据 |
| --- | --- |
| I1a 受控运行基础 | 两个真实 worker 进程执行 `SignedIntent` 测试。Clock RPC、计时器、受控消息和日志通过有限验收，24 对重放的完整虚拟轨迹逐字节一致。[I1a 验收结果](poc/docs/i1a-gate-result.md) |
| I1b 原生子项 | 受控任务、时钟和 RNG、存储桥接及持久化见证已分别通过独立验收。[运行时](poc/docs/i1b-runtime-slice-result.md) · [存储](poc/docs/native-storage-result.md) · [见证](poc/docs/native-witness-result.md) |
| 原生 Simplex Fresh | 五成员、四票 quorum，由两个真实 native worker 和最多三个脚本签名者组成。66 次运行经过独立检查，33 对重放结果逐字节一致。健康场景在两个真实 worker 各交付两块。[Fresh 验收结果](poc/docs/native-engine-fresh-result.md) |
| 同一 `cid` 的进程恢复 | `recover-pending-verify` 在 seed 7 下杀掉等待区块体的 worker，以新 PID 和 incarnation 恢复原 Verify 工作及原认证截止时间。场景按设计在零交付处结束。日志头暂存文件的两个独立回归用例已通过。[实现和复跑记录](poc/docs/native-recovery-implementation.md) |
| 轮转与外部服务 | 15-worker regular 和 D 派生处于准备阶段。Anvil 已通过本地手动出块 smoke。regular、Force、RESUME 及 PoC Web Observer 的端到端场景仍待实现。[regular 准备](poc/docs/contracts/regular-preparation-v1.md) · [D 派生准备](poc/docs/contracts/derivation-preparation-v1.md) · [Anvil smoke](poc/docs/setup/foundry-2026-11-15.md) |

完整 I1b/I1 仍开放。当前验收范围是 [原生恢复的 R1–R18](poc/docs/contracts/task-contract-native-recovery-v1.md#12-executable-acceptance-r1r18)，其中还包括真实 resolver 修复跨进程重启、未完成事务与丢失 Ack，以及多 seed 和乱序重放。随后按 regular 准备稿先用 15 个真实 worker 验证单一区域 session 的准入，再串联三个区域的交接与 GFR。

## 工作区与复现

- `poc/crates/` 是 I1a 的 hub、worker、运行时和共享类型。`poc/prototypes/i1b-runtime/` 是原生 Simplex 原型，有自己的 Cargo manifest 和锁文件。`poc/proto/` 存放两者的 RPC 定义，`poc/vendor/` 和 `poc/patches/` 存放固定版本的 Commonware 源码和补丁。
- `poc/docs/contracts/` 是任务合同和准备稿，`poc/verification/` 是独立检查器、用例和可版本化的结果。原始实验输出在本地忽略目录 `poc/artifacts/` 中，按 `setup/`、`verification/`、`simulation/` 存放。`.build/` 保存本机工具和构建缓存。
- `regional_BFT_sim/` 是固定版本的 Regional BFT 子模块，包含共识 benchmark 与回放页面。当前子模块提交 `691cfc1` 已在本机提交，尚需推送到子模块的 GitHub 远端后才能从远端初始化。PoC 的 15-worker 目标使用三个独立的五成员委员会。[交接说明](formalization_v10/poc_v2_xuelab_handoff_zh.md) · [接收记录](poc/docs/setup/preflight-2026-11-15.md)

从本 README 所在目录执行。先安装 Rust 1.95.0 和 rustfmt，初始化子模块。Cargo 缓存和编译产物保存在本 checkout 的 `.build/`。当前机器可用 `REGIONAL_BFT_RUST_BIN=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin` 选择已有工具链。

首次使用空缓存时，先执行 `./poc/scripts/bootstrap.sh fetch --locked` 和 `./poc/scripts/bootstrap.sh fetch --manifest-path prototypes/i1b-runtime/Cargo.toml --locked` 下载两个锁文件对应的依赖。以下测试使用本地缓存。

```sh
git submodule update --init --recursive
./poc/scripts/bootstrap.sh
./poc/scripts/bootstrap.sh fmt --manifest-path prototypes/i1b-runtime/Cargo.toml --all --check
./poc/scripts/bootstrap.sh test --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline
```

真实进程恢复需要 loopback，并使用新的输出目录。[运行命令与场景边界](poc/docs/native-recovery-implementation.md#bounded-process-scenario)

## 中文进度报告

[PoC 当前进度、验证范围和下一步](poc/docs/POC进度报告.md)。

[分任务交接索引与新会话工作约定](poc/docs/handoffs/README.md)。

历史设计、学习资料和会议记录保留在本仓库 Git 历史的 `0f013e2` 中。v10 文档里的历史来源链接可通过 `git show 0f013e2:路径` 查看，v10 正文与 PoC 设计保持原始内容。

## 规格快照与交接来源

本 checkout 的 [`megaeth_specs/`](megaeth_specs/) 是随交接纳入 Git 的普通文件，来源版本为 `a738f9b14c7880953d8cff8788db14e8e3e79124`。原研究仓库的旧规格链接记录历史来源，其中 `59797b45b1f59b7c2c147e6f99be39399a1f2b44` 是旧笔记引用的版本。本 checkout 已包含规格文件，普通 checkout 即可阅读。[交接来源清单](handoff_provenance/manifest.json)

`handoff_provenance/SHA256SUMS` 针对接收基线 `8086ce146776b88ea917986a521b0587765e1de2`。要复核原始交接，在临时目录解包该提交的 `git archive`，再在解包目录运行 `sha256sum -c handoff_provenance/SHA256SUMS`。[接收记录](poc/docs/setup/preflight-2026-11-15.md)
