use cerk::kernel::BrokerEvent;
use cerk::runtime::channel::Sender as CerkSender;
use std::sync::mpsc::{Sender, SyncSender};

/// Implementation for `cerk::runtime::channel::Sender` that uses `std::sync::mpsc::SyncSender` channel sender.
pub struct ThreadingSender {
    sender: SyncSender<BrokerEvent>,
}

impl ThreadingSender {
    #[allow(missing_docs)]
    pub fn new(sender: SyncSender<BrokerEvent>) -> Self {
        return ThreadingSender { sender };
    }
}

impl CerkSender for ThreadingSender {
    fn send(&self, event: BrokerEvent) {
        self.sender.send(event).unwrap();
    }
    fn clone_boxed(&self) -> Box<dyn CerkSender + Send> {
        Box::new(ThreadingSender {
            sender: self.sender.clone(),
        })
    }
}

/// Implementation for `cerk::runtime::channel::Sender` that uses `std::sync::mpsc::Sender` channel sender.
pub struct ThreadingKernelSender {
    sender: Sender<BrokerEvent>,
}

impl ThreadingKernelSender {
    #[allow(missing_docs)]
    pub fn new(sender: Sender<BrokerEvent>) -> Self {
        return ThreadingKernelSender { sender };
    }
}

impl CerkSender for ThreadingKernelSender {
    fn send(&self, event: BrokerEvent) {
        self.sender.send(event).unwrap();
    }
    fn clone_boxed(&self) -> Box<dyn CerkSender + Send> {
        Box::new(ThreadingKernelSender {
            sender: self.sender.clone(),
        })
    }
}
