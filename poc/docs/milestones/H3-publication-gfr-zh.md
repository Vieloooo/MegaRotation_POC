# H3：真实区域结果 → Publisher → SimEspresso → GFR

复用原07 chat/worktree，主模型Astra/ultra，正常合入M0准确SHA后推进。复用03、06、07和16已交付实现，按[总计划](README.md)冻结接口/独立预期再编码，不重写已通过组件。

## 路径与范围

单Builder拥有`poc/crates/publication-bridge/`（独立workspace）、本chat已有`poc/crates/viewer-gfr/`的必要接线扩展、专属合同与`poc/verification/publication-gfr-healthy/`。06 Publisher/SimEspresso和16兼容crate只消费；若其接口必须改，先给精确diff由总协调移交单一owner。H2负责Manager、H1负责runtime/签名，根shared types/manifest不并发写。

1. 消费H1真实Finalization+完整physical ancestry/body/anchor，调用同一真实FinalVerifier；支持公开proof/body分开，绝不以bool桩作为主场景验签。
2. 接已有Publisher持久任务、受控提交effect、SimEspresso proposed/finalized和逐reader实际交付；root/Ack与服务durability事实均可追溯。
3. 复用07纯规则，计算q_avail/接纳键、逐height Expire、满窗/SessionEnd，并将原子lifecycle输出交H2。A/B reader状态和交付顺序独立。
4. 将主场景实际区域最终结果的同一规范字节写成带hash/来源的publication artifact。经16官方本地dev-node提交、读回并核对q/index/namespace/payload字节。此真实发布不替代SimEspresso主验收，不伪造官方PE时序/QC证据。

## 关键接口与验收

先处理已知边界：03对象到GFR-PUBLIC-1 codec、真实FinalVerifier能力、06块上限1MiB与02单payload64KiB的可审查分片/连续前缀、Publisher StopDecision的公开position来源。前述接口先提交给H1/H2；没有完整前缀或本地材料时WAIT，不提前接纳。

独立测试真实proof先到/body后到、祖先缺失、多proof变体、未选中候选先公开、重复publication、同q/同theta、截止等号及padding中的Expire。fixture用于纯规则预备时必须标注；真实H1可用后补全密码学和受控消息证据。交付完整命名SHA与M1可启动接线，后续H4才扩展Force。
