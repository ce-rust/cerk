use cerk::kernel::{BrokerEvent, CloudEventRoutingArgs, IncomingCloudEvent};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerFnRefStatic, InternalServerId};
use chrono::Utc;
use cloudevents::{Event, EventBuilder, EventBuilderV10};
use std::env;
use std::thread;
use std::time::Duration;

const DEFAULT_SLEEP_MS: u64 = 1000;
pub const CLOUD_EVENT_TYPE: &'static str = "sequence-generator.counter";

fn send_events(id: &InternalServerId, sender_to_kernel: &BoxedSender) {
    let sleep = Duration::from_millis(if let Ok(time) = env::var("GENERATOR_SLEEP_MS") {
        match time.parse() {
            Ok(time) => time,
            Err(e) => {
                error!(
                    "failed to parse GENERATOR_SLEEP_MS {:?} -> using default",
                    e
                );
                DEFAULT_SLEEP_MS
            }
        }
    } else {
        DEFAULT_SLEEP_MS
    });

    if let Ok(amount) = env::var("GENERATOR_AMOUNT") {
        match amount.parse() {
            Ok(amount) => {
                for i in 1..=amount {
                    send_event(id, sender_to_kernel, i, &sleep);
                }
            }
            Err(e) => error!("failed to parse GENERATOR_AMOUNT {:?}", e),
        }
    } else {
        for i in 1.. {
            send_event(id, sender_to_kernel, i, &sleep);
        }
    }
    info!("{} finished generating events!", &id)
}

fn send_event(id: &String, sender_to_kernel: &BoxedSender, i: u32, sleep: &Duration) {
    debug!("send dummy event with sequence number {} to kernel", i);

    sender_to_kernel.send(BrokerEvent::IncomingCloudEvent(IncomingCloudEvent {
        routing_id: i.clone().to_string(),
        incoming_id: id.clone(),
        cloud_event: generate_sequence_event(i),
        args: CloudEventRoutingArgs::default(),
    }));
    thread::sleep(sleep.clone());
}

pub fn generate_sequence_event(i: u32) -> Event {
    EventBuilderV10::new()
        .id(format!("{}", i))
        .ty(CLOUD_EVENT_TYPE)
        .time(Utc::now())
        .source("http://example.com/dummy.sequence-generator")
        .data("text/plain", format!("{}", i))
        .build()
        .unwrap()
}

/// This port generates a CloudEvent every second (by default) and sends it to the Kernel.
/// This port is for testing!
///
/// # Env Options
///
/// * `GENERATOR_SLEEP_MS` define the sleep time between 2 events
/// * `GENERATOR_AMOUNT` define the total amount of events that should be generated
///
/// # Examples
///
/// * [Hello World Example](https://github.com/ce-rust/cerk/tree/master/examples/src/hello_world)
/// * [Generator to MQTT](https://github.com/ce-rust/cerk/tree/master/examples/src/mqtt/)
///
/// ## Limitations
///
/// * **reliability** this port does not support any `DeliveryGuarantee` and so does never resend an unprocessed event.
///
pub fn port_sequence_generator_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start sequence generator port with id {}", id);
    loop {
        match inbox.receive() {
            BrokerEvent::Init => (),
            BrokerEvent::ConfigUpdated(_, _) => {
                info!("{} received ConfigUpdated -> start generating events", &id);
                send_events(&id, &sender_to_kernel);
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

/// This is the pointer for the main function to start the port.
pub static PORT_SEQUENCE_GENERATOR: InternalServerFnRefStatic =
    &(port_sequence_generator_start as InternalServerFn);

#[cfg(test)]
mod test {
    use super::*;
    use cloudevents::AttributesReader;

    #[test]
    fn generate_event() {
        let event = generate_sequence_event(1);
        assert_eq!(event.ty(), CLOUD_EVENT_TYPE);
    }
}
