# M1 / M2 GFR 离线回放

## M1 Regular Handoff

用浏览器打开 [index.html](index.html)。同目录的 [force.html](force.html) 回放 M2。整个页面封装在一个 HTML 文件中，可复制到其他机器离线打开。页面打开时定位 h10/q9，可用「起点」从头播放。使用播放、暂停、前后单步或时间轴游标浏览，点击两个接棒按钮可定位到职责切换。点击 G 链里的接纳或接棒可查看条件、计算值和证据引用。

共享横轴是运行的虚拟时间，详情另列 Espresso 公开排序键 `θ`、`q_avail` 和接纳位置。L1 泳道按 fixture 的 finalized 时刻推进，逐 header 的 reader 收取时刻未记录。原始 fixture 共 31 个 header，本次轨迹结束时已有 ℓ0–ℓ14 达到 fixture finalized 时刻。

## 从封存证据生成

在本 worktree 根目录执行。生成脚本使用 Python 3 标准库。

```sh
M1_ARTIFACTS=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m1-regular/artifacts
M1_BUNDLE="$M1_ARTIFACTS/healthy-smoke-91547857-seed42-run01"
M1_INDEPENDENT="$M1_ARTIFACTS/independent-verification-91547857-seed42-run01"

python3 -B poc/verification/m1-gfr-replay/generate.py \
  --bundle "$M1_BUNDLE" --independent "$M1_INDEPENDENT"
python3 -B poc/verification/m1-gfr-replay/generate.py \
  --bundle "$M1_BUNDLE" --independent "$M1_INDEPENDENT" --check
python3 -B poc/verification/m1-gfr-replay/test_projection.py \
  --bundle "$M1_BUNDLE" --independent "$M1_INDEPENDENT"
```

`--check` 重新解码和核对输入，并逐字节比较已生成的页面。封存运行提交为 `9154785732d803c1f2f69aee3b2af67220a12f58`，seed 42。[冻结投影与验收合同](../../verification/m1-gfr-replay/contract-v1-zh.md)

## 证据对照

公共消息索引和独立报告数组索引从 0 起，trace 行号从 1 起。本次使用到的 trace 行号与 seq 相同。L1 覆盖位置按 `ts_L1 ≥ θ − 10,000,000 μs` 推导，origin 另列。

| 观察 | `raw/trace.jsonl` 行号 | 公共对象及独立报告 | L1 |
| --- | --- | --- | --- |
| h10/q9，R0 → R1。[11,20] 接 h10，选区参考 h0 | 区域首条 7288 · Publisher 7425 · G 7426 · Replay 7427 · Handoff 7428 | `raw/results/10.bin` · `raw/espresso/0009.bin` 正文 9、证明 10 · `public-replay.json` 的 `expected[9]`、`public_finals[9]` | 控制覆盖 ℓ6，origin ℓ7 |
| h20/q19，R1 → R2。[21,30] 接 h20，选区参考 h10 | 区域首条 15042 · Publisher 15173 · G 15174 · Replay 15175 · Handoff 15176 | `raw/results/20.bin` · `raw/espresso/0019.bin` 正文 9、证明 10 · `expected[19]`、`public_finals[19]` | 控制覆盖 ℓ11，origin ℓ12 |
| h21/q20，R2 首块接 h20 进入 G | 区域首条 15905 · Publisher 16037 · G 16038 · Replay 16039 | `raw/results/21.bin` · `raw/espresso/0020.bin` 正文 0、证明 1 · `expected[20]`、`public_finals[20]` | 控制覆盖 ℓ11，origin ℓ13 |

独立报告对 21 份公开最终证明进行了验签，并复算了 21 个 G 条目。页面生成对照报告中的证明字节、公开位置与逐块结果。SessionEnd 早停是 v10 的另一条 Regular 路径，本次两次接棒均由满窗接纳产生。[M1 健康满窗 smoke 记录](../../docs/milestones/results/M1-healthy-smoke-result-zh.md) · [v10 GFR §3–4](../../../formalization_v10/09_finality_rules_zh_v0_1.md)

## 验收记录

- 生成检查逐字节复现单文件，SHA-256 为 `531b5b2b8976011507fae5ea48e914c35c837fe23ccbefd31d42c1c50d1a552e`。负例覆盖二进制截断、尾随字节、原始文件摘要错误及独立证明不匹配。
- 独立读取核对 31 个 L1 header、21 条 Espresso 日志、21 次 G 接纳、105 条区域交付引用与两次接棒。证明位置与字节逐项对照封存报告。
- Firefox 158.0 用 `file://` 打开隔离目录中的单文件。通过播放、暂停、前后单步、拖动、轨道筛选、两次接棒跳转、h21 跳转、全部 21 个接纳面板，以及检查 L1 证据时保持当前职责。浏览器未加载外部页面资源。

