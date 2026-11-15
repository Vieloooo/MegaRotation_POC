//! Independent execution, metadata, Merge and finalized-reader acceptance.
#[path = "component_fixture.rs"]
mod fixture;
use poc_core::{
    execution::{apply, apply_with_byte_limit},
    fixture::{DepProvider, FixtureReader, ValidatedDepFixture, ValidatedFixture, check_setup},
    merge::{PhysicalSource, SelectedTip, merge, merge_from_tip},
    metadata::{MetadataProvider, RegionalMetadata},
};
use poc_types::domain::*;
use serde_json::Value;

fn ready_reader(config: &PublicConfig) -> FixtureReader {
    let raw = fixture::l1_fixture();
    let source = ValidatedFixture::load(config, raw.clone()).unwrap();
    let mut finalizer = source.finalizer();
    let mut reader = source.reader();
    for row in &raw.rows {
        let token = finalizer
            .finalize(&row.header, row.finalized_at_us)
            .unwrap();
        reader
            .deliver_header(&token, row.finalized_at_us + 500000)
            .unwrap();
        reader
            .deliver_empty_control(
                &token,
                row.empty_control_commitment,
                row.finalized_at_us + 500000,
            )
            .unwrap();
    }
    reader
}
fn dep(metadata: &Metadata, credits: &Value) -> Vec<Vec<u8>> {
    let mut out = vec![
        Attributes {
            metadata: metadata.clone(),
        }
        .encode_raw(),
    ];
    out.extend(
        credits
            .as_array()
            .unwrap()
            .iter()
            .map(|c| fixture::credit(c).encode_raw()),
    );
    out
}

#[test]
fn apply_matches_hand_computed_state_kept_and_production_roots() {
    let input = fixture::inputs();
    let mut state = fixture::execution_anchor().state;
    for step in input["steps"].as_array().unwrap() {
        let metadata = fixture::metadata(&step["metadata"]);
        let d = dep(&metadata, &step["credits"]);
        let txs: Vec<_> = step["transactions"]
            .as_array()
            .unwrap()
            .iter()
            .map(fixture::transfer)
            .collect();
        let before = state.clone();
        let out = apply(&state, &metadata, &d, &txs).unwrap();
        let repeated = apply(&state, &metadata, &d, &txs).unwrap();
        assert_eq!(state, before);
        assert_eq!(out.state, repeated.state);
        assert_eq!(out.kept, repeated.kept);
        assert_eq!(
            out.state,
            fixture::state(&step["metadata"], &step["expected_accounts"])
        );
        let kept: Vec<_> = step["expected_kept_indices"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| txs[i.as_u64().unwrap() as usize].clone())
            .collect();
        assert_eq!(out.kept, kept);
        assert_eq!(
            out.state.root().unwrap(),
            fixture::golden(&format!("state-h{}", metadata.height))
        );
        state = out.state;
    }
}

#[test]
fn zero_self_unknown_overflow_and_reject_atomicity_match_fixed_cases() {
    let input: Value = serde_json::from_str(include_str!("execution-boundaries.json")).unwrap();
    let metadata = fixture::metadata(&input["current_metadata"]);
    for case in input["cases"].as_array().unwrap() {
        let parent = fixture::state(&input["parent_metadata"], &case["accounts"]);
        let before = parent.clone();
        let d = dep(&metadata, &case["credits"]);
        let mut txs: Vec<_> = case["txs"]
            .as_array()
            .unwrap()
            .iter()
            .map(fixture::transfer)
            .collect();
        if let Some(extra) = case["extra_raw_hex"].as_array() {
            txs.extend(extra.iter().map(|x| fixture::unhex(x.as_str().unwrap())));
        }
        let result = apply_with_byte_limit(
            &parent,
            &metadata,
            &d,
            &txs,
            case["byte_limit"].as_u64().unwrap_or(65536) as usize,
        );
        assert_eq!(parent, before, "{}", case["id"]);
        if case["expected_status"] == "REJECT" {
            assert!(result.is_err(), "{}", case["id"]);
        } else {
            let out = result.unwrap();
            assert_eq!(
                out.state,
                fixture::state(&input["current_metadata"], &case["expected_accounts"]),
                "{}",
                case["id"]
            );
            let kept: Vec<_> = case["kept"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| txs[i.as_u64().unwrap() as usize].clone())
                .collect();
            assert_eq!(out.kept, kept, "{}", case["id"]);
        }
    }
}

