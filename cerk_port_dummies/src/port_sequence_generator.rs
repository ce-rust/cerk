use cerk::kernel::BrokerEvent;
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cloudevents::Data;
use cloudevents::CloudEvent;
use std::{thread, time};

fn generate_events(id: InternalServerId, sender_to_kernel: BoxedSender) {
    for i in 1.. {
        debug!("send dummy event with sequence number {} to kernel", i);

        let cloudevent = cloudevent_v1_0!(
            event_id: format!("{}", i),
            event_type: "sequence-generator.counter",
            time: "now",
            source: "dummy.sequence-generator",
            datacontenttype: "text/plain",
            data: Data::StringOrBinary(format!("sequence {}", i)),
        )
        .unwrap();

        sender_to_kernel.send(BrokerEvent::IncommingCloudEvent(id.clone(), CloudEvent::V1_0(cloudevent)));
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
