#[macro_use]
extern crate log;
use cerk::kernel::BrokerEvent;
use cerk::runtime::InternalServerId;
use cerk::runtime::{BoxedReceiver, BoxedSender};

pub fn start_routing(id: InternalServerId, inbox: BoxedReceiver, sender_to_kernel: BoxedSender) {
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