#[test]
fn dep_structure_and_global_limits_reject_before_committable_output() {
    let parent = fixture::execution_anchor().state;
    let before = parent.clone();
    let m = Metadata {
        height: 1,
        timestamp_us: 11000000,
        origin: 3,
    };
    let attr = Attributes {
        metadata: m.clone(),
    }
    .encode_raw();
    let credit = Credit { to: 0, amount: 5 }.encode_raw();
    let tx = Transfer {
        from: 0,
        to: 1,
        nonce: 0,
        amount: 1,
    }
    .encode_raw();
    let wrong_attr = Attributes {
        metadata: Metadata {
            origin: 4,
            ..m.clone()
        },
    }
    .encode_raw();
    for invalid in [
        vec![],
        vec![credit.clone(), attr.clone()],
        vec![attr.clone(), attr.clone()],
        vec![wrong_attr],
        vec![attr.clone(), tx.clone()],
    ] {
        assert!(apply(&parent, &m, &invalid, &[]).is_err());
        assert_eq!(parent, before);
    }
    assert!(apply(&parent, &m, &[attr.clone()], &[credit]).is_err());
    assert!(apply(&parent, &m, &[attr.clone()], &vec![tx; 1025]).is_err());
    let mut too_many = vec![attr.clone()];
    too_many.extend(vec![Credit { to: 2, amount: 0 }.encode_raw(); 1025]);
    assert!(apply(&parent, &m, &too_many, &[]).is_err());
    let mut invalid_parent = parent.clone();
    invalid_parent.accounts.insert(
        99,
        Account {
            nonce: 0,
            balance: 0,
        },
    );
    assert!(apply(&invalid_parent, &m, &[attr], &[]).is_err());
}

#[test]
fn finalized_source_and_two_readers_have_separate_explicit_progress() {
    let config = fixture::config();
    let raw = fixture::l1_fixture();
    let source = ValidatedFixture::load(&config, raw.clone()).unwrap();
    let mut finalizer = source.finalizer();
    let mut a = source.reader();
    let b = source.reader();
    assert!(matches!(
        a.header(0),
        InputStatus::Wait(InputMissing::L1Header(0))
    ));
    assert_eq!(a.finalized_tip(), None);
    assert!(finalizer.finalize(&raw.rows[0].header, 1499999).is_err());
    let zero = finalizer.finalize(&raw.rows[0].header, 1500000).unwrap();
    assert_eq!(a.finalized_tip(), None);
    assert_eq!(b.finalized_tip(), None);
    assert!(a.deliver_header(&zero, 1499999).is_err());
    assert_eq!(a.finalized_tip(), None);
    a.deliver_header(&zero, 2000000).unwrap();
    assert_eq!(a.header(0), InputStatus::Ready(raw.rows[0].header.clone()));
    assert!(matches!(
        a.require_control_coverage(0),
        InputStatus::Wait(InputMissing::ControlCoverage(0))
    ));
    assert!(matches!(b.header(0), InputStatus::Wait(_)));
    assert!(
        a.deliver_empty_control(&zero, Hash32::new([0; 32]), 2100000)
            .is_err()
    );
    assert!(matches!(
        a.require_control_coverage(0),
        InputStatus::Wait(_)
    ));
    a.deliver_empty_control(&zero, raw.rows[0].empty_control_commitment, 2100000)
        .unwrap();
    assert_eq!(a.require_control_coverage(0), InputStatus::Ready(()));
    let one = finalizer.finalize(&raw.rows[1].header, 3500000).unwrap();
    let two = finalizer.finalize(&raw.rows[2].header, 5500000).unwrap();
    a.deliver_header(&two, 6000000).unwrap();
    a.deliver_empty_control(&two, raw.rows[2].empty_control_commitment, 6000000)
        .unwrap();
    assert_eq!(a.finalized_tip().unwrap().height, 0);
    assert!(matches!(a.header(2), InputStatus::Wait(_)));
    a.deliver_header(&one, 6100000).unwrap();
    assert_eq!(a.finalized_tip().unwrap().height, 2);
    assert!(matches!(
        a.require_control_coverage(2),
        InputStatus::Wait(_)
    ));
    a.deliver_empty_control(&one, raw.rows[1].empty_control_commitment, 6100000)
        .unwrap();
    assert_eq!(a.require_control_coverage(2), InputStatus::Ready(()));
    a.deliver_header(&two, 7000000).unwrap();
    a.deliver_empty_control(&two, raw.rows[2].empty_control_commitment, 7000000)
        .unwrap();
    assert_eq!(a.finalized_tip().unwrap().height, 2);
    assert_eq!(b.finalized_tip(), None);
    assert!(matches!(
        a.header(7),
        InputStatus::Reject(InputError::FixtureExhausted {
            requested: 7,
            first: 0,
            last: 6
        })
    ));
    let mut wrong = raw.rows[1].header.clone();
    wrong.hash = Hash32::new([0x99; 32]);
    assert!(finalizer.finalize(&wrong, 8000000).is_err());
    assert_eq!(a.finalized_tip().unwrap().height, 2);
}

