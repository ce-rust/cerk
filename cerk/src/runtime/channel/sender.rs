use crate::kernel::BrokerEvent;

pub trait Sender: Send + Sync {
    fn send(&self, event: BrokerEvent);
}
