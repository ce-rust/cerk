use anyhow::Result;
use cerk::kernel::{
    BrokerEvent, Config, IncomingCloudEvent, OutgoingCloudEvent, ProcessingResult, RoutingResult,
};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerFnRefStatic, InternalServerId};
use std::convert::TryFrom;

fn route_event(
    sender_to_kernel: &BoxedSender,
    config: &Config,
    event: &IncomingCloudEvent,
) -> Result<()> {
    let routing: Vec<_> = Vec::<Config>::try_from(config)?
        .iter()
        .filter_map(|port_id| match port_id {
            Config::String(port_id) => Some(OutgoingCloudEvent {
                routing_id: event.routing_id.clone(),
                cloud_event: event.cloud_event.clone(),
                destination_id: port_id.clone(),
                args: event.args.clone(),
            }),
            _ => {
                error!("No valid routing config found, message could not be routed!");
                None
            }
        })
        .collect();

    sender_to_kernel.send(BrokerEvent::RoutingResult(RoutingResult {
        routing_id: event.routing_id.clone(),
        incoming_id: event.incoming_id.clone(),
        routing,
        args: event.args.clone(),
        result: ProcessingResult::Successful,
    }));
    Ok(())
}

/// This is the main function to start the router.
pub fn router_start(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    info!("start broadcast router with id {}", id);
    let mut config: Config = Config::Null;
    loop {
        match inbox.receive() {
            BrokerEvent::Init => info!("{} initiated", id),
            BrokerEvent::IncomingCloudEvent(event) => {
                if let Err(e) = route_event(&sender_to_kernel, &config, &event) {
                    error!("failed to rout message! {:?}", e);
                    sender_to_kernel.send(BrokerEvent::RoutingResult(RoutingResult {
                        result: ProcessingResult::PermanentError,
                        incoming_id: event.incoming_id,
                        routing: vec![],
                        routing_id: event.routing_id,
                        args: event.args,
                    }));
                }
            }
            BrokerEvent::ConfigUpdated(updated_config, _) => config = updated_config,
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

/// This is the pointer for the main function to start the router.
pub static ROUTER_BROADCAST: InternalServerFnRefStatic = &(router_start as InternalServerFn);
