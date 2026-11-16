# M4 Tester 运行说明

正式矩阵在已复核的同一干净候选 SHA 上执行。每次使用 M4 数据目录下的新证据目录。全部真实运行固定 seed 42。冻结合同规定虚拟上界 60 秒、事件上界 1,000,000 和单命令本机墙钟 watchdog 1800 秒。记录各命令的本机墙钟耗时，协议时限从虚拟 tick、L1 块时间和 Espresso 公共键复核。

| 覆盖 | 运行与证据 |
| --- | --- |
| DR01–DR02 | 15 个 worker 的主运行及 650 ms 暂停对照。核对完整规范化虚拟轨迹、native frames、Anvil 收据、DA 对象和切面字节。记录批次、CursorAdvanced、D 输出、CursorSet、ResumeRecorded、实际 anchor、G 替换及首块 h21。 |
| DR03、DR05–DR07、DR10 | 生产入口的独立输入向量。保留正式 Anvil receipt/prefix、DA 完整性与定位证据、两个同块 RESUME 和回滚对照。检查早于、等于、晚于窗口边界，输入补齐前后的 D 状态及不回退。 |
| DR04、DR08–DR09、DR11 | 生产 GFR/Manager 入口的确定性向量。核对联合覆盖和原位 WAIT、不可变切面、旧分支终止、同键控制优先、低目标 HALT、A/B 同位置和历史 checkpoint。 |
| DR12 | M1 健康满窗至 h21，M2 真实健康单签 Force 至 h16，M3 真实 2/3 自动 Force 至 h18，以及受影响协议向量。既有三个场景均在该 M4 候选工作树运行。 |

主运行预期实际 h1 批次于 L1 h6 收录。h2 批次于 h8 收录，严格晚于 h7 原生窗口末端。D h1 的普通交易重执行接上自己的父状态，h2 回退后与 G 的同高度 native hash 分歧。RESUME 预定在 L1 h9 收录，绑定同一交易的 CursorSet。GFR 在 act 28 秒原子对齐到实际历史 D(h20)，R2 最晚在 30.999 秒 Ready/Enter，首个合格 h21 在 61 秒 Expire 前进入 G。实际计划、锚点、分支、origin 与执行状态由公开输入独立计算。单纯出现控制收据或 SessionAssigned 不能代替最终 G 接纳。

主运行和暂停组、M1–M3 runner 自行取得 `/tmp/megaroation-poc-heavy.lock`。直接 Cargo、Forge 或 Anvil 向量命令用同一锁包起来。工具为 Rust/Cargo 1.95.0、Python 3.12.8、Foundry 1.0.0-stable、Solidity 0.8.20。Cargo 两 jobs、锁定离线，目录在 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m4-resume`。原回归入口通过 wrapper 改用 M4 缓存，保留原 CLI、ABI 和工具。

```sh
cd /home/jy/.codex/worktrees/poc-m4-resume/megaroation_poc
/home/jy/miniconda3/bin/python3 -B poc/verification/m4-resume/tester/run_matrix.py \
  --candidate "$M4_CANDIDATE" --output "$M4_NEW_MATRIX"
```

M4_NEW_MATRIX 为 M4 artifacts 下尚不存在的目录。真实 Anvil 使用本机回环。矩阵记录 argv、退出码、SHA、工作树状态、工具及锁文件 SHA-256、seed、profile 和子证据清单。独立复核从子证据目录读取原始记录。未运行、超时或不完整的用例分别记录和解释。
