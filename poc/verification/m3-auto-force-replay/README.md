# M3 自动 Force 离线回放

[打开正式 AF01 回放](../../web/m1-gfr-replay/auto-force.html)。输入是通过独立验收的封存运行及其 checker 输出。样本页用于查看数据和交互，页面标明合同参数、规则推导和合成记录，证据状态单独标示。[冻结回放合同](contract-v1-zh.md)

从回放 worktree 根目录运行：

```sh
python3 -B poc/verification/m3-auto-force-replay/generate_auto_force.py --sample
python3 -B poc/verification/m3-auto-force-replay/generate_auto_force.py --sample --check
python3 -B poc/verification/m3-auto-force-replay/test_projection.py
```

用浏览器打开 [合同样本](../../web/m1-gfr-replay/auto-force-sample.html)。L1、Espresso、区域结果、治理和 GFR 共用虚拟时间轴。播放器、筛选和阶段按钮均可离线使用。

## 核对顺序

1. 先定位五个 R0 worker 离线。空 heartbeat 的 q 和公开键继续推进，G 仍停在故障前头部。点击每名 signer 的轮询记录，可查看它自己的公布/已验证前缀、已处理位置、进展键、观测键、停滞差值和判断理由。
2. 进入签名与 L1 阶段，核对相同请求和 digest 的不同成员签名、门槛、持久准备、广播、实际收录、finalized 和可读时刻。治理 EIP-712 门槛和区域 Final BLS 门槛各自显示。
3. 进入 Force 和 Plan 阶段，按收录 header 计算公开激活键，并查看实际处理 tick。新 policy 从 Force 应用开始生效。按历史 L1 高度及 hash、region order、enabled mask 和 source region 核对下一职责。
4. 定位新结果及 G 接纳，查看 proof/body 和首次公开完整的位置。G 的 physical 区块及虚拟 padding 使用分别标明的记录，显示 physical parent 与虚拟前序状态。

正式页的证据目录和命令如下。

```sh
M3_BUILD=/mnt/reth/jy/megaroation_poc/.build/commander-jobs
M3_MATRIX="$M3_BUILD/m3-auto-force/artifacts/tester-7f4ade9-seed42-run01"
M3_BUNDLE="$M3_MATRIX/outage"
M3_INDEPENDENT="$M3_BUILD/m3-auto-force-replay/artifacts/af01-7f4ade9-seed42-independent"
(cd "$M3_INDEPENDENT" && sha256sum -c SHA256SUMS)

python3 -B poc/verification/m3-auto-force-replay/generate_auto_force.py \
  --bundle "$M3_BUNDLE" --independent "$M3_INDEPENDENT"
python3 -B poc/verification/m3-auto-force-replay/generate_auto_force.py \
  --bundle "$M3_BUNDLE" --independent "$M3_INDEPENDENT" --check
```

M1 和 M2 使用原有生成及投影测试入口，见 [M1/M2 生成命令与证据](../../web/m1-gfr-replay/README.md)。

合同样本通过 Firefox 158.0 单文件验收，覆盖十二个阶段入口、播放/暂停、单步、拖动、筛选和回退。样本含 40 帧，SHA-256 为 `f10baecbc25798de8f1659a6074b8cd8bf8648d713d6ab02ccd229590641b5a7`。投影测试检查 signer 前缀落后、q 增加但 θ 停滞、重复签名、不同 digest、历史选区顺序、原始输入缺失或被篡改，以及失败时保留已有输出。

## AF01 正式证据

封存的自动 Force 主运行使用 seed 42 和候选 `7f4ade983ea78ebdff73d17b3cefc31609e7b507`，运行 manifest SHA-256 为 `5045ce4c259cd41661a033fa051f6e9c855f700243591d71b51e784a3579f32e`。回放按原始 trace seq 排序同一虚拟刻度内的事件。

