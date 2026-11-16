# Mock Plan public interface v1 — M1

日期：2026-11-15。状态：**FROZEN for M1**。功能03 lead已批准本文件的名字、类型宽度、错误边界与规范编码，独立Verifier应在实现前固定对应向量。此接口提交只建立公共身份／引用，不计完整03或任何BFT/运行时验收。

授权来源为 [dependency-plan-v2](../commander/dependency-plan-v2.md)；它覆盖旧合同的等待整个01 gate。v10/v2的语义仍适用。03 lead是 `poc/crates/types/`、根manifest/lockfile的唯一writer。本文件只冻结 `poc_types::domain` 新增模块；02、06、09消费其版本化接口，不另定义相同身份和引用。

## 1. 兼容性和边界

`poc/crates/types/src/lib.rs` 仅新增 `pub mod domain;` 及确有需要的模块声明。现有 `wire`、`pub use wire::*`、I1常量、`hash_parts`、`digest`、`grant_hash`、`ack_hash`、`hex` 的名字、签名与行为完全保留；不把domain类型glob re-export到根而shadow已有wire类型。现有 `hash_parts` 仍返回 `Vec<u8>`，逐项使用u64BE字节长度 framing，不能为新Hash32改旧调用者。

M1是纯值类型及严格codec，不含Clock、reader delivery状态、网络认证、registry加载、PoP、证书验证、执行或I/O。引用不意味着对象已验证：`Cid` / `ConfigRef` 的32字节只提供类型区分，不能从其构造成功推出descriptor或配置有效。没有默认“对Cid/ConfigRef再hash得到身份”的方法。

## 2. 公共类型与操作

全部值类型实现 `Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash`，满足其字段的字典序；实现序列化时必须保持身份constructor校验。JSON不是协议hash输入，本接口不冻结JSON文本顺序。

| Rust名字 | 字段及合法域 | 最小构造／访问操作 |
| --- | --- | --- |
| `Hash32` | 私有 `[u8;32]` | `new([u8;32]) -> Self`；`TryFrom<&[u8]>`严格长度32；`as_bytes() -> &[u8;32]`；`into_bytes() -> [u8;32]`。 |
| `Cid` | 私有 `Hash32` | `new(Hash32) -> Self`；`as_hash() -> &Hash32`；`as_bytes() -> &[u8;32]`；`From<Hash32>`。 |
| `ConfigRef` | 私有 `Hash32` | 同Cid，独立类型，不可隐式互换。 |
| `RegionId` | 私有 `u8`，0..=2 | `new(u8) -> Result<Self,DomainError>`、`TryFrom<u8>`、`get()->u8`、`From<Self> for u8`。 |
| `MemberId` | 私有 `u8`，0..=4 | 同上。它是地区内公开member编号。 |
| `Pid` | 私有 `u8`，0..=14 | 同上；`from_region_member(RegionId,MemberId)->Self`、`region()->RegionId`、`member()->MemberId`。 |
| `SignerIndex` | 私有 `u8`，0..=4 | 同上。仅代表一个5人Commonware participant表的索引，不能由pid/member直接cast得出。 |
| `L1Position` | 公共 `block:u64, tx:u64, log:u64` | 位置从0计数，按 `(block,tx,log)` 字典序；无隐式秒／时间含义。 |
| `L1BlockRef` | 公共 `height:u64, hash:Hash32` | height相同hash不同仍是不同对象。 |
| `ResultKey` | 公共 `cid:Cid, block_hash:Hash32` | 对应v10区域结果键 `(cid,H(B))`，不包含proof表示或本地到达时间。 |

所有合法 `(region,member)` 满足 `Pid=5*region+member`，逆映射 `region=pid/5, member=pid%5`。新模块不给 `Pid→SignerIndex` 的算术捷径；03 registry在M2按实际identity key排序给出映射，02 endpoint认证和04签名各自使用适用身份。

Identity的Serde反序列化须通过 `try_from u8` / checked constructor，不能派生出绕过合法域的入口；JSON超过u8表示域的值在Serde解析层拒绝。Hash32反序列化只能接受精确32字节，Cid/ConfigRef保持同一限制。Serde表示不另提供unchecked public constructor。

## 3. 错误与解码顺序

`DomainError`是Rust结构化错误，实现 `Debug, Clone, PartialEq, Eq, Display, std::error::Error`。这里冻结语义variant与字段，不冻结网络错误数字或Display完整字符串：

```text
InvalidPid { value: u8 }
InvalidRegion { value: u8 }
InvalidMember { value: u8 }
InvalidSignerIndex { value: u8 }
InvalidLength { expected: usize, actual: usize }
UnsupportedVersion { version: u16 }
Malformed { reason: &'static str }
LimitExceeded { limit: usize, actual: usize }
Overflow
```

