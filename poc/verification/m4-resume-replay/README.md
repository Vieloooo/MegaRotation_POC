# M4 D 与 RESUME 离线回放

[打开正式 DR01 回放](../../web/m1-gfr-replay/resume.html)。回放基线为 `b404818e18be62663e84a7fa580b151c686ad951`。页面把 GFR 的 G、Inbox 的 G′ 与独立派生的 D 并列在同一虚拟时间轴上。点击共同高度可以比较原生 hash，检查类型、来源和执行后 root。事件详情保留实际父块、origin、执行前 root 和证据出处。[冻结投影合同](contract-v1-zh.md) · [实际组件字段与主运行的最小联接](source-fields-zh.md)

用浏览器打开 [合同样本](../../web/m1-gfr-replay/resume-sample.html)。这份可交互样本使用合同设定、规则推导与合成事件。生成、核对和投影测试从回放工作树根目录执行。

```sh
python3 -B poc/verification/m4-resume-replay/generate_resume.py --sample
python3 -B poc/verification/m4-resume-replay/generate_resume.py --sample --check
python3 -B poc/verification/m4-resume-replay/test_projection.py
```

在批次阶段联接成功 Inbox 接纳、认证 gpos 和 DA commitment。已接纳但缺少 DA 的输入由 D 在该输入位置等待。完整窗口结束且无可采用批次后，D 才可生成 deposit-only。共高分歧按 native hash 定位。高度较低的 G′ 仍显示其已物化前缀和实际 cursor。

RESUME 收录的两条日志分别标出规范位置和可用的 RPC 日志索引。切面详情显示控制位置、同交易 CursorSet、完整历史前缀和目标 anchor。切面及公共覆盖齐备后，GFR 在一帧中关闭旧职责、保留共同 native 前缀、替换其余后缀并选择新职责。D 保持自身派生进度。新首块用派生 anchor 的连接 ID 作为 physical parent，用目标块的执行后状态作为执行前态。首个区域 Final、公开完整与 G 接纳分别有事件。

## 正式输入

生成器的正式入口接受主运行封存目录和绑定同一 manifest 的独立 PASS 输出。正式输入来自候选 `108158364da5b7157b88c65b7dd6b1f56e506a77` 的 seed 42 封存主运行。manifest SHA-256 为 `5c48ee94161db8c425d52ace3ded8392b0f1366b0bef6edce1ad3a95a1098de1`。

```sh
M4_BUILD=/mnt/reth/jy/megaroation_poc/.build/commander-jobs
M4_BUNDLE="$M4_BUILD/m4-resume/artifacts/tester-1081583-seed42-run01/primary"
M4_INDEPENDENT="$M4_BUILD/m4-resume-replay/artifacts/dr01-1081583-seed42-independent"
(cd "$M4_INDEPENDENT" && sha256sum -c SHA256SUMS)

python3 -B poc/verification/m4-resume-replay/generate_resume.py \
  --bundle "$M4_BUNDLE" --independent "$M4_INDEPENDENT"
python3 -B poc/verification/m4-resume-replay/generate_resume.py \
  --bundle "$M4_BUNDLE" --independent "$M4_INDEPENDENT" --check
```

- 运行封存目录提供 `manifest.json`、`manifest.sha256` 和 `files` 中列出的原始数据。每个使用的文件核对 `bytes` 和 `sha256`。
- 正式 DR01 直接读取独立 checker 的 `report.json`、`public-replay.json`、`independent-D.json`、`verified-l1-headers.json` 和 `verified-cuts.json`。核对 `candidate` 和 `manifest_sha256`，保留原报告的字节摘要。每帧保留真实 seq/tick 及封存证据位置，必要的子步骤注明来自同一次观察。
- 已有 checker 另行提供版本化投影时，入口还可使用报告以 `resume_replay_sha256` 绑定的 `resume-replay-v1.json`。该投影的标识为 `M4-D-RESUME-REPLAY-1`。
- `resume_cut` 保留精确 `cut_bytes_path`。生成器对照原始字节摘要与规范位置、配对 CursorSet、目标、派生前缀、目标 anchor/执行状态。原始输入或关联出错时拒绝替换页面。

## 兼容性回归

M1–M3 的原生成与校验命令使用各自已经封存的运行与独立证据，均通过。M1 核对 21 次接纳及两次接棒，M2 核对真实 L1、同键 Force、旧块排除与 h16 接纳，M3 核对正式 AF01 和合同样本。 [M1/M2 命令](../../web/m1-gfr-replay/README.md) · [M3 命令](../m3-auto-force-replay/README.md)

## 合同样本验收

样本包含 112 帧，SHA-256 为 `12609576e07cac2ecc11110ba51c787c177c312dfb58f2eaba99b9b85a959bce`。样本按 DR01 计划参数示范 h1 的及时批次和 DA 等待、h2 的窗口回退及共高分歧、位置 `(9,0,1)` 的历史切面、公开键 28 s 的 RESUME，以及 R2 从 h20 继续生产 h21。样本中的具体虚拟时点、hash、签名与收据标识是合成数据。

