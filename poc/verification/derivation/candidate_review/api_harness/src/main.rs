//! Independent public-API negative checks against the exact candidate dependency.
//! Input RPC replies are fresh real evidence, replayed through the unchanged 09 Driver.
use l1_adapter::{Action, Command, Config, Query, rpc::{Rpc, RpcError}};
use poc_core::fixture::ValidatedDepFixture;
use poc_derivation::{Checkpoint, Derivation, DerivationConfig, DriverSource, FixtureSource, Outcome, Reject, Step, WaitReason};
use poc_types::domain::{DepFixture, ExecutionAnchor, Hash32, L1Position};
use serde_json::{Value, json};
use std::{cell::RefCell, collections::VecDeque, fs, path::Path, rc::Rc};

#[derive(Clone, Default)]
struct CapturedRpc(Rc<RefCell<VecDeque<Value>>>);
impl CapturedRpc {
    fn feed(&self, value: &Value) {
        assert!(self.0.borrow().is_empty());
        self.0.borrow_mut().extend(value.as_array().unwrap().iter().cloned());
    }
}
impl Rpc for CapturedRpc {
    fn call(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        let row = self.0.borrow_mut().pop_front().expect("unexpected RPC");
        assert_eq!(row["request"]["method"], method);
        assert_eq!(row["request"]["params"], params);
        assert_eq!(row["request"]["id"], row["response"]["id"]);
        if let Some(result) = row["response"].get("result") {
            assert_eq!(row["result"]["Ok"], *result);
            Ok(result.clone())
        } else {
            Err(RpcError::Remote(row["response"]["error"].clone()))
        }
    }
}
fn load(path: &Path) -> Value { serde_json::from_slice(&fs::read(path).unwrap()).unwrap() }
fn hash(text: &str) -> Hash32 {
    let mut out = [0;32];
    for i in 0..32 { out[i] = u8::from_str_radix(&text[2+2*i..4+2*i],16).unwrap(); }
    Hash32::new(out)
}
fn ready<T>(value: Outcome<T>) -> T {
    match value { Outcome::Ready(x) => x, Outcome::Wait(x) => panic!("unexpected WAIT {x:?}"), Outcome::Reject(x) => panic!("unexpected REJECT {x:?}") }
}
struct Inputs {
    config: DerivationConfig,
    genesis: ExecutionAnchor,
    dep: ValidatedDepFixture,
    source: DriverSource<CapturedRpc>,
}
fn source(run: &Path, reader: &str, stop_after_read: Option<u64>, window: u64) -> Inputs {
    let profile = load(&run.join("derivation-profile.json"));
    let original: DerivationConfig = serde_json::from_value(profile["config"].clone()).unwrap();
    let config = DerivationConfig::new(original.public().clone(),window).unwrap();
    let genesis = serde_json::from_value(profile["genesis"].clone()).unwrap();
    let fixture: DepFixture = serde_json::from_value(profile["deposits"].clone()).unwrap();
    let dep = ValidatedDepFixture::load(config.public(),fixture).unwrap();
    let driver: Config = serde_json::from_value(load(&run.join("driver-config.json"))).unwrap();
    let raw = load(&run.join("raw-rpc-bundle.json"));
    let trace = load(&run.join("driver-trace.json"));
    let rows = trace.as_array().unwrap();
    let rpc = CapturedRpc::default();
    rpc.feed(&rows[0]["startup_rpc"]);
    let mut source = DriverSource::new(&config, driver, reader.into(), hash(raw["blocks"][0]["hash"].as_str().unwrap()),rpc.clone()).unwrap();
    assert!(rpc.0.borrow().is_empty());
    for row in &rows[1..] {
        let command: Command = serde_json::from_value(row["command"].clone()).unwrap();
        let stop = matches!(&command.action, Action::Read {reader: r,query: Query::Block{height}} if r == reader && Some(*height) == stop_after_read);
        rpc.feed(&row["rpc"]);
        let reply = source.apply(command).unwrap();
        assert!(rpc.0.borrow().is_empty());
        assert_eq!(serde_json::to_value(reply).unwrap(),row["reply"]);
        if stop { break; }
    }
    Inputs {config,genesis,dep,source}
}
fn init(i: &Inputs) -> Derivation { ready(Derivation::init(i.config.clone(),i.genesis.clone(),i.dep.clone(),&i.source.inputs())) }
fn restore(i: &Inputs, cp: &Checkpoint) -> Outcome<Derivation> { Derivation::restore(cp,i.config.clone(),i.genesis.clone(),i.dep.clone(),&i.source.inputs()) }
fn flip(value: &mut Value) { *value = json!((value.as_u64().unwrap()+1)%256); }

