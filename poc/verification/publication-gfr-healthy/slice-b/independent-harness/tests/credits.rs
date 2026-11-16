//! Local credit invariants with actual02 ClockTracker/route/mailbox and synthetic registered receipts.
mod route_fixture;
use route_fixture::{DeliveryPort,Routed,SRC,DST};
use poc_publication_bridge::{BridgeError,CreditVerifier,CreditWindow};
use poc_types::domain::Hash32;
use sim_transport::*;
use std::{cell::{Cell,RefCell},collections::{BTreeMap,BTreeSet}};

fn grant()->ServiceGrant {
    let mut clock=ClockTracker::new(ClockIdentity {run:[0x11;32],config:[0x22;32],endpoint:SRC,
        incarnation:1,domain:[0x99;32],epsilon_us:0}).unwrap();
    let request_id=clock.request().unwrap();
    clock.install(1_000_001,ClockSample {run:[0x11;32],config:[0x22;32],endpoint:SRC,incarnation:1,
        domain:[0x99;32],rate_profile:framed_hash("SIMT/rate/v1",&[b"fixed-rate-one-v1"]),request_id,
        sequence:1,as_of_tick_us:1_000_001,local_at_as_of_us:1_000_001,uncertainty_us:0,lease_end_tick_us:2_000_000}).unwrap();
    ServiceGrant {target:EndpointBinding {endpoint:SRC,incarnation:1},
        step:StepStamp {logical_step_id:1,tick_us:1_000_001,event_id:1,causal_wave:0},
        clock:clock.view(1_000_001).unwrap(),input:ServiceInput::External {kind:1,bytes:b"SYNTHETIC-GRANT".to_vec()}}
}
fn message(index:u32,destination:Endpoint,n:usize)->Message {
    assert!((84..=61_523).contains(&n));
    let object=vec![0x5a;n-83];let binding=[0x77;32];
    let digest=poc_types::hash_parts("H3/TransportObject/v1",&[&[0],&binding,&object]);
    let mut payload=b"H3TF\0\x01\0".to_vec();payload.extend_from_slice(&binding);payload.extend_from_slice(&digest);
    payload.extend_from_slice(&(object.len() as u32).to_be_bytes());payload.extend_from_slice(&0u16.to_be_bytes());
    payload.extend_from_slice(&1u16.to_be_bytes());payload.extend_from_slice(&(object.len() as u32).to_be_bytes());payload.extend_from_slice(&object);
    assert_eq!(payload.len(),n);
    let mut m=Message {run:[0x11;32],config:[0x22;32],source:SRC,destination,
        channel:ChannelId(7),context:Some([0x33;32]),codec:[0x44;32],
        origin:Origin {step_id:1,effect_index:index,sent_tick_us:1_000_001,source_incarnation:1},
        payload_hash:payload_hash(&payload),logical_id:[0;32],payload};m.logical_id=message_id(&m);m
}
struct Port {available:Cell<bool>,charges:RefCell<BTreeMap<Hash32,Vec<u8>>>,releases:RefCell<BTreeSet<Id>>}
impl Port {
    fn new()->Self {Self {available:Cell::new(true),charges:RefCell::new(BTreeMap::new()),releases:RefCell::new(BTreeSet::new())}}
    fn charge(&self,h:Hash32)->Vec<u8> {let token=b"REGISTERED-SYNTHETIC-CHARGE".to_vec();self.charges.borrow_mut().insert(h,token.clone());token}
}
impl CreditVerifier for Port {
    fn verify_available(&self,_:&ServiceGrant,_:&[Message])->Result<(),BridgeError> {
        if self.available.get(){Ok(())}else{Err(BridgeError::Capacity)}
    }
    fn verify_charge(&self,h:Hash32,r:&[u8])->Result<(),BridgeError> {
        if self.charges.borrow().get(&h).is_some_and(|known|known==r){Ok(())}else{Err(BridgeError::InvalidResult)}
    }
    fn verify_release(&self,id:Id,r:&[u8])->Result<(),BridgeError> {
        if self.releases.borrow().contains(&id)&&r==b"REGISTERED-SYNTHETIC-RELEASE"{Ok(())}else{Err(BridgeError::InvalidResult)}
    }
}

#[test]
fn charge_and_release_require_matching_owner_outcomes() {
    let g=grant();let port=Port::new();let mut credits=CreditWindow::new();let m=message(0,DST,100);
    let plan=credits.plan(&g,vec![m.clone()],&port).unwrap();assert_eq!(plan.effects().len(),1);assert_eq!(credits.in_flight(),(0,0));
    assert!(credits.confirm(plan,b"not a receipt",&port).is_err());assert_eq!(credits.in_flight(),(0,0));
    let plan=credits.plan(&g,vec![m.clone()],&port).unwrap();let token=port.charge(plan.candidate_hash());credits.confirm(plan,&token,&port).unwrap();
    assert_eq!(credits.in_flight(),(1,100));assert!(credits.release(m.logical_id,b"not an ACK",&port).is_err());assert_eq!(credits.in_flight(),(1,100));
    // Register release only after actual02 route and reliable mailbox acknowledgement transitions.
    let delivery_port=DeliveryPort::default();let mut network=Routed::new(65536);let delivered=network.deliver(m.payload.clone(),&delivery_port);
    assert_eq!(delivered.envelope.message.logical_id,m.logical_id);
    port.releases.borrow_mut().insert(m.logical_id);credits.release(m.logical_id,b"REGISTERED-SYNTHETIC-RELEASE",&port).unwrap();
    assert_eq!(credits.in_flight(),(0,0));
}

#[test]
fn limits_and_shared_unavailability_leave_credit_unchanged() {
    let g=grant();let port=Port::new();let credits=CreditWindow::new();
    assert!(credits.plan(&g,(0..5).map(|i|message(i,DST,100)).collect(),&port).is_err());
    assert!(credits.plan(&g,(0..3).map(|i|message(i,Endpoint::Reader(i+1),100)).collect(),&port).is_err());
    let frames:Vec<_>=(0..4).map(|i|message(i,DST,61_523)).collect();
    assert!(credits.plan(&g,frames.clone(),&port).is_ok());
    port.available.set(false);assert!(credits.plan(&g,frames,&port).is_err());assert_eq!(credits.in_flight(),(0,0));
}

#[test]
fn stale_cross_instance_and_missing_ack_do_not_free_existing_budget() {
    let g=grant();let port=Port::new();let mut credits=CreditWindow::new();let frames:Vec<_>=(0..4).map(|i|message(i,DST,61_523)).collect();
    let one=credits.plan(&g,frames.clone(),&port).unwrap();let stale=credits.plan(&g,frames.clone(),&port).unwrap();let receipt=port.charge(one.candidate_hash());
    credits.confirm(one,&receipt,&port).unwrap();assert_eq!(credits.in_flight(),(4,246_092));
    assert!(credits.confirm(stale,&receipt,&port).is_err());
    assert!(credits.plan(&g,vec![message(4,DST,84)],&port).is_err());assert_eq!(credits.in_flight(),(4,246_092));
    let mut foreign=CreditWindow::new();let foreign_plan=foreign.plan(&g,vec![message(0,DST,84)],&port).unwrap();let token=port.charge(foreign_plan.candidate_hash());
    assert!(credits.confirm(foreign_plan,&token,&port).is_err());assert_eq!(foreign.in_flight(),(0,0));assert_eq!(credits.in_flight(),(4,246_092));
}
