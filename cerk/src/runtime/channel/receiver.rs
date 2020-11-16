use crate::kernel::BrokerEvent;
use std::time::Duration;

/// Wrapper for a platform-specific channel receiver.
pub trait Receiver: Send {
    /// Messages sent to the channel can be retrieved using this function.
    /// The call is blocking.
    fn receive(&self) -> BrokerEvent;

    /// Messages sent to the channel can be retrieved using this function.
    /// The call is blocking for a given duration.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The maximum time to block the caller and wait for a message.
    ///
    fn receive_timeout(&self, timeout: Duration) -> Option<BrokerEvent>;
}

/// Boxed wrapper for a platform-specific channel receiver.
pub type BoxedReceiver = Box<dyn Receiver + Send>;
