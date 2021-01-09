use super::channel::{new_channel_kernel_to_component, new_channel_with_size};
use cerk::kernel::{BrokerEvent, KernelFn, StartOptions};
use cerk::runtime::channel::BoxedSender;
use cerk::runtime::{InternalServerFnRefStatic, InternalServerId, ScheduleFn, ScheduleFnRefStatic};
use std::thread;

const CHANNEL_TO_KERNEL_SIZE: usize = 50;

fn schedule(
    id: InternalServerId,
    internal_server_fn: InternalServerFnRefStatic,
    sender_to_kernel: &BoxedSender,
) {
    debug!("schedule {} thread", id);
    let (sender_to_server, receiver_from_kernel) = new_channel_kernel_to_component();
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

/// This is the main function to start the scheduler.
///
/// This function gets invoked in the `bootstrap` function in the start phase of CERK.
pub fn threading_scheduler_start(start_options: StartOptions, start_kernel: KernelFn) {
    info!("start threading scheduler");

    let (sender_to_scheduler, receiver_from_kernel) = new_channel_kernel_to_component();
    let (sender_to_kernel, receiver_from_scheduler) = new_channel_with_size(CHANNEL_TO_KERNEL_SIZE);

    thread::spawn(move || {
        start_kernel(start_options, receiver_from_scheduler, sender_to_scheduler);
    });

    loop {
        let event = receiver_from_kernel.receive();
        match event {
            BrokerEvent::ScheduleInternalServer(event) => {
                schedule(event.id, event.function, &sender_to_kernel)
            }
            _ => warn!("Unknown event"),
        }
    }
}

/// This is the pointer for the main function to start the scheduler.
pub static THREADING_SCHEDULER: ScheduleFnRefStatic = &(threading_scheduler_start as ScheduleFn);
