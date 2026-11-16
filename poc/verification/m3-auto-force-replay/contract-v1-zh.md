# M3 自动 Force 离线回放投影合同 v1

回放基线为 `8a80e5a051fed08d92979c506ed4d634f369555e`。依据 M3 合同 `27fd202`、独立预期 `035f8ec`、v10 Authority §3.11–3.14、GFR §3.2–3.4/§4.2–4.5 和 PoC v2 §8.2–8.4。工作限于 `poc/web/m1-gfr-replay/`、`poc/verification/m3-auto-force-replay/`、现有只读回放解码器的兼容扩展和对应文档。

## 数据和出处

- 正式模式从主 chat 指定的已验收 AF01 封存运行读取 manifest、原始公共日志、区域结果、L1 收据、治理 trace 和独立 checker 输出。核对同一候选 SHA、运行配置与独立 PASS 的绑定，逐个校验所读输入的摘要。校验或证据不足时报出文件及字段，保留现有输出。只用真实记录生成正式页面。正式命令接受显式 bundle 与 checker 位置。
- 合同样本有显著的“合同样本”标识。使用合成记录、规则推导或计划值时显示来源，独立于正式输出路径。样本覆盖正向和关键错误形状，在没有正式 manifest 时可生成、播放和核验。合同中的 11.250 s 故障、约 13.25–13.4 s 的可签时段、ℓ7/14 s 收录、24 s 激活、anchor h1、R1 [18,27] 和 h2 过期 42 s 都按合同或由实际证据重新计算，正式页面没有预测代替实测字段。
- 从 trace 连接 5 个 R0 worker 的故障 tick、pid、cid 与随后公共证据。保留空 Espresso heartbeat 的 q、θ、finalized τ、message count，以及携带正文/证明的 publication。区域链的 physical block、G 已接纳 physical 和虚拟 padding 各有类型与证据引用。
- 每 signer 保留起码 advertised 与 verified L1/Espresso tip、上一观测 q/θ、replay pc、Tail/region/cid/head、x_progress、x_obs、差值和 T_stall、clock/input 状态、提议和 verdict。采样的本地钟字段独立显示。可查看各 signer 最接近签署的独立可签决策与其他 signer 的等待/签署状态，全部相关历史轮询可追溯。在途或未就绪时显示实际 reason。现场故障是解释用证据，签署理由来自 signer 自己的输入。
- 按 request、digest 联接不同成员的签名、达到阈值、已持久化的准备记录、广播、收录 receipt/log (block,tx,log)、finalized 与 reader 交付。标明治理 2/3 与区域 Final 4/5 两套门槛。治理签署和阈值按独立报告与实际收据核对，页面可定位验证出处。
- 按 `act=ts_L1+M_a`、实际 finalized Espresso watermark、L1 控制覆盖和 replay seq 显示 Force 公开激活。解释同键 L1 先于 SessionEnd/Espresso，activation 保留实际 G head、关闭旧 β。依据 active policy mask/order 与 source region 展示循环候选、有效 floor、历史高度 `ℓ_s=max(Ref.origin,floor)` 及实际 Anvil header/hash。验证 Plan 绑定的历史 basis、选择 R1 的原因及 first NEW descriptor/cid/window。若 runtime 与 checker 输出缺少必要的绑定或记录，明确列出最小缺口。
- 首个新结果的正文/证明、q_avail、接纳 public key 与虚拟 tick 分开。保留新增 G 区间、各 padding 与 physical parent/执行状态的区别，核对 checker 独立 G 条目。借用已通过独立 checker 的密码学验证结果，核对封存 proof/receipt bytes 与位置。

## 界面和验收

- 自含 HTML，离线显示 Anvil、Espresso、区域结果、G/GFR，故障/治理可另有轨道或面板。用一个虚拟 τ 游标、trace seq 决定同刻度顺序。已完整 GFR 转移原子呈现，public pc 与 G 高度独立推进。支持播放、暂停、回退、单步、拖动、筛选和关键步骤定位。显示 public key、L1 timestamp、local clock 与虚拟 tick 的含义。每个关键值提供 file、line/seq 或 JSON path、二进制记录位置。
- 验收故障后 heartbeat 持续推进且 G 暂停、至少两人独立观察达到门槛并对同一请求签名、真实收录后才公开激活、历史 Plan 选区、首个新结果由区域 Final 到 G。比对正式 AF01 数值和校验器输出，逐字节复现输出。用明确标注的夹具验收等待/陈旧前缀、q 增加但水位停滞、重复 signer、篡改或缺失证据。验收 M1/M2 旧生成命令与 HTML，保留轨迹锚点和证明数，浏览器核对回放与标签。
