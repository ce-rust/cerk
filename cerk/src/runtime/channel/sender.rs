use crate::kernel::BrokerEvent;

/// Wrapper for a platform-specific channel sender.
pub trait Sender: Send {
    /// send a BrokerEvent to the chanel receiver
    fn send(&self, event: BrokerEvent);

    /// clones a sender and returns new boxed instance
    ///
    /// # open issues
    ///
    /// * https://github.com/ce-rust/cerk/issues/21
    fn clone_boxed(&self) -> Box<dyn Sender + Send>;
}

/// Boxed wrapper for a platform-specific channel sender.
pub type BoxedSender = Box<dyn Sender + Send>;
