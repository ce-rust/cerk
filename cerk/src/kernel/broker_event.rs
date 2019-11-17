use crate::kernel::CloudEvent;
use crate::runtime::channel::BoxedSender;
use crate::runtime::{InternalServerFn, InternalServerId};

pub enum BrokerEvent {
    ScheduleInternalServer(InternalServerId, InternalServerFn),
    InernalServerScheduled(InternalServerId, BoxedSender),
    IncommingCloudEvent(InternalServerId, CloudEvent),
}
