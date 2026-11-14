# PoC v2：xuelab 单机环境与多 Agent 搭建说明

日期：2026-11-14。本文件区分已检查的机器状态、交接时准备的内容，以及新 chat 需要实际执行的搭建工作。

## 1. 路径与现状

| 项目 | 值/状态 |
| --- | --- |
| SSH 别名 | `xuelab_jump`，沿用已有跳板配置；不复制私钥 |
| 远端用户/主机名 | `jy` / `server`，本次已连接确认 |
| 现有项目 | `/mnt/reth/jy/regional_BFT_sim` |
| 新 PoC checkout | `/mnt/reth/jy/regional_BFT_sim/poc/sequencer_rotation_v2` |
| 磁盘 | `/mnt/reth` 本次约剩 1.5 TB；执行大型任务前重查 |
| CPU | 本次系统报告 48 logical CPUs；不要据此默认同时开 48 个编译任务 |
| Rust | 现有 `.tooling` 中有 1.95.0 工具链；远端非交互 PATH 中没有 rustc/cargo |
| Git/Python/Node/npm | 远端 PATH 中可见 |
| Foundry/protoc | 远端 PATH 中未发现；需要后续固定版本安装或确定已有其他路径 |
| Docker | 二进制存在，server version 未取得；daemon/权限尚未确认 |
| Codex | PATH 指向现有代理启动器；本次没有更改或重启 daemon/代理 |

检查在交接时完成；工具是否可编译/运行仍要在新 chat 中验证。不能把历史 bootstrap 测试结果当成本次 PoC 测试。

## 2. 本次快照怎样保存

本机保留现有 `clock-skew` 和未提交改动。交付使用 Git tracked 基线内容加工作区差异、新文档，并附精确状态与 SHA-256。接收端独立 Git init/commit 保存导入状态，不在远端父仓库 checkout/reset。

材料包括 v10、此前引用的规范/会议文档、`megaeth_specs` 文档快照、v1/v2 设计、本次交接/搭建说明，以及经过 PoC 适配的 agent 配置。不传输 `.tooling`、`target`、node_modules、SSH/登录凭据或原仓库 `.git`。

原 `megaeth_specs` 在快照中作为普通文档树物化；上游 revision 写入 provenance。原 `.gitmodules` 可作为来源记录保留，但接收端不自动网络初始化 submodule，也不把本快照当作上游可直接提交的 native-spec 工作树。

接收端 `SHA256SUMS` 核验成功后才建立 baseline commit。后续修改先建立自己的 `codex/` 分支；所有正式实验记录实际候选 commit 与工作区 dirty 状态。

## 3. 模型与角色

依据现有 Regional BFT README 的分工，主 chat 的 reasoning effort 按用户要求设为 **ultra**；子角色保持各自职责默认值，不把“主 chat Ultra”扩大为所有子 agent 都 Ultra。

| 角色 | 模型/effort 配置 | 所有权 |
| --- | --- | --- |
| 主协调 | 继承项目配置的 GPT-6 Astra；新 chat 显式请求 ultra | 接口、lockfile、集成、任务合同、最终结论 |
| `bft_spec` | GPT-6 Astra / xhigh | 指定 docs、接口合同、来源/假设映射 |
| `bft_builder` | GPT-6.1 Sol / high | 分配的源码与测试文件；一个路径一个 writer |
| `bft_verifier` | GPT-6 Astra / xhigh | 独立验收期望、只读审查、单独的验证路径 |
| `bft_simulator` | GPT-6.1 Sol / high | 已验证 revision 的场景运行与 artifacts |

角色文件放在接收 checkout 的 `.codex/agents/`，主配置为 `.codex/config.toml`，协作规则在根 `AGENTS.md`。现有项目的 `regional-bft-conformance` skill 随包复制，其原始 BFT 语义由根 `DESIGN.md` 指向对应固定来源。

配置允许的子 agent 数沿用 README 的上限 6；实际工具可能只开放更少并发槽。按实际可用数排队，不通过额外 CLI 会话规避限制。先一个 writer，加一个 verifier 准备独立案例；不要同时让多名 builder 修改 shared types、proto 或 lockfile。

如果宿主没有自动加载命名角色，可在可用 subagent 工具中显式采用相同模型、effort 与角色指令，并记录差异；不悄悄替换模型。没有工具支持的角色不能声称已经启动。

## 4. 首次 preflight

新 chat 的每次命令明确使用新 checkout 作为 workdir，不依赖最初由 Codex saved project 提供的父仓库 cwd。

```bash
cd /mnt/reth/jy/regional_BFT_sim/poc/sequencer_rotation_v2
git status --short
git log -1 --oneline
sha256sum -c handoff_provenance/SHA256SUMS
```

然后保存：OS/arch、CPU/memory/disk、Git 版本、已用端口、可执行文件版本、当前锁定依赖，以及父仓库已有 dirty 文件列表。只记录需要的配置，不打印 env、认证 token、代理订阅或私钥。

建议报告文件：`poc/docs/setup/preflight-2026-11-14.md`。这一路径是计划交付物；交接时不存在不表示已执行失败。

## 5. Rust、Foundry、前端与共享缓存

### Rust

