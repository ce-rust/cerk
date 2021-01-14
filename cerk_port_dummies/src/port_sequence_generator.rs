use anyhow::{Context, Result};
use cerk::kernel::{
    BrokerEvent, CloudEventRoutingArgs, Config, ConfigHelpers, DeliveryGuarantee,
    IncomingCloudEvent, ProcessingResult,
};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::{InternalServerFn, InternalServerFnRefStatic, InternalServerId};
use chrono::Utc;
use cloudevents::{Event, EventBuilder, EventBuilderV10};
use std::convert::TryFrom;
use std::env;
use std::option::Option;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

type ArcSequenceGenData = Arc<Mutex<SequenceGeneratorData>>;

const DEFAULT_SLEEP_MS: u64 = 1000;
pub const CLOUD_EVENT_TYPE: &'static str = "sequence-generator.counter";

/// The maximal amount of allowed unacked message.
/// When this amount exceeds no new messages will be sent.
/// The channel to the kernel has a size of 50 so it should be smaller then that.
const DEFAULT_UNACK_MAX_COUNT: usize = 30;

struct SequenceGeneratorData {
    config: Option<SequenceGeneratorConfig>,
    missing_deliveries: Vec<String>,
}

struct SequenceGeneratorConfig {
    sleep_between_messages: Duration,
    amount: Option<u32>,
    delivery_guarantee: DeliveryGuarantee,
    unack_max_count: usize,
}

impl Default for SequenceGeneratorConfig {
    fn default() -> Self {
        SequenceGeneratorConfig {
            sleep_between_messages: Duration::from_secs(1),
            amount: None,
            delivery_guarantee: DeliveryGuarantee::BestEffort,
            unack_max_count: DEFAULT_UNACK_MAX_COUNT,
        }
    }
}

macro_rules! get_config {
    ($data:expr, $field:tt) => {
        $data
            .lock()
            .map_err(|e| anyhow!("failed to acquire data: {:?}", e))?
            .config
            .as_ref()
            .ok_or(anyhow!("failed to get config"))?
            .$field
    };
}

fn get_sleep_between_messages() -> Result<Duration> {
    Ok(Duration::from_millis(
        if let Ok(time) = env::var("GENERATOR_SLEEP_MS") {
            match time.parse() {
                Ok(time) => time,
                Err(e) => bail!("failed to parse GENERATOR_SLEEP_MS {:?}", e),
            }
        } else {
            DEFAULT_SLEEP_MS
        },
    ))
}

fn get_amount() -> Result<Option<u32>> {
    if let Ok(amount) = env::var("GENERATOR_AMOUNT") {
        Ok(Some(
            amount.parse().context("failed to parse GENERATOR_AMOUNT")?,
        ))
    } else {
        Ok(None)
    }
}

fn build_config(_id: &InternalServerId, config: &Config) -> Result<SequenceGeneratorConfig> {
    Ok(SequenceGeneratorConfig {
        sleep_between_messages: get_sleep_between_messages()?,
        amount: get_amount()?,
        delivery_guarantee: get_delivery_guarantee(config)?,
        unack_max_count: DEFAULT_UNACK_MAX_COUNT,
    })
}

fn get_delivery_guarantee(config: &Config) -> Result<DeliveryGuarantee> {
    if let Some(c) = config.get_op_val_config("delivery_guarantee")? {
        DeliveryGuarantee::try_from(c)
    } else {
        Ok(DeliveryGuarantee::default())
    }
}

fn send_events(
    id: &InternalServerId,
    sender_to_kernel: &BoxedSender,
    data: ArcSequenceGenData,
) -> Result<()> {
    let amount = get_config!(data, amount);
    if let Some(amount) = amount {
        for i in 1..=amount {
            send_event_and_track(id, sender_to_kernel, i, &data)?;
        }
    } else {
        for i in 1.. {
            send_event_and_track(id, sender_to_kernel, i, &data)?;
        }
    }
    wait_until_delivered(id, &data, 0)?;
    info!("{} finished generating events!", &id);
    Ok(())
}

