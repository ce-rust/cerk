use cerk::kernel::BrokerEvent;
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cloudevents::CloudEvent;

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
                match cloud_event {
                    CloudEvent::V0_2(ref event) => event.event_id(),
                    CloudEvent::V1_0(ref event) => event.event_id(),
                }
            ),
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}
