use crate::{
    aux::{Identity, RecoveryLedger},
    native_config::Resolved,
    native_wire as p, native_wire_support as wire,
    root_step::StepJournal,
};
use serde_json::json;
use std::{fs, path::PathBuf};

fn place(label: &str) -> PathBuf {
    for n in 0..u64::MAX {
        let path = std::env::temp_dir().join(format!(
            "native-recovery-independent-{}-{label}-{n}",
            std::process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => {
                eprintln!("ARTIFACT {}", path.display());
                return path;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => panic!("reserve {}: {e}", path.display()),
        }
    }
    panic!("temporary counter exhausted")
}

#[test]
fn aux_authorized_complete_tail_with_crash_left_pending_head_recovers() {
    let root = place("aux-pending");
    let id = Identity {
        run: [1; 32],
        config: [2; 32],
        worker: 0,
    };
    let mut ledger = RecoveryLedger::create(&root, id).unwrap();
    let floor = ledger
        .append("GRANT_INTENT", None, &vec![7u64, 17])
        .unwrap();
    let old_head = fs::read(root.join("native-aux.head")).unwrap();
    let next = ledger
        .append("PREPARATION", Some(floor.hash), &vec![11u64, 12])
        .unwrap();
    let new_head = fs::read(root.join("native-aux.head")).unwrap();
    drop(ledger);
    // Exact crash image: synced complete descendant, old committed head, synced
    // new pending head, then death before rename. Original data is retained.
    fs::write(root.join("native-aux.head"), &old_head).unwrap();
    fs::write(root.join("native-aux.head.pending"), &new_head).unwrap();
    let allow = |r: &crate::aux::LedgerRecord| {
        r.header.sequence == next.sequence
            && r.header.kind == "PREPARATION"
            && r.header.grant_intent_ref == Some(crate::aux::hex(&floor.hash))
            && r.data == b"[11,12]"
    };
    let result = RecoveryLedger::open(&root, id, floor, Some(&allow));
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&json!({"leaf":"auxiliary","expected":"authorized descendant promoted despite valid crash-left pending head","minimum_sequence":floor.sequence,"next_sequence":next.sequence,"opened":result.is_ok(),"error":result.as_ref().err().map(|e|format!("{e:?}"))})).unwrap()).unwrap();
    let mut reopened =
        result.expect("valid crash-left auxiliary pending head must not block promotion");
    assert_eq!(reopened.receipt(), next);
    assert_eq!(fs::read(root.join("native-aux.head")).unwrap(), new_head);
    reopened
        .append("FOLLOWUP", Some(next.hash), &vec![13u64])
        .unwrap();
}

fn step(config: &Resolved, id: u64) -> (p::Grant, p::Ack) {
    let mut grant = p::Grant {
        binding: Some(config.binding(0)),
        logical_step_id: id,
        event_id: id,
        tick_us: 1000000 + id,
        tie_rank: wire::tie_hash(7, id).to_vec(),
        input: Some(p::Input {
            kind: Some(p::input::Kind::RequestSample(p::Empty {})),
        }),
        ..Default::default()
    };
    grant.grant_hash = wire::grant_hash(&grant).to_vec();
    let ack = p::Ack {
        binding: grant.binding.clone(),
        logical_step_id: id,
        event_id: id,
        grant_hash: grant.grant_hash.clone(),
        status: 1,
        ..Default::default()
    };
    (grant, ack)
}

#[test]
fn root_authorized_complete_tail_with_crash_left_pending_head_recovers() {
    let root = place("root-pending");
    let (config, _) = Resolved::generate("healthy", 7).unwrap();
    let mut journal = StepJournal::create(&root, &config, 0).unwrap();
    let (g1, a1) = step(&config, 1);
    journal.commit(g1, a1).unwrap();
    let floor = journal.receipt();
    let old_head = fs::read(root.join("root-step.head")).unwrap();
    let (g2, a2) = step(&config, 2);
    let exact = journal.commit(g2.clone(), a2).unwrap();
    let next = journal.receipt();
    let new_head = fs::read(root.join("root-step.head")).unwrap();
    drop(journal);
    fs::write(root.join("root-step.head"), &old_head).unwrap();
    fs::write(root.join("root-step.head.pending"), &new_head).unwrap();
    let result = StepJournal::open_recovery(&root, &config, 0, floor, Some(&g2));
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&json!({"leaf":"root-step","expected":"authorized descendant promoted despite valid crash-left pending head","minimum_sequence":floor.sequence,"next_sequence":next.sequence,"opened":result.is_ok(),"error":result.as_ref().err().map(|e|format!("{e:?}"))})).unwrap()).unwrap();
    let mut reopened = result.expect("valid crash-left root pending head must not block promotion");
    assert_eq!(reopened.receipt(), next);
    assert_eq!(reopened.cached(&g2).unwrap(), Some(exact));
    assert_eq!(fs::read(root.join("root-step.head")).unwrap(), new_head);
    let (g3, a3) = step(&config, 3);
    reopened.commit(g3, a3).unwrap();
}
