//! Explicitly delivered public readers and controlled prerequisite fixtures.
use crate::fixture::{Basis, E_SOURCE, ExactProofs, FIXTURE_ID, Fixture, L_SOURCE};
use poc_authorization::{PlanError, PlanOutcome};
use poc_core::{
    fixture::DepProvider,
    metadata::{MetadataProvider, RegionalMetadata},
};
use poc_types::domain::*;
use poc_viewer_gfr::*;
use sim_espresso::{
    BlockHeader, FinalizedBlock, FinalizedLogReader, LogMessage, ProposedBlock, PublicTimeUs,
    ReaderState, TickUs,
};
use std::{cell::Cell, collections::BTreeMap};

pub fn ep(key_us: u64, q: u64) -> ReplayPosition {
    ReplayPosition {
        key_us,
        kind: ReplayKind::Espresso,
        index: ReplayIndex::Espresso(q),
    }
}
pub fn end(key_us: u64, r: u64) -> ReplayPosition {
    ReplayPosition {
        key_us,
        kind: ReplayKind::SessionEnd,
        index: ReplayIndex::Session(SessionId { beta: 0, r }),
    }
}
pub fn expiry(key_us: u64, h: u64) -> ReplayPosition {
    ReplayPosition {
        key_us,
        kind: ReplayKind::Expire,
        index: ReplayIndex::Height(h),
    }
}
pub fn profile() -> VerificationProfile {
    VerificationProfile::NonCryptographicFixture {
        id: FIXTURE_ID.to_owned(),
    }
}

pub struct PlanGate<'a> {
    pub fixture: &'a Fixture,
    pub wait: Cell<Option<u64>>,
    pub none: Cell<Option<u64>>,
}
impl PlanProvider for PlanGate<'_> {
    fn resolve(&self, input: &PlanInput, refs: &dyn PublicReferenceView) -> PlanOutcome {
        if self.wait.get() == Some(input.session.r) {
            return PlanOutcome::Wait(InputMissing::Reference(input.reference_block.clone()));
        }
        if self.none.get() == Some(input.session.r) {
            return PlanOutcome::None;
        }
        self.fixture.plan.resolve(input, refs)
    }
    fn descriptor(&self, plan: &PlanRecord, anchor: &Anchor) -> Result<Descriptor, PlanError> {
        self.fixture.plan.descriptor(plan, anchor)
    }
}

pub struct ControlFixture {
    pub base: FixtureL1Reader,
    pub controls: BTreeMap<u64, Vec<ControlRef>>,
    pub source: Vec<u8>,
    pub changed_header: Option<L1Header>,
}
impl FinalizedL1Reader for ControlFixture {
    fn header(&self, h: u64) -> InputStatus<L1Header> {
        if let Some(x) = &self.changed_header {
            if x.height == h {
                return InputStatus::Ready(x.clone());
            }
        }
        self.base.header(h)
    }
    fn require_control_coverage(&self, h: u64) -> InputStatus<()> {
        self.base.require_control_coverage(h)
    }
    fn finalized_tip(&self) -> Option<L1BlockRef> {
        self.base.finalized_tip()
    }
}
impl L1ControlReader for ControlFixture {
    fn source_id(&self) -> &[u8] {
        &self.source
    }
    fn controls_at(&self, h: u64) -> InputStatus<Vec<ControlRef>> {
        match self.base.controls_at(h) {
            InputStatus::Ready(_) => {
                InputStatus::Ready(self.controls.get(&h).cloned().unwrap_or_default())
            }
            InputStatus::Wait(m) => InputStatus::Wait(m),
            InputStatus::Reject(e) => InputStatus::Reject(e),
        }
    }
}

