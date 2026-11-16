# 功能03交付记录：Mock Plan、公共对象与轻量执行

日期：**2026-11-16（Asia/Shanghai）**。本报告固定已实现接口和已经发生的组件证据，不自引用包含本报告的提交。**最终封存候选SHA、该SHA上的独立复核与Tester结论，以本chat最终交付和该SHA对应的原始run manifest为准；完整accepted由总协调决定。** 此处不预先声明最终集成PASS。

03已按[分层依赖计划v2](commander/dependency-plan-v2.md)推进完整独立组件，不再处于`waiting-dependency=01`。依据为[实现合同v2](contracts/task-contract-mock-plan-execution-v2.md)（SHA-256 `a18d2b33f9c541abc81b4fead601c3ccaead6221c90dc43345d52a850429cc15`）及[M1公共接口v1](contracts/mock-plan-public-interface-v1.md)。旧准备合同/LE黄金向量保留历史，生产采用BE v1与既有`hash_parts`，没有将旧实验编码改名冒充生产格式。

## 工作位置、角色与版本链

Worktree为 `/home/jy/.codex/worktrees/1c43/megaroation_poc`，分支 `codex/poc-mock-plan-execution`，宿主为已连接的xuelab_jump。原始交接基线为 `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`，代码祖先为 `36eff1c76d5c4df5d0790133c0e14719c0c73ebd`。Regional BFT gitlink始终为 `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`，子模块worktree和历史未改。Rust固定1.95.0、Commonware固定2026.10.0；严格Ed25519额外gate固定curve25519-dalek5.0.0。

| 责任 | 实际角色／配置 | 路径所有权 |
| --- | --- | --- |
| 功能主协调 | GPT-6 Astra / ultra；总协调已核实实际turn_context | 公共types、根workspace/lockfile、模块声明、接口决议与集成。 |
| Docer | `bft_spec`，GPT-6 Astra / xhigh | 指定合同、接口／准备／结果文档；不写实现或独立expected。 |
| Builder | `bft_builder`，GPT-6.1 Sol / high | authorization与core的execution/metadata/merge/fixture模块，依次领取单写者权限。 |
| 独立Verifier | `bft_verifier`，GPT-6 Astra / xhigh | `poc/verification/mock-plan-execution/`；先来源推导与固定expected，再看候选。 |
| 最终Tester | 按指定`bft_simulator`，GPT-6.1 Sol / high执行 | 只消费已复核的封存SHA；实际运行角色、命令和结果记在该次manifest，不从配置推断已执行。 |

