# Part III v1：第一轮可读性校对

审阅对象：`versions/part3_v1.tex`。先独立按正文顺序通读，再对照中文提纲检查组织与表达。以下位置均为该 TeX 文件行号。本轮只评估首次顺读能否理解，不判断协议或证明是否正确。主任务已确认 v1 PDF 为六页；本轮未另行核验排版。

## 总体判断

主线可以辨认：L1 定期冻结权重，session 提前固定排班，退出须等待旧职责终止与追责窗口。正文短句较多，区分选择参考块与执行 anchor 的例子尤其有用。不过，目前仍需要读者自行补齐若干记号，并在 §2、§4、§5、§6 之间拼出 authority 的流程。**尚不宜作为“无需回查即可顺读”的定稿；需要一次范围明确的表达修订。**

## 必须修复的理解问题

1. **补齐首次使用的记号与少量借用术语。** §1–2 的 `h(A_s)`、`lambda(E_s)`、`V_i(C_e)`、`w_o` 没有明确释义（48–52、93–108 行）；加权选择中的 `W` 应明说是输入权重之和（114–119 行）。罚没表的 `j` 与提款 checkpoint 的 `q` 完全未定义（216–225、308 行）。建议在 §1 增加两三句：`h(B)` 是 DATA 高度、`lambda(B)` 是该块已认证的 L1 引用高度；`V_i(C_e)` 是快照中的 region i 成员，`w_o` 是其中的冻结权重。`j`、`q` 若必须保留，在当地各用一句解释；若 `q` 后文不使用，可以删去该四元组。DATA/DRAIN、QC、opening、quit proof、Ready/Enter 不必重述 Part II，但首次出现时应给出足以读懂本节的一句功能说明。“View messages ... bind their view and position” 中第二个 position 也需明确是 stake position 还是消息槽位（209–212 行）。

2. **在选择公式前并列写清两个 anchor，并提早解释 plan 如何成为 authority。** 表格先说 plan 和 authority，§2 随即计算排班，但完整启动流程直到 §6 才出现（65–68、93–102、335–340 行）。建议在 §2 首次引入 regular successor 时直接写 `E_s=A_p`、`A_s=T_p`，接一句：“At p's start, only the selection reference is fixed. The execution anchor is bound when p completes.” 再用两句说明 plan 允许准备，完整交接证据与 Ready/Enter 才允许运行。这样读者无需先靠例子反推对象关系。

3. **把“实际余额”和“快照权重”的区别放到首次出现权重之前。** `B_o` 在对象表出现，`w_o` 在选区公式出现，但二者关系直到 §3 后半段才说清（59–62、108、185–190 行）。建议表后增加一句定义和一个短例子：“A position with snapshot weight 100 may later have escrow 70; certificates still use 100, while at most 70 remains available for penalties.” 这能解释全文最重要的金额区别，也可替代后文重复提醒。

4. **明确基础排班与实际可出块 leader 的关系。** §2 定义 `L_s^0`，§4 才说明初始 generation 也有 cutoff 禁任集，并用原公式对过滤后的成员重新计算（121–136、243–262 行）。读者容易先把 `L_s^0` 当成最终 leader。建议在公式后立即说明它是未应用该 generation 禁任集合的基础映射；实际 leader 使用同一输入和算法，但先过滤规定的禁任 key，并保留剩余成员原权重。随后明确 `g=0` 的集合来自 snapshot cutoff，`g>0` 来自 REC。这里需要补的是可执行的阅读顺序，不是扩大公平性承诺。

5. **交代取消申请这一已有引用的路径。** Join 段落出现 “pending-request cancellation path”，预算段出现 canceled entries，但此前没有定义可取消的申请，而 Exit 明确不可撤销（161–183 行）。至少用一句话指出哪些尚未纳入快照的 join 可以取消、由谁取消；避免读者误以为 exit 也可取消。这是把已有术语讲完整，具体规则由后续协议审查确定。

6. **让三类模块在组织上可见，并把轮转流程集中。** 现在 §4 把 slash 与 recovery authority 合写，§5 把提款与 Force bootstrap 合写，§6 才补 regular startup（207–267、270–330、333–349 行）。与提纲约定的“数据与接口—stake 管理—轮转与 authority”相比，读者需要来回拼接。建议保留六页预算，用三个明确的模块标题组织现有材料：A 放对象与公共接口；B 放队列、罚没及提款边界；C 集中 regular startup、recovery、Force bootstrap 和 Part II 衔接。可以移动现有段落，无需增加一套重复流程说明。

## 可选的表达和压缩改进

- **用一个贯穿例子补上快照采用时机。** 现有 138–143 行很好；再加一句：若执行 anchor 120 已引用更新的快照，既定 `(0,2)` 仍不重算，之后为 `(0,3)` 准备时才重新读取。若篇幅允许，再用同一节点说明“提出退出不取消已预排的职责”。相比新增抽象警告，这两句更容易让读者理解。
- **集中重复的边界提醒。** 可预测性／公平性分别在 131–134、264–267、379–380 行出现；外部最终性与 replay 检查在 299–308、368–375 行出现；资金退出不消除旧职责又在 §3、§5、§6 重述。保留规则首次出现处的必要限定，结尾只总结它的实际代价。不要删除关键限定，但可避免反复用 “not / never / does not prove” 抢占主流程。
- **替换少数过于抽象的说法。** “Late materialization” 可换成 “If the snapshot is published late”；“retains L1 on the recovery path” 可换成 “Each recovery generation waits for an L1 record”；“accepted lifetime” 应说清是 authority 仍有效的期间；“sound replay verifier” 首次出现时说清它验证的是公开历史按规则执行后的结果。“speculative sessions” 首次出现可解释为已经准备或运行、但输出尚未进入 G 的 session。
- **给时间预算一行口头读法。** 310–324 行虽定义了多数变量，却没有解释为何出现两份 `T_E`。加一句按顺序指出各项覆盖的事件，特别是哪一项对应 Force 关闭通知、哪一项对应结果交付；本轮不判断该总和是否足够。若无法用一句说明，应交下一轮协议审查核查。
- **保持六页应靠压缩重复和移动段落实现。** v1 已由主任务确认六页。无需为上述修订增加长附录，也不建议以缩小字号替代清楚解释；修订后应重新编译检查页数与表格换页。

## 可读性结论

**有条件通过初稿阅读，未通过“自包含、顺序阅读即可理解”的定稿标准。** 两个 anchor 的核心关系、权重冻结和批量退出的总体动机已经能读懂；优先修复记号、authority 的介绍顺序和模块交错，然后再进入协议问题检查。本报告不构成协议有效性或安全性结论。
