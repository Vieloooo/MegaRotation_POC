//! The frozen M2 value format. Structural decoding is not proof verification.
use super::{
    Cid, ConfigRef, DomainError, Hash32, L1BlockRef, L1Position, Pid, RegionId, ResultKey,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MAX_TRANSACTIONS: usize = 1024;
pub const MAX_TRANSACTION_BYTES: usize = 65_536;
pub const MAX_FIXTURE_ROWS: usize = 4096;

pub trait CanonicalObject: Sized {
    fn encode_checked(&self) -> Result<Vec<u8>, DomainError>;
    fn decode(bytes: &[u8]) -> Result<Self, DomainError>;
}

fn bad(reason: &'static str) -> DomainError {
    DomainError::Malformed { reason }
}
fn ensure(ok: bool, reason: &'static str) -> Result<(), DomainError> {
    if ok { Ok(()) } else { Err(bad(reason)) }
}
fn bound(actual: usize, limit: usize) -> Result<(), DomainError> {
    if actual <= limit {
        Ok(())
    } else {
        Err(DomainError::LimitExceeded { limit, actual })
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], DomainError> {
        let end = self.offset.checked_add(n).ok_or(DomainError::Overflow)?;
        let bytes = self
            .bytes
            .get(self.offset..end)
            .ok_or(bad("truncated object"))?;
        self.offset = end;
        Ok(bytes)
    }
    fn count(&mut self, max: usize) -> Result<usize, DomainError> {
        let n = u32::read(self)? as usize;
        bound(n, max)?;
        Ok(n)
    }
}
trait Raw: Sized {
    fn write(&self, out: &mut Vec<u8>) -> Result<(), DomainError>;
    fn read(input: &mut Reader<'_>) -> Result<Self, DomainError>;
}
macro_rules! integer {
    ($($t:ty),+) => {$ (
        impl Raw for $t {
            fn write(&self, out: &mut Vec<u8>) -> Result<(), DomainError> { out.extend_from_slice(&self.to_be_bytes()); Ok(()) }
            fn read(input: &mut Reader<'_>) -> Result<Self, DomainError> {
                Ok(Self::from_be_bytes(input.take(std::mem::size_of::<Self>())?.try_into().map_err(|_| bad("integer"))?))
            }
        }
    )+};
}
integer!(u8, u16, u32, u64);
impl<const N: usize> Raw for [u8; N] {
    fn write(&self, out: &mut Vec<u8>) -> Result<(), DomainError> {
        out.extend_from_slice(self);
        Ok(())
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, DomainError> {
        input.take(N)?.try_into().map_err(|_| bad("array"))
    }
}
impl Raw for Hash32 {
    fn write(&self, out: &mut Vec<u8>) -> Result<(), DomainError> {
        self.as_bytes().write(out)
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, DomainError> {
        Ok(Self::new(<[u8; 32]>::read(input)?))
    }
}
macro_rules! hash_ref {
    ($($t:ty),+) => {$ (
        impl Raw for $t {
            fn write(&self, out: &mut Vec<u8>) -> Result<(), DomainError> { self.as_hash().write(out) }
            fn read(input: &mut Reader<'_>) -> Result<Self, DomainError> { Ok(Self::new(Hash32::read(input)?)) }
        }
    )+};
}
hash_ref!(Cid, ConfigRef);
macro_rules! small_id {
    ($($t:ty),+) => {$ (
        impl Raw for $t {
            fn write(&self, out: &mut Vec<u8>) -> Result<(), DomainError> { self.get().write(out) }
            fn read(input: &mut Reader<'_>) -> Result<Self, DomainError> { Self::new(u8::read(input)?) }
        }
    )+};
}
small_id!(Pid, RegionId);
impl<T: Raw> Raw for Option<T> {
    fn write(&self, out: &mut Vec<u8>) -> Result<(), DomainError> {
        match self {
            None => 0u8.write(out),
            Some(v) => {
                1u8.write(out)?;
                v.write(out)
            }
        }
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, DomainError> {
        match u8::read(input)? {
            0 => Ok(None),
            1 => Ok(Some(T::read(input)?)),
            _ => Err(bad("option tag")),
        }
    }
}
fn write_list<T: Raw>(values: &[T], max: usize, out: &mut Vec<u8>) -> Result<(), DomainError> {
    bound(values.len(), max)?;
    (values.len() as u32).write(out)?;
    for v in values {
        v.write(out)?;
    }
    Ok(())
}
fn read_list<T: Raw>(input: &mut Reader<'_>, max: usize) -> Result<Vec<T>, DomainError> {
    let n = input.count(max)?;
    (0..n).map(|_| T::read(input)).collect()
}

macro_rules! object {
    ($t:ty) => {
        impl CanonicalObject for $t {
            fn encode_checked(&self) -> Result<Vec<u8>, DomainError> {
                let mut out = 1u16.to_be_bytes().to_vec();
                self.write(&mut out)?;
                Ok(out)
            }
            fn decode(bytes: &[u8]) -> Result<Self, DomainError> {
                let mut input = Reader { bytes, offset: 0 };
                let version = u16::read(&mut input)?;
                if version != 1 {
                    return Err(DomainError::UnsupportedVersion { version });
                }
                let result = Self::read(&mut input)?;
                ensure(input.offset == bytes.len(), "trailing bytes")?;
                Ok(result)
            }
        }
    };
}
macro_rules! fixed_raw {
    ($t:ty, $($field:ident : $ft:ty),+ $(,)?) => {
        impl Raw for $t {
            fn write(&self, out: &mut Vec<u8>) -> Result<(), DomainError> {
                self.validate()?; $(self.$field.write(out)?;)+ Ok(())
            }
            fn read(input: &mut Reader<'_>) -> Result<Self, DomainError> {
                let v = Self { $($field: <$ft>::read(input)?,)+ }; v.validate()?; Ok(v)
            }
        }
        object!($t);
    };
}
macro_rules! plain {
    ($name:ident { $($f:ident : $t:ty),+ $(,)? }) => {
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name { $(pub $f: $t,)+ }
    };
}
macro_rules! valid {
    ($($t:ty),+) => {$ (impl $t { pub fn validate(&self) -> Result<(), DomainError> { Ok(()) } })+};
}

plain!(SessionId { beta: u64, r: u64 });
plain!(Window {
    h_min: u64,
    h_max: u64
});
plain!(Metadata {
    height: u64,
    timestamp_us: u64,
    origin: u64
});
plain!(BlockRef {
    height: u64,
    origin: u64,
    block_id: Hash32
});
plain!(Anchor {
    block: BlockRef,
    timestamp_us: u64
});
plain!(Account {
    nonce: u64,
    balance: u64
});
plain!(ExecutionState { metadata: Metadata, accounts: BTreeMap<u8, Account> });
plain!(ExecutionAnchor {
    anchor: Anchor,
    state: ExecutionState,
    native_hash: Hash32
});
valid!(SessionId, Metadata, BlockRef, Anchor, Account);
impl Window {
    pub fn validate(&self) -> Result<(), DomainError> {
        ensure(
            self.h_max
                .checked_sub(self.h_min)
                .and_then(|x| x.checked_add(1))
                == Some(10),
            "window must contain ten heights",
        )
    }
}
impl ExecutionState {
    pub fn validate(&self) -> Result<(), DomainError> {
        bound(self.accounts.len(), 256)?;
        ensure(
            self.accounts
                .values()
                .all(|a| a.nonce != 0 || a.balance != 0),
            "explicit zero account",
        )
    }
}
impl ExecutionAnchor {
    pub fn validate(&self) -> Result<(), DomainError> {
        self.state.validate()?;
        ensure(
            self.state.metadata == self.anchor.metadata(),
            "anchor state metadata",
        )
    }
}
impl Anchor {
    pub fn metadata(&self) -> Metadata {
        Metadata {
            height: self.block.height,
            timestamp_us: self.timestamp_us,
            origin: self.block.origin,
        }
    }
}
fixed_raw!(SessionId, beta:u64, r:u64);
fixed_raw!(Window, h_min:u64, h_max:u64);
fixed_raw!(Metadata, height:u64, timestamp_us:u64, origin:u64);
fixed_raw!(BlockRef, height:u64, origin:u64, block_id:Hash32);
fixed_raw!(Anchor, block:BlockRef, timestamp_us:u64);
fixed_raw!(Account, nonce:u64, balance:u64);
fixed_raw!(ExecutionAnchor, anchor:Anchor, state:ExecutionState, native_hash:Hash32);
impl Raw for ExecutionState {
    fn write(&self, out: &mut Vec<u8>) -> Result<(), DomainError> {
        self.validate()?;
        self.metadata.write(out)?;
        (self.accounts.len() as u32).write(out)?;
        for (id, a) in &self.accounts {
            id.write(out)?;
            a.write(out)?;
        }
        Ok(())
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, DomainError> {
        let metadata = Metadata::read(input)?;
        let n = input.count(256)?;
        let mut accounts = BTreeMap::new();
        let mut last = None;
        for _ in 0..n {
            let id = u8::read(input)?;
            ensure(last.is_none_or(|x| x < id), "account order")?;
            accounts.insert(id, Account::read(input)?);
            last = Some(id);
        }
        let v = Self { metadata, accounts };
        v.validate()?;
        Ok(v)
    }
}
object!(ExecutionState);

mod array_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub fn serialize<S: Serializer, const N: usize>(v: &[u8; N], s: S) -> Result<S::Ok, S::Error> {
        v.as_slice().serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>, const N: usize>(
        d: D,
    ) -> Result<[u8; N], D::Error> {
        let v = Vec::<u8>::deserialize(d)?;
        v.try_into().map_err(|v: Vec<u8>| {
            serde::de::Error::custom(format!("expected {N} bytes, got {}", v.len()))
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryEntry {
    pub pid: Pid,
    pub identity_key: [u8; 32],
    #[serde(with = "array_serde")]
    pub vote_key: [u8; 96],
    #[serde(with = "array_serde")]
    pub pop: [u8; 48],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitteeEntry {
    pub pid: Pid,
    pub identity_key: [u8; 32],
    #[serde(with = "array_serde")]
    pub vote_key: [u8; 96],
    pub weight: u64,
}
plain!(RegistryRecord { entries: Vec<RegistryEntry> });
valid!(RegistryEntry);
impl CommitteeEntry {
    pub fn validate(&self) -> Result<(), DomainError> {
        ensure(self.weight == 1, "committee weight")
    }
}
impl RegistryRecord {
    pub fn validate(&self) -> Result<(), DomainError> {
        ensure(self.entries.len() == 15, "registry size")?;
        ensure(
            self.entries
                .iter()
                .enumerate()
                .all(|(i, e)| e.pid.get() as usize == i),
            "registry pid order",
        )
    }
}
fixed_raw!(RegistryEntry,pid:Pid,identity_key:[u8;32],vote_key:[u8;96],pop:[u8;48]);
fixed_raw!(CommitteeEntry,pid:Pid,identity_key:[u8;32],vote_key:[u8;96],weight:u64);
impl Raw for RegistryRecord {
    fn write(&self, out: &mut Vec<u8>) -> Result<(), DomainError> {
        self.validate()?;
        write_list(&self.entries, 15, out)
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, DomainError> {
        let v = Self {
            entries: read_list(input, 15)?,
        };
        v.validate()?;
        Ok(v)
    }
}
object!(RegistryRecord);

plain!(RuleSpec { rule_id:u16, version:u16, params:Vec<u8> });
impl RuleSpec {
    pub fn static_schedule() -> Self {
        Self {
            rule_id: 1,
            version: 1,
            params: vec![],
        }
    }
    pub fn validate(&self) -> Result<(), DomainError> {
        ensure(
            self.rule_id == 1 && self.version == 1 && self.params.is_empty(),
            "unsupported schedule rule",
        )
    }
}
impl Raw for RuleSpec {
    fn write(&self, out: &mut Vec<u8>) -> Result<(), DomainError> {
        self.validate()?;
        self.rule_id.write(out)?;
        self.version.write(out)?;
        0u32.write(out)
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, DomainError> {
        let v = Self {
            rule_id: u16::read(input)?,
            version: u16::read(input)?,
            params: read_list(input, 0)?,
        };
        v.validate()?;
        Ok(v)
    }
}
object!(RuleSpec);
plain!(PublicConfig {
    chain_id: u64,
    rule_version: u16,
    verifier_version: u16,
    registry_hash: Hash32,
    dep_fixture_hash: Hash32,
    l1_fixture_hash: Hash32,
    genesis: Anchor,
    genesis_state_root: Hash32,
    genesis_native_hash: Hash32,
    l2_slot_us: u64,
    l1_genesis_timestamp_us: u64,
    l1_block_us: u64,
    t_dep_us: u64,
    t_drift_us: u64,
    metadata_window_us: u64,
    byte_limit: u32
});
impl PublicConfig {
    pub fn validate(&self) -> Result<(), DomainError> {
        ensure(
            self.rule_version == 1 && self.verifier_version == 1,
            "unsupported config version",
        )?;
        ensure(
            self.l2_slot_us > 0 && self.l1_block_us > 0 && self.l2_slot_us <= self.l1_block_us,
            "invalid slot interval",
        )?;
        let drift = self
            .t_dep_us
            .checked_add(self.l1_block_us)
            .ok_or(DomainError::Overflow)?;
        ensure(drift <= self.t_drift_us, "invalid deposit lag/drift")?;
        ensure(
            self.byte_limit > 0 && self.byte_limit as usize <= MAX_TRANSACTION_BYTES,
            "invalid byte limit",
        )
    }
    pub fn reference(&self) -> Result<ConfigRef, DomainError> {
        Ok(ConfigRef::new(object_hash("PoC/PublicConfig/v1", self)?))
    }
}
fixed_raw!(PublicConfig,chain_id:u64,rule_version:u16,verifier_version:u16,registry_hash:Hash32,dep_fixture_hash:Hash32,l1_fixture_hash:Hash32,genesis:Anchor,genesis_state_root:Hash32,genesis_native_hash:Hash32,l2_slot_us:u64,l1_genesis_timestamp_us:u64,l1_block_us:u64,t_dep_us:u64,t_drift_us:u64,metadata_window_us:u64,byte_limit:u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryCause {
    Genesis,
    Handoff,
    Force,
    Resume,
}
impl Raw for BoundaryCause {
    fn write(&self, out: &mut Vec<u8>) -> Result<(), DomainError> {
        (*self as u8).write(out)
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, DomainError> {
        match u8::read(input)? {
            0 => Ok(Self::Genesis),
            1 => Ok(Self::Handoff),
            2 => Ok(Self::Force),
            3 => Ok(Self::Resume),
            _ => Err(bad("boundary cause")),
        }
    }
}
plain!(PlanInput { cause:BoundaryCause, session:SessionId, source_region:Option<RegionId>, window:Window, reference_block:BlockRef });
plain!(Boundary { input:PlanInput, anchor:Option<Anchor> });
impl PlanInput {
    pub fn validate(&self) -> Result<(), DomainError> {
        self.window.validate()
    }
}
impl Boundary {
    pub fn validate(&self) -> Result<(), DomainError> {
        self.input.validate()
    }
}
fixed_raw!(PlanInput,cause:BoundaryCause,session:SessionId,source_region:Option<RegionId>,window:Window,reference_block:BlockRef);
fixed_raw!(Boundary,input:PlanInput,anchor:Option<Anchor>);
plain!(PlanRecord {session:SessionId,cause:BoundaryCause,source_region:Option<RegionId>,region:RegionId,committee:Vec<CommitteeEntry>,total_weight:u64,fault_bound:u64,eligible_proposers:Vec<Pid>,leader_rule:RuleSpec,window:Window,reference_block:BlockRef,config_ref:ConfigRef,authorization_ref:Hash32});
plain!(ProgramConfig {chain_id:u64,rule_version:u16,verifier_version:u16,config_ref:ConfigRef,committee:Vec<CommitteeEntry>,fault_bound:u64,eligible_proposers:Vec<Pid>,leader_rule:RuleSpec,window:Window,byte_limit:u32,metadata_window_us:u64});
fn committee_valid(
    committee: &[CommitteeEntry],
    eligible: &[Pid],
    fault: u64,
) -> Result<(), DomainError> {
    ensure(
        committee.len() == 5 && eligible.len() == 5 && fault == 1,
        "committee profile",
    )?;
    let region = committee[0].pid.region();
    for (i, e) in committee.iter().enumerate() {
        e.validate()?;
        ensure(
            e.pid.region() == region && e.pid.member().get() as usize == i && eligible[i] == e.pid,
            "committee membership/order",
        )?;
    }
    Ok(())
}
impl PlanRecord {
    pub fn input(&self) -> PlanInput {
        PlanInput {
            cause: self.cause,
            session: self.session.clone(),
            source_region: self.source_region,
            window: self.window.clone(),
            reference_block: self.reference_block.clone(),
        }
    }
    pub fn validate(&self) -> Result<(), DomainError> {
        committee_valid(&self.committee, &self.eligible_proposers, self.fault_bound)?;
        ensure(
            self.total_weight == 5 && self.committee[0].pid.region() == self.region,
            "plan region/weight",
        )?;
        self.leader_rule.validate()?;
        self.window.validate()
    }
}
impl ProgramConfig {
    pub fn validate(&self) -> Result<(), DomainError> {
        committee_valid(&self.committee, &self.eligible_proposers, self.fault_bound)?;
        ensure(
            self.rule_version == 1 && self.verifier_version == 1,
            "unsupported program version",
        )?;
        ensure(
            self.byte_limit > 0 && self.byte_limit as usize <= MAX_TRANSACTION_BYTES,
            "invalid byte limit",
        )?;
        self.leader_rule.validate()?;
        self.window.validate()
    }
    pub fn from_plan(config: &PublicConfig, plan: &PlanRecord) -> Result<Self, DomainError> {
        config.validate()?;
        plan.validate()?;
        ensure(
            config.reference()? == plan.config_ref,
            "plan config reference",
        )?;
        let value = Self {
            chain_id: config.chain_id,
            rule_version: config.rule_version,
            verifier_version: config.verifier_version,
            config_ref: plan.config_ref,
            committee: plan.committee.clone(),
            fault_bound: plan.fault_bound,
            eligible_proposers: plan.eligible_proposers.clone(),
            leader_rule: plan.leader_rule.clone(),
            window: plan.window.clone(),
            byte_limit: config.byte_limit,
            metadata_window_us: config.metadata_window_us,
        };
        value.validate()?;
        Ok(value)
    }
}
impl Raw for PlanRecord {
    fn write(&self, o: &mut Vec<u8>) -> Result<(), DomainError> {
        self.validate()?;
        self.session.write(o)?;
        self.cause.write(o)?;
        self.source_region.write(o)?;
        self.region.write(o)?;
        write_list(&self.committee, 5, o)?;
        self.total_weight.write(o)?;
        self.fault_bound.write(o)?;
        write_list(&self.eligible_proposers, 5, o)?;
        self.leader_rule.write(o)?;
        self.window.write(o)?;
        self.reference_block.write(o)?;
        self.config_ref.write(o)?;
        self.authorization_ref.write(o)
    }
    fn read(r: &mut Reader<'_>) -> Result<Self, DomainError> {
        let v = Self {
            session: SessionId::read(r)?,
            cause: BoundaryCause::read(r)?,
            source_region: Option::read(r)?,
            region: RegionId::read(r)?,
            committee: read_list(r, 5)?,
            total_weight: u64::read(r)?,
            fault_bound: u64::read(r)?,
            eligible_proposers: read_list(r, 5)?,
            leader_rule: RuleSpec::read(r)?,
            window: Window::read(r)?,
            reference_block: BlockRef::read(r)?,
            config_ref: ConfigRef::read(r)?,
            authorization_ref: Hash32::read(r)?,
        };
        v.validate()?;
        Ok(v)
    }
}
object!(PlanRecord);
impl Raw for ProgramConfig {
    fn write(&self, o: &mut Vec<u8>) -> Result<(), DomainError> {
        self.validate()?;
        self.chain_id.write(o)?;
        self.rule_version.write(o)?;
        self.verifier_version.write(o)?;
        self.config_ref.write(o)?;
        write_list(&self.committee, 5, o)?;
        self.fault_bound.write(o)?;
        write_list(&self.eligible_proposers, 5, o)?;
        self.leader_rule.write(o)?;
        self.window.write(o)?;
        self.byte_limit.write(o)?;
        self.metadata_window_us.write(o)
    }
    fn read(r: &mut Reader<'_>) -> Result<Self, DomainError> {
        let v = Self {
            chain_id: u64::read(r)?,
            rule_version: u16::read(r)?,
            verifier_version: u16::read(r)?,
            config_ref: ConfigRef::read(r)?,
            committee: read_list(r, 5)?,
            fault_bound: u64::read(r)?,
            eligible_proposers: read_list(r, 5)?,
            leader_rule: RuleSpec::read(r)?,
            window: Window::read(r)?,
            byte_limit: u32::read(r)?,
            metadata_window_us: u64::read(r)?,
        };
        v.validate()?;
        Ok(v)
    }
}
object!(ProgramConfig);
plain!(Descriptor {
    plan: PlanRecord,
    anchor: Anchor,
    cid: Cid
});
impl Descriptor {
    pub fn validate(&self) -> Result<(), DomainError> {
        self.plan.validate()?;
        ensure(
            self.anchor.block.height < self.plan.window.h_min,
            "anchor outside window",
        )
    }
}
fixed_raw!(Descriptor,plan:PlanRecord,anchor:Anchor,cid:Cid);

plain!(Transfer {
    from: u8,
    to: u8,
    nonce: u64,
    amount: u64
});
plain!(Credit {
    to: u8,
    amount: u64
});
plain!(Attributes { metadata: Metadata });
macro_rules! transaction {
    ($t:ty,$tag:expr,$($f:ident:$ft:ty),+) => {
        impl $t {
            pub fn encode_raw(&self)->Vec<u8>{let mut out=vec![$tag];$(self.$f.write(&mut out).expect("fixed transaction field");)+out}
            pub fn decode_raw(bytes:&[u8])->Result<Self,DomainError>{let mut r=Reader{bytes,offset:0};ensure(u8::read(&mut r)?==$tag,"transaction opcode")?;let v=Self{$($f:<$ft>::read(&mut r)?,)+};ensure(r.offset==bytes.len(),"transaction trailing bytes")?;Ok(v)}
        }
    };
}
transaction!(Transfer,1,from:u8,to:u8,nonce:u64,amount:u64);
transaction!(Credit,2,to:u8,amount:u64);
transaction!(Attributes,3,metadata:Metadata);
impl Raw for Credit {
    fn write(&self, o: &mut Vec<u8>) -> Result<(), DomainError> {
        o.extend_from_slice(&self.encode_raw());
        Ok(())
    }
    fn read(r: &mut Reader<'_>) -> Result<Self, DomainError> {
        Self::decode_raw(r.take(10)?)
    }
}
pub fn validate_transactions(txs: &[Vec<u8>], byte_limit: usize) -> Result<(), DomainError> {
    bound(txs.len(), MAX_TRANSACTIONS)?;
    let mut total = 0usize;
    for tx in txs {
        Transfer::decode_raw(tx)?;
        total = total.checked_add(tx.len()).ok_or(DomainError::Overflow)?;
    }
    bound(total, byte_limit.min(MAX_TRANSACTION_BYTES))
}
pub fn validate_dep(metadata: &Metadata, dep: &[Vec<u8>]) -> Result<(), DomainError> {
    ensure(!dep.is_empty(), "missing attributes")?;
    bound(dep.len(), MAX_TRANSACTIONS + 1)?;
    ensure(
        Attributes::decode_raw(&dep[0])?.metadata == *metadata,
        "attributes metadata",
    )?;
    for credit in &dep[1..] {
        Credit::decode_raw(credit)?;
    }
    Ok(())
}
fn write_blobs(blobs: &[Vec<u8>], out: &mut Vec<u8>) -> Result<(), DomainError> {
    (blobs.len() as u32).write(out)?;
    for b in blobs {
        (b.len() as u32).write(out)?;
        out.extend_from_slice(b);
    }
    Ok(())
}
fn read_blobs(
    r: &mut Reader<'_>,
    max: usize,
    max_bytes: usize,
) -> Result<Vec<Vec<u8>>, DomainError> {
    let n = r.count(max)?;
    let mut out = Vec::with_capacity(n);
    let mut total = 0usize;
    for _ in 0..n {
        let len = r.count(25)?;
        total = total.checked_add(len).ok_or(DomainError::Overflow)?;
        bound(total, max_bytes)?;
        out.push(r.take(len)?.to_vec());
    }
    Ok(out)
}
plain!(PhysicalBlock {parent:Hash32,metadata:Metadata,pre_state_root:Hash32,transactions:Vec<Vec<u8>>});
plain!(NativeBlock {parent:Hash32,metadata:Metadata,dep:Vec<Vec<u8>>,kept:Vec<Vec<u8>>,post_state_root:Hash32});
impl PhysicalBlock {
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_transactions(&self.transactions, MAX_TRANSACTION_BYTES)
    }
}
impl NativeBlock {
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_dep(&self.metadata, &self.dep)?;
        validate_transactions(&self.kept, MAX_TRANSACTION_BYTES)
    }
}
impl Raw for PhysicalBlock {
    fn write(&self, o: &mut Vec<u8>) -> Result<(), DomainError> {
        self.validate()?;
        self.parent.write(o)?;
        self.metadata.write(o)?;
        self.pre_state_root.write(o)?;
        write_blobs(&self.transactions, o)
    }
    fn read(r: &mut Reader<'_>) -> Result<Self, DomainError> {
        let v = Self {
            parent: Hash32::read(r)?,
            metadata: Metadata::read(r)?,
            pre_state_root: Hash32::read(r)?,
            transactions: read_blobs(r, MAX_TRANSACTIONS, MAX_TRANSACTION_BYTES)?,
        };
        v.validate()?;
        Ok(v)
    }
}
object!(PhysicalBlock);
impl Raw for NativeBlock {
    fn write(&self, o: &mut Vec<u8>) -> Result<(), DomainError> {
        self.validate()?;
        self.parent.write(o)?;
        self.metadata.write(o)?;
        write_blobs(&self.dep, o)?;
        write_blobs(&self.kept, o)?;
        self.post_state_root.write(o)
    }
    fn read(r: &mut Reader<'_>) -> Result<Self, DomainError> {
        let v = Self {
            parent: Hash32::read(r)?,
            metadata: Metadata::read(r)?,
            dep: read_blobs(r, MAX_TRANSACTIONS + 1, 25 + 10 * MAX_TRANSACTIONS)?,
            kept: read_blobs(r, MAX_TRANSACTIONS, MAX_TRANSACTION_BYTES)?,
            post_state_root: Hash32::read(r)?,
        };
        v.validate()?;
        Ok(v)
    }
}
object!(NativeBlock);

plain!(DepFixture {first_height:u64,rows:Vec<Vec<Credit>>});
plain!(L1Header {
    height: u64,
    hash: Hash32,
    parent_hash: Hash32,
    timestamp_s: u64
});
plain!(L1FixtureRow {
    header: L1Header,
    finalized_at_us: u64,
    empty_control_commitment: Hash32
});
plain!(L1Fixture {rows:Vec<L1FixtureRow>});
impl DepFixture {
    pub fn validate(&self) -> Result<(), DomainError> {
        ensure(!self.rows.is_empty(), "empty deposit fixture")?;
        bound(self.rows.len(), MAX_FIXTURE_ROWS)?;
        self.first_height
            .checked_add(self.rows.len() as u64 - 1)
            .ok_or(DomainError::Overflow)?;
        for row in &self.rows {
            bound(row.len(), MAX_TRANSACTIONS)?;
        }
        Ok(())
    }
}
impl L1Header {
    pub fn timestamp_us(&self) -> Result<u64, DomainError> {
        self.timestamp_s
            .checked_mul(1_000_000)
            .ok_or(DomainError::Overflow)
    }
    pub fn validate(&self) -> Result<(), DomainError> {
        self.timestamp_us().map(|_| ())
    }
    pub fn reference(&self) -> L1BlockRef {
        L1BlockRef {
            height: self.height,
            hash: self.hash,
        }
    }
}
impl L1FixtureRow {
    pub fn validate(&self) -> Result<(), DomainError> {
        ensure(
            self.finalized_at_us >= self.header.timestamp_us()?,
            "premature source finality",
        )?;
        ensure(
            self.empty_control_commitment == empty_control_commitment(),
            "nonempty or invalid fixture controls",
        )
    }
}
impl L1Fixture {
    pub fn validate(&self) -> Result<(), DomainError> {
        ensure(!self.rows.is_empty(), "empty L1 fixture")?;
        bound(self.rows.len(), MAX_FIXTURE_ROWS)?;
        let mut previous: Option<&L1FixtureRow> = None;
        for (i, row) in self.rows.iter().enumerate() {
            row.validate()?;
            ensure(row.header.height == i as u64, "L1 fixture heights")?;
            if let Some(p) = previous {
                ensure(
                    row.header.parent_hash == p.header.hash
                        && row.header.timestamp_s > p.header.timestamp_s,
                    "L1 fixture linkage",
                )?;
            } else {
                ensure(
                    row.header.parent_hash == Hash32::new([0; 32]),
                    "genesis parent",
                )?;
            }
            previous = Some(row);
        }
        Ok(())
    }
}
fixed_raw!(L1Header,height:u64,hash:Hash32,parent_hash:Hash32,timestamp_s:u64);
fixed_raw!(L1FixtureRow,header:L1Header,finalized_at_us:u64,empty_control_commitment:Hash32);
impl Raw for DepFixture {
    fn write(&self, o: &mut Vec<u8>) -> Result<(), DomainError> {
        self.validate()?;
        self.first_height.write(o)?;
        (self.rows.len() as u32).write(o)?;
        for row in &self.rows {
            write_list(row, MAX_TRANSACTIONS, o)?;
        }
        Ok(())
    }
    fn read(r: &mut Reader<'_>) -> Result<Self, DomainError> {
        let first_height = u64::read(r)?;
        let n = r.count(MAX_FIXTURE_ROWS)?;
        let rows = (0..n)
            .map(|_| read_list(r, MAX_TRANSACTIONS))
            .collect::<Result<_, _>>()?;
        let v = Self { first_height, rows };
        v.validate()?;
        Ok(v)
    }
}
object!(DepFixture);
impl Raw for L1Fixture {
    fn write(&self, o: &mut Vec<u8>) -> Result<(), DomainError> {
        self.validate()?;
        write_list(&self.rows, MAX_FIXTURE_ROWS, o)
    }
    fn read(r: &mut Reader<'_>) -> Result<Self, DomainError> {
        let v = Self {
            rows: read_list(r, MAX_FIXTURE_ROWS)?,
        };
        v.validate()?;
        Ok(v)
    }
}
object!(L1Fixture);

fn hash(domain: &str, bytes: &[u8]) -> Hash32 {
    let v = crate::hash_parts(domain, &[bytes]);
    let mut out = [0; 32];
    out.copy_from_slice(&v);
    Hash32::new(out)
}
fn object_hash<T: CanonicalObject>(domain: &str, value: &T) -> Result<Hash32, DomainError> {
    Ok(hash(domain, &value.encode_checked()?))
}
macro_rules! digest_method {
    ($t:ty,$method:ident,$domain:literal) => {
        impl $t {
            pub fn $method(&self) -> Result<Hash32, DomainError> {
                object_hash($domain, self)
            }
        }
    };
}
digest_method!(RegistryRecord, digest, "PoC/Registry/v1");
digest_method!(ExecutionState, root, "PoC/ExecutionState/v1");
digest_method!(PhysicalBlock, digest, "PoC/PhysicalBlock/v1");
digest_method!(NativeBlock, digest, "PoC/NativeBlock/v1");
digest_method!(DepFixture, digest, "PoC/DepFixture/v1");
digest_method!(L1Fixture, digest, "PoC/L1Fixture/v1");
pub fn authorization_ref(
    config: ConfigRef,
    registry: Hash32,
    input: &PlanInput,
) -> Result<Hash32, DomainError> {
    let mut bytes = 1u16.to_be_bytes().to_vec();
    config.write(&mut bytes)?;
    registry.write(&mut bytes)?;
    input.write(&mut bytes)?;
    Ok(hash("PoC/Authorization/v1", &bytes))
}
pub fn regional_cid(
    session: &SessionId,
    anchor: Hash32,
    program: &ProgramConfig,
) -> Result<Cid, DomainError> {
    let mut bytes = 1u16.to_be_bytes().to_vec();
    session.write(&mut bytes)?;
    anchor.write(&mut bytes)?;
    program.write(&mut bytes)?;
    Ok(Cid::new(hash("PoC/Regional/v1", &bytes)))
}
pub fn derived_anchor_id(height: u64, native_hash: Hash32) -> Hash32 {
    let mut bytes = 1u16.to_be_bytes().to_vec();
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(native_hash.as_bytes());
    hash("PoC/DerivedAnchor/v1", &bytes)
}
pub fn empty_control_commitment() -> Hash32 {
    hash("PoC/EmptyL1Control/v1", &[0, 1, 0, 0, 0, 0])
}

// These typed capabilities deliberately have no canonical bytes or native verifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputMissing {
    Reference(BlockRef),
    L1Header(u64),
    ControlCoverage(u64),
    Parent(Hash32),
    Dep(u64),
    State(Hash32),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputError {
    InvalidSource,
    Conflict,
    FixtureCorrupt,
    FixtureExhausted {
        requested: u64,
        first: u64,
        last: u64,
    },
    Malformed(DomainError),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputStatus<T> {
    Ready(T),
    Wait(InputMissing),
    Reject(InputError),
}
pub trait PublicReferenceView {
    fn reference(&self, reference: &BlockRef) -> InputStatus<Anchor>;
}
pub trait FinalizedL1Reader {
    fn header(&self, height: u64) -> InputStatus<L1Header>;
    fn require_control_coverage(&self, through_height: u64) -> InputStatus<()>;
    fn finalized_tip(&self) -> Option<L1BlockRef>;
}
pub trait FinalVerifier {
    fn check(&self, descriptor: &Descriptor, block: &PhysicalBlock, proof: &[u8]) -> bool;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayKind {
    Force,
    Resume,
    Expire,
    SessionEnd,
    Espresso,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayIndex {
    L1(L1Position),
    Height(u64),
    Session(SessionId),
    Espresso(u64),
}
plain!(ReplayPosition {
    key_us: u64,
    kind: ReplayKind,
    index: ReplayIndex
});
impl ReplayPosition {
    pub fn validate(&self) -> Result<(), DomainError> {
        ensure(
            matches!(
                (&self.kind, &self.index),
                (ReplayKind::Force | ReplayKind::Resume, ReplayIndex::L1(_))
                    | (ReplayKind::Expire, ReplayIndex::Height(_))
                    | (ReplayKind::SessionEnd, ReplayIndex::Session(_))
                    | (ReplayKind::Espresso, ReplayIndex::Espresso(_))
            ),
            "replay index kind",
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnchorEvidenceSource {
    Genesis(ConfigRef),
    Regional(ResultKey),
    Force(ReplayPosition),
    Resume(L1Position),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorEvidenceKind {
    Genesis,
    Regional,
    Force,
    Resume,
}
plain!(AnchorEvidence {source:AnchorEvidenceSource,source_objects:Vec<Vec<u8>>});
impl AnchorEvidence {
    pub fn kind(&self) -> AnchorEvidenceKind {
        match self.source {
            AnchorEvidenceSource::Genesis(_) => AnchorEvidenceKind::Genesis,
            AnchorEvidenceSource::Regional(_) => AnchorEvidenceKind::Regional,
            AnchorEvidenceSource::Force(_) => AnchorEvidenceKind::Force,
            AnchorEvidenceSource::Resume(_) => AnchorEvidenceKind::Resume,
        }
    }
    pub fn validate_structure(&self) -> Result<(), DomainError> {
        if let AnchorEvidenceSource::Force(ref p) = self.source {
            p.validate()?;
            ensure(p.kind == ReplayKind::Force, "Force evidence position")?;
        }
        Ok(())
    }
}
plain!(RegionalResult {descriptor:Descriptor,block:PhysicalBlock,final_proof:Vec<u8>,chain:Vec<PhysicalBlock>,anchor_evidence:AnchorEvidence});
plain!(VirtualBlock {metadata:Metadata,dep:Vec<Vec<u8>>,pre_state_root:Hash32,transactions:Vec<Vec<u8>>});
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloseReason {
    Time,
    Complete,
    Force,
    Resume,
    Expired,
    Excluded,
    Incompatible,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleBody {
    SessionAssigned {
        descriptor: Descriptor,
    },
    SessionClosed {
        session: SessionId,
        reason: CloseReason,
    },
    BranchClosed {
        branch: u64,
        reason: CloseReason,
    },
    Resumed {
        control_seq: u64,
        branch: u64,
        anchor: Anchor,
        derived_prefix: Vec<VirtualBlock>,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleKind {
    SessionAssigned,
    SessionClosed,
    BranchClosed,
    Resumed,
}
plain!(LifecycleEvent {
    position: ReplayPosition,
    body: LifecycleBody
});
impl LifecycleEvent {
    pub fn kind(&self) -> LifecycleKind {
        match self.body {
            LifecycleBody::SessionAssigned { .. } => LifecycleKind::SessionAssigned,
            LifecycleBody::SessionClosed { .. } => LifecycleKind::SessionClosed,
            LifecycleBody::BranchClosed { .. } => LifecycleKind::BranchClosed,
            LifecycleBody::Resumed { .. } => LifecycleKind::Resumed,
        }
    }
}
