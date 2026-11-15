use crate::{ClockView, Delivery, EndpointBinding, Id, LocalTime, Message, StepStamp};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ServiceInput {
    Message(Delivery),
    Timer {
        timer_id: Id,
        absolute_local_deadline_us: LocalTime,
    },
    External {
        kind: u32,
        bytes: Vec<u8>,
    },
}

/// The controller supplies the step and clock under its grant authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ServiceGrant {
    pub target: EndpointBinding,
    pub step: StepStamp,
    pub clock: ClockView,
    pub input: ServiceInput,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ServiceEffect {
    Send(Message),
    ArmTimer {
        timer_id: Id,
        absolute_local_deadline_us: LocalTime,
    },
    CancelTimer {
        timer_id: Id,
    },
    Work {
        work_id: Id,
        duration_us: u64,
        input: Vec<u8>,
    },
}

/// Returned intents go through the controller's root-step commit.
pub trait GrantConsumer {
    type Error;

    fn on_grant(&mut self, grant: ServiceGrant) -> Result<Vec<ServiceEffect>, Self::Error>;
}
