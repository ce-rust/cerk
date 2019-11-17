use cerk::kernel::{BrokerEvent, CloudEvent, Config};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;

fn route_event(sender_to_kernel: &BoxedSender, port_ids: &Vec<Config>, cloud_event: CloudEvent) {
    for port_id in port_ids.iter() {
        match port_id {
            Config::String(port_id) => sender_to_kernel.send(BrokerEvent::OutgoingCloudEvent(
                cloud_event.clone(),
                port_id.clone(),
            )),
            _ => error!("No valid routing config found, message could not be routed!"),
        }
    }
}

pub fn router_start(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    info!("start broadcast router with id {}", id);
    let mut config: Config = Config::Null;
    loop {
        match inbox.receive() {
            BrokerEvent::Init => info!("{} initiated", id),
            BrokerEvent::IncommingCloudEvent(_, cloud_event) => match &config {
                Config::Array(port_ids) => route_event(&sender_to_kernel, &port_ids, cloud_event),
                _ => error!("No valid routing config found, message could not be routed!"),
            },
            BrokerEvent::ConfigUpdated(updated_config, _) => config = updated_config,
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}
