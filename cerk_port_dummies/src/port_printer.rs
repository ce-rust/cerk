use cerk::kernel::BrokerEvent;
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;

/// This port prints the CloudEvent id to the logger.
/// This port is for testing!
///
/// # Examples
///
/// * [Hello World Example](https://github.com/ce-rust/cerk/tree/master/examples/src/hello_world)
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
            BrokerEvent::OutgoingCloudEvent(cloud_event, _) => info!(
                "{} received cloud event with id={}!",
                id,
                cloud_event.event_id()
            ),
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}
