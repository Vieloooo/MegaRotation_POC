# Part III v1：authority 与 Part II 接口复核

范围：只读核对 `versions/part3_v1.tex`，对照当前 Part II v4.5 及 Part III 大纲。采用 proof-verifier 的 `Critical Error / Justification Gap` 分类；这是模块接口复核，不是整篇协议的安全证明。

**结论：主流程可行，但有三个应在定稿前补齐的 Justification Gap。未在本次范围内发现必须推翻设计的 Critical Error。**

## 必须补齐

1. **Genesis 的新 authority 没有明确落地。**
   - 位置：v1 第 93–101 行仅定义 regular successor；第 291–297 行定义 Force bootstrap；genesis 只在第 150、282 行提供 `C_0`、`m_0=0`。
   - 问题：首个 `(0,0)` 没有前驱，又不发生 Force。正文没有明确固定它的 `E_s`、初始 region/boot policy、排班上下文。`e(ell)=max{e:a_e<=ell}` 也需要 genesis reference 处非空，或明确 genesis 使用独立配置。大纲 §10.1 已要求初始 checkpoint 提供这些字段；Part II v4.5 第 423–446 行的初始化依赖旧 `AUTH_(0,0)`，而 v1 第 342–344 行已替换旧 authority 接口，不能不说明如何承接。
   - 影响：首次 authority 派生及其确定性结论尚无完整基例。这里需要补一个明确的初始化约定，不需要新协议。

2. **撤款 checkpoint 的四元组不是完整重放状态。**
   - 位置：v1 第 308 行：`The accepted checkpoint records (q,G,beta,m).`
   - 问题：若它将作为后续证明的起点，缺少 Part II 所需的 `Tail`（NEXT/PENDING、当前目标及开启边界）、`S_beta`、控制游标/待生效控制，以及本版新增 policy/config 依赖。Part II 第 1823–1832 行明确规定 `X=(Out,S_beta,kappa)`、`Out=(G,Tail)`；第 3519–3522 行另要求重放子位置；第 3546–3591 行保留可引用的公共对象。零输出 session 不会在 `G` 留下 session record（第 1739–1740 行），所以单有 `G` 不能替代 `Tail`。
   - 影响：不能仅从这个四元组验证下一次 Force 的目标、`r` 复用和完整控制历史。如果四元组只是提款合约保存的摘要，应明确完整已认证重放状态仍由证明绑定，摘要本身不是下一轮重放起点。正文“完整重放或其健全证明”的方向正确。

3. **新派生 authority 中 quota 与 BFT 参数的共同来源缺失。**
   - 位置：v1 第 67–68 行 authority 含 quota/区域 BFT 参数；第 342–349 行替换旧 authority 并要求它们固定；但 snapshot 字段（第 61–62 行）及派生公式没有给出这些字段的来源。
   - 问题：Part II 第 334–340 行由 `AUTH_(r,0)` 固定 `TB_r`、`F_i`、`Delta_i`、`K`。v1 明确了成员与权重的周期快照来源，却没有说明其余参数仍从哪个已认证对象或固定协议规则取得。仅声明 session 内固定，不能保证各验证者采用同一终点与门槛。
   - 影响：终点、admission、FINAL 阈值及新 authority ID 的重算仍有未定义输入。补共同来源与绑定关系即可；无需给出具体部署数值。

## 已正确处理，可接受

- **参考块与编号准确。** `E_s=A_p`；例子明确 `(0,2)` 使用 `A_(0,1)=T_(0,0)`。列表使用当前 `(beta,r)`，leader 哈希使用目标 `(beta,r+1,g,t)`。实际执行仍从 `T_p` 开始。
- **Recovery 未见循环依赖。** `REC(beta,r,g)` 由上一代的完整 Quit 及其 authority 依赖创建；leader 不依赖新恢复 anchor。首次有效记录冻结处罚集合，权重不改。可以等待外部恢复，不主张无条件进展。
- **历史验证与 Force 语义基本一致。** 旧签名用旧配置；Force 证明检查完整适用控制历史、真实激活和 current target，不能拿 Espresso 收录冒充接受。Force 的目标 `r` 明确沿用 Part II 零输出复用规则，没有改成恒定归零。
- **等待期论证有明确条件。** `t_c` 在验证有效 Force 之后；闭合传播、结果发布、额外证据收集及 L1 处理均计入，另要求链上时钟不得提前到期。`T_ev` 是新增服务假设，正文已承认它不由旧界限推出。没有用外部服务/故障假设之外的攻击否定本设计。

## 应保留为明确设计选择

- **Snapshot floor 与批量 Force 退休**超出大纲原来的“参考块处最新已激活快照”基础规则；可在 `lambda(E_s)` 仍旧时选择更高 epoch。v1 已在第 98–102 行声明例外，并在开头及成本段说明提款依赖 multisig Force、可能丢弃 `G` 之后的 speculative suffix。应保留这些说明，不能缩成“照常退出”。
- **Boot list 和 randList**替换了 Part II 的固定循环 `NextLive`；Force 不再天然保持“从 source region 后一个开始”的旧选择行为。正文已有新接口，不是内部矛盾；集成说明宜直接列明此项替换，避免读者以为仅 authority 索引变了。
- 第 35–37 行“end to all old duties”应限缩为旧 BFT 授权/运行职责结束。Part II 的保留数据与已欠发送义务不会被 Force 取消；v1 第 193–197、374–375 行已保留它们，因此这是措辞冲突，不是新的退休安全反例。
