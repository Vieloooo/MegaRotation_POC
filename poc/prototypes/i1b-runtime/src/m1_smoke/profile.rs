use crate::{Error, Id, Result, hash_parts, native_config::NativeScheme};
use commonware_codec::{Copying, Decode, Encode};
use commonware_consensus::simplex::types::Finalization;
use commonware_cryptography::{
    Signer,
    bls12381::primitives::{
        group::{G2, Private},
        ops,
        variant::MinSig,
    },
    certificate::Verifier,
    ed25519,
    sha256::Digest,
};
use commonware_parallel::Sequential;
use poc_authorization::{POP_NAMESPACE, PlanOutcome, StaticScheduleProvider, ValidatedRegistry};
use poc_core::{
    fixture::{CheckedSetup, check_setup, reg_a_l1_fixture},
    merge::{initial_padding, merge},
    metadata::{MetadataProvider as _, RegionalMetadata, l2_timestamp_us},
};
use poc_types::domain::*;
use poc_viewer_gfr::{FixtureL1Reader, GfrParameters, VerificationBasis};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(crate) const SEED: u64 = 42;
pub(crate) const SECOND: u64 = 1_000_000;
pub(crate) const START_US: u64 = 10_800_000;
pub(crate) const GRACE_US: u64 = 3 * SECOND;
pub(crate) const EXPIRY_US: u64 = 8 * SECOND;
pub(crate) const SOURCE: &[u8] = b"M1/healthy/espresso/v1";
pub(crate) const L1_SOURCE: &[u8] = b"M1/healthy/l1/v1";
pub(crate) const VIEW_MAX: u64 = 128;
pub(crate) const REFRESH_US: u64 = 80_000;
pub(crate) const CLOCK_TRANSIT_US: u64 = 250;
pub(crate) const LINK_US: u64 = 4_000;
pub(crate) const WORK_US: u64 = 5_000;
pub(crate) const LEASE_US: u64 = 180_000;
pub(crate) const STARTUP_SKEW_US: u64 = 3_000;
pub(crate) const CUT_US: u64 = 33 * SECOND;

pub(crate) fn parameters() -> GfrParameters {
    GfrParameters {
        announcement_us: 10 * SECOND,
        grace_us: GRACE_US,
        expiry_us: EXPIRY_US,
    }
}

fn keypair(pid: Pid) -> (Private, G2) {
    let mut rng = ChaCha20Rng::seed_from_u64(303_100 + u64::from(pid.get()));
    ops::keypair::<_, MinSig>(&mut rng)
}

