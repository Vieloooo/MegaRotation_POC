# M5 Tester 运行说明

正式矩阵绑定干净并已复核的同一候选 SHA。新运行目录位于 M5 数据盘专属目录。先保存真实运行，然后由 Verifier 根据 L1、Espresso、DA、D、GFR 和原生帧独立复核。

| 覆盖 | 执行与证据 |
| --- | --- |
| AR01–AR03 | 15-worker 主运行、650 ms 墙钟暂停复跑和固定 θ 的正例。验证停止 finalized 与空 heartbeat 持续推进 q 的区别。对照组比较规范化虚拟轨迹、native frames、L1 收据与 driver 记录、DA、已接受输入和精确位置切面字节。 |
| AR08–AR09 | 真实 Anvil 迟收录和较慢 Clock RPC。用真实收据计算 act，检查超出预期收录区间后的实际切面对齐和 HALT。时钟偏差、RPC 延迟、本地墙钟和公开时间键分别记录。 |
| AR04–AR07、AR10 | 短停、实际 hash 分歧与授权、无目标、输入等待及生产求解向量。分歧用例在真实收录后按有界观察截止，记录授权理由和收据轨迹。核对各 signer 实际输入与读者缺口、共同高度 native hash、拒签原因、H_guard 分量和最小合法目标。用例层级及输入真实性以独立报告为准。 |
| AR11–AR12 | 同一 SHA 的 M1 到 h21、M2 到 h16、M3 到 h18、M4 到 h21 和相关重放向量。公开 Rust checker 同 SHA 构建。专项用例包含 3 项求解与理由、1 项恢复时钟、1 项 native 签名守卫和 10 项 M4 重放。 |

正常主场景跨过旧实例本地 stop 21 秒与公开 Expire 24 秒。排序恢复后，两个 signer 从各自已校验输入得出恢复事由及可行区间。按声明的收录区间 `[26,28]` 秒和实际 H_guard 求解 h30，使用真实收录确定 act，核对新窗口、派生 anchor 和 G 实际接纳首块 h31。迟收录与超时结论单独依正式输入判断。所有运行固定 seed 42。虚拟上界 60 秒，事件上界 1,000,000，单命令墙钟 watchdog 1800 秒。

主运行与旧回归 launcher 自行取得 `/tmp/megaroation-poc-heavy.lock`。直接 Cargo 测试和 checker 构建用同一把锁。Rust/Cargo 1.95.0，构建使用两 jobs、锁定离线。另记录 Foundry、solc 和 Python 版本与 SHA-256。回归入口改用 M5 任务的 Cargo、target 和 artifact 目录，复用各原 runner。真实 Anvil 使用本机回环。

```sh
cd /home/jy/.codex/worktrees/poc-m5-auto-resume/megaroation_poc
/home/jy/miniconda3/bin/python3 -B poc/verification/m5-auto-resume/tester/run_matrix.py \
  --candidate "$M5_CANDIDATE" --output "$M5_NEW_MATRIX"
```

输出应为 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m5-auto-resume/artifacts` 下尚不存在的目录。矩阵记录命令实参、退出和实际本机墙钟耗时、候选、清洁状态、seed、场景及子清单摘要。独立报告依据虚拟或公开时间评估时限。失败、超时和未执行阶段保留在对应清单与日志中。