struct MetadataGate<'a> {
    base: RegionalMetadata<'a>,
    wait: Option<u64>,
}
impl MetadataProvider for MetadataGate<'_> {
    fn config_ref(&self) -> ConfigRef {
        self.base.config_ref()
    }
    fn at(&self, a: &Anchor, h: u64) -> InputStatus<Metadata> {
        if self.wait == Some(h) {
            InputStatus::Wait(InputMissing::L1Header(h))
        } else {
            self.base.at(a, h)
        }
    }
}
struct DepGate<'a> {
    base: &'a dyn DepProvider,
    wait: Option<u64>,
}
impl DepProvider for DepGate<'_> {
    fn config_ref(&self) -> ConfigRef {
        self.base.config_ref()
    }
    fn at(&self, m: &Metadata) -> InputStatus<Vec<Vec<u8>>> {
        if self.wait == Some(m.height) {
            InputStatus::Wait(InputMissing::Dep(m.height))
        } else {
            self.base.at(m)
        }
    }
}

pub struct World<'a> {
    pub f: &'a Fixture,
    pub plan: PlanGate<'a>,
    pub e: ReaderState,
    pub l1: ControlFixture,
    pub proofs: ExactProofs,
    pub basis: Basis,
    pub params: GfrParameters,
    pub binding: bool,
    pub binding_profile: VerificationProfile,
    pub wait_metadata: Option<u64>,
    pub wait_dep: Option<u64>,
}

impl<'a> World<'a> {
    pub fn new(f: &'a Fixture, id: &[u8]) -> Self {
        let base = FixtureL1Reader::new(L_SOURCE.to_vec(), f.setup.l1().reader()).unwrap();
        let mut world = Self {
            f,
            plan: PlanGate {
                fixture: f,
                wait: Cell::new(None),
                none: Cell::new(None),
            },
            e: ReaderState::new(E_SOURCE.to_vec(), id.to_vec()).unwrap(),
            l1: ControlFixture {
                base,
                controls: BTreeMap::new(),
                source: L_SOURCE.to_vec(),
                changed_header: None,
            },
            proofs: ExactProofs::default(),
            basis: Basis::default(),
            params: GfrParameters {
                announcement_us: 6_000_000,
                grace_us: 3_000_000,
                expiry_us: 30_000_000,
            },
            binding: true,
            binding_profile: profile(),
            wait_metadata: None,
            wait_dep: None,
        };
        world.deliver_l1(30, 30);
        world
    }

    pub fn clear_l1(&mut self) {
        self.l1.base = FixtureL1Reader::new(L_SOURCE.to_vec(), self.f.setup.l1().reader()).unwrap();
    }
    pub fn deliver_l1(&mut self, headers: u64, controls: u64) {
        for row in &self.f.tokens {
            if row.header().height <= headers {
                self.l1
                    .base
                    .deliver_header(row, row.finalized_at_us() + 500_000)
                    .unwrap();
            }
            if row.header().height <= controls {
                self.l1
                    .base
                    .deliver_empty_control(row, row.finalized_at_us() + 500_000)
                    .unwrap();
            }
        }
    }

    pub fn init_input(&self, input: &InitInput) -> Result<InitResult, GfrError> {
        let metadata = MetadataGate {
            base: RegionalMetadata::new(self.f.config(), &self.l1).unwrap(),
            wait: self.wait_metadata,
        };
        let dep = DepGate {
            base: self.f.setup.dep(),
            wait: self.wait_dep,
        };
        let services = ReplayServices {
            config: self.f.config(),
            plan: &self.plan,
            metadata: &metadata,
            dep: &dep,
            finality: if self.binding {
                Some(VerifierBinding {
                    verifier: &self.proofs,
                    basis: &self.basis,
                    profile: self.binding_profile.clone(),
                })
            } else {
                None
            },
        };
        init(
            input,
            &ReplayReads {
                espresso: &self.e,
                l1: &self.l1,
            },
            &services,
        )
    }
    pub fn init(&self) -> ReplayState {
        let input = InitInput::Genesis {
            config: self.f.config().clone(),
            genesis: self.f.genesis().clone(),
            parameters: self.params,
            expected_verification: profile(),
        };
        let result = self.init_input(&input).expect("fixture init error");
        assert!(
            matches!(result.status, ReplayStatus::Ready),
            "fixture init {:?}",
            result.status
        );
        result.state.expect("fixture ready state")
    }
    pub fn genesis_input(&self) -> InitInput {
        InitInput::Genesis {
            config: self.f.config().clone(),
            genesis: self.f.genesis().clone(),
            parameters: self.params,
            expected_verification: profile(),
        }
    }