| 步骤 | 实际记录 | 证据位置 |
| --- | --- | --- |
| 故障与水位 | G 在 q5 接纳 h1，τ 11.073 s。R0 的五个 worker 于 τ 11.250 s 离线。q6 heartbeat 的 θ 为 11.250 s，finalized τ 为 11.251 s。 | `raw/trace.jsonl` seq 737–739、824–829，`raw/espresso/0005.bin`、`0006.bin` |
| signer 观察与门槛 | 三名 signer 各自在 τ 13.300250 s 已验证到 q14 / θ 13.250 s，最近合格进展键为 11.072 s，差值 2.178 s。三份签名在 τ 13.310250 s 记录，两份进入实际签名包。 | trace seq 1076、1078–1083，独立 `report.json` 的 `governance`，`verified-controls.json[0]` |
| 提交、收录与可读 | τ 13.320250 s 首次广播结果未知，τ 13.330250 s 按同一交易哈希确认。Anvil 在 timestamp 14 s 的 ℓ7 收录。模拟 finalized τ 15.500 s，reader A/B 分别于 τ 15.500/15.510 s 收到。 | trace seq 1084–1087、1152–1153、1319–1321、1345，`raw/l1/hub-l1.jsonl`、`raw/l1/raw-rpc-bundle.json`，独立 `report.json.l1_stages` |
| Force 及历史 Plan | L1 位置 (7,0,0) 给出 act=24.000 s，GFR 于 τ 24.001 s 从 q57 得到覆盖并先处理 Force。旧 R0 的 SessionEnd 同键，G 保持 h1。mask 0b110、order [0,1,2]、source R0 和历史 Anvil ℓ7 选出 R1，职责为 β1/r1、窗口 [18,27]。 | trace seq 2293–2296，`raw/espresso/0057.bin`、`raw/descriptor-b1-r1.bin`，独立 `public-replay.json.force[0].historical_plan` |
| 新结果入 G | R1 physical h18 在 τ 28.035 s 区域交付，q74 / θ 28.040 s 首次公开完整，τ 28.041 s 原子接纳虚拟 h2–h17 和 physical h18。h18 的物理父块为 h1，执行前态来自虚拟 h17。 | trace seq 4955、5090–5109，`raw/results/18.bin`、`raw/espresso/0074.bin`，独立 `public-replay.json.expected[1:18]` |

## 验收和复验

独立 AF01 checker 的 `report.json` 绑定运行 manifest。独立检查同时覆盖 `outage-pause650`，规范化虚拟轨迹、原始 frames 和真实 L1 收据一致。回放核对 75 条 finalized Espresso 记录、73 条空 heartbeat、540 条 signer 轮询、76 个公共重放位置、2 个物理 G 接纳和 16 条 padding。九类原始输入或独立结果篡改、缺失用例被拒绝，已有输出字节保留。

正式页含 767 帧，SHA-256 为 `8be3c23d4a41ef9d23891abf921a13d73b8bb55de7e42280f1194d3727f4cc3a`。Firefox 158.0 从隔离目录打开单文件，验收故障、水位、门槛、提交、收录、finalized、reader、Force、历史 Plan、新区域 Final 和 G 接纳，及播放、暂停、筛选、单步和拖动。每条 signer 轮询保留在时间序列及历史表中，轨道上用里程碑标出决策。

用保持在运行候选的 M3 工作树复跑同一独立 checker。命令中的 Rust checker 二进制应该来自同候选的锁定离线构建。为输出指定一个新目录。

```sh
M3_REPO=/home/jy/.codex/worktrees/poc-m3-auto-force/megaroation_poc
M3_PUBLIC="$M3_BUILD/m3-auto-force/target/debug/m2-force-independent"
M3_NEW_VERDICT=/tmp/m3-af01-independent-new

/home/jy/miniconda3/bin/python3 -B "$M3_REPO/poc/verification/m3-auto-force/check.py" \
  --repository "$M3_REPO" --candidate 7f4ade983ea78ebdff73d17b3cefc31609e7b507 \
  --mode outage --run "$M3_BUNDLE" --repeat "$M3_MATRIX/outage-pause650" \
  --public-checker "$M3_PUBLIC" --output "$M3_NEW_VERDICT"
```

签名前和广播前再次核验的内部观察需要额外的 recheck 日志才能逐次展示。页面中的判断依据来自已记录的每名 signer 的 GovernancePoll、签名和提交事件。
