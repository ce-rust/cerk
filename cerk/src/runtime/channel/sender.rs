use crate::kernel::BrokerEvent;

pub trait Sender: Send {
    fn send(&self, event: BrokerEvent);
    fn clone_boxed(&self) -> Box<dyn Sender>; // TODO: make it less ugly (real Clone maybe)
}
