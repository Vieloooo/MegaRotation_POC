# H1 计划矩阵（全部 NOT_RUN）

独立期望先于新 H1 合同/实现读取；75 条输入配方见 vectors.json，均非已执行轨迹。所有密码学阴性须由真实证书变异得到，不以本JSON中符号字段冒充合法BLS。

| Gate | 输入/变体 | 后续必需证据 | 当前状态 |
| --- | --- | --- | --- |
| NR0 | 准确M0基线及最终shared shim的Fresh兼容 | exact SHA/tree/patch/lock，原Fresh healthy/two-real-alone/proof-body/Close-lease/crypto及受影响完整矩阵 | NOT_RUN |
| NR1 | 15 PID/5初始真实engine；身份/registry/权限变异；10非成员root | process/key/engine映射，真实PoP，raw envelope，完整Ack | NOT_RUN |
| NR2 | offset±1000、RPC250us/10ms、lease等号、样本乱序、原timer | request/sample/response、每次Clock read/sign，独立time checker | NOT_RUN |
| NR3 | vote/certificate/resolver/body；无重启的延迟触发repair | 四channel真实frames；零resolver不能计接通 | NOT_RUN |
| NR4 | 健康固定1–10十physical，真实改变状态交易及nonce/balance过滤 | raw inputs、pre/post roots、physical/native哈希、独立执行 | NOT_RUN |
| NR5 | 真实3/4票、重复signer、wrong kind/domain/cid/epoch/view/subject/body、历史验签 | BLS原始字节和公开verifier重算；无当前钟依赖 | NOT_RUN |
| NR6 | 13精确Accept时间边界、NotBefore、slot≠view、缺body/L1、首收认证乱序 | 原首收、work/callback id、WAIT/abstention、真实签名出口 | NOT_RUN |
| NR7 | 0/20/100/500/1500ms；3早2晚；late高view；3/2候选分裂 | 每cid固定active set及τ_lc/τ_sync、适用时限、无屏障 | NOT_RUN |
| NR8 | Close/stop/ClockUnsafe与Build/verify/certify/Sign竞态；cached/conflict | 持久terminal和实际Created/CacheHit/Denied、保留旧输出 | NOT_RUN |
| NR9 | late-invalid effect、假receipt、重复grant/release、stale durable plan | 完整候选Ack，pre/post全state，root/aux/witness sync、保管义务 | NOT_RUN |
| NR10 | seeds7/42/99 × 正/反进程启动 × wall pause0/2ms | 同候选/profile逻辑输入，完整canonical virtual trace/native帧逐字节比较；PID/wall单列 | NOT_RUN |
| LC | 重复Create/Enter；A→B→A；各phase关闭；future-cid权限 | 同PID真实engine原实例/view/locks/deadline/签名历史；多cid隔离 | NOT_RUN |
| PREP | exact-anchor完整执行；root-only；缺输入；取消与stale完成 | Ready依据、工作代次/cid绑定，迟到不复活 | NOT_RUN |
| DELIVERY | proof-before-body、缺ancestor、重复proof、坏body | proof观察与完整连续交付分层 | NOT_RUN |
| M1 | R0→R1→R2首结果入G；SessionEnd晚合法slot/padding；可选提前h11 | H1/H2/H3同SHA真实接线、公共位置；不移后继窗口 | NOT_RUN |
| OFFICIAL | M1实际结果同字节提交官方本地dev-node读回 | payload来源hash、namespace/q/index、独立wall/Unknown证据 | NOT_RUN |

执行命令状态均为 **PLANNED**。正常runner、flags、wire与freeze尚未批准，故不编造CLI。后续lead/Tester需在同一个命名已验证候选SHA上记录真实完整命令、cwd、工具chain、seed、profile、原始trace路径和结果；所有build/重任务用 `flock /tmp/megaroation-poc-heavy.lock` 及独立数据盘target/artifact，当前不执行。

复核当前只允许inline Python校验本JSON、来源hash、链接和算术。不调用生产函数，不运行密码学或process fixture。该静态检查的PASS不能提升任何上表状态。有限后续矩阵也不证明完整协议、所有调度、Byzantine状态空间、恢复或官方PE时序。
