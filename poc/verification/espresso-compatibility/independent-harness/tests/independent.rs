use base64::{Engine, engine::general_purpose::STANDARD};
use espresso_compatibility::{CaptureKind, CaptureRecord, Error, FIXTURE_CODEC, FinalityBasis,
    OfficialCaptureReader, PublicTimeUs, ReceiveOutcome, SourceBinding, SourceIdentity};
use serde_json::{Value, json};

fn vectors() -> Value { serde_json::from_str(include_str!("../../vectors-v1.json")).unwrap() }
fn raw(v: &Value) -> Vec<u8> { serde_json::to_vec(v).unwrap() }
fn unhex(s: &str) -> Vec<u8> { s.as_bytes().chunks_exact(2).map(|x| u8::from_str_radix(std::str::from_utf8(x).unwrap(),16).unwrap()).collect() }
struct R { r: OfficialCaptureReader, n: u64, id: String }
impl R {
    fn new(id: &str) -> Self {
        let v=vectors();
        let b=SourceBinding::new(serde_json::from_value(v["source_identity"].clone()).unwrap(),"1".repeat(64),CaptureKind::Fixture).unwrap();
        Self { r:OfficialCaptureReader::new(b,id).unwrap(),n:0,id:id.into() }
    }
    fn bytes(&mut self,method:&str,path:&str,status:Option<u16>,body:&[u8])->Result<ReceiveOutcome,Error>{
        self.n+=1;let sid=self.r.binding().source_id().to_owned();
        self.r.receive(CaptureRecord{source_id:&sid,reader_id:&self.id,id:self.n,method,path,status,response:body})
    }
    fn send(&mut self,path:&str,v:&Value)->Result<ReceiveOutcome,Error>{ self.bytes("GET",path,Some(200),&raw(v)) }
    fn block(&mut self,b:&Value){
        let q=b["q"].as_u64().unwrap();
        // Deliver transactions and leaf before the complete block metadata.
        for tx in b["transaction_responses"].as_array().unwrap().iter().rev(){
            self.send(&format!("/v1/availability/transaction/{q}/{}/noproof",tx["index"]),tx).unwrap();
        }
        self.send(&format!("/v1/availability/leaf/{q}"),&b["leaf_response"]).unwrap();
        self.send(&format!("/v1/availability/block/{q}"),&b["block_response"]).unwrap();
    }
    fn audit(&self,n:usize){
        let v=vectors();let e=&v["expected_audit_prefixes"][n];
        assert_eq!(self.r.binding().source_id(),v["expected_source_id"].as_str().unwrap());
        assert_eq!(self.r.public_audit_bytes(FIXTURE_CODEC).unwrap(),unhex(e["bytes_hex"].as_str().unwrap()));
        assert_eq!(self.r.public_sha256(FIXTURE_CODEC).unwrap(),e["sha256"].as_str().unwrap());
    }
}

#[test]
fn literal_audit_gaps_order_native_vs_compact_and_independent_readers(){
    let v=vectors();let b=&v["blocks"];
    let mut a=R::new("A");let mut c=R::new("B");a.audit(0);
    a.block(&b[2]);a.audit(0);a.block(&b[0]);a.audit(1);
    assert!(matches!(a.r.read(2),Err(Error::Wait{next_q:1,..})));
    c.block(&b[0]);c.block(&b[2]);c.audit(1);
    a.block(&b[1]);a.audit(3);
    let p=a.r.project_publication_messages(1,FIXTURE_CODEC).unwrap();
    assert_eq!(p.len(),3);
    for (j,x) in p.iter().enumerate(){assert_eq!(x.position.message_index,j as u32);assert_eq!(x.source_position.source_transaction_index,(j+1) as u32);}
    assert_eq!(p[0].message.bytes,b"A");assert_eq!(p[1].message.bytes,b"A");assert!(p[2].message.bytes.is_empty());
    assert!(a.r.project_publication_messages(2,FIXTURE_CODEC).unwrap().is_empty());
    assert!(a.r.require_watermark(PublicTimeUs(7_000_000)).is_ok());
    assert!(matches!(a.r.require_watermark(PublicTimeUs(7_000_001)),Err(Error::Wait{next_q:3,..})));
    a.block(&b[3]);a.audit(4);c.audit(1);
    assert_eq!(a.r.require_prefix_through(1).unwrap().blocks().len(),2);
    assert!(matches!(a.r.read(3).unwrap().finality_basis,FinalityBasis::Fixture{..}));
    assert_eq!(serde_json::to_value(a.r.read(3).unwrap().timing).unwrap(),json!({"proposed_at":"Unknown","finalized_at":"Unknown","live_at_proposal":"Unknown"}));
    assert_eq!(a.r.export_pe_log().unwrap_err().missing,["proposed_at","finalized_at","live_at_proposal"]);
    assert_eq!(a.r.export_pe_reader().err().unwrap().missing,["proposed_at","finalized_at","live_at_proposal"]);
    assert!(a.r.project_publication_messages(1,0).is_err());
}