    pub fn advance(&self, state: &ReplayState, through: &ReplayPosition) -> ReplayResult {
        let metadata = MetadataGate {
            base: RegionalMetadata::new(self.f.config(), &self.l1).unwrap(),
            wait: self.wait_metadata,
        };
        let dep = DepGate {
            base: self.f.setup.dep(),
            wait: self.wait_dep,
        };
        let services = ReplayServices {
            config: self.f.config(),
            plan: &self.plan,
            metadata: &metadata,
            dep: &dep,
            finality: if self.binding {
                Some(VerifierBinding {
                    verifier: &self.proofs,
                    basis: &self.basis,
                    profile: self.binding_profile.clone(),
                })
            } else {
                None
            },
        };
        advance_global(
            state,
            &ReplayReads {
                espresso: &self.e,
                l1: &self.l1,
            },
            &services,
            Some(through),
        )
    }

    pub fn body(&self, b: &PhysicalBlock) -> LogMessage {
        LogMessage {
            codec: PUBLICATION_CODEC,
            bytes: encode_publication(&PublicObject::Body(b.clone())).unwrap(),
        }
    }
    pub fn proof(&self, d: &Descriptor, b: &PhysicalBlock) -> LogMessage {
        let proof = self.proofs.register(d, b);
        LogMessage {
            codec: PUBLICATION_CODEC,
            bytes: encode_publication(&PublicObject::FinalProof {
                descriptor: d.clone(),
                target: b.digest().unwrap(),
                proof,
            })
            .unwrap(),
        }
    }
    pub fn invalid_proof(&self, d: &Descriptor, b: &PhysicalBlock) -> LogMessage {
        LogMessage {
            codec: PUBLICATION_CODEC,
            bytes: encode_publication(&PublicObject::FinalProof {
                descriptor: d.clone(),
                target: b.digest().unwrap(),
                proof: b"INVALID-NONCRYPTO-WITNESS".to_vec(),
            })
            .unwrap(),
        }
    }
    pub fn result_messages(&self, d: &Descriptor, bodies: &[PhysicalBlock]) -> Vec<LogMessage> {
        let mut m = bodies.iter().map(|b| self.body(b)).collect::<Vec<_>>();
        m.push(self.proof(d, bodies.last().unwrap()));
        m
    }
    pub fn block(q: u64, key: u64, messages: Vec<LogMessage>) -> FinalizedBlock {
        FinalizedBlock {
            proposal: ProposedBlock {
                q,
                theta_us: PublicTimeUs(key),
                proposed_at_us: TickUs(key),
                live_at_proposal: true,
                messages,
            },
            finalized_at_us: TickUs(key + 10),
        }
    }
    pub fn deliver(&mut self, b: &FinalizedBlock) {
        self.e
            .receive_block(E_SOURCE, b.clone(), TickUs(b.finalized_at_us.0 + 100))
            .unwrap();
    }
    pub fn deliver_header(&mut self, b: &FinalizedBlock) {
        self.e
            .receive_header(
                E_SOURCE,
                BlockHeader::from(b),
                TickUs(b.finalized_at_us.0 + 100),
            )
            .unwrap();
    }
    pub fn deliver_message(&mut self, b: &FinalizedBlock, i: u32) {
        self.e
            .receive_message(
                E_SOURCE,
                b.proposal.q,
                i,
                b.proposal.messages[i as usize].clone(),
                TickUs(b.finalized_at_us.0 + 100),
            )
            .unwrap();
    }
    pub fn tip(&self) -> Option<u64> {
        self.e.tip().map(|x| x.0)
    }
}
