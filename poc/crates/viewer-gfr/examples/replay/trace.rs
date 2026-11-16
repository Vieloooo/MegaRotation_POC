use poc_types::hex;
use poc_viewer_gfr::*;
use serde_json::{Value, json};
use sim_espresso::encode_finalized;

use super::fixture::{ESPRESSO_SOURCE, Fixture, PROFILE, Scenario};

pub fn source(fixture: &Fixture, scenario: &Scenario) {
    let setup = &fixture.setup;
    let espresso: Vec<_> = scenario.espresso.iter().map(|block| json!({
        "pelg_hex": hex(&encode_finalized(ESPRESSO_SOURCE, block).unwrap()),
        "timestamp_assessment_1s": format!("{:?}", block.proposal.assess_timestamp(1_000_000)),
    })).collect();
    let controls: Vec<_> = scenario
        .controls
        .values()
        .flatten()
        .map(|entry| {
            json!({
                "position": entry.position, "kind": entry.kind,
                "control_seq": entry.control_seq, "source_hex": hex(&entry.source_bytes),
            })
        })
        .collect();
    let parameters = scenario.parameters;
    println!(
        "{}",
        json!({
            "kind": "source", "format": "gfr-07-fixture-trace-v1", "scenario": scenario.name,
            "finality": PROFILE, "l1_authentication": "finite fixture; synthetic controls when present",
            "espresso_source_hex": hex(ESPRESSO_SOURCE), "l1_source_hex": hex(&scenario.l1_source()),
            "config": setup.config(), "config_ref": setup.config().reference().unwrap(),
            "genesis": setup.genesis(), "registry": fixture.plan.registry().record(),
            "parameters": {"announcement_us": parameters.announcement_us, "grace_us": parameters.grace_us, "expiry_us": parameters.expiry_us},
            "dep": fixture.dep_source, "l1": fixture.l1_source, "controls": controls, "espresso": espresso,
        })
    );
}

pub fn step(label: &str, reader: &str, result: &ReplayResult) {
    let state = result.state.public_state();
    let records: Vec<_> = result.added_records.iter().map(record).collect();
    let entries: Vec<_> = state.entries.iter().map(|entry| {
        let GlobalSource::Regional(key) = entry.source;
        json!({
            "h": entry.block.virtual_block.metadata.height, "acceptance": entry.position, "source": key,
            "physical": entry.block.physical_digest, "virtual": entry.block.virtual_block,
            "native": entry.block.native_block, "native_hash": entry.block.native_hash,
            "post_state": entry.block.state,
        })
    }).collect();
    let objects: Vec<_> = state.publications.results.iter().map(|(key, evidence)| {
        let proofs: Vec<_> = evidence.proofs.iter().map(|(bytes, proof)| json!({
            "payload_hex": hex(bytes), "verdict": format!("{:?}", proof.verdict),
            "locations": proof.locations.iter().map(|l| [l.q, u64::from(l.message_index)]).collect::<Vec<_>>(),
        })).collect();
        json!({"key": key, "q_avail": evidence.q_avail, "merge_verified": evidence.merge_verified,
            "exclusion": evidence.exclusion.as_ref().map(|e| format!("{e:?}")), "physical_chain": evidence.physical_chain,
            "proofs": proofs})
    }).collect();
    let bodies: Vec<_> = state.publications.bodies.iter().map(|(digest, body)| json!({"digest": digest,
        "locations": body.locations.iter().map(|l| [l.q, u64::from(l.message_index)]).collect::<Vec<_>>(),
    })).collect();
    let ignored: Vec<_> = state.publications.ignored.iter().map(|(loc, error)| json!({"location": [loc.q, u64::from(loc.message_index)], "error": format!("{error:?}")})).collect();
    println!(
        "{}",
        json!({
            "kind": "step", "label": label, "reader": reader, "pc": state.pc, "status": status(&result.status),
            "tail": tail(&state.tail), "branch": state.branch, "closed_branches": state.closed_branches,
            "closed_sessions": state.closed_sessions, "selected": state.selected.values().collect::<Vec<_>>(),
            "entries": entries, "records": records, "results": objects, "bodies": bodies, "ignored": ignored,
        })
    );
}

fn status(status: &ReplayStatus) -> Value {
    match status {
        ReplayStatus::Ready => json!({"kind": "READY"}),
        ReplayStatus::Wait {
            next_position,
            missing,
        } => json!({"kind": "WAIT", "next_position": next_position,
            "missing": missing.iter().map(|m| format!("{m:?}")).collect::<Vec<_>>() }),
        ReplayStatus::Reject(error) => json!({"kind": "REJECT", "error": format!("{error:?}")}),
        ReplayStatus::UnsupportedControl { position, kind } => {
            json!({"kind": "UNSUPPORTED_CONTROL", "position": position, "control": kind})
        }
    }
}

fn tail(tail: &Tail) -> Value {
    match tail {
        Tail::Next(descriptor) => json!({"kind": "NEXT", "descriptor": descriptor}),
        Tail::Pending(boundary) => json!({"kind": "PENDING", "boundary": boundary}),
        Tail::Halt { boundary, anchor } => {
            json!({"kind": "HALT", "boundary": boundary, "anchor": anchor})
        }
    }
}

fn record(record: &ReplayRecord) -> Value {
    let effects: Vec<_> = record
        .effects
        .iter()
        .map(|effect| match effect {
            ReplayEffect::Append {
                first,
                last,
                source: GlobalSource::Regional(key),
            } => json!({"effect": "APPEND", "first": first, "last": last, "key": key}),
            ReplayEffect::Duty { before, after } => {
                json!({"effect": "DUTY", "before": tail(before), "after": tail(after)})
            }
            ReplayEffect::Reset {
                resume_position,
                target_height,
            } => json!({"effect": "RESET", "position": resume_position, "target": target_height}),
        })
        .collect();
    let events: Vec<_> = record.events.iter().map(|event| json!({"event": event,
        "projection": hex(&lifecycle_projection(event).unwrap()), "id": lifecycle_id(event).unwrap(),
    })).collect();
    json!({"position": record.position, "position_hex": hex(&encode_position(&record.position).unwrap()),
        "input_ref": format!("{:?}", record.input_ref), "effects": effects, "events": events,
    })
}