#[test]
fn late_finalization_and_foreign_source_tokens_cannot_backdate_delivery() {
    let config = fixture::config();
    let raw = fixture::l1_fixture();
    let source = ValidatedFixture::load(&config, raw.clone()).unwrap();
    let mut finalizer = source.finalizer();
    let mut reader = source.reader();
    let late = finalizer.finalize(&raw.rows[0].header, 2500000).unwrap();
    assert!(reader.deliver_header(&late, 2000000).is_err());
    assert_eq!(reader.finalized_tip(), None);
    reader.deliver_header(&late, 3000000).unwrap();
    let mut foreign = raw;
    foreign.rows[0].header.hash = Hash32::new([0x99; 32]);
    foreign.rows[1].header.parent_hash = foreign.rows[0].header.hash;
    let mut foreign_config = config;
    foreign_config.l1_fixture_hash = foreign.digest().unwrap();
    let other = ValidatedFixture::load(&foreign_config, foreign.clone()).unwrap();
    let token = other
        .finalizer()
        .finalize(&foreign.rows[0].header, 1500000)
        .unwrap();
    let before = reader.header(0);
    assert!(reader.deliver_header(&token, 4000000).is_err());
    assert_eq!(reader.header(0), before);
}

#[test]
fn setup_and_fixture_sources_are_config_bound() {
    let config = fixture::config();
    let initial = fixture::execution_anchor();
    assert!(
        check_setup(
            &config,
            &initial,
            fixture::dep_fixture(),
            fixture::l1_fixture()
        )
        .is_ok()
    );
    let mut bad = initial.clone();
    bad.state.accounts.get_mut(&0).unwrap().balance += 1;
    assert!(check_setup(&config, &bad, fixture::dep_fixture(), fixture::l1_fixture()).is_err());
    let mut bad = initial;
    bad.native_hash = Hash32::new([0; 32]);
    assert!(check_setup(&config, &bad, fixture::dep_fixture(), fixture::l1_fixture()).is_err());
    let mut d = fixture::dep_fixture();
    d.rows[0][0].amount += 1;
    assert!(ValidatedDepFixture::load(&config, d).is_err());
    let mut l = fixture::l1_fixture();
    l.rows[2].header.parent_hash = Hash32::new([0; 32]);
    assert!(ValidatedFixture::load(&config, l).is_err());
    let provider = ValidatedDepFixture::load(&config, fixture::dep_fixture()).unwrap();
    assert!(matches!(
        provider.at(&Metadata {
            height: 4,
            timestamp_us: 14000000,
            origin: 4
        }),
        InputStatus::Reject(InputError::FixtureExhausted {
            requested: 4,
            first: 1,
            last: 3
        })
    ));
}

#[test]
fn metadata_uses_fixed_schedule_and_anchor_origin_instead_of_reader_tip() {
    let config = fixture::config();
    let reader = ready_reader(&config);
    let provider = RegionalMetadata::new(&config, &reader).unwrap();
    for (origin, expected) in [
        (3, vec![3, 3, 4, 4, 5, 5, 6]),
        (1, vec![2, 3, 4, 4, 5, 5, 6]),
        (5, vec![5, 5, 5, 5, 5, 5, 6]),
    ] {
        let mut anchor = config.genesis.clone();
        anchor.block.origin = origin;
        for (h, os) in (1u64..=7).zip(expected) {
            let actual = provider.at(&anchor, h);
            assert_eq!(
                actual,
                InputStatus::Ready(Metadata {
                    height: h,
                    timestamp_us: 10000000 + h * 1000000,
                    origin: os
                })
            );
        }
    }
    let raw = fixture::l1_fixture();
    let source = ValidatedFixture::load(&config, raw.clone()).unwrap();
    let mut finalizer = source.finalizer();
    let mut partial = source.reader();
    for row in &raw.rows[..3] {
        let token = finalizer
            .finalize(&row.header, row.finalized_at_us)
            .unwrap();
        partial
            .deliver_header(&token, row.finalized_at_us + 500000)
            .unwrap();
    }
    let p = RegionalMetadata::new(&config, &partial).unwrap();
    assert!(matches!(
        p.at(&config.genesis, 1),
        InputStatus::Wait(InputMissing::L1Header(3))
    ));
}

