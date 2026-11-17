# M5 自动 RESUME 离线回放

回放基线为 `e44b6e8e77aeb1aa3895c14109a57ff3894d7252`。同页可以选择 finalized 停止和 θ 冻结。播放器、筛选、阶段定位与证据详情使用共享虚拟时间轴。合同样本的计划参数、合成记录和推导结果均有标识。[投影合同](contract-v1-zh.md) · [证据字段](source-fields-zh.md)

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

原始输入适配直接读取 M5 封存 manifest、同运行的 checker PASS 及其独立公共、D、切面和 L1 证据。原报告与所用证据各自保留摘要。已按两条 M5 开发轨迹完成映射预检。入口也支持报告以 `auto_resume_replay_sha256` 绑定 `auto-resume-replay-v1.json` 的版本化投影。各帧反查 manifest 中的原始记录 seq/tick，核对 Cut 精确字节、位置、交易和执行状态。可多次指定配对的 `--bundle` 和 `--independent`，每对作为场景在同一个离线页中切换。

```sh
python3 -B poc/verification/m5-auto-resume-replay/generate_auto_resume.py \
  --bundle "$M5_BUNDLE" --independent "$M5_INDEPENDENT"
python3 -B poc/verification/m5-auto-resume-replay/generate_auto_resume.py \
  --bundle "$M5_BUNDLE" --independent "$M5_INDEPENDENT" --check
```

M1–M4 继续使用既有封存运行与独立报告验证原生成和投影命令。[M1/M2](../../web/m1-gfr-replay/README.md) · [M3](../m3-auto-force-replay/README.md) · [M4](../m4-resume-replay/README.md)

## 样本及回归结果

样本含三条可选择的合成时间线，共 236 帧，SHA-256 为 `c9d6cd11e24520e7c81fe89af3ceb43cee2e1697f3f2057376b0cae1aae598ea`。使用后端合同 `712c018` 的计划参数。各签名、哈希和样本阶段的局部 tick 属于合成记录。

Firefox 158.0 离线核对了三个场景、前后单步、时间轴游标、筛选和播放/暂停。finalized 停止样本的 q/θ 与 pc 保持，冻结样本的 q 与同键 pc 前进，D 独立增长。两条恢复样本均可看到本地 stop 与公开 HALT 分离，并逐步核对 signer、求解、签名包、签署时的收录区间、公开 act 和新首块。九项投影测试核对时间预算、最小整数、恢复条件、覆盖、签名组、迟收录/区间端点、打包往返和错误封存输入。旧页的 M1–M4 生成和投影回归通过。
