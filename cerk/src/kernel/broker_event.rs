use crate::kernel::CloudEvent;
use crate::runtime::channel::BoxedSender;
use crate::runtime::{InternalServerFn, InternalServerId};
use std::fmt;

pub enum BrokerEvent {
    ScheduleInternalServer(InternalServerId, InternalServerFn),
    InernalServerScheduled(InternalServerId, BoxedSender),
    IncommingCloudEvent(InternalServerId, CloudEvent),
}

impl fmt::Display for BrokerEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrokerEvent::ScheduleInternalServer(id, _) => {
                write!(f, "ScheduleInternalServer server_id={}", id)
            }
            BrokerEvent::InernalServerScheduled(id, _) => {
                write!(f, "InernalServerScheduled server_id={}", id)
            }
            BrokerEvent::IncommingCloudEvent(id, _) => write!(f, "IncommingCloudEvent source_id={}", id),
        }
    }
}
