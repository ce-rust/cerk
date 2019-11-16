use cerk::kernel::BrokerEvent;
use cerk::runtime::channel::Sender as CerkSender;
use std::sync::mpsc::SyncSender;

pub struct ThreadingSender {
    sender: SyncSender<BrokerEvent>,
}

impl ThreadingSender {
    pub fn new(sender: SyncSender<BrokerEvent>) -> Self {
        return ThreadingSender { sender: sender };
    }
}

impl CerkSender for ThreadingSender {
    fn send(&self, event: BrokerEvent) {
        self.sender.send(event).unwrap();
    }
}
