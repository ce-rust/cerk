//! Implementation of the Kernel

use super::{BrokerEvent, StartOptions};
use crate::kernel::{CloudEventMessageRoutingId, ProcessingResult};
use crate::runtime::channel::{BoxedReceiver, BoxedSender};
use crate::runtime::InternalServerId;
use std::collections::HashMap;

const ROUTER_ID: &str = "router";
const CONFIG_LOADER_ID: &str = "config_loader";

struct PendingDelivery {
    sender: InternalServerId,
    missing_receivers: Vec<InternalServerId>,
}

type Outboxes = HashMap<InternalServerId, BoxedSender>;
type PendingDeliveries = HashMap<CloudEventMessageRoutingId, PendingDelivery>;

fn process_broker_event(
    broker_event: BrokerEvent,
    outboxes: &mut Outboxes,
    number_of_servers: usize,
    pending_deliveries: &mut PendingDeliveries,
) {
    match broker_event {
        BrokerEvent::InternalServerScheduled(id, sender_to_server) => {
            init_internal_server(outboxes, number_of_servers, id, sender_to_server);
        }
        broker_event @ BrokerEvent::IncomingCloudEvent(_, _, _, _) => {
            outboxes
                .get(&String::from(ROUTER_ID))
                .unwrap()
                .send(broker_event) // if the router is not present: panic! we cant work without it
        }
        BrokerEvent::RoutingResult(event_id, incoming_port, outgoing_ports, args) => {
            debug!("received RoutingResult for event_id={}", &event_id);

            if outgoing_ports.is_empty() {
                debug!("routing is empty - nothing to do");
            } else {
                if args.delivery_guarantee.requires_acknowledgment() {
                    let missing_receivers: Vec<_> = outgoing_ports.iter().filter_map(|event| {
                        if let BrokerEvent::OutgoingCloudEvent(_,_, destination, _) = event {
                            Some(destination.clone())
                        }else{
                            error!("RoutingResult contained an event that is not of type OutgoingCloudEvent, but {}", event);
                            None
                        }
                    })
                        .collect();

                    if pending_deliveries
                        .insert(
                            event_id.clone(),
                            PendingDelivery {
                                sender: incoming_port,
                                missing_receivers,
                            },
                        )
                        .is_some()
                    {
                        error!("a routing for event_id={} already existed, the old one was overwritten", &event_id);
                    }
                } else {
                    debug!("no acknowledgments needed for event_id={}", &event_id)
                }

                for subevent in outgoing_ports {
                    if let BrokerEvent::OutgoingCloudEvent(
                        event_id,
                        cloud_event,
                        destination_server_id,
                        args,
                    ) = subevent
                    {
                        outboxes.get(&destination_server_id).unwrap().send(
                            BrokerEvent::OutgoingCloudEvent(
                                event_id,
                                cloud_event,
                                destination_server_id,
                                args,
                            ),
                        );
                    } else {
                        error!("RoutingResult contained an event that is not of type OutgoingCloudEvent, but {}", subevent);
                    }
                }
                debug!("all routings sent for event_id={}", event_id);
            }
        }
        BrokerEvent::OutgoingCloudEventProcessed(service_id, event_id, state) => {
            debug!(
                "received OutgoingCloudEventProcessed from={} event_id={}",
                service_id, event_id
            );
            if let Some(delivery) = pending_deliveries.get_mut(&event_id) {
                match state {
                    ProcessingResult::Successful => {
                        let size_before = delivery.missing_receivers.len();
                        delivery.missing_receivers.retain(|i| i.eq(&service_id));
                        let size = delivery.missing_receivers.len();
                        if size == 0 {
                            debug!("delivery for event_id={} was successful (all out port processing were successful) -> ack to sender", event_id);
                            outboxes
                                .get(&delivery.sender)
                                .unwrap()
                                .send(BrokerEvent::IncomingCloudEventProcessed(event_id, state));
                        } else if size_before == size {
                            warn!("{} sent OutgoingCloudEventProcessed for event_id={}, but was not expected to send this", service_id, event_id);
                        }
                    }
                    _ => {
                        if delivery.missing_receivers.contains(&service_id) {
                            debug!("delivery for event_id={} was NOT successful ({}) -> immediately notify the sender", event_id, state);
                            outboxes
                                .get(&delivery.sender)
                                .unwrap()
                                .send(BrokerEvent::IncomingCloudEventProcessed(event_id, state));
                        } else {
                            warn!("{} sent OutgoingCloudEventProcessed for event_id={}, but no response was expected", service_id, event_id);
                        }
                    }
                }
            } else {
                debug!("there was no pending delivery for event_id {}", event_id);
            }
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
                process_broker_event(
                    broker_event,
                    outboxes,
                    number_of_servers,
                    pending_deliveries,
                );
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
    let mut pending_deliveries = PendingDeliveries::new();

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
        process_broker_event(
            broker_event,
            &mut outboxes,
            number_of_servers,
            &mut pending_deliveries,
        );
    }
}