没有创建额外用户chat、修改总协调登记或其他功能目录，没有合入main或推送远端。重型运行统一宿主flock及`CARGO_BUILD_JOBS=2`；任务独立`.build`/大artifact使用 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03`，不共用其他job target。

| 提交 | 作用与已记录状态 |
| --- | --- |
| `183430e71e5d61533f54a76b2d7d22c5021f1704` | 准备合同和独立输入，历史准备阶段，不计实现验收。 |
| `548cbb24b390fedf2ddfd60f3aab0de770e24491` | 正常合入`0a951c482a9712678ab9de34765b781b9c3c9f42`依赖计划，开放03独立实现。 |
| `dae686b7152a4b8dabcc670fa08e9abd88bf276d` | M1合同与独立向量先于实现冻结。 |
| `0e72dc247de22afdc8d6681713b3d15f4aa05262` | M1接口，同SHA独立7项PASS，可消费接口里程碑。 |
| `862cc1657cf81afcdcf6548a197b003f47fd979b` | M2合同与生产BE独立向量先于实现冻结。 |
| `82c5eafa9080569235f14d4ab881c6e4fbf48a9e` → `fadde7e84b5eee9dc528b2c318127a7c2c5721df` | M2共享types及独立codec测试接入；后者5项M2+7项M1 PASS，并获只读复核。 |
| `aa0ce278c164357dc3e737904892daebb0b935b5` | 记录[M1/M2接口发布](mock-plan-public-interface-release.md)。 |
| `f40ba132d8296f038da646cb0f5a47e38ded7c68` → `81f84f8b3c1ed63f1fc39d05f52000777201bc87` | authorization/core完整实现，接入独立真实密码学／执行／reader场景。 |
| `7fe0e85884208480e8b453e45458fa74e23eeb46` | Merge先拒绝已知无效context，之后才等待缺body；Builder运行18项PASS（auth6+recipe1+core11）。 |
| `e30f5ed3b3352d58005f672bfebf22e9810552fb` | selected-tip路径进一步在缺ancestor前校验已提供执行材料和已知body错误；Builder运行core13项PASS，独立Verifier的实际库联合运行32项PASS、4个Python参考检查PASS，并输出3条实际证据JSON。[正式独立复核](../verification/mock-plan-execution/implementation-v1/review-e30f5ed.md)确认完整授权03组件范围PASS、无开放finding。它是本报告核对的代码候选，不冒充最终封存SHA。 |

## 已实现行为与证据范围

`poc-types::domain`提供checked身份、opaque引用、严格M1 codec、M2有界对象与checked hash。原I1根导出、protobuf和hash函数保持兼容。M2顶层u16BE版本1、结构字段固定顺序、嵌套raw不重复版本，数组/长度有界；规范state只保留非零账户。Plan/Descriptor/PhysicalBlock/NativeBlock及fixture有实际codec；AnchorEvidence/RegionalResult/ReplayPosition/LifecycleEvent为typed骨架，后者没有已批准的canonical wire格式，`FinalVerifier`没有默认通过实现。

`poc-authorization`执行真实15身份registry加载：严格Ed25519额外检查、MinSig G2/G1解码、真实PoP验证、重复身份／vote key拒绝，以及各区按实际identity公钥排序的Commonware participant map。公开fixture的registry hash为`82e053c4537f6482bc4b10abcf27882b6abb23bd2f72e012bbb3ded329f285d9`，三地区signer顺序分别为`[4,3,1,0,2]`、`[7,6,5,9,8]`、`[13,11,14,10,12]`。这是PoP/registry证据，不是原生最终证书的3/4 signer验收。

StaticScheduleProvider按冻结公式产生region/leader，独立区分pid/member/signer-index，支持来源不足WAIT、已知冲突REJECT、实际Plan重算和cid检查。regular选区reference与production anchor分开。`validate_descriptor`验证config/registry/Plan字段与cid的一致性；它不读取外部来源，消费者仍须通过`validate_plan(...,PublicReferenceView)`及实际anchor来源/执行状态验证。

`poc-core`提供纯Apply、metadata/DEP、Merge与selected-tip取body入口。包含zero/self/unknown-account、nonce/余额溢出、credit溢出原子拒绝、malformed整payload拒绝和Kept过滤；在padding后、当前Begin前核对physical pre-root。native hash使用实际连续native parent、metadata、DEP、Kept和post-root。已知坏context、错误提供材料或body不被缺ancestor的WAIT掩盖。单次Merge最多10个physical blocks、4096个展开高度；4096成功与4097拒绝均有实际运行证据，未声称无界历史处理。

L1 fixture有配置commitment、独立Finalize凭据和header/control delivery、逐reader连续可见前缀、out-of-order暂存、缺完整control覆盖WAIT、冲突原子拒绝及范围外FixtureExhausted。fixture默认heights0–30，timestamp=`2*height`秒，固定finalize下界timestamp+1.5s；finalize不是delivery，时钟到期不使reader自动读到文件。它只承诺完整空控制日志，不冒充真实Anvil receipts。D未来可用`actual_metadata`与同一执行器，不需要把regional origin或G state复制为D。

主执行向量在anchor h0之后补padding h1/h2并处理physical h3；最终账户为A=(nonce2,balance90)、B=(2,15)、C=(0,19)，raw交易Kept索引为`[1,3,4,6]`。

| 实际值 | 独立期望及e30日志中相同值 |
| --- | --- |
| h3 pre-root／h2 post-root | `df9c31e30169fdcf76b8539ec33fa9c400278362d415d8c670d61ee34083758a` |
| h3 post-root | `76bffa42f643077ad9e15c1f7a098c03cfe5e5562b791f7ef932493c83360349` |
| physical h3 | `806e5423463b9326eefc8771b2f0f8b6ff953509e714045d3a8e7bd4b3431cac` |
| native h3 | `bb6b678ab60e51eb8125a7c3279a6fa3f856d9611c8b7a297376a52a35077d85` |
| 4096-height边界末native hash | `b574146396dc53a3e9099193398887106a874121ed46564d98322c90a9146db3` |

## 下游调用边界与最小示例

Rust package名为`poc-types`、`poc-authorization`、`poc-core`，import分别为`poc_types`、`poc_authorization`、`poc_core`。下列示例按e30源码签名核对，Docer未单独编译报告片段；输入由调用者提供已绑定配置和公开材料。示例没有替调用者制作来源验证桩。

### Registry、公开reference与descriptor

```rust
use poc_authorization::{PlanOutcome, StaticScheduleProvider, ValidatedRegistry};
use poc_types::domain::{
    Anchor, Descriptor, InputStatus, PlanInput, PublicConfig,
    PublicReferenceView, RegistryRecord,
};

