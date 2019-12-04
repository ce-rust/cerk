use super::{ThreadingReceiver, ThreadingSender};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use std::sync::mpsc::sync_channel;

/// Create a new chanel with a `ThreadingSender` and a `ThreadingReceiver`.
/// The implementation is based on `std::sync::mpsc` chanel model.
/// The channel has an internal buffer of 50 messages on which messages will be queued.
pub fn new_channel() -> (BoxedSender, BoxedReceiver) {
    let (tx, rx) = sync_channel(50); // todo set with configs
    return (
        Box::new(ThreadingSender::new(tx)),
        Box::new(ThreadingReceiver::new(rx)),
    );
}
