//! Raw grant identity for the recovery ledger. Native actors receive only
//! their recovery hooks, while grant authority stays with the controller.

use crate::{Id, hash_parts};

pub(crate) fn input_id(
    run: Id,
    config: Id,
    worker: u32,
    incarnation: u64,
    sequence: u64,
    grant_hash: Id,
    grant_bytes: &[u8],
) -> Id {
    hash_parts(
        b"I1b/recovery-input/v1",
        &[
            &run,
            &config,
            &worker.to_be_bytes(),
            &incarnation.to_be_bytes(),
            &sequence.to_be_bytes(),
            &grant_hash,
            grant_bytes,
        ],
    )
}

use crate::{
    ControlledDriver, Error, Result,
    driver::{Authority, Grant as DriverGrant},
    native_recovery_ledger::{RecoveryLedger, hex},
};
use commonware_consensus::simplex::recovery::RecoveryReceipt;
use serde::Serialize;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ReceiptRecord {
    sequence: u64,
    hash: String,
}
impl From<RecoveryReceipt> for ReceiptRecord {
    fn from(value: RecoveryReceipt) -> Self {
        Self {
            sequence: value.sequence,
            hash: hex(&value.hash),
        }
    }
}

/// An intent is serialized independently of actor scheduling, and must be
/// committed before the root emits the corresponding NativeRx message.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct GrantIntent {
    pub input_id: String,
    pub input_sequence: u64,
    pub original_incarnation: u64,
    pub grant_id: u64,
    pub grant_hash: String,
    pub canonical_grant_bytes: String,
    pub predecessor_root_receipt: ReceiptRecord,
    pub predecessor_cut_hash: String,
    pub input_kind: &'static str,
    pub source_member: Option<u32>,
    pub channel: Option<u32>,
    pub exact_envelope_bytes: Option<String>,
    pub native_bytes: Option<String>,
    pub original_receive_tick_us: u64,
    pub original_receive_local_us: Option<i64>,
    pub transport_binding_verified: bool,
    pub decoded: bool,
    pub elected_leader_checked: bool,
    pub signature_state: &'static str,
}

impl GrantIntent {
    pub fn from_grant(
        config: &crate::native_config::Resolved,
        worker: u32,
        incarnation: u64,
        input_sequence: u64,
        grant: &crate::native_wire::Grant,
        predecessor_root: RecoveryReceipt,
        predecessor_cut: Id,
        original_local_us: Option<i64>,
    ) -> Result<(Id, Self)> {
        use crate::native_wire::{Channel, input::Kind};
        use crate::{native_network, native_wire_support as wire};
        use prost::Message as _;
        let mut expected = config.binding(worker);
        expected.incarnation = incarnation;
        if incarnation == 0
            || input_sequence == 0
            || grant.binding.as_ref() != Some(&expected)
            || grant.logical_step_id == 0
            || grant.event_id == 0
            || grant.tie_rank != wire::tie_hash(config.base.seed, grant.event_id)
            || grant.grant_hash != wire::grant_hash(grant)
        {
            return Err(Error("invalid native recovery grant identity"));
        }
        let kind = grant
            .input
            .as_ref()
            .and_then(|i| i.kind.as_ref())
            .ok_or(Error("invalid native recovery grant input"))?;
        let grant_bytes = grant.encode_to_vec();
        let grant_hash = grant
            .grant_hash
            .as_slice()
            .try_into()
            .map_err(|_| Error("native grant hash length"))?;
        let id = input_id(
            config.run_id,
            config.config_hash,
            worker,
            incarnation,
            input_sequence,
            grant_hash,
            &grant_bytes,
        );
        let mut result = Self {
            input_id: hex(&id),
            input_sequence,
            original_incarnation: incarnation,
            grant_id: grant.logical_step_id,
            grant_hash: hex(&grant_hash),
            canonical_grant_bytes: hex(&grant_bytes),
            predecessor_root_receipt: predecessor_root.into(),
            predecessor_cut_hash: hex(&predecessor_cut),
            input_kind: match kind {
                Kind::RequestSample(_) => "REQUEST_SAMPLE",
                Kind::Sample(_) => "SAMPLE",
                Kind::Start(_) => "START",
                Kind::Deliver(_) => "DELIVER",
                Kind::TimerFired(_) => "TIMER_FIRED",
                Kind::WorkCompleted(_) => "WORK_COMPLETED",
                Kind::Close(_) => "CLOSE",
                Kind::DriveReady(_) => "DRIVE_READY",
                Kind::FlushArchive(_) => "FLUSH_ARCHIVE",
                Kind::Recover(_) => "RECOVER",
                Kind::RecoveryWork(_) => "RECOVERY_WORK",
                Kind::ArmFault(_) => "ARM_FAULT",
            },
            source_member: None,
            channel: None,
            exact_envelope_bytes: None,
            native_bytes: None,
            original_receive_tick_us: grant.tick_us,
            original_receive_local_us: original_local_us,
            transport_binding_verified: false,
            decoded: true,
            elected_leader_checked: false,
            signature_state: "NOT_APPLICABLE",
        };
        if let Kind::Deliver(envelope) = kind {
            let exact = envelope.encode_to_vec();
            if exact.len() > wire::MAX_PAYLOAD {
                return Err(Error("recovery envelope exceeds raw custody bound"));
            }
            result.source_member = Some(envelope.src_member_id);
            result.channel = u32::try_from(envelope.channel).ok();
            result.exact_envelope_bytes = Some(hex(&exact));
            result.native_bytes = Some(hex(&envelope.payload_bytes));
            result.decoded = false;
            if let Ok(channel) = native_network::validate_envelope(config, envelope) {
                result.transport_binding_verified = envelope.dst_member_id == worker;
                if result.transport_binding_verified {
                    result.decoded = channel == Channel::Vote
                        && native_network::decode_vote(&envelope.payload_bytes).is_ok();
                    if matches!(channel, Channel::Vote | Channel::Certificate) {
                        result.signature_state = "PENDING";
                    }
                }
            }
        }
        Ok((id, result))
    }
}

#[derive(serde::Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BoundaryReceipt {
    sequence: u64,
    hash: String,
}

#[derive(serde::Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BoundaryFrame {
    logical_offset: u64,
    frame_hash: String,
}

#[derive(serde::Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BoundarySection {
    section: u64,
    durable_len: u64,
    frames: Vec<BoundaryFrame>,
}

#[derive(serde::Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NativeBoundary {
    generation: u64,
    sections: Vec<BoundarySection>,
    witness_receipt: BoundaryReceipt,
    barrier_ids: Vec<String>,
    eligible_payload_ids: Vec<String>,
    submission_ids: Vec<String>,
}

/// Verified shape of the selected cut's original native boundary. The
/// controller first validates the complete cut and its application evidence.
pub(crate) struct AuthorizedEmptyBoundary {
    cut: Id,
    generation: u64,
    witness: crate::WitnessHead,
}

impl AuthorizedEmptyBoundary {
    pub fn from_validated_cut(
        ledger: &RecoveryLedger,
        trusted_aux: RecoveryReceipt,
        expected_cut: Id,
        witness: crate::WitnessHead,
        generation: u64,
    ) -> Result<Self> {
        #[derive(serde::Deserialize)]
        struct CutBoundary {
            schema: u32,
            native_journal_boundary: NativeBoundary,
            cut_hash: String,
        }
        let record = ledger
            .records()
            .iter()
            .find(|record| record.receipt == trusted_aux)
            .filter(|record| record.header.kind == "RECOVERY_CUT")
            .ok_or(Error("empty witness branch lacks anchored recovery cut"))?;
        let document: CutBoundary = serde_json::from_slice(&record.data)
            .map_err(|_| Error("invalid recovery cut native boundary"))?;
        let check = format!(",\"cut_hash\":\"{}\"}}", hex(&expected_cut));
        let prefix = record
            .data
            .strip_suffix(check.as_bytes())
            .ok_or(Error("native boundary cut hash is not encoded canonically"))?;
        let mut cut_payload = prefix.to_vec();
        cut_payload.push(b'}');
        let boundary = document.native_journal_boundary;
        let encoded_boundary =
            serde_json::to_vec(&boundary).map_err(|_| Error("encode cut boundary"))?;
        let field = [
            b"\"native_journal_boundary\":".as_slice(),
            &encoded_boundary,
        ]
        .concat();
        if document.schema != 1
            || document.cut_hash != hex(&expected_cut)
            || hash_parts(b"I1b/recovery-cut/v1", &[&cut_payload]) != expected_cut
            || !record
                .data
                .windows(field.len())
                .any(|candidate| candidate == field)
            || generation == 0
            || boundary.generation != generation
            || witness.sequence == 0
            || boundary.witness_receipt.sequence != witness.sequence
            || boundary.witness_receipt.hash != hex(&witness.hash)
            || !boundary.sections.is_empty()
            || !boundary.barrier_ids.is_empty()
            || !boundary.eligible_payload_ids.is_empty()
            || !boundary.submission_ids.is_empty()
        {
            return Err(Error(
                "native store does not have an authenticated empty boundary",
            ));
        }
        Ok(Self {
            cut: expected_cut,
            generation,
            witness,
        })
    }

    pub fn cut(&self) -> Id {
        self.cut
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn witness(&self) -> crate::WitnessHead {
        self.witness
    }
}

struct PendingAux {
    token: u64,
    kind: &'static str,
    input: Option<Id>,
    bytes: Vec<u8>,
}
struct LedgerState {
    ledger: RecoveryLedger,
    frontier: u64,
    pending: VecDeque<PendingAux>,
    grant: Option<(u64, Id)>,
}

#[derive(Clone)]
pub(crate) struct LedgerWriter {
    state: Arc<Mutex<LedgerState>>,
}

impl LedgerWriter {
    pub fn new(ledger: RecoveryLedger) -> Self {
        let frontier = ledger.receipt().sequence;
        Self {
            state: Arc::new(Mutex::new(LedgerState {
                ledger,
                frontier,
                pending: VecDeque::new(),
                grant: None,
            })),
        }
    }

