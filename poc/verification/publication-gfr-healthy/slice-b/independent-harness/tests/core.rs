//! SOURCE-DERIVED PREPARATION ONLY UNTIL NAMED B CANDIDATE.
//! Synthetic Native finality and registered owner receipts are NOT H1 root evidence.
#![allow(dead_code)]
#[path = "../../../slice-a/independent-harness/tests/fixtures/mod.rs"]
mod fixtures;

use fixtures::{Basis, ExactProofs, Fixture};
use poc_core::metadata::RegionalMetadata;
use poc_publication_bridge::{BridgeError, BridgeResult, DurableCommitPort, RecordDelivery,
    StopLedger, StopLedgerBinding, StopPlan, fragment};
use poc_types::{domain::*, hash_parts};
use poc_viewer_gfr::{FixtureL1Reader, GfrParameters, GlobalSource, InitInput, InputRef,
    PublicObject, ReplayEffect, ReplayPosition, ReplayReads, ReplayRecord, ReplayServices,
    ReplayState, ReplayStatus, Tail, VerificationProfile, VerifierBinding, PUBLICATION_CODEC,
    advance_global, encode_position, encode_publication, init, lifecycle_id, lifecycle_projection};
use sim_espresso::{FinalizedLogReader, LogMessage, ReaderState, SimEspresso, TickUs, TimestampPolicy};
use std::{cell::RefCell, collections::BTreeMap};

const E: &[u8] = b"h3-B-source-E";
const L: &[u8] = b"h3-B-source-L";
const REV: &str = "SYNTHETIC_NATIVE_B_NOT_CRYPTO";

fn params() -> GfrParameters { GfrParameters { announcement_us: 6_000_000, grace_us: 3_000_000, expiry_us: 30_000_000 } }
fn profile() -> VerificationProfile { VerificationProfile::Native { protocol_version: 1, verifier_version: 1, source_revision: REV.into() } }
fn binding(f: &Fixture, p: GfrParameters) -> StopLedgerBinding {
    StopLedgerBinding { config: f.config().clone(), expected: profile(), parameters: p,
        espresso_source: E.to_vec(), l1_source: L.to_vec(), publisher_reader: b"B-publisher".to_vec(),
        owner_reference: b"SYNTHETIC_REGISTERED_OWNER_NOT_H1".to_vec() }
}

/// Only explicitly registered candidate hashes receive a test receipt.
#[derive(Default)]
struct Owner(RefCell<BTreeMap<Hash32, Vec<u8>>>);
impl Owner {
    fn register(&self, plan: &StopPlan) -> Vec<u8> {
        let expected = Hash32::try_from(hash_parts("H3/StopSnapshot/v1", &[plan.candidate_bytes()]).as_slice()).unwrap();
        assert_eq!(plan.candidate_hash(), expected);
        if let Some(token)=self.0.borrow().get(&expected) {return token.clone();}
        let mut token = b"SYNTHETIC-OWNER-RECEIPT:".to_vec();
        token.extend_from_slice(&(self.0.borrow().len() as u64).to_be_bytes());
        self.0.borrow_mut().insert(expected, token.clone()); token
    }
}
impl DurableCommitPort for Owner {
    fn verify_commit(&self, hash: Hash32, receipt: &[u8]) -> Result<(), BridgeError> {
        if self.0.borrow().get(&hash).is_some_and(|known| known == receipt) { Ok(()) }
        else { Err(BridgeError::InvalidResult) }
    }
}

fn result(f: &Fixture, proofs: &ExactProofs, round: u64, height: u64) -> BridgeResult {
    let descriptor = f.descriptor(round, &f.genesis().anchor, &f.genesis().anchor);
    let chain = f.physical_chain(f.genesis(), &[height], None); let block = chain[0].clone();
    let proof = proofs.register(&descriptor, &block);
    BridgeResult::new(RegionalResult { descriptor, block, chain, final_proof: proof,
        anchor_evidence: AnchorEvidence { source: AnchorEvidenceSource::Genesis(f.config().reference().unwrap()), source_objects: vec![] } }).unwrap()
}

