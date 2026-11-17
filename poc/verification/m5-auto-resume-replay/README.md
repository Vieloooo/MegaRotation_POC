# M5 自动 RESUME 离线回放

[打开正式 AR01/AR03 回放](../../web/m1-gfr-replay/auto-resume.html)。回放基线为 `e44b6e8e77aeb1aa3895c14109a57ff3894d7252`。同页可以选择 finalized 停止和 θ 冻结。播放器、筛选、阶段定位与证据详情使用共享虚拟时间轴。合同样本的计划参数、合成记录和推导结果均有标识。[投影合同](contract-v1-zh.md) · [证据字段](source-fields-zh.md)

从回放 worktree 根目录执行。

```sh
python3 -B poc/verification/m5-auto-resume-replay/generate_auto_resume.py --sample
python3 -B poc/verification/m5-auto-resume-replay/generate_auto_resume.py --sample --check
python3 -B poc/verification/m5-auto-resume-replay/test_projection.py
```

[打开合同样本](../../web/m1-gfr-replay/auto-resume-sample.html)

## 回放阶段

1. 先看 Espresso finalized 前缀与本地 stop。finalized 停止时，q 与 θ 均保持。θ 冻结场景中，q 可以增长并产生等键处理记录。公共 WAIT 标出被阻塞的事件位置和所缺覆盖。本地时钟阈值、实际虚拟 tick 与公共定时器用各自的证据。
2. 水位恢复后，先按公开全序处理 Expire。看各 signer 的可签判断、各自验证的前缀及 head、同请求达到门槛的 `t_eligible`，再检查 `H_guard` 三路上界、预计收录区间及最小合法高度。
3. 比对签名包、真实收录和按实际 L1 timestamp 计算的 act。使用准确 RESUME 位置与同交易 CursorSet 查询历史 Cut。GFR 原子对齐到目标，live D 保持自身进度。
4. 核对 Ready/Enter、首个 Regional Final、公开完整位置和首块进入 G。切面的原生 hash、执行后 root 与新首块的 physical parent 分列。

## 正式输入

原始输入适配直接读取 M5 封存 manifest、同运行的 checker PASS 及其独立公共、D、切面和 L1 证据。原报告与所用证据各自保留摘要。正式输入来自后端候选 `d6a36431a42cb6e155e9d192bea37f154f742eaa` 的 seed 42 封存 AR01 与 AR03。入口也支持报告以 `auto_resume_replay_sha256` 绑定 `auto-resume-replay-v1.json` 的版本化投影。各帧反查 manifest 中的原始记录 seq/tick，核对 Cut 精确字节、位置、交易和执行状态。可多次指定配对的 `--bundle` 和 `--independent`，每对作为场景在同一个离线页中切换。

```sh
M5_BUILD=/mnt/reth/jy/megaroation_poc/.build/commander-jobs
M5_MATRIX="$M5_BUILD/m5-auto-resume/artifacts/tester-d6a3643-seed42-run01"
M5_PROOF="$M5_BUILD/m5-auto-resume-replay/artifacts/ar01-ar03-d6a3643-seed42-independent"
(cd "$M5_PROOF/stopped" && sha256sum -c SHA256SUMS)
(cd "$M5_PROOF/frozen" && sha256sum -c SHA256SUMS)

python3 -B poc/verification/m5-auto-resume-replay/generate_auto_resume.py \
  --bundle "$M5_MATRIX/primary" --independent "$M5_PROOF/stopped" \
  --bundle "$M5_MATRIX/watermark-frozen" --independent "$M5_PROOF/frozen"
python3 -B poc/verification/m5-auto-resume-replay/generate_auto_resume.py \
  --bundle "$M5_MATRIX/primary" --independent "$M5_PROOF/stopped" \
  --bundle "$M5_MATRIX/watermark-frozen" --independent "$M5_PROOF/frozen" --check
```

M1–M4 继续使用既有封存运行与独立报告验证原生成和投影命令。[M1/M2](../../web/m1-gfr-replay/README.md) · [M3](../m3-auto-force-replay/README.md) · [M4](../m4-resume-replay/README.md)

## 样本及回归结果

样本含三条可选择的合成时间线，共 236 帧，SHA-256 为 `c9d6cd11e24520e7c81fe89af3ceb43cee2e1697f3f2057376b0cae1aae598ea`。使用后端合同 `712c018` 的计划参数。各签名、哈希和样本阶段的局部 tick 属于合成记录。

Firefox 158.0 离线核对了三个场景、前后单步、时间轴游标、筛选和播放/暂停。finalized 停止样本的 q/θ 与 pc 保持，冻结样本的 q 与同键 pc 前进，D 独立增长。两条恢复样本均可看到本地 stop 与公开 HALT 分离，并逐步核对 signer、求解、签名包、签署时的收录区间、公开 act 和新首块。九项投影测试核对时间预算、最小整数、恢复条件、覆盖、签名组、迟收录/区间端点、打包往返和错误封存输入。旧页的 M1–M4 生成和投影回归通过。

## 正式轨迹和证据

AR01 的 manifest SHA-256 为 `2e95f780d1a52682fc2224bd347d67175c6e7fad9ad10b3e088cec04aac3bf77`。AR03 为 `9bed4904de54b3babe433b22fbf9af2093a36b2fd4a73d5323fd7fdec68f2d86`。两条轨迹的原始虚拟坐标如下。seq 指各自 `raw/trace.jsonl` 的记录。