    pub async fn write_ahead(
        &self,
        driver: &mut ControlledDriver,
        grant: DriverGrant,
        cid: Id,
        intent_id: Id,
        intent: GrantIntent,
    ) -> Result<RecoveryReceipt> {
        let this = self.clone();
        driver
            .persist_root_auxiliary(grant, cid, move || {
                let bytes =
                    serde_json::to_vec(&intent).map_err(|_| Error("encode native grant intent"))?;
                let mut state = this.state.lock().unwrap();
                if !state.pending.is_empty()
                    || intent.grant_id != grant.id
                    || intent.input_id != hex(&intent_id)
                    || state.grant.is_some_and(|(last, _)| last >= grant.id)
                    || state.frontier != state.ledger.receipt().sequence
                    || state.frontier >= 8192
                {
                    return Err(Error("incompatible native auxiliary write-ahead frontier"));
                }
                let receipt =
                    state
                        .ledger
                        .append_canonical("GRANT_INTENT", Some(intent_id), bytes)?;
                state.frontier = receipt.sequence;
                state.grant = Some((grant.id, intent_id));
                Ok(receipt)
            })
            .await
    }

    pub fn stage(&self, kind: &'static str, value: &impl Serialize) -> Result<u64> {
        let bytes = serde_json::to_vec(value).map_err(|_| Error("encode native auxiliary fact"))?;
        self.stage_canonical(kind, bytes)
    }

    fn stage_canonical(&self, kind: &'static str, bytes: Vec<u8>) -> Result<u64> {
        let mut state = self.state.lock().unwrap();
        let (_, input) = state
            .grant
            .ok_or(Error("auxiliary fact without durable grant intent"))?;
        let token = state
            .frontier
            .checked_add(1)
            .filter(|seq| *seq <= 8192)
            .ok_or(Error("native auxiliary fact bound exceeded"))?;
        state.pending.push_back(PendingAux {
            token,
            kind,
            input: Some(input),
            bytes,
        });
        state.frontier = token;
        Ok(token)
    }

    pub fn frontier(&self) -> u64 {
        self.state.lock().unwrap().frontier
    }

    fn persist(&self, through: u64) -> Result<RecoveryReceipt> {
        let mut state = self.state.lock().unwrap();
        if through == 0 || through > state.frontier {
            return Err(Error("invalid native auxiliary flush frontier"));
        }
        while state
            .pending
            .front()
            .is_some_and(|next| next.token <= through)
        {
            let pending = state
                .pending
                .pop_front()
                .expect("checked nonempty auxiliary queue");
            let receipt =
                state
                    .ledger
                    .append_canonical(pending.kind, pending.input, pending.bytes)?;
            if receipt.sequence != pending.token {
                return Err(Error("native auxiliary record identity changed"));
            }
        }
        let records = state.ledger.records();
        let index =
            usize::try_from(through - 1).map_err(|_| Error("auxiliary frontier exceeds usize"))?;
        records
            .get(index)
            .map(|record| record.receipt)
            .ok_or(Error("native auxiliary flush omitted a registered fact"))
    }

    pub async fn flush_actor(
        &self,
        authority: &Authority,
        through: u64,
    ) -> Result<RecoveryReceipt> {
        let this = self.clone();
        let wait = authority.submit_auxiliary(move || this.persist(through))?;
        let result = wait
            .await
            .map_err(|_| Error("lost auxiliary actor result"))
            .and_then(|v| v.map_err(|_| Error("native auxiliary actor physical error")))
            .and_then(|v| v);
        if let Err(error) = &result {
            authority.fail(error.0);
        }
        result
    }

    pub async fn flush_root(
        &self,
        driver: &mut ControlledDriver,
        grant: DriverGrant,
        cid: Id,
        through: u64,
    ) -> Result<RecoveryReceipt> {
        let this = self.clone();
        driver
            .persist_root_auxiliary(grant, cid, move || this.persist(through))
            .await
    }
}

#[cfg(test)]
pub(crate) mod fixture {
    use commonware_consensus::simplex::recovery as r;
    use futures::future::BoxFuture;
    use std::sync::Mutex;

    pub struct Hooks {
        cut: r::CutRef,
        pub readies: Mutex<Vec<(r::ActorId, r::CutRef, [u8; 32])>>,
    }
    impl Hooks {
        pub fn new(cut: r::CutRef) -> Self {
            Self {
                cut,
                readies: Mutex::default(),
            }
        }
    }

    impl r::RecoveryStartup for Hooks {
        fn ready(
            &self,
            actor: r::ActorId,
            cut: r::CutRef,
            state_hash: [u8; 32],
        ) -> Result<(), r::RecoveryError> {
            let mut readies = self.readies.lock().unwrap();
            assert!(!readies.iter().any(|entry| entry.0 == actor));
            assert_eq!(cut, self.cut);
            readies.push((actor, cut, state_hash));
            Ok(())
        }
        fn released(&self) -> BoxFuture<'_, Result<(), r::RecoveryError>> {
            Box::pin(async { Ok(()) })
        }
    }
}

#[cfg(test)]
mod boundary_tests {
    use super::*;
    use crate::{
        WitnessLedger,
        native_config::Resolved,
        native_recovery_ledger::Identity,
        witness_recovery::{RecoveryPaths, recover_empty_witness_generation},
    };
    use std::fs;

