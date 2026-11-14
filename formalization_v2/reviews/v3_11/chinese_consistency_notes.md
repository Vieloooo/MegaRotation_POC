# v3.11 中文 companion 一致性记录

本记录是中文修订与机械检查记录，不是独立 safety／liveness verdict。作者初检时 v3.11 等待独立审查；主任务已在三份新报告完成后同步中文状态，并用“实际收件事件”区分请求到达与 BFT FINAL receipt。最终指纹另见 companion/verification.json 和 build_verification.json。

交付：[v3.11 完整中文阅读版](../../sync_hotstuff_rr_part2_rotation_v3_11_zh.md)。本次未修改英文、v3.9 中文或冻结 Model。v3.10 中文仅更新导言状态段，其编号导读及全部正文保留。

## 两项审查发现与本版修订

- [CE-OC](../v3_10/fresh_progress.md)：v3.10 的 OC collector 明确组证后广播，但收到别人组出的合法 OC 后，未规定强制 relay。选择性 adoption 可取消剩余正确 selection，关闭又可阻止 Ready。因此 universal OC formation 的局部命题失败，下游证明链不能直接认证；这不是已经证明最终 ingress-to-G 定理为假。
- [JG-I1 与 T-I1](../v3_10/fresh_interfaces.md)：原“请求到达 region”缺少 recipient-level 定义；另有 indicator 的排版缺反斜杠。用户随后明确原含义：“sent to a region means that every party in the region will receive it.” v3.11 从全体收件和原 fault bound 推出至少一个正确成员的实际完整认证 receipt；没有新增客户端 deadline，也不要求每个未来 roster 在 preparation 开始前知情。英文 v3.11 指标公式已修正，中文沿用预期的 `\mathbf 1`。

v3.10 的新 front status 明确记录上述历史 findings 和后续用户澄清；旧正文作为历史候选保留，未悄悄改成 v3.11 算法。

## 实际 source links

中文导言按实际 input chain 链接：main、input_selection、requests、ready_bundle、source_step、retirement 和 liveness 使用 v3.11；未改的 ownership、public_requests 使用 v3.10。英文还共享 v3.10 receipt_kernel／exports 与原 health／control／figures。没有生成或引用不存在的全面 v3.11 副本。

原通信范围继续引用 [v3.10 scope](../../revisions/v3_10_scope_zh.md)；新增 ingress 含义引用 [v3.11 变更说明](change_summary_zh.md)，并在中文导言、§9.10 premise 和 proof 中展开。

## 修改映射

| 位置 | 更新 |
|---|---|
| 导言 | v3.11 文件链接、first-possession relay、两项原模型含义、审查中的候选状态 |
| §6.3 Handler／store | InputPrepared 先保存 relay duty 再通知 entry；PublicStore／restart 保存证书、flag 和 cursor |
| §6.4 共享准备 | 同一个 K 保存 OCRelay；worker closure 不取消发送，Ready 不是 relay trigger |
| §6.8 退役 | FinishClose 实际移交每项 OC multicast 的完整副本及 cursor；Closed／Terminated 后继续 node-level relay |
| §8.1 OC | local formation／receipt 共用首次持有规则；每 K 一次初始 multicast；不获取 nonselected bodies；两种情况的形成证明 |
| §8.1 例子 | H0、H1 正确，B Byzantine；B withholding nonselected body，先关闭再选择性给 H1，H1 仍向 H0 relay |
| §9.7 准备 | 使用修复后的 possession／no-possession 形成证明，不把收到证书等同于已经广播 |
| §9.10 ingress | 每个 region party 最终收到，加原 fault bound，推出一个正确 authenticated member 的 finite actual receipt |
| §9.11／§10 | 保留原 timing／publication 范围与待审状态；扩展 OC relay 的存储及退出职责 |

## OC 修订逐项核对

1. 本地形成与外部接收用同一个 first-valid-OC rule。不能把 collector 的完整-offer 条件强加给已合法 OC。
2. 第一次 CheckOC 成功，先持久保存精确 certificate、所有 recipients 和 dispatch duty，再取消 selection 或通知 Ready。实际发送继续；没有增加“全部发送完成才准取消”的等待。
3. OCRelay[K] 绑定首次选定副本与 cursor。重复输入恢复旧工作；另一合法同 K OC 不创建第二次 first-possession multicast。
4. 该转发副本不成为 exclusive validation choice；原 no-local-OC-veto 保留。
5. Relay 只要求 selected body 和原 CheckOC 的完整依赖，不等待 nonselected keys 的 bodies。
6. Closed、stale handoff、worker cancellation、Terminated 不抑制 node-level relay；不产生 BFT vote、entry、timer 或新 offer signature。
7. Worker 退出前实际持久移交完整 certificate 和 unfinished dispatch 给已持有两者的 owner；未完成 retrieval task 不够。
8. 若已有一个正确 OC holder，其 finite relay 给所有正确 targets；否则没人能凭 OC 取消 job，全部正确 offers 最终形成第一张 OC，再适用同一 relay。
9. H0/H1/B 例子保留 decisive details：F=1，B 的较大非空 key 没有 body，OC 选 H0 的 x0，Closed 阻止 Ready，但 H1 实际发 OC 给 H0。
10. 原 R0、HYDRATED／offer coverage、session-first DATA、REPORT、independent ANCHOR 和模型正文不变。没有给 entire certificate transfer、native work 或新 pool scan 一个单 Delta 界。

