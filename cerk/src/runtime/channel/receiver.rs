use crate::kernel::BrokerEvent;

pub trait Receiver: Send {
    fn receive(&self) -> BrokerEvent;
}

pub type BoxedReceiver = Box<dyn Receiver>;
