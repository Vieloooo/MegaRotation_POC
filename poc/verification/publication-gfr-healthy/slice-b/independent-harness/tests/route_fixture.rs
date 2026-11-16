//! Actual02 route/mailbox algorithms with explicitly synthetic storage receipts.
#![allow(dead_code)]
use poc_publication_bridge::{BridgeError, DeliveryVerifier, ReassemblyRoute};
use sim_transport::*;
use std::cell::RefCell;

pub const SRC: Endpoint = Endpoint::Service(6);
pub const DST: Endpoint = Endpoint::Reader(1);
pub const ALT: Endpoint = Endpoint::Reader(2);
pub const CHANNEL: ChannelId = ChannelId(7);

pub fn route(payload_limit:usize)->ReassemblyRoute {
    ReassemblyRoute { run:[0x11;32],config:[0x22;32],source:EndpointBinding {endpoint:SRC,incarnation:1},
        destination:EndpointBinding {endpoint:DST,incarnation:1},channel:CHANNEL,context:Some([0x33;32]),
        codec:[0x44;32],payload_limit }
}

#[derive(Default)]
pub struct DeliveryPort(RefCell<Vec<Delivery>>);
impl DeliveryPort { pub fn register(&self,d:&Delivery){ self.0.borrow_mut().push(d.clone()); } }
impl DeliveryVerifier for DeliveryPort {
    fn verify_delivery(&self,d:&Delivery)->Result<(),BridgeError> {
        if self.0.borrow().contains(d){Ok(())}else{Err(BridgeError::InvalidResult)}
    }
}

fn config(limit:usize)->TransportConfig {
    let permit=ChannelPermit {channel:CHANNEL,contexts:vec![[0x33;32],[0x34;32]]};
    let mut limits=Limits::default();limits.payload_bytes=limit as u64;
    TransportConfig {schema:1,run:[0x11;32],config:[0x22;32],seed:7,limits,
        endpoints:[SRC,DST,ALT].into_iter().map(|endpoint|EndpointRegistration {
            binding:EndpointBinding {endpoint,incarnation:1},send:vec![permit.clone()],receive:vec![permit.clone()] }).collect(),
        channels:vec![ChannelSpec {id:CHANNEL,context:ContextPolicy::Required,reliability:Reliability::Reliable}],
        rules:vec![LinkRule {source:EndpointSelector::Any,destination:EndpointSelector::Any,
            channel:ChannelSelector::Exact(CHANNEL),profile:LinkProfile {latency_us:20,..LinkProfile::default()} }] }
}
fn receipt(plan:&CustodyPlan)->DurabilityReceipt {
    // This tests the released02 transition contract; it is not a real H1 root/storage receipt.
    DurabilityReceipt {owner:plan.owner(),base_revision:plan.base_revision(),transition_id:plan.transition_id(),
        candidate_record_hash:plan.candidate_record_hash(),storage_receipt:[0x66;32] }
}

pub struct Routed {
    kernel:RouteKernel,sender:ReliableMailbox,receiver:ReliableMailbox,step:u64,tick:u64,
}
impl Routed {
    pub fn new(limit:usize)->Self {
        let cfg=config(limit);
        Self {kernel:RouteKernel::new(cfg.clone()).unwrap(),
            sender:ReliableMailbox::new(cfg.clone(),SRC,ReliableLimits::default(),20_000).unwrap(),
            receiver:ReliableMailbox::new(cfg,DST,ReliableLimits::default(),20_000).unwrap(),step:0,tick:1_000_000}
    }
    pub fn deliver(&mut self,payload:Vec<u8>,port:&DeliveryPort)->Delivery {
        self.step+=1;self.tick+=1;
        let mut message=Message {run:[0x11;32],config:[0x22;32],source:SRC,destination:DST,channel:CHANNEL,
            context:Some([0x33;32]),codec:[0x44;32],origin:Origin {step_id:self.step,effect_index:0,
                sent_tick_us:self.tick,source_incarnation:1},payload_hash:payload_hash(&payload),logical_id:[0;32],payload};
        message.logical_id=message_id(&message);
        let enqueue=self.sender.plan(MailboxAction::Enqueue {message:message.clone(),now_us:self.tick}).unwrap();
        let rec=receipt(&enqueue);self.sender.confirm(enqueue,rec).unwrap();
        let attempt=self.sender.plan(MailboxAction::Attempt {logical_id:message.logical_id,now_us:self.tick,
            destination:EndpointBinding {endpoint:DST,incarnation:1}}).unwrap();
        let rec=receipt(&attempt);let custody=self.sender.confirm(attempt,rec).unwrap();
        let [CustodyRelease::Send(envelope)]=custody.releases.as_slice() else {panic!("02 send not released")};
        let stamp=StepStamp {logical_step_id:self.step,tick_us:self.tick,event_id:self.step,causal_wave:0};
        let plan=self.kernel.plan(stamp,&[RouteCommand::Send(envelope.clone())]).unwrap();self.kernel.commit(plan).unwrap();
        self.tick=self.kernel.next_wakeup_us().unwrap();self.step+=1;
        let stamp=StepStamp {logical_step_id:self.step,tick_us:self.tick,event_id:self.step,causal_wave:0};
        let plan=self.kernel.plan(stamp,&[RouteCommand::Drain]).unwrap();let events=self.kernel.commit(plan).unwrap();
        let delivery=events.into_iter().find_map(|e|if let RouteEventKind::Delivered(d)=e.kind{Some(d)}else{None}).unwrap();
        let receive=self.receiver.plan(MailboxAction::Receive {envelope:delivery.envelope.clone(),
            received_tick_us:delivery.delivered_tick_us,received_local_us:Some(delivery.delivered_tick_us as i64),
            authenticated_source:EndpointBinding {endpoint:SRC,incarnation:1}}).unwrap();
        let rec=receipt(&receive);let received=self.receiver.confirm(receive,rec).unwrap();
        let ack=received.releases.iter().find_map(|r|if let CustodyRelease::ReceiveAck(a)=r{Some(a.clone())}else{None}).unwrap();
        let acknowledgement=self.sender.plan(MailboxAction::Acknowledge {ack,
            authenticated_receiver:EndpointBinding {endpoint:DST,incarnation:1}}).unwrap();
        let rec=receipt(&acknowledgement);self.sender.confirm(acknowledgement,rec).unwrap();
        port.register(&delivery);delivery
    }
}
