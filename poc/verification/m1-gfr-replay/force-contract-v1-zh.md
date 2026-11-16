# M2 Healthy Force 与 M1 Regular 离线回放投影合同 v1

基线及 M2 受测提交为 `ea4d853bd8b62ee04c4e8e470550648242073950`。写入范围为 `poc/web/m1-gfr-replay/`、`poc/verification/m1-gfr-replay/` 和对应文档。规则依据 v10 GFR §3.2–3.4、§4.2–4.5、§4.7，及 [M2 合同](../../docs/milestones/M2-healthy-force-task-contract-zh.md)。M1 保留 [原合同](contract-v1-zh.md) 和生成命令默认行为。

## 证据输入与一致性

- Force 输入为封存 `merge-ea4d853b-seed42-force-run01`（seed 42），manifest SHA-256 `33a4c9d22845fe1150119113d04a56cb3a78759543b50a73064b963dd6561f9c`。核对所读文件大小和摘要。相邻 `-independent/` 的 report、controls、public-replay 与同一运行绑定。公开报告 SHA-256 `26779e78d59636825851d9e562cbcf0c293c1b21252fa5915a474929903a9e45`。输入缺失、解码失败、加入矛盾或篡改证据时生成失败。
- 从 `raw/l1/raw-rpc-bundle.json`、`receipts-by-case.json`、driver 或 hub 与 `raw/trace.jsonl` 联接指定交易成功 receipt、ForceRecorded log、实际 Anvil header 的 hash/parent/height/timestamp、收录与模拟 finalized、A/B reader 交付。A/B 可读用实际 Anvil hash。另由 `raw/l1.bin` 解码执行用高度/timestamp fixture，区分其 hash、空控制承诺与真实控制日志。receipt/control 语义采用已封存独立报告所验证的 ABI 和签名结果。展示交易及完整 L1 位置。`controls.json` 中请求的 floor 3 与激活后的 floor 6 分别标记。
- 从 `raw/espresso/*.bin` 和 `raw/results/*.bin` 解码 12 条 finalized PELG、正文和证明，按 `(cid,physical_id)` 联接 60 条区域交付与 12 条 PublisherRecorded，核对 q/消息位置、q_avail、时间和 exact proof bytes。按 13 条 GfrReplayRecord 核对公共位置和转换，含一个 Force 和一个无追加的 q10；另核对 11 份 GAccepted、5 份 GPadding、Regular/Force 生命周期及独立报告 16 个虚拟条目的身份、父链、native hash、state root 和证明出处。BFT 签名的有效性来自封存独立报告。各观察/规则值保留可定位的文件、JSON 路径、消息索引或 trace seq/行号。

## 同轴回放与规则

- 可独立用浏览器打开 `index.html`（原 Regular）及 `force.html`，生成端显式选择 Force，默认仍为 Regular。共用四类轨道、虚拟时间游标、前后单步、播放暂停、筛选与关键定位。实际同 tick 依 trace seq 排序。GFR 的完整转移按各 ReplayRecord 原子呈现。G 头与最后公共重放位置各自推进，Force 以及 q10 的空转移推进公共位置并保留 h10。旧 R1 区块在区域/Espresso 轨道有证据，G 轨道以排除或空转移显示。
- Force 详情并列 L1 收录 timestamp、公开激活键 `x=ts_L1+M_a`、实际 finalized/reader tick、Espresso watermark 与虚拟处理 tick。按 `q*=min{q:θ_q≥x}`，Force 前只处理 q0–q9。说明 L1 覆盖 x−M_a、Espresso 覆盖 x、Plan/anchor 验证材料、同键 L1 controls → Expire → SessionEnd → Espresso。当前 β=0、Tail NEXT，控制在 R0 生产时提交，R0→R1 仍在同 branch。R1 无已接纳 physical 贡献，Force 保留 r1 并从 `last(G)` 得 anchor/ref h10。β0 关闭并分配 β1/r1 的 R2。展示 BranchClosed(FORCE) 与 SessionAssigned，区分旧 R0 的已完成和 R1 的实际关闭证据。
- Force 窗口按 `h_min=max(A.h+1,slot(x+T_init)+1)`、`h_max=h_min+TB−1` 推导，展示输入和 [16,25]。同键 q10 旧 R1 h11 的证明及正文确实公开完整，已关闭的 cid 无责任。R2 physical h16 父块是 h10。q11 原子追加虚拟 [11,16]。h11–h15 为 padding，界面显示各自的 G 记录和来源 h16，旧 R1 physical h11 的 hash/evidence 与 padding h11 分别标明。显示整个新增段的最早 Expire（h11 29 s）、截止键和 q11 的判定，保留 h16 正文及 proof、origin 与 L1 覆盖的证据。

## 固定验收

| 位置 | 实际观测 |
| --- | --- |
| L1 | 提交 11.500 s seq 760，ℓ6 收录 12.000 s seq 943，finalized 13.500 s seq 2294，reader A 13.500 s seq 2295，B 13.510 s seq 2296。控制位置 (6,0,0)，receipt status 1。 |
| 在途 Regular | h10/q9，x 20.044 s、τ 20.045 s，seq 7502–7505。由 R0 接到 R1 [11,20]，当前仍为 β0。 |
| Force 和同键旧结果 | x 22.000 s，τ 22.001 s。seq 8764–8765 先关闭 β0、分配 R2，保留 h10；seq 8766–8767 后处理 q10 并排除旧 physical h11。 |
| 新分支进展 | q11，θ 26.040 s、τ 26.041 s，seq 11146–11153。padding h11–h15 与 R2 physical h16 一同追加。 |

验收两页的离线交互、生成确定性及异常输入。回归 M1 的 h10/q9、h20/q19 两次接棒、h21/q20 与全部 21 次接纳。记录候选 SHA、命令、证据链接和已封存单次运行的范围。
