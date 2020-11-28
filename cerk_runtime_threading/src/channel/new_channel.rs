use super::{ThreadingReceiver, ThreadingSender};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use std::sync::mpsc::sync_channel;

/// Create a new channel with a `ThreadingSender` and a `ThreadingReceiver`.
/// The implementation is based on `std::sync::mpsc` channel model.
/// The channel has an internal buffer of 50 messages on which messages will be queued.
///
/// # Arguments
///
/// * `bound` the size of the buffer of the underlying channel in which messages will be queued.
///
pub fn new_channel_with_size(bound: usize) -> (BoxedSender, BoxedReceiver) {
    let (tx, rx) = sync_channel(bound);
    return (
        Box::new(ThreadingSender::new(tx)),
        Box::new(ThreadingReceiver::new(rx)),
    );
}

/// crate a new channel; same as `new_channel_with_size` but with a predefined buffer size of 50.
pub fn new_channel() -> (BoxedSender, BoxedReceiver) {
    new_channel_with_size(50) // todo set with configs
}
