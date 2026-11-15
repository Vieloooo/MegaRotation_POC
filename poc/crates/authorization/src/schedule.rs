use crate::ValidatedRegistry;
use poc_types::domain::{
    Anchor, ConfigRef, Descriptor, DomainError, InputError, InputMissing, InputStatus, MemberId,
    Pid, PlanInput, PlanRecord, ProgramConfig, PublicConfig, PublicReferenceView, RegionId,
    RuleSpec, SessionId, authorization_ref, regional_cid,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigError {
    Malformed(DomainError),
    RegistryMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanError {
    Malformed(DomainError),
    InvalidWindow,
    ViewZero,
    PlanMismatch,
    CidMismatch,
    Source(InputError),
}

impl From<DomainError> for PlanError {
    fn from(error: DomainError) -> Self {
        Self::Malformed(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanOutcome {
    Ready(PlanRecord),
    Wait(InputMissing),
    None,
    Reject(PlanError),
}

#[derive(Clone, Debug)]
pub struct StaticScheduleProvider {
    config: PublicConfig,
    config_ref: ConfigRef,
    registry: ValidatedRegistry,
}

impl StaticScheduleProvider {
    pub fn new(config: PublicConfig, registry: ValidatedRegistry) -> Result<Self, ConfigError> {
        let config_ref = config.reference().map_err(ConfigError::Malformed)?;
        if config.registry_hash != registry.digest() {
            return Err(ConfigError::RegistryMismatch);
        }
        Ok(Self {
            config,
            config_ref,
            registry,
        })
    }

    pub fn config(&self) -> &PublicConfig {
        &self.config
    }

    pub fn config_ref(&self) -> ConfigRef {
        self.config_ref
    }

    pub fn registry(&self) -> &ValidatedRegistry {
        &self.registry
    }

    pub fn resolve(&self, input: &PlanInput, references: &dyn PublicReferenceView) -> PlanOutcome {
        match self.expected_plan(input) {
            Ok(plan) => Self::resolve_reference(plan, references),
            Err(error) => PlanOutcome::Reject(error),
        }
    }

    pub fn validate_plan(
        &self,
        plan: &PlanRecord,
        references: &dyn PublicReferenceView,
    ) -> PlanOutcome {
        match self.check_plan(plan) {
            Ok(()) => Self::resolve_reference(plan.clone(), references),
            Err(error) => PlanOutcome::Reject(error),
        }
    }

    pub fn leader(&self, plan: &PlanRecord, view: u64) -> Result<Pid, PlanError> {
        self.check_plan(plan)?;
        schedule(&plan.session, view).map(|(_, pid)| pid)
    }

    pub fn program_config(&self, plan: &PlanRecord) -> Result<ProgramConfig, PlanError> {
        self.check_plan(plan)?;
        Ok(ProgramConfig::from_plan(&self.config, plan)?)
    }

    pub fn descriptor(&self, plan: &PlanRecord, anchor: &Anchor) -> Result<Descriptor, PlanError> {
        let program = self.program_config(plan)?;
        if anchor.block.height < self.config.genesis.block.height
            || anchor.block.height >= plan.window.h_min
        {
            return Err(PlanError::InvalidWindow);
        }
        let cid = regional_cid(&plan.session, anchor.block.block_id, &program)?;
        Ok(Descriptor {
            plan: plan.clone(),
            anchor: anchor.clone(),
            cid,
        })
    }

    /// Recompute the registry/config binding and CID. Call validate_plan with a verified
    /// reference view to establish the public provenance of the embedded Plan.
    pub fn validate_descriptor(&self, descriptor: &Descriptor) -> Result<(), PlanError> {
        let expected = self.descriptor(&descriptor.plan, &descriptor.anchor)?;
        if descriptor.cid != expected.cid {
            return Err(PlanError::CidMismatch);
        }
        Ok(())
    }

    fn resolve_reference(plan: PlanRecord, references: &dyn PublicReferenceView) -> PlanOutcome {
        match references.reference(&plan.reference_block) {
            InputStatus::Ready(anchor) if anchor.block == plan.reference_block => {
                PlanOutcome::Ready(plan)
            }
            InputStatus::Ready(_) => {
                PlanOutcome::Reject(PlanError::Source(InputError::InvalidSource))
            }
            InputStatus::Wait(missing) => PlanOutcome::Wait(missing),
            InputStatus::Reject(error) => PlanOutcome::Reject(PlanError::Source(error)),
        }
    }

    fn expected_plan(&self, input: &PlanInput) -> Result<PlanRecord, PlanError> {
        input.validate()?;
        if input.window.h_min <= self.config.genesis.block.height {
            return Err(PlanError::InvalidWindow);
        }
        let (region, _) = schedule(&input.session, 1)?;
        let committee = self.registry.committee(region);
        let eligible_proposers = committee.iter().map(|entry| entry.pid).collect();
        Ok(PlanRecord {
            session: input.session.clone(),
            cause: input.cause,
            source_region: input.source_region,
            region,
            committee,
            total_weight: 5,
            fault_bound: 1,
            eligible_proposers,
            leader_rule: RuleSpec::static_schedule(),
            window: input.window.clone(),
            reference_block: input.reference_block.clone(),
            config_ref: self.config_ref,
            authorization_ref: authorization_ref(self.config_ref, self.registry.digest(), input)?,
        })
    }

    fn check_plan(&self, plan: &PlanRecord) -> Result<(), PlanError> {
        plan.validate()?;
        if *plan != self.expected_plan(&plan.input())? {
            return Err(PlanError::PlanMismatch);
        }
        Ok(())
    }
}

pub fn schedule(session: &SessionId, view: u64) -> Result<(RegionId, Pid), PlanError> {
    let zero_based = view.checked_sub(1).ok_or(PlanError::ViewZero)?;
    let region = RegionId::new(((session.beta % 3 + session.r % 3) % 3) as u8)?;
    let member = MemberId::new((zero_based % 5) as u8)?;
    Ok((region, Pid::from_region_member(region, member)))
}
