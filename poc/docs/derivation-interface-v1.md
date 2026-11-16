# 功能12：阶段 A 接口与启动记录

2026-11-16（Asia/Shanghai）。本文保留可编译接口发布时的记录；后续阶段A已独立复核和正式测试PASS，见[阶段A交付](derivation-stage-a-result.md)。完整功能12仍未通过。

## 基线与来源

- 复用本chat已有managed worktree：`/home/jy/.codex/worktrees/29df/megaroation_poc`，分支 `codex/poc-derivation`。初始 `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`，工作区干净。
- 顺序正常合并03 `e9c15e04bc83b335668f3ecd837367aa87344a89` 和09 `c443ae925ed549b046731c2d8ef3dd9ce0311d24`。合并提交分别为 `7e476e5da1beee96d8b97aa9ff04e98ce925343b`、`37b4f2b8a6d67b303ba90b1d4d1981c93ed8dd12`。未reset、改写历史、合入main或push。
- 冻结合同与独立期望提交：`4a86f8d5c6f2d39e52994bb7c862fcc7313ccf8c`。见[合同](contracts/task-contract-derivation-v1.md)、[来源独立预期](../verification/derivation/source-expectations.md)及[来源清单](../verification/derivation/source-manifest.json)。
- Regional BFT gitlink保持 `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`；只读源checkout中已初始化副本，未改其工作树或历史。Commonware pin保持2026.10.0。
- v2设计 SHA-256：`95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab`。

主chat自身turn metadata已核对：turn `01a1e1cc-a99a-7ad0-bcd4-18989dafcd4b`，`gpt-6-astra` / `ultra`，cwd为上述worktree。实际使用命名 `bft_spec`、`bft_verifier`（Astra/xhigh）和单 `bft_builder`（GPT-6.1 Sol/high）；Tester按流程在候选复核后启动。独立Verifier先来源、再合同，未读12源码即保存14项语义预期及独立03 native/state字节参考。另一个只读explorer仅核对09 runner复用，不承担协议验收。没有新建其他用户chat或第二个worktree。

## 接口

独立workspace为 `poc/crates/derivation/`，使用自己的lockfile。类型定义见[src/model.rs](../crates/derivation/src/model.rs)：

- `DerivationConfig`：03 PublicConfig及独立窗口参数，验证版本与checked arithmetic；`inclusion_is_late`只计算严格的窗口谓词，不表示实际批次通过。
- `L1Identity` / `DerivationIdentity`：公开链/合约/配置/genesis绑定。named reader属于本地输入能力，不进入native hash或公共cut身份。
- `DerivedItem` / `ResumeCut`：复用03 canonical对象、实际状态、native hash；cut分别保存Set位置、RESUME位置、目标anchor和该位置不可变前缀。
- `InputPoint`、`Outcome`、`WaitReason`、`Reject`：区分block边界、交易内log位置、fallback及明确的缺输入状态。

接口没有Init/Advance的未实现空壳。生产source能力、状态机、checkpoint和实际runner随后实现。阶段A遇任何真实CursorAdvanced均停在该位置 `Feature11AcceptedPayloadRequired`；不定义第二种frame/DA编码。

## 初次检查与环境

Builder在上述冻结基线上进行接口工作树检查：Rust/Cargo1.95.0，离线生成独立lockfile并执行 `cargo check --locked --offline --manifest-path poc/crates/derivation/Cargo.toml`，PASS；格式及 `git diff --check`通过。首次编译记录38.59秒，为本机编译耗时，不是协议测量。正式Verifier/Tester将检查后续同一已提交源码候选。

命令环境：

```sh
export PATH="/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin:$PATH"
export CARGO_HOME="$PWD/.build/cargo"
export CARGO_TARGET_DIR="$PWD/.build/target/derivation"
export CARGO_BUILD_JOBS=2
flock /tmp/megaroation-poc-heavy.lock cargo check --locked --offline \
  --manifest-path poc/crates/derivation/Cargo.toml
python3 poc/verification/derivation/independent_reference.py --check
```

本worktree `.build` 指向 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/12`；仅复制公开Cargo registry，保留原目录内容，target/artifacts/PID独立。启动时根分区约12GB可用，数据盘约1.4TB可用。Git公共metadata及数据盘目录在sandbox写根之外，所需定向升级已通过自动审核；没有修改全局配置。

独立参考结果 SHA-256为 `2c8ff2371f77b2b816c4e3af74038584cc13789cc8ae71e1cd93e4e094536429`，seed为 `none (explicit fixtures)`。参考向量属于规则准备，不冒充真实收据或DA证据。

## 后续验收

阶段A继续无batch fallback、前序接纳WAIT、真实09同区块多RESUME、D-ahead切面、whole-marker原子性、checkpoint验证。11精确接口SHA尚未交付；真实payload/DA/批次执行与substitution属于阶段B。13恢复策略及受控runtime接线保持开放；组件里程碑不增加完整accepted功能计数。

已只读核对总协调修订 `0c15aac` 和功能handoff修订 `32a3eb6b31ff8c1dfe2d05dbc2ed26c0c031bb77`，以及源checkout的 `poc/docs/commander/restart-free-handoff-track-20261116.md`。D/fallback/cut无需等待BFT同cid进程恢复；本合同原先即未施加该门禁。批次延迟/暂停、实际Batcher crash和BFT进程恢复分别报告；D的checkpoint重放不代表后两种恢复。没有为读取通知覆盖工作树或修改其他owner路径。

## 11 接入所需字段与不变量

此清单提出消费者需求，不另定义编码，待11准确接口提交后冻结阶段B适配。

1. 已接纳输入必须绑定09实际finalized `CursorAdvanced` 的 `(block,tx,log)`、block/transaction hash、既有 `BatchWire` 与commitment；保留实际inclusion，下载时刻不参与窗口判断。
2. 11提供commitment校验后的唯一typed frame/record接口，明确1–2条记录的wire顺序、连续height及first/last绑定；D不能通过任意本地bytes或bool补齐该能力。
3. 每条记录需有03 `Metadata` 的height/timestamp/origin和源Kept普通交易规范字节。D按自身实际parent重新生成DEP并执行；源parent/epoch hash若在线格式中存在，仅保留为证据，不用于选择D parent或canonical L1。
4. 区分缺数据/commitment不符而保持原位置WAIT、已取得并认证字节的格式失败，以及成功解码。具体失败分类以固定来源和11接口联合冻结，不将缺数据转成空batch。
5. 提供同frame剩余记录的确定性顺序及可重放依据；发生同高度替代后，下一条必须针对新的实际D parent重新检查。checkpoint与历史cut需要可重复取得此前实际接纳的相同输入。

接口提交 `3b1b34fce6f3a3bac5099d53bdf5f7df56079f80` 经独立Verifier只读检查，无实现前阻断问题；范围仅类型、配置、窗口谓词及位置语义，未宣称实现验收。后续源码候选与正式结果另行记录。
