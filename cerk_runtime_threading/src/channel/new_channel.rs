use super::{ThreadingReceiver, ThreadingSender};
use crate::channel::sender::ThreadingKernelSender;
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use std::sync::mpsc::{channel, sync_channel};

/// Create a new channel with a `ThreadingSender` and a `ThreadingReceiver`.
/// The implementation is based on `std::sync::mpsc` sync_channel model.
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

/// Create a new channel with a `ThreadingSender` and a `ThreadingReceiver`.
/// The implementation is based on `std::sync::mpsc` channel model.
///
/// This channel has an "infinite buffer" and should only be used to send message from the kernel to other components, so that the kernel is never blocked.
///
pub fn new_channel_kernel_to_component() -> (BoxedSender, BoxedReceiver) {
    let (tx, rx) = channel();
    return (
        Box::new(ThreadingKernelSender::new(tx)),
        Box::new(ThreadingReceiver::new(rx)),
    );
}
