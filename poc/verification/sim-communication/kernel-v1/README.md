# 独立 kernel 验收 v1

这是功能02的**组件**验收准备，完整功能的64条期望与SC-01–18仍保留。依赖分层授权允许先实现纯运输kernel；真实native/持久/RPC接线仍需要01/03等准确提交。

- [source-first](source-first.md)：新合同/API阅读前保存的来源预期。
- [expectations](expectations.md)：冻结前审阅、lead澄清、数值期望与adapter信任边界。
- [golden-vectors](golden-vectors.json)：独立hash/JSON/jitter/tie和四份custody record字面值。
- [vectors](vectors.json)：46条组件边界，全部NOT_RUN。
- [coverage](coverage.md)：K组件断言与完整SC保留项。
- `static-check.json`：只做准备一致性检查，不是runtime通过。

当前无Builder源码审阅、无Rust build、无native worker、无仿真。harness将放本目录自己的独立Cargo workspace，通过path dependency调用`poc/crates/sim-transport`公开API；在Builder释放唯一writer后补写，不修改Builder代码。独立预期先于harness与Builder实现。

未来运行必须使用命名候选SHA、02专属target/artifacts及全宿主heavy锁；Verifier/Tester使用同一候选，实际active native engines=0。组件结果不得提升为功能02 accepted或01真实持久性证明。
