# MegaETH Sequencer Rotation PoC v2

> **2026-11-17 · `main` 已合入 M1–M5 主场景和离线回放。** [架构与实验报告](poc/docs/PoC架构与阶段工作报告.md) · [回放入口](poc/web/index.html) · [打开方式](poc/web/README.md)

这是 Sequencer Rotation PoC 的独立 Git checkout。目标是在单机运行三个地区、每区五个等权 replica，并用可重放的场景验证 regular、Force 和 RESUME。先看 [PoC v2 设计](formalization_v10/poc_design_zh_v2.md) 和 [v10 协议规范](formalization_v10/README.md)。

## 已验证的主场景

主场景使用 seed 42、15 个独立 worker 进程和原生 Simplex 证书。Hub 通过受控 Grant/Ack 推进消息、计时和故障交付，节点时钟误差上限为 1 ms，Clock RPC 延迟单独建模。M2–M5 使用本地 Anvil 执行链上控制，M4–M5 从 L1 收据与 DA 独立派生 D。表中时间均为 Hub 虚拟时间。

| 场景 | 主运行终点 | 离线回放 |
| --- | --- | --- |
| M1 健康满窗 | R0、R1 在 h10、h20 交接，R2 的 h21 于 31.085 s 进入 G。 | [Regular](poc/web/m1-gfr-replay/index.html) |
| M2 健康主动 Force | 在途 Force 生效时保留 h10，R2 的 h16 于 26.041 s 进入 G。 | [Force](poc/web/m1-gfr-replay/force.html) |
| M3 地区离线后自动 Force | R0 五个 worker 离线，2-of-3 治理触发 Force，R1 的 h18 于 28.041 s 进入 G。 | [自动 Force](poc/web/m1-gfr-replay/auto-force.html) |
| M4 G/D 分歧后预设 RESUME | h2 的批次错过派生窗口，G 按历史 D(h20) 对齐，R2 的 h21 于 31.0525 s 进入 G。 | [RESUME](poc/web/m1-gfr-replay/resume.html) |
| M5 排序停滞后自动 RESUME | finalized 暂停和公开时间键冻结两条路径均由治理选出 h30，R2 的 h31 于 41.084 s 进入 G。 | [自动 RESUME](poc/web/m1-gfr-replay/auto-resume.html) |

在浏览器打开 `poc/web/index.html` 即可回放封存轨迹。时间轴上的实心点标记新增或替换区块，M4、M5 还可按高度对照 G、G′、D，并沿 physical parent 查看区域 Final。回放和远程浏览说明见 [Web README](poc/web/README.md)。

## 验收范围

M1 的满窗 smoke 已通过。完整 M1 仍跟踪 SessionEnd、多候选、独立节点 reader 接线、多 seed/墙钟暂停矩阵和官方 Espresso 字节读回。主场景使用确定性 SimEspresso。M4、M5 的主运行止于恢复后首块进入 G，该块与同高度 D 的后续分歧仍待跟踪。[M1 smoke 证据](poc/docs/milestones/results/M1-healthy-smoke-result-zh.md) · [完整 M1 合同](poc/docs/milestones/M1-regular-merge-zh.md) · [M1–M5 实现计划](poc/docs/milestones/M1-M5-implementation-plan-zh.md)

