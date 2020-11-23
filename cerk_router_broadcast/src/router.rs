use cerk::kernel::{
    BrokerEvent, CloudEventMessageRoutingId, CloudEventRoutingArgs, Config, OutgoingCloudEvent,
    RoutingResult,
};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cloudevents::Event;

fn route_event(
    incoming_port: InternalServerId,
    sender_to_kernel: &BoxedSender,
    port_ids: &Vec<Config>,
    event_id: CloudEventMessageRoutingId,
    cloud_event: Event,
    args: CloudEventRoutingArgs,
) {
    let routing: Vec<_> = port_ids
        .iter()
        .filter_map(|port_id| match port_id {
            Config::String(port_id) => Some(OutgoingCloudEvent {
                routing_id: event_id.clone(),
                cloud_event: cloud_event.clone(),
                destination_id: port_id.clone(),
                args: args.clone(),
            }),
            _ => {
                error!("No valid routing config found, message could not be routed!");
                None
            }
        })
        .collect();

    sender_to_kernel.send(BrokerEvent::RoutingResult(RoutingResult {
        routing_id: event_id,
        incoming_id: incoming_port,
        routing,
        args,
    }))
}

/// This is the main function to start the router.
pub fn router_start(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    info!("start broadcast router with id {}", id);
    let mut config: Config = Config::Null;
    loop {
        match inbox.receive() {
            BrokerEvent::Init => info!("{} initiated", id),
            BrokerEvent::IncomingCloudEvent(event) => match &config {
                Config::Vec(port_ids) => route_event(
                    event.incoming_id,
                    &sender_to_kernel,
                    &port_ids,
                    event.routing_id,
                    event.cloud_event,
                    event.args,
                ),
                _ => error!("No valid routing config found, message could not be routed!"),
            },
            BrokerEvent::ConfigUpdated(updated_config, _) => config = updated_config,
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}
