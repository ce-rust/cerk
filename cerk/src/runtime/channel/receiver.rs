use crate::kernel::BrokerEvent;

/// Wrapper for a platform-specific channel receiver.
pub trait Receiver: Send {
    fn receive(&self) -> BrokerEvent;
}

/// Boxed wrapper for a platform-specific channel receiver.
pub type BoxedReceiver = Box<dyn Receiver>;
