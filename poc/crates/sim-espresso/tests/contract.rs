use sim_espresso::{
    BlockHeader, DeliveryError, FinalizedBlock, FinalizedLogReader, LogError, LogMessage,
    MAX_BLOCK_BYTES, MAX_MESSAGE_BYTES, MAX_MESSAGES, MissingLog, ProposedBlock, PublicTimeUs,
    ReadError, ReaderState, SimEspresso, SourceError, TickUs, TimestampAssessment as Assessment,
    TimestampError, TimestampPolicy, WireError, decode_finalized, encode_finalized,
};

const SOURCE: &[u8] = b"S";

fn message(bytes: &[u8]) -> LogMessage {
    LogMessage {
        codec: 7,
        bytes: bytes.to_vec(),
    }
}

fn fixed(theta_us: u64) -> TimestampPolicy {
    TimestampPolicy::Fixed { theta_us }
}

fn block(q: u64, theta: u64, messages: Vec<LogMessage>) -> FinalizedBlock {
    FinalizedBlock {
        proposal: ProposedBlock {
            q,
            theta_us: PublicTimeUs(theta),
            proposed_at_us: TickUs(10 + q),
            live_at_proposal: true,
            messages,
        },
        finalized_at_us: TickUs(15 + q),
    }
}

#[test]
fn pe01_pe02_equal_timestamps_preserve_q_and_messages() {
    let mut log = SimEspresso::new(SOURCE.to_vec(), fixed(9_000)).unwrap();
    let duplicate = vec![message(b"P"), message(b"P")];
    for (q, tick, theta, messages) in [
        (0, 10_000, 9_000, vec![]),
        (1, 10_000, 11_000, duplicate.clone()),
        (2, 11_000, 11_000, vec![]),
    ] {
        log.set_policy(fixed(theta));
        let row = log.propose(TickUs(tick), true, messages.clone()).unwrap();
        assert_eq!(row.q, q);
        assert_eq!(row.theta_us, PublicTimeUs(theta));
        assert_eq!(row.messages, messages);
        assert_eq!(row.assess_timestamp(1_000), Assessment::WithinBound);
        log.finalize(q, TickUs(tick)).unwrap();
    }
    let mut reader = ReaderState::new(SOURCE.to_vec(), b"reader".to_vec()).unwrap();
    for q in 0..3 {
        reader
            .receive_block(SOURCE, log.finalized(q).unwrap().clone(), TickUs(80_000))
            .unwrap();
    }
    let blocks = reader.require_watermark(PublicTimeUs(11_000)).unwrap();
    assert_eq!(blocks.read(1).unwrap().proposal.messages, duplicate);
    assert_eq!(
        blocks
            .blocks()
            .iter()
            .map(|row| row.proposal.q)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
}

#[test]
fn pe02_pe03_fixed_theta_and_out_of_bound_proposal_timestamps_are_valid() {
    let mut log = SimEspresso::new(SOURCE.to_vec(), fixed(10_000)).unwrap();
    let mut reader = ReaderState::new(SOURCE.to_vec(), b"frozen".to_vec()).unwrap();
    for (q, proposal, expected) in [
        (0, 10_000, Assessment::WithinBound),
        (1, 11_000, Assessment::WithinBound),
        (
            2,
            11_001,
            Assessment::OutsideBound {
                deviation_us: 1_001,
                bound_us: 1_000,
            },
        ),
    ] {
        let row = log.propose(TickUs(proposal), true, vec![]).unwrap();
        assert_eq!(row.assess_timestamp(1_000), expected);
        let finalized = log.finalize(q, TickUs(proposal)).unwrap();
        reader
            .receive_block(SOURCE, finalized.clone(), TickUs(90_000))
            .unwrap();
    }
    assert_eq!(reader.tip(), Some((2, PublicTimeUs(10_000))));
    assert!(matches!(
        reader.require_watermark(PublicTimeUs(10_001)),
        Err(ReadError::Wait { next_q: 3, .. })
    ));
    assert!(reader.require_watermark(PublicTimeUs(10_000)).is_ok());

    let mut log = SimEspresso::new(SOURCE.to_vec(), fixed(11_001)).unwrap();
    log.propose(TickUs(10_000), true, vec![]).unwrap();
    let lead = log.finalize(0, TickUs(20_000)).unwrap();
    assert_eq!(
        lead.proposal.assess_timestamp(1_000),
        Assessment::OutsideBound {
            deviation_us: 1_001,
            bound_us: 1_000,
        }
    );
}

#[test]
fn pe04_pe05_policy_ranges_and_regression_reject_without_consuming_tick_or_q() {
    let quantize = TimestampPolicy::Offset {
        offset_us: 0,
        quantum_us: 10,
    };
    assert_eq!(quantize.theta(TickUs(109), None), Ok(PublicTimeUs(100)));
    assert_eq!(quantize.theta(TickUs(110), None), Ok(PublicTimeUs(110)));
    for (tick, offset_us, quantum_us, expected) in [
        (0, -1, 1, TimestampError::ArithmeticRange),
        (u64::MAX, 1, 1, TimestampError::ArithmeticRange),
        (10, 0, 0, TimestampError::InvalidQuantum),
    ] {
        let mut log = SimEspresso::new(
            SOURCE.to_vec(),
            TimestampPolicy::Offset {
                offset_us,
                quantum_us,
            },
        )
        .unwrap();
        assert_eq!(
            log.propose(TickUs(tick), true, vec![]),
            Err(LogError::Timestamp(expected))
        );
        log.set_policy(fixed(9));
        assert_eq!(log.propose(TickUs(0), true, vec![]).unwrap().q, 0);
    }

    let mut log = SimEspresso::new(SOURCE.to_vec(), fixed(10_000)).unwrap();
    log.propose(TickUs(10_000), true, vec![]).unwrap();
    let first = log.finalize(0, TickUs(10_001)).unwrap().clone();
    log.set_policy(fixed(9_999));
    assert_eq!(
        log.propose(TickUs(1_000_000), true, vec![]),
        Err(LogError::Timestamp(TimestampError::Regressed))
    );
    assert_eq!(log.finalized(0), Some(&first));
    log.set_policy(fixed(10_000));
    assert_eq!(log.propose(TickUs(10_001), true, vec![]).unwrap().q, 1);

    let mut log = SimEspresso::new(SOURCE.to_vec(), fixed(0)).unwrap();
    log.propose(TickUs(10_000), false, vec![]).unwrap();
    let old = log.finalize(0, TickUs(100_000)).unwrap();
    assert_eq!(
        old.proposal.assess_timestamp(1_000),
        Assessment::NotAssumedNonLive
    );
}

#[test]
fn pe06_finalize_prefix_and_duplicate_is_a_pure_replay() {
    let mut log = SimEspresso::new(SOURCE.to_vec(), fixed(100)).unwrap();
    log.propose(TickUs(10), true, vec![]).unwrap();
    log.propose(TickUs(11), true, vec![]).unwrap();
    assert!(log.finalized(0).is_none());
    assert_eq!(
        log.finalize(2, TickUs(999)),
        Err(LogError::UnknownProposal { q: 2 })
    );
    assert_eq!(
        log.finalize(1, TickUs(999)),
        Err(LogError::FinalizeOutOfOrder { next_q: 0 })
    );
    assert_eq!(log.finalize(0, TickUs(10)), Err(LogError::TickRegressed));
    let before = log.finalize(0, TickUs(12)).unwrap().clone();
    assert_eq!(log.finalize(0, TickUs(999)), Ok(&before));
    assert_eq!(log.finalize(0, TickUs(0)), Ok(&before));
    assert_eq!(
        log.finalize(1, TickUs(13)).unwrap().finalized_at_us,
        TickUs(13)
    );
}

fn unhex(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks_exact(2)
        .map(|chunk| u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn pe07_wire_literal_structure_and_full_size_bounds() {
    let rows = include_str!("../fixtures/pe-log-wire-v1.hex")
        .lines()
        .map(|line| unhex(line.split_whitespace().next().unwrap()))
        .collect::<Vec<_>>();
    let expected = [
        FinalizedBlock {
            proposal: ProposedBlock {
                q: 0,
                theta_us: PublicTimeUs(10),
                proposed_at_us: TickUs(9),
                live_at_proposal: true,
                messages: vec![],
            },
            finalized_at_us: TickUs(11),
        },
        FinalizedBlock {
            proposal: ProposedBlock {
                q: 1,
                theta_us: PublicTimeUs(10),
                proposed_at_us: TickUs(10),
                live_at_proposal: true,
                messages: vec![message(&[0xab, 0xcd]), message(&[])],
            },
            finalized_at_us: TickUs(12),
        },
    ];
    for (encoded, block) in rows.iter().zip(expected.iter()) {
        assert_eq!(
            decode_finalized(encoded),
            Ok((SOURCE.to_vec(), block.clone()))
        );
        assert_eq!(encode_finalized(SOURCE, block), Ok(encoded.clone()));
        for len in 0..encoded.len() {
            assert!(decode_finalized(&encoded[..len]).is_err());
        }
        let mut tail = encoded.clone();
        tail.push(0);
        assert_eq!(decode_finalized(&tail), Err(WireError::TrailingBytes));
    }
    for (row, at, value, error) in [
        (0, 3, 0, WireError::BadMagic),
        (0, 4, 2, WireError::UnsupportedVersion),
        (0, 6, 0, WireError::InvalidSourceId),
        (0, 7, 1, WireError::InvalidSourceId),
        (0, 35, 2, WireError::InvalidBoolean),
        (0, 36, 8, WireError::InvalidTime),
        (0, 46, 4, WireError::TooManyMessages),
        (1, 48, 0, WireError::InvalidCodec),
        (1, 54, 1, WireError::MessageTooLarge),
    ] {
        let mut invalid = rows[row].clone();
        invalid[at] = value;
        assert_eq!(decode_finalized(&invalid), Err(error), "wire offset {at}");
    }
    let mut log_row = block(0, 0, vec![message(&[]); MAX_MESSAGES]);
    assert!(encode_finalized(SOURCE, &log_row).is_ok());
    log_row.proposal.messages.push(message(&[]));
    assert_eq!(
        encode_finalized(SOURCE, &log_row),
        Err(WireError::TooManyMessages)
    );

    log_row.proposal.messages = vec![message(&vec![0; MAX_MESSAGE_BYTES]); 15];
    log_row.proposal.messages.push(message(&vec![0; 65_360]));
    let at_limit = encode_finalized(SOURCE, &log_row).unwrap();
    assert_eq!(at_limit.len(), MAX_BLOCK_BYTES);
    assert_eq!(decode_finalized(&at_limit).unwrap().1, log_row);
    log_row.proposal.messages.last_mut().unwrap().bytes.push(1);
    assert_eq!(
        encode_finalized(SOURCE, &log_row),
        Err(WireError::BlockTooLarge)
    );
    log_row.proposal.messages = vec![message(&vec![0; MAX_MESSAGE_BYTES + 1])];
    assert_eq!(
        encode_finalized(SOURCE, &log_row),
        Err(WireError::MessageTooLarge)
    );
}

#[test]
fn pe08_pe09_two_readers_wait_for_bytes_and_keep_public_positions() {
    let q0 = block(0, 100, vec![]);
    let q1 = block(1, 101, vec![message(b"P2"), message(b"B2")]);
    let q2 = block(2, 109, vec![message(b"B1")]);
    let mut a = ReaderState::new(SOURCE.to_vec(), b"A".to_vec()).unwrap();
    let mut b = ReaderState::new(SOURCE.to_vec(), b"B".to_vec()).unwrap();
    for row in [&q0, &q1, &q2] {
        a.receive_block(SOURCE, row.clone(), TickUs(19)).unwrap();
    }
    b.receive_block(SOURCE, q2.clone(), TickUs(20)).unwrap();
    assert_eq!(b.tip(), None);
    assert_eq!(
        b.read(2),
        Err(ReadError::Wait {
            next_q: 0,
            missing: MissingLog::Header
        })
    );
    b.receive_block(SOURCE, q0, TickUs(21)).unwrap();
    assert_eq!(b.tip(), Some((0, PublicTimeUs(100))));
    b.receive_header(SOURCE, BlockHeader::from(&q1), TickUs(22))
        .unwrap();
    b.receive_message(SOURCE, 1, 0, message(b"P2"), TickUs(23))
        .unwrap();
    assert_eq!(
        b.require_watermark(PublicTimeUs(109)).unwrap_err(),
        ReadError::Wait {
            next_q: 1,
            missing: MissingLog::Messages { indices: vec![1] },
        }
    );
    let snapshot = b.encode_snapshot();
    assert!(ReaderState::decode_snapshot(SOURCE, b"A", &snapshot).is_err());
    assert!(ReaderState::decode_snapshot(SOURCE, b"B", &snapshot[..snapshot.len() - 1]).is_err());
    let mut b = ReaderState::decode_snapshot(SOURCE, b"B", &snapshot).unwrap();
    assert_eq!(b.tip(), Some((0, PublicTimeUs(100))));
    b.receive_message(SOURCE, 1, 0, message(b"P2"), TickUs(24))
        .unwrap();
    b.receive_message(SOURCE, 1, 1, message(b"B2"), TickUs(25))
        .unwrap();
    assert_eq!(b.tip(), Some((2, PublicTimeUs(109))));
    assert_eq!(b.header_received_at(1), Some(TickUs(22)));
    assert_eq!(b.message_received_at(1, 0), Some(TickUs(23)));
    assert_eq!(b.receipt(1).unwrap().reader_received_at_us, TickUs(25));
    assert_eq!(b.receipt(2).unwrap().reader_received_at_us, TickUs(20));
    assert_eq!(
        a.require_prefix_through(2).unwrap().blocks(),
        b.require_prefix_through(2).unwrap().blocks()
    );

    let mut material = ReaderState::new(SOURCE.to_vec(), b"proof-first".to_vec()).unwrap();
    material
        .receive_block(SOURCE, block(0, 100, vec![message(b"P2")]), TickUs(25))
        .unwrap();
    let proof_only = material.require_prefix_through(0).unwrap();
    assert_eq!(
        proof_only.blocks()[0].proposal.messages,
        vec![message(b"P2")]
    );
    assert!(matches!(
        material.read(1),
        Err(ReadError::Wait { next_q: 1, .. })
    ));
}

#[test]
fn pe10_fragment_conflicts_and_receive_causality_preserve_state() {
    let q0 = block(0, 100, vec![message(b"first")]);
    let mut reader = ReaderState::new(SOURCE.to_vec(), b"R".to_vec()).unwrap();
    assert_eq!(
        reader.receive_block(b"foreign", q0.clone(), TickUs(15)),
        Err(DeliveryError::InvalidSource(SourceError::SourceMismatch),)
    );
    reader
        .receive_message(SOURCE, 0, 1, message(b"extra"), TickUs(15))
        .unwrap();
    let before = reader.encode_snapshot();
    assert_eq!(
        reader.receive_header(SOURCE, BlockHeader::from(&q0), TickUs(16)),
        Err(DeliveryError::InvalidSource(SourceError::InvalidIndex {
            q: 0,
            index: 1
        }),)
    );
    assert_eq!(reader.encode_snapshot(), before);

    let mut reader = ReaderState::new(SOURCE.to_vec(), b"R".to_vec()).unwrap();
    reader
        .receive_message(SOURCE, 0, 0, message(b"first"), TickUs(14))
        .unwrap();
    assert_eq!(
        reader.receive_header(SOURCE, BlockHeader::from(&q0), TickUs(16)),
        Err(DeliveryError::InvalidSource(
            SourceError::ReceivedBeforeFinalization { q: 0 }
        ),)
    );
    assert!(reader.tip().is_none());

    let mut reader = ReaderState::new(SOURCE.to_vec(), b"R".to_vec()).unwrap();
    reader
        .receive_message(SOURCE, 0, 0, message(b"first"), TickUs(15))
        .unwrap();
    reader
        .receive_header(SOURCE, BlockHeader::from(&q0), TickUs(16))
        .unwrap();
    let before = reader.encode_snapshot();
    reader
        .receive_block(SOURCE, q0.clone(), TickUs(18))
        .unwrap();
    reader
        .receive_message(SOURCE, 0, 0, message(b"first"), TickUs(20))
        .unwrap();
    assert_eq!(reader.encode_snapshot(), before);
    assert_eq!(
        reader.receive_message(SOURCE, 0, 0, message(b"changed"), TickUs(20)),
        Err(DeliveryError::InvalidSource(
            SourceError::ConflictingMessage { q: 0, index: 0 }
        ),)
    );
    assert_eq!(
        reader.receive_block(SOURCE, block(1, 99, vec![]), TickUs(20)),
        Err(DeliveryError::InvalidSource(SourceError::Regressed {
            earlier_q: 0,
            later_q: 1
        }),)
    );
    assert_eq!(reader.encode_snapshot(), before);
    assert_eq!(reader.read(0), Ok(&q0));
}
