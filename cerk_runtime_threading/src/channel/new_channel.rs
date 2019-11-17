use super::{ThreadingReceiver, ThreadingSender};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use std::sync::mpsc::sync_channel;

pub fn new_channel() -> (BoxedSender, BoxedReceiver) {
    let (tx, rx) = sync_channel(50); // todo set with configs
    return (
        Box::new(ThreadingSender::new(tx)),
        Box::new(ThreadingReceiver::new(rx)),
    );
}
