//! Independent, deliberately non-cryptographic adapter fixture. Expectations
//! precede implementation; this harness does not import Builder test support.
use std::{fs, path::PathBuf, sync::{Arc, atomic::{AtomicUsize, Ordering}}};
use poc_publisher::*;
use sim_espresso::*;

const EMPTY: &str = "50454c470100010000005300000000000000000a000000000000000900000000000000010b0000000000000000000000";
const TWO: &str = "50454c470100010000005301000000000000000a000000000000000a00000000000000010c00000000000000020000000700000002000000abcd0700000000000000";
fn hex(s: &str) -> Vec<u8> {(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
fn msg(bytes: &[u8]) -> LogMessage {LogMessage{codec:31,bytes:bytes.to_vec()}}
fn block(q:u64, theta:u64, messages:Vec<LogMessage>) -> FinalizedBlock {
    FinalizedBlock{proposal:ProposedBlock{q,theta_us:PublicTimeUs(theta),proposed_at_us:TickUs(100+q),live_at_proposal:true,messages},finalized_at_us:TickUs(101+q)}
}
fn reader(id:&[u8]) -> ReaderState {ReaderState::new(b"S".to_vec(),id.to_vec()).unwrap()}
fn bound() -> StoreBinding {StoreBinding{namespace:b"independent-run".to_vec(),configuration:b"fixture-v1".to_vec(),source_id:b"S".to_vec(),reader_id:b"R".to_vec()}}
fn path(name:&str) -> PathBuf {PathBuf::from(std::env::var_os("PE_VERIFY_DIR").expect("evidence directory")).join(name)}

#[derive(Clone)]
struct Fixture {record_calls:Arc<AtomicUsize>}
#[derive(Clone,Copy)]
struct Codec;
#[derive(Default)]
struct Inputs {complete:bool, fault:u8}
type Key=[u8;2];
type ResultObject=[u8;3];
type Pub=Publisher<Fixture,Codec>;
fn fixture() -> Fixture {Fixture{record_calls:Arc::new(AtomicUsize::new(0))}}
fn missing() -> Vec<MissingInput> {vec![MissingInput{code:1,reference:b"anchor/config/ancestry".to_vec()}]}
impl StateCodec<Key,ResultObject> for Codec {
    fn encode_key(&self,k:&Key)->Vec<u8>{k.to_vec()}
    fn decode_key(&self,b:&[u8])->Result<Key,CodecError>{b.try_into().map_err(|_|CodecError::InvalidKey)}
    fn encode_result(&self,r:&ResultObject)->Vec<u8>{r.to_vec()}
    fn decode_result(&self,b:&[u8])->Result<ResultObject,CodecError>{b.try_into().map_err(|_|CodecError::InvalidResult)}
}
impl PublicationVerifier for Fixture {
    type Key=Key; type Result=ResultObject; type PublicInputs=Inputs;
    fn key(&self,r:&ResultObject)->Result<Key,RejectReason>{Ok([r[0],r[1]])}
    fn verify_offer(&self,r:ResultObject,i:&Inputs)->OfferVerdict<Key,ResultObject>{
        let key=[r[0],r[1]];
        if !(1..=2).contains(&r[2]) {return OfferVerdict::Reject(RejectReason{code:7,reference:b"invalid fixture proof variant".to_vec()});}
        if !i.complete {return OfferVerdict::Wait{key,partial_result:r,missing:missing(),verified_deadline:Some(PublicTimeUs(105))};}
        OfferVerdict::Verified{key,complete_result:r,publication_messages:vec![msg(&[b'P',r[0],r[1],r[2]])],validation_basis:b"non-cryptographic-offer-fixture-v1".to_vec(),deadline:PublicTimeUs(105)}
    }
    fn verify_recorded(&self,t:&PublicationTask<Key,ResultObject>,p:&VerifiedPrefix<'_>,i:&Inputs)->RecordVerdict<Key>{
        self.record_calls.fetch_add(1,Ordering::SeqCst);
        let mut publication=None; let mut body=None;
        for b in p.blocks() {
            for (index,m) in b.proposal.messages.iter().enumerate() {
                if m.codec==31 && m.bytes.len()==4 && m.bytes[..3]==[b'P',t.key[0],t.key[1]] && (1..=2).contains(&m.bytes[3]) {
                    publication.get_or_insert(EspressoPosition{q:b.proposal.q,message_index:index as u32});
                }
                if *m==msg(&[b'B',t.key[0],t.key[1]]) {body.get_or_insert(b.proposal.q);}
            }
        }
        let (Some(position),Some(body))=(publication,body) else{return RecordVerdict::NotYetPublic;};
        let mut key=t.key;
        let mut receipt=RecordedAt{publication_position:position,complete_through_q:position.q.max(body),source_id:b"S".to_vec(),verification_basis:b"non-cryptographic-public-fixture-v1".to_vec()};
        match i.fault {
            1=>key[1]^=1,
            2=>receipt.source_id=b"foreign".to_vec(),
            3=>receipt.publication_position.message_index=999,
            4=>receipt.complete_through_q=receipt.publication_position.q.saturating_sub(1),
            5=>receipt.complete_through_q=p.tip().0+1,
            6=>receipt.verification_basis.clear(),
            _=>(),
        }
        RecordVerdict::Recorded{key,receipt}
    }
}
fn complete() -> Inputs {Inputs{complete:true,fault:0}}
fn new_pub(name:&str,f:Fixture)->Pub {Publisher::create(path(name),bound(),f,Codec).unwrap()}
fn reopen(name:&str,f:Fixture)->Pub {Publisher::open(path(name),bound(),f,Codec).unwrap()}
fn full_reader(id:&[u8])->ReaderState {
    let mut r=reader(id);
    for b in [block(0,100,vec![]),block(1,101,vec![msg(&[b'P',9,2,1])]),block(2,102,vec![msg(&[b'B',9,2])])] {
        r.receive_block(b"S",b,TickUs(200)).unwrap();
    }
    r
}

#[test]
fn t01_t06_l03_literal_wire_and_timestamp_oracle(){
    let e=decode_finalized(&hex(EMPTY)).unwrap();
    assert_eq!(e.0,b"S"); assert_eq!(e.1.proposal.q,0); assert!(e.1.proposal.messages.is_empty());
    assert_eq!(e.1.proposal.proposed_at_us,TickUs(9)); assert_eq!(e.1.finalized_at_us,TickUs(11));
    assert_eq!(encode_finalized(&e.0,&e.1).unwrap(),hex(EMPTY));
    let n=decode_finalized(&hex(TWO)).unwrap();
    assert_eq!(n.1.proposal.messages,vec![LogMessage{codec:7,bytes:vec![0xab,0xcd]},LogMessage{codec:7,bytes:vec![]}]);
    assert_eq!(encode_finalized(&n.0,&n.1).unwrap(),hex(TWO));
    let mut mutations=Vec::new();
    for (offset,value) in [(0,0),(4,2),(35,2),(10,0xff)] {let mut x=hex(EMPTY);x[offset]=value;mutations.push(x);}
    // The source byte is not semantically constrained, so remove that valid mutation.
    mutations.pop();
    let mut tail=hex(EMPTY);tail.push(0);mutations.push(tail);
    mutations.push(hex(EMPTY)[..47].to_vec());
    let mut source=hex(EMPTY);source[6..10].copy_from_slice(&257u32.to_le_bytes());mutations.push(source);
    let mut count=hex(EMPTY);count[44..48].copy_from_slice(&1025u32.to_le_bytes());mutations.push(count);
    let mut codec=hex(TWO);codec[48..52].copy_from_slice(&0u32.to_le_bytes());mutations.push(codec);
    let mut size=hex(TWO);size[52..56].copy_from_slice(&65537u32.to_le_bytes());mutations.push(size);
    let mut time=hex(TWO);time[36..44].copy_from_slice(&9u64.to_le_bytes());mutations.push(time);
    for bad in &mutations {assert!(decode_finalized(bad).is_err());}
    let mut producer=SimEspresso::new(b"S".to_vec(),TimestampPolicy::Fixed{theta_us:10000}).unwrap();
    for (tick,expected) in [(10000,TimestampAssessment::WithinBound),(11000,TimestampAssessment::WithinBound),(11001,TimestampAssessment::OutsideBound{deviation_us:1001,bound_us:1000})] {
        assert_eq!(producer.propose(TickUs(tick),true,vec![]).unwrap().assess_timestamp(1000),expected);
    }
    producer.set_policy(TimestampPolicy::Fixed{theta_us:9999});
    assert!(matches!(producer.propose(TickUs(12000),true,vec![]),Err(LogError::Timestamp(TimestampError::Regressed))));
    producer.set_policy(TimestampPolicy::Fixed{theta_us:10000});
    assert_eq!(producer.propose(TickUs(11002),true,vec![]).unwrap().q,3);
    producer.finalize(0,TickUs(11003)).unwrap();
    producer.finalize(0,TickUs(u64::MAX)).unwrap(); // pure replay must not advance time
    producer.finalize(1,TickUs(11004)).unwrap();
    assert_eq!(TimestampPolicy::Offset{offset_us:0,quantum_us:10}.theta(TickUs(109),None).unwrap(),PublicTimeUs(100));
    assert_eq!(TimestampPolicy::Offset{offset_us:-1,quantum_us:1}.theta(TickUs(0),None),Err(TimestampError::ArithmeticRange));
    assert_eq!(TimestampPolicy::Offset{offset_us:1,quantum_us:1}.theta(TickUs(u64::MAX),None),Err(TimestampError::ArithmeticRange));
    let mut nonlive=block(0,0,vec![]);nonlive.proposal.proposed_at_us=TickUs(10000);nonlive.proposal.live_at_proposal=false;nonlive.finalized_at_us=TickUs(100000);
    assert_eq!(nonlive.proposal.assess_timestamp(1000),TimestampAssessment::NotAssumedNonLive);
    println!("T/L literal oracle PASS: 2 exact wires, {} malformed wires, fixed/equal/regressed/nonlive keys",mutations.len());
}

#[test]
fn l02_l05_independent_fragments_recovery_and_no_future_leak(){
    let blocks=[block(0,100,vec![]),block(1,100,vec![msg(b"proof"),msg(b"ref")]),block(2,102,vec![msg(b"body")])];
    let mut a=reader(b"A");let mut b=reader(b"B");
    for row in &blocks {a.receive_block(b"S",row.clone(),TickUs(300)).unwrap();}
    b.receive_block(b"S",blocks[2].clone(),TickUs(210)).unwrap();assert_eq!(b.tip(),None);assert!(b.read(2).is_err());
    b.receive_block(b"S",blocks[0].clone(),TickUs(220)).unwrap();assert_eq!(b.tip().unwrap().0,0);
    b.receive_header(b"S",BlockHeader::from(&blocks[1]),TickUs(230)).unwrap();
    b.receive_message(b"S",1,0,msg(b"proof"),TickUs(231)).unwrap();
    assert_eq!(b.tip().unwrap().0,0);assert!(b.require_watermark(PublicTimeUs(101)).is_err());
    let snapshot=b.encode_snapshot();fs::write(path("reader-pending.snapshot"),&snapshot).unwrap();
    b=ReaderState::decode_snapshot(b"S",b"B",&fs::read(path("reader-pending.snapshot")).unwrap()).unwrap();
    assert_eq!(b.tip().unwrap().0,0);assert!(b.read(2).is_err());
    b.receive_message(b"S",1,1,msg(b"ref"),TickUs(240)).unwrap();
    assert_eq!(b.tip().unwrap().0,2);assert_eq!(b.message_received_at(1,0),Some(TickUs(231)));
    let before=b.encode_snapshot();
    b.receive_message(b"S",1,0,msg(b"proof"),TickUs(999)).unwrap();assert_eq!(before,b.encode_snapshot());
    assert!(b.receive_message(b"S",1,0,msg(b"conflict"),TickUs(999)).is_err());assert_eq!(before,b.encode_snapshot());
    assert!(b.receive_block(b"foreign",blocks[0].clone(),TickUs(999)).is_err());
    for q in 0..3 {assert_eq!(a.read(q).unwrap(),b.read(q).unwrap());}
    let mut c=reader(b"C");c.receive_message(b"S",0,0,msg(b"x"),TickUs(100)).unwrap();
    let c_before=c.encode_snapshot();let row=block(0,100,vec![msg(b"x")]);
    assert!(matches!(c.receive_header(b"S",BlockHeader::from(&row),TickUs(102)),Err(DeliveryError::InvalidSource(SourceError::ReceivedBeforeFinalization{q:0}))));
    assert_eq!(c_before,c.encode_snapshot());
    assert!(ReaderState::decode_snapshot(b"S",b"wrong",&snapshot).is_err());
    println!("L02-L05 PASS: B tips None/0/0/2; actual serialized pending recovery; A/B equal public positions");
}

#[test]
fn p01_p11_unverified_stopped_task_cannot_record_from_receipt(){
    let f=fixture();let calls=f.record_calls.clone();let mut p=new_pub("unverified",f.clone());
    p.offer([9,2,1],&Inputs::default()).unwrap();assert_eq!(p.task(&[9,2]).unwrap().phase(),Phase::Verifying);
    let r=full_reader(b"R");let prefix=r.require_prefix_through(2).unwrap();
    assert!(matches!(p.confirm_recorded(&[9,2],&prefix,&complete()).unwrap(),ConfirmOutcome::Wait{..}));
    assert_eq!(calls.load(Ordering::SeqCst),0);
    p.apply_public_stop(StopDecision{affected_keys:vec![[9,2]],reason:LifecycleReason::SessionClosed,public_basis:b"verified-scope-fixture".to_vec()}).unwrap();
    assert!(matches!(p.confirm_recorded(&[9,2],&prefix,&complete()).unwrap(),ConfirmOutcome::Wait{..}));assert_eq!(calls.load(Ordering::SeqCst),0);
    drop(p);p=reopen("unverified",f.clone());assert_eq!(p.task(&[9,2]).unwrap().phase(),Phase::Stopped);
    p.offer([9,2,2],&complete()).unwrap();assert_eq!(p.task(&[9,2]).unwrap().phase(),Phase::Stopped);
    assert!(p.prepare_attempt(&[9,2],TickUs(300)).is_err());
    assert!(matches!(p.confirm_recorded(&[9,2],&prefix,&complete()).unwrap(),ConfirmOutcome::Recorded{..}));
    assert_eq!(calls.load(Ordering::SeqCst),1);drop(p);p=reopen("unverified",f);
    let task=p.task(&[9,2]).unwrap();assert_eq!(task.phase(),Phase::Recorded);assert!(task.stop_reason.is_some());assert!(task.verified_payload.is_some());assert!(task.attempts.is_empty());
    assert!(p.prepare_attempt(&[9,2],TickUs(301)).is_err());
    println!("P01/P11 PASS: receipt verifier never called before full Offer verification; stop and record survive file reopen");
}

#[test]
fn p02_p03_p08_p09_receipt_guards_and_durable_attempt_identity(){
    let f=fixture();let mut p=new_pub("guards",f.clone());let key=[9,2];
    p.offer([9,2,1],&complete()).unwrap();let original=p.task(&key).unwrap().verified_payload.clone();
    p.offer([9,2,2],&complete()).unwrap();assert_eq!(p.task(&key).unwrap().result,[9,2,1]);assert_eq!(p.task(&key).unwrap().verified_payload,original);
    let generation=p.generation();assert!(p.offer([9,2,0],&complete()).is_err());assert_eq!(p.generation(),generation);
    let intent=p.prepare_attempt(&key,TickUs(50)).unwrap();drop(p);p=reopen("guards",f.clone());
    assert_eq!(p.prepare_attempt(&key,TickUs(60)).unwrap(),intent);
    p.mark_dispatched(&intent.stable_id,b"controlled-7".to_vec()).unwrap();drop(p);p=reopen("guards",f.clone());
    assert_eq!(p.task(&key).unwrap().phase(),Phase::Submitted);
    assert!(p.mark_service_acknowledged(&intent.stable_id,b"bad",&intent.exact_payload,b"ack".to_vec()).is_err());
    p.mark_service_acknowledged(&intent.stable_id,b"controlled-7",&intent.exact_payload,b"ack".to_vec()).unwrap();
    let r=full_reader(b"R");let short=r.require_prefix_through(1).unwrap();
    assert_eq!(p.confirm_recorded(&key,&short,&complete()).unwrap(),ConfirmOutcome::NotYetPublic);assert_eq!(p.task(&key).unwrap().phase(),Phase::Submitted);
    let other=full_reader(b"other");assert!(matches!(p.confirm_recorded(&key,&other.require_prefix_through(2).unwrap(),&complete()),Err(PublisherError::PrefixIdentityMismatch)));
    let prefix=r.require_prefix_through(2).unwrap();let before=p.generation();
    for fault in 1..=6 {assert!(matches!(p.confirm_recorded(&key,&prefix,&Inputs{complete:true,fault}),Err(PublisherError::InvalidReceipt)));assert_eq!(p.generation(),before);assert!(p.task(&key).unwrap().recorded_at.is_none());}
    assert!(matches!(p.confirm_recorded(&key,&prefix,&complete()).unwrap(),ConfirmOutcome::Recorded{..}));
    let receipt=p.task(&key).unwrap().recorded_at.clone().unwrap();assert_eq!(receipt.publication_position.q,1);assert_eq!(receipt.complete_through_q,2);
    assert_eq!(p.confirm_recorded(&key,&prefix,&complete()).unwrap(),ConfirmOutcome::AlreadyRecorded);
    p.apply_public_stop(StopDecision{affected_keys:vec![key],reason:LifecycleReason::BranchClosed,public_basis:b"historical-position".to_vec()}).unwrap();
    p.mark_service_acknowledged(&intent.stable_id,b"controlled-7",&intent.exact_payload,b"ack".to_vec()).unwrap();drop(p);p=reopen("guards",f);
    assert_eq!(p.task(&key).unwrap().phase(),Phase::Recorded);assert_eq!(p.task(&key).unwrap().recorded_at.as_ref(),Some(&receipt));assert!(p.task(&key).unwrap().stop_reason.is_some());
    println!("P02/P03/P08/P09 PASS: same-key variants, 6 forged receipt projections, stable intent/ACK, public q1 complete-through q2");
}

#[test]
fn p07_exact_cutoff_and_pending_effect_stop(){
    let f=fixture();let mut p=new_pub("cutoff",f.clone());let key=[9,2];p.offer([9,2,1],&complete()).unwrap();
    let intent=p.prepare_attempt(&key,TickUs(50)).unwrap();
    assert_eq!(p.apply_public_stop(StopDecision{affected_keys:vec![[8,8]],reason:LifecycleReason::OtherCidAssigned,public_basis:b"unrelated-scope".to_vec()}).unwrap(),0);
    let mut r=reader(b"R");let b0=block(0,104,vec![msg(&[b'P',9,2,1]),msg(&[b'B',9,2])]);r.receive_block(b"S",b0,TickUs(200)).unwrap();
    assert_eq!(p.apply_watermark(&key,&r.require_prefix_through(0).unwrap()).unwrap(),WatermarkOutcome::BelowDeadline);
    let b1=block(1,105,vec![msg(b"tick")]);r.receive_header(b"S",BlockHeader::from(&b1),TickUs(201)).unwrap();assert!(r.require_watermark(PublicTimeUs(105)).is_err());
    assert_eq!(p.prepare_attempt(&key,TickUs(60)).unwrap(),intent);
    r.receive_message(b"S",1,0,msg(b"tick"),TickUs(202)).unwrap();let prefix=r.require_prefix_through(1).unwrap();
    assert_eq!(p.apply_watermark(&key,&prefix).unwrap(),WatermarkOutcome::Applied);assert!(matches!(p.prepare_attempt(&key,TickUs(61)),Err(PublisherError::SubmissionUnavailable(Phase::Stopped))));
    drop(p);p=reopen("cutoff",f);assert!(p.prepare_attempt(&key,TickUs(62)).is_err());
    assert!(matches!(p.confirm_recorded(&key,&prefix,&complete()).unwrap(),ConfirmOutcome::Recorded{..}));assert!(p.task(&key).unwrap().stop_reason.is_some());assert!(p.prepare_attempt(&key,TickUs(63)).is_err());
    println!("P07 PASS: watermark 104 does not stop; incomplete 105 cannot authorize stop; complete 105 cancels prepared replay durably");
}

#[test]
fn p04_p10_external_io_failure_corruption_and_real_reopen(){
    let f=fixture();let mut p=new_pub("external-io",f.clone());let key=[9,2];p.offer([9,2,1],&complete()).unwrap();let main=path("external-io/state.snapshot");let old=fs::read(&main).unwrap();
    assert!(matches!(Publisher::open(path("external-io"),bound(),f.clone(),Codec),Err(PublisherError::Store(StoreError::WriterActive))));
    // A real filesystem obstruction, with no candidate private fault hook.
    fs::create_dir(path("external-io/state.tmp")).unwrap();
    assert!(matches!(p.prepare_attempt(&key,TickUs(7)),Err(PublisherError::Store(StoreError::Io{stage:CommitStage::OpenTemp,..}))));
    assert!(p.task(&key).unwrap().attempts.is_empty());assert_eq!(fs::read(&main).unwrap(),old);drop(p);
    fs::remove_dir(path("external-io/state.tmp")).unwrap();p=reopen("external-io",f.clone());assert!(p.task(&key).unwrap().attempts.is_empty());
    let first=p.prepare_attempt(&key,TickUs(7)).unwrap();drop(p);
    fs::write(path("external-io/state.tmp"),b"uncommitted garbage").unwrap();p=reopen("external-io",f.clone());assert_eq!(p.prepare_attempt(&key,TickUs(8)).unwrap(),first);drop(p);
    let valid=fs::read(&main).unwrap();fs::write(path("valid-before-corruption.snapshot"),&valid).unwrap();
    for field in 0..4 {let mut binding=bound();match field{0=>binding.namespace.push(0),1=>binding.configuration.push(0),2=>binding.source_id.push(0),_=>binding.reader_id.push(0)};assert!(matches!(Publisher::open(path("external-io"),binding,f.clone(),Codec),Err(PublisherError::Store(StoreError::BindingMismatch))));}
    let mut corrupt=valid.clone();*corrupt.last_mut().unwrap()^=1;fs::write(&main,&corrupt).unwrap();fs::write(path("checksum-corrupt.snapshot"),&corrupt).unwrap();
    assert!(matches!(Publisher::open(path("external-io"),bound(),f.clone(),Codec),Err(PublisherError::Store(StoreError::Checksum))));
    fs::remove_file(&main).unwrap();assert!(Publisher::open(path("external-io"),bound(),f.clone(),Codec).is_err());assert!(Publisher::create(path("external-io"),bound(),f.clone(),Codec).is_err());
    fs::write(&main,&valid).unwrap();p=reopen("external-io",f);assert_eq!(p.prepare_attempt(&key,TickUs(9)).unwrap(),first);
    println!("P04/P10 PASS: actual OpenTemp IO failure, old bytes unchanged, writer exclusion, 4 bindings, corrupt/missing main, stale temp ignored");
}
