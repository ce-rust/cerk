use super::channel::new_channel;
use cerk::kernel::{BrokerEvent, KernelFn, StartOptions};
use cerk::runtime::channel::BoxedSender;
use cerk::runtime::{InternalServerFn, InternalServerId};
use std::thread;

/// This is the main function to start the scheduler.
///
/// This function gets invoked in the `bootstrap` function in the start phase of CERK.
pub fn threading_scheduler_start(start_options: StartOptions, start_kernel: KernelFn) {
    info!("start threading scheduler");

    let (sender_to_scheduler, receiver_from_kernel) = new_channel();
    let (sender_to_kernel, receiver_from_scheduler) = new_channel();

    thread::spawn(move || {
        start_kernel(start_options, receiver_from_scheduler, sender_to_scheduler);
    });

    loop {
        let event = receiver_from_kernel.receive();
        match event {
            BrokerEvent::ScheduleInternalServer(id, internal_server) => {
                schedule(id, internal_server, &sender_to_kernel)
            }
            _ => warn!("Unknown event"),
        }
    }
}

fn schedule(
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
