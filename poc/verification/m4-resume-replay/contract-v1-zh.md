# M4 D 与 RESUME 离线回放投影合同 v1

回放基线 `b404818e18be62663e84a7fa580b151c686ad951`。依据 M1–M5 计划 §5/§7、v10 GFR §3.3–3.4、§4.6、§5、Authority §3.9、Batch Inbox §3.2–3.7、Architecture §3.2/§3.3 和 Manager §3.10。写入范围为 `poc/web/m1-gfr-replay/`、`poc/verification/m4-resume-replay/` 及相应回放文档。

## 输入与出处

- 先定义版本化回放输入和封存校验入口。本版本的字段是回放与输入适配之间的接口。记录正式运行的 manifest、candidate SHA、seed、checker PASS 及其 manifest 绑定，逐项核对所用原始输入及 checker 证据。每项显示值有原始文件、行/位置/消息序号或推导规则。正式运行发布后按其实际原始格式完成适配，核对各阶段和链条；证据缺失、校验失败、候选不一致时拒绝替换页面。
- 合同样本由标明来源的合成事件与规则推导组成。用独立的输出路径和醒目标识。样本要走通正常及时批次、后续错过窗口形成同高分歧、合法 RESUME、换后缀和新首块接纳。样本中的交易可用符号 ID，不能标为真实收据。测试向量另覆盖等高一致、仅高度落后、两条同 L1 block 的位置不同的 Cut、D 已超过目标、WAIT 后原位置重试和 ALIGN 后 HALT。
- L1/DA 保留 batch 提交、收录、receipt status、Inbox CursorAdvanced 的确切位置和游标、来源 gpos、采用区间、commitment、DA 对象的可读/校验时点、实际 D parent 与输出类型。L1 真实 header hash 和执行 fixture 的标识分别保存。接受但 DA 未读到要原位 WAIT，窗口不足也 WAIT。普通回退必须能追溯已处理的窗口和下一 origin。错过窗口的 deposit-only 与 CursorSet 立即填充分别标记。
- 保存 G、G′、D 的独立连续前缀及各自来源。G′ 是 Inbox 已接纳并经验证的认证源前缀，可在派生/读取切面就绪后从 CursorSet 对齐；以真实 G′ materialization 证据为准。共高比较按实际 native hash，从最早不同高度定位分歧；单侧高度领先/落后不构成 hash 分歧。保留 native hash、执行前/后 root、物理摘要、连接 anchor ID 和出处的类型区别。
- RESUME 收录要求同交易 `ResumeRecorded` 和 `CursorSet`，两条 log 各自保留完整 L1 位置。ResumeCut 用控制记录的完整 `p_res=(block,tx,log)` 索引，内容绑定同交易 CursorSet 处理结束时的 D、目标高度和 anchor，记录独立 Cut ID 或摘要、派生前缀、anchor 实际 L1 origin、native hash、连接 ID 与执行后 root。切面可超过目标，后续 D 可继续增长；G/G′ 采用时只取该切面到目标高度。连接 ID 按 v10 §3.2 的派生 anchor 规则验证，作为恢复首块物理 parent。
- GFR 接纳界限保存 `act=ts_L1+M_a`、覆盖所用 finalized L1 控制前缀及 Espresso `q/θ`、需要切面/DA/Plan 的 WAIT 和本次完成的真实虚拟 tick。RESUME 在一个公共位置原子完成后缀撤销与 DER 接纳和生命周期事件。按 native hash 求最长共同前缀，保留该段出处；旧分支职责关闭，新分支 `(β+1,0)`，可启动时 Plan 从无 source region 的 policy 循环选择，窗口 `[h_res+1,h_res+10]`、Ref 和 L1 floor 来自 D anchor。生命周期名称及顺序以真实同位置记录为准，保留 Manager 消费证据；缺输入 WAIT 时链和公开位置保持在该事件之前。严格过去的定时器给出对齐后 HALT，等键按控制、Expire、SessionEnd、Espresso 的全序处理。
- 新职责的区域 Final、Espresso 出版、公开完整 `q_avail`、G 接纳键与虚拟 tick 各自表示。核对新首块高度、cid、派生 anchor 的物理 parent ID、执行前 root、来源证明与接纳，切面 anchor、全局对齐和恢复生产各有阶段。

## 页面和验收

- 生成自含 HTML。使用共享虚拟 τ 轴和单调 trace seq；同一完整 GFR 转移仅提交一个可见状态。支持播放暂停、前后单步、拖动、筛选、按批次/分歧/切面/收录/WAIT/激活/首块跳转。页面并列显示某个共同高度的 G/G′/D、native hash 和出处，RESUME 详情显示旧/new G 后缀、保留区间、D 的独立末端和新职责。虚拟 tick、L1 timestamp/位置、公开键、可用的本地观测分别标示。
- 独立验证从规则推导预期，再核对样本和正式事件。覆盖切面身份/同事务绑定、已采纳 DA WAIT、最小共同前缀、拒绝中间替换、D 单调、anchor 两种 ID、首块父链和 root，以及错误/缺失证据时现有输出完整。回归 M1–M3 原生成和检查命令及页面字节。正式交付用已通过独立验收的 M4 主运行与 checker 输出， Firefox 验收播放与跳转。记录候选 Git SHA、生成及验收命令、证据路径和仍待锁定的实际字段。
