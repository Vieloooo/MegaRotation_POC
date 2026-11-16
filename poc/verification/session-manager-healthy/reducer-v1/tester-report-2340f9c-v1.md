# H2 Manager reducer v1 组件测试

候选提交 2340f9c1c8cbc3bc4be57a5c18f884cc74390563，分支 codex/h2-session-manager-implementation，组件源码树 78a9250a93da0c5db78578f19951bad5c04f6b80。
候选测试前后工作区干净。源码、锁文件、合同和测试输入 SHA-256 见 summary.json。
独立验证包清单 SHA-256 为 c351fdfbbea1f318d99fbe113e3e0bf49fe7bead8e7a3904c33398aeaaf2b566。

## 测试结果

| 测试 | 状态 | 通过 | 失败 | 退出码 | 本机墙钟秒 |
| --- | --- | ---: | ---: | ---: | ---: |
| component | PASS | 8 | 0 | 0 | 46.575092 |
| independent-first | PASS | 27 | 0 | 0 | 0.486943 |
| independent-repeat | PASS | 27 | 0 | 0 | 0.543508 |

独立 harness 规范化 stdout 字节比较 **MATCH**。
首次 SHA-256 44086f0bf26c72a9d9e052cc14603a5b0aa216c8e75aba914694a2963e0355a0，复跑 SHA-256 44086f0bf26c72a9d9e052cc14603a5b0aa216c8e75aba914694a2963e0355a0。
规范化仅删除 Rust test result 行末的墙钟耗时。保留测试名称、顺序、手写 PASS 记录、测试数量和所有其它 stdout 字节。

## 复跑参数和证据

- rustc 1.95.0 (59807616e 2026-04-14)，cargo 1.95.0 (f2d3ce0bd 2026-03-21)。
- 固定手写 trusted-input fixture，随机 seed N/A。Cargo 默认 test profile，offline + locked，编译 jobs 2，测试线程 1。
- 每条命令独占同一 flock。各目录保存原始 stdout、stderr、规范化 stdout、命令 argv、测试结果、退出码和墙钟耗时。
- 墙钟耗时包含 flock 等待、Cargo 构建或缓存检查和测试。stderr 用于本机诊断。
- 首次启动在文档测试阶段调用系统 rustup shim，触发工具链下载后人工中止，启动器退出 130。原始输出保留于 tester-2340f9c-v1。此轮将 RUSTDOC 显式指向同一已安装的 1.95.0 工具链并完整运行。
- 第二次启动等待 H3 tester 释放共享锁，Cargo 尚未启动时中止以补设指定 PATH。记录保留于 tester-2340f9c-v1-2。
- 本次完整矩阵超时 0，未完成 0，命令未设置超时。

    cd /home/jy/.codex/worktrees/a6cf/megaroation_poc
    export RUSTC=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc
    export RUSTDOC=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustdoc
    export PATH=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:/usr/bin:/bin
    export CARGO_HOME=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/h2-manager/cargo
    export CARGO_TARGET_DIR=/mnt/reth/jy/megaroation_poc/.build/commander-jobs/h2-manager/target
    export CARGO_BUILD_JOBS=2
    flock /tmp/megaroation-poc-heavy.lock /mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/cargo test --manifest-path poc/crates/session-manager/Cargo.toml --locked --offline -- --test-threads=1
    flock /tmp/megaroation-poc-heavy.lock /mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/cargo test --manifest-path poc/verification/session-manager-healthy/reducer-v1/independent-harness/Cargo.toml --locked --offline -- --nocapture --test-threads=1
    flock /tmp/megaroation-poc-heavy.lock /mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/cargo test --manifest-path poc/verification/session-manager-healthy/reducer-v1/independent-harness/Cargo.toml --locked --offline -- --nocapture --test-threads=1

## 证据范围

固定输入提供本地时钟和公共位置值。测试名称、PASS 记录和固定输入是组件测试证据。本轮未运行 native 虚拟时钟或事件队列，未测量真实网络。
真实 worker、ReadyState/密码学验证、受控 grant/Ack、H1 保存与故障恢复、H3 公开日志/发布接线和 M1 端到端验收均未执行。早 h11 与晚 h14 仅覆盖固定窗口及引用相关操作。
独立 harness 的覆盖边界见其 COVERAGE.md。此轮没有独立触达 action ledger4096、descriptor/lifecycle 字节数上限或 dependencies64。