完整 I1b/I1 仍开放。后续验收包括 resolver 跨进程修复、未完成事务、丢失 Ack，以及多 seed 和乱序重放。[原生恢复 R1–R18](poc/docs/contracts/task-contract-native-recovery-v1.md#12-executable-acceptance-r1r18)

| 范围 | 已有证据 |
| --- | --- |
| I1a 受控运行基础 | 两个真实 worker 进程执行 `SignedIntent` 测试。Clock RPC、计时器、受控消息和日志通过有限验收，24 对重放的完整虚拟轨迹逐字节一致。[I1a 验收结果](poc/docs/i1a-gate-result.md) |
| I1b 原生子项 | 受控任务、时钟和 RNG、存储桥接及持久化见证已分别通过独立验收。[运行时](poc/docs/i1b-runtime-slice-result.md) · [存储](poc/docs/native-storage-result.md) · [见证](poc/docs/native-witness-result.md) |
| 原生 Simplex Fresh | 五成员、四票 quorum，由两个真实 native worker 和最多三个脚本签名者组成。66 次运行经过独立检查，33 对重放结果逐字节一致。健康场景在两个真实 worker 各交付两块。[Fresh 验收结果](poc/docs/native-engine-fresh-result.md) |
| 同一 `cid` 的进程恢复 | `recover-pending-verify` 在 seed 7 下杀掉等待区块体的 worker，以新 PID 和 incarnation 恢复原 Verify 工作及原认证截止时间。场景按设计在零交付处结束。日志头暂存文件的两个独立回归用例已通过。[实现和复跑记录](poc/docs/native-recovery-implementation.md) |

## 工作区与复现

- `poc/crates/` 包含 I1a 运行基础和发布、GFR、L1、批次及派生组件。`poc/prototypes/i1b-runtime/` 包含原生 Simplex 运行时和 15-worker Hub，有独立的 Cargo manifest 和锁文件。`poc/scenarios/` 包含仿真入口，`poc/contracts/` 包含 Solidity 合约。
- `poc/proto/` 存放 RPC 定义，`poc/vendor/` 和 `poc/patches/` 存放固定版本的 Commonware 源码和补丁。`poc/verification/` 包含独立检查器、用例和可版本化的结果。报告中的 `.build/commander-jobs/` 原始证据路径指向 xuelab 本地输出。
- `regional_BFT_sim/` 固定在 `691cfc1`，包含 Regional BFT 的共识 benchmark 和回放页面。查看参考源码时可用 `git submodule update --init --recursive` 初始化该子模块。[交接说明](formalization_v10/poc_v2_xuelab_handoff_zh.md) · [接收记录](poc/docs/setup/preflight-2026-11-15.md)

从仓库根目录执行以下组件检查。先安装 Rust 1.95.0 和 rustfmt。运行时使用 Commonware 2026.10.0，Cargo 缓存和编译产物保存在 `.build/`。xuelab 可用 `REGIONAL_BFT_RUST_BIN=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin` 选择已有工具链。

首次使用空缓存时，先执行 `./poc/scripts/bootstrap.sh fetch --locked` 和 `./poc/scripts/bootstrap.sh fetch --manifest-path prototypes/i1b-runtime/Cargo.toml --locked` 下载两个锁文件对应的依赖。以下测试使用本地缓存。

```sh
./poc/scripts/bootstrap.sh
./poc/scripts/bootstrap.sh fmt --manifest-path prototypes/i1b-runtime/Cargo.toml --all --check
./poc/scripts/bootstrap.sh test --manifest-path prototypes/i1b-runtime/Cargo.toml --locked --offline
```

真实进程恢复需要 loopback，并使用新的输出目录。[运行命令与场景边界](poc/docs/native-recovery-implementation.md#bounded-process-scenario)

## 历史进度与交接

[I1 阶段进度记录](poc/docs/POC进度报告.md)。

[分任务交接索引与新会话工作约定](poc/docs/handoffs/README.md)。

历史设计、学习资料和会议记录保留在本仓库 Git 历史的 `0f013e2` 中。v10 文档里的历史来源链接可通过 `git show 0f013e2:路径` 查看，v10 正文与 PoC 设计保持原始内容。

## 规格快照与交接来源

本 checkout 的 [`megaeth_specs/`](megaeth_specs/) 是随交接纳入 Git 的普通文件，来源版本为 `a738f9b14c7880953d8cff8788db14e8e3e79124`。原研究仓库的旧规格链接记录历史来源，其中 `59797b45b1f59b7c2c147e6f99be39399a1f2b44` 是旧笔记引用的版本。本 checkout 已包含规格文件，普通 checkout 即可阅读。[交接来源清单](handoff_provenance/manifest.json)

`handoff_provenance/SHA256SUMS` 针对接收基线 `8086ce146776b88ea917986a521b0587765e1de2`。要复核原始交接，在临时目录解包该提交的 `git archive`，再在解包目录运行 `sha256sum -c handoff_provenance/SHA256SUMS`。[接收记录](poc/docs/setup/preflight-2026-11-15.md)