    #[test]
    fn anchored_empty_witness_history_moves_generation_without_fake_storage() {
        let root = (0..u64::MAX)
            .map(|i| {
                std::env::temp_dir().join(format!("native-absence-{}-{i}", std::process::id()))
            })
            .find(|path| fs::create_dir(path).is_ok())
            .unwrap();
        let (config, _) = Resolved::generate("healthy", 7).unwrap();
        let policy = config.witness(1).unwrap();
        let mut next = policy.clone();
        next.seal.config.generation = 2;
        let storage = root.join("native");
        fs::create_dir(&storage).unwrap();
        let witness = root.join("witness");
        let ledger = WitnessLedger::create(&witness, policy.clone()).unwrap();
        let head = ledger.head();
        drop(ledger);
        let identity = Identity {
            run: config.run_id,
            config: config.config_hash,
            worker: 1,
        };
        let mut aux = RecoveryLedger::create(&root, identity).unwrap();
        let boundary = NativeBoundary {
            generation: 1,
            sections: vec![],
            witness_receipt: BoundaryReceipt {
                sequence: head.sequence,
                hash: hex(&head.hash),
            },
            barrier_ids: vec![],
            eligible_payload_ids: vec![],
            submission_ids: vec![],
        };
        let make_cut = |boundary: &NativeBoundary| {
            let json = serde_json::to_string(boundary).unwrap();
            let payload = format!("{{\"schema\":1,\"native_journal_boundary\":{json}}}");
            let hash = hash_parts(b"I1b/recovery-cut/v1", &[payload.as_bytes()]);
            let bytes = format!(
                "{},\"cut_hash\":\"{}\"}}",
                &payload[..payload.len() - 1],
                hex(&hash)
            );
            (hash, bytes.into_bytes())
        };
        let (cut, bytes) = make_cut(&boundary);
        let receipt = aux
            .append_canonical("RECOVERY_CUT", Some([5; 32]), bytes)
            .unwrap();
        drop(aux);
        let aux = RecoveryLedger::open(&root, identity, receipt, None).unwrap();
        let empty =
            AuthorizedEmptyBoundary::from_validated_cut(&aux, receipt, cut, head, 1).unwrap();
        assert!(AuthorizedEmptyBoundary::from_validated_cut(&aux, receipt, cut, head, 2).is_err());
        assert!(
            AuthorizedEmptyBoundary::from_validated_cut(&aux, receipt, [44; 32], head, 1).is_err()
        );
        let target = root.join("next");
        let paths = || RecoveryPaths {
            source_storage: &storage,
            source_ledger: &witness,
            fresh_target: &target,
        };
        let partition = storage.join(&policy.seal.config.partition);
        fs::create_dir(&partition).unwrap();
        assert!(
            recover_empty_witness_generation(paths(), policy.clone(), next.clone(), &empty)
                .is_err()
        );
        assert!(!target.exists());
        fs::remove_dir(&partition).unwrap();
        let head_path = root.join("witness.head");
        let head_copy = fs::read(&head_path).unwrap();
        fs::remove_file(&head_path).unwrap();
        assert!(
            recover_empty_witness_generation(paths(), policy.clone(), next.clone(), &empty)
                .is_err()
        );
        assert!(!target.exists());
        fs::write(&head_path, &head_copy).unwrap();

        let report = recover_empty_witness_generation(paths(), policy, next, &empty).unwrap();
        assert_eq!(report.new_head.sequence, head.sequence + 2);
        assert_eq!(report.source.old_head_hash, head.hash);
        assert!(report.source.source_files.is_empty());
        assert!(!partition.exists());
        assert!(
            fs::read_dir(target.join("native"))
                .unwrap()
                .next()
                .is_none()
        );
        assert_eq!(fs::read(&head_path).unwrap(), head_copy);
        assert_eq!(fs::read(target.join("source-snapshot/absence.json")).unwrap(),
            format!("{{\"partition\":\"i1b-native-voter\",\"source_files\":[],\"absent\":true,\"cut_hash\":\"{}\"}}", hex(&cut)).into_bytes());
        drop(aux);
        fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod custody_tests {
    use super::*;
    use crate::{
        ClockSample, Event, Input, Profile,
        native_config::{Resolved, START},
        native_recovery_ledger::Identity,
        native_wire::{Empty, Grant, Input as NativeInput, input::Kind},
        native_wire_support as wire,
    };
    use std::fs;

    #[test]
    fn write_ahead_and_actor_flush_share_one_owned_ledger() {
        let root = (0..u64::MAX)
            .map(|i| {
                std::env::temp_dir().join(format!("native-aux-custody-{}-{i}", std::process::id()))
            })
            .find(|path| fs::create_dir(path).is_ok())
            .unwrap();
        let (config, _) = Resolved::generate("healthy", 7).unwrap();
        let identity = Identity {
            run: config.run_id,
            config: config.config_hash,
            worker: 0,
        };
        let writer = LedgerWriter::new(RecoveryLedger::create(&root, identity).unwrap());
        let profile = config.profile(0);
        let final_receipt = ControlledDriver::run_async(
            commonware_runtime::tokio::Config::default()
                .with_storage_directory(root.join("native")),
            profile.clone(),
            async |driver| {
                driver
                    .grant_async(
                        DriverGrant {
                            id: 1,
                            tick_us: START,
                        },
                        Input::RequestSample,
                        |_| Ok(()),
                    )
                    .await?;
                driver
                    .grant_async(
                        DriverGrant {
                            id: 2,
                            tick_us: START,
                        },
                        Input::DeliverSample(ClockSample {
                            run_id: config.run_id,
                            node: 0,
                            incarnation: 1,
                            clock_domain: profile.clock_domain(),
                            rate_profile: Profile::rate_profile(),
                            request_id: 1,
                            sequence: 1,
                            as_of_tick_us: START,
                            node_time_at_as_of_us: START as i64 + 1000,
                            uncertainty_us: 1000,
                            lease_end_tick_us: START + 100_000,
                        }),
                        |_| Ok(()),
                    )
                    .await?;
                let mut input = Grant {
                    binding: Some(config.binding(0)),
                    logical_step_id: 3,
                    event_id: 3,
                    tick_us: START,
                    tie_rank: wire::tie_hash(7, 3).to_vec(),
                    input: Some(NativeInput {
                        kind: Some(Kind::DriveReady(Empty {})),
                    }),
                    ..Default::default()
                };
                input.grant_hash = wire::grant_hash(&input).to_vec();
                let (id, intent) = GrantIntent::from_grant(
                    &config,
                    0,
                    1,
                    1,
                    &input,
                    RecoveryReceipt::default(),
                    [0; 32],
                    Some(START as i64 + 1000),
                )?;
                let grant = DriverGrant {
                    id: 3,
                    tick_us: START,
                };
                let before = writer
                    .write_ahead(driver, grant, config.cid, id, intent)
                    .await?;
                assert_eq!(before.sequence, 1);
                assert_eq!(
                    RecoveryLedger::open(&root, identity, before, None)?
                        .records()
                        .len(),
                    1
                );
                let action_writer = writer.clone();
                let step = driver
                    .grant_async(grant, Input::None, |driver| {
                        let authority = driver.recovery_authority(config.cid);
                        driver.spawn_session(config.cid, config.base.stop, move |_| async move {
                            let token =
                                action_writer.stage("FIXTURE_CALLBACK", &[1u64, 2]).unwrap();
                            assert_eq!(token, 2);
                            let receipt =
                                action_writer.flush_actor(&authority, token).await.unwrap();
                            assert_eq!(receipt.sequence, 2);
                            action_writer.stage("FIXTURE_ROOT_FRONTIER", &4u64).unwrap();
                        })
                    })
                    .await?;
                let issued = step
                    .events
                    .iter()
                    .find_map(|event| match event {
                        Event::IoIssued {
                            identity, target, ..
                        } if target.partition == "native-aux" => Some(identity.io_id),
                        _ => None,
                    })
                    .expect("actor flush used root disk queue");
                assert!(step.events.iter().any(|event| matches!(event,
                    Event::IoReleased {io_id,delivered:true} if *io_id == issued)));
                assert_eq!(writer.frontier(), 3);
                let clock = driver.recovery_clock_checkpoint(config.cid)?;
                assert_eq!(
                    serde_json::to_value(clock).unwrap()["last_issued_request_id"],
                    1
                );
                writer.flush_root(driver, grant, config.cid, 3).await
            },
        )
        .unwrap();
        assert_eq!(final_receipt.sequence, 3);
        let opened = RecoveryLedger::open(&root, identity, final_receipt, None).unwrap();
        assert_eq!(
            opened
                .records()
                .iter()
                .map(|r| r.header.kind.as_str())
                .collect::<Vec<_>>(),
            ["GRANT_INTENT", "FIXTURE_CALLBACK", "FIXTURE_ROOT_FRONTIER"]
        );
        let input = &opened.records()[0];
        let witness: serde_json::Value = serde_json::from_slice(&input.data).unwrap();
        assert_eq!(
            input.header.grant_intent_ref.as_deref(),
            witness["input_id"].as_str()
        );
        assert_eq!(
            opened.records()[1].header.grant_intent_ref,
            input.header.grant_intent_ref
        );
        drop(opened);
        fs::remove_dir_all(root).unwrap();
    }
}

/// Canonical ordered encoding for native-owned types that intentionally have
/// no serde dependency in Commonware.
macro_rules! record_json {
    ($($key:ident: $value:expr),* $(,)?) => {{
        #[allow(non_camel_case_types)]
        struct Fields<$($key),*> { $($key: $key),* }
        #[allow(non_camel_case_types)]
        impl<$($key: Serialize),*> Serialize for Fields<$($key),*> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok,S::Error> {
                use serde::ser::SerializeStruct as _;
                let mut fields = serializer.serialize_struct("RecoveryRecord", [$(stringify!($key)),*].len())?;
                $(fields.serialize_field(stringify!($key), &self.$key)?;)*
                fields.end()
            }
        }
        Fields { $($key: $value),* }
    }};
}

use commonware_consensus::simplex::recovery as native;
use futures::future::BoxFuture;
use std::collections::{BTreeMap, BTreeSet};

type StampedIngress = (Id, Vec<u8>, Vec<u8>);

#[derive(Default)]
struct ActorFacts {
    accepted_inputs: BTreeMap<Id, GrantIntent>,
    consumed: BTreeMap<u32, VecDeque<StampedIngress>>,
    consumed_ids: BTreeSet<Id>,
    claims: VecDeque<(native::RecordToken, native::CallbackClaim)>,
    operations: BTreeMap<native::RecordToken, native::OperationRecord>,
    helper_results: BTreeMap<native::RecordToken, Option<Vec<u8>>>,
    proposals: Vec<native::ProposalFact>,
    actor_phase: Option<(native::RecordToken, native::ActorPhase)>,
    timing: Option<(native::RecordToken, native::TimingSnapshot)>,
}

/// One store joins native actor observations, the exact application dispatch
/// and the controller's protected auxiliary WAL.
pub(crate) struct ActorStore {
    writer: LedgerWriter,
    authority: Authority,
    epoch: u64,
    restored: native::RecoveryBundle,
    restored_boundary: Vec<native::JournalPosition>,
    facts: Mutex<ActorFacts>,
}

impl ActorStore {
    pub fn new(
        writer: LedgerWriter,
        authority: Authority,
        epoch: u64,
        restored: native::RecoveryBundle,
        restored_boundary: Vec<native::JournalPosition>,
    ) -> Self {
        let facts = ActorFacts {
            operations: restored
                .operations
                .iter()
                .map(|operation| (operation.claimed_sequence, operation.clone()))
                .collect(),
            proposals: restored.proposals.clone(),
            ..Default::default()
        };
        Self {
            writer,
            authority,
            epoch,
            restored,
            restored_boundary,
            facts: Mutex::new(facts),
        }
    }

    pub async fn write_ahead(
        &self,
        driver: &mut ControlledDriver,
        grant: DriverGrant,
        intent_ref: Id,
        intent: GrantIntent,
    ) -> Result<RecoveryReceipt> {
        let receipt = self
            .writer
            .write_ahead(
                driver,
                grant,
                self.authority.cid,
                intent_ref,
                intent.clone(),
            )
            .await?;
        if intent.transport_binding_verified && intent.original_receive_local_us.is_some() {
            let mut facts = self.facts.lock().unwrap();
            if facts.accepted_inputs.insert(intent_ref, intent).is_some() {
                return Err(Error("duplicate durable native ingress identity"));
            }
        }
        Ok(receipt)
    }

    pub fn consumed(
        &self,
        channel: u32,
        source_member: u32,
        key: Vec<u8>,
        bytes: Vec<u8>,
        id: Id,
    ) -> Result<()> {
        let mut facts = self.facts.lock().unwrap();
        let intent = facts
            .accepted_inputs
            .get(&id)
            .ok_or(Error("native ingress has no committed input"))?;
        if intent.channel != Some(channel)
            || intent.source_member != Some(source_member)
            || intent.native_bytes.as_deref() != Some(&hex(&bytes))
        {
            return Err(Error("native ingress differs from committed input"));
        }
        if !facts.consumed_ids.insert(id) {
            return Err(Error("duplicate consumed native input"));
        }
        self.writer.stage(
            "INGRESS_CONSUMED",
            &record_json! {
                input_ref: hex(&id), channel: channel, source_member: source_member,
            },
        )?;
        if matches!(channel, 1 | 2) {
            facts
                .consumed
                .entry(channel)
                .or_default()
                .push_back((id, key, bytes));
        }
        Ok(())
    }