| 阶段 | 实际观测 | AR01 / AR03 证据 |
| --- | --- | --- |
| 停止或冻结，L1/D 独立推进 | τ 11.250 s 起，原水位 θ 11.072 s。AR01 在 q5 保持 finalized。AR03 产生 53 个等键空 finalized，观察 WAIT 时 pc 已到 q57。τ 24.100 s 的两次 WAIT 都缺覆盖 Expire 的 θ 24 s，D 已到 h6 | OrderingFault seq 972，两条 WAIT seq 8862 / 10454，`raw/espresso/` 及 `raw/derivation/trace.jsonl` |
| 本地 stop 和公开 HALT | 五个旧 worker 在自身时钟 `C_m=21 s` 停止，虚拟 tick 分布在 20.999–21.001 s。服务在 24.500 s 恢复，GFR 在 τ 24.501 s 处理公开键 24 s 的 Expire 并置 HALT，G 仍为 h1 | LocalStopped seq 8608–8642 / 9810–9847，公共 Expire seq 8896 / 10518，旧本地步骤关联 `AckCommitted` |
| 可签与求解 | `t_eligible=24.761 s`。理由为 HALTED_D_ADVANCED。取 `H_G=1,H_D=16,H_Inbox=1`，所以 H_guard=16。区间 [26,28] s 得唯一可行 L2 timestamp 40 s，目标 h30，signed origin ℓ13 | ResumeEligibility seq 8980 / 10602，实际 GovernancePoll 和 ResumeInputEvaluation，独立 `report.json.governance` |
| 入包、广播和收录 | 收录交易通过 2/3 治理验签。虚拟 24.820250 s 广播，自 t_eligible 起 **59.250 ms**，预算为 120 ms。Anvil ℓ13 timestamp 26 s 收录 | Signed seq 8994 / 10616，Submitted seq 8999 / 10621，L1Included seq 9203 / 10825，`raw/l1/raw-rpc-bundle.json`、`raw/l1/governance-transaction.json` |
| 历史 Cut 与 G 采用 | ResumeRecorded 位于 (13,0,1)，CursorSet 位于 (13,0,0)。D 在 27.500 s 的历史切面到 h30。公开 act 36.000 s，GFR 在 τ 36.001 s 采用 Cut 并分配 R2 [31,40]。live D 此时已到 h31 | GReset seq 11105 / 12727，GFR 提交 seq 11135 / 12757，覆盖 q52 / q105，`raw/derivation/cut-A-13-0-1.json` |
| 恢复首块 | R2 h31 经实际 Regional Final 和完整公开后进入 G，接纳键 41.083 s，虚拟 tick 41.084 s。新物理块 parent 对应 D(h30) 派生 anchor 的连接 ID。接纳时 G/G′/D 分别为 h31/h30/h33 | PublisherRecorded seq 14476 / 16098，GAccepted seq 14477 / 16099，q73 / q126 的 PELG 与 `raw/results/31.bin` |

## 验收和独立复验

正式页共 5,581 帧，其中 AR01 2,499 帧、AR03 3,082 帧。SHA-256 为 `128d83ea6002510c208c6bda85ba28c4a2154f333000a75c267a716b634eb541`。签名者实际记录的 2,190 / 2,667 个观察均可沿原始 seq/tick 定位。共享链状态与来源在 HTML 内按索引引用，选择场景时恢复该场景的时间轴。使用 Firefox 158.0 验收离线加载、场景切换、单步、拖动、播放/暂停、筛选和原子 RESUME 前后状态。

两条正式轨迹均通过独立 checker。AR01 的 +650 ms 墙钟暂停复跑通过规范化虚拟轨迹、原生帧、实际收据、DA 和 Cut 的一致性检查。九项投影测试及 M1–M4 的原生成检查/投影测试通过。独立检查对真实封存证据副本施加八类篡改，验证了拒绝生成及已有输出保留。

使用在运行候选的后端工作树和同候选已锁定构建的独立 Rust checker。两条独立报告输出使用新目录。

```sh
M5_REPO=/home/jy/.codex/worktrees/poc-m5-auto-resume/megaroation_poc
M5_PUBLIC="$M5_BUILD/m5-auto-resume/target/debug/m2-force-independent"
M5_CANDIDATE=d6a36431a42cb6e155e9d192bea37f154f742eaa
M5_NEW_PROOF="$(mktemp -d /tmp/m5-independent-new.XXXXXXXX)"

"/home/jy/miniconda3/bin/python3" -B "$M5_REPO/poc/verification/m5-auto-resume/check.py" \
  --repository "$M5_REPO" --candidate "$M5_CANDIDATE" --mode stopped \
  --run "$M5_MATRIX/primary" --repeat "$M5_MATRIX/primary-pause650" \
  --public-checker "$M5_PUBLIC" --output "$M5_NEW_PROOF/stopped"
"/home/jy/miniconda3/bin/python3" -B "$M5_REPO/poc/verification/m5-auto-resume/check.py" \
  --repository "$M5_REPO" --candidate "$M5_CANDIDATE" --mode frozen \
  --run "$M5_MATRIX/watermark-frozen" --public-checker "$M5_PUBLIC" \
  --output "$M5_NEW_PROOF/frozen"
```

G′ 标为依据成功接纳的批次和历史 Cut 重建。正式等待帧显示 Hub 观察时的 pc、请求覆盖位置以及缺失输入。服务故障结束、完整 θ 恢复、公共状态完成转移和首次可签分别使用原始记录。
