use crate::{HistoricalPlan, ValidatedRegistry};
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
        match self.expected_plan(&plan.input()) {
            Ok(expected) if expected == *plan => Self::resolve_reference(expected, references),
            Ok(_) => PlanOutcome::Reject(PlanError::PlanMismatch),
            Err(error) => PlanOutcome::Reject(error),
        }
    }

    /// The caller supplies the exact activated policy and verified history for this boundary.
    pub fn resolve_historical(
        &self,
        input: &PlanInput,
        references: &dyn PublicReferenceView,
        basis: &HistoricalPlan,
    ) -> PlanOutcome {
        match self.expected_historical(input, basis) {
            Ok(Some(plan)) => Self::resolve_reference(plan, references),
            Ok(None) => match references.reference(&input.reference_block) {
                InputStatus::Ready(anchor) if anchor.block == input.reference_block => PlanOutcome::None,
                InputStatus::Wait(missing) => PlanOutcome::Wait(missing),
                InputStatus::Reject(error) => PlanOutcome::Reject(PlanError::Source(error)),
                _ => PlanOutcome::Reject(PlanError::PlanMismatch),
            },
            Err(error) => PlanOutcome::Reject(error),
        }
    }

    pub fn validate_historical_plan(
        &self,
        plan: &PlanRecord,
        references: &dyn PublicReferenceView,
        basis: &HistoricalPlan,
    ) -> PlanOutcome {
        match self.resolve_historical(&plan.input(), references, basis) {
            PlanOutcome::Ready(expected) if expected == *plan => PlanOutcome::Ready(expected),
            PlanOutcome::Ready(_) | PlanOutcome::None => PlanOutcome::Reject(PlanError::PlanMismatch),
            other => other,
        }
    }

    fn expected_historical(
        &self,
        input: &PlanInput,
        basis: &HistoricalPlan,
    ) -> Result<Option<PlanRecord>, PlanError> {
        input.validate()?;
        if input.window.h_min <= self.config.genesis.block.height {
            return Err(PlanError::InvalidWindow);
        }
        if input.session.beta == 0 || basis.active.floor < basis.active.force.block
            || basis.height != input.reference_block.origin.max(basis.active.floor)
        {
            return Err(PlanError::PlanMismatch);
        }
        let Some(region) = basis.active.policy.select(input.source_region)? else { return Ok(None); };
        let mut plan = self.plan_for_region(input, region)?;
        plan.authorization_ref = basis.bind(plan.authorization_ref, region)?;
        plan.leader_rule = RuleSpec::historical_plan(plan.authorization_ref);
        Ok(Some(plan))
    }

    pub fn leader(&self, plan: &PlanRecord, view: u64) -> Result<Pid, PlanError> {
        self.check_plan(plan)?;
        let zero_based = view.checked_sub(1).ok_or(PlanError::ViewZero)?;
        Ok(Pid::from_region_member(plan.region, MemberId::new((zero_based % 5) as u8)?))
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
        let (region, _) = schedule(&input.session, 1)?;
        self.plan_for_region(input, region)
    }

    fn plan_for_region(&self, input: &PlanInput, region: RegionId) -> Result<PlanRecord, PlanError> {
        input.validate()?;
        if input.window.h_min <= self.config.genesis.block.height {
            return Err(PlanError::InvalidWindow);
        }
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
        let expected = if plan.leader_rule.rule_id == 2 && plan.session.beta > 0 {
            let mut expected = self.plan_for_region(&plan.input(), plan.region)?;
            expected.authorization_ref = plan.authorization_ref;
            expected.leader_rule = RuleSpec::historical_plan(plan.authorization_ref);
            expected
        } else {
            self.expected_plan(&plan.input())?
        };
        if *plan != expected {
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
