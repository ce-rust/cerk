mod receiver;
mod sender;

use crate::receiver::ThreadingReceiver;
use crate::sender::ThreadingSender;
use cerk::kernel::{KernelFn, StartOptions};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;

pub struct ThreadingScheduler {}

fn new_channel() -> (ThreadingSender, ThreadingReceiver) {
    let (tx, rx) = sync_channel(50); // todo set with configs
    return (ThreadingSender::new(tx), ThreadingReceiver::new(rx));
}

impl ThreadingScheduler {
    fn new() -> Self {
        ThreadingScheduler {}
    }
    pub fn start(start_options: StartOptions, start_kernel: KernelFn) {
        let mut scheduler = ThreadingScheduler::new();
        scheduler.init(start_options, start_kernel);
    }

    fn init(&mut self, start_options: StartOptions, start_kernel: KernelFn) {
        let (kernel_sender, scheduler_receiver) = new_channel();
        let (scheduler_sender, kernel_receiver) = new_channel();

        thread::spawn(move || {
            start_kernel(
                start_options,
                Box::new(kernel_receiver),
                Box::new(kernel_sender),
            );
        });
    }
}