struct MissingDep {
    config_ref: ConfigRef,
}
impl DepProvider for MissingDep {
    fn config_ref(&self) -> ConfigRef {
        self.config_ref
    }
    fn at(&self, m: &Metadata) -> InputStatus<Vec<Vec<u8>>> {
        InputStatus::Wait(InputMissing::Dep(m.height))
    }
}

#[test]
fn merge_gap_and_native_chain_match_independent_goldens() {
    let config = fixture::config();
    let reader = ready_reader(&config);
    let metadata = RegionalMetadata::new(&config, &reader).unwrap();
    let deposits = ValidatedDepFixture::load(&config, fixture::dep_fixture()).unwrap();
    let initial = fixture::execution_anchor();
    let physical = fixture::physical3();
    let window = Window {
        h_min: 1,
        h_max: 10,
    };
    let out = match merge(
        &initial,
        &[physical.clone()],
        &window,
        &metadata,
        &deposits,
        &config,
    ) {
        InputStatus::Ready(out) => out,
        other => panic!("{other:?}"),
    };
    assert_eq!(out.blocks.len(), 3);
    let input = fixture::inputs();
    for (i, block) in out.blocks.iter().enumerate() {
        let h = (i + 1) as u64;
        assert_eq!(block.virtual_block.metadata.height, h);
        assert_eq!(
            block.state,
            fixture::state(
                &input["steps"][i]["metadata"],
                &input["steps"][i]["expected_accounts"]
            )
        );
        assert_eq!(block.native_hash, fixture::golden(&format!("native-h{h}")));
        assert_eq!(
            block.native_block.post_state_root,
            fixture::golden(&format!("state-h{h}"))
        );
        if h == 3 {
            assert_eq!(block.physical_digest, Some(fixture::golden("physical-h3")));
            assert_eq!(
                block.virtual_block.pre_state_root,
                fixture::golden("state-h2")
            );
        } else {
            assert_eq!(block.physical_digest, None);
            assert!(block.virtual_block.transactions.is_empty());
        }
    }
    let repeated = match merge(
        &initial,
        &[physical],
        &window,
        &metadata,
        &deposits,
        &config,
    ) {
        InputStatus::Ready(out) => out,
        _ => panic!("repeat"),
    };
    assert_eq!(
        repeated.blocks.last().unwrap().native_hash,
        out.blocks.last().unwrap().native_hash
    );
    let empty = match merge(&initial, &[], &window, &metadata, &deposits, &config) {
        InputStatus::Ready(out) => out,
        _ => panic!("anchor only"),
    };
    assert!(empty.blocks.is_empty());
}

#[test]
fn merge_wrong_inputs_missing_inputs_and_config_limit_do_not_emit_prefix() {
    let config = fixture::config();
    let reader = ready_reader(&config);
    let metadata = RegionalMetadata::new(&config, &reader).unwrap();
    let deposits = ValidatedDepFixture::load(&config, fixture::dep_fixture()).unwrap();
    let initial = fixture::execution_anchor();
    let physical = fixture::physical3();
    let window = Window {
        h_min: 1,
        h_max: 10,
    };
    for which in 0..3 {
        let mut bad = physical.clone();
        match which {
            0 => bad.parent = Hash32::new([0x22; 32]),
            1 => bad.pre_state_root = fixture::golden("state-h0"),
            2 => bad.metadata.origin = 5,
            _ => unreachable!(),
        }
        assert!(matches!(
            merge(&initial, &[bad], &window, &metadata, &deposits, &config),
            InputStatus::Reject(_)
        ));
    }
    assert!(matches!(
        merge(
            &initial,
            &[physical.clone()],
            &Window {
                h_min: 4,
                h_max: 13
            },
            &metadata,
            &deposits,
            &config
        ),
        InputStatus::Reject(_)
    ));
    let missing = MissingDep {
        config_ref: config.reference().unwrap(),
    };
    assert!(matches!(
        merge(
            &initial,
            &[physical.clone()],
            &window,
            &metadata,
            &missing,
            &config
        ),
        InputStatus::Wait(InputMissing::Dep(1))
    ));
    let mut narrow = config.clone();
    narrow.byte_limit = 18;
    let nr = ready_reader(&narrow);
    let nm = RegionalMetadata::new(&narrow, &nr).unwrap();
    let nd = ValidatedDepFixture::load(&narrow, fixture::dep_fixture()).unwrap();
    assert!(matches!(
        merge(&initial, &[physical], &window, &nm, &nd, &narrow),
        InputStatus::Reject(_)
    ));
}