fn prepare_descriptor(
    config: PublicConfig,
    public_registry: RegistryRecord,
    input: &PlanInput,
    references: &dyn PublicReferenceView,
    production_anchor: &Anchor,
) -> Result<InputStatus<Descriptor>, String> {
    let registry = ValidatedRegistry::load(public_registry)
        .map_err(|e| format!("registry: {e:?}"))?;
    let provider = StaticScheduleProvider::new(config, registry)
        .map_err(|e| format!("config: {e:?}"))?;
    match provider.resolve(input, references) {
        PlanOutcome::Ready(plan) => {
            let descriptor = provider.descriptor(&plan, production_anchor)
                .map_err(|e| format!("descriptor: {e:?}"))?;
            provider.validate_descriptor(&descriptor)
                .map_err(|e| format!("descriptor check: {e:?}"))?;
            Ok(InputStatus::Ready(descriptor))
        }
        PlanOutcome::Wait(missing) => Ok(InputStatus::Wait(missing)),
        PlanOutcome::None => Err("no eligible Plan".into()),
        PlanOutcome::Reject(error) => Err(format!("Plan: {error:?}")),
    }
}
```

`PublicReferenceView::reference(&BlockRef)->InputStatus<Anchor>`必须由已验证来源实现；`Ready`引用要精确匹配height/origin/id。上面函数保留缺输入WAIT，诊断错误只作调用示例，不定义新的线上错误码。调用前/进入BFT前还须验证production anchor来源、实际执行后状态和Enter守卫；该函数没有完成这些步骤。对收到的现成Plan应调用`provider.validate_plan(&plan,references)`，不能只decode或只重算cid后视为授权完成。历史FinalCert还需04实现的`FinalVerifier::check`。

### Setup、显式Finalize与各reader delivery

```rust
use poc_core::fixture::{
    CheckedSetup, FixtureFinalizer, FixtureReader, check_setup,
};
use poc_types::domain::{DepFixture, ExecutionAnchor, InputError, L1Fixture, PublicConfig};

