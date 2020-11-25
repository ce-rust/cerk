//! Implementation of the Kernel

use super::{BrokerEvent, StartOptions};
use crate::kernel::broker_event::{
    OutgoingCloudEventProcessed, RoutingResult, ScheduleInternalServer,
};
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

fn process_routing_result(
    event: RoutingResult,
    outboxes: &mut Outboxes,
    pending_deliveries: &mut PendingDeliveries,
) {
    let RoutingResult {
        routing_id,
        routing,
        incoming_id: receiver_id,
        args,
    } = event;
    debug!("received RoutingResult for event_id={}", &routing_id);

    if routing.is_empty() {
        debug!("routing is empty - nothing to do");
    } else {
        if args.delivery_guarantee.requires_acknowledgment() {
            let missing_receivers: Vec<_> = routing
                .iter()
                .map(|event| event.destination_id.clone())
                .collect();

            if pending_deliveries
                .insert(
                    routing_id.clone(),
                    PendingDelivery {
                        sender: receiver_id,
                        missing_receivers,
                    },
                )
                .is_some()
            {
                error!(
                    "a routing for event_id={} already existed, the old one was overwritten",
                    &routing_id
                );
            }
        } else {
            debug!("no acknowledgments needed for event_id={}", &routing_id)
        }

        for subevent in routing {
            outboxes
                .get(&subevent.destination_id)
                .unwrap()
                .send(BrokerEvent::OutgoingCloudEvent(subevent));
        }
        debug!("all routings sent for event_id={}", routing_id);
    }
}

fn process_outgoing_cloud_event_processed(
    event: OutgoingCloudEventProcessed,
    outboxes: &mut Outboxes,
    pending_deliveries: &mut PendingDeliveries,
) {
    let OutgoingCloudEventProcessed {
        routing_id,
        sender_id,
        result,
    } = event;
    debug!(
        "received OutgoingCloudEventProcessed from={} event_id={}",
        sender_id, routing_id
    );
    let mut resolved_missing_delivery = false;
    if let Some(delivery) = pending_deliveries.get_mut(&routing_id) {
        match result {
            ProcessingResult::Successful => {
                let size_before = delivery.missing_receivers.len();
                delivery.missing_receivers.retain(|i| !i.eq(&sender_id));
                let size = delivery.missing_receivers.len();
                if size == 0 {
                    debug!("delivery for event_id={} was successful (all out port processing were successful) -> ack to sender", routing_id);
                    outboxes.get(&delivery.sender).unwrap().send(
                        BrokerEvent::IncomingCloudEventProcessed(routing_id.clone(), result),
                    );
                    resolved_missing_delivery = true
                } else if size_before == size {
                    warn!("{} sent OutgoingCloudEventProcessed for event_id={}, but was not expected to send this", sender_id, routing_id);
                }
            }
            _ => {
                if delivery.missing_receivers.contains(&sender_id) {
                    debug!("delivery for event_id={} was NOT successful ({}) -> immediately notify the sender", routing_id, result);
                    outboxes.get(&delivery.sender).unwrap().send(
                        BrokerEvent::IncomingCloudEventProcessed(routing_id.clone(), result),
                    );
                    resolved_missing_delivery = true
                } else {
                    warn!("{} sent OutgoingCloudEventProcessed for event_id={}, but no response was expected", sender_id, routing_id);
                }
            }
        }
    } else {
        debug!("there was no pending delivery for event_id {}", routing_id);
    }

    if resolved_missing_delivery {
        if pending_deliveries.remove_entry(&routing_id).is_none() {
            warn!(
                "failed to delete pending_deliveries for event_id={}",
                routing_id
            );
        }
    }
}

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
        broker_event @ BrokerEvent::IncomingCloudEvent(_) => {
            outboxes
                .get(&String::from(ROUTER_ID))
                .unwrap()
                .send(broker_event) // if the router is not present: panic! we cant work without it
        }
        BrokerEvent::RoutingResult(event) => {
            process_routing_result(event, outboxes, pending_deliveries)
        }
        BrokerEvent::OutgoingCloudEventProcessed(event) => {
            process_outgoing_cloud_event_processed(event, outboxes, pending_deliveries)
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
    // todo this list could grow and entries could potentially be there for ever - TTL at kernel level?
    let mut pending_deliveries = PendingDeliveries::new();

    sender_to_scheduler.send(BrokerEvent::ScheduleInternalServer(
        ScheduleInternalServer {
            id: String::from(ROUTER_ID),
            function: start_options.router,
        },
    ));
    sender_to_scheduler.send(BrokerEvent::ScheduleInternalServer(
        ScheduleInternalServer {
            id: String::from(CONFIG_LOADER_ID),
            function: start_options.config_loader,
        },
    ));

    for service in start_options.ports.iter() {
        sender_to_scheduler.send(BrokerEvent::ScheduleInternalServer(service.clone()));
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
