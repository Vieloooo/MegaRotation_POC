# M1 Regular Handoff 离线回放

用浏览器打开 [index.html](index.html)。整个页面封装在一个 HTML 文件中，可复制到其他机器离线打开。页面打开时定位 h10/q9，可用「起点」从头播放。使用播放、暂停、前后单步或时间轴游标浏览，点击两个接棒按钮可定位到职责切换。点击 G 链里的接纳或接棒可查看条件、计算值和证据引用。

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

- 生成检查逐字节复现单文件，SHA-256 为 `f4a303ef1e0c8d9f09dde6793a7cbd65ae1b7e2f5df274b065a574e31000750e`。负例覆盖二进制截断、尾随字节、原始文件摘要错误及独立证明不匹配。
- 独立读取核对 31 个 L1 header、21 条 Espresso 日志、21 次 G 接纳、105 条区域交付引用与两次接棒。证明位置与字节逐项对照封存报告。
- Firefox 158.0 用 `file://` 打开隔离目录中的单文件。通过播放、暂停、前后单步、拖动、轨道筛选、两次接棒跳转、h21 跳转、全部 21 个接纳面板，以及检查 L1 证据时保持当前职责。浏览器未加载外部页面资源。
