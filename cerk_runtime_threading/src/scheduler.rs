use super::channel::new_channel;
use cerk::kernel::{BrokerEvent, KernelFn, StartOptions};
use cerk::runtime::channel::BoxedSender;
use cerk::runtime::{InternalServerFn, InternalServerId};
use std::thread;

pub struct ThreadingScheduler {}

impl ThreadingScheduler {
    fn new() -> Self {
        ThreadingScheduler {}
    }
    pub fn start(start_options: StartOptions, start_kernel: KernelFn) {
        info!("start threading scheduler");
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
        debug!("schedule {} thread", id);
        let (sender_to_server, receiver_from_kernel) = new_channel();
        let server_sender_to_kernel = sender_to_kernel.clone_boxed();
        let new_server_id = id.clone();
        thread::spawn(move || {
            internal_server_fn(new_server_id, receiver_from_kernel, server_sender_to_kernel);
        });
        sender_to_kernel.send(BrokerEvent::InternalServerScheduled(
            id.clone(),
            sender_to_server,
        ));
    }
}
