# 功能09 准备与接口交接 v1

2026-11-15。Docer角色为命名 `bft_spec`（GPT-6 Astra/xhigh）；本报告只记录来源推导和合同准备，尚无实现/测试PASS。主chat Astra/ultra由总协调运行时记录核验（lead提供），不是Docer自行查询结果。

工作区 `/home/jy/.codex/worktrees/65ef/megaroation_poc`，分支 `codex/poc-anvil-l1`；初始 `ada48a584ad4de15e302c479ee8cd41b9e2a8d9b`，合入授权文档政策后基线 `0a951c482a9712678ab9de34765b781b9c3c9f42`。Regional BFT只读源 `/mnt/reth/jy/megaroation_poc/regional_BFT_sim`，固定 `691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6`，Commonware `2026.10.0`，未改子模块。

可审查接口为[task-contract-anvil-l1-v1](contracts/task-contract-anvil-l1-v1.md)，状态FROZEN，09 lead已落实独立Verifier审查条件。03消费者准备接口核对自 `183430e71e5d61533f54a76b2d7d22c5021f1704:poc/docs/contracts/task-contract-mock-plan-execution-v1.md`，它只冻结准备语义，不能当实际types已实现。09/11/12/13交接与分层依赖v2共同作为消费者需求，不等待未启动消费者chat才做本职选择。

本版工程选择：Solidity标准ABI、标准带salt的EIP-712域、真实secp256k1签名，完整cursor按tag携带opaque GLOBAL bytes或RESUME序号；GENESIS source为空。治理签名绑定实际L1 caller/rid，序号允许第一条0和以后跳号。去重返回原结果，签名身份检查不被cache绕过。BatchAttestation绑定完整prev/next/DA证书/range/gpos，不签未来收录位置；RESUME签名显式origin输入，实际origin上界由inclusion检查。控制policy保留完整字段，仅允许固定(7,0,1,2,l1Floor)的static-schedule-v1；l1Floor不改变该公式。

历史边界：`written_at`来自真实receipt；规范log是交易内索引，RPC logIndex另保留。RESUME的CursorSet与ResumeRecorded必须同成功交易精确配对，两条同block控制各自保留位置。Genesis用独立初始化事件定位，不伪造正常接纳。历史block-tag读与receipt重放相互核对。03负责公共L1对象，12负责真实ResumeCut；09不创建第二份ReplayPosition或假cut。

driver边界：同一个Rust组件的真实RPC backend承担显式Tick/Submit/Mine/Finalize/Deliver；Python只能编排与独立检查。部署也用该链路。reference μs、整数秒header、模拟finality及reader delivery分别记录。脚本输入可验收组件，02真实Clock/transport/grant/ack仍是完整09门槛。

独立Verifier负责 `poc/verification/anvil-l1/` 的source expectations、固定向量/独立preimages和审查报告；合同列A01–A19。Builder在冻结后唯一拥有 `poc/contracts/` 和独立 `poc/crates/l1-adapter/`。本Docer仅写这两份文档，不写实现/verification、不commit、不运行heavy。

来源SHA-256（读取的工作树内容）：

| 文件 | SHA-256 |
| --- | --- |
| `formalization_v10/00_arch_zh_v0_1.md` | `2de0929fc2f75fda51802c7c354146849a285a247283ca1d8fc7338b2a8e8871` |
| `formalization_v10/01_l1_authority_zh_v0_1.md` | `e95c0cbc8c8daa05b9ee60cba342793ebb85d0c9e37afa972f34259676ad84b6` |
| `formalization_v10/06_certifier_zh_v0_1.md` | `66e59d3db662021eb9c109114c2c423673d46a620eeed8ee3f599c6033e13167` |
| `formalization_v10/07_batch_inbox_zh_v0_1.md` | `666b5a3735f677da4a5b087e9da96d66b30524898eac7688a678b3ec59c410cf` |
| `formalization_v10/08_da_zh_v0_1.md` | `cb3a81091a8b45070e79ab8f0ae49df39e624e4595ea12248d3079e98114d788` |
| `formalization_v10/09_finality_rules_zh_v0_1.md` | `6922b77e1de0c8eaddde9f08d08d6734ae63844d406f0e6e32438e33ba0dabcc` |
| `formalization_v10/poc_design_zh_v2.md` | `95868b06d71b170dce84c9ec612beae83c161595520390191130feadd12d88ab` |

接下来的发布需准确接口/expected提交，然后源码候选SHA、独立复核、同SHA Tester。原始artifact使用任务09数据盘目录；最终提供绝对路径与摘要。现阶段接口准备以外全部实现结果均为INCONCLUSIVE，完整功能09未完成。

独立准备结果：4组真实签名、57项字段/域变异、40项非规范签名和34项Cast交叉核对PASS；见[独立准备结果](../verification/anvil-l1/preparation-results.json)。接口已冻结，EVM/driver仍待实现。已核对现有Foundry/solc四个工具hash，均匹配setup；任务私有Cargo registry缓存已复制，未复制任何凭证或全局配置。
