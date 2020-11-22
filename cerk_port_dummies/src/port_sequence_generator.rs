use cerk::kernel::{BrokerEvent, CloudEventRoutingArgs, IncomingCloudEvent};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cloudevents::Data;
use std::{thread, time};

fn generate_events(id: InternalServerId, sender_to_kernel: BoxedSender) {
    for i in 1.. {
        debug!("send dummy event with sequence number {} to kernel", i);

        let cloud_event = cloudevent!(
            event_id: format!("{}", i),
            event_type: "sequence-generator.counter",
            time: "now",
            source: "dummy.sequence-generator",
            datacontenttype: "text/plain",
            data: Data::StringOrBinary(format!("sequence {}", i)),
        )
        .unwrap();

        sender_to_kernel.send(BrokerEvent::IncomingCloudEvent(IncomingCloudEvent {
            routing_id: i.clone().to_string(),
            incoming_id: id.clone(),
            cloud_event,
            args: CloudEventRoutingArgs::default(),
        }));
        thread::sleep(time::Duration::from_secs(1));
    }
}

/// This port generates a CloudEvent every second and sends it to the Kernel.
/// This port is for testing!
///
/// # Examples
///
/// * [Hello World Example](https://github.com/ce-rust/cerk/tree/master/examples/src/hello_world)
/// * [Generator to MQTT](https://github.com/ce-rust/cerk/tree/master/examples/src/mqtt/)
///
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
