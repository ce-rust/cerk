use cerk::kernel::BrokerEvent;
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use chrono::Utc;
use cloudevents::{CloudEvent, Data};
use std::{thread, time};

fn generate_events(id: InternalServerId, sender_to_kernel: BoxedSender) {
    for i in 1.. {
        debug!("send dummy event with sequence number {} to kernel", i);
        sender_to_kernel.send(BrokerEvent::IncommingCloudEvent(
            id.clone(),
            CloudEvent {
                id: format!("{}", i),
                event_type: String::from("sequence-generator.counter"),
                spec_version: String::from("1.0"),
                time: Some(Utc::now().naive_utc()),
                source: String::from("dummy.sequence-generator"),
                subject: None,
                data_schema: None,
                data_content_type: Some(String::from("text/plain")),
                data: Data::String(format!("sequence {}", i)),
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
