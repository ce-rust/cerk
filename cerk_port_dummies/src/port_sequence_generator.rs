use cerk::kernel::{BrokerEvent, CloudEvent};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use std::{thread, time};

fn generate_events(id: InternalServerId, sender_to_kernel: BoxedSender) {
    for i in 1.. {
        debug!("send dummy event with sequence number {} to kernel", i);
        sender_to_kernel.send(BrokerEvent::IncommingCloudEvent(
            id.clone(),
            CloudEvent {
                id: format!("{}", i),
                source: String::from("dummy"),
            },
        ));
        thread::sleep(time::Duration::from_secs(1));
    }
}

pub fn port_sequence_generator_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start sequence generator port with id {}", id);
    loop {
        match inbox.receive() {
            BrokerEvent::Init => break,
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
    generate_events(id, sender_to_kernel);
}
