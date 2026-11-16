# M3 Auto Force Tester 准备

正式构建和仿真使用同一已复核、干净的候选 SHA。证据文件夹均从 M3 专属目录新建。预检已核对 M3-AUTO-FORCE-1 合同和独立预期。表格记录验收计划。

| 用例 | 运行入口 | 首要原始证据 |
| --- | --- | --- |
| AF01 | seed 42，真实 15 worker 自动 Force，R0 在 h1 入 G 后离线 | 真正的 5 个预期退出、各 signer 的独立公开视图及签名、L1 h7 的 14 s 收录、active policy 与历史 Plan、R1 h18 的证书和 G 接纳 |
| AF02 | 同 SHA 同种子及输入，增加 650 ms 墙钟暂停 | 完整规范化虚拟轨迹、原始 native frames、实际 L1 交易收据的逐字节比较。协议和公开键保留微秒精度 |
| AF03 | 同入口健康对照 | seed 42 至少达到 h10 regular，持续 G 进度和零 Force 签名/交易 |
| AF04–AF06 | 生产治理入口的公开流及决策向量 | signer 交付的水位和已验证前缀、缺数据 WAIT、q/θ 的真实推进情况、阈值两侧、Tail/时钟/预算 |
| AF07–AF08 | 真实 Anvil 门槛与请求幂等，治理持久重载 | 收据与签名身份及包、无效执行时零控制记录、替换组重试、未知广播重载的原参数/命令 ID/tx hash |
| AF09–AF11 | 独立历史窗口、重放和 worker 准备向量 | 按生效历史求得的 policy 和 Plan、ℓs 支持前缀、WAIT/NONE/REJECT、A/B 等位置和历史 cid |
| AF12 | M2 现有健康单签入口和既有独立检查 | h10 作为真实 anchor、R2[16,25]、h16 进入 G。保留单签 ABI 和 M2 原 profile 的实测 |

AF01 检查所有主要时点。虚拟 fault 为 11.250 s，L1 公布时间来自收录块。预期 act 公开键为 24 s，恰和旧 R0 的 SessionEnd 同键。真实执行取事件前的 G h1 与实际历史 ℓs 7。新窗口 [18,27]，准备最晚 27.999 s，并要求首个合格结果严格早于 h2 的 Expire 42 s。旧分支 GFR Close 及离线前持久签名记录分别查证。五个离线 worker 的预期退出与十个存活 worker 到观察截点后的退出分别记录。

性能诊断记录命令和各进程本机 wall 时间。协议时限以虚拟微秒和公开位置复核。暂停比较只根据已评审的 trace schema 排除 PID、procfs 或墙钟诊断字段。保留完整 trace、原始 frames、原始 L1 bundle 和实际命令。避免跨沙箱用宿主 procfs 推断进程状态，运行内的 spawn、受控 I/O、fault 与 wait 状态是可复查证据。

M3 实时、公开流和 L1 负例的 CLI 随候选 runner 确定。尚待入场的向量记录为未运行。每例保留命令 argv、退出码、超时、种子、profile、before/after SHA 与工作树状态、工具版本和二进制哈希。

## 同 SHA M2 回归

M2 已有 runner 将所有构建指向固定的模块常量 JOB。此处的 wrapper 在 M3 候选 checkout 加载原 runner，仅将 JOB 指向 M3 专属数据目录。原 CLI、候选检查、采集、ABI 核对、仿真及 host lock 都由原 runner 执行。wrapper 的 argv 会保留在 manifest。执行时使用 Python 3.12.8。系统 Python 3.10 缺少所需的 hashlib.file_digest。真实 Anvil 的 loopback 需要相应宿主网络权限。

```sh
cd /home/jy/.codex/worktrees/poc-m3-auto-force/megaroation_poc
M3_JOB=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/m3-auto-force
M3_PY=/home/jy/miniconda3/bin/python3
M3_RUST=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin
"$M3_PY" -B poc/verification/m3-auto-force/tester/m2_regression.py \
  --candidate "$M3_CANDIDATE" --seed 42 --wall-pause-ms 0 --output "$M3_NEW_M2_RUN"

export CARGO_HOME="$M3_JOB/cargo"
export CARGO_TARGET_DIR="$M3_JOB/target"
export CARGO_BUILD_JOBS=2
export PATH="$M3_RUST:$PATH"
flock /tmp/megaroation-poc-heavy.lock "$M3_RUST/cargo" build --locked --offline \
  --manifest-path poc/verification/m2-healthy-force/public/Cargo.toml
"$M3_PY" -B poc/verification/m2-healthy-force/check.py \
  --repository "$PWD" --candidate "$M3_CANDIDATE" \
  --raw "$M3_NEW_M2_RUN/raw" --receipts "$M3_NEW_M2_RUN/raw/l1/raw-rpc-bundle.json" \
  --driver-rpc "$M3_NEW_M2_RUN/raw/l1/driver-trace.json" \
  --hub-l1 "$M3_NEW_M2_RUN/raw/l1/hub-l1.jsonl" \
  --public-checker "$CARGO_TARGET_DIR/debug/m2-force-independent" --output "$M3_NEW_M2_VERDICT"
```

设置上面的候选和两个尚不存在的输出路径。M2 wrapper 和任何内部持锁的 M3 runner 直接启动。独立 Rust 构建、测试、合约检查及没有内锁的重型仿真使用同一宿主 flock。Cargo 使用两 jobs 和固定的 1.95.0，`--locked --offline`。实测宿主 Python 3.12.8、coincurve 20.0.0、Anvil/Forge 1.0.0、solc 0.8.20。
