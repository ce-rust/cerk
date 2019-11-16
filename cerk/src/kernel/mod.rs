mod broker_event;
use crate::runtime::channel::{BoxedReceiver, BoxedSender};
use crate::runtime::{InternalServerFn, InternalServerId, ScheduFn};
use std::collections::HashMap;

pub use crate::kernel::broker_event::BrokerEvent;

struct Kernel {}

impl Kernel {
    fn start(start_options: StartOptions, inbox: BoxedReceiver, sender_to_scheduler: BoxedSender) {
        let mut outboxes = HashMap::<InternalServerId, BoxedSender>::new();

        sender_to_scheduler.send(BrokerEvent::ScheduleInternalServer(
            "router",
            start_options.router_start,
        ));
        sender_to_scheduler.send(BrokerEvent::ScheduleInternalServer(
            "config_loader",
            start_options.config_loader_start,
        ));

        for (id, port_start) in start_options.ports.iter() {
            sender_to_scheduler.send(BrokerEvent::ScheduleInternalServer(id, *port_start));
        }

        loop {
            match inbox.receive() {
                BrokerEvent::InernalServerScheduled(id, sender_to_server) => {
                    outboxes.insert(id, sender_to_server);
                }
                _ => println!("event not implemented"),
            }
        }
    }
}

pub type KernelFn = fn(StartOptions, BoxedReceiver, BoxedSender);

pub struct StartOptions {
    pub scheduler_start: ScheduFn,
    pub router_start: InternalServerFn,
    pub config_loader_start: InternalServerFn,
    pub ports: Box<[(InternalServerId, InternalServerFn)]>,
}

pub fn start_kernel(start_options: StartOptions) {
    (start_options.scheduler_start)(start_options, Kernel::start);
}
