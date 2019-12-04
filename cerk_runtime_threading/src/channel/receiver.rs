use cerk::kernel::BrokerEvent;
use cerk::runtime::channel::Receiver as CerkReceiver;
use std::sync::mpsc::Receiver;

/// Implementation for `cerk::runtime::channel::Receiver` that uses `std::sync::mpsc::SyncSender` channel sender.
pub struct ThreadingReceiver {
    receiver: Receiver<BrokerEvent>,
}

impl ThreadingReceiver {
    #[allow(missing_docs)]
    pub fn new(receiver: Receiver<BrokerEvent>) -> Self {
        ThreadingReceiver { receiver: receiver }
    }
}

impl CerkReceiver for ThreadingReceiver {
    fn receive(&self) -> BrokerEvent {
        self.receiver.recv().unwrap()
    }
}
