# M3 自动 Force 离线回放

正式回放从通过独立验收的 AF01 封存运行生成。输入是运行目录及其 checker 输出目录。样本页用于查看数据和交互，页面标明合同参数、规则推导和合成记录，证据状态单独标示。[冻结回放合同](contract-v1-zh.md)

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

正式页命令：

```sh
python3 -B poc/verification/m3-auto-force-replay/generate_auto_force.py \
  --bundle "$M3_BUNDLE" --independent "$M3_INDEPENDENT"
python3 -B poc/verification/m3-auto-force-replay/generate_auto_force.py \
  --bundle "$M3_BUNDLE" --independent "$M3_INDEPENDENT" --check
```

M1 和 M2 使用原有生成及投影测试入口，见 [M1/M2 生成命令与证据](../../web/m1-gfr-replay/README.md)。

合同样本通过 Firefox 158.0 单文件验收，覆盖十二个阶段入口、播放/暂停、单步、拖动、筛选和回退。样本含 40 帧，SHA-256 为 `d844692ae715af92565c985a04c234c542afd11cd350ef3f3bebeddc757484df`。投影测试检查 signer 前缀落后、q 增加但 θ 停滞、重复签名、不同 digest 和历史选区顺序。

正式入口所需输入包括通过 checker 的主运行 manifest、`report.json`、`verified-controls.json` 和 `public-replay.json`。样本提交时，AF01 正式封存运行仍待提供。开发轨迹用于检查字段形状。首次广播、广播确认与签名前观察依据使用各自的 trace 记录。若要逐次查看签名与广播前重新核验的内部输入，runtime 需单独记录每次 recheck 的观察值和理由。
