use crate::runtime::channel::BoxedSender;
use crate::runtime::{InternalServerFn, InternalServerId};

pub enum BrokerEvent {
    EmptyEvent,
    ScheduleInternalServer(InternalServerId, InternalServerFn),
    InernalServerScheduled(InternalServerId, BoxedSender),
}
