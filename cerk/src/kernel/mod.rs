mod broker_event;
mod cloud_event;
use crate::runtime::channel::{BoxedReceiver, BoxedSender};
use crate::runtime::{InternalServerFn, InternalServerId, ScheduFn};
use std::collections::HashMap;

pub use crate::kernel::broker_event::BrokerEvent;
pub use crate::kernel::cloud_event::CloudEvent;

const ROUTER_ID: InternalServerId = "router";
const CONFIG_LOADER_ID: InternalServerId = "config_loader";

fn kernel_start(
    start_options: StartOptions,
    inbox: BoxedReceiver,
    sender_to_scheduler: BoxedSender,
) {
    let mut outboxes = HashMap::<InternalServerId, BoxedSender>::new();

    sender_to_scheduler.send(BrokerEvent::ScheduleInternalServer(
        ROUTER_ID,
        start_options.router_start,
    ));
    sender_to_scheduler.send(BrokerEvent::ScheduleInternalServer(
        CONFIG_LOADER_ID,
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
            broker_event @ BrokerEvent::IncommingCloudEvent(_, _) => {
                outboxes.get(ROUTER_ID).unwrap().send(broker_event) // if the router is not present: panic! we cant work without it
            }
            BrokerEvent::OutgoingCloudEvent(cloud_event, destionation_server_id) => {
                debug!("got outgoing event, forward to {}", destionation_server_id);
                outboxes.get(destionation_server_id).unwrap().send(
                    BrokerEvent::OutgoingCloudEvent(cloud_event, destionation_server_id),
                );
            }
            broker_event => warn!("event {} not implemented", broker_event),
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

pub fn bootstrap(start_options: StartOptions) {
    (start_options.scheduler_start)(start_options, kernel_start);
}
