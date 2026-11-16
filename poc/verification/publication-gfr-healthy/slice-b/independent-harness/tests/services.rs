//! Actual03/06/07 components with synthetic exact proofs and registered owner receipts.
#![allow(dead_code)]
#[path = "../../../slice-a/independent-harness/tests/fixtures/mod.rs"] mod fixtures;
use fixtures::{Basis,ExactProofs,Fixture};
use poc_publication_bridge::{AttemptCustody,BridgeError,BridgeInputs,BridgeResult,BridgeStateCodec,
    ContextVerifier,CustodyBinding,CustodyOperation,CustodyPlan,DurableCommitPort,NativePublicationVerifier,
    ScheduledMessage,StopLedger,StopLedgerBinding,SubmissionVerifier};
use poc_publisher::{ConfirmOutcome,Phase,Publisher,StoreBinding,Submission};
use poc_types::domain::*;
use poc_viewer_gfr::{CompletenessOutcome,FixtureL1Reader,GfrParameters,VerificationProfile,VerifierBinding,public_completeness};
use sim_espresso::{FinalizedLogReader,LogMessage,ReaderState,SimEspresso,TickUs,TimestampPolicy};
use std::{cell::RefCell,collections::BTreeMap,path::PathBuf,sync::atomic::{AtomicU64,Ordering}};