    fn stage_actor(&self, record: native::RecoveryRecord) -> Result<native::RecordToken> {
        use native::RecoveryRecord as R;
        let mut facts = self.facts.lock().unwrap();
        match record {
            R::ProposalSelected {
                native_proposal_bytes,
                origin,
            } => {
                let input = facts
                    .accepted_inputs
                    .get(&origin.input_ref)
                    .ok_or(Error("selected proposal lacks exact transport intent"))?;
                if facts.proposals.len() >= 1024 {
                    return Err(Error("native proposal fact bound exceeded"));
                }
                let seq = self
                    .writer
                    .frontier()
                    .checked_add(1)
                    .ok_or(Error("proposal sequence overflow"))?;
                let fact = native::ProposalFact {
                    fact_sequence: seq,
                    native_proposal_bytes,
                    selected_input_ref: origin.input_ref,
                    selection_sequence: seq,
                    first_receive_input_ref: origin.input_ref,
                    original_receive_tick_us: input.original_receive_tick_us,
                    original_receive_local_us: input.original_receive_local_us,
                    transport_binding_verified: input.transport_binding_verified,
                    decoded: true,
                    elected_leader_checked: origin.signature_state
                        == native::SignatureState::Pending,
                    signature_state: origin.signature_state,
                    verifying_parent_digest: None,
                    application_result_ref: None,
                    observed_slot_status: "UNVERIFIED",
                    conflicting_input_refs: vec![],
                };
                let token = self.writer.stage(
                    "PROPOSAL_FACT",
                    &record_json! {
                        fact_sequence: fact.fact_sequence,
                        native_proposal_bytes: hex(&fact.native_proposal_bytes),
                        selected_input_ref: hex(&fact.selected_input_ref),
                        selection_sequence: fact.selection_sequence,
                        first_receive_input_ref: hex(&fact.first_receive_input_ref),
                        original_receive_tick_us: fact.original_receive_tick_us,
                        original_receive_local_us: fact.original_receive_local_us,
                        transport_binding_verified: fact.transport_binding_verified,
                        decoded: fact.decoded,
                        elected_leader_checked: fact.elected_leader_checked,
                        signature_state: signature_state(fact.signature_state),
                        verifying_parent_digest: Option::<String>::None,
                        application_result_ref: fact.application_result_ref,
                        observed_slot_status: fact.observed_slot_status,
                        conflicting_input_refs: [String::new();0],
                    },
                )?;
                if token != seq {
                    return Err(Error("native selection sequence mismatch"));
                }
                facts.proposals.push(fact);
                Ok(token)
            }
            R::IngressSignature {
                input_ref,
                signature_state: state,
            } => {
                if !facts.accepted_inputs.contains_key(&input_ref) {
                    return Err(Error("unknown signed input reference"));
                }
                let token = self.writer.stage(
                    "INGRESS_SIGNATURE",
                    &record_json! {
                        input_ref: hex(&input_ref), signature_state: signature_state(state),
                    },
                )?;
                for fact in facts
                    .proposals
                    .iter_mut()
                    .filter(|fact| fact.selected_input_ref == input_ref)
                {
                    fact.signature_state = state;
                }
                Ok(token)
            }
            R::Claim(claim) => {
                if claim.epoch != self.epoch || facts.claims.len() >= 64 {
                    return Err(Error("native callback claim bound or epoch"));
                }
                let token = self.writer.stage("CALLBACK_CLAIM", &record_json!{
                    kind: operation_kind(claim.kind), epoch: claim.epoch, view: claim.view,
                    canonical_native_context_bytes: optional_hex(claim.context.as_deref()),
                    parent_view: claim.parent_view, parent_digest: optional_hex(claim.parent_digest.as_deref()),
                    requested_payload: optional_hex(claim.payload.as_deref()),
                })?;
                facts.claims.push_back((token, claim));
                Ok(token)
            }
            R::WorkScheduled {
                claim,
                start_tick_us,
                due_tick_us,
            } => {
                let op = facts
                    .operations
                    .get_mut(&claim)
                    .ok_or(Error("unclaimed native work schedule"))?;
                if op.phase != native::OperationPhase::Claimed
                    || op.helper_phase != native::HelperPhase::WaitInput
                    || start_tick_us >= due_tick_us
                {
                    return Err(Error(
                        "native work schedule conflicts with retained operation",
                    ));
                }
                let token=self.writer.stage("WORK_SCHEDULED",&record_json!{
                    operation_id:hex(&op.operation_id), claim:claim, start_tick_us:start_tick_us, due_tick_us:due_tick_us,
                })?;
                op.original_start_tick_us = Some(start_tick_us);
                op.original_due_tick_us = Some(due_tick_us);
                op.missing_dependencies.clear();
                op.helper_phase = native::HelperPhase::WorkScheduled;
                Ok(token)
            }
            R::WorkReady { claim, block_bytes } => {
                let op = facts
                    .operations
                    .get_mut(&claim)
                    .ok_or(Error("unclaimed native helper result"))?;
                if op.phase != native::OperationPhase::Claimed
                    || op.helper_phase != native::HelperPhase::WorkScheduled
                {
                    return Err(Error("native helper completion out of phase"));
                }
                let token=self.writer.stage("WORK_READY",&record_json!{
                    operation_id:hex(&op.operation_id), claim:claim, block_bytes:optional_hex(block_bytes.as_deref()),
                })?;
                op.helper_phase = native::HelperPhase::WorkReady;
                op.outcome_evidence_refs.push(token);
                facts.helper_results.insert(claim, block_bytes);
                Ok(token)
            }
            R::BodyPersisting { claim } => {
                if !facts
                    .helper_results
                    .get(&claim)
                    .is_some_and(Option::is_some)
                {
                    return Err(Error("body persistence lacks exact positive helper result"));
                }
                let op = facts
                    .operations
                    .get_mut(&claim)
                    .ok_or(Error("unclaimed native body persistence"))?;
                if op.phase != native::OperationPhase::Claimed
                    || op.helper_phase != native::HelperPhase::WorkReady
                {
                    return Err(Error("unexpected native body persistence phase"));
                }
                let token = self.writer.stage(
                    "BODY_PERSISTING",
                    &record_json! {
                        operation_id:hex(&op.operation_id), claim:claim,
                    },
                )?;
                op.helper_phase = native::HelperPhase::BodyPersisting;
                Ok(token)
            }
            R::BodyDurable {
                claim,
                digest,
                block_bytes,
            } => {
                if facts.helper_results.get(&claim).and_then(Option::as_ref) != Some(&block_bytes) {
                    return Err(Error("persisted body differs from helper result"));
                }
                let op = facts
                    .operations
                    .get_mut(&claim)
                    .ok_or(Error("unclaimed persisted native body"))?;
                if op.phase != native::OperationPhase::Claimed
                    || op.helper_phase != native::HelperPhase::BodyPersisting
                {
                    return Err(Error("persisted body observed outside persistence"));
                }
                let token=self.writer.stage("BODY_DURABLE",&record_json!{
                    operation_id:hex(&op.operation_id), claim:claim,digest:hex(&digest),block_bytes:hex(&block_bytes),
                })?;
                op.helper_phase = native::HelperPhase::ReadyToOffer;
                op.outcome_evidence_refs.push(token);
                Ok(token)
            }
            R::Offer {
                claim,
                outcome,
                evidence,
            } => {
                let op = facts
                    .operations
                    .get_mut(&claim)
                    .ok_or(Error("native offer without operation binding"))?;
                let ready = match outcome {
                    native::Outcome::Bool(true) | native::Outcome::Digest(_) => {
                        op.helper_phase == native::HelperPhase::ReadyToOffer
                    }
                    _ => op.helper_phase == native::HelperPhase::WorkReady,
                };
                if op.phase != native::OperationPhase::Claimed
                    || !ready
                    || op.outcome_evidence_refs != evidence
                {
                    return Err(Error("native offer lacks exact prerequisite evidence"));
                }
                let token=self.writer.stage("CALLBACK_OFFERED",&record_json!{
                    operation_id:hex(&op.operation_id), claim:claim,outcome:outcome_record(&outcome),
                    outcome_evidence_refs:evidence,
                })?;
                op.phase = native::OperationPhase::Offered;
                op.offered_sequence = Some(token);
                op.offered_outcome = Some(outcome);
                Ok(token)
            }
            R::Consume {
                claim,
                outcome,
                applied,
            } => {
                let op = facts
                    .operations
                    .get_mut(&claim)
                    .ok_or(Error("native response without operation binding"))?;
                if op.phase != native::OperationPhase::Offered
                    || op.offered_outcome.as_ref() != Some(&outcome)
                {
                    return Err(Error("native response differs from durable offer"));
                }
                let token = self.writer.stage("CALLBACK_CONSUMED", &record_json!{
                    operation_id: hex(&op.operation_id), claim: claim, outcome: outcome_record(&outcome), applied: applied,
                })?;
                op.phase = native::OperationPhase::Consumed;
                op.consumed_sequence = Some(token);
                Ok(token)
            }
            R::Cancel { claim, reason } => {
                let op = facts
                    .operations
                    .get_mut(&claim)
                    .ok_or(Error("native cancellation without operation binding"))?;
                if !matches!(
                    op.phase,
                    native::OperationPhase::Claimed | native::OperationPhase::Offered
                ) {
                    return Err(Error("native cancellation after retirement"));
                }
                let token = self.writer.stage(
                    "CALLBACK_CANCELLED",
                    &record_json! {
                        operation_id: hex(&op.operation_id), claim: claim, reason: reason,
                    },
                )?;
                op.phase = native::OperationPhase::Cancelled;
                op.cancelled_sequence = Some(token);
                op.cancellation_reason = Some(reason);
                Ok(token)
            }
            R::ActorPhase(phase) => {
                let token = self.writer.stage("ACTOR_PHASE", &record_json!{
                    pending_proposal_operation: phase.pending_proposal_operation,
                    pending_proposal_context: optional_hex(phase.pending_proposal_context.as_deref()),
                    proposal_phase: match phase.proposal_phase {
                        native::ProposalPhase::None => "NONE",
                        native::ProposalPhase::Preparing => "PREPARING",
                        native::ProposalPhase::Proposing => "PROPOSING",
                        native::ProposalPhase::Waiting => "WAITING",
                        native::ProposalPhase::Closed => "CLOSED",
                    },
                    pending_verify_operation: phase.pending_verify_operation,
                    verifying_proposal_bytes: optional_hex(phase.verifying_proposal_bytes.as_deref()),
                    verifying_parent_digest: optional_hex(phase.verifying_parent_digest.as_deref()),
                    outstanding_certify_operations: &phase.outstanding_certify_operations,
                    committed_relay_refs: &phase.committed_relay_refs,
                })?;
                facts.actor_phase = Some((token, phase));
                Ok(token)
            }
            R::Timing(snapshot) => {
                let rounds = snapshot.rounds.iter().map(|round| record_json!{
                    view: round.view,
                    entered_at_local_us: round.entered_at_local_us,
                    proposed_at_local_us: round.proposed_at_local_us,
                    comparison_decision_phase: match round.comparison_decision_phase {
                        native::DecisionPhase::Pending => "PENDING", native::DecisionPhase::Nullify => "NULLIFY",
                        native::DecisionPhase::Finalize => "FINALIZE",
                    },
                    leader_deadline_local_us: round.leader_deadline_local_us,
                    certification_deadline_local_us: round.certification_deadline_local_us,
                    latch_time_local_us: round.latch_time_local_us,
                    latch_reason: round.latch_reason,
                    latch_cause_ref: round.latch_cause_ref.map(|id|hex(&id)),
                    retry_deadline_local_us: round.retry_deadline_local_us,
                    retry_arm_local_us: round.retry_arm_local_us,
                    stall_deadline_local_us: round.stall_deadline_local_us,
                }).collect::<Vec<_>>();
                let token = self.writer.stage("TIMING_SNAPSHOT", &record_json!{
                    current_view: snapshot.current_view, last_finalized_view: snapshot.last_finalized_view,
                    stall_anchor: snapshot.stall_anchor, native_state_projection_hash: hex(&snapshot.native_state_projection_hash),
                    rounds: rounds,
                })?;
                facts.timing = Some((token, snapshot));
                Ok(token)
            }
            R::Preparation {
                positions,
                prospective_artifact,
            } => {
                // This record is the native side of a preparation. The root
                // must still bind signer, RNG and other controller frontiers.
                let positions = positions.iter().map(|pos| record_json!{
                    section: pos.section, offset: pos.offset, native_artifact_bytes: hex(&pos.frame_bytes),
                }).collect::<Vec<_>>();
                self.writer.stage(
                    "NATIVE_PREPARATION_STATE",
                    &record_json! {
                        actor_phase_ref: facts.actor_phase.as_ref().map(|(id,_)|*id),
                        timing_snapshot_ref: facts.timing.as_ref().map(|(id,_)|*id),
                        native_positions: positions,
                        prospective_artifact: optional_hex(prospective_artifact.as_deref()),
                    },
                )
            }
            R::ResolverStarted => self.writer.stage("NATIVE_RESOLVER_STARTED", &()),
        }
    }
}

impl native::RecoveryStore for ActorStore {
    fn stage(
        &self,
        record: native::RecoveryRecord,
    ) -> std::result::Result<u64, native::RecoveryError> {
        self.stage_actor(record)
            .map_err(|error| native::RecoveryError(error.0))
    }
    fn bind_callback(
        &self,
        binding: native::CallbackBinding,
    ) -> std::result::Result<u64, native::RecoveryError> {
        let mut facts = self.facts.lock().unwrap();
        let (claim, expected) = facts.claims.front().ok_or(native::RecoveryError(
            "unclaimed native application dispatch",
        ))?;
        if expected != &binding.claim
            || facts.operations.len() >= 1024
            || facts
                .operations
                .values()
                .any(|op| op.operation_id == binding.operation_id)
        {
            return Err(native::RecoveryError(
                "application dispatch differs from native claim",
            ));
        }
        let claim = *claim;
        let helper_phase = match (binding.original_start_tick_us, binding.original_due_tick_us) {
            (Some(start), Some(due)) if start < due && binding.missing_dependencies.is_empty() => {
                native::HelperPhase::WorkScheduled
            }
            (None, None) if !binding.missing_dependencies.is_empty() => {
                native::HelperPhase::WaitInput
            }
            _ => {
                return Err(native::RecoveryError(
                    "application dispatch has incomplete work schedule",
                ));
            }
        };
        let op = native::OperationRecord {
            operation_id: binding.operation_id,
            original_work_binding: binding.original_work_binding,
            kind: binding.claim.kind,
            canonical_native_context_bytes: binding.claim.context,
            epoch: binding.claim.epoch,
            view: binding.claim.view,
            parent_view: binding.claim.parent_view,
            parent_digest: binding.claim.parent_digest,
            requested_payload: binding.claim.payload,
            phase: native::OperationPhase::Claimed,
            helper_phase,
            claimed_sequence: claim,
            original_start_tick_us: binding.original_start_tick_us,
            original_due_tick_us: binding.original_due_tick_us,
            missing_dependencies: binding.missing_dependencies,
            offered_outcome: None,
            outcome_evidence_refs: vec![],
            offered_sequence: None,
            consumed_sequence: None,
            cancelled_sequence: None,
            cancellation_reason: None,
            reconnect_generation: 0,
        };
        self.writer.stage("OPERATION_RECORD", &record_json!{
            operation_id: hex(&op.operation_id), original_work_binding: hex(&op.original_work_binding),
            kind: operation_kind(op.kind), canonical_native_context_bytes: optional_hex(op.canonical_native_context_bytes.as_deref()),
            epoch: op.epoch, view: op.view, parent_view: op.parent_view, parent_digest: optional_hex(op.parent_digest.as_deref()),
            requested_payload: optional_hex(op.requested_payload.as_deref()), phase: "CLAIMED",
            helper_phase: if helper_phase == native::HelperPhase::WaitInput {"WAIT_INPUT"} else {"WORK_SCHEDULED"},
            claimed_sequence: claim,
            original_start_tick_us: op.original_start_tick_us, original_due_tick_us: op.original_due_tick_us,
            missing_dependencies: op.missing_dependencies.iter().map(|id|hex(id)).collect::<Vec<_>>(),
            offered_outcome: Option::<String>::None, outcome_evidence_refs: &op.outcome_evidence_refs,
            offered_sequence: op.offered_sequence, consumed_sequence: op.consumed_sequence,
            cancelled_sequence: op.cancelled_sequence, cancellation_reason: op.cancellation_reason,
            reconnect_generation: op.reconnect_generation,
        }).map_err(|error|native::RecoveryError(error.0))?;
        facts.claims.pop_front();
        facts.operations.insert(claim, op);
        Ok(claim)
    }
    fn flush(
        &self,
        through: u64,
    ) -> BoxFuture<'_, std::result::Result<RecoveryReceipt, native::RecoveryError>> {
        Box::pin(async move {
            self.writer
                .flush_actor(&self.authority, through)
                .await
                .map_err(|error| native::RecoveryError(error.0))
        })
    }
    fn load(
        &self,
        request: native::RecoveryLoad,
    ) -> BoxFuture<'_, std::result::Result<native::RecoveryBundle, native::RecoveryError>> {
        Box::pin(async move {
            if request.epoch != self.epoch
                || (request.actor == native::ActorId::Voter
                    && request.native_boundary != self.restored_boundary)
            {
                return Err(native::RecoveryError(
                    "native journal boundary differs from validated recovery",
                ));
            }
            Ok(self.restored.clone())
        })
    }
    fn checkpoint(
        &self,
        _: native::RecoveryCut,
    ) -> BoxFuture<'_, std::result::Result<RecoveryReceipt, native::RecoveryError>> {
        Box::pin(async {
            Err(native::RecoveryError(
                "native Cut commit requires the root controller",
            ))
        })
    }
    fn claim_ingress(
        &self,
        actor: native::ActorId,
        channel: u32,
        key: Vec<u8>,
        bytes: Vec<u8>,
    ) -> std::result::Result<Id, native::RecoveryError> {
        let mut facts = self.facts.lock().unwrap();
        let queue = facts.consumed.entry(channel).or_default();
        if actor != native::ActorId::Batcher
            || !matches!(channel, 1 | 2)
            || !queue
                .front()
                .is_some_and(|(_, expected_key, expected_bytes)| {
                    expected_key == &key && expected_bytes == &bytes
                })
        {
            return Err(native::RecoveryError("native ingress custody mismatch"));
        }
        Ok(queue.pop_front().expect("matched native ingress packet").0)
    }
}

