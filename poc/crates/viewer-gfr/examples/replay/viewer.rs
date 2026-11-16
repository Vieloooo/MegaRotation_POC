use std::{collections::BTreeMap, time::Duration};

use poc_core::metadata::RegionalMetadata;
use poc_types::domain::{FinalizedL1Reader, L1BlockRef, PublicReferenceView};
use poc_viewer_gfr::*;
use sim_espresso::{BlockHeader, FinalizedBlock, ReaderState, TickUs};

use super::fixture::{
    ESPRESSO_SOURCE, Fixture, NonCryptographicFinality, SEC, Scenario, verification,
};

pub struct Viewer<'a> {
    pub espresso: ReaderState,
    pub l1: DeliveredL1<'a>,
    pub state: ReplayState,
    pub pause_ms: u64,
    tick: u64,
}

impl<'a> Viewer<'a> {
    pub fn new(fixture: &Fixture, scenario: &'a Scenario, id: &[u8], pause_ms: u64) -> Self {
        let espresso = ReaderState::new(ESPRESSO_SOURCE.to_vec(), id.to_vec()).unwrap();
        let l1 = DeliveredL1 {
            base: FixtureL1Reader::new(scenario.l1_source(), fixture.setup.l1().reader()).unwrap(),
            simulated_controls: &scenario.controls,
        };
        let state = init(
            &InitInput::Genesis {
                config: fixture.setup.config().clone(),
                genesis: fixture.setup.genesis().clone(),
                parameters: scenario.parameters,
                expected_verification: verification(),
            },
            &ReplayReads {
                espresso: &espresso,
                l1: &l1,
            },
            &services(
                fixture,
                &RegionalMetadata::new(fixture.setup.config(), &l1).unwrap(),
                &fixture.plan,
            ),
        )
        .unwrap()
        .state
        .unwrap();
        Self {
            espresso,
            l1,
            state,
            pause_ms,
            tick: if id == b"A" { 100 * SEC } else { 200 * SEC },
        }
    }

    pub fn advance(
        &mut self,
        fixture: &Fixture,
        through: &ReplayPosition,
        wait_plan: Option<u64>,
    ) -> ReplayResult {
        let gate = PlanGate {
            schedule: &fixture.plan,
            wait_plan,
        };
        let result = advance_global(
            &self.state,
            &ReplayReads {
                espresso: &self.espresso,
                l1: &self.l1,
            },
            &services(
                fixture,
                &RegionalMetadata::new(fixture.setup.config(), &self.l1).unwrap(),
                &gate,
            ),
            Some(through),
        );
        self.state = result.state.clone();
        result
    }

    pub fn restore(
        &mut self,
        fixture: &Fixture,
        checkpoint: GlobalCheckpoint,
    ) -> Result<InitResult, GfrError> {
        let result = init(
            &InitInput::Checkpoint(checkpoint),
            &ReplayReads {
                espresso: &self.espresso,
                l1: &self.l1,
            },
            &services(
                fixture,
                &RegionalMetadata::new(fixture.setup.config(), &self.l1).unwrap(),
                &fixture.plan,
            ),
        )?;
        if let Some(state) = &result.state {
            self.state = state.clone();
        }
        Ok(result)
    }

    pub fn block(&mut self, block: &FinalizedBlock) {
        let tick = self.tick();
        self.espresso
            .receive_block(ESPRESSO_SOURCE, block.clone(), tick)
            .unwrap();
    }

    pub fn fragments(&mut self, block: &FinalizedBlock, seed: u64, withhold: Option<usize>) {
        let n = block.proposal.messages.len();
        let mut slots: Vec<_> = (0..=n).rev().collect();
        let rotation = (seed % slots.len() as u64) as usize;
        slots.rotate_left(rotation);
        for slot in slots {
            if slot == n {
                let tick = self.tick();
                self.espresso
                    .receive_header(ESPRESSO_SOURCE, BlockHeader::from(block), tick)
                    .unwrap();
            } else if Some(slot) != withhold {
                self.message(block, slot);
                if (seed + slot as u64) % 3 == 0 {
                    self.message(block, slot);
                }
            }
        }
    }

    pub fn message(&mut self, block: &FinalizedBlock, index: usize) {
        let tick = self.tick();
        self.espresso
            .receive_message(
                ESPRESSO_SOURCE,
                block.proposal.q,
                index as u32,
                block.proposal.messages[index].clone(),
                tick,
            )
            .unwrap();
    }

    pub fn headers(&mut self, fixture: &Fixture, heights: impl IntoIterator<Item = usize>) {
        for height in heights {
            let tick = self.tick();
            self.l1
                .base
                .deliver_header(&fixture.l1_rows[height], tick.0)
                .unwrap();
        }
    }

