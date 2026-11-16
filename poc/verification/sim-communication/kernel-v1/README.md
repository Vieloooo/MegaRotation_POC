# 独立 kernel 验收 v1

最新执行记录：[653a065修复复核](review-653a065.md)，46/46 tests PASS；待lead提交新增harness后复核最终命名SHA，再交Tester。历史 [bc4111a独立审查](review-bc4111a.md) 的两项失败及原始证据保留。下列准备文件保留冻结前来源，准备阶段的NOT_RUN不代表后续执行结果。

这是功能02的**组件**验收准备，完整功能的64条期望与SC-01–18仍保留。依赖分层授权允许先实现纯运输kernel；真实native/持久/RPC接线仍需要01/03等准确提交。

- [source-first](source-first.md)：新合同/API阅读前保存的来源预期。
- [expectations](expectations.md)：冻结前审阅、lead澄清、数值期望与adapter信任边界。
- [golden-vectors](golden-vectors.json)：独立hash/JSON/jitter/tie和四份custody record字面值。
- [vectors](vectors.json)：46条组件边界，全部NOT_RUN。
- [coverage](coverage.md)：K组件断言与完整SC保留项。
- `static-check.json`：只做准备一致性检查，不是runtime通过。

冻结准备阶段未读Builder源码或构建；此后已在本目录建立独立Cargo workspace，通过path dependency调用`poc/crates/sim-transport`公开API。独立预期先于harness与Builder实现。当前仍无native worker或完整系统仿真。

未来运行必须使用命名候选SHA、02专属target/artifacts及全宿主heavy锁；Verifier/Tester使用同一候选，实际active native engines=0。组件结果不得提升为功能02 accepted或01真实持久性证明。
