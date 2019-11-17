use super::{BrokerEvent, StartOptions};
use crate::runtime::channel::{BoxedReceiver, BoxedSender};
use crate::runtime::InternalServerId;
use std::collections::HashMap;

const ROUTER_ID: InternalServerId = "router";
const CONFIG_LOADER_ID: InternalServerId = "config_loader";

pub fn kernel_start(
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
