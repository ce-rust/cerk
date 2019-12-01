use super::{BrokerEvent, StartOptions};
use crate::runtime::channel::{BoxedReceiver, BoxedSender};
use crate::runtime::InternalServerId;
use std::collections::HashMap;

const ROUTER_ID: &str = "router";
const CONFIG_LOADER_ID: &str = "config_loader";

type Outboxes = HashMap<InternalServerId, BoxedSender>;

fn process_broker_event(
    broker_event: BrokerEvent,
    outboxes: &mut Outboxes,
    number_of_servers: usize,
) {
    match broker_event {
        BrokerEvent::InternalServerScheduled(id, sender_to_server) => {
            init_internal_server(outboxes, number_of_servers, id, sender_to_server);
        }
        broker_event @ BrokerEvent::IncommingCloudEvent(_, _) => {
            outboxes
                .get(&String::from(ROUTER_ID))
                .unwrap()
                .send(broker_event) // if the router is not present: panic! we cant work without it
        }
        BrokerEvent::OutgoingCloudEvent(cloud_event, destionation_server_id) => {
            debug!(
                "received OutgoingCloudEvent, forward to {}",
                destionation_server_id
            );
            outboxes
                .get(&destionation_server_id)
                .unwrap()
                .send(BrokerEvent::OutgoingCloudEvent(
                    cloud_event,
                    destionation_server_id,
                ));
        }
        BrokerEvent::ConfigUpdated(config, destionation_server_id) => {
            debug!(
                "received ConfigUpdated, forward to {}",
                destionation_server_id
            );
            outboxes
                .get(&destionation_server_id)
                .unwrap()
                .send(BrokerEvent::ConfigUpdated(config, destionation_server_id));
        }
        BrokerEvent::Batch(broker_events) => {
            for broker_event in broker_events.into_iter() {
                process_broker_event(broker_event, outboxes, number_of_servers);
            }
        }
        broker_event => warn!("event {} not implemented", broker_event),
    }
}

fn init_internal_server(
    outboxes: &mut Outboxes,
    number_of_servers: usize,
    id: InternalServerId,
    sender_to_server: BoxedSender,
) {
    outboxes.insert(id, sender_to_server);
    if outboxes.len() == number_of_servers {
        for (_, outbox) in outboxes.iter() {
            outbox.send(BrokerEvent::Init);
        }
    }
}

/// The function that gets started from the scheduler.
/// It implements the Kernel.
pub fn kernel_start(
    start_options: StartOptions,
    inbox: BoxedReceiver,
    sender_to_scheduler: BoxedSender,
) {
    let mut outboxes = Outboxes::new();

    sender_to_scheduler.send(BrokerEvent::ScheduleInternalServer(
        String::from(ROUTER_ID),
        start_options.router_start,
    ));
    sender_to_scheduler.send(BrokerEvent::ScheduleInternalServer(
        String::from(CONFIG_LOADER_ID),
        start_options.config_loader_start,
    ));

    for (id, port_start) in start_options.ports.iter() {
        sender_to_scheduler.send(BrokerEvent::ScheduleInternalServer(id.clone(), *port_start));
    }
    let number_of_servers = 2 + start_options.ports.len(); // 2 = router + config_loader

    loop {
        let broker_event = inbox.receive();
        process_broker_event(broker_event, &mut outboxes, number_of_servers);
    }
}
