use crate::kernel::BrokerEvent;

pub trait Sender: Send {
    fn send(&self, event: BrokerEvent);
}
