use strum_macros::Display;

/// result of the processing of the send attempt
#[derive(Display, Clone, Debug, PartialEq)]
pub enum ProcessingResult {
    /// Sending was successful
    ///
    /// Depending on the delivery guarantee that could mean:
    /// * the event was sent and acknowledged
    /// * the event was probably sent
    /// * the event was not sent, but that's still okay
    Successful,

    /// Sending was not successful, or may not successful (may a lost acknowledgment).
    ///
    /// Depending on the delivery guarantee, that means that the transmission could be retried.
    TransientError,

    /// The send action was not successful.
    /// However, the error is permanent (e.g., parsing or config error) and should not be retried.
    PermanentError,

    /// The send action was not responded by all components in the given time.
    /// The kernel canceled the routing.
    /// At the moment there is no guarantee that the timout will be sent after a certain time, it is only to prevent the tracking table to grow.
    Timeout,
}

/// from result to ProcessingResult; for every error a `ProcessingResult::PermanentError` is used
impl<T, E> From<Result<T, E>> for ProcessingResult {
    fn from(r: Result<T, E>) -> Self {
        if r.is_ok() {
            ProcessingResult::Successful
        } else {
            ProcessingResult::PermanentError
        }
    }
}