fn open_readers(
    config: &PublicConfig,
    genesis: &ExecutionAnchor,
    dep: DepFixture,
    l1: L1Fixture,
) -> Result<(CheckedSetup, FixtureFinalizer, FixtureReader, FixtureReader), InputError> {
    let checked = check_setup(config, genesis, dep, l1)?;
    let finalizer = checked.l1().finalizer();
    let reader_a = checked.l1().reader();
    let reader_b = checked.l1().reader();
    Ok((checked, finalizer, reader_a, reader_b))
}
```

对来源fixture的一行，三个实际API依次接受不同受控输入；finalizer要求来源连续finalize，reader可以out-of-order收包并保持连续可见前缀：

```rust
let finalized = finalizer.finalize(&row.header, actual_finalized_at_us)?;
reader_a.deliver_header(&finalized, header_delivered_at_us)?;
// header可见不表示完整control日志已交付。
reader_a.deliver_empty_control(
    &finalized, row.empty_control_commitment, control_delivered_at_us,
)?;
// reader_b仍保持自己的delivery进度。
```

这里`row`是已绑定fixture中的`L1FixtureRow`，三个时间为调用者控制的整数μs输入；`?`返回`InputError`。`FinalizedFixtureRow`字段私有，须由成功的Finalize得到。实际运行中02调度器分别提交这些动作；示例本身是同步组件调用，不是已经接入grant/Ack的服务。09可实现同一个`FinalizedL1Reader`：`header(height)`、`require_control_coverage(through_height)`、`finalized_tip()`，其真实receipt/finality来源仍需09验收。

### 固定window和实际parent上的Merge

```rust
use poc_core::{
    fixture::{CheckedSetup, FixtureReader},
    merge::{MergeOutcome, merge},
    metadata::RegionalMetadata,
};
use poc_types::domain::{DomainError, PhysicalBlock, Window};

fn merge_available(
    checked: &CheckedSetup,
    reader: &FixtureReader,
    chain: &[PhysicalBlock],
    window: &Window,
) -> Result<MergeOutcome, DomainError> {
    let metadata = RegionalMetadata::new(checked.config(), reader)?;
    Ok(merge(
        checked.genesis(), chain, window, &metadata,
        checked.dep(), checked.config(),
    ))
}
```

上例接genesis完整链；非genesis anchor改为已独立验证的`ExecutionAnchor`传给同一`merge`。它返回`InputStatus<MergeOutput>`，WAIT/REJECT均没有可提交的部分前缀。`MergeOutput.blocks`逐height含`virtual_block`、`physical_digest:Option<Hash32>`、执行后`state`、`native_block`与`native_hash`。若先取得选定tip而祖先body尚未齐，用`merge_from_tip(SelectedTip{anchor,execution_anchor,tip}, &dyn PhysicalSource, window, metadata, dep, config)`；缺body为`InputMissing::Parent(digest)`、缺执行材料为`InputMissing::State(anchor_id)`，调用方仍负责tip的最终证明和选中资格。纯Merge输出不等于Final、G接纳、Inbox接纳或D派生。

## 可复现命令、输入和原始证据

全部命令CWD为 `/home/jy/.codex/worktrees/1c43/megaroation_poc`。原始artifact根目录为 `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03/artifacts/`。这些是本worktree的实际绝对路径，Git只保存合同／输入／报告；最终Verifier/Tester manifest另记录候选、命令、工具版本、输入hash、stdout/stderr和证据摘要。

独立固定输入见[implementation-v1目录](../verification/mock-plan-execution/implementation-v1/)：M1为43个有效bytes、68个坏编码和9个I1 hash样例；M2为40个固定BE记录及手算执行边界；真实registry有单独public fixture/negative vectors。主场景无随机输入，`seed=null`；密码学测试recipe固定每pid Ed25519 seed=`303000+pid`、BLS ChaCha20 seed=`303100+pid`，只track公开key/PoP与recipe，不存私钥。4096边界也是固定输入，不是随机仿真。

M1与M2共享types的命令/证据分别见[独立M1报告](../verification/mock-plan-execution/implementation-v1/m1-review-0e72dc2.md)和[接口发布记录](mock-plan-public-interface-release.md)。已发生的Builder组件命令为：

```sh
# 7fe0e85884208480e8b453e45458fa74e23eeb46：auth6 + recipe1 + core11
flock /tmp/megaroation-poc-heavy.lock env CARGO_BUILD_JOBS=2 REGIONAL_BFT_RUST_BIN=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin ./poc/scripts/bootstrap.sh test -p poc-authorization -p poc-core --locked --offline

