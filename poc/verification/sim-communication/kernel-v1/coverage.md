# 完整SC与组件K的关系

下面的“组件覆盖”是计划覆盖，不是测试结果。原 [64条完整期望](../vectors-v1.json) 保留，不改写为较弱完成条件。

| 原gate | 组件范围 | 仍需真实集成 |
| --- | --- | --- |
| SC-01 | K01 typed配置、15地址及service/reader注册、opaque context | 03身份映射；实际PID/native engine及多cid接线。 |
| SC-02 | 无原生执行证明 | native vote/certificate/resolver/body四channel真实产生/消费和签名字节证据。 |
| SC-03 | K01运输binding/context/大小/hash拒绝 | 原生epoch/namespace与证书验证、真实流认证。 |
| SC-04 | K03原子route plan、stale/crossinstance；K04receipt绑定 | root-step Ack、witness/aux/native IO、RecoveredAck与trusted floors。 |
| SC-05 | K02纯FIFO/跨link/channel/Unordered算法与固定jitter/tie | 同一算法接在native正常入口。 |
| SC-06 | K02容量/分区端点/drain/drop；K04保管不随overflow删除 | 真实持久outbox和网络恢复。 |
| SC-07 | K04显式durable确认状态机、纯record恢复 | 真实enqueue/send cut进程crash、fsync及01恢复。 |
| SC-08 | K04receive/ACK/delete记录、stable release身份、dedup | 真实receiver/sender crash、root去重/保管事务。 |
| SC-09 | K01同ID/context拒绝，K04历史message保留 | 终止后真实签名gate/历史转发及验证入口。 |
| SC-10 | K05 ClockTracker rate1投影与两个offset | 实际RPC请求/采样/响应、每次Commonware/Manager/governor读钟。 |
| SC-11 | K05样本原子拒绝、有效乱序、无view续租 | RPC丢失/乱序实际交付与calibration bridge。 |
| SC-12 | K05 half-open lease/rebind模型 | 原stop/Terminated持久状态与真实签名出口。 |
| SC-13 | K04原retry；K05绝对deadline换算 | 原生timer/work恢复、首次Enter/first-receive admission与startup fence。 |
| SC-14 | K06 reader destination与opaque custody隔离 | 06/03公共前缀/canonical/ReplayPosition，真实reader重启和业务状态。 |
| SC-15 | K07纯组件12组合；变量是配置声明顺序，不是worker启动 | 真实启动顺序与0/2ms墙钟暂停完整native trace。 |
| SC-16 | K02命名网络fault；K06无admin consumer effect | 真正进程crash/panic/watchdog/PID管理与基础设施分类。 |
| SC-17 | 不用组件代替旧回归 | 同候选I1a、Fresh与01全部受影响恢复验收。 |
| SC-18 | K03/K04/K05纯state/record/receipt负例 | 完整raw trace、真实提前ACK/假签名/持久丢失负例。 |

组件成功只记component milestone；功能02仍未accepted。当前组件也未运行。真正持久、BLS/原生证书、业务prefix、G/Inbox/D均没有通过声明。