Firefox 单文件验收通过播放、暂停、拖动、前后单步、轨道筛选和各阶段跳转。在分歧帧选择 h1 时，三链 native hash 一致。选择 h2 时，G 与 D 不同，G′ 尚未物化到 h2。在 RESUME WAIT 帧，G 头及公开位置保持，随后在一个提交帧对齐到 h20。六项投影测试与独立负例覆盖原始证据摘要、切面位置及身份、等待、原子后缀、过期和新首块父链等关联。

## 原始字段适配

正式页从封存原始文件和独立报告生成，共 359 帧，SHA-256 为 `dfe2070231f300b1ce5aada538b5a19ea596acf0a5880eb191dcd24859f3e475`。两次独立输出生成的页面逐字节一致，Firefox 单文件通过阶段跳转、选择共高、单步、回退、拖动、播放/暂停和筛选。G′ 使用成功接纳的封闭批次所认证的历史 G，以及配对 CursorSet 的历史 Cut 重建。重建帧使用相关 reader 观察的虚拟 tick 和 seq，作为来源可用的位置。单独展示 G′ 的实测物化时间或 DA 下载完成时间需要对应事件。主路径的原位 GFR WAIT 演示来自合同样本，实际运行有记录时再显示该等待。

## DR01 证据对照

| 阶段 | 正式观测 | 原始位置 |
| --- | --- | --- |
| 及时批次 h1 | 虚拟 12.000 s 收录于 ℓ6，A 在 13.500 s 取得，D 采用实际 DA frame，native hash 与 G(h1) 一致 | Hub trace seq 869、1089、2538–2539，D trace 行 41，`raw/derivation/{batch-h1,accepted-A-6-0-0}.json` |
| h2 分歧及迟到 | 完整 ℓ7 在 A 可读后，D 于 15.500 s 回退 h2，保留共同前缀至 h1。迟到批次于 ℓ8 成功收录，D 处理时记录为 old。重建的 G′ 可物化认证的 G(h2) | Hub seq 4206–4209、4427、5878–5879，D trace 行 51、59、61，`raw/derivation/accepted-A-8-0-0.json` |
| RESUME 和历史 Cut | 虚拟 17.000 s 提交，ℓ9 timestamp 18 s 收录。CursorSet 为 (9,0,0)，ResumeRecorded 为 (9,0,1)。reader A 在 19.500 s 到达该历史切面，D 已有 h1–h3，CursorSet 立即填 h4–h20，目标 origin 9 | Hub seq 5275、6100、7546–7547、7566，D trace 行 70，`raw/derivation/cut-A-9-0-1.json` 与 B 的同位置文件 |
| 原子提交 | q89 为覆盖公开键 28.000 s 的 finalized Espresso。GFR 在 28.001 s 关闭旧分支，保留 h0–h1、撤销原 G h2–h17，采用历史 Cut 的 h2–h20。live D 仍为 h21。R2 获取 `β1/r0` 窗口 [21,30] | Hub seq 14589 起的 Reset/DER 证据，提交 seq 14609，通知 seq 14610，`raw/espresso/0089.bin`，`raw/descriptor-b1-r0.bin` |
| 恢复首块 | h21 物理父连接是 Cut 内 D(h20) 的派生 anchor ID。首个区域交付于 31.046500 s，q102 首次公开完整，公开键 31.051500 s，G 于 31.052500 s 接纳。末端 G h21、重建 G′ h20、D h22。D(h21) 是回退块，和新 G(h21) 的 native hash 不同 | Hub seq 16678、16814 和附近的 PublisherRecorded/GfrReplayRecord，`raw/results/21.bin`、`raw/espresso/0102.bin`，独立 `public-replay.json` 和 `independent-D.json` |

派生 anchor 的连接 ID 为 `d7c9ba9865ca33cc8331a0b02fd3d7882ae2597f8b45d1403850c612d949b60e`，目标原生 hash 为 `68aedbfc1b3aaee54f210b01d430a25baae3a575192180f680117faa609ae69f`。两个值及 target post-state 在 Cut 详情中分别列出。

用准确后端候选和该次独立 Rust checker 二进制复跑 DR01。输出使用新目录。

```sh
M4_REPO=/home/jy/.codex/worktrees/poc-m4-resume/megaroation_poc
M4_PUBLIC="$M4_BUILD/m4-resume/target/debug/m2-force-independent"
M4_NEW_VERDICT=/tmp/m4-dr01-independent-new

/home/jy/miniconda3/bin/python3 -B "$M4_REPO/poc/verification/m4-resume/check.py" \
  --repository "$M4_REPO" --candidate 108158364da5b7157b88c65b7dd6b1f56e506a77 \
  --run "$M4_BUNDLE" --public-checker "$M4_PUBLIC" --output "$M4_NEW_VERDICT"
```
