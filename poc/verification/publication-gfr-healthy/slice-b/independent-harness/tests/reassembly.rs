//! Actual02 RouteKernel + ReliableMailbox fixture; H1 durability remains synthetic/unverified.
mod route_fixture;
use route_fixture::{DeliveryPort,Routed,route};
use poc_publication_bridge::{Reassembler,fragment};
use sim_espresso::{FinalizedLogReader,LogMessage,ReaderState,SimEspresso,TickUs,TimestampPolicy,
    decode_finalized,encode_finalized};

fn wire(q:u64,value:u8)->Vec<u8> {
    let mut log=SimEspresso::new(b"E".to_vec(),TimestampPolicy::Offset {offset_us:0,quantum_us:1}).unwrap();
    for i in 0..=q {
        log.propose(TickUs(1000+i*1000),true,vec![LogMessage {codec:1,bytes:vec![value;80]}]).unwrap();
        log.finalize(i,TickUs(1001+i*1000)).unwrap();
    }
    encode_finalized(b"E",log.finalized(q).unwrap()).unwrap()
}

#[test]
fn reverse_and_duplicate_actual_deliveries_complete_once() {
    let owner=DeliveryPort::default();let r=route(140);let mut receiver=Reassembler::new(vec![route(140)],&owner).unwrap();
    let binding=receiver.register_finalized(&r,b"E",0).unwrap();let encoded=wire(0,7);
    let frames=fragment(1,binding,&encoded,140).unwrap();assert!(frames.len()>1);
    let mut network=Routed::new(140);let mut completed=Vec::new();
    for frame in frames.iter().rev() {
        let delivery=network.deliver(frame.clone(),&owner);
        if let Some(object)=receiver.receive(&delivery).unwrap(){completed.push(object.bytes);}
        // A new actual routed message can repeat the same object chunk.
        let duplicate=network.deliver(frame.clone(),&owner);
        assert!(receiver.receive(&duplicate).unwrap().is_none());
    }
    assert_eq!(completed,vec![encoded.clone()]);
    for frame in &frames {
        let delivery=network.deliver(frame.clone(),&owner);assert!(receiver.receive(&delivery).unwrap().is_none());
    }
    let (source,block)=decode_finalized(&encoded).unwrap();assert_eq!(source,b"E");assert_eq!(block.proposal.q,0);
}

#[test]
fn unregistered_delivery_and_same_binding_changed_digest_reject() {
    let owner=DeliveryPort::default();let r=route(140);let mut receiver=Reassembler::new(vec![route(140)],&owner).unwrap();
    let binding=receiver.register_finalized(&r,b"E",0).unwrap();
    let first=fragment(1,binding,&wire(0,1),140).unwrap();let conflicting=fragment(1,binding,&wire(0,2),140).unwrap();
    let mut network=Routed::new(140);let delivery=network.deliver(first[0].clone(),&owner);
    let mut forged=delivery.clone();forged.envelope.message.run[0]^=1;
    assert!(receiver.receive(&forged).is_err(),"public struct mutation is not authenticated delivery");
    assert!(receiver.receive(&delivery).unwrap().is_none());
    let conflict=network.deliver(conflicting[0].clone(),&owner);assert!(receiver.receive(&conflict).is_err());
    let mut result=None;
    for frame in &first[1..] {let d=network.deliver(frame.clone(),&owner);result=receiver.receive(&d).unwrap().or(result);}
    assert_eq!(result.unwrap().bytes,wire(0,1));
}

#[test]
fn reader_prefix_still_waits_for_earlier_complete_object() {
    let owner=DeliveryPort::default();let r=route(140);let mut receiver=Reassembler::new(vec![route(140)],&owner).unwrap();
    let b0=receiver.register_finalized(&r,b"E",0).unwrap();let b1=receiver.register_finalized(&r,b"E",1).unwrap();
    let mut reader=ReaderState::new(b"E".to_vec(),b"independent-B-reader".to_vec()).unwrap();let mut network=Routed::new(140);
    for (q,binding) in [(1,b1),(0,b0)] {
        for frame in fragment(1,binding,&wire(q,7),140).unwrap() {
            let delivery=network.deliver(frame,&owner);
            if let Some(object)=receiver.receive(&delivery).unwrap() {
                let (source,block)=decode_finalized(&object.bytes).unwrap();
                reader.receive_block(&source,block,TickUs(delivery.delivered_tick_us)).unwrap();
            }
        }
        if q==1 {assert!(reader.tip().is_none());assert!(reader.require_prefix_through(1).is_err());}
    }
    assert_eq!(reader.tip().unwrap().0,1);assert_eq!(reader.read(1).unwrap().proposal.q,1);
}

#[test]
fn third_incomplete_destination_object_is_bounded_without_losing_first() {
    let owner=DeliveryPort::default();let r=route(140);let mut receiver=Reassembler::new(vec![route(140)],&owner).unwrap();
    let mut network=Routed::new(140);let mut first_frames=None;
    for q in 0..3 {
        let binding=receiver.register_finalized(&r,b"E",q).unwrap();let frames=fragment(1,binding,&wire(q,7),140).unwrap();
        let d=network.deliver(frames[0].clone(),&owner);
        if q<2 {assert!(receiver.receive(&d).unwrap().is_none());}else{assert!(receiver.receive(&d).is_err());}
        if q==0{first_frames=Some(frames);}
    }
    let mut complete=None;
    for frame in &first_frames.unwrap()[1..] {let d=network.deliver(frame.clone(),&owner);complete=receiver.receive(&d).unwrap().or(complete);}
    assert_eq!(complete.unwrap().bytes,wire(0,7));
}
