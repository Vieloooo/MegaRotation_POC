# M1-HEALTHY-SMOKE-1

固定 seed 42，正向启动，健康链路。15 个独立 OS worker，三个五人等权委员会，各 worker 使用独立的签名身份。一次运行覆盖 R0 physical h1–h10、R1 physical h11–h20、R2 physical h21。终点是 h21 经 Publisher 和 SimEspresso 的实际公开记录被 GFR 接纳进入 G。使用既有 L1 fixture 和改变执行状态的交易。每个逻辑 session 仅接线一个合格候选。

## 来源与行为

- 按 v10 Session Program §§3.4–3.12、Session Manager §§3.4、3.7–3.10、GFR §§3.2–3.4、4.2–4.3，以及 v2 §§3–6、9.1、12.3、13.3。Commonware 锁定 2026.10.0，Regional BFT gitlink 锁定 691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6。本 smoke 采用 37d17fd 所载普通 native profile 的 namespace、epoch 1、protocol/verifier 1、views 1–128 和 per-cid witness 绑定。
- 五个真实引擎产生原生 Simplex Finalization，验证 descriptor、namespace、epoch/view、签名 subject、真实 payload digest 和四个不同有效签名者。历史验签独立于读者当前的本地时间。确定性 V 检查完整正文、metadata、执行和 anchor ancestry，propose/verify 另用本地 A，certify 使用 V。分别留存 proof observation、完整区域交付、公开完整和 G 接纳。
- GFR 接纳 h10、h20 后产生 COMPLETE 和 SessionAssigned，Manager 消费有公共位置的实际重放并提交 Prepare、Enter、Close。真实后继 anchor 是 G 接纳的 h10、h20 及其执行状态。后继 Plan 以其前驱初始 anchor 为参考。窗口固定为 [1,10]、[11,20]、[21,30]。场景提供输入与调度。GFR 决定 cid 的接棒。
- descriptor/cid 由实际 Plan、anchor 和配置的 canonical 编码派生，受控提交安装。Ready 使用已校验的 anchor 材料和执行状态，绑定 worker/cid/anchor。节点分别 Ready、Enter。Close 先持久化 terminal，再禁止新签名，保留历史证明、正文和未完交付。
- 统一虚拟时钟，节点时钟偏差不超过 1 ms，Clock RPC 延迟独立。分别记录虚拟、本地协议、公共和墙钟时间。Grant 内收集完整 Ack，持久提交及 Ack 验证后释放网络、签名、timer、工作、生命周期与交付输出。签名出口检查 lease、stop 和 terminal。意外 worker 退出判本次失败。
- GFR 使用 finalized L1 fixture 和真实 finalized SimEspresso 前缀，按首次完整公开位置、固定窗口、截止与完整性的原规则重放。h10/h20/h21 在真实发布、公开收录和 G 接纳间保留精确区块关联。

## 证据和通过条件

封存候选 SHA、分支与 dirty 状态、锁文件和二进制 hash、工具版本、输入与 seed、启动命令、15 个 PID 与退出状态、各 worker 日志、原始 Grant/Ack 与持久 journal、签名与 Final 字节、正文、L1 记录、Publisher/SimEspresso 记录及 GFR/Manager 事件。逐次使用新的证据目录。

独立检查 15 个 PID、委员会身份、实际签名与 ancestry、h1–h21 物理链、h10/h20 驱动的两次接棒、真实 anchor/window、完整提交顺序和 terminal 签名守卫。通过后以“健康满窗口 M1 smoke”交付 exact SHA、复现命令与原始路径。

在既有 M1 worktree 集成，源代码限于 poc/ 内受控 runtime、接线和 smoke runner。完整 M1 合同继续独立跟踪 h9→h11、多候选、完整矩阵、官方同字节与恢复验收。