fn send_event_and_track(
    id: &String,
    sender_to_kernel: &BoxedSender,
    i: u32,
    data: &ArcSequenceGenData,
) -> Result<()> {
    let unack_max_count = get_config!(data, unack_max_count);
    let delivery_guarantee = get_config!(data, delivery_guarantee);
    let sleep_between_messages = get_config!(data, sleep_between_messages);
    wait_until_delivered(id, data, unack_max_count)?;
    data.lock()
        .as_mut()
        .unwrap()
        .missing_deliveries
        .push(format!("{}", i));
    send_event(id, sender_to_kernel, i, delivery_guarantee);
    thread::sleep(sleep_between_messages.clone());
    Ok(())
}

fn wait_until_delivered(
    id: &String,
    data: &Arc<Mutex<SequenceGeneratorData>>,
    unack_max_count: usize,
) -> Result<()> {
    let delivery_guarantee = get_config!(data, delivery_guarantee);
    while delivery_guarantee.requires_acknowledgment()
        && data.lock().unwrap().missing_deliveries.len() >= unack_max_count
    {
        warn!("{} received unack_max_count - wait with resending", id);
        thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}

fn send_event(
    id: &String,
    sender_to_kernel: &BoxedSender,
    i: u32,
    delivery_guarantee: DeliveryGuarantee,
) {
    debug!("send dummy event with sequence number {} to kernel", i);

    sender_to_kernel.send(BrokerEvent::IncomingCloudEvent(IncomingCloudEvent {
        routing_id: i.clone().to_string(),
        incoming_id: id.clone(),
        cloud_event: generate_sequence_event(i),
        args: CloudEventRoutingArgs { delivery_guarantee },
    }));
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
/// * [Hello World](https://github.com/ce-rust/cerk/tree/master/examples/examples/src/hello_world)
/// * [Hello World Reliable](https://github.com/ce-rust/cerk/tree/master/examples/examples/src/hello_world_reliable)
/// * [Generator to MQTT](https://github.com/ce-rust/cerk/tree/master/examples/examples/src/mqtt/)
///
pub fn port_sequence_generator_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start sequence generator port with id {}", id);
    let tokio = tokio::runtime::Runtime::new().unwrap();
    let data = SequenceGeneratorData {
        config: None,
        missing_deliveries: vec![],
    };
    let data: ArcSequenceGenData = Arc::new(Mutex::new(data));
    loop {
        match inbox.receive() {
            BrokerEvent::Init => (),
            BrokerEvent::ConfigUpdated(config, _) => {
                data.lock().as_mut().unwrap().config = Some(match build_config(&id, &config) {
                    Err(e) => {
                        info!(
                            "was not able to read config -> will fallback to default; read error: {:?}",
                            e
                        );
                        SequenceGeneratorConfig::default()
                    }
                    Ok(settings) => settings,
                });
                info!("{} start generating events", &id);
                let data = data.clone();
                let id = id.clone();
                let sender_to_kernel = sender_to_kernel.clone_boxed();
                tokio.spawn(async move {
                    if let Err(e) = send_events(&id, &sender_to_kernel, data) {
                        error!("failed to generate sequence: {:?}", e)
                    }
                });
            }
            BrokerEvent::IncomingCloudEventProcessed(routing_id, result) => {
                if let Err(e) = process_incoming_event_result(
                    &id,
                    &sender_to_kernel,
                    data.clone(),
                    routing_id,
                    result,
                ) {
                    error!("failed to process IncomingCloudEventProcessed: {:?}", e);
                }
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

fn process_incoming_event_result(
    id: &String,
    sender_to_kernel: &BoxedSender,
    data: Arc<Mutex<SequenceGeneratorData>>,
    routing_id: String,
    result: ProcessingResult,
) -> Result<()> {
    let delivery_guarantee = get_config!(data, delivery_guarantee);
    if delivery_guarantee.requires_acknowledgment() {
        let idx = data
            .lock()
            .unwrap()
            .missing_deliveries
            .iter()
            .position(|e| *e == routing_id);
        if let Some(idx) = idx {
            match result {
                ProcessingResult::Successful => {
                    data.lock().unwrap().missing_deliveries.remove(idx);
                }
                ProcessingResult::PermanentError
                | ProcessingResult::TransientError
                | ProcessingResult::Timeout => {
                    // just resend it with a delay
                    thread::sleep(Duration::from_millis(10));
                    // the routing id is just the sequence id
                    send_event(
                        &id,
                        &sender_to_kernel,
                        routing_id.parse().unwrap(),
                        delivery_guarantee,
                    );
                }
            }
        }
    }
    Ok(())
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