fn operation_kind(value: native::OperationKind) -> &'static str {
    match value {
        native::OperationKind::Prepare => "PREPARE",
        native::OperationKind::Propose => "PROPOSE",
        native::OperationKind::Verify => "VERIFY",
        native::OperationKind::Certify => "CERTIFY",
    }
}
fn signature_state(value: native::SignatureState) -> &'static str {
    match value {
        native::SignatureState::NotApplicable => "NOT_APPLICABLE",
        native::SignatureState::Pending => "PENDING",
        native::SignatureState::Verified => "VERIFIED",
        native::SignatureState::Rejected => "REJECTED",
    }
}
fn optional_hex(value: Option<&[u8]>) -> Option<String> {
    value.map(hex)
}
#[derive(Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "SCREAMING_SNAKE_CASE")]
enum PlainOutcome {
    Digest(String),
    Bool(bool),
    HandoffWait,
    ResponseClosed,
}
fn outcome_record(value: &native::Outcome) -> PlainOutcome {
    match value {
        native::Outcome::Digest(bytes) => PlainOutcome::Digest(hex(bytes)),
        native::Outcome::Bool(value) => PlainOutcome::Bool(*value),
        native::Outcome::HandoffWait => PlainOutcome::HandoffWait,
        native::Outcome::ResponseClosed => PlainOutcome::ResponseClosed,
    }
}

type Json = serde_json::Value;
fn field<T: serde::de::DeserializeOwned>(value: &Json, key: &str) -> Result<T> {
    serde_json::from_value(
        value
            .get(key)
            .ok_or(Error("missing native recovery fact field"))?
            .clone(),
    )
    .map_err(|_| Error("invalid native recovery fact type"))
}
fn bytes(text: &str) -> Result<Vec<u8>> {
    if !text.len().is_multiple_of(2)
        || text
            .bytes()
            .any(|b| !b.is_ascii_digit() && !(b'a'..=b'f').contains(&b))
    {
        return Err(Error("noncanonical recovery hex bytes"));
    }
    (0..text.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&text[i..i + 2], 16)
                .map_err(|_| Error("invalid native recovery hex"))
        })
        .collect()
}
pub(crate) fn fixed(text: &str) -> Result<Id> {
    bytes(text)?
        .try_into()
        .map_err(|_| Error("invalid native recovery ID length"))
}
fn byte_field(value: &Json, key: &str) -> Result<Vec<u8>> {
    bytes(&field::<String>(value, key)?)
}
fn optional_bytes(value: &Json, key: &str) -> Result<Option<Vec<u8>>> {
    field::<Option<String>>(value, key)?
        .as_deref()
        .map(bytes)
        .transpose()
}
fn read_json(record: &crate::native_recovery_ledger::LedgerRecord) -> Result<Json> {
    serde_json::from_slice(&record.data).map_err(|_| Error("malformed recovery fact payload"))
}