fn replay(f: &Fixture, p: GfrParameters, lists: Vec<(u64, Vec<LogMessage>)>, through: ReplayPosition,
          proofs: &ExactProofs) -> ReplayState {
    replay_bound(f,p,lists,through,proofs,E,L,profile())
}
fn replay_bound(f: &Fixture, p:GfrParameters,lists:Vec<(u64,Vec<LogMessage>)>,through:ReplayPosition,
                proofs:&ExactProofs,e_source:&[u8],l_source:&[u8],verification:VerificationProfile)->ReplayState {
    let mut l1 = FixtureL1Reader::new(l_source.to_vec(), f.setup.l1().reader()).unwrap();
    for t in &f.tokens { l1.deliver_header(t, t.finalized_at_us()+500_000).unwrap();
        l1.deliver_empty_control(t, t.finalized_at_us()+500_000).unwrap(); }
    let mut log = SimEspresso::new(e_source.to_vec(), TimestampPolicy::Offset { offset_us: 0, quantum_us: 1 }).unwrap();
    let mut reader = ReaderState::new(e_source.to_vec(), b"B-replay-reader".to_vec()).unwrap();
    for (q,(key,messages)) in lists.into_iter().enumerate() {
        log.propose(TickUs(key),true,messages).unwrap();
        let b=log.finalize(q as u64,TickUs(key+1)).unwrap().clone();
        reader.receive_block(e_source,b,TickUs(100_000_000+q as u64)).unwrap();
    }
    let basis=Basis::default(); let metadata=RegionalMetadata::new(f.config(),&l1).unwrap();
    let services=ReplayServices { config:f.config(),plan:&f.plan,metadata:&metadata,dep:f.setup.dep(),
        finality:Some(VerifierBinding { verifier:proofs,basis:&basis,profile:verification.clone() }) };
    let reads=ReplayReads { espresso:&reader,l1:&l1 };
    let initial=init(&InitInput::Genesis { config:f.config().clone(),genesis:f.genesis().clone(),
        parameters:p,expected_verification:verification.clone() },&reads,&services).unwrap().state.unwrap();
    let output=advance_global(&initial,&reads,&services,Some(&through));
    assert!(matches!(output.status,ReplayStatus::Ready),"{:?}",output.status);output.state
}
fn ep(key:u64,q:u64)->ReplayPosition { ReplayPosition { key_us:key,kind:ReplayKind::Espresso,index:ReplayIndex::Espresso(q) } }
fn empty(f:&Fixture,p:GfrParameters)->ReplayState { replay(f,p,vec![(14_000_000,vec![])],ep(14_000_000,0),&ExactProofs::default()) }
fn closed(f:&Fixture,p:GfrParameters)->ReplayState {
    let end=21_000_000+p.grace_us;
    replay(f,p,vec![(14_000_000,vec![]),(end,vec![])],ReplayPosition { key_us:end,
        kind:ReplayKind::SessionEnd,index:ReplayIndex::Session(SessionId { beta:0,r:0 }) },&ExactProofs::default())
}

fn frame(bytes:&[u8])->Vec<u8> { let mut out=(bytes.len() as u32).to_be_bytes().to_vec();out.extend_from_slice(bytes);out }
fn tail(value:&Tail)->Vec<u8> {
    match value {
        Tail::Next(d)=>[vec![0],frame(&d.encode_checked().unwrap())].concat(),
        Tail::Pending(b)=>[vec![1],frame(&b.encode_checked().unwrap())].concat(),
        Tail::Halt { boundary,anchor }=>[vec![2],frame(&boundary.encode_checked().unwrap()),frame(&anchor.encode_checked().unwrap())].concat(),
    }
}
/// Independent implementation of the frozen private record projection, using existing03/07 nested codecs.
fn record_projection(r:&ReplayRecord)->Vec<u8> {
    let input=match &r.input_ref {
        InputRef::Espresso(q)=>[vec![0],q.to_be_bytes().to_vec()].concat(),
        InputRef::SessionEnd(s)=>[vec![1],frame(&s.encode_checked().unwrap())].concat(),
        InputRef::Expire(h)=>[vec![2],h.to_be_bytes().to_vec()].concat(),
        _=>panic!("outside regular fixture"),
    };
    let mut out=[vec![0,1],frame(&input),(r.effects.len() as u32).to_be_bytes().to_vec()].concat();
    for e in &r.effects {
        let bytes=match e {
            ReplayEffect::Append {first,last,source:GlobalSource::Regional(key)}=>
                [vec![0],first.to_be_bytes().to_vec(),last.to_be_bytes().to_vec(),vec![0],frame(&key.encode())].concat(),
            ReplayEffect::Duty {before,after}=>[vec![1],frame(&tail(before)),frame(&tail(after))].concat(),
            _=>panic!("outside regular fixture"),
        };out.extend(frame(&bytes));
    }
    out.extend_from_slice(&(r.events.len() as u32).to_be_bytes());
    for e in &r.events {out.extend(frame(&lifecycle_projection(e).unwrap()));}out
}
fn expected_record_id(f:&Fixture,r:&ReplayRecord)->Hash32 {
    Hash32::try_from(hash_parts("H3/Record/v1",&[&f.config().reference().unwrap().encode(),E,L,
        &1u16.to_be_bytes(),&1u16.to_be_bytes(),REV.as_bytes(),&encode_position(&r.position).unwrap(),&record_projection(r)]).as_slice()).unwrap()
}
fn commit(ledger:&mut StopLedger,record:&RecordDelivery,owner:&Owner)->(Vec<u8>,Vec<u8>) {
    let plan=ledger.plan_record(record).unwrap();let bytes=plan.candidate_bytes().to_vec();let receipt=owner.register(&plan);
    ledger.confirm(plan,&receipt,owner).unwrap();(bytes,receipt)
}

