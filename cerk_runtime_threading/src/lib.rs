mod receiver;
mod sender;

use crate::receiver::ThreadingReceiver;
use crate::sender::ThreadingSender;
use cerk::kernel::{BrokerEvent, KernelFn, StartOptions};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerId};
use std::sync::mpsc::sync_channel;
use std::thread;

pub struct ThreadingScheduler {}

fn new_channel() -> (BoxedSender, BoxedReceiver) {
    let (tx, rx) = sync_channel(50); // todo set with configs
    return (
        Box::new(ThreadingSender::new(tx)),
        Box::new(ThreadingReceiver::new(rx)),
    );
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
        let (sender_to_scheduler, receiver_from_kernel) = new_channel();
        let (sender_to_kernel, receiver_from_scheduler) = new_channel();

        thread::spawn(move || {
            start_kernel(start_options, receiver_from_scheduler, sender_to_scheduler);
        });

        loop {
            let event = receiver_from_kernel.receive();
            match event {
                BrokerEvent::ScheduleInternalServer(id, internal_server) => {
                    self.schedule(id, internal_server, &sender_to_kernel)
                }
                _ => println!("Unknown event"),
            }
        }
    }

    fn schedule(
        &self,
        id: InternalServerId,
        internal_server_fn: InternalServerFn,
        sender_to_kernel: &BoxedSender,
    ) {
        let (sender_to_server, receiver_from_kernel) = new_channel();
        let server_sender_to_kernel = sender_to_kernel.clone_boxed();
        thread::spawn(move || {
            internal_server_fn(receiver_from_kernel, server_sender_to_kernel);
        });
        sender_to_kernel.send(BrokerEvent::InernalServerScheduled(id, sender_to_server));
    }
}