const E:&[u8]=b"B-service-E";const L:&[u8]=b"B-service-L";const R:&[u8]=b"B-publisher";
fn parameters()->GfrParameters {GfrParameters {announcement_us:6_000_000,grace_us:3_000_000,expiry_us:30_000_000}}
fn profile()->VerificationProfile {VerificationProfile::Native {protocol_version:1,verifier_version:1,source_revision:"SYNTHETIC_B_SERVICE_NOT_CRYPTO".into()}}
fn l1(f:&Fixture)->FixtureL1Reader {
    let mut r=FixtureL1Reader::new(L.to_vec(),f.setup.l1().reader()).unwrap();
    for t in &f.tokens {r.deliver_header(t,t.finalized_at_us()+500_000).unwrap();r.deliver_empty_control(t,t.finalized_at_us()+500_000).unwrap();}r
}
struct Context<'a>(&'a Fixture);
impl ContextVerifier for Context<'_> {
    fn verify_context(&self,c:&PublicConfig,r:&RegionalResult)->InputStatus<ExecutionAnchor> {
        if c==self.0.config() && self.0.plan.validate_descriptor(&r.descriptor).is_ok()
            &&r.descriptor.anchor==self.0.genesis().anchor
            &&r.anchor_evidence.source==AnchorEvidenceSource::Genesis(c.reference().unwrap()) {
            InputStatus::Ready(self.0.genesis().clone())
        } else {InputStatus::Reject(InputError::InvalidSource)}
    }
}
struct MissingContext;
impl ContextVerifier for MissingContext {
    fn verify_context(&self,_:&PublicConfig,r:&RegionalResult)->InputStatus<ExecutionAnchor> {
        InputStatus::Wait(InputMissing::Reference(r.descriptor.plan.reference_block.clone()))
    }
}
fn result(f:&Fixture,p:&ExactProofs)->BridgeResult {
    let descriptor=f.descriptor(0,&f.genesis().anchor,&f.genesis().anchor);
    let chain=f.physical_chain(f.genesis(),&[1,3],None);let block=chain.last().unwrap().clone();
    let final_proof=p.register(&descriptor,&block);
    BridgeResult::new(RegionalResult {descriptor,block,final_proof,chain,
        anchor_evidence:AnchorEvidence {source:AnchorEvidenceSource::Genesis(f.config().reference().unwrap()),source_objects:vec![]} }).unwrap()
}
struct OwnDir(PathBuf);
impl OwnDir {fn new()->Self {static N:AtomicU64=AtomicU64::new(0);Self(std::env::temp_dir().join(format!("h3-B-independent-{}-{}",std::process::id(),N.fetch_add(1,Ordering::Relaxed))))}}
impl Drop for OwnDir {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
fn publisher<'a>(dir:&OwnDir,f:&Fixture,p:GfrParameters)->Publisher<NativePublicationVerifier<'a>,BridgeStateCodec> {
    Publisher::create(&dir.0,StoreBinding {namespace:b"B-file-publisher".to_vec(),configuration:b"B-config".to_vec(),source_id:E.to_vec(),reader_id:R.to_vec()},
        NativePublicationVerifier::new(f.config().clone(),profile(),p,E.to_vec()).unwrap(),BridgeStateCodec).unwrap()
}
fn ledger(f:&Fixture)->StopLedger {StopLedger::new(StopLedgerBinding {config:f.config().clone(),expected:profile(),parameters:parameters(),
    espresso_source:E.to_vec(),l1_source:L.to_vec(),publisher_reader:R.to_vec(),owner_reference:b"SYNTHETIC_OWNER".to_vec()}).unwrap()}

#[derive(Default)]struct Receipts(RefCell<BTreeMap<Hash32,Vec<u8>>>);
impl Receipts {fn register(&self,hash:Hash32)->Vec<u8> {
    if let Some(token)=self.0.borrow().get(&hash){return token.clone();}
    let mut token=b"REGISTERED_SYNTHETIC_RECEIPT".to_vec();token.extend_from_slice(&(self.0.borrow().len() as u64).to_be_bytes());
    self.0.borrow_mut().insert(hash,token.clone());token
}}
impl DurableCommitPort for Receipts {fn verify_commit(&self,h:Hash32,r:&[u8])->Result<(),BridgeError> {
    if self.0.borrow().get(&h).is_some_and(|v|v==r){Ok(())}else{Err(BridgeError::InvalidResult)}
}}
#[derive(Default)]struct Manifests(RefCell<Vec<Submission<ResultKey>>>);
impl SubmissionVerifier for Manifests {fn verify_submission(&self,source:&[u8],s:&Submission<ResultKey>)->Result<(),BridgeError> {
    if source==E&&self.0.borrow().contains(s){Ok(())}else{Err(BridgeError::InvalidResult)}
}}
fn pool()->AttemptCustody {AttemptCustody::new(CustodyBinding {run:[0x11;32],config:[0x22;32],source_id:E.to_vec(),owner_reference:b"SYNTHETIC_CUSTODY_OWNER".to_vec()}).unwrap()}
fn confirm(pool:&mut AttemptCustody,plan:CustodyPlan,owner:&Receipts) {
    let token=owner.register(plan.candidate_hash());pool.confirm(plan,&token,owner).unwrap();
}

#[test]
fn watermark_requires_verified_task_exact_parameters_and_source_reader() {
    let f=Fixture::new();let proof=ExactProofs::default();let basis=Basis::default();let l=l1(&f);let context=Context(&f);
    let inputs=BridgeInputs {finality:Some(VerifierBinding {verifier:&proof,basis:&basis,profile:profile()}),context:&context,l1:&l,dep:f.setup.dep()};
    let r=result(&f,&proof);let dir=OwnDir::new();let mut pubr=publisher(&dir,&f,parameters());pubr.offer(r.clone(),&inputs).unwrap();
    let deadline=24_000_000;let mut log=SimEspresso::new(E.to_vec(),TimestampPolicy::Offset {offset_us:0,quantum_us:1}).unwrap();
    let mut reader=ReaderState::new(E.to_vec(),R.to_vec()).unwrap();
    log.propose(TickUs(deadline-1),true,vec![]).unwrap();let first=log.finalize(0,TickUs(deadline)).unwrap().clone();
    reader.receive_block(E,first.clone(),TickUs(deadline+1)).unwrap();let mut stopped=ledger(&f);
    assert!(stopped.plan_watermark(&pubr,&r.key(),&reader.require_prefix_through(0).unwrap()).unwrap().is_none());
    log.propose(TickUs(deadline),true,vec![]).unwrap();let second=log.finalize(1,TickUs(deadline+1)).unwrap().clone();
    reader.receive_block(E,second.clone(),TickUs(deadline+2)).unwrap();
    let owner=Receipts::default();let plan=stopped.plan_watermark(&pubr,&r.key(),&reader.require_prefix_through(1).unwrap()).unwrap().unwrap();
    assert!(stopped.permit_dispatch(&r).unwrap());
    let receipt=owner.register(plan.candidate_hash());stopped.confirm(plan,&receipt,&owner).unwrap();
    assert!(!stopped.permit_dispatch(&r).unwrap());
    assert_eq!(stopped.deadline_watermark(&r).unwrap(),Some((1,sim_espresso::PublicTimeUs(deadline),sim_espresso::PublicTimeUs(deadline))));
    let mut foreign=ReaderState::new(E.to_vec(),b"another-reader".to_vec()).unwrap();
    foreign.receive_block(E,first,TickUs(deadline+1)).unwrap();foreign.receive_block(E,second,TickUs(deadline+2)).unwrap();
    assert!(ledger(&f).plan_watermark(&pubr,&r.key(),&foreign.require_prefix_through(1).unwrap()).is_err());
    let mut foreign_source=ReaderState::new(b"foreign-E".to_vec(),R.to_vec()).unwrap();
    let mut other_log=SimEspresso::new(b"foreign-E".to_vec(),TimestampPolicy::Offset {offset_us:0,quantum_us:1}).unwrap();
    other_log.propose(TickUs(deadline),true,vec![]).unwrap();
    foreign_source.receive_block(b"foreign-E",other_log.finalize(0,TickUs(deadline+1)).unwrap().clone(),TickUs(deadline+2)).unwrap();
    assert!(ledger(&f).plan_watermark(&pubr,&r.key(),&foreign_source.require_prefix_through(0).unwrap()).is_err());
    let mut different=parameters();different.grace_us+=1;let other_dir=OwnDir::new();let mut other=publisher(&other_dir,&f,different);
    other.offer(r.clone(),&inputs).unwrap();
    assert!(ledger(&f).plan_watermark(&other,&r.key(),&reader.require_prefix_through(1).unwrap()).is_err());
    let partial_dir=OwnDir::new();let mut partial_pub=publisher(&partial_dir,&f,parameters());
    let missing_context=MissingContext;
    let unknown_deadline=BridgeInputs {finality:Some(VerifierBinding {verifier:&proof,basis:&basis,profile:profile()}),context:&missing_context,l1:&l,dep:f.setup.dep()};
    partial_pub.offer(r.clone(),&unknown_deadline).unwrap();
    assert!(partial_pub.task(&r.key()).unwrap().verified_deadline.is_none());
    assert!(!matches!(ledger(&f).plan_watermark(&partial_pub,&r.key(),&reader.require_prefix_through(1).unwrap()),Ok(Some(_))),
        "an unverified deadline cannot produce a public-stop plan");
}

#[test]
fn custody_requires_registered_receipts_then_public_proof_and_body_complete_separately() {
    let f=Fixture::new();let proof=ExactProofs::default();let basis=Basis::default();let l=l1(&f);let context=Context(&f);
    let inputs=BridgeInputs {finality:Some(VerifierBinding {verifier:&proof,basis:&basis,profile:profile()}),context:&context,l1:&l,dep:f.setup.dep()};
    let r=result(&f,&proof);let dir=OwnDir::new();let mut pubr=publisher(&dir,&f,parameters());pubr.offer(r.clone(),&inputs).unwrap();
    let submission=pubr.prepare_attempt(&r.key(),TickUs(14_000_000)).unwrap();
    let manifests=Manifests::default();let receipts=Receipts::default();let mut custody=pool();
    assert!(custody.plan_admit(submission.clone(),&manifests).is_err());manifests.0.borrow_mut().push(submission.clone());
    let plan=custody.plan_admit(submission.clone(),&manifests).unwrap();
    assert!(custody.confirm(plan,b"unregistered receipt",&receipts).is_err());
    assert!(custody.plan_propose(Hash32::new([1;32]),0,&[ScheduledMessage {stable_id:submission.stable_id.clone(),ordinal:0,repeat:false}]).is_err());
    let plan=custody.plan_admit(submission.clone(),&manifests).unwrap();confirm(&mut custody,plan,&receipts);
    let duplicate=custody.plan_admit(submission.clone(),&manifests).unwrap();assert!(matches!(duplicate.operation(),CustodyOperation::AlreadyCommitted));
    let mut conflict=submission.clone();let mut changed=conflict.exact_payload.to_vec();changed[0].bytes.push(0);conflict.exact_payload=changed.into();
    manifests.0.borrow_mut().push(conflict.clone());assert!(custody.plan_admit(conflict,&manifests).is_err());
    let mut log=SimEspresso::new(E.to_vec(),TimestampPolicy::Offset {offset_us:0,quantum_us:1}).unwrap();
    let mut reader=ReaderState::new(E.to_vec(),R.to_vec()).unwrap();
    for (q,ordinal) in [(0,0),(1,2),(2,1)] {
        let intent=Hash32::new([q as u8+1;32]);let choice=[ScheduledMessage {stable_id:submission.stable_id.clone(),ordinal,repeat:false}];
        let plan=custody.plan_propose(intent,q,&choice).unwrap();let messages=match plan.operation() {
            CustodyOperation::Propose {intent_id,q:actual,messages}=>{assert_eq!(*intent_id,intent);assert_eq!(*actual,q);messages.clone()},
            _=>panic!("expected proposal intent"),
        };
        assert_eq!(messages,vec![submission.exact_payload[ordinal as usize].clone()]);confirm(&mut custody,plan,&receipts);
        let replay=custody.plan_propose(intent,q,&choice).unwrap();assert!(matches!(replay.operation(),CustodyOperation::AlreadyCommitted));
        let tick=14_000_000+q*10;log.propose(TickUs(tick),true,messages).unwrap();
        let block=log.finalize(q,TickUs(tick+1)).unwrap().clone();let plan=custody.plan_finalize(&block).unwrap();confirm(&mut custody,plan,&receipts);
        let repeated=custody.plan_finalize(&block).unwrap();assert!(matches!(repeated.operation(),CustodyOperation::AlreadyCommitted));
        reader.receive_block(E,block,TickUs(50_000_000+q)).unwrap();
        let prefix=reader.require_prefix_through(q).unwrap();
        let c=public_completeness(&prefix,q,f.config(),&profile(),&r.result().descriptor,r.key().block_hash,inputs.finality.as_ref());
        if q<2 {assert!(matches!(c,CompletenessOutcome::Incomplete));assert!(matches!(pubr.confirm_recorded(&r.key(),&prefix,&inputs).unwrap(),ConfirmOutcome::NotYetPublic));}
        else {let CompletenessOutcome::Complete(c)=c else {panic!("full public evidence missing")};assert_eq!(c.first_complete_q,2);assert_eq!(c.publication_position.q,1);
            assert!(matches!(pubr.confirm_recorded(&r.key(),&prefix,&inputs).unwrap(),ConfirmOutcome::Recorded {..}));}
    }
    assert_eq!(pubr.task(&r.key()).unwrap().phase(),Phase::Recorded);
}

#[test]
fn custody_stale_cross_instance_and_finalization_conflicts_preserve_pending_intents() {
    let f=Fixture::new();let proof=ExactProofs::default();let basis=Basis::default();let l=l1(&f);let context=Context(&f);
    let inputs=BridgeInputs {finality:Some(VerifierBinding {verifier:&proof,basis:&basis,profile:profile()}),context:&context,l1:&l,dep:f.setup.dep()};
    let r=result(&f,&proof);let dir=OwnDir::new();let mut pubr=publisher(&dir,&f,parameters());pubr.offer(r.clone(),&inputs).unwrap();
    let submission=pubr.prepare_attempt(&r.key(),TickUs(14_000_000)).unwrap();let manifests=Manifests::default();manifests.0.borrow_mut().push(submission.clone());
    let receipts=Receipts::default();let mut custody=pool();let one=custody.plan_admit(submission.clone(),&manifests).unwrap();let stale=custody.plan_admit(submission.clone(),&manifests).unwrap();
    let token=receipts.register(one.candidate_hash());custody.confirm(one,&token,&receipts).unwrap();assert!(custody.confirm(stale,&token,&receipts).is_err());
    let foreign=custody.plan_admit(submission.clone(),&manifests).unwrap();let token=receipts.register(foreign.candidate_hash());assert!(pool().confirm(foreign,&token,&receipts).is_err());
    let choice=[ScheduledMessage {stable_id:submission.stable_id.clone(),ordinal:0,repeat:false}];let intent=Hash32::new([1;32]);
    let plan=custody.plan_propose(intent,0,&choice).unwrap();confirm(&mut custody,plan,&receipts);
    assert!(custody.plan_propose(intent,0,&[ScheduledMessage {stable_id:submission.stable_id.clone(),ordinal:1,repeat:false}]).is_err());
    let mut log=SimEspresso::new(E.to_vec(),TimestampPolicy::Offset {offset_us:0,quantum_us:1}).unwrap();
    log.propose(TickUs(14_000_000),true,vec![submission.exact_payload[0].clone()]).unwrap();let block=log.finalize(0,TickUs(14_000_001)).unwrap().clone();
    let mut bad=block.clone();bad.proposal.messages[0].bytes.push(0);assert!(custody.plan_finalize(&bad).is_err());
    let plan=custody.plan_finalize(&block).unwrap();confirm(&mut custody,plan,&receipts);
    assert!(custody.plan_finalize(&bad).is_err());
}