前四项用于对应checked constructor。`Hash32::try_from`长度不等32为InvalidLength。Canonical解码顺序固定为：先核对该类型的精确总长度，失败返回InvalidLength；再读两字节版本，不等1返回UnsupportedVersion；最后检查字段合法域。尾随字节、截断、空bytes都拒绝，不悄悄消耗一个前缀或接受默认值。Malformed/LimitExceeded/Overflow预留给M2有界可变对象，不声称M1已实现这些对象。

## 4. Canonical codec：BE v1

公共trait为：

```rust
pub trait Canonical: Sized {
    fn encode(&self) -> Vec<u8>;
    fn decode(bytes: &[u8]) -> Result<Self, DomainError>;
}
```

M1每个顶层值从 `u16BE(1)` 开始；其后严格按下表字段排列。小身份为一个u8，所有L1位置/height为u64BE，hash为原32字节。**嵌套字段使用raw字段，不重复写嵌套版本。** 本节编码没有type tag，因此bytes只能在已知目标类型的上下文解码，不能把同一bytes“自动猜成类型”。M2对象hash使用独立类型domain避免跨类型hash含义混用。

| 类型 | `version` 后的bytes | 总长度 |
| --- | --- | --- |
| Hash32、Cid、ConfigRef | 原32字节 | 34 |
| Pid、RegionId、MemberId、SignerIndex | `u8(value)` | 3 |
| L1Position | `u64BE(block) || u64BE(tx) || u64BE(log)` | 26 |
| L1BlockRef | `u64BE(height) || raw32(hash)` | 42 |
| ResultKey | `raw32(cid) || raw32(block_hash)` | 66 |

精确例子：

```text
Pid(14)            = 0001 0e
RegionId(2)        = 0001 02
MemberId(4)        = 0001 04
SignerIndex(4)     = 0001 04
L1Position(1,2,3)  = 0001 0000000000000001 0000000000000002 0000000000000003
Hash32([0;32])     = 0001 || 00 repeated 32
Cid([0x11;32])     = 0001 || 11 repeated 32
ConfigRef([0x22;32]) = 0001 || 22 repeated 32
L1BlockRef(7, [0x33;32]) = 0001 0000000000000007 || 33 repeated 32
ResultKey(Cid([0x11;32]), [0x22;32]) = 0001 || 11 repeated 32 || 22 repeated 32
```

空白和 `||` 仅为展示，不进入bytes。最后一例不是 `0001 0001 11… 0001 22…`。`Pid::decode(00010f)`为InvalidPid(15)，`RegionId::decode(000103)`为InvalidRegion(3)，`MemberId/SignerIndex::decode(000105)`分别拒绝。`Pid::decode(000200)`为UnsupportedVersion(2)；`Pid::decode(00010000)`先报InvalidLength(3,4)。

M2的生产hash规则由lead批准为 `hash_parts("PoC/<Type>/v1", &[canonical_bytes])`；现有framing精确为 `u64BE(len(domain)) || domain || u64BE(len(canonical_bytes)) || canonical_bytes`，再SHA-256。`<Type>`大小写和对象的唯一domain须在对应M2合同列清；M1不引入隐式opaque-id重hash。旧 `MPX-PREP-LE64-SHA256-v0` **不被采纳为生产codec**，历史向量保留以说明准备过程，不可作为M1字节期望。

## 5. M1验收和消费者使用

Verifier在 `poc/verification/mock-plan-execution/` 新路径保存M1独立输入与golden，不改旧准备expected。必须独立覆盖全部15 pid及双向映射、3地区/5member边界、signer索引与member的类型区别、Hash32长度31/32/33、版本/截断/尾随bytes、u64上界的L1Position roundtrip与字典序、同高度不同block hash、同cid不同body hash，以及上节精确编码和I1 `hash_parts`兼容向量。Serde坏身份不得绕过constructor。

02只需公共pid/地区/member和opaque cid/config即可写运输组件；06可用ResultKey标识公开proof/body，自己的日志key/消息位置归06，不能将ResultKey当已验证结果；09可用L1Position/L1BlockRef建立真实receipt位置映射，完整Control/Inbox ABI归09。完整 `Descriptor/PhysicalBlock/RegionalResult` 与公开 `FinalVerifier` trait在03的M2对象合同先冻结再实现；M1不加入bool验证桩。

M1发布须回传具体提交SHA、独立向量SHA、公共导出和兼容性检查结果。M1 PASS只释放消费者组件开发，不代表03完整accepted、01恢复验收、04真实证书验证或15-worker集成。后续增量接口另版本化，不能复用v1字节修改意义。
