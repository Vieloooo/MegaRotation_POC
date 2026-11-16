# 旧H1准备复核的当前处置

本文件是附记，不改旧independent-capture、source-map、75基础配方或2条旧补充配方。依据d55d39f73593676325bdc0fbb7ba3845ca93025f及本次Docer修订，当前HEAD4741489已包含M0=35d382ac445cc8bc779f0d18fd23cb57e54680da（git ancestry只读核对）。

| 旧finding | 当前文字处置 | 尚未证明的行为 |
| --- | --- | --- |
| DOC-01 | interfaces-h1-v1.md:138明确RouteKernel与所有受影响ReliableMailbox同root/revision准入，保留义务；task-contract:100增加验收 | transport/runtime实现和实际持久原子性仍NOT_RUN |
| DOC-02 | interfaces:47–49与task-contract:66明确RootCommitted(actions_pending)、knownClose不等execReady、guard安装后失败不rollback | 实际签名出口、fsync/错误切点和ActionsApplied仍NOT_RUN |
| DOC-03 | task-contract:105–106分h9→h11指定提前主例与晚合法slot/padding；d55§9明确范围 | 两条真实生产/公开/G接纳轨迹仍NOT_RUN |

因此三项**文档澄清已处理**，不能把它们改记为runtime修复PASS。M0和实施授权已交付，不再充当纯API切片阻塞；正式所有权与分切片单writer仍适用。Force暂不实施，Q-ADMIT只阻相关native切片。