## 检查与指纹

以下由最终机械检查追加。编译、Markdown 格式检查与本地链接检查均不是 proof certification；没有生成中文 PDF 或运行 model checker。


- Section 2 bytes: **9080**; unchanged against v3.9 and v3.10.
- Section 2 SHA-256: `db9f1d7de0a6c9896b436f476fef2bc37148ec4684118ed793523f8930f6b4c0`.
- Numbered section order: **58**, unchanged.
- Code fences and display-math delimiter pairing: passed.
- Companion local links: **20** checked. 作者初检时 PDF 待生成；主任务已生成并逐页检查，最终本地链接无缺失。
- Whitespace check: `git diff --no-index --check` has no diagnostics (exit 1 denotes version differences).
- v3.10 current SHA-256: `b7182eaa5eeaf568c2a1f022b09483d4f671335dcf1e5c9634f624add715eed3`; only the front-status paragraph was edited, with an actual before/after body equality assertion.
- v3.11 companion: **2208 lines**, **285355 bytes**; SHA-256: `ea0897fcb48cfc42a982f2a98f44b1e7036c926ffbfc234b22e467559d6fc37d`.

### Source fingerprints at this check

| Input | SHA-256 |
|---|---|
| `sync_hotstuff_rr_part2_rotation_v3_11.tex` | `de2e36905fef1e2de7bcc53f0cd8f4a95eca92e4561d347b33bc260e7dc58bef` |
| `sections/rotation_v3_11_input_selection.tex` | `4213d7389ce8e349770881ddd8f72c6eceb3b6cb0278ce7261ac4f7e3a0dd9d5` |
| `sections/rotation_v3_11_liveness.tex` | `893eea6862ac027d30900126cac3701c1fbb4c4a47fad4f043bd5547b1cfd73b` |
| `sections/rotation_v3_11_ready_bundle.tex` | `b409be852eebe5cde1aa2dc60355d21fda84aed1c97e66fab2a556652d67da37` |
| `sections/rotation_v3_11_requests.tex` | `bb99d0f45f6622953c60647a3168dc5026d5c5ca05572f35c5efd8c97cdcecf4` |
| `sections/rotation_v3_11_retirement.tex` | `13da135289a71990ccae8d54df2c503420979809cc2db52cb94a252efc939f5f` |
| `sections/rotation_v3_11_source_step.tex` | `1a85e314c929428b561ade6e1e7549aee1f48c210579bd2de674d73da5a500f7` |
| `sections/rotation_v3_10_ownership.tex` | `5952df7ab66e12fdbbdbb48f8b6c3c4dd3beeca03ac6d11db9e79ecd97319f1c` |
| `sections/rotation_v3_10_public_requests.tex` | `1be92deba4065fe976b82238f5d0ea827eb0abe66bd0c2703bfd983f96a711da` |
| `sections/rotation_v3_10_receipt_kernel.tex` | `5891df938ef061b57ccae04bef91a3df907b4e71ebd7542468883d03ec5e38ae` |
| `sections/rotation_v3_10_exports.tex` | `5b5cdd9745d255cc30961bd551217f8a828960edc2cdf871e2c7a6ae7db4098e` |
| `reviews/v3_10/fresh_progress.md` | `93755a429e83103053adfa1bc7a86dbf409f80c2300db4cdbcd344cfc6bb18d1` |
| `reviews/v3_10/fresh_interfaces.md` | `ebb388f551acec1fbc65a4bb5fa8e9439119b69de79040c19f5698806082ec6b` |
| `reviews/v3_11/change_summary_zh.md` | `08717bfb4897e9da5addc4bcb7bac04a725cf09a281a7f08aad3bee2b9a51aa6` |
| `revisions/v3_10_scope_zh.md` | `e40730b059a8ad45aa8f349b1bb04bf3ba2c6ac3d00737e8443ecc007d37fedd` |
