use cerk::kernel::BrokerEvent;
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;

pub fn port_printer_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    _sender_to_kernel: BoxedSender,
) {
    info!("start printer port with id {}", id);
    loop {
        match inbox.receive() {
            BrokerEvent::OutgoingCloudEvent(cloud_event, _) => {
                info!("{} received cloud event with id={}!", id, cloud_event.id)
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}
