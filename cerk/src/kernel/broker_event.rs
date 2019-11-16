use crate::runtime::channel::BoxedSender;
use crate::runtime::InternalServerFn;

pub enum BrokerEvent {
    EmptyEvent,
    ScheduleInternalServer(InternalServerFn),
    InernalServerScheduled(BoxedSender),
}
