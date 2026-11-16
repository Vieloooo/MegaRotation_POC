# create_thread 会话工具差异诊断

2026-11-16。范围：总协调主会话与新启动的 `repair_create_thread` 子会话之间，Codex chat 创建工具的可见性差异。没有修改 PoC 实现、应用配置、权限或安全设置，没有重启任何 daemon/SSH，没有通过非公开入口调用工具。

## 结论

**父会话的工具缺失尚未修复；新子会话的标准工具可用，并已成功完成用户授权的真实恢复 chat 创建。** 这是已验证的会话差异与具体任务恢复，不是全局修复。

能确定的是：创建功能没有在所有会话中消失，同一已连接项目仍能由提供该工具的会话正常创建 managed-worktree chat。无法从现有证据确定父会话为何没有收到或保留这些工具，不能归因于旧恢复任务的安全提示、CLI版本或某个配置开关。

## 工具与连接证据

父会话 `/root` 在本次调查中再次报告实际检查结果：

- `typeof tools.mcp__codex_app__create_thread`、`send_message_to_thread`、`fork_thread` 均为 `undefined`。
- 对应条目不在其 `ALL_TOOLS` 中。
- `codex_app__list_threads` 等只读应用工具正常。

新子会话的首次实际检查（总工具目录176项）：

| 名称 | JavaScript 暴露 | ALL_TOOLS |
| --- | --- | --- |
| `mcp__codex_app__create_thread` | function | 存在 |
| `mcp__codex_app__send_message_to_thread` | function | 存在 |
| `mcp__codex_app__fork_thread` | function | 存在 |
| `mcp__codex_app__handoff_thread` | function | 存在 |
| `codex_app__list_projects` | function | 存在 |
| `codex_app__read_thread` | function | 存在 |

子会话收到的本轮工具增量声明包含 `mcp__codex_app` 命名空间。父会话随后重查仍缺失，故不能说它也恢复了。仅有 JavaScript 函数存在不证明操作可执行，下面的真实创建调用提供了操作证据。

子会话曾调用标准 `list_projects` / `list_threads`，但这些只读调用未及时返回，等待被结束。没有据此宣布项目不存在或主机断连；父会话可正常读取相同主机和项目，并独立确认创建结果。项目使用已核实的 `ccb826e2-d47a-4623-9b20-2632ac96d8e3` / `xuelab_jump`，没有切回已知不可连接的 direct 别名。

## 实际创建验证

没有创建无关测试chat。依照用户此前明确的新本地恢复会话要求及主协调派发，调用标准 `mcp__codex_app__create_thread`：

- 标题：`PoC 01 本地恢复续作与 I1 验收`
- 模型：`gpt-6-astra`，thinking：`ultra`
- target：已连接 xuelab_jump 项目，`environment.type=worktree`
- 未传虚构 startingState；prompt要求在自动分配的 worktree 中正常 fast-forward 导入既有 WIP 快照，保留旧worktree/index。
- 原 R1–R18 全部保留；实际范围是自己的本机模拟、loopback、本次child PID和私有fixture，任务未改成对外系统操作。

调用返回 `isError=false`：

```text
clientThreadId: client-new-thread:ade7163c-8e35-424a-a621-9de7a84252ad
hostId: remote-ssh-discovered:xuelab_jump
```

随后由本机 session_index、Git worktree 和实际 session turn_context 交叉核对，并由父会话通过 `list_threads` 再次确认：

```text
threadId: 01a1e1c6-25a9-7021-bd85-fcccd42bf829
worktree: /home/jy/.codex/worktrees/2123/megaroation_poc
branch: codex/poc-native-recovery-resume
HEAD: 8a81391c7a0b08be3ce1d2adf3b44e8bf4bbc509
turnId: 01a1e1c6-29b0-7e22-a5da-2f4fd2e0a54e
model: gpt-6-astra
effort: ultra
```

新chat已发出实际进展：干净worktree、fast-forward到准确快照、BFT pin正确，开始核对R1–R18覆盖和接口门禁。没有创建第二个worktree，没有移动旧branch tip，也没有宣称开发证据等于I1通过。

## 本机版本与配置观察

仅检查与本问题有关的字段，没有输出凭据：

- `/home/jy/.local/codex-proxy/bin/codex --version` 返回 `codex-cli 0.167.0`，伴随只读文件系统不能创建PATH aliases的提示；该提示没有阻止创建chat，不能视作根因。
- 用户配置默认 Astra/xhigh；项目配置 Astra/ultra。
- 两份所检配置没有 `[mcp_servers]` 工具 allowlist/denylist 条目；没有发现可直接修改的 create_thread 开关。
- 远端CLI版本不等于用户Mac桌面app版本；未获取桌面app源码或注册逻辑，不能声称已定位桌面缺陷。

## 修复状态与后续边界

没有执行设置修复，因此无需配置回滚。已解决的具体阻塞是：利用新子会话本来就提供的标准创建工具，创建了用户授权的恢复接手chat。父会话是否重新暴露 create_thread 仍由父会话实时工具检查决定。

当前最小、已验证的执行路径是：有标准创建工具的会话承担用户明确授权的创建动作，父会话通过其正常只读工具核对并管理。不能将工具名伪造进目录，不能靠修改安全配置或非公开HTTP/CLI接口替代缺失工具。若要定位父会话注册根因，仍需客户端侧工具刷新/注册证据；本次没有理由重装、升级或重启共享服务。