#[test]
fn complete_record_identity_keeps_empty_and_same_pc_lifecycle_members() {
    let f=Fixture::new();let state=closed(&f,params());assert_eq!(state.public_state().records.len(),2);
    for ordinal in 0..2 {
        let record=RecordDelivery::from_replay(&state,ordinal).unwrap();
        assert_eq!(record.parameters(),params());assert_eq!(record.id(),expected_record_id(&f,record.record()));
        let expected:Vec<_>=record.record().events.iter().map(|e|lifecycle_id(e).unwrap()).collect();
        assert_eq!(record.event_ids(),expected.as_slice());
    }
    let close=RecordDelivery::from_replay(&state,1).unwrap();assert_eq!(close.event_ids().len(),2);
    assert_ne!(close.event_ids()[0],close.event_ids()[1]);
    assert!(RecordDelivery::from_replay(&state,2).is_err());
}

#[test]
fn close_before_offer_and_failed_receipt_do_not_create_or_erase_scope() {
    let f=Fixture::new();let state=closed(&f,params());let p=ExactProofs::default();let old=result(&f,&p,0,3);let next=result(&f,&p,1,11);
    let mut ledger=StopLedger::new(binding(&f,params())).unwrap();let owner=Owner::default();
    let empty=RecordDelivery::from_replay(&state,0).unwrap();commit(&mut ledger,&empty,&owner);
    let close=RecordDelivery::from_replay(&state,1).unwrap();
    let plan=ledger.plan_record(&close).unwrap();
    assert!(ledger.confirm(plan,b"unregistered receipt",&owner).is_err());
    assert!(ledger.public_stop(&old).unwrap().is_none());
    let (snapshot,receipt)=commit(&mut ledger,&close,&owner);
    assert!(!ledger.permit_dispatch(&old).unwrap());
    assert!(ledger.permit_dispatch(&next).unwrap());
    assert_eq!(ledger.public_stop(&old).unwrap().unwrap().affected_keys,vec![old.key()]);
    let restored=StopLedger::restore(binding(&f,params()),&snapshot,&receipt,&owner).unwrap();
    assert!(!restored.permit_dispatch(&old).unwrap());
    assert!(StopLedger::restore(binding(&f,params()),&snapshot,b"unregistered receipt",&owner).is_err());
    let mut corrupt=snapshot.clone();let n=corrupt.len();corrupt[n-1]^=1;
    assert!(StopLedger::restore(binding(&f,params()),&corrupt,&receipt,&owner).is_err());
    commit(&mut ledger,&close,&owner);assert!(!ledger.permit_dispatch(&old).unwrap());
}

