//! Independent low-level recovery crash images. No native engine recovery claim.
use sha2::{Digest,Sha256};
type Id=[u8;32];
#[derive(Debug,PartialEq,Eq)] struct Error(&'static str);
type Result<T>=std::result::Result<T,Error>;
fn hash_parts(domain:&[u8],parts:&[&[u8]])->Id{let mut hash=Sha256::new();for part in std::iter::once(domain).chain(parts.iter().copied()){hash.update((part.len() as u64).to_be_bytes());hash.update(part);}hash.finalize().into()}
pub use poc_native_runtime::{native_wire,native_wire_support};

// Only the configuration projection used by the two storage leaves is supplied.
// No consensus/application/cut validation is implemented by this test substitute.
mod native_config{
 use super::*;
 pub struct RecoverySchedule{pub target_worker:u32,pub boundary:String,pub downtime_us:u64}
 pub struct Base{pub seed:u64,pub recovery:Option<RecoverySchedule>}
 pub struct Resolved{pub run_id:Id,pub config_hash:Id,pub base:Base}
 impl Resolved{
  pub fn generate(_scenario:&str,seed:u64)->Result<(Self,())>{Ok((Self{run_id:[1;32],config_hash:[2;32],base:Base{seed,recovery:None}},()))}
  pub fn binding(&self,node:u32)->native_wire::Binding{native_wire::Binding{schema_version:1,run_id:self.run_id.to_vec(),config_hash:self.config_hash.to_vec(),worker_id:node,member_id:node,incarnation:1}}
 }
}
#[path="../src/native_recovery_ledger.rs"] mod aux;
#[path="../src/native_step_journal.rs"] mod root_step;
#[path="../../../verification/native-recovery/ledger_acceptance.rs"] mod acceptance;
