use strum_macros::Display;

/// result of the processing of the send attempt
#[derive(Display)]
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
}
