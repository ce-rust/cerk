use cerk::kernel::{BrokerEvent, CloudEventRoutingArgs, IncomingCloudEvent};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerFnRefStatic, InternalServerId};
use chrono::Utc;
use cloudevents::{EventBuilder, EventBuilderV10};
use std::env;
use std::thread;
use std::time::Duration;

const DEFAULT_SLEEP_MS: u64 = 1000;

fn generate_events(id: InternalServerId, sender_to_kernel: BoxedSender) {
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
                    generate_event(&id, &sender_to_kernel, i, &sleep);
                }
            }
            Err(e) => error!("failed to parse GENERATOR_AMOUNT {:?}", e),
        }
    } else {
        for i in 1.. {
            generate_event(&id, &sender_to_kernel, i, &sleep);
        }
    }
    info!("{} finished generating events!", &id)
}

fn generate_event(id: &String, sender_to_kernel: &BoxedSender, i: i32, sleep: &Duration) {
    debug!("send dummy event with sequence number {} to kernel", i);

    let cloud_event = EventBuilderV10::new()
        .id(format!("{}", i))
        .ty("sequence-generator.counter")
        .time(Utc::now())
        .source("http://example.com/dummy.sequence-generator")
        .data("text/plain", format!("sequence {}", i))
        .build()
        .unwrap();

    sender_to_kernel.send(BrokerEvent::IncomingCloudEvent(IncomingCloudEvent {
        routing_id: i.clone().to_string(),
        incoming_id: id.clone(),
        cloud_event,
        args: CloudEventRoutingArgs::default(),
    }));
    thread::sleep(sleep.clone());
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
            BrokerEvent::Init => break,
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
    generate_events(id, sender_to_kernel);
}

/// This is the pointer for the main function to start the port.
pub static PORT_SEQUENCE_GENERATOR: InternalServerFnRefStatic =
    &(port_sequence_generator_start as InternalServerFn);
