use cerk::kernel::BrokerEvent;
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use serde_json;

/// This port prints the CloudEvent id to the logger.
/// This port is for testing!
///
/// # Examples
///
/// * [Hello World Example](https://github.com/ce-rust/cerk/tree/master/examples/src/hello_world)
/// * [Generator to MQTT](https://github.com/ce-rust/cerk/tree/master/examples/src/mqtt/)
///
pub fn port_printer_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    _sender_to_kernel: BoxedSender,
) {
    info!("start printer port with id {}", id);
    loop {
        match inbox.receive() {
            BrokerEvent::Init => info!("{} initiated", id),
            BrokerEvent::ConfigUpdated(_, _) => info!("{} received ConfigUpdated", id),
            BrokerEvent::OutgoingCloudEvent(_, cloud_event, _, _) => info!(
                "{} received cloud event: {}!",
                id,
                serde_json::to_string(&cloud_event).unwrap(),
            ),
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}
