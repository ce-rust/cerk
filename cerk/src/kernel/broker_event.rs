use super::Config;
use crate::runtime::channel::BoxedSender;
use crate::runtime::{InternalServerFn, InternalServerId};
use cloud_event::CloudEvent;
use std::fmt;

pub enum BrokerEvent {
    Init,
    ScheduleInternalServer(InternalServerId, InternalServerFn),
    InernalServerScheduled(InternalServerId, BoxedSender),
    IncommingCloudEvent(InternalServerId, CloudEvent),
    ConfigUpdated(Config, InternalServerId),
    OutgoingCloudEvent(CloudEvent, InternalServerId),
    Batch(Vec<BrokerEvent>),
}

impl fmt::Display for BrokerEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrokerEvent::Init => write!(f, "Init"),
            BrokerEvent::ScheduleInternalServer(id, _) => {
                write!(f, "ScheduleInternalServer server_id={}", id)
            }
            BrokerEvent::InernalServerScheduled(id, _) => {
                write!(f, "InernalServerScheduled server_id={}", id)
            }
            BrokerEvent::IncommingCloudEvent(id, _) => {
                write!(f, "IncommingCloudEvent source_id={}", id)
            }
            BrokerEvent::ConfigUpdated(_, id) => write!(f, "ConfigUpdated destination_id={}", id),
            BrokerEvent::OutgoingCloudEvent(_, id) => {
                write!(f, "OutgoingCloudEvent destination_id={}", id)
            }
            BrokerEvent::Batch(_) => write!(f, "Batch"),
        }
    }
}
