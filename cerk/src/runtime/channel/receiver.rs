use crate::kernel::BrokerEvent;

/// Wrapper for a platform-specific channel receiver.
pub trait Receiver: Send {
    /// Messages sent to the channel can be retrieved using this function.
    /// The call is blocking.
    fn receive(&self) -> BrokerEvent;
}

/// Boxed wrapper for a platform-specific channel receiver.
pub type BoxedReceiver = Box<dyn Receiver>;
