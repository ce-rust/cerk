use cerk::kernel::{BrokerEvent, Config};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cloudevents::v10::CloudEvent;

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

/// This router broadcasts all received CloudEvents to the configured ports.
///
/// # Configurations
///
/// The Socket expects a `Config::Vec([Config::String])` as configuration.
/// The strings should be Port ids, to which all received CloudEvents should be forwarded to.
///
/// e.g. `Config::Vec(vec![Config::String(String::from("output-port"))])`
///
/// # Examples
///
/// * [Hello World Example](https://github.com/ce-rust/cerk/tree/master/examples/src/hello_world)
/// * [UNIX Socket Example](https://github.com/ce-rust/cerk/tree/master/examples/src/unix_socket)
///
pub fn router_start(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    info!("start broadcast router with id {}", id);
    let mut config: Config = Config::Null;
    loop {
        match inbox.receive() {
            BrokerEvent::Init => info!("{} initiated", id),
            BrokerEvent::IncommingCloudEvent(_, cloud_event) => match &config {
                Config::Vec(port_ids) => route_event(&sender_to_kernel, &port_ids, cloud_event),
                _ => error!("No valid routing config found, message could not be routed!"),
            },
            BrokerEvent::ConfigUpdated(updated_config, _) => config = updated_config,
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}