# e30f5ed3b3352d58005f672bfebf22e9810552fb：core13，含实际证据JSON
flock /tmp/megaroation-poc-heavy.lock env CARGO_BUILD_JOBS=2 REGIONAL_BFT_RUST_BIN=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin ./poc/scripts/bootstrap.sh test -p poc-core --test mock_plan_core_independent --locked --offline -- --nocapture

# e30同SHA独立实际库运行：M1 7 + M2 codec5 + auth6 + recipe1 + core13 = 32 PASS
flock /tmp/megaroation-poc-heavy.lock env REGIONAL_BFT_RUST_BIN=/mnt/reth/jy/regional_BFT_sim/.tooling/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin CARGO_BUILD_JOBS=2 /home/jy/.codex/worktrees/1c43/megaroation_poc/poc/scripts/bootstrap.sh test -p poc-types -p poc-authorization -p poc-core --locked --offline -- --nocapture
```

| 已发生证据 | 原始路径／SHA-256 |
| --- | --- |
| M1独立7/7 PASS | `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03/artifacts/verification/m1/0e72dc247de22afdc8d6681713b3d15f4aa05262/run-manifest.json`；hash `89b601c3a158db6fbe2c912fb797fb2cd7cddd534f6a58eaa41fe133c133c896`。 |
| M2 types 5+7 PASS | `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03/artifacts/m2-types/codec-tests.log`；hash `e6bca9c19f3e40d6c6452090b10de813f14bf499545315ca12bb811d1a353c18`。 |
| Builder 7fe0：18 PASS | `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03/artifacts/build/7fe0e85/stdout.txt`；hash `bb74e2e39371816382dc3eca5b5665da340f4b36e48eb673ae1e8c07159cdecf`。 |
| Builder e30：core13 PASS | `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03/artifacts/build/e30f5ed/stdout.txt`；hash `97a19cf3012bb6393205da224bcfb21f57c1b2d5c99ed63e7aa7ea7cf8bd8f07`。同目录保存command、exit-code、commit前后及status。 |
| 独立e30：32实际库测试PASS | `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03/artifacts/verification/full03/e30f5ed3b3352d58005f672bfebf22e9810552fb/run-manifest.json`；hash `cbe83a448a391fac76c0ff8a3f8c357db390cbeb494e4e8b9cb9ca347f4937e8`，return_code=0，HEAD前后相同。 |
| 独立e30实际组件证据 | `/mnt/reth/jy/megaroation_poc/.build/commander-jobs/03/artifacts/verification/full03/e30f5ed3b3352d58005f672bfebf22e9810552fb/component-evidence.json`；hash `352b86ded10153a683bf743ee30e03dc23ffce5fe6ab52983e93618014a94753`，共3条。 |

e30 stdout含三条`COMPONENT_EVIDENCE`：实际三height的root/Kept/native parent/hash、reader A/B不同可见前缀与control覆盖、4096接受／4097拒绝及末native hash。它们是组件运行观测，非网络或墙钟性能测量。

e30独立manifest另记四个Python参考checker均return0，其中`check_extended_vectors.py`在该候选时是补充的未跟踪参考脚本，fingerprint已记录；不能谎称当时已包含在候选中。后续封存将该checker/报告及测试文件的纯格式调整纳入Git，再对封存SHA复验，不以e30结果替代最终SHA结果。

## 下游责任与结论边界

03对外提供可组合的纯值/授权/执行与fixture组件。02/04仍负责真实受控runtime与原生FinalCert适配，09负责真实Anvil/finality/receipt来源，07负责完整公共重放及lifecycle语义，12/13负责独立D与位置绑定ResumeCut/RESUME。此报告不把typed骨架、mock空控制日志、真实PoP或纯执行向量当成这些集成的通过证据。

最终交付应对封存候选重新记录Verifier与Tester的同SHA结果，而不改写本报告中的历史证据。最终manifest给出PASS/FAIL/INCONCLUSIVE、准确观察范围与限制；只有总协调审阅并接受后才计一个完整accepted功能。
