use crate::kernel::BrokerEvent;

/// Wrapper for a platform-specific channel sender.
pub trait Sender: Send {
    fn send(&self, event: BrokerEvent);
    fn clone_boxed(&self) -> Box<dyn Sender>; // TODO: make it less ugly (real Clone maybe)
}

/// Boxed wrapper for a platform-specific channel sender.
pub type BoxedSender = Box<dyn Sender>;