fn read_timing(data: &Json) -> Result<native::TimingSnapshot> {
    let rounds = field::<Vec<Json>>(data, "rounds")?
        .iter()
        .map(|round| {
            Ok(native::RoundTiming {
                view: field(round, "view")?,
                entered_at_local_us: field(round, "entered_at_local_us")?,
                proposed_at_local_us: field(round, "proposed_at_local_us")?,
                comparison_decision_phase: match field::<String>(
                    round,
                    "comparison_decision_phase",
                )?
                .as_str()
                {
                    "PENDING" => native::DecisionPhase::Pending,
                    "NULLIFY" => native::DecisionPhase::Nullify,
                    "FINALIZE" => native::DecisionPhase::Finalize,
                    _ => return Err(Error("unknown native timing decision phase")),
                },
                leader_deadline_local_us: field(round, "leader_deadline_local_us")?,
                certification_deadline_local_us: field(round, "certification_deadline_local_us")?,
                latch_time_local_us: field(round, "latch_time_local_us")?,
                latch_reason: match field::<Option<String>>(round, "latch_reason")? {
                    None => None,
                    Some(_) => {
                        return Err(Error(
                            "pending Verify restore unexpectedly has a latched timeout",
                        ));
                    }
                },
                latch_cause_ref: field::<Option<String>>(round, "latch_cause_ref")?
                    .as_deref()
                    .map(fixed)
                    .transpose()?,
                retry_deadline_local_us: field(round, "retry_deadline_local_us")?,
                retry_arm_local_us: field(round, "retry_arm_local_us")?,
                stall_deadline_local_us: field(round, "stall_deadline_local_us")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    if rounds.len() > 8 || rounds.windows(2).any(|pair| pair[0].view >= pair[1].view) {
        return Err(Error("invalid retained timing rounds"));
    }
    Ok(native::TimingSnapshot {
        current_view: field(data, "current_view")?,
        last_finalized_view: field(data, "last_finalized_view")?,
        stall_anchor: field(data, "stall_anchor")?,
        native_state_projection_hash: fixed(&field::<String>(
            data,
            "native_state_projection_hash",
        )?)?,
        rounds,
    })
}

/// Reconstruct the pending-Verify state from a root-pinned frontier.
pub(crate) fn load_pending_verify_fragment(
    ledger: &RecoveryLedger,
    through: RecoveryReceipt,
    cut: Id,
    config: &crate::native_config::Resolved,
    worker: u32,
) -> Result<native::RecoveryBundle> {
    let selected = ledger
        .records()
        .iter()
        .position(|r| r.receipt == through)
        .ok_or(Error("pending Verify frontier is outside anchored ledger"))?;
    let records = &ledger.records()[..=selected];
    let latest = |kind| {
        records
            .iter()
            .rev()
            .find(|r| r.header.kind == kind)
            .ok_or(Error("missing required pending Verify recovery fact"))
    };
    let snapshot = read_timing(&read_json(latest("TIMING_SNAPSHOT")?)?)?;
    let phase = read_json(latest("ACTOR_PHASE")?)?;
    let claim = field::<Option<u64>>(&phase, "pending_verify_operation")?
        .ok_or(Error("missing restored Verify operation claim"))?;
    let native_bytes = optional_bytes(&phase, "verifying_proposal_bytes")?
        .ok_or(Error("missing restored Verify proposal binding"))?;
    let parent = optional_bytes(&phase, "verifying_parent_digest")?
        .ok_or(Error("missing restored Verify parent binding"))?;
    if field::<Option<u64>>(&phase, "pending_proposal_operation")?.is_some()
        || optional_bytes(&phase, "pending_proposal_context")?.is_some()
        || field::<String>(&phase, "proposal_phase")? != "NONE"
        || !field::<Vec<u64>>(&phase, "outstanding_certify_operations")?.is_empty()
        || !field::<Vec<u64>>(&phase, "committed_relay_refs")?.is_empty()
    {
        return Err(Error(
            "pending Verify loader cannot restore additional actor work",
        ));
    }
    let operation_record = latest("OPERATION_RECORD")?;
    let op = read_json(operation_record)?;
    if field::<String>(&op, "kind")? != "VERIFY"
        || field::<String>(&op, "phase")? != "CLAIMED"
        || field::<String>(&op, "helper_phase")? != "WAIT_INPUT"
        || field::<u64>(&op, "claimed_sequence")? != claim
        || field::<Option<u64>>(&op, "original_start_tick_us")?.is_some()
        || field::<Option<u64>>(&op, "original_due_tick_us")?.is_some()
        || field::<Option<Json>>(&op, "offered_outcome")?.is_some()
        || field::<Option<u64>>(&op, "consumed_sequence")?.is_some()
        || field::<Option<u64>>(&op, "cancelled_sequence")?.is_some()
        || field::<Option<u64>>(&op, "offered_sequence")?.is_some()
        || field::<Option<String>>(&op, "cancellation_reason")?.is_some()
        || !field::<Vec<u64>>(&op, "outcome_evidence_refs")?.is_empty()
        || records
            .iter()
            .filter(|r| r.header.kind == "OPERATION_RECORD")
            .count()
            != 1
        || records.iter().any(|r| {
            r.header.sequence > operation_record.header.sequence
                && matches!(
                    r.header.kind.as_str(),
                    "WORK_SCHEDULED"
                        | "CALLBACK_CANCELLED"
                        | "CALLBACK_OFFERED"
                        | "CALLBACK_CONSUMED"
                )
        })
    {
        return Err(Error(
            "pending Verify phase incompatible with retained helper history",
        ));
    }
    let operation = native::OperationRecord {
        operation_id: fixed(&field::<String>(&op, "operation_id")?)?,
        original_work_binding: byte_field(&op, "original_work_binding")?,
        kind: native::OperationKind::Verify,
        canonical_native_context_bytes: optional_bytes(&op, "canonical_native_context_bytes")?,
        epoch: field(&op, "epoch")?,
        view: field(&op, "view")?,
        parent_view: field(&op, "parent_view")?,
        parent_digest: optional_bytes(&op, "parent_digest")?,
        requested_payload: optional_bytes(&op, "requested_payload")?,
        phase: native::OperationPhase::Claimed,
        helper_phase: native::HelperPhase::WaitInput,
        claimed_sequence: claim,
        original_start_tick_us: None,
        original_due_tick_us: None,
        missing_dependencies: field::<Vec<String>>(&op, "missing_dependencies")?
            .iter()
            .map(|hex| bytes(hex))
            .collect::<Result<_>>()?,
        offered_outcome: None,
        outcome_evidence_refs: vec![],
        offered_sequence: None,
        consumed_sequence: None,
        cancelled_sequence: None,
        cancellation_reason: None,
        reconnect_generation: field(&op, "reconnect_generation")?,
    };
    let actual_claim = records
        .iter()
        .find(|r| r.header.sequence == claim && r.header.kind == "CALLBACK_CLAIM")
        .ok_or(Error(
            "restored Verify claim does not exist in protected history",
        ))?;
    let claim_data = read_json(actual_claim)?;
    if field::<String>(&claim_data, "kind")? != "VERIFY"
        || field::<u64>(&claim_data, "epoch")? != config.base.epoch
        || operation.epoch != config.base.epoch
        || field::<u64>(&claim_data, "view")? != operation.view
        || field::<Option<u64>>(&claim_data, "parent_view")? != operation.parent_view
        || optional_bytes(&claim_data, "parent_digest")? != Some(parent.clone())
        || operation.parent_digest.as_ref() != Some(&parent)
        || optional_bytes(&claim_data, "requested_payload")? != operation.requested_payload
        || optional_bytes(&claim_data, "canonical_native_context_bytes")?
            != operation.canonical_native_context_bytes
    {
        return Err(Error(
            "persisted Verify dispatch differs from exact native claim",
        ));
    }

    let fact = read_pending_proposal(records, &native_bytes, &operation, config, worker)?;
    Ok(native::RecoveryBundle {
        cut,
        timing: Some(snapshot),
        proposals: vec![fact],
        operations: vec![operation],
        actor_phase: Some(native::ActorPhase {
            pending_proposal_operation: None,
            pending_proposal_context: None,
            proposal_phase: native::ProposalPhase::None,
            pending_verify_operation: Some(claim),
            verifying_proposal_bytes: Some(native_bytes),
            verifying_parent_digest: Some(parent),
            outstanding_certify_operations: vec![],
            committed_relay_refs: vec![],
        }),
    })
}

fn read_pending_proposal(
    records: &[crate::native_recovery_ledger::LedgerRecord],
    native_bytes: &[u8],
    operation: &native::OperationRecord,
    config: &crate::native_config::Resolved,
    worker: u32,
) -> Result<native::ProposalFact> {
    use crate::native_wire::{Channel, Grant, input::Kind};
    use commonware_codec::Encode as _;
    use commonware_consensus::simplex::types::{Attributable, Vote};
    use prost::Message as _;
    let mut facts = records.iter().filter(|r| r.header.kind == "PROPOSAL_FACT");
    let entry = facts
        .next()
        .ok_or(Error("missing voter-applied proposal fact"))?;
    let data = read_json(entry)?;
    let selected = field::<String>(&data, "selected_input_ref")?;
    let id = fixed(&selected)?;
    if facts.next().is_some()
        || field::<u64>(&data, "fact_sequence")? != entry.header.sequence
        || field::<u64>(&data, "selection_sequence")? != entry.header.sequence
        || field::<String>(&data, "first_receive_input_ref")? != selected
        || entry.header.sequence >= operation.claimed_sequence
        || byte_field(&data, "native_proposal_bytes")? != native_bytes
        || !field::<bool>(&data, "decoded")?
        || !field::<bool>(&data, "elected_leader_checked")?
        || !field::<bool>(&data, "transport_binding_verified")?
        || field::<String>(&data, "signature_state")? != "PENDING"
        || field::<String>(&data, "observed_slot_status")? != "UNVERIFIED"
        || field::<Option<u64>>(&data, "application_result_ref")?.is_some()
        || !field::<Vec<String>>(&data, "conflicting_input_refs")?.is_empty()
    {
        return Err(Error(
            "pending Verify requires one exact voter-applied optimistic proposal",
        ));
    }
    let intent_record = records
        .iter()
        .find(|r| {
            r.header.kind == "GRANT_INTENT"
                && r.header.grant_intent_ref.as_deref() == Some(&selected)
        })
        .ok_or(Error("missing write-ahead bytes for selected proposal"))?;
    let intent = read_json(intent_record)?;
    let receipt = field::<Json>(&intent, "predecessor_root_receipt")?;
    let predecessor = RecoveryReceipt {
        sequence: field(&receipt, "sequence")?,
        hash: fixed(&field::<String>(&receipt, "hash")?)?,
    };
    let original_local = field(&intent, "original_receive_local_us")?;
    let original_grant = byte_field(&intent, "canonical_grant_bytes")?;
    let grant = Grant::decode(original_grant.as_slice())
        .map_err(|_| Error("invalid selected input grant"))?;
    let (check_id, check) = GrantIntent::from_grant(
        config,
        worker,
        field(&intent, "original_incarnation")?,
        field(&intent, "input_sequence")?,
        &grant,
        predecessor,
        fixed(&field::<String>(&intent, "predecessor_cut_hash")?)?,
        original_local,
    )?;
    if check_id != id
        || serde_json::to_vec(&check).map_err(|_| Error("canonical selected input"))?
            != intent_record.data
        || original_local.is_none()
        || field::<Option<i64>>(&data, "original_receive_local_us")? != original_local
        || field::<u64>(&data, "original_receive_tick_us")? != grant.tick_us
        || intent_record.header.sequence >= entry.header.sequence
    {
        return Err(Error(
            "selected proposal has changed original custody or receipt",
        ));
    }
    let Some(Kind::Deliver(envelope)) = grant.input.and_then(|input| input.kind) else {
        return Err(Error(
            "selected proposal original input is not native delivery",
        ));
    };
    if !check.decoded
        || !check.transport_binding_verified
        || envelope.channel != Channel::Vote as i32
        || operation.view != 1
        || envelope.src_member_id != 0
    {
        return Err(Error(
            "selected proposal lacks its native view-one leader binding",
        ));
    }
    let Vote::Notarize(vote) = crate::native_network::decode_vote(&envelope.payload_bytes)? else {
        return Err(Error("selected proposal input was not leader Notarize"));
    };
    if vote.signer().get() != config.members[0].signer_index
        || vote.proposal.encode().as_ref() != native_bytes
        || operation.requested_payload.as_deref() != Some(vote.proposal.payload.0.as_slice())
    {
        return Err(Error(
            "selected proposal no longer matches original native bytes",
        ));
    }
    Ok(native::ProposalFact {
        fact_sequence: entry.header.sequence,
        native_proposal_bytes: native_bytes.to_vec(),
        selected_input_ref: id,
        selection_sequence: entry.header.sequence,
        first_receive_input_ref: id,
        original_receive_tick_us: grant.tick_us,
        original_receive_local_us: original_local,
        transport_binding_verified: true,
        decoded: true,
        elected_leader_checked: true,
        signature_state: native::SignatureState::Pending,
        verifying_parent_digest: operation.parent_digest.clone(),
        application_result_ref: None,
        observed_slot_status: "UNVERIFIED",
        conflicting_input_refs: vec![],
    })
}

/// Preserve all ordinary output bytes before the final cut is built. Cut and
/// readiness effects are appended only after that cut is available.
pub(crate) fn stage_output_intents(
    writer: &LedgerWriter,
    grant: &crate::native_wire::Grant,
    ack: &crate::native_wire::Ack,
) -> Result<u64> {
    use crate::native_wire::effect::Kind;
    use prost::Message as _;
    let (id, input) = writer
        .state
        .lock()
        .unwrap()
        .grant
        .ok_or(Error("output intent lacks grant custody"))?;
    if grant.logical_step_id != id
        || ack.logical_step_id != id
        || ack.binding != grant.binding
        || ack.grant_hash != grant.grant_hash
        || ack.event_id != grant.event_id
        || ack.recovery_receipt.is_some()
        || !ack.durable_step_hash.is_empty()
        || !ack.logical_ack_hash.is_empty()
    {
        return Err(Error(
            "output intent differs from authorized root transaction",
        ));
    }
    let mut effects = Vec::with_capacity(ack.effects.len());
    for (index, effect) in ack.effects.iter().enumerate() {
        if matches!(
            effect.kind,
            None | Some(Kind::RecoveryReady(_) | Kind::RecoveryCut(_))
        ) {
            return Err(Error(
                "output intent may contain only ordinary effect kinds",
            ));
        }
        effects.push(record_json! {
            effect_index:index,
            canonical_effect_bytes: hex(&effect.encode_to_vec()),
        });
    }
    writer.stage(
        "OUTPUT_INTENTS",
        &record_json! {
            schema:1, original_input_ref:hex(&input), original_step_id:id,
            original_grant_hash:hex(&grant.grant_hash), effects:effects,
        },
    )
}

const STARTUP_ACTORS: [native::ActorId; 6] = [
    native::ActorId::Voter,
    native::ActorId::Batcher,
    native::ActorId::Resolver,
    native::ActorId::Application,
    native::ActorId::Archive,
    native::ActorId::Driver,
];
fn actor_tag(actor: native::ActorId) -> &'static str {
    match actor {
        native::ActorId::Voter => "VOTER",
        native::ActorId::Batcher => "BATCHER",
        native::ActorId::Resolver => "RESOLVER",
        native::ActorId::Application => "APPLICATION",
        native::ActorId::Archive => "ARCHIVE",
        native::ActorId::Driver => "DRIVER",
    }
}
#[derive(Default)]
struct FenceState {
    ready: BTreeMap<native::ActorId, Id>,
    waiters: Vec<std::task::Waker>,
    released: bool,
}

/// Root-owned release authority. Native jobs receive only the RecoveryStartup
/// trait, so they can register readiness and park but cannot open the fence.
pub(crate) struct StartupFence {
    cut: Id,
    state: Mutex<FenceState>,
}
impl StartupFence {
    pub fn new(cut: Id) -> Self {
        Self {
            cut,
            state: Mutex::default(),
        }
    }
    fn canonical_release(&self) -> Result<Vec<u8>> {
        let state = self.state.lock().unwrap();
        if state.released
            || !STARTUP_ACTORS
                .iter()
                .all(|actor| state.ready.contains_key(actor))
        {
            return Err(Error(
                "startup release requires all six matching actor readies",
            ));
        }
        let readiness = STARTUP_ACTORS
            .into_iter()
            .map(|actor| {
                let tag = actor_tag(actor);
                let hash = state.ready[&actor];
                let ready = hash_parts(
                    b"I1b/recovery-ready/v1",
                    &[&self.cut, tag.as_bytes(), &hash],
                );
                record_json! { actor:tag, state_hash:hex(&hash), readiness_hash:hex(&ready) }
            })
            .collect::<Vec<_>>();
        serde_json::to_vec(&record_json! {
            cut_hash:hex(&self.cut), readiness:readiness,
        })
        .map_err(|_| Error("encode startup fence"))
    }
    pub fn stage_release(&self, writer: &LedgerWriter) -> Result<u64> {
        writer.stage_canonical("STARTUP_RELEASED", self.canonical_release()?)
    }
    pub fn release(&self, writer: &LedgerWriter, receipt: RecoveryReceipt) -> Result<()> {
        let release = self.canonical_release()?;
        let writer_state = writer.state.lock().unwrap();
        let matches = writer_state.ledger.records().iter().any(|record| {
            record.receipt == receipt
                && record.header.kind == "STARTUP_RELEASED"
                && record.data == release
        });
        if !matches {
            return Err(Error("startup release record is not durable"));
        }
        drop(writer_state);
        let waiters = {
            let mut state = self.state.lock().unwrap();
            if state.released {
                return Err(Error("recovery startup already released"));
            }
            state.released = true;
            std::mem::take(&mut state.waiters)
        };
        for waiter in waiters {
            waiter.wake();
        }
        Ok(())
    }
}
impl native::RecoveryStartup for StartupFence {
    fn ready(
        &self,
        actor: native::ActorId,
        cut: Id,
        state_hash: Id,
    ) -> std::result::Result<(), native::RecoveryError> {
        let mut state = self.state.lock().unwrap();
        if state.released
            || cut != self.cut
            || state_hash == [0; 32]
            || state.ready.contains_key(&actor)
        {
            return Err(native::RecoveryError(
                "invalid or duplicate recovery readiness",
            ));
        }
        state.ready.insert(actor, state_hash);
        Ok(())
    }
    fn released(&self) -> BoxFuture<'_, std::result::Result<(), native::RecoveryError>> {
        Box::pin(std::future::poll_fn(|context| {
            let mut state = self.state.lock().unwrap();
            if state.released {
                return std::task::Poll::Ready(Ok(()));
            }
            if !state
                .waiters
                .iter()
                .any(|waker| waker.will_wake(context.waker()))
            {
                state.waiters.push(context.waker().clone());
            }
            std::task::Poll::Pending
        }))
    }
}

pub(crate) struct EmptyCutContext<'a> {
    pub config: &'a crate::native_config::Resolved,
    pub worker: u32,
    pub generation: u64,
    pub sequence: u64,
    pub previous: Id,
    pub work_generation: u64,
    pub active: bool,
    pub terminal: bool,
}
pub(crate) async fn checkpoint_empty_trajectory(
    writer: &LedgerWriter,
    driver: &mut ControlledDriver,
    context: EmptyCutContext<'_>,
    grant: &crate::native_wire::Grant,
    ack: &crate::native_wire::Ack,
) -> Result<(Id, RecoveryReceipt)> {
    use crate::LedgerEntry;
    let ctx = context;
    let cid = ctx.config.cid;
    let coords = DriverGrant {
        id: grant.logical_step_id,
        tick_us: grant.tick_us,
    };
    let witness = driver
        .witness_head()
        .ok_or(Error("required selected witness receipt absent"))?;
    if driver.witness_records().is_none_or(|records| {
        records.iter().flat_map(|r| &r.entries).any(|entry| {
            !matches!(
                entry,
                LedgerEntry::Policy(_)
                    | LedgerEntry::GenerationStarted(_)
                    | LedgerEntry::Recovery(_)
            )
        })
    }) || ack
        .effects
        .iter()
        .any(|effect| matches!(effect.kind, Some(crate::native_wire::effect::Kind::Sign(_))))
    {
        return Err(Error(
            "empty-journal checkpoint contains native signing or committed bytes",
        ));
    }
    let clock = driver.recovery_clock_checkpoint(cid)?;
    let rngs = driver
        .capture_role_rngs(cid)?
        .into_iter()
        .map(|(role, cursor)| {
            record_json! {
                role_id:hex(&role), seed:hex(&cursor.seed),stream:cursor.stream,
                word_position_hi:cursor.word_position_hi,word_position_lo:cursor.word_position_lo,
            }
        })
        .collect::<Vec<_>>();
    let (last_timer, timers) = driver.recovery_timers(cid)?;
    let timers = timers
        .into_iter()
        .map(|timer| {
            record_json! {
                id:timer.id,cid:hex(&timer.cid),local_deadline_us:timer.deadline_us,
            }
        })
        .collect::<Vec<_>>();
    let archive=writer.stage("ARCHIVE_MANIFEST",&record_json!{
        blocks:[();0],durable_body_receipts:[();0],proofs:[();0],native_parents:[();0],
        body_requests:[();0],body_retry_deadlines:[();0],
        delivered_digest:hex(&ctx.config.genesis),delivered_height:0,emitted_delivery_ids:[();0],
        work_generation:ctx.work_generation,packet_attempt_counters:[();0],blocked_members:[();0],
    })?;
    let lifecycle = writer.stage(
        "LIFECYCLE",
        &record_json! {
            cid:hex(&cid),local_stop_us:ctx.config.base.stop,
            state:if ctx.terminal {"TERMINATED"} else if ctx.active {"ACTIVE"} else {"NOT_STARTED"},
        },
    )?;
    let intents = stage_output_intents(writer, grant, ack)?;
    let frontier = writer.flush_root(driver, coords, cid, intents).await?;
    let (input, proposal, operation, phase, timing) = {
        let state = writer.state.lock().unwrap();
        let index = |kind| {
            state
                .ledger
                .records()
                .iter()
                .rev()
                .find(|r| r.header.kind == kind)
                .map(|r| r.receipt.sequence)
        };
        (
            index("GRANT_INTENT"),
            index("PROPOSAL_FACT"),
            index("OPERATION_RECORD"),
            index("ACTOR_PHASE"),
            index("TIMING_SNAPSHOT"),
        )
    };
    let boundary = NativeBoundary {
        generation: ctx.generation,
        sections: vec![],
        witness_receipt: BoundaryReceipt {
            sequence: witness.sequence,
            hash: hex(&witness.hash),
        },
        barrier_ids: vec![],
        eligible_payload_ids: vec![],
        submission_ids: vec![],
    };
    let payload=serde_json::to_vec(&record_json!{
        schema:1,cut_sequence:ctx.sequence,previous_cut_hash:hex(&ctx.previous),
        run:hex(&ctx.config.run_id),config:hex(&ctx.config.config_hash),worker:ctx.worker,cid:hex(&cid),
        epoch:ctx.config.base.epoch,profile_hash:hex(&ctx.config.config_hash),
        grant_intent_frontier: input,auxiliary_frontier:ReceiptRecord::from(frontier),
        proposal_frontier:proposal,operation_frontier:operation,signer_cache_frontier:0,
        native_journal_boundary:boundary,witness_receipt:BoundaryReceipt{sequence:witness.sequence,hash:hex(&witness.hash)},
        archive_manifest_ref:archive,actor_phase_ref:phase,timing_snapshot_ref:timing,lifecycle_ref:lifecycle,
        role_rng_checkpoints:rngs,runtime_timer_checkpoint:record_json!{last_issued_timer_id:last_timer,timers:timers},
        quota_checkpoint:[();0],clock_checkpoint:clock,output_intents_ref:intents,
        intended_root_step_sequence:ctx.sequence,
    }).map_err(|_|Error("encode coherent empty recovery cut"))?;
    let hash = hash_parts(b"I1b/recovery-cut/v1", &[&payload]);
    let mut complete = payload;
    complete.pop();
    complete.extend_from_slice(format!(",\"cut_hash\":\"{}\"}}", hex(&hash)).as_bytes());
    let token = writer.stage_canonical("RECOVERY_CUT", complete)?;
    let receipt = writer.flush_root(driver, coords, cid, token).await?;
    Ok((hash, receipt))
}

pub(crate) fn wire_receipt(value: RecoveryReceipt) -> crate::native_wire::Receipt {
    crate::native_wire::Receipt {
        sequence: value.sequence,
        hash: value.hash.to_vec(),
    }
}
pub(crate) fn read_wire_receipt(
    value: Option<&crate::native_wire::Receipt>,
) -> Result<RecoveryReceipt> {
    let value = value.ok_or(Error("missing trusted recovery receipt"))?;
    if value.sequence == 0 {
        return Err(Error("zero trusted recovery sequence"));
    }
    Ok(RecoveryReceipt {
        sequence: value.sequence,
        hash: value
            .hash
            .as_slice()
            .try_into()
            .map_err(|_| Error("invalid trusted recovery hash"))?,
    })
}

pub(crate) struct ResumeCut {
    pub bundle: native::RecoveryBundle,
    pub clock: crate::clock::ClockCheckpoint,
    pub rngs: Vec<(Id, crate::RngCheckpoint)>,
    pub last_timer: u64,
    pub work_generation: u64,
    pub input_sequence: u64,
}
pub(crate) fn validate_resume_empty_cut(
    ledger: &RecoveryLedger,
    config: &crate::native_config::Resolved,
    worker: u32,
    floor: &crate::native_wire::TrustedRecoveryFloor,
    root: &crate::native_step_journal::StepJournal,
) -> Result<ResumeCut> {
    let minimum = read_wire_receipt(floor.auxiliary_receipt.as_ref())?;
    let root_floor = read_wire_receipt(floor.root_receipt.as_ref())?;
    let cut: Id = floor
        .cut_hash
        .as_slice()
        .try_into()
        .map_err(|_| Error("invalid trusted cut hash"))?;
    if floor.run_id != config.run_id
        || floor.config_hash != config.config_hash
        || floor.worker_id != worker
        || floor.cid != config.cid
        || floor.epoch != config.base.epoch
        || root.receipt() != root_floor
        || ledger.receipt() != minimum
    {
        return Err(Error(
            "bounded resumed roots do not match trusted scope and frontier",
        ));
    }
    let (grant, ack) = root.latest().ok_or(Error("no completed prior root Step"))?;
    let saved = ack
        .recovery_receipt
        .as_ref()
        .ok_or(Error("root Ack must bind recovery cut"))?;
    if saved.cut_hash != cut
        || saved.auxiliary != floor.auxiliary_receipt
        || ack.witness_receipt != floor.witness_receipt
    {
        return Err(Error("trusted root and selected auxiliary cut disagree"));
    }
    let record = ledger
        .records()
        .last()
        .ok_or(Error("missing bounded recovery cut"))?;
    let document = read_json(record)?;
    let cut_sequence = field::<u64>(&document, "cut_sequence")?;
    if record.header.kind != "RECOVERY_CUT"
        || field::<u64>(&document, "intended_root_step_sequence")? != root_floor.sequence
        || cut_sequence != root_floor.sequence
        || field::<String>(&document, "run")? != hex(&config.run_id)
        || field::<String>(&document, "config")? != hex(&config.config_hash)
        || field::<u32>(&document, "worker")? != worker
        || field::<String>(&document, "cid")? != hex(&config.cid)
        || field::<u64>(&document, "epoch")? != config.base.epoch
        || field::<u64>(&document, "signer_cache_frontier")? != 0
        || !field::<Vec<Json>>(&document, "quota_checkpoint")?.is_empty()
    {
        return Err(Error(
            "bounded recovery cut binding or empty frontier error",
        ));
    }
    let at = |key, kind| {
        let seq = field::<u64>(&document, key)?;
        ledger
            .records()
            .iter()
            .find(|r| r.header.sequence == seq && r.header.kind == kind)
            .ok_or(Error("missing bound auxiliary record"))
    };
    let archive = read_json(at("archive_manifest_ref", "ARCHIVE_MANIFEST")?)?;
    for field_name in [
        "blocks",
        "proofs",
        "durable_body_receipts",
        "native_parents",
        "body_requests",
        "body_retry_deadlines",
        "emitted_delivery_ids",
        "packet_attempt_counters",
        "blocked_members",
    ] {
        if !field::<Vec<Json>>(&archive, field_name)?.is_empty() {
            return Err(Error("bounded recovery archive is nonempty"));
        }
    }
    if field::<String>(&archive, "delivered_digest")? != hex(&config.genesis)
        || field::<u64>(&archive, "delivered_height")? != 0
    {
        return Err(Error("bounded recovery archive genesis disagrees"));
    }
    let life = read_json(at("lifecycle_ref", "LIFECYCLE")?)?;
    if field::<String>(&life, "state")? != "ACTIVE"
        || field::<i64>(&life, "local_stop_us")? != config.base.stop
    {
        return Err(Error(
            "recovery cut retains a terminal or changed lifecycle",
        ));
    }
    let intent = read_json(at("grant_intent_frontier", "GRANT_INTENT")?)?;
    use prost::Message as _;
    if byte_field(&intent, "canonical_grant_bytes")? != grant.encode_to_vec() {
        return Err(Error(
            "latest grant intent differs from root committed input",
        ));
    }
    let rngs = field::<Vec<Json>>(&document, "role_rng_checkpoints")?
        .iter()
        .map(|item| {
            Ok((
                fixed(&field::<String>(item, "role_id")?)?,
                crate::RngCheckpoint {
                    seed: fixed(&field::<String>(item, "seed")?)?,
                    stream: field(item, "stream")?,
                    word_position_hi: field(item, "word_position_hi")?,
                    word_position_lo: field(item, "word_position_lo")?,
                },
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    if rngs.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
        return Err(Error("unsorted recovery RNG roles"));
    }
    let bundle = load_pending_verify_fragment(ledger, minimum, cut, config, worker)?;
    Ok(ResumeCut {
        bundle,
        clock: field(&document, "clock_checkpoint")?,
        rngs,
        last_timer: field(
            &field::<Json>(&document, "runtime_timer_checkpoint")?,
            "last_issued_timer_id",
        )?,
        work_generation: field(&archive, "work_generation")?,
        input_sequence: field(&intent, "input_sequence")?,
    })
}

/// Genesis has no retained actor state to reconcile. The normal engine still
/// starts only after fresh calibration and authorized material validation.
pub(crate) struct InitialStartup;
impl native::RecoveryStartup for InitialStartup {
    fn ready(
        &self,
        _: native::ActorId,
        cut: Id,
        state: Id,
    ) -> std::result::Result<(), native::RecoveryError> {
        if cut != [0; 32] || state == [0; 32] {
            return Err(native::RecoveryError("invalid initial actor material"));
        }
        Ok(())
    }
    fn released(&self) -> BoxFuture<'_, std::result::Result<(), native::RecoveryError>> {
        Box::pin(async { Ok(()) })
    }
}
impl StartupFence {
    pub fn actor_reports(&self) -> Vec<crate::native_wire::ActorReady> {
        self.state
            .lock()
            .unwrap()
            .ready
            .iter()
            .map(|(actor, id)| crate::native_wire::ActorReady {
                actor: actor_tag(*actor).into(),
                state_hash: id.to_vec(),
            })
            .collect()
    }
}