#[test]
fn stale_and_cross_instance_plans_reject_without_rollback() {
    let f=Fixture::new();let state=closed(&f,params());let owner=Owner::default();
    let empty=RecordDelivery::from_replay(&state,0).unwrap();let close=RecordDelivery::from_replay(&state,1).unwrap();
    let mut ledger=StopLedger::new(binding(&f,params())).unwrap();commit(&mut ledger,&empty,&owner);
    let first=ledger.plan_record(&close).unwrap();let stale=ledger.plan_record(&close).unwrap();let receipt=owner.register(&first);
    ledger.confirm(first,&receipt,&owner).unwrap();assert!(ledger.confirm(stale,&receipt,&owner).is_err());
    let foreign=ledger.plan_record(&close).unwrap();let receipt=owner.register(&foreign);
    let mut other=StopLedger::new(binding(&f,params())).unwrap();assert!(other.confirm(foreign,&receipt,&owner).is_err());
    let old=result(&f,&ExactProofs::default(),0,3);assert!(!ledger.permit_dispatch(&old).unwrap());
    assert!(other.public_stop(&old).unwrap().is_none());
}

#[test]
fn exact_parameters_checked_before_duplicate_identity_and_restore() {
    let f=Fixture::new();let normal=empty(&f,params());let record=RecordDelivery::from_replay(&normal,0).unwrap();
    let owner=Owner::default();let mut ledger=StopLedger::new(binding(&f,params())).unwrap();let (snapshot,receipt)=commit(&mut ledger,&record,&owner);
    for field in 0..3 {
        let mut p=params();match field {0=>p.announcement_us+=1,1=>p.grace_us+=1,_=>p.expiry_us+=1};
        let changed=empty(&f,p);let other=RecordDelivery::from_replay(&changed,0).unwrap();
        assert_eq!(record.id(),other.id(),"parameters intentionally not in the8-part record hash");
        assert!(ledger.plan_record(&other).is_err());
        assert!(StopLedger::restore(binding(&f,p),&snapshot,&receipt,&owner).is_err());
    }
}

#[test]
fn record_source_profile_and_config_are_not_interchangeable() {
    let f=Fixture::new();let proofs=ExactProofs::default();let normal=empty(&f,params());
    let record=RecordDelivery::from_replay(&normal,0).unwrap();
    let ledger=StopLedger::new(binding(&f,params())).unwrap();
    let altered=VerificationProfile::Native {protocol_version:1,verifier_version:1,source_revision:"ANOTHER_SYNTHETIC_SOURCE".into()};
    for (es,ls,p) in [(b"foreign-E".as_slice(),L,profile()),(E,b"foreign-L".as_slice(),profile()),(E,L,altered)] {
        let state=replay_bound(&f,params(),vec![(14_000_000,vec![])],ep(14_000_000,0),&proofs,es,ls,p);
        let foreign=RecordDelivery::from_replay(&state,0).unwrap();assert!(ledger.plan_record(&foreign).is_err());
    }
    let mut other=binding(&f,params());other.config.chain_id+=1;
    let foreign=StopLedger::new(other).unwrap();assert!(foreign.plan_record(&record).is_err());
}

#[test]
fn forged_same_key_scope_rejects_before_touching_legitimate_open_task() {
    let f=Fixture::new();let state=closed(&f,params());let owner=Owner::default();let proofs=ExactProofs::default();
    let mut ledger=StopLedger::new(binding(&f,params())).unwrap();
    for i in 0..state.public_state().records.len() {commit(&mut ledger,&RecordDelivery::from_replay(&state,i).unwrap(),&owner);}
    let legitimate=result(&f,&proofs,1,11);assert!(ledger.permit_dispatch(&legitimate).unwrap());
    for mutation in 0..4 {
        let mut forged=legitimate.result().clone();
        match mutation {
            0=>forged.descriptor.plan.session=SessionId {beta:0,r:0},
            1=>forged.descriptor.plan.window=Window {h_min:21,h_max:30},
            2=>forged.descriptor.anchor.block.block_id=Hash32::new([0x55;32]),
            _=>forged.descriptor.plan.config_ref=ConfigRef::new(Hash32::new([0x66;32])),
        }
        let forged=BridgeResult::new(forged).unwrap();assert_eq!(forged.key(),legitimate.key());
        assert!(ledger.public_stop(&forged).is_err());assert!(ledger.permit_dispatch(&forged).is_err());
        assert!(ledger.public_stop(&legitimate).unwrap().is_none());assert!(ledger.permit_dispatch(&legitimate).unwrap());
    }
}