struct BodySource {
    body: Option<PhysicalBlock>,
}
impl PhysicalSource for BodySource {
    fn physical_block(&self, digest: Hash32) -> InputStatus<PhysicalBlock> {
        match &self.body {
            Some(body) if digest == fixture::golden("physical-h3") => {
                InputStatus::Ready(body.clone())
            }
            _ => InputStatus::Wait(InputMissing::Parent(digest)),
        }
    }
}

#[test]
fn selected_tip_missing_body_or_execution_state_waits_without_fake_padding() {
    let config = fixture::config();
    let reader = ready_reader(&config);
    let metadata = RegionalMetadata::new(&config, &reader).unwrap();
    let deposits = ValidatedDepFixture::load(&config, fixture::dep_fixture()).unwrap();
    let initial = fixture::execution_anchor();
    let window = Window {
        h_min: 1,
        h_max: 10,
    };
    let tip = fixture::golden("physical-h3");
    let body = BodySource {
        body: Some(fixture::physical3()),
    };
    let selected = SelectedTip {
        anchor: &initial.anchor,
        execution_anchor: Some(&initial),
        tip,
    };
    assert!(
        matches!(merge_from_tip(selected,&BodySource{body:None},&window,&metadata,&deposits,&config),InputStatus::Wait(InputMissing::Parent(hash))if hash==tip)
    );
    let selected = SelectedTip {
        anchor: &initial.anchor,
        execution_anchor: None,
        tip,
    };
    assert!(
        matches!(merge_from_tip(selected,&body,&window,&metadata,&deposits,&config),InputStatus::Wait(InputMissing::State(hash))if hash==initial.anchor.block.block_id)
    );
    let selected = SelectedTip {
        anchor: &initial.anchor,
        execution_anchor: Some(&initial),
        tip,
    };
    let out = match merge_from_tip(selected, &body, &window, &metadata, &deposits, &config) {
        InputStatus::Ready(out) => out,
        _ => panic!("selected tip ready"),
    };
    assert_eq!(out.blocks.len(), 3);
    assert_eq!(
        out.blocks.last().unwrap().native_hash,
        fixture::golden("native-h3")
    );
    let mut wrong = fixture::physical3();
    wrong.metadata.timestamp_us += 1;
    let selected = SelectedTip {
        anchor: &initial.anchor,
        execution_anchor: Some(&initial),
        tip,
    };
    assert!(matches!(
        merge_from_tip(
            selected,
            &BodySource { body: Some(wrong) },
            &window,
            &metadata,
            &deposits,
            &config
        ),
        InputStatus::Reject(_)
    ));
}

#[test]
fn known_invalid_context_rejects_before_missing_body_wait() {
    let config = fixture::config();
    let reader = ready_reader(&config);
    let metadata = RegionalMetadata::new(&config, &reader).unwrap();
    let deposits = ValidatedDepFixture::load(&config, fixture::dep_fixture()).unwrap();
    let initial = fixture::execution_anchor();
    let window = Window {
        h_min: 1,
        h_max: 10,
    };
    let tip = fixture::golden("physical-h3");
    let missing = BodySource { body: None };
    let selected = || SelectedTip {
        anchor: &initial.anchor,
        execution_anchor: Some(&initial),
        tip,
    };
    assert!(matches!(
        merge_from_tip(selected(), &missing, &window, &metadata, &deposits, &config),
        InputStatus::Wait(InputMissing::Parent(hash)) if hash == tip
    ));
    let wrong_provider = MissingDep {
        config_ref: ConfigRef::new(Hash32::new([0; 32])),
    };
    assert!(matches!(
        merge_from_tip(
            selected(),
            &missing,
            &window,
            &metadata,
            &wrong_provider,
            &config
        ),
        InputStatus::Reject(InputError::InvalidSource)
    ));
    let mut invalid_config = config.clone();
    invalid_config.byte_limit = 0;
    assert!(matches!(
        merge_from_tip(
            selected(),
            &missing,
            &window,
            &metadata,
            &deposits,
            &invalid_config
        ),
        InputStatus::Reject(_)
    ));
    let mut changed_config = config.clone();
    changed_config.byte_limit = 18;
    assert!(matches!(
        merge_from_tip(
            selected(),
            &missing,
            &window,
            &metadata,
            &deposits,
            &changed_config
        ),
        InputStatus::Reject(InputError::InvalidSource)
    ));
}