pub(crate) fn fixture_key(pid: Pid) -> Private {
    keypair(pid).0
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PublicInputs {
    pub config: PublicConfig,
    pub registry: RegistryRecord,
    pub genesis: ExecutionAnchor,
    pub dep: DepFixture,
    pub l1: L1Fixture,
}

impl PublicInputs {
    pub fn fixture() -> Result<Self> {
        let mut entries = Vec::new();
        for i in 0..15 {
            let pid = Pid::new(i).unwrap();
            let identity = ed25519::PrivateKey::from_seed(303_000 + u64::from(i));
            let (private, public) = keypair(pid);
            let pop = ops::sign_proof_of_possession::<MinSig>(&private, POP_NAMESPACE);
            entries.push(RegistryEntry {
                pid,
                identity_key: identity.public_key().encode().as_ref().try_into().unwrap(),
                vote_key: public.encode().as_ref().try_into().unwrap(),
                pop: pop.encode().as_ref().try_into().unwrap(),
            });
        }
        let registry = RegistryRecord { entries };
        let dep = DepFixture {
            first_height: 1,
            rows: (1..=40)
                .map(|amount| vec![Credit { to: 2, amount }])
                .collect(),
        };
        let l1 = reg_a_l1_fixture();
        let anchor = Anchor {
            block: BlockRef {
                height: 0,
                origin: 3,
                block_id: Hash32::new([0x21; 32]),
            },
            timestamp_us: 10 * SECOND,
        };
        let genesis = ExecutionAnchor {
            state: ExecutionState {
                metadata: anchor.metadata(),
                accounts: BTreeMap::from([(
                    1,
                    Account {
                        nonce: 0,
                        balance: 100,
                    },
                )]),
            },
            anchor,
            native_hash: Hash32::new([0x42; 32]),
        };
        let config = PublicConfig {
            chain_id: 7,
            rule_version: 1,
            verifier_version: 1,
            registry_hash: registry.digest().map_err(domain)?,
            dep_fixture_hash: dep.digest().map_err(domain)?,
            l1_fixture_hash: l1.digest().map_err(domain)?,
            genesis: genesis.anchor.clone(),
            genesis_state_root: genesis.state.root().map_err(domain)?,
            genesis_native_hash: genesis.native_hash,
            l2_slot_us: SECOND,
            l1_genesis_timestamp_us: 0,
            l1_block_us: 2 * SECOND,
            t_dep_us: 5 * SECOND,
            t_drift_us: 15 * SECOND,
            metadata_window_us: 200_000,
            byte_limit: 65_536,
        };
        Ok(Self {
            config,
            registry,
            genesis,
            dep,
            l1,
        })
    }

    pub fn setup(&self) -> Result<CheckedSetup> {
        check_setup(
            &self.config,
            &self.genesis,
            self.dep.clone(),
            self.l1.clone(),
        )
        .map_err(|_| Error("invalid M1 execution fixture"))
    }

    pub fn plans(&self) -> Result<StaticScheduleProvider> {
        let registry = ValidatedRegistry::load(self.registry.clone())
            .map_err(|_| Error("invalid M1 registry/PoP"))?;
        StaticScheduleProvider::new(self.config.clone(), registry)
            .map_err(|_| Error("invalid M1 plan configuration"))
    }

    pub fn deliver_l1(
        &self,
        setup: &CheckedSetup,
        local: &mut FixtureL1Reader,
        now: u64,
    ) -> Result<()> {
        let mut finalizer = setup.l1().finalizer();
        for row in self
            .l1
            .rows
            .iter()
            .take_while(|row| row.finalized_at_us <= now)
        {
            let row = finalizer
                .finalize(&row.header, row.finalized_at_us)
                .map_err(|_| Error("invalid M1 L1 fixture row"))?;
            local
                .deliver_header(&row, now)
                .map_err(|_| Error("M1 L1 header delivery"))?;
            local
                .deliver_empty_control(&row, now)
                .map_err(|_| Error("M1 L1 empty control delivery"))?;
        }
        Ok(())
    }
}

pub(crate) fn domain(_: DomainError) -> Error {
    Error("invalid M1 canonical domain object")
}

pub(crate) fn namespace(descriptor: &Descriptor) -> Id {
    hash_parts(
        b"PoC/RegionalBFT/Simplex/v1",
        &[
            descriptor.plan.config_ref.as_bytes(),
            descriptor.cid.as_bytes(),
            &1u64.to_be_bytes(),
            &1u16.to_be_bytes(),
            &1u16.to_be_bytes(),
        ],
    )
}

#[derive(Clone)]
pub(crate) struct NativeFinality(pub StaticScheduleProvider);

impl VerificationBasis for NativeFinality {
    fn require(&self, descriptor: &Descriptor, block: &PhysicalBlock) -> InputStatus<()> {
        let window = &descriptor.plan.window;
        if self.0.validate_descriptor(descriptor).is_ok()
            && self.0.config().rule_version == 1
            && self.0.config().verifier_version == 1
            && (window.h_min..=window.h_max).contains(&block.metadata.height)
            && block.validate().is_ok()
        {
            InputStatus::Ready(())
        } else {
            InputStatus::Reject(InputError::InvalidSource)
        }
    }
}
impl FinalVerifier for NativeFinality {
    fn check(&self, descriptor: &Descriptor, block: &PhysicalBlock, proof: &[u8]) -> bool {
        if !matches!(self.require(descriptor, block), InputStatus::Ready(()))
            || proof.is_empty()
            || proof.len() > 32_768
        {
            return false;
        }
        let namespace = namespace(descriptor);
        let verifier = NativeScheme::verifier(
            &namespace,
            self.0
                .registry()
                .participants(descriptor.plan.region)
                .clone(),
        );
        let Ok(finalized) = Finalization::<NativeScheme, Digest>::decode_cfg(
            Copying(proof),
            &verifier.certificate_codec_config(),
        ) else {
            return false;
        };
        let proposal = &finalized.proposal;
        finalized.encode().as_ref() == proof
            && proposal.round.epoch().get() == 1
            && (1..=VIEW_MAX).contains(&proposal.round.view().get())
            && proposal.parent < proposal.round.view()
            && block
                .digest()
                .is_ok_and(|hash| hash.as_bytes() == &proposal.payload.0)
            && finalized.verify(
                &mut ChaCha20Rng::from_seed(namespace),
                &verifier,
                &Sequential,
            )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AnchorStep {
    pub descriptor: Descriptor,
    pub chain: Vec<PhysicalBlock>,
    pub final_proof: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Install {
    pub descriptor: Descriptor,
    pub execution: ExecutionAnchor,
    pub history: Vec<AnchorStep>,
    pub prepared_at_us: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projected_l1_lag_us: Option<u64>,
}

pub(crate) struct ReadyMaterial {
    pub descriptor: Descriptor,
    pub execution: ExecutionAnchor,
    setup: CheckedSetup,
    l1: FixtureL1Reader,
    inputs: PublicInputs,
    projected_l1_lag_us: u64,
}

impl ReadyMaterial {
    pub fn prepare(inputs: PublicInputs, install: &Install, pid: Pid) -> Result<Self> {
        let setup = inputs.setup()?;
        let schedules = inputs.plans()?;
        let finality = NativeFinality(schedules.clone());
        let mut l1 = FixtureL1Reader::new(L1_SOURCE.to_vec(), setup.l1().reader())
            .map_err(|_| Error("M1 regional L1 reader"))?;
        let projected_l1_lag_us = install.projected_l1_lag_us.unwrap_or(0);
        inputs.deliver_l1(&setup, &mut l1, install.prepared_at_us.saturating_sub(projected_l1_lag_us))?;
        let metadata = RegionalMetadata::new(&inputs.config, &l1).map_err(domain)?;
        let mut anchor = inputs.genesis.clone();
        let mut prior_initial = inputs.genesis.anchor.clone();
        for (r, step) in install.history.iter().enumerate() {
            let tip = step.chain.last().ok_or(Error("M1 empty anchor history"))?;
            if step.descriptor.anchor != anchor.anchor
                || step.descriptor.plan.session
                    != (SessionId {
                        beta: 0,
                        r: r as u64,
                    })
                || tip.metadata.height != (r as u64 + 1) * 10
                || !matches!(
                    schedules.validate_plan(&step.descriptor.plan, &Reference(&prior_initial)),
                    PlanOutcome::Ready(_)
                )
                || !finality.check(&step.descriptor, tip, &step.final_proof)
            {
                return Err(Error("M1 anchor history proof/plan mismatch"));
            }
            let InputStatus::Ready(output) = merge(
                &anchor,
                &step.chain,
                &step.descriptor.plan.window,
                &metadata,
                setup.dep(),
                &inputs.config,
            ) else {
                return Err(Error("M1 anchor history cannot execute"));
            };
            let last = output
                .blocks
                .last()
                .ok_or(Error("M1 anchor history has no tip"))?;
            prior_initial = anchor.anchor;
            anchor = ExecutionAnchor {
                anchor: Anchor {
                    block: BlockRef {
                        block_id: tip.digest().map_err(domain)?,
                        height: tip.metadata.height,
                        origin: tip.metadata.origin,
                    },
                    timestamp_us: tip.metadata.timestamp_us,
                },
                native_hash: last.native_hash,
                state: last.state.clone(),
            };
        }
        let descriptor = &install.descriptor;
        let force = descriptor.plan.cause == BoundaryCause::Force;
        let expected_session = if force {
            // This profile switches from the old branch after the regular
            // predecessor completed. The selected old duty has no G contribution.
            SessionId { beta: 1, r: install.history.len() as u64 }
        } else {
            SessionId { beta: 0, r: install.history.len() as u64 }
        };
        let reference = if force { &anchor.anchor } else { &prior_initial };
        if anchor != install.execution
            || descriptor.anchor != anchor.anchor
            || descriptor.plan.session != expected_session
            || descriptor.plan.region != pid.region()
            || schedules.validate_descriptor(descriptor).is_err()
            || !matches!(
                schedules.validate_plan(&descriptor.plan, &Reference(reference)),
                PlanOutcome::Ready(_)
            )
        {
            return Err(Error("M1 exact anchor preparation failed"));
        }
        Ok(Self {
            descriptor: descriptor.clone(),
            execution: anchor,
            setup,
            l1,
            inputs,
            projected_l1_lag_us,
        })
    }

    pub fn refresh(&mut self, tick: u64) -> Result<()> {
        self.inputs.deliver_l1(&self.setup, &mut self.l1, tick.saturating_sub(self.projected_l1_lag_us))
    }

    pub fn metadata_window_us(&self) -> i64 {
        i64::try_from(self.inputs.config.metadata_window_us).expect("M1 metadata window")
    }

    pub fn timestamp(&self, height: u64) -> Result<u64> {
        l2_timestamp_us(&self.inputs.config, height).map_err(domain)
    }

    pub fn execution(&self, chain: &[PhysicalBlock]) -> InputStatus<ExecutionState> {
        let metadata = RegionalMetadata::new(&self.inputs.config, &self.l1)
            .expect("checked M1 metadata configuration");
        match merge(
            &self.execution,
            chain,
            &self.descriptor.plan.window,
            &metadata,
            self.setup.dep(),
            &self.inputs.config,
        ) {
            InputStatus::Ready(merged) => InputStatus::Ready(
                merged
                    .blocks
                    .last()
                    .map_or_else(|| self.execution.state.clone(), |tip| tip.state.clone()),
            ),
            InputStatus::Wait(wait) => InputStatus::Wait(wait),
            InputStatus::Reject(error) => InputStatus::Reject(error),
        }
    }

    pub fn build(
        &self,
        chain: &[PhysicalBlock],
        parent: Hash32,
        height: u64,
    ) -> InputStatus<PhysicalBlock> {
        let metadata = RegionalMetadata::new(&self.inputs.config, &self.l1)
            .expect("checked M1 metadata configuration");
        let next_state = if chain.is_empty()
            && self.descriptor.plan.window.h_min > self.execution.anchor.block.height + 1
        {
            match initial_padding(
                &self.execution,
                &self.descriptor.plan.window,
                &metadata,
                self.setup.dep(),
                &self.inputs.config,
            ) {
                InputStatus::Ready(output) => InputStatus::Ready(
                    output.blocks.last().expect("nonempty initial padding").state.clone()
                ),
                InputStatus::Wait(wait) => InputStatus::Wait(wait),
                InputStatus::Reject(error) => InputStatus::Reject(error),
            }
        } else {
            self.execution(chain)
        };
        let state = match next_state {
            InputStatus::Ready(state) => state,
            InputStatus::Wait(wait) => return InputStatus::Wait(wait),
            InputStatus::Reject(error) => return InputStatus::Reject(error),
        };
        let metadata = match metadata.at(&self.execution.anchor, height) {
            InputStatus::Ready(value) => value,
            InputStatus::Wait(wait) => return InputStatus::Wait(wait),
            InputStatus::Reject(error) => return InputStatus::Reject(error),
        };
        let Some(sender) = state.accounts.get(&1) else {
            return InputStatus::Reject(InputError::InvalidSource);
        };
        let amount = 1 + (SEED + height) % 3;
        let transaction = Transfer {
            from: 1,
            to: 2,
            nonce: sender.nonce,
            amount,
        }
        .encode_raw();
        InputStatus::Ready(PhysicalBlock {
            parent,
            metadata,
            pre_state_root: state.root().expect("checked parent state"),
            transactions: vec![transaction],
        })
    }
}

pub(crate) struct Reference<'a>(pub &'a Anchor);
impl PublicReferenceView for Reference<'_> {
    fn reference(&self, reference: &BlockRef) -> InputStatus<Anchor> {
        if &self.0.block == reference {
            InputStatus::Ready(self.0.clone())
        } else {
            InputStatus::Reject(InputError::InvalidSource)
        }
    }
}
