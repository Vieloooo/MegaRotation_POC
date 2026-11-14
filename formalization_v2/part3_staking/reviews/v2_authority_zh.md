# Part III v2：authority 接口收口复核

范围：核对 `versions/part3_v2.tex` 是否关闭 v1 的三个接口缺口，并检查修改引入的集成问题；不重新证明 Part II，不扩展新功能。

**结论：三个原有 Justification Gap 均已关闭；本次未发现新的 Critical Error 或必须补充的协议规则。完成已计划的 v3 局部表述修正后，可以在本次模块设计范围内定稿。**

## 必须落实的局部修正

- **Force 状态继承措辞。** 第 323–324 行 `No plan or BFT state crosses the closed branch boundary` 范围过宽。应落实已计划的修改：旧计划、投票、锁和计时器不继承；保留 anchor 对应的可执行状态及必需历史材料仍可保留/使用。否则容易与第 319–322 行从保留的 `G` 末块启动、以及 Part II 的数据保留职责混淆。这是边界表述修正，不要求更换设计。

## 已关闭的缺口

1. **Genesis 基例完整。** 第 270–275 行固定初始 policy、`S_0`、`m_0`、`E=A=A_0`、boot list 和完整初始重放状态；`lambda(A_0)>=a_0` 保证快照查询有定义。首个 session 不再暗中依赖不存在的 regular predecessor。
2. **重放 checkpoint 完整性明确。** 第 236–243 行要求完整适用历史与健全重放证明，checkpoint 明确绑定 `G`、NEXT/PENDING tail、status、policy、minimum、控制游标、待激活更新、准确 Espresso 位置及 source commitments。配合第 263–264、366–367 行的保留/可用性要求，已消除把四元组摘要当完整重放起点的问题。
3. **公共 BFT 参数来源明确。** 第 75–79 行由 genesis 固定 `TB`、`K`、`Delta_i` 和 `phi_i`，按冻结权重计算 `F_i`；第 284–285 行要求 authority 绑定这些参数，第 379–381 行说明参数变更需要显式扩展。终点与验签门槛不再是未定义输入。

## 可接受且应保留

- `E_s=A_p`、执行 anchor `A_s=T_p`、regular 列表的 current turn 与 leader 的 target turn 仍一致；Force 单独采用 Part II 的 target 与零输出复用规则。
- Recovery record 只依赖前一 generation 的 Quit 与历史 authority；新 opening 不参与恢复 leader 的选择，未引入循环。
- 第 328–333 行已明确列出 authority 索引、candidate/boot list 及 recovery record 对旧接口的替换，且保留 Part II 的 quit/lock/timer 规则。
- floor 例外、批量 Force 依赖、speculative suffix 丢弃与服务/时钟假设均明确保留，未把条件性退出写成无条件期限。
- 提前定义 `G`、简述 opening/checkpoint 的既定 v3 改动有助于独立阅读；不构成新增协议要求。

本结论限于上述 authority 与集成接口。它不认证 Part II 全部证明、合约实现或部署参数。
