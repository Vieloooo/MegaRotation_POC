# 模块 08：Data Availability（v10 工作稿 v0.1）

日期：2026-11-09。上一版：[v9 v0.1](../formalization_v9/08_da_zh_v0_1.md)。总览与共享模型见 [00 arch](00_arch_zh_v0_1.md)；来源为 [arch v0.2 §1.5、§4.5 DAObject](../formalization_v8/arch_zh_v0_2.md) 与 [Part 0 v0.8 §2.5](../formalization_v6/part0_model_goals_zh_v0_8.md)。

## 1. 定位与直觉

DA 是外部服务，保存批次字节，使 L1 只需记录一个短的 commitment。三个对象各有分工：

- **commitment $d$**：标识“要取哪份数据”；
- **证书 $\pi_{\rm DA}$**：证明 $d$ 已按 DA 规则完成认证，因而在保留期内可以取回；
- **payload $y$**：取回后的实际字节，是 Derivation 解码与执行的输入。

验证证书与执行 payload 是两个步骤：证书只说明数据可取回，不说明内容在协议上是否合法。内容合法性由 Certifier 和 Derivation 检查。

本文件把 [00 §1.5](00_arch_zh_v0_1.md) 的模型写成接口和编号性质。

## 2. 最小假设

**本模块依赖的模型条目。**

- hash 与签名假设（[00 §1.1](00_arch_zh_v0_1.md)）：commitment binding 归约到这些假设，允许可忽略失败概率。
- 时间参数 $T_{\rm DA},\Delta_{\rm DA},T_{\rm ret}>0$（[00 §1.5](00_arch_zh_v0_1.md)）。

DA 是外部服务，本身不依赖其他模块。保留期之后的可取回性不由 DA 提供，而依赖下面的保管假设。

**A-CUSTODY（保管，仅用于活性）。** 对每份已被 Batch Inbox 接纳的批次 payload，DA 保留期结束后至少有一个诚实保管者（例如诚实的 Batcher 或派生节点）保存相同字节，至少保存到其后的某个派生 checkpoint、RESUME 切面或结算 checkpoint 已经覆盖它、派生与恢复都不再需要读取它为止。

**上游输入与下游输出。**

| 方向 | 对方模块与接口 | 交换对象 | 连接 |
| --- | --- | --- | --- |
| 输入 | Batcher.`Deliver` | 提交字节 $y$ | W16、W17 |
| 输出 | Batcher → Certifier、Inbox | $(d,\pi_{\rm DA})$ | W17 |
| 输出 | Derivation.`Derive`／`DeriveAtResume`，Certifier | DAObject | W21 |

## 3. 接口

### 3.1 状态

DA 维护存储表 $\mathsf{Store}:d\mapsto(y,\tau_{\rm cert}(d))$，其中 $\tau_{\rm cert}(d)$ 是 $d$ 完成认证的时刻。表项只增不改。$\mathsf{Store}$ 是分析用的抽象；读取者只通过下列接口访问它。

### 3.2 数据结构

```text
DAObject = {
    commitment, payload, certificate
}
```

三个字段均为完整比特串，对应 $d,y,\pi_{\rm DA}$。DAObject 表示已经取得并核对的数据：certificate 通过 CheckDA，payload 按 DA 方案的绑定关系打开 commitment。不用空 payload 表示尚未下载。

### 3.3 接口概览

| 接口 | 主要输入 | 主要输出 | 用途 |
| --- | --- | --- | --- |
| `Disperse` | payload $y$ | $(d,\pi_{\rm DA})$ 或 WAIT | 提交数据并取得认证 |
| `CheckDA` | $d$、$\pi_{\rm DA}$ | 0／1 | 公开验证证书 |
| `Retrieve` | $d$ | DAObject 或 WAIT | 取回完整 payload |

### 3.4 Disperse(y)

**输入。** 有限比特串 $y$，满足 DA 的准入条件（大小、费用）。

**输出。** 认证完成后返回 $(d,\pi_{\rm DA})$；认证完成前返回 WAIT。

**处理。** 计算 commitment，完成 DA 规则规定的分发与认证，在 $\mathsf{Store}$ 中记录 $(y,\tau_{\rm cert}(d))$。同一 $y$ 的重试返回相同 $d$。

### 3.5 CheckDA(d, π_DA)

**输入。** Commitment $d$ 与证书 $\pi_{\rm DA}$。

**输出。** 1 表示 $\pi_{\rm DA}$ 证明 $d$ 已按 DA 规则认证；否则为 0。

**处理。** 公开、确定的验证谓词，不需要访问 $\mathsf{Store}$，也不需要同步时钟。

### 3.6 Retrieve(d)

**输入。** Commitment $d$。

**输出。** DAObject；数据尚未取得时返回 WAIT。

**处理。** 取得 $y$，核对它打开 $d$ 后返回。不能用空内容或其他字节代替未知数据。

## 4. 性质

DA-S1、DA-S2 是安全性质，对所有执行成立；DA-L1、DA-L2、DA-RET 是活性性质。DA 不可用只影响活性。

**DA-S1 — Binding（safety）。** 除可忽略概率外，每个 $d$ 至多能被有效打开为一个 payload；`Retrieve(d)` 只返回这个 payload。

*直觉：* 一旦 Certifier 签了 $d$，L1 上接纳的就是唯一确定的一份字节，派生节点取回的内容不会因人而异。

**DA-S2 — Certificate consistency（safety）。** CheckDA 是公开确定的谓词：对同一 $(d,\pi_{\rm DA})$，所有验证者得到相同结果；若 $d$ 已认证，$\tau_{\rm cert}(d)$ 对所有参与方相同。

**DA-L1 — Dispersal latency（liveness）。** 若 DA 在 $[\tau,\tau+T_{\rm DA}]$ 上可用，在 $\tau$ 提交且满足准入条件的 `Disperse(y)` 在 $\tau+T_{\rm DA}$ 之前返回 $(d,\pi_{\rm DA})$，且 $\mathsf{CheckDA}(d,\pi_{\rm DA})=1$。

**DA-L2 — Retrieval within retention（liveness）。** 若 $d$ 已认证、$\tau\le\tau_{\rm cert}(d)+T_{\rm ret}$，且 DA 在 $[\tau,\tau+\Delta_{\rm DA}]$ 上可用，则在 $\tau$ 发出的 `Retrieve(d)` 在 $\tau+\Delta_{\rm DA}$ 之前返回 DAObject。

**DA-RET — Retrievability of accepted data（liveness）。** 每份已被 Batch Inbox 接纳的批次 payload，在 Derivation、状态准备或 RESUME 仍需要它时都可以取回。它由两部分组成：保留期 $T_{\rm ret}$ 之内由 DA-L2 保证；保留期之后由 A-CUSTODY 保证。DA-RET 是系统层的组合保证，不是 DA 服务单独提供的性质。

违反 DA-RET 时，派生停在该输入处等待，不把未知内容当成空；依赖派生的强制包含路径也随之等待。若缺失数据是计算某条 RESUME 的 ResumeCut 所必需的，该切面尚不可用，GFR 可重放的前缀停在该事件之前（[09 §3.4、§4.6](09_finality_rules_zh_v0_1.md)），取得切面后才原子完成替换。安全不受影响。
