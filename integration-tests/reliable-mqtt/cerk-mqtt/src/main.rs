#[macro_use]
extern crate log;

use env_logger::Env;

use cerk::kernel::{bootstrap, BrokerEvent, Config, StartOptions};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk_runtime_threading::threading_scheduler_start;
use cerk_router_broadcast::router_start;
use cerk::runtime::InternalServerId;
use cerk_port_mqtt::port_mqtt_start;
use std::collections::HashMap;
use std::env;

const INBOX_PORT: &'static str = "inbox";
const OUTBOX_PORT: &'static str = "outbox";
const FAILING_PORT: &'static str = "failing";

fn static_config_loader_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start static config loader with id {}", id);
    let inbox_config: HashMap<String, Config> = [
        ("host".to_string(), Config::String(String::from("tcp://unlimited:1883"))),
        (
            "subscribe_topics".to_string(),
            Config::Vec(vec![Config::String("inbox".to_string())]),
        ),
        (
            "subscribe_qos".to_string(),
            Config::Vec(vec![Config::U8(1)]),
        ),
    ]
    .iter()
    .cloned()
    .collect();

    let outbox_config: HashMap<String, Config> = [
        ("host".to_string(), Config::String(String::from("tcp://unlimited:1883"))),
        (
            "send_topic".to_string(),
            Config::String("outbox".to_string()),
        ),
        (
            "send_qos".to_string(),
            Config::U8(1),
        ),
    ]
    .iter()
    .cloned()
    .collect();

    let failing_config: HashMap<String, Config> = [
        ("host".to_string(), Config::String(String::from("tcp://limited:1883"))),
        (
            "send_topic".to_string(),
            Config::String("outbox".to_string()),
        ),
        (
            "send_qos".to_string(),
            Config::U8(1),
        ),
    ]
    .iter()
    .cloned()
    .collect();

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::Vec(vec![
                        Config::String(String::from(OUTBOX_PORT)),
                        Config::String(String::from(FAILING_PORT))
                    ]),
                    String::from("router"),
                ));
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::HashMap(inbox_config.clone()),
                    String::from(INBOX_PORT),
                ));
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::HashMap(outbox_config.clone()),
                    String::from(OUTBOX_PORT),
                ));
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::HashMap(failing_config.clone()),
                    String::from(OUTBOX_PORT),
                ));
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

fn main() {
    env_logger::from_env(Env::default().default_filter_or("debug")).init();
    info!("start amqp to printer router");
    let start_options = StartOptions {
        scheduler_start: threading_scheduler_start,
        router_start: router_start,
        config_loader_start: static_config_loader_start,
        ports: Box::new([
            (String::from(INBOX_PORT), port_mqtt_start),
            (String::from(OUTBOX_PORT), port_mqtt_start),
            (String::from(FAILING_PORT), port_mqtt_start),
        ]),
    };
    bootstrap(start_options);
}