    pub fn controls(&mut self, fixture: &Fixture, heights: impl IntoIterator<Item = usize>) {
        for height in heights {
            let tick = self.tick();
            self.l1
                .base
                .deliver_empty_control(&fixture.l1_rows[height], tick.0)
                .unwrap();
        }
    }

    fn tick(&mut self) -> TickUs {
        if self.pause_ms > 0 {
            std::thread::sleep(Duration::from_millis(self.pause_ms));
        }
        self.tick += 1;
        TickUs(self.tick)
    }
}

fn services<'a>(
    fixture: &'a Fixture,
    metadata: &'a RegionalMetadata<'_>,
    plan: &'a dyn PlanProvider,
) -> ReplayServices<'a> {
    ReplayServices {
        config: fixture.setup.config(),
        metadata,
        plan,
        dep: fixture.setup.dep(),
        finality: Some(VerifierBinding {
            verifier: &NonCryptographicFinality,
            basis: &NonCryptographicFinality,
            profile: verification(),
        }),
    }
}

struct PlanGate<'a> {
    schedule: &'a StaticScheduleProvider,
    wait_plan: Option<u64>,
}

impl PlanProvider for PlanGate<'_> {
    fn resolve(&self, input: &PlanInput, references: &dyn PublicReferenceView) -> PlanOutcome {
        if Some(input.session.r) == self.wait_plan {
            return PlanOutcome::Wait(InputMissing::Reference(input.reference_block.clone()));
        }
        self.schedule.resolve(input, references)
    }

    fn descriptor(&self, plan: &PlanRecord, anchor: &Anchor) -> Result<Descriptor, PlanError> {
        self.schedule.descriptor(plan, anchor)
    }
}

/// Nonempty mock records share the 03 fixture's independently delivered headers. The source
/// name and bytes identify a synthetic control list without 09 receipt validation.
pub struct DeliveredL1<'a> {
    base: FixtureL1Reader,
    simulated_controls: &'a BTreeMap<u64, Vec<ControlRef>>,
}

impl FinalizedL1Reader for DeliveredL1<'_> {
    fn header(&self, height: u64) -> InputStatus<L1Header> {
        self.base.header(height)
    }

    fn require_control_coverage(&self, through_height: u64) -> InputStatus<()> {
        self.base.require_control_coverage(through_height)
    }

    fn finalized_tip(&self) -> Option<L1BlockRef> {
        self.base.finalized_tip()
    }
}

impl L1ControlReader for DeliveredL1<'_> {
    fn source_id(&self) -> &[u8] {
        self.base.source_id()
    }

    fn controls_at(&self, height: u64) -> InputStatus<Vec<ControlRef>> {
        match self.base.controls_at(height) {
            InputStatus::Ready(empty) => InputStatus::Ready(
                self.simulated_controls
                    .get(&height)
                    .cloned()
                    .unwrap_or(empty),
            ),
            status => status,
        }
    }
}

#[test]
fn established_public_proof_remains_usable_when_the_optional_binding_is_absent() {
    let fixture = Fixture::new();
    let mut scenario = Scenario::new("G2", &fixture);
    scenario.espresso[2].proposal.messages.clear();
    let mut viewer = Viewer::new(&fixture, &scenario, b"cached-proof-test", 0);
    viewer.headers(&fixture, 0..=30);
    viewer.controls(&fixture, 0..=30);
    for block in &scenario.espresso[..=3] {
        viewer.block(block);
    }
    let first = viewer.advance(&fixture, &scenario.bound(1), None);
    assert_eq!(first.status, ReplayStatus::Ready);
    assert_eq!(first.state.public_state().entries.len(), 1);

    let reads = ReplayReads {
        espresso: &viewer.espresso,
        l1: &viewer.l1,
    };
    let metadata = RegionalMetadata::new(fixture.setup.config(), &viewer.l1).unwrap();
    let without_finality = ReplayServices {
        finality: None,
        ..services(&fixture, &metadata, &fixture.plan)
    };
    let empty = advance_global(
        &first.state,
        &reads,
        &without_finality,
        Some(&scenario.bound(2)),
    );
    assert_eq!(empty.status, ReplayStatus::Ready);
    assert_eq!(
        empty.state.public_state().pc.as_ref(),
        Some(&scenario.bound(2))
    );

    let new_proof = advance_global(
        &empty.state,
        &reads,
        &without_finality,
        Some(&scenario.bound(3)),
    );
    assert!(
        matches!(new_proof.status, ReplayStatus::Wait { missing, .. } if missing == vec![Missing::FinalVerifierUnavailable])
    );
    assert_eq!(new_proof.state, empty.state);
}