fn main() {
    let arg = std::env::args().nth(1).expect("fresh verifier artifact root");
    let root = Path::new(&arg);
    let ahead = source(&root.join("ahead-300"),"A",None,40);
    let mut state = init(&ahead);
    let mut all_checkpoints = vec![state.checkpoint()];
    let mut first_marker45 = None;
    loop {
        match state.step(&ahead.source.inputs()) {
            Outcome::Ready(step) => {
                if matches!(step,Step::CursorSet{resume_position:L1Position{block:45,tx:0,log:1},..}) { first_marker45=Some(state.checkpoint()); }
                all_checkpoints.push(state.checkpoint());
            },
            Outcome::Wait(_) => break,
            Outcome::Reject(e) => panic!("valid replay rejected {e:?}"),
        }
    }
    assert_eq!(state.head().anchor.block.height,14);
    for cp in &all_checkpoints { assert_eq!(ready(restore(&ahead,cp)).checkpoint(),*cp); }
    let final_cp = state.checkpoint();
    let before = serde_json::to_value(first_marker45.unwrap()).unwrap();
    let mut rejected = Vec::new();
    for name in ["epoch","flag","action_count","window","source_hash","genesis_native","origin_header","last_log","covered_hash","covered_gap","covered_order","cursor_index","cursor_stamp","state","native_parent","native_hash","physical_parent","virtual_pre_root","cause","cut_digest","cut_position"] {
        let mut bad = before.clone();
        match name {
            "epoch" => bad["current_epoch"] = json!(99),
            "flag" => bad["epoch_has_output"] = json!(false),
            "action_count" => bad["action_count"] = json!(u64::MAX),
            "window" => bad["identity"]["sequence_window_blocks"] = json!(41),
            "source_hash" => flip(&mut bad["identity"]["l1"]["l1_genesis_hash"][0]),
            "genesis_native" => flip(&mut bad["genesis"]["native_hash"][0]),
            "origin_header" => flip(&mut bad["genesis_origin"]["hash"][0]),
            "last_log" => bad["last_log"]["tx"] = json!(9),
            "covered_hash" => flip(&mut bad["covered"][2]["reference"]["hash"][0]),
            "covered_gap" => { bad["covered"].as_array_mut().unwrap().remove(1); },
            "covered_order" => bad["covered"].as_array_mut().unwrap().swap(1,2),
            "cursor_index" => bad["cursor"]["next_event"] = json!(2),
            "cursor_stamp" => flip(&mut bad["cursor"]["block"]["read_evidence_digest"][0]),
            "state" => bad["derived"][0]["state"]["accounts"]["1"]["balance"] = json!(999),
            "native_parent" => flip(&mut bad["derived"][0]["native_block"]["parent"][0]),
            "native_hash" => flip(&mut bad["derived"][0]["native_hash"][0]),
            "physical_parent" => flip(&mut bad["derived"][0]["physical_block"]["parent"][0]),
            "virtual_pre_root" => flip(&mut bad["derived"][0]["virtual_block"]["pre_state_root"][0]),
            "cause" => bad["derived"][0]["cause"]["position"]["tx"] = json!(3),
            "cut_digest" => flip(&mut bad["cuts"][0]["evidence_digest"][0]),
            "cut_position" => bad["cuts"][0]["resume_position"]["tx"] = json!(8),
            _ => unreachable!(),
        }
        let bad: Checkpoint = serde_json::from_value(bad).unwrap();
        assert!(matches!(restore(&ahead,&bad),Outcome::Reject(_)),"tamper accepted: {name}");
        rejected.push(name);
    }
    for name in ["anchor","state","native"] {
        let mut genesis = ahead.genesis.clone();
        match name { "anchor"=>genesis.anchor.timestamp_us+=1, "state"=>genesis.state.accounts.get_mut(&1).unwrap().balance+=1, _=>genesis.native_hash=Hash32::new([0;32]) }
        assert!(matches!(Derivation::init(ahead.config.clone(),genesis,ahead.dep.clone(),&ahead.source.inputs()),Outcome::Reject(_)));
    }
    let b = source(&root.join("ahead-300"),"B",None,40);
    assert_eq!(ahead.source.inputs().identity(),b.source.inputs().identity());
    assert_eq!(ready(restore(&b,&final_cp)).checkpoint(),final_cp);
    let partial = source(&root.join("ahead-300"),"A",Some(2),40);
    assert!(matches!(restore(&partial,&final_cp),Outcome::Wait(_)));
    let alternate = source(&root.join("ahead-300"),"A",None,41);
    assert!(matches!(state.step(&alternate.source.inputs()),Outcome::Reject(_)));
    assert_eq!(state.checkpoint(),final_cp);
    for position in [L1Position{block:2,tx:0,log:0},L1Position{block:45,tx:0,log:0},L1Position{block:45,tx:1,log:3}] {
        assert!(matches!(state.derive_at_resume(position),Outcome::Reject(Reject::NonResume(_))));
    }
    assert!(matches!(state.derive_at_resume(L1Position{block:1000,tx:0,log:1}),Outcome::Wait(_)));
    for (position,target,prefix) in [(L1Position{block:45,tx:0,log:1},8,10),(L1Position{block:45,tx:1,log:1},11,11)] {
        let cut=ready(state.derive_at_resume(position));
        assert_eq!(cut.target_height,target);assert_eq!(cut.derived_prefix.len(),prefix);
        let target_state=&cut.derived_prefix[(target-1) as usize].state;
        assert_eq!(cut.target_anchor.state,*target_state);
    }
    let initial = init(&ahead);
    let mut modified = serde_json::to_value(initial.checkpoint()).unwrap();
    modified["epoch_has_output"] = json!(true);
    assert!(matches!(restore(&ahead,&serde_json::from_value(modified).unwrap()),Outcome::Reject(_)));
    let accepted = source(&root.join("accepted_wait-300"),"A",None,40);
    let mut waiting=init(&accepted);let first=waiting.advance(&accepted.source.inputs());
    assert!(matches!(first.status,Outcome::Wait(_)));assert!(first.new_items.is_empty());
    let cp=waiting.checkpoint();let again=waiting.advance(&accepted.source.inputs());
    assert_eq!(waiting.checkpoint(),cp);assert_eq!(first.status,again.status);
    assert_eq!(ready(restore(&accepted,&cp)).checkpoint(),cp);
    assert!(DerivationConfig::new(ahead.config.public().clone(),0).is_err());
    let mut invalid_config=serde_json::to_value(&ahead.config).unwrap();
    invalid_config["profile_version"]=json!(2);
    assert!(serde_json::from_value::<DerivationConfig>(invalid_config).unwrap().validate().is_err());
    let fixture=poc_core::fixture::reg_a_l1_fixture();
    let original=load(&root.join("ahead-300/derivation-profile.json"));
    let mut deposits:DepFixture=serde_json::from_value(original["deposits"].clone()).unwrap();
    deposits.rows[0][0].amount+=1;
    assert!(ValidatedDepFixture::load(ahead.config.public(),deposits).is_err());
    let mut fixture_genesis=ahead.genesis.clone();
    fixture_genesis.anchor.timestamp_us=10_000_000;
    fixture_genesis.anchor.block.origin=3;
    fixture_genesis.state.metadata.timestamp_us=10_000_000;
    fixture_genesis.state.metadata.origin=3;
    let mut fixture_public=ahead.config.public().clone();
    fixture_public.genesis=fixture_genesis.anchor.clone();
    fixture_public.genesis_state_root=fixture_genesis.state.root().unwrap();
    fixture_public.l1_genesis_timestamp_us=0;
    fixture_public.l1_fixture_hash=fixture.digest().unwrap();
    let cfg=DerivationConfig::new(fixture_public,2).unwrap();
    let dep=ValidatedDepFixture::load(cfg.public(),serde_json::from_value(original["deposits"].clone()).unwrap()).unwrap();
    let mut finite=FixtureSource::new(&cfg,fixture.clone()).unwrap();
    assert!(matches!(Derivation::init(cfg.clone(),fixture_genesis.clone(),dep.clone(),&finite.inputs()),Outcome::Wait(_)));
    for row in &fixture.rows[..6] { finite.deliver(&row.header,row.finalized_at_us,row.finalized_at_us,row.header.height!=5).unwrap(); }
    let mut d=ready(Derivation::init(cfg,fixture_genesis,dep,&finite.inputs()));
    let incomplete=d.advance(&finite.inputs());
    assert!(incomplete.new_items.is_empty());
    assert_eq!(d.complete_frontier().unwrap().height,4);
    assert!(matches!(incomplete.status,Outcome::Wait(poc_derivation::Wait{reason:WaitReason::CompleteBlock(5),..})));
    let row=&fixture.rows[5];
    finite.deliver(&row.header,row.finalized_at_us,row.finalized_at_us,true).unwrap();
    let resolved=d.advance(&finite.inputs());
    assert_eq!(resolved.new_items.iter().map(|x|x.state.metadata.origin).collect::<Vec<_>>(),vec![3,4]);
    assert_eq!(d.complete_frontier().unwrap().height,5);
    let mut corrupt=row.header.clone();corrupt.hash=Hash32::new([0;32]);
    assert!(finite.deliver(&corrupt,row.finalized_at_us,row.finalized_at_us,true).is_err());
    println!("{}",json!({"status":"PASS","candidate":"d0afeee9363ef6703f51eecfd742915f83aa27f2","all_action_checkpoints_restored":all_checkpoints.len(),"checkpoint_tamper_rejected":rejected,"genesis_mutations_rejected":3,"different_reader_restore":"PASS","missing_replay_input":"WAIT","alternate_window_source":"REJECT_WITHOUT_CHANGE","query_non_resume_and_global_log":"REJECT","future_query":"WAIT","accepted_repeated_wait_and_restore":"PASS","config_version_zero_window_and_corrupt_dep":"REJECT","bounded_03_fixture_header_only":"WAIT_THEN_EXACT_OUTPUT","bounded_03_fixture_changed_header":"REJECT","limits":["fresh real RPC transcript replay; does not launch a live service","bounded 03 fixture checks use only authentic committed rows 0..5","D checkpoint replay is not BFT same-cid recovery","no feature-11 payload verification"]}));
}