#[test]
fn backward_record_cannot_reopen_closed_scope() {
    let f=Fixture::new();let state=closed(&f,params());let owner=Owner::default();let old=result(&f,&ExactProofs::default(),0,3);
    let early=RecordDelivery::from_replay(&state,0).unwrap();let late=RecordDelivery::from_replay(&state,1).unwrap();
    let mut ledger=StopLedger::new(binding(&f,params())).unwrap();
    // Either reject an out-of-order input or accept its already verified monotonic fact.
    // Neither interpretation permits a later old record to undo an installed exclusion.
    if let Ok(plan)=ledger.plan_record(&late) {
        let receipt=owner.register(&plan);ledger.confirm(plan,&receipt,&owner).unwrap();
        assert!(!ledger.permit_dispatch(&old).unwrap());
        if let Ok(plan)=ledger.plan_record(&early) {let receipt=owner.register(&plan);ledger.confirm(plan,&receipt,&owner).unwrap();}
        assert!(!ledger.permit_dispatch(&old).unwrap());
    } else {assert!(ledger.public_stop(&old).unwrap().is_none());}
}

#[test]
fn same_position_different_sealed_record_conflicts() {
    let f=Fixture::new();let proofs=ExactProofs::default();let target=result(&f,&proofs,0,10);
    let msgs=vec![LogMessage {codec:PUBLICATION_CODEC,bytes:encode_publication(&PublicObject::Body(target.result().block.clone())).unwrap()},
        LogMessage {codec:PUBLICATION_CODEC,bytes:encode_publication(&PublicObject::FinalProof {descriptor:target.result().descriptor.clone(),target:target.key().block_hash,proof:target.result().final_proof.clone()}).unwrap()}];
    let no_result=replay(&f,params(),vec![(23_000_000,vec![])],ep(23_000_000,0),&proofs);
    let with_result=replay(&f,params(),vec![(23_000_000,msgs)],ep(23_000_000,0),&proofs);
    let one=RecordDelivery::from_replay(&no_result,0).unwrap();let two=RecordDelivery::from_replay(&with_result,0).unwrap();
    assert_eq!(one.record().position,two.record().position);assert_ne!(one.id(),two.id());
    let owner=Owner::default();let mut ledger=StopLedger::new(binding(&f,params())).unwrap();commit(&mut ledger,&one,&owner);
    assert!(ledger.plan_record(&two).is_err());
}

#[test]
fn exact_fragment_literal_and_capacity_boundaries() {
    let v:serde_json::Value=serde_json::from_str(include_str!("../../wire-vectors-v1.json")).unwrap();
    let actual=fragment(0,Hash32::new([0x11;32]),b"abc",86).unwrap();
    let expected=v["frame_vector"]["frame_hex"].as_str().unwrap();
    assert_eq!(actual[0].iter().map(|b|format!("{b:02x}")).collect::<String>(),expected);
    for (n,count,last) in [(61_440,1,61_440),(61_441,2,1),(1_048_576,18,4096)] {
        let lengths=if n==1_048_576 {let mut v=vec![65536;15];v.push(65360);v} else {vec![n-56]};
        let messages=lengths.into_iter().map(|len|LogMessage {codec:1,bytes:vec![0x5a;len]}).collect();
        let mut log=SimEspresso::new(b"E".to_vec(),TimestampPolicy::Offset {offset_us:0,quantum_us:1}).unwrap();
        log.propose(TickUs(1),true,messages).unwrap();
        let wire=sim_espresso::encode_finalized(b"E",log.finalize(0,TickUs(2)).unwrap()).unwrap();assert_eq!(wire.len(),n);
        let frames=fragment(1,Hash32::new([0x22;32]),&wire,65_536).unwrap();
        assert_eq!(frames.len(),count);assert_eq!(frames.last().unwrap().len(),83+last);
        assert!(frames.iter().all(|f|f.len()<=65_536));
    }
    assert!(fragment(0,Hash32::new([1;32]),b"",65_536).is_err());
    assert!(fragment(0,Hash32::new([1;32]),b"a",83).is_err());
    assert_eq!(fragment(0,Hash32::new([1;32]),&vec![1;1024],84).unwrap().len(),1024);
    assert!(fragment(0,Hash32::new([1;32]),&vec![1;1025],84).is_err());
    assert!(fragment(0,Hash32::new([1;32]),&vec![1;1_048_577],65_536).is_err());
    assert!(fragment(2,Hash32::new([1;32]),b"a",65_536).is_err());
}
