use cerk::kernel::BrokerEvent;
use cerk::runtime::channel::Receiver as CerkReceiver;
use std::sync::mpsc::Receiver;

pub struct ThreadingReceiver {
    receiver: Receiver<BrokerEvent>,
}

impl ThreadingReceiver {
    pub fn new(receiver: Receiver<BrokerEvent>) -> Self {
        ThreadingReceiver { receiver: receiver }
    }
}

impl CerkReceiver for ThreadingReceiver {
    fn receive(&self) -> BrokerEvent {
        self.receiver.recv().unwrap()
    }
}