#[test]
fn submission_http_outcomes_never_create_finality(){
    let mut r=R::new("A");
    assert!(matches!(r.bytes("POST","/v1/submit/submit",Some(200),br#""TX~ack""#),Ok(ReceiveOutcome::Submitted{..})));
    for code in [400,401,404,429,500,503]{assert!(matches!(r.bytes("POST","/v1/submit/submit",Some(code),b"rejected"),Ok(ReceiveOutcome::SubmitRejected{status}) if status==code));}
    for code in [None,Some(404),Some(408),Some(429),Some(500),Some(503)]{assert_eq!(r.bytes("GET","/v1/availability/block/0",code,b"missing").unwrap(),ReceiveOutcome::Unavailable);}
    assert!(r.bytes("GET","/v1/availability/block/0",Some(302),b"redirect").is_err());
    assert_eq!(r.bytes("GET","/v1/node/block-height",Some(200),b"999").unwrap(),ReceiveOutcome::HeightHint(999));
    r.audit(0);assert!(r.r.find_publication("TX~ack",b"A").is_empty());
    let v=vectors();for b in v["blocks"].as_array().unwrap(){r.block(b);}
    let tx=&v["blocks"][1]["transaction_responses"][1];
    let positions=r.r.find_publication(tx["hash"].as_str().unwrap(),b"A");
    assert_eq!(positions.iter().map(|p|p.source_transaction_index).collect::<Vec<_>>(),vec![1,2]);
    assert!(r.r.find_publication(tx["hash"].as_str().unwrap(),b"wrong").is_empty());
}

#[test]
fn conflicting_records_atomic_and_original_receipts_preserved(){
    let v=vectors();let mut r=R::new("A");r.block(&v["blocks"][0]);r.block(&v["blocks"][1]);
    let first=r.r.read(1).unwrap().transaction_refs[1].record_id;
    let tx=&v["blocks"][1]["transaction_responses"][1];let path="/v1/availability/transaction/1/1/noproof";
    assert_eq!(r.send(path,tx).unwrap(),ReceiveOutcome::Duplicate);
    assert_eq!(r.r.read(1).unwrap().transaction_refs[1].record_id,first);
    let mut conflict=tx.clone();conflict["transaction"]["payload"]=json!("Qw==");
    assert!(matches!(r.send(path,&conflict),Err(Error::InvalidSource(_))));r.audit(2);
    let pretty=serde_json::to_vec_pretty(tx).unwrap();assert_eq!(r.bytes("GET",path,Some(200),&pretty).unwrap(),ReceiveOutcome::Duplicate);
    for bad in [b"{".as_slice(),br#"{"index":1,"index":2}"#.as_slice()]{assert!(r.bytes("GET",path,Some(200),bad).is_err());r.audit(2);}
    let mut leaf=v["blocks"][2]["leaf_response"].clone();leaf["leaf"]["block_header"]["fields"]["timestamp"]=json!(6);
    assert!(r.send("/v1/availability/leaf/2",&leaf).is_err());r.audit(2);
    r.block(&v["blocks"][2]);r.audit(3);
}

#[test]
fn source_and_field_guards_leave_no_partial_prefix(){
    let v=vectors();let base=v["source_identity"].clone();
    for (key,value) in [("schema",json!("wrong")),("official_commit",json!("0".repeat(40))),("image_amd64_digest",json!("sha256:bad")),("protocol_version",json!("0.4")),("api_version",json!("v2")),("namespace",json!(9)),("run_id",json!("has\nnewline")),("genesis_block_hash",json!("BLOCK~"))]{
        let mut identity=base.clone();identity[key]=value;let identity:SourceIdentity=serde_json::from_value(identity).unwrap();assert!(identity.source_id().is_err(),"{key}");
    }
    let tx=v["blocks"][1]["transaction_responses"][1].clone();
    let mutations=[("index",json!(0)),("block_height",json!(9)),("namespace",json!(9)),("pos_in_namespace",json!(4294967296u64)),("block_hash",json!("BLOCK~wrong")),("hash",json!("bad"))];
    for (field,val) in mutations{let mut r=R::new("A");r.block(&v["blocks"][0]);r.send("/v1/availability/block/1",&v["blocks"][1]["block_response"]).unwrap();let mut bad=tx.clone();bad[field]=val;assert!(r.send("/v1/availability/transaction/1/1/noproof",&bad).is_err(),"{field}");r.audit(1);r.send("/v1/availability/transaction/1/1/noproof",&tx).unwrap();}
    let mut r=R::new("A");let sid=r.r.binding().source_id().to_owned();
    assert!(r.r.receive(CaptureRecord{source_id:"other",reader_id:"A",id:1,method:"GET",path:"/v1/node/block-height",status:Some(200),response:b"1"}).is_err());
    assert!(r.r.receive(CaptureRecord{source_id:&sid,reader_id:"other",id:1,method:"GET",path:"/v1/node/block-height",status:Some(200),response:b"1"}).is_err());r.audit(0);
}

#[test]
fn timestamp_edges_and_pending_headers_cannot_hide_regression(){
    let v=vectors();let mut r=R::new("A");r.block(&v["blocks"][0]);
    let mut b=v["blocks"][1].clone();b["block_response"]["header"]["fields"]["timestamp"]=json!(18446744073710u64);
    assert!(r.send("/v1/availability/block/1",&b["block_response"]).is_err());r.audit(1);
    b["block_response"]["header"]["fields"]["timestamp"]=json!(18446744073709u64);
    b["leaf_response"]["leaf"]["block_header"]=b["block_response"]["header"].clone();r.block(&b);
    assert_eq!(r.r.tip(),Some((1,PublicTimeUs(18446744073709000000))));
    let mut s=R::new("B");s.block(&v["blocks"][2]);let mut one=v["blocks"][1]["block_response"].clone();one["header"]["fields"]["timestamp"]=json!(8);
    assert!(s.send("/v1/availability/block/1",&one).is_err());s.audit(0);
}

#[test]
fn payload_count_projection_and_total_retention_boundaries(){
    let v=vectors();let mut r=R::new("A");let mut tx=v["blocks"][1]["transaction_responses"][0].clone();
    tx["transaction"]["payload"]=json!(STANDARD.encode(vec![7u8;65537]));
    assert!(matches!(r.send("/v1/availability/transaction/1/0/noproof",&tx),Err(Error::Capacity(_))));
    tx["transaction"]["payload"]=json!(STANDARD.encode(vec![7u8;65536]));
    for i in 0..1024u32{tx["index"]=json!(i);tx["pos_in_namespace"]=json!(i);r.send(&format!("/v1/availability/transaction/1/{i}/noproof"),&tx).unwrap();}
    r.send("/v1/availability/transaction/1/1023/noproof",&tx).unwrap();
    tx["index"]=json!(0);tx["block_height"]=json!(2);tx["block_hash"]=json!("BLOCK~fixture-2");tx["transaction"]["payload"]=json!("AA==");
    assert!(matches!(r.send("/v1/availability/transaction/2/0/noproof",&tx),Err(Error::Capacity(_))));r.audit(0);
    drop(r);
    let mut r=R::new("A");let mut block=v["blocks"][1]["block_response"].clone();block["num_transactions"]=json!(1025);
    assert!(matches!(r.send("/v1/availability/block/1",&block),Err(Error::Capacity(_))));block["num_transactions"]=json!(1024);r.send("/v1/availability/block/1",&block).unwrap();
    let mut r=R::new("C");let mut tx=v["blocks"][1]["transaction_responses"][1].clone();
    for i in 0..16 {tx["index"]=json!(i);tx["pos_in_namespace"]=json!(i);tx["transaction"]["payload"]=json!(STANDARD.encode(vec![8u8;if i==15{65293}else{65536}]));
        let p=format!("/v1/availability/transaction/1/{i}/noproof");if i==15{assert!(matches!(r.send(&p,&tx),Err(Error::Capacity(_))));tx["transaction"]["payload"]=json!(STANDARD.encode(vec![8u8;65292]));}r.send(&p,&tx).unwrap();}
    r.audit(0);
}

#[test]
fn pending_height_and_raw_response_limits_equal_and_one_above(){
    let v=vectors();let mut r=R::new("A");let mut leaf=v["blocks"][0]["leaf_response"].clone();
    for q in 1..=4096 {leaf["leaf"]["block_header"]["fields"]["height"]=json!(q);r.send(&format!("/v1/availability/leaf/{q}"),&leaf).unwrap();}
    r.send("/v1/availability/leaf/4096",&leaf).unwrap();leaf["leaf"]["block_header"]["fields"]["height"]=json!(4097);
    assert!(matches!(r.send("/v1/availability/leaf/4097",&leaf),Err(Error::Capacity(_))));r.audit(0);
    let mut body=b"0".to_vec();body.resize(16*1024*1024,b' ');
    assert_eq!(r.bytes("GET","/v1/node/block-height",Some(200),&body).unwrap(),ReceiveOutcome::HeightHint(0));body.push(b' ');
    assert!(matches!(r.bytes("GET","/v1/node/block-height",Some(200),&body),Err(Error::Capacity(_))));
}