## M2 Healthy Force

用浏览器打开 [force.html](force.html)。页面从 Force 提交开始，可用各阶段按钮定位。沿同一虚拟时间轴查看已在途的 L1 控制、h10/q9 常规交接、Force 激活、同键旧结果的排除，以及 R2 首块的接纳。L1 轨道展示真实 Anvil 收录、模拟 finalized 和 A/B reader 交付。执行用 `raw/l1.bin` 与 Anvil 按高度及 timestamp 对应，两者的 hash 各有出处。[Force 投影与验收合同](../../verification/m1-gfr-replay/force-contract-v1-zh.md)

从 worktree 根目录生成指定的 seed 42 封存运行：

```sh
M2_ARTIFACTS=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m2-healthy-force/artifacts
M2_BUNDLE="$M2_ARTIFACTS/merge-ea4d853b-seed42-force-run01"
M2_INDEPENDENT="${M2_BUNDLE}-independent"

python3 -B poc/verification/m1-gfr-replay/generate.py --scenario force \
  --bundle "$M2_BUNDLE" --independent "$M2_INDEPENDENT"
python3 -B poc/verification/m1-gfr-replay/generate.py --scenario force \
  --bundle "$M2_BUNDLE" --independent "$M2_INDEPENDENT" --check
python3 -B poc/verification/m1-gfr-replay/test_force_projection.py \
  --bundle "$M2_BUNDLE" --independent "$M2_INDEPENDENT"
```

M2 封存运行使用集成提交 `ea4d853bd8b62ee04c4e8e470550648242073950`。真实 Force receipt 的 status 为 1，交易位于 ℓ6 (6,0,0)。公共位置与实际观察 tick 可分别核对。

| 步骤 | 公开键与虚拟观察 | 原始与独立证据 |
| --- | --- | --- |
| L1 提交及收录 | τ 11.500 s 提交，12.000 s 收录，L1 timestamp 12 s | trace 760、943，`raw/l1/hub-l1.jsonl` 行 21、24，`raw/l1/raw-rpc-bundle.json` 中 tx `0x8cfb8a53de594ebddb28dee92ffb03e02c01e1bc0158386f3a1ba2f4f019eb63`，独立 `controls.json[0]` |
| L1 finalized 和可读 | τ 13.500 s finalized，A 13.500 s、B 13.510 s 可读 | trace 2294–2296，hub-l1 行 26–28，独立 `report.json` 的 `stages` |
| 在途 Regular | h10/q9，θ 20.044 s，τ 20.045 s，R0 → R1，β0 | trace 7502–7505，`raw/results/10.bin`、`raw/espresso/0009.bin`，独立 `public-replay.json.regular[0]` |
| Force | x 22.000 s，τ 22.001 s，关闭 β0，R2 接 h10，窗口 [16,25] | trace 8764–8765，R1 worker Close 8800、8804、8808、8812、8816，`raw/descriptor-b1-r1.bin`，独立 `public-replay.json.force[0]` |
| 同键 q10 | θ 22.000 s，τ 22.001 s，旧 R1 physical h11 公开完整，G 保持 h10 | trace 8763、8766–8767，`raw/results/11.bin`、`raw/espresso/0010.bin` 正文 0、证明 1，独立 `late[0]` |
| q11 | θ 26.040 s，τ 26.041 s，一次追加 padding h11–h15 和 R2 physical h16 | trace 11146–11153，`raw/results/16.bin`、`raw/espresso/0011.bin` 正文 0、证明 1，独立 `expected[10:16]` |

Force 用 q10 的 finalized 前缀提供 watermark。公开全序仍先执行 L1 控制。Force 从已接纳的 h10 建立新职责，q10 的空追加只推进公共重放位置。q11 的 h16 物理父块为 h10，新生成的 padding h11 由 h16 的证明覆盖，旧 R1 physical h11 有自己的正文和证明出处。

### M2 验收记录

- `generate.py --scenario force --check` 与投影测试通过，HTML 复现摘要为 `750a2dccd55835649f4b29a4b2f38baa0063f895267aa0761b9147fae73861c6`。独立核对覆盖 14 个 Anvil header、31 个 worker fixture header、12 条 Espresso 记录、60 条区域交付、12 份公开证明、13 个重放步骤和 16 个 G 条目。
- 独立负例覆盖 manifest、proof、缺失区域结果、reader 时间、receipt 和独立报告篡改。全部拒绝生成并保留既有页面字节。
- Firefox 158.0 在隔离目录独立打开每个 HTML。Force 验收覆盖收录及 finalized、A/B 交付、在途 Regular、同刻度 Force 与 q10 单步、q11 原子追加和游标回退。两页均通过播放、暂停与筛选。验证对象为封存的 seed 42 运行，证明字节对照了各自的独立验签报告。