复用现有工具缓存，而不是再写根分区。使用仓库公开的 bootstrap 约定或显式 PATH 选择 1.95.0，检查实际 `rustc --version` 与 `cargo --version`。

```bash
export REGIONAL_BFT_TOOLING_DIR=/mnt/reth/jy/regional_BFT_sim/.tooling
export RUSTUP_HOME="$REGIONAL_BFT_TOOLING_DIR/rustup"
export CARGO_HOME="$REGIONAL_BFT_TOOLING_DIR/cargo"
export PATH="$CARGO_HOME/bin:$PATH"
export CARGO_TARGET_DIR=/mnt/reth/jy/regional_BFT_sim/poc/sequencer_rotation_v2/.build/target
export CARGO_BUILD_JOBS=2
```

这些变量只在当前任务 shell/启动脚本内设置，不改远端全局 shell/代理配置。初始化 `poc/` Cargo workspace 后锁定依赖，按 `--locked` 运行；缓存未齐时不要谎称 offline 构建成功。Commonware 使用当前批准的 2026.10.0，不自动升级。

### Foundry / protoc

先确认是否存在不在 PATH 的安装；没有时安装到用户可写的任务工具目录，固定 release/校验来源并记录版本。Forge/Anvil 在控制链阶段需要，protoc/tonic 编译在 I1 就可能需要。优先完成真正需要的依赖，不因 Foundry 尚未安装就阻塞可独立完成的接口合同。

不把 `curl | sh` 的输出当成已锁定环境；安装后实际检查版本、编译一个最小合约/RPC 示例。不要申请 sudo 或改系统服务来获得并不必要的依赖。

### Web

Web 使用 TypeScript/React/Vite 和 observer-server 的 HTTP/SSE。固定 Node/package manager 与 lockfile，读取远端实际版本后选择兼容包。浏览器 Date.now 不参与协议 timeout；前端是只读观察端。

## 6. 通信与进程管理

推荐默认仅监听 loopback，端口可配置：

| 服务 | 建议端口 |
| --- | --- |
| SimHub gRPC/admin | 19400 / 独立受控 admin endpoint |
| Anvil | 18545 |
| Web Observer | 18080 |
| 官方 Espresso 兼容测试 | 18770，仅独立测试需要 |

使用前先查占用；不能停止未知现有服务抢端口。replica worker 作为 RPC client，可不为每个节点分配公开监听端口。日志、pid、journal 与 key fixture 全放该次 run 目录。

本地查看 Web 时可用：

```bash
ssh -N -L 18080:127.0.0.1:18080 xuelab_jump
```

此命令只转发该页面，不开放服务器公网端口。观察端与协议控制端分开；浏览器慢/断连不改变仿真进度。

Supervisor 只管理本次 run 启动的 child PID；退出时只终止它们，不使用 `pkill codex`、`killall anvil` 等广泛清理。显式场景 crash 与无脚本基础设施 panic 分开记录。

## 7. 多 Agent 工作顺序

### Doc

主协调给 spec 角色当前 v2、I1 范围和允许写的 docs 路径。产出 `task-contract-i1.md`：对象、接口、正常/边界行为、时钟连续性、样本年龄、租约、受控 Ack 与持久化、验收场景。

协议未定问题由主协调记录，不由 builder 自己发明更强假设。v2 已经确定的选择不再反复向用户求确认。

### Implement / Verify

合同冻结后，builder 写指定代码；verifier 从来源与合同独立写期望再读实现。两者可并行，writer 只有一个。若后续确需多个 writer，先有 Git baseline，再用受管理的独立 worktree，各自分配路径，主协调集成。

共享 types、protobuf 和 Cargo.lock 由主协调负责；接口变化必须回到合同并重跑相关用例。不得用源码实现本身计算“期望值”来当独立验证。

### Simulation

只在命名的已验证 revision 上运行同意的场景。一次一个 Rust build/较重 simulation；远端虽有多核，先限制编译压力，测量后调整。formal experiment artifacts 与 smoke/setup 结果分目录。

每次交接包含：合同版本、候选 revision/diff、命令、seed、原始 trace、检查结果、未覆盖边界。有限测试不是协议证明，不声称已经完成原生兼容、生产时钟同步或安全审计。

## 8. 第一阶段的可审查结果

I1 最小交付：两个真实 worker + Clock RPC + timer + message route + journal。重复输入在不同人为墙钟停顿下得到相同虚拟 trace；分别检查 RPC 高延迟、过期 sample、乱序回复、lease expiry、crash/restart。

通过后才扩展到三地区/五节点、regular，再到组织 Force 和 D/RESUME。15-worker、链与 Web 是最终第一阶段验收，不要求在第一个代码提交中一次完成。

详细验收列表引用 [v2 §12](poc_design_zh_v2.md)，不要重新定义另一个更弱的完成标准。

## 9. 验证/交付记录

交接时只检查了文档、SSH 路径、远端目录/工具存在性和源状态；bootstrap、新软件安装、I1/完整 PoC 均由新 chat 执行后记录。Git snapshot commit 只说明接收内容已保存。

实际新 chat 的 id、host、Ultra 创建结果、文件校验结果和 snapshot revision 将写入 delivery receipt。若宿主配置/权限阻止某步，应记录具体动作与错误，先完成不受影响的部分，不通过修改全局代理或覆盖父仓库绕过问题。
