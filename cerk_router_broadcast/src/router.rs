use cerk::kernel::BrokerEvent;
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;

pub fn router_start(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
    info!("start broadcast router with id {}", id);
    loop {
        match inbox.receive() {
            BrokerEvent::IncommingCloudEvent(_, cloud_event) => sender_to_kernel.send(
                BrokerEvent::OutgoingCloudEvent(cloud_event, "dummy-logger-output"),
            ),
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}
