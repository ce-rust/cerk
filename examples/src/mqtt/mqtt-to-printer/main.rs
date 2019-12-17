#[macro_use]
extern crate log;
use env_logger::Env;

use cerk::kernel::{bootstrap, BrokerEvent, Config, StartOptions};
use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
use cerk::runtime::InternalServerId;
use cerk_port_dummies::port_printer_start;
use cerk_port_mqtt::port_mqtt_start;
use cerk_router_broadcast::router_start;
use cerk_runtime_threading::threading_scheduler_start;
use std::collections::HashMap;
use std::env;

const MQTT_INPUT: &'static str = "mqtt-input";
const DUMMY_LOGGER_OUTPUT: &'static str = "dummy-logger-output";

fn static_config_loader_start(
    id: InternalServerId,
    inbox: BoxedReceiver,
    sender_to_kernel: BoxedSender,
) {
    info!("start static config loader with id {}", id);
    let mqtt_broker_url: String =
        env::var("MQTT_BROKER_URL").unwrap_or(String::from("tcp://mqtt-broker:1883"));
    let mqtt_in_config: HashMap<String, Config> = [
        ("host".to_string(), Config::String(mqtt_broker_url)),
        (
            "subscribe_topics".to_string(),
            Config::Vec(vec![Config::String("test".to_string())]),
        ),
        (
            "subscribe_qos".to_string(),
            Config::Vec(vec![Config::U8(0)]),
        ),
    ]
    .iter()
    .cloned()
    .collect();

    loop {
        match inbox.receive() {
            BrokerEvent::Init => {
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::Vec(vec![Config::String(String::from(DUMMY_LOGGER_OUTPUT))]),
                    String::from("router"),
                ));
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::HashMap(mqtt_in_config.clone()),
                    String::from(MQTT_INPUT),
                ));
                sender_to_kernel.send(BrokerEvent::ConfigUpdated(
                    Config::Null,
                    String::from(DUMMY_LOGGER_OUTPUT),
                ));
            }
            broker_event => warn!("event {} not implemented", broker_event),
        }
    }
}

fn main() {
    env_logger::from_env(Env::default().default_filter_or("debug")).init();
    info!("start mqtt to printer router");
    let start_options = StartOptions {
        scheduler_start: threading_scheduler_start,
        router_start: router_start,
        config_loader_start: static_config_loader_start,
        ports: Box::new([
            (String::from(MQTT_INPUT), port_mqtt_start),
            (String::from(DUMMY_LOGGER_OUTPUT), port_printer_start),
        ]),
    };
    bootstrap(start_options);
}
