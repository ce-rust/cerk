#[macro_use]
extern crate log;
use cerk::kernel::{BrokerEvent, CloudEvent};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use std::{thread, time};

pub fn port_printer_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start printer port");
}
pub fn port_sequence_generator_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start sequence generator port");
    for i in 1.. {
        debug!("send dummy event with sequence number {} to kernel", i);
        sender_to_kernel.send(BrokerEvent::IncommingCloudEvent(
            id,
            CloudEvent {
                id: format!("{}", i),
                source: String::from("dummy"),
            },
        ));
        thread::sleep(time::Duration::from_secs(1));
    }
}
